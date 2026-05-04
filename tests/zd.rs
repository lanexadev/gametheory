//! AC-02: ZD strategies must run, and ZD-Extortion must out-perform AllC.

use game_theory::strategies::always_cooperate::AlwaysCooperate;
use game_theory::strategies::zd::{zd_extortion, zd_generous};
use game_theory::Game;

#[test]
fn extortion_invariant_vs_alld() {
    // Vs an unconditional cooperator, ZD-Extortion exploits and earns strictly
    // more than its opponent across a long deterministic run.
    let game = Game {
        iterations: 5000,
        seed: Some(7),
        ..Game::default()
    };
    let zd = zd_extortion(2.0);
    let opp = AlwaysCooperate;
    let (sc_zd, sc_opp, _) = game.play(&zd, &opp, Some(7));
    assert!(sc_zd > sc_opp, "ZD-Extortion should out-score AllC (got zd={}, opp={})", sc_zd, sc_opp);
    // Sanity: both above the mutual-defection floor (P=1 → P*N=5000).
    assert!(sc_zd > 5000, "ZD-Extortion vs AllC should beat mutual-defection floor");
}

#[test]
fn generous_vs_allc() {
    // ZD-Generous should reach near-mutual-cooperation with AllC.
    let game = Game {
        iterations: 5000,
        seed: Some(11),
        ..Game::default()
    };
    let zd = zd_generous(2.0);
    let opp = AlwaysCooperate;
    let (sc_zd, sc_opp, _) = game.play(&zd, &opp, Some(11));
    let r = game.payoffs.1; // R = 3
    let n = game.iterations as i32;
    // Both should sit comfortably in the cooperative regime (≥80% of R*N).
    let floor = (r * n) * 8 / 10;
    assert!(sc_zd >= floor, "ZD-Generous should mostly cooperate with AllC (got {}, floor {})", sc_zd, floor);
    assert!(sc_opp >= floor, "AllC should mostly receive R against ZD-Generous (got {}, floor {})", sc_opp, floor);
}
