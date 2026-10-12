/**
 * The last "Opname doorrekenen" result per open document (follow-up of review 36),
 * so leaving the survey step and coming back still shows it.
 *
 * A result is kept with the survey and edition it was calculated from and is
 * only handed out while the document still has that survey (the reasons for
 * applied defaults aside, which are documentation) in that edition. Any other
 * edit, a change of edition, a discarded survey or another document gives no
 * result, the same as the panel clearing it on an edit.
 */
import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from 'react';
import type { OpnameAssessment } from '../core/nta/KernelClient';
import type { StoredSurvey } from '../core/nta/SurveyTemplates';
import { changedPaths } from './NtaDraftProvider';
import { useEnergy } from './EnergyContext';

interface Entry {
  survey: StoredSurvey;
  edition: string | null;
  result: OpnameAssessment;
}

/** The survey as the kernel reads it: the reasons for applied defaults are documentation only. */
function calculated(survey: StoredSurvey): unknown {
  const { inklapRedenen: _reasons, ...rest } = (survey.survey ?? {}) as Record<string, unknown>;
  return { kind: survey.kind, survey: rest };
}

/** Whether `entry` was calculated from `survey` in `edition`. */
export function resultMatches(entry: Entry | undefined, survey: StoredSurvey | undefined, edition: string | null): boolean {
  if (!entry || !survey || entry.edition !== edition) return false;
  return entry.survey === survey || changedPaths(calculated(entry.survey), calculated(survey)).length === 0;
}

export interface SurveyResultState {
  /** The result for the document's current survey and edition, or null. */
  result: OpnameAssessment | null;
  /** Keeps `result` as calculated from `survey` in `edition` for the current document. */
  record: (survey: StoredSurvey, edition: string | null, result: OpnameAssessment) => void;
  clear: () => void;
}

const SurveyResultContext = createContext<{
  entries: Map<string, Entry>; version: number; bump: () => void;
} | null>(null);

export function SurveyResultProvider({ children }: { children: ReactNode }) {
  const entries = useRef(new Map<string, Entry>());
  const [version, setVersion] = useState(0);
  const bump = useCallback(() => setVersion((value) => value + 1), []);
  const value = useMemo(() => ({ entries: entries.current, version, bump }), [version, bump]);
  return <SurveyResultContext.Provider value={value}>{children}</SurveyResultContext.Provider>;
}

/**
 * The survey result of the open document. Outside a provider (isolated panels,
 * unit tests) it is kept in the panel itself, under the same matching rule.
 */
export function useSurveyResult(): SurveyResultState {
  const shared = useContext(SurveyResultContext);
  const { state } = useEnergy();
  const project = state.project;
  const survey = project.basisopname as StoredSurvey | undefined;
  const edition = project.ntaCalculation?.normVersion ?? null;
  const [local, setLocal] = useState<Entry | undefined>(undefined);
  const projectId = project.id;

  const entry = shared ? shared.entries.get(projectId) : local;
  const result = resultMatches(entry, survey, edition) ? entry!.result : null;

  const record = useCallback((from: StoredSurvey, at: string | null, next: OpnameAssessment) => {
    const value = { survey: from, edition: at, result: next };
    if (shared) { shared.entries.set(projectId, value); shared.bump(); } else setLocal(value);
  }, [shared, projectId]);
  const clear = useCallback(() => {
    if (shared) { if (shared.entries.delete(projectId)) shared.bump(); } else setLocal(undefined);
  }, [shared, projectId]);
  return { result, record, clear };
}
