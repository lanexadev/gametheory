/// OmegaTFT: An advanced strategy that attempts to detect noise and random opponents.
/// It plays Tit For Tat but monitors if the opponent's behavior seems inconsistent
/// (defects even when rewarded). If it suspects a random opponent, it switches to Always Defect.
use crate::{Action, Strategy, StrategyScratch};
use rand::RngCore;

const OMEGA_WARMUP: usize = 10;
const OMEGA_THRESHOLD: usize = 5;

#[derive(Clone, Default)]
pub struct OmegaTFT;

impl Strategy for OmegaTFT {
    fn name(&self) -> &str { "Omega Tit For Tat" }

    fn next_move(&self, my_history: &[Action], opponent_history: &[Action], _: &mut dyn RngCore) -> Action {
        if opponent_history.len() < OMEGA_WARMUP {
            return opponent_history.last().cloned().unwrap_or(Action::Cooperate);
        }
        let mut inconsistencies = 0;
        for i in 1..opponent_history.len() {
            if my_history[i-1] == Action::Cooperate && opponent_history[i] == Action::Defect {
                inconsistencies += 1;
            }
        }
        if inconsistencies > OMEGA_THRESHOLD { Action::Defect }
        else { opponent_history.last().cloned().unwrap_or(Action::Cooperate) }
    }

    fn init_scratch(&self) -> StrategyScratch {
        StrategyScratch::OmegaDetector { inconsistencies: 0, processed: 0 }
    }

    fn next_move_stateful(
        &self,
        my_history: &[Action],
        opponent_history: &[Action],
        scratch: &mut StrategyScratch,
        rng: &mut dyn RngCore,
    ) -> Action {
        if let StrategyScratch::OmegaDetector { inconsistencies, processed } = scratch {
            // Fold the unprocessed tail. Inconsistency at index i needs my_history[i-1]
            // and opponent_history[i], so start at max(1, processed).
            let mut i = (*processed).max(1);
            while i < opponent_history.len() {
                if my_history[i-1] == Action::Cooperate && opponent_history[i] == Action::Defect {
                    *inconsistencies += 1;
                }
                i += 1;
            }
            *processed = opponent_history.len();

            if opponent_history.len() < OMEGA_WARMUP {
                return opponent_history.last().cloned().unwrap_or(Action::Cooperate);
            }
            if *inconsistencies > OMEGA_THRESHOLD { Action::Defect }
            else { opponent_history.last().cloned().unwrap_or(Action::Cooperate) }
        } else {
            self.next_move(my_history, opponent_history, rng)
        }
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
