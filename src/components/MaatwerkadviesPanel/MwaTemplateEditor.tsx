import { useState, type ReactNode } from 'react';
import { useI18n } from '../../i18n/i18n';
import type { IProject } from '../../core/energy/types';
import type { MwaMeasure, MwaPatchOperation } from '../../core/nta/KernelClient';
import {
  applyLightingChanges, buildTemplatePatch, derivedRenewableFlags, forfaitHeatPumpTemplate, HEAT_PUMP_KINDS, heatPumpRenewableTemplate, initialTemplate,
  insulationOptions, insulationValues, lightingChanges, normalizeTemplate, PV_OBSTRUCTION_SOURCE, pvSystemTemplate, TEMPLATE_CATEGORY,
  TEMPLATE_KINDS, TEMPLATE_LIFETIME, windowOptions,
  type InsulationPart, type MwaMeasureTemplate, type MwaTemplateKind,
} from '../../core/nta/MwaTemplates';
import { solarWaterHeaterTemplate } from '../../core/nta/NtaSystemTemplates';
import { CheckField, NumberField, read, SelectField, TextField, write, type Draft, type Path } from '../NtaPerformancePanel/NtaFormFields';
import { HotWaterGeneratorFields, SolarWaterHeaterFields, SpaceGeneratorFields } from '../NtaPerformancePanel/NtaSystemSections';
import { PvSystemFields } from '../NtaPerformancePanel/NtaPvFields';
import { VentilationUnitFields } from '../NtaPerformancePanel/NtaVentilationSection';
import { NtaLightingSection } from '../NtaPerformancePanel/NtaExtraSections';

type Block = Record<string, unknown>;

/** The measure with `template` and the patch it generates against `project`. */
export function applyTemplate(project: IProject, measure: MwaMeasure, template: MwaMeasureTemplate | null): MwaMeasure {
  const { incomplete: _drop, ...rest } = measure;
  void _drop;
  if (!template) return { ...rest, template: null };
  return { ...rest, template, patch: buildTemplatePatch(project, template, measure.id).patch };
}

/** Starting lifetime of a new measure before any template (MaatwerkadviesPanel). */
const NEW_MEASURE_LIFETIME = 30;

/**
 * A measure of a template kind. Category and lifetime follow the kind only
 * while they still hold the previous kind's starting values; what the
 * adviser entered is kept.
 */
export function templateMeasure(project: IProject, measure: MwaMeasure, kind: MwaTemplateKind): MwaMeasure {
  const previous = measure.template?.kind;
  const defaultLifetimes = new Set([NEW_MEASURE_LIFETIME, ...(previous ? [TEMPLATE_LIFETIME[previous]] : [])]);
  const categoryIsDefault = !previous || measure.category === TEMPLATE_CATEGORY[previous];
  return applyTemplate(project, {
    ...measure,
    category: categoryIsDefault ? TEMPLATE_CATEGORY[kind] : measure.category,
    target: 'project',
    lifetimeYears: defaultLifetimes.has(measure.lifetimeYears) ? TEMPLATE_LIFETIME[kind] : measure.lifetimeYears,
  }, initialTemplate(kind, project));
}

/** Fields of a template part that is edited with the NTA form's own components. */
function DraftFields({ draft, onDraft, children }: {
  draft: Draft;
  onDraft: (next: Draft) => void;
  children: (change: (path: Path, value: unknown) => void) => ReactNode;
}) {
  const change = (path: Path, value: unknown) => onDraft(write(draft, path, value));
  return <div className="nta-form mwa-template-draft"><div className="nta-form-grid">{children(change)}</div></div>;
}

function num(value: string): number | null {
  if (value.trim() === '') return null;
  const parsed = Number(value.replace(',', '.'));
  return Number.isFinite(parsed) ? parsed : null;
}

/** A number typed with a decimal comma or point; keeps the typed text while it is being edited. */
function DecimalInput({ value, onChange, disabled }: { value: number | null; onChange: (value: number | null) => void; disabled?: boolean }) {
  const [text, setText] = useState(value == null ? '' : String(value));
  // Follow outside changes (another template, a reset) but not our own parses.
  if (num(text) !== value && !(text.trim() === '' && value == null)) {
    const next = value == null ? '' : String(value);
    if (next !== text) setText(next);
  }
  return <input inputMode="decimal" value={text} disabled={disabled}
    onChange={(e) => { setText(e.target.value); onChange(num(e.target.value)); }} />;
}

/** Table 9.27/9.29 row inputs of a forfait heat pump at `generator.forfait`. */
function ForfaitHeatPumpFields({ draft, change }: { draft: Draft; change: (path: Path, value: unknown) => void }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const at = (...rest: Path): Path => ['generator', 'forfait', ...rest];
  return <>
    <SelectField {...field} path={at('scope')} label={t('mwa.template.hp.scope')} options={[
      ['residential_at_most25_kw', t('mwa.template.hp.scope.residential')],
      ['utility_collective_or_over25_kw', t('mwa.template.hp.scope.utility')]]} />
    <SelectField {...field} path={at('source')} label={t('mwa.template.hp.source')} options={[
      'outdoor_air', 'exhaust_air', 'ground', 'ground_or_groundwater_unknown', 'groundwater_below15_c', 'surface_water',
      'collective15_to20_c', 'collective20_to40_c', 'collective_at_least40_c'].map((key) => [key, t(`mwa.template.hp.source.${key}`)])} />
    <SelectField {...field} path={at('sink')} label={t('mwa.template.hp.sink')} options={[
      ['hydronic', t('mwa.template.hp.sink.hydronic')], ['indoor_air', t('mwa.template.hp.sink.air')]]} />
    <NumberField {...field} path={at('designSupplyTemperatureC')} label={t('mwa.template.hp.supply')} />
    <NumberField {...field} path={at('thermalCapacityKw')} label={t('mwa.template.hp.capacity')} optional />
    {read(draft, at('thermalCapacityKw')) != null &&
      <TextField {...field} path={at('capacitySourceReference')} label={t('mwa.template.hp.capacitySource')} />}
    <NumberField {...field} path={at('sourceCorrectionFactor')} label={t('mwa.template.hp.correction')} step="0.01" optional />
    {read(draft, at('sourceCorrectionFactor')) != null &&
      <TextField {...field} path={at('sourceCorrectionReference')} label={t('mwa.template.hp.correctionSource')} />}
    <TextField {...field} path={at('classificationSourceReference')} label={t('mwa.source')} />
    <TextField {...field} path={['generator', 'sourceSystemReference']} label={t('mwa.template.hp.sourceSystem')} />
  </>;
}

/** Short JSON of a patch value for the preview. */
function preview(operation: MwaPatchOperation): string {
  if (operation.op === 'remove') return '';
  const text = JSON.stringify(operation.value);
  return text.length > 120 ? `${text.slice(0, 117)}…` : text;
}

/** Template-based measure editing (ISSO 82.2/75.2 §4.3): choices, problems and the generated patch. */
export function MwaTemplateEditor({ project, measure, onChange }: {
  project: IProject;
  measure: MwaMeasure;
  onChange: (measure: MwaMeasure) => void;
}) {
  const { t } = useI18n();
  if (!measure.template) return null;
  // Saved snapshot forms (ventilation, lighting) open in their current form.
  const { template } = normalizeTemplate(project, measure.template);
  const set = (next: MwaMeasureTemplate) => onChange(applyTemplate(project, measure, next));
  const { patch, problems } = buildTemplatePatch(project, measure.template, measure.id);
  const block = project.ntaCalculation as unknown as Block | undefined;

  let fields: ReactNode = null;
  switch (template.kind) {
    case 'insulation': {
      const options = insulationOptions(project, template.part);
      const values = insulationValues(template.part, template.rcValue, template.uValue);
      fields = <>
        <label>{t('mwa.template.part')}
          <select value={template.part} onChange={(e) => set({ ...template, part: e.target.value as InsulationPart, surfaces: [] })}>
            {(['roof', 'facade', 'floor'] as const).map((part) => <option key={part} value={part}>{t(`mwa.template.part.${part}`)}</option>)}
          </select>
        </label>
        <label>{t('mwa.template.rc')}
          <DecimalInput value={template.rcValue} onChange={(value) => set({ ...template, rcValue: value })} />
        </label>
        <label>{t('mwa.template.u')}
          <DecimalInput value={template.uValue} disabled={template.rcValue != null}
            onChange={(value) => set({ ...template, uValue: value })} />
        </label>
        {values && <p className="mwa-wide nta-form-note">{t('mwa.template.resulting')}: R<sub>c</sub> {values.rc.toFixed(2)} m²·K/W, U {values.u.toFixed(3)} W/(m²·K)</p>}
        <fieldset className="mwa-wide mwa-checks">
          <legend>{t('mwa.template.surfaces')}</legend>
          {options.length === 0 && <em>{t('mwa.template.noOptions')}</em>}
          {options.map((option) => <label key={option.key} className="mwa-check">
            <input type="checkbox" checked={template.surfaces.includes(option.key)} onChange={(e) => set({
              ...template,
              surfaces: e.target.checked ? [...template.surfaces, option.key] : template.surfaces.filter((key) => key !== option.key),
            })} />
            {option.label}
          </label>)}
        </fieldset>
      </>;
      break;
    }
    case 'glazing': {
      const options = windowOptions(project);
      fields = <>
        <label>{t('mwa.template.windowU')}
          <DecimalInput value={template.uValue} onChange={(value) => set({ ...template, uValue: value })} />
        </label>
        <label>{t('mwa.template.windowG')}
          <DecimalInput value={template.gValue} onChange={(value) => set({ ...template, gValue: value })} />
        </label>
        <fieldset className="mwa-wide mwa-checks">
          <legend>{t('mwa.template.windows')}</legend>
          {options.length === 0 && <em>{t('mwa.template.noOptions')}</em>}
          {options.map((option) => <label key={option.key} className="mwa-check">
            <input type="checkbox" checked={template.windows.includes(option.key)} onChange={(e) => set({
              ...template,
              windows: e.target.checked ? [...template.windows, option.key] : template.windows.filter((key) => key !== option.key),
            })} />
            {option.label}
          </label>)}
        </fieldset>
      </>;
      break;
    }
    case 'airtightness':
      fields = <>
        <label>{t('mwa.template.qv10')}
          <DecimalInput value={template.qv10DmPerSM2} onChange={(value) => set({ ...template, qv10DmPerSM2: value })} />
        </label>
        <label className="mwa-wide">{t('mwa.source')}
          <input value={template.sourceReference} onChange={(e) => set({ ...template, sourceReference: e.target.value })} />
        </label>
        <p className="mwa-wide nta-form-note">{t('mwa.template.qv10Note')}</p>
      </>;
      break;
    case 'ventilation': {
      // The project's ventilation with the measure's system; only the system is stored.
      const ventilation = { ...((block?.ventilation as Block | undefined) ?? {}), system: template.system };
      fields = <DraftFields draft={{ ventilation, calculationScope: block?.calculationScope }}
        onDraft={(next) => set({ ...template, system: (next.ventilation as Block).system as Block })}>
        {(change) => <VentilationUnitFields draft={{ ventilation }} change={change}
          path={['ventilation', 'system', 'unit']} />}
      </DraftFields>;
      break;
    }
    case 'heat_pump': {
      const flags = derivedRenewableFlags(template.generator);
      const renewable = { ...(template.renewable ?? heatPumpRenewableTemplate()), ...(flags ?? {}) };
      const draft = { generator: template.generator, renewable };
      const heatPump = HEAT_PUMP_KINDS.has(String(template.generator.kind));
      fields = <DraftFields draft={draft} onDraft={(next) => {
        const generator = next.generator as Block;
        // A switch to the forfait heat pump starts its table-row input here.
        set({ ...template, renewable: next.renewable as Block, generator: generator.kind === 'heat_pump_forfait' && generator.forfait == null
          ? { ...generator, forfait: forfaitHeatPumpTemplate(project), sourceSystemReference: '' } : generator });
      }}>
        {(change) => <>
          <SpaceGeneratorFields draft={draft} change={change} base={['generator']} project={project} heatPumpNote={false} />
          {template.generator.kind === 'heat_pump_forfait' && <ForfaitHeatPumpFields draft={draft} change={change} />}
          {heatPump && <>
            <CheckField draft={draft} onChange={change} path={['renewable', 'sourceBelow20C']} label={t('mwa.template.hp.below20')}
              disabled={flags?.sourceBelow20C !== undefined} />
            <CheckField draft={draft} onChange={change} path={['renewable', 'exhaustAirSource']} label={t('mwa.template.hp.exhaustAir')}
              disabled={flags != null} />
            {flags && <p className="nta-form-note">{t('mwa.template.hp.flagsDerived')}</p>}
            <TextField draft={draft} onChange={change} path={['renewable', 'sourceReference']} label={t('mwa.template.hp.renewableSource')} />
          </>}
        </>}
      </DraftFields>;
      break;
    }
    case 'hot_water':
      fields = <DraftFields draft={{ hotWater: { generator: template.generator } }}
        onDraft={(next) => set({ ...template, generator: read(next, ['hotWater', 'generator']) as Block })}>
        {(change) => <HotWaterGeneratorFields draft={{ hotWater: { generator: template.generator } }} change={change}
          base={['hotWater', 'generator']} />}
      </DraftFields>;
      break;
    case 'pv': {
      const draft = { pvSystems: template.systems };
      fields = <DraftFields draft={draft} onDraft={(next) => set({ ...template, systems: (next.pvSystems as Block[]) ?? [] })}>
        {(change) => <>
          {template.systems.map((system, index) => <fieldset key={index} className="nta-form-row">
            <PvSystemFields draft={draft} change={change} base={['pvSystems', index]} label={String(system.id)} />
            {(system.obstruction as Block | undefined)?.method === 'minimal' && <>
              <TextField draft={draft} onChange={change} path={['pvSystems', index, PV_OBSTRUCTION_SOURCE]} label={t('mwa.template.pvObstructionSource')} />
              <p className="nta-form-note">{t('mwa.template.pvObstructionNote')}</p>
            </>}
            <button type="button" className="nta-form-remove"
              onClick={() => set({ ...template, systems: template.systems.filter((_, other) => other !== index) })}>{t('mwa.remove')}</button>
          </fieldset>)}
          <button type="button" className="btn" onClick={() => set({ ...template, systems: [...template.systems, pvSystemTemplate(template.systems.length)] })}>
            {t('mwa.template.addPv')}
          </button>
        </>}
      </DraftFields>;
      break;
    }
    case 'solar_water_heater':
      fields = <DraftFields draft={{ hotWater: { solar: template.systems } }}
        onDraft={(next) => set({ ...template, systems: (read(next, ['hotWater', 'solar']) as Block[] | undefined) ?? [] })}>
        {(change) => <SolarWaterHeaterFields draft={{ hotWater: { solar: template.systems } }} change={change} />}
      </DraftFields>;
      if (template.systems.length === 0) {
        fields = <>{fields}<button type="button" className="btn"
          onClick={() => set({ ...template, systems: [solarWaterHeaterTemplate(0)] })}>{t('mwa.template.addSolar')}</button></>;
      }
      break;
    case 'shower_heat_recovery': {
      const recovery = template.recovery;
      const showers = (recovery.showers as Block[] | undefined) ?? [];
      const setRecovery = (patchValue: Block) => set({ ...template, recovery: { ...recovery, ...patchValue } });
      const setShower = (index: number, shower: Block) => setRecovery({ showers: showers.map((item, other) => (other === index ? shower : item)) });
      fields = <>
        {showers.map((shower, index) => <fieldset key={index} className="mwa-wide mwa-patch-row">
          <label>{t('mwa.template.shower')} {index + 1}
            <select value={String(shower.unit)} onChange={(e) => setShower(index, e.target.value === 'declared'
              ? { unit: 'declared', efficiency: null, sourceReference: '' } : { unit: e.target.value })}>
              {['vertical', 'horizontal', 'declared', 'none'].map((unit) => <option key={unit} value={unit}>{t(`mwa.template.showerUnit.${unit}`)}</option>)}
            </select>
          </label>
          {shower.unit === 'declared' && <>
            <label>{t('mwa.template.showerEfficiency')}
              <DecimalInput value={typeof shower.efficiency === 'number' ? shower.efficiency : null}
                onChange={(value) => setShower(index, { ...shower, efficiency: value })} />
            </label>
            <label>{t('mwa.source')}
              <input value={String(shower.sourceReference ?? '')} onChange={(e) => setShower(index, { ...shower, sourceReference: e.target.value })} />
            </label>
          </>}
          {showers.length > 1 && <button type="button" className="btn"
            onClick={() => setRecovery({ showers: showers.filter((_, other) => other !== index) })}>{t('mwa.remove')}</button>}
        </fieldset>)}
        <button type="button" className="btn" onClick={() => setRecovery({ showers: [...showers, { unit: 'vertical' }] })}>
          {t('mwa.template.addShower')}
        </button>
        <label>{t('mwa.template.showerConnection')}
          <select value={String(recovery.connection)} onChange={(e) => setRecovery({ connection: e.target.value })}>
            {['mixer_and_heater', 'mixer_only', 'heater_only', 'shared_units', 'unknown'].map((item) =>
              <option key={item} value={item}>{t(`mwa.template.showerConnection.${item}`)}</option>)}
          </select>
        </label>
        <label className="mwa-wide">{t('mwa.source')}
          <input value={String(recovery.sourceReference ?? '')} onChange={(e) => setRecovery({ sourceReference: e.target.value })} />
        </label>
      </>;
      break;
    }
    case 'lighting': {
      // The project's lighting with the measure's changes; only the changes are stored.
      const current = (block?.lighting as Block[] | undefined) ?? [];
      const draft = { lighting: applyLightingChanges(current, template.zones), labelFunction: block?.labelFunction, calculationScope: block?.calculationScope };
      fields = <>
        {template.reviewed === false && <p className="mwa-wide nta-form-note" data-testid={`mwa-template-migrated-${measure.id}`}>
          {t('mwa.template.migrated')}{' '}
          <button type="button" className="btn" onClick={() => set({ ...template, reviewed: true })}>{t('mwa.template.migratedConfirm')}</button>
        </p>}
        <DraftFields draft={draft} onDraft={(next) => set({ ...template, zones: lightingChanges(current, (next.lighting as Block[]) ?? []) })}>
          {(change) => <NtaLightingSection draft={draft} change={change} project={project} />}
        </DraftFields>
      </>;
      break;
    }
  }

  return (
    <div className="mwa-wide mwa-template">
      <p className="nta-form-note">{t(`mwa.template.help.${template.kind}`)}</p>
      <div className="mwa-measure-fields">{fields}</div>
      {problems.length > 0 && <ul className="nta-performance-gaps mwa-template-problems" data-testid={`mwa-template-problems-${measure.id}`}>
        {problems.map((problem) => <li key={problem}>{t(`mwa.template.problem.${problem}`)}</li>)}
      </ul>}
      <details className="mwa-template-preview" data-testid={`mwa-template-preview-${measure.id}`}>
        <summary>{t('mwa.template.preview')} ({patch.length})</summary>
        <ol>{patch.map((operation, index) => (
          <li key={index}><code>{operation.op}</code> <code>{operation.path}</code> {preview(operation)}</li>
        ))}</ol>
      </details>
    </div>
  );
}

export { TEMPLATE_KINDS };
