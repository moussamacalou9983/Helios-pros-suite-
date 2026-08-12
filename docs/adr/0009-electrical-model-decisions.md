# ADR-0009 — Fermeture des sept décisions ouvertes d'`electrical_model`

**Statut** : Proposé (conception ; implémentation soumise à validation explicite)
**Date** : 2026-08-12
**Catégorie** : Nouvelle architecture + méthode numérique
**Référence** : `docs/design/32-revue-scientifique-finale-electrical-model.md`

**Complète l'ADR-0008**, qui avait volontairement laissé sept décisions
ouvertes (D-1 à D-7). Le présent ADR les ferme. Conformément à
`docs/governance/adr-policy.md`, l'ADR-0008 n'est pas réécrit : les
décisions qu'il avait prises restent valides, et celles qu'il avait
laissées en suspens sont tranchées ici.

**Renverse trois recommandations provisoires** de la doc 30, signalées
comme telles ci-dessous.

---

## Contexte

L'ADR-0008 avait fixé la stratégie sans graver les valeurs, en indiquant
explicitement que sept points seraient tranchés « après mesure » ou
« après analyse ». Cette analyse a été menée (doc 32) et a produit trois
résultats qui contredisent les recommandations provisoires. Les taire
pour préserver la cohérence apparente du dossier serait exactement la
faute que le principe directeur interdit : *la science ne s'adapte pas au
code déjà écrit* — ni aux documents déjà rédigés.

---

## Décision 1 — Trois abstractions, hiérarchisées par ce qui est dérivable

Ferme le raffinement de l'ADR-0008 D-2, qui n'en prévoyait que deux.

| Abstraction | Garantie |
|---|---|
| `ParameterizedElectricalModel` | Jeu de paramètres nommé, avec identité de modèle, provenance, méthode d'obtention et **domaine de validité** |
| `PointBasedPerformanceModel` | Rend les points remarquables à conditions données |
| `ContinuousIVModel` | Rend `I(V)` pour tout `V` du domaine |

**`ContinuousIVModel` a `PointBasedPerformanceModel` pour supertrait**, et
non l'inverse : qui produit la courbe en extrait les points ; la
réciproque est fausse. La hiérarchie suit ce qui est réellement
dérivable, pas une commodité de nommage.

`SapmParameters` et `Iec61853Matrix` implémentent le modèle à points et
**pas** le modèle continu. Une fonction générique sur `ContinuousIVModel`
les refuse **à la compilation**.

### Ajout : `ValidityDomain` obligatoire

Chaque jeu de paramètres porte son domaine. Ce n'est pas décoratif : le
modèle d'interpolation IEC 61853 a un domaine **structurellement fini**
(l'enveloppe convexe de sa grille), et le SAPM a une enveloppe de
régression hors de laquelle il extrapole sans fondement. Un modèle qui
ignore son domaine ne peut pas refuser, et un modèle qui ne peut pas
refuser finit par répondre n'importe quoi.

---

## Décision 2 — Le moteur ne choisit pas de modèle par défaut

Ferme D-3 de l'ADR-0008.

Lorsque plusieurs modèles sont initialisables depuis une même source, le
moteur **expose les possibilités avec leur statut** et laisse le choix à
l'appelant. Il ne classe pas, ne préfère pas, ne décide pas.

Cinq statuts, dont la sémantique est fixée :

| Statut | Signification |
|---|---|
| `SUPPORTED` | Initialisable sans hypothèse ajoutée |
| `PARTIALLY_SUPPORTED` | Initialisable au prix d'une perte d'information ou d'une hypothèse **déclarée** |
| `INSUFFICIENT_DATA` | Ce type de source pourrait suffire ; cet enregistrement-ci ne suffit pas |
| `INCONSISTENT_DATA` | Contradiction — **jamais arbitrée**, mécanisme de `pv_module` réutilisé |
| `NOT_SUPPORTED` | Ce type de source ne peut **jamais** initialiser ce modèle |

La distinction `INSUFFICIENT_DATA` / `NOT_SUPPORTED` n'est pas
cosmétique : la première invite à compléter l'enregistrement, la seconde
à changer de modèle. Les confondre enverrait l'utilisateur chercher une
donnée qui n'existe dans aucun enregistrement de ce type.

Tout statut autre que `SUPPORTED` nomme ce qui manque, pourquoi cela
bloque, et ce qui le lèverait. **Aucune valeur par défaut ne comble un
manque** : un `Adjust` mis à zéro parce qu'absent transformerait un
modèle CEC en modèle De Soto sous une étiquette fausse.

---

## Décision 3 — Identification : De Soto à cinq points, initialisée par Phang

Ferme la question de la méthode d'identification.

| Source | Méthode retenue |
|---|---|
| CEC | `PublishedAsIs` |
| Matrice IEC 61853 | `LeastSquaresOnMatrix` |
| Courbe I-V mesurée | `LeastSquaresOnIvCurve` |
| Fiche constructeur | **`FivePointDeSoto`**, estimation initiale par Phang (1984) |

### Villalva (2009) est écartée — motif épistémique

La méthode est simple, robuste et largement citée. Elle **fixe `n`
arbitrairement**, typiquement entre 1,0 et 1,5, sans que rien dans les
données ne le justifie. Les quatre autres paramètres, ajustés autour de
ce choix, en héritent l'arbitraire.

Un paramètre libre choisi par commodité produit un résultat qui a
l'apparence d'une mesure et la nature d'une convention. C'est ce que la
règle fondatrice interdit. La méthode est écartée **non pour un défaut
technique mais pour un défaut épistémique**.

### Pourquoi Phang est acceptée comme initialiseur alors que Lambert W est rejetée

Ce n'est pas une incohérence : les deux problèmes diffèrent.

- L'équation I-V **a** une garantie de convergence globale (Décision 4).
  Un initialiseur n'y comble aucun manque.
- Le système 5×5 d'identification **n'en a aucune**. Aucune propriété de
  convexité n'a pu y être établie. Un bon point de départ y est donc
  décisif, pas décoratif.

### `ln(I_0)` obligatoire

`I_0 ∈ [10⁻¹², 10⁻⁹] A` et `I_L ≈ 10 A` : neuf ordres de grandeur entre
deux inconnues du même système. Résoudre en `I_0` détruirait la précision
sur la plus petite. Le système est résolu en `ln(I_0)` — condition de
bien-posé, vérifiée par test.

---

## Décision 4 — Newton principal, sur convergence démontrée

**Ferme D-1, et renverse la recommandation provisoire de la doc 30 §8.6.**

### Ce que disait la recommandation provisoire

Brent portait la garantie de terminaison, Newton apportait la vitesse,
Lambert W fournissait le point de départ.

### Ce que l'analyse a établi

Pour `I_0 > 0`, `a > 0`, `R_sh > 0`, `R_s > 0`, le résidu `f(I)` est
**strictement décroissant** (`f' ≤ −1`) **et strictement concave**
(`f'' < 0`). Pour une fonction décroissante et concave, l'itération de
Newton converge **depuis tout point de départ**, en décroissant de façon
monotone dès la première itération (démonstration doc 32 §4.3).

**Newton porte donc lui-même la garantie mathématique.** Brent cesse
d'être la garantie et devient la réponse à la seule menace réelle : le
débordement de `exp` en `f64`, qui est une pathologie de représentation,
non de mathématique.

### Décision retenue

| Rang | Méthode | Rôle |
|---|---|---|
| 0 | Forme fermée | `R_s = 0` — le résidu devient **affine**, solution exacte sans itération |
| 1 | **Newton-Raphson** | Méthode principale, convergence globale démontrée |
| 2 | **Brent** | Repli sur l'encadrement démontré `[0, I_L + I_0]`, contre la pathologie flottante |
| — | Bisection | Oracle de test uniquement |

Le cas `R_s = 0`, que la doc 30 signalait comme un piège (singularité de
la formule Lambert W), est en réalité **le plus simple de tous**.

### Corollaire — le résidu majore l'erreur

Puisque `|f'| ≥ 1`, le théorème des accroissements finis donne

`|I_n − I*| ≤ |f(I_n)|`

Le critère de convergence porte donc sur le **seul résidu** : un critère
sur le pas serait redondant, et la tolérance se transporte directement en
borne d'erreur garantie sur le courant rendu.

### Corollaire — recherche du MPP

**Renverse également la recommandation provisoire.** Maximiser `P(V)`
directement plafonne la précision de `V_mp` à `√ε ≈ 1,5·10⁻⁸` en
relatif, limite intrinsèque au problème et non à la méthode. Le moteur
résoudra donc `dP/dV = 0` comme **recherche de racine**, où la pleine
précision est atteignable.

L'encadrement `[0, V_oc]` est garanti (`dP/dV` positif en 0, négatif en
`V_oc`). L'**unicité ne l'est pas** : elle équivaut à l'unimodalité de
`P`, qui peut tomber pour `R_sh` très faible. Le point trouvé sera donc
**vérifié comme maximum**, et l'échec de vérification signalé — un MPP
faux est plus dangereux qu'un MPP absent, parce qu'il est plausible.

---

## Décision 5 — Lambert W n'est pas implémentée

**Ferme D-1 dans son volet Lambert W, et renverse la doc 30 §8.6.**

Le critère posé était : si elle n'apporte pas assez de valeur, ne pas
l'implémenter. Elle ne l'apporte pas.

| Motif | Détail |
|---|---|
| Ne résout pas le débordement | Son argument déborde au même seuil que celui de Newton |
| Singulière là où c'est trivial | La formule en courant divise par `R_s` ; or `R_s = 0` a une solution exacte |
| Annulation catastrophique | `W_0(θ) ≈ ln θ − ln ln θ` pour `θ` grand — soustraction de grands nombres proches, dans le régime des modules réels précisément |
| Sous-flux | Le préfacteur ~10⁻¹⁰ multiplié par une exponentielle petite peut donner `θ = 0`, d'où perte totale d'information |
| Gain marginal | 2 à 3 itérations de Newton, dont la convergence est déjà démontrée |
| Coût réel | Absente de la bibliothèque standard ; la politique « zéro dépendance » imposerait de l'implémenter et de la **valider** — campagne de précision et jeu de tests propres — pour une primitive utilisée nulle part ailleurs |

Aucun risque de branche : l'argument étant toujours positif, seule `W_0`
interviendrait. Ce n'est pas ce qui la disqualifie.

La formulation de Jain & Kapoor (2004) reste un résultat exact, utilisable
comme **vérification croisée indépendante** hors chemin critique. Elle est
écartée comme composant du moteur, pas comme référence mathématique.

---

## Décision 6 — `E_g` appartient au matériau, et au jeu de paramètres

Ferme D-5.

### `E_g` ne dépend pas de l'architecture

HJT, TOPCon, PERC et IBC sont des **architectures de contact appliquées
au silicium cristallin** : leur `E_g` est celui du silicium, 1,121 eV. Le
gap est fixé par le matériau, pas par la manière de le contacter.

`E_g` se sélectionne donc sur `CellTechnology` **seule**. Toute
indexation sur `CellArchitecture` serait une erreur de catégorie — et
contredirait l'orthogonalité déjà inscrite dans `pv_module`.

### `E_g` appartient au jeu de paramètres, pas au module

Découverte de l'analyse. Les paramètres De Soto de la base CEC ont été
ajustés en supposant `E_g,ref = 1,121 eV` et `dE_g/dT = −2,677·10⁻⁴`
**pour tous les modules, quelle que soit leur technologie**. Pour un
module CdTe issu du CEC, les six paramètres publiés sont donc
**auto-cohérents avec une valeur silicium**.

Les traduire ensuite avec le vrai gap du CdTe **mélangerait deux cadres**
et produirait une incohérence.

| Méthode | `E_g` retenu |
|---|---|
| `PublishedAsIs` (CEC) | Valeurs silicium, enregistrées comme **convention d'ajustement de la source**, signalées quand la technologie n'est pas du silicium |
| `FivePointDeSoto`, `LeastSquaresOnMatrix` | `E_g` du matériau, **sourcé** ; à défaut `INSUFFICIENT_DATA` |

Le moteur **signale** sans corriger : corriger casserait l'auto-cohérence
du jeu de paramètres.

### Interdiction

Le moteur n'attribue **jamais** silencieusement une valeur silicium à une
technologie non silicium. Deux issues seulement : valeur sourcée, ou
`INSUFFICIENT_DATA`.

CIGS et pérovskite ne sont **pas représentables par une valeur unique** :
leur gap est réglé par la composition. Publier une valeur pour « CIGS »
serait une fiction, pas une approximation.

---

## Décision 7 — Tolérances dérivées, non choisies

Ferme D-7.

| Grandeur | Valeur | Origine |
|---|---|---|
| Tolérance résidu courant | `10⁻¹² · I_sc` | 450× le plancher d'arrondi, 10⁻¹⁰ de l'incertitude des données |
| Tolérance résidu tension | `10⁻¹² · V_oc` | Idem, forme en tension |
| Tolérance puissance | `2·10⁻¹² · P` | Propagation des relatifs sur `P = V·I` |
| Critère de convergence | Résidu **seul** | Le résidu majore l'erreur (Décision 4) |
| Itérations Newton | **30** | 4 à 8 typiques ; au-delà, pathologie et non difficulté |
| Itérations Brent | **81** | `2 × log₂((I_L+I_0)/τ) = 2 × 40,4` — **calculé** |
| Itérations bisection (test) | **41** | `⌈log₂((I_L+I_0)/τ)⌉` |

### Pourquoi relatif et non absolu

La mise à l'échelle relative rend **constants** les deux rapports qui
importent : marge au plancher d'arrondi (450×) et négligeabilité devant
l'incertitude des données (10⁻¹⁰). Vérifié pour `I_sc` de 0,5 A à 20 A,
les trois donnant exactement 40,4 étapes de bisection et 450× de marge.
Une tolérance absolue serait trop lâche pour les petits modules et trop
serrée pour les grands.

### Statut

Ces seuils sont **dérivés par analyse numérique**, non mesurés. Une
campagne de sensibilité sur données réelles reste à mener pour les
confirmer ; elle est inscrite au plan de tests.

---

## Décision 8 — Le typage porte les dimensions, parce que l'analyse l'impose

Non prévue à l'ouverture, imposée par la vérification dimensionnelle.

### `α_Isc` n'a pas la même dimension selon le modèle

| Modèle | Dimension imposée par l'homogénéité |
|---|---|
| De Soto | `A/K` — **absolu** |
| SAPM | `K⁻¹` — **relatif** |

Le rapport entre les deux est `I_sco ≈ 10 A` : les confondre introduit une
erreur d'un facteur dix. Ce n'est pas une hypothèse mais ce que
l'homogénéité **impose**, et cela concorde avec les sources — colonne CEC
`alpha_sc` en A/°C, `αIsc` de King *et al.* défini comme « normalized ».

Le SAPM mélange en outre les deux bases dans un même jeu : `α_Isc`
relatif, `β_Voco` absolu.

**Conséquence** : le type `TemperatureCoefficient` portant sa
`CoefficientBasis`, créé pour `pv_module`, n'était pas une précaution
mais une **nécessité dimensionnelle**, démontrée après coup. Toute
équation consommant un coefficient exigera la base attendue et refusera
l'autre.

### Autres exigences de typage

| Grandeur | Décision |
|---|---|
| `a` (facteur d'idéalité modifié) | **Type propre**, pas `Voltage` : `a + V_oc` doit être impossible à écrire (ferme D-2) |
| Résistance | Type `Resistance` en Ω, à créer |
| `E_g` | Type propre ; `k` en eV/K **dérivée** de la valeur CODATA en J/K, jamais saisie séparément |
| Températures | Les **rapports** exigent des kelvins ; les **différences** sont sûres en °C comme en K. Asymétrie portée par le typage |
| `C_3` (SAPM) | En **V⁻¹**, contrairement à `C_0`, `C_1`, `C_2` sans dimension : un type « coefficient SAPM » unique et adimensionnel serait faux |

---

## Décision 9 — PVsyst reste différé, et n'est jamais un oracle

Ferme D-6, en confirmant l'ADR-0008.

Le modèle est publié (Mermoud & Lejeune, EU PVSEC 2010) ; le format
`.PAN` reste bloqué. Aucune implémentation avant un ADR dédié statuant
séparément sur les deux.

PVsyst peut servir de **comparaison externe** quand il est accessible.
Comme pvlib, il n'est **jamais un oracle absolu** : un écart signale une
divergence à expliquer, dans un sens ou dans l'autre — jamais une erreur
présumée de notre côté, jamais une tolérance à élargir jusqu'à ce que
l'écart rentre.

---

## Décision 10 — Le modèle à deux diodes est hors périmètre

Sept paramètres pour cinq contraintes de fiche : **sous-déterminé**. La
littérature referme le système en fixant `a_1 = 1` et `a_2 = 2` par
hypothèse physique. Ces valeurs imposées sont des hypothèses de modèle,
non des mesures.

Écarté pour la première implémentation — non parce qu'il serait mauvais
(il est meilleur à faible irradiance) mais parce qu'il ne peut être
identifié qu'au prix d'hypothèses que nos données ne justifient pas. À
réexaminer quand des matrices IEC 61853 réelles seront disponibles, qui
le contraindraient correctement.

---

## Décision 11 — L'interpolation IEC 61853 est un modèle à part entière

Ajout de cette revue. La doc 30 traitait IEC 61853 comme une source de
données ; c'est une erreur de classement. La partie -3 définit un calcul
qui **interpole la matrice mesurée** sans postuler aucun circuit
équivalent.

| Propriété | Conséquence |
|---|---|
| Aucun paramètre ajusté | **Aucune classe F** — les valeurs restent mesurées (classe B) |
| Aucune hypothèse de modèle | Pas d'erreur de modèle, seulement une erreur d'interpolation |
| Extrapolation **interdite** | Hors de l'enveloppe convexe de la grille, il refuse |
| Ne produit pas `I(V)` | `PointBasedPerformanceModel` uniquement |

**C'est le modèle le plus exact et le moins général.** Il mérite
d'exister précisément parce que sa faiblesse est explicite : il ne
prétend rien hors du domaine mesuré, là où les modèles physiques
extrapolent avec une confiance qu'ils ne justifient pas toujours.

---

## Conséquences

### Positives

- Le solveur repose sur une **convergence démontrée**, pas sur un repli
  espéré. Les bornes d'itération sont calculées.
- Le critère de convergence porte une **borne d'erreur garantie**.
- Lambert W écartée : une primitive numérique en moins à implémenter, à
  valider et à maintenir.
- La recherche du MPP gagne huit ordres de grandeur de précision par
  rapport à la maximisation directe.
- Une confusion d'unités d'un facteur dix, indétectable à la lecture, est
  rendue impossible par le typage.

### Négatives assumées

- Trois documents antérieurs portent des recommandations renversées. Les
  corrections sont **datées et signalées** plutôt que réécrites, comme
  l'exige la politique ADR.
- Refuser de choisir un modèle par défaut demande une décision à
  l'appelant là où un concurrent trancherait. Cohérent avec la politique
  de conflits ; à assumer en ergonomie.
- Plus de types d'unités à créer (`Resistance`, `a`, `E_g`).
- Le modèle à deux diodes, meilleur à faible irradiance, est reporté.

### Traçabilité

| Élément | Emplacement |
|---|---|
| Analyse complète | `docs/design/32-revue-scientifique-finale-electrical-model.md` |
| Rapport de décision | `docs/design/33-rapport-decision-electrical-model.md` |
| Spécification | `docs/design/30-specification-electrical-model.md` |
| Plan de tests | `docs/design/31-plan-tests-electrical-model.md` |
| Matrice | `docs/design/21-matrice-tracabilite.md`, Partie D |
