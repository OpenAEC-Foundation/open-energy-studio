import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { ICoolingSystem, CoolingSystemType } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';

interface CoolingSystemDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const coolingTypes: CoolingSystemType[] = ['none', 'split_unit', 'central_chiller', 'heat_pump_reversible'];

const coolingTypeLabels: Record<CoolingSystemType, string> = {
  none: 'dialog.cooling.none',
  split_unit: 'dialog.cooling.splitUnit',
  central_chiller: 'dialog.cooling.centralChiller',
  heat_pump_reversible: 'dialog.cooling.heatPumpReversible',
};

export function CoolingSystemDialog({ editId, onClose }: CoolingSystemDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existing = editId
    ? state.project.coolingSystems.find((s) => s.id === editId)
    : null;

  const [name, setName] = useState(existing?.name ?? '');
  const [type, setType] = useState<CoolingSystemType>(existing?.type ?? 'split_unit');
  const [eer, setEer] = useState(existing?.eer ?? 3.0);

  const handleSave = () => {
    const system: ICoolingSystem = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      type,
      eer,
    };

    if (existing) {
      dispatch({
        type: 'UPDATE_COOLING_SYSTEM',
        payload: { id: existing.id, data: { name, type, eer } },
      });
    } else {
      dispatch({ type: 'ADD_COOLING_SYSTEM', payload: system });
    }
    onClose();
  };

  return (
    <DialogShell
      title={t('dialog.cooling.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label>{t('dialog.cooling.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.cooling.type')}</label>
          <select value={type} onChange={(e) => setType(e.target.value as CoolingSystemType)}>
            {coolingTypes.map((ct) => (
              <option key={ct} value={ct}>
                {t(coolingTypeLabels[ct])}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.cooling.eer')}</label>
          <input
            type="number"
            min={0}
            step={0.1}
            value={eer}
            onChange={(e) => setEer(parseFloat(e.target.value) || 0)}
          />
        </div>
    </DialogShell>
  );
}
