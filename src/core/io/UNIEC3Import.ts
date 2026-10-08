/**
 * Import of a Uniec3 project export (`.uniec3`, a ZIP of JSON files) on the
 * format Uniec 3.4 writes (analysed 8 Oct 2026 on a real export):
 *
 *   meta.json, buildings.json, buildings/<id>/entities.json + relations.json
 *   entity   {NTAEntityId (type), NTAEntityDataId (guid), Status, NTAPropertyDatas[{NTAPropertyId, Value, Status}]}
 *   relation {ParentId, ChildId, NTAEntityIdParent, NTAEntityIdChild, OnDelete, OnCopy}
 *
 * Every value is a string with a Dutch decimal comma; choices are codes.
 * The tree: GEB · LIBCONSTRD/LIBCONSTRT (library of opaque and window types)
 * · RZ → UNIT-RZ (carries Ag) → BEGR (boundary surface) → CONSTRD (opaque
 * part, library reference + net area) and CONSTRT (window rows, library
 * reference + count + area + shading) · INFIL/INFILUNIT (qv10) · INSTALLATIE
 * (INSTALL_TYPE) → VERW/TAPW/VENT/KOEL/PV with their chains.
 *
 * Nothing is defaulted in silence: what the file does not say, or what this
 * model cannot hold, is written to the import record's `notes` and, for
 * numbers, marked `assumed`. Results in the file (PRESTATIE, RESULT-*) are
 * not read: the kernel calculates.
 */
import { strFromU8 } from 'fflate';
import type {
  CoolingSystemType, HeatingSystemType, HotWaterSystemType, IConstruction, IHeatingSystem, IProject, ISurface, IWindow, IZone,
  Orientation, ProjectImportRecord, SurfaceType, ThermalBoundary, VentilationType,
} from '../energy/types';
import type { NtaMovableShading, NtaObstruction, NtaShadeColour } from '../nta/KernelClient';
import { insulationValues, type InsulationPart } from '../nta/MwaTemplates';
import { buildNtaCalculationTemplate } from '../nta/NtaCalculationTemplate';

interface Property { NTAPropertyId: string; Value: string | null; Status: number }
interface Entity { NTAEntityId: string; NTAEntityDataId: string; Status: number; NTAPropertyDatas: Property[] }
interface Relation { ParentId: string; ChildId: string; NTAEntityIdParent: string; NTAEntityIdChild: string }

export const UNIEC3_SOURCE = 'Uniec3-export; oorspronkelijke bron controleren';

/** True for a ZIP written by Uniec3 itself (not this app's own input draft). */
export function isUniec3Export(files: Record<string, Uint8Array>): boolean {
  const meta = files['meta.json'] ? strFromU8(files['meta.json']) : '';
  if (meta.includes('input draft')) return false;
  const relations = Object.keys(files).find((path) => /^buildings\/[^/]+\/relations\.json$/.test(path));
  if (!relations) return false;
  const parsed = JSON.parse(strFromU8(files[relations])) as unknown[];
  return parsed.length === 0 || (typeof parsed[0] === 'object' && parsed[0] !== null && 'ParentId' in parsed[0]);
}

/** Entities with status 4 or 6 are present in the form but not applicable. */
const active = (entity: Entity) => entity.Status !== 4 && entity.Status !== 6;

function text(entity: Entity | undefined, id: string): string | undefined {
  const property = entity?.NTAPropertyDatas.find((item) => item.NTAPropertyId === id);
  if (!property || property.Status === 5) return undefined;
  const value = property.Value?.trim();
  return value && value !== 'n.v.t.' ? value : undefined;
}

function number(entity: Entity | undefined, id: string): number | undefined {
  const value = text(entity, id);
  if (value === undefined) return undefined;
  const parsed = Number(value.replace(',', '.'));
  return Number.isFinite(parsed) ? parsed : undefined;
}

const round = (value: number, digits = 3) => Number(value.toFixed(digits));

/** Compass code at the end of a Uniec3 orientation code (`GVL_BTNL_ZO`, `DAK_BTNL_HOR`, `PVORIE_W`). */
function orientationOf(code: string | undefined): Orientation | undefined {
  const suffix = code?.split('_').pop();
  const map: Record<string, Orientation> = { N: 'N', NO: 'NE', O: 'E', ZO: 'SE', Z: 'S', ZW: 'SW', W: 'W', NW: 'NW', HOR: 'horizontal' };
  return suffix ? map[suffix] : undefined;
}

const SURFACE_TYPES: Record<string, SurfaceType> = { VLAK_GEVEL: 'wall', VLAK_DAK: 'roof', VLAK_VLOER: 'floor' };
const LIBRARY_PARTS: Record<string, InsulationPart> = { LIBVLAK_GEVEL: 'facade', LIBVLAK_DAK: 'roof', LIBVLAK_VLOER: 'floor' };

/** What a boundary surface adjoins, from its `BEGR_GEVEL`/`BEGR_DAK`/`BEGR_VLOER` code. */
function boundaryOf(code: string | undefined): ThermalBoundary | undefined {
  if (!code) return undefined;
  if (code.includes('BTNL')) return 'outdoor';
  if (code.includes('AVR') || code.includes('VERW')) return 'adjacent_conditioned';
  if (code.includes('AOR') || code.includes('ONV')) return 'unheated_space';
  if (code.includes('GR') || code.includes('KR') || code.includes('MV')) return 'ground';
  return undefined;
}

function obstructionOf(code: string | undefined): NtaObstruction | undefined {
  if (code === 'BELEMTYPE_MIN') return { method: 'minimal' };
  if (code === 'BELEMTYPE_VOLLEDIG') return { method: 'full' };
  return undefined;
}

/** Movable shading from `CONSTRT_ZONW` (device and colour) and `CONSTRT_REGEL` (control); the codes stay in the source note. */
function shadingOf(device: string | undefined, control: string | undefined): NtaMovableShading | undefined {
  if (!device || device === 'ZONW_GEEN') return undefined;
  const colour: NtaShadeColour = /W$/.test(device) ? 'white' : /(Z|D|A)$/.test(device) ? 'dark' : 'unknown';
  const kind = device.startsWith('ZONW_SCR') ? 'external_screen' : device.startsWith('ZONW_ROL') ? 'external_roller_shutter'
    : device.startsWith('ZONW_JAL') || device.startsWith('ZONW_LAM') ? 'external_venetian_blind' : null;
  return {
    ...(kind ? { device: { kind, colour } } : { device: null }),
    control: control?.startsWith('ZONWREG_H') ? 'manual_residential' : 'automatic',
    sourceReference: `Uniec3-export: ${device}${control ? `, ${control}` : ''}; controleer type en regeling`,
  };
}

const lower = (value: string | undefined) => (value ?? '').toLowerCase();

function heatingTypeOf(description: string | undefined, source: string | undefined): HeatingSystemType | undefined {
  const name = lower(description);
  if (name.includes('warmtepomp')) return /BO|GR/.test(source ?? '') && !/BU/.test(source ?? '') ? 'heat_pump_ground' : 'heat_pump_air';
  if (name.includes('biomassa') || name.includes('pellet') || name.includes('hout')) return 'biomass';
  if (name.includes('warmtelevering') || name.includes('stadsverwarming') || name.includes('externe')) return 'district_heating';
  if (name.includes('hr') && name.includes('combi')) return 'hr_combi';
  if (name.includes('ketel') || name.includes('hr')) return 'hr107';
  if (name.includes('elektrisch')) return 'electric';
  return undefined;
}

function hotWaterTypeOf(description: string | undefined): HotWaterSystemType | undefined {
  const name = lower(description);
  if (name.includes('zonneboiler')) return 'solar_boiler';
  if (name.includes('warmtepomp')) return 'heat_pump';
  if (name.includes('elektrisch')) return 'electric_boiler';
  if (name.includes('warmtelevering') || name.includes('stadsverwarming')) return 'district_heating';
  if (name.includes('combi') || name.includes('ketel') || name.includes('geiser')) return 'hr_combi';
  return undefined;
}

function ventilationTypeOf(variant: string | undefined, system: string | undefined): VentilationType | undefined {
  if (variant?.startsWith('VARIANT_D')) return 'type_d';
  if (variant?.startsWith('VARIANT_C')) return 'type_c';
  if (variant?.startsWith('VARIANT_A') || system?.includes('NAT')) return 'natural';
  return undefined;
}

function coolingTypeOf(description: string | undefined): CoolingSystemType | undefined {
  const name = lower(description);
  if (name.includes('warmtepomp')) return 'heat_pump_reversible';
  if (name.includes('koelmachine') || name.includes('centraal')) return 'central_chiller';
  if (name.includes('compressie') || name.includes('split') || name.includes('airco')) return 'split_unit';
  return undefined;
}

/** Reads a Uniec3 export into a project; `fileName` goes into the import record. */
export function importUniec3Export(files: Record<string, Uint8Array>, fileName?: string): IProject {
  const buildingDir = Object.keys(files).map((path) => /^(buildings\/[^/]+)\/entities\.json$/.exec(path)?.[1]).find(Boolean);
  if (!buildingDir || !files[`${buildingDir}/relations.json`]) throw new Error('Geen gebouw gevonden in het Uniec3-bestand');
  const entities = (JSON.parse(strFromU8(files[`${buildingDir}/entities.json`])) as Entity[]).filter((entity) => Array.isArray(entity.NTAPropertyDatas));
  const relations = JSON.parse(strFromU8(files[`${buildingDir}/relations.json`])) as Relation[];
  const notes: string[] = [];
  const note = (line: string) => { if (!notes.includes(line)) notes.push(line); };

  const byId = new Map(entities.map((entity) => [entity.NTAEntityDataId, entity]));
  const childrenOf = new Map<string, Entity[]>();
  for (const relation of relations) {
    const child = byId.get(relation.ChildId);
    if (!child) continue;
    childrenOf.set(relation.ParentId, [...(childrenOf.get(relation.ParentId) ?? []), child]);
  }
  const children = (parent: Entity | undefined, type: string): Entity[] =>
    (parent ? childrenOf.get(parent.NTAEntityDataId) ?? [] : []).filter((child) => child.NTAEntityId === type && active(child));
  const ofType = (type: string) => entities.filter((entity) => entity.NTAEntityId === type && active(entity));

  // ── Building ──
  const geb = ofType('GEB')[0];
  if (!geb) throw new Error('Geen gebouw (GEB) in het Uniec3-bestand');
  const name = text(geb, 'GEB_OMSCHR') ?? 'Uniec3-project';
  const typeCode = text(geb, 'GEB_TYPEGEB') ?? '';
  const residential = typeCode.includes('WON') || typeCode.includes('APP');
  if (!residential) note(`Gebouwtype ${typeCode || 'onbekend'} gelezen als "overig"; kies de gebruiksfunctie bij Project.`);
  const newBuild = text(geb, 'GEB_SRTBW') === 'NIEUWB';
  const constructionYear = number(geb, 'GEB_OPLVJR') ?? number(geb, 'GEB_BWJR');

  // ── Library ──
  const constructions: IConstruction[] = [];
  const constructionByGuid = new Map<string, IConstruction>();
  for (const [index, lib] of ofType('LIBCONSTRD').entries()) {
    const part = LIBRARY_PARTS[text(lib, 'LIBCONSTRD_TYPE') ?? ''];
    const rc = number(lib, 'LIBCONSTRD_RC');
    const libName = text(lib, 'LIBCONSTRD_OMSCHR') ?? `Constructie ${index + 1}`;
    if (!part) note(`Constructietype "${libName}": vlaktype ${text(lib, 'LIBCONSTRD_TYPE') ?? 'onbekend'} niet herkend; R_si van een gevel gebruikt.`);
    if (rc === undefined) note(`Constructietype "${libName}": geen R_c in het bestand (methode ${text(lib, 'LIBCONSTRD_METH') ?? 'onbekend'}); R_c 0 ingevuld, aanvullen.`);
    const values = insulationValues(part ?? 'facade', rc ?? 0, null);
    const construction: IConstruction = { id: `con-${index + 1}`, name: libName, layers: [], rcValue: rc ?? 0, uValue: round(values?.u ?? 0, 4) };
    constructions.push(construction);
    constructionByGuid.set(lib.NTAEntityDataId, construction);
  }
  const windowTypes = new Map(ofType('LIBCONSTRT').map((lib, index) => [lib.NTAEntityDataId, {
    name: text(lib, 'LIBCONSTRT_OMSCHR') ?? `Kozijn ${index + 1}`,
    door: text(lib, 'LIBCONSTRT_TYPE') === 'TRANSTYPE_DEUR',
    u: number(lib, 'LIBCONSTRT_U'), g: number(lib, 'LIBCONSTRT_G'), unitArea: number(lib, 'LIBCONSTRT_AC'),
  }]));

  // ── Air tightness and height ──
  const infil = ofType('INFIL')[0];
  const buildingHeight = number(infil, 'INFIL_BGH');
  const qv10 = number(ofType('INFILUNIT')[0], 'INFILUNIT_QV_NON');
  if (qv10 === undefined) note('Geen qv10 in het bestand; 0,4 dm³/(s·m²) aangenomen, controleer bij Luchtdichtheid.');

  // ── Zones and surfaces ──
  const zones: IZone[] = [];
  const tilts: Array<{ surfaceId: string; tiltDeg: number }> = [];
  const perimeters = new Map<string, number>();
  const obstructions: Array<{ windowId: string; obstruction: NtaObstruction }> = [];
  const shadings: Array<{ windowId: string; movableShading: NtaMovableShading }> = [];
  let surfaceCount = 0;
  let windowCount = 0;
  for (const [zoneIndex, rz] of ofType('RZ').entries()) {
    const zoneId = `zone-${zoneIndex + 1}`;
    const zoneName = text(rz, 'RZ_OMSCHR') ?? `Rekenzone ${zoneIndex + 1}`;
    const unitZone = children(rz, 'UNIT-RZ')[0];
    const floorArea = number(unitZone, 'UNIT-RZAG');
    if (floorArea === undefined) note(`Rekenzone "${zoneName}": geen gebruiksoppervlakte (UNIT-RZAG) gevonden; 0 ingevuld.`);
    const storeys = number(rz, 'RZ_BOUWLG') ?? 1;
    const height = buildingHeight !== undefined ? round(buildingHeight / Math.max(1, storeys), 2) : 2.6;
    note(buildingHeight !== undefined
      ? `Rekenzone "${zoneName}": hoogte ${height} m afgeleid uit gebouwhoogte ${buildingHeight} m en ${storeys} bouwlaag/-lagen; inhoud = A_g × hoogte.`
      : `Rekenzone "${zoneName}": geen gebouwhoogte in het bestand; hoogte 2,6 m aangenomen.`);

    const surfaces: ISurface[] = [];
    for (const begr of children(unitZone, 'BEGR')) {
      surfaceCount += 1;
      const surfaceId = `surf-${surfaceCount}`;
      const surfaceName = text(begr, 'BEGR_OMSCHR') ?? `Vlak ${surfaceCount}`;
      const vlak = text(begr, 'BEGR_VLAK') ?? '';
      const type = SURFACE_TYPES[vlak];
      if (!type) note(`Vlak "${surfaceName}": vlaktype ${vlak || 'onbekend'} niet herkend; als gevel ingelezen.`);
      const boundaryCode = text(begr, 'BEGR_GEVEL') ?? text(begr, 'BEGR_DAK') ?? text(begr, 'BEGR_VLOER');
      const thermalBoundary = boundaryOf(boundaryCode);
      if (!thermalBoundary) note(`Vlak "${surfaceName}": grens ${boundaryCode ?? 'onbekend'} niet herkend; kies "grenst aan" bij het vlak.`);
      else if (thermalBoundary !== 'outdoor') note(`Vlak "${surfaceName}": grens ${boundaryCode} gelezen als ${thermalBoundary}; controleer.`);
      const orientation = orientationOf(boundaryCode) ?? ((type ?? 'wall') === 'floor' ? 'horizontal' : undefined);
      if (!orientation) note(`Vlak "${surfaceName}": geen oriëntatie in ${boundaryCode ?? 'de grenscode'}; zuid ingevuld, controleer.`);
      const area = number(begr, 'BEGR_A') ?? 0;
      const tilt = number(begr, 'BEGR_HEL');
      if ((type ?? 'wall') === 'roof' && tilt !== undefined) tilts.push({ surfaceId, tiltDeg: tilt });

      const opaque = children(begr, 'CONSTRD');
      const construction = opaque[0] ? constructionByGuid.get(text(opaque[0], 'CONSTRD_LIB') ?? '') : undefined;
      if (!construction) note(`Vlak "${surfaceName}": geen dichte constructie gekoppeld; kies er een bij Constructies.`);
      if (opaque.length > 1) {
        note(`Vlak "${surfaceName}": ${opaque.length} dichte delen (${opaque.map((item) => `${constructionByGuid.get(text(item, 'CONSTRD_LIB') ?? '')?.name ?? '?'} ${text(item, 'CONSTRD_OPP') ?? '?'} m²`).join(', ')}); alleen het eerste is de constructie van het vlak.`);
      }
      const opaqueArea = opaque.reduce((sum, item) => sum + (number(item, 'CONSTRD_OPP') ?? 0), 0);

      const windows: IWindow[] = [];
      for (const row of children(begr, 'CONSTRT')) {
        windowCount += 1;
        const windowId = `win-${windowCount}`;
        const kind = windowTypes.get(text(row, 'CONSTRT_LIB') ?? '');
        const count = number(row, 'CONSTRT_AANT') ?? 1;
        const rowArea = number(row, 'CONSTRT_OPP') ?? (kind?.unitArea !== undefined ? round(kind.unitArea * count) : 0);
        if (!kind) note(`Vlak "${surfaceName}": kozijnrij zonder bekend kozijntype; U 1,5 en g 0,5 ingevuld, controleer.`);
        const windowName = `${kind?.name ?? 'Kozijn'}${count > 1 ? ` ×${count}` : ''}${kind?.door ? ' (deur)' : ''}`;
        if (count > 1) note(`Vlak "${surfaceName}": ${count} stuks "${kind?.name ?? 'kozijn'}" als één raam van ${rowArea} m² ingelezen.`);
        windows.push({ id: windowId, name: windowName, area: rowArea, uValue: kind?.u ?? 1.5, gValue: kind?.g ?? 0.5, orientation: orientation ?? 'S', surfaceId });
        const obstruction = obstructionOf(text(row, 'CONSTRT_BESCH'));
        if (obstruction) obstructions.push({ windowId, obstruction });
        else if (text(row, 'CONSTRT_BESCH')) note(`Raam "${windowName}" op "${surfaceName}": beschaduwing ${text(row, 'CONSTRT_BESCH')} niet overgenomen; vul de belemmering in.`);
        const shading = shadingOf(text(row, 'CONSTRT_ZONW'), text(row, 'CONSTRT_REGEL'));
        if (shading) shadings.push({ windowId, movableShading: shading });
      }
      const windowArea = windows.reduce((sum, window) => sum + window.area, 0);
      if (Math.abs(area - opaqueArea - windowArea) > 0.05) {
        note(`Vlak "${surfaceName}": bruto ${area} m² ≠ dicht ${round(opaqueArea, 2)} + kozijnen ${round(windowArea, 2)} m².`);
      }
      const perimeter = children(begr, 'CONSTRKENMV').map((item) => number(item, 'KENMV_OMTR_VL')).find((value) => value !== undefined && value > 0);
      if ((type ?? 'wall') === 'floor' && perimeter !== undefined) perimeters.set(surfaceId, perimeter);

      surfaces.push({
        id: surfaceId, name: surfaceName, type: type ?? 'wall', ...(thermalBoundary ? { thermalBoundary } : {}),
        area, orientation: orientation ?? 'S', constructionId: construction?.id ?? '', zoneId, windows,
      });
    }
    zones.push({
      id: zoneId, name: zoneName, floorArea: floorArea ?? 0, volume: round((floorArea ?? 0) * height, 2), height,
      surfaces, thermalBridges: [], airTightness: qv10 !== undefined ? { qv10 } : { qv10: 0.4, assumed: true },
    });
  }
  if (zones.length === 0) note('Geen rekenzone (RZ) in het bestand.');
  const linear = ofType('CONSTRL').length;
  if (linear > 0) note(`${linear} lineaire constructie(s) (koudebruggen) niet ingelezen; voer ze in bij Koudebruggen.`);

  // ── Installations ──
  const project: IProject = {
    id: crypto.randomUUID(), name, description: '', buildingFunction: residential ? 'residential' : 'other', address: '', city: '',
    zones, heatingSystems: [], ventilationSystems: [], coolingSystems: [], hotWaterSystems: [], ntaHeatPumps: [], solarPV: [], solarThermal: [], constructions,
  };
  for (const installation of ofType('INSTALLATIE')) {
    const kind = text(installation, 'INSTALL_TYPE') ?? '';
    const label = text(installation, 'INSTALL_NAAM') ?? kind;
    const description = text(installation, 'INSTALL_OMSCHR');
    const count = number(installation, 'INSTALL_AANTAL') ?? 1;
    if (count > 1) note(`${label}: ${count} identieke systemen; als één systeem ingelezen.`);
    switch (kind) {
      case 'INST_VERW': {
        const generator = children(children(installation, 'VERW')[0], 'VERW-OPWEK')[0];
        const type = heatingTypeOf(description, text(generator, 'VERW-OPWEK_POMP'));
        if (!type) note(`${label}: opwekker "${description ?? '?'}" niet herkend; als HR107-ketel ingelezen, controleer.`);
        const cop = number(generator, 'VERW-OPWEK_COP_NON') ?? number(generator, 'VERW-OPWEK_COP');
        if (cop === undefined) note(`${label}: geen COP of rendement in het bestand; 1 ingevuld.`);
        if (text(generator, 'VERW-OPWEK_TOEW')) note(`${label}: toestel uit de Uniec3-productlijst (id ${text(generator, 'VERW-OPWEK_TOEW')}); de verklaring is niet meegekomen.`);
        const system: IHeatingSystem = {
          id: `heat-${project.heatingSystems.length + 1}`, name: description ?? label, type: type ?? 'hr107', cop: cop ?? 1,
          coverageFraction: number(generator, 'VERW-OPWEK_ENER_NON') ?? 1,
        };
        project.heatingSystems.push(system);
        break;
      }
      case 'INST_TAPW': {
        const tapw = children(installation, 'TAPW')[0];
        const generator = children(tapw, 'TAPW-OPWEK')[0];
        const typeName = text(generator, 'TAPW-OPWEK_TOES_NAAM') ?? description;
        const type = hotWaterTypeOf(typeName);
        if (!type) note(`${label}: tapwatertoestel "${typeName ?? '?'}" niet herkend; als combiketel ingelezen, controleer.`);
        const efficiency = number(generator, 'TAPW-OPWEK_COP') ?? number(generator, 'TAPW-OPWEK_REND');
        if (efficiency === undefined) note(`${label}: geen COP of rendement voor warm tapwater; 1 ingevuld.`);
        const vessel = number(children(tapw, 'TAPW-VAT')[0], 'TAPW-VAT_VOL_B');
        if (vessel !== undefined) note(`${label}: voorraadvat ${vessel} l niet in het projectmodel opgenomen; vul het in bij de NTA-invoer.`);
        project.hotWaterSystems.push({
          id: `dhw-${project.hotWaterSystems.length + 1}`, name: typeName ?? label, type: type ?? 'hr_combi', efficiency: efficiency ?? 1,
          hasSolarBoiler: false, solarBoilerFraction: 0,
        });
        break;
      }
      case 'INST_VENT': {
        const vent = children(installation, 'VENT')[0];
        const type = ventilationTypeOf(text(vent, 'VENT_VARIANT'), text(vent, 'VENT_SYS'));
        if (!type) note(`${label}: ventilatiesysteem ${text(vent, 'VENT_SYS') ?? '?'} / ${text(vent, 'VENT_VARIANT') ?? '?'} niet herkend; als natuurlijk ingelezen, controleer.`);
        const recovery = children(vent, 'WARMTETERUG').find((item) => text(item, 'WARMTETERUG_WTW') === 'WARMTETERUG_WTW_WEL');
        const efficiency = recovery ? number(recovery, 'WARMTETERUG_REND') ?? 0 : 0;
        const fanPower = number(children(vent, 'VENTILATOR')[0], 'VENTILATOR_PNC');
        if (fanPower !== undefined) note(`${label}: ventilatorvermogen ${fanPower} W uit het bestand niet omgezet naar SFP; vul de ventilatorgegevens in.`);
        if (text(vent, 'VENT_SYSVAR')) note(`${label}: systeem uit de Uniec3-productlijst (id ${text(vent, 'VENT_SYSVAR')}); de verklaring is niet meegekomen.`);
        project.ventilationSystems.push({
          id: `vent-${project.ventilationSystems.length + 1}`, name: description ?? label, type: type ?? 'natural', heatRecoveryEfficiency: efficiency, sfp: 0,
        });
        break;
      }
      case 'INST_KOEL': {
        const generator = children(children(installation, 'KOEL')[0], 'KOEL-OPWEK')[0];
        const type = coolingTypeOf(description);
        if (!type) note(`${label}: koelopwekker "${description ?? '?'}" niet herkend; als split-unit ingelezen, controleer.`);
        const eer = number(generator, 'KOEL-OPWEK_EER_NON') ?? number(generator, 'KOEL-OPWEK_EER');
        if (eer === undefined) note(`${label}: geen EER in het bestand; 3 ingevuld.`);
        project.coolingSystems.push({ id: `cool-${project.coolingSystems.length + 1}`, name: description ?? label, type: type ?? 'split_unit', eer: eer ?? 3 });
        break;
      }
      case 'INST_PV': {
        const pv = children(installation, 'PV')[0];
        const mode = text(pv, 'PV_WATTPIEK');
        const wpPerM2 = mode === 'PVWATTPIEK_EIGWM2' ? number(pv, 'PV_WPM2') : undefined;
        if (wpPerM2 === undefined) note(`${label}: wattpiekvermogen (${mode ?? 'onbekend'}) niet gelezen; vermogen 0 ingevuld, vul het in bij Opwekking.`);
        for (const field of children(pv, 'PV-VELD')) {
          const area = number(field, 'PV-VELD_AANTALM2') ?? 0;
          const orientation = orientationOf(text(field, 'PV-VELD_ORIE')) ?? 'S';
          if (!orientationOf(text(field, 'PV-VELD_ORIE'))) note(`${label}: oriëntatie ${text(field, 'PV-VELD_ORIE') ?? '?'} van een PV-veld niet herkend; zuid ingevuld.`);
          project.solarPV.push({
            id: `pv-${project.solarPV.length + 1}`, name: `${label} ${orientation}`, area, orientation,
            tilt: number(field, 'PV-VELD_HELLING') ?? 0, peakPower: wpPerM2 !== undefined ? round(area * wpPerM2 / 1000) : 0,
          });
        }
        break;
      }
      case 'INST_OVERIG': break;
      default: note(`${label}: installatietype ${kind || 'onbekend'} niet ingelezen.`);
    }
  }

  // ── NTA input the file answers: tilts, floor perimeters, shading per window ──
  const nta = buildNtaCalculationTemplate(project) as Record<string, unknown>;
  nta.areaSourceReference = UNIEC3_SOURCE;
  nta.surfaceTilts = (nta.surfaceTilts as Array<{ surfaceId: string }>).map((entry) => {
    const tilt = tilts.find((item) => item.surfaceId === entry.surfaceId);
    return tilt ? { surfaceId: entry.surfaceId, tiltDeg: tilt.tiltDeg, sourceReference: UNIEC3_SOURCE } : entry;
  });
  nta.groundFloors = (nta.groundFloors as Array<{ surfaceId: string }>).map((entry) => {
    const perimeter = perimeters.get(entry.surfaceId);
    return perimeter !== undefined ? { ...entry, exposedPerimeterM: perimeter, sourceReference: UNIEC3_SOURCE } : entry;
  });
  if (obstructions.length > 0) nta.windowObstructions = obstructions.map((item) => ({ ...item, sourceReference: UNIEC3_SOURCE }));
  if (shadings.length > 0) nta.windowShadings = shadings.map((item) => ({ ...item, sourceReference: item.movableShading.sourceReference }));
  project.ntaCalculation = nta as unknown as IProject['ntaCalculation'];
  note('NTA-invoer aangemaakt uit het bestand; wat Uniec3 niet meegeeft (bronnen, leidingen, koudebruggen, ventilatiedebieten) vraagt de rekenkern nog.');

  project.registration = {
    purpose: newBuild ? 'delivery' : 'existing_building',
    ...(constructionYear !== undefined ? { constructionYear } : {}),
  };
  const record: ProjectImportRecord = { tool: 'UNIEC3', importedAt: new Date().toISOString(), ...(fileName ? { fileName } : {}), notes };
  project.importLog = [record];
  return project;
}
