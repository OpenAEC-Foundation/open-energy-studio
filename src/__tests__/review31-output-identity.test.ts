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
import { setVersion } from '../../scripts/bump-version.js';

const root = resolve(__dirname, '../..');
const read = (path: string) => readFileSync(resolve(root, path), 'utf8');
const load = (name: string) => ({
  project: JSON.parse(read(`training-data/nta8800-example-${name}.json`)) as IProject,
  assessment: JSON.parse(read(`training-data/nta8800-example-${name}.kernel-output.json`)) as ProjectPerformanceAssessment,
});
const at = new Date('2026-10-10T10:00:00Z');

function programVersions() {
  const lock = /name = "open-energy-studio"\nversion = "([^"]+)"/.exec(read('src-tauri/Cargo.lock'));
  return {
    packageJson: (JSON.parse(read('package.json')) as { version: string }).version,
    tauriConf: (JSON.parse(read('src-tauri/tauri.conf.json')) as { version: string }).version,
    cargoToml: /^version = "([^"]+)"/m.exec(read('src-tauri/Cargo.toml'))?.[1],
    cargoLock: lock?.[1],
  };
}

describe('one program version on every output', () => {
  it('is the same in package.json, the desktop package config, Cargo.toml and Cargo.lock', () => {
    const versions = programVersions();
    expect(new Set(Object.values(versions)).size, JSON.stringify(versions)).toBe(1);
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
