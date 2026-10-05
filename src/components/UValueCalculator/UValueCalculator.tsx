import { useState, useMemo, useCallback } from 'react';
import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { Plus, Trash2, Copy, Check, GripVertical, ChevronUp, ChevronDown } from 'lucide-react';
import './UValueCalculator.css';

// ============================================================
// Types
// ============================================================

type SurfacePosition = 'wall' | 'roof' | 'floor';

interface CalculatorLayer {
  id: string;
  material: string;
  thickness: number;       // m
  lambda: number;          // W/(m·K)
  fixedRc: number | null;  // m²·K/W — for air cavities with fixed Rc
}

// ============================================================
// Materials library (Dutch construction industry)
// ============================================================

interface MaterialDef {
  nameKey: string;         // i18n key
  lambda: number;          // W/(m·K), 0 if fixedRc is used
  fixedRc: number | null;  // m²·K/W
  defaultThickness: number; // m
  color: string;           // for visual bar
}

const MATERIALS_LIBRARY: MaterialDef[] = [
  { nameKey: 'uvalue.mat.concrete',          lambda: 2.00,  fixedRc: null, defaultThickness: 0.200, color: '#7a7a7a' },
  { nameKey: 'uvalue.mat.brick',             lambda: 0.80,  fixedRc: null, defaultThickness: 0.100, color: '#b85c38' },
  { nameKey: 'uvalue.mat.sandLime',          lambda: 1.10,  fixedRc: null, defaultThickness: 0.100, color: '#d4c9a8' },
  { nameKey: 'uvalue.mat.plasterboard',      lambda: 0.25,  fixedRc: null, defaultThickness: 0.0125, color: '#e8e0d0' },
  { nameKey: 'uvalue.mat.wood',              lambda: 0.13,  fixedRc: null, defaultThickness: 0.018, color: '#c4955a' },
  { nameKey: 'uvalue.mat.plywood',           lambda: 0.17,  fixedRc: null, defaultThickness: 0.018, color: '#a0845c' },
  { nameKey: 'uvalue.mat.osb',               lambda: 0.13,  fixedRc: null, defaultThickness: 0.018, color: '#b89a60' },
  { nameKey: 'uvalue.mat.mineralWool',       lambda: 0.035, fixedRc: null, defaultThickness: 0.150, color: '#e6d950' },
  { nameKey: 'uvalue.mat.glassWool',         lambda: 0.032, fixedRc: null, defaultThickness: 0.150, color: '#edd75a' },
  { nameKey: 'uvalue.mat.rockWool',          lambda: 0.035, fixedRc: null, defaultThickness: 0.150, color: '#a8a050' },
  { nameKey: 'uvalue.mat.purPir',            lambda: 0.023, fixedRc: null, defaultThickness: 0.120, color: '#f0c040' },
  { nameKey: 'uvalue.mat.eps',               lambda: 0.032, fixedRc: null, defaultThickness: 0.100, color: '#f0f0f0' },
  { nameKey: 'uvalue.mat.xps',               lambda: 0.035, fixedRc: null, defaultThickness: 0.100, color: '#80b8e0' },
  { nameKey: 'uvalue.mat.cellulose',         lambda: 0.038, fixedRc: null, defaultThickness: 0.150, color: '#a0a070' },
  { nameKey: 'uvalue.mat.airCavityStill',    lambda: 0,     fixedRc: 0.18, defaultThickness: 0.040, color: '#c8e8ff' },
  { nameKey: 'uvalue.mat.airCavityVent',     lambda: 0,     fixedRc: 0.09, defaultThickness: 0.040, color: '#a0d0f0' },
  { nameKey: 'uvalue.mat.steelSheet',        lambda: 50.0,  fixedRc: null, defaultThickness: 0.001, color: '#a0a8b0' },
];

// Rsi / Rse per surface position (NEN-EN-ISO 6946)
const RSI_RSE: Record<SurfacePosition, { rsi: number; rse: number }> = {
  wall:  { rsi: 0.13, rse: 0.04 },
  roof:  { rsi: 0.10, rse: 0.04 },
  floor: { rsi: 0.17, rse: 0.04 },
};

let layerIdCounter = 0;
function newLayerId(): string {
  return `layer-${++layerIdCounter}-${Date.now()}`;
}

// ============================================================
// Component
// ============================================================

export function UValueCalculator() {
  const { t, locale } = useI18n();
  const n = (value: number, digits: number) => formatNumber(value, locale, digits);

  const [surfacePosition, setSurfacePosition] = useState<SurfacePosition>('wall');
  const [layers, setLayers] = useState<CalculatorLayer[]>([
    { id: newLayerId(), material: t('uvalue.mat.brick'),       thickness: 0.100, lambda: 0.80,  fixedRc: null },
    { id: newLayerId(), material: t('uvalue.mat.airCavityStill'), thickness: 0.040, lambda: 0,  fixedRc: 0.18 },
    { id: newLayerId(), material: t('uvalue.mat.mineralWool'), thickness: 0.150, lambda: 0.035, fixedRc: null },
    { id: newLayerId(), material: t('uvalue.mat.plasterboard'), thickness: 0.0125, lambda: 0.25, fixedRc: null },
  ]);
  const [copied, setCopied] = useState(false);

  // ----------------------------------------------------------
  // Calculations
  // ----------------------------------------------------------
  const { layerRcs, totalRc, uValue, totalThickness } = useMemo(() => {
    const { rsi, rse } = RSI_RSE[surfacePosition];
    const layerRcs = layers.map(l =>
      l.fixedRc !== null ? l.fixedRc : (l.lambda > 0 ? l.thickness / l.lambda : 0)
    );
    const rcLayers = layerRcs.reduce((a, b) => a + b, 0);
    const totalRc = rsi + rcLayers + rse;
    const uValue = totalRc > 0 ? 1 / totalRc : 0;
    const totalThickness = layers.reduce((a, l) => a + l.thickness, 0);
    return { layerRcs, totalRc, uValue, totalThickness };
  }, [layers, surfacePosition]);

  // ----------------------------------------------------------
  // Handlers
  // ----------------------------------------------------------
  const addLayer = useCallback(() => {
    setLayers(prev => [
      ...prev,
      { id: newLayerId(), material: '', thickness: 0.100, lambda: 0.035, fixedRc: null },
    ]);
  }, []);

  const removeLayer = useCallback((id: string) => {
    setLayers(prev => prev.filter(l => l.id !== id));
  }, []);

  const updateLayer = useCallback((id: string, updates: Partial<CalculatorLayer>) => {
    setLayers(prev => prev.map(l => l.id === id ? { ...l, ...updates } : l));
  }, []);

  const moveLayer = useCallback((index: number, direction: -1 | 1) => {
    setLayers(prev => {
      const next = [...prev];
      const targetIndex = index + direction;
      if (targetIndex < 0 || targetIndex >= next.length) return prev;
      [next[index], next[targetIndex]] = [next[targetIndex], next[index]];
      return next;
    });
  }, []);

  const selectMaterial = useCallback((layerId: string, materialIndex: number) => {
    const mat = MATERIALS_LIBRARY[materialIndex];
    if (!mat) return;
    updateLayer(layerId, {
      material: t(mat.nameKey),
      lambda: mat.lambda,
      thickness: mat.defaultThickness,
      fixedRc: mat.fixedRc,
    });
  }, [t, updateLayer]);

  const copyResult = useCallback(() => {
    const text = `Rc = ${n(totalRc, 2)} m\u00b2\u00b7K/W | U = ${n(uValue, 3)} W/(m\u00b2\u00b7K)`;
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [totalRc, uValue, locale]);

  // ----------------------------------------------------------
  // Render
  // ----------------------------------------------------------
  const { rsi, rse } = RSI_RSE[surfacePosition];

  return (
    <div className="uvalue-view">
      <h2>{t('uvalue.title')}</h2>

      {/* Surface position selector */}
      <div className="uvalue-card">
        <div className="uvalue-card-header">
          <h3>{t('uvalue.surfaceType')}</h3>
        </div>
        <div className="uvalue-surface-selector">
          {(['wall', 'roof', 'floor'] as SurfacePosition[]).map(pos => (
            <button
              key={pos}
              className={`uvalue-surface-btn ${surfacePosition === pos ? 'active' : ''}`}
              onClick={() => setSurfacePosition(pos)}
            >
              {t(`uvalue.surface.${pos}`)}
              <span className="uvalue-surface-rsi-rse">
                Rsi={n(RSI_RSE[pos].rsi, 2)} / Rse={n(RSI_RSE[pos].rse, 2)}
              </span>
            </button>
          ))}
        </div>
      </div>

      {/* Results summary */}
      <div className="uvalue-results-bar">
        <div className="uvalue-result-item">
          <span className="uvalue-result-label">Rc</span>
          <span className="uvalue-result-value">{n(totalRc, 2)}</span>
          <span className="uvalue-result-unit">m{'\u00b2'}{'\u00b7'}K/W</span>
        </div>
        <div className="uvalue-result-item uvalue-result-highlight">
          <span className="uvalue-result-label">U</span>
          <span className="uvalue-result-value">{n(uValue, 3)}</span>
          <span className="uvalue-result-unit">W/(m{'\u00b2'}{'\u00b7'}K)</span>
        </div>
        <div className="uvalue-result-item">
          <span className="uvalue-result-label">{t('uvalue.totalThickness')}</span>
          <span className="uvalue-result-value">{n(totalThickness * 1000, 0)}</span>
          <span className="uvalue-result-unit">mm</span>
        </div>
        <button className="uvalue-copy-btn" onClick={copyResult} title={t('uvalue.copyResult')}>
          {copied ? <Check size={16} /> : <Copy size={16} />}
          <span>{copied ? t('uvalue.copied') : t('uvalue.copyResult')}</span>
        </button>
      </div>

      {/* Visual cross-section */}
      <div className="uvalue-card">
        <div className="uvalue-card-header">
          <h3>{t('uvalue.crossSection')}</h3>
        </div>
        <div className="uvalue-cross-section">
          {/* Rsi */}
          <div className="uvalue-cs-boundary" title={`Rsi = ${n(rsi, 2)}`}>
            <span className="uvalue-cs-label">Rsi</span>
          </div>
          {layers.map((layer) => {
            const pct = totalThickness > 0
              ? Math.max((layer.thickness / totalThickness) * 100, 4)
              : 100 / layers.length;
            const matDef = MATERIALS_LIBRARY.find(m => t(m.nameKey) === layer.material);
            const color = matDef?.color || '#888';
            return (
              <div
                key={layer.id}
                className="uvalue-cs-layer"
                style={{ flex: `${pct} 0 0%`, backgroundColor: color }}
                title={`${layer.material}: ${n(layer.thickness * 1000, 0)} mm`}
              >
                <span className="uvalue-cs-layer-label">
                  {n(layer.thickness * 1000, 0)}
                </span>
              </div>
            );
          })}
          {/* Rse */}
          <div className="uvalue-cs-boundary" title={`Rse = ${n(rse, 2)}`}>
            <span className="uvalue-cs-label">Rse</span>
          </div>
        </div>
      </div>

      {/* Layers table */}
      <div className="uvalue-card">
        <div className="uvalue-card-header">
          <h3>{t('uvalue.layers')}</h3>
          <button className="uvalue-add-btn" onClick={addLayer}>
            <Plus size={14} />
            {t('uvalue.addLayer')}
          </button>
        </div>

        <table className="uvalue-table">
          <thead>
            <tr>
              <th className="uvalue-th-order"></th>
              <th>{t('uvalue.material')}</th>
              <th>{t('uvalue.library')}</th>
              <th>{t('uvalue.thickness')} (mm)</th>
              <th>{'\u03bb'} (W/mK)</th>
              <th>Rc (m{'\u00b2'}K/W)</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {/* Rsi row */}
            <tr className="uvalue-boundary-row">
              <td></td>
              <td colSpan={3}>{t('uvalue.innerSurface')} (Rsi)</td>
              <td></td>
              <td className="uvalue-td-mono">{n(rsi, 2)}</td>
              <td></td>
            </tr>

            {layers.map((layer, index) => (
              <tr key={layer.id} className="uvalue-layer-row">
                <td className="uvalue-td-order">
                  <div className="uvalue-order-btns">
                    <button
                      className="uvalue-order-btn"
                      onClick={() => moveLayer(index, -1)}
                      disabled={index === 0}
                      title={t('uvalue.moveUp')}
                    >
                      <ChevronUp size={12} />
                    </button>
                    <GripVertical size={12} className="uvalue-grip" />
                    <button
                      className="uvalue-order-btn"
                      onClick={() => moveLayer(index, 1)}
                      disabled={index === layers.length - 1}
                      title={t('uvalue.moveDown')}
                    >
                      <ChevronDown size={12} />
                    </button>
                  </div>
                </td>
                <td>
                  <input
                    className="uvalue-input"
                    aria-label={`${t('uvalue.material')} ${index + 1}`}
                    type="text"
                    value={layer.material}
                    onChange={e => updateLayer(layer.id, { material: e.target.value })}
                    placeholder={t('uvalue.materialName')}
                  />
                </td>
                <td>
                  <select
                    className="uvalue-select"
                    aria-label={`${t('uvalue.library')} ${index + 1}`}
                    value=""
                    onChange={e => {
                      const idx = parseInt(e.target.value, 10);
                      if (!isNaN(idx)) selectMaterial(layer.id, idx);
                    }}
                  >
                    <option value="">{t('uvalue.selectMaterial')}</option>
                    {MATERIALS_LIBRARY.map((m, i) => (
                      <option key={i} value={i}>
                        {t(m.nameKey)} ({m.fixedRc !== null ? `Rc=${n(m.fixedRc, 2)}` : `\u03bb=${n(m.lambda, 3)}`})
                      </option>
                    ))}
                  </select>
                </td>
                <td>
                  <input
                    className="uvalue-input uvalue-input-num"
                    aria-label={`${t('uvalue.thickness')} (mm) ${index + 1}`}
                    type="number"
                    min={0}
                    step={1}
                    value={Math.round(layer.thickness * 1000)}
                    onChange={e => {
                      const mm = parseFloat(e.target.value) || 0;
                      updateLayer(layer.id, { thickness: mm / 1000 });
                    }}
                  />
                </td>
                <td>
                  {layer.fixedRc !== null ? (
                    <span className="uvalue-td-mono uvalue-fixed-label">
                      Rc={n(layer.fixedRc, 2)}
                    </span>
                  ) : (
                    <input
                      className="uvalue-input uvalue-input-num"
                      aria-label={`λ (W/mK) ${index + 1}`}
                      type="number"
                      min={0.001}
                      step={0.001}
                      value={layer.lambda}
                      onChange={e => {
                        const val = parseFloat(e.target.value);
                        if (!isNaN(val) && val > 0) updateLayer(layer.id, { lambda: val });
                      }}
                    />
                  )}
                </td>
                <td className="uvalue-td-mono">
                  {layerRcs[index] !== undefined ? n(layerRcs[index], 3) : '-'}
                </td>
                <td>
                  <button
                    className="uvalue-delete-btn"
                    onClick={() => removeLayer(layer.id)}
                    title={t('dialog.delete')}
                  >
                    <Trash2 size={14} />
                  </button>
                </td>
              </tr>
            ))}

            {/* Rse row */}
            <tr className="uvalue-boundary-row">
              <td></td>
              <td colSpan={3}>{t('uvalue.outerSurface')} (Rse)</td>
              <td></td>
              <td className="uvalue-td-mono">{n(rse, 2)}</td>
              <td></td>
            </tr>

            {/* Total row */}
            <tr className="uvalue-total-row">
              <td></td>
              <td colSpan={3}><strong>{t('uvalue.total')}</strong></td>
              <td className="uvalue-td-mono"><strong>{n(totalThickness * 1000, 0)} mm</strong></td>
              <td className="uvalue-td-mono"><strong>{n(totalRc, 3)}</strong></td>
              <td></td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
}
