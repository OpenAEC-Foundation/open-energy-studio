import { useI18n } from '../../i18n/i18n';
import './BENGIndicator.css';

interface BENGIndicatorProps {
  title: string;
  subtitle: string;
  value: number;
  limit: number;
  unit: string;
  pass: boolean;
  higherIsBetter?: boolean; // for BENG3
}

export function BENGIndicator({ title, subtitle, value, limit, unit, pass, higherIsBetter }: BENGIndicatorProps) {
  const { t } = useI18n();

  const ratio = higherIsBetter
    ? Math.min(value / limit, 2)
    : Math.min(value / limit, 2);
  const barWidth = Math.min(Math.abs(ratio) * 100, 200);

  return (
    <div className={`beng-indicator ${pass ? 'beng-pass' : 'beng-fail'}`}>
      <div className="beng-indicator-header">
        <span className="beng-indicator-title">{title}</span>
        <span className={`beng-indicator-badge ${pass ? 'pass' : 'fail'}`}>
          {pass ? t('results.pass') : t('results.fail')}
        </span>
      </div>
      <div className="beng-indicator-subtitle">{subtitle}</div>
      <div className="beng-indicator-value">
        {value.toFixed(1)} <span className="beng-indicator-unit">{unit}</span>
      </div>
      <div className="beng-indicator-bar-container">
        <div
          className={`beng-indicator-bar ${pass ? 'bar-pass' : 'bar-fail'}`}
          style={{ width: `${Math.min(barWidth, 100)}%` }}
        />
        <div
          className="beng-indicator-limit-line"
          style={{ left: `${(1 / Math.max(ratio, 1)) * 100}%` }}
        />
      </div>
      <div className="beng-indicator-limit">
        {t('results.limit')}: {higherIsBetter ? '≥' : '≤'} {limit} {unit}
      </div>
    </div>
  );
}
