import { useState, useRef, useCallback, useEffect } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { hasUnmodelledHeatPumpDetails, hasUnmodelledUnheatedTransmission, legacyHeatPumpInputIssue, validProjectFloorArea } from '../../core/energy/ProjectArea';
import { calculateProjectPerformanceShared } from '../../core/nta/useProjectPerformance';
import { summarizeForPreview, type PreviewSummary } from '../../core/nta/PreviewSummary';
import type { IProject } from '../../core/energy/types';
import { BENGIndicatorCompact } from './BENGIndicatorCompact';
import { MonthlyBarChart } from './MonthlyBarChart';
import { CalculationNotice } from '../CalculationNotice/CalculationNotice';
import { formatNumber } from '../../i18n/format';
import { PanelRightClose } from 'lucide-react';
import './PreviewPanel.css';

/** Delay before a project change triggers a kernel run, ms. */
const KERNEL_DEBOUNCE_MS = 400;

type KernelState =
  | { kind: 'idle' }
  | { kind: 'loading'; project: IProject }
  | { kind: 'error'; project: IProject; message: string }
  | { kind: 'done'; project: IProject; summary: PreviewSummary };


/** `embedded`: rendered inside the shell inspector, which owns the frame, header and width. */
export function PreviewPanel({ embedded = false }: { embedded?: boolean } = {}) {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const kwh = (value: number | null) => (value == null ? '–' : `${formatNumber(value, locale)} kWh`);
  const { project } = state;
  const invalidArea = project.zones.length > 0 && validProjectFloorArea(project) === null;
  const hasStandaloneHeatPumps = Boolean(project.ntaHeatPumps?.length);
  const hasPerformancePoints = hasUnmodelledHeatPumpDetails(project);
  const hasUnheatedTransmission = hasUnmodelledUnheatedTransmission(project);
  const heatPumpIssue = legacyHeatPumpInputIssue(project);
  const inputAlert = invalidArea ? 'calculation.invalidFloorArea'
    : hasStandaloneHeatPumps ? 'calculation.standaloneHeatPumps'
      : hasPerformancePoints ? 'calculation.performancePointsUnsupported'
        : hasUnheatedTransmission ? 'calculation.unheatedUnsupported'
          : heatPumpIssue === 'cop' ? 'calculation.invalidHeatPumpCop'
            : heatPumpIssue === 'coverage' ? 'calculation.invalidHeatPumpCoverage'
              : null;
  const hasNtaBlock = Boolean(project.ntaCalculation);

  // Single source of results: the Rust kernel (project performance route).
  const [kernel, setKernel] = useState<KernelState>({ kind: 'idle' });
  useEffect(() => {
    if (!hasNtaBlock || project.zones.length === 0) {
      setKernel({ kind: 'idle' });
      return;
    }
    let cancelled = false;
    setKernel({ kind: 'loading', project });
    const timer = setTimeout(() => {
      calculateProjectPerformanceShared(project).then(
        (assessment) => { if (!cancelled) setKernel({ kind: 'done', project, summary: summarizeForPreview(assessment) }); },
        (reason: unknown) => {
          if (!cancelled) setKernel({ kind: 'error', project, message: reason instanceof Error ? reason.message : String(reason) });
        },
      );
    }, KERNEL_DEBOUNCE_MS);
    return () => { cancelled = true; clearTimeout(timer); };
  }, [project, hasNtaBlock]);

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

  if (collapsed && !embedded) {
    return (
      <div className="preview-panel collapsed" onClick={() => setCollapsed(false)}>
        <div className="panel-collapsed-label-right">
          <span>{t('preview.title')}</span>
        </div>
      </div>
    );
  }

  // An earlier project's result must never render during the first frame of a switch.
  const visibleKernel: KernelState = kernel.kind !== 'idle' && kernel.project !== project
    ? hasNtaBlock ? { kind: 'loading', project } : { kind: 'idle' }
    : kernel;
  const summary = visibleKernel.kind === 'done' ? visibleKernel.summary : null;
  const calculated = summary?.status === 'calculated_unverified';

  return (
    <div className={embedded ? 'preview-panel embedded' : 'preview-panel'} style={embedded ? undefined : { width }}>
      {!embedded && <>
        <div className="panel-resize-handle panel-resize-handle-left" onMouseDown={onResizeStart} />
        <div className="preview-panel-header">
          <button className="panel-collapse-btn-right" onClick={() => setCollapsed(true)}><PanelRightClose size={14} /></button>
          <span>{t('preview.title')}</span>
        </div>
      </>}
      <div className="preview-panel-content">
        {project.zones.length === 0 ? (
          <div className="preview-empty">{t('preview.noData')}</div>
        ) : (
          <>
            {inputAlert && <div className="preview-empty" role="alert">{t(inputAlert)}</div>}
            <CalculationNotice compact />
            {/* Energy label: only the Rust kernel classifies (one label source). */}
            <p className="preview-label-unavailable">{t('preview.labelFromKernel')}</p>

            {!hasNtaBlock && (
              <div className="preview-empty preview-kernel-empty">{t('preview.kernelEmpty')}</div>
            )}
            {hasNtaBlock && visibleKernel.kind === 'loading' && <p role="status" className="preview-kernel-status">{t('preview.kernelLoading')}</p>}
            {hasNtaBlock && visibleKernel.kind === 'error' && (
              <div className="preview-empty" role="alert">{t('kernel.unavailable')} <small>{visibleKernel.message}</small></div>
            )}
            {summary && !calculated && (
              <div className="preview-empty preview-kernel-empty" role="status">
                {summary.status === 'incomplete'
                  ? t('preview.kernelIncomplete').replace('{count}', String(summary.gapCount))
                  : t('preview.kernelInvalid').replace('{count}', String(summary.issueCount))}
              </div>
            )}

            {summary && calculated && <>
              {summary.labelClass && (
                <div className="preview-kernel-label" aria-label={t('preview.energyLabel')}>
                  <span>{t('preview.energyLabel')}</span>
                  <strong>{summary.labelClass}</strong>
                </div>
              )}
              <div className="preview-section-title">BENG</div>
              <BENGIndicatorCompact
                label={`${t('results.beng1.title')} — ${t('results.beng1.subtitle')}`}
                value={summary.beng1}
                limit={summary.beng1Limit}
                unit={t('results.beng1.unit')}
              />
              <BENGIndicatorCompact
                label={`${t('results.beng2.title')} — ${t('results.beng2.subtitle')}`}
                value={summary.beng2}
                limit={summary.beng2Limit}
                unit={t('results.beng2.unit')}
              />
              <BENGIndicatorCompact
                label={`${t('results.beng3.title')} — ${t('results.beng3.subtitle')}`}
                value={summary.beng3}
                limit={summary.beng3Limit}
                unit={t('results.beng3.unit')}
                higherIsBetter
              />

              {summary.tojuliApplies && <div className="preview-to-juli">
                <div className="preview-to-juli-header">
                  <span className="preview-to-juli-label">{t('preview.toJuliKernel')}</span>
                  <span className="preview-beng-badge">{t('nta.performance.unverified')}</span>
                </div>
                <div className="preview-to-juli-value">
                  {summary.tojuliMaxK == null ? '–' : `${formatNumber(summary.tojuliMaxK, locale, 2)} K`} / {formatNumber(1.2, locale, 2)} K
                </div>
              </div>}

              <div className="preview-section-title">{t('preview.monthlyDemand')}</div>
              <MonthlyBarChart heating={summary.monthlyHeatingKwh} cooling={summary.monthlyCoolingKwh} />

              <div className="preview-section-title">{t('preview.keyFigures')}</div>
              <div className="preview-key-figures">
                <div className="preview-key-row">
                  <span className="preview-key-label">{t('preview.zebIndicator')}</span>
                  <span className="preview-key-value">{summary.zebIndicator == null ? '–' : `${formatNumber(summary.zebIndicator, locale, 2)} kWh/m²`}</span>
                </div>
                <div className="preview-key-row">
                  <span className="preview-key-label">{t('preview.finalEnergy')}</span>
                  <span className="preview-key-value">{kwh(summary.finalEnergyKwh)}</span>
                </div>
                <div className="preview-key-row">
                  <span className="preview-key-label">{t('preview.co2')}</span>
                  <span className="preview-key-value">{summary.co2KgPerM2 == null ? '–' : `${formatNumber(summary.co2KgPerM2, locale, 1)} kg/m²`}</span>
                </div>
              </div>
            </>}
          </>
        )}
      </div>
    </div>
  );
}
