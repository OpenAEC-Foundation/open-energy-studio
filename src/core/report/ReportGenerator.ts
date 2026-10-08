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
import { fileNamePart, htmlToPdf, savePdf } from './pdf';

// Every report download is a pdf (feedback 8 Oct 2026); the HTML templates are laid
// out in a hidden frame and drawn into A4. The function names keep their history.

/** Runs the maatwerkadvies of the project and downloads its report. */
export async function downloadMaatwerkadviesReportHTML(project: IProject): Promise<void> {
  if (!project.maatwerkadvies) throw new Error('Geen maatwerkadvies gedefinieerd.');
  const assessment = await assessMaatwerkadviesWithRust(project, project.maatwerkadvies);
  const html = generateMaatwerkadviesReportHTML(project, project.maatwerkadvies, assessment);
  await savePdf(`Maatwerkadvies-${fileNamePart(project.name)}.pdf`, await htmlToPdf(html, 'NTA 8800:2025+C1:2026'));
}

/** Export the NTA input and evidence inventory even when BENG is unavailable. */
export async function downloadNtaInputDossierHTML(project: IProject): Promise<void> {
  const html = generateNtaInputDossierHTML(project);
  await savePdf(`NTA8800-Invoer-${fileNamePart(project.name)}.pdf`, await htmlToPdf(html, 'NTA 8800:2025+C1:2026'));
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
  await savePdf(`NTA8800-Rekenrapport-${fileNamePart(project.name)}.pdf`, await htmlToPdf(html, 'NTA 8800:2025+C1:2026'));
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
  const opname = await assessStoredSurvey(project.basisopname);
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


/** The "Rapportage Energieprestatie (NTA 8800)" HTML for the chosen level and detail chapters. */
export async function buildEnergyPerformanceReport(project: IProject, options: ReportOptions): Promise<string> {
  const assessment = await calculateProjectPerformanceShared(project);
  return generateEnergyPerformanceReportHTML(project, assessment, { ...options, interpretations: options.interpretations ?? await interpretationsOrEmpty() });
}

/** Saves the "Rapportage Energieprestatie (NTA 8800)" as a printable HTML file. */
export async function downloadEnergyPerformanceReportHTML(project: IProject, options: ReportOptions): Promise<void> {
  const html = await buildEnergyPerformanceReport(project, options);
  await savePdf(`Rapportage-Energieprestatie-${fileNamePart(project.name)}.pdf`, await htmlToPdf(html, 'NTA 8800:2025+C1:2026'));
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
  await savePdf(`BENG-Rapport-${fileNamePart(project.name)}.pdf`, await htmlToPdf(html, 'NTA 8800:2025+C1:2026'));
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
