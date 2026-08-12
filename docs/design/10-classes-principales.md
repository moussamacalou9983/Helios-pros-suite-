# 10. Classes principales

Pseudocode orienté-objet (représentatif de l'implémentation Python/Rust réelle,
architecture hexagonale — ces classes appartiennent à la couche `domain`).

## 10.1 Gestion de projet

```
class Organization:
    id: UUID
    name: str
    tenant_schema: str
    subscription: SubscriptionPlan

class User:
    id: UUID
    organization_id: UUID
    email: str
    role: Role                 # Viewer, Engineer, LeadEngineer, Admin
    permissions: list[Permission]

class Project:
    id: UUID
    organization_id: UUID
    name: str
    site: Site
    scenarios: list[Scenario]
    created_by: UUID
    revisions: list[ProjectRevision]
    status: ProjectStatus       # Draft, InProgress, Validated, Archived

class Scenario:
    id: UUID
    project_id: UUID
    name: str
    pv_system: PVSystem | None
    storage_system: StorageSystem | None
    microgrid: Microgrid | None
    pumping_system: PumpingSystem | None
    ev_charging: EVChargingSystem | None
    load_profile: LoadProfile
    simulation_result: SimulationResult | None
    economic_result: EconomicResult | None
```

## 10.2 Site et SIG

```
class Site:
    id: UUID
    latitude: float
    longitude: float
    altitude: float
    timezone: str
    terrain_model: TerrainModel          # MNT/MNS
    obstacles: list[Obstacle3D]
    building_footprint: Geometry | None
    albedo_profile: AlbedoProfile

class TerrainModel:
    resolution_m: float
    elevation_grid: ndarray
    source: str                          # LiDAR, SRTM, import utilisateur

class Obstacle3D:
    geometry: Mesh3D
    height_m: float
    type: ObstacleType                   # building, tree, chimney, tracker_row...
```

## 10.3 Météo

```
class WeatherSeries:
    site_id: UUID
    timestep_minutes: int
    source: WeatherSource                # PVGIS, NASA_POWER, Meteonorm, imported
    records: list[WeatherRecord]
    quality_flags: list[QualityFlag]

class WeatherRecord:
    timestamp: datetime
    ghi: float
    dni: float
    dhi: float
    temp_ambient: float
    wind_speed: float
    relative_humidity: float | None
```

## 10.4 Générateur PV

```
class PVModule:
    id: UUID
    manufacturer: str
    model: str
    p_stc_w: float
    voc_v: float
    isc_a: float
    vmpp_v: float
    impp_a: float
    temp_coeff_pmax: float
    temp_coeff_voc: float
    temp_coeff_isc: float
    technology: CellTechnology            # mono_perc, bifacial, thin_film...
    bifacial_factor: float | None
    certifications: list[str]              # IEC 61215, IEC 61730...

class Inverter:
    id: UUID
    manufacturer: str
    model: str
    p_ac_max_w: float
    v_dc_max_v: float
    v_dc_mppt_range: tuple[float, float]
    n_mppt: int
    efficiency_curve: EfficiencyCurve
    certifications: list[str]

class StringConfig:
    module: PVModule
    modules_in_series: int
    strings_in_parallel: int
    tilt_deg: float
    azimuth_deg: float
    mppt_id: str

class PVArray:
    inverter: Inverter
    strings: list[StringConfig]
    cabling: CablingPlan
    tracker: TrackerConfig | None

class PVSystem:
    arrays: list[PVArray]
    shading_scene: ShadingScene
    losses: LossParameters

class CablingPlan:
    dc_cables: list[CableSegment]
    ac_cables: list[CableSegment]
    protections: list[ProtectionDevice]

class ProtectionDevice:
    type: ProtectionType                  # fuse, breaker, spd
    rating_a: float | None
    standard_reference: str                # ex. "IEC 62548 §X"
```

## 10.5 Stockage, microgrid, pompage, IRVE

```
class Battery:
    id: UUID
    chemistry: BatteryChemistry           # Li_NMC, Li_LFP, LeadAcid
    capacity_kwh: float
    max_charge_kw: float
    max_discharge_kw: float
    round_trip_efficiency: float
    soc_min: float
    soc_max: float
    cycle_life_curve: CycleLifeCurve

class StorageSystem:
    batteries: list[Battery]
    control_strategy: ControlStrategy      # SelfConsumption, PeakShaving, Arbitrage, Backup

class Microgrid:
    pv_system: PVSystem
    storage_system: StorageSystem | None
    generator: DieselGenerator | None
    grid_connection: GridConnection | None
    dispatch_strategy: DispatchStrategy    # RuleBased or Optimized(MILP)
    reliability_target: float              # LPSP cible

class PumpingSystem:
    pump_curve: PumpCurve
    motor: Motor
    tdh_m: float                           # hauteur manométrique totale
    water_demand_profile: DemandProfile
    reservoir: WaterReservoir | None

class EVChargingSystem:
    chargers: list[EVCharger]
    arrival_profiles: list[VehicleProfile]
    control_mode: EVControlMode            # Uncontrolled, SolarFollowing, V2G
```

## 10.6 Simulation et résultats

```
class SimulationEngine:
    def run(self, scenario: Scenario, weather: WeatherSeries,
             years: int, degradation: DegradationModel) -> SimulationResult: ...

class SimulationResult:
    timestep_minutes: int
    energy_series: TimeSeries              # production, consommation, SOC, dispatch...
    loss_breakdown: LossBreakdown
    performance_indicators: PerformanceIndicators   # PR, Yield, taux autoconsommation...
    reliability_indicators: ReliabilityIndicators | None   # LPSP...

class LossBreakdown:
    shading_pct: float
    iam_pct: float
    soiling_pct: float
    mismatch_pct: float
    dc_cabling_pct: float
    inverter_pct: float
    ac_cabling_pct: float
    degradation_pct: float
    availability_pct: float
```

## 10.7 Optimisation et économie

```
class OptimizationProblem:
    scenario_template: Scenario
    design_variables: list[DesignVariable]  # ex. tilt, n_modules, battery_kwh...
    objectives: list[Objective]              # minimiser LCOE, maximiser couverture...
    constraints: list[Constraint]            # RG-002, RG-003, budget...

class OptimizationResult:
    pareto_front: list[EvaluatedSolution]
    recommended_solution: EvaluatedSolution

class EconomicModel:
    capex: CapexBreakdown
    opex: OpexAssumptions
    tariff_scenario: TariffScenario
    financing: FinancingScheme

class EconomicResult:
    lcoe: float
    npv: float
    irr: float
    payback_years: float
    cashflows: TimeSeries
```

## 10.8 Base équipements, conformité, IA

```
class EquipmentCatalog:
    modules: list[PVModule]
    inverters: list[Inverter]
    batteries: list[Battery]
    pumps: list[PumpCurve]
    cables: list[CableSpec]
    version: str
    last_synced_at: datetime

class ComplianceRule:
    id: str
    standard_reference: str                 # "IEC 60364-7-712 §712.x"
    jurisdiction: str                        # FR, EU, generic...
    input_fields: list[str]
    check_expression: str                    # DSL déclaratif
    severity: Severity                       # Blocking, Warning, Info

class ComplianceReport:
    project_id: UUID
    results: list[ComplianceCheckResult]

class AIAssistantRequest:
    context: ProjectContext
    intent: AssistantIntent                  # PreSizing, AnomalyCheck, Forecast, Chat
```
