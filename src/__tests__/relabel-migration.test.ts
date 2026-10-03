import { describe, expect, it } from 'vitest';
import { migrateLegacyRelabel } from '../core/nta/Registration';
import type { NtaRegistration } from '../core/nta/KernelClient';

const invoice = { id: 'ev-1', kind: 'invoice' as const, fileName: 'factuur.pdf', sha256: 'a'.repeat(64) };

describe('legacy relabel projects (BRL 9500-W §4.2.3–4.2.4)', () => {
  it('marks a role-less invoice as the specified invoice and lists the new fields', () => {
    const registration: NtaRegistration = {
      messageType: 'relabel', evidence: [invoice],
      relabelComparison: { originalFileName: 'o.oes.json', assessment: { source: 'x', scheme: 'w', allowed: true, needsReview: false, changes: [] } },
    };
    const { project, missing } = migrateLegacyRelabel({ registration });
    expect(project.registration?.evidence?.[0].relabelProof).toBe('specified_invoice');
    expect(missing).toEqual(['originalCertificateNumber', 'originalEpOnlineNumber', 'relabelComparison']);
  });

  it('leaves regular registrations and up-to-date relabels alone', () => {
    const regular = { registration: { messageType: 'regular' as const, evidence: [invoice] } };
    expect(migrateLegacyRelabel(regular)).toEqual({ project: regular, missing: [] });
    const current = { registration: {
      messageType: 'relabel' as const, originalCertificateNumber: 'K1', originalEpOnlineNumber: 'EP-1',
      evidence: [{ ...invoice, relabelProof: 'quote_with_order' as const }],
      relabelComparison: { originalFileName: 'o.oes.json', originalProjectText: '{}',
        assessment: { source: 'x', scheme: 'w' as const, allowed: true, needsReview: false, changes: [] } },
    } };
    expect(migrateLegacyRelabel(current)).toEqual({ project: current, missing: [] });
  });
});
