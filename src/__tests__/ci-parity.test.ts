import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

// CI must run the gate the way a fresh local checkout does, so that CI cannot
// hide a gate bug (the frontend bundle used to be built by CI before the gate,
// which hid that the gate needed dist/ before its Tauri check).
const root = join(__dirname, '../..');
const read = (path: string) => readFileSync(join(root, path), 'utf8');

/** The step lines of one job, from `  <job>:` to the next job. */
function job(workflow: string, name: string): string {
  const start = workflow.indexOf(`\n  ${name}:\n`);
  expect(start).toBeGreaterThan(-1);
  const rest = workflow.slice(start + 1);
  const next = rest.slice(1).search(/\n {2}[a-z][a-z-]*:\n/);
  return next < 0 ? rest : rest.slice(0, next + 1);
}

describe('CI runs the same gate as a local checkout', () => {
  for (const [file, gateJob] of [
    ['.github/workflows/ci.yml', 'gate'],
    ['.github/workflows/release.yml', 'verify'],
  ] as const) {
    it(`${file}: the ${gateJob} job builds the frontend only through scripts/verify-nta.sh`, () => {
      const steps = job(read(file), gateJob);
      expect(steps).toContain('run: scripts/verify-nta.sh');
      expect(steps).not.toMatch(/run: npm run build/);
      expect(steps).not.toMatch(/NTA_SKIP_MSRV/);
      expect(steps).toContain('rustup toolchain install 1.77.2');
    });
  }

  it('the release workflow checks the released notes against the previous tag, as release-nta.sh does', () => {
    const verify = job(read('.github/workflows/release.yml'), 'verify');
    expect(verify).toContain('check-released');
    expect(read('scripts/release-nta.sh')).toContain('check-released');
  });

  it('the release workflow keeps assets that are already on the release', () => {
    const build = job(read('.github/workflows/release.yml'), 'build');
    expect(build).toContain('Keeping existing release asset');
    expect(build).not.toMatch(/gh release upload[^\n]*--clobber/);
  });

  it('release-nta.sh writes the GitHub release text', () => {
    expect(read('scripts/release-nta.sh')).toContain('scripts/nta-github-release.mjs --out "$archive/github-release.md"');
  });

  it('release-nta.sh stamps the testset dossier with the commit as the gate does, dirty marker included', () => {
    const script = read('scripts/release-nta.sh');
    const step = script.slice(script.indexOf('testsetdossier ISSO 54'), script.indexOf('packages=()'));
    expect(step).toContain('--commit "$testset_commit"');
    expect(step).toContain('(met lokale wijzigingen)');
    expect(read('scripts/verify-nta.sh')).toContain('(met lokale wijzigingen)');
  });

  it('both workflows pin the same version of every action they share', () => {
    const pins = (workflow: string) => {
      const found = new Map<string, Set<string>>();
      for (const [, action, version] of workflow.matchAll(/uses:\s*([^@\s]+)@(\S+)/g)) {
        found.set(action, (found.get(action) ?? new Set()).add(version));
      }
      return found;
    };
    const ci = pins(read('.github/workflows/ci.yml'));
    const release = pins(read('.github/workflows/release.yml'));
    const shared = [...ci.keys()].filter((action) => release.has(action));
    expect(shared).toEqual(expect.arrayContaining(['actions/checkout', 'actions/setup-node', 'azure/trusted-signing-action']));
    for (const action of shared) {
      // The action name rides along so a failure names the action that differs.
      const versions = [...new Set([...ci.get(action)!, ...release.get(action)!])];
      expect([action, versions]).toEqual([action, [...ci.get(action)!].slice(0, 1)]);
    }
  });

  it('the signing step uses the input names of the pinned trusted-signing-action major', () => {
    for (const file of ['.github/workflows/ci.yml', '.github/workflows/release.yml']) {
      const workflow = read(file);
      expect(workflow).toContain('azure/trusted-signing-action@v1');
      expect(workflow).toContain('signing-account-name:');
      expect(workflow).not.toContain('trusted-signing-account-name:');
    }
  });
});
