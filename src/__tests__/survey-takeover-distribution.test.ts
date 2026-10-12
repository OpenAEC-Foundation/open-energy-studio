import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import type { BuildingPerformanceInput } from '../core/nta/KernelClient';
import { surveyTakeover } from '../core/nta/SurveyTakeover';

// Found by the real-kernel app run: the 1930 survey derives a calculated
// distribution (9.4) with its distributionSystem; the takeover left the system
// out, and the kernel refused the taken-over project (distribution_system_required).

const output = JSON.parse(readFileSync(resolve(process.cwd(), 'training-data/nta8800-opname-1930-terraced.kernel-output.json'), 'utf8')) as {
  derivedInput: BuildingPerformanceInput & { spaceHeating: { distribution: { method: string }; distributionSystem: unknown } };
};

describe('survey takeover of the heating distribution', () => {
  it('takes the distribution system over with the calculated distribution', () => {
    const derived = output.derivedInput;
    expect(derived.spaceHeating.distribution.method).toBe('calculated');
    const { next, keys } = surveyTakeover(derived, {});
    expect(keys).toContain('distributionSystem');
    expect(next.distributionSystem).toEqual(derived.spaceHeating.distributionSystem);
  });

  it('leaves the draft\'s system alone when the survey derives none', () => {
    const derived = structuredClone(output.derivedInput) as unknown as { spaceHeating: Record<string, unknown> };
    delete derived.spaceHeating.distributionSystem;
    const { next, keys } = surveyTakeover(derived as unknown as BuildingPerformanceInput, { distributionSystem: { installation: 'individual' } });
    expect(keys).not.toContain('distributionSystem');
    expect(next.distributionSystem).toEqual({ installation: 'individual' });
  });
});
