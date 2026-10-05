/**
 * Status per workflow step, driven by the kernel (ontwerp §3.4).
 *
 * Errors are the kernel gaps (incomplete) or refusals (invalid), warnings the
 * plausibility findings; each is routed to a step through its path. A step is
 * complete when it has input and no errors. Steps for existing buildings stay
 * visible but are dimmed on new-build projects (besluit 5 oktober 2026).
 */
import type { IProject } from '../energy/types';
import { projectCalculated, type ProjectPerformanceAssessment } from './KernelClient';
import { routeForPath } from './gapRoutes';
import { WORKFLOW_STEPS, type StepId } from '../navigation/routes';

export type StepState = 'complete' | 'errors' | 'warnings' | 'todo' | 'dimmed';

export interface StepIssue {
  kind: 'error' | 'warning';
  code: string;
  path: string;
  detail?: string;
}

export interface StepStatus {
  step: StepId;
  errors: number;
  warnings: number;
  /** The step has input of its own. */
  hasInput: boolean;
  /** Not relevant for this project (existing-building steps on new build). */
  dimmed: boolean;
  state: StepState;
  issues: StepIssue[];
}

/**
 * The field behind an invalid NTA block. The kernel reports a block it cannot
 * read at `ntaCalculation`, with the serde path in the detail
 * (`setpoints: missing field \`heatingC\``, `ventilationFlows[0].months[3].conductanceWPerK: invalid type: null`);
 * that path is where "Ga naar" should go.
 */
export function invalidBlockPath(code: string, path: string, detail: string | null | undefined): string {
  if (code !== 'nta_calculation_block_invalid' || path !== 'ntaCalculation' || !detail) return path;
  const missing = /^(?:\.|([A-Za-z_][\w.[\]]*)): missing field `(\w+)`/.exec(detail);
  if (missing) return `ntaCalculation.${missing[1] ? `${missing[1]}.` : ''}${missing[2]}`;
  const located = /^([A-Za-z_][\w.[\]]*): /.exec(detail);
  return located ? `ntaCalculation.${located[1]}` : path;
}

/** Kernel findings with their kind; gaps are errors whatever the status (both block a result). */
export function kernelIssues(assessment: ProjectPerformanceAssessment | null | undefined): StepIssue[] {
  if (!assessment) return [];
  return [
    ...(assessment.gaps ?? []).map((gap) => ({
      kind: 'error' as const, code: gap.code, path: invalidBlockPath(gap.code, gap.path, gap.detail), ...(gap.detail ? { detail: gap.detail } : {}),
    })),
    ...(assessment.warnings ?? []).map((warning) => ({ kind: 'warning' as const, code: warning.code, path: warning.path, ...(warning.detail ? { detail: warning.detail } : {}) })),
  ];
}

/** True for new build (delivery or Bbl check): the existing-building steps are dimmed. */
export function isNewBuild(project: IProject): boolean {
  const purpose = project.registration?.purpose;
  return purpose === 'delivery' || purpose === 'bbl_check';
}

function hasInput(step: StepId, project: IProject, assessment: ProjectPerformanceAssessment | null | undefined): boolean {
  switch (step) {
    case 'project': return Boolean(project.name?.trim()) && Boolean(project.buildingFunction);
    case 'building': return project.zones.some((zone) => zone.surfaces.length > 0);
    case 'installations': return project.heatingSystems.length + project.ventilationSystems.length
      + project.hotWaterSystems.length > 0 || Boolean(project.ntaCalculation);
    case 'check': return assessment != null;
    case 'results': return projectCalculated(assessment?.status) && assessment?.performance != null;
    case 'survey': return project.basisopname != null;
    case 'advice': return project.maatwerkadvies != null;
    case 'relabel': return project.registration?.relabelComparison != null;
    case 'report': return projectCalculated(assessment?.status);
    case 'registration': return Boolean(project.registration?.epOnlineNumber);
    default: return false;
  }
}

const EXISTING: StepId[] = ['survey', 'advice', 'relabel'];

/** Status of every workflow step for the given project and kernel answer. */
export function stepStatuses(project: IProject, assessment: ProjectPerformanceAssessment | null | undefined): Record<StepId, StepStatus> {
  const issues = kernelIssues(assessment);
  const byStep = new Map<StepId, StepIssue[]>();
  for (const issue of issues) {
    const { step } = routeForPath(issue.path);
    byStep.set(step, [...(byStep.get(step) ?? []), issue]);
  }
  const newBuild = isNewBuild(project);
  const result = {} as Record<StepId, StepStatus>;
  for (const definition of WORKFLOW_STEPS) {
    const own = byStep.get(definition.id) ?? [];
    const errors = own.filter((issue) => issue.kind === 'error').length;
    const warnings = own.length - errors;
    const input = hasInput(definition.id, project, assessment);
    const dimmed = newBuild && EXISTING.includes(definition.id);
    const state: StepState = errors > 0 ? 'errors'
      : warnings > 0 ? 'warnings'
        : dimmed ? 'dimmed'
          : input ? 'complete' : 'todo';
    result[definition.id] = { step: definition.id, errors, warnings, hasInput: input, dimmed, state, issues: own };
  }
  result.tool = { step: 'tool', errors: 0, warnings: 0, hasInput: false, dimmed: false, state: 'todo', issues: [] };
  return result;
}
