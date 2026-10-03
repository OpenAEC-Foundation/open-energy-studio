import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import type { IBENGResultMonthly } from '../../core/energy/types';
import { useMemo, useState } from 'react';
import {
  downloadNtaCalculationReportHTML, downloadNtaInputDossierHTML, downloadProjectDossier,
} from '../../core/report/ReportGenerator';
import { checkDossierCompleteness, type DossierItem } from '../../core/report/ProjectDossier';
import { useProjectPerformance } from '../../core/nta/useProjectPerformance';
import { kernelReportModel, type KernelReportModel } from '../../core/report/KernelReportModel';
import { formatNumber } from '../../i18n/format';
import './ReportView.css';

/** Month name in the UI language. */
function monthName(index: number, locale: string): string {
  return new Date(2026, index, 1).toLocaleDateString(locale, { month: 'short' });
}

/** Energy-balance rows of the kernel report; a null delivered value means the per-service energy is missing. */
function balanceRows(model: KernelReportModel): Array<[string, number | null]> {
  const bd = model.breakdown;
  const delivered = (value: number) => (bd.deliveredUnavailable ? null : value);
  return [
    ['results.transmissionLoss', bd.transmissionLoss],
    ['results.ventilationInfiltrationLoss', bd.ventilationLoss],
    ['results.solarGain', bd.solarGain],
    ['results.internalGain', bd.internalGain],
    ...(bd.otherGain ? [['results.otherGain', bd.otherGain] as [string, number]] : []),
    ['results.heatingDemand', bd.heatingDemand],
    ['results.coolingDemand', bd.coolingDemand],
    ['results.heatingEnergy', delivered(bd.heatingEnergy)],
    ['results.coolingEnergy', delivered(bd.coolingEnergy)],
    ['results.ventilationEnergy', delivered(bd.ventilationEnergy)],
    ['results.hotWaterEnergy', delivered(bd.hotWaterEnergy)],
    ['results.lightingEnergy', delivered(bd.lightingEnergy)],
    ['results.auxiliaryEnergy', delivered(bd.auxiliaryEnergy)],
    ['results.pvProduction', bd.pvProduction],
    ['results.solarThermalProduction', bd.solarThermalProduction],
  ];
}

export function ReportView() {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const { project, result } = state;
  const kernelQuery = useProjectPerformance(project);
  const kernelPending = kernelQuery == null || kernelQuery.kind === 'loading';
  const model = kernelReportModel(kernelQuery?.kind === 'done' ? kernelQuery.assessment : null);
  const meetsText = (meets: boolean | null) =>
    t(meets == null ? 'report.notTestable' : meets ? 'report.meetsUnverified' : 'report.fails');
  const [calculationError, setCalculationError] = useState<string | null>(null);
  const [exported, setExported] = useState<{ checklist: DossierItem[]; missingEvidence: number } | null>(null);
  const [dossierBusy, setDossierBusy] = useState(false);
  const exportCalculation = () => {
    setCalculationError(null);
    downloadNtaCalculationReportHTML(project).catch((reason: unknown) =>
      setCalculationError(reason instanceof Error ? reason.message : String(reason)));
  };
  const exportDossier = () => {
    setCalculationError(null);
    setDossierBusy(true);
    downloadProjectDossier(project)
      .then((manifest) => setExported({ checklist: manifest.checklist, missingEvidence: manifest.missingEvidence.length }))
      .catch((reason: unknown) => setCalculationError(reason instanceof Error ? reason.message : String(reason)))
      .finally(() => setDossierBusy(false));
  };
  // Live check without kernel output; the export repeats it with the output.
  const checklist = useMemo(() => exported?.checklist ?? checkDossierCompleteness({ project }), [exported, project]);
  const open = checklist.filter((item) => item.status === 'missing' || item.status === 'check');

  return (
    <div className="report-view">
      <div className="report-content">
        <div className="report-header">
          <h1>{t('report.title')}</h1>
          <p className="report-date">{new Date().toLocaleDateString(locale)}</p>
        </div>
        <div className="report-verification-notice" role="status">
          <strong>{model ? t('report.kernelStatus') : result ? t('results.indicative') : t('results.noResults')}</strong>
          <p>{model ? t('report.kernelSource') : result ? t('results.indicativeDescription') : t('report.inputDossierScope')}</p>
        </div>
        <div className="report-input-dossier">
          <button type="button" onClick={() => downloadNtaInputDossierHTML(project)}>{t('report.exportInputDossier')}</button>
          <p>{t('report.inputDossierScope')}</p>
          <button type="button" onClick={exportCalculation}>{t('report.exportNtaCalculation')}</button>
          <p>{t('report.ntaCalculationScope')}</p>
          <button type="button" onClick={exportDossier} disabled={dossierBusy}>{t('report.exportProjectDossier')}</button>
          <p>{t('report.projectDossierScope')}</p>
          {calculationError && <p role="alert">{calculationError}</p>}
          {exported && exported.missingEvidence > 0 && (
            <p role="status">{t('report.dossierMissingEvidence', { count: exported.missingEvidence })}</p>
          )}
          <details className="report-dossier-checklist">
            <summary>{t('report.dossierChecklist', { open: open.length, total: checklist.length })}</summary>
            <table className="report-table">
              <tbody>
                {checklist.map((item) => (
                  <tr key={item.id} data-status={item.status}>
                    <td>{t(`report.dossierStatus.${item.status}`)}</td>
                    <td>{item.label}</td>
                    <td>{item.detail ?? ''}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </details>
        </div>

        {/* Project info */}
        <div className="report-section">
          <h2>{t('report.projectInfo')}</h2>
          <table className="report-table">
            <tbody>
              <tr><td>{t('dialog.projectInfo.name')}</td><td>{project.name || '-'}</td></tr>
              <tr><td>{t('dialog.projectInfo.function')}</td><td>{t('function.' + project.buildingFunction)}</td></tr>
              <tr><td>{t('dialog.projectInfo.address')}</td><td>{project.address || '-'}</td></tr>
              <tr><td>{t('dialog.projectInfo.city')}</td><td>{project.city || '-'}</td></tr>
            </tbody>
          </table>
        </div>

        {/* Building envelope */}
        <div className="report-section">
          <h2>{t('report.buildingEnvelope')}</h2>
          {project.zones.map(zone => (
            <div key={zone.id} className="report-subsection">
              <h3>{zone.name} — {formatNumber(zone.floorArea, locale, 1)} m², {formatNumber(zone.volume, locale, 1)} m³</h3>
              {zone.surfaces.length > 0 && (
                <table className="report-table">
                  <thead>
                    <tr>
                      <th>{t('properties.name')}</th>
                      <th>{t('properties.type')}</th>
                      <th>{t('properties.area')}</th>
                      <th>{t('properties.orientation')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    {zone.surfaces.map(s => (
                      <tr key={s.id}>
                        <td>{s.name}</td>
                        <td>{t(`surfaceType.${s.type}`)}</td>
                        <td>{formatNumber(s.area, locale, 1)} m²</td>
                        <td>{t(`orientation.${s.orientation}`)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          ))}
        </div>

        {/* Installations */}
        <div className="report-section">
          <h2>{t('report.installationSystems')}</h2>
          {project.heatingSystems.length > 0 && (
            <div className="report-subsection">
              <h3>{t('browser.heating')}</h3>
              {project.heatingSystems.map(h => (
                <p key={h.id}>{h.name} — {t(`report.heatingType.${h.type}`, { defaultValue: h.type })}, {h.type.startsWith('heat_pump')
                  ? `COP ${formatNumber(h.cop, locale, 2)}` : `η ${formatNumber(h.cop, locale, 3)}`}</p>
              ))}
            </div>
          )}
          {project.ventilationSystems.length > 0 && (
            <div className="report-subsection">
              <h3>{t('browser.ventilation')}</h3>
              {project.ventilationSystems.map(v => (
                <p key={v.id}>{v.name} — {t(`report.ventilationType.${v.type}`, { defaultValue: v.type })}, {t('report.heatRecovery')}: {formatNumber(v.heatRecoveryEfficiency * 100, locale)}%</p>
              ))}
            </div>
          )}
        </div>

        {/* Renewables */}
        <div className="report-section">
          <h2>{t('report.renewableEnergy')}</h2>
          {model?.pvSystems.map((pv) => (
            <p key={`kernel-${pv.id}`}>{pv.id} — {formatNumber(pv.annualKwh, locale)} kWh/{t('report.year')} ({t('report.kernelPv')})</p>
          ))}
          {project.solarPV.map(pv => (
            <p key={pv.id}>{pv.name} — {pv.peakPower} kWp, {t(`orientation.${pv.orientation}`)}, {pv.tilt}°</p>
          ))}
          {project.solarThermal.map(st => (
            <p key={st.id}>{st.name} — {st.collectorArea} m², {st.type}</p>
          ))}
        </div>

        {/* BENG results: the NTA kernel, the simplified engine only without a kernel result. */}
        {model && (
          <>
            <div className="report-section" data-testid="report-beng-kernel">
              <h2>{t('report.bengResults')}</h2>
              <p className="report-source">{t('report.kernelSource')}</p>
              <table className="report-table report-table-results">
                <thead>
                  <tr>
                    <th>{t('report.indicator')}</th>
                    <th>{t('report.value')}</th>
                    <th>{t('results.limit')}</th>
                    <th>{t('report.status')}</th>
                  </tr>
                </thead>
                <tbody>
                  {model.indicators.map((row) => (
                    <tr key={row.key}>
                      <td>{t(`results.${row.key}.title`)} — {t(`results.${row.key}.subtitle`)}</td>
                      <td>{formatNumber(row.value, locale, row.key === 'beng3' ? 1 : 2)} {t(`results.${row.key}.unit`)}</td>
                      <td>{row.limit == null ? '–' : `${row.higherIsBetter ? '≥' : '≤'} ${formatNumber(row.limit, locale, row.key === 'beng3' ? 0 : 1)}`}</td>
                      <td>{meetsText(row.meets)}</td>
                    </tr>
                  ))}
                  {model.tojuli && (
                    <tr>
                      <td>{t('results.toJuli')}</td>
                      <td>{formatNumber(model.tojuli.value, locale, 2)} K</td>
                      <td>{'≤'} {formatNumber(1.2, locale, 2)} K</td>
                      <td>{meetsText(model.tojuli.meets)}</td>
                    </tr>
                  )}
                  <tr>
                    <td>{t('report.labelClass')}</td>
                    <td>{model.labelClass ?? '–'}</td>
                    <td></td>
                    <td>{t('report.notRegistered')}</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <div className="report-section">
              <h2>{t('report.monthlyOverview')}</h2>
              <table className="report-table report-table-results">
                <thead>
                  <tr>
                    <th>{t('preview.month')}</th>
                    <th>{t('results.heatingDemand')}</th>
                    <th>{t('results.coolingDemand')}</th>
                    <th>{t('results.solarGain')}</th>
                    <th>{t('results.transmissionLoss')}</th>
                  </tr>
                </thead>
                <tbody>
                  {model.monthly.map((m) => (
                    <tr key={m.month}>
                      <td>{monthName(m.month - 1, locale)}</td>
                      <td>{formatNumber(m.heatingNeedKwh, locale)} kWh</td>
                      <td>{formatNumber(m.coolingNeedKwh, locale)} kWh</td>
                      <td>{formatNumber(m.solarGainKwh, locale)} kWh</td>
                      <td>{formatNumber(m.transmissionKwh, locale)} kWh</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div className="report-section">
              <h2>{t('results.breakdown')}</h2>
              <table className="report-table">
                <tbody>
                  {balanceRows(model).map(([key, value]) => (
                    <tr key={key}><td>{t(key)}</td><td>{value == null ? t('results.breakdownDeliveredUnavailable') : `${formatNumber(value, locale)} kWh`}</td></tr>
                  ))}
                </tbody>
              </table>
            </div>
          </>
        )}

        {!model && !kernelPending && result && (() => {
          const monthlyResult = 'monthly' in result ? result as IBENGResultMonthly : null;
          return (
            <>
              <div className="report-section" data-testid="report-beng-indicative">
                <h2>{t('report.bengResults')}</h2>
                <p className="report-indicative-note">{t('report.indicativeNotNta')}</p>
                <table className="report-table report-table-results">
                  <thead>
                    <tr>
                      <th>{t('report.indicator')}</th>
                      <th>{t('report.value')}</th>
                      <th>{t('results.limit')}</th>
                      <th>{t('report.status')}</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr>
                      <td>{t('results.beng1.title')} — {t('results.beng1.subtitle')}</td>
                      <td>{formatNumber(result.beng1, locale, 1)} {t('results.beng1.unit')}</td>
                      <td>{'≤'} {formatNumber(result.beng1Limit, locale)}</td>
                      <td className="report-indicative">{t('results.indicativeBadge')}</td>
                    </tr>
                    <tr>
                      <td>{t('results.beng2.title')} — {t('results.beng2.subtitle')}</td>
                      <td>{formatNumber(result.beng2, locale, 1)} {t('results.beng2.unit')}</td>
                      <td>{'≤'} {formatNumber(result.beng2Limit, locale)}</td>
                      <td className="report-indicative">{t('results.indicativeBadge')}</td>
                    </tr>
                    <tr>
                      <td>{t('results.beng3.title')} — {t('results.beng3.subtitle')}</td>
                      <td>{formatNumber(result.beng3, locale, 1)} {t('results.beng3.unit')}</td>
                      <td>{'≥'} {formatNumber(result.beng3Limit, locale)}%</td>
                      <td className="report-indicative">{t('results.indicativeBadge')}</td>
                    </tr>
                    {monthlyResult && (
                      <tr>
                        <td>{t('results.toJuli')} — GTO</td>
                        <td>{formatNumber(monthlyResult.toJuli.gto, locale, 2)}</td>
                        <td>{'≤'} {formatNumber(monthlyResult.toJuli.limit, locale)}</td>
                        <td className="report-indicative">{t('results.indicativeBadge')}</td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>
              {monthlyResult && (
                <div className="report-section">
                  <h2>{t('report.monthlyOverview')}</h2>
                  <table className="report-table report-table-results">
                    <thead>
                      <tr>
                        <th>{t('preview.month')}</th>
                        <th>{t('results.heatingDemand')}</th>
                        <th>{t('results.coolingDemand')}</th>
                        <th>{t('results.solarGain')}</th>
                        <th>{t('results.transmissionLoss')}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {monthlyResult.monthly.map((m, i) => (
                        <tr key={i}>
                          <td>{monthName(i, locale)}</td>
                          <td>{formatNumber(m.heatingDemand, locale)} kWh</td>
                          <td>{formatNumber(m.coolingDemand, locale)} kWh</td>
                          <td>{formatNumber(m.solarGain, locale)} kWh</td>
                          <td>{formatNumber(m.transmissionLoss, locale)} kWh</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </>
          );
        })()}
      </div>
    </div>
  );
}
