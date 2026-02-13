import type { IMonthlyBreakdown } from '../../core/energy/types';

interface MonthlyBarChartProps {
  monthly: IMonthlyBreakdown[];
}

const MONTH_LABELS = ['J', 'F', 'M', 'A', 'M', 'J', 'J', 'A', 'S', 'O', 'N', 'D'];

export function MonthlyBarChart({ monthly }: MonthlyBarChartProps) {
  // Find max value for scaling
  const maxVal = Math.max(
    ...monthly.map(m => Math.max(m.heatingDemand, m.coolingDemand)),
    1
  );

  return (
    <div className="preview-monthly-chart">
      <div className="preview-chart-bars">
        {monthly.map((m, i) => {
          const heatHeight = (m.heatingDemand / maxVal) * 100;
          const coolHeight = (m.coolingDemand / maxVal) * 100;

          return (
            <div key={i} className="preview-chart-column">
              <div className="preview-chart-bar-container">
                {m.heatingDemand > 0 && (
                  <div
                    className="preview-chart-bar heat"
                    style={{ height: `${heatHeight}%` }}
                    title={`${m.heatingDemand.toFixed(0)} kWh`}
                  />
                )}
                {m.coolingDemand > 0 && (
                  <div
                    className="preview-chart-bar cool"
                    style={{ height: `${coolHeight}%` }}
                    title={`${m.coolingDemand.toFixed(0)} kWh`}
                  />
                )}
              </div>
              <span className="preview-chart-label">{MONTH_LABELS[i]}</span>
            </div>
          );
        })}
      </div>
      <div className="preview-chart-legend">
        <span className="preview-legend-item">
          <span className="preview-legend-dot heat" />
          Verwarming
        </span>
        <span className="preview-legend-item">
          <span className="preview-legend-dot cool" />
          Koeling
        </span>
      </div>
    </div>
  );
}
