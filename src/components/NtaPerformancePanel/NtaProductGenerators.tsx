import { useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import {
  diagnoseDeclaredHeatingTableWithRust, type DeclaredHeatingTableAssessment, type DeclaredHeatingTableInput,
} from '../../core/nta/KernelClient';
import { withoutNulls } from '../../core/nta/KernelInput';
import {
  annexQModulatingTemplate, annexQPointTemplate, declaredHeatingPointTemplate, declaredHeatingRowTemplate,
  declaredHeatingTableTemplate, edgeInsulationTemplate, floorBelowTemplate, forfaitBoilerTemplate,
} from '../../core/nta/NtaSystemTemplates';
import {
  CheckField, NumberField, read, SelectField, TextField, write, type Draft, type Path,
} from './NtaFormFields';

// Generators with product data (annex Q heat pumps, annex M boilers, annex N
// heaters, table 9.25 heaters), the ground-floor details of §8.3 and the BCRG
// declaration table. The kernel validates; empty values stay null.

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

type Translate = (key: string) => string;

function list(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

function RemoveButton({ label, onClick }: { label: string; onClick: () => void }) {
  return <button type="button" className="nta-form-remove" onClick={onClick}>{label}</button>;
}

const opts = (t: Translate, prefix: string, keys: string[]): Array<[string, string]> =>
  keys.map((key) => [key, t(`${prefix}.${key}`)]);

/** Forfait gas boiler of table 9.25 (also the backup of an annex Q heat pump). */
export function BoilerForfaitFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  return <>
    <SelectField {...field} path={at('kind')} label={t('nta.form.boilerKind')} options={[
      ['hr107', 'HR107'], ['hr104', 'HR104'], ['hr100', 'HR100'], ['vr', 'VR'], ['conventional', t('nta.form.boiler.conventional')]]} />
    <SelectField {...field} path={at('location')} label={t('nta.form.boilerLocation')} options={[
      ['inside_thermal_boundary', t('nta.form.boiler.inside')], ['outside_thermal_boundary', t('nta.form.boiler.outside')]]} />
    <NumberField {...field} path={at('averageDesignEmissionTemperatureC')} label={t('nta.form.boilerTemperature')} />
    <SelectField {...field} path={at('emissionCircuit')} label={t('nta.form.boilerCircuit')} options={[
      ['direct', t('nta.form.boiler.direct')], ['mixing_with_return_limit', t('nta.form.boiler.mixingLimit')],
      ['mixing_without_return_limit', t('nta.form.boiler.mixingNoLimit')]]} />
    <TextField {...field} path={at('equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
    <TextField {...field} path={at('locationReference')} label={t('nta.form.boilerLocationSource')} />
    <TextField {...field} path={at('temperatureAndCircuitReference')} label={t('nta.form.boilerTemperatureSource')} />
  </>;
}

/** One measured point of tables Q.11/Q.15. */
function AnnexQPointFields({ draft, change, base, legend }: SectionProps & { base: Path; legend: string }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (key: string): Path => [...base, key];
  return <fieldset className="nta-form-row">
    <legend>{legend}</legend>
    <NumberField {...field} path={at('evaporatorInC')} label={t('nta.form.annexQ.evaporatorIn')} />
    <NumberField {...field} path={at('evaporatorOutC')} label={t('nta.form.annexQ.evaporatorOut')} />
    <NumberField {...field} path={at('condenserInC')} label={t('nta.form.annexQ.condenserIn')} />
    <NumberField {...field} path={at('condenserOutC')} label={t('nta.form.annexQ.condenserOut')} />
    <NumberField {...field} path={at('heatingPowerKw')} label={t('nta.form.annexQ.power')} />
    <NumberField {...field} path={at('cop')} label="COP" />
  </fieldset>;
}

const LOW_RANGE_LOADS = ['100 %', '88 %', '54 %', '35 %', '15 %'];

/** Annex Q heat pump: EN 14511/14825 product data, switch-off criteria, source and backup. */
export function AnnexQHeatPumpFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const hp = (...rest: Path): Path => at('heatPump', ...rest);
  const source = read(draft, hp('source'));
  const waterSource = source === 'brine_water' || source === 'water_water';
  const modulating = read(draft, hp('modulation', 'method')) === 'modulating';
  const backup = read(draft, at('backup', 'kind'));
  const optionalPoint = (condition: 'condition2' | 'condition3' | 'condition4') => {
    const present = read(draft, hp('maximumPower', condition)) != null;
    return <div key={condition}>
      <label className="nta-form-check">
        <input type="checkbox" checked={present}
          onChange={(event) => change(hp('maximumPower', condition), event.target.checked ? annexQPointTemplate() : null)} />
        {t(`nta.form.annexQ.${condition}`)}
      </label>
      {present && <AnnexQPointFields draft={draft} change={change} base={hp('maximumPower', condition)}
        legend={t(`nta.form.annexQ.${condition}`)} />}
    </div>;
  };
  const series = (range: 'lowRange' | 'highRange') => {
    const points = list(draft, hp('modulation', range));
    return <div className="nta-form-subsection">
      <strong>{t(`nta.form.annexQ.${range}`)}</strong>
      {points.map((_, index) => <div key={index}>
        <AnnexQPointFields draft={draft} change={change} base={hp('modulation', range, index)}
          legend={`${t('nta.form.annexQ.point')} ${LOW_RANGE_LOADS[index] ?? index + 1}`} />
        <RemoveButton label={t('nta.form.remove')}
          onClick={() => change(hp('modulation', range), points.filter((_, item) => item !== index))} />
      </div>)}
      <button type="button" className="btn"
        onClick={() => change(hp('modulation', range), [...points, annexQPointTemplate()])}>
        {t('nta.form.annexQ.addPoint')}
      </button>
    </div>;
  };
  return <div className="nta-form-subsection">
    <SelectField {...field} path={hp('source')} label={t('nta.form.annexQ.source')} options={opts(t, 'nta.form.annexQ.source',
      ['outdoor_air_water', 'brine_water', 'water_water', 'exhaust_air_water', 'combined_air_water', 'air_air'])} />
    {source === 'combined_air_water' &&
      <NumberField {...field} path={hp('outdoorAirFraction')} label={t('nta.form.annexQ.outdoorAirFraction')} />}
    <NumberField {...field} path={at('designSupplyTemperatureC')} label={t('nta.form.annexQ.supplyTemperature')} />
    <AnnexQPointFields draft={draft} change={change} base={hp('maximumPower', 'condition1')} legend={t('nta.form.annexQ.condition1')} />
    {optionalPoint('condition2')}
    {optionalPoint('condition3')}
    {optionalPoint('condition4')}
    <label>{t('nta.form.annexQ.modulation')}
      <select value={modulating ? 'modulating' : 'on_off'} onChange={(event) => change(hp('modulation'),
        event.target.value === 'modulating' ? annexQModulatingTemplate() : { method: 'on_off' })}>
        <option value="on_off">{t('nta.form.annexQ.onOff')}</option>
        <option value="modulating">{t('nta.form.annexQ.modulating')}</option>
      </select>
    </label>
    {modulating && <>
      <NumberField {...field} path={hp('modulation', 'minimumPowerKw')} label={t('nta.form.annexQ.minimumPower')} />
      <CheckField {...field} path={hp('modulation', 'condenserPumpModulating')} label={t('nta.form.annexQ.condenserPump')} />
      <CheckField {...field} path={hp('modulation', 'sourcePumpModulating')} label={t('nta.form.annexQ.sourcePumpModulating')} />
      {series('lowRange')}
      {source !== 'air_air' && <>
        <label className="nta-form-check">
          <input type="checkbox" checked={read(draft, hp('modulation', 'highRange')) != null}
            onChange={(event) => change(hp('modulation', 'highRange'),
              event.target.checked ? Array.from({ length: 5 }, annexQPointTemplate) : null)} />
          {t('nta.form.annexQ.highRangePresent')}
        </label>
        {read(draft, hp('modulation', 'highRange')) != null && series('highRange')}
      </>}
    </>}
    <fieldset className="nta-form-row">
      <legend>{t('nta.form.annexQ.switchOff')}</legend>
      <NumberField {...field} path={hp('switchOff', 'minEvaporatorInC')} label={t('nta.form.annexQ.minEvaporatorIn')} />
      <NumberField {...field} path={hp('switchOff', 'minEvaporatorOutC')} label={t('nta.form.annexQ.minEvaporatorOut')} />
      <NumberField {...field} path={hp('switchOff', 'maxCondenserInC')} label={t('nta.form.annexQ.maxCondenserIn')} />
      <NumberField {...field} path={hp('switchOff', 'maxCondenserOutC')} label={t('nta.form.annexQ.maxCondenserOut')} />
      <NumberField {...field} path={hp('switchOff', 'minCop')} label={t('nta.form.annexQ.minCop')} />
    </fieldset>
    <p className="nta-form-note">{t('nta.form.annexQ.switchOffNote')}</p>
    {waterSource && <>
      <label className="nta-form-check">
        <input type="checkbox" checked={read(draft, hp('sourcePump')) != null}
          onChange={(event) => change(hp('sourcePump'), event.target.checked ? { nominalPowerW: null, overrunS: null } : null)} />
        {t('nta.form.annexQ.sourcePump')}
      </label>
      {read(draft, hp('sourcePump')) != null && <>
        <NumberField {...field} path={hp('sourcePump', 'nominalPowerW')} label={t('nta.form.annexQ.sourcePumpPower')} />
        <NumberField {...field} path={hp('sourcePump', 'overrunS')} label={t('nta.form.annexQ.sourcePumpOverrun')} />
      </>}
      <label>{t('nta.form.annexQ.evaporatorInlet')}
        <select value={String(read(draft, hp('evaporatorInlet', 'method')) ?? '')} onChange={(event) => change(hp('evaporatorInlet'),
          event.target.value === 'declared' ? { method: 'declared', temperaturesC: [], sourceReference: '' }
            : event.target.value === 'forfait' ? { method: 'forfait' } : null)}>
          <option value="">—</option>
          <option value="forfait">{t('nta.form.annexQ.inletForfait')}</option>
          <option value="declared">{t('nta.form.annexQ.inletDeclared')}</option>
        </select>
      </label>
      {read(draft, hp('evaporatorInlet', 'method')) === 'declared' && <>
        <label>{t('nta.form.annexQ.inletTemperatures')}
          <input value={list(draft, hp('evaporatorInlet', 'temperaturesC')).join('; ')}
            onChange={(event) => change(hp('evaporatorInlet', 'temperaturesC'), event.target.value.split(';')
              .map((item) => item.trim().replace(',', '.')).filter((item) => item !== '').map(Number))} />
        </label>
        <TextField {...field} path={hp('evaporatorInlet', 'sourceReference')} label={t('nta.form.source')} />
      </>}
    </>}
    <label>{t('nta.form.annexQ.backup')}
      <select value={typeof backup === 'string' ? backup : ''} onChange={(event) => change(at('backup'),
        event.target.value === 'electric_resistance' ? { kind: 'electric_resistance' }
          : event.target.value === 'gas_boiler' ? { kind: 'gas_boiler', boiler: forfaitBoilerTemplate('backup') } : null)}>
        <option value="">{t('nta.form.annexQ.backupNone')}</option>
        <option value="electric_resistance">{t('nta.form.generator.electric')}</option>
        <option value="gas_boiler">{t('nta.form.generator.boiler')}</option>
      </select>
    </label>
    {backup === 'gas_boiler' && <BoilerForfaitFields draft={draft} change={change} base={at('backup', 'boiler')} />}
    <TextField {...field} path={hp('testReportReference')} label={t('nta.form.annexQ.testReport')} />
    <TextField {...field} path={at('equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
  </div>;
}

/** Annex M boiler with product values. */
export function ProductBoilerFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const b = (...rest: Path): Path => [...base, 'boiler', ...rest];
  const p = (...rest: Path): Path => b('product', ...rest);
  const condensing = read(draft, p('fullLoad', 'method')) === 'condensing';
  return <div className="nta-form-subsection">
    <SelectField {...field} path={b('technology')} label={t('nta.form.productBoiler.technology')} options={opts(t,
      'nta.form.productBoiler.technology', ['condensing_gas', 'condensing_oil', 'low_temperature', 'gas_oil_standard', 'solid_fuel_standard'])} />
    <SelectField {...field} path={b('fuel')} label={t('nta.form.productBoiler.fuel')} options={opts(t,
      'nta.form.productBoiler.fuel', ['natural_gas', 'oil', 'wood'])} />
    <SelectField {...field} path={b('placement')} label={t('nta.form.productBoiler.placement')} options={opts(t,
      'nta.form.productBoiler.placement', ['heated_space', 'installation_room', 'under_roof', 'outdoors'])} />
    <SelectField {...field} path={b('draught')} label={t('nta.form.productBoiler.draught')} options={opts(t,
      'nta.form.productBoiler.draught', ['fan_assisted', 'atmospheric'])} />
    <SelectField {...field} path={b('control')} label={t('nta.form.productBoiler.control')} options={opts(t,
      'nta.form.productBoiler.control', ['wall_hung_room_temperature', 'wall_hung_outdoor_compensated', 'floor_standing_outdoor_compensated'])} />
    <NumberField {...field} path={p('nominalPowerKw')} label={t('nta.form.productBoiler.nominalPower')} />
    <NumberField {...field} path={p('intermediatePowerKw')} label={t('nta.form.productBoiler.intermediatePower')} />
    <label>{t('nta.form.productBoiler.fullLoad')}
      <select value={condensing ? 'condensing' : 'single'} onChange={(event) => change(p('fullLoad'),
        event.target.value === 'condensing' ? { method: 'condensing', efficiencyAt60: null, efficiencyAt30: null }
          : { method: 'single', efficiency: null, testTemperatureC: null })}>
        <option value="single">{t('nta.form.productBoiler.fullLoad.single')}</option>
        <option value="condensing">{t('nta.form.productBoiler.fullLoad.condensing')}</option>
      </select>
    </label>
    {condensing ? <>
      <NumberField {...field} path={p('fullLoad', 'efficiencyAt60')} label={t('nta.form.productBoiler.efficiencyAt60')} />
      <NumberField {...field} path={p('fullLoad', 'efficiencyAt30')} label={t('nta.form.productBoiler.efficiencyAt30')} />
    </> : <>
      <NumberField {...field} path={p('fullLoad', 'efficiency')} label={t('nta.form.productBoiler.fullEfficiency')} />
      <NumberField {...field} path={p('fullLoad', 'testTemperatureC')} label={t('nta.form.productBoiler.testTemperature')} />
    </>}
    <NumberField {...field} path={p('partLoadEfficiency')} label={t('nta.form.productBoiler.partLoadEfficiency')} />
    <NumberField {...field} path={p('partLoadTestTemperatureC')} label={t('nta.form.productBoiler.partLoadTemperature')} />
    <NumberField {...field} path={p('standbyLossFactor')} label={t('nta.form.productBoiler.standbyLoss')} />
    <NumberField {...field} path={p('standbyTestTemperatureC')} label={t('nta.form.productBoiler.standbyTemperature')} />
    <NumberField {...field} path={p('auxiliaryFullW')} label={t('nta.form.productBoiler.auxFull')} />
    <NumberField {...field} path={p('auxiliaryIntermediateW')} label={t('nta.form.productBoiler.auxIntermediate')} />
    <NumberField {...field} path={p('auxiliaryStandbyW')} label={t('nta.form.productBoiler.auxStandby')} />
    <TextField {...field} path={p('sourceReference')} label={t('nta.form.productBoiler.productSource')} />
    <SelectField {...field} path={[...base, 'designTemperatureClass']} label={t('nta.form.productBoiler.designClass')}
      options={['30_27', '35_30', '40_35', '45_40', '50_42', '55_47', '60_50', '65_55', '70_60', '75_65', '80_60', '90_70']
        .map((key) => [key, key.replace('_', '/')])} />
    <TextField {...field} path={b('equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
  </div>;
}

const N6_PRODUCT_KEYS = [
  'inputFullKw', 'outputFullKw', 'combustionEfficiencyPercent', 'chimneyLossPercent', 'envelopeLossPercent',
  'pilotLossPercent', 'auxBurnerKw', 'auxAfterBurnerKw', 'auxStandbyKw', 'inputMinKw',
];

/** Annex N local, air or radiant heater or stove. */
export function LocalHeaterFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const h = (...rest: Path): Path => [...base, 'heater', ...rest];
  const type = String(read(draft, h('heaterType')) ?? '');
  return <div className="nta-form-subsection">
    <SelectField {...field} path={h('heaterType')} label={t('nta.form.localHeater.type')} options={opts(t, 'nta.form.localHeater.type', [
      'air_heater_atmospheric', 'air_heater_fan_burner', 'air_heater_modulating_combustion_air',
      'air_heater_modulating_no_combustion_air', 'air_heater_modulating_evaporative', 'condensing_air_heater',
      'high_temperature_radiant', 'radiant_tube_without_flue', 'radiant_tube_with_flue', 'stove'])} />
    <SelectField {...field} path={[...base, 'fuel']} label={t('nta.form.productBoiler.fuel')} options={opts(t,
      'nta.form.productBoiler.fuel', ['natural_gas', 'oil', 'biomass'])} />
    <SelectField {...field} path={h('control')} label={t('nta.form.localHeater.control')} options={opts(t,
      'nta.form.localHeater.control', ['on_off', 'high_low', 'modulating'])} />
    <SelectField {...field} path={h('productionPeriod')} label={t('nta.form.localHeater.period')} options={opts(t,
      'nta.form.localHeater.period', ['after2005', 'from1990_to2005', 'before1990'])} />
    <SelectField {...field} path={h('ventilation')} label={t('nta.form.localHeater.ventilation')} options={opts(t,
      'nta.form.localHeater.ventilation', ['required', 'interlocked', 'none'])} />
    <SelectField {...field} path={h('location')} label={t('nta.form.localHeater.location')} options={opts(t,
      'nta.form.localHeater.location', ['heated_space_free', 'heated_space_against_wall_or_roof', 'boiler_room',
        'under_roof_outside_heated_space', 'outdoors'])} />
    <CheckField {...field} path={h('condensing')} label={t('nta.form.localHeater.condensing')} />
    <CheckField {...field} path={h('pilotFlame')} label={t('nta.form.localHeater.pilotFlame')} />
    {type.startsWith('air_heater') || type === 'condensing_air_heater' ? <>
      <SelectField {...field} path={h('fan')} label={t('nta.form.localHeater.fan')} options={opts(t,
        'nta.form.localHeater.fan', ['centrifugal', 'axial'])} />
      <SelectField {...field} path={h('envelopeInsulation')} label={t('nta.form.localHeater.envelope')} options={opts(t,
        'nta.form.localHeater.envelope', ['well_insulated_new_high_efficiency', 'well_insulated_maintained', 'old_average',
          'old_poor', 'none'])} />
    </> : null}
    {type === 'stove' && <SelectField {...field} path={h('stoveKind')} label={t('nta.form.localHeater.stoveKind')} options={opts(t,
      'nta.form.localHeater.stoveKind', ['solid_fuel_room_heater', 'inset_or_open_fire', 'pellet', 'accumulating', 'gas_or_oil'])} />}
    {type.includes('radiant') && <NumberField {...field} path={h('roomHeightM')} label={t('nta.form.localHeater.roomHeight')} />}
    <fieldset className="nta-form-row">
      <legend>{t('nta.form.localHeater.product')}</legend>
      {N6_PRODUCT_KEYS.map((key) => <NumberField key={key} {...field} path={h('product', key)}
        label={t(`nta.form.localHeater.product.${key}`)} />)}
    </fieldset>
    <p className="nta-form-note">{t('nta.form.localHeater.note')}</p>
    <TextField {...field} path={h('sourceReference')} label={t('nta.form.source')} />
  </div>;
}

/** Table 9.25 "overige systemen": local heaters and gas air heaters without product data. */
export function ForfaitHeaterFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const kind = String(read(draft, at('heaterKind')) ?? '');
  return <>
    <SelectField {...field} path={at('heaterKind')} label={t('nta.form.forfaitHeater.kind')} options={opts(t, 'nta.form.forfaitHeater.kind', [
      'local_with_flue', 'local_without_flue', 'air_heater_conventional', 'air_heater_vr', 'air_heater_hr100',
      'air_heater_hr104', 'air_heater_hr107'])} />
    <SelectField {...field} path={at('fuel')} label={t('nta.form.productBoiler.fuel')} options={opts(t,
      'nta.form.productBoiler.fuel', ['natural_gas', 'oil'])} />
    {kind.startsWith('air_heater') &&
      <NumberField {...field} path={at('pilotFlames')} label={t('nta.form.forfaitHeater.pilotFlames')} step="1" />}
    <TextField {...field} path={at('equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
  </>;
}

/** §8.3: edge insulation, a crawlspace or unheated basement below, or a heated basement. */
export function GroundFloorDetailFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const below = read(draft, at('below', 'kind'));
  const basement = read(draft, at('heatedBasement')) != null;
  const edges = list(draft, at('edgeInsulation'));
  return <>
    <label>{t('nta.form.ground.below')}
      <select value={typeof below === 'string' ? below : ''} onChange={(event) => change(at('below'),
        event.target.value === 'crawlspace' || event.target.value === 'unheated_basement'
          ? floorBelowTemplate(event.target.value) : null)}>
        <option value="">{t('nta.form.ground.below.none')}</option>
        <option value="crawlspace">{t('nta.form.ground.below.crawlspace')}</option>
        <option value="unheated_basement">{t('nta.form.ground.below.unheatedBasement')}</option>
      </select>
    </label>
    {below != null && <>
      <NumberField {...field} path={at('below', 'floorResistanceM2kPerW')} label={t('nta.form.ground.belowFloorResistance')} />
      <SelectField {...field} path={at('below', 'depthClass')} label={t('nta.form.ground.depthClass')} options={[
        ['other', t('nta.form.ground.depthClass.other')], ['on_sand', t('nta.form.ground.depthClass.onSand')]]} />
      <NumberField {...field} path={at('below', 'wallResistanceM2kPerW')} label={t('nta.form.ground.wallResistance')} />
      <NumberField {...field} path={at('below', 'wallUValueWPerM2k')} label={t('nta.form.ground.wallUValue')} />
      {below === 'crawlspace'
        ? <NumberField {...field} path={at('below', 'ventilationOpeningM2PerM')} label={t('nta.form.ground.ventilationOpening')} />
        : <>
          <NumberField {...field} path={at('below', 'volumeM3')} label={t('nta.form.ground.basementVolume')} />
          <NumberField {...field} path={at('below', 'airChangesPerHour')} label={t('nta.form.ground.airChanges')} />
        </>}
    </>}
    <label className="nta-form-check">
      <input type="checkbox" checked={basement} onChange={(event) => change(at('heatedBasement'),
        event.target.checked ? { depthM: null, wallResistanceM2kPerW: null } : null)} />
      {t('nta.form.ground.heatedBasement')}
    </label>
    {basement && <>
      <NumberField {...field} path={at('heatedBasement', 'depthM')} label={t('nta.form.ground.basementDepth')} />
      <NumberField {...field} path={at('heatedBasement', 'wallResistanceM2kPerW')} label={t('nta.form.ground.basementWallResistance')} />
    </>}
    {edges.map((_, index) => <div key={index} className="nta-form-row">
      <SelectField {...field} path={at('edgeInsulation', index, 'kind')} label={t('nta.form.ground.edgeInsulation')} options={[
        ['horizontal', t('nta.form.ground.edge.horizontal')], ['vertical', t('nta.form.ground.edge.vertical')]]} />
      <NumberField {...field} path={at('edgeInsulation', index, 'resistanceM2kPerW')} label={t('nta.form.ground.edgeResistance')} />
      <NumberField {...field} path={at('edgeInsulation', index, 'thicknessM')} label={t('nta.form.ground.edgeThickness')} />
      <TextField {...field} path={at('edgeInsulation', index, 'sourceReference')} label={t('nta.form.source')} />
      <RemoveButton label={t('nta.form.remove')} onClick={() => change(at('edgeInsulation'), edges.filter((_, item) => item !== index))} />
    </div>)}
    {below == null && !basement && <button type="button" className="btn"
      onClick={() => change(at('edgeInsulation'), [...edges, edgeInsulationTemplate()])}>{t('nta.form.ground.addEdgeInsulation')}</button>}
  </>;
}

/** BCRG declaration table: the kernel interpolates η, F and W_aux; the result is informative. */
export function DeclaredHeatingTableTool({ diagnose = diagnoseDeclaredHeatingTableWithRust }: {
  diagnose?: (input: DeclaredHeatingTableInput) => Promise<DeclaredHeatingTableAssessment>;
}) {
  const { t } = useI18n();
  const [draft, setDraft] = useState<Draft>(() => declaredHeatingTableTemplate());
  const [result, setResult] = useState<DeclaredHeatingTableAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const field = { draft, onChange: change };
  const rows = list(draft, ['rows']);
  const run = async () => {
    setError(null);
    try {
      setResult(await diagnose(withoutNulls(draft) as unknown as DeclaredHeatingTableInput));
    } catch (failure) {
      setResult(null);
      setError(failure instanceof Error ? failure.message : String(failure));
    }
  };
  return <div className="nta-form-subsection" data-testid="declared-heating-table">
    <TextField {...field} path={['declarationId']} label={t('nta.form.bcrg.code')} />
    <TextField {...field} path={['declarationNormVersion']} label={t('nta.form.bcrg.normVersion')} />
    <TextField {...field} path={['tableScope']} label={t('nta.form.bcrg.scope')} />
    <TextField {...field} path={['sourceReference']} label={t('nta.form.source')} />
    <NumberField {...field} path={['grossHeatDemandKwhPerYear']} label={t('nta.form.bcrg.demand')} />
    <NumberField {...field} path={['designSupplyTemperatureC']} label={t('nta.form.bcrg.supplyTemperature')} />
    <CheckField {...field} path={['firstRowCoversLowerTemperatures']} label={t('nta.form.bcrg.firstRowLower')} />
    {rows.map((row, rowIndex) => {
      const points = list(row, ['points']);
      return <fieldset key={rowIndex} className="nta-form-row">
        <legend>{t('nta.form.bcrg.row')} {rowIndex + 1}</legend>
        <NumberField {...field} path={['rows', rowIndex, 'supplyTemperatureC']} label={t('nta.form.bcrg.rowTemperature')} />
        {points.map((_, index) => <div key={index} className="nta-form-row">
          <NumberField {...field} path={['rows', rowIndex, 'points', index, 'grossHeatDemandKwhPerYear']} label={t('nta.form.bcrg.demand')} />
          <NumberField {...field} path={['rows', rowIndex, 'points', index, 'generationEfficiency']} label={t('nta.form.bcrg.efficiency')} />
          <NumberField {...field} path={['rows', rowIndex, 'points', index, 'preferredEnergyFraction']} label={t('nta.form.bcrg.fraction')} />
          <NumberField {...field} path={['rows', rowIndex, 'points', index, 'auxiliaryElectricityKwhPerYear']} label={t('nta.form.bcrg.auxiliary')} />
          <RemoveButton label={t('nta.form.remove')}
            onClick={() => change(['rows', rowIndex, 'points'], points.filter((_, item) => item !== index))} />
        </div>)}
        <button type="button" className="btn" onClick={() => change(['rows', rowIndex, 'points'],
          [...points, declaredHeatingPointTemplate()])}>{t('nta.form.bcrg.addPoint')}</button>
        <RemoveButton label={t('nta.form.remove')} onClick={() => change(['rows'], rows.filter((_, item) => item !== rowIndex))} />
      </fieldset>;
    })}
    <button type="button" className="btn" onClick={() => change(['rows'], [...rows, declaredHeatingRowTemplate()])}>
      {t('nta.form.bcrg.addRow')}
    </button>
    <button type="button" className="btn" onClick={() => { void run(); }}>{t('nta.form.bcrg.diagnose')}</button>
    {error && <p className="nta-form-error">{error}</p>}
    {result && <p role="status">
      {result.status === 'input_valid'
        ? `η ${result.generationEfficiency ?? '–'} · F ${result.preferredEnergyFraction ?? '–'} · W_aux ${result.auxiliaryElectricityKwhPerYear ?? '–'} kWh`
        : `${t('nta.form.bcrg.invalid')}: ${result.issues.map((issue) => `${issue.code} (${issue.path})`).join(', ')}`}
    </p>}
    <p className="nta-form-note">{t('nta.form.bcrg.note')}</p>
  </div>;
}
