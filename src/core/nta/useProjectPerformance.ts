import { useEffect, useState } from 'react';
import type { IProject } from '../energy/types';
import { calculateProjectPerformanceWithRust, projectCalculated, type ProjectPerformanceAssessment } from './KernelClient';

export type ProjectPerformanceQuery =
  | { project: IProject; kind: 'loading' }
  | { project: IProject; kind: 'done'; assessment: ProjectPerformanceAssessment }
  | { project: IProject; kind: 'error'; error: string };

/** Wait after an edit before the kernel runs, like the live preview, so typing does not start a run per key. */
export const KERNEL_RUN_DEBOUNCE_MS = 400;

/**
 * In-flight kernel runs per project state: the results view, the report, the print preview
 * and the live preview start their run for the same project object within the same debounce
 * window and share it. A settled run is forgotten, so a later request asks the kernel again.
 */
const runs = new WeakMap<IProject, Promise<ProjectPerformanceAssessment>>();

export function calculateProjectPerformanceShared(project: IProject): Promise<ProjectPerformanceAssessment> {
  const existing = runs.get(project);
  if (existing) return existing;
  const run = calculateProjectPerformanceWithRust(project);
  runs.set(project, run);
  const forget = () => { if (runs.get(project) === run) runs.delete(project); };
  run.then(forget, forget);
  return run;
}

/**
 * Runs the Rust project route for `project` (debounced) and keeps the latest answer.
 * Pass `enabled = false` when a parent already supplies the query.
 */
export function useProjectPerformance(project: IProject, enabled = true): ProjectPerformanceQuery | null {
  const [query, setQuery] = useState<ProjectPerformanceQuery | null>(null);

  useEffect(() => {
    if (!enabled) return undefined;
    let cancelled = false;
    setQuery({ project, kind: 'loading' });
    const timer = setTimeout(() => {
      calculateProjectPerformanceShared(project).then(
        (value) => { if (!cancelled) setQuery({ project, kind: 'done', assessment: value }); },
        (reason: unknown) => {
          if (!cancelled) {
            setQuery({ project, kind: 'error', error: reason instanceof Error ? reason.message : String(reason) });
          }
        },
      );
    }, runs.has(project) ? 0 : KERNEL_RUN_DEBOUNCE_MS);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [project, enabled]);

  return enabled && query?.project === project ? query : null;
}

/** The calculated kernel assessment of a query, or null while loading, failed, incomplete or invalid. */
export function calculatedAssessment(query: ProjectPerformanceQuery | null): ProjectPerformanceAssessment | null {
  if (query?.kind !== 'done') return null;
  const { assessment } = query;
  return projectCalculated(assessment.status) && assessment.performance ? assessment : null;
}
