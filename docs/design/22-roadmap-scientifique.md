# 22. Roadmap scientifique — futurs moteurs

**Statut** : feuille de route de gouvernance scientifique, adoptée à la
Phase 1.5. Complète (sans le remplacer) `docs/design/15-feuille-de-route.md`
(feuille de route produit, par phase applicative) avec une vue centrée sur
les **moteurs de calcul scientifique** eux-mêmes : leur niveau de
maturité actuel, les références qui les fonderont, l'ordre de
développement recommandé (dépendances entre moteurs), et les critères qui
devront être satisfaits avant qu'ils soient considérés validés — au sens
de `docs/governance/scientific-review-process.md`.

## Vue d'ensemble et ordre de développement recommandé

```
1. PV Module          ─┐
2. Electrical Model     ├─► périmètre électrique de base (Phase 1, engine/)
3. Losses              ─┘
4. Transposition        ── déjà implémenté et validé (Phase 1, voir §Transposition)
5. Tracking             ─┐
6. Bifacial              ├─► Phase 2 (SIG avancé, 3D, optimisation)
7. Shading              ─┘
8. Storage              ─┐
9. Microgrid              ├─► Phase 3 (multi-technologies)
10. Hydrogen             ─┘
11. HVAC                 ── Phase 3+/4 (extension hors périmètre PV initial)
12. Economic Analysis    ── Phase 2 (dépend de la production de tous les moteurs en amont)
```

L'ordre reflète les dépendances physiques réelles : `electrical_model`
dépend des données de `pv_module` ; `bifacial` étend `transposition`
(vue-facteur, albédo) et bénéficie de `tracking` (géométrie mobile) déjà
disponible ; `microgrid` orchestre `storage` (et `pv_module`/
`electrical_model`) ; `economic_analysis` a besoin d'une production
énergétique crédible de tous les moteurs en amont pour produire un LCOE
significatif. Aucun moteur n'est développé hors de cet ordre sans ADR
documentant la dérogation (`docs/governance/adr-policy.md`).

## PV Module

- **Maturité** : squelette documenté (`engine/crates/pv_module`), aucune
  logique. Plan détaillé en préparation, voir
  `docs/design/23-plan-pv-module.md`.
- **Références clés** : IEC 61215-1/2 (qualification), IEC 61730-1/2
  (sécurité), IEC 61853-1..4 (caractérisation énergétique étendue),
  modèles de coefficients Sandia et CEC (California Energy Commission).
- **Ordre de développement** : premier moteur du périmètre électrique —
  aucune dépendance amont dans `engine/` au-delà de `core_units`/
  `core_constants`.
- **Critères de validation** : reproduction exacte des points de mesure
  IEC 61853 publiés pour au moins un module de référence ; cohérence
  dimensionnelle STC/NOCT avec `thermal_model` déjà livré.

## Electrical Model

- **Maturité** : squelette documenté (`engine/crates/electrical_model`),
  aucune logique.
- **Références clés** : De Soto, W., Klein, S.A., Beckman, W.A. (2006),
  « Improvement and validation of a model for photovoltaic array
  performance », *Solar Energy*, 80(1), 78-88 (modèle à diode unique,
  5 paramètres) ; King, D.L. et al., Sandia PV Array Performance Model
  (SAND2004-3535, déjà utilisé par `thermal_model::sandia`) comme
  alternative empirique ; IEC 60904-1 (mesure de courbe I-V).
- **Ordre de développement** : dépend de `pv_module` (fiche technique) et
  de `solar_position`/`transposition`/`thermal_model` (déjà livrés,
  irradiance POA et température de cellule).
- **Critères de validation** : reproduction des courbes I-V publiées
  (IEC 61853) pour plusieurs conditions d'irradiance/température ;
  recherche du point de puissance maximale validée par comparaison à une
  solution de référence (Lambert W ou solveur itératif documenté et
  testé indépendamment).

## Losses

- **Maturité** : squelette documenté (`engine/crates/losses`), aucune
  logique.
- **Références clés** : IEC 60364-5-52 / IEC 62548 (dimensionnement
  câblage, borne de perte Joule), IEC 61724-1 (définition standard des
  pertes de performance : mismatch, salissure, disponibilité).
- **Ordre de développement** : dépend d'`electrical_model` (les pertes
  s'appliquent à une production DC/AC déjà calculée).
- **Critères de validation** : chaque catégorie de perte comparée à une
  fourchette publiée dans la littérature (ex. pertes de mismatch typiques
  1-3 %) avec citation de la source de la fourchette, pas une valeur
  choisie arbitrairement.

## Transposition

- **Maturité** : **implémenté et validé** (Phase 1,
  `engine/crates/transposition`). 4 modèles : isotrope (Liu & Jordan
  1963), Hay-Davies (1980), HDKR (Reindl et al. 1990), Perez (1990).
  97-99 % de couverture par fichier (voir
  `engine/validation-data/latest_coverage_report.txt`).
- **Références clés** : voir `docs/references/bibliography.md` (section
  « Irradiance, décomposition et transposition »).
- **Validation restante** : comparaison externe directe contre
  `pvlib.irradiance` (fonctions équivalentes disponibles dans l'oracle
  déjà utilisé pour la position solaire) — validé aujourd'hui uniquement
  par invariants physiques internes, pas encore par un oracle externe.
  Inscrit comme travail futur en §7 de
  `docs/design/20-rapport-cloture-phase1.md`.

## Tracking

- **Maturité** : non commencé, aucune ligne de code.
- **Références clés** : algorithmes de poursuite mono/bi-axe standards du
  domaine (calcul d'angle de rotation optimal à partir de la position
  solaire déjà disponible) ; algorithme de « backtracking » pour trackers
  en rangées rapprochées (évitement d'ombrage inter-rangées) — référence
  usuelle : la méthode de backtracking décrite par plusieurs fabricants
  de trackers et reprise par pvlib (`pvlib.tracking.singleaxis`), à
  documenter précisément (auteur/norme d'origine) avant implémentation,
  conformément au principe « aucune formule sans publication ».
- **Ordre de développement** : dépend uniquement de `solar_position`
  (déjà livré) — peut être développé indépendamment de `pv_module`/
  `electrical_model`.
- **Critères de validation** : angle de rotation comparé à
  `pvlib.tracking` pour un jeu de cas représentatif ; test d'invariant
  géométrique (l'angle de poursuite optimal doit annuler l'angle
  d'incidence dans le plan de rotation, sauf lorsque le backtracking est
  actif).

## Bifacial

- **Maturité** : non commencé.
- **Références clés** : modèles de vue-facteur (*view factor*) pour
  irradiance arrière (ex. modèle « infinite sheds » largement documenté
  dans la littérature PV bifaciale, également implémenté dans
  `pvlib.bifacial`) ; norme IEC TS 60904-1-2 (mesure de dispositifs
  bifaciaux).
- **Ordre de développement** : étend `transposition` (nécessite
  irradiance avant et arrière) ; bénéficie de `tracking` si les deux sont
  développés en parallèle (les configurations bifaciales sont souvent
  couplées à des trackers en pratique).
- **Critères de validation** : comparaison à `pvlib.bifacial` sur un jeu
  de configurations (hauteur de structure, albédo, espacement de
  rangées) représentatif.

## Shading

- **Maturité** : non commencé. Dépend du « Service SIG » applicatif
  (modélisation 3D, obstacles, LiDAR — `docs/design/15-feuille-de-route.md`
  Phase 2), hors périmètre du seul moteur de calcul scientifique.
- **Références clés** : ray-casting géométrique standard (aucune
  publication spécifique au domaine PV requise pour la géométrie pure ;
  la modélisation de l'effet électrique de l'ombrage partiel, elle,
  s'appuie sur le modèle à diode unique d'`electrical_model` avec diodes
  de dérivation — référence à documenter au moment de l'implémentation).
- **Ordre de développement** : dépend de `solar_position` (position du
  soleil pour projeter les ombres) et d'`electrical_model` (effet
  électrique de l'ombrage partiel sur une chaîne de modules).
- **Critères de validation** : à définir précisément lors de l'ADR
  d'implémentation ; au minimum, comparaison de l'aire ombragée calculée
  contre une scène 3D de référence à géométrie connue analytiquement.

## Storage

- **Maturité** : non commencé.
- **Références clés** : modèle KiBaM (Kinetic Battery Model — Manwell,
  J.F. & McGowan, J.G. (1993), « Lead acid battery storage model for
  hybrid energy systems », *Solar Energy*, 50(5), 399-405) pour la
  dynamique de charge/décharge ; modèles de vieillissement (cyclage,
  calendaire) à documenter précisément selon la chimie ciblée
  (Li-ion NMC/LFP en priorité, cohérent avec le marché résidentiel/C&I
  visé par le périmètre V1 du cahier des charges).
- **Ordre de développement** : dépend d'`electrical_model` (production
  DC/AC disponible à stocker/déstocker).
- **Critères de validation** : reproduction de courbes de décharge
  publiées pour au moins une chimie de référence ; test d'invariant
  énergétique (conservation de l'énergie moins les pertes documentées).

## Microgrid

- **Maturité** : non commencé.
- **Références clés** : stratégies de dispatch rule-based et MILP
  (Mixed-Integer Linear Programming) pour l'optimisation de dispatch,
  métrique LPSP (*Loss of Power Supply Probability*) comme critère de
  dimensionnement standard du domaine — références précises à
  sélectionner lors de l'ADR d'implémentation.
- **Ordre de développement** : orchestre `pv_module`/`electrical_model`/
  `storage` (et un modèle de groupe électrogène/réseau à définir) —
  dernier des moteurs de production/stockage dans la chaîne de
  dépendances.
- **Critères de validation** : conservation énergétique sur un scénario
  de dispatch complet ; comparaison de la métrique LPSP à un outil de
  référence ou à un jeu de données publié.

## Hydrogen

- **Maturité** : non commencé. **Absent de la feuille de route produit
  actuelle** (`docs/design/15-feuille-de-route.md` couvre le pompage
  solaire en Phase 3 mais ne mentionne pas explicitement l'hydrogène) —
  ajouté ici à la demande explicite de gouvernance scientifique ; son
  rattachement à une phase précise de la feuille de route produit reste
  à trancher par un ADR dédié avant le démarrage de son développement.
- **Références clés** : modèles d'électrolyseur PEM/alcalin (courbe de
  polarisation, rendement en fonction de la charge) — littérature
  spécifique à identifier et citer précisément lors de l'ADR
  d'implémentation, conformément au principe « aucune formule sans
  publication ».
- **Ordre de développement** : dépend d'`electrical_model` (puissance DC/AC
  disponible pour l'électrolyse) ; conceptuellement proche de `storage`
  (l'hydrogène est une forme de stockage d'énergie) sans en dépendre
  strictement.
- **Critères de validation** : reproduction d'une courbe de polarisation
  publiée pour au moins une technologie d'électrolyseur de référence.

## HVAC

- **Maturité** : non commencé. Hors du périmètre PV initial du cahier
  des charges (`docs/design/01-cahier-des-charges.md` §1.7) — extension
  solaire thermique déjà anticipée comme réserve normative (ASHRAE 93,
  voir `docs/references/bibliography.md#ref-ashrae93`) mais non planifiée
  dans une phase précise de `15-feuille-de-route.md` à ce jour.
- **Références clés** : ASHRAE Standard 93 (méthodes d'essai de capteurs
  solaires thermiques), modèle de modificateur d'angle d'incidence
  ASHRAE/Martin-Ruiz déjà identifié dans les documents de conception
  (`docs/design/06-algorithmes.md` §3).
- **Ordre de développement** : réutilise `solar_position`/`transposition`
  (géométrie solaire commune au PV et au thermique) ; indépendant des
  moteurs électriques PV.
- **Critères de validation** : conformité aux méthodes d'essai ASHRAE 93
  sur un jeu de capteurs solaires thermiques de référence.

## Economic Analysis

- **Maturité** : non commencé.
- **Références clés** : méthodes standards LCOE (*Levelized Cost of
  Energy*), VAN/TRI (NPV/IRR) — formules financières standards,
  référence de méthode à documenter précisément (ex. IEA/NREL
  *Cost of Energy* methodology) plutôt que réinventées.
- **Ordre de développement** : **dernier** de la liste — a besoin d'une
  production énergétique crédible de tous les moteurs en amont
  (`pv_module`, `electrical_model`, `losses`, et selon le scénario
  `storage`/`microgrid`) pour produire un LCOE significatif ; peut
  néanmoins être développé en parallèle sur des données de production
  déjà disponibles (`simulation` actuel) pour un LCOE simplifié
  hors rendement électrique complet.
- **Critères de validation** : reproduction d'un calcul LCOE de référence
  publié (ex. étude NREL ATB) sur un cas d'école aux paramètres connus.

## Application de la procédure de revue

Chaque moteur listé ci-dessus, au moment de son développement, suit
intégralement `docs/governance/scientific-review-process.md` (six
critères) et est précédé d'un ADR (`docs/governance/adr-policy.md`)
documentant le choix de modèle et les références retenues. Cette roadmap
fixe l'ordre et les références **prévisionnelles** ; l'ADR d'implémentation
de chaque moteur est l'autorité finale sur le modèle réellement retenu,
qui peut différer de cette roadmap si la revue documentaire (menée au
moment de l'implémentation) révèle une meilleure source.
