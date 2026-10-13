import {
  CheckField, NumberField, read, SelectField, TextField, TriStateField, type Draft, type Path,
} from '../NtaPerformancePanel/NtaFormFields';
import { freshItemId } from '../../core/nta/EvidenceLinks';

// Survey answers that were only reachable through the JSON view: the dwelling
// or building type, the ceiling, a measured qv10, the ventilation principle
// and unit, the heating control and the hot-water taps (ISSO 82.1 / 75.1).

type Change = (path: Path, value: unknown) => void;
type T = (key: string, options?: Record<string, unknown>) => string;
interface Props { draft: Draft; change: Change; t: T }

/** ISSO 82.1 table 7.8 / 75.1 table 7.9 glass kinds (`GlassAnswer` in the kernel). */
export const SURVEY_GLASS_KINDS = ['triple_hr', 'hr_plus_plus', 'hr_plus', 'hr', 'coated_type_unknown', 'double', 'double_with_coating',
  'double_with_secondary', 'hr_with_secondary', 'hr_plus_plus_with_secondary', 'secondary_window', 'single', 'leaded_light',
  'glass_blocks'];

/** Table 11.5 system variants (`SystemVariant` in the kernel, snake_case). */
export const SYSTEM_VARIANTS = ['a1', 'a2a', 'a2b', 'a2c', 'b1', 'b2', 'b3', 'c1', 'c2a', 'c2b', 'c2c', 'c3a', 'c3b', 'c3c',
  'c4a', 'c4b', 'c4c', 'c5a', 'c5b', 'd1', 'd2', 'd3', 'd4a', 'd4b', 'd5a', 'd5b', 'd5c'];

function opts(t: T, prefix: string, keys: string[]): Array<[string, string]> {
  return keys.map((key) => [key, t(`${prefix}.${key}`)]);
}

/** ISSO 82.1 §7.1 afb. 7.1: single-family dwelling (position, roof) or apartment (floor, side). */
export function DwellingTypeFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const kind = read(draft, ['dwelling', 'kind']);
  return <>
    <SelectField {...field} path={['dwelling', 'kind']} label={t('opname.dwelling.kind')}
      options={opts(t, 'opname.dwelling.kindKind', ['single_family', 'apartment'])}
      onChange={(_, value) => change(['dwelling'], value === 'apartment'
        ? { kind: 'apartment', floor: 'ground_or_intermediate', side: 'unknown' }
        : { kind: 'single_family', position: 'unknown', roofType: 'pitched' })} />
    {kind === 'single_family' && <>
      <SelectField {...field} path={['dwelling', 'position']} label={t('opname.dwelling.position')}
        options={opts(t, 'opname.dwelling.positionKind', ['terraced', 'end_or_corner', 'detached', 'unknown'])} />
      <SelectField {...field} path={['dwelling', 'roofType']} label={t('opname.dwelling.roofType')}
        options={opts(t, 'opname.dwelling.roofKind', ['pitched', 'partly_flat', 'flat'])} />
    </>}
    {kind === 'apartment' && <SelectField {...field} path={['dwelling', 'side']} label={t('opname.dwelling.side')}
      options={opts(t, 'opname.dwelling.sideKind', ['middle', 'end_or_corner', 'unknown'])} />}
  </>;
}

/** ISSO 75.1 §7.1: single-layer building (position, roof), whole multi-layer building or part of one. */
export function BuildingTypeFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const kind = read(draft, ['buildingType', 'kind']);
  return <>
    <SelectField {...field} path={['buildingType', 'kind']} label={t('opname.buildingType.kind')}
      options={opts(t, 'opname.buildingType.kindKind', ['single_layer', 'multi_layer_whole', 'multi_layer_part'])}
      onChange={(_, value) => change(['buildingType'], value === 'single_layer'
        ? { kind: 'single_layer', position: 'detached', roof: 'pitched' }
        : value === 'multi_layer_part' ? { kind: 'multi_layer_part', level: 'intermediate', position: 'middle' }
          : { kind: 'multi_layer_whole' })} />
    {kind === 'single_layer' && <>
      <SelectField {...field} path={['buildingType', 'position']} label={t('opname.dwelling.position')}
        options={opts(t, 'opname.dwelling.positionKind', ['detached', 'end_or_corner', 'terraced'])} />
      <SelectField {...field} path={['buildingType', 'roof']} label={t('opname.dwelling.roofType')}
        options={opts(t, 'opname.dwelling.roofKind', ['pitched', 'partly_flat', 'flat'])} />
    </>}
    {kind === 'multi_layer_part' && <>
      <SelectField {...field} path={['buildingType', 'level']} label={t('opname.buildingType.level')}
        options={opts(t, 'opname.buildingType.levelKind', ['bottom', 'intermediate', 'top'])} />
      <SelectField {...field} path={['buildingType', 'position']} label={t('opname.dwelling.position')}
        options={opts(t, 'opname.buildingType.partPositionKind', ['end_or_corner', 'middle', 'whole_storey'])} />
    </>}
  </>;
}

/** Table 7.4/7.5: lighter ceiling, and (utility only) a closed or suspended ceiling; a measured qv10 (NEN 2686). */
export function ConstructionExtrasFields({ draft, change, t, utility }: Props & { utility: boolean }) {
  const field = { draft, onChange: change };
  const measured = read(draft, ['measuredInfiltration']);
  return <>
    <SelectField {...field} path={['construction', 'floor']} label={t('opname.construction.floor')}
      options={opts(t, 'opname.construction.massKind', ['light', 'heavy', 'very_heavy'])} />
    <SelectField {...field} path={['construction', 'wall']} label={t('opname.construction.wall')}
      options={opts(t, 'opname.construction.massKind', ['light', 'heavy', 'very_heavy'])} />
    <CheckField {...field} path={['construction', 'lighterCeiling']} label={t('opname.construction.lighterCeiling')} />
    {utility && <CheckField {...field} path={['construction', 'closedOrSuspendedCeiling']}
      label={t('opname.construction.closedCeiling')} />}
    <label className="nta-form-check">
      <input type="checkbox" checked={measured != null} onChange={(event) => change(['measuredInfiltration'],
        event.target.checked ? { qv10Dm3PerSM2: null, sourceReference: '' } : null)} />
      {t('opname.measuredInfiltration')}
    </label>
    {measured != null && <>
      <NumberField {...field} path={['measuredInfiltration', 'qv10Dm3PerSM2']} label={t('opname.measuredInfiltration.qv10')}
        unit="dm³/(s·m²)" />
      <TextField {...field} path={['measuredInfiltration', 'sourceReference']} label={t('opname.measuredInfiltration.source')} />
    </>}
  </>;
}

/** ISSO 82.1/75.1 chapter 11: the ventilation principle and the unit the survey found. */
export function VentilationBasicsFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const principle = read(draft, ['ventilation', 'principle']);
  return <>
    <SelectField {...field} path={['ventilation', 'principle']} label={t('opname.ventilation.principle')}
      options={opts(t, 'opname.ventilation.principleKind', ['natural', 'mechanical_supply', 'mechanical_extract', 'balanced'])} />
    {(principle === 'natural' || principle === 'mechanical_extract') && <>
      <TriStateField {...field} {...yesNo} path={['ventilation', 'selfRegulatingVents']} label={t('opname.ventilation.selfRegulating')} />
      <SelectField {...field} path={['ventilation', 'pressureClass']} label={t('opname.ventilation.pressureClass')}
        options={opts(t, 'opname.ventilation.pressureClassKind', ['at_most1_pa', 'from1_to5_pa', 'from5_to10_pa'])} />
    </>}
    <NumberField {...field} path={['ventilation', 'installationYear']} label={t('opname.ventilation.installationYear')} optional step="1" />
    {principle !== 'natural' && <>
      <NumberField {...field} path={['ventilation', 'unitManufactureYear']} label={t('opname.ventilation.unitYear')} optional step="1" />
      <SelectField {...field} path={['ventilation', 'motor']} label={t('opname.ventilation.motor')}
        options={opts(t, 'opname.ventilation.motorKind', ['ac', 'dc', 'unknown'])} />
    </>}
    <SelectField {...field} path={['ventilation', 'declaredVariant']} label={t('opname.ventilation.declaredVariant')}
      options={SYSTEM_VARIANTS.map((code) => [code, code.toUpperCase()])}
      onChange={(path, value) => change(path, value == null || value === '' ? undefined : value)} />
  </>;
}

/** ISSO 82.1/75.1 §9: room control and the pipes in unheated spaces (afb. 9.1). */
export function HeatingControlFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const pipes = read(draft, ['heating', 'unheatedPipes', 'kind']);
  return <>
    <SelectField {...field} path={['heating', 'control']} label={t('opname.heating.control')}
      options={opts(t, 'opname.heating.controlKind', ['room_thermostat', 'central_with_radiator_valves', 'individual_room_control', 'unknown'])} />
    <label>{t('opname.heating.unheatedPipes')}
      <select value={typeof pipes === 'string' ? pipes : ''} onChange={(event) => change(['heating', 'unheatedPipes'],
        event.target.value === '' ? null : event.target.value === 'absent' ? { kind: 'absent' } : { kind: 'present', lengthM: null })}>
        <option value="">{t('opname.heating.unheatedPipes.default')}</option>
        <option value="absent">{t('opname.heating.unheatedPipes.absent')}</option>
        <option value="present">{t('opname.heating.unheatedPipes.present')}</option>
      </select>
    </label>
    {pipes === 'present' && <NumberField {...field} path={['heating', 'unheatedPipes', 'lengthM']}
      label={t('opname.heating.unheatedPipes.length')} optional unit="m" />}
  </>;
}

const LAMP_TYPES = ['t12', 't8_conventional', 't8_high_frequency', 't5', 'led_in_luminaire', 'cfl_plug_in', 'cfl_integrated',
  'incandescent_or_halogen', 'led_lamp', 'unknown'];
const PRESENCE = ['none_or_central_on', 'room_switch', 'room_switch_with_sweep', 'sensors_type_unknown', 'auto_on_dimmed',
  'auto_on_auto_off', 'manual_on_dimmed', 'manual_on_auto_off'];

function items(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

/** ISSO 75.1 chapter 14: lighting zones with installed power (luminaires or lamps), presence and daylight control. */
export function UtilityLightingFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const zones = items(draft, ['lighting']);
  return <>
    {zones.map((zone, index) => {
      const base: Path = ['lighting', index];
      const method = read(draft, [...base, 'power', 'method']);
      const parts = items(draft, [...base, 'power', method === 'lamps' ? 'lamps' : 'luminaires']);
      return <div key={index} className="opname-item" role="group" aria-label={String(zone.id ?? index + 1)}>
        <strong>{String(zone.id ?? index + 1)}</strong>
        <NumberField {...field} path={[...base, 'areaM2']} label={t('opname.lighting.area')} unit="m²" />
        <SelectField {...field} path={[...base, 'power', 'method']} label={t('opname.lighting.power')}
          options={opts(t, 'opname.lighting.powerKind', ['luminaires', 'lamps', 'unknown'])}
          onChange={(_, value) => change([...base, 'power'], value === 'lamps'
            ? { method: 'lamps', lamps: [{ count: 1, lampPowerW: null, lampType: 'unknown' }] }
            : value === 'luminaires' ? { method: 'luminaires', luminaires: [{ count: 1, powerW: null }] }
              : { method: 'unknown', ledFrom2017: false })} />
        {method === 'unknown' && <CheckField {...field} path={[...base, 'power', 'ledFrom2017']} label={t('opname.lighting.ledFrom2017')} />}
        {(method === 'lamps' || method === 'luminaires') && parts.map((_, part) => <div key={part} className="nta-form-row" role="group"
          aria-label={`${t('opname.lighting.part')} ${part + 1}`}>
          <NumberField {...field} path={[...base, 'power', method, part, 'count']} label={t('opname.lighting.count')} step="1" />
          {method === 'lamps' ? <>
            <NumberField {...field} path={[...base, 'power', 'lamps', part, 'lampPowerW']} label={t('opname.lighting.lampPower')} unit="W" />
            <SelectField {...field} path={[...base, 'power', 'lamps', part, 'lampType']} label={t('opname.lighting.lampType')}
              options={opts(t, 'opname.lighting.lampTypeKind', LAMP_TYPES)} />
          </> : <NumberField {...field} path={[...base, 'power', 'luminaires', part, 'powerW']} label={t('opname.lighting.luminairePower')} unit="W" />}
          {parts.length > 1 && <button type="button" onClick={() => change([...base, 'power', method],
            parts.filter((__, other) => other !== part))}>{t('opname.remove')}</button>}
        </div>)}
        {(method === 'lamps' || method === 'luminaires') && <button type="button" onClick={() => change([...base, 'power', method],
          [...parts, method === 'lamps' ? { count: 1, lampPowerW: null, lampType: 'unknown' } : { count: 1, powerW: null }])}>
          {t('opname.lighting.addPart')}</button>}
        <SelectField {...field} path={[...base, 'presence']} label={t('opname.lighting.presence')}
          options={opts(t, 'opname.lighting.presenceKind', PRESENCE)} />
        <SelectField {...field} path={[...base, 'daylight']} label={t('opname.lighting.daylight')} fallback="none"
          options={opts(t, 'opname.lighting.daylightKind', ['none', 'switching', 'dimming', 'unknown'])} />
        <CheckField {...field} path={[...base, 'largeOfficeGroup']} label={t('opname.lighting.largeOfficeGroup')} />
        <CheckField {...field} path={[...base, 'extractedLuminaires']} label={t('opname.lighting.extracted')} />
        <TextField {...field} path={[...base, 'sourceReference']} label={t('opname.lighting.source')} />
        <button type="button" onClick={() => change(['lighting'], zones.filter((__, other) => other !== index))}>
          {t('opname.remove')}</button>
      </div>;
    })}
    <button type="button" onClick={() => change(['lighting'], [...zones, {
      id: freshItemId(zones, 'verlichting'), areaM2: null, power: { method: 'unknown', ledFrom2017: false },
      presence: 'none_or_central_on', sourceReference: '' }])}>{t('opname.lighting.add')}</button>
  </>;
}

/** ISSO 75.1 §11.6/§12: humidification, BACS, the air handling unit's ducts, flow control and recirculation. */
export function UtilityInstallationExtrasFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const humidification = read(draft, ['humidification']);
  const flow = read(draft, ['ventilation', 'flowControl']);
  const ahu = read(draft, ['ventilation', 'ahu']);
  return <>
    {/* utility.rs 2575: a proven percentage wins over the answer, so it only exists while recirculation is present. */}
    <SelectField {...field} path={['ventilation', 'recirculation']} label={t('opname.ventilation.recirculation')}
      options={opts(t, 'opname.ventilation.recirculationKind', ['none', 'present_percent_unknown', 'unknown'])}
      onChange={(_, value) => change(['ventilation'], {
        ...(read(draft, ['ventilation']) as Record<string, unknown> | undefined ?? {}),
        recirculation: value ?? undefined,
        recirculationPercent: value === 'present_percent_unknown' ? read(draft, ['ventilation', 'recirculationPercent']) ?? undefined : undefined,
      })} />
    {read(draft, ['ventilation', 'recirculation']) === 'present_percent_unknown' &&
      <NumberField {...field} path={['ventilation', 'recirculationPercent']} label={t('opname.ventilation.recirculationPercent')}
        optional step="1" unit="%" onChange={(path, value) => change(path, value ?? undefined)} />}
    <TriStateField {...field} {...yesNo} path={['ventilation', 'ductsLukaAbc']} label={t('opname.ventilation.ductsLukaAbc')} />
    <label className="nta-form-check">
      <input type="checkbox" checked={flow != null} onChange={(event) => change(['ventilation', 'flowControl'],
        event.target.checked ? { method: 'speed_control', minimumPercent: null } : null)} />
      {t('opname.ventilation.flowControl')}
    </label>
    {flow != null && <>
      <SelectField {...field} path={['ventilation', 'flowControl', 'method']} label={t('opname.ventilation.flowControlMethod')}
        options={opts(t, 'opname.ventilation.flowControlKind', ['throttle', 'inlet_vane', 'blade_pitch', 'speed_control', 'other'])} />
      <NumberField {...field} path={['ventilation', 'flowControl', 'minimumPercent']} label={t('opname.ventilation.flowMinimum')} optional unit="%" />
    </>}
    {ahu != null && <>
      <TriStateField {...field} {...yesNo} path={['ventilation', 'ahu', 'ductsOutsideThermalZone']} label={t('opname.ahu.ductsOutside')} />
      <SelectField {...field} path={['ventilation', 'ahu', 'ductLength']} label={t('opname.ahu.ductLength')}
        options={opts(t, 'opname.ahu.ductLengthKind', ['at_most20_m', 'from20_to40_m', 'at_least40_m'])} />
      <TriStateField {...field} {...yesNo} path={['ventilation', 'ahu', 'ductsInsulated']} label={t('opname.ahu.ductsInsulated')} />
    </>}
    <label className="nta-form-check">
      <input type="checkbox" checked={humidification != null} onChange={(event) => change(['humidification'],
        event.target.checked ? { humidifier: 'adiabatic', absorptionWheel: false, sourceReference: '' } : null)} />
      {t('opname.humidification')}
    </label>
    {humidification != null && <>
      <SelectField {...field} path={['humidification', 'humidifier']} label={t('opname.humidification.kind')}
        options={opts(t, 'opname.humidification.kindKind', ['electric_steam', 'non_electric_steam', 'adiabatic'])} />
      <CheckField {...field} path={['humidification', 'absorptionWheel']} label={t('opname.humidification.wheel')} />
      <TextField {...field} path={['humidification', 'sourceReference']} label={t('opname.humidification.source')} />
    </>}
    {/* Table 7.3 / §5.5.8: the survey forms state whether a system above 290 kW is present. */}
    <TriStateField {...field} {...yesNo} path={['bacs', 'above290Kw']} label={t('survey.bacs.above290')} />
    <TriStateField {...field} {...yesNo} path={['bacs', 'present']} label={t('opname.bacs.present')} />
    {read(draft, ['bacs', 'present']) === true && <>
      <NumberField {...field} path={['bacs', 'systemPowerKw']} label={t('opname.bacs.power')} optional unit="kW" />
      <NumberField {...field} path={['bacs', 'servedAreaM2']} label={t('opname.bacs.area')} optional unit="m²" />
      <TriStateField {...field} {...yesNo} path={['bacs', 'automationClassCOrBetter']} label={t('opname.bacs.automationC')} />
      <TriStateField {...field} {...yesNo} path={['bacs', 'managementClassBOrBetter']} label={t('opname.bacs.managementB')} />
      <TextField {...field} path={['bacs', 'evidenceReference']} label={t('opname.bacs.source')} />
    </>}
    <CheckField {...field} path={['heatingInstallation', 'collective']} label={t('opname.heatingInstallation.collective')} />
    <NumberField {...field} path={['heatingInstallation', 'capacityKw']} label={t('opname.heatingInstallation.capacity')} optional unit="kW" />
    {read(draft, ['cooling']) != null &&
      <NumberField {...field} path={['cooling', 'pipeInsulationYear']} label={t('opname.cooling.pipeInsulationYear')} optional step="1" />}
  </>;
}

/** ISSO 82.1/75.1 §8: survey surface answers for reed roofs, crawlspace floors, proven insulation and unheated spaces. */
/** NTA 8800 editions that take the measured height h of (8.47) (2022 p. 236); from 2023 h is fixed. */
const MEASURED_WALL_HEIGHT_EDITIONS = new Set(['2022', '2020+A1']);

export function SurfaceExtrasFields({ draft, change, t, base, edition }: Props & { base: Path; edition?: string | null }) {
  const field = { draft, onChange: change };
  const element = read(draft, [...base, 'element']);
  const boundary = read(draft, [...base, 'boundary', 'kind']);
  const spaces = items(draft, ['envelope', 'unheatedSpaces']);
  return <>
    {element === 'roof' && <NumberField {...field} path={[...base, 'reedThicknessMm']} label={t('opname.surface.reed')} optional unit="mm" />}
    {boundary === 'crawlspace' && <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')}
      path={[...base, 'crawlspaceBottomInsulated']} label={t('opname.surface.crawlspaceBottom')} />}
    {(boundary === 'crawlspace' || boundary === 'unheated_cellar') && edition != null && MEASURED_WALL_HEIGHT_EDITIONS.has(edition) &&
      <NumberField {...field} path={[...base, 'floorHeightAboveGroundM']} label={t('opname.surface.floorHeightAboveGround')} optional unit="m" />}
    {read(draft, [...base, 'insulation', 'kind']) === 'thickness' &&
      <CheckField {...field} path={[...base, 'insulation', 'proven']} label={t('opname.surface.proven')} />}
    {boundary === 'unheated_space' && <SelectField {...field} path={[...base, 'boundary', 'spaceId']} label={t('opname.surface.space')}
      options={spaces.map((space) => [String(space.id), space.description ? `${String(space.id)} — ${String(space.description)}` : String(space.id)])} />}
  </>;
}

/** ISSO 82.1/75.1 §8: doors with their surface, area, insulation, glass share and frame. */
export function SurveyDoorsFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const doors = items(draft, ['envelope', 'doors']);
  const surfaces = items(draft, ['envelope', 'surfaces']);
  return <>
    {doors.map((door, index) => {
      const base: Path = ['envelope', 'doors', index];
      return <div key={index} className="opname-item" role="group" aria-label={String(door.id ?? index + 1)}>
        <strong>{String(door.id ?? index + 1)}</strong>
        <SelectField {...field} path={[...base, 'surfaceId']} label={t('opname.window.surface')}
          options={surfaces.map((surface) => [String(surface.id), String(surface.id)])} />
        <NumberField {...field} path={[...base, 'areaM2']} label={t('opname.door.area')} unit="m²" />
        <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={[...base, 'insulated']} label={t('opname.door.insulated')} />
        <NumberField {...field} path={[...base, 'glassFraction']} label={t('opname.door.glassFraction')} optional
          onChange={(path, value) => change(path, value ?? undefined)} />
        {typeof door.glassFraction === 'number' && door.glassFraction > 0 &&
          <SelectField {...field} path={[...base, 'glass']} label={t('opname.door.glass')}
            options={opts(t, 'opname.window.glassKind', SURVEY_GLASS_KINDS)} />}
        <SelectField {...field} path={[...base, 'frame']} label={t('opname.window.frame')}
          options={opts(t, 'opname.window.frameKind', ['wood_or_plastic', 'metal_with_thermal_break', 'metal', 'none'])} />
        <TextField {...field} path={[...base, 'sourceReference']} label={t('opname.door.source')} />
        <button type="button" onClick={() => change(['envelope', 'doors'], doors.filter((__, other) => other !== index))}>
          {t('opname.remove')}</button>
      </div>;
    })}
    <button type="button" onClick={() => change(['envelope', 'doors'], [...doors, {
      id: freshItemId(doors, 'deur'), surfaceId: String(surfaces[0]?.id ?? ''), areaM2: 2, frame: 'wood_or_plastic',
      sourceReference: '' }])}>{t('opname.door.add')}</button>
  </>;
}

/** ISSO 75.1 p. 94: windows with solar-control glass, by id. */
export function SolarControlWindowsFields({ draft, change, t }: Props) {
  const windows = items(draft, ['envelope', 'windows']);
  const stored = read(draft, ['solarControlWindowIds']);
  const chosen = new Set(Array.isArray(stored) ? stored as string[] : []);
  if (windows.length === 0) return null;
  return <fieldset className="nta-form-row"><legend>{t('opname.solarControlWindows')}</legend>
    {windows.map((window) => {
      const id = String(window.id);
      return <label key={id} className="nta-form-check">
        <input type="checkbox" checked={chosen.has(id)} onChange={(event) => {
          const next = new Set(chosen);
          if (event.target.checked) next.add(id); else next.delete(id);
          change(['solarControlWindowIds'], [...next]);
        }} />{id}
      </label>;
    })}
  </fieldset>;
}

/** Hot-water taps: showers, and for dwellings the connected bathrooms and kitchens (13.19a); utility draw-off length. */
export function HotWaterTapsFields({ draft, change, t, utility }: Props & { utility: boolean }) {
  const field = { draft, onChange: change };
  return <>
    <NumberField {...field} path={['hotWater', 'showers']} label={t('opname.hotWater.showers')} optional step="1" />
    {utility
      ? <NumberField {...field} path={['hotWater', 'meanDrawOffLengthM']} label={t('opname.hotWater.meanDrawOff')} optional unit="m" />
      : <>
        <NumberField {...field} path={['hotWater', 'connectedBathrooms']} label={t('opname.hotWater.connectedBathrooms')} optional step="1" />
        <NumberField {...field} path={['hotWater', 'connectedKitchens']} label={t('opname.hotWater.connectedKitchens')} optional step="1" />
      </>}
  </>;
}

/**
 * ISSO 82.1/75.1 §8: the unheated spaces a surface can border (`envelope.unheatedSpaces`).
 * The basic survey derives H_ue = 5·A_T;iu (NTA I.8) for each, so only an id and a description.
 */
export function SurveyUnheatedSpacesFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const spaces = items(draft, ['envelope', 'unheatedSpaces']);
  const surfaces = items(draft, ['envelope', 'surfaces']);
  const remove = (index: number) => {
    const id = String(spaces[index]?.id ?? '');
    change(['envelope'], {
      ...(read(draft, ['envelope']) as Record<string, unknown>),
      unheatedSpaces: spaces.filter((__, other) => other !== index),
      // A surface that bordered the removed space falls back to outdoor air, so no dangling spaceId remains.
      surfaces: surfaces.map((surface) => (surface.boundary as Draft | undefined)?.kind === 'unheated_space'
        && (surface.boundary as Draft).spaceId === id ? { ...surface, boundary: { kind: 'outdoor' } } : surface),
    });
  };
  return <fieldset className="nta-form-row"><legend>{t('opname.unheatedSpaces')}</legend>
    {spaces.map((space, index) => <div key={index} className="opname-item" role="group" aria-label={String(space.id ?? index + 1)}>
      <strong>{String(space.id ?? index + 1)}</strong>
      <TextField {...field} path={['envelope', 'unheatedSpaces', index, 'description']} label={t('opname.unheatedSpaces.description')} />
      <button type="button" onClick={() => remove(index)}>{t('opname.remove')}</button>
    </div>)}
    <button type="button" onClick={() => change(['envelope', 'unheatedSpaces'],
      [...spaces, { id: freshItemId(spaces, 'onverwarmd'), description: '' }])}>{t('opname.unheatedSpaces.add')}</button>
  </fieldset>;
}

/** Survey fields the kernel requires as numbers; an empty one would make the kernel refuse the whole survey. */
export function emptyRequiredSurveyFields(survey: unknown): string[] {
  const missing: string[] = [];
  const draft = (survey ?? {}) as Draft;
  const measured = read(draft, ['measuredInfiltration']);
  if (measured != null && typeof read(draft, ['measuredInfiltration', 'qv10Dm3PerSM2']) !== 'number') {
    missing.push('measuredInfiltration.qv10Dm3PerSM2');
  }
  items(draft, ['lighting']).forEach((zone, index) => {
    if (typeof zone.areaM2 !== 'number') missing.push(`lighting[${index}].areaM2`);
    const power = (zone.power ?? {}) as Draft;
    const method = power.method;
    if (method === 'lamps' || method === 'luminaires') {
      const key = method === 'lamps' ? 'lampPowerW' : 'powerW';
      (Array.isArray(power[method]) ? power[method] as Draft[] : []).forEach((part, partIndex) => {
        if (typeof part.count !== 'number') missing.push(`lighting[${index}].power.${method}[${partIndex}].count`);
        if (typeof part[key] !== 'number') missing.push(`lighting[${index}].power.${method}[${partIndex}].${key}`);
      });
    }
  });
  items(draft, ['envelope', 'panels']).forEach((panel, index) => {
    if (typeof panel.areaM2 !== 'number') missing.push(`envelope.panels[${index}].areaM2`);
    if (typeof panel.surfaceId !== 'string' || panel.surfaceId === '') missing.push(`envelope.panels[${index}].surfaceId`);
    if (typeof panel.frame !== 'string') missing.push(`envelope.panels[${index}].frame`);
    const insulation = (panel.insulation ?? {}) as Draft;
    if (insulation.kind === 'known_thickness' && typeof insulation.thicknessMm !== 'number') {
      missing.push(`envelope.panels[${index}].insulation.thicknessMm`);
    }
  });
  items(draft, ['hotWater', 'storage']).forEach((vessel, index) => {
    if (typeof vessel.volumeL !== 'number') missing.push(`hotWater.storage[${index}].volumeL`);
  });
  items(draft, ['envelope', 'surfaces']).forEach((surface, index) => {
    const boundary = (surface.boundary ?? {}) as Draft;
    if (boundary.kind === 'unheated_space' && (typeof boundary.spaceId !== 'string' || boundary.spaceId === '')) {
      missing.push(`envelope.surfaces[${index}].boundary.spaceId`);
    }
  });
  return missing;
}
