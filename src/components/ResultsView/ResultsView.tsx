import { useRef } from 'react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import { BENGIndicator } from '../BENGIndicator/BENGIndicator';
import { EnergyBreakdownChart } from '../EnergyBreakdownChart/EnergyBreakdownChart';
import { MonthlyBreakdownChart } from '../MonthlyBreakdownChart/MonthlyBreakdownChart';
import { CalculationNotice } from '../CalculationNotice/CalculationNotice';
import { KernelAuditPanel } from '../KernelAuditPanel/KernelAuditPanel';
import { NtaPerformancePanel } from '../NtaPerformancePanel/NtaPerformancePanel';
import { MaatwerkadviesPanel } from '../MaatwerkadviesPanel/MaatwerkadviesPanel';
import { RelabelPanel } from '../MaatwerkadviesPanel/RelabelPanel';
import { useProjectPerformance, calculatedAssessment } from '../../core/nta/useProjectPerformance';
import { kernelEnergyBreakdown } from '../../core/nta/KernelBreakdown';
import type { IBENGResult, IBENGResultMonthly } from '../../core/energy/types';
import './ResultsView.css';

export function ResultsView() {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const { result, project } = state;

  // A project edit clears `result`; the last result of this project stays visible,
  // marked stale, so the panels below keep their place (and their own state).
  const lastResult = useRef<{ projectId: string; result: IBENGResult } | null>(null);
  if (result) lastResult.current = { projectId: project.id, result };
  else if (lastResult.current?.projectId !== project.id) lastResult.current = null;
  const shown = result ?? lastResult.current?.result ?? null;
  const stale = result == null && shown != null;

  // One kernel run feeds both the NTA panel and the cards, so they cannot disagree.
  const kernelQuery = useProjectPerformance(project);
  const kernel = calculatedAssessment(kernelQuery);
  const performance = kernel?.performance ?? null;
  const kernelPending = kernelQuery == null || kernelQuery.kind === 'loading';

  const monthlyResult = shown && 'monthly' in shown ? shown as IBENGResultMonthly : null;
  const surfaceCount = project.zones.reduce((count, zone) => count + zone.surfaces.length, 0);
  const systemCount = project.heatingSystems.length + project.ventilationSystems.length
    + project.coolingSystems.length + project.hotWaterSystems.length;
  const kernelFloorArea = kernel?.geometry?.usableFloorAreaM2 ?? null;
  const floorArea = kernelFloorArea ?? shown?.totalFloorArea ?? null;
  // Whether the last settled kernel run of this project had a result: while the next (debounced)
  // run is pending, the simplified output stays only if the kernel had none before.
  const settled = useRef<{ projectId: string; calculated: boolean } | null>(null);
  if (kernelQuery && kernelQuery.kind !== 'loading') settled.current = { projectId: project.id, calculated: kernel != null };
  const kernelHadNoResult = settled.current?.projectId === project.id && !settled.current.calculated;
  // Output of the simplified engine: only without a kernel result, and marked stale after an edit.
  // A kernel that refused the input (invalid or incomplete) withholds every number: the
  // simplified engine must not fill that gap with values the kernel would not stand behind.
  // A project without any NTA input yet (only `nta_calculation_block_missing`) has no kernel
  // verdict on its numbers; there the indicative engine may still show its estimate.
  const kernelDone = kernelQuery?.kind === 'done' ? kernelQuery.assessment : null;
  const kernelStatus = kernelDone?.status ?? null;
  const noNtaInputYet = kernelStatus === 'incomplete'
    && (kernelDone?.gaps ?? []).every((gap) => gap.code === 'nta_calculation_block_missing');
  const withheld = kernelStatus != null && kernel == null && kernelStatus !== 'calculated_unverified' && !noNtaInputYet;
  const indicative = !withheld && !performance && shown != null && (!kernelPending || kernelHadNoResult);
  const bbl = performance?.bblCheck ?? null;
  const utility = kernel?.derivedInput?.calculationScope === 'utility';

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
        <div className={kernelFloorArea == null && stale ? 'results-stale-value' : undefined}
          title={kernelFloorArea == null && stale ? t('results.staleShort') : undefined}>
          <strong>{formatNumber(floorArea, locale, 1)}</strong><span>m² {t('results.floorArea')}</span></div>
      </div>

      {!shown && !performance && <div className="results-empty">
        <p>{t('results.noResults')}</p>
      </div>}

      <CalculationNotice />
      <KernelAuditPanel project={project} />
      <NtaPerformancePanel query={kernelQuery} />
      <MaatwerkadviesPanel />
      <RelabelPanel />

      {performance && <div className="beng-cards" data-testid="beng-cards-kernel">
        {performance.needIndicatorKwhPerM2Year != null && <BENGIndicator
          title={t('results.beng1.title')}
          subtitle={t('results.beng1.subtitle')}
          value={performance.needIndicatorKwhPerM2Year}
          limit={bbl?.limits.energyNeedMaxKwhPerM2 ?? null}
          unit={t('results.beng1.unit')}
          source="kernel"
          digits={2}
        />}
        {performance.primaryFossilIndicatorKwhPerM2Year != null && <BENGIndicator
          title={t('results.beng2.title')}
          subtitle={t('results.beng2.subtitle')}
          value={performance.primaryFossilIndicatorKwhPerM2Year}
          limit={bbl?.limits.primaryFossilMaxKwhPerM2 ?? null}
          unit={t('results.beng2.unit')}
          source="kernel"
          digits={2}
        />}
        {performance.renewableSharePercent != null && <BENGIndicator
          title={t('results.beng3.title')}
          subtitle={t('results.beng3.subtitle')}
          value={performance.renewableSharePercent}
          limit={bbl?.limits.renewableShareMinPercent ?? null}
          unit={t('results.beng3.unit')}
          higherIsBetter
          source="kernel"
        />}
      </div>}

      {/* TO-juli: the kernel value (dwellings only), otherwise the indicative GTO. */}
      {performance && !utility && performance.tojuliMaxK != null && (
        <div className="to-juli-card" data-testid="to-juli-kernel">
          <div className="to-juli-header">
            <h3>{t('results.toJuli')}</h3>
            <span className="to-juli-badge">{t('results.kernelBadge')}</span>
          </div>
          <div className="to-juli-value">
            TO<sub>juli</sub>: {formatNumber(performance.tojuliMaxK, locale, 2)} K
            <span className="to-juli-limit">
              {' '}/ {t('results.limit')}: {'≤'} {formatNumber(1.2, locale, 2)} K
            </span>
          </div>
        </div>
      )}
      {performance && <EnergyBreakdownChart breakdown={kernelEnergyBreakdown(performance)} source="kernel" />}

      {withheld && <div className="results-withheld" role="status" data-testid="results-withheld">
        <strong>{t('results.withheld.title')}</strong>
        <p>{t(kernelStatus === 'incomplete' ? 'results.withheld.incomplete' : 'results.withheld.invalid')}</p>
      </div>}

      {/* The simplified engine only when the kernel has no result; never next to kernel values. */}
      {indicative && shown && <div className={stale ? 'results-indicative results-stale-block' : 'results-indicative'}
        data-testid="results-indicative">
        {stale && <p className="results-stale" role="status" data-testid="results-stale">{t('results.stale')}</p>}
        <p className="results-indicative-note">{t('results.indicativeNote')}</p>
        <div className="beng-cards" data-testid="beng-cards-indicative">
          <BENGIndicator
            title={t('results.beng1.title')}
            subtitle={t('results.beng1.subtitle')}
            value={shown.beng1}
            limit={shown.beng1Limit}
            unit={t('results.beng1.unit')}
          />
          <BENGIndicator
            title={t('results.beng2.title')}
            subtitle={t('results.beng2.subtitle')}
            value={shown.beng2}
            limit={shown.beng2Limit}
            unit={t('results.beng2.unit')}
          />
          <BENGIndicator
            title={t('results.beng3.title')}
            subtitle={t('results.beng3.subtitle')}
            value={shown.beng3}
            limit={shown.beng3Limit}
            unit={t('results.beng3.unit')}
            higherIsBetter
          />
        </div>
        {monthlyResult && (
          <div className="to-juli-card to-juli-indicative">
            <div className="to-juli-header">
              <h3>{t('results.toJuli')}</h3>
              <span className="to-juli-badge indicative">{t('results.indicativeBadge')}</span>
            </div>
            <div className="to-juli-value">
              GTO: {formatNumber(monthlyResult.toJuli.gto, locale, 2)}
              <span className="to-juli-limit">
                {' '}/ {t('results.limit')}: {'≤'} {formatNumber(monthlyResult.toJuli.limit, locale, 2)}
              </span>
            </div>
          </div>
        )}
        <EnergyBreakdownChart breakdown={shown.breakdown} />
        {monthlyResult && <MonthlyBreakdownChart monthly={monthlyResult.monthly} />}
      </div>}
    </div>
  );
}
