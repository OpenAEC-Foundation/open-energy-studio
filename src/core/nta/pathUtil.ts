/**
 * Kernel gap and issue paths such as `ntaCalculation.zoneData[0].ventilation.months[3]`
 * or `/zones/0/surfaces/2/windows/1/uValue` (JSON pointer). The step navigation (F3)
 * maps these to a page, tab and field; this module only splits them.
 */
export type PathSegment = string | number;

/** Split a kernel path in dot/bracket form or JSON-pointer form into its segments. */
export function parseKernelPath(path: string | null | undefined): PathSegment[] {
  if (!path) return [];
  const trimmed = path.trim();
  if (trimmed === '' || trimmed === '.' || trimmed === '/') return [];
  if (trimmed.startsWith('/')) {
    return trimmed
      .slice(1)
      .split('/')
      .map((raw) => raw.replace(/~1/g, '/').replace(/~0/g, '~'))
      .map((part) => (/^\d+$/.test(part) ? Number(part) : part));
  }
  const segments: PathSegment[] = [];
  const pattern = /([^.[\]]+)|\[(\d+)\]/g;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(trimmed)) !== null) {
    if (match[2] !== undefined) segments.push(Number(match[2]));
    else segments.push(match[1]);
  }
  return segments;
}

/** The inverse of `parseKernelPath` in dot/bracket form. */
export function formatKernelPath(segments: PathSegment[]): string {
  return segments
    .map((segment, index) => (typeof segment === 'number' ? `[${segment}]` : `${index === 0 ? '' : '.'}${segment}`))
    .join('');
}

/** True when `path` equals `prefix` or lies below it (segment-wise, not string-wise). */
export function isPathWithin(path: string, prefix: string): boolean {
  const a = parseKernelPath(path);
  const b = parseKernelPath(prefix);
  if (b.length > a.length) return false;
  return b.every((segment, index) => segment === a[index]);
}
