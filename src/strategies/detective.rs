/// Detective: Starts with a fixed sequence [C, D, C, C]. 
/// If the opponent never defects during this phase, it defects forever to exploit them.
/// If the opponent defects, it switches to Tit For Tat.
use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct Detective;
impl Strategy for Detective {
    fn name(&self) -> &str { "Detective" }
    fn next_move(&self, my_history: &[Action], opponent_history: &[Action]) -> Action {
        let turn = my_history.len();
        let opening = [Action::Cooperate, Action::Defect, Action::Cooperate, Action::Cooperate];
        
        if turn < opening.len() {
            return opening[turn];
        }
        
        let opponent_defected = opponent_history.iter().any(|&a| a == Action::Defect);
        if !opponent_defected {
            Action::Defect // Exploit the peaceful
        } else {
            opponent_history.last().cloned().unwrap_or(Action::Cooperate) // Play TFT
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
