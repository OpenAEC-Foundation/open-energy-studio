/**
 * The NTA input as a register of sections (ontwerp §6.1, F6).
 *
 * Every section of the former single form is one entry with the workflow step,
 * sub page and (for heating) stepper part where it is edited. The full form
 * (`NtaCalculationForm`, Controle › NTA-invoer) renders all entries in this
 * order; the step pages render the entries of their own page. Fields and paths
 * are unchanged.
 */
import { hasEdition2022Routes } from './NtaEdition2022Fields';
import type { ComponentType } from 'react';
import { DEFAULT_NORM_VERSION, IMPLEMENTED_NORM_VERSIONS } from '../../core/nta/KernelClient';
import type { IProject } from '../../core/energy/types';
import type { StepId } from '../../core/navigation/routes';
import { HeatingEmission2023Fields } from './NtaEdition2023Fields';
import { useI18n } from '../../i18n/i18n';
import {
  CheckField, NumberField, read, SelectField, TextField, TriStateField, useFieldPath, type Draft, type Path,
} from './NtaFormFields';
import { PvSystemFields } from './NtaPvFields';
import {
  AirHeatersFields, BacsFields, BBL_FUNCTIONS, FunctionAreasFields, LABEL_FUNCTIONS,
} from './NtaAdvancedSections';
import { NtaVentilationSection } from './NtaVentilationSection';
import { DynamicWindowsFields } from './NtaDynamicWindows';
import { WindowObstructionsFields } from './NtaWindowObstructions';
import { AnnexAaCalculationFields } from './NtaAnnexAaFields';
import { annexAaCalculations, annexAaEditionRules } from '../../core/nta/annexAaForm';
import { DeclaredHeatingTableTool, GroundFloorDetailFields } from './NtaProductGenerators';
import { NtaDistributionFields, NtaLightingSection, NtaUtilityGainsFields } from './NtaExtraSections';
import { ExternalSupplyFields } from './NtaExternalSupply';
import {
  AdditionalHeatingSystemsFields, AdditionalHotWaterSystemsFields, HotWaterGeneratorFields, KitchenPipeDiameterField, HotWaterGeneratorsFields, HotWaterStorageFields, SolarWaterHeaterFields,
  SpaceGeneratorFields, WindowObstructionFields,
} from './NtaSystemSections';
import {
  buildVentilationDraft, removeVentilation, syncVentilation, utilityInternalGains,
} from '../../core/nta/NtaFormModels';
import { bacsTemplate } from '../../core/nta/NtaSystemTemplates';
import {
  CollectiveAndRenewableFields, CoolingSystemsFields, DeclaredFlowsFields, HumidifiersFields, SpaceHeatingSolarFields, SunroomsFields,
} from './NtaProjectExtras';

// The block is edited as plain JSON data; the Rust kernel is the validator.

/** ρ_a·c_a/3600 with 1,205 kg/m³ and 1 005 J/(kg·K) (9.29), in W per (m³/h)·K. */
const AIR_HEAT_CAPACITY_W_PER_M3H_K = (1.205 * 1005) / 3600;
/** Usage functions of tables 7.13–7.15. */
const USAGE_FUNCTIONS = ['residential', 'office', 'education', 'retail', 'other_assembly', 'assembly_child_care',
  'other_healthcare', 'healthcare_with_beds', 'lodging', 'cell', 'sport'] as const;

/**
 * 7.3.3 vertical pipes: unknown (null, a kernel gap), none ([]) or a list.
 * Dwellings give the storeys of the zone, utility buildings the building
 * height H of 7.17a.
 */
export function VerticalPipesFields({ draft, change, residential, path, label }: {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
  residential: boolean;
  path: Path;
  label?: string;
}) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const pipes = read(draft, path);
  const kernelPath = useFieldPath(path);
  const list = Array.isArray(pipes) ? pipes as Draft[] : null;
  const newPipe = (index: number): Draft => ({
    id: `leiding-${index + 1}`,
    ...(residential ? { storeys: null } : { buildingHeightM: null }),
    insulated: false,
    sourceReference: '',
  });
  return <>
    <label data-path={kernelPath ?? undefined}>{label ? `${label} — ${t('nta.form.verticalPipes.state')}` : t('nta.form.verticalPipes.state')}
      <select value={list == null ? 'unknown' : list.length === 0 ? 'none' : 'listed'} onChange={(event) => change(path,
        event.target.value === 'unknown' ? null : event.target.value === 'none' ? [] : [newPipe(0)])}>
        <option value="unknown">{t('nta.form.verticalPipes.unknown')}</option>
        <option value="none">{t('nta.form.verticalPipes.none')}</option>
        <option value="listed">{t('nta.form.verticalPipes.listed')}</option>
      </select>
    </label>
    {list?.map((_, index) => <div key={index} className="nta-form-row">
      {residential
        ? <NumberField {...field} path={[...path, index, 'storeys']} label={t('nta.form.verticalPipes.storeys')} step="1" />
        : <NumberField {...field} path={[...path, index, 'buildingHeightM']} label={t('nta.form.verticalPipes.height')} />}
      <CheckField {...field} path={[...path, index, 'insulated']} label={t('nta.form.verticalPipes.insulated')} />
      <NumberField {...field} path={[...path, index, 'sharedZones']} label={t('nta.form.verticalPipes.shared')} step="1" />
      <TextField {...field} path={[...path, index, 'sourceReference']} label={t('nta.form.source')} />
      <button type="button" onClick={() => change(path, list.filter((_, other) => other !== index))}>
        {t('nta.form.verticalPipes.remove')}</button>
    </div>)}
    {list != null && list.length > 0 &&
      <button type="button" onClick={() => change(path, [...list, newPipe(list.length)])}>{t('nta.form.verticalPipes.add')}</button>}
  </>;
}

/** 8.36: the ψ_gr;j and ℓ_j of the floor-edge bridges (§8.3.6). */
export function GroundEdgeBridgesFields({ draft, change, base }: {
  draft: Draft; change: (path: Path, value: unknown) => void; base: Path;
}) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const bridges = (read(draft, base) as Draft[] | undefined) ?? [];
  return <>
    {bridges.map((_, bridge) => <div key={bridge} className="nta-form-row">
      <NumberField {...field} path={[...base, bridge, 'lengthM']} label={`${t('nta.form.edgeBridges.length')} ${bridge + 1}`} />
      <NumberField {...field} path={[...base, bridge, 'psiWPerMk']} label={t('nta.form.edgeBridges.psi')} />
      <TextField {...field} path={[...base, bridge, 'sourceReference']} label={t('nta.form.source')} />
      <button type="button" className="nta-form-remove" onClick={() => change(base, bridges.filter((__, other) => other !== bridge))}>
        {t('nta.form.remove')}
      </button>
    </div>)}
    <button type="button" onClick={() => change(base, [...bridges, { lengthM: null, psiWPerMk: null, sourceReference: '' }])}>
      {t('nta.form.edgeBridges.add')}
    </button>
  </>;
}

/** Table 7.13 θ_int;set;H;stc and θ_int;set;C;stc for a usage function, °C. */
export function table713Setpoints(usageFunction: unknown): { heatingC: number; coolingC: number } | null {
  if (typeof usageFunction !== 'string' || usageFunction === '') return null;
  const heatingC = usageFunction === 'healthcare_with_beds' ? 22
    : usageFunction === 'sport' ? 16
      : usageFunction === 'residential' ? 20 : 21;
  return { heatingC, coolingC: 24 };
}

/** One setpoint check: the zone (null for the block), the table value and where its setpoints live. */
export interface SetpointCheckRow {
  zoneId: string | null;
  /** Index in `zoneData`, null for the block or a zone without a `zoneData` entry. */
  zoneIndex: number | null;
  expected: { heatingC: number; coolingC: number };
  actual: { heatingC: unknown; coolingC: unknown };
  /** Path of the setpoints this zone uses: its own or the block's. */
  path: Path;
  ownSetpoints: boolean;
}

/** §6.5.3 profile setpoints: the table 7.13 values, area-weighted over `functionAreas` when given. */
function profileSetpoints(usageFunction: unknown, functionAreas: unknown): { heatingC: number; coolingC: number } | null {
  if (Array.isArray(functionAreas) && functionAreas.length > 0) {
    let total = 0;
    let heating = 0;
    let cooling = 0;
    for (const part of functionAreas as Draft[]) {
      const values = table713Setpoints(part?.function);
      const area = Number(part?.areaM2);
      if (!values || !Number.isFinite(area) || area <= 0) return null;
      total += area;
      heating += values.heatingC * area;
      cooling += values.coolingC * area;
    }
    return total > 0 ? { heatingC: heating / total, coolingC: cooling / total } : null;
  }
  return table713Setpoints(usageFunction);
}

/**
 * The checks the kernel makes (monthly_demand.rs `function_profile`, setpoints_table_7_13_mismatch).
 * Like the kernel it walks the calculation zones of the project (`zoneIds`): each zone with its own
 * `zoneData` function or function areas, else the block's, against the setpoints it uses (its own
 * or the block's). Without zones it checks the block. The project route has no usage fit
 * (annex Z belongs to the maatwerkadvies), so table 7.13 always applies here.
 */
export function setpointChecks(draft: Draft, zoneIds?: string[]): SetpointCheckRow[] {
  const blockFunction = read(draft, ['usageFunction']);
  const zoneData = (read(draft, ['zoneData']) as Draft[] | undefined) ?? [];
  const ids = zoneIds ?? zoneData.map((zone, index) => String(zone?.zoneId ?? index + 1));
  const rows: SetpointCheckRow[] = [];
  const add = (zoneId: string | null, zoneIndex: number | null, expected: SetpointCheckRow['expected'] | null,
    path: Path, ownSetpoints: boolean) => {
    if (!expected) return;
    rows.push({
      zoneId, zoneIndex, expected, path, ownSetpoints,
      actual: { heatingC: read(draft, [...path, 'heatingC']), coolingC: read(draft, [...path, 'coolingC']) },
    });
  };
  if (ids.length === 0) {
    add(null, null, profileSetpoints(blockFunction, read(draft, ['functionAreas'])), ['setpoints'], true);
    return rows;
  }
  ids.forEach((zoneId) => {
    const index = zoneData.findIndex((zone) => String(zone?.zoneId) === zoneId);
    const zone = index >= 0 ? zoneData[index] : undefined;
    const own = zone?.setpoints != null;
    add(zoneId, index >= 0 ? index : null, profileSetpoints(zone?.usageFunction ?? blockFunction, zone?.functionAreas),
      own ? ['zoneData', index, 'setpoints'] : ['setpoints'], own);
  });
  return rows;
}

const close = (value: unknown, expected: number) => typeof value === 'number' && Math.abs(value - expected) <= 1e-9;
const tableValue = (value: number) => Math.round(value * 1000) / 1000;
/** Source of setpoints written from table 7.13 when the block has no reference of its own. */
export const TABLE_713_SOURCE = 'NTA 8800 tabel 7.13';

/** The kernel refuses setpoints other than table 7.13 (setpoints_table_7_13_mismatch); say so before saving. */
export function SetpointCheck({ draft, change, zoneIds }: { draft: Draft; change: (path: Path, value: unknown) => void; zoneIds: string[] }) {
  const { t } = useI18n();
  const rows = setpointChecks(draft, zoneIds);
  if (rows.length === 0) return <p className="nta-form-note">{t('nta.form.setpointsTableNote')}</p>;
  // Zones that share the block setpoints but need different table values cannot all match the block.
  const sharedTargets = new Set(rows.filter((row) => !row.ownSetpoints)
    .map((row) => `${tableValue(row.expected.heatingC)}/${tableValue(row.expected.coolingC)}`));
  const blockSource = read(draft, ['setpoints', 'sourceReference']);
  const source = typeof blockSource === 'string' && blockSource.trim() !== '' ? blockSource : TABLE_713_SOURCE;
  return <div data-testid="nta-setpoint-check">{rows.map((row) => {
    const matches = close(row.actual.heatingC, row.expected.heatingC) && close(row.actual.coolingC, row.expected.coolingC);
    const zoneTarget = !row.ownSetpoints && sharedTargets.size > 1 && row.zoneId != null;
    const message = t(matches ? 'nta.form.setpointsTableMatch' : 'nta.form.setpointsTableMismatch',
      { heating: tableValue(row.expected.heatingC), cooling: tableValue(row.expected.coolingC) });
    return <p key={row.zoneId ?? 'block'} className={matches ? 'nta-form-note' : 'nta-form-note nta-form-error'}
      role={matches ? undefined : 'alert'}>
      {row.zoneId != null && <strong>{t('nta.form.setpointsZone', { zone: row.zoneId })}: </strong>}
      {message}
      {!matches && <> <button type="button" onClick={() => change(...setpointWriteBack(draft, row, zoneTarget, source))}>
        {t('nta.form.setpointsUseTable')}</button></>}
    </p>;
  })}</div>;
}

/**
 * Where and what "use table values" writes: the setpoints the zone uses, or, when zones sharing the
 * block need different values, the zone's own setpoints (a new `zoneData` entry when it has none).
 * The source reference is kept, else the block's, else table 7.13 itself: the kernel refuses an
 * empty one (check_reference).
 */
export function setpointWriteBack(draft: Draft, row: SetpointCheckRow, zoneTarget: boolean, source: string): [Path, unknown] {
  const values = { heatingC: row.expected.heatingC, coolingC: row.expected.coolingC };
  if (zoneTarget && row.zoneIndex == null) {
    const zoneData = (read(draft, ['zoneData']) as Draft[] | undefined) ?? [];
    return [['zoneData'], [...zoneData, { zoneId: row.zoneId, setpoints: { sourceReference: source, ...values } }]];
  }
  const target: Path = zoneTarget ? ['zoneData', row.zoneIndex as number, 'setpoints'] : row.path;
  const current = (read(draft, target) as Draft | undefined) ?? {};
  const own = typeof current.sourceReference === 'string' && current.sourceReference.trim() !== '' ? current.sourceReference : source;
  return [target, { ...current, sourceReference: own, ...values }];
}

// ── Register ────────────────────────────────────────────────────────

export interface NtaSectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
  /** Functional update of the whole draft. */
  update: (next: (current: Draft) => Draft) => void;
  project: IProject;
}

/** Parts of the heating page (stepper, mockup 04). */
export const HEATING_PARTS = ['generation', 'distribution', 'emission', 'control', 'auxiliary', 'solar'] as const;
export type HeatingPart = (typeof HEATING_PARTS)[number];

export interface NtaSectionDef {
  id: string;
  step: StepId;
  sub?: string;
  /** Stepper part on the heating page. */
  part?: HeatingPart;
  titleKey: string;
  /** Paths in `ntaCalculation` the section edits: its findings, "Ga naar" and "Bron & bewijs". */
  paths: string[];
  /** Behind "Geavanceerd" in the Basis view. */
  advanced?: boolean;
  /** The section renders its own fieldsets (chapter 11, lighting). */
  bare?: boolean;
  when?: (project: IProject, draft: Draft) => boolean;
  Component: ComponentType<NtaSectionProps>;
}

const residentialScope = (draft: Draft) => read(draft, ['calculationScope']) === 'residential';
const fieldOf = (props: NtaSectionProps) => ({ draft: props.draft, onChange: props.change });

function GeneralSection(props: NtaSectionProps) {
  const { t, locale } = useI18n();
  const { draft } = props;
  const f = fieldOf(props);
  return <>
    <SelectField {...f} path={['normVersion']} label={t('nta.form.normVersion')}
      options={IMPLEMENTED_NORM_VERSIONS.map((edition) => [edition, t(`nta.edition.${edition}`)])} />
    {read(draft, ['normVersion']) != null && read(draft, ['normVersion']) !== DEFAULT_NORM_VERSION
      && <p className="nta-form-note" role="note">{t('nta.form.normVersionLegacy')}</p>}
    <SelectField {...f} path={['calculationScope']} label={t('nta.form.scope')}
      options={[['residential', t('nta.form.scope.residential')], ['utility', t('nta.form.scope.utility')]]} />
    <TextField {...f} path={['areaSourceReference']} label={t('nta.form.areaSource')} />
    <SelectField {...f} path={['usageFunction']} label={t('nta.form.usageFunction')}
      options={USAGE_FUNCTIONS.map((key) => [key, t(`nta.form.usage.${key}`)])} />
    {read(draft, ['usageFunction']) === 'residential' && <SelectField {...f} path={['dwellingType']}
      label={t('nta.form.dwellingType')} options={[
        ['apartment_building', t('nta.form.dwellingType.apartment')], ['other', t('nta.form.dwellingType.other')]]} />}
    <SelectField {...f} path={['bblFunction']} label={t('nta.form.bblFunction')} options={[
      ['other_residential', t('nta.form.bbl.other_residential')], ['residential_building', t('nta.form.bbl.residential_building')],
      ['office', t('nta.form.bbl.office')], ['education', t('nta.form.bbl.education')], ['retail', t('nta.form.bbl.retail')],
      ['other_assembly', t('nta.form.bbl.other_assembly')], ['other_healthcare', t('nta.form.bbl.other_healthcare')],
      ['sport', t('nta.form.bbl.sport')], ['other_lodging', t('nta.form.bbl.other_lodging')],
    ]} />
    <SelectField {...f} path={['zebHeatDeliveryTemperature']}
      label={locale === 'nl' ? 'Aflevertemperatuur externe warmte voor ZEB (bijlage AB)' : 'External heat delivery temperature for ZEB (Annex AB)'}
      options={[
        ['at_least60', '≥ 60 °C'], ['from40_to60', '40–<60 °C'], ['from20_to40', '20–<40 °C'],
      ]} />
    <p className="nta-form-note">{locale === 'nl'
      ? 'Alleen voor de informatieve ZEB-berekening. Leeg gebruikt de normaanname ≥ 60 °C; leg een lagere klasse alleen vast met bronbewijs.'
      : 'For the informative ZEB calculation only. Blank uses the standard assumption ≥ 60 °C; record a lower class only with source evidence.'}</p>
    <CheckField {...f} path={['permitApplicationAfter20260529']} label={t('nta.form.permitAfter')} />
    <NumberField {...f} path={['constructionYear']} label={t('nta.form.constructionYear')} step="1" />
    <TriStateField {...f} path={['fossilAppliancesOutsideCalculation']} label={t('nta.form.fossilOutside')}
      yes={t('nta.form.yes')} no={t('nta.form.no')} />
  </>;
}

/**
 * Annex AA inputs that only the 2024 edition reads (INT-V1:2024): the effective
 * mass per m² and the roof area per room. The rest of the annex AA calculation
 * is entered in `AnnexAaCalculationFields`.
 */
export function AnnexAa2024Fields(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft } = props;
  const f = fieldOf(props);
  const rules = annexAaEditionRules(read(draft, ['normVersion']));
  const calculations = annexAaCalculations(read(draft, ['activeCooling', 'capacity']));
  if (!rules.monthly2024) {
    // Values left behind by a switch from the 2024 edition: offer to remove them.
    const stale = calculations.some(({ value }) => value.effectiveMassKgPerM2 != null
      || ((value.rooms as Draft[] | undefined) ?? []).some((room) => room?.roofAreaM2 != null));
    if (!stale) return null;
    return <p className="nta-form-note nta-form-error" role="alert">
      {t('ntaStep.staleEdition', { field: t('ntaStep.annexAa.fields') })}{' '}
      <button type="button" onClick={() => props.update((current) => {
        const clone = structuredClone(current) as Draft;
        for (const { path } of annexAaCalculations(read(clone, ['activeCooling', 'capacity']))) {
          const target = read(clone, path) as Draft;
          delete target.effectiveMassKgPerM2;
          for (const room of (target.rooms as Draft[] | undefined) ?? []) delete room.roofAreaM2;
        }
        return clone;
      })}>{t('nta.form.remove')}</button>
    </p>;
  }
  // Older editions build on the 2024 method but have no annex AA route; the
  // calculation form already says so.
  if (!rules.route) return null;
  // Without a calculation, `AnnexAaCalculationFields` offers to add one.
  if (calculations.length === 0) return null;
  return <div className="nta-form-row" data-testid="nta-annex-aa-2024">
    {calculations.map(({ path: base, zoneId, value }) => {
      const rooms = (value.rooms as Draft[] | undefined) ?? [];
      return <div key={base.join('.')} className="nta-form-row">
        {zoneId != null && <strong>{t('ntaStep.annexAa.zone', { zone: zoneId })}</strong>}
        <NumberField {...f} path={[...base, 'effectiveMassKgPerM2']} label={t('ntaStep.annexAa.effectiveMass')} optional />
        {rooms.map((room, index) => <NumberField key={String(room?.id ?? index)} {...f} path={[...base, 'rooms', index, 'roofAreaM2']}
          label={t('ntaStep.annexAa.roofArea', { room: String(room?.id ?? index + 1) })} optional />)}
      </div>;
    })}
    <p className="nta-form-note">{t('ntaStep.annexAa.note')}</p>
  </div>;
}

function ActiveCoolingSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change } = props;
  const f = fieldOf(props);
  return <>
    <SelectField {...f} path={['activeCooling', 'system']} label={t('nta.form.activeCooling')} options={[
      ['compression_table10_29', t('nta.form.ac.compression')], ['absorption_table10_30', t('nta.form.ac.absorption')],
      ['free_cooling_table10_34', t('nta.form.ac.free')], ['dew_point_cooling_humidified_exhaust', t('nta.form.ac.dewPoint')],
      ['heat_pump_with_cooling_emitter', t('nta.form.ac.heatPump')], ['external_cold_with_cooling_emitter', t('nta.form.ac.external')],
      ['split_units_in_every_habitable_room', t('nta.form.ac.split')], ['other_utility', t('nta.form.ac.other')]]}
      onChange={(path, value) => value == null
        ? change(['activeCooling'], null)
        : read(draft, ['activeCooling']) == null
          ? change(['activeCooling'], { system: value, capacity: { method: 'dynamic_cooling_load', sourceReference: '' }, sourceReference: '' })
          : change(path, value)} />
    {read(draft, ['activeCooling']) != null && <>
      <SelectField {...f} path={['activeCooling', 'capacity', 'method']} label={t('nta.form.ac.capacity')} options={[
        ['dynamic_cooling_load', t('nta.form.ac.dynamic')], ['annex_aa', t('nta.form.ac.annexAa')], ['solar_limitation', t('nta.form.ac.solar')]]} />
      {read(draft, ['activeCooling', 'capacity', 'method']) === 'solar_limitation' &&
        <SelectField {...f} path={['activeCooling', 'capacity', 'criterion']} label={t('nta.form.ac.criterion')} options={[
          ['small_window_area', t('nta.form.ac.smallWindows')], ['shaded_glazing', t('nta.form.ac.shaded')]]} />}
      {read(draft, ['activeCooling', 'capacity', 'method']) === 'annex_aa' && <>
        <AnnexAaCalculationFields draft={draft} change={change} project={props.project} />
        <AnnexAa2024Fields {...props} />
      </>}
      <TextField {...f} path={['activeCooling', 'capacity', 'sourceReference']} label={t('nta.form.source')} />
      <TextField {...f} path={['activeCooling', 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

function FunctionsSection({ draft, change }: NtaSectionProps) {
  return <>
    <FunctionAreasFields draft={draft} change={change} base="labelFunctions" functions={LABEL_FUNCTIONS} prefix="nta.form.labelFn" />
    <FunctionAreasFields draft={draft} change={change} base="bblFunctions" functions={BBL_FUNCTIONS} prefix="nta.form.bblFn" />
  </>;
}

function SetpointsSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const f = fieldOf(props);
  return <>
    <NumberField {...f} path={['setpoints', 'heatingC']} label={t('nta.form.heatingSetpoint')} />
    <NumberField {...f} path={['setpoints', 'coolingC']} label={t('nta.form.coolingSetpoint')} />
    <TextField {...f} path={['setpoints', 'sourceReference']} label={t('nta.form.source')} />
    <SetpointCheck draft={props.draft} change={props.change} zoneIds={props.project.zones.map((zone) => zone.id)} />
  </>;
}

function MassSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const f = fieldOf(props);
  return <>
    {(['floor', 'wall'] as const).map((part) => <SelectField key={part} {...f} path={['thermalMass', part]}
      label={t(`nta.form.mass.${part}`)} options={[['light', t('nta.form.mass.light')], ['heavy', t('nta.form.mass.heavy')], ['very_heavy', t('nta.form.mass.veryHeavy')]]} />)}
    <SelectField {...f} path={['thermalMass', 'ceiling']} label={t('nta.form.mass.ceiling')}
      options={[['open_or_none', t('nta.form.mass.open')], ['closed_or_suspended', t('nta.form.mass.closed')]]} />
    {hasEdition2022Routes(props.draft) || read(props.draft, ['thermalMass', 'massKgPerM2']) != null
      ? <NumberField {...f} path={['thermalMass', 'massKgPerM2']} label={t('ntaStep.massKgPerM2')} />
      : null}
    <TextField {...f} path={['thermalMass', 'sourceReference']} label={t('nta.form.source')} />
  </>;
}

function InternalGainsSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change } = props;
  const f = fieldOf(props);
  return <>
    {residentialScope(draft)
      ? <NumberField {...f} path={['internalGains', 'dwellingCount']} label={t('nta.form.dwellingCount')} step="1" />
      : <>
        <SelectField {...f} path={['internalGains', 'method']} label={t('nta.gains.method')}
          options={[['utility', t('nta.gains.method.utility')], ['declared', t('nta.gains.method.declared')]]}
          onChange={(_, value) => change(['internalGains'], value === 'utility'
            ? { ...utilityInternalGains(), sourceReference: read(draft, ['internalGains', 'sourceReference']) ?? '' }
            : { method: 'declared', heatFluxWPerM2: null, sourceReference: read(draft, ['internalGains', 'sourceReference']) ?? '' })} />
        {read(draft, ['internalGains', 'method']) === 'utility'
          ? <NtaUtilityGainsFields draft={draft} change={change} />
          : <NumberField {...f} path={['internalGains', 'heatFluxWPerM2']} label={t('nta.form.heatFlux')} />}
      </>}
    <TextField {...f} path={['internalGains', 'sourceReference']} label={t('nta.form.source')} />
  </>;
}

function WindowsSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change } = props;
  const f = fieldOf(props);
  return <>
    <NumberField {...f} path={['windowSolar', 'frameFraction']} label={t('nta.form.frameFraction')} />
    <WindowObstructionFields draft={draft} change={change} />
    <label data-path="ntaCalculation.windowSolar.movableShading">{t('nta.form.shading')}
      <select value={String(read(draft, ['windowSolar', 'movableShading', 'control']) ?? '')} onChange={(event) => change(
        ['windowSolar', 'movableShading'], event.target.value
          ? { reductionFactor: read(draft, ['windowSolar', 'movableShading', 'reductionFactor']) ?? null,
            control: event.target.value, sourceReference: read(draft, ['windowSolar', 'movableShading', 'sourceReference']) ?? '' }
          : null)}>
        <option value="">{t('nta.form.shading.none')}</option>
        <option value="manual_residential">{t('nta.form.shading.manual')}</option>
        <option value="automatic_residential_iso52016">{t('nta.form.shading.automaticResidential')}</option>
        <option value="automatic">{t('nta.form.shading.automatic')}</option>
        <option value="manual_utility_with_glare_protection">{t('nta.form.shading.manualUtilityGlare')}</option>
        <option value="manual_utility_without_glare_protection">{t('nta.form.shading.manualUtility')}</option>
      </select>
    </label>
    {read(draft, ['windowSolar', 'movableShading']) != null && <>
      <NumberField {...f} path={['windowSolar', 'movableShading', 'reductionFactor']} label={t('nta.form.shadingFc')} />
      <TextField {...f} path={['windowSolar', 'movableShading', 'sourceReference']} label={t('nta.form.source')} />
    </>}
    <TextField {...f} path={['windowSolar', 'sourceReference']} label={t('nta.form.source')} />
  </>;
}

const surfaceNameOf = (project: IProject) => {
  const surfaces = project.zones.flatMap((zone) => zone.surfaces);
  return (id: unknown) => surfaces.find((surface) => surface.id === id)?.name || String(id);
};

function RoofTiltsSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const f = fieldOf(props);
  const tilts = (read(props.draft, ['surfaceTilts']) as Draft[] | undefined) ?? [];
  const surfaceName = surfaceNameOf(props.project);
  return <>
    {tilts.map((item, index) => <div key={String(item.surfaceId)} className="nta-form-row">
      <NumberField {...f} path={['surfaceTilts', index, 'tiltDeg']} label={`${surfaceName(item.surfaceId)} — ${t('nta.form.tilt')}`} />
      <TextField {...f} path={['surfaceTilts', index, 'sourceReference']} label={t('nta.form.source')} />
    </div>)}
  </>;
}

function GroundFloorsSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change } = props;
  const f = fieldOf(props);
  const ground = (read(draft, ['groundFloors']) as Draft[] | undefined) ?? [];
  const surfaceName = surfaceNameOf(props.project);
  return <>
    {ground.map((item, index) => <div key={String(item.surfaceId)} className="nta-form-row">
      <NumberField {...f} path={['groundFloors', index, 'exposedPerimeterM']} label={`${surfaceName(item.surfaceId)} — ${t('nta.form.perimeter')}`} />
      <NumberField {...f} path={['groundFloors', index, 'constructionResistanceM2kPerW']} label={t('nta.form.floorResistance')} />
      <label data-path={`ntaCalculation.groundFloors[${index}].edgeThermalBridges`}>{t('nta.form.edgeBridges')}
        <select value={String(read(draft, ['groundFloors', index, 'edgeThermalBridges', 'method']) ?? '')} onChange={(event) => change(
          ['groundFloors', index, 'edgeThermalBridges'], event.target.value === 'forfait'
            ? { method: 'forfait' }
            : event.target.value === 'detailed' ? { method: 'detailed', bridges: [] } : null)}>
          <option value="">—</option>
          <option value="forfait">{t('nta.form.edgeBridges.forfait')}</option>
          <option value="detailed">{t('nta.form.edgeBridges.detailed')}</option>
        </select>
      </label>
      {read(draft, ['groundFloors', index, 'edgeThermalBridges', 'method']) === 'detailed' &&
        <GroundEdgeBridgesFields draft={draft} change={change} base={['groundFloors', index, 'edgeThermalBridges', 'bridges']} />}
      <GroundFloorDetailFields draft={draft} change={change} base={['groundFloors', index]} />
      <TextField {...f} path={['groundFloors', index, 'sourceReference']} label={t('nta.form.source')} />
    </div>)}
    {ground.some((item) => read(item, ['edgeThermalBridges', 'method']) === 'forfait') &&
      <p className="nta-form-note">{t('nta.form.edgeBridges.forfaitNote')}</p>}
  </>;
}

function VerticalPipesSection({ draft, change, project }: NtaSectionProps) {
  const { t } = useI18n();
  const residential = residentialScope(draft);
  return <>
    {project.zones.length > 1
      ? ((read(draft, ['zoneData']) as Draft[] | undefined) ?? []).map((item, index) =>
        <VerticalPipesFields key={String(item.zoneId)} draft={draft} change={change} residential={residential}
          path={['zoneData', index, 'verticalPipes']} label={String(item.zoneId)} />)
      : <VerticalPipesFields draft={draft} change={change} residential={residential} path={['verticalPipes']} />}
    <p className="nta-form-note">{t('nta.form.verticalPipes.note')}</p>
  </>;
}

function VentilationModeSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change, update, project } = props;
  const f = fieldOf(props);
  const chapter11 = read(draft, ['ventilation']) != null;
  const months = (read(draft, ['ventilationFlows', 0, 'months']) as Draft[] | undefined) ?? [];
  const constantConductance = months.length === 12 && months.every((month) => month.conductanceWPerK === months[0].conductanceWPerK)
    ? months[0].conductanceWPerK : null;
  const setConstantVentilation = (value: number | null) => change(['ventilationFlows', 0, 'months'],
    Array.from({ length: 12 }, (_, index) => ({ ...(months[index] ?? {}), month: index + 1, conductanceWPerK: value })));
  return <>
    <label data-path="ntaCalculation.ventilation">{t('nta.vent.mode')}
      <select value={chapter11 ? 'chapter11' : 'explicit'} onChange={(event) => update((current) => event.target.value === 'chapter11'
        ? syncVentilation({ ...current, ventilation: buildVentilationDraft(current, project) }, project)
        : removeVentilation(current))}>
        <option value="chapter11">{t('nta.vent.mode.chapter11')}</option>
        <option value="explicit">{t('nta.vent.mode.explicit')}</option>
      </select>
    </label>
    {!chapter11 && <>
      <label data-path="ntaCalculation.ventilationFlows">{t('nta.form.ventilationConstant')}
        <input type="number" step="any" value={typeof constantConductance === 'number' ? constantConductance : ''}
          onChange={(event) => setConstantVentilation(event.target.value === '' ? null : Number(event.target.value))} />
      </label>
      <label>{t('nta.form.ventilationFlow')}
        <input type="number" step="any" value={typeof constantConductance === 'number'
          ? Math.round((constantConductance / AIR_HEAT_CAPACITY_W_PER_M3H_K) * 100) / 100 : ''}
          onChange={(event) => setConstantVentilation(event.target.value === ''
            ? null : Number(event.target.value) * AIR_HEAT_CAPACITY_W_PER_M3H_K)} />
      </label>
      <TextField {...f} path={['ventilationFlows', 0, 'sourceReference']} label={t('nta.form.source')} />
      <p className="nta-form-note">{t('nta.form.ventilationNote')}</p>
    </>}
    {chapter11 && project.zones.length > 1 && <p className="nta-form-note">{t('nta.vent.multiZone')}</p>}
    {chapter11
      ? <p className="nta-form-note">{t('nta.vent.c1Automatic')}</p>
      : <CheckField {...f} path={['demandUsesFixedC1Ventilation']} label={t('nta.form.c1')} />}
  </>;
}

function EmissionSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change } = props;
  const f = fieldOf(props);
  return <>
    <SelectField {...f} path={['emission', 'system']} label={t('nta.form.emissionSystem')} options={[
      ['radiators_or_convectors', t('nta.form.emission.radiators')], ['floor_heating', t('nta.form.emission.floor')],
      ['fan_assisted_radiators_or_convectors', t('nta.form.emission.fanAssisted')],
      ['air_heating', t('nta.form.emission.air')], ['local_heater', t('nta.form.emission.local')], ['other_or_unknown', t('nta.form.unknown')]]} />
    {(read(draft, ['emission', 'system']) === 'air_heating' || read(draft, ['emission', 'airHeaters']) != null) &&
      <AirHeatersFields draft={draft} change={change} />}
    <HeatingEmission2023Fields draft={draft} change={change} />
    <TextField {...f} path={['emission', 'sourceReference']} label={t('nta.form.source')} />
  </>;
}

function EmissionFansSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change } = props;
  const f = fieldOf(props);
  return <>
    <label className="nta-form-check" data-path="ntaCalculation.emission.fans">
      <input type="checkbox" checked={read(draft, ['emission', 'fans']) != null}
        onChange={(event) => change(['emission', 'fans'], event.target.checked
          ? { kind: 'fan_convector', count: 1, sourceReference: '' } : undefined)} />
      {t('nta.form.emission.fans')}
    </label>
    {read(draft, ['emission', 'fans']) != null && <>
      <SelectField {...f} path={['emission', 'fans', 'kind']} label={t('nta.form.emission.fanKind')} options={[
        ['fan_convector', t('nta.form.emission.fanKind.convector')], ['electric_heating', t('nta.form.emission.fanKind.electric')],
        ['dynamic_storage', t('nta.form.emission.fanKind.storage')], ['unknown', t('nta.form.unknown')]]} />
      <NumberField {...f} path={['emission', 'fans', 'count']} label={t('nta.form.emission.fanCount')} step="1" />
      <NumberField {...f} path={['emission', 'fans', 'testedPowerW']} label={t('nta.form.emission.fanTestedPower')} />
      <TextField {...f} path={['emission', 'fans', 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

function EmissionControlSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const f = fieldOf(props);
  return <>
    <SelectField {...f} path={['emission', 'balancing']} label={t('nta.form.balancing')} options={[
      ['none_or_unknown', t('nta.form.unknown')], ['static', t('nta.form.balancing.static')],
      ['dynamic', t('nta.form.balancing.dynamic')], ['not_applicable', t('nta.form.balancing.na')]]} />
    <SelectField {...f} path={['emission', 'control']} label={t('nta.form.control')} options={[
      ['main_room_thermostat', t('nta.form.control.main')], ['central_with_room_valves', t('nta.form.control.central')],
      ['individual_room_thermostats', t('nta.form.control.individual')], ['other_or_unknown', t('nta.form.unknown')]]} />
  </>;
}

function DistributionSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const f = fieldOf(props);
  return <>
    <TextField {...f} path={['distribution', 'sourceReference']} label={t('nta.form.distributionSource')} />
    <NtaDistributionFields draft={props.draft} change={props.change} />
  </>;
}

function GeneratorSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const f = fieldOf(props);
  return <>
    <SpaceGeneratorFields draft={props.draft} change={props.change} base={['generator']} project={props.project} allowMultiple />
    <NumberField {...f} path={['identicalSystems']} label={t('nta.form.identicalSystems')} step="1" />
  </>;
}

function HotWaterSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change } = props;
  const f = fieldOf(props);
  const residential = residentialScope(draft);
  return <>
    {residential
      ? <NumberField {...f} path={['hotWater', 'need', 'dwellingCount']} label={t('nta.form.dwellingCount')} step="1" />
      : <>
        <SelectField {...f} path={['hotWater', 'need', 'areas', 0, 'function']} label={t('nta.form.hotWaterFunction')} options={[
          ['office', t('nta.form.label.office')], ['education', t('nta.form.label.education')], ['retail', t('nta.form.label.retail')],
          ['sport', t('nta.form.label.sport')], ['lodging', t('nta.form.label.lodging')], ['cell', t('nta.form.label.cell')],
          ['assembly_without_day_care', t('nta.form.label.assembly')], ['assembly_with_day_care', t('nta.form.label.dayCare')],
          ['healthcare_without_beds', t('nta.form.label.healthcare')], ['healthcare_with_beds', t('nta.form.label.healthcareBeds')]]} />
        <NumberField {...f} path={['hotWater', 'need', 'areas', 0, 'areaM2']} label={t('nta.form.area')} />
      </>}
    <TextField {...f} path={['hotWater', 'need', 'sourceReference']} label={t('nta.form.source')} />
    {residential ? <>
      <SelectField {...f} path={['hotWater', 'emission', 'served']} label={t('nta.form.hotWaterTaps')} options={[
        ['kitchen_and_bathroom', t('nta.form.dhwTaps.both')], ['bathroom_only', t('nta.form.dhwTaps.bathroom')], ['kitchen_only', t('nta.form.dhwTaps.kitchen')]]} />
      <NumberField {...f} path={['hotWater', 'emission', 'kitchenLengthM']} label={t('nta.form.hotWaterKitchenLength')} />
      <KitchenPipeDiameterField draft={draft} change={change} path={['hotWater', 'emission', 'kitchenPipeDiameter']} />
      <NumberField {...f} path={['hotWater', 'emission', 'bathroomLengthM']} label={t('nta.form.hotWaterBathroomLength')} />
    </> : <NumberField {...f} path={['hotWater', 'emission', 'meanLengthM']} label={t('nta.form.hotWaterMeanLength')} />}
    <TextField {...f} path={['hotWater', 'emission', 'sourceReference']} label={t('nta.form.source')} />
    <HotWaterGeneratorFields draft={draft} change={change} base={['hotWater', 'generator']} />
    <HotWaterGeneratorsFields draft={draft} change={change} />
    <HotWaterStorageFields draft={draft} change={change} />
    <TextField {...f} path={['hotWater', 'equipmentReference']} label={t('nta.form.boilerEquipmentSource')} />
  </>;
}

function PvSection({ draft, change }: NtaSectionProps) {
  const pv = (read(draft, ['pvSystems']) as Draft[] | undefined) ?? [];
  return <>
    {pv.map((item, index) => <div key={String(item.id)} className="nta-form-row">
      <PvSystemFields draft={draft} change={change} base={['pvSystems', index]} label={String(item.id)} />
    </div>)}
  </>;
}

function BacsSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const { draft, change } = props;
  const f = fieldOf(props);
  const bacs = read(draft, ['bacs']) != null;
  return <>
    <NumberField {...f} path={['bacsFactor']} label={t('nta.form.bacs')} />
    <TextField {...f} path={['bacsSourceReference']} label={t('nta.form.source')} />
    <label className="nta-form-check" data-path="ntaCalculation.bacs">
      <input type="checkbox" checked={bacs}
        onChange={(event) => change(['bacs'], event.target.checked ? bacsTemplate(residentialScope(draft)) : undefined)} />
      {t('nta.form.bacsDraft')}
    </label>
    {bacs && <BacsFields draft={draft} change={change} />}
  </>;
}

function ConfirmationsSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const f = fieldOf(props);
  return <>
    <CheckField {...f} path={['useInventoryComplete']} label={t('nta.form.useInventory')} />
    <CheckField {...f} path={['productionInventoryComplete']} label={t('nta.form.productionInventory')} />
  </>;
}

function StorageSection(props: NtaSectionProps) {
  const { t } = useI18n();
  const f = fieldOf(props);
  return <>
    <CheckField {...f} path={['batteryStoragePresent']} label={t('nta.form.battery')} />
    {read(props.draft, ['batteryStoragePresent']) === true && <>
      <NumberField {...f} path={['storage', 'buildingBoundElectricalKwh']} label={t('nta.form.storageElectrical')} />
      <NumberField {...f} path={['storage', 'buildingBoundThermalKwh']} label={t('nta.form.storageThermal')} />
      <TextField {...f} path={['storage', 'sourceReference']} label={t('nta.form.source')} />
    </>}
  </>;
}

function DynamicWindowsSection({ draft, change, project }: NtaSectionProps) {
  return <DynamicWindowsFields draft={draft} change={change} project={project} />;
}

const hasHotWater = (_: IProject, draft: Draft) => read(draft, ['hotWater']) != null;
const listed = (key: string) => (_: IProject, draft: Draft) => ((read(draft, [key]) as unknown[] | undefined) ?? []).length > 0;

/** All sections, in the order of the full form. */
export const NTA_SECTIONS: NtaSectionDef[] = [
  { id: 'general', step: 'project', titleKey: 'nta.form.general', Component: GeneralSection,
    paths: ['normVersion', 'calculationScope', 'areaSourceReference', 'usageFunction', 'dwellingType', 'bblFunction',
      'zebHeatDeliveryTemperature', 'permitApplicationAfter20260529', 'constructionYear', 'fossilAppliancesOutsideCalculation'] },
  { id: 'activeCooling', step: 'installations', sub: 'cooling', titleKey: 'ntaStep.section.activeCooling', Component: ActiveCoolingSection,
    paths: ['activeCooling'] },
  { id: 'functions', step: 'building', sub: 'zones', titleKey: 'nta.form.functions', Component: FunctionsSection,
    paths: ['labelFunctions', 'bblFunctions', 'functionAreas'] },
  { id: 'setpoints', step: 'building', sub: 'zones', titleKey: 'nta.form.setpoints', Component: SetpointsSection, paths: ['setpoints'] },
  { id: 'mass', step: 'building', sub: 'zones', titleKey: 'nta.form.mass', Component: MassSection, paths: ['thermalMass'] },
  { id: 'internalGains', step: 'building', sub: 'zones', titleKey: 'nta.form.internalGains', Component: InternalGainsSection, paths: ['internalGains'] },
  { id: 'windows', step: 'building', sub: 'envelope', titleKey: 'nta.form.windows', Component: WindowsSection, paths: ['windowSolar'] },
  { id: 'windowObstructions', step: 'building', sub: 'envelope', titleKey: 'nta.form.windowObstructions.title',
    Component: ({ draft, change, project }) => <WindowObstructionsFields draft={draft} change={change} project={project} />,
    paths: ['windowObstructions'] },
  { id: 'dynamicWindows', step: 'building', sub: 'envelope', titleKey: 'nta.form.dynamic.title', Component: DynamicWindowsSection,
    paths: ['dynamicWindows'], advanced: true },
  { id: 'sunrooms', step: 'building', sub: 'unheated', titleKey: 'nta.form.sunroom.title',
    Component: ({ draft, change }) => <SunroomsFields draft={draft} change={change} />, paths: ['sunrooms'] },
  { id: 'roofTilts', step: 'building', sub: 'envelope', titleKey: 'nta.form.roofTilts', Component: RoofTiltsSection,
    paths: ['surfaceTilts'], when: listed('surfaceTilts') },
  { id: 'groundFloors', step: 'building', sub: 'envelope', titleKey: 'nta.form.groundFloors', Component: GroundFloorsSection,
    paths: ['groundFloors'], when: listed('groundFloors') },
  { id: 'verticalPipes', step: 'installations', sub: 'heating', part: 'distribution', titleKey: 'nta.form.verticalPipes',
    Component: VerticalPipesSection, paths: ['verticalPipes'], advanced: true },
  { id: 'ventilation', step: 'installations', sub: 'ventilation', titleKey: 'nta.form.ventilation', Component: VentilationModeSection,
    paths: ['ventilationFlows', 'demandUsesFixedC1Ventilation'] },
  { id: 'ventilationChapter11', step: 'installations', sub: 'ventilation', titleKey: 'nta.form.ventilation', bare: true,
    Component: ({ draft, change }) => <NtaVentilationSection draft={draft} change={change} />,
    paths: ['ventilation'], when: (_, draft) => read(draft, ['ventilation']) != null },
  { id: 'emission', step: 'installations', sub: 'heating', part: 'emission', titleKey: 'ntaStep.section.emission', Component: EmissionSection,
    paths: ['emission.system', 'emission.airHeaters', 'emission.sourceReference'] },
  { id: 'emissionFans', step: 'installations', sub: 'heating', part: 'auxiliary', titleKey: 'ntaStep.section.fans', Component: EmissionFansSection,
    paths: ['emission.fans'] },
  { id: 'emissionControl', step: 'installations', sub: 'heating', part: 'control', titleKey: 'ntaStep.section.control', Component: EmissionControlSection,
    paths: ['emission.balancing', 'emission.control'] },
  { id: 'distribution', step: 'installations', sub: 'heating', part: 'distribution', titleKey: 'ntaStep.section.distribution',
    Component: DistributionSection, paths: ['distribution', 'distributionSystem'] },
  { id: 'generator', step: 'installations', sub: 'heating', part: 'generation', titleKey: 'nta.form.generator', Component: GeneratorSection,
    paths: ['generator', 'identicalSystems'] },
  { id: 'collective', step: 'installations', sub: 'heating', part: 'generation', titleKey: 'ntaStep.section.collective', advanced: true,
    Component: ({ draft, change }) => <CollectiveAndRenewableFields draft={draft} change={change} />,
    paths: ['collectiveConnection', 'heatPumpRenewable'] },
  { id: 'bcrg', step: 'installations', sub: 'heating', part: 'control', titleKey: 'nta.form.bcrg.title', advanced: true,
    Component: () => <DeclaredHeatingTableTool />, paths: [] },
  { id: 'heatingSystems', step: 'installations', sub: 'heating', part: 'generation', titleKey: 'nta.form.heatingSystems.title', advanced: true,
    Component: ({ draft, change, project }) => <AdditionalHeatingSystemsFields draft={draft} change={change} project={project} />,
    paths: ['additionalHeatingSystems', 'heatingSystems'], when: (project) => project.zones.length > 1 },
  { id: 'hotWater', step: 'installations', sub: 'hotWater', titleKey: 'nta.form.hotWater', Component: HotWaterSection,
    paths: ['hotWater'], when: hasHotWater },
  { id: 'dhwSystems', step: 'installations', sub: 'hotWater', titleKey: 'nta.form.dhwSystems.title', advanced: true,
    Component: ({ draft, change }) => <AdditionalHotWaterSystemsFields draft={draft} change={change} residential={residentialScope(draft)} />,
    paths: ['additionalHotWaterSystems'], when: hasHotWater },
  { id: 'solarWater', step: 'installations', sub: 'hotWater', titleKey: 'nta.form.solar.title', advanced: true,
    Component: ({ draft, change }) => <SolarWaterHeaterFields draft={draft} change={change} />,
    paths: ['hotWater.solar'], when: hasHotWater },
  { id: 'spaceHeatingSolar', step: 'installations', sub: 'heating', part: 'solar', titleKey: 'nta.form.spaceHeatingSolar.title',
    Component: ({ draft, change }) => <SpaceHeatingSolarFields draft={draft} change={change} />, paths: ['spaceHeatingSolar'] },
  { id: 'coolingSystems', step: 'installations', sub: 'cooling', titleKey: 'nta.form.cooling',
    Component: ({ draft, change, project }) => <CoolingSystemsFields draft={draft} change={change} project={project} />,
    paths: ['cooling', 'coolingSystems'] },
  { id: 'humidifiers', step: 'installations', sub: 'humidification', titleKey: 'nta.form.humidifiers.title',
    Component: ({ draft, change, project }) => <HumidifiersFields draft={draft} change={change} project={project} />, paths: ['humidifiers'] },
  { id: 'lighting', step: 'installations', sub: 'lighting', titleKey: 'ntaStep.section.lighting', bare: true,
    Component: ({ draft, change, project }) => <NtaLightingSection draft={draft} change={change} project={project} />,
    paths: ['lighting'], when: (_, draft) => !residentialScope(draft) },
  { id: 'pv', step: 'installations', sub: 'generation', titleKey: 'nta.form.pv', Component: PvSection,
    paths: ['pvSystems'], when: listed('pvSystems') },
  { id: 'bacs', step: 'installations', sub: 'bacs', titleKey: 'ntaStep.section.bacs', Component: BacsSection,
    paths: ['bacsFactor', 'bacsSourceReference', 'bacs'] },
  { id: 'externalSupply', step: 'installations', sub: 'generation', titleKey: 'ntaStep.section.externalSupply',
    Component: ({ draft, change }) => <ExternalSupplyFields draft={draft} change={change} />, paths: ['externalSupply'] },
  { id: 'confirmations', step: 'check', sub: 'input', titleKey: 'nta.form.inventory', Component: ConfirmationsSection,
    paths: ['useInventoryComplete', 'productionInventoryComplete'] },
  { id: 'declaredFlows', step: 'installations', sub: 'generation', titleKey: 'ntaStep.section.declaredFlows', advanced: true,
    Component: ({ draft, change }) => <DeclaredFlowsFields draft={draft} change={change} />,
    paths: ['declaredUses', 'onSiteProduction', 'declaredRenewableHeat'] },
  { id: 'storage', step: 'installations', sub: 'generation', titleKey: 'ntaStep.section.storage', advanced: true,
    Component: StorageSection, paths: ['batteryStoragePresent', 'storage'] },
];

/** The sections that apply to this draft, optionally narrowed by `filter`. */
export function visibleSections(project: IProject, draft: Draft, filter?: (def: NtaSectionDef) => boolean): NtaSectionDef[] {
  return NTA_SECTIONS.filter((def) => (!filter || filter(def)) && (!def.when || def.when(project, draft)));
}

/** The pages with NTA sections, in workflow order (for "Toepassen en verder" and the routes). */
export function sectionPages(): Array<{ step: StepId; sub?: string }> {
  const pages: Array<{ step: StepId; sub?: string }> = [];
  for (const def of NTA_SECTIONS) {
    if (!pages.some((page) => page.step === def.step && page.sub === def.sub)) pages.push({ step: def.step, sub: def.sub });
  }
  return pages;
}
