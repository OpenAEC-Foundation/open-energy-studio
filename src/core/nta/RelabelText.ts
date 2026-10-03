/**
 * Display helpers for the relabel comparison (BRL 9500 Bijlage 6a/6b). The kernel names
 * clusters and notes in fixed English sentences and changes by JSON pointer; these map
 * them to translation keys and readable element names.
 */

const CLUSTER_KEYS: Record<string, string> = {
  'geometric change of insulation: layout of windows, doors or panels': 'openingLayout',
  'geometric change: layout (zones, surfaces, windows added or removed)': 'layout',
  'building-bound production added or removed': 'production',
  'geometric change: usable or loss area, dimensions or layout': 'area',
  'geometric change: thermal boundary or zoning': 'boundary',
  'shading, overhang or obstruction': 'shading',
  'geometric change of the installation: shading, overhang or obstruction': 'installationShading',
  lighting: 'lighting',
  'change in distribution, emission or control': 'distribution',
  'installation: system change': 'systemChange',
  'installation part added or removed': 'installationPart',
  'one-to-one replacement or change: installation properties': 'installationProperties',
  'one-to-one replacement or change: insulation properties': 'insulationProperties',
  'not classified': 'notClassified',
  'relabel keeps the original survey date': 'surveyDate',
};

const NOTE_KEYS: Record<string, string> = {
  'allowed (6a) when the loss area per orientation stays equal, otherwise 6b': 'lossAreaOrientation',
  'a one-to-one replacement is allowed (6a), a system change is not (6b)': 'oneToOne',
  'listed under cooling in both 6a (p. 67) and 6b (p. 68); the adviser decides': 'coolingBoth',
  'lighting is not listed in Bijlage 6a or 6b; the adviser decides': 'lightingNotListed',
  'confirm: not insulation on the inside and not of elements outside the thermal zone (6b)': 'insulationConfirm',
  'decide with Bijlage 6a/6b and ISSO 82.1': 'decide',
};

type Translate = (key: string, options?: Record<string, unknown>) => string;

/** The cluster in the UI language; an unknown kernel text is shown as written. */
export function relabelCluster(t: Translate, cluster: string): string {
  const key = CLUSTER_KEYS[cluster];
  return key ? t(`relabel.cluster.${key}`) : cluster;
}

/** The note in the UI language; an unknown kernel text is shown as written. */
export function relabelNote(t: Translate, note: string | undefined): string {
  if (!note) return '';
  const key = NOTE_KEYS[note];
  return key ? t(`relabel.note.${key}`) : note;
}

/**
 * Readable name of a JSON pointer into the project: every array item with a `name`
 * (zone, surface, window, system) is shown by name, other segments as written.
 * `/zones/0/surfaces/1/windows/0/uValue` becomes `Zone A › Voorgevel › Raam 1 › uValue`.
 */
export function relabelElementName(project: unknown, pointer: string): string {
  const segments = pointer.split('/').slice(1).map((segment) => segment.replace(/~1/g, '/').replace(/~0/g, '~'));
  const parts: string[] = [];
  let node: unknown = project;
  for (const segment of segments) {
    const next = node != null && typeof node === 'object' ? (node as Record<string, unknown>)[segment] : undefined;
    if (Array.isArray(node) && /^\d+$/.test(segment)) {
      const name = next != null && typeof next === 'object' ? (next as Record<string, unknown>).name : undefined;
      if (typeof name === 'string' && name.trim()) parts[parts.length - 1] = name;
      else parts.push(`#${Number(segment) + 1}`);
    } else {
      parts.push(segment);
    }
    node = next;
  }
  return parts.join(' › ') || pointer;
}

/** A compared value in the UI language: numbers by locale, booleans as yes/no, objects shortened. */
export function relabelValue(t: Translate, locale: string, value: unknown): string {
  if (value === undefined || value === null) return '–';
  if (typeof value === 'number') return value.toLocaleString(locale, { maximumFractionDigits: 4 });
  if (typeof value === 'boolean') return t(value ? 'common.yes' : 'common.no');
  if (typeof value === 'string') return value.length > 80 ? `${value.slice(0, 77)}…` : value;
  const text = JSON.stringify(value);
  return text.length > 80 ? `${text.slice(0, 77)}…` : text;
}
