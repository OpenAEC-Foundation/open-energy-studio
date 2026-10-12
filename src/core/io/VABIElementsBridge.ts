/**
 * VABI Elements EPA Bridge
 * Import and export between Open Energy Studio IProject and VABI EPA format.
 *
 * Real VABI EPA format: ZIP archive containing project.xml
 * XML structure: Project > Algemeen > Objecten > Object > Rekenzones > Rekenzone
 *   > Geometrie > Hoofdvlak > DeelvlakList > Deelvlak (opaque/transparent)
 *   > KoudebrugList > Koudebrug
 *
 * Enum codes (from analysis of real VABI EPA files):
 *   Locatie: 0=dak, 2=voorgevel, 3=achtergevel, 4=linkergevel, 5=rechtergevel, 6=vloer
 *   Orientatie: 0=N, 1=SW, 2=W, 3=NW, 4=S, 5=NE, 6=E, 7=SE (VABI-specific encoding)
 *   Hellingshoek: 0=0°, 90=90°
 */

import { zipSync, strToU8 } from 'fflate';
import { safeUnzip } from './safeUnzip';
import type {
  IProject, IZone, ISurface, IWindow, IThermalBridge, IConstruction,
  IHeatingSystem, IVentilationSystem, ICoolingSystem, IHotWaterSystem,
  ISolarPV, ISolarThermal,
  Orientation, SurfaceType, HeatingSystemType, VentilationType,
  CoolingSystemType, HotWaterSystemType, SolarThermalType, BuildingFunction,
} from '../energy/types';
import { SURFACE_RESISTANCE } from '../energy/Constants';
import { ASSUMED_QV10 } from '../energy/normalizeProject';
import { buildNtaCalculationTemplate } from '../nta/NtaCalculationTemplate';
import {
  ImportFormatError, ImportIssues, droppedOnExport, lookup, ntaInputNotInFormat, parseDecimalText, unusableRatio,
  sourceVersionNotCarried, unreadData, type ImportResult, type ReadResult,
} from './importIssues';
import { downloadBytes, importFile, pickFile } from './importFile';

// ============================================================
// VABI EPA enum mappings (based on real file analysis)
// ============================================================

// Locatie (surface position): VABI code → our SurfaceType + orientation hint
const LOCATIE_MAP: Record<number, { type: SurfaceType; orientHint?: string }> = {
  0: { type: 'roof' },
  2: { type: 'wall', orientHint: 'front' },    // Voorgevel
  3: { type: 'wall', orientHint: 'rear' },      // Achtergevel
  4: { type: 'wall', orientHint: 'left' },      // Linkergevel
  5: { type: 'wall', orientHint: 'right' },     // Rechtergevel
  6: { type: 'floor' },
  7: { type: 'internal' },                       // Intern
};

const SURFACE_TO_LOCATIE: Record<SurfaceType, number> = {
  'roof': 0, 'wall': 2, 'floor': 6, 'internal': 7,
};

// Orientatie: VABI code → compass direction. Horizontal has no code; the
// tilt (Hellingshoek 0 or 180) says a surface is horizontal.
const VABI_ORIENT_MAP: Record<number, Orientation> = {
  0: 'N', 1: 'SW', 2: 'W', 3: 'NW', 4: 'S', 5: 'NE', 6: 'E', 7: 'SE',
};

const ORIENT_TO_VABI: Record<Orientation, number> = {
  'N': 0, 'SW': 1, 'W': 2, 'NW': 3, 'S': 4, 'NE': 5, 'E': 6, 'SE': 7, 'horizontal': 0,
};

// Building function mapping
const FUNC_TO_VABI: Record<BuildingFunction, string> = {
  'residential': 'Woonfunctie',
  'office': 'Kantoorfunctie',
  'education': 'Onderwijsfunctie',
  'healthcare': 'Gezondheidszorgfunctie',
  'retail': 'Winkelfunctie',
  'industrial': 'Industriefunctie',
  'other': 'Overig',
};

const FUNC_FROM_VABI: Record<string, BuildingFunction> = Object.fromEntries(
  Object.entries(FUNC_TO_VABI).map(([k, v]) => [v, k as BuildingFunction])
) as Record<string, BuildingFunction>;

// Heating type mapping
const HEATING_TO_VABI: Record<HeatingSystemType, number> = {
  'hr107': 1, 'hr_combi': 2, 'heat_pump_air': 3, 'heat_pump_ground': 4,
  'district_heating': 5, 'electric': 6, 'biomass': 7,
};

const HEATING_FROM_VABI: Record<number, HeatingSystemType> = Object.fromEntries(
  Object.entries(HEATING_TO_VABI).map(([k, v]) => [v, k as HeatingSystemType])
) as Record<number, HeatingSystemType>;

// Ventilation type mapping
const VENT_TO_VABI: Record<VentilationType, number> = {
  'natural': 0, 'type_c': 1, 'type_d': 2,
};

const VENT_FROM_VABI: Record<number, VentilationType> = Object.fromEntries(
  Object.entries(VENT_TO_VABI).map(([k, v]) => [v, k as VentilationType])
) as Record<number, VentilationType>;

// Cooling type mapping
const COOL_TO_VABI: Record<CoolingSystemType, number> = {
  'none': 0, 'split_unit': 1, 'central_chiller': 2, 'heat_pump_reversible': 3,
};

const COOL_FROM_VABI: Record<number, CoolingSystemType> = Object.fromEntries(
  Object.entries(COOL_TO_VABI).map(([k, v]) => [v, k as CoolingSystemType])
) as Record<number, CoolingSystemType>;

// Hot water type mapping
const HW_TO_VABI: Record<HotWaterSystemType, number> = {
  'hr_combi': 0, 'heat_pump': 1, 'electric_boiler': 2, 'solar_boiler': 3, 'district_heating': 4,
};

const HW_FROM_VABI: Record<number, HotWaterSystemType> = Object.fromEntries(
  Object.entries(HW_TO_VABI).map(([k, v]) => [v, k as HotWaterSystemType])
) as Record<number, HotWaterSystemType>;

// Solar thermal type
const ST_TO_VABI: Record<SolarThermalType, number> = {
  'flat_plate': 0, 'vacuum_tube': 1,
};

const ST_FROM_VABI: Record<number, SolarThermalType> = Object.fromEntries(
  Object.entries(ST_TO_VABI).map(([k, v]) => [v, k as SolarThermalType])
) as Record<number, SolarThermalType>;

// ============================================================
// XML helpers
// ============================================================

function escapeXml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function xmlTag(name: string, value: string | number | boolean): string {
  const val = typeof value === 'number' ? value.toString() : typeof value === 'boolean' ? (value ? 'true' : 'false') : escapeXml(String(value));
  return `<${name}>${val}</${name}>`;
}

function xmlOpen(name: string): string { return `<${name}>`; }
function xmlClose(name: string): string { return `</${name}>`; }

// DOM parsing helpers
function getEl(parent: Element, tagName: string): Element | null {
  const children = parent.children;
  for (let i = 0; i < children.length; i++) {
    if (children[i].tagName === tagName) return children[i];
  }
  return null;
}

function getText(parent: Element, tagName: string): string {
  const el = getEl(parent, tagName);
  return el?.textContent?.trim() ?? '';
}

/** Element text, or undefined when the element is absent or empty. */
function textOf(parent: Element, tagName: string): string | undefined {
  const text = getText(parent, tagName);
  return text === '' ? undefined : text;
}

function getChildren(parent: Element, tagName: string): Element[] {
  const result: Element[] = [];
  const children = parent.children;
  for (let i = 0; i < children.length; i++) {
    if (children[i].tagName === tagName) result.push(children[i]);
  }
  return result;
}

function getAllDescendants(parent: Element, tagName: string): Element[] {
  return Array.from(parent.getElementsByTagName(tagName));
}

/** An integer code; undefined when absent or not an integer. */
function codeText(parent: Element, tagName: string): number | undefined {
  const text = textOf(parent, tagName);
  return text !== undefined && /^[+-]?\d+$/.test(text) ? Number(text) : undefined;
}

/** A VABI code through its table; a missing or unknown code gives the fallback and an issue. */
function vabiCode<T>(
  issues: ImportIssues, path: string, parent: Element, tagName: string, table: Record<number, T>, fallback: T, what: string,
): T {
  const raw = textOf(parent, tagName);
  if (raw === undefined) {
    issues.add('defaulted', path, `${what} ontbreekt; ${String(fallback)} aangenomen.`, String(fallback));
    return fallback;
  }
  const code = codeText(parent, tagName);
  const value = code === undefined ? undefined : lookup(table, code);
  if (value === undefined) {
    issues.add('invalid', path, `Onbekende code "${raw.slice(0, 40)}" voor ${what.toLowerCase()}; ${String(fallback)} aangenomen.`, raw.slice(0, 40));
    return fallback;
  }
  return value;
}

/** NTA 8800 tilt of a surface, from `ntaCalculation.surfaceTilts`, when known. */
function surfaceTilt(project: IProject, surfaceId: string): number | undefined {
  const tilts = (project.ntaCalculation as { surfaceTilts?: Array<{ surfaceId?: string; tiltDeg?: number | null }> } | undefined)
    ?.surfaceTilts;
  const tilt = Array.isArray(tilts) ? tilts.find((item) => item?.surfaceId === surfaceId)?.tiltDeg : undefined;
  return typeof tilt === 'number' && Number.isFinite(tilt) ? tilt : undefined;
}

/**
 * The Hellingshoek a surface is exported with: its NTA tilt when the
 * project has one; otherwise 0 for horizontal surfaces and floors and 90
 * for walls. A pitched roof without a known tilt gets none, so the import
 * does not invent one.
 */
function exportTilt(project: IProject, surface: ISurface): number | undefined {
  const tilt = surfaceTilt(project, surface.id);
  if (tilt !== undefined) return tilt;
  if (surface.orientation === 'horizontal' || surface.type === 'floor') return 0;
  if (surface.type === 'wall' || surface.type === 'internal') return 90;
  return undefined;
}

// ============================================================
// EXPORT: IProject → VABI EPA ZIP
// ============================================================

export function exportToVABI(project: IProject): Uint8Array {
  const L = (n: number) => '  '.repeat(n);
  const lines: string[] = [];

  lines.push('<?xml version="1.0" encoding="utf-8"?>');
  lines.push('<Project xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">');
  lines.push(`${L(1)}${xmlOpen('Algemeen')}`);
  lines.push(`${L(2)}${xmlTag('Versie', '4.0')}`);
  lines.push(`${L(2)}${xmlTag('Applicatie', 'Open Energy Studio')}`);
  lines.push(`${L(2)}${xmlTag('Datum', new Date().toISOString())}`);
  lines.push(`${L(1)}${xmlClose('Algemeen')}`);

  lines.push(`${L(1)}${xmlOpen('Objecten')}`);
  lines.push(`${L(2)}${xmlOpen('Object')}`);
  lines.push(`${L(3)}${xmlTag('Naam', project.name || 'Naamloos')}`);
  lines.push(`${L(3)}${xmlTag('Omschrijving', project.description || '')}`);
  lines.push(`${L(3)}${xmlTag('Adres', project.address || '')}`);
  lines.push(`${L(3)}${xmlTag('Plaats', project.city || '')}`);
  lines.push(`${L(3)}${xmlTag('Gebruiksfunctie', lookup(FUNC_TO_VABI, project.buildingFunction) ?? 'Woonfunctie')}`);

  // Rekenzones
  lines.push(`${L(3)}${xmlOpen('Rekenzones')}`);
  for (const zone of project.zones) {
    lines.push(`${L(4)}${xmlOpen('Rekenzone')}`);
    lines.push(`${L(5)}${xmlTag('Naam', zone.name)}`);
    lines.push(`${L(5)}${xmlTag('Gebruiksoppervlakte', zone.floorArea)}`);
    lines.push(`${L(5)}${xmlTag('Volume', zone.volume)}`);
    lines.push(`${L(5)}${xmlTag('Hoogte', zone.height)}`);
    lines.push(`${L(5)}${xmlTag('Luchtdoorlatendheid', zone.airTightness.qv10)}`);

    // Geometrie > Hoofdvlakken
    lines.push(`${L(5)}${xmlOpen('Geometrie')}`);
    for (const surface of zone.surfaces) {
      const construction = project.constructions.find(c => c.id === surface.constructionId);
      const tilt = exportTilt(project, surface);

      lines.push(`${L(6)}${xmlOpen('Hoofdvlak')}`);
      lines.push(`${L(7)}${xmlTag('Naam', surface.name)}`);
      lines.push(`${L(7)}${xmlTag('Locatie', lookup(SURFACE_TO_LOCATIE, surface.type) ?? 2)}`);
      lines.push(`${L(7)}${xmlTag('Orientatie', lookup(ORIENT_TO_VABI, surface.orientation) ?? 4)}`);
      if (tilt !== undefined) lines.push(`${L(7)}${xmlTag('Hellingshoek', tilt)}`);
      lines.push(`${L(7)}${xmlTag('Oppervlakte', surface.area)}`);

      // Opaque sub-surfaces (DeelvlakList)
      lines.push(`${L(7)}${xmlOpen('DeelvlakList')}`);

      // Main opaque part (subtract windows). Written whenever the surface has a construction,
      // also when windows fill it: without the Deelvlak the import loses the construction.
      const windowArea = surface.windows.reduce((sum, w) => sum + w.area, 0);
      const opaqueArea = Math.max(0, surface.area - windowArea);

      if (construction) {
        lines.push(`${L(8)}${xmlOpen('Deelvlak')}`);
        lines.push(`${L(9)}${xmlTag('Type', 'Dicht')}`);
        lines.push(`${L(9)}${xmlTag('Naam', construction.name)}`);
        lines.push(`${L(9)}${xmlTag('Oppervlakte', opaqueArea)}`);
        lines.push(`${L(9)}${xmlTag('RcWaarde', construction.rcValue)}`);
        // The U as the project holds it: a construction entered by its U has no Rc to rebuild it from.
        lines.push(`${L(9)}${xmlTag('UWaarde', construction.uValue)}`);
        lines.push(`${L(8)}${xmlClose('Deelvlak')}`);
      }

      // Windows as transparent sub-surfaces
      for (const win of surface.windows) {
        lines.push(`${L(8)}${xmlOpen('Deelvlak')}`);
        lines.push(`${L(9)}${xmlTag('Type', 'Transparant')}`);
        lines.push(`${L(9)}${xmlTag('Naam', win.name)}`);
        lines.push(`${L(9)}${xmlTag('Oppervlakte', win.area)}`);
        lines.push(`${L(9)}${xmlTag('UWaarde', win.uValue)}`);
        lines.push(`${L(9)}${xmlTag('GWaarde', win.gValue)}`);
        lines.push(`${L(8)}${xmlClose('Deelvlak')}`);
      }

      lines.push(`${L(7)}${xmlClose('DeelvlakList')}`);
      lines.push(`${L(6)}${xmlClose('Hoofdvlak')}`);
    }
    lines.push(`${L(5)}${xmlClose('Geometrie')}`);

    // Thermal bridges belong to the zone: written once, not under every surface
    // (a list per Hoofdvlak multiplied ΣψL by the number of surfaces on import).
    lines.push(`${L(5)}${xmlOpen('KoudebrugList')}`);
    for (const tb of zone.thermalBridges) {
      lines.push(`${L(6)}${xmlOpen('Koudebrug')}`);
      lines.push(`${L(7)}${xmlTag('Naam', tb.name)}`);
      lines.push(`${L(7)}${xmlTag('PsiWaarde', tb.psiValue)}`);
      lines.push(`${L(7)}${xmlTag('Lengte', tb.length)}`);
      lines.push(`${L(6)}${xmlClose('Koudebrug')}`);
    }
    lines.push(`${L(5)}${xmlClose('KoudebrugList')}`);

    lines.push(`${L(4)}${xmlClose('Rekenzone')}`);
  }
  lines.push(`${L(3)}${xmlClose('Rekenzones')}`);

  // Installaties
  lines.push(`${L(3)}${xmlOpen('Installaties')}`);

  // Verwarming
  if (project.heatingSystems.length > 0) {
    lines.push(`${L(4)}${xmlOpen('Verwarming')}`);
    for (const hs of project.heatingSystems) {
      lines.push(`${L(5)}${xmlOpen('Opwekker')}`);
      lines.push(`${L(6)}${xmlTag('Naam', hs.name)}`);
      lines.push(`${L(6)}${xmlTag('Type', lookup(HEATING_TO_VABI, hs.type) ?? 1)}`);
      if (!unusableRatio(hs.cop)) lines.push(`${L(6)}${xmlTag('COP', hs.cop)}`);
      lines.push(`${L(6)}${xmlTag('Dekking', hs.coverageFraction)}`);
      lines.push(`${L(5)}${xmlClose('Opwekker')}`);
    }
    lines.push(`${L(4)}${xmlClose('Verwarming')}`);
  }

  // Ventilatie
  if (project.ventilationSystems.length > 0) {
    lines.push(`${L(4)}${xmlOpen('Ventilatie')}`);
    for (const vs of project.ventilationSystems) {
      lines.push(`${L(5)}${xmlOpen('Systeem')}`);
      lines.push(`${L(6)}${xmlTag('Naam', vs.name)}`);
      lines.push(`${L(6)}${xmlTag('Type', lookup(VENT_TO_VABI, vs.type) ?? 0)}`);
      lines.push(`${L(6)}${xmlTag('WTWRendement', vs.heatRecoveryEfficiency)}`);
      lines.push(`${L(6)}${xmlTag('SFP', vs.sfp)}`);
      lines.push(`${L(5)}${xmlClose('Systeem')}`);
    }
    lines.push(`${L(4)}${xmlClose('Ventilatie')}`);
  }

  // Koeling
  if (project.coolingSystems.length > 0) {
    lines.push(`${L(4)}${xmlOpen('Koeling')}`);
    for (const cs of project.coolingSystems) {
      lines.push(`${L(5)}${xmlOpen('Opwekker')}`);
      lines.push(`${L(6)}${xmlTag('Naam', cs.name)}`);
      lines.push(`${L(6)}${xmlTag('Type', lookup(COOL_TO_VABI, cs.type) ?? 0)}`);
      if (!unusableRatio(cs.eer, cs.type === 'none')) lines.push(`${L(6)}${xmlTag('EER', cs.eer)}`);
      lines.push(`${L(5)}${xmlClose('Opwekker')}`);
    }
    lines.push(`${L(4)}${xmlClose('Koeling')}`);
  }

  // Tapwater
  if (project.hotWaterSystems.length > 0) {
    lines.push(`${L(4)}${xmlOpen('Tapwater')}`);
    for (const hw of project.hotWaterSystems) {
      lines.push(`${L(5)}${xmlOpen('Opwekker')}`);
      lines.push(`${L(6)}${xmlTag('Naam', hw.name)}`);
      lines.push(`${L(6)}${xmlTag('Type', lookup(HW_TO_VABI, hw.type) ?? 0)}`);
      if (!unusableRatio(hw.efficiency)) lines.push(`${L(6)}${xmlTag('Rendement', hw.efficiency)}`);
      lines.push(`${L(6)}${xmlTag('Zonneboiler', hw.hasSolarBoiler)}`);
      lines.push(`${L(6)}${xmlTag('ZonneboilerFractie', hw.solarBoilerFraction)}`);
      lines.push(`${L(5)}${xmlClose('Opwekker')}`);
    }
    lines.push(`${L(4)}${xmlClose('Tapwater')}`);
  }

  lines.push(`${L(3)}${xmlClose('Installaties')}`);

  // PV & Solar thermal
  if (project.solarPV.length > 0 || project.solarThermal.length > 0) {
    lines.push(`${L(3)}${xmlOpen('Hernieuwbaar')}`);

    for (const pv of project.solarPV) {
      lines.push(`${L(4)}${xmlOpen('PVSysteem')}`);
      lines.push(`${L(5)}${xmlTag('Naam', pv.name)}`);
      lines.push(`${L(5)}${xmlTag('Piekvermogen', pv.peakPower)}`);
      lines.push(`${L(5)}${xmlTag('Orientatie', lookup(ORIENT_TO_VABI, pv.orientation) ?? 4)}`);
      lines.push(`${L(5)}${xmlTag('Hellingshoek', pv.tilt)}`);
      lines.push(`${L(5)}${xmlTag('Oppervlakte', pv.area)}`);
      lines.push(`${L(4)}${xmlClose('PVSysteem')}`);
    }

    for (const st of project.solarThermal) {
      lines.push(`${L(4)}${xmlOpen('ZonthermischSysteem')}`);
      lines.push(`${L(5)}${xmlTag('Naam', st.name)}`);
      lines.push(`${L(5)}${xmlTag('CollectorOppervlakte', st.collectorArea)}`);
      lines.push(`${L(5)}${xmlTag('Type', lookup(ST_TO_VABI, st.type) ?? 0)}`);
      lines.push(`${L(5)}${xmlTag('Orientatie', lookup(ORIENT_TO_VABI, st.orientation) ?? 4)}`);
      lines.push(`${L(5)}${xmlTag('Hellingshoek', st.tilt)}`);
      lines.push(`${L(4)}${xmlClose('ZonthermischSysteem')}`);
    }

    lines.push(`${L(3)}${xmlClose('Hernieuwbaar')}`);
  }

  lines.push(`${L(2)}${xmlClose('Object')}`);
  lines.push(`${L(1)}${xmlClose('Objecten')}`);

  // Construction library
  lines.push(`${L(1)}${xmlOpen('ConstructieBibliotheek')}`);
  for (const con of project.constructions) {
    lines.push(`${L(2)}${xmlOpen('Constructie')}`);
    lines.push(`${L(3)}${xmlTag('Naam', con.name)}`);
    lines.push(`${L(3)}${xmlTag('RcWaarde', con.rcValue)}`);
    lines.push(`${L(3)}${xmlTag('UWaarde', con.uValue)}`);
    if (con.layers.length > 0) {
      lines.push(`${L(3)}${xmlOpen('Lagen')}`);
      for (const layer of con.layers) {
        lines.push(`${L(4)}${xmlOpen('Laag')}`);
        lines.push(`${L(5)}${xmlTag('Materiaal', layer.material)}`);
        lines.push(`${L(5)}${xmlTag('Dikte', layer.thickness)}`);
        lines.push(`${L(5)}${xmlTag('Lambda', layer.lambda)}`);
        lines.push(`${L(4)}${xmlClose('Laag')}`);
      }
      lines.push(`${L(3)}${xmlClose('Lagen')}`);
    }
    lines.push(`${L(2)}${xmlClose('Constructie')}`);
  }
  lines.push(`${L(1)}${xmlClose('ConstructieBibliotheek')}`);

  lines.push('</Project>');

  const xml = lines.join('\n');

  // Create ZIP with project.xml inside; copy into this realm (see UNIEC3Exporter).
  return zipSync({ 'project.xml': new Uint8Array(strToU8(xml)) }, { level: 6 });
}

/** What a VABI export of this project leaves out (shown to the adviser). */
export function vabiExportLosses(project: IProject): string[] {
  return droppedOnExport(project, 'VABI');
}

// ============================================================
// IMPORT: VABI EPA ZIP → IProject
// ============================================================

/**
 * Decodes an XML file the way its byte-order mark or declaration says
 * (UTF-8, UTF-16, windows-1252, …), not always as UTF-8.
 */
export function decodeXml(bytes: Uint8Array): string {
  const decode = (label: string, data: Uint8Array) => {
    let decoder: TextDecoder;
    try {
      decoder = new TextDecoder(label, { fatal: true });
    } catch {
      throw new ImportFormatError('vabi_encoding_unknown', `Onbekende tekencodering "${label}" in het XML-bestand.`, { encoding: label });
    }
    try {
      return decoder.decode(data);
    } catch {
      throw new ImportFormatError('vabi_encoding_invalid', `Het XML-bestand is niet geldig in de tekencodering "${label}".`, { encoding: label });
    }
  };
  if (bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) return decode('utf-8', bytes.subarray(3));
  if (bytes[0] === 0xff && bytes[1] === 0xfe) return decode('utf-16le', bytes.subarray(2));
  if (bytes[0] === 0xfe && bytes[1] === 0xff) return decode('utf-16be', bytes.subarray(2));
  // UTF-16 without a byte-order mark starts with "<" and a zero byte.
  if (bytes[0] === 0x3c && bytes[1] === 0x00) return decode('utf-16le', bytes);
  if (bytes[0] === 0x00 && bytes[1] === 0x3c) return decode('utf-16be', bytes);
  let head = '';
  for (let i = 0; i < Math.min(bytes.length, 256); i++) head += String.fromCharCode(bytes[i]);
  const declared = head.match(/^<\?xml[^>]*\bencoding\s*=\s*["']([A-Za-z0-9._:-]+)["']/)?.[1];
  return decode(declared ?? 'utf-8', bytes);
}

/** Picks project.xml from an EPA archive; any other XML only when it is the only one. */
function projectXml(files: Record<string, Uint8Array>): Uint8Array {
  const xmlPaths = Object.keys(files).filter((path) => path.toLowerCase().endsWith('.xml'));
  const named = xmlPaths
    .filter((path) => path.split('/').pop()!.toLowerCase() === 'project.xml')
    .sort((a, b) => a.length - b.length);
  if (named.length > 0) return files[named[0]];
  if (xmlPaths.length === 1) return files[xmlPaths[0]];
  throw new ImportFormatError('vabi_no_project_xml',
    xmlPaths.length === 0 ? 'Geen project.xml gevonden in het EPA-bestand.' : 'Geen project.xml gevonden in het EPA-bestand (wel andere XML-bestanden).');
}

/** Imports a VABI EPA archive, with the list of what the program filled in or left out. */
export function readVABI(zipData: Uint8Array): ReadResult {
  // Only XML entries are read, with a size cap, so a crafted archive cannot
  // unpack gigabytes into memory.
  const files = safeUnzip(zipData, (name) => name.toLowerCase().endsWith('.xml'));
  return readVABIXml(decodeXml(projectXml(files)));
}

/** The project of a VABI EPA archive (see `readVABI` for the import issues). */
export function importFromVABI(zipData: Uint8Array): IProject {
  return readVABI(zipData).project;
}

/** Imports a VABI EPA archive or a bare project.xml. */
export function readVABIFile(bytes: Uint8Array): ReadResult {
  return bytes[0] === 0x50 && bytes[1] === 0x4b ? readVABI(bytes) : readVABIXml(decodeXml(bytes));
}

/** Signature of a KoudebrugList, to spot the same list repeated under every Hoofdvlak. */
function bridgeListSignature(listEl: Element | null): string {
  if (!listEl) return '';
  return getChildren(listEl, 'Koudebrug')
    .map((kb) => [getText(kb, 'Naam'), getText(kb, 'PsiWaarde'), getText(kb, 'Lengte')].join('|'))
    .join(';');
}

/**
 * The child elements the importer reads, per element (an Opwekker by its
 * parent, since a heating, cooling and hot-water generator differ). Any other
 * child element is reported as not read.
 */
const READ_ELEMENTS: Record<string, readonly string[]> = {
  Project: ['Algemeen', 'Objecten', 'ConstructieBibliotheek'],
  Algemeen: ['Versie', 'Applicatie', 'Datum'],
  Objecten: ['Object'],
  Object: ['Naam', 'Omschrijving', 'Adres', 'Plaats', 'Gebruiksfunctie', 'Rekenzones', 'Installaties', 'Hernieuwbaar'],
  Rekenzones: ['Rekenzone'],
  Rekenzone: ['Naam', 'Gebruiksoppervlakte', 'Volume', 'Hoogte', 'Luchtdoorlatendheid', 'Geometrie', 'KoudebrugList'],
  Geometrie: ['Hoofdvlak'],
  Hoofdvlak: ['Naam', 'Locatie', 'Orientatie', 'Hellingshoek', 'Oppervlakte', 'DeelvlakList', 'KoudebrugList'],
  DeelvlakList: ['Deelvlak'],
  Deelvlak: ['Type', 'Naam', 'Oppervlakte', 'RcWaarde', 'UWaarde', 'GWaarde'],
  KoudebrugList: ['Koudebrug'],
  Koudebrug: ['Naam', 'PsiWaarde', 'Lengte'],
  Installaties: ['Verwarming', 'Ventilatie', 'Koeling', 'Tapwater'],
  Verwarming: ['Opwekker'],
  'Verwarming/Opwekker': ['Naam', 'Type', 'COP', 'Dekking'],
  Ventilatie: ['Systeem'],
  Systeem: ['Naam', 'Type', 'WTWRendement', 'SFP'],
  Koeling: ['Opwekker'],
  'Koeling/Opwekker': ['Naam', 'Type', 'EER'],
  Tapwater: ['Opwekker'],
  'Tapwater/Opwekker': ['Naam', 'Type', 'Rendement', 'Zonneboiler', 'ZonneboilerFractie'],
  Hernieuwbaar: ['PVSysteem', 'ZonthermischSysteem'],
  PVSysteem: ['Naam', 'Piekvermogen', 'Orientatie', 'Hellingshoek', 'Oppervlakte'],
  ZonthermischSysteem: ['Naam', 'CollectorOppervlakte', 'Type', 'Orientatie', 'Hellingshoek'],
  ConstructieBibliotheek: ['Constructie'],
  Constructie: ['Naam', 'RcWaarde', 'UWaarde', 'Lagen'],
  Lagen: ['Laag'],
  Laag: ['Materiaal', 'Dikte', 'Lambda'],
};

const elementKey = (element: Element) => (element.tagName === 'Opwekker' && element.parentElement
  ? `${element.parentElement.tagName}/Opwekker` : element.tagName);

/**
 * Reports the elements of the file the importer has no field for, one issue
 * per kind of parent element. Only `objectEl` of the Objecten is walked (the
 * others are reported as 'multiple'); a Rekenzone without Geometrie keeps
 * the containers its Hoofdvlakken are read from.
 */
function reportUnreadElements(root: Element, objectEl: Element, issues: ImportIssues): void {
  const unread = new Map<string, Map<string, number>>();
  const walk = (element: Element) => {
    const key = elementKey(element);
    const known = READ_ELEMENTS[key];
    if (!known) return;
    const noGeometry = key === 'Rekenzone' && !getEl(element, 'Geometrie');
    for (const child of Array.from(element.children)) {
      if (child.tagName === 'Object' && child !== objectEl) continue;
      if (known.includes(child.tagName)) {
        walk(child);
        continue;
      }
      if (noGeometry && child.getElementsByTagName('Hoofdvlak').length > 0) continue;
      const names = unread.get(key) ?? new Map<string, number>();
      names.set(child.tagName, (names.get(child.tagName) ?? 0) + 1);
      unread.set(key, names);
    }
  };
  walk(root.tagName === 'Project' ? root : objectEl);
  for (const [key, names] of [...unread.entries()].sort(([a], [b]) => a.localeCompare(b))) {
    unreadData(issues, key, `Elementen onder ${key}`, names);
  }
}

/** The program and file version the Algemeen block names ("Vabi Elements, versie 4.0"). */
function algemeenVersion(doc: Document): string | undefined {
  const algemeen = Array.from(doc.getElementsByTagName('Algemeen')).find((el) => el.parentElement === doc.documentElement);
  if (!algemeen) return undefined;
  const parts = [textOf(algemeen, 'Applicatie'), textOf(algemeen, 'Versie') ? `versie ${textOf(algemeen, 'Versie')}` : undefined]
    .filter((part): part is string => !!part && part.trim() !== '')
    .map((part) => part.trim().slice(0, 80));
  return parts.length > 0 ? parts.join(', ') : undefined;
}

export function readVABIXml(xmlText: string): ReadResult {
  const xmlString = xmlText.charCodeAt(0) === 0xfeff ? xmlText.slice(1) : xmlText;
  // EPA files have no document type; refusing one up front rules out entity
  // expansion and external entities whatever the parser does with them.
  if (/<!DOCTYPE/i.test(xmlString)) {
    throw new ImportFormatError('vabi_doctype', 'Het XML-bestand bevat een DOCTYPE; dat wordt niet ingelezen.');
  }
  const parser = new DOMParser();
  const doc = parser.parseFromString(xmlString, 'application/xml');
  if (doc.querySelector('parsererror')) {
    throw new ImportFormatError('vabi_xml_invalid', 'Ongeldig VABI EPA XML-bestand.');
  }

  const issues = new ImportIssues();

  const objectEls = Array.from(doc.getElementsByTagName('Object'))
    .filter((element) => element.parentElement?.tagName === 'Objecten');
  const objectEl = objectEls[0];
  if (!objectEl) {
    if (doc.getElementsByTagName('gebouw')[0]) return importFromVABILegacy(doc, issues);
    throw new ImportFormatError('vabi_no_object', 'Geen Object-element gevonden in het EPA-bestand.');
  }
  if (objectEls.length > 1) {
    issues.add('multiple', 'Objecten', `Het bestand bevat ${objectEls.length} objecten; alleen het eerste is ingelezen.`, objectEls.length);
  }

  const projectName = getText(objectEl, 'Naam') || 'VABI Import';
  const description = getText(objectEl, 'Omschrijving') || '';
  const address = getText(objectEl, 'Adres') || '';
  const city = getText(objectEl, 'Plaats') || '';
  const funcText = textOf(objectEl, 'Gebruiksfunctie');
  let buildingFunction: BuildingFunction = 'residential';
  if (funcText === undefined) {
    issues.add('defaulted', 'buildingFunction', 'Gebruiksfunctie ontbreekt; residential aangenomen.', 'residential');
  } else {
    const found = lookup(FUNC_FROM_VABI, funcText);
    if (found) buildingFunction = found;
    else issues.add('invalid', 'buildingFunction', `Onbekende gebruiksfunctie "${funcText.slice(0, 40)}"; residential aangenomen.`, funcText.slice(0, 40));
  }

  // Construction library, by name; a repeated name keeps its first entry.
  const library = new Map<string, IConstruction>();
  const constructions: IConstruction[] = [];
  const constBib = doc.getElementsByTagName('ConstructieBibliotheek')[0];
  if (constBib) {
    getChildren(constBib, 'Constructie').forEach((constEl, index) => {
      const cp = `constructions[${index}]`;
      const layers: { material: string; thickness: number; lambda: number }[] = [];
      const lagenEl = getEl(constEl, 'Lagen');
      if (lagenEl) {
        getChildren(lagenEl, 'Laag').forEach((laag, layerIndex) => {
          const lp = `${cp}.layers[${layerIndex}]`;
          layers.push({
            material: getText(laag, 'Materiaal') || 'Materiaal',
            thickness: issues.number(`${lp}.thickness`, textOf(laag, 'Dikte'), { fallback: 0, min: 0 }),
            lambda: issues.number(`${lp}.lambda`, textOf(laag, 'Lambda'), { fallback: 0.03, min: 0, minExclusive: true }),
          });
        });
      }
      const name = getText(constEl, 'Naam') || 'Constructie';
      const rcValue = issues.number(`${cp}.rcValue`, textOf(constEl, 'RcWaarde'), { fallback: 0, min: 0, silentWhenMissing: true });
      const uValue = issues.number(`${cp}.uValue`, textOf(constEl, 'UWaarde'), { fallback: 0, min: 0, silentWhenMissing: true });
      const construction: IConstruction = { id: crypto.randomUUID(), name, layers, rcValue, uValue };
      if (library.has(name)) {
        issues.add('duplicate', cp, `Constructie "${name}" staat meer dan eens in de bibliotheek; de eerste wordt gekoppeld.`, name);
      } else {
        library.set(name, construction);
      }
      // A library entry without U stays out until a Deelvlak gives it one.
      if (uValue > 0) constructions.push(construction);
    });
  }

  /**
   * The construction of an opaque Deelvlak: linked to the library entry of
   * the same name when the values agree, else its own. The U is the file's
   * (Deelvlak, then library); only without one is it derived from Rc with
   * the element type's R_si (NTA 8800 table C.2), and never 0.
   */
  const constructionFor = (dvEl: Element, path: string, surfaceType: SurfaceType): string => {
    const name = getText(dvEl, 'Naam');
    const lib = name ? library.get(name) : undefined;
    const ownRc = issues.number(`${path}.rcValue`, textOf(dvEl, 'RcWaarde'), { fallback: 0, min: 0, silentWhenMissing: true });
    const ownU = issues.number(`${path}.uValue`, textOf(dvEl, 'UWaarde'), { fallback: 0, min: 0, silentWhenMissing: true });
    const rc = ownRc > 0 ? ownRc : lib?.rcValue ?? 0;
    let u = ownU > 0 ? ownU : (lib && lib.uValue > 0 ? lib.uValue : 0);
    if (!(u > 0) && rc > 0) {
      const { rsi, rse } = SURFACE_RESISTANCE[surfaceType];
      u = 1 / (rc + rsi + rse);
      issues.add('defaulted', `${path}.uValue`, `U ontbreekt; berekend uit Rc ${rc} met R_si ${rsi} en R_se ${rse}.`, u);
    }
    if (!(u > 0)) {
      issues.add('dropped', path, `Constructie "${name || '(zonder naam)'}" heeft geen U en geen Rc; niet gekoppeld, vul de constructie aan.`);
      return '';
    }
    if (lib && Math.abs(lib.uValue - u) < 1e-9 && Math.abs(lib.rcValue - rc) < 0.01) {
      if (!constructions.includes(lib)) constructions.push(lib);
      return lib.id;
    }
    const label = name || 'Constructie';
    let existing = constructions.find((c) => c.name === label && Math.abs(c.rcValue - rc) < 0.01 && Math.abs(c.uValue - u) < 1e-9);
    if (!existing) {
      existing = { id: crypto.randomUUID(), name: label, layers: lib?.layers ?? [], rcValue: rc, uValue: u };
      constructions.push(existing);
    }
    return existing.id;
  };

  // Tilts of non-horizontal, non-vertical surfaces; they go to ntaCalculation.surfaceTilts.
  const tilts: Array<{ surfaceId: string; tiltDeg: number }> = [];

  const zones: IZone[] = [];
  const rekenzonesEl = getEl(objectEl, 'Rekenzones');
  const rzEls = rekenzonesEl ? getChildren(rekenzonesEl, 'Rekenzone') : [];

  rzEls.forEach((rzEl, zoneIndex) => {
    const zp = `zones[${zoneIndex}]`;
    const zoneName = getText(rzEl, 'Naam') || 'Zone';
    const floorArea = issues.number(`${zp}.floorArea`, textOf(rzEl, 'Gebruiksoppervlakte'), { fallback: 0, min: 0 });
    const volume = issues.number(`${zp}.volume`, textOf(rzEl, 'Volume'), { fallback: 0, min: 0 });
    const height = issues.number(`${zp}.height`, textOf(rzEl, 'Hoogte'), { fallback: 2.6, min: 0 });
    const before = issues.list.length;
    const qv10 = issues.number(`${zp}.airTightness.qv10`, textOf(rzEl, 'Luchtdoorlatendheid'), {
      fallback: ASSUMED_QV10, min: 0, minExclusive: true, fallbackReason: 'aangenomen waarde, te controleren',
    });
    const qv10Assumed = issues.list.length > before;

    const surfaces: ISurface[] = [];
    const thermalBridges: IThermalBridge[] = [];
    const zoneId = crypto.randomUUID();

    const geoEl = getEl(rzEl, 'Geometrie');
    const hoofdvlakken = geoEl ? getChildren(geoEl, 'Hoofdvlak') : getAllDescendants(rzEl, 'Hoofdvlak');

    hoofdvlakken.forEach((hvEl) => {
      const sp = `${zp}.surfaces[${surfaces.length}]`;
      const surfName = getText(hvEl, 'Naam') || 'Vlak';
      const locatie = codeText(hvEl, 'Locatie');
      const locInfo = locatie === undefined ? undefined : lookup(LOCATIE_MAP, locatie);
      if (!locInfo) {
        issues.add(locatie === undefined && textOf(hvEl, 'Locatie') === undefined ? 'defaulted' : 'invalid', `${sp}.type`,
          `Locatie van hoofdvlak "${surfName}" ontbreekt of is onbekend; gevel aangenomen.`, textOf(hvEl, 'Locatie'));
      }
      const surfaceType: SurfaceType = locInfo?.type ?? 'wall';

      // Tilt: 0 or 180 is horizontal; another value than 90 is kept for NTA 8800 (surfaceTilts).
      const tiltText = textOf(hvEl, 'Hellingshoek');
      let tilt = tiltText === undefined ? undefined : parseDecimalText(tiltText) ?? undefined;
      if (tiltText !== undefined && (tilt === undefined || tilt < 0 || tilt > 180)) {
        issues.add('invalid', `${sp}.tilt`, `Hellingshoek "${tiltText.slice(0, 40)}" van "${surfName}" is ongeldig en genegeerd.`, tiltText.slice(0, 40));
        tilt = undefined;
      }
      const horizontal = (tilt !== undefined && (tilt === 0 || tilt === 180))
        || (tilt === undefined && surfaceType === 'floor');
      let orientation: Orientation = 'horizontal';
      if (!horizontal) {
        orientation = vabiCode(issues, `${sp}.orientation`, hvEl, 'Orientatie', VABI_ORIENT_MAP, 'S' as Orientation, 'Oriëntatie');
      }

      const area = issues.number(`${sp}.area`, textOf(hvEl, 'Oppervlakte'), { fallback: 0, min: 0 });

      // Deelvlakken: windows on the surface, opaque parts as constructions.
      const windows: IWindow[] = [];
      const opaque: Array<{ el: Element; area: number | undefined }> = [];
      const dvListEl = getEl(hvEl, 'DeelvlakList');
      for (const dvEl of dvListEl ? getChildren(dvListEl, 'Deelvlak') : []) {
        const dvType = getText(dvEl, 'Type');
        if (dvType === 'Transparant' || dvType.toLowerCase().includes('transparant')) {
          const wp = `${sp}.windows[${windows.length}]`;
          windows.push({
            id: crypto.randomUUID(),
            name: getText(dvEl, 'Naam') || 'Raam',
            area: issues.number(`${wp}.area`, textOf(dvEl, 'Oppervlakte'), { fallback: 0, min: 0 }),
            uValue: issues.number(`${wp}.uValue`, textOf(dvEl, 'UWaarde'), { fallback: 1.5, min: 0, minExclusive: true }),
            gValue: issues.number(`${wp}.gValue`, textOf(dvEl, 'GWaarde'), { fallback: 0.5, min: 0, max: 1, clamp: true }),
            orientation,
            surfaceId: '', // set below
          });
        } else {
          const text = textOf(dvEl, 'Oppervlakte');
          opaque.push({ el: dvEl, area: text === undefined ? undefined : issues.number(`${sp}.deelvlakken.area`, text, { fallback: 0, min: 0 }) });
        }
      }

      const surfaceOf = (name: string, surfaceArea: number, constructionId: string, surfaceWindows: IWindow[]): ISurface => {
        const surfaceId = crypto.randomUUID();
        for (const w of surfaceWindows) w.surfaceId = surfaceId;
        if (tilt !== undefined && !horizontal && !(tilt === 90 && surfaceType !== 'roof')) {
          tilts.push({ surfaceId, tiltDeg: tilt });
        }
        return { id: surfaceId, name, type: surfaceType, area: surfaceArea, orientation, constructionId, zoneId, windows: surfaceWindows };
      };

      // Several opaque parts (a wall with a door or panel) become separate surfaces, each with
      // its own construction and area; the windows stay on the first.
      const splittable = opaque.length > 1 && opaque.every((part) => part.area !== undefined && part.area > 0);
      if (splittable) {
        const windowArea = windows.reduce((sum, w) => sum + w.area, 0);
        opaque.forEach((part, partIndex) => {
          const partPath = `${zp}.surfaces[${surfaces.length}]`;
          const constructionId = constructionFor(part.el, `${partPath}.construction`, surfaceType);
          const partName = getText(part.el, 'Naam');
          surfaces.push(partIndex === 0
            ? surfaceOf(surfName, part.area! + windowArea, constructionId, windows)
            : surfaceOf(partName ? `${surfName} – ${partName}` : `${surfName} (${partIndex + 1})`, part.area!, constructionId, []));
        });
      } else {
        if (opaque.length > 1) {
          issues.add('dropped', `${sp}.constructionId`,
            `Hoofdvlak "${surfName}" heeft ${opaque.length} dichte deelvlakken zonder oppervlakte; alleen de eerste constructie is gekoppeld.`,
            opaque.length);
        }
        const constructionId = opaque.length > 0 ? constructionFor(opaque[0].el, `${sp}.construction`, surfaceType) : '';
        surfaces.push(surfaceOf(surfName, area, constructionId, windows));
      }

      if (tilt === undefined && surfaceType === 'roof' && orientation !== 'horizontal') {
        issues.add('dropped', `${sp}.tilt`, `Hellingshoek van dak "${surfName}" ontbreekt; vul die in bij de NTA-invoer.`);
      }
    });

    // Thermal bridges: the zone's own list, plus those under each Hoofdvlak. The same list under
    // every Hoofdvlak (written so by earlier Open Energy Studio exports) is read once.
    const bridgeLists: Element[] = [];
    const zoneList = getEl(rzEl, 'KoudebrugList');
    if (zoneList) bridgeLists.push(zoneList);
    const surfaceLists = hoofdvlakken.map((hvEl) => getEl(hvEl, 'KoudebrugList')).filter((el): el is Element => !!el);
    const signatures = surfaceLists.map(bridgeListSignature);
    const repeated = surfaceLists.length > 1 && signatures[0] !== '' && signatures.every((signature) => signature === signatures[0]);
    if (repeated) {
      bridgeLists.push(surfaceLists[0]);
      issues.add('duplicate', `${zp}.thermalBridges`,
        `Dezelfde koudebruglijst staat onder alle ${surfaceLists.length} hoofdvlakken; één keer ingelezen.`, surfaceLists.length);
    } else {
      bridgeLists.push(...surfaceLists);
    }
    for (const listEl of bridgeLists) {
      for (const kbEl of getChildren(listEl, 'Koudebrug')) {
        const bp = `${zp}.thermalBridges[${thermalBridges.length}]`;
        thermalBridges.push({
          id: crypto.randomUUID(),
          name: getText(kbEl, 'Naam') || 'Koudebrug',
          // ψ may be negative (NTA 8800 §8.2.3); a missing ψ is reported, not silently filled.
          psiValue: issues.number(`${bp}.psiValue`, textOf(kbEl, 'PsiWaarde'), { fallback: 0.05 }),
          length: issues.number(`${bp}.length`, textOf(kbEl, 'Lengte'), { fallback: 0, min: 0 }),
          zoneId,
        });
      }
    }

    zones.push({
      id: zoneId,
      name: zoneName,
      floorArea,
      volume,
      height,
      surfaces,
      thermalBridges,
      airTightness: qv10Assumed ? { qv10, assumed: true } : { qv10 },
    });
  });

  // Installations
  const heatingSystems: IHeatingSystem[] = [];
  const ventilationSystems: IVentilationSystem[] = [];
  const coolingSystems: ICoolingSystem[] = [];
  const hotWaterSystems: IHotWaterSystem[] = [];
  const solarPV: ISolarPV[] = [];
  const solarThermal: ISolarThermal[] = [];

  const installEl = getEl(objectEl, 'Installaties');
  if (installEl) {
    const verwEl = getEl(installEl, 'Verwarming');
    for (const opwEl of verwEl ? getChildren(verwEl, 'Opwekker') : []) {
      const hp = `heatingSystems[${heatingSystems.length}]`;
      heatingSystems.push({
        id: crypto.randomUUID(),
        name: getText(opwEl, 'Naam') || 'Verwarming',
        type: vabiCode(issues, `${hp}.type`, opwEl, 'Type', HEATING_FROM_VABI, 'hr107' as HeatingSystemType, 'Verwarmingssoort'),
        cop: issues.number(`${hp}.cop`, textOf(opwEl, 'COP'), { fallback: 1, min: 0, minExclusive: true }),
        coverageFraction: issues.number(`${hp}.coverageFraction`, textOf(opwEl, 'Dekking'), { fallback: 1, min: 0, max: 1, clamp: true }),
      });
    }

    const ventEl = getEl(installEl, 'Ventilatie');
    for (const sysEl of ventEl ? getChildren(ventEl, 'Systeem') : []) {
      const vp = `ventilationSystems[${ventilationSystems.length}]`;
      ventilationSystems.push({
        id: crypto.randomUUID(),
        name: getText(sysEl, 'Naam') || 'Ventilatie',
        type: vabiCode(issues, `${vp}.type`, sysEl, 'Type', VENT_FROM_VABI, 'natural' as VentilationType, 'Ventilatiesoort'),
        heatRecoveryEfficiency: issues.number(`${vp}.heatRecoveryEfficiency`, textOf(sysEl, 'WTWRendement'),
          { fallback: 0, min: 0, max: 1, clamp: true, silentWhenMissing: true }),
        sfp: issues.number(`${vp}.sfp`, textOf(sysEl, 'SFP'), { fallback: 0, min: 0, silentWhenMissing: true }),
      });
    }

    const koelEl = getEl(installEl, 'Koeling');
    for (const opwEl of koelEl ? getChildren(koelEl, 'Opwekker') : []) {
      const cp = `coolingSystems[${coolingSystems.length}]`;
      const type = vabiCode(issues, `${cp}.type`, opwEl, 'Type', COOL_FROM_VABI, 'none' as CoolingSystemType, 'Koelsoort');
      coolingSystems.push({
        id: crypto.randomUUID(),
        name: getText(opwEl, 'Naam') || 'Koeling',
        type,
        // Without cooling the EER is not used, so 0 is kept as it is.
        eer: issues.number(`${cp}.eer`, textOf(opwEl, 'EER'), { fallback: 3, min: 0, minExclusive: type !== 'none' }),
      });
    }

    const tapwEl = getEl(installEl, 'Tapwater');
    for (const opwEl of tapwEl ? getChildren(tapwEl, 'Opwekker') : []) {
      const wp = `hotWaterSystems[${hotWaterSystems.length}]`;
      hotWaterSystems.push({
        id: crypto.randomUUID(),
        name: getText(opwEl, 'Naam') || 'Tapwater',
        type: vabiCode(issues, `${wp}.type`, opwEl, 'Type', HW_FROM_VABI, 'hr_combi' as HotWaterSystemType, 'Tapwatersoort'),
        efficiency: issues.number(`${wp}.efficiency`, textOf(opwEl, 'Rendement'), { fallback: 1, min: 0, minExclusive: true }),
        hasSolarBoiler: getText(opwEl, 'Zonneboiler') === 'true',
        solarBoilerFraction: issues.number(`${wp}.solarBoilerFraction`, textOf(opwEl, 'ZonneboilerFractie'),
          { fallback: 0, min: 0, max: 1, clamp: true, silentWhenMissing: true }),
      });
    }
  }

  const herEl = getEl(objectEl, 'Hernieuwbaar');
  if (herEl) {
    for (const pvEl of getChildren(herEl, 'PVSysteem')) {
      const pp = `solarPV[${solarPV.length}]`;
      solarPV.push({
        id: crypto.randomUUID(),
        name: getText(pvEl, 'Naam') || 'PV',
        peakPower: issues.number(`${pp}.peakPower`, textOf(pvEl, 'Piekvermogen'), { fallback: 0, min: 0 }),
        orientation: vabiCode(issues, `${pp}.orientation`, pvEl, 'Orientatie', VABI_ORIENT_MAP, 'S' as Orientation, 'Oriëntatie'),
        tilt: issues.number(`${pp}.tilt`, textOf(pvEl, 'Hellingshoek'), { fallback: 30, min: 0, max: 90, clamp: true }),
        area: issues.number(`${pp}.area`, textOf(pvEl, 'Oppervlakte'), { fallback: 0, min: 0 }),
      });
    }

    for (const ztEl of getChildren(herEl, 'ZonthermischSysteem')) {
      const tp = `solarThermal[${solarThermal.length}]`;
      solarThermal.push({
        id: crypto.randomUUID(),
        name: getText(ztEl, 'Naam') || 'Zonnecollector',
        collectorArea: issues.number(`${tp}.collectorArea`, textOf(ztEl, 'CollectorOppervlakte'), { fallback: 0, min: 0 }),
        type: vabiCode(issues, `${tp}.type`, ztEl, 'Type', ST_FROM_VABI, 'flat_plate' as SolarThermalType, 'Collectorsoort'),
        orientation: vabiCode(issues, `${tp}.orientation`, ztEl, 'Orientatie', VABI_ORIENT_MAP, 'S' as Orientation, 'Oriëntatie'),
        tilt: issues.number(`${tp}.tilt`, textOf(ztEl, 'Hellingshoek'), { fallback: 45, min: 0, max: 90, clamp: true }),
      });
    }
  }

  const project: IProject = {
    id: crypto.randomUUID(),
    name: projectName,
    description,
    buildingFunction,
    address,
    city,
    zones,
    heatingSystems,
    ventilationSystems,
    coolingSystems,
    hotWaterSystems,
    solarPV,
    solarThermal,
    constructions,
  };

  // Tilts from the file start the NTA input (NTA 8800 surface tilts); the rest of
  // that input still has to be entered.
  if (tilts.length > 0) {
    const block = buildNtaCalculationTemplate(project) as Record<string, unknown> & {
      surfaceTilts: Array<{ surfaceId: string; tiltDeg: number | null; sourceReference: string }>;
    };
    for (const { surfaceId, tiltDeg } of tilts) {
      const entry = { surfaceId, tiltDeg, sourceReference: `VABI-bestand, Hellingshoek ${tiltDeg}°` };
      const index = block.surfaceTilts.findIndex((item) => item.surfaceId === surfaceId);
      if (index >= 0) block.surfaceTilts[index] = entry;
      else block.surfaceTilts.push(entry);
    }
    project.ntaCalculation = block as unknown as IProject['ntaCalculation'];
  }

  reportUnreadElements(doc.documentElement, objectEl, issues);
  const sourceVersion = algemeenVersion(doc);
  sourceVersionNotCarried(issues, 'VABI', sourceVersion);
  ntaInputNotInFormat(issues, 'VABI');
  return { project, issues, ...(sourceVersion ? { sourceVersion } : {}) };
}

// Legacy import for the old custom XML format (backwards compatibility)
function importFromVABILegacy(doc: Document, issues: ImportIssues): ReadResult {
  const gebouw = doc.getElementsByTagName('gebouw')[0];
  if (!gebouw) throw new ImportFormatError('vabi_no_object', 'Geen gebouw-element gevonden.');

  const constructions: IConstruction[] = [];
  const zones: IZone[] = [];

  const constEls = gebouw.getElementsByTagName('constructie');
  for (let i = 0; i < constEls.length; i++) {
    const el = constEls[i];
    constructions.push({
      id: el.getAttribute('id') || `con-${i}`,
      name: getText(el, 'naam') || `Constructie ${i + 1}`,
      layers: [],
      rcValue: issues.number(`constructions[${i}].rcValue`, textOf(el, 'rc-waarde'), { fallback: 0, min: 0, silentWhenMissing: true }),
      uValue: issues.number(`constructions[${i}].uValue`, textOf(el, 'u-waarde'), { fallback: 0, min: 0, silentWhenMissing: true }),
    });
  }

  const zoneEls = gebouw.getElementsByTagName('zone');
  for (let i = 0; i < zoneEls.length; i++) {
    const zEl = zoneEls[i];
    const zp = `zones[${i}]`;
    const zoneId = zEl.getAttribute('id') || `zone-${i}`;
    const surfaces: ISurface[] = [];
    const thermalBridges: IThermalBridge[] = [];

    const oppEls = zEl.getElementsByTagName('oppervlak');
    for (let j = 0; j < oppEls.length; j++) {
      const oEl = oppEls[j];
      const sp = `${zp}.surfaces[${j}]`;
      const surfId = oEl.getAttribute('id') || `surf-${i}-${j}`;
      const windows: IWindow[] = [];

      const raamEls = oEl.getElementsByTagName('raam');
      for (let k = 0; k < raamEls.length; k++) {
        const rEl = raamEls[k];
        const wp = `${sp}.windows[${k}]`;
        windows.push({
          id: rEl.getAttribute('id') || `win-${i}-${j}-${k}`,
          name: getText(rEl, 'naam') || 'Raam',
          area: issues.number(`${wp}.area`, textOf(rEl, 'oppervlakte'), { fallback: 0, min: 0 }),
          uValue: issues.number(`${wp}.uValue`, textOf(rEl, 'u-waarde'), { fallback: 1.5, min: 0, minExclusive: true }),
          gValue: issues.number(`${wp}.gValue`, textOf(rEl, 'g-waarde'), { fallback: 0.5, min: 0, max: 1, clamp: true }),
          orientation: 'S',
          surfaceId: surfId,
        });
      }

      // The legacy format has no element type, orientation or (always) construction.
      issues.add('defaulted', `${sp}.type`, 'Oud VABI-formaat zonder vlaksoort en oriëntatie; gevel op het zuiden aangenomen.', 'wall/S');
      const ownConstruction = textOf(oEl, 'constructie-id');
      if (!ownConstruction && constructions[0]) {
        issues.add('defaulted', `${sp}.constructionId`, `Geen constructie bij dit vlak; de eerste constructie "${constructions[0].name}" aangenomen.`);
      }
      surfaces.push({
        id: surfId,
        name: getText(oEl, 'naam') || 'Oppervlak',
        type: 'wall',
        area: issues.number(`${sp}.area`, textOf(oEl, 'oppervlakte'), { fallback: 0, min: 0 }),
        orientation: 'S',
        constructionId: ownConstruction || (constructions[0]?.id ?? ''),
        zoneId,
        windows,
      });
    }

    const tbEls = zEl.getElementsByTagName('koudebrug');
    for (let j = 0; j < tbEls.length; j++) {
      const bp = `${zp}.thermalBridges[${j}]`;
      thermalBridges.push({
        id: tbEls[j].getAttribute('id') || `tb-${i}-${j}`,
        name: getText(tbEls[j], 'naam') || 'Koudebrug',
        psiValue: issues.number(`${bp}.psiValue`, textOf(tbEls[j], 'psi-waarde'), { fallback: 0.05 }),
        length: issues.number(`${bp}.length`, textOf(tbEls[j], 'lengte'), { fallback: 0, min: 0 }),
        zoneId,
      });
    }

    const before = issues.list.length;
    const qv10 = issues.number(`${zp}.airTightness.qv10`, textOf(zEl, 'qv10'), {
      fallback: 1.0, min: 0, minExclusive: true, fallbackReason: 'aangenomen waarde, te controleren',
    });
    zones.push({
      id: zoneId,
      name: getText(zEl, 'naam') || `Zone ${i + 1}`,
      floorArea: issues.number(`${zp}.floorArea`, textOf(zEl, 'vloeroppervlak'), { fallback: 0, min: 0 }),
      volume: issues.number(`${zp}.volume`, textOf(zEl, 'volume'), { fallback: 0, min: 0 }),
      height: issues.number(`${zp}.height`, textOf(zEl, 'hoogte'), { fallback: 2.6, min: 0 }),
      surfaces,
      thermalBridges,
      airTightness: issues.list.length > before ? { qv10, assumed: true } : { qv10 },
    });
  }

  issues.add('defaulted', 'buildingFunction', 'Oud VABI-formaat zonder gebruiksfunctie; residential aangenomen.', 'residential');
  sourceVersionNotCarried(issues, 'VABI', undefined);
  ntaInputNotInFormat(issues, 'VABI');
  return {
    project: {
      id: crypto.randomUUID(),
      name: getText(gebouw, 'naam') || 'VABI Import',
      description: getText(gebouw, 'omschrijving'),
      buildingFunction: 'residential',
      address: getText(gebouw, 'adres'),
      city: getText(gebouw, 'plaats'),
      zones,
      heatingSystems: [],
      ventilationSystems: [],
      coolingSystems: [],
      hotWaterSystems: [],
      solarPV: [],
      solarThermal: [],
      constructions,
    },
    issues,
  };
}

// ============================================================
// Download helper (export as .epa ZIP)
// ============================================================

/**
 * Saves the EPA archive and returns what the format left out (empty when
 * nothing was lost), so the adviser can be told.
 */
export function downloadVABI(project: IProject): string[] {
  downloadBytes(exportToVABI(project), `${project.name || 'project'}.epa`);
  return vabiExportLosses(project);
}

// ============================================================
// File dialog helper (import .epa ZIP or .xml)
// ============================================================

/** Picks and imports a VABI EPA archive or project.xml, with its name, SHA-256 and import issues. */
export async function openVABIFileDialog(): Promise<ImportResult> {
  return importFile(await pickFile('.epa,.xml,.zip'), readVABIFile);
}
