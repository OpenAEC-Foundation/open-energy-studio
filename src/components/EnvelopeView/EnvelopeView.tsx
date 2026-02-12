import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import './EnvelopeView.css';

export function EnvelopeView() {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;

  return (
    <div className="envelope-view">
      <h2>{t('ribbon.envelope')}</h2>

      {project.zones.length === 0 && (
        <div className="envelope-empty">
          <p>{t('browser.zones')}: 0</p>
          <p>Voeg een zone toe via het Ribbon menu om te beginnen.</p>
        </div>
      )}

      {project.zones.map(zone => (
        <div key={zone.id} className="envelope-zone-card">
          <div className="envelope-zone-header">
            <h3>{zone.name}</h3>
            <div className="envelope-zone-meta">
              {zone.floorArea} m² | {zone.volume} m³
            </div>
            <button
              className="btn btn-sm btn-danger"
              onClick={() => dispatch({ type: 'DELETE_ZONE', payload: zone.id })}
            >
              {t('dialog.delete')}
            </button>
          </div>

          {/* Surfaces table */}
          <div className="envelope-section">
            <h4>{t('browser.surfaces')} ({zone.surfaces.length})</h4>
            {zone.surfaces.length > 0 && (
              <table className="envelope-table">
                <thead>
                  <tr>
                    <th>{t('properties.name')}</th>
                    <th>{t('properties.type')}</th>
                    <th>{t('properties.area')}</th>
                    <th>{t('properties.orientation')}</th>
                    <th>{t('browser.windows')}</th>
                  </tr>
                </thead>
                <tbody>
                  {zone.surfaces.map(surface => (
                    <tr
                      key={surface.id}
                      className={state.selectedItemId === surface.id ? 'selected' : ''}
                      onClick={() => dispatch({ type: 'SELECT_ITEM', payload: { id: surface.id, itemType: 'surface' } })}
                    >
                      <td>{surface.name}</td>
                      <td>{t(`surfaceType.${surface.type}`)}</td>
                      <td>{surface.area} m²</td>
                      <td>{t(`orientation.${surface.orientation}`)}</td>
                      <td>{surface.windows.length}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>

          {/* Thermal bridges */}
          <div className="envelope-section">
            <h4>{t('browser.thermalBridges')} ({zone.thermalBridges.length})</h4>
            {zone.thermalBridges.length > 0 && (
              <table className="envelope-table">
                <thead>
                  <tr>
                    <th>{t('properties.name')}</th>
                    <th>{t('properties.psiValue')}</th>
                    <th>{t('properties.length')}</th>
                    <th>H (W/K)</th>
                  </tr>
                </thead>
                <tbody>
                  {zone.thermalBridges.map(tb => (
                    <tr key={tb.id}>
                      <td>{tb.name}</td>
                      <td>{tb.psiValue}</td>
                      <td>{tb.length} m</td>
                      <td>{(tb.psiValue * tb.length).toFixed(2)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>

          {/* Air tightness */}
          <div className="envelope-section">
            <h4>{t('browser.airTightness')}</h4>
            <p className="envelope-meta">qv;10 = {zone.airTightness.qv10} dm³/(s·m²)</p>
          </div>
        </div>
      ))}

      {/* Constructions */}
      {project.constructions.length > 0 && (
        <div className="envelope-zone-card">
          <div className="envelope-zone-header">
            <h3>{t('browser.constructions')}</h3>
          </div>
          <table className="envelope-table">
            <thead>
              <tr>
                <th>{t('properties.name')}</th>
                <th>{t('properties.rcValue')}</th>
                <th>{t('properties.uValue')}</th>
              </tr>
            </thead>
            <tbody>
              {project.constructions.map(c => (
                <tr key={c.id}>
                  <td>{c.name}</td>
                  <td>{c.rcValue.toFixed(2)} m²K/W</td>
                  <td>{c.uValue.toFixed(3)} W/m²K</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
