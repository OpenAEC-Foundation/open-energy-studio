import { useRef, useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from '../../../i18n/i18n';
import { useEnergy } from '../../../context/EnergyContext';
import { generateReportHTML } from '../../../core/report/ReportTemplate';
import { useKernelQuery } from '../../../context/KernelProvider';
import { DialogShell } from '../DialogShell';
import './PrintPreviewDialog.css';

interface PrinterInfo {
  name: string;
  is_default: boolean;
}

interface PageSize { width: number; height: number; group: string }

const PAGE_SIZES: Record<string, PageSize> = {
  // ISO A series
  A0: { width: 841, height: 1189, group: 'ISO A' },
  A1: { width: 594, height: 841, group: 'ISO A' },
  A2: { width: 420, height: 594, group: 'ISO A' },
  A3: { width: 297, height: 420, group: 'ISO A' },
  A4: { width: 210, height: 297, group: 'ISO A' },
  A5: { width: 148, height: 210, group: 'ISO A' },
  A6: { width: 105, height: 148, group: 'ISO A' },
  // ISO B series
  B0: { width: 1000, height: 1414, group: 'ISO B' },
  B1: { width: 707, height: 1000, group: 'ISO B' },
  B2: { width: 500, height: 707, group: 'ISO B' },
  B3: { width: 353, height: 500, group: 'ISO B' },
  B4: { width: 250, height: 353, group: 'ISO B' },
  B5: { width: 176, height: 250, group: 'ISO B' },
  // ISO C series (envelopes)
  C3: { width: 324, height: 458, group: 'ISO C' },
  C4: { width: 229, height: 324, group: 'ISO C' },
  C5: { width: 162, height: 229, group: 'ISO C' },
  C6: { width: 114, height: 162, group: 'ISO C' },
  // North American
  Letter: { width: 215.9, height: 279.4, group: 'NA' },
  Legal: { width: 215.9, height: 355.6, group: 'NA' },
  Tabloid: { width: 279.4, height: 431.8, group: 'NA' },
  Ledger: { width: 431.8, height: 279.4, group: 'NA' },
  Executive: { width: 184.15, height: 266.7, group: 'NA' },
  'Statement (Half Letter)': { width: 139.7, height: 215.9, group: 'NA' },
  'ANSI C': { width: 431.8, height: 558.8, group: 'NA' },
  'ANSI D': { width: 558.8, height: 863.6, group: 'NA' },
  'ANSI E': { width: 863.6, height: 1117.6, group: 'NA' },
  // Japanese JIS B series
  'JIS B0': { width: 1030, height: 1456, group: 'JIS B' },
  'JIS B1': { width: 728, height: 1030, group: 'JIS B' },
  'JIS B2': { width: 515, height: 728, group: 'JIS B' },
  'JIS B3': { width: 364, height: 515, group: 'JIS B' },
  'JIS B4': { width: 257, height: 364, group: 'JIS B' },
  'JIS B5': { width: 182, height: 257, group: 'JIS B' },
  // Envelopes
  'DL Envelope': { width: 110, height: 220, group: 'Envelope' },
  '#10 Envelope': { width: 104.8, height: 241.3, group: 'Envelope' },
  'Monarch Envelope': { width: 98.4, height: 190.5, group: 'Envelope' },
};

const PAGE_SIZE_GROUPS = ['ISO A', 'ISO B', 'ISO C', 'NA', 'JIS B', 'Envelope'];

const MARGIN_MM = 10;
const MM_TO_PX = 3.7795;
const PAGE_GAP_PX = 24;

interface PrinterSettings {
  paper_width_mm: number;
  paper_height_mm: number;
  landscape: boolean;
}

interface PrintPreviewDialogProps {
  onClose: () => void;
}

function findMatchingPageSize(widthMm: number, heightMm: number): string | null {
  for (const [name, size] of Object.entries(PAGE_SIZES)) {
    if (Math.abs(size.width - widthMm) < 1 && Math.abs(size.height - heightMm) < 1) return name;
  }
  return null;
}

export function PrintPreviewDialog({ onClose }: PrintPreviewDialogProps) {
  const { t, locale } = useI18n();
  const { state } = useEnergy();
  const { project, result } = state;
  const kernelQuery = useKernelQuery(project);
  const kernel = kernelQuery?.kind === 'done' ? kernelQuery.assessment : null;
  const kernelPending = kernelQuery == null || kernelQuery.kind === 'loading';
  const measureRef = useRef<HTMLIFrameElement>(null);
  const previewRef = useRef<HTMLDivElement>(null);
  const [zoom, setZoom] = useState(75);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageBreaks, setPageBreaks] = useState<number[]>([0]);
  const [printers, setPrinters] = useState<PrinterInfo[]>([]);
  const [selectedPrinter, setSelectedPrinter] = useState('');
  const [pageSize, setPageSize] = useState('A4');
  const [landscape, setLandscape] = useState(false);
  const [customSize, setCustomSize] = useState<{ width: number; height: number } | null>(null);
  const [keepTables, setKeepTables] = useState(true);

  const pageDef = customSize ?? PAGE_SIZES[pageSize];
  const pageWidthMm = landscape ? pageDef.height : pageDef.width;
  const pageHeightMm = landscape ? pageDef.width : pageDef.height;
  const usableHeightPx = (pageHeightMm - 2 * MARGIN_MM) * MM_TO_PX;
  const totalPages = pageBreaks.length;

  useEffect(() => {
    invoke<PrinterInfo[]>('list_printers')
      .then((list) => {
        setPrinters(list);
        const def = list.find((p) => p.is_default);
        if (def) setSelectedPrinter(def.name);
        else if (list.length > 0) setSelectedPrinter(list[0].name);
      })
      .catch(() => setPrinters([]));
  }, []);

  // The kernel figures once its run settles; the simplified result only without a kernel result.
  const reportHTML = kernelPending ? '' : (kernel || result ? generateReportHTML(project, result, { kernel, locale }) : '');

  // Measure content and compute intelligent page breaks
  useEffect(() => {
    const iframe = measureRef.current;
    if (!iframe || !reportHTML) return;

    iframe.style.width = `${pageWidthMm - 2 * MARGIN_MM}mm`;

    const doc = iframe.contentDocument;
    if (!doc) return;
    const html = reportHTML.replace('</head>',
      '<style>body{margin:0!important;padding:0!important}</style></head>');
    doc.open();
    doc.write(html);
    doc.close();

    const timer = setTimeout(() => {
      if (!doc.body) return;
      const totalH = doc.body.scrollHeight;
      const bodyTop = doc.body.getBoundingClientRect().top;

      // Collect candidate break points (tops of block-level elements)
      const elements = doc.body.querySelectorAll(
        'p, h1, h2, h3, h4, h5, h6, table, tr, li, div, section, figure, pre, blockquote, hr, dt, dd'
      );
      const tops = new Set<number>();
      for (const el of elements) {
        const top = Math.round(el.getBoundingClientRect().top - bodyTop);
        if (top > 0) tops.add(top);
      }
      const sortedTops = [...tops].sort((a, b) => a - b);

      // Collect heading positions — headings must not be orphaned at page bottom
      const headingTops = new Set<number>();
      const headings = doc.body.querySelectorAll('h1, h2, h3, h4, h5, h6');
      for (const h of headings) {
        headingTops.add(Math.round(h.getBoundingClientRect().top - bodyTop));
      }

      // Collect table ranges for keepTables logic
      const tableRanges: { top: number; bottom: number }[] = [];
      if (keepTables) {
        const tables = doc.body.querySelectorAll('table');
        for (const tbl of tables) {
          const rect = tbl.getBoundingClientRect();
          tableRanges.push({
            top: Math.round(rect.top - bodyTop),
            bottom: Math.round(rect.bottom - bodyTop),
          });
        }
      }

      const breaks: number[] = [0];
      let currentStart = 0;

      while (currentStart + usableHeightPx < totalH) {
        const pageEnd = currentStart + usableHeightPx;
        let breakAt = pageEnd;

        // If keepTables, check if a table crosses the page boundary
        if (keepTables) {
          for (const tbl of tableRanges) {
            if (tbl.top > currentStart && tbl.top < pageEnd && tbl.bottom > pageEnd) {
              if (tbl.bottom - tbl.top <= usableHeightPx && tbl.top > currentStart + 50) {
                breakAt = tbl.top;
                break;
              }
            }
          }
        }

        // If no table-based break, use element-top breaks
        if (breakAt === pageEnd) {
          for (let i = sortedTops.length - 1; i >= 0; i--) {
            const t = sortedTops[i];
            if (t <= currentStart) break;
            if (t <= pageEnd && t > currentStart + 50) {
              breakAt = t;
              break;
            }
          }
        }

        // Don't orphan headings: if a heading is near the bottom of this page
        // (within last 100px before the break), move break before the heading
        for (const ht of headingTops) {
          if (ht >= breakAt - 100 && ht < breakAt && ht > currentStart + 50) {
            breakAt = ht;
            break;
          }
        }

        breaks.push(breakAt);
        currentStart = breakAt;
      }

      setPageBreaks(breaks);
    }, 200);
    return () => clearTimeout(timer);
  }, [reportHTML, pageWidthMm, usableHeightPx, keepTables]);

  // Generate HTML for a specific page using computed break positions
  const getPageHTML = useCallback((pageIndex: number) => {
    const offsetPx = pageBreaks[pageIndex] ?? 0;
    const nextBreak = pageIndex < pageBreaks.length - 1
      ? pageBreaks[pageIndex + 1]
      : offsetPx + usableHeightPx;
    const clipHeight = nextBreak - offsetPx;
    const css = `<style>
body{margin:0!important;padding:0!important}
.page-clip{height:${clipHeight}px;overflow:hidden}
.page-content{transform:translateY(-${offsetPx}px)}
</style>`;
    let html = reportHTML.replace('</head>', css + '</head>');
    html = html.replace(/<body([^>]*)>/, '<body$1><div class="page-clip"><div class="page-content">');
    html = html.replace('</body>', '</div></div></body>');
    return html;
  }, [reportHTML, pageBreaks, usableHeightPx]);

  const handleContainerScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const el = e.currentTarget;
    const pageStepPx = (pageHeightMm * MM_TO_PX + PAGE_GAP_PX) * (zoom / 100);
    const pg = Math.floor(el.scrollTop / pageStepPx) + 1;
    setCurrentPage(Math.min(pg, totalPages));
  }, [totalPages, zoom, pageHeightMm]);

  const handlePrint = useCallback(() => {
    const frame = document.createElement('iframe');
    frame.style.cssText = 'position:absolute;left:-9999px;width:0;height:0';
    document.body.appendChild(frame);
    const doc = frame.contentDocument;
    if (!doc) return;
    doc.open();
    doc.write(reportHTML);
    doc.close();
    setTimeout(() => {
      frame.contentWindow?.focus();
      frame.contentWindow?.print();
      setTimeout(() => document.body.removeChild(frame), 1000);
    }, 200);
  }, [reportHTML]);

  const handlePrinterProperties = useCallback(async () => {
    if (!selectedPrinter) return;
    try {
      const settings = await invoke<PrinterSettings | null>('open_printer_properties', { printer: selectedPrinter });
      if (!settings) return;
      setLandscape(settings.landscape);
      const w = settings.landscape ? settings.paper_height_mm : settings.paper_width_mm;
      const h = settings.landscape ? settings.paper_width_mm : settings.paper_height_mm;
      const match = findMatchingPageSize(w, h);
      if (match) {
        setPageSize(match);
        setCustomSize(null);
      } else {
        setCustomSize({ width: w, height: h });
      }
    } catch (e) {
      console.error('Failed to get printer properties:', e);
    }
  }, [selectedPrinter]);

  const handleZoomIn = useCallback(() => {
    setZoom(prev => Math.min(prev + 25, 200));
  }, []);

  const handleZoomOut = useCallback(() => {
    setZoom(prev => Math.max(prev - 25, 25));
  }, []);

  // Ctrl+scroll to zoom on the preview area
  useEffect(() => {
    const el = previewRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (!e.ctrlKey) return;
      e.preventDefault();
      setZoom(prev => {
        const delta = e.deltaY > 0 ? -5 : 5;
        return Math.min(200, Math.max(25, prev + delta));
      });
    };
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  }, []);

  if (!result) {
    return (
      <DialogShell
        title={t('report.printPreview')}
        onClose={onClose}
        footer={
          <div className="dialog-footer">
            <button className="btn" onClick={onClose}>{t('dialog.close')}</button>
          </div>
        }
      >
        <div className="print-preview-empty">
          <p>{t('results.noResults')}</p>
        </div>
      </DialogShell>
    );
  }

  return (
    <DialogShell
      title={t('report.printPreview')}
      onClose={onClose}
      className="print-preview-dialog"
      bodyClassName="print-preview-body"
      footer={null}
    >
      {/* ── Left sidebar ── */}
      <div className="pp-sidebar">
        <div className="pp-sidebar-content">
          {/* Printer */}
          <div className="pp-section">
            <label className="pp-label">{t('report.printer')}</label>
            <div className="pp-printer-row">
              <select
                className="pp-select"
                value={selectedPrinter}
                onChange={(e) => setSelectedPrinter(e.target.value)}
              >
                {printers.length === 0 && (
                  <option value="">{t('report.noPrinters')}</option>
                )}
                {printers.map((p) => (
                  <option key={p.name} value={p.name}>
                    {p.name}{p.is_default ? ` (${t('report.default')})` : ''}
                  </option>
                ))}
              </select>
              <button
                className="btn pp-props-btn"
                onClick={handlePrinterProperties}
                disabled={!selectedPrinter}
                title={t('report.printerProperties')}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="12" cy="12" r="3" />
                  <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
                </svg>
              </button>
            </div>
          </div>

          {/* Page size */}
          <div className="pp-section">
            <label className="pp-label">{t('report.pageSize')}</label>
            <select
              className="pp-select"
              value={customSize ? '' : pageSize}
              onChange={(e) => { setPageSize(e.target.value); setCustomSize(null); }}
            >
              {customSize && (
                <option value="">
                  {t('report.printerProperties')} ({customSize.width.toFixed(0)} × {customSize.height.toFixed(0)} mm)
                </option>
              )}
              {PAGE_SIZE_GROUPS.map((group) => (
                <optgroup key={group} label={group}>
                  {Object.entries(PAGE_SIZES)
                    .filter(([, s]) => s.group === group)
                    .map(([name, s]) => (
                      <option key={name} value={name}>
                        {name} ({s.width} × {s.height} mm)
                      </option>
                    ))}
                </optgroup>
              ))}
            </select>
          </div>

          {/* Orientation */}
          <div className="pp-section">
            <label className="pp-label">{t('report.orientation')}</label>
            <select
              className="pp-select"
              value={landscape ? 'landscape' : 'portrait'}
              onChange={(e) => setLandscape(e.target.value === 'landscape')}
            >
              <option value="portrait">{t('report.portrait')}</option>
              <option value="landscape">{t('report.landscape')}</option>
            </select>
          </div>

          {/* Keep tables together */}
          <div className="pp-section">
            <label className="pp-checkbox-label">
              <input
                type="checkbox"
                checked={keepTables}
                onChange={(e) => setKeepTables(e.target.checked)}
              />
              {t('report.keepTables')}
            </label>
          </div>

          {/* Zoom */}
          <div className="pp-section">
            <label className="pp-label">{t('report.zoom')}</label>
            <div className="pp-zoom-row">
              <button className="btn pp-zoom-btn" onClick={handleZoomOut}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="11" cy="11" r="8" />
                  <line x1="21" y1="21" x2="16.65" y2="16.65" />
                  <line x1="8" y1="11" x2="14" y2="11" />
                </svg>
              </button>
              <span className="pp-zoom-value">{zoom}%</span>
              <button className="btn pp-zoom-btn" onClick={handleZoomIn}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <circle cx="11" cy="11" r="8" />
                  <line x1="21" y1="21" x2="16.65" y2="16.65" />
                  <line x1="11" y1="8" x2="11" y2="14" />
                  <line x1="8" y1="11" x2="14" y2="11" />
                </svg>
              </button>
            </div>
          </div>

          {/* Page info */}
          <div className="pp-section">
            <span className="pp-page-info">
              {t('report.page')} {currentPage} / {totalPages}
            </span>
          </div>
        </div>

        {/* Buttons pinned to bottom */}
        <div className="pp-sidebar-actions">
          <button className="btn btn-primary pp-print-btn" onClick={handlePrint}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="6 9 6 2 18 2 18 9" />
              <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" />
              <rect x="6" y="14" width="12" height="8" />
            </svg>
            {t('report.print')}
          </button>
          <button className="btn pp-close-btn" onClick={onClose}>
            {t('dialog.close')}
          </button>
        </div>
      </div>

      {/* ── Right preview area ── */}
      <div className="pp-preview" ref={previewRef} onScroll={handleContainerScroll}>
        {/* Hidden iframe for measuring total content height */}
        <iframe
          ref={measureRef}
          style={{ position: 'absolute', left: '-9999px', visibility: 'hidden', height: '99999px' }}
          title="Measurement"
          aria-hidden="true"
          tabIndex={-1}
        />
        <div className="pp-pages-wrapper" style={{ zoom: zoom / 100 }}>
          {Array.from({ length: totalPages }, (_, i) => (
            <div
              key={i}
              className="pp-page"
              style={{
                width: `${pageWidthMm}mm`,
                height: `${pageHeightMm}mm`,
                padding: `${MARGIN_MM}mm`,
                boxSizing: 'border-box',
              }}
            >
              <iframe
                srcDoc={getPageHTML(i)}
                className="pp-iframe"
                title={`Page ${i + 1}`}
                scrolling="no"
              />
            </div>
          ))}
        </div>
      </div>

    </DialogShell>
  );
}
