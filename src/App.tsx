import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { EnergyProvider, useEnergy, useDocumentManager, useHasActiveDocument } from './context/EnergyContext';
import { isSurveyProject } from './core/nta/stepStatus';
import { KernelProvider, useKernel } from './context/KernelProvider';
import { projectCalculated } from './core/nta/KernelClient';
import { summarizeForPreview } from './core/nta/PreviewSummary';
import { NtaDraftProvider, useNtaDraft } from './context/NtaDraftProvider';
import { I18nProvider } from './i18n/I18nProvider';
import { WelcomeScreen, type NewProjectKind } from './components/WelcomeScreen/WelcomeScreen';
import { surveyTemplate } from './core/nta/SurveyTemplates';
import { surveySteps } from './core/survey/surveyFlow';
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
import { LazyPage, ManualView, PrintPreviewDialog } from './components/shell/lazyPages';
import { FeedbackDialog } from './components/dialogs/FeedbackDialog/FeedbackDialog';
import { SettingsDialog } from './components/SettingsDialog/SettingsDialog';
import { TopBar, type TopBarProps } from './components/shell/TopBar';
import { WorkflowNav } from './components/shell/WorkflowNav';
import { Inspector } from './components/shell/Inspector';
import { StepRouter } from './components/shell/StepRouter';
import { CommandPalette, buildPaletteEntries } from './components/shell/CommandPalette';
import { ShellActionsProvider, type ShellActions } from './components/shell/ShellActions';
import { calculateBENGMonthly } from './core/energy/BENGCalculatorMonthly';
import { hasUnmodelledHeatPumpDetails, legacyHeatPumpInputIssue, validProjectFloorArea } from './core/energy/ProjectArea';
import { useI18n } from './i18n/i18n';
import { downloadReportHTML } from './core/report/ReportGenerator';
import { bengIfcModel, downloadBENGIFC } from './core/ifc/IFCEnergyExporter';
import { calculateProjectPerformanceShared } from './core/nta/useProjectPerformance';
import { downloadModelIFC } from './core/ifc/IFCModelExporter';
import { downloadUNIEC3, openUNIEC3FileDialog } from './core/io/UNIEC3Exporter';
import { downloadVABI, openVABIFileDialog } from './core/io/VABIElementsBridge';
import { withImportRecord } from './core/io/importLog';
import { serializeProject, deserializeProjectFile, compareKernelStamp, describeStamp } from './core/io/ProjectSerializer';
import { migrateLegacyRelabel, relabelNoticeKey } from './core/nta/Registration';
import { normalizeProject } from './core/energy/normalizeProject';
import { stepStatuses } from './core/nta/stepStatus';
import { adjacentStep, routeFromHash, routeToHash, stepDefinition, type Route } from './core/navigation/routes';
import { selectionForPath } from './core/navigation/projectPaths';
import { isTauri } from '@tauri-apps/api/core';
import { ConfirmProvider, Dialog, ToastProvider, useConfirm, useToast } from './components/ui';
import { stampProject } from './core/io/KernelStampClient';
import { forgetRecentProject, readRecentProjects, recordRecentProject } from './core/io/recentProjects';
import { browserStore, readLibraryProject, removeFromLibrary, updateLibraryEntry } from './core/io/projectLibrary';
import { LibraryOutcomeRecorder } from './components/WelcomeScreen/LibraryOutcomeRecorder';
import { EXAMPLE_KINDS, exampleProject, type ExampleKind } from './core/nta/ExampleProjects';
import type { DialogType, IProject } from './core/energy/types';
import './components/shell/shell.css';
import { NewVersionBanner } from './components/shell/NewVersionBanner';
import { applySetup, firstOpenSurveyStep, type NewProjectSetup } from './core/io/newProjectSetup';

/** Whether the relabel migration notice for this key was shown already (per browser profile). */
function relabelNoticeShown(key: string): boolean {
  try {
    if (localStorage.getItem(key)) return true;
    localStorage.setItem(key, '1');
  } catch {
    // Storage unavailable: show the notice each time.
  }
  return false;
}

const INSPECTOR_KEY = 'oes.inspector.open';

function initialInspectorOpen(): boolean {
  try {
    const stored = localStorage.getItem(INSPECTOR_KEY);
    if (stored != null) return stored === '1';
  } catch { /* storage unavailable */ }
  return typeof window === 'undefined' || window.innerWidth >= 1360;
}

/** Keys typed into these never trigger step navigation or recalculation. */
function isTextEntry(target: EventTarget | null): boolean {
  return target instanceof HTMLElement && (target.tagName === 'TEXTAREA' || target.tagName === 'SELECT' || target.isContentEditable);
}

/** Commands that do not need an open document; shared by the welcome screen and the shell. */
interface FileCommands {
  onNewProject: () => void;
  /** A new project of a kind: existing dwelling or utility building (basisopname), or new build. */
  onNewProjectOf: (kind: NewProjectKind) => void;
  /** Shows the project library while documents stay open. */
  onShowLibrary: () => void;
  onOpenProject: () => void;
  onSaveProject: () => void;
  onSaveAsProject: () => void;
  onImportUNIEC3: () => void;
  onImportVABI: () => void;
  onCloseTab: (id: string) => void;
  onCloseActiveTab: () => void;
  onOpenSettings: () => void;
  onOpenFeedback: () => void;
}

// ── Active document: the shell with navigation, work area, inspector and status bar ──

function ActiveDocumentShell({ files, paletteOpen, setPaletteOpen }: {
  files: FileCommands;
  paletteOpen: boolean;
  setPaletteOpen: (open: boolean) => void;
}) {
  const { state, dispatch } = useEnergy();
  const kernel = useKernel();
  const { t, locale } = useI18n();
  const { dialog, project, result, route } = state;
  const [printPreviewOpen, setPrintPreviewOpen] = useState(false);
  const [calculationError, setCalculationError] = useState<string | null>(null);
  const [inspectorOpen, setInspectorOpen] = useState(initialInspectorOpen);

  useEffect(() => { setCalculationError(null); }, [project]);

  const toggleInspector = useCallback(() => {
    setInspectorOpen((open) => {
      try { localStorage.setItem(INSPECTOR_KEY, open ? '0' : '1'); } catch { /* storage unavailable */ }
      return !open;
    });
  }, []);

  const openDialog = useCallback((type: DialogType) => {
    dispatch({ type: 'OPEN_DIALOG', payload: { type } });
  }, [dispatch]);

  const closeDialog = useCallback(() => {
    dispatch({ type: 'CLOSE_DIALOG' });
  }, [dispatch]);

  const navigate = useCallback((target: Route) => {
    dispatch({ type: 'NAVIGATE', payload: target });
    // "Ga naar" on an item path also selects the item, so the inspector shows it.
    const selection = target.focusPath ? selectionForPath(project, target.focusPath) : null;
    if (selection) dispatch({ type: 'SELECT_ITEM', payload: selection });
  }, [dispatch, project]);

  const handleCalculate = useCallback(() => {
    kernel?.refresh();
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
      dispatch({ type: 'NAVIGATE', payload: { step: 'results' } });
      setCalculationError(null);
    } catch (error) {
      dispatch({ type: 'SET_RESULT', payload: null });
      setCalculationError(error instanceof Error ? error.message : String(error));
    }
  }, [project, dispatch, t, kernel]);

  const handleExportReport = useCallback(() => {
    downloadReportHTML(project, result, locale).catch((error: unknown) =>
      setCalculationError(error instanceof Error ? error.message : String(error)));
  }, [project, result, locale]);

  // The BENG IFC export follows the shared rule (KernelVerdict): kernel figures when calculated,
  // nothing when the kernel refused the input, the indicative result only without a verdict.
  const handleExportIFC = useCallback(() => {
    calculateProjectPerformanceShared(project)
      .then((assessment) => assessment, () => null)
      .then((assessment) => {
        const model = bengIfcModel(project, result, assessment);
        if (model === 'withheld') {
          setCalculationError(t(assessment?.status === 'incomplete' ? 'results.withheld.incomplete' : 'results.withheld.invalid'));
        } else if (model) {
          downloadBENGIFC(project, model);
        }
      });
  }, [project, result, t]);

  const actions = useMemo<ShellActions>(() => ({
    newProject: files.onNewProject,
    openProject: files.onOpenProject,
    saveProject: files.onSaveProject,
    saveAsProject: files.onSaveAsProject,
    calculate: handleCalculate,
    openDialog,
    navigate,
    exportReport: handleExportReport,
    printReport: () => setPrintPreviewOpen(true),
    exportIFC: handleExportIFC,
    exportModelIFC: () => downloadModelIFC(project),
    exportUNIEC3: () => downloadUNIEC3(project),
    importUNIEC3: files.onImportUNIEC3,
    exportVABI: () => downloadVABI(project),
    importVABI: files.onImportVABI,
    openSettings: files.onOpenSettings,
    openFeedback: files.onOpenFeedback,
    openPalette: () => setPaletteOpen(true),
    toggleInspector,
    togglePreview: () => {
      if (!inspectorOpen) toggleInspector();
      dispatch({ type: 'TOGGLE_PREVIEW' });
    },
  }), [files, handleCalculate, openDialog, navigate, handleExportReport, handleExportIFC, project, setPaletteOpen,
    toggleInspector, inspectorOpen, dispatch]);

  const statuses = useMemo(() => stepStatuses(project, kernel?.settled), [project, kernel?.settled]);

  // The URL hash mirrors the route (`#/installaties`); an opened link or edited hash navigates.
  const hashApplied = useRef(false);
  useEffect(() => {
    if (!hashApplied.current) {
      hashApplied.current = true;
      const fromHash = routeFromHash(window.location.hash);
      if (fromHash) { dispatch({ type: 'NAVIGATE', payload: fromHash }); return; }
    }
    const hash = routeToHash(route);
    if (window.location.hash !== hash) {
      window.history.replaceState(window.history.state, '', `${window.location.pathname}${window.location.search}${hash}`);
    }
  }, [route, dispatch]);
  useEffect(() => {
    const onHashChange = () => {
      const fromHash = routeFromHash(window.location.hash);
      if (fromHash) dispatch({ type: 'NAVIGATE', payload: fromHash });
    };
    window.addEventListener('hashchange', onHashChange);
    return () => window.removeEventListener('hashchange', onHashChange);
  }, [dispatch]);

  useEffect(() => {
    document.title = `${state.isDirty ? '• ' : ''}${project.name || t('app.untitledProject')} — ${t('app.title')}`;
  }, [project.name, state.isDirty, t]);

  // Shell shortcuts: Ctrl ↵ recalculates, Ctrl . toggles the inspector, Alt ↑/↓ steps, Alt ←/→ sub pages.
  useEffect(() => {
    const handler = (event: KeyboardEvent) => {
      if (document.querySelector('.dialog-overlay, .palette-backdrop')) return;
      const ctrl = event.ctrlKey || event.metaKey;
      if (ctrl && event.key === 'Enter' && !isTextEntry(event.target)) {
        event.preventDefault();
        handleCalculate();
      } else if (ctrl && event.key === '.') {
        event.preventDefault();
        toggleInspector();
      } else if (event.altKey && !ctrl && (event.key === 'ArrowUp' || event.key === 'ArrowDown') && !isTextEntry(event.target)) {
        event.preventDefault();
        navigate({ step: adjacentStep(route.step, event.key === 'ArrowDown' ? 1 : -1) });
      } else if (event.altKey && !ctrl && (event.key === 'ArrowLeft' || event.key === 'ArrowRight') && !isTextEntry(event.target)) {
        const subs = stepDefinition(route.step).subs;
        const index = subs.findIndex((sub) => sub.id === route.sub);
        if (subs.length < 2 || index < 0) return;
        event.preventDefault();
        const next = subs[(index + (event.key === 'ArrowRight' ? 1 : -1) + subs.length) % subs.length];
        navigate({ step: route.step, sub: next.id });
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [handleCalculate, toggleInspector, navigate, route]);

  const paletteEntries = useMemo(() => (paletteOpen
    ? buildPaletteEntries(t, project, actions, (item) => {
      actions.navigate(item.route);
      dispatch({ type: 'SELECT_ITEM', payload: { id: item.id, itemType: item.itemType } });
    })
    : []), [paletteOpen, t, project, actions, dispatch]);

  const showInspector = inspectorOpen && route.step !== 'project' && !isSurveyProject(project);
  const floorArea = kernel?.settled?.geometry?.usableFloorAreaM2
    ?? (project.zones.length ? project.zones.reduce((sum, zone) => sum + zone.floorArea, 0) : null);

  return (
    <ShellActionsProvider value={actions}>
      <TopBar hasDocument {...files} onOpenPalette={actions.openPalette} onRecalculate={handleCalculate}
        onToggleInspector={toggleInspector} inspectorOpen={showInspector} />
      <WorkflowNav project={project} route={route} statuses={statuses} floorAreaM2={floorArea} actions={actions} />
      <main id="main-content" className="shell-main" tabIndex={-1} aria-labelledby="page-title">
        {calculationError && <div className="calculation-input-error" role="alert">{calculationError}</div>}
        <StepRouter project={project} route={route} statuses={statuses} actions={actions} />
      </main>
      {showInspector && <Inspector onClose={toggleInspector} />}
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
        <LazyPage><PrintPreviewDialog onClose={() => setPrintPreviewOpen(false)} /></LazyPage>
      )}
      {paletteOpen && <CommandPalette entries={paletteEntries} onClose={() => setPaletteOpen(false)} />}
    </ShellActionsProvider>
  );
}

/** What the app level needs to know of the open document (its draft and kernel live below it). */
export interface DocumentReport {
  projectId: string;
  /** The NTA draft has changes that are not applied. */
  draftDirty: boolean;
  /** Label class of a calculated kernel answer, for the recent-projects list. */
  labelClass: string | null;
}

function DocumentReporter({ onReport }: { onReport: (report: DocumentReport | null) => void }) {
  const { state } = useEnergy();
  const draftDirty = useNtaDraft()?.dirty ?? false;
  const settled = useKernel()?.settled ?? null;
  const summary = settled ? summarizeForPreview(settled) : null;
  const labelClass = summary && projectCalculated(summary.status) ? summary.labelClass ?? null : null;
  const projectId = state.project.id;
  useEffect(() => { onReport({ projectId, draftDirty, labelClass }); }, [projectId, draftDirty, labelClass, onReport]);
  useEffect(() => () => onReport(null), [onReport]);
  return null;
}

/** One kernel run per open document, shared by every view of the shell. */
function ActiveDocumentContent({ onReport, ...props }: {
  files: FileCommands; paletteOpen: boolean; setPaletteOpen: (open: boolean) => void; onReport: (report: DocumentReport | null) => void;
}) {
  const { state } = useEnergy();
  return (
    <KernelProvider project={state.project}>
      <NtaDraftProvider>
        <DocumentReporter onReport={onReport} />
        <LibraryOutcomeRecorder />
        <ActiveDocumentShell {...props} />
      </NtaDraftProvider>
    </KernelProvider>
  );
}

// ── Minimal status bar when no document is open ──

/** The manual without an open document (welcome screen): the same viewer in a dialog. */
function ManualDialog({ onClose }: { onClose: () => void }) {
  const { t } = useI18n();
  const [chapter, setChapter] = useState<string | undefined>();
  return (
    <Dialog title={t('manual.title')} onClose={onClose} width={1100} className="manual-dialog">
      <LazyPage><ManualView chapterRef={chapter} onOpen={setChapter} /></LazyPage>
    </Dialog>
  );
}

function EmptyStatusBar() {
  const { t } = useI18n();
  return (
    <footer className="status-bar">
      <div className="status-section" role="status">
        <span className="status-hint">{t('status.ready')}</span>
      </div>
    </footer>
  );
}

// ── Main app shell ──

function AppContent() {
  const { docState, docDispatch } = useDocumentManager();
  const hasActiveDoc = useHasActiveDocument();
  // The project library over the open documents (feedback 8 Oct 2026: "hoe moet ik de bibliotheek vinden").
  const [libraryOpen, setLibraryOpen] = useState(false);
  const activeDocumentId = docState.activeDocumentId;
  useEffect(() => { setLibraryOpen(false); }, [activeDocumentId]);
  const { t } = useI18n();
  const toast = useToast();
  const confirmChoice = useConfirm();
  const untitledCounter = useRef(0);
  // Draft state and label of the active document (both live below this component).
  const activeReport = useRef<DocumentReport | null>(null);
  const setActiveReport = useCallback((report: DocumentReport | null) => { activeReport.current = report; }, []);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [feedbackOpen, setFeedbackOpen] = useState(false);
  const [paletteOpen, setPaletteOpen] = useState(false);
  const [manualDialogOpen, setManualDialogOpen] = useState(false);

  const createEmptyProject = useCallback((buildingFunction: IProject['buildingFunction'] = 'residential'): IProject => {
    untitledCounter.current += 1;
    return {
      id: crypto.randomUUID(),
      name: `Untitled ${untitledCounter.current}`,
      description: '',
      buildingFunction,
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

  // Welcome screen: a new dwelling or utility building (office; the function can be changed in Projectgegevens).
  const handleNewProjectOf = useCallback((kind: NewProjectKind) => {
    const utility = kind === 'utility' || kind === 'existing_utility';
    const project = createEmptyProject(utility ? 'office' : 'residential');
    const id = crypto.randomUUID();
    if (kind === 'existing_residential' || kind === 'existing_utility') {
      // An energy label for an existing building starts the basisopname question flow.
      const surveyKind = utility ? 'utility' : 'residential';
      docDispatch({ type: 'DOC_NEW', payload: { id, project: { ...project, basisopname: { ...surveyTemplate(surveyKind), progress: {} } } } });
      docDispatch({ type: 'DOC_DISPATCH', payload: { id, action: { type: 'NAVIGATE', payload: { step: 'survey', sub: surveySteps(surveyKind)[0].id } } } });
      return;
    }
    docDispatch({ type: 'DOC_NEW', payload: { id, project } });
  }, [docDispatch, createEmptyProject]);

  // The new-project window: kind, BAG address, dwelling type and what the building has (feedback 9 Oct 2026).
  const handleCreateProject = useCallback((setup: NewProjectSetup) => {
    const utility = setup.kind === 'utility' || setup.kind === 'existing_utility';
    const project = applySetup(createEmptyProject(utility ? 'office' : 'residential'), setup);
    const id = crypto.randomUUID();
    docDispatch({ type: 'DOC_NEW', payload: { id, project } });
    const step = firstOpenSurveyStep(project);
    if (step) docDispatch({ type: 'DOC_DISPATCH', payload: { id, action: { type: 'NAVIGATE', payload: { step: 'survey', sub: step } } } });
  }, [docDispatch, createEmptyProject]);

  // Recently opened or saved project files (desktop only: browser documents have no path).
  const [recentProjects, setRecentProjects] = useState(readRecentProjects);
  const rememberRecent = useCallback((filePath: string, project: IProject) => {
    const report = activeReport.current?.projectId === project.id ? activeReport.current : null;
    setRecentProjects(recordRecentProject({
      path: filePath, name: project.name, buildingFunction: project.buildingFunction, labelClass: report?.labelClass ?? undefined,
    }));
  }, []);

  // Project library (start screen): open, duplicate, move, archive and delete a card.
  const openLibraryEntry = useCallback((id: string) => {
    const open = docState.documents.find((doc) => doc.state.project.id === id);
    // The open project may be the one clicked: its id does not change, so close the library here.
    setLibraryOpen(false);
    if (open) { docDispatch({ type: 'DOC_SET_ACTIVE', payload: open.id }); return; }
    const store = browserStore();
    const snapshot = store ? readLibraryProject(store, id) : null;
    if (!snapshot) { toast.show({ tone: 'error', title: t('library.menu.open'), message: t('library.noMatch') }); return; }
    try {
      docDispatch({ type: 'DOC_OPEN', payload: { id, project: normalizeProject(snapshot), filePath: null } });
    } catch (err) {
      toast.show({ tone: 'error', title: t('library.menu.open'), message: (err as Error).message });
    }
  }, [docState.documents, docDispatch, toast, t]);
  const duplicateLibraryEntry = useCallback((id: string) => {
    const store = browserStore();
    const open = docState.documents.find((doc) => doc.state.project.id === id)?.state.project;
    const source = open ?? (store ? readLibraryProject(store, id) : null);
    if (!source) return;
    const copy: IProject = { ...structuredClone(source), id: crypto.randomUUID(), name: `${source.name} (kopie)` };
    delete copy.importLog;
    docDispatch({ type: 'DOC_NEW', payload: { id: copy.id, project: copy } });
  }, [docState.documents, docDispatch]);
  const deleteLibraryEntry = useCallback((id: string) => {
    const store = browserStore();
    if (!store) return;
    const name = docState.documents.find((doc) => doc.state.project.id === id)?.state.project.name ?? readLibraryProject(store, id)?.name ?? '';
    if (!window.confirm(t('library.deleteConfirm', { name }))) return;
    const open = docState.documents.find((doc) => doc.state.project.id === id);
    if (open) docDispatch({ type: 'DOC_CLOSE', payload: open.id });
    removeFromLibrary(store, id);
  }, [docState.documents, docDispatch, t]);
  // Trash (feedback 9 Oct 2026): a trashed project that is open closes; the library keeps it until the trash is emptied.
  const trashLibraryEntry = useCallback((id: string, trashed: boolean) => {
    const store = browserStore();
    if (!store) return;
    const open = docState.documents.find((doc) => doc.state.project.id === id);
    if (trashed && open) docDispatch({ type: 'DOC_CLOSE', payload: open.id });
    updateLibraryEntry(store, id, { trashed: trashed ? new Date().toISOString() : undefined });
  }, [docState.documents, docDispatch]);
  const moveLibraryEntry = useCallback((id: string, folder: string | null) => {
    const store = browserStore();
    if (store) updateLibraryEntry(store, id, { folder: folder ?? undefined });
  }, []);
  const archiveLibraryEntry = useCallback((id: string, archived: boolean) => {
    const store = browserStore();
    if (store) updateLibraryEntry(store, id, { archived });
  }, []);

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
  // Shared by the desktop dialog and the browser file input. `filePath` is the
  // saved location (desktop) or null (browser: a later save downloads a copy).
  const openProjectText = useCallback(async (json: string, filePath: string | null, label: string) => {
    try {
      const { project: rawProject, kernel: saved } = deserializeProjectFile(json);
      const opened = normalizeProject(rawProject);
      const { project: loaded, missing, markedForReview } = migrateLegacyRelabel(opened);
      docDispatch({ type: 'DOC_OPEN', payload: { id: crypto.randomUUID(), project: loaded, filePath } });
      if (filePath) rememberRecent(filePath, loaded);
      // Marked invoices are always reported; a notice with only open
      // fields is shown once per project.
      if (markedForReview > 0
        || (missing.length > 0 && !relabelNoticeShown(await relabelNoticeKey(loaded.id, label, json)))) {
        toast.show({
          tone: 'warn',
          title: t('app.toast.relabelMigrated'),
          message: t('relabel.migrationNotice', {
            fields: missing.map((field) => t(`relabel.migrationField.${field}`)).join(', '),
            count: String(markedForReview),
          }),
        });
      }
      const current = await stampProject(loaded);
      const [difference] = compareKernelStamp(saved, current);
      if (difference === 'version' && saved && current) {
        toast.show({
          tone: 'warn',
          title: t('app.toast.kernelChanged'),
          message: t('project.kernelChanged', { saved: describeStamp(saved), current: describeStamp(current) }),
        });
      } else if (difference === 'input') {
        toast.show({ tone: 'warn', title: t('app.toast.inputChanged'), message: t('project.inputChanged') });
      }
    } catch (err) {
      toast.show({
        tone: 'error',
        title: t('app.toast.openFailed'),
        message: t('project.openFailed', { message: (err as Error).message ?? String(err) }),
      });
    }
  }, [docDispatch, t, toast, rememberRecent]);

  const handleOpenRecent = useCallback(async (filePath: string) => {
    try {
      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      await openProjectText(await readTextFile(filePath), filePath, filePath);
    } catch (err) {
      setRecentProjects(forgetRecentProject(filePath));
      toast.show({
        tone: 'error',
        title: t('app.toast.openFailed'),
        message: t('project.openFailed', { message: (err as Error).message ?? String(err) }),
      });
    }
  }, [openProjectText, t, toast]);

  const handleOpenProject = useCallback(async () => {
    if (!isTauri()) {
      // Browser build: no native dialog, so pick the file with an input element.
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.oes.json,.json,application/json';
      input.onchange = () => {
        const file = input.files?.[0];
        if (file) void file.text().then((json) => openProjectText(json, null, file.name));
      };
      input.click();
      return;
    }
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const { readTextFile } = await import('@tauri-apps/plugin-fs');
      const filePath = await open({
        filters: [{ name: 'OES Project', extensions: ['oes.json', 'json'] }],
        multiple: false,
      });
      if (!filePath) return;
      const json = await readTextFile(filePath as string);
      await openProjectText(json, filePath as string, filePath as string);
    } catch (err) {
      const msg = (err as Error).message;
      if (msg && !msg.includes('cancelled')) {
        toast.show({ tone: 'error', title: t('app.toast.openFailed'), message: t('project.openFailed', { message: msg }) });
      }
    }
  }, [openProjectText, t, toast]);

  // ── Import (UNIEC3 / VABI): each opens as a new document ──
  const handleImportUNIEC3 = useCallback(async () => {
    try {
      const loaded = await openUNIEC3FileDialog();
      docDispatch({ type: 'DOC_NEW', payload: { id: crypto.randomUUID(), project: loaded } });
    } catch (err) {
      toast.show({ tone: 'error', title: t('app.toast.importFailed', { format: 'UNIEC3' }), message: (err as Error).message });
    }
  }, [docDispatch, toast, t]);

  const handleImportVABI = useCallback(async () => {
    try {
      const loaded = withImportRecord(await openVABIFileDialog(), 'VABI');
      docDispatch({ type: 'DOC_NEW', payload: { id: crypto.randomUUID(), project: loaded } });
    } catch (err) {
      toast.show({ tone: 'error', title: t('app.toast.importFailed', { format: 'VABI' }), message: (err as Error).message });
    }
  }, [docDispatch, toast, t]);

  // ── Save (Ctrl+S) — save to existing path, or prompt Save As if new ──
  const handleSaveProject = useCallback(async () => {
    const activeDoc = docState.documents.find(d => d.id === docState.activeDocumentId);
    if (!activeDoc) return;

    const savedPath = await writeProjectToDisk(activeDoc.state.project, activeDoc.filePath, false);
    if (savedPath === null) return; // Cancelled

    if (savedPath !== 'browser-download' && savedPath !== activeDoc.filePath) {
      docDispatch({ type: 'DOC_SET_FILE_PATH', payload: { id: activeDoc.id, filePath: savedPath } });
    }
    if (savedPath !== 'browser-download') rememberRecent(savedPath, activeDoc.state.project);
    docDispatch({ type: 'DOC_DISPATCH', payload: { id: activeDoc.id, action: { type: 'SET_DIRTY', payload: false } } });
  }, [docState, docDispatch, writeProjectToDisk, rememberRecent]);

  // ── Save As (always prompts for new path) ──
  const handleSaveAsProject = useCallback(async () => {
    const activeDoc = docState.documents.find(d => d.id === docState.activeDocumentId);
    if (!activeDoc) return;

    const savedPath = await writeProjectToDisk(activeDoc.state.project, activeDoc.filePath, true);
    if (savedPath === null) return; // Cancelled

    if (savedPath !== 'browser-download') {
      docDispatch({ type: 'DOC_SET_FILE_PATH', payload: { id: activeDoc.id, filePath: savedPath } });
      rememberRecent(savedPath, activeDoc.state.project);
    }
    docDispatch({ type: 'DOC_DISPATCH', payload: { id: activeDoc.id, action: { type: 'SET_DIRTY', payload: false } } });
  }, [docState, docDispatch, writeProjectToDisk, rememberRecent]);

  // ── Close tab (with unsaved-changes check) ──
  const handleCloseTab = useCallback(async (id: string) => {
    const doc = docState.documents.find(d => d.id === id);
    if (!doc) return;

    // NTA input that was edited but not applied is lost on closing (it is not part of the project yet).
    if (id === docState.activeDocumentId && activeReport.current?.projectId === doc.state.project.id && activeReport.current.draftDirty) {
      const choice = await confirmChoice({
        title: t('app.unappliedDraft.title'),
        message: t('app.unappliedDraft.message', { name: doc.state.project.name || t('app.untitled') }),
        confirmLabel: t('app.unappliedDraft.discard'),
        danger: true,
      });
      if (choice !== 'confirm') return;
    }

    if (doc.state.isDirty) {
      // One translated dialog for desktop and browser: Opslaan · Niet opslaan · Annuleren.
      const choice = await confirmChoice({
        title: t('app.unsaved.title'),
        message: t('app.unsaved.message', { name: doc.state.project.name || t('app.untitled') }),
        confirmLabel: t('app.unsaved.save'),
        denyLabel: t('app.unsaved.discard'),
      });
      if (choice === 'cancel') return;
      if (choice === 'confirm') {
        const savedPath = await writeProjectToDisk(doc.state.project, doc.filePath, false);
        if (savedPath === null) return; // User cancelled Save As — don't close
      }
    }

    docDispatch({ type: 'DOC_CLOSE', payload: id });
  }, [docState.documents, docState.activeDocumentId, docDispatch, writeProjectToDisk, confirmChoice, t]);

  const handleCloseActiveTab = useCallback(() => {
    if (docState.activeDocumentId) void handleCloseTab(docState.activeDocumentId);
  }, [docState.activeDocumentId, handleCloseTab]);

  // ── Keyboard shortcuts (all screens) ──
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
          handleCloseActiveTab();
          break;
        case 'k':
          e.preventDefault();
          setPaletteOpen(true);
          break;
        case ',':
          e.preventDefault();
          setSettingsOpen(true);
          break;
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [handleNewProject, handleOpenProject, handleSaveProject, handleSaveAsProject, handleCloseActiveTab]);

  const files = useMemo<FileCommands>(() => ({
    onNewProject: handleNewProject,
    onNewProjectOf: handleNewProjectOf,
    onShowLibrary: () => setLibraryOpen(true),
    onOpenProject: handleOpenProject,
    onSaveProject: handleSaveProject,
    onSaveAsProject: handleSaveAsProject,
    onImportUNIEC3: handleImportUNIEC3,
    onImportVABI: handleImportVABI,
    onCloseTab: handleCloseTab,
    onCloseActiveTab: handleCloseActiveTab,
    onOpenSettings: () => setSettingsOpen(true),
    onOpenFeedback: () => setFeedbackOpen(true),
  }), [handleNewProject, handleNewProjectOf, handleOpenProject, handleSaveProject, handleSaveAsProject, handleImportUNIEC3, handleImportVABI,
    handleCloseTab, handleCloseActiveTab]);

  // The live preview of the inspector is a per-document setting, offered in Instellingen.
  const activeDoc = docState.documents.find((doc) => doc.id === docState.activeDocumentId);
  const previewSetting = activeDoc ? {
    enabled: activeDoc.state.previewVisible,
    onChange: (enabled: boolean) => {
      if (enabled !== activeDoc.state.previewVisible) {
        docDispatch({ type: 'DOC_DISPATCH', payload: { id: activeDoc.id, action: { type: 'TOGGLE_PREVIEW' } } });
      }
    },
  } : undefined;

  const welcomeTopBar: TopBarProps = { hasDocument: false, ...files, onOpenPalette: () => setPaletteOpen(true) };
  const welcomePalette = useMemo(() => {
    if (hasActiveDoc || !paletteOpen) return [];
    const none = () => undefined;
    const actions: ShellActions = {
      newProject: handleNewProject, openProject: handleOpenProject, saveProject: none, saveAsProject: none, calculate: none,
      openDialog: none, navigate: none, exportReport: none, printReport: none, exportIFC: none, exportModelIFC: none,
      exportUNIEC3: none, importUNIEC3: handleImportUNIEC3, exportVABI: none, importVABI: handleImportVABI,
      openSettings: () => setSettingsOpen(true), openFeedback: () => setFeedbackOpen(true), openPalette: none,
      toggleInspector: none, togglePreview: none,
    };
    const allowed = new Set(['cmd:new', 'cmd:open', 'cmd:import-uniec3', 'cmd:import-vabi', 'cmd:settings', 'cmd:feedback']);
    return buildPaletteEntries(t, null, actions, none).filter((entry) => allowed.has(entry.id));
  }, [hasActiveDoc, paletteOpen, t, handleNewProject, handleOpenProject, handleImportUNIEC3, handleImportVABI]);

  useEffect(() => {
    if (!hasActiveDoc) document.title = t('app.title');
  }, [hasActiveDoc, t]);

  return (
    <div className={hasActiveDoc ? 'app-shell' : 'app-shell app-shell--empty'}>
      <a className="skip-link" href="#main-content"
        onClick={(event) => { event.preventDefault(); document.getElementById('main-content')?.focus(); }}>{t('nav.skip')}</a>
      <NewVersionBanner />
      {hasActiveDoc && !libraryOpen ? (
        <ActiveDocumentContent files={files} paletteOpen={paletteOpen} setPaletteOpen={setPaletteOpen} onReport={setActiveReport} />
      ) : (
        <>
          <TopBar {...welcomeTopBar} hasDocument={hasActiveDoc} />
          <main id="main-content" className="shell-main" tabIndex={-1}>
            <WelcomeScreen
              onNewProject={handleNewProjectOf}
              onCreateProject={handleCreateProject}
              onTrashEntry={trashLibraryEntry}
              onOpenProject={handleOpenProject}
              onOpenExample={handleOpenExample}
              onImportUNIEC3={handleImportUNIEC3}
              onImportVABI={handleImportVABI}
              recent={recentProjects}
              onOpenRecent={handleOpenRecent}
              onForgetRecent={(path) => setRecentProjects(forgetRecentProject(path))}
              onBack={hasActiveDoc ? () => setLibraryOpen(false) : undefined}
              onOpenEntry={openLibraryEntry}
              onDuplicateEntry={duplicateLibraryEntry}
              onDeleteEntry={deleteLibraryEntry}
              onMoveEntry={moveLibraryEntry}
              onArchiveEntry={archiveLibraryEntry}
            />
          </main>
          <EmptyStatusBar />
          {paletteOpen && <CommandPalette entries={welcomePalette} onClose={() => setPaletteOpen(false)} />}
        </>
      )}
      {settingsOpen && <SettingsDialog onClose={() => setSettingsOpen(false)} previewSetting={previewSetting}
        onOpenManual={() => {
          setSettingsOpen(false);
          // In a document the manual is a page (Gereedschap › Handleiding); on the welcome screen a dialog.
          if (activeDoc) {
            docDispatch({ type: 'DOC_DISPATCH', payload: { id: activeDoc.id, action: { type: 'NAVIGATE', payload: { step: 'tool', sub: 'manual' } } } });
          } else {
            setManualDialogOpen(true);
          }
        }} />}
      {manualDialogOpen && <ManualDialog onClose={() => setManualDialogOpen(false)} />}
      {feedbackOpen && <FeedbackDialog onClose={() => setFeedbackOpen(false)} />}
    </div>
  );
}

export default function App() {
  return (
    <I18nProvider>
      <ToastProvider>
        <ConfirmProvider>
          <EnergyProvider>
            <AppContent />
          </EnergyProvider>
        </ConfirmProvider>
      </ToastProvider>
    </I18nProvider>
  );
}
