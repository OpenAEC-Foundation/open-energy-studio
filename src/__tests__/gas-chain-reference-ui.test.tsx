import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { GasChainReferencePanel } from '../components/GasChainReferencePanel/GasChainReferencePanel';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => vi.unstubAllGlobals());

describe('gas chain reference comparison UI', () => {
  it('loads the synthetic 25-value case, compares via Rust API, and keeps verification unavailable', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true, text: async () => JSON.stringify({
      status: 'compared_pass', caseId: 'synthetic-gas-chain-2026-draft',
      referenceVerified: false, carrierAllocationAvailable: false, bengCalculationAvailable: false,
      metrics: [{ metric: 'equation962_unallocated_input_term', month: 1, expectedKwh: 625,
        actualKwh: 625, absoluteDifferenceKwh: 0, absoluteToleranceKwh: 1e-9, withinTolerance: true }],
      inputFingerprint: 'sha256:input', caseFingerprint: 'sha256:case', issues: [],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<GasChainReferencePanel />);
    await user.click(screen.getByText('Gas heat pump reference comparison'));
    expect(screen.getByRole('button', { name: 'Compare draft values' })).toBeDisabled();
    await user.click(screen.getByRole('button', { name: 'Load synthetic example' }));
    await user.click(screen.getByRole('button', { name: 'Compare draft values' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/v1/nta8800/reference/gas-chain-diagnostic/compare');
    const submitted = JSON.parse(options.body).case;
    expect(submitted.expected).toHaveLength(25);
    expect(submitted.source.documentId).toContain('not ISSO 54');
    expect(screen.getByRole('status')).toHaveTextContent('Submitted values within tolerance');
    expect(screen.getByRole('status')).toHaveTextContent('Reference not independently verified');
    expect(screen.getByRole('status')).toHaveTextContent('625.000');
    const provenance = screen.getByLabelText('Reference case provenance');
    expect(provenance).toHaveTextContent('Open Energy Studio internal arithmetic');
    expect(provenance).toHaveTextContent('synthetic gas heat-pump hand calculation; not ISSO 54');
    expect(provenance).toHaveTextContent('none; external verification pending');
    fireEvent.change(screen.getByLabelText('Case JSON'), { target: { value: '{invalid' } });
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Reference case provenance')).not.toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Compare draft values' }));
    expect(screen.getByRole('alert')).toHaveTextContent('SyntaxError');
    expect(fetchMock).toHaveBeenCalledOnce();
  });

  it('shows incomplete-case issues without presenting a comparison table', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: false, status: 422, text: async () => JSON.stringify({
      status: 'invalid_case', caseId: 'incomplete', referenceVerified: false,
      carrierAllocationAvailable: false, bengCalculationAvailable: false,
      metrics: [], inputFingerprint: 'sha256:input', caseFingerprint: 'sha256:case',
      issues: [{ path: 'expected.annualEquipmentAuxiliaryElectricity', code: 'metric_required' }],
    }) });
    vi.stubGlobal('fetch', fetchMock);
    const user = userEvent.setup();
    renderWithProviders(<GasChainReferencePanel />);
    await user.click(screen.getByText('Gas heat pump reference comparison'));
    fireEvent.change(screen.getByLabelText('Case JSON'), { target: { value: '{"caseId":"incomplete"}' } });
    await user.click(screen.getByRole('button', { name: 'Compare draft values' }));
    await waitFor(() => expect(screen.getByRole('status')).toHaveTextContent('Case incomplete or invalid'));
    expect(screen.getByRole('status')).toHaveTextContent('metric_required');
    expect(screen.queryByLabelText('Reference case provenance')).not.toBeInTheDocument();
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
  });

  it('imports a JSON file and replaces a stale comparison', async () => {
    const user = userEvent.setup();
    renderWithProviders(<GasChainReferencePanel />);
    await user.click(screen.getByText('Gas heat pump reference comparison'));
    await user.click(screen.getByRole('button', { name: 'Load synthetic example' }));
    const file = new File(['{"caseId":"imported"}'], 'case.json', { type: 'application/json' });
    Object.defineProperty(file, 'text', { value: vi.fn().mockResolvedValue('{"caseId":"imported"}') });
    await user.upload(screen.getByLabelText('Import JSON case'), file);
    await waitFor(() => expect(screen.getByLabelText('Case JSON')).toHaveValue('{"caseId":"imported"}'));
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });

  it('shows a backend shape error without claiming a comparison', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: false, status: 422, text: async () => 'Failed to deserialize JSON body: missing field expected',
    }));
    const user = userEvent.setup();
    renderWithProviders(<GasChainReferencePanel />);
    await user.click(screen.getByText('Gas heat pump reference comparison'));
    fireEvent.change(screen.getByLabelText('Case JSON'), { target: { value: '{"caseId":"incomplete"}' } });
    await user.click(screen.getByRole('button', { name: 'Compare draft values' }));
    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('missing field expected'));
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});
