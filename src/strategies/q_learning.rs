//! Tabular Q-Learning agent for the iterated PD.
//!
//! Model-free RL: maintains `Q(s, a)` for `s` = the last-`K` joint actions
//! encoded as a `u32` bit-field, `a ∈ {C, D}`. Action selection is ε-greedy;
//! per-turn TD update:
//!
//!   Q[s_prev][a_prev] += α · (r + γ · max_a' Q[s_new][a'] − Q[s_prev][a_prev])
//!
//! All randomness flows through the engine's `&mut dyn RngCore` — seedable
//! reproducibility is preserved. State lives in `StrategyScratch::Custom` so
//! it persists across turns within a match and resets between matches via
//! `init_scratch`.

use crate::{Action, Strategy, StrategyScratch};
use rand::{Rng, RngCore};
use std::collections::HashMap;

#[derive(Clone)]
pub struct QLearning {
    pub label: String,
    pub alpha: f64,
    pub gamma: f64,
    pub epsilon: f64,
    pub k: usize,
    /// Reward shaping: payoffs the agent uses for its TD updates. Default is
    /// canonical Axelrod (5,3,1,0). If the engine's payoffs differ, the agent
    /// learns sub-optimally — exposing this lets tests / power users align.
    pub r_t: f64,
    pub r_r: f64,
    pub r_p: f64,
    pub r_s: f64,
}

#[derive(Default)]
pub struct QLearningState {
    pub q: HashMap<u32, [f64; 2]>,
    pub last_state: Option<u32>,
    pub turns_processed: usize,
}

impl QLearning {
    pub fn new(alpha: f64, gamma: f64, epsilon: f64, k: usize) -> Self {
        let k = k.max(1).min(15);
        Self {
            label: format!(
                "Q-Learning (a={:.2}, g={:.2}, e={:.2}, K={})",
                alpha, gamma, epsilon, k
            ),
            alpha: alpha.clamp(0.0, 1.0),
            gamma: gamma.clamp(0.0, 1.0),
            epsilon: epsilon.clamp(0.0, 1.0),
            k,
            r_t: 5.0,
            r_r: 3.0,
            r_p: 1.0,
            r_s: 0.0,
        }
    }

    pub fn with_payoffs(mut self, t: f64, r: f64, p: f64, s: f64) -> Self {
        self.r_t = t;
        self.r_r = r;
        self.r_p = p;
        self.r_s = s;
        self
    }

    /// Encode the last-`K` joint actions (mine, opp) into a `u32` bit-field.
    /// 2 bits per turn: high bit = my action, low bit = opp action; 0 = C, 1 = D.
    fn encode_state(&self, my_h: &[Action], opp_h: &[Action]) -> u32 {
        let n = my_h.len().min(opp_h.len());
        let start = n.saturating_sub(self.k);
        let mut s: u32 = 0;
        for i in start..n {
            let mb = if my_h[i] == Action::Defect { 1u32 } else { 0u32 };
            let ob = if opp_h[i] == Action::Defect { 1u32 } else { 0u32 };
            s = (s << 2) | (mb << 1) | ob;
        }
        s
    }

    fn payoff(&self, mine: Action, opp: Action) -> f64 {
        match (mine, opp) {
            (Action::Cooperate, Action::Cooperate) => self.r_r,
            (Action::Cooperate, Action::Defect) => self.r_s,
            (Action::Defect, Action::Cooperate) => self.r_t,
            (Action::Defect, Action::Defect) => self.r_p,
        }
    }
}

impl Strategy for QLearning {
    fn name(&self) -> &str { &self.label }

    fn next_move(&self, _: &[Action], _: &[Action], _: &mut dyn RngCore) -> Action {
        // Stateless fallback (no learning). The engine always calls the
        // stateful path, so this is reached only when callers bypass scratch.
        Action::Cooperate
    }

    fn init_scratch(&self) -> StrategyScratch {
        StrategyScratch::Custom(Box::new(QLearningState::default()))
    }

    fn next_move_stateful(
        &self,
        my_h: &[Action],
        opp_h: &[Action],
        scratch: &mut StrategyScratch,
        rng: &mut dyn RngCore,
    ) -> Action {
        let StrategyScratch::Custom(b) = scratch else { return Action::Cooperate; };
        let Some(state) = b.downcast_mut::<QLearningState>() else { return Action::Cooperate; };

        // Fold any not-yet-seen turns into Q updates. We use the *actual*
        // played action `my_h[i]` (not the prior decision), so action_noise
        // and perception_noise are absorbed correctly: the agent learns from
        // what really happened, including its own slips.
        while state.turns_processed < my_h.len().min(opp_h.len()) {
            let i = state.turns_processed;
            let played = my_h[i];
            let payoff = self.payoff(played, opp_h[i]);
            let new_state = self.encode_state(&my_h[..=i], &opp_h[..=i]);

            if let Some(prev_s) = state.last_state {
                let next_q = state.q.get(&new_state).copied().unwrap_or([0.0, 0.0]);
                let next_max = next_q[0].max(next_q[1]);
                let entry = state.q.entry(prev_s).or_insert([0.0, 0.0]);
                let a_idx = if played == Action::Cooperate { 0 } else { 1 };
                entry[a_idx] += self.alpha * (payoff + self.gamma * next_max - entry[a_idx]);
            }
            state.last_state = Some(new_state);
            state.turns_processed += 1;
        }

        // ε-greedy action selection from the current state.
        let cur_state = self.encode_state(my_h, opp_h);
        let action = if self.epsilon > 0.0 && rng.random_bool(self.epsilon) {
            if rng.random_bool(0.5) { Action::Cooperate } else { Action::Defect }
        } else {
            let q = state.q.get(&cur_state).copied().unwrap_or([0.0, 0.0]);
            if q[1] > q[0] { Action::Defect } else { Action::Cooperate }
        };
        state.last_state = Some(cur_state);
        action
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
