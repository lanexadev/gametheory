# Phase 2: Orchestrate — Contract

## Overview
- ACs : 5
- Stories : 4
- ADRs : 3

## Acceptance Contracts

| AC-ID | Description | Given / When / Then |
|---|---|---|
| AC-01 | Validation payoffs Axelrod | G: payoffs `(t, r, p, s)` invalides — W: `Game::validate()` appelé — T: retourne `Err(message)` listant la violation. |
| AC-02 | Main rejette payoffs invalides | G: CLI `--payoff_t 1` (T<R) — W: `cargo run` — T: stderr message clair, exit code 1, aucun tournoi exécuté. |
| AC-03 | Repetitions par paire | G: `match_repetitions=5` + bruit > 0 — W: `run_round_robin_per_individual()` — T: chaque paire jouée 5×, score moyenné par tour, seeds dérivés différents. |
| AC-04 | Self-play désactivable | G: `include_self_play=false` — W: `run_round_robin_per_individual()` — T: pas de match `i==j`, scores cohérents. |
| AC-05 | Build vert + reproductibilité conservée | G: code intégré — W: `cargo build --release` puis `--seed 42` 2× — T: 0 warning, mêmes scores. |

## Architecture Decisions

### ADR-01 : `match_repetitions` et `include_self_play` comme champs publics de `Tournament` (pas builder pattern)
- **Contexte** : Rust 2024, code interne, surface API restreinte.
- **Décision** : champs publics + `Tournament::new` les initialise à des defaults rétrocompatibles (`1` et `true`). Setters chainables (`with_match_repetitions`, `with_include_self_play`) pour ergonomie.
- **Conséquences** : pas de breaking change pour appels existants. main.rs branche les flags CLI via setters.

### ADR-02 : Fitness en `f64` (score moyen par tour, moyenné sur reps)
- **Contexte** : avec discount ou bruit, comparer des sommes brutes biaise.
- **Décision** : `run_round_robin_per_individual` retourne `Vec<f64>` (score moyen normalisé). `run_round_robin -> HashMap<String, i32>` reste comme wrapper qui reprojette × `iterations` pour rester lisible.
- **Conséquences** : `run_evolution` trie sur `f64` (Total ord via `partial_cmp().unwrap_or(Equal)`). API publique HashMap inchangée.

### ADR-03 : Validation = `Result` dans la lib, exit propre dans main
- **Contexte** : pas de panic dans une lib réutilisable.
- **Décision** : `Game::validate(&self) -> Result<(), String>`. main appelle, `eprintln!` + `std::process::exit(1)` si erreur.
- **Conséquences** : pas d'erreur silencieuse, message clair pour user.

## Story Backlog

| Story ID | Titre | AC Refs | Fichiers | Complexité |
|---|---|---|---|---|
| S-01 | `Game::validate` + appel main | AC-01, AC-02 | `src/lib.rs`, `src/main.rs` | XS |
| S-02 | Champs `Tournament` (`match_repetitions`, `include_self_play`) + setters + branchement loop | AC-03, AC-04 | `src/lib.rs` | M |
| S-03 | `run_round_robin_per_individual` retourne `Vec<f64>` normalisé + cascade `run_evolution`, `run_round_robin`, `run_swiss`, `run_grand_finale` | AC-03 | `src/lib.rs` | M |
| S-04 | CLI : `--repetitions` rebranché, `--no-self-play`, suppression vieille boucle main.rs | AC-03, AC-04, AC-05 | `src/main.rs` | S |

## See Also
- [Contracts](02b-contracts.md)
- [Stories](02b-stories.md)
- [Architecture](02b-architecture.md)
