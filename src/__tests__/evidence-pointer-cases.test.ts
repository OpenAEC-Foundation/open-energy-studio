import { describe, expect, it } from 'vitest';
import cases from '../../training-data/nta8800-evidence-pointer-cases.json';
import { resolvePointer } from '../core/nta/EvidenceLinks';

// The same cases are resolved by resolve_evidence_pointer in
// crates/nta8800-core/src/registration.rs, which checks the links at
// registration; both must agree or a link the app writes is refused there.
describe('evidence pointer cases shared with the kernel', () => {
  for (const item of cases.cases) {
    it(`${item.pointer} → ${item.resolved ?? 'dangling'}`, () => {
      expect(resolvePointer(cases.root, item.pointer)).toBe(item.resolved);
    });
  }
});
