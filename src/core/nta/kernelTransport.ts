// One way to reach the Rust kernel from the UI. The desktop app calls its
// built-in kernel through Tauri; the browser build loads the WebAssembly
// module built from `crates/nta8800-wasm`; the Vite development server can
// also proxy to the HTTP API on port 3007. All three run the same operation
// registry, so a call site only names the Tauri command, the HTTP route and
// the request payload.
import { invoke, isTauri } from '@tauri-apps/api/core';

export interface WasmKernel {
  run(route: string, body: string): string;
  version(): string;
  listOperations(): string;
}

interface WasmOutcome<T> {
  status: number;
  body: T;
}

let wasmKernel: Promise<WasmKernel | null> | null = null;

/**
 * Loads the WebAssembly kernel once. Resolves to `null` when the module is not
 * part of this build or fails to initialise, so the caller can fall back.
 */
export function loadWasmKernel(): Promise<WasmKernel | null> {
  // Under vitest the UI tests mock `fetch` for the development-server path;
  // the module loader would consume those mocks. The wasm module itself is
  // tested directly in kernel-wasm.test.ts.
  if (import.meta.env.MODE === 'test') return Promise.resolve(null);
  if (!wasmKernel) {
    wasmKernel = (async () => {
      try {
        const mod = await import('../../kernel-wasm/nta8800.js');
        await mod.default();
        return { run: mod.run, version: mod.version, listOperations: mod.list_operations };
      } catch (error) {
        console.warn('[kernel] WebAssembly kernel unavailable', error);
        return null;
      }
    })();
  }
  return wasmKernel;
}

/** Which transport this build will use, for the status bar and diagnostics. */
export async function kernelTransport(): Promise<'tauri' | 'wasm' | 'http' | 'none'> {
  if (isTauri()) return 'tauri';
  if (await loadWasmKernel()) return 'wasm';
  if (import.meta.env.DEV) return 'http';
  return 'none';
}

const UNAVAILABLE = 'The Rust kernel is not available: the WebAssembly module did not load and this is not the desktop app or the development server.';

function parseOutcome<T>(text: string): T {
  const outcome = JSON.parse(text) as WasmOutcome<T>;
  // A kernel refusal (422) still carries the assessment with its `status`,
  // which the UI shows. A transport or shape error (400, 404, 500) carries
  // only the error envelope; that becomes an exception, as the HTTP client
  // raised one for a failed response.
  const body = outcome.body as { status?: unknown; code?: string; message?: string } | null;
  if (outcome.status >= 400 && (!body || typeof body !== 'object' || !('status' in body))) {
    throw new Error(`${body?.code ?? 'kernel_error'}: ${body?.message ?? `kernel status ${outcome.status}`}`);
  }
  return outcome.body;
}

/** POST operation: `command` is the Tauri command, `route` the HTTP route. */
export async function kernelCall<T>(command: string, route: string, payload: Record<string, unknown>): Promise<T> {
  if (isTauri()) return invoke<T>(command, payload);
  const wasm = await loadWasmKernel();
  if (wasm) return parseOutcome<T>(wasm.run(route, JSON.stringify(payload)));
  if (import.meta.env.DEV) {
    const response = await fetch(route, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(payload),
    });
    return httpOutcome<T>(response);
  }
  throw new Error(UNAVAILABLE);
}

/** Same rules as `parseOutcome`, for a response of the development server. */
async function httpOutcome<T>(response: Response): Promise<T> {
  let parsed: unknown;
  if (typeof response.text === 'function') {
    const text = await response.text();
    try {
      parsed = JSON.parse(text);
    } catch {
      throw new Error(`Rust API: HTTP ${response.status}: ${text}`);
    }
  } else {
    parsed = await response.json();
  }
  const body = parsed as { status?: unknown; error?: string; message?: string } | null;
  if (!response.ok && (!body || typeof body !== 'object' || !('status' in body))) {
    throw new Error(body?.message ?? body?.error ?? `Rust API: HTTP ${response.status}`);
  }
  return parsed as T;
}

/** GET operation without a request body. */
export async function kernelGet<T>(command: string, route: string): Promise<T> {
  if (isTauri()) return invoke<T>(command);
  const wasm = await loadWasmKernel();
  if (wasm) return parseOutcome<T>(wasm.run(route, ''));
  if (import.meta.env.DEV) {
    const response = await fetch(route);
    return response.json() as Promise<T>;
  }
  throw new Error(UNAVAILABLE);
}
