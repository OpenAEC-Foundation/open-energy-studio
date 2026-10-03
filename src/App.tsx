import { useCallback, useEffect, useRef, useState } from 'react';
import { EnergyProvider, useEnergy, useDocumentManager, useHasActiveDocument } from './context/EnergyContext';
import { I18nProvider } from './i18n/I18nProvider';
import { TitleBar } from './components/TitleBar/TitleBar';
import { DocumentTabs } from './components/DocumentTabs/DocumentTabs';
import { WelcomeScreen } from './components/WelcomeScreen/WelcomeScreen';
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
import { PointBridgeDialog } from './components/dialogs/PointBridgeDialog/PointBridgeDialog';
import { AirTightnessDialog } from './components/dialogs/AirTightnessDialog/AirTightnessDialog';
import { HeatingSystemDialog } from './components/dialogs/HeatingSystemDialog/HeatingSystemDialog';
import { VentilationSystemDialog } from './components/dialogs/VentilationSystemDialog/VentilationSystemDialog';
import { CoolingSystemDialog } from './components/dialogs/CoolingSystemDialog/CoolingSystemDialog';
import { HotWaterSystemDialog } from './components/dialogs/HotWaterSystemDialog/HotWaterSystemDialog';
import { SolarPVDialog } from './components/dialogs/SolarPVDialog/SolarPVDialog';
import { SolarThermalDialog } from './components/dialogs/SolarThermalDialog/SolarThermalDialog';
import { PrintPreviewDialog } from './components/dialogs/PrintPreviewDialog/PrintPreviewDialog';
import { calculateBENGMonthly } from './core/energy/BENGCalculatorMonthly';
import { hasUnmodelledHeatPumpDetails, legacyHeatPumpInputIssue, validProjectFloorArea } from './core/energy/ProjectArea';
import { useI18n } from './i18n/i18n';
import { PreviewPanel } from './components/PreviewPanel/PreviewPanel';
import { downloadReportHTML } from './core/report/ReportGenerator';
import { downloadBENGIFC } from './core/ifc/IFCEnergyExporter';
import { downloadModelIFC } from './core/ifc/IFCModelExporter';
import { downloadUNIEC3, openUNIEC3FileDialog } from './core/io/UNIEC3Exporter';
import { downloadVABI, openVABIFileDialog } from './core/io/VABIElementsBridge';
import { serializeProject, deserializeProjectFile, compareKernelStamp, describeStamp } from './core/io/ProjectSerializer';
import { migrateLegacyRelabel } from './core/nta/Registration';

/** Shows the relabel migration notice once per project (per browser profile). */
function relabelNoticeShown(projectId: string | undefined): boolean {
  const key = `oes-relabel-migration-notice:${projectId ?? 'unknown'}`;
  try {
    if (localStorage.getItem(key)) return true;
    localStorage.setItem(key, '1');
  } catch {
    // Storage unavailable: show the notice each time.
  }
  return false;
}
import { stampProject } from './core/io/KernelStampClient';
import { EXAMPLE_KINDS, exampleProject, type ExampleKind } from './core/nta/ExampleProjects';
import { AppMenu } from './components/AppMenu/AppMenu';
import type { DialogType, IProject } from './core/energy/types';

// ── Active document workspace (only rendered when a document is open) ──

function ActiveDocumentContent({
  onNewProject,
  onOpenProject,
  onSaveProject,
  onSaveAsProject,
  onCloseTab,
}: {
  onNewProject: () => void;
  onOpenProject: () => void;
  onSaveProject: () => void;
  onSaveAsProject: () => void;
  onCloseTab: (id: string) => void;
}) {
  const { state, dispatch } = useEnergy();
  const { t, locale } = useI18n();
  const { dialog, project, result } = state;
  const [appMenuOpen, setAppMenuOpen] = useState(false);
  const [printPreviewOpen, setPrintPreviewOpen] = useState(false);
  const [calculationError, setCalculationError] = useState<string | null>(null);

  useEffect(() => { setCalculationError(null); }, [project]);

  const openDialog = useCallback((type: string) => {
    dispatch({ type: 'OPEN_DIALOG', payload: { type: type as DialogType } });
  }, [dispatch]);

  const closeDialog = useCallback(() => {
    dispatch({ type: 'CLOSE_DIALOG' });
  }, [dispatch]);

  const handleCalculate = useCallback(() => {
    if (project.ntaHeatPumps?.length) {
      dispatch({ type: 'SET_RESULT', payload: null });
      setCalculationError(t('calculation.standaloneHeatPumps'));
      return;
    }
    if (hasUnmodelledHeatPumpDetails(project)) {
      dispatch({ type: 'SET_RESULT', payload: null });
      setCalculationError(t('calculation.performancePointsUnsupported'));
      return;
    }
    if (validProjectFloorArea(project) === null) {
      dispatch({ type: 'SET_RESULT', payload: null });
      setCalculationError(t('calculation.invalidFloorArea'));
      return;
    }
    const heatPumpIssue = legacyHeatPumpInputIssue(project);
    if (heatPumpIssue) {
      dispatch({ type: 'SET_RESULT', payload: null });
      setCalculationError(t(heatPumpIssue === 'cop'
        ? 'calculation.invalidHeatPumpCop' : 'calculation.invalidHeatPumpCoverage'));
      return;
    }
    try {
      const bengResult = calculateBENGMonthly(project);
      dispatch({ type: 'SET_RESULT', payload: bengResult });
      dispatch({ type: 'SET_VIEW_MODE', payload: 'results' });
      dispatch({ type: 'SET_RIBBON_TAB', payload: 'results' });
      setCalculationError(null);
    } catch (error) {
      dispatch({ type: 'SET_RESULT', payload: null });
      setCalculationError(error instanceof Error ? error.message : String(error));
    }
  }, [project, dispatch, t]);

  const handleTogglePreview = useCallback(() => {
    dispatch({ type: 'TOGGLE_PREVIEW' });
  }, [dispatch]);

  const handleExportReport = useCallback(() => {
    downloadReportHTML(project, result, locale).catch((error: unknown) =>
      setCalculationError(error instanceof Error ? error.message : String(error)));
  }, [project, result, locale]);

  const handlePrintReport = useCallback(() => {
    setPrintPreviewOpen(true);
  }, []);

  const handleExportIFC = useCallback(() => {
    if (result) downloadBENGIFC(project, result);
  }, [project, result]);

  const handleExportModelIFC = useCallback(() => {
    downloadModelIFC(project);
  }, [project]);

  const handleExportUNIEC3 = useCallback(() => {
    downloadUNIEC3(project);
  }, [project]);

  const { docDispatch } = useDocumentManager();

  const handleImportUNIEC3 = useCallback(async () => {
    try {
      const loaded = await openUNIEC3FileDialog();
      docDispatch({ type: 'DOC_NEW', payload: { id: crypto.randomUUID(), project: loaded } });
    } catch (err) {
      alert('UNIEC3 import mislukt: ' + (err as Error).message);
    }
  }, [docDispatch]);

  const handleExportVABI = useCallback(() => {
    downloadVABI(project);
  }, [project]);

  const handleImportVABI = useCallback(async () => {
    try {
      const loaded = await openVABIFileDialog();
      docDispatch({ type: 'DOC_NEW', payload: { id: crypto.randomUUID(), project: loaded } });
    } catch (err) {
      alert('VABI import mislukt: ' + (err as Error).message);
    }
  }, [docDispatch]);

  return (
    <>
      <Ribbon
        onOpenDialog={openDialog}
        onCalculate={handleCalculate}
        onNewProject={onNewProject}
        onSaveProject={onSaveProject}
        onOpenProject={onOpenProject}
        onExportReport={handleExportReport}
        onExportIFC={handleExportIFC}
        onExportModelIFC={handleExportModelIFC}
        onPrintReport={handlePrintReport}
        onTogglePreview={handleTogglePreview}
        onExportUNIEC3={handleExportUNIEC3}
        onImportUNIEC3={handleImportUNIEC3}
        onExportVABI={handleExportVABI}
        onImportVABI={handleImportVABI}
        onOpenAppMenu={() => setAppMenuOpen(true)}
      />
      <DocumentTabs onCloseTab={onCloseTab} onNewProject={onNewProject} onOpenProject={onOpenProject} />
      {calculationError && <div className="calculation-input-error" role="alert">{calculationError}</div>}
      {appMenuOpen && (
        <AppMenu
          isOpen={appMenuOpen}
          onClose={() => setAppMenuOpen(false)}
          onNewProject={onNewProject}
          onOpenProject={onOpenProject}
          onSaveProject={onSaveProject}
          onSaveAsProject={onSaveAsProject}
          onExportReport={handleExportReport}
          onExportIFC={handleExportIFC}
          onExportModelIFC={handleExportModelIFC}
          onPrintReport={handlePrintReport}
          onExportUNIEC3={handleExportUNIEC3}
          onImportUNIEC3={handleImportUNIEC3}
          onExportVABI={handleExportVABI}
          onImportVABI={handleImportVABI}
          onOpenDialog={openDialog}
        />
      )}
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
      {dialog.type === 'point-bridge' && (
        <PointBridgeDialog editId={dialog.editId} onClose={closeDialog} />
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
      {printPreviewOpen && (
        <PrintPreviewDialog onClose={() => setPrintPreviewOpen(false)} />
      )}
    </>
  );
}

// ── Minimal status bar when no document is open ──

function EmptyStatusBar() {
  return (
    <div className="status-bar">
      <div className="status-section">
        <span className="status-hint">Ready</span>
      </div>
      <div className="status-section" />
    </div>
  );
}

// ── Main app shell ──

function AppContent() {
  const { docState, docDispatch } = useDocumentManager();
  const hasActiveDoc = useHasActiveDocument();
  const { t } = useI18n();
  const untitledCounter = useRef(0);

  const createEmptyProject = useCallback((): IProject => {
    untitledCounter.current += 1;
    return {
      id: crypto.randomUUID(),
      name: `Untitled ${untitledCounter.current}`,
      description: '',
      buildingFunction: 'residential',
      address: '',
      city: '',
      zones: [],
      heatingSystems: [],
      ventilationSystems: [],
      coolingSystems: [],
      hotWaterSystems: [],
      ntaHeatPumps: [],
      solarPV: [],
      solarThermal: [],
      constructions: [],
    };
  }, []);

  // ── Helper: write project to disk (returns filePath or null if cancelled) ──
  const writeProjectToDisk = useCallback(async (
    project: IProject,
    existingPath: string | null,
    forcePrompt: boolean,
  ): Promise<string | null> => {
    const json = serializeProject(project, await stampProject(project));

    // Save directly if we have a path and aren't forcing a prompt
    if (existingPath && !forcePrompt) {
      try {
        const { writeTextFile } = await import('@tauri-apps/plugin-fs');
        await writeTextFile(existingPath, json);
        return existingPath;
      } catch { /* fall through to Save As dialog */ }
    }

    // Show Save As dialog
    try {
      const { save } = await import('@tauri-apps/plugin-dialog');
      const { writeTextFile } = await import('@tauri-apps/plugin-fs');
      const filePath = await save({
        defaultPath: existingPath || `${project.name || 'project'}.oes.json`,
        filters: [{ name: 'OES Project', extensions: ['oes.json', 'json'] }],
      });
      if (!filePath) return null; // User cancelled
      await writeTextFile(filePath, json);
      return filePath;
    } catch {
      // Browser fallback (non-Tauri)
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${project.name || 'project'}.oes.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      return 'browser-download';
    }
  }, []);

  // ── New ──
  const handleNewProject = useCallback(() => {
    docDispatch({ type: 'DOC_NEW', payload: { id: crypto.randomUUID(), project: createEmptyProject() } });
  }, [docDispatch, createEmptyProject]);

  const handleOpenExample = useCallback((kind: ExampleKind) => {
    docDispatch({ type: 'DOC_NEW', payload: { id: crypto.randomUUID(), project: exampleProject(kind) } });
  }, [docDispatch]);

  // `?example=small_office` opens an example at start (demo and review links).
  const exampleFromUrl = useRef(false);
  useEffect(() => {
    if (exampleFromUrl.current) return;
    exampleFromUrl.current = true;
    const kind = new URLSearchParams(window.location.search).get('example');
    if (EXAMPLE_KINDS.includes(kind as ExampleKind)) handleOpenExample(kind as ExampleKind);
  }, [handleOpenExample]);

  // ── Open ──
  const handleOpenProject = useCallback(async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      const filePath = await open({
        filters: [{ name: 'OES Project', extensions: ['oes.json', 'json'] }],
        multiple: false,
      });
      if (!filePath) return;
      const json = await readTextFile(filePath as string);
      const { project: opened, kernel: saved } = deserializeProjectFile(json);
      const { project: loaded, missing } = migrateLegacyRelabel(opened);
      docDispatch({ type: 'DOC_OPEN', payload: { id: crypto.randomUUID(), project: loaded, filePath: filePath as string } });
      if (missing.length > 0 && !relabelNoticeShown(loaded.id)) {
        alert(t('relabel.migrationNotice', { fields: missing.map((field) => t(`relabel.migrationField.${field}`)).join(', ') }));
      }
      const current = await stampProject(loaded);
      const [difference] = compareKernelStamp(saved, current);
      if (difference === 'version' && saved && current) {
        alert(t('project.kernelChanged', { saved: describeStamp(saved), current: describeStamp(current) }));
      } else if (difference === 'input') {
        alert(t('project.inputChanged'));
      }
    } catch (err) {
      const msg = (err as Error).message;
      if (msg && !msg.includes('cancelled')) {
        alert('Failed to open project: ' + msg);
      }
    }
  }, [docDispatch, t]);

  // ── Save (Ctrl+S) — save to existing path, or prompt Save As if new ──
  const handleSaveProject = useCallback(async () => {
    const activeDoc = docState.documents.find(d => d.id === docState.activeDocumentId);
    if (!activeDoc) return;

    const savedPath = await writeProjectToDisk(activeDoc.state.project, activeDoc.filePath, false);
    if (savedPath === null) return; // Cancelled

    if (savedPath !== 'browser-download' && savedPath !== activeDoc.filePath) {
      docDispatch({ type: 'DOC_SET_FILE_PATH', payload: { id: activeDoc.id, filePath: savedPath } });
    }
    docDispatch({ type: 'DOC_DISPATCH', payload: { id: activeDoc.id, action: { type: 'SET_DIRTY', payload: false } } });
  }, [docState, docDispatch, writeProjectToDisk]);

  // ── Save As (always prompts for new path) ──
  const handleSaveAsProject = useCallback(async () => {
    const activeDoc = docState.documents.find(d => d.id === docState.activeDocumentId);
    if (!activeDoc) return;

    const savedPath = await writeProjectToDisk(activeDoc.state.project, activeDoc.filePath, true);
    if (savedPath === null) return; // Cancelled

    if (savedPath !== 'browser-download') {
      docDispatch({ type: 'DOC_SET_FILE_PATH', payload: { id: activeDoc.id, filePath: savedPath } });
    }
    docDispatch({ type: 'DOC_DISPATCH', payload: { id: activeDoc.id, action: { type: 'SET_DIRTY', payload: false } } });
  }, [docState, docDispatch, writeProjectToDisk]);

  // ── Close tab (with unsaved-changes check) ──
  const handleCloseTab = useCallback(async (id: string) => {
    const doc = docState.documents.find(d => d.id === id);
    if (!doc) return;

    if (doc.state.isDirty) {
      try {
        const { ask } = await import('@tauri-apps/plugin-dialog');
        const shouldSave = await ask(
          `"${doc.state.project.name || 'Untitled'}" has unsaved changes. Save before closing?`,
          { title: 'Unsaved Changes', kind: 'warning', okLabel: 'Save', cancelLabel: 'Discard' },
        );
        if (shouldSave) {
          const savedPath = await writeProjectToDisk(doc.state.project, doc.filePath, false);
          if (savedPath === null) return; // User cancelled Save As — don't close
        }
      } catch {
        const shouldDiscard = confirm(
          `"${doc.state.project.name || 'Untitled'}" has unsaved changes. Discard and close?`
        );
        if (!shouldDiscard) return;
      }
    }

    docDispatch({ type: 'DOC_CLOSE', payload: id });
  }, [docState.documents, docDispatch, writeProjectToDisk]);

  // ── Keyboard shortcuts ──
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const ctrl = e.ctrlKey || e.metaKey;
      if (!ctrl) return;

      switch (e.key.toLowerCase()) {
        case 'n':
          e.preventDefault();
          handleNewProject();
          break;
        case 'o':
          e.preventDefault();
          handleOpenProject();
          break;
        case 's':
          e.preventDefault();
          if (e.shiftKey) {
            handleSaveAsProject();
          } else {
            handleSaveProject();
          }
          break;
        case 'w':
          e.preventDefault();
          if (docState.activeDocumentId) {
            handleCloseTab(docState.activeDocumentId);
          }
          break;
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [handleNewProject, handleOpenProject, handleSaveProject, handleSaveAsProject, handleCloseTab, docState.activeDocumentId]);

  return (
    <div className="app">
      <TitleBar
        onNewProject={handleNewProject}
        onOpenProject={handleOpenProject}
        onSaveProject={handleSaveProject}
      />
      {hasActiveDoc ? (
        <ActiveDocumentContent
          onNewProject={handleNewProject}
          onOpenProject={handleOpenProject}
          onSaveProject={handleSaveProject}
          onSaveAsProject={handleSaveAsProject}
          onCloseTab={handleCloseTab}
        />
      ) : (
        <>
          <WelcomeScreen
            onNewProject={handleNewProject}
            onOpenProject={handleOpenProject}
            onOpenExample={handleOpenExample}
          />
          <EmptyStatusBar />
        </>
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
