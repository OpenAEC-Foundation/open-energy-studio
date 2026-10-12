import { readFileSync } from 'fs';
import { join } from 'path';
import { describe, expect, it } from 'vitest';

// A release from a fresh checkout ran the gate without ../dist, and Tauri's
// generate_context! refused to compile. The gate builds the frontend first.
describe('scripts/verify-nta.sh', () => {
  const gate = readFileSync(join(__dirname, '../../scripts/verify-nta.sh'), 'utf8');

  it('builds the frontend before the Tauri steps', () => {
    const build = gate.indexOf('npm run build');
    expect(build).toBeGreaterThan(-1);
    expect(build).toBeLessThan(gate.indexOf('step "Tauri check"'));
    expect(build).toBeLessThan(gate.indexOf('step "Tauri command tests"'));
  });

  it('collects every background pipeline before the UI tests and fails on a failed one', () => {
    // The Rust 1.77.2 run and the frontend/Tauri pipeline run next to the kernel steps;
    // vitest must not run next to a cargo build (timing tests), and a failure must stop the gate.
    expect(gate).toContain('start_bg "core on Rust 1.77.2" msrv_core');
    expect(gate).toContain('start_bg "frontend bundle and Tauri" frontend_and_tauri');
    const collect = gate.lastIndexOf('\nfinish_bg\n');
    expect(collect).toBeGreaterThan(gate.indexOf('step "TypeScript"'));
    expect(collect).toBeLessThan(gate.indexOf('step "UI tests"'));
    expect(gate).toContain('exit "$status"');
    // Review 61: the trap stops every background tree, marks the failed step and keeps the exit status.
    expect(gate).toContain(`trap 'nta_exit=$?; stop_bg; timing_summary "$nta_exit"; exit "$nta_exit"' EXIT`);
    expect(gate).toMatch(/kill_tree\(\) \{[\s\S]*pgrep -P "\$1"[\s\S]*kill_tree "\$child"/);
    expect(gate).toContain('step_done " (failed, exit $status)"');
  });

  it('keeps the Rust 1.77.2 run of the whole kernel test suite', () => {
    expect(gate).toMatch(/CARGO_TARGET_DIR=target\/msrv-core cargo \+1\.77\.2 test --locked --quiet \\\n\s+--manifest-path crates\/nta8800-core\/Cargo\.toml\n/);
  });

  it('runs the app against the real kernel through kernel_once, after the UI tests', () => {
    const e2e = gate.indexOf('step "app against the real kernel (kernel_once)"');
    expect(e2e).toBeGreaterThan(gate.indexOf('step "UI tests"'));
    expect(gate).toContain('--example kernel_once');
    expect(gate).toContain('OES_KERNEL_ONCE=');
    // Review 59: the binary is found where cargo put it, also with CARGO_TARGET_DIR set.
    expect(gate).toContain('${CARGO_TARGET_DIR:-crates/nta8800-service/target}/debug/examples/kernel_once');
    expect(gate).toContain('src/__tests__/real-kernel-e2e.test.tsx');
  });

  it('runs the clear-field sweep through the kernel after the UI tests (review 58)', () => {
    const ui = gate.indexOf('NTA_CLEAR_SWEEP_OUT=');
    const kernel = gate.indexOf('NTA_CLEAR_SWEEP_DOCS=');
    expect(ui).toBeGreaterThan(gate.indexOf('step "UI tests"'));
    expect(kernel).toBeGreaterThan(gate.indexOf('step "project-form states through the kernel"'));
  });
});
