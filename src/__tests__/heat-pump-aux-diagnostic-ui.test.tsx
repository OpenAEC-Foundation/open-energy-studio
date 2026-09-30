import { afterEach, describe, expect, it, vi } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { HeatPumpAuxDiagnosticPanel } from '../components/HeatPumpAuxDiagnosticPanel/HeatPumpAuxDiagnosticPanel';
import type { INtaHeatPumpInput } from '../core/energy/types';
import { renderWithProviders, userEvent } from './test-utils';

const pump: INtaHeatPumpInput = {
  id: 'hp-measured', source: 'outdoor_air', sink: 'hydronic', drive: 'electric_compression',
  reversible: false, hybrid: false, booster: false,
  performanceEvidence: { kind: 'controlled_quality_declaration', reference: 'product schedule' },
};

afterEach(() => vi.unstubAllGlobals());

describe('draft heat-pump auxiliary UI', () => {
  it('requires every month, sends traceable measured input and keeps the result scoped', async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        status: 'input_valid', inputFingerprint: 'sha256:measured',
        finalEditionVerified: false, referenceVerified: false, bengCalculationAvailable: false,
        primaryElectricityFactorUsed: 1.45, usefulPumpFractionUsed: 0.5,
        derivedCoefficients: { aAnnualKwh: 87.6, bKw: 0.19, cDimensionless: 0.5,
          nominalElectricDriveKw: 2, sourceReference: 'meter; cycle' },
        auxiliary: { annualAuxiliaryElectricityKwh: 315.6,
          monthlyAuxiliaryElectricityKwh: Array.from({ length: 12 }, (_, index) => ({ month: index + 1, electricityKwh: 26.3 })) },
        issues: [],
      }),
    });
    vi.stubGlobal('fetch', fetchMock);
    const onSave = vi.fn();
    const view = renderWithProviders(<HeatPumpAuxDiagnosticPanel pump={pump} onSave={onSave} />);

    await user.click(screen.getByRole('button', { name: 'Calculate draft diagnostic' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Complete every field');
    expect(fetchMock).not.toHaveBeenCalled();

    const fill = (label: string, value: string) => fireEvent.change(screen.getByLabelText(label), { target: { value } });
    fill('Electronics standby power (W)', '10');
    fill('Delivery pump during compressor operation (W)', '200');
    fill('Delivery pump during pre/post-run (W)', '90');
    fill('Pump pre-run time (s)', '300');
    fill('Pump post-run time (s)', '300');
    fill('Average compressor on-time per cycle (s)', '600');
    fill('Mean compressor modulation (0–1)', '0.5');
    fill('Nominal electric input power (kW)', '2');
    fill('Power measurement source', 'meter');
    fill('Cycle/modulation source', 'cycle');
    fill('Monthly generator electricity source', 'monthly meter');
    for (let month = 1; month <= 12; month += 1) fill(`Month ${month}`, '100');
    await user.click(screen.getByRole('button', { name: 'Calculate draft diagnostic' }));

    expect(await screen.findByText('315.60 kWh/year')).toBeInTheDocument();
    expect(screen.getByText(/No forfait, annual heat pump performance, BENG result or energy label/)).toBeInTheDocument();
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toBe('/api/v1/nta8800/heat-pumps/heating-aux-measured-draft/diagnose');
    const input = JSON.parse(options.body).input;
    expect(input.generatorId).toBe('hp-measured');
    expect(input.generatorSourceReference).toBe('product schedule');
    expect(input.measurements.averageCompressorOnSeconds).toBe(600);
    expect(input.months).toHaveLength(12);
    expect(input.months[11]).toEqual({ month: 12, generatorInputElectricityKwh: 100 });

    await user.click(screen.getByRole('button', { name: 'Save measured inputs' }));
    expect(onSave).toHaveBeenCalledWith(input);
    fill('Electronics standby power (W)', '11');
    expect(screen.queryByText('315.60 kWh/year')).not.toBeInTheDocument();
    view.unmount();
    renderWithProviders(<HeatPumpAuxDiagnosticPanel pump={{ ...pump, heatingAuxMeasuredDraft: input }} />);
    expect(screen.getByLabelText('Electronics standby power (W)')).toHaveValue(10);
    expect(screen.getByLabelText('Month 12')).toHaveValue(100);
    expect(screen.getByText(/Measured inputs saved in this project/)).toBeInTheDocument();

  });
});
