# 05 - Export Report

## Summary
3 bugs critiques corrigés en mode patch/solo. Build vert, smoke tests réussis.

## Fichiers modifiés
- `src/lib.rs` — Trait `Strategy::next_move` accepte maintenant `&mut dyn RngCore`. Ajout de `Tournament::run_round_robin_per_individual` (Vec<i64> indexé par individu). `run_evolution` utilise désormais cette fitness par-individu au lieu d'agréger par nom.
- `src/strategies/mod.rs` — Closures `FunctionalStrategy` mises à jour. Bug Pattern Matcher (tautologie if/else) remplacé par retour explicite `Action::Defect` avec commentaire justifiant le choix de réponse optimale.
- `src/strategies/{always_cooperate,always_defect,alternator,detective,gradual,grudger,handshake,joss,omega_tft,pavlov,soft_grudger,statistical,suspicious_tit_for_tat,tit_for_tat,tit_for_tat_with_forgiveness,tit_for_two_tats}.rs` — Signature trait alignée. Joss et TitForTatWithForgiveness consomment désormais le RNG passé par `Game::play()` au lieu de `rand::rng()` (thread global).

## Validation
- `cargo build --release` : OK, 0 warning.
- Reproductibilité : `--seed 42` → 81680 (deux runs identiques). `--seed 999` → 81630 (différent → seed actif).
- Évolution : 5 gens × 600 stratégies sans crash, 18+ stratégies distinctes survivent (preuve que la sélection ne dégénère plus en monoculture mécanique).

## Hors scope (à traiter dans futurs forge)
- Sélection évolutionnaire stochastique (roulette wheel, mutation) — c'est de la conception, pas un bug.
- Validation des payoffs Axelrod (`T>R>P>S`, `2R>T+S`).
- Tests unitaires (#[test]) — aucun n'existe à ce jour.
- Reproductibilité spatiale (SpatialTournament utilise OS rng).
