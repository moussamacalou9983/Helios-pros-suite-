# ADR-0008 — Architecture d'`electrical_model` : multi-modèles, et séparation points / courbe

**Statut** : Proposé (conception ; implémentation soumise à validation explicite)
**Date** : 2026-08-11
**Catégorie** : Nouvelle architecture
**Référence** : `docs/design/30-specification-electrical-model.md`

Ne remplace aucune décision antérieure. Prolonge l'ADR-0004 Décision 4
(séparation données physiques / paramètres de modèle), qui avait été
prise avant que le contenu réel des bases CEC et Sandia ne soit inspecté,
et que la doc 26 a depuis confirmée sur pièces.

---

## Contexte

`pv_module` est gelé et décrit ce qu'est un module. `electrical_model`
doit décrire comment il se comporte. Trois faits contraignent
l'architecture avant tout choix esthétique.

1. **Les sources ne permettent pas d'initialiser les mêmes modèles.** Un
   enregistrement CEC porte les six paramètres De Soto/CEC ; un
   enregistrement Sandia porte les 28 coefficients SAPM ; une fiche
   constructeur ne porte ni les uns ni les autres. Constaté par
   inspection directe (doc 26 §2.6, §3).
2. **Le SAPM ne produit pas de courbe continue**, seulement cinq points
   remarquables.
3. **L'équation à diode unique est implicite**, donc tout accès à un point
   de la courbe passe par une résolution numérique dont la terminaison
   doit être garantie, pas espérée.

---

## Décision 1 — Architecture multi-modèles, sélection explicite

Le crate portera **plusieurs modèles coexistants**, et la sélection sera
toujours explicite.

### Pourquoi pas un modèle unique

Le réflexe serait de choisir le meilleur modèle et de tout y ramener. Il
échoue sur le fait 1 : un modèle qu'on ne peut pas initialiser depuis la
source dont on dispose est inutilisable, quelles que soient ses qualités.
Imposer De Soto à un enregistrement Sandia obligerait à ré-identifier ses
paramètres depuis le seul point STC, en jetant 28 coefficients mesurés —
une perte d'information déguisée en simplification.

### Ce qui est décidé

- Un type énuméré nomme le modèle, et les paramètres portent cette
  identité de façon inséparable.
- Le moteur répond à la question « **quels modèles cette source
  permet-elle d'initialiser ?** » et **refuse** les autres en nommant les
  paramètres manquants.
- Aucun paramètre absent n'est remplacé par une valeur par défaut. Un
  `Adjust` mis à zéro parce qu'il manque transformerait silencieusement un
  modèle CEC en modèle De Soto sous une étiquette fausse.

### Conséquence sur le choix par défaut

Lorsque plusieurs modèles sont applicables, **le moteur ne choisit pas
tout seul** (décision D-3, à finaliser). L'option de refuser de choisir
est cohérente avec la politique de conflits déjà adoptée : là où deux
options sont défendables et où rien dans les données ne tranche, le
moteur présente et n'arbitre pas.

---

## Décision 2 — Séparer les modèles à points des modèles à courbe

Deux abstractions distinctes :

- **modèle à points remarquables** — rend `I_sc`, `V_oc`, `I_mp`, `V_mp`,
  `P_max` ;
- **modèle à courbe continue** — rend `I(V)` pour tout `V` admissible.

Le diode unique implémente les deux. **Le SAPM n'implémente que le
premier.**

### Pourquoi c'est une décision d'architecture et non un détail

Une interface unique « donne-moi `I` à n'importe quel `V` » obligerait le
SAPM à interpoler entre ses cinq points et **présenterait l'interpolation
comme le modèle**. L'utilisateur recevrait une valeur d'apparence
identique à celle d'un modèle physique, sans moyen de savoir qu'elle a
été inventée entre deux points.

C'est exactement la faute que la classification A–H empêche en amont :
présenter une valeur dérivée avec l'autorité d'une valeur mesurée.

En séparant les deux abstractions, **le compilateur** interdit de
demander une courbe au SAPM. La contrainte n'est pas documentaire : elle
est structurelle, et le plan de tests prévoit des tests `compile_fail`
qui le vérifient (E7.2).

---

## Décision 3 — Tout paramètre de modèle est de classe `Fitted`

Sans exception, y compris ceux repris tels quels d'une base publique.

### Pourquoi même les paramètres publiés

Un `R_s` publié par le CEC n'a pas été mesuré : il a été **ajusté** par le
CEC sur des mesures. Le redistribuer ne change pas sa nature. Le classer
`ManufacturerData` parce qu'il provient d'une base officielle
confondrait l'autorité de la source avec la nature de la grandeur.

Ce que le paramètre porte en plus de sa valeur : le modèle
d'appartenance, la provenance, la **méthode d'obtention**, les données
d'ajustement, l'incertitude quand elle existe, et les résidus.

La méthode d'obtention est décisive : reprendre des paramètres CEC
(`PublishedAsIs`) et les ré-identifier depuis une fiche
(`FivePointDatasheet`) produisent des objets de même type et de qualité
radicalement différente. Sans la méthode, rien ne les distingue.

---

## Décision 4 — Robustesse numérique fondée sur une preuve, non sur un repli optimiste

À tension fixée, le résidu de l'équation implicite est **strictement
décroissant en courant, de dérivée majorée par −1** (doc 30 §4.5). Il en
découle que la racine est **unique** et **encadrable *a priori***.

### Ce qui est décidé

- Une méthode à encadrement (Brent) sert de **garantie de terminaison**,
  sur un intervalle dont l'existence est démontrée et non postulée.
- Newton-Raphson sert de **chemin rapide**, borné en itérations et
  contraint au domaine physique.
- La fonction W de Lambert sert d'**estimation initiale** lorsque son
  argument est numériquement sûr — jamais de garantie de robustesse.

### Pourquoi Lambert W n'est pas retenue comme méthode principale

Son caractère explicite est trompeur. `W_0` s'évalue elle-même
itérativement ; l'argument fait déborder l'exponentielle en `f64` dans le
domaine où se trouvent les modules réels ; et la formule en courant
**divise par `R_s`**, donc est singulière pour `R_s = 0`, cas
physiquement admissible. Le caractère explicite déplace la difficulté, il
ne la supprime pas.

### Ce qui reste ouvert

Le choix définitif (D-1) sera acté **après mesure comparative** des trois
méthodes sur une population réelle de modules. Cet ADR fixe la
*stratégie* et son fondement ; il ne grave pas encore les tolérances ni
les bornes d'itération, qui seront mesurées (doc 30 §8.7).

---

## Décision 5 — Aucune boucle sans critère de convergence formel

Toute résolution itérative porte : une tolérance relative à une grandeur
de référence du module, un critère portant sur le **résidu** et sur le
**pas**, une borne d'itérations finie, et un comportement de
non-convergence qui est une **erreur typée**.

Rendre un résultat approché comme s'il était convergé est la faute la
plus dangereuse d'un moteur de simulation, parce qu'elle est invisible
en aval. Un `Result` la rend impossible à ignorer.

Corollaire pour le MPP : la recherche par section dorée suppose
l'unimodalité de `P(V)`, qui **n'est pas garantie** pour des paramètres
pathologiques. L'hypothèse ne sera pas faite en silence : le point trouvé
sera vérifié comme maximum, et l'échec de vérification sera signalé. Un
MPP faux est plus dangereux qu'un MPP absent, parce qu'il est plausible.

---

## Décision 6 — PVsyst différé, modèle et format traités séparément

Le modèle est décrit dans une publication de conférence accessible
(Mermoud & Lejeune, EU PVSEC 2010) : sa physique est de la science
publiée. Le format `.PAN` reste bloqué (doc 27 §12.4).

Distinguer les deux est nécessaire pour rester exact. Mais le risque de
reproduire par inadvertance une interface propriétaire en implémentant le
modèle *et* son format est réel, et la règle fondatrice du projet est
stricte.

**Décision : différé.** Aucune implémentation avant un ADR dédié statuant
séparément sur le modèle et sur le format.

---

## Conséquences

### Positives

- Chaque source est exploitée avec le modèle que ses données permettent
  réellement d'initialiser, sans perte d'information.
- Le compilateur, et non la documentation, empêche de demander au SAPM ce
  qu'il ne sait pas produire.
- La terminaison du solveur repose sur une propriété démontrée.
- Aucun paramètre ajusté ne peut se faire passer pour une mesure.

### Négatives assumées

- Plus de types que dans une architecture à modèle unique. C'est le prix
  du fait 1, qui n'est pas négociable.
- Refuser de choisir un modèle par défaut demandera une décision à
  l'appelant là où un concurrent trancherait à sa place. Cohérent avec la
  politique de conflits ; à assumer en ergonomie.
- La stratégie hybride est plus complexe qu'une méthode unique. La
  complexité est justifiée par la garantie de terminaison ; elle devra
  rester confinée derrière une interface simple.

### Décisions restant ouvertes

D-1 (méthode définitive), D-2 (type de `a`), D-3 (modèle par défaut),
D-4 (seuil SAPM), D-5 (`E_g` non-silicium), D-6 (PVsyst), D-7
(tolérances) — voir doc 30 §15. Aucune ne doit être prise implicitement
par le premier code écrit.

### Traçabilité

| Élément | Emplacement |
|---|---|
| Spécification | `docs/design/30-specification-electrical-model.md` |
| Plan de tests | `docs/design/31-plan-tests-electrical-model.md` |
| Matrice | `docs/design/21-matrice-tracabilite.md`, Partie D |
| Validation empirique amont | `docs/design/26-validation-empirique-cec-sandia.md` |
