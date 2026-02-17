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

import { zipSync, unzipSync, strToU8, strFromU8 } from 'fflate';
import type {
  IProject, IZone, ISurface, IWindow, IThermalBridge, IConstruction,
  IHeatingSystem, IVentilationSystem, ICoolingSystem, IHotWaterSystem,
  ISolarPV, ISolarThermal,
  Orientation, SurfaceType, HeatingSystemType, VentilationType,
  CoolingSystemType, HotWaterSystemType, SolarThermalType, BuildingFunction,
} from '../energy/types';

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

// Orientatie: VABI code → compass direction
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

function getNum(parent: Element, tagName: string, fallback = 0): number {
  const text = getText(parent, tagName).replace(',', '.');
  const val = parseFloat(text);
  return isNaN(val) ? fallback : val;
}

function getInt(parent: Element, tagName: string, fallback = 0): number {
  const text = getText(parent, tagName);
  const val = parseInt(text, 10);
  return isNaN(val) ? fallback : val;
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
  lines.push(`${L(3)}${xmlTag('Gebruiksfunctie', FUNC_TO_VABI[project.buildingFunction] || 'Woonfunctie')}`);

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

      lines.push(`${L(6)}${xmlOpen('Hoofdvlak')}`);
      lines.push(`${L(7)}${xmlTag('Naam', surface.name)}`);
      lines.push(`${L(7)}${xmlTag('Locatie', SURFACE_TO_LOCATIE[surface.type] ?? 2)}`);
      lines.push(`${L(7)}${xmlTag('Orientatie', ORIENT_TO_VABI[surface.orientation] ?? 4)}`);
      lines.push(`${L(7)}${xmlTag('Hellingshoek', surface.type === 'roof' ? 0 : surface.type === 'floor' ? 0 : 90)}`);
      lines.push(`${L(7)}${xmlTag('Oppervlakte', surface.area)}`);

      // Opaque sub-surfaces (DeelvlakList)
      lines.push(`${L(7)}${xmlOpen('DeelvlakList')}`);

      // Main opaque part (subtract windows)
      const windowArea = surface.windows.reduce((sum, w) => sum + w.area, 0);
      const opaqueArea = Math.max(0, surface.area - windowArea);

      if (opaqueArea > 0 && construction) {
        lines.push(`${L(8)}${xmlOpen('Deelvlak')}`);
        lines.push(`${L(9)}${xmlTag('Type', 'Dicht')}`);
        lines.push(`${L(9)}${xmlTag('Naam', construction.name)}`);
        lines.push(`${L(9)}${xmlTag('Oppervlakte', opaqueArea)}`);
        lines.push(`${L(9)}${xmlTag('RcWaarde', construction.rcValue)}`);
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

      // Thermal bridges for this surface
      lines.push(`${L(7)}${xmlOpen('KoudebrugList')}`);
      for (const tb of zone.thermalBridges) {
        lines.push(`${L(8)}${xmlOpen('Koudebrug')}`);
        lines.push(`${L(9)}${xmlTag('Naam', tb.name)}`);
        lines.push(`${L(9)}${xmlTag('PsiWaarde', tb.psiValue)}`);
        lines.push(`${L(9)}${xmlTag('Lengte', tb.length)}`);
        lines.push(`${L(8)}${xmlClose('Koudebrug')}`);
      }
      lines.push(`${L(7)}${xmlClose('KoudebrugList')}`);

      lines.push(`${L(6)}${xmlClose('Hoofdvlak')}`);
    }
    lines.push(`${L(5)}${xmlClose('Geometrie')}`);

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
      lines.push(`${L(6)}${xmlTag('Type', HEATING_TO_VABI[hs.type] ?? 1)}`);
      lines.push(`${L(6)}${xmlTag('COP', hs.cop)}`);
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
      lines.push(`${L(6)}${xmlTag('Type', VENT_TO_VABI[vs.type] ?? 0)}`);
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
      lines.push(`${L(6)}${xmlTag('Type', COOL_TO_VABI[cs.type] ?? 0)}`);
      lines.push(`${L(6)}${xmlTag('EER', cs.eer)}`);
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
      lines.push(`${L(6)}${xmlTag('Type', HW_TO_VABI[hw.type] ?? 0)}`);
      lines.push(`${L(6)}${xmlTag('Rendement', hw.efficiency)}`);
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
      lines.push(`${L(5)}${xmlTag('Orientatie', ORIENT_TO_VABI[pv.orientation] ?? 4)}`);
      lines.push(`${L(5)}${xmlTag('Hellingshoek', pv.tilt)}`);
      lines.push(`${L(5)}${xmlTag('Oppervlakte', pv.area)}`);
      lines.push(`${L(4)}${xmlClose('PVSysteem')}`);
    }

    for (const st of project.solarThermal) {
      lines.push(`${L(4)}${xmlOpen('ZonthermischSysteem')}`);
      lines.push(`${L(5)}${xmlTag('Naam', st.name)}`);
      lines.push(`${L(5)}${xmlTag('CollectorOppervlakte', st.collectorArea)}`);
      lines.push(`${L(5)}${xmlTag('Type', ST_TO_VABI[st.type] ?? 0)}`);
      lines.push(`${L(5)}${xmlTag('Orientatie', ORIENT_TO_VABI[st.orientation] ?? 4)}`);
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

  // Create ZIP with project.xml inside
  const files: Record<string, Uint8Array> = {
    'project.xml': strToU8(xml),
  };

  return zipSync(files, { level: 6 });
}

// ============================================================
// IMPORT: VABI EPA ZIP → IProject
// ============================================================

export function importFromVABI(zipData: Uint8Array): IProject {
  const files = unzipSync(zipData);

  // Find project.xml in the ZIP
  let xmlData: Uint8Array | undefined;
  for (const [path, data] of Object.entries(files)) {
    if (path.toLowerCase().endsWith('project.xml') || path.toLowerCase().endsWith('.xml')) {
      xmlData = data;
      break;
    }
  }

  if (!xmlData) {
    throw new Error('Geen project.xml gevonden in EPA bestand');
  }

  const xmlString = strFromU8(xmlData);
  return importFromVABIXml(xmlString);
}

function importFromVABIXml(xmlString: string): IProject {
  const parser = new DOMParser();
  const doc = parser.parseFromString(xmlString, 'application/xml');

  const parserError = doc.querySelector('parsererror');
  if (parserError) {
    throw new Error('Ongeldig VABI EPA XML bestand');
  }

  // Find first Object element
  const objectEls = doc.getElementsByTagName('Object');
  let objectEl: Element | null = null;
  for (let i = 0; i < objectEls.length; i++) {
    if (objectEls[i].parentElement?.tagName === 'Objecten') {
      objectEl = objectEls[i];
      break;
    }
  }

  if (!objectEl) {
    // Try legacy format with <gebouw>
    const gebouw = doc.getElementsByTagName('gebouw')[0];
    if (gebouw) {
      return importFromVABILegacy(doc);
    }
    throw new Error('Geen Object element gevonden in EPA bestand');
  }

  const projectName = getText(objectEl, 'Naam') || 'VABI Import';
  const description = getText(objectEl, 'Omschrijving') || '';
  const address = getText(objectEl, 'Adres') || '';
  const city = getText(objectEl, 'Plaats') || '';
  const funcText = getText(objectEl, 'Gebruiksfunctie') || '';
  const buildingFunction = FUNC_FROM_VABI[funcText] || 'residential';

  // Parse construction library
  const constructions: IConstruction[] = [];
  const constBib = doc.getElementsByTagName('ConstructieBibliotheek')[0];
  if (constBib) {
    const constEls = getChildren(constBib, 'Constructie');
    for (const constEl of constEls) {
      const layers: { material: string; thickness: number; lambda: number }[] = [];
      const lagenEl = getEl(constEl, 'Lagen');
      if (lagenEl) {
        for (const laag of getChildren(lagenEl, 'Laag')) {
          layers.push({
            material: getText(laag, 'Materiaal') || 'Materiaal',
            thickness: getNum(laag, 'Dikte'),
            lambda: getNum(laag, 'Lambda', 0.03),
          });
        }
      }
      constructions.push({
        id: crypto.randomUUID(),
        name: getText(constEl, 'Naam') || 'Constructie',
        layers,
        rcValue: getNum(constEl, 'RcWaarde'),
        uValue: getNum(constEl, 'UWaarde'),
      });
    }
  }

  // Parse zones (Rekenzones)
  const zones: IZone[] = [];
  const rekenzonesEl = getEl(objectEl, 'Rekenzones');
  const rzEls = rekenzonesEl ? getChildren(rekenzonesEl, 'Rekenzone') : [];

  for (const rzEl of rzEls) {
    const zoneName = getText(rzEl, 'Naam') || 'Zone';
    const floorArea = getNum(rzEl, 'Gebruiksoppervlakte');
    const volume = getNum(rzEl, 'Volume');
    const height = getNum(rzEl, 'Hoogte', 2.6);
    const qv10 = getNum(rzEl, 'Luchtdoorlatendheid', 0.4);

    const surfaces: ISurface[] = [];
    const thermalBridges: IThermalBridge[] = [];
    const zoneId = crypto.randomUUID();

    // Parse Geometrie > Hoofdvlak
    const geoEl = getEl(rzEl, 'Geometrie');
    const hoofdvlakken = geoEl ? getChildren(geoEl, 'Hoofdvlak') : getAllDescendants(rzEl, 'Hoofdvlak');

    for (const hvEl of hoofdvlakken) {
      const surfName = getText(hvEl, 'Naam') || 'Vlak';
      const locatie = getInt(hvEl, 'Locatie', 2);
      const orientCode = getInt(hvEl, 'Orientatie', 4);
      const area = getNum(hvEl, 'Oppervlakte');

      const locInfo = LOCATIE_MAP[locatie] || { type: 'wall' as SurfaceType };
      const surfaceType = locInfo.type;
      const orientation = VABI_ORIENT_MAP[orientCode] || 'S';

      // Parse Deelvlakken
      const windows: IWindow[] = [];
      let constructionId = '';
      const dvListEl = getEl(hvEl, 'DeelvlakList');
      const deelvlakken = dvListEl ? getChildren(dvListEl, 'Deelvlak') : [];

      for (const dvEl of deelvlakken) {
        const dvType = getText(dvEl, 'Type');
        const dvName = getText(dvEl, 'Naam') || 'Element';

        if (dvType === 'Transparant' || dvType.toLowerCase().includes('transparant')) {
          // Window/glass
          windows.push({
            id: crypto.randomUUID(),
            name: dvName,
            area: getNum(dvEl, 'Oppervlakte'),
            uValue: getNum(dvEl, 'UWaarde', 1.5),
            gValue: getNum(dvEl, 'GWaarde', 0.5),
            orientation,
            surfaceId: '', // set below
          });
        } else {
          // Opaque construction
          const rc = getNum(dvEl, 'RcWaarde');
          if (rc > 0 || dvName) {
            // Find or create construction
            let existing = constructions.find(c => c.name === dvName && Math.abs(c.rcValue - rc) < 0.01);
            if (!existing && dvName) {
              existing = {
                id: crypto.randomUUID(),
                name: dvName,
                layers: [],
                rcValue: rc,
                uValue: rc > 0 ? 1 / (rc + 0.17 + 0.04) : 0,
              };
              constructions.push(existing);
            }
            if (existing) constructionId = existing.id;
          }
        }
      }

      const surfaceId = crypto.randomUUID();
      for (const w of windows) w.surfaceId = surfaceId;

      surfaces.push({
        id: surfaceId,
        name: surfName,
        type: surfaceType,
        area,
        orientation,
        constructionId,
        zoneId,
        windows,
      });

      // Parse thermal bridges (KoudebrugList)
      const kbListEl = getEl(hvEl, 'KoudebrugList');
      const kbEls = kbListEl ? getChildren(kbListEl, 'Koudebrug') : [];
      for (const kbEl of kbEls) {
        thermalBridges.push({
          id: crypto.randomUUID(),
          name: getText(kbEl, 'Naam') || 'Koudebrug',
          psiValue: getNum(kbEl, 'PsiWaarde', 0.05),
          length: getNum(kbEl, 'Lengte'),
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
      airTightness: { qv10 },
    });
  }

  // Parse installations
  const heatingSystems: IHeatingSystem[] = [];
  const ventilationSystems: IVentilationSystem[] = [];
  const coolingSystems: ICoolingSystem[] = [];
  const hotWaterSystems: IHotWaterSystem[] = [];
  const solarPV: ISolarPV[] = [];
  const solarThermal: ISolarThermal[] = [];

  const installEl = getEl(objectEl, 'Installaties');
  if (installEl) {
    // Heating
    const verwEl = getEl(installEl, 'Verwarming');
    if (verwEl) {
      for (const opwEl of getChildren(verwEl, 'Opwekker')) {
        heatingSystems.push({
          id: crypto.randomUUID(),
          name: getText(opwEl, 'Naam') || 'Verwarming',
          type: HEATING_FROM_VABI[getInt(opwEl, 'Type', 1)] || 'hr_combi',
          cop: getNum(opwEl, 'COP', 1),
          coverageFraction: getNum(opwEl, 'Dekking', 1),
        });
      }
    }

    // Ventilation
    const ventEl = getEl(installEl, 'Ventilatie');
    if (ventEl) {
      for (const sysEl of getChildren(ventEl, 'Systeem')) {
        ventilationSystems.push({
          id: crypto.randomUUID(),
          name: getText(sysEl, 'Naam') || 'Ventilatie',
          type: VENT_FROM_VABI[getInt(sysEl, 'Type', 0)] || 'natural',
          heatRecoveryEfficiency: getNum(sysEl, 'WTWRendement'),
          sfp: getNum(sysEl, 'SFP'),
        });
      }
    }

    // Cooling
    const koelEl = getEl(installEl, 'Koeling');
    if (koelEl) {
      for (const opwEl of getChildren(koelEl, 'Opwekker')) {
        coolingSystems.push({
          id: crypto.randomUUID(),
          name: getText(opwEl, 'Naam') || 'Koeling',
          type: COOL_FROM_VABI[getInt(opwEl, 'Type', 0)] || 'none',
          eer: getNum(opwEl, 'EER', 3),
        });
      }
    }

    // Hot water
    const tapwEl = getEl(installEl, 'Tapwater');
    if (tapwEl) {
      for (const opwEl of getChildren(tapwEl, 'Opwekker')) {
        hotWaterSystems.push({
          id: crypto.randomUUID(),
          name: getText(opwEl, 'Naam') || 'Tapwater',
          type: HW_FROM_VABI[getInt(opwEl, 'Type', 0)] || 'hr_combi',
          efficiency: getNum(opwEl, 'Rendement', 1),
          hasSolarBoiler: getText(opwEl, 'Zonneboiler') === 'true',
          solarBoilerFraction: getNum(opwEl, 'ZonneboilerFractie'),
        });
      }
    }
  }

  // Renewables
  const herEl = getEl(objectEl, 'Hernieuwbaar');
  if (herEl) {
    for (const pvEl of getChildren(herEl, 'PVSysteem')) {
      solarPV.push({
        id: crypto.randomUUID(),
        name: getText(pvEl, 'Naam') || 'PV',
        peakPower: getNum(pvEl, 'Piekvermogen'),
        orientation: VABI_ORIENT_MAP[getInt(pvEl, 'Orientatie', 4)] || 'S',
        tilt: getNum(pvEl, 'Hellingshoek', 30),
        area: getNum(pvEl, 'Oppervlakte'),
      });
    }

    for (const ztEl of getChildren(herEl, 'ZonthermischSysteem')) {
      solarThermal.push({
        id: crypto.randomUUID(),
        name: getText(ztEl, 'Naam') || 'Zonnecollector',
        collectorArea: getNum(ztEl, 'CollectorOppervlakte'),
        type: ST_FROM_VABI[getInt(ztEl, 'Type', 0)] || 'flat_plate',
        orientation: VABI_ORIENT_MAP[getInt(ztEl, 'Orientatie', 4)] || 'S',
        tilt: getNum(ztEl, 'Hellingshoek', 45),
      });
    }
  }

  return {
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
}

// Legacy import for the old custom XML format (backwards compatibility)
function importFromVABILegacy(doc: Document): IProject {
  const gebouw = doc.getElementsByTagName('gebouw')[0];
  if (!gebouw) throw new Error('Geen gebouw element');

  const constructions: IConstruction[] = [];
  const zones: IZone[] = [];

  // Parse constructions
  const constEls = gebouw.getElementsByTagName('constructie');
  for (let i = 0; i < constEls.length; i++) {
    const el = constEls[i];
    constructions.push({
      id: el.getAttribute('id') || `con-${i}`,
      name: getText(el, 'naam') || `Constructie ${i + 1}`,
      layers: [],
      rcValue: getNum(el, 'rc-waarde'),
      uValue: getNum(el, 'u-waarde'),
    });
  }

  // Parse zones
  const zoneEls = gebouw.getElementsByTagName('zone');
  for (let i = 0; i < zoneEls.length; i++) {
    const zEl = zoneEls[i];
    const zoneId = zEl.getAttribute('id') || `zone-${i}`;

    const surfaces: ISurface[] = [];
    const thermalBridges: IThermalBridge[] = [];

    const oppEls = zEl.getElementsByTagName('oppervlak');
    for (let j = 0; j < oppEls.length; j++) {
      const oEl = oppEls[j];
      const surfId = oEl.getAttribute('id') || `surf-${i}-${j}`;
      const windows: IWindow[] = [];

      const raamEls = oEl.getElementsByTagName('raam');
      for (let k = 0; k < raamEls.length; k++) {
        const rEl = raamEls[k];
        windows.push({
          id: rEl.getAttribute('id') || `win-${i}-${j}-${k}`,
          name: getText(rEl, 'naam') || 'Raam',
          area: getNum(rEl, 'oppervlakte'),
          uValue: getNum(rEl, 'u-waarde', 1.5),
          gValue: getNum(rEl, 'g-waarde', 0.5),
          orientation: 'S',
          surfaceId: surfId,
        });
      }

      surfaces.push({
        id: surfId,
        name: getText(oEl, 'naam') || 'Oppervlak',
        type: 'wall',
        area: getNum(oEl, 'oppervlakte'),
        orientation: 'S',
        constructionId: getText(oEl, 'constructie-id') || (constructions[0]?.id ?? ''),
        zoneId,
        windows,
      });
    }

    const tbEls = zEl.getElementsByTagName('koudebrug');
    for (let j = 0; j < tbEls.length; j++) {
      thermalBridges.push({
        id: tbEls[j].getAttribute('id') || `tb-${i}-${j}`,
        name: getText(tbEls[j], 'naam') || 'Koudebrug',
        psiValue: getNum(tbEls[j], 'psi-waarde', 0.05),
        length: getNum(tbEls[j], 'lengte'),
        zoneId,
      });
    }

    zones.push({
      id: zoneId,
      name: getText(zEl, 'naam') || `Zone ${i + 1}`,
      floorArea: getNum(zEl, 'vloeroppervlak'),
      volume: getNum(zEl, 'volume'),
      height: getNum(zEl, 'hoogte', 2.6),
      surfaces,
      thermalBridges,
      airTightness: { qv10: getNum(zEl, 'qv10', 1.0) },
    });
  }

  return {
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
  };
}

// ============================================================
// Download helper (export as .epa ZIP)
// ============================================================

export function downloadVABI(project: IProject): void {
  const zipData = exportToVABI(project);
  const blob = new Blob([zipData.buffer as ArrayBuffer], { type: 'application/zip' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${project.name || 'project'}.epa`;
  a.click();
  URL.revokeObjectURL(url);
}

// ============================================================
// File dialog helper (import .epa ZIP or .xml)
// ============================================================

export function openVABIFileDialog(): Promise<IProject> {
  return new Promise((resolve, reject) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.epa,.xml,.zip';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) { reject(new Error('Geen bestand geselecteerd')); return; }
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const data = new Uint8Array(reader.result as ArrayBuffer);

          // Check if it's a ZIP file (starts with PK)
          if (data[0] === 0x50 && data[1] === 0x4B) {
            resolve(importFromVABI(data));
          } else {
            // Assume raw XML
            const xmlString = new TextDecoder('utf-8').decode(data);
            resolve(importFromVABIXml(xmlString));
          }
        } catch (err) {
          reject(err);
        }
      };
      reader.onerror = () => reject(new Error('Fout bij lezen bestand'));
      reader.readAsArrayBuffer(file);
    };
    input.click();
  });
}
