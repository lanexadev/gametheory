/// Alternator: Alternates between Cooperation and Defection at every turn.
use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct Alternator;
impl Strategy for Alternator {
    fn name(&self) -> &str { "Alternator" }
    fn next_move(&self, my_history: &[Action], _: &[Action]) -> Action {
        if my_history.len() % 2 == 0 { Action::Cooperate } else { Action::Defect }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
