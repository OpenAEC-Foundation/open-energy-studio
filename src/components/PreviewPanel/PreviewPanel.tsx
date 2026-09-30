import { useState, useMemo, useRef, useCallback, useEffect } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { calculateBENGMonthly } from '../../core/energy/BENGCalculatorMonthly';
import { hasUnmodelledHeatPumpDetails, hasUnmodelledUnheatedTransmission, legacyHeatPumpInputIssue, validProjectFloorArea } from '../../core/energy/ProjectArea';
import { calculateEnergyLabel } from '../../core/energy/EnergyLabel';
import { BENGIndicatorCompact } from './BENGIndicatorCompact';
import { MonthlyBarChart } from './MonthlyBarChart';
import { CalculationNotice } from '../CalculationNotice/CalculationNotice';
import { PanelRightClose } from 'lucide-react';
import './PreviewPanel.css';

export function PreviewPanel() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { project } = state;
  const invalidArea = project.zones.length > 0 && validProjectFloorArea(project) === null;
  const hasStandaloneHeatPumps = Boolean(project.ntaHeatPumps?.length);
  const hasPerformancePoints = hasUnmodelledHeatPumpDetails(project);
  const hasUnheatedTransmission = hasUnmodelledUnheatedTransmission(project);
  const heatPumpIssue = legacyHeatPumpInputIssue(project);

  // Auto-recalculate when project changes
  const result = useMemo(() => {
    const hasZones = project.zones.length > 0;
    if (!hasZones || validProjectFloorArea(project) === null
      || project.ntaHeatPumps?.length || hasUnmodelledHeatPumpDetails(project)
      || hasUnmodelledUnheatedTransmission(project)
      || legacyHeatPumpInputIssue(project)) return null;
    return calculateBENGMonthly(project);
  }, [project]);

  const [collapsed, setCollapsed] = useState(false);
  const [width, setWidth] = useState(320);
  const resizing = useRef(false);
  const startX = useRef(0);
  const startWidth = useRef(0);

  const onResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    resizing.current = true;
    startX.current = e.clientX;
    startWidth.current = width;
  }, [width]);

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!resizing.current) return;
      const newWidth = Math.min(500, Math.max(160, startWidth.current - (e.clientX - startX.current)));
      setWidth(newWidth);
    };
    const onMouseUp = () => { resizing.current = false; };
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    return () => {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };
  }, []);

  if (collapsed) {
    return (
      <div className="preview-panel collapsed" onClick={() => setCollapsed(false)}>
        <div className="panel-collapsed-label-right">
          <span>{t('preview.title')}</span>
        </div>
      </div>
    );
  }

  return (
    <div className="preview-panel" style={{ width }}>
      <div className="panel-resize-handle panel-resize-handle-left" onMouseDown={onResizeStart} />
      <div className="preview-panel-header">
        <button className="panel-collapse-btn-right" onClick={() => setCollapsed(true)}><PanelRightClose size={14} /></button>
        <span>{t('preview.title')}</span>
      </div>
      <div className="preview-panel-content">
        {!result ? (
          <div className="preview-empty" role={invalidArea || hasStandaloneHeatPumps || hasPerformancePoints || hasUnheatedTransmission || heatPumpIssue ? 'alert' : undefined}>
            {t(hasStandaloneHeatPumps ? 'calculation.standaloneHeatPumps'
              : hasPerformancePoints ? 'calculation.performancePointsUnsupported'
                : hasUnheatedTransmission ? 'calculation.unheatedUnsupported'
                : invalidArea ? 'calculation.invalidFloorArea'
                : heatPumpIssue === 'cop' ? 'calculation.invalidHeatPumpCop'
                  : heatPumpIssue === 'coverage' ? 'calculation.invalidHeatPumpCoverage'
                    : 'preview.noData')}
          </div>
        ) : (
          <>
            <CalculationNotice compact />
            {/* Energy label */}
            {project.buildingFunction === 'residential' ? (() => {
              const labelResult = calculateEnergyLabel(result.beng2);
              return (
                <div className="preview-energy-label" aria-label={`${t('results.indicative')}: ${labelResult.label}`}>
                  <div className="preview-energy-label-arrow" style={{ backgroundColor: labelResult.color }}>
                    <span className="preview-energy-label-text">{labelResult.label}</span>
                  </div>
                </div>
              );
            })() : (
              <p className="preview-label-unavailable">{t('results.labelUnavailable')}</p>
            )}

            {/* BENG indicators */}
            <div className="preview-section-title">BENG</div>
            <BENGIndicatorCompact
              label={`${t('results.beng1.title')} \u2014 ${t('results.beng1.subtitle')}`}
              value={result.beng1}
              limit={result.beng1Limit}
              unit={t('results.beng1.unit')}
            />
            <BENGIndicatorCompact
              label={`${t('results.beng2.title')} \u2014 ${t('results.beng2.subtitle')}`}
              value={result.beng2}
              limit={result.beng2Limit}
              unit={t('results.beng2.unit')}
            />
            <BENGIndicatorCompact
              label={`${t('results.beng3.title')} \u2014 ${t('results.beng3.subtitle')}`}
              value={result.beng3}
              limit={result.beng3Limit}
              unit={t('results.beng3.unit')}
              higherIsBetter
            />

            {/* TO-juli */}
            <div className="preview-to-juli indicative">
              <div className="preview-to-juli-header">
                <span className="preview-to-juli-label">{t('preview.toJuli')}</span>
                <span className="preview-beng-badge indicative">{t('results.indicativeBadge')}</span>
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
