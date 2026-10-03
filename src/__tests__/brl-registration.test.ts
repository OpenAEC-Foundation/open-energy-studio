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
    };
    recordBagRegistration(first, store);
    recordBagRegistration({ ...first, projectName: 'Woning A (herzien)' }, store);
    expect(readBagLedger(store)).toHaveLength(1);
    const second = { ...first, projectId: 'p2', projectName: 'Woning B' };
    expect(bagConflicts(second, readBagLedger(store)).map((item) => item.projectId)).toEqual(['p1']);
    expect(bagConflicts({ ...second, messageType: 'relabel' }, readBagLedger(store))).toEqual([]);
    expect(bagConflicts({ ...second, residential: false }, readBagLedger(store))).toEqual([]);
    expect(bagConflicts({ ...second, bagObjectId: '0363010000000002' }, readBagLedger(store))).toEqual([]);
    expect(bagConflicts(first, readBagLedger(store))).toEqual([]);
    expect(readBagLedger({ getItem: () => 'not json', setItem: () => {} })).toEqual([]);
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
  });

  it('notes missing registration data', () => {
    const assessment = {
      status: 'incomplete', targetNormVersion: 'x', kernelVersion: 'y', inputFingerprint: 'z',
      attestStatus: 'unattested', gaps: [], geometry: null, derivedInput: null, performance: null,
    } as unknown as ProjectPerformanceAssessment;
    expect(generateNtaCalculationReportHTML(createDefaultProject(), assessment)).toContain('Geen registratiegegevens ingevuld');
  });
});
