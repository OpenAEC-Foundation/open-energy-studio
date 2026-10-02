import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { BENGIndicator } from '../BENGIndicator/BENGIndicator';
import { EnergyBreakdownChart } from '../EnergyBreakdownChart/EnergyBreakdownChart';
import { MonthlyBreakdownChart } from '../MonthlyBreakdownChart/MonthlyBreakdownChart';
import { CalculationNotice } from '../CalculationNotice/CalculationNotice';
import { KernelAuditPanel } from '../KernelAuditPanel/KernelAuditPanel';
import { NtaPerformancePanel } from '../NtaPerformancePanel/NtaPerformancePanel';
import { MaatwerkadviesPanel } from '../MaatwerkadviesPanel/MaatwerkadviesPanel';
import { RelabelPanel } from '../MaatwerkadviesPanel/RelabelPanel';
import type { IBENGResultMonthly } from '../../core/energy/types';
import './ResultsView.css';

export function ResultsView() {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { result, project } = state;

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
  const surfaceCount = project.zones.reduce((count, zone) => count + zone.surfaces.length, 0);
  const systemCount = project.heatingSystems.length + project.ventilationSystems.length
    + project.coolingSystems.length + project.hotWaterSystems.length;

  return (
    <div className="results-view">
      <div className="results-heading">
        <div>
          <span className="results-eyebrow">{t('results.calculationOverview')}</span>
          <h2>{project.name || t('results.title')}</h2>
          <p>{t('results.inputSummary')}</p>
        </div>
        <button
          type="button"
          className="results-review-button"
          onClick={() => {
            dispatch({ type: 'SET_VIEW_MODE', payload: 'project' });
            dispatch({ type: 'SET_RIBBON_TAB', payload: 'start' });
          }}
        >
          {t('results.reviewInput')}
        </button>
      </div>

      <div className="results-input-summary" aria-label={t('results.inputSummary')}>
        <div><strong>{project.zones.length}</strong><span>{t('results.zones')}</span></div>
        <div><strong>{surfaceCount}</strong><span>{t('results.surfaces')}</span></div>
        <div><strong>{systemCount}</strong><span>{t('results.systems')}</span></div>
        <div><strong>{result.totalFloorArea.toFixed(1)}</strong><span>m² {t('results.floorArea')}</span></div>
      </div>

      <CalculationNotice />
      <KernelAuditPanel project={project} />
      <NtaPerformancePanel />
      <MaatwerkadviesPanel />
      <RelabelPanel />

      <div className="beng-cards">
        <BENGIndicator
          title={t('results.beng1.title')}
          subtitle={t('results.beng1.subtitle')}
          value={result.beng1}
          limit={result.beng1Limit}
          unit={t('results.beng1.unit')}
        />
        <BENGIndicator
          title={t('results.beng2.title')}
          subtitle={t('results.beng2.subtitle')}
          value={result.beng2}
          limit={result.beng2Limit}
          unit={t('results.beng2.unit')}
        />
        <BENGIndicator
          title={t('results.beng3.title')}
          subtitle={t('results.beng3.subtitle')}
          value={result.beng3}
          limit={result.beng3Limit}
          unit={t('results.beng3.unit')}
          higherIsBetter
        />
      </div>

      {/* TO-juli indicator */}
      {monthlyResult && (
        <div className="to-juli-card to-juli-indicative">
          <div className="to-juli-header">
            <h3>{t('results.toJuli')}</h3>
            <span className="to-juli-badge indicative">{t('results.indicativeBadge')}</span>
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
