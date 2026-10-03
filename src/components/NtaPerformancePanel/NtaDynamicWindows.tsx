import type { IProject } from '../../core/energy/types';
import { useI18n } from '../../i18n/i18n';
import { NumberField, read, SelectField, TextField, type Draft, type Path } from './NtaFormFields';

// Annex A dynamic transparent elements (`dynamicWindows`): per project
// window method A (monthly state weights, A.1–A.4) or method B (one state),
// each with optional step-2 correction factors (p. 770).

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

const MONTHS = 12;

function newState(index: number): Draft {
  return { id: `toestand-${index + 1}`, gPerpendicular: null, uValueWPerM2k: null };
}

function emptyRows(columns: number): Array<Array<number | null>> {
  return Array.from({ length: MONTHS }, () => Array.from({ length: columns }, () => null));
}

export function dynamicWindowTemplate(windowId: string): Draft {
  return { windowId, dynamic: { method: 'single_state', state: newState(0), sourceReference: '' } };
}

/**
 * One `U_dyn;i`/`g_dyn;i` state. τ_sol/τ_vis (A.3/A.4) are not asked:
 * chapter 14 has no input for them (14.38 has no transmittance term, 14.41
 * fixes τ_D65 = 0,6), so they would not change the result.
 */
function StateFields({ draft, change, base }: SectionProps & { base: Path }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  return <div className="nta-form-row">
    <TextField {...field} path={[...base, 'id']} label={t('nta.form.dynamic.stateId')} />
    <NumberField {...field} path={[...base, 'gPerpendicular']} label={t('nta.form.dynamic.g')} />
    <NumberField {...field} path={[...base, 'uValueWPerM2k']} label={t('nta.form.dynamic.u')} />
  </div>;
}

/** Twelve rows with one weight per state (A.1 or A.2). */
function WeightGrid({ draft, change, path, states, label }: SectionProps & { path: Path; states: Draft[]; label: string }) {
  const rows = (read(draft, path) as Array<Array<number | null>> | undefined) ?? [];
  return <fieldset className="nta-form-months"><legend>{label}</legend>
    {Array.from({ length: MONTHS }, (_, month) => <div key={month} className="nta-form-row">
      {states.map((state, column) => {
        const value = rows[month]?.[column];
        return <input key={column} type="number" step="any" aria-label={`${label} ${month + 1} ${String(state.id ?? column + 1)}`}
          value={typeof value === 'number' ? value : ''}
          onChange={(event) => change([...path, month, column], event.target.value === '' ? null : Number(event.target.value))} />;
      })}
    </div>)}
  </fieldset>;
}

function MonthlyFactors({ draft, change, path, label }: SectionProps & { path: Path; label: string }) {
  const values = (read(draft, path) as Array<number | null> | undefined) ?? [];
  return <fieldset className="nta-form-months"><legend>{label}</legend>
    {Array.from({ length: MONTHS }, (_, index) => <input key={index} type="number" step="any" aria-label={`${label} ${index + 1}`}
      value={typeof values[index] === 'number' ? values[index] as number : ''}
      onChange={(event) => change([...path, index], event.target.value === '' ? null : Number(event.target.value))} />)}
  </fieldset>;
}

function DynamicWindowFields({ draft, change, base, windowName }: SectionProps & { base: Path; windowName: string }) {
  const { t } = useI18n();
  const field = { draft, onChange: change };
  const dynamic: Path = [...base, 'dynamic'];
  const method = read(draft, [...dynamic, 'method']);
  const states = (read(draft, [...dynamic, 'states']) as Draft[] | undefined) ?? [];
  const correction = read(draft, [...dynamic, 'correction']) != null;
  const setMethod = (value: unknown) => {
    const source = read(draft, [...dynamic, 'sourceReference']) ?? '';
    const first = (read(draft, [...dynamic, 'state']) as Draft | undefined) ?? states[0] ?? newState(0);
    const kept = read(draft, [...dynamic, 'correction']);
    const extra = kept != null ? { correction: kept } : {};
    change(dynamic, value === 'weighted_states'
      ? { method: 'weighted_states', states: [first, newState(1)], solarWeights: emptyRows(2),
        temperatureWeights: emptyRows(2), sourceReference: source, ...extra }
      : { method: 'single_state', state: first, sourceReference: source, ...extra });
  };
  const resize = (next: Draft[], keep: (row: Array<number | null>) => Array<number | null>) => {
    const value = read(draft, dynamic) as Draft;
    const map = (rows: unknown) => ((rows as Array<Array<number | null>> | undefined) ?? emptyRows(states.length)).map(keep);
    change(dynamic, { ...value, states: next, solarWeights: map(value.solarWeights), temperatureWeights: map(value.temperatureWeights) });
  };
  return <fieldset className="nta-form-group"><legend>{windowName}</legend>
    <SelectField {...field} path={[...dynamic, 'method']} label={t('nta.form.dynamic.method')} options={[
      ['single_state', t('nta.form.dynamic.methodB')], ['weighted_states', t('nta.form.dynamic.methodA')]]}
      onChange={(_, value) => setMethod(value)} />
    {method === 'single_state' && <StateFields draft={draft} change={change} base={[...dynamic, 'state']} />}
    {method === 'weighted_states' && <>
      {states.map((_, index) => <div key={index}>
        <StateFields draft={draft} change={change} base={[...dynamic, 'states', index]} />
        {states.length > 1 && <button type="button" onClick={() => resize(states.filter((__, other) => other !== index),
          (row) => row.filter((__, other) => other !== index))}>{t('nta.form.dynamic.removeState')}</button>}
      </div>)}
      <button type="button" onClick={() => resize([...states, newState(states.length)], (row) => [...row, null])}>
        {t('nta.form.dynamic.addState')}</button>
      <p className="nta-form-note">{t('nta.form.dynamic.weightsHelp')}</p>
      <WeightGrid draft={draft} change={change} path={[...dynamic, 'solarWeights']} states={states} label={t('nta.form.dynamic.solarWeights')} />
      <WeightGrid draft={draft} change={change} path={[...dynamic, 'temperatureWeights']} states={states} label={t('nta.form.dynamic.temperatureWeights')} />
    </>}
    <TextField {...field} path={[...dynamic, 'sourceReference']} label={t('nta.form.source')} />
    <label className="nta-form-check">
      <input type="checkbox" checked={correction} onChange={(event) => change([...dynamic, 'correction'], event.target.checked
        ? { uFactors: Array(MONTHS).fill(null), gFactors: Array(MONTHS).fill(null), sourceReference: '' } : null)} />
      {t('nta.form.dynamic.correction')}
    </label>
    {correction && <>
      <MonthlyFactors draft={draft} change={change} path={[...dynamic, 'correction', 'uFactors']} label={t('nta.form.dynamic.uFactors')} />
      <MonthlyFactors draft={draft} change={change} path={[...dynamic, 'correction', 'gFactors']} label={t('nta.form.dynamic.gFactors')} />
      <TextField {...field} path={[...dynamic, 'correction', 'sourceReference']} label={t('nta.form.dynamic.correctionSource')} />
    </>}
  </fieldset>;
}

/**
 * Annex A per project window; windows not listed keep their fixed U and g.
 * Only surfaces with an explicit outdoor boundary: a surface without one is
 * skipped by the project route (gap), so its windows cannot be dynamic.
 */
export function DynamicWindowsFields({ draft, change, project }: SectionProps & { project: IProject }) {
  const { t } = useI18n();
  const windows = project.zones.flatMap((zone) => zone.surfaces)
    .filter((surface) => surface.thermalBoundary === 'outdoor')
    .flatMap((surface) => surface.windows);
  const entries = (read(draft, ['dynamicWindows']) as Draft[] | undefined) ?? [];
  const name = (id: unknown) => windows.find((window) => window.id === id)?.name || String(id);
  const free = windows.filter((window) => !entries.some((entry) => entry.windowId === window.id));
  return <>
    <p className="nta-form-note">{t('nta.form.dynamic.help')}</p>
    {entries.map((entry, index) => <div key={String(entry.windowId)}>
      <DynamicWindowFields draft={draft} change={change} base={['dynamicWindows', index]} windowName={name(entry.windowId)} />
      <button type="button" onClick={() => change(['dynamicWindows'], entries.filter((_, other) => other !== index))}>
        {t('nta.form.dynamic.remove')}</button>
    </div>)}
    {free.length > 0 && <label>{t('nta.form.dynamic.add')}
      <select value="" onChange={(event) => event.target.value
        && change(['dynamicWindows'], [...entries, dynamicWindowTemplate(event.target.value)])}>
        <option value="">—</option>
        {free.map((window) => <option key={window.id} value={window.id}>{window.name || window.id}</option>)}
      </select>
    </label>}
  </>;
}
