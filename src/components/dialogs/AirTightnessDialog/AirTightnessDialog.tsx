import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';

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

  const handleZoneChange = (newZoneId: string) => {
    setZoneId(newZoneId);
    const zone = project.zones.find((z) => z.id === newZoneId);
    if (zone) {
      setQv10(zone.airTightness.qv10);
    }
  };

  const handleSave = () => {
    if (!zoneId) return;

    dispatch({
      type: 'UPDATE_AIR_TIGHTNESS',
      payload: { zoneId, airTightness: { qv10 } },
    });
    onClose();
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">{t('dialog.airTightness.title')}</div>

        <div className="dialog-field">
          <label>{t('dialog.surface.zone')}</label>
          <select value={zoneId} onChange={(e) => handleZoneChange(e.target.value)}>
            <option value="">--</option>
            {project.zones.map((z) => (
              <option key={z.id} value={z.id}>
                {z.name}
              </option>
            ))}
          </select>
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
