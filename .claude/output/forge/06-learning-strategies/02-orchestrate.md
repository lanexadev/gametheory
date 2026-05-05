# Orchestrate — 06-learning-strategies

## Acceptance Contracts
- **AC-1 Q-Learning** — `QLearning::new(alpha, gamma, epsilon, k)` builds a strategy that maintains a `HashMap<u32, [f64;2]>` Q-table inside `StrategyScratch::Custom`, encodes state from the last-`k` joint actions, picks ε-greedy actions, applies TD update `Q[s][a] += α(r + γ max_a' Q[s'][a'] - Q[s][a])` each turn, and resets state per match via `init_scratch`.
- **AC-2 Bayesian** — `BayesianOpponent::new(archetypes)` maintains a posterior probability per archetype, updates each turn via likelihood of the observed opponent action (Laplace-smoothed), and plays the action with highest expected payoff under the posterior-weighted predicted opponent move.
- **AC-3 Lookahead** — `Lookahead::new(depth, opponent_model)` does D-ply minimax search using `opponent_model` as the simulated adversary; picks the action maximising the discounted payoff sum over the rollout.
- **AC-4 Integration** — All three strategies registered in `get_generative_strategies()` (~5 variants each, ~15 total).
- **AC-5 Tests** — `tests/learning.rs` validates: (a) Q-learner converges toward defection vs `AlwaysDefect`; (b) Bayesian classifies `AlwaysCooperate` and exploits; (c) Lookahead-1 with TFT model cooperates against TFT; (d) determinism under fixed seed; (e) no panics over 200 turns.
- **AC-6** — `cargo build` and `cargo test` clean.

## Stories (TDD)
1. **S1 — Q-Learning** — define `QLearning` + `QLearningState`, write convergence test, then impl.
2. **S2 — Bayesian** — define `BayesianOpponent` + posterior state, write classification test, impl.
3. **S3 — Lookahead** — define `Lookahead` + minimax, write TFT-cooperation test, impl.
4. **S4 — Wire-up** — register variants in `get_generative_strategies`, smoke test.

## ADR
- **ADR-1**: All three live as separate modules. Reuse `StrategyScratch::Custom(Box<dyn Any+Send>)` so no `lib.rs` enum churn.
- **ADR-2**: Lookahead's opponent model is `Box<dyn Strategy>`; the minimax simulates the model's `next_move` (not stateful) — keeps the search cheap and side-effect free.
- **ADR-3**: All RNG-using paths take `&mut dyn RngCore` from the engine — preserves seed reproducibility.
- **ADR-4**: State `u32` packs up to 16 joint actions (K≤16), more than enough.
