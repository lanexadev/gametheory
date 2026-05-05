use crate::{Action, Strategy, StrategyScratch, FunctionalStrategy};
use rand::Rng;
use rand::RngCore;

pub mod always_cooperate;
pub mod always_defect;
pub mod tit_for_tat;
pub mod pavlov;
pub mod grudger;
pub mod handshake;
pub mod tit_for_two_tats;
pub mod suspicious_tit_for_tat;
pub mod joss;
pub mod tit_for_tat_with_forgiveness;
pub mod statistical;
pub mod alternator;
pub mod detective;
pub mod gradual;
pub mod omega_tft;
pub mod soft_grudger;
pub mod zd;
pub mod wsls;
pub mod q_learning;
pub mod bayesian;
pub mod lookahead;

/// Parameterised Gradual variant — punish/cooldown state machine where the
/// punishment length scales with `mult`. Carries scratch so the per-turn
/// update is O(1) instead of O(turn).
#[derive(Clone)]
struct GradualFamily {
    name: String,
    mult: usize,
}

impl GradualFamily {
    #[inline]
    fn step(opp_defects: &mut usize, p_left: &mut usize, c_left: &mut usize, mult: usize, act: Action) {
        if *p_left > 0 {
            *p_left -= 1;
            if *p_left == 0 { *c_left = 2; }
        } else if *c_left > 0 {
            *c_left -= 1;
        } else if act == Action::Defect {
            *opp_defects += 1;
            *p_left = (*opp_defects * mult / 10) + 1;
            *c_left = 2;
        }
    }
}

impl Strategy for GradualFamily {
    fn name(&self) -> &str { &self.name }

    fn next_move(&self, _: &[Action], opp_h: &[Action], _: &mut dyn RngCore) -> Action {
        let mut opp_defects = 0;
        let mut p_left = 0;
        let mut c_left = 0;
        for &act in opp_h {
            Self::step(&mut opp_defects, &mut p_left, &mut c_left, self.mult, act);
        }
        if p_left > 0 { Action::Defect } else { Action::Cooperate }
    }

    fn init_scratch(&self) -> StrategyScratch {
        StrategyScratch::Gradual { opp_defects: 0, p_left: 0, c_left: 0, processed: 0 }
    }

    fn next_move_stateful(
        &self,
        my_history: &[Action],
        opp_h: &[Action],
        scratch: &mut StrategyScratch,
        rng: &mut dyn RngCore,
    ) -> Action {
        if let StrategyScratch::Gradual { opp_defects, p_left, c_left, processed } = scratch {
            while *processed < opp_h.len() {
                Self::step(opp_defects, p_left, c_left, self.mult, opp_h[*processed]);
                *processed += 1;
            }
            if *p_left > 0 { Action::Defect } else { Action::Cooperate }
        } else {
            self.next_move(my_history, opp_h, rng)
        }
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

/// Parameterised Omega-Detector — counts (my=C, opp=D) inconsistencies and
/// switches to defect once the running count exceeds `threshold/2`. Scratch
/// keeps the running count across turns.
#[derive(Clone)]
struct OmegaDetectorFamily {
    name: String,
    threshold: usize,
}

impl Strategy for OmegaDetectorFamily {
    fn name(&self) -> &str { &self.name }

    fn next_move(&self, my_h: &[Action], opp_h: &[Action], _: &mut dyn RngCore) -> Action {
        if opp_h.len() < self.threshold {
            return opp_h.last().cloned().unwrap_or(Action::Cooperate);
        }
        let mut inconsistencies = 0;
        for i in 1..opp_h.len() {
            if my_h[i-1] == Action::Cooperate && opp_h[i] == Action::Defect {
                inconsistencies += 1;
            }
        }
        if inconsistencies > self.threshold / 2 { Action::Defect }
        else { opp_h.last().cloned().unwrap_or(Action::Cooperate) }
    }

    fn init_scratch(&self) -> StrategyScratch {
        StrategyScratch::OmegaDetector { inconsistencies: 0, processed: 0 }
    }

    fn next_move_stateful(
        &self,
        my_h: &[Action],
        opp_h: &[Action],
        scratch: &mut StrategyScratch,
        rng: &mut dyn RngCore,
    ) -> Action {
        if let StrategyScratch::OmegaDetector { inconsistencies, processed } = scratch {
            let mut i = (*processed).max(1);
            while i < opp_h.len() {
                if my_h[i-1] == Action::Cooperate && opp_h[i] == Action::Defect {
                    *inconsistencies += 1;
                }
                i += 1;
            }
            *processed = opp_h.len();

            if opp_h.len() < self.threshold {
                return opp_h.last().cloned().unwrap_or(Action::Cooperate);
            }
            if *inconsistencies > self.threshold / 2 { Action::Defect }
            else { opp_h.last().cloned().unwrap_or(Action::Cooperate) }
        } else {
            self.next_move(my_h, opp_h, rng)
        }
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

pub fn get_all_strategies() -> Vec<Box<dyn Strategy>> {
    get_generative_strategies()
}

pub fn get_generative_strategies() -> Vec<Box<dyn Strategy>> {
    let mut strategies = Vec::new();
    
    // --- FAMILLE STOCHASTIQUE RÉACTIVE (100 variants) ---
    // Ces stratégies jouent C avec probabilité P si l'autre a fait C,
    // et probabilité Q si l'autre a fait D. (Mémoire 1 stochastique)
    for i in 1..=10 {
        for j in 1..=10 {
            let p = i as f64 / 10.0;
            let q = j as f64 / 10.0;
            let name = format!("Reactive (p={:.1}, q={:.1})", p, q);
            strategies.push(Box::new(FunctionalStrategy {
                name,
                next_move_fn: move |_: &[Action], opp_h: &[Action], rng: &mut dyn RngCore| {
                    match opp_h.last() {
                        Some(Action::Cooperate) => if rng.random_bool(p) { Action::Cooperate } else { Action::Defect },
                        Some(Action::Defect) => if rng.random_bool(q) { Action::Cooperate } else { Action::Defect },
                        None => Action::Cooperate,
                    }
                },
            }) as Box<dyn Strategy>);
        }
    }

    // --- FAMILLE PATTERN MATCHER (50 variants) ---
    // Tente de détecter si l'adversaire joue une séquence cyclique (ex: C-C-D)
    for window in 2..=11 {
        let name = format!("Pattern Matcher (W={})", window);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |_: &[Action], opp_h: &[Action], _: &mut dyn RngCore| {
                if opp_h.len() < window * 2 { return Action::Cooperate; }
                let last_pattern = &opp_h[opp_h.len()-window..];
                let prev_pattern = &opp_h[opp_h.len()-window*2..opp_h.len()-window];
                if last_pattern == prev_pattern {
                    // Cycle détecté → on prédit le prochain coup de l'adversaire
                    // et on joue la meilleure réponse :
                    //   - prédiction C → D (exploitation, gain T)
                    //   - prédiction D → D (autoprotection, P > S)
                    Action::Defect
                } else {
                    opp_h.last().cloned().unwrap_or(Action::Cooperate)
                }
            },
        }) as Box<dyn Strategy>);
    }

    // --- FAMILLE ADAPTIVE TFT (50 variants) ---
    // Ajuste son pardon dynamiquement selon le taux de coopération global
    for target in 1..=50 {
        let target_rate = target as f64 / 50.0;
        let name = format!("Adaptive TFT (Target {:.0}%)", target_rate * 100.0);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |_: &[Action], opp_h: &[Action], _: &mut dyn RngCore| {
                if opp_h.is_empty() { return Action::Cooperate; }
                let current_rate = opp_h.iter().filter(|&&a| a == Action::Cooperate).count() as f64 / opp_h.len() as f64;
                if current_rate < target_rate { Action::Defect } else { Action::Cooperate }
            },
        }) as Box<dyn Strategy>);
    }

    // --- FAMILLE BACKSTABBER (50 variants) ---
    // Coopère jusqu'au tour N, puis trahit pour toujours
    for n in (10..510).step_by(10) {
        let name = format!("Backstabber (T={})", n);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |my_h: &[Action], _: &[Action], _: &mut dyn RngCore| {
                if my_h.len() >= n { Action::Defect } else { Action::Cooperate }
            },
        }) as Box<dyn Strategy>);
    }

    // --- FAMILLE BULLY / PARADOXICAL (50 variants) ---
    // L'inverse de TFT : Trahit quand l'autre coopère, coopère quand l'autre trahit
    for i in 1..=50 {
        let prob = i as f64 / 50.0;
        let name = format!("Bully ({:.0}%)", prob * 100.0);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |_: &[Action], opp_h: &[Action], rng: &mut dyn RngCore| {
                if !rng.random_bool(prob) { return Action::Defect; }
                match opp_h.last() {
                    Some(Action::Cooperate) => Action::Defect,
                    Some(Action::Defect) => Action::Cooperate,
                    None => Action::Cooperate,
                }
            },
        }) as Box<dyn Strategy>);
    }

    // --- FAMILLES PRÉCÉDENTES (Ré-équilibrées) ---
    for i in 1..=100 {
        let prob = i as f64 / 100.0;
        let name = format!("Forgiving TFT ({:.1}%)", prob * 100.0);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |_: &[Action], opp_h: &[Action], rng: &mut dyn RngCore| {
                match opp_h.last() {
                    Some(Action::Defect) => {
                        if rng.random_bool(prob) { Action::Cooperate } else { Action::Defect }
                    }
                    _ => Action::Cooperate,
                }
            },
        }) as Box<dyn Strategy>);
    }

    for mult in 1..=50 {
        let name = format!("Gradual (x{})", mult);
        strategies.push(Box::new(GradualFamily { name, mult }) as Box<dyn Strategy>);
    }

    for code_id in 1..=50 {
        let name = format!("Handshake (Code #{})", code_id);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |my_h: &[Action], opp_h: &[Action], _: &mut dyn RngCore| {
                let turn = my_h.len();
                if turn < 3 {
                    if (code_id + turn) % 2 == 0 { Action::Cooperate } else { Action::Defect }
                } else {
                    let mut match_code = true;
                    for t in 0..3 {
                        let expected = if (code_id + t) % 2 == 0 { Action::Cooperate } else { Action::Defect };
                        if opp_h.get(t) != Some(&expected) { match_code = false; break; }
                    }
                    if match_code { opp_h.last().cloned().unwrap_or(Action::Cooperate) }
                    else { Action::Defect }
                }
            },
        }) as Box<dyn Strategy>);
    }

    for threshold in 2..=51 {
        let name = format!("Omega-Detector (Thresh {})", threshold);
        strategies.push(Box::new(OmegaDetectorFamily { name, threshold }) as Box<dyn Strategy>);
    }

    for i in 1..=100 {
        let prob = i as f64 / 100.0;
        let name = format!("Biased Random ({:.0}%)", prob * 100.0);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |_: &[Action], _: &[Action], rng: &mut dyn RngCore| {
                if rng.random_bool(prob) { Action::Cooperate } else { Action::Defect }
            },
        }) as Box<dyn Strategy>);
    }

    // --- ZD STRATEGIES (Press-Dyson 2012, Stewart-Plotkin 2013) ---
    // 5 chi values × 2 modes = 10 variants. Mathematically valid for canonical
    // payoffs (5,3,1,0) only; under custom payoffs the Press-Dyson invariant
    // no longer holds but the strategies still play coherently.
    for chi_x10 in [11u32, 13, 15, 20, 30] {
        let chi = chi_x10 as f64 / 10.0;
        strategies.push(Box::new(zd::zd_extortion(chi)) as Box<dyn Strategy>);
        strategies.push(Box::new(zd::zd_generous(chi)) as Box<dyn Strategy>);
    }

    // --- WSLS STOCHASTIC FAMILY (25 variants) ---
    // 5×5 grid over (p_stay_win, p_switch_loss). (1.0, 1.0) recovers Pavlov.
    for sw in [0.5, 0.7, 0.85, 0.95, 1.0] {
        for sl in [0.5, 0.7, 0.85, 0.95, 1.0] {
            strategies.push(Box::new(wsls::wsls(sw, sl)) as Box<dyn Strategy>);
        }
    }

    // --- LEARNING / MODEL-BASED FAMILY ---
    // Three orthogonal "smart" archetypes that exploit `StrategyScratch::Custom`:
    //   • Q-Learning  — model-free RL, ε-greedy over Q(state, action) with
    //     state = last-K joint moves. K=1/2 chosen to keep the table small
    //     enough to converge inside a 200–1000-turn match.
    //   • Bayesian    — posterior over a finite archetype basis (AC, AD, TFT,
    //     Random); plays expected-value best response. Identifies and
    //     exploits exploitable opponents within ~10–20 turns.
    //   • Lookahead   — depth-limited minimax against a fixed opponent model.
    //     With TFT model and depth ≥ 2, learns to cooperate; with AlwaysC
    //     model and any depth, learns to defect.

    // 6 Q-Learning variants spanning fast-greedy → slow-explorative learners.
    let q_grid = [
        (0.50, 0.90, 0.05, 1usize),
        (0.30, 0.95, 0.10, 1),
        (0.20, 0.95, 0.05, 2),
        (0.10, 0.95, 0.10, 2),
        (0.05, 0.99, 0.20, 2),
        (0.10, 0.99, 0.05, 3),
    ];
    for (a, g, e, k) in q_grid {
        strategies.push(Box::new(q_learning::QLearning::new(a, g, e, k)) as Box<dyn Strategy>);
    }

    // 4 Bayesian classifiers — full basis vs reduced bases, default smoothing.
    let bay_payoffs = (5.0, 3.0, 1.0, 0.0);
    use bayesian::Archetype as A;
    let archetype_sets: Vec<Vec<A>> = vec![
        vec![A::AlwaysC, A::AlwaysD, A::TitForTat, A::Random],
        vec![A::AlwaysC, A::AlwaysD, A::TitForTat],
        vec![A::AlwaysC, A::AlwaysD],
        vec![A::TitForTat, A::AlwaysD, A::Random],
    ];
    for set in archetype_sets {
        strategies.push(Box::new(bayesian::BayesianOpponent::new(set, bay_payoffs)) as Box<dyn Strategy>);
    }

    // 5 Lookahead agents over varying depth × opponent model.
    let look_specs: Vec<(usize, f64, Box<dyn Strategy>)> = vec![
        (1, 0.95, Box::new(tit_for_tat::TitForTat)),
        (2, 0.95, Box::new(tit_for_tat::TitForTat)),
        (3, 0.95, Box::new(tit_for_tat::TitForTat)),
        (2, 0.95, Box::new(grudger::Grudger)),
        (2, 0.95, Box::new(always_cooperate::AlwaysCooperate)),
    ];
    for (d, g, model) in look_specs {
        strategies.push(Box::new(lookahead::Lookahead::new(d, g, model)) as Box<dyn Strategy>);
    }

    strategies
}
