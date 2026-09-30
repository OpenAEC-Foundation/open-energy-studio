import { useI18n } from '../../i18n/i18n';

interface BENGIndicatorCompactProps {
  label: string;
  value: number;
  limit: number;
  unit: string;
  higherIsBetter?: boolean;
}

export function BENGIndicatorCompact({ label, value, limit, unit, higherIsBetter }: BENGIndicatorCompactProps) {
  const { t } = useI18n();

  return (
    <div className="preview-beng-card indicative">
      <div className="preview-beng-header">
        <span className="preview-beng-label">{label}</span>
        <span className="preview-beng-badge indicative">{t('results.indicativeBadge')}</span>
      </div>
      <div className="preview-beng-value">
        {value.toFixed(1)} <span className="preview-beng-unit">{unit}</span>
      </div>
      <div className="preview-beng-limit">
        {higherIsBetter ? '\u2265' : '\u2264'} {limit} {unit}
      </div>
    </div>
  );
}
