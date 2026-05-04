/// Always Defect: A "Nasty" strategy that always tries to exploit the opponent and never cooperates.
use crate::{Action, Strategy};
use rand::RngCore;

#[derive(Clone, Default)]
pub struct AlwaysDefect;
impl Strategy for AlwaysDefect {
    fn name(&self) -> &str { "Always Defect" }
    fn next_move(&self, _: &[Action], _: &[Action], _: &mut dyn RngCore) -> Action { Action::Defect }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
