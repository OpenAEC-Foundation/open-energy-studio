import type { ProjectPerformanceAssessment } from './KernelClient';

/**
 * Warnings of the building calculation that the project warnings do not carry yet. The kernel lifts each
 * `performance.warnings` entry into `warnings` at its project path, with `derivedInput.<path>` as detail
 * (review 53); lists that show both use this to show each warning once.
 */
export function buildingOnlyWarnings(assessment: Pick<ProjectPerformanceAssessment, 'warnings' | 'performance'> | null | undefined): Array<{ code: string; path: string }> {
  const lifted = new Set((assessment?.warnings ?? [])
    .filter((warning) => warning.detail?.startsWith('derivedInput.'))
    .map((warning) => `${warning.code} ${warning.detail}`));
  return (assessment?.performance?.warnings ?? [])
    .filter((warning) => !lifted.has(`${warning.code} derivedInput.${warning.path}`));
}
