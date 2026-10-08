import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import { WindowObstructionsFields, withWindowObstruction } from '../components/NtaPerformancePanel/NtaWindowObstructions';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { renderWithProviders, userEvent } from './test-utils';

const project = {
  id: 'p', name: 'p', description: '', buildingFunction: 'residential', address: '', city: '',
  zones: [{ id: 'z1', surfaces: [
    { id: 'vg', name: 'Voorgevel', type: 'wall', thermalBoundary: 'outdoor', area: 30, orientation: 'W', constructionId: 'c', zoneId: 'z1',
      windows: [{ id: 'win-W', name: 'Raam voor', area: 4.8, uValue: 1.4, gValue: 0.6, orientation: 'W', surfaceId: 'vg' }] },
    { id: 'ag', name: 'Achtergevel', type: 'wall', thermalBoundary: 'outdoor', area: 30, orientation: 'E', constructionId: 'c', zoneId: 'z1',
      windows: [{ id: 'win-E', name: 'Raam achter', area: 7.35, uValue: 1.4, gValue: 0.6, orientation: 'E', surfaceId: 'ag' }] },
    { id: 'u', name: 'Wand berging', type: 'wall', thermalBoundary: 'unheated', area: 10, orientation: 'N', constructionId: 'c', zoneId: 'z1',
      windows: [{ id: 'win-U', name: 'Raam berging', area: 1, uValue: 1.1, gValue: 0.6, orientation: 'N', surfaceId: 'u' }] },
  ] }],
  heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [], solarPV: [], solarThermal: [], constructions: [],
} as unknown as IProject;

function Harness({ initial = {} }: { initial?: Draft }) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    <WindowObstructionsFields draft={draft} change={change} project={project} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const current = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');

describe('Obstruction per window', () => {
  it('lists outdoor windows as the project and stores an own situation per window', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    expect(screen.queryByLabelText('Obstruction Raam berging')).toBeNull();
    const rear = screen.getByLabelText('Obstruction Raam achter') as HTMLSelectElement;
    expect(rear.value).toBe('');
    await user.selectOptions(rear, 'overhang');
    expect(current().windowObstructions).toEqual([
      { windowId: 'win-E', obstruction: { method: 'overhang', relativeHeight: null }, sourceReference: '' }]);
    const group = screen.getByRole('group', { name: 'Raam achter' });
    await user.type(within(group).getByLabelText('Relative height (h/a)'), '0.5');
    await user.type(within(group).getByLabelText('Source'), 'tekening');
    expect(current().windowObstructions[0]).toEqual(
      { windowId: 'win-E', obstruction: { method: 'overhang', relativeHeight: 0.5 }, sourceReference: 'tekening' });
    expect((screen.getByLabelText('Obstruction Raam voor') as HTMLSelectElement).value).toBe('');

    await user.selectOptions(screen.getByLabelText('Obstruction Raam achter'), '');
    expect(current().windowObstructions).toEqual([]);
  });

  it('keeps the source when the situation changes and offers removal of a stale window', () => {
    const entries = [{ windowId: 'win-E', obstruction: { method: 'overhang', relativeHeight: 0.5 }, sourceReference: 'tekening' }];
    expect(withWindowObstruction(entries, 'win-E', 'side_obstruction')).toEqual([{ windowId: 'win-E',
      obstruction: { method: 'side_obstruction', side: 'both', relativeWidth: null, coolingHeightCondition: false }, sourceReference: 'tekening' }]);
    renderWithProviders(<Harness initial={{ windowObstructions: [
      { windowId: 'gone', obstruction: { method: 'minimal' }, sourceReference: 'x' }] }} />);
    const stale = screen.getByRole('group', { name: 'gone (not in the project)' });
    expect(within(stale).getByRole('button', { name: 'Remove' })).toBeTruthy();
  });

  it('names a window that no longer borders outdoor air instead of calling it missing', () => {
    renderWithProviders(<Harness initial={{ windowObstructions: [
      { windowId: 'win-U', obstruction: { method: 'minimal' }, sourceReference: 'x' }] }} />);
    const stale = screen.getByRole('group', { name: /^Raam berging \(no longer in an outdoor surface/ });
    expect(stale.getAttribute('data-code')).toBe('window_obstruction_not_outdoor');
    expect(within(stale).getByRole('button', { name: 'Remove' })).toBeTruthy();
  });
});
