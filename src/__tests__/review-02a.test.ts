import { describe, expect, it } from 'vitest';
import { createDefaultProject } from '../context/EnergyContext';
import { generateReportHTML } from '../core/report/ReportTemplate';
import { indicatorDecimals } from '../core/report/KernelReportModel';
import { dutchTimeHtml } from '../core/report/DutchReportText';
import { generateNtaInputDossierHTML } from '../core/report/NtaInputDossier';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';

function assessment(values: [number, number, number], limits: [number, number, number]): ProjectPerformanceAssessment {
  return {
    status: 'calculated_unverified', gaps: [], warnings: [],
    derivedInput: { calculationScope: 'utility' }, geometry: { usableFloorAreaM2: 1000 },
    performance: {
      status: 'calculated_unverified', issues: [], warnings: [],
      needIndicatorKwhPerM2Year: values[0], primaryFossilIndicatorKwhPerM2Year: values[1], renewableSharePercent: values[2],
      indicativeLabelClass: 'A', tojuli: [], pvSystems: [],
      bblCheck: { source: 'Bbl', function: 'office', lossAreaRatio: 2.137,
        limits: { energyNeedMaxKwhPerM2: limits[0], primaryFossilMaxKwhPerM2: limits[1], renewableShareMinPercent: limits[2],
          lightConstructionAllowanceApplied: false },
        energyNeedMeets: true, primaryFossilMeets: true, renewableShareMeets: true },
      spaceHeating: { omittedTerms: [], additionalZoneDemands: [], monthly: [],
        demand: { annualHeatingNeedKwh: 0, annualCoolingNeedKwh: 0, omittedCorrections: [], monthly: [] } },
    },
  } as unknown as ProjectPerformanceAssessment;
}

describe('BENG report rounding', () => {
  it('prints each limit with the precision of its value, so a row never contradicts its toets', () => {
    expect([indicatorDecimals('beng1'), indicatorDecimals('beng2'), indicatorDecimals('beng3'), indicatorDecimals('tojuli')])
      .toEqual([2, 2, 1, 2]);
    // A_ls/A_g 2,137: BENG 1 limit 74,11; a weighted BENG 3 limit of 32,5 against 32,6.
    const html = generateReportHTML(createDefaultProject(), null, {
      kernel: assessment([74.11, 39.5, 32.6], [74.11, 40, 32.5]), locale: 'nl' });
    expect(html).toContain('≤ 74,11');
    expect(html).not.toMatch(/≤ 74,1</);
    expect(html).toContain('≥ 32,5');
    expect(html).not.toContain('≥ 33');
    expect(html).toContain('≤ 40,00');
  });
});

describe('document timestamps', () => {
  it('carries the exact ISO time next to the Dutch text', () => {
    const date = new Date('2026-10-03T12:34:56.789Z');
    const time = dutchTimeHtml(date);
    expect(time).toContain('datetime="2026-10-03T12:34:56.789Z"');
    expect(time).toMatch(/oktober 2026/);
    const dossier = generateNtaInputDossierHTML(createDefaultProject(), date);
    expect(dossier).toContain('<time datetime="2026-10-03T12:34:56.789Z">');
  });
});
