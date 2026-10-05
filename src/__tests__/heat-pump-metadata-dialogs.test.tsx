import { describe, expect, it } from 'vitest';
import { useState } from 'react';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { HeatingSystemDialog } from '../components/dialogs/HeatingSystemDialog/HeatingSystemDialog';
import { HotWaterSystemDialog } from '../components/dialogs/HotWaterSystemDialog/HotWaterSystemDialog';
import { renderWithProviders, userEvent } from './test-utils';

function HeatingEditor() {
  const { state } = useEnergy();
  const [open, setOpen] = useState(true);
  return <>{open && <HeatingSystemDialog editId="heat-1" onClose={() => setOpen(false)} />}
    <button type="button" onClick={() => setOpen(true)}>Reopen heating editor</button>
    <output data-testid="heating-source">{state.project.heatingSystems[0].ntaHeatPump?.source ?? 'none'}</output>
    <output data-testid="heating-reference">{state.project.heatingSystems[0].ntaHeatPump?.performanceEvidence.reference ?? 'none'}</output>
    <output data-testid="heating-auxiliaries">{state.project.heatingSystems[0].ntaHeatPump?.auxiliaryComponents?.length ?? 0}</output>
    <output data-testid="heating-links">{JSON.stringify(state.project.heatingSystems[0].ntaHeatPump?.systemLinks ?? [])}</output>
    <output data-testid="heating-limits">{JSON.stringify(state.project.heatingSystems[0].ntaHeatPump?.declaredOperatingLimits ?? null)}</output>
  </>;
}

function HotWaterEditor() {
  const { state } = useEnergy();
  const [open, setOpen] = useState(true);
  return <>{open && <HotWaterSystemDialog editId="hw-1" onClose={() => setOpen(false)} />}
    <button type="button" onClick={() => setOpen(true)}>Reopen hot water editor</button>
    <output data-testid="hot-water-sink">{state.project.hotWaterSystems[0].ntaHeatPump?.sink ?? 'none'}</output>
    <output data-testid="hot-water-auxiliaries">{state.project.hotWaterSystems[0].ntaHeatPump?.auxiliaryComponents?.length ?? 0}</output>
    <output data-testid="hot-water-links">{state.project.hotWaterSystems[0].ntaHeatPump?.systemLinks?.length ?? 0}</output>
    <output data-testid="hot-water-dhw-tests">{state.project.hotWaterSystems[0].ntaHeatPump?.dhwTestPoints?.length ?? 0}</output>
    <output data-testid="hot-water-source-flow">{state.project.hotWaterSystems[0].ntaHeatPump?.dhwTestPoints?.[0]?.sourceAirFlowM3PerHour ?? ''}</output>
  </>;
}

async function addAuxiliary(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Add component' }));
  await user.type(screen.getByLabelText('Rated power (W)'), '80');
  await user.type(screen.getByLabelText('Source or product reference'), 'Product sheet 1');
  await user.click(screen.getByRole('button', { name: 'Save component' }));
}

describe('heat pump classification dialogs', () => {
  it('retains declared hybrid shutoff limits when a heating installation is reopened', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HeatingEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'controlled_quality_declaration');
    await user.type(screen.getByLabelText('Quality declaration reference'), 'BCRG 20240234GK');
    await user.click(screen.getByRole('button', { name: 'Add limits' }));
    await user.type(screen.getByLabelText('Minimum operating COP'), '2');
    await user.type(screen.getByLabelText('Maximum supply temperature (°C)'), '55');
    await user.type(screen.getByLabelText('Declaration norm edition'), 'NTA 8800:2024');
    await user.type(screen.getByLabelText('Limit source reference'), 'BCRG p2');
    await user.click(screen.getByRole('button', { name: 'Save limits' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    await user.click(screen.getByRole('button', { name: 'Reopen heating editor' }));
    expect(screen.getByText(/COP ≥ 2/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('heating-limits').textContent ?? 'null')).toMatchObject({
      minimumOperatingCop: 2, maximumSupplyTemperatureC: 55, sourceReference: 'BCRG p2',
    });
  }, 60000);
  it('saves a traceable heating heat pump classification', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HeatingEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await user.selectOptions(screen.getByLabelText('Heat source'), 'exhaust_air');
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'controlled_quality_declaration');
    await user.type(screen.getByLabelText('Quality declaration reference'), 'BCRG-test-123');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('heating-source')).toHaveTextContent('exhaust_air');
    expect(screen.getByTestId('heating-reference')).toHaveTextContent('BCRG-test-123');
  });

  it('keeps the heating editor open when a drive change invalidates a saved point', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HeatingEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await user.selectOptions(screen.getByLabelText('Drive'), 'gas_engine');
    await user.click(screen.getByRole('button', { name: 'Add operating point' }));
    await user.type(screen.getByLabelText('Source temperature (°C)'), '7');
    await user.type(screen.getByLabelText('Sink temperature (°C)'), '35');
    await user.type(screen.getByLabelText('Useful capacity (kW)'), '5');
    await user.type(screen.getByLabelText('Input power (kW)'), '1.5');
    await user.type(screen.getByLabelText('Test or declaration reference'), 'Report 5');
    await user.click(screen.getByRole('button', { name: 'Save operating point' }));
    await user.selectOptions(screen.getByLabelText('Drive'), 'electric_compression');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByRole('alert')).toHaveTextContent('electric compressor');
    expect(screen.getByLabelText('Drive')).toBeInTheDocument();
    expect(screen.getByTestId('heating-source')).toHaveTextContent('none');
  });

  it('prevents two operating points with the same conditions', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HeatingEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    for (const report of ['Report 1', 'Report 2']) {
      await user.click(screen.getByRole('button', { name: 'Add operating point' }));
      await user.type(screen.getByLabelText('Source temperature (°C)'), '7');
      await user.type(screen.getByLabelText('Sink temperature (°C)'), '35');
      await user.type(screen.getByLabelText('Useful capacity (kW)'), '5');
      await user.type(screen.getByLabelText('Input power (kW)'), '2');
      await user.type(screen.getByLabelText('Test or declaration reference'), report);
      await user.click(screen.getByRole('button', { name: 'Save operating point' }));
    }
    expect(screen.getByRole('alert')).toHaveTextContent('already exists');
    expect(screen.getByText(/5 kW \/ 2 kW · Report 1/)).toBeInTheDocument();
  });

  it('records an exhaust-air heat pump source ventilation link', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HeatingEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await user.selectOptions(screen.getByLabelText('Heat source'), 'exhaust_air');
    await user.click(screen.getByRole('button', { name: 'Add link' }));
    await user.selectOptions(screen.getByLabelText('Role'), 'source_ventilation');
    expect(screen.getByLabelText('Target type')).toHaveValue('ventilation_system');
    expect(screen.getByLabelText('Linked system')).toHaveValue('vent-1');
    await user.type(screen.getByLabelText('Link source reference'), 'Ventilation scheme 1');
    await user.click(screen.getByRole('button', { name: 'Save link' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('heating-links').textContent ?? '[]')).toMatchObject([{
      role: 'source_ventilation', targetKind: 'ventilation_system', targetId: 'vent-1',
      evidenceReference: 'Ventilation scheme 1',
    }]);
  });

  it('saves a tapwater heat pump with domestic hot water sink', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HotWaterEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('hot-water-sink')).toHaveTextContent('domestic_hot_water');
  });

  it('keeps heating auxiliary components when an existing installation is reopened and saved', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HeatingEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await addAuxiliary(user);
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('heating-auxiliaries')).toHaveTextContent('1');
    await user.click(screen.getByRole('button', { name: 'Reopen heating editor' }));
    expect(screen.getByText(/80 W · Product sheet 1/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('heating-auxiliaries')).toHaveTextContent('1');
  });

  it('keeps hot water auxiliary components when an existing installation is reopened and saved', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HotWaterEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await addAuxiliary(user);
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('hot-water-auxiliaries')).toHaveTextContent('1');
    await user.click(screen.getByRole('button', { name: 'Reopen hot water editor' }));
    expect(screen.getByText(/80 W · Product sheet 1/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('hot-water-auxiliaries')).toHaveTextContent('1');
  });

  it('keeps a declared DHW test profile when an installation is reopened', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HotWaterEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await user.selectOptions(screen.getByLabelText('Performance evidence'), 'controlled_quality_declaration');
    await user.type(screen.getByLabelText('Quality declaration reference'), 'BCRG 20240123GK');
    await user.click(screen.getByRole('button', { name: 'Add test profile' }));
    for (const [label, value] of [
      ['Tap profile', 'M'], ['Useful test energy (kWh/day)', '5.865'],
      ['Input test energy (kWh/day)', '2.52'], ['Nominal capacity (kW)', '4.71'],
      ['Supplied practice factor', '0.9'], ['Test setpoint (°C)', '49.1'],
      ['Design setpoint (°C)', '55'], ['Declaration norm edition', 'NTA 8800:2020'],
      ['Declaration and table/page', 'BCRG 20240123GK p3'],
      ['Source air flow (m³/hour, if declared)', '159'],
      ['Source air dry bulb (°C, if declared)', '20'],
      ['Source air wet bulb (°C, if declared)', '12'],
    ]) await user.type(screen.getByLabelText(label), value);
    await user.click(screen.getByRole('button', { name: 'Save test profile' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('hot-water-dhw-tests')).toHaveTextContent('1');
    expect(screen.getByTestId('hot-water-source-flow')).toHaveTextContent('159');
    await user.click(screen.getByRole('button', { name: 'Reopen hot water editor' }));
    expect(screen.getByText(/5.865\/2.52 kWh\/d/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('hot-water-dhw-tests')).toHaveTextContent('1');
    expect(screen.getByTestId('hot-water-source-flow')).toHaveTextContent('159');
  }, 60000);

  it('keeps an equipment link when the hot water installation is reopened and saved', async () => {
    const user = userEvent.setup();
    renderWithProviders(<HotWaterEditor />);
    await user.click(screen.getByLabelText('Add heat pump details for Rust input validation'));
    await user.click(screen.getByRole('button', { name: 'Add link' }));
    await user.selectOptions(screen.getByLabelText('Linked system'), 'heat-1');
    await user.type(screen.getByLabelText('Link source reference'), 'Scheme 4');
    await user.click(screen.getByRole('button', { name: 'Save link' }));
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('hot-water-links')).toHaveTextContent('1');
    await user.click(screen.getByRole('button', { name: 'Reopen hot water editor' }));
    expect(screen.getByText(/heat-1 · Scheme 4/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByTestId('hot-water-links')).toHaveTextContent('1');
  });
});
