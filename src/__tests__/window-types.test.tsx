/**
 * Window types (kozijntypen, 8 Oct 2026): a window refers to a type with a
 * count; U, g and the area are resolved onto the window, and a change of the
 * type reaches every window that refers to it.
 */
import { describe, expect, it } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { applyWindowTypes, detachWindowType, resolveWindow, windowTypeUsage } from '../core/energy/windowTypes';
import { normalizeProject } from '../core/energy/normalizeProject';
import { WindowEditorDialog } from '../components/dialogs/WindowEditorDialog/WindowEditorDialog';
import type { IProject, IWindowType } from '../core/energy/types';

const type: IWindowType = { id: 'wt-1', name: 'Merk A', kind: 'window', uValue: 0.88, gValue: 0.6, unitArea: 1.05 };

function typedProject(): IProject {
  const project = createDefaultProject();
  const surface = project.zones[0].surfaces[0];
  surface.windows = surface.windows.map((window, index) => (index === 0 ? { ...window, typeId: 'wt-1', count: 3 } : window));
  return { ...project, windowTypes: [type] };
}

describe('window types', () => {
  it('resolves U, g and count × unit area onto the window', () => {
    const window = { id: 'w', name: 'Raam', area: 9, uValue: 1.5, gValue: 0.5, orientation: 'S' as const, surfaceId: 's', typeId: 'wt-1', count: 3 };
    expect(resolveWindow(window, type)).toMatchObject({ uValue: 0.88, gValue: 0.6, area: 3.15 });
    // Without a unit area the entered area stays.
    expect(resolveWindow(window, { ...type, unitArea: undefined }).area).toBe(9);
    expect(resolveWindow(window, undefined)).toBe(window);
  });

  it('writes a type change through to its windows and keeps other objects identical', () => {
    const project = typedProject();
    const resolved = applyWindowTypes(project);
    const first = resolved.zones[0].surfaces[0].windows[0];
    expect(first).toMatchObject({ uValue: 0.88, gValue: 0.6, area: 3.15 });
    expect(resolved.zones[0].surfaces[1]).toBe(project.zones[0].surfaces[1]);
    expect(applyWindowTypes(resolved)).toBe(resolved);
    expect(windowTypeUsage(project, 'wt-1')).toBe(1);
    const detached = detachWindowType(resolved, 'wt-1');
    expect(detached.zones[0].surfaces[0].windows[0]).not.toHaveProperty('typeId');
    expect(detached.zones[0].surfaces[0].windows[0].uValue).toBe(0.88);
  });

  it('normalises the window types when present and leaves a project without them alone', () => {
    const plain = createDefaultProject();
    expect(normalizeProject(plain)).not.toHaveProperty('windowTypes');
    expect(() => normalizeProject({ ...plain, windowTypes: [{ ...type, uValue: -1 }] })).toThrow(/windowTypes\[0\]\.uValue/);
  });

  it('lets the window editor pick a type, which fills U and g and derives the area from the count', async () => {
    const user = userEvent.setup();
    function Harness() {
      const { state, dispatch } = useEnergy();
      if (!state.project.windowTypes) dispatch({ type: 'ADD_WINDOW_TYPE', payload: type });
      const windows = state.project.zones.flatMap((zone) => zone.surfaces.flatMap((surface) => surface.windows));
      return <>
        <WindowEditorDialog onClose={() => undefined} />
        <output data-testid="windows">{JSON.stringify(windows.filter((window) => window.typeId))}</output>
      </>;
    }
    renderWithProviders(<Harness />);
    await user.type(screen.getByLabelText('Name'), 'Voorraam');
    await user.selectOptions(screen.getByLabelText('Window type'), 'wt-1');
    expect(screen.getByLabelText('Uw-value (W/m²K)')).toBeDisabled();
    // A controlled number input: set the value in one change (typing after clear would append to the 1).
    fireEvent.change(screen.getByLabelText('Count'), { target: { value: '2' } });
    expect(screen.getByLabelText('Area (m²)')).toHaveValue(2.1);
    await user.selectOptions(screen.getByLabelText('Zone'), screen.getAllByRole('option').find((option) => option.textContent === 'Woonfunctie')!.getAttribute('value')!);
    const surfaceSelect = screen.getByLabelText('Surface (parent)') as HTMLSelectElement;
    await user.selectOptions(surfaceSelect, surfaceSelect.options[1].value);
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const stored = JSON.parse(screen.getByTestId('windows').textContent ?? '[]');
    expect(stored).toHaveLength(1);
    expect(stored[0]).toMatchObject({ name: 'Voorraam', typeId: 'wt-1', count: 2, uValue: 0.88, gValue: 0.6, area: 2.1 });
  }, 60000);
});
