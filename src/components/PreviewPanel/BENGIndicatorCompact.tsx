import { useI18n } from '../../i18n/i18n';

interface BENGIndicatorCompactProps {
  label: string;
  value: number | null;
  limit: number | null;
  unit: string;
  higherIsBetter?: boolean;
  digits?: number;
}

/** One kernel indicator with its Bbl limit (unverified until attested). */
export function BENGIndicatorCompact({ label, value, limit, unit, higherIsBetter, digits = 1 }: BENGIndicatorCompactProps) {
  const { t } = useI18n();
  const meets = value == null || limit == null ? null : higherIsBetter ? value >= limit : value <= limit;
  return (
    <div className={`preview-beng-card${meets === false ? ' fails' : ''}`}>
      <div className="preview-beng-header">
        <span className="preview-beng-label">{label}</span>
        <span className="preview-beng-badge">{t('nta.performance.unverified')}</span>
      </div>
      <div className="preview-beng-value">
        {value == null ? '–' : value.toFixed(digits)} <span className="preview-beng-unit">{unit}</span>
      </div>
      {limit != null && (
        <div className="preview-beng-limit">
          {higherIsBetter ? '≥' : '≤'} {limit.toFixed(digits)} {unit}
        </div>
      )}
    </div>
  );
}
