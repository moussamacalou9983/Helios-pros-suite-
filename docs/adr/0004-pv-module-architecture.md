# ADR-0004 — Architecture du crate `pv_module`

**Statut** : Proposé (validation attendue avant implémentation)
**Date** : 2026-08-06
**Catégorie** : Nouvelle architecture (+ Modification d'API publique, Décision 2)

## Contexte

`pv_module` sera la fondation du périmètre électrique de HeliosPro :
`electrical_model`, `losses`, puis `bifacial`, `shading`, `tracking` en
dépendront tous. Le crate existe aujourd'hui comme squelette documenté
sans logique (Phase 1). La revue d'architecture complète
(`docs/design/24-revue-architecture-pv-module.md`) a fait émerger cinq
décisions structurantes qui doivent être tranchées **avant** toute
implémentation, parce qu'aucune n'est réversible à faible coût une fois
que trois crates en aval en dépendront.

## Décision 1 — `pv_module` représente, ne calcule pas

### Options considérées

**Option A — Crate « module PV » complet** : données de fiche technique
*et* évaluation de performance (courbe I-V, MPP) dans le même crate.

- Avantages : un seul point d'entrée pour « tout ce qui concerne un
  module » ; pas de découpage à expliquer.
- Inconvénients : mélange une description d'objet physique et un modèle
  de simulation ; impose de choisir *un* modèle électrique de référence
  dans le crate de données ; rend impossible la coexistence propre de
  plusieurs modèles électriques (De Soto, SAPM) sans que le crate de
  données connaisse chacun d'eux.

**Option B — Séparation données / physique** : `pv_module` = modèle de
domaine pur ; `electrical_model` = physique.

- Avantages : reproduit la discipline déjà établie (`core_units`,
  `core_constants` ne calculent rien) ; permet plusieurs modèles
  électriques concurrents consommant la même description ; testable sans
  aucune dépendance physique.
- Inconvénients : deux crates à traverser pour un cas d'usage complet.

### Décision

**Option B.** Critère décisif : le patron `SolarPositionAlgorithm`
d'ADR-0001 a démontré la valeur de plusieurs modèles indépendants sur une
même donnée d'entrée. Reproduire ce patron exige que la donnée d'entrée
soit indépendante de tout modèle. Test opérationnel de la frontière : si
une fonction requiert une irradiance ou une température *d'exploitation*,
elle n'appartient pas à `pv_module`.

---

## Décision 2 — Étendre `core_units` avec les grandeurs électriques

### Contexte spécifique

`core_units` n'expose aujourd'hui aucun type de tension, courant,
puissance, masse ou surface (constat C-1 de la revue). Or `core_units`
fait partie du périmètre **gelé** au tag `v1.0.0-scientific-core`.

### Options considérées

**Option A — `f64` nus dans `pv_module`** pour Voc, Isc, Pmax, masse…

- Avantages : aucune modification d'un crate gelé.
- Inconvénients : viole frontalement le principe « unités SI »
  (`docs/scientific-principles.md` §7) sur le crate qui sera la fondation
  de tout le périmètre électrique. Inacceptable.

**Option B — Types électriques locaux à `pv_module`**

- Avantages : `core_units` reste intact.
- Inconvénients : `electrical_model`, `losses`, et tout futur crate
  électrique devraient soit dépendre de `pv_module` pour leurs unités
  (dépendance absurde : un modèle de pertes câblage n'a pas besoin d'une
  fiche technique de module), soit redéfinir leurs propres types →
  prolifération de types incompatibles pour la même grandeur physique.
  C'est exactement le défaut que `core_units` existe pour empêcher.

**Option C — Étendre `core_units`** de `Voltage`, `Current`, `Power`,
`Mass`, `Area`, `TemperatureCoefficient`.

- Avantages : un seul type par grandeur physique dans tout le workspace ;
  conforme au principe §7 ; réutilisable par tous les crates
  électriques futurs.
- Inconvénients : modifie un crate gelé.

### Décision

**Option C.** Critère décisif : le gel de la Phase 1 interdit les
changements **cassants**, pas les ajouts. Ajouter un type public est
strictement additif au sens de
`docs/design/19-politique-semver-api.md` §19.3.2 — aucune signature
existante n'est touchée, aucun code appelant existant ne casse. Cela
justifie un incrément **MINOR** (0.1.0 → 0.2.0) et non un PATCH.

Note sur `TemperatureCoefficient` : un type dédié est nécessaire parce
que `Ratio` est borné à `[0, 1]` (constat C-2) et que β\_Voc et γ\_Pmax
sont négatifs. Ce type est **signé et non borné**.

Note sur le gel : cette extension ne touche **aucun** fichier du moteur
de position solaire lui-même (`solar_geometry`, `solar_position`), qui
reste strictement inchangé conformément à l'engagement de clôture de
Phase 1.

---

## Décision 3 — `#[non_exhaustive]` sur les énumérations de classification

### Contexte spécifique

`docs/design/19-politique-semver-api.md` §19.4 constate qu'aucun type du
workspace n'est `#[non_exhaustive]`, et que cette décision serait
réexaminée avant `1.0.0`. Cette ADR la réexamine pour un cas précis.

### Options considérées

**Option A — Énumérations exhaustives** (cohérence avec l'existant)

- Avantages : le code appelant peut faire un `match` exhaustif sans bras
  `_`, ce qui force à traiter explicitement chaque cas.
- Inconvénients : l'industrialisation de la tandem pérovskite/silicium
  est en cours ; chaque nouvelle technologie deviendrait un changement
  cassant pour tout code appelant. Sur une liste dont on **sait** qu'elle
  s'allongera, c'est garantir des ruptures répétées.

**Option B — `#[non_exhaustive]` + variante `Other { name: String }`**

- Avantages : ajouter une technologie devient additif ; la variante
  `Other` permet de représenter fidèlement un module réel dont la
  technologie n'est pas encore modélisée, plutôt que de le rejeter ou de
  le classer faussement.
- Inconvénients : le code appelant doit prévoir un bras `_` ; perte de
  la vérification exhaustive à la compilation.

### Décision

**Option B**, restreinte aux **énumérations de classification**
(`CellTechnology`, `CellArchitecture`, `Certification`) — pas aux
structures de données numériques, qui restent exhaustives.

Critère décisif : la distinction entre une énumération **fermée par la
physique** (il n'y a que deux moteurs de position solaire parce qu'on a
décidé d'en implémenter deux) et une énumération **ouverte par le
monde réel** (la liste des technologies PV commercialisées ne nous
appartient pas). Les secondes doivent être `#[non_exhaustive]`.

Cette décision **précise** §19.4 sans la contredire : elle constitue le
réexamen que §19.4 annonçait, appliqué à un cas où la justification est
établie.

---

## Décision 4 — Les coefficients de modèle ajusté n'appartiennent pas à `pv_module`

### Contexte spécifique

Les bases CEC et Sandia distribuent, en plus des grandeurs mesurées, des
**paramètres de modèle ajustés par régression** : cinq paramètres De Soto
pour CEC, coefficients SAPM pour Sandia. Ces jeux ne sont pas
inter-convertibles sans perte.

### Options considérées

**Option A — Les stocker dans `pv_module`** comme blocs optionnels.

- Avantages : un import CEC produit un seul objet ; « tout ce qu'on sait
  du module » est au même endroit.
- Inconvénients : rend la description d'un objet physique dépendante du
  choix d'un modèle de simulation ; `pv_module` devrait connaître la
  forme de chaque modèle électrique futur ; confond des grandeurs
  mesurées et des paramètres de régression.

**Option B — Les placer avec le modèle qui les interprète**
(`electrical_model`).

- Avantages : `pv_module` reste une description physique pure ; chaque
  modèle possède ses propres paramètres ; ajouter un modèle électrique
  ne touche pas `pv_module`.
- Inconvénients : un import CEC produit un couple
  `(PvModule, Option<DeSotoParameters>)` et traverse donc deux crates.

### Décision

**Option B.** Critère décisif : des paramètres De Soto sont dépourvus de
sens hors du modèle De Soto — ce ne sont pas des propriétés du module,
ce sont des propriétés d'un *ajustement*. Le principe de priorité à la
physique (`docs/scientific-principles.md` §1) impose de ne pas présenter
un résultat de régression comme une caractéristique physique mesurée.

L'inconvénient (import traversant deux crates) est **assumé et
documenté** plutôt que masqué — voir §4.3 de la revue.

---

## Décision 5 — La couche d'import vit hors de `pv_module`

### Options considérées

**Option A — Parsers dans `pv_module`** (modules `cec`, `sandia`, `csv`…).

- Avantages : import « clé en main » depuis le crate de données.
- Inconvénients : introduit de l'I/O dans un crate qui doit rester pur
  (principe de déterminisme §6) ; chaque nouveau format modifierait
  `pv_module`, ce qui contredit frontalement l'exigence « importer sans
  modifier le modèle interne » ; les parsers JSON/CSV robustes pousseront
  à ajouter des dépendances externes, contaminant le moteur (ADR-0002).

**Option B — Crate d'adaptation séparé** (`pv_module_io`) dépendant de
`pv_module`.

- Avantages : `pv_module` ignore jusqu'à l'existence des formats —
  l'exigence est satisfaite littéralement, pas approximativement ;
  ajouter un format = zéro ligne modifiée dans `pv_module` ; une
  éventuelle dérogation à la politique zéro-dépendance pour un parseur
  reste confinée hors du moteur scientifique.
- Inconvénients : un crate de plus ; l'utilisateur doit connaître deux
  crates pour importer un module.

### Décision

**Option B.** Critère décisif : l'exigence était « prévoir l'import futur
de CEC, Sandia, PAN, CSV, JSON, IEC 61853 **sans modifier le modèle
interne** ». Seule l'inversion de dépendance (l'adaptateur connaît le
modèle, jamais l'inverse) garantit cette propriété de façon structurelle
plutôt que par discipline.

Le format **PAN** fait exception : propriétaire PVsyst, non spécifié
publiquement à notre connaissance. Aucune implémentation ne sera
entreprise avant vérification juridique et ADR dédié — cohérent avec la
règle fondatrice interdisant toute reprise de PVsyst / PV\*SOL /
Archelios Calc.

---

## Conséquences

**Rendu plus facile**

- Plusieurs modèles électriques (De Soto, SAPM, futurs) peuvent coexister
  sur la même description de module.
- Ajouter une technologie de cellule ou un format d'import devient
  additif.
- `pv_module` est testable sans aucune dépendance physique ni I/O.
- Tout le périmètre électrique futur dispose de types SI partagés.

**Rendu plus difficile / plus coûteux**

- Un import CEC traverse deux crates (Décision 4) — assumé.
- Un `match` sur `CellTechnology` exige un bras `_` (Décision 3) —
  assumé.
- `core_units` doit être étendu avant que `pv_module` puisse être écrit :
  cela crée une **dépendance d'ordre d'implémentation** (`core_units`
  d'abord, `pv_module` ensuite) à respecter.

**Dette introduite**

- Aucune dette scientifique nouvelle. Une dette **préexistante** est
  documentée à cette occasion : `thermal_model::noct::cell_temperature`
  accepte le NOCT en `f64` nu (constat C-3), proposée comme DEBT-015 à
  inscrire au registre au moment de l'implémentation.

**Impact sur le gel de la Phase 1**

- `solar_geometry` et `solar_position` : **strictement inchangés**.
- `core_units` : extension additive (Décision 2), incrément MINOR.
- `thermal_model` : inchangé ; l'écart C-3 est documenté, pas corrigé.

## Références

- `docs/design/24-revue-architecture-pv-module.md` — revue complète.
- `docs/design/23-plan-pv-module.md` — plan général validé.
- `docs/design/19-politique-semver-api.md` §19.3.2 (additivité), §19.4
  (`#[non_exhaustive]`), §19.6 (gel).
- `docs/scientific-principles.md` §1 (physique), §6 (déterminisme),
  §7 (unités SI).
- ADR-0001 (patron multi-moteurs), ADR-0002 (zéro dépendance).
- De Soto, W., Klein, S.A., Beckman, W.A. (2006), *Solar Energy* 80(1),
  78-88 — définit les champs de fiche technique requis.
- King, D.L. et al. (2004), SAND2004-3535 — modèle SAPM.
- IEC 61215, 61730, 61853-1/2, 60904-1/-3, 60891, TS 60904-1-2.
