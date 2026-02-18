import { useState, useMemo } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import {
  calculateHeatPumpSizing,
  BUILDING_FUNCTION_PARAMS,
  DESIGN_OUTDOOR_TEMP,
  type ZoneSizingResult,
} from '../../core/energy/HeatPumpSizingCalc';
import './HeatPumpSizingCalculator.css';

export function HeatPumpSizingCalculator() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { project } = state;

  const [designOutdoorTemp, setDesignOutdoorTemp] = useState(DESIGN_OUTDOOR_TEMP);
  const [customIndoorTemp, setCustomIndoorTemp] = useState<number | null>(null);
  const [safetyMargin, setSafetyMargin] = useState(10); // percent
  const [includeReheat, setIncludeReheat] = useState(true);
  const [expandedZones, setExpandedZones] = useState<Set<string>>(new Set(['zone-main']));

  const params = BUILDING_FUNCTION_PARAMS[project.buildingFunction];

  const result = useMemo(() => {
    if (project.zones.length === 0) return null;
    return calculateHeatPumpSizing({
      zones: project.zones,
      constructions: project.constructions,
      buildingFunction: project.buildingFunction,
      ventilationSystems: project.ventilationSystems,
      designOutdoorTemp,
      designIndoorTemp: customIndoorTemp ?? undefined,
      safetyMargin: safetyMargin / 100,
      includeReheat,
    });
  }, [project.zones, project.constructions, project.buildingFunction, project.ventilationSystems,
      designOutdoorTemp, customIndoorTemp, safetyMargin, includeReheat]);

  const toggleZone = (zoneId: string) => {
    setExpandedZones(prev => {
      const next = new Set(prev);
      if (next.has(zoneId)) next.delete(zoneId);
      else next.add(zoneId);
      return next;
    });
  };

  const fmt = (v: number, decimals = 0) => v.toFixed(decimals);
  const fmtW = (v: number) => v >= 1000 ? `${(v / 1000).toFixed(2)} kW` : `${v.toFixed(0)} W`;

  if (project.zones.length === 0) {
    return (
      <div className="hp-view">
        <h2>{t('hp.title')}</h2>
        <div className="hp-no-data">{t('hp.noZones')}</div>
      </div>
    );
  }

  return (
    <div className="hp-view">
      <h2>{t('hp.title')}</h2>

      {/* Results bar */}
      {result && (
        <div className="hp-results-bar">
          <div className="hp-result-item">
            <span className="hp-result-label">{t('hp.transmissionShort')}</span>
            <span className="hp-result-value">{fmtW(result.totalTransmissionLoss)}</span>
          </div>
          <div className="hp-result-divider" />
          <div className="hp-result-item">
            <span className="hp-result-label">{t('hp.ventilationShort')}</span>
            <span className="hp-result-value">{fmtW(result.totalVentilationLoss)}</span>
          </div>
          <div className="hp-result-divider" />
          <div className="hp-result-item">
            <span className="hp-result-label">{t('hp.infiltrationShort')}</span>
            <span className="hp-result-value">{fmtW(result.totalInfiltrationLoss)}</span>
          </div>
          {result.includeReheat && (
            <>
              <div className="hp-result-divider" />
              <div className="hp-result-item">
                <span className="hp-result-label">{t('hp.reheatShort')}</span>
                <span className="hp-result-value">{fmtW(result.totalReheatAllowance)}</span>
              </div>
            </>
          )}
          <div className="hp-result-divider" />
          <div className="hp-result-item hp-result-highlight">
            <span className="hp-result-label">{t('hp.recommendation')}</span>
            <span className="hp-result-value">{result.recommendedKW.toFixed(1)}</span>
            <span className="hp-result-unit">kW</span>
          </div>
        </div>
      )}

      {/* Input parameters */}
      <div className="hp-card">
        <div className="hp-card-header">
          <h3>{t('hp.inputParams')}</h3>
          <div className="hp-param-badges">
            <span className="hp-param-badge">{t(`function.${project.buildingFunction}`)}</span>
            <span className="hp-param-badge">f_rh = {params.reheatFactor} W/m²</span>
            <span className="hp-param-badge">q_v = {params.ventilationRate} dm³/s·m²</span>
          </div>
        </div>
        <div className="hp-card-body">
          <div className="hp-input-row">
            <div className="hp-input-group">
              <span className="hp-input-label">{t('hp.outdoorTemp')}</span>
              <input
                type="number"
                className="hp-input"
                value={designOutdoorTemp}
                onChange={e => setDesignOutdoorTemp(Number(e.target.value))}
                step={1}
              />
            </div>
            <div className="hp-input-group">
              <span className="hp-input-label">{t('hp.indoorTemp')}</span>
              <input
                type="number"
                className="hp-input"
                value={customIndoorTemp ?? params.thetaInt}
                onChange={e => {
                  const v = Number(e.target.value);
                  setCustomIndoorTemp(v === params.thetaInt ? null : v);
                }}
                step={1}
              />
            </div>
            <div className="hp-input-group">
              <span className="hp-input-label">{t('hp.safetyMargin')}</span>
              <input
                type="number"
                className="hp-input"
                value={safetyMargin}
                onChange={e => setSafetyMargin(Number(e.target.value))}
                min={0}
                max={50}
                step={5}
              />
            </div>
            <div className="hp-input-group">
              <span className="hp-input-label">&nbsp;</span>
              <div className="hp-checkbox-group">
                <input
                  type="checkbox"
                  id="hp-reheat"
                  checked={includeReheat}
                  onChange={e => setIncludeReheat(e.target.checked)}
                />
                <label htmlFor="hp-reheat">{t('hp.reheatAllowance')}</label>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Detail per zone */}
      {result && result.zones.map(zone => (
        <ZoneDetail
          key={zone.zoneId}
          zone={zone}
          expanded={expandedZones.has(zone.zoneId)}
          onToggle={() => toggleZone(zone.zoneId)}
          t={t}
        />
      ))}

      {/* Summary */}
      {result && (
        <div className="hp-card">
          <div className="hp-card-header">
            <h3>{t('hp.summary')}</h3>
          </div>
          <div className="hp-card-body">
            <div className="hp-summary-grid">
              <div className="hp-summary-item">
                <span className="hp-summary-label">{t('hp.totalTransmission')}</span>
                <span className="hp-summary-value">{fmtW(result.totalTransmissionLoss)}</span>
              </div>
              <div className="hp-summary-item">
                <span className="hp-summary-label">{t('hp.totalVentilation')}</span>
                <span className="hp-summary-value">{fmtW(result.totalVentilationLoss)}</span>
              </div>
              <div className="hp-summary-item">
                <span className="hp-summary-label">{t('hp.totalInfiltration')}</span>
                <span className="hp-summary-value">{fmtW(result.totalInfiltrationLoss)}</span>
              </div>
              {result.includeReheat && (
                <div className="hp-summary-item">
                  <span className="hp-summary-label">{t('hp.totalReheat')}</span>
                  <span className="hp-summary-value">{fmtW(result.totalReheatAllowance)}</span>
                </div>
              )}
              <div className="hp-summary-item">
                <span className="hp-summary-label">{t('hp.designLoad')}</span>
                <span className="hp-summary-value">{fmtW(result.totalDesignLoad)}</span>
              </div>
              <div className="hp-summary-item">
                <span className="hp-summary-label">{t('hp.withMargin')} ({fmt(safetyMargin)}%)</span>
                <span className="hp-summary-value">{fmtW(result.totalWithMargin)}</span>
              </div>
              <div className="hp-summary-item hp-summary-highlight">
                <span className="hp-summary-label">{t('hp.recommendation')}</span>
                <span className="hp-summary-value">{result.recommendedKW.toFixed(1)} kW</span>
              </div>
              <div className="hp-summary-item">
                <span className="hp-summary-label">H_t ({t('hp.transmissionCoeff')})</span>
                <span className="hp-summary-value">{result.htTransmission.toFixed(1)} W/K</span>
              </div>
              <div className="hp-summary-item">
                <span className="hp-summary-label">{t('hp.specificLoad')}</span>
                <span className="hp-summary-value">{result.specificLoad.toFixed(1)} W/m²</span>
              </div>
              <div className="hp-summary-item">
                <span className="hp-summary-label">{t('hp.totalFloorArea')}</span>
                <span className="hp-summary-value">{result.totalFloorArea.toFixed(1)} m²</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// ---- Zone detail subcomponent ----

function ZoneDetail({ zone, expanded, onToggle, t }: {
  zone: ZoneSizingResult;
  expanded: boolean;
  onToggle: () => void;
  t: (key: string) => string;
}) {
  const fmtW = (v: number) => v.toFixed(0);

  return (
    <div className="hp-card">
      <div className="hp-card-header">
        <div className="hp-zone-header" onClick={onToggle}>
          <span className={`hp-zone-toggle ${expanded ? 'hp-expanded' : ''}`}>&#9654;</span>
          <h4>{zone.zoneName}</h4>
          <span className="hp-zone-badge">{zone.floorArea.toFixed(1)} m²</span>
          <span className="hp-zone-badge">{(zone.totalZoneLoss / 1000).toFixed(2)} kW</span>
        </div>
      </div>
      {expanded && (
        <div className="hp-zone-content">
          {/* Surfaces */}
          {zone.surfaceLosses.length > 0 && (
            <>
              <div className="hp-section-title">{t('hp.opaqueElements')}</div>
              <table className="hp-table">
                <thead>
                  <tr>
                    <th>{t('hp.element')}</th>
                    <th>{t('hp.type')}</th>
                    <th style={{ textAlign: 'right' }}>A_net (m²)</th>
                    <th style={{ textAlign: 'right' }}>U (W/m²K)</th>
                    <th style={{ textAlign: 'right' }}>{t('hp.deltaT')} (K)</th>
                    <th style={{ textAlign: 'right' }}>{t('hp.heatLoss')} (W)</th>
                  </tr>
                </thead>
                <tbody>
                  {zone.surfaceLosses.map(s => (
                    <tr key={s.surfaceId}>
                      <td>{s.surfaceName}</td>
                      <td>{t(`surfaceType.${s.surfaceType}`)}</td>
                      <td className="hp-td-mono">{s.netArea.toFixed(1)}</td>
                      <td className="hp-td-mono">{s.uValue.toFixed(3)}</td>
                      <td className="hp-td-mono">{s.deltaT.toFixed(0)}</td>
                      <td className="hp-td-mono">{fmtW(s.heatLoss)}</td>
                    </tr>
                  ))}
                  <tr className="hp-subtotal-row">
                    <td colSpan={5}>{t('hp.subtotalSurfaces')}</td>
                    <td className="hp-td-mono">
                      {fmtW(zone.surfaceLosses.reduce((s, x) => s + x.heatLoss, 0))}
                    </td>
                  </tr>
                </tbody>
              </table>
            </>
          )}

          {/* Windows */}
          {zone.windowLosses.length > 0 && (
            <>
              <div className="hp-section-title">{t('hp.windows')}</div>
              <table className="hp-table">
                <thead>
                  <tr>
                    <th>{t('hp.element')}</th>
                    <th>{t('hp.surface')}</th>
                    <th style={{ textAlign: 'right' }}>A (m²)</th>
                    <th style={{ textAlign: 'right' }}>U_w (W/m²K)</th>
                    <th style={{ textAlign: 'right' }}>{t('hp.deltaT')} (K)</th>
                    <th style={{ textAlign: 'right' }}>{t('hp.heatLoss')} (W)</th>
                  </tr>
                </thead>
                <tbody>
                  {zone.windowLosses.map(w => (
                    <tr key={w.windowId}>
                      <td>{w.windowName}</td>
                      <td>{w.surfaceName}</td>
                      <td className="hp-td-mono">{w.area.toFixed(1)}</td>
                      <td className="hp-td-mono">{w.uValue.toFixed(2)}</td>
                      <td className="hp-td-mono">{w.deltaT.toFixed(0)}</td>
                      <td className="hp-td-mono">{fmtW(w.heatLoss)}</td>
                    </tr>
                  ))}
                  <tr className="hp-subtotal-row">
                    <td colSpan={5}>{t('hp.subtotalWindows')}</td>
                    <td className="hp-td-mono">
                      {fmtW(zone.windowLosses.reduce((s, x) => s + x.heatLoss, 0))}
                    </td>
                  </tr>
                </tbody>
              </table>
            </>
          )}

          {/* Thermal bridges */}
          {zone.thermalBridgeLosses.length > 0 && (
            <>
              <div className="hp-section-title">{t('hp.thermalBridges')}</div>
              <table className="hp-table">
                <thead>
                  <tr>
                    <th>{t('hp.element')}</th>
                    <th style={{ textAlign: 'right' }}>{t('hp.psiValue')} (W/mK)</th>
                    <th style={{ textAlign: 'right' }}>{t('hp.length')} (m)</th>
                    <th style={{ textAlign: 'right' }}>{t('hp.deltaT')} (K)</th>
                    <th style={{ textAlign: 'right' }}>{t('hp.heatLoss')} (W)</th>
                  </tr>
                </thead>
                <tbody>
                  {zone.thermalBridgeLosses.map(tb => (
                    <tr key={tb.bridgeId}>
                      <td>{tb.bridgeName}</td>
                      <td className="hp-td-mono">{tb.psiValue.toFixed(3)}</td>
                      <td className="hp-td-mono">{tb.length.toFixed(1)}</td>
                      <td className="hp-td-mono">{tb.deltaT.toFixed(0)}</td>
                      <td className="hp-td-mono">{fmtW(tb.heatLoss)}</td>
                    </tr>
                  ))}
                  <tr className="hp-subtotal-row">
                    <td colSpan={4}>{t('hp.subtotalBridges')}</td>
                    <td className="hp-td-mono">
                      {fmtW(zone.thermalBridgeLosses.reduce((s, x) => s + x.heatLoss, 0))}
                    </td>
                  </tr>
                </tbody>
              </table>
            </>
          )}

          {/* Zone summary */}
          <table className="hp-table">
            <tbody>
              <tr className="hp-subtotal-row">
                <td>{t('hp.totalTransmission')}</td>
                <td className="hp-td-mono">{fmtW(zone.transmissionLoss)} W</td>
              </tr>
              <tr className="hp-subtotal-row">
                <td>{t('hp.totalVentilation')}</td>
                <td className="hp-td-mono">{fmtW(zone.ventilationLoss)} W</td>
              </tr>
              <tr className="hp-subtotal-row">
                <td>{t('hp.totalInfiltration')}</td>
                <td className="hp-td-mono">{fmtW(zone.infiltrationLoss)} W</td>
              </tr>
              {zone.reheatAllowance > 0 && (
                <tr className="hp-subtotal-row">
                  <td>{t('hp.totalReheat')}</td>
                  <td className="hp-td-mono">{fmtW(zone.reheatAllowance)} W</td>
                </tr>
              )}
              <tr className="hp-total-row">
                <td>{t('hp.totalZone')}</td>
                <td className="hp-td-mono">{fmtW(zone.totalZoneLoss)} W</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
