import { describe, expect, it } from 'vitest';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { DETAIL_SECTIONS, generateEnergyPerformanceReportHTML } from '../core/report/EnergyPerformanceReport';
import { dutchNumber } from '../core/report/DutchReportText';

const root = resolve(__dirname, '../..');
const examples = ['terraced-dwelling', 'office'] as const;
const load = (name: string) => ({
  project: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.json`), 'utf8')) as IProject,
  assessment: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.kernel-output.json`), 'utf8')) as ProjectPerformanceAssessment,
});
const at = new Date('2026-10-05T10:00:00Z');

describe('Rapportage Energieprestatie (NTA 8800)', () => {
  for (const name of examples) {
    it(`renders every level for the ${name} example`, () => {
      const { project, assessment } = load(name);
      const performance = assessment.performance!;
      const summary = generateEnergyPerformanceReportHTML(project, assessment, { level: 'summary', generatedAt: at });
      const standard = generateEnergyPerformanceReportHTML(project, assessment, { level: 'standard', generatedAt: at });
      const detailed = generateEnergyPerformanceReportHTML(project, assessment, { level: 'detailed', generatedAt: at, interpretations: [] });
      if (process.env.WRITE_REPORT) {
        for (const [level, html] of [['summary', summary], ['standard', standard], ['detailed', detailed]]) {
          writeFileSync(resolve(root, `.rep/report-${name}-${level}.html`), html);
        }
      }
      // Summary: results with the kernel's own values and limits.
      for (const html of [summary, standard, detailed]) {
        expect(html).toContain('Rapportage Energieprestatie');
        expect(html).toContain(dutchNumber(performance.primaryFossilIndicatorKwhPerM2Year, 2));
        expect(html).toContain(dutchNumber(performance.bblCheck!.limits.primaryFossilMaxKwhPerM2, 2));
        expect(html).toContain(assessment.inputFingerprint);
        expect(html).toContain('Onverifieerde berekening');
      }
      expect(summary).not.toContain('Bouwkundige uitgangspunten');
      expect(summary).not.toContain('Invoeroverzicht');
      // Standard: building, envelope, installations, energy and the input overview.
      for (const title of ['Gebouw en rekenzones', 'Bouwkundige uitgangspunten', 'Installatietechnische uitgangspunten',
        'Energiegebruik per functie en drager', 'Invoeroverzicht']) {
        expect(standard).toContain(title);
      }
      expect(standard).not.toContain('Berekening: Warmte- en koudebalans');
      // Detailed: monthly kernel values with their formula references.
      const zone = performance.spaceHeating.demand;
      expect(detailed).toContain('Berekening: Warmte- en koudebalans per rekenzone');
      expect(detailed).toContain(dutchNumber(zone.monthly[0].heating.needKwh, 0));
      expect(detailed).toContain(dutchNumber(zone.monthly[0].heating.utilization, 3));
      expect(detailed).toContain(dutchNumber(zone.transmission!.directConductanceWPerK, 2));
      expect(detailed).toContain('7.46–7.49');
      expect(detailed).toContain('formula');
      // Every detail chapter that has data appears in the table of contents.
      expect(detailed).toContain('Berekening: Transmissie');
      expect(detailed).toContain('Berekening: Primaire energie');
    });
  }

  it('omits unselected detail chapters', () => {
    const { project, assessment } = load('terraced-dwelling');
    const details = Object.fromEntries(DETAIL_SECTIONS.map((section) => [section, section === 'balance']));
    const html = generateEnergyPerformanceReportHTML(project, assessment, { level: 'detailed', details, generatedAt: at });
    expect(html).toContain('Berekening: Warmte- en koudebalans');
    expect(html).not.toContain('Berekening: Transmissie');
    expect(html).not.toContain('Berekening: Verwarmingsketen');
  });

  it('per-window solar gains add up to the zone total', () => {
    const { project, assessment } = load('office');
    const month = assessment.performance!.spaceHeating.demand.monthly[0];
    const total = (month.windowSolarByWindow ?? []).reduce((sum, window) => sum + window.heatingKwh, 0);
    expect(total).toBeCloseTo(month.windowSolarGainsKwh, 6);
    const html = generateEnergyPerformanceReportHTML(project, assessment, { level: 'detailed', generatedAt: at });
    expect(html).toContain('Zonnewinst per raam en maand');
  });

  it('withholds every result when the kernel refused the input', () => {
    const { project, assessment } = load('terraced-dwelling');
    const refused: ProjectPerformanceAssessment = {
      ...assessment, status: 'invalid', performance: null,
      gaps: [{ code: 'construction_u_value_missing', path: 'constructions[0]', detail: null }],
    } as unknown as ProjectPerformanceAssessment;
    const html = generateEnergyPerformanceReportHTML(project, refused, { level: 'detailed', generatedAt: at });
    expect(html).toContain('Resultaten achtergehouden');
    expect(html).toContain('construction_u_value_missing');
    expect(html).not.toContain(dutchNumber(assessment.performance!.primaryFossilIndicatorKwhPerM2Year, 2));
    expect(html).not.toContain('Berekening:');
  });
});
