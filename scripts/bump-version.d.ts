// Types for scripts/bump-version.js, used by its Vitest test.
export const VERSION_FILES: string[];
export function isoWeek(date: Date): { year: number; week: number };
export function weekVersion(date: Date, tags: string[]): string;
export function msiVersion(version: string): string;
export function setVersion(files: Record<string, string>, version: string): Record<string, string>;
