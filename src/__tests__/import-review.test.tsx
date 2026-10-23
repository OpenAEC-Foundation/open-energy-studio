/**
 * Issue #8 (BRL 9501 §4.3.1, p. 8): a registration may not rest on data
 * another program read in. Per import the adviser confirms on the
 * registration page that the data was checked; the confirmation is stored in
 * `importLog[i].reviewedAt`, which the kernel requires for a complete dossier.
 */
import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { ImportReview } from '../components/shell/pages/RegistrationPage';
import { withImportRecord } from '../core/io/importLog';
import { routeForPath } from '../core/nta/gapRoutes';
import { renderWithProviders, userEvent } from './test-utils';

function Harness() {
  const { state, dispatch } = useEnergy();
  useEffect(() => {
    const project = withImportRecord(
      withImportRecord(createDefaultProject(), 'UNIEC3', 'woning.uniec3', '2026-10-20T09:00:00Z'),
      'VABI', 'kantoor.xml', '2026-10-21T09:00:00Z',
    );
    dispatch({ type: 'SET_PROJECT', payload: project });
  }, [dispatch]);
  return <>
    <ImportReview />
    <output data-testid="log">{JSON.stringify(state.project.importLog ?? [])}</output>
  </>;
}

const log = () => JSON.parse(screen.getByTestId('log').textContent ?? '[]') as { tool: string; reviewedAt?: string }[];

describe('confirming imported data before registration', () => {
  it('stores and withdraws the confirmation per import', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const confirm = await screen.findAllByRole('button', { name: 'Data read in checked' });
    expect(confirm).toHaveLength(2);
    await user.click(confirm[1]);
    expect(log()[0].reviewedAt).toBeUndefined();
    expect(log()[1].reviewedAt).toMatch(/^\d{4}-\d{2}-\d{2}T/);
    expect(screen.getAllByRole('button', { name: 'Data read in checked' })).toHaveLength(1);
    await user.click(screen.getByRole('button', { name: 'Withdraw confirmation' }));
    expect(log().every((record) => record.reviewedAt === undefined)).toBe(true);
    expect(log().map((record) => record.tool)).toEqual(['UNIEC3', 'VABI']);
  });

  it('shows nothing without imports', () => {
    renderWithProviders(<ImportReview />);
    expect(screen.queryByRole('button', { name: 'Data read in checked' })).toBeNull();
  });

  it('sends the kernel message to the registration step', () => {
    expect(routeForPath('importLog[0].reviewedAt').step).toBe('registration');
  });
});
