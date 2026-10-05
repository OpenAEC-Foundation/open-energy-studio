/**
 * Monthly stacked bar chart in hand-written SVG (UI redesign F7, ontwerp §5.5).
 *
 * Positive series stack upwards from the zero line, negative ones (PV, export
 * credit) downwards. Marks follow the dataviz rules: thin bars with a 2px
 * surface gap between segments, rounded data ends, a recessive grid, one axis,
 * a legend for two or more series and a hover/focus tooltip. Every month is a
 * focusable group with a full text label, and a table view shows the same
 * numbers, so nothing depends on colour or on the pointer alone.
 */
import { useId, useMemo, useState, type KeyboardEvent } from 'react';
import { Table2 } from 'lucide-react';
import { useI18n } from '../../../../i18n/i18n';
import { formatNumber } from '../../../../i18n/format';
import { Button } from '../../../ui';
import type { ChartSeries } from './resultsData';

const WIDTH = 720;
const DEFAULT_HEIGHT = 260;
const PAD = { top: 12, right: 12, bottom: 28, left: 52 };
const GAP = 2;

export interface MonthlyChartProps {
  series: ChartSeries[];
  /** Accessible name of the chart. */
  label: string;
  /** Unit shown on the axis and in the tooltip ("kWh"). */
  unit: string;
  /** Summary sentence read by screen readers next to the table view. */
  summary?: string;
  testId?: string;
  /** `stacked` (default) stacks the series; `grouped` places them side by side (different quantities). */
  layout?: 'stacked' | 'grouped';
  /** Height of the plot in viewBox units (default 260). */
  height?: number;
}

function niceStep(range: number): number {
  if (range <= 0) return 1;
  const raw = range / 4;
  const power = 10 ** Math.floor(Math.log10(raw));
  const fraction = raw / power;
  const nice = fraction <= 1 ? 1 : fraction <= 2 ? 2 : fraction <= 2.5 ? 2.5 : fraction <= 5 ? 5 : 10;
  return nice * power;
}

export function MonthlyChart({ series, label, unit, summary, testId, layout = 'stacked', height = DEFAULT_HEIGHT }: MonthlyChartProps) {
  const HEIGHT = height;
  const grouped = layout === 'grouped';
  const { t, locale } = useI18n();
  const id = useId();
  const [showTable, setShowTable] = useState(false);
  const [active, setActive] = useState<number | null>(null);
  const months = useMemo(() => Array.from({ length: 12 }, (_, index) =>
    new Date(2026, index, 1).toLocaleString(locale, { month: 'short' }).replace('.', '')), [locale]);

  const { up, down } = useMemo(() => {
    const combine = (pick: (value: number) => number) => Array.from({ length: 12 }, (_, month) =>
      grouped
        ? series.reduce((best, item) => (Math.abs(pick(item.values[month] ?? 0)) > Math.abs(best) ? pick(item.values[month] ?? 0) : best), 0)
        : series.reduce((sum, item) => sum + pick(item.values[month] ?? 0), 0));
    const up = combine((value) => Math.max(0, value));
    const down = combine((value) => Math.min(0, value));
    return { up, down };
  }, [series, grouped]);

  const maxUp = Math.max(0, ...up);
  const minDown = Math.min(0, ...down);
  const step = niceStep(maxUp - minDown);
  const top = Math.max(step, Math.ceil(maxUp / step) * step);
  const bottom = minDown < 0 ? Math.floor(minDown / step) * step : 0;
  const plotH = HEIGHT - PAD.top - PAD.bottom;
  const plotW = WIDTH - PAD.left - PAD.right;
  const y = (value: number) => PAD.top + ((top - value) / (top - bottom)) * plotH;
  const slot = plotW / 12;
  const barW = Math.min(36, slot * 0.56);
  const ticks: number[] = [];
  for (let value = bottom; value <= top + step / 2; value += step) ticks.push(value);

  const fmt = (value: number) => formatNumber(Math.round(value), locale, 0);
  const describe = (month: number) => [
    months[month],
    ...series.filter((item) => (item.values[month] ?? 0) !== 0)
      .map((item) => `${t(item.labelKey)} ${fmt(item.values[month])} ${unit}`),
  ].join(', ');

  const onKeyDown = (event: KeyboardEvent<SVGGElement>, month: number) => {
    const next = event.key === 'ArrowRight' ? month + 1 : event.key === 'ArrowLeft' ? month - 1 : null;
    if (next == null || next < 0 || next > 11) return;
    event.preventDefault();
    const target = (event.currentTarget.parentNode as SVGGElement | null)
      ?.querySelector<SVGGElement>(`[data-month="${next}"]`);
    target?.focus();
  };

  const tooltipMonth = active;
  const tooltipLeft = tooltipMonth == null ? 0 : ((PAD.left + slot * (tooltipMonth + 0.5)) / WIDTH) * 100;

  return (
    <div className="mchart" data-testid={testId}>
      <div className="mchart__bar">
        {series.length >= 2 && <ul className="mchart__legend" aria-label={t('results.chart.legend')}>
          {series.map((item) => <li key={item.key}>
            <span className={item.hatched ? 'mchart__swatch mchart__swatch--hatch' : 'mchart__swatch'}
              style={{ ['--swatch' as string]: item.color }} aria-hidden="true" />
            {t(item.labelKey)}{' '}
            <span className="mchart__legend-total">{fmt(item.values.reduce((sum, value) => sum + value, 0))}</span>
          </li>)}
        </ul>}
        <Button size="sm" variant="ghost" className="mchart__toggle" aria-pressed={showTable} icon={<Table2 aria-hidden="true" />}
          onClick={() => setShowTable((value) => !value)}>
          {showTable ? t('results.chart.showChart') : t('results.chart.showTable')}
        </Button>
      </div>

      {showTable ? (
        <div className="mchart__table-wrap">
          <table className="ui-table mchart__table">
            <caption>{label} ({unit})</caption>
            <thead><tr><th scope="col">{t('results.chart.month')}</th>
              {series.map((item) => <th key={item.key} scope="col" className="num">{t(item.labelKey)}</th>)}</tr></thead>
            <tbody>{months.map((name, month) => <tr key={name}>
              <th scope="row">{name}</th>
              {series.map((item) => <td key={item.key} className="num">{fmt(item.values[month] ?? 0)}</td>)}
            </tr>)}</tbody>
            <tfoot><tr><th scope="row">{t('results.chart.year')}</th>
              {series.map((item) => <td key={item.key} className="num">{fmt(item.values.reduce((sum, value) => sum + value, 0))}</td>)}
            </tr></tfoot>
          </table>
        </div>
      ) : (
        <div className="mchart__plot">
          <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} role="group" aria-label={label} aria-describedby={summary ? `${id}-sum` : undefined}
            preserveAspectRatio="xMidYMid meet">
            <defs>
              <pattern id={`${id}-hatch`} patternUnits="userSpaceOnUse" width="6" height="6" patternTransform="rotate(45)">
                <rect width="6" height="6" fill="currentColor" opacity="0.35" />
                <line x1="0" y1="0" x2="0" y2="6" stroke="currentColor" strokeWidth="2.5" />
              </pattern>
            </defs>
            {ticks.map((value) => <g key={value} aria-hidden="true">
              <line className={value === 0 ? 'mchart__zero' : 'mchart__grid'} x1={PAD.left} x2={WIDTH - PAD.right} y1={y(value)} y2={y(value)} />
              <text className="mchart__tick" x={PAD.left - 8} y={y(value) + 4} textAnchor="end">{fmt(value)}</text>
            </g>)}
            {months.map((name, month) => {
              const x = PAD.left + slot * month + (slot - barW) / 2;
              let upCursor = 0;
              let downCursor = 0;
              const groupW = grouped ? Math.max(4, (barW - GAP * (series.length - 1)) / Math.max(1, series.length)) : barW;
              const segments = series.map((item, index) => {
                const value = item.values[month] ?? 0;
                if (value === 0) return null;
                if (grouped) {
                  const y1 = y(Math.max(0, value));
                  const y2 = y(Math.min(0, value));
                  return <rect key={item.key} x={x + index * (groupW + GAP)} width={groupW} y={y1} height={Math.max(0, y2 - y1)} rx={2}
                    style={item.hatched ? { color: item.color } : undefined}
                    fill={item.hatched ? `url(#${id}-hatch)` : item.color} />;
                }
                const from = value > 0 ? upCursor : downCursor;
                const to = from + value;
                if (value > 0) upCursor = to; else downCursor = to;
                const y1 = y(Math.max(from, to));
                const y2 = y(Math.min(from, to));
                const height = Math.max(0, y2 - y1 - GAP);
                return <rect key={item.key} x={x} width={barW} y={value > 0 ? y1 : y1 + GAP} height={height} rx={2}
                  style={item.hatched ? { color: item.color } : undefined}
                  fill={item.hatched ? `url(#${id}-hatch)` : item.color} />;
              });
              return <g key={name} data-month={month} tabIndex={0} role="img" aria-label={describe(month)}
                className={active === month ? 'mchart__month mchart__month--on' : 'mchart__month'}
                onMouseEnter={() => setActive(month)} onMouseLeave={() => setActive(null)}
                onFocus={() => setActive(month)} onBlur={() => setActive(null)}
                onKeyDown={(event) => onKeyDown(event, month)}>
                <rect className="mchart__hit" x={PAD.left + slot * month} width={slot} y={PAD.top} height={plotH} />
                {segments}
                <text className="mchart__tick" x={x + barW / 2} y={HEIGHT - 8} textAnchor="middle" aria-hidden="true">{name}</text>
              </g>;
            })}
          </svg>
          {tooltipMonth != null && <div className="mchart__tip" role="presentation"
            style={{ left: `${tooltipLeft}%` }}>
            <strong>{months[tooltipMonth]}</strong>
            {series.filter((item) => (item.values[tooltipMonth] ?? 0) !== 0).map((item) => <span key={item.key} className="mchart__tip-row">
              <span className={item.hatched ? 'mchart__swatch mchart__swatch--hatch' : 'mchart__swatch'}
                style={{ ['--swatch' as string]: item.color }} aria-hidden="true" />
              <span>{t(item.labelKey)}</span>
              <b>{fmt(item.values[tooltipMonth])} {unit}</b>
            </span>)}
          </div>}
        </div>
      )}
      {summary && <p id={`${id}-sum`} className="visually-hidden">{summary}</p>}
    </div>
  );
}
