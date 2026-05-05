# Acceptance Contracts

| AC | Given | When | Then | Test |
|---|---|---|---|---|
| **AC-01** Trait extensible | A new strategy needs typed mutable state | Author returns `StrategyScratch::Custom(Box::new(MyState))` from `init_scratch` and downcasts in `next_move_stateful` | Strategy compiles and state survives across turns without `lib.rs` edit | `tests/extensible_state.rs::test_custom_scratch_persists` |
| **AC-02a** ZD Extortion | Canonical Axelrod payoffs `(5,3,1,0)`, no noise, 5000 turns | `ZDExtortion(chi=2.0)` plays `AllD` | `(score_ZD − P) ≈ chi · (score_AllD − P)` within ±15% (probabilistic, allow some slack) | `tests/zd.rs::extortion_invariant_vs_alld` |
| **AC-02b** ZD Generous | Canonical payoffs | `ZDGenerous(chi=2.0)` plays `AllC` over 5000 turns | Both score above mutual-defection baseline; ZD never falls below `AllC` mean | `tests/zd.rs::generous_vs_allc` |
| **AC-03** WSLS family | Variant `WSLS(stay_win=1.0, switch_loss=1.0)` | Plays itself with no noise | Mutual cooperation lock (deterministic Pavlov behavior) | `tests/wsls.rs::deterministic_wsls_equals_pavlov` |
| **AC-04** Topologies | `--spatial --topology vonneumann --grid_size 10 --generations 1` | One step | Each cell only interacts with 4 cardinal neighbors | `tests/topology.rs::vonneumann_neighbor_count` |
| **AC-05** Matrix export | Tournament with N≥2 strategies | `--export-matrix path.csv` | CSV is `(N+1)×(N+1)` (header row+col); diagonal entries = self-play scores; off-diagonal cells finite | `tests/matrix.rs::matrix_dimensions_and_diagonal` |
| **AC-06** No regression | Default CLI flags | Run round-robin and evolution | `cargo build --release` + `cargo test` succeed; legacy CSVs still well-formed | `cargo test` global gate |

## Test Specifications

| Test ID | AC | File | Function | Assertions |
|---|---|---|---|---|
| T-01 | AC-01 | `tests/extensible_state.rs` | `test_custom_scratch_persists` | After 10 turns, the custom counter equals 10 (proving scratch persists) |
| T-02 | AC-02a | `tests/zd.rs` | `extortion_invariant_vs_alld` | Press-Dyson differential ratio within ±15% of `chi` |
| T-03 | AC-02b | `tests/zd.rs` | `generous_vs_allc` | Both ZD and AllC mean score > P; ZD ≥ AllC mean |
| T-04 | AC-03 | `tests/wsls.rs` | `deterministic_wsls_equals_pavlov` | Final history is full cooperation |
| T-05 | AC-04 | `tests/topology.rs` | `vonneumann_neighbor_count` | Function `Neighborhood::offsets` returns 4 entries for VonNeumann, 8 for Moore, 6 for Hex |
| T-06 | AC-05 | `tests/matrix.rs` | `matrix_dimensions_and_diagonal` | N+1 rows, N+1 cols; cells parse as f64 |
| T-07 | AC-06 | `cargo test` | (all of the above) | Global gate |
