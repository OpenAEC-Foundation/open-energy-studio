import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import { MaatwerkadviesPanel } from '../components/MaatwerkadviesPanel/MaatwerkadviesPanel';
import { NtaLightingSection } from '../components/NtaPerformancePanel/NtaExtraSections';
import { renderWithProviders, userEvent } from './test-utils';

const dwelling = JSON.parse(readFileSync(resolve(process.cwd(), 'training-data/nta8800-example-terraced-dwelling.json'), 'utf8')) as IProject;

function Editor() {
  const { state, dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: structuredClone(dwelling) }); }, [dispatch]);
  if (state.project.id !== dwelling.id) return null;
  return <MaatwerkadviesPanel />;
}

describe('maatwerkadvies editor review fixes (merge 02fafc5)', () => {
  it('warns about a zero investment and blocks an incomplete PV measure', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Editor />);
    await user.click(await screen.findByRole('button', { name: 'Start tailored advice' }));
    await user.click(screen.getByRole('button', { name: 'Add measure' }));
    expect(screen.getByTestId('mwa-investment-zero-m1')).toHaveTextContent('The investment is 0 €');
    await user.selectOptions(screen.getByLabelText('Measure type'), 'pv');
    expect(screen.getByTestId('mwa-template-problems-m1')).toHaveTextContent('Enter the peak power of every PV system.');
    await user.clear(screen.getByLabelText('Investment [€]'));
    await user.type(screen.getByLabelText('Investment [€]'), '5000');
    expect(screen.queryByTestId('mwa-investment-zero-m1')).toBeNull();
  }, 60000);

  it('disables daylight under forfait power and notes central on-switching (14.16/14.17, 14.24)', () => {
    const draft = {
      lighting: [{
        zoneId: 'z1', functions: [{ function: 'office', areaM2: 100 }], sourceReference: '',
        lightingZones: [{
          id: 'lz1', areaM2: 100, power: { method: 'forfait', ledFrom2017: true }, parasitic: { method: 'forfait' },
          occupancy: { control: 'auto_on_auto_off', centralOnControl: true }, daylight: { method: 'forfait', daylightControl: true },
        }],
      }],
    };
    renderWithProviders(<NtaLightingSection draft={draft} change={() => {}} project={dwelling} />);
    expect(screen.getByLabelText('Daylight')).toBeDisabled();
    expect(screen.getByText(/F_D = 1 \(14\.24, p\. 667\)/)).toBeInTheDocument();
    expect(screen.getByText(/F_o;D = F_o;N = 1 \(14\.16\/14\.17, p\. 664\)/)).toBeInTheDocument();
  });
});
