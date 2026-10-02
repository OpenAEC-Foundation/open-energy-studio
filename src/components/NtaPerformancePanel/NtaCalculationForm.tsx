import { useState, type ReactNode } from 'react';
import type { IProject } from '../../core/energy/types';
import { useI18n } from '../../i18n/i18n';

// The block is edited as plain JSON data; the Rust kernel is the validator.

/** ρ_a·c_a/3600 with 1,205 kg/m³ and 1 005 J/(kg·K) (9.29), in W per (m³/h)·K. */
const AIR_HEAT_CAPACITY_W_PER_M3H_K = (1.205 * 1005) / 3600;
type Draft = Record<string, unknown>;
type Path = Array<string | number>;

function read(source: unknown, path: Path): unknown {
  return path.reduce<unknown>((value, key) => (value == null ? undefined : (value as Record<string | number, unknown>)[key]), source);
}

function write(source: Draft, path: Path, value: unknown): Draft {
  const clone = structuredClone(source) as Record<string | number, unknown>;
  let cursor = clone;
  path.slice(0, -1).forEach((key, index) => {
    const next = cursor[key];
    if (next == null || typeof next !== 'object') {
      cursor[key] = typeof path[index + 1] === 'number' ? [] : {};
    }
    cursor = cursor[key] as Record<string | number, unknown>;
  });
  cursor[path[path.length - 1]] = value;
  return clone as Draft;
}

interface FieldProps {
  draft: Draft;
  path: Path;
  label: string;
  onChange: (path: Path, value: unknown) => void;
}

function NumberField({ draft, path, label, onChange, step = 'any' }: FieldProps & { step?: string }) {
  const value = read(draft, path);
  return <label>{label}
    <input type="number" step={step} value={typeof value === 'number' ? value : ''}
      onChange={(event) => onChange(path, event.target.value === '' ? null : Number(event.target.value))} />
  </label>;
}

function TextField({ draft, path, label, onChange }: FieldProps) {
  const value = read(draft, path);
  return <label>{label}
    <input value={typeof value === 'string' ? value : ''} onChange={(event) => onChange(path, event.target.value)} />
  </label>;
}

function SelectField({ draft, path, label, onChange, options }: FieldProps & { options: Array<[string, string]> }) {
  const value = read(draft, path);
  return <label>{label}
    <select value={value == null ? '' : String(value)} onChange={(event) => onChange(path, event.target.value || null)}>
      <option value="">—</option>
      {options.map(([key, text]) => <option key={key} value={key}>{text}</option>)}
    </select>
  </label>;
}

function CheckField({ draft, path, label, onChange }: FieldProps) {
  return <label className="nta-form-check">
    <input type="checkbox" checked={read(draft, path) === true} onChange={(event) => onChange(path, event.target.checked)} />
    {label}
  </label>;
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return <fieldset className="nta-form-section"><legend>{title}</legend><div className="nta-form-grid">{children}</div></fieldset>;
}

export function NtaCalculationForm({ project, initial, onSave, onCancel }: {
  project: IProject;
  initial: Draft;
  onSave: (block: Draft) => void;
  onCancel: () => void;
}) {
  const { t } = useI18n();
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  const field = { draft, onChange: change };
  const residential = read(draft, ['calculationScope']) === 'residential';
  const surfaces = project.zones.flatMap((zone) => zone.surfaces);
  const tilts = (read(draft, ['surfaceTilts']) as Draft[] | undefined) ?? [];
  const ground = (read(draft, ['groundFloors']) as Draft[] | undefined) ?? [];
  const pv = (read(draft, ['pvSystems']) as Draft[] | undefined) ?? [];
  const generatorKind = read(draft, ['generator', 'kind']);
  const months = (read(draft, ['ventilationFlows', 0, 'months']) as Draft[] | undefined) ?? [];
  const constantConductance = months.length === 12 && months.every((month) => month.conductanceWPerK === months[0].conductanceWPerK)
    ? months[0].conductanceWPerK : null;
  const surfaceName = (id: unknown) => surfaces.find((surface) => surface.id === id)?.name || String(id);
  const setConstantVentilation = (value: number | null) => change(['ventilationFlows', 0, 'months'],
    Array.from({ length: 12 }, (_, index) => ({ ...(months[index] ?? {}), month: index + 1, conductanceWPerK: value })));

  return <form className="nta-form" aria-label={t('nta.form.title')} onSubmit={(event) => { event.preventDefault(); onSave(draft); }}>
    <p>{t('nta.form.help')}</p>
    <Section title={t('nta.form.general')}>
      <SelectField {...field} path={['calculationScope']} label={t('nta.form.scope')}
        options={[['residential', t('nta.form.scope.residential')], ['utility', t('nta.form.scope.utility')]]} />
      <TextField {...field} path={['areaSourceReference']} label={t('nta.form.areaSource')} />
      <SelectField {...field} path={['bblFunction']} label={t('nta.form.bblFunction')} options={[
        ['other_residential', t('nta.form.bbl.other_residential')], ['residential_building', t('nta.form.bbl.residential_building')],
        ['office', t('nta.form.bbl.office')], ['education', t('nta.form.bbl.education')], ['retail', t('nta.form.bbl.retail')],
        ['other_assembly', t('nta.form.bbl.other_assembly')], ['other_healthcare', t('nta.form.bbl.other_healthcare')],
        ['sport', t('nta.form.bbl.sport')], ['other_lodging', t('nta.form.bbl.other_lodging')],
      ]} />
      <SelectField {...field} path={['activeCooling', 'system']} label={t('nta.form.activeCooling')} options={[
        ['compression_table10_29', t('nta.form.ac.compression')], ['absorption_table10_30', t('nta.form.ac.absorption')],
        ['free_cooling_table10_34', t('nta.form.ac.free')], ['dew_point_cooling_humidified_exhaust', t('nta.form.ac.dewPoint')],
        ['heat_pump_with_cooling_emitter', t('nta.form.ac.heatPump')], ['external_cold_with_cooling_emitter', t('nta.form.ac.external')],
        ['split_units_in_every_habitable_room', t('nta.form.ac.split')], ['other_utility', t('nta.form.ac.other')]]}
        onChange={(path, value) => value == null
          ? change(['activeCooling'], null)
          : read(draft, ['activeCooling']) == null
            ? change(['activeCooling'], { system: value, capacity: { method: 'annex_aa', sourceReference: '' }, sourceReference: '' })
            : change(path, value)} />
      {read(draft, ['activeCooling']) != null && <>
        <SelectField {...field} path={['activeCooling', 'capacity', 'method']} label={t('nta.form.ac.capacity')} options={[
          ['dynamic_cooling_load', t('nta.form.ac.dynamic')], ['annex_aa', t('nta.form.ac.annexAa')], ['solar_limitation', t('nta.form.ac.solar')]]} />
        {read(draft, ['activeCooling', 'capacity', 'method']) === 'solar_limitation' &&
          <SelectField {...field} path={['activeCooling', 'capacity', 'criterion']} label={t('nta.form.ac.criterion')} options={[
            ['small_window_area', t('nta.form.ac.smallWindows')], ['shaded_glazing', t('nta.form.ac.shaded')]]} />}
        <TextField {...field} path={['activeCooling', 'capacity', 'sourceReference']} label={t('nta.form.source')} />
        <TextField {...field} path={['activeCooling', 'sourceReference']} label={t('nta.form.source')} />
      </>}
      <CheckField {...field} path={['permitApplicationAfter20260529']} label={t('nta.form.permitAfter')} />
    </Section>
    <Section title={t('nta.form.setpoints')}>
      <NumberField {...field} path={['setpoints', 'heatingC']} label={t('nta.form.heatingSetpoint')} />
      <NumberField {...field} path={['setpoints', 'coolingC']} label={t('nta.form.coolingSetpoint')} />
      <TextField {...field} path={['setpoints', 'sourceReference']} label={t('nta.form.source')} />
    </Section>
    <Section title={t('nta.form.mass')}>
      {(['floor', 'wall'] as const).map((part) => <SelectField key={part} {...field} path={['thermalMass', part]}
        label={t(`nta.form.mass.${part}`)} options={[['light', t('nta.form.mass.light')], ['heavy', t('nta.form.mass.heavy')], ['very_heavy', t('nta.form.mass.veryHeavy')]]} />)}
      <SelectField {...field} path={['thermalMass', 'ceiling']} label={t('nta.form.mass.ceiling')}
        options={[['open_or_none', t('nta.form.mass.open')], ['closed_or_suspended', t('nta.form.mass.closed')]]} />
      <TextField {...field} path={['thermalMass', 'sourceReference']} label={t('nta.form.source')} />
    </Section>
    <Section title={t('nta.form.internalGains')}>
      {residential
        ? <NumberField {...field} path={['internalGains', 'dwellingCount']} label={t('nta.form.dwellingCount')} step="1" />
        : <NumberField {...field} path={['internalGains', 'heatFluxWPerM2']} label={t('nta.form.heatFlux')} />}
      <TextField {...field} path={['internalGains', 'sourceReference']} label={t('nta.form.source')} />
    </Section>
    <Section title={t('nta.form.windows')}>
      <NumberField {...field} path={['windowSolar', 'frameFraction']} label={t('nta.form.frameFraction')} />
      <label>{t('nta.form.obstruction')}
        <select value={String(read(draft, ['windowSolar', 'obstruction', 'method']) ?? '')} onChange={(event) => change(
          ['windowSolar', 'obstruction'], event.target.value === 'declared'
            ? { method: 'declared', heating: Array(12).fill(null), cooling: Array(12).fill(null), sourceReference: '' }
            : { method: 'minimal' })}>
          <option value="minimal">{t('nta.form.obstruction.minimal')}</option>
          <option value="declared">{t('nta.form.obstruction.declared')}</option>
        </select>
      </label>
      <label>{t('nta.form.shading')}
        <select value={String(read(draft, ['windowSolar', 'movableShading', 'control']) ?? '')} onChange={(event) => change(
          ['windowSolar', 'movableShading'], event.target.value
            ? { reductionFactor: read(draft, ['windowSolar', 'movableShading', 'reductionFactor']) ?? null,
              control: event.target.value, sourceReference: read(draft, ['windowSolar', 'movableShading', 'sourceReference']) ?? '' }
            : null)}>
          <option value="">{t('nta.form.shading.none')}</option>
          <option value="manual_residential">{t('nta.form.shading.manual')}</option>
          <option value="automatic">{t('nta.form.shading.automatic')}</option>
        </select>
      </label>
      {read(draft, ['windowSolar', 'movableShading']) != null && <>
        <NumberField {...field} path={['windowSolar', 'movableShading', 'reductionFactor']} label={t('nta.form.shadingFc')} />
        <TextField {...field} path={['windowSolar', 'movableShading', 'sourceReference']} label={t('nta.form.source')} />
      </>}
      <TextField {...field} path={['windowSolar', 'sourceReference']} label={t('nta.form.source')} />
    </Section>
    {tilts.length > 0 && <Section title={t('nta.form.roofTilts')}>
      {tilts.map((item, index) => <div key={String(item.surfaceId)} className="nta-form-row">
        <NumberField {...field} path={['surfaceTilts', index, 'tiltDeg']} label={`${surfaceName(item.surfaceId)} — ${t('nta.form.tilt')}`} />
        <TextField {...field} path={['surfaceTilts', index, 'sourceReference']} label={t('nta.form.source')} />
      </div>)}
    </Section>}
    {ground.length > 0 && <Section title={t('nta.form.groundFloors')}>
      {ground.map((item, index) => <div key={String(item.surfaceId)} className="nta-form-row">
        <NumberField {...field} path={['groundFloors', index, 'exposedPerimeterM']} label={`${surfaceName(item.surfaceId)} — ${t('nta.form.perimeter')}`} />
        <NumberField {...field} path={['groundFloors', index, 'constructionResistanceM2kPerW']} label={t('nta.form.floorResistance')} />
        <TextField {...field} path={['groundFloors', index, 'sourceReference']} label={t('nta.form.source')} />
      </div>)}
    </Section>}
    <Section title={t('nta.form.ventilation')}>
      <label>{t('nta.form.ventilationConstant')}
        <input type="number" step="any" value={typeof constantConductance === 'number' ? constantConductance : ''}
          onChange={(event) => setConstantVentilation(event.target.value === '' ? null : Number(event.target.value))} />
      </label>
      <label>{t('nta.form.ventilationFlow')}
        <input type="number" step="any" value={typeof constantConductance === 'number'
          ? Math.round((constantConductance / AIR_HEAT_CAPACITY_W_PER_M3H_K) * 100) / 100 : ''}
          onChange={(event) => setConstantVentilation(event.target.value === ''
            ? null : Number(event.target.value) * AIR_HEAT_CAPACITY_W_PER_M3H_K)} />
      </label>
      <TextField {...field} path={['ventilationFlows', 0, 'sourceReference']} label={t('nta.form.source')} />
      <p className="nta-form-note">{t('nta.form.ventilationNote')}</p>
    </Section>
    <Section title={t('nta.form.emission')}>
      <SelectField {...field} path={['emission', 'system']} label={t('nta.form.emissionSystem')} options={[
        ['radiators_or_convectors', t('nta.form.emission.radiators')], ['floor_heating', t('nta.form.emission.floor')],
        ['air_heating', t('nta.form.emission.air')], ['local_heater', t('nta.form.emission.local')], ['other_or_unknown', t('nta.form.unknown')]]} />
      <SelectField {...field} path={['emission', 'balancing']} label={t('nta.form.balancing')} options={[
        ['none_or_unknown', t('nta.form.unknown')], ['static', t('nta.form.balancing.static')],
        ['dynamic', t('nta.form.balancing.dynamic')], ['not_applicable', t('nta.form.balancing.na')]]} />
      <SelectField {...field} path={['emission', 'control']} label={t('nta.form.control')} options={[
        ['main_room_thermostat', t('nta.form.control.main')], ['central_with_room_valves', t('nta.form.control.central')],
        ['individual_room_thermostats', t('nta.form.control.individual')], ['other_or_unknown', t('nta.form.unknown')]]} />
      <TextField {...field} path={['emission', 'sourceReference']} label={t('nta.form.source')} />
      <TextField {...field} path={['distribution', 'sourceReference']} label={t('nta.form.distributionSource')} />
    </Section>
    <Section title={t('nta.form.generator')}>
      <label>{t('nta.form.generatorKind')}
        <select value={typeof generatorKind === 'string' ? generatorKind : ''} onChange={(event) => {
          const kind = event.target.value;
          if (kind === 'external_heat') change(['generator'], { kind, supplierReference: '', qualityDeclarationPresent: false });
          if (kind === 'gas_boiler') change(['generator'], { kind, boiler: {
            generatorId: 'boiler', role: 'individual_main', location: null, kind: null, fuel: 'natural_gas',
            averageDesignEmissionTemperatureC: null, emissionCircuit: null, equipmentReference: '',
            locationReference: '', temperatureAndCircuitReference: '', pilotFlamePresent: false } });
          if (kind === 'electric_resistance') change(['generator'], { kind, equipmentReference: '' });
          if (kind === 'biomass') change(['generator'], { kind, appliance: null, location: 'inside_thermal_boundary',
            annexRCompliantAtMost500Kw: false, annexRReference: '', equipmentReference: '' });
          if (kind === 'heat_pump_forfait') change(['generator'], {
            kind, forfait: project.heatingSystems[0]?.ntaHeatPump?.forfaitHeatPumpDraft ?? null,
            sourceSystem: 'individual', sourceSystemReference: '' });
        }}>
          <option value="gas_boiler">{t('nta.form.generator.boiler')}</option>
          <option value="external_heat">{t('nta.form.generator.external')}</option>
          <option value="heat_pump_forfait">{t('nta.form.generator.heatPump')}</option>
          <option value="electric_resistance">{t('nta.form.generator.electric')}</option>
          <option value="biomass">{t('nta.form.generator.biomass')}</option>
          {generatorKind === 'hybrid_heat_pump' && <option value="hybrid_heat_pump">{t('nta.form.generator.hybrid')}</option>}
        </select>
      </label>
      {generatorKind === 'external_heat' && <>
        <TextField {...field} path={['generator', 'supplierReference']} label={t('nta.form.externalSource')} />
        <p className="nta-form-note">{t('nta.form.externalNote')}</p>
      </>}
      {generatorKind === 'electric_resistance' && <TextField {...field} path={['generator', 'equipmentReference']} label={t('nta.form.source')} />}
      {generatorKind === 'biomass' && <>
        <SelectField {...field} path={['generator', 'appliance']} label={t('nta.form.biomassAppliance')} options={[
          ['freestanding_wood_stove', t('nta.form.biomass.stove')], ['insert_stove', t('nta.form.biomass.insert')],
          ['pellet_stove', t('nta.form.biomass.pellet')], ['accumulating_stove', t('nta.form.biomass.accumulating')],
          ['central_boiler', t('nta.form.biomass.boiler')]]} />
        <SelectField {...field} path={['generator', 'location']} label={t('nta.form.boilerLocation')} options={[
          ['inside_thermal_boundary', t('nta.form.boiler.inside')], ['outside_thermal_boundary', t('nta.form.boiler.outside')]]} />
        <CheckField {...field} path={['generator', 'annexRCompliantAtMost500Kw']} label={t('nta.form.biomassAnnexR')} />
        <TextField {...field} path={['generator', 'annexRReference']} label={t('nta.form.biomassAnnexRSource')} />
        <TextField {...field} path={['generator', 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
      </>}
      {generatorKind === 'gas_boiler' ? <>
        <SelectField {...field} path={['generator', 'boiler', 'kind']} label={t('nta.form.boilerKind')} options={[
          ['hr107', 'HR107'], ['hr104', 'HR104'], ['hr100', 'HR100'], ['vr', 'VR'], ['conventional', t('nta.form.boiler.conventional')]]} />
        <SelectField {...field} path={['generator', 'boiler', 'location']} label={t('nta.form.boilerLocation')} options={[
          ['inside_thermal_boundary', t('nta.form.boiler.inside')], ['outside_thermal_boundary', t('nta.form.boiler.outside')]]} />
        <NumberField {...field} path={['generator', 'boiler', 'averageDesignEmissionTemperatureC']} label={t('nta.form.boilerTemperature')} />
        <SelectField {...field} path={['generator', 'boiler', 'emissionCircuit']} label={t('nta.form.boilerCircuit')} options={[
          ['direct', t('nta.form.boiler.direct')], ['mixing_with_return_limit', t('nta.form.boiler.mixingLimit')],
          ['mixing_without_return_limit', t('nta.form.boiler.mixingNoLimit')]]} />
        <TextField {...field} path={['generator', 'boiler', 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
        <TextField {...field} path={['generator', 'boiler', 'locationReference']} label={t('nta.form.boilerLocationSource')} />
        <TextField {...field} path={['generator', 'boiler', 'temperatureAndCircuitReference']} label={t('nta.form.boilerTemperatureSource')} />
      </> : ['external_heat', 'electric_resistance', 'biomass'].includes(String(generatorKind)) ? null : <p className="nta-form-note">{t('nta.form.heatPumpNote')}</p>}
    </Section>
    {read(draft, ['hotWater']) != null && <Section title={t('nta.form.hotWater')}>
      {residential
        ? <NumberField {...field} path={['hotWater', 'need', 'dwellingCount']} label={t('nta.form.dwellingCount')} step="1" />
        : <NumberField {...field} path={['hotWater', 'need', 'specificNeedKwhPerM2Year']} label={t('nta.form.hotWaterNeed')} />}
      <TextField {...field} path={['hotWater', 'need', 'sourceReference']} label={t('nta.form.source')} />
      <NumberField {...field} path={['hotWater', 'emissionEfficiency']} label={t('nta.form.hotWaterEmission')} />
      <NumberField {...field} path={['hotWater', 'distributionEfficiency']} label={t('nta.form.hotWaterDistribution')} />
      <NumberField {...field} path={['hotWater', 'generationEfficiency']} label={t('nta.form.hotWaterGeneration')} />
      <SelectField {...field} path={['hotWater', 'carrier']} label={t('nta.form.carrier')}
        options={[['gas', t('nta.performance.gas')], ['el', t('nta.performance.electricity')], ['oil', t('nta.form.oil')]]} />
      <CheckField {...field} path={['hotWater', 'renewableHeatPump']} label={t('nta.form.hotWaterHeatPump')} />
      <TextField {...field} path={['hotWater', 'efficiencySourceReference']} label={t('nta.form.source')} />
    </Section>}
    <Section title={t('nta.form.cooling')}>
      <label>{t('nta.form.coolingGenerator')}
        <select value={String(read(draft, ['cooling', 'generators', 0, 'generator', 'kind']) ?? '')} onChange={(event) => {
          const kind = event.target.value;
          if (!kind) { change(['cooling'], null); return; }
          const current = (read(draft, ['cooling']) as Draft | null) ?? {
            emission: { emitter: null, balancing: null, control: null, fanCoilCount: 0, sourceReference: '' },
            distribution: null, generators: [], boosterHeatPumpExtractionKwh: [] };
          const generator = kind === 'free_cooling'
            ? { kind, source: null, heatPumpSource: false, groundAboveZeroDemonstrated: false } : { kind };
          change(['cooling'], { ...current, generators: [{ id: 'cold-1', generator, capacityKw: null, equipmentReference: '' }] });
        }}>
          <option value="">{t('nta.form.cooling.none')}</option>
          <option value="compression">{t('nta.form.cooling.compression')}</option>
          <option value="room_air_conditioner">{t('nta.form.cooling.rac')}</option>
          <option value="gas_absorption">{t('nta.form.cooling.absorption')}</option>
          <option value="absorption_external_heat">{t('nta.form.cooling.absorptionDh')}</option>
          <option value="external_cold">{t('nta.form.cooling.external')}</option>
          <option value="unknown_collective">{t('nta.form.cooling.unknownCollective')}</option>
          <option value="free_cooling">{t('nta.form.cooling.free')}</option>
        </select>
      </label>
      {read(draft, ['cooling']) != null && <>
        <TextField {...field} path={['cooling', 'generators', 0, 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
        {read(draft, ['cooling', 'generators', 0, 'generator', 'kind']) === 'free_cooling' && <>
          <SelectField {...field} path={['cooling', 'generators', 0, 'generator', 'source']} label={t('nta.form.freeCoolingSource')} options={[
            ['aquifer_from2013', t('nta.form.free.aquiferNew')], ['aquifer_dwellings_before2013', t('nta.form.free.aquiferOld')],
            ['aquifer_utility_before2013', t('nta.form.free.aquiferUtility')], ['surface_water', t('nta.form.free.surface')],
            ['closed_ground_loop', t('nta.form.free.ground')], ['dew_point_cooling', t('nta.form.free.dewPoint')]]} />
          <CheckField {...field} path={['cooling', 'generators', 0, 'generator', 'heatPumpSource']} label={t('nta.form.free.heatPumpSource')} />
        </>}
        <SelectField {...field} path={['cooling', 'emission', 'emitter']} label={t('nta.form.coolingEmitter')} options={[
          ['floor_cooling', t('nta.form.coolEm.floor')], ['wall_cooling', t('nta.form.coolEm.wall')],
          ['fan_coil_or_rac_on_outer_wall', t('nta.form.coolEm.fanWall')], ['ceiling_cooling', t('nta.form.coolEm.ceiling')],
          ['fan_coil_or_rac_on_ceiling', t('nta.form.coolEm.fanCeiling')], ['other_or_unknown', t('nta.form.coolEm.other')]]} />
        <SelectField {...field} path={['cooling', 'emission', 'balancing']} label={t('nta.form.balancing')} options={[
          ['none_or_unknown', t('nta.form.coolBal.none')], ['static', t('nta.form.coolBal.static')],
          ['dynamic', t('nta.form.coolBal.dynamic')], ['not_applicable', t('nta.form.coolBal.na')]]} />
        <SelectField {...field} path={['cooling', 'emission', 'control']} label={t('nta.form.coolingControl')} options={[
          ['unknown_or_other', t('nta.form.coolCtrl.unknown')], ['standalone_per_room', t('nta.form.coolCtrl.standalone')],
          ['central_with_room_control', t('nta.form.coolCtrl.central')]]} />
        <NumberField {...field} path={['cooling', 'emission', 'fanCoilCount']} label={t('nta.form.coolingFanCoils')} />
        <TextField {...field} path={['cooling', 'emission', 'sourceReference']} label={t('nta.form.source')} />
        <label className="nta-form-check">
          <input type="checkbox" checked={read(draft, ['cooling', 'distribution']) != null}
            onChange={(event) => change(['cooling', 'distribution'], event.target.checked
              ? { designTemperature: 't6_to12_or_unknown', pipe: { kind: 'insulated_before1980_or_unknown_age' }, fittingsInsulated: false, sourceReference: '' }
              : null)} />
          {t('nta.form.coolingWaterBased')}
        </label>
        {read(draft, ['cooling', 'distribution']) != null && <>
          <SelectField {...field} path={['cooling', 'distribution', 'designTemperature']} label={t('nta.form.coolingDesign')} options={[
            ['t6_to12_or_unknown', '6/12'], ['t12_to16', '12/16'], ['t12_to18', '12/18'], ['t17_to21', '17/21']]} />
          <SelectField {...field} path={['cooling', 'distribution', 'pipe', 'kind']} label={t('nta.form.coolingPipe')} options={[
            ['insulated_from1995', t('nta.form.coolPipe.from1995')], ['insulated1980_to1995', t('nta.form.coolPipe.1980')],
            ['insulated_before1980_or_unknown_age', t('nta.form.coolPipe.before1980')], ['uninsulated', t('nta.form.coolPipe.none')]]} />
          <CheckField {...field} path={['cooling', 'distribution', 'fittingsInsulated']} label={t('nta.form.coolingFittings')} />
          <TextField {...field} path={['cooling', 'distribution', 'sourceReference']} label={t('nta.form.source')} />
        </>}
      </>}
    </Section>
    {pv.length > 0 && <Section title={t('nta.form.pv')}>
      {pv.map((item, index) => <div key={String(item.id)} className="nta-form-row">
        <SelectField {...field} path={['pvSystems', index, 'peakPower', 'method']} label={`${String(item.id)} — ${t('nta.form.pvPeak')}`}
          options={[['panels', t('nta.form.pvPeak.panels')], ['declared_specific', t('nta.form.pvPeak.declared')], ['table16_1', t('nta.form.pvPeak.table')]]} />
        {read(draft, ['pvSystems', index, 'peakPower', 'method']) === 'panels' ? <>
          <NumberField {...field} path={['pvSystems', index, 'peakPower', 'panelPeakPowerW']} label={t('nta.form.pvPanelPower')} />
          <NumberField {...field} path={['pvSystems', index, 'peakPower', 'panelCount']} label={t('nta.form.pvPanelCount')} />
        </> : <>
          {read(draft, ['pvSystems', index, 'peakPower', 'method']) === 'table16_1'
            ? <SelectField {...field} path={['pvSystems', index, 'peakPower', 'moduleType']} label={t('nta.form.pvModuleType')} options={[
              ['monocrystalline_from2018', 'mono ≥ 2018'], ['monocrystalline2015_to2017', 'mono 2015–2017'],
              ['monocrystalline2011_to2014', 'mono 2011–2014'], ['monocrystalline2001_to2010', 'mono 2001–2010'],
              ['monocrystalline_before2001', 'mono < 2001'], ['multicrystalline_from2018', 'multi ≥ 2018'],
              ['multicrystalline2015_to2017', 'multi 2015–2017'], ['multicrystalline2011_to2014', 'multi 2011–2014'],
              ['multicrystalline2001_to2010', 'multi 2001–2010'], ['multicrystalline_before2001', 'multi < 2001'],
              ['amorphous_single_junction', 'a-Si'], ['amorphous_multi_junction', 'a-Si multi'], ['cigs', 'CIGS'], ['cd_te', 'CdTe']]} />
            : <NumberField {...field} path={['pvSystems', index, 'peakPower', 'peakPowerWPerM2']} label={t('nta.form.pvSpecificPeak')} />}
          <NumberField {...field} path={['pvSystems', index, 'peakPower', 'panelAreaM2']} label={t('nta.form.pvArea')} />
        </>}
        <NumberField {...field} path={['pvSystems', index, 'azimuthDeg']} label={t('nta.form.pvAzimuth')} />
        <NumberField {...field} path={['pvSystems', index, 'tiltDeg']} label={t('nta.form.tilt')} />
        <SelectField {...field} path={['pvSystems', index, 'mounting']} label={t('nta.form.pvPerformance')}
          options={[['unknown', t('nta.form.pvMount.unknown')], ['not_ventilated', '0,76'], ['moderately_ventilated', '0,80'], ['strongly_ventilated', '0,82']]} />
        <NumberField {...field} path={['pvSystems', index, 'obstructionFactors', 0]} label={t('nta.form.obstruction')} />
        <TextField {...field} path={['pvSystems', index, 'sourceReference']} label={t('nta.form.source')} />
      </div>)}
    </Section>}
    <Section title={t('nta.form.inventory')}>
      <NumberField {...field} path={['bacsFactor']} label={t('nta.form.bacs')} />
      <TextField {...field} path={['bacsSourceReference']} label={t('nta.form.source')} />
      <CheckField {...field} path={['useInventoryComplete']} label={t('nta.form.useInventory')} />
      <CheckField {...field} path={['productionInventoryComplete']} label={t('nta.form.productionInventory')} />
      <CheckField {...field} path={['demandUsesFixedC1Ventilation']} label={t('nta.form.c1')} />
      <CheckField {...field} path={['batteryStoragePresent']} label={t('nta.form.battery')} />
    </Section>
    <div className="nta-form-actions">
      <button type="button" onClick={onCancel}>{t('dialog.cancel')}</button>
      <button type="submit">{t('dialog.save')}</button>
    </div>
  </form>;
}
