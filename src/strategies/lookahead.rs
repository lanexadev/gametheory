//! Depth-limited minimax-against-fixed-model lookahead.
//!
//! Treats the opponent as a known model (any `Box<dyn Strategy>`) and searches
//! `depth` plies ahead, picking the action that maximises the discounted
//! expected payoff. Self-action branching is exhaustive (binary tree of size
//! `2^depth`), opponent branching is collapsed to its model's deterministic
//! response — so cost is O(2^depth · depth), trivial for `depth ≤ 6`.
//!
//! The model's `next_move` is called *stateless* (not stateful): rollouts
//! intentionally avoid mutating any scratch the model might own. This keeps
//! the search side-effect free and cheap to clone.

use crate::{Action, Strategy};
use rand::RngCore;

pub struct Lookahead {
    pub label: String,
    pub depth: usize,
    pub gamma: f64,
    pub payoffs: (f64, f64, f64, f64),
    pub opponent_model: Box<dyn Strategy>,
}

impl Clone for Lookahead {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            depth: self.depth,
            gamma: self.gamma,
            payoffs: self.payoffs,
            opponent_model: self.opponent_model.clone_box(),
        }
    }
}

impl Lookahead {
    pub fn new(depth: usize, gamma: f64, opponent_model: Box<dyn Strategy>) -> Self {
        let depth = depth.max(1).min(8);
        let label = format!("Lookahead-{} ({})", depth, opponent_model.name());
        Self {
            label,
            depth,
            gamma: gamma.clamp(0.0, 1.0),
            payoffs: (5.0, 3.0, 1.0, 0.0),
            opponent_model,
        }
    }

    pub fn with_payoffs(mut self, t: f64, r: f64, p: f64, s: f64) -> Self {
        self.payoffs = (t, r, p, s);
        self
    }

    fn score(&self, mine: Action, theirs: Action) -> f64 {
        let (t, r, p, s) = self.payoffs;
        match (mine, theirs) {
            (Action::Cooperate, Action::Cooperate) => r,
            (Action::Cooperate, Action::Defect) => s,
            (Action::Defect, Action::Cooperate) => t,
            (Action::Defect, Action::Defect) => p,
        }
    }

    /// Recursive max-over-my-actions rollout. Opponent plays its model's
    /// stateless `next_move` (with histories swapped — the model views from
    /// its own perspective). Returns the best discounted sum reachable from
    /// the given state.
    fn rollout(
        &self,
        depth: usize,
        my_buf: &mut Vec<Action>,
        opp_buf: &mut Vec<Action>,
        rng: &mut dyn RngCore,
    ) -> f64 {
        if depth == 0 { return 0.0; }
        let mut best = f64::NEG_INFINITY;
        for &a_self in &[Action::Cooperate, Action::Defect] {
            // Model's "my history" is opp_buf, model's "opp history" is my_buf.
            let a_opp = self.opponent_model.next_move(opp_buf, my_buf, rng);
            let immediate = self.score(a_self, a_opp);
            my_buf.push(a_self);
            opp_buf.push(a_opp);
            let future = self.gamma * self.rollout(depth - 1, my_buf, opp_buf, rng);
            my_buf.pop();
            opp_buf.pop();
            let total = immediate + future;
            if total > best { best = total; }
        }
        best
    }
}

impl Strategy for Lookahead {
    fn name(&self) -> &str { &self.label }

    fn next_move(&self, my_h: &[Action], opp_h: &[Action], rng: &mut dyn RngCore) -> Action {
        let mut my_buf: Vec<Action> = my_h.to_vec();
        let mut opp_buf: Vec<Action> = opp_h.to_vec();
        let mut best_action = Action::Cooperate;
        let mut best_value = f64::NEG_INFINITY;
        for &a_self in &[Action::Cooperate, Action::Defect] {
            let a_opp = self.opponent_model.next_move(&opp_buf, &my_buf, rng);
            let immediate = self.score(a_self, a_opp);
            my_buf.push(a_self);
            opp_buf.push(a_opp);
            let future = self.gamma * self.rollout(self.depth - 1, &mut my_buf, &mut opp_buf, rng);
            my_buf.pop();
            opp_buf.pop();
            let total = immediate + future;
            if total > best_value {
                best_value = total;
                best_action = a_self;
            }
        }
        best_action
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
