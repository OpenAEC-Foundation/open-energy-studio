import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import type { IProject } from '../core/energy/types';
import {
  deviceOf, deviceValue, WindowShadingsFields, withShadingDevice, withWindowShading,
} from '../components/NtaPerformancePanel/NtaWindowShadings';
import { VentilationUnitFields } from '../components/NtaPerformancePanel/NtaVentilationSection';
import { write, type Draft, type Path } from '../components/NtaPerformancePanel/NtaFormFields';
import { renderWithProviders, userEvent } from './test-utils';

const project = {
  id: 'p', name: 'p', description: '', buildingFunction: 'residential', address: '', city: '',
  zones: [{ id: 'z1', surfaces: [
    { id: 'dak', name: 'Dak west', type: 'roof', thermalBoundary: 'outdoor', area: 55, orientation: 'W', constructionId: 'c', zoneId: 'z1',
      windows: [{ id: 'DR', name: 'Dakraam', area: 1.51, uValue: 0.92, gValue: 0.5, orientation: 'W', surfaceId: 'dak' }] },
    { id: 'ag', name: 'Achtergevel', type: 'wall', thermalBoundary: 'outdoor', area: 30, orientation: 'S', constructionId: 'c', zoneId: 'z1',
      windows: [{ id: 'win-S', name: 'Pui', area: 7.6, uValue: 1.1, gValue: 0.6, orientation: 'S', surfaceId: 'ag' }] },
  ] }],
  heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [], solarPV: [], solarThermal: [], constructions: [],
} as unknown as IProject;

function Shadings({ initial = {} }: { initial?: Draft }) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    <WindowShadingsFields draft={draft} change={change} project={project} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

function Unit({ initial }: { initial: Draft }) {
  const [draft, setDraft] = useState<Draft>(initial);
  const change = (path: Path, value: unknown) => setDraft((current) => write(current, path, value));
  return <form aria-label="test">
    <VentilationUnitFields draft={draft} change={change} path={['ventilation', 'system', 'unit']} />
    <output data-testid="draft">{JSON.stringify(draft)}</output>
  </form>;
}

const current = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}');

describe('Shading per window', () => {
  it('stores screens on one window with a table 7.5 device and keeps the others as the project', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Shadings />);
    const roof = screen.getByLabelText('Shading Dakraam') as HTMLSelectElement;
    expect(roof.value).toBe('');
    await user.selectOptions(roof, 'manual_residential');
    expect(current().windowShadings).toEqual([
      { windowId: 'DR', movableShading: { sourceReference: '', control: 'manual_residential' }, sourceReference: '' }]);
    const group = screen.getByRole('group', { name: 'Dakraam' });
    await user.selectOptions(within(group).getByLabelText('Type (table 7.5/7.6)'), 'external_screen:dark');
    expect(current().windowShadings[0].movableShading.device).toEqual({ kind: 'external_screen', colour: 'dark' });
    await user.type(within(group).getByLabelText('Source'), 'printout');
    expect(current().windowShadings[0].sourceReference).toBe('printout');
    expect((screen.getByLabelText('Shading Pui') as HTMLSelectElement).value).toBe('');

    await user.selectOptions(screen.getByLabelText('Shading Dakraam'), '');
    expect(current().windowShadings).toEqual([]);
  });

  it('removes the project shading from one window with "none"', () => {
    expect(withWindowShading([], 'win-S', 'none')).toEqual([{ windowId: 'win-S', sourceReference: '' }]);
    const entries = [{ windowId: 'DR', movableShading: { control: 'manual_residential', device: { kind: 'drop_arm_awning' }, sourceReference: 's' },
      sourceReference: 'r' }];
    expect(withWindowShading(entries, 'DR', 'automatic')).toEqual([{ windowId: 'DR',
      movableShading: { control: 'automatic', device: { kind: 'drop_arm_awning' }, sourceReference: 's' }, sourceReference: 'r' }]);
    expect(withWindowShading(entries, 'DR', 'none')).toEqual([{ windowId: 'DR', sourceReference: 'r' }]);
  });

  it('switches between a table device and a declared F_c', () => {
    expect(deviceOf('external_venetian_blind:white')).toEqual({ kind: 'external_venetian_blind', colour: 'white' });
    expect(deviceOf('folding_arm_awning')).toEqual({ kind: 'folding_arm_awning' });
    expect(deviceValue({ kind: 'external_screen', colour: 'dark' })).toBe('external_screen:dark');
    const declared = withShadingDevice({ control: 'automatic', device: { kind: 'drop_arm_awning' }, sourceReference: '' }, '');
    expect(declared).toEqual({ control: 'automatic', reductionFactor: null, sourceReference: '' });
    expect(withShadingDevice({ ...declared, reductionFactor: 0.3 }, 'internal_metallised_fabric'))
      .toEqual({ control: 'automatic', device: { kind: 'internal_metallised_fabric' }, sourceReference: '' });
  });
});

describe('Declared f_ctrl', () => {
  it('adds a declared control factor with its declaration and removes it again', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Unit initial={{ ventilation: { system: { kind: 'single',
      unit: { variant: 'c4c', ducts: 'unknown', equipmentReference: 'Itho' } } } }} />);
    const toggle = screen.getByLabelText('Declared f_ctrl (quality declaration)') as HTMLInputElement;
    expect(toggle.checked).toBe(false);
    await user.click(toggle);
    expect(current().ventilation.system.unit.declaredControlFactor).toEqual({ value: null, declarationReference: '' });
    await user.type(screen.getByLabelText('f_ctrl'), '0.51');
    await user.type(screen.getByLabelText('Declaration (number, manufacturer, type)'), 'BCRG 20201914GG');
    expect(current().ventilation.system.unit.declaredControlFactor).toEqual({ value: 0.51, declarationReference: 'BCRG 20201914GG' });
    await user.click(screen.getByLabelText('Declared f_ctrl (quality declaration)'));
    expect(current().ventilation.system.unit.declaredControlFactor).toBeUndefined();
  });

  it('offers no declaration on the decentral part of E.1', () => {
    renderWithProviders(<form aria-label="test">
      <VentilationUnitFields draft={{ ventilation: { system: { kind: 'combined',
        decentral: { variant: 'd5b', ducts: 'unknown', equipmentReference: '' } } } }}
        change={() => undefined} path={['ventilation', 'system', 'decentral']} />
    </form>);
    expect(screen.queryByLabelText('Declared f_ctrl (quality declaration)')).toBeNull();
  });
});
