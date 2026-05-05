# FORGE Mission: 02-compute-variance

**Created:** 2026-05-04
**Task:** Logique de calcul / variance — Items #5, #6, #7, #8 du backlog
**Type:** feature (mix bug-fix de fidélité + ajout capacités)
**Mode:** Solo (economy)

## Flags
- Auto: true
- Guard: false (skipped — pas de surface sécurité, projet Rust pur local)
- TDD: false (pas de framework de test en place — couvert dans backlog #17)
- Save: true
- Swarm: false
- Branch: false (commit direct sur main, comme la session précédente)
- PR: false

## User Request
Occupe toi de : Logique de calcul / variance

## Scope (4 items)

- **#5 Variance par paire** — Un seul match par paire avec bruit → variance énorme. Standard Axelrod : N répétitions, moyennées.
- **#6 Auto-jeu** — `for j in i..n` inclut `i==j`. Rendre explicite via flag.
- **#7 Normalisation par tours** — Discount factor → matchs de durée variable, scores incomparables. Normaliser.
- **#8 Validation payoffs Axelrod** — Aucune vérif `T>R>P>S` ni `2R>T+S`. Assertion au démarrage.

## Acceptance Criteria
- [ ] AC-01: `Game::new` (ou `Tournament::new`) panique avec message clair si payoffs ne respectent pas `T>R>P>S` et `2R>T+S`.
- [ ] AC-02: `Tournament` accepte un paramètre `match_repetitions` (>=1). Score d'une paire = moyenne sur N répétitions.
- [ ] AC-03: `Tournament` accepte un flag `include_self_play` (par défaut `true` pour fidélité Axelrod historique).
- [ ] AC-04: Score normalisé exposé (par tour joué) dans les méthodes existantes ou via une nouvelle fonction. Pas de breaking change majeur en CLI.
- [ ] AC-05: `cargo build --release` reste propre (0 warning). Build + smoke run avec --seed 42 reste reproductible.

## Progress
- [x] Phase 0 — Triage
