// @vitest-environment node
import { describe, expect, it } from 'vitest';
// @ts-expect-error jsdom ships without type declarations here; only DOMParser is used.
import { JSDOM } from 'jsdom';
import { createDefaultProject } from '../context/EnergyContext';
import { exportToVABI, importFromVABI } from '../core/io/VABIElementsBridge';

// fflate needs the Node realm; the importer only needs a DOMParser.
globalThis.DOMParser = new JSDOM().window.DOMParser;

describe('VABI import (NTA 8800 table C.2)', () => {
  it('imports VABI constructions with the R_si of their element type', () => {
    const project = createDefaultProject();
    const imported = importFromVABI(exportToVABI(project));
    for (const surface of imported.zones.flatMap((zone) => zone.surfaces)) {
      const construction = imported.constructions.find((item) => item.id === surface.constructionId);
      if (!construction || construction.rcValue <= 0) continue;
      const rsi = { wall: 0.13, roof: 0.10, floor: 0.17, internal: 0.13 }[surface.type];
      expect(construction.uValue).toBeCloseTo(1 / (construction.rcValue + rsi + 0.04), 12);
    }
    expect(imported.zones.flatMap((zone) => zone.surfaces).some((surface) => surface.type === 'roof')).toBe(true);
  });
});
