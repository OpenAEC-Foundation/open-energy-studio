/**
 * Import of a real Uniec3 export (format of Uniec 3.4, analysed 8 Oct 2026).
 * The fixture is the anonymised export of a one-zone dwelling: 9 boundary
 * surfaces with 10 window rows from a library of 3 opaque and 9 window
 * types, and 6 installations. Nothing is defaulted in silence: what the
 * model cannot hold is an import issue with its kind and Uniec3 field.
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { strToU8, zipSync } from 'fflate';
import { importFromUNIEC3, readUNIEC3 } from '../core/io/UNIEC3Exporter';
import { isUniec3Export } from '../core/io/UNIEC3Import';

const fixture = JSON.parse(readFileSync(resolve(__dirname, '../../training-data/uniec3-voorbeeldwoning.json'), 'utf8')) as {
  meta: unknown; buildings: unknown; entities: unknown; relations: unknown;
};
const archive = () => zipSync({
  'meta.json': strToU8(JSON.stringify(fixture.meta)),
  'buildings.json': strToU8(JSON.stringify(fixture.buildings)),
  'buildings/1/entities.json': strToU8(JSON.stringify(fixture.entities)),
  'buildings/1/relations.json': strToU8(JSON.stringify(fixture.relations)),
});

describe('Uniec3 export import', () => {
  const read = readUNIEC3(archive());
  const project = read.project;
  const surfaces = project.zones.flatMap((zone) => zone.surfaces);
  const byName = (name: string) => surfaces.find((surface) => surface.name === name)!;
  const nta = project.ntaCalculation as unknown as Record<string, Array<Record<string, unknown>>>;
  const notes = read.issues.list.map((issue) => issue.message);

  it('recognises the real format and reads the building', () => {
    expect(isUniec3Export({ 'meta.json': strToU8('input draft - unverified'), 'buildings/1/relations.json': strToU8('[]') })).toBe(false);
    expect(project.name).toBe('Voorbeeldwoning (Uniec3)');
    expect(project.buildingFunction).toBe('residential');
    expect(project.registration).toMatchObject({ purpose: 'delivery', constructionYear: 2023 });
    expect(importFromUNIEC3(archive()).name).toBe(project.name);
  });

  it('reads the zone with its usable floor area and the measured qv10', () => {
    expect(project.zones).toHaveLength(1);
    const zone = project.zones[0];
    expect(zone.name).toBe('Woning');
    expect(zone.floorArea).toBe(35.74);
    expect(zone.airTightness).toEqual({ qv10: 0.98 });
    // Height from the building height and one storey; volume derived and noted.
    expect(zone.height).toBe(3.91);
    expect(zone.volume).toBeCloseTo(35.74 * 3.91, 2);
    expect(notes.some((line) => line.includes('hoogte 3.91 m afgeleid'))).toBe(true);
  });

  it('reads the library types and the nine surfaces with their constructions', () => {
    expect(project.constructions.map((item) => [item.name, item.rcValue])).toEqual([['Gevel', 4.73], ['Dak', 6.3], ['Vloer', 5.33]]);
    // U = 1/(R_si 0,13 + R_c + R_se 0,04) for a facade (NTA 8800 table C.2).
    expect(project.constructions[0].uValue).toBeCloseTo(1 / 4.9, 4);
    expect(surfaces).toHaveLength(9);
    expect(byName('Gevel Noord')).toMatchObject({ type: 'wall', orientation: 'N', thermalBoundary: 'outdoor', area: 10.28, constructionId: 'con-1' });
    expect(byName('Dak Oost Lang')).toMatchObject({ type: 'roof', orientation: 'E', area: 13.14, constructionId: 'con-2' });
    expect(byName('Vloer')).toMatchObject({ type: 'floor', orientation: 'horizontal', thermalBoundary: 'ground', area: 35.74, constructionId: 'con-3' });
    // Every surface satisfies gross = opaque + windows, so no area note.
    expect(notes.some((line) => line.includes('bruto'))).toBe(false);
  });

  it('reads the window rows from the window types, with count, doors and shading', () => {
    const windows = surfaces.flatMap((surface) => surface.windows);
    expect(windows).toHaveLength(10);
    const west = byName('Gevel West').windows;
    expect(west.map((window) => [window.name, window.area, window.uValue, window.gValue])).toEqual([
      ['A', 1.8, 1, 0.4], ['B', 4.69, 0.88, 0.4], ['I', 2.45, 1.5, 0],
    ]);
    const rooflights = byName('Dak Oost Lang').windows;
    expect(rooflights).toHaveLength(1);
    expect(rooflights[0]).toMatchObject({ name: 'Dakraam', count: 2, area: 2.4, uValue: 0.7, gValue: 0.5, orientation: 'E' });
    // The library rows are window types; the door is a door type.
    expect(project.windowTypes).toHaveLength(9);
    expect(project.windowTypes!.find((type) => type.id === rooflights[0].typeId)).toMatchObject({ name: 'Dakraam', kind: 'window', unitArea: 1.2 });
    expect(project.windowTypes!.find((type) => type.id === west[2].typeId)).toMatchObject({ name: 'I', kind: 'door', uValue: 1.5, gValue: 0 });
    // Obstruction and movable shading per window go to the NTA block; the door has neither.
    expect(nta.windowObstructions).toHaveLength(9);
    expect(nta.windowObstructions.find((item) => item.windowId === west[0].id)?.obstruction).toEqual({ method: 'minimal' });
    expect(nta.windowObstructions.find((item) => item.windowId === byName('Gevel Zuid').windows[0].id)?.obstruction).toEqual({ method: 'full' });
    expect(nta.windowShadings).toHaveLength(9);
    expect(nta.windowShadings.find((item) => item.windowId === rooflights[0].id)?.movableShading)
      .toMatchObject({ device: { kind: 'external_screen', colour: 'white' }, control: 'automatic' });
  });

  it('answers the NTA input the file holds: roof tilts and the floor perimeter', () => {
    const roofs = surfaces.filter((surface) => surface.type === 'roof');
    expect(roofs).toHaveLength(4);
    for (const roof of roofs) expect(nta.surfaceTilts.find((item) => item.surfaceId === roof.id)).toMatchObject({ tiltDeg: 22 });
    expect(nta.groundFloors).toHaveLength(1);
    expect(nta.groundFloors[0]).toMatchObject({ surfaceId: byName('Vloer').id, exposedPerimeterM: 28.8 });
  });

  it('reads the installations with their efficiencies and notes what the model cannot hold', () => {
    expect(project.heatingSystems).toHaveLength(1);
    expect(project.heatingSystems[0]).toMatchObject({ type: 'heat_pump_air', cop: 2.8, coverageFraction: 1 });
    expect(project.hotWaterSystems[0]).toMatchObject({ type: 'electric_boiler', efficiency: 2.3 });
    expect(project.ventilationSystems[0]).toMatchObject({ type: 'type_d', heatRecoveryEfficiency: 0.9 });
    expect(project.coolingSystems[0]).toMatchObject({ type: 'split_unit', eer: 3 });
    expect(project.solarPV.map((pv) => [pv.orientation, pv.area, pv.peakPower, pv.tilt])).toEqual([['W', 5, 2.15, 23], ['E', 8, 3.44, 23]]);
    expect(notes.some((line) => line.includes('voorraadvat 150 l'))).toBe(true);
    expect(notes.some((line) => line.includes('ventilatorvermogen 113.3 W'))).toBe(true);
    expect(notes.some((line) => line.includes('productlijst'))).toBe(true);
  });

  // Issue #12: the points are structured, so the toast and the import record show them too.
  it('gives every point a kind and the Uniec3 field', () => {
    const issues = read.issues.list;
    expect(issues.length).toBeGreaterThan(0);
    expect(issues.find((issue) => issue.message.includes('voorraadvat 150 l'))).toMatchObject({ kind: 'dropped', path: 'TAPW-VAT' });
    expect(issues.find((issue) => issue.message.includes('hoogte 3.91 m afgeleid'))).toMatchObject({ kind: 'defaulted', path: 'INFIL_BGH' });
    // The importer leaves the one import record to the caller.
    expect(project.importLog).toBeUndefined();
  });
});

// Issue #11: an archive with two buildings reads the first and says so.
describe('Uniec3 export with more than one building', () => {
  const twoBuildings = () => zipSync({
    'meta.json': strToU8(JSON.stringify(fixture.meta)),
    'buildings/10/entities.json': strToU8(JSON.stringify(fixture.entities)),
    'buildings/10/relations.json': strToU8(JSON.stringify(fixture.relations)),
    'buildings/2/entities.json': strToU8(JSON.stringify(fixture.entities)),
    'buildings/2/relations.json': strToU8(JSON.stringify(fixture.relations)),
  });

  it('reads the lowest building id and reports the other as multiple', () => {
    const { project, issues } = readUNIEC3(twoBuildings());
    expect(project.zones).toHaveLength(1);
    const multiple = issues.list.filter((issue) => issue.kind === 'multiple' && issue.path === 'buildings');
    expect(multiple).toHaveLength(1);
    expect(multiple[0]).toMatchObject({ value: 2 });
    expect(multiple[0].message).toContain('2 gebouwen (2, 10)');
    expect(multiple[0].message).toContain('alleen gebouw 2 is ingelezen');
  });

  // Re-check 26 Oct 2026: a second building folder without relations.json was dropped in silence.
  it('reports an incomplete second building by its path', () => {
    const { project, issues } = readUNIEC3(zipSync({
      'meta.json': strToU8(JSON.stringify(fixture.meta)),
      'buildings/1/entities.json': strToU8(JSON.stringify(fixture.entities)),
      'buildings/1/relations.json': strToU8(JSON.stringify(fixture.relations)),
      'buildings/2/entities.json': strToU8(JSON.stringify(fixture.entities)),
    }));
    expect(project.zones).toHaveLength(1);
    expect(issues.list.find((issue) => issue.path === 'buildings/2')).toMatchObject({ kind: 'dropped' });
    expect(issues.list.find((issue) => issue.path === 'buildings/2')?.message).toContain('relations.json ontbreekt');
    const multiple = issues.list.find((issue) => issue.kind === 'multiple' && issue.path === 'buildings');
    expect(multiple?.message).toContain('2 gebouwen (1, 2)');
    expect(multiple?.message).toContain('alleen gebouw 1 is ingelezen');
  });

  it('reads the complete building when the first is incomplete', () => {
    const { project, issues } = readUNIEC3(zipSync({
      'meta.json': strToU8(JSON.stringify(fixture.meta)),
      'buildings/1/relations.json': strToU8(JSON.stringify(fixture.relations)),
      'buildings/3/entities.json': strToU8(JSON.stringify(fixture.entities)),
      'buildings/3/relations.json': strToU8(JSON.stringify(fixture.relations)),
    }));
    expect(project.zones).toHaveLength(1);
    expect(issues.list.find((issue) => issue.path === 'buildings/1')?.message).toContain('entities.json ontbreekt');
    expect(issues.list.find((issue) => issue.kind === 'multiple')?.message).toContain('alleen gebouw 3 is ingelezen');
  });

  it('reports nothing about buildings for a single building', () => {
    expect(readUNIEC3(archive()).issues.list.some((issue) => issue.path === 'buildings')).toBe(false);
  });
});
