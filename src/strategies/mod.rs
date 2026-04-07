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
    // On ne met plus les singletons, on ne passe que par le générateur massif
    get_generative_strategies()
}

pub fn get_generative_strategies() -> Vec<Box<dyn Strategy>> {
    let mut strategies = Vec::new();
    
    // --- FAMILLE TIT-FOR-TAT (100 variants) ---
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

    // --- FAMILLE GRADUAL (50 variants) ---
    // On varie la sévérité de la punition (combien de trahisons par affrontement)
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

    // --- FAMILLE HANDSHAKE (50 variants) ---
    // On varie les codes secrets
    for code_id in 1..=50 {
        let name = format!("Handshake (Code #{})", code_id);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |my_h, opp_h| {
                let turn = my_h.len();
                if turn < 3 {
                    // Code secret basé sur l'ID (ex: C, D, C ou D, C, D)
                    if (code_id + turn) % 2 == 0 { Action::Cooperate } else { Action::Defect }
                } else {
                    // Vérifie si l'autre a fait le même code
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

    // --- FAMILLE PAVLOV / WIN-STAY (50 variants) ---
    for i in 1..=50 {
        let mistake_prob = i as f64 / 100.0;
        let name = format!("Adaptive Pavlov ({:.0}%)", mistake_prob * 100.0);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |my_h, opp_h| {
                match (my_h.last(), opp_h.last()) {
                    (None, _) => Action::Cooperate,
                    (Some(&my), Some(&opp)) => {
                        let mut move_choice = if my == opp { Action::Cooperate } else { Action::Defect };
                        // Pavlov avec un peu de "réflexion" aléatoire
                        let mut rng = rand::rng();
                        if rng.random_bool(mistake_prob) { move_choice = move_choice.flip(); }
                        move_choice
                    }
                    _ => Action::Cooperate,
                }
            },
        }) as Box<dyn Strategy>);
    }

    // --- FAMILLE OMEGA / ANTI-RANDOM (50 variants) ---
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

    // --- FAMILLE BIASED RANDOM (100 variants) ---
    // On garde les prédateurs car ils sont essentiels pour tester la robustesse
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
