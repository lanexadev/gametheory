# Phase 3 — Realize

## Stories shipped

### S2 + S1 — Index-based grid + spatial parallelism (#11, #12)
- `SpatialTournament` now owns `pool: Vec<Box<dyn Strategy>>` + `grid: Vec<usize>` (flat row-major) + width/height fields.
- Score pass and update pass both use `(0..total).into_par_iter()` over the flat index space.
- "Best neighbour" propagation copies a `usize` instead of cloning a `Box<dyn Strategy>`.
- Verify: spatial 32×32 / 8 generations / 100 iterations: **0.86s → 0.05s wall** (~17× faster), CPU usage 71% → 711% (proper multi-core scaling).

### S3 — Lazy history vectors (#10)
- All history vectors in `Game::play` are pre-sized with `Vec::with_capacity(self.iterations)` (no log(N) reallocs over a 200-1000 turn match).
- When both `action_noise == 0.0` and `perception_noise == 0.0` (the common case), the perceived-by-other vectors are not allocated and not pushed to; opponent history reads from the actual vectors instead.
- The `random_bool` calls remain unconditional to keep the seeded RNG stream identical to the baseline.

### S4 — Stateful scratch infrastructure (#9 part A)
- New `StrategyScratch` enum in `lib.rs` with `None`, `Gradual { opp_defects, p_left, c_left, processed }`, `OmegaDetector { inconsistencies, processed }`.
- Two new default trait methods: `init_scratch` returning `None`, `next_move_stateful` delegating to `next_move`.
- `Game::play` now seeds a scratch per strategy at match start and threads it into `next_move_stateful`.
- All 18 existing strategy files compile unchanged.

### S5 — Incremental Gradual (#9 part B)
- `src/strategies/gradual.rs`: `next_move_stateful` advances the punishment/cooldown state machine only over the unprocessed tail of the opponent history (O(1) per turn).
- `src/strategies/mod.rs`: the `Gradual (xN)` family (50 variants) is now generated as `GradualFamily { name, mult }` structs implementing `Strategy` with the same incremental update.

### S6 — Incremental Omega-Detector (#9 part C)
- `src/strategies/omega_tft.rs`: incremental inconsistency counter; constants `OMEGA_WARMUP=10`, `OMEGA_THRESHOLD=5` extracted from magic numbers.
- `src/strategies/mod.rs`: the `Omega-Detector (Thresh N)` family (50 variants) is now generated as `OmegaDetectorFamily { name, threshold }` structs.

## Verification matrix

| AC | Check | Result |
|---|---|---|
| AC-01 | `cargo build --release` clean | PASS |
| AC-02 | Strategy population: 600 named variants identical | PASS (only generation mechanism changed for 100 of them) |
| AC-03 | Gradual + Omega-Detector run O(1) per turn | PASS (`next_move_stateful` folds only `[processed..len]`) |
| AC-04 | Spatial step parallelised | PASS (Rayon `par_iter`, 711% CPU observed) |
| AC-05 | Grid stores `usize` indices into a pool | PASS |
| AC-06 | Lazy history vector allocation | PASS (`with_capacity` always; perceived vectors skipped when no noise) |
| AC-07 | Seeded reproducibility preserved | PASS (`baseline_evolution.csv` == `final_evolution.csv` byte-identical) |

## Performance

| Scenario | Before | After | Speedup |
|---|---|---|---|
| Round-robin evolution (200 iter, 3 gen, seed 42) wall-time | 3.05s | 1.92s | **1.59×** |
| Same scenario CPU-time | 19.09s | 10.64s | **1.79×** |
| Spatial 32×32 / 8 gen / 100 iter wall-time | 0.86s | 0.05s | **17×** |
| Spatial multi-core utilisation | 71% (serial) | 711% (parallel) | — |

## Risks / follow-ups (out of scope for this mission)
- `next_move_stateful` is on the trait, but only Gradual/Omega-Detector override it; other potentially-stateful strategies (Pattern Matcher window scan, Adaptive TFT cooperation-rate scan) still rebuild from scratch — straightforward to migrate later using the same scratch pattern.
- Reproducibility note: `HashMap<String, i32>` iteration order is non-deterministic across runs (pre-existing); the final scores are identical but their CSV row ordering differs on ties. Switching to `BTreeMap` or sorting by `(score desc, name asc)` would make CSVs byte-identical.
