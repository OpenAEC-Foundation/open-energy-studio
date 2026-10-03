import { useState, useMemo } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IConstruction, IConstructionLayer } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';
import { NtaConstructionSection, type NtaConstructionResult } from './NtaConstructionSection';

interface ConstructionEditorDialogProps {
  editId?: string | null;
  onClose: () => void;
}

export function ConstructionEditorDialog({ editId, onClose }: ConstructionEditorDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existing = editId
    ? state.project.constructions.find((c) => c.id === editId)
    : null;

  const [name, setName] = useState(existing?.name ?? '');
  const [layers, setLayers] = useState<IConstructionLayer[]>(
    existing?.layers ?? [{ material: '', thickness: 0.1, lambda: 0.04 }]
  );

  // Kernel result applied by the user; replaces the simple Σd/λ values below.
  const [ntaResult, setNtaResult] = useState<NtaConstructionResult | null>(null);

  // Rsi + Rse for walls (NTA 8800 default)
  const rSurface = 0.13 + 0.04; // 0.17

  const rcValue = useMemo(() => {
    return layers.reduce((sum, layer) => {
      if (layer.lambda > 0) {
        return sum + layer.thickness / layer.lambda;
      }
      return sum;
    }, 0);
  }, [layers]);

  const uValue = useMemo(() => {
    const rTotal = rcValue + rSurface;
    return rTotal > 0 ? 1 / rTotal : 0;
  }, [rcValue]);

  const handleLayerChange = (index: number, field: keyof IConstructionLayer, value: string | number) => {
    setNtaResult(null);
    setLayers((prev) =>
      prev.map((layer, i) =>
        i === index ? { ...layer, [field]: value } : layer
      )
    );
  };

  const addLayer = () => {
    setNtaResult(null);
    setLayers((prev) => [...prev, { material: '', thickness: 0.1, lambda: 0.04 }]);
  };

  const removeLayer = (index: number) => {
    setNtaResult(null);
    setLayers((prev) => prev.filter((_, i) => i !== index));
  };

  const handleSave = () => {
    const construction: IConstruction = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      layers,
      rcValue: ntaResult?.rc ?? Math.round(rcValue * 100) / 100,
      uValue: ntaResult?.u ?? Math.round(uValue * 1000) / 1000,
      // Only stored when it differs from the R_se = 0,04 the kernel assumes (C.10).
      exteriorSurfaceResistance: ntaResult && Math.abs(ntaResult.rse - 0.04) > 1e-9 ? ntaResult.rse : undefined,
    };

    if (existing) {
      dispatch({
        type: 'UPDATE_CONSTRUCTION',
        payload: { id: existing.id, data: { name, layers, rcValue: construction.rcValue, uValue: construction.uValue, exteriorSurfaceResistance: construction.exteriorSurfaceResistance } },
      });
    } else {
      dispatch({ type: 'ADD_CONSTRUCTION', payload: construction });
    }
    onClose();
  };

  return (
    <DialogShell
      title={t('dialog.construction.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
      style={{ minWidth: 500 }}
    >

        <div className="dialog-field">
          <label>{t('dialog.construction.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.construction.layers')}</label>
          <table style={{ width: '100%', borderCollapse: 'collapse', marginTop: 4 }}>
            <thead>
              <tr>
                <th style={{ textAlign: 'left', fontSize: 12, color: 'var(--text-secondary)', padding: '4px 8px' }}>
                  {t('dialog.construction.material')}
                </th>
                <th style={{ textAlign: 'left', fontSize: 12, color: 'var(--text-secondary)', padding: '4px 8px' }}>
                  {t('dialog.construction.thickness')}
                </th>
                <th style={{ textAlign: 'left', fontSize: 12, color: 'var(--text-secondary)', padding: '4px 8px' }}>
                  {t('dialog.construction.lambda')}
                </th>
                <th style={{ width: 32 }}></th>
              </tr>
            </thead>
            <tbody>
              {layers.map((layer, i) => (
                <tr key={i}>
                  <td style={{ padding: '2px 4px' }}>
                    <input
                      type="text"
                      value={layer.material}
                      onChange={(e) => handleLayerChange(i, 'material', e.target.value)}
                      style={{ width: '100%', padding: '6px 8px', background: 'var(--bg-secondary)', border: '1px solid var(--border-light)', borderRadius: 4, color: 'var(--text-primary)', fontSize: 13 }}
                    />
                  </td>
                  <td style={{ padding: '2px 4px' }}>
                    <input
                      type="number"
                      min={0}
                      step={0.001}
                      value={layer.thickness}
                      onChange={(e) => handleLayerChange(i, 'thickness', parseFloat(e.target.value) || 0)}
                      style={{ width: '100%', padding: '6px 8px', background: 'var(--bg-secondary)', border: '1px solid var(--border-light)', borderRadius: 4, color: 'var(--text-primary)', fontSize: 13, fontFamily: "'JetBrains Mono', monospace" }}
                    />
                  </td>
                  <td style={{ padding: '2px 4px' }}>
                    <input
                      type="number"
                      min={0.001}
                      step={0.001}
                      value={layer.lambda}
                      onChange={(e) => handleLayerChange(i, 'lambda', parseFloat(e.target.value) || 0)}
                      style={{ width: '100%', padding: '6px 8px', background: 'var(--bg-secondary)', border: '1px solid var(--border-light)', borderRadius: 4, color: 'var(--text-primary)', fontSize: 13, fontFamily: "'JetBrains Mono', monospace" }}
                    />
                  </td>
                  <td style={{ padding: '2px 4px', textAlign: 'center' }}>
                    {layers.length > 1 && (
                      <button
                        className="btn btn-sm"
                        onClick={() => removeLayer(i)}
                        style={{ padding: '2px 6px', color: 'var(--danger)' }}
                      >
                        x
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          <button className="btn btn-sm" onClick={addLayer} style={{ marginTop: 8 }}>
            {t('dialog.construction.addLayer')}
          </button>
        </div>

        <div style={{ display: 'flex', gap: 16, marginTop: 8 }}>
          <div className="dialog-field" style={{ flex: 1 }}>
            <label>{t('dialog.construction.rcValue')}</label>
            <input type="text" readOnly value={rcValue.toFixed(2)} />
          </div>
          <div className="dialog-field" style={{ flex: 1 }}>
            <label>{t('dialog.construction.uValue')}</label>
            <input type="text" readOnly value={uValue.toFixed(3)} />
          </div>
        </div>
        {ntaResult && <p role="status" style={{ fontSize: 12 }}>
          {t('nta.construction.applied')}: U = {ntaResult.u.toFixed(2)} W/m²K
          {ntaResult.rc != null && <> · R<sub>c</sub> = {ntaResult.rc.toFixed(2)} m²K/W</>}
          {' '}<button type="button" className="btn btn-sm" onClick={() => setNtaResult(null)}>{t('nta.construction.undo')}</button>
        </p>}
        <NtaConstructionSection layers={layers} onApply={setNtaResult} />

    </DialogShell>
  );
}
