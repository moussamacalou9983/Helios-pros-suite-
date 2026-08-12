# 21. Matrice de traçabilité — Exigence → Module → Tests → Documentation

**Statut** : matrice de clôture de la Phase 1 (moteur scientifique,
tag `v1.0.0-scientific-core`). Couvre (A) les exigences d'ingénierie qui
ont piloté la construction du moteur scientifique depuis le premier brief
de session jusqu'à cette clôture, et (B) les exigences fonctionnelles/non
fonctionnelles du produit complet (`01-cahier-des-charges.md`), pour
qu'aucune exigence ne reste sans statut explicite — y compris celles
délibérément hors périmètre de cette phase, qui sont marquées comme
telles avec un renvoi vers la phase de la feuille de route
(`15-feuille-de-route.md`) qui les couvrira, plutôt que simplement
omises.

## A. Exigences d'ingénierie du moteur scientifique

| ID | Exigence | Module(s) | Tests | Documentation |
|---|---|---|---|---|
| EI-001 | Workspace Cargo, 14 crates exacts, édition Rust 2024 | `engine/Cargo.toml`, les 14 crates | `cargo build --workspace` | `engine/README.md` |
| EI-002 | Zéro dépendance externe hors bibliothèque standard | tous | `cargo tree --workspace` (dépendances `path` uniquement) | `engine/README.md` |
| EI-003 | Toutes les unités en SI via types forts, conversions automatiques | `core_units` | 17 tests unitaires (`Angle`, `Length`, `Pressure`, `Ratio`, `Temperature`, `Velocity`, `Irradiance`) | rustdoc de chaque type |
| EI-004 | Aucune constante non référencée | `core_constants` + constantes locales sourcées dans chaque crate | audit `RUSTFLAGS="-D missing_docs"` (0 avertissement) | `core_constants/src/lib.rs`, `docs/references/bibliography.md` |
| EI-005 | Aucune formule inventée, toute formule référencée à une publication | `solar_geometry`, `solar_position`, `irradiance`, `transposition`, `thermal_model` | tests de non-régression + validation externe (voir EI-010) | `18-annexe-technique-position-solaire.md`, `docs/references/bibliography.md` |
| EI-006 | Moteur déterministe (pas d'aléatoire, pas d'E/S dans le calcul) | `solar_geometry::spa`, `solar_position::spa` | `reda_andreas_2004_worked_example_geocentric`/`_full_topocentric` (valeurs numériques figées) | `18-annexe...md` §18.4 |
| EI-007 | SPA NREL complet (Reda & Andreas 2004) comme moteur primaire | `solar_geometry::spa`, `solar_position::spa` | `spa_matches_pvlib_to_a_tiny_fraction_of_a_degree[_across_the_whole_world]` | `18-annexe...md` §18.4, `docs/references/bibliography.md#ref-reda2004` |
| EI-008 | Meeus (1998) conservé comme moteur secondaire de validation croisée | `solar_geometry::meeus`, `solar_position::meeus` | `meeus_matches_pvlib_within_its_documented_accuracy_budget[...]`, `spa_and_meeus_agree_within_their_documented_accuracy_budget` | `18-annexe...md` §18.5 |
| EI-009 | Abstraction sélectionnable SPA/Meeus, SPA par défaut | `solar_position::algorithm` (`SolarPositionAlgorithm`) | `algorithm_names_identify_the_engine_and_its_role`, `default_entry_point_matches_explicit_spa_selection` | rustdoc `solar_position::lib`, `19-politique-semver-api.md` §19.2 |
| EI-010 | Campagne de validation automatisée vs pvlib (max/moyenne/RMS/relative/temps) | `validation::report` | `report::tests::*` (91 cas) | `latest_validation_report.md` |
| EI-011 | Chaque module documenté (hypothèses/limites/unités/équations/références) | tous | audit `RUSTFLAGS="-D missing_docs"` (0 avertissement, aucune dérogation) | chaque `lib.rs`/`mod.rs`, `19-politique-semver-api.md` §19.1 |
| EI-012 | Architecture compatible futurs moteurs, tout en restant indépendante | `core_units`/`core_math`/`core_constants` (agnostiques du domaine), patron `SolarPositionAlgorithm` | — (revue architecturale, aucune dépendance inverse constatée) | `engine/README.md` §« Extending the engine » |
| EI-013 | Couverture de tests > 95 % | tous | `cargo llvm-cov --workspace` : 97.99 % lignes | `engine/validation-data/latest_coverage_report.txt` |
| EI-014 | 0 avertissement, `clippy` propre, 0 panic non justifié | tous | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --release` | messages de commit, `20-rapport-cloture-phase1.md` |
| EI-015 | Documentation API publique complète | tous | audit `RUSTFLAGS="-D missing_docs"` | `19-politique-semver-api.md` §19.1 |
| EI-016 | Benchmarks de performance (budget de non-régression) | `tests::benchmarks` | `spa_computes_one_year_of_hourly_positions_within_budget`, `meeus_...`, `spa_is_not_dramatically_slower_than_meeus` | `latest_benchmark_report.md` |
| EI-017 | Campagne de validation mondiale (enveloppe complète lat/lon/alt/années) | `validation::global_campaign` | 7 tests (`spa_matches_pvlib_to_a_tiny_fraction_of_a_degree_across_the_whole_world`, etc.) | `latest_global_campaign_report.md` |
| EI-018 | Analyse d'incertitude (arrondi, `f32`/`f64`, constantes astronomiques) | `solar_geometry::spa::precision_experiment`, `core_math::stats` | `f32_vs_f64_summation_of_the_largest_periodic_series` | `17-analyse-incertitude-numerique.md` |
| EI-019 | Tests de stabilité numérique (angles limites, pôles, années extrêmes, minuit) | `tests::numerical_stability` | 6 tests dédiés | `17-...md`, doc de module du fichier de test |
| EI-020 | Benchmarks étendus (moyenne, écart-type, mémoire, allocations) | `tests::benchmarks`, `core_math::stats::standard_deviation` | `spa_timing_mean_and_standard_deviation`, `spa_allocates_nothing_on_the_heap_per_call`, etc. | `latest_benchmark_report.md` |
| EI-021 | Document scientifique complet, publiable comme annexe technique | — (documentation) | — | `18-annexe-technique-position-solaire.md` |
| EI-022 | API publique stable, politique SemVer explicite | tous | audit `RUSTFLAGS="-D missing_docs"` | `19-politique-semver-api.md` |
| EI-023 | Gel officiel : tag Git + Release Notes | — | `git tag -v v1.0.0-scientific-core` | `docs/releases/v1.0.0-scientific-core.md` |
| EI-024 | Séparation explicite API publique / interne / expérimentale | `solar_geometry`, `solar_position`, `validation` | — | `19-politique-semver-api.md` §19.6 |
| EI-025 | Bibliographie complète liée aux modules | — (documentation) | — | `docs/references/bibliography.md`, `bibliography.bib` |
| EI-026 | Matrice de traçabilité Exigence → Module → Tests → Documentation | — | — | ce document |
| EI-027 | Gel des données de validation (jeux de référence figés) | `validation-data/` (source), `validation/golden/` (copie figée) | `dataset::tests::dataset_has_the_expected_case_count`, `global_campaign::dataset::tests::dataset_parses_to_the_expected_case_count` (vérifient l'intégrité des jeux sources) | `validation/golden/README.md` |
| EI-028 | Intégration continue complète, bloquant toute régression | `.github/workflows/ci.yml` | Le workflow CI lui-même (`fmt`, `clippy`, `test`, `doc`, validation, benchmarks) | `.github/workflows/ci.yml` (commentaires en tête de fichier) |
| EI-029 | Rapport final de clôture de Phase 1 | — (documentation) | — | `20-rapport-cloture-phase1.md` |

**Bilan Partie A** : 29/29 exigences couvertes (module, test et/ou
documentation identifiés pour chacune). Aucune exigence d'ingénierie de
cette phase ne reste sans preuve associée.

## B. Exigences du cahier des charges produit (`01-cahier-des-charges.md`)

Le périmètre de cette phase est explicitement le **moteur scientifique
seul** (consigne reçue en début de session : « Ne développe pas encore :
Interface graphique, Base de données, API, Web, IA, Rapports »). La
majorité des exigences fonctionnelles produit (EF-xxx) et non
fonctionnelles (ENF-xxx) relèvent donc, par construction, de phases
ultérieures de `15-feuille-de-route.md` — elles sont listées ci-dessous
avec un statut explicite, jamais silencieusement omises.

| ID | Exigence (résumé) | Statut Phase 1 | Détail |
|---|---|---|---|
| EF-001 | Projet géolocalisé | Hors périmètre | Couche application/GUI — prévu Phase 1 (roadmap) « Frontend web minimal » / Phase 2 |
| EF-002 | Import météo TMY (PVGIS, NASA POWER, etc.) | Hors périmètre | Service Météo applicatif — prévu Phase 1 (roadmap) |
| EF-003 | Géométrie 3D et masques | Hors périmètre | Service SIG — prévu Phase 1 (roadmap, minimal) puis Phase 2 (3D complet) |
| EF-004 | Concevoir un générateur PV (modules/chaînes/onduleurs) | Squelette prêt, non implémenté | `pv_module`, `electrical_model`, `losses` : crates scaffoldés et documentés (0 logique) — prochain module de développement, cf. instruction reçue |
| EF-005 | Simuler la production horaire | Partiellement couvert | `simulation::compute_energy_balance` calcule position solaire → irradiance POA → température de cellule ; s'arrête avant la puissance DC/AC (attend `electrical_model`) — voir `simulation/src/lib.rs` |
| EF-006 | Stockage batterie | Hors périmètre | Prévu Phase 3 (roadmap) |
| EF-007 | Micro-réseau hybride | Hors périmètre | Prévu Phase 3 |
| EF-008 | Pompage solaire | Hors périmètre | Prévu Phase 3 |
| EF-009 | Recharge VE | Hors périmètre | Prévu Phase 3 |
| EF-010 | Analyse économique | Hors périmètre | Prévu Phase 2 |
| EF-011 | Optimisation automatique | Hors périmètre | Prévu Phase 2 |
| EF-012 | Rapport d'étude PDF/DOCX | Hors périmètre | Explicitement exclu de cette session (« Ne développe pas encore : Rapports ») ; prévu Phase 1 (roadmap, basique) / Phase 2 (personnalisable) |
| EF-013 | Import/export de données | Hors périmètre | Couche application — `validation-data/` a néanmoins son propre mécanisme d'import/export interne au moteur (CSV, JSON), documenté dans `validation-data/README.md` |
| EF-014 | Assistance IA | Hors périmètre | Explicitement exclu (« Ne développe pas encore : IA ») ; prévu Phase 4 |
| EF-015 | Collaboration multi-utilisateur | Hors périmètre | Prévu Phase 2 |
| EF-016 | Base de données équipements | Hors périmètre | Explicitement exclu (« Ne développe pas encore : Base de données ») ; prévu Phase 1 (roadmap) |
| EF-017 | Conformité électrique IEC 60364/62548 | Hors périmètre | Attend `losses`/`electrical_model` (références IEC déjà documentées, voir `docs/references/bibliography.md`) ; module de conformité prévu Phase 3 |
| EF-018 | Bifacial et agrivoltaïsme | Hors périmètre | Prévu Phase 2 |
| EF-019 | Trackers et backtracking | Hors périmètre | Prévu Phase 2 |
| EF-020 | API publique REST | Hors périmètre | Explicitement exclu (« Ne développe pas encore : API ») ; prévu Phase 1 (roadmap) / Phase 4 (enrichie) |
| ENF-001 | Simulation horaire annuelle < 5 s | Couvert (pour la position solaire) | SPA : ~47 ms pour 8760 positions (`latest_benchmark_report.md`), très en dessous du budget ; la chaîne complète avec rendement électrique sera re-mesurée une fois `electrical_model` livré |
| ENF-002 | Optimisation < 5 min | Hors périmètre | Aucun optimiseur dans le périmètre Phase 1 |
| ENF-003 | Précision < 3 %/< 5 % vs mesures/inter-outils | Partiellement couvert | Le facteur géométrique/solaire de l'erreur est validé à < 0.01° (`latest_global_campaign_report.md`), très en dessous de ce qu'exige un budget de 3 % sur la production annuelle ; le facteur électrique (non encore implémenté) reste à valider séparément |
| ENF-004 | SLA SaaS ≥ 99.5 % | Hors périmètre | Infrastructure applicative, sans rapport avec le moteur de calcul |
| ENF-005 | Sécurité (chiffrement, OIDC/MFA) | Hors périmètre | Infrastructure applicative |
| ENF-006 | Portabilité Web/Desktop | Hors périmètre | Couche application ; le moteur lui-même est déjà portable par construction (Rust standard, zéro dépendance système) |
| ENF-007 | Localisation i18n | Hors périmètre | Couche application |
| ENF-008 | Accessibilité WCAG 2.1 AA | Hors périmètre | Couche application (aucune UI dans ce périmètre) |
| ENF-009 | Couverture ≥ 85 % cœur métier, ≥ 95 % modèles physiques | **Couvert et dépassé** | 97.99 % de couverture mesurée sur l'ensemble du moteur scientifique actuel — précisément le périmètre « modèles physiques » visé par cette exigence (`latest_coverage_report.txt`) |
| ENF-010 | Scalabilité 500 MWc / >100k modules | Hors périmètre | Non applicable au moteur de position solaire seul ; performance par site déjà mesurée (`latest_benchmark_report.md`) |
| ENF-011 | Journal d'audit immuable | Hors périmètre | Couche application/persistance |
| ENF-012 | Export intégral (pas de données prisonnières) | Hors périmètre | Couche application ; le moteur n'a pas d'état persistant à exporter |
| ENF-013 | Confidentialité multi-tenant | Hors périmètre | Infrastructure applicative |

**Bilan Partie B** : 33/33 exigences produit ont un statut explicite. 2
couvertes/dépassées (ENF-001, ENF-009), 2 partiellement couvertes (EF-005,
ENF-003), 1 en squelette prêt (EF-004), 28 explicitement hors périmètre
avec renvoi vers la phase de la feuille de route qui les traitera. Aucune
exigence n'est laissée sans trace.

## C. Exigences de conception `pv_module` (Phase 2 — conception)

Exigences issues des briefs de conception successifs de `pv_module`,
**mises à jour après l'implémentation du 11 août 2026**. La colonne
« Implémentation » désigne le code livré ; la colonne « Tests » désigne
des tests **existants et passants**, nommés d'après l'exigence ou
l'invariant qu'ils couvrent.

| ID | Exigence | Spécification | Implémentation | Tests | Documentation |
|---|---|---|---|---|---|
| PVM-001 | Architecture interne : responsabilités, interfaces, dépendances autorisées/interdites | doc 24 §1, doc 27 §13.3 | `lib.rs`, découpage en 13 modules | compilation, `cargo doc` | ADR-0004 |
| PVM-002 | Modèle de données justifié champ par champ | doc 25 §10, doc 27 §4-§10 | `identity.rs`, `technology.rs`, `geometry.rs`, `configuration.rs`, `electrical.rs`, `thermal.rs`, `condition.rs`, `bifacial.rs` | 88 tests unitaires | doc 25, doc 27 |
| PVM-003 | Compatibilité fabricants sans modification d'architecture | doc 25 §1.3, §3 | `MountingForm`, `PowerTolerance`, `CellDivision` | `a_flexible_module_carries_a_bend_radius…`, `both_forms_of_production_tolerance…` | doc 25 |
| PVM-004 | Compatibilité bases de données (CEC, Sandia, PAN, CSV, JSON, IEC 61853) | doc 25 §6, doc 26, doc 27 §12 | `adapters.rs` (interfaces), `ADAPTER_CATALOGUE` | `the_catalogue_names_the_five_specified_sources`, `the_pan_adapter_is_blocked…`, `a_record_shaped_like_a_cec_row…` | doc 26 |
| PVM-005 | Validation scientifique : définition, unité, mesure, référence par champ | doc 25 §10 | documentation `///` de chaque champ | `cargo doc` sans avertissement | doc 25 §10 |
| PVM-006 | Extensibilité (TOPCon, HJT, IBC, CdTe, CIGS, tandem, pérovskite, bifacial, flexible) | doc 25 §5, doc 27 §7.3 | `CellTechnology` récursif, `#[non_exhaustive]` | `tandem_nests_recursively`, `a_tandem_of_tandems_is_representable`, `a_thin_film_module_is_describable…` | ADR-0004 D-3 |
| PVM-007 | Diagrammes UML, dépendances, flux | doc 24 §7, doc 25 §9, doc 27 §13 | — | — | doc 27 §13 |
| PVM-008 | Classification épistémique A–H des données | doc 27 §1 | `provenance/classification.rs` | `codes_match_the_specification_letters`, `one_quantity_classifies_differently…` | ADR-0006 D-2 |
| PVM-009 | Provenance par valeur (9 champs) | doc 27 §2 | `provenance/source.rs`, `provenance/measurement.rs` | `the_full_provenance_trail_survives_construction`, `inv_prv_001_…` | ADR-0006 D-1 |
| PVM-010 | Système formel de conflits (VALID/WARNING/CONFLICT/INVALID/UNKNOWN) | doc 27 §3 | `validation/status.rs`, `validation/issue.rs` | `severity_orders_from_valid_to_invalid`, `only_conflicts_are_resolvable` | ADR-0006 D-3, D-4 |
| PVM-011 | **Aucun arbitrage silencieux d'une donnée en conflit** | doc 27 §3.3 | `validation/conflict.rs`, `PvModuleBuilder::build` | **`a_conflict_blocks_and_names_both_candidates`, `a_recorded_decision_lifts_the_conflict…`, `an_anonymous_decision_is_not_representable`, `every_conflict_needs_its_own_decision`** | ADR-0006 D-4 |
| PVM-012 | Séparation stricte données physiques / paramètres de modèle | doc 27 §4 | `INV-PRV-003` | `inv_prv_003_a_fitted_parameter_never_occupies_a_physical_field` | ADR-0004 D-4 |
| PVM-013 | Conditions de référence distinctes et abstraction générique | doc 27 §5 | `condition.rs`, `TemperatureSpecification` | `a_cell_temperature_is_never_comparable_to_an_ambient_one`, `ptc_shares_stc_irradiance_but_not_its_temperature_kind` | ADR-0006 D-5 |
| PVM-014 | Bifacialité structurée, modèles arrière exclus du module | doc 27 §6 | `bifacial.rs` | `inv_bif_001_…` à `inv_bif_004_…`, `the_advertised_bifacial_gain_is_recorded_but_structurally_unusable` | ADR-0006 D-6 |
| PVM-015 | Technologie descriptive, jamais prescriptive | doc 27 §7 | `technology.rs` — aucun branchement physique | `technology_and_architecture_are_independent`, `unknown_is_distinct_from_other` | ADR-0006 D-7 |
| PVM-016 | Cinq surfaces distinctes, aucune identifiée à une autre | doc 27 §8 | `geometry.rs`, `preferred_area_for_efficiency` | `rectangular_area_and_gross_area_are_separate_quantities`, `the_efficiency_invariants_name_the_area_they_used`, `inv_geo_002_…` | ADR-0006 D-6 |
| PVM-017 | Politique d'unités : SI interne strict, conversion à l'entrée | doc 27 §9 | `core_units` étendu (7 types), aucun `f64` nu à la frontière | `units_convert_once_at_the_boundary…`, `a_current_measured_in_milliamperes…` | ADR-0004 D-2 |
| PVM-018 | Incertitude associable à une donnée (GUM) | doc 27 §10 | `Uncertainty`, `Measurement<T>` | `expanded_uncertainty_divides_by_its_coverage_factor`, `an_uncertainty_reports_its_magnitude_and_its_form` | ADR-0006 D-1 |
| PVM-019 | Invariants de validation physique avec niveaux | doc 27 §11 (**52** invariants) | `validation/invariants.rs` | **`tests/invariants.rs` — 207 scénarios exécutés, ≥ 3 par invariant** | doc 27 §11, ADR-0007 |
| PVM-020 | Architecture des adaptateurs, modèle interne indépendant des sources | doc 27 §12 | `adapters.rs` — interfaces seules | `an_import_records_the_adapter_and_its_version…`, `a_missing_mandatory_field_yields_no_module_and_no_placeholder` | ADR-0004 D-5 |
| PVM-021 | Plan de tests | — | 175 tests | `cargo test --workspace` | doc 28 |
| PVM-022 | Aucun code métier durant la phase de conception | — | **Levée** : la conception est close, l'implémentation autorisée le 11/08/2026 | — | doc 27 §15 |
| PVM-023 | Inventaire des invariants vérifiable mécaniquement + `INV-STR-004` | ADR-0007 | `ALL_INVARIANT_IDS` (52), `is_known_invariant` | `the_invariant_inventory_is_complete_and_free_of_duplicates`, `every_family_is_represented_in_the_inventory`, `inv_str_004` | ADR-0007 |
| PVM-024 | **≥ 3 scénarios réellement exécutés par invariant, vérifié mécaniquement et protégé par la CI** | doc 29 §6.0 | `tests/scenario/mod.rs` — scénarios sous forme de données, recensement, matrice | `every_invariant_carries_at_least_three_scenarios`, `every_scenario_produces_the_outcome_it_declared`, `every_invariant_in_the_inventory_has_a_test_of_its_own`, + 6 auto-tests du harnais | doc 29 §6.0, `validation-data/invariant-scenario-matrix.md` |

**Bilan Partie C** : 24 exigences, dont **22 implémentées et testées**,
1 documentaire sans code attendu (PVM-007, diagrammes) et 1 levée par la
clôture de la phase de conception (PVM-022).

Couverture mesurée du crate `pv_module` : **99,51 % des régions,
99,61 % des lignes, 100 % des fonctions** (`cargo llvm-cov`).

Scénarios de validation mesurés : **207 exécutés sur 52 invariants,
207 PASS, 0 FAIL, minimum 3 par invariant** — matrice complète dans
`engine/validation-data/invariant-scenario-matrix.md`, régénérée à chaque
exécution de la suite et publiée en artefact de CI.

Trois exigences restent **ouvertes** et sont suivies comme telles :

| Exigence | Point ouvert | Suivi |
|---|---|---|
| PVM-003 | Validation sur ≥ 5 fiches techniques réelles — pas d'accès réseau aux PDF constructeurs | doc 28, T4.9 |
| PVM-004 | Grille exacte IEC 61853-1 — lecture du texte normatif requise | doc 27 §12.3 |
| PVM-004 | `PANAdapter` — statut juridique du format à vérifier | doc 27 §12.4 |

---

## Partie D — Exigences de conception d'`electrical_model`

Exigences issues du brief de conception d'`electrical_model`
(11 août 2026). **Aucune n'est couverte par du code** : le crate demeure
le squelette livré en Phase 1, et l'implémentation est conditionnée à une
validation explicite. La colonne « Tests » désigne le test **planifié**
(`docs/design/31-plan-tests-electrical-model.md`), pas un test existant.

| ID | Exigence | Spécification | Tests planifiés | Documentation |
|---|---|---|---|---|
| ELM-001 | Objectif du moteur : I-V, P-V, `V_oc`, `I_sc`, `V_mp`, `I_mp`, `P_max`, MPP, hors STC | doc 30 §1.1 | E1, E2, E3 | doc 30 |
| ELM-002 | Chaîne données → paramètres → modèle → I-V → P-V → MPP | doc 30 §2 | E4, E8 | doc 30 §2 |
| ELM-003 | Comparaison argumentée des modèles avant décision | doc 30 §5 | — (documentaire) | doc 30 §5 |
| ELM-004 | **Aucun modèle unique pour toutes les données** | doc 30 §5.3, §9.3 | E6.2, E7.1 | ADR-0008 D-1 |
| ELM-005 | Équation à diode unique complète, sept grandeurs définies | doc 30 §4.1, §4.2 | E1, E2 | doc 30 §4 |
| ELM-006 | Méthodes d'identification des paramètres distinguées | doc 30 §7.2 | E6, E8.2 | ADR-0008 D-3 |
| ELM-007 | Comparaison Lambert W / Newton / Brent avant choix | doc 30 §8.2–§8.4 | E1.6, E3.4, E3.5 | ADR-0008 D-4 |
| ELM-008 | **Jamais confondre données constructeur et paramètres de modèle** | doc 30 §3 | **E7.1, E7.3, E8.1, E8.4** | ADR-0008 D-3 |
| ELM-009 | Paramètres portant classification, provenance, méthode, incertitude, modèle | doc 30 §3.2 | E8.1–E8.5 | ADR-0008 D-3 |
| ELM-010 | Compatibilité CEC / Sandia / De Soto / IEC 61853 établie | doc 30 §9 | E9.1, E9.2 | doc 26, doc 30 §9 |
| ELM-011 | Matrice modèle × données requises × disponibles × manquantes × méthode | doc 30 §9.2 | E6.5 | doc 30 §9.2 |
| ELM-012 | Tolérances, convergence, itérations, non-convergence définies avant implémentation | doc 30 §8.1, §8.7 | E3.1–E3.3 | ADR-0008 D-5 |
| ELM-013 | **Aucune boucle numérique sans critère de convergence formel** | doc 30 §8.1 | **E3.2, E3.3** | ADR-0008 D-5 |
| ELM-014 | 14 cas limites spécifiés avant codage | doc 30 §10 | E5.1–E5.14 | doc 30 §10 |
| ELM-015 | Stratégie de validation indépendante à cinq niveaux | doc 30 §11.2 | E1, E2, E4, E9, E10 | doc 30 §11 |
| ELM-016 | **Une implémentation de référence n'est jamais une preuve de vérité** | doc 30 §11.1 | **E10.2** | doc 30 §11.1 |
| ELM-017 | SI strict à l'intérieur, conversions aux frontières | doc 30 §12 | E7.6 | doc 30 §12 |
| ELM-018 | Architecture ouverte aux extensions futures sans modification | doc 30 §13.4 | E7.4, E7.5 | ADR-0008 |
| ELM-019 | Diagrammes UML | doc 30 §2, §13.1, §13.2 | — | doc 30 §13 |
| ELM-020 | Plan de tests | — | doc 31 | doc 31 |
| ELM-021 | **Aucun code de calcul durant la phase de conception** | — | E7.4 | doc 30 §14 |
| ELM-022 | **SAPM ne produit pas de courbe continue — séparation structurelle** | doc 30 §5.6 | **E7.2** (`compile_fail`) | ADR-0008 D-2 |
| ELM-023 | Monotonie du résidu démontrée, encadrement garanti | doc 30 §4.5 | E3.4, E3.5 | ADR-0008 D-4 |
| ELM-024 | ≥ 3 scénarios par règle et par cas limite, mesurés et protégés par la CI | doc 31 §0 | E5, doc 31 §12 | doc 29 §6.0 |

| ELM-025 | **Trois abstractions** : `ParameterizedElectricalModel` / `PointBasedPerformanceModel` / `ContinuousIVModel` | doc 32 §2 | **E7.2 (`compile_fail`)** | ADR-0009 D-1 |
| ELM-026 | `ValidityDomain` obligatoire — un modèle qui ignore son domaine ne peut pas refuser | doc 32 §2.4 | E5.1, E9.1 | ADR-0009 D-1 |
| ELM-027 | Cinq statuts de suffisance des données, matrice source × modèle complète | doc 32 §8 | E6.2, E6.5 | ADR-0009 D-2 |
| ELM-028 | Identification `FivePointDeSoto` initialisée par Phang ; **Villalva écartée** (paramètre libre arbitraire) | doc 32 §3.3, §3.4 | E6.1, E6.6 | ADR-0009 D-3 |
| ELM-029 | **Newton globalement convergent** — démonstration (décroissance + concavité) | doc 32 §4.3 | E1.6, E3.4, E3.5 | ADR-0009 D-4 |
| ELM-030 | **Le résidu majore l'erreur** — critère de convergence = borne garantie | doc 32 §4.4 | E3.1 | ADR-0009 D-4 |
| ELM-031 | `R_s = 0` : résidu affine, **solution exacte sans itération** | doc 32 §4.5 | E1.1, E1.2, E5.4 | ADR-0009 D-4 |
| ELM-032 | **Lambert W non implémentée** — ne paie pas son coût | doc 32 §5 | — (absence vérifiée) | ADR-0009 D-5 |
| ELM-033 | **MPP résolu comme racine de `dP/dV`**, jamais par maximisation de `P` (plafond `√ε`) | doc 32 §7.4 | E2.1, E5.10 | ADR-0009 D-4 |
| ELM-034 | Unicité du MPP non garantie → **vérification obligatoire** du maximum | doc 32 §7.4 | E5.7, E5.10 | ADR-0009 D-4 |
| ELM-035 | `E_g` = propriété du **matériau**, jamais de l'architecture (HJT/TOPCon = silicium) | doc 32 §6.2 | E6.2 | ADR-0009 D-6 |
| ELM-036 | `E_g` appartient au **jeu de paramètres** — cohérence identification/translation | doc 32 §6.4 | E8.2, E8.4 | ADR-0009 D-6 |
| ELM-037 | Tolérances **dérivées** : 450× le plancher d'arrondi, 10⁻¹⁰ de l'incertitude des données | doc 32 §7.2, §7.3 | E3.1, E11.1 | ADR-0009 D-7 |
| ELM-038 | Bornes d'itération **calculées** : Brent 81 = 2·log₂((I_L+I_0)/τ) | doc 32 §7.3 | E3.2 | ADR-0009 D-7 |
| ELM-039 | **Vérification dimensionnelle** de chaque équation avant implémentation | doc 32 §10 | E7.6 | ADR-0009 D-8 |
| ELM-040 | **`α_Isc` absolu dans De Soto, relatif dans le SAPM** — facteur 10 | doc 32 §10.4 | **E7.6** | ADR-0009 D-8 |
| ELM-041 | `C_3` (SAPM) en V⁻¹, contrairement à `C_0`–`C_2` sans dimension | doc 32 §10.5 | E7.6 | ADR-0009 D-8 |
| ELM-042 | IEC 61853 = **modèle à part entière** (interpolation, zéro paramètre ajusté) | doc 32 §1.3 | E9.3 | ADR-0009 D-11 |
| ELM-043 | Deux diodes **hors périmètre** — sous-déterminé sans hypothèses injustifiées | doc 32 §1.4 | — | ADR-0009 D-10 |
| ELM-044 | Tests de conservation C-1 à C-8, dont **zéro NaN/Inf en sortie** | doc 32 §9.3 | E2, E3.6 | doc 32 §9.3 |

**Bilan Partie D** : 44 exigences de conception spécifiées et reliées à un
test planifié ou à un contrôle documentaire. **0/44 implémentées** — état
attendu et conforme : l'implémentation est conditionnée à une validation
explicite.

### Fermeture des sept décisions (phase 2A)

Les sept décisions ouvertes par l'ADR-0008 sont **fermées** par
l'ADR-0009, sur l'analyse de la doc 32.

| Décision | Fermée par | Référence |
|---|---|---|
| D-1 | Newton principal (convergence démontrée), forme fermée si `R_s = 0`, Brent en repli, **Lambert W écartée** | doc 32 §4.8, §5 |
| D-2 | Type propre pour `a` — `a + V_oc` doit être impossible à écrire | doc 32 §11.1 |
| D-3 | **Le moteur ne choisit pas** — il expose les modèles applicables avec leur statut | doc 32 §8 |
| D-4 | Refus **hors enveloppe de régression** via `ValidityDomain` ; pas de constante universelle | doc 32 §1.2, §2.4 |
| D-5 | `E_g` = matériau, jamais architecture ; appartient au jeu de paramètres ; sourcé ou `INSUFFICIENT_DATA` | doc 32 §6 |
| D-6 | PVsyst différé, ADR dédié ; **jamais un oracle** | doc 32 §9.2 |
| D-7 | `10⁻¹²·I_sc` relatif ; Newton 30 ; **Brent 81 calculé** ; critère sur le seul résidu | doc 32 §7 |

### Audit mathématique final (2026-08-12) — ADR-0010

L'audit (doc 34) a **invalidé deux garanties** annoncées par l'ADR-0009 et
**démontré** une propriété que celui-ci déclarait hors d'atteinte.

| ID | Exigence | Spécification | Tests planifiés | Documentation |
|---|---|---|---|---|
| ELM-045 | ⚠️ **RETRAIT** : « Newton converge depuis tout point de départ » — vraie dans ℝ, **fausse en `f64`** | doc 34 §4.2 | E3.4, E5.14 | ADR-0010 D-1 |
| ELM-046 | Garantie de remplacement : depuis `I₀ = 0`, itérés **prouvés** dans `[I*, I_L]`, `u_max` calculable a priori | doc 34 §4.3 | E3.1, E3.2 | ADR-0010 D-1 |
| ELM-047 | ⚠️ **RETRAIT** : borne de Brent « 81 » — la borne réelle de Brent est ≈ 1636 | doc 34 §9.2 | — | ADR-0010 D-2 |
| ELM-048 | Borne **82** démontrée pour un schéma à **bissection imposée** 1 itération sur 2 | doc 34 §9.3 | E3.2 | ADR-0010 D-2 |
| ELM-049 | ✅ **UNICITÉ DU MPP DÉMONTRÉE** — via `V_j`, 3 lemmes, vérifiée sur 9 jeux dont `R_sh = 1,5 Ω` | doc 34 §10 | E2.1, E5.7, E5.10 | ADR-0010 D-3 |
| ELM-050 | Solveur = Newton **avec** sauvegarde ; ni Newton seul, ni Brent seul | doc 34 §8.2 | E3.4, E3.5 | ADR-0010 D-4 |
| ELM-051 | ⚠️ Borne résidu/erreur **ne se transpose pas** en tension : `|ΔV| ≤ R_sh·|h|` | doc 34 §5.2 | E3.1, E7.6 | ADR-0010 D-5 |
| ELM-052 | `tol_V` seuil **distinct** : `10⁻¹²·V_oc/R_sh`, plafonné au plancher d'arrondi | doc 34 §12.3 | E3.1 | ADR-0010 D-5 |
| ELM-053 | **Aucun seuil** pour `R_s → 0` — pas de singularité, prédicat **exact** `R_s == 0` | doc 34 §6.2 | E1.1, E1.2, E5.4 | ADR-0010 D-6 |
| ELM-054 | Débordement `exp` : contrôle **a priori** + domaine logarithmique + `expm1`, **jamais `try/catch`** | doc 34 §7.2 | E5.14 | ADR-0010 D-7 |
| ELM-055 | ⚠️ **CORRECTION** : Phang exige les pentes `dV/dI`, **absentes des fiches** | doc 34 §13.2 | E6.1 | ADR-0010 D-8 |
| ELM-056 | Point de départ ≠ paramètre : solution convergée **vérifiée** contre les 5 contraintes | doc 34 §13.4 | E6.1, E6.6 | ADR-0010 D-8 |
| ELM-057 | Échec d'identification **n'invalide jamais le moteur** (`PublishedAsIs` reste utilisable) | doc 34 §13.5 | E6.2, E6.4 | ADR-0010 D-8 |
| ELM-058 | `E_g` : **constante d'ajustement du modèle (F)** ≠ **gap intrinsèque du matériau (B/C)** — deux types | doc 34 §14.3 | E8.2, E8.4 | ADR-0010 |
| ELM-059 | IEC 61853 : mesuré (B) / interpolé (D) / hors domaine **interdit** ; méthode d'interpolation **différée** | doc 34 §15 | E9.3 | ADR-0010 |
| ELM-060 | Tolérance relative **auto-cohérente jusqu'aux dénormaux** — marge 450× invariante de 20 A à 10⁻²⁰ A | doc 34 §12.5 | E3.1, E11.1 | ADR-0010 D-5 |
| ELM-061 | **22 jeux de tests** scientifiques préparés, ≥ 3 scénarios chacun | doc 34 §16 | E1–E5 | doc 34 §16 |
| ELM-062 | Purge des formulations absolues non prouvées | doc 34 §17 | — (revue documentaire) | ADR-0010 |

### Formalisation mathématique (2026-08-12) — ADR-0011

Aucun renversement — c'est en soi un résultat : les corrections de
l'ADR-0010 ont tenu à la formalisation complète. Une **simplification
structurelle** non prévue a en revanche été trouvée.

| ID | Exigence | Spécification | Tests planifiés | Documentation |
|---|---|---|---|---|
| ELM-063 | Équation single-diode formalisée : 11 symboles définis, conventions de signe | doc 35 §1 | PB-01 | ADR-0011 |
| ELM-064 | Analyse dimensionnelle terme à terme, `[∂f/∂I] = 1`, `[∂f/∂V] = Ω⁻¹` | doc 35 §1.4, §2.2, §2.4 | PB-16, E7.6 | ADR-0011 |
| ELM-065 | Dérivées `∂f/∂I`, `∂²f/∂I²`, `∂f/∂V` **calculées analytiquement** | doc 35 §2.2–§2.4 | PB-16 | ADR-0011 |
| ELM-066 | Existence + unicité de `I(V)` démontrées, bornes `[0, I_ph+I_0]` | doc 35 §3 | PB-01, PB-03 | ADR-0011 |
| ELM-067 | `I(V) ∈ C^∞` par le théorème des fonctions implicites | doc 35 §3.5 | PB-02, PB-04 | ADR-0011 |
| ELM-068 | **Invariant d'intervalle** de Newton : `I_k ∈ [I*, I_ph]` depuis `I₀=0` | doc 35 §4.4 | PB-17 | ADR-0011 |
| ELM-069 | Conditions d'arrêt **et de sortie** de Newton énumérées | doc 35 §4.5 | PB-15 | ADR-0011 |
| ELM-070 | `|I−I*| ≤ |f(I)|` démontrée **et** vérifiée dimensionnellement | doc 35 §5.1, §5.2 | PB-14 | ADR-0011 |
| ELM-071 | Repli spécifié : `I_low`, `I_high`, signe, réduction, arrêt | doc 35 §6.2 | PB-15 | ADR-0011 |
| ELM-072 | **Borne 82 démontrée**, garanties réelles de Brent documentées séparément | doc 35 §6.1, §6.3 | PB-15 | ADR-0011 |
| ELM-073 | Garde anti-débordement : seuil **imposé par IEEE 754**, non choisi | doc 35 §7.2 | PB-05, PB-18 | ADR-0011 D-6 |
| ELM-074 | Sous-flux démontré non dangereux | doc 35 §7.6 | PB-05, PB-12 | ADR-0011 |
| ELM-075 | `R_s = 0` : forme fermée démontrée + **inspection exhaustive** — aucune division par `R_s` | doc 35 §8 | PB-10 | ADR-0011 D-7 |
| ELM-076 | Unicité du MPP : **6 étapes démontrées séparément** | doc 35 §9.4 | PB-07, PB-08, PB-20..22 | ADR-0011 |
| ELM-077 | Domaine de validité du MPP : 7 hypothèses, rôle de chacune | doc 35 §10.1 | PB-18 | ADR-0011 D-5 |
| ELM-078 | **`dP/dV_j` explicite — la recherche du MPP n'appelle jamais le solveur** | doc 35 §11.2 | PB-07, PB-09 | **ADR-0011 D-1** |
| ELM-079 | Différentiation numérique **rejetée par calcul** (plafond `√ε`) | doc 35 §11.1 | PB-09 | ADR-0011 D-4 |
| ELM-080 | Algorithme MPP spécifié de bout en bout, 6 règles | doc 35 §12 | PB-07..09 | ADR-0011 |
| ELM-081 | **35 propriétés prouvées → 35 invariants logiciels nommés** | doc 36 §3 | PB-01..22 | ADR-0011 D-2 |
| ELM-082 | **15 observations numériques, aucune promue en théorème** | doc 36 §4 | — | ADR-0011 D-3 |
| ELM-083 | 15 hypothèses : 9 vérifiables (refus), 6 de modélisation (déclarées) | doc 36 §2 | PB-18 | ADR-0011 D-5 |
| ELM-084 | 8 validations empiriques requises, **aucune acquise** | doc 36 §5 | E9, E10 | ADR-0011 |
| ELM-085 | Protocole : 4 natures de validation distinguées, 7 familles | doc 36 §6 | A–G | ADR-0011 |
| ELM-086 | **22 tests de propriétés** définis, domaine de génération spécifié | doc 36 §7 | PB-01..22 | ADR-0011 |

**Bilan Partie D après formalisation** : **86 exigences, 0 implémentée**.
35 propriétés prouvées, 35 invariants logiciels, 22 tests de propriétés,
15 observations non promues, 8 validations empiriques en attente.

**Bilan Partie D après audit** : 62 exigences, **0 implémentée**. Deux
garanties excessives retirées, une propriété gagnée, deux corrections.

**Trois renversements** de recommandations provisoires sont consignés :
Newton porte la garantie (et non Brent), Lambert W n'est pas implémentée,
le MPP se résout comme racine de `dP/dV` et non par maximisation.

Une **découverte non prévue** de l'analyse dimensionnelle : `α_Isc` n'a
pas la même dimension dans De Soto (A/K) et dans le SAPM (K⁻¹), soit un
facteur `I_sco ≈ 10`. Ceci démontre rétroactivement que la
`CoefficientBasis` de `pv_module` était une nécessité dimensionnelle, non
une précaution.

Deux points ouverts **hérités** de `pv_module` restent bloquants pour une
validation complète, sans bloquer la conception :

| Point | Suivi |
|---|---|
| Cinq fiches constructeurs réelles — **à ne pas simuler** | doc 31 §13 |
| Grille exacte IEC 61853-1 — lecture du texte normatif requise | doc 30 §9.4 |
