//! Evolution selection regimes:
//! - truncation (legacy) is deterministic given a seed
//! - mutation injects strategies from outside the current population
//! - softmax roulette preserves diversity better than truncation

use game_theory::strategies::always_cooperate::AlwaysCooperate;
use game_theory::strategies::always_defect::AlwaysDefect;
use game_theory::strategies::tit_for_tat::TitForTat;
use game_theory::{Game, Strategy, Tournament};

fn small_pop() -> Vec<Box<dyn Strategy>> {
    let mut v: Vec<Box<dyn Strategy>> = Vec::new();
    for _ in 0..6 { v.push(Box::new(AlwaysCooperate)); }
    for _ in 0..6 { v.push(Box::new(AlwaysDefect)); }
    for _ in 0..6 { v.push(Box::new(TitForTat)); }
    v
}

fn game(seed: u64) -> Game {
    Game { iterations: 50, seed: Some(seed), ..Game::default() }
}

#[test]
fn truncation_is_deterministic_with_seed() {
    let mut t1 = Tournament::new(small_pop(), game(7));
    let mut t2 = Tournament::new(small_pop(), game(7));
    let (_, h1) = t1.run_evolution_with_options(5, 0.3, 0.0, 0.0, None);
    let (_, h2) = t2.run_evolution_with_options(5, 0.3, 0.0, 0.0, None);
    assert_eq!(h1, h2, "same seed + truncation must give identical evolution histories");
}

#[test]
fn mutation_injects_strategies_outside_population() {
    // Population starts as 100% AlwaysCooperate (not in the mutation pool below
    // necessarily, but we verify that mutation actually mixes things up).
    let mut pop: Vec<Box<dyn Strategy>> = (0..12).map(|_| Box::new(AlwaysCooperate) as Box<dyn Strategy>).collect();
    let pool: Vec<Box<dyn Strategy>> = vec![
        Box::new(AlwaysDefect),
        Box::new(TitForTat),
    ];
    let mut t = Tournament::new(pop.drain(..).collect(), game(123));
    // High mutation rate ensures a meaningful number of injections in 3 gens.
    let (_, history) = t.run_evolution_with_options(3, 0.5, 0.8, 0.0, Some(pool));
    // After at least one gen of mutation, the population should contain at
    // least one strategy that isn't AlwaysCooperate.
    let last = history.last().expect("history must be non-empty");
    let non_ac: usize = last
        .iter()
        .filter(|(name, _)| name.as_str() != "Always Cooperate")
        .map(|(_, &c)| c)
        .sum();
    assert!(
        non_ac > 0,
        "mutation should inject strategies outside the initial uniform AC population, got {:?}",
        last
    );
}

#[test]
fn roulette_with_high_temperature_preserves_diversity() {
    // With T very high, weights tend to uniform → roulette ≈ random sampling,
    // which preserves all three species better than truncation.
    let mut t = Tournament::new(small_pop(), game(42));
    let (_, history) = t.run_evolution_with_options(8, 0.5, 0.0, 100.0, None);
    let last = history.last().unwrap();
    let species: usize = last.values().filter(|&&c| c > 0).count();
    assert!(
        species >= 2,
        "high-temperature roulette should keep ≥ 2 species alive after 8 gens, got {:?}",
        last
    );
}
