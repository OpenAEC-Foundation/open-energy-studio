import { useEffect, useState } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { HeatPumpSizingCalculator } from '../components/HeatPumpSizingCalculator/HeatPumpSizingCalculator';
import { openPointsEmptyText } from '../components/shell/openPointsEmpty';
import type { KernelState } from '../context/KernelProvider';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { useEnergy } from '../context/EnergyContext';
import { exampleProject } from '../core/nta/ExampleProjects';
import i18next from '../i18n/i18n';
import { renderWithProviders } from './test-utils';

// Findings of the first-time adviser walkthrough of 10 October 2026.

const t = (key: string) => i18next.t(key);
const state = (phase: KernelState['phase'], settled: unknown = null) => ({ phase, settled } as unknown as KernelState);

describe('first-time adviser walkthrough', () => {
  afterEach(async () => { await i18next.changeLanguage('en'); });

  it('says "no open points" only once the kernel has answered', async () => {
    await i18next.changeLanguage('nl');
    expect(openPointsEmptyText(t, state('current', {} as ProjectPerformanceAssessment))).toBe('Geen open punten van de rekenkern.');
    expect(openPointsEmptyText(t, state('error'))).toBe('Rekenkern niet bereikbaar');
    expect(openPointsEmptyText(t, state('loading'))).toBe('Berekenen…');
    expect(openPointsEmptyText(t, state('idle'))).toBe('Nog geen antwoord van de rekenkern');
    expect(openPointsEmptyText(t, null)).toBe('Nog geen antwoord van de rekenkern');
  });

  it('shows the heat-pump sizing tool with a decimal comma in Dutch', async () => {
    await i18next.changeLanguage('nl');
    function Tool() {
      const { dispatch } = useEnergy();
      const [ready, setReady] = useState(false);
      useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: exampleProject('small_office') }); setReady(true); }, [dispatch]);
      return ready ? <HeatPumpSizingCalculator /> : null;
    }
    renderWithProviders(<Tool />);
    const text = (await screen.findAllByText(/kW/)).map((node) => node.textContent ?? '').join(' ');
    expect(text).toMatch(/\d,\d{2} kW/);
    expect(text).not.toMatch(/\d\.\d{2} kW/);
  });

  it('names the kernel "rekenkern" in the Dutch interface', async () => {
    await i18next.changeLanguage('nl');
    for (const key of ['kernel.subtitle', 'kernel.unavailable', 'kernel.version', 'status.kernelName', 'nta.performance.title', 'opname.intro']) {
      expect(i18next.t(key)).not.toMatch(/Rust-kern|Kernelversie/);
    }
    for (const key of ['preview.kernelEmpty', 'preview.kernelIncomplete', 'results.withheld.invalid', 'results.withheld.incomplete']) {
      expect(i18next.t(key)).not.toMatch(/NTA-paneel/);
    }
  });
});
