/**
 * Woningpas in the maatwerkadvies of a survey project (feedback 8 Oct 2026):
 * the renovation passport goes to the kernel with the building base, its
 * measure lists holding known measures only.
 */
import { describe, expect, it, vi } from 'vitest';
import * as kernel from '../core/nta/KernelClient';
import { assessSurveyMaatwerkadvies } from '../core/mwa/surveyMeasures';
import type { NtaMaatwerkadvies } from '../core/nta/KernelClient';

vi.mock('../core/nta/KernelClient', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../core/nta/KernelClient')>()),
  assessResidentialSurveyWithRust: vi.fn(async () => ({ issues: [], appliedDefaults: [], derivedInput: { zones: [] }, performance: null })),
  assessMaatwerkadviesInputWithRust: vi.fn(async () => ({ issues: [] })),
}));

describe('survey maatwerkadvies with a renovation passport', () => {
  it('sends the passport with known measures only', async () => {
    const definition: NtaMaatwerkadvies = {
      measures: [{ id: 'm1', name: 'Dak', category: 'insulation', target: 'building', patch: [], investmentEur: 1000, costSource: 'offerte', lifetimeYears: 40,
        template: { source: 'survey', change: { kind: 'roof', surfaceIds: [], thicknessMm: 200 } } as unknown as NonNullable<kernel.MwaMeasure['template']> }],
      packages: [{ id: 'p1', name: 'Schil', measureIds: ['m1'] }],
      tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, sourceReference: 'x' },
      renovationPassport: { demandPackageId: 'p1', systemsPackageId: 'p2', productionPackageId: 'p3', overheatingMeasureIds: ['m1', 'gone'], storageConsidered: true },
    };
    await assessSurveyMaatwerkadvies({ kind: 'residential', survey: { envelope: { surfaces: [] } } }, definition);
    const input = vi.mocked(kernel.assessMaatwerkadviesInputWithRust).mock.calls[0][0] as Record<string, unknown>;
    expect(input.base).toMatchObject({ kind: 'building' });
    expect(input.renovationPassport).toEqual({
      demandPackageId: 'p1', systemsPackageId: 'p2', productionPackageId: 'p3', overheatingMeasureIds: ['m1'], storageConsidered: true,
    });
  });
});
