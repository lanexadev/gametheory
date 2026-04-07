/// Tit For Tat: The classic reciprocal strategy. Starts with Cooperation, then mimics the opponent's last move.
use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct TitForTat;
impl Strategy for TitForTat {
    fn name(&self) -> &str { "Tit For Tat" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        opponent_history.last().cloned().unwrap_or(Action::Cooperate)
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
