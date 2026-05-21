/**
 * Thin wrapper around `@tauri-apps/api/core::invoke` providing a typed
 * surface for the commands declared in `src-tauri/src/commands/`.
 *
 * Every function in this module corresponds 1:1 to a `#[tauri::command]`
 * function on the Rust side. Errors propagated by Tauri carry the structured
 * `{ kind, message }` shape defined by `src-tauri/src/error.rs::AppError`.
 */
import { invoke } from '@tauri-apps/api/core';

import type { ProjectV2 } from '../types/project';
import type { AppError, ProjectResult } from '../types/result';

/** Detects whether we're running inside the Tauri runtime. */
export function isTauri(): boolean {
  return (
    typeof window !== 'undefined' &&
    Boolean((window as unknown as { __TAURI__?: unknown }).__TAURI__)
  );
}

/** Convert any thrown value into a structured `AppError`. */
export function toAppError(value: unknown): AppError {
  if (
    value &&
    typeof value === 'object' &&
    'kind' in value &&
    'message' in value
  ) {
    return value as AppError;
  }
  return { kind: 'other', message: String(value) };
}

// ── calculation ──────────────────────────────────────────────────────────

export async function calculate(project: ProjectV2): Promise<ProjectResult> {
  return invoke<ProjectResult>('calculate', { project });
}

// ── project I/O ──────────────────────────────────────────────────────────

export async function newProject(name: string): Promise<ProjectV2> {
  return invoke<ProjectV2>('new_project', { name });
}

export async function loadProject(path: string): Promise<ProjectV2> {
  return invoke<ProjectV2>('load_project', { path });
}

export async function saveProject(
  path: string,
  project: ProjectV2,
  result: ProjectResult | null,
): Promise<void> {
  return invoke<void>('save_project', { path, project, result });
}

export async function migrateLegacy(path: string): Promise<ProjectV2> {
  return invoke<ProjectV2>('migrate_legacy', { path });
}

// ── schemas (returned as JSON-encoded strings) ──────────────────────────

export async function projectSchema(): Promise<string> {
  return invoke<string>('project_schema');
}

export async function resultSchema(): Promise<string> {
  return invoke<string>('result_schema');
}
