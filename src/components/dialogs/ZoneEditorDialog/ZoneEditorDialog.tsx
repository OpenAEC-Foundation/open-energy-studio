import { useState, useId } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IZone } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';

interface ZoneEditorDialogProps {
  editId?: string | null;
  onClose: () => void;
}

export function ZoneEditorDialog({ editId, onClose }: ZoneEditorDialogProps) {
  const fieldId = useId();
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existingZone = editId
    ? state.project.zones.find((z) => z.id === editId)
    : null;

  const [name, setName] = useState(existingZone?.name ?? '');
  const [floorArea, setFloorArea] = useState(existingZone?.floorArea ?? 0);
  const [volume, setVolume] = useState(existingZone?.volume ?? 0);
  const [height, setHeight] = useState(existingZone?.height ?? 2.6);

  const handleSave = () => {
    if (existingZone) {
      dispatch({
        type: 'UPDATE_ZONE',
        payload: { id: existingZone.id, data: { name, floorArea, volume, height } },
      });
    } else {
      const newZone: IZone = {
        id: crypto.randomUUID(),
        name,
        floorArea,
        volume,
        height,
        surfaces: [],
        thermalBridges: [],
        pointThermalBridges: [],
        pointBridgeInventoryComplete: false,
        airTightness: { qv10: 0.4 },
      };
      dispatch({ type: 'ADD_ZONE', payload: newZone });
    }
    onClose();
  };

  return (
    <DialogShell variant="sheet"
      title={t('dialog.zone.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label htmlFor={`${fieldId}-1`}>{t('dialog.zone.name')}</label>
          <input id={`${fieldId}-1`}
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-2`}>{t('dialog.zone.floorArea')}</label>
          <input id={`${fieldId}-2`}
            type="number"
            min={0}
            step={0.1}
            value={floorArea}
            onChange={(e) => setFloorArea(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-3`}>{t('dialog.zone.volume')}</label>
          <input id={`${fieldId}-3`}
            type="number"
            min={0}
            step={0.1}
            value={volume}
            onChange={(e) => setVolume(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-4`}>{t('dialog.zone.height')}</label>
          <input id={`${fieldId}-4`}
            type="number"
            min={0}
            step={0.01}
            value={height}
            onChange={(e) => setHeight(parseFloat(e.target.value) || 0)}
          />
        </div>
    </DialogShell>
  );
}
