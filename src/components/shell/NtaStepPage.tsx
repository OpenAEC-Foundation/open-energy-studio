/**
 * The NTA input on the workflow steps (ontwerp §6, F6, mockup 04).
 *
 * Each step page shows the sections of the register (`NtaSections`) that
 * belong to it, all on the one shared draft of `NtaDraftProvider`. Basis shows
 * the common sections; the rest waits behind "Geavanceerd" and opens by itself
 * when it holds input, a kernel finding or the field of "Ga naar". Heating has
 * a stepper over its parts. Every section closes with "Bron & bewijs": its
 * source references, filled or open. One bar at the bottom applies the draft.
 */
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import './ntaStep.css';
import { ArrowLeft, ArrowRight, FileText, Paperclip, Undo2 } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { useKernel } from '../../context/KernelProvider';
import { useNtaDraft, type NtaDraftState } from '../../context/NtaDraftProvider';
import { isPathWithin, parseKernelPath, formatKernelPath } from '../../core/nta/pathUtil';
import { routeForPath } from '../../core/nta/gapRoutes';
import { kernelIssues, type StepIssue } from '../../core/nta/stepStatus';
import { WORKFLOW_STEPS, type Route } from '../../core/navigation/routes';
import type { IProject } from '../../core/energy/types';
import { Banner, Button, Card, Segmented, Stepper, cx, type StepItem } from '../ui';
import { focusPathIn } from './focusPath';
import { FieldPathPrefixProvider, read, Section, type Draft } from '../NtaPerformancePanel/NtaFormFields';
import { NTA_PATH_PREFIX } from '../NtaPerformancePanel/NtaCalculationForm';
import {
  HEATING_PARTS, NTA_SECTIONS, visibleSections, type HeatingPart, type NtaSectionDef, type NtaSectionProps,
} from '../NtaPerformancePanel/NtaSections';

export type FieldMode = 'basic' | 'all';
const FIELD_MODE_KEY = 'oes.nta.fieldMode';

function storedFieldMode(): FieldMode {
  try {
    return window.localStorage.getItem(FIELD_MODE_KEY) === 'all' ? 'all' : 'basic';
  } catch {
    return 'basic';
  }
}

const prefixed = (path: string) => `${NTA_PATH_PREFIX}.${path}`;

/** The sections registered for a page, whether or not they apply to the draft. */
export function pageSections(step: string, sub: string | undefined): NtaSectionDef[] {
  return NTA_SECTIONS.filter((def) => def.step === step && (def.sub ?? undefined) === (sub ?? undefined));
}

/** The NTA input pages in workflow order, heating parts included. */
export function ntaPages(project: IProject, draft: Draft | null): Array<{ route: Route; part?: HeatingPart }> {
  const pages: Array<{ route: Route; part?: HeatingPart }> = [];
  const applies = (def: NtaSectionDef) => draft == null || !def.when || def.when(project, draft);
  for (const step of WORKFLOW_STEPS) {
    const subs = step.subs.length ? step.subs.map((sub) => sub.id) : [undefined];
    for (const sub of subs) {
      const defs = pageSections(step.id, sub).filter(applies);
      if (defs.length === 0) continue;
      const route: Route = { step: step.id, ...(sub ? { sub } : {}) };
      if (step.id === 'installations' && sub === 'heating') {
        for (const part of HEATING_PARTS) if (defs.some((def) => def.part === part)) pages.push({ route, part });
      } else {
        pages.push({ route });
      }
    }
  }
  return pages;
}

/** True when a value holds input (not blank, not an empty list or object, not `false`). */
function hasContent(value: unknown): boolean {
  if (value == null || value === false || value === '') return false;
  if (Array.isArray(value)) return value.length > 0;
  if (typeof value === 'object') return Object.values(value as object).some(hasContent);
  return true;
}

/** The heating part of a path in the NTA input (for "Ga naar" into the stepper). */
export function heatingPartForPath(path: string | undefined): HeatingPart | null {
  if (!path) return null;
  if (/\.verticalPipes\b/.test(path)) return 'distribution';
  let best: { part: HeatingPart; length: number } | null = null;
  for (const def of pageSections('installations', 'heating')) {
    for (const own of def.paths) {
      const candidate = prefixed(own);
      if (def.part && isPathWithin(path, candidate) && (!best || candidate.length > best.length)) best = { part: def.part, length: candidate.length };
    }
  }
  return best?.part ?? null;
}

/** The heating chain on the Installaties overview (mockup 03): opwekking › distributie › afgifte › regeling. */
export const HEATING_CHAIN: HeatingPart[] = ['generation', 'distribution', 'emission', 'control'];

/** Whether the NTA input holds anything for a heating part. */
export function heatingPartFilled(nta: Draft | null | undefined, part: HeatingPart): boolean {
  if (!nta) return false;
  return pageSections('installations', 'heating').some((def) =>
    def.part === part && def.paths.some((own) => hasContent(read(nta, parseKernelPath(own)))));
}

interface Evidence {
  path: string;
  filled: boolean;
}

/** Source references inside the paths of a section ("Bron & bewijs"). */
export function sectionEvidence(draft: Draft, def: NtaSectionDef): Evidence[] {
  const found: Evidence[] = [];
  const visit = (value: unknown, segments: Array<string | number>) => {
    const last = segments[segments.length - 1];
    if (typeof last === 'string' && /Reference$/.test(last) && (value == null || typeof value === 'string')) {
      found.push({ path: formatKernelPath(segments), filled: typeof value === 'string' && value.trim() !== '' });
      return;
    }
    if (Array.isArray(value)) value.forEach((item, index) => visit(item, [...segments, index]));
    else if (value != null && typeof value === 'object') {
      for (const [key, item] of Object.entries(value as Record<string, unknown>)) visit(item, [...segments, key]);
    }
  };
  for (const own of def.paths) visit(read(draft, parseKernelPath(own)), parseKernelPath(own));
  return found;
}

function ntaIssues(issues: StepIssue[]): Array<StepIssue & { focus: string }> {
  return issues
    .map((issue) => ({ ...issue, focus: routeForPath(issue.path).focusPath ?? '' }))
    .filter((issue) => isPathWithin(issue.focus, NTA_PATH_PREFIX) && issue.focus !== NTA_PATH_PREFIX);
}

const within = (issue: { focus: string }, def: NtaSectionDef) => def.paths.some((own) => isPathWithin(issue.focus, prefixed(own)));

function EvidenceBlock({ draft, def, container }: { draft: Draft; def: NtaSectionDef; container: () => HTMLElement | null }) {
  const { t } = useI18n();
  const items = sectionEvidence(draft, def);
  if (items.length === 0) return null;
  const open = items.filter((item) => !item.filled).length;
  const goTo = (path: string) => {
    const element = container();
    if (element) focusPathIn(element, prefixed(path));
  };
  return (
    <div className={cx('nta-evidence', open > 0 && 'nta-evidence--open')} role="group" aria-label={`${t('ntaStep.evidence')}: ${t(def.titleKey)}`}>
      <div className="nta-evidence__head">
        <strong>{t('ntaStep.evidence')}</strong>
        {open > 0
          ? <span className="nta-evidence__badge">{t('ntaStep.evidence.open', { n: open })}</span>
          : <span className="nta-evidence__ok">{t('ntaStep.evidence.complete')}</span>}
      </div>
      <ul className="nta-evidence__list">
        {items.map((item) => {
          const value = read(draft, parseKernelPath(item.path));
          return (
            <li key={item.path} className={item.filled ? 'filled' : 'open'}>
              <code>{item.path}</code>
              {item.filled
                ? <span className="nta-evidence__value"><Paperclip aria-hidden="true" /> {String(value)}</span>
                : <span className="nta-evidence__value">{t('ntaStep.evidence.missing')}</span>}
              <button type="button" className="ui-btn ui-btn--ghost ui-btn--sm" onClick={() => goTo(item.path)}
                aria-label={`${t('ui.goTo')}: ${item.path}`}>
                {t('ui.goTo')} <ArrowRight aria-hidden="true" />
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

function SectionBlock({ def, props, container }: { def: NtaSectionDef; props: NtaSectionProps; container: () => HTMLElement | null }) {
  const { t } = useI18n();
  const { Component } = def;
  const [first, ...rest] = def.paths.map(prefixed);
  if (def.bare) return <div className="nta-step-section nta-step-section--bare" data-path={first} data-paths={rest.join(' ') || undefined}>
    <Component {...props} />
    <EvidenceBlock draft={props.draft} def={def} container={container} />
  </div>;
  return <div className="nta-step-section">
    <Section title={t(def.titleKey)} path={first} extraPaths={rest}>
      <Component {...props} />
    </Section>
    <EvidenceBlock draft={props.draft} def={def} container={container} />
  </div>;
}

/** "Geavanceerd · n" around the advanced sections in the Basis view. */
function AdvancedBlock({ defs, draft, open, children }: { defs: NtaSectionDef[]; draft: Draft; open: boolean; children: React.ReactNode }) {
  const { t } = useI18n();
  const filled = defs.filter((def) => def.paths.some((own) => hasContent(read(draft, parseKernelPath(own))))).length;
  return (
    <details className="nta-advanced" open={open}>
      <summary>
        <span className="nta-advanced__title">{t('ntaStep.advanced', { n: defs.length })}</span>
        <span className="nta-advanced__list">— {defs.map((def) => t(def.titleKey)).join(', ')}</span>
        <span className="nta-advanced__state">{filled === 0 ? t('ntaStep.advanced.default') : t('ntaStep.advanced.filled', { n: filled })}</span>
      </summary>
      <div className="nta-advanced__body">{children}</div>
    </details>
  );
}

export const PART_KEYS: Record<HeatingPart, string> = {
  generation: 'ntaStep.part.generation',
  distribution: 'ntaStep.part.distribution',
  emission: 'ntaStep.part.emission',
  control: 'ntaStep.part.control',
  auxiliary: 'ntaStep.part.auxiliary',
  solar: 'ntaStep.part.solar',
};

/** The NTA sections of the current page, or nothing when the page has none. */
export function NtaStepSections({ route }: { route: Route }) {
  const { t } = useI18n();
  const shared = useNtaDraft();
  const kernel = useKernel();
  const ref = useRef<HTMLDivElement>(null);
  const [mode, setMode] = useState<FieldMode>(storedFieldMode);
  const all = pageSections(route.step, route.sub);
  const heating = route.step === 'installations' && route.sub === 'heating';
  const setHeatingPart = shared?.setHeatingPart;

  // "Ga naar" into another heating part opens that part before the field is focused.
  useLayoutEffect(() => {
    const part = heating ? heatingPartForPath(route.focusPath) : null;
    if (part && setHeatingPart) setHeatingPart(part);
  }, [route, heating, setHeatingPart]);

  const issues = useMemo(() => ntaIssues(kernelIssues(kernel?.settled)), [kernel?.settled]);
  useIssueMarks(ref, issues, shared?.draft ?? null, mode, shared?.heatingPart);

  if (!shared || all.length === 0) return null;
  const { draft, project } = shared;
  const changeMode = (next: FieldMode) => {
    setMode(next);
    try { window.localStorage.setItem(FIELD_MODE_KEY, next); } catch { /* per-viewer convenience only */ }
  };

  const titleId = `nta-step-title-${route.step}-${route.sub ?? 'main'}`;
  const toolbar = <Segmented<FieldMode> size="sm" aria-label={t('ntaStep.fieldMode')} value={mode} onChange={changeMode}
    options={[{ value: 'basic', label: t('ntaStep.fieldMode.basic') }, { value: 'all', label: t('ntaStep.fieldMode.all') }]} />;

  if (draft == null) {
    return (
      <Card level={2} className="nta-step" title={<span id={titleId}>{t('installations.ntaInput')}</span>}>
        <p className="page-lead">{t('installations.ntaMissing')}</p>
        <Button variant="primary" onClick={shared.start}>{t('nta.performance.start')}</Button>
      </Card>
    );
  }

  const applicable = visibleSections(project, draft, (def) => all.includes(def));
  const parts = heating ? HEATING_PARTS.filter((part) => applicable.some((def) => def.part === part)) : [];
  const part = heating ? (parts.includes(shared.heatingPart as HeatingPart) ? shared.heatingPart as HeatingPart : parts[0]) : undefined;
  const shown = heating ? applicable.filter((def) => def.part === part) : applicable;
  const basic = mode === 'all' ? shown : shown.filter((def) => !def.advanced);
  const advanced = mode === 'all' ? [] : shown.filter((def) => def.advanced);
  const props: NtaSectionProps = { draft, change: shared.change, update: shared.update, project };
  const container = () => ref.current;
  const advancedOpen = advanced.some((def) =>
    def.paths.some((own) => hasContent(read(draft, parseKernelPath(own))))
    || issues.some((issue) => within(issue, def))
    || (route.focusPath != null && def.paths.some((own) => isPathWithin(route.focusPath!, prefixed(own)))));

  const partSteps: Array<StepItem<HeatingPart>> = parts.map((id) => {
    const own = issues.filter((issue) => applicable.some((def) => def.part === id && within(issue, def)));
    const errors = own.filter((issue) => issue.kind === 'error').length;
    const warnings = own.length - errors;
    return {
      id,
      label: <>{t(PART_KEYS[id])}{own.length > 0 && <span className={cx('nta-part-count', errors > 0 && 'nta-part-count--error')}>{own.length}</span>}</>,
      status: id === part ? 'current' : errors > 0 ? 'error' : warnings > 0 ? 'warn' : 'done',
      statusText: [errors > 0 && t('ntaStep.check.errors', { n: errors }), warnings > 0 && t('ntaStep.check.warnings', { n: warnings })]
        .filter(Boolean).join(', ') || undefined,
    };
  });

  return (
    <section className="nta-step" aria-labelledby={titleId} ref={ref} data-field-mode={mode}>
      <header className="nta-step__head">
        <div>
          <h2 id={titleId} className="nta-step__title">{t('installations.ntaInput')}</h2>
          <p className="nta-step__lead">{applicable.length === 0 ? t('ntaStep.noneApplicable') : t('ntaStep.lead')}</p>
        </div>
        {toolbar}
      </header>
      {heating && parts.length > 0 && <Stepper<HeatingPart> className="nta-step__parts" aria-label={t('ntaStep.parts')}
        steps={partSteps} onSelect={(id) => shared.setHeatingPart(id)} />}
      <FieldPathPrefixProvider value={NTA_PATH_PREFIX}>
        <div className="nta-form nta-step__sections">
          {basic.map((def) => <SectionBlock key={def.id} def={def} props={props} container={container} />)}
          {advanced.length > 0 && <AdvancedBlock key={`${route.sub}-${part ?? ''}-${advancedOpen}`} defs={advanced} draft={draft} open={advancedOpen}>
            {advanced.map((def) => <SectionBlock key={def.id} def={def} props={props} container={container} />)}
          </AdvancedBlock>}
        </div>
      </FieldPathPrefixProvider>
    </section>
  );
}

/** Red or yellow edge on the fields with a kernel finding (ontwerp §6.4 "inline per veld"). */
function useIssueMarks(ref: React.RefObject<HTMLElement | null>, issues: Array<StepIssue & { focus: string }>,
  draft: Draft | null, mode: FieldMode, part: string | undefined) {
  const { t } = useI18n();
  useEffect(() => {
    const container = ref.current;
    if (!container) return;
    const marked = new Map<HTMLElement, { kind: 'error' | 'warning'; codes: string[] }>();
    const elements = Array.from(container.querySelectorAll<HTMLElement>('label[data-path]'));
    for (const issue of issues) {
      let best: HTMLElement | null = null;
      let bestLength = -1;
      for (const element of elements) {
        const candidate = element.dataset.path ?? '';
        if (isPathWithin(issue.focus, candidate) && candidate.length > bestLength) { best = element; bestLength = candidate.length; }
      }
      if (!best) continue;
      const current = marked.get(best);
      marked.set(best, {
        kind: current?.kind === 'error' || issue.kind === 'error' ? 'error' : 'warning',
        codes: [...(current?.codes ?? []), issue.code],
      });
    }
    for (const element of Array.from(container.querySelectorAll<HTMLElement>('[data-issue]'))) {
      if (!marked.has(element)) { element.removeAttribute('data-issue'); element.removeAttribute('title'); }
    }
    for (const [element, mark] of marked) {
      element.dataset.issue = mark.kind;
      element.title = `${t(`ui.severity.${mark.kind}`)}: ${mark.codes.join(', ')}`;
    }
  }, [ref, issues, draft, mode, part, t]);
}

/** Changed field names for the apply bar: the last member of each path. */
function changeNames(changes: string[]): string[] {
  const names: string[] = [];
  for (const path of changes) {
    const segments = parseKernelPath(path).filter((segment) => typeof segment === 'string') as string[];
    const name = segments[segments.length - 1];
    if (name && !names.includes(name)) names.push(name);
  }
  return names;
}

/**
 * The one bar for the NTA draft: "n wijzigingen … · Ongedaan maken · Vorige stap ·
 * Toepassen en verder" (ontwerp §6.2). Toepassen is the former Opslaan.
 */
export function NtaApplyBar({ route, navigate }: { route: Route; navigate: (route: Route) => void }) {
  const { t } = useI18n();
  const shared = useNtaDraft();
  if (!shared || !shared.dirty || route.step === 'results') return null;
  const pages = ntaPages(shared.project, shared.draft);
  const here = pages.findIndex((page) => page.route.step === route.step && page.route.sub === route.sub
    && (page.part == null || page.part === shared.heatingPart));
  const go = (index: number) => {
    const page = pages[index];
    if (!page) return;
    if (page.part) shared.setHeatingPart(page.part);
    navigate(page.route);
  };
  const names = changeNames(shared.changes);
  const count = shared.isNew ? null : shared.changes.length;
  const applyAndNext = (state: NtaDraftState) => {
    state.apply();
    if (here >= 0 && here < pages.length - 1) go(here + 1);
  };
  return (
    <div className="nta-apply-bar" role="region" aria-label={t('ntaStep.apply.region')}>
      <span className="nta-apply-bar__dot" aria-hidden="true" />
      <p className="nta-apply-bar__text" role="status">
        {count == null
          ? <strong>{t('ntaStep.apply.new')}</strong>
          : <><strong>{t(count === 1 ? 'ntaStep.apply.one' : 'ntaStep.apply.many', { n: count })}</strong> {t('ntaStep.apply.pending')}</>}
      </p>
      {names.length > 0 && <p className="nta-apply-bar__names" title={shared.changes.join(', ')}>
        <FileText aria-hidden="true" /> {names.slice(0, 3).join(', ')}{names.length > 3 ? ` +${names.length - 3}` : ''}
      </p>}
      <div className="nta-apply-bar__actions">
        <Button variant="ghost" size="sm" icon={<Undo2 aria-hidden="true" />} disabled={!shared.canUndo} onClick={shared.undo}>
          {t('ntaStep.apply.undo')}
        </Button>
        {here > 0 && <Button size="sm" icon={<ArrowLeft aria-hidden="true" />} onClick={() => go(here - 1)}>{t('ntaStep.apply.previous')}</Button>}
        {here >= 0 && here < pages.length - 1
          ? <Button variant="primary" size="sm" onClick={() => applyAndNext(shared)}>
            {t('ntaStep.apply.applyNext')} <ArrowRight aria-hidden="true" />
          </Button>
          : <Button variant="primary" size="sm" onClick={shared.apply}>{t('ntaStep.apply.apply')}</Button>}
      </div>
    </div>
  );
}

/** Shown on Controle › NTA-invoer above the full form when steps hold unapplied input. */
export function NtaDraftNotice() {
  const { t } = useI18n();
  const shared = useNtaDraft();
  if (!shared?.dirty) return null;
  return <Banner tone="warn">{t('ntaStep.draftNotice')}</Banner>;
}
