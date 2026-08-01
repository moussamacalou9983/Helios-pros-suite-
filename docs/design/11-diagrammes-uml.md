# 11. Diagrammes UML

## 11.1 Diagramme de classes — Domaine « Générateur PV »

```mermaid
classDiagram
  class Project {
    +UUID id
    +String name
    +Site site
    +List~Scenario~ scenarios
  }
  class Scenario {
    +UUID id
    +PVSystem pvSystem
    +StorageSystem storageSystem
    +SimulationResult simulationResult
  }
  class PVSystem {
    +List~PVArray~ arrays
    +ShadingScene shadingScene
    +LossParameters losses
  }
  class PVArray {
    +Inverter inverter
    +List~StringConfig~ strings
    +CablingPlan cabling
    +TrackerConfig tracker
  }
  class StringConfig {
    +PVModule module
    +int modulesInSeries
    +int stringsInParallel
    +float tiltDeg
    +float azimuthDeg
  }
  class PVModule {
    +float pStcW
    +float vocV
    +float iscA
    +float tempCoeffPmax
  }
  class Inverter {
    +float pAcMaxW
    +Tuple vDcMpptRange
    +EfficiencyCurve efficiencyCurve
  }
  class CablingPlan {
    +List~CableSegment~ dcCables
    +List~CableSegment~ acCables
    +List~ProtectionDevice~ protections
  }

  Project "1" --> "*" Scenario
  Scenario "1" --> "0..1" PVSystem
  PVSystem "1" --> "*" PVArray
  PVArray "1" --> "*" StringConfig
  PVArray "1" --> "1" Inverter
  PVArray "1" --> "1" CablingPlan
  StringConfig "*" --> "1" PVModule
```

## 11.2 Diagramme de classes — Domaine « Stockage & Microgrid »

```mermaid
classDiagram
  class Microgrid {
    +PVSystem pvSystem
    +StorageSystem storageSystem
    +DieselGenerator generator
    +DispatchStrategy dispatchStrategy
    +float reliabilityTarget
  }
  class StorageSystem {
    +List~Battery~ batteries
    +ControlStrategy controlStrategy
  }
  class Battery {
    +BatteryChemistry chemistry
    +float capacityKwh
    +float roundTripEfficiency
    +CycleLifeCurve cycleLifeCurve
  }
  class DispatchStrategy {
    <<interface>>
    +compute(state, forecast) DispatchDecision
  }
  class RuleBasedDispatch {
    +compute(state, forecast) DispatchDecision
  }
  class OptimizedMILPDispatch {
    +compute(state, forecast) DispatchDecision
  }

  Microgrid "1" --> "0..1" StorageSystem
  StorageSystem "1" --> "*" Battery
  Microgrid "1" --> "1" DispatchStrategy
  DispatchStrategy <|.. RuleBasedDispatch
  DispatchStrategy <|.. OptimizedMILPDispatch
```

## 11.3 Diagramme de séquence — Lancement d'une simulation

```mermaid
sequenceDiagram
  actor U as Utilisateur
  participant UI as Frontend
  participant GW as API Gateway
  participant SIM as Service Simulation
  participant Q as File de tâches
  participant W as Worker Rust/Python
  participant TS as TimescaleDB

  U->>UI: Clique "Lancer simulation"
  UI->>GW: POST /scenarios/{id}/simulate
  GW->>SIM: Valider requête, créer job
  SIM->>Q: Enfiler job (idempotency key)
  SIM-->>UI: 202 Accepted + job_id
  Q->>W: Distribuer job
  W->>W: Exécuter moteur physique (8760+ pas)
  W->>TS: Écrire série temporelle résultat
  W->>Q: Publier "simulation.completed"
  Q-->>SIM: Notifier complétion
  SIM-->>UI: WebSocket push (progress puis résultat)
  UI-->>U: Afficher indicateurs de performance
```

## 11.4 Diagramme d'état — Cycle de vie d'un projet

```mermaid
stateDiagram-v2
  [*] --> Draft
  Draft --> InProgress: Ajout site/système
  InProgress --> Simulated: Simulation exécutée
  Simulated --> Optimized: Optimisation exécutée
  Simulated --> Validated: Validation ingénieur
  Optimized --> Validated: Sélection solution retenue
  Validated --> ReportGenerated: Génération rapport
  ReportGenerated --> Archived: Clôture projet
  InProgress --> Draft: Modification majeure
  Validated --> InProgress: Révision demandée
  Archived --> [*]
```

## 11.5 Diagramme de composants — Moteur de simulation

```mermaid
flowchart TB
  subgraph SimEngine[simulation-engine (Rust)]
    SG[solar-geometry]
    SH[shading]
    PVE[pv-electrical]
    INV[inverter]
    BAT[battery]
    HYD[hydraulics]
    DISP[microgrid-dispatch]
    CORE[core-sim orchestrator]
  end
  CORE --> SG
  CORE --> SH
  CORE --> PVE
  CORE --> INV
  CORE --> BAT
  CORE --> HYD
  CORE --> DISP
  PYBIND[Binding Python PyO3]
  WASMBIND[Binding WASM]
  CORE --> PYBIND
  CORE --> WASMBIND
  PYBIND --> APIService[Service Orchestration Python/FastAPI]
```

## 11.6 Diagramme de déploiement

```mermaid
flowchart TB
  subgraph Client
    Browser
    DesktopApp[App Desktop Tauri]
  end
  subgraph K8s[Cluster Kubernetes]
    GW[API Gateway]
    Services[Microservices métier]
    Workers[Pool Workers Simulation/Optimisation]
  end
  subgraph Data[Couche données]
    PG[(PostgreSQL/PostGIS)]
    TSDB[(TimescaleDB)]
    S3[(Object Storage S3)]
    RedisC[(Redis)]
  end
  Browser -->|HTTPS| GW
  DesktopApp -->|HTTPS / local WASM| GW
  GW --> Services
  Services --> Workers
  Services --> PG
  Services --> TSDB
  Services --> S3
  Services --> RedisC
```

## 11.7 Diagramme de cas d'utilisation détaillé (Optimisation)

```mermaid
flowchart LR
  Engineer((Ingénieur BE))
  Engineer --> DefObj[Définir objectifs et contraintes]
  DefObj --> RunOpt[Lancer optimisation]
  RunOpt --> ViewPareto[Consulter front de Pareto]
  ViewPareto --> SelectSol[Sélectionner une solution]
  SelectSol --> CreateScenario[Créer scénario à partir de la solution]
  RunOpt -.include.-> ValidateConstraints[Vérifier contraintes bloquantes RG-002/003]
```
