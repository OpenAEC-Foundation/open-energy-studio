/**
 * One registration draft per open document (review 35), like the NTA draft of
 * `NtaDraftProvider`: the Registratie form edits it, so leaving the step loses
 * nothing, and the close checks know about unsaved registration data.
 * Nothing reaches the project until Opslaan in the form.
 *
 * Registration data stored elsewhere (the relabel comparison, an evidence
 * link from the context panel, undo of the document) becomes the new base;
 * the members the adviser changed in the draft are kept on top of it. The
 * evidence register merges per item, so a link made from the context panel
 * survives unsaved edits to other evidence items.
 */
import { createContext, useCallback, useContext, useMemo, useReducer, type ReactNode } from 'react';
import type { NtaEvidenceItem, NtaRegistration } from '../core/nta/KernelClient';
import { cleanRegistration } from '../core/nta/Registration';
import { changedPaths } from './NtaDraftProvider';
import { useEnergy } from './EnergyContext';

/** The registration as it would be stored, without the program stamp. */
function comparable(registration: NtaRegistration | undefined): NtaRegistration {
  const cleaned = registration ? cleanRegistration(registration) : undefined;
  if (!cleaned) return {};
  const { software: _software, ...rest } = cleaned;
  return rest;
}

/** Whether saving `draft` would change what is stored as `stored` (the program stamp aside). */
export function registrationChanged(draft: NtaRegistration, stored: NtaRegistration | undefined): boolean {
  return changedPaths(comparable(draft), comparable(stored)).length > 0;
}

const same = (a: unknown, b: unknown) => changedPaths(a, b).length === 0;

/**
 * Three-way merge of the evidence register by item id: items and links added
 * elsewhere stay, the adviser's edits and deletions stay. An item changed both
 * in the draft and elsewhere keeps the adviser's version and is reported in
 * `conflicts` (by file name).
 */
export function mergeEvidence(
  draft: NtaEvidenceItem[], oldBase: NtaEvidenceItem[], newBase: NtaEvidenceItem[], conflicts: string[] = [],
): NtaEvidenceItem[] {
  const byId = (items: NtaEvidenceItem[]) => new Map(items.map((item) => [item.id, item]));
  const before = byId(oldBase);
  const mine = byId(draft);
  const result: NtaEvidenceItem[] = [];
  for (const theirs of newBase) {
    const base = before.get(theirs.id);
    const own = mine.get(theirs.id);
    if (!base) {
      // Added elsewhere (or by both under the same id: the adviser's version).
      if (own && !same(own, theirs)) conflicts.push(own.fileName);
      result.push(own ?? theirs);
      continue;
    }
    const changedHere = !own || !same(own, base);
    const changedThere = !same(theirs, base);
    if (!changedHere) result.push(theirs);
    else {
      if (changedThere) conflicts.push(base.fileName);
      if (own) result.push(own);
    }
  }
  const inNew = new Set(newBase.map((item) => item.id));
  for (const own of draft) {
    if (inNew.has(own.id)) continue;
    const base = before.get(own.id);
    // Added in the draft, or removed elsewhere while the adviser edited it.
    if (!base) result.push(own);
    else if (!same(own, base)) { conflicts.push(own.fileName); result.push(own); }
  }
  return result;
}

/**
 * The draft on a new base: the top-level members the adviser changed against
 * the old base stay; everything else follows the new base.
 */
export function rebaseRegistration(
  draft: NtaRegistration, oldBase: NtaRegistration, newBase: NtaRegistration, conflicts: string[] = [],
): NtaRegistration {
  const result: Record<string, unknown> = { ...newBase };
  const before = oldBase as Record<string, unknown>;
  const after = draft as Record<string, unknown>;
  for (const key of new Set([...Object.keys(before), ...Object.keys(after)])) {
    if (key === 'software' || key === 'relabelComparison') continue;
    if (changedPaths(before[key], after[key]).length === 0) continue;
    if (key === 'evidence') {
      const merged = mergeEvidence(draft.evidence ?? [], oldBase.evidence ?? [], newBase.evidence ?? [], conflicts);
      if (merged.length > 0 || draft.evidence !== undefined) result.evidence = merged;
      else delete result.evidence;
      continue;
    }
    if (after[key] === undefined) delete result[key];
    else result[key] = after[key];
  }
  return result as NtaRegistration;
}

interface State {
  projectId: string;
  /** `project.registration` the draft follows (by identity). */
  base: NtaRegistration | undefined;
  draft: NtaRegistration;
  /** Evidence files changed both in the draft and elsewhere since the last save (the adviser's version kept). */
  evidenceConflicts: string[];
}

type Action =
  | { type: 'rebase'; projectId: string; base: NtaRegistration | undefined }
  | { type: 'set'; next: (current: NtaRegistration) => NtaRegistration }
  | { type: 'discard' };

function reducer(state: State, action: Action): State {
  switch (action.type) {
    case 'rebase': {
      const fresh = action.base ?? {};
      if (action.projectId !== state.projectId || !registrationChanged(state.draft, state.base)) {
        return { projectId: action.projectId, base: action.base, draft: fresh, evidenceConflicts: [] };
      }
      const found: string[] = [];
      const draft = rebaseRegistration(state.draft, state.base ?? {}, fresh, found);
      if (!registrationChanged(draft, action.base)) return { ...state, base: action.base, draft: fresh, evidenceConflicts: [] };
      const evidenceConflicts = [...new Set([...state.evidenceConflicts, ...found])];
      return { ...state, base: action.base, draft, evidenceConflicts };
    }
    case 'set':
      return { ...state, draft: action.next(state.draft) };
    case 'discard':
      return { ...state, draft: state.base ?? {}, evidenceConflicts: [] };
    default:
      return state;
  }
}

export interface RegistrationDraftState {
  draft: NtaRegistration;
  /** The stored registration the draft is compared with. */
  stored: NtaRegistration | undefined;
  /** Saving would change the stored registration. */
  dirty: boolean;
  /** Evidence files changed both in the draft and elsewhere; the draft keeps the adviser's version. */
  evidenceConflicts: string[];
  update: (patch: Partial<NtaRegistration>) => void;
  discard: () => void;
}

const RegistrationDraftContext = createContext<RegistrationDraftState | null>(null);

export function RegistrationDraftProvider({ children }: { children: ReactNode }) {
  const { state: energy } = useEnergy();
  const project = energy.project;
  const stored = project.registration;
  const [state, send] = useReducer(reducer, null, () => ({ projectId: project.id, base: stored, draft: stored ?? {}, evidenceConflicts: [] as string[] }));
  // A new document or registration data stored elsewhere (adjusting state during render).
  if (state.projectId !== project.id || state.base !== stored) send({ type: 'rebase', projectId: project.id, base: stored });
  const current = state.projectId === project.id && state.base === stored
    ? state : reducer(state, { type: 'rebase', projectId: project.id, base: stored });

  const update = useCallback((patch: Partial<NtaRegistration>) =>
    send({ type: 'set', next: (draft) => ({ ...draft, ...patch }) }), []);
  const discard = useCallback(() => send({ type: 'discard' }), []);
  const draft = current.draft;
  const evidenceConflicts = current.evidenceConflicts;
  const value = useMemo<RegistrationDraftState>(() => ({
    draft, stored, dirty: registrationChanged(draft, stored), evidenceConflicts, update, discard,
  }), [draft, stored, evidenceConflicts, update, discard]);
  return <RegistrationDraftContext.Provider value={value}>{children}</RegistrationDraftContext.Provider>;
}

/** The document's registration draft, or null outside a document (isolated panels, unit tests). */
export function useRegistrationDraft(): RegistrationDraftState | null {
  return useContext(RegistrationDraftContext);
}
