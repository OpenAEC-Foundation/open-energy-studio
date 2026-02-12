import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IThermalBridge } from '../../../core/energy/types';

interface ThermalBridgeDialogProps {
  editId?: string | null;
  onClose: () => void;
}

export function ThermalBridgeDialog({ editId, onClose }: ThermalBridgeDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;

  // Find existing thermal bridge across all zones
  let existingBridge: IThermalBridge | null = null;
  let existingZoneId: string | null = null;
  if (editId) {
    for (const zone of project.zones) {
      const found = zone.thermalBridges.find((b) => b.id === editId);
      if (found) {
        existingBridge = found;
        existingZoneId = zone.id;
        break;
      }
    }
  }

  const [name, setName] = useState(existingBridge?.name ?? '');
  const [psiValue, setPsiValue] = useState(existingBridge?.psiValue ?? 0.05);
  const [length, setLength] = useState(existingBridge?.length ?? 0);
  const [zoneId, setZoneId] = useState(existingZoneId ?? (project.zones[0]?.id ?? ''));

  const handleSave = () => {
    if (!zoneId) return;

    if (existingBridge && existingZoneId) {
      dispatch({
        type: 'UPDATE_THERMAL_BRIDGE',
        payload: {
          zoneId: existingZoneId,
          bridgeId: existingBridge.id,
          data: { name, psiValue, length },
        },
      });
    } else {
      const bridge: IThermalBridge = {
        id: crypto.randomUUID(),
        name,
        psiValue,
        length,
        zoneId,
      };
      dispatch({ type: 'ADD_THERMAL_BRIDGE', payload: { zoneId, bridge } });
    }
    onClose();
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">{t('dialog.thermalBridge.title')}</div>

        <div className="dialog-field">
          <label>{t('dialog.thermalBridge.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.thermalBridge.psiValue')}</label>
          <input
            type="number"
            min={0}
            step={0.001}
            value={psiValue}
            onChange={(e) => setPsiValue(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.thermalBridge.length')}</label>
          <input
            type="number"
            min={0}
            step={0.1}
            value={length}
            onChange={(e) => setLength(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.thermalBridge.zone')}</label>
          <select
            value={zoneId}
            onChange={(e) => setZoneId(e.target.value)}
            disabled={!!existingBridge}
          >
            <option value="">--</option>
            {project.zones.map((z) => (
              <option key={z.id} value={z.id}>
                {z.name}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-actions">
          <button className="btn" onClick={onClose}>
            {t('dialog.cancel')}
          </button>
          <button className="btn btn-primary" onClick={handleSave}>
            {t('dialog.save')}
          </button>
        </div>
      </div>
    </div>
  );
}
