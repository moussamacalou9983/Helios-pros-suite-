# 33. Rapport scientifique de décision — `electrical_model`

**Date** : 2026-08-12
**Phase** : 2A — revue scientifique finale
**Objet** : consigner les décisions prises, leur fondement, et ce qu'elles coûtent
**Statut du code** : **inchangé — squelette, zéro logique métier**

---

## 1. Ce que cette phase devait produire

Fermer sept décisions ouvertes, sans hypothèse implicite, sans paramètre
arbitraire, sans valeur par défaut non justifiée, sans code métier.

**Les sept sont fermées.** Trois l'ont été en contredisant les
recommandations provisoires que j'avais moi-même formulées à la phase
précédente.

---

## 2. Les trois renversements

C'est le résultat principal de cette phase. Une revue qui n'aurait fait
que confirmer ses propres recommandations n'aurait rien vérifié.

### 2.1 Newton, et non Brent, porte la garantie

**Ce que je proposais** : Brent garantit la terminaison, Newton apporte la
vitesse.

**Ce que la démonstration établit** : pour `R_s > 0`, le résidu est
strictement décroissant (`f' ≤ −1`) **et strictement concave**
(`f'' < 0`). Pour une telle fonction, Newton converge **depuis tout point
de départ**, de façon monotone dès la première itération.

**Ce que cela change** : Newton n'a besoin d'aucune globalisation,
d'aucun amortissement de pas, d'aucune garde de domaine. Brent cesse
d'être la garantie mathématique et devient la réponse à la seule menace
réelle — le débordement de `exp` en `f64`, qui est une pathologie de
représentation, pas de mathématique.

**Ce que cela apporte en plus** : puisque `|f'| ≥ 1`, le résidu **majore
l'erreur** : `|I_n − I*| ≤ |f(I_n)|`. Le critère de convergence devient
une borne d'erreur garantie, et non une estimation.

**Et un bonus** : pour `R_s = 0`, le résidu devient affine. Le cas que je
signalais comme un piège est en réalité le plus simple de tous — solution
exacte, zéro itération.

### 2.2 Lambert W n'est pas implémentée

**Ce que je proposais** : Lambert W comme estimation initiale.

**Ce que l'analyse établit** : elle déborde au même seuil que Newton, donc
ne résout pas le seul problème numérique sérieux ; elle est singulière en
`R_s = 0`, cas qui a une solution exacte triviale ; elle souffre
d'annulation catastrophique (`W_0(θ) ≈ ln θ − ln ln θ`) précisément dans
le régime des modules réels ; et elle n'économiserait que deux ou trois
itérations d'une méthode dont la convergence est désormais démontrée.

**Ce qu'elle coûterait** : absente de la bibliothèque standard, la
politique « zéro dépendance » imposerait de l'implémenter **et de la
valider** — sa propre campagne de précision, son propre jeu de tests —
pour une primitive utilisée nulle part ailleurs dans le moteur.

**Décision** : écartée. Le critère posé était clair, et il tranche sans
ambiguïté.

### 2.3 Le MPP se cherche comme une racine, pas comme un maximum

**Ce que j'envisageais** : recherche par section dorée sur `P(V)`.

**Ce que l'analyse numérique établit** : près d'un maximum,
`P ≈ P_max − ½|P''|(V−V_mp)²`. Une erreur `δ` sur la position ne produit
qu'une erreur `O(δ²)` sur la valeur. Dès que `δ² < ε`, les valeurs
deviennent indiscernables et la position ne s'affine plus. **La précision
plafonne à `√ε ≈ 1,5·10⁻⁸`** — soit 0,75 µV sur un module de 50 V.

Cette limite est **intrinsèque au problème**, pas à la méthode : toute
recherche de maximum la subit.

**Décision** : résoudre `dP/dV = 0` comme recherche de racine, où
`g'(V_mp) ≠ 0` rend la pleine précision atteignable. Gain : **huit ordres
de grandeur** sur la localisation du MPP.

L'encadrement `[0, V_oc]` est garanti par le changement de signe.
L'unicité ne l'est pas — elle équivaut à l'unimodalité de `P`, qui peut
tomber pour `R_sh` très faible. Le point trouvé sera donc **vérifié**
comme maximum, et l'échec signalé.

---

## 3. La découverte non prévue

L'analyse dimensionnelle, demandée comme vérification de routine, a
révélé un piège qu'aucune relecture n'aurait attrapé.

### `α_Isc` n'a pas la même dimension dans les deux modèles

| Modèle | Équation | Dimension **imposée** |
|---|---|---|
| De Soto | `I_L,ref + α_Isc·ΔT` doit être en A | `α_Isc` en **A/K** — absolu |
| SAPM | `[1 + α_Isc·ΔT]` doit être sans dimension | `α_Isc` en **K⁻¹** — relatif |

Le rapport entre les deux vaut `I_sco ≈ 10 A`. **Confondre les deux
introduit une erreur d'un facteur dix**, sur une grandeur qui porte le
même nom et le même symbole dans les deux littératures.

Ce n'est pas une conjecture : c'est ce que l'homogénéité impose, et cela
concorde avec les sources — colonne CEC `alpha_sc` en A/°C, `αIsc` de
King *et al.* défini comme « normalized (1/°C) ». Le SAPM mélange en outre
les deux bases dans un même jeu de coefficients : `α_Isc` relatif,
`β_Voco` absolu.

**Ce que cela valide rétroactivement** : le type
`TemperatureCoefficient` portant sa `CoefficientBasis`, créé pour
`pv_module` parce que la base CEC mélangeait les unités, n'était pas une
précaution d'ingénieur méfiant. C'était une **nécessité dimensionnelle**,
que cette analyse démontre après coup.

Un second piège de même nature a été trouvé dans le SAPM : `C_3` est en
V⁻¹, contrairement à `C_0`, `C_1` et `C_2` qui sont sans dimension. Un
type « coefficient SAPM » unique et adimensionnel serait faux.

---

## 4. Les sept décisions

| # | Décision | Fermée par |
|---|---|---|
| **D-1** | Méthode numérique | Newton principal (convergence démontrée), forme fermée si `R_s = 0`, Brent en repli, bisection comme oracle de test. **Lambert W écartée** |
| **D-2** | Type de `a` | Type propre — `a + V_oc` doit être impossible à écrire |
| **D-3** | Modèle par défaut | **Le moteur ne choisit pas.** Il expose les modèles applicables avec leur statut |
| **D-4** | Seuil SAPM | Refus **hors de l'enveloppe de régression**, portée par `ValidityDomain` ; pas de constante universelle |
| **D-5** | `E_g` non-silicium | Propriété du **matériau** (jamais de l'architecture), appartient au **jeu de paramètres** (jamais au module) ; sourcée ou `INSUFFICIENT_DATA` |
| **D-6** | PVsyst | Différé, ADR dédié ; utilisable en comparaison, **jamais en oracle** |
| **D-7** | Tolérances | `10⁻¹²·I_sc` relatif ; Newton 30 ; **Brent 81, calculé** ; critère sur le seul résidu |

### Le cas `E_g` mérite un mot

La question posée incluait « `E_g` pour HJT/TOPCon ». La réponse est
**1,121 eV, celle du silicium** : HJT et TOPCon sont des architectures de
contact appliquées à du silicium cristallin. Le gap est une propriété du
matériau, pas de la manière de le contacter. C'est exactement
l'orthogonalité `CellTechnology` / `CellArchitecture` déjà inscrite dans
`pv_module`.

Et une subtilité découverte en chemin : les paramètres De Soto du CEC ont
été ajustés en supposant `E_g = 1,121 eV` **pour tous les modules**, y
compris CdTe. Les traduire ensuite avec le vrai gap du CdTe mélangerait
deux cadres. D'où la décision que **`E_g` appartient au jeu de
paramètres**, pas au module : le moteur signale l'incohérence sans la
corriger, car corriger casserait l'auto-cohérence du jeu.

---

## 5. Comment les tolérances ont été établies

Aucune valeur n'a été choisie par habitude. Chacune découle de quatre
contraintes chiffrées.

| Contrainte | Valeur |
|---|---|
| Plancher d'arrondi `f64` sur le résidu | `≈ 10 ε I_sc ≈ 2,2·10⁻¹⁵ A` pour `I_sc = 10 A` |
| Résolution des fiches constructeurs | `~10⁻³` en relatif |
| Incertitude de mesure IEC 60904 | 2 à 3 % élargie |
| Borne d'erreur du solveur | `|I_n − I*| ≤ |f(I_n)|` |

**Choix : `τ = 10⁻¹² · I_sc`**, qui place le seuil à **450× le plancher
d'arrondi** (terminaison sûre) et **10⁻¹⁰ de l'incertitude des données**
(numériquement indétectable).

La mise à l'échelle **relative** est ce qui rend ces deux rapports
**constants** : vérifié pour `I_sc` de 0,5 A, 10 A et 20 A, le rapport au
plancher vaut exactement 450 dans les trois cas, et le nombre d'étapes de
bisection exactement 40,4. Une tolérance absolue briserait cette
invariance.

**La borne de Brent est calculée, pas choisie** :
`2 × log₂((I_L+I_0)/τ) = 2 × 40,4 ≈ 81`.

---

## 6. Ce que cette revue n'a pas établi

Consigné pour que ces limites ne soient pas découvertes plus tard comme
des surprises.

| Limite | Portée |
|---|---|
| **Unicité du MPP** | Non démontrée. L'existence d'une racine de `dP/dV` l'est. D'où la vérification obligatoire |
| **Convergence du système 5×5 d'identification** | Non démontrée — aucune propriété de convexité n'a pu y être établie, contrairement à l'équation I-V. D'où l'estimation initiale par Phang et le traitement explicite de la non-convergence |
| **Adéquation des modèles au réel** | Impossible à établir sans mesures I-V de terrain, dont nous ne disposons pas |
| **Tolérances confirmées sur données réelles** | Les seuils sont **dérivés par analyse**, pas mesurés. Campagne de sensibilité à mener |
| **Grille IEC 61853-1** | Toujours ouverte — à lire dans le texte normatif |
| **Cinq fiches constructeurs réelles** | Toujours ouvertes — **à ne pas simuler** |
| **`E_g` des matériaux non silicium** | À sourcer avant tout usage ; aucune valeur retenue à ce stade |

Ces limites ne bloquent pas l'implémentation du solveur et du modèle De
Soto depuis données CEC. Elles bloquent la prétention à une validation
complète.

---

## 7. Ce que ces décisions coûtent

Par honnêteté sur les contreparties.

| Coût | Nature |
|---|---|
| Trois documents portent des recommandations renversées | Corrigées par ADR daté, non réécrites — conformément à la politique ADR |
| Refuser de choisir un modèle par défaut | Demande une décision à l'appelant là où un concurrent trancherait à sa place |
| Trois nouveaux types d'unités | `Resistance`, type propre pour `a`, type propre pour `E_g` |
| Deux diodes reporté | Meilleur à faible irradiance, mais non identifiable sans hypothèses injustifiées |
| Stratégie de solveur à trois niveaux | Plus complexe qu'une méthode unique ; à confiner derrière une interface simple |

---

## 8. Critère de sortie

| Exigence | État |
|---|---|
| Les 7 décisions ouvertes définitivement résolues | **Oui** — §4 |
| Aucune hypothèse implicite | Oui — les hypothèses restantes sont nommées au §6 |
| Aucun paramètre arbitraire | Oui — Villalva écartée pour cette raison précise |
| Aucune valeur par défaut non justifiée | Oui — `INSUFFICIENT_DATA` plutôt qu'un repli silicium |
| Aucun code métier | **Oui** — le crate reste le squelette de Phase 1 |
| Matrice comparative finale | doc 32 §1 |
| Décision architecturale | doc 32 §2, ADR-0009 D-1 |
| Méthode d'identification | doc 32 §3, ADR-0009 D-3 |
| Solveur | doc 32 §4, ADR-0009 D-4 |
| Lambert W | doc 32 §5, ADR-0009 D-5 |
| Politique `E_g(T)` | doc 32 §6, ADR-0009 D-6 |
| Tolérances | doc 32 §7, ADR-0009 D-7 |
| Données insuffisantes | doc 32 §8, ADR-0009 D-2 |
| Stratégie de validation | doc 32 §9 |
| Vérification dimensionnelle | doc 32 §10 |
| ADR | ADR-0009 |
| Traçabilité | doc 21, Partie D |
| Rapport de décision | Ce document |

---

## 9. Suite

**Arrêt demandé.** L'implémentation d'`electrical_model` n'est pas
entamée et ne le sera qu'après validation explicite.

Ce que cette phase laisse à l'implémentation : un solveur dont la
convergence est démontrée et les bornes calculées, une architecture où le
compilateur interdit de demander à un modèle ce qu'il ne sait pas
produire, une politique de données insuffisantes qui refuse plutôt que
d'inventer, et un typage qui rend impossible une confusion d'unités d'un
facteur dix.

Ce qu'elle ne lui laisse pas : la liberté de trancher par défaut une
question que ces documents ont laissée ouverte.
