import { useState, useId } from 'react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { IHeatingSystem, HeatingSystemType } from '../../../core/energy/types';
import { fractionFromPercent, heatPumpDraftOf } from '../dialogValues';
import { DialogShell } from '../DialogShell';
import { defaultHeatPumpDraft, hasDhwDeclarationMismatch, hasElectricCarrierMismatch, hasIncompleteRegistryRecord, hasOperatingLimitEvidenceMismatch, HeatPumpMetadataFields, type HeatPumpDraft } from '../HeatPumpMetadataFields/HeatPumpMetadataFields';

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
const isHeatPumpType = (value: HeatingSystemType) => value === 'heat_pump_air' || value === 'heat_pump_ground';

export function HeatingSystemDialog({ editId, onClose }: HeatingSystemDialogProps) {
  const fieldId = useId();
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const existing = editId
    ? state.project.heatingSystems.find((s) => s.id === editId)
    : null;

  const [name, setName] = useState(existing?.name ?? '');
  const [type, setType] = useState<HeatingSystemType>(existing?.type ?? 'hr107');
  const [cop, setCop] = useState(existing?.cop ?? defaultCop['hr107']);
  const [coveragePercent, setCoveragePercent] = useState((existing?.coverageFraction ?? 1) * 100);
  const [classifyHeatPump, setClassifyHeatPump] = useState(Boolean(existing?.ntaHeatPump));
  const [heatPumpError, setHeatPumpError] = useState<string | null>(null);
  const [heatPumpDraft, setHeatPumpDraft] = useState<HeatPumpDraft>(existing?.ntaHeatPump
    ? heatPumpDraftOf(existing.ntaHeatPump)
    : defaultHeatPumpDraft(existing?.type === 'heat_pump_ground' ? 'ground' : 'outdoor_air', 'hydronic'));

  const handleTypeChange = (newType: HeatingSystemType) => {
    setType(newType);
    if (isHeatPumpType(newType)) {
      setHeatPumpDraft((draft) => ({ ...draft, source: newType === 'heat_pump_ground' ? 'ground' : 'outdoor_air' }));
    }
    if (!existing) {
      setCop(defaultCop[newType]);
    }
  };

  const handleSave = () => {
    if (isHeatPumpType(type) && classifyHeatPump && hasOperatingLimitEvidenceMismatch(heatPumpDraft)) {
      setHeatPumpError(t('kernel.operatingLimits.invalid'));
      return;
    }
    if (isHeatPumpType(type) && classifyHeatPump && hasDhwDeclarationMismatch(heatPumpDraft)) {
      setHeatPumpError(t('kernel.dhwTest.invalid'));
      return;
    }
    if (isHeatPumpType(type) && classifyHeatPump && hasElectricCarrierMismatch(heatPumpDraft)) {
      setHeatPumpError(t('kernel.points.carrierMismatch'));
      return;
    }
    if (isHeatPumpType(type) && classifyHeatPump && hasIncompleteRegistryRecord(heatPumpDraft)) {
      setHeatPumpError(t('kernel.metadata.registryIncomplete'));
      return;
    }
    const id = existing?.id ?? crypto.randomUUID();
    const system: IHeatingSystem = {
      id,
      name,
      type,
      cop,
      coverageFraction: fractionFromPercent(coveragePercent, existing?.coverageFraction ?? 1),
      ntaHeatPump: isHeatPumpType(type) && classifyHeatPump ? { id, ...heatPumpDraft } : undefined,
    };

    if (existing) {
      dispatch({
        type: 'UPDATE_HEATING_SYSTEM',
        payload: { id: existing.id, data: { name, type, cop, coverageFraction: system.coverageFraction, ntaHeatPump: system.ntaHeatPump } },
      });
    } else {
      dispatch({ type: 'ADD_HEATING_SYSTEM', payload: system });
    }
    onClose();
  };

  return (
    <DialogShell variant="sheet"
      title={t('dialog.heating.title')}
      onClose={onClose}
      onSubmit={handleSave}
      submitLabel={t('dialog.save')}
      cancelLabel={t('dialog.cancel')}
    >
        <div className="dialog-field">
          <label htmlFor={`${fieldId}-1`}>{t('dialog.heating.name')}</label>
          <input id={`${fieldId}-1`}
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-2`}>{t('dialog.heating.type')}</label>
          <select id={`${fieldId}-2`} value={type} onChange={(e) => handleTypeChange(e.target.value as HeatingSystemType)}>
            {heatingTypes.map((ht) => (
              <option key={ht} value={ht}>
                {t(heatingTypeLabels[ht])}
              </option>
            ))}
          </select>
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-3`}>{t('dialog.heating.cop')}</label>
          <input id={`${fieldId}-3`}
            type="number"
            min={0}
            step={0.01}
            value={cop}
            onChange={(e) => { const value = parseFloat(e.target.value) || 0; if (Number.isFinite(value) && value >= 0) setCop(value); }}
          />
        </div>

        <div className="dialog-field">
          <label htmlFor={`${fieldId}-4`}>{t('dialog.heating.coverage')}</label>
          <input id={`${fieldId}-4`}
            type="number"
            min={0}
            max={100}
            step={1}
            value={coveragePercent}
            onChange={(e) => { const value = parseFloat(e.target.value) || 0; if (Number.isFinite(value) && value >= 0 && value <= 100) setCoveragePercent(value); }}
          />
        </div>
        {isHeatPumpType(type) && <>
          <div className="dialog-field">
            <label style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <input type="checkbox" checked={classifyHeatPump} onChange={(event) => setClassifyHeatPump(event.target.checked)} style={{ width: 'auto' }} />
              {t('kernel.metadata.enable')}
            </label>
          </div>
          {classifyHeatPump && <HeatPumpMetadataFields value={heatPumpDraft} onChange={setHeatPumpDraft} selfId={existing?.id} />}
          {classifyHeatPump && heatPumpError && <p role="alert">{heatPumpError}</p>}
        </>}
    </DialogShell>
  );
}
