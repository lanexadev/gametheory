# GameTheory — High-Performance Iterated Prisoner's Dilemma Engine

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A research-grade Rust simulation engine for the **Iterated Prisoner's Dilemma** in the lineage of Axelrod (1984), Press & Dyson (2012), and the modern evolutionary-game-theory literature. Designed for honest, reproducible experimentation: seedable RNG end-to-end, validated payoffs, normalised scoring, and per-individual fitness so evolutionary runs measure intrinsic strategy quality rather than population size.

660+ strategy variants spanning the full taxonomy — from `AlwaysCooperate` to **Zero-Determinant extortioners**, **stochastic Win-Stay-Lose-Shift**, **Q-Learning**, **Bayesian opponent classifiers**, and **depth-limited minimax**.

---

## Table of contents

- [Quick start](#quick-start)
- [What you get](#what-you-get)
- [Tournament modes](#tournament-modes)
- [Strategy catalogue](#strategy-catalogue)
- [Architecture](#architecture)
- [Reproducibility & fidelity](#reproducibility--fidelity)
- [Performance](#performance)
- [Analysis pipeline](#analysis-pipeline)
- [Contributing](#contributing)

---

## Quick start

### Build

```bash
cargo build --release
```

Requires the Rust 2024 edition (rustc ≥ 1.85). Dependencies: `rand`, `rand_chacha`, `rayon`, `clap`, `csv`, `serde`.

### Run a round-robin

```bash
cargo run --release -- --iterations 200 --seed 42
```

### Run an evolutionary tournament with diversity-preserving selection

```bash
cargo run --release -- \
  --evolution --generations 500 \
  --iterations 200 --seed 42 \
  --reproduction-rate 0.10 \
  --selection-temperature 5.0 \
  --mutation-rate 0.02 \
  --export-csv results/run.csv
```

`selection-temperature > 0` switches from top-N truncation to softmax roulette; `mutation-rate > 0` injects fresh strategies from the global pool each generation, so the population is no longer bounded forever by the initial set.

### Run a spatial 2D cellular automaton

```bash
cargo run --release -- \
  --spatial --grid-size 50 --generations 200 \
  --topology hex --seed 42
```

Topology accepts `moore` (8-neighbour, default), `vonneumann` (4-neighbour orthogonal), or `hex` (6-neighbour offset).

### Inspect the full pair-score matrix

```bash
cargo run --release -- \
  --iterations 200 --seed 42 \
  --export-matrix results/pairs.csv
```

The matrix is N×N of mean per-turn scores — strategy `i` against strategy `j`. Useful for clustering, Nash equilibrium detection, and "who exploits whom" analysis.

---

## What you get

- **Strategy diversity**: 660+ variants, 19 distinct families. Stateless functional combinators and stateful agents share a single `Strategy` trait.
- **Stateful learning agents**: model-free (Q-Learning), model-based (Bayesian opponent classification), and decision-theoretic (depth-limited minimax) — all using a typed `StrategyScratch::Custom` slot for per-match state.
- **Honest evolution**: per-individual fitness (not aggregated by name), softmax selection with configurable temperature, mutation drawn from a global strategy pool.
- **Spatial dynamics**: parallel cellular automaton with toroidal wraparound, configurable neighbourhood topology, deterministic per-step seeding.
- **Noise models**: independent action-noise (execution errors) and perception-noise (misunderstandings).
- **Validated payoffs**: rejects payoff matrices that violate `T > R > P > S` or `2R > T + S`. ZD strategies emit a warning when run under non-canonical payoffs (Press-Dyson invariant only holds for `(5,3,1,0)`).
- **Score normalisation**: every metric is per-turn; a discount-factor cut short doesn't silently demote strategies whose matches end early.
- **Reproducibility**: `--seed` is honoured end-to-end through `ChaCha8Rng`. Every match, every round, every spatial step uses a deterministically-derived sub-seed.
- **Parallel everything**: round-robin, Swiss, and spatial step all use `rayon`.

---

## Tournament modes

| Mode | Flag | Description |
|------|------|-------------|
| Round-robin | (default) | Every strategy plays every other (and itself unless `--no-self-play`). |
| Swiss | `--swiss --swiss-rounds N` | Pairing by current score over N rounds. |
| Grand finale | `--finale` | Top-N from round-robin replay at 5× iterations. |
| Evolution | `--evolution` | Generational fitness-driven reproduction. |
| Spatial | `--spatial --grid-size N` | 2D toroidal cellular automaton. |

### Key flags

| Flag | Default | Purpose |
|------|---------|---------|
| `--iterations N` | 200 | Turns per match. |
| `--repetitions N` | 1 | Replays of each pair (variance reduction under noise). |
| `--action-noise p` | 0.0 | Probability a chosen action is flipped before scoring. |
| `--perception-noise p` | 0.0 | Probability the opponent observes the wrong action. |
| `--discount-factor p` | 0.0 | Per-turn match-end probability (Axelrod's "shadow of the future"). |
| `--no-self-play` | off | Exclude diagonal pairs from round-robin. |
| `--seed N` | random | Seeds RNG for full reproducibility. |
| `--payoff-t/r/p/s` | 5/3/1/0 | Override the canonical Axelrod payoffs. |
| `--reproduction-rate p` | 0.20 | Fraction of population replaced per generation. |
| `--selection-temperature T` | 0.0 | `0` = truncation; `>0` = softmax roulette. |
| `--mutation-rate p` | 0.0 | Probability a child slot is drawn from the global pool. |
| `--topology` | moore | Spatial neighbourhood: `moore`, `vonneumann`, `hex`. |
| `--export-csv path` | — | Final scores + evolution history. |
| `--export-matrix path` | — | Full N×N pair-score CSV. |

---

## Strategy catalogue

All families implement the `Strategy` trait. Stochastic variants are seedable; stateful variants use `StrategyScratch::Custom` for O(1) per-turn updates.

| Family | Variants | Notes |
|---|---|---|
| Always Cooperate / Always Defect | 2 | Baselines. |
| Tit-for-Tat & variants | ~5 | Classic, suspicious, two-tats, forgiveness. |
| Pavlov / WSLS | 26 | Deterministic Pavlov + 25-variant stochastic Win-Stay-Lose-Shift grid. |
| Grudger / Soft Grudger | 2 | Permanent vs decaying retaliation. |
| Forgiving TFT | 100 | Probabilistic forgiveness over [1%, 100%]. |
| Reactive (P, Q) | 100 | Memory-1 stochastic (10×10 grid). |
| Pattern Matcher | 10 | Cycle-detection over a sliding window. |
| Adaptive TFT | 50 | Targets a configurable opponent-coop rate. |
| Backstabber | 50 | Cooperates until turn N, then defects forever. |
| Bully / Paradoxical | 50 | The inverse of TFT. |
| Gradual | 50 | Punishment scales with cumulative defections. |
| Handshake | 50 | Recognition codes that gate cooperation. |
| Omega-Detector / OmegaTFT | ~51 | Detects "rope-a-dope" cooperators that defect mid-match. |
| Biased Random | 100 | Ignores history, fixed cooperation probability. |
| Detective | 1 | Probes early then commits to TFT or AlwaysD. |
| Joss / Statistical / Alternator / Handshake / Suspicious | several | Classical reference strategies. |
| **ZD-Extortion / ZD-Generous** | 10 | Press-Dyson 2012 + Stewart-Plotkin 2013 closed forms. |
| **Q-Learning** | 6 | Tabular RL, ε-greedy, state = last-K joint actions. |
| **Bayesian opponent classifier** | 4 | Log-space posterior over `{AC, AD, TFT, Random}` archetypes. |
| **Lookahead minimax** | 5 | Depth-limited search against a fixed `Box<dyn Strategy>` model. |

Total: **660+ strategies** in the default population.

---

## Architecture

```
src/
├── lib.rs                  // Strategy trait, Game, Tournament, SpatialTournament,
│                           // RoundRobinReport, StrategyScratch, Neighborhood
├── main.rs                 // CLI (clap)
└── strategies/
    ├── mod.rs              // get_generative_strategies()  ← register here
    ├── always_cooperate.rs
    ├── always_defect.rs
    ├── tit_for_tat.rs
    ├── pavlov.rs
    ├── grudger.rs
    ├── soft_grudger.rs
    ├── handshake.rs
    ├── tit_for_two_tats.rs
    ├── suspicious_tit_for_tat.rs
    ├── joss.rs
    ├── tit_for_tat_with_forgiveness.rs
    ├── statistical.rs
    ├── alternator.rs
    ├── detective.rs
    ├── gradual.rs
    ├── omega_tft.rs
    ├── zd.rs               // Memory-1 stochastic + Press-Dyson closed forms
    ├── wsls.rs             // Stochastic Win-Stay-Lose-Shift
    ├── q_learning.rs       // Tabular Q-learner
    ├── bayesian.rs         // Posterior-weighted best-response
    └── lookahead.rs        // Depth-limited minimax
tests/                       // 22 integration tests (cargo test)
```

### The `Strategy` trait

```rust
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn next_move(&self, my: &[Action], opp: &[Action], rng: &mut dyn RngCore) -> Action;
    fn init_scratch(&self) -> StrategyScratch { StrategyScratch::None }
    fn next_move_stateful(&self, my: &[Action], opp: &[Action],
                          scratch: &mut StrategyScratch, rng: &mut dyn RngCore) -> Action {
        self.next_move(my, opp, rng)
    }
    fn clone_box(&self) -> Box<dyn Strategy>;
}
```

The default `next_move_stateful` impl forwards to the stateless `next_move`, so existing strategies are not forced to opt into scratch.

### `StrategyScratch::Custom`

For stateful agents that don't fit the predefined `Gradual` / `OmegaDetector` shapes:

```rust
pub enum StrategyScratch {
    None,
    Gradual { /* ... */ },
    OmegaDetector { /* ... */ },
    Custom(Box<dyn Any + Send>),  // anything stateful: Q-tables, posteriors, …
}
```

A strategy returns its typed state from `init_scratch`, then `downcast_mut` it inside `next_move_stateful`. See `tests/extensible_state.rs` for a minimal example.

### Adding a new strategy

1. Create `src/strategies/my_strategy.rs` and implement `Strategy`.
2. `pub mod my_strategy;` in `src/strategies/mod.rs`.
3. Register variants inside `get_generative_strategies()`.
4. Add an integration test under `tests/`.

---

## Reproducibility & fidelity

Decisions taken explicitly to make results trustworthy:

- **Seeded RNG everywhere**. `Game::play` derives a per-match `ChaCha8Rng` from the user seed; `Tournament::run_round_robin_report` derives a unique sub-seed per `(i, j, repetition)`; spatial steps mix the step counter into the per-cell seed.
- **Per-individual fitness**. Earlier versions aggregated scores by strategy *name*, so a strategy present `N` times in the population scored `N×` — minorities went extinct mechanically. Evolution now uses `score_total_of_individual / matches_played`.
- **Per-turn score normalisation**. Both `run_round_robin` and `run_swiss` normalise by `history.len()` (real turn count after `discount_factor`), then re-project to an `iterations`-equivalent total for backward-compatible display.
- **Validated payoffs**. `Game::validate()` rejects matrices that aren't IPDs (`T > R > P > S` and `2R > T + S`).
- **Optional self-play**. Axelrod's original tournament included it; our default matches that, but `--no-self-play` is available.
- **ZD payoff warning**. The Press-Dyson invariant is closed-form for `(5,3,1,0)` only — main warns when ZD strategies run under custom payoffs.

---

## Performance

- Round-robin and spatial step are parallelised with `rayon`.
- Stateful strategies (Gradual, OmegaTFT, Adaptive TFT) use `StrategyScratch` for O(1) per-turn updates instead of O(N) history rescans → O(1) per turn × N turns = O(N) per match instead of O(N²).
- The spatial grid stores `Vec<usize>` indices into a deduplicated strategy pool (rather than `Vec<Vec<Box<dyn Strategy>>>`), which makes the per-step "copy best neighbour" pass a trivial memcpy and avoids `clone_box` calls in the hot path.
- Histories are pre-sized to `iterations` (no log-N reallocations during a match) and the perceived-history vectors are skipped entirely when both noise sources are zero.

A 660-strategy round-robin at 200 iterations runs in ~1–2 seconds in release mode on a modern laptop. Evolutionary runs of 500 generations × 200 iterations on the full population take ~10 minutes.

---

## Analysis pipeline

For end-to-end "compile → simulate → visualise":

```bash
./run_complete_analysis.sh -g 500 -i 200 -r 0.10 -a 0.04 -p 0.02 -s 999
```

Generates a CSV of population history per generation and a stacked-area chart via `visualize_evolution.py` (requires `pip install pandas matplotlib`). Outputs land in `results/`.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). The TL;DR:

- `main` is the **stable / release** branch; **never push directly**.
- `develop` is the **integration / unstable** branch; PRs land here.
- Branch naming: `feature/<short-name>`, `fix/<short-name>`, `refactor/<short-name>`, `perf/<short-name>`, `docs/<short-name>`, `hotfix/<short-name>`.
- Commits use the project's themed-line convention: `Topic: short summary` followed by a body with bullets.
- `cargo test` must pass before opening a PR.
- New strategies should ship with at least one acceptance test in `tests/`.

See [CHANGELOG.md](CHANGELOG.md) for the version history.

---

## License

MIT — see [LICENSE](LICENSE).

## References

- Axelrod, R. (1984). *The Evolution of Cooperation*.
- Press, W. H. & Dyson, F. J. (2012). *Iterated Prisoner's Dilemma contains strategies that dominate any evolutionary opponent*. PNAS.
- Stewart, A. J. & Plotkin, J. B. (2013). *From extortion to generosity, evolution in the Iterated Prisoner's Dilemma*. PNAS.
- Hilbe, C., Nowak, M. A. & Sigmund, K. (2013). *Evolution of extortion in Iterated Prisoner's Dilemma games*. PNAS.
- Nowak, M. A. & Sigmund, K. (1993). *A strategy of win-stay, lose-shift that outperforms tit-for-tat in the Prisoner's Dilemma game*. Nature.
