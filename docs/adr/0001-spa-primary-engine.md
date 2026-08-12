# ADR-0001 — SPA (Reda & Andreas 2004) comme moteur de position solaire primaire

**Statut** : Accepté
**Date** : rétroactif, décision prise en session 3 de la Phase 1 (voir
`docs/design/16-phase1-validation-report.md` §17)
**Catégorie** : Nouvel algorithme

## Contexte

Le moteur de position solaire doit atteindre une précision de référence,
comparable aux outils professionnels du secteur (PVsyst, PV*SOL,
Archelios Calc). Deux algorithmes publiés étaient candidats : la série
réduite de Meeus (1998, *Astronomical Algorithms*) — plus simple, plus
rapide, précision ~0.01-0.02° — et l'algorithme SPA complet du NREL
(Reda & Andreas 2004) — précision annoncée ±0.0003°, mais nécessitant
l'implémentation intégrale de tables de coefficients volumineuses
(séries héliocentriques VSOP87 tronquées, nutation IAU 1980) et un accès
fiable aux valeurs exactes de ces tables.

## Options considérées

### Option A — Meeus (1998) réduit comme moteur unique

- Avantages : implémentation compacte, rapide (~0.5 µs/appel), déjà
  livrée et testée en session 1-2.
- Inconvénients : précision ~0.01-0.02°, insuffisante pour revendiquer
  une exactitude de référence ; pas l'algorithme explicitement demandé
  par le cahier des charges du projet.

### Option B — SPA complet comme moteur unique

- Avantages : précision de référence ±0.0003°, algorithme le plus cité
  du domaine, table de coefficients extraite et vérifiée par recoupement
  indépendant (pvlib-python).
- Inconvénients : ~8-9× plus lent que Meeus (encore largement dans le
  budget de performance), perte de la validation croisée par une seconde
  implémentation indépendante si Meeus est supprimé.

### Option C — SPA comme moteur par défaut, Meeus conservé comme moteur secondaire sélectionnable

- Avantages : cumule la précision de référence de SPA avec la valeur de
  Meeus comme témoin indépendant (deux implémentations de sources
  différentes qui s'accordent est une preuve plus forte qu'une
  implémentation unique testée contre elle-même) ; permet une abstraction
  réutilisable (`SolarPositionAlgorithm`) pour de futurs moteurs
  multi-implémentations.
- Inconvénients : complexité additionnelle de l'abstraction ; nécessite
  de maintenir deux implémentations.

## Décision

**Option C.** Le critère décisif : la validation indépendante
(`docs/scientific-principles.md` §4) est plus forte avec deux
implémentations de sources distinctes qui convergent qu'avec une seule
implémentation, aussi précise soit-elle sur le papier. SPA devient le
moteur par défaut (précision de référence), Meeus est conservé comme
moteur secondaire explicitement sélectionnable via le trait
`SolarPositionAlgorithm` (`engine/crates/solar_position/src/algorithm.rs`),
jamais silencieusement supprimé.

## Conséquences

- Le trait `SolarPositionAlgorithm` devient le patron de référence pour
  toute future abstraction multi-moteurs du projet (batteries,
  stratégies de dispatch, etc.) — voir `engine/README.md` §« Extending
  the engine ».
- Deux implémentations à maintenir en parallèle ; tout changement de
  comportement de l'une doit être vérifié contre l'accord documenté avec
  l'autre (`solar_position::algorithm::tests::spa_and_meeus_agree_within_their_documented_accuracy_budget`).
- Dette scientifique introduite : la nutation SPA est tronquée à 63 des
  106 termes IAU 1980 (hérité de la sélection propre à Reda & Andreas
  2004) — voir `docs/scientific-debt-register.md`, entrée DEBT-003.
- La table de coefficients SPA n'a pas pu être vérifiée chiffre par
  chiffre contre le PDF NREL original (accès réseau indisponible dans
  l'environnement de développement) ; elle a été extraite
  programmatiquement de pvlib-python et vérifiée par accord byte-exact
  sur le cas canonique publié — voir `engine/validation-data/README.md`.

## Références

- Reda, I. & Andreas, A. (2004). « Solar Position Algorithm for Solar
  Radiation Applications ». *Solar Energy*, 76(5), 577-589
  (NREL/TP-560-34302).
- Meeus, J. (1998). *Astronomical Algorithms*, 2e éd., Willmann-Bell.
- `docs/design/16-phase1-validation-report.md` §17 (récit complet de la
  décision et de sa mise en œuvre).
- `docs/design/18-annexe-technique-position-solaire.md` §18.4-18.5
  (équations complètes des deux moteurs).
