import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import './ReportView.css';

export function ReportView() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { project, result } = state;

  return (
    <div className="report-view">
      <div className="report-content">
        <div className="report-header">
          <h1>{t('report.title')}</h1>
          <p className="report-date">{new Date().toLocaleDateString('nl-NL')}</p>
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
        {result && (
          <div className="report-section">
            <h2>{t('report.bengResults')}</h2>
            <table className="report-table report-table-results">
              <thead>
                <tr>
                  <th>Indicator</th>
                  <th>Waarde</th>
                  <th>Eis</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>{t('results.beng1.title')} — {t('results.beng1.subtitle')}</td>
                  <td>{result.beng1.toFixed(1)} {t('results.beng1.unit')}</td>
                  <td>≤ {result.beng1Limit}</td>
                  <td className={result.beng1Pass ? 'report-pass' : 'report-fail'}>
                    {result.beng1Pass ? t('results.pass') : t('results.fail')}
                  </td>
                </tr>
                <tr>
                  <td>{t('results.beng2.title')} — {t('results.beng2.subtitle')}</td>
                  <td>{result.beng2.toFixed(1)} {t('results.beng2.unit')}</td>
                  <td>≤ {result.beng2Limit}</td>
                  <td className={result.beng2Pass ? 'report-pass' : 'report-fail'}>
                    {result.beng2Pass ? t('results.pass') : t('results.fail')}
                  </td>
                </tr>
                <tr>
                  <td>{t('results.beng3.title')} — {t('results.beng3.subtitle')}</td>
                  <td>{result.beng3.toFixed(1)} {t('results.beng3.unit')}</td>
                  <td>≥ {result.beng3Limit}%</td>
                  <td className={result.beng3Pass ? 'report-pass' : 'report-fail'}>
                    {result.beng3Pass ? t('results.pass') : t('results.fail')}
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
}
