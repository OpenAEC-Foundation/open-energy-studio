/**
 * Numbered workflow navigation (ontwerp §3.2, mockup 01). Each step shows its
 * kernel-driven status; the active step has aria-current="page" and lists its
 * sub pages. The footer holds the compact Gereedschap menu (calculators,
 * exchange formats, feedback) and Instellingen — there is no tab strip of tools.
 */
import { useRef, type KeyboardEvent } from 'react';
import {
  Box, Calculator, Check, ChevronRight, Download, Flame, MessageSquare, SlidersHorizontal, Upload, Wrench,
} from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { Kbd, Tag } from '../ui';
import { useMenu } from './TopBar';
import { STEP_GROUPS, WORKFLOW_STEPS, TOOL_STEP, visibleSubs, type Route, type StepId } from '../../core/navigation/routes';
import { isNewBuild, isSurveyProject, type StepStatus } from '../../core/nta/stepStatus';
import type { IProject } from '../../core/energy/types';
import type { ShellActions } from './ShellActions';
import {
  questionForPath, questionKey, questionState, stepState, surveySteps, stepForKind, type SurveyProgress,
} from '../../core/survey/surveyFlow';
import { currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import { labelColor } from './pages/results/resultsData';
import type { StoredSurvey } from '../../core/nta/SurveyTemplates';

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

/** The steps of the basisopname with their state, the questions of the current step, and what comes after. */
function SurveyNavList({ stored, route, statuses, navigate, t }: {
  stored: StoredWithProgress; route: Route; statuses: Record<StepId, StepStatus>;
  navigate: ShellActions['navigate']; t: Translate;
}) {
  const assessment = useSurveyAssessment();
  const result = currentResult(assessment, stored);
  const steps = surveySteps(stored.kind);
  const currentStep = route.step === 'survey' ? stepForKind(route.sub, stored.kind) : null;
  const issues = (id: string) => result?.issues.filter((item) => questionForPath(item.path, stored).step === id).length ?? 0;
  const workflowStep = (step: typeof WORKFLOW_STEPS[number]) => {
    const status = statuses[step.id];
    const current = route.step === step.id;
    return <li key={step.id}>
      <button type="button" className="nav-step" aria-current={current ? 'page' : undefined} data-step={step.id}
        onClick={() => navigate({ step: step.id })}>
        <StepBadge status={status} number={step.number} current={current} />
        <span className="nav-step-label">{t(step.labelKey)}</span>
        <span className="visually-hidden">, {stepStateText(t, status)}</span>
      </button>
    </li>;
  };
  return <>
    <li className="nav-group-item">
      <div className="nav-group" id="nav-group-survey">{t('nav.survey.steps')}</div>
      <ol aria-labelledby="nav-group-survey" style={{ listStyle: 'none', margin: 0, padding: 0 }}>
        {steps.map((step, index) => {
          const state = stepState(step, stored.progress);
          const errors = issues(step.id);
          const current = currentStep === step.id;
          const badge = state === 'done' ? 'complete' : state === 'skipped' ? 'skipped' : errors > 0 ? 'errors' : current ? 'current' : 'todo';
          const stateText = state === 'done' ? t('nav.state.complete') : state === 'skipped' ? t('nav.state.skipped')
            : state === 'partial' ? t('nav.state.partial') : t('nav.state.todo');
          return <li key={step.id}>
            <button type="button" className="nav-step" aria-current={current ? 'page' : undefined} data-survey-step={step.id}
              data-state={state} onClick={() => navigate({ step: 'survey', sub: step.id })}>
              <span className={`nav-step-no ${badge}`} aria-hidden="true">
                {state === 'done' ? <Check /> : state === 'skipped' ? '↷' : index + 1}
              </span>
              <span className="nav-step-label">{t(step.labelKey)}</span>
              {errors > 0 && <span className="nav-count errors" aria-hidden="true">{errors}</span>}
              {errors === 0 && state === 'skipped' && <span className="nav-later" aria-hidden="true">{t('survey.open.skipped')}</span>}
              <span className="visually-hidden">, {stateText}{errors > 0 ? `, ${t('nav.state.errors', { count: String(errors) })}` : ''}</span>
            </button>
            {current && step.questions.length > 1 && <ul className="nav-subs">
              {step.questions.map((question) => {
                const answered = questionState(stored.progress, questionKey(step.id, question.id));
                const here = (route.question ?? step.questions[0].id) === question.id;
                return <li key={question.id}>
                  <button type="button" className={`nav-sub nav-question nav-question--${answered}`} aria-current={here ? 'step' : undefined}
                    onClick={() => navigate({ step: 'survey', sub: step.id, question: question.id })}>
                    <span className="nav-question-mark" aria-hidden="true">{answered === 'done' ? '✓' : answered === 'skipped' ? '↷' : here ? '›' : '·'}</span>
                    {t(`survey.navQuestion.${step.id}.${question.id}`)}
                  </button>
                </li>;
              })}
            </ul>}
          </li>;
        })}
      </ol>
    </li>
    <li className="nav-group-item">
      <div className="nav-group" id="nav-group-after">{t('nav.survey.after')}</div>
      <ol aria-labelledby="nav-group-after" style={{ listStyle: 'none', margin: 0, padding: 0 }}>
        {WORKFLOW_STEPS.filter((step) => ['advice', 'relabel', 'report', 'registration'].includes(step.id)).map(workflowStep)}
      </ol>
    </li>
    <li className="nav-group-item">
      <details className="nav-more" open={['project', 'building', 'installations', 'check', 'results'].includes(route.step)}>
        <summary className="nav-group">{t('nav.survey.more')}</summary>
        <ol style={{ listStyle: 'none', margin: 0, padding: 0 }}>
          {WORKFLOW_STEPS.filter((step) => ['project', 'building', 'installations', 'check', 'results'].includes(step.id)).map(workflowStep)}
        </ol>
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

function StepBadge({ status, number, current }: { status: StepStatus; number: number; current: boolean }) {
  const kind = status.state === 'complete' ? 'complete'
    : status.state === 'errors' ? 'errors'
      : status.state === 'warnings' ? 'warnings'
        : current ? 'current' : status.state === 'dimmed' ? 'dimmed' : 'todo';
  return (
    <span className={`nav-step-no ${kind}`} aria-hidden="true">
      {kind === 'complete' ? <Check /> : number}
    </span>
  );
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
        {!survey && STEP_GROUPS.map((group) => (
          <li key={group.id} className="nav-group-item">
            <div className="nav-group" id={`nav-group-${group.id}`}>{t(group.labelKey)}</div>
            <ol aria-labelledby={`nav-group-${group.id}`} style={{ listStyle: 'none', margin: 0, padding: 0 }}>
              {WORKFLOW_STEPS.filter((step) => step.group === group.id).map((step) => {
                const status = statuses[step.id];
                const current = route.step === step.id;
                const count = status.errors > 0 ? status.errors : status.warnings;
                return (
                  <li key={step.id}>
                    <button type="button" className={`nav-step${status.dimmed ? ' is-dimmed' : ''}`}
                      aria-current={current ? 'page' : undefined} data-step={step.id} data-state={status.state}
                      onClick={() => actions.navigate({ step: step.id })}>
                      <StepBadge status={status} number={step.number} current={current} />
                      <span className="nav-step-label">{t(step.labelKey)}</span>
                      {count > 0 && <span className={`nav-count ${status.errors > 0 ? 'errors' : 'warnings'}`} aria-hidden="true">{count}</span>}
                      <span className="visually-hidden">, {t('nav.stepNumber', { number: String(step.number) })}, {stepStateText(t, status)}</span>
                    </button>
                    {current && visibleSubs(step, project).length > 0 && (
                      <ul className="nav-subs">
                        {visibleSubs(step, project).map((sub) => (
                          <li key={sub.id}>
                            <button type="button" className="nav-sub" aria-current={route.sub === sub.id ? 'page' : undefined}
                              onClick={() => actions.navigate({ step: step.id, sub: sub.id })}>{t(sub.labelKey)}</button>
                          </li>
                        ))}
                      </ul>
                    )}
                  </li>
                );
              })}
            </ol>
          </li>
        ))}
      </ol>

      <div className="nav-foot">
        <button type="button" ref={tools.buttonRef} className="nav-step" aria-haspopup="menu" aria-expanded={tools.open}
          aria-current={route.step === 'tool' ? 'page' : undefined} onClick={() => tools.setOpen(!tools.open)}>
          <Wrench aria-hidden="true" /><span className="nav-step-label">{t('nav.tools')}</span><ChevronRight aria-hidden="true" />
        </button>
        {tools.open && (
          <div ref={tools.menuRef} role="menu" aria-label={t('nav.tools.menu')} className="shell-menu shell-menu--up" onKeyDown={tools.onKeyDown}>
            <div className="shell-menu-group" role="presentation">{t('nav.tools.calculators')}</div>
            {TOOL_STEP.subs.map((sub) => (
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
