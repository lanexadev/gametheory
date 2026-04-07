use std::collections::HashMap;
use rand::{Rng, SeedableRng};
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
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action]) -> Action;
    fn clone_box(&self) -> Box<dyn Strategy>;
}

pub struct FunctionalStrategy<F>
where F: Fn(&[Action], &[Action]) -> Action + Send + Sync + Clone + 'static {
    pub name: String,
    pub next_move_fn: F,
}

impl<F> Strategy for FunctionalStrategy<F>
where F: Fn(&[Action], &[Action]) -> Action + Send + Sync + Clone + 'static {
    fn name(&self) -> &str { &self.name }
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action]) -> Action {
        (self.next_move_fn)(my_history, opponent_history)
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

            let m1 = s1.next_move(&h1_actual, &h2_perceived_by_1);
            let m2 = s2.next_move(&h2_actual, &h1_perceived_by_2);

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
}

impl Tournament {
    pub fn new(strategies: Vec<Box<dyn Strategy>>, game: Game) -> Self {
        Self { strategies, game }
    }

    pub fn run_round_robin(&self) -> HashMap<String, i32> {
        let n = self.strategies.len();
        let mut pairs = Vec::new();
        for i in 0..n {
            for j in i..n {
                pairs.push((i, j));
            }
        }

        let results: Vec<_> = pairs.into_par_iter().map(|(i, j)| {
            let s1 = &self.strategies[i];
            let s2 = &self.strategies[j];
            let match_seed = self.game.seed.map(|s| s.wrapping_add((i * n + j) as u64));
            let (sc1, sc2, _) = self.game.play(s1.as_ref(), s2.as_ref(), match_seed);
            (i, j, sc1, sc2)
        }).collect();

        let mut scores = HashMap::new();
        for s in &self.strategies {
            scores.insert(s.name().to_string(), 0);
        }

        for (i, j, sc1, sc2) in results {
            let name1 = self.strategies[i].name();
            let name2 = self.strategies[j].name();
            *scores.get_mut(name1).unwrap() += sc1;
            if i != j {
                *scores.get_mut(name2).unwrap() += sc2;
            }
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
            // Record current population counts
            let mut counts = HashMap::new();
            for s in &self.strategies {
                *counts.entry(s.name().to_string()).or_insert(0) += 1;
            }
            history.push(counts);

            let scores = self.run_round_robin();
            let mut sorted_scores: Vec<_> = scores.into_iter().collect();
            sorted_scores.sort_by(|a, b| b.1.cmp(&a.1));
            
            let pop_size = self.strategies.len();
            let keep_count = (pop_size as f64 * (1.0 - reproduction_rate)) as usize;
            
            let mut next_gen = Vec::new();
            for i in 0..keep_count {
                let name = &sorted_scores[i].0;
                let strat = self.strategies.iter().find(|s| s.name() == name).unwrap();
                next_gen.push(strat.clone());
            }
            
            let replace_count = pop_size - keep_count;
            for i in 0..replace_count {
                let name = &sorted_scores[i % keep_count.max(1)].0;
                let strat = self.strategies.iter().find(|s| s.name() == name).unwrap();
                next_gen.push(strat.clone());
            }
            
            self.strategies = next_gen;
        }
        
        // Final counts for last gen
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
