# Bibliographie HeliosPro — moteur scientifique (Phase 1)

Cette bibliographie recense **toutes** les références scientifiques et
normatives citées dans le code source du moteur `engine/` à la date du
tag `v1.0.0-scientific-core`. Chaque entrée est extraite directement des
commentaires `**Source**:`/`//!` du code (jamais reconstituée de mémoire)
et indique précisément quel(s) module(s)/crate(s) l'utilisent, pour
satisfaire l'exigence de traçabilité de `docs/design/20-rapport-cloture-phase1.md`.

Voir `bibliography.bib` pour la version BibTeX (mêmes entrées, clés
identiques aux ancres `#ref-...` ci-dessous).

## Position solaire et temps

<a id="ref-reda2004"></a>
**[reda2004] Reda, I. & Andreas, A. (2004).** « Solar Position Algorithm
for Solar Radiation Applications ». *Solar Energy*, 76(5), 577-589
(NREL/TP-560-34302).

- **Utilisé dans** : `engine/crates/solar_geometry/src/spa/` (tables de
  coefficients et procédure géocentrique complète), `engine/crates/solar_position/src/spa.rs`
  (étapes topocentriques : parallaxe, réfraction, azimut/zénith).
- **Rôle** : algorithme SPA complet, moteur de position solaire primaire
  du projet (précision annoncée ±0.0003°, années -2000 à 6000).

<a id="ref-meeus1998"></a>
**[meeus1998] Meeus, J. (1998).** *Astronomical Algorithms*, 2e éd.,
Willmann-Bell.

- **Utilisé dans** : `engine/crates/core_math/src/julian.rs` (Ch. 7, Jour
  Julien), `engine/crates/solar_geometry/src/meeus.rs` (Ch. 12, 13, 22,
  25, 28 : temps sidéral, transformation de coordonnées, nutation,
  coordonnées solaires, équation du temps — moteur secondaire), `engine/crates/core_constants/src/lib.rs`
  (constantes dérivées).
- **Rôle** : moteur secondaire de validation croisée (~0.01-0.02°
  d'exactitude), et référence pour le Jour Julien utilisé par tout le
  moteur.

<a id="ref-laskar1986"></a>
**[laskar1986] Laskar, J. (1986).** Polynôme d'obliquité moyenne de
l'écliptique à haute précision (degré 10, cité via Reda & Andreas (2004),
Eq. 3.5.1).

- **Utilisé dans** : `engine/crates/solar_geometry/src/spa/mod.rs`
  (obliquité moyenne de l'écliptique).
- **Rôle** : précision de l'obliquité moyenne au-delà de ce qu'offre le
  polynôme classique de Meeus (utilisé, lui, par le moteur secondaire).

<a id="ref-iau1980"></a>
**[iau1980] International Astronomical Union (1980).** Théorie de
nutation IAU 1980 (série complète de 106 termes ; 63 termes sélectionnés
par Reda & Andreas (2004), Table A3.2, sont ceux implémentés).

- **Utilisé dans** : `engine/crates/solar_geometry/src/spa/tables.rs`
  (`NUTATION_ABCD`, `NUTATION_YTERM`).
- **Rôle** : nutation en longitude et en obliquité, étape de la chaîne
  SPA. Troncature à 63/106 termes héritée de la publication source — voir
  `docs/design/17-analyse-incertitude-numerique.md` §17.4d.

<a id="ref-espenak2006"></a>
**[espenak2006] Espenak, F. & Meeus, J. (2006).** *Five Millennium Canon
of Solar Eclipses: -1999 to +3000*, NASA/TP-2006-214141, « Polynomial
Expressions for Delta T ».

- **Utilisé dans** : `engine/crates/solar_geometry/src/delta_t.rs`
  (branche polynomiale 2005-2050).
- **Rôle** : estimation de ΔT = TT − UT1. Domaine de validité documenté
  et respecté — voir `docs/design/17-analyse-incertitude-numerique.md`
  §17.4a pour l'analyse quantifiée de l'extrapolation hors domaine.

<a id="ref-urban2013"></a>
**[urban2013] Urban, S.E. & Seidelmann, P.K. (eds., 2013).** *Explanatory
Supplement to the Astronomical Almanac*, 3e éd., University Science
Books.

- **Utilisé dans** : cité en appui de la démonstration de la parallaxe
  topocentrique, `docs/design/18-annexe-technique-position-solaire.md`
  §18.6.2.
- **Rôle** : référence de fond pour la dérivation de la correction de
  parallaxe (§7.4 de l'ouvrage).

<a id="ref-smart1977"></a>
**[smart1977] Smart, W.M. (1977).** *Textbook on Spherical Astronomy*,
6e éd., Cambridge University Press.

- **Utilisé dans** : cité en appui de la démonstration de la
  transformation équatoriale → horizontale,
  `docs/design/18-annexe-technique-position-solaire.md` §18.6.1.
- **Rôle** : référence de fond pour la trigonométrie sphérique (loi des
  cosinus, loi des cinq éléments) utilisée dans la démonstration.

<a id="ref-holmgren2018"></a>
**[holmgren2018] Holmgren, W.F., Hansen, C.W., Mikofski, M.A. (2018).**
« pvlib python: a python package for modeling solar energy systems ».
*Journal of Open Source Software*, 3(29), 884.

- **Utilisé dans** : `engine/validation-data/` (génération des jeux de
  données de référence), `engine/crates/validation/` (oracle de
  comparaison externe pour SPA et Meeus).
- **Rôle** : implémentation open-source indépendante et largement
  utilisée du même algorithme SPA, servant d'oracle de validation externe
  (jamais copiée comme code — voir `engine/validation-data/README.md`
  pour la justification complète).

## Irradiance, décomposition et transposition

<a id="ref-duffie2013"></a>
**[duffie2013] Duffie, J.A. & Beckman, W.A. (2013).** *Solar Engineering
of Thermal Processes*, 4e éd., John Wiley & Sons.

- **Utilisé dans** : `engine/crates/core_constants/src/lib.rs` (constante
  solaire), `engine/crates/irradiance/src/` (irradiance extraterrestre,
  décomposition Erbs), `engine/crates/solar_geometry/src/{incidence,time}.rs`
  (angle d'incidence, temps solaire), `engine/crates/transposition/src/`
  (Liu-Jordan, Hay-Davies, HDKR, Perez — reproductions des formules
  originales telles que compilées dans cet ouvrage de référence),
  `engine/crates/thermal_model/src/noct.rs`.
- **Rôle** : ouvrage de référence transversal du domaine solaire
  thermique/PV ; source directe ou de compilation pour la majorité des
  modules d'irradiance et de transposition.

<a id="ref-spencer1971"></a>
**[spencer1971] Spencer, J.W. (1971).** « Fourier series representation
of the position of the sun », *Search*, 2(5), 172 (reproduit dans Duffie
& Beckman (2013)).

- **Utilisé dans** : `engine/crates/irradiance/src/extraterrestrial.rs`
  (correction d'excentricité de l'orbite terrestre).
- **Rôle** : série de Fourier pour le facteur de correction
  excentricité/distance Terre-Soleil.

<a id="ref-erbs1982"></a>
**[erbs1982] Erbs, D.G., Klein, S.A., Duffie, J.A. (1982).** « Estimation
of the diffuse radiation fraction for hourly, daily and monthly-average
global radiation », *Solar Energy*, 28(4), 293-302.

- **Utilisé dans** : `engine/crates/irradiance/src/decomposition.rs`.
- **Rôle** : modèle de décomposition GHI → DNI/DHI (corrélation à
  l'indice de clarté).

<a id="ref-liujordan1963"></a>
**[liujordan1963] Liu, B.Y. & Jordan, R.C. (1963).** « The
interrelationship and characteristic distribution of direct, diffuse, and
total solar radiation », *Solar Energy*, 4(3), 1-19.

- **Utilisé dans** : `engine/crates/transposition/src/isotropic.rs`,
  `engine/crates/transposition/src/inputs.rs` (rapport de vue albédo).
- **Rôle** : modèle de ciel isotrope pour la transposition sur plan
  incliné.

<a id="ref-haydavies1980"></a>
**[haydavies1980] Hay, J.E. & Davies, J.A. (1980).** « Calculation of the
solar radiation incident on an inclined surface », *Proc. First Canadian
Solar Radiation Data Workshop*, Toronto.

- **Utilisé dans** : `engine/crates/transposition/src/hay_davies.rs`,
  `engine/crates/transposition/src/hdkr.rs`.
- **Rôle** : modèle anisotrope Hay-Davies (indice d'anisotropie) pour la
  transposition sur plan incliné.

<a id="ref-klucher1979"></a>
**[klucher1979] Klucher, T.M. (1979).** « Evaluation of models to predict
insolation on tilted surfaces », *Solar Energy*, 23(2), 111-114.

- **Utilisé dans** : `engine/crates/transposition/src/hdkr.rs` (terme de
  rehaussement d'horizon).
- **Rôle** : terme correctif d'éclaircissement de l'horizon, intégré au
  modèle HDKR.

<a id="ref-reindl1990"></a>
**[reindl1990] Reindl, D.T., Beckman, W.A., Duffie, J.A. (1990).**
« Evaluation of hourly tilted surface radiation models », *Solar Energy*,
45(1), 9-17.

- **Utilisé dans** : `engine/crates/transposition/src/hdkr.rs`.
- **Rôle** : modèle HDKR (Hay-Davies-Klucher-Reindl) complet.

<a id="ref-perez1990"></a>
**[perez1990] Perez, R., Ineichen, P., Seals, R., Michalsky, J., Stewart,
R. (1990).** « Modeling daylight availability and irradiance components
from direct and global irradiance », *Solar Energy*, 44(5), 271-289.

- **Utilisé dans** : `engine/crates/transposition/src/perez.rs`.
- **Rôle** : modèle anisotrope Perez (le plus précis des quatre modèles
  de transposition implémentés), table de coefficients empiriques
  (Table 6).

<a id="ref-kastenyoung1989"></a>
**[kastenyoung1989] Kasten, F. & Young, A.T. (1989).** « Revised optical
air mass tables and approximation formula », *Applied Optics*, 28(22),
4735-4738.

- **Utilisé dans** : `engine/crates/transposition/src/perez.rs` (masse
  d'air relative, argument du modèle Perez).
- **Rôle** : formule de masse d'air optique relative.

## Modèle thermique

<a id="ref-ross1976"></a>
**[ross1976] Ross, R.G. (1976).** « Interface design considerations for
terrestrial solar cell modules », *Conference Record, 12th IEEE
Photovoltaic Specialists Conference*, 801-806.

- **Utilisé dans** : `engine/crates/thermal_model/src/noct.rs`.
- **Rôle** : modèle NOCT (Nominal Operating Cell Temperature) de
  température de cellule.

<a id="ref-king2004"></a>
**[king2004] King, D.L., Boyson, W.E., Kratochvil, J.A. (2004).**
*Photovoltaic Array Performance Model*, SAND2004-3535, Sandia National
Laboratories.

- **Utilisé dans** : `engine/crates/thermal_model/src/sandia.rs`.
- **Rôle** : modèle thermique Sandia (coefficients a/b/ΔT dépendant du
  type de montage), alternative empirique au modèle NOCT.

## Constantes physiques et métrologie

<a id="ref-codata2018"></a>
**[codata2018] CODATA (2018).** Valeurs recommandées des constantes
physiques fondamentales, valeurs exactes sous la redéfinition SI 2019.

- **Utilisé dans** : `engine/crates/core_constants/src/lib.rs`
  (`STEFAN_BOLTZMANN_W_M2_K4`).

<a id="ref-iso2533"></a>
**[iso2533] ISO 2533:1975.** *Standard Atmosphere*.

- **Utilisé dans** : `engine/crates/core_constants/src/lib.rs`
  (`STANDARD_ATMOSPHERE_PA`), `engine/crates/solar_position/src/common.rs`
  (pression par défaut), `engine/crates/validation/src/global_campaign/`
  (formule barométrique pour la pression en fonction de l'altitude dans
  la campagne mondiale).

<a id="ref-iso80000"></a>
**[iso80000] ISO 80000-1:2022 et ISO 80000-3:2019.** *Quantités et
unités — Partie 1 : Généralités ; Partie 3 : Espace et temps*.

- **Utilisé dans** : `engine/crates/core_units/src/ratio.rs`,
  `engine/crates/core_constants/src/lib.rs` (secondes/jour, unités
  angulaires).

<a id="ref-iso9060"></a>
**[iso9060] ISO 9060:2018.** *Solar energy — Specification and
classification of instruments for measuring hemispherical solar and
direct solar radiation*.

- **Utilisé dans** : `engine/crates/core_units/src/irradiance.rs`,
  `engine/crates/irradiance/src/decomposition.rs` (convention d'unité
  W/m² et classification des grandeurs mesurées).

<a id="ref-iso6709"></a>
**[iso6709] ISO 6709.** *Standard representation of geographic point
location by coordinates*.

- **Utilisé dans** : `engine/crates/solar_geometry/src/lib.rs`
  (convention de signe de la longitude, positive vers l'est).

<a id="ref-wgs84"></a>
**[wgs84] National Geospatial-Intelligence Agency (NGA).** *World
Geodetic System 1984 (WGS84)*.

- **Utilisé dans** : `engine/crates/core_constants/src/lib.rs`
  (`WGS84_FLATTENING`, `WGS84_SEMI_MAJOR_AXIS_M`),
  `engine/crates/solar_position/src/spa.rs` (comparaison explicite avec
  la constante d'ellipsoïde propre à Reda & Andreas (2004), voir
  `docs/design/17-analyse-incertitude-numerique.md` §17.4b).

## Normes IEC — modèle électrique, pertes, module PV (références planifiées)

Ces normes sont citées dans les crates encore au statut « squelette
documenté » (`pv_module`, `electrical_model`, `losses` — voir
`engine/README.md`) comme références **planifiées** pour leur
implémentation future ; aucun code ne les utilise encore à la date du tag
`v1.0.0-scientific-core`.

<a id="ref-iec61215"></a>
**[iec61215] IEC 61215-1/2.** *Terrestrial photovoltaic (PV) modules —
Design qualification and type approval*.

- **Utilisé dans (implémenté)** : `engine/crates/thermal_model/src/noct.rs`
  (conditions de référence NOCT : G=800 W/m², vent 1 m/s).
- **Utilisé dans (planifié)** : `engine/crates/pv_module/`.

<a id="ref-iec61730"></a>
**[iec61730] IEC 61730-1/2.** *Photovoltaic (PV) module safety
qualification*.

- **Utilisé dans (planifié)** : `engine/crates/pv_module/`.

<a id="ref-iec61853"></a>
**[iec61853] IEC 61853-1..4.** *Photovoltaic (PV) module performance
testing and energy rating*.

- **Utilisé dans (implémenté)** : `engine/crates/thermal_model/src/sandia.rs`
  (mention comme source possible des coefficients de montage
  spécifiques à l'installation).
- **Utilisé dans (planifié)** : `engine/crates/pv_module/`.

<a id="ref-iec60904"></a>
**[iec60904] IEC 60904-1.** *Photovoltaic devices — Part 1: Measurement
of photovoltaic current-voltage characteristics*.

- **Utilisé dans (planifié)** : `engine/crates/electrical_model/`
  (conditions de mesure de la courbe I-V).

<a id="ref-iec60364"></a>
**[iec60364] IEC 60364-5-52.** *Low-voltage electrical installations —
Part 5-52: Selection and erection of electrical equipment — Wiring
systems*.

- **Utilisé dans (planifié)** : `engine/crates/losses/` (dimensionnement
  des câbles, borne de perte Joule).

<a id="ref-iec62548"></a>
**[iec62548] IEC 62548.** *Photovoltaic (PV) arrays — Design
requirements*.

- **Utilisé dans (planifié)** : `engine/crates/losses/`.

<a id="ref-iec61724"></a>
**[iec61724] IEC 61724-1.** *Photovoltaic system performance — Part 1:
Monitoring*.

- **Utilisé dans (planifié)** : `engine/crates/losses/` (définition des
  pertes de performance standard).

## ASHRAE (référence planifiée, hors périmètre Phase 1)

<a id="ref-ashrae93"></a>
**[ashrae93] ASHRAE Standard 93.** *Methods of Testing to Determine the
Thermal Performance of Solar Collectors*.

- **Utilisé dans (planifié, documenté dans les documents de conception,
  pas encore dans le code)** : `docs/design/06-algorithmes.md` §3
  (modificateur d'angle d'incidence ASHRAE/Martin-Ruiz),
  `docs/design/07-modeles-physiques.md`,
  `docs/design/08-normes-references.md`. Réservé à une future extension
  solaire thermique (hors périmètre PV de la Phase 1) — non implémenté
  dans `engine/`.

## Extraction de données / validation croisée

<a id="ref-Reda2004data"></a>
Les tables de coefficients numériques de `engine/crates/solar_geometry/src/spa/tables.rs`
sont des données scientifiques publiques (17 U.S.C. § 105, document
gouvernemental américain) extraites programmatiquement depuis
[holmgren2018] (pvlib-python `spa.py`), jamais retranscrites manuellement
— voir `engine/validation-data/README.md` pour le script exact et la
justification complète de cette méthode d'extraction.
