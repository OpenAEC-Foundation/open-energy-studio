import { describe, expect, it, beforeEach } from 'vitest';
import { strFromU8, unzipSync } from 'fflate';
import type { IProject } from '../core/energy/types';
import type { NtaEvidenceItem, ProjectPerformanceAssessment } from '../core/nta/KernelClient';
import {
  clearSessionEvidence, createEvidenceItem, guessEvidenceKind, nextEvidenceId, sha256Hex,
} from '../core/nta/Evidence';
import { cleanRegistration, softwareIdentity } from '../core/nta/Registration';
import { buildProjectDossier, checkDossierCompleteness, zipProjectDossier } from '../core/report/ProjectDossier';
import { parseGps } from '../components/dialogs/ProjectInfoDialog/EvidenceRegister';

function project(registration: IProject['registration'] = undefined): IProject {
  return {
    id: 'p1', name: 'Woning', description: '', buildingFunction: 'residential', address: '', city: '',
    zones: [], heatingSystems: [], ventilationSystems: [], solarPV: [], solarThermal: [], constructions: [],
    ...(registration ? { registration } : {}),
  } as unknown as IProject;
}

const evidence = (id: string, kind: NtaEvidenceItem['kind'], checkedBy?: string): NtaEvidenceItem => ({
  id, kind, fileName: `${id}.pdf`, sha256: 'a'.repeat(64), ...(checkedBy ? { checkedBy } : {}),
});

const assessment = {
  status: 'calculated_unverified', targetNormVersion: 'NTA 8800:2025+C1:2026', kernelVersion: '0.1.0',
  inputFingerprint: 'abc', attestStatus: 'unattested', gaps: [],
  geometry: { usableFloorAreaM2: 1500, lossAreaM2: 0, envelopeAreaM2: 0, lossAreaRatio: null, unclassifiedSurfaceCount: 0 },
  derivedInput: null, performance: null,
} as unknown as ProjectPerformanceAssessment;

describe('evidence register', () => {
  beforeEach(() => clearSessionEvidence());

  it('hashes files with SHA-256 and numbers them', async () => {
    expect(await sha256Hex(new TextEncoder().encode('abc')))
      .toBe('ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
    const item = await createEvidenceItem({ name: 'factuur-ketel.pdf', bytes: new TextEncoder().encode('x') }, []);
    expect(item.id).toBe('ev-1');
    expect(item.kind).toBe('invoice');
    expect(nextEvidenceId([item, { ...item, id: 'ev-2' }])).toBe('ev-3');
    expect(guessEvidenceKind('IMG_0001.JPG')).toBe('photo_detail');
  });

  it('parses GPS only when both coordinates are numbers', () => {
    expect(parseGps('52.37, 4.89')).toEqual({ latitude: 52.37, longitude: 4.89 });
    expect(parseGps('52.37')).toBeUndefined();
    expect(parseGps('abc, 4')).toBeUndefined();
  });

  it('keeps evidence and set triggers when cleaning the registration', () => {
    const cleaned = cleanRegistration({
      evidence: [evidence('ev-1', 'invoice')],
      detailSurveyTriggers: { energyPerformanceFee: true, addedAfter2021: false },
      surveyingAdvisor: { name: '', competenceNumber: '' },
    });
    expect(cleaned).toEqual({
      evidence: [evidence('ev-1', 'invoice')],
      detailSurveyTriggers: { energyPerformanceFee: true },
      software: softwareIdentity(),
    });
    expect(cleanRegistration({ evidence: [], detailSurveyTriggers: { addedAfter2021: false } })).toBeUndefined();
  });
});

describe('project dossier', () => {
  beforeEach(() => clearSessionEvidence());

  it('varies the Bijlage 3 checklist with purpose, evidence and relabel', () => {
    const empty = checkDossierCompleteness({ project: project() });
    expect(empty.find((item) => item.id === 'address')?.status).toBe('missing');
    expect(empty.find((item) => item.id === 'quality_declarations')?.status).toBe('not_applicable');
    expect(empty.some((item) => item.group === 'relabel')).toBe(false);

    const bbl = checkDossierCompleteness({
      project: project({
        purpose: 'bbl_check', surveyType: 'detailed', postcode: '1011AB', houseNumber: '1',
        registrationDate: '2028-03-01',
        evidence: [evidence('ev-1', 'drawing', 'A'), evidence('ev-2', 'datasheet')],
      }),
      assessment,
    });
    const status = (id: string) => bbl.find((item) => item.id === id)?.status;
    expect(status('address')).toBe('ok');
    expect(status('installation_types')).toBe('ok');
    expect(status('quality_declarations')).toBe('missing');
    expect(status('photos')).toBe('not_applicable');
    expect(status('evidence_checked')).toBe('missing');
    expect(status('output_file')).toBe('ok');
    expect(status('wlc_gwp')).toBe('check');
    expect(status('delivered_report')).toBe('ok');
    expect(status('electronic_files')).toBe('ok');
    expect(status('gto_cooling_load')).toBe('check');
    expect(status('product_documentation')).toBe('ok');
    expect(status('forfait_justification')).toBe('check');
    expect(status('apartment_labels')).toBe('check');
    expect(status('entered_areas')).toBe('missing');

    // WLC-GWP: only for toets Bbl from 1-1-2028 (BRL 9500-W p. 21).
    const before2028 = checkDossierCompleteness({
      project: project({ purpose: 'bbl_check', registrationDate: '2027-12-31' }),
      assessment,
    });
    expect(before2028.some((item) => item.id === 'wlc_gwp')).toBe(false);

    // A reference object needs linked representativity evidence.
    const reference = (linked: boolean) => checkDossierCompleteness({
      project: project({
        representation: 'reference',
        evidence: [{ ...evidence('ev-3', 'other', 'A'), ...(linked ? { linkedPaths: ['/registration/representation'] } : {}) }],
      }),
    }).find((item) => item.id === 'representativity')?.status;
    expect(reference(false)).toBe('missing');
    expect(reference(true)).toBe('ok');

    const relabel = checkDossierCompleteness({
      project: project({ relabel: true, evidence: [{ ...evidence('ev-1', 'invoice', 'A'), relabelProof: 'specified_invoice' }] }),
      relabel: {
        source: 'x', scheme: 'w', allowed: false, needsReview: true,
        changes: [{ path: '/pvSystems/0', before: null, after: {}, verdict: 'review', cluster: 'x' }],
      },
    });
    const relabelStatus = (id: string) => relabel.find((item) => item.id === id)?.status;
    expect(relabelStatus('relabel_changes')).toBe('missing');
    expect(relabelStatus('relabel_review')).toBe('check');
    expect(relabelStatus('relabel_invoice')).toBe('ok');
    expect(relabelStatus('relabel_improvement_date')).toBe('missing');
    expect(relabelStatus('relabel_production_photos')).toBe('missing');
    // The message type replaces the old `relabel` flag (BRL 9500-W p. 24).
    const typed = checkDossierCompleteness({
      project: project({ messageType: 'relabel', evidence: [evidence('ev-1', 'invoice', 'A')] }),
    });
    expect(typed.some((item) => item.id === 'relabel_invoice')).toBe(true);
    const replacement = checkDossierCompleteness({ project: project({ messageType: 'replacement' }) });
    expect(replacement.some((item) => item.group === 'relabel')).toBe(false);
  });

  it('takes the relabel comparison kept with the registration and checks the relabel proof (BRL 9500-W §4.2.3)', async () => {
    const assessmentW = {
      source: 'x', scheme: 'w' as const, allowed: true, needsReview: false,
      changes: [{ path: '/solarPV/0/area', before: 10, after: 14, verdict: 'review' as const, cluster: 'x' }],
    };
    const base = project({
      messageType: 'relabel', certificateNumber: 'K1', originalCertificateNumber: 'k1', originalEpOnlineNumber: 'EP-1',
      surveyDate: '2026-01-31', improvementDate: '2028-02-01',
      evidence: [evidence('ev-1', 'invoice', 'A')],
    });
    const { labelInputSha256 } = await import('../core/nta/Registration');
    base.registration!.relabelComparison = {
      originalFileName: 'origineel.oes.json', currentSha256: await labelInputSha256(base), assessment: assessmentW,
    };
    const status = (items: ReturnType<typeof checkDossierCompleteness>, id: string) =>
      items.find((item) => item.id === id);
    let items = checkDossierCompleteness({ project: base, labelInputSha256: await labelInputSha256(base) });
    expect(status(items, 'relabel_changes')?.status).toBe('ok');
    expect(status(items, 'relabel_original_label')?.status).toBe('ok');
    // An invoice without the relabel role does not prove the improvement.
    expect(status(items, 'relabel_invoice')?.status).toBe('missing');
    // 2028-02-01 lies after 31-01-2028, the end of the 24 months.
    expect(status(items, 'relabel_improvement_date')?.status).toBe('missing');
    expect(status(items, 'relabel_improvement_date')?.detail).toContain('2028-01-31');
    expect(status(items, 'relabel_production_photos')?.status).toBe('missing');
    expect(status(items, 'relabel_production_connection')?.status).toBe('missing');
    expect(status(items, 'relabel_utility_confirmation')?.status).toBe('not_applicable');

    base.registration!.improvementDate = '2028-01-31';
    base.registration!.productionPhysicallyConnected = true;
    base.registration!.evidence = [
      { ...evidence('ev-1', 'invoice', 'A'), relabelProof: 'specified_invoice' },
      { ...evidence('ev-2', 'photo_overview', 'A'), relabelProof: 'production_photo' },
    ];
    items = checkDossierCompleteness({ project: base, labelInputSha256: await labelInputSha256(base) });
    for (const id of ['relabel_invoice', 'relabel_improvement_date', 'relabel_production_photos', 'relabel_production_connection']) {
      expect(status(items, id)?.status, id).toBe('ok');
    }
    // Another certificate holder may not relabel.
    base.registration!.originalCertificateNumber = 'K2';
    items = checkDossierCompleteness({ project: base });
    expect(status(items, 'relabel_original_label')?.status).toBe('missing');

    // An edit after the comparison makes it out of date.
    const edited = { ...base, name: 'Woning na verbouwing', zones: [{ id: 'z1' }] } as unknown as IProject;
    items = checkDossierCompleteness({ project: edited, labelInputSha256: await labelInputSha256(edited) });
    expect(status(items, 'relabel_changes')?.status).toBe('check');

    // The dossier export writes the kept comparison.
    const bundle = await buildProjectDossier({ project: base });
    const record = JSON.parse(strFromU8(bundle.files['herlabel-vergelijking.json']));
    expect(record.originalFileName).toBe('origineel.oes.json');
    expect(record.assessment.changes).toHaveLength(1);
  });

  it('keeps the relabel comparison and stated answers when cleaning the registration', () => {
    const comparison = {
      originalFileName: 'o.oes.json', assessment: { source: 'x', scheme: 'u' as const, allowed: true, needsReview: false, changes: [] },
    };
    const cleaned = cleanRegistration({
      messageType: 'relabel', relabelComparison: comparison, productionPhysicallyConnected: false, noExcludedChangesConfirmed: true,
    });
    expect(cleaned?.relabelComparison).toEqual(comparison);
    expect(cleaned?.productionPhysicallyConnected).toBe(false);
    expect(cleaned?.noExcludedChangesConfirmed).toBe(true);
  });

  it('requires a collapse reason for every applied default of a basic survey', () => {
    const items = checkDossierCompleteness({
      project: project({ surveyType: 'basic' }),
      opname: {
        appliedDefaults: [
          { rule: 'a', path: 'x', value: '1', source: 's', inklapReden: 'niet zichtbaar' },
          { rule: 'b', path: 'y', value: '1', source: 's' },
        ],
      } as never,
    });
    const item = items.find((entry) => entry.id === 'collapse_reasons');
    expect(item?.status).toBe('missing');
    expect(item?.detail).toContain('1');
  });

  it('bundles files with a hashed manifest and leaves out unavailable evidence', async () => {
    const stored = await createEvidenceItem({ name: 'foto.jpg', bytes: new Uint8Array([1, 2, 3]) }, []);
    const missing = { ...evidence('ev-9', 'invoice'), sha256: 'b'.repeat(64) };
    const bundle = await buildProjectDossier({
      project: project({ evidence: [stored, missing] }),
      assessment,
      reportHtml: '<html></html>',
      generatedAt: '2026-10-02T00:00:00Z',
    });
    const archive = unzipSync(zipProjectDossier(bundle));
    expect(Object.keys(archive).sort()).toEqual([
      'dossier-checklist.json', 'evidence/ev-1-foto.jpg', 'kernel-output.json', 'manifest.json',
      'project.oes.json', 'rekenrapport.html',
    ]);
    const manifest = JSON.parse(strFromU8(archive['manifest.json']));
    expect(manifest.kernel.inputFingerprint).toBe('abc');
    expect(manifest.missingEvidence).toEqual([{ id: 'ev-9', fileName: 'ev-9.pdf', reason: expect.any(String) }]);
    const photo = manifest.files.find((file: { path: string }) => file.path === 'evidence/ev-1-foto.jpg');
    expect(photo.sha256).toBe(stored.sha256);
    expect(photo.bytes).toBe(3);
    const saved = JSON.parse(strFromU8(archive['project.oes.json']));
    expect(saved.kernel.kernelVersion).toBe('0.1.0');
  });
});
