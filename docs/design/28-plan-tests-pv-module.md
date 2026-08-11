# 28. Plan de tests — `pv_module`

**Statut** : plan de tests, rédigé **avant** l'implémentation. Aucun code
écrit. Complète `docs/design/27-specification-finale-pv-module.md`
(autorité) et applique `docs/design/14-strategie-tests.md`.

## 0. Ce que ce crate peut et ne peut pas être testé pour

`pv_module` ne calcule aucune physique (§1.1 du doc 24). Les tests ne
peuvent donc **pas** porter sur l'exactitude d'un résultat physique. Ils
portent sur quatre propriétés vérifiables :

| Propriété | Question posée | Famille de tests |
|---|---|---|
| Fidélité de représentation | Le modèle absorbe-t-il une source réelle sans perte ni déformation ? | T3, T4 |
| Correction de la validation | Les 47 invariants classent-ils correctement les cas ? | T2 |
| Traçabilité | Provenance, classification et conflits survivent-ils à un aller-retour ? | T5 |
| Conformité architecturale | Les règles structurelles sont-elles respectées ? | T6 |

Toute affirmation de précision physique relèvera d'`electrical_model`, et
sera validée par la matrice IEC 61853 que `pv_module` se contente de
porter.

---

## T1 — Tests unitaires des types

| # | Objet | Critère de réussite |
|---|---|---|
| T1.1 | Nouveaux types `core_units` (`Voltage`, `Current`, `Power`, `Mass`, `Area`, `Wavelength`) | Aller-retour constructeur/accesseur exact à 1e-12 ; conversions non-SI (g→kg, mm²→m², nm→m) exactes |
| T1.2 | `TemperatureCoefficient` | Signé : accepte des valeurs négatives ; conversion %/°C ↔ K⁻¹ exacte ; **absolu et relatif ne sont pas interchangeables** (types ou drapeaux distincts) |
| T1.3 | `Measurement<T>` | Conserve valeur, classification, provenance, incertitude ; `Measurement::stated()` place la bonne classe par défaut |
| T1.4 | `Uncertainty` | `U = k·u` ; `coverage_factor = None` ⇒ incertitude-type |
| T1.5 | `OperatingCondition` | `CellTemperature` et `AmbientTemperature` ne sont pas confondues ; deux conditions de même `kind` sont distinguables |
| T1.6 | Énumérations de classification | `Unknown` ≠ `Other { name }` ; `#[non_exhaustive]` effectif |

## T2 — Invariants de validation (le cœur du plan)

**Règle** : chacun des 47 invariants du §11 reçoit **au minimum trois
tests** — un cas conforme, un cas fautif, un cas limite au seuil. Pour
les invariants à seuil (`INV-ELE-004`, `INV-GEO-002`, `INV-DER-001`),
quatre tests : sous le seuil bas, entre les seuils, au-dessus du seuil
haut, exactement au seuil.

| # | Objet | Critère de réussite |
|---|---|---|
| T2.1 | Positivité (`INV-POS-001..012`) | Toute grandeur ≤ 0 ⇒ **ERROR**, construction refusée |
| T2.2 | Ordonnancement électrique (`INV-ELE-001..012`) | `v_oc ≤ v_mpp` ⇒ ERROR ; facteur de forme > 1 ⇒ ERROR ; ratio hors plage ⇒ WARNING sans refus |
| T2.3 | `INV-ELE-004` (`p_max` vs `v_mpp·i_mpp`) | 0,5 % ⇒ VALID ; 2 % ⇒ WARNING ; 8 % ⇒ **CONFLICT** ; exactement 1 % et 5 % ⇒ comportement documenté et stable |
| T2.4 | `INV-GEO-002` (surface vs dimensions) | 0,05 % ⇒ VALID ; 0,5 % ⇒ WARNING ; 20 % ⇒ **CONFLICT** nommant les deux valeurs |
| T2.5 | Emboîtement des surfaces (`INV-GEO-001/003`) | `aperture > gross` ⇒ ERROR ; `cell > active` ⇒ ERROR |
| T2.6 | Coefficients thermiques (`INV-TMP-001..006`) | Signes inattendus ⇒ **WARNING**, jamais ERROR (une technologie future peut différer) |
| T2.7 | `INV-TMP-005` (unité ambiguë) | Un adaptateur ne pouvant déterminer l'unité produit **CONFLICT**, et **aucune valeur n'est construite** |
| T2.8 | Conditions (`INV-CND-001..006`) | Deux conditions de même `kind` ⇒ ERROR ; température hors [−100, 200] °C ⇒ ERROR |
| T2.9 | Bifacialité (`INV-BIF-001..004`) | φ > 1 ⇒ ERROR ; bloc sans φ ⇒ WARNING, construction acceptée |
| T2.10 | Provenance (`INV-PRV-001..003`) | Classe E sans justification ⇒ ERROR ; paramètre ajusté présenté comme mesuré ⇒ ERROR |
| T2.11 | Explication obligatoire | **Aucun** `ValidationFinding` ne porte une `explanation` vide, pour aucun invariant |
| T2.12 | Statut agrégé | `worst_status()` respecte l'ordre INVALID > CONFLICT > WARNING > UNKNOWN > VALID |

## T3 — Non-arbitrage (exigence validée)

Ces tests vérifient le principe explicitement validé : *une donnée en
conflit n'est jamais arbitrée silencieusement.*

| # | Objet | Critère de réussite |
|---|---|---|
| T3.1 | Conflit non résolu | Construction **refusée**. Le rapport nomme les deux valeurs en cause et l'invariant |
| T3.2 | Absence de choix implicite | Sur `INV-GEO-002`, aucune des deux surfaces n'est retenue ; le module n'existe pas |
| T3.3 | Résolution explicite | Avec une `ConflictResolution` fournie, la construction réussit **et** la résolution (décideur + justification) est conservée dans le module |
| T3.4 | Résolution non appariée | Une résolution portant sur un autre invariant ne débloque pas le conflit courant |
| T3.5 | Justification obligatoire | Une `ConflictResolution` sans `rationale` est refusée |
| T3.6 | Rendement dérivé | Le rendement calculé **indique** quelle surface a servi ; il n'est pas produit tant que `INV-GEO-002` est en conflit |

## T4 — Fidélité sur données réelles

| # | Objet | Critère de réussite |
|---|---|---|
| T4.1 | Échantillon CEC | ≥ 200 enregistrements de la base réelle (doc 26) importés ; **aucune panique**, chaque échec porte un statut et une explication |
| T4.2 | Les 29 anomalies CEC | Les modules à écart surface > 10 % (doc 26 §1) produisent tous **CONFLICT**, aucun ne se construit silencieusement |
| T4.3 | Unités mixtes CEC | `alpha_sc`/`beta_oc` interprétés en absolu, `gamma_r` en relatif ; un module de contrôle reconstitue les valeurs relatives attendues |
| T4.4 | Bifacial CEC | Les 133 modules `Bifacial=1` produisent un bloc bifacial avec `is_bifacial = true` et `bifaciality_factor = None`, statut WARNING |
| T4.5 | BIPV CEC | Les 164 modules `BIPV='Y'` renseignent `building_integrated = Some(true)` |
| T4.6 | PTC CEC | La valeur PTC produit un `OperatingPoint` de `kind = PtcPvusa`, distinct du point STC |
| T4.7 | Vocabulaire technologique | `Thin Film` → `Other { name }` ; architecture → `Unknown` (le CEC n'en fournit aucune) |
| T4.8 | Échantillon Sandia | ≥ 100 enregistrements ; les 28 champs SAPM partent vers `SapmParameters`, **aucun** n'atterrit dans `pv_module` |
| T4.9 | Fiches techniques réelles | ≥ 5 fiches de familles distinctes (c-Si PERC, TOPCon, HJT, IBC, CdTe) absorbées sans perte — **exigence ouverte**, cf. §5 |
| T4.10 | Module monolithique | Une fiche CdTe se construit via `CellConfiguration::Monolithic` sans champ inventé |

## T5 — Traçabilité

| # | Objet | Critère de réussite |
|---|---|---|
| T5.1 | Provenance préservée | Chaque valeur importée porte `source_type` et `extraction_method` non vides |
| T5.2 | Classification correcte | Une valeur issue d'une fiche ⇒ classe A ; d'un rapport d'essai ⇒ B ; d'une saisie ⇒ E |
| T5.3 | Provenances hétérogènes | Un module dont le STC vient d'une fiche et la matrice d'un essai conserve **deux** provenances distinctes |
| T5.4 | Dérivés non enrobés | Les accesseurs dérivés retournent un type nu et sont classés D |
| T5.5 | Rapport attaché | Un module construit avec WARNING conserve son rapport, inspectable en aval |
| T5.6 | Version d'adaptateur | `adapter_version` figure dans la provenance de tout import automatisé |

## T6 — Conformité architecturale

Tests structurels, exécutables sans physique.

| # | Objet | Critère de réussite |
|---|---|---|
| T6.1 | Dépendances | `cargo tree -p pv_module` ne montre que `core_units` et `core_constants` — aucune crate de calcul, aucune crate externe |
| T6.2 | Absence d'I/O | Aucune occurrence de `std::fs`, `std::net`, `std::io` dans `pv_module` |
| T6.3 | Absence de calcul physique | Aucune fonction ne prend une irradiance ou une température **d'exploitation** en entrée (test de frontière du §1.1 doc 24) |
| T6.4 | Technologie descriptive | **Aucun `match`/`if` sur `CellTechnology` ou `CellArchitecture`** dans `electrical_model`, `thermal_model`, `losses`, `bifacial` (règle §7.2 doc 27) — contrôle par recherche textuelle en CI |
| T6.5 | Homogénéité SI | Aucune API publique de `pv_module` n'expose une grandeur physique en `f64` nu |
| T6.6 | Couverture de documentation | `RUSTFLAGS="-D missing_docs" cargo doc` : zéro avertissement |
| T6.7 | Zéro `unsafe` | `unsafe_code = "forbid"` actif |
| T6.8 | Gel Phase 1 | `solar_geometry`, `solar_position`, `thermal_model` inchangés ; extension de `core_units` strictement additive |

## T7 — Robustesse et cas dégénérés

| # | Objet | Critère de réussite |
|---|---|---|
| T7.1 | Absence de panique | Aucune entrée, même absurde (valeurs extrêmes, chaînes vides, champs manquants), ne provoque de panique — elle produit un statut |
| T7.2 | NaN et infinis | `NaN`/`±inf` en entrée ⇒ **INVALID**, jamais propagés dans un module construit |
| T7.3 | Valeurs extrêmes | `f64::MAX`, `f64::MIN_POSITIVE` traités sans débordement silencieux |
| T7.4 | Source vide | Un enregistrement sans aucun champ ⇒ UNKNOWN partout, construction refusée avec explication |
| T7.5 | Déterminisme | Deux imports de la même source produisent des modules **bit-à-bit identiques**, rapports compris |

## T8 — Non-régression

| # | Objet | Critère de réussite |
|---|---|---|
| T8.1 | Golden dataset | Un jeu de modules de référence figé (`validation/golden/`) est reconstruit à l'identique |
| T8.2 | Stabilité des rapports | Le rapport de validation d'un module de référence est stable entre versions ; tout changement est intentionnel et revu |

---

## Critères quantitatifs de sortie

| Critère | Seuil | Vérification |
|---|---|---|
| Couverture de lignes | **> 95 %** | `cargo llvm-cov` (seuil déjà appliqué en CI) |
| Invariants testés | **47 / 47**, ≥ 3 cas chacun | Revue de la table T2 |
| Enregistrements réels importés | ≥ 200 CEC, ≥ 100 Sandia | T4 |
| Fiches techniques réelles | ≥ 5 familles distinctes | T4.9 |
| Paniques | **0** | T7.1 |
| Avertissements build/clippy/doc | **0** | CI |

## Ce que ce plan ne couvre pas

1. **La précision physique** : hors périmètre par construction (§0).
2. **T4.9** (fiches techniques réelles) reste une **exigence ouverte** :
   cet environnement n'a pas d'accès réseau aux PDF constructeurs. Les
   bases CEC et Sandia, déjà exploitées (doc 26), sont des sources
   *dérivées* et ne s'y substituent pas.
3. **`PANAdapter`** : non testé car non implémenté (blocage juridique,
   doc 27 §12.4).
4. **La grille IEC 61853-1** : le jeu exact des couples (irradiance,
   température) doit être lu dans le texte de la norme avant d'écrire
   T4 pour cet adaptateur.
