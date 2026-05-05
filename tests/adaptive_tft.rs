//! Adaptive TFT — verify the new struct-backed implementation produces the same
//! decisions as the documented per-turn rule (defect when opp coop-rate < target).

use game_theory::strategies::always_defect::AlwaysDefect;
use game_theory::strategies::get_all_strategies;
use game_theory::{Action, Game};

#[test]
fn adaptive_tft_present_and_named() {
    let all = get_all_strategies();
    let n_adaptive = all
        .iter()
        .filter(|s| s.name().starts_with("Adaptive TFT"))
        .count();
    assert_eq!(n_adaptive, 50, "expected 50 Adaptive TFT variants, got {}", n_adaptive);
}

#[test]
fn adaptive_tft_50_pct_defects_against_pure_defector() {
    // "Adaptive TFT (Target 50%)" requires opp coop-rate >= 50%. Against an
    // AllD opponent (coop-rate = 0%) it must defect from turn 2 onward.
    let strategies = get_all_strategies();
    let target = strategies
        .iter()
        .find(|s| s.name() == "Adaptive TFT (Target 50%)")
        .expect("Adaptive TFT (Target 50%) must exist");
    let alld = AlwaysDefect;
    let game = Game { iterations: 20, seed: Some(1), ..Game::default() };
    let (_, _, history) = game.play(target.as_ref(), &alld, Some(1));
    // Turn 1: opp_h is empty → cooperate. Turns 2..: rate(D-only) = 0 < 0.5 → defect.
    assert_eq!(history[0].0, Action::Cooperate, "first move must be C (no history)");
    for (turn, &(my, _)) in history.iter().enumerate().skip(1) {
        assert_eq!(my, Action::Defect, "turn {} must defect (opp coop-rate=0 < 50%)", turn);
    }
}
