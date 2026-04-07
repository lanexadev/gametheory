use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct SuspiciousTitForTat;
impl Strategy for SuspiciousTitForTat {
    fn name(&self) -> &str { "Suspicious Tit For Tat" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        opponent_history.last().cloned().unwrap_or(Action::Defect)
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
