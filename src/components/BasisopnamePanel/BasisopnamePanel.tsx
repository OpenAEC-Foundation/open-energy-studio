import { useRef, useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import {
  CheckField, NumberField, read, Section, SelectField, TextField, write, type Draft, type Path,
} from '../NtaPerformancePanel/NtaFormFields';
import {
  assessResidentialSurveyWithRust, assessUtilitySurveyWithRust, type OpnameAssessment,
} from '../../core/nta/KernelClient';
import {
  asResidential, asUtility, heatingGeneratorTemplate, hotWaterGeneratorTemplate, pvTemplate,
  solarTemplate, surveyTemplate, windowTemplate, type StoredSurvey, type SurveyKind,
} from '../../core/nta/SurveyTemplates';
import '../NtaPerformancePanel/NtaPerformancePanel.css';
import './BasisopnamePanel.css';

// ISSO 82.1 (dwellings) and 75.1 (utility) basisopname: structured fields for
// the main survey sections, a JSON view for the rest, and the kernel route
// that derives the NTA 8800 input with the applied defaults (ISSO pages).

type Change = (path: Path, value: unknown) => void;
type T = (key: string) => string;

const HEATING_KINDS = ['boiler', 'heat_pump', 'district_heat', 'electric', 'biomass', 'chp', 'none_present'];
const HOT_WATER_KINDS: Record<SurveyKind, string[]> = {
  residential: ['gas_appliance', 'electric_boiler', 'electric_instantaneous', 'heat_pump', 'district_heat',
    'collective_unknown', 'delivery_set_from_heating', 'none'],
  utility: ['gas_appliance', 'gas_storage_heater', 'electric_boiler', 'electric_instantaneous', 'heat_pump',
    'district_heat', 'collective_unknown', 'none'],
};
const COOLING_GENERATORS = ['room_air_conditioner', 'compression', 'gas_absorption', 'external_cold', 'unknown_collective',
  'aquifer_before2013', 'aquifer_from2013', 'aquifer_year_unknown', 'surface_water', 'closed_ground_loop', 'dew_point_cooling'];
const COOLING_EMITTERS = ['split_indoor_units_on_wall', 'split_indoor_units_on_ceiling', 'fan_coil_on_outer_wall',
  'fan_coil_on_ceiling', 'floor_cooling', 'concrete_core_activation', 'wall_cooling', 'ceiling_cooling', 'other'];
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

function HeatingGeneratorFields({ draft, path, change, t }: { draft: Draft; path: Path; change: Change; t: T }) {
  const field = { draft, onChange: change };
  const kind = read(draft, [...path, 'kind']);
  return <>
    <KindSelect draft={draft} path={path} label={t('opname.heating.generator')} kinds={HEATING_KINDS}
      prefix="opname.heating.kind" template={heatingGeneratorTemplate} change={change} t={t} />
    {kind === 'boiler' && <>
      <SelectField {...field} path={[...path, 'boilerType']} label={t('opname.heating.boilerType')}
        options={opts(t, 'opname.heating.boiler', ['conventional', 'vr', 'hr100', 'hr104', 'hr107', 'hydrogen'])} />
      <CheckField {...field} path={[...path, 'insideThermalBoundary']} label={t('opname.insideThermalBoundary')} />
      <NumberField {...field} path={[...path, 'manufactureYear']} label={t('opname.manufactureYear')} step="1" />
    </>}
    {kind === 'heat_pump' && <>
      <SelectField {...field} path={[...path, 'source']} label={t('opname.heating.source')}
        options={opts(t, 'opname.heating.hpSource', ['outdoor_air', 'exhaust_air', 'outdoor_and_exhaust_air', 'ground',
          'groundwater', 'surface_water', 'water_based_unknown'])} />
      <NumberField {...field} path={[...path, 'capacityKw']} label={t('opname.capacityKw')} />
      <CheckField {...field} path={[...path, 'highTemperature']} label={t('opname.heating.highTemperature')} />
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
  const { t } = useI18n();
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
  const heatingExtras = list(draft, ['heating', 'additionalGenerators']);
  const hotWaterExtras = list(draft, ['hotWater', 'additionalGenerators']);
  const solar = list(draft, ['hotWater', 'solar']);
  const pv = list(draft, ['pv']);
  const reasons = (read(draft, ['inklapRedenen']) ?? {}) as Record<string, string>;
  const verticalPipes = read(draft, ['verticalPipes']);

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
      {kind === 'utility' && <NumberField {...field} path={['toiletStacks']} label={t('opname.toiletStacks')} step="1" />}
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

    <Section title={t('opname.envelope')}>
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
              {['outdoor', 'ground', 'crawlspace', 'adjacent_heated', 'unheated_cellar', 'strongly_ventilated'].map((key) =>
                <option key={key} value={key}>{t(`opname.surface.boundaryKind.${key}`)}</option>)}
            </select>
          </label>
          <NumberField {...field} path={[...base, 'grossAreaM2']} label={t('opname.surface.area')} />
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
          <RemoveButton label={t('opname.remove')} onRemove={() => change(['envelope', 'windows'], windows.filter((_, item) => item !== index))} />
        </div>;
      })}
      <ListControls label={t('opname.addWindow')}
        onAdd={() => change(['envelope', 'windows'], [...windows, windowTemplate(windows.length, String(surfaces[0]?.id ?? ''))])} />
    </Section>

    <Section title={t('opname.heating')}>
      <HeatingGeneratorFields draft={draft} path={['heating', 'generator']} change={change} t={t} />
      <NumberField {...field} path={['heating', 'nominalPowerKw']} label={t('opname.nominalPowerKw')} />
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
        <CheckField {...field} path={['cooling', 'waterBased']} label={t('opname.cooling.waterBased')} />
        <SelectField {...field} path={['cooling', 'balanced']} label={t('opname.cooling.balanced')}
          options={[['none', t('opname.cooling.balanced.none')], ['static', t('opname.cooling.balanced.static')],
            ['dynamic', t('opname.cooling.balanced.dynamic')]]} />
        <CheckField {...field} path={['cooling', 'heatPumpSource']} label={t('opname.cooling.heatPumpSource')} />
        <CheckField {...field} path={['cooling', 'groundAboveZeroDemonstrated']} label={t('opname.cooling.groundAboveZero')} />
        {kind === 'residential' &&
          <CheckField {...field} path={['coolingCollective']} label={t('opname.cooling.collective')} />}
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
      <p><strong>{t('opname.status')}:</strong> {result.status}</p>
      {performance && <ul className="opname-indicators">
        <li>{t('opname.label')}: <strong>{performance.indicativeLabelClass ?? '—'}</strong></li>
        <li>BENG 1: {performance.needIndicatorKwhPerM2Year ?? '—'} kWh/m²</li>
        <li>BENG 2: {performance.primaryFossilIndicatorKwhPerM2Year ?? '—'} kWh/m²</li>
        <li>BENG 3: {performance.renewableSharePercent ?? '—'} %</li>
      </ul>}
      {result.issues.length > 0 && <ul className="opname-issues">
        {result.issues.map((item, index) => <li key={index}><code>{item.code}</code> {item.path}</li>)}
      </ul>}
      {result.warnings.length > 0 && <ul className="opname-warnings">
        {result.warnings.map((item, index) => <li key={index}><code>{item.code}</code> {item.note}</li>)}
      </ul>}
      {result.appliedDefaults.length > 0 && <table className="opname-defaults">
        <caption>{t('opname.defaults')}</caption>
        <thead><tr><th>{t('opname.defaults.path')}</th><th>{t('opname.defaults.value')}</th>
          <th>{t('opname.defaults.source')}</th><th>{t('opname.defaults.reason')}</th></tr></thead>
        <tbody>
          {result.appliedDefaults.map((item, index) => <tr key={index}>
            <td><code>{item.path}</code></td><td>{item.value}</td><td>{item.source}</td>
            <td><input aria-label={`${t('opname.defaults.reason')} ${item.path}`} value={reasons[item.path] ?? ''}
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
