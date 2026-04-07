use std::collections::HashMap;
use rand::Rng;

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

pub struct Game {
    pub iterations: usize,
    pub noise: f64, // Probability of flipping an action
}

impl Game {
    pub fn play(&self, s1: &dyn Strategy, s2: &dyn Strategy) -> (i32, i32, History) {
        let mut h1 = Vec::new();
        let mut h2 = Vec::new();
        let mut score1 = 0;
        let mut score2 = 0;
        let mut history = Vec::new();

        let mut rng = rand::rng();

        for _ in 0..self.iterations {
            let m1 = s1.next_move(&h1, &h2);
            let m2 = s2.next_move(&h2, &h1);

            // Apply noise
            let m1_actual = if rng.random_bool(self.noise) { m1.flip() } else { m1 };
            let m2_actual = if rng.random_bool(self.noise) { m2.flip() } else { m2 };

            let (p1, p2) = match (m1_actual, m2_actual) {
                (Action::Cooperate, Action::Cooperate) => (3, 3),
                (Action::Cooperate, Action::Defect) => (0, 5),
                (Action::Defect, Action::Cooperate) => (5, 0),
                (Action::Defect, Action::Defect) => (1, 1),
            };

            score1 += p1;
            score2 += p2;
            h1.push(m1_actual);
            h2.push(m2_actual);
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
    pub fn new(strategies: Vec<Box<dyn Strategy>>, iterations: usize, noise: f64) -> Self {
        Self {
            strategies,
            game: Game { iterations, noise },
        }
    }

    pub fn run_round_robin(&self) -> HashMap<String, i32> {
        let mut scores = HashMap::new();
        for s in &self.strategies {
            scores.insert(s.name().to_string(), 0);
        }

        for i in 0..self.strategies.len() {
            for j in i..self.strategies.len() {
                let s1 = &self.strategies[i];
                let s2 = &self.strategies[j];

                let (sc1, sc2, _) = self.game.play(s1.as_ref(), s2.as_ref());

                *scores.get_mut(s1.name()).unwrap() += sc1;
                if i != j {
                    *scores.get_mut(s2.name()).unwrap() += sc2;
                }
            }
        }
        scores
    }

    pub fn run_swiss(&self, rounds: usize) -> HashMap<String, i32> {
        let mut scores: Vec<(i32, Box<dyn Strategy>)> = self.strategies.iter().map(|s| (0, s.clone())).collect();

        for _ in 0..rounds {
            // Sort by score for pairing
            scores.sort_by(|a, b| b.0.cmp(&a.0));

            // Pair adjacent players (1-2, 3-4, etc.)
            for i in (0..scores.len()).step_by(2) {
                if i + 1 < scores.len() {
                    let (sc1, sc2, _) = {
                        let s1 = &scores[i].1;
                        let s2 = &scores[i+1].1;
                        self.game.play(s1.as_ref(), s2.as_ref())
                    };
                    scores[i].0 += sc1;
                    scores[i+1].0 += sc2;
                }
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

        // Finals is a Round Robin with more iterations
        let finals_game = Game { iterations: self.game.iterations * 5, noise: self.game.noise };
        let finals_tournament = Tournament::new(top_strats, finals_game.iterations, finals_game.noise);
        let final_results = finals_tournament.run_round_robin();

        final_results.into_iter().max_by_key(|&(_, score)| score).unwrap().0
    }
}
