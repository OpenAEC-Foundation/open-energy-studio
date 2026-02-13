// ============================================================
// Open Energy Studio – Construction Rc/U-value Calculator
// ============================================================

import type { IConstructionLayer, SurfaceType } from './types';
import { SURFACE_RESISTANCE } from './Constants';

/**
 * Calculate the thermal resistance (Rc) of a construction from its layers.
 *
 * Rc = sum of (thickness / lambda) for each layer.
 *
 * @param layers - Construction layers with material, thickness, and lambda
 * @returns Thermal resistance Rc in m²·K/W
 */
export function calculateRc(layers: IConstructionLayer[]): number {
  let rc = 0;
  for (const layer of layers) {
    if (layer.lambda > 0) {
      rc += layer.thickness / layer.lambda;
    }
  }
  return rc;
}

/**
 * Calculate the U-value of a construction including surface resistances.
 *
 * U = 1 / (Rsi + Rc + Rse)
 *
 * @param rc - Thermal resistance of the construction in m²·K/W
 * @param surfaceType - Type of surface (wall, roof, floor) for correct Rsi/Rse
 * @returns U-value in W/(m²·K)
 */
export function calculateU(rc: number, surfaceType: SurfaceType = 'wall'): number {
  const resistance = SURFACE_RESISTANCE[surfaceType] ?? SURFACE_RESISTANCE.wall;
  const totalR = resistance.rsi + rc + resistance.rse;
  if (totalR <= 0) return 10; // fallback for invalid input
  return 1 / totalR;
}

/**
 * Calculate both Rc and U-value from layers and surface type.
 *
 * @param layers - Construction layers
 * @param surfaceType - Type of surface
 * @returns Object with rc and u values
 */
export function calculateRcU(
  layers: IConstructionLayer[],
  surfaceType: SurfaceType = 'wall'
): { rc: number; u: number } {
  const rc = calculateRc(layers);
  const u = calculateU(rc, surfaceType);
  return { rc, u };
}
