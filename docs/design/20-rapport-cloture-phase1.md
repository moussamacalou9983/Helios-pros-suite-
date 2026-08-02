# 20. Rapport final de clôture — Phase 1 (Moteur scientifique HeliosPro)

**Document officiel de clôture de la Phase 1.** Tag associé :
`v1.0.0-scientific-core`. Voir `docs/releases/v1.0.0-scientific-core.md`
pour les notes de version détaillées et `docs/design/21-matrice-tracabilite.md`
pour la preuve exigence par exigence que ce rapport synthétise.

## 1. Résumé exécutif

La Phase 1 livre le **moteur de position solaire** de HeliosPro : calcul
déterministe, en Rust pur (zéro dépendance hors bibliothèque standard),
de la position topocentrique du Soleil (azimut, élévation, zénith) et des
grandeurs géocentriques associées (déclinaison, ascension droite,
distance Terre-Soleil, équation du temps), via deux moteurs indépendants
et sélectionnables — l'algorithme SPA complet du NREL (Reda & Andreas,
2004, primaire) et un moteur réduit de Meeus (1998, secondaire, validation
croisée) — plus les modules de support directement nécessaires
(irradiance extraterrestre, décomposition GHI/DNI/DHI, transposition sur
plan incliné à 4 modèles, modèle thermique de cellule à 2 modèles).

Le moteur a été validé scientifiquement, pas seulement testé au niveau du
code : campagne de validation externe de 2918 cas couvrant l'enveloppe
complète latitude/longitude/altitude/année du produit, analyse
d'incertitude numérique quantifiée, tests de stabilité aux cas limites,
benchmarks de performance et de mémoire, documentation technique
publiable, et vérification mécanique de la couverture de documentation et
de tests. Ce rapport en est la synthèse ; chaque chiffre cité est tiré
d'un rapport auto-généré ou d'une mesure reproductible, jamais estimé.

## 2. Architecture

14 crates dans le workspace `engine/` (édition Rust 2024, résolveur de
dépendances v2), organisés en couches strictement acycliques :

```
core_constants, core_units, core_math   (fondations, agnostiques du domaine PV)
        │
solar_geometry ──► solar_position       (position solaire : géocentrique → topocentrique)
        │
irradiance ──► transposition            (irradiance extraterrestre, décomposition, plan incliné)
        │
thermal_model                            (température de cellule)
        │
simulation                               (orchestration : position → irradiance POA → température)
        │
pv_module, electrical_model, losses      (squelettes documentés — prochain module de développement)
        │
validation, tests                        (validation externe + tests croisés, jamais une dépendance
                                           du moteur de production)
```

9 crates portent une implémentation complète (`core_constants`,
`core_units`, `core_math`, `solar_geometry`, `solar_position`,
`irradiance`, `transposition`, `thermal_model`, `simulation`), 2 crates
sont des outils de développement/validation (`validation`, `tests`, non
consommés par le moteur de production), 3 crates restent des squelettes
documentés sans logique (`pv_module`, `electrical_model`, `losses`) —
voir `engine/README.md` pour la carte complète et
`docs/design/19-politique-semver-api.md` pour la surface d'API exacte de
chaque crate, gelée à ce tag.

## 3. Précision scientifique

| Comparaison | Cas | Écart absolu maximal mesuré |
|---|---|---|
| SPA HeliosPro vs pvlib-python (échantillon 91 cas, sites/dates représentatifs) | 91 | < 0.0002° (toutes grandeurs angulaires) |
| SPA HeliosPro vs pvlib-python (campagne mondiale, enveloppe complète) | 2918 | < 0.01° |
| Meeus (secondaire) vs pvlib-python (hors singularités documentées) | 2918 | < 0.02-0.06° (< 0.02° pour les grandeurs géocentriques) |

Deux effets de bord réels — pas des défauts de code — ont été mesurés,
expliqués physiquement et bornés par des tests dédiés plutôt que masqués
par un seuil élargi sans justification : une divergence de convention de
coupure de réfraction atmosphérique près de l'horizon entre SPA et Meeus
(chaque moteur suit fidèlement sa publication source, qui définit cette
coupure différemment), et une singularité de coordonnées de la formule
d'azimut au passage au zénith (propriété mathématique de la formule
elle-même, partagée par les deux moteurs). Voir
`docs/design/18-annexe-technique-position-solaire.md` §18.6-18.8 pour les
démonstrations et `engine/crates/validation/src/global_campaign/mod.rs`
pour la mesure et la preuve.

**Sources d'incertitude quantifiées** (`docs/design/17-analyse-incertitude-numerique.md`) :
arrondi `f64` négligeable (~4×10⁻⁵ s au niveau du Jour Julien sur
1900-2100) ; `f32` mesuré ~0.02″ de divergence sur la seule série
héliocentrique dominante (justifie le choix de `f64` partout) ; ΔT
(TT−UT1) identifié comme le facteur dominant hors de sa fenêtre de
validité documentée (2005-2050), avec analyse chiffrée de l'extrapolation
polynomiale.

## 4. Performances

Mesures `std::time::Instant` (aucune dépendance de benchmarking externe),
sur un an d'appels horaires (8760 positions) :

| Mesure | SPA | Meeus (réduit) |
|---|---|---|
| Temps moyen par appel | ~5.3-5.5 µs | ~0.6-0.9 µs |
| Écart-type par appel | ~2.7 µs | ~0.7 µs |
| Allocations sur le tas (8760 appels, régime stable) | **0** | **0** |
| Ratio SPA/Meeus | ~9-11× (attendu : SPA somme des séries périodiques bien plus grandes) |

Voir `engine/validation-data/latest_benchmark_report.md` (régénéré à
chaque exécution de `cargo test -p tests`) pour les chiffres exacts de la
dernière exécution, et `validation/golden/reports/` pour l'instantané
figé au tag de clôture.

## 5. Couverture de tests

Mesurée avec `cargo-llvm-cov` (outil de développement installé à la
demande, pas une dépendance du workspace) :

| Métrique | Couverture mesurée | Exigence |
|---|---|---|
| Lignes | **97.99 %** | > 95 % |
| Régions | 97.89 % | — |
| Fonctions | 98.01 % | — |

164 tests au total (`cargo test --workspace --release`), 0 échec.
Documentation de l'API publique : 100 % (vérifié en promouvant
`missing_docs` en erreur bloquante — `RUSTFLAGS="-D missing_docs" cargo
doc --workspace --no-deps`, 0 avertissement, aucune dérogation locale).
`cargo fmt --check` et `cargo clippy --workspace --all-targets -- -D
warnings` : 0 avertissement. Toutes ces vérifications sont désormais
automatisées par `.github/workflows/ci.yml` à chaque commit.

## 6. Limites connues

Consolidées depuis `docs/design/17-analyse-incertitude-numerique.md` §17.4
et `docs/design/18-annexe-technique-position-solaire.md` §18.9 :

1. **Temps UTC uniquement** — aucune notion de fuseau horaire/heure
   civile dans le moteur (décision d'architecture délibérée, pas un
   oubli).
2. **Calendrier grégorien uniquement**, années ≥ 1582.
3. **ΔT estimé** par défaut valide seulement 2005-2050 ; hors de cette
   fenêtre, un `delta_t` explicite doit être fourni via les entrées
   `*_precise` pour une exactitude garantie.
4. **Nutation SPA tronquée** à 63 des 106 termes IAU 1980 — hérité de la
   publication source (Reda & Andreas 2004), non re-dérivé
   indépendamment.
5. **Précision SPA annoncée (±0.0003°) valable −2000/6000** — la
   stabilité numérique (absence de NaN) a été testée jusqu'à l'année
   9999, mais aucune exactitude astronomique n'est revendiquée au-delà de
   la fenêtre publiée.
6. **Aucune comparaison directe avec PVsyst, SAM (interface native) ou
   Archelios Calc** — hors de portée dans l'environnement de
   développement actuel (aucun accès réseau/licence) ; pvlib-python reste
   la seule source de validation externe disponible à ce jour.
7. **Modèles d'irradiance/transposition/thermique** validés uniquement
   par invariants physiques internes (`tests` crate), pas encore par
   comparaison externe pvlib/SAM — contrairement à la position solaire,
   qui a bénéficié de la campagne de validation renforcée de cette phase.
8. **Chaîne de simulation incomplète** : `simulation::compute_energy_balance`
   s'arrête à la température de cellule ; aucune puissance DC/AC n'est
   encore produite (attend `electrical_model`).
9. **Aucun effet relativiste modélisé** — cohérent avec la publication
   SPA source, qui ne les inclut pas non plus (amplitude négligeable
   devant ±0.0003°).

## 7. Travaux futurs

Immédiat (prochain module, autorisé après cette clôture) :

1. **`pv_module`** — modèle de données fiche technique module PV
   (IEC 61215, IEC 61853, Sandia, CEC), premier module du périmètre
   électrique.
2. **`electrical_model`** — modèle à diode unique (De Soto et al. 2006),
   consommera `pv_module`.
3. **`losses`** — pertes systématiques (câblage IEC 60364-5-52/62548,
   mismatch, salissure, etc.).

Recommandations issues de cette phase (`docs/design/16-phase1-validation-report.md`
§17.5, `§20.7`) :

4. Étendre `validation` aux modules `irradiance`, `transposition`,
   `thermal_model` par comparaison directe contre `pvlib.irradiance` /
   `pvlib.temperature` (fonctions équivalentes déjà disponibles dans
   l'oracle utilisé).
5. Si un accès réseau à nrel.gov redevient possible, relecture chiffre
   par chiffre du rapport NREL/TP-560-34302 original contre les tables
   actuelles (vérification indépendante supplémentaire, non bloquante).
6. Recalcul indépendant du résidu de troncature de la série de nutation
   IAU 1980 (63/106 termes), actuellement hérité de la publication source
   sans re-dérivation.
7. Une fois `electrical_model` livré, revalider `ENF-001` (< 5 s pour 8760
   pas) et `ENF-003` (< 3 % vs mesures) sur la chaîne de production
   complète, pas seulement le facteur géométrique solaire déjà validé
   dans cette phase — voir `docs/design/21-matrice-tracabilite.md` Partie B.

## 8. Documents associés

| Document | Contenu |
|---|---|
| `docs/design/16-phase1-validation-report.md` | Historique complet des décisions de la Phase 1 (sessions 1-4), rapport de validation détaillé |
| `docs/design/17-analyse-incertitude-numerique.md` | Analyse d'incertitude numérique complète |
| `docs/design/18-annexe-technique-position-solaire.md` | Annexe technique publiable (algorithmes, équations, démonstrations) |
| `docs/design/19-politique-semver-api.md` | Gel de l'API, politique SemVer, séparation publique/interne/expérimentale |
| `docs/design/21-matrice-tracabilite.md` | Matrice de traçabilité exigence → module → tests → documentation |
| `docs/references/bibliography.{md,bib}` | Bibliographie complète |
| `docs/releases/v1.0.0-scientific-core.md` | Notes de version |
| `validation/golden/` | Jeux de données et rapports de validation figés |
| `.github/workflows/ci.yml` | Intégration continue |
| `engine/validation-data/latest_*.{md,txt}` | Rapports auto-générés (vivants, régénérés à chaque `cargo test`) |

## 9. Déclaration de clôture

Les 7 critères de sortie de la campagne de validation renforcée
(§20.7 de `16-phase1-validation-report.md`) et les 7 actions de clôture de
phase (versionnement, gel d'API, bibliographie, traçabilité, gel des
données, CI, ce rapport) sont satisfaits. **Aucune modification ne sera
apportée au moteur de position solaire sauf correction de bug critique**,
conformément à la consigne reçue. Le développement peut commencer sur le
module suivant, `pv_module`.
