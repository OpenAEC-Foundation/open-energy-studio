// @vitest-environment node
import { describe, expect, it } from 'vitest';
// @ts-expect-error jsdom ships without type declarations here; only DOMParser is used.
import { JSDOM } from 'jsdom';
import { readdirSync, readFileSync } from 'node:fs';
import { strFromU8, strToU8, unzipSync, zipSync } from 'fflate';
import type { IProject } from '../core/energy/types';
import { normalizeProject } from '../core/energy/normalizeProject';
import { exportToUNIEC3, readUNIEC3, uniec3ExportLosses } from '../core/io/UNIEC3Exporter';
import { exportToVABI, readVABI, readVABIFile, readVABIXml, vabiExportLosses } from '../core/io/VABIElementsBridge';
import { droppedOnExport } from '../core/io/importIssues';
import { importFile } from '../core/io/importFile';
import { withImportRecord } from '../core/io/importLog';
import { nl } from '../i18n/nl';
import { en } from '../i18n/en';

// fflate needs the Node realm; the importer only needs a DOMParser.
globalThis.DOMParser = new JSDOM().window.DOMParser;

// Review 39: fidelity of the UNIEC3 and VABI import and export.

/** Every training file that holds a project model. */
const PROJECTS = readdirSync('training-data')
  .filter((file) => file.endsWith('.json'))
  .flatMap((file) => {
    try {
      const raw = JSON.parse(readFileSync(`training-data/${file}`, 'utf8'));
      const project = raw?.project ?? raw;
      return Array.isArray(project?.zones) && Array.isArray(project?.heatingSystems) ? [{ file, project: normalizeProject(project) }] : [];
    } catch {
      return [];
    }
  });

const load = (file: string): IProject => PROJECTS.find((entry) => entry.file === file)!.project;

/** Bookkeeping fields that are regenerated on import; references are compared through what they point at. */
const REGENERATED = new Set(['id', 'zoneId', 'surfaceId', 'constructionId']);

/**
 * The project as `path → value`, with each surface's construction inlined
 * (`zones[0].surfaces[1].construction.uValue`) so a lost link shows up.
 */
function flatten(project: IProject): Map<string, unknown> {
  const out = new Map<string, unknown>();
  const walk = (value: unknown, path: string) => {
    if (Array.isArray(value)) {
      // An empty list holds nothing, like an absent one.
      value.forEach((item, index) => walk(item, `${path}[${index}]`));
      return;
    }
    if (value !== null && typeof value === 'object') {
      for (const [key, item] of Object.entries(value)) {
        if (REGENERATED.has(key) || item === undefined) continue;
        walk(item, path ? `${path}.${key}` : key);
      }
      return;
    }
    out.set(path, value);
  };
  const { constructions, ...rest } = project;
  walk(rest, '');
  project.zones.forEach((zone, zoneIndex) => zone.surfaces.forEach((surface, surfaceIndex) => {
    const construction = constructions.find((item) => item.id === surface.constructionId);
    if (construction) {
      walk({ name: construction.name, rcValue: construction.rcValue, uValue: construction.uValue },
        `zones[${zoneIndex}].surfaces[${surfaceIndex}].construction`);
    }
  }));
  return out;
}

const pattern = (path: string) => path.replace(/\[\d+\]/g, '[*]');
const close = (a: unknown, b: unknown) => a === b
  || (typeof a === 'number' && typeof b === 'number' && Math.abs(a - b) <= 1e-9 * Math.max(1, Math.abs(a)));

/**
 * Paths whose value changed on export → import that neither the export
 * losses nor the import issues name. A path the import added is allowed
 * only inside the NTA block (VABI tilts start one).
 */
function unexplained(project: IProject, losses: string[], back: IProject, issuePaths: string[]): string[] {
  const before = flatten(project);
  const after = flatten(back);
  const explained = (path: string) => losses.some((loss) => pattern(path) === loss || pattern(path).startsWith(`${loss}.`)
    || pattern(path).startsWith(`${loss}[`))
    || issuePaths.some((issue) => path === issue || path.startsWith(`${issue}.`) || path.startsWith(`${issue}[`));
  const paths = new Set([...before.keys(), ...after.keys()]);
  return [...paths].filter((path) => {
    if (close(before.get(path), after.get(path))) return false;
    if (!before.has(path) && path.startsWith('ntaCalculation')) return false;
    return !explained(path);
  }).sort();
}

describe('round trip of every training project (review 39)', () => {
  it('found the training projects', () => {
    expect(PROJECTS.length).toBeGreaterThan(20);
  });

  for (const format of ['UNIEC3', 'VABI'] as const) {
    it(`changes nothing on a ${format} export and import that the losses or issues do not name`, () => {
      const problems: string[] = [];
      for (const { file, project } of PROJECTS) {
        const losses = format === 'UNIEC3' ? uniec3ExportLosses(project) : vabiExportLosses(project);
        const { project: read, issues } = format === 'UNIEC3' ? readUNIEC3(exportToUNIEC3(project)) : readVABI(exportToVABI(project));
        const back = normalizeProject(read);
        for (const path of unexplained(project, losses, back, issues.list.map((issue) => issue.path))) {
          problems.push(`${file}: ${path}`);
        }
      }
      expect(problems).toEqual([]);
    });
  }
});

describe('export losses (review 39)', () => {
  it('names installation fields neither format holds', () => {
    const project = load('2467-goejanverwelledijk-gouda.oes.json');
    expect(project.solarPV.some((pv) => 'comment' in pv)).toBe(true);
    for (const format of ['UNIEC3', 'VABI'] as const) expect(droppedOnExport(project, format)).toContain('solarPV[*].comment');
    const heatPump = { ...project, heatingSystems: [{ ...project.heatingSystems[0], ntaHeatPump: { source: 'outdoor_air' } as never }] };
    expect(droppedOnExport(heatPump, 'VABI')).toContain('heatingSystems[*].ntaHeatPump');
  });

  it('names a construction no surface uses for UNIEC3 only', () => {
    const project = load('2522-woning-aalten.oes.json');
    const spare = { ...project, constructions: [...project.constructions, { id: 'spare', name: 'Reserve', layers: [], rcValue: 3, uValue: 0.3 }] };
    expect(droppedOnExport(spare, 'UNIEC3')).toContain('constructions[unused]');
    expect(droppedOnExport(project, 'UNIEC3')).not.toContain('constructions[unused]');
    expect(droppedOnExport(spare, 'VABI')).not.toContain('constructions[unused]');
    expect(readVABI(exportToVABI(spare)).project.constructions.map((item) => item.name)).toContain('Reserve');
  });

  it('names a VABI window that faces another way than its surface', () => {
    const project = load('2522-woning-aalten.oes.json');
    const zone = project.zones[0];
    const surfaceIndex = zone.surfaces.findIndex((surface) => surface.windows.length > 0);
    const surface = zone.surfaces[surfaceIndex];
    const turned = surface.orientation === 'N' ? 'E' : 'N';
    const changed: IProject = { ...project, zones: [{ ...zone, surfaces: zone.surfaces.map((item, index) => (index === surfaceIndex
      ? { ...item, windows: item.windows.map((window, windowIndex) => (windowIndex === 0 ? { ...window, orientation: turned } : window)) }
      : item)) }, ...project.zones.slice(1)] };
    expect(droppedOnExport(project, 'VABI')).not.toContain('zones[*].surfaces[*].windows[*].orientation');
    expect(droppedOnExport(changed, 'VABI')).toContain('zones[*].surfaces[*].windows[*].orientation');
    expect(droppedOnExport(changed, 'UNIEC3')).not.toContain('zones[*].surfaces[*].windows[*].orientation');
  });
});

describe('VABI export of a surface filled by windows (review 39)', () => {
  it('keeps the construction of a fully glazed surface', () => {
    const project = load('nta8800-public-comparison-f.json');
    const glazed = project.zones[0].surfaces.filter((surface) => surface.windows.reduce((sum, w) => sum + w.area, 0) >= surface.area);
    expect(glazed.length).toBeGreaterThan(0);
    const back = readVABI(exportToVABI(project)).project;
    for (const surface of glazed) {
      const index = project.zones[0].surfaces.indexOf(surface);
      const own = project.constructions.find((item) => item.id === surface.constructionId)!;
      const read = back.constructions.find((item) => item.id === back.zones[0].surfaces[index].constructionId);
      expect(read?.name).toBe(own.name);
      expect(read?.uValue).toBe(own.uValue);
    }
  });
});

type Entity = { NTAEntityDataId: string; NTAEntityId: string; NTAPropertyDatas: { NTAPropertyId: string; Value: string }[] };
const ent = (id: string, type: string, props: Record<string, string> = {}): Entity => ({
  NTAEntityDataId: id, NTAEntityId: type,
  NTAPropertyDatas: Object.entries(props).map(([NTAPropertyId, Value]) => ({ NTAPropertyId, Value })),
});
const rel = (id: string, kind: string, parent: string, child: string) =>
  ({ NTAEntityDataId: id, NTAEntityRelationId: kind, ParentNTAEntityDataId: parent, ChildNTAEntityDataId: child });

describe('data the importer does not read (review 39)', () => {
  it('lists UNIEC3 entities and properties it passes over', () => {
    const zip = zipSync({
      'buildings/1/entities.json': strToU8(JSON.stringify([
        ent('g', 'GEB', { GEB_NAAM: 'A', GEB_BOUWJAAR: '1975' }),
        ent('u', 'UNIT'), ent('r', 'RZ', { RZ_AG: '100' }),
        ent('b', 'BEGR', { BEGR_OPP: '20', BEGR_TYPE: 'GVL_BEGR_GEVEL', BEGR_ORIENT: 'GVL_BTNL_Z', BEGR_HELLING: '90' }),
        ent('c1', 'CONSTRD', { CONSTRD_U: '0.3' }), ent('c2', 'CONSTRD', { CONSTRD_U: '0.2' }),
        ent('loose', 'BEGR', { BEGR_OPP: '5' }),
        ent('v1', 'VERL'), ent('v2', 'VERL'),
        ent('s', 'SETTINGS'), ent('lib', 'LIBCONSTRD', { CONSTRD_U: '0.3' }),
      ])),
      'buildings/1/relations.json': strToU8(JSON.stringify([
        rel('1', 'UNIT_RZ', 'u', 'r'), rel('2', 'RZ_BEGR', 'r', 'b'),
        rel('3', 'BEGR_CONSTRD', 'b', 'c1'), rel('4', 'BEGR_CONSTRD', 'b', 'c2'),
      ])),
    });
    const { issues } = readUNIEC3(zip);
    const entities = issues.list.find((issue) => issue.path === 'entities' && issue.kind === 'dropped');
    // The second construction of the surface, the loose surface and the two lighting entities;
    // the empty settings entity and the library copy hold nothing of their own.
    expect(entities?.message).toBe('Entiteiten niet ingelezen: BEGR, CONSTRD, VERL (2×).');
    expect(entities?.value).toBe(4);
    expect(issues.list.find((issue) => issue.path === 'GEB')?.message).toBe('Eigenschappen van GEB niet ingelezen: GEB_BOUWJAAR.');
    expect(issues.list.find((issue) => issue.path === 'BEGR')?.message).toBe('Eigenschappen van BEGR niet ingelezen: BEGR_HELLING.');
  });

  it('lists VABI elements it passes over', () => {
    const xml = `<?xml version="1.0" encoding="utf-8"?><Project><Algemeen><Versie>4.1</Versie><Applicatie>Vabi Elements</Applicatie></Algemeen>`
      + `<Objecten><Object><Naam>T</Naam><Bouwjaar>1975</Bouwjaar><Rekenzones><Rekenzone><Naam>Z</Naam><Gebruiksoppervlakte>100</Gebruiksoppervlakte>`
      + `<Bezetting>3</Bezetting><Geometrie><Hoofdvlak><Naam>V</Naam><Locatie>2</Locatie><Orientatie>4</Orientatie><Oppervlakte>20</Oppervlakte>`
      + `<Zonwering>1</Zonwering></Hoofdvlak></Geometrie></Rekenzone></Rekenzones>`
      + `<Installaties><Verwarming><Opwekker><Naam>K</Naam><Type>1</Type><COP>1</COP><Dekking>1</Dekking><EER>4</EER></Opwekker></Verwarming>`
      + `<Verlichting><Vermogen>8</Vermogen></Verlichting></Installaties></Object></Objecten></Project>`;
    const { issues } = readVABIXml(xml);
    const message = (path: string) => issues.list.find((issue) => issue.path === path && issue.kind === 'dropped')?.message;
    expect(message('Object')).toBe('Elementen onder Object niet ingelezen: Bouwjaar.');
    expect(message('Rekenzone')).toBe('Elementen onder Rekenzone niet ingelezen: Bezetting.');
    expect(message('Hoofdvlak')).toBe('Elementen onder Hoofdvlak niet ingelezen: Zonwering.');
    expect(message('Verwarming/Opwekker')).toBe('Elementen onder Verwarming/Opwekker niet ingelezen: EER.');
    expect(message('Installaties')).toBe('Elementen onder Installaties niet ingelezen: Verlichting.');
  });

  it('reports nothing unread for its own export', () => {
    for (const { project } of PROJECTS) {
      const lists = [readUNIEC3(exportToUNIEC3(project)).issues.list, readVABI(exportToVABI(project)).issues.list];
      for (const list of lists) {
        expect(list.filter((issue) => issue.message.includes('niet ingelezen'))).toEqual([]);
      }
    }
  });
});

describe('source version and edition (review 39)', () => {
  it('records the version a UNIEC3 file names and does not carry its edition over', async () => {
    const project = load('2522-woning-aalten.oes.json');
    const files = unzipSync(exportToUNIEC3(project));
    files['meta.json'] = strToU8(JSON.stringify({ VersionId: 300, NTAVersion: 'NTA8800 v3.1.5.0', Generator: 'Uniec 3' }));
    const result = await importFile(new File([new Uint8Array(zipSync(files))], 'oud.uniec3'), readUNIEC3);
    expect(result.sourceVersion).toBe('NTA8800 v3.1.5.0, VersionId 300, Uniec 3');
    const issue = result.issues.find((item) => item.path === 'normVersion');
    expect(issue?.kind).toBe('dropped');
    expect(issue?.message).toContain('NTA8800 v3.1.5.0');
    expect(issue?.message).toContain('controleer');
    // The imported project has no edition of its own: the NTA input decides it.
    expect(result.project.ntaCalculation).toBeUndefined();
    const recorded = withImportRecord(result.project, 'UNIEC3', result.fileName, '2026-10-11T00:00:00Z',
      { sha256: result.sha256, sourceVersion: result.sourceVersion, issues: result.issues });
    expect(recorded.importLog?.[0].sourceVersion).toBe('NTA8800 v3.1.5.0, VersionId 300, Uniec 3');
  });

  it('flags a file that names no version', () => {
    const files = unzipSync(exportToUNIEC3(load('2522-woning-aalten.oes.json')));
    delete files['meta.json'];
    const { issues, sourceVersion } = readUNIEC3(zipSync(files));
    expect(sourceVersion).toBeUndefined();
    expect(issues.list.find((item) => item.path === 'normVersion')?.message).toMatch(/zonder programma- of NTA 8800-versie/);
  });

  it('records the version a VABI file names', () => {
    const xml = strFromU8(unzipSync(exportToVABI(load('2522-woning-aalten.oes.json')))['project.xml']);
    const { sourceVersion, issues } = readVABIFile(strToU8(xml));
    expect(sourceVersion).toBe('Open Energy Studio, versie 4.0');
    expect(issues.list.some((item) => item.path === 'normVersion' && item.kind === 'dropped')).toBe(true);
  });

  it('labels the source version in the import record in NL and EN', () => {
    expect(nl['import.log.source']).toBe('Bronversie');
    expect(en['import.log.source']).toBe('Source version');
  });
});

// Follow-ups of review 39: a ratio of 0 and the tilt of a UNIEC3 pitched roof.
describe('a COP, EER or efficiency of 0 (review 39 follow-up)', () => {
  const read = (format: 'UNIEC3' | 'VABI', project: IProject) =>
    (format === 'UNIEC3' ? readUNIEC3(exportToUNIEC3(project)) : readVABI(exportToVABI(project)));

  it('keeps an EER of 0 on cooling of type "none" without an issue', () => {
    const project = load('2786-reddingspost-kijkduin.oes.json');
    expect(project.coolingSystems[0]).toMatchObject({ type: 'none', eer: 0 });
    for (const format of ['UNIEC3', 'VABI'] as const) {
      expect(droppedOnExport(project, format)).not.toContain('coolingSystems[*].eer');
      const { project: back, issues } = read(format, project);
      expect(back.coolingSystems[0]).toMatchObject({ type: 'none', eer: 0 });
      expect(issues.list.filter((issue) => issue.path === 'coolingSystems[0].eer')).toEqual([]);
    }
  });

  it('names a hot-water efficiency of 0 in the export warning and leaves it out of the file', () => {
    const project = load('edr-2022-epw001-project.json');
    expect(project.hotWaterSystems[0].efficiency).toBe(0);
    for (const format of ['UNIEC3', 'VABI'] as const) {
      expect(droppedOnExport(project, format)).toContain('hotWaterSystems[*].efficiency');
      const issue = read(format, project).issues.list.find((item) => item.path === 'hotWaterSystems[0].efficiency');
      expect(issue?.kind).toBe('defaulted');
    }
  });

  it('names an EER of 0 on real cooling and a COP of 0, not a usable value', () => {
    const project = load('2522-woning-aalten.oes.json');
    const zero: IProject = {
      ...project,
      heatingSystems: project.heatingSystems.map((system, index) => (index === 0 ? { ...system, cop: 0 } : system)),
      coolingSystems: [{ id: 'c', name: 'Split', type: 'split_unit', eer: 0 }],
    };
    for (const format of ['UNIEC3', 'VABI'] as const) {
      expect(droppedOnExport(project, format)).not.toContain('coolingSystems[*].eer');
      expect(droppedOnExport(project, format)).not.toContain('heatingSystems[*].cop');
      expect(droppedOnExport(zero, format)).toEqual(expect.arrayContaining(['coolingSystems[*].eer', 'heatingSystems[*].cop']));
      const issues = read(format, zero).issues.list;
      expect(issues.find((item) => item.path === 'coolingSystems[0].eer')?.kind).toBe('defaulted');
      expect(issues.find((item) => item.path === 'heatingSystems[0].cop')?.kind).toBe('defaulted');
    }
  });
});

describe('tilt of a pitched roof from UNIEC3 (review 39 follow-up)', () => {
  it('asks for the tilt of a pitched roof, as a VABI import does, and not for a flat roof', () => {
    const project = load('2522-woning-aalten.oes.json');
    const zone = project.zones[0];
    const pitched: IProject = { ...project, zones: [{ ...zone, surfaces: [
      ...zone.surfaces, { ...zone.surfaces[0], id: 'roof-s', name: 'Dakvlak zuid', type: 'roof', orientation: 'S', windows: [] },
      { ...zone.surfaces[0], id: 'roof-flat', name: 'Platdak', type: 'roof', orientation: 'horizontal', windows: [] },
    ] }, ...project.zones.slice(1)] };
    const sloped = pitched.zones[0].surfaces.length - 2;
    for (const issues of [readUNIEC3(exportToUNIEC3(pitched)).issues.list, readVABI(exportToVABI(pitched)).issues.list]) {
      const tilt = issues.find((item) => item.path === `zones[0].surfaces[${sloped}].tilt`);
      expect(tilt?.kind).toBe('dropped');
      expect(tilt?.message).toContain('Dakvlak zuid');
      expect(issues.find((item) => item.path === `zones[0].surfaces[${sloped + 1}].tilt`)).toBeUndefined();
    }
  });
});
