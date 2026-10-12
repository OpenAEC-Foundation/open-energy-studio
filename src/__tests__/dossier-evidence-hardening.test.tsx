/**
 * Fifteenth review: dossier, evidence and webview hardening.
 * Evidence size/type limits, ids that stay unique when additions race,
 * duplicate ids repaired on open, archive names that never overwrite each
 * other, compressed files stored, the dossier size limit, the zip-bomb
 * guard of the imports, sandboxed report frames and the desktop CSP.
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { useEffect } from 'react';
import { beforeEach, describe, expect, it } from 'vitest';
import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import { unzipSync, zipSync } from 'fflate';
import { useEnergy } from '../context/EnergyContext';
import type { IProject } from '../core/energy/types';
import type { NtaEvidenceItem } from '../core/nta/KernelClient';
import {
  appendEvidence, checkEvidenceFile, clearSessionEvidence, createEvidenceItem, EvidenceImportError,
  evidenceArchiveName, forgetSessionEvidence, MAX_EVIDENCE_BYTES, nextEvidenceId, rememberEvidenceBytes,
  releaseEvidenceIds, repairEvidenceIds, safeName, sessionEvidenceCount, sha256Hex,
} from '../core/nta/Evidence';
import { normalizeProject } from '../core/energy/normalizeProject';
import {
  buildProjectDossier, DossierTooLargeError, MAX_DOSSIER_BYTES, uniqueArchivePath, zipProjectDossier,
  type DossierBundle,
} from '../core/report/ProjectDossier';
import { ArchiveTooLargeError, safeUnzip } from '../core/io/safeUnzip';
import { importFromUNIEC3 } from '../core/io/UNIEC3Exporter';
import { importFromVABI } from '../core/io/VABIElementsBridge';
import { REPORT_FRAME_SANDBOX, REPORT_PRINT_SANDBOX } from '../core/report/reportFrame';
import { EvidenceAttach } from '../components/EvidenceLink/EvidenceLink';
import { dossierErrorText } from '../components/ReportView/useDossier';
import { DossierWriteError } from '../core/report/ReportGenerator';
import { renderWithProviders } from './test-utils';

const encoder = new TextEncoder();

function project(evidence: NtaEvidenceItem[], extra: Record<string, unknown> = {}): IProject {
  return {
    id: 'p1', name: 'Woning', description: '', buildingFunction: 'residential', address: '', city: '',
    zones: [], heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [],
    solarPV: [], solarThermal: [], constructions: [],
    registration: { evidence, ...extra },
  } as unknown as IProject;
}

const entry = (id: string, fileName: string, sha256: string): NtaEvidenceItem => ({ id, kind: 'other', fileName, sha256 });

describe('evidence import limits', () => {
  beforeEach(() => clearSessionEvidence());

  it('refuses files that are too large or of an unsupported type, before reading them', async () => {
    expect(() => checkEvidenceFile({ name: 'film.mp4', size: 10 })).toThrow(EvidenceImportError);
    expect(() => checkEvidenceFile({ name: 'setup.exe', size: 10 })).toThrow(/evidence_type_not_allowed/);
    expect(() => checkEvidenceFile({ name: 'zonder-extensie', size: 10 })).toThrow(/evidence_type_not_allowed/);
    expect(() => checkEvidenceFile({ name: 'factuur.PDF', size: MAX_EVIDENCE_BYTES + 1 })).toThrow(/evidence_too_large/);
    expect(() => checkEvidenceFile({ name: 'factuur.pdf', size: MAX_EVIDENCE_BYTES })).not.toThrow();
    for (const name of ['foto.jpg', 'foto.HEIC', 'tekening.dwg', 'model.ifc', 'offerte.docx', 'meting.xlsx']) {
      expect(() => checkEvidenceFile({ name, size: 1 })).not.toThrow();
    }
    await expect(createEvidenceItem({ name: 'script.js', bytes: encoder.encode('x') }, [])).rejects.toBeInstanceOf(EvidenceImportError);
  });

  it('shows the refusal at the attach box and adds nothing', async () => {
    const { container } = renderWithProviders(<>
      <Seed value={project([])} />
      <EvidenceAttach pointer="/zones/0" />
      <EvidenceProbe />
    </>);
    const input = container.querySelector('input[type="file"]') as HTMLInputElement;
    expect(input.accept).toContain('.pdf');
    const file = new File([encoder.encode('MZ')], 'setup.exe');
    await act(async () => { fireEvent.change(input, { target: { files: [file] } }); });
    expect((await screen.findByRole('alert')).textContent).toMatch(/setup\.exe/);
    expect(evidenceIn()).toHaveLength(0);
  });

  it('hashes a whole buffer without copying and a partial view correctly', async () => {
    const bytes = encoder.encode('abcdef');
    const view = new Uint8Array(bytes.buffer, 2, 3);
    expect(await sha256Hex(view)).toBe(await sha256Hex(encoder.encode('cde')));
    expect(await sha256Hex(bytes)).toBe(await sha256Hex(new Uint8Array(bytes)));
  });

  it('drops the session bytes of a removed entry unless another entry shares the file', () => {
    rememberEvidenceBytes('a'.repeat(64), encoder.encode('x'));
    rememberEvidenceBytes('b'.repeat(64), encoder.encode('y'));
    forgetSessionEvidence('a'.repeat(64), [entry('ev-2', 'b.pdf', 'b'.repeat(64))]);
    expect(sessionEvidenceCount()).toBe(1);
    forgetSessionEvidence('b'.repeat(64), [entry('ev-3', 'b-kopie.pdf', 'b'.repeat(64))]);
    expect(sessionEvidenceCount()).toBe(1);
  });
});

describe('evidence ids', () => {
  beforeEach(() => clearSessionEvidence());

  it('gives two additions running at once different ids', async () => {
    const [first, second] = await Promise.all([
      createEvidenceItem({ name: 'a.pdf', bytes: encoder.encode('a') }, []),
      createEvidenceItem({ name: 'b.pdf', bytes: encoder.encode('b') }, []),
    ]);
    expect(first.id).not.toBe(second.id);
    releaseEvidenceIds([first.id, second.id]);
    expect(nextEvidenceId([])).toBe('ev-1');
  });

  it('appends to the register as it is, renaming a taken id', () => {
    const current = [entry('ev-1', 'a.pdf', 'a'.repeat(64)), entry('ev-2', 'b.pdf', 'b'.repeat(64))];
    const next = appendEvidence(current, [entry('ev-2', 'c.pdf', 'c'.repeat(64))]);
    expect(next.map((item) => item.id)).toEqual(['ev-1', 'ev-2', 'ev-3']);
    expect(next[2].fileName).toBe('c.pdf');
  });

  it('keeps both files and registration edits when two boxes add files at the same time', async () => {
    const { container } = renderWithProviders(<>
      <Seed value={project([entry('ev-1', 'a.pdf', 'a'.repeat(64))], { surveyDate: '2026-10-01' })} />
      <EvidenceAttach pointer="/zones/0" />
      <EvidenceAttach pointer="/zones/1" />
      <RegistrationEdit />
      <EvidenceProbe />
    </>);
    const [one, two] = Array.from(container.querySelectorAll('input[type="file"]')) as HTMLInputElement[];
    await act(async () => {
      fireEvent.change(one, { target: { files: [new File([encoder.encode('1')], 'een.pdf')] } });
      fireEvent.change(two, { target: { files: [new File([encoder.encode('2')], 'twee.pdf')] } });
      // An edit made while the files are still being hashed.
      fireEvent.click(screen.getByRole('button', { name: 'edit-registration' }));
    });
    await waitFor(() => expect(evidenceIn()).toHaveLength(3));
    const ids = evidenceIn().map((item) => item.id);
    expect(new Set(ids).size).toBe(3);
    expect(evidenceIn().map((item) => item.fileName).sort()).toEqual(['a.pdf', 'een.pdf', 'twee.pdf']);
    expect(screen.getByTestId('registration').textContent).toBe('2026-10-09');
  });

  it('repairs repeated ids on open and notes the change', () => {
    const repeated = [
      entry('ev-1', 'a.pdf', 'a'.repeat(64)), entry('ev-1', 'b.pdf', 'b'.repeat(64)), entry('ev-2', 'c.pdf', 'c'.repeat(64)),
    ];
    const { renamed } = repairEvidenceIds(repeated);
    expect(renamed).toEqual([{ from: 'ev-1', to: 'ev-3' }]);
    const opened = normalizeProject(project(repeated));
    const evidence = opened.registration?.evidence ?? [];
    expect(evidence.map((item) => item.id)).toEqual(['ev-1', 'ev-3', 'ev-2']);
    expect(evidence[1].description).toMatch(/was ev-1/);
    const unique = project([entry('ev-1', 'a.pdf', 'a'.repeat(64))]);
    expect(normalizeProject(unique).registration).toBe(unique.registration);
  });
});

describe('dossier archive', () => {
  beforeEach(() => clearSessionEvidence());

  it('keeps file names safe and their extension when shortened', () => {
    expect(safeName('../../etc/passwd')).toBe('etc-passwd');
    expect(safeName('foto.')).toBe('foto');
    expect(safeName('scan  .pdf . ')).toBe('scan-.pdf');
    expect(safeName(`${'a'.repeat(120)}😀.jpg`)).toMatch(/^a+\.jpg$/);
    expect(Array.from(safeName(`${'a'.repeat(120)}.jpg`)).length).toBeLessThanOrEqual(80);
    expect(safeName('warmtepomp-😀.jpg')).toBe('warmtepomp-😀.jpg');
    expect(safeName('Ølfabriek.pdf')).toBe('Ølfabriek.pdf');
    expect(safeName('...')).toBe('bestand');
  });

  it('gives colliding evidence names a suffix so the manifest hash matches the archived bytes', async () => {
    const one = encoder.encode('eerste');
    const two = encoder.encode('tweede');
    const [shaOne, shaTwo] = [await sha256Hex(one), await sha256Hex(two)];
    rememberEvidenceBytes(shaOne, one);
    rememberEvidenceBytes(shaTwo, two);
    // "ev 2" and "ev-2" map to the same archive name.
    const evidence = [entry('ev 2', 'foto.jpg', shaOne), entry('ev-2', 'foto.jpg', shaTwo)];
    expect(evidenceArchiveName(evidence[0])).toBe(evidenceArchiveName(evidence[1]));
    const bundle = await buildProjectDossier({ project: project(evidence), assessment: null, opname: null, reportHtml: null });
    const [first, second] = bundle.manifest.evidence;
    expect(first.archivePath).not.toBe(second.archivePath);
    const archive = unzipSync(zipProjectDossier(bundle));
    for (const item of bundle.manifest.evidence) {
      expect(await sha256Hex(archive[item.archivePath as string])).toBe(item.sha256);
    }
    expect(uniqueArchivePath('evidence/a.jpg', { 'evidence/a.jpg': 1, 'evidence/a-2.jpg': 1 })).toBe('evidence/a-3.jpg');
  });

  it('stores already compressed files and refuses a dossier over the ZIP limit', () => {
    const zeros = new Uint8Array(20_000);
    const zip = zipProjectDossier({ files: { 'evidence/ev-1-foto.jpg': zeros }, manifest: {} } as unknown as DossierBundle);
    expect(zip.byteLength).toBeGreaterThan(zeros.byteLength);
    const deflated = zipProjectDossier({ files: { 'kernel-output.json': zeros }, manifest: {} } as unknown as DossierBundle);
    expect(deflated.byteLength).toBeLessThan(1_000);
    const huge = { byteLength: MAX_DOSSIER_BYTES + 1 } as unknown as Uint8Array;
    expect(() => zipProjectDossier({ files: { 'evidence/film.mp4': huge }, manifest: {} } as unknown as DossierBundle))
      .toThrow(DossierTooLargeError);
  });

  it('explains a refused or failed export', () => {
    const t = (key: string, options?: Record<string, unknown>) => `${key}:${JSON.stringify(options ?? {})}`;
    expect(dossierErrorText(new DossierTooLargeError(3 * 1024 ** 3), t)).toBe('dossier.error.tooLarge:{"size":"3.0","max":2}');
    expect(dossierErrorText(new DossierWriteError('disk full'), t)).toBe('dossier.error.writeFailed:{"reason":"disk full"}');
  });
});

describe('import archives', () => {
  it('reads only wanted entries and refuses entries over the limit before inflating', () => {
    const zip = zipSync({ 'project.xml': encoder.encode('<x/>'), 'video.bin': new Uint8Array(5_000) }, { level: 9 });
    expect(Object.keys(safeUnzip(zip, (name) => name.endsWith('.xml')))).toEqual(['project.xml']);
    expect(() => safeUnzip(zip, () => true, { maxEntryBytes: 1_000, maxTotalBytes: 1e9 })).toThrow(ArchiveTooLargeError);
    expect(() => safeUnzip(zip, () => true, { maxEntryBytes: 1e9, maxTotalBytes: 2_000 })).toThrow(ArchiveTooLargeError);
  });

  it('does not unpack a zip bomb in the UNIEC3 or VABI import', () => {
    // 80 MB of zeros compresses to a few kB; the declared size gives it away.
    const bomb = new Uint8Array(80 * 1024 * 1024);
    expect(() => importFromUNIEC3(zipSync({ 'buildings/1/entities.json': bomb }, { level: 9 }))).toThrow(ArchiveTooLargeError);
    expect(() => importFromVABI(zipSync({ 'project.xml': bomb }, { level: 9 }))).toThrow(ArchiveTooLargeError);
    // An unrelated large entry is never inflated: the import fails on the missing data, not on memory.
    expect(() => importFromVABI(zipSync({ 'film.bin': bomb }, { level: 9 }))).toThrow(/project\.xml/);
  });
});

describe('webview hardening', () => {
  const root = resolve(__dirname, '..', '..');
  const read = (path: string) => readFileSync(resolve(root, path), 'utf8');

  it('runs report frames without scripts', () => {
    expect(REPORT_FRAME_SANDBOX).not.toMatch(/allow-scripts/);
    expect(REPORT_PRINT_SANDBOX).not.toMatch(/allow-scripts/);
    for (const file of ['src/components/ReportView/ReportBuilder.tsx', 'src/components/dialogs/PrintPreviewDialog/PrintPreviewDialog.tsx']) {
      const source = read(file);
      const frames = source.match(/<iframe[\s\S]*?\/>/g) ?? [];
      expect(frames.length).toBeGreaterThan(0);
      for (const frame of frames) expect(frame).toMatch(/sandbox=\{REPORT_FRAME_SANDBOX\}/);
    }
    expect(read('src/components/dialogs/PrintPreviewDialog/PrintPreviewDialog.tsx'))
      .toMatch(/setAttribute\('sandbox', REPORT_PRINT_SANDBOX\)/);
  });

  it('sets a CSP that blocks inline scripts and allows what the app loads', () => {
    const config = JSON.parse(read('src-tauri/tauri.conf.json')) as { app: { security: { csp: Record<string, string> } } };
    const csp = config.app.security.csp;
    expect(csp['script-src']).toBe("'self'");
    expect(csp['object-src']).toBe("'none'");
    expect(csp['connect-src']).toMatch(/ipc:/);
    // Every external origin the app fetches must be allowed.
    const fetched = new Set<string>();
    for (const file of ['src/components/dialogs/FeedbackDialog/FeedbackDialog.tsx']) {
      for (const url of read(file).match(/https:\/\/[a-z0-9.-]+/g) ?? []) fetched.add(url);
    }
    expect(fetched.size).toBeGreaterThan(0);
    for (const origin of fetched) expect(csp['connect-src']).toContain(origin);
    // The fonts linked in index.html.
    const html = read('index.html');
    if (html.includes('fonts.googleapis.com')) expect(csp['style-src']).toContain('https://fonts.googleapis.com');
    if (html.includes('fonts.gstatic.com')) expect(csp['font-src']).toContain('https://fonts.gstatic.com');
    // Evidence thumbnails are blob: URLs.
    expect(csp['img-src']).toContain('blob:');
  });
});

function Seed({ value }: { value: IProject }) {
  const { dispatch } = useEnergy();
  useEffect(() => { dispatch({ type: 'SET_PROJECT', payload: structuredClone(value) }); }, [dispatch, value]);
  return null;
}

function EvidenceProbe() {
  const { state } = useEnergy();
  return <pre data-testid="evidence">{JSON.stringify(state.project.registration?.evidence ?? [])}</pre>;
}

function RegistrationEdit() {
  const { state, dispatch } = useEnergy();
  const registration = state.project.registration as (Record<string, unknown> & IProject['registration']) | undefined;
  return <>
    <button type="button" onClick={() => dispatch({
      type: 'UPDATE_PROJECT_INFO', payload: { registration: { ...(registration ?? {}), surveyDate: '2026-10-09' } as IProject['registration'] },
    })}>edit-registration</button>
    <span data-testid="registration">{String((registration as Record<string, unknown> | undefined)?.surveyDate ?? '')}</span>
  </>;
}

const evidenceIn = () => JSON.parse(screen.getByTestId('evidence').textContent ?? '[]') as NtaEvidenceItem[];
