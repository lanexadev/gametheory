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
    let mut all: Vec<Box<dyn Strategy>> = vec![
        Box::new(always_cooperate::AlwaysCooperate::default()),
        Box::new(always_defect::AlwaysDefect::default()),
        Box::new(tit_for_tat::TitForTat::default()),
        Box::new(pavlov::Pavlov::default()),
        Box::new(grudger::Grudger::default()),
        Box::new(handshake::Handshake::default()),
        Box::new(tit_for_two_tats::TitForTwoTats::default()),
        Box::new(suspicious_tit_for_tat::SuspiciousTitForTat::default()),
        Box::new(joss::Joss::default()),
        Box::new(tit_for_tat_with_forgiveness::TitForTatWithForgiveness::default()),
        Box::new(statistical::Statistical::default()),
        Box::new(alternator::Alternator::default()),
        Box::new(detective::Detective::default()),
        Box::new(gradual::Gradual::default()),
        Box::new(omega_tft::OmegaTFT::default()),
        Box::new(soft_grudger::SoftGrudger::default()),
    ];
    
    all.extend(get_generative_strategies());
    all
}

pub fn get_generative_strategies() -> Vec<Box<dyn Strategy>> {
    let mut strategies = Vec::new();
    
    // 200 variants de Forgiving Tit For Tat (de 0.5% à 100% de pardon)
    for i in 1..=200 {
        let prob = i as f64 / 200.0;
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

    // 100 variants de N-Tats (mémoire de 2 à 100 trahisons)
    for n in 2..=101 {
        let name = format!("{}-Tats", n);
        strategies.push(Box::new(FunctionalStrategy {
            name,
            next_move_fn: move |_, opp_h| {
                if opp_h.len() < n { return Action::Cooperate; }
                let last_n = &opp_h[opp_h.len()-n..];
                if last_n.iter().all(|&a| a == Action::Defect) { Action::Defect } else { Action::Cooperate }
            },
        }) as Box<dyn Strategy>);
    }

    // 100 variants de "Random Biased" (de 1% à 100% de coopération aléatoire)
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
