import { useI18n } from '../../i18n/i18n';

interface BENGIndicatorCompactProps {
  label: string;
  value: number;
  limit: number;
  unit: string;
  pass: boolean;
  higherIsBetter?: boolean;
}

export function BENGIndicatorCompact({ label, value, limit, unit, pass, higherIsBetter }: BENGIndicatorCompactProps) {
  const { t } = useI18n();

  return (
    <div className={`preview-beng-card ${pass ? 'pass' : 'fail'}`}>
      <div className="preview-beng-header">
        <span className="preview-beng-label">{label}</span>
        <span className={`preview-beng-badge ${pass ? 'pass' : 'fail'}`}>
          {pass ? t('results.pass') : t('results.fail')}
        </span>
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
