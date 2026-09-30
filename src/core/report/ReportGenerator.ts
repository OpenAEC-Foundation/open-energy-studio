import type { IProject, IBENGResult } from '../energy/types';
import { generateReportHTML } from './ReportTemplate';
import { generateNtaInputDossierHTML } from './NtaInputDossier';

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
