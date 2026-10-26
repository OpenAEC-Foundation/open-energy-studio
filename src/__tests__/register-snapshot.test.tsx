/**
 * Issue #15: snapshot of a register declaration (BCRG) with the evidence
 * item. The hash is the kernel's (crates/nta8800-core/src/register_snapshot.rs),
 * a change voids the adviser's confirmation, the register check is separate
 * and the project dossier carries the snapshot.
 */
import { useState } from 'react';
import { describe, expect, it } from 'vitest';
import { screen, within } from '@testing-library/react';
import { createDefaultProject } from '../context/EnergyContext';
import { sha256Hex } from '../core/nta/Evidence';
import type { NtaEvidenceItem, NtaRegisterSnapshot } from '../core/nta/KernelClient';
import {
  canHoldSnapshot, newSnapshot, sha256Text, snapshotStatus, snapshotValuesSha256, withField, withValues,
} from '../core/nta/registerSnapshot';
import { checkDossierCompleteness } from '../core/report/ProjectDossier';
import { EvidenceRegister } from '../components/dialogs/ProjectInfoDialog/EvidenceRegister';
import { renderWithProviders, userEvent } from './test-utils';
import { SLOW_UI_TEST_MS } from './timeouts';

const declaration: NtaEvidenceItem = {
  id: 'ev-1', kind: 'quality_declaration', fileName: 'verklaring.pdf', sha256: 'a'.repeat(64), date: '2026-10-20', checkedBy: 'A. Adviseur',
};

function confirmed(): NtaRegisterSnapshot {
  const values = [{ label: 'COP A7/W35', value: '4,21', path: '/heatingSystems/0/cop' }];
  return {
    register: 'BCRG', route: 'manual', declarationNumber: '20230256GK', revision: '2', productName: 'Warmtepomp X',
    capturedOn: '2026-10-26', values, valuesSha256: snapshotValuesSha256(values),
    confirmedBy: 'A. Adviseur', confirmedOn: '2026-10-26', registryCheck: { state: 'current', checkedOn: '2026-10-26' },
  };
}

describe('register snapshot hash', () => {
  it('equals WebCrypto SHA-256, also over multiple blocks and non-ASCII text', async () => {
    for (const text of ['', 'abc', 'COP\t4,21\t\t\n', 'é'.repeat(100), 'x'.repeat(55), 'x'.repeat(56), 'x'.repeat(64), 'x'.repeat(1000)]) {
      expect(sha256Text(text), JSON.stringify(text.slice(0, 10))).toBe(await sha256Hex(new TextEncoder().encode(text)));
    }
  });

  it('is the kernel hash: label TAB value TAB unit TAB path LF per value', () => {
    expect(snapshotValuesSha256([{ label: 'COP', value: '4,21' }])).toBe(sha256Text('COP\t4,21\t\t\n'));
    expect(snapshotValuesSha256([{ label: 'COP', value: '4,21', unit: '-', path: '/a' }, { label: 'P', value: '6' }]))
      .toBe(sha256Text('COP\t4,21\t-\t/a\nP\t6\t\t\n'));
  });
});

describe('register snapshot rules', () => {
  it('starts manual, unconfirmed and not checked in the register', () => {
    const snapshot = newSnapshot(declaration);
    expect(snapshot).toMatchObject({ register: 'BCRG', route: 'manual', capturedOn: '2026-10-20', registryCheck: { state: 'not_checked' } });
    expect(snapshotStatus(snapshot)).toEqual({ confirmed: false, registry: 'not_checked' });
    expect(canHoldSnapshot(declaration)).toBe(true);
    expect(canHoldSnapshot({ kind: 'invoice' })).toBe(false);
  });

  it('voids the confirmation when values or identity change, not when the register is checked', () => {
    const snapshot = confirmed();
    expect(snapshotStatus(snapshot).confirmed).toBe(true);
    const edited = withValues(snapshot, [{ label: 'COP A7/W35', value: '4,3' }]);
    expect(snapshotStatus(edited).confirmed).toBe(false);
    expect(edited.valuesSha256).toBe(snapshotValuesSha256(edited.values));
    expect(snapshotStatus(withField(snapshot, 'revision', '3')).confirmed).toBe(false);
    // The register check is a separate act: it keeps the values and the confirmation.
    const checked: NtaRegisterSnapshot = { ...snapshot, registryCheck: { state: 'withdrawn', checkedOn: '2026-11-01' } };
    expect(snapshotStatus(checked)).toEqual({ confirmed: true, registry: 'withdrawn' });
    expect(checked.values).toEqual(snapshot.values);
  });
});

describe('register snapshot in the evidence register', () => {
  function Harness() {
    const [evidence, setEvidence] = useState<NtaEvidenceItem[]>([declaration]);
    return <>
      <EvidenceRegister evidence={evidence} onChange={setEvidence} />
      <output data-testid="snapshot">{JSON.stringify(evidence[0].registerSnapshot ?? null)}</output>
    </>;
  }
  const stored = () => JSON.parse(screen.getByTestId('snapshot').textContent ?? 'null') as NtaRegisterSnapshot | null;

  it('records, confirms and voids the confirmation after a change', async () => {
    const user = userEvent.setup();
    renderWithProviders(<Harness />);
    await user.click(screen.getByRole('button', { name: 'Record register snapshot' }));
    const status = () => within(screen.getByRole('group', { name: 'Register snapshot' })).getByRole('status').textContent ?? '';
    expect(status()).toBe('Copied by hand, not yet confirmed · register not checked');
    await user.type(screen.getByLabelText('Declaration number'), '20230256GK');
    await user.type(screen.getByLabelText('Product'), 'Warmtepomp X');
    await user.type(screen.getByLabelText('Quantity 1'), 'COP A7/W35');
    await user.type(screen.getByLabelText('Value 1'), '4,21');
    expect(stored()?.values).toEqual([{ label: 'COP A7/W35', value: '4,21' }]);
    expect(stored()?.valuesSha256).toBe(snapshotValuesSha256([{ label: 'COP A7/W35', value: '4,21' }]));
    const confirm = screen.getByRole('button', { name: 'Confirm values and scope' });
    expect(confirm).toBeDisabled();
    await user.type(screen.getByLabelText('Confirmed by'), 'A. Adviseur');
    await user.click(confirm);
    expect(status()).toBe('Copied by hand and confirmed · register not checked');
    expect(stored()?.confirmedOn).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    // A later change voids it.
    await user.type(screen.getByLabelText('Value 1'), '5');
    expect(stored()?.confirmedOn).toBeUndefined();
    expect(status()).toMatch(/^Copied by hand, not yet confirmed/);
    // The register check is separate and dated.
    await user.selectOptions(screen.getByLabelText('Status in the register'), 'withdrawn');
    expect(stored()?.registryCheck).toMatchObject({ state: 'withdrawn', checkedOn: expect.stringMatching(/^\d{4}-/) });
  }, SLOW_UI_TEST_MS);
});

describe('register snapshot in the project dossier', () => {
  const projectWith = (snapshot: NtaRegisterSnapshot) => ({
    ...createDefaultProject(),
    registration: { purpose: 'existing_building' as const, evidence: [{ ...declaration, registerSnapshot: snapshot }] },
  });
  const item = (snapshot: NtaRegisterSnapshot) =>
    checkDossierCompleteness({ project: projectWith(snapshot) }).find((entry) => entry.id === 'register_snapshots');

  it('is ok when confirmed and not withdrawn, and names what is not', () => {
    expect(item(confirmed())?.status).toBe('ok');
    const { confirmedOn: _on, ...unconfirmed } = confirmed();
    expect(item(unconfirmed as NtaRegisterSnapshot)).toMatchObject({ status: 'missing', detail: '1 niet bevestigd' });
    expect(item({ ...confirmed(), registryCheck: { state: 'withdrawn', checkedOn: '2026-11-01' } })).toMatchObject({ status: 'missing', detail: '1 ingetrokken' });
    expect(item({ ...confirmed(), registryCheck: undefined })).toMatchObject({ status: 'ok', detail: '1 niet in het register gecontroleerd' });
  });

  it('is not listed without snapshots', () => {
    const project = { ...createDefaultProject(), registration: { purpose: 'existing_building' as const, evidence: [declaration] } };
    expect(checkDossierCompleteness({ project }).some((entry) => entry.id === 'register_snapshots')).toBe(false);
  });
});
