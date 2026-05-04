//! AC-05: pair-score matrix has the expected dimensions and finite values.

use game_theory::strategies::always_cooperate::AlwaysCooperate;
use game_theory::strategies::always_defect::AlwaysDefect;
use game_theory::strategies::tit_for_tat::TitForTat;
use game_theory::{Game, Strategy, Tournament};

#[test]
fn matrix_dimensions_and_diagonal() {
    let strats: Vec<Box<dyn Strategy>> = vec![
        Box::new(AlwaysCooperate),
        Box::new(AlwaysDefect),
        Box::new(TitForTat),
    ];
    let game = Game {
        iterations: 100,
        seed: Some(1),
        ..Game::default()
    };
    let tournament = Tournament::new(strats, game);
    let report = tournament.run_round_robin_report();
    assert_eq!(report.fitness.len(), 3);
    assert_eq!(report.matrix.len(), 3);
    for row in &report.matrix {
        assert_eq!(row.len(), 3);
        for &cell in row {
            assert!(cell.is_finite(), "matrix cell must be finite");
        }
    }
    // Diagonal: AllC vs AllC mean per-turn = R = 3.
    assert!((report.matrix[0][0] - 3.0).abs() < 1e-6, "AllC self-play should yield R=3 per turn");
    // AllC vs AllD mean per-turn = S = 0; AllD vs AllC = T = 5.
    assert!((report.matrix[0][1] - 0.0).abs() < 1e-6);
    assert!((report.matrix[1][0] - 5.0).abs() < 1e-6);
}

#[test]
fn matrix_csv_export() {
    let strats: Vec<Box<dyn Strategy>> = vec![
        Box::new(AlwaysCooperate),
        Box::new(AlwaysDefect),
    ];
    let game = Game { iterations: 50, seed: Some(2), ..Game::default() };
    let tournament = Tournament::new(strats, game);
    let report = tournament.run_round_robin_report();
    let path = std::env::temp_dir().join("forge_matrix_test.csv");
    let path_str = path.to_string_lossy().into_owned();
    report.export_matrix_csv(&path_str).expect("matrix export should succeed");
    let content = std::fs::read_to_string(&path).expect("CSV should be readable");
    // Header line + 2 rows
    assert_eq!(content.lines().count(), 3);
    let header = content.lines().next().unwrap();
    assert!(header.starts_with("Strategy,"), "first column must be 'Strategy'");
}
