import type { ProjectPerformanceAssessment } from './KernelClient';

/**
 * The one rule for what the simplified (indicative) engine may show next to the NTA kernel.
 * The results view, the BENG report (tab, HTML export, print, print preview) and the IFC
 * export all use it, so none of them can fill a refused kernel result with numbers the
 * kernel would not stand behind.
 *
 * - `calculated`: the kernel calculated the project; its figures are shown.
 * - `withheld`: the kernel refused the input (invalid or incomplete); every number is held back.
 * - `noNtaInputYet`: the project has no NTA input at all yet (only `nta_calculation_block_missing`),
 *   so the kernel has no verdict on the numbers; the indicative estimate may show.
 * - `unknown`: no settled kernel answer (not run, still running or unavailable).
 */
export type KernelVerdict = 'calculated' | 'withheld' | 'noNtaInputYet' | 'unknown';

export function kernelVerdict(assessment: ProjectPerformanceAssessment | null | undefined): KernelVerdict {
  if (!assessment) return 'unknown';
  if (assessment.status === 'calculated_unverified' && assessment.performance) return 'calculated';
  const gaps = assessment.gaps ?? [];
  if (assessment.status === 'incomplete' && gaps.length > 0
    && gaps.every((gap) => gap.code === 'nta_calculation_block_missing')) return 'noNtaInputYet';
  return 'withheld';
}

/** True when the kernel refused the input, so no simplified numbers may be shown or exported. */
export function kernelWithheld(assessment: ProjectPerformanceAssessment | null | undefined): boolean {
  return kernelVerdict(assessment) === 'withheld';
}

/** True when the simplified engine may show its (indicative) figures for this kernel answer. */
export function indicativeAllowed(assessment: ProjectPerformanceAssessment | null | undefined): boolean {
  const verdict = kernelVerdict(assessment);
  return verdict === 'noNtaInputYet' || verdict === 'unknown';
}
