/// Soft Grudger: Cooperates until the opponent defects. 
/// Then punishes with [D, D, D, D, C, C]. More forgiving than the original Grudger.
use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct SoftGrudger;
impl Strategy for SoftGrudger {
    fn name(&self) -> &str { "Soft Grudger" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        let mut last_defector_idx = None;
        
        for (i, &act) in opponent_history.iter().enumerate() {
            if act == Action::Defect {
                last_defector_idx = Some(i);
            }
        }

        if let Some(idx) = last_defector_idx {
            let turns_since = opponent_history.len() - idx - 1;
            match turns_since {
                0..=3 => Action::Defect,
                4..=5 => Action::Cooperate,
                _ => Action::Cooperate,
            }
        } else {
            Action::Cooperate
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
