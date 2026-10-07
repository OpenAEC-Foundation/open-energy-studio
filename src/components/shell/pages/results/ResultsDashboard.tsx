/**
 * Results dashboard of the workflow step "Resultaten" (UI redesign F7, mockups 05/05b).
 *
 * Reads the one kernel run of the document (KernelProvider). When the kernel
 * did not calculate the project (input refused, or no NTA input yet) the page
 * falls back to the ResultsView, which applies the shared KernelVerdict rule
 * (withheld results, indicative estimate); the dashboard itself only ever shows
 * kernel figures.
 */
import { useEffect, useMemo, useState } from 'react';
import { CheckCircle2, Info, XCircle } from 'lucide-react';
import { useI18n } from '../../../../i18n/i18n';
import { formatNumber } from '../../../../i18n/format';
import { useEnergy } from '../../../../context/EnergyContext';
import { useKernel } from '../../../../context/KernelProvider';
import { routeForPath } from '../../../../core/nta/gapRoutes';
import { kernelIssues } from '../../../../core/nta/stepStatus';
import { summarizeServiceEnergy } from '../../../../core/nta/ServiceEnergy';
import {
  fetchKernelInterpretations, legacyEdition, type NtaInterpretationGroup, type ProjectPerformanceAssessment,
} from '../../../../core/nta/KernelClient';
import type { Route } from '../../../../core/navigation/routes';
import { ResultsView } from '../../../ResultsView/ResultsView';
import {
  Banner, Card, DataTable, ErrorState, IssueList, Segmented, Skeleton, StaleBanner, StatusPill, type Column,
} from '../../../ui';
import { routeLabel } from '../../PageHeader';
import { MonthlyChart } from './MonthlyChart';
import {
  energySeries, gauges, isResidential, keyFigures, LABEL_CLASSES, labelColor, labelTextColor, monthlyNeed, monthlyPv,
  zoneDemands, type ChartMode, type Gauge,
} from './resultsData';
import './results.css';

export type ResultsSub = 'overview' | 'services' | 'zones' | 'monthly' | 'provenance';

export function ResultsDashboard({ sub, onNavigate }: { sub: string | undefined; onNavigate: (route: Route) => void }) {
  const { t } = useI18n();
  const kernel = useKernel();

  if (!kernel) return <ResultsView workflowPanels={false} />;
  const assessment = kernel.settled;
  if (!assessment) {
    if (kernel.phase === 'error') {
      return <ErrorState title={t('results.dash.errorTitle')}>{kernel.error ?? t('results.dash.errorText')}</ErrorState>;
    }
    return <div className="results-dash results-dash--loading" role="status" aria-live="polite">
      <p>{t('results.dash.loading')}</p>
      <div className="results-gauges">{[0, 1, 2, 3].map((index) => <Skeleton key={index} height={132} />)}</div>
    </div>;
  }
  if (kernel.verdict !== 'calculated' || !assessment.performance) return <ResultsView workflowPanels={false} />;

  const stale = kernel.phase === 'stale';
  return (
    <div className={stale ? 'results-dash results-dash--stale' : 'results-dash'} data-testid="results-dashboard">
      {stale && <StaleBanner>{t('results.dash.stale')}</StaleBanner>}
      <Banner tone="unv" title={t('results.dash.unverifiedTitle')}>{t('results.dash.unverifiedText')}</Banner>
      {legacyEdition(assessment) && <Banner tone="warn" role="note" title={t('nta.edition.legacyTitle')}>
        {t('nta.edition.legacyBody', { edition: assessment.targetNormVersion })}</Banner>}
      <div className="results-dash__body" aria-busy={stale}>
        {(sub ?? 'overview') === 'overview' && <Overview assessment={assessment} onNavigate={onNavigate} />}
        {sub === 'services' && <ServicesTab assessment={assessment} />}
        {sub === 'zones' && <ZonesTab assessment={assessment} />}
        {sub === 'monthly' && <MonthlyTab assessment={assessment} />}
        {sub === 'provenance' && <ProvenanceTab assessment={assessment} />}
      </div>
    </div>
  );
}

// ── Overview ─────────────────────────────────────────────────────────

function Overview({ assessment, onNavigate }: { assessment: ProjectPerformanceAssessment; onNavigate: (route: Route) => void }) {
  const { t, locale } = useI18n();
  const performance = assessment.performance!;
  const [mode, setMode] = useState<ChartMode>('delivered');
  const series = useMemo(() => energySeries(performance, mode), [performance, mode]);
  const need = useMemo(() => monthlyNeed(performance), [performance]);
  const figures = keyFigures(assessment);
  const items = gauges(assessment);
  const residential = isResidential(assessment);

  return <>
    <div className="results-gauges">
      <LabelCard assessment={assessment} />
      {items.map((gauge) => <GaugeCard key={gauge.key} gauge={gauge} />)}
      {!residential && <Card className="results-gauge results-gauge--na" title={<>TO<sub>juli</sub></>}>
        <p className="results-gauge__na">{t('results.dash.tojuliNotApplicable')}</p>
      </Card>}
    </div>

    <div className="results-grid">
      <Card className="results-grid__main" title={t('results.dash.energyTitle')} subtitle={t(`results.dash.mode.${mode}.unit`)}
        actions={<Segmented<ChartMode> size="sm" value={mode} onChange={setMode} aria-label={t('results.dash.modeLabel')}
          options={[
            { value: 'delivered', label: t('results.dash.mode.delivered') },
            { value: 'primary', label: t('results.dash.mode.primary') },
            { value: 'carrier', label: t('results.dash.mode.carrier') },
          ]} />}>
        <MonthlyChart series={series} unit="kWh" testId="results-energy-chart"
          label={t(`results.dash.mode.${mode}.chart`)}
          summary={t('results.dash.chartSummary', {
            total: formatNumber(series.reduce((sum, item) => sum + item.values.reduce((a, b) => a + b, 0), 0), locale, 0),
          })} />
      </Card>
      <div className="results-grid__side">
        <Card title={t('results.dash.needTitle')} subtitle={t('results.dash.needSubtitle')}>
          <MonthlyChart layout="grouped" height={200} unit="kWh" testId="results-need-chart" label={t('results.dash.needTitle')}
            series={[
              { key: 'heat', labelKey: 'results.series.heatNeed', color: 'var(--viz-heating)', values: need.heat },
              { key: 'cold', labelKey: 'results.series.coldNeed', color: 'var(--viz-cooling)', values: need.cold },
            ]} />
        </Card>
        <AttentionCard assessment={assessment} items={items} onNavigate={onNavigate} />
      </div>
    </div>

    <div className="results-figures" role="list" aria-label={t('results.dash.figures')}>
      <Figure label={t('results.dash.figure.primaryFossil')} value={formatNumber(figures.primaryFossilKwh, locale, 0)} unit="kWh/jr" />
      <Figure label={t('results.dash.figure.renewable')} value={formatNumber(figures.renewableKwh, locale, 0)} unit="kWh/jr" />
      <Figure label={t('results.dash.figure.final')} value={formatNumber(figures.finalEnergyKwh, locale, 0)} unit="kWh/jr" />
      <Figure label={t('results.dash.figure.co2')} value={formatNumber(figures.co2KgPerM2, locale, 1)} unit={t('unit.kgPerM2Year')} />
      <Figure label={t('results.dash.figure.zeb')} value={formatNumber(figures.zebKwhPerM2, locale, 2)} unit={t('unit.kwhPerM2Year')} />
    </div>
  </>;
}

function Figure({ label, value, unit }: { label: string; value: string; unit: string }) {
  return <div className="results-figure" role="listitem">
    <span className="results-figure__label">{label}</span>
    <span className="results-figure__value">{value} <small>{unit}</small></span>
  </div>;
}

function LabelCard({ assessment }: { assessment: ProjectPerformanceAssessment }) {
  const { t, locale } = useI18n();
  const performance = assessment.performance!;
  const labelClass = performance.indicativeLabelClass;
  const ep2 = performance.labelPrimaryFossilIndicatorKwhPerM2Year ?? performance.primaryFossilIndicatorKwhPerM2Year;
  return <Card className="results-gauge results-label" title={t('results.dash.label')}
    actions={<StatusPill tone="unv">{t('results.dash.indicative')}</StatusPill>}>
    <div className="results-label__row">
      <span className="results-label__badge"
        style={labelClass ? { background: labelColor(labelClass), color: labelTextColor(labelColor(labelClass)) } : undefined}
        data-testid="results-label-class">{labelClass ?? '–'}</span>
      <span className="results-label__ep">
        <b>{formatNumber(ep2, locale, 2)}</b> {t('unit.kwhPerM2Year')}
        <small>EP<sub>2</sub> {t('results.dash.labelBasis')}</small>
      </span>
    </div>
    <ol className="results-label__scale" aria-label={t('results.dash.labelScale')}>
      {LABEL_CLASSES.map((item) => <li key={item} style={{ background: labelColor(item) }}
        aria-current={item === labelClass ? 'true' : undefined} title={item}>
        <span className="visually-hidden">{item}</span>
      </li>)}
    </ol>
    <div className="results-label__ends" aria-hidden="true"><span>A++++</span><span>G</span></div>
  </Card>;
}

const GAUGE_TITLE: Record<Gauge['key'], string> = {
  beng1: 'results.beng1.title', beng2: 'results.beng2.title', beng3: 'results.beng3.title', tojuli: 'results.toJuli',
};

export function GaugeCard({ gauge }: { gauge: Gauge }) {
  const { t, locale } = useI18n();
  const unit = gauge.key === 'beng3' ? '%' : gauge.key === 'tojuli' ? 'K' : t('unit.kwhPerM2Year');
  const scaleMax = Math.max(gauge.value ?? 0, gauge.limit ?? 0) * 1.35 || 1;
  const fill = gauge.value == null ? 0 : Math.min(100, Math.max(0, (gauge.value / scaleMax) * 100));
  const marker = gauge.limit == null ? null : Math.min(100, (gauge.limit / scaleMax) * 100);
  const tone = gauge.meets == null ? 'neutral' : gauge.meets ? 'ok' : 'err';
  const verdict = gauge.meets == null ? null
    : <StatusPill tone={gauge.meets ? 'ok' : 'err'}>{t(gauge.meets ? 'results.dash.meets' : 'results.dash.fails')}</StatusPill>;
  const marginText = gauge.margin == null ? null : gauge.key === 'beng3'
    ? t(gauge.margin >= 0 ? 'results.dash.marginAbovePct' : 'results.dash.marginBelowPct', { value: formatNumber(Math.abs(gauge.margin), locale, 1) })
    : t(gauge.margin >= 0 ? 'results.dash.marginBelow' : 'results.dash.marginAbove', { value: formatNumber(Math.abs(gauge.margin), locale, gauge.digits) });
  const title = gauge.key === 'tojuli' ? <>TO<sub>juli</sub></> : t(GAUGE_TITLE[gauge.key]);
  return <Card className={`results-gauge results-gauge--${tone}`} title={title} actions={verdict}>
    <span className="results-gauge__sub">{t(`results.dash.gauge.${gauge.key}`)}</span>
    <span className="results-gauge__value" data-testid={`results-gauge-${gauge.key}`}>
      {formatNumber(gauge.value, locale, gauge.digits)} <small>{unit}</small>
    </span>
    <div className="results-gauge__track" role="meter" aria-valuemin={0} aria-valuemax={Number(scaleMax.toFixed(2))}
      aria-valuenow={gauge.value ?? undefined} aria-label={`${typeof title === 'string' ? title : 'TOjuli'}, ${t('results.limit')} ${formatNumber(gauge.limit, locale, gauge.digits)}`}>
      <span className="results-gauge__fill" style={{ width: `${fill}%` }} />
      {marker != null && <span className="results-gauge__marker" style={{ left: `${marker}%` }} aria-hidden="true" />}
    </div>
    <div className="results-gauge__foot">
      <span>{t('results.limit')} {gauge.higherIsBetter ? '≥' : '≤'} {formatNumber(gauge.limit, locale, gauge.digits)}</span>
      {marginText && <span className={gauge.meets === false ? 'results-gauge__excess' : undefined}>{marginText}</span>}
    </div>
  </Card>;
}

function AttentionCard({ assessment, items, onNavigate }: {
  assessment: ProjectPerformanceAssessment; items: Gauge[]; onNavigate: (route: Route) => void;
}) {
  const { t, locale } = useI18n();
  const failures = items.filter((gauge) => gauge.meets === false);
  const issues = kernelIssues(assessment).filter((issue) => issue.kind === 'warning');
  const count = failures.length + issues.length;
  return <Card title={<>{t('results.dash.attention')} <span className="results-count">{count}</span></>} flush>
    {count === 0 && <p className="results-attention__empty"><CheckCircle2 aria-hidden="true" /> {t('results.dash.noAttention')}</p>}
    {failures.length > 0 && <ul className="results-attention">
      {failures.map((gauge) => <li key={gauge.key}>
        <XCircle aria-hidden="true" className="results-attention__icon" />
        <span>
          <b>{t(`results.dash.fail.${gauge.key}`, { value: formatNumber(Math.abs(gauge.margin ?? 0), locale, gauge.digits) })}</b>
          <small>{t('results.dash.failHint')}</small>
        </span>
      </li>)}
    </ul>}
    <IssueList issues={issues.map((issue) => ({
      code: issue.code, severity: 'warning' as const, path: issue.path, detail: issue.detail,
      location: routeLabel(t, routeForPath(issue.path)),
    }))} onGoTo={(issue) => onNavigate(routeForPath(issue.path ?? ''))} />
  </Card>;
}

// ── Per dienst ───────────────────────────────────────────────────────

function ServicesTab({ assessment }: { assessment: ProjectPerformanceAssessment }) {
  const { t, locale } = useI18n();
  const summary = summarizeServiceEnergy(assessment.performance!.energyByService);
  if (!summary) return <Banner tone="info">{t('results.dash.noServices')}</Banner>;
  type Row = (typeof summary.rows)[number];
  const fmt = (value: number) => formatNumber(value, locale, 0);
  const columns: Array<Column<Row>> = [
    { key: 'service', header: t('results.dash.col.service'), render: (row) => t(`nta.performance.service.${row.service}`) },
    ...summary.carriers.map((carrier): Column<Row> => ({
      key: carrier, header: t(`results.carrier.${carrier}`), unit: 'kWh', numeric: true, render: (row) => fmt(row.usedKwh[carrier] ?? 0),
    })),
    { key: 'delivered', header: t('results.dash.col.delivered'), unit: 'kWh', numeric: true, render: (row) => fmt(row.deliveredKwh) },
    { key: 'primary', header: t('results.dash.col.primaryFossil'), unit: 'kWh', numeric: true, render: (row) => fmt(row.primaryFossilKwh) },
    { key: 'renewable', header: t('results.dash.col.renewable'), unit: 'kWh', numeric: true, render: (row) => fmt(row.renewablePrimaryKwh) },
  ];
  return <Card title={t('nta.performance.byService')} flush>
    <DataTable columns={columns} rows={summary.rows} rowKey={(row) => row.service} caption={t('nta.performance.byService')} />
    <dl className="results-totals">
      <div><dt>{t('results.dash.exportCredit')}</dt><dd>−{fmt(summary.exportedElectricityCreditKwh + summary.storageCorrectionKwh)} kWh</dd></div>
      <div><dt>{t('results.dash.renewableElectricity')}</dt><dd>{fmt(summary.renewableElectricityKwh)} kWh</dd></div>
      <div><dt>EP<sub>tot</sub></dt><dd data-testid="results-eptot">{fmt(summary.primaryFossilTotalKwh)} kWh</dd></div>
      <div><dt>EP<sub>ren;tot</sub></dt><dd>{fmt(summary.renewableTotalKwh)} kWh</dd></div>
    </dl>
    <p className="results-note">{t('nta.performance.byService.note')}</p>
  </Card>;
}

// ── Per zone ─────────────────────────────────────────────────────────

function ZonesTab({ assessment }: { assessment: ProjectPerformanceAssessment }) {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const performance = assessment.performance!;
  const demands = zoneDemands(performance);
  const rows = demands.map((zone, index) => ({
    id: String(index),
    name: state.project.zones[index]?.name ?? `${t('results.dash.zone')} ${index + 1}`,
    heat: zone.annualHeatingNeedKwh,
    cold: zone.annualCoolingNeedKwh,
    capacity: zone.specificHeatCapacityKjPerM2k,
  }));
  type Row = (typeof rows)[number];
  const columns: Array<Column<Row>> = [
    { key: 'name', header: t('results.dash.col.zone'), render: (row) => row.name },
    { key: 'heat', header: t('results.series.heatNeed'), unit: 'kWh', numeric: true, render: (row) => formatNumber(row.heat, locale, 0) },
    { key: 'cold', header: t('results.series.coldNeed'), unit: 'kWh', numeric: true, render: (row) => formatNumber(row.cold, locale, 0) },
    { key: 'capacity', header: t('results.dash.col.capacity'), unit: 'kJ/m²K', numeric: true, render: (row) => formatNumber(row.capacity, locale, 0) },
  ];
  const tojuli = performance.tojuli.flatMap((zone) => zone.orientations.map((orientation) => ({
    id: `${zone.zoneId}-${orientation.orientation}`, zone: zone.zoneId, ...orientation,
  })));
  type TRow = (typeof tojuli)[number];
  const tColumns: Array<Column<TRow>> = [
    { key: 'zone', header: t('results.dash.col.zone'), render: (row) => row.zone },
    { key: 'orientation', header: t('results.dash.col.orientation'), render: (row) => row.orientation },
    { key: 'area', header: t('results.dash.col.area'), unit: 'm²', numeric: true, render: (row) => formatNumber(row.areaM2, locale, 1) },
    { key: 'tojuli', header: 'TOjuli', unit: 'K', numeric: true, render: (row) => row.assessed ? formatNumber(row.tojuliK, locale, 2) : '–' },
  ];
  return <>
    <Card title={t('results.dash.zonesTitle')} flush>
      <DataTable columns={columns} rows={rows} rowKey={(row) => row.id} caption={t('results.dash.zonesTitle')} />
    </Card>
    {isResidential(assessment) && tojuli.length > 0 && <Card title={t('results.dash.tojuliTitle')} flush>
      <DataTable columns={tColumns} rows={tojuli} rowKey={(row) => row.id} caption={t('results.dash.tojuliTitle')} />
    </Card>}
  </>;
}

// ── Maandwaarden ─────────────────────────────────────────────────────

function MonthlyTab({ assessment }: { assessment: ProjectPerformanceAssessment }) {
  const { t, locale } = useI18n();
  const performance = assessment.performance!;
  const need = monthlyNeed(performance);
  const pv = monthlyPv(performance);
  const delivered = energySeries(performance, 'delivered').filter((item) => item.key !== 'pv');
  const primary = energySeries(performance, 'primary');
  const months = Array.from({ length: 12 }, (_, index) => index);
  const fmt = (value: number) => formatNumber(value, locale, 0);
  const balance = performance.electricityBalance ?? [];
  const rows = months.map((month) => ({
    month,
    heat: need.heat[month],
    cold: need.cold[month],
    delivered: delivered.reduce((sum, item) => sum + item.values[month], 0),
    pv: pv[month],
    exported: balance.find((row) => row.month === month + 1)?.exportedKwh ?? 0,
    primary: primary.reduce((sum, item) => sum + item.values[month], 0),
  }));
  type Row = (typeof rows)[number];
  const name = (month: number) => new Date(2026, month, 1).toLocaleString(locale, { month: 'long' });
  const columns: Array<Column<Row>> = [
    { key: 'month', header: t('results.chart.month'), render: (row) => name(row.month) },
    { key: 'heat', header: t('results.series.heatNeed'), unit: 'kWh', numeric: true, render: (row) => fmt(row.heat) },
    { key: 'cold', header: t('results.series.coldNeed'), unit: 'kWh', numeric: true, render: (row) => fmt(row.cold) },
    { key: 'delivered', header: t('results.dash.col.delivered'), unit: 'kWh', numeric: true, render: (row) => fmt(row.delivered) },
    { key: 'pv', header: t('results.series.pv'), unit: 'kWh', numeric: true, render: (row) => fmt(row.pv) },
    { key: 'exported', header: t('results.dash.col.exported'), unit: 'kWh', numeric: true, render: (row) => fmt(row.exported) },
    { key: 'primary', header: t('results.dash.col.primaryFossil'), unit: 'kWh', numeric: true, render: (row) => fmt(row.primary) },
  ];
  return <Card title={t('results.dash.monthlyTitle')} flush>
    <DataTable columns={columns} rows={rows} rowKey={(row) => String(row.month)} caption={t('results.dash.monthlyTitle')} />
  </Card>;
}

// ── Herkomst ─────────────────────────────────────────────────────────

function ProvenanceTab({ assessment }: { assessment: ProjectPerformanceAssessment }) {
  const { t } = useI18n();
  const performance = assessment.performance!;
  const [groups, setGroups] = useState<NtaInterpretationGroup[] | null>(null);
  useEffect(() => {
    let cancelled = false;
    fetchKernelInterpretations().then((value) => { if (!cancelled) setGroups(value); }).catch(() => { if (!cancelled) setGroups([]); });
    return () => { cancelled = true; };
  }, []);
  return <>
    <Card title={t('results.dash.provenanceTitle')}>
      <dl className="results-provenance" data-testid="results-provenance">
        <div><dt>{t('results.dash.prov.kernel')}</dt><dd>{assessment.kernelVersion}</dd></div>
        <div><dt>{t('results.dash.prov.norm')}</dt><dd>{assessment.targetNormVersion}</dd></div>
        <div><dt>{t('results.dash.prov.status')}</dt><dd>{assessment.status}</dd></div>
        <div><dt>{t('results.dash.prov.attest')}</dt><dd>{t('results.dash.prov.unattested')}</dd></div>
        <div><dt>{t('results.dash.prov.fingerprint')}</dt><dd><code>{assessment.inputFingerprint}</code></dd></div>
        <div><dt>{t('results.dash.prov.label')}</dt><dd>{performance.labelSource}</dd></div>
        {performance.bblCheck && <div><dt>{t('results.dash.prov.bbl')}</dt><dd>{performance.bblCheck.source}</dd></div>}
      </dl>
    </Card>
    <Card title={t('results.dash.interpretations')}>
      {groups == null && <p role="status"><Info aria-hidden="true" /> {t('results.dash.loadingInterpretations')}</p>}
      {groups != null && groups.length === 0 && <p>{t('results.dash.noInterpretations')}</p>}
      {groups != null && groups.map((group) => <details key={`${group.part}-${group.module}`} className="results-interp">
        <summary>{group.part} <small>({group.items.length})</small></summary>
        <ul>{group.items.map((item) => <li key={item}>{item}</li>)}</ul>
      </details>)}
    </Card>
  </>;
}
