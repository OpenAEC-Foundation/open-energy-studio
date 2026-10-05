import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import './BENGIndicator.css';

interface BENGIndicatorProps {
  title: string;
  subtitle: string;
  value: number;
  /** The Bbl limit; null when it is not determined. */
  limit: number | null;
  unit: string;
  higherIsBetter?: boolean; // for BENG3
  /** `kernel`: a value of the NTA kernel; `indicative`: the simplified engine. */
  source?: 'indicative' | 'kernel';
  digits?: number;
}

export function BENGIndicator({
  title, subtitle, value, limit, unit, higherIsBetter, source = 'indicative', digits = 1,
}: BENGIndicatorProps) {
  const { t, locale } = useI18n();

  const ratio = limit ? Math.min(value / limit, 2) : 0;
  const barWidth = Math.min(Math.abs(ratio) * 100, 200);

  return (
    <div className={`beng-indicator ${source === 'kernel' ? 'beng-kernel' : 'beng-indicative'}`} data-source={source}>
      <div className="beng-indicator-header">
        <span className="beng-indicator-title">{title}</span>
        <span className={`beng-indicator-badge ${source === 'kernel' ? 'kernel' : 'indicative'}`}>
          {source === 'kernel' ? t('results.kernelBadge') : t('results.indicativeBadge')}
        </span>
      </div>
      <div className="beng-indicator-subtitle">{subtitle}</div>
      <div className="beng-indicator-value">
        {formatNumber(value, locale, digits)} <span className="beng-indicator-unit">{unit}</span>
      </div>
      {limit != null && <>
        <div className="beng-indicator-bar-container">
          <div
            className="beng-indicator-bar bar-indicative"
            style={{ width: `${Math.min(barWidth, 100)}%` }}
          />
          <div
            className="beng-indicator-limit-line"
            style={{ left: `${(1 / Math.max(ratio, 1)) * 100}%` }}
          />
        </div>
        <div className="beng-indicator-limit">
          {t('results.limit')}: {higherIsBetter ? '≥' : '≤'} {formatNumber(limit, locale, Number.isInteger(limit) ? 0 : digits)} {unit}
        </div>
      </>}
    </div>
  );
}
