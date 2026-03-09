import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IThermalBridge } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';

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
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleSave = () => {
    const newErrors: Record<string, string> = {};
    if (!name.trim()) newErrors.name = t('dialog.validation.required');
    if (!zoneId) newErrors.zoneId = t('dialog.validation.required');
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }
    setErrors({});

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
    <DialogShell
      title={t('dialog.thermalBridge.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label>{t('dialog.thermalBridge.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => { setName(e.target.value); setErrors(prev => ({ ...prev, name: '' })); }}
            className={errors.name ? 'field-error' : ''}
          />
          {errors.name && <span className="field-error-text">{errors.name}</span>}
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
            onChange={(e) => { setZoneId(e.target.value); setErrors(prev => ({ ...prev, zoneId: '' })); }}
            disabled={!!existingBridge}
            className={errors.zoneId ? 'field-error' : ''}
          >
            <option value="">--</option>
            {project.zones.map((z) => (
              <option key={z.id} value={z.id}>
                {z.name}
              </option>
            ))}
          </select>
          {errors.zoneId && <span className="field-error-text">{errors.zoneId}</span>}
        </div>
    </DialogShell>
  );
}
