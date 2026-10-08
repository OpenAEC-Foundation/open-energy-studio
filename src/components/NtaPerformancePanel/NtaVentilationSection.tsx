import { useI18n } from '../../i18n/i18n';
import {
  heatRecoveryTemplate, VENTILATION_VARIANTS, ventilationOp, ventilationUnit,
} from '../../core/nta/NtaFormModels';
import type { VentilationSystemVariant } from '../../core/nta/KernelClient';
import {
  CheckField, NumberField, read, Section, SelectField, TextField, type Draft, type Path,
} from './NtaFormFields';

const AIRTIGHTNESS_TYPES = [
  'pitched_roof_terraced', 'pitched_roof_end_or_corner', 'pitched_roof_detached', 'pitched_roof_detached_partly_flat',
  'flat_roof_terraced', 'flat_roof_end_or_corner', 'flat_roof_detached',
  'storey_middle_lower_or_intermediate', 'storey_end_lower_or_intermediate', 'storey_middle_top', 'storey_end_top',
  'multi_storey_whole_building', 'multi_storey_whole_top_layer', 'multi_storey_whole_intermediate_layer',
  'multi_storey_whole_bottom_layer',
] as const;

const APPLIANCES = [
  'gas_geyser', 'gas_open_atmospheric', 'gas_bath_geyser', 'gas_boiler_storage', 'gas_central_heating_boiler',
  'gas_fire_natural_flue_type_i', 'gas_fire_mechanical_flue_type_i_i', 'gas_fire_type_i_i', 'open_fireplace_solid_fuel',
  'gas_stove', 'oil_stove', 'coal_stove', 'solid_fuel_stove',
] as const;

const APPLIANCE_CLASSES = ['gas_type_a', 'gas_type_b', 'room_sealed', 'kitchen_stove', 'open_fireplace', 'specific_gas_appliance'] as const;

const EXCHANGERS = [
  'none', 'run_around_coil_ahu', 'plate_or_tube', 'cross_flow', 'two_element', 'heat_pipe', 'rotary', 'enthalpy',
  'counter_flow_aluminium', 'counter_flow_plastic',
] as const;

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

/** One system unit of table 11.5 at `path`. */
export function VentilationUnitFields(props: SectionProps & { path: Path }) {
  return <UnitFields {...props} />;
}

function UnitFields({ draft, change, path }: SectionProps & { path: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const unit = (read(draft, path) as Draft | undefined) ?? {};
  const op = ventilationOp(unit.variant);
  const recovery = read(draft, [...path, 'heatRecovery']) as Draft | undefined;
  const ahu = read(draft, [...path, 'airHandlingUnit']) as Draft | undefined;
  const declared = read(draft, [...path, 'declaredControlFactor']) as Draft | undefined;
  const efficiencyMethod = read(draft, [...path, 'heatRecovery', 'efficiency', 'method']);
  const bypass = read(draft, [...path, 'heatRecovery', 'bypass', 'kind']);
  const insulation = read(draft, [...path, 'heatRecovery', 'supplyDuctInsulation', 'kind']);
  return <>
    <SelectField {...field} path={[...path, 'variant']} label={t('nta.vent.variant')}
      options={VENTILATION_VARIANTS.map(({ variant }) => [variant, t(`nta.vent.variant.${variant}`)])}
      onChange={(_, value) => change(path, ventilationUnit((value as VentilationSystemVariant | null) ?? null, unit))} />
    {op && op !== 'natural' && <SelectField {...field} path={[...path, 'ducts']} label={t('nta.vent.ducts')} options={[
      ['unknown', t('nta.vent.ducts.unknown')], ['luka_a_b_c', t('nta.vent.ducts.abc')], ['luka_d', t('nta.vent.ducts.d')],
      ['no_ducts', t('nta.vent.ducts.none')]]} />}
    <TextField {...field} path={[...path, 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
    {/* E.1 fixes f_ctrl of the decentral D.5b part (11.51/11.52). */}
    {unit.variant != null && path[path.length - 1] !== 'decentral' && <label className="nta-form-check">
      <input type="checkbox" checked={declared != null}
        onChange={(event) => change([...path, 'declaredControlFactor'],
          event.target.checked ? { value: null, declarationReference: '' } : undefined)} />
      {t('nta.form.ventilation.declaredControlFactor')}
    </label>}
    {declared != null && <>
      <p className="nta-form-note">{t('nta.form.ventilation.declaredControlFactorHelp')}</p>
      <NumberField {...field} path={[...path, 'declaredControlFactor', 'value']} label="f_ctrl" />
      <TextField {...field} path={[...path, 'declaredControlFactor', 'declarationReference']}
        label={t('nta.form.ventilation.declarationReference')} />
    </>}
    {op === 'balanced' && <label className="nta-form-check">
      <input type="checkbox" checked={recovery != null}
        onChange={(event) => change([...path, 'heatRecovery'], event.target.checked ? heatRecoveryTemplate() : undefined)} />
      {t('nta.vent.heatRecovery')}
    </label>}
    {op === 'balanced' && recovery != null && <>
      <SelectField {...field} path={[...path, 'heatRecovery', 'efficiency', 'method']} label={t('nta.vent.hrMethod')}
        options={[['declared', t('nta.vent.hrDeclared')], ['table', t('nta.vent.hrTable')]]}
        onChange={(_, value) => change([...path, 'heatRecovery', 'efficiency'], value === 'table'
          ? { method: 'table', exchanger: null }
          : { method: 'declared', value: null, standard: 'en13141_7', sourceReference: '' })} />
      {efficiencyMethod === 'table'
        ? <SelectField {...field} path={[...path, 'heatRecovery', 'efficiency', 'exchanger']} label={t('nta.vent.exchanger')}
          options={EXCHANGERS.map((key) => [key, t(`nta.vent.exchanger.${key}`)])} />
        : <>
          <NumberField {...field} path={[...path, 'heatRecovery', 'efficiency', 'value']} label={t('nta.vent.hrValue')} />
          <SelectField {...field} path={[...path, 'heatRecovery', 'efficiency', 'standard']} label={t('nta.vent.hrStandard')}
            options={[['en13141_7', 'NEN-EN 13141-7'], ['en13141_8', 'NEN-EN 13141-8'], ['en13142', 'NEN-EN 13142'], ['en13053', 'NEN-EN 13053']]} />
          <TextField {...field} path={[...path, 'heatRecovery', 'efficiency', 'sourceReference']} label={t('nta.form.source')} />
        </>}
      <SelectField {...field} path={[...path, 'heatRecovery', 'bypass', 'kind']} label={t('nta.vent.bypass')} options={[
        ['none', t('nta.vent.bypass.none')], ['full', t('nta.vent.bypass.full')], ['partial', t('nta.vent.bypass.partial')],
        ['unknown', t('nta.vent.bypass.unknown')]]}
        onChange={(_, value) => change([...path, 'heatRecovery', 'bypass'], value === 'partial'
          ? { kind: value, fraction: null } : value === 'unknown' ? { kind: value, bypassPresent: false } : { kind: value ?? 'none' })} />
      {bypass === 'partial' && <NumberField {...field} path={[...path, 'heatRecovery', 'bypass', 'fraction']} label={t('nta.vent.bypassFraction')} />}
      {bypass === 'unknown' && <CheckField {...field} path={[...path, 'heatRecovery', 'bypass', 'bypassPresent']} label={t('nta.vent.bypassPresent')} />}
      {bypass === 'full' && <label>{t('nta.vent.coldRecovery')}
        <input value={String(read(draft, [...path, 'heatRecovery', 'bypass', 'coldRecoveryEvidence']) ?? '')}
          onChange={(event) => change([...path, 'heatRecovery', 'bypass', 'coldRecoveryEvidence'], event.target.value || undefined)} />
      </label>}
      <SelectField {...field} path={[...path, 'heatRecovery', 'layout']} label={t('nta.vent.layout')}
        options={[['central', t('nta.vent.layout.central')], ['decentral', t('nta.vent.layout.decentral')]]} />
      <CheckField {...field} path={[...path, 'heatRecovery', 'constantVolumeControl']} label={t('nta.vent.constantVolume')} />
      <NumberField {...field} path={[...path, 'heatRecovery', 'supplyDuctLengthM']} label={t('nta.vent.supplyDuctLength')} />
      <SelectField {...field} path={[...path, 'heatRecovery', 'supplyDuctInsulation', 'kind']} label={t('nta.vent.supplyDuctInsulation')}
        options={[['unknown', t('nta.form.unknown')], ['insulated', t('nta.vent.insulated')], ['uninsulated', t('nta.vent.uninsulated')],
          ['specified', t('nta.vent.specified')]]}
        onChange={(_, value) => change([...path, 'heatRecovery', 'supplyDuctInsulation'], value === 'specified'
          ? { kind: value, thicknessM: null, conductivityWPerMK: null } : { kind: value ?? 'unknown' })} />
      {insulation === 'specified' && <>
        <NumberField {...field} path={[...path, 'heatRecovery', 'supplyDuctInsulation', 'thicknessM']} label={t('nta.vent.insulationThickness')} />
        <NumberField {...field} path={[...path, 'heatRecovery', 'supplyDuctInsulation', 'conductivityWPerMK']} label="λ W/(m·K)" />
      </>}
      <NumberField {...field} path={[...path, 'heatRecovery', 'manufactureYear']} label={t('nta.vent.manufactureYear')} step="1" />
      <TextField {...field} path={[...path, 'heatRecovery', 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
    </>}
    {(op === 'balanced' || op === 'supply') && <label className="nta-form-check">
      <input type="checkbox" checked={ahu != null}
        onChange={(event) => change([...path, 'airHandlingUnit'], event.target.checked
          ? { insideThermalZone: true, supplyDuctsOutside: 'none' } : undefined)} />
      {t('nta.vent.ahu')}
    </label>}
    {ahu != null && <>
      <CheckField {...field} path={[...path, 'airHandlingUnit', 'insideThermalZone']} label={t('nta.vent.ahuInside')} />
      <SelectField {...field} path={[...path, 'airHandlingUnit', 'supplyDuctsOutside']} label={t('nta.vent.ahuDucts')} options={[
        ['none', t('nta.vent.ahuDucts.none')], ['situation1', t('nta.vent.ahuDucts.1')], ['situation2', t('nta.vent.ahuDucts.2')],
        ['situation3', t('nta.vent.ahuDucts.3')]]} />
      <CheckField {...field} path={[...path, 'airHandlingUnit', 'heatingCoil']} label={t('nta.vent.ahuHeatingCoil')} />
      <CheckField {...field} path={[...path, 'airHandlingUnit', 'coolingCoil']} label={t('nta.vent.ahuCoolingCoil')} />
    </>}
  </>;
}

/** Optional text that is omitted, not sent empty, when cleared. */
function OptionalText({ draft, change, path, label }: SectionProps & { path: Path; label: string }) {
  return <label>{label}
    <input value={String(read(draft, path) ?? '')} onChange={(event) => change(path, event.target.value || undefined)} />
  </label>;
}

export function NtaVentilationSection({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const base: Path = ['ventilation'];
  const systemKind = read(draft, [...base, 'system', 'kind']);
  const infiltration = read(draft, [...base, 'infiltration', 'method']);
  const appliances = (read(draft, [...base, 'combustionAppliances']) as Draft[] | undefined) ?? [];
  const cooling = read(draft, [...base, 'ventilativeCooling']) as Draft | undefined;
  const openings = (cooling?.openings as Draft[] | undefined) ?? [];
  const preheating = read(draft, [...base, 'grillePreheating']) as Draft | undefined;
  const fansMethod = read(draft, [...base, 'fans', 'method']);
  const fans = (read(draft, [...base, 'fans', 'fans']) as Draft[] | undefined) ?? [];
  const installed = read(draft, [...base, 'installedCapacity']) as Draft | undefined;
  const utility = read(draft, ['calculationScope']) === 'utility';

  return <>
    <Section title={t('nta.vent.title')}>
      <p className="nta-form-note">{t('nta.vent.note')}</p>
      <NumberField {...field} path={[...base, 'buildingHeightM']} label={t('nta.vent.height')} />
      <NumberField {...field} path={[...base, 'constructionYear']} label={t('nta.vent.constructionYear')} step="1" />
      <CheckField {...field} path={[...base, 'floorAboveCrawlspace']} label={t('nta.vent.crawlspace')} />
      <TextField {...field} path={[...base, 'sourceReference']} label={t('nta.form.source')} />
    </Section>
    <Section title={t('nta.vent.system')}>
      <SelectField {...field} path={[...base, 'system', 'kind']} label={t('nta.vent.systemKind')}
        options={[['single', t('nta.vent.single')], ['combined', t('nta.vent.combined')]]}
        onChange={(_, value) => change([...base, 'system'], value === 'combined'
          ? { kind: 'combined', decentralAreaM2: null, totalResidenceAreaM2: null,
            decentral: { ...ventilationUnit('d5b'), heatRecovery: heatRecoveryTemplate() }, other: ventilationUnit('c1') }
          : { kind: 'single', unit: ventilationUnit('c1') })} />
      {systemKind === 'combined' ? <>
        <NumberField {...field} path={[...base, 'system', 'decentralAreaM2']} label={t('nta.vent.decentralArea')} />
        <NumberField {...field} path={[...base, 'system', 'totalResidenceAreaM2']} label={t('nta.vent.totalResidenceArea')} />
        <p className="nta-form-note">{t('nta.vent.decentralPart')}</p>
        <UnitFields draft={draft} change={change} path={[...base, 'system', 'decentral']} />
        <p className="nta-form-note">{t('nta.vent.otherPart')}</p>
        <UnitFields draft={draft} change={change} path={[...base, 'system', 'other']} />
      </> : <UnitFields draft={draft} change={change} path={[...base, 'system', 'unit']} />}
      <OptionalText draft={draft} change={change} path={[...base, 'maximumCapacityForCooling']} label={t('nta.vent.maxCapacity')} />
      <label className="nta-form-check">
        <input type="checkbox" checked={installed != null} onChange={(event) => change([...base, 'installedCapacity'],
          event.target.checked ? { totalDm3PerS: null, naturalSupplyDm3PerS: 0, sourceReference: '' } : undefined)} />
        {t('nta.vent.installed')}
      </label>
      {installed != null && <>
        <NumberField {...field} path={[...base, 'installedCapacity', 'totalDm3PerS']} label={t('nta.vent.installedTotal')} />
        <NumberField {...field} path={[...base, 'installedCapacity', 'naturalSupplyDm3PerS']} label={t('nta.vent.installedNatural')} />
        <TextField {...field} path={[...base, 'installedCapacity', 'sourceReference']} label={t('nta.form.source')} />
      </>}
      {utility && <>
        <CheckField {...field} path={[...base, 'flowReduction', 'collective']} label={t('nta.vent.collective')} />
        <NumberField {...field} path={[...base, 'flowReduction', 'recirculationPercent']} label={t('nta.vent.recirculation')} step="10" />
        <NumberField {...field} path={[...base, 'flowReduction', 'flowControlPercent']} label={t('nta.vent.flowControl')} step="10" />
        <TextField {...field} path={[...base, 'flowReduction', 'evidenceReference']} label={t('nta.vent.flowReductionEvidence')} />
      </>}
    </Section>
    <Section title={t('nta.vent.infiltration')}>
      <SelectField {...field} path={[...base, 'infiltration', 'method']} label={t('nta.vent.infiltrationMethod')}
        options={[['measured', t('nta.vent.measured')], ['reference', t('nta.vent.reference')]]}
        onChange={(_, value) => change([...base, 'infiltration'], value === 'measured'
          ? { method: 'measured', qv10DmPerSM2: null, sourceReference: '' } : { method: 'reference', buildingType: null })} />
      {infiltration === 'measured' ? <>
        <NumberField {...field} path={[...base, 'infiltration', 'qv10DmPerSM2']} label={t('nta.vent.qv10')} />
        <TextField {...field} path={[...base, 'infiltration', 'sourceReference']} label={t('nta.form.source')} />
      </> : <>
        <SelectField {...field} path={[...base, 'infiltration', 'buildingType']} label={t('nta.vent.buildingType')}
          options={AIRTIGHTNESS_TYPES.map((key) => [key, t(`nta.vent.type.${key}`)])} />
        <NumberField {...field} path={[...base, 'infiltration', 'renovationYear']} label={t('nta.vent.renovationYear')} step="1" />
      </>}
    </Section>
    <Section title={t('nta.vent.combustion')}>
      {appliances.map((item, index) => <div key={String(item.id)} className="nta-form-row">
        <SelectField {...field} path={[...base, 'combustionAppliances', index, 'kind']} label={t('nta.vent.appliance')}
          options={APPLIANCES.map((key) => [key, t(`nta.vent.appliance.${key}`)])} />
        <SelectField {...field} path={[...base, 'combustionAppliances', index, 'class']} label={t('nta.vent.applianceClass')}
          options={APPLIANCE_CLASSES.map((key) => [key, t(`nta.vent.applianceClass.${key}`)])} />
        <NumberField {...field} path={[...base, 'combustionAppliances', index, 'nominalInputKw']} label={t('nta.vent.nominalInput')} />
        <TextField {...field} path={[...base, 'combustionAppliances', index, 'sourceReference']} label={t('nta.form.source')} />
        <button type="button" onClick={() => change([...base, 'combustionAppliances'], appliances.filter((_, i) => i !== index))}>
          {t('nta.vent.remove')}</button>
      </div>)}
      <button type="button" onClick={() => change([...base, 'combustionAppliances'], [...appliances, {
        id: `appliance-${appliances.length + 1}`, kind: null, class: null, sourceReference: '' }])}>{t('nta.vent.addAppliance')}</button>
      <p className="nta-form-note">{t('nta.vent.combustionNote')}</p>
    </Section>
    <Section title={t('nta.vent.cooling')}>
      <label className="nta-form-check">
        <input type="checkbox" checked={cooling != null} onChange={(event) => change([...base, 'ventilativeCooling'],
          event.target.checked ? { openings: [], operation: 'manual', conditionsEvidence: '' } : undefined)} />
        {t('nta.vent.coolingPresent')}
      </label>
      {cooling != null && <>
        <SelectField {...field} path={[...base, 'ventilativeCooling', 'operation']} label={t('nta.vent.operation')} options={[
          ['manual', t('nta.vent.operation.manual')], ['automatic', t('nta.vent.operation.automatic')],
          ['automatic_with_temperature', t('nta.vent.operation.temperature')]]} />
        <TextField {...field} path={[...base, 'ventilativeCooling', 'conditionsEvidence']} label={t('nta.vent.conditions')} />
        {openings.map((item, index) => <div key={String(item.id)} className="nta-form-row">
          <NumberField {...field} path={[...base, 'ventilativeCooling', 'openings', index, 'area', 'netAreaM2']} label={`${String(item.id)} — ${t('nta.vent.netArea')}`} />
          <NumberField {...field} path={[...base, 'ventilativeCooling', 'openings', index, 'centreHeightM']} label={t('nta.vent.centreHeight')} />
          <NumberField {...field} path={[...base, 'ventilativeCooling', 'openings', index, 'openingHeightM']} label={t('nta.vent.openingHeight')} />
          <NumberField {...field} path={[...base, 'ventilativeCooling', 'openings', index, 'azimuthDeg']} label={t('nta.form.pvAzimuth')} />
          <NumberField {...field} path={[...base, 'ventilativeCooling', 'openings', index, 'tiltDeg']} label={t('nta.form.tilt')} />
          <button type="button" onClick={() => change([...base, 'ventilativeCooling', 'openings'], openings.filter((_, i) => i !== index))}>
            {t('nta.vent.remove')}</button>
        </div>)}
        <button type="button" onClick={() => change([...base, 'ventilativeCooling', 'openings'], [...openings, {
          id: `opening-${openings.length + 1}`, area: { method: 'declared', netAreaM2: null },
          centreHeightM: null, openingHeightM: null, azimuthDeg: null, tiltDeg: 90 }])}>{t('nta.vent.addOpening')}</button>
      </>}
    </Section>
    <Section title={t('nta.vent.preheating')}>
      <label className="nta-form-check">
        <input type="checkbox" checked={preheating != null} onChange={(event) => change([...base, 'grillePreheating'],
          event.target.checked ? { control: { method: 'fallback' }, sourceReference: '' } : undefined)} />
        {t('nta.vent.preheatingPresent')}
      </label>
      {preheating != null && <>
        <SelectField {...field} path={[...base, 'grillePreheating', 'control', 'method']} label={t('nta.vent.preheatingControl')}
          options={[['fallback', t('nta.vent.preheating.fallback')], ['specified', t('nta.vent.preheating.specified')]]}
          onChange={(_, value) => change([...base, 'grillePreheating', 'control'], value === 'specified'
            ? { method: 'specified', maxPowerWPerDm3PerS: null, maxTemperatureRiseK: null, switchOnBelowC: null, maxSupplyTemperatureC: null }
            : { method: 'fallback' })} />
        {read(draft, [...base, 'grillePreheating', 'control', 'method']) === 'specified' && <>
          <NumberField {...field} path={[...base, 'grillePreheating', 'control', 'maxPowerWPerDm3PerS']} label={t('nta.vent.preheatPower')} />
          <NumberField {...field} path={[...base, 'grillePreheating', 'control', 'maxTemperatureRiseK']} label={t('nta.vent.preheatRise')} />
          <NumberField {...field} path={[...base, 'grillePreheating', 'control', 'switchOnBelowC']} label={t('nta.vent.preheatOn')} />
          <NumberField {...field} path={[...base, 'grillePreheating', 'control', 'maxSupplyTemperatureC']} label={t('nta.vent.preheatMax')} />
        </>}
        <NumberField {...field} path={[...base, 'grillePreheating', 'preheatedDesignFlowM3PerH']} label={t('nta.vent.preheatFlow')} />
        <TextField {...field} path={[...base, 'grillePreheating', 'sourceReference']} label={t('nta.form.source')} />
      </>}
    </Section>
    <Section title={t('nta.vent.fans')}>
      <SelectField {...field} path={[...base, 'fans', 'method']} label={t('nta.vent.fansMethod')}
        options={[['forfait', t('nta.vent.fans.forfait')], ['declared', t('nta.vent.fans.declared')]]}
        onChange={(_, value) => change([...base, 'fans'], value === 'declared'
          ? { method: 'declared', fans: [{ id: 'fan-1', power: { method: 'nominal', nominalPowerW: null } }],
            control: { method: utility ? 'flow_control' : 'residential_table', ...(utility ? { control: 'other' } : {}) },
            buildingShare: 1, sourceReference: '' }
          : { method: 'forfait', current: 'dc', manufactureYear: null })} />
      {fansMethod === 'declared' ? <>
        {fans.map((item, index) => <div key={String(item.id)} className="nta-form-row">
          <NumberField {...field} path={[...base, 'fans', 'fans', index, 'power', 'nominalPowerW']} label={`${String(item.id)} — ${t('nta.vent.fanPower')}`} />
          <button type="button" onClick={() => change([...base, 'fans', 'fans'], fans.filter((_, i) => i !== index))}>{t('nta.vent.remove')}</button>
        </div>)}
        <button type="button" onClick={() => change([...base, 'fans', 'fans'], [...fans, {
          id: `fan-${fans.length + 1}`, power: { method: 'nominal', nominalPowerW: null } }])}>{t('nta.vent.addFan')}</button>
        <SelectField {...field} path={[...base, 'fans', 'control', 'method']} label={t('nta.vent.fanControl')}
          options={[['residential_table', t('nta.vent.fanControl.table')], ['flow_control', t('nta.vent.fanControl.flow')]]}
          onChange={(_, value) => change([...base, 'fans', 'control'], value === 'flow_control'
            ? { method: 'flow_control', control: 'other' } : { method: 'residential_table' })} />
        {read(draft, [...base, 'fans', 'control', 'method']) === 'flow_control' &&
          <SelectField {...field} path={[...base, 'fans', 'control', 'control']} label={t('nta.vent.flowControlMethod')} options={[
            ['throttle', t('nta.vent.flowControl.throttle')], ['inlet_vane_or_blade_pitch', t('nta.vent.flowControl.vane')],
            ['speed_control', t('nta.vent.flowControl.speed')], ['other', t('nta.vent.flowControl.other')]]} />}
        <NumberField {...field} path={[...base, 'fans', 'buildingShare']} label={t('nta.vent.buildingShare')} />
        <TextField {...field} path={[...base, 'fans', 'sourceReference']} label={t('nta.form.source')} />
      </> : <>
        <SelectField {...field} path={[...base, 'fans', 'current']} label={t('nta.vent.fanCurrent')}
          options={[['dc', t('nta.vent.fanCurrent.dc')], ['ac', t('nta.vent.fanCurrent.ac')]]} />
        <NumberField {...field} path={[...base, 'fans', 'manufactureYear']} label={t('nta.vent.manufactureYear')} step="1" />
      </>}
    </Section>
  </>;
}
