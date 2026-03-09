import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import type { RibbonTab, ViewMode } from '../../core/energy/types';
import {
  FilePlus, FolderOpen, Save,
  Info, Calculator, Eye,
  Layers, Square, PanelTop, Grid3X3,
  Thermometer, Wind, Snowflake, Droplets,
  Zap, SunMedium,
  BarChart3, FileText, Printer, Box,
  Plus, Download, Upload, Ruler, Box as Box3D,
} from 'lucide-react';
import './Ribbon.css';

interface RibbonProps {
  onOpenDialog: (type: string) => void;
  onCalculate: () => void;
  onNewProject: () => void;
  onSaveProject: () => void;
  onOpenProject: () => void;
  onExportReport: () => void;
  onExportIFC: () => void;
  onExportModelIFC: () => void;
  onPrintReport: () => void;
  onTogglePreview: () => void;
  onExportUNIEC3: () => void;
  onImportUNIEC3: () => void;
  onExportVABI: () => void;
  onImportVABI: () => void;
  onOpenAppMenu: () => void;
}

/* ── Reusable sub-components ── */

function RibbonGroup({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="ribbon-group">
      <div className="ribbon-group-content">{children}</div>
      <div className="ribbon-group-label">{label}</div>
    </div>
  );
}

function RibbonButton({
  icon, label, onClick, title, active, accent, disabled,
}: {
  icon: React.ReactNode;
  label: string;
  onClick?: () => void;
  title?: string;
  active?: boolean;
  accent?: boolean;
  disabled?: boolean;
}) {
  return (
    <button
      className={`ribbon-btn${active ? ' active' : ''}${accent ? ' ribbon-btn-accent' : ''}`}
      onClick={onClick}
      title={title || label}
      disabled={disabled}
    >
      <div className="ribbon-btn-icon">{icon}</div>
      <span className="ribbon-btn-label">{label}</span>
    </button>
  );
}

export function Ribbon({
  onOpenDialog,
  onCalculate,
  onNewProject,
  onSaveProject,
  onOpenProject,
  onExportReport,
  onExportIFC,
  onExportModelIFC,
  onPrintReport,
  onTogglePreview,
  onExportUNIEC3,
  onImportUNIEC3,
  onExportVABI,
  onImportVABI,
  onOpenAppMenu,
}: RibbonProps) {
  const { t } = useI18n();
  const { state, dispatch } = useEnergy();

  const activeTab = state.activeRibbonTab;
  const setActiveTab = (tab: RibbonTab) => {
    dispatch({ type: 'SET_RIBBON_TAB', payload: tab });
    const viewMap: Record<RibbonTab, ViewMode> = {
      start: 'project',
      envelope: 'envelope',
      installations: 'installations',
      renewables: 'renewables',
      results: 'results',
      report: 'report',
      model3d: 'model3d',
      tools: 'uvalue-calc',
    };
    dispatch({ type: 'SET_VIEW_MODE', payload: viewMap[tab] });
  };

  const tabs: { id: RibbonTab; label: string }[] = [
    { id: 'start', label: t('ribbon.start') },
    { id: 'envelope', label: t('ribbon.envelope') },
    { id: 'installations', label: t('ribbon.installations') },
    { id: 'renewables', label: t('ribbon.renewables') },
    { id: 'results', label: t('ribbon.results') },
    { id: 'report', label: t('ribbon.report') },
    { id: 'model3d', label: t('ribbon.model3d') },
    { id: 'tools', label: t('ribbon.tools') },
  ];

  return (
    <div className="ribbon-container">
      {/* ── Tab bar ── */}
      <div className="ribbon-tabs">
        <button className="ribbon-tab file-tab" onClick={onOpenAppMenu}>
          {t('ribbon.file')}
        </button>
        {tabs.map(tab => (
          <button
            key={tab.id}
            className={`ribbon-tab${activeTab === tab.id ? ' active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* ── Content area ── */}
      <div className="ribbon-content">
        <div className="ribbon-groups">
          {activeTab === 'start' && (
            <>
              <RibbonGroup label={t('ribbon.file')}>
                <RibbonButton icon={<FilePlus size={24} />} label={t('ribbon.new')} onClick={onNewProject} />
                <RibbonButton icon={<FolderOpen size={24} />} label={t('ribbon.open')} onClick={onOpenProject} />
                <RibbonButton icon={<Save size={24} />} label={t('ribbon.save')} onClick={onSaveProject} />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.project')}>
                <RibbonButton icon={<Info size={24} />} label={t('ribbon.projectInfo')} onClick={() => onOpenDialog('project-info')} />
                <RibbonButton icon={<Calculator size={24} />} label={t('ribbon.calculate')} onClick={onCalculate} accent />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.settings')}>
                <RibbonButton
                  icon={<Eye size={24} />}
                  label={t('ribbon.preview')}
                  onClick={onTogglePreview}
                  active={state.previewVisible}
                />
              </RibbonGroup>
            </>
          )}

          {activeTab === 'envelope' && (
            <>
              <RibbonGroup label={t('ribbon.zones')}>
                <RibbonButton icon={<><Plus size={12} /><Layers size={24} /></>} label={t('ribbon.addZone')} onClick={() => onOpenDialog('zone-editor')} />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.constructions')}>
                <RibbonButton icon={<><Plus size={12} /><Grid3X3 size={24} /></>} label={t('ribbon.addConstruction')} onClick={() => onOpenDialog('construction-editor')} />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.surfaces')}>
                <RibbonButton icon={<><Plus size={12} /><Square size={24} /></>} label={t('ribbon.addSurface')} onClick={() => onOpenDialog('surface-editor')} />
                <RibbonButton icon={<><Plus size={12} /><PanelTop size={24} /></>} label={t('ribbon.addWindow')} onClick={() => onOpenDialog('window-editor')} />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.thermalBridges')}>
                <RibbonButton icon={<><Plus size={12} /><Thermometer size={24} /></>} label={t('ribbon.addThermalBridge')} onClick={() => onOpenDialog('thermal-bridge')} />
                <RibbonButton icon={<Wind size={24} />} label={t('ribbon.airTightness')} onClick={() => onOpenDialog('air-tightness')} />
              </RibbonGroup>
            </>
          )}

          {activeTab === 'installations' && (
            <>
              <RibbonGroup label={t('ribbon.heating')}>
                <RibbonButton icon={<><Plus size={12} /><Thermometer size={24} /></>} label={t('ribbon.addHeating')} onClick={() => onOpenDialog('heating-system')} />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.ventilation')}>
                <RibbonButton icon={<><Plus size={12} /><Wind size={24} /></>} label={t('ribbon.addVentilation')} onClick={() => onOpenDialog('ventilation-system')} />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.cooling')}>
                <RibbonButton icon={<><Plus size={12} /><Snowflake size={24} /></>} label={t('ribbon.addCooling')} onClick={() => onOpenDialog('cooling-system')} />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.hotWater')}>
                <RibbonButton icon={<><Plus size={12} /><Droplets size={24} /></>} label={t('ribbon.addHotWater')} onClick={() => onOpenDialog('hot-water-system')} />
              </RibbonGroup>
            </>
          )}

          {activeTab === 'renewables' && (
            <>
              <RibbonGroup label={t('ribbon.solarPV')}>
                <RibbonButton icon={<><Plus size={12} /><Zap size={24} /></>} label={t('ribbon.addSolarPV')} onClick={() => onOpenDialog('solar-pv')} />
              </RibbonGroup>

              <RibbonGroup label={t('ribbon.solarThermal')}>
                <RibbonButton icon={<><Plus size={12} /><SunMedium size={24} /></>} label={t('ribbon.addSolarThermal')} onClick={() => onOpenDialog('solar-thermal')} />
              </RibbonGroup>
            </>
          )}

          {activeTab === 'results' && (
            <RibbonGroup label={t('ribbon.bengResults')}>
              <RibbonButton icon={<Calculator size={24} />} label={t('results.calculate')} onClick={onCalculate} accent />
              <RibbonButton icon={<BarChart3 size={24} />} label={t('ribbon.bengResults')} onClick={() => setActiveTab('results')} />
            </RibbonGroup>
          )}

          {activeTab === 'report' && (
            <>
              <RibbonGroup label={t('ribbon.report')}>
                <RibbonButton icon={<FileText size={24} />} label={t('report.export')} onClick={onExportReport} />
                <RibbonButton icon={<Printer size={24} />} label={t('report.print')} onClick={onPrintReport} />
                <RibbonButton icon={<Box size={24} />} label="IFC Export" onClick={onExportIFC} />
              </RibbonGroup>

              <RibbonGroup label="UNIEC3">
                <RibbonButton icon={<Download size={24} />} label={t('ribbon.exportUNIEC3')} onClick={onExportUNIEC3} />
                <RibbonButton icon={<Upload size={24} />} label={t('ribbon.importUNIEC3')} onClick={onImportUNIEC3} />
              </RibbonGroup>

              <RibbonGroup label="VABI Elements">
                <RibbonButton icon={<Download size={24} />} label={t('ribbon.exportVABI')} onClick={onExportVABI} />
                <RibbonButton icon={<Upload size={24} />} label={t('ribbon.importVABI')} onClick={onImportVABI} />
              </RibbonGroup>
            </>
          )}

          {activeTab === 'model3d' && (
            <RibbonGroup label={t('ribbon.model3d')}>
              <RibbonButton icon={<Box3D size={24} />} label={t('ribbon.exportModelIFC')} onClick={onExportModelIFC} />
            </RibbonGroup>
          )}

          {activeTab === 'tools' && (
            <RibbonGroup label={t('ribbon.tools')}>
              <RibbonButton
                icon={<Ruler size={24} />}
                label={t('ribbon.uvalueCalc')}
                onClick={() => dispatch({ type: 'SET_VIEW_MODE', payload: 'uvalue-calc' })}
                active={state.viewMode === 'uvalue-calc'}
              />
              <RibbonButton
                icon={<Thermometer size={24} />}
                label={t('ribbon.thermalBridgeCalc')}
                onClick={() => dispatch({ type: 'SET_VIEW_MODE', payload: 'thermal-bridge-calc' })}
                active={state.viewMode === 'thermal-bridge-calc'}
              />
              <RibbonButton
                icon={<Zap size={24} />}
                label={t('ribbon.heatPumpSizing')}
                onClick={() => dispatch({ type: 'SET_VIEW_MODE', payload: 'heat-pump-sizing' })}
                active={state.viewMode === 'heat-pump-sizing'}
              />
            </RibbonGroup>
          )}
        </div>
      </div>
    </div>
  );
}
