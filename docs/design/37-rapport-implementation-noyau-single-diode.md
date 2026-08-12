# 37 — Rapport d'implémentation du noyau Single-Diode d'`electrical_model`

**Date** : 2026-08-12
**Périmètre** : noyau mathématique de l'équation à diode unique
**Statut** : livré, en attente de validation explicite
**Références** : docs 35 et 36 (normatives), ADR-0012

> **Règle appliquée sans exception.** Le code est une traduction des
> mathématiques validées. Aucune équation n'a été réécrite pour arranger
> l'implémentation, aucune observation `[O]` n'a été promue en garantie
> `[P]`, et chaque propriété `[P]` du périmètre est devenue un invariant
> logiciel ou un test.

---

## 1. Ce qui est livré, et ce qui ne l'est pas

| Livré | Absent, et pourquoi |
|---|---|
| `f(I,V)`, `∂f/∂I`, `∂²f/∂I²`, `∂f/∂V`, `dI/dV` | Recherche du MPP — interdite tant que ce noyau n'est pas validé |
| `φ(V_j)`, `φ'`, `φ''` — forme explicite en tension de jonction | `P = V·I`, `dP/dV`, `dP/dV_j`, `ψ` — relèvent du MPP |
| Branche fermée `R_s = 0` (équation 18) | Translation De Soto, CEC, Sandia |
| Newton depuis `I₀ = 0` avec invariant d'intervalle | IEC 61853 |
| Repli encadré à bissection imposée, borne 82 | Interface générale de modèle |
| Garde anti-débordement fermée, refus a priori | Identification de paramètres |
| Sept types d'erreur, aucun `unwrap()` ni `expect()` dans le chemin normal | — |

Le tableau des absences figure aussi dans la documentation du module,
pour qu'un lecteur du code n'ait pas à le déduire.

---

## 2. Traduction des équations

Chaque fonction publique nomme l'équation de la doc 35 qu'elle
implémente. Aucune n'a été réarrangée.

| Équation | Fonction | Note d'implémentation |
|---|---|---|
| (3) | `residual` | Somme dans l'ordre de l'équation |
| (4) | `residual_derivative` | — |
| (6) | `residual_second_derivative` | Exactement `0.0` si `R_s = 0` |
| (8) | `residual_voltage_derivative` | Conductance, en Ω⁻¹ |
| (10) | `current_voltage_slope` | `Option` : jamais d'infini rendu |
| (18) | `closed_form_zero_series` | **Est** `f(0)`, déjà évalué par le test de domaine |
| (20)–(23) | `current_at_junction_voltage`, `junction_current_slope`, `junction_current_curvature` | Forme explicite : aucun solveur imbriqué |

### 2.1 La seule liberté prise, et sa justification

`diode_term` évalue `I_0·(e^u − 1)` par deux chemins selon `u` :

| Régime | Expression | Raison |
|---|---|---|
| `u ≤ 1` | `I_0 · expm1(u)` | Contre l'annulation catastrophique au voisinage de `V = 0`, où `e^u − 1` perd ses chiffres significatifs |
| `u > 1` | `exp(ln I_0 + u) − I_0` | Domaine logarithmique, contre le débordement |

Ce n'est **pas** une modification de l'équation : les deux expressions
sont la même quantité, et le point de bascule `u = 1` est celui où
`expm1` cesse d'apporter quoi que ce soit. Le choix est documenté sur la
fonction.

---

## 3. Invariants logiciels

**36 des 44 identifiants d'invariants** de la doc 36 sont couverts par un
test nommé, documenté et traçable par simple recherche dans les sources.
Un invariant supplémentaire a été ajouté.

| Famille | Testés | Différés |
|---|---|---|
| `DOM` — refus hors `𝓗` | 01 à 09 | — |
| `DIM` — homogénéité | 01 à 05 | — |
| `DER` — signes des dérivées | 01, 02 | — |
| `SOL` — existence, unicité, bornes | 01 à 04 | — |
| `NEW` — invariant d'intervalle | 01 | — |
| `ERR` — bornes d'erreur | 01, 02 | — |
| `FBK` — repli | 01 à 03, **+ 04 (ajouté)** | — |
| `OVF` — débordement | 01 à 03 | — |
| `RS0` — branche `R_s = 0` | 01, 02 | — |
| `MPP` — jonction | 01, 02, 10 | **03 à 09** |
| `TOL` — tolérances | 01, 02 | **03** |

**Huit invariants différés**, tous pour la même raison : `MPP-03` à
`MPP-09` traduisent `P-27` à `P-33`, qui portent sur `ψ` et sur
`d²P/dV_j²`, et `TOL-03` porte sur `τ_P`. Les trois grandeurs supposent
la puissance, que ce noyau ne calcule pas.

### 3.1 `INV-EM-FBK-04`, ajouté

`current_at_voltage_by_bracketing` existe pour que Newton puisse être
confronté à quelque chose qui ne lui doit rien. Cela ne vaut que si les
deux points d'entrée s'accordent sur le domaine admissible : un
contre-contrôle qui accepterait des tensions que le chemin de production
refuse comparerait deux problèmes différents. Rien ne le vérifiait.

### 3.2 `INV-EM-DIM-01` est un test de compilation

L'homogénéité de l'équation (1) ne peut pas être vérifiée à l'exécution :
une fois la grandeur devenue un `f64`, sa dimension a disparu. Ce qui est
vérifiable, c'est qu'aucune expression inhomogène **ne compile**. Trois
blocs `compile_fail` sont exécutés par `cargo test --doc` :

- `Current + Voltage` ;
- `ModifiedIdealityFactor + Voltage` — `a` est en volts sans être une
  tension du module (ADR-0009 D-2) ;
- `residual(Voltage, Current, …)` — intervertir les arguments est une
  erreur de type, non un nombre plausiblement faux.

Ces blocs sont dans la documentation de la **bibliothèque**, non dans le
fichier de tests : `rustdoc` n'exécute pas les blocs de code des tests
d'intégration, où ils n'auraient été que décoratifs.

---

## 4. Campagne de propriétés

14 propriétés génératives sur les 22 spécifiées au §7.2 de la doc 36.
Générateur SplitMix64, graine fixe et imprimée, flux distinct par
propriété (FNV-1a de l'identifiant mélangé à la graine) pour que deux
propriétés n'explorent pas le même tirage.

| Campagne | Cas générés | Échecs |
|---|---:|---:|
| Par défaut (256/propriété) | 3 584 | 0 |
| CI (5 000/propriété) | 70 000 | 0 |
| Vérification longue (50 000/propriété) | **700 000** | 0 |
| Trois graines alternatives, 4 000/propriété | 168 000 | 0 |

Le plancher de 200 cas par propriété est protégé par la suite elle-même :
`HELIOSPRO_PB_CASES` ne peut que l'élever. Une campagne écourtée ne peut
donc pas se faire passer pour une campagne complète.

### 4.1 Huit propriétés différées, motif consigné

`PB-06` à `PB-09`, `PB-13`, `PB-20` à `PB-22`. Elles portent sur la
puissance, sur le MPP ou sur la translation d'éclairement. La matrice
générée les **liste avec leur motif**, pour que le nombre de propriétés
vérifiées ne puisse jamais être pris pour le nombre de propriétés
spécifiées.

`PB-12` est implémentée sous sa **forme noyau** : le §7.2 l'énonce en
termes d'éclairement `G`, que le noyau ne connaît pas — l'éclairement
n'atteint l'équation à diode unique qu'à travers le modèle de
translation. Ce qui est vérifiable ici est le comportement quand `I_ph`
tend vers zéro, jusqu'à `10⁻¹²` A. La forme en éclairement reste **due**.

### 4.2 Ce que la campagne a trouvé

Un cas sur 3 584 a échoué au premier passage, sur `PB-17`. Le diagnostic
est au §5.1 : **l'assertion était fausse, pas le code**.

---

## 5. Deux mesures produites par cette phase

### 5.1 Le budget de Newton est dépassé dans 0,64 % du domaine de génération

`PB-17` affirmait qu'un repli sur la bissection signalait qu'un itéré
avait quitté l'intervalle. C'était une confusion entre deux des trois
sorties objectives du §4.5 : l'épuisement du budget en est une autre, et
elle est **légitime** — c'est précisément pourquoi 30 est documenté comme
un budget et non comme une borne.

Le phénomène est calculable. Profondément dans le régime exponentiel,
`f ≈ −I_0 e^u` et `f' ≈ −(R_s I_0/a) e^u`, donc le pas de Newton vaut
`−a/R_s` : une constante, indépendante de `I_0`, de `V` et de la distance
restante. La descente coûte donc `≈ (I_ph − I*)·R_s/a` itérations.

| Mesure (200 000 jeux admissibles) | Valeur |
|---|---|
| Au-dessus du budget de 30 | **0,642 %** |
| Pire décompte | **218** |
| Prédiction pour ce jeu | **217,1** |

Reproductible :
`cargo run --release -p electrical_model --example newton_budget_census`.

Pour un module réel le groupe `I_ph·R_s/a` vaut 1,6. Le domaine de
génération monte à `R_s = 5 Ω`, qu'aucun module ne présente : le
phénomène apparaît là et non sur les neuf jeux de référence, ce qui est
exactement le service qu'un test génératif doit rendre.

**Statut épistémique** : `[O]`. L'estimation est asymptotique et ne dit
rien de rigoureux du régime de transition. Elle n'est pas promue en
borne.

### 5.2 Le critère d'arrêt en largeur du repli n'est jamais atteint

Sur 237 000 cas — dont 76 535 tirés délibérément raides, avec `V` à moins
de `10⁻¹⁶` relatif de `V_oc` — le critère `|f| ≤ τ_f` se déclenche
**toujours** avant le critère `I_high − I_low ≤ τ_f`. Pire décompte
observé : 31 itérations sur une borne de 82.

Le critère en largeur est conservé : la démonstration de la borne 82
(doc 35 §6.3) raisonne **sur la largeur de l'encadrement**. Si le code ne
testait pas la largeur, la démonstration ne décrirait plus le code.

---

## 6. Couverture, et ce qu'elle ne couvre pas

| Fichier | Lignes | Régions |
|---|---:|---:|
| `error.rs` | **100,00 %** | 100,00 % |
| `single_diode/parameters.rs` | 99,33 % | 98,12 % |
| `single_diode/residual.rs` | 99,59 % | 99,82 % |
| `single_diode/solver.rs` | **88,99 %** | 93,00 % |
| **Atelier complet** | **98,22 %** | 98,26 % |

Le seuil de 95 % de la CI porte sur l'atelier et il est tenu.
`solver.rs` reste à 89 %, et **aucune exclusion de couverture n'a été
posée** pour redresser le chiffre : masquer ces lignes aurait supprimé
l'information, pas le risque.

### 6.1 Les 35 lignes non couvertes de `solver.rs`, une par une

| Lignes | Ce que c'est | Pourquoi c'est inatteignable |
|---|---|---|
| 69, 77 | Branches dégénérées de `fallback_iteration_bound` | Exigent `τ_f ≤ 0`, donc `I_ph = 0`, refusé par H-1 au constructeur |
| 183–186 | `NonFiniteValue` — résidu en `I = 0` | La garde (17) borne l'exposant avant tout calcul |
| 205–208 | `NonFiniteValue` — courant résolu | Idem |
| 274 | Newton — résidu non fini | Idem |
| 297 | Newton — pente nulle ou positive | `∂f/∂I ≤ −1` sous `𝓗` (P-04) |
| 304 | Newton — itéré hors `[0, I_ph + I_0]` | Invariant d'intervalle (P-13) |
| 357, 360 | Repli — repli sur bissection quand la fausse position dégénère | Exige `f_high = f_low`, or `f_low > 0 > f_high` strictement (P-18) |
| 368–371 | `NonFiniteValue` pendant le repli | Garde (17) |
| 394–400 | Critère d'arrêt en largeur | Mesuré inatteignable — §5.2 |
| 404–410 | `NonConvergence` | La borne 82 est démontrée suffisante (P-19) |

Trois lignes supplémentaires (`parameters.rs` 456 et 531, `residual.rs`
435) sont des arguments de message d'assertion, évalués seulement en cas
d'échec.

**Ce qu'il faut en retenir** : ces lignes ne sont pas du code non testé,
ce sont des refus dont les démonstrations établissent qu'ils ne se
déclenchent pas. Les supprimer reviendrait à faire reposer la correction
sur la validité des preuves seule. Les garder, c'est appliquer la règle
du projet — **refuser plutôt qu'avertir** — jusqu'aux cas que l'on croit
impossibles.

---

## 7. Portes de qualité

| Porte | Résultat |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 avertissement |
| `cargo test --workspace` | ✅ 0 échec |
| `cargo doc --workspace --no-deps` | ✅ 0 avertissement |
| Couverture ≥ 95 % (atelier) | ✅ 98,22 % |
| Aucune régression sur les crates existants | ✅ |
| `unsafe` | ✅ interdit par `#![forbid(unsafe_code)]` |
| Dépendances externes | ✅ aucune, hors bibliothèque standard |
| `unwrap()` / `expect()` dans le chemin normal | ✅ aucun |

### 7.1 Trois avertissements Clippy ont révélé du vrai

| Avertissement | Ce qu'il pointait |
|---|---|
| `neg_cmp_op_on_partial_ord` sur `!(guard < LIMIT)` | La négation était **volontaire** — elle refuse aussi un `NaN`. Réécrite en `guard.is_nan() \|\| guard >= LIMIT`, ce qui dit la même chose lisiblement |
| Idem sur `!(derivative <= -1.0)` | Même motif, dans un test |
| Dernier `return None` de Newton inatteignable | **Code mort** : la boucle rendait toujours la main avant. Remplacé par un `break`, ce qui rend la ligne atteignable et testée |

---

## 8. Défauts corrigés en cours d'implémentation

| Défaut | Nature | Correction |
|---|---|---|
| `sweep` calculait `(V_oc·k)/n` | Deux arrondis : au point final, un ULP **au-dessus** de `V_oc`, hors du domaine fermé. Dix tests en échec | Fraction formée d'abord : `k/n` vaut exactement `1.0` au bout |
| `INV-EM-RS0-01` comparait relativement au résultat | Près de `V_oc`, trois termes de l'ordre de `I_ph` s'annulent vers `10⁻¹⁴` : aucune précision relative n'est atteignable sur une différence aussi petite | Tolérance absolue à l'échelle de `I_ph`, comme `τ_f` |
| `PB-17` traitait tout repli comme une sortie d'intervalle | Confusion entre deux sorties objectives distinctes | Le test rejoue la récurrence, **classe** la sortie, et exige que la méthode rapportée corresponde |
| `thermal_voltage` sans appelant | L'expression `k·T_c/q` était réécrite dans ses deux voisines : deux routes vers la même grandeur, donc une occasion de diverger | `from_diode` et `diode_ideality` passent par elle |
| `from_diode` infaillible | `N_s = 0` produisait `a = 0` silencieusement, refusé ensuite **au nom de `a`** | Renvoie un `Result` et refuse au nom du bon paramètre |
| `INV-EM-DIM-01` et `DIM-05` sans test | Deux invariants spécifiés sans traduction | `compile_fail` pour l'un, test de mise à l'échelle pour l'autre |

---

## 9. Ce que le noyau permet d'affirmer

> Sous les hypothèses `𝓗`, vérifiées à l'entrée et refusées sinon, et
> pour `V ∈ [0, V_oc]`, le solveur rend l'unique solution de l'équation
> (1) avec une erreur majorée par la tolérance, en un nombre d'itérations
> borné par 82.

Chaque terme est soutenu : `𝓗` par `INV-EM-DOM-01..09`, l'unicité par
`SOL-01`, la borne d'erreur par `ERR-01` et l'équation (14), la borne
d'itérations par `FBK-02` et la démonstration du §6.3.

## 10. Ce qu'il ne permet pas d'affirmer

> Que le modèle décrit correctement un module photovoltaïque réel.

**Aucune des huit validations empiriques `[E]` n'est acquise.** Aucune
fiche constructeur réelle n'a été confrontée au noyau. Deux points
ouverts hérités restent bloquants pour une validation complète :

| Point | Suivi |
|---|---|
| Cinq fiches constructeurs réelles — **à ne pas simuler** | doc 31 §13 |
| Grille exacte IEC 61853-1 — lecture du texte normatif requise | doc 30 §9.4 |

---

## 11. Critère d'arrêt

Le noyau Single-Diode est livré et vérifié. **Aucun MPP, aucun CEC,
aucun Sandia, aucune IEC 61853 n'a été implémenté**, conformément à
l'instruction. La suite est conditionnée à une nouvelle validation
explicite.
