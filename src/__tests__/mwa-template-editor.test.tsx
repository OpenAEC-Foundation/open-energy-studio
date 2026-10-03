import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import { MaatwerkadviesPanel, regenerateTemplatePatches } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { renderWithProviders, userEvent } from './test-utils';

const dwelling = JSON.parse(readFileSync(resolve(process.cwd(), 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject;

function Editor() {
  const { state, dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: structuredClone(dwelling) }); }, [dispatch]);
  if (state.project.id !== dwelling.id) return null;
  return <><MaatwerkadviesPanel /><output data-testid="mwa">{JSON.stringify(state.project.maatwerkadvies ?? null)}</output></>;
}

const stored = () => JSON.parse(screen.getByTestId('mwa').textContent ?? 'null');

describe('maatwerkadvies measure templates in the panel', () => {
  it('builds an insulation measure from its template and falls back to manual patch rows', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(await screen.findByRole('button', { name: 'Start tailored advice' }));
    await user.click(screen.getByRole('button', { name: 'Add measure' }));
    // A new measure starts as insulation and says what it still needs.
    expect(screen.getByRole('combobox', { name: 'Measure type' })).toHaveValue('insulation');
    expect(screen.getByTestId('mwa-template-problems-m1')).toHaveTextContent('Enter the new value.');

    await user.selectOptions(screen.getByRole('combobox', { name: 'Building part' }), 'facade');
    await user.type(screen.getByRole('textbox', { name: 'New Rc, m²·K/W' }), '6');
    await user.click(screen.getByRole('checkbox', { name: 'Woning — Gevel N' }));
    await user.click(screen.getByRole('checkbox', { name: 'Woning — Gevel S' }));
    expect(screen.queryByTestId('mwa-template-problems-m1')).toBeNull();
    let measure = stored().measures[0];
    expect(measure.category).toBe('insulation');
    expect(measure.template).toMatchObject({ kind: 'insulation', part: 'facade', rcValue: 6, surfaces: ['z1/wall-N', 'z1/wall-S'] });
    expect(measure.patch).toEqual([
      { op: 'add', path: '/constructions/-', value: { id: 'm1-c-wall', name: 'Gevel Rc 4,7 (m1)', layers: [], rcValue: 6, uValue: 0.1621 } },
      { op: 'replace', path: '/zones/0/surfaces/0/constructionId', value: 'm1-c-wall' },
      { op: 'replace', path: '/zones/0/surfaces/1/constructionId', value: 'm1-c-wall' },
    ]);
    expect(screen.getByTestId('mwa-template-preview-m1')).toHaveTextContent('/zones/0/surfaces/1/constructionId');

    // Another template replaces the patch; "manual" keeps it as editable rows.
    await user.selectOptions(screen.getByRole('combobox', { name: 'Measure type' }), 'airtightness');
    measure = stored().measures[0];
    expect(measure.category).toBe('airtightness');
    expect(measure.patch).toEqual([]);
    await user.type(screen.getByRole('textbox', { name: 'Target qv10, dm³/(s·m²)' }), '0,25');
    await user.type(screen.getAllByRole('textbox', { name: 'Source / justification' }).slice(-1)[0], 'blowerdoortest');
    measure = stored().measures[0];
    expect(measure.patch[0]).toEqual({ op: 'replace', path: '/ntaCalculation/ventilation/infiltration',
      value: { method: 'measured', qv10DmPerSM2: 0.25, sourceReference: 'blowerdoortest' } });
    await user.selectOptions(screen.getByRole('combobox', { name: 'Measure type' }), 'manual');
    measure = stored().measures[0];
    expect(measure.template).toBeNull();
    expect(measure.patch).toHaveLength(2);
    expect(screen.getAllByRole('textbox', { name: 'Path (JSON pointer)' })[0]).toHaveValue('/ntaCalculation/ventilation/infiltration');
  }, 60000);

  it('renders every template with the NTA form fields', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(await screen.findByRole('button', { name: 'Start tailored advice' }));
    await user.click(screen.getByRole('button', { name: 'Add measure' }));
    const kind = () => screen.getByRole('combobox', { name: 'Measure type' });
    await user.selectOptions(kind(), 'ventilation');
    expect(screen.getByRole('combobox', { name: /variant/i })).toHaveValue('d2');
    expect(stored().measures[0].patch.map((operation: { path: string }) => operation.path)).toEqual(['/ntaCalculation/ventilation/system']);
    await user.selectOptions(kind(), 'heat_pump');
    expect(screen.getByRole('combobox', { name: 'Source' })).toHaveValue('outdoor_air');
    expect(screen.getByTestId('mwa-template-problems-m1')).toHaveTextContent('Enter the source.');
    await user.selectOptions(kind(), 'hot_water');
    expect(stored().measures[0].patch).toEqual([{ op: 'replace', path: '/ntaCalculation/hotWater/generator',
      value: { kind: 'heat_pump', exhaustAirSource: false, measuredClass: 'class4' } }]);
    await user.selectOptions(kind(), 'pv');
    expect(stored().measures[0].patch[0]).toMatchObject({ op: 'add', path: '/ntaCalculation/pvSystems/-', value: { id: 'm1-pv-1' } });
    await user.selectOptions(kind(), 'solar_water_heater');
    expect(stored().measures[0].patch[0]).toMatchObject({ op: 'add', path: '/ntaCalculation/hotWater/solar', value: [{ id: 'm1-solar-1' }] });
    await user.selectOptions(kind(), 'shower_heat_recovery');
    expect(stored().measures[0].category).toBe('hot_water');
    await user.selectOptions(kind(), 'lighting');
    expect(screen.getByTestId('mwa-template-problems-m1')).toHaveTextContent('The project has no chapter 14 lighting.');
  }, 60000);

  it('regenerates template patches against the current project', () => {
    const definition = {
      measures: [{
        id: 'm1', name: 'Glas', category: 'glazing' as const, target: 'project' as const, patch: [],
        investmentEur: 0, costSource: 'x', lifetimeYears: 30,
        template: { kind: 'glazing' as const, windows: ['z1/wall-S/win-S'], uValue: 0.7, gValue: null },
      }],
      packages: [],
      tariffs: { gasEurPerM3: 1.4, electricityEurPerKwh: 0.3, sourceReference: 'x' },
    };
    const project = structuredClone(dwelling);
    expect(regenerateTemplatePatches(project, definition).measures[0].patch)
      .toEqual([{ op: 'replace', path: '/zones/0/surfaces/1/windows/0/uValue', value: 0.7 }]);
    // The north wall removed: the south window moves to surface 0.
    project.zones[0].surfaces.splice(0, 1);
    expect(regenerateTemplatePatches(project, definition).measures[0].patch)
      .toEqual([{ op: 'replace', path: '/zones/0/surfaces/0/windows/0/uValue', value: 0.7 }]);
  });
});
