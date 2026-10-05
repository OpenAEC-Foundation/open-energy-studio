import { describe, expect, it } from 'vitest';
import { createDefaultProject } from '../context/EnergyContext';
import { importTools, withImportRecord } from '../core/io/importLog';
import { labelInputSha256 } from '../core/nta/Registration';
import type { MaatwerkadviesAssessment, NtaMaatwerkadvies, ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import { generateMaatwerkadviesReportHTML } from '../core/report/MaatwerkadviesReport';

describe('BRL 9501 (29-05-2026) §4.3.1: data read in with another tool', () => {
  it('records each import and lists the distinct tools', () => {
    const project = createDefaultProject();
    const once = withImportRecord(project, 'UNIEC3', 'woning.uniec3', '2026-10-05T10:00:00.000Z');
    const twice = withImportRecord(withImportRecord(once, 'VABI'), 'UNIEC3');
    expect(once.importLog).toEqual([{ tool: 'UNIEC3', fileName: 'woning.uniec3', importedAt: '2026-10-05T10:00:00.000Z' }]);
    expect(importTools(twice)).toEqual(['UNIEC3', 'VABI']);
    expect(importTools(project)).toEqual([]);
    // The original project is not changed.
    expect(project.importLog).toBeUndefined();
  });

  it('keeps the import log out of the relabel label input', async () => {
    const project = createDefaultProject();
    const logged = withImportRecord(project, 'UNIEC3');
    expect(await labelInputSha256(logged)).toBe(await labelInputSha256(project));
  });

  it('prints the import declaration in the calculation report', () => {
    const project = { ...createDefaultProject(), registration: { client: 'X' } };
    const assessment = (dataImport: unknown) => ({
      status: 'incomplete', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
      inputFingerprint: 'sha256:feed', attestStatus: 'unattested', gaps: [], geometry: null,
      derivedInput: null, performance: null, labelData: null,
      registration: {
        source: 'BRL 9500', validUntil: null, registrationDeadline: null, relabelDeadline: null,
        readyForRegistration: false, issues: [], dataImport,
      },
    }) as unknown as ProjectPerformanceAssessment;
    const imported = generateNtaCalculationReportHTML(project,
      assessment({ source: 'BRL 9501 (29-05-2026) §4.3.1 opmerking, p. 8', dataImported: true, tools: ['UNIEC3', 'VABI'] }));
    expect(imported).toContain('Gegevens ingelezen (BRL 9501 §4.3.1)');
    expect(imported).toContain('ja, met UNIEC3, VABI');
    const none = generateNtaCalculationReportHTML(project,
      assessment({ source: 'x', dataImported: false, tools: [] }));
    expect(none).toContain('<th>Gegevens ingelezen (BRL 9501 §4.3.1)</th><td>nee</td>');
  });
});

describe('BRL 9500-MWA-W/U (24-03-2026): renovation passport and registration type', () => {
  it('prints the registration type and the utility scheme', () => {
    const project = createDefaultProject();
    const definition = {
      measures: [], packages: [],
      tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, electricityExportEurPerKwh: 0.05, sourceReference: 't' },
      economics: { discountRate: 0.03, energyPriceChange: 0, sourceReference: 'e' },
    } as unknown as NtaMaatwerkadvies;
    const step = {
      id: 'passport-step-1', name: 'Stap 1', kind: 'passport_step', measureIds: [], valid: true,
      label: { labelClass: 'B', needIndicatorKwhPerM2: 80, primaryFossilIndicatorKwhPerM2: 150, renewableSharePercent: 10, tojuliMaxK: null },
      actualUse: null, savings: null, investmentEur: 0, maintenanceEurPerYear: 0, simplePaybackYears: null,
      netPresentValueEur: null, horizonYears: 0, phasing: [], systemChecks: [], issues: [],
    };
    const assessment = {
      status: 'calculated_unverified', scope: 'x', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
      inputFingerprint: 'sha256:x', attestStatus: 'unattested',
      current: step, measures: [], packages: [], fitCheck: null,
      advice: { packageId: null, chosenBy: 'automatic_highest_npv', motivation: null, warnings: [], specialistNotes: [], notes: [] },
      renovationPassport: {
        scheme: 'U', source: 'BRL 9500-MWA-U', steps: [step, step, step],
        requirements: [{ code: 'renovation_standard_ep2', met: false, detail: null }],
        eligible: false, requiredStatements: [],
      },
      registrationType: 'maatwerkadvies',
      interpretations: [], issues: [],
    } as unknown as MaatwerkadviesAssessment;
    const html = generateMaatwerkadviesReportHTML(project, definition, assessment);
    expect(html).toContain('Registratie (BRL 9500-MWA §4.2.8):</strong> Maatwerkadvies');
    expect(html).toContain('Renovatiepaspoort (BRL 9500-MWA-U §3.2, ISSO 75.2)');
    expect(html).toContain('EP2 op of onder de renovatiestandaard (stap 3)');
    const withPassport = generateMaatwerkadviesReportHTML(project, definition,
      { ...assessment, registrationType: 'maatwerkadvies_met_renovatiepaspoort' });
    expect(withPassport).toContain('Maatwerkadvies met renovatiepaspoort');
  });
});
