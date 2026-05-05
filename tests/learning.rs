//! AC-1..AC-6: Learning / model-based strategies.

use game_theory::strategies::always_cooperate::AlwaysCooperate;
use game_theory::strategies::always_defect::AlwaysDefect;
use game_theory::strategies::bayesian::{Archetype, BayesianOpponent};
use game_theory::strategies::lookahead::Lookahead;
use game_theory::strategies::q_learning::QLearning;
use game_theory::strategies::tit_for_tat::TitForTat;
use game_theory::{Action, Game};

const PAYOFFS: (f64, f64, f64, f64) = (5.0, 3.0, 1.0, 0.0);

fn coop_count(history: &[(Action, Action)]) -> usize {
    history.iter().filter(|(my, _)| *my == Action::Cooperate).count()
}

fn defect_count(history: &[(Action, Action)]) -> usize {
    history.iter().filter(|(my, _)| *my == Action::Defect).count()
}

#[test]
fn q_learner_converges_toward_defection_vs_always_defect() {
    // 1000 turns is plenty for the Q-table to lock in D-D as the dominant
    // pair regardless of state — there's no exploitable cooperative future
    // against AlwaysD.
    let q = QLearning::new(0.30, 0.90, 0.05, 1);
    let game = Game {
        iterations: 1000,
        seed: Some(42),
        ..Game::default()
    };
    let (_, _, history) = game.play(&q, &AlwaysDefect, Some(42));
    let last = &history[history.len() - 100..];
    let defects = defect_count(last);
    assert!(
        defects >= 80,
        "Q-learner should defect ≥80/100 turns vs AlwaysDefect by t=900, got {}",
        defects
    );
}

#[test]
fn bayesian_classifies_and_exploits_always_cooperate() {
    let bay = BayesianOpponent::new(
        vec![Archetype::AlwaysC, Archetype::AlwaysD, Archetype::TitForTat, Archetype::Random],
        PAYOFFS,
    );
    let game = Game {
        iterations: 200,
        seed: Some(7),
        ..Game::default()
    };
    let (_, _, history) = game.play(&bay, &AlwaysCooperate, Some(7));
    // After ~5 turns the AlwaysC archetype dominates the posterior, so the
    // best-response action becomes Defect (T > R) for the rest of the match.
    let last = &history[history.len() - 100..];
    let defects = defect_count(last);
    assert!(
        defects >= 90,
        "Bayesian should defect ≥90/100 against AlwaysC after classification, got {}",
        defects
    );
}

#[test]
fn lookahead_cooperates_against_tit_for_tat() {
    // depth=2, gamma=0.95: at t=0, choosing C yields 3 + γ·max(C-future)
    // ≈ 3 + 0.95·5 ≈ 7.75; choosing D yields 5 + γ·max(post-defect)
    // ≈ 5 + 0.95·1 ≈ 5.95. Lookahead picks C, sustains C-C forever.
    let look = Lookahead::new(2, 0.95, Box::new(TitForTat));
    let game = Game {
        iterations: 200,
        seed: Some(11),
        ..Game::default()
    };
    let (_, _, history) = game.play(&look, &TitForTat, Some(11));
    let coops = coop_count(&history);
    assert!(
        coops >= 195,
        "Lookahead-2/TFT-model vs TFT should cooperate ≥195/200, got {}",
        coops
    );
}

#[test]
fn lookahead_defects_against_always_cooperate_model() {
    // Model = AlwaysC → opponent never retaliates in the simulation. Best
    // response is to always defect (T at every turn).
    let look = Lookahead::new(3, 0.95, Box::new(AlwaysCooperate));
    let game = Game {
        iterations: 100,
        seed: Some(5),
        ..Game::default()
    };
    let (_, _, history) = game.play(&look, &AlwaysCooperate, Some(5));
    let defects = defect_count(&history);
    assert_eq!(
        defects, 100,
        "Lookahead with AC model and AC opponent should defect every turn"
    );
}

#[test]
fn learners_are_deterministic_under_seed() {
    let game = Game {
        iterations: 200,
        seed: Some(99),
        ..Game::default()
    };
    let q1 = QLearning::new(0.20, 0.90, 0.10, 2);
    let q2 = QLearning::new(0.20, 0.90, 0.10, 2);
    let (sc_a, _, hist_a) = game.play(&q1, &TitForTat, Some(99));
    let (sc_b, _, hist_b) = game.play(&q2, &TitForTat, Some(99));
    assert_eq!(sc_a, sc_b, "Q-learning should be deterministic under fixed seed");
    assert_eq!(hist_a, hist_b, "Q-learning histories should match exactly");

    let bay1 = BayesianOpponent::new(
        vec![Archetype::AlwaysC, Archetype::AlwaysD, Archetype::TitForTat],
        PAYOFFS,
    );
    let bay2 = BayesianOpponent::new(
        vec![Archetype::AlwaysC, Archetype::AlwaysD, Archetype::TitForTat],
        PAYOFFS,
    );
    let (sc_a, _, _) = game.play(&bay1, &TitForTat, Some(99));
    let (sc_b, _, _) = game.play(&bay2, &TitForTat, Some(99));
    assert_eq!(sc_a, sc_b, "Bayesian should be deterministic under fixed seed");
}
