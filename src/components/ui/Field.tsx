import {
  cloneElement, isValidElement, useEffect, useId, useRef, useState,
  type ChangeEvent, type InputHTMLAttributes, type KeyboardEvent, type MutableRefObject, type ReactElement, type ReactNode,
  type SelectHTMLAttributes,
} from 'react';
import { useI18n } from '../../i18n/i18n';
import { parseDecimal } from '../../i18n/format';
import { cx } from './Button';

// ── Field ────────────────────────────────────────────────────────────

export interface FieldProps {
  label: ReactNode;
  /** Norm reference shown right of the label, e.g. "9.3.2". */
  reference?: string;
  hint?: ReactNode;
  error?: ReactNode;
  /** Kernel path of the input; used later by "Ga naar" (F3). */
  path?: string;
  /** Explicit id for the control; generated when omitted. */
  id?: string;
  span?: 1 | 2 | 3 | 4;
  className?: string;
  /** A single control element; it receives `id`, `aria-describedby` and `aria-invalid`. */
  children: ReactElement;
}

/**
 * Label + control + hint or error. The error is linked with `aria-describedby`
 * and announced with `role="alert"`.
 */
export function Field({ label, reference, hint, error, path, id, span, className, children }: FieldProps) {
  const generated = useId();
  const childId = isValidElement<{ id?: string }>(children) ? children.props.id : undefined;
  const controlId = id ?? childId ?? `f-${generated}`;
  const hintId = `${controlId}-hint`;
  const errorId = `${controlId}-error`;
  const describedBy = [hint ? hintId : null, error ? errorId : null].filter(Boolean).join(' ') || undefined;
  const control = isValidElement(children)
    ? cloneElement(children as ReactElement<Record<string, unknown>>, {
      id: controlId,
      'aria-describedby': describedBy,
      'aria-invalid': error ? true : undefined,
    })
    : children;
  return (
    <div className={cx('ui-field', span && span > 1 && `ui-field--span${span}`, className)} data-path={path}>
      <label className="ui-field__label" htmlFor={controlId}>
        <span>{label}</span>
        {reference && <Ref>{reference}</Ref>}
      </label>
      {control}
      {error
        ? <span id={errorId} className="ui-field__error" role="alert">{error}</span>
        : hint && <span id={hintId} className="ui-field__hint">{hint}</span>}
    </div>
  );
}

/** Norm or source reference in muted text: "§ 9.3.2". */
export function Ref({ children }: { children: ReactNode }) {
  return <span className="ui-ref">§&nbsp;{children}</span>;
}

// ── TextInput ────────────────────────────────────────────────────────

export interface TextInputProps extends InputHTMLAttributes<HTMLInputElement> {
  unit?: ReactNode;
}

export function TextInput({ unit, className, ...rest }: TextInputProps) {
  return (
    <span className={cx('ui-input', rest.disabled && 'ui-input--disabled', className)}>
      <input className="ui-input__control" {...rest} />
      {unit && <><span className="visually-hidden">, {unit}</span><span className="ui-input__unit" aria-hidden="true">{unit}</span></>}
    </span>
  );
}

// ── NumberInput ──────────────────────────────────────────────────────

export interface NumberInputProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, 'value' | 'onChange' | 'type' | 'step'> {
  value: number | null | undefined;
  /**
   * Called with the parsed number, or with the empty value: `undefined` when
   * `optional` (the key is dropped), otherwise `null` (same as NumberField).
   */
  onChange: (value: number | null | undefined) => void;
  unit?: ReactNode;
  optional?: boolean;
  step?: number;
  /** Decimals used to show the value when the field is not being edited. */
  digits?: number;
  /** Read-only value derived by the kernel (dashed border, "afgeleid"). */
  derived?: boolean;
  /** Placeholder showing the forfait kernel value when the field is empty. */
  forfait?: number | null;
}

function display(value: number | null | undefined, locale: string, digits?: number): string {
  if (value == null || !Number.isFinite(value)) return '';
  // Latin digits in every locale: the field must be editable, and parseDecimal reads
  // Latin input (fa/ar would otherwise show ۱۲۳۴٫۵ with direction marks).
  return value.toLocaleString(latinDigits(locale), {
    useGrouping: false,
    minimumFractionDigits: digits ?? 0,
    maximumFractionDigits: digits ?? 10,
  });
}

function latinDigits(locale: string): string {
  return locale.includes('-u-') ? locale : `${locale}-u-nu-latn`;
}

/** Clamp a stepped value to the field's finite `min`/`max` attributes, as a native number input does. */
function clampToBounds(next: number, min: unknown, max: unknown): number {
  const lower = min == null || min === '' ? Number.NaN : Number(min);
  const upper = max == null || max === '' ? Number.NaN : Number(max);
  let result = next;
  if (Number.isFinite(lower) && result < lower) result = lower;
  if (Number.isFinite(upper) && result > upper) result = upper;
  return result;
}

/** ARIA value attributes for a text field that acts as a spinbutton. */
function spinbuttonAria(text: string, min: unknown, max: unknown) {
  const now = parseDecimal(text);
  const lower = min == null || min === '' ? Number.NaN : Number(min);
  const upper = max == null || max === '' ? Number.NaN : Number(max);
  return {
    'aria-valuenow': now != null && Number.isFinite(now) ? now : undefined,
    'aria-valuemin': Number.isFinite(lower) ? lower : undefined,
    'aria-valuemax': Number.isFinite(upper) ? upper : undefined,
  };
}

/**
 * Decimal input that accepts a comma or a point, shows the value in the UI
 * locale and keeps the unit inside the field. No spinners: ↑/↓ change the
 * value by `step`, Shift multiplies the step by ten.
 */
export function NumberInput({
  value, onChange, unit, optional = false, step = 1, digits, derived = false, forfait,
  className, disabled, readOnly, onBlur, onFocus, onKeyDown, placeholder, ...rest
}: NumberInputProps) {
  const { locale } = useI18n();
  const [text, setText] = useState(() => display(value, locale, digits));
  const [invalid, setInvalid] = useState(false);
  const editing = useRef(false);

  // Follow outside changes (undo, another field) unless the user is typing.
  useEffect(() => {
    if (!editing.current) {
      setText(display(value, locale, digits));
      setInvalid(false);
    }
  }, [value, locale, digits]);

  const emit = (raw: string) => {
    const parsed = parseDecimal(raw);
    if (parsed === null) {
      setInvalid(false);
      onChange(optional ? undefined : null);
      return;
    }
    if (Number.isNaN(parsed)) {
      setInvalid(true);
      return;
    }
    setInvalid(false);
    onChange(parsed);
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    onKeyDown?.(event);
    if (event.defaultPrevented || readOnly || derived) return;
    if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') return;
    event.preventDefault();
    const current = parseDecimal(text);
    const base = current == null || Number.isNaN(current) ? (value ?? 0) : current;
    const delta = (event.key === 'ArrowUp' ? 1 : -1) * step * (event.shiftKey ? 10 : 1);
    // Round to the step's precision so 0.1 + 0.2 stays 0.3.
    const decimals = Math.max(0, (String(step).split('.')[1] ?? '').length);
    const next = clampToBounds(Number((base + delta).toFixed(decimals)), rest.min, rest.max);
    setText(display(next, locale));
    setInvalid(false);
    onChange(next);
  };

  const forfaitText = forfait != null && Number.isFinite(forfait) ? display(forfait, locale, digits) : undefined;
  return (
    <span className={cx(
      'ui-input', 'ui-input--number',
      invalid && 'ui-input--invalid',
      derived && 'ui-input--derived',
      disabled && 'ui-input--disabled',
      className,
    )}>
      <input
        {...rest}
        className="ui-input__control"
        type="text"
        inputMode="decimal"
        autoComplete="off"
        value={text}
        placeholder={placeholder ?? forfaitText}
        disabled={disabled}
        readOnly={readOnly || derived}
        aria-invalid={invalid || rest['aria-invalid'] ? true : undefined}
        onFocus={(event) => { editing.current = true; onFocus?.(event); }}
        onBlur={(event) => {
          editing.current = false;
          if (!invalid) setText(display(value, locale, digits));
          onBlur?.(event);
        }}
        onChange={(event) => { setText(event.target.value); emit(event.target.value); }}
        onKeyDown={handleKeyDown}
      />
      {unit && <><span className="visually-hidden">, {unit}</span><span className="ui-input__unit" aria-hidden="true">{unit}</span></>}
    </span>
  );
}

// ── DecimalInput ─────────────────────────────────────────────────────

export interface DecimalInputProps
  extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type' | 'value' | 'defaultValue' | 'step'> {
  /** The value as a raw `type="number"` input took it: a number or a point-decimal string. */
  value?: number | string | null;
  step?: number | string;
}

function decimalText(value: number | string | null | undefined, locale: string): string {
  if (value == null || value === '') return '';
  const number = typeof value === 'number' ? value : parseDecimal(String(value));
  if (number == null || Number.isNaN(number) || !Number.isFinite(number)) return String(value);
  return display(number, locale);
}

/**
 * Drop-in for a raw `<input type="number">` with the same `value`/`onChange`
 * contract, but accepting a decimal comma (Dutch users type "0,35"). The
 * handler receives an event whose `target.value` is the point-decimal string
 * ("0.35"), or "" when the field is cleared; text that is not a number is
 * marked invalid and not passed on. Keeps the spinbutton role.
 */
export function DecimalInput({
  value, onChange, step, className, onBlur, onFocus, onKeyDown, readOnly, ...rest
}: DecimalInputProps) {
  const { locale } = useI18n();
  const [text, setText] = useState(() => decimalText(value, locale));
  const [invalid, setInvalid] = useState(false);
  // The parent did not take the last typed number (a rejected prefix such as the
  // "0" of "0,04" when only values above zero are accepted).
  const [rejected, setRejected] = useState(false);
  const editing = useRef(false);
  // The number last passed to onChange (null for a cleared field). When the
  // parent does not take it, the field keeps the typed text while it has focus
  // (overwriting it mid-typing would turn "0,04" into a different number) and
  // marks it invalid; on leaving the field it shows the parent's value again.
  const emitted = useRef<{ number: number | null } | undefined>(undefined);

  useEffect(() => {
    if (!editing.current) {
      setText(decimalText(value, locale));
      setInvalid(false);
    }
  }, [value, locale]);

  useEffect(() => {
    const sent = emitted.current;
    if (!sent) return;
    emitted.current = undefined;
    const current = value == null || value === '' ? null
      : typeof value === 'number' ? value : parseDecimal(String(value));
    const taken = sent.number === null ? current === null : current === sent.number;
    if (taken) { setRejected(false); return; }
    if (editing.current) setRejected(true);
    else setText(decimalText(value, locale));
  });

  const pass = (event: ChangeEvent<HTMLInputElement>, normalized: string) => {
    emitted.current = { number: normalized === '' ? null : Number(normalized) };
    if (!onChange) return;
    const target = { value: normalized, name: event.target.name, id: event.target.id };
    onChange({ ...event, target, currentTarget: target } as unknown as ChangeEvent<HTMLInputElement>);
  };

  const stepValue = Number(step) > 0 ? Number(step) : 1;
  return (
    <input
      role="spinbutton"
      {...spinbuttonAria(text, rest.min, rest.max)}
      {...rest}
      className={cx('decimal-input', className, (invalid || rejected) && 'ui-input--invalid')}
      type="text"
      inputMode="decimal"
      autoComplete="off"
      value={text}
      readOnly={readOnly}
      aria-invalid={invalid || rejected || rest['aria-invalid'] ? true : undefined}
      onFocus={(event) => { editing.current = true; onFocus?.(event); }}
      onBlur={(event) => {
        editing.current = false;
        if (!invalid || rejected) {
          setText(decimalText(value, locale));
          setInvalid(false);
        }
        setRejected(false);
        onBlur?.(event);
      }}
      onChange={(event) => {
        const raw = event.target.value;
        setText(raw);
        const parsed = parseDecimal(raw);
        if (parsed === null) { setInvalid(false); pass(event, ''); return; }
        if (Number.isNaN(parsed)) { setInvalid(true); setRejected(false); return; }
        setInvalid(false);
        pass(event, String(parsed));
      }}
      onKeyDown={(event) => {
        onKeyDown?.(event);
        if (event.defaultPrevented || readOnly || rest.disabled) return;
        if (event.key !== 'ArrowUp' && event.key !== 'ArrowDown') return;
        event.preventDefault();
        const current = parseDecimal(text);
        const base = current == null || Number.isNaN(current) ? Number(value) || 0 : current;
        const decimals = Math.max(0, (String(stepValue).split('.')[1] ?? '').length);
        const next = clampToBounds(
          Number((base + (event.key === 'ArrowUp' ? 1 : -1) * stepValue * (event.shiftKey ? 10 : 1)).toFixed(decimals)),
          rest.min, rest.max,
        );
        setText(display(next, locale));
        setInvalid(false);
        emitted.current = { number: next };
        if (onChange) {
          const target = { value: String(next), name: event.currentTarget.name, id: event.currentTarget.id };
          onChange({ target, currentTarget: target } as unknown as ChangeEvent<HTMLInputElement>);
        }
      }}
    />
  );
}

// ── Select ───────────────────────────────────────────────────────────

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps extends Omit<SelectHTMLAttributes<HTMLSelectElement>, 'onChange'> {
  options: SelectOption[];
  onChange: (value: string) => void;
  /** Option shown first with an empty value. */
  placeholder?: string;
}

/** Native select in a themed wrapper: keyboard- and webview-safe. */
export function Select({ options, onChange, placeholder, className, value, ...rest }: SelectProps) {
  const current = options.find((option) => option.value === value);
  return (
    <span className={cx('ui-select', rest.disabled && 'ui-input--disabled', className)}>
      <select
        {...rest}
        value={value}
        title={current?.label}
        className="ui-select__control"
        onChange={(event) => onChange(event.target.value)}
      >
        {placeholder !== undefined && <option value="">{placeholder}</option>}
        {options.map((option) => (
          <option key={option.value} value={option.value} disabled={option.disabled}>{option.label}</option>
        ))}
      </select>
    </span>
  );
}

// ── Segmented / TriState ─────────────────────────────────────────────

export interface SegmentedProps<T extends string> {
  options: Array<{ value: T; label: ReactNode; disabled?: boolean }>;
  value: T | null | undefined;
  onChange: (value: T) => void;
  'aria-label'?: string;
  'aria-labelledby'?: string;
  id?: string;
  size?: 'sm' | 'md';
  className?: string;
}

/** 2–4 exclusive choices as a radio group; arrow keys move the selection. */
export function Segmented<T extends string>({
  options, value, onChange, id, size = 'md', className, ...aria
}: SegmentedProps<T>) {
  const refs = useRef<Array<HTMLButtonElement | null>>([]);
  const enabled = options.map((option, index) => ({ option, index })).filter(({ option }) => !option.disabled);
  const selectedIndex = options.findIndex((option) => option.value === value);
  const focusIndex = selectedIndex >= 0 ? selectedIndex : enabled[0]?.index ?? 0;

  const move = (from: number, direction: 1 | -1) => {
    const position = enabled.findIndex(({ index }) => index === from);
    const next = enabled[(position + direction + enabled.length) % enabled.length];
    if (!next) return;
    onChange(next.option.value);
    refs.current[next.index]?.focus();
  };

  return (
    <div id={id} role="radiogroup" className={cx('ui-seg', size === 'sm' && 'ui-seg--sm', className)} {...aria}>
      {options.map((option, index) => {
        const checked = option.value === value;
        return (
          <button
            key={option.value}
            ref={(element) => { refs.current[index] = element; }}
            type="button"
            role="radio"
            aria-checked={checked}
            tabIndex={index === focusIndex ? 0 : -1}
            disabled={option.disabled}
            className={cx('ui-seg__item', checked && 'ui-seg__item--on')}
            onClick={() => onChange(option.value)}
            onKeyDown={(event) => {
              if (event.key === 'ArrowRight' || event.key === 'ArrowDown') { event.preventDefault(); move(index, 1); }
              if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') { event.preventDefault(); move(index, -1); }
            }}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
}

export type TriStateValue = boolean | null;

export interface TriStateProps {
  value: TriStateValue | undefined;
  onChange: (value: TriStateValue) => void;
  'aria-label'?: string;
  'aria-labelledby'?: string;
  id?: string;
}

/** Ja / Nee / Onbekend as a segmented control instead of a select. */
export function TriState({ value, onChange, ...rest }: TriStateProps) {
  const { t } = useI18n();
  const key = value === true ? 'yes' : value === false ? 'no' : 'unknown';
  return (
    <Segmented
      {...rest}
      value={key}
      onChange={(next) => onChange(next === 'yes' ? true : next === 'no' ? false : null)}
      options={[
        { value: 'yes', label: t('ui.yes') },
        { value: 'no', label: t('ui.no') },
        { value: 'unknown', label: t('ui.unknown') },
      ]}
    />
  );
}

// ── Check / Switch ───────────────────────────────────────────────────

export interface CheckProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type' | 'onChange'> {
  label: ReactNode;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function Check({ label, checked, onChange, className, ...rest }: CheckProps) {
  return (
    <label className={cx('ui-check', rest.disabled && 'ui-check--disabled', className)}>
      <input type="checkbox" checked={checked} onChange={(event) => onChange(event.target.checked)} {...rest} />
      <span className="ui-check__box" aria-hidden="true" />
      <span>{label}</span>
    </label>
  );
}

export function Switch({ label, checked, onChange, className, ...rest }: CheckProps) {
  return (
    <label className={cx('ui-switch', rest.disabled && 'ui-check--disabled', className)}>
      <input type="checkbox" role="switch" checked={checked} onChange={(event) => onChange(event.target.checked)} {...rest} />
      <span className="ui-switch__track" aria-hidden="true" />
      <span>{label}</span>
    </label>
  );
}

// ── FileButton ───────────────────────────────────────────────────────

export interface FileButtonProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'type' | 'children'> {
  /** Button text; defaults to the translated "Bestand kiezen…". */
  label?: ReactNode;
  /** Text shown next to the button; defaults to the chosen file names or "Geen bestand gekozen". */
  status?: ReactNode;
  icon?: ReactNode;
  /** Access to the hidden input, e.g. to reset its value after reading the file. */
  inputRef?: MutableRefObject<HTMLInputElement | null>;
}

/**
 * Translated replacement for the native file input, whose "Browse…" /
 * "No file selected" text follows the OS language instead of the app.
 * The real input stays in the DOM (visually hidden) for forms and tests.
 */
export function FileButton({ label, status, icon, className, onChange, disabled, id, inputRef, ...rest }: FileButtonProps) {
  const { t } = useI18n();
  const input = useRef<HTMLInputElement | null>(null);
  const generated = useId();
  const inputId = id ?? `file-${generated}`;
  const [names, setNames] = useState<string[]>([]);
  return (
    <span className={cx('ui-file', className)}>
      <input
        {...rest}
        id={inputId}
        ref={(element) => {
          input.current = element;
          if (inputRef) inputRef.current = element;
        }}
        type="file"
        className="visually-hidden"
        tabIndex={-1}
        disabled={disabled}
        onChange={(event) => {
          setNames(Array.from(event.target.files ?? []).map((file) => file.name));
          onChange?.(event);
        }}
      />
      <button type="button" className="ui-btn ui-btn--secondary ui-btn--sm" disabled={disabled}
        aria-controls={inputId} onClick={() => input.current?.click()}>
        {icon}{label ?? t('ui.chooseFile')}
      </button>
      <span className="ui-file__status">{status ?? (names.length > 0 ? names.join(', ') : t('ui.noFileChosen'))}</span>
    </span>
  );
}
