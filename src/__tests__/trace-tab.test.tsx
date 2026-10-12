/**
 * Rekenspoor in the results (feedback 9 Oct 2026): the kernel trace opens from
 * the zone down to an element with its source; a search narrows it; a monthly
 * step opens the H_tr it uses.
 */
import { describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import { renderWithProviders, userEvent } from './test-utils';
import { TraceTab } from '../components/shell/pages/results/TraceTab';
import type { KernelTraceStep, ProjectPerformanceAssessment } from '../core/nta/KernelClient';

const element = (id: string, area: number, u: number): KernelTraceStep => ({
  key: `z1.transmission.hd.${id}`, quantity: id, formula: 'A·U', values: `${area} × ${u}`, result: area * u, unit: 'W/K', reference: '8.1',
  source: 'survey: Rc table 8.9',
});
const trace: KernelTraceStep[] = [{
  key: 'z1.transmission', quantity: 'Transmissie z1', formula: 'Σ Q_H;tr over de maanden', values: '690 + 545', result: 1235, unit: 'kWh', reference: '7.14',
  children: [
    {
      key: 'z1.transmission.htr', quantity: 'H_tr', formula: 'H_D + H_U + H_p', values: '54.85 + 3.6', result: 58.45, unit: 'W/K', reference: '7.16',
      children: [{
        key: 'z1.transmission.hd', quantity: 'H_D', formula: 'Σ A·U + Σ L·ψ + Σ χ', values: '26 × 0.365 + 22 × 0.365', result: 17.52, unit: 'W/K', reference: '8.1',
        children: [element('gevel-1', 26, 0.365), element('gevel-2', 22, 0.365)],
      }],
    },
    {
      key: 'z1.transmission.qhtr.1', quantity: 'Q_H;tr jan', formula: 'H_tr · (θ_int;calc;H − θ_e) · t / 1000 + Q_H;tr;g', values: '58.45 × (20 − 2.6) × 744 / 1000 + 0',
      result: 756.6, unit: 'kWh', reference: '7.14', uses: ['z1.transmission.htr'],
    },
  ],
}];
const assessment = { performance: { trace } } as unknown as ProjectPerformanceAssessment;

describe('rekenspoor tab', () => {
  it('opens from the zone to an element with its source, searches and opens a used step', async () => {
    const user = userEvent.setup();
    renderWithProviders(<TraceTab assessment={assessment} />);
    expect(screen.getByText('Transmissie z1')).toBeVisible();
    await user.click(screen.getByText('H_tr', { selector: '.trace-step__quantity' }));
    await user.click(screen.getByText('H_D', { selector: '.trace-step__quantity' }));
    const element1 = document.querySelector('[data-key="z1.transmission.hd.gevel-1"]') as HTMLElement;
    expect(within(element1).getByText(/Source: /)).toBeInTheDocument();
    // The search keeps the matching element and the steps leading to it.
    await user.type(screen.getByRole('searchbox'), 'gevel-2');
    expect(document.querySelector('[data-key="z1.transmission.hd.gevel-1"]')).toBeNull();
    expect(document.querySelector('[data-key="z1.transmission.hd.gevel-2"]')).not.toBeNull();
    await user.clear(screen.getByRole('searchbox'));
    // The monthly step opens the H_tr it uses.
    await user.click(screen.getByText('Q_H;tr jan'));
    await user.click(screen.getByRole('button', { name: 'Uses: H_tr' }));
    expect(document.querySelectorAll('[data-key="z1.transmission.htr"]')).toHaveLength(2);
    expect(screen.getByRole('button', { name: 'Export calculation trace (JSON)' })).toBeInTheDocument();
  });
});
