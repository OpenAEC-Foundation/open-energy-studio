import { afterEach, describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { ProjectInfoDialog } from '../components/dialogs/ProjectInfoDialog/ProjectInfoDialog';
import { softwareIdentity } from '../core/nta/Registration';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  localStorage.removeItem('nta-bag-registrations');
});

function Harness() {
  const { state } = useEnergy();
  return <>
    <ProjectInfoDialog onClose={() => undefined} />
    <output data-testid="registration">{JSON.stringify(state.project.registration ?? null)}</output>
  </>;
}

describe('registration dialog', () => {
  it('stores the message type, WLC-GWP and program, and warns about a second label on one BAG object', async () => {
    localStorage.setItem('nta-bag-registrations', JSON.stringify([{
      bagObjectId: '0363010000000001', projectId: 'other', projectName: 'Woning elders',
      residential: true, messageType: 'regular', epOnlineNumber: 'EP-0',
    }]));
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    expect(screen.getByTestId('reg-software').textContent).toContain('not yet attested under BRL 9501');
    await user.type(screen.getByRole('textbox', { name: 'BAG object id' }), '0363010000000001');
    expect(screen.getByRole('alert').textContent).toContain('Woning elders');
    await user.selectOptions(screen.getByRole('combobox', { name: 'Message type (BRL 9500 §4.2.5)' }), 'replacement');
    expect(screen.queryByRole('alert')).toBeNull();
    await user.type(screen.getByRole('textbox', { name: 'EP-Online number of the replaced label' }), 'EP-1');
    await user.type(screen.getByRole('spinbutton', { name: 'WLC-GWP, kg CO₂-eq/m²·yr' }), '7.5');
    await user.type(screen.getByRole('textbox', { name: 'Report in the project dossier' }), 'wlc.pdf');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    const stored = JSON.parse(screen.getByTestId('registration').textContent ?? 'null');
    expect(stored).toMatchObject({
      bagObjectId: '0363010000000001', messageType: 'replacement', replacedEpOnlineNumber: 'EP-1',
      wlcGwp: { valueKgCo2EqPerM2Year: 7.5, reportReference: 'wlc.pdf' }, software: softwareIdentity(),
    });
    const ledger = JSON.parse(localStorage.getItem('nta-bag-registrations') ?? '[]');
    // Not registered yet (no EP-Online number), so not in the ledger.
    expect(ledger.map((item: { projectId: string }) => item.projectId)).toEqual(['other']);
  }, 60000);
});
