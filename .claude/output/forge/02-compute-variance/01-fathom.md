# Phase 1: Fathom — Intelligence Report

## Tech Stack Manifest
- Rust 2024, cargo, deps : `rand 0.9` / `rand_chacha 0.9` / `rayon` / `clap` / `csv`.
- Pas de framework de tests utilisé (`#[test]` absent du repo).
- CLI : `clap derive` dans `src/main.rs`.

## Territory Map
- `src/lib.rs`
  - `Game` (l.57-78) : structure publique, payoffs `(T, R, P, S)`. Pas de validation.
  - `Game::play` (l.80-129) : retourne `(i32, i32, History)`. `history.len()` == nb tours effectivement joués (utile pour normaliser).
  - `Tournament::run_round_robin_per_individual` (l.146-171) : itère `(i, j)` avec `i..n` (inclut self-play). 1 match par paire.
  - `Tournament::run_evolution` (l.232-279) : utilise fitness par individu.
- `src/main.rs`
  - `--repetitions` (l.20-21, l.124-129) : refait `run_round_robin` N fois et somme les HashMaps. Avec seed fixe = N runs identiques → flag stérile pour la variance.
  - `--payoff_t/_r/_p/_s` (l.53-60) sans validation.

## Historical Record
- `d2ba7d0` : RNG seedé + per-individual fitness + Pattern Matcher fix posés. Sémantique de `run_round_robin_per_individual` neuve, libre de legacy.
- `--repetitions` n'a jamais été corrigé pour produire de la variance utile sous seed.

## Path Options

### Option A — Repetitions au niveau **paire** (retenue)
- Champ `match_repetitions: usize` dans `Tournament`.
- Chaque paire `(i, j)` jouée `match_repetitions` fois avec **seeds décalés** : `game.seed + offset(i, j, rep)`.
- Score d'une paire = score moyen par tour (corrige aussi #7 : variance des durées discount).
- CLI `--repetitions` rebranché à ce niveau (ancien comportement supprimé : redondant + biaisé).

### Option B — Couches séparées
Rejetée. Pas de bénéfice à séparer normalisation et répétition.

### Option C — Validation payoffs (#8)
- `Game::validate(&self) -> Result<(), String>` : `T>R>P>S` ET `2R>T+S`.
- Appelée dans `main.rs` au démarrage → `eprintln!` + `exit(1)` si erreur. Pas de panic dans la lib.

### Option D — Self-play optionnel (#6)
- Champ `include_self_play: bool` (default `true` = Axelrod historique).
- `false` → loop `for j in (i+1)..n`.
- Flag CLI `--no-self-play`.

## Impact Map
| Fichier | Modifs | Risque |
|---|---|---|
| `src/lib.rs` | Champs `Tournament` + nouvelles méthodes + `Game::validate` + `run_round_robin_per_individual` retourne `Vec<f64>` (normalisé) | Moyen — change signature interne, mais HashMap public préservé |
| `src/main.rs` | Validation payoffs + rebrancher `--repetitions` + `--no-self-play` | Faible |
| `run_complete_analysis.sh` | Aucun (pas d'usage de `--repetitions`) | Aucun |
| `analysis/*.py` | Aucun (CSV inchangé) | Aucun |

## Décisions auto-appliquées
1. `match_repetitions` default = 1 (rétrocompat).
2. `include_self_play` default = `true`.
3. Score normalisé = `somme(score_match / turns_match) / repetitions`.
4. `run_round_robin -> HashMap<String, i32>` reste en wrapper et reprojette en "score équivalent à `iterations` tours" pour rester lisible vs anciens runs.
5. Validation : `Result` dans la lib, sortie propre dans main.

## Open Questions
Aucune en suspens (auto mode).
