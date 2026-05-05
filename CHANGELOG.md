# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
once a `v0.1.0` tag is published.

The version anchors below are retroactive, mapped to the themed commits on
`main`. Tag them locally with `git tag vX.Y.Z <sha>` if you want a clean
SemVer history.

## [Unreleased]

### Added
- Project documentation: `README.md` (full rewrite), `CHANGELOG.md`,
  `CONTRIBUTING.md`.
- Git Flow branch model: `main` (release/stable) and `develop`
  (integration/unstable).

---

## [0.7.0] — 2026-05-05  *(commit `bb53ddd`)*

### Added
- `.gitignore` ignores `.DS_Store` and `.claude/tmp/`.
- `.claude/output/forge/` — per-task plan and decision artifacts (Fathom
  synthesis, contracts/stories, realize/export reports) committed as a
  record of how each themed change set was scoped and built.

---

## [0.6.0] — 2026-05-05  *(commit `232f473`)* — Evolution Diversity & Tournament Fidelity

### Added
- `Tournament::run_evolution_with_options(generations, repro_rate,
  mutation_rate, selection_temperature, mutation_pool)`.
  - **Softmax roulette**: `selection_temperature > 0` switches from top-N
    truncation to softmax-weighted sampling. Higher T preserves
    diversity; lower T converges toward truncation.
  - **Mutation**: `mutation_rate > 0` replaces a child slot with a fresh
    draw from a global strategy pool — the population is no longer
    bounded forever by its initial set.
- CLI flags: `--mutation-rate`, `--selection-temperature`.
- `tests/adaptive_tft.rs`, `tests/evolution.rs`, `tests/spatial_seed.rs`,
  `tests/swiss_normalization.rs`.
- Startup warning when ZD strategies run under non-canonical payoffs
  (Press-Dyson invariant is closed-form only for `(5,3,1,0)`).

### Changed
- **Swiss tournament normalisation**: per-round contribution is now
  `score / turns_played`, projected back to `iterations`-equivalent for
  display. Matches whose `discount_factor` cuts them short are no longer
  silently demoted.
- **Adaptive TFT family** rewritten from O(N) per turn to O(1) using
  `StrategyScratch::Custom` (running coop count + processed cursor).
  Mirrors the existing `Gradual` / `OmegaDetector` pattern.

---

## [0.5.0] — 2026-05-05  *(commit `4d5affc`)* — Learning Strategies

### Added
- `src/strategies/q_learning.rs` — tabular model-free RL, ε-greedy over
  `Q(state, action)` with state = last-K joint moves packed as a `u32`
  bit-field. Per-turn TD update; converges in-match.
- `src/strategies/bayesian.rs` — log-space posterior over an archetype
  basis (`AlwaysC`, `AlwaysD`, `TitForTat`, `Random`) with Laplace
  smoothing. Plays the expected-value best response to the
  posterior-weighted predicted opponent move.
- `src/strategies/lookahead.rs` — depth-limited minimax against any
  `Box<dyn Strategy>` as a fixed opponent model. With TFT model and
  depth ≥ 2 it learns to cooperate; with `AlwaysC` model it learns to
  defect.
- 15 default variants in `get_generative_strategies()` (6 Q-Learning,
  4 Bayesian, 5 Lookahead).
- `tests/learning.rs` — 5 acceptance tests covering convergence,
  classification, cooperation emergence, and seed determinism.

---

## [0.4.0]  *(commit `dda0987`)* — Gameplay Expressivity

### Added
- **Zero-Determinant strategies** (`src/strategies/zd.rs`):
  `Memory1Stochastic` parameterised by `(p_cc, p_cd, p_dc, p_dd)` plus
  Press-Dyson and Stewart-Plotkin closed-forms (`zd_extortion(chi)`,
  `zd_generous(chi)`).
- **Stochastic Win-Stay-Lose-Shift** family
  (`src/strategies/wsls.rs`): independent `(p_stay_win, p_switch_loss)`.
- **Configurable spatial topology**: `Neighborhood::{Moore, VonNeumann,
  Hex}`. CLI flag `--topology`.
- **Pair-score matrix export** (`--export-matrix`): full N×N CSV of
  mean per-turn scores (i vs j).
- **Extensible scratch slot**: `StrategyScratch::Custom(Box<dyn Any +
  Send>)` lets strategies stash typed state without modifying `lib.rs`.
- Tests: `tests/zd.rs`, `tests/wsls.rs`, `tests/topology.rs`,
  `tests/matrix.rs`, `tests/extensible_state.rs`.

---

## [0.3.0]  *(commit `6465640`)* — Performance

### Added
- Parallel spatial step (`rayon` over the flat-grid index space).

### Changed
- **Stateful strategies are O(1) per turn**. `Gradual` and `OmegaTFT`
  now thread their counters through `StrategyScratch` instead of
  rescanning the entire opponent history each turn.
- **Spatial grid storage**: `Vec<usize>` indices into a deduplicated
  strategy pool (`Vec<Box<dyn Strategy>>`) instead of
  `Vec<Vec<Box<dyn Strategy>>>` with cell-by-cell clones. The per-step
  "copy best neighbour" pass becomes a trivial memcpy.
- **Lazy history allocation**: histories pre-sized to `iterations` (no
  realloc); perceived-history vectors skipped entirely when both noise
  sources are zero.

---

## [0.2.0]  *(commit `89d5075`)* — Compute Fidelity

### Added
- `Game::validate()` enforces Axelrod's IPD constraints:
  `T > R > P > S` and `2R > T + S`. Without the second, an alternating
  C/D, D/C strategy outperforms mutual cooperation and the dilemma
  collapses.
- `Tournament::with_match_repetitions(n)` — replays each pair `n` times
  with deterministically-derived sub-seeds, then averages. Variance
  control under noise.
- `Tournament::with_include_self_play(bool)` — Axelrod's original
  tournament includes self-play; this lets you opt out.
- CLI flag `--no-self-play`, `--repetitions`.

### Changed
- All scores are now **per-turn normalised** internally and projected
  back to an `iterations`-equivalent for display, so
  `discount_factor > 0` no longer demotes early-ending matches.

---

## [0.1.0]  *(commit `d2ba7d0`)* — Fidelity Fixes

### Fixed
- **Pattern Matcher** no longer always returns `Defect` regardless of
  the cycle prediction (both branches of the `if` had the same body).
- **RNG seeding now reaches every strategy**: stochastic strategies
  used to call `rand::rng()` (the global thread RNG), ignoring
  `--seed`. The `Strategy` trait now passes a `&mut dyn RngCore` from
  the engine so `--seed` is honoured end-to-end.
- **Per-individual fitness in evolution**: previously the round-robin
  result was aggregated by strategy *name*, so a strategy present `N`
  times in the population scored `N×`. Minorities went extinct
  mechanically rather than because of intrinsic fitness. Evolution now
  uses `score_total_of_individual / matches_played`.

---

## [0.0.1]  *(commit `f7801bd`)* — Massive Strategy Expansion

### Added
- 600+ strategy variants across the `Reactive (P,Q)`,
  `Pattern Matcher`, `Adaptive TFT`, `Backstabber`, and `Bully`
  families.
- Foundation `Tournament`, `SpatialTournament`, evolutionary loop,
  CSV export.

[Unreleased]: https://github.com/lanexadev/gametheory/compare/main...HEAD
[0.7.0]: https://github.com/lanexadev/gametheory/commit/bb53ddd
[0.6.0]: https://github.com/lanexadev/gametheory/commit/232f473
[0.5.0]: https://github.com/lanexadev/gametheory/commit/4d5affc
[0.4.0]: https://github.com/lanexadev/gametheory/commit/dda0987
[0.3.0]: https://github.com/lanexadev/gametheory/commit/6465640
[0.2.0]: https://github.com/lanexadev/gametheory/commit/89d5075
[0.1.0]: https://github.com/lanexadev/gametheory/commit/d2ba7d0
[0.0.1]: https://github.com/lanexadev/gametheory/commit/f7801bd
