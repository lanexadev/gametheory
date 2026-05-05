# Phase 2 — Orchestrate

## Stories (executed in this order; each must compile and produce identical results vs. baseline given a fixed seed)

1. **S1 — Spatial parallelism (#11)**: parallelise `SpatialTournament::step` score and update passes with Rayon.
2. **S2 — Index-based spatial grid (#12)**: pool of strategies + flat `Vec<usize>` grid; eliminate per-cell `Box::clone`.
3. **S3 — Lazy history vectors (#10)**: pre-size all history vectors and skip the perceived-history vectors when both noises are 0.
4. **S4 — Stateful scratch infrastructure (#9 part A)**: add `StrategyScratch` enum + `init_scratch` + `next_move_stateful` default methods; thread scratch through `Game::play`. No strategy logic change yet — verifies the trait extension is compatible.
5. **S5 — Incremental Gradual (#9 part B)**: override scratch methods on `Gradual` and convert the `Gradual (xN)` family in `mod.rs` to a dedicated `GradualFamily` struct.
6. **S6 — Incremental Omega-Detector (#9 part C)**: same for `OmegaTFT` and the `Omega-Detector (Thresh N)` family.

## Architecture decisions

- **ADR-1 (#9)**: opt-in scratch via separate trait method with default delegation. Stateless strategies remain untouched. Avoids `&mut self` clone storms and `Mutex` overhead.
- **ADR-2 (#11)**: keep `match_seed: None` in spatial; spatial is intrinsically stochastic and we don't claim reproducibility for it today. Parallelism is therefore safe to introduce without seed plumbing changes.
- **ADR-3 (#12)**: pool strategies by name (first occurrence wins). Names are unique across `get_generative_strategies` so this is well-defined.

## Test/verification gates
- After each story: `cargo build --release` (no warnings about unused params for trait defaults).
- After S6: full `cargo build --release` + run a small evolution with a fixed seed and compare CSV against a pre-refactor baseline. Same outputs → green.
