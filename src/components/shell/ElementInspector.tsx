/**
 * Inline editor of the selected building element in the inspector (UI
 * redesign F5, mockup 02). Unit-aware `NumberInput`s update the element in
 * place (`UPDATE_*`, ids stay stable); the full editor stays one click away as
 * a side sheet. An empty required number is not written: the field shows the
 * last valid value again.
 */
import { cloneElement, type ReactElement, type ReactNode } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { formatNumber } from '../../i18n/format';
import type { IProject, ISurface, IWindow, IZone, IThermalBridge, IPointThermalBridge, Orientation, ThermalBoundary } from '../../core/energy/types';
import { NumberInput, Select } from '../ui';
import { ItemActions } from '../ItemActions/ItemActions';
import { netArea, surfaceTilt } from './pages/BuildingPages';
import { useShellActions } from './ShellActions';
import { EvidenceAttach } from '../EvidenceLink/EvidenceLink';

const ORIENTATIONS: Orientation[] = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW', 'horizontal'];
const BOUNDARIES: ThermalBoundary[] = ['outdoor', 'ground', 'unheated_space', 'adjacent_conditioned', 'internal'];

type Found =
  | { type: 'zone'; zone: IZone }
  | { type: 'surface'; zone: IZone; surface: ISurface }
  | { type: 'window'; zone: IZone; surface: ISurface; window: IWindow }
  | { type: 'thermalBridge'; zone: IZone; bridge: IThermalBridge }
  | { type: 'pointBridge'; zone: IZone; bridge: IPointThermalBridge };

/** The selected building element, or null for other selections (systems, constructions, nothing). */
export function findBuildingElement(project: IProject, itemType: string | null, id: string | null): Found | null {
  if (!itemType || !id) return null;
  for (const zone of project.zones) {
    if (itemType === 'zone' && zone.id === id) return { type: 'zone', zone };
    if (itemType === 'thermalBridge') {
      const bridge = zone.thermalBridges.find((item) => item.id === id);
      if (bridge) return { type: 'thermalBridge', zone, bridge };
    }
    if (itemType === 'pointBridge') {
      const bridge = (zone.pointThermalBridges ?? []).find((item) => item.id === id);
      if (bridge) return { type: 'pointBridge', zone, bridge };
    }
    for (const surface of zone.surfaces) {
      if (itemType === 'surface' && surface.id === id) return { type: 'surface', zone, surface };
      if (itemType === 'window') {
        const win = surface.windows.find((item) => item.id === id);
        if (win) return { type: 'window', zone, surface, window: win };
      }
    }
  }
  return null;
}

/** JSON pointer of a found element in the project (`/zones/0/surfaces/2/windows/1`), for evidence links. */
export function elementPointer(project: IProject, found: Found): string {
  const zoneIndex = project.zones.indexOf(found.zone);
  const zone = `/zones/${zoneIndex}`;
  switch (found.type) {
    case 'zone': return zone;
    case 'surface': return `${zone}/surfaces/${found.zone.surfaces.indexOf(found.surface)}`;
    case 'window': return `${zone}/surfaces/${found.zone.surfaces.indexOf(found.surface)}/windows/${found.surface.windows.indexOf(found.window)}`;
    case 'thermalBridge': return `${zone}/thermalBridges/${found.zone.thermalBridges.indexOf(found.bridge)}`;
    case 'pointBridge': return `${zone}/pointThermalBridges/${(found.zone.pointThermalBridges ?? []).indexOf(found.bridge)}`;
  }
}

/** A label and its control; the control gets the label as its accessible name (the unit stays out of it). */
function Row({ label, children }: { label: string; children: ReactElement<{ 'aria-label'?: string }> }) {
  return (
    <div className="element-row">
      <span className="element-row__label" aria-hidden="true">{label}</span>
      <span className="element-row__value">{cloneElement(children, { 'aria-label': label })}</span>
    </div>
  );
}

function Readout({ label, value }: { label: ReactNode; value: ReactNode }) {
  return (
    <div className="element-row element-row--readout">
      <span className="element-row__label">{label}</span>
      <span className="element-row__value ui-num">{value}</span>
    </div>
  );
}

/** A positive number setter that skips empty input (a required field cannot be cleared inline). */
const positive = (apply: (value: number) => void) => (value: number | null | undefined) => {
  if (value == null || !Number.isFinite(value) || value < 0) return;
  apply(value);
};

export function ElementInspector() {
  const { t, locale } = useI18n();
  const { state, dispatch } = useEnergy();
  const actions = useShellActions();
  const { project } = state;
  const found = findBuildingElement(project, state.selectedItemType, state.selectedItemId);
  if (!found) return null;
  const n = (value: number | null | undefined, digits = 2) => formatNumber(value, locale, digits);

  let title: string;
  let kind: string;
  let body: ReactNode;
  let itemType: string;
  let id: string;
  switch (found.type) {
    case 'zone': {
      const { zone } = found;
      const update = (data: Partial<IZone>) => dispatch({ type: 'UPDATE_ZONE', payload: { id: zone.id, data } });
      title = zone.name; kind = t('inspector.kind.zone'); itemType = 'zone'; id = zone.id;
      body = <>
        <Row label={t('properties.area')}><NumberInput value={zone.floorArea} unit="m²" digits={1} onChange={positive((value) => update({ floorArea: value }))} /></Row>
        <Row label={t('properties.volume')}><NumberInput value={zone.volume} unit="m³" digits={1} onChange={positive((value) => update({ volume: value }))} /></Row>
        <Row label={t('properties.height')}><NumberInput value={zone.height} unit="m" digits={2} onChange={positive((value) => update({ height: value }))} /></Row>
      </>;
      break;
    }
    case 'surface': {
      const { zone, surface } = found;
      const update = (data: Partial<ISurface>) => dispatch({ type: 'UPDATE_SURFACE', payload: { zoneId: zone.id, surfaceId: surface.id, data } });
      const construction = project.constructions.find((item) => item.id === surface.constructionId);
      const tilt = surfaceTilt(project, surface);
      title = surface.name; kind = `${t(`surfaceType.${surface.type}`)} · ${zone.name}`; itemType = 'surface'; id = surface.id;
      body = <>
        <h4 className="element-section">{t('inspector.geometry')}</h4>
        <Row label={t('envelope.grossArea')}><NumberInput value={surface.area} unit="m²" digits={2} onChange={positive((value) => update({ area: value }))} /></Row>
        <Readout label={t('envelope.netArea')} value={`${n(netArea(surface))} m²`} />
        <Row label={t('properties.orientation')}>
          <Select value={surface.orientation} onChange={(value) => update({ orientation: value as Orientation })}
            options={ORIENTATIONS.map((value) => ({ value, label: t(`orientation.${value}`) }))} />
        </Row>
        <Readout label={t('envelope.tilt')} value={tilt == null ? '—' : `${n(tilt, 0)}°`} />
        <h4 className="element-section">{t('envelope.construction')}</h4>
        <Row label={t('inspector.construction')}>
          <Select value={surface.constructionId} placeholder={t('envelope.noConstruction')}
            onChange={(value) => { if (value) update({ constructionId: value }); }}
            options={project.constructions.map((item) => ({ value: item.id, label: item.name }))} />
        </Row>
        <Readout label="U" value={construction ? `${n(construction.uValue, 3)} W/m²K` : '—'} />
        <Readout label="R_c" value={construction ? `${n(construction.rcValue, 2)} m²K/W` : '—'} />
        <Row label={t('kernel.boundary.label')}>
          <Select value={surface.thermalBoundary ?? ''} placeholder={t('kernel.boundary.unknown')}
            onChange={(value) => update({ thermalBoundary: (value || undefined) as ThermalBoundary | undefined })}
            options={BOUNDARIES.map((value) => ({ value, label: t(`kernel.boundary.${value}`) }))} />
        </Row>
        {construction && <Readout label="H_T" value={`${n(construction.uValue * netArea(surface), 1)} W/K`} />}
        {actions && <button type="button" className="ui-btn ui-btn--ghost ui-btn--sm element-link"
          onClick={() => actions.navigate({ step: 'tool', sub: 'uvalue' })}>{t('constructions.openCalculator')}</button>}
        <h4 className="element-section">{t('inspector.windows')}</h4>
        <ul className="element-windows">
          {surface.windows.map((win) => (
            <li key={win.id}>
              <button type="button" className="element-window"
                onClick={() => dispatch({ type: 'SELECT_ITEM', payload: { id: win.id, itemType: 'window' } })}>
                <span>{win.name}</span>
                <span className="ui-num">{n(win.area)} m² · U {n(win.uValue, 1)} · g {n(win.gValue, 2)}</span>
              </button>
            </li>
          ))}
        </ul>
        <button type="button" className="ui-btn ui-btn--secondary ui-btn--sm"
          onClick={() => dispatch({ type: 'OPEN_DIALOG', payload: { type: 'window-editor' } })}>{t('inspector.addWindow')}</button>
      </>;
      break;
    }
    case 'window': {
      const { zone, surface, window: win } = found;
      const update = (data: Partial<IWindow>) => dispatch({ type: 'UPDATE_WINDOW', payload: { zoneId: zone.id, surfaceId: surface.id, windowId: win.id, data } });
      title = win.name; kind = `${t('envelope.window')} · ${surface.name}`; itemType = 'window'; id = win.id;
      body = <>
        <Row label={t('properties.area')}><NumberInput value={win.area} unit="m²" digits={2} onChange={positive((value) => update({ area: value }))} /></Row>
        <Row label={t('properties.uValue')}><NumberInput value={win.uValue} unit="W/m²K" digits={2} step={0.1} onChange={positive((value) => update({ uValue: value }))} /></Row>
        <Row label={t('properties.gValue')}><NumberInput value={win.gValue} digits={2} step={0.05} onChange={positive((value) => { if (value <= 1) update({ gValue: value }); })} /></Row>
        <Readout label="H_T" value={`${n(win.uValue * win.area, 1)} W/K`} />
      </>;
      break;
    }
    case 'thermalBridge': {
      const { zone, bridge } = found;
      const update = (data: Partial<IThermalBridge>) => dispatch({ type: 'UPDATE_THERMAL_BRIDGE', payload: { zoneId: zone.id, bridgeId: bridge.id, data } });
      title = bridge.name; kind = `${t('inspector.kind.thermalBridge')} · ${zone.name}`; itemType = 'thermalBridge'; id = bridge.id;
      body = <>
        <Row label="ψ"><NumberInput value={bridge.psiValue} unit="W/(m·K)" digits={3} step={0.01} onChange={(value) => { if (value != null && Number.isFinite(value)) update({ psiValue: value }); }} /></Row>
        <Row label={t('properties.length')}><NumberInput value={bridge.length} unit="m" digits={2} onChange={positive((value) => update({ length: value }))} /></Row>
        <Readout label="H" value={`${n(bridge.psiValue * bridge.length, 2)} W/K`} />
      </>;
      break;
    }
    case 'pointBridge': {
      const { zone, bridge } = found;
      const update = (data: Partial<IPointThermalBridge>) => dispatch({ type: 'UPDATE_POINT_BRIDGE', payload: { zoneId: zone.id, bridgeId: bridge.id, data } });
      title = bridge.name; kind = `${t('kernel.pointBridge.title')} · ${zone.name}`; itemType = 'pointBridge'; id = bridge.id;
      body = <>
        <Row label="χ"><NumberInput value={bridge.chiValue} unit="W/K" digits={3} step={0.01} onChange={(value) => { if (value != null && Number.isFinite(value)) update({ chiValue: value }); }} /></Row>
        <Readout label={t('kernel.pointBridge.source')} value={bridge.sourceReference || '—'} />
      </>;
      break;
    }
  }

  return (
    <section className="element-inspector" aria-label={t('inspector.element')}>
      <header className="element-inspector__head">
        <span className="element-inspector__kind">{kind}</span>
        <h3 className="element-inspector__title">{title}</h3>
        <ItemActions itemType={itemType} id={id} name={title} />
      </header>
      {body}
      <EvidenceAttach pointer={elementPointer(project, found)} hint={t('evidenceLink.elementHint')} />
      <p className="element-inspector__hint">{t('inspector.inlineHint')}</p>
    </section>
  );
}
