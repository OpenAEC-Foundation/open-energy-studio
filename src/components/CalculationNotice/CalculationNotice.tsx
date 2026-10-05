import { Info } from 'lucide-react';
import { useI18n } from '../../i18n/i18n';
import './CalculationNotice.css';

interface CalculationNoticeProps {
  compact?: boolean;
}

export function CalculationNotice({ compact = false }: CalculationNoticeProps) {
  const { t } = useI18n();

  return (
    <div className={`calculation-notice${compact ? ' calculation-notice-compact' : ''}`} role="status">
      <Info size={compact ? 15 : 18} aria-hidden="true" />
      <div>
        <strong>{t('results.indicative')}</strong>
        <p>{t('results.indicativeDescription')}</p>
      </div>
    </div>
  );
}
