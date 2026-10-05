import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { ResultsView } from '../components/ResultsView/ResultsView';
import { ReportView } from '../components/ReportView/ReportView';
import { renderWithProviders } from './test-utils';

function ShowIndicativeResults() {
  const { state, dispatch } = useEnergy();
  useEffect(() => {
    dispatch({ type: 'SET_RESULT', payload: calculateBENGMonthly(state.project) });
  }, [dispatch]);
  return <><ResultsView /><ReportView /></>;
}

describe('unverified result presentation', () => {
  it('shows indicative status in results and report without pass/fail claims', async () => {
    renderWithProviders(<ShowIndicativeResults />);
    expect((await screen.findAllByText('Indicative')).length).toBeGreaterThanOrEqual(4);
    expect(screen.getAllByText(/Do not use these results as an official energy label/i).length).toBeGreaterThan(0);
    expect(screen.queryByText('Pass')).not.toBeInTheDocument();
    expect(screen.queryByText('Fail')).not.toBeInTheDocument();
  });
});
