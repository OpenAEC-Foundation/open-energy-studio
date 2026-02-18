// ============================================================
// Open Energy Studio – Heat Pump Sizing Calculation
// EN 12831 / NTA 8800 design heat load for heat pump dimensioning
// ============================================================

import type { IZone, IConstruction, BuildingFunction, IWindow } from './types';
import { calculateTransmissionLossCoefficient } from './MonthlyTransmissionLoss';
import { QV10_CORRECTION_FACTOR } from './Constants';

// ------------------------------------------------------------
// Design parameters per building function (EN 12831 / NTA 8800)
// ------------------------------------------------------------

export interface BuildingFunctionParams {
  /** Design indoor temperature (°C) */
  thetaInt: number;
  /** Design ventilation rate (dm³/s·m²) */
  ventilationRate: number;
  /** Reheat factor (W/m²) */
  reheatFactor: number;
}

export const BUILDING_FUNCTION_PARAMS: Record<BuildingFunction, BuildingFunctionParams> = {
  residential:  { thetaInt: 20, ventilationRate: 0.9, reheatFactor: 11 },
  office:       { thetaInt: 20, ventilationRate: 1.3, reheatFactor: 15 },
  education:    { thetaInt: 20, ventilationRate: 1.5, reheatFactor: 15 },
  healthcare:   { thetaInt: 22, ventilationRate: 1.5, reheatFactor: 11 },
  retail:       { thetaInt: 20, ventilationRate: 1.0, reheatFactor: 15 },
  industrial:   { thetaInt: 18, ventilationRate: 1.0, reheatFactor: 20 },
  other:        { thetaInt: 20, ventilationRate: 1.0, reheatFactor: 11 },
};

/** Design outdoor temperature for the Netherlands, climate zone I (NEN-EN 12831) */
export const DESIGN_OUTDOOR_TEMP = -10; // °C

// Air volumetric heat capacity: ρ × c_p = 1.2 kg/m³ × 1000 J/(kg·K) = 1200 J/(m³·K)

// ------------------------------------------------------------
// Detailed result types
// ------------------------------------------------------------

export interface SurfaceLossDetail {
  surfaceId: string;
  surfaceName: string;
  surfaceType: string;
  grossArea: number;       // m²
  netArea: number;         // m² (gross - windows)
  uValue: number;          // W/(m²·K)
  deltaT: number;          // K
  heatLoss: number;        // W
}

export interface WindowLossDetail {
  windowId: string;
  windowName: string;
  surfaceName: string;
  area: number;            // m²
  uValue: number;          // W/(m²·K)
  deltaT: number;          // K
  heatLoss: number;        // W
}

export interface ThermalBridgeLossDetail {
  bridgeId: string;
  bridgeName: string;
  psiValue: number;        // W/(m·K)
  length: number;          // m
  deltaT: number;          // K
  heatLoss: number;        // W
}

export interface ZoneSizingResult {
  zoneId: string;
  zoneName: string;
  floorArea: number;       // m²
  volume: number;          // m³

  // Detailed losses
  surfaceLosses: SurfaceLossDetail[];
  windowLosses: WindowLossDetail[];
  thermalBridgeLosses: ThermalBridgeLossDetail[];

  // Aggregated per zone (W)
  transmissionLoss: number;
  ventilationLoss: number;
  infiltrationLoss: number;
  reheatAllowance: number;
  totalZoneLoss: number;
}

export interface HeatPumpSizingResult {
  // Input parameters used
  designOutdoorTemp: number;
  designIndoorTemp: number;
  deltaT: number;
  buildingFunction: BuildingFunction;
  safetyMargin: number;    // fraction (e.g. 0.10)
  includeReheat: boolean;

  // Per-zone results
  zones: ZoneSizingResult[];

  // Building totals (W)
  totalTransmissionLoss: number;
  totalVentilationLoss: number;
  totalInfiltrationLoss: number;
  totalReheatAllowance: number;
  totalDesignLoad: number;

  // With safety margin
  totalWithMargin: number;
  recommendedKW: number;

  // Cross-check
  htTransmission: number;  // W/K from reusable function
  totalFloorArea: number;
  specificLoad: number;     // W/m²
}

// ------------------------------------------------------------
// Main calculation
// ------------------------------------------------------------

export interface HeatPumpSizingInput {
  zones: IZone[];
  constructions: IConstruction[];
  buildingFunction: BuildingFunction;
  ventilationSystems: { type: string; heatRecoveryEfficiency: number }[];
  designOutdoorTemp?: number;    // default: -10°C
  designIndoorTemp?: number;     // default: from building function
  safetyMargin?: number;         // default: 0.10 (10%)
  includeReheat?: boolean;       // default: true
}

export function calculateHeatPumpSizing(input: HeatPumpSizingInput): HeatPumpSizingResult {
  const {
    zones,
    constructions,
    buildingFunction,
    ventilationSystems,
    designOutdoorTemp = DESIGN_OUTDOOR_TEMP,
    safetyMargin = 0.10,
    includeReheat = true,
  } = input;

  const params = BUILDING_FUNCTION_PARAMS[buildingFunction];
  const designIndoorTemp = input.designIndoorTemp ?? params.thetaInt;
  const deltaT = designIndoorTemp - designOutdoorTemp;

  // Build construction lookup
  const constructionMap = new Map<string, IConstruction>();
  for (const c of constructions) {
    constructionMap.set(c.id, c);
  }

  // Determine heat recovery efficiency from ventilation systems
  const wtwEfficiency = ventilationSystems.length > 0
    ? Math.max(...ventilationSystems.map(v => v.heatRecoveryEfficiency))
    : 0;

  const zoneResults: ZoneSizingResult[] = [];

  for (const zone of zones) {
    const surfaceLosses: SurfaceLossDetail[] = [];
    const windowLosses: WindowLossDetail[] = [];
    const thermalBridgeLosses: ThermalBridgeLossDetail[] = [];

    // --- Surfaces and windows ---
    for (const surface of zone.surfaces) {
      const construction = constructionMap.get(surface.constructionId);
      if (!construction) continue;

      const windowArea = surface.windows.reduce((sum: number, w: IWindow) => sum + w.area, 0);
      const netArea = Math.max(0, surface.area - windowArea);

      // Opaque surface loss
      const surfaceLoss = construction.uValue * netArea * deltaT;
      surfaceLosses.push({
        surfaceId: surface.id,
        surfaceName: surface.name,
        surfaceType: surface.type,
        grossArea: surface.area,
        netArea,
        uValue: construction.uValue,
        deltaT,
        heatLoss: surfaceLoss,
      });

      // Window losses
      for (const win of surface.windows) {
        const winLoss = win.uValue * win.area * deltaT;
        windowLosses.push({
          windowId: win.id,
          windowName: win.name,
          surfaceName: surface.name,
          area: win.area,
          uValue: win.uValue,
          deltaT,
          heatLoss: winLoss,
        });
      }
    }

    // --- Thermal bridges ---
    for (const tb of zone.thermalBridges) {
      const tbLoss = tb.psiValue * tb.length * deltaT;
      thermalBridgeLosses.push({
        bridgeId: tb.id,
        bridgeName: tb.name,
        psiValue: tb.psiValue,
        length: tb.length,
        deltaT,
        heatLoss: tbLoss,
      });
    }

    // --- Ventilation loss ---
    // Φ_v = q_v × ρc_p × (1 - η_wtw) × ΔT
    // q_v in dm³/s = ventilationRate × floorArea
    // Convert dm³/s to m³/s: ÷ 1000
    // ρc_p = 1200 J/(m³·K) = 1200 W·s/(m³·K)
    // So: Φ_v = (ventRate × A / 1000) × 1200 × (1 - η) × ΔT
    //        = ventRate × A × 1.2 × (1 - η) × ΔT
    const qv = params.ventilationRate * zone.floorArea; // dm³/s
    const ventilationLoss = qv / 1000 * 1200 * (1 - wtwEfficiency) * deltaT;

    // --- Infiltration loss ---
    // H_i = qv10 × A_env × correction_factor × ρc_p
    // Simplified: use qv10 × floor_area × QV10_CORRECTION_FACTOR as flow rate
    // Then multiply by 1200 (ρc_p) and ΔT
    const qvInf = zone.airTightness.qv10 * zone.floorArea * QV10_CORRECTION_FACTOR; // dm³/s
    const infiltrationLoss = qvInf / 1000 * 1200 * deltaT;

    // --- Reheat allowance ---
    const reheatAllowance = includeReheat ? params.reheatFactor * zone.floorArea : 0;

    // --- Zone totals ---
    const transmissionLoss =
      surfaceLosses.reduce((sum, s) => sum + s.heatLoss, 0) +
      windowLosses.reduce((sum, w) => sum + w.heatLoss, 0) +
      thermalBridgeLosses.reduce((sum, t) => sum + t.heatLoss, 0);

    const totalZoneLoss = transmissionLoss + ventilationLoss + infiltrationLoss + reheatAllowance;

    zoneResults.push({
      zoneId: zone.id,
      zoneName: zone.name,
      floorArea: zone.floorArea,
      volume: zone.volume,
      surfaceLosses,
      windowLosses,
      thermalBridgeLosses,
      transmissionLoss,
      ventilationLoss,
      infiltrationLoss,
      reheatAllowance,
      totalZoneLoss,
    });
  }

  // --- Building totals ---
  const totalTransmissionLoss = zoneResults.reduce((sum, z) => sum + z.transmissionLoss, 0);
  const totalVentilationLoss = zoneResults.reduce((sum, z) => sum + z.ventilationLoss, 0);
  const totalInfiltrationLoss = zoneResults.reduce((sum, z) => sum + z.infiltrationLoss, 0);
  const totalReheatAllowance = zoneResults.reduce((sum, z) => sum + z.reheatAllowance, 0);
  const totalDesignLoad = totalTransmissionLoss + totalVentilationLoss + totalInfiltrationLoss + totalReheatAllowance;

  const totalWithMargin = totalDesignLoad * (1 + safetyMargin);
  const recommendedKW = Math.ceil(totalWithMargin / 100) / 10; // round up to 0.1 kW

  // Cross-check: Ht from reusable function
  const htTransmission = calculateTransmissionLossCoefficient(zones, constructions);
  const totalFloorArea = zones.reduce((sum, z) => sum + z.floorArea, 0);
  const specificLoad = totalFloorArea > 0 ? totalDesignLoad / totalFloorArea : 0;

  return {
    designOutdoorTemp,
    designIndoorTemp,
    deltaT,
    buildingFunction,
    safetyMargin,
    includeReheat,
    zones: zoneResults,
    totalTransmissionLoss,
    totalVentilationLoss,
    totalInfiltrationLoss,
    totalReheatAllowance,
    totalDesignLoad,
    totalWithMargin,
    recommendedKW,
    htTransmission,
    totalFloorArea,
    specificLoad,
  };
}
