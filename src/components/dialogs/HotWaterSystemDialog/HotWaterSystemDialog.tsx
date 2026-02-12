import { useState } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IHotWaterSystem, HotWaterSystemType } from '../../../core/energy/types';

interface HotWaterSystemDialogProps {
  editId?: string | null;
  onClose: () => void;
}

const hotWaterTypes: HotWaterSystemType[] = [
  'hr_combi', 'heat_pump', 'electric_boiler', 'solar_boiler', 'district_heating',
];

const hotWaterTypeLabels: Record<HotWaterSystemType, string> = {
  hr_combi: 'dialog.hotWater.hrCombi',
  heat_pump: 'dialog.hotWater.heatPump',
  electric_boiler: 'dialog.hotWater.electricBoiler',
  solar_boiler: 'dialog.hotWater.solarBoiler',
  district_heating: 'dialog.hotWater.districtHeating',
};

export function HotWaterSystemDialog({ editId, onClose }: HotWaterSystemDialogProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existing = editId
    ? state.project.hotWaterSystems.find((s) => s.id === editId)
    : null;

  const [name, setName] = useState(existing?.name ?? '');
  const [type, setType] = useState<HotWaterSystemType>(existing?.type ?? 'hr_combi');
  const [efficiency, setEfficiency] = useState(existing?.efficiency ?? 0.95);
  const [hasSolarBoiler, setHasSolarBoiler] = useState(existing?.hasSolarBoiler ?? false);
  const [solarFractionPercent, setSolarFractionPercent] = useState(
    (existing?.solarBoilerFraction ?? 0) * 100
  );

  const handleSave = () => {
    const system: IHotWaterSystem = {
      id: existing?.id ?? crypto.randomUUID(),
      name,
      type,
      efficiency,
      hasSolarBoiler,
      solarBoilerFraction: solarFractionPercent / 100,
    };

    if (existing) {
      dispatch({
        type: 'UPDATE_HOT_WATER_SYSTEM',
        payload: {
          id: existing.id,
          data: {
            name,
            type,
            efficiency,
            hasSolarBoiler,
            solarBoilerFraction: system.solarBoilerFraction,
          },
        },
      });
    } else {
      dispatch({ type: 'ADD_HOT_WATER_SYSTEM', payload: system });
    }
    onClose();
  };

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <div className="dialog-title">{t('dialog.hotWater.title')}</div>

        <div className="dialog-field">
          <label>{t('dialog.hotWater.name')}</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label>{t('dialog.hotWater.type')}</label>
          <select value={type} onChange={(e) => setType(e.target.value as HotWaterSystemType)}>
            {hotWaterTypes.map((ht) => (
              <option key={ht} value={ht}>
                {t(hotWaterTypeLabels[ht])}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label>{t('dialog.hotWater.efficiency')}</label>
          <input
            type="number"
            min={0}
            max={5}
            step={0.01}
            value={efficiency}
            onChange={(e) => setEfficiency(parseFloat(e.target.value) || 0)}
          />
        </div>

        <div className="dialog-field">
          <label style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <input
              type="checkbox"
              checked={hasSolarBoiler}
              onChange={(e) => setHasSolarBoiler(e.target.checked)}
              style={{ width: 'auto' }}
            />
            {t('dialog.hotWater.hasSolarBoiler')}
          </label>
        </div>

        {hasSolarBoiler && (
          <div className="dialog-field">
            <label>{t('dialog.hotWater.solarFraction')}</label>
            <input
              type="number"
              min={0}
              max={100}
              step={1}
              value={solarFractionPercent}
              onChange={(e) => setSolarFractionPercent(parseFloat(e.target.value) || 0)}
            />
          </div>
        )}

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
