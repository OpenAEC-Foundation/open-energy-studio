/**
 * The input of a project without a basisopname (new build, NTA input) as a
 * question flow, in the style of the basisopname (UI redesign 2026-10):
 * one topic per screen with Vorige / Overslaan / Volgende, the provisional
 * outcome and what is still open beside it, and Controle as the one page
 * with everything. The questions are the existing sub pages of Project,
 * Gebouw and Installaties; their content is unchanged.
 */
import { useEffect, useRef, type ReactNode } from 'react';
import { Building2, Factory, GraduationCap, HeartPulse, Home, MoreHorizontal, ShoppingBag, Upload } from 'lucide-react';
import '../SurveyWizard/SurveyWizard.css';
import './BuildFlow.css';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import { useKernel } from '../../context/KernelProvider';
import { useNtaDraft } from '../../context/NtaDraftProvider';
import { useShellActions } from '../shell/ShellActions';
import { useInvalidFields } from '../SurveyWizard/SurveyWizard';
import { IssueList } from '../ui';
import { routeForPath } from '../../core/nta/gapRoutes';
import { kernelIssues, type StepIssue, type StepStatus } from '../../core/nta/stepStatus';
import { summarizeForPreview, type PreviewSummary } from '../../core/nta/PreviewSummary';
import { projectCalculated } from '../../core/nta/KernelClient';
import { labelColor } from '../shell/pages/results/resultsData';
import { routeLabel } from '../shell/PageHeader';
import type { BuildingFunction, IProject } from '../../core/energy/types';
import type { Route, StepId } from '../../core/navigation/routes';
import {
  PROJECT_QUESTION, buildFlowSteps, buildQuestionKey, buildQuestionOf, buildQuestionRoute, buildQuestionState,
  buildStepState, markProgress, type BuildFlowStep, type BuildFlowStepId,
} from '../../core/navigation/buildFlow';

type T = (key: string, params?: Record<string, string | number>) => string;

const STEP_LABEL: Record<BuildFlowStepId, string> = {
  project: 'nav.step.project', building: 'nav.step.building', installations: 'nav.step.installations', check: 'nav.step.check',
};

/** Navigation label of a question. */
export function buildQuestionLabel(step: BuildFlowStepId, question: string): string {
  return step === 'project' ? 'build.navQuestion.info' : `nav.sub.${step}.${question}`;
}

/** Kernel findings that belong to a question (through the "Ga naar" route of their path). */
function issuesOf(issues: StepIssue[], step: BuildFlowStepId, question?: string): StepIssue[] {
  return issues.filter((issue) => {
    const route = routeForPath(issue.path);
    if (route.step !== step) return false;
    return question == null || step === 'project' || route.sub === question;
  });
}

function useFlow() {
  const { state, dispatch } = useEnergy();
  const project = state.project;
  const steps = buildFlowSteps(project);
  const progress = project.workflowProgress;
  const mark = (step: BuildFlowStepId, question: string, value: 'done' | 'skipped') =>
    dispatch({ type: 'SET_WORKFLOW_PROGRESS', payload: markProgress(progress, buildQuestionKey(step, question), value) });
  const markStep = (step: BuildFlowStepId) => {
    if (progress?.done?.includes(step)) return;
    dispatch({ type: 'SET_WORKFLOW_PROGRESS', payload: { done: [...(progress?.done ?? []), step], skipped: progress?.skipped ?? [] } });
  };
  return { project, steps, progress, mark, markStep };
}

/** Label, BENG indicators and TOjuli of the last kernel answer. */
function Indicators({ summary, t, locale }: { summary: PreviewSummary; t: T; locale: string }) {
  return <dl className="survey-indicators">
    <div><dt>BENG 1</dt><dd>{formatNumber(summary.beng1, locale, 1)}</dd></div>
    <div><dt>BENG 2</dt><dd>{formatNumber(summary.beng2, locale, 1)}</dd></div>
    <div><dt>BENG 3</dt><dd>{summary.beng3 != null ? `${formatNumber(summary.beng3, locale, 0)} %` : '—'}</dd></div>
    {summary.tojuliApplies && <div><dt>{t('build.result.tojuli')}</dt><dd>{formatNumber(summary.tojuliMaxK, locale, 2)}</dd></div>}
  </dl>;
}

function ResultCard({ t, locale }: { t: T; locale: string }) {
  const kernel = useKernel();
  const navigate = useShellActions()?.navigate;
  const assessment = kernel?.settled ?? null;
  const summary = assessment ? summarizeForPreview(assessment) : null;
  const calculated = summary != null && projectCalculated(summary.status);
  const issues = kernelIssues(assessment);
  const errors = issues.filter((issue) => issue.kind === 'error').length;
  return <div className="survey-card" aria-live="polite">
    <span className="survey-overline">{t('build.result.title')}</span>
    {calculated && summary ? <>
      {summary.labelClass && <div className="survey-label-row">
        <span className="survey-label" style={{ background: labelColor(summary.labelClass) }}>{summary.labelClass}</span>
        <span>{t('build.result.label')}</span>
      </div>}
      <dl className="build-gauges">
        <div><dt>BENG 1</dt><dd>{formatNumber(summary.beng1, locale, 1)}{summary.beng1Limit != null && <small> / {formatNumber(summary.beng1Limit, locale, 1)}</small>}</dd></div>
        <div><dt>BENG 2</dt><dd>{formatNumber(summary.beng2, locale, 1)}{summary.beng2Limit != null && <small> / {formatNumber(summary.beng2Limit, locale, 1)}</small>}</dd></div>
        <div><dt>BENG 3</dt><dd>{formatNumber(summary.beng3, locale, 0)} %{summary.beng3Limit != null && <small> / {formatNumber(summary.beng3Limit, locale, 0)} %</small>}</dd></div>
        {summary.tojuliApplies && <div><dt>{t('build.result.tojuli')}</dt><dd>{formatNumber(summary.tojuliMaxK, locale, 2)}<small> / 1,20</small></dd></div>}
      </dl>
    </> : <p className="survey-muted">{kernel?.phase === 'loading' ? t('survey.result.busy')
      : kernel?.phase === 'error' ? t('survey.result.error') : t('build.result.none')}</p>}
    {errors > 0 && (navigate
      ? <button type="button" className="survey-issues-note survey-issues-link" onClick={() => navigate({ step: 'check', sub: 'overview' })}>
        {t('survey.result.issues', { count: errors })} ›</button>
      : <p className="survey-issues-note">{t('survey.result.issues', { count: errors })}</p>)}
    <p className="survey-muted">{t('build.result.note')}</p>
  </div>;
}

/** What is still open: the steps that are not done, with skipped or partly. */
function OpenList({ current, t }: { current: BuildFlowStepId | null; t: T }) {
  const { steps, progress } = useFlow();
  const navigate = useShellActions()?.navigate;
  const open = steps.filter((step) => step.questions.length > 0 && step.id !== current && buildStepState(step, progress) !== 'done');
  return <div className="survey-card">
    <span className="survey-overline">{t('survey.open.title')}</span>
    {open.length === 0 ? <p className="survey-done">{t('survey.open.none')}</p> : <ul className="survey-open">
      {open.map((step) => {
        const state = buildStepState(step, progress);
        const first = step.questions.find((question) => buildQuestionState(progress, step.id, question) !== 'done') ?? step.questions[0];
        return <li key={step.id}>
          <button type="button" onClick={() => navigate?.(buildQuestionRoute(step.id, first))}>
            <span>{t(STEP_LABEL[step.id])}</span>
            <span className={`survey-open-tag survey-open-tag--${state}`}>{t(`survey.open.${state}`)}</span>
          </button>
        </li>;
      })}
    </ul>}
  </div>;
}

/**
 * One question of the flow around an existing page body. Outside the flow
 * (3D model, installation overview, heat pumps) the body shows as it is.
 */
export function BuildFlowFrame({ route, lead, toolbar, children }: {
  route: Route; lead?: ReactNode; toolbar?: ReactNode; children: ReactNode;
}) {
  const { t, locale } = useI18n();
  const { steps, progress, mark } = useFlow();
  const shared = useNtaDraft();
  const navigate = useShellActions()?.navigate ?? (() => undefined);
  // Fields the kernel names as missing or wrong are outlined in red (feedback 8 Oct 2026).
  useInvalidFields('.build-flow', (useKernel()?.settled?.gaps ?? []).map((gap) => gap.path));
  const question = buildQuestionOf(route, steps);
  const stepIndex = steps.findIndex((step) => step.id === route.step);
  const step = steps[stepIndex];
  // A question left after a change counts as answered, also when left through the navigation.
  const energy = useEnergy();
  const dispatch = energy.dispatch;
  const latest = useRef(energy.state.project);
  latest.current = energy.state.project;
  const visitedKey = step && question != null ? buildQuestionKey(step.id, question) : null;
  useEffect(() => {
    if (!visitedKey) return undefined;
    const entry = latest.current;
    return () => {
      const now = latest.current;
      const [stepId, ...rest] = visitedKey.split('.');
      if (now === entry || buildQuestionState(now.workflowProgress, stepId as BuildFlowStepId, rest.join('.')) !== 'todo') return;
      dispatch({ type: 'SET_WORKFLOW_PROGRESS', payload: markProgress(now.workflowProgress, visitedKey, 'done') });
    };
  }, [visitedKey, dispatch]);
  if (!step || question == null) return <>{children}</>;

  const questionIndex = step.questions.indexOf(question);
  const flat = steps.flatMap((candidate) => candidate.questions.length
    ? candidate.questions.map((id) => ({ step: candidate.id, question: id }))
    : [{ step: candidate.id, question: '' }]);
  const here = flat.findIndex((item) => item.step === step.id && item.question === question);
  const next = flat[here + 1];
  const previous = flat[here - 1];
  const go = (item: { step: BuildFlowStepId; question: string } | undefined) => {
    if (!item) return;
    // The NTA draft of this page is applied before moving on (the former "Toepassen en verder").
    if (shared?.dirty) shared.apply();
    navigate(buildQuestionRoute(item.step, item.question || undefined));
  };
  const advance = (value: 'done' | 'skipped') => {
    mark(step.id, question, value);
    go(next);
  };
  const lastOfStep = questionIndex === step.questions.length - 1;
  const nextLabel = !lastOfStep ? t('survey.nav.nextQuestion')
    : next ? t('survey.nav.next', { step: t(STEP_LABEL[next.step]).toLowerCase() }) : t('survey.nav.finish');
  const state = buildQuestionState(progress, step.id, question);

  return <div className="survey-wizard build-flow">
    <div className="survey-main">
      <p className="survey-crumb">{t('survey.crumb', { number: stepIndex + 1, total: steps.length, step: t(STEP_LABEL[step.id]) })}
        {step.questions.length > 1 && ` · ${t('survey.crumbQuestion', { number: questionIndex + 1, total: step.questions.length })}`}</p>
      <div className="build-flow-head">
        <h1 id="page-title" tabIndex={-1}>{t(`build.q.${question}`)}</h1>
        {toolbar && <div className="build-flow-tools">{toolbar}</div>}
      </div>
      <p className="survey-lead">{lead ?? t(`build.q.${question}.help`)}</p>
      <div className="survey-question build-flow-body" key={`${step.id}.${question}`}>{children}</div>
      {state === 'skipped' && <p className="survey-skipped-note">{t('survey.skippedNote')}</p>}
      <div className="survey-actions">
        <button type="button" className="btn" onClick={() => go(previous)} disabled={!previous}>{t('survey.nav.previous')}</button>
        <div className="survey-actions-right">
          {shared?.dirty && shared.canUndo && <button type="button" className="btn" onClick={shared.undo}>{t('ntaStep.apply.undo')}</button>}
          <button type="button" className="btn survey-skip" onClick={() => advance('skipped')}>{t('survey.nav.skip')}</button>
          <button type="button" className="btn btn-primary" onClick={() => advance('done')}>{nextLabel}</button>
        </div>
      </div>
    </div>
    <aside className="survey-aside" aria-label={t('survey.aside')}>
      <ResultCard t={t} locale={locale} />
      <OpenList current={step.id} t={t} />
    </aside>
  </div>;
}

const FUNCTIONS: Array<{ id: BuildingFunction; icon: ReactNode }> = [
  { id: 'residential', icon: <Home /> },
  { id: 'office', icon: <Building2 /> },
  { id: 'education', icon: <GraduationCap /> },
  { id: 'healthcare', icon: <HeartPulse /> },
  { id: 'retail', icon: <ShoppingBag /> },
  { id: 'industrial', icon: <Factory /> },
  { id: 'other', icon: <MoreHorizontal /> },
];

/** The project question: name, address and the building function as cards. */
export function ProjectInfoQuestion() {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const actions = useShellActions();
  const { project } = state;
  const set = (payload: Partial<Pick<IProject, 'name' | 'description' | 'buildingFunction' | 'address' | 'city'>>) =>
    dispatch({ type: 'UPDATE_PROJECT_INFO', payload });
  return <div className="build-project">
    <div className="build-fields">
      <label className="build-field build-field--wide">{t('dialog.projectInfo.name')}
        <input type="text" value={project.name} onChange={(event) => set({ name: event.target.value })} />
      </label>
      <label className="build-field">{t('dialog.projectInfo.address')}
        <input type="text" value={project.address} onChange={(event) => set({ address: event.target.value })} />
      </label>
      <label className="build-field">{t('dialog.projectInfo.city')}
        <input type="text" value={project.city} onChange={(event) => set({ city: event.target.value })} />
      </label>
      <label className="build-field build-field--wide">{t('dialog.projectInfo.description')}
        <textarea rows={2} value={project.description} onChange={(event) => set({ description: event.target.value })} />
      </label>
    </div>
    <h2 className="build-subtitle">{t('dialog.projectInfo.function')}</h2>
    <div className="survey-choices" role="group" aria-label={t('dialog.projectInfo.function')}>
      {FUNCTIONS.map((option) => (
        <button key={option.id} type="button" className="survey-choice" aria-pressed={project.buildingFunction === option.id}
          onClick={() => set({ buildingFunction: option.id })}>
          <span className="survey-choice-icon" aria-hidden="true">{option.icon}</span>
          <span className="survey-choice-title">{t(`function.${option.id}`)}</span>
        </button>
      ))}
    </div>
    {actions && <p className="build-import">
      <span>{t('build.import')}</span>
      <button type="button" className="btn btn-sm" onClick={actions.importUNIEC3}><Upload aria-hidden="true" /> {t('ribbon.importUNIEC3')}</button>
      <button type="button" className="btn btn-sm" onClick={actions.importVABI}><Upload aria-hidden="true" /> {t('ribbon.importVABI')}</button>
    </p>}
  </div>;
}

function stepFacts(step: BuildFlowStepId, project: IProject, t: T, locale: string, agM2: number | null): string[] {
  switch (step) {
    case 'project': return [
      t(`function.${project.buildingFunction}`),
      [project.address, project.city].filter(Boolean).join(', ') || t('build.fact.noAddress'),
      ...(agM2 != null ? [`A_g ${formatNumber(agM2, locale, 1)} m²`] : []),
    ];
    case 'building': {
      const surfaces = project.zones.reduce((sum, zone) => sum + zone.surfaces.length, 0);
      const windows = project.zones.reduce((sum, zone) => zone.surfaces.reduce((count, surface) => count + surface.windows.length, sum), 0);
      const bridges = project.zones.reduce((sum, zone) => sum + zone.thermalBridges.length, 0);
      return [
        `${t('browser.zones')}: ${project.zones.length}`,
        `${t('browser.surfaces')}: ${surfaces} · ${t('browser.windows')}: ${windows}`,
        `${t('browser.thermalBridges')}: ${bridges}`,
      ];
    }
    case 'installations': {
      const names = [...project.heatingSystems, ...project.hotWaterSystems, ...project.ventilationSystems,
        ...project.coolingSystems, ...project.solarPV, ...project.solarThermal].map((system) => system.name).filter(Boolean);
      return names.length ? names.slice(0, 5) : [t('installations.empty')];
    }
    default: return [];
  }
}

/** Controle: everything on one page, an edit button per step, the kernel notices with "Ga naar". */
export function BuildCheckPage({ statuses }: { statuses: Record<StepId, StepStatus> }) {
  const { t, locale } = useI18n();
  const { project, steps, progress, markStep } = useFlow();
  const kernel = useKernel();
  const navigate = useShellActions()?.navigate ?? (() => undefined);
  const assessment = kernel?.settled ?? null;
  const summary = assessment ? summarizeForPreview(assessment) : null;
  const calculated = summary != null && projectCalculated(summary.status);
  const issues = kernelIssues(assessment);
  const stepIndex = steps.findIndex((step) => step.id === 'check');
  const inputSteps = steps.filter((step): step is BuildFlowStep => step.questions.length > 0);
  const edit = (step: BuildFlowStep) => {
    const first = step.questions.find((question) => buildQuestionState(progress, step.id, question) !== 'done') ?? step.questions[0];
    navigate(buildQuestionRoute(step.id, first));
  };

  return <div className="survey-wizard build-flow">
    <div className="survey-main">
      <p className="survey-crumb">{t('survey.crumb', { number: stepIndex + 1, total: steps.length, step: t('nav.step.check') })}</p>
      <h1 id="page-title" tabIndex={-1}>{t('survey.check.title')}</h1>
      <p className="survey-lead">{t('build.check.lead')}</p>
      <div className="survey-check">
        <section className="survey-card survey-outcome" aria-label={t('survey.check.outcome')}>
          {calculated && summary?.labelClass
            ? <span className="survey-label survey-label--big" style={{ background: labelColor(summary.labelClass) }}>{summary.labelClass}</span>
            : !calculated && <span className="survey-muted">{t('build.result.none')}</span>}
          {calculated && summary && <Indicators summary={summary} t={t} locale={locale} />}
        </section>
        {(project.importLog ?? []).filter((record) => record.notes?.length).map((record, index) =>
          <section key={index} className="survey-card build-import-notes" aria-label={t('build.check.imported', { tool: record.tool })}>
            <h2>{t('build.check.imported', { tool: record.tool })}{record.fileName ? <small> · {record.fileName}</small> : null}</h2>
            <p className="survey-muted">{t('build.check.importedLead')}</p>
            <ul>{record.notes!.map((line, at) => <li key={at}>{line}</li>)}</ul>
          </section>)}
        <div className="survey-check-grid">
          {inputSteps.map((step) => {
            const state = buildStepState(step, progress);
            const errors = statuses[step.id]?.errors ?? 0;
            return <section key={step.id} className="survey-card survey-check-card" aria-label={t(STEP_LABEL[step.id])}>
              <div className="survey-check-head">
                <h2>{t(STEP_LABEL[step.id])}</h2>
                <button type="button" className="btn btn-sm" onClick={() => edit(step)}>{t('survey.check.edit')}</button>
              </div>
              <ul>{stepFacts(step.id, project, t, locale, assessment?.geometry?.usableFloorAreaM2 ?? null).map((fact) => <li key={fact}>{fact}</li>)}</ul>
              {state === 'skipped' && <p className="survey-skipped-note">{t('survey.check.skipped')}</p>}
              {state === 'partial' && <p className="survey-muted">{t('build.check.partial')}</p>}
              {errors > 0 && <p className="survey-issues-note">{t('survey.result.issues', { count: errors })}</p>}
            </section>;
          })}
        </div>
        {issues.length > 0 && <section className="survey-card" aria-label={t('survey.check.notices')}>
          <span className="survey-overline">{t('survey.check.notices')}</span>
          <IssueList empty={t('overview.noIssues')}
            issues={issues.map((issue) => ({
              code: issue.code, severity: issue.kind, path: issue.path, detail: issue.detail,
              location: routeLabel(t, routeForPath(issue.path)),
            }))}
            onGoTo={(issue) => navigate(routeForPath(issue.path))} />
        </section>}
        <div className="survey-next-actions">
          <button type="button" className="btn" onClick={() => navigate({ step: 'check', sub: 'input' })}>{t('build.check.allInput')}</button>
          <button type="button" className="btn" onClick={() => navigate({ step: 'report' })}>{t('survey.label.report')}</button>
          <button type="button" className="btn btn-primary" onClick={() => { markStep('check'); navigate({ step: 'results' }); }}>
            {t('build.check.results')}</button>
        </div>
      </div>
    </div>
    <aside className="survey-aside" aria-label={t('survey.aside')}>
      <OpenList current={null} t={t} />
      <p className="survey-defaults-note">{t('build.check.note')}</p>
    </aside>
  </div>;
}

/** The steps of the flow in the navigation, with state, error counts and the questions of the current step. */
export function BuildNavList({ route, statuses, navigate }: {
  route: Route; statuses: Record<StepId, StepStatus>; navigate: (route: Route) => void;
}) {
  const { t } = useI18n();
  const { steps, progress } = useFlow();
  const kernel = useKernel();
  const issues = kernelIssues(kernel?.settled ?? null).filter((issue) => issue.kind === 'error');
  return <>
    {steps.map((step, index) => {
      const state = buildStepState(step, progress);
      const errors = statuses[step.id]?.errors ?? 0;
      const current = route.step === step.id;
      const badge = state === 'done' && errors === 0 ? 'complete' : errors > 0 ? 'errors' : state === 'skipped' ? 'skipped' : current ? 'current' : 'todo';
      const stateText = state === 'done' ? t('nav.state.complete') : state === 'skipped' ? t('nav.state.skipped')
        : state === 'partial' ? t('nav.state.partial') : t('nav.state.todo');
      const target = step.questions.length
        ? buildQuestionRoute(step.id, step.questions.find((question) => buildQuestionState(progress, step.id, question) !== 'done') ?? step.questions[0])
        : buildQuestionRoute(step.id);
      return <li key={step.id}>
        <button type="button" className="nav-step" aria-current={current ? 'page' : undefined} data-step={step.id} data-state={state}
          onClick={() => navigate?.(target)}>
          <span className={`nav-step-no ${badge}`} aria-hidden="true">{badge === 'complete' ? '✓' : badge === 'skipped' ? '↷' : index + 1}</span>
          <span className="nav-step-label">{t(STEP_LABEL[step.id])}</span>
          {errors > 0 && <span className="nav-count errors" aria-hidden="true">{errors}</span>}
          {errors === 0 && state === 'skipped' && <span className="nav-later" aria-hidden="true">{t('survey.open.skipped')}</span>}
          <span className="visually-hidden">, {stateText}{errors > 0 ? `, ${t('nav.state.errors', { count: String(errors) })}` : ''}</span>
        </button>
        {current && step.questions.length > 1 && <ul className="nav-subs">
          {step.questions.map((question) => {
            const answered = buildQuestionState(progress, step.id, question);
            const here = route.sub === question;
            const own = issuesOf(issues, step.id, question).length;
            return <li key={question}>
              <button type="button" className={`nav-sub nav-question nav-question--${answered}`} aria-current={here ? 'step' : undefined}
                onClick={() => navigate?.(buildQuestionRoute(step.id, question))}>
                <span className="nav-question-mark" aria-hidden="true">{answered === 'done' ? '✓' : answered === 'skipped' ? '↷' : here ? '›' : '·'}</span>
                {t(buildQuestionLabel(step.id, question))}
                {own > 0 && <span className="nav-count errors" aria-hidden="true">{own}</span>}
              </button>
            </li>;
          })}
        </ul>}
      </li>;
    })}
  </>;
}

export { PROJECT_QUESTION };
