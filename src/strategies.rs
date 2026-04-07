use crate::{Action, Strategy, FunctionalStrategy};
use rand::Rng;

#[derive(Clone, Default)]
pub struct AlwaysCooperate;
impl Strategy for AlwaysCooperate {
    fn name(&self) -> &str { "Always Cooperate" }
    fn next_move(&self, _: &[Action], _: &[Action]) -> Action { Action::Cooperate }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct AlwaysDefect;
impl Strategy for AlwaysDefect {
    fn name(&self) -> &str { "Always Defect" }
    fn next_move(&self, _: &[Action], _: &[Action]) -> Action { Action::Defect }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct TitForTat;
impl Strategy for TitForTat {
    fn name(&self) -> &str { "Tit For Tat" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        opponent_history.last().cloned().unwrap_or(Action::Cooperate)
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct Random;
impl Strategy for Random {
    fn name(&self) -> &str { "Random" }
    fn next_move(&self, _: &[Action], _: &[Action]) -> Action {
        if rand::random() { Action::Cooperate } else { Action::Defect }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct Grudger;
impl Strategy for Grudger {
    fn name(&self) -> &str { "Grudger" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        let has_defected = opponent_history.iter().any(|&a| a == Action::Defect);
        if has_defected { Action::Defect } else { Action::Cooperate }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct TitForTwoTats;
impl Strategy for TitForTwoTats {
    fn name(&self) -> &str { "Tit For Two Tats" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        if opponent_history.len() < 2 {
            return Action::Cooperate;
        }
        let last_two = &opponent_history[opponent_history.len()-2..];
        if last_two.iter().all(|&a| a == Action::Defect) {
            Action::Defect
        } else {
            Action::Cooperate
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct SuspiciousTitForTat;
impl Strategy for SuspiciousTitForTat {
    fn name(&self) -> &str { "Suspicious Tit For Tat" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        opponent_history.last().cloned().unwrap_or(Action::Defect)
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct Pavlov;
impl Strategy for Pavlov {
    fn name(&self) -> &str { "Pavlov (Win-Stay, Lose-Shift)" }
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action]) -> Action {
        match (my_history.last(), opponent_history.last()) {
            (None, _) => Action::Cooperate,
            (Some(&my), Some(&opp)) => {
                if my == opp { Action::Cooperate } else { Action::Defect }
            }
            _ => Action::Cooperate,
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct Joss;
impl Strategy for Joss {
    fn name(&self) -> &str { "Joss" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        let mut rng = rand::rng();
        if rng.random_bool(0.1) {
            Action::Defect
        } else {
            opponent_history.last().cloned().unwrap_or(Action::Cooperate)
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct TitForTatWithForgiveness;
impl Strategy for TitForTatWithForgiveness {
    fn name(&self) -> &str { "Tit For Tat With Forgiveness" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        match opponent_history.last() {
            Some(Action::Defect) => {
                let mut rng = rand::rng();
                if rng.random_bool(0.1) { Action::Cooperate } else { Action::Defect }
            }
            _ => Action::Cooperate,
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct Handshake;
impl Strategy for Handshake {
    fn name(&self) -> &str { "Handshake" }
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action]) -> Action {
        let turn = my_history.len();
        match turn {
            0 => Action::Cooperate,
            1 => Action::Defect,
            _ => {
                if opponent_history.len() >= 2 && opponent_history[0] == Action::Cooperate && opponent_history[1] == Action::Defect {
                    opponent_history.last().cloned().unwrap_or(Action::Cooperate) // Act like TitForTat
                } else {
                    Action::Defect // Punish outsiders
                }
            }
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[derive(Clone, Default)]
pub struct Statistical;
impl Strategy for Statistical {
    fn name(&self) -> &str { "Statistical" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        if opponent_history.is_empty() {
            return Action::Cooperate;
        }
        let defect_count = opponent_history.iter().filter(|&&a| a == Action::Defect).count();
        if defect_count as f64 / opponent_history.len() as f64 > 0.5 {
            Action::Defect
        } else {
            Action::Cooperate
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

pub fn get_generative_strategies() -> Vec<Box<dyn Strategy>> {
    let mut strategies = Vec::new();

    // Create many variants of "Forgiving Tit For Tat" with different probabilities
    for i in 1..=10 {
        let prob = i as f64 / 20.0;
        let name = format!("Forgiving Tit For Tat ({:.0}%)", prob * 100.0);
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

    // Create variants of "N-Tats" (Defect if opponent defected N times in a row)
    for n in 3..=6 {
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

    strategies
}

pub fn get_all_strategies() -> Vec<Box<dyn Strategy>> {
    let mut all = vec![
        Box::new(AlwaysCooperate::default()) as Box<dyn Strategy>,
        Box::new(AlwaysDefect::default()) as Box<dyn Strategy>,
        Box::new(TitForTat::default()) as Box<dyn Strategy>,
        Box::new(Random::default()) as Box<dyn Strategy>,
        Box::new(Grudger::default()) as Box<dyn Strategy>,
        Box::new(TitForTwoTats::default()) as Box<dyn Strategy>,
        Box::new(SuspiciousTitForTat::default()) as Box<dyn Strategy>,
        Box::new(Pavlov::default()) as Box<dyn Strategy>,
        Box::new(Joss::default()) as Box<dyn Strategy>,
        Box::new(TitForTatWithForgiveness::default()) as Box<dyn Strategy>,
        Box::new(Handshake::default()) as Box<dyn Strategy>,
        Box::new(Statistical::default()) as Box<dyn Strategy>,
    ];
    all.extend(get_generative_strategies());
    all
}
