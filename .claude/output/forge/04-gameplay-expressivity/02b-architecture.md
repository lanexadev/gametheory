# Architecture Decision Records

## ADR-01 — Trait extensibility via additive `Custom` variant
- **Context.** `StrategyScratch` is a closed enum (`lib.rs:31-52`). Adding a stateful strategy currently requires editing `lib.rs`. Q-learning, Bayesian opponent modeling, deep RL agents need arbitrary state.
- **Decision.** Add a `Custom(Box<dyn Any + Send>)` variant. Existing typed variants stay (zero overhead). New strategies allocate state in `init_scratch` and `downcast_mut` in `next_move_stateful`.
- **Alternatives.** Replace by `Box<dyn Any + Send>` only — loses ergonomics. Trait associated `type State` — breaks `Vec<Box<dyn Strategy>>` (object-unsafe).
- **Consequences.** Backward compatible. One indirection + downcast per turn for new stateful strategies. Anyone can add a stateful strategy without touching core.

## ADR-02 — ZD coefficients pinned to canonical payoffs
- **Context.** Press-Dyson coefficients depend on `(T,R,P,S)`. Strategies are constructed before `Game`. Threading payoffs through factory chains is invasive.
- **Decision.** ZD strategies hardcode coefficients for canonical Axelrod `(5,3,1,0)`. Doc-comment warns. Optional helper `zd::warn_if_nonstandard_payoffs(game)` users can call before running.
- **Consequences.** Mathematically valid only for canonical payoffs. Future work: parametrize the constructor by `(T,R,P,S)`.

## ADR-03 — `Neighborhood` enum with offset slices
- **Context.** Spatial step has Moore-8 nested loops (`lib.rs:444-475`). Need Moore + Von Neumann + Hex.
- **Decision.** `enum Neighborhood { Moore, VonNeumann, Hex }` with `fn offsets(self, even_row: bool) -> &'static [(i32, i32)]`. Spatial step iterates the chosen stencil. CLI flag `--topology`.
- **Consequences.** Hex is the only topology with row-parity branching. Default Moore preserves backward compatibility.

## ADR-04 — `RoundRobinReport` exposes pair matrix
- **Context.** `run_round_robin_per_individual` already collects `(i, j, sc1_per_turn, sc2_per_turn)` then folds and discards. Matrix export needs the full pair data.
- **Decision.** Refactor the collection step into `run_round_robin_report() -> RoundRobinReport { fitness, matrix, names }`. `run_round_robin_per_individual` becomes a thin wrapper for backward compat.
- **Consequences.** Single source of truth for pair scores. Matrix CSV export is pure formatting. Existing call sites unchanged.
