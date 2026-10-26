/**
 * Issue #16: the README and the wiki name the published release and the
 * kernel on `main` apart, and describe how the app reaches the kernel as
 * src/core/nta/kernelTransport.ts does. The sources of truth: package.json
 * (program version), crates/nta8800-core/Cargo.toml (KERNEL_VERSION) and the
 * newest "## Rekenkern X.Y.Z" section of the release notes (released kernel).
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(__dirname, '../..');
const read = (path: string) => readFileSync(resolve(root, path), 'utf8').replace(/\r\n/g, '\n');

const program = (JSON.parse(read('package.json')) as { version: string }).version;
const kernel = /^version = "([^"]+)"/m.exec(read('crates/nta8800-core/Cargo.toml'))?.[1] ?? '';
const released = /^## Rekenkern (\d+\.\d+\.\d+) /m.exec(read('docs/nta8800-releasenotes.md'))?.[1] ?? '';

describe('versions in the README and the wiki', () => {
  it('reads the three versions', () => {
    expect(program).toMatch(/^\d+\.\d+\.\d+$/);
    expect(kernel).toMatch(/^\d+\.\d+\.\d+$/);
    expect(released).toMatch(/^\d+\.\d+\.\d+$/);
  });

  it('puts the current release first in the version tables', () => {
    for (const path of ['README.md', 'wiki/Home.md', 'wiki/Versiebeheer-en-releases.md']) {
      const firstRow = read(path).split('\n').find((line) => /^\| \*\*/.test(line)) ?? '';
      expect(firstRow, path).toContain(`**${program}**`);
      expect(firstRow, path).toContain(`**${released}**`);
    }
    expect(read('wiki/_Footer.md')).toContain(`Programma ${program} · Rekenkern ${released}`);
  });

  it('names the kernel on main apart when it is ahead of the release', () => {
    if (kernel === released) return;
    expect(read('README.md')).toContain(`**On \`main\`:** calculation kernel \`${kernel}\``);
    for (const path of ['wiki/Home.md', 'wiki/Versiebeheer-en-releases.md']) {
      expect(read(path), path).toContain(`**Op \`main\`:** rekenkern **${kernel}**`);
    }
    expect(read('wiki/_Footer.md')).toContain(`(main: ${kernel})`);
  });
});

describe('the missing attest', () => {
  const attested = (JSON.parse(read('src/core/nta/attest.json')) as { attestNumber: string }).attestNumber.trim() !== '';

  it('is stated in the README, the wiki and the installer while there is no attest number', () => {
    if (attested) return;
    expect(read('README.md')).toContain('**Not attested.**');
    for (const path of ['wiki/Home.md', 'wiki/Versiebeheer-en-releases.md']) expect(read(path), path).toContain('**Niet geattesteerd.**');
    expect(read('wiki/_Footer.md')).toContain('niet geattesteerd');
    // The Windows setup shows this page before it installs (bundle.licenseFile).
    const conf = JSON.parse(read('src-tauri/tauri.conf.json')) as { bundle: { licenseFile?: string } };
    expect(conf.bundle.licenseFile).toBe('installer-notice.txt');
    const notice = read('src-tauri/installer-notice.txt');
    expect(notice).toContain('NIET GEATTESTEERD');
    expect(notice).toContain('NOT ATTESTED');
    // Plain ASCII, so the setup shows it the same in every code page.
    expect(/^[\x20-\x7e\n]*$/.test(notice)).toBe(true);
  });
});

describe('how the app reaches the kernel', () => {
  it('describes WebAssembly in the browser and HTTP only as the development fallback', () => {
    const readme = read('README.md');
    expect(readme).toContain('WebAssembly kernel');
    expect(readme).not.toContain('The browser build calculates through the local HTTP API');
    const install = read('wiki/Installatie.md');
    expect(install).toContain('WebAssembly');
    expect(install).not.toContain('De webversie rekent via de lokale HTTP-API');
  });

  it('no longer says that no real Uniec 3 file was tested', () => {
    for (const path of ['wiki/Bekende-beperkingen.md', 'wiki/Import-en-export.md']) {
      expect(read(path), path).not.toContain('er is nog geen echt voorbeeldbestand getest');
    }
  });
});
