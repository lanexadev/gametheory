/// Always Cooperate: A "Saint" strategy that never defects, regardless of the opponent's behavior.
use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct AlwaysCooperate;
impl Strategy for AlwaysCooperate {
    fn name(&self) -> &str { "Always Cooperate" }
    fn next_move(&self, _: &[Action], _: &[Action]) -> Action { Action::Cooperate }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
