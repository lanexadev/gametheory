/// Joss: A "sneaky" Tit For Tat. Usually mimics the opponent, but has a 10% chance to defect
/// even when the opponent cooperated, hoping to gain extra points.
use crate::{Action, Strategy};
use rand::{Rng, RngCore};

#[derive(Clone, Default)]
pub struct Joss;
impl Strategy for Joss {
    fn name(&self) -> &str { "Joss" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action], rng: &mut dyn RngCore) -> Action {
        if rng.random_bool(0.1) {
            Action::Defect
        } else {
            opponent_history.last().cloned().unwrap_or(Action::Cooperate)
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
