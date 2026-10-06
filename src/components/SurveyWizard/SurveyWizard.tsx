/**
 * The basisopname as a question flow (UI redesign 2026-10, mockup B and C):
 * one question per screen, choice cards for the main questions, "skip, fill in
 * later", a live provisional label, a Controle page with everything on one
 * page and a Label page. The fields themselves are the BasisopnamePanel parts.
 */
import { useEffect, useMemo, type ReactNode } from 'react';
import {
  Ban, Building2, CircleHelp, Factory, Fan, Flame, Home, Network, Plug, Thermometer, TreePine, Wind, Zap,
} from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import {
  BasisopnamePanel, HEATING_KINDS, HOT_WATER_KINDS, VENTILATION_PRINCIPLES, defaultPathLabel, defaultValueLabel,
} from '../BasisopnamePanel/BasisopnamePanel';
import { read, SelectField, write, type Draft, type Path } from '../NtaPerformancePanel/NtaFormFields';
import {
  heatingGeneratorTemplate, hotWaterGeneratorTemplate, surveyTemplate, type StoredSurvey, type SurveyKind,
} from '../../core/nta/SurveyTemplates';
import {
  markProgress, questionForPath, questionKey, questionState, stepForKind, stepState, surveyStep, surveySteps,
  type SurveyFlowStep, type SurveyPart, type SurveyProgress, type SurveyStepId,
} from '../../core/survey/surveyFlow';
import { assessSurvey, currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import { labelColor } from '../shell/pages/results/resultsData';
import { KernelCode } from '../KernelCode/KernelCode';
import type { Route } from '../../core/navigation/routes';
import type { OpnameAssessment } from '../../core/nta/KernelClient';
import './SurveyWizard.css';

type T = (key: string, options?: Record<string, unknown>) => string;
type Stored = StoredSurvey & { progress?: SurveyProgress };

export interface SurveyWizardProps {
  route: Route;
  navigate: (route: Route) => void;
}

const HEATING_ICONS: Record<string, ReactNode> = {
  boiler: <Flame />, heat_pump: <Thermometer />, district_heat: <Network />, electric: <Zap />, biomass: <TreePine />,
  chp: <Factory />, local_fired: <Flame />, gas_air_heater: <Wind />, none_present: <Ban />,
};
const HOT_WATER_ICONS: Record<string, ReactNode> = {
  gas_appliance: <Flame />, gas_storage_heater: <Flame />, electric_boiler: <Plug />, electric_instantaneous: <Zap />,
  heat_pump: <Thermometer />, district_heat: <Network />, collective_unknown: <Building2 />, delivery_set_from_heating: <Network />,
  none: <Ban />,
};
const VENTILATION_ICONS: Record<string, ReactNode> = {
  natural: <Wind />, mechanical_extract: <Fan />, balanced: <Fan />, mechanical_supply: <Fan />,
};
const DWELLINGS = ['terraced', 'end_or_corner', 'detached', 'apartment'] as const;

/** The dwelling type of the survey as one of the choice cards. */
function dwellingChoice(draft: Draft): string | null {
  const kind = read(draft, ['dwelling', 'kind']);
  if (kind === 'apartment') return 'apartment';
  const position = read(draft, ['dwelling', 'position']);
  return typeof position === 'string' && position !== 'unknown' ? position : null;
}

function ChoiceCards({ options, selected, onPick, label }: {
  options: Array<{ id: string; title: string; hint: string; icon: ReactNode }>;
  selected: string | null;
  onPick: (id: string) => void;
  label: string;
}) {
  return <div className="survey-choices" role="group" aria-label={label}>
    {options.map((option) => (
      <button key={option.id} type="button" className="survey-choice" aria-pressed={selected === option.id}
        onClick={() => onPick(option.id)}>
        <span className="survey-choice-icon" aria-hidden="true">{option.icon}</span>
        <span className="survey-choice-title">{option.title}</span>
        <span className="survey-choice-hint">{option.hint}</span>
      </button>
    ))}
  </div>;
}

/** The fields of one question: choice cards, or the matching part of the survey form. */
function QuestionBody({ part, stored, draft, change, t }: {
  part: SurveyPart; stored: Stored; draft: Draft; change: (path: Path, value: unknown) => void; t: T;
}) {
  const field = { draft, onChange: change };
  switch (part) {
    case 'dwellingType': {
      const choice = dwellingChoice(draft);
      const pick = (id: string) => {
        if (id === 'apartment') {
          change(['dwelling'], {
            kind: 'apartment',
            floor: read(draft, ['dwelling', 'floor']) ?? 'ground_or_intermediate',
            side: read(draft, ['dwelling', 'side']) ?? 'middle',
          });
        } else {
          change(['dwelling'], { kind: 'single_family', position: id, roofType: read(draft, ['dwelling', 'roofType']) ?? 'pitched' });
        }
      };
      return <>
        <ChoiceCards label={t('survey.q.woning.soort')} selected={choice} onPick={pick}
          options={DWELLINGS.map((id) => ({
            id, title: t(`survey.dwelling.${id}`), hint: t(`survey.dwelling.${id}.hint`),
            icon: id === 'apartment' ? <Building2 /> : <Home />,
          }))} />
        <div className="nta-form survey-followup">
          {choice === 'apartment' ? <>
            <SelectField {...field} path={['dwelling', 'floor']} label={t('opname.dwelling.floor')}
              options={['ground_or_intermediate', 'top', 'roof_and_floor'].map((key) => [key, t(`opname.dwelling.floorKind.${key}`)])} />
            <SelectField {...field} path={['dwelling', 'side']} label={t('survey.dwelling.side')}
              options={['middle', 'end_or_corner', 'unknown'].map((key) => [key, t(`survey.dwelling.sideKind.${key}`)])} />
          </> : choice != null && <SelectField {...field} path={['dwelling', 'roofType']} label={t('survey.dwelling.roofType')}
            options={['pitched', 'partly_flat', 'flat'].map((key) => [key, t(`survey.dwelling.roofTypeKind.${key}`)])} />}
        </div>
      </>;
    }
    case 'heatingKind': {
      const current = read(draft, ['heating', 'generator', 'kind']);
      return <ChoiceCards label={t('survey.q.verwarming.toestel')} selected={typeof current === 'string' ? current : null}
        onPick={(id) => { if (id !== current) change(['heating', 'generator'], heatingGeneratorTemplate(id)); }}
        options={HEATING_KINDS.map((id) => ({
          id, title: t(`opname.heating.kind.${id}`), hint: t(`survey.heating.${id}.hint`), icon: HEATING_ICONS[id] ?? <CircleHelp />,
        }))} />;
    }
    case 'hotWaterKind': {
      const current = read(draft, ['hotWater', 'generator', 'kind']);
      return <ChoiceCards label={t('survey.q.warm-water.toestel')} selected={typeof current === 'string' ? current : null}
        onPick={(id) => { if (id !== current) change(['hotWater', 'generator'], hotWaterGeneratorTemplate(id)); }}
        options={HOT_WATER_KINDS[stored.kind].map((id) => ({
          id, title: t(`opname.hotWater.kind.${id}`), hint: t(`survey.hotWater.${id}.hint`), icon: HOT_WATER_ICONS[id] ?? <CircleHelp />,
        }))} />;
    }
    case 'ventilationPrinciple': {
      const current = read(draft, ['ventilation', 'principle']);
      return <ChoiceCards label={t('survey.q.ventilatie.systeem')} selected={typeof current === 'string' ? current : null}
        onPick={(id) => change(['ventilation', 'principle'], id)}
        options={VENTILATION_PRINCIPLES.map((id) => ({
          id, title: t(`survey.ventilation.principleKind.${id}`), hint: t(`survey.ventilation.${id}.hint`),
          icon: VENTILATION_ICONS[id] ?? <Fan />,
        }))} />;
    }
    default:
      return <BasisopnamePanel part={part} />;
  }
}

/** Short facts of a step for the Controle page. */
function stepFacts(step: SurveyStepId, draft: Draft, t: T, locale: string): string[] {
  const n = (value: unknown, digits = 0) => typeof value === 'number' ? formatNumber(value, locale, digits) : '—';
  const surfaces = (read(draft, ['envelope', 'surfaces']) as Array<Record<string, unknown>> | undefined) ?? [];
  const area = (items: Array<Record<string, unknown>>, key: string) => items.reduce((sum, item) => sum + (Number(item[key]) || 0), 0);
  switch (step) {
    case 'woning': {
      const choice = dwellingChoice(draft);
      return [choice ? t(`survey.dwelling.${choice}`) : '—',
        t('survey.fact.year', { year: n(read(draft, ['constructionYear'])) }),
        t('survey.fact.area', { area: n(read(draft, ['usableFloorAreaM2']), 1) })];
    }
    case 'gebouw': {
      const functions = (read(draft, ['functions']) as Array<Record<string, unknown>> | undefined) ?? [];
      return [t('survey.fact.year', { year: n(read(draft, ['constructionYear'])) }),
        t('survey.fact.functions', { count: functions.length, area: n(area(functions, 'areaM2'), 0) })];
    }
    case 'zones': return [t('survey.fact.zones', { count: ((read(draft, ['zones']) as unknown[]) ?? []).length })];
    case 'gevels': {
      const facades = surfaces.filter((surface) => surface.element === 'facade');
      const windows = (read(draft, ['envelope', 'windows']) as Array<Record<string, unknown>> | undefined) ?? [];
      return [t('survey.fact.facades', { count: facades.length, area: n(area(facades, 'grossAreaM2'), 1) }),
        t('survey.fact.windows', { count: windows.length, area: n(area(windows, 'areaM2'), 1) })];
    }
    case 'dak-vloer': {
      const roofs = surfaces.filter((surface) => surface.element === 'roof');
      const floors = surfaces.filter((surface) => surface.element === 'floor');
      return [t('survey.fact.roofs', { count: roofs.length, area: n(area(roofs, 'grossAreaM2'), 1) }),
        t('survey.fact.floors', { count: floors.length, area: n(area(floors, 'grossAreaM2'), 1) })];
    }
    case 'verwarming': {
      const kind = read(draft, ['heating', 'generator', 'kind']);
      const emitters = read(draft, ['heating', 'emitters']);
      return [kind ? t(`opname.heating.kind.${String(kind)}`) : '—', emitters ? t(`opname.heating.emitters.${String(emitters)}`) : '—'];
    }
    case 'warm-water': {
      const kind = read(draft, ['hotWater', 'generator', 'kind']);
      return [kind ? t(`opname.hotWater.kind.${String(kind)}`) : '—'];
    }
    case 'ventilatie': {
      const principle = read(draft, ['ventilation', 'principle']);
      return [principle ? t(`survey.ventilation.principleKind.${String(principle)}`) : '—'];
    }
    case 'koeling': return [read(draft, ['cooling']) != null ? t('survey.fact.cooling') : t('survey.fact.noCooling')];
    case 'zonnepanelen': {
      const pv = (read(draft, ['pv']) as Array<Record<string, unknown>> | undefined) ?? [];
      return [pv.length === 0 ? t('survey.fact.noPv') : t('survey.fact.pv', { count: pv.length, area: n(area(pv, 'panelAreaM2'), 1) })];
    }
    default: return [];
  }
}

function ResultCard({ result, busy, error, t, locale }: {
  result: OpnameAssessment | null; busy: boolean; error: string | null; t: T; locale: string;
}) {
  const performance = result?.performance;
  const label = performance?.indicativeLabelClass ?? null;
  return <div className="survey-card" aria-live="polite">
    <span className="survey-overline">{t('survey.result.title')}</span>
    {label ? <div className="survey-label-row">
      <span className="survey-label" style={{ background: labelColor(label) }}>{label}</span>
      <span>{t('survey.result.ep2')}<br /><strong>{formatNumber(performance?.primaryFossilIndicatorKwhPerM2Year, locale, 1)}</strong> {t('unit.kwhPerM2Year')}</span>
    </div> : <p className="survey-muted">{busy ? t('survey.result.busy') : error ? t('survey.result.error') : t('survey.result.none')}</p>}
    {result && result.issues.length > 0 && <p className="survey-issues-note">{t('survey.result.issues', { count: result.issues.length })}</p>}
    <p className="survey-muted">{t('survey.result.note')}</p>
  </div>;
}

/** Everything on one page: the outcome, a card per step, the kernel notices and the defaults. */
function CheckPage({ stored, steps, result, onEdit, onGoToPath, t, locale }: {
  stored: Stored; steps: SurveyFlowStep[]; result: OpnameAssessment | null;
  onEdit: (step: SurveyStepId) => void; onGoToPath: (path: string) => void; t: T; locale: string;
}) {
  const draft = stored.survey as Draft;
  const performance = result?.performance;
  const issuesFor = (step: SurveyStepId) => result?.issues.filter((item) => questionForPath(item.path, stored).step === step).length ?? 0;
  return <div className="survey-check">
    <section className="survey-card survey-outcome" aria-label={t('survey.check.outcome')}>
      {performance?.indicativeLabelClass
        ? <span className="survey-label survey-label--big" style={{ background: labelColor(performance.indicativeLabelClass) }}>
          {performance.indicativeLabelClass}</span>
        : <span className="survey-muted">{t('survey.result.none')}</span>}
      <dl className="survey-indicators">
        <div><dt>{t('survey.result.ep2')}</dt><dd>{formatNumber(performance?.primaryFossilIndicatorKwhPerM2Year, locale, 1)}</dd></div>
        <div><dt>BENG 1</dt><dd>{formatNumber(performance?.needIndicatorKwhPerM2Year, locale, 1)}</dd></div>
        <div><dt>BENG 3</dt><dd>{formatNumber(performance?.renewableSharePercent, locale, 1)} %</dd></div>
        <div><dt>{t('survey.check.defaults')}</dt><dd>{result?.appliedDefaults.length ?? '—'}</dd></div>
      </dl>
    </section>

    <div className="survey-check-grid">
      {steps.filter((step) => !step.special).map((step) => {
        const state = stepState(step, stored.progress);
        const issues = issuesFor(step.id);
        return <section key={step.id} className="survey-card survey-check-card" aria-label={t(step.labelKey)}>
          <div className="survey-check-head">
            <h2>{t(step.labelKey)}</h2>
            <button type="button" className="btn btn-sm" onClick={() => onEdit(step.id)}>{t('survey.check.edit')}</button>
          </div>
          <ul>{stepFacts(step.id, draft, t, locale).map((fact, index) => <li key={index}>{fact}</li>)}</ul>
          {state === 'skipped' && <p className="survey-skipped-note">{t('survey.check.skipped')}</p>}
          {issues > 0 && <p className="survey-issues-note">{t('survey.result.issues', { count: issues })}</p>}
        </section>;
      })}
    </div>

    {result && result.issues.length > 0 && <section className="survey-card" aria-label={t('survey.check.notices')}>
      <h2>{t('survey.check.notices')}</h2>
      <ul className="survey-notices">
        {result.issues.map((item, index) => {
          const target = questionForPath(item.path, stored);
          return <li key={index}>
            <span><KernelCode code={item.code} prefixes={['opname.issue.', 'nta.gap.', 'kernel.issue.']} /></span>
            <button type="button" className="btn btn-sm" onClick={() => onGoToPath(item.path)}>
              {t('survey.check.goTo', { step: t(`survey.step.${target.step}`) })}</button>
          </li>;
        })}
      </ul>
    </section>}

    {result && result.appliedDefaults.length > 0 && <details className="survey-card survey-defaults">
      <summary>{t('survey.check.defaultsList', { count: result.appliedDefaults.length })}</summary>
      <table>
        <thead><tr><th>{t('opname.defaults.path')}</th><th>{t('opname.defaults.value')}</th></tr></thead>
        <tbody>{result.appliedDefaults.map((item, index) => <tr key={index}>
          <td>{defaultPathLabel(t, item.path)}</td><td>{defaultValueLabel(t, item.value, locale)}</td>
        </tr>)}</tbody>
      </table>
    </details>}

    <details className="survey-card survey-all-fields">
      <summary>{t('survey.check.allFields')}</summary>
      <BasisopnamePanel />
    </details>
  </div>;
}

export function SurveyWizard({ route, navigate }: SurveyWizardProps) {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const stored = state.project.basisopname as Stored | undefined;
  const assessment = useSurveyAssessment();

  const kind: SurveyKind = stored?.kind ?? 'residential';
  const steps = useMemo(() => surveySteps(kind), [kind]);
  const stepId = stepForKind(route.sub, kind);
  const step = surveyStep(stepId, kind);
  const stepIndex = steps.findIndex((candidate) => candidate.id === step.id);
  const surveyJson = stored ? JSON.stringify(stored.survey) : null;

  // Recalculate after each change, once the typing pauses.
  useEffect(() => {
    if (!stored) return undefined;
    const timer = window.setTimeout(() => { void assessSurvey(stored); }, 400);
    return () => window.clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [surveyJson]);

  // The question is part of the route, so the navigation can show it; "Ga naar" lands on the
  // question of the kernel path. A step without a question starts at its first.
  const requestedQuestion = route.question
    ?? (stored && route.focusPath ? questionForPath(route.focusPath, stored).question : undefined);
  const questionIndex = Math.max(0, step.questions.findIndex((question) => question.id === requestedQuestion));
  const setQuestionIndex = (index: number) =>
    navigate({ step: 'survey', sub: step.id, question: step.questions[index]?.id });

  if (!stored) {
    const start = (surveyKind: SurveyKind) => {
      dispatch({ type: 'SET_BASISOPNAME', payload: { ...surveyTemplate(surveyKind), progress: {} } });
      navigate({ step: 'survey', sub: surveySteps(surveyKind)[0].id });
    };
    return <div className="survey-start">
      <h1>{t('survey.start.title')}</h1>
      <p className="survey-muted">{t('survey.start.lead')}</p>
      <div className="survey-choices">
        <button type="button" className="survey-choice" onClick={() => start('residential')}>
          <span className="survey-choice-icon" aria-hidden="true"><Home /></span>
          <span className="survey-choice-title">{t('survey.start.residential')}</span>
          <span className="survey-choice-hint">{t('survey.start.residentialHint')}</span>
        </button>
        <button type="button" className="survey-choice" onClick={() => start('utility')}>
          <span className="survey-choice-icon" aria-hidden="true"><Building2 /></span>
          <span className="survey-choice-title">{t('survey.start.utility')}</span>
          <span className="survey-choice-hint">{t('survey.start.utilityHint')}</span>
        </button>
      </div>
    </div>;
  }

  const draft = stored.survey as Draft;
  const result = currentResult(assessment, stored);
  const save = (next: Partial<Stored>) => dispatch({ type: 'SET_BASISOPNAME', payload: { ...stored, ...next } });
  const change = (path: Path, value: unknown) => save({ survey: write(draft, path, value) });
  const goToStep = (id: SurveyStepId) => navigate({ step: 'survey', sub: id });
  const goToPath = (path: string) => {
    const target = questionForPath(path, stored);
    navigate({ step: 'survey', sub: target.step, focusPath: `basisopname.${path.replace(/^basisopname\./, '')}` });
  };

  const question = step.questions[Math.min(questionIndex, Math.max(0, step.questions.length - 1))];
  const lastQuestion = questionIndex >= step.questions.length - 1;
  const nextStep = steps[stepIndex + 1];
  const previousStep = steps[stepIndex - 1];

  const advance = (mark: 'done' | 'skipped' | null) => {
    let progress = stored.progress;
    if (mark) progress = markProgress(progress, step.special ? step.id : questionKey(step.id, question.id), mark);
    if (progress !== stored.progress) save({ progress });
    if (!step.special && !lastQuestion) setQuestionIndex(questionIndex + 1);
    else if (nextStep) goToStep(nextStep.id);
  };
  const back = () => {
    if (!step.special && questionIndex > 0) setQuestionIndex(questionIndex - 1);
    // Back from the first question lands on the last question of the previous step.
    else if (previousStep) navigate({ step: 'survey', sub: previousStep.id, question: previousStep.questions[previousStep.questions.length - 1]?.id });
  };
  const nextLabel = !step.special && !lastQuestion
    ? t('survey.nav.nextQuestion')
    : nextStep ? t('survey.nav.next', { step: t(nextStep.labelKey).toLowerCase() }) : t('survey.nav.finish');

  const open = steps.filter((candidate) => candidate.id !== step.id && !candidate.special
    && stepState(candidate, stored.progress) !== 'done');

  return <div className="survey-wizard">
    <div className="survey-main">
      <p className="survey-crumb">{t('survey.crumb', {
        number: stepIndex + 1, total: steps.length, step: t(step.labelKey),
      })}{!step.special && step.questions.length > 1 && ` · ${t('survey.crumbQuestion', { number: questionIndex + 1, total: step.questions.length })}`}</p>

      {step.special === 'check' && <>
        <h1>{t('survey.check.title')}</h1>
        <p className="survey-lead">{t('survey.check.lead')}</p>
        <CheckPage stored={stored} steps={steps} result={result} onEdit={goToStep} onGoToPath={goToPath} t={t} locale={locale} />
      </>}

      {step.special === 'label' && <>
        <h1>{t('survey.label.title')}</h1>
        <p className="survey-lead">{t('survey.label.lead')}</p>
        <section className="survey-card survey-outcome">
          {result?.performance?.indicativeLabelClass
            ? <span className="survey-label survey-label--big" style={{ background: labelColor(result.performance.indicativeLabelClass) }}>
              {result.performance.indicativeLabelClass}</span>
            : <span className="survey-muted">{t('survey.result.none')}</span>}
          <p>{t('survey.label.status')}</p>
        </section>
        <div className="survey-next-actions">
          <button type="button" className="btn" onClick={() => navigate({ step: 'advice' })}>{t('survey.label.advice')}</button>
          <button type="button" className="btn" onClick={() => navigate({ step: 'report' })}>{t('survey.label.report')}</button>
          <button type="button" className="btn btn-primary" onClick={() => navigate({ step: 'registration' })}>{t('survey.label.registration')}</button>
        </div>
      </>}

      {!step.special && question && <>
        <h1>{t(question.titleKey)}</h1>
        <p className="survey-lead">{t(question.helpKey)}</p>
        <div className="survey-question" key={`${step.id}.${question.id}`}>
          <QuestionBody part={question.part} stored={stored} draft={draft} change={change} t={t} />
        </div>
        {questionState(stored.progress, questionKey(step.id, question.id)) === 'skipped' &&
          <p className="survey-skipped-note">{t('survey.skippedNote')}</p>}
      </>}

      {step.special !== 'label' && <div className="survey-actions">
        <button type="button" className="btn" onClick={back} disabled={stepIndex === 0 && questionIndex === 0}>{t('survey.nav.previous')}</button>
        <div className="survey-actions-right">
          {!step.special && <button type="button" className="btn survey-skip" onClick={() => advance('skipped')}>{t('survey.nav.skip')}</button>}
          <button type="button" className="btn btn-primary" onClick={() => advance('done')}>{nextLabel}</button>
        </div>
      </div>}
    </div>

    <aside className="survey-aside" aria-label={t('survey.aside')}>
      <ResultCard result={result} busy={assessment.busy} error={assessment.error} t={t} locale={locale} />
      <div className="survey-card">
        <span className="survey-overline">{t('survey.open.title')}</span>
        {open.length === 0 ? <p className="survey-done">{t('survey.open.none')}</p> : <ul className="survey-open">
          {open.map((candidate) => {
            const candidateState = stepState(candidate, stored.progress);
            return <li key={candidate.id}>
              <button type="button" onClick={() => goToStep(candidate.id)}>
                <span>{t(candidate.labelKey)}</span>
                <span className={`survey-open-tag survey-open-tag--${candidateState}`}>{t(`survey.open.${candidateState}`)}</span>
              </button>
            </li>;
          })}
        </ul>}
      </div>
      <p className="survey-defaults-note">{t('survey.defaultsNote')}</p>
    </aside>
  </div>;
}
