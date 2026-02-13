// ============================================================
// Open Energy Studio – NTA 8800:2020+A1:2020 Climate Data
// Reference climate: De Bilt (Netherlands)
// ============================================================

import type { Orientation } from './types';

// ------------------------------------------------------------
// Table 17.1 – Monthly climate data
// ------------------------------------------------------------

/** Monthly average outdoor temperature θ_e (°C) – NTA 8800 Table 17.1 */
export const MONTHLY_AVERAGE_TEMPERATURE: readonly number[] = [
   2.61,  // Jan
   4.82,  // Feb
   5.91,  // Mar
   9.32,  // Apr
  14.73,  // May
  16.12,  // Jun
  18.05,  // Jul
  18.48,  // Aug
  15.63,  // Sep
  10.40,  // Oct
   7.99,  // Nov
   4.00,  // Dec
] as const;

/** Hours per month t_mi (h) – NTA 8800 Table 17.1 */
export const HOURS_PER_MONTH: readonly number[] = [
  744,  // Jan (31d)
  672,  // Feb (28d)
  744,  // Mar (31d)
  720,  // Apr (30d)
  744,  // May (31d)
  720,  // Jun (30d)
  744,  // Jul (31d)
  744,  // Aug (31d)
  720,  // Sep (30d)
  744,  // Oct (31d)
  720,  // Nov (30d)
  744,  // Dec (31d)
] as const;

/** Days per month */
export const DAYS_PER_MONTH: readonly number[] = [
  31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
] as const;

/** Heating setpoint temperature (°C) – NTA 8800 */
export const HEATING_SETPOINT = 20;

// ------------------------------------------------------------
// Heating degree days (derived from NTA 8800 temperatures)
// HDD[m] = max(0, (θ_set - θ_e[m])) × days[m]
// Kept for backward compatibility
// ------------------------------------------------------------

export const MONTHLY_HEATING_DEGREE_DAYS: readonly number[] =
  DAYS_PER_MONTH.map((d, m) =>
    Math.max(0, (HEATING_SETPOINT - MONTHLY_AVERAGE_TEMPERATURE[m]) * d)
  );

// ------------------------------------------------------------
// Table 17.2 – Solar irradiance I_sol;mi (W/m²)
// Monthly average, ground reflectance ρ = 0.2
// Organized by tilt angle β, then orientation γ
//
// NTA 8800 orientations:
//   180°=S, 225°=SW, 270°=W, 315°=NW, 360°=N, 45°=NE, 90°=E, 135°=SE
// ------------------------------------------------------------

type OrientationKey = Exclude<Orientation, 'horizontal'>;

/** Solar irradiance in W/m² per orientation per month, for a given tilt */
interface TiltData {
  S:  readonly number[];
  SW: readonly number[];
  W:  readonly number[];
  NW: readonly number[];
  N:  readonly number[];
  NE: readonly number[];
  E:  readonly number[];
  SE: readonly number[];
}

/** β = 0° (horizontal) – single row, orientation-independent */
const IRRADIANCE_0: readonly number[] = [
  28.0, 49.3, 96.6, 160.5, 197.0, 209.3, 191.0, 177.2, 123.9, 73.2, 34.3, 21.0,
];

/** β = 30° */
const IRRADIANCE_30: TiltData = {
  S:  [50.5, 69.1, 122.5, 189.5, 211.1, 211.2, 196.1, 197.9, 154.0, 102.4, 54.8, 38.3],
  SW: [44.4, 61.2, 109.3, 174.5, 201.5, 210.7, 193.2, 198.3, 146.2,  91.5, 47.7, 32.6],
  W:  [29.0, 46.2,  87.7, 146.5, 179.9, 199.4, 180.2, 178.4, 121.1,  68.8, 32.9, 20.6],
  NW: [16.2, 32.9,  66.7, 115.6, 155.8, 180.6, 162.1, 147.6,  91.6,  47.3, 20.5, 12.5],
  N:  [14.9, 27.2,  56.4, 104.6, 148.5, 171.0, 153.0, 125.8,  73.7,  36.3, 18.6, 12.2],
  NE: [15.8, 34.5,  72.8, 125.1, 160.6, 173.0, 156.9, 127.5,  86.5,  48.9, 20.9, 12.5],
  E:  [26.9, 49.4,  97.6, 158.9, 186.3, 189.7, 175.0, 152.8, 113.7,  71.6, 33.8, 21.2],
  SE: [42.2, 63.7, 117.7, 184.1, 206.3, 204.4, 190.0, 179.3, 140.1,  93.6, 48.6, 33.1],
};

/** β = 45° */
const IRRADIANCE_45: TiltData = {
  S:  [57.9, 74.1, 126.6, 189.7, 202.7, 197.3, 185.0, 193.5, 157.6, 109.4, 61.0, 44.1],
  SW: [49.4, 63.2, 109.1, 171.0, 191.1, 199.3, 182.5, 194.9, 147.0,  94.2, 51.1, 36.1],
  W:  [28.7, 44.0,  82.0, 136.7, 164.4, 186.2, 166.8, 169.8, 115.3,  64.8, 31.3, 19.9],
  NW: [14.9, 29.2,  56.6,  96.5, 128.7, 156.3, 139.0, 127.2,  78.0,  40.2, 18.5, 11.7],
  N:  [14.3, 25.9,  44.3,  70.0, 113.6, 139.6, 123.5,  91.5,  52.9,  33.5, 17.8, 11.7],
  NE: [14.5, 30.4,  63.1, 107.1, 134.5, 145.9, 132.7, 102.9,  72.2,  41.4, 18.8, 11.7],
  E:  [26.2, 47.9,  94.2, 152.2, 172.0, 173.3, 160.4, 137.9, 106.2,  68.4, 32.4, 20.5],
  SE: [46.3, 66.5, 120.2, 183.5, 197.3, 190.7, 179.1, 171.0, 139.2,  97.2, 52.2, 36.7],
};

/** β = 60° */
const IRRADIANCE_60: TiltData = {
  S:  [62.2, 75.4, 124.3, 180.2, 184.5, 175.1, 165.9, 179.7, 153.3, 110.7, 63.9, 47.4],
  SW: [51.8, 62.1, 103.9, 160.4, 173.4, 180.9, 165.4, 182.9, 141.5,  92.6, 51.8, 37.6],
  W:  [27.8, 41.1,  74.8, 125.1, 146.3, 169.1, 150.6, 156.9, 107.2,  59.9, 28.9, 19.0],
  NW: [13.8, 26.4,  49.6,  83.1, 107.5, 134.1, 119.2, 110.2,  68.6,  35.9, 17.0, 10.9],
  N:  [13.4, 24.1,  41.5,  57.8,  78.5, 102.9,  90.4,  68.0,  48.6,  31.5, 16.6, 10.9],
  NE: [13.5, 27.3,  56.3,  93.9, 113.2, 123.3, 112.3,  85.8,  62.3,  36.6, 17.3, 10.9],
  E:  [24.7, 45.4,  88.5, 142.0, 154.7, 154.5, 143.2, 122.0,  97.2,  63.5, 30.4, 19.6],
  SE: [48.1, 66.3, 116.9, 174.2, 179.9, 170.7, 161.8, 156.4, 132.6,  96.0, 53.2, 38.4],
};

/** β = 90° (vertical walls) */
const IRRADIANCE_90: TiltData = {
  S:  [60.1, 66.7, 101.8, 135.1, 124.9, 112.7, 109.7, 128.5, 122.3, 96.2, 59.5, 46.2],
  SW: [48.1, 52.2,  82.1, 121.9, 122.1, 127.8, 117.1, 137.1, 112.2, 76.3, 45.6, 34.9],
  W:  [23.4, 32.8,  57.3,  96.2, 107.3, 125.7, 112.7, 120.0,  83.9, 46.7, 22.7, 15.2],
  NW: [11.4, 20.9,  38.5,  64.1,  78.9,  97.8,  88.5,  83.1,  53.6, 28.7, 13.8,  8.9],
  N:  [11.1, 19.5,  34.8,  49.4,  61.9,  73.0,  66.7,  55.9,  41.4, 26.4, 13.6,  8.9],
  NE: [11.1, 21.5,  44.2,  72.9,  82.9,  92.0,  81.2,  63.9,  47.9, 29.1, 14.0,  8.9],
  E:  [20.2, 36.5,  70.7, 112.2, 114.6, 114.8, 104.9,  89.0,  73.7, 49.8, 23.9, 15.9],
  SE: [43.9, 56.8,  95.4, 135.8, 128.4, 118.0, 113.2, 112.4, 103.6, 80.3, 47.1, 35.8],
};

/** All tilt data tables indexed by angle */
const TILT_TABLES: { angle: number; data: TiltData }[] = [
  { angle: 30, data: IRRADIANCE_30 },
  { angle: 45, data: IRRADIANCE_45 },
  { angle: 60, data: IRRADIANCE_60 },
  { angle: 90, data: IRRADIANCE_90 },
];

// ------------------------------------------------------------
// Lookup functions
// ------------------------------------------------------------

/**
 * Map our Orientation type to a TiltData key.
 * For 'horizontal', returns 'S' (NTA 8800: use South for tilt < 15°).
 */
function mapOrientation(orientation: Orientation): OrientationKey {
  if (orientation === 'horizontal') return 'S';
  return orientation;
}

/**
 * Get the raw solar irradiance I_sol;mi (W/m²) for a specific
 * orientation, tilt angle, and month.
 *
 * Linearly interpolates between available tilt angles per NTA 8800 §17.2.
 *
 * @param orientation - Surface compass orientation
 * @param tiltDeg - Tilt angle in degrees (0=horizontal, 90=vertical)
 * @param month - Month index (0=Jan, 11=Dec)
 * @returns Irradiance in W/m²
 */
export function getSolarIrradiance(
  orientation: Orientation,
  tiltDeg: number,
  month: number
): number {
  const tilt = Math.max(0, Math.min(90, tiltDeg));
  const ori = mapOrientation(orientation);

  // β = 0°: horizontal, orientation-independent
  if (tilt < 15) {
    // NTA 8800: for tilt < 15°, use South orientation
    // Interpolate between horizontal (0°) and 30° South
    if (tilt <= 0) return IRRADIANCE_0[month];
    const t = tilt / 30; // 0..0.5
    return IRRADIANCE_0[month] * (1 - t) + IRRADIANCE_30.S[month] * t;
  }

  // Find bracketing tilt angles
  for (let i = 0; i < TILT_TABLES.length - 1; i++) {
    const lo = TILT_TABLES[i];
    const hi = TILT_TABLES[i + 1];

    if (tilt >= lo.angle && tilt <= hi.angle) {
      const t = (tilt - lo.angle) / (hi.angle - lo.angle);
      return lo.data[ori][month] * (1 - t) + hi.data[ori][month] * t;
    }
  }

  // Below 30° (but >= 15): interpolate between horizontal and 30°
  if (tilt < 30) {
    const t = (tilt - 0) / 30;
    return IRRADIANCE_0[month] * (1 - t) + IRRADIANCE_30[ori][month] * t;
  }

  // 90° or above: use vertical
  return IRRADIANCE_90[ori][month];
}

/**
 * Get monthly solar energy for a tilted surface.
 * Converts I_sol;mi (W/m²) → E_sol (kWh/m²) per month.
 *
 * E_sol[m] = I_sol;mi[m] × t_mi[m] / 1000
 *
 * @param orientation - Surface compass orientation
 * @param tiltDeg - Tilt angle in degrees (0=horizontal, 90=vertical)
 * @returns Array of 12 monthly values in kWh/m²
 */
export function getSolarEnergyMonthly(
  orientation: Orientation,
  tiltDeg: number
): number[] {
  const result: number[] = [];
  for (let m = 0; m < 12; m++) {
    const irr = getSolarIrradiance(orientation, tiltDeg, m);
    result.push(irr * HOURS_PER_MONTH[m] / 1000);
  }
  return result;
}

// ------------------------------------------------------------
// MONTHLY_SOLAR_RADIATION – β=90° wall values in kWh/m²
// Backward-compatible export for MonthlySolarGain.ts
// ------------------------------------------------------------

function toMonthlyEnergy(irradiance: readonly number[]): readonly number[] {
  return irradiance.map((val, m) => Math.round(val * HOURS_PER_MONTH[m] / 10) / 100);
}

export const MONTHLY_SOLAR_RADIATION: Record<Orientation, readonly number[]> = {
  S:          toMonthlyEnergy(IRRADIANCE_90.S),
  SE:         toMonthlyEnergy(IRRADIANCE_90.SE),
  SW:         toMonthlyEnergy(IRRADIANCE_90.SW),
  E:          toMonthlyEnergy(IRRADIANCE_90.E),
  W:          toMonthlyEnergy(IRRADIANCE_90.W),
  NE:         toMonthlyEnergy(IRRADIANCE_90.NE),
  NW:         toMonthlyEnergy(IRRADIANCE_90.NW),
  N:          toMonthlyEnergy(IRRADIANCE_90.N),
  horizontal: toMonthlyEnergy(IRRADIANCE_0),
} as const;

// ------------------------------------------------------------
// Legacy PV tilt factor (kept for reference, not used in NTA 8800 path)
// ------------------------------------------------------------

/**
 * Get monthly solar radiation for a given orientation (vertical walls).
 * @deprecated Use getSolarEnergyMonthly() for tilt-specific radiation
 */
export function getMonthlySolarRadiation(orientation: Orientation): readonly number[] {
  return MONTHLY_SOLAR_RADIATION[orientation] ?? MONTHLY_SOLAR_RADIATION.S;
}

/**
 * @deprecated Use getSolarEnergyMonthly() with actual tilt angle instead
 */
export function getPVTiltFactor(orientation: Orientation, tilt: number): number {
  if (orientation === 'horizontal') return 0.90;
  const TILT_FACTOR_BASE: Record<Orientation, number> = {
    S: 1.00, SE: 0.95, SW: 0.95, E: 0.82, W: 0.82,
    NE: 0.60, NW: 0.60, N: 0.45, horizontal: 0.90,
  };
  const baseFactor = TILT_FACTOR_BASE[orientation];
  const optimalTilt = 35;
  const tiltDeviation = Math.abs(tilt - optimalTilt) / 90;
  const tiltPenalty = tiltDeviation * tiltDeviation * 0.3;
  return Math.max(0.3, baseFactor * (1 - tiltPenalty));
}
