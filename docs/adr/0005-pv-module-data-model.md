# ADR-0005 — Modèle de données et incertitudes de `pv_module`

**Statut** : Proposé (validation attendue avant implémentation)
**Date** : 2026-08-06
**Catégorie** : Nouvelle architecture (modèle de données)

Complète ADR-0004 (architecture, dépendances, découpage en crates). Le
présent ADR traite les décisions **internes au modèle de données** issues
de la spécification `docs/design/25-specification-pv-module.md`.

## Contexte

La spécification détaillée a fait apparaître cinq questions que
l'architecture générale (ADR-0004) ne tranchait pas, et dont chacune est
coûteuse à corriger une fois `electrical_model`, `losses` et les moteurs
suivants construits dessus.

---

## Décision 1 — Taxonomie stricte obligatoire / optionnel / dérivé

### Options considérées

**Option A — Tous les champs optionnels**, validés à l'usage.

- Avantages : n'importe quelle source partielle est importable sans
  friction.
- Inconvénients : un `PvModule` construit pourrait ne contenir aucune
  donnée exploitable ; chaque consommateur devrait revalider les mêmes
  invariants ; l'erreur se manifesterait tard, au moment du calcul, loin
  de sa cause.

**Option B — Noyau obligatoire minimal + reste optionnel**, avec une
règle d'appartenance explicite : *est obligatoire ce sans quoi aucun
modèle électrique ne peut produire de résultat*.

- Avantages : un `PvModule` construit est garanti exploitable ; l'erreur
  est détectée à la construction, au plus près de la source ; les
  consommateurs n'ont pas à revalider.
- Inconvénients : une source très partielle (certains enregistrements de
  base de données) pourrait être rejetée.

### Décision

**Option B**, avec le noyau défini en §3.2 de la spécification :
identité, classification technologique, dimensions L et l, les cinq
grandeurs STC, les trois coefficients de température, et la tension
système maximale.

Critère décisif : un type dont la construction ne garantit rien ne
fournit aucune valeur par rapport à un dictionnaire de `f64`. L'intérêt
d'un modèle de domaine typé est précisément qu'un exemplaire valide
existe *ou n'existe pas*.

Corollaire non négociable : **un champ absent est `None`, jamais une
valeur par défaut.** Un `0.0` silencieux se propagerait comme une donnée
réelle dans toute la chaîne de simulation.

---

## Décision 2 — Les grandeurs dérivées ne sont pas stockées (révision du document 24)

### Contexte spécifique

Le document 24 §2.4 proposait de **stocker** `gross_area` au motif que
certaines fiches affichent une surface qui n'est pas exactement le
produit `longueur × largeur`.

### Options considérées

**Option A — Stocker `length`, `width` **et** `gross_area`** (position
initiale du document 24).

- Avantages : la valeur affichée par le fabricant est conservée telle
  quelle.
- Inconvénients : trois champs dont deux déterminent le troisième. Une
  incohérence devient représentable, et le moteur n'a **aucune règle**
  pour trancher laquelle des deux sources fait foi. C'est exactement la
  situation « deux sources de vérité » que le projet a déjà refusée pour
  la courbe P-V (document 24 §2.11).

**Option B — `gross_area` dérivé, valeur affichée conservée à part**
sous `stated_gross_area`, avec contrôle de cohérence à la construction.

- Avantages : une seule valeur autoritaire ; la valeur affichée reste
  traçable ; un écart devient un **signal de qualité de donnée** exploité
  plutôt qu'une ambiguïté subie.
- Inconvénients : un champ de plus ; un seuil de tolérance à documenter.

### Décision

**Option B**, appliquée identiquement à `gross_area` et à `efficiency`.
Cette décision **révise explicitement** le document 24 §2.4.

Critère décisif : la cohérence avec le traitement déjà retenu pour la
courbe P-V. Refuser deux sources de vérité pour P-V et les accepter pour
la surface aurait été une incohérence de principe.

Sont donc **dérivés, jamais stockés comme autorité** : `gross_area`,
`efficiency`, `fill_factor`, les coefficients de température absolus, la
courbe P-V, la densité de puissance.

---

## Décision 3 — L'incertitude de mesure est un bloc parallèle, non un enrobage générique

### Options considérées

**Option A — `Measured<T> { value: T, uncertainty: Option<Uncertainty> }`**
appliqué à chaque grandeur physique.

- Avantages : uniforme, rigoureux, impossible d'oublier l'incertitude
  d'une grandeur.
- Inconvénients : impose `.value()` sur **tous** les accès, y compris
  dans le cas majoritaire où aucune incertitude n'est connue ; compose
  mal avec les champs déjà optionnels (`Option<Measured<T>>`) ; alourdit
  durablement l'API de tous les consommateurs en aval.

**Option B — Bloc `UncertaintyBudget` optionnel, parallèle aux
grandeurs.**

- Avantages : le chemin courant reste direct ; la complexité n'est payée
  que lorsque l'incertitude existe réellement.
- Inconvénients : le lien grandeur ↔ incertitude est nominal (par nom de
  champ) plutôt que structurel ; un ajout de grandeur exige d'ajouter le
  champ correspondant dans le budget.

### Décision

**Option B.** Critère décisif : l'incertitude n'est disponible que dans
les rapports d'essai accrédités, c'est-à-dire une **minorité** des
sources (une fiche technique commerciale n'en publie pas, une base CEC
non plus). Faire payer sa complexité à chaque champ de chaque module
serait un coût permanent pour un bénéfice occasionnel.

Cadre normatif retenu : **GUM, JCGM 100:2008** — incertitude-type `u`,
incertitude élargie `U = k·u`, facteur d'élargissement `k`, évaluations
de Type A (statistique) et Type B (autre).

---

## Décision 4 — Trois dispersions distinctes, jamais fusionnées

### Contexte spécifique

Trois grandeurs de dispersion coexistent sur une fiche technique et sont
fréquemment confondues dans les outils du secteur :

1. l'**incertitude de mesure** (métrologique, GUM) ;
2. la **tolérance de production** (dispersion entre exemplaires, gérée
   par tri en classes de puissance : « 0/+5 W ») ;
3. la **dégradation** (évolution temporelle).

### Options considérées

**Option A — Une marge d'erreur unique** agrégeant les trois.

- Avantages : simple à présenter à l'utilisateur.
- Inconvénients : scientifiquement faux. Elles ne s'appliquent pas au
  même objet (un exemplaire mesuré / un lot de fabrication / une date),
  ne se combinent pas de la même manière, et n'ont pas la même portée
  décisionnelle. Leur agrégation produirait un nombre non interprétable.

**Option B — Trois champs distincts**, dans trois structures
distinctes.

- Avantages : chaque dispersion conserve sa signification ; un futur
  calcul de propagation peut les traiter correctement.
- Inconvénients : l'utilisateur doit comprendre la distinction.

### Décision

**Option B.** Critère décisif : le principe de priorité à la physique
(`docs/scientific-principles.md` §1) interdit d'agréger des grandeurs de
natures différentes pour la commodité de la présentation.

Conséquence de portée : `PowerTolerance` est un type à **deux
variantes** (`Symmetric` et `Asymmetric`), parce qu'une tolérance
`0/+5 W` n'est pas représentable par un scalaire — et c'est la forme
majoritaire sur les fiches modernes.

Précision de périmètre : `pv_module` **stocke** des incertitudes, il ne
les **propage pas**. La propagation (loi du GUM, ou Monte-Carlo selon
JCGM 101) relèvera de `simulation` ou d'un module dédié.

---

## Décision 5 — Anticiper les tandems 4-terminaux

### Contexte spécifique

Un tandem 2-terminaux (empilement monolithique) présente une seule paire
(Voc, Isc). Un tandem **4-terminaux** (empilement mécanique) présente
**deux sorties électriques indépendantes**, chacune avec sa propre
courbe I-V. Ce second cas existe aujourd'hui en recherche, pas en
production commerciale.

### Options considérées

**Option A — Ignorer le cas** ; le traiter s'il se commercialise.

- Avantages : aucun coût aujourd'hui.
- Inconvénients : l'hypothèse « un module = un couple (Voc, Isc) »
  s'inscrirait dans la signature de `StcParameters` et dans tous les
  consommateurs en aval. La lever plus tard serait une refonte
  traversant `pv_module`, `electrical_model`, `losses` et `simulation`.

**Option B — Champ `additional_outputs: Vec<ElectricalOutput>`**, vide
pour tous les modules commerciaux actuels.

- Avantages : coût nul aujourd'hui (un `Vec` vide) ; supprime un risque
  de refonte à dix ans ; le cas 2-terminaux reste inchangé et direct.
- Inconvénients : un champ que personne n'utilisera avant longtemps.

### Décision

**Option B.** Critère décisif : l'objectif explicite de cette phase est
« représenter n'importe quel module commercial des 30 prochaines années
sans modification d'architecture ». Un `Vec` vide est un prix
négligeable pour éliminer l'un des rares scénarios identifiés qui
forceraient une refonte transverse.

Cette anticipation est **bornée et justifiée**, pas spéculative : le cas
existe physiquement et est documenté dans la littérature. Les cas non
retenus (modules à électronique intégrée, hybrides PV-thermiques) sont
listés en §12 de la spécification comme limites assumées, précisément
parce qu'ils n'ont ni cas d'usage ni donnée de validation aujourd'hui.

---

## Conséquences

**Rendu plus facile**

- Un `PvModule` construit est garanti exploitable par au moins un modèle
  électrique.
- Aucune ambiguïté sur la valeur autoritaire d'une grandeur dérivable.
- L'incertitude de mesure devient exprimable de façon normalisée sans
  alourdir le cas courant.
- Les tandems 4-terminaux ne forceront pas de refonte.

**Rendu plus difficile / plus coûteux**

- Une source très partielle peut être rejetée à la construction
  (Décision 1) — assumé : mieux vaut un rejet explicite qu'un objet
  silencieusement inutilisable.
- Les champs `stated_*` ajoutent de la surface d'API pour un usage de
  recoupement seulement (Décision 2).
- Le lien grandeur ↔ incertitude est nominal, donc à maintenir
  manuellement lors d'un ajout de grandeur (Décision 3).

**Dette introduite**

Aucune dette scientifique nouvelle. Deux points de vigilance consignés :

- La `Warranty` suppose une dégradation **linéaire**, forme sous laquelle
  les garanties sont contractuellement exprimées. Les pérovskites
  présentent des mécanismes de dégradation qualitativement différents.
  Ce n'est pas une dette du modèle : `Warranty` décrit un engagement
  contractuel, pas un modèle physique de vieillissement — lequel relèvera
  d'un futur moteur dédié (spécification §5.1).
- Les numéros de normes cités dans la spécification résultent d'une revue
  documentaire et non d'une lecture du texte normatif intégral,
  inaccessible dans cet environnement. Vérification exigée avant fusion
  (critères 1 et 2 de `docs/governance/scientific-review-process.md`).

**Impact sur le gel de la Phase 1**

- `solar_geometry`, `solar_position`, `thermal_model` : **inchangés**.
- `core_units` : extension additive déjà décidée en ADR-0004
  (Décision 2), à laquelle la spécification §7.3 ajoute `Wavelength`
  pour la réponse spectrale. Toujours strictement additif.

## Références

- `docs/design/25-specification-pv-module.md` — spécification complète.
- `docs/design/24-revue-architecture-pv-module.md` — revue d'architecture
  (§2.4 révisé par la Décision 2 du présent ADR).
- ADR-0004 — architecture, dépendances, découpage.
- `docs/scientific-principles.md` §1 (physique), §3 (transparence),
  §7 (unités SI).
- JCGM 100:2008, *Evaluation of measurement data — Guide to the
  expression of uncertainty in measurement* (GUM).
- JCGM 101:2008, propagation de distributions par méthode de Monte-Carlo
  (hors périmètre `pv_module`, cité pour la propagation future).
- De Soto, W., Klein, S.A., Beckman, W.A. (2006), *Solar Energy* 80(1),
  78-88.
- IEC 60891, 60904-1/-3/-8, 61215, 61730, 61853-1/-2/-3, TS 60904-1-2.
