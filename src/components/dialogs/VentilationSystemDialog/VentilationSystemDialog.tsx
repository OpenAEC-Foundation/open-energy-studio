import { useState, useId } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IVentilationSystem, VentilationType } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';

interface VentilationSystemDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const ventilationTypes: VentilationType[] = ['natural', 'type_c', 'type_d'];

const ventilationTypeLabels: Record<VentilationType, string> = {
  natural: 'dialog.ventilation.natural',
  type_c: 'dialog.ventilation.typeC',
  type_d: 'dialog.ventilation.typeD',
};

export function VentilationSystemDialog({ editId, onClose }: VentilationSystemDialogProps) {
  const fieldId = useId();
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existing = editId
    ? state.project.ventilationSystems.find((s) => s.id === editId)
    : null;

  const [name, setName] = useState(existing?.name ?? '');
  const [type, setType] = useState<VentilationType>(existing?.type ?? 'natural');
  const [heatRecoveryPercent, setHeatRecoveryPercent] = useState(
    (existing?.heatRecoveryEfficiency ?? 0) * 100
  );
  const [sfp, setSfp] = useState(existing?.sfp ?? 0);

  const handleSave = () => {
    const system: IVentilationSystem = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      type,
      heatRecoveryEfficiency: heatRecoveryPercent / 100,
      sfp,
    };

    if (existing) {
      dispatch({
        type: 'UPDATE_VENTILATION_SYSTEM',
        payload: {
          id: existing.id,
          data: { name, type, heatRecoveryEfficiency: system.heatRecoveryEfficiency, sfp },
        },
      });
    } else {
      dispatch({ type: 'ADD_VENTILATION_SYSTEM', payload: system });
    }
    onClose();
  };

  return (
    <DialogShell
      title={t('dialog.ventilation.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label htmlFor={`${fieldId}-1`}>{t('dialog.ventilation.name')}</label>
          <input id={`${fieldId}-1`}
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-2`}>{t('dialog.ventilation.type')}</label>
          <select id={`${fieldId}-2`} value={type} onChange={(e) => setType(e.target.value as VentilationType)}>
            {ventilationTypes.map((vt) => (
              <option key={vt} value={vt}>
                {t(ventilationTypeLabels[vt])}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-3`}>{t('dialog.ventilation.heatRecovery')}</label>
          <input id={`${fieldId}-3`}
            type="number"
            min={0}
            max={100}
            step={1}
            value={heatRecoveryPercent}
            onChange={(e) => setHeatRecoveryPercent(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-4`}>{t('dialog.ventilation.sfp')}</label>
          <input id={`${fieldId}-4`}
            type="number"
            min={0}
            step={0.1}
            value={sfp}
            onChange={(e) => setSfp(parseFloat(e.target.value) || 0)}
          />
        </div>
    </DialogShell>
  );
}
