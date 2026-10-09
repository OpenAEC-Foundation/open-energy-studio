/**
 * Several opaque parts in one surface (planner, 9 Oct 2026): each part is a
 * kernel surface of its own that follows its surface in element, boundary,
 * orientation, tilt and zone; it has its own area and construction.
 */
import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { surveyTemplate, type StoredSurvey } from '../core/nta/SurveyTemplates';
import { addPart, parentOf, partsOf, removeSurface, syncParts } from '../core/survey/surfaceParts';

const facade = { id: 'gevel-1', element: 'facade', boundary: { kind: 'outdoor' }, orientation: 'south', grossAreaM2: 30, cavity: true,
  insulation: { kind: 'none_or_unknown' }, sourceReference: '' };
const withFacade = (): StoredSurvey => {
  const stored = surveyTemplate('residential');
  (stored.survey.envelope as { surfaces: unknown[] }).surfaces = [structuredClone(facade)];
  return stored;
};
const surfaces = (stored: StoredSurvey) => (stored.survey.envelope as { surfaces: Array<Record<string, unknown>> }).surfaces;

describe('surface parts', () => {
  it('adds a part that takes over the shared answers, follows them and goes with its surface', () => {
    const stored = addPart(withFacade(), 'gevel-1');
    const part = surfaces(stored)[1];
    expect(part).toMatchObject({ id: 'gevel-1-deel-2', element: 'facade', boundary: { kind: 'outdoor' }, orientation: 'south', grossAreaM2: 0, cavity: true });
    expect(parentOf(stored, 'gevel-1-deel-2')).toBe('gevel-1');
    expect(partsOf(stored, 'gevel-1')).toEqual(['gevel-1-deel-2']);

    surfaces(stored)[0].orientation = 'west';
    surfaces(stored)[1].insulation = { kind: 'thickness', thicknessMm: 80 };
    const synced = syncParts(stored);
    expect(surfaces(synced)[1]).toMatchObject({ orientation: 'west', insulation: { kind: 'thickness', thicknessMm: 80 } });

    const { stored: without, removed } = removeSurface(synced, 'gevel-1');
    expect(removed).toEqual(['gevel-1', 'gevel-1-deel-2']);
    expect(surfaces(without)).toEqual([]);
    expect(without.surfaceParts).toEqual({});
  });
});

describe('opaque parts in the facade card', () => {
  function Harness() {
    const { state, dispatch } = useEnergy();
    useEffect(() => { dispatch({ type: 'SET_BASISOPNAME', payload: withFacade() }); }, [dispatch]);
    return <>{state.project.basisopname && <BasisopnamePanel part="walls" />}
      <output data-testid="survey">{JSON.stringify(state.project.basisopname ?? null)}</output></>;
  }
  const stored = () => JSON.parse(screen.getByTestId('survey').textContent ?? '{}') as StoredSurvey;

  it('adds a part in the card, keeps it out of the surface list and moves it along with the orientation', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(await screen.findByRole('button', { name: 'Add opaque part' }));
    expect(screen.getAllByText(/^Opaque part 2/).length).toBeGreaterThan(0);
    expect(screen.getByText('2 opaque parts')).toBeInTheDocument();
    // One card: the part is shown inside its facade, not as a facade of its own.
    expect(document.querySelectorAll('.opname-card:not(.opname-card--loose)')).toHaveLength(1);
    const part = screen.getAllByText(/^Opaque part 2/)[0].closest('.opname-part') as HTMLElement;
    await user.type(within(part).getByRole('spinbutton', { name: /area/i }), '6');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Orientation' }), 'east');
    expect(surfaces(stored())[1]).toMatchObject({ grossAreaM2: 6, orientation: 'east' });
    await user.click(within(part).getByRole('button', { name: 'Remove opaque part' }));
    expect(surfaces(stored())).toHaveLength(1);
  }, 60000);
});
