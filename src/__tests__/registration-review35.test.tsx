/**
 * Review 35 of the registration end: the registration draft survives leaving
 * the step and counts in the close checks, a reverted field is no change, and
 * the evidence register's text fields keep what the adviser types.
 */
import { useState } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { RegistrationDraftProvider, useRegistrationDraft } from '../context/RegistrationDraftProvider';
import { RegistrationForm } from '../components/shell/pages/RegistrationForm';
import { EvidenceRegister } from '../components/dialogs/ProjectInfoDialog/EvidenceRegister';
import { DossierPage } from '../components/shell/pages/DeliveryPages';
import { confirmCloseAll, documentsNeedingConfirmation, type ClosingDocument } from '../core/io/projectFiles';
import type { NtaEvidenceItem } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

afterEach(() => {
  localStorage.removeItem('nta-bag-registrations');
});

const evidenceItem: NtaEvidenceItem = { id: 'ev-1', kind: 'photo_detail', fileName: 'gevel.jpg', sha256: 'a'.repeat(64) };

function Register({ initial }: { initial: NtaEvidenceItem[] }) {
  const [evidence, setEvidence] = useState(initial);
  return <>
    <EvidenceRegister evidence={evidence} onChange={setEvidence} />
    <button type="button" onClick={() => setEvidence(initial)}>reset</button>
    <output data-testid="evidence">{JSON.stringify(evidence)}</output>
  </>;
}

const evidenceIn = () => JSON.parse(screen.getByTestId('evidence').textContent ?? '[]') as NtaEvidenceItem[];

describe('evidence register text fields', () => {
  it('takes a typed list of linked inputs, comma by comma', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Register initial={[evidenceItem]} />);
    const field = screen.getByLabelText(/^Supports/);
    await user.type(field, '/zones/0, /zones/1');
    expect((field as HTMLInputElement).value).toBe('/zones/0, /zones/1');
    expect(evidenceIn()[0].linkedPaths).toEqual(['/zones/0', '/zones/1']);
  });

  it('shows the stored value again after the register is reset from outside', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Register initial={[{ ...evidenceItem, gps: { latitude: 52.37, longitude: 4.89 }, linkedPaths: ['/zones/0'] }]} />);
    const gps = screen.getByLabelText(/^GPS/) as HTMLInputElement;
    const links = screen.getByLabelText(/^Supports/) as HTMLInputElement;
    await user.clear(gps);
    await user.type(gps, '51.1, 5.2');
    await user.type(links, ', /zones/1');
    await user.click(screen.getByRole('button', { name: 'reset' }));
    expect(gps.value).toBe('52.37, 4.89');
    expect(links.value).toBe('/zones/0');
  });
});

function Harness() {
  const { state } = useEnergy();
  const [shown, setShown] = useState(true);
  const draft = useRegistrationDraft();
  return <>
    <button type="button" onClick={() => setShown((value) => !value)}>toggle step</button>
    {shown && <RegistrationForm />}
    <output data-testid="dirty">{String(draft?.dirty ?? false)}</output>
    <output data-testid="registration">{JSON.stringify(state.project.registration ?? null)}</output>
  </>;
}

function renderRegistration() {
  return renderWithProviders(<RegistrationDraftProvider><Harness /></RegistrationDraftProvider>);
}

describe('registration draft', () => {
  it('keeps unsaved registration data when the adviser leaves the step and comes back', async () => {
    const user = userEvent.setup();
    renderRegistration();
    await user.type(screen.getByLabelText('BAG object id'), '0363010000000001');
    expect(screen.getByTestId('dirty').textContent).toBe('true');
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    // Away from the step the unsaved draft is still known (for the close checks).
    expect(screen.getByTestId('dirty').textContent).toBe('true');
    await user.click(screen.getByRole('button', { name: 'toggle step' }));
    expect((screen.getByLabelText('BAG object id') as HTMLInputElement).value).toBe('0363010000000001');
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('registration').textContent ?? 'null')).toMatchObject({ bagObjectId: '0363010000000001' });
    expect(screen.getByTestId('dirty').textContent).toBe('false');
  }, SLOW_UI_TEST_MS);

  it('counts a field set and cleared again as no change', async () => {
    const user = userEvent.setup();
    renderRegistration();
    await user.type(screen.getByLabelText('Surveying adviser'), 'A');
    await user.clear(screen.getByLabelText('Surveying adviser'));
    await user.click(screen.getByLabelText(/^Serial project/));
    await user.click(screen.getByLabelText(/^Serial project/));
    await user.selectOptions(screen.getByLabelText(/^k\./), 'yes');
    await user.selectOptions(screen.getByLabelText(/^k\./), '');
    expect(screen.getByTestId('dirty').textContent).toBe('false');
    expect(screen.getByText('No changes to save')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  }, SLOW_UI_TEST_MS);

  it('keeps the unsaved draft when another step stores registration data', async () => {
    const user = userEvent.setup();
    function Store() {
      const { state, dispatch } = useEnergy();
      return <button type="button" onClick={() => dispatch({ type: 'UPDATE_PROJECT_INFO', payload: {
        registration: { ...(state.project.registration ?? {}), originalKernelVersion: '0.3.0' },
      } })}>store elsewhere</button>;
    }
    renderWithProviders(<RegistrationDraftProvider><Harness /><Store /></RegistrationDraftProvider>);
    await user.type(screen.getByLabelText('BAG object id'), '0363010000000001');
    await user.click(screen.getByRole('button', { name: 'store elsewhere' }));
    expect((screen.getByLabelText('BAG object id') as HTMLInputElement).value).toBe('0363010000000001');
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(JSON.parse(screen.getByTestId('registration').textContent ?? 'null'))
      .toMatchObject({ bagObjectId: '0363010000000001', originalKernelVersion: '0.3.0' });
  }, SLOW_UI_TEST_MS);
});

describe('EP-Online overview on the checklist', () => {
  it('shows the answers to label statements k and l next to the export fields', async () => {
    const user = userEvent.setup();
    function Answer() {
      const { state, dispatch } = useEnergy();
      return <button type="button" onClick={() => dispatch({ type: 'UPDATE_PROJECT_INFO', payload: {
        registration: { ...(state.project.registration ?? {}), labelStatements: { respondsToExternalSignals: true } },
      } })}>answer k</button>;
    }
    renderWithProviders(<><Answer /><DossierPage actions={{ navigate: () => undefined }} /></>);
    await user.click(screen.getByRole('button', { name: 'answer k' }));
    const statements = within(screen.getByTestId('ep-online-statements'));
    expect(statements.getByText(/^Statements k and l/)).toBeInTheDocument();
    const rows = screen.getByTestId('ep-online-statements').querySelectorAll('tr');
    expect(rows[0].textContent).toMatch(/^k\..*yes$/);
    expect(rows[1].textContent).toMatch(/^l\..*not answered$/);
  }, SLOW_UI_TEST_MS);
});

describe('close checks', () => {
  it('asks before closing a document with unsaved registration data', async () => {
    const doc: ClosingDocument = { id: 'a', name: 'A', isDirty: false, draftDirty: false, registrationDirty: true };
    expect(documentsNeedingConfirmation([doc])).toEqual([doc]);
    const asked: ClosingDocument[] = [];
    const closes = await confirmCloseAll([doc], {
      confirmDiscardDraft: async (item) => { asked.push(item); return false; },
      askUnsaved: async () => 'discard',
      save: async () => true,
    });
    expect(asked).toEqual([doc]);
    expect(closes).toBe(false);
  });
});
