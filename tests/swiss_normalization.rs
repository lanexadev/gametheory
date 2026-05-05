//! Swiss tournament must score per-turn (not raw), so discount_factor doesn't
//! silently demote strategies whose matches end early.

use game_theory::strategies::always_cooperate::AlwaysCooperate;
use game_theory::strategies::always_defect::AlwaysDefect;
use game_theory::strategies::tit_for_tat::TitForTat;
use game_theory::{Game, Strategy, Tournament};

#[test]
fn swiss_scores_are_iterations_scale() {
    let strats: Vec<Box<dyn Strategy>> = vec![
        Box::new(AlwaysCooperate),
        Box::new(AlwaysDefect),
        Box::new(TitForTat),
        Box::new(AlwaysDefect),
    ];
    let game = Game {
        iterations: 200,
        // Aggressive discount: average match length ≈ 100 turns instead of 200.
        discount_factor: 0.01,
        seed: Some(11),
        ..Game::default()
    };
    let tournament = Tournament::new(strats, game);
    let scores = tournament.run_swiss(4);

    // After normalisation + scaling, scores should be on the iterations scale
    // (positive, well above what the raw-sum implementation would yield with
    // matches half as long). AllD playing 4 rounds against various opponents
    // should accumulate at least a few hundred points on a 200-iteration
    // projection.
    let max = scores.values().copied().max().unwrap_or(0);
    assert!(
        max > 200,
        "normalised Swiss scores should reach iterations-scale values, got max={}",
        max
    );
    // Sanity: AllD reliably beats AllC head-on, so its score should not be
    // negative or zero.
    let alld = scores.get("Always Defect").copied().unwrap_or(i32::MIN);
    assert!(alld > 0, "AllD should accumulate positive score over 4 rounds, got {}", alld);
}

#[test]
fn swiss_is_reproducible_with_seed() {
    let pop = || -> Vec<Box<dyn Strategy>> {
        vec![
            Box::new(AlwaysCooperate),
            Box::new(AlwaysDefect),
            Box::new(TitForTat),
            Box::new(AlwaysDefect),
        ]
    };
    let game = || Game {
        iterations: 50,
        seed: Some(99),
        ..Game::default()
    };
    let a = Tournament::new(pop(), game()).run_swiss(3);
    let b = Tournament::new(pop(), game()).run_swiss(3);
    assert_eq!(a, b, "same seed → same Swiss results");
}
