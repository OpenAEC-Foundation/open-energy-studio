import type { IProject } from '../../core/energy/types';
import { useI18n } from '../../i18n/i18n';
import { NumberField, read, TextField, type Draft, type Path } from './NtaFormFields';
import { outdoorWindows } from './NtaWindowObstructions';

// Movable sun shading per project window (`windowShadings`): 7.42/7.43 give
// g_gl and F_c per window. A listed window takes its own shading (or none);
// a window that is not listed keeps `windowSolar.movableShading`.

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

const SAME = '';
const NONE = 'none';

/** Table 7.5/7.6 devices as `kind` or `kind:colour`; '' is a declared F_c. */
export const SHADING_DEVICES = [
  'external_screen:dark', 'external_screen:other', 'external_screen:white', 'external_screen:unknown',
  'external_venetian_blind:dark', 'external_venetian_blind:other', 'external_venetian_blind:white', 'external_venetian_blind:unknown',
  'external_roller_shutter:other', 'external_roller_shutter:white', 'external_roller_shutter:unknown',
  'internal_metallised_fabric', 'drop_arm_awning', 'folding_arm_awning',
] as const;

export function deviceValue(device: unknown): string {
  if (!device || typeof device !== 'object') return '';
  const { kind, colour } = device as { kind?: unknown; colour?: unknown };
  return colour ? `${String(kind)}:${String(colour)}` : String(kind ?? '');
}

export function deviceOf(value: string): Draft | null {
  if (!value) return null;
  const [kind, colour] = value.split(':');
  return colour ? { kind, colour } : { kind };
}

/**
 * The `windowShadings` list after choosing for one window: '' is "as the
 * project", 'none' is no shading on this window, otherwise a control of
 * tables 7.7–7.9.
 */
export function withWindowShading(entries: Draft[], windowId: string, choice: string): Draft[] {
  const index = entries.findIndex((entry) => entry.windowId === windowId);
  if (choice === SAME) return entries.filter((_, other) => other !== index);
  const current = index < 0 ? undefined : entries[index];
  const sourceReference = current?.sourceReference ?? '';
  const entry: Draft = choice === NONE
    ? { windowId, sourceReference }
    : {
      windowId,
      movableShading: {
        ...((current?.movableShading as Draft | undefined) ?? { sourceReference: '' }),
        control: choice,
      },
      sourceReference,
    };
  if (index < 0) return [...entries, entry];
  return entries.map((item, other) => other === index ? entry : item);
}

/** The shading of one entry with a table 7.5/7.6 device or a declared F_c. */
export function withShadingDevice(shading: Draft, value: string): Draft {
  const { device: _device, reductionFactor: _factor, ...rest } = shading;
  const device = deviceOf(value);
  return device ? { ...rest, device } : { ...rest, reductionFactor: null };
}

export function WindowShadingsFields({ draft, change, project }: SectionProps & { project: IProject }) {
  const { t } = useI18n();
  const entries = (read(draft, ['windowShadings']) as Draft[] | undefined) ?? [];
  const windows = outdoorWindows(project);
  const label = (window: { id: string; name?: string }) => window.name || window.id;
  const choiceOf = (entry: Draft | undefined) => {
    if (!entry) return SAME;
    const shading = entry.movableShading as Draft | undefined;
    return shading ? String(shading.control ?? '') : NONE;
  };
  return <>
    <p className="nta-form-note">{t('nta.form.windowShadings.help')}</p>
    <div className="nta-performance-table"><table>
      <thead><tr>
        <th>{t('nta.form.windowObstructions.window')}</th>
        <th>{t('nta.form.windowObstructions.surface')}</th>
        <th>{t('nta.form.windowShadings.shading')}</th>
      </tr></thead>
      <tbody>
        {windows.map(({ window, surface }) => {
          const entry = entries.find((item) => item.windowId === window.id);
          return <tr key={window.id}>
            <td>{label(window)}</td>
            <td>{surface.name || surface.id}</td>
            <td>
              <select aria-label={`${t('nta.form.windowShadings.shading')} ${label(window)}`} value={choiceOf(entry)}
                onChange={(event) => change(['windowShadings'], withWindowShading(entries, window.id, event.target.value))}>
                <option value={SAME}>{t('nta.form.windowObstructions.same')}</option>
                <option value={NONE}>{t('nta.form.shading.none')}</option>
                <option value="manual_residential">{t('nta.form.shading.manual')}</option>
                <option value="automatic_residential_iso52016">{t('nta.form.shading.automaticResidential')}</option>
                <option value="automatic">{t('nta.form.shading.automatic')}</option>
                <option value="manual_utility_with_glare_protection">{t('nta.form.shading.manualUtilityGlare')}</option>
                <option value="manual_utility_without_glare_protection">{t('nta.form.shading.manualUtility')}</option>
              </select>
            </td>
          </tr>;
        })}
      </tbody>
    </table></div>
    {entries.map((entry, index) => {
      const found = windows.find(({ window }) => window.id === entry.windowId);
      const shading = entry.movableShading as Draft | undefined;
      const base: Path = ['windowShadings', index, 'movableShading'];
      return <fieldset key={`${String(entry.windowId)}-${index}`} className="nta-form-group"
        data-path={`ntaCalculation.windowShadings[${index}]`}
        data-code={found ? undefined : 'window_shading_without_window'}>
        <legend>{found ? label(found.window) : `${String(entry.windowId)} ${t('nta.form.windowObstructions.missing')}`}</legend>
        {shading && <>
          <label data-path={`ntaCalculation.windowShadings[${index}].movableShading.device`}>{t('nta.form.windowShadings.device')}
            <select value={deviceValue(shading.device)}
              onChange={(event) => change(base, withShadingDevice(shading, event.target.value))}>
              <option value="">{t('nta.form.windowShadings.declared')}</option>
              {SHADING_DEVICES.map((value) => <option key={value} value={value}>{t(`nta.form.windowShadings.device.${value}`)}</option>)}
            </select>
          </label>
          {!shading.device && <NumberField draft={draft} onChange={change} path={[...base, 'reductionFactor']} label={t('nta.form.shadingFc')} />}
          <TextField draft={draft} onChange={change} path={[...base, 'sourceReference']} label={t('nta.form.windowShadings.deviceSource')} />
        </>}
        <TextField draft={draft} onChange={change} path={['windowShadings', index, 'sourceReference']} label={t('nta.form.source')} />
        {!found && <button type="button" onClick={() => change(['windowShadings'], entries.filter((_, other) => other !== index))}>
          {t('nta.form.windowObstructions.remove')}</button>}
      </fieldset>;
    })}
  </>;
}
