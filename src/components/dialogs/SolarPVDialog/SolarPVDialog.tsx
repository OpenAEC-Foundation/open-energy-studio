import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { ISolarPV, Orientation } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';

interface SolarPVDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const orientations: Orientation[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'];

export function SolarPVDialog({ editId, onClose }: SolarPVDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existing = editId
    ? state.project.solarPV.find((s) => s.id === editId)
    : null;

  const [name, setName] = useState(existing?.name ?? '');
  const [peakPower, setPeakPower] = useState(existing?.peakPower ?? 0);
  const [orientation, setOrientation] = useState<Orientation>(existing?.orientation ?? 'S');
  const [tilt, setTilt] = useState(existing?.tilt ?? 35);
  const [area, setArea] = useState(existing?.area ?? 0);

  const handleSave = () => {
    const pv: ISolarPV = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      peakPower,
      orientation,
      tilt,
      area,
    };

    if (existing) {
      dispatch({
        type: 'UPDATE_SOLAR_PV',
        payload: { id: existing.id, data: { name, peakPower, orientation, tilt, area } },
      });
    } else {
      dispatch({ type: 'ADD_SOLAR_PV', payload: pv });
    }
    onClose();
  };

  return (
    <DialogShell
      title={t('dialog.solarPV.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >

        <div className="dialog-field">
          <label>{t('dialog.solarPV.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.solarPV.peakPower')}</label>
          <input
            type="number"
            min={0}
            step={0.1}
            value={peakPower}
            onChange={(e) => setPeakPower(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.solarPV.orientation')}</label>
          <select value={orientation} onChange={(e) => setOrientation(e.target.value as Orientation)}>
            {orientations.map((o) => (
              <option key={o} value={o}>
                {t(`orientation.${o}`)}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.solarPV.tilt')}</label>
          <input
            type="number"
            min={0}
            max={90}
            step={1}
            value={tilt}
            onChange={(e) => setTilt(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.solarPV.area')}</label>
          <input
            type="number"
            min={0}
            step={0.1}
            value={area}
            onChange={(e) => setArea(parseFloat(e.target.value) || 0)}
          />
        </div>

    </DialogShell>
  );
}
