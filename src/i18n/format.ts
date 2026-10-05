/** Number formatting in the active UI language (a BCP 47 tag such as `nl` or `en`). */
export function formatNumber(value: number | null | undefined, locale: string, digits = 0): string {
  if (value == null || !Number.isFinite(value)) return '–';
  return value.toLocaleString(locale, { minimumFractionDigits: digits, maximumFractionDigits: digits });
}

/** Non-breaking space between a number and its unit (ontwerp.md §5.3). */
export const NBSP = ' ';
/** Typographic minus for negative numbers. */
export const MINUS = '−';

export type QuantityKind =
  | 'beng' | 'bengStatus' | 'share' | 'tojuli' | 'energy' | 'gas' | 'area' | 'usableArea'
  | 'uValue' | 'rc' | 'psi' | 'qv10' | 'temperature' | 'temperatureMeasured' | 'money' | 'payback'
  | 'power' | 'number';

interface QuantitySpec {
  unit: string;
  digits: number;
  /** Unit before the number (currency). */
  prefix?: boolean;
  /** Thousands grouping (energy, money); off for small engineering values. */
  grouping?: boolean;
}

/**
 * Display rules per quantity: unit and decimals (docs/ui-redesign/ontwerp.md §5.3).
 * The report uses the same table, so screen and report round alike.
 */
export const QUANTITIES: Record<QuantityKind, QuantitySpec> = {
  beng: { unit: 'kWh/m²·jr', digits: 2, grouping: true },
  bengStatus: { unit: 'kWh/m²·jr', digits: 1, grouping: true },
  share: { unit: '%', digits: 1 },
  tojuli: { unit: 'K', digits: 2 },
  energy: { unit: 'kWh', digits: 0, grouping: true },
  gas: { unit: 'm³', digits: 0, grouping: true },
  area: { unit: 'm²', digits: 2, grouping: true },
  usableArea: { unit: 'm²', digits: 1, grouping: true },
  uValue: { unit: 'W/m²K', digits: 3 },
  rc: { unit: 'm²K/W', digits: 2 },
  psi: { unit: 'W/mK', digits: 3 },
  qv10: { unit: 'dm³/(s·m²)', digits: 2 },
  temperature: { unit: '°C', digits: 0 },
  temperatureMeasured: { unit: '°C', digits: 1 },
  money: { unit: '€', digits: 0, prefix: true, grouping: true },
  payback: { unit: 'jr', digits: 1 },
  power: { unit: 'kW', digits: 1, grouping: true },
  number: { unit: '', digits: 2, grouping: true },
};

/** English unit spellings where the Dutch abbreviation differs. */
const UNIT_EN: Record<string, string> = { 'kWh/m²·jr': 'kWh/m²·yr', jr: 'yr' };

/**
 * A number with its unit in the active locale: comma decimals in Dutch, a non-breaking
 * space before the unit and a typographic minus. `null`/non-finite values render as "–".
 */
export function formatQuantity(
  value: number | null | undefined,
  kind: QuantityKind,
  locale: string,
  options: { digits?: number; unit?: boolean } = {},
): string {
  if (value == null || !Number.isFinite(value)) return '–';
  const spec = QUANTITIES[kind];
  const digits = options.digits ?? spec.digits;
  const formatted = Math.abs(value).toLocaleString(locale, {
    minimumFractionDigits: digits,
    maximumFractionDigits: digits,
    useGrouping: spec.grouping ?? false,
  });
  const rounded = Number(Math.abs(value).toFixed(digits));
  const sign = value < 0 && rounded !== 0 ? MINUS : '';
  if (options.unit === false || !spec.unit) return `${sign}${formatted}`;
  const unit = locale.startsWith('nl') ? spec.unit : (UNIT_EN[spec.unit] ?? spec.unit);
  if (spec.prefix) return `${sign}${unit}${NBSP}${formatted}`;
  return `${sign}${formatted}${NBSP}${unit}`;
}

/**
 * Parse a user-typed decimal that may use a comma or a point as decimal separator
 * (and thin/normal spaces or points as thousands separators in Dutch input such as
 * "1.234,5"). Returns `null` for empty input and `NaN` for text that is not a number.
 */
export function parseDecimal(text: string): number | null {
  const trimmed = text.replace(/[\s  ]/g, '').replace(MINUS, '-');
  if (trimmed === '') return null;
  let normalized = trimmed;
  const hasComma = normalized.includes(',');
  const hasPoint = normalized.includes('.');
  if (hasComma && hasPoint) {
    // The separator that appears last is the decimal separator.
    if (normalized.lastIndexOf(',') > normalized.lastIndexOf('.')) {
      normalized = normalized.replace(/\./g, '').replace(',', '.');
    } else {
      normalized = normalized.replace(/,/g, '');
    }
  } else if (hasComma) {
    if ((normalized.match(/,/g) ?? []).length > 1) return Number.NaN;
    normalized = normalized.replace(',', '.');
  }
  if (!/^[-+]?(\d+\.?\d*|\.\d+)(e[-+]?\d+)?$/i.test(normalized)) return Number.NaN;
  return Number(normalized);
}

export const KERNEL_CODE_PREFIXES = [
  'nta.gap.', 'nta.warning.', 'kernel.issue.', 'opname.issue.', 'opname.warning.', 'mwa.issue.', 'registration.issue.',
];

/**
 * Translated label of a kernel code (gap, warning, issue). The given prefixes are tried
 * first, so a context-specific text wins; then every kernel-code prefix, so a code that
 * only has a general label still shows translated. `known` is false when no translation
 * exists; the caller then shows the bare code.
 */
export function kernelCodeLabel(
  t: (key: string, options?: Record<string, unknown>) => string,
  code: string,
  prefixes: string[] = KERNEL_CODE_PREFIXES,
): { text: string; known: boolean } {
  for (const prefix of [...prefixes, ...KERNEL_CODE_PREFIXES.filter((item) => !prefixes.includes(item))]) {
    const key = `${prefix}${code}`;
    const text = t(key);
    if (text !== key) return { text, known: true };
  }
  return { text: code, known: false };
}
