import { useRef, useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import {
  CheckField, NumberField, read, Section, SelectField, TextField, TriStateField, write, type Draft, type Path,
} from '../NtaPerformancePanel/NtaFormFields';
import {
  assessResidentialSurveyWithRust, assessUtilitySurveyWithRust, type OpnameAssessment,
} from '../../core/nta/KernelClient';
import {
  asResidential, asUtility, calculationZoneTemplate, heatingGeneratorTemplate, hotWaterGeneratorTemplate, pvTemplate,
  solarTemplate, surveyTemplate, windowTemplate, type StoredSurvey, type SurveyKind,
} from '../../core/nta/SurveyTemplates';
import { KernelCode } from '../KernelCode/KernelCode';
import { formatNumber } from '../../i18n/format';
import { dutchDefaultValue, snakeCase } from '../../core/nta/OpnameValueText';
import '../NtaPerformancePanel/NtaPerformancePanel.css';
import './BasisopnamePanel.css';

// ISSO 82.1 (dwellings) and 75.1 (utility) basisopname: structured fields for
// the main survey sections, a JSON view for the rest, and the kernel route
// that derives the NTA 8800 input with the applied defaults (ISSO pages).

type Change = (path: Path, value: unknown) => void;
type T = (key: string, options?: Record<string, unknown>) => string;

const HEATING_KINDS = ['boiler', 'heat_pump', 'local_fired', 'gas_air_heater', 'district_heat', 'electric', 'biomass',
  'chp', 'none_present'];
const HEAT_PUMP_SOURCES = ['outdoor_air', 'exhaust_air', 'outdoor_and_exhaust_air', 'heat_pump_panel', 'ground',
  'groundwater', 'surface_water', 'high_temperature', 'water_based_unknown'];
const WATER_SOURCES = ['ground', 'groundwater', 'surface_water', 'high_temperature', 'water_based_unknown'];
const HOT_WATER_KINDS: Record<SurveyKind, string[]> = {
  residential: ['gas_appliance', 'electric_boiler', 'electric_instantaneous', 'heat_pump', 'district_heat',
    'collective_unknown', 'delivery_set_from_heating', 'none'],
  utility: ['gas_appliance', 'gas_storage_heater', 'electric_boiler', 'electric_instantaneous', 'heat_pump',
    'district_heat', 'collective_unknown', 'none'],
};
const COOLING_GENERATORS = ['room_air_conditioner', 'compression', 'gas_engine_compression', 'gas_absorption', 'external_cold',
  'unknown_collective', 'aquifer_before2013', 'aquifer_from2013', 'aquifer_year_unknown', 'surface_water', 'closed_ground_loop',
  'dew_point_cooling'];
const HEAT_RECOVERY = ['counter_flow_aluminium', 'counter_flow_plastic', 'counter_flow_unknown_material', 'cross_flow',
  'plate_or_tube', 'rotary', 'enthalpy', 'heat_pipe', 'two_element', 'cold_storage_with_ahu', 'unknown'];
const COOLING_EMITTERS = ['split_indoor_units_on_wall', 'split_indoor_units_on_ceiling', 'fan_coil_on_outer_wall',
  'fan_coil_on_ceiling', 'floor_cooling', 'concrete_core_activation', 'wall_cooling', 'ceiling_cooling', 'other'];
const UTILITY_FUNCTIONS = ['office', 'assembly_without_day_care', 'assembly_with_day_care', 'education',
  'healthcare_without_beds', 'healthcare_with_beds', 'retail', 'sport', 'lodging', 'cell'];
const EMITTERS = ['radiators', 'low_temperature_radiators', 'floor_heating', 'floor_heating_and_radiators', 'air_heating',
  'local_heaters'];
const AIR_HEATING_KINDS = ['direct', 'indirect', 'via_air_handling_unit'];
const ORIENTATIONS = ['north', 'north_east', 'east', 'south_east', 'south', 'south_west', 'west', 'north_west'];

function opts(t: T, prefix: string, keys: string[]): Array<[string, string]> {
  return keys.map((key) => [key, t(`${prefix}.${key}`)]);
}

function KindSelect({ draft, path, label, kinds, prefix, template, change, t }: {
  draft: Draft; path: Path; label: string; kinds: string[]; prefix: string;
  template: (kind: string) => Record<string, unknown>; change: Change; t: T;
}) {
  const value = read(draft, [...path, 'kind']);
  return <label>{label}
    <select value={typeof value === 'string' ? value : ''} onChange={(event) => change(path, template(event.target.value))}>
      {kinds.map((kind) => <option key={kind} value={kind}>{t(`${prefix}.${kind}`)}</option>)}
    </select>
  </label>;
}

/** ISSO table 9.16: emitters, and with air heating the air-heater type (unknown: null). */
export function EmitterFields({ draft, change, t }: { draft: Draft; change: Change; t: T }) {
  const field = { draft, onChange: change };
  const emitters = read(draft, ['heating', 'emitters']);
  const air = read(draft, ['heating', 'airHeating', 'kind']);
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  return <>
    <label>{t('opname.heating.emitters')}
      <select value={typeof emitters === 'string' ? emitters : ''} onChange={(event) => change(['heating'], {
        ...(read(draft, ['heating']) as Draft), emitters: event.target.value,
        airHeating: event.target.value === 'air_heating' ? read(draft, ['heating', 'airHeating']) ?? null : undefined,
      })}>
        {EMITTERS.map((key) => <option key={key} value={key}>{t(`opname.heating.emitters.${key}`)}</option>)}
      </select>
    </label>
    {emitters === 'air_heating' && <>
      <SelectField {...field} path={['heating', 'airHeating', 'kind']} label={t('opname.airHeating')}
        options={opts(t, 'opname.airHeating.kind', AIR_HEATING_KINDS)}
        onChange={(_, value) => change(['heating', 'airHeating'], value == null ? null
          : value === 'direct' ? { kind: value, radialFan: null, count: null }
            : value === 'indirect' ? { kind: value, roomHeightAbove8M: null, warmAirReturn: null, ecMotor: null, count: null }
              : { kind: value })} />
      {air === 'direct' && <TriStateField {...field} {...yesNo} path={['heating', 'airHeating', 'radialFan']} label={t('opname.airHeating.radialFan')} />}
      {air === 'indirect' && <>
        <TriStateField {...field} {...yesNo} path={['heating', 'airHeating', 'roomHeightAbove8M']} label={t('opname.airHeating.above8m')} />
        <TriStateField {...field} {...yesNo} path={['heating', 'airHeating', 'warmAirReturn']} label={t('opname.airHeating.warmAirReturn')} />
        <TriStateField {...field} {...yesNo} path={['heating', 'airHeating', 'ecMotor']} label={t('opname.airHeating.ecMotor')} />
      </>}
      {(air === 'direct' || air === 'indirect') &&
        <NumberField {...field} path={['heating', 'airHeating', 'count']} label={t('opname.airHeating.count')} step="1" />}
    </>}
  </>;
}

/** ISSO §9.4.2 and table 9.12: one- or two-pipe system and the pipe insulation (unknown: not insulated). */
export function DistributionFields({ draft, change, t }: { draft: Draft; change: Change; t: T }) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const type = read(draft, ['heating', 'distributionType', 'kind']);
  const insulation = read(draft, ['heating', 'pipeInsulation']);
  const insulated = read(draft, ['heating', 'pipeInsulation', 'insulated']) === true;
  return <>
    <SelectField {...field} path={['heating', 'distributionType', 'kind']} label={t('opname.heating.distributionType')}
      options={opts(t, 'opname.heating.distributionTypeKind', ['two_pipe', 'one_pipe', 'renovated_one_pipe'])}
      onChange={(_, value) => change(['heating', 'distributionType'], value == null ? null
        : value === 'one_pipe' ? { kind: value, emitterCount: 1 } : { kind: value })} />
    {type === 'one_pipe' && <NumberField {...field} path={['heating', 'distributionType', 'emitterCount']}
      label={t('opname.heating.emitterCount')} step="1" />}
    <TriStateField {...field} {...yesNo} path={['heating', 'pipeInsulation', 'insulated']} label={t('opname.heating.pipesInsulated')}
      onChange={(_, value) => change(['heating', 'pipeInsulation'], value == null ? null
        : { ...(insulation as Draft | null ?? {}), insulated: value })} />
    {insulated && <NumberField {...field} path={['heating', 'pipeInsulation', 'insulationYear']}
      label={t('opname.heating.insulationYear')} step="1" />}
    {insulation != null && <TriStateField {...field} {...yesNo} path={['heating', 'pipeInsulation', 'fittingsInsulated']}
      label={t('opname.heating.fittingsInsulated')} />}
  </>;
}

/** ISSO §11.4.1/§11.5.6: passive cooling proven by a supplier project document. */
export function PassiveCoolingFields({ draft, change, t }: { draft: Draft; change: Change; t: T }) {
  const field = { draft, onChange: change };
  const present = read(draft, ['ventilation', 'passiveCooling']) != null;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={present} onChange={(event) => change(['ventilation', 'passiveCooling'],
        event.target.checked ? { evidenceReference: '', installedCapacityDm3PerS: null } : null)} />
      {t('opname.passiveCooling.present')}
    </label>
    {present && <>
      <TextField {...field} path={['ventilation', 'passiveCooling', 'evidenceReference']} label={t('opname.passiveCooling.evidence')} />
      <NumberField {...field} path={['ventilation', 'passiveCooling', 'installedCapacityDm3PerS']} label={t('opname.passiveCooling.installed')} />
    </>}
    <p className="nta-form-note">{t('opname.passiveCooling.note')}</p>
  </>;
}

const CO2_MEASUREMENTS = ['none', 'living_room', 'living_room_and_main_bedroom', 'every_habitable_room'];
const CONTROL_TARGETS = ['none', 'supply', 'extract', 'supply_and_extract'];

/**
 * ISSO 82.1 §11.3 (p. 143–146): controls of tables 11.4–11.6, central or
 * decentral heat recovery, system E and grilles with heating strips.
 */
export function VentilationSurveyFields({ draft, change, t, withControls = true }: {
  draft: Draft; change: Change; t: T;
  /** The utility survey has no table 11.4–11.6 control answers (declared variant only). */
  withControls?: boolean;
}) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const controls = read(draft, ['ventilation', 'controls']) != null;
  const combined = read(draft, ['ventilation', 'combined']) != null;
  const strips = read(draft, ['ventilation', 'grilleHeatingStrips']) != null;
  const toggle = (path: Path, label: string, present: boolean, template: Record<string, unknown>) =>
    <label className="nta-form-check">
      <input type="checkbox" checked={present} onChange={(event) => change(path, event.target.checked ? template : null)} />
      {label}
    </label>;
  return <>
    <SelectField {...field} path={['ventilation', 'heatRecoveryLayout']} label={t('opname.ventilation.recoveryLayout')}
      options={opts(t, 'opname.ventilation.recoveryLayoutKind', ['central', 'decentral'])} />
    {withControls && toggle(['ventilation', 'controls'], t('opname.ventilation.controls'), controls, { evidenceReference: '' })}
    {withControls && controls && <>
      <SelectField {...field} path={['ventilation', 'controls', 'co2Measurement']} label={t('opname.ventilation.co2Measurement')}
        options={opts(t, 'opname.ventilation.co2MeasurementKind', CO2_MEASUREMENTS)} />
      <SelectField {...field} path={['ventilation', 'controls', 'co2Control']} label={t('opname.ventilation.co2Control')}
        options={opts(t, 'opname.ventilation.target', CONTROL_TARGETS)} />
      <SelectField {...field} path={['ventilation', 'controls', 'timeControl']} label={t('opname.ventilation.timeControl')}
        options={opts(t, 'opname.ventilation.target', CONTROL_TARGETS)} />
      <TriStateField {...field} {...yesNo} path={['ventilation', 'controls', 'zoning']} label={t('opname.ventilation.zoning')} />
      <TriStateField {...field} {...yesNo} path={['ventilation', 'controls', 'extractPerHabitableRoom']}
        label={t('opname.ventilation.extractPerRoom')} />
      <TextField {...field} path={['ventilation', 'controls', 'evidenceReference']} label={t('opname.ventilation.controlsEvidence')} />
    </>}
    {toggle(['ventilation', 'combined'], t('opname.ventilation.combined'), combined,
      { decentralAreaM2: 0, totalResidenceAreaM2: 0 })}
    {combined && <>
      <NumberField {...field} path={['ventilation', 'combined', 'decentralAreaM2']} label={t('opname.ventilation.decentralArea')} />
      <NumberField {...field} path={['ventilation', 'combined', 'totalResidenceAreaM2']} label={t('opname.ventilation.totalResidenceArea')} />
    </>}
    {toggle(['ventilation', 'grilleHeatingStrips'], t('opname.ventilation.grilleStrips'), strips, { sourceReference: '' })}
    {strips && <>
      <NumberField {...field} path={['ventilation', 'grilleHeatingStrips', 'maxPowerWPerDm3PerS']} label={t('opname.ventilation.stripPower')} />
      <NumberField {...field} path={['ventilation', 'grilleHeatingStrips', 'maxTemperatureRiseK']} label={t('opname.ventilation.stripRise')} />
      <NumberField {...field} path={['ventilation', 'grilleHeatingStrips', 'switchOnBelowC']} label={t('opname.ventilation.stripSwitchOn')} />
      <NumberField {...field} path={['ventilation', 'grilleHeatingStrips', 'maxSupplyTemperatureC']} label={t('opname.ventilation.stripSupply')} />
      <TextField {...field} path={['ventilation', 'grilleHeatingStrips', 'sourceReference']} label={t('opname.sourceReference')} />
    </>}
    <p className="nta-form-note">{t('opname.ventilation.note')}</p>
  </>;
}

/**
 * ISSO 75.1 chapter 11 (p. 145–153) for the utility survey: heat recovery
 * with tables 11.9–11.12, duct airtightness (table 11.13), the installed
 * capacity (§11.4.1), system E and grilles with heating strips.
 */
export function UtilityVentilationFields({ draft, change, t }: { draft: Draft; change: Change; t: T }) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const duct = read(draft, ['ventilation', 'supplyDuctInsulation', 'kind']);
  return <>
    <SelectField {...field} path={['ventilation', 'heatRecovery']} label={t('opname.ventilation.heatRecovery')}
      options={opts(t, 'opname.ventilation.heatRecoveryKind', HEAT_RECOVERY)} />
    <SelectField {...field} path={['ventilation', 'ductAirtightness']} label={t('opname.ventilation.ductAirtightness')}
      options={opts(t, 'opname.ventilation.ductAirtightnessKind', ['luka_abc', 'luka_d', 'no_ducts', 'unknown'])} />
    <NumberField {...field} path={['ventilation', 'installedCapacityDm3PerS']} label={t('opname.ventilation.installedCapacity')} />
    <SelectField {...field} path={['ventilation', 'supplyDuctInsulation', 'kind']} label={t('opname.ventilation.supplyDuct')}
      options={opts(t, 'opname.ventilation.supplyDuctKind', ['uninsulated', 'insulated', 'specified'])}
      onChange={(_, value) => change(['ventilation', 'supplyDuctInsulation'], value == null ? null
        : value === 'specified' ? { kind: value, thicknessM: 0.02, conductivityWPerMK: 0.04 } : { kind: value })} />
    {duct === 'specified' && <>
      <NumberField {...field} path={['ventilation', 'supplyDuctInsulation', 'thicknessM']} label={t('opname.ventilation.supplyDuctThickness')} />
      <NumberField {...field} path={['ventilation', 'supplyDuctInsulation', 'conductivityWPerMK']} label={t('opname.ventilation.supplyDuctLambda')} />
    </>}
    <NumberField {...field} path={['ventilation', 'supplyDuctLengthM']} label={t('opname.ventilation.supplyDuctLength')} />
    <TriStateField {...field} {...yesNo} path={['ventilation', 'constantVolumeControl']} label={t('opname.ventilation.constantVolume')} />
    <NumberField {...field} path={['ventilation', 'bypassPercent']} label={t('opname.ventilation.bypassPercent')} step="1" />
    <VentilationSurveyFields draft={draft} change={change} t={t} withControls={false} />
  </>;
}

/** ISSO 75.1 table 10.2: the gas engine of a gas-driven chiller. */
function GasEngineFields({ draft, base, change, t }: { draft: Draft; base: Path; change: Change; t: T }) {
  const field = { draft, onChange: change };
  return <>
    <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={[...base, 'gasEngine', 'from2007']}
      label={t('opname.cooling.gasEngineFrom2007')} />
    <NumberField {...field} path={[...base, 'gasEngine', 'electricPowerKw']} label={t('opname.cooling.gasEnginePower')} />
    <CheckField {...field} path={[...base, 'gasEngine', 'hreDeclared']} label={t('opname.cooling.gasEngineHre')} />
  </>;
}

/** Zone of a surface or lighting zone; empty: split by A_g (surfaces) or not set. A reference to a zone that
 * no longer exists is shown as such, so the kernel's `surface_zone_unknown` is visible in the form. */
function ZoneSelect({ draft, change, path, label, ids, empty, unknown }: {
  draft: Draft; change: Change; path: Path; label: string; ids: string[]; empty: string; unknown: string;
}) {
  const value = read(draft, path);
  const current = typeof value === 'string' ? value : '';
  return <label>{label}
    <select value={current}
      onChange={(event) => change(path, event.target.value === '' ? null : event.target.value)}>
      <option value="">{empty}</option>
      {current !== '' && !ids.includes(current) && <option value={current}>{`${current} (${unknown})`}</option>}
      {ids.map((id) => <option key={id} value={id}>{id}</option>)}
    </select>
  </label>;
}

/** Rewrites the `zoneId` of surfaces and lighting zones from zone `from` to `to`, or clears them when `to` is null. */
export function moveZoneReferences(draft: Draft, from: string, to: string | null): Draft {
  const next = structuredClone(draft);
  const lists = [read(next, ['envelope', 'surfaces']), read(next, ['lighting'])];
  for (const items of lists) {
    if (!Array.isArray(items)) continue;
    for (const item of items as Array<Record<string, unknown>>) {
      if (item && item.zoneId === from) item.zoneId = to;
    }
  }
  return next;
}

/** Renames zone `index`; its references follow when the old name was unique and the new one is free. */
export function renameZone(draft: Draft, index: number, id: string): Draft {
  const zones = list(draft, ['zones']);
  const old = String(zones[index]?.id ?? '');
  const others = zones.filter((_, item) => item !== index).map((zone) => String(zone.id ?? ''));
  const moved = old !== '' && id !== '' && !others.includes(old) && !others.includes(id)
    ? moveZoneReferences(draft, old, id) : draft;
  return write(moved, ['zones', index, 'id'], id);
}

/**
 * A recorded default value in the UI language: an enum id (snake_case or PascalCase)
 * translated with the id as reference, yes/no for booleans, a number by locale, and the
 * kernel's fixed value texts in Dutch for a Dutch UI. Other text stays as written.
 */
export function defaultValueLabel(t: T, value: string, locale = 'en') {
  if (value === 'true' || value === 'false') return t(value === 'true' ? 'common.yes' : 'common.no');
  if (/^-?\d+(\.\d+)?$/.test(value)) return formatNumber(Number(value), locale, value.split('.')[1]?.length ?? 0);
  const pascal = /^[A-Z][a-z0-9]+(?:[A-Z][a-z0-9]*)+$/.test(value);
  if (pascal || /^[a-z][a-z0-9]*(_[a-z0-9]+)+$/.test(value)) {
    const id = pascal ? snakeCase(value) : value;
    const key = `opname.value.${id}`;
    const text = t(key);
    const label = text === key ? id.replace(/_/g, ' ') : text;
    return <>{label} <code className="kernel-code-ref">{value}</code></>;
  }
  if (locale.toLowerCase().startsWith('nl')) return dutchDefaultValue(value) ?? value;
  return value;
}

/** Removes zone `index` and clears the references to it (unless another zone has the same name). */
export function removeZone(draft: Draft, index: number): Draft {
  const zones = list(draft, ['zones']);
  const old = String(zones[index]?.id ?? '');
  const others = zones.filter((_, item) => item !== index).map((zone) => String(zone.id ?? ''));
  const cleared = old !== '' && !others.includes(old) ? moveZoneReferences(draft, old, null) : draft;
  return write(cleared, ['zones'], zones.filter((_, item) => item !== index));
}

/** ISSO 75.1 §6.5 (afb. 6.6, p. 52–54): calculation zones with their use functions, and the zone per lighting zone. */
export function CalculationZoneFields({ draft, change, replace, t }: {
  draft: Draft; change: Change; replace: (next: Draft) => void; t: T;
}) {
  const field = { draft, onChange: change };
  const zones = list(draft, ['zones']);
  const lighting = list(draft, ['lighting']);
  const ids = zones.map((zone) => String(zone.id ?? '')).filter((id) => id !== '');
  const systemE = read(draft, ['ventilation', 'combined']) != null;
  return <>
    <p className="nta-form-note">{t('opname.zones.note')}</p>
    {zones.map((_, index) => {
      const base: Path = ['zones', index];
      const functions = list(draft, [...base, 'functions']);
      const combined = read(draft, [...base, 'combined']) != null;
      return <div key={index} className="opname-zone" role="group"
        aria-label={t('opname.zones.heading', { n: index + 1, id: String(read(draft, [...base, 'id']) ?? '') })}>
        <strong className="opname-zone-heading">
          {t('opname.zones.heading', { n: index + 1, id: String(read(draft, [...base, 'id']) ?? '') })}
        </strong>
        <TextField draft={draft} path={[...base, 'id']} label={t('opname.zones.id')}
          onChange={(_, value) => replace(renameZone(draft, index, String(value ?? '')))} />
        {functions.map((_, functionIndex) => <div key={functionIndex} className="opname-served">
          <SelectField {...field} path={[...base, 'functions', functionIndex, 'function']} label={t('opname.zones.function')}
            options={opts(t, 'opname.functionKind', UTILITY_FUNCTIONS)} />
          <NumberField {...field} path={[...base, 'functions', functionIndex, 'areaM2']} label={t('opname.zones.area')} />
          <RemoveButton label={t('opname.remove')}
            onRemove={() => change([...base, 'functions'], functions.filter((_, item) => item !== functionIndex))} />
        </div>)}
        <ListControls label={t('opname.zones.addFunction')}
          onAdd={() => change([...base, 'functions'], [...functions, { function: 'office', areaM2: 0 }])} />
        <NumberField {...field} path={[...base, 'installedCapacityDm3PerS']} label={t('opname.zones.installedCapacity')} />
        <NumberField {...field} path={[...base, 'swimmingPoolAreaM2']} label={t('opname.zones.swimmingPool')} />
        {systemE && <label className="nta-form-check">
          <input type="checkbox" checked={combined}
            onChange={(event) => change([...base, 'combined'],
              event.target.checked ? { decentralAreaM2: 0, totalResidenceAreaM2: 0 } : null)} />
          {t('opname.zones.combined')}
        </label>}
        {systemE && combined && <>
          <NumberField {...field} path={[...base, 'combined', 'decentralAreaM2']} label={t('opname.zones.combinedDecentral')} />
          <NumberField {...field} path={[...base, 'combined', 'totalResidenceAreaM2']} label={t('opname.zones.combinedTotal')} />
        </>}
        <RemoveButton label={t('opname.zones.remove')} onRemove={() => replace(removeZone(draft, index))} />
      </div>;
    })}
    <ListControls label={t('opname.zones.add')}
      onAdd={() => change(['zones'], [...zones, calculationZoneTemplate(zones.length)])} />
    {ids.length > 1 && lighting.map((item, index) =>
      <ZoneSelect key={`l${index}`} draft={draft} change={change} path={['lighting', index, 'zoneId']}
        label={`${t('opname.zones.lightingZone')} ${String(item.id ?? index)}`} ids={ids} empty={t('opname.zones.notSet')}
        unknown={t('opname.zones.unknownZone')} />)}
  </>;
}

/** ISSO 75.1 §10.3.2–§10.4.5 (p. 131–137): further generators and the distribution answers. */
export function UtilityCoolingFields({ draft, change, t }: { draft: Draft; change: Change; t: T }) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const waterBased = read(draft, ['cooling', 'waterBased']) === true;
  const extras = list(draft, ['cooling', 'additionalGenerators']);
  return <>
    {read(draft, ['cooling', 'generator']) === 'gas_engine_compression' &&
      <GasEngineFields draft={draft} base={['cooling']} change={change} t={t} />}
    {!waterBased && <SelectField {...field} path={['cooling', 'directExpansion']} label={t('opname.cooling.directExpansion')}
      options={opts(t, 'opname.cooling.directExpansionKind', ['room', 'air_handling_unit'])} />}
    {waterBased && <>
      <TriStateField {...field} {...yesNo} path={['cooling', 'fittingsInsulated']} label={t('opname.cooling.fittingsInsulated')} />
      <TriStateField {...field} {...yesNo} path={['cooling', 'coldMeters']} label={t('opname.cooling.coldMeters')} />
      <NumberField {...field} path={['cooling', 'pipeLengthM']} label={t('opname.cooling.pipeLength')} />
      <NumberField {...field} path={['cooling', 'uncooledPipeLengthM']} label={t('opname.cooling.uncooledPipeLength')} />
      <NumberField {...field} path={['cooling', 'maxPipeLengthM']} label={t('opname.cooling.maxPipeLength')} />
    </>}
    {extras.map((_, index) => {
      const base: Path = ['cooling', 'additionalGenerators', index];
      return <div key={index} className="opname-item">
        <strong>{t('opname.additionalGenerator')} {index + 1}</strong>
        <SelectField {...field} path={[...base, 'generator']} label={t('opname.cooling.generator')}
          options={opts(t, 'opname.cooling.generatorKind', COOLING_GENERATORS)} />
        <NumberField {...field} path={[...base, 'capacityKw']} label={t('opname.cooling.capacityKw')} />
        {read(draft, [...base, 'generator']) === 'aquifer_year_unknown' &&
          <NumberField {...field} path={[...base, 'aquiferPermitYear']} label={t('opname.cooling.aquiferPermitYear')} step="1" />}
        {read(draft, [...base, 'generator']) === 'gas_engine_compression' &&
          <GasEngineFields draft={draft} base={base} change={change} t={t} />}
        <RemoveButton label={t('opname.remove')}
          onRemove={() => change(['cooling', 'additionalGenerators'], extras.filter((_, item) => item !== index))} />
      </div>;
    })}
    <ListControls label={t('opname.addGenerator')}
      onAdd={() => change(['cooling', 'additionalGenerators'], [...extras, { generator: 'compression', capacityKw: null }])} />
    <p className="nta-form-note">{t('opname.cooling.priorityNote')}</p>
  </>;
}

/** ISSO 82.1 p. 94: solar-control glass or film with the product g-value. */
export function SolarControlField({ draft, base, change, t }: { draft: Draft; base: Path; change: Change; t: T }) {
  const field = { draft, onChange: change };
  const present = read(draft, [...base, 'solarControl']) != null;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={present} onChange={(event) => change([...base, 'solarControl'],
        event.target.checked ? { gValue: null, sourceReference: '' } : null)} />
      {t('opname.window.solarControl')}
    </label>
    {present && <>
      <NumberField {...field} path={[...base, 'solarControl', 'gValue']} label={t('opname.window.solarControlG')} />
      <TextField {...field} path={[...base, 'solarControl', 'sourceReference']} label={t('opname.window.solarControlSource')} />
    </>}
  </>;
}

function HeatingGeneratorFields({ draft, path, change, t }: { draft: Draft; path: Path; change: Change; t: T }) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const kind = read(draft, [...path, 'kind']);
  const source = read(draft, [...path, 'source']);
  const collectiveSource = read(draft, [...path, 'collectiveSourceReference']) != null;
  const generator = (read(draft, path) ?? {}) as Record<string, unknown>;
  // Hidden answers must not survive a change of source or appliance: the
  // kernel would reject them (collective_source_water_based_only,
  // local_heater_fuel_contradiction).
  const without = (keys: string[]) => Object.fromEntries(Object.entries(generator).filter(([key]) => !keys.includes(key)));
  const changeSource = (_: Path, value: unknown) => {
    const stale: string[] = [];
    if (typeof value !== 'string' || !WATER_SOURCES.includes(value)) stale.push('collectiveSourceReference');
    if (value !== 'groundwater') stale.push('groundwaterSystem');
    const keepsCollective = !stale.includes('collectiveSourceReference') && collectiveSource;
    if (!(value === 'groundwater' || value === 'high_temperature' || (value === 'surface_water' && keepsCollective))) {
      stale.push('sourceTemperatureC', 'sourceTemperatureReference');
    }
    if (value !== 'high_temperature') stale.push('sourceQualityDeclarationReference');
    change(path, { ...without(stale), source: value });
  };
  const changeCollectiveSource = (checked: boolean) => {
    if (checked) change(path, { ...generator, collectiveSourceReference: '' });
    else change(path, without(source === 'surface_water'
      ? ['collectiveSourceReference', 'sourceTemperatureC', 'sourceTemperatureReference']
      : ['collectiveSourceReference']));
  };
  const changeAppliance = (_: Path, value: unknown) =>
    change(path, { ...(value === 'steam_boiler' ? generator : without(['fuel'])), appliance: value });
  return <>
    <KindSelect draft={draft} path={path} label={t('opname.heating.generator')} kinds={HEATING_KINDS}
      prefix="opname.heating.kind" template={heatingGeneratorTemplate} change={change} t={t} />
    {kind === 'boiler' && <>
      <SelectField {...field} path={[...path, 'boilerType']} label={t('opname.heating.boilerType')}
        options={opts(t, 'opname.heating.boiler', ['conventional', 'vr', 'hr100', 'hr104', 'hr107', 'hydrogen', 'oil'])} />
      <CheckField {...field} path={[...path, 'insideThermalBoundary']} label={t('opname.insideThermalBoundary')} />
      <NumberField {...field} path={[...path, 'manufactureYear']} label={t('opname.manufactureYear')} step="1" />
    </>}
    {kind === 'heat_pump' && <>
      <SelectField {...field} path={[...path, 'drive']} label={t('opname.heating.drive')}
        options={opts(t, 'opname.heating.driveKind', ['electric', 'gas_engine', 'gas_absorption'])} />
      <SelectField {...field} onChange={changeSource} path={[...path, 'source']} label={t('opname.heating.source')}
        options={opts(t, 'opname.heating.hpSource', HEAT_PUMP_SOURCES)} />
      <NumberField {...field} path={[...path, 'capacityKw']} label={t('opname.capacityKw')} />
      <CheckField {...field} path={[...path, 'highTemperature']} label={t('opname.heating.highTemperature')} />
      {typeof source === 'string' && WATER_SOURCES.includes(source) && <>
        <label className="nta-form-check">
          <input type="checkbox" checked={collectiveSource}
            onChange={(event) => changeCollectiveSource(event.target.checked)} />
          {t('opname.heating.collectiveSource')}
        </label>
        {collectiveSource && <TextField {...field} path={[...path, 'collectiveSourceReference']}
          label={t('opname.heating.collectiveSourceReference')} />}
      </>}
      {source === 'groundwater' && <SelectField {...field} path={[...path, 'groundwaterSystem']}
        label={t('opname.heating.groundwaterSystem')}
        options={opts(t, 'opname.heating.groundwaterSystemKind', ['doublet', 'recirculation'])} />}
      {(source === 'groundwater' || source === 'high_temperature' || (source === 'surface_water' && collectiveSource)) && <>
        <NumberField {...field} path={[...path, 'sourceTemperatureC']} label={t('opname.heating.sourceTemperature')} />
        <TextField {...field} path={[...path, 'sourceTemperatureReference']} label={t('opname.heating.sourceTemperatureReference')} />
      </>}
      {source === 'high_temperature' && <TextField {...field} path={[...path, 'sourceQualityDeclarationReference']}
        label={t('opname.heating.sourceQualityDeclaration')} />}
      <p className="nta-form-note">{t('opname.heating.heatPumpNote')}</p>
    </>}
    {kind === 'local_fired' && <>
      <SelectField {...field} onChange={changeAppliance} path={[...path, 'appliance']} label={t('opname.heating.appliance')}
        options={opts(t, 'opname.heating.localFired', ['gas_heater', 'oil_heater', 'steam_boiler'])} />
      {read(draft, [...path, 'appliance']) === 'steam_boiler' && <SelectField {...field} path={[...path, 'fuel']}
        label={t('opname.heating.fuel')} options={opts(t, 'opname.heating.fuelKind', ['natural_gas', 'oil'])} />}
      <CheckField {...field} path={[...path, 'flueGasExhaust']} label={t('opname.heating.flueGasExhaust')} />
      <TriStateField {...field} {...yesNo} path={[...path, 'electricityConnected']} label={t('opname.heating.electricityConnected')} />
    </>}
    {kind === 'gas_air_heater' && <>
      <SelectField {...field} path={[...path, 'heaterType']} label={t('opname.heating.airHeaterType')}
        options={opts(t, 'opname.heating.boiler', ['conventional', 'vr', 'hr100', 'hr104', 'hr107'])} />
      <TriStateField {...field} {...yesNo} path={[...path, 'pilotFlame']} label={t('opname.heating.pilotFlame')} />
      <NumberField {...field} path={[...path, 'count']} label={t('opname.airHeating.count')} step="1" />
    </>}
    {kind === 'electric' && <NumberField {...field} path={[...path, 'connectedDevices']} label={t('opname.heating.connectedDevices')} step="1" />}
    {kind === 'biomass' && <>
      <SelectField {...field} path={[...path, 'appliance']} label={t('opname.heating.appliance')}
        options={opts(t, 'opname.heating.biomass', ['freestanding_wood_stove', 'insert_stove', 'pellet_stove',
          'accumulating_stove', 'central_boiler'])} />
      <CheckField {...field} path={[...path, 'insideThermalBoundary']} label={t('opname.insideThermalBoundary')} />
      <CheckField {...field} path={[...path, 'soleHeatingInServedRooms']} label={t('opname.heating.soleHeating')} />
    </>}
    {kind === 'chp' && <>
      <NumberField {...field} path={[...path, 'electricalPowerKw']} label={t('opname.chp.electricalPowerKw')} />
      <NumberField {...field} path={[...path, 'thermalPowerKw']} label={t('opname.chp.thermalPowerKw')} />
      <SelectField {...field} path={[...path, 'engine']} label={t('opname.chp.engine')}
        options={opts(t, 'opname.chp.engineKind', ['gas_engine', 'diesel_engine', 'micro_turbine'])} />
      <NumberField {...field} path={[...path, 'manufactureYear']} label={t('opname.manufactureYear')} step="1" />
      <CheckField {...field} path={[...path, 'lowTemperature']} label={t('opname.chp.lowTemperature')} />
    </>}
  </>;
}

function HotWaterGeneratorFields({ draft, path, kind, change, t }: {
  draft: Draft; path: Path; kind: SurveyKind; change: Change; t: T;
}) {
  const field = { draft, onChange: change };
  const generator = read(draft, [...path, 'kind']);
  return <>
    <KindSelect draft={draft} path={path} label={t('opname.hotWater.generator')} kinds={HOT_WATER_KINDS[kind]}
      prefix="opname.hotWater.kind" template={hotWaterGeneratorTemplate} change={change} t={t} />
    {generator === 'gas_appliance' && <>
      <SelectField {...field} path={[...path, 'applianceType']} label={t('opname.hotWater.applianceType')}
        options={opts(t, 'opname.hotWater.appliance', ['bath_geyser', 'combi', 'kitchen_geyser', 'unknown'])} />
      <SelectField {...field} path={[...path, 'gaskeur']} label={t('opname.hotWater.gaskeur')}
        options={opts(t, 'opname.hotWater.gaskeurKind', ['none', 'gaskeur', 'gaskeur_cw', 'gaskeur_hr_cw', 'unknown'])} />
      {kind === 'residential' && <SelectField {...field} path={[...path, 'cwClass']} label={t('opname.hotWater.cwClass')}
        options={opts(t, 'opname.hotWater.cw', ['cw1', 'cw2', 'cw3', 'cw4_to6', 'unknown'])} />}
      <NumberField {...field} path={[...path, 'burnerLoadKw']} label={t('opname.hotWater.burnerLoadKw')} />
    </>}
    {generator === 'heat_pump' && <CheckField {...field} path={[...path, 'exhaustAirSource']} label={t('opname.hotWater.exhaustAir')} />}
    {generator === 'gas_storage_heater' && <>
      <NumberField {...field} path={[...path, 'volumeL']} label={t('opname.volumeL')} />
      <CheckField {...field} path={[...path, 'before1985']} label={t('opname.hotWater.before1985')} />
      <CheckField {...field} path={[...path, 'inHeatedZone']} label={t('opname.inHeatedZone')} />
    </>}
  </>;
}

function ListControls({ label, onAdd }: { label: string; onAdd: () => void }) {
  return <div className="opname-list-controls">
    <button type="button" className="btn" onClick={onAdd}>{label}</button>
  </div>;
}

function RemoveButton({ label, onRemove }: { label: string; onRemove: () => void }) {
  return <button type="button" className="btn opname-remove" onClick={onRemove}>{label}</button>;
}

function list(draft: Draft, path: Path): Array<Record<string, unknown>> {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Array<Record<string, unknown>> : [];
}

export function BasisopnamePanel() {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const stored = state.project.basisopname as StoredSurvey | undefined;
  const [result, setResult] = useState<OpnameAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [json, setJson] = useState<string | null>(null);
  const requestId = useRef(0);

  const save = (next: StoredSurvey | undefined, keepResult = false) => {
    dispatch({ type: 'SET_BASISOPNAME', payload: next });
    if (!keepResult) {
      requestId.current += 1;
      setResult(null);
      setError(null);
      setBusy(false);
    }
  };

  if (!stored) {
    return <section className="nta-performance opname-panel" aria-label={t('opname.title')}>
      <h3>{t('opname.title')}</h3>
      <p className="nta-form-note">{t('opname.intro')}</p>
      <div className="opname-actions">
        <button type="button" className="btn btn-primary" onClick={() => save(surveyTemplate('residential'))}>{t('opname.startResidential')}</button>
        <button type="button" className="btn" onClick={() => save(surveyTemplate('utility'))}>{t('opname.startUtility')}</button>
      </div>
    </section>;
  }

  const kind = stored.kind;
  const draft = stored.survey as Draft;
  const change: Change = (path, value) => save({ kind, survey: write(draft, path, value) });
  const field = { draft, onChange: change };
  const surfaces = list(draft, ['envelope', 'surfaces']);
  const windows = list(draft, ['envelope', 'windows']);
  const rooflights = list(draft, ['envelope', 'rooflights']);
  const buildingKind = read(draft, ['envelope', 'buildingKind', 'kind']);
  const hotWaterSystems = list(draft, ['additionalHotWaterSystems']);
  const ahu = read(draft, ['ventilation', 'ahu']);
  const heatingExtras = list(draft, ['heating', 'additionalGenerators']);
  const hotWaterExtras = list(draft, ['hotWater', 'additionalGenerators']);
  const solar = list(draft, ['hotWater', 'solar']);
  const pv = list(draft, ['pv']);
  const reasons = (read(draft, ['inklapRedenen']) ?? {}) as Record<string, string>;
  const verticalPipes = read(draft, ['verticalPipes']);
  const zoneIds = list(draft, ['zones']).map((zone) => String(zone.id ?? '')).filter((id) => id !== '');

  const run = async () => {
    const current = ++requestId.current;
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      const assessment = kind === 'residential'
        ? await assessResidentialSurveyWithRust(asResidential(stored))
        : await assessUtilitySurveyWithRust(asUtility(stored));
      if (requestId.current === current) setResult(assessment);
    } catch (failure) {
      if (requestId.current === current) setError(failure instanceof Error ? failure.message : String(failure));
    } finally {
      if (requestId.current === current) setBusy(false);
    }
  };

  const performance = result?.performance;
  return <section className="nta-performance opname-panel" aria-label={t('opname.title')}>
    <h3>{t('opname.title')} — {t(`opname.kind.${kind}`)}</h3>
    <p className="nta-form-note">{t('opname.scope')}</p>

    <div className="nta-form">
    <Section title={t('opname.general')}>
      <TextField {...field} path={['id']} label={t('opname.id')} />
      <NumberField {...field} path={['constructionYear']} label={t('opname.constructionYear')} step="1" />
      {kind === 'residential' && <NumberField {...field} path={['usableFloorAreaM2']} label={t('opname.usableFloorArea')} />}
      <NumberField {...field} path={['buildingHeightM']} label={t('opname.buildingHeight')} />
      <NumberField {...field} path={['storeys']} label={t('opname.storeys')} step="1" />
      {kind === 'residential' && read(draft, ['dwelling', 'kind']) === 'apartment' &&
        <SelectField {...field} path={['dwelling', 'floor']} label={t('opname.dwelling.floor')}
          options={opts(t, 'opname.dwelling.floorKind', ['ground_or_intermediate', 'top', 'roof_and_floor'])} />}
      {kind === 'utility' && <>
        <NumberField {...field} path={['toiletStacks']} label={t('opname.toiletStacks')} step="1" />
        <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={['fossilFuelOnPlot']} label={t('opname.fossilFuelOnPlot')} />
        <NumberField {...field} path={['sportHallAreaM2']} label={t('opname.sportHallArea')} />
        <NumberField {...field} path={['swimmingPoolAreaM2']} label={t('opname.swimmingPoolArea')} />
        <CheckField {...field} path={['openlyConnectedResidenceAreas']} label={t('opname.openlyConnected')} />
      </>}
      <label>{t('opname.verticalPipes')}
        <select value={verticalPipes == null ? 'default' : Array.isArray(verticalPipes) && verticalPipes.length === 0 ? 'none' : 'count'}
          onChange={(event) => change(['verticalPipes'], event.target.value === 'default' ? null
            : event.target.value === 'none' ? [] : [{ insulated: false }])}>
          <option value="default">{t('opname.verticalPipes.default')}</option>
          <option value="none">{t('opname.verticalPipes.none')}</option>
          <option value="count">{t('opname.verticalPipes.count')}</option>
        </select>
      </label>
      {Array.isArray(verticalPipes) && verticalPipes.length > 0 && <label>{t('opname.verticalPipes.number')}
        <input type="number" step="1" min="1" value={verticalPipes.length}
          onChange={(event) => change(['verticalPipes'], Array.from({ length: Math.max(1, Number(event.target.value) || 1) },
            (_, index) => (verticalPipes as unknown[])[index] ?? { insulated: false }))} />
      </label>}
    </Section>

    {kind === 'utility' && <Section title={t('opname.zones')}>
      <CalculationZoneFields draft={draft} change={change} replace={(next) => save({ kind, survey: next })} t={t} />
    </Section>}

    <Section title={t('opname.envelope')}>
      {kind === 'residential' && <label>{t('opname.buildingKind')}
        <select value={typeof buildingKind === 'string' ? buildingKind : 'regular'}
          onChange={(event) => change(['envelope', 'buildingKind'], event.target.value === 'regular' ? null
            : event.target.value === 'floating' ? { kind: 'floating', newBerthSince2018: false } : { kind: event.target.value })}>
          {['regular', 'caravan', 'floating'].map((key) =>
            <option key={key} value={key}>{t(`opname.buildingKind.${key}`)}</option>)}
        </select>
      </label>}
      {buildingKind === 'floating' &&
        <CheckField {...field} path={['envelope', 'buildingKind', 'newBerthSince2018']} label={t('opname.buildingKind.newBerth')} />}
      {surfaces.map((surface, index) => {
        const base: Path = ['envelope', 'surfaces', index];
        const insulation = read(draft, [...base, 'insulation', 'kind']);
        return <div key={index} className="opname-item">
          <strong>{String(surface.id ?? index)}</strong>
          <SelectField {...field} path={[...base, 'element']} label={t('opname.surface.element')}
            options={opts(t, 'opname.surface.elementKind', ['facade', 'roof', 'floor'])} />
          <label>{t('opname.surface.boundary')}
            <select value={String(read(draft, [...base, 'boundary', 'kind']) ?? '')}
              onChange={(event) => change([...base, 'boundary'], { kind: event.target.value })}>
              {['outdoor', 'ground', 'crawlspace', 'adjacent_heated', 'unheated_cellar', 'strongly_ventilated', 'sunroom', 'water'].map((key) =>
                <option key={key} value={key}>{t(`opname.surface.boundaryKind.${key}`)}</option>)}
            </select>
          </label>
          <NumberField {...field} path={[...base, 'grossAreaM2']} label={t('opname.surface.area')} />
          {kind === 'utility' && zoneIds.length > 1 &&
            <ZoneSelect draft={draft} change={change} path={[...base, 'zoneId']} label={t('opname.surface.zone')}
              ids={zoneIds} empty={t('opname.zones.splitByArea')} unknown={t('opname.zones.unknownZone')} />}
          <label>{t('opname.surface.insulation')}
            <select value={typeof insulation === 'string' ? insulation : ''}
              onChange={(event) => change([...base, 'insulation'], event.target.value === 'thickness'
                ? { kind: 'thickness', thicknessMm: 50 } : { kind: event.target.value })}>
              {['none_or_unknown', 'present_unknown_thickness', 'cavity_filled_unknown_width', 'thickness'].map((key) =>
                <option key={key} value={key}>{t(`opname.surface.insulationKind.${key}`)}</option>)}
            </select>
          </label>
          {insulation === 'thickness' && <NumberField {...field} path={[...base, 'insulation', 'thicknessMm']} label={t('opname.surface.thicknessMm')} />}
          <CheckField {...field} path={[...base, 'thermalCushions']} label={t('opname.surface.thermalCushions')} />
          {insulation === 'present_unknown_thickness' && <>
            <NumberField {...field} path={[...base, 'renovation', 'year']} label={t('opname.surface.renovationYear')} step="1" />
            <CheckField {...field} path={[...base, 'renovation', 'meetsRequirementsOfYear']} label={t('opname.surface.renovationEvidence')} />
          </>}
        </div>;
      })}
      {windows.map((window, index) => {
        const base: Path = ['envelope', 'windows', index];
        const situation = read(draft, [...base, 'shading', 'situation']);
        return <div key={`w${index}`} className="opname-item">
          <strong>{String(window.id ?? index)}</strong>
          <SelectField {...field} path={[...base, 'surfaceId']} label={t('opname.window.surface')}
            options={surfaces.map((surface) => [String(surface.id), String(surface.id)])} />
          <NumberField {...field} path={[...base, 'areaM2']} label={t('opname.window.area')} />
          <SelectField {...field} path={[...base, 'glass']} label={t('opname.window.glass')}
            options={opts(t, 'opname.window.glassKind', ['triple_hr', 'hr_plus_plus', 'hr_plus', 'hr', 'double', 'single'])} />
          <SelectField {...field} path={[...base, 'frame']} label={t('opname.window.frame')}
            options={opts(t, 'opname.window.frameKind', ['wood_or_plastic', 'metal_with_thermal_break', 'metal', 'none'])} />
          <label>{t('opname.window.shading')}
            <select value={typeof situation === 'string' ? situation : ''}
              onChange={(event) => change([...base, 'shading'], event.target.value === '' ? undefined
                : event.target.value === 'constant_height_obstruction' || event.target.value === 'constant_overhang'
                  ? { situation: event.target.value, relativeHeight: 0.5 }
                  : event.target.value === 'overhang_with_obstructions'
                    ? { situation: event.target.value, overhangRelativeHeight: 0.5 }
                    : { situation: event.target.value })}>
              <option value="">{t('opname.window.shadingDefault')}</option>
              {['minimal', 'constant_height_obstruction', 'constant_overhang', 'full', 'overhang_with_obstructions', 'other'].map((key) =>
                <option key={key} value={key}>{t(`opname.window.situation.${key}`)}</option>)}
            </select>
          </label>
          {(situation === 'constant_height_obstruction' || situation === 'constant_overhang') &&
            <NumberField {...field} path={[...base, 'shading', 'relativeHeight']} label={t('opname.window.relativeHeight')} />}
          {situation === 'overhang_with_obstructions' &&
            <NumberField {...field} path={[...base, 'shading', 'overhangRelativeHeight']} label={t('opname.window.relativeHeight')} />}
          <SolarControlField draft={draft} base={base} change={change} t={t} />
          <RemoveButton label={t('opname.remove')} onRemove={() => change(['envelope', 'windows'], windows.filter((_, item) => item !== index))} />
        </div>;
      })}
      <ListControls label={t('opname.addWindow')}
        onAdd={() => change(['envelope', 'windows'], [...windows, windowTemplate(windows.length, String(surfaces[0]?.id ?? ''))])} />
      {rooflights.map((rooflight, index) => {
        const base: Path = ['envelope', 'rooflights', index];
        return <div key={`r${index}`} className="opname-item">
          <strong>{String(rooflight.id ?? index)}</strong>
          <SelectField {...field} path={[...base, 'surfaceId']} label={t('opname.window.surface')}
            options={surfaces.filter((surface) => surface.element === 'roof').map((surface) => [String(surface.id), String(surface.id)])} />
          <NumberField {...field} path={[...base, 'areaM2']} label={t('opname.rooflight.area')} />
          <NumberField {...field} path={[...base, 'uValue']} label={t('opname.rooflight.uValue')} />
          <SelectField {...field} path={[...base, 'glass']} label={t('opname.window.glass')}
            options={opts(t, 'opname.window.glassKind', ['triple_hr', 'hr_plus_plus', 'hr_plus', 'hr', 'double', 'single'])} />
          <TextField {...field} path={[...base, 'qualityDeclarationReference']} label={t('opname.rooflight.declaration')} />
          <RemoveButton label={t('opname.remove')} onRemove={() => change(['envelope', 'rooflights'], rooflights.filter((_, item) => item !== index))} />
        </div>;
      })}
      <ListControls label={t('opname.addRooflight')}
        onAdd={() => change(['envelope', 'rooflights'], [...rooflights, {
          id: `lichtkoepel-${rooflights.length + 1}`,
          surfaceId: String(surfaces.find((surface) => surface.element === 'roof')?.id ?? ''),
          areaM2: 1, uValue: 2.5, glass: 'double', qualityDeclarationReference: '',
        }])} />
    </Section>

    <Section title={t('opname.heating')}>
      <HeatingGeneratorFields draft={draft} path={['heating', 'generator']} change={change} t={t} />
      <NumberField {...field} path={['heating', 'nominalPowerKw']} label={t('opname.nominalPowerKw')} />
      <EmitterFields draft={draft} change={change} t={t} />
      <SelectField {...field} path={['heating', 'designClass']} label={t('opname.heating.designClass')}
        options={opts(t, 'opname.heating.designClassKind', ['c45_40', 'c55_47', 'c70_50', 'c90_70'])} />
      {read(draft, ['heating', 'designClass']) === 'c90_70' &&
        <TextField {...field} path={['heating', 'heatPumpAbove70Declaration']} label={t('opname.heating.above70Declaration')} />}
      <DistributionFields draft={draft} change={change} t={t} />
      <CheckField {...field} path={['heating', 'addedPreferredGenerator']} label={t('opname.heating.addedPreferred')} />
      <label className="nta-form-check">
        <input type="checkbox" checked={read(draft, ['heating', 'collective']) != null}
          onChange={(event) => change(['heating', 'collective'], event.target.checked ? {} : null)} />
        {t('opname.heating.collective')}
      </label>
      {read(draft, ['heating', 'collective']) != null && <>
        <NumberField {...field} path={['heating', 'collective', 'connectedDwellings']} label={t('opname.connectedDwellings')} step="1" />
        <NumberField {...field} path={['heating', 'collective', 'connectedUsableAreaM2']} label={t('opname.connectedArea')} />
      </>}
      {heatingExtras.map((_, index) => <div key={index} className="opname-item">
        <strong>{t('opname.additionalGenerator')} {index + 1}</strong>
        <HeatingGeneratorFields draft={draft} path={['heating', 'additionalGenerators', index, 'generator']} change={change} t={t} />
        <NumberField {...field} path={['heating', 'additionalGenerators', index, 'nominalPowerKw']} label={t('opname.nominalPowerKw')} />
        <RemoveButton label={t('opname.remove')} onRemove={() => change(['heating', 'additionalGenerators'], heatingExtras.filter((_, item) => item !== index))} />
      </div>)}
      <ListControls label={t('opname.addGenerator')}
        onAdd={() => change(['heating', 'additionalGenerators'], [...heatingExtras, { generator: heatingGeneratorTemplate('boiler'), nominalPowerKw: 20 }])} />
    </Section>

    <Section title={t('opname.hotWater')}>
      <HotWaterGeneratorFields draft={draft} path={['hotWater', 'generator']} kind={kind} change={change} t={t} />
      <NumberField {...field} path={['hotWater', 'nominalPowerKw']} label={t('opname.nominalPowerKw')} />
      {kind === 'residential' && <>
        <label className="nta-form-check">
          <input type="checkbox" checked={read(draft, ['hotWater', 'collective']) != null}
            onChange={(event) => change(['hotWater', 'collective'], event.target.checked ? {} : null)} />
          {t('opname.hotWater.collective')}
        </label>
        {read(draft, ['hotWater', 'collective']) != null &&
          <NumberField {...field} path={['hotWater', 'collective', 'connectedDwellings']} label={t('opname.connectedDwellings')} step="1" />}
        {read(draft, ['hotWater', 'generator', 'kind']) === 'electric_boiler' && <>
          <NumberField {...field} path={['hotWater', 'boilerVessel', 'volumeL']} label={t('opname.volumeL')} />
          <CheckField {...field} path={['hotWater', 'boilerVessel', 'kitchenCabinet']} label={t('opname.hotWater.kitchenCabinet')} />
          <NumberField {...field} path={['hotWater', 'boilerVessel', 'manufactureYear']} label={t('opname.manufactureYear')} step="1" />
        </>}
      </>}
      {hotWaterExtras.map((_, index) => <div key={index} className="opname-item">
        <strong>{t('opname.additionalGenerator')} {index + 1}</strong>
        <HotWaterGeneratorFields draft={draft} path={['hotWater', 'additionalGenerators', index, 'generator']} kind={kind} change={change} t={t} />
        <NumberField {...field} path={['hotWater', 'additionalGenerators', index, 'nominalPowerKw']} label={t('opname.nominalPowerKw')} />
        <RemoveButton label={t('opname.remove')} onRemove={() => change(['hotWater', 'additionalGenerators'], hotWaterExtras.filter((_, item) => item !== index))} />
      </div>)}
      <ListControls label={t('opname.addGenerator')}
        onAdd={() => change(['hotWater', 'additionalGenerators'], [...hotWaterExtras, { generator: hotWaterGeneratorTemplate('electric_instantaneous'), nominalPowerKw: 10 }])} />
      {solar.map((_, index) => {
        const base: Path = ['hotWater', 'solar', index];
        return <div key={`s${index}`} className="opname-item">
          <strong>{t('opname.solar')} {index + 1}</strong>
          <SelectField {...field} path={[...base, 'collector']} label={t('opname.solar.collector')}
            options={opts(t, 'opname.solar.collectorKind', ['unglazed', 'glazed', 'evacuated_tube', 'unknown'])} />
          <NumberField {...field} path={[...base, 'collectorAreaM2']} label={t('opname.solar.area')} />
          <SelectField {...field} path={[...base, 'orientation']} label={t('opname.orientation')}
            options={opts(t, 'opname.orientationKind', ORIENTATIONS)} />
          <NumberField {...field} path={[...base, 'tiltDeg']} label={t('opname.tilt')} />
          <SelectField {...field} path={[...base, 'backup']} label={t('opname.solar.backup')}
            options={opts(t, 'opname.solar.backupKind', ['separate_heater', 'integrated_gas', 'integrated_electric', 'unknown'])} />
          <NumberField {...field} path={[...base, 'storageVolumeL']} label={t('opname.volumeL')} />
          <CheckField {...field} path={[...base, 'alsoSpaceHeating']} label={t('opname.solar.alsoSpaceHeating')} />
          <TextField {...field} path={[...base, 'sourceReference']} label={t('opname.sourceReference')} />
          <RemoveButton label={t('opname.remove')} onRemove={() => change(['hotWater', 'solar'], solar.filter((_, item) => item !== index))} />
        </div>;
      })}
      <ListControls label={t('opname.addSolar')} onAdd={() => change(['hotWater', 'solar'], [...solar, solarTemplate(solar.length)])} />
      {kind === 'utility' && hotWaterSystems.map((_, index) => {
        const base: Path = ['additionalHotWaterSystems', index];
        const served = list(draft, [...base, 'servedAreas']);
        return <div key={`h${index}`} className="opname-item">
          <strong>{t('opname.hotWater.system')} {index + 2}</strong>
          <HotWaterGeneratorFields draft={draft} path={[...base, 'generator']} kind={kind} change={change} t={t} />
          <NumberField {...field} path={[...base, 'nominalPowerKw']} label={t('opname.nominalPowerKw')} />
          {served.map((_, areaIndex) => <div key={areaIndex} className="opname-served">
            <SelectField {...field} path={[...base, 'servedAreas', areaIndex, 'function']} label={t('opname.hotWater.servedFunction')}
              options={opts(t, 'opname.functionKind', UTILITY_FUNCTIONS)} />
            <NumberField {...field} path={[...base, 'servedAreas', areaIndex, 'areaM2']} label={t('opname.hotWater.servedArea')} />
            <RemoveButton label={t('opname.remove')}
              onRemove={() => change([...base, 'servedAreas'], served.filter((_, item) => item !== areaIndex))} />
          </div>)}
          <ListControls label={t('opname.hotWater.addServedArea')}
            onAdd={() => change([...base, 'servedAreas'], [...served, { function: String(read(draft, ['functions', 0, 'function']) ?? 'office'), areaM2: 0 }])} />
          <TextField {...field} path={[...base, 'sourceReference']} label={t('opname.sourceReference')} />
          <RemoveButton label={t('opname.remove')}
            onRemove={() => change(['additionalHotWaterSystems'], hotWaterSystems.filter((_, item) => item !== index))} />
        </div>;
      })}
      {kind === 'utility' && <ListControls label={t('opname.hotWater.addSystem')}
        onAdd={() => change(['additionalHotWaterSystems'], [...hotWaterSystems, {
          generator: hotWaterGeneratorTemplate('electric_instantaneous'), showerHeatRecovery: 'none',
          servedAreas: [{ function: String(read(draft, ['functions', 0, 'function']) ?? 'office'), areaM2: 0 }], sourceReference: '',
        }])} />}
    </Section>

    {kind === 'utility' && ahu != null && <Section title={t('opname.ahu')}>
      <CheckField {...field} path={['ventilation', 'ahu', 'heatingConnected']} label={t('opname.ahu.heatingConnected')} />
      <CheckField {...field} path={['ventilation', 'ahu', 'coolingConnected']} label={t('opname.ahu.coolingConnected')} />
    </Section>}

    {kind === 'residential' && <Section title={t('opname.ventilation')}>
      <VentilationSurveyFields draft={draft} change={change} t={t} />
    </Section>}

    {kind === 'utility' && <Section title={t('opname.ventilation')}>
      <UtilityVentilationFields draft={draft} change={change} t={t} />
    </Section>}

    <Section title={t('opname.passiveCooling')}>
      <PassiveCoolingFields draft={draft} change={change} t={t} />
    </Section>

    <Section title={t('opname.cooling')}>
      <label className="nta-form-check">
        <input type="checkbox" checked={read(draft, ['cooling']) != null}
          onChange={(event) => {
            const cooling = event.target.checked
              ? { generator: 'room_air_conditioner', emitter: 'split_indoor_units_on_wall', fanCoilCount: 1, waterBased: false, sourceReference: '' }
              : null;
            // One write, so both fields land in the same draft.
            const next = write(draft, ['cooling'], cooling);
            save({ kind, survey: kind === 'residential' ? write(next, ['coolingPresent'], event.target.checked) : next });
          }} />
        {t('opname.cooling.present')}
      </label>
      {read(draft, ['cooling']) != null && <>
        <SelectField {...field} path={['cooling', 'generator']} label={t('opname.cooling.generator')}
          options={opts(t, 'opname.cooling.generatorKind', COOLING_GENERATORS)} />
        <SelectField {...field} path={['cooling', 'emitter']} label={t('opname.cooling.emitter')}
          options={opts(t, 'opname.cooling.emitterKind', COOLING_EMITTERS)} />
        <NumberField {...field} path={['cooling', 'fanCoilCount']} label={t('opname.cooling.fanCoilCount')} step="1" />
        <NumberField {...field} path={['cooling', 'capacityKw']} label={t('opname.cooling.capacityKw')} />
        <label className="nta-form-check">
          <input type="checkbox" checked={read(draft, ['cooling', 'waterBased']) === true}
            onChange={(event) => {
              // §10.4.1: direct expansion only without a water distribution; the
              // hidden answer is cleared so it cannot block the survey.
              const next = write(draft, ['cooling', 'waterBased'], event.target.checked);
              save({ kind, survey: event.target.checked ? write(next, ['cooling', 'directExpansion'], undefined) : next });
            }} />
          {t('opname.cooling.waterBased')}
        </label>
        <SelectField {...field} path={['cooling', 'balanced']} label={t('opname.cooling.balanced')}
          options={[['none', t('opname.cooling.balanced.none')], ['static', t('opname.cooling.balanced.static')],
            ['dynamic', t('opname.cooling.balanced.dynamic')]]} />
        {['static', 'dynamic', true].includes(read(draft, ['cooling', 'balanced']) as string | boolean) &&
          <TextField {...field} path={['cooling', 'balancingEvidenceReference']} label={t('opname.cooling.balancingEvidence')} />}
        <CheckField {...field} path={['cooling', 'heatPumpSource']} label={t('opname.cooling.heatPumpSource')} />
        <CheckField {...field} path={['cooling', 'groundAboveZeroDemonstrated']} label={t('opname.cooling.groundAboveZero')} />
        {kind === 'residential' &&
          <CheckField {...field} path={['coolingCollective']} label={t('opname.cooling.collective')} />}
        {kind === 'utility' && <UtilityCoolingFields draft={draft} change={change} t={t} />}
        <TextField {...field} path={['cooling', 'sourceReference']} label={t('opname.sourceReference')} />
      </>}
    </Section>

    <Section title={t('opname.pv')}>
      {pv.map((_, index) => {
        const base: Path = ['pv', index];
        const method = read(draft, [...base, 'shading', 'method']);
        return <div key={index} className="opname-item">
          <strong>{String(pv[index].id ?? index)}</strong>
          <NumberField {...field} path={[...base, 'panelAreaM2']} label={t('opname.pv.area')} />
          <SelectField {...field} path={[...base, 'moduleType']} label={t('opname.pv.moduleType')}
            options={opts(t, 'opname.pv.module', ['monocrystalline', 'polycrystalline', 'cigs', 'cd_te', 'amorphous_unknown', 'unknown'])} />
          <NumberField {...field} path={[...base, 'azimuthDeg']} label={t('opname.pv.azimuth')} />
          <NumberField {...field} path={[...base, 'tiltDeg']} label={t('opname.tilt')} />
          <label>{t('opname.pv.shading')}
            <select value={typeof method === 'string' ? method : ''}
              onChange={(event) => change([...base, 'shading'], event.target.value === '' ? undefined
                : event.target.value === 'side_obstruction' ? { method: 'side_obstruction', side: 'both', relativeWidth: 0.5 }
                  : event.target.value === 'roof_edge' ? { method: 'roof_edge', heightM: 1, distanceM: 0.5 }
                    : { method: event.target.value })}>
              <option value="">{t('opname.pv.shadingDefault')}</option>
              {['minimal', 'side_obstruction', 'full', 'roof_edge', 'other'].map((key) =>
                <option key={key} value={key}>{t(`opname.pv.situation.${key}`)}</option>)}
            </select>
          </label>
          <RemoveButton label={t('opname.remove')} onRemove={() => change(['pv'], pv.filter((_, item) => item !== index))} />
        </div>;
      })}
      <ListControls label={t('opname.addPv')} onAdd={() => change(['pv'], [...pv, pvTemplate(pv.length)])} />
    </Section>

    </div>

    <details className="opname-json">
      <summary>{t('opname.json')}</summary>
      <textarea aria-label={t('opname.json')} rows={14} value={json ?? JSON.stringify(draft, null, 2)}
        onChange={(event) => setJson(event.target.value)} />
      <div className="opname-actions">
        <button type="button" className="btn" disabled={json == null} onClick={() => {
          try {
            save({ kind, survey: JSON.parse(json ?? '{}') as Record<string, unknown> });
            setJson(null);
          } catch {
            setError(t('opname.jsonInvalid'));
          }
        }}>{t('opname.jsonApply')}</button>
      </div>
    </details>

    <div className="opname-actions">
      <button type="button" className="btn btn-primary" disabled={busy} onClick={() => { void run(); }}>{t('opname.calculate')}</button>
      <button type="button" className="btn" onClick={() => save(undefined)}>{t('opname.discard')}</button>
    </div>
    {error && <p className="opname-error" role="alert">{error}</p>}

    {result && <div className="opname-result" aria-label={t('opname.result')}>
      <h4 className="opname-result-heading">{t('opname.resultHeading')}</h4>
      <p className="nta-form-note">{t('opname.resultNote')}</p>
      <p><strong>{t('opname.status')}:</strong> {t(`opname.statusValue.${result.status}`, { defaultValue: result.status })}</p>
      {performance && <ul className="opname-indicators">
        <li>{t('opname.label')}: <strong>{performance.indicativeLabelClass ?? '—'}</strong></li>
        <li>BENG 1: {formatNumber(performance.needIndicatorKwhPerM2Year, locale, 2)} kWh/m²</li>
        <li>BENG 2: {formatNumber(performance.primaryFossilIndicatorKwhPerM2Year, locale, 2)} kWh/m²</li>
        <li>BENG 3: {formatNumber(performance.renewableSharePercent, locale, 1)} %</li>
      </ul>}
      {result.issues.length > 0 && <ul className="opname-issues">
        {result.issues.map((item, index) => {
          const hint = t(`opname.issueHint.${item.code}`, { defaultValue: '' });
          return <li key={index}><KernelCode code={item.code} prefixes={['opname.issue.', 'nta.gap.', 'kernel.issue.']} />
            {' '}<code>{item.path}</code>{hint && <small> {hint}</small>}</li>;
        })}
      </ul>}
      {result.warnings.length > 0 && <ul className="opname-warnings">
        {result.warnings.map((item, index) => <li key={index}>
          <KernelCode code={item.code} prefixes={['opname.warning.', 'nta.warning.', 'nta.gap.']} /> {item.note}</li>)}
      </ul>}
      {result.appliedDefaults.length > 0 && <table className="opname-defaults">
        <caption>{t('opname.defaults')}</caption>
        <thead><tr><th>{t('opname.defaults.path')}</th><th>{t('opname.defaults.value')}</th>
          <th>{t('opname.defaults.source')}</th><th>{t('opname.defaults.reason')}</th></tr></thead>
        <tbody>
          {result.appliedDefaults.map((item, index) => <tr key={index}>
            <td><code>{item.path}</code></td><td>{defaultValueLabel(t, item.value, locale)}</td><td>{item.source}</td>
            <td><input className="opname-reason" aria-label={`${t('opname.defaults.reason')} ${item.path}`} value={reasons[item.path] ?? ''}
              onChange={(event) => {
                const next = { ...reasons };
                if (event.target.value === '') delete next[item.path]; else next[item.path] = event.target.value;
                // The reason is documentation (BRL 9500 §4.2.2): keep the result.
                save({ kind, survey: write(draft, ['inklapRedenen'], next) }, true);
              }} /></td>
          </tr>)}
        </tbody>
      </table>}
    </div>}
  </section>;
}
