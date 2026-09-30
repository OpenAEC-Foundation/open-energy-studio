import { describe, expect, it } from 'vitest';
import { strFromU8, unzipSync } from 'fflate';
import { createDefaultProject } from '../context/EnergyContext';
import { exportToUNIEC3, importFromUNIEC3 } from '../core/io/UNIEC3Exporter';

describe('UNIEC3 draft export', () => {
  it('keeps project input without publishing indicative performance values', () => {
    const project = createDefaultProject();
    const archive = exportToUNIEC3(project);
    const files = unzipSync(archive);
    const summaryPath = Object.keys(files).find((path) => path.endsWith('/summary.json'));
    const entitiesPath = Object.keys(files).find((path) => path.endsWith('/entities.json'));
    expect(summaryPath).toBeDefined();
    expect(entitiesPath).toBeDefined();
    expect(JSON.parse(strFromU8(files[summaryPath!]))).toEqual({});
    const entities = JSON.parse(strFromU8(files[entitiesPath!])) as { NTAEntityId: string }[];
    expect(entities.some((entity) => entity.NTAEntityId === 'PRESTATIE')).toBe(false);
    expect(strFromU8(files['meta.json'])).toContain('input draft - unverified');

    const restored = importFromUNIEC3(archive);
    expect(restored.name).toBe(project.name);
    expect(restored.zones.length).toBe(project.zones.length);
  });
});
