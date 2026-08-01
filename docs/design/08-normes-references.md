# 8. Normes IEC, ISO et autres références applicables

## 8.1 Normes électriques et systèmes PV

| Norme | Objet | Usage dans HeliosPro |
|---|---|---|
| IEC 60364-7-712 | Installations électriques BT — Alimentations photovoltaïques | Vérification câblage, protections, mise à la terre (M18) |
| NF C 15-100 (+ NF C 15-712-1/2/3) | Déclinaison française des installations BT PV | Règles de vérification France, dossier Consuel (M18, M15) |
| IEC 62548 | Exigences de conception des générateurs PV | Contraintes de calepinage/câblage (M5) |
| IEC 62446-1/2 | Essais, documentation et maintenance des systèmes PV | Structure des rapports d'essai (M15) |
| IEC 61730-1/2 | Sécurité des modules PV (qualification) | Filtrage catalogue équipements (M14) |
| IEC 61215-1/2 | Qualification de la conception des modules PV | Filtrage catalogue équipements (M14) |
| IEC 62109-1/2 | Sécurité des onduleurs pour systèmes PV | Filtrage catalogue équipements (M14) |
| IEC 62305-1..4 | Protection contre la foudre | Dimensionnement parafoudre (M5/M18) |
| IEC 60947 / IEC 60269 | Appareillage de coupure, fusibles BT | Choix protections (M5) |

## 8.2 Caractérisation et performance PV

| Norme | Objet | Usage |
|---|---|---|
| IEC 60904-1..10 | Mesure des caractéristiques I-V, conditions STC | Modèle électrique module (M7 physique) |
| IEC 61853-1..4 | Évaluation de la performance énergétique (matrice de rendement) | Modèle de performance étendu, alternative au modèle single-diode (§7) |
| IEC 61724-1..3 | Suivi de performance des systèmes PV | Définition des indicateurs (PR, Yield) (M6) |
| IEC TS 60904-13 | Imagerie électroluminescence (référence qualité) | Hors périmètre calcul, référence documentaire |

## 8.3 Stockage et systèmes hybrides

| Norme | Objet | Usage |
|---|---|---|
| IEC 62619 | Sécurité des batteries Li-ion pour applications industrielles | Filtrage catalogue batteries (M14) |
| IEC 62933-1..5 | Systèmes de stockage d'énergie électrique (performance, sécurité, environnement) | Cadre de modélisation stockage (M7) |
| IEC 61427-1/2 | Batteries pour systèmes photovoltaïques (autonomes et raccordés réseau) | Critères de dimensionnement (M7) |
| IEEE 1547 | Interconnexion de ressources énergétiques distribuées (réseau) | Référence dispatch/raccordement (M8), hors France |
| VDE-AR-N 4105 | Raccordement BT en Allemagne | Référence localisation DE (roadmap) |
| Engineering Recommendation G98/G99 | Raccordement au Royaume-Uni | Référence localisation UK (roadmap) |
| UTE C15-712-1 | Guide pratique PV raccordé réseau BT (France) | Vérifications France (M18) |

## 8.4 Données climatiques et solaires

| Norme / Référence | Objet | Usage |
|---|---|---|
| ISO 9060:2018 | Classification des pyranomètres/pyrhéliomètres | Qualité des sources de mesure importées (M3) |
| ISO 15927-4 | Données climatiques pour l'évaluation énergétique des bâtiments | Référence méthodologique TMY (M3) |
| Format TMY3 (NREL) / EPW (DOE) | Formats standards d'année météo typique | Import/export interopérable (M16) |
| WMO No. 8 | Guide des instruments et méthodes d'observation météorologique | Référence qualité des données (M3) |
| ASHRAE 93 | Méthodes d'essai de capteurs solaires thermiques | Référence pour extension solaire thermique (hors PV, roadmap) |

## 8.5 Qualité logicielle, sécurité et données

| Norme | Objet | Usage |
|---|---|---|
| ISO/IEC 25010 | Modèle de qualité du produit logiciel (SQuaRE) | Référentiel qualité interne (§14) |
| ISO/IEC 25012 | Modèle de qualité des données | Contrôle qualité données météo/équipements |
| ISO/IEC 27001 | Management de la sécurité de l'information | Cadre SMSI pour le SaaS |
| ISO 9001 | Management de la qualité | Cadre process pour le développement (optionnel, non bloquant V1) |
| RGPD (UE 2016/679) | Protection des données personnelles | Conformité hébergement SaaS UE |
| OWASP ASVS | Vérification de la sécurité des applications | Référentiel de tests sécurité (§14) |

## 8.6 Modèles physiques de référence (littérature scientifique — non normatifs)

Ces références ne sont pas des normes mais des publications scientifiques
ouvertes servant de base aux modèles physiques du §7 (implémentation propre et
indépendante) :

- **NREL Solar Position Algorithm (SPA)** — Reda & Andreas (2004).
- **Perez et al. (1990)** — modèle de transposition anisotrope du rayonnement diffus.
- **De Soto et al. (2006)** — modèle électrique à cinq paramètres (single-diode).
- **Sandia Photovoltaic Array Performance Model** — King et al. (2004), SNL.
- **Faiman (2008)** — modèle de température de cellule.
- **Huld et al. (2010)** — modèle de performance PVGIS (référence de comparaison).
- **Manwell & McGowan (1993)** — Kinetic Battery Model (KiBaM).
- **Rainflow counting (ASTM E1049-85)** — comptage de cycles pour fatigue/vieillissement batterie.

## 8.7 Moteur de conformité normative (traçabilité)

Le module M18 implémente un **moteur de règles déclaratif** associant chaque
exigence normative à : (1) la donnée d'entrée requise, (2) la formule de
vérification, (3) le seuil, (4) le texte de référence exact (article/paragraphe),
(5) le statut (conforme/non-conforme/non-applicable). Ce moteur est piloté par un
référentiel versionné (YAML/JSON) permettant d'ajouter de nouvelles juridictions
(pays) sans modification du code (voir §12.4 pour le schéma de données associé).
