/**
 * The Registratie step (UI redesign F9): the registration data moved from the
 * project-info dialog to `RegistrationForm` with the same fields and
 * assertions; plus the readiness reasons and the basics-only dialog.
 */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { RegistrationForm } from '../components/shell/pages/RegistrationForm';
import { RegistrationPage, readinessReasons } from '../components/shell/pages/RegistrationPage';
import { createDefaultProject } from '../context/EnergyContext';
import { ProjectInfoDialog } from '../components/dialogs/ProjectInfoDialog/ProjectInfoDialog';
import { softwareIdentity } from '../core/nta/Registration';
import type { RegistrationAssessment } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';

afterEach(() => {
  localStorage.removeItem('nta-bag-registrations');
});

function Harness() {
  const { state } = useEnergy();
  return <>
    <RegistrationForm />
    <output data-testid="registration">{JSON.stringify(state.project.registration ?? null)}</output>
  </>;
}

describe('registration page', () => {
  it('stores the message type, WLC-GWP and program, and warns about a second label on one BAG object', async () => {
    localStorage.setItem('nta-bag-registrations', JSON.stringify([{
      bagObjectId: '0363010000000001', projectId: 'other', projectName: 'Woning elders',
      residential: true, messageType: 'regular', epOnlineNumber: 'EP-0',
    }]));
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    expect(screen.getByTestId('reg-software').textContent).toContain('not yet attested under BRL 9501');
    await user.type(screen.getByLabelText('BAG object id'), '0363010000000001');
    expect(screen.getByRole('alert').textContent).toContain('Woning elders');
    await user.selectOptions(screen.getByLabelText('Message type (BRL 9500 §4.2.5)'), 'replacement');
    expect(screen.queryByRole('alert')).toBeNull();
    await user.type(screen.getByLabelText('EP-Online number of the replaced label'), 'EP-1');
    await user.type(screen.getByLabelText('WLC-GWP, kg CO₂-eq/m²·yr'), '7.5');
    await user.type(screen.getByLabelText('Report in the project dossier'), 'wlc.pdf');
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

  it('shows the relabel fields only for a relabel (formerly in the project-info dialog)', async () => {
    const user = userEvent.setup();
    renderWithProviders(<RegistrationForm />);
    expect(screen.getByRole('form', { name: 'Registration data' })).toBeInTheDocument();
    expect(screen.queryByLabelText('Kernel version of original survey (relabel)')).not.toBeInTheDocument();
    await user.selectOptions(screen.getByLabelText('Message type (BRL 9500 §4.2.5)'), 'relabel');
    expect(screen.getByLabelText('Kernel version of original survey (relabel)')).toBeInTheDocument();
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Discard' }));
    expect(screen.queryByLabelText('Kernel version of original survey (relabel)')).not.toBeInTheDocument();
  }, 60000);

  it('names every reason a registration is not ready', () => {
    const base = { issues: [], plausibility: [] } as unknown as RegistrationAssessment;
    expect(readinessReasons(null)).toEqual([]);
    expect(readinessReasons({ ...base, readyForRegistration: true, dossierComplete: true, softwareAttested: true })).toEqual([]);
    expect(readinessReasons({ ...base, readyForRegistration: false, dossierComplete: false, softwareAttested: false }))
      .toEqual(['registration.page.reason.dossier', 'registration.page.reason.attest']);
    expect(readinessReasons({ ...base, readyForRegistration: false, dossierComplete: true, softwareAttested: false }))
      .toEqual(['registration.page.reason.attest']);
  });
});

describe('registration step', () => {
  it('blocks registration without an attest but keeps the form editable', () => {
    renderWithProviders(<RegistrationPage project={createDefaultProject()} actions={{ navigate: vi.fn(), openDialog: vi.fn() }} />);
    expect(screen.getByText('Registration in EP-Online requires a BRL 9501 attest')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Calculation program and attest' })).toBeInTheDocument();
    expect(screen.getByLabelText('BAG object id')).toBeEnabled();
  }, 60000);
});

describe('project information dialog', () => {
  it('keeps only the project basics and points to the Registratie step', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    function Basics() {
      const { state } = useEnergy();
      return <>
        <ProjectInfoDialog onClose={onClose} />
        <output data-testid="project">{JSON.stringify({ name: state.project.name, city: state.project.city, registration: state.project.registration ?? null })}</output>
      </>;
    }
    renderWithProviders(<Basics />);
    expect(screen.queryByLabelText('Message type (BRL 9500 §4.2.5)')).not.toBeInTheDocument();
    expect(screen.getByText(/now in the Registration step/)).toBeInTheDocument();
    const name = screen.getByLabelText('Project Name');
    await user.clear(name);
    await user.type(name, 'Woning 1');
    const city = screen.getByLabelText('City');
    await user.clear(city);
    await user.type(city, 'Delft');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(onClose).toHaveBeenCalledOnce();
    const stored = JSON.parse(screen.getByTestId('project').textContent ?? 'null');
    expect(stored).toMatchObject({ name: 'Woning 1', city: 'Delft' });
  }, 60000);
});
