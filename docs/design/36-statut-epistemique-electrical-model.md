# 36. Statut épistémique, protocole de validation et tests de propriétés — `electrical_model`

**Date** : 2026-08-12
**Phase** : 2B — formalisation
**Statut** : **aucun code métier**
**Complète** : `docs/design/35-formalisation-mathematique-electrical-model.md`

> **Principe absolu.** Une propriété observée n'est pas une preuve. Une
> preuve hors domaine de validité n'est pas une garantie. Une garantie
> mathématique doit être traduite en invariant logiciel testable.
>
> Ce document existe pour rendre ce principe **vérifiable** : chaque
> énoncé du projet y reçoit un statut, et aucun énoncé ne peut changer de
> catégorie sans ADR.

---

## 1. Les quatre statuts

| Statut | Définition | Ce qui l'établit | Ce qui le détruit |
|---|---|---|---|
| **[P] Prouvé** | Démontré analytiquement sous des hypothèses nommées | Une démonstration écrite, relisible | Une erreur dans la démonstration |
| **[O] Observé** | Constaté numériquement sur un échantillon fini | Une mesure reproductible | Un contre-exemple |
| **[H] Hypothèse** | Admis sans démonstration, **par choix explicite** | Une décision documentée | Rien — c'est une hypothèse |
| **[E] Empirique** | Requiert une confrontation à des données réelles | Des mesures externes | Un désaccord avec le réel |

**Règle de non-promotion** :

> Une propriété **[O]** ne devient jamais **[P]** par accumulation de cas.
> Mille jeux de paramètres ne font pas une démonstration.

---

## 2. Tableau des hypothèses — **[H]**

### 2.1 Hypothèses mathématiques — vérifiables par le moteur

Ensemble `𝓗`, à vérifier **avant tout calcul** :

| # | Hypothèse | Vérifiable ? | Invariant logiciel |
|---|---|---|---|
| H-1 | `I_ph > 0` et fini | **Oui** | `INV-EM-DOM-01` |
| H-2 | `I_0 > 0` et fini | **Oui** | `INV-EM-DOM-02` |
| H-3 | `a > 0` et fini | **Oui** | `INV-EM-DOM-03` |
| H-4 | `R_sh > 0` et fini | **Oui** | `INV-EM-DOM-04` |
| H-5 | `R_s ≥ 0` et fini | **Oui** | `INV-EM-DOM-05` |
| H-6 | `N_s ≥ 1`, entier | **Oui** | `INV-EM-DOM-06` |
| H-7 | `T_c > 0` en **kelvins** | **Oui** | `INV-EM-DOM-07` |
| H-8 | `V ∈ [0, V_oc]` (quadrant de fonctionnement) | **Oui** | `INV-EM-DOM-08` |
| H-9 | Garde anti-débordement (17) respectée | **Oui** | `INV-EM-DOM-09` |

**Chacune est un refus, pas un avertissement.** Hors de `𝓗`, aucune
propriété **[P]** du §3 ne tient, et le moteur ne doit rien tenter : il
rendrait un point plausible et faux.

### 2.2 Hypothèses de modélisation — non vérifiables par le moteur

| # | Hypothèse | Origine | Pourquoi non vérifiable |
|---|---|---|---|
| H-10 | Le circuit à diode unique décrit le dispositif | Physique du modèle | Exige des mesures I-V réelles |
| H-11 | `R_s` est constant avec `G` et `T` | De Soto 2006 | Hypothèse du modèle, pas un fait |
| H-12 | `R_sh ∝ 1/G` | De Soto 2006 | Idem |
| H-13 | `a ∝ T_c` | De Soto 2006 | Idem |
| H-14 | `E_g(T)` suit la loi linéaire du silicium | De Soto 2006 | Spécifique au matériau (ADR-0010) |
| H-15 | Le module est électriquement uniforme (pas de mismatch) | Hypothèse de tout modèle à une diode | Faux sous ombrage partiel |

> **H-10 à H-15 ne sont ni prouvées ni prouvables dans le moteur.** Elles
> doivent être **déclarées** avec chaque résultat, jamais tues. Un
> résultat obtenu sous H-15 dans une situation d'ombrage est faux sans
> qu'aucun test interne ne puisse le détecter.

---

## 3. Tableau des propriétés PROUVÉES — **[P]**

Chacune renvoie à sa démonstration et à l'invariant testable qui la
traduit.

| # | Propriété | Démonstration | Invariant logiciel |
|---|---|---|---|
| P-01 | `f ∈ C^∞(ℝ)`, définie partout | doc 35 §2.1 | — (structurel) |
| P-02 | Équation (1) homogène en ampères | doc 35 §1.4 | `INV-EM-DIM-01` (typage) |
| P-03 | `[a] = V`, argument de `exp` sans dimension | doc 35 §1.4 | `INV-EM-DIM-02` |
| P-04 | `∂f/∂I ≤ −1 < 0` | doc 35 §2.2 | `INV-EM-DER-01` |
| P-05 | `∂f/∂I` **sans dimension** | doc 35 §2.2 | `INV-EM-DIM-03` |
| P-06 | `∂²f/∂I² < 0` si `R_s > 0` ; `= 0` si `R_s = 0` | doc 35 §2.3 | `INV-EM-DER-02` |
| P-07 | `∂f/∂V < 0`, en Ω⁻¹ | doc 35 §2.4 | `INV-EM-DIM-04` |
| P-08 | **Unicité** de `I*` | doc 35 §3.2 | `INV-EM-SOL-01` |
| P-09 | **Existence** de `I*` | doc 35 §3.3 | `INV-EM-SOL-02` |
| P-10 | `I* ∈ [0, I_ph + I_0]` pour `V ∈ [0,V_oc]` | doc 35 §3.4 | `INV-EM-SOL-03` |
| P-11 | `I(V) ∈ C^∞`, strictement décroissante | doc 35 §3.5 | `INV-EM-SOL-04` |
| P-12 | Newton converge dans `ℝ` (décroissance + concavité) | doc 35 §4.2 | — (hors `f64`) |
| P-13 | **Invariant d'intervalle** : depuis `I₀=0`, `I_k ∈ [I*, I_ph]` | doc 35 §4.4 | `INV-EM-NEW-01` |
| P-14 | `u ≤ u_max = (V_oc + I_ph R_s)/a` | doc 35 §4.4 | `INV-EM-OVF-01` |
| P-15 | `|I − I*| ≤ |f(I)|` | doc 35 §5.1 | `INV-EM-ERR-01` |
| P-16 | Cohérence dimensionnelle de P-15 | doc 35 §5.2 | `INV-EM-DIM-05` |
| P-17 | `|V − V*| ≤ R_sh·|h(V)|` — **forme différente** | doc 35 §5.4 | `INV-EM-ERR-02` |
| P-18 | Changement de signe sur `[0, I_ph+I_0]` | doc 35 §6.2 | `INV-EM-FBK-01` |
| P-19 | **Borne 82** du repli à bissection imposée | doc 35 §6.3 | `INV-EM-FBK-02` |
| P-20 | Borne indépendante du module et de la machine | doc 35 §6.3 | `INV-EM-FBK-03` |
| P-21 | Garde (17) suffisante, seuil IEEE 754 | doc 35 §7.2 | `INV-EM-OVF-02` |
| P-22 | Aucun sous-flux dangereux | doc 35 §7.6 | `INV-EM-OVF-03` |
| P-23 | Solution exacte pour `R_s = 0` | doc 35 §8.1 | `INV-EM-RS0-01` |
| P-24 | **Aucune division par `R_s`** dans la branche générale | doc 35 §8.2 | `INV-EM-RS0-02` |
| P-25 | `φ' < 0`, `φ'' < 0` | doc 35 (22),(23) | `INV-EM-MPP-01` |
| P-26 | `V ↔ V_j` bijection croissante | doc 35 (24) | `INV-EM-MPP-02` |
| P-27 | `ψ' > 0` — étape 1 | doc 35 §9.4 | `INV-EM-MPP-03` |
| P-28 | `{ψ ≥ 0}` est un intervalle — étape 2 | doc 35 §9.4 | `INV-EM-MPP-04` |
| P-29 | Aucun point critique dans `{ψ < 0}` — étape 3 | doc 35 §9.4 | `INV-EM-MPP-05` |
| P-30 | `d²P/dV_j² < 0` sur `{ψ ≥ 0}` — étape 4 | doc 35 §9.4 | `INV-EM-MPP-06` |
| P-31 | Au moins un point critique — étape 5 | doc 35 §9.4 | `INV-EM-MPP-07` |
| P-32 | Au plus un point critique — étape 6 | doc 35 §9.4 | `INV-EM-MPP-08` |
| P-33 | **MPP existe et est unique** sous `𝓗` | doc 35 §9.5 | `INV-EM-MPP-09` |
| P-34 | `dP/dV_j` **explicite** — aucun solveur imbriqué | doc 35 §11.2 | `INV-EM-MPP-10` |
| P-35 | Tolérances `τ_f`, `τ_h`, `τ_P` dérivées | doc 35 §13 | `INV-EM-TOL-01..03` |

**35 propriétés prouvées, 35 invariants logiciels correspondants.**

---

## 4. Tableau des propriétés SEULEMENT OBSERVÉES — **[O]**

> **Aucune de ces lignes n'est un théorème.** Elles corroborent, elles
> détectent des erreurs de dérivation, elles ne démontrent rien.

| # | Observation | Échantillon | Ce qu'elle ne dit pas |
|---|---|---|---|
| O-01 | `min|∂f/∂I| ∈ [1,000 ; 1,200]` | 9 jeux × 5 151 couples | Que P-04 vaut hors échantillon — P-04 le démontre |
| O-02 | `max ∂²f/∂I² ∈ [−2,9·10⁻¹⁰ ; 0]` | idem | Idem pour P-06 |
| O-03 | Newton : **2 à 13 itérations** depuis `I₀ = 0` | 9 jeux | **Aucune borne** — le budget de 30 n'est pas démontré, et O-16 montre qu'il est dépassé hors de ces 9 jeux |
| O-04 | Newton déborde depuis `I₀ = 10⁴` | 1 contre-exemple | Suffit à **réfuter** « depuis tout point de départ » |
| O-05 | `u_max ∈ [10,1 ; 35,7]` | 9 jeux | Que `u_max` soit borné sur tout `𝓗` — il ne l'est pas, d'où la garde (17) |
| O-06 | Garde (17) : marge `≥ 696` unités log | 9 jeux | Idem |
| O-07 | Garde (17) **refuse** `a = 0,05` | 1 jeu construit | Que la garde soit optimale |
| O-08 | **Exactement 1 extremum** de `P` | 9 jeux × 4 001 points | **RIEN** : une grille ne peut exclure deux racines rapprochées. C'est P-33 qui démontre l'unicité |
| O-09 | Étapes 1, 3, 4 vérifiées point par point | idem | Idem — corroboration seulement |
| O-10 | `R_s = 0` et `R_s = 10⁻⁹` : 2 itérations, écart `< 10⁻⁹` | 2 jeux | Que ce soit vrai pour tout `R_s` petit |
| O-11 | `dI/dV` analytique vs différences : écart `2·10⁻¹²` à `1,2·10⁻⁹` | 9 jeux | La justesse de (10) — l'écart mesure les **différences finies** |
| O-12 | `dP/dV_j` analytique vs différences : `2,4·10⁻¹¹` à `1,5·10⁻¹⁰` | 9 jeux | Idem pour (27) |
| O-13 | MPP en `V_j` vs balayage : écart `\le 7,6·10⁻⁴ V` | 9 jeux | Une erreur de la méthode — c'est la **résolution du balayage** |
| O-14 | Marge au plancher = **450**, invariante | `I_sc` de 20 A à 10⁻²⁰ A | Que ce soit vrai en dessous des dénormaux |
| O-15 | `\log_2(W_0/τ_f) = 40,4` invariant | idem | — (P-20 le démontre) |
| O-16 | Budget Newton dépassé dans **0,642 %** du domaine §7.1, pire décompte **218**, prédiction `(I_ph − I*)·R_s/a` = 217,1 | 200 000 jeux | Que 218 soit une borne. L'estimation est **asymptotique** dans le régime exponentiel et ne dit rien du régime de transition |
| O-17 | Le critère d'arrêt **en largeur** du repli n'est **jamais** atteint ; pire décompte 31 sur 82 | 237 000 cas, dont 76 535 raides avec `V` à `10⁻¹⁶` de `V_oc` | Qu'il soit inatteignable. Il est conservé parce que la démonstration de la borne 82 raisonne sur la largeur |

**17 observations. Aucune n'est promue.**

> **O-16 et O-17 sont nées de l'implémentation**, pas de la
> formalisation, et elles corrigent l'une et complètent l'autre. O-16
> réfute la lecture générale de O-03 ; O-17 explique pourquoi
> `solver.rs` conserve des lignes non couvertes. Voir ADR-0012 D-5 et
> D-6, et la doc 37 §5.

### 4.1 Le cas O-08 mérite d'être isolé

C'est l'observation la plus tentante à confondre avec une preuve :
« 36 009 points, toujours un seul extremum ». **Elle ne prouve rien.**
Un balayage à pas fini ne peut pas exclure deux racines séparées de moins
d'un pas. L'unicité vient de P-33, et de rien d'autre.

Inversement, si le balayage avait trouvé deux extrema, **cela aurait
réfuté** la démonstration et imposé d'en chercher l'erreur. C'est le rôle
d'une corroboration : elle ne confirme pas, elle peut infirmer.

---

## 5. Tableau des validations EMPIRIQUES requises — **[E]**

| # | Ce qui doit être confronté au réel | Données requises | Statut |
|---|---|---|---|
| E-01 | Le modèle décrit-il un module réel ? | Courbes I-V mesurées | **Aucune donnée** |
| E-02 | Les tolérances sont-elles adaptées ? | Balayage CEC/Sandia complet | Non menée |
| E-03 | Les cinq fiches constructeurs | Fiches réelles | **Ouvert — à ne pas simuler** |
| E-04 | La grille IEC 61853-1 | Texte normatif | **Ouvert** |
| E-05 | `E_g` des matériaux non silicium | Bibliographie | **Ouvert** |
| E-06 | Méthode d'interpolation IEC 61853 optimale | Matrice réelle | **Ouvert** |
| E-07 | Convergence du système 5×5 d'identification | Population réelle | **Non démontrée** (doc 34 §19) |
| E-08 | Accord avec pvlib | pvlib | Non menée |

> **Aucune propriété [E] n'est acquise à ce jour.** Le moteur peut être
> implémenté et testé mathématiquement sans elles ; il ne peut pas être
> déclaré **validé physiquement** sans elles.

---

## 6. Protocole de validation

### 6.1 Quatre natures de validation, à ne jamais confondre

| Nature | Question posée | Vérité de référence | Indépendance |
|---|---|---|---|
| **Mathématique** | Les démonstrations sont-elles correctes ? | Relecture, cohérence interne, corroboration numérique | Totale |
| **Numérique** | L'implémentation résout-elle ce qu'elle prétend ? | Solutions analytiques, résidus, croisement de méthodes | Totale |
| **Physique** | Le modèle est-il cohérent avec la physique attendue ? | Invariants de conservation, signes, monotonies | Totale |
| **Contre données réelles** | Le modèle décrit-il un module réel ? | Mesures | **Nulle** — dépend de données absentes |

**Les trois premières sont réalisables aujourd'hui. La quatrième ne
l'est pas.** Les confondre reviendrait à présenter une vérification
interne comme une validation physique.

### 6.2 Les sept familles

| Famille | Nature | Contenu |
|---|---|---|
| **A — Cas analytiques** | Numérique | `R_s = 0` (18) ; `R_sh → ∞` ; `I_0 → 0` ; `V = 0` ; `I = 0` |
| **B — Cas synthétiques** | Numérique | Aller-retour : paramètres imposés → courbe → paramètres ré-identifiés |
| **C — Cas CEC** | Physique + données réelles | Balayage de la bibliothèque, distribution des résidus |
| **D — Cas Sandia** | Physique + données réelles | Idem pour le SAPM |
| **E — Cas limites** | Numérique | Les 22 jeux de la doc 34 §16 |
| **F — Cas pathologiques** | Numérique | Hors `𝓗` : refus attendu, jamais un résultat |
| **G — Comparaison pvlib** | Contre données réelles | Divergence à **expliquer**, jamais à absorber |

### 6.3 Position sur les oracles — inchangée

> Ni pvlib ni PVsyst ne sont un oracle absolu.

Un écart signale une divergence à expliquer, **dans un sens ou dans
l'autre**. Jamais une erreur présumée de notre côté. Jamais une tolérance
élargie jusqu'à ce que l'écart rentre.

La famille A est la plus forte et la plus négligée : `R_s = 0` donne une
solution exacte qu'aucun oracle n'est requis pour vérifier.

---

## 7. Tests de propriétés (property-based)

### 7.1 Domaine de génération

Les générateurs tirent dans `𝓗`, avec des plages couvrant les
technologies connues :

| Paramètre | Plage de génération | Distribution |
|---|---|---|
| `I_ph` | `[10⁻³, 30]` A | log-uniforme |
| `I_0` | `[10⁻¹⁵, 10⁻⁶]` A | log-uniforme |
| `R_s` | `[0, 5]` Ω | uniforme, **avec `0` tiré explicitement** |
| `R_sh` | `[1, 10¹²]` Ω | log-uniforme |
| `a` | `[0,5, 10]` V | uniforme |
| `V` | `[0, V_oc]` | uniforme, bornes incluses |

Les jeux violant la garde (17) sont **rejetés à la génération**, non au
calcul : le test porte sur le domaine où les propriétés sont démontrées.

Un second générateur tire **hors `𝓗`** pour la famille F.

### 7.2 Propriétés à vérifier

| # | Propriété | Traduit | Statut vérifié |
|---|---|---|---|
| **PB-01** | `I(V)` existe et est unique : deux méthodes indépendantes coïncident à `τ_f` | P-08, P-09 | **[P]** |
| **PB-02** | `I(V)` strictement décroissante sur `[0, V_oc]` | P-11 | **[P]** |
| **PB-03** | `I(V) ∈ [0, I_sc]` et `I(V_oc) = 0` à `τ_f` | P-10 | **[P]** |
| **PB-04** | `I(V)` continue : `|I(V+δ) − I(V)| ≤ L·δ` avec `L` borné par (10) | P-11 | **[P]** |
| **PB-05** | **Aucun `NaN`, aucun `±∞`** en sortie, jamais | — | Invariant absolu |
| **PB-06** | `P = V·I` à `2·10⁻¹²` relatif | P-35 | **[P]** |
| **PB-07** | Le MPP existe | P-31 | **[P]** |
| **PB-08** | Le MPP est unique : deux départs distincts convergent au même point à `τ_{V_j}` | P-33 | **[P]** |
| **PB-09** | `P(V_mp) ≥ P(V)` pour tout `V` tiré | P-33 | **[P]** |
| **PB-10** | `R_s = 0` → égalité avec (18) à `1 ulp`, 1 itération | P-23, P-24 | **[P]** |
| **PB-11** | `R_sh → ∞` → convergence vers la forme sans fuite | P-11 | **[P]** |
| **PB-12** | `G → 0` : `I_sc → 0`, `V_oc → 0`, `P → 0`, sans `NaN` | — | Continuité |
| **PB-13** | `G = 0` exactement : sortie nulle, **solveur non appelé** | — | Prédicat exact |
| **PB-14** | `|f(I_rendu)| ≤ τ_f`, donc `|I − I*| ≤ τ_f` | P-15 | **[P]** |
| **PB-15** | Itérations `≤ 82` en toute circonstance | P-19 | **[P]** |
| **PB-16** | `∂f/∂I ≤ −1` en tout point évalué | P-04 | **[P]** |
| **PB-17** | Invariant d'intervalle : `I_k ∈ [0, I_ph+I_0]` à chaque itération | P-13 | **[P]** |
| **PB-18** | Hors `𝓗` : **refus**, jamais un résultat | H-1..H-9 | Invariant absolu |
| **PB-19** | Déterminisme : deux appels identiques → bits identiques | — | Invariant absolu |
| **PB-20** | `ψ` strictement croissante sur le domaine | P-27 | **[P]** |
| **PB-21** | `dP/dV_j > 0` partout où `ψ < 0` | P-29 | **[P]** |
| **PB-22** | `d²P/dV_j² < 0` partout où `ψ ≥ 0` | P-30 | **[P]** |

**22 propriétés, dont 17 traduisent directement une propriété prouvée.**

### 7.3 Ce qu'un test de propriété peut et ne peut pas faire

| Peut | Ne peut pas |
|---|---|
| **Réfuter** une propriété prouvée → révéler une erreur de démonstration ou d'implémentation | **Démontrer** une propriété |
| Explorer un domaine bien plus large qu'une liste de cas | Couvrir un domaine continu |
| Trouver un contre-exemple minimal par réduction | Garantir l'absence de contre-exemple |

> **PB-08 est le plus instructif** : il vérifie l'unicité du MPP, qui est
> **démontrée**. S'il échouait, ce ne serait pas la propriété qui serait
> fausse — ce serait la démonstration ou le code. C'est exactement le rôle
> attendu.

### 7.4 Discipline de décompte

Le décompte de scénarios établi sur `pv_module` s'applique : **minimum
trois scénarios par propriété**, mesurés et non affirmés, matrice générée
et publiée en artefact, seuil protégé par la CI.

Pour les tests de propriétés, le décompte porte sur le **nombre de cas
générés** et sur la **couverture du domaine**, publiés avec la graine
utilisée pour que tout échec soit reproductible.

---

## 8. Récapitulatif

| Catégorie | Nombre | Signification |
|---|---|---|
| **[H]** Hypothèses vérifiables | 9 | Refus si violées |
| **[H]** Hypothèses de modélisation | 6 | Déclarées, jamais tues |
| **[P]** Propriétés prouvées | 35 | Chacune avec sa démonstration |
| **[O]** Observations numériques | 15 | **Aucune promue** |
| **[E]** Validations empiriques requises | 8 | **Aucune acquise** |
| Invariants logiciels | 35 | Un par propriété prouvée |
| Propriétés testables (PB) | 22 | 17 traduisent un **[P]** |

### 8.1 Ce que le moteur pourra légitimement affirmer après implémentation

> Sous les hypothèses `𝓗`, vérifiées à l'entrée et refusées sinon, le
> solveur rend l'unique solution de l'équation (1) avec une erreur
> majorée par la tolérance, en un nombre d'itérations borné par 82, et
> le point de puissance maximale est l'unique point critique de `P` sur
> `[0, V_oc]`.

### 8.2 Ce qu'il ne pourra pas affirmer

> Que le modèle décrit correctement un module photovoltaïque réel.

Cela relève de **[E]**, et aucune donnée ne le permet aujourd'hui.
