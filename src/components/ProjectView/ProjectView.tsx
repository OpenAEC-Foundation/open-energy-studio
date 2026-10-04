import { useI18n } from '../../i18n/i18n';
import { formatNumber } from '../../i18n/format';
import { useEnergy } from '../../context/EnergyContext';
import type { IProject } from '../../core/energy/types';
import { Building2, Layers, Thermometer, Wind, Zap } from 'lucide-react';
import { KernelAuditPanel } from '../KernelAuditPanel/KernelAuditPanel';
import { NtaPerformancePanel } from '../NtaPerformancePanel/NtaPerformancePanel';
import { HeatPumpInventoryPanel } from '../HeatPumpInventoryPanel/HeatPumpInventoryPanel';
import { UnheatedSpacesPanel } from '../UnheatedSpacesPanel/UnheatedSpacesPanel';
import { BasisopnamePanel } from '../BasisopnamePanel/BasisopnamePanel';
import { GasChainReferencePanel } from '../GasChainReferencePanel/GasChainReferencePanel';
import './ProjectView.css';

/**
 * Installed PV peak power: from the NTA input when it has PV systems (16.4a/16.4b),
 * otherwise from the simplified model. A table 16.1 system has no peak power of
 * its own in the input; the total is then marked as partial.
 */
function projectPvPeak(project: IProject): { kwp: number; partial: boolean } {
  const systems = project.ntaCalculation?.pvSystems ?? [];
  if (systems.length === 0) return { kwp: project.solarPV.reduce((sum, pv) => sum + pv.peakPower, 0), partial: false };
  let watts = 0;
  let partial = false;
  for (const system of systems) {
    const peak = system.peakPower;
    if (peak.method === 'panels') watts += (peak.panelPeakPowerW ?? 0) * (peak.panelCount ?? 0);
    else if (peak.method === 'declared_specific') watts += (peak.peakPowerWPerM2 ?? 0) * (peak.panelAreaM2 ?? 0);
    else partial = true;
  }
  return { kwp: watts / 1000, partial };
}

export function ProjectView() {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const { project } = state;
  const pvPeak = projectPvPeak(project);

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
      <NtaPerformancePanel />
      <BasisopnamePanel />
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
            <div className="summary-card-value">{formatNumber(totalFloorArea, locale, 0)} m²</div>
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
            <div className="summary-card-value">{formatNumber(pvPeak.kwp, locale, 1)}{pvPeak.partial ? '+' : ''} kWp</div>
            <div className="summary-card-label">{t('browser.solarPV')}</div>
          </div>
        </div>
      </div>
    </div>
  );
}
