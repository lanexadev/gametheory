# Phase 5: Export — Final Report

## FORGE Complete — 02-compute-variance

**Task:** Logique de calcul / variance — Items #5, #6, #7, #8 du backlog
**Type:** feature
**Mode:** Solo (economy)
**Status:** Completed

### Phases
- [x] Triage → feature
- [x] Fathom → léger (zone connue depuis 01)
- [x] Orchestrate → 5 ACs, 4 stories, 3 ADRs
- [x] Realize → 4/4 stories
- [ ] Guard → SKIP (pas de surface sécurité, pas de `-x`)
- [x] Export → commit local (pas de `-pr`)

### Files Modified
- `src/lib.rs` — `Game::validate`, `Tournament` enrichi (`match_repetitions`, `include_self_play` + setters), `run_round_robin_per_individual` retourne `Vec<f64>` normalisé/moyenné, `run_round_robin` reprojette × iterations, `run_evolution` trie sur `f64`.
- `src/main.rs` — appel `Game::validate` avec exit propre, flag `--no-self-play`, `--repetitions` rebranché au niveau paire, suppression boucle `for _ in 0..reps` stérile.

### Acceptance Criteria
- [x] AC-01 : `Game::validate` retourne `Err` sur `T<R<P<S` violé ou `2R<=T+S`.
- [x] AC-02 : main exit 1 + message stderr clair quand validation échoue.
- [x] AC-03 : `--repetitions N` joue chaque paire N fois, seeds dérivés différents, score moyenné par tour.
- [x] AC-04 : `--no-self-play` exclut les matchs `i==j`, header round-robin l'affiche.
- [x] AC-05 : `cargo build --release` 0 warning, CSV trié strictement identique entre runs avec même seed.

### Validation
- Build : `cargo build --release` → OK, 0 warning.
- Validation #1 : `--payoff-t 1 --payoff-r 5` → exit 1, "T > R > P > S (got T=1, R=5, P=1, S=0)".
- Validation #2 : `--payoff-t 10 --payoff-r 4` → exit 1, "2R > T + S (got R=4, T=10, S=0; otherwise alternating defection beats mutual cooperation)".
- Reproductibilité : 2 runs `--seed 42 --repetitions 3 --action-noise 0.05`, CSVs triés alphabétiquement strictement identiques.
- Évolution : `--evolution --generations 3 --repetitions 2 --action-noise 0.04 --seed 42` tourne, diversité préservée (multiples Adaptive TFT distincts au top).

### Hors scope (futurs forges)
- Sélection évolutionnaire stochastique (roulette wheel + mutation) — ADR de conception, pas un bug.
- Tests unitaires (#[test]) — backlog #17.
- Tri d'affichage stable pour ex-aequo (`HashMap` non-déterministe entre runs).
- `SpatialTournament` : reproductibilité spatiale et match répétés non couverts (toujours OS rng + 1 match/voisin).
