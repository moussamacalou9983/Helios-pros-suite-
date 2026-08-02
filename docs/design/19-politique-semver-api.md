# 19. Vérification de l'API publique et politique SemVer

**Statut** : audit effectué, politique adoptée, **API gelée** au tag
`v1.0.0-scientific-core` (voir `docs/releases/v1.0.0-scientific-core.md`).
Répond à l'exigence de sortie de phase : « Vérifier que l'API publique est
stable. Documenter chaque fonction. Respecter SemVer. », ainsi qu'à
l'exigence de clôture de phase : « Considérer l'API publique du moteur
solaire comme stable. Identifier clairement : API publiques, API internes,
API expérimentales. » (§19.6).

**Portée du gel** : les crates `core_constants`, `core_units`, `core_math`,
`solar_geometry`, `solar_position` et `validation`. Les crates
`pv_module`, `electrical_model`, `losses` restent des squelettes
documentés sans API à geler (aucun item public au-delà de la documentation
de module elle-même — voir `engine/README.md`) ; leur API sera définie et
gelée séparément lors de leur implémentation.

## 19.1 Couverture de documentation — vérifié, pas supposé

Chaque crate active le lint workspace `missing_docs = "warn"` (voir
`engine/Cargo.toml`, `[workspace.lints.rust]`, hérité via `[lints]
workspace = true` dans chaque `Cargo.toml` de crate). Pour vérifier que
cette règle ne laisse passer aucun élément public non documenté (un
`warn` peut passer inaperçu dans un flux de build volumineux), la
commande suivante promeut le lint en erreur bloquante pour un audit
ponctuel :

```bash
RUSTFLAGS="-D missing_docs" cargo doc --workspace --no-deps
```

**Résultat de cet audit (2026-08-02)** : `Finished` sans aucun
avertissement ni erreur, sur les 14 crates du workspace. Une recherche
complémentaire (`grep -rn "allow(missing_docs)"`) confirme qu'aucune
dérogation locale à cette règle n'existe nulle part dans le code source —
la couverture à 100 % de la documentation de l'API publique n'est donc pas
une estimation, c'est une propriété vérifiée mécaniquement à chaque build.

## 19.2 Surface d'API publique — crates du périmètre « Position Solaire »

Liste extraite directement du code source (`grep -rn "^pub "`, pas une
transcription manuelle) des éléments de premier niveau exposés par les
crates concernés par la validation de cette phase. Les méthodes associées
(`impl` blocks, en particulier les nombreux constructeurs/accesseurs de
`core_units`) ne sont pas ré-énumérées ici individuellement — la
documentation `cargo doc` générée reste la référence exhaustive et à jour ;
cette section fixe la forme des **types et fonctions de premier niveau**,
qui est la partie la plus coûteuse à faire évoluer de façon compatible.

### `core_constants`

13 constantes physiques/astronomiques publiques (`SOLAR_CONSTANT_W_M2`,
`STANDARD_GRAVITY_M_S2`, `STANDARD_ATMOSPHERE_PA`,
`STEFAN_BOLTZMANN_W_M2_K4`, `CELSIUS_ZERO_KELVIN`, `SECONDS_PER_DAY`,
`DAYS_PER_JULIAN_CENTURY`, `J2000_EPOCH_JD`, `WGS84_FLATTENING`,
`WGS84_SEMI_MAJOR_AXIS_M`, `ASTRONOMICAL_UNIT_M`,
`SOLAR_PARALLAX_ARCSEC_AT_1AU`, `SOLAR_MEAN_ANGULAR_RADIUS_DEG`).

### `core_units`

Types forts (newtypes autour de `f64`, chacun avec constructeurs/
accesseurs par unité) : `Angle`, `Irradiance`, `Length`, `Pressure`,
`Ratio`, `Temperature`, `Velocity`.

### `core_math`

- `julian::CalendarDateTime` (struct), `julian::JulianDayError` (enum)
- `interpolation::linear_interpolate`
- `stats::{mean, standard_deviation, mean_bias_error, mean_absolute_error, root_mean_square_error, max_absolute_error, relative_error}`

### `solar_geometry`

- Modules publics : `delta_t`, `incidence`, `meeus`, `spa`, `time`
- `delta_t::estimate_delta_t_seconds`
- `incidence::{SurfaceOrientation, angle_of_incidence}`
- `meeus::{Ephemeris, compute}`
- `spa::{SpaGeocentricPosition, compute}`
- `time::{day_of_year, equation_of_time_minutes, apparent_solar_time_hours, solar_noon_utc_hour}`

### `solar_position`

- `SolarPositionAlgorithm` (trait), `Spa`, `MeeusReduced` (implémentations)
- `Observer`, `AtmosphericConditions`, `SolarPosition` (structs communes)
- `solar_position` (fonction racine du crate, engine par défaut)
- `spa::{DEFAULT_ATMOSPHERIC_REFRACTION_AT_HORIZON_DEG, SpaDetailedResult, solar_position, solar_position_detailed, solar_position_precise}`
- `meeus::{solar_position, solar_position_precise}`

### `validation`

- Modules publics : `dataset`, `global_campaign`, `report`
- `dataset::{PvlibReferenceCase, DATASET}`
- `global_campaign::{dataset (sous-module), GlobalCase, DATASET, CategoryQuantityStats, GlobalCampaignReport, evaluate_spa, evaluate_meeus}`
- `report::{QuantityErrorStats, EngineErrorReport, evaluate}`

## 19.3 Politique SemVer adoptée

Tous les crates sont actuellement en version `0.1.0` (`engine/Cargo.toml`,
`[workspace.package] version = "0.1.0"`, héritée par chaque crate via
`version.workspace = true`). SemVer (semver.org, spécification 2.0.0)
laisse explicitement la compatibilité indéfinie pour la branche `0.y.z` :
*« Anything MAY change at any time » (§4)*. Ce projet adopte néanmoins,
volontairement et dès la version `0.1.0`, une discipline plus stricte que
ce minimum, cohérente avec l'exigence du projet qu'aucune API ne soit
« instable par défaut » :

1. **Un incrément de version PATCH (0.1.x → 0.1.(x+1))** est réservé aux
   corrections qui ne changent la signature d'aucun élément public
   existant : corrections de bug internes, ajout de tests, ajout de
   documentation, corrections de performance sans changement d'API.
2. **Un incrément de version MINOR (0.x.0 → 0.(x+1).0)** est utilisé pour
   tout ajout **additif** et non cassant : nouvelle fonction publique,
   nouveau champ dans un struct déjà `#[non_exhaustive]` (aucun struct du
   projet n'est actuellement marqué ainsi — voir §19.4), nouvelle variante
   d'enum, nouveau crate dans le workspace.
3. **Tout changement cassant** (signature de fonction modifiée, champ
   public supprimé/renommé, variante d'enum supprimée, type de retour
   changé, comportement numérique changé de façon à invalider un test de
   régression existant du type `docs/design/14-strategie-tests.md`) est
   traité comme justifiant un incrément MINOR au minimum tant que le
   projet reste en `0.y.z` (conformément à la pratique répandue de
   l'écosystème Rust pour les crates `0.x`, où MINOR joue le rôle de
   MAJOR), et sera documenté explicitement dans le message de commit et,
   le cas échéant, dans une note de migration dans
   `docs/design/15-feuille-de-route.md`.
4. **Le passage à `1.0.0`** ne sera proposé qu'une fois l'ensemble des 14
   crates du workspace au statut « implémenté » (`pv_module`,
   `electrical_model`, `losses` sont encore des squelettes documentés,
   voir `engine/README.md`), et après qu'au moins une campagne de
   validation externe complète (au niveau de rigueur de
   `validation::global_campaign`) ait couvert chacun d'eux — pas
   seulement le sous-système de position solaire couvert par cette phase.

## 19.4 Non-exhaustivité des types publics

Aucun `struct`/`enum` public de ce workspace n'est actuellement marqué
`#[non_exhaustive]`. Cela signifie que l'ajout d'un champ à un struct
public existant (ex. `SolarPosition`, `Observer`) ou d'une variante à un
enum public existant (ex. `JulianDayError`) est, au sens strict de SemVer,
un changement cassant pour tout code appelant qui construit ces types par
syntaxe littérale ou qui les « match » exhaustivement — et sera donc traité
selon la règle §19.3.3 ci-dessus (incrément MINOR minimum), pas silencieusement
glissé dans un PATCH. Marquer ces types `#[non_exhaustive]` dès
maintenant a été considéré et délibérément **rejeté** pour cette phase :
cela ajouterait une indirection (obligation d'utiliser des méthodes
constructrices au lieu de la syntaxe littérale `Struct { champ: valeur }`)
sans bénéfice actuel, ce moteur n'ayant pour l'instant aucun consommateur
externe au workspace — cette décision sera réexaminée avant `1.0.0`
(§19.3.4).

## 19.5 Stabilité constatée durant cette phase

Aucune fonction publique des crates `core_constants`, `core_units`,
`core_math` n'a changé de signature depuis leur livraison initiale
(§16.1 de `16-phase1-validation-report.md`). Dans `solar_geometry` et
`solar_position`, les seuls ajouts effectués durant cette phase sont
strictement additifs au sens de §19.3.2 :

- `solar_position::meeus::solar_position_precise` (nouvelle fonction,
  ajoutée pour la campagne de validation mondiale, §21 du suivi de
  session) — n'a modifié la signature d'aucune fonction existante ;
  `solar_position::meeus::solar_position` continue de fonctionner à
  l'identique (elle délègue désormais à la nouvelle fonction en interne,
  changement d'implémentation invisible de l'extérieur).
- `core_math::stats::standard_deviation` (nouvelle fonction, §24 du suivi
  de session).

Le seul changement de comportement interne sans changement de signature
publique est la correction documentée en
`17-analyse-incertitude-numerique.md` §17.4c
(`solar_position::spa::solar_position_precise` utilise désormais
`core_constants::SOLAR_PARALLAX_ARCSEC_AT_1AU` au lieu du littéral
`8.794` identique) : valeur numérique inchangée, confirmée par la suite de
tests complète (164 tests au moment de cet audit, `cargo test --workspace`)
avant et après — ce n'est donc, par
définition, ni un changement cassant ni même un changement de
comportement observable, seulement un changement de traçabilité interne.

**Conclusion** : l'API publique du sous-système de position solaire est
stable au sens où ce document la définit ; aucun changement cassant n'a
été introduit pendant cette phase de validation renforcée.

## 19.6 Gel de l'API — séparation Publique / Interne / Expérimentale

Tous les items listés en §19.2 sont `pub` au sens de la visibilité Rust
(sinon ils n'apparaîtraient pas dans cette liste extraite par `grep`),
mais « visible depuis l'extérieur du crate » et « destiné à être le point
d'entrée stable pour du code applicatif externe au workspace » sont deux
questions différentes. Cette section répond à la seconde, ce que la
visibilité Rust seule ne peut pas exprimer.

### API publique — le contrat stable, point d'entrée recommandé

Ce sont les items que du code applicatif (futur module `pv_module`, futur
GUI/API/rapports, ou tout consommateur externe au workspace) doit utiliser
directement. Couverts par la politique SemVer §19.3 dans toute sa rigueur :
tout changement cassant ici requiert au minimum un incrément MINOR
documenté.

| Crate | Items |
|---|---|
| `core_constants` | Les 13 constantes (§19.2) |
| `core_units` | `Angle`, `Irradiance`, `Length`, `Pressure`, `Ratio`, `Temperature`, `Velocity` et leurs méthodes associées |
| `core_math` | `julian::{CalendarDateTime, JulianDayError}`, `stats::*` (7 fonctions), `interpolation::linear_interpolate` |
| `solar_geometry` | `incidence::{SurfaceOrientation, angle_of_incidence}`, `time::{day_of_year, equation_of_time_minutes, apparent_solar_time_hours, solar_noon_utc_hour}` |
| `solar_position` | `solar_position` (fonction racine), `SolarPositionAlgorithm` (trait), `Spa`, `MeeusReduced`, `Observer`, `AtmosphericConditions`, `SolarPosition` |

C'est la surface qu'un futur `pv_module`/`electrical_model` doit consommer :
`solar_position::solar_position(...)` (ou `Spa`/`MeeusReduced` explicite via
le trait) pour la position solaire, `solar_geometry::incidence::angle_of_incidence`
pour l'angle d'incidence sur un plan orienté — jamais les modules
géocentriques bas niveau ci-dessous directement.

### API interne — visibilité `pub` requise pour le câblage inter-crates, non destinée à un usage applicatif direct

Ces items doivent rester `pub` (ils sont consommés depuis un autre crate du
workspace, ce que la visibilité `pub(crate)` ne permettrait pas), mais ne
sont pas le point d'entrée recommandé : ils exposent des étapes
intermédiaires spécifiques à une implémentation donnée (géocentrique avant
correction topocentrique, `delta_t` non estimé, etc.), destinées à être
consommées par `solar_position` (pour SPA/Meeus) et par `validation` (pour
reproduire exactement les cas de référence externes avec un `delta_t`
connu). Un changement ici est **toujours** traité comme un changement
d'implémentation interne au sens de §19.3.1 (PATCH), sauf s'il modifie
une garantie déjà documentée dans l'annexe technique
(`18-annexe-technique-position-solaire.md`), auquel cas §19.3.3 s'applique.

| Crate | Items | Consommé par |
|---|---|---|
| `solar_geometry` | `delta_t::estimate_delta_t_seconds` | `solar_position::{spa, meeus}` |
| `solar_geometry` | `meeus::{Ephemeris, compute}` | `solar_position::meeus` |
| `solar_geometry` | `spa::{SpaGeocentricPosition, compute}` | `solar_position::spa`, `validation` (tests géocentriques) |
| `solar_position` | `meeus::solar_position_precise` | `validation::global_campaign` (delta_t explicite, hors fenêtre 2005-2050) |
| `solar_position` | `spa::{solar_position_precise, DEFAULT_ATMOSPHERIC_REFRACTION_AT_HORIZON_DEG}` | `solar_position::spa::{solar_position, solar_position_detailed}`, `validation` |
| `validation` | `dataset::*`, `global_campaign::*`, `report::*` (crate entier) | Suite de tests uniquement (`cargo test -p validation`) ; `validation` n'est jamais une dépendance de `simulation` ni d'un futur crate applicatif — c'est un outil de développement/CI, pas une brique du moteur de production |

### API expérimentale — forme non garantie, susceptible d'évoluer sans incrément MINOR

Un seul item est classé ainsi à ce jour : `solar_position::spa::{solar_position_detailed, SpaDetailedResult}`.
Contrairement au reste de l'API interne (dont la forme est dictée par
l'algorithme publié et donc stable), `SpaDetailedResult` expose un
ensemble de champs intermédiaires (parallaxe, ascension droite topocentrique,
etc.) choisi pour les besoins actuels de `validation::report` — cet
ensemble de champs pourrait croître ou se réorganiser si un besoin de
diagnostic différent apparaît, sans que cela reflète un changement de
l'algorithme SPA lui-même. Un changement de forme de ce struct spécifique
est donc explicitement exempté de la règle §19.3.3 (pas besoin d'incrément
MINOR), tant que `solar_position_precise` et `solar_position` (l'API
publique, ci-dessus) restent inchangées.

### Ce que ce gel garantit concrètement à partir du tag `v1.0.0-scientific-core`

- Aucune signature de l'API publique (tableau ci-dessus) ne changera sans
  incrément MINOR documenté et note de migration.
- `pv_module` et les modules suivants pourront dépendre de l'API publique
  en toute confiance dès maintenant.
- Toute évolution de l'API interne ou expérimentale reste possible sans
  cérémonie, tant que l'API publique et les budgets de précision
  documentés (`18-annexe-technique-position-solaire.md` §18.7) restent
  intacts — vérifié à chaque commit par la CI (§7 de
  `docs/design/20-rapport-cloture-phase1.md`).
