import { useState } from 'react';
import { useEnergy } from '../../../context/EnergyContext';
import type { IPointThermalBridge, ThermalBoundary } from '../../../core/energy/types';
import { useI18n } from '../../../i18n/i18n';
import { DialogShell } from '../DialogShell';

const boundaries: ThermalBoundary[] = ['outdoor', 'ground', 'unheated_space', 'adjacent_conditioned', 'internal'];

export function PointBridgeDialog({ editId, onClose }: { editId?: string | null; onClose: () => void }) {
  const { state, dispatch } = useEnergy();
  const { t } = useI18n();
  const existingZone = state.project.zones.find((zone) =>
    zone.pointThermalBridges?.some((bridge) => bridge.id === editId));
  const existing = existingZone?.pointThermalBridges?.find((bridge) => bridge.id === editId);
  const [name, setName] = useState(existing?.name ?? '');
  const [chi, setChi] = useState(existing ? String(existing.chiValue) : '');
  const [boundary, setBoundary] = useState<ThermalBoundary | ''>(existing?.thermalBoundary ?? '');
  const [unheatedSpaceId, setUnheatedSpaceId] = useState(existing?.unheatedSpaceId ?? '');
  const [source, setSource] = useState(existing?.sourceReference ?? '');
  const [zoneId, setZoneId] = useState(existingZone?.id ?? state.project.zones[0]?.id ?? '');
  const [error, setError] = useState<string | null>(null);

  const save = () => {
    const chiValue = chi.trim() === '' ? NaN : Number(chi);
    if (!name.trim() || !zoneId || !Number.isFinite(chiValue) || !source.trim()
      || (boundary === 'unheated_space' && !unheatedSpaceId)) {
      setError(t('kernel.pointBridge.invalid'));
      return;
    }
    const bridge: IPointThermalBridge = {
      id: existing?.id ?? crypto.randomUUID(), name: name.trim(), chiValue,
      zoneId, thermalBoundary: boundary || undefined, sourceReference: source.trim(),
      unheatedSpaceId: boundary === 'unheated_space' ? unheatedSpaceId : undefined,
    };
    if (existing && existingZone) {
      dispatch({ type: 'UPDATE_POINT_BRIDGE', payload: {
        zoneId: existingZone.id, bridgeId: existing.id, data: bridge,
      } });
    } else {
      dispatch({ type: 'ADD_POINT_BRIDGE', payload: { zoneId, bridge } });
    }
    onClose();
  };

  return <DialogShell variant="sheet" title={t('kernel.pointBridge.title')} onClose={onClose} onSubmit={save}
    submitLabel={t('dialog.save')} cancelLabel={t('dialog.cancel')}>
    <p>{t('kernel.pointBridge.scope')}</p>
    <div className="dialog-field"><label htmlFor="point-bridge-name">{t('properties.name')}</label>
      <input id="point-bridge-name" value={name} onChange={(event) => setName(event.target.value)} />
    </div>
    {boundary === 'unheated_space' && <div className="dialog-field">
      <label htmlFor="point-bridge-unheated-space">{t('kernel.unheated.space')}</label>
      <select id="point-bridge-unheated-space" value={unheatedSpaceId} onChange={(event) => setUnheatedSpaceId(event.target.value)}>
        <option value="">--</option>
        {(state.project.unheatedSpaces ?? []).map((space) => <option key={space.id} value={space.id}>{space.name}</option>)}
      </select>
    </div>}
    <div className="dialog-field"><label htmlFor="point-bridge-chi">{t('kernel.pointBridge.chi')}</label>
      <input id="point-bridge-chi" type="number" step="any" value={chi}
        onChange={(event) => setChi(event.target.value)} />
    </div>
    <div className="dialog-field"><label htmlFor="point-bridge-boundary">{t('kernel.boundary.label')}</label>
      <select id="point-bridge-boundary" value={boundary}
        onChange={(event) => setBoundary(event.target.value as ThermalBoundary | '')}>
        <option value="">{t('kernel.boundary.unknown')}</option>
        {boundaries.map((value) => <option key={value} value={value}>{t(`kernel.boundary.${value}`)}</option>)}
      </select>
    </div>
    <div className="dialog-field"><label htmlFor="point-bridge-source">{t('kernel.pointBridge.source')}</label>
      <input id="point-bridge-source" value={source} onChange={(event) => setSource(event.target.value)} />
    </div>
    <div className="dialog-field"><label htmlFor="point-bridge-zone">{t('dialog.thermalBridge.zone')}</label>
      <select id="point-bridge-zone" value={zoneId} disabled={Boolean(existing)}
        onChange={(event) => setZoneId(event.target.value)}>
        <option value="">--</option>
        {state.project.zones.map((zone) => <option key={zone.id} value={zone.id}>{zone.name}</option>)}
      </select>
    </div>
    {error && <p role="alert" className="field-error-text">{error}</p>}
  </DialogShell>;
}
