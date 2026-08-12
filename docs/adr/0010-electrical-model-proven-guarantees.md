# ADR-0010 — Garanties mathématiques d'`electrical_model` : deux retraits, une démonstration

**Statut** : Proposé
**Date** : 2026-08-12
**Catégorie** : Méthode numérique — correction de garanties
**Référence** : `docs/design/34-audit-mathematique-electrical-model.md`

**Corrige l'ADR-0009**, Décisions 4 et 7, et **infirme partiellement** sa
Décision 4. Conformément à `docs/governance/adr-policy.md`, l'ADR-0009
n'est pas réécrit : les affirmations corrigées y restent consultables pour
la traçabilité historique.

---

## Contexte

L'ADR-0009 annonçait trois garanties mathématiques. Un audit formel, avec
vérification numérique sur neuf jeux de paramètres dont pathologiques, en
a invalidé deux et en a démontré une troisième que l'ADR-0009 déclarait
hors d'atteinte.

Une revue qui n'aurait fait que confirmer ses propres conclusions
n'aurait rien vérifié.

---

## Décision 1 — Retrait de « Newton converge depuis tout point de départ »

### Ce qui était affirmé (ADR-0009 D-4)

> « Pour une fonction décroissante et concave, l'itération de Newton
> converge **depuis tout point de départ**. »

### Ce qui est vrai

L'énoncé est **exact dans ℝ** et la démonstration tient (audit §4.1). Il
est **faux en `f64`**.

Contre-exemple mesuré, jeu nominal 72 cellules à `V = 0,8 V_oc` : depuis
`I₀ = 10⁴ A`, `exp` déborde dès la première évaluation. La suite
mathématique existe ; la suite calculée n'existe pas.

| `I₀` | Résultat |
|---:|---|
| −10⁶ à 200 | Converge (5 à 33 itérations) |
| **10⁴** | **`OverflowError`** |

### Ce qui est décidé

L'affirmation est **retirée**. Elle est remplacée par une garantie
**conditionnée au point de départ spécifié** :

> Depuis `I₀ = 0`, et sous les hypothèses de physicité, Newton converge
> vers l'unique racine ; **tous les itérés restent dans `[I*, I_L]`**, et
> l'argument de l'exponentielle reste borné par
> `u_max = (V_oc + I_L R_s)/a`, **calculable avant exécution**.

Démonstration : `f(0) ≥ 0` et `|f'| ≥ 1` bornent le premier pas par
`|f(0)| ≤ I_L` ; la monotonie de la suite fait le reste (audit §4.3).

Cette garantie est **plus faible** que celle retirée, et **plus utile** :
elle fournit une borne calculable au lieu d'une promesse invérifiable.

---

## Décision 2 — Retrait de la borne d'itérations « 81 »

### Ce qui était affirmé (ADR-0009 D-7)

> « Itérations Brent : **81**, dérivé — Brent réduit l'intervalle d'au
> moins moitié toutes les deux itérations. »

### Ce qui est vrai

Cette propriété **n'est pas celle que Brent démontre**. La borne établie
par Brent (1973, ch. 4) est de l'ordre de `(log₂((b−a)/δ))²`, soit
**≈ 1636** pour notre encadrement — vingt fois la valeur annoncée.
L'algorithme de Brent garantit la convergence et interdit la lenteur
pathologique de Dekker, mais pas une réduction géométrique d'un facteur
deux toutes les deux itérations.

**La valeur 81 était une borne théorique erronée.**

### Ce qui est décidé

Le repli n'est plus « Brent textuel » mais un **schéma spécifié par nous** :
une **bissection est imposée une itération sur deux**.

L'intervalle est alors réduit d'au moins moitié toutes les deux
itérations **par construction**. Après `2m` itérations sa largeur est
`≤ (b−a)/2^m` ; la condition `(b−a)/τ ≤ 2^m` donne `m = 41`, d'où

> **Borne : 82 itérations, démontrée pour l'algorithme que nous
> spécifions — et non revendiquée de Brent.**

Bornes de référence, toutes dérivées :

| Méthode | Borne | Nature |
|---|---|---|
| Bissection pure | **41** | Serrée, exacte |
| **Repli spécifié** | **82** | **Démontrée pour notre schéma** |
| Brent textuel | ≈ 1636 | Rigoureuse, non exploitée |
| Newton | 30 | **Budget** avant repli, non borne — 13 mesurées au pire |

**Indépendance de la machine** : `(I_L + I_0)/τ = 1,5·10¹²` quel que soit
le module, numérateur et dénominateur étant tous deux proportionnels à
`I_sc`. Vérifié : `log₂` exactement 40,4 pour `I_sc` de 20 A à 10⁻²⁰ A.

---

## Décision 3 — L'unicité du MPP est démontrée

### Ce que l'ADR-0009 déclarait

> « L'unicité **n'est pas** garantie : elle équivaut à l'unimodalité de
> `P`, qui peut tomber pour `R_sh` très faible. »

### Ce qui est démontré

**Elle est garantie.** La démonstration passe par la tension de jonction
`V_j = V + I R_s`, qui rend le courant **explicite** :
`I = φ(V_j)`, avec `φ' < 0` et `φ'' < 0`.

Avec `ψ(V_j) = V_j − 2 R_s φ(V_j)` :

| Lemme | Énoncé |
|---|---|
| 1 | `ψ' = 1 + 2R_s|φ'| > 0` — `ψ` strictement croissante, donc `{ψ ≥ 0}` est un **intervalle** |
| 2 | Sur `{ψ < 0}` : `dP/dV_j = φ + φ'ψ > 0` — `P` strictement croissante, **aucun point critique** |
| 3 | Sur `{ψ ≥ 0}` : `d²P/dV_j² = 2φ' + φ''ψ − 2R_sφ'² < 0` — **strictement concave** |

Tout point critique est donc dans un intervalle où `P` est strictement
concave : **il y en a au plus un**. Le changement de signe de `dP/dV`
entre `V = 0` (`= I_sc > 0`) et `V = V_oc` (`< 0`) en garantit **au moins
un**. Exactement un. ∎

**Vérification numérique** : les trois lemmes et le comptage d'extrema
tiennent sur 4 001 points par jeu, pour les neuf jeux — y compris
`R_sh = 1,5 Ω`, le cas que l'ADR-0009 désignait comme le risque
principal. **Exactement un extremum dans tous les cas.**

### Conséquence

La recherche du MPP **peut** désormais être présentée comme garantie dans
le domaine de physicité. La vérification que le point trouvé est un
maximum **reste exigée** — non plus contre la multiplicité des extrema,
qui est exclue, mais contre une erreur d'implémentation ou une violation
non détectée des hypothèses.

---

## Décision 4 — Le solveur est Newton **avec** sauvegarde, ni l'un ni l'autre seul

L'ADR-0009 laissait entendre que Newton portait seul la garantie. La
Décision 1 ci-dessus l'infirme.

| Option | Rejetée / retenue |
|---|---|
| Newton seul | **Rejetée** — garantie conditionnée au point de départ, sans recours si la condition est violée |
| Brent seul | **Rejetée** — abandonnerait la convergence quadratique et le résultat exact en une itération pour `R_s = 0`, sans rien gagner |
| **Newton + sauvegarde** | **Retenue** — Newton pour la vitesse, l'encadrement démontré `[0, I_L + I_0]` pour la garantie et la borne |

Déclenchement du repli sur conditions **objectives** : itéré non fini,
sortie de l'encadrement, ou dépassement du budget d'itérations. Aucune
heuristique.

---

## Décision 5 — La borne « résidu ≥ erreur » ne se transpose pas à la tension

`f' = df/dI` est **sans dimension**, d'où `|ΔI| ≤ |f|` — dimensionnellement
correct, et valide sur tout ℝ.

Mais `dh/dV` est une **conductance** (A/V). La minoration disponible est
`|dh/dV| ≥ 1/R_sh`, d'où

> `|ΔV| ≤ R_sh · |h|`

**forme différente, dépendant de `R_sh`**. Transposer naïvement la
relation du courant aurait été une faute dimensionnelle.

**Conséquence** : `tol_V` est un seuil **distinct**, `τ_h = 10⁻¹² V_oc/R_sh`,
plafonné par le plancher d'arrondi. Les quatre tolérances sont désormais
justifiées séparément : `tol_residual` (analyse numérique), `tol_I`
(dérivée du précédent), `tol_V` (propre, via `R_sh`), `tol_P` (dérivée,
propagation sur `P = V·I`).

---

## Décision 6 — Aucun seuil pour `R_s → 0`

Newton **ne divise jamais par `R_s`**. En `R_s = 0`, `f` est affine,
`f' = −1`, et le premier pas donne la solution **exacte**. Pour `R_s`
petit, `f` est presque affine et Newton converge en une à deux itérations.

*Mesure* : `R_s = 0` → 2 itérations ; `R_s = 10⁻⁹ Ω` → 2 itérations,
résultats identiques à `10⁻⁹` près.

**Il n'existe aucune singularité, donc aucun seuil à choisir.** La forme
fermée est sélectionnée par le prédicat **exact** `R_s == 0`, qui est une
égalité et non un seuil.

Argument supplémentaire, découvert ici, en faveur du rejet de Lambert W :
sa formule en courant divise **réellement** par `R_s`.

---

## Décision 7 — Débordement de `exp` : contrôle a priori, jamais `try/catch`

Trois lignes de défense, dans cet ordre :

1. **Contrôle a priori** — `u_max = (V_oc + I_L R_s)/a` est calculé avant
   toute itération et comparé au seuil de sûreté. S'il le dépasse, le
   calcul est **refusé** avec un diagnostic nommant `a`, `V_oc` et `R_s`.
2. **Domaine logarithmique** — `I_0 e^u` est évalué comme
   `exp(ln I_0 + u)`, ce qui gagne `|ln I_0| ≈ 23` unités.
3. **`expm1`** pour `e^u − 1` aux petits `u`, contre l'annulation.

*Mesure* : `ln I_0 + u_max ∈ [−12,0 ; +13,6]` sur les neuf jeux, contre un
seuil de 709,78 — marge de plus de 690 unités logarithmiques.

**Un débordement est un symptôme d'incohérence des paramètres**, pas une
difficulté numérique intrinsèque : pour un jeu cohérent,
`V_oc/a ≈ ln(I_L/I_0) ≈ 25`. Le refuser a priori est scientifiquement
correct.

---

## Décision 8 — Phang : corrigée, et l'identification ne bloque jamais le moteur

L'ADR-0009 D-3 retenait Phang comme initialiseur du système 5×5 **depuis
une fiche**. C'est inexact : la méthode de Phang, Chan & Phillips (1984)
exige les deux pentes `dV/dI` en `V_oc` et en `I_sc`, **absentes de toute
fiche constructeur**.

| Cas | Initialisation |
|---|---|
| Courbe I-V mesurée | **Phang** — les pentes sont mesurables |
| Fiche seule | **Estimation heuristique documentée** |

**Un point de départ arbitraire n'est pas un paramètre arbitraire.**
C'est ce qui sépare une initialisation de la méthode de Villalva :
Villalva **fixe** `n` et le conserve dans le résultat ; une initialisation
ne fait que démarrer l'itération. Garde-fou exigé : la solution convergée
est **vérifiée contre les cinq contraintes** et rejetée si l'une est
violée.

**Stratégie d'échec** : le système 5×5 n'a aucune garantie de
convergence. Son échec produit `IDENTIFICATION_FAILED` et **n'invalide
jamais le moteur** : les modèles ne requérant pas d'identification
(`PublishedAsIs` depuis le CEC) restent pleinement utilisables.
L'identification est un service, pas un passage obligé.

---

## Conséquences

### Positives

- Les garanties annoncées sont désormais soit démontrées, soit
  conditionnées explicitement.
- L'unicité du MPP, déclarée hors d'atteinte, est acquise : la recherche
  du MPP devient un problème bien posé.
- Le domaine d'évaluation de `exp` est borné a priori, donc contrôlable
  sans exception.
- Une borne d'itérations fausse d'un facteur vingt a été retirée avant
  d'être inscrite dans du code.

### Négatives assumées

- Trois documents portent des affirmations corrigées. Signalées par des
  avis datés, non réécrites.
- La garantie de Newton est plus faible qu'annoncée : elle vaut depuis un
  point de départ spécifié, pas universellement.
- Le repli n'est plus « Brent » au sens textuel mais un schéma
  hybride que nous spécifions et dont nous démontrons la borne.

### Traçabilité

| Élément | Emplacement |
|---|---|
| Audit complet, démonstrations | `docs/design/34-audit-mathematique-electrical-model.md` |
| Décisions initiales corrigées | `docs/adr/0009-electrical-model-decisions.md` |
| Analyse amont | `docs/design/32-revue-scientifique-finale-electrical-model.md` |
| Jeux de tests (22) | doc 34 §16 |
| Matrice | `docs/design/21-matrice-tracabilite.md`, Partie D |
