# Phase 1 — Fathom

## Tech Stack Manifest
- Rust edition 2024, single binary + library crate `game_theory`
- Deps: `rand 0.9`, `rand_chacha 0.9`, `rayon 1.10`, `clap 4.5`, `csv 1.3`, `serde 1.0`
- No tests, no bench harness yet

## Territory Map
- `src/lib.rs` (421 LOC): traits + Game/Tournament/SpatialTournament engines
- `src/strategies/mod.rs` (207 LOC): 600+ strategy variants generated via `FunctionalStrategy` closures
- `src/strategies/*.rs` (18 files): hand-written strategies; all use `&self` next_move with both histories
- `src/main.rs`: CLI plumbing, untouched by this refactor

## Historical Record
- Two prior FORGE missions (`01-fix-fidelity-bugs`, `02-compute-variance`) shipped fixes for items #1, #2, #3, #5, #6, #7, #8.
- Strategy trait signature changed: it now takes `rng: &mut dyn RngCore`. Gradual + OmegaTFT files updated to match but kept their O(N) re-scan.
- Current branch: `main`; clean tree apart from `.DS_Store` and `.claude/`.

## Hot path baseline
- Round robin: ~180k matches × 200 turns × `match_repetitions`. Inner cost dominated by `next_move` calls.
- ~17% of strategies are Gradual or Omega-Detector; ~30% of pairs include at least one. For those pairs each turn costs O(turn) instead of O(1) due to `for &act in opp_h` inside the closure → ~40k extra ops per affected match instead of 200.
- `SpatialTournament::step`: width × height × 8 matches per step, fully serial. `Tournament` already uses Rayon.
- `Game::play` allocates 4 history vectors of capacity = `iterations`; with both noises = 0 (common case) two of those are pure duplicates.
- Spatial step clones `Box<dyn Strategy>` per cell every step.

## Path Options

### Item #9 — incremental stateful strategies
- A. Replace trait with `&mut self`: requires cloning the Box for each match. Many allocations.
- **B. Add optional scratch parameter**: `StrategyScratch` enum + `next_move_stateful` default-delegating method. Stateless strategies untouched, only Gradual + Omega-Detector override. **Chosen.**
- C. `RefCell` interior mutability: not `Sync`, would require `Mutex` and serialise the hot path. Rejected.

### Item #10 — quadruple history allocations
- **A. Skip perception vectors when both noises are 0** + pre-size with `Vec::with_capacity(iterations)`. **Chosen** — preserves correctness with noise, common case avoids two vectors entirely.

### Item #11 — spatial parallelism
- Use Rayon `par_iter` over a flat `(y, x)` index space for the score pass and grid-update pass. `Game::play` already takes `match_seed` so no shared RNG state.

### Item #12 — index-based grid
- Replace `Vec<Vec<Box<dyn Strategy>>>` with a `Vec<Box<dyn Strategy>>` pool + `Vec<usize>` flat row-major grid. Cells store indices; best-neighbour propagation copies indices.

## Impact Map

| File | Item | Change scope |
|---|---|---|
| `src/lib.rs` | #9 | Add `StrategyScratch` enum, default trait methods, thread scratch through `Game::play` |
| `src/lib.rs` | #10 | Conditional vec allocation in `Game::play` |
| `src/lib.rs` | #11 | Use Rayon `par_iter` in `SpatialTournament::step` |
| `src/lib.rs` | #12 | Re-architect `SpatialTournament` to pool + indices |
| `src/strategies/gradual.rs` | #9 | Override `init_scratch` + `next_move_stateful` with O(1) update |
| `src/strategies/omega_tft.rs` | #9 | Same |
| `src/strategies/mod.rs` | #9 | Convert `Gradual (xN)` and `Omega-Detector (Thresh N)` families from `FunctionalStrategy` to dedicated parameterised structs (`GradualFamily`, `OmegaDetectorFamily`) |

Risk: trait extension (#9) — every existing implementation must compile unchanged because `next_move_stateful` defaults to delegating. Verify with `cargo build`.

## Open Questions
None — assumptions inline.
