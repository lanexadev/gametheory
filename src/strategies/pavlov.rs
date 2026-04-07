use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct Pavlov;
impl Strategy for Pavlov {
    fn name(&self) -> &str { "Pavlov (Win-Stay, Lose-Shift)" }
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action]) -> Action {
        match (my_history.last(), opponent_history.last()) {
            (None, _) => Action::Cooperate,
            (Some(&my), Some(&opp)) => {
                if my == opp { Action::Cooperate } else { Action::Defect }
            }
            _ => Action::Cooperate,
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
