import { useEffect } from 'react';
import { describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { createDefaultProject, useEnergy } from '../context/EnergyContext';
import { calculateBENGMonthly } from '../core/energy/BENGCalculatorMonthly';
import { generateReportHTML } from '../core/report/ReportTemplate';
import { exportBENGToIFC } from '../core/ifc/IFCEnergyExporter';
import { StatusBar } from '../components/StatusBar/StatusBar';
import { renderWithProviders } from './test-utils';

function CalculatedStatusBar() {
  const { state, dispatch } = useEnergy();
  useEffect(() => {
    dispatch({ type: 'SET_RESULT', payload: calculateBENGMonthly(state.project) });
  }, [dispatch]);
  return <StatusBar />;
}

describe('legacy result status across outputs', () => {
  it('shows indicative BENG status in the status bar', async () => {
    renderWithProviders(<CalculatedStatusBar />);
    expect(await screen.findByText('BENG: Indicative calculation')).toBeInTheDocument();
    expect(screen.queryByText(/BENG: (Pass|Fail)/)).not.toBeInTheDocument();
  });

  it('marks standalone HTML and IFC exports as unverified', () => {
    const project = createDefaultProject();
    const result = calculateBENGMonthly(project);
    const report = generateReportHTML(project, result);
    expect(report).toContain('Indicatieve BENG-waarden');
    expect(report).toContain('Geen geverifieerde NTA 8800-berekening');
    expect(report).not.toContain('VOLDOET');

    const model = exportBENGToIFC(project, result);
    const values = [...model.entities.values()]
      .filter((entity) => entity.type === 'IFCTEXT')
      .map((entity) => entity.attributes[0]);
    expect(values).toContain('UNVERIFIED');
    expect(values.filter((value) => value === 'INDICATIVE')).toHaveLength(3);
    expect(values).not.toContain('PASS');
    expect(values).not.toContain('FAIL');
  });

  it('escapes entered project text in the standalone indicative HTML report', () => {
    const project = createDefaultProject();
    project.name = '<img src=x onerror=alert(1)>';
    const report = generateReportHTML(project, calculateBENGMonthly(project));
    expect(report).not.toContain('<img src=x onerror=alert(1)>');
    expect(report).toContain('&lt;img src=x onerror=alert(1)&gt;');
  });
});
