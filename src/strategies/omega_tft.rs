/// OmegaTFT: An advanced strategy that attempts to detect noise and random opponents.
/// It plays Tit For Tat but monitors if the opponent's behavior seems inconsistent 
/// (defects even when rewarded). If it suspects a random opponent, it switches to Always Defect.
use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct OmegaTFT;
impl Strategy for OmegaTFT {
    fn name(&self) -> &str { "Omega Tit For Tat" }
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action]) -> Action {
        if opponent_history.len() < 10 {
            return opponent_history.last().cloned().unwrap_or(Action::Cooperate);
        }

        // Count inconsistencies: cases where we cooperated and the opponent defected 
        let mut inconsistencies = 0;
        for i in 1..opponent_history.len() {
            if my_history[i-1] == Action::Cooperate && opponent_history[i] == Action::Defect {
                inconsistencies += 1;
            }
        }

        // If too many inconsistencies, the opponent is either noisy or random
        if inconsistencies > 5 {
            Action::Defect
        } else {
            opponent_history.last().cloned().unwrap_or(Action::Cooperate)
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
