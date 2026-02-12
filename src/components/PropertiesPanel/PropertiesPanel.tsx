import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import type { IZone, ISurface, IWindow, IThermalBridge, IConstruction, IHeatingSystem, IVentilationSystem, ICoolingSystem, IHotWaterSystem, ISolarPV, ISolarThermal } from '../../core/energy/types';
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

export function PropertiesPanel() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { item, type } = findItem(state);

  return (
    <div className="properties-panel">
      <div className="properties-panel-header">
        <span>{t('properties.title')}</span>
      </div>
      <div className="properties-panel-content">
        {!item && (
          <div className="properties-empty">{t('properties.noSelection')}</div>
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
