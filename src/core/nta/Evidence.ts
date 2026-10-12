import type { NtaEvidenceItem, NtaEvidenceKind } from './KernelClient';

/**
 * Evidence register of the project dossier (BRL 9500-W/U Bijlage 3).
 * The project keeps each file's metadata and SHA-256; the bytes live in
 * the desktop app's data folder (`evidence/<hash>-<name>`) or, in the
 * browser, in this session's memory until the dossier is exported.
 */

export const EVIDENCE_KINDS: NtaEvidenceKind[] = [
  'photo_overview', 'photo_detail', 'invoice', 'drawing', 'datasheet',
  'declaration_of_performance', 'quality_declaration', 'client_statement', 'other',
];

/** Prefix that links a `…Reference` text to an evidence item. */
export const EVIDENCE_REFERENCE_PREFIX = 'evidence:';

const sessionFiles = new Map<string, Uint8Array>();

/** Largest file accepted in the register (per file). */
export const MAX_EVIDENCE_BYTES = 200 * 1024 * 1024;

/**
 * File types accepted as evidence: documents, images, drawings and office
 * files (BRL 9500 Bijlage 3 asks for photos, invoices, drawings, datasheets
 * and declarations). Anything else is refused with a clear message.
 */
export const EVIDENCE_EXTENSIONS = [
  'pdf', 'jpg', 'jpeg', 'png', 'gif', 'webp', 'heic', 'heif', 'tif', 'tiff', 'bmp', 'svg',
  'doc', 'docx', 'xls', 'xlsx', 'odt', 'ods', 'rtf', 'txt', 'csv', 'xml', 'json',
  'dwg', 'dxf', 'ifc', 'msg', 'eml',
] as const;

/** The `accept` attribute that matches `EVIDENCE_EXTENSIONS`. */
export const EVIDENCE_ACCEPT = EVIDENCE_EXTENSIONS.map((extension) => `.${extension}`).join(',');

export type EvidenceImportCode = 'evidence_too_large' | 'evidence_type_not_allowed';

/** A file the register refuses; `code` is an i18n key under `evidence.error.`. */
export class EvidenceImportError extends Error {
  constructor(readonly code: EvidenceImportCode, readonly fileName: string) {
    super(`${code}: ${fileName}`);
    this.name = 'EvidenceImportError';
  }
}

function extensionOf(fileName: string): string {
  const name = fileName.trim();
  const dot = name.lastIndexOf('.');
  return dot > 0 && dot < name.length - 1 ? name.slice(dot + 1).toLowerCase() : '';
}

/** Throws an `EvidenceImportError` for a file that is too large or of a type the register does not take. */
export function checkEvidenceFile(file: { name: string; size: number }): void {
  if (!(EVIDENCE_EXTENSIONS as readonly string[]).includes(extensionOf(file.name))) {
    throw new EvidenceImportError('evidence_type_not_allowed', file.name);
  }
  if (!Number.isFinite(file.size) || file.size > MAX_EVIDENCE_BYTES) {
    throw new EvidenceImportError('evidence_too_large', file.name);
  }
}

export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  // Copy into a buffer of this realm: under jsdom on Node 20 a view that came
  // from another realm (TextEncoder, FileReader) is rejected by WebCrypto.
  const copy = new Uint8Array(new ArrayBuffer(bytes.byteLength));
  copy.set(bytes);
  const digest = await crypto.subtle.digest('SHA-256', copy);
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, '0')).join('');
}

/**
 * Ids handed out to files that are still being read or hashed, so two
 * additions running at the same time never get the same id.
 */
const reservedIds = new Set<string>();

export function nextEvidenceId(existing: NtaEvidenceItem[]): string {
  const used = new Set(existing.map((item) => item.id));
  let index = existing.length + 1;
  while (used.has(`ev-${index}`) || reservedIds.has(`ev-${index}`)) index += 1;
  return `ev-${index}`;
}

/** Releases ids reserved by `createEvidenceItem` once the items are in the register. */
export function releaseEvidenceIds(ids: string[]): void {
  for (const id of ids) reservedIds.delete(id);
}

/**
 * Adds items to a register, renaming any whose id is already taken (by an
 * entry added meanwhile), so ids stay unique.
 */
export function appendEvidence(existing: NtaEvidenceItem[], added: NtaEvidenceItem[]): NtaEvidenceItem[] {
  const result = [...existing];
  const used = new Set(existing.map((item) => item.id));
  for (const item of added) {
    let id = item.id;
    if (used.has(id)) {
      let index = result.length + 1;
      while (used.has(`ev-${index}`)) index += 1;
      id = `ev-${index}`;
    }
    used.add(id);
    result.push(id === item.id ? item : { ...item, id });
  }
  return result;
}

/**
 * Makes evidence ids unique (a hand-edited file or an old race could repeat
 * one): later duplicates get a fresh `ev-N`. Returns the renamed ids.
 */
export function repairEvidenceIds(evidence: NtaEvidenceItem[]): {
  evidence: NtaEvidenceItem[]; renamed: Array<{ from: string; to: string }>;
} {
  const seen = new Set<string>();
  const all = new Set(evidence.map((item) => item.id));
  const renamed: Array<{ from: string; to: string }> = [];
  const repaired = evidence.map((item) => {
    if (item.id && !seen.has(item.id)) {
      seen.add(item.id);
      return item;
    }
    let index = 1;
    while (all.has(`ev-${index}`) || seen.has(`ev-${index}`)) index += 1;
    const id = `ev-${index}`;
    seen.add(id);
    all.add(id);
    renamed.push({ from: item.id, to: id });
    return { ...item, id };
  });
  return { evidence: renamed.length ? repaired : evidence, renamed };
}

/** Guesses the evidence kind from the file name. */
export function guessEvidenceKind(fileName: string): NtaEvidenceKind {
  const lower = fileName.toLowerCase();
  if (/factuur|invoice/.test(lower)) return 'invoice';
  if (/dop|prestatieverklaring/.test(lower)) return 'declaration_of_performance';
  if (/kwaliteitsverklaring|kiwa|komo|bcrg/.test(lower)) return 'quality_declaration';
  if (/tekening|plattegrond|doorsnede|\.dwg$|\.dxf$/.test(lower)) return 'drawing';
  if (/datasheet|datablad|specificatie/.test(lower)) return 'datasheet';
  if (/\.(jpe?g|png|heic|webp)$/.test(lower)) return 'photo_detail';
  return 'other';
}

const MAX_NAME_LENGTH = 80;

/**
 * A file-name part that is safe in a ZIP and on every file system: letters
 * and digits of any script, marks and emoji are kept, as are dot,
 * underscore and hyphen; no path separators, no leading dots, no trailing
 * dots or spaces (Windows strips them on extract), and the extension is
 * kept when a long name is shortened.
 */
export function safeName(name: string): string {
  const cleaned = name
    .normalize('NFC')
    .replace(/[^\p{L}\p{M}\p{N}\p{Extended_Pictographic}._-]+/gu, '-')
    .replace(/-{2,}/g, '-')
    .replace(/^[.-]+/, '')
    .replace(/[.\s-]+$/u, '');
  const chars = Array.from(cleaned);
  if (chars.length <= MAX_NAME_LENGTH) return cleaned || 'bestand';
  const dot = cleaned.lastIndexOf('.');
  const extension = dot > 0 && cleaned.length - dot <= 10 ? cleaned.slice(dot) : '';
  const base = Array.from(extension ? cleaned.slice(0, dot) : cleaned)
    .slice(0, MAX_NAME_LENGTH - Array.from(extension).length)
    .join('')
    .replace(/[.\s-]+$/u, '');
  return `${base}${extension}` || 'bestand';
}

export function evidenceArchiveName(item: NtaEvidenceItem): string {
  return `evidence/${safeName(item.id)}-${safeName(item.fileName)}`;
}

/** Keeps the file for this session and, in the desktop app, in app data. */
async function storeEvidenceBytes(hash: string, fileName: string, bytes: Uint8Array): Promise<string | undefined> {
  try {
    const { writeFile, mkdir, BaseDirectory } = await import('@tauri-apps/plugin-fs');
    const { appDataDir, join } = await import('@tauri-apps/api/path');
    await mkdir('evidence', { baseDir: BaseDirectory.AppData, recursive: true });
    const relative = `evidence/${hash}-${safeName(fileName)}`;
    await writeFile(relative, bytes, { baseDir: BaseDirectory.AppData });
    const path = await join(await appDataDir(), relative);
    // On disk now: the session does not need to keep the bytes as well.
    sessionFiles.delete(hash);
    return path;
  } catch {
    // Browser (or a failed write): the session keeps the file until export.
    sessionFiles.set(hash, bytes);
    return undefined;
  }
}

/** Creates a register entry for a file the adviser added. */
export async function createEvidenceItem(
  file: { name: string; bytes: Uint8Array; lastModified?: number },
  existing: NtaEvidenceItem[],
): Promise<NtaEvidenceItem> {
  checkEvidenceFile({ name: file.name, size: file.bytes.byteLength });
  // Reserve the id before the (slow) hash and write, so a second addition
  // running meanwhile picks another one; `releaseEvidenceIds` frees it.
  const id = nextEvidenceId(existing);
  reservedIds.add(id);
  const sha256 = await sha256Hex(file.bytes);
  const storedPath = await storeEvidenceBytes(sha256, file.name, file.bytes);
  const date = file.lastModified ? new Date(file.lastModified).toISOString().slice(0, 10) : undefined;
  return {
    id,
    kind: guessEvidenceKind(file.name),
    fileName: file.name,
    sha256,
    ...(date ? { date } : {}),
    ...(storedPath ? { storedPath } : {}),
  };
}

/** The file of an entry, when it is available on this machine and unchanged. */
export async function loadEvidenceBytes(item: NtaEvidenceItem): Promise<Uint8Array | null> {
  let bytes = sessionFiles.get(item.sha256) ?? null;
  if (!bytes && item.storedPath) {
    try {
      const { readFile } = await import('@tauri-apps/plugin-fs');
      bytes = await readFile(item.storedPath);
    } catch {
      bytes = null;
    }
  }
  if (!bytes) return null;
  return (await sha256Hex(bytes)) === item.sha256 ? bytes : null;
}

/** For tests: forget the files kept in memory. */
export function clearSessionEvidence(): void {
  sessionFiles.clear();
  reservedIds.clear();
}

/** Drops session bytes that no register entry refers to any more. */
export function pruneSessionEvidence(referenced: Iterable<NtaEvidenceItem>): void {
  const keep = new Set(Array.from(referenced, (item) => item.sha256));
  for (const hash of Array.from(sessionFiles.keys())) {
    if (!keep.has(hash)) sessionFiles.delete(hash);
  }
}

/**
 * Drops the session bytes of a removed entry, unless another entry still
 * refers to the same file (same hash).
 */
export function forgetSessionEvidence(sha256: string, remaining: Iterable<NtaEvidenceItem>): void {
  for (const item of remaining) if (item.sha256 === sha256) return;
  sessionFiles.delete(sha256);
}

/** For tests: the number of files kept in memory. */
export function sessionEvidenceCount(): number {
  return sessionFiles.size;
}

/** Puts bytes in the session store (e.g. after reading a file in the browser). */
export function rememberEvidenceBytes(hash: string, bytes: Uint8Array): void {
  sessionFiles.set(hash, bytes);
}
