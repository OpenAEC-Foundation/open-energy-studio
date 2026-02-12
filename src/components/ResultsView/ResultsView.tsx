import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { BENGIndicator } from '../BENGIndicator/BENGIndicator';
import { EnergyBreakdownChart } from '../EnergyBreakdownChart/EnergyBreakdownChart';
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

      <EnergyBreakdownChart breakdown={result.breakdown} />
    </div>
  );
}
