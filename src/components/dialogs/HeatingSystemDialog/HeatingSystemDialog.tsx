import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IHeatingSystem, HeatingSystemType } from '../../../core/energy/types';
import { DialogShell } from '../DialogShell';

interface HeatingSystemDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const heatingTypes: HeatingSystemType[] = [
  'hr107', 'hr_combi', 'heat_pump_air', 'heat_pump_ground', 'district_heating', 'electric', 'biomass',
];

const heatingTypeLabels: Record<HeatingSystemType, string> = {
  hr107: 'dialog.heating.hr107',
  hr_combi: 'dialog.heating.hrCombi',
  heat_pump_air: 'dialog.heating.heatPumpAir',
  heat_pump_ground: 'dialog.heating.heatPumpGround',
  district_heating: 'dialog.heating.districtHeating',
  electric: 'dialog.heating.electric',
  biomass: 'dialog.heating.biomass',
};

const defaultCop: Record<HeatingSystemType, number> = {
  hr107: 0.95,
  hr_combi: 0.95,
  heat_pump_air: 4.0,
  heat_pump_ground: 5.0,
  district_heating: 0.9,
  electric: 1.0,
  biomass: 0.85,
};

export function HeatingSystemDialog({ editId, onClose }: HeatingSystemDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existing = editId
    ? state.project.heatingSystems.find((s) => s.id === editId)
    : null;

  const [name, setName] = useState(existing?.name ?? '');
  const [type, setType] = useState<HeatingSystemType>(existing?.type ?? 'hr107');
  const [cop, setCop] = useState(existing?.cop ?? defaultCop['hr107']);
  const [coveragePercent, setCoveragePercent] = useState((existing?.coverageFraction ?? 1) * 100);

  const handleTypeChange = (newType: HeatingSystemType) => {
    setType(newType);
    if (!existing) {
      setCop(defaultCop[newType]);
    }
  };

  const handleSave = () => {
    const system: IHeatingSystem = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      type,
      cop,
      coverageFraction: coveragePercent / 100,
    };

    if (existing) {
      dispatch({
        type: 'UPDATE_HEATING_SYSTEM',
        payload: { id: existing.id, data: { name, type, cop, coverageFraction: system.coverageFraction } },
      });
    } else {
      dispatch({ type: 'ADD_HEATING_SYSTEM', payload: system });
    }
    onClose();
  };

  return (
    <DialogShell
      title={t('dialog.heating.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label>{t('dialog.heating.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.heating.type')}</label>
          <select value={type} onChange={(e) => handleTypeChange(e.target.value as HeatingSystemType)}>
            {heatingTypes.map((ht) => (
              <option key={ht} value={ht}>
                {t(heatingTypeLabels[ht])}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.heating.cop')}</label>
          <input
            type="number"
            min={0}
            step={0.01}
            value={cop}
            onChange={(e) => setCop(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.heating.coverage')}</label>
          <input
            type="number"
            min={0}
            max={100}
            step={1}
            value={coveragePercent}
            onChange={(e) => setCoveragePercent(parseFloat(e.target.value) || 0)}
          />
        </div>
    </DialogShell>
  );
}
