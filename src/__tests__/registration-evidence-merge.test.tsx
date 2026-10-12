/**
 * Follow-up of review 35: an evidence link made from the context panel while
 * the registration form has unsaved evidence edits is kept on Opslaan. The
 * draft's evidence register merges with the stored one per item.
 */
import { useEffect } from 'react';
import { afterEach, describe, expect, it } from 'vitest';
import { screen } from '@testing-library/react';
import { useEnergy } from '../context/EnergyContext';
import { mergeEvidence, RegistrationDraftProvider } from '../context/RegistrationDraftProvider';
import { RegistrationForm } from '../components/shell/pages/RegistrationForm';
import { linkEvidence } from '../core/nta/EvidenceLinks';
import type { NtaEvidenceItem } from '../core/nta/KernelClient';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

afterEach(() => {
  localStorage.removeItem('nta-bag-registrations');
});

const photo: NtaEvidenceItem = { id: 'ev-1', kind: 'photo_detail', fileName: 'gevel.jpg', sha256: 'a'.repeat(64) };
const invoice: NtaEvidenceItem = { id: 'ev-2', kind: 'invoice', fileName: 'factuur.pdf', sha256: 'b'.repeat(64) };
const sheet: NtaEvidenceItem = { id: 'ev-3', kind: 'datasheet', fileName: 'blad.pdf', sha256: 'c'.repeat(64) };

describe('mergeEvidence', () => {
  it('keeps items and links added elsewhere next to the adviser edits and deletions', () => {
    const oldBase = [photo, invoice];
    const draft = [{ ...invoice, description: 'cv-ketel' }]; // photo deleted, invoice edited
    const newBase = [photo, { ...invoice }, sheet]; // sheet added elsewhere
    const conflicts: string[] = [];
    expect(mergeEvidence(draft, oldBase, newBase, conflicts)).toEqual([{ ...invoice, description: 'cv-ketel' }, sheet]);
    expect(conflicts).toEqual([]);
  });

  it('takes a link made elsewhere on an item the adviser did not touch', () => {
    const draft = [photo, { ...invoice, checkedBy: 'A. Adviseur' }];
    const newBase = linkEvidence([photo, invoice], 'ev-1', '/zones/0');
    expect(mergeEvidence(draft, [photo, invoice], newBase)).toEqual([
      { ...photo, linkedPaths: ['/zones/0'] }, { ...invoice, checkedBy: 'A. Adviseur' },
    ]);
  });

  it('keeps the adviser version of an item changed both ways and reports it', () => {
    const conflicts: string[] = [];
    const draft = [{ ...photo, description: 'voorgevel' }];
    const newBase = linkEvidence([photo], 'ev-1', '/zones/0');
    expect(mergeEvidence(draft, [photo], newBase, conflicts)).toEqual(draft);
    expect(conflicts).toEqual(['gevel.jpg']);
  });

  it('keeps an item the adviser deleted deleted, and one removed elsewhere removed unless edited', () => {
    const conflicts: string[] = [];
    expect(mergeEvidence([], [photo], linkEvidence([photo], 'ev-1', '/zones/0'), conflicts)).toEqual([]);
    expect(conflicts).toEqual(['gevel.jpg']);
    expect(mergeEvidence([photo], [photo], [])).toEqual([]);
    const edited = [{ ...photo, description: 'voorgevel' }];
    expect(mergeEvidence(edited, [photo], [])).toEqual(edited);
  });
});

function Harness() {
  const { state, dispatch } = useEnergy();
  const registration = state.project.registration ?? {};
  // What the context panel's evidence link does: store a link at once.
  const link = () => dispatch({ type: 'UPDATE_PROJECT_INFO', payload: {
    registration: { ...registration, evidence: linkEvidence(registration.evidence ?? [], 'ev-1', '/zones/0') },
  } });
  return <>
    <RegistrationForm />
    <button type="button" onClick={link}>link from context panel</button>
    <output data-testid="registration">{JSON.stringify(state.project.registration ?? null)}</output>
  </>;
}

function Seed({ evidence }: { evidence: NtaEvidenceItem[] }) {
  const { state, dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'UPDATE_PROJECT_INFO', payload: { registration: { evidence } } }); }, [dispatch, evidence]);
  if (!state.project.registration) return null;
  return <RegistrationDraftProvider><Harness /></RegistrationDraftProvider>;
}

const stored = () => JSON.parse(screen.getByTestId('registration').textContent ?? 'null') as { evidence: NtaEvidenceItem[] };

describe('registration form with a link from the context panel', () => {
  it('keeps the new link and the unsaved evidence edit on Opslaan', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Seed evidence={[photo, invoice]} />);
    await user.type(screen.getAllByLabelText('Description')[1], 'cv-ketel');
    await user.click(screen.getByRole('button', { name: 'link from context panel' }));
    expect(screen.queryByText(/Also changed elsewhere/)).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(stored().evidence).toEqual([
      { ...photo, linkedPaths: ['/zones/0'] }, { ...invoice, description: 'cv-ketel' },
    ]);
  }, SLOW_UI_TEST_MS);

  it('keeps the adviser version of the same item and says so', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Seed evidence={[photo, invoice]} />);
    await user.type(screen.getAllByLabelText('Description')[0], 'voorgevel');
    await user.click(screen.getByRole('button', { name: 'link from context panel' }));
    expect(screen.getByText(/Also changed elsewhere, your unsaved version is kept: gevel\.jpg/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Save' }));
    expect(stored().evidence).toEqual([{ ...photo, description: 'voorgevel' }, invoice]);
    expect(screen.queryByText(/Also changed elsewhere/)).toBeNull();
  }, SLOW_UI_TEST_MS);
});
