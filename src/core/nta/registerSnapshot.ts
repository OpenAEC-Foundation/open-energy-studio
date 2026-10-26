/**
 * Snapshot of a register declaration (issue #15), kept with the evidence item
 * of that declaration: the values the adviser copied from the BCRG register
 * (or another register), with declaration number, revision, product, scope,
 * date and a SHA-256 over the values. The kernel checks it at registration
 * (crates/nta8800-core/src/register_snapshot.rs).
 *
 * There is no register API contract yet, so the route is `manual`. The
 * snapshot never changes the project and the project never changes the
 * snapshot: a later change on either side shows as a finding. Whether the
 * declaration is still current is a separate check the adviser records.
 */
import type { NtaEvidenceItem, NtaRegisterSnapshot, NtaRegisterValue } from './KernelClient';

const K = new Uint32Array([
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
]);

/**
 * SHA-256 (lowercase hex) of UTF-8 text, synchronous, so the snapshot hash
 * follows every keystroke without an await between typing and showing.
 */
export function sha256Text(text: string): string {
  const data = new TextEncoder().encode(text);
  const length = data.length;
  const padded = new Uint8Array(((length + 9 + 63) >> 6) << 6);
  padded.set(data);
  padded[length] = 0x80;
  const view = new DataView(padded.buffer);
  view.setUint32(padded.length - 8, Math.floor(length / 0x20000000));
  view.setUint32(padded.length - 4, (length << 3) >>> 0);
  const h = new Uint32Array([0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19]);
  const w = new Uint32Array(64);
  const rotr = (x: number, n: number) => (x >>> n) | (x << (32 - n));
  for (let offset = 0; offset < padded.length; offset += 64) {
    for (let i = 0; i < 16; i += 1) w[i] = view.getUint32(offset + i * 4);
    for (let i = 16; i < 64; i += 1) {
      const s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >>> 3);
      const s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >>> 10);
      w[i] = (w[i - 16] + s0 + w[i - 7] + s1) >>> 0;
    }
    let [a, b, c, d, e, f, g, hh] = h;
    for (let i = 0; i < 64; i += 1) {
      const t1 = (hh + (rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25)) + ((e & f) ^ (~e & g)) + K[i] + w[i]) >>> 0;
      const t2 = ((rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22)) + ((a & b) ^ (a & c) ^ (b & c))) >>> 0;
      hh = g; g = f; f = e; e = (d + t1) >>> 0; d = c; c = b; b = a; a = (t1 + t2) >>> 0;
    }
    h[0] += a; h[1] += b; h[2] += c; h[3] += d; h[4] += e; h[5] += f; h[6] += g; h[7] += hh;
  }
  return Array.from(h, (word) => word.toString(16).padStart(8, '0')).join('');
}

export const REGISTRY_STATES: Array<NonNullable<NtaRegisterSnapshot['registryCheck']>['state']> = [
  'not_checked', 'current', 'changed', 'withdrawn',
];

/** The evidence kinds a register snapshot belongs to. */
export function canHoldSnapshot(item: Pick<NtaEvidenceItem, 'kind'>): boolean {
  return item.kind === 'quality_declaration' || item.kind === 'declaration_of_performance';
}

/**
 * SHA-256 over the values, one line per value: label TAB value TAB unit TAB
 * path LF (absent fields empty), as the kernel computes it.
 */
export function snapshotValuesSha256(values: NtaRegisterValue[]): string {
  return sha256Text(values.map((item) => [item.label, item.value, item.unit ?? '', item.path ?? ''].join('\t') + '\n').join(''));
}

const today = () => new Date().toISOString().slice(0, 10);

/** A new, unconfirmed snapshot for an evidence item; the adviser fills it in. */
export function newSnapshot(item: NtaEvidenceItem, on: string = today()): NtaRegisterSnapshot {
  const values: NtaRegisterValue[] = [{ label: '', value: '' }];
  return {
    register: 'BCRG',
    route: 'manual',
    declarationNumber: '',
    productName: '',
    capturedOn: item.date ?? on,
    values,
    valuesSha256: snapshotValuesSha256(values),
    registryCheck: { state: 'not_checked' },
  };
}

/**
 * The snapshot with new values: the hash follows the values, and the
 * adviser's confirmation lapses, so changed values are never taken as
 * confirmed.
 */
export function withValues(snapshot: NtaRegisterSnapshot, values: NtaRegisterValue[]): NtaRegisterSnapshot {
  const { confirmedBy: _by, confirmedOn: _on, ...rest } = snapshot;
  return { ...rest, values, valuesSha256: snapshotValuesSha256(values) };
}

/** Identity fields that change what was confirmed. */
const CONFIRMED_FIELDS: Array<keyof NtaRegisterSnapshot> = [
  'register', 'declarationNumber', 'revision', 'productName', 'manufacturer', 'scope', 'capturedOn',
];

/** The snapshot with a changed identity field; a confirmed snapshot needs confirming again. */
export function withField<K extends keyof NtaRegisterSnapshot>(snapshot: NtaRegisterSnapshot, key: K, value: NtaRegisterSnapshot[K]): NtaRegisterSnapshot {
  const next: Partial<NtaRegisterSnapshot> = { ...snapshot, [key]: value };
  if (value === undefined || value === '') {
    // Required fields stay as an empty text; the kernel reports them.
    if (key === 'revision' || key === 'manufacturer' || key === 'scope') delete next[key];
  }
  if (CONFIRMED_FIELDS.includes(key)) {
    delete next.confirmedBy;
    delete next.confirmedOn;
  }
  return next as NtaRegisterSnapshot;
}

/** Status for the adviser: manual and (un)confirmed, and the register check. */
export function snapshotStatus(snapshot: NtaRegisterSnapshot): { confirmed: boolean; registry: NonNullable<NtaRegisterSnapshot['registryCheck']>['state'] } {
  return {
    confirmed: Boolean(snapshot.confirmedBy?.trim() && snapshot.confirmedOn),
    registry: snapshot.registryCheck?.state ?? 'not_checked',
  };
}
