import type { IProject } from '../../core/energy/types';
import { useI18n } from '../../i18n/i18n';
import { NumberField, read, TextField, type Draft, type Path } from './NtaFormFields';
import { outdoorWindows, projectWindow } from './NtaWindowObstructions';

// Glazing details per project window (`windowGlazings`): 7.6.6.1.2/7.6.6.1.3
// give g_gl per window from a table 7.4 type, fixed louvres (7.41a/7.41b) or
// the ISO 15099 values of diffusing glazing (7.41). A window that is not
// listed uses its own g-value as g_gl;n.

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

const SAME = '';
const DIFFUSING = 'diffusing';
const DETAILS = 'details';

/** Table 7.4 types (`glazingType`). */
export const GLAZING_TYPES = [
  'single', 'double', 'double_low_e', 'triple_one_coating', 'triple_two_coatings',
  'single_with_secondary_pane', 'solar_control',
] as const;

/** Fixed louvres of 7.41a (table 7.4a). */
export const FIXED_LOUVRES = ['horizontal90', 'horizontal_angled'] as const;

/**
 * The `windowGlazings` list after choosing for one window: '' is "as the
 * project" (the window's g-value), 'diffusing' the 7.41 values, 'details'
 * a table 7.4 type and/or fixed louvres.
 */
export function withWindowGlazing(entries: Draft[], windowId: string, choice: string): Draft[] {
  const index = entries.findIndex((entry) => entry.windowId === windowId);
  if (choice === SAME) return entries.filter((_, other) => other !== index);
  const current = index < 0 ? undefined : entries[index];
  const sourceReference = current?.sourceReference ?? '';
  const glazing: Draft = choice === DIFFUSING
    ? { diffusing: { gAltitude45: null, gDiffuse: null, sourceReference: '' } }
    : {};
  const entry: Draft = { windowId, glazing, sourceReference };
  if (index < 0) return [...entries, entry];
  return entries.map((item, other) => other === index ? entry : item);
}

/** The glazing of one entry with a table 7.4 type ('' removes it). */
export function withGlazingType(glazing: Draft, value: string): Draft {
  const { glazingType: _type, ...rest } = glazing;
  return value ? { ...rest, glazingType: value } : rest;
}

/** The glazing of one entry with fixed louvres ('' removes them). */
export function withFixedLouvres(glazing: Draft, value: string): Draft {
  const { fixedLouvres: _louvres, ...rest } = glazing;
  return value ? { ...rest, fixedLouvres: { kind: value } } : rest;
}

export function WindowGlazingsFields({ draft, change, project }: SectionProps & { project: IProject }) {
  const { t } = useI18n();
  const entries = (read(draft, ['windowGlazings']) as Draft[] | undefined) ?? [];
  const windows = outdoorWindows(project);
  const label = (window: { id: string; name?: string }) => window.name || window.id;
  const choiceOf = (entry: Draft | undefined) => {
    if (!entry) return SAME;
    const glazing = entry.glazing as Draft | undefined;
    return glazing?.diffusing ? DIFFUSING : DETAILS;
  };
  return <>
    <p className="nta-form-note">{t('nta.form.windowGlazings.help')}</p>
    <div className="nta-performance-table"><table>
      <thead><tr>
        <th>{t('nta.form.windowObstructions.window')}</th>
        <th>{t('nta.form.windowObstructions.surface')}</th>
        <th>{t('nta.form.windowGlazings.glazing')}</th>
      </tr></thead>
      <tbody>
        {windows.map(({ window, surface }) => {
          const entry = entries.find((item) => item.windowId === window.id);
          return <tr key={window.id}>
            <td>{label(window)}</td>
            <td>{surface.name || surface.id}</td>
            <td>
              <select aria-label={`${t('nta.form.windowGlazings.glazing')} ${label(window)}`} value={choiceOf(entry)}
                onChange={(event) => change(['windowGlazings'], withWindowGlazing(entries, window.id, event.target.value))}>
                <option value={SAME}>{t('nta.form.windowGlazings.same')}</option>
                <option value={DETAILS}>{t('nta.form.windowGlazings.details')}</option>
                <option value={DIFFUSING}>{t('nta.form.windowGlazings.diffusing')}</option>
              </select>
            </td>
          </tr>;
        })}
      </tbody>
    </table></div>
    {entries.map((entry, index) => {
      const found = windows.find(({ window }) => window.id === entry.windowId);
      const elsewhere = found ? null : projectWindow(project, entry.windowId);
      const note = elsewhere
        ? t('nta.form.windowObstructions.notOutdoor')
        : t('nta.form.windowObstructions.missing');
      const glazing = (entry.glazing as Draft | undefined) ?? {};
      const base: Path = ['windowGlazings', index, 'glazing'];
      const louvres = (glazing.fixedLouvres as Draft | undefined)?.kind;
      return <fieldset key={`${String(entry.windowId)}-${index}`} className="nta-form-group"
        data-path={`ntaCalculation.windowGlazings[${index}]`}
        data-code={found ? undefined : elsewhere ? 'window_glazing_not_outdoor' : 'window_glazing_without_window'}>
        <legend>{found ? label(found.window) : `${elsewhere ? label(elsewhere) : String(entry.windowId)} ${note}`}</legend>
        {glazing.diffusing ? <>
          <NumberField draft={draft} onChange={change} path={[...base, 'diffusing', 'gAltitude45']} label={t('nta.form.windowGlazings.gAltitude45')} />
          <NumberField draft={draft} onChange={change} path={[...base, 'diffusing', 'gDiffuse']} label={t('nta.form.windowGlazings.gDiffuse')} />
          <TextField draft={draft} onChange={change} path={[...base, 'diffusing', 'sourceReference']} label={t('nta.form.windowGlazings.diffusingSource')} />
        </> : <>
          <label data-path={`ntaCalculation.windowGlazings[${index}].glazing.glazingType`}>{t('nta.form.windowGlazings.type')}
            <select value={String(glazing.glazingType ?? '')}
              onChange={(event) => change(base, withGlazingType(glazing, event.target.value))}>
              <option value="">{t('nta.form.windowGlazings.ownG')}</option>
              {GLAZING_TYPES.map((value) => <option key={value} value={value}>{t(`nta.form.windowGlazings.type.${value}`)}</option>)}
            </select>
          </label>
          <label data-path={`ntaCalculation.windowGlazings[${index}].glazing.fixedLouvres`}>{t('nta.form.windowGlazings.louvres')}
            <select value={String(louvres ?? '')}
              onChange={(event) => change(base, withFixedLouvres(glazing, event.target.value))}>
              <option value="">{t('nta.form.shading.none')}</option>
              {FIXED_LOUVRES.map((value) => <option key={value} value={value}>{t(`nta.form.windowGlazings.louvres.${value}`)}</option>)}
            </select>
          </label>
        </>}
        <TextField draft={draft} onChange={change} path={['windowGlazings', index, 'sourceReference']} label={t('nta.form.source')} />
        {!found && <button type="button" onClick={() => change(['windowGlazings'], entries.filter((_, other) => other !== index))}>
          {t('nta.form.windowObstructions.remove')}</button>}
      </fieldset>;
    })}
  </>;
}
