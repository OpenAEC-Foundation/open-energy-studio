// Types for scripts/nta-kernel-version.mjs, used by its Vitest test.
export interface UnreleasedEntry {
  line: number;
  date: string;
  title: string;
  changesResults: boolean;
}
export interface VersionSection {
  version: string;
  date: string;
  line: number;
}
export function parseVersion(text: string): [number, number, number];
export function compareVersions(a: string, b: string): number;
export function parseReleaseNotes(text: string): {
  sections: VersionSection[];
  unreleased: UnreleasedEntry[];
  problems: string[];
};
export function checkReleaseNotes(text: string, kernelVersion: string): string[];
export function releaseNotes(text: string, kernelVersion: string, date: string): string;
export function kernelVersion(root: string): string;
export function dutchDate(isoDate: string): string;
export function normaliseNewlines(text: string): string;
export function checkReleasedSections(previousText: string, currentText: string): string[];
