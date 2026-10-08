import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { version } from '../../package.json';
import attest from '../core/nta/attest.json';
import { SOFTWARE_ATTEST_NUMBER, SOFTWARE_NAME } from '../core/nta/Registration';
import { readIdentity, renderLeveringsdocument } from '../../scripts/nta-leveringsdocument.mjs';

const root = join(__dirname, '../..');
const template = readFileSync(join(root, 'docs/templates/nta8800-leveringsdocument.md'), 'utf8');

describe('leveringsdocument (BRL 9501 §6.1)', () => {
  it('reads one identity shared with the app', () => {
    const identity = readIdentity(root);
    expect(identity.softwareName).toBe(SOFTWARE_NAME);
    expect(identity.attestNumber).toBe(SOFTWARE_ATTEST_NUMBER);
    expect(identity.attestNumber).toBe(attest.attestNumber);
    expect(identity.programVersion).toBe(version);
    expect(identity.kernelVersion).toMatch(/^\d+\.\d+\.\d+$/);
    expect(identity.targetNormVersion).toBe('NTA 8800:2025+C1:2026');
  });

  it('fills every field and lists every package with its SHA-256', () => {
    const text = renderLeveringsdocument(template, {
      ...readIdentity(root),
      commit: 'abc123',
      tag: 'oes-v1-kernel-0.2.0',
      releaseDate: '8 oktober 2026',
      packages: [{ name: 'open-energy-studio.deb', sha256: 'f'.repeat(64) }],
    });
    expect(text).not.toMatch(/\{\{/);
    expect(text).toContain(`| Rekenkernversie (\`KERNEL_VERSION\`) | ${readIdentity(root).kernelVersion} |`);
    expect(text).toContain(`| \`open-energy-studio.deb\` | \`${'f'.repeat(64)}\` |`);
    expect(text).toContain('| Broncode (commit) | abc123 |');
    // Without an attest the document says so instead of leaving fields blank.
    expect(text).toContain('| Attestnummer (BRL 9501) | nog niet toegekend |');
    expect(text).toContain('niet geattesteerd volgens BRL 9501');
  });

  it('names the attest once it is granted, and refuses unknown fields', () => {
    const attested = renderLeveringsdocument(template, {
      ...readIdentity(root),
      attestNumber: 'K12345',
      identificationCode: 'OES-01',
      releaseDate: '1 december 2026',
      packages: [],
    });
    expect(attested).toContain('| Attestnummer (BRL 9501) | K12345 |');
    expect(attested).toContain('onder attestnummer K12345');
    expect(attested).toContain('(geen pakketten gebouwd)');
    expect(() => renderLeveringsdocument('{{nope}}', { ...readIdentity(root), releaseDate: '', packages: [] }))
      .toThrow(/Onbekend veld/);
  });
});
