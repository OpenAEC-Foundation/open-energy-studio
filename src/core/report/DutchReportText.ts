import { nl } from '../../i18n/nl';
import { kernelCodeLabel } from '../../i18n/format';
import { escapeHtml } from './HtmlEscaping';

/*
 * The BRL 9500 documents (NTA 8800 calculation report, input dossier, project dossier,
 * maatwerkadvies report) are Dutch records for registration and advice, whatever the UI
 * language. Within them every number, date and code is Dutch too: decimal comma,
 * Dutch dates and translated codes, with the code kept as a reference.
 */

/** Number with a decimal comma and no thousands separator (unambiguous in tables). */
export function dutchNumber(value: number | null | undefined, digits = 0): string {
  if (value == null || !Number.isFinite(value)) return '—';
  return value.toLocaleString('nl-NL', { minimumFractionDigits: digits, maximumFractionDigits: digits, useGrouping: false });
}

/** Date and time in Dutch, e.g. "3 oktober 2026 14:05". */
export function dutchTimestamp(date: Date = new Date()): string {
  return date.toLocaleString('nl-NL', { day: 'numeric', month: 'long', year: 'numeric', hour: '2-digit', minute: '2-digit' });
}

/**
 * The generation time as `<time datetime="ISO">Dutch text</time>`: readable, and machine-exact so it
 * matches the ISO `generatedAt` of the dossier manifest.
 */
export function dutchTimeHtml(date: Date = new Date()): string {
  return `<time datetime="${date.toISOString()}">${escapeHtml(dutchTimestamp(date))}</time>`;
}

const PREFIXES = ['nta.gap.', 'nta.warning.', 'kernel.issue.', 'opname.issue.', 'mwa.issue.', 'registration.issue.'];

/** Escaped HTML of a code: the Dutch label with the code as a small reference, or the bare code. */
export function dutchCodeHtml(code: string): string {
  const { text, known } = kernelCodeLabel((key) => nl[key] ?? key, code, PREFIXES);
  return known ? `${escapeHtml(text)} <code>${escapeHtml(code)}</code>` : `<code>${escapeHtml(code)}</code>`;
}

/** A table cell with the translated code. */
export function dutchCodeCell(code: string): string {
  return `<td>${dutchCodeHtml(code)}</td>`;
}
