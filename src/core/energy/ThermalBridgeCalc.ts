// ============================================================
// Thermal Bridge (Psi-value) Calculator — NTA 8800 Bijlage H
// ============================================================

/** Detail junction type */
export type DetailType =
  | 'wall-floor'
  | 'wall-roof'
  | 'wall-foundation'
  | 'window-frame'
  | 'wall-corner'
  | 'wall-internal-wall'
  | 'roof-internal-wall'
  | 'balcony';

/** Insulation position */
export type InsulationPosition = 'inside' | 'cavity' | 'outside';

/** Window frame material (for window-frame detail) */
export type FrameMaterial = 'wood' | 'plastic' | 'aluminium';

/** Input parameters for a thermal bridge calculation */
export interface ThermalBridgeInput {
  detailType: DetailType;
  rcWall: number;               // m2K/W
  rcFloorRoof: number;          // m2K/W  (floor or roof Rc, depending on type)
  insulationPosition: InsulationPosition;
  hasThermalBreak: boolean;     // relevant for balcony
  frameMaterial: FrameMaterial;  // relevant for window-frame
  frameWidth: number;           // mm, relevant for window-frame
  foundationDepth: number;      // m, relevant for wall-foundation
}

/** Output of the thermal bridge calculation */
export interface ThermalBridgeResult {
  psiCalculated: number;        // W/(mK) — interpolated from tables
  psiForfait: number;           // W/(mK) — NTA 8800 default
  detailType: DetailType;
  heatLossPerMeter: number;     // W/m at deltaT = 1K (equals psi)
  percentOfForfait: number;     // calculated / forfait * 100
}

// ============================================================
// NTA 8800 Bijlage H — Lookup Tables (simplified)
// ============================================================

/** Rc breakpoints used in interpolation */
const RC_BREAKPOINTS = [2.5, 3.5, 4.5, 5.0];

// Wall-floor psi-values indexed by [insulation position][Rc index]
// Average of wall Rc and floor Rc is used for lookup
const WALL_FLOOR_TABLE: Record<InsulationPosition, number[]> = {
  inside:  [0.35, 0.30, 0.25, 0.20],
  cavity:  [0.15, 0.10, 0.08, 0.05],
  outside: [0.10, 0.05, 0.03, 0.02],
};

// Wall-roof uses same structure as wall-floor with slight adjustment
const WALL_ROOF_TABLE: Record<InsulationPosition, number[]> = {
  inside:  [0.40, 0.35, 0.30, 0.25],
  cavity:  [0.15, 0.10, 0.07, 0.05],
  outside: [0.08, 0.05, 0.03, 0.02],
};

// Wall-foundation: higher psi due to ground contact
const WALL_FOUNDATION_TABLE: Record<InsulationPosition, number[]> = {
  inside:  [0.50, 0.45, 0.40, 0.35],
  cavity:  [0.25, 0.20, 0.15, 0.10],
  outside: [0.15, 0.10, 0.08, 0.05],
};

// Window frame: indexed by [frame material][Rc index]
const WINDOW_FRAME_TABLE: Record<FrameMaterial, number[]> = {
  wood:      [0.06, 0.05, 0.04, 0.03],
  plastic:   [0.08, 0.06, 0.05, 0.04],
  aluminium: [0.12, 0.10, 0.08, 0.06],
};

// Wall corner: simple lookup by Rc
const WALL_CORNER_VALUES = [0.08, 0.06, 0.05, 0.04];

// Wall-internal wall
const WALL_INTERNAL_WALL_VALUES = [0.15, 0.12, 0.10, 0.08];

// Roof-internal wall
const ROOF_INTERNAL_WALL_VALUES = [0.12, 0.10, 0.08, 0.06];

// Balcony: with and without thermal break
const BALCONY_VALUES = {
  withoutBreak: [1.00, 0.95, 0.90, 0.85],
  withBreak:    [0.35, 0.32, 0.30, 0.28],
};

// ============================================================
// Forfaitaire waarden (NTA 8800 defaults)
// ============================================================

const FORFAIT_VALUES: Record<DetailType, number | ((input: ThermalBridgeInput) => number)> = {
  'wall-floor':          0.50,
  'wall-roof':           0.50,
  'wall-foundation':     0.60,
  'window-frame':        (input: ThermalBridgeInput) =>
    input.frameMaterial === 'aluminium' ? 0.15 : 0.08,
  'wall-corner':         0.05,
  'wall-internal-wall':  0.10,
  'roof-internal-wall':  0.10,
  'balcony':             (input: ThermalBridgeInput) =>
    input.hasThermalBreak ? 0.30 : 0.90,
};

// ============================================================
// Interpolation helper
// ============================================================

/**
 * Linear interpolation between two lookup table entries.
 * Clamps to table bounds.
 */
function interpolateTable(rcValue: number, table: number[]): number {
  if (rcValue <= RC_BREAKPOINTS[0]) return table[0];
  if (rcValue >= RC_BREAKPOINTS[RC_BREAKPOINTS.length - 1]) return table[table.length - 1];

  for (let i = 0; i < RC_BREAKPOINTS.length - 1; i++) {
    if (rcValue >= RC_BREAKPOINTS[i] && rcValue <= RC_BREAKPOINTS[i + 1]) {
      const t = (rcValue - RC_BREAKPOINTS[i]) / (RC_BREAKPOINTS[i + 1] - RC_BREAKPOINTS[i]);
      return table[i] + t * (table[i + 1] - table[i]);
    }
  }
  return table[table.length - 1];
}

// ============================================================
// Main calculation function
// ============================================================

export function calculatePsiValue(input: ThermalBridgeInput): ThermalBridgeResult {
  let psiCalculated: number;

  // Average Rc for the two components meeting at the junction
  const rcAvg = (input.rcWall + input.rcFloorRoof) / 2;

  switch (input.detailType) {
    case 'wall-floor': {
      const table = WALL_FLOOR_TABLE[input.insulationPosition];
      psiCalculated = interpolateTable(rcAvg, table);
      break;
    }
    case 'wall-roof': {
      const table = WALL_ROOF_TABLE[input.insulationPosition];
      psiCalculated = interpolateTable(rcAvg, table);
      break;
    }
    case 'wall-foundation': {
      const table = WALL_FOUNDATION_TABLE[input.insulationPosition];
      // Foundation depth factor: deeper = slightly worse
      const depthFactor = 1.0 + Math.max(0, input.foundationDepth - 0.8) * 0.1;
      psiCalculated = interpolateTable(rcAvg, table) * depthFactor;
      break;
    }
    case 'window-frame': {
      const table = WINDOW_FRAME_TABLE[input.frameMaterial];
      psiCalculated = interpolateTable(input.rcWall, table);
      // Frame width correction: wider frames = slightly worse
      const widthFactor = 1.0 + Math.max(0, input.frameWidth - 60) * 0.002;
      psiCalculated *= widthFactor;
      break;
    }
    case 'wall-corner': {
      psiCalculated = interpolateTable(input.rcWall, WALL_CORNER_VALUES);
      break;
    }
    case 'wall-internal-wall': {
      psiCalculated = interpolateTable(input.rcWall, WALL_INTERNAL_WALL_VALUES);
      break;
    }
    case 'roof-internal-wall': {
      psiCalculated = interpolateTable(input.rcFloorRoof, ROOF_INTERNAL_WALL_VALUES);
      break;
    }
    case 'balcony': {
      const table = input.hasThermalBreak
        ? BALCONY_VALUES.withBreak
        : BALCONY_VALUES.withoutBreak;
      psiCalculated = interpolateTable(input.rcWall, table);
      break;
    }
    default:
      psiCalculated = 0.10;
  }

  // Round to 3 decimal places
  psiCalculated = Math.round(psiCalculated * 1000) / 1000;

  // Get forfait value
  const forfaitEntry = FORFAIT_VALUES[input.detailType];
  const psiForfait = typeof forfaitEntry === 'function' ? forfaitEntry(input) : forfaitEntry;

  return {
    psiCalculated,
    psiForfait,
    detailType: input.detailType,
    heatLossPerMeter: psiCalculated,   // psi = heat loss per meter at deltaT=1K
    percentOfForfait: psiForfait > 0 ? Math.round((psiCalculated / psiForfait) * 100) : 0,
  };
}

// ============================================================
// Default input factory
// ============================================================

export function createDefaultInput(): ThermalBridgeInput {
  return {
    detailType: 'wall-floor',
    rcWall: 4.5,
    rcFloorRoof: 3.5,
    insulationPosition: 'cavity',
    hasThermalBreak: false,
    frameMaterial: 'wood',
    frameWidth: 60,
    foundationDepth: 0.8,
  };
}

// ============================================================
// Detail type metadata
// ============================================================

export interface DetailTypeMeta {
  id: DetailType;
  labelKey: string;           // i18n key
  needsRcFloorRoof: boolean;  // show floor/roof Rc input
  needsInsulationPos: boolean;
  needsFrameMaterial: boolean;
  needsFrameWidth: boolean;
  needsFoundationDepth: boolean;
  needsThermalBreak: boolean;
  rcFloorRoofLabel: string;   // 'floor' | 'roof'
}

export const DETAIL_TYPE_META: DetailTypeMeta[] = [
  {
    id: 'wall-floor',
    labelKey: 'tb.wallFloor',
    needsRcFloorRoof: true,
    needsInsulationPos: true,
    needsFrameMaterial: false,
    needsFrameWidth: false,
    needsFoundationDepth: false,
    needsThermalBreak: false,
    rcFloorRoofLabel: 'floor',
  },
  {
    id: 'wall-roof',
    labelKey: 'tb.wallRoof',
    needsRcFloorRoof: true,
    needsInsulationPos: true,
    needsFrameMaterial: false,
    needsFrameWidth: false,
    needsFoundationDepth: false,
    needsThermalBreak: false,
    rcFloorRoofLabel: 'roof',
  },
  {
    id: 'wall-foundation',
    labelKey: 'tb.wallFoundation',
    needsRcFloorRoof: true,
    needsInsulationPos: true,
    needsFrameMaterial: false,
    needsFrameWidth: false,
    needsFoundationDepth: true,
    needsThermalBreak: false,
    rcFloorRoofLabel: 'floor',
  },
  {
    id: 'window-frame',
    labelKey: 'tb.windowFrame',
    needsRcFloorRoof: false,
    needsInsulationPos: false,
    needsFrameMaterial: true,
    needsFrameWidth: true,
    needsFoundationDepth: false,
    needsThermalBreak: false,
    rcFloorRoofLabel: 'floor',
  },
  {
    id: 'wall-corner',
    labelKey: 'tb.wallCorner',
    needsRcFloorRoof: false,
    needsInsulationPos: false,
    needsFrameMaterial: false,
    needsFrameWidth: false,
    needsFoundationDepth: false,
    needsThermalBreak: false,
    rcFloorRoofLabel: 'floor',
  },
  {
    id: 'wall-internal-wall',
    labelKey: 'tb.wallInternalWall',
    needsRcFloorRoof: false,
    needsInsulationPos: false,
    needsFrameMaterial: false,
    needsFrameWidth: false,
    needsFoundationDepth: false,
    needsThermalBreak: false,
    rcFloorRoofLabel: 'floor',
  },
  {
    id: 'roof-internal-wall',
    labelKey: 'tb.roofInternalWall',
    needsRcFloorRoof: true,
    needsInsulationPos: false,
    needsFrameMaterial: false,
    needsFrameWidth: false,
    needsFoundationDepth: false,
    needsThermalBreak: false,
    rcFloorRoofLabel: 'roof',
  },
  {
    id: 'balcony',
    labelKey: 'tb.balcony',
    needsRcFloorRoof: false,
    needsInsulationPos: false,
    needsFrameMaterial: false,
    needsFrameWidth: false,
    needsFoundationDepth: false,
    needsThermalBreak: true,
    rcFloorRoofLabel: 'floor',
  },
];
