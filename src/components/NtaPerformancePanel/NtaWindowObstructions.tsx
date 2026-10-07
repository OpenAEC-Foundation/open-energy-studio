import type { IProject } from '../../core/energy/types';
import { windowObstructionTemplate } from '../../core/nta/NtaSystemTemplates';
import { useI18n } from '../../i18n/i18n';
import { read, TextField, type Draft, type Path } from './NtaFormFields';
import { WindowObstructionFields } from './NtaSystemSections';

// External obstruction per project window (`windowObstructions`): 7.13 takes
// F_sh;obst per window and 17.3.2 one situation per window. A window that is
// not listed keeps the project-wide `windowSolar.obstruction`.

interface SectionProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
}

const SAME = '';

/** Outdoor project windows, the only ones the project route calculates. */
export function outdoorWindows(project: IProject) {
  return project.zones.flatMap((zone) => zone.surfaces
    .filter((surface) => surface.thermalBoundary === 'outdoor')
    .flatMap((surface) => surface.windows.map((window) => ({ window, surface }))));
}

/** The `windowObstructions` list after choosing `method` for one window; '' is "as the project". */
export function withWindowObstruction(entries: Draft[], windowId: string, method: string): Draft[] {
  const index = entries.findIndex((entry) => entry.windowId === windowId);
  if (method === SAME) return entries.filter((_, other) => other !== index);
  const obstruction = windowObstructionTemplate(method);
  if (index < 0) return [...entries, { windowId, obstruction, sourceReference: '' }];
  return entries.map((entry, other) => other === index ? { ...entry, obstruction } : entry);
}

export function WindowObstructionsFields({ draft, change, project }: SectionProps & { project: IProject }) {
  const { t } = useI18n();
  const entries = (read(draft, ['windowObstructions']) as Draft[] | undefined) ?? [];
  const windows = outdoorWindows(project);
  const label = (window: { id: string; name?: string }) => window.name || window.id;
  return <>
    <p className="nta-form-note">{t('nta.form.windowObstructions.help')}</p>
    <div className="nta-performance-table"><table>
      <thead><tr>
        <th>{t('nta.form.windowObstructions.window')}</th>
        <th>{t('nta.form.windowObstructions.surface')}</th>
        <th>{t('nta.form.windowObstructions.situation')}</th>
      </tr></thead>
      <tbody>
        {windows.map(({ window, surface }) => {
          const index = entries.findIndex((entry) => entry.windowId === window.id);
          const method = index < 0 ? SAME : String(read(draft, ['windowObstructions', index, 'obstruction', 'method']) ?? SAME);
          return <tr key={window.id}>
            <td>{label(window)}</td>
            <td>{surface.name || surface.id}</td>
            <td>
              <select aria-label={`${t('nta.form.windowObstructions.situation')} ${label(window)}`} value={method}
                onChange={(event) => change(['windowObstructions'], withWindowObstruction(entries, window.id, event.target.value))}>
                <option value={SAME}>{t('nta.form.windowObstructions.same')}</option>
                <option value="minimal">{t('nta.form.obstruction.minimal')}</option>
                <option value="parallel_obstruction">{t('nta.form.obstruction.parallel')}</option>
                <option value="overhang">{t('nta.form.obstruction.overhang')}</option>
                <option value="side_obstruction">{t('nta.form.obstruction.side')}</option>
                <option value="full">{t('nta.form.obstruction.full')}</option>
                <option value="other">{t('nta.form.obstruction.other')}</option>
                <option value="declared">{t('nta.form.obstruction.declared')}</option>
              </select>
            </td>
          </tr>;
        })}
      </tbody>
    </table></div>
    {entries.map((entry, index) => {
      const found = windows.find(({ window }) => window.id === entry.windowId);
      return <fieldset key={`${String(entry.windowId)}-${index}`} className="nta-form-group"
        data-path={`ntaCalculation.windowObstructions[${index}]`}>
        <legend>{found ? label(found.window) : `${String(entry.windowId)} ${t('nta.form.windowObstructions.missing')}`}</legend>
        <WindowObstructionFields draft={draft} change={change} base={['windowObstructions', index, 'obstruction']} />
        <TextField draft={draft} onChange={change} path={['windowObstructions', index, 'sourceReference']} label={t('nta.form.source')} />
        {!found && <button type="button" onClick={() => change(['windowObstructions'], entries.filter((_, other) => other !== index))}>
          {t('nta.form.windowObstructions.remove')}</button>}
      </fieldset>;
    })}
  </>;
}
