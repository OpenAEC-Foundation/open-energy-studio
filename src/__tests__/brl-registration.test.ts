import { describe, expect, it } from 'vitest';
import { createDefaultProject } from '../context/EnergyContext';
import { compareKernelStamp, deserializeProject, deserializeProjectFile, serializeProject } from '../core/io/ProjectSerializer';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import {
  bagConflicts, cleanRegistration, readBagLedger, recordBagRegistration, SOFTWARE_NAME, softwareIdentity,
} from '../core/nta/Registration';
import type { BagLedgerEntry } from '../core/nta/Registration';
import { version } from '../../package.json';
import type { ProjectPerformanceAssessment } from '../core/nta/KernelClient';

const stamp = { kernelVersion: '0.1.0', targetNormVersion: 'NTA 8800:2025+C1:2026', inputFingerprint: 'sha256:aa' };

describe('project file kernel stamp', () => {
  it('round-trips the stamp and stays compatible with old files', () => {
    const project = createDefaultProject();
    const json = serializeProject(project, stamp);
    expect(deserializeProjectFile(json)).toEqual({ project, kernel: stamp });
    expect(deserializeProject(json)).toEqual(project);
    const old = serializeProject(project);
    expect(JSON.parse(old).kernel).toBeUndefined();
    expect(deserializeProjectFile(old).kernel).toBeUndefined();
  });

  it('reports a version change before an input change', () => {
    expect(compareKernelStamp(stamp, { ...stamp })).toEqual([]);
    expect(compareKernelStamp(stamp, { ...stamp, kernelVersion: '0.2.0', inputFingerprint: 'sha256:bb' })).toEqual(['version']);
    expect(compareKernelStamp(stamp, { ...stamp, targetNormVersion: 'NTA 8800:2027' })).toEqual(['version']);
    expect(compareKernelStamp(stamp, { ...stamp, inputFingerprint: 'sha256:bb' })).toEqual(['input']);
    expect(compareKernelStamp(undefined, stamp)).toEqual([]);
    expect(compareKernelStamp(stamp, null)).toEqual([]);
  });
});

describe('registration block', () => {
  it('drops empty values so an untouched block is not saved', () => {
    expect(cleanRegistration({})).toBeUndefined();
    expect(cleanRegistration({ client: ' ', relabel: false, surveyingAdvisor: { name: '', competenceNumber: ' ' } })).toBeUndefined();
    expect(cleanRegistration({ client: ' Eigenaar ', serialProject: true, constructionYear: 1930 }))
      .toEqual({ client: 'Eigenaar', serialProject: true, constructionYear: 1930, software: softwareIdentity() });
  });

  it('records the program and cleans the WLC-GWP block (Regeling art. 5, BRL 9500-W p. 21)', () => {
    expect(softwareIdentity()).toEqual({ name: SOFTWARE_NAME, version });
    expect(cleanRegistration({ wlcGwp: { reportReference: ' ' } })).toBeUndefined();
    expect(cleanRegistration({ messageType: 'replacement', wlcGwp: { valueKgCo2EqPerM2Year: 7.5, reportReference: ' wlc.pdf ' } }))
      .toEqual({ messageType: 'replacement', wlcGwp: { valueKgCo2EqPerM2Year: 7.5, reportReference: 'wlc.pdf' }, software: softwareIdentity() });
  });

  it('flags a second residential label on the same BAG object (Praktijkhandboek p. 46)', () => {
    const memory = new Map<string, string>();
    const store = { getItem: (key: string) => memory.get(key) ?? null, setItem: (key: string, value: string) => { memory.set(key, value); } };
    const first: BagLedgerEntry = {
      bagObjectId: '0363010000000001', projectId: 'p1', projectName: 'Woning A', residential: true, messageType: 'regular',
      epOnlineNumber: 'EP-1', surveyDate: '2026-01-31',
    };
    // A draft or scenario copy without an EP-Online number is no label.
    recordBagRegistration({ ...first, projectId: 'draft', epOnlineNumber: undefined }, store);
    expect(readBagLedger(store)).toEqual([]);
    recordBagRegistration(first, store);
    recordBagRegistration({ ...first, projectName: 'Woning A (herzien)' }, store);
    expect(readBagLedger(store)).toHaveLength(1);
    const second = { ...first, projectId: 'p2', projectName: 'Woning B' };
    expect(bagConflicts(second, readBagLedger(store)).map((item) => item.projectId)).toEqual(['p1']);
    expect(bagConflicts({ ...second, messageType: 'relabel' }, readBagLedger(store))).toEqual([]);
    expect(bagConflicts({ ...second, residential: false }, readBagLedger(store))).toEqual([]);
    expect(bagConflicts({ ...second, bagObjectId: '0363010000000002' }, readBagLedger(store))).toEqual([]);
    expect(bagConflicts(first, readBagLedger(store))).toEqual([]);
    // One label per object at a time: after ten years the old one expired.
    expect(bagConflicts({ ...second, surveyDate: '2036-02-01' }, readBagLedger(store))).toEqual([]);
    expect(bagConflicts({ ...second, surveyDate: '2036-01-31' }, readBagLedger(store))).toHaveLength(1);
    // Clearing the BAG id (or the number) removes the project's entry.
    recordBagRegistration({ ...first, bagObjectId: '' }, store);
    expect(readBagLedger(store)).toEqual([]);
    expect(readBagLedger({ getItem: () => 'not json', setItem: () => {} })).toEqual([]);
  });

  it('keeps the program of the original calculation on a relabel (BRL 9500-W §4.2.4)', () => {
    const original = { name: SOFTWARE_NAME, version: '0.1.0-alpha' };
    expect(cleanRegistration({ messageType: 'relabel', client: 'X', software: original })?.software).toEqual(original);
    expect(cleanRegistration({ relabel: true, client: 'X', software: original })?.software).toEqual(original);
    expect(cleanRegistration({ messageType: 'regular', client: 'X', software: original })?.software).toEqual(softwareIdentity());
    expect(cleanRegistration({ messageType: 'relabel', client: 'X' })?.software).toEqual(softwareIdentity());
    // A replacement is a new calculation with the current version (p. 23).
    expect(cleanRegistration({ messageType: 'replacement', client: 'X', software: original })?.software).toEqual(softwareIdentity());
  });

  it('prints advisers, dates, validity and label data in the report', () => {
    const project = {
      ...createDefaultProject(),
      registration: {
        purpose: 'existing_building' as const,
        surveyingAdvisor: { name: 'A. Opnemer', competenceNumber: 'V-1' },
        registeringAdvisor: { name: 'B. Registreerder', competenceNumber: 'V-2' },
        surveyDate: '2026-03-15',
        registrationDate: '2026-04-01',
        software: { name: 'Open Energy Studio', version: '0.1.6-alpha' },
        wlcGwp: { valueKgCo2EqPerM2Year: 7.5, reportReference: 'wlc.pdf' },
      },
    };
    const assessment = {
      status: 'incomplete', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
      inputFingerprint: 'sha256:feed', attestStatus: 'unattested', gaps: [], geometry: null,
      derivedInput: null, performance: null,
      registration: {
        source: 'BRL 9500', validUntil: '2036-03-15', registrationDeadline: '2026-06-15', relabelDeadline: null,
        readyForRegistration: false, issues: [{ code: 'client_required', path: 'registration.client', severity: 'missing' }],
        messageType: 'regular', wlcGwpRequired: true,
        plausibility: [{ code: 'plausibility_borderline_label', path: 'performance', severity: 'warning' }],
      },
      labelData: null,
    } as unknown as ProjectPerformanceAssessment;
    const html = generateNtaCalculationReportHTML(project, assessment);
    expect(html).toContain('A. Opnemer (vakbekwaamheid V-1)');
    expect(html).toContain('B. Registreerder (vakbekwaamheid V-2)');
    expect(html).toContain('2036-03-15');
    expect(html).toContain('2026-06-15');
    expect(html).toContain('client_required');
    expect(html).toContain('bestaand gebouw');
    expect(html).toContain('Open Energy Studio 0.1.6-alpha, nog niet geattesteerd (BRL 9501)');
    expect(html).toContain('7.50 kg CO₂-eq/m²·jr (wlc.pdf)');
    expect(html).toContain('plausibility_borderline_label');
    expect(html).toContain('registratie');
    // Dossier state and readiness are shown apart (Regeling art. 2/3).
    expect(html).toContain('<th>Dossier compleet</th><td>nee</td>');
    const attestMissing = {
      ...assessment,
      registration: { ...assessment.registration!, issues: [], dossierComplete: true, softwareAttested: false, readyForRegistration: false },
    } as ProjectPerformanceAssessment;
    const ready = generateNtaCalculationReportHTML(project, attestMissing);
    expect(ready).toContain('<th>Dossier compleet</th><td>ja</td>');
    expect(ready).toContain('nee, rekenprogramma nog niet geattesteerd (BRL 9501)');
  });

  it('notes missing registration data', () => {
    const assessment = {
      status: 'incomplete', targetNormVersion: 'x', kernelVersion: 'y', inputFingerprint: 'z',
      attestStatus: 'unattested', gaps: [], geometry: null, derivedInput: null, performance: null,
    } as unknown as ProjectPerformanceAssessment;
    expect(generateNtaCalculationReportHTML(createDefaultProject(), assessment)).toContain('Geen registratiegegevens ingevuld');
  });
});
