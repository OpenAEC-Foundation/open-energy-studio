/**
 * Placeholder ProjectV2 type — generated from the Rust JSON Schema in Step 5
 * via `npm run generate-types`. Keep this file in sync with what
 * `openaec-project-shared` exports.
 *
 * For now we type ProjectV2 as a loose record so the store compiles before
 * code generation is wired up. Production code MUST use the generated types.
 */
export interface ProjectV2 {
  schema_version: string;
  shared: Record<string, unknown>;
  geometry: Record<string, unknown>;
  calcs: Record<string, unknown>;
  [key: string]: unknown;
}
