/**
 * Table view of the facades (feedback 9 Oct 2026): a row per facade with its
 * opaque parts indented below, the openings in a table of their own, Enter to
 * the same column in the next row, and the choice remembered.
 */
import { useEffect } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { useEnergy } from '../context/EnergyContext';
import { BasisopnamePanel } from '../components/BasisopnamePanel/BasisopnamePanel';
import { surveyTemplate, type StoredSurvey } from '../core/nta/SurveyTemplates';
import { addPart } from '../core/survey/surfaceParts';

afterEach(() => localStorage.clear());

const facade = (id: string, orientation: string, grossAreaM2: number) => ({
  id, element: 'facade', boundary: { kind: 'outdoor' }, orientation, grossAreaM2, cavity: true, insulation: { kind: 'none_or_unknown' }, sourceReference: '',
});

function Harness() {
  const { state, dispatch } = useEnergy();
  useEffect(() => {
    const stored = surveyTemplate('residential');
    const envelope = stored.survey.envelope as Record<string, unknown[]>;
    envelope.surfaces = [facade('gevel-1', 'south', 24), facade('gevel-2', 'east', 38)];
    envelope.windows = [{ id: 'raam-1', surfaceId: 'gevel-1', areaM2: 4, glass: 'hr_plus_plus', frame: 'wood_or_plastic', sourceReference: '' }];
    dispatch({ type: 'SET_BASISOPNAME', payload: addPart(stored, 'gevel-1') });
  }, [dispatch]);
  return <>{state.project.basisopname && <BasisopnamePanel part="walls" />}
    <output data-testid="survey">{JSON.stringify(state.project.basisopname ?? null)}</output></>;
}
const surfaces = () => ((JSON.parse(screen.getByTestId('survey').textContent ?? '{}') as StoredSurvey).survey.envelope as { surfaces: Array<Record<string, unknown>> }).surfaces;

describe('envelope table', () => {
  it('shows a row per facade with its part below, edits in place and remembers the view', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(await screen.findByRole('button', { name: 'Table' }));
    expect(localStorage.getItem('oes.survey.envelopeView')).toBe('table');
    const [surfaceTable, openingTable] = screen.getAllByRole('table');
    const rows = within(surfaceTable).getAllByRole('row').slice(1, -1);
    expect(rows.map((row) => within(row).getByRole('rowheader').textContent)).toEqual(['Facade 1', '↳ Opaque part 2', 'Facade 2']);
    expect(within(openingTable).getAllByRole('row')).toHaveLength(3);

    // Enter goes down the column: from the area of facade 1 to the area of its part.
    const areaOf = (row: HTMLElement) => within(row).getByRole('spinbutton', { name: /area/i });
    await user.clear(areaOf(rows[0]));
    await user.type(areaOf(rows[0]), '30{Enter}');
    expect(areaOf(rows[1])).toHaveFocus();
    await user.type(areaOf(rows[1]), '5');
    expect(surfaces()[0].grossAreaM2).toBe(30);
    expect(surfaces()[2]).toMatchObject({ id: 'gevel-1-deel-2', grossAreaM2: 5 });
    // The part follows its facade.
    await user.selectOptions(within(rows[0]).getByRole('combobox', { name: 'Orientation' }), 'west');
    expect(surfaces()[2].orientation).toBe('west');
  }, 60000);
});
