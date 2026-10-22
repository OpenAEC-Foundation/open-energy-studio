/**
 * Review 31 of the BRL output requirements: the program version printed on every output is the
 * one the desktop package, the Cargo lockfile and the leveringsdocument carry; the attest
 * placeholder is the same text everywhere; adviser data in the reports is escaped; the
 * Standaard voor Woningisolatie row is for dwellings only.
 */
import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import type { IProject } from '../core/energy/types';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import { generateEnergyPerformanceReportHTML } from '../core/report/EnergyPerformanceReport';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import { generateNtaInputDossierHTML } from '../core/report/NtaInputDossier';
import { ATTEST_NOT_YET, programIdentity } from '../core/report/ProgramIdentity';
import { readIdentity, renderLeveringsdocument } from '../../scripts/nta-leveringsdocument.mjs';
import { isoWeek, msiVersion, setVersion, weekVersion } from '../../scripts/bump-version.js';

const root = resolve(__dirname, '../..');
// A Windows checkout has CRLF line endings (core.autocrlf); the checks read the content.
const read = (path: string) => readFileSync(resolve(root, path), 'utf8').replace(/\r\n/g, '\n');
const load = (name: string) => ({
  project: JSON.parse(read(`training-data/nta8800-example-${name}.json`)) as IProject,
  assessment: JSON.parse(read(`training-data/nta8800-example-${name}.kernel-output.json`)) as ProjectPerformanceAssessment,
});
const at = new Date('2026-10-10T10:00:00Z');

function programVersions() {
  const lock = /name = "open-energy-studio"\nversion = "([^"]+)"/.exec(read('src-tauri/Cargo.lock'));
  const npmLock = JSON.parse(read('package-lock.json')) as { version: string; packages: Record<string, { version: string }> };
  return {
    packageJson: (JSON.parse(read('package.json')) as { version: string }).version,
    packageLock: npmLock.version,
    packageLockRoot: npmLock.packages[''].version,
    tauriConf: (JSON.parse(read('src-tauri/tauri.conf.json')) as { version: string }).version,
    cargoToml: /^version = "([^"]+)"/m.exec(read('src-tauri/Cargo.toml'))?.[1],
    cargoLock: lock?.[1],
  };
}

describe('one program version on every output', () => {
  it('is the same in package.json, package-lock.json, the desktop package config, Cargo.toml and Cargo.lock', () => {
    const versions = programVersions();
    expect(new Set(Object.values(versions)).size, JSON.stringify(versions)).toBe(1);
  });

  it('gives the Windows MSI the short form of the program version', () => {
    const conf = JSON.parse(read('src-tauri/tauri.conf.json')) as { version: string; bundle?: { windows?: { wix?: { version?: string } } } };
    const msi = msiVersion(conf.version);
    expect(conf.bundle?.windows?.wix?.version ?? conf.version).toBe(msi);
    expect(Number(msi.split('.')[0])).toBeLessThanOrEqual(255);
  });

  it('is what the reports print and what the leveringsdocument stamps', () => {
    const delivery = readIdentity(root);
    const identity = programIdentity(null, null);
    expect(identity.programVersion).toBe(delivery.programVersion);
    expect(identity.name).toBe(delivery.softwareName);
    expect(identity.attestNumber ?? '').toBe(delivery.attestNumber);
  });

  it('states a missing attest number with the same words in the leveringsdocument', () => {
    const text = renderLeveringsdocument('{{attestNumber}}', {
      ...readIdentity(root), attestNumber: '', releaseDate: '2026-10-10', packages: [], manual: [],
    });
    expect(text).toBe(ATTEST_NOT_YET);
  });

  it('bumps every place that carries the program version, the lockfile included', () => {
    const files = {
      'package.json': '{\n  "name": "x",\n  "version": "0.2.0"\n}\n',
      'src-tauri/tauri.conf.json': '{\n  "productName": "x",\n  "version": "0.2.0"\n}\n',
      'src-tauri/Cargo.toml': '[package]\nname = "open-energy-studio"\nversion = "0.2.0"\n',
      'src-tauri/Cargo.lock': '[[package]]\nname = "aho"\nversion = "0.2.0"\n\n[[package]]\nname = "open-energy-studio"\nversion = "0.2.0"\n',
    };
    const out = setVersion(files, '0.3.0');
    expect(JSON.parse(out['package.json']).version).toBe('0.3.0');
    expect(JSON.parse(out['src-tauri/tauri.conf.json']).version).toBe('0.3.0');
    expect(out['src-tauri/Cargo.toml']).toContain('version = "0.3.0"');
    expect(out['src-tauri/Cargo.lock']).toContain('name = "open-energy-studio"\nversion = "0.3.0"');
    // Another package that happens to share the old version stays as it is.
    expect(out['src-tauri/Cargo.lock']).toContain('name = "aho"\nversion = "0.2.0"');
    // A semver version is a valid MSI version: no separate wix version.
    expect(JSON.parse(out['src-tauri/tauri.conf.json']).bundle?.windows?.wix?.version).toBeUndefined();
  });

  it('bumps to a week version: the lockfile of npm and the MSI version follow', () => {
    const files = {
      'package.json': '{\n  "name": "x",\n  "version": "2026.43.0"\n}\n',
      'package-lock.json': '{\n  "name": "x",\n  "version": "2026.43.0",\n  "packages": {\n    "": {\n      "version": "2026.43.0"\n    },\n    "node_modules/a": {\n      "version": "2026.43.0"\n    }\n  }\n}\n',
      'src-tauri/tauri.conf.json': '{\n  "version": "2026.43.0",\n  "bundle": {\n    "windows": {\n      "wix": {\n        "language": "nl-NL",\n        "version": "26.43.0"\n      }\n    }\n  }\n}\n',
      'src-tauri/Cargo.toml': '[package]\nname = "open-energy-studio"\nversion = "2026.43.0"\n',
      'src-tauri/Cargo.lock': '[[package]]\nname = "open-energy-studio"\nversion = "2026.43.0"\n',
    };
    const out = setVersion(files, '2026.44.1');
    const lock = JSON.parse(out['package-lock.json']);
    expect(lock.version).toBe('2026.44.1');
    expect(lock.packages[''].version).toBe('2026.44.1');
    expect(lock.packages['node_modules/a'].version).toBe('2026.43.0');
    const wix = JSON.parse(out['src-tauri/tauri.conf.json']).bundle.windows.wix;
    expect(wix).toEqual({ language: 'nl-NL', version: '26.44.1' });
  });
});

describe('the automatic program version year.week.n (issue #10)', () => {
  it('takes the ISO week, also around the turn of the year', () => {
    expect(isoWeek(new Date('2026-10-22T12:00:00Z'))).toEqual({ year: 2026, week: 43 });
    expect(isoWeek(new Date('2026-10-26T00:00:00Z'))).toEqual({ year: 2026, week: 44 });
    // 1 January 2027 is a Friday: still week 53 of 2026.
    expect(isoWeek(new Date('2027-01-01T12:00:00Z'))).toEqual({ year: 2026, week: 53 });
    // 29 December 2025 is a Monday in week 1 of 2026.
    expect(isoWeek(new Date('2025-12-29T12:00:00Z'))).toEqual({ year: 2026, week: 1 });
  });

  it('numbers the releases of a week from 0, after the existing tags', () => {
    const date = new Date('2026-10-22T12:00:00Z');
    expect(weekVersion(date, ['v0.2.0', 'oes-v0.3.0-kernel-v0.4.0'])).toBe('2026.43.0');
    expect(weekVersion(date, ['v2026.43.0'])).toBe('2026.43.1');
    expect(weekVersion(date, ['v2026.43.0', 'oes-v2026.43.4-kernel-v0.4.0', 'v2026.42.7'])).toBe('2026.43.5');
    // Week 4 must not take the tags of week 43.
    expect(weekVersion(new Date('2026-01-22T12:00:00Z'), ['v2026.43.2'])).toBe('2026.4.0');
  });

  it('gives an MSI version with a major version an MSI accepts', () => {
    expect(msiVersion('2026.43.0')).toBe('26.43.0');
    expect(msiVersion('2026.4.12')).toBe('26.4.12');
    expect(msiVersion('0.3.0')).toBe('0.3.0');
  });
});

describe('adviser data in the outputs is escaped', () => {
  const hostile = '<img src=x onerror=alert(1)>';
  const withAdvisers = (project: IProject): IProject => ({
    ...project,
    registration: {
      ...(project.registration ?? {}),
      surveyDate: '2026-10-01',
      surveyingAdvisor: { name: `Opnemer ${hostile}`, competenceNumber: `V-1 ${hostile}` },
      registeringAdvisor: { name: `Registreerder ${hostile}`, competenceNumber: `V-2 ${hostile}` },
    } as IProject['registration'],
  });

  it('in the energy performance report, the calculation report and the input dossier', () => {
    const { project, assessment } = load('terraced-dwelling');
    const hostileProject = withAdvisers(project);
    const outputs = [
      ...(['summary', 'standard', 'detailed'] as const).map((level) =>
        generateEnergyPerformanceReportHTML(hostileProject, assessment, { level, generatedAt: at, interpretations: [] })),
      generateNtaCalculationReportHTML(hostileProject, assessment, [], at),
      generateNtaInputDossierHTML(hostileProject),
    ];
    for (const html of outputs) {
      expect(html).not.toContain('<img src=x');
    }
    expect(outputs.some((html) => html.includes('Opnemer &lt;img'))).toBe(true);
  });
});

describe('Standaard voor Woningisolatie', () => {
  it('is stated for a dwelling and not for a utility building', () => {
    const dwelling = load('terraced-dwelling');
    const office = load('office');
    for (const level of ['summary', 'standard', 'detailed'] as const) {
      expect(generateEnergyPerformanceReportHTML(dwelling.project, dwelling.assessment, { level, generatedAt: at, interpretations: [] }))
        .toContain('Standaard voor Woningisolatie');
      expect(generateEnergyPerformanceReportHTML(office.project, office.assessment, { level, generatedAt: at, interpretations: [] }))
        .not.toContain('Standaard voor Woningisolatie');
    }
  });
});
