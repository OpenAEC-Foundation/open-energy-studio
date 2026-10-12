/**
 * Output requirements of BRL 9500-W/U and BRL 9501 (docs/nta8800-uitvoereisen.md): every
 * output names the program apart from the rekenkern and states a missing attest number;
 * the energy performance report states the net heat need against the Standaard voor
 * Woningisolatie and always has the registration block; the input dossier carries the
 * survey date and the surveying adviser; the MWA report states the NCW period.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { version } from '../../package.json';
import type { IProject } from '../core/energy/types';
import type { MaatwerkadviesAssessment, ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { generateEnergyPerformanceReportHTML } from '../core/report/EnergyPerformanceReport';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import { generateNtaInputDossierHTML } from '../core/report/NtaInputDossier';
import { generateMaatwerkadviesReportHTML } from '../core/report/MaatwerkadviesReport';
import { buildEpOnlineOverview } from '../core/report/EpOnlineOverview';
import { buildProjectDossier } from '../core/report/ProjectDossier';
import { ATTEST_NOT_YET, attestText, programIdentity } from '../core/report/ProgramIdentity';

const root = resolve(__dirname, '../..');
const load = (name: string) => ({
  project: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.json`), 'utf8')) as IProject,
  assessment: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.kernel-output.json`), 'utf8')) as ProjectPerformanceAssessment,
});
const at = new Date('2026-10-10T10:00:00Z');

describe('program identity on every output', () => {
  it('keeps program, rekenkern and attest apart and states a missing attest number', () => {
    const identity = programIdentity('0.4.0', 'NTA 8800:2025+C1:2026', null);
    expect(identity).toEqual({
      name: 'Open Energy Studio', programVersion: version, kernelVersion: '0.4.0',
      normEdition: 'NTA 8800:2025+C1:2026', attestNumber: null,
    });
    expect(attestText(identity)).toBe(ATTEST_NOT_YET);
    expect(attestText(programIdentity('0.4.0', null, ' K-2026-01 '))).toBe('K-2026-01');
  });

  it('puts the program version and the attest status on the energy performance report', () => {
    const { project, assessment } = load('terraced-dwelling');
    const html = generateEnergyPerformanceReportHTML(project, assessment, { level: 'summary', generatedAt: at });
    expect(html).toContain(`Open Energy Studio ${version}, rekenkern ${assessment.kernelVersion} (niet geattesteerd)`);
    expect(html).toContain('<dt>BRL 9501-attestnummer</dt><dd>nog niet toegekend</dd>');
    expect(html).toContain('<th>BRL 9501-attestnummer</th><td>nog niet toegekend</td>');
    const attested = generateEnergyPerformanceReportHTML(project, assessment, { level: 'summary', generatedAt: at, attestNumber: 'K-2026-01' });
    expect(attested).toContain('<dt>BRL 9501-attestnummer</dt><dd>K-2026-01</dd>');
    expect(attested).not.toContain(ATTEST_NOT_YET);
  });

  it('puts the program and attest status on the calculation report, in Dutch', () => {
    const { project, assessment } = load('office');
    const html = generateNtaCalculationReportHTML(project, assessment, [], at);
    expect(html).toContain(`<th>Rekenprogramma</th><td>Open Energy Studio ${version}</td>`);
    expect(html).toContain('<th>BRL 9501-attestnummer</th><td>nog niet toegekend</td>');
    expect(html).toContain('<th>Atteststatus</th><td>niet geattesteerd (BRL 9501)</td>');
    expect(html).not.toMatch(/<td>unattested<\/td>/);
  });

  it('records the program in the dossier manifest and the EP-Online overview', async () => {
    const { project, assessment } = load('terraced-dwelling');
    const { manifest } = await buildProjectDossier({ project, assessment, generatedAt: at.toISOString() });
    expect(manifest.software).toEqual({
      name: 'Open Energy Studio', programVersion: version, kernelVersion: assessment.kernelVersion,
      normEdition: assessment.targetNormVersion, attestNumber: null, attestNumberText: ATTEST_NOT_YET,
    });
    const overview = buildEpOnlineOverview(project, assessment);
    expect(overview.berekening).toEqual({
      rekenprogramma: 'Open Energy Studio', programmaversie: version, rekenkernversie: assessment.kernelVersion,
      uitgave: assessment.targetNormVersion, invoervingerafdruk: assessment.inputFingerprint,
      attestnummer: null, attestnummerTekst: ATTEST_NOT_YET,
    });
  });
});

describe('energy performance report (BRL 9500-W, p. 7; §4.2, p. 17–18)', () => {
  it('states the net heat need against the Standaard voor Woningisolatie at every level for a dwelling', () => {
    const { project, assessment } = load('terraced-dwelling');
    for (const level of ['summary', 'standard', 'detailed'] as const) {
      const html = generateEnergyPerformanceReportHTML(project, assessment, { level, generatedAt: at, interpretations: [] });
      expect(html).toContain('Nettowarmtebehoefte E<sub>H;nd</sub> (5.3a)');
      expect(html).toContain('Standaard voor Woningisolatie (NTA 8800 §5.3.2)');
      expect(html).toMatch(/grens 69 kWh\/m²·jr — nettowarmtebehoefte (binnen|boven) de standaard/);
    }
  });

  it('leaves the Standaard voor Woningisolatie out for a utility building', () => {
    const { project, assessment } = load('office');
    const html = generateEnergyPerformanceReportHTML(project, assessment, { level: 'summary', generatedAt: at });
    expect(html).not.toContain('Standaard voor Woningisolatie (NTA 8800 §5.3.2)');
  });

  it('always has the registration block, naming what is missing without registration data', () => {
    const { project, assessment } = load('office');
    const html = generateEnergyPerformanceReportHTML({ ...project, registration: undefined }, assessment, { level: 'summary', generatedAt: at });
    expect(html).toContain('<h3>Registratie</h3>');
    expect(html).toContain('opnemend en registrerend adviseur met hun vakbekwaamheidsnummer');
  });
});

describe('input dossier (BRL 9500-W bijlage 3, p. 62)', () => {
  it('carries the survey date and the surveying adviser, or says they are not filled in', () => {
    const { project } = load('terraced-dwelling');
    const empty = generateNtaInputDossierHTML({ ...project, registration: undefined }, at);
    expect(empty).toContain('<th>Opnamedatum</th><td>niet ingevuld</td>');
    expect(empty).toContain('<th>Opnemend adviseur</th><td>niet ingevuld</td>');
    expect(empty).toContain('<th>BRL 9501-attestnummer</th><td>nog niet toegekend</td>');
    const filled = generateNtaInputDossierHTML({
      ...project,
      registration: {
        ...(project.registration ?? {}),
        surveyDate: '2026-10-01',
        surveyingAdvisor: { name: 'A. Adviseur', competenceNumber: 'EP-123' },
      } as IProject['registration'],
    }, at);
    expect(filled).toContain('<th>Opnamedatum</th><td>2026-10-01</td>');
    expect(filled).toContain('A. Adviseur (vakbekwaamheid EP-123)');
  });
});

describe('maatwerkadvies report (BRL 9501 §4.1.3, p. 7)', () => {
  const variant = (id: string, horizon: number) => ({
    id, name: id, kind: 'package' as const, measureIds: [], valid: true,
    label: { labelClass: 'C', needIndicatorKwhPerM2: null, primaryFossilIndicatorKwhPerM2: 200, renewableSharePercent: 0, tojuliMaxK: null },
    actualUse: null, savings: null, investmentEur: 0, maintenanceEurPerYear: 0, simplePaybackYears: null,
    netPresentValueEur: null, horizonYears: horizon, phasing: [], systemChecks: [], issues: [],
  });
  const assessment = {
    status: 'calculated_unverified', scope: 's', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.4.0',
    inputFingerprint: 'f', attestStatus: 'unattested', current: variant('current', 30), measures: [],
    packages: [variant('p1', 30)], fitCheck: null, advice: null, renovationPassport: null, interpretations: [], issues: [],
  } as unknown as MaatwerkadviesAssessment;
  const definition = {
    measures: [], packages: [],
    tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, sourceReference: 'tarieven 2026' },
  };

  it('states the NCW period with the other financial inputs, and the program', () => {
    const project = load('terraced-dwelling').project;
    const chosen = generateMaatwerkadviesReportHTML(project, definition, assessment);
    expect(chosen).toContain('<th>Looptijd netto contante waarde</th><td>30 jaar (langste levensduur van de maatregelen)</td>');
    expect(chosen).toContain(`<th>Rekenprogramma</th><td>Open Energy Studio ${version}</td>`);
    expect(chosen).toContain('<th>BRL 9501-attestnummer</th><td>nog niet toegekend</td>');
    const given = generateMaatwerkadviesReportHTML(project, {
      ...definition, economics: { horizonYears: 15, maintenancePriceChange: 0.02, baseYear: 2027 },
    }, assessment);
    expect(given).toContain('<td>15 jaar (opgegeven)</td>');
    expect(given).toContain('<td>2,0 % / 2027</td>');
  });
});
