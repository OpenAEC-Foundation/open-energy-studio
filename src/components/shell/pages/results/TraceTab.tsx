/**
 * Rekenspoor (feedback 9 Oct 2026): the kernel's calculation trace, opened
 * from the result down to the input. Every line shows the quantity, the
 * formula with the values the kernel used, the result and the NTA 8800
 * formula number; the input lines show their source. Steps that a step uses
 * from elsewhere in the trace (the monthly Q_H;tr uses H_tr) open there.
 * Transmission, ventilation, the heat and cold balance and the installations come first; primary energy follows.
 */
import { useMemo, useState } from 'react';
import { useI18n } from '../../../../i18n/i18n';
import { formatNumber } from '../../../../i18n/format';
import { dutchSource } from '../../../../core/nta/OpnameValueText';
import type { KernelTraceStep, ProjectPerformanceAssessment } from '../../../../core/nta/KernelClient';
import { Button, Card } from '../../../ui';
import { useEnergy } from '../../../../context/EnergyContext';
import type { IProject } from '../../../../core/energy/types';

/** Project names behind the kernel element ids (`surface:wall-N:opaque`, `window:win-S`), as in the report. */
function elementNames(project: IProject): (id: string) => string {
  const names = new Map<string, string>();
  for (const zone of project.zones) {
    for (const surface of zone.surfaces) {
      names.set(`surface:${surface.id}`, surface.name);
      for (const window of surface.windows ?? []) names.set(`window:${window.id}`, `${window.name} (${surface.name})`);
    }
  }
  return (id) => names.get(id.replace(/:opaque$/, '')) ?? id;
}

/** The kernel writes values with a decimal point; Dutch shows a comma (formula numbers stay as they are). */
const valuesText = (values: string, locale: string) => (locale.startsWith('nl') ? values.replace(/(\d)\.(\d)/g, '$1,$2') : values);

/** Decimals of a result: three below 10, two below 1000, none above (kWh per year). */
const resultDigits = (value: number) => (Math.abs(value) < 10 ? 3 : Math.abs(value) < 1000 ? 2 : 0);

/** Every step of the trace by key, for the "uses" links. */
function index(steps: KernelTraceStep[], into = new Map<string, KernelTraceStep>()): Map<string, KernelTraceStep> {
  for (const step of steps) {
    into.set(step.key, step);
    if (step.children) index(step.children, into);
  }
  return into;
}

/** Whether a step or one of its children matches the search. */
function matches(step: KernelTraceStep, needle: string): boolean {
  if (!needle) return true;
  const text = `${step.quantity} ${step.key} ${step.source ?? ''} ${step.reference}`.toLowerCase();
  return text.includes(needle) || (step.children ?? []).some((child) => matches(child, needle));
}

function Step({ step, depth, needle, byKey, locale, t, name }: {
  step: KernelTraceStep; depth: number; needle: string; byKey: Map<string, KernelTraceStep>; locale: string;
  t: (key: string, params?: Record<string, string | number>) => string;
  name: (id: string) => string;
}) {
  const [usesOpen, setUsesOpen] = useState(false);
  if (!matches(step, needle)) return null;
  const children = step.children ?? [];
  const uses = (step.uses ?? []).map((key) => byKey.get(key)).filter((item): item is KernelTraceStep => item != null);
  const line = <>
    <span className="trace-step__quantity">{name(step.quantity)}</span>
    <span className="trace-step__formula">{step.formula}</span>
    <span className="trace-step__values">= {valuesText(step.values, locale)}</span>
    <span className="trace-step__result">= {formatNumber(step.result, locale, resultDigits(step.result))} {step.unit}</span>
    <span className="trace-step__ref">{step.reference}</span>
  </>;
  if (children.length === 0 && uses.length === 0) {
    return <div className="trace-step trace-step--leaf" data-key={step.key}>
      <div className="trace-step__line">{line}</div>
      {step.source && <div className="trace-step__source">{t('trace.source')}: {dutchSource(step.source)}</div>}
    </div>;
  }
  return <details className="trace-step" data-key={step.key} open={depth < 1 || (needle !== '' && children.some((child) => matches(child, needle)))}>
    <summary className="trace-step__line">{line}</summary>
    {step.source && <div className="trace-step__source">{t('trace.source')}: {dutchSource(step.source)}</div>}
    {uses.length > 0 && <div className="trace-step__uses">
      <button type="button" className="link-button" aria-expanded={usesOpen} onClick={() => setUsesOpen((open) => !open)}>
        {t('trace.uses', { list: uses.map((item) => name(item.quantity)).join(', ') })}</button>
      {usesOpen && uses.map((item) => <Step key={item.key} step={item} depth={depth + 1} needle="" byKey={byKey} locale={locale} t={t} name={name} />)}
    </div>}
    {children.map((child) => <Step key={child.key} step={child} depth={depth + 1} needle={needle} byKey={byKey} locale={locale} t={t} name={name} />)}
  </details>;
}

export function TraceTab({ assessment }: { assessment: ProjectPerformanceAssessment }) {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const name = useMemo(() => elementNames(state.project), [state.project]);
  const trace = useMemo(() => assessment.performance?.trace ?? [], [assessment]);
  const byKey = useMemo(() => index(trace), [trace]);
  const [query, setQuery] = useState('');
  const needle = query.trim().toLowerCase();
  const download = () => {
    const blob = new Blob([JSON.stringify(trace, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'rekenspoor.json';
    link.click();
    URL.revokeObjectURL(url);
  };
  if (trace.length === 0) return <Card title={t('trace.title')}><p>{t('trace.empty')}</p></Card>;
  return <Card title={t('trace.title')}>
    <p className="trace-intro">{t('trace.intro')}</p>
    <div className="trace-tools">
      <label className="trace-search">{t('trace.search')}
        <input type="search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder={t('trace.searchHint')} />
      </label>
      <Button onClick={download}>{t('trace.export')}</Button>
    </div>
    <div className="trace">
      {trace.map((step) => <Step key={step.key} step={step} depth={0} needle={needle} byKey={byKey} locale={locale} t={t} name={name} />)}
    </div>
  </Card>;
}
