import type { IProject, ProjectImportIssue } from '../energy/types';

/**
 * What an importer did with the foreign file beyond copying values
 * (BRL 9500/9501: the adviser must be able to see which data came from
 * another tool and what the program filled in):
 * - `defaulted`: a value was missing and a default was used;
 * - `invalid`: a value could not be read (or was out of range) and a
 *   default was used instead;
 * - `clamped`: a value lay outside its range and was limited to it;
 * - `dropped`: data in the file (or needed by the NTA calculation) could
 *   not be carried over;
 * - `unknown_reference`: a reference to an element that is not in the file;
 * - `duplicate`: the same element appeared more than once; the first counts;
 * - `multiple`: the file holds more than one building, unit or object and
 *   only the first was read.
 */
export type ImportIssue = ProjectImportIssue;
export type ImportIssueKind = ProjectImportIssue['kind'];

export interface ImportResult {
  project: IProject;
  fileName?: string;
  /** SHA-256 (hex) of the file as read. */
  sha256?: string;
  /** Program and NTA 8800 version the file names (Uniec 3 meta.json, VABI Algemeen), when it names one. */
  sourceVersion?: string;
  issues: ImportIssue[];
}

/** What a reader returns: the project, its import issues and the version the file names. */
export interface ReadResult {
  project: IProject;
  issues: ImportIssues;
  sourceVersion?: string;
}

/**
 * A file the importer cannot read at all. `code` selects the NL/EN text
 * (`import.error.<code>`); `message` is the Dutch fallback.
 */
export class ImportFormatError extends Error {
  readonly code: string;
  readonly params: Record<string, string | number>;

  constructor(code: string, message: string, params: Record<string, string | number> = {}) {
    super(message);
    this.name = 'ImportFormatError';
    this.code = code;
    this.params = params;
  }
}

/** Own-property lookup: a code such as `constructor` or `__proto__` never resolves to Object.prototype. */
export function lookup<T>(table: Record<string | number, T>, key: string | number): T | undefined {
  return Object.prototype.hasOwnProperty.call(table, key) ? table[key] : undefined;
}

/**
 * Reads a number written by a Dutch or an English tool: "0,35", "0.35",
 * "1.234,5" (thousands dot with decimal comma) and "1 234,5". Null for
 * anything that is not a finite number.
 */
export function parseDecimalText(text: string | undefined | null): number | null {
  if (text == null) return null;
  let value = String(text).trim().replace(/[\s  ]/g, '');
  if (value === '') return null;
  if (value.includes(',')) {
    // A comma is the decimal separator; dots before it are thousands separators.
    if (value.indexOf(',') !== value.lastIndexOf(',')) return null;
    value = value.replace(/\./g, '').replace(',', '.');
  }
  if (!/^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(value)) return null;
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

export interface NumberRule {
  /** Used when the value is missing or unreadable; recorded as an issue. */
  fallback: number;
  /** Lower bound; `minExclusive` makes it exclusive (e.g. qv10 > 0). */
  min?: number;
  minExclusive?: boolean;
  max?: number;
  /** Limit an out-of-range value to the bound instead of using the fallback. */
  clamp?: boolean;
  /** No issue when the value is missing (the fallback is the documented meaning of "absent"). */
  silentWhenMissing?: boolean;
  /** Why the fallback is used, for the issue text. */
  fallbackReason?: string;
}

/** Collects the issues of one import. */
export class ImportIssues {
  readonly list: ImportIssue[] = [];

  add(kind: ImportIssueKind, path: string, message: string, value?: string | number): void {
    this.list.push({ path, kind, message, ...(value === undefined ? {} : { value }) });
  }

  /**
   * Reads `text` as a number under `rule`. A missing value gives the
   * fallback (`defaulted`), an unreadable one the fallback (`invalid`), an
   * out-of-range one the bound (`clamped`) or the fallback (`invalid`).
   */
  number(path: string, text: string | undefined | null, rule: NumberRule): number {
    const reason = rule.fallbackReason ? ` (${rule.fallbackReason})` : '';
    if (text == null || String(text).trim() === '') {
      if (!rule.silentWhenMissing) {
        this.add('defaulted', path, `Waarde ontbreekt in het bestand; ${rule.fallback} aangenomen${reason}.`, rule.fallback);
      }
      return rule.fallback;
    }
    const value = parseDecimalText(text);
    if (value === null) {
      this.add('invalid', path, `"${String(text).slice(0, 40)}" is geen getal; ${rule.fallback} aangenomen${reason}.`, String(text).slice(0, 40));
      return rule.fallback;
    }
    const belowMin = rule.min !== undefined && (rule.minExclusive ? value <= rule.min : value < rule.min);
    const aboveMax = rule.max !== undefined && value > rule.max;
    if (!belowMin && !aboveMax) return value;
    if (rule.clamp) {
      const bound = belowMin ? rule.min! : rule.max!;
      this.add('clamped', path, `${value} ligt buiten het bereik; begrensd op ${bound}.`, value);
      return bound;
    }
    this.add('invalid', path, `${value} ligt buiten het bereik; ${rule.fallback} aangenomen${reason}.`, value);
    return rule.fallback;
  }
}

/** Fields each format carries; anything else in the project is left out on export. */
const EXPORTED = {
  project: ['id', 'name', 'description', 'buildingFunction', 'address', 'city', 'zones', 'heatingSystems',
    'ventilationSystems', 'coolingSystems', 'hotWaterSystems', 'solarPV', 'solarThermal', 'constructions'],
  zone: ['id', 'name', 'floorArea', 'volume', 'height', 'surfaces', 'thermalBridges', 'airTightness'],
  surface: ['id', 'name', 'type', 'area', 'orientation', 'constructionId', 'zoneId', 'windows'],
  window: ['id', 'name', 'area', 'uValue', 'gValue', 'orientation', 'surfaceId'],
  thermalBridge: ['id', 'name', 'psiValue', 'length', 'zoneId'],
  // Installations: the fields both writers put in the file; anything else (the NTA heat-pump
  // classification, a comment) is not in either format.
  heatingSystems: ['id', 'name', 'type', 'cop', 'coverageFraction'],
  ventilationSystems: ['id', 'name', 'type', 'heatRecoveryEfficiency', 'sfp'],
  coolingSystems: ['id', 'name', 'type', 'eer'],
  hotWaterSystems: ['id', 'name', 'type', 'efficiency', 'hasSolarBoiler', 'solarBoilerFraction'],
  solarPV: ['id', 'name', 'peakPower', 'orientation', 'tilt', 'area'],
  solarThermal: ['id', 'name', 'collectorArea', 'type', 'orientation', 'tilt'],
} as const;

const SYSTEM_LISTS = ['heatingSystems', 'ventilationSystems', 'coolingSystems', 'hotWaterSystems', 'solarPV', 'solarThermal'] as const;

/**
 * Whether a COP, EER or efficiency is not a usable value: 0 or below (or not
 * a number). Both formats' importers refuse it, so the writers leave it out
 * and the export warning names it. An EER on cooling of type "none" is not
 * used and may be 0; it is written as it is.
 */
export function unusableRatio(value: number, notApplicable = false): boolean {
  return !notApplicable && !(Number.isFinite(value) && value > 0);
}

/** Fields the program keeps for itself and never writes to a foreign format. */
const NOT_INPUT = new Set(['importLog']);

/**
 * The project data a UNIEC3 or VABI export leaves out, as path patterns
 * (`zones[*].surfaces[*].thermalBoundary`). Empty lists and absent fields
 * count as nothing lost.
 */
export function droppedOnExport(project: IProject, format: 'UNIEC3' | 'VABI'): string[] {
  const dropped = new Set<string>();
  const present = (item: unknown) => item !== undefined && item !== null && !(Array.isArray(item) && item.length === 0);
  const check = (value: unknown, keep: readonly string[], prefix: string) => {
    if (value == null || typeof value !== 'object') return;
    for (const [key, item] of Object.entries(value as Record<string, unknown>)) {
      if (keep.includes(key) || !present(item)) continue;
      dropped.add(`${prefix}${key}`);
    }
  };
  for (const [key, item] of Object.entries(project)) {
    if (NOT_INPUT.has(key) || (EXPORTED.project as readonly string[]).includes(key) || !present(item)) continue;
    dropped.add(key);
  }
  // Neither format, as far as its files are known here, holds the ground route of NTA 8800 8.3: the
  // perimeter P, a crawlspace or unheated cellar below, a heated basement with z and its wall area.
  // Named on its own so the adviser sees the ground data are not in the file.
  const groundFloors = (project.ntaCalculation as { groundFloors?: unknown[] } | undefined)?.groundFloors;
  if (Array.isArray(groundFloors) && groundFloors.length > 0) dropped.add('ntaCalculation.groundFloors');
  for (const zone of project.zones ?? []) {
    check(zone, EXPORTED.zone, 'zones[*].');
    if (zone.airTightness?.assumed) dropped.add('zones[*].airTightness.assumed');
    for (const surface of zone.surfaces ?? []) {
      check(surface, EXPORTED.surface, 'zones[*].surfaces[*].');
      for (const window of surface.windows ?? []) {
        check(window, EXPORTED.window, 'zones[*].surfaces[*].windows[*].');
        // A VABI Deelvlak has no orientation of its own: a window facing another way than its
        // surface comes back facing the surface's way.
        if (format === 'VABI' && window.orientation !== surface.orientation) {
          dropped.add('zones[*].surfaces[*].windows[*].orientation');
        }
      }
    }
    for (const bridge of zone.thermalBridges ?? []) check(bridge, EXPORTED.thermalBridge, 'zones[*].thermalBridges[*].');
  }
  for (const key of SYSTEM_LISTS) {
    for (const system of project[key] ?? []) check(system, EXPORTED[key], `${key}[*].`);
  }
  // A COP, EER or efficiency of 0 is not a value either importer takes; it is left out of the file.
  if ((project.heatingSystems ?? []).some((system) => unusableRatio(system.cop))) dropped.add('heatingSystems[*].cop');
  if ((project.coolingSystems ?? []).some((system) => unusableRatio(system.eer, system.type === 'none'))) {
    dropped.add('coolingSystems[*].eer');
  }
  if ((project.hotWaterSystems ?? []).some((system) => unusableRatio(system.efficiency))) {
    dropped.add('hotWaterSystems[*].efficiency');
  }
  // VABI carries a construction's layers; UNIEC3 only its Rc and U.
  const constructionKeep = format === 'VABI'
    ? ['id', 'name', 'layers', 'rcValue', 'uValue']
    : ['id', 'name', 'rcValue', 'uValue'];
  for (const construction of project.constructions ?? []) check(construction, constructionKeep, 'constructions[*].');
  // UNIEC3 holds a construction only on the surfaces that use it (VABI also has its library):
  // a construction no surface uses is not in the file.
  if (format === 'UNIEC3') {
    const used = new Set((project.zones ?? []).flatMap((zone) => (zone.surfaces ?? []).map((surface) => surface.constructionId)));
    if ((project.constructions ?? []).some((construction) => !used.has(construction.id))) dropped.add('constructions[unused]');
  }
  return [...dropped].sort();
}

/**
 * The NTA 8800 input neither format carries; recorded once per import so
 * the adviser knows the NTA input still has to be entered.
 */
export function ntaInputNotInFormat(issues: ImportIssues, tool: string): void {
  issues.add('dropped', 'ntaCalculation',
    `${tool} bevat geen NTA 8800-rekeninvoer (rekenblok, grens per vlak, vloeren op grond met perimeter, kruipruimte of verwarmde kelder, onverwarmde ruimten, puntkoudebruggen); vul die aan bij de NTA-invoer.`);
}

/**
 * The program and NTA 8800 version the foreign file names, recorded once per
 * import: the importer carries geometry and installations over, never the
 * edition or the calculation settings of the other program, so the adviser
 * checks the edition of the NTA input (an older Uniec 3 file was made under
 * an older edition).
 */
export function sourceVersionNotCarried(issues: ImportIssues, tool: string, source: string | undefined): void {
  issues.add('dropped', 'normVersion', source
    ? `${tool}-bestand gemaakt met ${source}; de NTA 8800-editie en de rekeninstellingen van dat programma worden niet overgenomen. Dit project rekent met de editie van de NTA-invoer; controleer die.`
    : `${tool}-bestand zonder programma- of NTA 8800-versie; de editie en de rekeninstellingen worden niet overgenomen. Dit project rekent met de editie van de NTA-invoer; controleer die.`,
  source ? source.slice(0, 80) : undefined);
}

/** Longest list of names in one "not read" issue; the rest is counted. */
const UNREAD_NAMES_SHOWN = 20;

/**
 * Records data the importer saw but did not read: `names` maps each element
 * or property name to how often it occurred, under `path` (an entity type or
 * a parent element). One issue per path, so a rich file gives a readable list.
 */
export function unreadData(issues: ImportIssues, path: string, what: string, names: Map<string, number>): void {
  if (names.size === 0) return;
  const sorted = [...names.entries()].sort(([a], [b]) => a.localeCompare(b));
  const shown = sorted.slice(0, UNREAD_NAMES_SHOWN).map(([name, count]) => (count > 1 ? `${name} (${count}×)` : name));
  const more = sorted.length > UNREAD_NAMES_SHOWN ? ` en nog ${sorted.length - UNREAD_NAMES_SHOWN}` : '';
  issues.add('dropped', path, `${what} niet ingelezen: ${shown.join(', ')}${more}.`, sorted.reduce((sum, [, count]) => sum + count, 0));
}

/** Strips a UTF-8 byte-order mark. */
export function withoutBom(text: string): string {
  return text.charCodeAt(0) === 0xfeff ? text.slice(1) : text;
}
