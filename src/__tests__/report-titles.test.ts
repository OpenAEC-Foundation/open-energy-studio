/**
 * Report fixes of 5 October 2026: subscripts in chapter titles and the table of
 * contents, and the translated Bbl use function under table 3.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { generateEnergyPerformanceReportHTML, titleHtml } from '../core/report/EnergyPerformanceReport';

const root = resolve(__dirname, '../..');
const load = (name: string) => ({
  project: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.json`), 'utf8')) as IProject,
  assessment: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.kernel-output.json`), 'utf8')) as ProjectPerformanceAssessment,
});
const at = new Date('2026-10-05T10:00:00Z');

describe('report titles', () => {
  it('renders symbol subscripts and escapes the rest', () => {
    expect(titleHtml('H_D, H_g, H_U en H_tr')).toBe('H<sub>D</sub>, H<sub>g</sub>, H<sub>U</sub> en H<sub>tr</sub>');
    expect(titleHtml('Q_H;nd & Q_C;nd')).toBe('Q<sub>H;nd</sub> &amp; Q<sub>C;nd</sub>');
    expect(titleHtml('Gebouw <schil>')).toBe('Gebouw &lt;schil&gt;');
    expect(titleHtml('snake_case_word')).toBe('snake_case_word');
  });

  for (const name of ['terraced-dwelling', 'office'] as const) {
    it(`shows no raw underscores in the headings and contents of the ${name} report`, () => {
      const { project, assessment } = load(name);
      const html = generateEnergyPerformanceReportHTML(project, assessment, { level: 'detailed', generatedAt: at, interpretations: [] });
      const toc = html.match(/<nav[^>]*>[\s\S]*?<\/nav>/)?.[0] ?? '';
      const headings = [...html.matchAll(/<h2>([\s\S]*?)<\/h2>/g)].map((match) => match[1]);
      expect(headings.length).toBeGreaterThan(3);
      for (const text of [toc, ...headings]) expect(text).not.toMatch(/\b[A-Za-z]_[A-Za-z0-9]/);
      if (html.includes('H_tr') || toc.includes('H<sub>tr</sub>')) expect(toc).toContain('H<sub>tr</sub>');
    });
  }

  it('names the Bbl use function in Dutch under table 3', () => {
    const { project, assessment } = load('terraced-dwelling');
    expect(assessment.performance?.bblCheck?.function).toBe('other_residential');
    const html = generateEnergyPerformanceReportHTML(project, assessment, { level: 'standard', generatedAt: at });
    expect(html).not.toContain('other_residential');
    expect(html).toContain('Gebruiksfunctie woonfunctie (niet in een woongebouw)');
    const office = load('office');
    const officeHtml = generateEnergyPerformanceReportHTML(office.project, office.assessment, { level: 'standard', generatedAt: at });
    expect(officeHtml).toContain('Gebruiksfunctie kantoorfunctie');
  });
});
