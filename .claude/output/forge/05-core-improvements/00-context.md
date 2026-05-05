# FORGE Mission: 05-core-improvements

**Created:** 2026-05-04 21:56:20
**Task:** Address all remaining core/gameplay improvements identified in prior analysis (OneShot)
**Type:** feature (multi-faceted)
**Mode:** Standard
**Flags:** -a (auto), -s (save)

## Scope (carry-over from prior analysis)

1. **Evolution selection** — add mutation + roulette wheel (top-priority fidelity fix)
2. **Spatial seed bug** — `lib.rs:580` always reuses `Game.seed` → identical matches across steps
3. **Swiss tournament normalization** — `run_swiss` uses raw scores, inconsistent with normalized `run_round_robin`
4. **Adaptive TFT O(1)** — use `StrategyScratch::Custom` to avoid per-turn O(N) scan
5. **ZD payoff warning** — log warning when ZD strategies run under non-canonical payoffs

## Progress
