/**
 * PDF output of the reports (feedback 8 Oct 2026: downloads give a pdf, not an
 * html file). A report page or HTML document is laid out by the browser and
 * drawn into an A4 pdf with jsPDF and html2canvas; text stays text. The Inter
 * font (OFL, public/fonts/pdf) is embedded so ₂, ≤, Δ, ψ and χ render.
 */

const MARGIN_PT = 36;

function fontUrl(name: string): string {
  const base = (import.meta.env?.BASE_URL as string | undefined) ?? '/';
  return `${base.endsWith('/') ? base : `${base}/`}fonts/pdf/${name}`;
}

async function base64(url: string): Promise<string> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`font ${url}: ${response.status}`);
  const bytes = new Uint8Array(await response.arrayBuffer());
  let text = '';
  for (let index = 0; index < bytes.length; index += 0x8000) {
    text += String.fromCharCode(...bytes.subarray(index, index + 0x8000));
  }
  return btoa(text);
}

/** Draws an element (laid out in its own document) into an A4 pdf. */
export async function elementToPdf(element: HTMLElement): Promise<Blob> {
  const [{ jsPDF }, html2canvas] = await Promise.all([import('jspdf'), import('html2canvas').then((module) => module.default)]);
  // jsPDF.html looks html2canvas up on the window.
  (window as unknown as { html2canvas: unknown }).html2canvas = html2canvas;
  const doc = new jsPDF({ unit: 'pt', format: 'a4', compress: true });
  let fontFamily = 'helvetica';
  try {
    const [regular, bold] = await Promise.all([base64(fontUrl('Inter-Regular.ttf')), base64(fontUrl('Inter-Bold.ttf'))]);
    doc.addFileToVFS('Inter-Regular.ttf', regular);
    doc.addFont('Inter-Regular.ttf', 'Inter', 'normal');
    doc.addFileToVFS('Inter-Bold.ttf', bold);
    doc.addFont('Inter-Bold.ttf', 'Inter', 'bold');
    fontFamily = 'Inter';
  } catch { /* the standard font still gives a pdf */ }
  doc.setFont(fontFamily, 'normal');
  const previous = element.style.fontFamily;
  element.style.fontFamily = `${fontFamily}, sans-serif`;
  const width = doc.internal.pageSize.getWidth() - 2 * MARGIN_PT;
  const windowWidth = Math.max(element.scrollWidth, 720);
  try {
    await new Promise<void>((resolve, reject) => {
      try {
        void doc.html(element, {
          callback: () => resolve(),
          margin: [MARGIN_PT, MARGIN_PT, MARGIN_PT, MARGIN_PT],
          autoPaging: 'text',
          width,
          windowWidth,
          html2canvas: { backgroundColor: '#ffffff', useCORS: true },
        });
      } catch (error) {
        reject(error);
      }
    });
  } finally {
    element.style.fontFamily = previous;
  }
  return doc.output('blob');
}

/** Lays out a whole HTML document (a report template) in a hidden frame and draws it into a pdf. */
export async function htmlToPdf(html: string): Promise<Blob> {
  const frame = document.createElement('iframe');
  frame.setAttribute('aria-hidden', 'true');
  frame.style.cssText = 'position:fixed;left:-10000px;top:0;width:800px;height:1200px;border:0;visibility:hidden;';
  document.body.appendChild(frame);
  try {
    const loaded = new Promise<void>((resolve) => { frame.onload = () => resolve(); });
    frame.srcdoc = html;
    await loaded;
    const body = frame.contentDocument?.body;
    if (!body) throw new Error('report frame without body');
    // Light background and dark text for paper, whatever the report's screen styling.
    body.style.background = '#ffffff';
    return await elementToPdf(body);
  } finally {
    frame.remove();
  }
}

/** Saves a pdf: the desktop save dialog in Tauri, a browser download otherwise. */
export async function savePdf(fileName: string, blob: Blob): Promise<void> {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeFile } = await import('@tauri-apps/plugin-fs');
    const path = await save({ defaultPath: fileName, filters: [{ name: 'PDF', extensions: ['pdf'] }] });
    if (path) {
      await writeFile(path, new Uint8Array(await blob.arrayBuffer()));
      return;
    }
    if (path === null) return;
  } catch { /* browser fallback */ }
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/** A file name part from a project name. */
export function fileNamePart(name: string | undefined): string {
  return (name || 'project').replace(/[^\p{L}\p{N}._-]+/gu, '-');
}
