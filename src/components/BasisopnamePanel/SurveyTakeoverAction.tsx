import { useMemo, useState } from 'react';
import i18next, { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { changedPaths, useNtaDraft } from '../../context/NtaDraftProvider';
import type { OpnameAssessment } from '../../core/nta/KernelClient';
import { parseKernelPath } from '../../core/nta/pathUtil';
import {
  applyBridgeRoute, applyGroundFloors, bridgeRouteItem, canTakeOver, defaultGroundSelection, formatTakeoverNumber, groundFloorItems, groundValueLines, previewValue,
  projectGroundSurfaces,
  proposeGroundPairs, surveyBridgeRoute, surveyGroundFloors, surveyTakeover,
} from '../../core/nta/SurveyTakeover';
import { read, type Draft } from '../NtaPerformancePanel/NtaFormFields';
import { Button, Dialog } from '../ui';

/** Rows shown in the preview; the rest is counted. */
const MAX_ROWS = 60;

/**
 * "Overnemen in projectmodel": writes the NTA input derived from the survey
 * into the shared NTA draft after a preview of every changed value. Nothing
 * is applied here; the apply bar ("Toepassen") does that, and undo works.
 * The survey's floors to the ground go into the `groundFloors` entry of the
 * project floor the adviser pairs them with, value by value.
 */
export function SurveyTakeoverAction({ result }: { result: OpnameAssessment }) {
  const { t, locale } = useI18n();
  const shared = useNtaDraft();
  const { state } = useEnergy();
  const [open, setOpen] = useState(false);
  const [done, setDone] = useState(false);
  const [pairs, setPairs] = useState<Record<string, string | null>>({});
  const [accepted, setAccepted] = useState<Set<string>>(new Set());
  const [routeAccepted, setRouteAccepted] = useState(false);
  const available = shared != null && canTakeOver(result);
  const current = (shared?.draft ?? shared?.base ?? null) as Draft | null;
  const surfaces = useMemo(() => projectGroundSurfaces(state.project as never), [state.project]);
  const floors = useMemo(() => canTakeOver(result) ? surveyGroundFloors(result.derivedInput) : [], [result]);
  // The zone the chapter 11 ventilation follows on Toepassen (syncVentilation); lighting and humidifiers follow it too.
  const firstZone = state.project?.zones?.[0];
  const projectZones = state.project?.zones;
  const takeoverZone = useMemo(() => {
    if (!firstZone) return undefined;
    // The hot water follows the building's A_g when the project has more zones (review 59).
    const total = (projectZones ?? []).reduce((sum, zone) => sum + (typeof zone.floorArea === 'number' && zone.floorArea > 0 ? zone.floorArea : 0), 0);
    return { id: firstZone.id, floorArea: firstZone.floorArea ?? null, buildingFloorArea: total > 0 ? total : null };
  }, [firstZone, projectZones]);
  const projectHasPsi = useMemo(() => (state.project?.zones ?? []).some((zone) => (zone.thermalBridges ?? []).length > 0), [state.project]);
  const routeItem = useMemo(() => canTakeOver(result) ? bridgeRouteItem(surveyBridgeRoute(result), current, projectHasPsi) : null,
    [result, current, projectHasPsi]);
  const preview = useMemo(() => {
    if (!open || !canTakeOver(result)) return null;
    const takeover = surveyTakeover(result.derivedInput, current, takeoverZone);
    const changes = changedPaths(current ?? {}, takeover.next).map((path) => {
      const segments = parseKernelPath(path);
      return { path, from: read(current ?? {}, segments), to: read(takeover.next, segments) };
    });
    return { ...takeover, changes, ground: groundFloorItems(result.derivedInput, current, pairs) };
  }, [open, result, current, pairs, takeoverZone]);

  const start = () => {
    if (!canTakeOver(result)) return;
    const proposed = proposeGroundPairs(floors, surfaces);
    setPairs(proposed);
    setAccepted(defaultGroundSelection(groundFloorItems(result.derivedInput, current, proposed)));
    setRouteAccepted(routeItem?.status === 'new');
    setDone(false);
    setOpen(true);
  };
  const pair = (surveyId: string, surfaceId: string | null) => {
    if (!canTakeOver(result)) return;
    const next = { ...pairs, [surveyId]: surfaceId };
    setPairs(next);
    setAccepted(defaultGroundSelection(groundFloorItems(result.derivedInput, current, next)));
  };
  const toggle = (key: string) => setAccepted((selection) => {
    const next = new Set(selection);
    if (next.has(key)) next.delete(key); else next.add(key);
    return next;
  });
  const routeLabel = (value: unknown) => value === 'forfait' ? t('nta.form.thermalBridgeRoute.forfait')
    : value === 'detailed' ? t('nta.form.thermalBridgeRoute.detailed') : t('nta.form.thermalBridgeRoute.fromFloors');
  const surfaceName = (id: string) => surfaces.find((surface) => surface.id === id)?.name ?? id;
  const groundLabel = (key: string) => i18next.exists(`opname.takeover.ground.${key}`) ? t(`opname.takeover.ground.${key}`) : undefined;
  const groundValue = (value: unknown, field: string) => {
    const lines = groundValueLines(value, locale, groundLabel, field);
    if (lines.length === 1 && lines[0].label == null) return lines[0].text;
    return <ul className="opname-takeover__value">
      {lines.map((line, index) => <li key={index}>{line.label != null && <span>{line.label}: </span>}{line.text}</li>)}
    </ul>;
  };

  if (!shared) return null;
  return (
    <div className="opname-takeover">
      <Button variant="secondary" size="sm" disabled={!available} onClick={start}>
        {t('opname.takeover.button')}
      </Button>
      {!available && <small className="nta-form-note"> {t('opname.takeover.unavailable')}</small>}
      {done && <p className="nta-form-note" role="status">{t('opname.takeover.done')}</p>}
      {open && preview && <Dialog title={t('opname.takeover.title')} onClose={() => setOpen(false)} width={720}
        footer={<>
          <Button variant="ghost" onClick={() => setOpen(false)}>{t('opname.takeover.cancel')}</Button>
          <Button variant="primary" disabled={preview.changes.length === 0 && accepted.size === 0 && !routeAccepted} onClick={() => {
            if (!canTakeOver(result)) return;
            const derived = result.derivedInput;
            const route = routeAccepted ? routeItem : null;
            shared.update((draft) => applyBridgeRoute(applyGroundFloors(surveyTakeover(derived, draft, takeoverZone).next, derived, pairs, accepted),
              route, new Set(route ? [route.key] : [])) as Draft);
            setOpen(false);
            setDone(true);
          }}>{t('opname.takeover.confirm')}</Button>
        </>}>
        <p>{t('opname.takeover.intro')}</p>
        <p><strong>{t('opname.takeover.count', { count: preview.changes.length })}</strong></p>
        {preview.changes.length === 0
          ? <p className="nta-form-note">{t('opname.takeover.noChanges')}</p>
          : <table className="opname-takeover__diff" aria-label={t('opname.takeover.title')}>
            <thead><tr>
              <th scope="col">{t('opname.takeover.path')}</th>
              <th scope="col">{t('opname.takeover.from')}</th>
              <th scope="col">{t('opname.takeover.to')}</th>
            </tr></thead>
            <tbody>
              {preview.changes.slice(0, MAX_ROWS).map((row) => <tr key={row.path}>
                <td><code>{row.path}</code></td>
                <td>{previewValue(row.from, locale)}</td>
                <td>{previewValue(row.to, locale)}</td>
              </tr>)}
            </tbody>
          </table>}
        {preview.changes.length > MAX_ROWS &&
          <p className="nta-form-note">{t('opname.takeover.more', { count: preview.changes.length - MAX_ROWS })}</p>}
        {floors.length > 0 && <section className="opname-takeover__ground" aria-label={t('opname.takeover.ground.title')}>
          <h4>{t('opname.takeover.ground.title')}</h4>
          <p className="nta-form-note">{t('opname.takeover.ground.intro')}</p>
          {floors.map((floor) => <div key={floor.id} className="nta-form-row">
            <label>{t('opname.takeover.ground.pair', { floor: floor.id, area: floor.areaM2 == null ? '—' : formatTakeoverNumber(floor.areaM2, locale) })}
              <select value={pairs[floor.id] ?? ''} onChange={(event) => pair(floor.id, event.target.value || null)}>
                <option value="">{t('opname.takeover.ground.none')}</option>
                {surfaces.map((surface) => {
                  // One project floor takes one survey floor.
                  const other = floors.find((item) => item.id !== floor.id && pairs[item.id] === surface.id);
                  return <option key={surface.id} value={surface.id} disabled={other != null}>
                    {`${surface.name} (${formatTakeoverNumber(surface.areaM2, locale)} m²)`}
                    {other ? ` ${t('opname.takeover.ground.pairedElsewhere', { floor: other.id })}` : ''}
                  </option>;
                })}
              </select>
            </label>
          </div>)}
          {surfaces.length === 0 && <p className="nta-form-note">{t('opname.takeover.ground.noSurfaces')}</p>}
          {preview.ground.length > 0 &&
            <table className="opname-takeover__diff opname-takeover__ground-table" aria-label={t('opname.takeover.ground.title')}>
              <thead><tr>
                <th scope="col">{t('opname.takeover.ground.accept')}</th>
                <th scope="col">{t('opname.takeover.path')}</th>
                <th scope="col">{t('opname.takeover.from')}</th>
                <th scope="col">{t('opname.takeover.to')}</th>
              </tr></thead>
              <tbody>
                {preview.ground.map((item) => {
                  const label = `${surfaceName(item.surfaceId)} — ${t(`opname.takeover.ground.field.${item.field}`)}`;
                  return <tr key={item.key} className={item.status === 'conflict' ? 'opname-takeover__conflict' : undefined}>
                    <td><input type="checkbox" checked={accepted.has(item.key)} onChange={() => toggle(item.key)}
                      aria-label={t('opname.takeover.ground.acceptItem', { item: label })} /></td>
                    <th scope="row">{label}{item.status === 'conflict' && <> <small>{t('opname.takeover.ground.conflict')}</small></>}</th>
                    <td>{groundValue(item.from, item.field)}</td>
                    <td>{item.to === undefined ? t('opname.takeover.ground.remove') : groundValue(item.to, item.field)}</td>
                  </tr>;
                })}
              </tbody>
            </table>}
        </section>}
        {routeItem && <section className="opname-takeover__route" aria-label={t('opname.takeover.route.title')}>
          <h4>{t('opname.takeover.route.title')}</h4>
          <p className="nta-form-note">{t('opname.takeover.route.intro')}</p>
          <label className={routeItem.status === 'conflict' ? 'opname-takeover__conflict' : undefined}>
            <input type="checkbox" checked={routeAccepted} onChange={() => setRouteAccepted((value) => !value)} />
            {' '}{t('opname.takeover.route.item', { from: routeLabel(routeItem.from), to: routeLabel(routeItem.to) })}
            {routeItem.status === 'conflict' && <> <small>{t('opname.takeover.route.conflict')}</small></>}
          </label>
        </section>}
        <h4>{t('opname.takeover.skipped')}</h4>
        <ul className="opname-takeover__skipped">
          {[...new Set(preview.skipped.map((part) => part.reason))].map((reason) =>
            <li key={reason}>{t(`opname.takeover.skipped.${reason}`)}</li>)}
        </ul>
      </Dialog>}
    </div>
  );
}
