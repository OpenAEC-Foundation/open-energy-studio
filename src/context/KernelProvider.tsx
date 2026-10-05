/**
 * One kernel run per open document (ontwerp §11, F3).
 *
 * The provider runs the Rust project route for the active project once and
 * shares the answer with every view: results, report, print preview, the NTA
 * panel, the status bar and the step status in the navigation. It also keeps
 * the last settled answer of the same project, so a re-run after an edit shows
 * as "verouderd" instead of flashing empty values.
 *
 * Components rendered outside the provider (unit tests, isolated panels) keep
 * running their own query through `useKernelQuery`.
 */
import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from 'react';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { useProjectPerformance, type ProjectPerformanceQuery } from '../core/nta/useProjectPerformance';
import { kernelVerdict, type KernelVerdict } from '../core/nta/KernelVerdict';

export type KernelPhase = 'idle' | 'loading' | 'current' | 'stale' | 'error';

export interface KernelState {
  project: IProject;
  query: ProjectPerformanceQuery | null;
  /** The last settled assessment of this project (also while the next run is pending). */
  settled: ProjectPerformanceAssessment | null;
  verdict: KernelVerdict;
  /**
   * current: the shown answer belongs to this project state; stale: an older
   * answer of this project is shown while the next run is pending; loading: no
   * answer yet; error: the kernel could not be reached.
   */
  phase: KernelPhase;
  error: string | null;
  /** Ask the kernel again for the same project ("Herberekenen"). */
  refresh: () => void;
}

const KernelContext = createContext<KernelState | null>(null);

export function KernelProvider({ project, children }: { project: IProject; children: ReactNode }) {
  const [refreshKey, setRefreshKey] = useState(0);
  const query = useProjectPerformance(project, true, refreshKey);
  const settledRef = useRef<{ projectId: string; assessment: ProjectPerformanceAssessment } | null>(null);
  if (query?.kind === 'done') settledRef.current = { projectId: project.id, assessment: query.assessment };
  else if (settledRef.current?.projectId !== project.id) settledRef.current = null;
  const settled = settledRef.current?.assessment ?? null;

  const refresh = useCallback(() => setRefreshKey((key) => key + 1), []);
  const phase: KernelPhase = query?.kind === 'done' ? 'current'
    : query?.kind === 'error' ? 'error'
      : settled ? 'stale' : query ? 'loading' : 'idle';
  const value = useMemo<KernelState>(() => ({
    project,
    query,
    settled,
    verdict: kernelVerdict(settled),
    phase,
    error: query?.kind === 'error' ? query.error : null,
    refresh,
  }), [project, query, settled, phase, refresh]);

  return <KernelContext.Provider value={value}>{children}</KernelContext.Provider>;
}

/** The shared kernel state, or null outside a document (welcome screen, isolated tests). */
export function useKernel(): KernelState | null {
  return useContext(KernelContext);
}

/**
 * The kernel query for `project`: the shared one when the provider runs for the
 * same project state, otherwise an own (debounced) run. Pass `enabled = false`
 * when a parent supplies the query.
 */
export function useKernelQuery(project: IProject, enabled = true): ProjectPerformanceQuery | null {
  const shared = useContext(KernelContext);
  const usesShared = shared != null && shared.project === project;
  const own = useProjectPerformance(project, enabled && !usesShared);
  if (!enabled) return null;
  return usesShared ? shared.query : own;
}
