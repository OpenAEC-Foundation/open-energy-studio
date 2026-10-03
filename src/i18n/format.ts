/** Number formatting in the active UI language (a BCP 47 tag such as `nl` or `en`). */
export function formatNumber(value: number | null | undefined, locale: string, digits = 0): string {
  if (value == null || !Number.isFinite(value)) return '–';
  return value.toLocaleString(locale, { minimumFractionDigits: digits, maximumFractionDigits: digits });
}

const KERNEL_CODE_PREFIXES = ['nta.gap.', 'nta.warning.', 'kernel.issue.', 'opname.issue.', 'mwa.issue.', 'registration.issue.'];

/**
 * Translated label of a kernel code (gap, warning, issue). `known` is false when no
 * translation exists; the caller then shows the bare code.
 */
export function kernelCodeLabel(
  t: (key: string, options?: Record<string, unknown>) => string,
  code: string,
  prefixes: string[] = KERNEL_CODE_PREFIXES,
): { text: string; known: boolean } {
  for (const prefix of prefixes) {
    const key = `${prefix}${code}`;
    const text = t(key);
    if (text !== key) return { text, known: true };
  }
  return { text: code, known: false };
}
