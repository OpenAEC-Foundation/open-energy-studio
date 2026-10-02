import type { IProject, IBENGResult } from '../energy/types';
import { generateReportHTML } from './ReportTemplate';
import { generateNtaInputDossierHTML } from './NtaInputDossier';
import { generateNtaCalculationReportHTML } from './NtaCalculationReport';
import { calculateProjectPerformanceWithRust } from '../nta/KernelClient';
import { buildProjectDossier, zipProjectDossier } from './ProjectDossier';
import { assessMaatwerkadviesWithRust } from '../nta/KernelClient';
import { generateMaatwerkadviesReportHTML } from './MaatwerkadviesReport';

/** Runs the maatwerkadvies of the project and downloads its report. */
export async function downloadMaatwerkadviesReportHTML(project: IProject): Promise<void> {
  if (!project.maatwerkadvies) throw new Error('Geen maatwerkadvies gedefinieerd.');
  const assessment = await assessMaatwerkadviesWithRust(project, project.maatwerkadvies);
  const html = generateMaatwerkadviesReportHTML(project, project.maatwerkadvies, assessment);
  const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `Maatwerkadvies-${(project.name || 'project').replace(/[^\p{L}\p{N}._-]+/gu, '-')}.html`;
  link.click();
  URL.revokeObjectURL(url);
}

/** Export the NTA input and evidence inventory even when BENG is unavailable. */
export function downloadNtaInputDossierHTML(project: IProject): void {
  const html = generateNtaInputDossierHTML(project);
  const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `NTA8800-Invoer-${(project.name || 'project').replace(/[^\p{L}\p{N}._-]+/gu, '-')}.html`;
  link.click();
  URL.revokeObjectURL(url);
}

/** Runs the Rust project chain and downloads its unverified calculation report. */
export async function downloadNtaCalculationReportHTML(project: IProject): Promise<void> {
  const assessment = await calculateProjectPerformanceWithRust(project);
  const html = generateNtaCalculationReportHTML(project, assessment);
  const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `NTA8800-Rekenrapport-${(project.name || 'project').replace(/[^\p{L}\p{N}._-]+/gu, '-')}.html`;
  link.click();
  URL.revokeObjectURL(url);
}

/**
 * Builds the BRL 9500 project dossier (project file, kernel output,
 * report, evidence files, checklist and hashed manifest) as one ZIP and
 * saves it. Returns the checklist so the caller can show open items.
 */
export async function downloadProjectDossier(project: IProject) {
  let assessment = null;
  try {
    assessment = await calculateProjectPerformanceWithRust(project);
  } catch {
    assessment = null;
  }
  const reportHtml = assessment ? generateNtaCalculationReportHTML(project, assessment) : null;
  const bundle = await buildProjectDossier({ project, assessment, reportHtml });
  const zip = zipProjectDossier(bundle);
  const fileName = `Projectdossier-${(project.name || 'project').replace(/[^\p{L}\p{N}._-]+/gu, '-')}.zip`;
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeFile } = await import('@tauri-apps/plugin-fs');
    const path = await save({ defaultPath: fileName, filters: [{ name: 'ZIP', extensions: ['zip'] }] });
    if (path) {
      await writeFile(path, zip);
      return bundle.manifest;
    }
    if (path === null) return bundle.manifest;
  } catch { /* browser fallback */ }
  const blob = new Blob([zip.buffer as ArrayBuffer], { type: 'application/zip' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.click();
  URL.revokeObjectURL(url);
  return bundle.manifest;
}

/**
 * Downloads the BENG report as a standalone HTML file.
 */
export function downloadReportHTML(project: IProject, result: IBENGResult): void {
  const html = generateReportHTML(project, result);
  const blob = new Blob([html], { type: 'text/html' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `BENG-Rapport-${project.name || 'project'}.html`;
  a.click();
  URL.revokeObjectURL(url);
}

/**
 * Opens the BENG report in a new window and triggers print.
 */
export function printReport(project: IProject, result: IBENGResult): void {
  const html = generateReportHTML(project, result);
  const win = window.open('', '_blank');
  if (win) {
    win.document.write(html);
    win.document.close();
    win.focus();
    win.print();
  }
}
