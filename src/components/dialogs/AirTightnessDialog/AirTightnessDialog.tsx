import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { DialogShell } from '../DialogShell';

interface AirTightnessDialogProps {
  editId?: string | null;
  onClose: () => void;
}

export function AirTightnessDialog({ editId, onClose }: AirTightnessDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;

  // editId can be used as the zoneId to edit air tightness for
  const existingZone = editId
    ? project.zones.find((z) => z.id === editId)
    : null;

  const [zoneId, setZoneId] = useState(existingZone?.id ?? (project.zones[0]?.id ?? ''));
  const selectedZone = project.zones.find((z) => z.id === zoneId);
  const [qv10, setQv10] = useState(selectedZone?.airTightness.qv10 ?? 0.4);
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleZoneChange = (newZoneId: string) => {
    setZoneId(newZoneId);
    const zone = project.zones.find((z) => z.id === newZoneId);
    if (zone) {
      setQv10(zone.airTightness.qv10);
    }
  };

  const handleSave = () => {
    const newErrors: Record<string, string> = {};
    if (!zoneId) newErrors.zoneId = t('dialog.validation.required');
    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors);
      return;
    }
    setErrors({});

    dispatch({
      type: 'UPDATE_AIR_TIGHTNESS',
      payload: { zoneId, airTightness: { qv10 } },
    });
    onClose();
  };

  return (
    <DialogShell
      title={t('dialog.airTightness.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label>{t('dialog.surface.zone')}</label>
          <select
            value={zoneId}
            onChange={(e) => { handleZoneChange(e.target.value); setErrors(prev => ({ ...prev, zoneId: '' })); }}
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

        <div className="dialog-field">
          <label>{t('dialog.airTightness.qv10')}</label>
          <input
            type="number"
            min={0}
            step={0.01}
            value={qv10}
            onChange={(e) => setQv10(parseFloat(e.target.value) || 0)}
          />
        </div>
    </DialogShell>
  );
}
