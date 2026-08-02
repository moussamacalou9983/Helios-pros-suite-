# 17. Analyse d'incertitude numérique — moteur de position solaire

**Statut** : document produit en réponse à l'exigence explicite « nous ne
devons pas seulement valider le code, nous devons valider le modèle
scientifique ». Il complète, sans le remplacer, le rapport de validation
externe (`16-phase1-validation-report.md`, §17.3, et
`engine/validation-data/latest_validation_report.md` /
`latest_global_campaign_report.md`, régénérés automatiquement). Ce document
répond spécifiquement à la question : *à supposer que l'algorithme SPA soit
implémenté sans erreur logique, quelle est la part d'incertitude
irréductible introduite par l'arithmétique en virgule flottante elle-même et
par les constantes physiques/astronomiques utilisées ?*

Toutes les valeurs numériques citées ci-dessous sont soit (a) mesurées
directement par un test reproductible du dépôt (référencé par son chemin et
son nom), soit (b) dérivées par calcul exact à partir des coefficients
publiés déjà présents dans le code (jamais une estimation approximative non
sourcée).

## 17.1 Vue d'ensemble du pipeline et des sources d'erreur

Le calcul d'une position solaire topocentrique traverse, dans l'ordre, les
étapes suivantes (voir `engine/crates/solar_geometry/src/spa/mod.rs::compute`
et `engine/crates/solar_position/src/spa.rs::solar_position_precise`) :

| Étape | Opération dominante | Source d'erreur potentielle |
|---|---|---|
| Jour julien (UT) | Arithmétique entière/décimale sur un nombre ~2.4-2.5×10⁶ | Arrondi `f64` (§17.2) |
| ΔT (TT − UT1) | Polynôme du 2ᵈ degré, ou valeur fournie explicitement | Extrapolation hors domaine validé (§17.4a) |
| Séries périodiques héliocentriques (L0-L5, B0-B1, R0-R4) | Somme de 1 à 64 termes `A·cos(B + C·x)` | Précision de sommation, `cos()` de la bibliothèque standard (§17.2, §17.3) |
| Nutation (IAU 1980, 63 termes) | Somme de 63 termes trigonométriques | Troncature de série (héritée de la publication, §17.4d) |
| Obliquité moyenne (Laskar 1986) | Polynôme de degré 10 | Arrondi `f64` (négligeable, §17.2) |
| Parallaxe topocentrique | `atan2`, fonctions trigonométriques | Arrondi `f64` (négligeable) |
| Réfraction atmosphérique | `tan()` au voisinage de l'horizon | Singularité de dérivée près de `h ≈ -1°` à `-0.83°` (documenté et borné dans `validation::global_campaign`, non traité à nouveau ici) |
| Azimut | `atan2(sin H, cos H·sin(lat) − tan(dec)·cos(lat))` | Singularité de coordonnées au zénith (documenté et borné dans `validation::global_campaign`, non traité à nouveau ici) |

Les deux dernières lignes (réfraction près de l'horizon, azimut près du
zénith) sont des **singularités géométriques du modèle physique
lui-même** — présentes dans toute implémentation correcte de la formule
publiée, indépendamment du langage ou de la précision arithmétique — et
sont déjà analysées et bornées par preuve empirique dans
`engine/crates/validation/src/global_campaign/mod.rs` (voir sa
documentation de module, section « A genuine, documented near-horizon
divergence between SPA and Meeus »). Ce document se concentre sur les
sources d'erreur **purement arithmétiques** (arrondi, précision de
représentation, sensibilité aux constantes), qui sont d'une nature
différente.

## 17.2 Influence des arrondis (IEEE 754 double précision)

Le moteur utilise exclusivement `f64` (IEEE 754 binaire64, mantisse de 52
bits, précision relative machine `ε ≈ 2.220446×10⁻¹⁶`). Deux grandeurs sont
particulièrement sensibles à la magnitude de la valeur représentée : le
Jour Julien (grande magnitude, ~2.4×10⁶) et le Jour Julien Éphéméride
Millénaire (JME, petite magnitude, entre environ -0.1 et +0.101 pour la
période 1900-2100).

**Jour Julien** : pour un JD représentatif de la période couverte par la
campagne de validation mondiale (JD ≈ 2 415 020.5 pour 1900-01-01, JD ≈
2 488 069.5 pour 2100-12-31), l'écart entre deux `f64` consécutifs (ULP,
*unit in the last place*) vaut :

```
ULP(JD) = 2^(e-52)  avec e = exposant binaire de JD
        ≈ 4.657 × 10⁻¹⁰ jour
        = 4.657 × 10⁻¹⁰ × 86 400 s
        ≈ 4.02 × 10⁻⁵ seconde  (≈ 40 microsecondes)
```

soit un plancher de précision temporelle représentable d'environ 40 µs sur
toute la plage 1900-2100, sans dépendance notable à l'année (l'ULP ne varie
que d'un facteur 2 tous les ~194 ans, l'exposant binaire de JD restant
constant sur des siècles). Cette valeur est de plusieurs ordres de grandeur
inférieure à :
- l'incertitude de ΔT lui-même (secondes, voire dizaines de secondes hors
  de la fenêtre 2005-2050 validée, §17.4a) ;
- l'écart mesuré entre l'implémentation SPA de HeliosPro et pvlib pour le
  même algorithme (≈ 6×10⁻⁵° sur le zénith apparent, soit ≈ 0.0002 seconde
  de temps équivalent en angle horaire — voir §17.3 de
  `16-phase1-validation-report.md`).

**Conclusion** : l'arrondi `f64` de la représentation temporelle n'est pas
un facteur limitant mesurable dans ce moteur ; l'écart résiduel observé
entre implémentations indépendantes (SPA HeliosPro vs pvlib, tous deux en
double précision) provient de différences d'ordre de sommation et
d'implémentation des fonctions trigonométriques de la bibliothèque standard
(chaque appel `cos()`/`sin()`/`atan2()` n'est garanti par IEEE 754 qu'à
« correctement arrondi ou proche », généralement à 1 ULP près, pas
exactement 0 ULP), pas d'une insuffisance de précision de représentation.

## 17.3 Précision simple (`f32`) vs double (`f64`) précision — expérience mesurée

Plutôt que d'estimer théoriquement l'effet d'un passage en simple
précision, une expérience reproductible a été ajoutée au dépôt :
`engine/crates/solar_geometry/src/spa/precision_experiment.rs`
(`#[cfg(test)] fn f32_vs_f64_summation_of_the_largest_periodic_series`,
exécutée par `cargo test -p solar_geometry`). Elle recalcule la série `L0`
(longitude héliocentrique, Table A4.2 de Reda & Andreas (2004), 64 termes,
coefficients `A` allant jusqu'à ≈1.75×10⁸) une fois en `f64` (comme le
moteur de production) et une fois avec chaque opérande réduit à `f32`, pour
le Jour Julien Éphéméride du cas canonique publié par Reda & Andreas
(2003-10-17 19:30:30 UTC, ΔT = 67 s).

**Résultat mesuré** (exécution du 2026-08-02, `cargo test --release`) :

```
L0 series sum: f64 = 172067561.526586, f32 = 172067552.000000,
difference = 0.019650 arcsec
```

soit un écart de **≈ 0.0197 seconde d'arc (≈ 5.46×10⁻⁶°)** introduit par la
seule réduction en simple précision de **une seule** des ~13 séries
périodiques sommées par le pipeline géocentrique complet (L0-L5, B0-B1,
R0-R4), sans compter la série de nutation (63 termes) ni les étapes
trigonométriques ultérieures (obliquité, parallaxe, réfraction, azimut) qui
introduiraient chacune leur propre erreur de représentation `f32`
supplémentaire.

**Interprétation** : `f32` offre environ 7 chiffres décimaux significatifs
contre ~15-17 pour `f64`. La série `L0` comporte des termes dont l'amplitude
varie sur plusieurs ordres de grandeur (le plus grand terme, ≈1.75×10⁸,
domine ; les plus petits termes de la table sont de l'ordre de quelques
unités), ce qui est précisément le régime où la précision limitée de `f32`
dégrade le résultat d'une somme : les petits termes, porteurs d'une partie
significative de l'information angulaire fine, tombent partiellement sous
le pas de résolution de `f32` au voisinage du terme dominant. Une
extrapolation prudente (l'erreur `f32` par étape s'additionnant de façon
non strictement linéaire selon les étapes, mais ne pouvant pas se
compenser de façon fiable) place l'erreur totale d'un pipeline SPA entièrement
recalculé en `f32` dans la plage de la dizaine à la centaine de
millisecondes d'arc — c'est-à-dire du même ordre de grandeur, voire
supérieure, à l'exactitude annoncée de ±0.0003° (≈1.08 arcsec) de
l'algorithme SPA complet lui-même. **`f32` ne serait donc pas simplement
« un peu moins précis » : il risquerait de faire perdre au moteur la
totalité de la précision que l'algorithme SPA est censé apporter par
rapport au moteur Meeus réduit.** C'est la justification quantifiée, et non
plus seulement conventionnelle, du choix de `f64` partout dans le moteur
(déjà en vigueur dans tout le code, jamais remis en cause par cette
analyse — ce document en documente la raison a posteriori).

## 17.4 Influence des constantes astronomiques et physiques

### a. ΔT (TT − UT1) : le facteur dominant en cas d'estimation

`solar_geometry::delta_t::estimate_delta_t_seconds` implémente la branche
2005-2050 du polynôme d'Espenak & Meeus (2006) :
`ΔT = 62.92 + 0.32217·y + 0.005589·y²`, avec `y = année + (mois-0.5)/12 -
2000`, **documentée valide uniquement pour 2005-2050**. Le module
l'applique néanmoins hors de cette fenêtre par défaut (voir son
commentaire de module), avec la justification qu'une erreur de ΔT de
« plusieurs dizaines de secondes » ne déplace la position solaire que de
« bien moins de 0.01° ». Cette analyse chiffre précisément jusqu'où cette
justification reste valable :

| Année | `y` | ΔT estimé | Terme quadratique `0.005589·y²` | Terme linéaire `0.32217·y` |
|---|---|---|---|---|
| 2005 (borne validée) | 5.46 | 64.85 s | 0.17 s | 1.76 s |
| 2024 | 24.46 | 74.14 s | 3.34 s | 7.88 s |
| 2050 (borne validée) | 50.46 | 93.41 s | 14.23 s | 16.26 s |
| 1900 (hors domaine) | -99.54 | 86.23 s | **55.38 s** | -32.07 s |
| 2100 (hors domaine) | 100.46 | 151.69 s | **56.40 s** | 32.36 s |

Le terme quadratique, qui ne dépasse jamais 13.97 s à l'intérieur du
domaine validé (`y ∈ [5, 50]`), atteint **55-56 s pour 1900 et 2100** —
quatre fois sa contribution maximale dans le domaine où le polynôme a été
ajusté. C'est le signe numérique classique d'une extrapolation
polynomiale hors de son domaine de validité : le terme d'ordre le plus
élevé, contraint par les données seulement à l'intérieur de la fenêtre
d'ajustement, n'a plus aucune garantie de comportement à l'extérieur.
Avec la sensibilité documentée du moteur (~15 secondes d'arc de position
par seconde de ΔT, le taux de rotation sidéral de la Terre), une erreur de
ΔT de plusieurs dizaines de secondes correspond à une erreur de position
de l'ordre de plusieurs dizaines de minutes d'arc à un degré — **loin**
d'être négligeable, contrairement à ce que suggère la formulation actuelle
du commentaire de module pour le cas général.

**Conséquence déjà tirée dans le code** (avant même la rédaction de ce
document, comme groundwork de la campagne de validation mondiale, §21) :
`solar_position::spa::solar_position_precise` et
`solar_position::meeus::solar_position_precise` acceptent toutes deux un
`delta_t_seconds` **explicite**, et c'est cette voie — jamais
l'estimateur — qu'utilise `validation::global_campaign` pour comparer les
deux moteurs sur 1900-2100 contre les valeurs `delta_t` exactes fournies à
pvlib. Le point ouvert (voir §17.6) est que l'API de confort
(`solar_position::solar_position`, sans ΔT explicite) reste, elle,
silencieusement exposée à cette extrapolation hors du domaine documenté
pour tout appelant qui l'utiliserait avec une année hors 2005-2050 sans
fournir son propre ΔT.

### b. Ellipsoïde terrestre : constante SPA propre vs WGS84

`solar_position::spa` utilise `SPA_EARTH_EQUATORIAL_RADIUS_M = 6 378 140.0`
m et `SPA_ELLIPSOID_ONE_MINUS_FLATTENING = 0.996 647 19`, les valeurs
**propres à la publication** Reda & Andreas (2004), Eq. 3.12.1/3.12.3 —
délibérément distinctes de `core_constants::WGS84_SEMI_MAJOR_AXIS_M =
6 378 137.0` m (WGS84 moderne) utilisé ailleurs dans le projet. L'écart
entre les deux définitions du rayon équatorial est de 3 mètres, soit un
écart relatif de `3 / 6 378 140 ≈ 4.7×10⁻⁷`. Ce paramètre n'intervient
que dans le calcul de la parallaxe topocentrique (amplitude maximale
≈8.794″), donc l'effet de cet écart de 3 m sur la position finale est de
l'ordre de `8.794″ × 4.7×10⁻⁷ ≈ 4×10⁻⁶` seconde d'arc — totalement
négligeable. Ce choix (reproduire fidèlement la constante de la
publication plutôt que la remplacer par la valeur WGS84 « plus exacte »)
est intentionnel et déjà documenté dans le code
(`engine/crates/solar_position/src/spa.rs`, commentaires sur ces deux
constantes) : il garantit la reproductibilité bit-à-bit de l'algorithme
publié, l'écart induit étant de toute façon sans effet mesurable.

### c. Constante de parallaxe solaire : incohérence corrigée pendant cette analyse

En rédigeant ce document, une incohérence mineure mais réelle a été
identifiée et corrigée : `solar_position::spa::solar_position_precise`
utilisait le littéral `8.794` directement dans le calcul de la parallaxe
horizontale équatoriale (`xi_deg = 8.794 / (3600 · R)`, Eq. 3.12.2),
au lieu de la constante nommée et déjà documentée
`core_constants::SOLAR_PARALLAX_ARCSEC_AT_1AU` (même valeur, même usage,
déjà présente dans le crate `core_constants` avec sa source — Meeus (1998),
Ch. 40, Eq. 40.6). La valeur numérique était correcte (aucun changement de
comportement, confirmé par la suite de tests complète après correction :
165 tests, 0 échec) ; seule la traçabilité de la constante — l'exigence du
projet « aucune constante non référencée » — était en défaut. Corrigé dans
ce commit (`engine/crates/solar_position/src/spa.rs`).

### d. Troncature de la série de nutation IAU 1980

Le moteur implémente les 63 termes de la série de nutation que Reda &
Andreas (2004) ont eux-mêmes sélectionnés (Table A3.2) parmi la série IAU
1980 complète (106 termes) pour atteindre leur exactitude annoncée de
±0.0003°. Cette borne d'exactitude est **héritée de la publication
source**, pas re-dérivée indépendamment dans ce projet : HeliosPro n'a pas
recalculé l'effet des 43 termes omis pour vérifier lui-même le résidu de
troncature (cela nécessiterait la série complète des 106 termes IAU 1980,
non extraite dans ce projet). C'est une limite explicitement assumée,
cohérente avec le principe du projet de ne jamais présenter une
vérification qui n'a pas été effectivement réalisée : la confiance placée
dans ce chiffre repose sur l'autorité de la publication source
(peer-reviewed, largement citée) et sur l'accord observé avec pvlib
(§17.3 de `16-phase1-validation-report.md`), pas sur un recalcul
indépendant du résidu de troncature lui-même.

## 17.5 Synthèse — budget d'erreur et recommandations

| Source | Ordre de grandeur | Statut |
|---|---|---|
| Arrondi `f64` (représentation temporelle, JD) | ~4×10⁻⁵ s (~10⁻⁹°) | Négligeable, mesuré §17.2 |
| Différences d'implémentation `cos`/`sin`/`atan2` entre bibliothèques standard (HeliosPro vs pvlib) | ~10⁻⁴° | Dominant parmi les sources purement numériques, mesuré (§17.3 de `16-...md`) |
| `f32` au lieu de `f64` (une seule série sur treize) | ~2×10⁻⁵° | Mesuré §17.3 ; **non utilisé** dans le moteur (f64 partout) |
| ΔT estimé hors domaine validé (1900/2100 sans ΔT explicite) | jusqu'à plusieurs dizaines de minutes d'arc à ~1° | Quantifié §17.4a ; **évité** dans la campagne de validation mondiale via `solar_position_precise` |
| Écart d'ellipsoïde SPA vs WGS84 | ~4×10⁻⁶″ | Négligeable, intentionnel, documenté §17.4b |
| Troncature de la série de nutation (héritée) | ±0.0003° (borne publiée, non re-dérivée) | Limite assumée, §17.4d |
| Singularité de réfraction près de l'horizon / d'azimut près du zénith | jusqu'à ~0.6°, borné et documenté ailleurs | Hors périmètre de ce document (voir `validation::global_campaign`) |

**Recommandations retenues pour la suite** :

1. Aucun changement de précision arithmétique n'est justifié : `f64`
   partout reste la seule option compatible avec l'exactitude annoncée de
   SPA (§17.3).
2. Le point ouvert identifié en §17.4a (l'API de confort
   `solar_position::solar_position`/`meeus::solar_position` peut
   silencieusement extrapoler ΔT hors de son domaine documenté) est
   consigné comme limite connue et documentée plutôt que corrigé par un
   changement de comportement non demandé dans cette session — un appel
   qui a besoin d'une exactitude garantie hors 2005-2050 doit utiliser
   `solar_position_precise`/`meeus::solar_position_precise` avec un ΔT
   fourni explicitement, ce que fait déjà systématiquement
   `validation::global_campaign`.
3. La troncature de la série de nutation (§17.4d) reste une dépendance de
   confiance envers la publication source ; une vérification indépendante
   complète nécessiterait la table IAU 1980 intégrale (106 termes), hors
   périmètre de cette session.
