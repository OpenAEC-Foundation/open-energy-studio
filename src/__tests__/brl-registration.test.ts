import { describe, expect, it } from 'vitest';
import { createDefaultProject } from '../context/EnergyContext';
import { compareKernelStamp, deserializeProject, deserializeProjectFile, serializeProject } from '../core/io/ProjectSerializer';
import { generateNtaCalculationReportHTML } from '../core/report/NtaCalculationReport';
import { cleanRegistration } from '../core/nta/Registration';
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
      .toEqual({ client: 'Eigenaar', serialProject: true, constructionYear: 1930 });
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
      },
    };
    const assessment = {
      status: 'incomplete', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
      inputFingerprint: 'sha256:feed', attestStatus: 'unattested', gaps: [], geometry: null,
      derivedInput: null, performance: null,
      registration: {
        source: 'BRL 9500', validUntil: '2036-03-15', registrationDeadline: '2026-06-15', relabelDeadline: null,
        readyForRegistration: false, issues: [{ code: 'client_required', path: 'registration.client', severity: 'missing' }],
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
  });

  it('notes missing registration data', () => {
    const assessment = {
      status: 'incomplete', targetNormVersion: 'x', kernelVersion: 'y', inputFingerprint: 'z',
      attestStatus: 'unattested', gaps: [], geometry: null, derivedInput: null, performance: null,
    } as unknown as ProjectPerformanceAssessment;
    expect(generateNtaCalculationReportHTML(createDefaultProject(), assessment)).toContain('Geen registratiegegevens ingevuld');
  });
});
