/**
 * Fifteenth review, finding 7: in the desktop app a failed dossier write
 * must reach the user, not fall back to a browser download that does
 * nothing in the webview and makes the export look successful.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { IProject } from '../core/energy/types';

const writeFile = vi.fn();
const save = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true, invoke: vi.fn(() => Promise.reject(new Error('no kernel'))) }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ save: (...args: unknown[]) => save(...args) }));
vi.mock('@tauri-apps/plugin-fs', () => ({ writeFile: (...args: unknown[]) => writeFile(...args) }));

const project = {
  id: 'p1', name: 'Woning', description: '', buildingFunction: 'residential', address: '', city: '',
  zones: [], heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [],
  solarPV: [], solarThermal: [], constructions: [],
} as unknown as IProject;

describe('dossier export in the desktop app', () => {
  beforeEach(() => {
    writeFile.mockReset();
    save.mockReset();
  });

  it('reports a failed write instead of pretending a download', async () => {
    const { downloadProjectDossier, DossierWriteError } = await import('../core/report/ReportGenerator');
    save.mockResolvedValue('/alleen-lezen/Projectdossier-Woning.zip');
    writeFile.mockRejectedValue(new Error('Permission denied'));
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click');
    await expect(downloadProjectDossier(project)).rejects.toBeInstanceOf(DossierWriteError);
    await expect(downloadProjectDossier(project)).rejects.toThrow(/Permission denied/);
    expect(click).not.toHaveBeenCalled();
    click.mockRestore();
  });

  it('writes the ZIP to the chosen path and treats a cancelled dialog as no export', async () => {
    const { downloadProjectDossier } = await import('../core/report/ReportGenerator');
    save.mockResolvedValueOnce('/dossiers/Projectdossier-Woning.zip');
    writeFile.mockResolvedValue(undefined);
    const manifest = await downloadProjectDossier(project);
    expect(writeFile).toHaveBeenCalledTimes(1);
    expect(writeFile.mock.calls[0][0]).toBe('/dossiers/Projectdossier-Woning.zip');
    expect(manifest.files.length).toBeGreaterThan(0);
    save.mockResolvedValueOnce(null);
    await downloadProjectDossier(project);
    expect(writeFile).toHaveBeenCalledTimes(1);
  });
});
