import { useId, useRef, type KeyboardEvent, type ReactNode } from 'react';
import { Check as CheckIcon } from 'lucide-react';
import { cx } from './Button';

// ── Tabs ─────────────────────────────────────────────────────────────

export interface TabItem<T extends string> {
  id: T;
  label: ReactNode;
  /** Small counter or status after the label. */
  badge?: ReactNode;
  disabled?: boolean;
}

export interface TabsProps<T extends string> {
  tabs: Array<TabItem<T>>;
  active: T;
  onChange: (id: T) => void;
  'aria-label': string;
  /** Prefix for the panel ids, so a panel can use `aria-labelledby`. */
  idPrefix?: string;
  className?: string;
}

/** Tabs for equivalent views; arrow keys move between tabs (WAI-ARIA tabs pattern). */
export function Tabs<T extends string>({ tabs, active, onChange, idPrefix, className, ...aria }: TabsProps<T>) {
  const generated = useId();
  const prefix = idPrefix ?? `tabs-${generated}`;
  const refs = useRef<Record<string, HTMLButtonElement | null>>({});
  const enabled = tabs.filter((tab) => !tab.disabled);

  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>, id: T) => {
    const index = enabled.findIndex((tab) => tab.id === id);
    let next: TabItem<T> | undefined;
    if (event.key === 'ArrowRight') next = enabled[(index + 1) % enabled.length];
    if (event.key === 'ArrowLeft') next = enabled[(index - 1 + enabled.length) % enabled.length];
    if (event.key === 'Home') next = enabled[0];
    if (event.key === 'End') next = enabled[enabled.length - 1];
    if (!next) return;
    event.preventDefault();
    onChange(next.id);
    refs.current[next.id]?.focus();
  };

  return (
    <div role="tablist" className={cx('ui-tabs', className)} aria-label={aria['aria-label']}>
      {tabs.map((tab) => {
        const selected = tab.id === active;
        return (
          <button
            key={tab.id}
            ref={(element) => { refs.current[tab.id] = element; }}
            type="button"
            role="tab"
            id={`${prefix}-tab-${tab.id}`}
            aria-selected={selected}
            aria-controls={`${prefix}-panel-${tab.id}`}
            tabIndex={selected ? 0 : -1}
            disabled={tab.disabled}
            className={cx('ui-tabs__tab', selected && 'ui-tabs__tab--on')}
            onClick={() => onChange(tab.id)}
            onKeyDown={(event) => onKeyDown(event, tab.id)}
          >
            {tab.label}
            {tab.badge}
          </button>
        );
      })}
    </div>
  );
}

// ── Stepper ──────────────────────────────────────────────────────────

export type StepStatus = 'todo' | 'current' | 'done' | 'warn' | 'error';

export interface StepItem<T extends string> {
  id: T;
  label: ReactNode;
  status: StepStatus;
  /** Accessible status text, e.g. "2 aandachtspunten". */
  statusText?: string;
}

export interface StepperProps<T extends string> {
  steps: Array<StepItem<T>>;
  onSelect?: (id: T) => void;
  'aria-label': string;
  className?: string;
}

/** Ordered sub-steps with a status per step. */
export function Stepper<T extends string>({ steps, onSelect, className, ...aria }: StepperProps<T>) {
  return (
    <ol className={cx('ui-stepper', className)} aria-label={aria['aria-label']}>
      {steps.map((step, index) => {
        const content = (
          <>
            <span className={cx('ui-stepper__no', `ui-stepper__no--${step.status}`)} aria-hidden="true">
              {step.status === 'done' ? <CheckIcon /> : index + 1}
            </span>
            <span className="ui-stepper__label">{step.label}</span>
            {step.statusText && <span className="visually-hidden">{step.statusText}</span>}
          </>
        );
        return (
          <li key={step.id} className={cx('ui-stepper__step', `ui-stepper__step--${step.status}`)}>
            {index > 0 && <span className={cx('ui-stepper__line', step.status === 'done' && 'ui-stepper__line--done')} aria-hidden="true" />}
            {onSelect
              ? (
                <button type="button" className="ui-stepper__btn" onClick={() => onSelect(step.id)}
                  aria-current={step.status === 'current' ? 'step' : undefined}>
                  {content}
                </button>
              )
              : <span className="ui-stepper__btn" aria-current={step.status === 'current' ? 'step' : undefined}>{content}</span>}
          </li>
        );
      })}
    </ol>
  );
}
