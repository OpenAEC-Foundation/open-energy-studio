import type { NtaCalculationInput, NtaMaatwerkadvies, NtaRegistration } from '../nta/KernelClient';
// ============================================================
// Open Energy Studio – BENG Data Model (NTA 8800)
// ============================================================

/** Orientation for surfaces / solar panels */
export type Orientation = 'N' | 'NE' | 'E' | 'SE' | 'S' | 'SW' | 'W' | 'NW' | 'horizontal';

/** Building function */
export type BuildingFunction = 'residential' | 'office' | 'education' | 'healthcare' | 'retail' | 'industrial' | 'other';

/** Surface type */
export type SurfaceType = 'wall' | 'roof' | 'floor' | 'internal';

/** Explicit thermal boundary for future NTA transmission routes; absent in legacy projects. */
export type ThermalBoundary = 'outdoor' | 'ground' | 'unheated_space' | 'adjacent_conditioned' | 'internal';

/** Heating system type */
export type HeatingSystemType = 'hr107' | 'hr_combi' | 'heat_pump_air' | 'heat_pump_ground' | 'district_heating' | 'electric' | 'biomass';

/** Ventilation system type */
export type VentilationType = 'natural' | 'type_c' | 'type_d';

/** Cooling system type */
export type CoolingSystemType = 'none' | 'split_unit' | 'central_chiller' | 'heat_pump_reversible';

/** Hot water system type */
export type HotWaterSystemType = 'hr_combi' | 'heat_pump' | 'electric_boiler' | 'solar_boiler' | 'district_heating';

/** Solar thermal collector type */
export type SolarThermalType = 'flat_plate' | 'vacuum_tube';

// ------------------------------------------------------------
// Construction layer
// ------------------------------------------------------------
export interface IConstructionLayer {
  material: string;
  thickness: number;      // m
  lambda: number;         // W/(m·K)
}

// ------------------------------------------------------------
// Construction
// ------------------------------------------------------------
export interface IConstruction {
  id: string;
  name: string;
  layers: IConstructionLayer[];
  rcValue: number;        // m²·K/W  (computed or manual)
  uValue: number;         // W/(m²·K)
  /**
   * Exterior resistance contained in uValue, m²·K/W (NTA 8800 table C.2 /
   * C.3.3). Absent means R_se = 0,04. Towards an unheated space the kernel
   * replaces it by the space-side R_si (8.4.2.1).
   */
  exteriorSurfaceResistance?: number;
}

// ------------------------------------------------------------
// Window
// ------------------------------------------------------------
export interface IWindow {
  id: string;
  name: string;
  area: number;           // m²
  uValue: number;         // W/(m²·K)  (Uw)
  gValue: number;         // ZTA / g-value  (-)
  orientation: Orientation;
  surfaceId: string;      // parent surface
}

// ------------------------------------------------------------
// Surface (wall/roof/floor)
// ------------------------------------------------------------
export interface ISurface {
  id: string;
  name: string;
  type: SurfaceType;
  thermalBoundary?: ThermalBoundary;
  unheatedSpaceId?: string;
  area: number;           // m² gross
  orientation: Orientation;
  constructionId: string;
  zoneId: string;
  windows: IWindow[];
}

// ------------------------------------------------------------
// Thermal bridge
// ------------------------------------------------------------
export interface IThermalBridge {
  id: string;
  name: string;
  psiValue: number;       // W/(m·K)
  length: number;         // m
  zoneId: string;
  thermalBoundary?: ThermalBoundary;
  unheatedSpaceId?: string;
  /** Construction parts the bridge lies on, for the TOjuli split (NTA 8800 §5.7.2). */
  orientations?: Orientation[];
}

export interface IPointThermalBridge {
  id: string;
  name: string;
  chiValue: number;        // W/K
  zoneId: string;
  thermalBoundary?: ThermalBoundary;
  unheatedSpaceId?: string;
  /** Construction parts the bridge lies on, for the TOjuli split (NTA 8800 §5.7.2). */
  orientations?: Orientation[];
  sourceReference: string;
}

// ------------------------------------------------------------
// Air tightness
// ------------------------------------------------------------
export interface IAirTightness {
  qv10: number;           // dm³/(s·m²)  at 10 Pa
  /** Set when the value was not in the file but filled in on open (importer default 0,4); the user should check it. */
  assumed?: boolean;
}

// ------------------------------------------------------------
// Zone
// ------------------------------------------------------------
export interface IZone {
  id: string;
  name: string;
  floorArea: number;      // m²  (Ag)
  volume: number;         // m³
  height: number;         // m
  surfaces: ISurface[];
  thermalBridges: IThermalBridge[];
  /** Absent in old projects means the point-bridge inventory is unknown. */
  pointThermalBridges?: IPointThermalBridge[];
  /** Explicit reviewer confirmation; adding one point does not imply the inventory is complete. */
  pointBridgeInventoryComplete?: boolean;
  airTightness: IAirTightness;
}

// ------------------------------------------------------------
// Heating system
// ------------------------------------------------------------
export interface INtaHeatPumpPerformancePoint {
  id: string;
  service: 'space_heating' | 'domestic_hot_water' | 'space_cooling';
  sourceTemperatureC: number;
  sinkTemperatureC: number;
  usefulCapacityKw: number;
  inputPowerKw: number;
  inputEnergyCarrier: 'electricity' | 'gas' | 'district_heat' | 'other';
  testReference: string;
}

export interface INtaHeatPumpDhwTestPoint {
  id: string;
  tapProfile: string;
  usefulEnergyKwhPerDay: number;
  inputEnergyKwhPerDay: number;
  nominalCapacityKw: number;
  practiceFactor: number;
  testSetpointC: number;
  designSetpointC: number;
  sourceAirFlowM3PerHour?: number;
  sourceAirDryBulbC?: number;
  sourceAirWetBulbC?: number;
  declarationNormVersion: string;
  sourceReference: string;
}

export interface INtaHeatPumpOperatingLimits {
  minimumOperatingCop?: number;
  maximumSupplyTemperatureC?: number;
  declarationNormVersion: string;
  sourceReference: string;
}

export interface INtaHeatPumpAuxiliaryComponent {
  id: string;
  kind: 'source_pump' | 'source_fan' | 'indoor_fan' | 'distribution_pump' | 'controls_standby' | 'defrost' | 'backup_heater' | 'other';
  service: INtaHeatPumpPerformancePoint['service'];
  nominalPowerW: number;
  energyCarrier: INtaHeatPumpPerformancePoint['inputEnergyCarrier'];
  measurementBoundary: 'included_in_declared_performance' | 'additional' | 'unknown';
  evidenceReference: string;
}

export interface INtaHeatPumpSystemLink {
  id: string;
  role: 'backup_generator' | 'upstream_heat_pump' | 'shared_source' | 'source_ventilation';
  targetKind: 'heat_pump' | 'heating_system' | 'hot_water_system' | 'ventilation_system';
  targetId: string;
  evidenceReference: string;
}

export interface INtaHeatPumpInput {
  id: string;
  servedZoneIds?: string[];
  source: 'outdoor_air' | 'exhaust_air' | 'ground' | 'groundwater' | 'surface_water' | 'district_water' | 'waste_heat' | 'other';
  sink: 'indoor_air' | 'hydronic' | 'domestic_hot_water' | 'combined_hydronic_and_hot_water';
  drive: 'electric_compression' | 'gas_engine' | 'absorption';
  reversible: boolean;
  hybrid: boolean;
  booster: boolean;
  performanceEvidence: {
    kind: 'normative_default' | 'controlled_quality_declaration';
    reference: string | null;
    /** User-entered registry identity; it is not an authenticated BCRG lookup. */
    registryRecord?: {
      registrationNumber: string;
      productName: string;
      manufacturer: string;
      sourceUrl: string;
    };
  };
  /** Declared/measured points only; no normative weighting is implemented. */
  performancePoints?: INtaHeatPumpPerformancePoint[];
  /** Declared EN 16147 tap-profile inputs; never interpreted as annual NTA performance. */
  dhwTestPoints?: INtaHeatPumpDhwTestPoint[];
  /** Product-specific shutoff thresholds; no NTA dispatch or backup route is inferred. */
  declaredOperatingLimits?: INtaHeatPumpOperatingLimits;
  /** Rated auxiliaries and metering boundaries; no annual NTA use is inferred. */
  auxiliaryComponents?: INtaHeatPumpAuxiliaryComponent[];
  /** Saved measured input for the provisional chapter-9 diagnostic, never a verified result. */
  heatingAuxMeasuredDraft?: import('../nta/KernelClient').HeatingAuxMeasuredDraftInput;
  /** Saved consultation-table selection and evidence, never a verified seasonal COP. */
  forfaitHeatPumpDraft?: import('../nta/KernelClient').ForfaitHeatPumpDraftInput;
  /** Saved gas-engine/absorption consultation-table input; no verified gas use or label. */
  gasHeatPumpForfaitDraft?: import('../nta/KernelClient').GasHeatPumpForfaitDraftInput;
  /** Saved gas-generator auxiliary input; this remains a consultation-draft diagnostic. */
  gasHeatPumpAuxDraft?: import('../nta/KernelClient').GasHeatPumpAuxDraftInput;
  /** Explicit equipment relations; no operating sequence or energy share is inferred. */
  systemLinks?: INtaHeatPumpSystemLink[];
}

export interface IHeatingSystem {
  id: string;
  name: string;
  type: HeatingSystemType;
  cop: number;            // COP or efficiency (-)
  coverageFraction: number; // 0-1
  /** Classified input for the Rust NTA kernel; does not enable a calculation route. */
  ntaHeatPump?: INtaHeatPumpInput;
}

// ------------------------------------------------------------
// Ventilation system
// ------------------------------------------------------------
export interface IVentilationSystem {
  id: string;
  name: string;
  type: VentilationType;
  heatRecoveryEfficiency: number;  // 0-1  (WTW rendement)
  sfp: number;                      // W/(dm³/s)  specific fan power
}

// ------------------------------------------------------------
// Cooling system
// ------------------------------------------------------------
export interface ICoolingSystem {
  id: string;
  name: string;
  type: CoolingSystemType;
  eer: number;            // Energy Efficiency Ratio
}

// ------------------------------------------------------------
// Hot water system
// ------------------------------------------------------------
export interface IHotWaterSystem {
  id: string;
  name: string;
  type: HotWaterSystemType;
  efficiency: number;     // (-)
  hasSolarBoiler: boolean;
  solarBoilerFraction: number; // 0-1
  /** Classified input for the Rust NTA kernel; does not enable a calculation route. */
  ntaHeatPump?: INtaHeatPumpInput;
}

// ------------------------------------------------------------
// Solar PV
// ------------------------------------------------------------
export interface ISolarPV {
  id: string;
  name: string;
  peakPower: number;      // kWp
  orientation: Orientation;
  tilt: number;           // degrees
  area: number;           // m²
}

// ------------------------------------------------------------
// Solar thermal
// ------------------------------------------------------------
export interface ISolarThermal {
  id: string;
  name: string;
  collectorArea: number;  // m²
  type: SolarThermalType;
  orientation: Orientation;
  tilt: number;           // degrees
}

// ------------------------------------------------------------
// Project
// ------------------------------------------------------------
export interface IProject {
  id: string;
  name: string;
  description: string;
  buildingFunction: BuildingFunction;
  address: string;
  city: string;
  zones: IZone[];
  heatingSystems: IHeatingSystem[];
  ventilationSystems: IVentilationSystem[];
  coolingSystems: ICoolingSystem[];
  hotWaterSystems: IHotWaterSystem[];
  /** Standalone NTA equipment inventory, outside the legacy indicative calculator. */
  ntaHeatPumps?: INtaHeatPumpInput[];
  /** Supplied factors only; sources are recorded, not verified by the kernel. */
  unheatedSpaces?: Array<{
    id: string;
    name: string;
    /** Declared b_U; exclusive with `outside`. */
    reductionFactor?: number;
    factorSourceReference?: string;
    /** Derive b_U from the space's losses to outside (8.53–8.59); other project zones on the space are added. */
    outside?: {
      transmission: { elements: Array<{ id: string; areaM2: number; uValueWPerM2k: number; sourceReference: string }>; linearBridges?: Array<{ id: string; lengthM: number; psiWPerMk: number; sourceReference: string }> };
      ventilation: { method: 'flow'; airflowM3PerH: number; sourceReference: string } | { method: 'half_of_transmission' };
      otherZonesConductanceWPerK?: number;
    };
  }>;
  solarPV: ISolarPV[];
  solarThermal: ISolarThermal[];
  constructions: IConstruction[];
  /** NTA 8800 inputs the legacy model does not hold, each with a source reference. */
  ntaCalculation?: NtaCalculationInput;
  /** Registration data of the EP report (BRL 9500 §4.2.5); part of the input fingerprint. */
  registration?: NtaRegistration;
  /** Maatwerkadvies definition (BRL 9500-MWA); not part of the label fingerprint. */
  maatwerkadvies?: NtaMaatwerkadvies;
  /** ISSO 82.1/75.1 basisopname (survey) kept with the project; not part of the label fingerprint. */
  basisopname?: { kind: 'residential' | 'utility'; survey: Record<string, unknown> };
}

// ------------------------------------------------------------
// BENG Results
// ------------------------------------------------------------
export interface IEnergyBreakdown {
  transmissionLoss: number;       // kWh/year
  ventilationLoss: number;        // kWh/year
  infiltrationLoss: number;       // kWh/year
  solarGain: number;              // kWh/year
  internalGain: number;           // kWh/year
  heatingDemand: number;          // kWh/year
  coolingDemand: number;          // kWh/year
  heatingEnergy: number;          // kWh/year  (delivered)
  coolingEnergy: number;          // kWh/year  (delivered)
  ventilationEnergy: number;      // kWh/year  (fan energy)
  hotWaterEnergy: number;         // kWh/year  (delivered)
  lightingEnergy: number;         // kWh/year
  auxiliaryEnergy: number;        // kWh/year  (pumps, standby, controls)
  totalPrimaryEnergy: number;     // kWh/year  (gross fossil, incl. auxiliary)
  renewableEnergy: number;        // kWh/year
  pvProduction: number;           // kWh/year
  solarThermalProduction: number; // kWh/year
  /** Kernel only: heating gains not in solar/internal (sunroom gains 7.37 and other terms). */
  otherGain?: number;               // kWh/year
  /** Kernel only: the per-service delivered energy is unavailable, so the delivered bars are not zero but unknown. */
  deliveredUnavailable?: boolean;
}

export interface IBENGResult {
  beng1: number;                  // kWh/(m²·year) - Energy demand
  beng2: number;                  // kWh/(m²·year) - Primary fossil energy
  beng3: number;                  // % - Renewable energy share
  beng1Limit: number;
  beng2Limit: number;
  beng3Limit: number;
  beng1Pass: boolean;
  beng2Pass: boolean;
  beng3Pass: boolean;
  breakdown: IEnergyBreakdown;
  totalFloorArea: number;         // m²
}

// ------------------------------------------------------------
// UI State
// ------------------------------------------------------------
export type ViewMode = 'project' | 'envelope' | 'installations' | 'renewables' | 'results' | 'report' | 'model3d' | 'uvalue-calc' | 'thermal-bridge-calc' | 'heat-pump-sizing';

export type RibbonTab = 'start' | 'envelope' | 'installations' | 'renewables' | 'results' | 'report' | 'model3d' | 'tools';

export type DialogType =
  | 'project-info'
  | 'zone-editor'
  | 'construction-editor'
  | 'surface-editor'
  | 'window-editor'
  | 'thermal-bridge'
  | 'point-bridge'
  | 'air-tightness'
  | 'heating-system'
  | 'ventilation-system'
  | 'cooling-system'
  | 'hot-water-system'
  | 'solar-pv'
  | 'solar-thermal'
  | null;

export interface IDialogState {
  type: DialogType;
  editId: string | null;       // null = new, string = editing existing
}

// ------------------------------------------------------------
// Monthly calculation types
// ------------------------------------------------------------

/** Tuple of 12 monthly values (Jan..Dec) */
export type MonthlyValues = [number, number, number, number, number, number,
                              number, number, number, number, number, number];

/** Monthly energy breakdown for a single month */
export interface IMonthlyBreakdown {
  month: number;                    // 0-11
  transmissionLoss: number;         // kWh
  ventilationLoss: number;          // kWh
  infiltrationLoss: number;         // kWh
  solarGain: number;                // kWh
  internalGain: number;             // kWh
  heatingDemand: number;            // kWh
  coolingDemand: number;            // kWh
  utilizationFactorHeating: number; // 0..1
  utilizationFactorCooling: number; // 0..1
}

/** TO-juli (summer comfort / overheating) result */
export interface ITOJuliResult {
  gto: number;                      // Gewogen Temperatuur Overschrijding
  limit: number;                    // 1.20 (residential)
  pass: boolean;
  monthlyRisk: number[];            // 12 monthly values
}

/** Extended BENG result with monthly breakdown */
export interface IBENGResultMonthly extends IBENGResult {
  monthly: IMonthlyBreakdown[];
  toJuli: ITOJuliResult;
  monthlyPVProduction: MonthlyValues;
  monthlySolarThermalProduction: MonthlyValues;
}
