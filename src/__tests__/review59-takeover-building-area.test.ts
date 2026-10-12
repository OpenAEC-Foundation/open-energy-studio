import { describe, expect, it } from 'vitest';
import { surveyTakeover } from '../core/nta/SurveyTakeover';
import type { BuildingPerformanceInput } from '../core/nta/KernelClient';

// Review 59: the hot water of a single-zone utility survey serves the whole
// building (13.20: its function areas add up to at most A_g of the building).
// Taken over into a project with more than one zone, its areas follow the
// building's area, not the first zone's; the lighting (14.4) stays per zone.
const derived = {
  spaceHeating: { demand: { zoneId: 'utiliteit', usableFloorAreaM2: 1000 } },
  hotWater: { need: { areas: [{ function: 'office', areaM2: 1000 }] } },
  lighting: [{ zoneId: 'utiliteit', lightingZones: [{ areaM2: 1000 }] }],
} as unknown as BuildingPerformanceInput;

const hotWaterArea = (next: Record<string, unknown>) =>
  ((next.hotWater as { need: { areas: Array<{ areaM2: number }> } }).need.areas[0].areaM2);
const lightingArea = (next: Record<string, unknown>) =>
  ((next.lighting as Array<{ lightingZones: Array<{ areaM2: number }> }>)[0].lightingZones[0].areaM2);

describe('survey takeover into a project with several zones', () => {
  it('scales the hot-water areas to the building, the lighting to its zone', () => {
    const { next } = surveyTakeover(derived, {}, { id: 'z1', floorArea: 400, buildingFloorArea: 1000 });
    expect(hotWaterArea(next)).toBeCloseTo(1000);
    expect(lightingArea(next)).toBeCloseTo(400);
  });

  it('keeps the single-zone behaviour when the building is the zone', () => {
    const { next } = surveyTakeover(derived, {}, { id: 'z1', floorArea: 400 });
    expect(hotWaterArea(next)).toBeCloseTo(400);
    expect(lightingArea(next)).toBeCloseTo(400);
  });
});
