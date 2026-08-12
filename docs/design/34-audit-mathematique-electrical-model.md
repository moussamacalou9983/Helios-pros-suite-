# 34. Audit mathématique final — `electrical_model`

**Date** : 2026-08-12
**Statut** : audit — **aucun code métier**
**Objet** : prouver, corriger ou retirer les garanties mathématiques annoncées
**Vérification numérique** : 9 jeux de paramètres, dont pathologiques (script d'analyse, hors du crate)

---

## 0. Verdict

| # | Affirmation auditée | Verdict |
|---|---|---|
| **A-1** | « Newton converge depuis tout point de départ » | ⚠️ **RETIRÉE** — vraie dans ℝ, **fausse en `f64`**. Remplacée par une garantie conditionnée au point de départ spécifié |
| **A-2** | « Borne d'itérations de Brent : 81 » | ⚠️ **RETIRÉE** — n'est pas une borne théorique de Brent. La borne réelle est ~1636. 82 ne vaut que pour un schéma à bissection **imposée** |
| **A-3** | « Unicité du MPP non démontrée » | ✅ **RENFORCÉE** — l'unicité **est** démontrable, et démontrée ici (§4) |
| **A-4** | « Phang initialise le système 5×5 depuis une fiche » | ⚠️ **CORRIGÉE** — Phang exige les pentes `dI/dV`, absentes des fiches |
| **A-5** | « `|f'| ≥ 1` donc le résidu majore l'erreur » | ✅ **CONFIRMÉE** pour la forme en courant. **Ne se transpose pas** à la forme en tension (§5) |
| **A-6** | « `R_s = 0` exige une branche séparée » | ✅ **CONFIRMÉE**, mais **aucun seuil** n'est requis pour `R_s → 0` (§6) |

Deux garanties excessives retirées, une propriété gagnée, deux
corrections. Le solde net est favorable, mais le fait que deux
affirmations n'aient pas survécu justifie l'audit.

---

## 1. Cadre : hypothèses de physicité

Toutes les démonstrations qui suivent supposent :

$$ I_L > 0,\quad I_0 > 0,\quad a > 0,\quad R_{sh} > 0,\quad R_s \ge 0,\quad n > 0,\quad N_s \ge 1 $$

toutes finies, et le domaine de fonctionnement

$$ V \in [0,\,V_{oc}], \qquad I \in [0,\,I_{sc}] $$

**Hors de ces hypothèses, rien n'est garanti et rien n'est tenté** : les
paramètres non physiques sont refusés avant tout calcul. La polarisation
inverse (`V > V_oc`, `I < 0`) relève du mismatch et de l'ombrage, hors
périmètre.

### 1.1 Fonction résidu

$$ f(I) \;=\; I_L - I_0\left[e^{u} - 1\right] - \frac{V + I R_s}{R_{sh}} - I,
\qquad u = \frac{V + I R_s}{a} $$

**Domaine de définition** : ℝ tout entier. `f` est composée de la
fonction exponentielle et de fonctions affines, toutes définies partout.

**Continuité et dérivabilité** : `f ∈ C^∞(ℝ)`. Aucune singularité,
aucun point anguleux, aucune restriction de domaine.

---

## 2. Signes des dérivées — démonstration

### 2.1 Dérivée première

$$ f'(I) \;=\; -\frac{I_0 R_s}{a}e^{u} \;-\; \frac{R_s}{R_{sh}} \;-\; 1 $$

Sous les hypothèses de §1, les trois termes sont respectivement `≤ 0`,
`≤ 0` et `= −1`. Donc

$$ \boxed{\;f'(I) \le -1 < 0 \quad \text{pour tout } I \in \mathbb{R}\;} $$

**`f` est strictement décroissante sur ℝ.**

*Homogénéité* : `[I_0 R_s/a] = A·Ω/V = A·(V/A)/V = 1`, et `[R_s/R_sh] = 1`.
`f'` est **sans dimension**, comme il se doit pour la dérivée d'un courant
par rapport à un courant.

*Vérification numérique* : sur 9 jeux de paramètres et 5 151 couples
`(V, I)` chacun, `min|f'|` observé va de 1,000000 (`R_s = 0`) à 1,200000
(`R_sh = 1,5 Ω`). Aucune violation.

### 2.2 Dérivée seconde

$$ f''(I) \;=\; -\frac{I_0 R_s^2}{a^2}e^{u} $$

$$ \boxed{\;f''(I) < 0 \;\text{ si } R_s > 0 \;;\qquad f''(I) = 0 \;\text{ si } R_s = 0\;} $$

**`f` est strictement concave pour `R_s > 0`, et affine pour `R_s = 0`.**

*Vérification numérique* : `max f''` observé entre `−7,3·10⁻²⁹` et
`−2,9·10⁻¹⁰`, et exactement `0` pour `R_s = 0`. Aucune violation.

---

## 3. Existence, unicité, encadrement

### 3.1 Unicité

Immédiate : une fonction strictement monotone s'annule au plus une fois.

### 3.2 Existence

`f` est continue sur ℝ.

- **`I → −∞`** : pour `R_s > 0`, `u → −∞` donc `e^u → 0`, et
  `f(I) ≈ I_L + I_0 − V/R_{sh} − I(1 + R_s/R_{sh}) → +∞`. Pour `R_s = 0`,
  `f(I) = C − I → +∞`. Dans les deux cas `f(−∞) = +∞`.
- **`I → +∞`** : le terme `−I` et, si `R_s > 0`, `−I_0 e^u`, divergent
  vers `−∞`. Donc `f(+∞) = −∞`.

Par le théorème des valeurs intermédiaires, **une racine existe**. Avec
§3.1 : **il en existe exactement une**, notée `I*`. ∎

### 3.3 Encadrement calculable

**Borne supérieure.** En `I = I_L + I_0` :

$$ f(I_L + I_0) = -I_0 e^{u} - \frac{V + (I_L+I_0)R_s}{R_{sh}} < 0 $$

pour `V ≥ 0`, les deux termes étant strictement négatifs. Donc
`I* < I_L + I_0`.

**Borne inférieure.** En `I = 0` :
`f(0) = I_L − I_0[e^{V/a} − 1] − V/R_{sh}`, qui est `≥ 0` pour tout
`V ≤ V_oc` **par définition même de `V_oc`** (qui est la tension annulant
le courant). Donc `I* ≥ 0`.

$$ \boxed{\;I^* \in [\,0,\; I_L + I_0\,] \quad \text{pour } V \in [0, V_{oc}]\;} $$

Encadrement **calculable sans itération**, à partir des seuls paramètres.

---

## 4. Comportement de Newton — la garantie, corrigée

### 4.1 Ce qui est vrai dans ℝ

**Proposition.** Pour `f` strictement décroissante et strictement concave
sur ℝ, l'itération `I_{n+1} = I_n − f(I_n)/f'(I_n)` converge vers `I*`
depuis tout `I_0 ∈ ℝ`, en décroissant de façon monotone dès `n = 1`.

*Démonstration.* Par concavité, la tangente en `I_n`,
`T_n(x) = f(I_n) + f'(I_n)(x − I_n)`, majore `f` partout. Par
construction `T_n(I_{n+1}) = 0`. Or `T_n(I^*) ≥ f(I^*) = 0 = T_n(I_{n+1})`,
et `T_n` est strictement décroissante (pente `f'(I_n) < 0`), d'où
`I^* \le I_{n+1}`. Ainsi **dès la première itération l'itéré est à droite
de la racine**.

Pour `n ≥ 1` : `I_n ≥ I^*` donne `f(I_n) ≤ 0` ; avec `f'(I_n) < 0`, le pas
`−f(I_n)/f'(I_n) ≤ 0`, donc `I_{n+1} ≤ I_n`. La suite est décroissante et
minorée par `I^*`, donc convergente vers une limite finie `L ∈ [I^*, I_1]`.
En passant à la limite, `f(L)/f'(L) = 0` avec `f'(L)` finie et non nulle,
donc `f(L) = 0`, donc `L = I^*`. ∎

### 4.2 Ce qui est faux en `f64` — retrait de A-1

La démonstration ci-dessus est valide **en arithmétique exacte**. Elle
suppose que `f(I_n)` est **évaluable**. Ce n'est pas le cas partout en
virgule flottante.

**Contre-exemple mesuré** (jeu nominal 72 cellules, `V = 0,8 V_oc`) :

| `I_0` de départ | Itérations | Convergence | `u` maximal atteint |
|---:|---:|---|---:|
| −10⁶ | 5 | oui | 21,2 |
| −100 | 5 | oui | 21,2 |
| **0** | **5** | **oui** | **21,2** |
| 5 | 5 | oui | 21,2 |
| 10,4 | 4 | oui | 21,2 |
| 50 | 9 | oui | 27,7 |
| 200 | 33 | oui | 52,0 |
| **10⁴** | — | **`OverflowError`** | **> 709,78** |

Depuis `I_0 = 10⁴`, `exp` déborde dès la première évaluation : la suite
mathématique existe, la suite calculée n'existe pas.

**L'affirmation « depuis tout point de départ » est donc retirée.**

Noter aussi que les points de départ très négatifs sont sûrs : `f` y est
quasi affine, et le premier pas de Newton atterrit près de `I_L`. C'est
le côté **droit** qui est dangereux, pas le gauche.

### 4.3 La garantie de remplacement — conditionnée et prouvée

**Proposition.** Depuis `I_0 = 0`, pour `V ∈ [0, V_oc]`, tous les itérés
de Newton restent dans `[I^*, I_L + I_0^{sat}] ⊂ [0, I_L + I_0^{sat}]`.

*Démonstration.* `f(0) ≥ 0` (§3.3) et `f'(0) < 0`, donc le premier pas
`−f(0)/f'(0) ≥ 0` : `I_1 ≥ 0`. Sa longueur est bornée par
`|f(0)/f'(0)| ≤ |f(0)|` puisque `|f'| ≥ 1` (§2.1), et `f(0) ≤ I_L`. Donc
`I_1 ∈ [I^*, I_L]`. Par §4.1, la suite décroît ensuite en restant
`≥ I^*`. Tous les itérés sont donc dans `[I^*, I_L]`. ∎

**Conséquence décisive** : l'argument de l'exponentielle est **borné a
priori** :

$$ u \;\le\; u_{\max} \;=\; \frac{V_{oc} + I_L R_s}{a} $$

quantité calculable **avant toute itération**, à partir des seuls
paramètres.

*Vérification numérique* : `u_max` observé de 10,1 à 35,7 sur les neuf
jeux, contre un seuil de débordement de 709,78. Marge d'un facteur 20 au
minimum. Et en domaine logarithmique, `ln(I_0) + u_max` va de `−12,0` à
`+13,6` — très loin de la limite.

**Énoncé retenu, à substituer partout :**

> Depuis le point de départ spécifié `I₀ = 0`, et sous les hypothèses de
> physicité, Newton converge vers l'unique racine, tous les itérés
> restant dans `[I*, I_L]`, et l'argument de l'exponentielle restant
> borné par `u_max = (V_oc + I_L R_s)/a`, vérifiable avant exécution.

C'est **plus faible** que l'affirmation retirée, et **plus utile** :
elle fournit une borne calculable.

### 4.4 Vitesse

`f'(I^*) ≤ −1 ≠ 0` : la racine est simple, la convergence est
**quadratique** au voisinage.

*Mesure* : depuis `I_0 = 0`, entre **2 et 13 itérations** sur les neuf
jeux, pour un résidu inférieur à `10⁻¹⁴ A`. Le maximum (13) correspond au
cas `R_s = 2 Ω`, volontairement extrême.

---

## 5. Résidu et erreur — vérification dimensionnelle

### 5.1 Forme en courant — confirmée

`f` est en ampères ; `I` est en ampères ; `f' = df/dI` est **sans
dimension** (§2.1).

Par le théorème des accroissements finis, il existe `ξ` entre `I_n` et
`I*` tel que `f(I_n) − f(I^*) = f'(ξ)(I_n − I^*)`. Avec `f(I^*) = 0` et
`|f'| ≥ 1` :

$$ \boxed{\;|I_n - I^*| \;\le\; |f(I_n)| \;} \qquad \text{[A]} \le \text{[A]} \;\checkmark $$

**Valide sur tout ℝ**, puisque `|f'| ≥ 1` y est valide partout (§2.1) et
que `f` est partout dérivable (§1.1).

**Réserve en virgule flottante** : la relation porte sur `f` exacte. Le
résidu calculé `f̂` diffère de `f` d'une erreur d'évaluation `η`, d'où

$$ |I_n - I^*| \;\le\; |\hat f(I_n)| + \eta, \qquad \eta \approx 10\,\varepsilon\,I_{sc} $$

Avec `τ = 10⁻¹² I_sc` et `η ≈ 2,2·10⁻¹⁵ I_sc`, la borne effective vaut
`τ(1 + 2,2·10⁻³)`. La correction est négligeable mais doit être
**énoncée**, pas tue.

### 5.2 Forme en tension — la relation NE se transpose PAS

Pour `V` à `I` donné, le résidu `h(V)` est toujours en ampères, mais

$$ \frac{dh}{dV} = -\frac{I_0}{a}e^{u} - \frac{1}{R_{sh}}
\qquad [\mathrm{A/V}] = [\Omega^{-1}] $$

C'est une **conductance**, pas un nombre sans dimension. La minoration
disponible est `|dh/dV| ≥ 1/R_{sh}`, d'où

$$ \boxed{\;|V_n - V^*| \;\le\; R_{sh}\,|h(V_n)|\;} \qquad [\mathrm{V}] = [\Omega][\mathrm{A}] \;\checkmark $$

**La borne d'erreur en tension fait intervenir `R_sh`.** Elle n'a pas la
même forme et se dégrade quand `R_sh` est grand. Transposer naïvement la
relation de §5.1 aurait été une faute dimensionnelle.

**Conséquence sur les tolérances** : pour garantir
`|ΔV| ≤ 10⁻¹² V_oc`, il faut un résidu inférieur à
`10⁻¹² V_oc / R_{sh}`, seuil **dérivé** et distinct de celui du courant.

---

## 6. Le cas `R_s = 0`, et l'absence de seuil pour `R_s → 0`

### 6.1 Solution analytique

Pour `R_s = 0`, `f''≡ 0` : `f` est **affine** en `I`.

$$ f(I) = \underbrace{I_L - I_0\left[e^{V/a} - 1\right] - \frac{V}{R_{sh}}}_{\text{indépendant de } I} - I $$

$$ \boxed{\;I^* = I_L - I_0\left[e^{V/a} - 1\right] - \frac{V}{R_{sh}}\;} $$

Solution **exacte**, sans itération. ∎

### 6.2 `R_s → 0` — aucun seuil n'est nécessaire

C'est la question à laquelle il fallait répondre sans inventer une
constante.

Newton appliqué à `f` **ne divise jamais par `R_s`**. En `R_s = 0`
exactement, `f' = −1` et le premier pas donne

$$ I_1 = I_0^{\text{départ}} + f(I_0^{\text{départ}}) = I^* $$

soit la **solution exacte en une itération**. Pour `R_s` petit mais non
nul, `f` est presque affine et Newton converge en une à deux itérations.

*Mesure* : `R_s = 0` → 2 itérations ; `R_s = 10⁻⁹ Ω` → 2 itérations ;
résultats identiques à `10⁻⁹` près.

**Il n'existe aucune singularité, donc aucun seuil à choisir.** Le
solveur général traite tout `R_s ≥ 0`. La forme fermée du §6.1 reste
utile comme **vérification croisée** et comme court-circuit sélectionné
par le prédicat **exact** `R_s == 0`, qui n'est pas un seuil mais une
égalité.

Ceci contraste avec la formulation de Lambert W, qui divise réellement
par `R_s` — argument supplémentaire, découvert ici, en faveur de son rejet.

---

## 7. Débordement de `exp` — stratégie, sans `try/catch`

### 7.1 Domaine d'évaluation, borné a priori

Par §4.3, les itérés restent dans `[0, I_L]`, donc

$$ u \in \left[\frac{V}{a},\; \frac{V + I_L R_s}{a}\right] \subseteq \left[0,\; u_{\max}\right],
\qquad u_{\max} = \frac{V_{oc} + I_L R_s}{a} $$

`u_max` est **calculable avant la première itération**.

### 7.2 Trois lignes de défense, dans cet ordre

1. **Contrôle a priori** — `u_max` est comparé au seuil de sûreté avant
   toute itération. S'il le dépasse, le calcul est **refusé** avec un
   diagnostic nommant `a`, `V_oc` et `R_s`. Aucun calcul n'est tenté.
2. **Évaluation en domaine logarithmique** — le produit `I_0 e^u` est
   évalué comme `exp(ln I_0 + u)`, ce qui déplace le seuil de
   débordement de `u > 709,78` à `ln I_0 + u > 709,78`, soit un gain de
   `|ln I_0| ≈ 23` unités. `ln I_0` est précalculé une fois.
3. **Terme `e^u − 1`** — évalué par `expm1` pour les petits `u`, afin
   d'éviter l'annulation catastrophique près de `V = 0`.

*Mesure* : sur les neuf jeux, `ln I_0 + u_max ∈ [−12,0 ; +13,6]`. La
marge au seuil de 709,78 est de plus de 690 unités logarithmiques.

### 7.3 Ce que le débordement signifie réellement

Pour un jeu de paramètres **physiquement cohérent**, `e^{V_oc/a} ≈ I_L/I_0`,
donc `V_oc/a ≈ ln(I_L/I_0) ≈ 25`. Un `u_max` proche de 709 supposerait
`a` absurdement petit ou `V_oc` incohérent avec `I_0`.

**Le débordement est donc un symptôme d'incohérence des paramètres, pas
une difficulté numérique intrinsèque.** Le refuser a priori est
scientifiquement correct, et non un contournement.

---

## 8. Le solveur — comparaison formelle et décision

### 8.1 Comparaison

| Critère | Newton seul | Brent seul | Newton + sauvegarde Brent |
|---|---|---|---|
| **Garantie de convergence** | Prouvée en ℝ ; **conditionnée** au point de départ en `f64` (§4.3) | Prouvée sur encadrement (§3.3) | Prouvée : celle de Brent, quel que soit le sort de Newton |
| **Vitesse** | Quadratique — 2 à 13 itérations mesurées | Superlinéaire — plus lente | Quadratique en régime normal |
| **Robustesse** | Bonne depuis `I₀ = 0` ; nulle depuis un point arbitraire | Totale sur l'encadrement | Totale |
| **Débordement** | Possible si `u_max` non contrôlé | Idem (mais itérés confinés à l'encadrement) | Idem, contrôlé a priori |
| **`R_s = 0`** | Exact en 1 itération | Converge, plus lentement | Exact en 1 itération |
| **`R_sh → ∞`** | 5 itérations (mesuré) | Converge | 5 itérations |
| **`R_sh` faible (1,5 Ω)** | 3 itérations, `|f'| ≥ 1,2` — **plus rapide** | Converge | 3 itérations |
| **Borne d'itérations démontrable** | Non (quadratique, pas de borne serrée) | Oui (§9) | Oui, portée par le repli |

### 8.2 Décision

**Newton avec sauvegarde par méthode à encadrement.** Ni Newton seul, ni
Brent seul.

*Pourquoi pas Newton seul* : sa garantie est conditionnée au point de
départ, et une garantie conditionnelle sans filet laisse le moteur sans
recours si une condition est violée par une donnée imprévue.

*Pourquoi pas Brent seul* : il abandonnerait la convergence quadratique
et le résultat exact en une itération pour `R_s = 0`, sans rien gagner
puisque la sauvegarde suffit à porter la garantie.

*Pourquoi l'hybride* : Newton fournit la vitesse et la garantie en
régime normal ; l'encadrement `[0, I_L + I_0]` étant **démontré**
(§3.3), le repli fournit une borne d'itérations démontrable (§9) et
couvre exactement le cas que Newton ne couvre pas.

**Déclenchement du repli**, sur conditions objectives et non heuristiques :
itéré non fini, sortie de l'encadrement, ou dépassement du budget
d'itérations.

---

## 9. Borne d'itérations — retrait de A-2

### 9.1 Ce qui était affirmé

« Brent réduit l'intervalle d'au moins moitié toutes les deux itérations,
d'où `2 × 40,4 ≈ 81`. »

### 9.2 Ce qui est vrai

Cette propriété **n'est pas** celle que Brent démontre. La borne établie
par Brent (1973, *Algorithms for Minimization Without Derivatives*, ch. 4)
sur le nombre d'évaluations est de l'ordre de

$$ \left(\log_2\frac{b-a}{\delta}\right)^2 $$

soit, pour notre encadrement, `40,4² ≈ 1636` — **vingt fois** la valeur
annoncée. L'algorithme de Brent garantit la convergence et interdit la
lenteur pathologique de Dekker, mais **pas** une réduction géométrique
d'un facteur deux toutes les deux itérations.

**La valeur 81 était donc une borne théorique erronée.** Elle est retirée.

### 9.3 Ce qui est retenu

Deux options honnêtes ; la seconde est choisie.

| Option | Borne | Statut |
|---|---|---|
| Brent textuel | `≈ 1636` | Rigoureuse mais inexploitable comme détecteur de pathologie |
| **Bissection imposée une itération sur deux** | **82** | **Rigoureuse pour l'algorithme que nous spécifions** |

En imposant qu'une itération sur deux soit une bissection, l'intervalle
est réduit d'au moins moitié toutes les deux itérations **par
construction**. Après `2m` itérations, sa largeur est `≤ (b−a)/2^m`. La
condition `(b−a)/2^m ≤ τ` donne `m ≥ log₂((b−a)/τ) = 40,4`, d'où
`m = 41` et **82 itérations**.

**C'est une propriété de l'algorithme que nous spécifions, pas une
propriété revendiquée de Brent.** La distinction est exactement celle
qu'exigeait l'audit.

Bornes de référence, toutes dérivées :

| Méthode | Borne | Nature |
|---|---|---|
| Bissection pure | `⌈log₂((I_L+I_0)/τ)⌉ = 41` | Serrée, exacte |
| **Repli spécifié** (bissection imposée 1 sur 2) | **82** | **Démontrée pour notre schéma** |
| Brent textuel | `≈ 1636` | Rigoureuse, non exploitée |
| Newton (budget avant repli) | 30 | **Budget**, non borne — 13 maximum mesuré |

### 9.4 Indépendance de la machine

`(b − a)/τ = (I_L + I_0)/(10⁻¹² I_{sc})`. Numérateur et dénominateur sont
tous deux proportionnels à `I_sc` : le rapport vaut `1,5·10¹²` **quel que
soit le module**.

*Vérification* : `log₂` exactement égal à **40,4** pour `I_sc` valant
20 A, 10 A, 1 A, 10⁻² A, 10⁻⁴ A, 10⁻⁸ A et 10⁻²⁰ A. La borne est
indépendante de la taille du module, du matériel et de la machine.

---

## 10. Unicité du MPP — démontrée (A-3)

La doc 32 déclarait cette propriété non démontrée. **Elle l'est.**

### 10.1 Changement de variable

Soit `V_j = V + I R_s` la **tension de jonction**. Le courant devient
**explicite** :

$$ I = \varphi(V_j) = I_L + I_0 - I_0 e^{V_j/a} - \frac{V_j}{R_{sh}} $$

$$ \varphi'(V_j) = -\frac{I_0}{a}e^{V_j/a} - \frac{1}{R_{sh}} < 0,
\qquad \varphi''(V_j) = -\frac{I_0}{a^2}e^{V_j/a} < 0 $$

`φ` est strictement décroissante et strictement concave.

**La correspondance `V ↔ V_j` est une bijection croissante** :
`dV/dV_j = 1 − R_s φ' = 1 + R_s|φ'| > 0`. Les points critiques de `P` se
correspondent donc un à un dans les deux paramétrages, et le maximum est
le même.

### 10.2 Puissance et dérivées

`V = V_j − R_s φ`, donc

$$ P(V_j) = V_j\varphi - R_s\varphi^2 $$

$$ \frac{dP}{dV_j} = \varphi + \varphi'\,\psi, \qquad \text{où } \psi(V_j) = V_j - 2R_s\varphi(V_j) $$

$$ \frac{d^2P}{dV_j^2} = 2\varphi' + \varphi''\psi - 2R_s\varphi'^2 $$

### 10.3 Trois lemmes

**Lemme 1 — `ψ` est strictement croissante.**
`ψ' = 1 − 2R_s φ' = 1 + 2R_s|φ'| > 0`. Donc `ψ` s'annule au plus une
fois, et `{ψ ≥ 0}` est un **intervalle**. ∎

**Lemme 2 — aucun point critique là où `ψ < 0`.**
Sur cette région, `φ' < 0` et `ψ < 0` donnent `φ'ψ > 0` ; avec `φ > 0`
dans le quadrant de fonctionnement, `dP/dV_j = φ + φ'ψ > 0` **strictement**.
`P` y est strictement croissante : aucun extremum. ∎

**Lemme 3 — `P` est strictement concave là où `ψ ≥ 0`.**
`2φ' < 0` ; `φ'' < 0` et `ψ ≥ 0` donnent `φ''ψ ≤ 0` ; `−2R_s φ'² ≤ 0`.
La somme est strictement négative. ∎

### 10.4 Théorème

**Sous les hypothèses de §1, `P` admet exactement un maximum sur
`[0, V_oc]`.**

*Démonstration.* Par le lemme 2, tout point critique appartient à
`{ψ ≥ 0}`, qui est un intervalle (lemme 1). Sur cet intervalle `P` est
strictement concave (lemme 3), donc y possède **au plus un** point
critique. D'où l'unicité globale.

Existence : `dP/dV|_{V=0} = I_{sc} > 0` et
`dP/dV|_{V=V_{oc}} = V_{oc}\,I'(V_{oc}) < 0` puisque `I' < 0`. Le
changement de signe garantit **au moins une** racine.

Exactement une. ∎

### 10.5 Localisation de la zone non concave

Sur `{ψ < 0}` on a `V_j < 2R_s φ ≤ 2R_s I_{sc}`, et `V ≤ V_j`. La zone
éventuellement non concave est donc confinée à

$$ V \in [\,0,\; 2R_s I_{sc}\,] $$

soit environ 6,2 V sur 45,2 V pour le jeu nominal — les 14 premiers
pourcents de la courbe, où `P` est de toute façon strictement croissante.

### 10.6 Vérification numérique

Les trois lemmes et le comptage d'extrema ont été vérifiés sur 4 001
points par jeu, pour les neuf jeux :

| Jeu | `ψ` croissante | `dP/dV_j > 0` si `ψ < 0` | Concave si `ψ ≥ 0` | Extrema |
|---|---|---|---|---|
| nominal 72c | ✔ | ✔ | ✔ | **1** |
| `R_s = 0` | ✔ | ✔ | ✔ | **1** |
| `R_s` très faible | ✔ | ✔ | ✔ | **1** |
| `R_s` élevé (2 Ω) | ✔ | ✔ | ✔ | **1** |
| `R_sh → ∞` | ✔ | ✔ | ✔ | **1** |
| `R_sh` faible (5 Ω) | ✔ | ✔ | ✔ | **1** |
| `R_sh` très faible (1,5 Ω) | ✔ | ✔ | ✔ | **1** |
| `G` très faible | ✔ | ✔ | ✔ | **1** |
| couche mince | ✔ | ✔ | ✔ | **1** |

Y compris pour le cas `R_sh = 1,5 Ω` que la doc 32 désignait comme le
risque principal d'unimodalité perdue.

### 10.7 Conséquences

1. La recherche du MPP **peut** être présentée comme garantie, dans le
   domaine de physicité.
2. L'encadrement `[0, V_oc]` de `dP/dV` est valide et contient **une
   seule** racine : une méthode à encadrement y converge vers le MPP,
   sans ambiguïté.
3. **La vérification que le point trouvé est un maximum reste exigée** —
   non plus contre la multiplicité des extrema, qui est exclue, mais
   contre une erreur d'implémentation ou une violation non détectée des
   hypothèses de physicité.

---

## 11. MPP et modèles à points — cloisonnement

`dP/dV` n'existe que si `I(V)` est définie et dérivable pour tout `V`.

**Le calcul du MPP par `dP/dV = 0` n'est donc exposé que par
`ContinuousIVModel`.** Il est structurellement inaccessible aux modèles
à points.

| Modèle | MPP disponible ? | Origine |
|---|---|---|
| De Soto, CEC, deux diodes | Oui | Calculé par `dP/dV = 0` |
| SAPM | **Non calculé** | `V_mp`, `I_mp` sont **régressés**, rendus tels quels |
| IEC 61853 | **Non calculé** | `P_max` est **mesuré**, interpolé dans la grille |

Le SAPM et l'interpolation IEC 61853 **rapportent** un MPP ; ils ne le
**calculent** pas. La distinction est portée par les types : le trait
`ContinuousIVModel` n'est pas implémenté par ces modèles, donc aucune
fonction générique exigeant une courbe ne les accepte — refus à la
compilation, vérifié par test `compile_fail`.

---

## 12. Tolérances — justification indépendante par grandeur

### 12.1 `tol_residual` (forme en courant)

Quatre contraintes chiffrées :

| Contrainte | Valeur |
|---|---|
| Plancher d'arrondi de l'évaluation de `f` | `≈ 10 ε I_{sc} = 2,22·10⁻¹⁵ I_{sc}` |
| Résolution des fiches (`I_sc` au centiampère) | `~10⁻³` relatif |
| Incertitude de mesure IEC 60904 | 2 à 3 % élargie |
| Borne d'erreur (§5.1) | `|ΔI| ≤ |f|` |

**Retenu : `τ_f = 10⁻¹² · I_sc`**, soit **450×** le plancher d'arrondi et
**10⁻¹⁰** de l'incertitude des données.

### 12.2 `tol_I` — dérivée, non indépendante

Par §5.1, `|ΔI| ≤ τ_f`. La tolérance sur le courant **est** celle du
résidu ; l'annoncer comme une seconde tolérance indépendante serait un
doublon trompeur.

### 12.3 `tol_V` — justification propre

Par §5.2, `|ΔV| ≤ R_sh · |h|`. Pour garantir `|ΔV| ≤ 10⁻¹² V_oc` il faut

$$ \tau_h = \frac{10^{-12}\,V_{oc}}{R_{sh}} $$

Seuil **distinct** de `τ_f`, dépendant de `R_sh`. Pour `V_oc = 45 V` et
`R_sh = 800 Ω` : `τ_h = 5,6·10⁻¹⁴ A`, soit environ 25 fois plus serré
que `τ_f`. Le retour du plancher d'arrondi doit donc être vérifié cas par
cas, et le seuil plafonné à `max(τ_h, 100 ε I_sc)`.

### 12.4 `tol_P` — dérivée

`P = V I`, donc `δP/P ≤ δV/V + δI/I ≤ 2·10⁻¹²`. Dérivée, énoncée comme
telle.

### 12.5 `tol_relative` contre `tol_absolue` — et le cas `I_sc → 0`

*Mesure* du comportement quand `I_sc` décroît :

| `I_sc` | `τ = 10⁻¹² I_sc` | Plancher `10 ε I_sc` | Marge | `log₂((I_L+I_0)/τ)` |
|---:|---:|---:|---:|---:|
| 20 A | 2,0·10⁻¹¹ | 4,4·10⁻¹⁴ | **450** | 40,4 |
| 1 A | 1,0·10⁻¹² | 2,2·10⁻¹⁵ | **450** | 40,4 |
| 10⁻⁴ A | 1,0·10⁻¹⁶ | 2,2·10⁻¹⁹ | **450** | 40,4 |
| 10⁻⁸ A | 1,0·10⁻²⁰ | 2,2·10⁻²³ | **450** | 40,4 |
| 10⁻²⁰ A | 1,0·10⁻³² | 2,2·10⁻³⁵ | **450** | 40,4 |

**La marge et le nombre d'itérations sont invariants** : le plancher
d'arrondi décroît exactement comme la tolérance, puisque tous deux sont
proportionnels au plus grand terme de `f`, lui-même proportionnel à
`I_sc`. La mise à l'échelle relative est donc **auto-cohérente jusqu'aux
dénormaux**.

Le seul cas dégénéré est `G = 0` exactement, traité par un **prédicat
exact** (tous les points nuls, solveur non appelé), et non par un seuil.

**Décision : tolérance relative pure, prédicat exact en `G = 0`, aucun
seuil arbitraire.**

### 12.6 Impact sur le nombre d'itérations

Newton : convergence quadratique, le nombre d'itérations varie comme
`log log(1/τ)` — passer de `10⁻⁸` à `10⁻¹²` coûte **une** itération.
Mesuré : 2 à 13 itérations pour `τ = 10⁻¹⁴ A`.

Repli : `log₂(1/τ)`, invariant à 40,4 (§9.4).

---

## 13. Phang — correction (A-4)

### 13.1 Ce qui était affirmé

« Phang (1984) fournit l'estimation initiale du système 5×5 de De Soto »
depuis une fiche constructeur.

### 13.2 Ce qui est vrai

La méthode de Phang, Chan & Phillips (*Electronics Letters* 20(10),
406–408, 1984) calcule explicitement les cinq paramètres à partir de
`I_sc`, `V_oc`, `I_mp`, `V_mp` **et de deux pentes** :

- `R_{so} = -\left.\frac{dV}{dI}\right|_{V=V_{oc}}`
- `R_{sho} = -\left.\frac{dV}{dI}\right|_{I=I_{sc}}`

**Ces deux pentes ne figurent sur aucune fiche constructeur.** Elles se
lisent sur une courbe I-V mesurée.

**La méthode est donc inapplicable au cas « fiche seule »**, qui est
précisément celui pour lequel je l'avais proposée. L'affirmation est
corrigée.

### 13.3 Ce qui est retenu

| Cas | Initialisation |
|---|---|
| Courbe I-V mesurée disponible | **Phang** — les pentes sont mesurables, la méthode est applicable et bien fondée |
| Fiche seule | **Estimation heuristique documentée** (§13.4) |

### 13.4 Distinction essentielle : point de départ ≠ résultat

Un point de départ arbitraire n'est **pas** un paramètre arbitraire.

C'est ce qui sépare l'estimation initiale de la méthode de Villalva :
Villalva **fixe** `n` et le conserve dans le résultat ; une estimation
initiale ne fait que **démarrer** l'itération, et les cinq équations
déterminent ensuite la solution.

**Garde-fou exigé** : la solution convergée est **vérifiée contre les
cinq contraintes**, et rejetée si l'une d'elles est violée au-delà de la
tolérance. L'influence du point de départ est ainsi éliminée par
construction — et non par confiance.

### 13.5 Stratégie d'échec — le moteur n'est jamais invalidé

Le système 5×5 **n'a aucune garantie de convergence** : aucune propriété
de convexité n'y a été établie, contrairement à l'équation I-V.

**Conséquence exigée** : l'échec de l'identification ne doit jamais
invalider le moteur.

| Échec | Réponse | Effet |
|---|---|---|
| Non-convergence | `IDENTIFICATION_FAILED` avec résidus et dernier itéré | Les modèles ne requérant pas d'identification (`PublishedAsIs`) restent **pleinement utilisables** |
| Solution non physique (`R_s < 0`, `R_sh < 0`) | Rejet | Idem |
| Contraintes violées après convergence | Rejet | Idem |
| `β_Voc` absent | `INSUFFICIENT_DATA` | Idem |

L'identification est un **service**, pas un passage obligé. Un
enregistrement CEC n'en a aucun besoin.

---

## 14. `E_g` — modèle CEC contre propriété du matériau

### 14.1 Pourquoi les paramètres CEC portent les hypothèses du CEC

Les six paramètres CEC sont le résultat d'un **ajustement**, et tout
ajustement est solidaire des constantes employées pendant l'ajustement.
Le CEC — comme les implémentations de référence qui le reproduisent —
emploie `E_{g,ref} = 1{,}121\ \mathrm{eV}` et
`dE_g/dT = -2{,}677\cdot10^{-4}` **pour tous les modules**, technologie
comprise.

Un jeu CEC est donc **auto-cohérent avec ces constantes**, et avec elles
seules.

### 14.2 Pourquoi ce n'est pas une valeur universelle

1,121 eV est le gap du **silicium**. Pour un module CdTe, ce n'est pas le
gap du matériau : c'est un **coefficient d'ajustement** qui se trouve
porter le nom d'une grandeur physique.

Retraduire un jeu CEC avec le vrai gap du CdTe mélangerait deux cadres et
produirait une incohérence — sans qu'aucun symptôme visible n'apparaisse.

### 14.3 Comment HeliosPro évite le mélange

**Par les types.** Deux grandeurs distinctes, jamais interchangeables :

| Grandeur | Nature | Classe | Appartient à |
|---|---|---|---|
| **Constante d'ajustement du modèle** | Convention de la source | **F** (ajustée) | Le **jeu de paramètres** |
| **Gap intrinsèque du matériau** | Propriété physique | **B** ou **C** | Une table de matériaux, sourcée |

Trois règles en découlent :

1. Un jeu de paramètres **porte** sa constante d'ajustement ; la
   translation en température utilise **celle-là**, jamais une valeur
   récupérée ailleurs.
2. Quand la technologie n'est pas du silicium et que la source est le
   CEC, le moteur **signale** que la constante est une convention
   d'ajustement, sans la corriger — corriger casserait l'auto-cohérence.
3. Le gap intrinsèque n'est utilisé que pour une identification **menée
   par nous**, et doit alors être **sourcé** ; à défaut,
   `INSUFFICIENT_DATA`.

**Le moteur n'attribue jamais silencieusement une valeur silicium à une
technologie non silicium.** Et `E_g` se sélectionne sur le **matériau**,
jamais sur l'architecture : HJT, TOPCon, PERC et IBC sont du silicium.

---

## 15. IEC 61853 — cohérence du classement

### 15.1 Ce qui est mesuré, ce qui est interpolé

| Grandeur | Statut |
|---|---|
| `P_max` aux nœuds de la grille | **Mesurée** — classe B |
| `I_sc`, `V_oc`, `I_mp`, `V_mp` aux nœuds, si le rapport les donne | **Mesurées** — classe B |
| Toute valeur **entre** les nœuds | **Interpolée** — classe D (dérivée) |
| Toute valeur **hors** de la grille | **Interdite** |

Le classement comme modèle à part entière est cohérent : il **transforme**
des données d'entrée en une performance à conditions données, ce qui est
la définition d'un modèle. Sa singularité est de n'avoir **aucun
paramètre ajusté**, donc aucune classe F — d'où l'absence d'erreur de
modèle, remplacée par une erreur d'interpolation.

### 15.2 Ce qui est autorisé entre les points

L'interpolation doit être **nommée** et son erreur **caractérisée**. Un
point mérite attention : `P` n'est pas linéaire en `G` (il croît à peu
près linéairement avec une correction logarithmique via `V_oc`). Une
interpolation bilinéaire naïve sur `P` introduirait une erreur
systématique.

**Piste retenue, à confirmer sur données réelles** : interpoler
l'efficacité `η = P/(G·A)` plutôt que `P`, quantité bien plus plate en
`G`, donc mieux approchée linéairement. Décision différée jusqu'à
disposer d'une matrice réelle — **aucune méthode n'est gravée sur une
intuition**.

### 15.3 Hors domaine

Le domaine est l'**enveloppe convexe** des nœuds de la grille. Toute
demande hors de cette enveloppe reçoit `OUT_OF_DOMAIN`, avec les bornes
du domaine et le point demandé.

**Aucune extrapolation, silencieuse ou non.** Contrairement aux modèles
physiques, ce modèle n'a aucun fondement hors du domaine mesuré : y
répondre serait inventer.

---

## 16. Jeux de tests scientifiques à préparer

Jeux **synthétiques**, construits pour être cohérents, et documentés comme
tels — ils ne prétendent représenter aucun produit réel. Les neuf
premiers ont déjà servi à la vérification numérique du présent audit.

| # | Cas | `I_L` (A) | `I_0` (A) | `R_s` (Ω) | `R_sh` (Ω) | `a` (V) | Vérifie |
|---|---|---|---|---|---|---|---|
| T-1 | Nominal 72 cellules | 10,4 | 2,5e-10 | 0,30 | 800 | 1,85 | Référence |
| T-2 | `R_s = 0` | 10,4 | 2,5e-10 | **0** | 800 | 1,85 | Forme fermée, §6.1 |
| T-3 | `R_s → 0` | 10,4 | 2,5e-10 | **1e-9** | 800 | 1,85 | Absence de seuil, §6.2 |
| T-4 | `R_s` élevé | 10,4 | 2,5e-10 | **2,0** | 800 | 1,85 | Convergence lente (13 iters) |
| T-5 | `R_sh → ∞` | 10,4 | 2,5e-10 | 0,30 | **1e12** | 1,85 | Limite sans fuite |
| T-6 | `R_sh` faible | 10,4 | 2,5e-10 | 0,30 | **5,0** | 1,85 | Facteur de forme effondré |
| T-7 | `R_sh` très faible | 10,4 | 2,5e-10 | 0,30 | **1,5** | 1,85 | Unimodalité (§10.6) |
| T-8 | `G` très faible | **0,0104** | 2,5e-10 | 0,30 | **8e5** | 1,85 | `I_sc → 0`, §12.5 |
| T-9 | Couche mince | 1,9 | 1e-11 | 3,0 | 2000 | **4,0** | `V_oc` élevée, `u_max` |
| T-10 | `G = 0` | **0** | 2,5e-10 | 0,30 | ∞ | 1,85 | Prédicat exact, solveur non appelé |
| T-11 | `G` élevé (1400 W/m²) | 14,6 | 2,5e-10 | 0,30 | 570 | 1,85 | Au-dessus de STC |
| T-12 | Température basse (−40 °C) | 10,1 | dérivé | 0,30 | 800 | 1,44 | `a ∝ T`, `u` plus grand |
| T-13 | Température élevée (85 °C) | 10,7 | dérivé | 0,30 | 800 | 2,22 | `I_0` très augmenté |
| T-14 | `I_sc` faible | **0,5** | 2,5e-10 | 0,30 | 16000 | 1,85 | Invariance de la tolérance |
| T-15 | Débordement potentiel | 10,4 | 2,5e-10 | 0,30 | 800 | **0,05** | `u_max ≫ 709` → **refus a priori** |
| T-16 | `I_0 ≤ 0` | 10,4 | **0** | 0,30 | 800 | 1,85 | Paramètre non physique → refus |
| T-17 | `R_sh ≤ 0` | 10,4 | 2,5e-10 | 0,30 | **−800** | 1,85 | Idem |
| T-18 | `a ≤ 0` | 10,4 | 2,5e-10 | 0,30 | 800 | **0** | Idem |
| T-19 | Convergence rapide | 10,4 | 2,5e-10 | 0,05 | 5000 | 1,85 | ≤ 4 itérations attendues |
| T-20 | MPP proche de `V_oc` | 10,4 | 2,5e-10 | 0,02 | 1e6 | 1,85 | `V_mp/V_oc` élevé |
| T-21 | MPP proche de zéro | 10,4 | 2,5e-10 | 0,30 | **0,8** | 1,85 | `V_mp/V_oc` faible |
| T-22 | Non-physicité combinée | −1,0 | −1e-10 | −0,3 | −800 | −1,0 | Refus intégral |

Pour chacun, au moins **trois scénarios** conformément à la discipline
établie sur `pv_module` : cas cohérent, cas violé, cas limite.

**Aucun de ces jeux ne prétend décrire un module réel.** Les cinq fiches
constructeurs et la grille IEC 61853-1 restent ouvertes et ne sont pas
simulées.

---

## 17. Purge des formulations non prouvées

Conformément au critère de sortie, aucune formulation absolue ne subsiste
sans preuve correspondante.

| Formulation | Sort |
|---|---|
| « Newton converge **depuis tout point de départ** » | **Retirée** — remplacée par la garantie conditionnée du §4.3 |
| « Brent : borne d'itérations de **81** » | **Retirée** — remplacée par 82, borne du schéma spécifié (§9.3) |
| « convergence **garantie** » (Newton, sans condition) | **Reformulée** : garantie *sous les hypothèses de §1 et depuis `I₀ = 0`* |
| « l'unicité du MPP **n'est pas démontrée** » | **Retirée** — l'unicité est démontrée (§10.4) |
| « Phang initialise depuis une **fiche** » | **Corrigée** — Phang exige les pentes (§13.2) |
| « le résidu majore l'erreur » | **Conservée** pour la forme en courant ; **inapplicable** en tension (§5.2) |

Toute formulation absolue restant dans les documents est désormais
accompagnée soit d'une démonstration, soit d'une condition explicite.

---

## 18. Bilan de l'audit

| Critère de sortie | État |
|---|---|
| Garanties mathématiques prouvées **ou** retirées | ✅ — 2 retirées, 1 gagnée, 2 corrigées |
| Aucun « toujours / garanti / depuis tout point de départ » sans preuve | ✅ — §17 |
| Domaine initial garanti documenté | ✅ — §4.3, `I₀ = 0`, itérés dans `[I*, I_L]` |
| Stratégie de débordement sans `try/catch` | ✅ — §7, borne a priori + domaine logarithmique |
| `R_s = 0` formalisé, `R_s → 0` sans seuil | ✅ — §6 |
| MPP : existence, unicité, domaine, bornes | ✅ — §10, démontré |
| MPP jamais exposé pour les modèles à points | ✅ — §11 |
| Tolérances justifiées indépendamment | ✅ — §12, quatre grandeurs distinguées |
| Borne d'itérations dérivée, indépendante de la machine | ✅ — §9, 82 démontrée, invariance vérifiée |
| Phang documenté, échec non bloquant | ✅ — §13 |
| `E_g` : modèle CEC ≠ propriété du matériau | ✅ — §14 |
| IEC 61853 : mesuré / interpolé / interdit | ✅ — §15 |
| Jeux de tests préparés | ✅ — §16, 22 jeux |
| **Aucun code métier** | ✅ — `electrical_model` reste le squelette de 31 lignes |

---

## 19. Ce qui reste non démontré

| Point | Statut |
|---|---|
| Convergence du système 5×5 d'identification | **Non démontrée** — d'où la stratégie d'échec du §13.5 |
| Adéquation des modèles à des modules réels | Impossible sans mesures I-V de terrain |
| Méthode d'interpolation IEC 61853 optimale | Différée jusqu'à disposer d'une matrice réelle |
| Tolérances confirmées sur données réelles | Dérivées par analyse ; campagne de sensibilité à mener |
| `E_g` des matériaux non silicium | À sourcer ; aucune valeur retenue |
| Grille IEC 61853-1 | À lire dans le texte normatif |
| Cinq fiches constructeurs réelles | **À ne pas simuler** |

Ces points sont **explicitement** hors des garanties annoncées. Aucun
n'est présenté comme acquis.
