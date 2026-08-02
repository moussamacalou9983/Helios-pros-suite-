# 16. Phase 1 — Rapport de validation du moteur scientifique (Core Engine)

**Statut** : Phase 1 implémentée, testée, documentée. Voir `engine/` pour le code.

## 16.1 Périmètre livré

7 crates Rust, workspace `engine/`, zéro dépendance externe (uniquement la
bibliothèque standard Rust), **111 tests unitaires**, **0 avertissement**
`cargo build` / `cargo clippy`, compilation en profils `dev` et `release`
validée.

| Crate | Rôle | Tests |
|---|---|---|
| `heliospro-constants` | Constantes physiques référencées | 4 |
| `heliospro-units` | Unités SI (types forts) | 17 |
| `heliospro-math` | Jour julien, interpolation, statistiques d'erreur | 17 |
| `heliospro-solar` | Temps solaire, position solaire, incidence, extraterrestre, décomposition | 40 |
| `heliospro-transposition` | Isotrope, Hay-Davies, HDKR, Perez | 18 |
| `heliospro-thermal` | NOCT, Sandia | 10 |
| `heliospro-energy-balance` | Orchestration complète | 5 |
| **Total** | | **111** |

Aucune interface graphique, aucune base de données, aucun accès réseau ou
disque — conformément au périmètre imposé pour cette phase.

## 16.2 Décision d'ingénierie majeure : précision de l'algorithme de position solaire

**Contexte** : la consigne demandait explicitement « l'algorithme SPA NREL »
(Reda & Andreas, 2004, NREL/TP-560-34302), dont la précision annoncée
(±0.0003°) repose sur une table de coefficients périodiques (VSOP87 tronqué)
d'environ 250 valeurs numériques (Tables A4.1-A4.4 de l'article).

**Problème rencontré** : l'accès réseau sortant vers `nrel.gov` était
indisponible dans cet environnement de développement (échec DNS), et aucun
miroir fiable et exploitable automatiquement de ces tables n'a pu être
récupéré (échecs HTTP 403 sur les sources alternatives testées). Transcrire
~250 coefficients depuis la seule mémoire du modèle, sans aucun moyen de les
vérifier dans cette session, aurait constitué exactement le « code
approximatif » que les règles du projet interdisent.

**Décision prise** : implémenter la structure complète et exacte de
l'algorithme SPA (position héliocentrique → coordonnées géocentriques →
nutation/obliquité → temps sidéral apparent → angle horaire local →
coordonnées topocentriques → réfraction atmosphérique → angle d'incidence),
mais en utilisant, pour les deux étapes qui dans le SPA original s'appuient
sur les grandes tables VSOP87/nutation, les séries réduites **publiées et
citées explicitement par Meeus (1998), *Astronomical Algorithms*, 2nd ed.** :
- Chapitre 25 (« Solar Coordinates ») pour la longitude solaire — précision
  annoncée par Meeus lui-même : ~0.01°.
- Chapitre 22, éq. 22.1 (série de nutation réduite, présentée par Meeus
  comme « suffisante pour la plupart des usages ») — précision ~0.5
  arcsec.
- La correction de parallaxe topocentrique (amplitude max. ≈ 0.0025°,
  Meeus Ch. 40) est volontairement omise : elle est inférieure au plancher
  de précision déjà fixé par la série de longitude, son ajout n'améliorerait
  donc pas la précision globale tout en étant une source classique de bugs
  de quadrant d'azimut.

Cette décision, ses justifications complètes, ses références bibliographiques
et le chemin de mise à niveau vers la table complète sont documentés dans le
code source (`engine/crates/heliospro-solar/src/lib.rs`, ADR-004) — pas
seulement dans ce rapport.

**Précision résultante estimée** : ~0.01-0.02° en zénith/azimut, contre
±0.0003° pour le SPA complet. Cette précision reste largement suffisante
pour l'objectif métier réel (écart <1% sur l'irradiance/l'énergie), l'erreur
de position solaire ne se traduisant en erreur d'irradiance que de manière
fortement atténuée sauf très près du lever/coucher du soleil.

## 16.3 Méthodologie de validation effectivement appliquée

En l'absence d'accès à PVsyst, SAM ou Archelios Calc dans cet environnement
(aucun de ces outils n'est installé ni accessible), la validation n'a **pas**
pu être une comparaison numérique directe avec leurs sorties. Trois familles
de tests indépendantes ont été utilisées à la place, choisies précisément
parce qu'elles ne dépendent d'aucune donnée de référence externe non
vérifiable :

### a) Invariants astronomiques indépendants (faits publics incontestables)
- Déclinaison solaire ≈ 0° aux équinoxes 2024 (dates connues), ≈ ±23.44° aux
  solstices 2024.
- Distance Terre-Soleil ≈ 0.9833 UA au périhélie (début janvier), ≈ 1.0167 UA
  à l'aphélie (début juillet).
- Extrema de l'équation du temps : ≈ -14 min mi-février, ≈ +16 min début
  novembre (phénomène de l'analemme, notoire).
- Lever/coucher du soleil exactement à l'Est/Ouest (azimut 90°/270°) à
  l'équinoxe, pour **toute** latitude — testé explicitement de 60°S à 60°N.
- Élévation solaire au passage au méridien = 90° − latitude à l'équinoxe.

### b) Invariants algébriques dérivés (preuves par substitution directe)
- Les quatre modèles de transposition (isotrope, Hay-Davies, HDKR, Perez)
  se réduisent tous algébriquement à `POA_global = GHI` pour une surface
  horizontale (tilt=0), démontré par substitution directe dans chaque
  formule et vérifié par test pour chacun des quatre modèles.
- Le modèle NOCT retrouve exactement la valeur NOCT du fabricant dans les
  conditions mêmes de sa définition (G=800 W/m², Tamb=20°C) — vérification
  triviale mais rigoureuse par construction.
- La relation de fermeture GHI = DNI·cos(zénith) + DHI est vérifiée exacte
  après décomposition d'Erbs, pour une gamme de conditions de ciel.

### c) Validation croisée entre formules indépendantes
- La distance Terre-Soleil calculée via l'ellipse képlérienne (Meeus 25.5)
  et celle calculée via la série de Fourier de Spencer (1971) — deux
  formules publiées indépendamment pour la même grandeur physique — sont
  comparées sur 365 jours : écart maximal < 0.1%, confirmant qu'aucune des
  deux ne comporte d'erreur grossière de transcription.

## 16.4 Ce qui n'est PAS encore validé (limites explicites)

Par honnêteté d'ingénierie, ce qui suit reste **ouvert** et devra être fait
dès qu'un accès aux outils de référence sera possible :
- Comparaison numérique directe de la production annuelle simulée contre une
  sortie PVsyst/SAM/Archelios Calc sur un cas réel identique.
- Vérification indépendante des ~250 coefficients de la table SPA complète
  (Reda & Andreas 2004) — actuellement non implémentée, voir §16.2.
- Le modèle électrique du module (diode unique), l'onduleur, et le bilan
  énergétique électrique complet n'existent pas encore : cette phase
  s'arrête à l'irradiance sur plan incliné et à la température de cellule,
  conformément au périmètre demandé.

## 16.5 Conclusion de phase

Le moteur scientifique Phase 1 est **compilable, entièrement testé (111/111
tests verts), documenté et reproductible**. Chaque formule physique porte sa
référence bibliographique ou normative en commentaire de code. La seule
déviation vis-à-vis de la demande initiale — la précision de l'algorithme de
position solaire — est intentionnelle, documentée, justifiée, et n'empêche
pas la validation actuelle. Elle est proposée ci-dessus comme point de
vigilance explicite avant tout usage réglementaire/commercial nécessitant la
précision ±0.0003° du SPA complet.

**Recommandation** : considérer la Phase 1 comme validée pour passer à la
Phase 2 (modèle électrique PV, cf. feuille de route `15-feuille-de-route.md`),
sous réserve que le porteur de projet valide explicitement le compromis de
précision décrit en §16.2, ou fournisse un accès à la table SPA complète
pour mise à niveau immédiate.
