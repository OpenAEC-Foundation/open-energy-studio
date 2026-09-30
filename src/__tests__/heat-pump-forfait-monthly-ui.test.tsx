import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@testing-library/react';
import { HeatPumpForfaitMonthlyPanel } from '../components/HeatPumpForfaitMonthlyPanel/HeatPumpForfaitMonthlyPanel';
import type { INtaHeatPumpInput } from '../core/energy/types';
import { renderWithProviders, userEvent } from './test-utils';

const pump: INtaHeatPumpInput = {
  id: 'hp-monthly', source: 'district_water', sink: 'hydronic', drive: 'electric_compression',
  reversible: false, hybrid: false, booster: false,
  performanceEvidence: { kind: 'normative_default', reference: null },
  forfaitHeatPumpDraft: {
    generatorId: 'hp-monthly', classificationSourceReference: 'source design',
    scope: 'residential_at_most25_kw', source: 'collective15_to20_c', sink: 'hydronic',
    designSupplyTemperatureC: 35, sourceCorrectionFactor: null, sourceCorrectionReference: null,
    sourceTemperatureC: 15, sourceTemperatureEvidenceReference: 'source meter',
  },
};

afterEach(() => vi.unstubAllGlobals());

describe('draft monthly heat-pump input UI', () => {
  it('requires complete supplied months and displays only a draft monthly diagnostic', async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn().mockResolvedValue({ json: async () => ({
      status: 'diagnostic_valid', correctedCop: 4.8, collectiveSourceCorrectionFactor: 0.022,
      monthly: Array.from({ length: 12 }, (_, index) => ({ month: index + 1,
        collectiveSourceHeatKwh: 1000 * (1 - 1 / 4.8),
        generatorInputElectricityKwh: 1000 / 4.8 - 1000 * (1 - 1 / 4.8) * 0.022 })),
      collectiveSourceHeatDerived: true,
      inputFingerprint: 'sha256:monthly', issues: [],
      bengCalculationAvailable: false, annualPerformanceAvailable: false,
    }) });
    vi.stubGlobal('fetch', fetchMock);
    renderWithProviders(<HeatPumpForfaitMonthlyPanel pump={pump} />);
    await user.click(screen.getByRole('button', { name: 'Calculate draft monthly values' }));
    expect(fetchMock).not.toHaveBeenCalled();
    expect(screen.getByRole('alert')).toHaveTextContent('all twelve months');
    fireEvent.change(screen.getByLabelText('Monthly generator heat evidence'), { target: { value: 'heat meter' } });
    fireEvent.change(screen.getByLabelText('Source system evidence'), { target: { value: 'source design' } });
    for (let month = 1; month <= 12; month += 1) {
      fireEvent.change(screen.getByLabelText(`Generator heat (kWh) ${month}`), { target: { value: '1000' } });
    }
    await user.click(screen.getByRole('button', { name: 'Calculate draft monthly values' }));
    await waitFor(() => expect(fetchMock).toHaveBeenCalledOnce());
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/v1/nta8800/heat-pumps/forfait-monthly-draft/diagnose');
    const input = JSON.parse(options.body).input;
    expect(input.generatorOutputKwh).toHaveLength(12);
    expect(input.sourceSystemReference).toBe('source design');
    expect(input.collectiveSourceHeatKwh).toBeUndefined();
    expect(input.sourceSystem).toBe('collective_groundwater_surface_or_at_least15_c');
    expect(screen.getByRole('status')).toHaveTextContent('791.67');
    expect(screen.getByRole('status')).toHaveTextContent('190.92');
    expect(screen.getByRole('status')).toHaveTextContent('BENG and energy label are unavailable');
    fireEvent.change(screen.getByLabelText('Generator heat (kWh) 1'), { target: { value: '900' } });
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});
