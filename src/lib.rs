use std::any::Any;
use std::collections::HashMap;
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;

pub mod strategies;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Cooperate,
    Defect,
}

impl Action {
    pub fn flip(&self) -> Self {
        match self {
            Action::Cooperate => Action::Defect,
            Action::Defect => Action::Cooperate,
        }
    }
}

pub type History = Vec<(Action, Action)>;

/// Per-match scratchpad threaded through `next_move_stateful` so strategies
/// whose decision is a function of accumulated history can update their
/// state in O(1) per turn instead of rescanning the whole opponent history.
/// Strategies that don't need scratch use the default `None` and the default
/// `next_move_stateful` impl, which delegates to the stateless `next_move`.
///
/// `Custom` lets external strategies stash typed state (Q-tables, Bayesian
/// priors, deep RL inference handles, etc.) without editing this enum. The
/// owning strategy `downcast_mut` to its concrete state type.
pub enum StrategyScratch {
    None,
    /// Shared shape used by the punish/cooldown state machine of both
    /// `Gradual` (standalone) and the `Gradual (xN)` family. The strategy
    /// owns the transition rule; this scratch only stores the running
    /// counters and how many opponent-history entries we've already folded
    /// in.
    Gradual {
        opp_defects: usize,
        p_left: usize,
        c_left: usize,
        processed: usize,
    },
    /// Used by `OmegaTFT` (standalone) and the `Omega-Detector (Thresh N)`
    /// family. Counts the number of (my=C, opp=D) transitions seen so far
    /// and how many history pairs we've already folded in.
    OmegaDetector {
        inconsistencies: usize,
        processed: usize,
    },
    /// Open-ended slot for any new stateful strategy. The implementor owns
    /// downcasting via `state.downcast_mut::<MyState>()`.
    Custom(Box<dyn Any + Send>),
}

impl Default for StrategyScratch {
    fn default() -> Self { StrategyScratch::None }
}

pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action], rng: &mut dyn RngCore) -> Action;

    /// Allocate the per-match scratch for this strategy. Default: no scratch.
    /// Stateful strategies override this to seed their counters.
    fn init_scratch(&self) -> StrategyScratch { StrategyScratch::None }

    /// Stateful next-move call used by the engine. The default impl ignores
    /// the scratch and delegates to `next_move`, which keeps every existing
    /// strategy compatible without modification.
    fn next_move_stateful(
        &self,
        my_history: &[Action],
        opponent_history: &[Action],
        _scratch: &mut StrategyScratch,
        rng: &mut dyn RngCore,
    ) -> Action {
        self.next_move(my_history, opponent_history, rng)
    }

    fn clone_box(&self) -> Box<dyn Strategy>;
}

pub struct FunctionalStrategy<F>
where F: Fn(&[Action], &[Action], &mut dyn RngCore) -> Action + Send + Sync + Clone + 'static {
    pub name: String,
    pub next_move_fn: F,
}

impl<F> Strategy for FunctionalStrategy<F>
where F: Fn(&[Action], &[Action], &mut dyn RngCore) -> Action + Send + Sync + Clone + 'static {
    fn name(&self) -> &str { &self.name }
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action], rng: &mut dyn RngCore) -> Action {
        (self.next_move_fn)(my_history, opponent_history, rng)
    }
    fn clone_box(&self) -> Box<dyn Strategy> {
        Box::new(Self {
            name: self.name.clone(),
            next_move_fn: self.next_move_fn.clone(),
        })
    }
}

impl Clone for Box<dyn Strategy> {
    fn clone(&self) -> Box<dyn Strategy> {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct Game {
    pub iterations: usize,
    pub action_noise: f64,
    pub perception_noise: f64,
    pub discount_factor: f64,
    pub payoffs: (i32, i32, i32, i32), // T, R, P, S
    pub seed: Option<u64>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            iterations: 200,
            action_noise: 0.0,
            perception_noise: 0.0,
            discount_factor: 0.0,
            payoffs: (5, 3, 1, 0),
            seed: None,
        }
    }
}

impl Game {
    /// Validates Axelrod's IPD payoff constraints: T > R > P > S and 2R > T + S.
    /// The second constraint prevents an alternating C/D, D/C strategy from outperforming
    /// mutual cooperation — without it, the game is no longer a proper iterated dilemma.
    pub fn validate(&self) -> Result<(), String> {
        let (t, r, p, s) = self.payoffs;
        if !(t > r && r > p && p > s) {
            return Err(format!(
                "Axelrod payoffs must satisfy T > R > P > S (got T={}, R={}, P={}, S={})",
                t, r, p, s
            ));
        }
        if !(2 * r > t + s) {
            return Err(format!(
                "Axelrod payoffs must satisfy 2R > T + S (got R={}, T={}, S={}; otherwise alternating defection beats mutual cooperation)",
                r, t, s
            ));
        }
        Ok(())
    }

    pub fn play(&self, s1: &dyn Strategy, s2: &dyn Strategy, match_seed: Option<u64>) -> (i32, i32, History) {
        // Pre-size to avoid log(N) reallocations during a 200-1000 turn match.
        // When neither noise source is active, the perceived histories are
        // value-identical to the actual histories — skip allocating and
        // pushing to those two vectors, and pass the actuals as opponent
        // history. The `random_bool` calls below are still executed in that
        // case to keep the RNG sequence (and therefore reproducibility)
        // identical to the legacy code path.
        let no_noise = self.action_noise == 0.0 && self.perception_noise == 0.0;
        let cap = self.iterations;
        let mut h1_actual: Vec<Action> = Vec::with_capacity(cap);
        let mut h2_actual: Vec<Action> = Vec::with_capacity(cap);
        let mut h1_perceived_by_2: Vec<Action> = if no_noise { Vec::new() } else { Vec::with_capacity(cap) };
        let mut h2_perceived_by_1: Vec<Action> = if no_noise { Vec::new() } else { Vec::with_capacity(cap) };

        let mut score1 = 0;
        let mut score2 = 0;
        let mut history = Vec::with_capacity(cap);

        let mut rng = match match_seed.or(self.seed) {
            Some(s) => ChaCha8Rng::seed_from_u64(s),
            None => ChaCha8Rng::from_os_rng(),
        };

        let mut scratch1 = s1.init_scratch();
        let mut scratch2 = s2.init_scratch();

        for _ in 0..self.iterations {
            if self.discount_factor > 0.0 && rng.random_bool(self.discount_factor) {
                break;
            }

            let opp_for_1: &[Action] = if no_noise { &h2_actual } else { &h2_perceived_by_1 };
            let opp_for_2: &[Action] = if no_noise { &h1_actual } else { &h1_perceived_by_2 };
            let m1 = s1.next_move_stateful(&h1_actual, opp_for_1, &mut scratch1, &mut rng);
            let m2 = s2.next_move_stateful(&h2_actual, opp_for_2, &mut scratch2, &mut rng);

            let m1_actual = if rng.random_bool(self.action_noise) { m1.flip() } else { m1 };
            let m2_actual = if rng.random_bool(self.action_noise) { m2.flip() } else { m2 };

            let m1_perceived = if rng.random_bool(self.perception_noise) { m1_actual.flip() } else { m1_actual };
            let m2_perceived = if rng.random_bool(self.perception_noise) { m2_actual.flip() } else { m2_actual };

            let (t, r, p, s) = self.payoffs;
            let (p1, p2) = match (m1_actual, m2_actual) {
                (Action::Cooperate, Action::Cooperate) => (r, r),
                (Action::Cooperate, Action::Defect) => (s, t),
                (Action::Defect, Action::Cooperate) => (t, s),
                (Action::Defect, Action::Defect) => (p, p),
            };

            score1 += p1;
            score2 += p2;

            h1_actual.push(m1_actual);
            h2_actual.push(m2_actual);
            if !no_noise {
                h1_perceived_by_2.push(m1_perceived);
                h2_perceived_by_1.push(m2_perceived);
            }
            history.push((m1_actual, m2_actual));
        }

        (score1, score2, history)
    }
}

/// Output of `Tournament::run_round_robin_report`. Carries the per-individual
/// fitness (used by evolution and display) AND the full N×N pair-score matrix
/// (used by `--export-matrix` for richer offline analysis).
#[derive(Debug, Clone)]
pub struct RoundRobinReport {
    /// `fitness[i]` = mean score-per-turn of individual `i` over all its matches.
    pub fitness: Vec<f64>,
    /// `matrix[i][j]` = mean score-per-turn of `i` playing against `j`.
    /// Off-diagonal cells where the pair never played (e.g., `include_self_play=false`
    /// keeps the diagonal at 0.0) stay at 0.0.
    pub matrix: Vec<Vec<f64>>,
    pub names: Vec<String>,
}

impl RoundRobinReport {
    /// Write the matrix as a square CSV: header row + each row prefixed with
    /// the strategy name. Cell `[i,j]` is `i`'s mean per-turn score vs `j`.
    pub fn export_matrix_csv(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(path)?;
        let mut header = vec!["Strategy".to_string()];
        for n in &self.names { header.push(n.clone()); }
        wtr.write_record(&header)?;
        for (i, row) in self.matrix.iter().enumerate() {
            let mut record = Vec::with_capacity(row.len() + 1);
            record.push(self.names[i].clone());
            for cell in row {
                record.push(format!("{:.6}", cell));
            }
            wtr.write_record(&record)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

pub struct Tournament {
    pub strategies: Vec<Box<dyn Strategy>>,
    pub game: Game,
    /// Number of times each pair plays. Reduces noise variance; with seed,
    /// each repetition uses a deterministically-derived offset seed.
    pub match_repetitions: usize,
    /// Whether each strategy plays itself. Default true (Axelrod's original setup).
    pub include_self_play: bool,
}

impl Tournament {
    pub fn new(strategies: Vec<Box<dyn Strategy>>, game: Game) -> Self {
        Self {
            strategies,
            game,
            match_repetitions: 1,
            include_self_play: true,
        }
    }

    pub fn with_match_repetitions(mut self, n: usize) -> Self {
        self.match_repetitions = n.max(1);
        self
    }

    pub fn with_include_self_play(mut self, include: bool) -> Self {
        self.include_self_play = include;
        self
    }

    /// Full pair-wise round-robin report: per-individual mean fitness AND the
    /// N×N matrix of mean per-turn scores. Single source of truth — the lighter
    /// `run_round_robin_per_individual` and `run_round_robin` derive their
    /// outputs from this. Matrix cell `(i, j)` is the mean per-turn score
    /// strategy `i` obtained against strategy `j` (averaged over repetitions).
    pub fn run_round_robin_report(&self) -> RoundRobinReport {
        let n = self.strategies.len();
        let reps = self.match_repetitions.max(1);

        let mut pairs = Vec::new();
        for i in 0..n {
            let start_j = if self.include_self_play { i } else { i + 1 };
            for j in start_j..n {
                for rep in 0..reps {
                    pairs.push((i, j, rep));
                }
            }
        }

        let results: Vec<_> = pairs.into_par_iter().map(|(i, j, rep)| {
            let s1 = &self.strategies[i];
            let s2 = &self.strategies[j];
            let match_seed = self.game.seed.map(|s| {
                s.wrapping_add(((i * n + j) as u64).wrapping_mul(reps as u64))
                    .wrapping_add(rep as u64)
            });
            let (sc1, sc2, history) = self.game.play(s1.as_ref(), s2.as_ref(), match_seed);
            let turns = history.len().max(1) as f64;
            (i, j, sc1 as f64 / turns, sc2 as f64 / turns)
        }).collect();

        // matrix[i][j] = mean per-turn score of i playing against j.
        let mut matrix_sum: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
        let mut matrix_count: Vec<Vec<u32>> = vec![vec![0; n]; n];
        for &(i, j, sc1_per_turn, sc2_per_turn) in &results {
            matrix_sum[i][j] += sc1_per_turn;
            matrix_count[i][j] += 1;
            if i != j {
                matrix_sum[j][i] += sc2_per_turn;
                matrix_count[j][i] += 1;
            }
        }
        let matrix: Vec<Vec<f64>> = matrix_sum
            .into_iter()
            .zip(matrix_count)
            .map(|(row_sum, row_count)| {
                row_sum
                    .into_iter()
                    .zip(row_count)
                    .map(|(s, c)| if c > 0 { s / c as f64 } else { 0.0 })
                    .collect()
            })
            .collect();

        // Per-individual fitness: mean of all matches the individual played.
        let mut sums = vec![0f64; n];
        let mut counts = vec![0u32; n];
        for (i, j, sc1_per_turn, sc2_per_turn) in results {
            sums[i] += sc1_per_turn;
            counts[i] += 1;
            if i != j {
                sums[j] += sc2_per_turn;
                counts[j] += 1;
            }
        }
        let fitness: Vec<f64> = sums
            .into_iter()
            .zip(counts)
            .map(|(s, c)| if c > 0 { s / c as f64 } else { 0.0 })
            .collect();

        let names = self.strategies.iter().map(|s| s.name().to_string()).collect();
        RoundRobinReport { fitness, matrix, names }
    }

    /// Per-individual fitness: average score per turn, averaged over `match_repetitions`.
    /// Thin wrapper around `run_round_robin_report` for backward compatibility.
    pub fn run_round_robin_per_individual(&self) -> Vec<f64> {
        self.run_round_robin_report().fitness
    }

    /// Aggregated by strategy name, reprojected to an `iterations`-turn equivalent
    /// so display values stay comparable to legacy raw-sum runs. Display/CSV only —
    /// evolutionary selection must use `run_round_robin_per_individual`.
    pub fn run_round_robin(&self) -> HashMap<String, i32> {
        let per_individual = self.run_round_robin_per_individual();
        let scale = self.game.iterations as f64;
        let mut scores = HashMap::new();
        for (i, score_per_turn) in per_individual.into_iter().enumerate() {
            let projected = (score_per_turn * scale).round() as i32;
            *scores.entry(self.strategies[i].name().to_string()).or_insert(0i32) += projected;
        }
        scores
    }

    /// Swiss-system tournament. Per-turn normalised scoring (matches `run_round_robin`):
    /// each round's contribution is `score / turns_played`, accumulated over rounds, then
    /// projected back to an `iterations`-equivalent total for display compatibility.
    /// Without normalisation, `discount_factor > 0` would silently demote strategies
    /// whose matches end early.
    pub fn run_swiss(&self, rounds: usize) -> HashMap<String, i32> {
        let mut scores: Vec<(f64, Box<dyn Strategy>)> = self.strategies.iter()
            .map(|s| (0.0f64, s.clone()))
            .collect();

        for r in 0..rounds {
            scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

            // Extract pairs sequentially to handle mutability
            let mut pairs = Vec::new();
            for i in (0..scores.len()).step_by(2) {
                if i + 1 < scores.len() {
                    pairs.push((i, i + 1));
                }
            }

            let results: Vec<_> = pairs.into_par_iter().map(|(i, j)| {
                let s1 = &scores[i].1;
                let s2 = &scores[j].1;
                let match_seed = self.game.seed.map(|s| s.wrapping_add((r * scores.len() + i) as u64));
                let (sc1, sc2, history) = self.game.play(s1.as_ref(), s2.as_ref(), match_seed);
                let turns = history.len().max(1) as f64;
                (i, j, sc1 as f64 / turns, sc2 as f64 / turns)
            }).collect();

            for (i, j, sc1_per_turn, sc2_per_turn) in results {
                scores[i].0 += sc1_per_turn;
                scores[j].0 += sc2_per_turn;
            }
        }

        let scale = self.game.iterations as f64;
        scores
            .into_iter()
            .map(|(per_turn_sum, strat)| {
                (strat.name().to_string(), (per_turn_sum * scale).round() as i32)
            })
            .collect()
    }

    pub fn run_grand_finale(&self, top_n: usize) -> String {
        let scores = self.run_round_robin();
        let mut sorted_scores: Vec<_> = scores.into_iter().collect();
        sorted_scores.sort_by(|a, b| b.1.cmp(&a.1));

        let top_strats: Vec<_> = sorted_scores.into_iter().take(top_n).map(|(name, _)| {
            self.strategies.iter().find(|s| s.name() == name).unwrap().clone()
        }).collect();

        let mut finals_game = self.game.clone();
        finals_game.iterations *= 5;
        let finals_tournament = Tournament::new(top_strats, finals_game);
        let final_results = finals_tournament.run_round_robin();

        final_results.into_iter().max_by_key(|&(_, score)| score).unwrap().0
    }

    /// Legacy truncation-only evolution (kept for backward compatibility).
    /// New callers should prefer `run_evolution_with_options`.
    pub fn run_evolution(&mut self, generations: usize, reproduction_rate: f64) -> (HashMap<String, i32>, Vec<HashMap<String, usize>>) {
        self.run_evolution_with_options(generations, reproduction_rate, 0.0, 0.0, None)
    }

    /// Evolutionary tournament with three selection regimes, picked by params:
    ///
    /// - `selection_temperature == 0.0` → **truncation**: top-`keep_count` reproduce
    ///   identically (legacy behaviour). Deterministic given a seed.
    /// - `selection_temperature > 0.0`  → **softmax roulette wheel**: each parent is
    ///   sampled with probability proportional to `exp((fitness - max) / T)`. Higher T
    ///   = more diversity preserved; lower T → truncation in the limit.
    ///
    /// `mutation_rate` injects exploration: with probability `p_mut`, a child slot is
    /// replaced by a fresh sample from `mutation_pool` instead of a clone of the
    /// selected parent. Without this, the population is bounded forever by its
    /// initial set — no novelty can emerge. Pool defaults to the current population
    /// (effectively a permutation re-sample) when `None` is passed and mutation is on.
    ///
    /// All randomness is derived from `Game.seed` (when set) so runs are reproducible.
    pub fn run_evolution_with_options(
        &mut self,
        generations: usize,
        reproduction_rate: f64,
        mutation_rate: f64,
        selection_temperature: f64,
        mutation_pool: Option<Vec<Box<dyn Strategy>>>,
    ) -> (HashMap<String, i32>, Vec<HashMap<String, usize>>) {
        let mut history = Vec::new();
        let mut rng = match self.game.seed {
            Some(s) => ChaCha8Rng::seed_from_u64(s.wrapping_add(0xE_0_E_0_E_0_E_0)),
            None => ChaCha8Rng::from_os_rng(),
        };
        let mut_rate = mutation_rate.clamp(0.0, 1.0);
        let temp = selection_temperature.max(0.0);

        for _ in 0..generations {
            let mut counts = HashMap::new();
            for s in &self.strategies {
                *counts.entry(s.name().to_string()).or_insert(0) += 1;
            }
            history.push(counts);

            // Per-individual fitness: each clone is judged on its own matches, not on the
            // sum of every clone's score. Without this, a strategy present N times in the
            // population gets ~N× the score mechanically and outcompetes minorities
            // regardless of intrinsic quality.
            let fitness = self.run_round_robin_per_individual();
            let pop_size = self.strategies.len();
            let keep_count = (pop_size as f64 * (1.0 - reproduction_rate)) as usize;
            let keep_count = keep_count.max(1).min(pop_size);

            // Build parent index list according to selection regime.
            let parent_indices: Vec<usize> = if temp <= 0.0 {
                let mut ranked: Vec<(usize, f64)> = fitness
                    .iter()
                    .enumerate()
                    .map(|(i, &f)| (i, f))
                    .collect();
                ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let mut out = Vec::with_capacity(pop_size);
                for k in 0..keep_count {
                    out.push(ranked[k].0);
                }
                let replace_count = pop_size - keep_count;
                for k in 0..replace_count {
                    out.push(ranked[k % keep_count].0);
                }
                out
            } else {
                // Softmax over (fitness - max) / T to keep weights numerically stable.
                let max_f = fitness.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let weights: Vec<f64> = fitness
                    .iter()
                    .map(|&f| ((f - max_f) / temp).exp())
                    .collect();
                let total: f64 = weights.iter().sum();
                if total <= 0.0 || !total.is_finite() {
                    // Degenerate weights → fall back to uniform sampling.
                    (0..pop_size).map(|_| rng.random_range(0..pop_size)).collect()
                } else {
                    (0..pop_size)
                        .map(|_| {
                            let r: f64 = rng.random::<f64>() * total;
                            let mut acc = 0.0;
                            for (i, &w) in weights.iter().enumerate() {
                                acc += w;
                                if acc >= r {
                                    return i;
                                }
                            }
                            weights.len() - 1
                        })
                        .collect()
                }
            };

            let mut next_gen: Vec<Box<dyn Strategy>> = Vec::with_capacity(pop_size);
            for &idx in &parent_indices {
                if mut_rate > 0.0 && rng.random_bool(mut_rate) {
                    if let Some(ref pool) = mutation_pool {
                        if !pool.is_empty() {
                            let p = rng.random_range(0..pool.len());
                            next_gen.push(pool[p].clone_box());
                            continue;
                        }
                    }
                    // Pool absent → resample from current population (still injects
                    // recombination-like noise relative to pure truncation).
                    let p = rng.random_range(0..self.strategies.len());
                    next_gen.push(self.strategies[p].clone_box());
                    continue;
                }
                next_gen.push(self.strategies[idx].clone_box());
            }

            self.strategies = next_gen;
        }

        let mut final_counts = HashMap::new();
        for s in &self.strategies {
            *final_counts.entry(s.name().to_string()).or_insert(0) += 1;
        }
        history.push(final_counts);

        (self.run_round_robin(), history)
    }
}

/// Cellular-automaton neighborhood stencil. Each variant returns the offsets
/// `(dy, dx)` that define which cells count as neighbours. Hex uses axial
/// offset coordinates and depends on row parity (odd-row shift).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Neighborhood {
    /// 8-neighbor king-move (diagonals included). Backward-compatible default.
    Moore,
    /// 4-neighbor orthogonal (no diagonals).
    VonNeumann,
    /// 6-neighbor hex grid (offset coords; even/odd rows differ).
    Hex,
}

impl Neighborhood {
    /// Returns the static slice of `(dy, dx)` offsets defining the neighbourhood.
    /// `even_row` is only relevant for `Hex` — Moore/VonNeumann ignore it.
    pub fn offsets(self, even_row: bool) -> &'static [(i32, i32)] {
        const MOORE: [(i32, i32); 8] = [
            (-1, -1), (-1, 0), (-1, 1),
            ( 0, -1),          ( 0, 1),
            ( 1, -1), ( 1, 0), ( 1, 1),
        ];
        const VON_NEUMANN: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        // Axial offset hex: even rows shifted left, odd rows shifted right.
        const HEX_EVEN: [(i32, i32); 6] = [(-1, -1), (-1, 0), (0, -1), (0, 1), (1, -1), (1, 0)];
        const HEX_ODD:  [(i32, i32); 6] = [(-1, 0), (-1, 1), (0, -1), (0, 1), (1, 0), (1, 1)];
        match self {
            Neighborhood::Moore => &MOORE,
            Neighborhood::VonNeumann => &VON_NEUMANN,
            Neighborhood::Hex => if even_row { &HEX_EVEN } else { &HEX_ODD },
        }
    }
}

pub struct SpatialTournament {
    /// Flat row-major grid of indices into `pool`. Storing `usize` (8 bytes)
    /// instead of `Box<dyn Strategy>` (16 bytes + heap) makes the per-step
    /// "copy best neighbour" pass a trivial memcpy and lets us share the
    /// underlying strategy objects across cells without `clone_box()`.
    grid: Vec<usize>,
    width: usize,
    height: usize,
    /// Deduplicated strategy templates. Cells reference these by index.
    pool: Vec<Box<dyn Strategy>>,
    pub game: Game,
    pub topology: Neighborhood,
    /// Monotonic step counter mixed into per-match seeds so consecutive steps
    /// produce different RNG sequences. Without this, every (A vs B) match
    /// at the same neighbourhood offset would be replayed identically each
    /// step under noise, collapsing the spatial dynamic to a fixed point.
    step_count: u64,
}

impl SpatialTournament {
    pub fn new(width: usize, height: usize, strategies: Vec<Box<dyn Strategy>>, game: Game) -> Self {
        Self::new_with_topology(width, height, strategies, game, Neighborhood::Moore)
    }

    pub fn new_with_topology(
        width: usize,
        height: usize,
        strategies: Vec<Box<dyn Strategy>>,
        game: Game,
        topology: Neighborhood,
    ) -> Self {
        // Honour the Game seed when seeding the initial random grid layout —
        // otherwise spatial runs were silently non-reproducible even with --seed.
        let mut rng = match game.seed {
            Some(s) => ChaCha8Rng::seed_from_u64(s),
            None => ChaCha8Rng::from_os_rng(),
        };
        let pool = strategies;
        let mut grid = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            grid.push(rng.random_range(0..pool.len()));
        }
        Self { grid, width, height, pool, game, topology, step_count: 0 }
    }

    #[inline]
    fn cell(&self, y: usize, x: usize) -> usize {
        self.grid[y * self.width + x]
    }

    pub fn step(&mut self) {
        let width = self.width;
        let height = self.height;
        let total = width * height;
        let topology = self.topology;
        let step_id = self.step_count;
        let base_seed = self.game.seed;

        // Score pass: each cell's payoff is independent, parallelise over the
        // flat index space. Reading `self.grid` and `self.pool` by shared
        // reference is safe because `Box<dyn Strategy>` is `Send + Sync`.
        let scores: Vec<i32> = (0..total).into_par_iter().map(|idx| {
            let y = idx / width;
            let x = idx % width;
            let s1 = self.pool[self.cell(y, x)].as_ref();
            let mut cell_score = 0;
            for (n_idx, &(dy, dx)) in topology.offsets(y % 2 == 0).iter().enumerate() {
                let ny = (y as isize + dy as isize).rem_euclid(height as isize) as usize;
                let nx = (x as isize + dx as isize).rem_euclid(width as isize) as usize;
                let s2 = self.pool[self.cell(ny, nx)].as_ref();
                // Per-(step, cell, neighbour) seed derivation: without mixing
                // step_id, every match between the same pair at the same
                // offset would be replayed bit-identically each step.
                let match_seed = base_seed.map(|base| {
                    base.wrapping_add(step_id.wrapping_mul(1_000_003))
                        .wrapping_add((idx as u64).wrapping_mul(31))
                        .wrapping_add(n_idx as u64)
                });
                let (sc1, _, _) = self.game.play(s1, s2, match_seed);
                cell_score += sc1;
            }
            cell_score
        }).collect();

        // Update pass: each cell adopts the highest-scoring neighbour (or
        // stays put). Same stencil as the score pass.
        let new_grid: Vec<usize> = (0..total).into_par_iter().map(|idx| {
            let y = idx / width;
            let x = idx % width;
            let mut best_score = scores[idx];
            let mut best_idx = self.cell(y, x);
            for &(dy, dx) in topology.offsets(y % 2 == 0) {
                let ny = (y as isize + dy as isize).rem_euclid(height as isize) as usize;
                let nx = (x as isize + dx as isize).rem_euclid(width as isize) as usize;
                let nidx = ny * width + nx;
                if scores[nidx] > best_score {
                    best_score = scores[nidx];
                    best_idx = self.cell(ny, nx);
                }
            }
            best_idx
        }).collect();

        self.grid = new_grid;
        self.step_count = self.step_count.wrapping_add(1);
    }

    pub fn get_population_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for &idx in &self.grid {
            *counts.entry(self.pool[idx].name().to_string()).or_insert(0) += 1;
        }
        counts
    }
}
