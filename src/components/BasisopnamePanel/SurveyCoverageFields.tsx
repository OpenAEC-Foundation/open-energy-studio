import {
  CheckField, NumberField, read, SelectField, TextField, TriStateField, type Draft, type Path,
} from '../NtaPerformancePanel/NtaFormFields';
import { freshItemId } from '../../core/nta/EvidenceLinks';

// Survey answers that only the JSON view reached before (form coverage,
// docs/ui-redesign/formulierdekking.md): the area source and storage, the
// orientation, tilt, cavity and exposed perimeter of a surface, panels,
// declared window obstruction, the heating balance and storeys, PV mounting
// and obstruction factors, the bypass, the taps served, shower heat
// recovery, and the utility cooling and hot-water answers (ISSO 82.1/75.1).

type Change = (path: Path, value: unknown) => void;
type T = (key: string, options?: Record<string, unknown>) => string;
interface Props { draft: Draft; change: Change; t: T }

function opts(t: T, prefix: string, keys: string[]): Array<[string, string]> {
  return keys.map((key) => [key, t(`${prefix}.${key}`)]);
}

function items(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

/** An optional answer: an empty choice leaves the field out, so the kernel takes its default. */
const omitEmpty = (change: Change) => (path: Path, value: unknown) => change(path, value == null || value === '' ? undefined : value);

const ORIENTATIONS = ['north', 'north_east', 'east', 'south_east', 'south', 'south_west', 'west', 'north_west'];

/**
 * A list of numbers as text ("1; 0,9; …"): one value or twelve monthly values (§17.3). Decimal commas
 * are read as points; an empty part (an emptied field or a trailing ";") is dropped, not read as 0,
 * and a part that is not a number is dropped, so the kernel reports a wrong count. An empty field
 * leaves the list out.
 */
export function parseNumberList(text: string): number[] | undefined {
  const numbers = text.split(';')
    .map((part) => part.trim())
    .filter((part) => part !== '')
    .map((part) => Number(part.replace(',', '.')))
    .filter((number) => Number.isFinite(number));
  return numbers.length > 0 ? numbers : undefined;
}

function NumberListField({ draft, change, path, label, hint }: { draft: Draft; change: Change; path: Path; label: string; hint: string }) {
  const value = read(draft, path);
  const text = Array.isArray(value) ? (value as unknown[]).map((item) => String(item).replace('.', ',')).join('; ') : '';
  return <label>{label}
    <input type="text" defaultValue={text} key={text} placeholder={hint}
      onBlur={(event) => change(path, parseNumberList(event.target.value))} />
  </label>;
}

/** The source of the usable floor area (NEN 2580, p. 60) and building-bound storage (§15.5). */
export function SurveyGeneralCoverageFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const storage = read(draft, ['storage']);
  // §15.5: building-bound storage counts only together with PV (storage_requires_pv).
  const pv = items(draft, ['pv']).length > 0;
  return <>
    <TextField {...field} path={['areaSourceReference']} label={t('opname.areaSourceReference')} />
    {(pv || storage != null) && <label className="nta-form-check">
      <input type="checkbox" checked={storage != null} onChange={(event) => change(['storage'],
        event.target.checked ? { electricalKwh: 0, thermalKwh: 0, sourceReference: '' } : undefined)} />
      {t('opname.storage')}
    </label>}
    {storage != null && <>
      {/* §15.5: both capacities default to 0 in the kernel, so an empty field is left out. */}
      <NumberField {...field} path={['storage', 'electricalKwh']} label={t('opname.storage.electrical')} unit="kWh"
        optional onChange={omitEmpty(change)} />
      <NumberField {...field} path={['storage', 'thermalKwh']} label={t('opname.storage.thermal')} unit="kWh"
        optional onChange={omitEmpty(change)} />
      <TextField {...field} path={['storage', 'sourceReference']} label={t('opname.storage.source')} />
    </>}
  </>;
}

/** ISSO 82.1/75.1 §8: orientation and tilt of a surface, a cavity, and the exposed perimeter of a floor. */
export function SurfaceGeometryFields({ draft, change, t, base }: Props & { base: Path }) {
  const field = { draft, onChange: change };
  const element = read(draft, [...base, 'element']);
  const boundary = read(draft, [...base, 'boundary', 'kind']);
  const groundFloor = element === 'floor' && (boundary === 'ground' || boundary === 'crawlspace' || boundary === 'unheated_cellar');
  // ISSO 82.1 §8.4 (7e druk p. 74–75): a façade part against the ground goes with its floor, with H1.
  const groundWall = element === 'facade' && boundary === 'ground';
  const floors = groundWall ? items(draft, ['envelope', 'surfaces']).filter((surface) => surface.element === 'floor'
    && ['ground', 'crawlspace', 'unheated_cellar'].includes(String((surface.boundary as Draft | undefined)?.kind))) : [];
  return <>
    {element !== 'floor' && <SelectField {...field} path={[...base, 'orientation']} label={t('opname.surface.orientation')}
      options={opts(t, 'opname.orientationKind', ORIENTATIONS)} onChange={omitEmpty(change)} />}
    <NumberField {...field} path={[...base, 'tiltDeg']} label={t('opname.surface.tilt')} optional unit="°"
      onChange={omitEmpty(change)} />
    <CheckField {...field} path={[...base, 'cavity']} label={t('opname.surface.cavity')} />
    {groundFloor && <NumberField {...field} path={[...base, 'exposedPerimeterM']} label={t('opname.surface.exposedPerimeter')}
      optional unit="m" onChange={omitEmpty(change)} />}
    {groundWall && <>
      <NumberField {...field} path={[...base, 'depthBelowGroundM']} label={t('opname.surface.depthBelowGround')}
        optional unit="m" onChange={omitEmpty(change)} />
      <SelectField {...field} path={[...base, 'groundFloorId']} label={t('opname.surface.groundFloor')}
        options={floors.map((floor) => [String(floor.id), String(floor.id)])} onChange={omitEmpty(change)} />
    </>}
  </>;
}

/** ISSO 82.1/75.1 §8: opaque panels in a surface (insulation, cavity, frame). */
export function SurveyPanelsFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const panels = items(draft, ['envelope', 'panels']);
  const surfaces = items(draft, ['envelope', 'surfaces']);
  return <fieldset className="nta-form-row"><legend>{t('opname.panels')}</legend>
    {panels.map((panel, index) => {
      const base: Path = ['envelope', 'panels', index];
      const insulation = read(draft, [...base, 'insulation', 'kind']);
      return <div key={index} className="opname-item" role="group" aria-label={String(panel.id ?? index + 1)}>
        <strong>{String(panel.id ?? index + 1)}</strong>
        <SelectField {...field} path={[...base, 'surfaceId']} label={t('opname.panel.surface')}
          options={surfaces.map((surface) => [String(surface.id), String(surface.id)])}
          fallback={surfaces[0] == null ? undefined : String(surfaces[0].id)} />
        <NumberField {...field} path={[...base, 'areaM2']} label={t('opname.panel.area')} unit="m²" />
        <label>{t('opname.panel.insulation')}
          <select value={typeof insulation === 'string' ? insulation : 'absent_or_unknown'}
            onChange={(event) => change([...base, 'insulation'], event.target.value === 'known_thickness'
              ? { kind: 'known_thickness', thicknessMm: 50 } : { kind: event.target.value })}>
            {['absent_or_unknown', 'present_unknown_thickness', 'known_thickness'].map((key) =>
              <option key={key} value={key}>{t(`opname.panel.insulationKind.${key}`)}</option>)}
          </select>
        </label>
        {insulation === 'known_thickness' &&
          <NumberField {...field} path={[...base, 'insulation', 'thicknessMm']} label={t('opname.panel.thickness')} unit="mm" />}
        <CheckField {...field} path={[...base, 'cavity']} label={t('opname.panel.cavity')} />
        <SelectField {...field} path={[...base, 'frame']} label={t('opname.panel.frame')}
          options={opts(t, 'opname.window.frameKind', ['wood_or_plastic', 'metal_with_thermal_break', 'metal', 'none'])}
          fallback="wood_or_plastic" />
        <TextField {...field} path={[...base, 'sourceReference']} label={t('opname.panel.source')} />
        <button type="button" onClick={() => change(['envelope', 'panels'], panels.filter((__, other) => other !== index))}>
          {t('opname.remove')}</button>
      </div>;
    })}
    <button type="button" onClick={() => change(['envelope', 'panels'], [...panels, {
      id: freshItemId(panels, 'paneel'), surfaceId: String(surfaces[0]?.id ?? ''), areaM2: 1,
      insulation: { kind: 'absent_or_unknown' }, cavity: false, frame: 'wood_or_plastic', sourceReference: '',
    }])}>{t('opname.panels.add')}</button>
  </fieldset>;
}

/** §17.3: declared obstruction factors of a window (12 heating, 12 cooling), instead of a table 8.24/8.25 situation. */
export function WindowDeclaredObstructionFields({ draft, change, t, base }: Props & { base: Path }) {
  const declared = read(draft, [...base, 'obstruction']) != null;
  const window = read(draft, base) as Draft | undefined;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={declared} onChange={(event) => change(base, {
        ...(window ?? {}),
        // Declared factors and a situation exclude each other (window_obstruction_conflict).
        shading: event.target.checked ? undefined : window?.shading,
        obstruction: event.target.checked
          ? { heating: Array(12).fill(1), cooling: Array(12).fill(1), sourceReference: '' } : undefined,
      })} />
      {t('opname.window.obstructionDeclared')}
    </label>
    {declared && <>
      <NumberListField draft={draft} change={change} path={[...base, 'obstruction', 'heating']}
        label={t('opname.window.obstructionHeating')} hint={t('opname.monthlyListHint')} />
      <NumberListField draft={draft} change={change} path={[...base, 'obstruction', 'cooling']}
        label={t('opname.window.obstructionCooling')} hint={t('opname.monthlyListHint')} />
      <TextField draft={draft} onChange={change} path={[...base, 'obstruction', 'sourceReference']}
        label={t('opname.window.obstructionSource')} />
    </>}
  </>;
}

/** ISSO 82.1/75.1 §9: hydronic balancing and the storeys the distribution serves (9.37). */
export function SurveyHeatingBalanceFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  return <>
    <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={['heating', 'balanced']} label={t('opname.heating.balanced')} />
    <NumberField {...field} path={['heating', 'storeys']} label={t('opname.heating.storeys')} optional step="1"
      onChange={omitEmpty(change)} />
  </>;
}

/** ISSO 82.1/75.1 §15.4: PV module ventilation (table 16.2) and declared obstruction factors F_sh;obst. */
export function PvMountingFields({ draft, change, t, base }: Props & { base: Path }) {
  const field = { draft, onChange: change };
  const factors = read(draft, [...base, 'obstructionFactors']) != null;
  const pv = read(draft, base) as Draft | undefined;
  return <>
    <SelectField {...field} path={[...base, 'mounting']} label={t('opname.pv.mounting')}
      options={opts(t, 'opname.pv.mountingKind', ['not_ventilated', 'moderately_ventilated', 'strongly_ventilated', 'unknown'])}
      fallback="unknown" />
    <label className="nta-form-check">
      <input type="checkbox" checked={factors} onChange={(event) => change(base, {
        ...(pv ?? {}),
        // Factors and a shading situation exclude each other (pv_shading_declared_twice).
        shading: event.target.checked ? undefined : pv?.shading,
        obstructionFactors: event.target.checked ? [1] : undefined,
      })} />
      {t('opname.pv.obstructionFactors')}
    </label>
    {factors && <NumberListField draft={draft} change={change} path={[...base, 'obstructionFactors']}
      label={t('opname.pv.obstructionFactors.values')} hint={t('opname.oneOrTwelveHint')} />}
  </>;
}

/**
 * ISSO 82.1/75.1 §11: a bypass on the heat recovery (table 11.13). Shown for balanced ventilation and
 * for a combined system (E), whose heat recovery the kernel also reads it for, including
 * `passive_cooling_requires_bypass` (opname/ventilation.rs).
 */
export function VentilationBypassField({ draft, change, t }: Props) {
  const balanced = read(draft, ['ventilation', 'principle']) === 'balanced';
  const combined = read(draft, ['ventilation', 'combined']) != null;
  if (!balanced && !combined) return null;
  return <TriStateField draft={draft} onChange={change} yes={t('opname.yes')} no={t('opname.no')}
    path={['ventilation', 'bypassPresent']} label={t('opname.ventilation.bypass')} />;
}

/** ISSO 82.1 §13: the taps the generator serves and the pipe lengths to them (13.28). */
export function SurveyHotWaterServedFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const served = read(draft, ['hotWater', 'served']);
  return <>
    <SelectField {...field} path={['hotWater', 'served']} label={t('opname.hotWater.served')}
      options={opts(t, 'opname.hotWater.servedKind', ['kitchen_and_bathroom', 'bathroom_only', 'kitchen_only'])}
      fallback="kitchen_and_bathroom" />
    {served !== 'bathroom_only' && <NumberField {...field} path={['hotWater', 'kitchenLengthM']}
      label={t('opname.hotWater.kitchenLength')} optional unit="m" onChange={omitEmpty(change)} />}
    {served !== 'kitchen_only' && <NumberField {...field} path={['hotWater', 'bathroomLengthM']}
      label={t('opname.hotWater.bathroomLength')} optional unit="m" onChange={omitEmpty(change)} />}
  </>;
}

/** ISSO 82.1/75.1 table 13.16: shower heat recovery (vertical, horizontal, or not determinable). */
export function SurveyShowerHeatRecoveryField({ draft, change, t }: Props) {
  return <SelectField draft={draft} onChange={change} path={['hotWater', 'showerHeatRecovery']}
    label={t('opname.hotWater.showerHeatRecovery')} fallback="none"
    options={opts(t, 'opname.hotWater.showerHeatRecoveryKind', ['none', 'vertical', 'horizontal', 'unknown'])} />;
}

/**
 * ISSO 75.1 §10: design temperature, control and pipe insulation of the cooling (tables 10.5–10.7).
 * The design temperature and the pipe insulation belong to a water distribution; the kernel reads them
 * only for water-based cooling (opname/utility.rs), so they are shown only then.
 */
export function UtilityCoolingCoverageFields({ draft, change, t, base = ['cooling'] }: Props & { base?: Path }) {
  const field = { draft, onChange: change };
  const waterBased = read(draft, [...base, 'waterBased']) === true;
  return <>
    {waterBased && <SelectField {...field} path={[...base, 'designTemperature']} label={t('opname.cooling.designTemperature')}
      options={opts(t, 'opname.cooling.designTemperatureKind', ['t6_to12', 't12_to16', 't12_to18', 't17_to21'])}
      onChange={omitEmpty(change)} />}
    <SelectField {...field} path={[...base, 'control']} label={t('opname.cooling.control')}
      options={opts(t, 'opname.cooling.controlKind', ['standalone', 'central_with_room_control', 'other_or_unknown'])}
      onChange={omitEmpty(change)} />
    {waterBased && <TriStateField {...field} yes={t('opname.yes')} no={t('opname.no')} path={[...base, 'pipesInsulated']}
      label={t('opname.cooling.pipesInsulated')} />}
  </>;
}

/** ISSO 75.1 §13: circulation and the vessels whose losses are not in the generator efficiency (p. 172–173). */
export function UtilityHotWaterStorageFields({ draft, change, t }: Props) {
  const field = { draft, onChange: change };
  const yesNo = { yes: t('opname.yes'), no: t('opname.no') };
  const vessels = items(draft, ['hotWater', 'storage']);
  return <>
    <TriStateField {...field} {...yesNo} path={['hotWater', 'circulation']} label={t('opname.hotWater.circulation')} />
    <fieldset className="nta-form-row"><legend>{t('opname.hotWater.vessels')}</legend>
      {vessels.map((vessel, index) => {
        const base: Path = ['hotWater', 'storage', index];
        return <div key={index} className="opname-item" role="group" aria-label={String(vessel.id ?? index + 1)}>
          <strong>{String(vessel.id ?? index + 1)}</strong>
          <NumberField {...field} path={[...base, 'volumeL']} label={t('opname.vessel.volume')} unit="l" />
          <SelectField {...field} path={[...base, 'label']} label={t('opname.vessel.energyLabel')}
            options={['a_plus', 'a', 'b', 'c', 'd', 'e', 'f', 'g'].map((key) => [key, key === 'a_plus' ? 'A+' : key.toUpperCase()])}
            onChange={omitEmpty(change)} />
          <TriStateField {...field} {...yesNo} path={[...base, 'producedFrom2018']} label={t('opname.vessel.from2018')} />
          <SelectField {...field} path={[...base, 'connection']} label={t('opname.vessel.connection')}
            options={opts(t, 'opname.vessel.connectionKind', ['fully_insulated', 'straight_only_four', 'straight_only_more_than_four', 'uninsulated'])}
            onChange={omitEmpty(change)} />
          <TriStateField {...field} {...yesNo} path={[...base, 'inHeatedZone']} label={t('opname.vessel.inHeatedZone')} />
          <TextField {...field} path={[...base, 'sourceReference']} label={t('opname.vessel.source')} />
          <button type="button" onClick={() => change(['hotWater', 'storage'], vessels.filter((__, other) => other !== index))}>
            {t('opname.remove')}</button>
        </div>;
      })}
      <button type="button" onClick={() => change(['hotWater', 'storage'], [...vessels, {
        id: freshItemId(vessels, 'vat'), volumeL: 150, sourceReference: '',
      }])}>{t('opname.hotWater.addVessel')}</button>
    </fieldset>
  </>;
}
