# FORGE Mission: 03-perf-scaling

**Created:** 2026-05-04
**Task:** Performance / scaling — items #9, #10, #11, #12 of the prior audit
**Type:** refactor
**Mode:** Solo (economy)

## Flags
- Auto: true
- Guard: false
- TDD: false
- Save: true
- Swarm: false
- Branch: false
- PR: false

## User Request
"Occupe toi de : Performance / scaling"

Targeted issues from the previously-discussed improvement list:
- **#9** Stateful strategies in O(N²) global — Gradual (`mod.rs:139-153`) and Omega-Detector (`mod.rs:182-191`) rescan the whole opponent history every turn (~N²/2 ops per match instead of N).
- **#10** Quadruple history allocated per match — `lib.rs:102-105` keeps four `Vec<Action>` (`h1_actual`, `h2_actual`, `h1_perceived_by_2`, `h2_perceived_by_1`); most strategies only use a small window.
- **#11** Spatial step not parallelised — `SpatialTournament::step` (`lib.rs:359-409`) is fully serial; `Tournament` already uses Rayon.
- **#12** Grid = `Vec<Vec<Box<dyn Strategy>>>` + clones — heavy allocation each step (`lib.rs:387-407`); use indices into a strategy pool.

## Acceptance Criteria
- [ ] AC-01: `cargo build --release` succeeds; no public API regressions for the binary CLI flags.
- [ ] AC-02: Strategy population (`get_generative_strategies`) keeps producing the same number of variants and the same names.
- [ ] AC-03: Item #9 — Gradual and Omega-Detector run in O(1) extra work per turn (no full opponent-history rescan inside the hot path).
- [ ] AC-04: Item #11 — `SpatialTournament::step` parallelises the per-cell match-evaluation loop with Rayon.
- [ ] AC-05: Item #12 — Grid stores `usize` indices into a shared `Vec<Box<dyn Strategy>>` pool, removing per-step `Box` clones.
- [ ] AC-06: Item #10 — `Game::play` either drops the unused fourth history vector OR caps history allocations under no-noise mode (whichever is cleanly safe).
- [ ] AC-07: With a fixed `--seed`, evolutionary runs produce identical population trajectories before vs. after the refactor (reproducibility intact).

## Progress
- [x] Phase 0 — Triage
- [ ] Phase 1 — Fathom
- [ ] Phase 2 — Orchestrate
- [ ] Phase 3 — Realize
- [ ] Phase 4 — Guard (skipped, no `-x`)
- [ ] Phase 5 — Export
