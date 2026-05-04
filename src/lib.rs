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
#[derive(Clone, Default, Debug)]
pub enum StrategyScratch {
    #[default]
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

    /// Per-individual fitness: average score per turn, averaged over `match_repetitions`.
    /// This normalises away both noise variance (#5) and discount-factor duration variance (#7)
    /// so that selection pressure tracks intrinsic quality rather than match length.
    pub fn run_round_robin_per_individual(&self) -> Vec<f64> {
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

        sums.into_iter().zip(counts).map(|(s, c)| {
            if c > 0 { s / c as f64 } else { 0.0 }
        }).collect()
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

    pub fn run_swiss(&self, rounds: usize) -> HashMap<String, i32> {
        let mut scores: Vec<(i32, Box<dyn Strategy>)> = self.strategies.iter().map(|s| (0, s.clone())).collect();

        for r in 0..rounds {
            scores.sort_by(|a, b| b.0.cmp(&a.0));
            
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
                let (sc1, sc2, _) = self.game.play(s1.as_ref(), s2.as_ref(), match_seed);
                (i, j, sc1, sc2)
            }).collect();

            for (i, j, sc1, sc2) in results {
                scores[i].0 += sc1;
                scores[j].0 += sc2;
            }
        }

        scores.into_iter().map(|(s, strat)| (strat.name().to_string(), s)).collect()
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

    pub fn run_evolution(&mut self, generations: usize, reproduction_rate: f64) -> (HashMap<String, i32>, Vec<HashMap<String, usize>>) {
        let mut history = Vec::new();

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
            let per_individual = self.run_round_robin_per_individual();
            let mut ranked: Vec<(usize, f64)> = per_individual
                .into_iter()
                .enumerate()
                .collect();
            ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let pop_size = self.strategies.len();
            let keep_count = (pop_size as f64 * (1.0 - reproduction_rate)) as usize;
            let keep_count = keep_count.max(1).min(pop_size);

            let mut next_gen = Vec::with_capacity(pop_size);
            for k in 0..keep_count {
                let idx = ranked[k].0;
                next_gen.push(self.strategies[idx].clone());
            }

            let replace_count = pop_size - keep_count;
            for k in 0..replace_count {
                let idx = ranked[k % keep_count].0;
                next_gen.push(self.strategies[idx].clone());
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
}

impl SpatialTournament {
    pub fn new(width: usize, height: usize, strategies: Vec<Box<dyn Strategy>>, game: Game) -> Self {
        let mut rng = ChaCha8Rng::from_os_rng();
        let pool = strategies;
        let mut grid = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            grid.push(rng.random_range(0..pool.len()));
        }
        Self { grid, width, height, pool, game }
    }

    #[inline]
    fn cell(&self, y: usize, x: usize) -> usize {
        self.grid[y * self.width + x]
    }

    pub fn step(&mut self) {
        let width = self.width;
        let height = self.height;
        let total = width * height;

        // Score pass: each cell's payoff is independent, parallelise over the
        // flat index space. Reading `self.grid` and `self.pool` by shared
        // reference is safe because `Box<dyn Strategy>` is `Send + Sync`.
        let scores: Vec<i32> = (0..total).into_par_iter().map(|idx| {
            let y = idx / width;
            let x = idx % width;
            let s1 = self.pool[self.cell(y, x)].as_ref();
            let mut cell_score = 0;
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    if dy == 0 && dx == 0 { continue; }
                    let ny = (y as isize + dy as isize).rem_euclid(height as isize) as usize;
                    let nx = (x as isize + dx as isize).rem_euclid(width as isize) as usize;
                    let s2 = self.pool[self.cell(ny, nx)].as_ref();
                    let (sc1, _, _) = self.game.play(s1, s2, None);
                    cell_score += sc1;
                }
            }
            cell_score
        }).collect();

        // Update pass: each cell adopts the highest-scoring Moore neighbour
        // (or stays put). Independent per cell, parallelisable, and we copy
        // a single `usize` instead of cloning a strategy box.
        let new_grid: Vec<usize> = (0..total).into_par_iter().map(|idx| {
            let y = idx / width;
            let x = idx % width;
            let mut best_score = scores[idx];
            let mut best_idx = self.cell(y, x);
            for dy in -1..=1i32 {
                for dx in -1..=1i32 {
                    let ny = (y as isize + dy as isize).rem_euclid(height as isize) as usize;
                    let nx = (x as isize + dx as isize).rem_euclid(width as isize) as usize;
                    let nidx = ny * width + nx;
                    if scores[nidx] > best_score {
                        best_score = scores[nidx];
                        best_idx = self.cell(ny, nx);
                    }
                }
            }
            best_idx
        }).collect();

        self.grid = new_grid;
    }

    pub fn get_population_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for &idx in &self.grid {
            *counts.entry(self.pool[idx].name().to_string()).or_insert(0) += 1;
        }
        counts
    }
}
