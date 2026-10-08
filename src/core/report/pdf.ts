/**
 * PDF output of the reports (feedback 8 Oct 2026: downloads give a pdf, not an
 * html file). A report page or HTML document is laid out by the browser and
 * drawn into an A4 pdf with jsPDF and html2canvas; text stays text. The Inter
 * font (OFL, public/fonts/pdf) is embedded so ₂, ≤, Δ, ψ and χ render.
 *
 * House style of Open Energy Studio on a white page (feedback 8 Oct 2026):
 * headings in Space Grotesk, text in Inter, a dark band with the amber mark,
 * amber rules under headings and on table heads.
 */

/** Colours of src/styles/tokens.css. */
const FORGE_900 = '#2A2A32';
const FORGE_975 = '#1F1F25';
const AMBER_500 = '#F59E0B';
const AMBER_600 = '#D97706';
const AMBER_700 = '#B45309';

const BRAND_CSS = `
.pdf-brand { background: #ffffff !important; color: ${FORGE_975} !important; font-family: Inter, sans-serif !important; }
.pdf-brand h1, .pdf-brand h2, .pdf-brand h3, .pdf-brand h4 { font-family: SpaceGrotesk, Inter, sans-serif !important; color: ${FORGE_900} !important; }
.pdf-brand h1, .pdf-brand h2 { border-bottom: 3px solid ${AMBER_500} !important; padding-bottom: 4px !important; }
.pdf-brand h3 { border-bottom: 1px solid ${AMBER_500} !important; }
.pdf-brand th { background: #FEF3C7 !important; color: ${FORGE_900} !important; border-bottom: 1px solid ${AMBER_600} !important; }
.pdf-brand a { color: ${AMBER_700} !important; }
.pdf-brand-band { display: flex; align-items: center; gap: 10px; background: ${FORGE_900}; color: #ffffff; padding: 10px 14px;
  border-radius: 6px; margin: 0 0 16px; font-family: SpaceGrotesk, Inter, sans-serif; }
.pdf-brand-band i { display: inline-block; width: 20px; height: 20px; border-radius: 5px; background: ${AMBER_500}; }
.pdf-brand-band b { font-size: 14px; font-weight: 700; }
.pdf-brand-band span { margin-left: auto; font-size: 11px; color: #D4D4D8; font-family: Inter, sans-serif; }
`;

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

/** Draws an element (laid out in its own document) into an A4 pdf in the house style. */
export async function elementToPdf(element: HTMLElement, band?: string): Promise<Blob> {
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
    const [display, displayBold] = await Promise.all([base64(fontUrl('SpaceGrotesk-Medium.ttf')), base64(fontUrl('SpaceGrotesk-Bold.ttf'))]);
    doc.addFileToVFS('SpaceGrotesk-Medium.ttf', display);
    doc.addFont('SpaceGrotesk-Medium.ttf', 'SpaceGrotesk', 'normal');
    doc.addFileToVFS('SpaceGrotesk-Bold.ttf', displayBold);
    doc.addFont('SpaceGrotesk-Bold.ttf', 'SpaceGrotesk', 'bold');
  } catch { /* the standard font still gives a pdf */ }
  doc.setFont(fontFamily, 'normal');
  const previous = element.style.fontFamily;
  element.style.fontFamily = `${fontFamily}, sans-serif`;
  const owner = element.ownerDocument;
  const style = owner.createElement('style');
  style.textContent = BRAND_CSS;
  owner.head.appendChild(style);
  element.classList.add('pdf-brand');
  let header: HTMLElement | null = null;
  if (band !== undefined) {
    header = owner.createElement('div');
    header.className = 'pdf-brand-band';
    header.innerHTML = '<i></i><b>Open Energy Studio</b>';
    const note = owner.createElement('span');
    note.textContent = band;
    header.appendChild(note);
    element.prepend(header);
  }
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
    element.classList.remove('pdf-brand');
    style.remove();
    header?.remove();
  }
  return doc.output('blob');
}

/** Lays out a whole HTML document (a report template) in a hidden frame and draws it into a pdf. */
export async function htmlToPdf(html: string, band = ''): Promise<Blob> {
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
    return await elementToPdf(body, band);
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
