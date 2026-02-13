import { useMemo } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { calculateBENGMonthly } from '../../core/energy/BENGCalculatorMonthly';
import { calculateEnergyLabel } from '../../core/energy/EnergyLabel';
import { BENGIndicatorCompact } from './BENGIndicatorCompact';
import { MonthlyBarChart } from './MonthlyBarChart';
import './PreviewPanel.css';

export function PreviewPanel() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { project } = state;

  // Auto-recalculate when project changes
  const result = useMemo(() => {
    const hasZones = project.zones.length > 0;
    if (!hasZones) return null;
    return calculateBENGMonthly(project);
  }, [project]);

  return (
    <div className="preview-panel">
      <div className="preview-panel-header">
        <span>{t('preview.title')}</span>
      </div>
      <div className="preview-panel-content">
        {!result ? (
          <div className="preview-empty">{t('preview.noData')}</div>
        ) : (
          <>
            {/* Energy label */}
            {(() => {
              const labelResult = calculateEnergyLabel(result.beng2);
              return (
                <div className="preview-energy-label">
                  <div className="preview-energy-label-arrow" style={{ backgroundColor: labelResult.color }}>
                    <span className="preview-energy-label-text">{labelResult.label}</span>
                  </div>
                </div>
              );
            })()}

            {/* BENG indicators */}
            <div className="preview-section-title">BENG</div>
            <BENGIndicatorCompact
              label={`${t('results.beng1.title')} \u2014 ${t('results.beng1.subtitle')}`}
              value={result.beng1}
              limit={result.beng1Limit}
              unit={t('results.beng1.unit')}
              pass={result.beng1Pass}
            />
            <BENGIndicatorCompact
              label={`${t('results.beng2.title')} \u2014 ${t('results.beng2.subtitle')}`}
              value={result.beng2}
              limit={result.beng2Limit}
              unit={t('results.beng2.unit')}
              pass={result.beng2Pass}
            />
            <BENGIndicatorCompact
              label={`${t('results.beng3.title')} \u2014 ${t('results.beng3.subtitle')}`}
              value={result.beng3}
              limit={result.beng3Limit}
              unit={t('results.beng3.unit')}
              pass={result.beng3Pass}
              higherIsBetter
            />

            {/* TO-juli */}
            <div className={`preview-to-juli ${result.toJuli.pass ? 'pass' : 'fail'}`}>
              <div className="preview-to-juli-header">
                <span className="preview-to-juli-label">{t('preview.toJuli')}</span>
                <span className={`preview-beng-badge ${result.toJuli.pass ? 'pass' : 'fail'}`}>
                  {result.toJuli.pass ? t('results.pass') : t('results.fail')}
                </span>
              </div>
              <div className="preview-to-juli-value">
                GTO: {result.toJuli.gto.toFixed(2)} / {result.toJuli.limit}
              </div>
            </div>

            {/* Monthly chart */}
            <div className="preview-section-title">{t('preview.monthlyDemand')}</div>
            <MonthlyBarChart monthly={result.monthly} />

            {/* Key figures */}
            <div className="preview-section-title">{t('preview.keyFigures')}</div>
            <div className="preview-key-figures">
              <div className="preview-key-row">
                <span className="preview-key-label">{t('results.transmissionLoss')}</span>
                <span className="preview-key-value">{result.breakdown.transmissionLoss.toFixed(0)} kWh</span>
              </div>
              <div className="preview-key-row">
                <span className="preview-key-label">{t('results.ventilationLoss')}</span>
                <span className="preview-key-value">{result.breakdown.ventilationLoss.toFixed(0)} kWh</span>
              </div>
              <div className="preview-key-row">
                <span className="preview-key-label">{t('results.solarGain')}</span>
                <span className="preview-key-value">{result.breakdown.solarGain.toFixed(0)} kWh</span>
              </div>
              <div className="preview-key-row">
                <span className="preview-key-label">{t('results.heatingDemand')}</span>
                <span className="preview-key-value">{result.breakdown.heatingDemand.toFixed(0)} kWh</span>
              </div>
              <div className="preview-key-row">
                <span className="preview-key-label">{t('results.coolingDemand')}</span>
                <span className="preview-key-value">{result.breakdown.coolingDemand.toFixed(0)} kWh</span>
              </div>
              <div className="preview-key-row">
                <span className="preview-key-label">{t('results.pvProduction')}</span>
                <span className="preview-key-value">{result.breakdown.pvProduction.toFixed(0)} kWh</span>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
