/**
 * The last maatwerkadvies calculation per open document (follow-up of review 38),
 * so leaving the maatwerkadvies step and coming back still shows it.
 *
 * Each calculation is kept with the kernel input it was made from; the panel
 * shows it only while the document still has that input (`matchingAssessment`
 * in MaatwerkadviesPanel), so an edit, an edition switch, undo or another
 * document never shows a result that does not belong to it.
 */
import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from 'react';
import type { IProject } from '../core/energy/types';
import type { MaatwerkadviesAssessment } from '../core/nta/KernelClient';
import { useEnergy } from './EnergyContext';

/** A calculation with the kernel input it was made from (review 38). */
export interface StoredAssessment {
  projectId: string;
  project: IProject;
  input: string;
  assessment: MaatwerkadviesAssessment;
}

export interface MaatwerkadviesResultState {
  /** The calculation kept for the open document, matching or not. */
  stored: StoredAssessment | null;
  /** Keeps `value` for the document it was calculated for (`value.projectId`). */
  record: (value: StoredAssessment) => void;
  /** Drops the calculation of the open document. */
  clear: () => void;
}

const MaatwerkadviesResultContext = createContext<{
  entries: Map<string, StoredAssessment>; version: number; bump: () => void;
} | null>(null);

export function MaatwerkadviesResultProvider({ children }: { children: ReactNode }) {
  const entries = useRef(new Map<string, StoredAssessment>());
  const [version, setVersion] = useState(0);
  const bump = useCallback(() => setVersion((value) => value + 1), []);
  const value = useMemo(() => ({ entries: entries.current, version, bump }), [version, bump]);
  return <MaatwerkadviesResultContext.Provider value={value}>{children}</MaatwerkadviesResultContext.Provider>;
}

/**
 * The maatwerkadvies calculation of the open document. Outside a provider
 * (isolated panels, unit tests) it is kept in the panel itself.
 */
export function useMaatwerkadviesResult(): MaatwerkadviesResultState {
  const shared = useContext(MaatwerkadviesResultContext);
  const { state } = useEnergy();
  const projectId = state.project.id;
  const [local, setLocal] = useState<StoredAssessment | null>(null);

  const stored = shared ? shared.entries.get(projectId) ?? null : local;
  const record = useCallback((value: StoredAssessment) => {
    if (shared) { shared.entries.set(value.projectId, value); shared.bump(); } else setLocal(value);
  }, [shared]);
  const clear = useCallback(() => {
    if (shared) { if (shared.entries.delete(projectId)) shared.bump(); } else setLocal(null);
  }, [shared, projectId]);
  return { stored, record, clear };
}
