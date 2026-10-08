import type { IProject } from '../energy/types';
import type {
  BuildingPerformanceAssessment, MonthlyDemandAssessment, NtaInterpretationGroup, NtaLabelStatements,
  ProjectPerformanceAssessment,
} from '../nta/KernelClient';
import { attestMark, type AttestMark } from '../nta/Attest';
import { kernelVerdict } from '../nta/KernelVerdict';
import { nl } from '../../i18n/nl';
import { escapeHtml } from './HtmlEscaping';
import { dutchCodeCell, dutchDetailHtml, dutchNumber, dutchTimeHtml, dutchTimestamp } from './DutchReportText';
import { indicatorDecimals, kernelReportModel } from './KernelReportModel';
import {
  chapterFiveSection, coolingSection, hotWaterSection, interpretationsSection, pvSection,
  registrationSection, serviceEnergySection, zebSection,
} from './NtaCalculationReport';

/*
 * "Rapportage Energieprestatie (NTA 8800)": the professional report with three levels.
 *
 * - summary: project, results against the Bbl limits, label data and status;
 * - standard: plus building data, envelope, installations, energy per function and the
 *   input overview;
 * - detailed: plus the selected calculation chapters, down to the monthly intermediate
 *   values of the kernel with their NTA 8800 formula numbers ("formule → waarden → uitkomst").
 *
 * Like the other BRL 9500 documents it is Dutch whatever the UI language. Every number comes
 * from the kernel output (or from the derived kernel input for the input overview); the report
 * computes only presentation values such as A·U per element or the ratios it shows as checks.
 */

export type ReportLevel = 'summary' | 'standard' | 'detailed';

export const DETAIL_SECTIONS = [
  'transmission', 'ventilation', 'gains', 'balance', 'heatingChain', 'hotWater', 'cooling',
  'lighting', 'pv', 'primary', 'tojuli', 'beng1',
] as const;
export type DetailSection = (typeof DETAIL_SECTIONS)[number];

/** Dutch titles of the detail chapters, also used by the report dialog. */
export const DETAIL_SECTION_TITLES: Record<DetailSection, string> = {
  transmission: 'Transmissie naar buiten, grond en onverwarmde ruimten',
  ventilation: 'Ventilatie en infiltratie per maand',
  gains: 'Interne en zonnewinst per maand',
  balance: 'Warmte- en koudebalans per rekenzone',
  heatingChain: 'Verwarmingsketen: afgifte, distributie, opwekking',
  hotWater: 'Warm tapwater per maand',
  cooling: 'Koeling per maand',
  lighting: 'Verlichting',
  pv: 'Zonnestroom per systeem',
  primary: 'Primaire energie, export en hernieuwbaar aandeel',
  tojuli: 'TOjuli per oriëntatie',
  beng1: 'BENG 1 met vast ventilatiesysteem C1',
};

export interface ReportOptions {
  level: ReportLevel;
  /** Detail chapters to include when `level` is `detailed`; all when omitted. */
  details?: Partial<Record<DetailSection, boolean>>;
  interpretations?: NtaInterpretationGroup[];
  generatedAt?: Date;
  /** BRL 9501 attest number; defaults to the program's own (`softwareAttestNumber`). */
  attestNumber?: string | null;
}

const MONTHS = ['jan', 'feb', 'mrt', 'apr', 'mei', 'jun', 'jul', 'aug', 'sep', 'okt', 'nov', 'dec'];
const ORIENTATION: Record<string, string> = {
  north: 'N', north_east: 'NO', east: 'O', south_east: 'ZO', south: 'Z', south_west: 'ZW', west: 'W', north_west: 'NW',
  horizontal: 'horizontaal',
};
const FUNCTION: Record<string, string> = {
  residential: 'woonfunctie', office: 'kantoorfunctie', education: 'onderwijsfunctie', retail: 'winkelfunctie',
  healthcare: 'gezondheidszorgfunctie', sport: 'sportfunctie', lodging: 'logiesfunctie', assembly: 'bijeenkomstfunctie',
  cell: 'celfunctie', industrial: 'industriefunctie', other: 'overige gebruiksfunctie',
};
/** Bbl table 4.148A use functions (Dutch names of the kernel's `NtaBblFunction` values). */
const BBL_FUNCTION: Record<string, string> = {
  residential_building: 'woongebouw', other_residential: 'woonfunctie (niet in een woongebouw)', caravan: 'woonwagen',
  floating_building_after2018_berth: 'drijvend bouwwerk (ligplaats na 2018)', floating_building_other_berth: 'drijvend bouwwerk (andere ligplaats)',
  assembly_child_care: 'bijeenkomstfunctie voor kinderopvang', other_assembly: 'andere bijeenkomstfunctie', cell: 'celfunctie',
  healthcare_with_beds: 'gezondheidszorgfunctie met bedgebied', other_healthcare: 'andere gezondheidszorgfunctie', office: 'kantoorfunctie',
  lodging_in_lodging_building: 'logiesfunctie in een logiesgebouw', other_lodging: 'andere logiesfunctie', education: 'onderwijsfunctie',
  sport: 'sportfunctie', retail: 'winkelfunctie',
};

/**
 * Escaped title text with symbol subscripts: `H_D`, `H_tr` and `Q_H;nd` become H<sub>D</sub> …,
 * so headings and the table of contents read like the tables (review finding).
 */
export function titleHtml(title: string): string {
  return escapeHtml(title).replace(/\b([A-Za-z])_([A-Za-z0-9;]+)/g, '$1<sub>$2</sub>');
}

const STATUS: Record<string, string> = {
  calculated_unverified: 'berekend (onverifieerd)', calculated_legacy_edition: 'berekend in oudere uitgave (niet voor registratie)',
  incomplete: 'onvolledig', invalid: 'ongeldig',
  derived_input_rejected: 'afgeleide invoer afgewezen',
};

const n = (value: number | null | undefined, digits = 0) => dutchNumber(value, digits);
const td = (value: unknown) => `<td>${escapeHtml(value)}</td>`;
const tdn = (value: number | null | undefined, digits = 0) => `<td class="n">${n(value, digits)}</td>`;
const sum = (values: Array<number | null | undefined>) => values.reduce<number>((total, value) =>
  total + (value != null && Number.isFinite(value) ? value : 0), 0);
const meets = (value: boolean | null | undefined) =>
  value == null ? 'niet te toetsen' : value ? 'voldoet (onverifieerd)' : 'voldoet niet';
type Loose = Record<string, unknown>;
const isObject = (value: unknown): value is Loose => value != null && typeof value === 'object' && !Array.isArray(value);

/** Numbered tables, chapters and the table of contents of one report. */
class ReportDocument {
  private tables = 0;
  readonly chapters: Array<{ id: string; title: string; detail: boolean }> = [];

  chapter(id: string, title: string, body: string, detail = false): string {
    if (!body.trim()) return '';
    this.chapters.push({ id, title, detail });
    const number = this.chapters.length;
    return `<section class="chapter" id="${id}"><h2>${number}. ${titleHtml(title)}</h2>${body}</section>`;
  }

  table(caption: string, head: string, body: string, note = ''): string {
    if (!body.trim()) return '';
    this.tables += 1;
    return `<figure class="tbl"><figcaption>Tabel ${this.tables} — ${caption}</figcaption>
      <table>${head ? `<thead>${head}</thead>` : ''}<tbody>${body}</tbody></table>${note ? `<p class="note">${note}</p>` : ''}</figure>`;
  }

  /** One calculation step: "formule → waarden → uitkomst" with its NTA 8800 reference. */
  step(label: string, formula: string, values: string, result: string, reference: string): string {
    return `<div class="step"><span class="label">${label}</span><span class="formula">${formula}</span>`
      + `<span class="values">= ${values}</span><span class="result">= <strong>${result}</strong></span>`
      + `<span class="ref">${escapeHtml(reference)}</span></div>`;
  }
}

/** Monthly table: one row per month plus an annual total row where it makes sense. */
function monthlyTable(doc: ReportDocument, caption: string, columns: Array<{ head: string; values: Array<number | null | undefined>; digits?: number; total?: boolean }>, note = ''): string {
  if (columns.length === 0) return '';
  const head = `<tr><th>Maand</th>${columns.map((column) => `<th>${column.head}</th>`).join('')}</tr>`;
  const rows = MONTHS.map((month, index) => `<tr><th>${month}</th>${columns
    .map((column) => tdn(column.values[index], column.digits ?? 0)).join('')}</tr>`).join('');
  const totals = columns.some((column) => column.total)
    ? `<tr class="total"><th>jaar</th>${columns.map((column) => column.total ? tdn(sum(column.values), column.digits ?? 0) : '<td></td>').join('')}</tr>`
    : '';
  return doc.table(caption, head, rows + totals, note);
}

/** Readable element name: project names behind the kernel ids (`surface:wall-N:opaque`, `window:win-S`). */
function elementNamer(project: IProject) {
  const names = new Map<string, string>();
  for (const zone of project.zones) {
    for (const surface of zone.surfaces) {
      names.set(`surface:${surface.id}`, surface.name);
      for (const window of surface.windows ?? []) names.set(`window:${window.id}`, `${window.name} (${surface.name})`);
    }
  }
  return (id: string) => {
    const base = id.replace(/:opaque$/, '');
    return names.get(base) ?? names.get(id) ?? id;
  };
}

/** The derived demand inputs and the kernel's zone results, in the same order. */
function zonesOf(assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment) {
  const derived = (assessment.derivedInput ?? null) as Loose | null;
  const heating = isObject(derived?.spaceHeating) ? derived!.spaceHeating as Loose : null;
  const inputs: Loose[] = [];
  if (heating && isObject(heating.demand)) inputs.push(heating.demand as Loose);
  for (const zone of (Array.isArray(heating?.additionalZones) ? heating!.additionalZones as Loose[] : [])) {
    if (isObject(zone.demand)) inputs.push(zone.demand as Loose);
  }
  const results: MonthlyDemandAssessment[] = [performance.spaceHeating.demand, ...(performance.spaceHeating.additionalZoneDemands ?? [])];
  return results.map((result, index) => ({
    result,
    input: inputs[index] ?? null,
    id: String(inputs[index]?.zoneId ?? `zone ${index + 1}`),
  }));
}

/* ------------------------------------------------------------------ summary */

function introduction(doc: ReportDocument, project: IProject, assessment: ProjectPerformanceAssessment, generatedAt: Date, attest: AttestMark): string {
  const derived = assessment.derivedInput as unknown as Loose | null;
  const scope = derived?.calculationScope === 'utility' ? 'utiliteitsbouw' : derived?.calculationScope === 'residential' ? 'woningbouw' : '—';
  const registration = project.registration;
  const body = doc.table('Projectgegevens', '', `
      <tr><th>Project</th>${td(project.name || '—')}<th>Project-ID</th>${td(project.id)}</tr>
      <tr><th>Adres</th>${td([project.address, project.city].filter(Boolean).join(', ') || '—')}<th>Gebruiksfunctie</th>${td(FUNCTION[project.buildingFunction] ?? project.buildingFunction)}</tr>
      <tr><th>Rekenroute</th>${td(scope)}<th>Bouwjaar</th>${td(assessment.labelData?.general.constructionYear ?? registration?.constructionYear ?? '—')}</tr>
      <tr><th>Soort opname</th>${td(registration?.surveyType === 'detailed' ? 'detailopname' : registration?.surveyType === 'basic' ? 'basisopname' : '—')}<th>Opdrachtgever</th>${td(registration?.client ?? '—')}</tr>`)
    + doc.table('Uitgangspunten van de berekening', '', `
      <tr><th>Bepalingsmethode</th>${td(assessment.targetNormVersion)}<th>Rekenkern</th>${td(`Open Energy Studio, kernelversie ${assessment.kernelVersion}`)}</tr>
      <tr><th>Status</th>${td(STATUS[assessment.status] ?? assessment.status)}<th>Attest</th>${td(assessment.attestStatus === 'unattested' ? 'niet geattesteerd (BRL 9501)' : assessment.attestStatus)}</tr>
      <tr><th>Datum rapport</th><td>${dutchTimeHtml(generatedAt)}</td><th>Invoervingerafdruk</th><td><code>${escapeHtml(assessment.inputFingerprint)}</code></td></tr>`)
    + `<div class="notice"><strong>Onverifieerde berekening — geen officieel energielabel${attest.attested ? '' : ', niet geattesteerd'}.</strong>
      De uitkomsten komen uit de rekenkern van Open Energy Studio. Een energielabel wordt pas vastgesteld na registratie
      in EP-Online door een gecertificeerde adviseur met een BRL 9501-geattesteerd rekenprogramma.</div>`;
  return doc.chapter('inleiding', 'Inleiding', body);
}

function results(doc: ReportDocument, project: IProject, assessment: ProjectPerformanceAssessment): string {
  const performance = assessment.performance!;
  const model = kernelReportModel(assessment);
  const bbl = performance.bblCheck;
  const label: Record<string, string> = { beng1: 'BENG 1 — energiebehoefte', beng2: 'BENG 2 — primair fossiel energiegebruik', beng3: 'BENG 3 — aandeel hernieuwbare energie' };
  const unit: Record<string, string> = { beng1: 'kWh/m²·jr', beng2: 'kWh/m²·jr', beng3: '%' };
  const rows = (model?.indicators ?? []).map((row) => `<tr><th>${label[row.key]}</th>
      <td class="n">${row.limit == null ? '—' : `${row.higherIsBetter ? '≥' : '≤'} ${n(row.limit, indicatorDecimals(row.key))}`}</td>
      <td class="n">${n(row.value, indicatorDecimals(row.key))}</td>${td(unit[row.key])}${td(meets(row.meets))}</tr>`).join('')
    + (model?.tojuli ? `<tr><th>TO<sub>juli</sub> — risico op oververhitting</th><td class="n">≤ ${n(1.2, 2)}</td>
      <td class="n">${n(model.tojuli.value, 2)}</td>${td('K')}${td(meets(model.tojuli.meets))}</tr>` : '');
  const resultTable = doc.table('Eisen en resultaten (Bbl art. 4.149 en 4.149b, tabel 4.148A)',
    '<tr><th>Indicator</th><th>Eis</th><th>Resultaat</th><th>Eenheid</th><th>Oordeel</th></tr>', rows,
    bbl ? `Gebruiksfunctie ${escapeHtml(BBL_FUNCTION[bbl.function] ?? bbl.function)}, A<sub>ls</sub>/A<sub>g</sub> = ${n(bbl.lossAreaRatio, 2)}${bbl.limits.lightConstructionAllowanceApplied ? '; toeslag lichte bouw (lid 4) toegepast' : ''}. De eis en het resultaat staan met dezelfde nauwkeurigheid; de kern toetst onafgeronde waarden.` : '');
  const elements = assessment.labelData?.indicators?.elements as unknown as Loose | undefined;
  const labelRows = `<tr><th>Indicatieve labelklasse</th>${td(performance.indicativeLabelClass ?? '—')}<th>Labelbron</th>${td(performance.labelSource)}</tr>
    <tr><th>Primair fossiel (label)</th><td class="n">${n(performance.labelPrimaryFossilIndicatorKwhPerM2Year ?? performance.primaryFossilIndicatorKwhPerM2Year, 2)} kWh/m²·jr</td>
      <th>Hernieuwbaar aandeel (label)</th><td class="n">${n(performance.labelPrimaryFossilIndicatorKwhPerM2Year != null ? performance.labelRenewableSharePercent : performance.renewableSharePercent, 1)} %</td></tr>
    <tr><th>CO<sub>2</sub>-emissie</th><td class="n">${n(performance.co2KgPerM2, 1)} kg/m²·jr</td><th>Warmtebehoefte (5.3a)</th><td class="n">${n(performance.chapter5?.heatingNeedKwhPerM2, 2)} kWh/m²·jr</td></tr>
    ${elements ? Object.entries(elements).filter(([key, value]) => value != null && typeof value !== 'object' && !STATEMENT_ELEMENTS.has(key)).map(([key, value]) =>
      `<tr><th colspan="2">${escapeHtml(LABEL_ELEMENT[key] ?? key)}</th><td colspan="2">${escapeHtml(typeof value === 'number' ? n(value, 1) : value)}</td></tr>`).join('') : ''}
    ${project.registration ? labelStatementRows(assessment.labelData?.indicators?.elements, project.registration.labelStatements) : ''}`;
  const labelTable = doc.table('Indicatieve labelklasse en labelgegevens (Omgevingsregeling art. 5.11–5.13a)', '', labelRows);
  const warnings: Array<{ code: string; path: string; detail?: string | null }> = [...(assessment.warnings ?? []), ...(performance.warnings ?? [])];
  const warningTable = warnings.length
    ? doc.table('Meldingen (houden de berekening niet tegen)', '<tr><th>Melding</th><th>Pad</th><th>Toelichting</th></tr>',
      warnings.map((item) => `<tr>${dutchCodeCell(item.code)}${td(item.path)}<td>${dutchDetailHtml(item.detail)}</td></tr>`).join(''))
    : '<p>Geen meldingen.</p>';
  const a0 = performance.a0Check;
  const a0Table = a0 ? doc.table('Aanduiding A0 (Omgevingsregeling art. 5.11/5.12 lid 5)', '', `
      <tr><th>BENG 2 ≤ ${n(a0.primaryFossilMaxKwhPerM2)}</th>${td(meets(a0.primaryFossilMeets))}<th>Geen fossiele verbranding ter plaatse</th>${td(meets(a0.noOnSiteFossilCombustion))}</tr>
      <tr><th>A0 mogelijk</th><td colspan="3">${a0.eligible == null ? 'niet te toetsen' : a0.eligible ? 'ja (onverifieerd)' : 'nee'}</td></tr>`) : '';
  return doc.chapter('resultaten', 'Eisen en resultaten', resultTable + labelTable + a0Table + warningTable
    + `<p class="note">De gekozen lezingen van de rekenkern waar de norm meerdere uitleggen toelaat staan in de bijlage "Interpretaties".</p>`
    + (project.registration ? registrationSection(project.registration, assessment.registration, assessment.labelData?.general.constructionYear).replace(/<h2>Registratie<\/h2>/, '<h3>Registratie</h3>') : ''));
}

/** Label elements k and l are statements, shown by `labelStatementRows`, not in the generic element rows. */
const STATEMENT_ELEMENTS = new Set(['respondsToExternalSignals', 'lowTemperatureHeating']);

/**
 * Omgevingsregeling art. 5.13a lid 1 onder k en l: the adviser's yes/no statements. The kernel's
 * label elements carry them from the registration; the registration itself is the fallback when
 * the kernel gave no label data (no calculated result).
 */
function labelStatementRows(
  kernel: { respondsToExternalSignals: boolean | null; lowTemperatureHeating: boolean | null } | null | undefined,
  statements: NtaLabelStatements | undefined,
): string {
  const answer = (value: boolean | null | undefined) => (value == null ? 'niet beantwoord' : value ? 'ja' : 'nee');
  const k = kernel?.respondsToExternalSignals ?? statements?.respondsToExternalSignals;
  const l = kernel?.lowTemperatureHeating ?? statements?.lowTemperatureHeating;
  return `<tr><th colspan="2">k. Reageert op externe signalen (verklaring adviseur)</th><td colspan="2">${answer(k)}</td></tr>
    <tr><th colspan="2">l. Afgiftesysteem ontworpen voor lage temperatuur (verklaring adviseur)</th><td colspan="2">${answer(l)}</td></tr>`;
}

/**
 * The slot of the NL-EPBD mark on the cover (BRL 9501 §8.4 opmerking, p. 15): only for an
 * attested program. The official artwork replaces the placeholder text once licensed.
 */
function attestMarkSlot(attest: AttestMark): string {
  return attest.markText ? `<div class="attest-mark" data-mark="nl-epbd">${escapeHtml(attest.markText)}</div>` : '';
}

const LABEL_ELEMENT: Record<string, string> = {
  operationalCo2KgPerM2: 'Operationele CO₂-emissie [kg/m²·jr]', wlcGwpKgCo2EqPerM2Year: 'WLC-GWP [kg CO₂-eq/m²·jr]',
  finalEnergyKwhPerM2: 'Finaal energiegebruik [kWh/m²·jr]', annualPrimaryFossilKwh: 'Primair fossiel energiegebruik [kWh/jr]',
  annualRenewablePrimaryKwh: 'Hernieuwbare primaire energie [kWh/jr]', annualFinalEnergyKwh: 'Finaal energiegebruik [kWh/jr]',
  onSiteRenewableProductionKwh: 'Hernieuwbare opwekking op locatie [kWh/jr]', mainEnergyCarrier: 'Belangrijkste energiedrager',
  mainRenewableSource: 'Belangrijkste hernieuwbare bron',
};

/* ------------------------------------------------------------------ standard */

function buildingChapter(doc: ReportDocument, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  const geometry = assessment.geometry;
  const zones = zonesOf(assessment, performance);
  const general = geometry ? doc.table('Algemene gebouwgegevens (§6.6, §6.7)', '', `
      <tr><th>Gebruiksoppervlakte A<sub>g</sub></th><td class="n">${n(geometry.usableFloorAreaM2, 2)} m²</td><th>Verliesoppervlakte A<sub>ls</sub></th><td class="n">${n(geometry.lossAreaM2, 2)} m²</td></tr>
      <tr><th>Compactheid A<sub>ls</sub>/A<sub>g</sub></th><td class="n">${n(geometry.lossAreaRatio, 2)}</td><th>Schiloppervlak</th><td class="n">${n(geometry.envelopeAreaM2, 2)} m²</td></tr>`) : '';
  const zoneRows = zones.map(({ id, input, result }) => `<tr>${td(id)}${td(input?.usageFunction ?? '—')}${tdn(Number(input?.usableFloorAreaM2), 2)}
      ${tdn(result.specificHeatCapacityKjPerM2k)}${tdn(result.transmission?.conductanceWPerK, 2)}${tdn(result.annualHeatingNeedKwh)}${tdn(result.annualCoolingNeedKwh)}</tr>`).join('');
  const zoneTable = doc.table('Rekenzones (§6.5)', '<tr><th>Rekenzone</th><th>Gebruiksfunctie</th><th>A<sub>g</sub> [m²]</th><th>D<sub>m;int</sub> [kJ/m²K]</th><th>H<sub>tr</sub> [W/K]</th><th>Q<sub>H;nd</sub> [kWh]</th><th>Q<sub>C;nd</sub> [kWh]</th></tr>', zoneRows);
  return doc.chapter('gebouw', 'Gebouw en rekenzones', general + zoneTable);
}

function envelopeChapter(doc: ReportDocument, project: IProject, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  const name = elementNamer(project);
  let html = '';
  for (const { id, input } of zonesOf(assessment, performance)) {
    if (!input) continue;
    const transmission = isObject(input.transmission) ? input.transmission as Loose : null;
    const direct = isObject(transmission?.direct) ? transmission!.direct as Loose : null;
    const windows = new Map((Array.isArray(input.windows) ? input.windows as Loose[] : []).map((window) => [String(window.id), window]));
    const elements = Array.isArray(direct?.elements) ? direct!.elements as Loose[] : [];
    const rows = elements.map((element) => {
      const window = windows.get(String(element.id));
      const area = Number(element.areaM2);
      const u = Number(element.uValueWPerM2k);
      return `<tr>${td(name(String(element.id)))}${td(window ? 'raam/deur' : 'dicht')}${td(window ? ORIENTATION[String(window.orientation)] ?? window.orientation : '—')}
        ${tdn(area, 2)}${tdn(u, 3)}${tdn(window ? Number(window.gPerpendicular) : null, 2)}${tdn(area * u, 2)}</tr>`;
    }).join('');
    html += doc.table(`Scheidingsconstructies naar buiten, ${escapeHtml(id)} (§8.2; U inclusief forfaitaire toeslag ΔU<sub>for</sub> waar van toepassing)`,
      '<tr><th>Element</th><th>Soort</th><th>Oriëntatie</th><th>A [m²]</th><th>U [W/m²K]</th><th>g<sub>gl;n</sub></th><th>A·U [W/K]</th></tr>', rows);
    const ground = Array.isArray(transmission?.groundFloors) ? transmission!.groundFloors as Loose[] : [];
    html += doc.table(`Begane grondvloeren, ${escapeHtml(id)} (§8.3, bijlage D)`, '<tr><th>Vloer</th><th>A [m²]</th><th>Omtrek P [m]</th><th>R<sub>si</sub>+R<sub>c</sub> [m²K/W]</th><th>Randisolatie</th><th>Bron</th></tr>',
      ground.map((floor) => `<tr>${td(floor.id)}${tdn(Number(floor.areaM2), 2)}${tdn(Number(floor.exposedPerimeterM), 2)}${tdn(Number(floor.constructionResistanceM2kPerW), 2)}
        ${td(Array.isArray(floor.edgeInsulation) && floor.edgeInsulation.length ? `${floor.edgeInsulation.length} laag/lagen` : 'geen')}${td(floor.sourceReference ?? '')}</tr>`).join(''));
    const linear = Array.isArray(direct?.linearBridges) ? direct!.linearBridges as Loose[] : [];
    const point = Array.isArray(direct?.pointBridges) ? direct!.pointBridges as Loose[] : [];
    html += doc.table(`Thermische bruggen, ${escapeHtml(id)} (§8.2.3/8.2.4)`, '<tr><th>Brug</th><th>Soort</th><th>ψ [W/mK] / χ [W/K]</th><th>Lengte [m] / aantal</th><th>Bron</th></tr>',
      [...linear.map((bridge) => `<tr>${td(bridge.id)}${td('lijnvormig')}${tdn(Number(bridge.psiWPerMk), 3)}${tdn(Number(bridge.lengthM), 2)}${td(bridge.sourceReference ?? '')}</tr>`),
        ...point.map((bridge) => `<tr>${td(bridge.id)}${td('puntvormig')}${tdn(Number(bridge.chiWPerK), 3)}${tdn(Number(bridge.count), 0)}${td(bridge.sourceReference ?? '')}</tr>`)].join('')
      || '', linear.length || point.length ? '' : '');
    if (!linear.length && !point.length) {
      const forfait = elements.some((element) => String(element.sourceReference ?? '').includes('ΔU_for'));
      html += `<p class="note">${forfait ? 'Lineaire thermische bruggen forfaitair via ΔU<sub>for</sub> op alle constructies (8.2/8.3).' : 'Geen thermische bruggen ingevoerd.'}</p>`;
    }
    const pipes = Array.isArray(transmission?.verticalPipes) ? transmission!.verticalPipes as Loose[] : [];
    if (pipes.length) {
      html += doc.table(`Verticale leidingen, ${escapeHtml(id)} (§7.3.3, 7.17)`, '<tr><th>Leiding</th><th>Geïsoleerd</th><th>Bouwlagen</th><th>Bron</th></tr>',
        pipes.map((pipe) => `<tr>${td(pipe.id)}${td(pipe.insulated ? 'ja' : 'nee')}${tdn(Number(pipe.storeys))}${td(pipe.sourceReference ?? '')}</tr>`).join(''));
    }
  }
  return doc.chapter('bouwkundig', 'Bouwkundige uitgangspunten', html);
}

/** Dutch words for the frequent tokens of the kernel input names. */
const WORDS: Record<string, string> = {
  boiler: 'ketel', fuel: 'brandstof', kind: 'soort', location: 'plaats', source: 'bron', reference: 'bron',
  equipment: 'product', installation: 'installatie', year: 'jaar', control: 'regeling', system: 'systeem',
  balancing: 'inregeling', method: 'methode', need: 'behoefte', storage: 'opslag', emission: 'afgifte',
  generator: 'opwekker', generators: 'opwekkers', distribution: 'distributie', area: 'oppervlakte', areas: 'oppervlakten',
  function: 'functie', id: 'id', role: 'rol', present: 'aanwezig', pilot: 'waak', flame: 'vlam', design: 'ontwerp',
  temperature: 'temperatuur', average: 'gemiddelde', circuit: 'circuit', mean: 'gemiddelde', length: 'lengte',
  connection: 'aansluiting', factor: 'factor', volume: 'volume', peak: 'piek', power: 'vermogen', tilt: 'helling',
  azimuth: 'azimut', mounting: 'montage', obstruction: 'belemmering', factors: 'factoren', collective: 'collectief',
  heating: 'verwarming', cooling: 'koeling', hot: 'warm', water: 'water', ventilation: 'ventilatie', supply: 'toevoer',
  extract: 'afvoer', flow: 'debiet', flows: 'debieten', heat: 'warmte', recovery: 'terugwinning', efficiency: 'rendement',
  exchanger: 'warmtewisselaar', infiltration: 'infiltratie', measured: 'gemeten', building: 'gebouw', type: 'type',
  zone: 'zone', zones: 'zones', months: 'maanden', month: 'maand', declared: 'opgegeven', pump: 'pomp',
  external: 'extern', district: 'net', electricity: 'elektriciteit', tap: 'tap', boiling: 'kokend', inside: 'binnen',
};

function readableKey(path: string): string {
  return path.split('.').map((segment) => {
    const index = segment.match(/\[(\d+)\]$/)?.[1];
    const words = segment.replace(/\[\d+\]$/, '').replace(/([a-z0-9])([A-Z])/g, '$1 $2').toLowerCase().split(' ')
      .filter((word) => !['m2', 'c', 'kw', 'kwh', 'k', 'w', 'deg', 'm', 'mk'].includes(word))
      .map((word) => WORDS[word] ?? word);
    return words.join(' ') + (index ? ` ${index}` : '');
  }).join(' › ');
}

/** Dutch label of an enum value from the app's own labels; else the id with spaces. */
let valueLabels: Map<string, string> | null = null;
function readableValue(value: string): string {
  if (!/^[a-z][a-z0-9]*(_[a-z0-9]+)*$/.test(value) || !value.includes('_')) return value;
  if (!valueLabels) {
    valueLabels = new Map();
    for (const [key, text] of Object.entries(nl)) {
      if (!/^(opname\.value|nta\.form|report\.heatingType|report\.ventilationType)\./.test(key)) continue;
      const id = key.slice(key.lastIndexOf('.') + 1);
      if (/^[a-z0-9_]+$/.test(id) && !valueLabels.has(id)) valueLabels.set(id, String(text));
    }
  }
  return valueLabels.get(value) ?? value.replace(/_/g, ' ');
}

/** Readable rows of a derived input object: Dutch name, value and the input path as reference. */
function inputRows(value: unknown, prefix = '', depth = 0, rows: string[] = []): string[] {
  if (depth > 7 || rows.length > 600) return rows;
  const row = (text: string) => rows.push(`<tr><th>${escapeHtml(readableKey(prefix))}</th>${td(text)}<td><code>${escapeHtml(prefix)}</code></td></tr>`);
  if (Array.isArray(value)) {
    if (value.length && value.every((item) => typeof item === 'number')) {
      row(`${value.length === 12 ? '12 maandwaarden: ' : ''}${value.map((item) => n(item as number, 3)).join(' · ')}`);
      return rows;
    }
    value.forEach((item, index) => inputRows(item, `${prefix}[${index + 1}]`, depth + 1, rows));
    return rows;
  }
  if (isObject(value)) {
    for (const [key, item] of Object.entries(value)) inputRows(item, prefix ? `${prefix}.${key}` : key, depth + 1, rows);
    return rows;
  }
  if (value == null) return rows;
  row(typeof value === 'number' ? n(value, Number.isInteger(value) ? 0 : 3) : typeof value === 'boolean' ? (value ? 'ja' : 'nee') : readableValue(String(value)));
  return rows;
}

function installationsChapter(doc: ReportDocument, assessment: ProjectPerformanceAssessment): string {
  const derived = assessment.derivedInput as unknown as Loose | null;
  if (!derived) return '';
  const installations = assessment.labelData?.installations;
  const list = (values: string[] | undefined) => (values && values.length ? values.map(readableValue).join(', ') : '—');
  const overview = installations ? doc.table('Installaties per functie', '', `
      <tr><th>Verwarming</th>${td(list(installations.heatingGenerators?.length ? installations.heatingGenerators : installations.heatingGenerator ? [installations.heatingGenerator] : []))}</tr>
      <tr><th>Warm tapwater</th>${td(list(installations.hotWaterGenerator ? [installations.hotWaterGenerator] : []))}</tr>
      <tr><th>Ventilatie</th>${td(list(installations.ventilationSystems))}</tr>
      <tr><th>Koeling</th>${td(list(installations.coolingGenerators))}</tr>
      <tr><th>Zonnestroom</th>${td(`${installations.pvSystemCount} systeem/systemen`)}</tr>
      <tr><th>Zonneboilers</th>${td(installations.solarWaterHeaterCount)}</tr>
      <tr><th>Verlichtingszones</th>${td(installations.lightingZoneCount)}</tr>`) : '';
  const heating = isObject(derived.spaceHeating) ? derived.spaceHeating as Loose : null;
  const part = (title: string, value: unknown) => {
    const rows = inputRows(value).join('');
    return rows ? doc.table(title, '<tr><th>Gegeven</th><th>Waarde</th><th>Invoerpad</th></tr>', rows) : '';
  };
  return doc.chapter('installaties', 'Installatietechnische uitgangspunten', overview
    + part('Verwarming: opwekking (§9.6)', heating?.generator)
    + part('Verwarming: distributie (§9.5)', heating?.distribution)
    + part('Verwarming: afgifte (§9.4)', heating?.emission)
    + part('Warm tapwater (hoofdstuk 13)', derived.hotWater)
    + part('Ventilatie (hoofdstuk 11)', isObject(heating?.demand) ? (heating!.demand as Loose).ventilation : null)
    + part('Koeling (hoofdstuk 10)', derived.cooling)
    + part('Zonnestroom (hoofdstuk 16)', derived.pvSystems)
    + part('Externe levering (§5.8, bijlage P)', derived.externalSupply));
}

function energyChapter(doc: ReportDocument, performance: BuildingPerformanceAssessment): string {
  const demote = (html: string) => html.replace(/<h2>/g, '<h3>').replace(/<\/h2>/g, '</h3>');
  return doc.chapter('energie', 'Energiegebruik per functie en drager',
    demote(serviceEnergySection(performance) + chapterFiveSection(performance.chapter5) + hotWaterSection(performance)
      + coolingSection(performance) + pvSection(performance) + zebSection(performance)));
}

function inputOverviewChapter(doc: ReportDocument, assessment: ProjectPerformanceAssessment): string {
  if (!assessment.derivedInput) return '';
  const rows = inputRows(assessment.derivedInput).join('');
  return doc.chapter('invoer', 'Invoeroverzicht', `<p>De volledige invoer zoals de rekenkern die heeft ontvangen, met de bronverwijzing per gegeven.
    Getallen in de eenheden van het invoermodel.</p>` + doc.table('Afgeleide kerninvoer', '<tr><th>Gegeven</th><th>Waarde</th><th>Invoerpad</th></tr>', rows));
}

/* ------------------------------------------------------------------ detailed */

function transmissionDetail(doc: ReportDocument, project: IProject, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  const name = elementNamer(project);
  let html = '';
  for (const { id, input, result } of zonesOf(assessment, performance)) {
    const t = result.transmission;
    if (!t) continue;
    const direct = isObject((input?.transmission as Loose | undefined)?.direct) ? ((input!.transmission as Loose).direct as Loose) : null;
    const elements = Array.isArray(direct?.elements) ? direct!.elements as Loose[] : [];
    const products = elements.map((element) => ({ id: String(element.id), area: Number(element.areaM2), u: Number(element.uValueWPerM2k) }));
    const hd = sum(products.map((item) => item.area * item.u));
    html += `<h3>${escapeHtml(id)}</h3>`;
    if (products.length) {
      html += doc.step('H<sub>D</sub>', 'Σ A<sub>i</sub>·U<sub>i</sub> + Σ ψ·l + Σ χ',
        products.map((item) => `${n(item.area, 2)}×${n(item.u, 3)}`).join(' + '), `${n(t.directConductanceWPerK ?? hd, 2)} W/K`, '8.2');
    }
    html += doc.table(`Warmteoverdrachtscoëfficiënten, ${escapeHtml(id)}`, '<tr><th>Term</th><th>Waarde [W/K]</th><th>Bron</th></tr>', `
      <tr><th>H<sub>D</sub> direct naar buiten</th>${tdn(t.directConductanceWPerK, 2)}${td('8.2')}</tr>
      <tr><th>H<sub>U</sub> via onverwarmde ruimten</th>${tdn(t.unheatedConductanceWPerK, 2)}${td('8.4, b_U 8.53–8.59')}</tr>
      <tr><th>H<sub>p</sub> verticale leidingen</th>${tdn(t.verticalPipeConductanceWPerK, 2)}${td('7.17')}</tr>
      <tr><th>H<sub>tr</sub> (zonder grond)</th>${tdn(t.conductanceWPerK, 2)}${td('7.14/7.15')}</tr>
      <tr><th>H<sub>g</sub> stationair</th>${tdn(t.groundSteadyConductanceWPerK, 2)}${td('8.3, bijlage D')}</tr>
      <tr><th>H<sub>H;g;adj</sub> / H<sub>C;g;adj</sub></th><td class="n">${n(t.groundHeatingAdjustedWPerK, 2)} / ${n(t.groundCoolingAdjustedWPerK, 2)}</td>${td('D.2/D.3')}</tr>`);
    if (products.length) {
      html += doc.table(`H<sub>D</sub> per element, ${escapeHtml(id)}`, '<tr><th>Element</th><th>A [m²]</th><th>U [W/m²K]</th><th>A·U [W/K]</th></tr>',
        products.map((item) => `<tr>${td(name(item.id))}${tdn(item.area, 2)}${tdn(item.u, 3)}${tdn(item.area * item.u, 2)}</tr>`).join('')
        + `<tr class="total"><th>Σ</th><td></td><td></td>${tdn(hd, 2)}</tr>`);
    }
    html += monthlyTable(doc, `Grond per maand, ${escapeHtml(id)} (bijlage D.1)`, [
      { head: 'θ<sub>e</sub> [°C]', values: result.monthly.map((m) => m.outdoorTemperatureC), digits: 1 },
      { head: 'H<sub>g;an;mi</sub> [W/K]', values: result.monthly.map((m) => m.groundConductanceWPerK), digits: 2 },
      { head: 'Q<sub>H;tr</sub> [kWh]', values: result.monthly.map((m) => m.heating.transmissionKwh), total: true },
      { head: 'Q<sub>C;tr</sub> [kWh]', values: result.monthly.map((m) => m.cooling.transmissionKwh), total: true },
    ], 'Q<sub>tr</sub> volgens 7.14/7.15 met de rekentemperatuur van §7.9.');
  }
  return html;
}

function ventilationDetail(doc: ReportDocument, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  let html = '';
  for (const { id, result } of zonesOf(assessment, performance)) {
    const months = result.ventilation?.months ?? [];
    html += monthlyTable(doc, `Luchtstromen en H<sub>ve</sub> verwarming, ${escapeHtml(id)} (7.19/7.20, hoofdstuk 11)`, [
      { head: 'q<sub>V;ODA;req</sub> [m³/h]', values: months.map((m) => m.heating.requiredOutdoorAirM3PerH) },
      { head: 'infiltratie [m³/h]', values: months.map((m) => m.heating.infiltrationM3PerH) },
      { head: 'mech. toevoer [m³/h]', values: months.map((m) => m.heating.mechanicalSupplyM3PerH) },
      { head: 'mech. afvoer [m³/h]', values: months.map((m) => m.heating.mechanicalExtractM3PerH) },
      { head: 'nat. toevoer [m³/h]', values: months.map((m) => m.heating.naturalSupplyM3PerH) },
      { head: 'H<sub>ve</sub> [W/K]', values: result.monthly.map((m) => m.heating.ventilationConductanceWPerK), digits: 2 },
      { head: 'Q<sub>H;ve</sub> [kWh]', values: result.monthly.map((m) => m.heating.ventilationKwh), total: true },
    ]);
    html += monthlyTable(doc, `H<sub>ve</sub> koeling en hulpenergie, ${escapeHtml(id)}`, [
      { head: 'H<sub>C;ve</sub> [W/K]', values: result.monthly.map((m) => m.cooling.ventilationConductanceWPerK), digits: 2 },
      { head: 'Q<sub>C;ve</sub> [kWh]', values: result.monthly.map((m) => m.cooling.ventilationKwh), total: true },
      { head: 'ventilatoren [kWh]', values: months.map((m) => m.fanElectricityKwh), total: true },
      { head: 'vorstbeveiliging [kWh]', values: months.map((m) => m.frostProtectionElectricityKwh), total: true },
      { head: 'voorverw. roosters [kWh]', values: months.map((m) => m.grillePreheatingElectricityKwh), total: true },
    ]);
    const jan = result.monthly[0];
    if (jan && result.ventilation?.months?.[0]) {
      const flow = result.ventilation.months[0].heating;
      html += doc.step('H<sub>ve</sub> jan', 'ρ<sub>a</sub>c<sub>a</sub>·Σ b<sub>v,k</sub>·q<sub>V,k</sub>',
        `effectief ${n(flow.effectiveOutdoorAirM3PerH)} m³/h, gewogen`, `${n(jan.heating.ventilationConductanceWPerK, 2)} W/K`, '7.19/7.20');
    }
  }
  return html;
}

function gainsDetail(doc: ReportDocument, project: IProject, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  const name = elementNamer(project);
  let html = '';
  for (const { id, result } of zonesOf(assessment, performance)) {
    html += monthlyTable(doc, `Warmtewinst per maand, ${escapeHtml(id)} (warmtebalans)`, [
      { head: 'Φ<sub>int</sub> [kWh] (7.21–7.29)', values: result.monthly.map((m) => m.internalGainsKwh), total: true },
      { head: 'zon ramen [kWh] (7.32/7.40)', values: result.monthly.map((m) => m.windowSolarGainsKwh), total: true },
      { head: 'zon dicht [kWh] (7.33/7.39)', values: result.monthly.map((m) => m.opaqueSolarGainsKwh), total: true },
      { head: 'serre [kWh] (7.30b)', values: result.monthly.map((m) => m.sunroomGainsKwh ?? null), total: true },
      { head: 'Q<sub>H;gn</sub> [kWh]', values: result.monthly.map((m) => m.heating.gainsKwh), total: true },
    ]);
    const windows = result.monthly[0]?.windowSolarByWindow ?? [];
    if (windows.length) {
      const head = `<tr><th>Raam</th><th>Oriëntatie</th>${MONTHS.map((month) => `<th>${month}</th>`).join('')}<th>jaar</th></tr>`;
      const rows = windows.map((window, index) => {
        const values = result.monthly.map((m) => m.windowSolarByWindow?.[index]?.heatingKwh ?? null);
        return `<tr>${td(name(window.id))}${td(ORIENTATION[window.orientation] ?? window.orientation)}${values.map((value) => tdn(value)).join('')}${tdn(sum(values))}</tr>`;
      }).join('');
      html += doc.table(`Zonnewinst per raam en maand, verwarming [kWh], ${escapeHtml(id)} (7.40 met §17.3)`, head, rows);
    }
  }
  return html;
}

function balanceDetail(doc: ReportDocument, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  let html = '';
  for (const { id, result } of zonesOf(assessment, performance)) {
    const h = result.monthly.map((m) => m.heating);
    const c = result.monthly.map((m) => m.cooling);
    html += `<h3>${escapeHtml(id)}</h3>`;
    html += monthlyTable(doc, `Warmtebalans verwarming, ${escapeHtml(id)}`, [
      { head: 'θ<sub>set</sub> [°C] (7.76)', values: h.map((t) => t.setpointC), digits: 1 },
      { head: 'θ<sub>calc</sub> [°C] (7.59)', values: h.map((t) => t.calculationTemperatureC), digits: 2 },
      { head: 'Q<sub>H;ht</sub> [kWh]', values: h.map((t) => t.heatTransferKwh), total: true },
      { head: 'Q<sub>H;gn</sub> [kWh]', values: h.map((t) => t.gainsKwh), total: true },
      { head: 'γ<sub>H</sub>', values: h.map((t) => t.gamma), digits: 3 },
      { head: 'τ<sub>H</sub> [h] (7.57)', values: h.map((t) => t.timeConstantH), digits: 1 },
      { head: 'a<sub>H</sub> (7.51)', values: h.map((t) => t.a), digits: 3 },
      { head: 'η<sub>H;gn</sub> (7.46–7.49)', values: h.map((t) => t.utilization), digits: 3 },
      { head: 'Q<sub>H;nd</sub> [kWh]', values: h.map((t) => t.needKwh), total: true },
    ]);
    html += monthlyTable(doc, `Warmtebalans koeling, ${escapeHtml(id)}`, [
      { head: 'θ<sub>set</sub> [°C]', values: c.map((t) => t.setpointC), digits: 1 },
      { head: 'Q<sub>C;ht</sub> [kWh]', values: c.map((t) => t.heatTransferKwh), total: true },
      { head: 'Q<sub>C;gn</sub> [kWh]', values: c.map((t) => t.gainsKwh), total: true },
      { head: 'γ<sub>C</sub>', values: c.map((t) => t.gamma), digits: 3 },
      { head: 'τ<sub>C</sub> [h] (7.58)', values: c.map((t) => t.timeConstantH), digits: 1 },
      { head: 'η<sub>C;ls</sub> (7.52–7.54)', values: c.map((t) => t.utilization), digits: 3 },
      { head: 'Q<sub>C;nd</sub> [kWh]', values: c.map((t) => t.needKwh), total: true },
    ]);
    const jan = result.monthly[0];
    if (jan) {
      const ht = jan.heating;
      html += '<h4>Rekenvoorbeeld januari, verwarming</h4>';
      html += doc.step('Q<sub>H;ht</sub>', 'Q<sub>H;tr</sub> + Q<sub>H;ve</sub>', `${n(ht.transmissionKwh)} + ${n(ht.ventilationKwh)}`, `${n(ht.heatTransferKwh)} kWh`, '7.14/7.15, 7.18');
      html += doc.step('Q<sub>H;gn</sub>', 'Φ<sub>int</sub> + Q<sub>sol;ramen</sub> + Q<sub>sol;dicht</sub> + Q<sub>serre</sub>',
        `${n(jan.internalGainsKwh)} + ${n(jan.windowSolarGainsKwh)} + ${n(jan.opaqueSolarGainsKwh)} + ${n(jan.sunroomGainsKwh ?? 0)}`, `${n(ht.gainsKwh)} kWh`, '7.21–7.40');
      if (ht.gamma != null) html += doc.step('γ<sub>H</sub>', 'Q<sub>H;gn</sub> / Q<sub>H;ht</sub>', `${n(ht.gainsKwh)} / ${n(ht.heatTransferKwh)}`, n(ht.gamma, 3), '7.46–7.49');
      html += doc.step('Q<sub>H;nd</sub>', 'Q<sub>H;ht</sub> − η<sub>H;gn</sub> · Q<sub>H;gn</sub>', `${n(ht.heatTransferKwh)} − ${n(ht.utilization, 3)} × ${n(ht.gainsKwh)}`,
        `${n(ht.needKwh)} kWh`, '§7.2, 7.46–7.49');
    }
    const jul = result.monthly[6];
    if (jul) {
      const ct = jul.cooling;
      html += '<h4>Rekenvoorbeeld juli, koeling</h4>';
      html += doc.step('Q<sub>C;nd</sub>', 'a<sub>C;red</sub> · (Q<sub>C;gn</sub> − η<sub>C;ls</sub> · Q<sub>C;ht</sub>)',
        `${n(ct.reductionFactor, 3)} × (${n(ct.gainsKwh)} − ${n(ct.utilization, 3)} × ${n(ct.heatTransferKwh)})`, `${n(ct.needKwh)} kWh`, '§7.2, 7.52–7.54, 7.74');
    }
  }
  return html;
}

function heatingChainDetail(doc: ReportDocument, performance: BuildingPerformanceAssessment): string {
  const m = performance.spaceHeating.monthly;
  if (!m.length) return '';
  const carrier = m.map((row) => row.generatorElectricityKwh + row.naturalGasKwh + row.districtHeatKwh + row.oilKwh + row.biomassKwh);
  let html = monthlyTable(doc, 'Verwarmingsketen per maand [kWh] (hoofdstuk 9)', [
    { head: 'Q<sub>H;nd</sub>', values: m.map((row) => row.heatingNeedKwh), total: true },
    { head: 'afgifteverlies', values: m.map((row) => row.emissionLossKwh), total: true },
    { head: 'Q<sub>H;em;in</sub>', values: m.map((row) => row.emissionInputKwh), total: true },
    { head: 'distributieverlies', values: m.map((row) => row.distributionLossKwh), total: true },
    { head: 'Q<sub>H;gen;out</sub>', values: m.map((row) => row.generatorOutputKwh), total: true },
    { head: 'drager-invoer', values: carrier, total: true },
    { head: 'hulpenergie', values: m.map((row) => row.auxiliaryElectricityKwh), total: true },
    { head: 'terugwinbaar', values: m.map((row) => row.recoverableLossKwh), total: true },
  ], 'Drager-invoer = elektriciteit opwekker + gas + externe warmte + olie + biomassa. De rendementen hieronder zijn uit deze kolommen afgeleid.');
  const need = sum(m.map((row) => row.heatingNeedKwh));
  const emIn = sum(m.map((row) => row.emissionInputKwh));
  const dis = sum(m.map((row) => row.distributionLossKwh));
  const out = sum(m.map((row) => row.generatorOutputKwh));
  const input = sum(carrier);
  if (emIn > 0) html += doc.step('η<sub>H;em</sub> (jaar)', 'Q<sub>H;nd</sub> / Q<sub>H;em;in</sub>', `${n(need)} / ${n(emIn)}`, n(need / emIn, 3), '§9.3 (afgeleid)');
  if (emIn + dis > 0) html += doc.step('η<sub>H;dis</sub> (jaar)', 'Q<sub>H;em;in</sub> / (Q<sub>H;em;in</sub> + Q<sub>H;dis;ls</sub>)', `${n(emIn)} / (${n(emIn)} + ${n(dis)})`, n(emIn / (emIn + dis), 3), '§9.5 (afgeleid)');
  if (input > 0) html += doc.step('η<sub>H;gen</sub> (jaar)', 'Q<sub>H;gen;out</sub> / Σ drager-invoer', `${n(out)} / ${n(input)}`, n(out / input, 3), '§9.6 (afgeleid)');
  html += monthlyTable(doc, 'Drager-invoer verwarming per maand [kWh]', [
    { head: 'elektriciteit', values: m.map((row) => row.generatorElectricityKwh), total: true },
    { head: 'warmtepomp-output', values: m.map((row) => row.heatPumpOutputKwh), total: true },
    { head: 'aardgas', values: m.map((row) => row.naturalGasKwh), total: true },
    { head: 'externe warmte', values: m.map((row) => row.districtHeatKwh), total: true },
    { head: 'olie', values: m.map((row) => row.oilKwh), total: true },
    { head: 'biomassa', values: m.map((row) => row.biomassKwh), total: true },
  ]);
  return html;
}

function hotWaterDetail(doc: ReportDocument, performance: BuildingPerformanceAssessment): string {
  const hot = performance.hotWater;
  if (!hot) return '';
  const m = hot.months;
  return monthlyTable(doc, 'Warm tapwater per maand [kWh] (hoofdstuk 13)', [
    { head: 'Q<sub>W;nd</sub>', values: m.map((row) => row.netNeedKwh), total: true },
    { head: 'Q<sub>W;em;in</sub> (13.17)', values: m.map((row) => row.emissionInputKwh), total: true },
    { head: 'circulatie (13.26)', values: m.map((row) => row.circulationLossKwh), total: true },
    { head: 'opslag (13.58)', values: m.map((row) => row.storageLossKwh), total: true },
    { head: 'Q<sub>W;gen;out</sub>', values: m.map((row) => row.generatorOutputKwh), total: true },
    { head: 'drager-invoer', values: m.map((row) => row.carrierInputKwh), total: true },
    { head: 'η<sub>W;gen</sub>', values: m.map((row) => row.generationEfficiency), digits: 3 },
    { head: 'hulpenergie', values: m.map((row) => row.auxiliaryElectricityKwh), total: true },
    { head: 'zon (hernieuwbaar)', values: m.map((row) => row.solarRenewableKwh), total: true },
  ]);
}

function coolingDetail(doc: ReportDocument, performance: BuildingPerformanceAssessment): string {
  const cooling = performance.cooling;
  if (!cooling) return '';
  const m = cooling.months;
  return monthlyTable(doc, 'Koeling per maand [kWh] (hoofdstuk 10)', [
    { head: 'Q<sub>C;nd</sub>', values: m.map((row) => row.needKwh), total: true },
    { head: 'afgifteverlies (10.15)', values: m.map((row) => row.emissionLossKwh), total: true },
    { head: 'distributieverlies', values: m.map((row) => row.distributionLossKwh), total: true },
    { head: 'opgewekte koude', values: m.map((row) => row.generatorColdKwh), total: true },
    { head: 'elektriciteit', values: m.map((row) => row.electricityKwh), total: true },
    { head: 'hulpenergie', values: m.map((row) => row.auxiliaryElectricityKwh), total: true },
  ]);
}

function lightingDetail(doc: ReportDocument, performance: BuildingPerformanceAssessment): string {
  const lighting = performance.lighting ?? [];
  if (!lighting.length) return '';
  let html = doc.table('Verlichting per rekenzone (hoofdstuk 14)', '<tr><th>Rekenzone</th><th>W<sub>L</sub> [kWh/jr]</th><th>Interne warmte Φ<sub>int;L</sub> [W] (7.28)</th></tr>',
    lighting.map((zone) => `<tr>${td(zone.zoneId)}${tdn(zone.annualKwh)}${tdn(zone.internalGainW, 1)}</tr>`).join(''));
  html += monthlyTable(doc, 'Verlichting per maand [kWh] (verdeling t<sub>mi</sub>/t<sub>an</sub>)',
    lighting.map((zone) => ({ head: escapeHtml(zone.zoneId), values: zone.monthlyKwh, total: true })));
  return html;
}

function pvDetail(doc: ReportDocument, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  const systems = performance.pvSystems ?? [];
  if (!systems.length) return '';
  const inputs = new Map(((assessment.derivedInput as unknown as Loose | null)?.pvSystems as Loose[] | undefined ?? []).map((item) => [String(item.id), item]));
  let html = doc.table('Zonnestroomsystemen (hoofdstuk 16)', '<tr><th>Systeem</th><th>Azimut [°]</th><th>Helling [°]</th><th>Montage</th><th>E<sub>pr;el</sub> [kWh/jr]</th></tr>',
    systems.map((system) => {
      const input = inputs.get(system.id);
      return `<tr>${td(system.id)}${tdn(Number(input?.azimuthDeg))}${tdn(Number(input?.tiltDeg))}${td(input?.mounting ?? '—')}${tdn(system.annualKwh)}</tr>`;
    }).join(''));
  html += monthlyTable(doc, 'Zonnestroom per maand [kWh] (16.2–16.4)', systems.map((system) => ({ head: escapeHtml(system.id), values: system.monthlyKwh, total: true })));
  html += monthlyTable(doc, 'Elektriciteitsbalans per maand [kWh] (5.10–5.13)', [
    { head: 'gebruik', values: performance.electricityBalance.map((row) => row.usedKwh), total: true },
    { head: 'opwekking', values: performance.electricityBalance.map((row) => row.producedKwh), total: true },
    { head: 'eigen gebruik', values: performance.electricityBalance.map((row) => row.selfUsedKwh), total: true },
    { head: 'export', values: performance.electricityBalance.map((row) => row.exportedKwh), total: true },
  ]);
  return html;
}

function primaryDetail(doc: ReportDocument, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  const annual = performance.energyByService?.annual ?? [];
  const carriers = [...new Set(annual.map((row) => row.carrier))];
  let html = doc.table('Afgenomen en primair fossiel per drager (§5.5, tabel 5.2)', '<tr><th>Drager</th><th>Gebruik [kWh]</th><th>Afgenomen [kWh]</th><th>Primair fossiel [kWh]</th><th>Effectieve f<sub>P;del</sub></th></tr>',
    carriers.map((carrier) => {
      const rows = annual.filter((row) => row.carrier === carrier);
      const delivered = sum(rows.map((row) => row.deliveredKwh));
      const primary = sum(rows.map((row) => row.primaryFossilKwh));
      return `<tr>${td(carrier)}${tdn(sum(rows.map((row) => row.usedKwh)))}${tdn(delivered)}${tdn(primary)}${tdn(delivered > 0 ? primary / delivered : null, 3)}</tr>`;
    }).join(''), 'De effectieve factor is primair fossiel gedeeld door afgenomen energie per drager, vóór de exportcorrectie.');
  html += monthlyTable(doc, 'Gebruik per drager per maand [kWh] (E<sub>EPus;ci</sub>)', carriers.map((carrier) => ({
    head: escapeHtml(carrier),
    values: MONTHS.map((_, index) => sum(performance.carriers.filter((row) => row.carrier === carrier && row.month === index + 1).map((row) => row.usedKwh))),
    total: true,
  })));
  const area = assessment.geometry?.usableFloorAreaM2 ?? null;
  const epTot = performance.annualPrimaryFossilKwh;
  const epRen = performance.annualRenewablePrimaryKwh;
  if (area && area > 0) {
    html += doc.step('BENG 2', 'EP<sub>tot</sub> / A<sub>g</sub>', `${n(epTot)} / ${n(area, 2)}`, `${n(performance.primaryFossilIndicatorKwhPerM2Year, 2)} kWh/m²·jr`, '§5.3.1, afgerond op 0,01 naar boven');
  }
  if (epTot != null && epRen != null && epTot + epRen > 0) {
    html += doc.step('BENG 3 (RER)', 'EP<sub>ren</sub> / (EP<sub>tot</sub> + EP<sub>ren</sub>) × 100', `${n(epRen)} / (${n(epTot)} + ${n(epRen)}) × 100`,
      `${n(performance.renewableSharePercent, 1)} %`, '5.3, afgerond op 0,1 naar beneden');
  }
  return html;
}

function tojuliDetail(doc: ReportDocument, performance: BuildingPerformanceAssessment): string {
  const zones = performance.tojuli ?? [];
  if (!zones.length) return '';
  return doc.table('TO<sub>juli</sub> per rekenzone en oriëntatie (§5.7, 5.40)', '<tr><th>Zone</th><th>Oriëntatie</th><th>A [m²]</th><th>Aandeel</th><th>H [W/K]</th><th>Q<sub>C;nd;juli</sub> [kWh]</th><th>TO<sub>juli</sub> [K]</th></tr>',
    zones.flatMap((zone) => zone.activeCooling
      ? [`<tr>${td(zone.zoneId)}<td colspan="5">actieve koeling met voldoende capaciteit (§5.7.1)</td>${tdn(0, 2)}</tr>`]
      : zone.orientations.filter((item) => item.assessed).map((item) => `<tr>${td(zone.zoneId)}${td(ORIENTATION[item.orientation] ?? item.orientation)}
          ${tdn(item.areaM2, 2)}${tdn(item.share, 3)}${tdn(item.conductanceWPerK, 2)}${tdn(item.coolingNeedJulyKwh)}${tdn(item.tojuliK, 2)}</tr>`)).join(''));
}

function beng1Detail(doc: ReportDocument, assessment: ProjectPerformanceAssessment, performance: BuildingPerformanceAssessment): string {
  let html = '';
  let heating = 0;
  let cooling = 0;
  for (const { id, result } of zonesOf(assessment, performance)) {
    const fixed = result.fixedC1;
    if (!fixed) continue;
    heating += fixed.annualHeatingNeedKwh ?? 0;
    cooling += fixed.annualCoolingNeedKwh ?? 0;
    html += monthlyTable(doc, `Behoefte met vast ventilatiesysteem C1, ${escapeHtml(id)} [kWh] (§5.4.2/5.4.3)`, [
      { head: 'Q<sub>H;nd</sub> (C1)', values: fixed.monthlyHeatingNeedKwh, total: true },
      { head: 'Q<sub>C;nd</sub> (C1)', values: fixed.monthlyCoolingNeedKwh, total: true },
    ]);
  }
  const area = assessment.geometry?.usableFloorAreaM2 ?? null;
  if (html && area && area > 0) {
    html += doc.step('BENG 1', '(Q<sub>H;nd</sub> + Q<sub>C;nd</sub>) / A<sub>g</sub>', `(${n(heating)} + ${n(cooling)}) / ${n(area, 2)}`,
      `${n(performance.needIndicatorKwhPerM2Year, 2)} kWh/m²·jr`, '§5.4, afgerond op 0,01 naar boven');
  }
  return html;
}

/* ------------------------------------------------------------------ document */

const STYLE = `
  :root{--ink:#1d2430;--muted:#5a6475;--line:#cfd6df;--head:#eef2f6;--accent:#1f4e79}
  *{box-sizing:border-box}
  body{font:10.5pt/1.45 "Segoe UI",system-ui,sans-serif;color:var(--ink);margin:0;background:#fff}
  .page{max-width:190mm;margin:0 auto;padding:12mm 0}
  .run{display:flex;justify-content:space-between;border-bottom:1px solid var(--line);color:var(--muted);font-size:8.5pt;padding:2mm 0;margin-bottom:6mm}
  .cover{min-height:70mm;border-bottom:3px solid var(--accent);margin-bottom:8mm}
  .cover h1{font-size:22pt;color:var(--accent);margin:0 0 2mm}
  .cover .sub{font-size:13pt;margin:0}
  .cover dl{display:grid;grid-template-columns:40mm 1fr;gap:1mm 4mm;margin-top:8mm;font-size:10pt}
  .cover dt{color:var(--muted)}
  h2{font-size:14pt;color:var(--accent);border-bottom:1px solid var(--line);padding-bottom:1mm;margin:8mm 0 3mm}
  h3{font-size:11.5pt;margin:6mm 0 2mm}
  h4{font-size:10.5pt;margin:4mm 0 1mm}
  nav.toc ol{columns:2;padding-left:6mm}
  nav.toc li.detail{color:var(--muted)}
  figure.tbl{margin:3mm 0 5mm}
  figcaption{font-weight:600;font-size:9.5pt;margin-bottom:1mm}
  table{border-collapse:collapse;width:100%;font-size:9pt}
  th,td{border:1px solid var(--line);padding:1.2mm 2mm;text-align:left;vertical-align:top}
  thead th{background:var(--head)}
  tbody th{background:#f7f9fb;font-weight:500}
  td.n{text-align:right;font-variant-numeric:tabular-nums;white-space:nowrap}
  tr.total th,tr.total td{font-weight:700;background:#f2f5f8}
  p.note{color:var(--muted);font-size:8.5pt;margin:1mm 0 0}
  .notice{border-left:4px solid #b45309;background:#fff8e7;padding:3mm 4mm;margin:4mm 0}
  .attest-mark{display:inline-block;border:1px solid #1f3a5f;padding:2mm 4mm;margin-top:3mm;font-weight:600}
  .step{display:grid;grid-template-columns:30mm 1fr;gap:0 3mm;border-left:3px solid var(--accent);background:#f6f9fc;padding:1.5mm 3mm;margin:1.5mm 0;font-size:9.5pt}
  .step .label{grid-row:span 3;font-weight:600}
  .step .ref{grid-column:2;color:var(--muted);font-size:8.5pt}
  .step .ref::before{content:"NTA 8800 "}
  code{font-size:8.5pt;word-break:break-all}
  @page{size:A4;margin:14mm 12mm 16mm}
  @media print{
    .page{max-width:none;padding:0}
    .run{position:fixed;top:-9mm;left:0;right:0;border:none;margin:0}
    section.chapter{break-before:page}
    section.chapter:first-of-type{break-before:auto}
    figure.tbl,.step{break-inside:avoid}
    h2,h3,h4{break-after:avoid}
    a{color:inherit;text-decoration:none}
  }`;

function detailEnabled(options: ReportOptions, section: DetailSection): boolean {
  if (options.level !== 'detailed') return false;
  return options.details?.[section] ?? true;
}

/**
 * Generates the "Rapportage Energieprestatie (NTA 8800)" as one printable HTML document.
 * When the kernel refused the input (invalid or incomplete) the results are withheld and
 * the report lists the reasons instead.
 */
export function generateEnergyPerformanceReportHTML(project: IProject, assessment: ProjectPerformanceAssessment, options: ReportOptions): string {
  const generatedAt = options.generatedAt ?? new Date();
  const attest = options.attestNumber === undefined ? attestMark() : attestMark(options.attestNumber);
  const doc = new ReportDocument();
  const levelText = options.level === 'summary' ? 'samenvatting' : options.level === 'standard' ? 'standaard' : 'gedetailleerd';
  const runHeader = `<div class="run"><span>${escapeHtml(project.name || 'Project')} — Rapportage Energieprestatie (NTA 8800)</span>
    <span>${escapeHtml(dutchTimestamp(generatedAt))} · kern ${escapeHtml(assessment.kernelVersion)}${attest.attestNumber ? ` · attest ${escapeHtml(attest.attestNumber)}` : ''}</span></div>`;
  const cover = `<header class="cover"><h1>Rapportage Energieprestatie</h1><p class="sub">NTA 8800:2025+C1:2026 — ${escapeHtml(project.name || 'Project')}</p>
    <dl><dt>Adres</dt><dd>${escapeHtml([project.address, project.city].filter(Boolean).join(', ') || '—')}</dd>
    <dt>Rapportniveau</dt><dd>${levelText}</dd><dt>Datum</dt><dd>${dutchTimeHtml(generatedAt)}</dd>
    <dt>Rekenkern</dt><dd>Open Energy Studio ${escapeHtml(assessment.kernelVersion)} (${attest.attestNumber ? `BRL 9501-attest ${escapeHtml(attest.attestNumber)}` : 'niet geattesteerd'})</dd></dl>${attestMarkSlot(attest)}</header>`;
  let chapters = introduction(doc, project, assessment, generatedAt, attest);
  const performance = assessment.performance;
  if (kernelVerdict(assessment) !== 'calculated' || !performance) {
    const reasons = [...assessment.gaps.map((gap) => `<tr>${dutchCodeCell(gap.code)}${td(gap.path)}<td>${dutchDetailHtml(gap.detail)}</td></tr>`),
      ...(performance?.issues ?? []).map((item) => `<tr>${dutchCodeCell(item.code)}${td(item.path)}<td></td></tr>`)].join('');
    chapters += doc.chapter('resultaten', 'Resultaten achtergehouden', `<p>De rekenkern geeft voor deze invoer geen uitkomst
      (status: ${escapeHtml(STATUS[assessment.status] ?? assessment.status)}). Er worden daarom geen BENG-waarden of labelklasse getoond.
      Los eerst de onderstaande punten op.</p>` + doc.table('Invoergaten en afwijzingen', '<tr><th>Code</th><th>Pad</th><th>Toelichting</th></tr>', reasons));
  } else {
    chapters += results(doc, project, assessment);
    if (options.level !== 'summary') {
      chapters += buildingChapter(doc, assessment, performance);
      chapters += envelopeChapter(doc, project, assessment, performance);
      chapters += installationsChapter(doc, assessment);
      chapters += energyChapter(doc, performance);
    }
    if (options.level === 'detailed') {
      const detail: Record<DetailSection, () => string> = {
        transmission: () => transmissionDetail(doc, project, assessment, performance),
        ventilation: () => ventilationDetail(doc, assessment, performance),
        gains: () => gainsDetail(doc, project, assessment, performance),
        balance: () => balanceDetail(doc, assessment, performance),
        heatingChain: () => heatingChainDetail(doc, performance),
        hotWater: () => hotWaterDetail(doc, performance),
        cooling: () => coolingDetail(doc, performance),
        lighting: () => lightingDetail(doc, performance),
        pv: () => pvDetail(doc, assessment, performance),
        primary: () => primaryDetail(doc, assessment, performance),
        tojuli: () => tojuliDetail(doc, performance),
        beng1: () => beng1Detail(doc, assessment, performance),
      };
      for (const section of DETAIL_SECTIONS) {
        if (!detailEnabled(options, section)) continue;
        chapters += doc.chapter(`detail-${section}`, `Berekening: ${DETAIL_SECTION_TITLES[section]}`, detail[section](), true);
      }
    }
    if (options.level !== 'summary') chapters += inputOverviewChapter(doc, assessment);
  }
  chapters += doc.chapter('versie', 'Versie en herleidbaarheid', doc.table('Versiegegevens', '', `
      <tr><th>Bepalingsmethode</th>${td(assessment.targetNormVersion)}</tr>
      <tr><th>Kernelversie</th>${td(assessment.kernelVersion)}</tr>
      <tr><th>Invoervingerafdruk</th><td><code>${escapeHtml(assessment.inputFingerprint)}</code></td></tr>
      <tr><th>Rapport gegenereerd</th><td>${dutchTimeHtml(generatedAt)}</td></tr>`)
    + (performance ? `<ul class="note">${[...new Set([...performance.spaceHeating.demand.omittedCorrections, ...performance.spaceHeating.omittedTerms])]
      .map((item) => `<li>${escapeHtml(item)}</li>`).join('')}</ul>` : ''));
  if (options.interpretations?.length) {
    chapters += doc.chapter('interpretaties', 'Bijlage: interpretaties van de rekenkern',
      interpretationsSection(options.interpretations).replace(/<h2>[^<]*<\/h2>/, ''));
  }
  const toc = `<nav class="toc"><h2>Inhoud</h2><ol>${doc.chapters.map((chapter) =>
    `<li${chapter.detail ? ' class="detail"' : ''}><a href="#${chapter.id}">${titleHtml(chapter.title)}</a></li>`).join('')}</ol></nav>`;
  return `<!doctype html><html lang="nl"><head><meta charset="utf-8">
    <title>Rapportage Energieprestatie (NTA 8800) — ${escapeHtml(project.name || 'project')}</title><style>${STYLE}</style></head>
    <body><div class="page">${runHeader}${cover}${toc}${chapters}</div></body></html>`;
}
