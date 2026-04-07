use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct Statistical;
impl Strategy for Statistical {
    fn name(&self) -> &str { "Statistical" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        if opponent_history.is_empty() {
            return Action::Cooperate;
        }
        let defect_count = opponent_history.iter().filter(|&&a| a == Action::Defect).count();
        if defect_count as f64 / opponent_history.len() as f64 > 0.5 {
            Action::Defect
        } else {
            Action::Cooperate
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
