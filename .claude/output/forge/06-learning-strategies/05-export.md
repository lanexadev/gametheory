# Export — 06-learning-strategies

## Status
✅ All ACs (AC-1..AC-6) satisfied. 22/22 tests pass.

## Files modified / added
- `src/strategies/mod.rs` — registered 3 modules + 15 variants.
- `src/strategies/q_learning.rs` — new.
- `src/strategies/bayesian.rs` — new.
- `src/strategies/lookahead.rs` — new.
- `tests/learning.rs` — new.

## Suggested commit
```
feat: add Q-Learning, Bayesian, and Lookahead learning strategies

Three orthogonal "smart" archetypes built on StrategyScratch::Custom:

- Q-Learning: model-free RL with ε-greedy over Q(state, action), where
  state encodes the last-K joint actions. Tabular, in-match learning.
- Bayesian: posterior over archetype basis (AC, AD, TFT, Random) with
  Laplace smoothing in log-space; myopic best-response on expected payoff.
- Lookahead: depth-limited minimax against a fixed opponent model.

Adds 15 variants (6 Q-Learning, 4 Bayesian, 5 Lookahead) to the
tournament population, plus 5 acceptance tests in tests/learning.rs.
All randomness routed through &mut dyn RngCore — fully reproducible
under --seed.
```

## Not pushed
No `-b` or `-pr` flags — branch and PR creation skipped per user invocation.
