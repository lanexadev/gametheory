# FORGE Mission: 01-fix-fidelity-bugs

**Created:** 2026-05-04
**Task:** Bugs critiques / problèmes de fidélité (issues #1, #2, #3 de l'audit précédent)
**Type:** patch
**Mode:** Solo (economy)
**Flags:** -a -s -e

## Scope
Trois bugs critiques identifiés dans l'audit qui faussent silencieusement les simulations :

1. **Pattern Matcher tautology** — `src/strategies/mod.rs:63` : les deux branches du `if/else` retournent `Action::Defect`.
2. **RNG global non-seedé** — Stratégies stochastiques (`Reactive`, `Bully`, `Forgiving TFT`, `Biased Random`) utilisent `rand::rng()` au lieu du `ChaCha8Rng` seedé du `Game`. Conséquence : `--seed` ne rend rien reproductible.
3. **Évolution biaisée par agrégation de score** — `run_round_robin` somme les scores par *nom de stratégie* ; les stratégies présentes en N exemplaires ont mécaniquement N× plus de score, créant une spirale d'extinction des minorités.

## Progress
- [x] Phase 0: Triage
- [ ] Phase 1: Fathom (light)
- [ ] Phase 3: Realize (direct fixes)
- [ ] Phase 5: Export
