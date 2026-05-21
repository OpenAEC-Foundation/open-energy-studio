/**
 * Central application state: the open project, last computed result,
 * undo/redo history, dirty flag, and calculation status.
 *
 * Follows the `open-heatloss-studio` pattern: a single Zustand store with
 * immer-style mutators, plus a snapshot stack for undo/redo (50-step cap).
 */
import { create } from 'zustand';
import { produce, type Draft } from 'immer';

import * as backend from '../lib/backend';
import type { ProjectV2 } from '../types/project';
import type { AppError, ProjectResult } from '../types/result';

const HISTORY_CAP = 50;

interface ProjectState {
  // ── data ────────────────────────────────────────────────────────────
  project: ProjectV2 | null;
  result: ProjectResult | null;
  filePath: string | null;
  isDirty: boolean;

  // ── status ──────────────────────────────────────────────────────────
  isCalculating: boolean;
  isLoading: boolean;
  isSaving: boolean;
  error: AppError | null;

  // ── history ─────────────────────────────────────────────────────────
  past: ProjectV2[];
  future: ProjectV2[];

  // ── actions ─────────────────────────────────────────────────────────
  setProject: (project: ProjectV2) => void;
  patchProject: (fn: (draft: Draft<ProjectV2>) => void) => void;
  setResult: (result: ProjectResult | null) => void;
  setError: (error: AppError | null) => void;
  undo: () => void;
  redo: () => void;
  reset: () => void;

  // ── async ───────────────────────────────────────────────────────────
  newProject: (name?: string) => Promise<void>;
  loadProject: (path: string) => Promise<void>;
  saveProject: (path?: string) => Promise<void>;
  calculate: () => Promise<void>;
}

const initial: Pick<
  ProjectState,
  | 'project'
  | 'result'
  | 'filePath'
  | 'isDirty'
  | 'isCalculating'
  | 'isLoading'
  | 'isSaving'
  | 'error'
  | 'past'
  | 'future'
> = {
  project: null,
  result: null,
  filePath: null,
  isDirty: false,
  isCalculating: false,
  isLoading: false,
  isSaving: false,
  error: null,
  past: [],
  future: [],
};

export const useProjectStore = create<ProjectState>((set, get) => ({
  ...initial,

  setProject: (project) =>
    set((s) => ({
      project,
      past: pushHistory(s.past, s.project),
      future: [],
      isDirty: true,
      result: null,
    })),

  patchProject: (fn) =>
    set((s) => {
      if (!s.project) return s;
      const next = produce(s.project, fn);
      return {
        project: next,
        past: pushHistory(s.past, s.project),
        future: [],
        isDirty: true,
        // result is invalidated by any mutation; downstream may schedule a
        // re-calc via auto-calc.
        result: null,
      };
    }),

  setResult: (result) => set({ result }),
  setError: (error) => set({ error }),

  undo: () =>
    set((s) => {
      if (s.past.length === 0 || !s.project) return s;
      const previous = s.past[s.past.length - 1]!;
      return {
        project: previous,
        past: s.past.slice(0, -1),
        future: [s.project, ...s.future],
        result: null,
        isDirty: true,
      };
    }),

  redo: () =>
    set((s) => {
      if (s.future.length === 0 || !s.project) return s;
      const next = s.future[0]!;
      return {
        project: next,
        past: pushHistory(s.past, s.project),
        future: s.future.slice(1),
        result: null,
        isDirty: true,
      };
    }),

  reset: () => set({ ...initial }),

  newProject: async (name = 'Nieuw project') => {
    set({ isLoading: true, error: null });
    try {
      const project = await backend.newProject(name);
      set({
        project,
        result: null,
        filePath: null,
        isDirty: true,
        past: [],
        future: [],
        isLoading: false,
      });
    } catch (err) {
      set({ error: backend.toAppError(err), isLoading: false });
    }
  },

  loadProject: async (path) => {
    set({ isLoading: true, error: null });
    try {
      const project = await backend.loadProject(path);
      set({
        project,
        result: null,
        filePath: path,
        isDirty: false,
        past: [],
        future: [],
        isLoading: false,
      });
    } catch (err) {
      set({ error: backend.toAppError(err), isLoading: false });
    }
  },

  saveProject: async (path) => {
    const { project, result, filePath } = get();
    if (!project) return;
    const target = path ?? filePath;
    if (!target) {
      set({
        error: {
          kind: 'validation',
          message: 'No file path provided and no current file open',
        },
      });
      return;
    }
    set({ isSaving: true, error: null });
    try {
      await backend.saveProject(target, project, result);
      set({ filePath: target, isDirty: false, isSaving: false });
    } catch (err) {
      set({ error: backend.toAppError(err), isSaving: false });
    }
  },

  calculate: async () => {
    const { project } = get();
    if (!project) return;
    set({ isCalculating: true, error: null });
    try {
      const result = await backend.calculate(project);
      set({ result, isCalculating: false });
    } catch (err) {
      set({ error: backend.toAppError(err), isCalculating: false });
    }
  },
}));

function pushHistory(past: ProjectV2[], current: ProjectV2 | null): ProjectV2[] {
  if (!current) return past;
  const next = past.length >= HISTORY_CAP ? past.slice(-(HISTORY_CAP - 1)) : past;
  return [...next, current];
}
