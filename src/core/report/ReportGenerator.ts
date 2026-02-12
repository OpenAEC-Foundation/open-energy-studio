import type { IProject, IBENGResult } from '../energy/types';
import { generateReportHTML } from './ReportTemplate';

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
