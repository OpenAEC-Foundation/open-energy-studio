/**
 * A report names the edition the result was calculated in (BRL 9501
 * traceability): a calculation in NTA 8800:2022 never carries the label of
 * the designated edition on its cover, notice or dossier.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { generateEnergyPerformanceReportHTML } from '../core/report/EnergyPerformanceReport';
import { buildEpOnlineOverview } from '../core/report/EpOnlineOverview';
import { generateNtaInputDossierHTML } from '../core/report/NtaInputDossier';
import { generateReportHTML } from '../core/report/ReportTemplate';

const root = resolve(__dirname, '../..');
const project = JSON.parse(readFileSync(resolve(root, 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject;
const current = JSON.parse(readFileSync(resolve(root, 'training-data/nta8800-example-terraced-dwelling.kernel-output.json'), 'utf8')) as ProjectPerformanceAssessment;

function in2022(): ProjectPerformanceAssessment {
  const copy = JSON.parse(JSON.stringify(current)) as ProjectPerformanceAssessment;
  copy.status = 'calculated_legacy_edition';
  copy.registrationEligible = false;
  (copy as unknown as { normVersion: string }).normVersion = '2022';
  copy.targetNormVersion = 'NTA 8800:2022';
  return copy;
}

describe('reports name the calculated edition', () => {
  it('puts the edition of the result on the energy performance report cover', () => {
    const html = generateEnergyPerformanceReportHTML(project, in2022(), { level: 'summary', generatedAt: new Date('2026-10-09T10:00:00Z') });
    expect(html).toContain('<p class="sub">NTA 8800:2022 — ');
    expect(html).not.toContain('<p class="sub">NTA 8800:2025+C1:2026');
  });

  it('names the edition in the kernel notice of the BENG report', () => {
    const legacy = generateReportHTML(project, null, { kernel: in2022() });
    expect(legacy).toContain('Rust-rekenkern van Open Energy Studio (NTA 8800:2022)');
    const designated = generateReportHTML(project, null, { kernel: current });
    expect(designated).toContain('Rust-rekenkern van Open Energy Studio (NTA 8800:2025+C1:2026)');
  });

  it('gives the project edition as Doeluitgave in the input dossier', () => {
    const html = generateNtaInputDossierHTML({ ...project, ntaCalculation: { ...project.ntaCalculation!, normVersion: '2022' } });
    expect(html).toContain('<th>Doeluitgave</th><td>NTA 8800:2022</td>');
    expect(generateNtaInputDossierHTML(project)).toContain('NTA 8800:2025+C1:2026');
  });

  it('names the full edition label in the EP-Online overview', () => {
    const overview = buildEpOnlineOverview(project, in2022());
    expect(overview.notice).toContain('oudere uitgave NTA 8800:2022:');
  });
});
