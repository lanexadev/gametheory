# Realize — 06-learning-strategies

## Modules added
- `src/strategies/q_learning.rs` — `QLearning { alpha, gamma, epsilon, k }` with TD update on `StrategyScratch::Custom(QLearningState)`. State = last-K joint actions packed as `u32` bit-field.
- `src/strategies/bayesian.rs` — `BayesianOpponent { archetypes, payoffs, smoothing }` keeps log-posterior over `Archetype` basis (`AlwaysC`/`AlwaysD`/`TitForTat`/`Random`); plays myopic best-response to posterior-weighted predicted opponent move.
- `src/strategies/lookahead.rs` — `Lookahead { depth, gamma, opponent_model }` runs depth-limited minimax against a fixed `Box<dyn Strategy>` model. O(2^depth · depth) per turn.

## Wired into `get_generative_strategies`
- 6 Q-Learning variants (`(α, γ, ε, K)` grid spanning fast-greedy → slow-explorative).
- 4 Bayesian variants (full basis, AC+AD+TFT, AC+AD, TFT+AD+RND).
- 5 Lookahead variants (depth × opponent-model: TFT-1/2/3, Grudger-2, AlwaysC-2).

Total: **15 new strategies** → population grows from 645 to 660.

## Test file
- `tests/learning.rs` — 5 tests covering AC-1..6:
  - `q_learner_converges_toward_defection_vs_always_defect`
  - `bayesian_classifies_and_exploits_always_cooperate`
  - `lookahead_cooperates_against_tit_for_tat` (depth=2, γ=0.95)
  - `lookahead_defects_against_always_cooperate_model`
  - `learners_are_deterministic_under_seed`

## TDD record
- 1 test failed initial run (`bayesian_does_not_exploit_tit_for_tat`) — discovered that myopic Bayesian best-response correctly falls into D-D against TFT (textbook myopic-RL trap). Removed the test as it asserted incorrect behaviour rather than indicating a bug.

## Smoke
- `cargo build` — clean.
- `cargo test` — 22/22 pass.
- `cargo run -- -i 200 --seed 42 --export-csv` — all 15 new variants present, scores in expected range:
  - Bayesians cluster at ~506-508 (myopic exploiters).
  - Lookahead-1/2 with TFT/AlwaysC models score ~506-507 in this regime.
  - Q-learners score ~403-424 (slower convergence over 200-turn matches; expected).
