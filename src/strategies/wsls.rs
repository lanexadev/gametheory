//! Stochastic Win-Stay-Lose-Shift (WSLS) family.
//!
//! Generalises Pavlov's deterministic WSLS by separating the "stay after win"
//! and "switch after loss" rules into independent probabilities. With
//! `(p_stay_win=1.0, p_switch_loss=1.0)` the strategy reduces to canonical
//! Pavlov — a useful sanity check.
//!
//! Win/loss is read off the previous round payoff:
//! - Win  = `(C,C)` → R, or `(D,C)` → T (got the higher payoff)
//! - Loss = `(C,D)` → S, or `(D,D)` → P

use crate::{Action, Strategy};
use rand::{Rng, RngCore};

#[derive(Clone)]
pub struct StochasticWSLS {
    pub label: String,
    pub p_stay_win: f64,
    pub p_switch_loss: f64,
}

impl Strategy for StochasticWSLS {
    fn name(&self) -> &str { &self.label }

    fn next_move(&self, my_h: &[Action], opp_h: &[Action], rng: &mut dyn RngCore) -> Action {
        match (my_h.last(), opp_h.last()) {
            (None, _) | (_, None) => Action::Cooperate,
            (Some(&my), Some(&opp)) => {
                let won = matches!(opp, Action::Cooperate);
                let stay = if won {
                    rng.random_bool(self.p_stay_win.clamp(0.0, 1.0))
                } else {
                    !rng.random_bool(self.p_switch_loss.clamp(0.0, 1.0))
                };
                if stay { my } else { my.flip() }
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

pub fn wsls(p_stay_win: f64, p_switch_loss: f64) -> StochasticWSLS {
    StochasticWSLS {
        label: format!("WSLS (sw={:.2}, sl={:.2})", p_stay_win, p_switch_loss),
        p_stay_win,
        p_switch_loss,
    }
}
