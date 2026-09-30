import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import { Building2, Layers, Thermometer, Wind, Zap } from 'lucide-react';
import { KernelAuditPanel } from '../KernelAuditPanel/KernelAuditPanel';
import { HeatPumpInventoryPanel } from '../HeatPumpInventoryPanel/HeatPumpInventoryPanel';
import { UnheatedSpacesPanel } from '../UnheatedSpacesPanel/UnheatedSpacesPanel';
import { GasChainReferencePanel } from '../GasChainReferencePanel/GasChainReferencePanel';
import './ProjectView.css';

export function ProjectView() {
  const { t } = useI18n();
  const { state } = useEnergy();
  const { project } = state;

  const zoneCount = project.zones.length;
  const surfaceCount = project.zones.reduce((sum, z) => sum + z.surfaces.length, 0);
  const windowCount = project.zones.reduce((sum, z) => z.surfaces.reduce((s, surf) => s + surf.windows.length, sum), 0);
  const totalFloorArea = project.zones.reduce((sum, z) => sum + z.floorArea, 0);

  return (
    <div className="project-view">
      <div className="project-view-header">
        <Building2 size={24} />
        <div>
          <h2>{project.name || t('app.untitledProject')}</h2>
          <p>{project.description || t('function.' + project.buildingFunction)}</p>
        </div>
      </div>

      <KernelAuditPanel project={project} />
      <UnheatedSpacesPanel />
      <HeatPumpInventoryPanel />
      <GasChainReferencePanel />

      <div className="project-summary-grid">
        <div className="summary-card">
          <div className="summary-card-icon"><Layers size={20} /></div>
          <div className="summary-card-content">
            <div className="summary-card-value">{zoneCount}</div>
            <div className="summary-card-label">{t('browser.zones')}</div>
          </div>
        </div>

        <div className="summary-card">
          <div className="summary-card-icon"><Building2 size={20} /></div>
          <div className="summary-card-content">
            <div className="summary-card-value">{totalFloorArea.toFixed(0)} m²</div>
            <div className="summary-card-label">{t('properties.area')}</div>
          </div>
        </div>

        <div className="summary-card">
          <div className="summary-card-icon"><Thermometer size={20} /></div>
          <div className="summary-card-content">
            <div className="summary-card-value">{surfaceCount}</div>
            <div className="summary-card-label">{t('browser.surfaces')}</div>
          </div>
        </div>

        <div className="summary-card">
          <div className="summary-card-icon"><Wind size={20} /></div>
          <div className="summary-card-content">
            <div className="summary-card-value">{windowCount}</div>
            <div className="summary-card-label">{t('browser.windows')}</div>
          </div>
        </div>

        <div className="summary-card">
          <div className="summary-card-icon"><Thermometer size={20} /></div>
          <div className="summary-card-content">
            <div className="summary-card-value">{project.heatingSystems.length}</div>
            <div className="summary-card-label">{t('browser.heating')}</div>
          </div>
        </div>

        <div className="summary-card">
          <div className="summary-card-icon"><Zap size={20} /></div>
          <div className="summary-card-content">
            <div className="summary-card-value">{project.solarPV.reduce((sum, pv) => sum + pv.peakPower, 0).toFixed(1)} kWp</div>
            <div className="summary-card-label">{t('browser.solarPV')}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
