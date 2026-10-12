import { sha256Hex } from '../nta/Evidence';
import { ImportFormatError, type ImportResult, type ReadResult } from './importIssues';
import { IMPORT_UNZIP_LIMITS } from './safeUnzip';

/**
 * Largest file an import reads: an UNIEC3 or VABI file is a few MB, so a
 * larger one is the wrong file. Checked before reading, so a picked disk
 * image or video does not fill the memory of the tab.
 */
export const MAX_IMPORT_FILE_BYTES = IMPORT_UNZIP_LIMITS.maxTotalBytes;

/** Lets the adviser pick one file; rejects when the dialog closes without one. */
export function pickFile(accept: string): Promise<File> {
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = accept;
    input.onchange = () => {
      const file = input.files?.[0];
      if (file) resolve(file);
      else reject(new Error('Geen bestand geselecteerd'));
    };
    input.click();
  });
}

/**
 * Reads a picked file with `read`, and returns the project with the file's
 * name, its SHA-256 (of the exact bytes read), the version the file names
 * and the import issues, so the import record can say what came from which
 * file (BRL 9501 §4.3.1).
 */
export async function importFile(
  file: Pick<File, 'name' | 'size' | 'arrayBuffer'>,
  read: (bytes: Uint8Array) => ReadResult,
): Promise<ImportResult> {
  if (!Number.isFinite(file.size) || file.size > MAX_IMPORT_FILE_BYTES) {
    throw new ImportFormatError('file_too_large',
      `${file.name} is te groot om in te lezen (${file.size} bytes; hoogstens ${MAX_IMPORT_FILE_BYTES}).`,
      { file: file.name, bytes: file.size, max: MAX_IMPORT_FILE_BYTES });
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  const sha256 = await sha256Hex(bytes);
  const { project, issues, sourceVersion } = read(bytes);
  return { project, fileName: file.name, sha256, ...(sourceVersion ? { sourceVersion } : {}), issues: issues.list };
}

/** Saves bytes as a download (browser build and the desktop webview). */
export function downloadBytes(bytes: Uint8Array, fileName: string): void {
  const blob = new Blob([bytes.slice().buffer as ArrayBuffer], { type: 'application/zip' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = fileName;
  a.click();
  URL.revokeObjectURL(url);
}
