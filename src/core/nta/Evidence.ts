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

export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  // Copy into a buffer of this realm: under jsdom on Node 20 a view that came
  // from another realm (TextEncoder, FileReader) is rejected by WebCrypto.
  const copy = new Uint8Array(new ArrayBuffer(bytes.byteLength));
  copy.set(bytes);
  const digest = await crypto.subtle.digest('SHA-256', copy);
  return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, '0')).join('');
}

export function nextEvidenceId(existing: NtaEvidenceItem[]): string {
  const used = new Set(existing.map((item) => item.id));
  let index = existing.length + 1;
  while (used.has(`ev-${index}`)) index += 1;
  return `ev-${index}`;
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

function safeName(name: string): string {
  return name.replace(/[^\p{L}\p{N}._-]+/gu, '-').slice(0, 80) || 'bestand';
}

export function evidenceArchiveName(item: NtaEvidenceItem): string {
  return `evidence/${safeName(item.id)}-${safeName(item.fileName)}`;
}

/** Keeps the file for this session and, in the desktop app, in app data. */
async function storeEvidenceBytes(hash: string, fileName: string, bytes: Uint8Array): Promise<string | undefined> {
  sessionFiles.set(hash, bytes);
  try {
    const { writeFile, mkdir, BaseDirectory } = await import('@tauri-apps/plugin-fs');
    const { appDataDir, join } = await import('@tauri-apps/api/path');
    await mkdir('evidence', { baseDir: BaseDirectory.AppData, recursive: true });
    const relative = `evidence/${hash}-${safeName(fileName)}`;
    await writeFile(relative, bytes, { baseDir: BaseDirectory.AppData });
    return await join(await appDataDir(), relative);
  } catch {
    return undefined;
  }
}

/** Creates a register entry for a file the adviser added. */
export async function createEvidenceItem(
  file: { name: string; bytes: Uint8Array; lastModified?: number },
  existing: NtaEvidenceItem[],
): Promise<NtaEvidenceItem> {
  const sha256 = await sha256Hex(file.bytes);
  const storedPath = await storeEvidenceBytes(sha256, file.name, file.bytes);
  const date = file.lastModified ? new Date(file.lastModified).toISOString().slice(0, 10) : undefined;
  return {
    id: nextEvidenceId(existing),
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
}

/** Puts bytes in the session store (e.g. after reading a file in the browser). */
export function rememberEvidenceBytes(hash: string, bytes: Uint8Array): void {
  sessionFiles.set(hash, bytes);
}
