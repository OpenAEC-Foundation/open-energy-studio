// ============================================================
// Open Energy Studio – BENG Data Model (NTA 8800)
// ============================================================

/** Orientation for surfaces / solar panels */
export type Orientation = 'N' | 'NE' | 'E' | 'SE' | 'S' | 'SW' | 'W' | 'NW' | 'horizontal';

/** Building function */
export type BuildingFunction = 'residential' | 'office' | 'education' | 'healthcare' | 'retail' | 'industrial' | 'other';

/** Surface type */
export type SurfaceType = 'wall' | 'roof' | 'floor' | 'internal';

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
}

// ------------------------------------------------------------
// Air tightness
// ------------------------------------------------------------
export interface IAirTightness {
  qv10: number;           // dm³/(s·m²)  at 10 Pa
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
  airTightness: IAirTightness;
}

// ------------------------------------------------------------
// Heating system
// ------------------------------------------------------------
export interface IHeatingSystem {
  id: string;
  name: string;
  type: HeatingSystemType;
  cop: number;            // COP or efficiency (-)
  coverageFraction: number; // 0-1
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
  solarPV: ISolarPV[];
  solarThermal: ISolarThermal[];
  constructions: IConstruction[];
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
  totalPrimaryEnergy: number;     // kWh/year
  renewableEnergy: number;        // kWh/year
  pvProduction: number;           // kWh/year
  solarThermalProduction: number; // kWh/year
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
export type ViewMode = 'project' | 'envelope' | 'installations' | 'renewables' | 'results' | 'report';

export type RibbonTab = 'start' | 'envelope' | 'installations' | 'renewables' | 'results' | 'report';

export type DialogType =
  | 'project-info'
  | 'zone-editor'
  | 'construction-editor'
  | 'surface-editor'
  | 'window-editor'
  | 'thermal-bridge'
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
