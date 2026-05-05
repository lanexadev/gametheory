# Fathom — 06-learning-strategies

## Tech Stack
Rust 2024 edition; deps: `rand 0.9`, `rand_chacha 0.9`, `rayon 1.10`, `csv 1.3`, `clap 4.5`, `serde 1.0`. No std-extra heap deps.

## Territory Map
- `src/lib.rs` — `Strategy` trait, `StrategyScratch` enum (with `Custom(Box<dyn Any+Send>)` slot for typed user state), `Game::play` threads scratch via `next_move_stateful`, `Tournament`, `SpatialTournament`.
- `src/strategies/` — one file per strategy archetype; `mod.rs` aggregates and exposes `get_generative_strategies()`.
- `tests/extensible_state.rs` — reference for the `Custom` scratch pattern (typed downcast).

## Existing patterns to reuse
- `Memory1Stochastic` (`zd.rs`) → 4-prob memory-1 Markov stochastic with RNG.
- `AdaptiveTftFamily` (`mod.rs:90`) → exemplar of `Custom(Box<dyn Any>)` scratch with O(1) per-turn update.
- All RNG sourced from `&mut dyn RngCore` parameter — never `rand::rng()`.
- `clone_box` returns `Box<dyn Strategy>` for tournament cloning.

## Path
Implement three orthogonal "learning" archetypes that exploit `StrategyScratch::Custom`:
1. **Q-Learning** — model-free RL, ε-greedy, state = last-K joint actions.
2. **Bayesian opponent classifier** — Dirichlet posterior over archetype hypotheses, best-response.
3. **Lookahead/minimax** — D-ply tree search with a parameterised base opponent model.

## Impact Map
- New: `src/strategies/q_learning.rs`, `src/strategies/bayesian.rs`, `src/strategies/lookahead.rs`, `tests/learning.rs`.
- Touch: `src/strategies/mod.rs` (`pub mod ...` + register in `get_generative_strategies`).
- Risk: low. Trait surface unchanged; scratch already extensible; no changes to lib.rs core paths.

## Open Questions (auto-mode → assumed)
- K (Q-learning history window) → **2** (4²=16 states, balances expressivity vs sample efficiency on 200-turn matches).
- Default opponent model for Lookahead → **TitForTat** (most rational adversarial baseline).
- Bayesian archetype set → **{AlwaysC, AlwaysD, TFT, Random}** (4-element basis spanning the IPD axes).
