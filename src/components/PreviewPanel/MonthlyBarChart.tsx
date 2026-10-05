import { useI18n } from '../../i18n/i18n';

interface MonthlyBarChartProps {
  /** Monthly heating need Q_H;nd, kWh (January first). */
  heating: number[];
  /** Monthly cooling need Q_C;nd, kWh (January first). */
  cooling: number[];
}

const MONTH_LABELS = ['J', 'F', 'M', 'A', 'M', 'J', 'J', 'A', 'S', 'O', 'N', 'D'];

export function MonthlyBarChart({ heating, cooling }: MonthlyBarChartProps) {
  const { t } = useI18n();
  const maxVal = Math.max(...heating, ...cooling, 1);

  return (
    <div className="preview-monthly-chart">
      <div className="preview-chart-bars">
        {MONTH_LABELS.map((label, i) => {
          const heat = heating[i] ?? 0;
          const cool = cooling[i] ?? 0;
          return (
            <div key={i} className="preview-chart-column">
              <div className="preview-chart-bar-container">
                {heat > 0 && (
                  <div className="preview-chart-bar heat" style={{ height: `${(heat / maxVal) * 100}%` }}
                    title={`${heat.toFixed(0)} kWh`} />
                )}
                {cool > 0 && (
                  <div className="preview-chart-bar cool" style={{ height: `${(cool / maxVal) * 100}%` }}
                    title={`${cool.toFixed(0)} kWh`} />
                )}
              </div>
              <span className="preview-chart-label">{label}</span>
            </div>
          );
        })}
      </div>
      <div className="preview-chart-legend">
        <span className="preview-legend-item">
          <span className="preview-legend-dot heat" />
          {t('preview.heating')}
        </span>
        <span className="preview-legend-item">
          <span className="preview-legend-dot cool" />
          {t('preview.cooling')}
        </span>
      </div>
    </div>
  );
}
