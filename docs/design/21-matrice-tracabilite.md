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

Exigences issues des briefs de conception successifs de `pv_module`.
**Aucune n'est encore couverte par du code** : le crate demeure un
squelette à zéro élément public. La colonne « Tests » désigne le test
*planifié* (`docs/design/28-plan-tests-pv-module.md`), pas un test
existant.

| ID | Exigence | Spécification | Tests planifiés | Documentation |
|---|---|---|---|---|
| PVM-001 | Architecture interne : responsabilités, interfaces, dépendances autorisées/interdites | doc 24 §1, doc 27 §13.3 | T6.1, T6.2, T6.3 | ADR-0004 |
| PVM-002 | Modèle de données justifié champ par champ | doc 25 §10, doc 27 §4-§10 | T1, T4 | doc 25, doc 27 |
| PVM-003 | Compatibilité fabricants sans modification d'architecture | doc 25 §1.3, §3 | T4.9, T4.10 | doc 25 |
| PVM-004 | Compatibilité bases de données (CEC, Sandia, PAN, CSV, JSON, IEC 61853) | doc 25 §6, doc 26, doc 27 §12 | T4.1–T4.8 | doc 26 |
| PVM-005 | Validation scientifique : définition, unité, mesure, référence par champ | doc 25 §10 | — (documentaire) | doc 25 §10 |
| PVM-006 | Extensibilité (TOPCon, HJT, IBC, CdTe, CIGS, tandem, pérovskite, bifacial, flexible) | doc 25 §5, doc 27 §7.3 | T1.6, T4.10 | ADR-0004 D-3 |
| PVM-007 | Diagrammes UML, dépendances, flux | doc 24 §7, doc 25 §9, doc 27 §13 | — | doc 27 §13 |
| PVM-008 | Classification épistémique A–H des données | doc 27 §1 | T5.2, T5.4 | ADR-0006 D-2 |
| PVM-009 | Provenance par valeur (9 champs) | doc 27 §2 | T5.1, T5.3, T5.6 | ADR-0006 D-1 |
| PVM-010 | Système formel de conflits (VALID/WARNING/CONFLICT/INVALID/UNKNOWN) | doc 27 §3 | T2.12, T3.1–T3.6 | ADR-0006 D-3, D-4 |
| PVM-011 | **Aucun arbitrage silencieux d'une donnée en conflit** | doc 27 §3.3 | **T3.1–T3.6** | ADR-0006 D-4 |
| PVM-012 | Séparation stricte données physiques / paramètres de modèle | doc 27 §4 | T4.8, T2.10 | ADR-0004 D-4 |
| PVM-013 | Conditions de référence distinctes et abstraction générique | doc 27 §5 | T1.5, T2.8, T4.6 | ADR-0006 D-5 |
| PVM-014 | Bifacialité structurée, modèles arrière exclus du module | doc 27 §6 | T2.9, T4.4 | ADR-0006 D-6 |
| PVM-015 | Technologie descriptive, jamais prescriptive | doc 27 §7 | **T6.4** | ADR-0006 D-7 |
| PVM-016 | Cinq surfaces distinctes, aucune identifiée à une autre | doc 27 §8 | T2.4, T2.5, T3.6 | ADR-0006 D-6 |
| PVM-017 | Politique d'unités : SI interne strict, conversion à l'entrée | doc 27 §9 | T1.1, T1.2, T6.5, T4.3 | ADR-0004 D-2 |
| PVM-018 | Incertitude associable à une donnée (GUM) | doc 27 §10 | T1.3, T1.4 | ADR-0006 D-1 |
| PVM-019 | Invariants de validation physique avec niveaux | doc 27 §11 (47 invariants) | **T2.1–T2.12** | doc 27 §11 |
| PVM-020 | Architecture des adaptateurs, modèle interne indépendant des sources | doc 27 §12 | T4, T6.1 | ADR-0004 D-5 |
| PVM-021 | Plan de tests | — | doc 28 | doc 28 |
| PVM-022 | Aucun code métier durant la phase de conception | — | T6.3 | doc 27 §15 |

**Bilan Partie C** : 22/22 exigences de conception spécifiées et reliées
à un test planifié ou à un contrôle documentaire. **0/22 implémentées** —
état attendu et conforme : l'implémentation est conditionnée à une
validation explicite.

Trois exigences restent **ouvertes** et sont suivies comme telles :

| Exigence | Point ouvert | Suivi |
|---|---|---|
| PVM-003 | Validation sur ≥ 5 fiches techniques réelles — pas d'accès réseau aux PDF constructeurs | doc 28, T4.9 |
| PVM-004 | Grille exacte IEC 61853-1 — lecture du texte normatif requise | doc 27 §12.3 |
| PVM-004 | `PANAdapter` — statut juridique du format à vérifier | doc 27 §12.4 |
