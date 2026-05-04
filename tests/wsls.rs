//! AC-03: deterministic WSLS(1, 1) recovers Pavlov's mutual-cooperation lock.

use game_theory::strategies::wsls::wsls;
use game_theory::Game;

#[test]
fn deterministic_wsls_equals_pavlov() {
    let game = Game {
        iterations: 200,
        seed: Some(99),
        ..Game::default()
    };
    let a = wsls(1.0, 1.0);
    let b = wsls(1.0, 1.0);
    let (sc_a, sc_b, history) = game.play(&a, &b, Some(99));
    assert_eq!(sc_a, sc_b, "symmetric Pavlov-equivalents tie");
    let r = game.payoffs.1;
    assert_eq!(sc_a, r * 200, "two deterministic WSLS lock into mutual cooperation");
    // History should be all (C, C).
    use game_theory::Action::*;
    assert!(history.iter().all(|&(m1, m2)| m1 == Cooperate && m2 == Cooperate),
        "every turn must be mutual cooperation");
}
