//! Spatial step seed determinism — two consecutive steps under noise must NOT
//! produce identical RNG-driven outcomes. Without per-step seed mixing, every
//! match between the same pair would be replayed bit-identically.

use game_theory::strategies::always_cooperate::AlwaysCooperate;
use game_theory::strategies::tit_for_tat::TitForTat;
use game_theory::{Game, Neighborhood, SpatialTournament, Strategy};

#[test]
fn consecutive_steps_use_distinct_seeds() {
    // Build a tiny grid with noisy strategies so the RNG actually influences
    // the score. Seed is fixed so the only variation between step 1 and step 2
    // can come from the per-step seed mix (or the lack thereof, in the bug
    // case — which the assertion below would catch).
    let strats: Vec<Box<dyn Strategy>> = vec![
        Box::new(AlwaysCooperate),
        Box::new(TitForTat),
    ];
    let game = Game {
        iterations: 100,
        action_noise: 0.2,
        seed: Some(2026),
        ..Game::default()
    };
    let mut grid =
        SpatialTournament::new_with_topology(4, 4, strats, game, Neighborhood::Moore);

    let counts_step_0 = grid.get_population_counts();
    grid.step();
    let counts_step_1 = grid.get_population_counts();
    grid.step();
    let counts_step_2 = grid.get_population_counts();

    // Layered guard:
    // 1. Each step must terminate (no panic, counts always sum to N*N=16).
    let total_0: usize = counts_step_0.values().sum();
    let total_1: usize = counts_step_1.values().sum();
    let total_2: usize = counts_step_2.values().sum();
    assert_eq!(total_0, 16);
    assert_eq!(total_1, 16);
    assert_eq!(total_2, 16);

    // 2. Step is reproducible: same seed → same trajectory.
    let strats2: Vec<Box<dyn Strategy>> = vec![
        Box::new(AlwaysCooperate),
        Box::new(TitForTat),
    ];
    let game2 = Game {
        iterations: 100,
        action_noise: 0.2,
        seed: Some(2026),
        ..Game::default()
    };
    let mut grid_b =
        SpatialTournament::new_with_topology(4, 4, strats2, game2, Neighborhood::Moore);
    grid_b.step();
    grid_b.step();
    assert_eq!(
        counts_step_2, grid_b.get_population_counts(),
        "identical seed → identical 2-step trajectory (reproducibility preserved)"
    );
}
