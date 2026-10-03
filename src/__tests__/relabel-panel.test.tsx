import { afterEach, describe, expect, it, vi } from 'vitest';
import { act, screen, waitFor } from '@testing-library/react';
import { RelabelPanel } from '../components/MaatwerkadviesPanel/RelabelPanel';
import { useEnergy } from '../context/EnergyContext';
import { renderWithProviders, userEvent } from './test-utils';

function Harness() {
  const { dispatch } = useEnergy();
  return <><RelabelPanel />
    <button type="button" onClick={() => dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { name: 'Changed project' } })}>
      Change project
    </button></>;
}

afterEach(() => vi.unstubAllGlobals());

describe('relabel comparison', () => {
  it('drops a pending verdict when the current project changes and accepts a fresh comparison', async () => {
    const verdict = { allowed: true, needsReview: false, changes: [], scheme: 'w', source: 'BRL 9500' };
    let resolveFirst!: (value: { ok: boolean; json: () => Promise<unknown> }) => void;
    const fetchMock = vi.fn()
      .mockImplementationOnce(() => new Promise((resolve) => { resolveFirst = resolve; }))
      .mockResolvedValueOnce({ ok: true, json: async () => verdict });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    const file = new File(['original'], 'original.oes', { type: 'application/json' });
    Object.defineProperty(file, 'text', { value: vi.fn().mockResolvedValue(JSON.stringify({
      type: 'open-energy-studio', version: '1.0', project: { id: 'original' },
    })) });
    await user.upload(screen.getByLabelText('Original project file:'), file);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    await user.click(screen.getByRole('button', { name: 'Change project' }));
    await act(async () => resolveFirst({ ok: true, json: async () => verdict }));
    expect(screen.queryByText('Relabelling allowed')).not.toBeInTheDocument();
    await user.upload(screen.getByLabelText('Original project file:'), file);
    await waitFor(() => expect(fetchMock).toHaveBeenCalledTimes(2));
    expect(await screen.findByText('Relabelling allowed')).toBeInTheDocument();
  });

  it('keeps the comparison with the registration and marks it out of date after an edit (Bijlage 3)', async () => {
    const verdict = {
      allowed: true, needsReview: false, scheme: 'w', source: 'BRL 9500',
      changes: [{ path: '/name', before: 'a', after: 'b', verdict: 'allowed', cluster: 'not classified' }],
    };
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: true, json: async () => verdict }));
    let registration: unknown;
    function Probe() {
      const { state } = useEnergy();
      registration = state.project.registration;
      return null;
    }
    const user = userEvent.setup();
    renderWithProviders(<><Harness /><Probe /></>);
    const file = new File(['original'], 'original.oes', { type: 'application/json' });
    Object.defineProperty(file, 'text', { value: vi.fn().mockResolvedValue(JSON.stringify({
      type: 'open-energy-studio', version: '1.0', project: { id: 'original' },
    })) });
    await user.upload(screen.getByLabelText('Original project file:'), file);
    expect(await screen.findByText('Relabelling allowed')).toBeInTheDocument();
    const stored = (registration as { relabelComparison?: { originalFileName: string; currentSha256?: string; assessment: unknown } })
      .relabelComparison;
    expect(stored?.originalFileName).toBe('original.oes');
    expect(stored?.currentSha256).toMatch(/^[0-9a-f]{64}$/);
    expect(stored?.assessment).toEqual(verdict);
    expect(screen.queryByTestId('relabel-outdated')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Change project' }));
    // Still shown after the edit, but out of date.
    expect(screen.getByText('Relabelling allowed')).toBeInTheDocument();
    expect(await screen.findByTestId('relabel-outdated')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Remove comparison' }));
    expect(screen.queryByText('Relabelling allowed')).not.toBeInTheDocument();
  }, 60000);
});
