# ADR-0011 — Formalisation mathématique d'`electrical_model` : 35 propriétés prouvées, 35 invariants logiciels

**Statut** : Proposé
**Date** : 2026-08-12
**Catégorie** : Méthode numérique + spécification formelle
**Références** : `docs/design/35-formalisation-mathematique-electrical-model.md`, `docs/design/36-statut-epistemique-electrical-model.md`

**Complète** les ADR-0009 et ADR-0010 sans les contredire. Aucune décision
antérieure n'est infirmée : cette phase formalise, elle ne corrige pas.

---

## Contexte

L'ADR-0010 avait retiré deux garanties excessives et démontré l'unicité du
MPP. Restait à transformer ces résultats en **propriétés formelles
directement traduisibles en tests**, et à séparer rigoureusement ce qui
est prouvé de ce qui est seulement observé.

Cette phase n'a produit **aucun renversement** — c'est en soi un
résultat : les corrections de l'ADR-0010 ont tenu à la formalisation
complète. Elle a en revanche produit une **simplification structurelle**
qui n'avait pas été vue.

---

## Décision 1 — La recherche du MPP n'appelle jamais le solveur implicite

**Découverte de cette phase.**

En variable de tension de jonction `V_j = V + I R_s`, le courant devient
**explicite** :

$$ I = \varphi(V_j) = I_{ph} + I_0 - I_0 e^{V_j/a} - \frac{V_j}{R_{sh}} $$

et la dérivée de la puissance également :

$$ \frac{dP}{dV_j} = \varphi + \varphi'\,\psi, \qquad \psi = V_j - 2R_s\varphi $$

Ni `φ` ni `φ'` ne demandent de résolution. **La racine de `dP/dV_j`
se cherche donc sur une fonction fermée.**

### Ce que cela change

| Avant | Après |
|---|---|
| Chaque évaluation de `dP/dV` appelait le solveur implicite | Aucune |
| Deux problèmes numériques imbriqués | **Découplés** |
| Erreur du solveur propagée dans la recherche du MPP | Aucune propagation |

Le problème implicite (`I` à `V` donné) et le problème d'optimisation
(le MPP) deviennent **indépendants**. C'est une simplification de fond,
pas une optimisation.

**[O]** Corroboré : le MPP obtenu en `V_j` coïncide avec un balayage de
40 001 points **à la résolution du balayage** (`≈ 10⁻³ V`), tandis que la
méthode par racine atteint `10⁻¹³ V_oc`. L'écart mesure la limite de la
maximisation discrète, pas celle de la méthode retenue.

---

## Décision 2 — 35 propriétés prouvées, 35 invariants logiciels

Chaque propriété démontrée reçoit un **invariant logiciel nommé**,
testable, et traçable jusqu'à sa démonstration.

| Famille | Invariants | Objet |
|---|---|---|
| `INV-EM-DOM-01..09` | 9 | Vérification de `𝓗` — refus si violée |
| `INV-EM-DIM-01..05` | 5 | Homogénéité dimensionnelle portée par le typage |
| `INV-EM-DER-01..02` | 2 | Signes des dérivées |
| `INV-EM-SOL-01..04` | 4 | Existence, unicité, bornes, monotonie de `I(V)` |
| `INV-EM-NEW-01` | 1 | Invariant d'intervalle de Newton |
| `INV-EM-ERR-01..02` | 2 | Bornes d'erreur, **distinctes** en `I` et en `V` |
| `INV-EM-FBK-01..03` | 3 | Encadrement, borne 82, invariance |
| `INV-EM-OVF-01..03` | 3 | Débordement, sous-flux |
| `INV-EM-RS0-01..02` | 2 | `R_s = 0` exact, aucune division par `R_s` |
| `INV-EM-MPP-01..10` | 10 | Les six étapes de la démonstration + explicitation |
| `INV-EM-TOL-01..03` | 3 | Tolérances dérivées |

**Le principe directeur devient opérationnel** : une garantie
mathématique qui ne se traduit pas en invariant testable n'est pas
retenue.

---

## Décision 3 — Statut épistémique obligatoire et non promouvable

Quatre statuts, exclusifs :

| Statut | Ce qui l'établit |
|---|---|
| **[P]** Prouvé | Une démonstration écrite |
| **[O]** Observé | Une mesure reproductible sur un échantillon fini |
| **[H]** Hypothèse | Une décision documentée |
| **[E]** Empirique | Une confrontation à des données réelles |

**Règle de non-promotion** :

> Une propriété **[O]** ne devient jamais **[P]** par accumulation de cas.
> Mille jeux de paramètres ne font pas une démonstration.

Le changement de statut d'un énoncé exige un ADR.

### Le cas exemplaire

L'observation « exactement un extremum sur 9 jeux × 4 001 points » est la
plus tentante à confondre avec une preuve. **Elle ne prouve rien** : un
balayage à pas fini ne peut exclure deux racines séparées de moins d'un
pas. L'unicité vient de la démonstration en six étapes, et d'elle seule.

Inversement, si le balayage avait trouvé deux extrema, **il aurait
réfuté** la démonstration. C'est le rôle d'une corroboration : elle ne
confirme pas, elle peut infirmer.

**Bilan actuel** : 35 **[P]**, 15 **[O]** (aucune promue), 15 **[H]**
(9 vérifiables, 6 non), 8 **[E]** (aucune acquise).

---

## Décision 4 — La dérivée est analytique, la différentiation numérique est rejetée par calcul

| Méthode | Précision atteignable | Verdict |
|---|---|---|
| **Analytique** | `~ε` | **Retenue** |
| Différences finies | `~\sqrt{\varepsilon} \approx 1{,}5\cdot10^{-8}` | **Rejetée** |
| Différentiation automatique | `~ε` | Rejetée — équivalente à l'analytique, avec la machinerie en plus |

Le rejet des différences finies n'est pas une préférence : leur erreur
optimale est **du même ordre que le plafond** qui a fait rejeter la
maximisation discrète du MPP. Les employer réintroduirait exactement le
défaut que la recherche par racine élimine.

**[O]** Vérification croisée : les expressions analytiques `dI/dV` et
`dP/dV_j` s'accordent aux différences centrées à `10⁻¹²`–`10⁻⁹` près,
écart qui **mesure les différences finies**, non les expressions.

---

## Décision 5 — Le refus hors domaine est un invariant, pas un avertissement

> **La démonstration ne vaut que sous `𝓗`. Hors de `𝓗`, l'algorithme ne
> produirait pas une erreur détectable : il rendrait un point plausible et
> faux.**

Neuf hypothèses vérifiables (`I_ph > 0`, `I_0 > 0`, `a > 0`, `R_sh > 0`,
`R_s ≥ 0`, `N_s ≥ 1`, `T_c > 0` en kelvins, quadrant de fonctionnement,
garde anti-débordement) sont contrôlées **avant tout calcul**, et leur
violation est un **refus** nommant le paramètre fautif.

Six hypothèses de **modélisation** (circuit à diode unique, `R_s`
constant, `R_sh ∝ 1/G`, `a ∝ T`, `E_g(T)` du silicium, uniformité
électrique) ne sont **pas vérifiables par le moteur**. Elles doivent être
**déclarées avec chaque résultat**, jamais tues : un résultat obtenu sous
l'hypothèse d'uniformité dans une situation d'ombrage est faux sans
qu'aucun test interne ne puisse le détecter.

---

## Décision 6 — Le seuil anti-débordement est imposé par IEEE 754

$$ \ln I_0 + \ln\max\!\left(1, \frac{R_s}{a}\right) + u_{\max} < \ln(\texttt{f64::MAX}) = 709{,}78 $$

**709,78 n'est pas un choix** : c'est `ln(2^{1024})`, imposé par la norme
IEEE 754 binary64. La garde est donc entièrement dérivée, sans constante
arbitraire.

Trois lignes de défense : contrôle a priori (refus), évaluation en
domaine logarithmique (`exp(ln I_0 + u)`, gain de `|\ln I_0| ≈ 23`
unités), et `expm1` contre l'annulation près de `V = 0`. **Aucun
`try/catch`.**

**[P]** Un débordement est un **symptôme d'incohérence des paramètres** :
pour un jeu auto-cohérent, `V_{oc}/a \approx \ln(I_{ph}/I_0) \approx 25`.

---

## Décision 7 — Aucun seuil pour `R_s → 0`, par inspection exhaustive

Inspection de **toutes** les expressions de la branche générale — `f`,
`u`, `f'`, `f''`, le pas de Newton, l'encadrement : **aucune ne divise
par `R_s`**. Le paramètre n'apparaît qu'en numérateur ou dans des
dénominateurs valant `a` ou `R_sh`.

Il n'existe donc **aucune singularité**, donc **aucun seuil à choisir**.
La forme fermée est sélectionnée par le prédicat **exact** `R_s == 0`,
qui est une égalité et non un seuil.

**Propriété testable** : pour `R_s = 0`, le solveur général doit égaler la
forme fermée à `1 ulp`, en une itération.

---

## Conséquences

### Positives

- La recherche du MPP est découplée du solveur implicite : deux problèmes
  numériques deviennent indépendants.
- Chaque garantie mathématique est adossée à un invariant testable nommé.
- Le statut épistémique est explicite et non promouvable : aucune
  observation ne peut dériver vers le statut de théorème.
- Toutes les constantes numériques sont dérivées (IEEE 754, plancher
  d'arrondi, encadrement démontré) — aucune n'est choisie.

### Négatives assumées

- 35 invariants à implémenter et tester, plus 22 propriétés génératives.
  C'est le prix d'une garantie vérifiable.
- Six hypothèses de modélisation restent non vérifiables ; elles devront
  accompagner chaque résultat, ce qui alourdit l'API.
- Huit validations empiriques restent inaccessibles faute de données. Le
  moteur pourra être déclaré mathématiquement correct, **pas** validé
  physiquement.

### Traçabilité

| Élément | Emplacement |
|---|---|
| Formalisation, 29 équations numérotées, démonstrations | doc 35 |
| Statut épistémique, protocole, tests de propriétés | doc 36 |
| Audit amont | doc 34, ADR-0010 |
| Matrice | `docs/design/21-matrice-tracabilite.md`, Partie D |
