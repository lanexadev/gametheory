use crate::{Action, Strategy, FunctionalStrategy};
use rand::Rng;

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
                next_move_fn: move |_, opp_h| {
                    let mut rng = rand::rng();
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
            next_move_fn: move |_, opp_h| {
                if opp_h.len() < window * 2 { return Action::Cooperate; }
                let last_pattern = &opp_h[opp_h.len()-window..];
                let prev_pattern = &opp_h[opp_h.len()-window*2..opp_h.len()-window];
                if last_pattern == prev_pattern {
                    // Si un cycle est détecté, on prédit le prochain coup et on le contre
                    let next_pred = last_pattern[0]; // Le cycle va recommencer
                    if next_pred == Action::Cooperate { Action::Defect } else { Action::Defect }
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
            next_move_fn: move |_, opp_h| {
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
            next_move_fn: move |my_h, _| {
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
            next_move_fn: move |_, opp_h| {
                let mut rng = rand::rng();
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
            next_move_fn: move |_, opp_h| {
                match opp_h.last() {
                    Some(Action::Defect) => {
                        let mut rng = rand::rng();
                        if rng.random_bool(prob) { Action::Cooperate } else { Action::Defect }
                    }
                    _ => Action::Cooperate,
                }
            },
        }) as Box<dyn Strategy>);
    }

    for mult in 1..=50 {
        let name = format!("Gradual (x{})", mult);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |_, opp_h| {
                let mut opp_defects = 0;
                let mut p_left = 0;
                let mut c_left = 0;
                for &act in opp_h {
                    if p_left > 0 { p_left -= 1; if p_left == 0 { c_left = 2; } }
                    else if c_left > 0 { c_left -= 1; }
                    else if act == Action::Defect {
                        opp_defects += 1;
                        p_left = (opp_defects * mult / 10) + 1; 
                        c_left = 2;
                    }
                }
                if p_left > 0 { Action::Defect } else { Action::Cooperate }
            },
        }) as Box<dyn Strategy>);
    }

    for code_id in 1..=50 {
        let name = format!("Handshake (Code #{})", code_id);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |my_h, opp_h| {
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
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |my_h, opp_h| {
                if opp_h.len() < threshold { return opp_h.last().cloned().unwrap_or(Action::Cooperate); }
                let mut inconsistencies = 0;
                for i in 1..opp_h.len() {
                    if my_h[i-1] == Action::Cooperate && opp_h[i] == Action::Defect { inconsistencies += 1; }
                }
                if inconsistencies > threshold / 2 { Action::Defect }
                else { opp_h.last().cloned().unwrap_or(Action::Cooperate) }
            },
        }) as Box<dyn Strategy>);
    }

    for i in 1..=100 {
        let prob = i as f64 / 100.0;
        let name = format!("Biased Random ({:.0}%)", prob * 100.0);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |_, _| {
                let mut rng = rand::rng();
                if rng.random_bool(prob) { Action::Cooperate } else { Action::Defect }
            },
        }) as Box<dyn Strategy>);
    }

    strategies
}
