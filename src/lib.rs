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

pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action], rng: &mut dyn RngCore) -> Action;
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
        let mut h1_actual = Vec::new();
        let mut h2_actual = Vec::new();
        let mut h1_perceived_by_2 = Vec::new();
        let mut h2_perceived_by_1 = Vec::new();
        
        let mut score1 = 0;
        let mut score2 = 0;
        let mut history = Vec::new();

        let mut rng = match match_seed.or(self.seed) {
            Some(s) => ChaCha8Rng::seed_from_u64(s),
            None => ChaCha8Rng::from_os_rng(),
        };

        for _ in 0..self.iterations {
            if self.discount_factor > 0.0 && rng.random_bool(self.discount_factor) {
                break;
            }

            let m1 = s1.next_move(&h1_actual, &h2_perceived_by_1, &mut rng);
            let m2 = s2.next_move(&h2_actual, &h1_perceived_by_2, &mut rng);

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
            h1_perceived_by_2.push(m1_perceived);
            h2_perceived_by_1.push(m2_perceived);
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
    pub grid: Vec<Vec<Box<dyn Strategy>>>,
    pub game: Game,
}

impl SpatialTournament {
    pub fn new(width: usize, height: usize, strategies: Vec<Box<dyn Strategy>>, game: Game) -> Self {
        let mut rng = ChaCha8Rng::from_os_rng();
        let mut grid = Vec::new();
        for _ in 0..height {
            let mut row = Vec::new();
            for _ in 0..width {
                let idx = rng.random_range(0..strategies.len());
                row.push(strategies[idx].clone());
            }
            grid.push(row);
        }
        Self { grid, game }
    }

    pub fn step(&mut self) {
        let height = self.grid.len();
        let width = self.grid[0].len();
        
        // Calculate scores for each cell
        let mut scores = vec![vec![0; width]; height];
        
        for y in 0..height {
            for x in 0..width {
                let s1 = &self.grid[y][x];
                let mut cell_score = 0;
                
                // Moore neighborhood (8 neighbors)
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dy == 0 && dx == 0 { continue; }
                        let ny = (y as isize + dy).rem_euclid(height as isize) as usize;
                        let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                        let s2 = &self.grid[ny][nx];
                        let (sc1, _, _) = self.game.play(s1.as_ref(), s2.as_ref(), None);
                        cell_score += sc1;
                    }
                }
                scores[y][x] = cell_score;
            }
        }
        
        // Update grid
        let mut new_grid = Vec::new();
        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                let mut best_score = scores[y][x];
                let mut best_strat = self.grid[y][x].clone();
                
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let ny = (y as isize + dy).rem_euclid(height as isize) as usize;
                        let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                        if scores[ny][nx] > best_score {
                            best_score = scores[ny][nx];
                            best_strat = self.grid[ny][nx].clone();
                        }
                    }
                }
                row.push(best_strat);
            }
            new_grid.push(row);
        }
        self.grid = new_grid;
    }
    
    pub fn get_population_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for row in &self.grid {
            for cell in row {
                *counts.entry(cell.name().to_string()).or_insert(0) += 1;
            }
        }
        counts
    }
}
