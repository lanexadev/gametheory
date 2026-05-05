# Phase 1 — Fathom (synthesis)

## Tech Stack Manifest
```json
{
  "language": "rust",
  "edition": "2024",
  "deps": ["rand 0.9", "rand_chacha 0.9", "rayon 1.10", "serde 1", "clap 4.5", "csv 1.3"],
  "concurrency": "rayon (data-parallel)",
  "rng": "ChaCha8Rng (seedable)"
}
```

## Territory Map

**Core (`src/lib.rs`)**
- `StrategyScratch` enum **closed** (`lib.rs:31-52`): `None`, `Gradual{...}`, `OmegaDetector{...}`. Adding a new stateful strategy requires editing `lib.rs`. → Target #13.
- Trait `Strategy` (`lib.rs:54-76`): `name`, `next_move(&self, my_h, opp_h, &mut dyn RngCore)`, `init_scratch`, `next_move_stateful`, `clone_box`.
- `Game::play` (`lib.rs:148-212`) threads `ChaCha8Rng` per match and one `StrategyScratch` per side.
- `Tournament::run_round_robin_per_individual` (`lib.rs:248-288`) **already collects pair-level results** in `Vec<(i, j, sc1_per_turn, sc2_per_turn)>` then folds into `Vec<f64>` and discards. → Target #16 reuses this.
- `SpatialTournament` (`lib.rs:402-489`): flat `Vec<usize>` grid + pool, **Moore-8 hardcoded** in nested `for dy in -1..=1 / for dx in -1..=1` (`lib.rs:444-455` and `460-475`). → Target #15.

**Strategies (`src/strategies/*.rs`)**
- 16 standalone files; `mod.rs` hosts 600+ generative variants.
- Pavlov is deterministic only.

**CLI (`src/main.rs`)**
- No `--topology`, no `--export-matrix`.

## Historical Record
| Commit | Effect on this phase's surface |
|---|---|
| `6465640` | Closed-enum `StrategyScratch` introduced — extension hot-spot. |
| `89d5075` | Per-individual fitness collects pair data but discards. |
| `d2ba7d0` | Trait now takes `&mut dyn RngCore` — clean entry point for stochastic ZD/WSLS. |

## Path Options

### Axis A — Trait extensibility (#13)
| Option | Pros | Cons |
|---|---|---|
| **A1.** Replace `StrategyScratch` by `Box<dyn Any + Send>` | Fully open | Loses typed-scratch ergonomics; downcast everywhere |
| **A2.** Add `Custom(Box<dyn Any + Send>)` variant | Backward compatible; existing variants stay typed | One downcast per stateful new strategy |
| **A3.** Trait associated `type State` | Compile-time typed | Breaks `Vec<Box<dyn Strategy>>` (object-unsafe). Heavy refactor. |

→ **Choose A2**.

### Axis B — ZD strategies (#14a)
- ZD Extortion (Press-Dyson 2012): memory-1 stochastic with `(p1,p2,p3,p4)` derived from `(T,R,P,S)` and a `chi` extortion factor. Coefficients depend on payoffs → strategy is registered against canonical Axelrod `(5,3,1,0)` with doc warning.
- ZD Generous (Stewart-Plotkin 2013): same family, cooperative ZD subset.
- Test: differential `score_X − P ≈ chi·(score_Y − P)` over long noiseless match.

### Axis C — WSLS stochastique (#14b)
- Family `WSLS(p_stay_win, p_switch_loss)`, parameter grid (e.g., 5×5 = 25 variants).

### Axis D — Topologies (#15)
- Add `enum Neighborhood { Moore, VonNeumann, Hex }` + `fn offsets(&self, even_row: bool) -> &[(i32,i32)]`. Refactor `SpatialTournament::step` to iterate the chosen stencil. CLI `--topology`.

### Axis E — Export matrice N×N (#16)
- New method `Tournament::run_round_robin_matrix() -> RoundRobinReport { fitness: Vec<f64>, matrix: Vec<Vec<f64>>, names: Vec<String> }`. CSV writer produces N×N grid with header row/col. CLI `--export-matrix`.

## Impact Map

| File | Change | Risk |
|---|---|---|
| `src/lib.rs` | `Custom` variant; `Neighborhood` enum + threading; `RoundRobinReport` + matrix collection | Medium |
| `src/strategies/zd.rs` (new) | ZD Extortion / Generous family | Low |
| `src/strategies/wsls.rs` (new) | WSLS stochastique family | Low |
| `src/strategies/mod.rs` | Register ZD + WSLS | Low |
| `src/main.rs` | `--topology`, `--export-matrix` | Low |
| `Cargo.toml` | No new deps | None |

## Open Questions (auto-resolved per `-a`)
1. **ZD payoff dependence**: hardcode against canonical `(5,3,1,0)`, runtime warning if differs.
2. **A2 chosen** over A1/A3.
3. **Matrix CSV shape**: square N×N, headers = strategy names, cells = `mean_score_per_turn(i vs j)`.
4. **Hex coords**: axial offset; even-row stencil `[(-1,-1),(-1,0),(0,-1),(0,1),(1,-1),(1,0)]`, odd-row shifted.
