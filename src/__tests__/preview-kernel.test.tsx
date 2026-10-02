import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { PreviewPanel } from '../components/PreviewPanel/PreviewPanel';
import { summarizeForPreview } from '../core/nta/PreviewSummary';
import type { NtaCalculationInput, ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  vi.unstubAllGlobals();
});

const balance = (needKwh: number) => ({ needKwh });
const zone = (heating: number, cooling: number) => ({
  monthly: Array.from({ length: 12 }, () => ({ heating: balance(heating), cooling: balance(cooling) })),
});

function assessment(): ProjectPerformanceAssessment {
  return {
    status: 'calculated_unverified',
    gaps: [],
    geometry: null,
    derivedInput: null,
    performance: {
      issues: [],
      needIndicatorKwhPerM2Year: 61.23,
      primaryFossilIndicatorKwhPerM2Year: 22.5,
      renewableSharePercent: 64.1,
      indicativeLabelClass: 'A++',
      bblCheck: { limits: { energyNeedMaxKwhPerM2: 65, primaryFossilMaxKwhPerM2: 30, renewableShareMinPercent: 50 } },
      tojuliMaxK: 0.84,
      tojuliMeetsBblLimit: true,
      zebPrimaryTotalIndicatorKwhPerM2: 48.37,
      annualFinalEnergyKwh: 9876,
      co2KgPerM2: 7.4,
      spaceHeating: { demand: zone(100, 5), additionalZoneDemands: [zone(50, 0)] },
    },
  } as unknown as ProjectPerformanceAssessment;
}

describe('preview summary from the kernel', () => {
  it('takes the indicators, limits and monthly need from the kernel result', () => {
    const summary = summarizeForPreview(assessment());
    expect(summary.beng1).toBe(61.23);
    expect(summary.beng2Limit).toBe(30);
    expect(summary.beng3Limit).toBe(50);
    expect(summary.labelClass).toBe('A++');
    expect(summary.zebIndicator).toBe(48.37);
    // Σ over the zones.
    expect(summary.monthlyHeatingKwh[0]).toBe(150);
    expect(summary.monthlyCoolingKwh[6]).toBe(5);
  });

  it('shows no indicators for an incomplete result', () => {
    const incomplete = { ...assessment(), status: 'incomplete', gaps: [{ code: 'x', path: 'y' }] } as ProjectPerformanceAssessment;
    const summary = summarizeForPreview(incomplete);
    expect(summary.beng1).toBeNull();
    expect(summary.gapCount).toBe(1);
    expect(summary.monthlyHeatingKwh.every((value) => value === 0)).toBe(true);
  });
});

function SetNtaBlock() {
  const { dispatch } = useEnergy();
  return (
    <button onClick={() => dispatch({ type: 'SET_NTA_CALCULATION', payload: { calculationScope: 'residential' } as unknown as NtaCalculationInput })}>
      Set NTA block
    </button>
  );
}

describe('preview panel', () => {
  it('shows no legacy BENG figures without an NTA block', () => {
    renderWithProviders(<PreviewPanel />);
    expect(screen.getByText(/No NTA 8800 calculation yet/)).toBeInTheDocument();
    expect(screen.queryByText(/BENG 1/)).not.toBeInTheDocument();
    expect(document.querySelector('.preview-beng-card')).not.toBeInTheDocument();
  });

  it('shows the kernel indicators once the NTA block exists', async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, json: async () => assessment() });
    vi.stubGlobal('fetch', fetchMock);
    renderWithProviders(<><SetNtaBlock /><PreviewPanel /></>);
    await user.click(screen.getByRole('button', { name: 'Set NTA block' }));
    await waitFor(() => expect(screen.getByText('A++')).toBeInTheDocument(), { timeout: 5000 });
    expect(fetchMock).toHaveBeenCalledWith('/api/v1/nta8800/project/performance', expect.anything());
    expect(screen.getByText(/61\.2/)).toBeInTheDocument();
    expect(screen.getByText(/48\.37 kWh\/m²/)).toBeInTheDocument();
    expect(screen.getByText(/0\.84 K/)).toBeInTheDocument();
    expect(screen.getAllByText('Unverified').length).toBeGreaterThan(0);
  });
});
