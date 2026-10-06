import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { initSync, list_operations, run, version } from '../kernel-wasm/nta8800.js';

// The WebAssembly kernel is the same Rust code as the desktop kernel and the
// HTTP API, built with `npm run build:wasm`. These tests load the committed
// module and run it the way the browser build does.
const wasmPath = join(__dirname, '..', 'kernel-wasm', 'nta8800_bg.wasm');
const survey = JSON.parse(readFileSync(join(__dirname, '..', '..', 'training-data', 'nta8800-opname-1930-terraced.json'), 'utf8'));
const project = JSON.parse(readFileSync(join(__dirname, '..', '..', 'training-data', 'nta8800-example-terraced-dwelling.json'), 'utf8'));

describe('WebAssembly kernel', () => {
  initSync({ module: readFileSync(wasmPath) });

  it('reports the same kernel identity as the API', () => {
    const identity = JSON.parse(version());
    expect(identity.kernelVersion).toMatch(/^\d+\.\d+\.\d+$/);
    expect(identity.targetNormVersion).toBe('NTA 8800:2025+C1:2026');
    const operations = JSON.parse(list_operations()) as Array<{ name: string; path: string }>;
    expect(operations.length).toBeGreaterThan(40);
    expect(operations.some((op) => op.path === '/v1/nta8800/project/performance')).toBe(true);
  });

  it('runs a basisopname through the same route the browser client uses', () => {
    const outcome = JSON.parse(run('/api/v1/nta8800/opname/residential', JSON.stringify({ survey })));
    expect(outcome.status).toBe(200);
    expect(outcome.body.status).toBe('calculated_unverified');
    expect(outcome.body.performance.primaryFossilIndicatorKwhPerM2Year).toBeGreaterThan(0);
  });

  it('calculates a project and keeps the HTTP status rules', () => {
    const payload = project.project ?? project;
    const outcome = JSON.parse(run('/api/v1/nta8800/project/performance', JSON.stringify({ project: payload })));
    expect([200, 422]).toContain(outcome.status);
    expect(outcome.body.status).toBeDefined();
    const unknown = JSON.parse(run('/api/v1/nope', '{}'));
    expect(unknown.status).toBe(404);
    expect(unknown.body.code).toBe('unknown_operation');
  });
});
