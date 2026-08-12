# ADR-0012 — Implémentation du noyau Single-Diode : décisions prises en traduisant les mathématiques en code

**Statut** : Proposé
**Date** : 2026-08-12
**Catégorie** : Implémentation numérique
**Références** : `docs/design/35-formalisation-mathematique-electrical-model.md`, `docs/design/36-statut-epistemique-electrical-model.md`, `docs/design/37-rapport-implementation-noyau-single-diode.md`

**Complète** l'ADR-0011. Aucune équation de la doc 35 n'a été modifiée
pour faciliter l'implémentation, aucune observation `[O]` n'a été promue
en garantie `[P]`. Les décisions ci-dessous portent sur des points que la
formalisation laissait ouverts parce qu'ils n'apparaissent qu'au contact
du code.

---

## Contexte

La phase 2B avait produit 35 propriétés démontrées et les invariants
logiciels correspondants. L'écriture du noyau a soulevé sept questions
que la formalisation ne pouvait pas trancher d'avance, et a produit
**deux mesures qui corrigent ou précisent le texte de la doc 35**.

---

## Décision 1 — La borne fermée `V_oc ≤ a·ln(1 + I_ph/I_0)` lève la circularité de la garde

La garde anti-débordement (17) fait intervenir
`u_max = (V_oc + I_ph·R_s)/a`. Or `V_oc` est lui-même une racine : exiger
un solveur pour décider si l'on a le droit de solver est circulaire.

La majoration `V_oc ≤ a·ln(1 + I_ph/I_0)` — obtenue en négligeant le
terme de fuite, qui ne peut qu'abaisser `V_oc` — rend la garde
**entièrement fermée**. Elle est calculée au constructeur, avant toute
évaluation d'exponentielle.

> **Conséquence mesurée, qui corrige une analyse antérieure.** Le terme
> dominant de la garde est `I_ph·R_s/a`, **et non `V_oc/a`**. La borne
> fermée plafonne `V_oc/a` à `ln(1 + I_ph/I_0) ≈ 24` pour tout jeu de
> forme physique. Un `a` petit ne déborde donc pas : avec `a = 0,05 V`,
> `V_oc` tombe à 1,22 V et la garde vaut 66,5 — le jeu est numériquement
> sain et le refuser aurait été une erreur.

## Décision 2 — `R_s = 0` est un **prédicat exact**, jamais un seuil

`has_zero_series_resistance()` teste `R_s == 0.0`. Aucun epsilon.

Ce choix n'est tenable que parce que la branche générale **ne contient
aucune division par `R_s`** (P-24) : un `R_s` de `10⁻³⁰⁰ Ω` y passe sans
difficulté. Un seuil aurait introduit une discontinuité artificielle à un
endroit où la physique n'en a pas, et aurait fallu le justifier — ce qui
est impossible, aucune valeur ne se distinguant des autres.

Vérifié pour `R_s ∈ {10⁻³⁰⁰, 10⁻³⁰, 10⁻¹², 10⁻⁹, 10⁻⁶} Ω`
(`INV-EM-RS0-02`).

## Décision 3 — Le test de domaine `V ≤ V_oc` est `f(0; V) ≥ 0`, sans résolution

`f(0; ·)` est strictement décroissante en `V` par l'équation (8). Donc
`V ≤ V_oc` **est** `f(0; V) ≥ 0` : une équivalence, pas une
approximation. Le test coûte une évaluation du résidu, et cette
évaluation **est déjà** la solution quand `R_s = 0` (équation 18), donc
la forme fermée ne coûte rien de plus que le test de domaine.

## Décision 4 — Le repli est nommé pour ce qu'il est

L'algorithme implémenté est une **fausse position avec bissection imposée
une itération sur deux**. Il n'est pas appelé « Brent », ni dans le code,
ni dans la documentation, ni dans les messages d'erreur. La borne de 82
est celle de cet algorithme-là, démontrée au §6.3 de la doc 35 ; la borne
de Brent (≈ 1636) n'est pas exploitée.

## Décision 5 — Le budget de Newton est **dépassé en pratique**, et pour une raison calculable

**Observation `[O]` produite par cette phase, non anticipée par la doc 35.**

Le §4.5 annonçait « 2 à 13 itérations mesurées sur les 9 jeux ». C'est
exact sur ces neuf jeux, et trompeur comme énoncé général.

Profondément dans le régime exponentiel, `f ≈ −I_0 e^u` et
`f' ≈ −(R_s I_0/a) e^u`, donc le pas de Newton vaut

$$ -\frac{f}{f'} \;\approx\; -\frac{a}{R_s} $$

— une **constante**, indépendante de `I_0`, de `V` et de la distance
restant à parcourir. La descente depuis `I_1 ≈ I_ph` jusqu'à `I*` coûte
donc environ `(I_ph − I*)·R_s/a` itérations, majoré par `I_ph·R_s/a` :
**le même groupe que celui qui domine la garde**.

Mesure reproductible
(`cargo run --release -p electrical_model --example newton_budget_census`,
200 000 jeux admissibles du domaine de génération §7.1) :

| Grandeur | Valeur |
|---|---|
| Jeux au-dessus du budget de 30 | **0,642 %** |
| Pire décompte observé | **218** |
| Prédiction `(I_ph − I*)·R_s/a` pour ce jeu | **217,1** |

Pour un module réel — `I_ph ≈ 10 A`, `R_s ≈ 0,3 Ω`, `a ≈ 1,85 V` — le
groupe vaut 1,6 : moins de deux itérations. Le domaine de génération
monte à `R_s = 5 Ω`, qu'aucun module ne présente, et c'est pourquoi le
phénomène apparaît là et non sur les neuf jeux de référence.

**Décision** : le budget de 30 est **conservé tel quel**, et reste
documenté comme un budget et non comme une borne. Le repli n'est donc pas
un filet théorique : il est effectivement atteint, et `PB-01`, `PB-14` et
`PB-15` sont ce qui établit que la réponse qu'il rend est la bonne.

**Ce qui n'est pas fait** : promouvoir cette estimation en borne. Elle
est asymptotique dans le régime exponentiel et ne dit rien de rigoureux
du régime de transition. Elle reste `[O]`.

## Décision 6 — Le critère d'arrêt en largeur du repli est **redondant**, et conservé

Le repli s'arrête sur `|f| ≤ τ_f` **ou** sur `I_high − I_low ≤ τ_f`.

**Observation `[O]`** : sur 237 000 cas admissibles — dont 76 535 tirés
délibérément raides (`R_s ∈ [2, 5] Ω`, `a ∈ [0,5, 1,5] V`) avec `V` à
moins de `10⁻¹⁶` relatif de `V_oc` — le critère en résidu est **toujours**
celui qui se déclenche. Le critère en largeur n'a jamais été atteint, et
l'épuisement de la borne jamais non plus (pire décompte : 31 sur 82).

**Décision** : le conserver malgré tout. La démonstration de la borne 82
(§6.3) est un raisonnement **sur la largeur de l'encadrement**. Si le
code ne testait pas la largeur, la démonstration ne décrirait plus le
code. Ces lignes ne sont donc pas du code mort : elles sont ce qui rend
la preuve applicable à l'implémentation.

**Conséquence assumée sur la couverture** : `solver.rs` reste à 88,99 %
de lignes couvertes. Les lignes non couvertes sont énumérées et
justifiées une par une à la doc 37 §6. Aucune exclusion de couverture
n'a été posée pour redresser le chiffre — masquer ces lignes aurait
supprimé l'information au lieu du risque.

## Décision 7 — H-6 et H-7 sont refusées à l'équation (2), pas au constructeur

`N_s` et `T_c` n'apparaissent pas dans `SingleDiodeParameters` : ils
entrent par l'équation (2), qui construit `a`. `from_diode` était
initialement infaillible, ce qui laissait `N_s = 0` produire `a = 0`
silencieusement, refusé ensuite par le constructeur **au nom de `a`**.

`from_diode` renvoie désormais un `Result` et refuse au nom du bon
paramètre. Un appelant qui s'est trompé de compte de cellules, ou qui a
passé des degrés Celsius là où des kelvins étaient attendus, l'apprend
tel quel (`INV-EM-DOM-06`, `INV-EM-DOM-07`).

---

## Conséquences

| Conséquence | Nature |
|---|---|
| La garde est entièrement fermée, calculée avant toute exponentielle | Positive |
| Aucun seuil arbitraire n'existe dans le noyau | Positive |
| Le repli est atteint dans 0,64 % du domaine de génération | **Neutre, mesurée** |
| `solver.rs` conserve 35 lignes non couvertes, toutes justifiées | **Assumée** |
| `from_diode` est un changement d'API rompant (retourne `Result`) | Assumée — le crate n'est pas publié |
| Deux critères d'arrêt du repli sont redondants sous `𝓗` | Assumée — la preuve l'exige |

## Ce que cette phase ne permet toujours pas d'affirmer

Que le modèle décrit correctement un module photovoltaïque réel. Cela
relève de `[E]`, et **aucune des huit validations empiriques requises
n'est acquise** : aucune fiche constructeur réelle n'a été confrontée au
noyau, et aucune ne doit être simulée pour combler ce vide.
