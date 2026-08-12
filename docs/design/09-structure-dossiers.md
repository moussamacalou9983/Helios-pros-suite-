# 9. Structure complète des dossiers

Organisation en **monorepo** (facilite le versionnage cohérent domaine/API/UI),
avec séparation stricte des couches (Clean Architecture) par service.

```
heliospro/
├── apps/
│   ├── web/                        # Application web React/TypeScript
│   │   ├── src/
│   │   │   ├── pages/
│   │   │   ├── components/
│   │   │   ├── features/           # un dossier par domaine (project, pv-design, storage, gis, reports...)
│   │   │   ├── hooks/
│   │   │   ├── store/               # état global (Zustand/Redux)
│   │   │   ├── api/                 # clients générés depuis l'OpenAPI/GraphQL schema
│   │   │   ├── i18n/
│   │   │   └── styles/
│   │   ├── public/
│   │   └── tests/
│   └── desktop/                    # Wrapper Tauri (réutilise apps/web)
│       ├── src-tauri/
│       └── tauri.conf.json
│
├── services/
│   ├── gateway/                    # API Gateway / BFF (FastAPI)
│   ├── auth/                       # Authentification, RBAC/ABAC, licences
│   ├── projects/                   # Gestion de projet, versionnage, collaboration
│   ├── gis/                        # SIG, MNT/MNS, ombrage, calepinage
│   ├── meteo/                      # Agrégation météo, TMY
│   ├── simulation-engine/          # Moteur Rust (cœur physique) + binding Python
│   │   ├── crates/
│   │   │   ├── solar-geometry/     # Position solaire, rayonnement
│   │   │   ├── pv-electrical/      # Modèle single-diode, courbes I-V
│   │   │   ├── shading/            # Ray-casting, view-factor
│   │   │   ├── inverter/           # Modèle onduleur
│   │   │   ├── battery/            # KiBaM, vieillissement
│   │   │   ├── hydraulics/         # Pompage solaire
│   │   │   ├── microgrid-dispatch/ # Dispatch règles + interface solveur MILP
│   │   │   └── core-sim/           # Orchestrateur de simulation horaire
│   │   ├── bindings/
│   │   │   ├── python/             # PyO3
│   │   │   └── wasm/               # wasm-bindgen (exécution client)
│   │   └── tests/
│   │       ├── unit/
│   │       └── validation/         # comparaison à jeux de données de référence publiés
│   ├── optimization/               # NSGA-II, MILP, optimisation bayésienne
│   ├── economics/                  # LCOE, VAN/TRI, cashflows
│   ├── ai-assistant/                # Prévision, détection anomalies, recommandations
│   │   ├── models/
│   │   ├── training/
│   │   └── inference/
│   ├── equipment-catalog/          # Base équipements (modules, onduleurs, batteries, pompes, câbles)
│   ├── reporting/                  # Génération PDF/DOCX/Excel, templates
│   ├── compliance/                 # Moteur de règles normatives (M18)
│   └── notifications/
│
├── libs/                           # Bibliothèques partagées entre services
│   ├── common-python/              # Types, erreurs, utilitaires partagés (Python)
│   ├── common-ts/                  # Types partagés frontend (générés depuis OpenAPI)
│   ├── proto/                      # Définitions gRPC/Protobuf partagées
│   └── design-system/              # Composants UI partagés (React)
│
├── infra/
│   ├── docker/                     # Dockerfiles par service
│   ├── helm/                       # Charts Helm (SaaS + on-premise)
│   ├── terraform/                  # IaC (réseau, bases de données, clusters)
│   └── ci/                         # Pipelines CI/CD
│
├── data/
│   ├── reference-datasets/         # Jeux de données publics de validation (NREL, PVGIS...)
│   ├── standards-rulesets/         # Référentiels normatifs versionnés (YAML/JSON, §8.7)
│   └── seed/                       # Données d'amorçage (équipements de démonstration)
│
├── docs/
│   ├── design/                     # Ce dossier de conception
│   ├── api/                        # Documentation API générée (OpenAPI/GraphQL)
│   └── user-guide/
│
├── scripts/                        # Scripts d'automatisation (dev, migration, seed)
├── .github/workflows/              # CI/CD
├── docker-compose.yml              # Environnement de développement local
├── LICENSE
└── README.md
```

## Conventions

- Un service = un domaine métier, propriétaire de ses données (pas de base
  partagée entre services hors lecture via API).
- Chaque service Python suit l'architecture hexagonale :
  `domain/` (entités, règles métier pures, sans dépendance framework),
  `application/` (cas d'usage, orchestration), `infrastructure/` (DB, API
  externes, framework web), `interfaces/` (contrôleurs REST/gRPC).
- Le moteur Rust (`simulation-engine`) est le seul composant partagé sous forme
  de bibliothèque compilée (crate), jamais dupliqué en logique métier ailleurs.
