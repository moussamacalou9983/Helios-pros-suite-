# 15. Feuille de route de développement

## Phase 0 — Conception (ce dossier) — Terminée
Livrables : cahier des charges, analyses fonctionnelle et besoins, architecture,
modules, algorithmes, modèles physiques, normes, structure, classes, UML, BDD,
formats, stratégie de tests, feuille de route.

## Phase 1 — Socle scientifique & MVP PV (fondation)
**Objectif** : valider la précision scientifique avant tout développement UI avancé.
- Moteur `simulation-engine` (Rust) : géométrie solaire, transposition Perez,
  modèle électrique 5-paramètres, modèle thermique, modèle onduleur.
- Tests de validation scientifique (§14.4) et golden datasets.
- Service Météo (import TMY3/EPW, connecteur PVGIS).
- Service SIG minimal (localisation, MNT, ombrage lointain par ray-casting).
- Dimensionnement PV de base (calepinage manuel, vérifications électriques
  RG-002/003) et service Base Équipements (import CSV/Excel).
- API REST V1 (projets, scénarios, simulation).
- Frontend web minimal : création de projet, saisie système, visualisation
  résultats (productible, PR, pertes).
- Rapport PDF basique.
- **Critère de sortie de phase** : écart de productible annuel < 3 % sur un
  panel de ≥ 10 sites de validation documentés publiquement.

## Phase 2 — SIG avancé, 3D, optimisation
- Modélisation 3D complète (obstacles, LiDAR, ombrage proche par ray-casting BVH).
- Calepinage automatique (bin-packing contraint).
- Trackers et backtracking.
- Bifacial et agrivoltaïsme (view-factor, albédo variable).
- Service Optimisation (NSGA-II, MILP pour sous-problèmes).
- Analyse économique complète (LCOE, VAN/TRI, sensibilité).
- Rapports personnalisables (templates).
- Collaboration multi-utilisateur (édition simultanée, historique de révisions).

## Phase 3 — Multi-technologies (différenciation majeure)
- Module Stockage batteries (KiBaM, stratégies de pilotage, vieillissement).
- Module Micro-réseaux (dispatch règles + MILP, LPSP, groupe électrogène).
- Module Pompage solaire (couplage direct, courbes pompe, stockage hydraulique).
- Module IRVE (pilotage solar-following, foisonnement).
- Moteur de conformité normative (M18) multi-juridictions (FR, UE, extensions).
- Version Desktop (Tauri) avec moteur WASM embarqué (mode offline/confidentiel).

## Phase 4 — Intelligence artificielle & écosystème
- Service IA : pré-dimensionnement automatique, détection d'anomalies,
  prévision production/consommation, assistant conversationnel.
- API publique enrichie (GraphQL, webhooks), marketplace de templates de rapport.
- Extensions de localisation (normes DE/UK/US, langues additionnelles).
- Mode on-premise packagé (Helm chart autonome) pour BE à contraintes de
  confidentialité renforcées.

## Phase 5 — Montée en échelle et certification
- Optimisation de performance pour portefeuilles > 500 MWc / >100k modules.
- Certification/audit qualité tiers (SMSI ISO/IEC 27001, revue par bureau de
  contrôle du moteur de calcul).
- Programme de retour d'expérience terrain (comparaison production
  simulée/mesurée sur parc de projets réels clients, boucle d'amélioration
  continue des modèles physiques).
- V2G (Vehicle-to-Grid), micro-réseaux multi-sources avancés (MPC), scénarios
  de changement climatique sur les données météo.

## Organisation des équipes par phase (indicatif)

| Rôle | Phase 1 | Phase 2 | Phase 3 | Phase 4-5 |
|---|---|---|---|---|
| Ingénieur PV senior | Validation modèles physiques | Bifacial/agrivoltaïsme | Conformité normative | REX terrain |
| Ingénieur logiciel / architecte | Moteur Rust, API | Optimisation, collaboration | Microservices multi-domaines | Scalabilité |
| Expert calcul scientifique | Modèles §7 | View-factor, trackers | KiBaM, hydraulique | — |
| Expert optimisation | — | NSGA-II/MILP | Dispatch MILP | MPC |
| Expert BDD | Schéma core/equipment | TimescaleDB, PostGIS avancé | Versionnement SCD2 | Partitionnement à l'échelle |
| Expert SIG | Ray-casting horizon | BVH ombrage proche, LiDAR | — | — |
| Expert IA | — | — | — | Prévision, anomalies, assistant |
| UI/UX | Frontend minimal | 3D, calepinage, collaboration | Multi-domaines dashboards | i18n, accessibilité |
| QA | Golden datasets, CI | Tests E2E | Tests conformité | Certification tierce |

## Gouvernance de la conception

Ce dossier de conception (`docs/design/`) est un document vivant : toute
évolution significative de périmètre ou d'architecture au fil du développement
doit être répercutée ici avant implémentation, afin de conserver la
traçabilité entre exigences (§1), fonctions (§2) et code.
