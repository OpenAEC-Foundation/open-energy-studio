import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { calculateBENG } from '../core/energy/BENGCalculator';
import { PreviewPanel } from '../components/PreviewPanel/PreviewPanel';
import { renderWithProviders, userEvent } from './test-utils';
import type { INtaHeatPumpInput } from '../core/energy/types';

const classifiedHeatPump: INtaHeatPumpInput = {
  id: 'heat-1', source: 'outdoor_air', sink: 'hydronic',
  drive: 'electric_compression', reversible: false, hybrid: false, booster: false,
  performanceEvidence: { kind: 'normative_default', reference: null },
  performancePoints: [{
    id: 'p1', service: 'space_heating', sourceTemperatureC: 7, sinkTemperatureC: 35,
    usefulCapacityKw: 5, inputPowerKw: 1.5, inputEnergyCarrier: 'electricity',
    testReference: 'lab-1',
  }],
};

function InvalidCopButton() {
  const { state, dispatch } = useEnergy();
  return <button onClick={() => dispatch({
    type: 'UPDATE_HEATING_SYSTEM',
    payload: { id: state.project.heatingSystems[0].id, data: { cop: 0 } },
  })}>Invalid COP</button>;
}

function DeclaredPointButton() {
  const { state, dispatch } = useEnergy();
  return <button onClick={() => dispatch({
    type: 'UPDATE_HEATING_SYSTEM',
    payload: { id: state.project.heatingSystems[0].id, data: { ntaHeatPump: classifiedHeatPump } },
  })}>Add declared point</button>;
}

function DeclaredAuxiliaryButton() {
  const { state, dispatch } = useEnergy();
  return <button onClick={() => dispatch({
    type: 'UPDATE_HEATING_SYSTEM',
    payload: { id: state.project.heatingSystems[0].id, data: { ntaHeatPump: {
      ...classifiedHeatPump, performancePoints: [], auxiliaryComponents: [{
        id: 'source-pump', kind: 'source_pump', service: 'space_heating',
        nominalPowerW: 80, energyCarrier: 'electricity',
        measurementBoundary: 'additional', evidenceReference: 'datasheet-1',
      }],
    } } },
  })}>Add auxiliary</button>;
}

describe('invalid legacy heat pump inputs', () => {
  it('does not return an indicative result for a nonpositive COP', () => {
    const project = createDefaultProject();
    project.heatingSystems[0].cop = 0;
    expect(() => calculateBENGMonthly(project)).toThrow(/COP must be finite and greater than zero/);
    expect(() => calculateBENG(project)).toThrow(/COP must be finite and greater than zero/);
  });

  it('does not return an indicative result for a coverage fraction above one', () => {
    const project = createDefaultProject();
    project.heatingSystems[0].coverageFraction = 1.2;
    expect(() => calculateBENGMonthly(project)).toThrow(/coverage fraction must be between zero and one/);
    expect(() => calculateBENG(project)).toThrow(/coverage fraction must be between zero and one/);
  });

  it('shows the invalid COP in the preview without rendering a label', async () => {
    const user = userEvent.setup();
    renderWithProviders(<><InvalidCopButton /><PreviewPanel /></>);
    await user.click(screen.getByRole('button', { name: 'Invalid COP' }));
    expect(screen.getByRole('alert')).toHaveTextContent(/COP greater than zero/);
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
  });

  it('does not silently ignore declared operating points in either calculator or preview', async () => {
    const project = createDefaultProject();
    project.heatingSystems[0].ntaHeatPump = classifiedHeatPump;
    expect(() => calculateBENGMonthly(project)).toThrow(/heat pump details are not included/);
    expect(() => calculateBENG(project)).toThrow(/heat pump details are not included/);

    const user = userEvent.setup();
    renderWithProviders(<><DeclaredPointButton /><PreviewPanel /></>);
    await user.click(screen.getByRole('button', { name: 'Add declared point' }));
    expect(screen.getByRole('alert')).toHaveTextContent(/heat pump details.*cannot include/i);
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
  });

  it('does not silently ignore declared auxiliary power', async () => {
    const project = createDefaultProject();
    project.heatingSystems[0].ntaHeatPump = {
      ...classifiedHeatPump, performancePoints: [], auxiliaryComponents: [{
        id: 'source-pump', kind: 'source_pump', service: 'space_heating',
        nominalPowerW: 80, energyCarrier: 'electricity',
        measurementBoundary: 'additional', evidenceReference: 'datasheet-1',
      }],
    };
    expect(() => calculateBENGMonthly(project)).toThrow(/heat pump details are not included/);
    expect(() => calculateBENG(project)).toThrow(/heat pump details are not included/);

    const user = userEvent.setup();
    renderWithProviders(<><DeclaredAuxiliaryButton /><PreviewPanel /></>);
    await user.click(screen.getByRole('button', { name: 'Add auxiliary' }));
    expect(screen.getByRole('alert')).toHaveTextContent(/heat pump details.*cannot include/i);
    expect(document.querySelector('.preview-energy-label')).not.toBeInTheDocument();
  });

  it('does not silently ignore an embedded heat pump system link', () => {
    const project = createDefaultProject();
    project.heatingSystems[0].ntaHeatPump = {
      ...classifiedHeatPump, performancePoints: [], systemLinks: [{
        id: 'backup', role: 'backup_generator', targetKind: 'heating_system',
        targetId: 'boiler-1', evidenceReference: 'hydraulic scheme',
      }],
    };
    expect(() => calculateBENGMonthly(project)).toThrow(/heat pump details are not included/);
    expect(() => calculateBENG(project)).toThrow(/heat pump details are not included/);
  });
});
