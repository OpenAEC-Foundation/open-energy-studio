import { useEffect, useState } from 'react';
import { useI18n } from '../../i18n/i18n';
import { useEnergy } from '../../context/EnergyContext';
import type { RibbonTab, ViewMode } from '../../core/energy/types';
import type { Locale } from '../../i18n/i18n';
import {
  FilePlus, FolderOpen, Save,
  Info, Calculator, Sun, Moon,
  Layers, Square, PanelTop, Grid3X3,
  Thermometer, Wind, Snowflake, Droplets,
  Zap, SunMedium,
  BarChart3, FileText, Printer, Box,
  Plus,
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
  onPrintReport: () => void;
}

export function Ribbon({
  onOpenDialog,
  onCalculate,
  onNewProject,
  onSaveProject,
  onOpenProject,
  onExportReport,
  onExportIFC,
  onPrintReport,
}: RibbonProps) {
  const { t, locale, setLocale } = useI18n();
  const { state, dispatch } = useEnergy();

  const [theme, setTheme] = useState<'dark' | 'light'>(() => {
    const stored = localStorage.getItem('energy-theme');
    return (stored === 'light' || stored === 'dark') ? stored : 'dark';
  });

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem('energy-theme', theme);
  }, [theme]);

  const toggleTheme = () => setTheme(prev => prev === 'dark' ? 'light' : 'dark');

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
  ];

  return (
    <div className="ribbon">
      <div className="ribbon-tabs">
        {tabs.map(tab => (
          <button
            key={tab.id}
            className={`ribbon-tab ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      <div className="ribbon-content">
        {activeTab === 'start' && (
          <>
            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={onNewProject} title={t('ribbon.new')}>
                  <FilePlus size={20} />
                  <span>{t('ribbon.new')}</span>
                </button>
                <button className="ribbon-btn" onClick={onOpenProject} title={t('ribbon.open')}>
                  <FolderOpen size={20} />
                  <span>{t('ribbon.open')}</span>
                </button>
                <button className="ribbon-btn" onClick={onSaveProject} title={t('ribbon.save')}>
                  <Save size={20} />
                  <span>{t('ribbon.save')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.file')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('project-info')} title={t('ribbon.projectInfo')}>
                  <Info size={20} />
                  <span>{t('ribbon.projectInfo')}</span>
                </button>
                <button className="ribbon-btn ribbon-btn-accent" onClick={onCalculate} title={t('ribbon.calculate')}>
                  <Calculator size={20} />
                  <span>{t('ribbon.calculate')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.project')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={toggleTheme} title={t('ribbon.theme')}>
                  {theme === 'dark' ? <Sun size={20} /> : <Moon size={20} />}
                  <span>{t('ribbon.theme')}</span>
                </button>
                <button
                  className="ribbon-btn"
                  onClick={() => setLocale(locale === 'nl' ? 'en' : 'nl' as Locale)}
                  title={t('ribbon.language')}
                >
                  <span className="ribbon-btn-text-icon">{locale.toUpperCase()}</span>
                  <span>{t('ribbon.language')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.settings')}</div>
            </div>
          </>
        )}

        {activeTab === 'envelope' && (
          <>
            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('zone-editor')} title={t('ribbon.addZone')}>
                  <Plus size={16} />
                  <Layers size={20} />
                  <span>{t('ribbon.addZone')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.zones')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('construction-editor')} title={t('ribbon.addConstruction')}>
                  <Plus size={16} />
                  <Grid3X3 size={20} />
                  <span>{t('ribbon.addConstruction')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.constructions')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('surface-editor')} title={t('ribbon.addSurface')}>
                  <Plus size={16} />
                  <Square size={20} />
                  <span>{t('ribbon.addSurface')}</span>
                </button>
                <button className="ribbon-btn" onClick={() => onOpenDialog('window-editor')} title={t('ribbon.addWindow')}>
                  <Plus size={16} />
                  <PanelTop size={20} />
                  <span>{t('ribbon.addWindow')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.surfaces')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('thermal-bridge')} title={t('ribbon.addThermalBridge')}>
                  <Plus size={16} />
                  <Thermometer size={20} />
                  <span>{t('ribbon.addThermalBridge')}</span>
                </button>
                <button className="ribbon-btn" onClick={() => onOpenDialog('air-tightness')} title={t('ribbon.setAirTightness')}>
                  <Wind size={20} />
                  <span>{t('ribbon.airTightness')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.thermalBridges')}</div>
            </div>
          </>
        )}

        {activeTab === 'installations' && (
          <>
            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('heating-system')} title={t('ribbon.addHeating')}>
                  <Plus size={16} />
                  <Thermometer size={20} />
                  <span>{t('ribbon.addHeating')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.heating')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('ventilation-system')} title={t('ribbon.addVentilation')}>
                  <Plus size={16} />
                  <Wind size={20} />
                  <span>{t('ribbon.addVentilation')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.ventilation')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('cooling-system')} title={t('ribbon.addCooling')}>
                  <Plus size={16} />
                  <Snowflake size={20} />
                  <span>{t('ribbon.addCooling')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.cooling')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('hot-water-system')} title={t('ribbon.addHotWater')}>
                  <Plus size={16} />
                  <Droplets size={20} />
                  <span>{t('ribbon.addHotWater')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.hotWater')}</div>
            </div>
          </>
        )}

        {activeTab === 'renewables' && (
          <>
            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('solar-pv')} title={t('ribbon.addSolarPV')}>
                  <Plus size={16} />
                  <Zap size={20} />
                  <span>{t('ribbon.addSolarPV')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.solarPV')}</div>
            </div>

            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={() => onOpenDialog('solar-thermal')} title={t('ribbon.addSolarThermal')}>
                  <Plus size={16} />
                  <SunMedium size={20} />
                  <span>{t('ribbon.addSolarThermal')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.solarThermal')}</div>
            </div>
          </>
        )}

        {activeTab === 'results' && (
          <>
            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn ribbon-btn-accent" onClick={onCalculate} title={t('results.calculate')}>
                  <Calculator size={20} />
                  <span>{t('results.calculate')}</span>
                </button>
                <button className="ribbon-btn" onClick={() => setActiveTab('results')} title={t('ribbon.bengResults')}>
                  <BarChart3 size={20} />
                  <span>{t('ribbon.bengResults')}</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.bengResults')}</div>
            </div>
          </>
        )}

        {activeTab === 'report' && (
          <>
            <div className="ribbon-group">
              <div className="ribbon-group-content">
                <button className="ribbon-btn" onClick={onExportReport} title={t('report.export')}>
                  <FileText size={20} />
                  <span>{t('report.export')}</span>
                </button>
                <button className="ribbon-btn" onClick={onPrintReport} title={t('report.print')}>
                  <Printer size={20} />
                  <span>{t('report.print')}</span>
                </button>
                <button className="ribbon-btn" onClick={onExportIFC} title={t('report.exportIFC')}>
                  <Box size={20} />
                  <span>IFC Export</span>
                </button>
              </div>
              <div className="ribbon-group-title">{t('ribbon.report')}</div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
