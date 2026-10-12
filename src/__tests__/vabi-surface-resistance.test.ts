// @vitest-environment node
import { describe, expect, it } from 'vitest';
// @ts-expect-error jsdom ships without type declarations here; only DOMParser is used.
import { JSDOM } from 'jsdom';
import { strFromU8, strToU8, unzipSync, zipSync } from 'fflate';
import { createDefaultProject } from '../context/EnergyContext';
import { exportToVABI, importFromVABI } from '../core/io/VABIElementsBridge';

// fflate needs the Node realm; the importer only needs a DOMParser.
globalThis.DOMParser = new JSDOM().window.DOMParser;

function withoutUValues(zip: Uint8Array): Uint8Array {
  const xml = strFromU8(unzipSync(zip)['project.xml']).replace(/\s*<UWaarde>[^<]*<\/UWaarde>/g, '');
  return zipSync({ 'project.xml': new Uint8Array(strToU8(xml)) });
}

describe('VABI import (NTA 8800 table C.2)', () => {
  it('keeps the U of the project on a round trip', () => {
    const project = createDefaultProject();
    const imported = importFromVABI(exportToVABI(project));
    const byName = new Map(project.constructions.map((item) => [item.name, item.uValue]));
    for (const surface of imported.zones.flatMap((zone) => zone.surfaces)) {
      const construction = imported.constructions.find((item) => item.id === surface.constructionId);
      if (!construction || !byName.has(construction.name)) continue;
      expect(construction.uValue).toBe(byName.get(construction.name));
    }
  });

  it('derives a missing U from Rc with the R_si of the element type', () => {
    const project = createDefaultProject();
    const imported = importFromVABI(withoutUValues(exportToVABI(project)));
    let checked = 0;
    for (const surface of imported.zones.flatMap((zone) => zone.surfaces)) {
      const construction = imported.constructions.find((item) => item.id === surface.constructionId);
      if (!construction || construction.rcValue <= 0) continue;
      const rsi = { wall: 0.13, roof: 0.10, floor: 0.17, internal: 0.13 }[surface.type];
      expect(construction.uValue).toBeCloseTo(1 / (construction.rcValue + rsi + 0.04), 12);
      checked += 1;
    }
    expect(checked).toBeGreaterThan(0);
    expect(imported.zones.flatMap((zone) => zone.surfaces).some((surface) => surface.type === 'roof')).toBe(true);
  });
});
