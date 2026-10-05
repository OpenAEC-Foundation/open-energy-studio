import { describe, expect, it } from 'vitest';
import { strFromU8 } from 'fflate';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import {
  EP_ONLINE_FIELDS, EP_ONLINE_REQUIRED, buildEpOnlineOverview, epOnlineOverviewXml,
} from '../core/report/EpOnlineOverview';
import { buildProjectDossier } from '../core/report/ProjectDossier';

const CLASSES = ['A+++++', 'A++++', 'A+++', 'A++', 'A+', 'A', 'B', 'C', 'D', 'E', 'F', 'G'];

function project(overrides: Partial<IProject> = {}): IProject {
  return {
    id: 'p1', name: 'Woning', description: '', buildingFunction: 'residential', address: '', city: '',
    zones: [], heatingSystems: [], ventilationSystems: [], solarPV: [], solarThermal: [], constructions: [],
    registration: {
      purpose: 'delivery', surveyType: 'detailed', representation: 'unique',
      bagObjectId: '0363010000123456', postcode: '1234 ab', houseNumber: '12A',
      certificateNumber: 'CERT-1', buildingType: 'Tussenwoning',
      surveyDate: '2026-09-01', registrationDate: '2026-09-15', constructionYear: 2026,
    },
    ...overrides,
  } as unknown as IProject;
}

function assessment(labelEp2: number): ProjectPerformanceAssessment {
  return {
    status: 'calculated_unverified', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
    inputFingerprint: 'abc', attestStatus: 'unattested', gaps: [],
    geometry: { usableFloorAreaM2: 120.456, lossAreaM2: 200, envelopeAreaM2: 200, lossAreaRatio: 1.66, unclassifiedSurfaceCount: 0 },
    derivedInput: null,
    performance: {
      status: 'calculated_unverified',
      needIndicatorKwhPerM2Year: 55.123,
      primaryFossilIndicatorKwhPerM2Year: 25.5,
      renewableSharePercent: 61.27,
      labelPrimaryFossilIndicatorKwhPerM2Year: labelEp2,
      labelRenewableSharePercent: 55.0,
      indicativeLabelClass: 'A+++',
      tojuliMaxK: 0.85,
      bblCheck: { limits: { energyNeedMaxKwhPerM2: 59.8, primaryFossilMaxKwhPerM2: 30, renewableShareMinPercent: 50, lightConstructionAllowanceApplied: false } },
    },
    registration: { validUntil: '2036-09-01' },
    labelData: { general: { constructionYear: 2026 }, indicators: { heatingNeedKwhPerM2: 40.2 } },
  } as unknown as ProjectPerformanceAssessment;
}

describe('EP-Online data overview', () => {
  it('uses the export schema field names in schema order with valid simple types', () => {
    const overview = buildEpOnlineOverview(project(), assessment(25.5));
    const fields = Object.keys(overview.pandcertificaat);
    const order = EP_ONLINE_FIELDS.map(([name]) => name as string);
    expect(fields.every((field) => order.includes(field))).toBe(true);
    expect(fields).toEqual([...fields].sort((a, b) => order.indexOf(a) - order.indexOf(b)));
    const kinds = new Map(EP_ONLINE_FIELDS.map(([name, kind]) => [name as string, kind as string]));
    for (const [field, value] of Object.entries(overview.pandcertificaat)) {
      const kind = kinds.get(field);
      if (kind === 'date') expect(value).toMatch(/^\d{4}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])$/);
      if (kind === 'decimal' || kind === 'integer') expect(typeof value).toBe('number');
      if (kind === 'integer') expect(Number.isInteger(value)).toBe(true);
      if (kind === 'boolean') expect(typeof value).toBe('boolean');
      if (kind === 'zipcode') expect(value).toMatch(/^\d{4}[A-Z]{2}$/);
      if (kind === 'bagid') expect(value).toMatch(/^\d{16}$/);
      if (kind === 'gebouwklasse') expect(['W', 'U']).toContain(value);
      if (kind === 'class') expect(value === null || CLASSES.includes(value as string)).toBe(true);
      if (kind === 'opnametype') expect(['Basisopname', 'Detailopname']).toContain(value);
    }
    expect(overview.missingRequired).toEqual([]);
    expect(EP_ONLINE_REQUIRED.every((field) => field in overview.pandcertificaat)).toBe(true);
    expect(overview.pandcertificaat).toMatchObject({
      Registratiedatum: '20260915', Opnamedatum: '20260901', Geldig_tot: '20360901',
      Status: 'Oplevering', Gebouwklasse: 'W', Postcode: '1234AB', Huisnummer: 12, Huisletter: 'A',
      BAGVerblijfsobjectID: '0363010000123456', Gebruiksoppervlakte_thermische_zone: 120.46,
      Energieklasse: 'A+++', Primaire_fossiele_energie: 25.5, Eis_temperatuuroverschrijding: 1.2,
    });
    // Without area measures the forfait fields stay out (bijlage 2: only with EMG).
    expect(overview.pandcertificaat).not.toHaveProperty('Primaire_fossiele_energie_EMG_forfaitair');
    expect(overview.notice).toContain('GEEN registratiebestand');
  });

  it('adds the EMG forfait values for a dwelling with area measures', () => {
    const overview = buildEpOnlineOverview(project(), assessment(41.2));
    expect(overview.pandcertificaat.Primaire_fossiele_energie_EMG_forfaitair).toBe(41.2);
    expect(overview.pandcertificaat.Aandeel_hernieuwbare_energie_EMG_forfaitair).toBe(55);
  });

  it('lists missing required fields and keeps a nil class without a calculation', () => {
    const overview = buildEpOnlineOverview(project({ registration: undefined }), null);
    expect(overview.pandcertificaat.Energieklasse).toBeNull();
    expect(overview.missingRequired).toEqual(['Registratiedatum', 'Opnamedatum', 'Geldig_tot', 'Certificaathouder']);
    expect(epOnlineOverviewXml(overview)).toContain('<Energieklasse xsi:nil="true"/>');
  });

  it('writes the XML elements in schema order and maps a pand id to BAGPandIDs', () => {
    const utility = project({ buildingFunction: 'office' } as Partial<IProject>);
    utility.registration = { ...utility.registration, bagObjectId: '0363100012345678', purpose: 'existing_building' };
    const overview = buildEpOnlineOverview(utility, assessment(25.5));
    expect(overview.pandcertificaat.Gebouwklasse).toBe('U');
    expect(overview.pandcertificaat.BAGPandIDs).toEqual(['0363100012345678']);
    // Existing building: no requirement values, no TOjuli for utility.
    expect(overview.pandcertificaat).not.toHaveProperty('Eis_energiebehoefte');
    expect(overview.pandcertificaat).not.toHaveProperty('Temperatuuroverschrijding');
    const xml = epOnlineOverviewXml(overview);
    const tags = [...xml.matchAll(/<Pandcertificaat>(.*)<\/Pandcertificaat>/g)][0][1]
      .match(/<([A-Za-z_0-9]+)[ >/]/g)!.map((tag) => tag.slice(1, -1)).filter((tag) => tag !== 'BAGPandID');
    expect(tags).toEqual(Object.keys(overview.pandcertificaat));
  });

  it('is part of the project dossier', async () => {
    const bundle = await buildProjectDossier({ project: project(), assessment: assessment(25.5) });
    const file = JSON.parse(strFromU8(bundle.files['ep-online-gegevensoverzicht.json']));
    expect(file.schema).toContain('EpbdExportTypesV4');
    expect(bundle.manifest.files.some((entry) => entry.path === 'ep-online-gegevensoverzicht.json')).toBe(true);
  });
});
