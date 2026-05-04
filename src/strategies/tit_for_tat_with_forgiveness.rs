/// Tit For Tat With Forgiveness: A robust version of Tit For Tat that has a 10% chance to
/// forgive a defection by cooperating anyway, helping to break infinite revenge cycles.
use crate::{Action, Strategy};
use rand::{Rng, RngCore};

#[derive(Clone, Default)]
pub struct TitForTatWithForgiveness;
impl Strategy for TitForTatWithForgiveness {
    fn name(&self) -> &str { "Tit For Tat With Forgiveness" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action], rng: &mut dyn RngCore) -> Action {
        match opponent_history.last() {
            Some(Action::Defect) => {
                if rng.random_bool(0.1) { Action::Cooperate } else { Action::Defect }
            }
            _ => Action::Cooperate,
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
