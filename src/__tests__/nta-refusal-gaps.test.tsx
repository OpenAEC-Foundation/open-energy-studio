/**
 * A route the building calculation refuses is a project gap at the project
 * input (kernel `building_issue_gaps`), so "Ga naar" lands on the field that
 * feeds it, not on the check overview.
 */
import { useEffect, useState } from 'react';
import { screen, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { useEnergy } from '../context/EnergyContext';
import type { Route } from '../core/navigation/routes';
import type { NtaCalculationInput } from '../core/nta/KernelClient';
import { buildNtaCalculationTemplate } from '../core/nta/NtaCalculationTemplate';
import { hasExplicitRoute, routeForPath } from '../core/nta/gapRoutes';
import { NtaStepSections } from '../components/shell/NtaStepPage';
import { focusPathIn } from '../components/shell/focusPath';
import { renderWithProviders } from './test-utils';

function Harness({ initial }: { initial: Route }) {
  const { state, dispatch } = useEnergy();
  const [route] = useState<Route>(initial);
  useEffect(() => {
    dispatch({ type: 'SET_NTA_CALCULATION', payload: buildNtaCalculationTemplate(state.project) as unknown as NtaCalculationInput });
    // Once, on mount.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  return <div data-testid="page"><NtaStepSections key={`${route.step}/${route.sub}`} route={route} /></div>;
}

describe('refused routes as project gaps', () => {
  it('routes the gap paths the kernel gives for refused routes', () => {
    // The example office under NTA 8800:2020+A1 (kernel test refused_routes_are_project_gaps).
    expect(routeForPath('ntaCalculation.pvSystems[0].peakPower.panelPeakPowerW'))
      .toMatchObject({ step: 'installations', sub: 'generation' });
    expect(routeForPath('ntaCalculation.lighting[0].lightingZones[0].power.ledFrom2017'))
      .toMatchObject({ step: 'installations', sub: 'lighting' });
    expect(routeForPath('ntaCalculation.internalGains.lighting'))
      .toMatchObject({ step: 'building', sub: 'zones' });
    // The label function had no route; the kernel refuses a mismatch with the scope there.
    expect(hasExplicitRoute('ntaCalculation.labelFunction')).toBe(true);
    expect(routeForPath('ntaCalculation.labelFunction')).toMatchObject({ step: 'project' });
  });

  it('routes the per-zone, per-window and per-system paths of refused routes', () => {
    // Kernel test refusal_paths_follow_zones_floors_and_systems.
    expect(routeForPath('ntaCalculation.zoneData[1].emission.fans'))
      .toMatchObject({ step: 'installations', sub: 'heating' });
    expect(routeForPath('ntaCalculation.zoneData[1].distribution'))
      .toMatchObject({ step: 'installations', sub: 'heating' });
    expect(routeForPath('ntaCalculation.zoneData[1].internalGains.lighting'))
      .toMatchObject({ step: 'building', sub: 'zones' });
    expect(routeForPath('ntaCalculation.windowObstructions[0].obstruction.method'))
      .toMatchObject({ step: 'building', sub: 'envelope' });
    expect(routeForPath('zones[1].surfaces[2].windows[0].gValue'))
      .toMatchObject({ step: 'building', sub: 'envelope' });
    expect(routeForPath('ntaCalculation.additionalHeatingSystems[0].identicalSystems'))
      .toMatchObject({ step: 'installations', sub: 'heating' });
    expect(routeForPath('ntaCalculation.groundFloors[1].heatedBasement'))
      .toMatchObject({ step: 'building', sub: 'envelope' });
  });

  it('lands "Ga naar" on the label function field', async () => {
    const route = routeForPath('ntaCalculation.labelFunction');
    renderWithProviders(<Harness initial={route} />);
    const page = await screen.findByTestId('page');
    const select = await within(page).findByLabelText('Label function of the building (§5.3.1)');
    expect(focusPathIn(page, 'ntaCalculation.labelFunction')).toBe(select);
  });
});
