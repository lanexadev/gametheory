//! Bayesian opponent classifier.
//!
//! Maintains a posterior over a finite set of opponent archetypes. After each
//! turn, the posterior is updated by the likelihood of the observed opponent
//! action under each archetype's predicted move. The agent then plays the
//! action maximising expected immediate payoff under the *posterior-weighted*
//! prediction of the opponent's next move.
//!
//! Posteriors are kept in log-space for numerical stability under long
//! matches (200+ turns) where a naive product would underflow to zero.

use crate::{Action, Strategy, StrategyScratch};
use rand::RngCore;

/// Predicted-next-move archetypes the classifier reasons over. Adding a new
/// archetype is local: implement `coop_prob` and add it to the constructor's
/// vector in the population.
#[derive(Clone, Copy, Debug)]
pub enum Archetype {
    AlwaysC,
    AlwaysD,
    TitForTat,
    Random,
}

impl Archetype {
    /// `P(opponent plays Cooperate next | history)` under this archetype.
    /// `my_h` and `opp_h` are written from *the classifier's* perspective —
    /// `opp_h` is the opponent's history, `my_h` is what the classifier played.
    /// TFT mirrors *my* last move, so it reads `my_h.last()`.
    pub fn coop_prob(self, my_h: &[Action], _opp_h: &[Action]) -> f64 {
        match self {
            Archetype::AlwaysC => 1.0,
            Archetype::AlwaysD => 0.0,
            Archetype::TitForTat => match my_h.last() {
                None | Some(Action::Cooperate) => 1.0,
                Some(Action::Defect) => 0.0,
            },
            Archetype::Random => 0.5,
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Archetype::AlwaysC => "AC",
            Archetype::AlwaysD => "AD",
            Archetype::TitForTat => "TFT",
            Archetype::Random => "RND",
        }
    }
}

#[derive(Clone)]
pub struct BayesianOpponent {
    pub label: String,
    pub archetypes: Vec<Archetype>,
    /// Engine payoffs `(T, R, P, S)` used for expected-value computation.
    pub payoffs: (f64, f64, f64, f64),
    /// Likelihood smoothing (`epsilon` for clamping to `[ε, 1-ε]`). Without
    /// this, a single anomalous observation drives a deterministic archetype's
    /// posterior to zero permanently.
    pub smoothing: f64,
}

#[derive(Clone)]
pub struct BayesianState {
    /// Log-posterior. Initialised to a uniform prior of `ln(1/N)`.
    pub log_posterior: Vec<f64>,
    pub turns_processed: usize,
}

impl BayesianOpponent {
    pub fn new(archetypes: Vec<Archetype>, payoffs: (f64, f64, f64, f64)) -> Self {
        let names: Vec<&str> = archetypes.iter().map(|a| a.short()).collect();
        Self {
            label: format!("Bayesian ({})", names.join("+")),
            archetypes,
            payoffs,
            smoothing: 0.05,
        }
    }

    pub fn with_smoothing(mut self, eps: f64) -> Self {
        self.smoothing = eps.clamp(1e-6, 0.49);
        self
    }
}

impl Strategy for BayesianOpponent {
    fn name(&self) -> &str { &self.label }

    fn next_move(&self, _: &[Action], _: &[Action], _: &mut dyn RngCore) -> Action {
        Action::Cooperate
    }

    fn init_scratch(&self) -> StrategyScratch {
        let n = self.archetypes.len().max(1);
        let prior = (1.0_f64 / n as f64).ln();
        StrategyScratch::Custom(Box::new(BayesianState {
            log_posterior: vec![prior; n],
            turns_processed: 0,
        }))
    }

    fn next_move_stateful(
        &self,
        my_h: &[Action],
        opp_h: &[Action],
        scratch: &mut StrategyScratch,
        _rng: &mut dyn RngCore,
    ) -> Action {
        let StrategyScratch::Custom(b) = scratch else { return Action::Cooperate; };
        let Some(state) = b.downcast_mut::<BayesianState>() else { return Action::Cooperate; };

        // Fold all unprocessed observations into the log-posterior. For each
        // turn `i`, score every archetype's prediction against the actual
        // `opp_h[i]` using the history *prior to* turn i (what the archetype
        // saw when it "decided").
        let pairs = my_h.len().min(opp_h.len());
        while state.turns_processed < pairs {
            let i = state.turns_processed;
            let actual = opp_h[i];
            let my_prefix = &my_h[..i];
            let opp_prefix = &opp_h[..i];
            for (k, arch) in self.archetypes.iter().enumerate() {
                let p_c = arch
                    .coop_prob(my_prefix, opp_prefix)
                    .clamp(self.smoothing, 1.0 - self.smoothing);
                let lik = if actual == Action::Cooperate { p_c } else { 1.0 - p_c };
                state.log_posterior[k] += lik.ln();
            }
            state.turns_processed += 1;
        }

        // Convert log-posterior → normalised probabilities (subtract max for
        // numerical stability) and compute expected `P(opp cooperates next)`.
        let max_lp = state
            .log_posterior
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let weights: Vec<f64> = state
            .log_posterior
            .iter()
            .map(|lp| (lp - max_lp).exp())
            .collect();
        let total: f64 = weights.iter().sum();
        let denom = if total > 0.0 { total } else { 1.0 };
        let pc: f64 = weights
            .iter()
            .zip(self.archetypes.iter())
            .map(|(w, arch)| (w / denom) * arch.coop_prob(my_h, opp_h))
            .sum();

        let (t, r, p, s) = self.payoffs;
        let ev_c = pc * r + (1.0 - pc) * s;
        let ev_d = pc * t + (1.0 - pc) * p;
        if ev_d > ev_c { Action::Defect } else { Action::Cooperate }
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
