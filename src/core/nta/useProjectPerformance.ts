import { useEffect, useState } from 'react';
import type { IProject } from '../energy/types';
import { calculateProjectPerformanceWithRust, type ProjectPerformanceAssessment } from './KernelClient';

export type ProjectPerformanceQuery =
  | { project: IProject; kind: 'loading' }
  | { project: IProject; kind: 'done'; assessment: ProjectPerformanceAssessment }
  | { project: IProject; kind: 'error'; error: string };

/**
 * Runs the Rust project route for `project` and keeps the latest answer.
 * Pass `enabled = false` when a parent already supplies the query.
 */
export function useProjectPerformance(project: IProject, enabled = true): ProjectPerformanceQuery | null {
  const [query, setQuery] = useState<ProjectPerformanceQuery | null>(null);

  useEffect(() => {
    if (!enabled) return undefined;
    let cancelled = false;
    setQuery({ project, kind: 'loading' });
    calculateProjectPerformanceWithRust(project).then(
      (value) => { if (!cancelled) setQuery({ project, kind: 'done', assessment: value }); },
      (reason: unknown) => {
        if (!cancelled) {
          setQuery({ project, kind: 'error', error: reason instanceof Error ? reason.message : String(reason) });
        }
      },
    );
    return () => { cancelled = true; };
  }, [project, enabled]);

  return enabled && query?.project === project ? query : null;
}

/** The calculated kernel assessment of a query, or null while loading, failed, incomplete or invalid. */
export function calculatedAssessment(query: ProjectPerformanceQuery | null): ProjectPerformanceAssessment | null {
  if (query?.kind !== 'done') return null;
  const { assessment } = query;
  return assessment.status === 'calculated_unverified' && assessment.performance ? assessment : null;
}
