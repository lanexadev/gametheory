//! AC-04: `Neighborhood` returns the right offset count for each topology.

use game_theory::Neighborhood;

#[test]
fn vonneumann_neighbor_count() {
    assert_eq!(Neighborhood::Moore.offsets(true).len(), 8);
    assert_eq!(Neighborhood::VonNeumann.offsets(true).len(), 4);
    assert_eq!(Neighborhood::VonNeumann.offsets(false).len(), 4);
    assert_eq!(Neighborhood::Hex.offsets(true).len(), 6);
    assert_eq!(Neighborhood::Hex.offsets(false).len(), 6);
}

#[test]
fn vonneumann_excludes_diagonals() {
    let offs = Neighborhood::VonNeumann.offsets(true);
    for &(dy, dx) in offs {
        // Cardinal: exactly one of dy/dx is 0, the other is ±1.
        assert!(
            (dy == 0) ^ (dx == 0),
            "VonNeumann offset {:?} must be cardinal (exactly one zero)",
            (dy, dx)
        );
    }
}

#[test]
fn hex_offsets_differ_by_row_parity() {
    let even = Neighborhood::Hex.offsets(true);
    let odd = Neighborhood::Hex.offsets(false);
    assert_ne!(even, odd, "hex offsets must depend on row parity");
}
