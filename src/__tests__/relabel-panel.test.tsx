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
});
