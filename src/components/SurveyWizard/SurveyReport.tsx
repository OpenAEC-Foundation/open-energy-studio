/**
 * Rapport en dossier of a basisopname project (feedback 8 Oct 2026): the report
 * is built from the survey outcome, not from the (empty) new-build calculation.
 * One choice of level on three cards, print or save as pdf, and the preview of
 * the report itself; the input file, the BRL 9500 checklist and the exports
 * stay available in a fold.
 */
import { useEffect, useRef, useState } from 'react';
import { FileDown, Printer } from 'lucide-react';
import './SurveyWizard.css';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import { assessSurvey, currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import { surveySteps, type SurveyStepId } from '../../core/survey/surveyFlow';
import type { StoredSurvey } from '../../core/nta/SurveyTemplates';
import type { Draft } from '../NtaPerformancePanel/NtaFormFields';
import { labelColor } from '../shell/pages/results/resultsData';
import { DossierPage, ExportsPage, InputDossierPage } from '../shell/pages/DeliveryPages';
import type { ShellActions } from '../shell/ShellActions';
import { defaultStep, projectAddress, stepFacts, StepDefaults, SurveyBlocked, surveyBlocked } from './SurveyWizard';
import { questionForPath } from '../../core/survey/surveyFlow';
import { useKernel } from '../../context/KernelProvider';
import { elementToPdf, fileNamePart, savePdf } from '../../core/report/pdf';
import { version } from '../../../package.json';

/** SHA-256 of a text, hex; null without WebCrypto. */
async function sha256(text: string): Promise<string | null> {
  try {
    const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(text));
    return Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, '0')).join('');
  } catch {
    return null;
  }
}

type Level = 'summary' | 'standard' | 'detailed';
const LEVEL_KEY = 'oes.surveyReport.level';
const LEVELS: Level[] = ['summary', 'standard', 'detailed'];

function storedLevel(): Level {
  try {
    const value = window.localStorage.getItem(LEVEL_KEY);
    return LEVELS.includes(value as Level) ? value as Level : 'standard';
  } catch {
    return 'standard';
  }
}

interface MonthRow { month: number; service: string; usedKwh: number; primaryFossilKwh: number }

export function SurveyReport({ actions }: { actions: ShellActions }) {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const project = state.project;
  const stored = project.basisopname as (StoredSurvey & { surveyDate?: string }) | undefined;
  const assessment = useSurveyAssessment();
  const [level, setLevel] = useState<Level>(storedLevel);
  const surveyJson = stored ? JSON.stringify(stored.survey) : null;
  const kernel = useKernel();
  const docRef = useRef<HTMLElement>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fingerprint, setFingerprint] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    if (surveyJson) void sha256(surveyJson).then((hash) => { if (live) setFingerprint(hash); });
    return () => { live = false; };
  }, [surveyJson]);

  // The report needs the survey outcome, also when the survey pages were not opened in this session.
  useEffect(() => {
    if (stored) void assessSurvey(stored);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [surveyJson]);

  if (!stored) return null;
  const result = currentResult(assessment, stored);
  if (surveyBlocked(result)) {
    const goToPath = (path: string) => actions.navigate({ step: 'survey', sub: questionForPath(path, stored).step,
      focusPath: `basisopname.${path.replace(/^basisopname\./, '')}` });
    return <div className="survey-report">
      <h1 id="page-title" tabIndex={-1}>{t('survey.report.title')}</h1>
      <SurveyBlocked result={result} busy={assessment.busy} stored={stored} onGoToPath={goToPath} t={t} />
    </div>;
  }
  const performance = result?.performance;
  const draft = stored.survey as Draft;
  const steps = surveySteps(stored.kind).filter((step) => !step.special);
  const address = projectAddress(project);
  const choose = (next: Level) => {
    setLevel(next);
    try { window.localStorage.setItem(LEVEL_KEY, next); } catch { /* per-viewer convenience only */ }
  };
  const n = (value: number | null | undefined, digits: number) => formatNumber(value, locale, digits);
  const label = performance?.indicativeLabelClass ?? null;
  const months = ((performance as unknown as { energyByService?: { months?: MonthRow[] } } | null)?.energyByService?.months ?? []);
  const pvMonths = ((performance as unknown as { pvSystems?: Array<{ monthlyKwh?: number[] }> } | null)?.pvSystems ?? [])
    .reduce((sum, system) => sum.map((value, index) => value + (system.monthlyKwh?.[index] ?? 0)), Array<number>(12).fill(0));
  const services = ['heating', 'hotWater', 'ventilation', 'auxiliary', 'cooling'].filter((service) => months.some((row) => row.service === service));
  const used = (month: number, service: string) => months.filter((row) => row.month === month && row.service === service)
    .reduce((sum, row) => sum + row.usedKwh, 0);
  const year = typeof draft.constructionYear === 'number' ? String(draft.constructionYear) : '—';
  const registration = project.registration;
  const evidence = registration?.evidence ?? [];
  // Evidence linked to the survey (JSON pointer /basisopname/survey/...) names its step.
  const evidencePart = (paths: string[] | undefined) => {
    const pointer = paths?.find((path) => path.startsWith('/basisopname/survey/'));
    if (!pointer) return '—';
    const surveyPath = pointer.slice('/basisopname/survey/'.length).split('/')
      .map((part, index) => (/^\d+$/.test(part) ? `[${part}]` : `${index ? '.' : ''}${part}`)).join('');
    return t(`survey.step.${questionForPath(surveyPath, stored).step}`);
  };

  return <div className="survey-report">
    <div className="survey-report-head">
      <div>
        <h1 id="page-title" tabIndex={-1}>{t('survey.report.title')}</h1>
        <p className="survey-lead">{t('survey.report.lead')}</p>
      </div>
      <div className="survey-report-buttons">
        <button type="button" className="btn" onClick={() => window.print()}><Printer aria-hidden="true" /> {t('survey.report.print')}</button>
        <button type="button" className="btn btn-primary" disabled={busy} onClick={() => {
          if (!docRef.current) return;
          setBusy(true);
          setError(null);
          elementToPdf(docRef.current)
            .then((blob) => savePdf(`Opname-${fileNamePart(project.name || (typeof stored.survey.id === 'string' ? stored.survey.id : undefined))}.pdf`, blob))
            .catch((failure: unknown) => setError(failure instanceof Error ? failure.message : String(failure)))
            .finally(() => setBusy(false));
        }}>
          <FileDown aria-hidden="true" /> {busy ? t('survey.report.pdfBusy') : t('survey.report.pdf')}</button>
      </div>
    </div>

    {error && <p className="survey-issues-note" role="alert">{error}</p>}
    <div className="survey-report-levels" role="radiogroup" aria-label={t('survey.report.level')}>
      {LEVELS.map((item) => <button key={item} type="button" role="radio" aria-checked={level === item}
        className="survey-choice survey-report-level" onClick={() => choose(item)}>
        <span className="survey-choice-title">{t(`survey.report.level.${item}`)}</span>
        <span className="survey-choice-hint">{t(`survey.report.level.${item}.hint`)}</span>
      </button>)}
    </div>

    <details className="survey-card survey-report-more">
      <summary>{t('survey.report.more')}</summary>
      <InputDossierPage />
      <DossierPage actions={actions} />
      <ExportsPage actions={actions} />
    </details>

    <article className="survey-report-doc" aria-label={t('survey.report.preview')} ref={docRef}>
      <div className="survey-report-band" aria-hidden="true"><i /><b>Open Energy Studio</b>
        <span>{t('survey.report.statusIndicative')}</span></div>
      <header>
        <h2>{t(stored.kind === 'residential' ? 'survey.report.docTitle' : 'survey.report.docTitleUtility')}</h2>
        <p className="survey-report-sub">{t('survey.report.method', { method: stored.kind === 'residential' ? 'ISSO 82.1' : 'ISSO 75.1' })} · NTA 8800:2025+C1:2026</p>
      </header>
      <dl className="survey-report-meta">
        <dt>{t('survey.report.project')}</dt><dd>{project.name || (typeof draft.id === 'string' && draft.id) || '—'}</dd>
        <dt>{t('survey.report.address')}</dt><dd>{address ?? '—'}</dd>
        <dt>{t('survey.report.building')}</dt><dd>{[stepFacts('woning', draft, t, locale)[0], t('survey.fact.year', { year })].filter(Boolean).join(', ')}</dd>
        <dt>{t('survey.surveyDate')}</dt><dd>{stored.surveyDate ? new Date(stored.surveyDate).toLocaleDateString(locale) : '—'}</dd>
        <dt>{t('survey.report.adviser')}</dt><dd>{[registration?.surveyingAdvisor?.name, registration?.surveyingAdvisor?.competenceNumber]
          .filter((part) => part && part.trim()).join(', ') || '—'}</dd>
        <dt>{t('survey.report.certificate')}</dt><dd>{registration?.certificateNumber || '—'}</dd>
        <dt>{t('survey.report.status')}</dt><dd>{t('survey.report.statusIndicative')}</dd>
      </dl>
      <section className="survey-report-result">
        {label ? <span className="survey-label survey-label--big" style={{ background: labelColor(label) }}>{label}</span>
          : <span className="survey-muted">{t('survey.result.none')}</span>}
        <dl className="survey-indicators survey-indicators--units">
          <div><dt>{t('survey.result.ep2')}</dt><dd>{n(performance?.primaryFossilIndicatorKwhPerM2Year, 1)} <small>kWh/m²</small></dd></div>
          <div><dt>{t('survey.final.beng1')}</dt><dd>{n(performance?.needIndicatorKwhPerM2Year, 1)} <small>kWh/m²</small></dd></div>
          <div><dt>{t('survey.final.beng3')}</dt><dd>{n(performance?.renewableSharePercent, 1)} <small>%</small></dd></div>
          <div><dt>TOjuli</dt><dd>{n(performance?.tojuliMaxK, 2)} <small>K</small></dd></div>
        </dl>
      </section>

      {level !== 'summary' && <>
        {steps.map((step, index) => {
          const defaults = result?.appliedDefaults.filter((item) => defaultStep(item, stored) === step.id) ?? [];
          return <section key={step.id} className="survey-report-section">
            <h3>{index + 1}. {t(step.labelKey)}</h3>
            <ul>{stepFacts(step.id as SurveyStepId, draft, t, locale, address ?? undefined).map((fact, at) => <li key={at}>{fact}</li>)}</ul>
            {defaults.length > 0 && <>
              <p className="survey-report-defaults-title">{t(defaults.length === 1 ? 'survey.check.oneDefault' : 'survey.check.stepDefaults', { count: defaults.length })}</p>
              <StepDefaults items={defaults} draft={draft} t={t} locale={locale} plain />
            </>}
          </section>;
        })}
      </>}

      {level === 'detailed' && months.length > 0 && <section className="survey-report-section">
        <h3>{t('survey.report.monthly')}</h3>
        <table className="survey-report-table">
          <thead><tr>
            <th>{t('survey.report.month')}</th>
            {services.map((service) => <th key={service}>{t(`survey.report.service.${service}`)}</th>)}
            <th>{t('survey.report.service.pv')}</th>
          </tr></thead>
          <tbody>{Array.from({ length: 12 }, (_, index) => index + 1).map((month) => <tr key={month}>
            <td>{month}</td>
            {services.map((service) => <td key={service}>{n(used(month, service), 0)}</td>)}
            <td>{n(pvMonths[month - 1], 0)}</td>
          </tr>)}</tbody>
        </table>
        <p className="survey-muted">{t('survey.report.monthlyNote')}</p>
      </section>}

      <section className="survey-report-section">
        <h3>{t('survey.report.evidence')}</h3>
        {evidence.length === 0 ? <p className="survey-muted">{t('survey.report.evidenceNone')}</p>
          : <table className="survey-report-table survey-report-table--evidence">
            <thead><tr><th>{t('survey.report.evidenceFile')}</th><th>{t('survey.report.evidenceKind')}</th>
              <th>{t('survey.report.evidencePart')}</th><th>{t('survey.report.evidenceDate')}</th><th>SHA-256</th></tr></thead>
            <tbody>{evidence.map((item) => <tr key={item.id}>
              <td>{item.fileName}{item.description ? ` — ${item.description}` : ''}</td>
              <td>{t(`evidence.kind.${item.kind}`)}</td>
              <td>{evidencePart(item.linkedPaths)}</td>
              <td>{item.date ?? '—'}</td>
              <td className="survey-report-code">{item.sha256.slice(0, 12)}…</td>
            </tr>)}</tbody>
          </table>}
      </section>

      <section className="survey-report-section">
        <h3>{t('survey.report.accountability')}</h3>
        <p>{t('survey.report.accountabilityText', { count: result?.appliedDefaults.length ?? 0 })}</p>
        {typeof draft.sourceReference === 'string' && draft.sourceReference.trim() &&
          <p>{t('survey.final.source', { source: draft.sourceReference })}</p>}
      </section>

      <section className="survey-report-section">
        <h3>{t('survey.report.software')}</h3>
        <p>{t('survey.report.softwareText', {
          version, kernel: kernel?.settled?.kernelVersion ?? '—', norm: kernel?.settled?.targetNormVersion ?? 'NTA 8800:2025+C1:2026',
        })}</p>
        {fingerprint && <p className="survey-report-code">{t('survey.report.fingerprint', { hash: fingerprint })}</p>}
      </section>
    </article>
  </div>;
}
