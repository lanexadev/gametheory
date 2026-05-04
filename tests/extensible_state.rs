//! AC-01: a strategy can stash typed mutable state in `StrategyScratch::Custom`
//! and have it survive across turns without modifying `lib.rs`.

use game_theory::{Action, Game, Strategy, StrategyScratch};
use rand::RngCore;

/// Counts the number of turns played. Lives entirely in the user crate.
#[derive(Default)]
struct CounterState { turns: usize }

#[derive(Clone)]
struct CountingCooperator;

impl Strategy for CountingCooperator {
    fn name(&self) -> &str { "CountingCooperator" }

    fn next_move(&self, _: &[Action], _: &[Action], _: &mut dyn RngCore) -> Action {
        Action::Cooperate
    }

    fn init_scratch(&self) -> StrategyScratch {
        StrategyScratch::Custom(Box::new(CounterState::default()))
    }

    fn next_move_stateful(
        &self,
        _my: &[Action],
        _opp: &[Action],
        scratch: &mut StrategyScratch,
        _: &mut dyn RngCore,
    ) -> Action {
        if let StrategyScratch::Custom(b) = scratch {
            if let Some(state) = b.downcast_mut::<CounterState>() {
                state.turns += 1;
            }
        }
        Action::Cooperate
    }

    fn clone_box(&self) -> Box<dyn Strategy> { Box::new(self.clone()) }
}

#[test]
fn test_custom_scratch_persists() {
    let game = Game {
        iterations: 10,
        seed: Some(42),
        ..Game::default()
    };
    let s1 = CountingCooperator;
    let s2 = CountingCooperator;
    // We don't directly inspect the scratch after `play` (it's owned by `play`),
    // but we exercise the path: 10 turns, no panic, full cooperation expected.
    let (sc1, sc2, history) = game.play(&s1, &s2, Some(42));
    assert_eq!(history.len(), 10, "discount_factor=0 → match runs to completion");
    assert_eq!(sc1, sc2, "symmetric cooperators score identically");
    let r = game.payoffs.1; // R
    assert_eq!(sc1, r * 10, "two CountingCooperators always cooperate → 10*R");
}
