/**
 * Issue #14: export of the geometry to the public Uniec 3 Excel import
 * (specification 18-09-2026, dwelling example 28-05-2026). The project below
 * is the dwelling of that example: one zone of 96 m², four facades with
 * R_c 6, windows and a door on the south, windows with roller shutters on the
 * west, a floor on the ground (perimeter 20 m) and a flat roof.
 */
import { writeFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { strFromU8, unzipSync } from 'fflate';
import { createDefaultProject } from '../context/EnergyContext';
import type { IProject, ISurface } from '../core/energy/types';
import {
  GENERAL_COLUMNS, GEOMETRY_COLUMNS, GEOMETRY_SHEET, GENERAL_SHEET, buildUniecGeometryExport, contractText, uniecGeometryWorkbook,
} from '../core/io/UniecGeometryExport';
import { columnName } from '../core/io/xlsxWriter';

const surface = (id: string, name: string, type: ISurface['type'], orientation: ISurface['orientation'], area: number,
  constructionId: string, thermalBoundary: ISurface['thermalBoundary'], windows: ISurface['windows'] = []): ISurface =>
  ({ id, name, type, orientation, area, constructionId, zoneId: 'z1', thermalBoundary, windows });

function exampleDwelling(): IProject {
  const base = createDefaultProject();
  return {
    ...base,
    name: 'VB import woning',
    city: 'Rotterdam',
    buildingFunction: 'residential',
    registration: { purpose: 'delivery', constructionYear: 2027 },
    constructions: [
      { id: 'c-wall', name: 'metselwerk', layers: [], rcValue: 6, uValue: 0.16 },
      { id: 'c-floor', name: 'begane grondvloer', layers: [], rcValue: 6, uValue: 0.16 },
      { id: 'c-roof', name: 'plat dak', layers: [], rcValue: 6, uValue: 0.16 },
    ],
    windowTypes: [
      { id: 't-raam-z', name: 'raam', kind: 'window', uValue: 1.4, gValue: 0.6, unitArea: 6 },
      { id: 't-deur', name: 'deur', kind: 'door', uValue: 2, gValue: 0, unitArea: 2.5 },
      { id: 't-raam-w', name: 'raam west', kind: 'window', uValue: 2.4, gValue: 0.8, unitArea: 6 },
    ],
    zones: [{
      id: 'z1', name: 'RZ 1', floorArea: 96, volume: 260, height: 2.7, thermalBridges: [], airTightness: { qv10: 0.4 },
      surfaces: [
        surface('s-n', 'gevel noord', 'wall', 'N', 43.2, 'c-wall', 'outdoor'),
        surface('s-o', 'gevel oost', 'wall', 'E', 32.4, 'c-wall', 'outdoor'),
        surface('s-z', 'gevel zuid', 'wall', 'S', 43.2, 'c-wall', 'outdoor', [
          { id: 'w-z', name: 'raam', area: 24, uValue: 1.4, gValue: 0.6, orientation: 'S', surfaceId: 's-z', typeId: 't-raam-z', count: 4 },
          { id: 'w-d', name: 'deur', area: 2.5, uValue: 2, gValue: 0, orientation: 'S', surfaceId: 's-z', typeId: 't-deur', count: 1 },
        ]),
        surface('s-w', 'gevel west', 'wall', 'W', 32.4, 'c-wall', 'outdoor', [
          { id: 'w-w', name: 'raam west', area: 12, uValue: 2.4, gValue: 0.8, orientation: 'W', surfaceId: 's-w', typeId: 't-raam-w', count: 2 },
        ]),
        surface('s-v', 'vloer', 'floor', 'horizontal', 38, 'c-floor', 'ground'),
        surface('s-d', 'dak', 'roof', 'horizontal', 48, 'c-roof', 'outdoor'),
      ],
    }],
    ntaCalculation: {
      thermalMass: { floor: 'heavy', wall: 'heavy', ceiling: 'open_or_none', sourceReference: '' },
      surfaceTilts: [],
      windowSolar: { frameFraction: null, obstruction: { method: 'minimal' }, sourceReference: '' },
      windowShadings: [{ windowId: 'w-w', movableShading: { device: { kind: 'external_roller_shutter', colour: 'white' }, control: 'automatic', sourceReference: '' }, sourceReference: '' }],
      groundFloors: [{ surfaceId: 's-v', exposedPerimeterM: 20, constructionResistanceM2kPerW: 6, edgeThermalBridges: null, sourceReference: '' }],
    } as unknown as IProject['ntaCalculation'],
  };
}

const byName = (rows: ReturnType<typeof buildUniecGeometryExport>['geometry'], name: string, type = 'dicht') =>
  rows.filter((row) => row.NaamConstructie === name && row.TypeConstructie === type);

describe('Uniec geometry export of the example dwelling', () => {
  const exported = buildUniecGeometryExport(exampleDwelling());

  it('is not refused and writes the general data of the example', () => {
    expect(exported.points.filter((point) => point.kind === 'missing')).toEqual([]);
    expect(exported.ok).toBe(true);
    expect(exported.general).toMatchObject({
      Omschrijving: 'VB import woning', Plaats: 'Rotterdam', TypeGebouw: 'grondgebonden woning', SoortBouw: 'nieuwbouw',
      Bouwjaar: 2027, Opname: 'detailopname', NaamExterneTool: 'Open Energy Studio',
    });
  });

  it('writes the opaque parts net of the windows, with orientation and tilt', () => {
    expect(byName(exported.geometry, 'gevel noord')[0]).toMatchObject({
      TypeZone: 'rekenzone', NaamZone: 'RZ 1', Ag: 96, Vlak: 'gevel', Begrenzing: 'buitenlucht', Orientatie: 'N', Helling: 90,
      NaamBibliotheek: 'metselwerk', Methodiek: 'vrije invoer', Rc: 6, Oppervlakte: 43.2,
    });
    // 43,2 m² gross less 4 × 6 m² of window and a door of 2,5 m²: 16,7 m², as in the example.
    expect(byName(exported.geometry, 'gevel zuid')[0]).toMatchObject({ Orientatie: 'Z', Oppervlakte: 16.7 });
    expect(byName(exported.geometry, 'gevel west')[0]).toMatchObject({ Orientatie: 'W', Oppervlakte: 20.4 });
    expect(byName(exported.geometry, 'dak')[0]).toMatchObject({ Vlak: 'dak', Orientatie: 'HOR', NaamBibliotheek: 'plat dak', Oppervlakte: 48 });
    expect(byName(exported.geometry, 'dak')[0].Helling).toBeUndefined();
  });

  it('writes the floor on the ground with its perimeter', () => {
    expect(byName(exported.geometry, 'vloer')[0]).toMatchObject({
      Vlak: 'vloer', Begrenzing: 'grond', Maaiveld: 'opBovenMv', Perimeter: 20, Oppervlakte: 38, Rc: 6,
    });
  });

  it('writes windows and doors with U, g, shading and ventilative cooling', () => {
    const south = byName(exported.geometry, 'gevel zuid', 'transparant');
    // Area per unit and count, as in the example, since every window names its type and count.
    expect(south.map((row) => [row.TypeTransparant, row['Uw/Ud'], row['ggl;n'], row.A, row.AantalTransparanteConstructies, row.Oppervlakte]))
      .toEqual([['raam', 1.4, 0.6, 6, 4, undefined], ['deur', 2, 0, 2.5, 1, undefined]]);
    // The windows come before the opaque part of their facade.
    const order = exported.geometry.filter((row) => row.NaamConstructie === 'gevel zuid').map((row) => row.TypeConstructie);
    expect(order).toEqual(['transparant', 'transparant', 'dicht']);
    expect(south[0]).toMatchObject({ Beschaduwing: 'minimale belemmering', Zonwering: 'geen zonwering', VentilatieveKoeling: 'niet aanwezig' });
    // A door with g = 0 needs no shading (F126, F150).
    expect(south[1].Beschaduwing).toBeUndefined();
    expect(south[1].Zonwering).toBeUndefined();
    const west = byName(exported.geometry, 'gevel west', 'transparant')[0];
    expect(west).toMatchObject({ 'Uw/Ud': 2.4, 'ggl;n': 0.8, Zonwering: 'rolluiken (buiten), wit', Regeling: 'automatisch overig' });
  });

  it('names the derived values', () => {
    const where = exported.points.filter((point) => point.kind === 'derived').map((point) => point.where);
    expect(where).toEqual(expect.arrayContaining(['Plaats', 'BouwwijzeVloeren', 'BouwwijzePlafonds', 'vlak vloer']));
  });
});

describe('Uniec geometry export refuses what the contract needs and the project lacks', () => {
  it('without city, construction year, thermal mass and perimeter', () => {
    const project = exampleDwelling();
    project.city = '';
    project.registration = { purpose: 'delivery' };
    const nta = project.ntaCalculation as unknown as Record<string, unknown>;
    nta.thermalMass = { floor: null, wall: null, ceiling: 'open_or_none', sourceReference: '' };
    nta.groundFloors = [{ surfaceId: 's-v', exposedPerimeterM: null }];
    const exported = buildUniecGeometryExport(project);
    expect(exported.ok).toBe(false);
    expect(exported.points.filter((point) => point.kind === 'missing').map((point) => point.where))
      .toEqual(['Plaats', 'Bouwjaar', 'BouwwijzeVloeren', 'BouwwijzeWanden', 'vlak vloer']);
  });

  it('for a utility building, a crawl space and an obstruction with distances', () => {
    const project = exampleDwelling();
    project.buildingFunction = 'office';
    const nta = project.ntaCalculation as unknown as Record<string, unknown>;
    nta.groundFloors = [{ surfaceId: 's-v', exposedPerimeterM: 20, below: { kind: 'crawlspace' } }];
    nta.windowObstructions = [{ windowId: 'w-z', obstruction: { method: 'overhang', relativeHeight: 0.5 }, sourceReference: '' }];
    const missing = buildUniecGeometryExport(project).points.filter((point) => point.kind === 'missing').map((point) => point.where);
    expect(missing).toEqual(['TypeGebouw', 'raam raam op gevel zuid', 'vlak vloer']);
  });

  it('lists thermal bridges as not exported', () => {
    const project = exampleDwelling();
    project.zones[0].thermalBridges = [{ id: 'tb', name: 'vloerrand', psiValue: 0.8, length: 8, zoneId: 'z1' }];
    const exported = buildUniecGeometryExport(project);
    expect(exported.ok).toBe(true);
    expect(exported.points.some((point) => point.kind === 'lost' && point.message.includes('Koudebruggen'))).toBe(true);
  });
});

describe('contract rules', () => {
  it('keeps texts to the allowed characters and 32 characters', () => {
    expect(contractText('Gevel (noord) — 1e verdieping', 'x')).toBe('Gevel noord 1e verdieping');
    expect(contractText('Café Ærø', 'x')).toBe('Cafe r');
    expect(contractText('***', 'Vlak')).toBe('Vlak');
    expect(contractText('a'.repeat(40), 'x')).toHaveLength(32);
  });

  it('rounds g down to a multiple of 0,05 and U to the contract rule', () => {
    const project = exampleDwelling();
    project.zones[0].surfaces[2].windows[0] = { ...project.zones[0].surfaces[2].windows[0], typeId: undefined, uValue: 1.234, gValue: 0.63 };
    const row = buildUniecGeometryExport(project).geometry.find((item) => item.NaamConstructie === 'gevel zuid' && item.TypeTransparant === 'raam')!;
    expect(row['Uw/Ud']).toBe(1.2);
    expect(row['ggl;n']).toBe(0.6);
  });

  it('gives a library name one Vlak (F214)', () => {
    const project = exampleDwelling();
    // The roof uses the facade construction.
    project.zones[0].surfaces[5].constructionId = 'c-wall';
    const rows = buildUniecGeometryExport(project).geometry.filter((row) => row.TypeConstructie === 'dicht');
    const vlakByName = new Map<string, Set<string>>();
    for (const row of rows) vlakByName.set(String(row.NaamBibliotheek), new Set([...(vlakByName.get(String(row.NaamBibliotheek)) ?? []), String(row.Vlak)]));
    expect([...vlakByName.values()].every((vlakken) => vlakken.size === 1)).toBe(true);
    expect(rows.find((row) => row.Vlak === 'dak')?.NaamBibliotheek).toBe('metselwerk 2');
  });
});

describe('the workbook', () => {
  it('has the two sheets with the contract headers and the rows', () => {
    const exported = buildUniecGeometryExport(exampleDwelling());
    const files = unzipSync(uniecGeometryWorkbook(exported));
    const workbook = strFromU8(files['xl/workbook.xml']);
    expect(workbook).toContain(`name="${GENERAL_SHEET}"`);
    expect(workbook).toContain(`name="${GEOMETRY_SHEET}"`);
    const general = strFromU8(files['xl/worksheets/sheet1.xml']);
    expect(general).toContain('<c r="A1" t="inlineStr"><is><t xml:space="preserve">Omschrijving</t></is></c>');
    expect(general).toContain('<c r="E2"><v>2027</v></c>');
    const geometry = strFromU8(files['xl/worksheets/sheet2.xml']);
    const lastHeader = `${columnName(GEOMETRY_COLUMNS.length - 1)}1`;
    expect(geometry).toContain(`<c r="${lastHeader}" t="inlineStr"><is><t xml:space="preserve">Opmerking</t></is></c>`);
    expect(geometry.match(/<row /g)).toHaveLength(exported.geometry.length + 1);
    expect(GENERAL_COLUMNS).toHaveLength(9);
    // For the comparison with the official example (docs/nta8800-uniec-geometrie-export.md).
    if (process.env.OES_WRITE_UNIEC_GEOMETRY) writeFileSync(process.env.OES_WRITE_UNIEC_GEOMETRY, uniecGeometryWorkbook(exported));
  });

  it('names columns like a spreadsheet', () => {
    expect([0, 25, 26, 51, 52, 701, 702].map(columnName)).toEqual(['A', 'Z', 'AA', 'AZ', 'BA', 'ZZ', 'AAA']);
  });
});
