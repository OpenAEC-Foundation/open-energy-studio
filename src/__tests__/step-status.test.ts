/**
 * Step status per workflow step (UI redesign F3, ontwerp §3.4), checked on
 * both example projects with their stored kernel output.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { invalidBlockPath, isNewBuild, kernelIssues, stepStatuses } from '../core/nta/stepStatus';
import { WORKFLOW_STEPS } from '../core/navigation/routes';

const root = resolve(__dirname, '../..');
const load = (name: string) => ({
  project: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.json`), 'utf8')) as IProject,
  assessment: JSON.parse(readFileSync(resolve(root, `training-data/nta8800-example-${name}.kernel-output.json`), 'utf8')) as ProjectPerformanceAssessment,
});

describe('stepStatuses on the example projects', () => {
  for (const name of ['terraced-dwelling', 'office'] as const) {
    it(`marks the input and calculation steps complete for the ${name} example`, () => {
      const { project, assessment } = load(name);
      const statuses = stepStatuses(project, assessment);
      for (const step of WORKFLOW_STEPS) expect(statuses[step.id]).toBeDefined();
      expect(statuses.project.state).toBe('complete');
      expect(statuses.building.state).toBe('complete');
      expect(statuses.installations.state).toBe('complete');
      expect(statuses.check.state).toBe('complete');
      expect(statuses.results.state).toBe('complete');
      expect(statuses.report.state).toBe('complete');
      // Not registered yet: the last step is still to do.
      expect(statuses.registration.state).toBe('todo');
      for (const step of WORKFLOW_STEPS) expect(statuses[step.id].errors).toBe(0);
    });

    it(`routes kernel gaps and warnings of the ${name} example to their step`, () => {
      const { project, assessment } = load(name);
      const broken: ProjectPerformanceAssessment = {
        ...assessment,
        status: 'incomplete',
        performance: null,
        gaps: [
          { code: 'missing_g_value', path: 'zones[0].surfaces[0].windows[0].gValue' },
          { code: 'missing_generator', path: 'ntaCalculation.generator.kind' },
          { code: 'missing_client', path: 'registration.client' },
        ],
        warnings: [{ code: 'heat_pump_declaration_missing', path: 'ntaHeatPumps[0].declaration' }],
      };
      const statuses = stepStatuses(project, broken);
      expect(statuses.building).toMatchObject({ state: 'errors', errors: 1, warnings: 0 });
      // Since F6 an NTA input path counts for the step that edits it (generator: Installaties).
      expect(statuses.check).toMatchObject({ errors: 0, warnings: 0 });
      expect(statuses.registration).toMatchObject({ state: 'errors', errors: 1 });
      expect(statuses.installations).toMatchObject({ state: 'errors', errors: 1, warnings: 1 });
      expect(statuses.installations.issues.map((issue) => issue.path)).toEqual(['ntaCalculation.generator.kind', 'ntaHeatPumps[0].declaration']);
      // Withheld: no result, no report.
      expect(statuses.results.state).toBe('todo');
      expect(statuses.report.state).toBe('todo');
      expect(kernelIssues(broken).map((issue) => issue.kind)).toEqual(['error', 'error', 'error', 'warning']);
    });
  }

  it('dims the existing-building steps on new build only', () => {
    const { project, assessment } = load('terraced-dwelling');
    const delivery = { ...project, registration: { ...project.registration, purpose: 'delivery' as const } };
    const existing = { ...project, registration: { ...project.registration, purpose: 'existing_building' as const } };
    expect(isNewBuild(delivery)).toBe(true);
    expect(isNewBuild(existing)).toBe(false);
    const dimmed = stepStatuses(delivery, assessment);
    for (const step of ['survey', 'advice', 'relabel'] as const) {
      expect(dimmed[step]).toMatchObject({ dimmed: true, state: 'dimmed' });
      expect(stepStatuses(existing, assessment)[step].dimmed).toBe(false);
    }
    expect(dimmed.building.dimmed).toBe(false);
  });

  it('keeps a dimmed step visible with its errors', () => {
    const { project, assessment } = load('terraced-dwelling');
    const delivery = { ...project, registration: { purpose: 'delivery' as const } };
    const statuses = stepStatuses(delivery, { ...assessment, gaps: [{ code: 'x', path: 'basisopname.zones[0]' }] });
    expect(statuses.survey).toMatchObject({ dimmed: true, state: 'errors', errors: 1 });
  });

  it('has no kernel status without an assessment', () => {
    const { project } = load('office');
    const statuses = stepStatuses(project, null);
    expect(statuses.check.state).toBe('todo');
    expect(statuses.results.state).toBe('todo');
    expect(statuses.building.state).toBe('complete');
  });
});

describe('invalid NTA block', () => {
  it('locates the field from the serde path in the detail ("Ga naar")', () => {
    expect(invalidBlockPath('nta_calculation_block_invalid', 'ntaCalculation', 'setpoints: missing field `heatingC`'))
      .toBe('ntaCalculation.setpoints.heatingC');
    expect(invalidBlockPath('nta_calculation_block_invalid', 'ntaCalculation', '.: missing field `thermalMass`'))
      .toBe('ntaCalculation.thermalMass');
    expect(invalidBlockPath('nta_calculation_block_invalid', 'ntaCalculation', 'ventilationFlows[0].months[3].conductanceWPerK: invalid type: null'))
      .toBe('ntaCalculation.ventilationFlows[0].months[3].conductanceWPerK');
    expect(invalidBlockPath('nta_calculation_block_invalid', 'ntaCalculation', 'invalid type: map')).toBe('ntaCalculation');
    expect(invalidBlockPath('missing_generator', 'ntaCalculation.generator', 'generator: x')).toBe('ntaCalculation.generator');
  });
});
