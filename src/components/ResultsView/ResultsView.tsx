import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { BENGIndicator } from '../BENGIndicator/BENGIndicator';
import { EnergyBreakdownChart } from '../EnergyBreakdownChart/EnergyBreakdownChart';
import { MonthlyBreakdownChart } from '../MonthlyBreakdownChart/MonthlyBreakdownChart';
import type { IBENGResultMonthly } from '../../core/energy/types';
import './ResultsView.css';

export function ResultsView() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { result } = state;

  if (!result) {
    return (
      <div className="results-view">
        <div className="results-empty">
          <h2>{t('results.title')}</h2>
          <p>{t('results.noResults')}</p>
        </div>
      </div>
    );
  }

  // Check if result has monthly data (IBENGResultMonthly)
  const monthlyResult = 'monthly' in result ? result as IBENGResultMonthly : null;

  return (
    <div className="results-view">
      <h2>{t('results.title')}</h2>

      <div className="beng-cards">
        <BENGIndicator
          title={t('results.beng1.title')}
          subtitle={t('results.beng1.subtitle')}
          value={result.beng1}
          limit={result.beng1Limit}
          unit={t('results.beng1.unit')}
          pass={result.beng1Pass}
        />
        <BENGIndicator
          title={t('results.beng2.title')}
          subtitle={t('results.beng2.subtitle')}
          value={result.beng2}
          limit={result.beng2Limit}
          unit={t('results.beng2.unit')}
          pass={result.beng2Pass}
        />
        <BENGIndicator
          title={t('results.beng3.title')}
          subtitle={t('results.beng3.subtitle')}
          value={result.beng3}
          limit={result.beng3Limit}
          unit={t('results.beng3.unit')}
          pass={result.beng3Pass}
          higherIsBetter
        />
      </div>

      {/* TO-juli indicator */}
      {monthlyResult && (
        <div className={`to-juli-card ${monthlyResult.toJuli.pass ? 'to-juli-pass' : 'to-juli-fail'}`}>
          <div className="to-juli-header">
            <h3>{t('results.toJuli')}</h3>
            <span className={`to-juli-badge ${monthlyResult.toJuli.pass ? 'pass' : 'fail'}`}>
              {monthlyResult.toJuli.pass ? t('results.pass') : t('results.fail')}
            </span>
          </div>
          <div className="to-juli-value">
            GTO: {monthlyResult.toJuli.gto.toFixed(2)}
            <span className="to-juli-limit">
              {' '}/ {t('results.limit')}: {'\u2264'} {monthlyResult.toJuli.limit}
            </span>
          </div>
        </div>
      )}

      <EnergyBreakdownChart breakdown={result.breakdown} />

      {/* Monthly breakdown chart */}
      {monthlyResult && (
        <MonthlyBreakdownChart monthly={monthlyResult.monthly} />
      )}
    </div>
  );
}
