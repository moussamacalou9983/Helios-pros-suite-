# Registre de dette scientifique — HeliosPro

**Statut** : registre vivant, ouvert à la Phase 1.5. Applique le principe
de transparence de `docs/scientific-principles.md` §3 : toute
approximation, troncature, hypothèse par défaut ou domaine de validité
restreint identifié dans le moteur est enregistré ici, avec sa
localisation exacte dans le code, son impact quantifié quand il est
connu, et son statut. **Aucune entrée n'est supprimée** quand une dette
est résolue — elle est marquée « Résolue » avec la référence du commit/PR
qui l'a close, pour préserver l'historique (même principe que les ADR
remplacés, jamais réécrits).

Une entrée est ajoutée ici **au moment où l'approximation est introduite**
(revue scientifique, `docs/governance/scientific-review-process.md`,
critère 6 « analyse des limites »), pas après coup.

## Format d'une entrée

| Champ | Description |
|---|---|
| ID | `DEBT-NNN`, séquentiel, jamais réutilisé |
| Type | Troncature de série / Simplification de modèle / Hypothèse par défaut / Domaine de validité restreint / Limite d'environnement |
| Localisation | Fichier(s) et fonction(s) exacts |
| Impact | Quantifié si mesuré, sinon justifié qualitativement avec la raison de l'absence de mesure |
| Statut | Acceptée et documentée / Ouverte — à traiter / Résolue (réf. commit) |
| Référence | Document qui détaille l'analyse complète |

## Registre

### DEBT-001 — Moteur Meeus réduit : précision inférieure à SPA

- **Type** : Simplification de modèle (délibérée, publiée).
- **Localisation** : `engine/crates/solar_geometry/src/meeus.rs`,
  `engine/crates/solar_position/src/meeus.rs`.
- **Impact** : ~0.01-0.02° d'écart type contre pvlib sur les grandeurs
  géocentriques (mesuré, `latest_global_campaign_report.md`), jusqu'à
  ~0.06° pour l'azimut/l'élévation apparente dans les bandes de
  singularité documentées (DEBT-005, DEBT-006).
- **Statut** : Acceptée et documentée — c'est un moteur secondaire de
  validation croisée par construction (ADR-0001), pas le moteur de
  production par défaut.
- **Référence** : `docs/adr/0001-spa-primary-engine.md`.

### DEBT-002 — Nutation IAU 1980 tronquée à 4 termes dans Meeus

- **Type** : Troncature de série (héritée de la publication source).
- **Localisation** : `engine/crates/solar_geometry/src/meeus.rs`
  (`delta_psi_arcsec`, `delta_epsilon_arcsec` — 4 termes, contre 63 dans
  SPA et 106 dans la série IAU 1980 complète).
- **Impact** : non mesuré isolément (le moteur Meeus dans son ensemble
  est mesuré à ~0.01-0.02°, DEBT-001, sans décomposition par source
  d'erreur individuelle).
- **Statut** : Acceptée et documentée — Meeus (1998) qualifie lui-même
  cette série réduite de « suffisante pour la plupart des usages »
  (Eq. 22.1).
- **Référence** : `docs/references/bibliography.md#ref-meeus1998`.

### DEBT-003 — Nutation IAU 1980 tronquée à 63/106 termes dans SPA

- **Type** : Troncature de série (héritée de la publication source, pas
  re-dérivée indépendamment).
- **Localisation** : `engine/crates/solar_geometry/src/spa/tables.rs`
  (`NUTATION_ABCD`, `NUTATION_YTERM`).
- **Impact** : compris dans la précision annoncée ±0.0003° de Reda &
  Andreas (2004) ; le résidu de troncature lui-même n'a pas été
  recalculé indépendamment dans ce projet (nécessiterait la table IAU
  1980 complète, non extraite).
- **Statut** : Acceptée et documentée — limite explicitement assumée,
  confiance placée dans l'autorité de la publication source et l'accord
  mesuré avec pvlib (qui utilise la même sélection de 63 termes).
- **Référence** : `docs/design/17-analyse-incertitude-numerique.md` §17.4d.

### DEBT-004 — ΔT estimé par défaut, extrapolation hors fenêtre 2005-2050

- **Type** : Domaine de validité restreint, silencieusement extrapolé par
  l'API de confort.
- **Localisation** : `engine/crates/solar_geometry/src/delta_t.rs`
  (`estimate_delta_t_seconds`), consommé par
  `solar_position::spa::solar_position`/`solar_position::meeus::solar_position`
  (les entrées de confort, pas les entrées `*_precise`).
- **Impact** : quantifié — le terme quadratique du polynôme atteint
  55-56 s à l'extrapolation pour 1900/2100, contre un maximum de 14 s
  dans la fenêtre validée 2005-2050 ; à la sensibilité documentée de
  ~15″/s de ΔT, cela correspond à une erreur de position pouvant
  approcher le degré pour un appel de confort sans `delta_t` explicite
  sur une date hors fenêtre.
- **Statut** : Ouverte — à traiter. Le contournement existe
  (`solar_position_precise`/`meeus::solar_position_precise` avec `delta_t`
  explicite, utilisé systématiquement par `validation::global_campaign`),
  mais l'API de confort continue d'extrapoler silencieusement hors de sa
  fenêtre documentée pour tout appelant qui ne fournit pas son propre
  ΔT. Amélioration possible : faire retourner un avertissement ou un
  type de résultat distinct hors fenêtre — décision à trancher par un ADR
  avant modification (changement potentiel de signature d'API publique).
- **Référence** : `docs/design/17-analyse-incertitude-numerique.md` §17.4a.

### DEBT-005 — Divergence de convention de coupure de réfraction près de l'horizon (SPA vs Meeus)

- **Type** : Domaine de validité restreint (propriété des deux modèles
  publiés, pas un bug).
- **Localisation** : `engine/crates/solar_position/src/spa.rs`
  (coupure à `-(0.26667° + atmos_refract)`) vs
  `engine/crates/solar_position/src/meeus.rs::refraction_correction_degrees`
  (coupure fixe à `-1.0°`).
- **Impact** : mesuré — jusqu'à 0.62° d'écart d'élévation/zénith apparent
  entre SPA et Meeus dans la bande étroite (~0.17°) entre les deux
  coupures, sur la campagne mondiale de 2918 cas.
- **Statut** : Acceptée et documentée — chaque moteur reproduit
  fidèlement sa publication source ; harmoniser les coupures serait un
  choix arbitraire non justifié par une publication, contraire au
  principe « aucune formule sans publication ».
- **Référence** : `engine/crates/validation/src/global_campaign/mod.rs`
  (doc de module), `docs/design/18-annexe-technique-position-solaire.md` §18.8.

### DEBT-006 — Singularité de coordonnées de l'azimut au passage au zénith

- **Type** : Limite mathématique inhérente au modèle (pas un bug, pas
  spécifique à une implémentation).
- **Localisation** : formule d'azimut, `engine/crates/solar_position/src/spa.rs`
  et `engine/crates/solar_geometry/src/meeus.rs::topocentric_from_hour_angle`
  (les deux partagent la même structure `atan2`).
- **Impact** : mesuré — jusqu'à 0.19° d'écart d'azimut entre SPA et pvlib
  à moins de 1.2° du zénith exact, sur la campagne mondiale.
- **Statut** : Acceptée et documentée — propriété mathématique de la
  transformation de coordonnées horizontale elle-même (démontrée en
  `docs/design/18-annexe-technique-position-solaire.md` §18.6.1), présente
  dans toute formulation publiée correcte, non « corrigeable » sans
  changer de système de coordonnées entièrement.
- **Référence** : `docs/design/18-annexe-technique-position-solaire.md` §18.6.1, §18.8.

### DEBT-007 — Latitude écliptique du Soleil traitée comme exactement nulle dans Meeus

- **Type** : Simplification de modèle (publiée, quantifiée par la source).
- **Localisation** : `engine/crates/solar_geometry/src/meeus.rs`
  (déclinaison/ascension droite calculées avec β=0).
- **Impact** : Meeus (1998) documente lui-même que la latitude écliptique
  réelle du Soleil ne dépasse jamais 0.00033° — négligeable devant le
  budget ~0.01-0.02° du moteur (DEBT-001).
- **Statut** : Acceptée et documentée — approximation publiée et
  quantifiée par sa propre source, pas une simplification introduite par
  ce projet.
- **Référence** : Meeus, J. (1998), Ch. 25.

### DEBT-008 — Ellipsoïde terrestre : constante propre à SPA distincte de WGS84

- **Type** : Choix de constante délibéré (fidélité à la publication
  source plutôt qu'à la référence géodésique la plus moderne).
- **Localisation** : `engine/crates/solar_position/src/spa.rs`
  (`SPA_EARTH_EQUATORIAL_RADIUS_M = 6 378 140.0`, distinct de
  `core_constants::WGS84_SEMI_MAJOR_AXIS_M = 6 378 137.0`).
- **Impact** : quantifié — écart relatif ~4.7×10⁻⁷, effet sur la
  parallaxe topocentrique ~4×10⁻⁶″, négligeable.
- **Statut** : Acceptée et documentée — choix intentionnel de
  reproductibilité bit-à-bit avec la publication Reda & Andreas (2004).
- **Référence** : `docs/design/17-analyse-incertitude-numerique.md` §17.4b.

### DEBT-009 — Conditions atmosphériques par défaut (atmosphère standard) quand non mesurées

- **Type** : Hypothèse par défaut.
- **Localisation** : `engine/crates/solar_position/src/common.rs`
  (`AtmosphericConditions::default()` = pression ISO 2533:1975 au niveau
  de la mer, température 15 °C).
- **Impact** : non quantifié pour un site à altitude/climat très
  différent de la référence ISO 2533 sans mesure locale fournie ; affecte
  uniquement le terme de réfraction atmosphérique (jamais la géométrie
  géocentrique/topocentrique elle-même).
- **Statut** : Ouverte — à traiter si un futur module a besoin d'une
  pression/température de site systématiquement mesurée plutôt que la
  valeur par défaut ; pour l'instant, tout appelant disposant de mesures
  réelles peut déjà les fournir explicitement via `AtmosphericConditions`.
- **Référence** : `engine/crates/solar_position/src/common.rs`.

### DEBT-010 — Décomposition Erbs et al. (1982) : dispersion empirique connue

- **Type** : Simplification de modèle (corrélation empirique publiée,
  dispersion inhérente reconnue par la littérature du domaine).
- **Localisation** : `engine/crates/irradiance/src/decomposition.rs`.
- **Impact** : non quantifié dans ce projet par comparaison à des mesures
  réelles (validé uniquement par invariants internes à ce jour — voir
  `docs/design/21-matrice-tracabilite.md`, ligne « irradiance » hors
  périmètre de validation externe de la Phase 1) ; la littérature source
  documente une dispersion significative de la corrélation Erbs autour
  de sa courbe moyenne, propriété du modèle statistique lui-même, pas de
  cette implémentation.
- **Statut** : Ouverte — à traiter lors de l'extension de `validation` à
  `pvlib.irradiance` (`docs/design/20-rapport-cloture-phase1.md` §7,
  action 4).
- **Référence** : `docs/references/bibliography.md#ref-erbs1982`.

### DEBT-011 — Masse d'air relative Kasten & Young (1989) : approximation près de l'horizon

- **Type** : Domaine de validité (formule empirique, comportement
  dégradé près de l'horizon comme toute formule de masse d'air).
- **Localisation** : `engine/crates/transposition/src/perez.rs::relative_air_mass`
  (zénith explicitement plafonné à 90° dans le code — `z_deg =
  solar_zenith.degrees().min(90.0)` — pour rester dans le domaine où la
  formule est physiquement significative).
- **Impact** : non quantifié spécifiquement dans ce projet ; la
  publication source documente la formule comme valide sur l'ensemble de
  la plage 0-90° sans la singularité `1/cos(Z)` d'une masse d'air naïve
  près de l'horizon — c'est justement la raison de son choix plutôt
  qu'une formule plus simple.
- **Statut** : Acceptée et documentée — comportement conforme à la
  publication source, plafond à 90° explicite dans le code plutôt
  qu'implicite.
- **Référence** : `docs/references/bibliography.md#ref-kastenyoung1989`.

### DEBT-012 — Calendrier grégorien uniquement, années < 1582 rejetées

- **Type** : Domaine de validité restreint (décision de scope, pas une
  approximation numérique).
- **Localisation** : `engine/crates/core_math/src/julian.rs`
  (`JulianDayError::PreGregorianDate`).
- **Impact** : aucun cas d'usage du projet (durées de vie de projets
  photovoltaïques, archives météo) ne nécessite de dates antérieures à
  1582 ; branche du calendrier julien délibérément non implémentée.
- **Statut** : Acceptée et documentée — hors scope du projet, pas une
  dette à résorber.
- **Référence** : `engine/crates/core_math/src/julian.rs` (commentaire de
  module).

### DEBT-013 — Aucun effet relativiste modélisé

- **Type** : Simplification de modèle (cohérente avec la publication
  source).
- **Localisation** : l'ensemble du moteur `solar_geometry`/`solar_position`.
- **Impact** : amplitude de l'ordre de la milliseconde d'arc pour le
  Soleil (non mesurée dans ce projet, ordre de grandeur cité par la
  littérature d'astrométrie générale) — très en dessous de la précision
  annoncée ±0.0003° = 1.08″ de SPA.
- **Statut** : Acceptée et documentée — Reda & Andreas (2004) lui-même
  n'inclut aucun effet relativiste ; cohérent de ne pas en ajouter
  unilatéralement dans une réimplémentation fidèle.
- **Référence** : `docs/design/18-annexe-technique-position-solaire.md` §18.9.

### DEBT-014 — Irradiance/transposition/thermique validés uniquement par invariants internes

- **Type** : Couverture de validation externe incomplète.
- **Localisation** : `engine/crates/irradiance`, `engine/crates/transposition`,
  `engine/crates/thermal_model`.
- **Impact** : contrairement à la position solaire (campagne de 2918 cas
  contre pvlib), ces modules n'ont pas encore de comparaison contre un
  oracle externe — seulement des tests d'invariants physiques internes
  (`engine/crates/tests`).
- **Statut** : Ouverte — à traiter, action 4 de
  `docs/design/20-rapport-cloture-phase1.md` §7 (extension de
  `validation` à `pvlib.irradiance`/`pvlib.temperature`).
- **Référence** : `docs/design/21-matrice-tracabilite.md`, Partie A,
  ligne EI-005.

## Synthèse

| Statut | Nombre d'entrées |
|---|---|
| Acceptée et documentée | 9 (DEBT-001, 002, 003, 005, 006, 007, 008, 011, 012, 013) |
| Ouverte — à traiter | 4 (DEBT-004, 009, 010, 014) |
| Résolue | 0 |

Les entrées « Ouvertes » ne bloquent pas la clôture de la Phase 1 (aucune
n'affecte l'exactitude déjà démontrée du moteur de position solaire dans
son usage documenté), mais sont des points de suivi explicites pour les
phases suivantes — elles seront reprises dans les rapports de clôture
correspondants (`docs/templates/phase-closure-report-template.md`).
