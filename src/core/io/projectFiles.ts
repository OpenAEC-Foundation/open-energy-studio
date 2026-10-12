// Saving, opening and closing project files, apart from the React shell so
// the decisions can be tested.
//
// Desktop: a save goes through the Rust command `save_project`, which writes
// atomically (temporary file, flush, rename, `.bak` of the previous version)
// and only to a file picked in a dialog or on the Rust-side recent list. A
// failed write is a failure: the document stays dirty and the user sees the
// error; nothing falls back to a browser download. Browser: the file is
// offered as a download.

/** What a save ended in. */
export type SaveOutcome =
  | { kind: 'saved'; path: string }
  | { kind: 'downloaded' }
  | { kind: 'cancelled' }
  | { kind: 'failed'; message: string };

export interface SaveDeps {
  /** True in the desktop app. */
  desktop: boolean;
  /** Save As dialog; null when the user cancels. */
  pickPath(defaultPath: string): Promise<string | null>;
  /** Writes the text to a picked or known path; throws on failure. */
  write(path: string, text: string): Promise<void>;
  /** Browser build: offers the text as a download. */
  download(fileName: string, text: string): void;
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

/**
 * Saves `text` as the project file. With a known path and no prompt the file
 * is written there; otherwise the Save As dialog asks for one.
 */
export async function saveProjectText(
  text: string,
  fileName: string,
  existingPath: string | null,
  forcePrompt: boolean,
  deps: SaveDeps,
): Promise<SaveOutcome> {
  if (!deps.desktop) {
    deps.download(fileName, text);
    return { kind: 'downloaded' };
  }
  let path = existingPath && !forcePrompt ? existingPath : null;
  if (!path) {
    try {
      path = await deps.pickPath(existingPath || fileName);
    } catch (error) {
      return { kind: 'failed', message: messageOf(error) };
    }
    if (!path) return { kind: 'cancelled' };
  }
  try {
    await deps.write(path, text);
    return { kind: 'saved', path };
  } catch (error) {
    return { kind: 'failed', message: messageOf(error) };
  }
}

/** Whether a read error means the file is gone (and may leave the recent list). */
export function isMissingFileError(error: unknown): boolean {
  return messageOf(error).startsWith('project_file_not_found');
}

/** A document as the close check sees it. */
export interface ClosingDocument {
  id: string;
  name: string;
  isDirty: boolean;
  /** NTA input edited but not applied (only known for the active document). */
  draftDirty: boolean;
  /** Registration data edited but not saved into the project (only known for the active document). */
  registrationDirty?: boolean;
}

export type CloseChoice = 'save' | 'discard' | 'cancel';

export interface CloseDeps {
  /** Unapplied NTA input or unsaved registration data: true to discard it and go on, false to stop. */
  confirmDiscardDraft(doc: ClosingDocument): Promise<boolean>;
  /** Unsaved changes: Opslaan · Niet opslaan · Annuleren. */
  askUnsaved(doc: ClosingDocument): Promise<CloseChoice>;
  /** Saves the document; false when the save was cancelled or failed. */
  save(doc: ClosingDocument): Promise<boolean>;
}

/** Documents that need a question before the window closes. */
export function documentsNeedingConfirmation(documents: ClosingDocument[]): ClosingDocument[] {
  return documents.filter((doc) => doc.isDirty || doc.draftDirty || doc.registrationDirty);
}

/**
 * The close flow for the whole window: per document with unsaved work the
 * same questions as closing its tab. Returns true when the window may close,
 * false when the user cancelled or a save did not succeed.
 */
export async function confirmCloseAll(documents: ClosingDocument[], deps: CloseDeps): Promise<boolean> {
  for (const doc of documentsNeedingConfirmation(documents)) {
    if ((doc.draftDirty || doc.registrationDirty) && !(await deps.confirmDiscardDraft(doc))) return false;
    if (!doc.isDirty) continue;
    const choice = await deps.askUnsaved(doc);
    if (choice === 'cancel') return false;
    if (choice === 'save' && !(await deps.save(doc))) return false;
  }
  return true;
}
