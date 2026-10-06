/**
 * Workflow navigation and step pages — the ribbon tests rewritten for the app
 * shell (UI redesign F4). Same intent: every former ribbon action still calls
 * the same handler or opens the same editor, now from the step where it
 * belongs, the Tools menu or the command palette.
 */
import { describe, expect, it, vi } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { useEnergy } from '../context/EnergyContext';
import { WorkflowNav } from '../components/shell/WorkflowNav';
import { StepRouter } from '../components/shell/StepRouter';
import { SHELL_COMMANDS } from '../components/shell/commands';
import type { ShellActions } from '../components/shell/ShellActions';
import { stepStatuses } from '../core/nta/stepStatus';
import type { Route } from '../core/navigation/routes';
import { en } from '../i18n/en';
import { nl } from '../i18n/nl';

function makeActions(overrides: Partial<ShellActions> = {}): ShellActions {
  return {
    newProject: vi.fn(), openProject: vi.fn(), saveProject: vi.fn(), saveAsProject: vi.fn(), calculate: vi.fn(),
    openDialog: vi.fn(), navigate: vi.fn(), exportReport: vi.fn(), printReport: vi.fn(), exportIFC: vi.fn(),
    exportModelIFC: vi.fn(), exportUNIEC3: vi.fn(), importUNIEC3: vi.fn(), exportVABI: vi.fn(), importVABI: vi.fn(),
    openSettings: vi.fn(), openFeedback: vi.fn(), openPalette: vi.fn(), toggleInspector: vi.fn(), togglePreview: vi.fn(),
    ...overrides,
  };
}

/** Navigation and work area on the real document state; `navigate` dispatches NAVIGATE. */
function Shell({ actions, route }: { actions: ShellActions; route?: Route }) {
  const { state, dispatch } = useEnergy();
  const shown = route ?? state.route;
  const wired: ShellActions = {
    ...actions,
    navigate: (target) => { actions.navigate(target); dispatch({ type: 'NAVIGATE', payload: target }); },
  };
  const statuses = stepStatuses(state.project, null);
  return <>
    <WorkflowNav project={state.project} route={shown} statuses={statuses} actions={wired} />
    <main><StepRouter project={state.project} route={shown} statuses={statuses} actions={wired} /></main>
    <output data-testid="route">{JSON.stringify(state.route)}</output>
  </>;
}

const route = () => JSON.parse(screen.getByTestId('route').textContent ?? '{}') as Route;

describe('WorkflowNav', () => {
  it('shows the input flow, what comes after and no ribbon tabs', () => {
    renderWithProviders(<Shell actions={makeActions()} />);
    const nav = screen.getByRole('navigation', { name: 'Workflow steps' });
    expect(within(nav).getByText('Input')).toBeInTheDocument();
    const steps = within(nav).getAllByRole('button').filter((button) => button.classList.contains('nav-step') && button.dataset.step);
    expect(steps.map((button) => button.dataset.step)).toEqual([
      'project', 'building', 'installations', 'check', 'results', 'report', 'registration',
    ]);
    expect(document.querySelector('.ribbon-tab, .ribbon-container')).toBeNull();
  });

  it('marks the active step with aria-current and navigates on click', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} />);
    const nav = screen.getByRole('navigation', { name: 'Workflow steps' });
    expect(within(nav).getByRole('button', { name: /^Project/ })).toHaveAttribute('aria-current', 'page');
    await user.click(within(nav).getByRole('button', { name: /^Building/ }));
    expect(actions.navigate).toHaveBeenCalledWith({ step: 'building', sub: 'envelope' });
    expect(route()).toEqual({ step: 'building', sub: 'envelope' });
    expect(within(nav).getByRole('button', { name: /^Building/ })).toHaveAttribute('aria-current', 'page');
    expect(within(nav).getByRole('button', { name: /^Project/ })).not.toHaveAttribute('aria-current');
    // The questions of the active step are listed below it; the 3D model is under the other parts.
    await user.click(within(nav).getByRole('button', { name: /Thermal bridges/ }));
    expect(route()).toEqual({ step: 'building', sub: 'thermalBridges' });
    await user.click(within(nav).getByRole('button', { name: '3D model' }));
    expect(route()).toEqual({ step: 'building', sub: 'model3d' });
  });

  it('moves focus through the steps with the arrow keys', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Shell actions={makeActions()} />);
    const nav = screen.getByRole('navigation', { name: 'Workflow steps' });
    within(nav).getByRole('button', { name: /^Project/ }).focus();
    await user.keyboard('{ArrowDown}');
    expect(within(nav).getByRole('button', { name: /^Building/ })).toHaveFocus();
    await user.keyboard('{End}');
    const all = Array.from(nav.querySelectorAll<HTMLButtonElement>('ol.nav-steps button.nav-step, ol.nav-steps button.nav-sub'));
    expect(all[all.length - 1]).toHaveFocus();
  });

  it('offers the calculators and exchange formats in the Tools menu', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} />);
    const open = async () => user.click(screen.getByRole('button', { name: 'Tools' }));
    const menu = async () => { await open(); return screen.getByRole('menu', { name: 'Tools and exchange' }); };

    await user.click(within(await menu()).getByRole('menuitem', { name: 'U-value Calculator' }));
    expect(actions.navigate).toHaveBeenLastCalledWith({ step: 'tool', sub: 'uvalue' });
    await user.click(within(await menu()).getByRole('menuitem', { name: 'Thermal Bridge Calculator' }));
    expect(actions.navigate).toHaveBeenLastCalledWith({ step: 'tool', sub: 'thermal-bridge' });
    await user.click(within(await menu()).getByRole('menuitem', { name: 'Heat Pump Sizing' }));
    expect(actions.navigate).toHaveBeenLastCalledWith({ step: 'tool', sub: 'heat-pump-sizing' });
    await user.click(within(await menu()).getByRole('menuitem', { name: 'UNIEC3 input draft' }));
    expect(actions.exportUNIEC3).toHaveBeenCalledOnce();
    await user.click(within(await menu()).getByRole('menuitem', { name: 'UNIEC3 Import' }));
    expect(actions.importUNIEC3).toHaveBeenCalledOnce();
    await user.click(within(await menu()).getByRole('menuitem', { name: 'VABI Export' }));
    expect(actions.exportVABI).toHaveBeenCalledOnce();
    await user.click(within(await menu()).getByRole('menuitem', { name: 'VABI Import' }));
    expect(actions.importVABI).toHaveBeenCalledOnce();
    await user.click(within(await menu()).getByRole('menuitem', { name: 'IFC 3D Model' }));
    expect(actions.exportModelIFC).toHaveBeenCalledOnce();
    await user.click(within(await menu()).getByRole('menuitem', { name: 'Send feedback' }));
    expect(actions.openFeedback).toHaveBeenCalledOnce();
  }, 60000);

  it('closes the Tools menu on Escape and returns focus to its button', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Shell actions={makeActions()} />);
    await user.click(screen.getByRole('button', { name: 'Tools' }));
    expect(screen.getByRole('menuitem', { name: 'U-value Calculator' })).toHaveFocus();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('menu')).toBeNull();
    expect(screen.getByRole('button', { name: 'Tools' })).toHaveFocus();
  });

  it('opens the settings from the navigation footer', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} />);
    await user.click(screen.getByRole('button', { name: /^Settings/ }));
    expect(actions.openSettings).toHaveBeenCalledOnce();
  });
});

describe('Step pages keep the ribbon actions', () => {
  it('Project: project info and imports', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} route={{ step: 'project' }} />);
    // The project data are the first question of the flow, filled in on the page itself.
    expect(screen.getByRole('heading', { level: 1, name: 'What kind of project is it?' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /Office/ })).toHaveAttribute('aria-pressed', 'false');
    await user.click(screen.getByRole('button', { name: 'UNIEC3 Import' }));
    expect(actions.importUNIEC3).toHaveBeenCalledOnce();
    await user.click(screen.getByRole('button', { name: 'VABI Import' }));
    expect(actions.importVABI).toHaveBeenCalledOnce();
  });

  // F5: each Gebouw sub page has its own add actions.
  it.each([
    [/Add zone/i, 'zone-editor', 'zones'],
    [/Add construction/i, 'construction-editor', 'constructions'],
    [/Add surface/i, 'surface-editor', 'envelope'],
    [/Add window/i, 'window-editor', 'envelope'],
    [/Add thermal bridge/i, 'thermal-bridge', 'thermalBridges'],
    [/Add point thermal bridge/i, 'point-bridge', 'thermalBridges'],
    [/Air tightness/i, 'air-tightness', 'airTightness'],
  ])('Building: %s opens %s', async (title, dialog, sub) => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} route={{ step: 'building', sub }} />);
    await user.click(within(screen.getByRole('group', { name: 'Add' })).getByTitle(title));
    expect(actions.openDialog).toHaveBeenCalledWith(dialog);
  });

  it.each([
    [/Add heating/i, 'heating-system'],
    [/Add ventilation/i, 'ventilation-system'],
    [/Add cooling/i, 'cooling-system'],
    [/Add hot water/i, 'hot-water-system'],
    [/Add PV/i, 'solar-pv'],
    [/Add Solar Thermal/i, 'solar-thermal'],
  ])('Installations: %s opens %s', async (title, dialog) => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} route={{ step: 'installations', sub: 'systems' }} />);
    await user.click(within(screen.getByRole('group', { name: 'Add' })).getByTitle(title));
    expect(actions.openDialog).toHaveBeenCalledWith(dialog);
  });

  it('Installations: lists the systems of a service on its sub page (formerly the project tree)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Shell actions={makeActions()} route={{ step: 'installations', sub: 'heating' }} />);
    expect(screen.getByRole('heading', { name: /^Heating \(1\)/ })).toBeInTheDocument();
    const row = screen.getByRole('row', { name: /Warmtepomp lucht/ });
    expect(row).toHaveAttribute('data-path', 'heatingSystems[0]');
    await user.click(within(row).getByText(/Warmtepomp lucht/));
    expect(row).toHaveClass('selected');
    expect(within(row).getByRole('button', { name: /^Edit: Warmtepomp lucht/ })).toBeInTheDocument();
  });

  it('Installations: the overview shows a chain card per present service and links to its sub page', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} route={{ step: 'installations', sub: 'systems' }} />);
    const heating = screen.getByRole('heading', { name: 'Heating', level: 2 }).closest('section') as HTMLElement;
    expect(within(heating).getByText(/Warmtepomp lucht/).closest('[data-path]')).toHaveAttribute('data-path', 'heatingSystems[0]');
    await user.click(within(heating).getByRole('button', { name: 'Open: Heating' }));
    expect(actions.navigate).toHaveBeenCalledWith({ step: 'installations', sub: 'heating' });
    // Services without systems or NTA input share one dashed card.
    expect(screen.getByRole('region', { name: 'Services not present' })).toHaveTextContent(/Humidification/);
  });

  it('Building: zones and constructions have their own sub pages (formerly the project tree)', () => {
    const { unmount } = renderWithProviders(<Shell actions={makeActions()} route={{ step: 'building', sub: 'zones' }} />);
    expect(screen.getByRole('row', { name: /Woonfunctie/ })).toHaveAttribute('data-path', 'zones[0]');
    unmount();
    renderWithProviders(<Shell actions={makeActions()} route={{ step: 'building', sub: 'constructions' }} />);
    // The page title is the question of the flow, the section keeps its own heading.
    expect(screen.getByRole('heading', { name: 'Which constructions are used?', level: 1 })).toBeInTheDocument();
    expect(screen.getAllByRole('heading', { name: 'Constructions' })).toHaveLength(1);
  });

  it('Report: export, print, IFC and exchange exports', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    const { unmount } = renderWithProviders(<Shell actions={actions} route={{ step: 'report' }} />);
    const subtabs = screen.getByRole('navigation', { name: 'Report & dossier' });
    expect(within(subtabs).getAllByRole('button').map((tab) => tab.textContent))
      .toEqual(['Calculation report', 'Input dossier', 'BRL 9500 checklist', 'Exports']);
    const head = document.querySelector('.page-actions') as HTMLElement;
    await user.click(within(head).getByRole('button', { name: 'Export Report' }));
    expect(actions.exportReport).toHaveBeenCalledOnce();
    await user.click(within(head).getByRole('button', { name: 'Print' }));
    expect(actions.printReport).toHaveBeenCalledOnce();
    await user.click(within(head).getByRole('button', { name: 'Exports' }));
    expect(actions.navigate).toHaveBeenCalledWith({ step: 'report', sub: 'exports' });
    unmount();
    // UI redesign F9: the exchange exports moved to the Exports sub page.
    renderWithProviders(<Shell actions={actions} route={{ step: 'report', sub: 'exports' }} />);
    const body = document.querySelector('.page-body') as HTMLElement;
    await user.click(within(body).getByRole('button', { name: 'IFC (BENG)' }));
    expect(actions.exportIFC).toHaveBeenCalledOnce();
    await user.click(within(body).getByRole('button', { name: 'UNIEC3 input draft' }));
    expect(actions.exportUNIEC3).toHaveBeenCalledOnce();
    await user.click(within(body).getByRole('button', { name: 'VABI Export' }));
    expect(actions.exportVABI).toHaveBeenCalledOnce();
    await user.click(within(body).getByRole('button', { name: 'IFC 3D Model' }));
    expect(actions.exportModelIFC).toHaveBeenCalledOnce();
  }, 30000);

  it('Report: the BRL 9500 checklist shows every status, the evidence and the EP-Online overview', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} route={{ step: 'report', sub: 'checklist' }} />);
    expect(screen.getByRole('heading', { name: 'BRL 9500 checklist', level: 1 })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Export project dossier (ZIP)' })).toBeInTheDocument();
    const rows = document.querySelectorAll('.delivery-checklist tr[data-status]');
    expect(rows.length).toBeGreaterThan(10);
    const statuses = new Set(Array.from(rows, (row) => row.getAttribute('data-status')));
    for (const status of statuses) expect(['ok', 'missing', 'check', 'not_applicable', 'pending']).toContain(status);
    expect(screen.getByTestId('ep-online-overview')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Edit in Registration' }));
    expect(actions.navigate).toHaveBeenCalledWith({ step: 'registration', focusPath: 'registration.evidence' });
  }, 30000);

  it('3D model: exports the IFC model', async () => {
    const user = userEvent.setup();
    const actions = makeActions();
    renderWithProviders(<Shell actions={actions} route={{ step: 'building', sub: 'model3d' }} />);
    await user.click(within(document.querySelector('.page-actions') as HTMLElement).getByRole('button', { name: 'IFC 3D Model' }));
    expect(actions.exportModelIFC).toHaveBeenCalledOnce();
  }, 30000);
});

describe('ribbon action inventory', () => {
  const RIBBON_ACTIONS = [
    'new', 'open', 'save', 'save-as', 'project-info', 'calculate', 'toggle-preview',
    'add-zone', 'add-construction', 'add-surface', 'add-window', 'add-thermal-bridge', 'add-point-bridge', 'air-tightness',
    'add-heating', 'add-ventilation', 'add-cooling', 'add-hot-water', 'add-pv', 'add-solar-thermal',
    'results', 'export-report', 'print-report', 'export-ifc', 'export-uniec3', 'import-uniec3', 'export-vabi', 'import-vabi',
    'model3d', 'export-model-ifc', 'tool-uvalue', 'tool-thermal-bridge', 'tool-heat-pump-sizing', 'settings', 'feedback',
  ];

  it('maps every former ribbon, title-bar and file-menu action to a shell command', () => {
    const ids = SHELL_COMMANDS.map((command) => command.id);
    for (const id of RIBBON_ACTIONS) expect(ids).toContain(id);
    for (const command of SHELL_COMMANDS) {
      expect(command.ribbonSource).not.toBe('');
      expect(command.shellPlace).not.toBe('');
      expect(en[command.labelKey], command.labelKey).toBeTruthy();
      expect(nl[command.labelKey], command.labelKey).toBeTruthy();
    }
  });

  it('runs each command through the same shell action', () => {
    const actions = makeActions();
    for (const command of SHELL_COMMANDS) command.run(actions);
    expect(actions.openDialog).toHaveBeenCalledWith('zone-editor');
    expect(actions.openDialog).toHaveBeenCalledWith('solar-thermal');
    expect(actions.openDialog).toHaveBeenCalledWith('project-info');
    expect(actions.calculate).toHaveBeenCalledOnce();
    expect(actions.togglePreview).toHaveBeenCalledOnce();
    expect(actions.navigate).toHaveBeenCalledWith({ step: 'results' });
    expect(actions.navigate).toHaveBeenCalledWith({ step: 'tool', sub: 'uvalue' });
    for (const handler of ['newProject', 'openProject', 'saveProject', 'saveAsProject', 'exportReport', 'printReport', 'exportIFC',
      'exportModelIFC', 'exportUNIEC3', 'importUNIEC3', 'exportVABI', 'importVABI', 'openSettings', 'openFeedback'] as const) {
      expect(actions[handler], handler).toHaveBeenCalledOnce();
    }
  });
});
