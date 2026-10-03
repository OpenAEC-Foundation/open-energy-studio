import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { buildNtaCalculationTemplate } from '../core/nta/NtaCalculationTemplate';
import { NtaCalculationForm } from '../components/NtaPerformancePanel/NtaCalculationForm';
import { renderWithProviders } from './test-utils';

describe('NTA form JSON fields', () => {
  it('blocks saving invalid JSON and saves the latest valid field value', () => {
    const onSave = vi.fn();
    function Harness() {
      const { state } = useEnergy();
      const template = buildNtaCalculationTemplate(state.project);
      const initial = {
        ...template,
        externalSupply: {
          ...(template.externalSupply as Record<string, unknown>),
          areaElectricity: [{ kind: 'pv' }],
        },
      };
      return <NtaCalculationForm project={state.project} initial={initial} onSave={onSave} onCancel={() => {}} />;
    }
    renderWithProviders(<Harness />);
    const field = document.querySelector<HTMLTextAreaElement>('textarea[data-nta-json-field]');
    expect(field).not.toBeNull();
    const save = screen.getByRole('button', { name: 'Save' });

    fireEvent.change(field!, { target: { value: '{invalid' } });
    expect(field).toHaveAttribute('aria-invalid', 'true');
    fireEvent.click(save);
    expect(onSave).not.toHaveBeenCalled();

    fireEvent.change(field!, { target: { value: '{"kind":"pv","sourceReference":"case"}' } });
    expect(field).toHaveAttribute('aria-invalid', 'false');
    fireEvent.click(save);
    expect(onSave).toHaveBeenCalledTimes(1);
    expect(onSave.mock.calls[0][0].externalSupply.areaElectricity[0])
      .toEqual({ kind: 'pv', sourceReference: 'case' });
  });
});
