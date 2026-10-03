import { useI18n } from '../../i18n/i18n';
import type { IProject } from '../../core/energy/types';
import {
  chpClassTemplate, collectiveConnectionTemplate, coolingSystemTemplate, declaredRenewableHeatTemplate, declaredUseTemplate,
  heatPumpRenewableTemplate, humidifierTemplate, onSiteProductionTemplate, sunroomSurfaceTemplate, sunroomTemplate,
} from '../../core/nta/NtaSystemTemplates';
import {
  CheckField, NumberField, read, SelectField, TextField, type Draft, type Path,
} from './NtaFormFields';
import { CoolingPerformanceFields, MonthlyValues, SolarWaterHeaterFields } from './NtaSystemSections';

// Project inputs of the NTA form without an own section elsewhere: sunrooms
// (7.30b), humidifiers (chapter 12), one or several cooling systems (§10.2),
// solar space heating (§13.7), the collective installation (9.6.1), the
// renewable evidence of a heat pump (5.31/5.32) and declared monthly uses
// and productions (§5.5, chapter 16).

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

function list(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

function RemoveButton({ label, onClick }: { label: string; onClick: () => void }) {
  return <button type="button" className="nta-form-remove" onClick={onClick}>{label}</button>;
}

/** 7.30b adjacent unheated sunrooms with their glazing and surfaces. */
export function SunroomsFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const rooms = list(draft, ['sunrooms']);
  return <div className="nta-form-subsection">
    {rooms.map((_, index) => {
      const at = (...rest: Path): Path => ['sunrooms', index, ...rest];
      const surfaces = list(draft, at('surfaces'));
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.form.sunroom')} {index + 1}</legend>
        <TextField {...field} path={at('id')} label={t('nta.form.sunroom.id')} />
        <NumberField {...field} path={at('glazingGHeating')} label={t('nta.form.sunroom.gHeating')} />
        <NumberField {...field} path={at('glazingGCooling')} label={t('nta.form.sunroom.gCooling')} />
        <NumberField {...field} path={at('exteriorFrameFraction')} label={t('nta.form.sunroom.frame')} />
        <NumberField {...field} path={at('reductionFactor')} label={t('nta.form.sunroom.reduction')} />
        <NumberField {...field} path={at('zoneConductanceWPerK')} label={t('nta.form.sunroom.conductance')} />
        <NumberField {...field} path={at('distributionFactor')} label={t('nta.form.sunroom.distribution')} />
        {surfaces.map((__, surface) => <div key={surface} className="nta-form-row">
          <NumberField {...field} path={at('surfaces', surface, 'areaM2')} label={t('nta.form.sunroom.surfaceArea')} />
          <NumberField {...field} path={at('surfaces', surface, 'absorptance')} label={t('nta.form.sunroom.absorptance')} />
          <NumberField {...field} path={at('surfaces', surface, 'azimuthDeg')} label={t('nta.form.sunroom.azimuth')} />
          <NumberField {...field} path={at('surfaces', surface, 'tiltDeg')} label={t('nta.form.tilt')} />
          <RemoveButton label={t('nta.form.sunroom.removeSurface')}
            onClick={() => change(at('surfaces'), surfaces.filter((___, other) => other !== surface))} />
        </div>)}
        <button type="button" onClick={() => change(at('surfaces'), [...surfaces, sunroomSurfaceTemplate()])}>
          {t('nta.form.sunroom.addSurface')}
        </button>
        <TextField {...field} path={at('sourceReference')} label={t('nta.form.source')} />
        <RemoveButton label={t('nta.form.remove')} onClick={() => change(['sunrooms'], rooms.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" className="btn" onClick={() => change(['sunrooms'], [...rooms, sunroomTemplate(rooms.length)])}>
      {t('nta.form.sunroom.add')}
    </button>
  </div>;
}

/** Chapter 12: an atomising or steam humidifier per calculation zone. */
export function HumidifiersFields({ draft, change, project }: SectionProps & { project: IProject }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const rows = list(draft, ['humidifiers']);
  const zones: Array<[string, string]> = project.zones.map((zone) => [zone.id, zone.name || zone.id]);
  return <div className="nta-form-subsection">
    {rows.map((_, index) => {
      const at = (...rest: Path): Path => ['humidifiers', index, ...rest];
      const kind = read(draft, at('humidification', 'humidifier', 'kind'));
      return <fieldset key={index} className="nta-form-row">
        <legend>{t('nta.form.humidifier')} {index + 1}</legend>
        <SelectField {...field} path={at('zoneId')} label={t('nta.form.humidifier.zone')} options={zones} />
        <SelectField {...field} path={at('humidification', 'humidifier', 'kind')} label={t('nta.form.humidifier.kind')} options={[
          ['atomising', t('nta.form.humidifier.atomising')], ['steam', t('nta.form.humidifier.steam')]]}
          onChange={(_, value) => change(at('humidification', 'humidifier'), value === 'steam'
            ? { kind: 'steam', carrier: 'electricity' } : { kind: 'atomising' })} />
        {kind === 'steam' && <SelectField {...field} path={at('humidification', 'humidifier', 'carrier')}
          label={t('nta.form.humidifier.carrier')} options={[
            ['electricity', t('nta.form.humidifier.electricity')], ['gas_or_oil', t('nta.form.humidifier.gasOrOil')]]} />}
        <CheckField {...field} path={at('humidification', 'rotaryWheel')} label={t('nta.form.humidifier.rotaryWheel')} />
        <TextField {...field} path={at('humidification', 'equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
        <RemoveButton label={t('nta.form.remove')} onClick={() => change(['humidifiers'], rows.filter((__, other) => other !== index))} />
      </fieldset>;
    })}
    <button type="button" className="btn" onClick={() => change(['humidifiers'], [...rows, humidifierTemplate(project.zones[0]?.id ?? '')])}>
      {t('nta.form.humidifier.add')}
    </button>
  </div>;
}

const COOLING_KINDS = ['compression', 'room_air_conditioner', 'gas_engine_compression', 'gas_absorption',
  'absorption_external_heat', 'absorption_chp', 'external_cold', 'unknown_collective', 'free_cooling'] as const;
const COOLING_KIND_LABELS: Record<string, string> = {
  compression: 'nta.form.cooling.compression', room_air_conditioner: 'nta.form.cooling.rac',
  gas_engine_compression: 'nta.form.cooling.gasEngine',
  gas_absorption: 'nta.form.cooling.absorption', absorption_external_heat: 'nta.form.cooling.absorptionDh',
  absorption_chp: 'nta.form.cooling.absorptionChp',
  external_cold: 'nta.form.cooling.external', unknown_collective: 'nta.form.cooling.unknownCollective',
  free_cooling: 'nta.form.cooling.free',
};

/** The kernel's `CoolingGeneratorKind` of `kind` with its required members. */
function coolingGeneratorTemplate(kind: string): Draft {
  switch (kind) {
    case 'free_cooling':
      return { kind, source: null, heatPumpSource: false, groundAboveZeroDemonstrated: false };
    case 'gas_engine_compression':
      return { kind, gasEngine: chpClassTemplate() };
    case 'absorption_chp':
      return { kind, chp: chpClassTemplate() };
    default:
      return { kind };
  }
}

/** Chapter 10: one cooling system at `base` (`cooling` or `coolingSystems[i].system`). */
export function CoolingSystemFields({ draft, change, base, allowNone }: SectionProps & { base: Path; allowNone: boolean }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => [...base, ...rest];
  const kind = read(draft, at('generators', 0, 'generator', 'kind'));
  const present = read(draft, base) != null;
  return <>
    <label>{t('nta.form.coolingGenerator')}
      <select value={String(kind ?? '')} onChange={(event) => {
        const next = event.target.value;
        if (!next) { change(base, null); return; }
        const current = (read(draft, base) as Draft | null) ?? coolingSystemTemplate();
        const generator = coolingGeneratorTemplate(next);
        // Only the first generator changes kind; further generators stay.
        const generators = list(current, ['generators']);
        const first = generators[0] ?? { id: 'cold-1', capacityKw: null, equipmentReference: '' };
        change(base, { ...current, generators: [{ ...first, generator }, ...generators.slice(1)] });
      }}>
        {(allowNone || kind == null) && <option value="">{t('nta.form.cooling.none')}</option>}
        {COOLING_KINDS.map((key) => <option key={key} value={key}>{t(COOLING_KIND_LABELS[key])}</option>)}
      </select>
    </label>
    {present && <>
      <TextField {...field} path={at('generators', 0, 'equipmentReference')} label={t('nta.form.boilerEquipmentSource')} />
      {(kind === 'compression' || kind === 'room_air_conditioner') &&
        <CoolingPerformanceFields draft={draft} change={change} base={at('generators', 0, 'generator')} />}
      {(kind === 'gas_engine_compression' || kind === 'absorption_chp') && (() => {
        const chp = at('generators', 0, 'generator', kind === 'absorption_chp' ? 'chp' : 'gasEngine');
        return <>
          <NumberField {...field} path={[...chp, 'powerKw']} label={t('nta.form.chp.power')} />
          <CheckField {...field} path={[...chp, 'builtAfter2006']} label={t('nta.form.chp.after2006')} />
          <CheckField {...field} path={[...chp, 'hreDeclared']} label={t('nta.form.chp.hre')} />
        </>;
      })()}
      {kind === 'free_cooling' && <>
        <SelectField {...field} path={at('generators', 0, 'generator', 'source')} label={t('nta.form.freeCoolingSource')} options={[
          ['aquifer_from2013', t('nta.form.free.aquiferNew')], ['aquifer_dwellings_before2013', t('nta.form.free.aquiferOld')],
          ['aquifer_utility_before2013', t('nta.form.free.aquiferUtility')], ['surface_water', t('nta.form.free.surface')],
          ['closed_ground_loop', t('nta.form.free.ground')], ['dew_point_cooling', t('nta.form.free.dewPoint')]]} />
        <CheckField {...field} path={at('generators', 0, 'generator', 'heatPumpSource')} label={t('nta.form.free.heatPumpSource')} />
      </>}
      <SelectField {...field} path={at('emission', 'emitter')} label={t('nta.form.coolingEmitter')} options={[
        ['floor_cooling', t('nta.form.coolEm.floor')], ['wall_cooling', t('nta.form.coolEm.wall')],
        ['fan_coil_or_rac_on_outer_wall', t('nta.form.coolEm.fanWall')], ['ceiling_cooling', t('nta.form.coolEm.ceiling')],
        ['fan_coil_or_rac_on_ceiling', t('nta.form.coolEm.fanCeiling')], ['other_or_unknown', t('nta.form.coolEm.other')]]} />
      <SelectField {...field} path={at('emission', 'balancing')} label={t('nta.form.balancing')} options={[
        ['none_or_unknown', t('nta.form.coolBal.none')], ['static', t('nta.form.coolBal.static')],
        ['dynamic', t('nta.form.coolBal.dynamic')], ['not_applicable', t('nta.form.coolBal.na')]]} />
      <SelectField {...field} path={at('emission', 'control')} label={t('nta.form.coolingControl')} options={[
        ['unknown_or_other', t('nta.form.coolCtrl.unknown')], ['standalone_per_room', t('nta.form.coolCtrl.standalone')],
        ['central_with_room_control', t('nta.form.coolCtrl.central')]]} />
      <NumberField {...field} path={at('emission', 'fanCoilCount')} label={t('nta.form.coolingFanCoils')} />
      <TextField {...field} path={at('emission', 'sourceReference')} label={t('nta.form.source')} />
      <label className="nta-form-check">
        <input type="checkbox" checked={read(draft, at('distribution')) != null}
          onChange={(event) => change(at('distribution'), event.target.checked
            ? { designTemperature: 't6_to12_or_unknown', pipe: { kind: 'insulated_before1980_or_unknown_age' }, fittingsInsulated: false, sourceReference: '' }
            : null)} />
        {t('nta.form.coolingWaterBased')}
      </label>
      {read(draft, at('distribution')) != null && <>
        <SelectField {...field} path={at('distribution', 'designTemperature')} label={t('nta.form.coolingDesign')} options={[
          ['t6_to12_or_unknown', '6/12'], ['t12_to16', '12/16'], ['t12_to18', '12/18'], ['t17_to21', '17/21']]} />
        <SelectField {...field} path={at('distribution', 'pipe', 'kind')} label={t('nta.form.coolingPipe')} options={[
          ['insulated_from1995', t('nta.form.coolPipe.from1995')], ['insulated1980_to1995', t('nta.form.coolPipe.1980')],
          ['insulated_before1980_or_unknown_age', t('nta.form.coolPipe.before1980')], ['uninsulated', t('nta.form.coolPipe.none')]]} />
        <CheckField {...field} path={at('distribution', 'fittingsInsulated')} label={t('nta.form.coolingFittings')} />
        <TextField {...field} path={at('distribution', 'sourceReference')} label={t('nta.form.source')} />
      </>}
    </>}
  </>;
}

/**
 * §10.2: one cooling system for the building (`cooling`) or several, each
 * with the calculation zones it serves (`coolingSystems`); the kernel
 * allows only one of the two.
 */
export function CoolingSystemsFields({ draft, change, project }: SectionProps & { project: IProject }) {
  const { t } = useI18n();
  const systems = list(draft, ['coolingSystems']);
  const several = systems.length > 0;
  const allZones = project.zones.map((zone) => zone.id);
  const taken = (except: number) => new Set(systems.flatMap((system, index) =>
    index === except ? [] : (Array.isArray(system.zoneIds) ? system.zoneIds as string[] : [])));
  const toggle = (on: boolean) => {
    if (on) {
      const single = read(draft, ['cooling']) as Draft | null | undefined;
      change(['cooling'], null);
      change(['coolingSystems'], [{ zoneIds: allZones, system: single ?? coolingSystemTemplate('compression') }]);
    } else {
      change(['coolingSystems'], []);
      change(['cooling'], (systems[0]?.system as Draft | undefined) ?? null);
    }
  };
  return <>
    {project.zones.length > 1 && <label className="nta-form-check">
      <input type="checkbox" checked={several} onChange={(event) => toggle(event.target.checked)} />
      {t('nta.form.coolingSystems.several')}
    </label>}
    {!several && <CoolingSystemFields draft={draft} change={change} base={['cooling']} allowNone />}
    {several && <div className="nta-form-subsection">
      {systems.map((system, index) => {
        const zoneIds = Array.isArray(system.zoneIds) ? system.zoneIds as string[] : [];
        const elsewhere = taken(index);
        return <fieldset key={index} className="nta-form-row">
          <legend>{t('nta.form.coolingSystems.system')} {index + 1}</legend>
          {project.zones.map((zone) => <label key={zone.id} className="nta-form-check">
            <input type="checkbox" checked={zoneIds.includes(zone.id)} disabled={elsewhere.has(zone.id)}
              onChange={(event) => change(['coolingSystems', index, 'zoneIds'], event.target.checked
                ? [...zoneIds, zone.id] : zoneIds.filter((id) => id !== zone.id))} />
            {zone.name || zone.id}
          </label>)}
          <CoolingSystemFields draft={draft} change={change} base={['coolingSystems', index, 'system']} allowNone={false} />
          <RemoveButton label={t('nta.form.remove')}
            onClick={() => change(['coolingSystems'], systems.filter((__, other) => other !== index))} />
        </fieldset>;
      })}
      <button type="button" className="btn" onClick={() => change(['coolingSystems'],
        [...systems, { zoneIds: [], system: coolingSystemTemplate('compression') }])}>
        {t('nta.form.coolingSystems.add')}
      </button>
    </div>}
  </>;
}

/** §13.7 solar systems for space heating without a hot-water system (SHS). */
export function SpaceHeatingSolarFields({ draft, change }: SectionProps) {
  return <SolarWaterHeaterFields draft={draft} change={change} base={['spaceHeatingSolar']} solarUse="space_heating" />;
}

/** 9.6.1: the whole building on one collective installation, and 5.31/5.32 heat-pump evidence. */
export function CollectiveAndRenewableFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const collective = read(draft, ['collectiveConnection']) != null;
  const renewable = read(draft, ['heatPumpRenewable']) != null;
  const combined = read(draft, ['heatPumpRenewable', 'combinedOutdoorAndExhaustAir']) === true;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={collective}
        onChange={(event) => change(['collectiveConnection'], event.target.checked ? collectiveConnectionTemplate() : null)} />
      {t('nta.form.collectiveConnection')}
    </label>
    {collective && <>
      <NumberField {...field} path={['collectiveConnection', 'connectedUsableAreaM2']} label={t('nta.form.collectiveConnection.area')} />
      <TextField {...field} path={['collectiveConnection', 'sourceReference']} label={t('nta.form.source')} />
    </>}
    <label className="nta-form-check">
      <input type="checkbox" checked={renewable}
        onChange={(event) => change(['heatPumpRenewable'], event.target.checked ? heatPumpRenewableTemplate() : null)} />
      {t('nta.form.heatPumpRenewable')}
    </label>
    {renewable && <>
      <CheckField {...field} path={['heatPumpRenewable', 'sourceBelow20C']} label={t('nta.form.heatPumpRenewable.below20')} />
      <CheckField {...field} path={['heatPumpRenewable', 'exhaustAirSource']} label={t('nta.form.heatPumpRenewable.exhaust')} />
      <label className="nta-form-check">
        <input type="checkbox" checked={combined} onChange={(event) => change(['heatPumpRenewable'], {
          ...(read(draft, ['heatPumpRenewable']) as Draft),
          combinedOutdoorAndExhaustAir: event.target.checked,
          // The kernel rejects a fraction without the combined source; hidden values are cleared.
          ...(event.target.checked ? {} : { outdoorAirHeatFraction: null, outdoorAirFractionReference: null }),
        })} />
        {t('nta.form.heatPumpRenewable.combined')}
      </label>
      {combined && <>
        <NumberField {...field} path={['heatPumpRenewable', 'outdoorAirHeatFraction']} label={t('nta.form.heatPumpRenewable.fraction')} />
        <TextField {...field} path={['heatPumpRenewable', 'outdoorAirFractionReference']} label={t('nta.form.heatPumpRenewable.fractionSource')} />
      </>}
      <TextField {...field} path={['heatPumpRenewable', 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

const SERVICES = ['domestic_hot_water', 'domestic_hot_water_auxiliary', 'ventilation_fans', 'space_cooling',
  'space_cooling_auxiliary', 'lighting', 'pv_auxiliary', 'humidification'] as const;

/** §5.5: uses determined outside the calculation, renewable hot-water heat and on-site production per month. */
export function DeclaredFlowsFields({ draft, change }: SectionProps) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const uses = list(draft, ['declaredUses']);
  const heat = list(draft, ['declaredRenewableHeat']);
  const production = list(draft, ['onSiteProduction']);
  const carriers: Array<[string, string]> = [['el', t('nta.form.carrier.el')], ['gas', t('nta.form.carrier.gas')], ['oil', t('nta.form.carrier.oil')]];
  return <div className="nta-form-subsection">
    {uses.map((_, index) => <fieldset key={`use-${index}`} className="nta-form-row">
      <legend>{t('nta.form.declaredUse')} {index + 1}</legend>
      <TextField {...field} path={['declaredUses', index, 'id']} label={t('nta.form.declared.id')} />
      <SelectField {...field} path={['declaredUses', index, 'service']} label={t('nta.form.declaredUse.service')}
        options={SERVICES.map((key) => [key, t(`nta.form.service.${key}`)])} />
      <SelectField {...field} path={['declaredUses', index, 'carrier']} label={t('nta.form.declaredUse.carrier')} options={carriers} />
      <MonthlyValues draft={draft} change={change} path={['declaredUses', index, 'monthlyKwh']} label={t('nta.form.declared.monthly')} />
      <TextField {...field} path={['declaredUses', index, 'sourceReference']} label={t('nta.form.source')} />
      <RemoveButton label={t('nta.form.remove')} onClick={() => change(['declaredUses'], uses.filter((__, other) => other !== index))} />
    </fieldset>)}
    <button type="button" className="btn" onClick={() => change(['declaredUses'], [...uses, declaredUseTemplate(uses.length)])}>
      {t('nta.form.declaredUse.add')}
    </button>
    {heat.map((_, index) => <fieldset key={`heat-${index}`} className="nta-form-row">
      <legend>{t('nta.form.declaredHeat')} {index + 1}</legend>
      <TextField {...field} path={['declaredRenewableHeat', index, 'id']} label={t('nta.form.declared.id')} />
      <MonthlyValues draft={draft} change={change} path={['declaredRenewableHeat', index, 'monthlyKwh']} label={t('nta.form.declared.monthly')} />
      <TextField {...field} path={['declaredRenewableHeat', index, 'sourceReference']} label={t('nta.form.source')} />
      <RemoveButton label={t('nta.form.remove')} onClick={() => change(['declaredRenewableHeat'], heat.filter((__, other) => other !== index))} />
    </fieldset>)}
    <button type="button" className="btn" onClick={() => change(['declaredRenewableHeat'], [...heat, declaredRenewableHeatTemplate(heat.length)])}>
      {t('nta.form.declaredHeat.add')}
    </button>
    {production.map((_, index) => <fieldset key={`prod-${index}`} className="nta-form-row">
      <legend>{t('nta.form.production')} {index + 1}</legend>
      <TextField {...field} path={['onSiteProduction', index, 'id']} label={t('nta.form.declared.id')} />
      <SelectField {...field} path={['onSiteProduction', index, 'kind']} label={t('nta.form.production.kind')} options={[
        ['pv', t('nta.form.production.pv')], ['pvt', t('nta.form.production.pvt')], ['wind', t('nta.form.production.wind')]]} />
      <MonthlyValues draft={draft} change={change} path={['onSiteProduction', index, 'monthlyKwh']} label={t('nta.form.declared.monthly')} />
      <TextField {...field} path={['onSiteProduction', index, 'sourceReference']} label={t('nta.form.source')} />
      <RemoveButton label={t('nta.form.remove')} onClick={() => change(['onSiteProduction'], production.filter((__, other) => other !== index))} />
    </fieldset>)}
    <button type="button" className="btn" onClick={() => change(['onSiteProduction'], [...production, onSiteProductionTemplate(production.length)])}>
      {t('nta.form.production.add')}
    </button>
  </div>;
}
