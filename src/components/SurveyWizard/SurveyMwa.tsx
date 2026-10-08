/**
 * Maatwerkadvies of a basisopname project (feedback 8 Oct 2026): measures are
 * changes to the survey, picked per part on cards, each with its effect; then
 * packages, use and tariffs, and the advice. The calculation is
 * src/core/mwa/surveyMeasures.ts on the kernel's building base.
 */
import { useEffect, useMemo, useRef, useState } from 'react';
import { Droplet, Flame, Home, Layers, PanelTop, Sun, Wind, LayoutGrid } from 'lucide-react';
import './SurveyWizard.css';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import type { Route } from '../../core/navigation/routes';
import type { StoredSurvey } from '../../core/nta/SurveyTemplates';
import type { MaatwerkadviesAssessment, MwaMeasure, MwaVariantResult, NtaMaatwerkadvies } from '../../core/nta/KernelClient';
import {
  CATEGORY, SURVEY_MEASURE_KINDS, assessSurveyMaatwerkadvies, patchesOverlap, surveyTemplateOf,
  type SurveyMeasureChange, type SurveyMeasureKind, type SurveyMwaRun,
} from '../../core/mwa/surveyMeasures';
import { labelColor } from '../shell/pages/results/resultsData';
import { generateMaatwerkadviesReportHTML } from '../../core/report/MaatwerkadviesReport';
import { fileNamePart, htmlToPdf, savePdf } from '../../core/report/pdf';
import { adviceText } from '../../core/nta/MwaAdviceText';
import { KernelCode } from '../KernelCode/KernelCode';

type T = (key: string, options?: Record<string, unknown>) => string;
type Json = Record<string, unknown>;
type MwaStep = 'measures' | 'packages' | 'use' | 'advice';
const STEPS: MwaStep[] = ['measures', 'packages', 'use', 'advice'];

const ICONS: Record<SurveyMeasureKind, React.ReactNode> = {
  roof: <Home />, facade: <LayoutGrid />, floor: <Layers />, windows: <PanelTop />, heating: <Flame />,
  hotWater: <Droplet />, ventilation: <Wind />, pv: <Sun />,
};
const LIFETIME: Record<SurveyMeasureKind, number> = { roof: 40, facade: 40, floor: 40, windows: 30, heating: 15, hotWater: 15, ventilation: 15, pv: 25 };
const HEATING_OPTIONS: Record<string, Json> = {
  heat_pump_outdoor_air: { kind: 'heat_pump', source: 'outdoor_air', drive: 'electric' },
  heat_pump_ground: { kind: 'heat_pump', source: 'ground', drive: 'electric' },
  boiler_hr107: { kind: 'boiler', boilerType: 'hr107', insideThermalBoundary: true },
};
const HOT_WATER_OPTIONS: Record<string, Json> = {
  heat_pump: { kind: 'heat_pump', exhaustAirSource: false },
  electric_boiler: { kind: 'electric_boiler' },
  gas_combi: { kind: 'gas_appliance', applianceType: 'combi', gaskeur: 'gaskeur_hr_cw' },
};
const GLASS = ['triple_hr', 'hr_plus_plus', 'hr_plus'];
const RECOVERY = ['counter_flow_unknown_material', 'cross_flow', 'enthalpy'];
const DEFAULT_TARIFFS = { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, electricityExportEurPerKwh: 0.05, sourceReference: 'Starttarieven Open Energy Studio, nog te controleren' };

function emptyDefinition(): NtaMaatwerkadvies {
  return { measures: [], packages: [], tariffs: { ...DEFAULT_TARIFFS } };
}

/** A sensible first change per part, from the survey as it is. */
function defaultChange(kind: SurveyMeasureKind, survey: Json): SurveyMeasureChange {
  const surfaces = (((survey.envelope as Json | undefined)?.surfaces) as Json[] | undefined) ?? [];
  const thickest = (element: string) => Math.max(0, ...surfaces.filter((surface) => surface.element === element)
    .map((surface) => Number((surface.insulation as Json | undefined)?.thicknessMm) || 0));
  const pv = ((survey.pv as Json[] | undefined) ?? [])[0];
  const pvArea = ((survey.pv as Json[] | undefined) ?? []).reduce((sum, system) => sum + (Number(system.panelAreaM2) || 0), 0);
  switch (kind) {
    case 'roof': return { kind, surfaceIds: [], thicknessMm: Math.max(200, thickest('roof') + 60) };
    case 'facade': return { kind, surfaceIds: [], thicknessMm: Math.max(120, thickest('facade') + 60) };
    case 'floor': return { kind, surfaceIds: [], thicknessMm: Math.max(140, thickest('floor') + 60) };
    case 'windows': return { kind, surfaceIds: [], glass: 'triple_hr' };
    case 'heating': return { kind, generator: HEATING_OPTIONS.heat_pump_outdoor_air };
    case 'hotWater': return { kind, generator: HOT_WATER_OPTIONS.heat_pump };
    case 'ventilation': return { kind, principle: 'balanced', heatRecovery: 'counter_flow_unknown_material' };
    case 'pv': return {
      kind, panelAreaM2: Math.round((pvArea > 0 ? pvArea * 2 : 12) * 10) / 10,
      azimuthDeg: Number(pv?.azimuthDeg ?? 180), tiltDeg: Number(pv?.tiltDeg ?? 35), moduleType: String(pv?.moduleType ?? 'monocrystalline'),
    };
  }
}

const money = (value: number | null | undefined, locale: string) => value == null ? '—' : `€ ${formatNumber(value, locale, 0)}`;

function Label({ value }: { value: string | null | undefined }) {
  return value ? <span className="survey-label" style={{ background: labelColor(value) }}>{value}</span> : <span>—</span>;
}

export function SurveyMwa({ route, navigate }: { route: Route; navigate: (route: Route) => void }) {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const project = state.project;
  const stored = project.basisopname as StoredSurvey | undefined;
  const definition = project.maatwerkadvies ?? emptyDefinition();
  const step: MwaStep = STEPS.includes(route.sub as MwaStep) ? route.sub as MwaStep : route.sub === 'passport' ? 'advice' : 'measures';
  const save = (next: NtaMaatwerkadvies) => dispatch({ type: 'SET_MAATWERKADVIES', payload: next });
  const [run, setRun] = useState<SurveyMwaRun | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const request = useRef(0);
  const key = useMemo(() => JSON.stringify([stored?.survey, definition]), [stored?.survey, definition]);

  // Recalculate after a change, once the typing pauses; the newest run wins.
  useEffect(() => {
    if (!stored) return undefined;
    const current = ++request.current;
    const timer = window.setTimeout(() => {
      setBusy(true);
      assessSurveyMaatwerkadvies(stored, definition)
        .then((result) => { if (request.current === current) { setRun(result); setError(null); } })
        .catch((failure: unknown) => { if (request.current === current) setError(failure instanceof Error ? failure.message : String(failure)); })
        .finally(() => { if (request.current === current) setBusy(false); });
    }, 600);
    return () => window.clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);

  if (!stored) return null;
  const survey = stored.survey as Json;
  const assessment = run?.assessment ?? null;
  const resultOf = (id: string, kind: 'measure' | 'package') =>
    (kind === 'measure' ? assessment?.measures : assessment?.packages)?.find((variant) => variant.id === id) ?? null;
  const go = (next: MwaStep) => navigate({ step: 'advice', sub: next });
  const index = STEPS.indexOf(step);

  return <div className="survey-wizard">
    <div className="survey-main">
      <nav className="mwa-steps" aria-label={t('mwaSurvey.steps')}>
        {STEPS.map((item, at) => <button key={item} type="button" className={`mwa-step${item === step ? ' mwa-step--on' : ''}`}
          aria-current={item === step ? 'step' : undefined} onClick={() => go(item)}>{at + 1} {t(`mwaSurvey.step.${item}`)}</button>)}
      </nav>
      <p className="survey-crumb">{t('mwaSurvey.crumb', { number: index + 1, total: STEPS.length })}</p>
      <h1 id="page-title" tabIndex={-1}>{t(`mwaSurvey.q.${step}`)}</h1>
      <p className="survey-lead">{t(`mwaSurvey.q.${step}.help`)}</p>
      {error && <p className="survey-issues-note" role="alert">{t('mwaSurvey.error', { error })}</p>}
      {assessment && assessment.issues.length > 0 && <section className="survey-card" role="alert">
        <span className="survey-overline">{t('mwaSurvey.issues')}</span>
        <ul className="mwa-issues">{assessment.issues.map((issue, at) => {
          // Known fields in words, with the step to fix them; the rest as the kernel says it.
          const measure = /^measures[(d+)]/.exec(issue.path);
          const name = measure ? definition.measures[Number(measure[1])]?.name : undefined;
          const known = issue.path === 'tariffs.sourceReference' ? { text: t('mwaSurvey.issue.tariffsSource'), step: 'use' as MwaStep }
            : /^(currentUse|futureUse)./.test(issue.path) ? { text: t('mwaSurvey.issue.use'), step: 'use' as MwaStep }
              : measure && issue.path.endsWith('.costSource') ? { text: t('mwaSurvey.issue.costSource', { name: name ?? '' }), step: 'measures' as MwaStep }
                : measure ? { text: t('mwaSurvey.issue.measure', { name: name ?? '' }), step: 'measures' as MwaStep } : null;
          return <li key={at}>
            {known ? <span>{known.text}</span> : <KernelCode code={issue.code} prefixes={['mwa.issue.', 'kernel.issue.', 'nta.gap.']} hideCode />}
            {known && known.step !== step && <button type="button" className="btn btn-sm" onClick={() => go(known.step)}>{t('survey.check.goTo', { step: t(`mwaSurvey.step.${known.step}`) })}</button>}
          </li>;
        })}</ul>
      </section>}

      {step === 'measures' && <Measures definition={definition} survey={survey} save={save} resultOf={(id) => resultOf(id, 'measure')}
        current={assessment?.current ?? null} t={t} locale={locale} />}
      {step === 'packages' && <Packages definition={definition} save={save} run={run} resultOf={(id) => resultOf(id, 'package')} t={t} locale={locale} />}
      {step === 'use' && <Use definition={definition} save={save} assessment={assessment} t={t} locale={locale} />}
      {step === 'advice' && <Advice definition={definition} save={save} assessment={assessment} t={t} locale={locale}
        onPdf={() => assessment && htmlToPdf(generateMaatwerkadviesReportHTML(project, definition, assessment), 'BRL 9500-MWA · ISSO 82.2')
          .then((blob) => savePdf(`Maatwerkadvies-${fileNamePart(project.name)}.pdf`, blob))
          .catch((failure: unknown) => setError(failure instanceof Error ? failure.message : String(failure)))} />}

      <div className="survey-actions">
        <button type="button" className="btn" disabled={index === 0} onClick={() => go(STEPS[index - 1])}>{t('survey.nav.previous')}</button>
        {index < STEPS.length - 1 && <button type="button" className="btn btn-primary" onClick={() => go(STEPS[index + 1])}>
          {t('survey.nav.next', { step: t(`mwaSurvey.step.${STEPS[index + 1]}`).toLowerCase() })}</button>}
      </div>
    </div>

    <aside className="survey-aside" aria-label={t('mwaSurvey.aside')}>
      <div className="survey-card" aria-live="polite">
        <span className="survey-overline">{t('mwaSurvey.labelPath')}</span>
        {busy && !assessment && <p className="survey-muted">{t('survey.result.busy')}</p>}
        {assessment?.current && <div className="mwa-path-row"><Label value={assessment.current.label.labelClass} />
          <span>{t('mwaSurvey.now')} · {formatNumber(assessment.current.label.primaryFossilIndicatorKwhPerM2, locale, 1)}</span></div>}
        {(assessment?.packages ?? []).map((pack) => <div key={pack.id} className="mwa-path-row"><Label value={pack.label.labelClass} />
          <span>{pack.name} · {formatNumber(pack.label.primaryFossilIndicatorKwhPerM2, locale, 1)}</span></div>)}
        <p className="survey-muted">{t('mwaSurvey.ep2Note')}</p>
      </div>
      <div className="survey-card">
        <span className="survey-overline">{t('survey.open.title')}</span>
        <ul className="mwa-todo">
          {definition.measures.length === 0 && <li>{t('mwaSurvey.todo.measures')}</li>}
          {definition.packages.length < 2 && <li>{t('mwaSurvey.todo.packages')}</li>}
          {definition.measures.some((measure) => !measure.investmentEur) && <li>{t('mwaSurvey.todo.investment')}</li>}
          {!definition.tariffs.sourceReference.trim() && <li>{t('mwaSurvey.todo.tariffs')}</li>}
          {definition.measures.length > 0 && definition.packages.length >= 2 && definition.measures.every((measure) => measure.investmentEur)
            && definition.tariffs.sourceReference.trim() && <li className="survey-done">✓ {t('survey.open.none')}</li>}
        </ul>
      </div>
    </aside>
  </div>;
}

function Effect({ result, current, t, locale }: { result: MwaVariantResult | null; current: MwaVariantResult | null; t: T; locale: string }) {
  if (!result || !current) return <span className="mwa-effect mwa-effect--none">{t(current ? 'mwaSurvey.effect.pending' : 'mwaSurvey.effect.blocked')}</span>;
  return <span className="mwa-effect">
    EP2 {formatNumber(current.label.primaryFossilIndicatorKwhPerM2, locale, 1)} → {formatNumber(result.label.primaryFossilIndicatorKwhPerM2, locale, 1)}
    {result.label.labelClass !== current.label.labelClass && ` · ${current.label.labelClass} → ${result.label.labelClass}`}
    {result.savings && ` · ${t('mwaSurvey.effect.saving', { amount: money(result.savings.energyCostEur, locale) })}`}
  </span>;
}

function NumberInput({ label, value, onChange, step = 'any' }: { label: string; value: number | undefined; onChange: (value: number) => void; step?: string }) {
  return <label>{label}<input type="number" step={step} value={Number.isFinite(value) ? value : ''} onChange={(event) => onChange(Number(event.target.value))} /></label>;
}

function Measures({ definition, survey, save, resultOf, current, t, locale }: {
  definition: NtaMaatwerkadvies; survey: Json; save: (next: NtaMaatwerkadvies) => void;
  resultOf: (id: string) => MwaVariantResult | null; current: MwaVariantResult | null; t: T; locale: string;
}) {
  const surfaces = (((survey.envelope as Json | undefined)?.surfaces) as Json[] | undefined) ?? [];
  const add = (kind: SurveyMeasureKind) => {
    const id = `m${Date.now().toString(36)}`;
    const measure: MwaMeasure = {
      id, name: t(`mwaSurvey.kind.${kind}.name`), category: CATEGORY[kind], target: 'building', patch: [],
      investmentEur: 0, costSource: t('mwaSurvey.costSourceDefault'), lifetimeYears: LIFETIME[kind],
      template: { source: 'survey', change: defaultChange(kind, survey) } as unknown as MwaMeasure['template'],
    };
    save({ ...definition, measures: [...definition.measures, measure] });
  };
  const update = (id: string, patch: Partial<MwaMeasure>) =>
    save({ ...definition, measures: definition.measures.map((measure) => measure.id === id ? { ...measure, ...patch } : measure) });
  const setChange = (measure: MwaMeasure, change: SurveyMeasureChange) =>
    update(measure.id, { template: { source: 'survey', change } as unknown as MwaMeasure['template'] });
  const remove = (id: string) => save({
    ...definition, measures: definition.measures.filter((measure) => measure.id !== id),
    packages: definition.packages.map((pack) => ({ ...pack, measureIds: pack.measureIds.filter((item) => item !== id) })),
  });
  return <>
    <div className="survey-choices mwa-kinds" role="group" aria-label={t('mwaSurvey.addMeasure')}>
      {SURVEY_MEASURE_KINDS.map((kind) => <button key={kind} type="button" className="survey-choice mwa-kind" onClick={() => add(kind)}>
        <span className="survey-choice-icon" aria-hidden="true">{ICONS[kind]}</span>
        <span className="survey-choice-title">{t(`mwaSurvey.kind.${kind}`)}</span>
      </button>)}
    </div>
    {definition.measures.length === 0 && <p className="survey-muted">{t('mwaSurvey.noMeasures')}</p>}
    {definition.measures.map((measure) => {
      const template = surveyTemplateOf(measure);
      const change = template?.change;
      return <section key={measure.id} className="survey-card mwa-measure" aria-label={measure.name}>
        <div className="mwa-measure-head">
          <input className="mwa-name" type="text" value={measure.name} aria-label={t('mwaSurvey.name')}
            onChange={(event) => update(measure.id, { name: event.target.value })} />
          <Effect result={resultOf(measure.id)} current={current} t={t} locale={locale} />
        </div>
        <div className="nta-form"><div className="nta-form-grid">
          {change && (change.kind === 'roof' || change.kind === 'facade' || change.kind === 'floor') && <>
            <NumberInput label={t('mwaSurvey.thickness')} value={change.thicknessMm} step="10" onChange={(value) => setChange(measure, { ...change, thicknessMm: value })} />
            <label>{t('mwaSurvey.surfaces')}
              <select value={change.surfaceIds.length === 0 ? '' : change.surfaceIds[0]}
                onChange={(event) => setChange(measure, { ...change, surfaceIds: event.target.value ? [event.target.value] : [] })}>
                <option value="">{t('mwaSurvey.allSurfaces')}</option>
                {surfaces.filter((surface) => surface.element === (change.kind === 'roof' ? 'roof' : change.kind === 'facade' ? 'facade' : 'floor'))
                  .map((surface) => <option key={String(surface.id)} value={String(surface.id)}>{String(surface.id)}</option>)}
              </select>
            </label>
          </>}
          {change?.kind === 'windows' && <label>{t('opname.window.glass')}
            <select value={change.glass} onChange={(event) => setChange(measure, { ...change, glass: event.target.value })}>
              {GLASS.map((glass) => <option key={glass} value={glass}>{t(`opname.window.glassKind.${glass}`)}</option>)}
            </select></label>}
          {change?.kind === 'heating' && <label>{t('mwaSurvey.newGenerator')}
            <select value={Object.keys(HEATING_OPTIONS).find((key) => JSON.stringify(HEATING_OPTIONS[key]) === JSON.stringify(change.generator)) ?? ''}
              onChange={(event) => setChange(measure, { ...change, generator: HEATING_OPTIONS[event.target.value] })}>
              {Object.keys(HEATING_OPTIONS).map((key) => <option key={key} value={key}>{t(`mwaSurvey.heating.${key}`)}</option>)}
            </select></label>}
          {change?.kind === 'hotWater' && <label>{t('mwaSurvey.newGenerator')}
            <select value={Object.keys(HOT_WATER_OPTIONS).find((key) => JSON.stringify(HOT_WATER_OPTIONS[key]) === JSON.stringify(change.generator)) ?? ''}
              onChange={(event) => setChange(measure, { ...change, generator: HOT_WATER_OPTIONS[event.target.value] })}>
              {Object.keys(HOT_WATER_OPTIONS).map((key) => <option key={key} value={key}>{t(`mwaSurvey.hotWater.${key}`)}</option>)}
            </select></label>}
          {change?.kind === 'ventilation' && <label>{t('survey.ventilation.heatRecovery')}
            <select value={change.heatRecovery ?? ''} onChange={(event) => setChange(measure, { ...change, heatRecovery: event.target.value || null })}>
              <option value="">{t('mwaSurvey.noRecovery')}</option>
              {RECOVERY.map((key) => <option key={key} value={key}>{t(`opname.ventilation.heatRecoveryKind.${key}`)}</option>)}
            </select></label>}
          {change?.kind === 'pv' && <>
            <NumberInput label={t('mwaSurvey.pvArea')} value={change.panelAreaM2} onChange={(value) => setChange(measure, { ...change, panelAreaM2: value })} />
            <NumberInput label={t('opname.pv.azimuth')} value={change.azimuthDeg} step="1" onChange={(value) => setChange(measure, { ...change, azimuthDeg: value })} />
            <NumberInput label={t('opname.tilt')} value={change.tiltDeg} step="1" onChange={(value) => setChange(measure, { ...change, tiltDeg: value })} />
          </>}
          <NumberInput label={t('mwaSurvey.investment')} value={measure.investmentEur} step="50" onChange={(value) => update(measure.id, { investmentEur: value })} />
          <NumberInput label={t('mwaSurvey.lifetime')} value={measure.lifetimeYears} step="1" onChange={(value) => update(measure.id, { lifetimeYears: value })} />
          <NumberInput label={t('mwaSurvey.maintenance')} value={measure.maintenanceEurPerYear ?? 0} step="10" onChange={(value) => update(measure.id, { maintenanceEurPerYear: value })} />
          <label>{t('mwaSurvey.costSource')}<input type="text" value={measure.costSource} onChange={(event) => update(measure.id, { costSource: event.target.value })} /></label>
        </div></div>
        <div className="mwa-measure-actions"><button type="button" className="btn btn-sm" onClick={() => remove(measure.id)}>{t('opname.remove')}</button></div>
      </section>;
    })}
  </>;
}

function Packages({ definition, save, run, resultOf, t, locale }: {
  definition: NtaMaatwerkadvies; save: (next: NtaMaatwerkadvies) => void; run: SurveyMwaRun | null;
  resultOf: (id: string) => MwaVariantResult | null; t: T; locale: string;
}) {
  const add = () => save({ ...definition, packages: [...definition.packages, {
    id: `p${Date.now().toString(36)}`, name: t('mwaSurvey.packageName', { number: definition.packages.length + 1 }), measureIds: [],
  }] });
  const update = (id: string, patch: Partial<NtaMaatwerkadvies['packages'][number]>) =>
    save({ ...definition, packages: definition.packages.map((pack) => pack.id === id ? { ...pack, ...patch } : pack) });
  return <>
    {definition.measures.length === 0 && <p className="survey-muted">{t('mwaSurvey.todo.measures')}</p>}
    {definition.packages.map((pack) => {
      const result = resultOf(pack.id);
      const overlap = run && pack.measureIds.some((first, at) => pack.measureIds.slice(at + 1)
        .some((second) => patchesOverlap(run.patches[first] ?? [], run.patches[second] ?? [])));
      return <section key={pack.id} className="survey-card mwa-measure mwa-package" aria-label={pack.name}>
        <div className="mwa-measure-head">
          <input className="mwa-name" type="text" value={pack.name} aria-label={t('mwaSurvey.name')} onChange={(event) => update(pack.id, { name: event.target.value })} />
          {result && <Label value={result.label.labelClass} />}
        </div>
        <p className="nta-form-subhead">{t('mwaSurvey.inPackage')}</p>
        {pack.measureIds.length === 0 && <p className="survey-muted">{t('mwaSurvey.emptyPackage')}</p>}
        <div className="mwa-checks">{definition.measures.map((measure) => <label key={measure.id} className="mwa-check">
          <input type="checkbox" checked={pack.measureIds.includes(measure.id)} onChange={(event) => update(pack.id, {
            measureIds: event.target.checked ? [...pack.measureIds, measure.id] : pack.measureIds.filter((item) => item !== measure.id),
          })} />{measure.name}</label>)}</div>
        {overlap && <p className="survey-issues-note">{t('mwaSurvey.overlap')}</p>}
        {result && <dl className="survey-indicators survey-indicators--units">
          <div><dt>EP2</dt><dd>{formatNumber(result.label.primaryFossilIndicatorKwhPerM2, locale, 1)} <small>kWh/m²</small></dd></div>
          <div><dt>{t('mwaSurvey.investment')}</dt><dd>{money(result.investmentEur, locale)}</dd></div>
          <div><dt>{t('mwaSurvey.savingPerYear')}</dt><dd>{money(result.savings?.energyCostEur, locale)}</dd></div>
          <div><dt>{t('mwaSurvey.payback')}</dt><dd>{result.simplePaybackYears == null ? '—' : `${formatNumber(result.simplePaybackYears, locale, 1)} ${t('mwaSurvey.years')}`}</dd></div>
        </dl>}
        <div className="mwa-measure-actions">
          {result && <span className="survey-muted">{t('mwaSurvey.npv')}: {money(result.netPresentValueEur, locale)}</span>}
          <button type="button" className="btn btn-sm" onClick={() => save({ ...definition, packages: definition.packages.filter((item) => item.id !== pack.id) })}>{t('opname.remove')}</button>
        </div>
      </section>;
    })}
    <div><button type="button" className="btn" onClick={add}>+ {t('mwaSurvey.addPackage')}</button></div>
    <p className="survey-muted">{t('mwaSurvey.packagesNote')}</p>
  </>;
}

function Use({ definition, save, assessment, t, locale }: {
  definition: NtaMaatwerkadvies; save: (next: NtaMaatwerkadvies) => void; assessment: MaatwerkadviesAssessment | null; t: T; locale: string;
}) {
  const tariffs = definition.tariffs;
  const economics = definition.economics ?? {};
  const measured = definition.measured ?? {};
  const setTariffs = (patch: Partial<typeof tariffs>) => save({ ...definition, tariffs: { ...tariffs, ...patch } });
  const fit = assessment?.fitCheck;
  return <div className="nta-form">
    <div className="nta-form-grid">
      <p className="nta-form-subhead">{t('mwaSurvey.use.profile')}</p>
      <label>{t('mwaSurvey.use.profileLabel')}
        <select value={definition.currentUse?.profile ?? 'nta'} onChange={(event) => save({ ...definition,
          currentUse: event.target.value === 'nta' ? undefined : { profile: event.target.value as NonNullable<NtaMaatwerkadvies['currentUse']>['profile'], sourceReference: 'ISSO 82.2 §2.5.2' } })}>
          {['nta', 'energy_conscious', 'average', 'not_energy_conscious'].map((key) => <option key={key} value={key}>{t(`mwaSurvey.profile.${key}`)}</option>)}
        </select></label>
      <p className="nta-form-subhead">{t('mwaSurvey.use.tariffs')}</p>
      <NumberInput label={t('mwaSurvey.tariff.gas')} value={tariffs.gasEurPerM3} step="0.01" onChange={(value) => setTariffs({ gasEurPerM3: value })} />
      <NumberInput label={t('mwaSurvey.tariff.electricity')} value={tariffs.electricityEurPerKwh} step="0.01" onChange={(value) => setTariffs({ electricityEurPerKwh: value })} />
      <NumberInput label={t('mwaSurvey.tariff.export')} value={tariffs.electricityExportEurPerKwh ?? 0} step="0.01" onChange={(value) => setTariffs({ electricityExportEurPerKwh: value })} />
      <NumberInput label={t('mwaSurvey.tariff.gasFixed')} value={tariffs.gasFixedEurPerYear ?? 0} step="1" onChange={(value) => setTariffs({ gasFixedEurPerYear: value })} />
      <label className="survey-address-wide">{t('mwaSurvey.tariff.source')}
        <input type="text" value={tariffs.sourceReference} onChange={(event) => setTariffs({ sourceReference: event.target.value })} /></label>
      <p className="nta-form-subhead">{t('mwaSurvey.use.economics')}</p>
      <NumberInput label={t('mwaSurvey.economics.discount')} value={(economics.discountRate ?? 0.03) * 100} step="0.1"
        onChange={(value) => save({ ...definition, economics: { ...economics, discountRate: value / 100 } })} />
      <NumberInput label={t('mwaSurvey.economics.priceChange')} value={(economics.energyPriceChange ?? 0) * 100} step="0.1"
        onChange={(value) => save({ ...definition, economics: { ...economics, energyPriceChange: value / 100 } })} />
      <p className="nta-form-subhead">{t('mwaSurvey.use.measured')}</p>
      <NumberInput label={t('mwaSurvey.measured.gas')} value={measured.annualGasM3} step="1"
        onChange={(value) => save({ ...definition, measured: { ...measured, annualGasM3: value || undefined } })} />
      <NumberInput label={t('mwaSurvey.measured.electricity')} value={measured.annualElectricityKwh} step="1"
        onChange={(value) => save({ ...definition, measured: { ...measured, annualElectricityKwh: value || undefined } })} />
    </div>
    {fit && <p className="survey-muted">{t('mwaSurvey.fit', {
      gas: fit.gasDeviationPercent == null ? '—' : `${formatNumber(fit.gasDeviationPercent, locale, 0)} %`,
      electricity: fit.electricityDeviationPercent == null ? '—' : `${formatNumber(fit.electricityDeviationPercent, locale, 0)} %`,
    })}</p>}
  </div>;
}

function Advice({ definition, save, assessment, onPdf, t, locale }: {
  definition: NtaMaatwerkadvies; save: (next: NtaMaatwerkadvies) => void; assessment: MaatwerkadviesAssessment | null;
  onPdf: () => void; t: T; locale: string;
}) {
  const chosen = assessment?.advice?.packageId ?? null;
  return <>
    <div className="survey-choices mwa-advice" role="radiogroup" aria-label={t('mwaSurvey.choose')}>
      <button type="button" role="radio" aria-checked={!definition.advisedPackageId} className="survey-choice"
        onClick={() => save({ ...definition, advisedPackageId: undefined })}>
        <span className="survey-choice-title">{t('mwaSurvey.automatic')}</span>
        <span className="survey-choice-hint">{t('mwaSurvey.automaticHint')}</span>
      </button>
      {(assessment?.packages ?? []).map((pack) => <button key={pack.id} type="button" role="radio" className="survey-choice"
        aria-checked={definition.advisedPackageId === pack.id} onClick={() => save({ ...definition, advisedPackageId: pack.id })}>
        <span className="survey-choice-title">{pack.name} <Label value={pack.label.labelClass} /></span>
        <span className="survey-choice-hint">{t('mwaSurvey.packageSummary', {
          saving: money(pack.savings?.energyCostEur, locale), investment: money(pack.investmentEur, locale),
          payback: pack.simplePaybackYears == null ? '—' : formatNumber(pack.simplePaybackYears, locale, 1),
        })}</span>
      </button>)}
    </div>
    {chosen && <p className="survey-muted">{t('mwaSurvey.chosen', { name: assessment?.packages.find((pack) => pack.id === chosen)?.name ?? chosen })}</p>}
    <div className="nta-form"><label>{t('mwaSurvey.motivation')}
      <textarea rows={3} value={definition.adviceMotivation ?? ''} onChange={(event) => save({ ...definition, adviceMotivation: event.target.value || undefined })} />
    </label></div>
    {(assessment?.advice?.warnings.length ?? 0) > 0 && <section className="survey-card">
      <span className="survey-overline">{t('mwaSurvey.warnings')}</span>
      <ul className="mwa-todo">{assessment!.advice!.warnings.map((warning) => <li key={warning}>{adviceText(warning, locale)}</li>)}</ul>
    </section>}
    <div className="survey-next-actions">
      <button type="button" className="btn btn-primary" disabled={!assessment} onClick={onPdf}>{t('mwaSurvey.pdf')}</button>
    </div>
  </>;
}
