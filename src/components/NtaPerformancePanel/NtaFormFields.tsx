import type { ReactNode } from 'react';

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

export interface FieldProps {
  draft: Draft;
  path: Path;
  label: string;
  onChange: (path: Path, value: unknown) => void;
}

/**
 * A number input. Clearing writes `null` (a blank the kernel reports at its
 * path), or with `optional` drops the member so the kernel default applies.
 */
export function NumberField({ draft, path, label, onChange, step = 'any', optional = false }:
  FieldProps & { step?: string; optional?: boolean }) {
  const value = read(draft, path);
  return <label>{label}
    <input type="number" step={step} value={typeof value === 'number' ? value : ''}
      onChange={(event) => onChange(path, event.target.value === '' ? (optional ? undefined : null) : Number(event.target.value))} />
  </label>;
}

export function TextField({ draft, path, label, onChange }: FieldProps) {
  const value = read(draft, path);
  return <label>{label}
    <input value={typeof value === 'string' ? value : ''} onChange={(event) => onChange(path, event.target.value)} />
  </label>;
}

export function SelectField({ draft, path, label, onChange, options }: FieldProps & { options: Array<[string, string]> }) {
  const value = read(draft, path);
  return <label>{label}
    <select value={value == null ? '' : String(value)} onChange={(event) => onChange(path, event.target.value || null)}>
      <option value="">—</option>
      {options.map(([key, text]) => <option key={key} value={key}>{text}</option>)}
    </select>
  </label>;
}

export function CheckField({ draft, path, label, onChange }: FieldProps) {
  return <label className="nta-form-check">
    <input type="checkbox" checked={read(draft, path) === true} onChange={(event) => onChange(path, event.target.checked)} />
    {label}
  </label>;
}

/** Optional boolean: yes, no or unknown (null). */
export function TriStateField({ draft, path, label, onChange, yes, no }: FieldProps & { yes: string; no: string }) {
  const value = read(draft, path);
  return <label>{label}
    <select value={value === true ? 'true' : value === false ? 'false' : ''}
      onChange={(event) => onChange(path, event.target.value === '' ? null : event.target.value === 'true')}>
      <option value="">—</option>
      <option value="true">{yes}</option>
      <option value="false">{no}</option>
    </select>
  </label>;
}

export function Section({ title, children }: { title: string; children: ReactNode }) {
  return <fieldset className="nta-form-section"><legend>{title}</legend><div className="nta-form-grid">{children}</div></fieldset>;
}
