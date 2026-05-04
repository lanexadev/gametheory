//! Zero-Determinant (ZD) strategies — Press & Dyson (2012), Stewart & Plotkin (2013).
//!
//! Memory-1 stochastic strategies whose cooperation probabilities are tuned so
//! that the long-run expected scores satisfy a *linear relation* enforced
//! against ANY opponent. Two variants are exposed:
//!
//! - **ZD-Extortion(chi)**: forces `(score_X - P) = chi * (score_Y - P)` with
//!   `chi >= 1`. Extortioner cannot be exploited and gains a multiplicative
//!   advantage when paired with adaptive opponents.
//!
//! - **ZD-Generous(chi)**: cooperative ZD subset (Stewart-Plotkin 2013). Plays
//!   nicely with cooperative opponents, recovers from mistakes.
//!
//! ## Payoff dependency
//! The cooperation probabilities `(p1, p2, p3, p4)` are derived from the
//! payoff matrix `(T,R,P,S)`. We pre-compute them for the **canonical Axelrod
//! payoffs `(5, 3, 1, 0)`**. Under non-canonical payoffs, the strategies
//! still run but the theoretical Press-Dyson invariant no longer holds.

use crate::{Action, Strategy};
use rand::{Rng, RngCore};

/// Memory-1 stochastic strategy parameterised by 4 cooperation probabilities:
/// `p_cc`, `p_cd`, `p_dc`, `p_dd` — the probability of cooperating after
/// (my=C, opp=C), (my=C, opp=D), (my=D, opp=C), (my=D, opp=D) respectively.
#[derive(Clone)]
pub struct Memory1Stochastic {
    pub label: String,
    pub p_cc: f64,
    pub p_cd: f64,
    pub p_dc: f64,
    pub p_dd: f64,
    pub first_move: Action,
}

impl Strategy for Memory1Stochastic {
    fn name(&self) -> &str { &self.label }

    fn next_move(&self, my_h: &[Action], opp_h: &[Action], rng: &mut dyn RngCore) -> Action {
        let p = match (my_h.last(), opp_h.last()) {
            (None, _) | (_, None) => return self.first_move,
            (Some(Action::Cooperate), Some(Action::Cooperate)) => self.p_cc,
            (Some(Action::Cooperate), Some(Action::Defect))    => self.p_cd,
            (Some(Action::Defect),    Some(Action::Cooperate)) => self.p_dc,
            (Some(Action::Defect),    Some(Action::Defect))    => self.p_dd,
        };
        if rng.random_bool(p.clamp(0.0, 1.0)) { Action::Cooperate } else { Action::Defect }
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

/// Build a ZD-Extortion strategy with extortion factor `chi >= 1`.
/// Coefficients pinned to canonical IPD payoffs (T=5, R=3, P=1, S=0) per
/// Hilbe-Nowak-Sigmund 2013 closed form.
pub fn zd_extortion(chi: f64) -> Memory1Stochastic {
    // For canonical IPD, choose phi at its maximum extortion bound:
    //   phi = 1 / (chi * (T - P) + (P - S)) = 1 / (4*chi + 1)
    // Cooperation probabilities (Hilbe et al. 2013):
    //   p_cc = 1 - phi * (chi - 1) * (T - R)
    //   p_cd = 1 - phi * ((chi - 1) * R + (T - chi * P) - (P - S))   // simplified
    //   p_dc = phi * (chi * (T - R) + (R - P))                       // simplified
    //   p_dd = 0
    // Closed-form for (5,3,1,0):
    //   phi   = 1 / (4*chi + 1)
    //   p_cc  = 1 - 2*phi*(chi - 1)
    //   p_cd  = max(0, 1 - phi*(4*chi + 1))     // pure extortion → p_cd = 0 at phi_max
    //   p_dc  = phi * (2*chi + 2)
    //   p_dd  = 0
    let chi = chi.max(1.0);
    let phi = 1.0 / (4.0 * chi + 1.0);
    let p_cc = (1.0 - 2.0 * phi * (chi - 1.0)).clamp(0.0, 1.0);
    let p_cd = 0.0;
    let p_dc = (phi * (2.0 * chi + 2.0)).clamp(0.0, 1.0);
    let p_dd = 0.0;
    Memory1Stochastic {
        label: format!("ZD-Extortion (chi={:.1})", chi),
        p_cc,
        p_cd,
        p_dc,
        p_dd,
        first_move: Action::Cooperate,
    }
}

/// Build a ZD-Generous strategy with generosity factor `chi >= 1` (Stewart-Plotkin
/// 2013 cooperative ZD subset). Coefficients pinned to canonical IPD payoffs.
/// In the limit `chi → 1` the strategy degenerates to GTFT-like behaviour with
/// near-full cooperation.
pub fn zd_generous(chi: f64) -> Memory1Stochastic {
    // Generous ZD enforces `(score_X - R) = chi * (score_Y - R)` with chi >= 1,
    // which in canonical IPD yields:
    //   phi   = 1 / (4*chi + 1)
    //   p_cc  = 1
    //   p_cd  = 1 - phi * (chi + 4)        // generous on punishment
    //   p_dc  = 1 - phi * (3*chi - 2)      // forgive after exploitation
    //   p_dd  = phi * (chi - 1)            // small chance to forgive mutual D
    let chi = chi.max(1.0);
    let phi = 1.0 / (4.0 * chi + 1.0);
    let p_cc = 1.0;
    let p_cd = (1.0 - phi * (chi + 4.0)).clamp(0.0, 1.0);
    let p_dc = (1.0 - phi * (3.0 * chi - 2.0)).clamp(0.0, 1.0);
    let p_dd = (phi * (chi - 1.0)).clamp(0.0, 1.0);
    Memory1Stochastic {
        label: format!("ZD-Generous (chi={:.1})", chi),
        p_cc,
        p_cd,
        p_dc,
        p_dd,
        first_move: Action::Cooperate,
    }
}
