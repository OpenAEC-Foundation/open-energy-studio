import { useI18n } from '../../i18n/i18n';
import type { IMonthlyBreakdown } from '../../core/energy/types';
import './MonthlyBreakdownChart.css';

interface MonthlyBreakdownChartProps {
  monthly: IMonthlyBreakdown[];
}

const MONTH_KEYS = [
  'month.jan', 'month.feb', 'month.mar', 'month.apr',
  'month.may', 'month.jun', 'month.jul', 'month.aug',
  'month.sep', 'month.oct', 'month.nov', 'month.dec',
];

const MONTH_SHORT = ['J', 'F', 'M', 'A', 'M', 'J', 'J', 'A', 'S', 'O', 'N', 'D'];

export function MonthlyBreakdownChart({ monthly }: MonthlyBreakdownChartProps) {
  const { t } = useI18n();

  // Calculate max for scaling the stacked bars
  const maxDemand = Math.max(
    ...monthly.map(m => m.heatingDemand + m.coolingDemand),
    1
  );

  return (
    <div className="monthly-breakdown-chart">
      <h3>{t('preview.monthlyProfile')}</h3>

      {/* Stacked bar chart */}
      <div className="monthly-chart-container">
        {monthly.map((m, i) => {
          const heatPct = (m.heatingDemand / maxDemand) * 100;
          const coolPct = (m.coolingDemand / maxDemand) * 100;

          return (
            <div key={i} className="monthly-chart-col">
              <div className="monthly-chart-stacked">
                {m.coolingDemand > 0 && (
                  <div
                    className="monthly-chart-stack-bar cooling"
                    style={{ height: `${coolPct}%` }}
                    title={`${t('results.coolingDemand')}: ${m.coolingDemand.toFixed(0)} kWh`}
                  />
                )}
                {m.heatingDemand > 0 && (
                  <div
                    className="monthly-chart-stack-bar heating"
                    style={{ height: `${heatPct}%` }}
                    title={`${t('results.heatingDemand')}: ${m.heatingDemand.toFixed(0)} kWh`}
                  />
                )}
              </div>
              <span className="monthly-chart-month-label">{MONTH_SHORT[i]}</span>
            </div>
          );
        })}
      </div>

      {/* Legend */}
      <div className="monthly-chart-legend">
        <span className="monthly-legend-item">
          <span className="monthly-legend-dot" style={{ background: 'var(--viz-heating)' }} />
          {t('results.heatingDemand')}
        </span>
        <span className="monthly-legend-item">
          <span className="monthly-legend-dot" style={{ background: 'var(--viz-cooling)' }} />
          {t('results.coolingDemand')}
        </span>
      </div>

      {/* Data table */}
      <div className="monthly-table-container">
        <table className="monthly-table">
          <thead>
            <tr>
              <th>{t('preview.month')}</th>
              <th>{t('results.heatingDemand')}</th>
              <th>{t('results.coolingDemand')}</th>
              <th>{t('results.solarGain')}</th>
              <th>{t('results.transmissionLoss')}</th>
            </tr>
          </thead>
          <tbody>
            {monthly.map((m, i) => (
              <tr key={i}>
                <td>{t(MONTH_KEYS[i])}</td>
                <td>{m.heatingDemand.toFixed(0)}</td>
                <td>{m.coolingDemand.toFixed(0)}</td>
                <td>{m.solarGain.toFixed(0)}</td>
                <td>{m.transmissionLoss.toFixed(0)}</td>
              </tr>
            ))}
            <tr>
              <td>{t('preview.total')}</td>
              <td>{monthly.reduce((s, m) => s + m.heatingDemand, 0).toFixed(0)}</td>
              <td>{monthly.reduce((s, m) => s + m.coolingDemand, 0).toFixed(0)}</td>
              <td>{monthly.reduce((s, m) => s + m.solarGain, 0).toFixed(0)}</td>
              <td>{monthly.reduce((s, m) => s + m.transmissionLoss, 0).toFixed(0)}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}
