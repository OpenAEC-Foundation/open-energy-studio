import type { ReactNode } from 'react';
import { useI18n } from '../../i18n/i18n';
import type { IProject } from '../../core/energy/types';
import { CheckField, NumberField, read, SelectField, TextField, type Draft, type Path } from './NtaFormFields';
import { SunroomsFields } from './NtaProjectExtras';
import { utilityInternalGains } from '../../core/nta/NtaFormModels';

// Kernel input that only the JSON view reached before (form coverage,
// docs/ui-redesign/formulierdekking.md): per calculation zone its own use
// function, dwelling type, function areas, thermal mass and sunrooms
// (`zoneData`), a swimming pool in a sport ventilation function
// (§11.2.2.5.1) and the zone of the lighting. The collective shares are in
// NtaCollectiveShare.

type Change = (path: Path, value: unknown) => void;
interface SectionProps { draft: Draft; change: Change }

function list(draft: Draft, path: Path): Draft[] {
  const value = read(draft, path);
  return Array.isArray(value) ? value as Draft[] : [];
}

const USAGE_FUNCTIONS = ['residential', 'office', 'education', 'retail', 'other_assembly', 'assembly_child_care',
  'other_healthcare', 'healthcare_with_beds', 'lodging', 'cell', 'sport'];
/** Function areas (§6.5.3) split a utility zone; a residential part is refused (function_areas_residential_mixed). */
const FUNCTION_AREA_FUNCTIONS = USAGE_FUNCTIONS.filter((key) => key !== 'residential');

/** Whether any zone of the NTA block has a common space (6.6.4). */
export function hasCommonSpaces(draft: Draft): boolean {
  return list(draft, ['zoneData']).some((zone) => Array.isArray(zone.commonSpaces) && zone.commonSpaces.length > 0);
}

/**
 * With common spaces the kernel fills an empty hot-water or lighting function list with the within-zone split
 * (13.19; chapter 14, t_D, t_N and P_n;spec). Ticked: the list is empty and the entry fields are hidden; unticked:
 * one blank entry to fill in by hand (for instance Uniec's building split).
 */
export function CommonSpaceFillField({ draft, change, path, children, available = true }:
  SectionProps & { path: Path; children: ReactNode; /** False where the kernel does not fill (several hot-water systems, 13.20). */ available?: boolean }) {
  const { t } = useI18n();
  const value = read(draft, path);
  const filled = Array.isArray(value) && value.length === 0;
  if (!available || !hasCommonSpaces(draft)) return <>{children}</>;
  return <>
    <label className="nta-form-check">
      <input type="checkbox" checked={filled}
        onChange={(event) => change(path, event.target.checked ? [] : [{ function: null, areaM2: null }])} />
      {t('nta.zoneData.commonSpace.fill')}
    </label>
    {filled ? <p className="nta-form-note">{t('nta.zoneData.commonSpace.fillNote')}</p> : children}
  </>;
}

/**
 * Common spaces lying in a zone (6.6.4): the kernel allocates each one pro rata over the functions that rely on it,
 * within the zone only over the functions of the zone, for the building (Bbl limits) over all of them. With common
 * spaces the zone's function areas are its own (initial) areas. No function ticked: all functions of the building.
 */
export function CommonSpacesFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const spaces = list(draft, base);
  return <>
    {spaces.length > 0 && <p className="nta-form-note">{t('nta.zoneData.commonSpace.note')}</p>}
    {spaces.map((space, index) => {
      const served = Array.isArray(space.servedFunctions) ? space.servedFunctions as string[] : [];
      return <fieldset key={index} className="nta-form-row"><legend>{t('nta.zoneData.commonSpace.title')}</legend>
        <NumberField {...field} path={[...base, index, 'areaM2']} label={t('nta.zoneData.commonSpace.area')} unit="m²" />
        <span className="nta-form-note">{t('nta.zoneData.commonSpace.served')}</span>
        {/* A common space of a utility zone serves utility functions; a residential part is a separate calculation. */}
        {FUNCTION_AREA_FUNCTIONS.map((key) => <label key={key} className="nta-form-check">
          <input type="checkbox" checked={served.includes(key)} onChange={(event) => {
            const next = event.target.checked ? [...served, key] : served.filter((item) => item !== key);
            change([...base, index, 'servedFunctions'], next.length > 0 ? next : undefined);
          }} />
          {t(`nta.form.usage.${key}`)}
        </label>)}
        <TextField {...field} path={[...base, index, 'sourceReference']} label={t('nta.zoneData.commonSpace.source')} />
        <button type="button" className="nta-form-remove" onClick={() => {
          const rest = spaces.filter((__, other) => other !== index);
          change(base, rest.length > 0 ? rest : undefined);
        }}>{t('nta.form.remove')}</button>
      </fieldset>;
    })}
    <button type="button" onClick={() => change(base, [...spaces, { areaM2: null, sourceReference: '' }])}>
      {t('nta.zoneData.commonSpace.add')}</button>
  </>;
}

/** The ventilation functions of a block (`base`: `['ventilation']` or a zone's): a swimming pool in a sport function doubles q_usi;spec. */
export function SwimmingPoolFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const functions = list(draft, [...base, 'functions']);
  return <>{functions.map((item, index) => item.function === 'sport'
    ? <CheckField key={index} draft={draft} onChange={change} path={[...base, 'functions', index, 'swimmingPool']}
      label={t('nta.vent.swimmingPool')} />
    : null)}</>;
}

/** The calculation zone a lighting block belongs to (`lighting[i].zoneId`); only asked with several zones. */
export function LightingZoneField({ draft, change, project, index }: SectionProps & { project: IProject; index: number }) {
  const { t } = useI18n();
  if (project.zones.length < 2) return null;
  return <SelectField draft={draft} onChange={change} path={['lighting', index, 'zoneId']} label={t('nta.light.zone')}
    options={project.zones.map((zone) => [zone.id, zone.name ? `${zone.name} (${zone.id})` : zone.id])} />;
}

const copy = (value: unknown) => JSON.parse(JSON.stringify(value)) as unknown;

/**
 * A zone's own data starts as a copy of the block's internal gains, ventilation flows and chapter 11
 * ventilation: with a `zoneData` entry the kernel takes these from the zone only (no fallback to the
 * block), and every entry needs internal gains. The block's chapter 11 ventilation is mirrored on the
 * first zone (`syncVentilation`), so the copy gets this zone's id and floor area: the kernel requires
 * `ventilation.usableFloorAreaM2` to equal the zone's area (`ventilation_area_mismatch`).
 */
export function zoneDataTemplate(draft: Draft, zone: { id: string; floorArea?: number | null }): Draft {
  const gains = read(draft, ['internalGains']);
  const flows = read(draft, ['ventilationFlows']);
  const ventilation = read(draft, ['ventilation']) as Draft | null | undefined;
  const residential = read(draft, ['calculationScope']) !== 'utility';
  return {
    zoneId: zone.id,
    internalGains: gains != null && typeof gains === 'object' && 'method' in (gains as Draft) ? copy(gains)
      : residential ? { method: 'residential', dwellingCount: 1, sourceReference: '' } : utilityInternalGains(),
    ...(Array.isArray(flows) ? { ventilationFlows: copy(flows) } : {}),
    ...(ventilation != null ? { ventilation: zoneVentilation(ventilation, zone) } : {}),
  };
}

/** The block's chapter 11 ventilation for another zone: its id, its area, and function areas scaled to it. */
function zoneVentilation(ventilation: Draft, zone: { id: string; floorArea?: number | null }): Draft {
  const result = copy(ventilation) as Draft;
  const area = typeof zone.floorArea === 'number' ? zone.floorArea : null;
  const previous = typeof ventilation.usableFloorAreaM2 === 'number' ? ventilation.usableFloorAreaM2 : null;
  result.zoneId = zone.id;
  result.usableFloorAreaM2 = area;
  if (Array.isArray(result.functions)) {
    const functions = result.functions as Draft[];
    result.functions = functions.map((item) => ({
      ...item,
      areaM2: area == null ? null
        : functions.length === 1 || previous == null || previous <= 0 || typeof item.areaM2 !== 'number'
          ? area : item.areaM2 * area / previous,
    }));
  }
  return result;
}

/**
 * Per calculation zone (§6.3): its own use function, dwelling type, function areas (§6.5.3),
 * thermal mass (table 7.10) and sunrooms (7.30b), instead of the block's. With one zone the
 * block's data apply; with several, every zone needs its own entry (zone_data_missing).
 */
export function ZoneDataFields({ draft, change, project }: SectionProps & { project: IProject }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const zoneData = list(draft, ['zoneData']);
  // With one zone the block's data apply until the zone gets its own entry, which a utility zone needs for
  // several use functions (function areas) or a common space (6.6.4).
  const single = project.zones.length < 2 && zoneData.length === 0;
  return <div className="nta-form-subsection">
    <p className="nta-form-note">{t(single ? 'nta.zoneData.singleZone' : 'nta.zoneData.note')}</p>
    {project.zones.map((zone) => {
      const index = zoneData.findIndex((item) => String(item.zoneId) === zone.id);
      const title = zone.name ? `${zone.name} (${zone.id})` : zone.id;
      if (index < 0) {
        return <fieldset key={zone.id} className="nta-form-row"><legend>{title}</legend>
          <button type="button" onClick={() => change(['zoneData'], [...zoneData, zoneDataTemplate(draft, zone)])}>
            {t('nta.zoneData.add')}</button>
        </fieldset>;
      }
      const base: Path = ['zoneData', index];
      const usage = read(draft, [...base, 'usageFunction']) ?? read(draft, ['usageFunction']);
      const areas = list(draft, [...base, 'functionAreas']);
      const mass = read(draft, [...base, 'thermalMass']);
      return <fieldset key={zone.id} className="nta-form-row"><legend>{title}</legend>
        <SelectField {...field} path={[...base, 'usageFunction']} label={t('nta.zoneData.usageFunction')}
          options={USAGE_FUNCTIONS.map((key) => [key, t(`nta.form.usage.${key}`)])}
          onChange={(path, value) => {
            change(path, value ?? undefined);
            // With its own use function the zone no longer takes the block's dwelling type
            // (dwelling_type_required / dwelling_type_not_applicable).
            const own = read(draft, [...base, 'dwellingType']);
            if (value === 'residential') {
              if (own == null) change([...base, 'dwellingType'], read(draft, ['dwellingType']) ?? undefined);
            } else if (own != null) change([...base, 'dwellingType'], undefined);
            // A residential zone has no function areas (function_areas_residential_mixed) and no form for
            // common spaces, so none are kept out of sight.
            if (value === 'residential' && areas.length > 0) change([...base, 'functionAreas'], undefined);
            if (value === 'residential' && read(draft, [...base, 'commonSpaces']) != null) change([...base, 'commonSpaces'], undefined);
          }} />
        {usage === 'residential' && <SelectField {...field} path={[...base, 'dwellingType']} label={t('nta.zoneData.dwellingType')}
          options={[['apartment_building', t('nta.form.dwellingType.apartment')], ['other', t('nta.form.dwellingType.other')]]}
          onChange={(path, value) => change(path, value ?? undefined)} />}
        {areas.map((_, area) => <div key={area} className="nta-form-row">
          <SelectField {...field} path={[...base, 'functionAreas', area, 'function']} label={t('nta.zoneData.functionArea.function')}
            options={FUNCTION_AREA_FUNCTIONS.map((key) => [key, t(`nta.form.usage.${key}`)])} />
          <NumberField {...field} path={[...base, 'functionAreas', area, 'areaM2']} label={t('nta.zoneData.functionArea.area')} unit="m²" />
          <button type="button" className="nta-form-remove"
            onClick={() => change([...base, 'functionAreas'], areas.filter((__, other) => other !== area))}>{t('nta.form.remove')}</button>
        </div>)}
        {usage !== 'residential' && <button type="button" onClick={() => change([...base, 'functionAreas'],
          [...areas, { function: typeof usage === 'string' && usage !== 'residential' ? usage : 'office', areaM2: null }])}>
          {t('nta.zoneData.functionArea.add')}</button>}
        {usage !== 'residential' && <CommonSpacesFields draft={draft} change={change} base={[...base, 'commonSpaces']} />}
        <label className="nta-form-check">
          <input type="checkbox" checked={mass != null} onChange={(event) => change([...base, 'thermalMass'], event.target.checked
            ? { floor: 'heavy', wall: 'heavy', ceiling: 'open_or_none', sourceReference: '' } : undefined)} />
          {t('nta.zoneData.thermalMass')}
        </label>
        {mass != null && <>
          {(['floor', 'wall'] as const).map((part) => <SelectField key={part} {...field} path={[...base, 'thermalMass', part]}
            label={t(`nta.form.mass.${part}`)} options={[['light', t('nta.form.mass.light')], ['heavy', t('nta.form.mass.heavy')], ['very_heavy', t('nta.form.mass.veryHeavy')]]} />)}
          <SelectField {...field} path={[...base, 'thermalMass', 'ceiling']} label={t('nta.form.mass.ceiling')}
            options={[['open_or_none', t('nta.form.mass.open')], ['closed_or_suspended', t('nta.form.mass.closed')]]} />
          <TextField {...field} path={[...base, 'thermalMass', 'sourceReference']} label={t('nta.zoneData.thermalMass.source')} />
        </>}
        <SunroomsFields draft={draft} change={change} base={[...base, 'sunrooms']} project={project} />
        {read(draft, [...base, 'ventilation']) != null &&
          <SwimmingPoolFields draft={draft} change={change} base={[...base, 'ventilation']} />}
        <button type="button" className="nta-form-remove"
          onClick={() => {
            // With several zones every zone needs its own data (zone_data_missing).
            if (project.zones.length > 1 && typeof window !== 'undefined' && typeof window.confirm === 'function'
              && !window.confirm(t('nta.zoneData.removeConfirm'))) return;
            change(['zoneData'], zoneData.filter((__, other) => other !== index));
          }}>{t('nta.zoneData.remove')}</button>
      </fieldset>;
    })}
  </div>;
}
