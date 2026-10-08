// Types for scripts/nta-leveringsdocument.mjs, used by its Vitest test.
export interface ReleaseIdentity {
  softwareName: string;
  attestNumber: string;
  identificationCode: string;
  attestingBody: string;
  programVersion: string;
  kernelVersion: string;
  targetNormVersion: string;
}
export interface LeveringsdocumentValues extends ReleaseIdentity {
  commit?: string;
  tag?: string;
  releaseDate: string;
  packages: { name: string; sha256: string }[];
}
export function readIdentity(root: string): ReleaseIdentity;
export function sha256File(path: string): string;
export function renderLeveringsdocument(template: string, values: LeveringsdocumentValues): string;
