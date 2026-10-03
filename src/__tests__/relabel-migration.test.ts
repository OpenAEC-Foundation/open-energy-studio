import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import {
  canonicalNumber, labelInputSha256, migrateLegacyRelabel, originalProjectTextForStorage, relabelNoticeKey,
} from '../core/nta/Registration';
import type { NtaRegistration } from '../core/nta/KernelClient';

const invoice = { id: 'ev-1', kind: 'invoice' as const, fileName: 'factuur.pdf', sha256: 'a'.repeat(64) };

describe('legacy relabel projects (BRL 9500-W §4.2.3–4.2.4)', () => {
  it('marks a role-less invoice for review, not as proof, and lists the new fields', () => {
    const registration: NtaRegistration = {
      messageType: 'relabel', evidence: [invoice],
      relabelComparison: { originalFileName: 'o.oes.json', assessment: { source: 'x', scheme: 'w', allowed: true, needsReview: false, changes: [] } },
    };
    const { project, missing, markedForReview } = migrateLegacyRelabel({ registration });
    expect(project.registration?.evidence?.[0].relabelProof).toBe('review');
    expect(markedForReview).toBe(1);
    expect(missing).toEqual(['originalCertificateNumber', 'originalEpOnlineNumber', 'relabelComparison', 'relabelProof']);
  });

  it('is idempotent: a migrated file is not touched again', () => {
    const registration: NtaRegistration = { relabel: true, evidence: [invoice, { ...invoice, id: 'ev-2' }] };
    const first = migrateLegacyRelabel({ registration });
    expect(first.markedForReview).toBe(2);
    const second = migrateLegacyRelabel(first.project);
    expect(second).toEqual({ project: first.project, missing: [], markedForReview: 0 });
  });

  it('leaves regular registrations and relabels under the new rules alone', () => {
    const regular = { registration: { messageType: 'regular' as const, evidence: [invoice] } };
    expect(migrateLegacyRelabel(regular)).toEqual({ project: regular, missing: [], markedForReview: 0 });
    // A file with any of the new fields is not a legacy file, even when an
    // invoice still has no role: the adviser is already working on it.
    const current = { registration: {
      messageType: 'relabel' as const, originalCertificateNumber: 'K1',
      evidence: [invoice],
    } };
    expect(migrateLegacyRelabel(current)).toEqual({ project: current, missing: [], markedForReview: 0 });
  });

  it('keys the one-time notice per file when there is no project id', async () => {
    const a = await relabelNoticeKey(undefined, '/tmp/a.oes.json', '{}');
    const b = await relabelNoticeKey(undefined, '/tmp/b.oes.json', '{}');
    expect(a).not.toBe(b);
    const c = await relabelNoticeKey(undefined, undefined, '{"a":1}');
    const d = await relabelNoticeKey(undefined, undefined, '{"a":2}');
    expect(c).not.toBe(d);
    expect(await relabelNoticeKey('p1', '/tmp/a.oes.json', '{}')).toBe('oes-relabel-migration-notice:p1');
  });
});

describe('stored original project', () => {
  it('leaves out a nested comparison with its own original', () => {
    const nested = JSON.stringify({
      type: 'open-energy-studio', version: '1.0',
      project: { name: 'W', registration: { epOnlineNumber: 'EP-1', relabelComparison: { originalFileName: 'x', originalProjectText: '{"big":true}', assessment: {} } } },
    });
    const stored = JSON.parse(originalProjectTextForStorage(nested));
    expect(stored.project.registration.epOnlineNumber).toBe('EP-1');
    expect(stored.project.registration.relabelComparison.originalFileName).toBe('x');
    expect(stored.project.registration.relabelComparison.originalProjectText).toBeUndefined();
    // Other files are kept byte for byte.
    const plain = '{"type":"open-energy-studio","project":{"name":"W"}}';
    expect(originalProjectTextForStorage(plain)).toBe(plain);
  });
});

describe('label input hash', () => {
  it('writes numbers like the kernel', () => {
    expect(canonicalNumber(String(-0))).toBe('0');
    expect(canonicalNumber('100')).toBe('1e2');
    expect(canonicalNumber('1e-7')).toBe('1e-7');
    expect(canonicalNumber('0.0000001')).toBe('1e-7');
    expect(canonicalNumber('1e+21')).toBe('1e21');
    expect(canonicalNumber('-12.50')).toBe('-125e-1');
  });

  it('equals the kernel hash on the shared cases', async () => {
    const fixture = JSON.parse(readFileSync('training-data/nta8800-label-hash-cases.json', 'utf8')) as {
      cases: Array<{ name: string; project: object; sha256: string }>;
    };
    for (const item of fixture.cases) {
      expect(await labelInputSha256(item.project), item.name).toBe(item.sha256);
    }
  });
});
