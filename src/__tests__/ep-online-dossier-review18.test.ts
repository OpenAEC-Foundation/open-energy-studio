import { describe, expect, it } from 'vitest';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { buildEpOnlineOverview, epOnlineOverviewXml } from '../core/report/EpOnlineOverview';
import { buildProjectDossier, checkDossierCompleteness, usesQualityDeclaration } from '../core/report/ProjectDossier';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { generateEnergyPerformanceReportHTML } from '../core/report/EnergyPerformanceReport';

function project(registration: IProject['registration'], buildingFunction = 'residential', extra: Partial<IProject> = {}): IProject {
  return {
    id: 'p1', name: 'W', description: '', buildingFunction, address: '', city: '',
    zones: [], heatingSystems: [], ventilationSystems: [], solarPV: [], solarThermal: [], constructions: [],
    registration, ...extra,
  } as unknown as IProject;
}

function assessment(scope: 'residential' | 'utility' | null = null): ProjectPerformanceAssessment {
  return {
    status: 'calculated_unverified', targetNormVersion: 'x', kernelVersion: '0', inputFingerprint: 'a',
    attestStatus: 'unattested', gaps: [], geometry: { usableFloorAreaM2: 100 },
    derivedInput: scope ? { calculationScope: scope } : null, performance: null,
  } as unknown as ProjectPerformanceAssessment;
}

const item = (items: ReturnType<typeof checkDossierCompleteness>, id: string) => items.find((entry) => entry.id === id);

describe('EP-Online overview (review 18)', () => {
  it('marks the gelijkende woning, not the visited referentiewoning, as based on a reference (BRL 9500-W §4.3 p. 26)', () => {
    const similar = buildEpOnlineOverview(project({ representation: 'similar', referenceObjectId: 'r1' } as never), null);
    const reference = buildEpOnlineOverview(project({ representation: 'reference' } as never), null);
    const unique = buildEpOnlineOverview(project({ representation: 'unique' } as never), null);
    expect(similar.pandcertificaat.Op_basis_van_referentiegebouw).toBe(true);
    expect(reference.pandcertificaat.Op_basis_van_referentiegebouw).toBe(false);
    expect(unique.pandcertificaat.Op_basis_van_referentiegebouw).toBe(false);
  });

  it('keeps a too long Huisnummertoevoeging out of the XML and reports it (XSD maxLength 4)', () => {
    const overview = buildEpOnlineOverview(project({ houseNumber: '12', houseNumberAddition: 'bis-23' } as never), null);
    expect(overview.pandcertificaat.Huisnummertoevoeging).toBeUndefined();
    expect(epOnlineOverviewXml(overview)).not.toContain('bis-23');
    expect(overview.omitted.some((entry) => entry.field === 'Huisnummertoevoeging')).toBe(true);
    const fits = buildEpOnlineOverview(project({ houseNumber: '12', houseNumberAddition: 'bis' } as never), null);
    expect(fits.pandcertificaat.Huisnummertoevoeging).toBe('bis');
  });

  it('reports a too long addition inside the house number instead of dropping the number silently', () => {
    const overview = buildEpOnlineOverview(project({ houseNumber: '12-1234567' } as never), null);
    expect(overview.pandcertificaat.Huisnummer).toBe(12);
    expect(overview.pandcertificaat.Huisnummertoevoeging).toBeUndefined();
    expect(overview.omitted.some((entry) => entry.field === 'Huisnummertoevoeging')).toBe(true);
  });

  it('reports an unparsable house number', () => {
    const overview = buildEpOnlineOverview(project({ houseNumber: 'achter de kerk' } as never), null);
    expect(overview.pandcertificaat.Huisnummer).toBeUndefined();
    expect(overview.omitted.some((entry) => entry.field === 'Huisnummer')).toBe(true);
  });

  it('limits Gebouwtype to 75 characters', () => {
    const overview = buildEpOnlineOverview(project({ buildingType: 'x'.repeat(76) } as never), null);
    expect(overview.pandcertificaat.Gebouwtype).toBeUndefined();
    expect(overview.omitted.some((entry) => entry.field === 'Gebouwtype')).toBe(true);
  });

  it('labels a logiesfunctie calculated as a woning with the woning class (art. 5.11 lid 1, 5.13)', () => {
    const lodging = buildEpOnlineOverview(project({} as never, 'lodging'), assessment('residential'));
    expect(lodging.pandcertificaat.Gebouwklasse).toBe('W');
    const office = buildEpOnlineOverview(project({} as never, 'residential'), assessment('utility'));
    expect(office.pandcertificaat.Gebouwklasse).toBe('U');
    // Without kernel input the building function decides.
    expect(buildEpOnlineOverview(project({} as never, 'office'), null).pandcertificaat.Gebouwklasse).toBe('U');
  });
});

describe('dossier checklist (review 18)', () => {
  it('requires the registering adviser\'s registration number (BRL 9500-W Bijlage 3 p. 62)', () => {
    const base = { registrationDate: '2026-10-01', epOnlineNumber: '1' };
    const without = checkDossierCompleteness({
      project: project({ ...base, registeringAdvisor: { name: 'A', competenceNumber: '' } } as never), assessment: assessment(),
    });
    expect(item(without, 'registration_data')?.status).toBe('missing');
    const withNumber = checkDossierCompleteness({
      project: project({ ...base, registeringAdvisor: { name: 'A', competenceNumber: 'EP-123' } } as never), assessment: assessment(),
    });
    expect(item(withNumber, 'registration_data')?.status).toBe('ok');
  });

  it('marks the delivered report missing when the dossier carries no report', async () => {
    const without = await buildProjectDossier({ project: project({} as never), assessment: assessment(), reportHtml: null });
    expect(without.files['rekenrapport.html']).toBeUndefined();
    expect(without.manifest.checklist.find((entry) => entry.id === 'delivered_report')?.status).toBe('missing');
    const withReport = await buildProjectDossier({ project: project({} as never), assessment: assessment(), reportHtml: '<p>rapport</p>' });
    expect(withReport.manifest.checklist.find((entry) => entry.id === 'delivered_report')?.status).toBe('ok');
  });

  it('requires a used kwaliteitsverklaring for every purpose, with a photo or invoice (W Bijlage 3 p. 63)', () => {
    const nta = { ntaCalculation: { heating: { generator: { kind: 'heat_pump', qualityDeclaration: { cop: 4.2 } } } } } as unknown as Partial<IProject>;
    const plain = checkDossierCompleteness({ project: project({ purpose: 'existing_building' } as never), assessment: assessment() });
    expect(item(plain, 'quality_declarations')?.status).toBe('not_applicable');
    const declared = checkDossierCompleteness({ project: project({ purpose: 'existing_building' } as never, 'residential', nta), assessment: assessment() });
    expect(item(declared, 'quality_declarations')?.status).toBe('missing');
    const evidence = [
      { id: 'ev-1', kind: 'quality_declaration', fileName: 'v.pdf', sha256: 'a', addedAt: '2026-10-01' },
      { id: 'ev-2', kind: 'invoice', fileName: 'f.pdf', sha256: 'b', addedAt: '2026-10-01' },
    ];
    const proven = checkDossierCompleteness({
      project: project({ purpose: 'existing_building', evidence } as never, 'residential', nta), assessment: assessment(),
    });
    expect(item(proven, 'quality_declarations')?.status).toBe('ok');
  });

  it('recognises the declaration routes of the input', () => {
    expect(usesQualityDeclaration({ ventilation: { units: [{ declaredControlFactor: { value: 0.5 } }] } })).toBe(true);
    expect(usesQualityDeclaration({ heating: { generator: { declaredAuxiliaryConstants: { a: 1 } } } })).toBe(true);
    expect(usesQualityDeclaration({ hotWater: { generator: { declared: { value: 2.1 } } } })).toBe(true);
    expect(usesQualityDeclaration({ x: { qualityDeclarationPresent: false } })).toBe(false);
    expect(usesQualityDeclaration({ windowObstructions: [{ obstruction: { method: 'declared' } }] })).toBe(false);
  });

  it('asks a utility dossier for humidification and lighting (BRL 9500-U Bijlage 3 p. 53)', () => {
    const utility = checkDossierCompleteness({ project: project({} as never, 'office'), assessment: assessment('utility') });
    expect(item(utility, 'humidification_type')).toBeDefined();
    expect(item(utility, 'lighting_type')).toBeDefined();
    const dwelling = checkDossierCompleteness({ project: project({} as never), assessment: assessment('residential') });
    expect(item(dwelling, 'humidification_type')).toBeUndefined();
    expect(item(dwelling, 'lighting_type')).toBeUndefined();
  });
});

describe('report A0 table (review 18)', () => {
  it('names the primary fossil condition after art. 5.11 lid 5, not "BENG 2", and shows conditions a and c', () => {
    const root = resolve(__dirname, '../..');
    const read = (file: string) => JSON.parse(readFileSync(resolve(root, 'training-data', file), 'utf8'));
    const projectInput = read('nta8800-example-terraced-dwelling.json') as IProject;
    const kernel = read('nta8800-example-terraced-dwelling.kernel-output.json') as ProjectPerformanceAssessment;
    kernel.performance!.a0Check = {
      source: 'Omgevingsregeling art. 5.11 lid 5', primaryFossilMaxKwhPerM2: 27,
      energyNeedMeets: true, primaryFossilMeets: false, renewableShareMeets: true,
      noOnSiteFossilCombustion: false, eligible: false,
    };
    const html = generateEnergyPerformanceReportHTML(projectInput, kernel, { level: 'standard', generatedAt: new Date('2026-10-10T10:00:00Z') });
    const a0 = html.slice(html.indexOf('Aanduiding A0'));
    expect(a0).not.toContain('BENG 2 ≤');
    expect(a0).toContain('Primair fossiel energiegebruik ≤');
    expect(a0).toContain('a. Energiebehoefte');
    expect(a0).toContain('c. Aandeel hernieuwbare energie');
    expect(a0).toContain('d. Geen fossiele verbranding');
  });
});
