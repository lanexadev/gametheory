/// Gradual: Punishes defection with an increasing number of defections.
/// First defection = 1 D, second = 2 D, etc. After each punishment, it cooperates twice.
use crate::{Action, Strategy, StrategyScratch};
use rand::RngCore;

#[derive(Clone, Default)]
pub struct Gradual;

fn step_state(opp_defects: &mut usize, p_left: &mut usize, c_left: &mut usize, act: Action) {
    if *p_left > 0 {
        *p_left -= 1;
        if *p_left == 0 { *c_left = 2; }
    } else if *c_left > 0 {
        *c_left -= 1;
    } else if act == Action::Defect {
        *opp_defects += 1;
        *p_left = *opp_defects - 1;
        if *p_left == 0 { *c_left = 2; }
    }
}

fn decide(p_left: usize, c_left: usize, last: Option<&Action>) -> Action {
    if p_left > 0 || (last == Some(&Action::Defect) && c_left == 0) {
        Action::Defect
    } else {
        Action::Cooperate
    }
}

impl Strategy for Gradual {
    fn name(&self) -> &str { "Gradual" }

    fn next_move(&self, _: &[Action], opponent_history: &[Action], _: &mut dyn RngCore) -> Action {
        // Stateless fallback: rebuild the state machine from scratch every call.
        // Quadratic in match length; only used if `next_move_stateful` isn't reached.
        let mut opp_defects = 0;
        let mut p_left = 0;
        let mut c_left = 0;
        for &act in opponent_history {
            step_state(&mut opp_defects, &mut p_left, &mut c_left, act);
        }
        decide(p_left, c_left, opponent_history.last())
    }

    fn init_scratch(&self) -> StrategyScratch {
        StrategyScratch::Gradual { opp_defects: 0, p_left: 0, c_left: 0, processed: 0 }
    }

    fn next_move_stateful(
        &self,
        my_history: &[Action],
        opponent_history: &[Action],
        scratch: &mut StrategyScratch,
        rng: &mut dyn RngCore,
    ) -> Action {
        // O(1) per turn: only fold in the unprocessed tail of the opponent history.
        if let StrategyScratch::Gradual { opp_defects, p_left, c_left, processed } = scratch {
            while *processed < opponent_history.len() {
                step_state(opp_defects, p_left, c_left, opponent_history[*processed]);
                *processed += 1;
            }
            decide(*p_left, *c_left, opponent_history.last())
        } else {
            self.next_move(my_history, opponent_history, rng)
        }
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}
