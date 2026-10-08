import { useRef, useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import {
  CheckField, FieldPathPrefixProvider, NumberField, read, Section, SelectField, TextField, TriStateField, write, type Draft, type Path,
} from '../NtaPerformancePanel/NtaFormFields';
import { Pill } from '../ui';
import { labelColor } from '../shell/pages/results/resultsData';
import {
  assessResidentialSurveyWithRust, assessUtilitySurveyWithRust, DEFAULT_NORM_VERSION, type OpnameAssessment,
} from '../../core/nta/KernelClient';
import {
  asResidential, asUtility, calculationZoneTemplate, heatingGeneratorTemplate, hotWaterGeneratorTemplate, pvTemplate,
  solarTemplate, surveyTemplate, windowTemplate, type StoredSurvey, type SurveyKind,
} from '../../core/nta/SurveyTemplates';
import { KernelCode } from '../KernelCode/KernelCode';
import { EvidenceAttach } from '../EvidenceLink/EvidenceLink';
import { SurveyTakeoverAction } from './SurveyTakeoverAction';
import { freshItemId, jsonPointer, linksAfterRemoval } from '../../core/nta/EvidenceLinks';
import { formatNumber } from '../../i18n/format';
import { dutchDefaultValue, dutchSource, snakeCase } from '../../core/nta/OpnameValueText';
import type { SurveyPart } from '../../core/survey/surveyFlow';
import { currentResult, useSurveyAssessment } from '../../core/survey/surveyAssessment';
import '../NtaPerformancePanel/NtaPerformancePanel.css';
import './BasisopnamePanel.css';
import '../shell/pages/existing/existing.css';

/** Survey design temperature classes: table 9.9 defaults and NTA table 9.14 rows (supply/return, °C). */
const DESIGN_CLASSES = ['c30_27', 'c35_30', 'c40_35', 'c45_40', 'c50_42', 'c55_47', 'c60_45', 'c60_50', 'c65_55', 'c70_50', 'c70_60', 'c75_65', 'c80_60', 'c90_70'];
/**
 * NTA table 9.28 test conditions per heat-pump source (NEN-EN 14511); the
 * kernel checks each measured COP against its minimum. Other sources have no
 * table 9.28 row.
 */
const TABLE_9_28_CONDITIONS: Record<string, string[]> = {
  ground: ['b0_w45', 'b0_w35'],
  groundwater: ['w10_w45', 'w10_w35'],
  outdoor_air: ['a7_wet6_w45', 'a7_wet6_w35', 'a_minus7_wet_minus8_w45'],
};
/** NTA table 9.28: the COP each test point must exceed (kernel forfait_heat_pump_draft.rs). */
const TABLE_9_28_MINIMUM: Record<string, number> = {
  b0_w45: 3.0, b0_w35: 3.5, w10_w45: 3.75, w10_w35: 4.4, a7_wet6_w45: 2.75, a7_wet6_w35: 2.85, a_minus7_wet_minus8_w45: 1.9,
};
/** Test standard edition the kernel accepts for table 9.28. */
const TABLE_9_28_TEST_STANDARD = 'NEN-EN 14511-2:2022';
/** Classes with a design supply temperature above 70 °C: a heat pump needs a controlled declaration. */
const DESIGN_CLASSES_ABOVE_70 = ['c75_65', 'c80_60', 'c90_70'];

// ISSO 82.1 (dwellings) and 75.1 (utility) basisopname: structured fields for
// the main survey sections, a JSON view for the rest, and the kernel route
// that derives the NTA 8800 input with the applied defaults (ISSO pages).

type Change = (path: Path, value: unknown) => void;
type T = (key: string, options?: Record<string, unknown>) => string;

export const HEATING_KINDS = ['boiler', 'heat_pump', 'local_fired', 'gas_air_heater', 'district_heat', 'electric', 'biomass',
  'chp', 'none_present'];
const HEAT_PUMP_SOURCES = ['outdoor_air', 'exhaust_air', 'outdoor_and_exhaust_air', 'heat_pump_panel', 'ground',
  'groundwater', 'surface_water', 'high_temperature', 'water_based_unknown'];
const WATER_SOURCES = ['ground', 'groundwater', 'surface_water', 'high_temperature', 'water_based_unknown'];
export const HOT_WATER_KINDS: Record<SurveyKind, string[]> = {
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

/** Photos of one survey item (BRL 9500 Bijlage 3), linked by JSON pointer into the stored survey. */
export function SurveyPhotos({ path }: { path: Path }) {
  return <EvidenceAttach photo pointer={surveyPointer(path)} />;
}

/** JSON pointer of a survey path in the project (`/basisopname/survey/...`). */
export function surveyPointer(path: Path): string {
  return jsonPointer(['basisopname', 'survey', ...path]);
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
/** Ventilation principles of the survey (ISSO 82.1 §11). */
export const VENTILATION_PRINCIPLES = ['natural', 'mechanical_extract', 'balanced', 'mechanical_supply'];

/**
 * The residential ventilation answers before the controls: the principle, the
 * heat recovery and bypass of a balanced system, the fans, and the supply
 * grilles of natural and extract systems (ISSO 82.1 tables 11.3–11.15).
 */
export function ResidentialVentilationBasics({ draft, change, t, hidePrinciple = false }: {
  draft: Draft; change: Change; t: T;
  /** The question flow asks the principle on its own page. */
  hidePrinciple?: boolean;
}) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const principle = read(draft, ['ventilation', 'principle']);
  const mechanical = principle !== 'natural';
  const grilles = principle === 'natural' || principle === 'mechanical_extract';
  return <>
    {!hidePrinciple && <SelectField {...field} path={['ventilation', 'principle']} label={t('survey.ventilation.principle')}
      options={opts(t, 'survey.ventilation.principleKind', VENTILATION_PRINCIPLES)} />}
    {principle === 'balanced' && <>
      <SelectField {...field} path={['ventilation', 'heatRecovery']} label={t('survey.ventilation.heatRecovery')}
        options={opts(t, 'opname.ventilation.heatRecoveryKind', HEAT_RECOVERY)} />
      <TriStateField {...field} {...yesNo} path={['ventilation', 'bypassPresent']} label={t('survey.ventilation.bypass')} />
    </>}
    {mechanical && <>
      <NumberField {...field} path={['ventilation', 'unitManufactureYear']} label={t('survey.ventilation.unitYear')} step="1" optional />
      <SelectField {...field} path={['ventilation', 'motor']} label={t('survey.ventilation.motor')}
        options={opts(t, 'survey.ventilation.motorKind', ['dc', 'ac', 'unknown'])} />
      <SelectField {...field} path={['ventilation', 'ductAirtightness']} label={t('survey.ventilation.ducts')}
        options={opts(t, 'survey.ventilation.ductsKind', ['luka_abc', 'luka_d', 'no_ducts', 'unknown'])} />
    </>}
    {grilles && <>
      <TriStateField {...field} {...yesNo} path={['ventilation', 'selfRegulatingVents']} label={t('survey.ventilation.selfRegulating')} />
      {read(draft, ['ventilation', 'selfRegulatingVents']) === true &&
        <SelectField {...field} path={['ventilation', 'pressureClass']} label={t('survey.ventilation.pressureClass')}
          options={opts(t, 'survey.ventilation.pressureClassKind', ['at_most1_pa', 'from1_to5_pa', 'from5_to10_pa'])} />}
    </>}
    <NumberField {...field} path={['ventilation', 'installationYear']} label={t('survey.ventilation.installationYear')} step="1" optional />
  </>;
}

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
/**
 * The survey section a default belongs to (its first path segment), in the UI
 * language; the full path stays next to it as a small reference.
 */
export function defaultPathLabel(t: T, path: string): string {
  const section = path.split(/[.[]/)[0];
  for (const key of [`opname.section.${section}`, `opname.${section}`]) {
    const text = t(key);
    if (text !== key) return text;
  }
  return section;
}

export function defaultValueLabel(t: T, value: string, locale = 'en') {
  if (value === 'true' || value === 'false') return t(value === 'true' ? 'common.yes' : 'common.no');
  // Recorded numbers (years, counts, areas) without thousands grouping: "1985", not "1.985".
  if (/^-?\d+(\.\d+)?$/.test(value)) {
    return Number(value).toLocaleString(locale, {
      useGrouping: false,
      minimumFractionDigits: value.split('.')[1]?.length ?? 0,
      maximumFractionDigits: value.split('.')[1]?.length ?? 0,
    });
  }
  const pascal = /^[A-Z][a-z0-9]+(?:[A-Z][a-z0-9]*)+$/.test(value);
  if (pascal || /^[a-z][a-z0-9]*(_[a-z0-9]+)+$/.test(value)) {
    const id = pascal ? snakeCase(value) : value;
    const key = `opname.value.${id}`;
    const text = t(key);
    const label = text === key ? id.replace(/_/g, ' ') : text;
    return <span title={value}>{label}</span>;
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

/** ISSO 75.1 p. 55–56: building type and position, for the infiltration value. */
const UTILITY_BUILDING_TYPES: Record<string, Draft> = {
  single_layer: { kind: 'single_layer', position: 'detached', roof: 'flat' },
  multi_layer_whole: { kind: 'multi_layer_whole' },
  multi_layer_part: { kind: 'multi_layer_part', level: 'intermediate', position: 'middle' },
};

export function UtilityBuildingTypeFields({ draft, change, t }: { draft: Draft; change: Change; t: T }) {
  const kind = read(draft, ['buildingType', 'kind']) as string | undefined;
  const select = (path: Path, label: string, prefix: string, keys: string[]) => <label>{label}
    <select value={String(read(draft, path) ?? '')} onChange={(event) => change(path, event.target.value)}>
      {keys.map((key) => <option key={key} value={key}>{t(`${prefix}.${key}`)}</option>)}
    </select></label>;
  return <>
    <label>{t('survey.buildingType')}
      <select value={kind ?? ''} onChange={(event) => change(['buildingType'], event.target.value ? { ...UTILITY_BUILDING_TYPES[event.target.value] } : undefined)}>
        <option value="">{t('survey.choose')}</option>
        {Object.keys(UTILITY_BUILDING_TYPES).map((key) => <option key={key} value={key}>{t(`survey.buildingType.${key}`)}</option>)}
      </select></label>
    {kind === 'single_layer' && <>
      {select(['buildingType', 'position'], t('survey.buildingType.position'), 'survey.buildingType.singlePosition', ['detached', 'end_or_corner', 'terraced'])}
      {select(['buildingType', 'roof'], t('survey.buildingType.roof'), 'survey.buildingType.roofKind', ['pitched', 'partly_flat', 'flat'])}
    </>}
    {kind === 'multi_layer_part' && <>
      {select(['buildingType', 'level'], t('survey.buildingType.level'), 'survey.buildingType.levelKind', ['bottom', 'intermediate', 'top'])}
      {select(['buildingType', 'position'], t('survey.buildingType.position'), 'survey.buildingType.partPosition', ['end_or_corner', 'middle', 'whole_storey'])}
    </>}
  </>;
}

/** ISSO 75.1 §7.2.1 (p. 63–64) and p. 39–40: the building's EP-liable use functions with A_g (NEN 2580). */
export function BuildingFunctionFields({ draft, change, t }: { draft: Draft; change: Change; t: T }) {
  const { locale } = useI18n();
  const field = { draft, onChange: change };
  const functions = list(draft, ['functions']);
  const total = functions.reduce((sum, item) => sum + (Number(item.areaM2) || 0), 0);
  return <div className="nta-form-subsection" role="group" aria-label={t('opname.functions')}>
    <strong>{t('opname.functions')}</strong>
    <p className="nta-form-note">{t('opname.functions.note')}</p>
    {functions.map((_, index) => <div key={index} className="opname-served">
      <SelectField {...field} path={['functions', index, 'function']} label={t('opname.zones.function')}
        options={opts(t, 'opname.functionKind', UTILITY_FUNCTIONS)} />
      <NumberField {...field} path={['functions', index, 'areaM2']} label={t('opname.functions.area')} />
      <RemoveButton label={t('opname.remove')}
        onRemove={() => change(['functions'], functions.filter((_, item) => item !== index))} />
    </div>)}
    <ListControls label={t('opname.zones.addFunction')}
      onAdd={() => change(['functions'], [...functions, { function: 'office', areaM2: 0 }])} />
    <p className="nta-form-note">{t('opname.functions.total', { area: formatNumber(total, locale, 1) })}</p>
  </div>;
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
    {ids.length > 1 && lighting.map((item, index) => <div key={`l${index}`} className="opname-served">
      <ZoneSelect draft={draft} change={change} path={['lighting', index, 'zoneId']}
        label={`${t('opname.zones.lightingZone')} ${String(item.id ?? index)}`} ids={ids} empty={t('opname.zones.notSet')}
        unknown={t('opname.zones.unknownZone')} />
      {/* The lighting zones of a calculation zone cover its A_g (14.3.4). */}
      <NumberField draft={draft} onChange={change} path={['lighting', index, 'areaM2']}
        label={t('opname.zones.lightingArea', { id: String(item.id ?? index) })} />
    </div>)}
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

export function HeatingGeneratorFields({ draft, path, change, t, hideKind = false }: {
  draft: Draft; path: Path; change: Change; t: T;
  /** The question flow asks the kind on its own page with choice cards. */
  hideKind?: boolean;
}) {
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
    // The table 9.28 test conditions differ per source.
    stale.push('highEfficiencyEvidence');
    change(path, { ...without(stale), source: value });
  };
  const table928Conditions = typeof source === 'string' ? TABLE_9_28_CONDITIONS[source] : undefined;
  const drive = read(draft, [...path, 'drive']);
  const showTable928 = table928Conditions != null && (drive == null || drive === 'electric');
  const evidencePath = [...path, 'highEfficiencyEvidence'];
  const hasEvidence = read(draft, evidencePath) != null;
  const changeTable928 = (checked: boolean) => change(path, checked
    ? {
      ...generator,
      highEfficiencyEvidence: {
        productReference: '',
        testReportReference: '',
        testStandardEdition: TABLE_9_28_TEST_STANDARD,
        points: (table928Conditions ?? []).map((condition) => ({ condition, measuredCop: null })),
      },
    }
    : without(['highEfficiencyEvidence']));
  const changeCollectiveSource = (checked: boolean) => {
    if (checked) change(path, { ...generator, collectiveSourceReference: '' });
    else change(path, without(source === 'surface_water'
      ? ['collectiveSourceReference', 'sourceTemperatureC', 'sourceTemperatureReference']
      : ['collectiveSourceReference']));
  };
  const changeAppliance = (_: Path, value: unknown) =>
    change(path, { ...(value === 'steam_boiler' ? generator : without(['fuel'])), appliance: value });
  return <>
    {!hideKind && <KindSelect draft={draft} path={path} label={t('opname.heating.generator')} kinds={HEATING_KINDS}
      prefix="opname.heating.kind" template={heatingGeneratorTemplate} change={change} t={t} />}
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
      <NumberField {...field} path={[...path, 'capacityKw']} label={`${t('opname.capacityKw')} (${t('survey.heating.capacityEmpty')})`} optional />
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
      {showTable928 && <>
        <label className="nta-form-check">
          <input type="checkbox" checked={hasEvidence} onChange={(event) => changeTable928(event.target.checked)} />
          {t('opname.heating.table928')}
        </label>
        {hasEvidence && <>
          <TextField {...field} path={[...evidencePath, 'productReference']} label={t('opname.heating.table928Product')} />
          <TextField {...field} path={[...evidencePath, 'testReportReference']} label={t('opname.heating.table928Report')} />
          <TextField {...field} path={[...evidencePath, 'testStandardEdition']} label={t('opname.heating.table928Standard')} />
          {(table928Conditions ?? []).map((condition, index) =>
            <NumberField key={condition} {...field} path={[...evidencePath, 'points', index, 'measuredCop']}
              label={`${t('opname.heating.table928Cop', { condition: t(`opname.heating.table928Condition.${condition}`) })} (${t('survey.heating.table928Minimum', { value: String(TABLE_9_28_MINIMUM[condition] ?? '').replace('.', ',') })})`} />)}
          <p className="nta-form-note">{t('opname.heating.table928Note')}</p>
          <p className="nta-form-note">{t('survey.heating.table928Declaration')}</p>
        </>}
      </>}
      <NumberField {...field} path={[...path, 'manufactureYear']} label={t('opname.manufactureYear')} step="1" />
      <NumberField {...field} path={[...path, 'installationYear']} label={t('opname.installationYear')} step="1" />
      <p className="nta-form-note">{t('opname.heating.heatPumpYearNote')}</p>
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

export function HotWaterGeneratorFields({ draft, path, kind, change, t, hideKind = false }: {
  draft: Draft; path: Path; kind: SurveyKind; change: Change; t: T;
  /** The question flow asks the kind on its own page with choice cards. */
  hideKind?: boolean;
}) {
  const field = { draft, onChange: change };
  const generator = read(draft, [...path, 'kind']);
  return <>
    {!hideKind && <KindSelect draft={draft} path={path} label={t('opname.hotWater.generator')} kinds={HOT_WATER_KINDS[kind]}
      prefix="opname.hotWater.kind" template={hotWaterGeneratorTemplate} change={change} t={t} />}
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

const INSULATION_KINDS = ['none_or_unknown', 'present_unknown_thickness', 'cavity_filled_unknown_width', 'thickness'];

/**
 * One insulation answer for all surfaces of an element (feedback 8 Oct 2026:
 * "isolatie per gevel niet relevant, geef het totaal in het begin"). ISSO 82.1
 * asks kind and thickness, not an Rc; the kernel derives the Rc per surface.
 * Party walls (adjacent heated) are left alone; each surface can still differ.
 */
export function ElementInsulation({ draft, element, replace, t }: {
  draft: Draft; element: 'facade' | 'roof' | 'floor'; replace: (next: Draft) => void; t: T;
}) {
  const surfaces = list(draft, ['envelope', 'surfaces']).map((surface, index) => ({ surface, index }))
    .filter(({ surface }) => surface.element === element && (surface.boundary as { kind?: string } | undefined)?.kind !== 'adjacent_heated');
  if (surfaces.length < 2) return null;
  const answers = surfaces.map(({ surface }) => JSON.stringify(surface.insulation ?? null));
  const same = answers.every((answer) => answer === answers[0]);
  const current = same ? surfaces[0].surface.insulation as { kind?: string; thicknessMm?: number } | undefined : undefined;
  const apply = (insulation: Record<string, unknown>) => {
    let next = draft;
    for (const { index } of surfaces) next = write(next, ['envelope', 'surfaces', index, 'insulation'], insulation);
    replace(next);
  };
  return <div className="opname-element-insulation">
    <label>{t(`survey.allSurfaces.${element}`)}
      <select value={current?.kind ?? ''} onChange={(event) => apply(event.target.value === 'thickness'
        ? { kind: 'thickness', thicknessMm: current?.thicknessMm ?? 50 } : { kind: event.target.value })}>
        {!same && <option value="">{t('survey.allSurfaces.mixed')}</option>}
        {INSULATION_KINDS.map((key) => <option key={key} value={key}>{t(`opname.surface.insulationKind.${key}`)}</option>)}
      </select>
    </label>
    {current?.kind === 'thickness' && <label>{t('opname.surface.thicknessMm')}
      <input type="number" step="10" min="0" value={current.thicknessMm ?? ''}
        onChange={(event) => apply({ kind: 'thickness', thicknessMm: Number(event.target.value) || 0 })} />
    </label>}
    <p className="nta-form-note nta-form-hint">{t('survey.allSurfaces.hint', { count: surfaces.length })}</p>
  </div>;
}

/** The Rc the kernel derived for a surface (from its source note, "R_c 1.25"), or null. */
export function derivedRc(result: OpnameAssessment | null, surfaceId: string): number | null {
  const elements = ((result?.derivedInput as { opaqueElements?: Array<{ id?: string; sourceReference?: string }> } | null)?.opaqueElements) ?? [];
  const note = elements.find((item) => item.id === surfaceId)?.sourceReference ?? '';
  const match = /R_c (-?\d+(?:\.\d+)?)/.exec(note);
  return match ? Number(match[1]) : null;
}

function list(draft: Draft, path: Path): Array<Record<string, unknown>> {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Array<Record<string, unknown>> : [];
}

/** Survey wizard sections (UI redesign F8); the order of the progress list. */
export const SURVEY_SECTIONS = ['general', 'zones', 'envelope', 'heating', 'hotWater', 'ventilation', 'cooling', 'pv', 'result'] as const;
export type SurveySection = typeof SURVEY_SECTIONS[number];

/** The wizard section of a survey path (`envelope.surfaces[0]…`, with or without `basisopname.`/`derivedInput.`). */
export function surveySectionForPath(path: string | null | undefined): SurveySection {
  const head = (path ?? '').replace(/^basisopname\./, '').split(/[.[]/)[0];
  switch (head) {
    case 'zones': case 'lighting': return 'zones';
    case 'envelope': return 'envelope';
    case 'heating': return 'heating';
    case 'hotWater': case 'additionalHotWaterSystems': return 'hotWater';
    case 'ventilation': return 'ventilation';
    case 'cooling': case 'coolingPresent': case 'coolingCollective': return 'cooling';
    case 'pv': return 'pv';
    case 'derivedInput': case '': return 'result';
    default: return 'general';
  }
}

/**
 * Feedback after "Opname doorrekenen" in the section the user is on: the
 * status, the issues of this section with "Ga naar", and the way to the full
 * outcome. Without it a calculation from Algemeen showed nothing in the page.
 */
function SectionOutcome({ result, section, goTo, showResult }: {
  result: OpnameAssessment; section: SurveySection; goTo: (path: string) => void; showResult: () => void;
}) {
  const { t, locale } = useI18n();
  const here = result.issues.filter((item) => surveySectionForPath(item.path) === section);
  const label = result.performance?.indicativeLabelClass;
  const ep = result.performance?.primaryFossilIndicatorKwhPerM2Year;
  return <div className="opname-section-outcome" role="status" aria-label={t('opname.sectionOutcome')}>
    <p>
      <strong>{t('opname.sectionOutcome.calculated')}</strong>{' '}
      {t(`opname.statusValue.${result.status}`, { defaultValue: result.status })}
      {label != null && <> · {t('opname.label')} <strong>{label}</strong></>}
      {ep != null && <> · {formatNumber(ep, locale, 1)} {t('unit.kwhPerM2Year')} EP₂</>}
    </p>
    {here.length === 0
      ? <p className="nta-form-note">{t('opname.sectionOutcome.none', { section: t(`opname.section.${section}`) })}</p>
      : <ul className="opname-issues">
        {here.map((item, index) => <li key={index}>
          <KernelCode code={item.code} prefixes={['opname.issue.', 'nta.gap.', 'kernel.issue.']} />{' '}
          <button type="button" className="btn btn-sm opname-goto" onClick={() => goTo(item.path)}>
            {t('opname.sectionOutcome.field')}</button>
        </li>)}
      </ul>}
    {result.issues.length > here.length && <p className="nta-form-note">
      {t('opname.sectionOutcome.elsewhere', { count: result.issues.length - here.length })}</p>}
    <button type="button" className="btn btn-sm" onClick={showResult}>{t('opname.sectionOutcome.show')}</button>
  </div>;
}

interface BasisopnamePanelProps {
  /** Wizard mode (shell step): only this section, with the progress list and result card. */
  section?: SurveySection;
  onSection?: (section: SurveySection, focusPath?: string) => void;
  /**
   * Question mode (the question flow): only the fields of this part, without
   * heading, progress, result or actions; the question page around it has those.
   */
  part?: SurveyPart;
}

/** The panel section that holds a question part; null for parts the question page draws itself. */
const PART_SECTION: Record<SurveyPart, SurveySection | null> = {
  address: null, dwellingType: null, general: 'general', zones: 'zones', walls: 'envelope', roofFloor: 'envelope',
  heatingKind: null, heatingGenerator: 'heating', heatingRest: 'heating',
  hotWaterKind: null, hotWaterGenerator: 'hotWater', hotWaterRest: 'hotWater',
  ventilationPrinciple: null, ventilation: 'ventilation', cooling: 'cooling', pv: 'pv',
};

/** A new survey surface of an element, for the question pages (facades on a wall question, roofs and floors on the other). */
function surfaceTemplate(index: number, element: 'facade' | 'roof' | 'floor'): Record<string, unknown> {
  const base = { id: `${element === 'facade' ? 'gevel' : element === 'roof' ? 'dak' : 'vloer'}-${index + 1}`, element, cavity: false,
    insulation: { kind: 'none_or_unknown' }, grossAreaM2: 10, sourceReference: '' };
  if (element === 'facade') return { ...base, boundary: { kind: 'outdoor' }, orientation: 'south' };
  if (element === 'roof') return { ...base, boundary: { kind: 'outdoor' }, orientation: 'south', tiltDeg: 45 };
  return { ...base, boundary: { kind: 'ground' }, exposedPerimeterM: 10 };
}

export function BasisopnamePanel({ section: requested, onSection, part }: BasisopnamePanelProps = {}) {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const stored = state.project.basisopname as StoredSurvey | undefined;
  const edition = state.project.ntaCalculation?.normVersion ?? null;
  const [result, setResult] = useState<OpnameAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [json, setJson] = useState<string | null>(null);
  const requestId = useRef(0);
  // The shared survey outcome of the question flow, for the derived Rc per surface.
  const surveyResult = currentResult(useSurveyAssessment(), stored);

  const save = (next: StoredSurvey | undefined, keepResult = false) => {
    // The question-flow progress lives beside the survey; an edit here must not drop it.
    dispatch({ type: 'SET_BASISOPNAME', payload: next && stored ? { ...stored, ...next } : next });
    if (!keepResult) {
      requestId.current += 1;
      setResult(null);
      setError(null);
      setBusy(false);
    }
  };

  if (!stored) {
    return <section className="nta-performance opname-panel" aria-label={t('opname.title')}>
      <h2>{t('opname.title')}</h2>
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
  // Removes a survey item and keeps the photo links of the other items on their item
  // (items without an id are linked by position, so the ones after it move up).
  const removeAt = (path: Path, index: number) => {
    const next: StoredSurvey = { kind, survey: write(draft, path, list(draft, path).filter((_, item) => item !== index)) };
    const registration = state.project.registration;
    const evidence = registration?.evidence ?? [];
    save(next);
    if (evidence.length === 0) return;
    const moved = linksAfterRemoval(evidence, state.project, { ...state.project, basisopname: next },
      surveyPointer(path), index);
    if (moved.some((item, position) => item !== evidence[position])) {
      dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: { ...registration, evidence: moved } } });
    }
  };
  const field = { draft, onChange: change };
  const surfaces = list(draft, ['envelope', 'surfaces']);
  const windows = list(draft, ['envelope', 'windows']);
  const rooflights = list(draft, ['envelope', 'rooflights']);
  const doors = list(draft, ['envelope', 'doors']);
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
      // The survey is calculated in the project's edition (`ntaCalculation.normVersion`).
      const assessment = kind === 'residential'
        ? await assessResidentialSurveyWithRust(asResidential(stored, edition))
        : await assessUtilitySurveyWithRust(asUtility(stored, edition));
      // A survey the kernel cannot read (e.g. a cleared required field) comes back as
      // `{ error, message }` without issues; show it as an error, not as a result.
      const refused = assessment as Partial<OpnameAssessment> & { error?: string; message?: string };
      if (!Array.isArray(refused.issues)) throw new Error(refused.message ?? refused.error ?? 'invalid response');
      if (requestId.current === current) setResult(assessment);
    } catch (failure) {
      if (requestId.current === current) setError(failure instanceof Error ? failure.message : String(failure));
    } finally {
      if (requestId.current === current) setBusy(false);
    }
  };

  const performance = result?.performance;
  const embedded = part != null;
  const wizard = requested != null && !embedded;
  const sections = SURVEY_SECTIONS.filter((name) => name !== 'zones' || kind === 'utility');
  // Rekenzones only exist in the utility survey; a residential survey opens Algemeen instead.
  const section = requested && sections.includes(requested) ? requested : wizard ? sections[0] : undefined;
  const show = (name: SurveySection) => embedded ? PART_SECTION[part] === name : !wizard || section === name;
  // Question mode splits a section over several questions.
  const showSurface = (element: unknown) => part === 'walls' ? element === 'facade'
    : part === 'roofFloor' ? element !== 'facade' : true;
  const heatingGenerator = !embedded || part === 'heatingGenerator';
  const heatingRest = !embedded || part === 'heatingRest';
  const hotWaterGenerator = !embedded || part === 'hotWaterGenerator';
  const hotWaterRest = !embedded || part === 'hotWaterRest';
  const issueCount = (name: SurveySection) => result?.issues.filter((item) => surveySectionForPath(item.path) === name).length ?? 0;
  const goTo = (path: string) => onSection?.(surveySectionForPath(path), `basisopname.${path.replace(/^basisopname\./, '')}`);
  const position = section ? sections.indexOf(section) : -1;
  const done = result ? sections.filter((name) => name !== 'result' && issueCount(name) === 0).length : 0;

  return <section className={`nta-performance opname-panel${wizard ? ' opname-wizard' : ''}${embedded ? ' opname-embedded' : ''}`} aria-label={t('opname.title')}>
    {wizard && <nav className="opname-progress" aria-label={t('opname.progress')}>
      <p className="opname-progress-title">{t('opname.progress')}</p>
      <div className="opname-progress-bar" role="progressbar" aria-label={t('opname.progress')}
        aria-valuemin={0} aria-valuemax={sections.length - 1} aria-valuenow={done}>
        <span style={{ width: `${Math.round(100 * done / Math.max(1, sections.length - 1))}%` }} />
      </div>
      <p className="opname-progress-note">{result
        ? t('opname.progress.summary', { done, total: sections.length - 1, defaults: result.appliedDefaults.length })
        : t('opname.progress.notCalculated')}</p>
      <ol className="opname-steps">
        {sections.map((name) => {
          const errors = issueCount(name);
          const state = name === 'result' ? (result ? 'done' : 'todo') : !result ? 'todo' : errors > 0 ? 'errors' : 'done';
          return <li key={name}>
            <button type="button" className={`opname-step opname-step--${state}`} aria-current={section === name ? 'step' : undefined}
              onClick={() => onSection?.(name)}>
              <span className="opname-step-dot" aria-hidden="true">{state === 'done' ? '✓' : state === 'errors' ? errors : ''}</span>
              <span className="opname-step-label">{t(`opname.section.${name}`)}</span>
              {state === 'errors' && <span className="visually-hidden">{t('opname.section.errors', { count: errors })}</span>}
            </button>
          </li>;
        })}
      </ol>
    </nav>}
    <div className="opname-main">
    {!embedded && <h2>{t('opname.title')} — {t(`opname.kind.${kind}`)}</h2>}
    {!embedded && <p className="nta-form-note">{t('opname.scope')}</p>}
    {wizard && section && section !== 'result' && result && <SectionOutcome result={result} section={section}
      goTo={goTo} showResult={() => onSection?.('result')} />}

    <FieldPathPrefixProvider value="basisopname">
    <div className="nta-form">
    {show('general') && <><Section title={t('opname.general')}>
      <TextField {...field} path={['id']} label={t('opname.id')} />
      <label>{t('survey.surveyDate')}
        <input type="date" value={stored.surveyDate ?? ''}
          onChange={(event) => dispatch({ type: 'SET_BASISOPNAME', payload: { ...stored, surveyDate: event.target.value || undefined } })} />
      </label>
      <NumberField {...field} path={['constructionYear']} label={t('opname.constructionYear')} step="1" />
      <TextField {...field} path={['sourceReference']} label={t('survey.sourceReference')} />
      <p className="nta-form-note nta-form-hint">{t('survey.sourceReference.hint')}</p>
      <p className="nta-form-subhead">{t('survey.group.size')}</p>
      {kind === 'residential' && <NumberField {...field} path={['usableFloorAreaM2']} label={t('opname.usableFloorArea')} />}
      <NumberField {...field} path={['buildingHeightM']} label={t('opname.buildingHeight')} />
      <NumberField {...field} path={['storeys']} label={t('opname.storeys')} step="1" />
      {kind === 'residential' && read(draft, ['dwelling', 'kind']) === 'apartment' &&
        <SelectField {...field} path={['dwelling', 'floor']} label={t('opname.dwelling.floor')}
          options={opts(t, 'opname.dwelling.floorKind', ['ground_or_intermediate', 'top', 'roof_and_floor'])} />}
      {kind === 'utility' && <UtilityBuildingTypeFields draft={draft} change={change} t={t} />}
      {kind === 'utility' && <BuildingFunctionFields draft={draft} change={change} t={t} />}
      {kind === 'utility' && <>
        <NumberField {...field} path={['toiletStacks']} label={t('opname.toiletStacks')} step="1" />
        <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={['fossilFuelOnPlot']} label={t('opname.fossilFuelOnPlot')} />
        <NumberField {...field} path={['sportHallAreaM2']} label={t('opname.sportHallArea')} />
        <NumberField {...field} path={['swimmingPoolAreaM2']} label={t('opname.swimmingPoolArea')} />
        <CheckField {...field} path={['openlyConnectedResidenceAreas']} label={t('opname.openlyConnected')} />
      </>}
      <p className="nta-form-subhead">{t('survey.group.construction')}</p>
      <SelectField {...field} path={['construction', 'floor']} label={t('survey.construction.floor')}
        options={opts(t, 'survey.construction.floorKind', ['light', 'heavy', 'very_heavy'])} />
      <SelectField {...field} path={['construction', 'wall']} label={t('survey.construction.wall')}
        options={opts(t, 'survey.construction.wallKind', ['light', 'heavy', 'very_heavy'])} />
      <CheckField {...field} path={['construction', 'lighterCeiling']} label={t('survey.construction.lighterCeiling')} />
      <CheckField {...field} path={['construction', 'closedOrSuspendedCeiling']} label={t('survey.construction.closedCeiling')} />
      <p className="nta-form-subhead">{t('survey.group.pipes')}</p>
      <label>{t('opname.verticalPipes')}
        <select value={verticalPipes == null ? 'default' : Array.isArray(verticalPipes) && verticalPipes.length === 0 ? 'none' : 'count'}
          onChange={(event) => change(['verticalPipes'], event.target.value === 'default' ? null
            : event.target.value === 'none' ? [] : [{ insulated: null }])}>
          <option value="default">{t('opname.verticalPipes.default')}</option>
          <option value="none">{t('opname.verticalPipes.none')}</option>
          <option value="count">{t('opname.verticalPipes.count')}</option>
        </select>
      </label>
      {Array.isArray(verticalPipes) && verticalPipes.length > 0 && <label>{t('opname.verticalPipes.number')}
        <input type="number" step="1" min="1" value={verticalPipes.length}
          onChange={(event) => change(['verticalPipes'], Array.from({ length: Math.max(1, Number(event.target.value) || 1) },
            (_, index) => (verticalPipes as unknown[])[index] ?? { insulated: null }))} />
      </label>}
      {Array.isArray(verticalPipes) && verticalPipes.map((_, index) => <div key={`vp${index}`} className="opname-item opname-item--plain">
        <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={['verticalPipes', index, 'insulated']}
          label={t('survey.verticalPipe.insulated', { n: index + 1 })} />
        <NumberField {...field} path={['verticalPipes', index, 'sharedZones']} label={t('survey.verticalPipe.sharedZones', { n: index + 1 })}
          step="1" optional />
      </div>)}
    </Section></>}

    {show('zones') && <>{kind === 'utility' && <Section title={t('opname.zones')}>
      <CalculationZoneFields draft={draft} change={change} replace={(next) => save({ kind, survey: next })} t={t} />
    </Section>}</>}

    {show('envelope') && <><Section title={part === 'walls' ? t('survey.section.walls') : part === 'roofFloor' ? t('survey.section.roofFloor') : t('opname.envelope')}>
      <p className="nta-form-note">{t('evidenceLink.photosHint')}</p>
      {kind === 'residential' && part !== 'roofFloor' && <label>{t('opname.buildingKind')}
        <select value={typeof buildingKind === 'string' ? buildingKind : 'regular'}
          onChange={(event) => change(['envelope', 'buildingKind'], event.target.value === 'regular' ? null
            : event.target.value === 'floating' ? { kind: 'floating', newBerthSince2018: false } : { kind: event.target.value })}>
          {['regular', 'caravan', 'floating'].map((key) =>
            <option key={key} value={key}>{t(`opname.buildingKind.${key}`)}</option>)}
        </select>
      </label>}
      {buildingKind === 'floating' && part !== 'roofFloor' &&
        <CheckField {...field} path={['envelope', 'buildingKind', 'newBerthSince2018']} label={t('opname.buildingKind.newBerth')} />}
      {(['facade', 'roof', 'floor'] as const).filter((element) => showSurface(element)).map((element) =>
        <ElementInsulation key={element} draft={draft} element={element} replace={(next) => save({ kind, survey: next })} t={t} />)}
      {(() => {
        // Feedback 8 Oct 2026: every surface is a card with its windows, doors and rooflights
        // right below it, so "In vlak" no longer has to be picked to see what belongs where.
        const counts: Record<string, number> = {};
        const names = new Map<string, string>();
        surfaces.forEach((surface) => {
          const element = String(surface.element ?? 'facade');
          counts[element] = (counts[element] ?? 0) + 1;
          names.set(String(surface.id ?? ''), `${t(`survey.element.${element}`)} ${counts[element]}`);
        });
        const surfaceOptions: Array<[string, string]> = surfaces.map((surface) => [String(surface.id), names.get(String(surface.id)) ?? String(surface.id)]);
        const area = (items: Array<Record<string, unknown>>, id: string) =>
          items.filter((item) => item.surfaceId === id).reduce((sum, item) => sum + (Number(item.areaM2) || 0), 0);
        const glassOptions = opts(t, 'opname.window.glassKind', ['triple_hr', 'hr_plus_plus', 'hr_plus', 'hr', 'double', 'single']);
        const frameOptions = opts(t, 'opname.window.frameKind', ['wood_or_plastic', 'metal_with_thermal_break', 'metal', 'none']);
        const windowFields = (_window: Record<string, unknown>, index: number) => {
          const base: Path = ['envelope', 'windows', index];
          const situation = read(draft, [...base, 'shading', 'situation']);
          return <div key={`w${index}`} className="opname-sub">
            <p className="opname-sub-kind">{t('survey.window')} {index + 1}</p>
            <div className="nta-form-grid">
              <NumberField {...field} path={[...base, 'areaM2']} label={t('opname.window.area')} />
              <SelectField {...field} path={[...base, 'glass']} label={t('opname.window.glass')} options={glassOptions} />
              <SelectField {...field} path={[...base, 'frame']} label={t('opname.window.frame')} options={frameOptions} />
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
              <SelectField {...field} path={[...base, 'surfaceId']} label={t('survey.moveTo')} options={surfaceOptions} />
              <SolarControlField draft={draft} base={base} change={change} t={t} />
              <SurveyPhotos path={base} />
              <RemoveButton label={t('opname.remove')} onRemove={() => removeAt(['envelope', 'windows'], index)} />
            </div>
          </div>;
        };
        const doorFields = (_door: Record<string, unknown>, index: number) => {
          const base: Path = ['envelope', 'doors', index];
          const glazed = Number(read(draft, [...base, 'glassFraction']) ?? 0) > 0;
          return <div key={`d${index}`} className="opname-sub">
            <p className="opname-sub-kind">{t('survey.door')} {index + 1}</p>
            <div className="nta-form-grid">
              <NumberField {...field} path={[...base, 'areaM2']} label={t('survey.door.area')} />
              <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={[...base, 'insulated']} label={t('survey.door.insulated')} />
              <NumberField {...field} path={[...base, 'glassFraction']} label={t('survey.door.glassFraction')} optional />
              {glazed && <SelectField {...field} path={[...base, 'glass']} label={t('opname.window.glass')} options={glassOptions} />}
              <SelectField {...field} path={[...base, 'frame']} label={t('opname.window.frame')} options={frameOptions} />
              <SelectField {...field} path={[...base, 'surfaceId']} label={t('survey.moveTo')} options={surfaceOptions} />
              <SurveyPhotos path={base} />
              <RemoveButton label={t('opname.remove')} onRemove={() => removeAt(['envelope', 'doors'], index)} />
            </div>
          </div>;
        };
        const rooflightFields = (_rooflight: Record<string, unknown>, index: number) => {
          const base: Path = ['envelope', 'rooflights', index];
          return <div key={`r${index}`} className="opname-sub">
            <p className="opname-sub-kind">{t('survey.rooflight')} {index + 1}</p>
            <div className="nta-form-grid">
              <NumberField {...field} path={[...base, 'areaM2']} label={t('opname.rooflight.area')} />
              <NumberField {...field} path={[...base, 'uValue']} label={t('opname.rooflight.uValue')} />
              <SelectField {...field} path={[...base, 'glass']} label={t('opname.window.glass')} options={glassOptions} />
              <TextField {...field} path={[...base, 'qualityDeclarationReference']} label={t('opname.rooflight.declaration')} />
              <SurveyPhotos path={base} />
              <RemoveButton label={t('opname.remove')} onRemove={() => removeAt(['envelope', 'rooflights'], index)} />
            </div>
          </div>;
        };
        const cards = surfaces.map((surface, index) => {
          if (!showSurface(surface.element)) return null;
          const base: Path = ['envelope', 'surfaces', index];
          const id = String(surface.id ?? '');
          const element = String(read(draft, [...base, 'element']) ?? 'facade');
          const insulation = read(draft, [...base, 'insulation', 'kind']);
          const thickness = read(draft, [...base, 'insulation', 'thicknessMm']);
          const orientation = read(draft, [...base, 'orientation']);
          // A roof up to 5° is flat: no orientation (the kernel asks one only above 5°).
          const oriented = element === 'facade' || (element === 'roof' && Number(read(draft, [...base, 'tiltDeg']) ?? 0) > 5);
          const ownWindows = windows.map((item, at) => [item, at] as const).filter(([item]) => item.surfaceId === id);
          const ownDoors = doors.map((item, at) => [item, at] as const).filter(([item]) => item.surfaceId === id);
          const ownRooflights = rooflights.map((item, at) => [item, at] as const).filter(([item]) => item.surfaceId === id);
          const gross = Number(surface.grossAreaM2) || 0;
          const net = gross - area(windows, id) - area(doors, id) - area(rooflights, id);
          return <details key={index} className="opname-card" open>
            <summary className="opname-card-head">
              <strong>{names.get(id) ?? id}</strong>
              {oriented && typeof orientation === 'string' &&
                <span className="opname-tag">{t(`opname.orientationKind.${orientation}`)}</span>}
              {element === 'roof' && <span className="opname-tag">{formatNumber(Number(read(draft, [...base, 'tiltDeg']) ?? 0), locale, 0)}°</span>}
              {derivedRc(surveyResult, id) != null && <span className="opname-tag" title={t('survey.derivedRcHint')}>R_c {formatNumber(derivedRc(surveyResult, id), locale, 2)}</span>}
              {typeof insulation === 'string' && <span className="opname-tag">{insulation === 'thickness' && typeof thickness === 'number'
                ? `${formatNumber(thickness, locale, 0)} mm` : t(`opname.surface.insulationKind.${insulation}`)}</span>}
              {ownWindows.length > 0 && <span className="opname-tag">{t('survey.windowsCount', { count: ownWindows.length })}</span>}
              {ownDoors.length > 0 && <span className="opname-tag">{t('survey.doorsCount', { count: ownDoors.length })}</span>}
              <span className={`opname-card-net${net < 0 ? ' opname-card-net--invalid' : ''}`}>
                {t('survey.grossNet', { gross: formatNumber(gross, locale, 2), net: formatNumber(net, locale, 2) })}</span>
            </summary>
            <div className="nta-form-grid opname-card-body">
              {!embedded && <SelectField {...field} path={[...base, 'element']} label={t('opname.surface.element')}
                options={opts(t, 'opname.surface.elementKind', ['facade', 'roof', 'floor'])} />}
              {element === 'roof' && <NumberField {...field} path={[...base, 'tiltDeg']} label={t('opname.tilt')} />}
              {oriented && <SelectField {...field} path={[...base, 'orientation']} label={t('opname.orientation')}
                options={opts(t, 'opname.orientationKind', ORIENTATIONS)} />}
              {element === 'roof' && !oriented && <p className="nta-form-note nta-form-hint">{t('survey.flatRoofNoOrientation')}</p>}
              {element === 'floor' && <NumberField {...field} path={[...base, 'exposedPerimeterM']} label={t('opname.surface.perimeter')} />}
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
              {insulation === 'present_unknown_thickness' && <>
                <NumberField {...field} path={[...base, 'renovation', 'year']} label={t('opname.surface.renovationYear')} step="1" />
                <CheckField {...field} path={[...base, 'renovation', 'meetsRequirementsOfYear']} label={t('opname.surface.renovationEvidence')} />
              </>}
              <CheckField {...field} path={[...base, 'thermalCushions']} label={t('opname.surface.thermalCushions')} />
              <SurveyPhotos path={base} />
            </div>
            {ownWindows.map(([item, at]) => windowFields(item, at))}
            {ownDoors.map(([item, at]) => doorFields(item, at))}
            {ownRooflights.map(([item, at]) => rooflightFields(item, at))}
            <div className="opname-card-actions">
              {element !== 'floor' && <button type="button" className="btn btn-sm"
                onClick={() => change(['envelope', 'windows'], [...windows, windowTemplate(windows.length, id)])}>
                {t(element === 'roof' ? 'survey.addRoofWindowHere' : 'survey.addWindowHere')}</button>}
              {element === 'facade' && <button type="button" className="btn btn-sm"
                onClick={() => change(['envelope', 'doors'], [...doors, {
                  id: `deur-${doors.length + 1}`, surfaceId: id, areaM2: 2, insulated: null, glassFraction: 0,
                  frame: 'wood_or_plastic', sourceReference: '',
                }])}>{t('survey.addDoorHere')}</button>}
              {element === 'roof' && <button type="button" className="btn btn-sm"
                onClick={() => change(['envelope', 'rooflights'], [...rooflights, {
                  id: `lichtkoepel-${rooflights.length + 1}`, surfaceId: id, areaM2: 1, uValue: 2.5, glass: 'double', qualityDeclarationReference: '',
                }])}>{t('opname.addRooflight')}</button>}
              <RemoveButton label={t('survey.removeSurface')} onRemove={() => {
                // Openings on the removed surface go with it: the kernel rejects an opening without its surface.
                const keep = (items: Array<Record<string, unknown>>) => items.filter((item) => item.surfaceId !== id);
                let next = write(draft, ['envelope', 'surfaces'], surfaces.filter((_, item) => item !== index));
                next = write(next, ['envelope', 'windows'], keep(windows));
                next = write(next, ['envelope', 'rooflights'], keep(rooflights));
                if (doors.length > 0) next = write(next, ['envelope', 'doors'], keep(doors));
                save({ kind, survey: next });
              }} />
            </div>
          </details>;
        });
        // Openings without a (shown) surface: a mistake to fix, so they stay visible with "In vlak".
        const known = new Set(surfaces.map((surface) => String(surface.id ?? '')));
        const loose = [
          ...windows.map((item, at) => [item, at, 'w'] as const),
          ...(part !== 'roofFloor' ? doors.map((item, at) => [item, at, 'd'] as const) : []),
        ].filter(([item]) => !known.has(String(item.surfaceId ?? '')));
        return <>
          {cards}
          {loose.length > 0 && <div className="opname-card opname-card--loose">
            <p className="opname-card-head"><strong>{t('survey.unlinked')}</strong></p>
            {loose.map(([item, at, type]) => type === 'w' ? windowFields(item, at) : doorFields(item, at))}
          </div>}
        </>;
      })()}
      <div className="opname-list-controls">
        {part !== 'roofFloor' && <button type="button" className="btn"
          onClick={() => change(['envelope', 'surfaces'], [...surfaces, surfaceTemplate(surfaces.length, 'facade')])}>{t('survey.addFacade')}</button>}
        {part !== 'walls' && <>
          <button type="button" className="btn"
            onClick={() => change(['envelope', 'surfaces'], [...surfaces, surfaceTemplate(surfaces.length, 'roof')])}>{t('survey.addRoof')}</button>
          <button type="button" className="btn"
            onClick={() => change(['envelope', 'surfaces'], [...surfaces, surfaceTemplate(surfaces.length, 'floor')])}>{t('survey.addFloor')}</button>
        </>}
      </div>
    </Section></>}

    {show('heating') && <><Section title={part === 'heatingRest' ? t('survey.section.heatingRest') : t('opname.heating')}>
      {heatingGenerator && <>
      <HeatingGeneratorFields draft={draft} path={['heating', 'generator']} change={change} t={t} hideKind={part === 'heatingGenerator'} />
      <NumberField {...field} path={['heating', 'nominalPowerKw']} label={`${t('opname.nominalPowerKw')} (${t('survey.onlyWithMoreGenerators')})`}
        disabled={heatingExtras.length === 0} />
      <SurveyPhotos path={['heating', 'generator']} />
      </>}
      {heatingRest && <>
      <EmitterFields draft={draft} change={change} t={t} />
      {kind === 'utility' && <>
        <CheckField {...field} path={['heatingInstallation', 'collective']} label={t('survey.heatingInstallation.collective')} />
        <NumberField {...field} path={['heatingInstallation', 'capacityKw']} label={t('survey.heatingInstallation.capacity')} optional />
      </>}
      <SelectField {...field} path={['heating', 'designClass']} label={t('opname.heating.designClass')}
        options={opts(t, 'opname.heating.designClassKind', DESIGN_CLASSES)} />
      {DESIGN_CLASSES_ABOVE_70.includes(String(read(draft, ['heating', 'designClass']))) &&
        <TextField {...field} path={['heating', 'heatPumpAbove70Declaration']} label={t('opname.heating.above70Declaration')} />}
      <DistributionFields draft={draft} change={change} t={t} />
      <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={['heating', 'balanced']} label={t('survey.heating.balanced')} />
      <SelectField {...field} path={['heating', 'control']} label={t('survey.heating.control')}
        options={opts(t, 'survey.heating.controlKind', ['room_thermostat', 'central_with_radiator_valves', 'individual_room_control', 'unknown'])}
        onChange={(path, value) => change(path, value ?? 'unknown')} />
      <label>{t('survey.heating.unheatedPipes')}
        <select value={String(read(draft, ['heating', 'unheatedPipes', 'kind']) ?? '')}
          onChange={(event) => change(['heating', 'unheatedPipes'], event.target.value === '' ? null : { kind: event.target.value })}>
          <option value="">{t('survey.heating.unheatedPipesKind.unknown')}</option>
          <option value="absent">{t('survey.heating.unheatedPipesKind.absent')}</option>
          <option value="present">{t('survey.heating.unheatedPipesKind.present')}</option>
        </select>
      </label>
      {read(draft, ['heating', 'unheatedPipes', 'kind']) === 'present' &&
        <NumberField {...field} path={['heating', 'unheatedPipes', 'lengthM']} label={t('survey.heating.unheatedPipesLength')} optional />}
      <NumberField {...field} path={['heating', 'storeys']} label={t('survey.heating.storeys')} step="1" optional />
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
      </>}
      {heatingGenerator && <>
      {heatingExtras.map((_, index) => <div key={index} className="opname-item">
        <strong>{t('opname.additionalGenerator')} {index + 1}</strong>
        <HeatingGeneratorFields draft={draft} path={['heating', 'additionalGenerators', index, 'generator']} change={change} t={t} />
        <NumberField {...field} path={['heating', 'additionalGenerators', index, 'nominalPowerKw']} label={t('opname.nominalPowerKw')} />
        <SurveyPhotos path={['heating', 'additionalGenerators', index]} />
        <RemoveButton label={t('opname.remove')} onRemove={() => removeAt(['heating', 'additionalGenerators'], index)} />
      </div>)}
      <ListControls label={t('opname.addGenerator')}
        onAdd={() => change(['heating', 'additionalGenerators'], [...heatingExtras, { id: freshItemId(heatingExtras, 'opwekker'), generator: heatingGeneratorTemplate('boiler'), nominalPowerKw: 20 }])} />
      </>}
    </Section></>}

    {show('hotWater') && <><Section title={part === 'hotWaterRest' ? t('survey.section.hotWaterRest') : t('opname.hotWater')}>
      {hotWaterGenerator && <>
      <HotWaterGeneratorFields draft={draft} path={['hotWater', 'generator']} kind={kind} change={change} t={t} hideKind={part === 'hotWaterGenerator'} />
      <NumberField {...field} path={['hotWater', 'nominalPowerKw']} label={`${t('opname.nominalPowerKw')} (${t('survey.onlyWithMoreGenerators')})`}
        disabled={hotWaterExtras.length === 0} />
      <SurveyPhotos path={['hotWater', 'generator']} />
      {kind === 'residential' && read(draft, ['hotWater', 'generator', 'kind']) === 'heat_pump' &&
        <p className="nta-form-note">{t('survey.hotWater.heatPumpVessel')}</p>}
      </>}
      {hotWaterRest && kind === 'residential' && <>
        <SelectField {...field} path={['hotWater', 'served']} label={t('survey.hotWater.served')}
          options={opts(t, 'survey.hotWater.servedKind', ['kitchen_and_bathroom', 'bathroom_only', 'kitchen_only'])} />
        <NumberField {...field} path={['hotWater', 'kitchenLengthM']} label={t('survey.hotWater.kitchenLength')} optional />
        <NumberField {...field} path={['hotWater', 'bathroomLengthM']} label={t('survey.hotWater.bathroomLength')} optional />
        <NumberField {...field} path={['hotWater', 'showers']} label={t('survey.hotWater.showers')} step="1" />
        <SelectField {...field} path={['hotWater', 'showerHeatRecovery']} label={t('survey.hotWater.showerHeatRecovery')}
          options={opts(t, 'survey.hotWater.showerHeatRecoveryKind', ['none', 'vertical', 'horizontal', 'unknown'])} />
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
      {hotWaterGenerator && <>
      {hotWaterExtras.map((_, index) => <div key={index} className="opname-item">
        <strong>{t('opname.additionalGenerator')} {index + 1}</strong>
        <HotWaterGeneratorFields draft={draft} path={['hotWater', 'additionalGenerators', index, 'generator']} kind={kind} change={change} t={t} />
        <NumberField {...field} path={['hotWater', 'additionalGenerators', index, 'nominalPowerKw']} label={t('opname.nominalPowerKw')} />
        <SurveyPhotos path={['hotWater', 'additionalGenerators', index]} />
        <RemoveButton label={t('opname.remove')} onRemove={() => removeAt(['hotWater', 'additionalGenerators'], index)} />
      </div>)}
      <ListControls label={t('opname.addGenerator')}
        onAdd={() => change(['hotWater', 'additionalGenerators'], [...hotWaterExtras, { id: freshItemId(hotWaterExtras, 'tapwateropwekker'), generator: hotWaterGeneratorTemplate('electric_instantaneous'), nominalPowerKw: 10 }])} />
      </>}
      {hotWaterRest && solar.map((_, index) => {
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
          <SurveyPhotos path={base} />
          <RemoveButton label={t('opname.remove')} onRemove={() => removeAt(['hotWater', 'solar'], index)} />
        </div>;
      })}
      {hotWaterRest && <ListControls label={t('opname.addSolar')} onAdd={() => change(['hotWater', 'solar'], [...solar, solarTemplate(solar.length)])} />}
      {hotWaterRest && kind === 'utility' && hotWaterSystems.map((_, index) => {
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
          <SurveyPhotos path={base} />
          <RemoveButton label={t('opname.remove')}
            onRemove={() => removeAt(['additionalHotWaterSystems'], index)} />
        </div>;
      })}
      {hotWaterRest && kind === 'utility' && <ListControls label={t('opname.hotWater.addSystem')}
        onAdd={() => change(['additionalHotWaterSystems'], [...hotWaterSystems, {
          id: freshItemId(hotWaterSystems, 'tapwatersysteem'),
          generator: hotWaterGeneratorTemplate('electric_instantaneous'), showerHeatRecovery: 'none',
          servedAreas: [{ function: String(read(draft, ['functions', 0, 'function']) ?? 'office'), areaM2: 0 }], sourceReference: '',
        }])} />}
    </Section></>}

    {show('ventilation') && <>{kind === 'utility' && ahu != null && <Section title={t('opname.ahu')}>
      <CheckField {...field} path={['ventilation', 'ahu', 'heatingConnected']} label={t('opname.ahu.heatingConnected')} />
      <CheckField {...field} path={['ventilation', 'ahu', 'coolingConnected']} label={t('opname.ahu.coolingConnected')} />
    </Section>}

    {kind === 'residential' && <Section title={t('opname.ventilation')}>
      <ResidentialVentilationBasics draft={draft} change={change} t={t} hidePrinciple={part === 'ventilation'} />
      <VentilationSurveyFields draft={draft} change={change} t={t} />
      <SurveyPhotos path={['ventilation']} />
    </Section>}

    {kind === 'utility' && <Section title={t('opname.ventilation')}>
      <UtilityVentilationFields draft={draft} change={change} t={t} />
      <SurveyPhotos path={['ventilation']} />
    </Section>}

    <Section title={t('opname.passiveCooling')}>
      <PassiveCoolingFields draft={draft} change={change} t={t} />
    </Section></>}

    {show('cooling') && <><Section title={t('opname.cooling')}>
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
        <SurveyPhotos path={['cooling']} />
      </>}
    </Section></>}

    {show('pv') && <><Section title={t('opname.pv')}>
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
          <NumberField {...field} path={[...base, 'installationYear']} label={t('survey.pv.installationYear')} step="1" optional />
          <SelectField {...field} path={[...base, 'mounting']} label={t('survey.pv.mounting')}
            options={opts(t, 'survey.pv.mountingKind', ['not_ventilated', 'moderately_ventilated', 'strongly_ventilated', 'unknown'])}
            onChange={(path, value) => change(path, value ?? 'unknown')} />
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
          <SurveyPhotos path={base} />
          <RemoveButton label={t('opname.remove')} onRemove={() => removeAt(['pv'], index)} />
        </div>;
      })}
      <ListControls label={t('opname.addPv')} onAdd={() => change(['pv'], [...pv, pvTemplate(pv.length)])} />
    </Section></>}

    </div>
    </FieldPathPrefixProvider>

    {wizard && section !== 'result' && <div className="opname-wizard-nav">
      <button type="button" className="btn" disabled={position <= 0}
        onClick={() => onSection?.(sections[position - 1])}>{t('opname.wizard.previous')}</button>
      <button type="button" className="btn btn-primary" disabled={position < 0 || position >= sections.length - 1}
        onClick={() => onSection?.(sections[position + 1])}>{t('opname.wizard.next')}</button>
    </div>}

    {show('result') && <details className="opname-json">
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
    </details>}

    {!wizard && !embedded && <div className="opname-actions">
      <button type="button" className="btn btn-primary" disabled={busy} onClick={() => { void run(); }}>{t('opname.calculate')}</button>
      <button type="button" className="btn" onClick={() => save(undefined)}>{t('opname.discard')}</button>
    </div>}
    {error && <p className="opname-error" role="alert">{error}</p>}
    {wizard && section === 'result' && !result && <p className="nta-form-note">{t('opname.progress.notCalculated')}</p>}

    {result && show('result') && <div className="opname-result" aria-label={t('opname.result')}>
      <h3 className="opname-result-heading">{t('opname.resultHeading')}</h3>
      <p className="nta-form-note">{t('opname.resultNote')}</p>
      <p><strong>{t('opname.status')}:</strong> {t(`opname.statusValue.${result.status}`, { defaultValue: result.status })}</p>
      <p className="opname-edition"><strong>{t('opname.edition')}:</strong> {t(`nta.edition.${result.normVersion ?? DEFAULT_NORM_VERSION}`)}</p>
      {performance && <ul className="opname-indicators">
        <li>{t('opname.label')}: <strong>{performance.indicativeLabelClass ?? '—'}</strong></li>
        <li>BENG 1: {formatNumber(performance.needIndicatorKwhPerM2Year, locale, 2)} kWh/m²</li>
        <li>BENG 2: {formatNumber(performance.primaryFossilIndicatorKwhPerM2Year, locale, 2)} kWh/m²</li>
        <li>BENG 3: {formatNumber(performance.renewableSharePercent, locale, 1)} %</li>
      </ul>}
      <SurveyTakeoverAction result={result} />
      {result.issues.length > 0 && <ul className="opname-issues">
        {result.issues.map((item, index) => {
          const hint = t(`opname.issueHint.${item.code}`, { defaultValue: '' });
          return <li key={index}><KernelCode code={item.code} prefixes={['opname.issue.', 'nta.gap.', 'kernel.issue.']} />
            {' '}<code>{item.path}</code>{hint && <small> {hint}</small>}
            {wizard && surveySectionForPath(item.path) !== 'result' && <>{' '}
              <button type="button" className="btn btn-sm opname-goto" onClick={() => goTo(item.path)}>
                {t('opname.goTo', { section: t(`opname.section.${surveySectionForPath(item.path)}`) })}</button></>}</li>;
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
            <td>{defaultPathLabel(t, item.path)} <small><code>{item.path}</code></small></td>
            <td>{defaultValueLabel(t, item.value, locale)}</td>
            <td>{locale.toLowerCase().startsWith('nl') ? dutchSource(item.source) : item.source}</td>
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
    </div>

    {wizard && <aside className="opname-aside" aria-label={t('opname.resultCard')}>
      <div className="opname-aside-head">
        <strong>{t('opname.resultCard')}</strong>
        <Pill tone="unv">{t('opname.resultCard.indicative')}</Pill>
      </div>
      {result ? <>
        <div className="opname-aside-label">
          <span className="opname-aside-class" style={{ background: performance?.indicativeLabelClass ? labelColor(performance.indicativeLabelClass) : undefined }}>
            {performance?.indicativeLabelClass ?? '—'}</span>
          <span><strong>{formatNumber(performance?.primaryFossilIndicatorKwhPerM2Year, locale, 1)}</strong> {t('unit.kwhPerM2Year')} EP₂
            <small>{t('opname.resultCard.separate')}</small></span>
        </div>
        <p className="opname-aside-status">{t(`opname.statusValue.${result.status}`, { defaultValue: result.status })}</p>
        <p className="opname-aside-edition">{t('opname.edition')}: {t(`nta.edition.${result.normVersion ?? DEFAULT_NORM_VERSION}`)}</p>
        {result.registrationEligible === false && <Pill tone="warn">{t('nta.edition.legacyTitle')}</Pill>}
        {result.issues.length > 0 && <p className="opname-aside-issues" role="status">
          {t('opname.resultCard.issues', { count: result.issues.length })}</p>}
        {result.appliedDefaults.length > 0 && <p className="opname-aside-defaults">
          {t('opname.resultCard.defaults', { count: result.appliedDefaults.length })}</p>}
        <button type="button" className="btn btn-sm" onClick={() => onSection?.('result')}>{t('opname.resultCard.details')}</button>
        {/* The card's primary action (ontwerp §587–594); on the outcome page it sits with the details. */}
        {section !== 'result' && <SurveyTakeoverAction result={result} />}
      </> : <p className="nta-form-note">{t('opname.progress.notCalculated')}</p>}
      <div className="opname-actions">
        <button type="button" className="btn btn-primary" disabled={busy} onClick={() => { void run(); }}>{t('opname.calculate')}</button>
        <button type="button" className="btn" onClick={() => save(undefined)}>{t('opname.discard')}</button>
      </div>
    </aside>}
  </section>;
}
