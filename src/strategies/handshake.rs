/// Handshake: A group-recognition strategy. Starts with [Cooperate, Defect]. 
/// If the opponent matches this sequence, it cooperates forever; otherwise, it defects forever.
use crate::{Action, Strategy};

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
