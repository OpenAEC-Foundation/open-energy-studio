// ============================================================
// Open Energy Studio – Energy Label Classification
// Based on NTA 8800 / BENG primary fossil energy indicator
// ============================================================

export type EnergyLabelClass =
  | 'A++++' | 'A+++' | 'A++' | 'A+' | 'A'
  | 'B' | 'C' | 'D' | 'E' | 'F' | 'G';

export interface EnergyLabelResult {
  label: EnergyLabelClass;
  color: string;
}

/**
 * Determine the energy label based on BENG2 (primary fossil energy per m²).
 *
 * Classification thresholds for dwellings, Omgevingsregeling annex IX
 * (article 5.11, fourth paragraph; consolidated version 2026-01-01):
 *   A++++  ≤ 0
 *   A+++   ≤ 50
 *   A++    ≤ 75
 *   A+     ≤ 105
 *   A      ≤ 160
 *   B      ≤ 190
 *   C      ≤ 250
 *   D      ≤ 290
 *   E      ≤ 335
 *   F      ≤ 380
 *   G      > 380
 */
export function calculateEnergyLabel(beng2: number): EnergyLabelResult {
  if (beng2 <= 0)   return { label: 'A++++', color: '#006837' };
  if (beng2 <= 50)  return { label: 'A+++',  color: '#1a9641' };
  if (beng2 <= 75)  return { label: 'A++',   color: '#66bd63' };
  if (beng2 <= 105) return { label: 'A+',    color: '#a6d96a' };
  if (beng2 <= 160) return { label: 'A',     color: '#d9ef8b' };
  if (beng2 <= 190) return { label: 'B',     color: '#fee08b' };
  if (beng2 <= 250) return { label: 'C',     color: '#fdae61' };
  if (beng2 <= 290) return { label: 'D',     color: '#f46d43' };
  if (beng2 <= 335) return { label: 'E',     color: '#d73027' };
  if (beng2 <= 380) return { label: 'F',     color: '#a50026' };
  return              { label: 'G',     color: '#67001f' };
}
