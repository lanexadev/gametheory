use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct Grudger;
impl Strategy for Grudger {
    fn name(&self) -> &str { "Grudger" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        let has_defected = opponent_history.iter().any(|&a| a == Action::Defect);
        if has_defected { Action::Defect } else { Action::Cooperate }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
