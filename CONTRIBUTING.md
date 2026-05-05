# Contributing to GameTheory

Thanks for your interest in contributing. This document covers the
**branch model**, **commit conventions**, **testing requirements**, and
**code style** used in this repository. Read it once; everything below
is a hard convention rather than a suggestion.

---

## Branch model — Git Flow (lite)

We use a two-permanent-branch flow with short-lived topic branches
merging into them. The two long-lived branches are:

| Branch | Role | Stability | Direct push? |
|---|---|---|---|
| `main` | Release / stable | Tagged releases only | **No** |
| `develop` | Integration / unstable | Tip of active development | **No** |

All work happens on **short-lived topic branches** that merge into
`develop` via PR. `main` only ever advances by merging `develop` (or a
`hotfix/*` branch) at release time.

```
                 v0.7.0     v0.8.0
main  ───────────●──────────●─────────────────
                  ╲          ╲
                   ╲          ╲
develop ─●───●───●──●───●───●──●───●───●──────
          ╲   ╲       ╲       ╲   ╲
           ●───●       ●───────●   ●  ← feature/refactor/fix/...
```

### Topic branch naming

| Prefix | When to use | Merges into |
|---|---|---|
| `feature/<short-name>` | New capability, new strategy, new tournament mode. | `develop` |
| `fix/<short-name>` | Bug fix on `develop`. | `develop` |
| `refactor/<short-name>` | Internal restructuring with no behaviour change. | `develop` |
| `perf/<short-name>` | Performance work with no behaviour change. | `develop` |
| `docs/<short-name>` | Documentation only. | `develop` |
| `test/<short-name>` | Adding tests against existing behaviour. | `develop` |
| `hotfix/<short-name>` | **Critical fix shipped directly off `main`**. | `main` AND `develop` |
| `release/<x.y.z>` | Release-candidate stabilisation. | `main` AND `develop` |

`<short-name>` is kebab-case, max ~4 words: `feature/q-learning`,
`fix/pattern-matcher-cycle`, `perf/spatial-grid-flat`,
`hotfix/zd-payoff-warning`.

### Typical workflow

```bash
# Start from a clean develop
git checkout develop
git pull --ff-only

# Branch off
git checkout -b feature/my-thing

# Work, commit (see commit conventions below)
# ...

# Sync against develop before opening a PR
git fetch origin
git rebase origin/develop

# Push and open PR
git push -u origin feature/my-thing
gh pr create --base develop
```

### Releasing

1. Cut a `release/<x.y.z>` branch off `develop`.
2. Bump `Cargo.toml` version. Update `CHANGELOG.md`. Run the full test
   suite.
3. Open a PR `release/<x.y.z>` → `main`.
4. After merge, tag `main` as `vX.Y.Z` (`git tag -a v0.7.0 -m "0.7.0"`)
   and push the tag.
5. Merge `main` back into `develop` to fast-forward the version bump.

### Hotfixes

1. Branch off `main` as `hotfix/<bug>`.
2. Fix + test.
3. PR into `main`. After merge, tag a patch release.
4. Merge `main` back into `develop` so the fix lands in the integration
   line.

---

## Commit messages

The project uses **themed-line commits**: a short imperative title that
names a *theme*, followed by a body of bullets.

### Format

```
<Theme>: <short imperative summary>

<body — what / why, written for someone reading git log without context>

- Bullet 1: concrete change.
- Bullet 2: concrete change.
- Bullet 3: concrete change.
```

### Examples (real history)

```
Performance: O(1) stateful strategies, parallel spatial grid, lazy history allocation
```

```
Compute Fidelity: Axelrod payoff validation, per-pair repetitions, score normalization, optional self-play
```

```
Learning Strategies: Q-Learning, Bayesian classifier, and Lookahead minimax

Three orthogonal model-based archetypes built on StrategyScratch::Custom:

- Q-Learning: tabular model-free RL with epsilon-greedy action
  selection over Q(state, action), where state = last-K joint moves
  packed as a u32 bit-field. Per-turn TD update; converges in-match.
- Bayesian: log-space posterior over an archetype basis ...
- Lookahead: depth-limited minimax against any Box<dyn Strategy> ...
```

### Rules

- **Theme is a noun**, not a verb. Reads like a section header in a
  changelog: `Performance:`, `Compute Fidelity:`, `Learning Strategies:`.
- **Title under 70 chars**.
- **Body explains *why***, not what — `git diff` already shows what.
- **Bullets** in the body for multi-change commits. One change per
  bullet.
- **Imperative mood**: "add", "fix", "rewrite" — not "added", "fixes".
- **No `Co-Authored-By` trailers** for AI assistants. Only credit
  human authors.
- **No issue numbers in titles**. Reference them in the body if needed.

### When to split commits

- One commit per logical concern. A "themed" commit can group several
  small changes around the same theme — that's the point of the format —
  but a feature + an unrelated bug fix is two commits.
- Refactors that don't change behaviour go in their own commit, ahead
  of the behavioural change that motivates them.

---

## Code style

Rust 2024 edition. `cargo fmt` is the formatter; run it before
committing. `cargo clippy --all-targets -- -D warnings` should pass
clean.

### House conventions

- **Comments explain *why***, not what. Identifiers explain what. Don't
  comment a line that paraphrases what it does.
- **Avoid `unwrap()`** outside of tests. Prefer `expect("…")` with a
  message that documents the invariant being relied on.
- **Doc comments** on public items in `lib.rs` and on every strategy
  module. The module-level doc comment should cite the source paper or
  give the closed-form for parameterised strategies.
- **Names**: strategy structs are TitleCase nouns matching the
  literature. Display names (`Strategy::name()`) are human-readable
  with parameters in parentheses: `Q-Learning (a=0.30, g=0.95, e=0.10, K=2)`.
- **No `rand::rng()`** (the global thread RNG). Always thread the
  `&mut dyn RngCore` parameter from the engine — that's how `--seed`
  reaches every strategy.

### Adding a new strategy

1. Create `src/strategies/my_strategy.rs`.
2. Implement `Strategy`. If the strategy is stateful, override
   `init_scratch` and `next_move_stateful`; use
   `StrategyScratch::Custom(Box::new(MyState::default()))` if your
   state shape isn't already in the enum.
3. `pub mod my_strategy;` in `src/strategies/mod.rs`.
4. Register variants inside `get_generative_strategies()`.
5. Add an integration test under `tests/`. See `tests/learning.rs`,
   `tests/zd.rs`, or `tests/extensible_state.rs` for patterns.

---

## Testing

`cargo test` must pass before opening a PR.

### Required for a feature PR

- At least one **integration test** under `tests/` exercising the new
  behaviour end-to-end through `Game::play` or `Tournament`.
- **Determinism check** under a fixed seed for any strategy or
  tournament code path that uses RNG.
- For numerical / dynamic-system claims (e.g. "agent X converges to
  defection vs AlwaysDefect"), the test asserts the *behaviour*, not
  the exact score, with a margin that survives RNG jitter.

### Recommended

- `cargo clippy --all-targets -- -D warnings`.
- A short manual run of the new code path in `cargo run --release`,
  with a screenshot or paste of the relevant CSV/console output in the
  PR description.

---

## Pull requests

A PR description should include:

- **What changes**, in 2–3 sentences.
- **Why** — the underlying problem, not the patch.
- **Test plan** — the commands you ran and what you observed.
- **Linked issues** if any.

Keep PRs **focused**: one theme per PR. If a refactor is needed before
the feature, ship it as a separate, earlier PR.

---

## Governance

This is currently a single-maintainer project. Reviews are best-effort.
Open a Discussion or Issue first if you're planning a large change —
saves rework if the design needs adjustment.

---

Questions? Open an issue. Thanks for contributing.
