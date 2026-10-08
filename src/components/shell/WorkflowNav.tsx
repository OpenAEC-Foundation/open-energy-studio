/**
 * Numbered workflow navigation (ontwerp §3.2, mockup 01). Each step shows its
 * kernel-driven status; the active step has aria-current="page" and lists its
 * sub pages. The footer holds the compact Gereedschap menu (calculators,
 * exchange formats, feedback) and Instellingen — there is no tab strip of tools.
 */
import { useRef, type KeyboardEvent, type ReactNode } from 'react';
import {
  BookOpen, Box, Calculator, Check, ChevronRight, Download, Flame, MessageSquare, SlidersHorizontal, Upload, Wrench,
} from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { Kbd, Tag } from '../ui';
import { useMenu } from './TopBar';
import { HELP_TOOL_SUBS, TOOL_STEP, type Route, type StepId } from '../../core/navigation/routes';
import { isNewBuild, isSurveyProject, type StepStatus } from '../../core/nta/stepStatus';
import type { IProject } from '../../core/energy/types';
import type { ShellActions } from './ShellActions';
import { questionForPath, questionKey, questionState, stepState, surveySteps, stepForKind, type SurveyProgress } from '../../core/survey/surveyFlow';
import { defaultsWithoutReason } from '../../core/survey/surveyRegistration';
import { currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import { labelColor } from './pages/results/resultsData';
import type { StoredSurvey } from '../../core/nta/SurveyTemplates';
import { BuildNavList } from '../BuildFlow/BuildFlow';

/** What is not part of the numbered sequence of a new-build project (feedback 8 Oct 2026: "hoofdmenu erg warrig"). */
const BUILD_MORE: Array<{ route: Route; labelKey: string }> = [
  { route: { step: 'building', sub: 'model3d' }, labelKey: 'nav.sub.building.model3d' },
  { route: { step: 'advice' }, labelKey: 'nav.step.advice' },
];

/** One step of the numbered sequence after the input steps. */
function SequenceStep({ number, labelKey, target, current, status, onClick, t, note, children }: {
  number: number; labelKey: string; target: string; current: boolean; status: 'complete' | 'errors' | 'todo';
  onClick: () => void; t: Translate; note?: string; children?: ReactNode;
}) {
  const badge = status === 'complete' ? 'complete' : status === 'errors' ? 'errors' : current ? 'current' : 'todo';
  return <li>
    <button type="button" className="nav-step" aria-current={current ? 'page' : undefined} data-step={target} onClick={onClick}>
      <span className={`nav-step-no ${badge}`} aria-hidden="true">{badge === 'complete' ? <Check /> : badge === 'errors' ? '!' : number}</span>
      <span className="nav-step-label">{t(labelKey)}</span>
      <span className="visually-hidden">, {t(status === 'complete' ? 'nav.state.complete' : status === 'errors' ? 'nav.state.attention' : 'nav.state.todo')}</span>
    </button>
    {note && <p className="nav-step-note">{note}</p>}
    {current && children}
  </li>;
}

const stepStatusOf = (status: StepStatus | undefined): 'complete' | 'errors' | 'todo' =>
  status?.state === 'complete' ? 'complete' : status?.state === 'errors' ? 'errors' : 'todo';

/** The input flow of a new-build project, results and registration, and the few other parts. */
function BuildFlowNav({ route, statuses, navigate, t }: {
  route: Route; statuses: Record<StepId, StepStatus>; navigate: ShellActions['navigate']; t: Translate;
}) {
  const isHere = (target: Route) => route.step === target.step && (target.sub == null || route.sub === target.sub);
  const moreOpen = BUILD_MORE.some((item) => isHere(item.route));
  return <>
    <li className="nav-group-item">
      <ol aria-label={t('nav.sequence')} style={{ listStyle: 'none', margin: 0, padding: 0 }}>
        <BuildNavList route={route} statuses={statuses} navigate={navigate} />
        <SequenceStep number={5} labelKey="nav.step.resultsReport" target="results" t={t}
          current={route.step === 'results' || route.step === 'report'} status={stepStatusOf(statuses.results)}
          onClick={() => navigate({ step: 'results' })}>
          <ul className="nav-subs">
            {(['results', 'report'] as const).map((step) => <li key={step}>
              <button type="button" className="nav-sub" aria-current={route.step === step ? 'page' : undefined}
                onClick={() => navigate({ step })}>{t(`nav.step.${step}`)}</button>
            </li>)}
          </ul>
        </SequenceStep>
        <SequenceStep number={6} labelKey="nav.step.registration" target="registration" t={t}
          current={route.step === 'registration'} status={stepStatusOf(statuses.registration)}
          onClick={() => navigate({ step: 'registration' })} />
      </ol>
    </li>
    <li className="nav-group-item">
      <details className="nav-more" open={moreOpen}>
        <summary className="nav-group">{t('nav.more')}</summary>
        <ul className="nav-subs">
          {BUILD_MORE.map((item) => <li key={item.labelKey}>
            <button type="button" className="nav-sub" aria-current={isHere(item.route) ? 'page' : undefined}
              onClick={() => navigate(item.route)}>{t(item.labelKey)}</button>
          </li>)}
        </ul>
      </details>
    </li>
  </>;
}

type Translate = (key: string, params?: Record<string, string>) => string;
type StoredWithProgress = StoredSurvey & { progress?: SurveyProgress };

/** Label class and progress of the basisopname, for the project card. */
function SurveyCardStatus({ stored, t }: { stored: StoredWithProgress; t: Translate }) {
  const assessment = useSurveyAssessment();
  const label = currentResult(assessment, stored)?.performance?.indicativeLabelClass ?? null;
  const steps = surveySteps(stored.kind).filter((step) => !step.special);
  const done = steps.filter((step) => stepState(step, stored.progress) === 'done').length;
  return <div className="nav-survey-status">
    <span>{t('nav.survey.progress', { done: String(done), total: String(steps.length) })}</span>
    {label && <span className="nav-survey-label" style={{ background: labelColor(label) }} title={t('survey.result.title')}>{label}</span>}
  </div>;
}

/**
 * The numbered sequence of a basisopname project (feedback 8 Oct 2026):
 * Opname (its parts only while you are in it), Controle, Label en rapport,
 * Maatwerkadvies and Registratie; Herlabelen under "Meer".
 */
function SurveyNavList({ stored, route, statuses, navigate, t }: {
  stored: StoredWithProgress; route: Route; statuses: Record<StepId, StepStatus>;
  navigate: ShellActions['navigate']; t: Translate;
}) {
  const assessment = useSurveyAssessment();
  const result = currentResult(assessment, stored);
  const steps = surveySteps(stored.kind);
  const inputSteps = steps.filter((step) => !step.special);
  const currentStep = route.step === 'survey' ? stepForKind(route.sub, stored.kind) : null;
  const inInput = route.step === 'survey' && currentStep != null && currentStep !== 'controle' && currentStep !== 'label';
  const issues = (id: string) => result?.issues.filter((item) => questionForPath(item.path, stored).step === id).length ?? 0;
  const inputErrors = inputSteps.reduce((sum, step) => sum + issues(step.id), 0);
  const inputDone = inputSteps.every((step) => stepState(step, stored.progress) === 'done');
  const firstOpen = inputSteps.find((step) => stepState(step, stored.progress) !== 'done') ?? inputSteps[0];
  const reasonsMissing = result ? defaultsWithoutReason(stored, result) : 0;
  const blocked = !result?.performance || (result?.issues.length ?? 0) > 0;
  const checkDone = Boolean(stored.progress?.done?.includes('controle'));
  return <>
    <li className="nav-group-item">
      <ol aria-label={t('nav.sequence')} style={{ listStyle: 'none', margin: 0, padding: 0 }}>
        <li>
          <button type="button" className="nav-step" aria-current={inInput ? 'page' : undefined} data-step="survey-input"
            onClick={() => navigate({ step: 'survey', sub: firstOpen.id })}>
            <span className={`nav-step-no ${inputDone && inputErrors === 0 ? 'complete' : inputErrors > 0 ? 'errors' : inInput ? 'current' : 'todo'}`} aria-hidden="true">
              {inputDone && inputErrors === 0 ? <Check /> : inputErrors > 0 ? '!' : 1}
            </span>
            <span className="nav-step-label">{t('nav.survey.steps')}</span>
            {inputErrors > 0 && <span className="nav-count errors" aria-hidden="true">{inputErrors}</span>}
            <span className="visually-hidden">, {t(inputDone ? 'nav.state.complete' : 'nav.state.todo')}</span>
          </button>
          {inInput && <ul className="nav-subs">
            {inputSteps.map((step) => {
              const state = stepState(step, stored.progress);
              const errors = issues(step.id);
              const here = currentStep === step.id;
              return <li key={step.id}>
                <button type="button" className={`nav-sub nav-question nav-question--${state === 'done' ? 'done' : state === 'skipped' ? 'skipped' : 'todo'}`}
                  aria-current={here ? 'step' : undefined} data-survey-step={step.id} data-state={state}
                  onClick={() => navigate({ step: 'survey', sub: step.id })}>
                  <span className="nav-question-mark" aria-hidden="true">{state === 'done' ? '✓' : state === 'skipped' ? '↷' : here ? '›' : '·'}</span>
                  {t(step.labelKey)}
                  {errors > 0 && <span className="nav-count errors" aria-hidden="true">{errors}</span>}
                </button>
                {here && step.questions.length > 1 && <ul className="nav-subs">
                  {step.questions.map((question) => {
                    const answered = questionState(stored.progress, questionKey(step.id, question.id));
                    const at = (route.question ?? step.questions[0].id) === question.id;
                    return <li key={question.id}>
                      <button type="button" className={`nav-sub nav-question nav-question--${answered}`} aria-current={at ? 'step' : undefined}
                        onClick={() => navigate({ step: 'survey', sub: step.id, question: question.id })}>
                        <span className="nav-question-mark" aria-hidden="true">{answered === 'done' ? '✓' : answered === 'skipped' ? '↷' : at ? '›' : '·'}</span>
                        {t(`survey.navQuestion.${step.id}.${question.id}`)}
                      </button>
                    </li>;
                  })}
                </ul>}
              </li>;
            })}
          </ul>}
        </li>
        <SequenceStep number={2} labelKey="survey.step.controle" target="survey-check" t={t}
          current={route.step === 'survey' && currentStep === 'controle'}
          status={(result?.issues.length ?? 0) > 0 ? 'errors' : checkDone && result?.performance ? 'complete' : 'todo'}
          note={reasonsMissing > 0 ? t('nav.survey.reasonsMissing', { count: String(reasonsMissing) }) : undefined}
          onClick={() => navigate({ step: 'survey', sub: 'controle' })} />
        <SequenceStep number={3} labelKey="survey.step.label" target="survey-label" t={t}
          current={(route.step === 'survey' && currentStep === 'label') || route.step === 'report'}
          status={blocked ? 'todo' : 'complete'}
          onClick={() => navigate({ step: 'survey', sub: 'label' })} />
        <SequenceStep number={4} labelKey="nav.step.advice" target="advice" t={t}
          current={route.step === 'advice'} status={stepStatusOf(statuses.advice)}
          onClick={() => navigate({ step: 'advice' })} />
        <SequenceStep number={5} labelKey="nav.step.registration" target="registration" t={t}
          current={route.step === 'registration'} status={stepStatusOf(statuses.registration)}
          onClick={() => navigate({ step: 'registration' })} />
      </ol>
    </li>
    <li className="nav-group-item">
      <details className="nav-more" open={route.step === 'relabel'}>
        <summary className="nav-group">{t('nav.more')}</summary>
        <ul className="nav-subs">
          <li><button type="button" className="nav-sub" aria-current={route.step === 'relabel' ? 'page' : undefined}
            onClick={() => navigate({ step: 'relabel' })}>{t('nav.step.relabel')}</button></li>
        </ul>
      </details>
    </li>
  </>;
}

export interface WorkflowNavProps {
  project: IProject;
  route: Route;
  statuses: Record<StepId, StepStatus>;
  floorAreaM2?: number | null;
  actions: Pick<ShellActions, 'navigate' | 'openSettings' | 'openFeedback' | 'exportUNIEC3' | 'importUNIEC3'
    | 'exportVABI' | 'importVABI' | 'exportModelIFC'>;
}

/** Screen-reader text of a step status, e.g. "2 aandachtspunt(en)". */
export function stepStateText(t: (key: string, params?: Record<string, string>) => string, status: StepStatus): string {
  switch (status.state) {
    case 'errors': return t('nav.state.errors', { count: String(status.errors) });
    case 'warnings': return t('nav.state.warnings', { count: String(status.warnings) });
    case 'complete': return t('nav.state.complete');
    case 'dimmed': return t('nav.state.dimmed');
    default: return t('nav.state.todo');
  }
}

export function WorkflowNav({ project, route, statuses, floorAreaM2, actions }: WorkflowNavProps) {
  const { t, locale } = useI18n();
  const listRef = useRef<HTMLOListElement>(null);
  const tools = useMenu();
  const newBuild = isNewBuild(project);
  const utility = project.buildingFunction !== 'residential';

  // Roving focus over the step buttons: ↑/↓, Home and End.
  const onListKeyDown = (event: KeyboardEvent) => {
    if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key) || event.altKey) return;
    const buttons = Array.from(listRef.current?.querySelectorAll<HTMLButtonElement>('button.nav-step, button.nav-sub') ?? []);
    const index = buttons.indexOf(document.activeElement as HTMLButtonElement);
    if (index < 0) return;
    event.preventDefault();
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? buttons.length - 1
      : (index + (event.key === 'ArrowDown' ? 1 : -1) + buttons.length) % buttons.length;
    buttons[next]?.focus();
  };

  const city = project.city?.trim();
  const survey = isSurveyProject(project) ? project.basisopname as StoredWithProgress : null;

  return (
    <nav className="workflow-nav" aria-label={t('nav.ariaLabel')}>
      <div className="nav-project">
        <div className="nav-project-name" title={project.name}>{project.name || t('app.untitledProject')}</div>
        {(city || floorAreaM2 != null) && <div className="nav-project-meta">
          {city}{city && floorAreaM2 != null && ' · '}
          {floorAreaM2 != null && <>{formatNumber(floorAreaM2, locale, 1)} m² A<sub>g</sub></>}
        </div>}
        <div className="nav-project-tags">
          <Tag>{t(utility ? 'nav.tag.utility' : 'nav.tag.residential')}</Tag>
          {project.registration?.purpose && <Tag>{t(newBuild ? 'nav.tag.newBuild' : 'nav.tag.existing')}</Tag>}
        </div>
        {survey && <SurveyCardStatus stored={survey} t={t} />}
      </div>

      <ol className="nav-steps" ref={listRef} onKeyDown={onListKeyDown}>
        {survey && <SurveyNavList stored={survey} route={route} statuses={statuses} navigate={actions.navigate} t={t} />}
        {!survey && <BuildFlowNav route={route} statuses={statuses} navigate={actions.navigate} t={t} />}
      </ol>

      <div className="nav-foot">
        <button type="button" ref={tools.buttonRef} className="nav-step" aria-haspopup="menu" aria-expanded={tools.open}
          aria-current={route.step === 'tool' ? 'page' : undefined} onClick={() => tools.setOpen(!tools.open)}>
          <Wrench aria-hidden="true" /><span className="nav-step-label">{t('nav.tools')}</span><ChevronRight aria-hidden="true" />
        </button>
        {tools.open && (
          <div ref={tools.menuRef} role="menu" aria-label={t('nav.tools.menu')} className="shell-menu shell-menu--up" onKeyDown={tools.onKeyDown}>
            <div className="shell-menu-group" role="presentation">{t('nav.tools.calculators')}</div>
            {TOOL_STEP.subs.filter((sub) => !HELP_TOOL_SUBS.has(sub.id)).map((sub) => (
              <button key={sub.id} type="button" role="menuitem" className="shell-menu-item"
                onClick={tools.select(() => actions.navigate({ step: 'tool', sub: sub.id }))}>
                {sub.id === 'heat-pump-sizing' ? <Flame aria-hidden="true" /> : <Calculator aria-hidden="true" />}<span>{t(sub.labelKey)}</span>
              </button>
            ))}
            <div className="shell-menu-group" role="presentation">{t('nav.tools.exchange')}</div>
            <button type="button" role="menuitem" className="shell-menu-item" onClick={tools.select(actions.exportUNIEC3)}>
              <Download aria-hidden="true" /><span>{t('ribbon.exportUNIEC3Draft')}</span></button>
            <button type="button" role="menuitem" className="shell-menu-item" onClick={tools.select(actions.importUNIEC3)}>
              <Upload aria-hidden="true" /><span>{t('ribbon.importUNIEC3')}</span></button>
            <button type="button" role="menuitem" className="shell-menu-item" onClick={tools.select(actions.exportVABI)}>
              <Download aria-hidden="true" /><span>{t('ribbon.exportVABI')}</span></button>
            <button type="button" role="menuitem" className="shell-menu-item" onClick={tools.select(actions.importVABI)}>
              <Upload aria-hidden="true" /><span>{t('ribbon.importVABI')}</span></button>
            <button type="button" role="menuitem" className="shell-menu-item" onClick={tools.select(actions.exportModelIFC)}>
              <Box aria-hidden="true" /><span>{t('ribbon.exportModelIFC')}</span></button>
            <div className="shell-menu-sep" role="separator" />
            <button type="button" role="menuitem" className="shell-menu-item"
              onClick={tools.select(() => actions.navigate({ step: 'tool', sub: 'manual' }))}>
              <BookOpen aria-hidden="true" /><span>{t('manual.title')}</span></button>
            <button type="button" role="menuitem" className="shell-menu-item" onClick={tools.select(actions.openFeedback)}>
              <MessageSquare aria-hidden="true" /><span>{t('nav.feedback')}</span></button>
          </div>
        )}
        <button type="button" className="nav-step" onClick={actions.openSettings} aria-keyshortcuts="Control+,">
          <SlidersHorizontal aria-hidden="true" /><span className="nav-step-label">{t('nav.settings')}</span><Kbd>Ctrl ,</Kbd>
        </button>
      </div>
    </nav>
  );
}
