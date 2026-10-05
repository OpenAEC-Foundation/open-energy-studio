/**
 * Every command of the former ribbon, title bar and file menu, in one list.
 * The command palette shows them; the step pages and menus call the same
 * `ShellActions`. `ribbonSource` documents where the command used to live, so
 * the mapping from ribbon to shell can be tested (workflow-nav.test).
 */
import type { ShellActions } from './ShellActions';
import type { DialogType } from '../../core/energy/types';

export type CommandGroup = 'file' | 'project' | 'building' | 'installations' | 'calculation' | 'report' | 'exchange' | 'tools' | 'app';

export interface ShellCommand {
  id: string;
  labelKey: string;
  group: CommandGroup;
  /** Where the command was in the ribbon era, e.g. "Gebouwschil › Zone toevoegen". */
  ribbonSource: string;
  /** Where it lives now (besides the palette). */
  shellPlace: string;
  shortcut?: string;
  run: (actions: ShellActions) => void;
}

const dialog = (type: DialogType) => (actions: ShellActions) => actions.openDialog(type);

export const SHELL_COMMANDS: ShellCommand[] = [
  // File (title bar quick access and the File tab / app menu)
  { id: 'new', labelKey: 'ribbon.new', group: 'file', ribbonSource: 'Bestand/Start › Nieuw', shellPlace: 'Bovenbalk › Bestand', shortcut: 'Ctrl N', run: (a) => a.newProject() },
  { id: 'open', labelKey: 'ribbon.open', group: 'file', ribbonSource: 'Bestand/Start › Openen', shellPlace: 'Bovenbalk › Bestand', shortcut: 'Ctrl O', run: (a) => a.openProject() },
  { id: 'save', labelKey: 'ribbon.save', group: 'file', ribbonSource: 'Bestand/Start › Opslaan', shellPlace: 'Bovenbalk › Opslaan', shortcut: 'Ctrl S', run: (a) => a.saveProject() },
  { id: 'save-as', labelKey: 'ribbon.saveAs', group: 'file', ribbonSource: 'Bestand › Opslaan als', shellPlace: 'Bovenbalk › Bestand', shortcut: 'Ctrl Shift S', run: (a) => a.saveAsProject() },
  // Start tab
  { id: 'project-info', labelKey: 'ribbon.projectInfo', group: 'project', ribbonSource: 'Start › Projectgegevens', shellPlace: 'Project › Projectgegevens', run: dialog('project-info') },
  { id: 'calculate', labelKey: 'shell.recalculate', group: 'calculation', ribbonSource: 'Start/Resultaten › Berekenen', shellPlace: 'Bovenbalk › Herberekenen', shortcut: 'Ctrl ↵', run: (a) => a.calculate() },
  { id: 'toggle-preview', labelKey: 'ribbon.preview', group: 'app', ribbonSource: 'Start › Preview', shellPlace: 'Contextpaneel › Voorbeeld · Instellingen', run: (a) => a.togglePreview() },
  { id: 'toggle-inspector', labelKey: 'shell.inspector', group: 'app', ribbonSource: '— (nieuw)', shellPlace: 'Bovenbalk › Contextpaneel', shortcut: 'Ctrl .', run: (a) => a.toggleInspector() },
  // Building envelope tab
  { id: 'add-zone', labelKey: 'ribbon.addZone', group: 'building', ribbonSource: 'Gebouwschil › Zone toevoegen', shellPlace: 'Gebouw › Toevoegen', run: dialog('zone-editor') },
  { id: 'add-construction', labelKey: 'ribbon.addConstruction', group: 'building', ribbonSource: 'Gebouwschil › Constructie toevoegen', shellPlace: 'Gebouw › Toevoegen', run: dialog('construction-editor') },
  { id: 'add-surface', labelKey: 'ribbon.addSurface', group: 'building', ribbonSource: 'Gebouwschil › Oppervlak toevoegen', shellPlace: 'Gebouw › Toevoegen', run: dialog('surface-editor') },
  { id: 'add-window', labelKey: 'ribbon.addWindow', group: 'building', ribbonSource: 'Gebouwschil › Raam toevoegen', shellPlace: 'Gebouw › Toevoegen', run: dialog('window-editor') },
  { id: 'add-thermal-bridge', labelKey: 'ribbon.addThermalBridge', group: 'building', ribbonSource: 'Gebouwschil › Koudebrug toevoegen', shellPlace: 'Gebouw › Toevoegen', run: dialog('thermal-bridge') },
  { id: 'add-point-bridge', labelKey: 'kernel.pointBridge.add', group: 'building', ribbonSource: 'Gebouwschil › Puntkoudebrug toevoegen', shellPlace: 'Gebouw › Toevoegen', run: dialog('point-bridge') },
  { id: 'air-tightness', labelKey: 'ribbon.airTightness', group: 'building', ribbonSource: 'Gebouwschil › Luchtdichtheid', shellPlace: 'Gebouw › Toevoegen', run: dialog('air-tightness') },
  // Installations and renewables tabs
  { id: 'add-heating', labelKey: 'ribbon.addHeating', group: 'installations', ribbonSource: 'Installaties › Verwarming toevoegen', shellPlace: 'Installaties › Toevoegen', run: dialog('heating-system') },
  { id: 'add-ventilation', labelKey: 'ribbon.addVentilation', group: 'installations', ribbonSource: 'Installaties › Ventilatie toevoegen', shellPlace: 'Installaties › Toevoegen', run: dialog('ventilation-system') },
  { id: 'add-cooling', labelKey: 'ribbon.addCooling', group: 'installations', ribbonSource: 'Installaties › Koeling toevoegen', shellPlace: 'Installaties › Toevoegen', run: dialog('cooling-system') },
  { id: 'add-hot-water', labelKey: 'ribbon.addHotWater', group: 'installations', ribbonSource: 'Installaties › Warm water toevoegen', shellPlace: 'Installaties › Toevoegen', run: dialog('hot-water-system') },
  { id: 'add-pv', labelKey: 'ribbon.addSolarPV', group: 'installations', ribbonSource: 'Hernieuwbaar › PV toevoegen', shellPlace: 'Installaties › Toevoegen', run: dialog('solar-pv') },
  { id: 'add-solar-thermal', labelKey: 'ribbon.addSolarThermal', group: 'installations', ribbonSource: 'Hernieuwbaar › Zonneboiler toevoegen', shellPlace: 'Installaties › Toevoegen', run: dialog('solar-thermal') },
  // Results tab
  { id: 'results', labelKey: 'ribbon.bengResults', group: 'calculation', ribbonSource: 'Resultaten › BENG Resultaten', shellPlace: 'Stap 5 Resultaten', run: (a) => a.navigate({ step: 'results' }) },
  // Report tab
  { id: 'export-report', labelKey: 'report.export', group: 'report', ribbonSource: 'Rapport › Exporteer rapport', shellPlace: 'Rapport & dossier › Exports', run: (a) => a.exportReport() },
  { id: 'print-report', labelKey: 'report.print', group: 'report', ribbonSource: 'Rapport › Afdrukken', shellPlace: 'Rapport & dossier › Exports', run: (a) => a.printReport() },
  { id: 'export-ifc', labelKey: 'report.page.ifc', group: 'report', ribbonSource: 'Rapport › IFC Export', shellPlace: 'Rapport & dossier › Exports', run: (a) => a.exportIFC() },
  { id: 'export-uniec3', labelKey: 'ribbon.exportUNIEC3Draft', group: 'exchange', ribbonSource: 'Rapport › UNIEC3 invoerconcept', shellPlace: 'Gereedschap › Uitwisseling', run: (a) => a.exportUNIEC3() },
  { id: 'import-uniec3', labelKey: 'ribbon.importUNIEC3', group: 'exchange', ribbonSource: 'Rapport › UNIEC3 import', shellPlace: 'Project › Importeren · Gereedschap', run: (a) => a.importUNIEC3() },
  { id: 'export-vabi', labelKey: 'ribbon.exportVABI', group: 'exchange', ribbonSource: 'Rapport › VABI export', shellPlace: 'Gereedschap › Uitwisseling', run: (a) => a.exportVABI() },
  { id: 'import-vabi', labelKey: 'ribbon.importVABI', group: 'exchange', ribbonSource: 'Rapport › VABI import', shellPlace: 'Project › Importeren · Gereedschap', run: (a) => a.importVABI() },
  // 3D model tab
  { id: 'model3d', labelKey: 'nav.sub.building.model3d', group: 'building', ribbonSource: '3D Model (tab)', shellPlace: 'Gebouw › 3D-model', run: (a) => a.navigate({ step: 'building', sub: 'model3d' }) },
  { id: 'export-model-ifc', labelKey: 'ribbon.exportModelIFC', group: 'exchange', ribbonSource: '3D Model › IFC 3D Model', shellPlace: 'Gebouw › 3D-model · Gereedschap', run: (a) => a.exportModelIFC() },
  // Tools tab
  { id: 'tool-uvalue', labelKey: 'ribbon.uvalueCalc', group: 'tools', ribbonSource: 'Gereedschap › U-waarde calculator', shellPlace: 'Gereedschap', run: (a) => a.navigate({ step: 'tool', sub: 'uvalue' }) },
  { id: 'tool-thermal-bridge', labelKey: 'ribbon.thermalBridgeCalc', group: 'tools', ribbonSource: 'Gereedschap › Koudebrug calculator', shellPlace: 'Gereedschap', run: (a) => a.navigate({ step: 'tool', sub: 'thermal-bridge' }) },
  { id: 'tool-heat-pump-sizing', labelKey: 'ribbon.heatPumpSizing', group: 'tools', ribbonSource: 'Gereedschap › Warmtepomp dimensionering', shellPlace: 'Gereedschap', run: (a) => a.navigate({ step: 'tool', sub: 'heat-pump-sizing' }) },
  // Title bar
  { id: 'settings', labelKey: 'nav.settings', group: 'app', ribbonSource: 'Titelbalk › Instellingen', shellPlace: 'Navigatie › Instellingen', shortcut: 'Ctrl ,', run: (a) => a.openSettings() },
  { id: 'feedback', labelKey: 'nav.feedback', group: 'app', ribbonSource: 'Titelbalk › Send Feedback', shellPlace: 'Gereedschap', run: (a) => a.openFeedback() },
];

export function commandById(id: string): ShellCommand | undefined {
  return SHELL_COMMANDS.find((command) => command.id === id);
}
