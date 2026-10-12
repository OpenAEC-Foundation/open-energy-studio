// @vitest-environment node
import { describe, expect, it } from 'vitest';
// @ts-expect-error jsdom ships without type declarations here; only DOMParser is used.
import { JSDOM } from 'jsdom';
import { readFileSync } from 'node:fs';
import { strFromU8, strToU8, unzipSync, zipSync } from 'fflate';
import type { IProject } from '../core/energy/types';
import { normalizeProject } from '../core/energy/normalizeProject';
import { exportToUNIEC3, importFromUNIEC3, readUNIEC3, uniec3ExportLosses } from '../core/io/UNIEC3Exporter';
import { exportToVABI, importFromVABI, readVABIFile, readVABIXml, vabiExportLosses } from '../core/io/VABIElementsBridge';
import { droppedOnExport, ImportFormatError, ImportIssues, lookup, parseDecimalText } from '../core/io/importIssues';
import { importFile } from '../core/io/importFile';
import { importErrorText, importIssuesSummary } from '../core/io/importText';
import { withImportRecord } from '../core/io/importLog';
import { labelInputSha256 } from '../core/nta/Registration';
import { nl } from '../i18n/nl';
import { en } from '../i18n/en';

// fflate needs the Node realm; the importer only needs a DOMParser.
globalThis.DOMParser = new JSDOM().window.DOMParser;

const TRAINING = ['2522-woning-aalten.oes.json', '2467-goejanverwelledijk-gouda.oes.json', '2786-reddingspost-kijkduin.oes.json'];
const load = (file: string): IProject =>
  normalizeProject(JSON.parse(readFileSync(`training-data/${file}`, 'utf8')).project);

const psiL = (project: IProject) => project.zones.map((zone) =>
  zone.thermalBridges.reduce((sum, bridge) => sum + bridge.psiValue * bridge.length, 0));

function translate(table: Record<string, string>) {
  return (key: string, params?: Record<string, string | number>) => {
    const text = table[key];
    if (text === undefined) return key;
    return text.replace(/\{\{(\w+)\}\}/g, (_, name) => String(params?.[name] ?? ''));
  };
}

type Entity = { NTAEntityDataId: string; NTAEntityId: string; NTAPropertyDatas: { NTAPropertyId: string; Value: string }[] };
const ent = (id: string, type: string, props: Record<string, string> = {}): Entity => ({
  NTAEntityDataId: id, NTAEntityId: type,
  NTAPropertyDatas: Object.entries(props).map(([NTAPropertyId, Value]) => ({ NTAPropertyId, Value })),
});
const rel = (id: string, kind: string, parent: string, child: string) =>
  ({ NTAEntityDataId: id, NTAEntityRelationId: kind, ParentNTAEntityDataId: parent, ChildNTAEntityDataId: child });
const uniec = (entities: unknown[], relations: unknown[] = []) => zipSync({
  'buildings/1/entities.json': strToU8(JSON.stringify(entities)),
  'buildings/1/relations.json': strToU8(JSON.stringify(relations)),
});
const zoneWith = (props: Record<string, string>, extra: Entity[] = [], extraRel: unknown[] = []) => uniec(
  [ent('g', 'GEB'), ent('u', 'UNIT'), ent('r', 'RZ', props), ...extra],
  [rel('x', 'UNIT_RZ', 'u', 'r'), ...extraRel],
);

const vabiXml = (zoneBody: string, extra = '') =>
  `<?xml version="1.0" encoding="utf-8"?><Project><Objecten><Object><Naam>T</Naam><Rekenzones><Rekenzone><Naam>Z</Naam>`
  + `<Gebruiksoppervlakte>100</Gebruiksoppervlakte><Volume>260</Volume><Hoogte>2,6</Hoogte><Luchtdoorlatendheid>0,6</Luchtdoorlatendheid>`
  + `${zoneBody}</Rekenzone></Rekenzones></Object></Objecten>${extra}</Project>`;
const hoofdvlak = (inner: string) => `<Hoofdvlak><Naam>V</Naam><Locatie>2</Locatie><Orientatie>4</Orientatie>${inner}</Hoofdvlak>`;
const dicht = (name: string, area: number, rc?: number, u?: number) => `<Deelvlak><Type>Dicht</Type><Naam>${name}</Naam>`
  + `<Oppervlakte>${area}</Oppervlakte>${rc === undefined ? '' : `<RcWaarde>${rc}</RcWaarde>`}${u === undefined ? '' : `<UWaarde>${u}</UWaarde>`}</Deelvlak>`;

describe('VABI thermal bridges (review 16, fix 1)', () => {
  it('keeps ΣψL per zone on a round trip of the training projects', () => {
    for (const file of TRAINING) {
      const project = load(file);
      const back = importFromVABI(exportToVABI(project));
      const before = psiL(project);
      const after = psiL(back);
      expect(after.length).toBe(before.length);
      after.forEach((value, index) => expect(value).toBeCloseTo(before[index], 9));
      expect(back.zones.map((zone) => zone.thermalBridges.length)).toEqual(project.zones.map((zone) => zone.thermalBridges.length));
    }
  });

  it('reads the old export (the same list under every Hoofdvlak) once, with an issue', () => {
    const list = '<KoudebrugList><Koudebrug><Naam>K</Naam><PsiWaarde>0,1</PsiWaarde><Lengte>10</Lengte></Koudebrug></KoudebrugList>';
    const surfaces = [1, 2, 3].map(() => hoofdvlak(`<Oppervlakte>10</Oppervlakte><DeelvlakList>${dicht('W', 10, 3, 0.3)}</DeelvlakList>${list}`)).join('');
    const { project, issues } = readVABIXml(vabiXml(`<Geometrie>${surfaces}</Geometrie>`));
    expect(project.zones[0].thermalBridges).toHaveLength(1);
    expect(psiL(project)[0]).toBeCloseTo(1, 12);
    expect(issues.list.some((issue) => issue.kind === 'duplicate')).toBe(true);
  });

  it('records a missing ψ as an issue', () => {
    const list = '<KoudebrugList><Koudebrug><Naam>K</Naam><Lengte>10</Lengte></Koudebrug></KoudebrugList>';
    const { issues } = readVABIXml(vabiXml(`<Geometrie></Geometrie>${list}`));
    expect(issues.list.some((issue) => issue.kind === 'defaulted' && issue.path.includes('psiValue'))).toBe(true);
  });
});

describe('U values (review 16, fix 2 and 8)', () => {
  it('keeps the U of a construction entered without Rc and never imports U = 0', () => {
    const project = load('2522-woning-aalten.oes.json');
    const target = project.constructions.find((item) => project.zones.some((zone) => zone.surfaces.some((s) => s.constructionId === item.id)))!;
    target.rcValue = 0;
    const back = importFromVABI(exportToVABI(project));
    const linked = back.zones.flatMap((zone) => zone.surfaces)
      .map((surface) => back.constructions.find((item) => item.id === surface.constructionId))
      .filter((item) => item !== undefined);
    expect(linked.some((item) => item.name === target.name && item.uValue === target.uValue)).toBe(true);
    expect(back.constructions.every((item) => item.uValue > 0)).toBe(true);
    const uniecBack = importFromUNIEC3(exportToUNIEC3(project));
    expect(uniecBack.constructions.every((item) => item.uValue > 0)).toBe(true);
  });

  it('falls back to the library by name and leaves a construction without U or Rc unlinked', () => {
    const lib = '<ConstructieBibliotheek><Constructie><Naam>Gevel</Naam><RcWaarde>4,7</RcWaarde><UWaarde>0,2</UWaarde></Constructie></ConstructieBibliotheek>';
    const xml = vabiXml(`<Geometrie>${hoofdvlak(`<Oppervlakte>20</Oppervlakte><DeelvlakList>${dicht('Gevel', 20)}</DeelvlakList>`)}`
      + `${hoofdvlak(`<Oppervlakte>5</Oppervlakte><DeelvlakList>${dicht('Leeg', 5)}</DeelvlakList>`)}</Geometrie>`, lib);
    const { project, issues } = readVABIXml(xml);
    const [first, second] = project.zones[0].surfaces;
    expect(project.constructions.find((item) => item.id === first.constructionId)?.uValue).toBe(0.2);
    expect(second.constructionId).toBe('');
    expect(issues.list.some((issue) => issue.kind === 'dropped' && issue.message.includes('Leeg'))).toBe(true);
    expect(project.constructions.every((item) => item.uValue > 0)).toBe(true);
  });

  it('keeps the construction count on a round trip', () => {
    for (const file of TRAINING) {
      const project = load(file);
      const used = new Set(project.zones.flatMap((zone) => zone.surfaces.map((s) => s.constructionId)));
      const back = importFromVABI(exportToVABI(project));
      expect(back.constructions.length).toBeLessThanOrEqual(project.constructions.length);
      expect(back.constructions.length).toBeGreaterThanOrEqual([...used].filter((id) => project.constructions.some((c) => c.id === id)).length);
    }
  });

  it('splits a Hoofdvlak with several opaque Deelvlakken', () => {
    const xml = vabiXml(`<Geometrie>${hoofdvlak(`<Oppervlakte>30</Oppervlakte><DeelvlakList>${dicht('A', 20, 3, 0.3)}${dicht('B', 10, 1, 0.8)}</DeelvlakList>`)}</Geometrie>`);
    const { project } = readVABIXml(xml);
    const surfaces = project.zones[0].surfaces;
    expect(surfaces).toHaveLength(2);
    expect(surfaces.map((s) => s.area)).toEqual([20, 10]);
    const u = surfaces.map((s) => project.constructions.find((c) => c.id === s.constructionId)?.uValue);
    expect(u).toEqual([0.3, 0.8]);
  });
});

describe('numbers and normalizing (review 16, fix 3, 5 and 9)', () => {
  it('reads Dutch thousands separators and rejects text and NaN', () => {
    expect(parseDecimalText('1.234,5')).toBe(1234.5);
    expect(parseDecimalText('0,98')).toBe(0.98);
    expect(parseDecimalText('12')).toBe(12);
    expect(parseDecimalText('abc')).toBeNull();
    expect(parseDecimalText('NaN')).toBeNull();
    expect(parseDecimalText('Infinity')).toBeNull();
    expect(parseDecimalText('')).toBeNull();
    const { project } = readUNIEC3(zoneWith({ RZ_AG: '1.234,5' }));
    expect(project.zones[0].floorArea).toBe(1234.5);
  });

  it('records invalid and out-of-range numbers and the project survives normalizing', () => {
    const { project, issues } = readUNIEC3(zoneWith({ RZ_AG: 'abc' }));
    expect(issues.list.some((issue) => issue.kind === 'invalid' && issue.path.includes('floorArea'))).toBe(true);
    expect(Number.isFinite(project.zones[0].floorArea)).toBe(true);
    expect(() => normalizeProject(project)).not.toThrow();
  });

  it('marks qv10 0 as assumed with an issue', () => {
    const { project, issues } = readUNIEC3(zoneWith({}, [ent('l', 'LUCHTDH', { LUCHTDH_QV10: '0' })], [rel('y', 'RZ_LUCHTDH', 'r', 'l')]));
    expect(project.zones[0].airTightness.qv10).toBeGreaterThan(0);
    expect(project.zones[0].airTightness.assumed).toBe(true);
    expect(issues.list.some((issue) => issue.path.includes('qv10'))).toBe(true);
    expect(() => normalizeProject(project)).not.toThrow();
  });

  it('exports full precision to UNIEC3', () => {
    const project = load('2522-woning-aalten.oes.json');
    project.zones[0].airTightness = { ...project.zones[0].airTightness, qv10: 0.987654 };
    project.zones[0].floorArea = 123.456789;
    const back = importFromUNIEC3(exportToUNIEC3(project));
    expect(back.zones[0].airTightness.qv10).toBe(0.987654);
    expect(back.zones[0].floorArea).toBe(123.456789);
  });

  it('records silently defaulted VABI values', () => {
    const { project, issues } = readVABIXml(vabiXml('<Geometrie></Geometrie>').replace('<Hoogte>2,6</Hoogte>', ''));
    expect(project.zones[0].height).toBe(2.6);
    expect(issues.list.some((issue) => issue.kind === 'defaulted' && issue.path === 'zones[0].height')).toBe(true);
  });
});

describe('own-property lookups (review 16, fix 4)', () => {
  it('ignores prototype names as codes', () => {
    expect(lookup({ a: 1 }, 'constructor')).toBeUndefined();
    expect(lookup({ a: 1 }, '__proto__')).toBeUndefined();
    expect(lookup({ a: 1 }, 'toString')).toBeUndefined();
    const { project } = readUNIEC3(uniec(
      [ent('g', 'GEB', { GEB_FUNCTIE: 'constructor' }), ent('u', 'UNIT'), ent('r', 'RZ'), ent('b', 'BEGR', { BEGR_TYPE: 'toString', BEGR_ORIENT: '__proto__' })],
      [rel('x', 'UNIT_RZ', 'u', 'r'), rel('y', 'RZ_BEGR', 'r', 'b')],
    ));
    expect(typeof project.buildingFunction).toBe('string');
    const surface = project.zones[0].surfaces[0];
    expect(typeof surface.type).toBe('string');
    expect(typeof surface.orientation).toBe('string');
    expect(() => normalizeProject(project)).not.toThrow();
  });
});

describe('dropped model data (review 16, fix 6)', () => {
  it('lists what each format cannot hold and records it on import', () => {
    for (const file of TRAINING) {
      const project = load(file);
      if (!project.ntaCalculation) continue;
      expect(droppedOnExport(project, 'UNIEC3')).toContain('ntaCalculation');
      expect(uniec3ExportLosses(project)).toContain('ntaCalculation');
      expect(vabiExportLosses(project)).toContain('ntaCalculation');
      // The ground route (perimeter, crawlspace, heated basement) is named on its own.
      const ground = (project.ntaCalculation as { groundFloors?: unknown[] }).groundFloors ?? [];
      expect(droppedOnExport(project, 'UNIEC3').includes('ntaCalculation.groundFloors')).toBe(ground.length > 0);
      expect(droppedOnExport(project, 'VABI').includes('ntaCalculation.groundFloors')).toBe(ground.length > 0);
    }
    const project = load('2522-woning-aalten.oes.json');
    expect(droppedOnExport({ ...project, importLog: [{ tool: 'X', importedAt: 'now' }] }, 'UNIEC3')).not.toContain('importLog');
    const { issues } = readUNIEC3(exportToUNIEC3(project));
    expect(issues.list.some((issue) => issue.kind === 'dropped' && issue.path === 'ntaCalculation')).toBe(true);
    expect(issues.list.find((issue) => issue.path === 'ntaCalculation')?.message).toMatch(/verwarmde kelder/);
  });
});

describe('VABI tilt and orientation (review 16, fix 7)', () => {
  const roof = (tilt: string) => vabiXml(`<Geometrie><Hoofdvlak><Naam>Dak</Naam><Locatie>1</Locatie><Orientatie>2</Orientatie>`
    + `<Hellingshoek>${tilt}</Hellingshoek><Oppervlakte>40</Oppervlakte><DeelvlakList>${dicht('D', 40, 6, 0.16)}</DeelvlakList></Hoofdvlak></Geometrie>`);

  it('reads Hellingshoek 0 and 180 as horizontal', () => {
    for (const tilt of ['0', '180']) {
      const { project } = readVABIXml(roof(tilt));
      expect(project.zones[0].surfaces[0].orientation).toBe('horizontal');
      expect(project.ntaCalculation?.surfaceTilts?.length ?? 0).toBe(0);
    }
  });

  it('keeps another tilt in ntaCalculation.surfaceTilts', () => {
    const { project } = readVABIXml(roof('35'));
    const surface = project.zones[0].surfaces[0];
    expect(surface.orientation).not.toBe('horizontal');
    const entry = project.ntaCalculation?.surfaceTilts?.find((item) => item.surfaceId === surface.id);
    expect(entry?.tiltDeg).toBe(35);
  });

  it('exports the tilt it holds', () => {
    const { project } = readVABIXml(roof('35'));
    const xml = strFromU8(unzipSync(exportToVABI(project))['project.xml']);
    expect(xml).toContain('<Hellingshoek>35</Hellingshoek>');
  });
});

describe('several buildings and identifiers (review 16, fix 10)', () => {
  it('warns on more than one building and on duplicate and unknown ids', () => {
    const zip = zipSync({
      'buildings/1/entities.json': strToU8(JSON.stringify([ent('g', 'GEB', { GEB_NAAM: 'A' }), ent('g', 'GEB'), ent('u', 'UNIT'), ent('r', 'RZ')])),
      'buildings/1/relations.json': strToU8(JSON.stringify([rel('x', 'UNIT_RZ', 'u', 'r'), rel('y', 'RZ_BEGR', 'r', 'missing')])),
      'buildings/2/entities.json': strToU8(JSON.stringify([ent('g', 'GEB', { GEB_NAAM: 'B' })])),
      'buildings/2/relations.json': strToU8('[]'),
    });
    const { project, issues } = readUNIEC3(zip);
    expect(project.name).toBe('A');
    const kinds = issues.list.map((issue) => issue.kind);
    expect(kinds).toContain('multiple');
    expect(kinds).toContain('duplicate');
    expect(kinds).toContain('unknown_reference');
  });

  it('prefers project.xml over other XML in the EPA file', () => {
    const zip = zipSync({ 'a/lib.xml': strToU8('<Lib/>'), 'project.xml': strToU8(vabiXml('<Geometrie></Geometrie>').replace('<Naam>T</Naam>', '<Naam>Echt</Naam>')) });
    expect(readVABIFile(zip).project.name).toBe('Echt');
  });
});

describe('VABI encodings (review 16, fix 11)', () => {
  it('reads windows-1252, UTF-16 and a UTF-8 BOM', () => {
    const xml1252 = vabiXml('').replace('utf-8', 'windows-1252').replace('<Naam>T</Naam>', '<Naam>Caf\xe9</Naam>');
    const bytes1252 = Uint8Array.from([...xml1252].map((c) => c.charCodeAt(0)));
    expect(importFromVABI(zipSync({ 'project.xml': bytes1252 })).name).toBe('Café');
    const xml16 = vabiXml('').replace('utf-8', 'utf-16').replace('<Naam>T</Naam>', '<Naam>Ü</Naam>');
    const bytes16 = new Uint8Array([0xff, 0xfe, ...[...xml16].flatMap((c) => [c.charCodeAt(0) & 0xff, c.charCodeAt(0) >> 8])]);
    expect(importFromVABI(zipSync({ 'project.xml': bytes16 })).name).toBe('Ü');
    const bom = new Uint8Array([0xef, 0xbb, 0xbf, ...strToU8(vabiXml(''))]);
    expect(importFromVABI(zipSync({ 'project.xml': bom })).name).toBe('T');
  });

  it('refuses a DOCTYPE and invalid UTF-8', () => {
    const doctype = '<?xml version="1.0"?><!DOCTYPE Project [<!ENTITY x SYSTEM "file:///etc/hostname">]><Project><Objecten><Object><Naam>&x;</Naam></Object></Objecten></Project>';
    expect(() => importFromVABI(zipSync({ 'project.xml': strToU8(doctype) }))).toThrow(ImportFormatError);
    try { importFromVABI(zipSync({ 'project.xml': strToU8(doctype) })); } catch (err) { expect((err as ImportFormatError).code).toBe('vabi_doctype'); }
    const broken = new Uint8Array([...strToU8('<?xml version="1.0" encoding="utf-8"?><Project><Naam>'), 0xff, 0xfe, ...strToU8('</Naam></Project>')]);
    expect(() => importFromVABI(zipSync({ 'project.xml': broken }))).toThrow(ImportFormatError);
  });
});

describe('error texts (review 16, fix 12)', () => {
  it('reports malformed UNIEC3 JSON with a code and NL/EN text', () => {
    const zip = zipSync({ 'buildings/1/entities.json': strToU8('[{'), 'buildings/1/relations.json': strToU8('[]') });
    let caught: unknown;
    try { importFromUNIEC3(zip); } catch (err) { caught = err; }
    expect(caught).toBeInstanceOf(ImportFormatError);
    expect((caught as ImportFormatError).code).toBe('uniec3_json_invalid');
    expect(importErrorText(caught, translate(nl))).toContain('geen geldige JSON');
    expect(importErrorText(caught, translate(en))).toContain('not valid JSON');

    let shape: unknown;
    try { importFromUNIEC3(uniec([{ NTAEntityDataId: 'g', NTAEntityId: 'GEB', NTAPropertyDatas: 'x' }])); } catch (err) { shape = err; }
    expect((shape as ImportFormatError).code).toBe('uniec3_shape_invalid');
  });

  it('defines every error code and issue kind in NL and EN', () => {
    const codes = ['uniec3_json_invalid', 'uniec3_shape_invalid', 'uniec3_no_building', 'uniec3_files_missing', 'uniec3_no_geb',
      'vabi_encoding_unknown', 'vabi_encoding_invalid', 'vabi_no_project_xml', 'vabi_doctype', 'vabi_xml_invalid', 'vabi_no_object'];
    const kinds = ['defaulted', 'invalid', 'clamped', 'dropped', 'unknown_reference', 'duplicate', 'multiple'];
    for (const table of [nl, en] as Record<string, string>[]) {
      for (const code of codes) expect(table[`import.error.${code}`]).toBeTruthy();
      for (const kind of kinds) expect(table[`import.issue.kind.${kind}`]).toBeTruthy();
    }
  });
});

describe('import record (design)', () => {
  it('stores file name, SHA-256 and issues, outside the label input hash', async () => {
    const issues = new ImportIssues();
    issues.add('defaulted', 'zones[0].height', 'Hoogte ontbrak', 2.6);
    const project = load('2522-woning-aalten.oes.json');
    const recorded = withImportRecord(project, 'VABI', 'a.epa', '2026-10-10T00:00:00Z', { sha256: 'ab'.repeat(32), issues: issues.list });
    const record = recorded.importLog![recorded.importLog!.length - 1];
    expect(record).toMatchObject({ tool: 'VABI', fileName: 'a.epa', sha256: 'ab'.repeat(32) });
    expect(record.issues).toHaveLength(1);
    expect(await labelInputSha256(recorded)).toBe(await labelInputSha256(project));
  });

  it('hashes the file and returns the common import result', async () => {
    const bytes = exportToVABI(load('2522-woning-aalten.oes.json'));
    const file = new File([new Uint8Array(bytes)], 'woning.epa');
    const result = await importFile(file, readVABIFile);
    expect(result.fileName).toBe('woning.epa');
    expect(result.sha256).toMatch(/^[0-9a-f]{64}$/);
    expect(Array.isArray(result.issues)).toBe(true);
    expect(result.project.zones.length).toBeGreaterThan(0);
  });

  it('summarizes issues for the toast in NL and EN', () => {
    const issues = new ImportIssues();
    for (let i = 0; i < 6; i += 1) issues.add('defaulted', `zones[${i}].height`, 'x', 2.6);
    const nlText = importIssuesSummary(issues.list, translate(nl));
    expect(nlText).toContain('6×');
    expect(nlText).toContain('en nog 2 meer');
    expect(importIssuesSummary(issues.list, translate(en))).toContain('and 2 more');
  });
});
