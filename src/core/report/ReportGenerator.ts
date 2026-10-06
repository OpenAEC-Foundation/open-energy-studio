import type { IProject, IBENGResult } from '../energy/types';
import { generateReportHTML } from './ReportTemplate';
import { generateNtaInputDossierHTML } from './NtaInputDossier';
import { generateNtaCalculationReportHTML } from './NtaCalculationReport';
import { calculateProjectPerformanceShared } from '../nta/useProjectPerformance';
import { calculateProjectPerformanceWithRust, fetchKernelInterpretations } from '../nta/KernelClient';
import { buildProjectDossier, zipProjectDossier } from './ProjectDossier';
import { assessStoredSurvey } from '../nta/SurveyTemplates';
import { assessMaatwerkadviesWithRust } from '../nta/KernelClient';
import { generateMaatwerkadviesReportHTML } from './MaatwerkadviesReport';
import { generateEnergyPerformanceReportHTML, type ReportOptions } from './EnergyPerformanceReport';

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

/** The kernel's interpretation lists for the report appendix; empty when unavailable. */
async function interpretationsOrEmpty() {
  try {
    return await fetchKernelInterpretations();
  } catch {
    return [];
  }
}

/** Runs the Rust project chain and downloads its unverified calculation report. */
export async function downloadNtaCalculationReportHTML(project: IProject): Promise<void> {
  const assessment = await calculateProjectPerformanceWithRust(project);
  const html = generateNtaCalculationReportHTML(project, assessment, await interpretationsOrEmpty());
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
  // One moment for the report's <time datetime> and the manifest's generatedAt.
  const generatedAt = new Date();
  const reportHtml = assessment
    ? generateNtaCalculationReportHTML(project, assessment, await interpretationsOrEmpty(), generatedAt)
    : null;
  const opname = await assessStoredSurvey(project.basisopname, project.ntaCalculation?.normVersion);
  const bundle = await buildProjectDossier({ project, assessment, opname, reportHtml, generatedAt: generatedAt.toISOString() });
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

/** Saves a text file: the desktop save dialog in Tauri, a browser download otherwise. */
async function saveTextFile(fileName: string, text: string, extension: string, filterName: string): Promise<void> {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeFile } = await import('@tauri-apps/plugin-fs');
    const path = await save({ defaultPath: fileName, filters: [{ name: filterName, extensions: [extension] }] });
    if (path) {
      await writeFile(path, new TextEncoder().encode(text));
      return;
    }
    if (path === null) return;
  } catch { /* browser fallback */ }
  const blob = new Blob([text], { type: 'text/html;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.click();
  URL.revokeObjectURL(url);
}

/** The "Rapportage Energieprestatie (NTA 8800)" HTML for the chosen level and detail chapters. */
export async function buildEnergyPerformanceReport(project: IProject, options: ReportOptions): Promise<string> {
  const assessment = await calculateProjectPerformanceShared(project);
  return generateEnergyPerformanceReportHTML(project, assessment, { ...options, interpretations: options.interpretations ?? await interpretationsOrEmpty() });
}

/** Saves the "Rapportage Energieprestatie (NTA 8800)" as a printable HTML file. */
export async function downloadEnergyPerformanceReportHTML(project: IProject, options: ReportOptions): Promise<void> {
  const html = await buildEnergyPerformanceReport(project, options);
  const name = (project.name || 'project').replace(/[^\p{L}\p{N}._-]+/gu, '-');
  await saveTextFile(`Rapportage-Energieprestatie-${name}.html`, html, 'html', 'HTML');
}

/** Opens the "Rapportage Energieprestatie (NTA 8800)" in a window and starts printing (or saving as PDF). */
export async function printEnergyPerformanceReport(project: IProject, options: ReportOptions): Promise<void> {
  const html = await buildEnergyPerformanceReport(project, options);
  const win = window.open('', '_blank');
  if (win) {
    win.document.write(html);
    win.document.close();
    win.focus();
    win.print();
  }
}

/** The kernel assessment for the BENG report, or null when the kernel is unavailable. */
async function kernelOrNull(project: IProject) {
  try {
    return await calculateProjectPerformanceShared(project);
  } catch {
    return null;
  }
}

/**
 * Downloads the BENG report as a standalone HTML file, in the UI language. The figures
 * come from the NTA kernel; the simplified result is only used when the kernel has none.
 */
export async function downloadReportHTML(project: IProject, result: IBENGResult | null, locale?: string): Promise<void> {
  const html = generateReportHTML(project, result, { kernel: await kernelOrNull(project), locale });
  const blob = new Blob([html], { type: 'text/html;charset=utf-8' });
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
export async function printReport(project: IProject, result: IBENGResult | null, locale?: string): Promise<void> {
  const html = generateReportHTML(project, result, { kernel: await kernelOrNull(project), locale });
  const win = window.open('', '_blank');
  if (win) {
    win.document.write(html);
    win.document.close();
    win.focus();
    win.print();
  }
}
