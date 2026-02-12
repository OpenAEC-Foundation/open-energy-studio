import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { ISolarThermal, SolarThermalType, Orientation } from '../../../core/energy/types';

interface SolarThermalDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const collectorTypes: SolarThermalType[] = ['flat_plate', 'vacuum_tube'];

const collectorTypeLabels: Record<SolarThermalType, string> = {
  flat_plate: 'dialog.solarThermal.flatPlate',
  vacuum_tube: 'dialog.solarThermal.vacuumTube',
};

const orientations: Orientation[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'];

export function SolarThermalDialog({ editId, onClose }: SolarThermalDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existing = editId
    ? state.project.solarThermal.find((s) => s.id === editId)
    : null;

  const [name, setName] = useState(existing?.name ?? '');
  const [collectorArea, setCollectorArea] = useState(existing?.collectorArea ?? 0);
  const [type, setType] = useState<SolarThermalType>(existing?.type ?? 'flat_plate');
  const [orientation, setOrientation] = useState<Orientation>(existing?.orientation ?? 'S');
  const [tilt, setTilt] = useState(existing?.tilt ?? 45);

  const handleSave = () => {
    const solar: ISolarThermal = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      collectorArea,
      type,
      orientation,
      tilt,
    };

    if (existing) {
      dispatch({
        type: 'UPDATE_SOLAR_THERMAL',
        payload: { id: existing.id, data: { name, collectorArea, type, orientation, tilt } },
      });
    } else {
      dispatch({ type: 'ADD_SOLAR_THERMAL', payload: solar });
    }
    onClose();
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">{t('dialog.solarThermal.title')}</div>

        <div className="dialog-field">
          <label>{t('dialog.solarThermal.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.solarThermal.collectorArea')}</label>
          <input
            type="number"
            min={0}
            step={0.1}
            value={collectorArea}
            onChange={(e) => setCollectorArea(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.solarThermal.type')}</label>
          <select value={type} onChange={(e) => setType(e.target.value as SolarThermalType)}>
            {collectorTypes.map((ct) => (
              <option key={ct} value={ct}>
                {t(collectorTypeLabels[ct])}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.solarThermal.orientation')}</label>
          <select value={orientation} onChange={(e) => setOrientation(e.target.value as Orientation)}>
            {orientations.map((o) => (
              <option key={o} value={o}>
                {t(`orientation.${o}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.solarThermal.tilt')}</label>
          <input
            type="number"
            min={0}
            max={90}
            step={1}
            value={tilt}
            onChange={(e) => setTilt(parseFloat(e.target.value) || 0)}
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
