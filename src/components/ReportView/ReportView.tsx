import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import type { IBENGResultMonthly } from '../../core/energy/types';
import { useState } from 'react';
import { downloadNtaCalculationReportHTML, downloadNtaInputDossierHTML } from '../../core/report/ReportGenerator';
import './ReportView.css';

export function ReportView() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { project, result } = state;
  const [calculationError, setCalculationError] = useState<string | null>(null);
  const exportCalculation = () => {
    setCalculationError(null);
    downloadNtaCalculationReportHTML(project).catch((reason: unknown) =>
      setCalculationError(reason instanceof Error ? reason.message : String(reason)));
  };

  return (
    <div className="report-view">
      <div className="report-content">
        <div className="report-header">
          <h1>{t('report.title')}</h1>
          <p className="report-date">{new Date().toLocaleDateString('nl-NL')}</p>
        </div>
        <div className="report-verification-notice" role="status">
          <strong>{result ? t('results.indicative') : t('results.noResults')}</strong>
          <p>{result ? t('results.indicativeDescription') : t('report.inputDossierScope')}</p>
        </div>
        <div className="report-input-dossier">
          <button type="button" onClick={() => downloadNtaInputDossierHTML(project)}>{t('report.exportInputDossier')}</button>
          <p>{t('report.inputDossierScope')}</p>
          <button type="button" onClick={exportCalculation}>{t('report.exportNtaCalculation')}</button>
          <p>{t('report.ntaCalculationScope')}</p>
          {calculationError && <p role="alert">{calculationError}</p>}
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
              <h3>{zone.name} — {zone.floorArea} m², {zone.volume} m³</h3>
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
                        <td>{s.area} m²</td>
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
                <p key={h.id}>{h.name} — {h.type}, COP: {h.cop}</p>
              ))}
            </div>
          )}
          {project.ventilationSystems.length > 0 && (
            <div className="report-subsection">
              <h3>{t('browser.ventilation')}</h3>
              {project.ventilationSystems.map(v => (
                <p key={v.id}>{v.name} — {v.type}, WTW: {(v.heatRecoveryEfficiency * 100).toFixed(0)}%</p>
              ))}
            </div>
          )}
        </div>

        {/* Renewables */}
        <div className="report-section">
          <h2>{t('report.renewableEnergy')}</h2>
          {project.solarPV.map(pv => (
            <p key={pv.id}>{pv.name} — {pv.peakPower} kWp, {t(`orientation.${pv.orientation}`)}, {pv.tilt}°</p>
          ))}
          {project.solarThermal.map(st => (
            <p key={st.id}>{st.name} — {st.collectorArea} m², {st.type}</p>
          ))}
        </div>

        {/* BENG Results */}
        {result && (() => {
          const monthlyResult = 'monthly' in result ? result as IBENGResultMonthly : null;
          const monthKeys = [
            'month.jan', 'month.feb', 'month.mar', 'month.apr',
            'month.may', 'month.jun', 'month.jul', 'month.aug',
            'month.sep', 'month.oct', 'month.nov', 'month.dec',
          ];

          return (
            <>
              <div className="report-section">
                <h2>{t('report.bengResults')}</h2>
                <table className="report-table report-table-results">
                  <thead>
                    <tr>
                      <th>Indicator</th>
                      <th>{t('report.value')}</th>
                      <th>{t('results.limit')}</th>
                      <th>Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr>
                      <td>{t('results.beng1.title')} — {t('results.beng1.subtitle')}</td>
                      <td>{result.beng1.toFixed(1)} {t('results.beng1.unit')}</td>
                      <td>{'\u2264'} {result.beng1Limit}</td>
                      <td className="report-indicative">{t('results.indicativeBadge')}</td>
                    </tr>
                    <tr>
                      <td>{t('results.beng2.title')} — {t('results.beng2.subtitle')}</td>
                      <td>{result.beng2.toFixed(1)} {t('results.beng2.unit')}</td>
                      <td>{'\u2264'} {result.beng2Limit}</td>
                      <td className="report-indicative">{t('results.indicativeBadge')}</td>
                    </tr>
                    <tr>
                      <td>{t('results.beng3.title')} — {t('results.beng3.subtitle')}</td>
                      <td>{result.beng3.toFixed(1)} {t('results.beng3.unit')}</td>
                      <td>{'\u2265'} {result.beng3Limit}%</td>
                      <td className="report-indicative">{t('results.indicativeBadge')}</td>
                    </tr>
                    {monthlyResult && (
                      <tr>
                        <td>{t('results.toJuli')} — GTO</td>
                        <td>{monthlyResult.toJuli.gto.toFixed(2)}</td>
                        <td>{'\u2264'} {monthlyResult.toJuli.limit}</td>
                        <td className="report-indicative">{t('results.indicativeBadge')}</td>
                      </tr>
                    )}
                  </tbody>
                </table>
              </div>

              {/* Monthly overview */}
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
                          <td>{t(monthKeys[i])}</td>
                          <td>{m.heatingDemand.toFixed(0)} kWh</td>
                          <td>{m.coolingDemand.toFixed(0)} kWh</td>
                          <td>{m.solarGain.toFixed(0)} kWh</td>
                          <td>{m.transmissionLoss.toFixed(0)} kWh</td>
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
