# HeliosPro Engineering Suite — Dossier de Conception

**Version :** 0.1.0 (Phase de conception)
**Date :** 2026-08-01
**Statut :** Conception complète — développement non démarré

## Vision produit

HeliosPro est une suite logicielle professionnelle de dimensionnement et de simulation
énergétique destinée aux bureaux d'études, EPC, développeurs de projets et ingénieurs
indépendants. Elle couvre le photovoltaïque (résidentiel, tertiaire, centrales au sol,
agrivoltaïsme, flottant), le stockage par batteries, les micro-réseaux hybrides, le
pompage solaire et les infrastructures de recharge pour véhicules électriques (IRVE),
avec simulation énergétique horaire/sub-horaire, analyse technico-économique,
cartographie SIG native et assistance par intelligence artificielle.

HeliosPro ne reprend ni le code, ni les interfaces, ni les formats propriétaires
d'Archelios Calc, PVsyst ou PV*SOL. Il s'appuie exclusivement sur des modèles physiques
publiés (littérature scientifique, NREL/Sandia, IEC), des normes internationales
ouvertes, et une architecture logicielle moderne conçue indépendamment.

## Objectifs de dépassement (vs. Archelios Calc / PVsyst / PV*SOL)

| Axe | Limite constatée chez les concurrents | Réponse HeliosPro |
|---|---|---|
| Multi-technologies | Outils cloisonnés (PV seul, ou StorageVet séparé) | PV + stockage + microgrid + pompage + IRVE dans un modèle énergétique unique et couplé |
| Collaboration | Fichiers projet locaux, pas de travail simultané | Projets cloud versionnés, édition collaborative, historique complet |
| SIG | Import de coordonnées seul | SIG natif (couches cadastrales, occupation des sols, LiDAR, MNT/MNS) intégré au moteur d'ombrage |
| Optimisation | Simulation manuelle « essai-erreur » | Optimisation multi-objectif automatisée (coût, LCOE, taux de couverture, empreinte carbone) |
| IA | Absente ou marginale | Assistant de dimensionnement, prévision, détection d'anomalies, auto-tuning des ratios |
| Interopérabilité | Formats propriétaires fermés | API REST/GraphQL ouverte, IFC/BIM, GeoJSON, EPW, formats d'échange documentés |
| Traçabilité normative | Rapports figés | Moteur de conformité normative traçant chaque exigence (IEC/NF) jusqu'au résultat |
| Déploiement | Desktop Windows uniquement | Web + Desktop (Tauri) + on-premise, cloud-agnostic |

## Sommaire du dossier

1. [Cahier des charges](01-cahier-des-charges.md)
2. [Analyse fonctionnelle](02-analyse-fonctionnelle.md)
3. [Analyse des besoins des bureaux d'études](03-analyse-besoins-bureaux-etudes.md)
4. [Architecture logicielle](04-architecture-logicielle.md)
5. [Modules du logiciel](05-modules.md)
6. [Algorithmes](06-algorithmes.md)
7. [Modèles physiques](07-modeles-physiques.md)
8. [Normes IEC/ISO et références](08-normes-references.md)
9. [Structure des dossiers](09-structure-dossiers.md)
10. [Classes principales](10-classes-principales.md)
11. [Diagrammes UML](11-diagrammes-uml.md)
12. [Bases de données](12-bases-de-donnees.md)
13. [Formats d'import/export](13-formats-import-export.md)
14. [Stratégie de tests](14-strategie-tests.md)
15. [Feuille de route](15-feuille-de-route.md)

## Prochaine étape

Une fois ce dossier validé par le porteur de projet, le développement démarrera
module par module en suivant la feuille de route (§15), en commençant par le
noyau de calcul (moteur solaire + modèle électrique PV + simulation horaire),
qui constitue le socle de validation scientifique de l'ensemble de la suite.
