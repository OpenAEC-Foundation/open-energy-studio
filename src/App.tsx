import { useCallback } from 'react';
import { EnergyProvider, useEnergy } from './context/EnergyContext';
import { I18nProvider } from './i18n/I18nProvider';
import { TitleBar } from './components/TitleBar/TitleBar';
import { Ribbon } from './components/Ribbon/Ribbon';
import { ProjectBrowser } from './components/ProjectBrowser/ProjectBrowser';
import { PropertiesPanel } from './components/PropertiesPanel/PropertiesPanel';
import { MainView } from './components/MainView/MainView';
import { StatusBar } from './components/StatusBar/StatusBar';
import { ProjectInfoDialog } from './components/dialogs/ProjectInfoDialog/ProjectInfoDialog';
import { ZoneEditorDialog } from './components/dialogs/ZoneEditorDialog/ZoneEditorDialog';
import { ConstructionEditorDialog } from './components/dialogs/ConstructionEditorDialog/ConstructionEditorDialog';
import { SurfaceEditorDialog } from './components/dialogs/SurfaceEditorDialog/SurfaceEditorDialog';
import { WindowEditorDialog } from './components/dialogs/WindowEditorDialog/WindowEditorDialog';
import { ThermalBridgeDialog } from './components/dialogs/ThermalBridgeDialog/ThermalBridgeDialog';
import { AirTightnessDialog } from './components/dialogs/AirTightnessDialog/AirTightnessDialog';
import { HeatingSystemDialog } from './components/dialogs/HeatingSystemDialog/HeatingSystemDialog';
import { VentilationSystemDialog } from './components/dialogs/VentilationSystemDialog/VentilationSystemDialog';
import { CoolingSystemDialog } from './components/dialogs/CoolingSystemDialog/CoolingSystemDialog';
import { HotWaterSystemDialog } from './components/dialogs/HotWaterSystemDialog/HotWaterSystemDialog';
import { SolarPVDialog } from './components/dialogs/SolarPVDialog/SolarPVDialog';
import { SolarThermalDialog } from './components/dialogs/SolarThermalDialog/SolarThermalDialog';
import { calculateBENGMonthly } from './core/energy/BENGCalculatorMonthly';
import { PreviewPanel } from './components/PreviewPanel/PreviewPanel';
import { downloadReportHTML, printReport } from './core/report/ReportGenerator';
import { downloadBENGIFC } from './core/ifc/IFCEnergyExporter';
import { downloadModelIFC } from './core/ifc/IFCModelExporter';
import { downloadUNIEC3, openUNIEC3FileDialog } from './core/io/UNIEC3Exporter';
import { downloadVABI, openVABIFileDialog } from './core/io/VABIElementsBridge';
import { serializeProject, deserializeProject } from './core/io/ProjectSerializer';
import type { DialogType } from './core/energy/types';

function AppContent() {
  const { state, dispatch } = useEnergy();
  const { dialog, project, result } = state;

  const openDialog = useCallback((type: string) => {
    dispatch({ type: 'OPEN_DIALOG', payload: { type: type as DialogType } });
  }, [dispatch]);

  const closeDialog = useCallback(() => {
    dispatch({ type: 'CLOSE_DIALOG' });
  }, [dispatch]);

  const handleCalculate = useCallback(() => {
    const bengResult = calculateBENGMonthly(project);
    dispatch({ type: 'SET_RESULT', payload: bengResult });
    dispatch({ type: 'SET_VIEW_MODE', payload: 'results' });
    dispatch({ type: 'SET_RIBBON_TAB', payload: 'results' });
  }, [project, dispatch]);

  const handleTogglePreview = useCallback(() => {
    dispatch({ type: 'TOGGLE_PREVIEW' });
  }, [dispatch]);

  const handleNewProject = useCallback(() => {
    dispatch({ type: 'SET_PROJECT', payload: {
      id: crypto.randomUUID(),
      name: '',
      description: '',
      buildingFunction: 'residential',
      address: '',
      city: '',
      zones: [],
      heatingSystems: [],
      ventilationSystems: [],
      coolingSystems: [],
      hotWaterSystems: [],
      solarPV: [],
      solarThermal: [],
      constructions: [],
    }});
  }, [dispatch]);

  const handleSaveProject = useCallback(() => {
    const json = serializeProject(project);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${project.name || 'project'}.oes.json`;
    a.click();
    URL.revokeObjectURL(url);
    dispatch({ type: 'SET_DIRTY', payload: false });
  }, [project, dispatch]);

  const handleOpenProject = useCallback(() => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json,.oes.json';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const loaded = deserializeProject(reader.result as string);
          dispatch({ type: 'SET_PROJECT', payload: loaded });
        } catch (err) {
          alert('Ongeldig projectbestand: ' + (err as Error).message);
        }
      };
      reader.readAsText(file);
    };
    input.click();
  }, [dispatch]);

  const handleExportReport = useCallback(() => {
    if (result) downloadReportHTML(project, result);
  }, [project, result]);

  const handlePrintReport = useCallback(() => {
    if (result) printReport(project, result);
  }, [project, result]);

  const handleExportIFC = useCallback(() => {
    if (result) downloadBENGIFC(project, result);
  }, [project, result]);

  const handleExportModelIFC = useCallback(() => {
    downloadModelIFC(project);
  }, [project]);

  const handleExportUNIEC3 = useCallback(() => {
    if (result) downloadUNIEC3(project, result);
  }, [project, result]);

  const handleImportUNIEC3 = useCallback(async () => {
    try {
      const loaded = await openUNIEC3FileDialog();
      dispatch({ type: 'SET_PROJECT', payload: loaded });
    } catch (err) {
      alert('UNIEC3 import mislukt: ' + (err as Error).message);
    }
  }, [dispatch]);

  const handleExportVABI = useCallback(() => {
    downloadVABI(project);
  }, [project]);

  const handleImportVABI = useCallback(async () => {
    try {
      const loaded = await openVABIFileDialog();
      dispatch({ type: 'SET_PROJECT', payload: loaded });
    } catch (err) {
      alert('VABI import mislukt: ' + (err as Error).message);
    }
  }, [dispatch]);

  return (
    <div className="app">
      <TitleBar
        onNewProject={handleNewProject}
        onOpenProject={handleOpenProject}
        onSaveProject={handleSaveProject}
      />
      <Ribbon
        onOpenDialog={openDialog}
        onCalculate={handleCalculate}
        onNewProject={handleNewProject}
        onSaveProject={handleSaveProject}
        onOpenProject={handleOpenProject}
        onExportReport={handleExportReport}
        onExportIFC={handleExportIFC}
        onExportModelIFC={handleExportModelIFC}
        onPrintReport={handlePrintReport}
        onTogglePreview={handleTogglePreview}
        onExportUNIEC3={handleExportUNIEC3}
        onImportUNIEC3={handleImportUNIEC3}
        onExportVABI={handleExportVABI}
        onImportVABI={handleImportVABI}
      />
      <div className="main-content">
        <ProjectBrowser />
        <MainView />
        {state.previewVisible ? <PreviewPanel /> : <PropertiesPanel />}
      </div>
      <StatusBar />

      {/* Dialogs */}
      {dialog.type === 'project-info' && (
        <ProjectInfoDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'zone-editor' && (
        <ZoneEditorDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'construction-editor' && (
        <ConstructionEditorDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'surface-editor' && (
        <SurfaceEditorDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'window-editor' && (
        <WindowEditorDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'thermal-bridge' && (
        <ThermalBridgeDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'air-tightness' && (
        <AirTightnessDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'heating-system' && (
        <HeatingSystemDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'ventilation-system' && (
        <VentilationSystemDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'cooling-system' && (
        <CoolingSystemDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'hot-water-system' && (
        <HotWaterSystemDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'solar-pv' && (
        <SolarPVDialog editId={dialog.editId} onClose={closeDialog} />
      )}
      {dialog.type === 'solar-thermal' && (
        <SolarThermalDialog editId={dialog.editId} onClose={closeDialog} />
      )}
    </div>
  );
}

export default function App() {
  return (
    <I18nProvider>
      <EnergyProvider>
        <AppContent />
      </EnergyProvider>
    </I18nProvider>
  );
}
