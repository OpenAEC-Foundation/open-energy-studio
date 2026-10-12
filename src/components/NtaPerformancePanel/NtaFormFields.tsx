import { createContext, useContext, useLayoutEffect, useRef, type ReactNode } from 'react';
import { NumberInput } from '../ui/Field';
import { formatKernelPath } from '../../core/nta/pathUtil';

// Field components of the NTA input form; the block is edited as plain JSON
// data and the Rust kernel is the validator.

export type Draft = Record<string, unknown>;
export type Path = Array<string | number>;

export function read(source: unknown, path: Path): unknown {
  return path.reduce<unknown>((value, key) => (value == null ? undefined : (value as Record<string | number, unknown>)[key]), source);
}

export function write(source: Draft, path: Path, value: unknown): Draft {
  const clone = structuredClone(source) as Record<string | number, unknown>;
  let cursor = clone;
  path.slice(0, -1).forEach((key, index) => {
    const next = cursor[key];
    if (next == null || typeof next !== 'object') {
      cursor[key] = typeof path[index + 1] === 'number' ? [] : {};
    }
    cursor = cursor[key] as Record<string | number, unknown>;
  });
  cursor[path[path.length - 1]] = value;
  return clone as Draft;
}

/**
 * Kernel path prefix of the draft (`ntaCalculation` for the NTA input). When set,
 * every field carries `data-path` with its kernel path, so "Ga naar" can focus it.
 * The survey and maatwerkadvies forms reuse these fields without a prefix.
 */
const FieldPathPrefix = createContext<string | null>(null);
export const FieldPathPrefixProvider = FieldPathPrefix.Provider;

/** The kernel path of a draft path, e.g. `ntaCalculation.groundFloors[0].exposedPerimeterM`; null without prefix. */
export function useFieldPath(path: Path): string | null {
  const prefix = useContext(FieldPathPrefix);
  return prefix == null ? null : formatKernelPath([prefix, ...path]);
}

export interface FieldProps {
  draft: Draft;
  path: Path;
  label: string;
  onChange: (path: Path, value: unknown) => void;
}

/**
 * A number input that accepts a comma or a point and shows the value in the UI
 * locale (ontwerp §5.3). Clearing writes `null` (a blank the kernel reports at
 * its path), or with `optional` drops the member so the kernel default applies.
 * It keeps the spinbutton role of the former `type="number"` input.
 */
export function NumberField({ draft, path, label, onChange, step = 'any', optional = false, disabled = false, unit }:
  FieldProps & { step?: string; optional?: boolean; disabled?: boolean; unit?: ReactNode }) {
  const value = read(draft, path);
  const kernelPath = useFieldPath(path);
  const numeric = typeof value === 'number' ? value : null;
  const stepValue = step === 'any' ? 1 : Number(step) || 1;
  return <label data-path={kernelPath ?? undefined}>{label}
    <NumberInput value={numeric} step={stepValue} optional={optional} disabled={disabled} unit={unit}
      role="spinbutton" aria-valuenow={numeric ?? undefined}
      onChange={(next) => onChange(path, next)} />
  </label>;
}

export function TextField({ draft, path, label, onChange }: FieldProps) {
  const value = read(draft, path);
  const kernelPath = useFieldPath(path);
  return <label data-path={kernelPath ?? undefined}>{label}
    <input value={typeof value === 'string' ? value : ''} onChange={(event) => onChange(path, event.target.value)} />
  </label>;
}

/**
 * A choice from `options`. With `fallback`, an absent value shows the option the kernel applies
 * when the field is left out (for example the designated edition) instead of "—", and there is
 * no empty option. With `required` (the tag of a choice whose handler writes the members of the
 * chosen variant) there is no "—" once a value is chosen: the handler would turn it into one of
 * the options anyway (review 58).
 */
export function SelectField({ draft, path, label, onChange, options, disabled = false, fallback, required = false }:
  FieldProps & { options: Array<[string, string]>; disabled?: boolean; fallback?: string; required?: boolean }) {
  const stored = read(draft, path);
  const value = stored ?? fallback;
  const kernelPath = useFieldPath(path);
  const selected = options.find(([key]) => key === String(value))?.[1];
  // Long option texts get two grid columns, and the full text as tooltip, so they are not cut off.
  const wide = options.some(([, text]) => text.length > WIDE_SELECT_CHARS);
  return <label className={wide ? 'nta-form-wide' : undefined} data-path={kernelPath ?? undefined}>{label}
    <select value={value == null ? '' : String(value)} title={selected} disabled={disabled}
      onChange={(event) => onChange(path, event.target.value || null)}>
      {fallback == null && (!required || value == null) && <option value="">—</option>}
      {options.map(([key, text]) => <option key={key} value={key}>{text}</option>)}
    </select>
  </label>;
}

export function CheckField({ draft, path, label, onChange, disabled = false }: FieldProps & { disabled?: boolean }) {
  const kernelPath = useFieldPath(path);
  return <label className="nta-form-check" data-path={kernelPath ?? undefined}>
    <input type="checkbox" checked={read(draft, path) === true} disabled={disabled} onChange={(event) => onChange(path, event.target.checked)} />
    {label}
  </label>;
}

/** Optional boolean: yes, no or unknown (null). */
export function TriStateField({ draft, path, label, onChange, yes, no }: FieldProps & { yes: string; no: string }) {
  const value = read(draft, path);
  const kernelPath = useFieldPath(path);
  return <label data-path={kernelPath ?? undefined}>{label}
    <select value={value === true ? 'true' : value === false ? 'false' : ''}
      onChange={(event) => onChange(path, event.target.value === '' ? null : event.target.value === 'true')}>
      <option value="">—</option>
      <option value="true">{yes}</option>
      <option value="false">{no}</option>
    </select>
  </label>;
}

/**
 * Option texts longer than this are cut off in one grid column (minimum
 * 200 px, about 26 characters after the padding and the arrow), so the field
 * takes two columns.
 */
export const WIDE_SELECT_CHARS = 26;

/**
 * Marks the label of every select whose longest option text is longer than
 * WIDE_SELECT_CHARS with `data-wide-select`, so CSS gives it two grid
 * columns. Done on the DOM after each render because many sections build
 * their own selects; selects in table cells keep their column.
 */
export function markWideSelects(root: HTMLElement | null) {
  if (!root) return;
  for (const select of Array.from(root.querySelectorAll('select'))) {
    const label = select.closest('label');
    if (!label || !root.contains(label) || select.closest('td, th')) continue;
    const longest = Math.max(0, ...Array.from(select.options).map((option) => (option.textContent ?? '').trim().length));
    label.toggleAttribute('data-wide-select', longest > WIDE_SELECT_CHARS);
  }
}

export function Section({ title, children, path, extraPaths }: {
  title: string; children: ReactNode; path?: string; extraPaths?: string[];
}) {
  const ref = useRef<HTMLFieldSetElement>(null);
  useLayoutEffect(() => markWideSelects(ref.current));
  // Lists inside a section can add selects, or change option texts, without a render of the
  // section itself.
  useLayoutEffect(() => {
    const root = ref.current;
    if (!root || typeof MutationObserver === 'undefined') return undefined;
    const observer = new MutationObserver(() => markWideSelects(root));
    observer.observe(root, { childList: true, characterData: true, subtree: true });
    return () => observer.disconnect();
  }, []);
  return <fieldset ref={ref} className="nta-form-section" data-path={path} data-paths={extraPaths?.length ? extraPaths.join(' ') : undefined}>
    <legend>{title}</legend><div className="nta-form-grid">{children}</div>
  </fieldset>;
}
