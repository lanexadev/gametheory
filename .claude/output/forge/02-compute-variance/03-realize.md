# Phase 3: Realize — Execution Log

Pas de TDD (#17 reste backlog). Smoke tests inline pour chaque AC.

## Stories

### S-01 — `Game::validate` + appel main
- **Status:** complete
- Modif `src/lib.rs:80` : ajout `Game::validate -> Result<(), String>` (vérifie `T>R>P>S` et `2R>T+S`).
- Modif `src/main.rs:84` : appel après construction du `Game`, exit 1 sur erreur.
- **Smoke** :
  - `--payoff-t 1 --payoff-r 5` → exit 1, message `"T > R > P > S (got T=1, R=5...)"`.
  - `--payoff-t 10 --payoff-r 4` → exit 1, message `"2R > T + S (got R=4, T=10, S=0; otherwise alternating defection beats mutual cooperation)"`.

### S-02 — `Tournament` enrichi (`match_repetitions`, `include_self_play`)
- **Status:** complete
- Champs publics + setters chainables `with_match_repetitions(n)` / `with_include_self_play(b)`.
- Default rétrocompatible : `1, true` (Axelrod historique).
- Loop adapté : `start_j = if include_self_play { i } else { i + 1 }`. Ajoute dimension `rep` pour répéter chaque paire.

### S-03 — `run_round_robin_per_individual -> Vec<f64>` (normalisé + moyenné)
- **Status:** complete
- Score par paire = `score_match_i32 / history.len()` (par tour). Sommé puis divisé par `counts[i]` → score moyen par tour, par individu, sur reps.
- Seeds dérivés : `base + (i*n+j)*reps + rep` → uniques et déterministes.
- `run_round_robin -> HashMap<String, i32>` reprojette × `iterations` pour rester lisible.
- `run_evolution` trie sur `f64` (`partial_cmp().unwrap_or(Equal)`).

### S-04 — CLI : `--repetitions` rebranché + `--no-self-play`
- **Status:** complete
- `--repetitions` rebranché au niveau paire via setter (suppression de la boucle `for _ in 0..reps` qui était stérile sous seed).
- Nouveau flag `--no-self-play`.
- Header round-robin : `"Running Round Robin (match_repetitions=N, self_play=B)..."`.

## Validation finale (AC-05)
- `cargo build --release` : 0 warning.
- `--seed 42` 2× : sortie textuelle apparemment différente (ordre HashMap pour ex-aequo) **mais CSV trié alphabétiquement strictement identique** → reproductibilité bit-pour-bit confirmée.
- `--evolution --generations 3 --repetitions 2 --action-noise 0.04 --seed 42` : tourne, sélection préserve diversité (multiple Adaptive TFT distincts au top).
