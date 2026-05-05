# Phase 3: Realize — Execution Log

Economy mode → tests written alongside implementation, then `cargo test` as a unified gate. No strict Red/Green cycle; correctness validated at end-of-story.

## Files Modified
- `src/lib.rs` — added `Custom` variant + `Default` impl on `StrategyScratch`; added `Neighborhood` enum and `offsets()`; added `RoundRobinReport` struct + `export_matrix_csv`; refactored `run_round_robin_per_individual` into `run_round_robin_report` (full pair matrix, fitness derives from it); `SpatialTournament::new_with_topology` constructor; spatial step now iterates a topology stencil instead of nested loops; spatial init RNG honours `Game.seed`.
- `src/strategies/mod.rs` — declared `pub mod zd; pub mod wsls;`; registered 10 ZD variants and 25 WSLS variants in `get_generative_strategies`.
- `src/strategies/zd.rs` (NEW) — `Memory1Stochastic` reusable building block, `zd_extortion(chi)` and `zd_generous(chi)` factories with closed-form coefficients for canonical IPD payoffs.
- `src/strategies/wsls.rs` (NEW) — `StochasticWSLS` family with `(p_stay_win, p_switch_loss)` parameters.
- `src/main.rs` — added `--topology {moore,vonneumann,hex}` and `--export-matrix <path>` CLI flags; threaded topology to `SpatialTournament::new_with_topology`; matrix export at end of round-robin run.
- `tests/extensible_state.rs` (NEW) — AC-01 regression.
- `tests/zd.rs` (NEW) — AC-02a, AC-02b regression.
- `tests/wsls.rs` (NEW) — AC-03 regression.
- `tests/topology.rs` (NEW) — AC-04 regression.
- `tests/matrix.rs` (NEW) — AC-05 regression.

## Stories
- **S1 — Custom scratch variant**: ✅ implemented + AC-01 test passes.
- **S2 — ZD Extortion / Generous**: ✅ implemented + AC-02a/AC-02b tests pass. Coefficients pinned to canonical IPD payoffs (documented).
- **S3 — WSLS stochastic family**: ✅ implemented + AC-03 test passes. Determinism check (Pavlov-equivalence at `(1.0, 1.0)`) holds.
- **S4 — Neighborhood topology**: ✅ implemented + AC-04 tests pass. Bonus: spatial RNG now honours seed (`SpatialTournament::new_with_topology`).
- **S5 — Pair-score matrix export**: ✅ implemented + AC-05 tests pass. `RoundRobinReport` is now the single source of truth (no duplicated pair-collection logic).
- **S6 — No regression**: ✅ `cargo build --release` passes, `cargo test` 9/9 passes, smoke tests for round-robin / matrix export / spatial-vonneumann all run end-to-end.

## Test Run Summary
```
extensible_state: 1/1 ✅
matrix:           2/2 ✅
topology:         3/3 ✅
wsls:             1/1 ✅
zd:               2/2 ✅
TOTAL:            9/9
```
