/**
 * Placeholder ProjectResult type — generated from `isso51_core::ProjectResult`
 * in Step 5. See the comment in `./project.ts` for the same caveat.
 */
export interface ProjectResult {
  rooms: unknown[];
  summary: Record<string, unknown>;
  [key: string]: unknown;
}

/** Stable shape of errors surfaced by Rust command handlers. */
export interface AppError {
  kind:
    | 'io'
    | 'serde'
    | 'calc'
    | 'view'
    | 'migration'
    | 'validation'
    | 'other';
  message: string;
}
