# ADR-0006 — Porteur de valeur, provenance et gestion formelle des conflits

**Statut** : Proposé (validation attendue avant implémentation)
**Date** : 2026-08-06
**Catégorie** : Nouvelle architecture (modèle de données)

**Remplace la Décision 3 de l'ADR-0005.** L'ADR-0005 reste valide pour
ses décisions 1, 2, 4 et 5 ; sa Décision 3 est explicitement annulée par
la Décision 1 du présent ADR. Conformément à
`docs/governance/adr-policy.md`, l'ADR-0005 n'est pas réécrit : la
décision remplacée reste consultable pour la traçabilité historique.

## Contexte

La finalisation de la spécification physique
(`docs/design/27-specification-finale-pv-module.md`) a introduit deux
exigences qui n'existaient pas lorsque l'ADR-0005 a été rédigé :

1. **Classification épistémique par valeur** (§1) : distinguer donnée
   constructeur, mesurée, normalisée, dérivée, estimée, paramètre
   ajusté, paramètre de modèle, métadonnée.
2. **Provenance par valeur** (§2) : « chaque valeur importée devra
   pouvoir conserver sa provenance ».

S'y ajoute le principe validé explicitement : *une donnée en conflit ne
doit jamais être arbitrée silencieusement.*

---

## Décision 1 — Adopter `Measurement<T>` (renversement d'ADR-0005 D-3)

### Ce que disait la décision remplacée

L'ADR-0005 D-3 avait **rejeté** un enrobage générique
`Measurement<T>` au profit d'un bloc `UncertaintyBudget` parallèle, au
motif que l'incertitude n'est disponible que dans une minorité de
sources et que faire payer sa complexité ergonomique à chaque champ
serait un coût permanent pour un bénéfice occasionnel.

**Ce raisonnement était correct sur ses prémisses.** Il portait sur **un
seul attribut rare** attaché aux valeurs.

### Pourquoi il ne tient plus

Les prémisses ont changé. Il ne s'agit plus d'un attribut mais de
**trois attributs par valeur** : classification, provenance,
incertitude. Or :

- la classification n'est **pas** une propriété du champ mais de la
  valeur (§1.2 : le même `v_oc` est classe A depuis une fiche, B depuis
  un rapport d'essai, E depuis une saisie) ;
- la provenance est **hétérogène au sein d'un même module** (§2.3 : STC
  issu d'une fiche, matrice issue d'un essai, masse estimée).

Trois blocs parallèles indexés par nom de champ seraient fragiles (le
lien champ ↔ attribut n'étant pas vérifiable par le compilateur) et se
désynchroniseraient au premier ajout de champ.

### Options reconsidérées

**Option A — Maintenir trois blocs parallèles**

- Avantages : chemin courant direct, aucune friction ergonomique.
- Inconvénients : aucune garantie de cohérence à la compilation ; un
  champ ajouté sans son entrée dans les trois blocs passe inaperçu ;
  impossible d'exprimer qu'une valeur donnée a telle provenance
  *et* telle classification de façon indissociable.

**Option B — `Measurement<T>` enrobant valeur, classification,
provenance et incertitude**

- Avantages : les quatre informations voyagent ensemble par
  construction ; impossible d'obtenir une valeur sans sa classification ;
  un champ ajouté porte nécessairement ses attributs.
- Inconvénients : `.value()` sur chaque accès ; composition avec
  `Option<T>` donnant `Option<Measurement<T>>`.

### Décision

**Option B.** Critère décisif : lorsqu'un attribut devient
**obligatoire par valeur** (la classification l'est : sans elle, une
estimation est indiscernable d'une mesure), il doit être porté par le
type, non par une table parallèle. Le coût ergonomique est réel et
assumé — c'est le prix de la traçabilité par valeur, désormais une
exigence explicite et non plus une option.

**Portée** : seules les grandeurs physiques issues d'une source externe
sont enrobées. Les métadonnées (classe H), les grandeurs dérivées
(classe D, dont la classification est connue et la provenance héritée)
et les structures de classification restent nues (§10.3).

---

## Décision 2 — Statut variable de la classification

### Options considérées

**Option A — Table statique champ → classe**, définie à la conception.

- Avantages : simple, documentable en un tableau.
- Inconvénients : **faux** pour la majorité des champs électriques. Une
  table figeant `v_oc = donnée constructeur` mentirait dès qu'un rapport
  d'essai IEC 61853 fournit la même grandeur mesurée.

**Option B — Classification portée par la valeur**, dérivée de la
provenance, avec une table pour les seuls champs à classe intrinsèque.

- Avantages : exact ; permet de distinguer une estimation d'une mesure
  pour la même grandeur.
- Inconvénients : la classe n'est pas connue statiquement pour ces
  champs.

### Décision

**Option B**, avec la répartition explicite du §1.3 : classe **fixe**
pour les métadonnées, la configuration, les conditions normalisées, les
dérivées et les paramètres ajustés ; classe **variable** (A/B/E) pour
les grandeurs électriques, géométriques, thermiques et bifaciales.

Critère décisif : une classification fausse serait pire qu'absente —
elle présenterait une estimation comme une mesure, ce que le principe de
transparence interdit précisément.

---

## Décision 3 — Cinq statuts de validation et refus d'arbitrage

### Options considérées

**Option A — Booléen valide/invalide.**

- Avantages : trivial.
- Inconvénients : ne distingue pas « physiquement impossible » de « deux
  sources se contredisent », ni « suspect » de « inévaluable ». Or ces
  cas appellent des réponses opposées : refus définitif pour le premier,
  arbitrage humain pour le second.

**Option B — Cinq statuts** : VALID, WARNING, CONFLICT, INVALID,
UNKNOWN, avec explication obligatoire.

- Avantages : la distinction CONFLICT / INVALID est la clé
  opérationnelle du principe validé. INVALID = rien à arbitrer
  (`v_oc < v_mpp`) ; CONFLICT = **deux valeurs individuellement
  plausibles**, une seule vraie, et le modèle ne sait pas laquelle —
  exactement le cas mesuré sur 29 modules CEC (doc 26 §1).
- Inconvénients : plus de surface d'API ; l'appelant doit traiter cinq
  cas.

### Décision

**Option B.** Le champ `explanation` est **obligatoire** : un statut sans
explication laisserait l'utilisateur devant un refus opaque, ce qui
reviendrait à arbitrer sans le dire.

Le rapport est **attaché au module construit**, pas jeté : un module
portant des WARNING doit rester inspectable par les moteurs en aval.

---

## Décision 4 — Un conflit ne se franchit que par une décision tracée

### Options considérées

**Option A — CONFLICT bloque définitivement.**

- Avantages : garantie maximale.
- Inconvénients : rend inexploitables 0,41 % de la base CEC, y compris
  quand un opérateur sait laquelle des deux valeurs est correcte.

**Option B — CONFLICT bloque, sauf `ConflictResolution` explicite** avec
décideur et justification obligatoires, conservée dans le module.

- Avantages : opérationnalise le principe — le moteur ne choisit
  jamais, un humain ou une politique nommée choisit, et sa décision
  devient elle-même une donnée de provenance auditable.
- Inconvénients : un utilisateur pressé peut résoudre à tort ; mais sa
  décision est alors **tracée et attribuée**, ce qu'un arbitrage
  automatique n'aurait jamais permis.

### Décision

**Option B.** Critère décisif : le principe validé interdit au *moteur*
d'arbitrer, pas à un *humain*. La différence entre les deux est la
traçabilité, et c'est elle que la structure garantit.

---

## Décision 5 — `OperatingCondition` générique avec spécification de température

### Contexte spécifique

Le modèle antérieur (doc 25 §4.6) portait NOCT et NMOT comme deux blocs
nommés. Le document 26 a révélé une troisième condition (PTC) présente
sur les 21 535 modules CEC, et l'instruction en exige d'autres
(IEC 61853, conditions utilisateur).

### Options considérées

**Option A — Un bloc nommé par condition** (`noct`, `nmot`, `ptc`, …).

- Avantages : accès direct par nom.
- Inconvénients : chaque nouvelle condition est une modification de
  structure — exactement ce que l'objectif de stabilité proscrit.

**Option B — `Vec<OperatingPoint>`**, chaque point portant sa condition
complète, avec `kind` `#[non_exhaustive]`.

- Avantages : ajouter une condition est un ajout de variante ; les
  conditions ne peuvent pas être mélangées puisque chaque point porte la
  sienne.
- Inconvénients : accès par recherche plutôt que par champ.

### Décision

**Option B**, avec une exigence physique que l'Option A masquait :
`TemperatureSpecification` distingue `CellTemperature` (STC, IEC 61853-1)
d'`AmbientTemperature` (NOCT, NMOT, PTC).

Critère décisif : les STC imposent la température de **cellule**, NOCT et
PTC imposent l'**ambiante** et laissent la cellule s'établir. Une
abstraction ne distinguant pas ces deux natures serait physiquement
fausse et rendrait ces conditions faussement comparables.

---

## Décision 6 — Cinq surfaces distinctes, aucune identifiée à une autre

### Contexte spécifique

Le document 25 identifiait `gross_area` à `longueur × largeur`. Le
document 26 a montré 29 contre-exemples réels. L'instruction ajoute que
cette conclusion « ne doit pas devenir une règle universelle ».

### Décision

Cinq grandeurs conservées séparément — `overall_dimensions`,
`gross_area`, `active_area`, `aperture_area`, `cell_area` — plus une
dérivée **au nom distinct**, `rectangular_area()`.

Critère décisif : c'est le **nommage** qui empêche la confusion.
`rectangular_area()` n'usurpe pas le nom de `gross_area` ; l'invariant
`INV-GEO-002` oppose donc deux grandeurs explicitement nommées, et non
une valeur à sa propre définition supposée. L'ordre de préférence pour
un calcul de rendement (`aperture` > `gross` > `rectangular`) est
documenté mais **jamais appliqué automatiquement**.

---

## Décision 7 — La technologie est descriptive, jamais prescriptive

### Décision

Aucun code de physique fondamentale ne branche sur `CellTechnology` ou
`CellArchitecture`. Ces types **sélectionnent des paramètres** dans une
bibliothèque ; ils ne modifient pas les équations.

Critère décisif : un branchement physique sur une étiquette
technologique rendrait le cœur du moteur modifiable à chaque nouvelle
technologie, et laisserait les modules `Unknown` sans comportement
défini. Le document 26 §2.1 a établi que **tout import CEC produit
`architecture = Unknown`**, faute d'information dans la source — ce cas
est donc majoritaire, pas marginal.

Contrôle : recherche textuelle en CI d'un `match`/`if` sur ces types dans
les crates de physique (plan de tests T6.4).

---

## Conséquences

**Rendu plus facile**

- Une valeur ne peut pas exister sans sa classification : confondre une
  estimation et une mesure devient impossible par construction.
- Ajouter une condition de référence ou une technologie est additif.
- Un conflit produit un refus explicite nommant les valeurs en cause,
  jamais un résultat faux silencieux.
- La provenance hétérogène au sein d'un module est représentable.

**Rendu plus difficile / plus coûteux**

- `.value()` sur chaque accès à une grandeur physique ;
  `Option<Measurement<T>>` pour les champs optionnels.
- L'appelant traite cinq statuts au lieu d'un booléen.
- Les conditions s'accèdent par recherche dans un `Vec` plutôt que par
  champ nommé.
- 47 invariants à implémenter et tester (≥ 3 cas chacun).

Ces coûts sont assumés : ils sont la contrepartie directe des exigences
de traçabilité et de non-arbitrage.

**Dette introduite**

Aucune dette scientifique. Deux points de vigilance :

- L'ergonomie de `Measurement<T>` sera à réévaluer après l'écriture des
  premiers consommateurs (`electrical_model`). Si elle s'avère
  prohibitive, la réponse sera un ADR, pas un contournement.
- Les seuils numériques des invariants (1 %, 5 %, 2 %) sont issus de
  l'observation de la base CEC (doc 26) et de la pratique des arrondis de
  fiches techniques. Ils devront être confirmés sur les fiches réelles
  (plan de tests T4.9, exigence ouverte).

**Impact sur le gel de la Phase 1**

- `solar_geometry`, `solar_position`, `thermal_model` : **inchangés**.
- `core_units` : extension additive (ADR-0004 D-2, complétée par
  `Wavelength`). Aucun fichier du moteur solaire touché.

## Références

- `docs/design/27-specification-finale-pv-module.md` — spécification
  finale (autorité).
- `docs/design/26-validation-empirique-cec-sandia.md` — mesures fondant
  les décisions 3, 4, 6 et 7.
- `docs/design/28-plan-tests-pv-module.md` — plan de tests.
- ADR-0004 (architecture), ADR-0005 (modèle de données ; **D-3
  remplacée**).
- `docs/scientific-principles.md` §1, §3, §5, §7.
- JCGM 100:2008 (GUM) ; JCGM 101:2008 (propagation, hors périmètre).
- IEC 61215, 61730, 61853-1/-2/-3, 60891, 60904-1/-3/-8,
  TS 60904-1-2.
