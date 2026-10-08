/**
 * The project library (feedback 8 Oct 2026): every project the app has had
 * open, as a card with what it is, where it is, its last outcome and its
 * state, kept in this browser after the document is closed. Each project has
 * a snapshot (`project.<id>`) and a card entry (`entry.<id>`); an index lists
 * the ids. The outcome lives on the entry, not in the project file, so the
 * kernel input and the fingerprints stay as they are.
 */
import type { IProject } from '../energy/types';
import { buildFlowSteps, buildQuestionState } from '../navigation/buildFlow';
import { kernelProject } from '../nta/KernelInput';
import { missingSurveyAnswers, type StoredSurvey } from '../nta/SurveyTemplates';
import { stepState, surveySteps } from '../survey/surveyFlow';

export const LIBRARY_INDEX = 'oes.library.v1.index';
const ENTRY = 'oes.library.v1.entry.';
const PROJECT = 'oes.library.v1.project.';

export type LibraryKind = 'existing_residential' | 'existing_utility' | 'new_residential' | 'new_utility';
export type LibraryDwelling = 'regular' | 'apartment' | 'floating' | 'caravan';

export interface LibraryOutcome {
  label: string | null;
  ep2: number | null;
  beng1: number | null;
  beng3: number | null;
  tojuli: number | null;
  /** Survey projects: applied defaults without a reason at the time of the outcome. */
  defaultsWithoutReason?: number;
  at: string;
  /** Hash of the input the outcome belongs to; the card says "verouderd" when the project moved on. */
  sourceHash: string;
}

export interface LibraryStatus {
  surveyDone?: number;
  surveyTotal?: number;
  buildDone?: number;
  buildTotal?: number;
  openAnswers?: number;
  importNotes?: number;
}

export interface LibraryEntry {
  id: string;
  name: string;
  address: string;
  city: string;
  kind: LibraryKind;
  dwelling?: LibraryDwelling;
  /** Folder (client) chosen by the user; absent: no folder. */
  folder?: string;
  archived?: boolean;
  filePath?: string | null;
  updatedAt: string;
  outcome?: LibraryOutcome;
  outcomeStale?: boolean;
  status: LibraryStatus;
}

export type Store = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;

export function browserStore(): Store | null {
  try { return typeof localStorage === 'undefined' ? null : localStorage; } catch { return null; }
}

const readJson = <T,>(store: Store, key: string): T | null => {
  try {
    const raw = store.getItem(key);
    return raw ? JSON.parse(raw) as T : null;
  } catch {
    return null;
  }
};
const writeJson = (store: Store, key: string, value: unknown) => {
  try { store.setItem(key, JSON.stringify(value)); } catch { /* storage full or blocked */ }
};

/** FNV-1a of a text, hex: cheap enough for every autosave tick. */
export function cheapHash(text: string): string {
  let hash = 0x811c9dc5;
  for (let index = 0; index < text.length; index += 1) {
    hash ^= text.charCodeAt(index);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return hash.toString(16).padStart(8, '0');
}

const survey = (project: IProject) => project.basisopname as (StoredSurvey & { surveyDate?: string }) | undefined;
const isNewBuild = (project: IProject) => {
  const purpose = project.registration?.purpose;
  return purpose === 'delivery' || purpose === 'bbl_check';
};

/** The input the outcome of a project belongs to: the survey, or the project as the kernel gets it. */
export function sourceHashOf(project: IProject): string {
  const stored = survey(project);
  return cheapHash(stored && !isNewBuild(project) ? JSON.stringify(stored.survey) : JSON.stringify(kernelProject(project)));
}

export function libraryKindOf(project: IProject): LibraryKind {
  const stored = survey(project);
  const residential = project.buildingFunction === 'residential';
  if (stored && !isNewBuild(project)) return residential ? 'existing_residential' : 'existing_utility';
  return residential ? 'new_residential' : 'new_utility';
}

export function libraryDwellingOf(project: IProject): LibraryDwelling | undefined {
  const stored = survey(project);
  if (!stored || project.buildingFunction !== 'residential') return undefined;
  const building = (stored.survey.envelope as { buildingKind?: { kind?: string } } | undefined)?.buildingKind?.kind;
  if (building === 'floating' || building === 'caravan') return building;
  return (stored.survey.dwelling as { kind?: string } | undefined)?.kind === 'apartment' ? 'apartment' : 'regular';
}

export function libraryStatusOf(project: IProject): LibraryStatus {
  const status: LibraryStatus = {};
  const notes = (project.importLog ?? []).reduce((sum, record) => sum + (record.notes?.length ?? 0), 0);
  if (notes > 0) status.importNotes = notes;
  const stored = survey(project);
  if (stored && !isNewBuild(project)) {
    const steps = surveySteps(stored.kind).filter((step) => !step.special);
    status.surveyTotal = steps.length;
    status.surveyDone = steps.filter((step) => stepState(step, stored.progress) === 'done').length;
    status.openAnswers = missingSurveyAnswers(stored).length;
    return status;
  }
  const steps = buildFlowSteps(project).filter((step) => step.questions.length > 0);
  status.buildTotal = steps.reduce((sum, step) => sum + step.questions.length, 0);
  status.buildDone = steps.reduce((sum, step) => sum + step.questions.filter((question) =>
    buildQuestionState(project.workflowProgress, step.id, question) === 'done').length, 0);
  return status;
}

/** The card entry of a project; folder, archive flag and outcome come from the previous entry. */
export function libraryEntryFrom(project: IProject, previous: LibraryEntry | null, filePath: string | null | undefined, now: string): LibraryEntry {
  const outcome = previous?.outcome;
  return {
    id: project.id,
    name: project.name,
    address: project.address,
    city: project.city,
    kind: libraryKindOf(project),
    ...(libraryDwellingOf(project) ? { dwelling: libraryDwellingOf(project) } : {}),
    ...(previous?.folder ? { folder: previous.folder } : {}),
    ...(previous?.archived ? { archived: true } : {}),
    ...(filePath !== undefined ? { filePath } : previous?.filePath !== undefined ? { filePath: previous.filePath } : {}),
    updatedAt: now,
    ...(outcome ? { outcome, outcomeStale: outcome.sourceHash !== sourceHashOf(project) } : {}),
    status: libraryStatusOf(project),
  };
}

export function libraryIds(store: Store): string[] {
  const ids = readJson<unknown>(store, LIBRARY_INDEX);
  return Array.isArray(ids) ? ids.filter((id): id is string => typeof id === 'string') : [];
}

export function readLibrary(store: Store): LibraryEntry[] {
  return libraryIds(store).map((id) => readJson<LibraryEntry>(store, ENTRY + id)).filter((entry): entry is LibraryEntry => entry != null && typeof entry.id === 'string');
}

export function readLibraryEntry(store: Store, id: string): LibraryEntry | null {
  return readJson<LibraryEntry>(store, ENTRY + id);
}

export function readLibraryProject(store: Store, id: string): IProject | null {
  return readJson<IProject>(store, PROJECT + id);
}

function writeEntry(store: Store, entry: LibraryEntry) {
  writeJson(store, ENTRY + entry.id, entry);
  const ids = libraryIds(store);
  if (!ids.includes(entry.id)) writeJson(store, LIBRARY_INDEX, [...ids, entry.id]);
  notify();
}

/**
 * Saves the open documents whose project changed since `seen` (a map of the
 * projects last written); returns the new map. A document is written in full
 * the first time, so a project also lands here after a restore.
 */
export function saveDocumentsToLibrary(
  store: Store, documents: Array<{ id: string; filePath: string | null; project: IProject }>, seen: Map<string, IProject>, now = new Date().toISOString(),
): Map<string, IProject> {
  const next = new Map(seen);
  for (const doc of documents) {
    if (seen.get(doc.id) === doc.project) continue;
    writeJson(store, PROJECT + doc.project.id, doc.project);
    writeEntry(store, libraryEntryFrom(doc.project, readLibraryEntry(store, doc.project.id), doc.filePath, now));
    next.set(doc.id, doc.project);
  }
  return next;
}

export function updateLibraryEntry(store: Store, id: string, patch: Partial<Pick<LibraryEntry, 'folder' | 'archived' | 'name'>>): LibraryEntry | null {
  const entry = readLibraryEntry(store, id);
  if (!entry) return null;
  const next: LibraryEntry = { ...entry, ...patch };
  if (!next.folder) delete next.folder;
  if (!next.archived) delete next.archived;
  writeEntry(store, next);
  return next;
}

/** Records a calculated outcome on the entry of an open project. */
export function rememberOutcome(store: Store, project: IProject, outcome: Omit<LibraryOutcome, 'sourceHash' | 'at'>, now = new Date().toISOString()): void {
  const previous = readLibraryEntry(store, project.id);
  const entry = libraryEntryFrom(project, previous, undefined, previous?.updatedAt ?? now);
  const full: LibraryOutcome = { ...outcome, at: now, sourceHash: sourceHashOf(project) };
  if (previous?.outcome && JSON.stringify({ ...previous.outcome, at: '' }) === JSON.stringify({ ...full, at: '' })) return;
  writeEntry(store, { ...entry, outcome: full, outcomeStale: false });
}

export function removeFromLibrary(store: Store, id: string): void {
  try { store.removeItem(ENTRY + id); store.removeItem(PROJECT + id); } catch { /* ignore */ }
  writeJson(store, LIBRARY_INDEX, libraryIds(store).filter((item) => item !== id));
  notify();
}

/** The folders in use, sorted. */
export function libraryFolders(entries: LibraryEntry[]): string[] {
  return [...new Set(entries.map((entry) => entry.folder).filter((folder): folder is string => Boolean(folder)))].sort((a, b) => a.localeCompare(b));
}

// ── Change notification for the page ──
const listeners = new Set<() => void>();
let version = 0;
function notify() {
  version += 1;
  for (const listener of listeners) listener();
}
export function subscribeLibrary(listener: () => void): () => void {
  listeners.add(listener);
  return () => { listeners.delete(listener); };
}
export const libraryVersion = () => version;
