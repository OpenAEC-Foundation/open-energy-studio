/**
 * Step 2 Gebouw (UI redesign F5, mockup 02): one page per part of the
 * building — schil & ramen, rekenzones, constructies, koudebruggen,
 * luchtdichtheid. Lists are `DataTable`s; a click selects the element and the
 * inspector edits it, a double click or Enter opens the full editor as a side
 * sheet. Edits update the item in place, so ids stay stable (relabel,
 * maatwerkadvies templates).
 */
import { useMemo, useState, type ReactNode } from 'react';
import { Plus, Search } from 'lucide-react';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { useKernel } from '../../../context/KernelProvider';
import { editAction } from '../../../core/energy/projectItems';
import { formatKernelPath } from '../../../core/nta/pathUtil';
import { routeForPath } from '../../../core/nta/gapRoutes';
import { kernelIssues } from '../../../core/nta/stepStatus';
import { formatNumber } from '../../../i18n/format';
import type { DialogType, IConstruction, IProject, ISurface, IWindow, IWindowType, IZone, Orientation, SurfaceType } from '../../../core/energy/types';
import { Banner, Button, Card, DataTable, IssueList, NumberInput, Pill, Segmented, type Column } from '../../ui';
import { ItemActions } from '../../ItemActions/ItemActions';
import { ELEMENT_KINDS, elementRcActions, sharedConstruction, sharedWindowU, surfacesOf } from '../../../core/energy/elementRc';
import { WINDOW_KINDS, newWindowTypeId, windowTypeUsage } from '../../../core/energy/windowTypes';
import { useShellActions } from '../ShellActions';
import { routeLabel } from '../PageHeader';

const ORIENTATION_ORDER: Orientation[] = ['S', 'SE', 'SW', 'E', 'W', 'NE', 'NW', 'N', 'horizontal'];
const TYPE_ORDER: SurfaceType[] = ['wall', 'roof', 'floor', 'internal'];

/** Compass bearing of an orientation, for the small compass glyph. */
const BEARING: Record<Orientation, number | null> = {
  N: 0, NE: 45, E: 90, SE: 135, S: 180, SW: 225, W: 270, NW: 315, horizontal: null,
};

export function Compass({ orientation }: { orientation: Orientation }) {
  const bearing = BEARING[orientation];
  if (bearing == null) return <span className="compass compass--flat" aria-hidden="true" />;
  return (
    <svg className="compass" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">
      <circle cx="8" cy="8" r="6.5" fill="none" stroke="currentColor" strokeOpacity=".45" />
      <path d="M8 2.5 L9.6 8 L8 7.2 L6.4 8 Z" fill="currentColor" transform={`rotate(${bearing} 8 8)`} />
    </svg>
  );
}

/** Tilt in degrees from the NTA input, else the usual value of the surface type (wall 90°, floor 0°). */
export function surfaceTilt(project: IProject, surface: ISurface): number | null {
  const declared = project.ntaCalculation?.surfaceTilts?.find((entry) => entry.surfaceId === surface.id);
  if (declared) return declared.tiltDeg;
  if (surface.type === 'wall') return 90;
  if (surface.type === 'floor') return 0;
  return null;
}

/** The add buttons of a building page; the accessible name and title are the former ribbon labels. */
function AddBar({ entries }: { entries: Array<{ dialog: DialogType; labelKey: string; primary?: boolean }> }) {
  const { t } = useI18n();
  const actions = useShellActions();
  const { dispatch } = useEnergy();
  const open = (dialog: DialogType) => {
    if (actions) actions.openDialog(dialog);
    else dispatch({ type: 'OPEN_DIALOG', payload: { type: dialog } });
  };
  return (
    <div className="add-bar" role="group" aria-label={t('building.add')}>
      {entries.map((entry) => (
        <Button key={entry.dialog} size="sm" variant={entry.primary ? 'primary' : undefined} icon={<Plus aria-hidden="true" />}
          title={t(entry.labelKey)} onClick={() => open(entry.dialog)}>{t(entry.labelKey)}</Button>
      ))}
    </div>
  );
}

function useSelection() {
  const { state, dispatch } = useEnergy();
  return {
    selectedId: state.selectedItemId,
    select: (id: string, itemType: string) => dispatch({ type: 'SELECT_ITEM', payload: { id, itemType } }),
    edit: (itemType: string, id: string) => { const action = editAction(itemType, id); if (action) dispatch(action); },
  };
}

// ── Schil & ramen ────────────────────────────────────────────────────

type KindFilter = 'all' | 'wall' | 'roof' | 'floor';
type GroupBy = 'orientation' | 'type' | 'construction';

interface EnvelopeRow {
  key: string;
  kind: 'surface' | 'window';
  zone: IZone;
  surface: ISurface;
  window?: IWindow;
  path: string;
  group: string;
}

/** Net opaque area: gross minus the windows in the surface. */
export function netArea(surface: ISurface): number {
  return Math.max(0, surface.area - surface.windows.reduce((sum, win) => sum + win.area, 0));
}

export function EnvelopePage() {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const { project } = state;
  const { selectedId, select, edit } = useSelection();
  const [kind, setKind] = useState<KindFilter>('all');
  const [groupBy, setGroupBy] = useState<GroupBy>('orientation');
  const [query, setQuery] = useState('');
  const constructions = new Map(project.constructions.map((construction) => [construction.id, construction]));
  const n = (value: number | null | undefined, digits = 2) => formatNumber(value, locale, digits);

  const allSurfaces = project.zones.flatMap((zone, zi) => zone.surfaces.map((surface, si) => ({ zone, zi, surface, si })));
  const counts = {
    all: allSurfaces.length,
    wall: allSurfaces.filter(({ surface }) => surface.type === 'wall').length,
    roof: allSurfaces.filter(({ surface }) => surface.type === 'roof').length,
    floor: allSurfaces.filter(({ surface }) => surface.type === 'floor').length,
  };
  const windowCount = allSurfaces.reduce((sum, { surface }) => sum + surface.windows.length, 0);

  const groupOf = (surface: ISurface): { id: string; rank: number; label: string } => {
    if (groupBy === 'type') return { id: surface.type, rank: TYPE_ORDER.indexOf(surface.type), label: t(`surfaceType.${surface.type}`) };
    if (groupBy === 'construction') {
      const name = constructions.get(surface.constructionId)?.name ?? t('envelope.noConstruction');
      return { id: surface.constructionId || '-', rank: 0, label: name };
    }
    return { id: surface.orientation, rank: ORIENTATION_ORDER.indexOf(surface.orientation), label: t(`orientation.${surface.orientation}`) };
  };

  const rows = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const visible = allSurfaces.filter(({ surface }) => (kind === 'all' || surface.type === kind) && (
      !needle
      || surface.name.toLowerCase().includes(needle)
      || (constructions.get(surface.constructionId)?.name ?? '').toLowerCase().includes(needle)
      || surface.windows.some((win) => win.name.toLowerCase().includes(needle))
    ));
    const decorated = visible.map((entry) => ({ ...entry, group: groupOf(entry.surface) }));
    decorated.sort((a, b) => a.group.rank - b.group.rank || a.group.label.localeCompare(b.group.label, locale) || 0);
    const summary = new Map<string, { count: number; area: number }>();
    for (const { group, surface } of decorated) {
      const item = summary.get(group.id) ?? { count: 0, area: 0 };
      summary.set(group.id, { count: item.count + 1, area: item.area + surface.area });
    }
    const out: EnvelopeRow[] = [];
    for (const { zone, zi, surface, si, group } of decorated) {
      const total = summary.get(group.id)!;
      const label = `${group.label} · ${t(total.count === 1 ? 'envelope.surfaceCount.one' : 'envelope.surfaceCount.other').replace('{count}', String(total.count))} · ${n(total.area, 1)} m²`;
      out.push({ key: surface.id, kind: 'surface', zone, surface, path: formatKernelPath(['zones', zi, 'surfaces', si]), group: label });
      surface.windows.forEach((win, wi) => out.push({
        key: win.id, kind: 'window', zone, surface, window: win, path: formatKernelPath(['zones', zi, 'surfaces', si, 'windows', wi]), group: label,
      }));
    }
    return out;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [project, kind, groupBy, query, locale, t]);

  const boundary = (surface: ISurface) => surface.thermalBoundary ? t(`kernel.boundary.${surface.thermalBoundary}`) : t('kernel.boundary.unknown');
  const columns: Array<Column<EnvelopeRow>> = [
    {
      key: 'name', header: t('properties.name'), render: (row) => row.kind === 'window'
        ? <span className="envelope-child"><span className="envelope-child__mark" aria-hidden="true" />{row.window!.name}{(row.window!.count ?? 1) > 1 ? ` ×${row.window!.count}` : ''}</span>
        : row.surface.name,
    },
    { key: 'type', header: t('properties.type'), render: (row) => row.kind === 'window'
      ? t(`windowKind.${(project.windowTypes ?? []).find((item) => item.id === row.window!.typeId)?.kind ?? 'window'}`) : t(`surfaceType.${row.surface.type}`) },
    { key: 'boundary', header: t('envelope.boundary'), render: (row) => row.kind === 'window' ? '' : boundary(row.surface) },
    {
      key: 'orientation', header: t('properties.orientation'), render: (row) => row.kind === 'window' ? '—'
        : <span className="envelope-orientation"><Compass orientation={row.surface.orientation} />{t(`orientation.${row.surface.orientation}`)}</span>,
    },
    { key: 'tilt', header: t('envelope.tilt'), numeric: true, render: (row) => { const tilt = surfaceTilt(project, row.surface); return tilt == null ? '—' : `${n(tilt, 0)}°`; } },
    { key: 'area', header: t('envelope.grossArea'), unit: 'm²', numeric: true, render: (row) => n(row.kind === 'window' ? row.window!.area : row.surface.area) },
    {
      key: 'construction', header: t('envelope.construction'), render: (row) => row.kind === 'window'
        ? `${(project.windowTypes ?? []).find((item) => item.id === row.window!.typeId)?.name ?? ''} U_w ${n(row.window!.uValue, 1)} · g ${n(row.window!.gValue, 2)}`.trim()
        : (constructions.get(row.surface.constructionId)?.name ?? <Pill tone="warn">{t('envelope.noConstruction')}</Pill>),
    },
    {
      key: 'u', header: 'U', unit: 'W/m²K', numeric: true, render: (row) => row.kind === 'window'
        ? n(row.window!.uValue, 3)
        : n(constructions.get(row.surface.constructionId)?.uValue, 3),
    },
    { key: 'windows', header: t('browser.windows'), numeric: true, render: (row) => row.kind === 'window' ? '' : String(row.surface.windows.length) },
    {
      key: 'actions', header: <span className="visually-hidden">{t('envelope.actions')}</span>, width: 84, render: (row) => row.kind === 'window'
        ? <ItemActions compact itemType="window" id={row.window!.id} name={row.window!.name} />
        : <ItemActions compact itemType="surface" id={row.surface.id} name={row.surface.name} />,
    },
  ];

  return <>
    <div className="envelope-toolbar">
      <Segmented<KindFilter> size="sm" aria-label={t('envelope.filterType')} value={kind} onChange={setKind} options={[
        { value: 'all', label: `${t('envelope.all')} ${counts.all}` },
        { value: 'wall', label: `${t('envelope.walls')} ${counts.wall}` },
        { value: 'roof', label: `${t('envelope.roofs')} ${counts.roof}` },
        { value: 'floor', label: `${t('envelope.floors')} ${counts.floor}` },
      ]} />
      <label className="envelope-search">
        <Search aria-hidden="true" />
        <span className="visually-hidden">{t('envelope.filter')}</span>
        <input type="search" value={query} placeholder={t('envelope.filterPlaceholder')} onChange={(event) => setQuery(event.target.value)} />
      </label>
      <span className="envelope-toolbar__label" id="envelope-group-label">{t('envelope.groupBy')}</span>
      <Segmented<GroupBy> size="sm" aria-labelledby="envelope-group-label" value={groupBy} onChange={setGroupBy} options={[
        { value: 'orientation', label: t('properties.orientation') },
        { value: 'type', label: t('properties.type') },
        { value: 'construction', label: t('envelope.construction') },
      ]} />
    </div>
    <p className="page-lead envelope-summary">{t('envelope.summary')
      .replace('{surfaces}', String(counts.all)).replace('{windows}', String(windowCount))}</p>
    <DataTable<EnvelopeRow>
      caption={t('nav.sub.building.envelope')}
      columns={columns}
      rows={rows}
      rowKey={(row) => row.key}
      groupBy={(row) => row.group}
      selectedKey={selectedId}
      onSelect={(row) => row.kind === 'window' ? select(row.window!.id, 'window') : select(row.surface.id, 'surface')}
      onActivate={(row) => row.kind === 'window' ? edit('window', row.window!.id) : edit('surface', row.surface.id)}
      rowProps={(row) => ({ 'data-path': row.path, className: row.kind === 'window' ? 'ui-table__row--child' : undefined })}
      empty={project.zones.length === 0 ? t('building.emptyHint') : t('envelope.noMatch')}
    />
    <div className="envelope-cards">
      <TransmissionShare project={project} />
      <EnvelopeChecks project={project} />
    </div>
  </>;
}

/** Share of the indicative H_T per element type, from the project model (U·A, ψ·l, χ). */
export function transmissionShares(project: IProject): Array<{ key: string; value: number }> {
  const constructions = new Map(project.constructions.map((construction) => [construction.id, construction]));
  const sums: Record<string, number> = { windows: 0, wall: 0, roof: 0, floor: 0, bridges: 0 };
  for (const zone of project.zones) {
    for (const surface of zone.surfaces) {
      const u = constructions.get(surface.constructionId)?.uValue ?? 0;
      const key = surface.type === 'internal' ? 'wall' : surface.type;
      sums[key] += u * netArea(surface);
      for (const win of surface.windows) sums.windows += win.uValue * win.area;
    }
    for (const bridge of zone.thermalBridges) sums.bridges += bridge.psiValue * bridge.length;
    for (const bridge of zone.pointThermalBridges ?? []) sums.bridges += bridge.chiValue;
  }
  return Object.entries(sums).map(([key, value]) => ({ key, value }));
}

function TransmissionShare({ project }: { project: IProject }) {
  const { t, locale } = useI18n();
  const shares = transmissionShares(project);
  const total = shares.reduce((sum, share) => sum + share.value, 0);
  return (
    <Card level={2} title={<>{t('envelope.htShare')}</>} subtitle={`${t('envelope.htShareSub')} · ${formatNumber(total, locale, 1)} W/K`}>
      {total <= 0 ? <p className="page-lead">{t('envelope.htShareEmpty')}</p> : <>
        <div className="ht-bar" role="img" aria-label={shares.map((share) => `${t(`envelope.ht.${share.key}`)} ${formatNumber(share.value, locale, 1)} W/K`).join(', ')}>
          {shares.filter((share) => share.value > 0).map((share) => (
            <span key={share.key} className={`ht-seg ht-seg--${share.key}`} style={{ flexGrow: share.value }} />
          ))}
        </div>
        <ul className="ht-legend">
          {shares.filter((share) => share.value > 0).map((share) => (
            <li key={share.key}><span className={`ht-dot ht-seg--${share.key}`} aria-hidden="true" />
              {t(`envelope.ht.${share.key}`)} <b>{formatNumber(share.value, locale, 1)}</b> W/K</li>
          ))}
        </ul>
        <p className="ht-note">{t('envelope.htNote')}</p>
      </>}
    </Card>
  );
}

function EnvelopeChecks({ project }: { project: IProject }) {
  const { t } = useI18n();
  const kernel = useKernel();
  const actions = useShellActions();
  const surfaces = project.zones.flatMap((zone) => zone.surfaces);
  const classified = surfaces.filter((surface) => surface.thermalBoundary).length;
  const withoutConstruction = surfaces.filter((surface) => !project.constructions.some((c) => c.id === surface.constructionId)).length;
  const issues = kernelIssues(kernel?.settled).filter((issue) => routeForPath(issue.path).step === 'building');
  const linearBridges = project.zones.reduce((sum, zone) => sum + zone.thermalBridges.length, 0);
  return (
    <Card level={2} title={t('envelope.checks')} flush>
      <ul className="check-list">
        <li className={classified === surfaces.length ? 'ok' : 'warn'}>
          <b>{t('envelope.check.classified').replace('{n}', String(classified)).replace('{total}', String(surfaces.length))}</b>
          <span>{classified === surfaces.length ? t('envelope.check.classifiedOk') : t('envelope.check.classifiedOpen')}</span>
        </li>
        <li className={withoutConstruction === 0 ? 'ok' : 'warn'}>
          <b>{withoutConstruction === 0 ? t('envelope.check.constructionsOk') : t('envelope.check.constructionsOpen').replace('{n}', String(withoutConstruction))}</b>
        </li>
        {linearBridges === 0 && <li className="info">
          <b>{t('envelope.check.bridgesForfait')}</b>
          <span>{t('envelope.check.bridgesForfaitText')}</span>
        </li>}
      </ul>
      {issues.length > 0 && <IssueList
        issues={issues.map((issue) => ({ code: issue.code, severity: issue.kind, path: issue.path, detail: issue.detail, location: routeLabel(t, routeForPath(issue.path)) }))}
        onGoTo={actions ? (issue) => actions.navigate(routeForPath(issue.path)) : undefined} />}
    </Card>
  );
}

// ── Rekenzones ──────────────────────────────────────────────────────

export function ZonesPage() {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const { project } = state;
  const { selectedId, select, edit } = useSelection();
  const n = (value: number | null | undefined, digits = 1) => formatNumber(value, locale, digits);
  const columns: Array<Column<{ zone: IZone; index: number }>> = [
    { key: 'name', header: t('properties.name'), render: ({ zone }) => zone.name },
    { key: 'ag', header: 'A_g', unit: 'm²', numeric: true, render: ({ zone }) => n(zone.floorArea) },
    { key: 'volume', header: t('zones.volume'), unit: 'm³', numeric: true, render: ({ zone }) => n(zone.volume) },
    { key: 'height', header: t('zones.height'), unit: 'm', numeric: true, render: ({ zone }) => n(zone.height, 2) },
    { key: 'surfaces', header: t('browser.surfaces'), numeric: true, render: ({ zone }) => String(zone.surfaces.length) },
    { key: 'qv10', header: 'q_v;10', unit: 'dm³/(s·m²)', numeric: true, render: ({ zone }) => n(zone.airTightness.qv10, 2) },
    { key: 'actions', header: <span className="visually-hidden">{t('envelope.actions')}</span>, width: 84, render: ({ zone }) => <ItemActions compact itemType="zone" id={zone.id} name={zone.name} /> },
  ];
  const usage = project.ntaCalculation?.usageFunction;
  return <>
    <DataTable
      caption={t('nav.sub.building.zones')}
      columns={columns}
      rows={project.zones.map((zone, index) => ({ zone, index }))}
      rowKey={({ zone }) => zone.id}
      selectedKey={selectedId}
      onSelect={({ zone }) => select(zone.id, 'zone')}
      onActivate={({ zone }) => edit('zone', zone.id)}
      rowProps={({ index }) => ({ 'data-path': formatKernelPath(['zones', index]) })}
      empty={t('building.emptyHint')}
    />
    <Banner tone="info">
      {usage ? t('zones.usage').replace('{usage}', t(`nta.form.usage.${usage}`) === `nta.form.usage.${usage}` ? usage : t(`nta.form.usage.${usage}`)) : t('zones.usageMissing')}
    </Banner>
  </>;
}

// ── Constructies ─────────────────────────────────────────────────────

/** A number field that commits on leaving it or Enter, so typing does not make a construction per keystroke. */
function CommitNumber({ label, value, placeholder, onCommit }: {
  label: string; value: number | null; placeholder?: string; onCommit: (value: number) => void;
}) {
  const shown = value == null ? '' : String(Number(value.toFixed(3)));
  const [text, setText] = useState(shown);
  const [editing, setEditing] = useState(false);
  const commit = () => {
    setEditing(false);
    const number = Number(text.replace(',', '.'));
    if (text.trim() !== '' && Number.isFinite(number) && number >= 0 && text !== shown) onCommit(number);
  };
  return <label>{label}
    <input type="text" inputMode="decimal" value={editing ? text : shown} placeholder={placeholder}
      onFocus={() => { setText(shown); setEditing(true); }} onChange={(event) => setText(event.target.value)} onBlur={commit}
      onKeyDown={(event) => { if (event.key === 'Enter') (event.target as HTMLInputElement).blur(); }} />
  </label>;
}

/** One Rc per kind of surface and one U for all windows, before the per-surface constructions (feedback 8 Oct 2026). */
function ElementRcFields() {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;
  const windows = project.zones.flatMap((zone) => zone.surfaces.flatMap((surface) => surface.windows.map((window) => ({ zoneId: zone.id, surfaceId: surface.id, window }))));
  const windowU = sharedWindowU(project);
  return <Card level={2} title={t('constructions.elementRc.title')} subtitle={t('constructions.elementRc.lead')}>
    <div className="nta-form"><div className="nta-form-grid">
      {ELEMENT_KINDS.map((kind) => {
        const count = surfacesOf(project, kind).length;
        const shared = sharedConstruction(project, kind);
        return <div key={kind} className="element-rc">
          <CommitNumber label={t(`constructions.elementRc.${kind}`)} value={shared?.rcValue ?? null}
            placeholder={count === 0 ? t('constructions.elementRc.none') : t('constructions.elementRc.mixed')}
            onCommit={(rc) => elementRcActions(project, kind, rc, t(`constructions.elementRc.name.${kind}`)).forEach((action) => dispatch(action))} />
          <small className="envelope-meta">{count === 0 ? t('constructions.elementRc.none')
            : shared ? `U ${formatNumber(shared.uValue, locale, 3)} W/m²K · ${t('constructions.elementRc.surfaces', { count })}`
              : t('constructions.elementRc.mixedHint', { count })}</small>
        </div>;
      })}
      <div className="element-rc">
        <CommitNumber label={t('constructions.elementRc.windows')} value={windowU}
          placeholder={windows.length === 0 ? t('constructions.elementRc.none') : t('constructions.elementRc.mixed')}
          onCommit={(u) => {
            if (u <= 0) return;
            // Typed windows follow their type; the rest get the value directly.
            (project.windowTypes ?? []).forEach((type) => dispatch({ type: 'UPDATE_WINDOW_TYPE', payload: { id: type.id, data: { uValue: u } } }));
            windows.filter(({ window }) => !window.typeId).forEach(({ zoneId, surfaceId, window }) =>
              dispatch({ type: 'UPDATE_WINDOW', payload: { zoneId, surfaceId, windowId: window.id, data: { uValue: u } } }));
          }} />
        <small className="envelope-meta">{t('constructions.elementRc.windowCount', { count: windows.length })}</small>
      </div>
    </div></div>
  </Card>;
}

/** The window types (kozijntypen): one row per type, edited in place; a type in use cannot be removed. */
function WindowTypesCard() {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;
  const types = project.windowTypes ?? [];
  const update = (id: string, data: Partial<IWindowType>) => dispatch({ type: 'UPDATE_WINDOW_TYPE', payload: { id, data } });
  const add = () => dispatch({ type: 'ADD_WINDOW_TYPE', payload: {
    id: newWindowTypeId(project), name: `${t('windowKind.window')} ${types.length + 1}`, kind: 'window', uValue: 1.1, gValue: 0.5,
  } });
  return <Card level={2} title={t('constructions.windowTypes.title')} subtitle={t('constructions.windowTypes.lead')} flush
    actions={<Button size="sm" icon={<Plus aria-hidden="true" />} onClick={add}>{t('constructions.windowTypes.add')}</Button>}>
    {types.length === 0 ? <p className="page-lead" style={{ padding: '12px 16px' }}>{t('constructions.windowTypes.empty')}</p>
      : <table className="window-types" aria-label={t('constructions.windowTypes.title')}>
        <thead><tr>
          <th scope="col">{t('constructions.windowTypes.name')}</th><th scope="col">{t('constructions.windowTypes.kind')}</th>
          <th scope="col">U (W/m²K)</th><th scope="col">g (-)</th><th scope="col">{t('constructions.windowTypes.unitArea')}</th>
          <th scope="col">{t('constructions.windowTypes.usedBy')}</th><th scope="col"><span className="visually-hidden">{t('envelope.actions')}</span></th>
        </tr></thead>
        <tbody>{types.map((type) => {
          const used = windowTypeUsage(project, type.id);
          return <tr key={type.id} data-path={`windowTypes[${types.indexOf(type)}]`}>
            <td><input type="text" aria-label={t('constructions.windowTypes.name')} value={type.name} onChange={(event) => update(type.id, { name: event.target.value })} /></td>
            <td><select aria-label={t('constructions.windowTypes.kind')} value={type.kind} onChange={(event) => update(type.id, { kind: event.target.value as IWindowType['kind'] })}>
              {WINDOW_KINDS.map((kind) => <option key={kind} value={kind}>{t(`windowKind.${kind}`)}</option>)}
            </select></td>
            <td><CommitNumber label="U" value={type.uValue} onCommit={(uValue) => update(type.id, { uValue })} /></td>
            <td><CommitNumber label="g" value={type.gValue} onCommit={(gValue) => update(type.id, { gValue })} /></td>
            <td><CommitNumber label={t('constructions.windowTypes.unitArea')} value={type.unitArea ?? null} onCommit={(unitArea) => update(type.id, { unitArea })} /></td>
            <td>{used}</td>
            <td><Button size="sm" variant="ghost" disabled={used > 0} title={used > 0 ? t('constructions.windowTypes.inUse') : undefined}
              onClick={() => dispatch({ type: 'DELETE_WINDOW_TYPE', payload: type.id })}>{t('constructions.windowTypes.delete')}</Button></td>
          </tr>;
        })}</tbody>
      </table>}
  </Card>;
}

export function ConstructionsPage() {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const { project } = state;
  const { selectedId, select, edit } = useSelection();
  const actions = useShellActions();
  const used = (construction: IConstruction) => project.zones.reduce((sum, zone) => sum + zone.surfaces.filter((surface) => surface.constructionId === construction.id).length, 0);
  const columns: Array<Column<{ construction: IConstruction; index: number }>> = [
    { key: 'name', header: t('properties.name'), render: ({ construction }) => construction.name },
    { key: 'layers', header: t('constructions.layers'), numeric: true, render: ({ construction }) => construction.layers.length ? String(construction.layers.length) : '—' },
    { key: 'rc', header: 'R_c', unit: 'm²K/W', numeric: true, render: ({ construction }) => formatNumber(construction.rcValue, locale, 2) },
    { key: 'u', header: 'U', unit: 'W/m²K', numeric: true, render: ({ construction }) => formatNumber(construction.uValue, locale, 3) },
    { key: 'rse', header: 'R_se', unit: 'm²K/W', numeric: true, render: ({ construction }) => formatNumber(construction.exteriorSurfaceResistance ?? 0.04, locale, 2) },
    { key: 'used', header: t('constructions.usedBy'), numeric: true, render: ({ construction }) => String(used(construction)) },
    { key: 'actions', header: <span className="visually-hidden">{t('envelope.actions')}</span>, width: 84, render: ({ construction }) => <ItemActions compact itemType="construction" id={construction.id} name={construction.name} /> },
  ];
  return (<>
    <ElementRcFields />
    <WindowTypesCard />
    <Card level={2} title={t('browser.constructions')} flush actions={actions
      ? <Button size="sm" variant="ghost" onClick={() => actions.navigate({ step: 'tool', sub: 'uvalue' })}>{t('constructions.openCalculator')}</Button>
      : undefined}>
      <DataTable
        caption={t('browser.constructions')}
        columns={columns}
        rows={project.constructions.map((construction, index) => ({ construction, index }))}
        rowKey={({ construction }) => construction.id}
        selectedKey={selectedId}
        onSelect={({ construction }) => select(construction.id, 'construction')}
        onActivate={({ construction }) => edit('construction', construction.id)}
        rowProps={({ index }) => ({ 'data-path': formatKernelPath(['constructions', index]) })}
        empty={t('constructions.empty')}
      />
    </Card>
  </>);
}

// ── Koudebruggen ─────────────────────────────────────────────────────

export function ThermalBridgesPage() {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;
  const { selectedId, select, edit } = useSelection();
  const boundary = (value: string | undefined) => value ? t(`kernel.boundary.${value}`) : t('kernel.boundary.unknown');
  if (project.zones.length === 0) return <p className="page-lead">{t('building.emptyHint')}</p>;
  return <>{project.zones.map((zone, zi) => {
    const linear = zone.thermalBridges.map((bridge, index) => ({ bridge, index }));
    const points = (zone.pointThermalBridges ?? []).map((bridge, index) => ({ bridge, index }));
    const hLinear = zone.thermalBridges.reduce((sum, bridge) => sum + bridge.psiValue * bridge.length, 0);
    return (
      <Card key={zone.id} level={2} title={zone.name} subtitle={`${t('bridges.hTotal')} ${formatNumber(hLinear + points.reduce((sum, { bridge }) => sum + bridge.chiValue, 0), locale, 2)} W/K`} flush>
        <h3 className="card-subhead">{t('browser.thermalBridges')} ({linear.length})</h3>
        <DataTable
          caption={`${zone.name} — ${t('browser.thermalBridges')}`}
          columns={[
            { key: 'name', header: t('properties.name'), render: ({ bridge }) => bridge.name },
            { key: 'psi', header: 'ψ', unit: 'W/(m·K)', numeric: true, render: ({ bridge }) => formatNumber(bridge.psiValue, locale, 3) },
            { key: 'length', header: t('properties.length'), unit: 'm', numeric: true, render: ({ bridge }) => formatNumber(bridge.length, locale, 2) },
            { key: 'h', header: 'H', unit: 'W/K', numeric: true, render: ({ bridge }) => formatNumber(bridge.psiValue * bridge.length, locale, 2) },
            { key: 'boundary', header: t('kernel.boundary.label'), render: ({ bridge }) => boundary(bridge.thermalBoundary) },
            { key: 'actions', header: <span className="visually-hidden">{t('envelope.actions')}</span>, width: 84, render: ({ bridge }) => <ItemActions compact itemType="thermalBridge" id={bridge.id} name={bridge.name} /> },
          ]}
          rows={linear}
          rowKey={({ bridge }) => bridge.id}
          selectedKey={selectedId}
          onSelect={({ bridge }) => select(bridge.id, 'thermalBridge')}
          onActivate={({ bridge }) => edit('thermalBridge', bridge.id)}
          rowProps={({ index }) => ({ 'data-path': formatKernelPath(['zones', zi, 'thermalBridges', index]) })}
          empty={t('bridges.noneLinear')}
        />
        <h3 className="card-subhead">{t('kernel.pointBridge.title')} ({points.length})</h3>
        <div className="card-inline">
          <span className="envelope-meta">{zone.pointBridgeInventoryComplete ? t('kernel.pointBridge.inventoryComplete') : t('kernel.pointBridge.inventoryUnknown')}</span>
          {!zone.pointBridgeInventoryComplete && <Button size="sm" onClick={() => dispatch({ type: 'UPDATE_ZONE', payload: { id: zone.id, data: {
            pointThermalBridges: zone.pointThermalBridges ?? [], pointBridgeInventoryComplete: true,
          } } })}>{t('kernel.pointBridge.confirmInventory')}</Button>}
        </div>
        {points.length > 0 && <DataTable
          caption={`${zone.name} — ${t('kernel.pointBridge.title')}`}
          columns={[
            { key: 'name', header: t('properties.name'), render: ({ bridge }) => bridge.name },
            { key: 'chi', header: 'χ', unit: 'W/K', numeric: true, render: ({ bridge }) => formatNumber(bridge.chiValue, locale, 3) },
            { key: 'boundary', header: t('kernel.boundary.label'), render: ({ bridge }) => boundary(bridge.thermalBoundary) },
            { key: 'source', header: t('kernel.pointBridge.source'), render: ({ bridge }) => bridge.sourceReference },
            { key: 'actions', header: <span className="visually-hidden">{t('kernel.pointBridge.actions')}</span>, width: 84, render: ({ bridge }) => <ItemActions compact itemType="pointBridge" id={bridge.id} name={bridge.name} /> },
          ]}
          rows={points}
          rowKey={({ bridge }) => bridge.id}
          selectedKey={selectedId}
          onSelect={({ bridge }) => select(bridge.id, 'pointBridge')}
          onActivate={({ bridge }) => edit('pointBridge', bridge.id)}
          rowProps={({ index }) => ({ 'data-path': formatKernelPath(['zones', zi, 'pointThermalBridges', index]) })}
        />}
      </Card>
    );
  })}
  <Banner tone="info">{t('bridges.forfaitNote')}</Banner>
  </>;
}

// ── Luchtdichtheid ───────────────────────────────────────────────────

export function AirTightnessPage() {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();
  const { project } = state;
  if (project.zones.length === 0) return <p className="page-lead">{t('building.emptyHint')}</p>;
  return <>{project.zones.map((zone, zi) => (
    <Card key={zone.id} level={2} title={zone.name}>
      <div className="air-row" data-path={formatKernelPath(['zones', zi, 'airTightness'])}>
        <label className="ui-field">
          <span className="ui-field__label">{t('airTightness.qv10')}</span>
          <NumberInput value={zone.airTightness.qv10} unit="dm³/(s·m²)" digits={2} step={0.1}
            aria-label={`${t('airTightness.qv10')} — ${zone.name}`}
            onChange={(value) => {
              if (value == null || value < 0) return;
              dispatch({ type: 'UPDATE_AIR_TIGHTNESS', payload: { zoneId: zone.id, airTightness: { qv10: value } } });
            }} />
        </label>
        {zone.airTightness.assumed && <Pill tone="warn"><span data-testid="air-tightness-assumed">{t('browser.airTightnessAssumed')}</span></Pill>}
      </div>
      <p className="ht-note">{t('airTightness.note')}</p>
    </Card>
  ))}</>;
}

// ── The building step ────────────────────────────────────────────────

export const BUILDING_PAGE_ADD: Record<string, Array<{ dialog: DialogType; labelKey: string; primary?: boolean }>> = {
  envelope: [
    { dialog: 'window-editor', labelKey: 'ribbon.addWindow' },
    { dialog: 'surface-editor', labelKey: 'ribbon.addSurface', primary: true },
  ],
  zones: [{ dialog: 'zone-editor', labelKey: 'ribbon.addZone', primary: true }],
  constructions: [{ dialog: 'construction-editor', labelKey: 'ribbon.addConstruction', primary: true }],
  thermalBridges: [
    { dialog: 'point-bridge', labelKey: 'kernel.pointBridge.add' },
    { dialog: 'thermal-bridge', labelKey: 'ribbon.addThermalBridge', primary: true },
  ],
  airTightness: [{ dialog: 'air-tightness', labelKey: 'ribbon.airTightness', primary: true }],
};

export function BuildingAddBar({ sub }: { sub: string | undefined }) {
  const entries = BUILDING_PAGE_ADD[sub ?? ''];
  return entries ? <AddBar entries={entries} /> : null;
}

/** Lead line of a building page with the key counts (mockup 02). */
export function BuildingLead({ sub }: { sub: string | undefined }): ReactNode {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const kernel = useKernel();
  const { project } = state;
  const surfaces = project.zones.reduce((sum, zone) => sum + zone.surfaces.length, 0);
  const windows = project.zones.reduce((sum, zone) => zone.surfaces.reduce((count, surface) => count + surface.windows.length, sum), 0);
  const lossArea = kernel?.settled?.geometry?.lossAreaM2;
  switch (sub) {
    case 'envelope': return [
      project.zones.length === 1 ? `${t('lead.zone')} ${project.zones[0].name}` : `${project.zones.length} ${t('lead.zones')}`,
      `${surfaces} ${t('lead.surfaces')}`, `${windows} ${t('lead.windows')}`,
      ...(lossArea != null ? [`A_ls ${formatNumber(lossArea, locale, 1)} m²`] : []),
    ].join(' · ');
    case 'zones': return t('lead.building.zones');
    case 'constructions': return t('lead.building.constructions');
    case 'thermalBridges': return t('lead.building.thermalBridges');
    case 'airTightness': return t('lead.building.airTightness');
    default: return t('page.building.lead');
  }
}
