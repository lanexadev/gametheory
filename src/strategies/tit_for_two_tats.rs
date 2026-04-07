/// Tit For Two Tats: A more forgiving version of Tit For Tat. 
/// Only defects if the opponent has defected in both of the last two turns.
use crate::{Action, Strategy};

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
