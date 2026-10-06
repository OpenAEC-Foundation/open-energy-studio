import { useRef, useState } from 'react';
import { KernelCode, KernelDetail } from '../KernelCode/KernelCode';
import { adviceText } from '../../core/nta/MwaAdviceText';
import { formatNumber } from '../../i18n/format';
import { ClipboardList, FileDown, Plus, RefreshCw } from 'lucide-react';
import { useEnergy } from '../../context/EnergyContext';
import i18next, { useI18n } from '../../i18n/i18n';
import {
  assessMaatwerkadviesWithRust,
  DEFAULT_NORM_VERSION,
  type MaatwerkadviesAssessment,
  type MwaMeasure,
  type MwaMeasureCategory,
  type MwaMeasuredUse,
  type MwaPatchOperation,
  type MwaRenovationPassportInput,
  type MwaPackage,
  type MwaUserProfile,
  type MwaVariantResult,
  type NtaMaatwerkadvies,
} from '../../core/nta/KernelClient';
import { downloadMaatwerkadviesReportHTML } from '../../core/report/ReportGenerator';
import type { IProject } from '../../core/energy/types';
import { regenerateTemplatePatches, type MwaTemplateKind } from '../../core/nta/MwaTemplates';
import { applyTemplate, MwaTemplateEditor, TEMPLATE_KINDS, templateMeasure } from './MwaTemplateEditor';
import '../NtaPerformancePanel/NtaPerformancePanel.css';
import './MaatwerkadviesPanel.css';
import { Banner, Button, Card, EmptyState, Pill, Segmented, SideSheet } from '../ui';
import {
  LabelPathChart, MeasureCard, PackageComparison, PackageInspector, labelPathBars, measureProblems,
} from '../shell/pages/existing/MwaViews';
import '../shell/pages/existing/existing.css';

const CATEGORIES: MwaMeasureCategory[] = [
  'insulation', 'glazing', 'airtightness', 'ventilation', 'heat_recovery', 'heating', 'heat_pump',
  'hot_water', 'cooling', 'pv', 'solar_thermal', 'lighting', 'control', 'other',
];
const PROFILES: MwaUserProfile[] = ['nta', 'energy_conscious', 'average', 'not_energy_conscious'];

export function emptyMaatwerkadvies(): NtaMaatwerkadvies {
  return {
    currentUse: { profile: 'nta', sourceReference: '' },
    measures: [],
    packages: [],
    tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, electricityExportEurPerKwh: 0.05, sourceReference: '' },
    economics: { discountRate: 0.03, energyPriceChange: 0, sourceReference: '' },
  };
}

/** Number in the active UI language. */
function format(value: number | null | undefined, digits = 0): string {
  return formatNumber(value, i18next.language || 'nl', digits);
}

/** Monthly values separated by `;` or new lines (decimal comma allowed); `-` or empty is `null`. */
export function parseMonthly(text: string): Array<number | null> | undefined {
  if (!text.trim()) return undefined;
  return text.split(/[;\n]/).map((part) => {
    const value = part.trim();
    if (value === '' || value === '-') return null;
    const parsed = Number(value.replace(',', '.'));
    return Number.isFinite(parsed) ? parsed : null;
  });
}

function showMonthly(values: Array<number | null> | undefined): string {
  return values ? values.map((value) => (value == null ? '-' : String(value))).join('; ') : '';
}

function nextId(prefix: string, taken: string[]): string {
  let index = taken.length + 1;
  while (taken.includes(`${prefix}${index}`)) index += 1;
  return `${prefix}${index}`;
}

/** A patch value as typed: a number, true/false/null or JSON when it parses, else plain text. */
export function parsePatchValue(text: string): unknown {
  try {
    return JSON.parse(text) as unknown;
  } catch {
    return text;
  }
}

/** JSON type of a patch value; the row keeps it, so "2" stays text and 2 a number. */
export type PatchValueKind = 'number' | 'text' | 'boolean' | 'null' | 'json';

export function patchValueKind(value: unknown): PatchValueKind {
  if (typeof value === 'number') return 'number';
  if (typeof value === 'string') return 'text';
  if (typeof value === 'boolean') return 'boolean';
  if (value === null || value === undefined) return 'null';
  return 'json';
}

/** The value converted to another type, as far as it carries over. */
export function convertPatchValue(value: unknown, kind: PatchValueKind): unknown {
  switch (kind) {
    case 'number': {
      const number = typeof value === 'number' ? value : typeof value === 'string' ? Number(value.replace(',', '.')) : NaN;
      return Number.isFinite(number) && !(typeof value === 'string' && value.trim() === '') ? number : null;
    }
    case 'text':
      if (typeof value === 'string') return value;
      if (value === null || value === undefined) return '';
      return typeof value === 'object' ? JSON.stringify(value) : String(value);
    case 'boolean': return value === true || value === 'true';
    case 'null': return null;
    case 'json': return typeof value === 'object' && value !== null ? value : {};
  }
}

const VALUE_KINDS: PatchValueKind[] = ['number', 'text', 'boolean', 'null', 'json'];

/** The value of one patch row with its JSON type chosen explicitly. */
function PatchValueField({ value, fresh, onChange }: { value: unknown; fresh: boolean; onChange: (value: unknown) => void }) {
  const { t } = useI18n();
  // A row added now (null) starts as a number, the usual change; a saved
  // null reopens as null.
  const [kind, setKind] = useState<PatchValueKind>(() => (fresh && value === null ? 'number' : patchValueKind(value)));
  const [text, setText] = useState(() => (kind === 'json' ? JSON.stringify(value)
    : typeof value === 'number' ? String(value) : ''));
  return <>
    <label>{t('mwa.patch.valueType')}
      <select value={kind} onChange={(e) => {
        const next = e.target.value as PatchValueKind;
        const converted = convertPatchValue(value, next);
        setKind(next);
        setText(next === 'json' ? JSON.stringify(converted) : typeof converted === 'number' ? String(converted) : '');
        onChange(converted);
      }}>
        {VALUE_KINDS.map((item) => <option key={item} value={item}>{t(`mwa.patch.type.${item}`)}</option>)}
      </select>
    </label>
    {kind === 'number' && <label>{t('mwa.patch.value')}
      <input inputMode="decimal" value={text} onChange={(e) => {
        setText(e.target.value);
        onChange(convertPatchValue(e.target.value, 'number'));
      }} />
    </label>}
    {kind === 'text' && <label>{t('mwa.patch.value')}
      <input value={typeof value === 'string' ? value : ''} onChange={(e) => onChange(e.target.value)} />
    </label>}
    {kind === 'boolean' && <label>{t('mwa.patch.value')}
      <select value={value === true ? 'true' : 'false'} onChange={(e) => onChange(e.target.value === 'true')}>
        <option value="true">true</option>
        <option value="false">false</option>
      </select>
    </label>}
    {kind === 'json' && <label>{t('mwa.patch.value')}
      <textarea rows={3} value={text} aria-invalid={patchValueKind(parsePatchValue(text)) !== 'json'} onChange={(e) => {
        setText(e.target.value);
        const parsed = parsePatchValue(e.target.value);
        if (patchValueKind(parsed) === 'json') onChange(parsed);
      }} />
    </label>}
  </>;
}

/** The RFC 6902 operations of a measure, one row per operation: op, path and value. */
export function MeasurePatchFields({ patch, onChange }: {
  patch: MwaPatchOperation[];
  onChange: (patch: MwaPatchOperation[]) => void;
}) {
  const { t } = useI18n();
  // Stable row keys, so a row's local value state stays with its row when
  // another row is removed; rows added in this editor are marked fresh.
  const keys = useRef<{ next: number; ids: number[]; fresh: Set<number> }>({ next: 0, ids: [], fresh: new Set() });
  const rows = keys.current;
  while (rows.ids.length < patch.length) rows.ids.push(rows.next++);
  if (rows.ids.length > patch.length) rows.ids.length = patch.length;
  const update = (index: number, operation: MwaPatchOperation) =>
    onChange(patch.map((item, other) => (other === index ? operation : item)));
  return (
    <fieldset className="mwa-wide mwa-patch">
      <legend>{t('mwa.measure.patch')}</legend>
      {patch.map((operation, index) => (
        <div key={rows.ids[index]} className="mwa-patch-row">
          <label>{t('mwa.patch.op')}
            <select value={operation.op} onChange={(e) => {
              const op = e.target.value as MwaPatchOperation['op'];
              update(index, op === 'remove' ? { op, path: operation.path }
                : { op, path: operation.path, value: 'value' in operation ? operation.value : null });
            }}>
              <option value="replace">{t('mwa.patch.replace')}</option>
              <option value="add">{t('mwa.patch.add')}</option>
              <option value="remove">{t('mwa.patch.remove')}</option>
            </select>
          </label>
          <label>{t('mwa.patch.path')}
            <input value={operation.path} placeholder="/ntaCalculation/…"
              onChange={(e) => update(index, { ...operation, path: e.target.value })} />
          </label>
          {operation.op !== 'remove' && <PatchValueField value={operation.value} fresh={rows.fresh.has(rows.ids[index])}
            onChange={(value) => update(index, { ...operation, value })} />}
          <button type="button" className="btn" onClick={() => {
            rows.ids.splice(index, 1);
            onChange(patch.filter((_, other) => other !== index));
          }}>{t('mwa.remove')}</button>
        </div>
      ))}
      <button type="button" className="btn" onClick={() => {
        const id = rows.next++;
        rows.ids.push(id);
        rows.fresh.add(id);
        onChange([...patch, { op: 'replace', path: '', value: null }]);
      }}>
        {t('mwa.patch.addOperation')}
      </button>
    </fieldset>
  );
}


function MeasureEditor({ project, measure, onChange, onRemove }: {
  project: IProject;
  measure: MwaMeasure;
  onChange: (measure: MwaMeasure) => void;
  onRemove: () => void;
}) {
  const { t } = useI18n();
  const num = (value: string) => (value.trim() === '' ? undefined : Number(value));
  const kind = measure.template?.kind ?? 'manual';
  return (
    <fieldset className="mwa-measure">
      <legend>{measure.name || measure.id}</legend>
      <label className="mwa-measure-kind">{t('mwa.template.kind')}
        <select value={kind} title={t(kind === 'manual' ? 'mwa.template.kind.manual' : `mwa.template.kind.${kind}`)} onChange={(e) => {
          const next = e.target.value;
          onChange(next === 'manual' ? applyTemplate(project, measure, null) : templateMeasure(project, measure, next as MwaTemplateKind));
        }}>
          {TEMPLATE_KINDS.map((item) => <option key={item} value={item}>{t(`mwa.template.kind.${item}`)}</option>)}
          <option value="manual">{t('mwa.template.kind.manual')}</option>
        </select>
      </label>
      <label>{t('mwa.measure.name')}
        <input value={measure.name} onChange={(e) => onChange({ ...measure, name: e.target.value })} />
      </label>
      <label>{t('mwa.measure.category')}
        <select value={measure.category} onChange={(e) => onChange({ ...measure, category: e.target.value as MwaMeasureCategory })}>
          {CATEGORIES.map((item) => <option key={item} value={item}>{t(`mwa.category.${item}`)}</option>)}
        </select>
      </label>
      <label>{t('mwa.measure.target')}
        <select value={measure.target} onChange={(e) => onChange({ ...measure, target: e.target.value as MwaMeasure['target'] })}>
          <option value="project">{t('mwa.target.project')}</option>
          <option value="building">{t('mwa.target.building')}</option>
        </select>
      </label>
      <label>{t('mwa.measure.investment')}
        <input type="number" value={measure.investmentEur} onChange={(e) => onChange({ ...measure, investmentEur: Number(e.target.value) })} />
      </label>
      {measure.investmentEur === 0 && <p className="mwa-wide nta-form-note" data-testid={`mwa-investment-zero-${measure.id}`}>
        {t('mwa.measure.investmentZero')}</p>}
      <label>{t('mwa.measure.costSource')}
        <input value={measure.costSource} onChange={(e) => onChange({ ...measure, costSource: e.target.value })} />
      </label>
      <label>{t('mwa.measure.lifetime')}
        <input type="number" value={measure.lifetimeYears} onChange={(e) => onChange({ ...measure, lifetimeYears: Number(e.target.value) })} />
      </label>
      <label>{t('mwa.measure.maintenance')}
        <input type="number" value={measure.maintenanceEurPerYear ?? ''} onChange={(e) => onChange({ ...measure, maintenanceEurPerYear: num(e.target.value) })} />
      </label>
      <label>{t('mwa.measure.phaseYear')}
        <input type="number" value={measure.phaseYear ?? ''} onChange={(e) => onChange({ ...measure, phaseYear: num(e.target.value) })} />
      </label>
      <label className="mwa-wide">{t('mwa.measure.specialist')}
        <input value={measure.specialistNote ?? ''} onChange={(e) => onChange({ ...measure, specialistNote: e.target.value || undefined })} />
      </label>
      {measure.template
        ? <MwaTemplateEditor project={project} measure={measure} onChange={onChange} />
        : <MeasurePatchFields patch={measure.patch} onChange={(patch) => onChange({ ...measure, patch })} />}
      <button type="button" className="btn" onClick={onRemove}>{t('mwa.remove')}</button>
    </fieldset>
  );
}

type ResultOrder = 'input' | 'payback' | 'npv';

/** ISSO 82.2 §6.2.4 prescribes no ranking; the adviser may sort by payback or NPV. */
export function orderResults(results: MwaVariantResult[], order: ResultOrder): MwaVariantResult[] {
  if (order === 'input') return results;
  const key = (item: MwaVariantResult) => (order === 'payback' ? item.simplePaybackYears : item.netPresentValueEur);
  return [...results].sort((a, b) => {
    const left = key(a);
    const right = key(b);
    if (left == null || !Number.isFinite(left)) return right == null || !Number.isFinite(right) ? 0 : 1;
    if (right == null || !Number.isFinite(right)) return -1;
    return order === 'payback' ? left - right : right - left;
  });
}

/** A variant row; an invalid variant gets a second row with the kernel's reasons. */
export function ResultRow({ result, t }: { result: MwaVariantResult; t: (key: string) => string }) {
  const use = result.actualUse;
  const name = result.kind === 'current' ? t('mwa.current') : result.name;
  return (
    <>
    <tr data-invalid={result.valid ? undefined : 'true'}>
      <th scope="row">{name}{!result.valid && <span aria-label={t('mwa.variantInvalid')} title={t('mwa.variantInvalid')}> ⚠</span>}</th>
      <td>{result.label.labelClass ?? '–'}</td>
      <td>{format(result.label.primaryFossilIndicatorKwhPerM2, 1)}</td>
      <td>{format(use?.gasM3)}</td>
      <td>{format(use ? use.electricityImportKwh - use.electricityExportKwh : null)}</td>
      <td>{format(use?.co2Kg)}</td>
      <td>{format(use?.energyCostEur)}</td>
      <td>{format(result.savings?.energyCostEur)}</td>
      <td>{format(result.investmentEur)}</td>
      <td>{format(result.simplePaybackYears, 1)}</td>
      <td>{format(result.netPresentValueEur)}</td>
    </tr>
    {result.issues.length > 0 && (
      <tr className="mwa-variant-issues" data-testid={`mwa-issues-${result.id}`}>
        <td colSpan={11}>
          <ul>{result.issues.map((item, index) => (
            <li key={index}><KernelCode code={item.code} prefixes={['mwa.issue.', 'nta.gap.', 'kernel.issue.']} />
              {item.detail && <> {item.code === 'measure_template_incomplete'
                ? item.detail.split(', ').map((key) => t(`mwa.template.problem.${key}`)).join(' ')
                : <KernelDetail detail={item.detail} />}</>}
              {item.path && <> <code className="kernel-code-ref" title={item.path}>{item.path}</code></>}</li>
          ))}</ul>
        </td>
      </tr>
    )}
    </>
  );
}

/** Maatwerkadvies (BRL 9500-MWA, ISSO 82.2/75.2): measures, packages, results and advice. */
/** Measures to tick for a passport requirement (step 1 measures). */
function MeasureChecks({ label, measures, selected, onChange }: {
  label: string;
  measures: Array<{ id: string; name: string }>;
  selected: string[];
  onChange: (ids: string[]) => void;
}) {
  return (
    <div className="mwa-wide mwa-checks">
      <em>{label}</em>
      {measures.map((measure) => (
        <label key={measure.id} className="mwa-check">
          <input type="checkbox" checked={selected.includes(measure.id)}
            onChange={(e) => onChange(e.target.checked
              ? [...selected, measure.id]
              : selected.filter((id) => id !== measure.id))} />
          {measure.name || measure.id}
        </label>
      ))}
    </div>
  );
}

/** Tabs of the Maatwerkadvies step (ontwerp §8): maatregelen & pakketten, gemeten verbruik, woningpas, advies & rapport. */
export type MwaTab = 'measures' | 'use' | 'passport' | 'advice';

export function MaatwerkadviesPanel({ tab }: { tab?: MwaTab } = {}) {
  const { state, dispatch } = useEnergy();
  const { t } = useI18n();
  const project = state.project;
  const definition = project.maatwerkadvies;
  const [assessment, setAssessment] = useState<MaatwerkadviesAssessment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [order, setOrder] = useState<ResultOrder>('input');
  // The measure open in the side sheet (tab layout).
  const [editing, setEditing] = useState<string | null>(null);

  const update = (next: NtaMaatwerkadvies) => {
    dispatch({ type: 'SET_MAATWERKADVIES', payload: next });
    setAssessment(null);
  };

  if (!definition) {
    if (tab) {
      return <EmptyState icon={<ClipboardList />} title={t('mwa.title')}
        action={<Button variant="primary" onClick={() => update(emptyMaatwerkadvies())}>{t('mwa.start')}</Button>}>
        {t('mwa.intro')}</EmptyState>;
    }
    return (
      <section className="nta-performance mwa-panel" aria-label={t('mwa.title')}>
        <div className="nta-performance-title"><ClipboardList size={18} /><div><h3>{t('mwa.title')}</h3><p>{t('mwa.intro')}</p></div></div>
        <div className="nta-performance-actions">
          <button type="button" className="btn btn-primary" onClick={() => update(emptyMaatwerkadvies())}>{t('mwa.start')}</button>
        </div>
      </section>
    );
  }

  const use = definition.currentUse ?? { profile: 'nta' as MwaUserProfile, sourceReference: '' };
  const measuredUse: MwaMeasuredUse = definition.measured ?? {};
  const setMeasured = (patch: Partial<MwaMeasuredUse>) => update({ ...definition, measured: { ...measuredUse, ...patch } });
  const passport = definition.renovationPassport;
  // BRL 9500-MWA-W §3.2 for dwellings, -U §3.2 for utility buildings.
  const residentialPassport = project.buildingFunction === 'residential';
  const setPassport = (patch: Partial<MwaRenovationPassportInput>) => {
    if (passport) update({ ...definition, renovationPassport: { ...passport, ...patch } });
  };
  const verdictText = (value: boolean | null | undefined) =>
    value == null ? t('mwa.verdict.unknown') : value ? t('mwa.verdict.pass') : t('mwa.verdict.fail');
  const setMeasure = (index: number, measure: MwaMeasure) =>
    update({ ...definition, measures: definition.measures.map((item, i) => (i === index ? measure : item)) });
  const removeMeasure = (index: number) => {
    const id = definition.measures[index].id;
    update({
      ...definition,
      measures: definition.measures.filter((_, i) => i !== index),
      packages: definition.packages.map((item) => ({ ...item, measureIds: item.measureIds.filter((m) => m !== id) })),
    });
  };
  // A new measure starts from the insulation template; "manual" keeps the
  // expert patch rows.
  const addMeasure = () => {
    const id = nextId('m', definition.measures.map((item) => item.id));
    update({
      ...definition,
      measures: [...definition.measures, templateMeasure(project, {
        id, name: '', category: 'insulation', target: 'project', patch: [],
        investmentEur: 0, costSource: '', lifetimeYears: 30,
      }, 'insulation')],
    });
    return id;
  };
  const setPackage = (index: number, value: MwaPackage) =>
    update({ ...definition, packages: definition.packages.map((item, i) => (i === index ? value : item)) });
  const addPackage = () => update({
    ...definition,
    packages: [...definition.packages, { id: nextId('p', definition.packages.map((item) => item.id)), name: '', measureIds: [] }],
  });

  const calculate = async () => {
    setBusy(true);
    setError(null);
    try {
      // Template patches use array indices; regenerate them against the
      // project as it is now.
      const current = regenerateTemplatePatches(project, definition);
      if (JSON.stringify(current) !== JSON.stringify(definition)) dispatch({ type: 'SET_MAATWERKADVIES', payload: current });
      setAssessment(await assessMaatwerkadviesWithRust(project, current));
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      setBusy(false);
    }
  };

  const advice = assessment?.advice;
  const chosen = assessment?.packages.find((item) => item.id === advice?.packageId);

  // Old layout (all sections in one column) without `tab`; the step page passes a tab (F8).
  const block = (summary: React.ReactNode, open: boolean, body: React.ReactNode) => (tab
    ? <Card title={summary} level={2} className="mwa-block">{body}</Card>
    : <details open={open}><summary>{summary}</summary>{body}</details>);

  const useBlock = block(t('mwa.use'), true, <>
    <div className="mwa-grid">
      <label>{t('mwa.profile')}
        <select value={use.profile} onChange={(e) => update({ ...definition, currentUse: { ...use, profile: e.target.value as MwaUserProfile } })}>
          {PROFILES.map((item) => <option key={item} value={item}>{t(`mwa.profile.${item}`)}</option>)}
        </select>
      </label>
      <label>{t('mwa.occupants')}
        <input type="number" value={use.occupants ?? ''}
          onChange={(e) => update({ ...definition, currentUse: { ...use, occupants: e.target.value === '' ? undefined : Number(e.target.value) } })} />
      </label>
      <label>{t('mwa.heatingSetpoint')}
        <input type="number" value={use.heatingSetpointC ?? ''}
          onChange={(e) => update({ ...definition, currentUse: { ...use, heatingSetpointC: e.target.value === '' ? undefined : Number(e.target.value) } })} />
      </label>
      <label>{t('mwa.persons')}
        <input type="number" value={use.persons ?? ''}
          onChange={(e) => update({ ...definition, currentUse: { ...use, persons: e.target.value === '' ? undefined : Number(e.target.value) } })} />
      </label>
      <label>{t('mwa.lightingHours')}
        <input type="number" step="0.05" value={use.lightingHoursFactor ?? ''}
          onChange={(e) => update({ ...definition, currentUse: { ...use, lightingHoursFactor: e.target.value === '' ? undefined : Number(e.target.value) } })} />
      </label>
      <label className="mwa-wide">{t('mwa.source')}
        <input value={use.sourceReference} onChange={(e) => update({ ...definition, currentUse: { ...use, sourceReference: e.target.value } })} />
      </label>
    </div>
  </>);

  const tariffsBlock = block(t('mwa.tariffs'), true, <>
    <div className="mwa-grid">
      <label>{t('mwa.tariff.gas')}
        <input type="number" step="0.01" value={definition.tariffs.gasEurPerM3}
          onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, gasEurPerM3: Number(e.target.value) } })} />
      </label>
      <label>{t('mwa.tariff.electricity')}
        <input type="number" step="0.01" value={definition.tariffs.electricityEurPerKwh}
          onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, electricityEurPerKwh: Number(e.target.value) } })} />
      </label>
      <label>{t('mwa.tariff.export')}
        <input type="number" step="0.01" value={definition.tariffs.electricityExportEurPerKwh ?? 0}
          onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, electricityExportEurPerKwh: Number(e.target.value) } })} />
      </label>
      <label>{t('mwa.tariff.heat')}
        <input type="number" step="0.01" value={definition.tariffs.districtHeatEurPerKwh ?? 0}
          onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, districtHeatEurPerKwh: Number(e.target.value) } })} />
      </label>
      <label>{t('mwa.discount')}
        <input type="number" step="0.005" value={definition.economics?.discountRate ?? 0.03}
          onChange={(e) => update({ ...definition, economics: { ...definition.economics, discountRate: Number(e.target.value) } })} />
      </label>
      <label className="mwa-wide">{t('mwa.source')}
        <input value={definition.tariffs.sourceReference}
          onChange={(e) => update({ ...definition, tariffs: { ...definition.tariffs, sourceReference: e.target.value } })} />
      </label>
    </div>
  </>);

  const measuredBlock = block(t('mwa.measured'), false, <>
    <div className="mwa-grid">
      <label>{t('mwa.measured.gas')}
        <input type="number" value={measuredUse.annualGasM3 ?? ''}
          onChange={(e) => setMeasured({ annualGasM3: e.target.value === '' ? undefined : Number(e.target.value) })} />
      </label>
      <label>{t('mwa.measured.electricity')}
        <input type="number" value={measuredUse.annualElectricityKwh ?? ''}
          onChange={(e) => setMeasured({ annualElectricityKwh: e.target.value === '' ? undefined : Number(e.target.value) })} />
      </label>
      <label>{t('mwa.measured.heat')}
        <input type="number" value={measuredUse.annualHeatKwh ?? ''}
          onChange={(e) => setMeasured({ annualHeatKwh: e.target.value === '' ? undefined : Number(e.target.value) })} />
      </label>
      <label className="mwa-wide">{t('mwa.measured.monthlyGas')}
        <input defaultValue={showMonthly(measuredUse.monthlyGasM3)}
          onBlur={(e) => setMeasured({ monthlyGasM3: parseMonthly(e.target.value) })} />
      </label>
      <label className="mwa-wide">{t('mwa.measured.monthlyElectricity')}
        <input defaultValue={showMonthly(measuredUse.monthlyElectricityKwh)}
          onBlur={(e) => setMeasured({ monthlyElectricityKwh: parseMonthly(e.target.value) })} />
      </label>
      <label className="mwa-wide">{t('mwa.measured.monthlyHeat')}
        <input defaultValue={showMonthly(measuredUse.monthlyHeatKwh)}
          onBlur={(e) => setMeasured({ monthlyHeatKwh: parseMonthly(e.target.value) })} />
      </label>
      <label className="mwa-wide">{t('mwa.measured.temperature')}
        <input defaultValue={showMonthly(measuredUse.monthlyOutdoorTemperatureC)}
          onBlur={(e) => setMeasured({ monthlyOutdoorTemperatureC: parseMonthly(e.target.value) })} />
      </label>
      <label className="mwa-wide">{t('mwa.source')}
        <input value={measuredUse.sourceReference ?? ''} onChange={(e) => setMeasured({ sourceReference: e.target.value })} />
      </label>
    </div>
  </>);

  const measuresBlock = block(<>{t('mwa.measures')} ({definition.measures.length})</>, true, <>
    <p>{t('mwa.patchHelp')}</p>
    {definition.measures.map((measure, index) => (
      <MeasureEditor key={measure.id} project={project} measure={measure}
        onChange={(value) => setMeasure(index, value)} onRemove={() => removeMeasure(index)} />
    ))}
    <button type="button" className="btn" onClick={addMeasure}>{t('mwa.addMeasure')}</button>
  </>);

  const packagesBlock = block(<>{t('mwa.packages')} ({definition.packages.length})</>, true, <>
    {definition.packages.map((item, index) => (
      <fieldset key={item.id} className="mwa-measure">
        <legend>{item.name || item.id}</legend>
        <label>{t('mwa.measure.name')}
          <input value={item.name} onChange={(e) => setPackage(index, { ...item, name: e.target.value })} />
        </label>
        <div className="mwa-wide mwa-checks">
          {definition.measures.map((measure) => (
            <label key={measure.id} className="mwa-check">
              <input type="checkbox" checked={item.measureIds.includes(measure.id)}
                onChange={(e) => setPackage(index, {
                  ...item,
                  measureIds: e.target.checked
                    ? [...item.measureIds, measure.id]
                    : item.measureIds.filter((id) => id !== measure.id),
                })} />
              {measure.name || measure.id}
            </label>
          ))}
        </div>
        <label className="mwa-wide">{t('mwa.partialWarning')}
          <input value={item.partialExecutionWarning ?? ''}
            onChange={(e) => setPackage(index, { ...item, partialExecutionWarning: e.target.value || undefined })} />
        </label>
        <button type="button" className="btn" onClick={() => update({ ...definition, packages: definition.packages.filter((_, i) => i !== index) })}>{t('mwa.remove')}</button>
      </fieldset>
    ))}
    <button type="button" className="btn" onClick={addPackage}>{t('mwa.addPackage')}</button>
    <label className="mwa-advised">{t('mwa.advised')}
      <select value={definition.advisedPackageId ?? ''}
        onChange={(e) => update({ ...definition, advisedPackageId: e.target.value || undefined })}>
        <option value="">{t('mwa.advised.automatic')}</option>
        {definition.packages.map((item) => <option key={item.id} value={item.id}>{item.name || item.id}</option>)}
      </select>
    </label>
  </>);

  const passportBlock = block(t('mwa.passport'), false, <>
    <label className="mwa-check">
      <input type="checkbox" checked={definition.renovationPassport != null}
        onChange={(e) => update({
          ...definition,
          renovationPassport: e.target.checked
            ? { demandPackageId: '', systemsPackageId: '', productionPackageId: '', overheatingMeasureIds: [] }
            : undefined,
        })} />
      {t('mwa.passport.enable')}
    </label>
    {passport && <div className="mwa-grid">
      {(['demandPackageId', 'systemsPackageId', 'productionPackageId'] as const).map((key) => (
        <label key={key}>{t(`mwa.passport.${key}`)}
          <select value={passport[key]} onChange={(e) => setPassport({ [key]: e.target.value })}>
            <option value="">–</option>
            {definition.packages.map((item) => <option key={item.id} value={item.id}>{item.name || item.id}</option>)}
          </select>
        </label>
      ))}
      {residentialPassport ? <>
        <p className="mwa-wide">{t('mwa.passport.schemeW')}</p>
        <label className="mwa-check">
          <input type="checkbox" checked={passport.insulationStandardMet ?? false}
            onChange={(e) => setPassport({ insulationStandardMet: e.target.checked })} />
          {t('mwa.passport.insulationStandard')}
        </label>
        <label className="mwa-check">
          <input type="checkbox" checked={passport.prewarStandard ?? false}
            onChange={(e) => setPassport({ prewarStandard: e.target.checked })} />
          {t('mwa.passport.prewar')}
        </label>
        {passport.prewarStandard && <label className="mwa-wide">{t('mwa.passport.prewarMotivation')}
          <input value={passport.prewarMotivation ?? ''} onChange={(e) => setPassport({ prewarMotivation: e.target.value })} />
        </label>}
        <label className="mwa-wide">{t('mwa.passport.gasFreeNotRealistic')}
          <input value={passport.gasFreeNotRealisticMotivation ?? ''}
            onChange={(e) => setPassport({ gasFreeNotRealisticMotivation: e.target.value || undefined })} />
        </label>
        <label className="mwa-check">
          <input type="checkbox" checked={passport.storageConsidered ?? false}
            onChange={(e) => setPassport({ storageConsidered: e.target.checked })} />
          {t('mwa.passport.storage')}
        </label>
        <MeasureChecks label={t('mwa.passport.overheating')} measures={definition.measures}
          selected={passport.overheatingMeasureIds ?? []}
          onChange={(ids) => setPassport({ overheatingMeasureIds: ids })} />
      </> : <>
        <p className="mwa-wide">{t('mwa.passport.schemeU')}</p>
        <label className="mwa-check">
          <input type="checkbox" checked={passport.lowTemperatureReady ?? false}
            onChange={(e) => setPassport({ lowTemperatureReady: e.target.checked })} />
          {t('mwa.passport.lowTemperatureReady')}
        </label>
        <label className="mwa-wide">{t('mwa.passport.facadeImpossible')}
          <input value={passport.facadeInsulationImpossibleMotivation ?? ''}
            onChange={(e) => setPassport({ facadeInsulationImpossibleMotivation: e.target.value || undefined })} />
        </label>
        <MeasureChecks label={t('mwa.passport.cooling')} measures={definition.measures}
          selected={passport.coolingMeasureIds ?? []}
          onChange={(ids) => setPassport({ coolingMeasureIds: ids })} />
      </>}
    </div>}
  </>);

  const actionsBar = (
    <div className="nta-performance-actions mwa-actions">
      <button type="button" className="btn btn-primary" onClick={calculate} disabled={busy}>{busy ? t('mwa.calculating') : t('mwa.calculate')}</button>
      <button type="button" className="btn" onClick={() => { void downloadMaatwerkadviesReportHTML(project).catch((reason: unknown) => setError(String(reason))); }}>
        {t('mwa.report')}
      </button>
    </div>
  );

  // Every variant is calculated in the base project's edition.
  const editionBlock = assessment && (
    <p className="mwa-edition">{t('mwa.edition')}: <strong>{t(`nta.edition.${assessment.normVersion ?? DEFAULT_NORM_VERSION}`)}</strong>
      {assessment.registrationEligible === false && <> — {t('nta.edition.legacyTitle')}</>}</p>
  );

  const issuesBlock = assessment && assessment.issues.length > 0 && (
    <ul className="nta-performance-gaps">
      {assessment.issues.map((item, index) => <li key={index}><KernelCode code={item.code} prefixes={['mwa.issue.', 'nta.gap.', 'kernel.issue.']} /> <code>{item.path}</code> <KernelDetail detail={item.detail} /></li>)}
    </ul>
  );

  const tableBlock = assessment?.current && (
    <div className={tab ? 'ui-table-wrap' : 'nta-performance-table'}>
      {!tab && <label>{t('mwa.order')}{' '}
        <select value={order} onChange={(event) => setOrder(event.target.value as ResultOrder)}>
          <option value="input">{t('mwa.order.input')}</option>
          <option value="payback">{t('mwa.order.payback')}</option>
          <option value="npv">{t('mwa.order.npv')}</option>
        </select>
      </label>}
      <table className={tab ? 'ui-table mwa-variants' : undefined}>
        <thead><tr>
          <th>{t('mwa.col.variant')}</th><th>{t('mwa.col.label')}</th><th>EP2</th><th>{t('mwa.col.gas')}</th>
          <th>{t('mwa.col.electricity')}</th><th>CO₂ [kg]</th><th>{t('mwa.col.cost')}</th><th>{t('mwa.col.saving')}</th>
          <th>{t('mwa.col.investment')}</th><th>{t('mwa.col.payback')}</th><th>{t('mwa.col.npv')}</th>
        </tr></thead>
        <tbody>
          <ResultRow result={assessment.current} t={t} />
          {orderResults(assessment.measures, order).map((item) => <ResultRow key={`m-${item.id}`} result={item} t={t} />)}
          {orderResults(assessment.packages, order).map((item) => <ResultRow key={`p-${item.id}`} result={item} t={t} />)}
        </tbody>
      </table>
    </div>
  );

  const adviceBlock = advice && (
    <div className="nta-performance-bbl">
      <strong>{t('mwa.advice')}: {chosen
        ? <>{chosen.name} — {advice.chosenBy === 'adviser' ? t('mwa.advised.adviser') : t('mwa.advised.automatic')}</>
        : t('mwa.advice.none')}</strong>
      {advice.warnings.length > 0 && <><em>{t('mwa.warnings')}</em><ul>{advice.warnings.map((item) => <li key={item}>{adviceText(item, i18next.language || 'nl')}</li>)}</ul></>}
      {advice.specialistNotes.length > 0 && <><em>{t('mwa.specialist')}</em><ul>{advice.specialistNotes.map((item) => <li key={item}>{adviceText(item, i18next.language || 'nl')}</li>)}</ul></>}
      {chosen && chosen.systemChecks.some((check) => check.limit != null) && <><em>{t('mwa.systemChecks')}</em><ul>
        {chosen.systemChecks.filter((check) => check.limit != null).map((check) => (
          <li key={check.system}>{t(`mwa.system.${check.system}`)}: {format(check.value, 2)} / {format(check.limit, 2)} {check.unit} — {verdictText(check.meets)}</li>
        ))}
      </ul></>}
    </div>
  );

  const fitBlock = assessment?.fitCheck && (
    <p className="mwa-fit">{t('mwa.fitCriteria')}: <strong>{verdictText(assessment.fitCheck.criteria.withinCriteria)}</strong></p>
  );

  const passportResultBlock = assessment?.renovationPassport && (
    <div className="nta-performance-bbl">
      <strong>{t('mwa.passport')}: {verdictText(assessment.renovationPassport.eligible)}</strong>
      {assessment.registrationType && <p>{t('mwa.registrationType')}: {t(`mwa.registrationType.${assessment.registrationType}`)}</p>}
      <ul>{assessment.renovationPassport.requirements.map((item) => (
        <li key={item.code}>{t(`mwa.passport.req.${item.code}`)}: {verdictText(item.met)}</li>
      ))}</ul>
    </div>
  );

  if (!tab) {
    return (
      <section className="nta-performance mwa-panel" aria-label={t('mwa.title')}>
        <div className="nta-performance-head">
          <div className="nta-performance-title"><ClipboardList size={18} /><div><h3>{t('mwa.title')}</h3><p>{t('mwa.intro')}</p></div></div>
          <span className="nta-performance-badge">{t('mwa.unverified')}</span>
        </div>
        {useBlock}
        {tariffsBlock}
        {measuredBlock}
        {measuresBlock}
        {packagesBlock}
        {passportBlock}
        {actionsBar}
        {error && <p role="alert">{error}</p>}
        {assessment && (
          <div className="mwa-results">
            {editionBlock}
            {issuesBlock}
            {tableBlock}
            {adviceBlock}
            {fitBlock}
            {passportResultBlock}
          </div>
        )}
      </section>
    );
  }

  const advisedId = advice?.packageId ?? definition.advisedPackageId ?? null;
  const editingIndex = editing == null ? -1 : definition.measures.findIndex((item) => item.id === editing);
  const exportReport = () => { void downloadMaatwerkadviesReportHTML(project).catch((reason: unknown) => setError(String(reason))); };
  const togglePackage = (packageIndex: number, measureId: string) => {
    const item = definition.packages[packageIndex];
    setPackage(packageIndex, {
      ...item,
      measureIds: item.measureIds.includes(measureId)
        ? item.measureIds.filter((id) => id !== measureId)
        : [...item.measureIds, measureId],
    });
  };
  const notCalculated = <Banner tone="info" role="status"
    action={<Button size="sm" onClick={() => { void calculate(); }} disabled={busy}>{t('mwa.calculate')}</Button>}>
    {t('mwa.page.notCalculated')}</Banner>;

  return (
    <section className="mwa-page" aria-label={t('mwa.title')}>
      <div className="mwa-toolbar">
        <Pill tone="unv">{t('mwa.unverified')}</Pill>
        <span className="mwa-toolbar__note">{t('mwa.page.basis')}</span>
        <Button variant="primary" icon={<RefreshCw aria-hidden="true" />} loading={busy} disabled={busy}
          onClick={() => { void calculate(); }}>{busy ? t('mwa.calculating') : t('mwa.calculate')}</Button>
      </div>
      {error && <Banner tone="err">{error}</Banner>}

      {tab === 'measures' && <>
        {issuesBlock && <Card title={t('mwa.page.issues')} level={2}>{issuesBlock}</Card>}
        {assessment ? <div className="mwa-overview">
          <Card title={t('mwa.page.labelPath')} subtitle={t('mwa.page.labelPathSub')} level={2}>
            {editionBlock}
            {assessment.current
              ? <LabelPathChart bars={labelPathBars(assessment, advisedId, t('mwa.current'))} />
              : <p className="nta-form-note">{t('mwa.page.noVariants')}</p>}
          </Card>
          <Card title={t('mwa.page.compare')} aria-label={t('mwa.page.compare')} level={2}>
            {assessment.packages.length > 0
              ? <PackageComparison packages={orderResults(assessment.packages, order)} advisedId={advisedId}
                order={order} onOrder={setOrder} />
              : <p className="nta-form-note">{t('mwa.page.noPackages')}</p>}
            <Banner tone="info" role="note">{t('mwa.page.actualUseNote')}</Banner>
          </Card>
        </div> : notCalculated}
        <Card title={<>{t('mwa.measures')} ({definition.measures.length})</>} subtitle={t('mwa.page.measuresSub')} level={2}
          actions={<Button size="sm" icon={<Plus aria-hidden="true" />} onClick={() => setEditing(addMeasure())}>{t('mwa.addMeasure')}</Button>}>
          {definition.measures.length === 0
            ? <EmptyState title={t('mwa.page.noMeasures')}>{t('mwa.patchHelp')}</EmptyState>
            : <ul className="mwa-cards">
              {definition.measures.map((measure, index) => (
                <MeasureCard key={measure.id} measure={measure} packages={definition.packages}
                  problems={measureProblems(project, measure)}
                  result={assessment?.measures.find((item) => item.id === measure.id)}
                  kinds={TEMPLATE_KINDS}
                  onKind={(next) => setMeasure(index, next === 'manual'
                    ? applyTemplate(project, measure, null) : templateMeasure(project, measure, next))}
                  onTogglePackage={(packageIndex) => togglePackage(packageIndex, measure.id)}
                  onEdit={() => setEditing(measure.id)} onRemove={() => removeMeasure(index)} />
              ))}
            </ul>}
        </Card>
        {packagesBlock}
      </>}

      {tab === 'use' && <div className="mwa-columns">
        {useBlock}
        {tariffsBlock}
        {measuredBlock}
        {fitBlock && <Card title={t('mwa.page.fit')} level={2}>{fitBlock}</Card>}
      </div>}

      {tab === 'passport' && <>
        {passportBlock}
        {passportResultBlock
          ? <Card title={t('mwa.page.passportResult')} level={2}>{passportResultBlock}</Card>
          : definition.renovationPassport && !assessment && notCalculated}
      </>}

      {tab === 'advice' && <>
        {assessment ? <div className="mwa-advice">
          <div className="mwa-advice__main">
            {issuesBlock && <Card title={t('mwa.page.issues')} level={2}>{issuesBlock}</Card>}
            {tableBlock && <Card title={t('mwa.page.variants')} level={2} flush
              actions={<Segmented<ResultOrder> size="sm" aria-label={t('mwa.order')} value={order} onChange={setOrder} options={[
                { value: 'npv', label: t('mwa.page.order.npv') },
                { value: 'payback', label: t('mwa.page.order.payback') },
                { value: 'input', label: t('mwa.page.order.input') },
              ]} />}>{tableBlock}</Card>}
            {adviceBlock && <Card title={t('mwa.advice')} level={2}>{adviceBlock}</Card>}
          </div>
          {chosen && advice && <PackageInspector current={assessment.current} chosen={chosen} chosenBy={advice.chosenBy} />}
        </div> : notCalculated}
        <Card title={t('mwa.page.report')} level={2}
          actions={<Button icon={<FileDown aria-hidden="true" />} onClick={exportReport}>{t('mwa.report')}</Button>}>
          <p className="nta-form-note">{t('mwa.page.reportSub')}</p>
        </Card>
      </>}

      {editingIndex >= 0 && <SideSheet width={640} onClose={() => setEditing(null)}
        title={definition.measures[editingIndex].name || definition.measures[editingIndex].id}
        footer={<Button variant="primary" onClick={() => setEditing(null)}>{t('mwa.page.done')}</Button>}>
        <MeasureEditor project={project} measure={definition.measures[editingIndex]}
          onChange={(value) => setMeasure(editingIndex, value)}
          onRemove={() => { setEditing(null); removeMeasure(editingIndex); }} />
      </SideSheet>}
    </section>
  );
}
