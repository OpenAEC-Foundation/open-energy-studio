import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { useNtaDraft } from '../context/NtaDraftProvider';
import type { IProject } from '../core/energy/types';
import type { BuildingPerformanceInput, OpnameAssessment } from '../core/nta/KernelClient';
import {
  applyBridgeRoute, bridgeRouteItem, FORFAIT_BRIDGE_RULE, surveyBridgeRoute,
} from '../core/nta/SurveyTakeover';
import { SurveyTakeoverAction } from '../components/BasisopnamePanel/SurveyTakeoverAction';
import terraced from '../../training-data/nta8800-example-terraced-dwelling.json';
import { renderWithProviders } from './test-utils';

// Review 53 follow-up: the survey (ISSO 82.1/75.1) calculates with the forfait
// thermal bridges of 8.2.1 (ΔU_for of 8.3), so "Overnemen in projectmodel"
// proposes `thermalBridgeRoute: forfait` like any other value: ticked when the
// project has no method yet, unticked when it would mix methods.

type Loose = Record<string, unknown>;

const flat = { spaceHeating: { demand: { transmission: { groundFloors: [] } } } } as unknown as BuildingPerformanceInput;
const forfaitRule = { rule: FORFAIT_BRIDGE_RULE, path: 'envelope', value: 'ΔU_for 0.100 W/(m²K)', source: 'NTA 8800 §8.2.1' };

function assessment(appliedDefaults: unknown[], derivedInput: BuildingPerformanceInput = flat): OpnameAssessment & { derivedInput: BuildingPerformanceInput } {
  return {
    status: 'calculated_unverified', scope: 'residential', source: 'ISSO 82.1', appliedDefaults, warnings: [], issues: [],
    derivedInput, performance: null, referenceVerified: false,
  } as unknown as OpnameAssessment & { derivedInput: BuildingPerformanceInput };
}

describe('survey takeover of the thermal-bridge method', () => {
  it('reads forfait from the survey ΔU_for or a forfait floor edge, and nothing otherwise', () => {
    expect(surveyBridgeRoute(assessment([forfaitRule]))).toBe('forfait');
    const edge = { spaceHeating: { demand: { transmission: { groundFloors: [{ id: 'v', edgeThermalBridges: { method: 'forfait' } }] } } } };
    expect(surveyBridgeRoute(assessment([], edge as unknown as BuildingPerformanceInput))).toBe('forfait');
    expect(surveyBridgeRoute(assessment([]))).toBeNull();
    expect(bridgeRouteItem(null, {}, false)).toBeNull();
  });

  it('proposes forfait for an upper-floor dwelling without a ground floor', () => {
    const item = bridgeRouteItem('forfait', { groundFloors: [] }, false);
    expect(item).toEqual({ key: 'thermalBridgeRoute', from: undefined, to: 'forfait', status: 'new' });
    expect(applyBridgeRoute({}, item, new Set([item!.key])).thermalBridgeRoute).toBe('forfait');
    expect(applyBridgeRoute({}, item, new Set())).not.toHaveProperty('thermalBridgeRoute');
  });

  it('sees no change when the draft already has forfait', () => {
    expect(bridgeRouteItem('forfait', { thermalBridgeRoute: 'forfait' }, false)).toBeNull();
  });

  it('marks a conflict for another method, a detailed floor edge or ψ-values', () => {
    expect(bridgeRouteItem('forfait', { thermalBridgeRoute: 'detailed' }, false)?.status).toBe('conflict');
    const detailed = { groundFloors: [{ surfaceId: 'f', edgeThermalBridges: { method: 'detailed', bridges: [] } }] };
    expect(bridgeRouteItem('forfait', detailed, false)?.status).toBe('conflict');
    expect(bridgeRouteItem('forfait', {}, true)?.status).toBe('conflict');
    // A forfait floor edge already agrees with forfait.
    const forfaitEdge = { groundFloors: [{ surfaceId: 'f', edgeThermalBridges: { method: 'forfait' } }] };
    expect(bridgeRouteItem('forfait', forfaitEdge, false)?.status).toBe('new');
  });
});

function Seed({ value }: { value: IProject }) {
  const { dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: structuredClone(value) }); }, [dispatch, value]);
  return null;
}

function DraftProbe() {
  const shared = useNtaDraft();
  return <>
    <pre data-testid="draft">{JSON.stringify((shared?.draft ?? shared?.base ?? null) as Loose | null)}</pre>
    <button type="button" onClick={() => shared?.undo()}>probe-undo</button>
  </>;
}

const draft = () => JSON.parse(screen.getByTestId('draft').textContent ?? '{}') as Loose;

describe('survey takeover dialog for the thermal-bridge method', () => {
  it('ticks the proposal when the project has no method, writes it, and undo removes it', async () => {
    renderWithProviders(<>
      <Seed value={terraced as unknown as IProject} />
      <SurveyTakeoverAction result={assessment([forfaitRule])} />
      <DraftProbe />
    </>);
    await waitFor(() => expect(draft().groundFloors).toBeDefined());
    expect(draft()).not.toHaveProperty('thermalBridgeRoute');
    fireEvent.click(screen.getByRole('button', { name: /projectmodel|project model/i }));
    const dialog = await screen.findByRole('dialog');
    const route = within(dialog).getByRole('region', { name: /koudebrugmethode|thermal-bridge method/i });
    const box = within(route).getByRole('checkbox') as HTMLInputElement;
    expect(box.checked).toBe(true);
    fireEvent.click(within(dialog).getByRole('button', { name: /in concept|into draft/i }));
    await waitFor(() => expect(draft().thermalBridgeRoute).toBe('forfait'));
    fireEvent.click(screen.getByRole('button', { name: 'probe-undo' }));
    await waitFor(() => expect(draft()).not.toHaveProperty('thermalBridgeRoute'));
  });

  it('leaves a conflicting method unticked and unchanged', async () => {
    const project = structuredClone(terraced) as unknown as IProject & { ntaCalculation: Loose };
    project.ntaCalculation.thermalBridgeRoute = 'detailed';
    renderWithProviders(<>
      <Seed value={project} />
      <SurveyTakeoverAction result={assessment([forfaitRule])} />
      <DraftProbe />
    </>);
    await waitFor(() => expect(draft().thermalBridgeRoute).toBe('detailed'));
    fireEvent.click(screen.getByRole('button', { name: /projectmodel|project model/i }));
    const dialog = await screen.findByRole('dialog');
    const route = within(dialog).getByRole('region', { name: /koudebrugmethode|thermal-bridge method/i });
    expect((within(route).getByRole('checkbox') as HTMLInputElement).checked).toBe(false);
    expect(route.textContent).toMatch(/ψ/);
    // Nothing else differs, so there is nothing to take over until the adviser ticks it.
    expect((within(dialog).getByRole('button', { name: /in concept|into draft/i }) as HTMLButtonElement).disabled).toBe(true);
    expect(draft().thermalBridgeRoute).toBe('detailed');
  });

  it('shows nothing about the method when the survey implies none', async () => {
    renderWithProviders(<>
      <Seed value={terraced as unknown as IProject} />
      <SurveyTakeoverAction result={assessment([])} />
      <DraftProbe />
    </>);
    await waitFor(() => expect(draft().groundFloors).toBeDefined());
    fireEvent.click(screen.getByRole('button', { name: /projectmodel|project model/i }));
    const dialog = await screen.findByRole('dialog');
    expect(within(dialog).queryByRole('region', { name: /koudebrugmethode|thermal-bridge method/i })).toBeNull();
  });
});
