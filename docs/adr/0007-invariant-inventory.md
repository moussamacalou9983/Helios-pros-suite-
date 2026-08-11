# ADR-0007 — Inventaire des invariants : décompte de référence et ajout d'`INV-STR-004`

**Statut** : Proposé
**Date** : 2026-08-11
**Catégorie** : Règles de validation (complément à l'ADR-0004 et à l'ADR-0006)

Ne remplace aucune décision antérieure. Corrige une erreur de décompte
dans `docs/design/27-specification-finale-pv-module.md` §11 et enregistre
un invariant introduit pendant l'implémentation.

---

## Contexte

L'implémentation de `pv_module` a fait apparaître deux écarts entre la
spécification et le code. La règle fondamentale posée au moment du feu
vert impose que toute logique non spécifiée soit **identifiée,
documentée, portée par un ADR, tracée et testée** plutôt qu'introduite
silencieusement. Le présent ADR exécute cette procédure.

### Écart 1 — Le décompte annoncé ne correspondait pas aux tableaux

La section §11 de la spécification finale annonçait « **47** invariants
nommés ». Le dénombrement direct de ses propres tableaux en donne
**51** :

| Famille | Effectif |
|---|---|
| `INV-POS` | 12 |
| `INV-ELE` | 9 |
| `INV-GEO` | 5 |
| `INV-DER` | 3 |
| `INV-TMP` | 6 |
| `INV-CND` | 6 |
| `INV-BIF` | 4 |
| `INV-STR` | 3 |
| `INV-PRV` | 3 |
| **Total** | **51** |

**Il s'agissait d'une erreur d'addition, non d'une omission de règles.**
Aucun invariant n'avait été perdu ; le texte introductif comptait mal les
lignes qu'il présentait lui-même. Le chiffre 47 avait ensuite été repris
tel quel dans le critère de sortie (§15) et dans l'instruction de mise en
œuvre, propageant l'erreur sans jamais affecter le contenu normatif.

### Écart 2 — Un invariant manquait pour les modules à couche mince

La spécification distingue explicitement deux configurations physiques
(§4) : `Discrete`, où des cellules dénombrables sont connectées entre
elles, et `Monolithic`, où l'empilement est scribé en segments sans
cellule individualisable. Cette distinction existe précisément parce que
les technologies à couche mince (CdTe, CIGS, silicium amorphe) ne
comportent pas de cellules discrètes.

Or aucun invariant ne vérifiait la cohérence entre les deux axes. Un
enregistrement décrivant un module CdTe avec « 72 cellules en série »
était acceptable sans la moindre remarque, alors que la combinaison ne
correspond à aucune construction industrielle connue et signale presque
toujours soit une erreur de saisie du matériau, soit un comptage de
segments présenté comme un comptage de cellules.

Le besoin est donc réel et relève bien de la cohérence structurelle,
famille `INV-STR`, dont les trois membres existants portent déjà sur des
incohérences de topologie.

---

## Décision 1 — L'inventaire de référence est la constante du code

`pv_module::validation::invariants::ALL_INVARIANT_IDS` devient
**l'inventaire faisant foi**. La spécification décrit les invariants ;
la constante les dénombre.

### Pourquoi le code plutôt que le document

Un décompte inscrit en prose se désynchronise dès la première addition,
comme cet ADR vient de le constater. Une constante :

- est vérifiable mécaniquement — le test
  `the_invariant_inventory_is_complete_and_free_of_duplicates` contrôle
  l'absence de doublon et la bonne forme de chaque identifiant ;
- est vérifiable **par famille** — le test
  `every_family_is_represented_in_the_inventory` fixe l'effectif attendu
  de chacune des huit familles et exige que leur somme soit la longueur
  du tableau, de sorte qu'un ajout non déclaré échoue au test plutôt que
  de passer inaperçu ;
- ne peut pas diverger du code qui l'utilise, `pv_module::is_known_invariant`
  y renvoyant directement.

La spécification reste la source normative de **ce que chaque invariant
signifie**. Elle cesse d'être la source du **nombre**.

### Conséquence

Le total est porté à **52** dans §11 et §15 de la spécification, avec une
note de correction datée conservant la trace de l'erreur. Conformément à
`docs/governance/adr-policy.md`, le texte fautif n'est pas effacé : il
est corrigé et la correction est signalée.

---

## Décision 2 — Ajouter `INV-STR-004`

> **`INV-STR-004`** — un matériau à couche mince associé à une
> configuration `Discrete` est signalé.
>
> **Niveau : WARNING.**

### Énoncé précis

L'invariant se déclenche lorsque `ModuleTechnology::is_thin_film()` est
vrai — c'est-à-dire pour `CadmiumTelluride`,
`CopperIndiumGalliumSelenide` et `AmorphousSilicon` — et que la
configuration n'est pas monolithique.

### Pourquoi WARNING et non ERROR

Trois raisons, dans l'ordre de leur poids.

1. **Ce n'est pas une impossibilité physique.** Rien n'interdit de
   découper un stratifié à couche mince puis d'assembler les morceaux.
   Aucun produit industriel connu ne le fait, mais une règle de format ne
   doit pas se prononcer sur ce qui est fabricable.
2. **La technologie est descriptive, jamais prescriptive** (§7 de la
   spécification). Refuser une construction sur la base d'une étiquette
   commerciale reviendrait à faire brancher le moteur sur cette étiquette,
   ce que la spécification interdit explicitement. Signaler n'est pas
   brancher.
3. **Les sources réelles sont pauvres.** La bibliothèque CEC ne porte
   aucune information d'architecture et range CdTe et CIGS sous une
   catégorie `"Thin Film"` unique
   (`docs/design/26-validation-empirique-cec-sandia.md` §2.1). Un import
   de cette base produirait des refus en masse là où l'information
   manquante est celle de la base, pas celle du module.

L'invariant est donc un **indicateur de qualité de donnée**, du même
registre que `INV-GEO-004`, et il est explicitement documenté comme tel
dans le code.

### Direction unique

L'invariant ne se déclenche que dans le sens couche mince → discret. Un
module au silicium déclaré monolithique n'est pas signalé par
`INV-STR-004`, car `is_thin_film()` est faux : la relation contrôlée est
« un matériau à couche mince est normalement scribé », pas « seule une
couche mince peut être monolithique ». Le test
`inv_str_004_thin_film_materials_pair_with_monolithic_configurations`
vérifie explicitement cette asymétrie.

---

## Conséquences

### Positives

- Le décompte cesse d'être une affirmation invérifiable.
- Une incohérence structurelle réelle, jusqu'ici invisible, est signalée.
- La procédure d'introduction d'une règle non spécifiée est exercée pour
  la première fois et se révèle praticable.

### Négatives assumées

- Un import CEC d'un module à couche mince dont la configuration a été
  saisie en cellules discrètes produira un avertissement supplémentaire.
  C'est l'effet recherché ; l'avertissement ne bloque pas.
- La spécification et le code portent désormais deux totaux dans leur
  historique (47 annoncé, 51 réel, 52 après ajout). La note de correction
  datée rend cette histoire lisible plutôt que de la masquer.

### Traçabilité

| Élément | Emplacement |
|---|---|
| Énoncé normatif | `docs/design/27-specification-finale-pv-module.md` §11.7 |
| Implémentation | `engine/crates/pv_module/src/validation/invariants.rs`, `check_structure` |
| Inventaire | `ALL_INVARIANT_IDS` (52 entrées) |
| Tests | `tests/invariants.rs` : `inv_str_004_…` (4 cas), `the_invariant_inventory_…`, `every_family_is_represented_…` |
| Matrice | `docs/design/21-matrice-tracabilite.md`, Partie C, PVM-023 |
