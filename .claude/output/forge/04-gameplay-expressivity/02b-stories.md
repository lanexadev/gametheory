# Story Backlog

| Story | Title | AC | Files | Complexity | Status |
|---|---|---|---|---|---|
| **S1** | Trait extensible (`Custom` scratch) | AC-01 | `lib.rs`, `tests/extensible_state.rs` | S | pending |
| **S2** | ZD Extortion + ZD Generous | AC-02a, AC-02b | `strategies/zd.rs`, `strategies/mod.rs`, `tests/zd.rs` | M | pending |
| **S3** | WSLS stochastique family | AC-03 | `strategies/wsls.rs`, `strategies/mod.rs`, `tests/wsls.rs` | S | pending |
| **S4** | `Neighborhood` enum + CLI flag | AC-04 | `lib.rs`, `main.rs`, `tests/topology.rs` | M | pending |
| **S5** | `RoundRobinReport` + matrix CSV | AC-05 | `lib.rs`, `main.rs`, `tests/matrix.rs` | M | pending |
| **S6** | Final gate | AC-06 | `cargo test`, `cargo build --release` | trivial | pending |

## Dependencies
- S1 must land before S2/S3 only if those choose to keep typed state. S2 (ZD) is stochastic memory-1 → no scratch needed → S1 not strictly blocking. Order kept by theme.
- S4 and S5 are independent of S1–S3.
- S6 depends on all others.
