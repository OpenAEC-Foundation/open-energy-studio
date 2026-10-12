/**
 * Results in an older NTA 8800 edition: the kernel stamps the nested
 * `performance` block with the legacy status too (norm_versions::request::stamp).
 * The report, the report model and the EP-Online overview must keep treating
 * such a result as calculated but not registrable.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { generateEnergyPerformanceReportHTML } from '../core/report/EnergyPerformanceReport';
import { buildEpOnlineOverview } from '../core/report/EpOnlineOverview';
import { kernelReportModel } from '../core/report/KernelReportModel';

const root = resolve(__dirname, '../..');
const project = JSON.parse(readFileSync(resolve(root, 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject;
const current = JSON.parse(readFileSync(resolve(root, 'training-data/nta8800-example-terraced-dwelling.kernel-output.json'), 'utf8')) as ProjectPerformanceAssessment;

function legacy(): ProjectPerformanceAssessment {
  const copy = JSON.parse(JSON.stringify(current)) as ProjectPerformanceAssessment & { performance: { status: string; registrationEligible?: boolean } };
  copy.status = 'calculated_legacy_edition';
  copy.registrationEligible = false;
  (copy as unknown as { normVersion: string }).normVersion = '2022';
  copy.targetNormVersion = 'NTA 8800:2022';
  copy.performance.status = 'calculated_legacy_edition';
  copy.performance.registrationEligible = false;
  return copy;
}

describe('legacy edition with a restamped nested performance block', () => {
  it('keeps the performance in the report model', () => {
    const model = kernelReportModel(legacy());
    expect(model).not.toBeNull();
    expect(model?.indicators.length ?? 0).toBeGreaterThan(0);
  });

  it('labels the report as an older edition', () => {
    const html = generateEnergyPerformanceReportHTML(project, legacy(), { level: 'standard', generatedAt: new Date('2026-10-10T10:00:00Z') });
    expect(html).toContain('berekend in oudere uitgave (niet voor registratie)');
  });

  it('leaves the energy class out of the EP-Online overview', () => {
    const overview = buildEpOnlineOverview(project, legacy());
    expect((overview.pandcertificaat as Record<string, unknown>).Energieklasse ?? null).toBeNull();
    expect(overview.omitted.some((entry) => entry.field === 'Energieklasse')).toBe(true);
  });
});
