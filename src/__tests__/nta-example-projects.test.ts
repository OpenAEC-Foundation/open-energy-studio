import { afterEach, describe, expect, it, vi } from 'vitest';
import { EXAMPLE_KINDS, exampleProject } from '../core/nta/ExampleProjects';
import { calculateProjectPerformanceWithRust } from '../core/nta/KernelClient';
import { kernelProject, nullPaths, withoutNulls } from '../core/nta/KernelInput';
import { syncVentilation } from '../core/nta/NtaFormModels';
import { spaceGeneratorTemplate } from '../core/nta/NtaSystemTemplates';

// Top-level fields without a serde default in NtaCalculationInput
// (crates/nta8800-core/src/project_performance.rs).
const REQUIRED = [
  'calculationScope', 'areaSourceReference', 'usageFunction', 'setpoints', 'thermalMass', 'internalGains',
  'windowSolar', 'emission', 'distribution', 'generator', 'bacsFactor', 'bacsSourceReference',
  'useInventoryComplete', 'productionInventoryComplete', 'demandUsesFixedC1Ventilation', 'batteryStoragePresent',
];

describe('NTA example projects', () => {
  afterEach(() => vi.unstubAllGlobals());

  it.each(EXAMPLE_KINDS)('%s serializes the saved form draft to the kernel shape', (kind) => {
    const project = exampleProject(kind);
    expect(project.zones.length).toBeGreaterThan(0);
    expect(project.zones[0].floorArea).toBeGreaterThan(0);
    // The form saves the draft through syncVentilation; the client strips nulls.
    const draft = syncVentilation(project.ntaCalculation as unknown as Record<string, unknown>, project);
    const sent = kernelProject({ ...project, ntaCalculation: draft as never });
    const block = JSON.parse(JSON.stringify(sent.ntaCalculation)) as Record<string, unknown>;
    expect(nullPaths(block)).toEqual([]);
    for (const key of REQUIRED) expect(block, key).toHaveProperty(key);
    expect(block.calculationScope).toBe(kind === 'small_office' ? 'utility' : 'residential');
    expect(block).toEqual(draft);
  });

  it('gives every example a fresh id', () => {
    expect(exampleProject('small_office').id).not.toBe(exampleProject('small_office').id);
  });

  it('posts the example without nulls to the project route', async () => {
    vi.stubEnv('DEV', true);
    const fetchMock = vi.fn(async () => new Response(JSON.stringify({ status: 'calculated_unverified', gaps: [] })));
    vi.stubGlobal('fetch', fetchMock);
    const project = exampleProject('terraced_dwelling');
    const generator = spaceGeneratorTemplate('gas_boiler', project);
    const result = await calculateProjectPerformanceWithRust({
      ...project, ntaCalculation: { ...project.ntaCalculation!, generator: generator as never },
    });
    expect(result.status).toBe('calculated_unverified');
    const [url, init] = fetchMock.mock.calls[0] as unknown as [string, RequestInit];
    expect(url).toBe('/api/v1/nta8800/project/performance');
    const body = JSON.parse(String(init.body)) as { project: { ntaCalculation: Record<string, unknown> } };
    expect(nullPaths(body)).toEqual([]);
    // The template's unknown values are absent, so the kernel names them as gaps.
    const boiler = (body.project.ntaCalculation.generator as { boiler: Record<string, unknown> }).boiler;
    expect(boiler).not.toHaveProperty('kind');
    expect(nullPaths(generator)).toContain('boiler.kind');
    vi.unstubAllEnvs();
  });
});

describe('kernel input nulls', () => {
  it('drops null object members at any depth and keeps array positions', () => {
    const value = { a: null, b: { c: null, d: 1 }, e: [{ f: null, g: 'x' }, null], h: 0, i: false, j: '' };
    expect(withoutNulls(value)).toEqual({ b: { d: 1 }, e: [{ g: 'x' }, null], h: 0, i: false, j: '' });
    expect(nullPaths(value)).toEqual(['a', 'b.c', 'e[0].f', 'e[1]']);
  });
});
