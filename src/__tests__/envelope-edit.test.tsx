import { describe, expect, it } from 'vitest';
import { fireEvent, screen, within } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { EnvelopeView } from '../components/EnvelopeView/EnvelopeView';
import { ProjectBrowser } from '../components/ProjectBrowser/ProjectBrowser';
import { PropertiesPanel } from '../components/PropertiesPanel/PropertiesPanel';
import { WindowEditorDialog } from '../components/dialogs/WindowEditorDialog/WindowEditorDialog';
import { SurfaceEditorDialog } from '../components/dialogs/SurfaceEditorDialog/SurfaceEditorDialog';
import { ConstructionEditorDialog } from '../components/dialogs/ConstructionEditorDialog/ConstructionEditorDialog';
import { ZoneEditorDialog } from '../components/dialogs/ZoneEditorDialog/ZoneEditorDialog';
import { deleteTarget } from '../core/energy/projectItems';
import { renderWithProviders, userEvent } from './test-utils';

function Harness({ browser = false, properties = false }: { browser?: boolean; properties?: boolean }) {
  const { state, dispatch } = useEnergy();
  const close = () => dispatch({ type: 'CLOSE_DIALOG' });
  const { dialog, project } = state;
  return <>
    <button type="button">Trigger</button>
    {browser && <ProjectBrowser />}
    <EnvelopeView />
    {properties && <PropertiesPanel />}
    {dialog.type === 'window-editor' && <WindowEditorDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'surface-editor' && <SurfaceEditorDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'construction-editor' && <ConstructionEditorDialog editId={dialog.editId} onClose={close} />}
    {dialog.type === 'zone-editor' && <ZoneEditorDialog editId={dialog.editId} onClose={close} />}
    <output data-testid="windows-s">{JSON.stringify(project.zones[0]?.surfaces.find((s) => s.id === 'surf-wall-s')?.windows ?? [])}</output>
    <output data-testid="surface-ids">{JSON.stringify(project.zones[0]?.surfaces.map((s) => s.id) ?? [])}</output>
    <output data-testid="constructions">{JSON.stringify(project.constructions.map((c) => ({ id: c.id, name: c.name })))}</output>
  </>;
}

const windowsOnSouth = () => JSON.parse(screen.getByTestId('windows-s').textContent ?? '[]') as Array<{ id: string; name: string; uValue: number }>;

describe('editing existing envelope elements', () => {
  it('edits an existing window from the envelope view and keeps its id and position', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const before = windowsOnSouth().map((w) => w.id);
    await user.click(within(screen.getByRole('row', { name: /Raam Zuid groot/ }))
      .getByRole('button', { name: 'Edit: Raam Zuid groot' }));
    const dialog = screen.getByRole('dialog');
    fireEvent.change(within(dialog).getByLabelText('Uw-value (W/m²K)'), { target: { value: '0.8' } });
    fireEvent.change(within(dialog).getByLabelText('Name'), { target: { value: 'Raam Zuid HR+++' } });
    await user.click(within(dialog).getByRole('button', { name: 'Save' }));
    const after = windowsOnSouth();
    expect(after.map((w) => w.id)).toEqual(before);
    expect(after[0]).toMatchObject({ id: 'win-s1', name: 'Raam Zuid HR+++', uValue: 0.8 });
  }, 60000);

  it('opens the surface editor on double-click and keeps the surface order', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const ids = screen.getByTestId('surface-ids').textContent;
    await user.dblClick(screen.getByRole('row', { name: /^Gevel Oost/ }));
    const dialog = screen.getByRole('dialog');
    fireEvent.change(within(dialog).getByLabelText('Gross Area (m²)'), { target: { value: '40' } });
    await user.click(within(dialog).getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('surface-ids').textContent).toBe(ids);
    expect(screen.getByRole('row', { name: /^Gevel Oost/ })).toHaveTextContent('40');
  }, 60000);

  it('deletes a window only after confirmation', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const row = screen.getByRole('row', { name: /Deur Zuid/ });
    await user.click(within(row).getByRole('button', { name: 'Delete: Deur Zuid' }));
    expect(windowsOnSouth().some((w) => w.id === 'win-s3')).toBe(true);
    await user.click(within(row).getByRole('button', { name: 'Cancel' }));
    expect(windowsOnSouth().some((w) => w.id === 'win-s3')).toBe(true);
    await user.click(within(row).getByRole('button', { name: 'Delete: Deur Zuid' }));
    await user.click(within(row).getByRole('button', { name: 'Yes, delete' }));
    expect(windowsOnSouth().map((w) => w.id)).toEqual(['win-s1', 'win-s2']);
  }, 60000);

  it('refuses to delete a construction that surfaces still use', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const constructions = screen.getByTestId('constructions').textContent;
    const name = JSON.parse(constructions ?? '[]').find((c: { id: string }) => c.id === 'con-wall').name as string;
    await user.click(screen.getByRole('button', { name: `Delete: ${name}` }));
    expect(screen.getByRole('alert')).toHaveTextContent(/surface/);
    expect(screen.getByTestId('constructions').textContent).toBe(constructions);
  }, 60000);

  it('opens the window editor by double-click in the project tree and from the properties panel', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness browser properties />);
    await user.click(screen.getByText('Surfaces'));
    await user.click(screen.getAllByText('Gevel Zuid')[0]);
    await user.dblClick(screen.getAllByText('Raam Zuid 2')[0]);
    expect(within(screen.getByRole('dialog')).getByLabelText('Name')).toHaveValue('Raam Zuid 2');
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).toBeNull();
    // The single click selected the window; the properties panel offers edit and delete.
    const panel = document.querySelector('.properties-panel') as HTMLElement;
    await user.click(within(panel).getByRole('button', { name: 'Edit: Raam Zuid 2' }));
    expect(within(screen.getByRole('dialog')).getByLabelText('Name')).toHaveValue('Raam Zuid 2');
  }, 60000);

  it('maps every item type to a delete action', () => {
    const project = { zones: [], constructions: [], heatingSystems: [{ id: 'h1' }] } as never;
    expect(deleteTarget(project, 'heatingSystem', 'h1')).toEqual({ kind: 'action', action: { type: 'DELETE_HEATING_SYSTEM', payload: 'h1' }, cascade: [], buildingMeasures: [] });
    expect(deleteTarget(project, 'surface', 'missing')).toEqual({ kind: 'none' });
    expect(deleteTarget(project, 'heatingSystem', 'missing')).toEqual({ kind: 'none' });
  });
});

describe('dialog keyboard handling', () => {
  it('moves focus in, traps Tab, closes on Escape and returns focus to the trigger', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const editButton = screen.getByRole('button', { name: 'Edit: Raam Zuid 2' });
    editButton.focus();
    await user.keyboard('{Enter}');
    const dialog = screen.getByRole('dialog');
    expect(dialog.contains(document.activeElement)).toBe(true);
    expect(within(dialog).getByLabelText('Name')).toHaveFocus();
    for (let i = 0; i < 25; i += 1) {
      await user.tab();
      expect(dialog.contains(document.activeElement)).toBe(true);
    }
    for (let i = 0; i < 5; i += 1) {
      await user.tab({ shift: true });
      expect(dialog.contains(document.activeElement)).toBe(true);
    }
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).toBeNull();
    expect(screen.getByRole('button', { name: 'Edit: Raam Zuid 2' })).toHaveFocus();
  }, 60000);

  it('labels every control in the zone, window, surface and construction dialogs', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const check = () => {
      const dialog = screen.getByRole('dialog');
      for (const control of dialog.querySelectorAll('input, select, textarea')) {
        const labelled = control.getAttribute('aria-label') || control.getAttribute('aria-labelledby')
          || (control.id && dialog.querySelector(`label[for="${CSS.escape(control.id)}"]`)) || control.closest('label');
        expect(labelled, control.outerHTML).toBeTruthy();
      }
    };
    for (const button of [/Edit: Woonfunctie/, /Edit: Raam Zuid 2/, /Edit: Gevel Oost/]) {
      await user.click(screen.getByRole('button', { name: button }));
      check();
      await user.keyboard('{Escape}');
    }
    const construction = JSON.parse(screen.getByTestId('constructions').textContent ?? '[]')[0].name as string;
    await user.click(screen.getByRole('button', { name: `Edit: ${construction}` }));
    check();
  }, 60000);
});
