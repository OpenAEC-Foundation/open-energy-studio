import { useEffect } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { StatusBar } from '../components/StatusBar/StatusBar';
import { KernelProvider } from '../context/KernelProvider';
import { useEnergy } from '../context/EnergyContext';
import type { NtaCalculationInput } from '../core/nta/KernelClient';
import { renderWithProviders } from './test-utils';

// BRL 9501 §5.2: the status bar names the rekenkern version and the NTA 8800 edition at every width.

function Bar({ edition, withKernel }: { edition?: '2024'; withKernel?: boolean }) {
  const { state, dispatch } = useEnergy();
  useEffect(() => {
    if (edition) dispatch({ type: 'SET_NTA_CALCULATION', payload: { normVersion: edition } as NtaCalculationInput });
  }, [dispatch, edition]);
  return withKernel ? <KernelProvider project={state.project}><StatusBar /></KernelProvider> : <StatusBar />;
}

const versionChip = () => document.querySelector('.status-version') as HTMLElement;

describe('status bar kernel version and edition', () => {
  afterEach(() => { vi.unstubAllGlobals(); vi.unstubAllEnvs(); });

  it('names the designated edition before any kernel answer, in a long and a compact form', () => {
    renderWithProviders(<Bar />);
    const chip = versionChip();
    expect(chip.querySelector('.status-version-long')).toHaveTextContent('NTA 8800:2025+C1:2026 (designated)');
    expect(chip.querySelector('.status-version-short')).toHaveTextContent('NTA 8800 2025+C1');
    expect(chip).toHaveAttribute('aria-label', 'Method NTA 8800:2025+C1:2026 (designated) (no kernel answer yet)');
    // Not hidden at narrow widths like the optional items.
    expect(chip).not.toHaveClass('status-optional');
  });

  it('names an older edition of the project and keeps the not-for-registration marker', async () => {
    renderWithProviders(<Bar edition="2024" />);
    await waitFor(() => expect(versionChip().querySelector('.status-version-short')).toHaveTextContent('NTA 8800 2024'));
    expect(screen.getByText('NTA 8800:2024 with INT-V1:2024 — not for registration')).toBeInTheDocument();
  });

  it('shows the kernel version and the edition of the kernel answer', async () => {
    vi.stubEnv('DEV', true);
    vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({
      status: 'incomplete', normVersion: '2025+C1', targetNormVersion: 'NTA 8800:2025+C1:2026',
      kernelVersion: '9.8.7', registrationEligible: true, attestStatus: 'unattested', gaps: [], warnings: [],
      derivedInput: null, geometry: null, performance: null,
    }))));
    renderWithProviders(<Bar withKernel />);
    await waitFor(() => expect(versionChip()).toHaveTextContent('Kernel 9.8.7'), { timeout: 5000 });
    expect(versionChip().querySelector('.status-version-long')).toHaveTextContent('NTA 8800:2025+C1:2026');
    expect(versionChip()).toHaveAttribute('aria-label', 'Calculation kernel 9.8.7 · method NTA 8800:2025+C1:2026');
  });
});
