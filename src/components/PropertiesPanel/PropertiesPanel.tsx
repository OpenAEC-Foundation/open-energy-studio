import { useState, useRef, useCallback, useEffect } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { PanelRightClose } from 'lucide-react';
import type { IZone, ISurface, IWindow, IThermalBridge, IPointThermalBridge, IConstruction, IHeatingSystem, IVentilationSystem, ICoolingSystem, IHotWaterSystem, ISolarPV, ISolarThermal } from '../../core/energy/types';
import { ItemActions } from '../ItemActions/ItemActions';
import './PropertiesPanel.css';

function findItem(state: ReturnType<typeof useEnergy>['state']): { item: unknown; type: string | null } {
  const { project, selectedItemId, selectedItemType } = state;
  if (!selectedItemId || !selectedItemType) return { item: null, type: null };

  switch (selectedItemType) {
    case 'zone':
      return { item: project.zones.find(z => z.id === selectedItemId), type: 'zone' };
    case 'surface':
      for (const z of project.zones)
        for (const s of z.surfaces)
          if (s.id === selectedItemId) return { item: s, type: 'surface' };
      return { item: null, type: null };
    case 'window':
      for (const z of project.zones)
        for (const s of z.surfaces)
          for (const w of s.windows)
            if (w.id === selectedItemId) return { item: w, type: 'window' };
      return { item: null, type: null };
    case 'thermalBridge':
      for (const z of project.zones)
        for (const tb of z.thermalBridges)
          if (tb.id === selectedItemId) return { item: tb, type: 'thermalBridge' };
      return { item: null, type: null };
    case 'pointBridge':
      for (const z of project.zones)
        for (const bridge of z.pointThermalBridges ?? [])
          if (bridge.id === selectedItemId) return { item: bridge, type: 'pointBridge' };
      return { item: null, type: null };
    case 'construction':
      return { item: project.constructions.find(c => c.id === selectedItemId), type: 'construction' };
    case 'heatingSystem':
      return { item: project.heatingSystems.find(h => h.id === selectedItemId), type: 'heatingSystem' };
    case 'ventilationSystem':
      return { item: project.ventilationSystems.find(v => v.id === selectedItemId), type: 'ventilationSystem' };
    case 'coolingSystem':
      return { item: project.coolingSystems.find(c => c.id === selectedItemId), type: 'coolingSystem' };
    case 'hotWaterSystem':
      return { item: project.hotWaterSystems.find(h => h.id === selectedItemId), type: 'hotWaterSystem' };
    case 'solarPV':
      return { item: project.solarPV.find(p => p.id === selectedItemId), type: 'solarPV' };
    case 'solarThermal':
      return { item: project.solarThermal.find(s => s.id === selectedItemId), type: 'solarThermal' };
    default:
      return { item: null, type: null };
  }
}

function PropertyRow({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="property-row">
      <span className="property-label">{label}</span>
      <span className="property-value">{value}</span>
    </div>
  );
}

/** `embedded`: rendered inside the shell inspector, which owns the frame, header and width. */
export function PropertiesPanel({ embedded = false }: { embedded?: boolean } = {}) {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { item, type } = findItem(state);
  const [collapsed, setCollapsed] = useState(false);
  const [width, setWidth] = useState(280);
  const resizing = useRef(false);
  const startX = useRef(0);
  const startWidth = useRef(0);

  const onResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    resizing.current = true;
    startX.current = e.clientX;
    startWidth.current = width;
  }, [width]);

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!resizing.current) return;
      const newWidth = Math.min(500, Math.max(160, startWidth.current - (e.clientX - startX.current)));
      setWidth(newWidth);
    };
    const onMouseUp = () => { resizing.current = false; };
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    return () => {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    };
  }, []);

  if (collapsed && !embedded) {
    return (
      <div className="properties-panel collapsed" onClick={() => setCollapsed(false)}>
        <div className="panel-collapsed-label-right">
          <span>{t('properties.title')}</span>
        </div>
      </div>
    );
  }

  return (
    <div className={embedded ? 'properties-panel embedded' : 'properties-panel'} style={embedded ? undefined : { width }}>
      {!embedded && <>
        <div className="panel-resize-handle panel-resize-handle-left" onMouseDown={onResizeStart} />
        <div className="properties-panel-header">
          <button className="panel-collapse-btn-right" onClick={() => setCollapsed(true)}><PanelRightClose size={14} /></button>
          <span>{t('properties.title')}</span>
        </div>
      </>}
      <div className="properties-panel-content">
        {!item && (
          <div className="properties-empty">{t('properties.noSelection')}</div>
        )}

        {Boolean(item) && type && state.selectedItemId && (
          <div className="properties-actions">
            <ItemActions itemType={type} id={state.selectedItemId}
              name={(item as { name?: string }).name ?? state.selectedItemId} />
          </div>
        )}

        {type === 'zone' && (() => {
          const zone = item as IZone;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={zone.name} />
              <PropertyRow label={t('properties.area')} value={`${zone.floorArea} m²`} />
              <PropertyRow label={t('properties.volume')} value={`${zone.volume} m³`} />
              <PropertyRow label={t('properties.height')} value={`${zone.height} m`} />
            </>
          );
        })()}

        {type === 'surface' && (() => {
          const surface = item as ISurface;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={surface.name} />
              <PropertyRow label={t('properties.type')} value={t(`surfaceType.${surface.type}`)} />
              <PropertyRow label={t('properties.area')} value={`${surface.area} m²`} />
              <PropertyRow label={t('properties.orientation')} value={t(`orientation.${surface.orientation}`)} />
            </>
          );
        })()}

        {type === 'window' && (() => {
          const win = item as IWindow;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={win.name} />
              <PropertyRow label={t('properties.area')} value={`${win.area} m²`} />
              <PropertyRow label={t('properties.uValue')} value={`${win.uValue} W/m²K`} />
              <PropertyRow label={t('properties.gValue')} value={`${win.gValue}`} />
              <PropertyRow label={t('properties.orientation')} value={t(`orientation.${win.orientation}`)} />
            </>
          );
        })()}

        {type === 'thermalBridge' && (() => {
          const tb = item as IThermalBridge;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={tb.name} />
              <PropertyRow label={t('properties.psiValue')} value={`${tb.psiValue} W/mK`} />
              <PropertyRow label={t('properties.length')} value={`${tb.length} m`} />
            </>
          );
        })()}

        {type === 'pointBridge' && (() => {
          const bridge = item as IPointThermalBridge;
          return <>
            <PropertyRow label={t('properties.name')} value={bridge.name} />
            <PropertyRow label={t('kernel.pointBridge.chi')} value={`${bridge.chiValue} W/K`} />
            <PropertyRow label={t('kernel.pointBridge.source')} value={bridge.sourceReference} />
          </>;
        })()}

        {type === 'construction' && (() => {
          const c = item as IConstruction;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={c.name} />
              <PropertyRow label={t('properties.rcValue')} value={`${c.rcValue.toFixed(2)} m²K/W`} />
              <PropertyRow label={t('properties.uValue')} value={`${c.uValue.toFixed(3)} W/m²K`} />
            </>
          );
        })()}

        {type === 'heatingSystem' && (() => {
          const h = item as IHeatingSystem;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={h.name} />
              <PropertyRow label={t('properties.type')} value={h.type} />
              <PropertyRow label={t('properties.cop')} value={h.cop.toFixed(2)} />
              <PropertyRow label={t('properties.coverage')} value={`${(h.coverageFraction * 100).toFixed(0)}%`} />
            </>
          );
        })()}

        {type === 'ventilationSystem' && (() => {
          const v = item as IVentilationSystem;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={v.name} />
              <PropertyRow label={t('properties.type')} value={v.type} />
              <PropertyRow label={t('properties.efficiency')} value={`${(v.heatRecoveryEfficiency * 100).toFixed(0)}%`} />
            </>
          );
        })()}

        {type === 'coolingSystem' && (() => {
          const c = item as ICoolingSystem;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={c.name} />
              <PropertyRow label={t('properties.type')} value={c.type} />
              <PropertyRow label={t('properties.eer')} value={c.eer.toFixed(2)} />
            </>
          );
        })()}

        {type === 'hotWaterSystem' && (() => {
          const hw = item as IHotWaterSystem;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={hw.name} />
              <PropertyRow label={t('properties.type')} value={hw.type} />
              <PropertyRow label={t('properties.efficiency')} value={hw.efficiency.toFixed(2)} />
            </>
          );
        })()}

        {type === 'solarPV' && (() => {
          const pv = item as ISolarPV;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={pv.name} />
              <PropertyRow label={t('properties.peakPower')} value={`${pv.peakPower} kWp`} />
              <PropertyRow label={t('properties.orientation')} value={t(`orientation.${pv.orientation}`)} />
              <PropertyRow label={t('properties.tilt')} value={`${pv.tilt}°`} />
            </>
          );
        })()}

        {type === 'solarThermal' && (() => {
          const st = item as ISolarThermal;
          return (
            <>
              <PropertyRow label={t('properties.name')} value={st.name} />
              <PropertyRow label={t('properties.area')} value={`${st.collectorArea} m²`} />
              <PropertyRow label={t('properties.orientation')} value={t(`orientation.${st.orientation}`)} />
              <PropertyRow label={t('properties.tilt')} value={`${st.tilt}°`} />
            </>
          );
        })()}
      </div>
    </div>
  );
}
