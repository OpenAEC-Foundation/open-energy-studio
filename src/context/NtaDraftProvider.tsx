/**
 * One NTA input draft per open document (ontwerp §6.2, F6).
 *
 * The NTA sections on Project, Gebouw and Installaties all edit this shared
 * draft of `project.ntaCalculation`, so moving between steps loses nothing.
 * Nothing reaches the kernel until "Toepassen", which writes the draft into the
 * project exactly like the former Opslaan of the single form (with the
 * ventilation synchronised). A new block from outside (the JSON editor, the
 * full form, an import, undo of the document) replaces the draft.
 */
import { createContext, useCallback, useContext, useMemo, useReducer, useState, type ReactNode } from 'react';
import type { IProject } from '../core/energy/types';
import type { NtaCalculationInput } from '../core/nta/KernelClient';
import { buildNtaCalculationTemplate } from '../core/nta/NtaCalculationTemplate';
import { syncVentilation } from '../core/nta/NtaFormModels';
import { formatKernelPath, type PathSegment } from '../core/nta/pathUtil';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { useEnergy } from './EnergyContext';

const MAX_HISTORY = 100;

/** Leaf paths (kernel notation, without prefix) where `a` and `b` differ. */
export function changedPaths(a: unknown, b: unknown, path: PathSegment[] = []): string[] {
  if (Object.is(a, b)) return [];
  const objectA = a != null && typeof a === 'object';
  const objectB = b != null && typeof b === 'object';
  if (!objectA || !objectB || Array.isArray(a) !== Array.isArray(b)) {
    // A member set to undefined is the same as an absent one.
    if (a === undefined && b === undefined) return [];
    return [formatKernelPath(path)];
  }
  const keys = Array.isArray(a)
    ? Array.from({ length: Math.max((a as unknown[]).length, (b as unknown[]).length) }, (_, index) => index)
    : [...new Set([...Object.keys(a as object), ...Object.keys(b as object)])];
  return keys.flatMap((key) => changedPaths(
    (a as Record<string | number, unknown>)[key], (b as Record<string | number, unknown>)[key], [...path, key]));
}

interface DraftState {
  projectId: string;
  /** The block the draft started from (`project.ntaCalculation` by identity). */
  base: Draft | null;
  draft: Draft | null;
  history: Array<Draft | null>;
}

type DraftAction =
  | { type: 'reset'; projectId: string; base: Draft | null }
  | { type: 'set'; next: (current: Draft) => Draft; fallback: () => Draft }
  | { type: 'undo' }
  | { type: 'discard' };

function reducer(state: DraftState, action: DraftAction): DraftState {
  switch (action.type) {
    case 'reset':
      return { projectId: action.projectId, base: action.base, draft: action.base, history: [] };
    case 'set': {
      const next = action.next(state.draft ?? action.fallback());
      if (next === state.draft) return state;
      return { ...state, draft: next, history: [...state.history, state.draft].slice(-MAX_HISTORY) };
    }
    case 'undo':
      if (state.history.length === 0) return state;
      return { ...state, draft: state.history[state.history.length - 1], history: state.history.slice(0, -1) };
    case 'discard':
      return { ...state, draft: state.base, history: [] };
    default:
      return state;
  }
}

export interface NtaDraftState {
  project: IProject;
  /** The applied block, or null when the project has none. */
  base: Draft | null;
  /** The block being edited, or null when there is none and none was started. */
  draft: Draft | null;
  /** Leaf paths changed against the applied block (empty for an unchanged draft). */
  changes: string[];
  /** A new block that is not applied yet. */
  isNew: boolean;
  dirty: boolean;
  canUndo: boolean;
  change: (path: Path, value: unknown) => void;
  /** Functional update of the whole draft (e.g. switching the ventilation mode). */
  update: (next: (current: Draft) => Draft) => void;
  /** Start a block from the template of the project. */
  start: () => void;
  undo: () => void;
  discard: () => void;
  /** Write the draft into the project ("Toepassen"). */
  apply: () => void;
  /** Open part of the heating stepper (Opwekking … Zonneverwarming). */
  heatingPart: string;
  setHeatingPart: (part: string) => void;
}

const NtaDraftContext = createContext<NtaDraftState | null>(null);

export function NtaDraftProvider({ children }: { children: ReactNode }) {
  const { state: energy, dispatch } = useEnergy();
  const project = energy.project;
  const applied = (project.ntaCalculation as unknown as Draft | undefined) ?? null;
  const [heatingPart, setHeatingPart] = useState('generation');
  const [state, send] = useReducer(reducer, null, () => ({ projectId: project.id, base: applied, draft: applied, history: [] }));
  // A new document or a block written elsewhere replaces the draft (adjusting state during render).
  if (state.projectId !== project.id || state.base !== applied) send({ type: 'reset', projectId: project.id, base: applied });
  const current = state.projectId === project.id && state.base === applied ? state : { ...state, base: applied, draft: applied, history: [] };

  const fallback = useCallback(() => buildNtaCalculationTemplate(project) as Draft, [project]);
  const change = useCallback((path: Path, value: unknown) => send({ type: 'set', next: (draft) => write(draft, path, value), fallback }), [fallback]);
  const update = useCallback((next: (draft: Draft) => Draft) => send({ type: 'set', next, fallback }), [fallback]);
  const start = useCallback(() => send({ type: 'set', next: (draft) => draft, fallback }), [fallback]);
  const undo = useCallback(() => send({ type: 'undo' }), []);
  const discard = useCallback(() => send({ type: 'discard' }), []);
  const draft = current.draft;
  const apply = useCallback(() => {
    if (draft == null) return;
    dispatch({ type: 'SET_NTA_CALCULATION', payload: syncVentilation(draft, project) as unknown as NtaCalculationInput });
  }, [dispatch, draft, project]);

  const value = useMemo<NtaDraftState>(() => {
    const isNew = current.base == null && draft != null;
    const changes = isNew ? [] : changedPaths(current.base, draft);
    return {
      project, base: current.base, draft, changes, isNew,
      dirty: isNew || changes.length > 0,
      canUndo: current.history.length > 0,
      change, update, start, undo, discard, apply, heatingPart, setHeatingPart,
    };
  }, [project, current.base, current.history.length, draft, change, update, start, undo, discard, apply, heatingPart]);

  return <NtaDraftContext.Provider value={value}>{children}</NtaDraftContext.Provider>;
}

/** The shared NTA draft, or null outside a document (isolated panels, unit tests). */
export function useNtaDraft(): NtaDraftState | null {
  return useContext(NtaDraftContext);
}
