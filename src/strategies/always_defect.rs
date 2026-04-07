use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct AlwaysDefect;
impl Strategy for AlwaysDefect {
    fn name(&self) -> &str { "Always Defect" }
    fn next_move(&self, _: &[Action], _: &[Action]) -> Action { Action::Defect }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
