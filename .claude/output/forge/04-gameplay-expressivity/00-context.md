# FORGE Mission: 04-gameplay-expressivity

**Created:** 2026-05-04T19:29:01Z
**Task:** Logique de gameplay / expressivité — étendre le moteur pour traiter #13–#16 de la revue précédente
**Type:** feature
**Mode:** Solo (economy)

## Flags
- Auto: true
- Guard: false
- TDD: false (économie, mais on ajoute des tests ciblés)
- Save: true
- Swarm: false
- Branch: false (rester sur main)
- PR: false

## User Request
`/forge -a -s -e Logique de gameplay / expressivité`

Couvrir les points #13–#16 de la revue antérieure :
- #13 Trait `Strategy` extensible (state typé arbitraire, déverrouille Q-learning, modèles bayésiens)
- #14 Stratégies emblématiques modernes (ZD Extortion, ZD Generous, WSLS stochastique)
- #15 Topologies spatiales configurables (Moore, Von Neumann, hex)
- #16 Export matrice N×N de payoffs

## Acceptance Criteria
- [ ] AC-01: Une stratégie peut stocker un état typé arbitraire (Q-table, croyances) sans modifier `lib.rs`
- [ ] AC-02: ZD Extortion et ZD Generous présentes, paramétrables, et passent un test de régression Press-Dyson contre AllD/AllC
- [ ] AC-03: Famille WSLS stochastique paramétrable (p_stay_after_win, p_switch_after_loss)
- [ ] AC-04: `--topology {moore,vonneumann,hex}` change le voisinage du `SpatialTournament`
- [ ] AC-05: `--export-matrix path.csv` produit une matrice N×N de scores moyens normalisés par tour
- [ ] AC-06: `cargo build --release` passe, `cargo test` passe, aucune régression sur les flags existants

## Progress
- [x] Phase 0: Triage
- [x] Phase 1: Fathom
- [x] Phase 2: Orchestrate
- [x] Phase 3: Realize (6/6 stories, 9/9 tests pass)
- [x] Phase 4: Guard (skipped, pas de -x)
- [x] Phase 5: Export
