import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';
import { deserializeProjectFile } from '../core/io/ProjectSerializer';
import { normalizeProject } from '../core/energy/normalizeProject';
import { migrateLegacyRelabel } from '../core/nta/Registration';
import { projectRouteInput } from '../core/nta/KernelInput';

// The testset dossier (BRL 9501 §3.3) gives every ISSO 54 deeltest as a project
// file the app opens. These files are written by the same generator as the
// dossier (crates/nta8800-service/tests/testset_roundtrip.rs keeps them current
// and proves the kernel answer for the file equals the reference suite). Here
// the app opens them as App.tsx does and must send the file's project to the
// kernel unchanged, so the attesting body sees in the app what the dossier lists.
const fixtures = ['isso54-v2-EPW001', 'isso54-v2-EPW101a', 'isso54-v2-EPU001'];
const root = resolve(__dirname, 'fixtures/testset');

describe('testset project files', () => {
  it.each(fixtures)('%s opens and is calculated as written', (caseId) => {
    const text = readFileSync(resolve(root, `${caseId}.oes.json`), 'utf8');
    const written = JSON.parse(text) as { project: unknown };
    const { project: raw, kernel, unknownFields, fileVersion } = deserializeProjectFile(text);
    expect(fileVersion).toBe('1.0');
    expect(unknownFields).toBeUndefined();
    // A valid stamp: the app compares it with the current kernel on opening.
    expect(kernel?.kernelVersion).toMatch(/^\d+\.\d+\.\d+$/);
    expect(kernel?.inputFingerprint).toBeTruthy();
    // The open path of App.tsx changes nothing in these projects.
    const { project: loaded, missing, markedForReview } = migrateLegacyRelabel(normalizeProject(raw));
    expect(missing).toEqual([]);
    expect(markedForReview).toBe(0);
    expect(projectRouteInput(loaded)).toEqual(written.project);
  });

  it('keeps the edition of the deeltest (NTA 8800:2022)', () => {
    for (const caseId of fixtures) {
      const { project } = deserializeProjectFile(readFileSync(resolve(root, `${caseId}.oes.json`), 'utf8'));
      expect(project.ntaCalculation?.normVersion).toBe('2022');
    }
  });
});
