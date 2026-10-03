import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { formatNumber } from '../../i18n/format';
import { editAction } from '../../core/energy/projectItems';
import { ItemActions } from '../ItemActions/ItemActions';
import './EnvelopeView.css';

export function EnvelopeView() {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;
  const n = (value: number | null | undefined, digits = 0) => formatNumber(value, locale, digits);
  const decimals = (value: number) => (Number.isInteger(value) ? 0 : Math.min(3, String(value).split('.')[1]?.length ?? 0));
  const num = (value: number | null | undefined) => (value == null ? '–' : n(value, decimals(value)));
  const select = (id: string, itemType: string) => dispatch({ type: 'SELECT_ITEM', payload: { id, itemType } });
  const openEdit = (itemType: string, id: string) => {
    const action = editAction(itemType, id);
    if (action) dispatch(action);
  };

  return (
    <div className="envelope-view">
      <h2>{t('ribbon.envelope')}</h2>

      {project.zones.length === 0 && (
        <div className="envelope-empty">
          <p>{t('browser.zones')}: 0</p>
          <p>{t('envelope.emptyHint')}</p>
        </div>
      )}

      {project.zones.map(zone => (
        <div key={zone.id} className="envelope-zone-card">
          <div className="envelope-zone-header">
            <h3>{zone.name}</h3>
            <div className="envelope-zone-meta">
              {num(zone.floorArea)} m² | {num(zone.volume)} m³
            </div>
            <ItemActions itemType="zone" id={zone.id} name={zone.name} />
          </div>

          {/* Surfaces and their windows */}
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
                    <th>{t('envelope.actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  {zone.surfaces.map(surface => (
                    <tr
                      key={surface.id}
                      className={state.selectedItemId === surface.id ? 'selected' : ''}
                      onClick={() => select(surface.id, 'surface')}
                      onDoubleClick={() => openEdit('surface', surface.id)}
                    >
                      <td>{surface.name}</td>
                      <td>{t(`surfaceType.${surface.type}`)}</td>
                      <td>{num(surface.area)} m²</td>
                      <td>{t(`orientation.${surface.orientation}`)}</td>
                      <td>{surface.windows.length}</td>
                      <td><ItemActions itemType="surface" id={surface.id} name={surface.name} /></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
            {zone.surfaces.some((surface) => surface.windows.length > 0) && <>
              <h4>{t('envelope.windowsOf')} ({zone.surfaces.reduce((sum, s) => sum + s.windows.length, 0)})</h4>
              <table className="envelope-table">
                <thead>
                  <tr>
                    <th>{t('properties.name')}</th>
                    <th>{t('envelope.surfaceOf')}</th>
                    <th>{t('properties.area')}</th>
                    <th>{t('properties.uValue')}</th>
                    <th>{t('properties.gValue')}</th>
                    <th>{t('envelope.actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  {zone.surfaces.flatMap(surface => surface.windows.map(win => (
                    <tr
                      key={win.id}
                      className={state.selectedItemId === win.id ? 'selected' : ''}
                      onClick={() => select(win.id, 'window')}
                      onDoubleClick={() => openEdit('window', win.id)}
                    >
                      <td>{win.name}</td>
                      <td>{surface.name}</td>
                      <td>{num(win.area)} m²</td>
                      <td>{num(win.uValue)} W/m²K</td>
                      <td>{num(win.gValue)}</td>
                      <td><ItemActions itemType="window" id={win.id} name={win.name} /></td>
                    </tr>
                  )))}
                </tbody>
              </table>
            </>}
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
                    <th>{t('envelope.actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  {zone.thermalBridges.map(tb => (
                    <tr key={tb.id} onDoubleClick={() => openEdit('thermalBridge', tb.id)}>
                      <td>{tb.name}</td>
                      <td>{num(tb.psiValue)}</td>
                      <td>{num(tb.length)} m</td>
                      <td>{n(tb.psiValue * tb.length, 2)}</td>
                      <td><ItemActions itemType="thermalBridge" id={tb.id} name={tb.name} /></td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>

          {/* Point thermal bridges */}
          <div className="envelope-section">
            <h4>{t('kernel.pointBridge.title')} ({zone.pointThermalBridges?.length ?? 0})</h4>
            <p className="envelope-meta">{zone.pointBridgeInventoryComplete
              ? t('kernel.pointBridge.inventoryComplete')
              : t('kernel.pointBridge.inventoryUnknown')}</p>
            {!zone.pointBridgeInventoryComplete && <button type="button" className="btn btn-sm"
              onClick={() => dispatch({ type: 'UPDATE_ZONE', payload: { id: zone.id, data: {
                pointThermalBridges: zone.pointThermalBridges ?? [], pointBridgeInventoryComplete: true,
              } } })}>{t('kernel.pointBridge.confirmInventory')}</button>}
            {(zone.pointThermalBridges?.length ?? 0) > 0 && <table className="envelope-table">
              <thead><tr><th>{t('properties.name')}</th><th>{t('kernel.pointBridge.chi')}</th>
                <th>{t('kernel.boundary.label')}</th><th>{t('kernel.pointBridge.source')}</th><th>{t('kernel.pointBridge.actions')}</th></tr></thead>
              <tbody>{zone.pointThermalBridges?.map((bridge) => <tr key={bridge.id}
                onDoubleClick={() => openEdit('pointBridge', bridge.id)}>
                <td>{bridge.name}</td><td>{num(bridge.chiValue)} W/K</td>
                <td>{bridge.thermalBoundary ? t(`kernel.boundary.${bridge.thermalBoundary}`) : t('kernel.boundary.unknown')}</td>
                <td>{bridge.sourceReference}</td>
                <td><ItemActions itemType="pointBridge" id={bridge.id} name={bridge.name} /></td>
              </tr>)}</tbody>
            </table>}
          </div>

          {/* Air tightness */}
          <div className="envelope-section">
            <h4>{t('browser.airTightness')}</h4>
            <p className="envelope-meta">qv;10 = {num(zone.airTightness.qv10)} dm³/(s·m²)</p>
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
                <th>{t('envelope.actions')}</th>
              </tr>
            </thead>
            <tbody>
              {project.constructions.map(c => (
                <tr key={c.id}
                  className={state.selectedItemId === c.id ? 'selected' : ''}
                  onClick={() => select(c.id, 'construction')}
                  onDoubleClick={() => openEdit('construction', c.id)}>
                  <td>{c.name}</td>
                  <td>{n(c.rcValue, 2)} m²K/W</td>
                  <td>{n(c.uValue, 3)} W/m²K</td>
                  <td><ItemActions itemType="construction" id={c.id} name={c.name} /></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
