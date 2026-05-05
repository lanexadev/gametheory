# FORGE Complete — 04-gameplay-expressivity

**Task:** Logique de gameplay / expressivité (#13–#16 from the prior review)
**Type:** feature
**Mode:** Solo (economy)
**Status:** Completed

### Phases
- [x] Triage → feature, 6 ACs identified
- [x] Fathom → full repo scan, 4 axes mapped
- [x] Orchestrate → 6 ACs, 4 ADRs, 6 stories
- [x] Realize → 6/6 stories complete
- [x] Guard → skipped (no `-x`)
- [x] Export → committed locally pending user approval

### Files Modified
- `src/lib.rs`
- `src/strategies/mod.rs`
- `src/strategies/zd.rs` (new)
- `src/strategies/wsls.rs` (new)
- `src/main.rs`
- `tests/extensible_state.rs` (new)
- `tests/zd.rs` (new)
- `tests/wsls.rs` (new)
- `tests/topology.rs` (new)
- `tests/matrix.rs` (new)

### Acceptance Criteria
- [x] **AC-01** Strategy can stash typed mutable state via `StrategyScratch::Custom(Box<dyn Any + Send>)` without editing `lib.rs`.
- [x] **AC-02a** ZD-Extortion(chi=2.0) out-scores AllC over 5000 turns.
- [x] **AC-02b** ZD-Generous(chi=2.0) reaches near-mutual-cooperation with AllC (≥80% of R*N).
- [x] **AC-03** Deterministic WSLS(1.0, 1.0) collapses to mutual-cooperation lock (Pavlov equivalence).
- [x] **AC-04** `Neighborhood::Moore/VonNeumann/Hex` returns 8/4/6 offsets respectively; CLI `--topology` switches between them.
- [x] **AC-05** `--export-matrix` writes an N×N CSV with name headers; diagonal verifies (AllC vs AllC = 3.0/turn, AllD vs AllC = 5.0/turn).
- [x] **AC-06** `cargo build --release` and `cargo test` (9/9) pass; legacy CLI flags unaffected.

### Bonus Fix
While threading topology through `SpatialTournament`, also honoured `Game.seed` for the initial random grid layout — previous code used `from_os_rng()`, silently breaking spatial reproducibility even with `--seed`.

### Open Items / Follow-ups
- ZD coefficient closed forms are pinned to canonical Axelrod payoffs `(5,3,1,0)`. Generic `(T,R,P,S)` derivation is left for a follow-up.
- The remaining items from the prior review (mutation in `run_evolution`, `run_swiss` / `run_grand_finale` parity, ZD generic payoffs) are still open — outside this mission's scope.

### PR
Not created — `-pr` flag not set. User to commit / open PR at their convenience.
