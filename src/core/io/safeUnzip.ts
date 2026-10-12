import { unzipSync, type Unzipped } from 'fflate';

/** Limits for reading an imported archive (UNIEC3, VABI EPA). */
export interface UnzipLimits {
  /** Largest single entry, by its declared size. */
  maxEntryBytes: number;
  /** Largest total of the entries that are read. */
  maxTotalBytes: number;
}

/** Project data in an UNIEC3 or VABI archive is a few MB at most. */
export const IMPORT_UNZIP_LIMITS: UnzipLimits = {
  maxEntryBytes: 64 * 1024 * 1024,
  maxTotalBytes: 256 * 1024 * 1024,
};

/** An archive whose entries are larger than an import can need (a zip bomb, or not project data). */
export class ArchiveTooLargeError extends Error {
  constructor(readonly entry: string, readonly declaredBytes: number) {
    super(`Het archief is te groot om in te lezen (${entry}: ${declaredBytes} bytes uitgepakt).`);
    this.name = 'ArchiveTooLargeError';
  }
}

/**
 * Unzips only the entries `accept` wants, and refuses the archive before
 * inflating when a wanted entry (or the wanted entries together) declare
 * more bytes than the limits allow. A small archive that unpacks to
 * gigabytes therefore never reaches memory.
 */
export function safeUnzip(
  data: Uint8Array,
  accept: (name: string) => boolean,
  limits: UnzipLimits = IMPORT_UNZIP_LIMITS,
): Unzipped {
  let total = 0;
  return unzipSync(data, {
    filter(file) {
      if (!accept(file.name)) return false;
      if (file.originalSize > limits.maxEntryBytes) throw new ArchiveTooLargeError(file.name, file.originalSize);
      total += file.originalSize;
      if (total > limits.maxTotalBytes) throw new ArchiveTooLargeError(file.name, total);
      return true;
    },
  });
}
