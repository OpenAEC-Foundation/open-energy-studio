// ============================================================
// Open Energy Studio – NTA 8800 Constants
// ============================================================

import type { Orientation, BuildingFunction, SurfaceType } from '../energy/types';

// ------------------------------------------------------------
// Climate data (Netherlands)
// ------------------------------------------------------------

/** Heating degree days for the Netherlands reference climate */
export const HEATING_DEGREE_DAYS = 2600; // K·d

// ------------------------------------------------------------
// Primary energy factors (NTA 8800)
// ------------------------------------------------------------

export const PRIMARY_ENERGY_FACTOR_GAS = 1.0;
export const PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL = 1.45;
export const PRIMARY_ENERGY_FACTOR_ELECTRICITY_RENEWABLE = 0;
export const PRIMARY_ENERGY_FACTOR_DISTRICT_HEATING = 0.8;
export const PRIMARY_ENERGY_FACTOR_BIOMASS = 0.5;

export interface PrimaryEnergyFactors {
  fossil: number;
  renewable: number;
}

export const PRIMARY_ENERGY_FACTORS: Record<string, PrimaryEnergyFactors> = {
  gas:              { fossil: PRIMARY_ENERGY_FACTOR_GAS, renewable: 0 },
  electricity:      { fossil: PRIMARY_ENERGY_FACTOR_ELECTRICITY_FOSSIL, renewable: PRIMARY_ENERGY_FACTOR_ELECTRICITY_RENEWABLE },
  district_heating: { fossil: PRIMARY_ENERGY_FACTOR_DISTRICT_HEATING, renewable: 0 },
  biomass:          { fossil: PRIMARY_ENERGY_FACTOR_BIOMASS, renewable: 0 },
};

// ------------------------------------------------------------
// BENG limits per building function
// ------------------------------------------------------------

export interface BENGLimits {
  beng1: number; // kWh/(m2·year) max energy demand
  beng2: number; // kWh/(m2·year) max primary fossil energy
  beng3: number; // % min renewable energy share
}

export const BENG_LIMITS: Record<BuildingFunction, BENGLimits> = {
  residential: { beng1: 70, beng2: 25, beng3: 50 },
  office:      { beng1: 50, beng2: 40, beng3: 50 },
  education:   { beng1: 70, beng2: 60, beng3: 50 },
  healthcare:  { beng1: 120, beng2: 80, beng3: 50 },
  retail:      { beng1: 70, beng2: 60, beng3: 50 },
  industrial:  { beng1: 100, beng2: 80, beng3: 50 },
  other:       { beng1: 100, beng2: 80, beng3: 50 },
};

// ------------------------------------------------------------
// Solar radiation per orientation (kWh/(m2·year))
// Netherlands reference climate, vertical surface
// ------------------------------------------------------------

export const SOLAR_RADIATION: Record<Orientation, number> = {
  S:          1100,
  SE:         1000,
  SW:         1000,
  E:           850,
  W:           850,
  NE:          600,
  NW:          600,
  N:           400,
  horizontal: 1050,
};

// ------------------------------------------------------------
// Internal gains
// ------------------------------------------------------------

/** Internal heat gains per building function in W/m2 */
export const INTERNAL_GAIN_POWER: Record<BuildingFunction, number> = {
  residential: 5,
  office:      8,
  education:   6,
  healthcare:  8,
  retail:      6,
  industrial:  4,
  other:       5,
};

/** Usage hours per year */
export const USAGE_HOURS_PER_YEAR = 8760;

// ------------------------------------------------------------
// Hot water demand (forfaitair)
// ------------------------------------------------------------

/** Hot water demand per building function in kWh/year (non-residential) */
export const HOT_WATER_DEMAND: Record<BuildingFunction, number> = {
  residential: 2100,  // Not used directly; see getHotWaterDemand()
  office:      500,
  education:   300,
  healthcare:  3000,
  retail:      200,
  industrial:  300,
  other:       500,
};

/**
 * Calculate hot water demand based on building function and floor area.
 *
 * For residential, NTA 8800 Ch 13 scales demand with number of tap points
 * (which correlates with Ag). Approximation:
 *   Q_W = max(1800, 25 × Ag + 200) kWh/year
 *
 * This gives ~1875 kWh for 67 m² (vs Uniec ~1750) and
 * ~3525 kWh for 133 m² (vs Uniec ~4060).
 *
 * For non-residential: fixed values from HOT_WATER_DEMAND table.
 */
export function getHotWaterDemand(
  buildingFunction: BuildingFunction,
  totalFloorArea: number
): number {
  if (buildingFunction === 'residential') {
    return Math.max(1800, 25 * totalFloorArea + 200);
  }
  return HOT_WATER_DEMAND[buildingFunction] ?? HOT_WATER_DEMAND.other;
}

// ------------------------------------------------------------
// Lighting energy demand
// ------------------------------------------------------------

/** Lighting energy per building function in kWh/(m2·year)
 * NTA 8800 §14.3: W_L;spec = 0 for residential BENG calculations
 * Utility functions use Table 14.1/14.3/14.6 (simplified here) */
export const LIGHTING_ENERGY: Record<BuildingFunction, number> = {
  residential: 0,   // NTA 8800 §14.3: 0 for BENG
  office:      12,
  education:   10,
  healthcare:  15,
  retail:      20,
  industrial:  8,
  other:       8,
};

// ------------------------------------------------------------
// Surface heat transfer resistances (m2·K/W)
// ------------------------------------------------------------

export interface SurfaceResistance {
  rsi: number; // interior surface resistance
  rse: number; // exterior surface resistance
}

/**
 * NTA 8800 table C.2 (p. 778). A construction U-value always carries
 * R_se = 0,04 (C.10, p. 777), also for an `internal` surface: towards an
 * unheated space the kernel swaps R_se for the space-side R_si itself
 * (8.4.2.1, p. 266), so a U with R_si on both sides would be corrected twice.
 */
export const SURFACE_RESISTANCE: Record<SurfaceType, SurfaceResistance> = {
  wall:     { rsi: 0.13, rse: 0.04 },
  roof:     { rsi: 0.10, rse: 0.04 },
  floor:    { rsi: 0.17, rse: 0.04 },
  internal: { rsi: 0.13, rse: 0.04 },
};

// ------------------------------------------------------------
// Ventilation constants
// ------------------------------------------------------------

/** Design flow rate for residential in dm³/(s·m²) – used for heat loss calculation */
export const VENTILATION_DESIGN_FLOW_RATE_RESIDENTIAL = 0.9;

/** Time-average factor for fan energy (NTA 8800: demand-controlled ≈ 0.67)
 *  Fan energy = SFP × q_v;design × f_time × 8760 / 1000 */
export const VENTILATION_FAN_TIME_FACTOR = 0.67;

/** Correction factor for qv10 to average infiltration rate */
export const QV10_CORRECTION_FACTOR = 0.067;

/** Distribution loss factors (NTA 8800 §10/§13 simplified)
 *  Accounts for pipe/duct losses not captured in net demand */
export const DISTRIBUTION_LOSS_FACTOR_HEATING = 0.10;
export const DISTRIBUTION_LOSS_FACTOR_HOT_WATER = 0.15;

// ------------------------------------------------------------
// Frame factor for windows (fraction of glass area vs total)
// ------------------------------------------------------------

export const FRAME_FACTOR = 0.9;

// ------------------------------------------------------------
// PV performance ratio
// ------------------------------------------------------------

/** Standard performance ratio for PV panels */
export const PV_PERFORMANCE_RATIO = 0.85;

// ------------------------------------------------------------
// Ventilation type defaults
// ------------------------------------------------------------

import type { VentilationType } from '../energy/types';

export interface VentilationTypeDefaults {
  hasFan: boolean;
  hasHeatRecovery: boolean;
  defaultSfp: number;         // W/(dm³/s)
  defaultHeatRecovery: number; // 0-1
}

export const VENTILATION_TYPE_DEFAULTS: Record<VentilationType, VentilationTypeDefaults> = {
  natural: { hasFan: false, hasHeatRecovery: false, defaultSfp: 0, defaultHeatRecovery: 0 },
  type_c:  { hasFan: true,  hasHeatRecovery: false, defaultSfp: 0.5, defaultHeatRecovery: 0 },
  type_d:  { hasFan: true,  hasHeatRecovery: true,  defaultSfp: 0.8, defaultHeatRecovery: 0.85 },
};

// ------------------------------------------------------------
// TO-juli (summer comfort) constants
// ------------------------------------------------------------

/** GTO limit for residential buildings */
export const TO_JULI_LIMIT = 1.20;

/** Base temperature for overheating calculation (°C) */
export const TO_JULI_BASE_TEMP = 25;
