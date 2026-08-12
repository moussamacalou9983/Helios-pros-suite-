# Rapport de clôture — Sous-phase `pv_module`

**Date** : 2026-08-11
**Périmètre** : implémentation du crate `pv_module` et extension de `core_units`
**Référence normative** : `docs/design/27-specification-finale-pv-module.md` (commit `5ffcd2e`)
**Statut** : livré — arrêt demandé avant `electrical_model`

---

## 1. Objet

Implémenter le modèle de domaine du module photovoltaïque conformément à
la spécification finale validée, sans en réinventer l'architecture et
sans introduire de logique non spécifiée autrement que par la procédure
prévue (identification, documentation, ADR, traçabilité, tests).

Aucun modèle de calcul n'a été écrit : ni Single Diode, ni De Soto, ni
SAPM, ni Sandia, ni thermique, ni pertes, ni simulation, ni transposition,
ni ombrage, ni modèle bifacial de site.

---

## 2. Ce qui a été livré

### 2.1 Extension de `core_units`

Sept types d'unités ajoutés, plus une méthode manquante :

| Type | Unité SI interne | Motif |
|---|---|---|
| `Voltage` | V | Grandeurs électriques du module |
| `Current` | A | Idem |
| `Power` | W | Idem ; `Voltage × Current → Power` typé |
| `Mass` | kg | Masse du module |
| `Area` | m² | Cinq surfaces distinctes |
| `Wavelength` | m | Réponse spectrale (usage futur) |
| `TemperatureCoefficient` | K⁻¹ | **Porte sa base** : absolue ou relative |
| `Length::from_millimeters` | m | Les fiches publient des millimètres |

`TemperatureCoefficient` est le seul type de la famille à porter un
discriminant. Ce n'est pas une commodité : la bibliothèque CEC publie
`alpha_sc` et `beta_oc` en A/K et V/K mais `gamma_r` en %/K **dans le même
enregistrement**. Un type sans base laisserait deux grandeurs distinctes
de plusieurs ordres de grandeur partager une représentation.

### 2.2 Le crate `pv_module`

13 modules, 3 658 régions de code, zéro dépendance hors `core_units`.

| Module | Rôle |
|---|---|
| `provenance/classification.rs` | Classification épistémique A–H |
| `provenance/source.rs` | `Provenance`, `SourceType`, `ExtractionMethod`, `Confidence` |
| `provenance/measurement.rs` | `Measurement<T>`, `Uncertainty` (GUM) |
| `identity.rs` | `ModuleIdentity`, `Certification` |
| `technology.rs` | `CellTechnology` (récursif), `CellArchitecture` |
| `geometry.rs` | `ModuleDimensions`, cinq surfaces, `MountingForm` |
| `configuration.rs` | `Discrete` / `Monolithic` |
| `electrical.rs` | `ElectricalCharacteristics`, `ElectricalLimits`, `PowerTolerance` |
| `thermal.rs` | Les trois coefficients obligatoires, le quatrième optionnel |
| `condition.rs` | `OperatingCondition` générique, `TemperatureSpecification` |
| `bifacial.rs` | Propriétés intrinsèques seules |
| `validation/` | 5 statuts, 52 invariants, conflits et résolutions |
| `module.rs` | `PvModule`, `PvModuleBuilder` — la porte de validation |
| `adapters.rs` | Interfaces des cinq sources, sans aucun parseur |

---

## 3. Les cinq mécanismes, rendus opérationnels

| Mécanisme | Comment il est devenu du code |
|---|---|
| Classification par valeur | `DataClass` porté par `Measurement<T>`, jamais par le champ |
| Provenance par valeur | `Option<Box<Provenance>>` dans `Measurement<T>` ; champs privés, mise à jour par `with_*` consommant la valeur |
| Conflits jamais arbitrés | `build()` refuse ; seul un `ConflictResolution` nommant décideur **et** justification lève le blocage |
| Physique ≠ modèle | `INV-PRV-003` refuse une classe `Fitted` dans un champ physique |
| Conditions explicites | `TemperatureSpecification` sépare température cellule et ambiante ; `is_comparable_to` interdit la comparaison croisée |

### 3.1 « Une donnée en conflit n'est jamais arbitrée silencieusement »

Le principe est rendu **structurel**, pas seulement documentaire :

- un conflit interrompt la construction et présente les deux candidats,
  leurs sources et l'écart relatif ;
- `ConflictResolution::try_new` renvoie `None` si le décideur ou la
  justification est vide ou n'est faite que d'espaces — une résolution
  anonyme n'est donc pas représentable ;
- une résolution ne lève **que** l'invariant qu'elle nomme ;
- une résolution ne peut **pas** lever un statut `Invalid` : il n'y a pas
  de second candidat entre lesquels choisir ;
- le conflit et sa résolution restent tous deux attachés au module
  construit, qui demeure auditable.

Quatre tests d'intégration couvrent ces cinq propriétés.

---

## 4. Les 52 invariants

Chaque invariant possède un identifiant stable, un niveau, une
explication obligatoire non vide, les champs concernés, et est
exerçable isolément — chaque contrôle est une fonction libre ne recevant
que ce dont elle a besoin.

| Famille | Effectif | Portée |
|---|---|---|
| `INV-POS` | 12 | Positivité et existence |
| `INV-ELE` | 9 | Ordonnancement et cohérence électriques |
| `INV-GEO` | 5 | Surfaces et nidification |
| `INV-DER` | 3 | Rendement dérivé |
| `INV-TMP` | 6 | Coefficients thermiques |
| `INV-CND` | 6 | Conditions de référence |
| `INV-BIF` | 4 | Bifacialité |
| `INV-STR` | 4 | Cohérence structurelle |
| `INV-PRV` | 3 | Provenance |
| **Total** | **52** | |

### 4.1 Écart de décompte — signalé, non dissimulé

La spécification annonçait « 47 invariants » alors que ses propres
tableaux en énuméraient **51**. Il s'agissait d'une erreur d'addition,
non d'une perte de règles : aucun invariant ne manquait. Le chiffre 47
avait ensuite été repris dans le critère de sortie et dans l'instruction
d'implémentation.

L'implémentation a par ailleurs ajouté **`INV-STR-004`** (matériau à
couche mince associé à une configuration discrète, niveau WARNING),
portant le total à 52.

Les deux points sont traités par **ADR-0007**, qui fait de
`ALL_INVARIANT_IDS` l'inventaire de référence : un décompte inscrit en
prose se désynchronise, une constante vérifiée par test ne le peut pas.
Deux tests contrôlent l'inventaire — unicité et bonne forme des
identifiants d'une part, effectif attendu de chacune des huit familles
avec somme égale à la longueur du tableau d'autre part.

### 4.2 Seuils

Aucun seuil n'est un nombre nu. Chacun est une constante nommée,
documentée et publique, tirée soit de la pratique d'arrondi des fiches
constructeurs, soit de la mesure directe de la bibliothèque CEC
(`docs/design/26-validation-empirique-cec-sandia.md`). Un bloc `const`
vérifie **à la compilation** qu'aucun seuil d'avertissement ne dépasse
son seuil de conflit, ce qui rendrait l'avertissement inatteignable.

---

## 5. Adaptateurs — interfaces seules

`ADAPTER_CATALOGUE` nomme les cinq sources avec leur statut. Aucun
parseur, aucun format de fichier, aucune cartographie de champ n'a été
écrit.

| Adaptateur | Statut |
|---|---|
| `CECAdapter` | Spécifié |
| `SandiaAdapter` | Spécifié |
| `IEC61853Adapter` | En attente de données de référence |
| `ManufacturerDatasheetAdapter` | En attente de données de référence |
| `PANAdapter` | **Bloqué** — statut juridique du format à établir + ADR dédié |

Le sens de dépendance est unique : un adaptateur connaît sa source **et**
le modèle de domaine ; le modèle de domaine ne connaît ni l'un ni
l'autre. Le type `Record` est associé au trait, jamais au `PvModule`.

Aucune dépendance à pvlib, à PVsyst ou à toute autre suite. Aucun accès
réseau, aucun scraping : les sources sont des fichiers ou des saisies,
remis à l'adaptateur par l'appelant.

---

## 6. Tests

| Cible | Tests | Objet |
|---|---|---|
| `pv_module` (unitaires) | 88 | Comportement de chaque type |
| `tests/invariants.rs` | 65 | **207 scénarios** répartis sur 52 invariants, contrôles appelés isolément |
| `tests/integration.rs` | 23 | Nominal, unités, provenance, absence, invalide, conflits, enregistrement de forme CEC |
| `tests/api_surface.rs` | 8 | Points d'entrée publics non atteints par les précédents |
| **Total `pv_module`** | **184** | |
| **Workspace** | **364** | Aucune régression sur les 180 tests antérieurs |

### 6.0 Le décompte des scénarios est mesuré, pas affirmé

Compter les fonctions `#[test]` ne dit rien de la profondeur d'un
contrôle : une fonction peut vérifier trois cas ou trente, et aucun outil
ne peut le savoir de l'extérieur. Les scénarios sont donc devenus des
**données** : chacun nomme son invariant, décrit sa situation et déclare
le résultat attendu **avant** exécution.

Trois propriétés en découlent :

1. **Le décompte est une mesure.** La matrice
   `engine/validation-data/invariant-scenario-matrix.md` est produite par
   la suite elle-même et donne, par invariant, le nombre de scénarios
   réellement joués, ainsi que PASS et FAIL.
2. **Le seuil ne peut pas régresser en silence.**
   `every_invariant_carries_at_least_three_scenarios` échoue si un seul
   invariant descend sous trois scénarios, et la CI exécute ce test.
3. **Chaque invariant reste exécutable seul.** Un `#[test]` par invariant
   est engendré, donc `cargo test inv_geo_002` joue exactement les
   scénarios de cet invariant.

Un scénario attendant le **silence** compte autant qu'un scénario
attendant une remarque : un invariant qui se déclenche à tort est aussi
défectueux qu'un invariant muet à tort. Sur les 207 scénarios, **106**
attendent le silence et **101** une remarque graduée (57 INVALID,
39 WARNING, 4 CONFLICT, 1 INFO).

Le harnais lui-même est testé (six tests dédiés) : un compteur incapable
de signaler un déficit, ou un juge qui accepte tout, transformerait le
contrôle en décoration. Ces tests vérifient qu'un invariant à deux
scénarios est bien détecté, qu'une remarque sans explication ou sans
champ nommé est refusée, et qu'un scénario défaillant est compté FAIL et
non PASS.

**Résultat mesuré** : 52 invariants, **207 scénarios**, **207 PASS**,
**0 FAIL**, **0 invariant sous le seuil**.

| Scénarios par invariant | Invariants |
|---:|---:|
| 3 (le minimum requis) | 19 |
| 4 | 20 |
| 5 | 8 |
| 6 | 5 |
| **Total** | **52** |

Aucun invariant n'a donc dû être complété : le seuil était déjà tenu
partout, ce que cette vérification établit désormais par mesure plutôt
que par affirmation.

### 6.1 Couverture

Mesurée par `cargo llvm-cov`, non estimée :

| Métrique | Valeur |
|---|---|
| Régions | **99,51 %** |
| Lignes | **99,61 %** |
| Fonctions | **100,00 %** |

Exigence : ≥ 95 %. Les rares régions non couvertes sont des bras
`panic!` de tests (inatteignables quand les tests passent) et une branche
défensive de `PvModule::efficiency` qu'une construction validée rend
inaccessible — elle est documentée comme telle plutôt que retirée, pour
que la fonction reste totale.

La couverture n'est pas artificielle : les branches de validation,
d'erreur et de conflit sont exercées par des cas qui vérifient le statut
produit, l'explication, les champs nommés et les valeurs observées, non
par de simples appels.

---

## 7. Qualité

| Contrôle | Résultat |
|---|---|
| `cargo fmt --all -- --check` | Propre |
| `cargo clippy --workspace --all-targets -- -D warnings` | **0 avertissement** |
| `cargo test --workspace` | 355 réussis, 0 échec |
| `cargo doc --workspace --no-deps` | Propre, `#![warn(missing_docs)]` actif |
| `#![forbid(unsafe_code)]` | Actif sur `pv_module` |
| Régressions | Aucune |

---

## 8. Décisions prises pendant l'implémentation

Trois écarts au plan initial, chacun documenté.

### 8.1 `INV-STR-004` ajouté — ADR-0007

Voir §4.1. Procédure complète appliquée : besoin identifié, décision
documentée, ADR créé, traçabilité mise à jour (PVM-023), quatre cas de
test ajoutés.

### 8.2 `Provenance` mise en boîte dans `Measurement<T>`

`Provenance` compte neuf champs, dont six chaînes allouées, soit environ
230 octets. Inlinée, elle porterait chaque `Measurement<T>` à près de
300 octets quel que soit `T`, pour une vingtaine de valeurs par module.
La boîte coûte une indirection sur un champ lu à l'audit et jamais dans
une boucle de calcul, et reste invisible à l'API puisque l'accesseur
rendait déjà une référence. Un test garde la décision : il échoue si
`Measurement<Voltage>` repasse au-dessus de 128 octets, **ou** si
`Provenance` redevenait assez petite pour que la boîte soit injustifiée.

### 8.3 Blocs différés

Les blocs esquissés au §13.1 de la spécification pour la réponse
angulaire, la réponse spectrale, les limites mécaniques, la garantie et
la construction **ne sont pas implémentés**. Chacun appelle sa propre
norme de mesure et ses propres invariants, et aucun n'est requis par le
critère de sortie du modèle de domaine. La matrice de performance
IEC 61853-1, en revanche, n'a pas besoin de structure propre : un point
de matrice **est** un point de fonctionnement mesuré à température de
cellule imposée, et il est porté comme tel.

---

## 9. Points ouverts — inchangés et non simulés

| Point | Suivi |
|---|---|
| Cinq fiches constructeurs réelles | Non fournies, **non inventées**. Les fixtures de test sont explicitement construites, documentées comme telles, et ne prétendent à aucune origine. |
| Grille exacte IEC 61853-1 | À lire dans le texte normatif. Les irradiances et températures employées en test sont des valeurs de substitution signalées. |
| `PANAdapter` | Bloqué, statut inchangé |

Fabriquer une fiche constructeur reviendrait à présenter une estimation
sous l'autorité d'une donnée constructeur — précisément la confusion que
la classification A–H existe pour empêcher.

---

## 10. Critère de fin

| Exigence | État |
|---|---|
| Spécification implémentée | Oui |
| Invariants implémentés | 52, chacun identifié, gradué, expliqué, testable isolément, tracé |
| Tests complets | 175, ≥ 3 cas par invariant, intégration incluse |
| Provenance opérationnelle | Oui, par valeur, immuable hors mécanisme explicite |
| Gestion des conflits opérationnelle | Oui, structurellement non contournable |
| API documentée | Oui, `missing_docs` actif |
| Couverture ≥ 95 % | 99,51 % |
| Zéro avertissement | Oui, `clippy -D warnings` |
| Zéro régression | Oui, 355 tests |
| `fmt` / `clippy` / `doc` propres | Oui |
| Rapport de clôture | Ce document |
| Traçabilité mise à jour | doc 21, Partie C, 23 exigences |
| ADR mis à jour | ADR-0007 créé, index mis à jour |

---

## 11. Gel de `pv_module`

**`pv_module` est gelé au 11 août 2026**, après validation explicite de
la sous-phase et vérification de traçabilité des scénarios.

| Modification | Autorisée ? |
|---|---|
| Correction d'un bug critique | Oui, avec ADR si la correction change un comportement observable |
| Ajout de logique de calcul | **Non** |
| Ajout d'un modèle électrique | **Non** |
| Ajout d'un modèle thermique | **Non** |
| Ajout d'un invariant | Seulement par la procédure ADR (voir ADR-0007) |

`pv_module` reste le **modèle de domaine du composant PV** : il décrit ce
qu'est le module. Tout ce qui relève de son comportement appartient à
`electrical_model` et aux moteurs suivants.

Le gel est protégé par la CI : le seuil de trois scénarios par invariant,
l'inventaire des 52 identifiants et la couverture ≥ 95 % échouent le
build si une modification les entame.

---

## 12. Suite

**Arrêt demandé.** `electrical_model` n'est pas entamé et ne le sera
qu'après validation explicite de la présente sous-phase.

Ce que `pv_module` met à disposition de son successeur : un module
validé, dont chaque grandeur porte sa classe, sa provenance, son
incertitude éventuelle, et dont les contradictions ont été soit écartées,
soit explicitement décidées et consignées. Ce que `pv_module` ne fait
pas, et ne doit jamais faire : extraire des paramètres, évaluer une
courbe I-V, chercher un point de puissance maximale.
