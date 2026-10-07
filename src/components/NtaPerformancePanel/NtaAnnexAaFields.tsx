import { useState } from 'react';
import type { IProject } from '../../core/energy/types';
import {
  ANNEX_AA_PATH, annexAaTotals, checkAnnexAa, duplicateAnnexAaRoom, newAnnexAaCalculation, newAnnexAaRoom,
  windowsAssignedElsewhere, type AnnexAaFormIssue,
} from '../../core/nta/annexAaForm';
import { formatKernelPath } from '../../core/nta/pathUtil';
import { formatNumber, kernelCodeLabel } from '../../i18n/format';
import { useI18n } from '../../i18n/i18n';
import { CheckField, NumberField, read, TextField, useFieldPath, type Draft, type Path } from './NtaFormFields';

// Annex AA (§5.7.1, cooling capacity evidence) as a form: the zone data, a
// table of habitable rooms (add, duplicate, remove) and per room its project
// windows. The 2024-only SWM and roof areas stay in `AnnexAa2024Fields`.
// "JSON bekijken" shows the same block as editable JSON for advanced use.

interface AnnexAaProps {
  draft: Draft;
  change: (path: Path, value: unknown) => void;
  project: IProject;
}

const BASE: Path = [...ANNEX_AA_PATH];
/** Editions whose 5.7.1 has annex AA as capacity evidence (`annex_aa_route`). */
const AA_EDITIONS = new Set(['2025+C1', '2024']);

function projectWindows(project: IProject) {
  return project.zones.flatMap((zone) => zone.surfaces)
    .filter((surface) => surface.thermalBoundary === 'outdoor')
    .flatMap((surface) => surface.windows);
}

/** The first issue at `path` (relative to `ntaCalculation`) as an inline message. */
function IssueAt({ issues, path }: { issues: AnnexAaFormIssue[]; path: Path }) {
  const { t } = useI18n();
  const key = formatKernelPath(path);
  const found = issues.find((issue) => formatKernelPath(issue.path) === key);
  if (!found) return null;
  return <small className="nta-form-error nta-aa-issue" role="alert" data-code={found.code}>
    {kernelCodeLabel(t, found.code).text}
  </small>;
}

export function AnnexAaCalculationFields({ draft, change, project }: AnnexAaProps) {
  const { t, locale } = useI18n();
  const [showJson, setShowJson] = useState(false);
  const field = { draft, onChange: change };
  const calculation = read(draft, BASE);
  const edition = String(read(draft, ['normVersion']) ?? '2025+C1');
  const kernelPath = useFieldPath(BASE);
  const prefix = useFieldPath([]);
  const windows = projectWindows(project);

  if (calculation == null || typeof calculation !== 'object') {
    return <div className="nta-form-group nta-aa" data-testid="nta-annex-aa" data-path={kernelPath ?? undefined}>
      <p className="nta-form-note">{t('ntaStep.annexAa.intro')}</p>
      {!AA_EDITIONS.has(edition) && <p className="nta-form-note nta-form-error" role="note">{t('ntaStep.annexAa.notInEdition')}</p>}
      <button type="button" onClick={() => change(BASE, newAnnexAaCalculation(read(draft, ['constructionYear'])))}>
        {t('ntaStep.annexAa.start')}
      </button>
    </div>;
  }

  const rooms = (read(draft, [...BASE, 'rooms']) as Draft[] | undefined) ?? [];
  const issues = checkAnnexAa(calculation, { edition2024: edition === '2024', windowIds: windows.map((window) => window.id) });
  const totals = annexAaTotals(calculation);
  const windowName = (id: unknown) => {
    const bare = String(id ?? '').replace(/^window:/, '');
    const found = windows.find((window) => window.id === bare);
    return found ? `${found.name || found.id} · ${formatNumber(found.area, locale, 2)} m² · ${found.orientation}` : null;
  };
  const setRooms = (next: Draft[]) => change([...BASE, 'rooms'], next);

  return <fieldset className="nta-form-group nta-aa" data-testid="nta-annex-aa" data-path={kernelPath ?? undefined}>
    <legend>{t('ntaStep.annexAa.title')}</legend>
    <p className="nta-form-note">{t('ntaStep.annexAa.intro')}</p>
    {!AA_EDITIONS.has(edition) && <p className="nta-form-note nta-form-error" role="note">{t('ntaStep.annexAa.notInEdition')}</p>}

    <div className="nta-aa-field">
      <NumberField {...field} path={[...BASE, 'constructionYear']} label={t('ntaStep.annexAa.constructionYear')} step="1" />
      <IssueAt issues={issues} path={[...BASE, 'constructionYear']} />
    </div>
    <CheckField {...field} path={[...BASE, 'postInsulated']} label={t('ntaStep.annexAa.postInsulated')} />
    <div className="nta-aa-field">
      <NumberField {...field} path={[...BASE, 'generatorCapacityKw']} label={t('ntaStep.annexAa.generatorCapacity')} unit="kW" optional />
      <small>{t('ntaStep.annexAa.generatorNote')}</small>
      <IssueAt issues={issues} path={[...BASE, 'generatorCapacityKw']} />
    </div>

    <div className="nta-aa-summary">
      <table>
        <caption>{t('ntaStep.annexAa.rooms')}</caption>
        <thead><tr>
          <th scope="col">{t('ntaStep.annexAa.summary.room')}</th>
          <th scope="col">{t('ntaStep.annexAa.summary.area')}</th>
          <th scope="col">{t('ntaStep.annexAa.summary.capacity')}</th>
          <th scope="col">{t('ntaStep.annexAa.summary.windows')}</th>
          <th scope="col">{t('ntaStep.annexAa.summary.living')}</th>
        </tr></thead>
        <tbody>
          {rooms.map((room, index) => <tr key={index}>
            <th scope="row"><a href={`#nta-aa-room-${index}`}>{String(room?.id || index + 1)}</a></th>
            <td>{typeof room?.areaM2 === 'number' ? formatNumber(room.areaM2, locale, 1) : '—'}</td>
            <td>{typeof room?.installedCapacityKw === 'number' ? formatNumber(room.installedCapacityKw, locale, 2) : '—'}</td>
            <td>{Array.isArray(room?.windows) ? room.windows.length : 0}</td>
            <td>{room?.living === true ? '✓' : ''}</td>
          </tr>)}
        </tbody>
      </table>
      <p className="nta-aa-totals">{t('ntaStep.annexAa.totals', {
        rooms: totals.rooms, area: formatNumber(totals.areaM2, locale, 1), capacity: formatNumber(totals.capacityKw, locale, 2),
      })}</p>
    </div>
    <IssueAt issues={issues} path={[...BASE, 'rooms']} />

    {rooms.map((room, index) => {
      const at = (...path: Path): Path => [...BASE, 'rooms', index, ...path];
      const roomName = String(room?.id || index + 1);
      const roomWindows = Array.isArray(room?.windows) ? room.windows as Draft[] : [];
      const elsewhere = windowsAssignedElsewhere(rooms, index);
      const here = new Set(roomWindows.map((window) => String(window?.windowId ?? '')));
      const free = windows.filter((window) => !elsewhere.has(window.id) && !here.has(window.id));
      return <fieldset key={index} id={`nta-aa-room-${index}`} className="nta-form-group nta-aa-room"
        aria-label={t('ntaStep.annexAa.room', { room: roomName })}>
        <legend>{t('ntaStep.annexAa.room', { room: roomName })}</legend>
        <div className="nta-aa-field">
          <TextField {...field} path={at('id')} label={t('ntaStep.annexAa.roomId')} />
          <IssueAt issues={issues} path={at('id')} />
        </div>
        <div className="nta-aa-field">
          <NumberField {...field} path={at('areaM2')} label={t('ntaStep.annexAa.area')} unit="m²" />
          <IssueAt issues={issues} path={at('areaM2')} />
        </div>
        <div className="nta-aa-field">
          <NumberField {...field} path={at('opaqueInnerAreaM2')} label={t('ntaStep.annexAa.opaqueArea')} unit="m²" />
          <IssueAt issues={issues} path={at('opaqueInnerAreaM2')} />
        </div>
        <div className="nta-aa-field">
          <NumberField {...field} path={at('installedCapacityKw')} label={t('ntaStep.annexAa.installedCapacity')} unit="kW" />
          <IssueAt issues={issues} path={at('installedCapacityKw')} />
        </div>
        <CheckField {...field} path={at('living')} label={t('ntaStep.annexAa.living')} />

        <div className="nta-aa-windows" role="group" aria-label={t('ntaStep.annexAa.windows', { room: roomName })}>
          <strong>{t('ntaStep.annexAa.windows', { room: roomName })}</strong>
          {roomWindows.length === 0 && <small>{t('ntaStep.annexAa.noWindows')}</small>}
          {roomWindows.map((window, w) => {
            const id = String(window?.windowId ?? '');
            const options = [...(id !== '' ? [id] : []), ...free.map((item) => item.id)];
            return <div key={w} className="nta-aa-window">
              <label data-path={prefix == null ? undefined : formatKernelPath([prefix, ...at('windows', w, 'windowId')])}>{t('ntaStep.annexAa.window')}
                <select value={id} onChange={(event) => change(at('windows', w, 'windowId'), event.target.value)}>
                  {id === '' && <option value="">—</option>}
                  {options.map((option) => <option key={option} value={option}>
                    {windowName(option) ?? t('ntaStep.annexAa.windowUnknown', { id: option })}
                  </option>)}
                </select>
              </label>
              <NumberField {...field} path={at('windows', w, 'uWithShutterWPerM2k')} label={t('ntaStep.annexAa.uShutter')}
                unit="W/(m²·K)" optional />
              <button type="button" className="nta-form-remove"
                onClick={() => change(at('windows'), roomWindows.filter((_, other) => other !== w))}>{t('nta.form.remove')}</button>
              <IssueAt issues={issues} path={at('windows', w, 'windowId')} />
              <IssueAt issues={issues} path={at('windows', w, 'uWithShutterWPerM2k')} />
            </div>;
          })}
          {free.length > 0
            ? <button type="button" onClick={() => change(at('windows'), [...roomWindows, { windowId: free[0].id }])}>
              {t('ntaStep.annexAa.addWindow')}</button>
            : windows.length > 0 && <small>{t('ntaStep.annexAa.noFreeWindows')}</small>}
        </div>

        <div className="nta-aa-actions">
          <button type="button" onClick={() => setRooms(duplicateAnnexAaRoom(rooms, index))}>{t('ntaStep.annexAa.duplicateRoom')}</button>
          <button type="button" className="nta-form-remove" onClick={() => setRooms(rooms.filter((_, other) => other !== index))}>
            {t('ntaStep.annexAa.removeRoom')}</button>
        </div>
      </fieldset>;
    })}

    <div className="nta-aa-actions">
      <button type="button" onClick={() => setRooms([...rooms, newAnnexAaRoom(rooms) as unknown as Draft])}>{t('ntaStep.annexAa.addRoom')}</button>
      <button type="button" onClick={() => setShowJson((shown) => !shown)} aria-expanded={showJson}>
        {showJson ? t('ntaStep.annexAa.hideJson') : t('ntaStep.annexAa.showJson')}</button>
      <button type="button" className="nta-form-remove" onClick={() => change(BASE, null)}>{t('ntaStep.annexAa.removeCalculation')}</button>
    </div>
    {showJson && <AnnexAaJson value={calculation} onApply={(next) => change(BASE, next)} />}
  </fieldset>;
}

/** The calculation as editable JSON; applied only when it parses to an object. */
function AnnexAaJson({ value, onApply }: { value: unknown; onApply: (next: unknown) => void }) {
  const { t } = useI18n();
  const [text, setText] = useState(() => JSON.stringify(value, null, 2));
  const [invalid, setInvalid] = useState(false);
  return <div className="nta-aa-json">
    <label>{t('ntaStep.annexAa.jsonLabel')}
      <textarea rows={12} spellCheck={false} value={text} onChange={(event) => { setText(event.target.value); setInvalid(false); }} />
    </label>
    {invalid && <small className="nta-form-error" role="alert">{t('ntaStep.annexAa.jsonInvalid')}</small>}
    <button type="button" onClick={() => {
      try {
        const parsed: unknown = JSON.parse(text);
        if (parsed == null || typeof parsed !== 'object' || Array.isArray(parsed)) throw new Error('not an object');
        onApply(parsed);
      } catch {
        setInvalid(true);
      }
    }}>{t('ntaStep.annexAa.jsonApply')}</button>
  </div>;
}
