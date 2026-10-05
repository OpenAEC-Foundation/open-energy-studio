/**
 * Maatwerkadvies views of the redesign (ontwerp §8 "Maatwerkadvies", mockup 08):
 * the label path chart, the package comparison, the measure cards and the
 * package inspector. The panel keeps the state; these only render it.
 */
import { Pencil, Trash2 } from 'lucide-react';
import { useI18n } from '../../../../i18n/i18n';
import { formatNumber } from '../../../../i18n/format';
import type { IProject } from '../../../../core/energy/types';
import type {
  MaatwerkadviesAssessment, MwaMeasure, MwaPackage, MwaVariantResult,
} from '../../../../core/nta/KernelClient';
import { buildTemplatePatch, type MwaTemplateKind } from '../../../../core/nta/MwaTemplates';
import { labelColor } from '../results/resultsData';
import { Button, IconButton, Pill, Segmented, StatusPill } from '../../../ui';

export type MwaOrder = 'input' | 'payback' | 'npv';

export interface LabelPathBar {
  id: string;
  name: string;
  kind: MwaVariantResult['kind'];
  labelClass: string | null;
  ep2: number | null;
  advised: boolean;
  valid: boolean;
}

/** The bars of the label path: the current state, then the packages (or the measures when there are no packages). */
export function labelPathBars(assessment: MaatwerkadviesAssessment, advisedId: string | null | undefined, currentName: string): LabelPathBar[] {
  const variants = assessment.packages.length > 0 ? assessment.packages : assessment.measures;
  const bar = (item: MwaVariantResult, name: string): LabelPathBar => ({
    id: `${item.kind}-${item.id}`, name, kind: item.kind,
    labelClass: item.label.labelClass, ep2: item.label.primaryFossilIndicatorKwhPerM2,
    advised: item.kind === 'package' && item.id === advisedId, valid: item.valid,
  });
  return [
    ...(assessment.current ? [bar(assessment.current, currentName)] : []),
    ...variants.map((item, index) => bar(item, `${index + 1} · ${item.name || item.id}`)),
  ];
}

const BAR_W = 56;
const SLOT = 120;
const HEIGHT = 220;
const TOP = 54;
const BOTTOM = 14;

/** EP₂ per variant as columns with the label class above each column; the advised package is amber. */
export function LabelPathChart({ bars }: { bars: LabelPathBar[] }) {
  const { t, locale } = useI18n();
  const max = Math.max(1, ...bars.map((item) => item.ep2 ?? 0));
  const plot = HEIGHT - TOP - BOTTOM;
  const width = Math.max(SLOT * bars.length, SLOT);
  return <figure className="mwa-labelpath">
    <div className="mwa-labelpath__scroll">
      <svg viewBox={`0 0 ${width} ${HEIGHT}`} width={width} height={HEIGHT} role="img"
        aria-label={t('mwa.page.labelPath.summary', {
          items: bars.map((item) => `${item.name}: ${item.labelClass ?? '–'} ${formatNumber(item.ep2, locale, 0)}`).join('; '),
        })}>
        {bars.map((item, index) => {
          const value = item.ep2 ?? 0;
          const height = Math.max(2, (value / max) * plot);
          const x = index * SLOT + (SLOT - BAR_W) / 2;
          const y = HEIGHT - BOTTOM - height;
          return <g key={item.id} className={item.advised ? 'mwa-labelpath__bar mwa-labelpath__bar--advised' : 'mwa-labelpath__bar'}>
            <title>{`${item.name}: ${item.labelClass ?? '–'}, EP₂ ${formatNumber(item.ep2, locale, 1)} kWh/m²`}</title>
            <rect x={x} y={y} width={BAR_W} height={height} rx={3} />
            {item.labelClass && <>
              <path d={`M${x + 6} ${y - 46} h30 l9 10 l-9 10 h-30 z`} fill={labelColor(item.labelClass)} />
              <text x={x + 21} y={y - 32} textAnchor="middle" className="mwa-labelpath__class">{item.labelClass}</text>
            </>}
            <text x={x + BAR_W / 2} y={y - 8} textAnchor="middle" className="mwa-labelpath__value">
              {item.ep2 == null ? '–' : formatNumber(item.ep2, locale, 0)}</text>
          </g>;
        })}
      </svg>
      <ol className="mwa-labelpath__names" style={{ width }}>
        {bars.map((item) => <li key={item.id} className={item.advised ? 'is-advised' : undefined}>
          {item.name}{!item.valid && <span title={t('mwa.variantInvalid')}> ⚠</span>}</li>)}
      </ol>
    </div>
  </figure>;
}

/** Packages with investment, payback and NPV; sort with NCW / TVT / invoervolgorde. */
export function PackageComparison({ packages, advisedId, order, onOrder }: {
  packages: MwaVariantResult[];
  advisedId: string | null | undefined;
  order: MwaOrder;
  onOrder: (order: MwaOrder) => void;
}) {
  const { t, locale } = useI18n();
  const money = (value: number | null | undefined) => (value == null ? '–' : `€ ${formatNumber(value, locale, 0)}`);
  return <div className="mwa-compare">
    <Segmented<MwaOrder> size="sm" aria-label={t('mwa.order')} value={order} onChange={onOrder} options={[
      { value: 'npv', label: t('mwa.page.order.npv') },
      { value: 'payback', label: t('mwa.page.order.payback') },
      { value: 'input', label: t('mwa.page.order.input') },
    ]} />
    <div className="ui-table-wrap">
      <table className="ui-table mwa-compare__table">
        <thead><tr>
          <th scope="col">{t('mwa.col.variant')}</th>
          <th scope="col" className="ui-num-cell">{t('mwa.col.label')}</th>
          <th scope="col" className="ui-num-cell">EP₂</th>
          <th scope="col" className="ui-num-cell">{t('mwa.col.investment')}</th>
          <th scope="col" className="ui-num-cell">{t('mwa.page.col.payback')}</th>
          <th scope="col" className="ui-num-cell">{t('mwa.col.npv')}</th>
        </tr></thead>
        <tbody>
          {packages.map((item) => {
            const advised = item.id === advisedId;
            return <tr key={item.id} className={advised ? 'is-advised' : undefined} data-invalid={item.valid ? undefined : 'true'}>
              <th scope="row">{item.name || item.id}{advised && <> <Pill tone="accent">{t('mwa.page.advised')}</Pill></>}
                {!item.valid && <> <StatusPill tone="warn">{t('mwa.variantInvalid')}</StatusPill></>}</th>
              <td className="ui-num-cell">{item.label.labelClass ?? '–'}</td>
              <td className="ui-num-cell">{formatNumber(item.label.primaryFossilIndicatorKwhPerM2, locale, 0)}</td>
              <td className="ui-num-cell">{money(item.investmentEur)}</td>
              <td className="ui-num-cell">{item.simplePaybackYears == null ? '–' : `${formatNumber(item.simplePaybackYears, locale, 1)} ${t('mwa.page.years')}`}</td>
              <td className={`ui-num-cell ${item.netPresentValueEur != null && item.netPresentValueEur < 0 ? 'is-negative' : 'is-positive'}`}>
                {money(item.netPresentValueEur)}</td>
            </tr>;
          })}
        </tbody>
      </table>
    </div>
  </div>;
}

/** Open problems of a measure before the kernel runs (template problems, or a manual measure without changes). */
export function measureProblems(project: IProject, measure: MwaMeasure): string[] {
  if (measure.template) {
    try {
      return buildTemplatePatch(project, measure.template, measure.id).problems;
    } catch {
      return ['calculationRequired'];
    }
  }
  return measure.patch.length === 0 ? ['noChange'] : [];
}

/** One measure as a card: template picker, key figures, package membership, edit and remove. */
export function MeasureCard({ measure, packages, problems, result, kinds, onKind, onTogglePackage, onEdit, onRemove }: {
  measure: MwaMeasure;
  packages: MwaPackage[];
  problems: string[];
  result: MwaVariantResult | undefined;
  kinds: readonly MwaTemplateKind[];
  onKind: (kind: MwaTemplateKind | 'manual') => void;
  onTogglePackage: (packageIndex: number) => void;
  onEdit: () => void;
  onRemove: () => void;
}) {
  const { t, locale } = useI18n();
  const kind = measure.template?.kind ?? 'manual';
  const name = measure.name || measure.id;
  const selectId = `mwa-kind-${measure.id}`;
  return <li className={`mwa-card${problems.length > 0 ? ' mwa-card--incomplete' : ''}`} data-testid={`mwa-card-${measure.id}`}>
    <div className="mwa-card__head">
      <strong className="mwa-card__name">{name}</strong>
      <span className="mwa-card__category">{t(`mwa.category.${measure.category}`)}</span>
      {problems.length > 0
        ? <StatusPill tone="warn" title={problems.map((key) => t(`mwa.template.problem.${key}`)).join(' ')}>{t('mwa.page.incomplete')}</StatusPill>
        : <StatusPill tone="ok">{t('mwa.page.complete')}</StatusPill>}
    </div>
    <div className="mwa-card__body">
      <label className="mwa-card__kind" htmlFor={selectId}>{t('mwa.template.kind')}</label>
      <select id={selectId} className="ui-input" value={kind} onChange={(event) => onKind(event.target.value as MwaTemplateKind | 'manual')}>
        {kinds.map((item) => <option key={item} value={item}>{t(`mwa.template.kind.${item}`)}</option>)}
        <option value="manual">{t('mwa.template.kind.manual')}</option>
      </select>
      <dl className="mwa-card__figures">
        <div><dt>{t('mwa.col.investment')}</dt><dd>€ {formatNumber(measure.investmentEur, locale, 0)}</dd></div>
        <div><dt>{t('mwa.measure.lifetime')}</dt><dd>{formatNumber(measure.lifetimeYears, locale, 0)}</dd></div>
        {result && <>
          <div><dt>{t('mwa.page.col.payback')}</dt><dd>{result.simplePaybackYears == null ? '–'
            : `${formatNumber(result.simplePaybackYears, locale, 1)} ${t('mwa.page.years')}`}</dd></div>
          <div><dt>{t('mwa.col.npv')}</dt><dd className={result.netPresentValueEur != null && result.netPresentValueEur < 0 ? 'is-negative' : 'is-positive'}>
            {result.netPresentValueEur == null ? '–' : `€ ${formatNumber(result.netPresentValueEur, locale, 0)}`}</dd></div>
        </>}
      </dl>
      {problems.length > 0 && <ul className="mwa-card__problems">
        {problems.map((key) => <li key={key}>{t(`mwa.template.problem.${key}`)}</li>)}
      </ul>}
    </div>
    <div className="mwa-card__foot">
      {packages.length > 0 && <div className="mwa-card__packages" role="group" aria-label={t('mwa.page.inPackage', { name })}>
        {packages.map((item, index) => {
          const member = item.measureIds.includes(measure.id);
          return <button key={item.id} type="button" className={`mwa-chip${member ? ' is-on' : ''}`} aria-pressed={member}
            title={item.name || item.id} aria-label={item.name || item.id} onClick={() => onTogglePackage(index)}>{index + 1}</button>;
        })}
      </div>}
      <span className="mwa-card__spacer" />
      <Button size="sm" variant="ghost" icon={<Pencil aria-hidden="true" />} onClick={onEdit}
        aria-label={t('mwa.page.editMeasure', { name })}>{t('mwa.page.edit')}</Button>
      <IconButton size="sm" aria-label={t('mwa.page.removeMeasure', { name })} onClick={onRemove}
        icon={<Trash2 aria-hidden="true" />} />
    </div>
  </li>;
}

/** The chosen package: label from → to and the key figures (mockup 08, inspector). */
export function PackageInspector({ current, chosen, chosenBy }: {
  current: MwaVariantResult | null;
  chosen: MwaVariantResult;
  chosenBy: 'adviser' | 'automatic' | string;
}) {
  const { t, locale } = useI18n();
  const savings = chosen.savings;
  const badge = (labelClass: string | null | undefined) => <span className="mwa-inspector__class"
    style={{ background: labelClass ? labelColor(labelClass) : undefined }}>{labelClass ?? '–'}</span>;
  const figure = (label: string, value: string) => <div className="mwa-inspector__figure"><span>{label}</span><strong>{value}</strong></div>;
  const signed = (value: number | null | undefined, digits = 0) => (value == null ? '–'
    : `${value > 0 ? '−' : value < 0 ? '+' : ''}${formatNumber(Math.abs(value), locale, digits)}`);
  return <section className="mwa-inspector" aria-label={t('mwa.page.inspector', { name: chosen.name })}>
    <header className="mwa-inspector__head">
      <strong>{chosen.name || chosen.id}</strong>
      <Pill tone="accent">{chosenBy === 'adviser' ? t('mwa.advised.adviser') : t('mwa.advised.automatic')}</Pill>
    </header>
    <p className="mwa-inspector__path" aria-label={t('mwa.page.labelFromTo', {
      from: current?.label.labelClass ?? '–', to: chosen.label.labelClass ?? '–',
    })}>{badge(current?.label.labelClass)}<span aria-hidden="true">→</span>{badge(chosen.label.labelClass)}</p>
    <div className="mwa-inspector__grid">
      {figure(t('mwa.col.investment'), `€ ${formatNumber(chosen.investmentEur, locale, 0)}`)}
      {figure(t('mwa.col.saving'), savings ? `€ ${formatNumber(savings.energyCostEur, locale, 0)}` : '–')}
      {figure(t('mwa.page.col.payback'), chosen.simplePaybackYears == null ? '–'
        : `${formatNumber(chosen.simplePaybackYears, locale, 1)} ${t('mwa.page.years')}`)}
      {figure('CO₂ [kg]', signed(savings?.co2Kg))}
      {figure(t('mwa.col.gas'), signed(savings?.gasM3))}
      {figure(t('mwa.page.electricityKwh'), signed(savings?.electricityKwh))}
    </div>
    {chosen.phasing.length > 0 && <div className="mwa-inspector__phasing">
      <span>{t('mwa.page.phasing')}</span>
      <ul>{chosen.phasing.map((phase, index) => <li key={index}>
        <strong>{phase.year ?? t('mwa.page.phaseNow')}</strong> {phase.measureIds.join(', ')}</li>)}</ul>
    </div>}
  </section>;
}
