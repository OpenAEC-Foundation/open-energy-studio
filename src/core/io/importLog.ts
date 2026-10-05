import type { IProject, ProjectImportRecord } from '../energy/types';

/**
 * Records that the adviser read data in with `tool` (BRL 9501 29-05-2026
 * §4.3.1 opmerking, p. 8: the registration states whether, and with which
 * tool, data was read in by hand; automatic reading is not allowed). Every
 * import in this program is started by the adviser.
 */
export function withImportRecord(
  project: IProject,
  tool: string,
  fileName?: string,
  importedAt: string = new Date().toISOString(),
): IProject {
  const record: ProjectImportRecord = { tool, importedAt, ...(fileName ? { fileName } : {}) };
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
