import type { ProjectImportIssue } from '../energy/types';
import { ImportFormatError } from './importIssues';

type Translate = (key: string, params?: Record<string, string | number>) => string;

const KINDS: ProjectImportIssue['kind'][] = ['defaulted', 'invalid', 'clamped', 'dropped', 'unknown_reference', 'duplicate', 'multiple'];

/** The NL/EN text of an import that could not be read at all. */
export function importErrorText(err: unknown, t: Translate): string {
  if (err instanceof ImportFormatError) {
    const key = `import.error.${err.code}`;
    const text = t(key, err.params);
    return text === key ? err.message : text;
  }
  if (err instanceof Error) return err.message;
  return String(err);
}

/** One line per issue: what happened, and where (NL/EN). */
export function importIssueLine(issue: ProjectImportIssue, t: Translate): string {
  const value = issue.value === undefined ? '' : ` (${issue.value})`;
  return `${t(`import.issue.kind.${issue.kind}`)}: ${issue.path}${value}`;
}

/**
 * Toast text after an import: the count per kind and the first lines; the
 * full list is in the project information (import record).
 */
export function importIssuesSummary(issues: ProjectImportIssue[], t: Translate, shown = 4): string {
  const counts = KINDS
    .map((kind) => [kind, issues.filter((issue) => issue.kind === kind).length] as const)
    .filter(([, count]) => count > 0)
    .map(([kind, count]) => `${count}× ${t(`import.issue.kind.${kind}`)}`)
    .join(', ');
  const lines = issues.slice(0, shown).map((issue) => `• ${importIssueLine(issue, t)}`);
  const more = issues.length > shown ? [t('import.issues.more', { count: issues.length - shown })] : [];
  return [counts, ...lines, ...more, t('import.issues.where')].join('\n');
}

/** Toast text after an export that left project data out (NL/EN). */
export function exportLossesSummary(losses: string[], t: Translate, shown = 5): string {
  const lines = losses.slice(0, shown).map((path) => `• ${path}`);
  const more = losses.length > shown ? [t('import.issues.more', { count: losses.length - shown })] : [];
  return [t('export.losses.intro'), ...lines, ...more].join('\n');
}
