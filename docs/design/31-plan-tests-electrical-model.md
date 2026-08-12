# 31. Plan de tests — `electrical_model`

**Date** : 2026-08-11
**Statut** : conception — **aucun test écrit, aucun code**
**Référence** : `docs/design/30-specification-electrical-model.md`

---

## 0. Principe

Le plan hérite de la discipline établie sur `pv_module` et validée par la
vérification de traçabilité :

- les **scénarios sont des données**, pas des fonctions — le décompte est
  une mesure, pas une affirmation (doc 29 §6.0) ;
- **minimum trois scénarios** par règle, par cas limite et par invariant ;
- un scénario attendant l'**absence** de résultat compte autant qu'un
  scénario attendant un résultat ;
- le seuil est **protégé par la CI**.

Une différence de fond avec `pv_module` : ce crate **calcule**. S'ajoutent
donc deux exigences absentes en amont — la **précision numérique** et la
**convergence** — qui demandent leurs propres familles de tests.

---

## 1. Familles de tests

| Famille | Objet | Vérité de référence |
|---|---|---|
| **E1** | Solutions analytiques exactes | Mathématique, aucune référence externe |
| **E2** | Auto-cohérence du modèle | Propriétés internes |
| **E3** | Vérification du solveur | Résidu de l'équation implicite |
| **E4** | Aller-retour synthétique | Paramètres imposés puis ré-identifiés |
| **E5** | Cas limites | Comportement spécifié au §10 de la doc 30 |
| **E6** | Identification des paramètres | Convergence et refus |
| **E7** | Typage et frontières | Compilation et API |
| **E8** | Traçabilité des paramètres | Classe, provenance, méthode |
| **E9** | Bases publiques (CEC, Sandia) | Comportement à grande échelle |
| **E10** | Oracle externe (pvlib) | Divergences à expliquer |
| **E11** | Performance et non-régression | Budgets mesurés |

---

## 2. E1 — Solutions analytiques

La famille la plus forte : elle ne dépend d'aucune implémentation tierce.

| # | Cas | Solution exacte |
|---|---|---|
| E1.1 | `R_s = 0`, `R_sh → ∞` | `I = I_L − I_0[exp(V/a) − 1]`, explicite |
| E1.2 | `R_s = 0`, `R_sh` fini | Explicite en `I`, sans itération |
| E1.3 | `I_0 → 0` | `I = I_L − V/R_sh`, droite |
| E1.4 | `V = 0` | `I_sc` par résolution directe |
| E1.5 | `I = 0` | `V_oc` par forme fermée quand `R_s = 0` |
| E1.6 | Cohérence Newton / repli à encadrement / bissection | Les trois doivent coïncider à la tolérance |

**E1.6 est central** : trois méthodes indépendantes convergeant vers la
même valeur constituent une vérification croisée qu'aucun oracle externe
ne remplace. *(Corrigé le 12/08/2026 : Lambert W n'est pas implémentée —
ADR-0010 ; la bissection fournit la troisième méthode.)*

---

## 3. E2 — Auto-cohérence

Les huit invariants du §11.3 de la doc 30, chacun avec au moins trois
scénarios (nominal, limite, dégradé) :

| # | Invariant |
|---|---|
| E2.1 | `P(V_mp) ≥ P(V)` sur tout le balayage |
| E2.2 | `I(0) = I_sc`, `I(V_oc) = 0` |
| E2.3 | `I(V)` strictement décroissante sur `[0, V_oc]` |
| E2.4 | `P_max` croissante en `G` |
| E2.5 | `P_max` décroissante en `T` si `γ < 0` |
| E2.6 | `γ_Pmax` reconstitué cohérent avec la fiche |
| E2.7 | `V_oc(G,T)` cohérent avec `β_Voc` |
| E2.8 | Résidu de l'équation implicite sous tolérance |

---

## 4. E3 — Vérification du solveur

Distincte de E2 : **le solveur et le modèle échouent différemment.**

| # | Cas |
|---|---|
| E3.1 | Le résidu au point rendu est sous la tolérance déclarée |
| E3.2 | Le nombre d'itérations reste sous la borne |
| E3.3 | La non-convergence produit une **erreur typée**, jamais une valeur |
| E3.4 | Le repli s'active sur condition **objective** : itéré non fini, sortie de l'encadrement, ou budget épuisé |
| E3.5 | Le résultat est identique quelle que soit la branche empruntée |
| E3.6 | Déterminisme : deux appels identiques donnent des bits identiques |

---

## 5. E4 — Aller-retour synthétique

Paramètres choisis → courbe engendrée → paramètres ré-identifiés.

| # | Cas | Attendu |
|---|---|---|
| E4.1 | Aller-retour sans bruit | Les paramètres d'origine, à la tolérance |
| E4.2 | Aller-retour avec bruit contrôlé | Écart cohérent avec le bruit injecté |
| E4.3 | Sur-détermination (matrice) | Meilleur conditionnement que cinq points |
| E4.4 | Sous-détermination | **Refus**, pas une solution arbitraire |

E4.1 est la seule manière de vérifier une identification sans disposer de
mesures réelles : la vérité est connue parce qu'elle a été posée.

---

## 6. E5 — Cas limites

Les 14 cas du §10 de la doc 30, **trois scénarios chacun au minimum** :
en deçà du seuil, au seuil, au-delà.

Cas exigeant une attention particulière :

| # | Cas | Piège |
|---|---|---|
| E5.1 | `G → 0` | Ni NaN ni `−∞` ; le SAPM doit **refuser**, pas extrapoler |
| E5.4 | `R_s = 0` | Résidu **affine**, solution exacte en 1 itération ; **aucun seuil** pour `R_s → 0` (doc 34 §6) |
| E5.7 | `R_sh` faible | L'unicité du MPP est **démontrée** (doc 34 §10) ; le test **vérifie la démonstration**, y compris à `R_sh = 1,5 Ω` |
| E5.10 | Pas de MPP | Erreur typée, jamais un point plausible mais faux |
| E5.14 | Débordement d'exponentielle | `u_max` contrôlé **a priori** ; refus, jamais `try/catch` (doc 34 §7) |

---

## 7. E6 — Identification

| # | Cas | Attendu |
|---|---|---|
| E6.1 | Fiche complète | Convergence |
| E6.2 | `β_Voc` absent | Système sous-déterminé → **refus explicite** |
| E6.3 | Solution non physique (`R_sh < 0`) | Rejet, pas de troncature silencieuse |
| E6.4 | Non-convergence | Erreur typée avec diagnostic |
| E6.5 | `PublishedAsIs` | Aucun calcul, valeurs reprises telles quelles |
| E6.6 | Conditionnement | La résolution en `ln(I_0)` améliore mesurablement |

---

## 8. E7 — Typage et frontières

Certains de ces tests doivent **échouer à la compilation** ; ils seront
donc des tests `compile_fail` documentés, non des tests d'exécution.

| # | Cas |
|---|---|
| E7.1 | Paramètres De Soto avec évaluateur SAPM → **ne compile pas** |
| E7.2 | Demander une courbe continue au SAPM → **ne compile pas** (§5.6) |
| E7.3 | Paramètres de référence utilisés comme effectifs → **ne compile pas** |
| E7.4 | `pv_module` ne dépend pas d'`electrical_model` |
| E7.5 | Aucune dépendance externe |
| E7.6 | Toute grandeur publique porte un type d'unité, jamais un `f64` nu |

---

## 9. E8 — Traçabilité des paramètres

| # | Cas |
|---|---|
| E8.1 | Tout paramètre de modèle est de classe `Fitted` |
| E8.2 | La méthode d'identification voyage avec le résultat |
| E8.3 | La provenance survit à la translation vers les conditions |
| E8.4 | Le modèle d'appartenance est inséparable des paramètres |
| E8.5 | Les résidus sont disponibles quand la méthode en produit |
| E8.6 | Un conflit entre sources n'est **jamais arbitré** |

---

## 10. E9 / E10 — Données réelles et oracle

| # | Cas |
|---|---|
| E9.1 | Balayage de la base CEC : aucun NaN, aucune non-convergence non signalée |
| E9.2 | Balayage Sandia : idem pour le SAPM |
| E9.3 | Distribution des résidus, publiée en rapport |
| E10.1 | Comparaison à pvlib sur un échantillon large |
| E10.2 | **Toute divergence est expliquée**, jamais absorbée dans un seuil élargi |

Le E10.2 reprend la discipline de la campagne de position solaire, où
deux divergences réelles ont été trouvées, comprises et documentées
plutôt que masquées par un desserrage de tolérance.

---

## 11. E11 — Performance

| # | Cas |
|---|---|
| E11.1 | Temps moyen d'un point I-V, écart-type mesuré |
| E11.2 | Temps d'une courbe complète |
| E11.3 | Zéro allocation en régime établi |
| E11.4 | Budget de non-régression en CI |

---

## 12. Couverture et seuils

| Exigence | Valeur |
|---|---|
| Couverture de lignes | ≥ 95 % |
| Scénarios par règle / cas limite | ≥ 3, **mesurés** |
| Avertissements clippy | 0 |
| Régressions | 0 |
| Matrice de scénarios | Générée et publiée en artefact |

---

## 12bis. Jeux de tests scientifiques — préparés

Les **22 jeux de paramètres** exigés avant implémentation sont définis
dans `docs/design/34-audit-mathematique-electrical-model.md` §16. Neuf
d'entre eux ont déjà servi à la vérification numérique de l'audit.

Ils couvrent : `R_s = 0`, `R_s → 0`, `R_s` élevé, `R_sh → ∞`, `R_sh`
faible, irradiance nulle, très faible, STC, élevée, températures extrêmes,
`I_sc` faible, paramètres incohérents, débordement potentiel, convergence
rapide, convergence difficile, absence de solution, MPP proche des bornes,
et extrema multiples (dont l'audit démontre l'impossibilité, §10 — le jeu
sert donc à **vérifier la démonstration**, pas à traiter un cas attendu).

Tous sont **synthétiques et documentés comme tels** : aucun ne prétend
décrire un produit réel.

---

## 13. Points ouverts

| Point | Statut |
|---|---|
| Cinq fiches constructeurs réelles | **Toujours ouvert** — à ne pas simuler |
| Grille exacte IEC 61853-1 | **Toujours ouvert** — à lire dans la norme |
| Mesures I-V de terrain | Non disponibles — limite à déclarer (doc 30 §11.4) |
| Valeurs de tolérance | À mesurer avant d'être gravées (doc 30 §8.7) |

Ces points ne bloquent pas la conception. Ils bloquent la prétention à
une validation complète, et doivent rester visibles à ce titre.
