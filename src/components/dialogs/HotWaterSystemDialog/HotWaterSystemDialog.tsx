import { useState, useId } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IHotWaterSystem, HotWaterSystemType } from '../../../core/energy/types';
import { fractionFromPercent, heatPumpDraftOf } from '../dialogValues';
import { DialogShell } from '../DialogShell';
import { defaultHeatPumpDraft, hasDhwDeclarationMismatch, hasElectricCarrierMismatch, hasIncompleteRegistryRecord, hasOperatingLimitEvidenceMismatch, HeatPumpMetadataFields, type HeatPumpDraft } from '../HeatPumpMetadataFields/HeatPumpMetadataFields';

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
  const fieldId = useId();
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
  const [classifyHeatPump, setClassifyHeatPump] = useState(Boolean(existing?.ntaHeatPump));
  const [heatPumpError, setHeatPumpError] = useState<string | null>(null);
  const [heatPumpDraft, setHeatPumpDraft] = useState<HeatPumpDraft>(existing?.ntaHeatPump
    ? heatPumpDraftOf(existing.ntaHeatPump)
    : defaultHeatPumpDraft('outdoor_air', 'domestic_hot_water'));

  const handleSave = () => {
    if (type === 'heat_pump' && classifyHeatPump && hasOperatingLimitEvidenceMismatch(heatPumpDraft)) {
      setHeatPumpError(t('kernel.operatingLimits.invalid'));
      return;
    }
    if (type === 'heat_pump' && classifyHeatPump && hasDhwDeclarationMismatch(heatPumpDraft)) {
      setHeatPumpError(t('kernel.dhwTest.invalid'));
      return;
    }
    if (type === 'heat_pump' && classifyHeatPump && hasElectricCarrierMismatch(heatPumpDraft)) {
      setHeatPumpError(t('kernel.points.carrierMismatch'));
      return;
    }
    if (type === 'heat_pump' && classifyHeatPump && hasIncompleteRegistryRecord(heatPumpDraft)) {
      setHeatPumpError(t('kernel.metadata.registryIncomplete'));
      return;
    }
    const id = existing?.id ?? crypto.randomUUID();
    const system: IHotWaterSystem = {
      id,
      name,
      type,
      efficiency,
      hasSolarBoiler,
      solarBoilerFraction: fractionFromPercent(solarFractionPercent, existing ? existing.solarBoilerFraction : 0),
      ntaHeatPump: type === 'heat_pump' && classifyHeatPump ? { id, ...heatPumpDraft } : undefined,
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
            ntaHeatPump: system.ntaHeatPump,
          },
        },
      });
    } else {
      dispatch({ type: 'ADD_HOT_WATER_SYSTEM', payload: system });
    }
    onClose();
  };

  return (
    <DialogShell variant="sheet"
      title={t('dialog.hotWater.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label htmlFor={`${fieldId}-1`}>{t('dialog.hotWater.name')}</label>
          <input id={`${fieldId}-1`}
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-2`}>{t('dialog.hotWater.type')}</label>
          <select id={`${fieldId}-2`} value={type} onChange={(e) => setType(e.target.value as HotWaterSystemType)}>
            {hotWaterTypes.map((ht) => (
              <option key={ht} value={ht}>
                {t(hotWaterTypeLabels[ht])}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-3`}>{t('dialog.hotWater.efficiency')}</label>
          <input id={`${fieldId}-3`}
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
            <label htmlFor={`${fieldId}-4`}>{t('dialog.hotWater.solarFraction')}</label>
            <input id={`${fieldId}-4`}
              type="number"
              min={0}
              max={100}
              step={1}
              value={solarFractionPercent}
              onChange={(e) => setSolarFractionPercent(parseFloat(e.target.value) || 0)}
            />
          </div>
        )}
        {type === 'heat_pump' && <>
          <div className="dialog-field">
            <label style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <input type="checkbox" checked={classifyHeatPump} onChange={(event) => setClassifyHeatPump(event.target.checked)} style={{ width: 'auto' }} />
              {t('kernel.metadata.enable')}
            </label>
          </div>
          {classifyHeatPump && <HeatPumpMetadataFields value={heatPumpDraft} onChange={setHeatPumpDraft} hotWaterOnly selfId={existing?.id} />}
          {classifyHeatPump && heatPumpError && <p role="alert">{heatPumpError}</p>}
        </>}
    </DialogShell>
  );
}
