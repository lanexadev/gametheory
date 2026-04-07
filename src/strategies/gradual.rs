/// Gradual: Punishes defection with an increasing number of defections.
/// First defection = 1 D, second = 2 D, etc. After each punishment, it cooperates twice.
use crate::{Action, Strategy};

#[derive(Clone, Default)]
pub struct Gradual {
    pub opponent_defects: usize,
    pub punishment_left: usize,
    pub cooldown_left: usize,
}

impl Strategy for Gradual {
    fn name(&self) -> &str { "Gradual" }
    fn next_move(&self, _: &[Action], opponent_history: &[Action]) -> Action {
        // This is a stateful strategy, we need to recalculate state from history
        // since the engine handles strategies as clones or fresh instances.
        let mut opp_defects = 0;
        let mut p_left = 0;
        let mut c_left = 0;
        
        for &act in opponent_history {
            if p_left > 0 {
                p_left -= 1;
                if p_left == 0 { c_left = 2; }
            } else if c_left > 0 {
                c_left -= 1;
            } else if act == Action::Defect {
                opp_defects += 1;
                p_left = opp_defects - 1; // First turn of punishment is now
                if p_left == 0 { c_left = 2; }
            }
        }

        if p_left > 0 || (opponent_history.last() == Some(&Action::Defect) && c_left == 0) {
            Action::Defect
        } else {
            Action::Cooperate
        }
    }
    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
