import type { IProject, ProjectImportIssue, ProjectImportRecord } from '../energy/types';

/**
 * Records that the adviser read data in with `tool` (BRL 9501 29-05-2026
 * §4.3.1 opmerking, p. 8: the registration states whether, and with which
 * tool, data was read in by hand; automatic reading is not allowed). Every
 * import in this program is started by the adviser.
 *
 * The record names the file, its SHA-256 (so the source file can be
 * identified later), the program and NTA 8800 version the file names, and
 * what the importer filled in or left out. It is
 * provenance, not label input: like the registration it stays outside the
 * label input hash (`labelInputSha256`, kernel `label_input_hash`), so a
 * relabel comparison never trips on it.
 */
export function withImportRecord(
  project: IProject,
  tool: string,
  fileName?: string,
  importedAt: string = new Date().toISOString(),
  details: { sha256?: string; sourceVersion?: string; issues?: ProjectImportIssue[] } = {},
): IProject {
  const record: ProjectImportRecord = {
    tool,
    importedAt,
    ...(fileName ? { fileName } : {}),
    ...(details.sha256 ? { sha256: details.sha256 } : {}),
    ...(details.sourceVersion ? { sourceVersion: details.sourceVersion } : {}),
    ...(details.issues ? { issues: details.issues } : {}),
  };
  return { ...project, importLog: [...(project.importLog ?? []), record] };
}

/** Distinct tools of the import log, in import order. */
export function importTools(project: Pick<IProject, 'importLog'>): string[] {
  const tools: string[] = [];
  for (const record of project.importLog ?? []) {
    const tool = record.tool?.trim();
    if (tool && !tools.includes(tool)) tools.push(tool);
  }
  return tools;
}
