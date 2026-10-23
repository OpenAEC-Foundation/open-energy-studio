/**
 * Export of the building geometry to the public Uniec 3 Excel import
 * (issue #14): a workbook with the sheets "algemene gegevens" and
 * "geometrische gegevens" after the specification of 18-09-2026
 * (import-beschrijving-2026-09-18.xlsx, SHA-256 312f5028…0cd) and the
 * dwelling example of 28-05-2026 (SHA-256 1c21e68b…a71). The mapping, what
 * is derived and what is left out is in docs/nta8800-uniec-geometrie-export.md.
 *
 * Scope of this first adapter: one ground-bound dwelling ("grondgebonden
 * woning") with its calculation zones, opaque parts, windows and doors and
 * floors directly on the ground. Data the contract requires and the project
 * cannot give refuses the export with the reason (`missing`); data the
 * contract cannot hold is listed (`lost`); values that are derived or
 * rounded to the contract's rules are listed too (`derived`).
 *
 * Status: concept. The file has not been imported into Uniec; the counts,
 * areas, orientations and construction values have to be checked there.
 */
import type { IConstruction, IProject, ISurface, IWindow, IWindowType, Orientation, ThermalBoundary } from '../energy/types';
import type { NtaMassClass, NtaMovableShading, NtaObstruction, NtaShadingControl, NtaShadingDevice } from '../nta/KernelClient';
import { writeXlsx, type XlsxCell } from './xlsxWriter';
import { downloadBytes } from './importFile';

/** The contract this adapter follows. */
export const UNIEC_GEOMETRY_CONTRACT = {
  specification: 'import-beschrijving-2026-09-18.xlsx',
  specificationSha256: '312f5028cda667003609dd1eaeae4ee396e982378a8f476826999b1f247360cd',
  example: '2026-05-28-voorbeeld-Excel-import-bestand-woning.xlsx',
  exampleSha256: '1c21e68becfb2b354b1e092c9798ba4e7e485d976e5c99067c8d2de4c7e3ca71',
  status: 'concept: niet in Uniec geïmporteerd',
} as const;

export const GENERAL_SHEET = 'algemene gegevens';
export const GEOMETRY_SHEET = 'geometrische gegevens';

/** Column names of "algemene gegevens", in the order of the specification. */
export const GENERAL_COLUMNS = [
  'Omschrijving', 'Plaats', 'TypeGebouw', 'SoortBouw', 'Bouwjaar', 'Renovatiejaar', 'Eigendom', 'Opname', 'NaamExterneTool',
] as const;

/** Column names of "geometrische gegevens", in the order of the specification (18-09-2026). */
export const GEOMETRY_COLUMNS = [
  'WoningOmschrijving', 'Positie', 'AantalWoonfunctie', 'AantalUnits', 'TypeZone', 'NaamZone', 'BouwwijzeVloeren',
  'BouwwijzeWanden', 'BouwwijzePlafonds', 'TypePlafond', 'Dm;int;eff', 'Cm;int;eff', 'nbouwlaag', 'nbouwlaagUnit', 'Ag',
  'NaamConstructie', 'Vlak', 'Begrenzing', 'AOR_AOS', 'Vloer', 'Orientatie', 'Maaiveld', 'Helling', 'NaamBibliotheek',
  'TypeConstructie', 'TypeTransparant', 'Methodiek', 'TypeKozijn', 'PositieLineair', 'OmschrijvingBibliotheek',
  'RietDikteIsolatie', 'RietDikteRietpakket', 'Rc', 'Uw/Ud', 'ggl;n', 'A', 'psi', 'Χ', 'AantalTransparanteConstructies',
  'Oppervlakte', 'Beschaduwing', 'BelemmeringAfstand', 'BelemmeringHoogte', 'OverstekAfstand', 'OverstekHoogte',
  'LinksAfstand', 'LinksBreedte', 'LinksHoogte', 'RechtsAfstanda', 'RechtsBreedte', 'RechtsHoogte', 'Zonwering',
  'GglAlt', 'GglDif', 'Regeling', 'VentilatieveKoeling', 'InvoerVoorziening', 'HoogteMiddenOpeningTotMaaiveld',
  'HoogteDoorlaatOpening', 'BrutoDoorlaatVoorziening', 'DischargeCoefficient', 'EntryLossCoefficient', 'DoorlaatFactor',
  'OpeningshoekVoorziening', 'Lengte', 'Aantal', 'Perimeter', 'Kruipruimteventilatie', 'AfstandMaaiveld', 'Rbw',
  'RbfForf', 'RbfProd', 'RbfProdWaarde', 'Opmerking',
] as const;

type GeneralColumn = typeof GENERAL_COLUMNS[number];
type GeometryColumn = typeof GEOMETRY_COLUMNS[number];
export type GeneralRow = Partial<Record<GeneralColumn, string | number>>;
export type GeometryRow = Partial<Record<GeometryColumn, string | number>>;

export interface UniecExportPoint {
  /** `missing`: the contract needs it and the project cannot give it (export refused); `lost`: not carried over; `derived`: derived or rounded. */
  kind: 'missing' | 'lost' | 'derived';
  /** Where in the project (zone, surface, window) or the contract column. */
  where: string;
  message: string;
}

export interface UniecGeometryExport {
  /** False when a `missing` point refuses the export. */
  ok: boolean;
  general: GeneralRow;
  geometry: GeometryRow[];
  points: UniecExportPoint[];
}

// ── Rounding rules of the specification (sheet "toelichting kolom F") ──

const halfUp = (value: number, decimals: number) => {
  const factor = 10 ** decimals;
  return Math.round((value + Math.sign(value) * 1e-9) * factor) / factor;
};
/** R014: half up to 2 decimals. */
const r014 = (value: number) => halfUp(value, 2);
/** R016: half up to 0 decimals. */
const r016 = (value: number) => halfUp(value, 0);
/** R012: above 1 half up to 1 decimal, else half up to 2 decimals. */
const r012 = (value: number) => (value > 1 ? halfUp(value, 1) : halfUp(value, 2));
/** R011: down to a multiple of 0,05. */
const r011 = (value: number) => halfUp(Math.floor(value / 0.05 + 1e-9) * 0.05, 2);

/** Texts may hold letters, digits, space and - _ . , only, at most 32 characters (F002). */
export function contractText(text: string, fallback: string): string {
  const cleaned = text.normalize('NFD').replace(/[̀-ͯ]/g, '')
    .replace(/[^-_.,A-Za-z0-9 ]+/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 32).trim();
  return cleaned || fallback;
}

const ORIENTATIONS: Record<Orientation, string> = {
  N: 'N', NE: 'NO', E: 'O', SE: 'ZO', S: 'Z', SW: 'ZW', W: 'W', NW: 'NW', horizontal: 'HOR',
};

/** Table 7.10 class to a construction choice of the contract that has that class. */
const FLOOR_MASS: Record<NtaMassClass, string> = {
  light: 'hsb, sfb, schuimbeton of hout (licht)',
  heavy: 'staal-beton, hout-beton of niet-massief beton (zwaar)',
  very_heavy: 'massief beton (zeer zwaar)',
};
const WALL_MASS: Record<NtaMassClass, string> = {
  light: 'hsb, sfb of staalskeletbouw (licht)',
  heavy: 'dragend metselwerk (zwaar)',
  very_heavy: 'betonnen wand-vloer skeletbouw (zeer zwaar)',
};
const CEILING_MASS: Record<NtaMassClass, string> = {
  light: 'hsb, sfb, schuimbeton of hout (licht)',
  heavy: 'staal-beton, hout-beton of niet-massief beton (zwaar)',
  very_heavy: 'massief beton (zeer zwaar)',
};

const COLOURS = { white: 'wit', dark: 'zwart, antraciet, donkerbruin', other: 'overige kleuren', unknown: 'onbekende kleur' } as const;

function shadingDevice(device: NtaShadingDevice): { name: string; derived?: string } {
  switch (device.kind) {
    case 'external_screen': return { name: `screens (buiten), ${COLOURS[device.colour]}` };
    case 'external_venetian_blind': return { name: `jaloezieën (buiten), ${COLOURS[device.colour]}` };
    case 'external_roller_shutter':
      // The contract has no dark roller shutter.
      return device.colour === 'dark'
        ? { name: 'rolluiken (buiten), overige kleuren', derived: 'donker rolluik als "overige kleuren" geschreven (het contract kent geen donker rolluik)' }
        : { name: `rolluiken (buiten), ${COLOURS[device.colour]}` };
    case 'internal_metallised_fabric': return { name: 'gemetalliseerde weefsels (binnenzonwering)' };
    case 'drop_arm_awning': return { name: 'uitvalschermen' };
    case 'folding_arm_awning': return { name: 'knikarmschermen' };
  }
}

const CONTROLS: Partial<Record<NtaShadingControl, string>> = {
  manual_residential: 'handbediend',
  automatic_residential_iso52016: 'automatisch intelligent',
  automatic: 'automatisch overig',
};

type NtaInput = NonNullable<IProject['ntaCalculation']> & {
  thermalMass?: { floor?: NtaMassClass | null; wall?: NtaMassClass | null; annexBElements?: unknown[] };
  surfaceTilts?: Array<{ surfaceId: string; tiltDeg: number | null }>;
  windowSolar?: { frameFraction?: number | null; obstruction?: NtaObstruction | null; movableShading?: NtaMovableShading | null };
  windowObstructions?: Array<{ windowId: string; obstruction: NtaObstruction }>;
  windowShadings?: Array<{ windowId: string; movableShading?: NtaMovableShading | null }>;
  windowGlazings?: Array<{ windowId: string; glazing: { fixedLouvres?: { kind: string; control?: string } | null; diffusing?: { gAltitude45: number; gDiffuse: number } | null } }>;
  dynamicWindows?: Array<{ windowId: string }>;
  groundFloors?: Array<{ surfaceId: string; exposedPerimeterM: number | null; edgeThermalBridges?: { method: string } | null; below?: unknown; heatedBasement?: unknown }>;
  ventilativeCooling?: { openings?: unknown[] } | null;
  dwellingType?: string | null;
};

/** The rows of the Uniec geometry import for `project`, with every point the adviser has to know. */
export function buildUniecGeometryExport(project: IProject): UniecGeometryExport {
  const points: UniecExportPoint[] = [];
  const missing = (where: string, message: string) => points.push({ kind: 'missing', where, message });
  const lost = (where: string, message: string) => points.push({ kind: 'lost', where, message });
  const derived = (where: string, message: string) => points.push({ kind: 'derived', where, message });
  const nta = (project.ntaCalculation ?? {}) as NtaInput;
  const text = (value: string, fallback: string, where: string) => {
    const result = contractText(value, fallback);
    if (result !== value) derived(where, `"${value}" geschreven als "${result}" (alleen letters, cijfers, spatie en - _ . , tot 32 tekens).`);
    return result;
  };

  // ── algemene gegevens ──
  if (project.buildingFunction !== 'residential' || nta.dwellingType === 'apartment_building') {
    missing('TypeGebouw', 'Deze export ondersteunt alleen een grondgebonden woning; utiliteitsbouw en appartementen volgen later.');
  }
  const registration = project.registration ?? {};
  const newBuild = registration.purpose === 'delivery' || registration.purpose === 'bbl_check';
  if (!registration.purpose) derived('SoortBouw', 'Geen doel van de berekening in de registratie; als bestaande bouw geschreven.');
  else if (!newBuild) derived('SoortBouw', 'Bestaande bouw geschreven als "niet gerenoveerd"; het project legt geen renovatie vast.');
  const city = project.city?.trim() ?? '';
  if (!city) missing('Plaats', 'Vul de plaats in bij Project; Uniec vraagt een plaatsnaam uit de CBS-lijst.');
  else derived('Plaats', `Plaats "${city}": Uniec accepteert alleen namen uit de CBS-lijst van de specificatie; controleer de schrijfwijze.`);
  const year = registration.constructionYear;
  if (year === undefined || year === null) missing('Bouwjaar', 'Vul het bouwjaar in bij Registratie.');
  const dwellingName = text(project.name || 'Woning', 'Woning', 'Omschrijving');
  const general: GeneralRow = {
    Omschrijving: dwellingName,
    ...(city ? { Plaats: city } : {}),
    TypeGebouw: 'grondgebonden woning',
    SoortBouw: newBuild ? 'nieuwbouw' : 'bestaande bouw - niet gerenoveerd',
    ...(year !== undefined && year !== null ? { Bouwjaar: Math.ceil(year) } : {}),
    Opname: 'detailopname',
    NaamExterneTool: 'Open Energy Studio',
  };
  if (project.basisopname) derived('Opname', 'Het project heeft een basisopname, maar de berekening is een detailberekening; als detailopname geschreven.');

  // ── zone terms ──
  const mass = nta.thermalMass;
  if (!mass?.floor) missing('BouwwijzeVloeren', 'Kies de bouwwijze van de vloeren (tabel 7.10) in de NTA-invoer.');
  if (!mass?.wall) missing('BouwwijzeWanden', 'Kies de bouwwijze van de wanden (tabel 7.10) in de NTA-invoer.');
  if (mass?.floor && mass?.wall) {
    derived('BouwwijzeVloeren', 'De klasse van tabel 7.10 is geschreven als een bouwwijze van het contract met die klasse; controleer de omschrijving in Uniec.');
    derived('BouwwijzePlafonds', 'Het project legt de bouwwijze van de plafonds niet apart vast; de klasse van de vloeren is gebruikt.');
  }
  if (mass?.annexBElements?.length) lost('Cm;int;eff', 'De warmtecapaciteit volgens bijlage B is niet geëxporteerd; de klasse van tabel 7.10 is gebruikt.');
  if (nta.ventilativeCooling?.openings?.length) lost('VentilatieveKoeling', 'De openingen voor ventilatieve koeling zijn niet per raam vast te leggen; alle ramen staan op "niet aanwezig". Vul ze in Uniec in.');
  if (nta.windowSolar?.frameFraction != null) lost('Uw/Ud', 'De kozijnfractie staat niet in het contract.');
  if (nta.dynamicWindows?.length) lost('TypeTransparant', 'Dynamische beglazing (bijlage A) staat niet in het contract.');

  // ── library names: one name per (construction, Vlak) and per (window kind, values, Vlak) ──
  const libraryNames = new Map<string, string>();
  const takenNames = new Map<string, string>();
  const libraryName = (key: string, name: string, typeConstructie: string) => {
    const existing = libraryNames.get(key);
    if (existing) return existing;
    let candidate = name;
    for (let index = 2; takenNames.has(`${candidate}|${typeConstructie}`); index += 1) {
      candidate = contractText(`${name.slice(0, 28)} ${index}`, name);
    }
    takenNames.set(`${candidate}|${typeConstructie}`, key);
    libraryNames.set(key, candidate);
    return candidate;
  };

  const constructions = new Map<string, IConstruction>(project.constructions.map((item) => [item.id, item]));
  const windowTypes = new Map<string, IWindowType>((project.windowTypes ?? []).map((item) => [item.id, item]));
  const tilt = (surfaceId: string) => nta.surfaceTilts?.find((item) => item.surfaceId === surfaceId)?.tiltDeg ?? null;
  const groundFloor = (surfaceId: string) => nta.groundFloors?.find((item) => item.surfaceId === surfaceId);
  const obstructionOf = (window: IWindow) => nta.windowObstructions?.find((item) => item.windowId === window.id)?.obstruction
    ?? nta.windowSolar?.obstruction ?? { method: 'minimal' as const };
  const shadingOf = (window: IWindow) => {
    const own = nta.windowShadings?.find((item) => item.windowId === window.id);
    return own ? own.movableShading ?? null : nta.windowSolar?.movableShading ?? null;
  };

  // A transparent row gives the area per unit and the count (A, AantalTransparanteConstructies)
  // or the total area (Oppervlakte); the contract wants one way for all rows (F112).
  const allWindows = project.zones.flatMap((zone) => zone.surfaces.flatMap((item) => item.windows));
  const perUnit = allWindows.length > 0 && allWindows.every((window) => {
    const unitArea = window.typeId ? windowTypes.get(window.typeId)?.unitArea : undefined;
    return unitArea !== undefined && unitArea > 0 && (window.count ?? 0) >= 1 && Number.isInteger(window.count)
      && Math.abs(unitArea * window.count! - window.area) < 0.005;
  });

  const geometry: GeometryRow[] = [];
  if (project.zones.length === 0) missing('NaamZone', 'Het project heeft geen rekenzone.');
  for (const zone of project.zones) {
    const zoneName = text(zone.name, 'Rekenzone', `rekenzone ${zone.name}`);
    if (!(zone.floorArea > 0)) missing(`rekenzone ${zone.name}`, 'Vul de gebruiksoppervlakte (A_g) van de rekenzone in.');
    if (zone.thermalBridges.length > 0 || (zone.pointThermalBridges?.length ?? 0) > 0) {
      lost(`rekenzone ${zone.name}`, 'Koudebruggen zijn niet geëxporteerd: het contract vraagt per koudebrug de positie (fundering, vloer, dak, vloerongebonden). Voer ze in Uniec in.');
    }
    const zoneColumns: GeometryRow = {
      WoningOmschrijving: dwellingName,
      TypeZone: 'rekenzone',
      NaamZone: zoneName,
      ...(mass?.floor ? { BouwwijzeVloeren: FLOOR_MASS[mass.floor], BouwwijzePlafonds: CEILING_MASS[mass.floor] } : {}),
      ...(mass?.wall ? { BouwwijzeWanden: WALL_MASS[mass.wall] } : {}),
      ...(zone.floorArea > 0 ? { Ag: r014(zone.floorArea) } : {}),
    };
    for (const surface of zone.surfaces) {
      const where = `vlak ${surface.name}`;
      const placement = surfacePlacement(surface);
      if (placement.derived) derived(where, placement.derived);
      if (placement.missing) { missing(where, placement.missing); continue; }
      if (placement.skip) { lost(where, placement.skip); continue; }
      const { vlak, begrenzing } = placement;
      const surfaceName = text(surface.name, 'Vlak', where);
      const base: GeometryRow = { ...zoneColumns, NaamConstructie: surfaceName, Vlak: vlak!, Begrenzing: begrenzing! };
      if (begrenzing === 'buitenlucht' && vlak !== 'vloer boven buitenlucht') {
        if (surface.orientation === 'horizontal' && vlak !== 'dak') { missing(where, 'Een horizontale gevel kan het contract niet bevatten; kies een oriëntatie.'); continue; }
        base.Orientatie = ORIENTATIONS[surface.orientation];
      }
      if (vlak === 'gevel') base.Helling = 90;
      if (vlak === 'dak' && surface.orientation !== 'horizontal') {
        const roofTilt = tilt(surface.id);
        if (roofTilt === null) { missing(where, 'Vul de hellingshoek van dit dak in bij de NTA-invoer.'); continue; }
        base.Helling = r016(roofTilt);
      }
      if (vlak === 'vloer' && begrenzing === 'grond') {
        const floor = groundFloor(surface.id);
        if (floor?.below || floor?.heatedBasement) {
          missing(where, 'Een vloer boven een kruipruimte of een verwarmde kelder ondersteunt deze export nog niet; voer die vloer in Uniec in.');
          continue;
        }
        base.Maaiveld = 'opBovenMv';
        derived(where, 'Vloer op de grond geschreven als op of boven het maaiveld.');
        if (floor?.exposedPerimeterM == null || !(floor.exposedPerimeterM > 0)) {
          missing(where, 'Vul de omtrek (perimeter) van deze vloer op de grond in bij de NTA-invoer.');
        } else {
          base.Perimeter = r014(floor.exposedPerimeterM);
        }
        if (floor?.edgeThermalBridges?.method === 'detailed') lost(where, 'De koudebruggen aan de vloerrand zijn niet geëxporteerd; Uniec rekent met de perimeter.');
      }

      // Opaque part: the gross area less the windows; written after the windows of the surface, as in the example.
      let opaqueRow: GeometryRow | null = null;
      const windowArea = surface.windows.reduce((sum, window) => sum + window.area, 0);
      const opaqueArea = surface.area - windowArea;
      if (opaqueArea > 0.005) {
        const construction = constructions.get(surface.constructionId);
        if (!construction) {
          missing(where, 'Koppel een constructie aan dit vlak.');
        } else if (!(construction.rcValue > 0) || construction.rcValue > 50) {
          missing(where, `Constructie "${construction.name}": R_c ${construction.rcValue} ligt buiten 0 < R_c ≤ 50 m²K/W van het contract.`);
        } else {
          const name = libraryName(`opaque|${construction.id}|${vlak}`, text(construction.name, 'Constructie', `constructie ${construction.name}`), 'dicht');
          const rc = r014(construction.rcValue);
          if (rc !== construction.rcValue) derived(where, `R_c ${construction.rcValue} afgerond op ${rc} (twee decimalen).`);
          opaqueRow = { ...base, NaamBibliotheek: name, TypeConstructie: 'dicht', Methodiek: 'vrije invoer', Rc: rc, Oppervlakte: r014(opaqueArea) };
        }
      } else if (opaqueArea < -0.005) {
        missing(where, `De ramen (${r014(windowArea)} m²) zijn groter dan het vlak (${r014(surface.area)} m²).`);
      }

      for (const window of surface.windows) {
        const windowWhere = `raam ${window.name} op ${surface.name}`;
        if (!['buitenlucht', 'aor_for', 'avr'].includes(begrenzing!)) {
          missing(windowWhere, 'Een raam in dit soort scheidingsconstructie staat niet in het contract.');
          continue;
        }
        if (!(window.area > 0)) { missing(windowWhere, 'Het raam heeft geen oppervlakte.'); continue; }
        if (!(window.uValue > 0) || window.uValue > 10) { missing(windowWhere, `U_w ${window.uValue} ligt buiten 0 < U ≤ 10 W/m²K.`); continue; }
        const type = window.typeId ? windowTypes.get(window.typeId) : undefined;
        const kind = type?.kind === 'door' ? 'deur' : type?.kind === 'panel' ? 'paneel in kozijn' : 'raam';
        const u = r012(window.uValue);
        const g = r011(Math.min(1, Math.max(0, window.gValue)));
        if (u !== window.uValue) derived(windowWhere, `U_w ${window.uValue} afgerond op ${u}.`);
        if (g !== window.gValue) derived(windowWhere, `g ${window.gValue} naar beneden afgerond op ${g} (veelvoud van 0,05).`);
        const name = libraryName(`window|${kind}|${type?.id ?? window.name}|${u}|${g}|${vlak}`, text(type?.name ?? window.name, 'Kozijn', windowWhere), 'transparant');
        const row: GeometryRow = {
          ...base, NaamBibliotheek: name, TypeConstructie: 'transparant', TypeTransparant: kind, Methodiek: 'vrije invoer',
          'Uw/Ud': u, 'ggl;n': g,
          ...(perUnit ? { A: r014(type!.unitArea!), AantalTransparanteConstructies: window.count! } : { Oppervlakte: r014(window.area) }),
        };
        if (begrenzing === 'buitenlucht') {
          row.VentilatieveKoeling = 'niet aanwezig';
          if (g > 0) {
            const shade = windowShade(obstructionOf(window), shadingOf(window),
              nta.windowGlazings?.find((item) => item.windowId === window.id)?.glazing);
            for (const message of shade.missing) missing(windowWhere, message);
            for (const message of shade.derived) derived(windowWhere, message);
            if (shade.missing.length > 0) continue;
            Object.assign(row, shade.columns);
          }
        }
        geometry.push(row);
      }
      if (opaqueRow) geometry.push(opaqueRow);
    }
  }
  return { ok: !points.some((point) => point.kind === 'missing'), general, geometry, points };
}

/** Vlak and Begrenzing of a surface. */
function surfacePlacement(surface: ISurface): { vlak?: string; begrenzing?: string; missing?: string; skip?: string; derived?: string } {
  let boundary: ThermalBoundary | undefined = surface.thermalBoundary;
  let note: string | undefined;
  if (!boundary) {
    boundary = surface.type === 'floor' ? 'ground' : 'outdoor';
    note = `Geen grens vastgelegd; als ${boundary === 'ground' ? 'grond' : 'buitenlucht'} geschreven.`;
  }
  if (surface.type === 'internal' || boundary === 'internal') return { skip: 'Een inwendige scheiding hoort niet bij de thermische schil en is niet geëxporteerd.' };
  const vlak = surface.type === 'wall' ? 'gevel' : surface.type === 'roof' ? 'dak' : 'vloer';
  switch (boundary) {
    case 'outdoor': return { vlak: vlak === 'vloer' ? 'vloer boven buitenlucht' : vlak, begrenzing: 'buitenlucht', derived: note };
    case 'ground':
      if (vlak !== 'vloer') return { missing: 'Een wand of dak tegen de grond (kelderwand) ondersteunt deze export nog niet.' };
      return { vlak, begrenzing: 'grond', derived: note };
    case 'unheated_space':
      return { vlak, begrenzing: 'aor_for', derived: 'Grens naar een aangrenzende onverwarmde ruimte geschreven als "aor_for" (forfaitaire b-factor in Uniec); de gegevens van de ruimte zelf gaan niet mee.' };
    case 'adjacent_conditioned':
      return { vlak, begrenzing: 'avr', derived: note };
  }
  return { missing: 'Onbekende grens van het vlak.' };
}

/** Beschaduwing, Zonwering and Regeling of a window to outside air with g > 0. */
function windowShade(
  obstruction: NtaObstruction, shading: NtaMovableShading | null,
  glazing: { fixedLouvres?: { kind: string; control?: string } | null; diffusing?: { gAltitude45: number; gDiffuse: number } | null } | undefined,
): { columns: GeometryRow; missing: string[]; derived: string[] } {
  const columns: GeometryRow = {};
  const missing: string[] = [];
  const derived: string[] = [];
  switch (obstruction.method) {
    case 'minimal': columns.Beschaduwing = 'minimale belemmering'; break;
    case 'full': columns.Beschaduwing = 'volledige belemmering'; break;
    case 'other':
      columns.Beschaduwing = 'overige belemmering';
      if (obstruction.overhangRelativeHeight != null) derived.push('Het overstek bij een overige belemmering staat niet in het contract en is weggelaten.');
      break;
    default:
      missing.push('Deze export schrijft alleen een minimale, volledige of overige belemmering. Een belemmering met afstanden en hoogtes (evenwijdig, overstek, zijbelemmering) of opgegeven factoren vult u in Uniec in; kies hier dan minimaal, of laat dit raam weg.');
  }
  if (shading && glazing?.fixedLouvres) missing.push('Beweegbare zonwering en vaste lamellen op één raam kan het contract niet bevatten.');
  if (glazing?.diffusing) {
    columns.Zonwering = 'vaste lamellen - ISO 15099;2003';
    columns.GglAlt = glazing.diffusing.gAltitude45;
    columns.GglDif = glazing.diffusing.gDiffuse;
  } else if (glazing?.fixedLouvres) {
    const louvres = glazing.fixedLouvres.kind;
    columns.Zonwering = louvres === 'horizontal90' ? 'vaste horizontale lamellen, niet verdraaibaar, 90°'
      : louvres === 'horizontal_angled' ? 'vaste horizontale lamellen, niet verdraaibaar, onder hoek'
        : 'vaste horizontale lamellen, verdraaibaar';
    if (louvres === 'rotatable') {
      const control = CONTROLS[glazing.fixedLouvres.control as NtaShadingControl];
      if (!control) missing.push('De regeling van de verdraaibare lamellen is niet bekend of geldt niet voor woningbouw.');
      else columns.Regeling = control;
    }
  } else if (shading) {
    if (!shading.device) {
      missing.push('Zonwering met alleen een reductiefactor (F_c) staat niet in het contract; kies het type zonwering (tabel 7.5/7.6).');
    } else {
      const device = shadingDevice(shading.device);
      columns.Zonwering = device.name;
      if (device.derived) derived.push(device.derived);
      // Every movable device needs its control (Regeling, F158).
      const control = CONTROLS[shading.control];
      if (!control) missing.push('De regeling van de zonwering geldt niet voor woningbouw.');
      else if (shading.device.kind === 'internal_metallised_fabric' && control === 'handbediend') missing.push('Binnenzonwering mag in het contract niet handbediend zijn.');
      else columns.Regeling = control;
    }
  } else {
    columns.Zonwering = 'geen zonwering';
  }
  return { columns, missing, derived };
}

/** The workbook of an export that is not refused. */
export function uniecGeometryWorkbook(exported: UniecGeometryExport): Uint8Array {
  const generalRows: XlsxCell[][] = [[...GENERAL_COLUMNS], GENERAL_COLUMNS.map((column) => exported.general[column])];
  const geometryRows: XlsxCell[][] = [[...GEOMETRY_COLUMNS], ...exported.geometry.map((row) => GEOMETRY_COLUMNS.map((column) => row[column]))];
  return writeXlsx([{ name: GENERAL_SHEET, rows: generalRows }, { name: GEOMETRY_SHEET, rows: geometryRows }]);
}

/** Downloads the workbook when the export is possible; returns the export with its points either way. */
export function downloadUniecGeometry(project: IProject): UniecGeometryExport {
  const exported = buildUniecGeometryExport(project);
  if (exported.ok) downloadBytes(uniecGeometryWorkbook(exported), `${contractText(project.name || 'project', 'project')}.uniec-geometrie.xlsx`);
  return exported;
}
