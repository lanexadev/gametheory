# FORGE 05-core-improvements — Final Report

## Status: ✅ Complete (17/17 tests passing, release build clean)

## Changes by file

### `src/lib.rs`
- **Spatial seed bug** — `SpatialTournament` now carries a `step_count` field; each match seed mixes `(base_seed, step_id, cell_idx, neighbour_idx)` so consecutive steps produce distinct RNG sequences. Without this, every (A vs B) match at the same neighbourhood offset was replayed bit-identically each step.
- **Swiss normalization** — `run_swiss` accumulates **per-turn** scores (matching `run_round_robin`), then projects to an `iterations`-equivalent total for display compatibility. Discount-factor runs no longer silently demote strategies whose matches end early.
- **Evolution: roulette + mutation** — new `run_evolution_with_options(generations, reproduction_rate, mutation_rate, selection_temperature, mutation_pool)`:
  - `selection_temperature == 0.0` → legacy top-N truncation (deterministic).
  - `selection_temperature > 0.0` → softmax-weighted roulette over fitness (numerically stable via `(f - max) / T`).
  - `mutation_rate ∈ (0, 1]` → with that probability, a child slot is replaced by a fresh draw from `mutation_pool` (or current population if pool absent).
  - Seeded ChaCha8 derived from `Game.seed` ⊕ constant — runs reproducible.
  - Legacy `run_evolution` preserved as thin wrapper for back-compat.

### `src/strategies/mod.rs`
- **Adaptive TFT O(1)** — converted 50-variant family from closure-based `FunctionalStrategy` (O(N²) per match) to `AdaptiveTftFamily` struct using `StrategyScratch::Custom(Box<AdaptiveTftState>)`. Running cooperation count + processed index update incrementally → O(1) per turn.

### `src/main.rs`
- New CLI flags: `--mutation-rate`, `--selection-temperature`.
- ZD warning at startup if `(payoff_t, payoff_r, payoff_p, payoff_s) ≠ (5, 3, 1, 0)` and population contains `ZD-*` strategies.
- Evolution branch wires the new `run_evolution_with_options` and pre-builds a fresh full-pool when `mutation_rate > 0`.

### Tests added
- `tests/evolution.rs` — 3 tests: truncation determinism, mutation injects outside-population strategies, high-temperature roulette preserves diversity.
- `tests/spatial_seed.rs` — 1 test: 2-step trajectory under noise is reproducible across runs and counts always sum to N×N.
- `tests/swiss_normalization.rs` — 2 tests: scores reach iterations-scale despite discount, same seed → same results.
- `tests/adaptive_tft.rs` — 2 tests: 50 variants present, "Target 50%" defects correctly against AllD.

**Total: 9 new tests, 17/17 green.**

## Smoke verifications (binary)
- `--evolution --mutation-rate 0.1 --selection-temperature 0.5 --seed 42` → ranked output emerges, mutation+roulette functional.
- `--payoff-t 4 --payoff-r 3 --payoff-p 2 --payoff-s 1` → ZD warning printed correctly.
- `--spatial --topology hex --action-noise 0.05` → spatial dynamic runs, populations evolve.
- `--swiss --discount-factor 0.05` → scores in iterations-scale range.

## Items not addressed (out of declared scope)
- Pattern Matcher per-turn scratch (lower-impact than Adaptive TFT, the Pattern Matcher window scan is constant in `window`, not in `N`).
- Population diversity rebalancing (subjective design choice — left to user).
- Migration / kin recognition / small-world graphs (larger architectural change).
- True parametric mutation on closure-based families (would require trait-level `mutate` and a refactor of the 5+ closure families; structural mutation via fresh-pool sampling provides equivalent exploration in this finely-tessellated parameter space).
