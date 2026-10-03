import type { IProject } from '../energy/types';
import type {
  BuildingPerformanceAssessment, LabelData, NtaChapterFiveIndicators, NtaCoolingResult, NtaInterpretationGroup,
  NtaRegistration, ProjectPerformanceAssessment, RegistrationAssessment,
} from '../nta/KernelClient';
import { escapeHtml } from './HtmlEscaping';
import { summarizeExtras } from '../nta/NtaResultSummary';
import { summarizeServiceEnergy } from '../nta/ServiceEnergy';

const MONTHS = ['jan', 'feb', 'mrt', 'apr', 'mei', 'jun', 'jul', 'aug', 'sep', 'okt', 'nov', 'dec'];

function cell(value: unknown): string { return `<td>${escapeHtml(value)}</td>`; }
function num(value: number | null | undefined, digits = 0): string {
  return value == null || !Number.isFinite(value) ? '—' : value.toFixed(digits);
}
function meets(value: boolean | null | undefined): string {
  return value == null ? 'niet te toetsen' : value ? 'voldoet (onverifieerd)' : 'voldoet niet';
}


const PURPOSE: Record<string, string> = { existing_building: 'bestaand gebouw', delivery: 'oplevering', bbl_check: 'toets Bbl' };
const SURVEY: Record<string, string> = { basic: 'basisopname', detailed: 'detailopname' };
const REPRESENTATION: Record<string, string> = { unique: 'uniek', reference: 'referentiewoning', similar: 'gelijkende woning' };
const CATEGORY: Record<string, string> = { facade: 'Gevel', roof: 'Dak', floor: 'Vloer', glazing: 'Beglazing' };

/** NTA 8800 chapter 5 label and record indicators. */
function chapterFiveSection(indicators: NtaChapterFiveIndicators | null | undefined): string {
  if (!indicators) return '';
  const r = indicators.renewableByCarrier;
  const carbonFree = indicators.locallyCarbonFree == null ? 'niet beoordeeld' : indicators.locallyCarbonFree ? 'ja' : 'nee';
  return `<h2>Indicatoren hoofdstuk 5</h2>
    <table><tbody>
      <tr><th>Nettowarmtebehoefte E<sub>H;nd</sub> (5.3a)</th><td class="n">${num(indicators.heatingNeedKwhPerM2, 2)} kWh/m²·jr</td><th>Nettokoudebehoefte E<sub>C;nd</sub> (5.3d)</th><td class="n">${num(indicators.coolingNeedKwhPerM2, 2)} kWh/m²·jr</td></tr>
      <tr><th>E<sub>H+C;nd</sub> (5.3g)</th><td class="n">${num(indicators.heatingAndCoolingNeedKwhPerM2, 2)} kWh/m²·jr</td><th>Standaard voor woningisolatie (§5.3.2)</th><td class="n">${indicators.standardInsulationKwhPerM2 == null ? '—' : `${num(indicators.standardInsulationKwhPerM2)} kWh/m²·jr — ${meets(indicators.meetsStandardInsulation)}`}</td></tr>
      <tr><th>Hernieuwbaar EwePrenTot (§5.3.1.3)</th><td class="n">${num(indicators.renewableIndicatorKwhPerM2, 2)} kWh/m²·jr</td><th>Renovatiestandaard (tabel 5.7)</th><td class="n">${indicators.renovationStandardKwhPerM2 == null ? '—' : `${num(indicators.renovationStandardKwhPerM2, 2)} kWh/m²·jr — ${meets(indicators.meetsRenovationStandard)}`}</td></tr>
      <tr><th>Finaal energiegebruik EweFinal (5.3h)</th><td class="n">${num(indicators.finalEnergyKwhPerM2, 2)} kWh/m²·jr</td><th>EweFinal;EED (5.3i)</th><td class="n">${num(indicators.finalEnergyEedKwhPerM2, 2)} kWh/m²·jr</td></tr>
      <tr><th>Elektriciteit afgenomen (5.17)</th><td class="n">${num(indicators.deliveredElectricityKwh)} kWh · ${num(indicators.deliveredElectricityKwhPerM2, 1)} kWh/m²</td><th>Externe warmte/koude (5.18)</th><td class="n">${num(indicators.deliveredExternalGj, 1)} GJ · ${num(indicators.deliveredExternalGjPerM2, 3)} GJ/m²</td></tr>
      <tr><th>Overige dragers (5.19)</th><td class="n">${num(indicators.deliveredOtherM3Aeq)} m³ aeq · ${num(indicators.deliveredOtherM3AeqPerM2, 2)} m³ aeq/m²</td><th>Lokaal koolstofemissievrij (§5.5.7)</th>${cell(carbonFree)}</tr>
      <tr><th>Hernieuwbaar per drager (5.39a–h) [kWh]</th><td colspan="3">elektriciteit ${num(r.electricity)} · warmtepomp ${num(r.heatPumpHeat)} · zon thermisch ${num(r.solarHeat)} · koude ${num(r.cold)} · biomassa ${num(r.biomass)} · externe warmte ${num(r.externalHeat)} · externe koude ${num(r.externalCold)}</td></tr>
    </tbody></table>`;
}

const SERVICE: Record<string, string> = {
  heating: 'Verwarming (E<sub>H;corr</sub>)', hotWater: 'Warm tapwater (E<sub>W</sub>)', cooling: 'Koeling (E<sub>C;corr</sub>)',
  humidification: 'Bevochtiging (E<sub>hum</sub>)', ventilation: 'Ventilatie (E<sub>V</sub>)', lighting: 'Verlichting (E<sub>L</sub>)',
  auxiliary: 'Hulpenergie (W<sub>aux;tot</sub>, 5.21)', heatPumpSource: 'Collectieve warmtepompbron (Q<sub>HD;hp;in;bron</sub>)',
};
const CARRIER: Record<string, string> = { el: 'el', gas: 'gas', oil: 'olie', bm: 'biomassa', dh: 'warmte (dh)', dw: 'tapwater (dw)', dc: 'koude (dc)' };

/** §5.5.3/5.20: energy per energy function and carrier. */
function serviceEnergySection(performance: BuildingPerformanceAssessment): string {
  const summary = summarizeServiceEnergy(performance.energyByService);
  if (!summary) return '';
  const head = summary.carriers.map((carrier) => `<th>${escapeHtml(CARRIER[carrier] ?? carrier)}</th>`).join('');
  const rows = summary.rows.map((row) => `<tr><th>${SERVICE[row.service] ?? escapeHtml(row.service)}</th>${summary.carriers
    .map((carrier) => `<td class="n">${num(row.usedKwh[carrier] ?? 0)}</td>`).join('')}
    <td class="n">${num(row.deliveredKwh)}</td><td class="n">${num(row.primaryFossilKwh)}</td><td class="n">${num(row.renewablePrimaryKwh)}</td></tr>`).join('');
  const span = summary.carriers.length + 1;
  return `<h2>Energie per energiefunctie (§5.5.3, 5.20)</h2>
    <table><thead><tr><th>Energiefunctie</th>${head}<th>Afgenomen [kWh]</th><th>Primair fossiel [kWh]</th><th>Hernieuwbaar [kWh]</th></tr></thead><tbody>${rows}
      <tr><th>Export elektriciteit (5.10/5.13)</th><td colspan="${span}"></td><td class="n">−${num(summary.exportedElectricityCreditKwh)}</td><td></td></tr>
      <tr><th>Opslagcorrectie (5.14a)</th><td colspan="${span}"></td><td class="n">−${num(summary.storageCorrectionKwh)}</td><td></td></tr>
      <tr><th>Hernieuwbare elektriciteit (5.39a)</th><td colspan="${span}"></td><td></td><td class="n">${num(summary.renewableElectricityKwh)}</td></tr>
      <tr><th>Totaal (EPtot / EPrenTot)</th><td colspan="${span}"></td><td class="n">${num(summary.primaryFossilTotalKwh)}</td><td class="n">${num(summary.renewableTotalKwh)}</td></tr>
    </tbody></table>
    <p>Gebruik per drager in kWh (E<sub>EPus;ci</sub>). De eigen benutte opwekking (5.22) is naar rato van het elektriciteitsgebruik over de energiefuncties verdeeld; de norm bepaalt die alleen per drager.</p>`;
}

/** Chapter 13: need, losses and generator. */
function hotWaterSection(performance: BuildingPerformanceAssessment): string {
  const hot = performance.hotWater;
  if (!hot) return '';
  const total = (field: keyof (typeof hot.months)[number]) =>
    hot.months.reduce((sum, month) => sum + (Number(month[field]) || 0), 0);
  const carrier = total('carrierInputKwh');
  return `<h2>Warm tapwater (hoofdstuk 13)</h2><table><tbody>
    <tr><th>Netto behoefte Q<sub>W;nd</sub></th><td class="n">${num(hot.annualNetNeedKwh)} kWh</td><th>Afgifterendement η<sub>W;em</sub></th><td class="n">${num(hot.emissionEfficiency, 3)}</td></tr>
    <tr><th>Afgifte-invoer Q<sub>W;em;in</sub> (13.17)</th><td class="n">${num(total('emissionInputKwh'))} kWh</td><th>Circulatieverlies (13.26)</th><td class="n">${num(total('circulationLossKwh'))} kWh</td></tr>
    <tr><th>Opslagverlies (13.58)</th><td class="n">${num(total('storageLossKwh'))} kWh</td><th>Terugwinbaar verlies (13.1.2)</th><td class="n">${num(total('recoverableLossKwh'))} kWh</td></tr>
    <tr><th>Opwekkeroutput Q<sub>W;gen;out</sub></th><td class="n">${num(hot.annualGeneratorOutputKwh)} kWh</td><th>Opwekkerinvoer (drager)</th><td class="n">${num(carrier)} kWh</td></tr>
    <tr><th>Opwekkingsrendement (jaar)</th><td class="n">${carrier > 0 ? num(hot.annualGeneratorOutputKwh / carrier, 3) : '—'}</td><th>Hulpenergie W<sub>W;aux</sub></th><td class="n">${num(total('auxiliaryElectricityKwh'))} kWh</td></tr>
    <tr><th>Zonne-energie (hernieuwbaar)</th><td class="n">${num(hot.annualSolarRenewableKwh)} kWh</td><th>Omgevingswarmte warmtepomp</th><td class="n">${num(total('ambientHeatKwh'))} kWh</td></tr>
    <tr><th>Opwekkers</th><td colspan="3">${hot.generators.map((item) => `#${item.index + 1}: ${num(item.monthlyOutputKwh.reduce((a, b) => a + b, 0))} kWh`).join(' · ') || '—'}</td></tr>
  </tbody></table>`;
}

function coolingRows(result: NtaCoolingResult, label: string): string {
  const total = (field: 'needKwh' | 'emissionLossKwh' | 'distributionLossKwh' | 'generatorColdKwh' | 'electricityKwh' | 'auxiliaryElectricityKwh') =>
    result.months.reduce((sum, month) => sum + (month[field] ?? 0), 0);
  const generators = result.generatorShares.map((item) => {
    const eer = item.monthlyEer.filter((value) => value > 0);
    const mean = eer.length ? eer.reduce((a, b) => a + b, 0) / eer.length : null;
    return `${escapeHtml(item.id)} (methode ${item.method}, aandeel jul–sep ${num(item.shareJulyToSeptember * 100)} %, gem. EER ${num(mean, 2)})`;
  }).join('; ');
  return `<tr>${cell(label)}<td class="n">${num(total('needKwh'))}</td><td class="n">${num(total('emissionLossKwh'))}</td><td class="n">${num(total('distributionLossKwh'))}</td>
    <td class="n">${num(total('generatorColdKwh'))}</td><td class="n">${num(total('electricityKwh'))}</td><td class="n">${num(total('auxiliaryElectricityKwh'))}</td><td>${generators || '—'}</td></tr>`;
}

/** Chapter 10 per cooling system (§10.2). */
function coolingSection(performance: BuildingPerformanceAssessment): string {
  const cooling = performance.cooling;
  if (!cooling) return '';
  const systems = cooling.systems?.length
    ? cooling.systems.map((item, index) => coolingRows(item.assessment, `koelsysteem ${index + 1} (zones ${item.zoneIndexes.map((zone) => zone + 1).join(', ')})`))
    : [coolingRows(cooling, 'koelsysteem')];
  return `<h2>Koeling (hoofdstuk 10)</h2><table><thead><tr><th>Systeem</th><th>Q<sub>C;nd</sub> [kWh]</th><th>Afgifteverlies [kWh]</th><th>Distributieverlies [kWh]</th>
    <th>Opgewekte koude [kWh]</th><th>Elektriciteit [kWh]</th><th>Hulpenergie [kWh]</th><th>Opwekkers</th></tr></thead><tbody>${systems.join('')}</tbody></table>
    <p>Koelgrens ${num(cooling.coolingLimitC, 1)} °C · Δϑ<sub>int;inc</sub> ${num(cooling.internalTemperatureShiftK, 1)} K.</p>`;
}

/** Chapter 16 per PV system. */
function pvSection(performance: BuildingPerformanceAssessment): string {
  const systems = performance.pvSystems ?? [];
  if (systems.length === 0) return '';
  const rows = systems.map((item) => `<tr>${cell(item.id)}<td class="n">${num(item.annualKwh)}</td>${item.monthlyKwh.map((value) => `<td class="n">${num(value)}</td>`).join('')}</tr>`).join('');
  return `<h2>Zonnestroom (hoofdstuk 16)</h2><table><thead><tr><th>Systeem</th><th>E<sub>pr;el</sub> [kWh/jr]</th>${MONTHS.map((month) => `<th>${month}</th>`).join('')}</tr></thead><tbody>${rows}</tbody></table>`;
}

/** Annex AB (informative) ZEB indicator. */
function zebSection(performance: BuildingPerformanceAssessment): string {
  if (performance.zebPrimaryTotalIndicatorKwhPerM2 == null && performance.annualZebPrimaryTotalKwh == null) return '';
  return `<h2>ZEB-indicator (bijlage AB, informatief)</h2><table><tbody>
    <tr><th>EweP,ZEB;Tot (AB.1)</th><td class="n">${num(performance.zebPrimaryTotalIndicatorKwhPerM2, 2)} kWh/m²·jr</td><th>E<sub>P,ZEB;Tot;an</sub></th><td class="n">${num(performance.annualZebPrimaryTotalKwh)} kWh</td></tr>
    <tr><th>m<sub>CO2;ZEB</sub> (AB.3)</th><td class="n">${num(performance.annualZebCo2Kg)} kg/jr</td><th>Status</th>${cell('informatief, telt niet mee voor label of Bbl')}</tr>
  </tbody></table>`;
}

/** Appendix: the kernel's interpretation choices. */
function interpretationsSection(groups: NtaInterpretationGroup[] | undefined): string {
  if (!groups || groups.length === 0) return '';
  return `<h2>Bijlage: interpretaties van de rekenkern</h2>
    <p>Waar de normtekst meerdere lezingen toelaat of een formule als gedrukt een implausibele uitkomst geeft, legt de kern de gekozen lezing vast. Paginaverwijzingen gaan naar NTA 8800:2025+C1:2026.</p>
    ${groups.map((group) => `<h3>${escapeHtml(group.part)} <small>(${escapeHtml(group.module)})</small></h3><ul>${group.items.map((item) => `<li>${escapeHtml(item)}</li>`).join('')}</ul>`).join('')}`;
}

const MESSAGE_TYPE: Record<string, string> = {
  regular: 'registratie', relabel: 'herlabelen', replacement: 'vervangen onjuist label',
};

function advisor(value: NtaRegistration['surveyingAdvisor']): string {
  if (!value) return '—';
  return `${value.name || '—'} (vakbekwaamheid ${value.competenceNumber || '—'})`;
}

/** Every reason a registration is not ready: an incomplete dossier and/or no attest. */
export function readinessText(assessment: RegistrationAssessment | null | undefined): string {
  if (!assessment) return '—';
  if (assessment.readyForRegistration) return 'ja';
  const reasons: string[] = [];
  if (!(assessment.dossierComplete ?? assessment.issues.length === 0)) reasons.push('dossier onvolledig');
  if (assessment.softwareAttested === false) reasons.push('rekenprogramma nog niet geattesteerd (BRL 9501)');
  return reasons.length ? `nee, ${reasons.join('; ')}` : 'nee';
}

/** Report header of BRL 9500 §4.2.5/§4.2.6: advisers, dates and validity. */
function registrationSection(registration: NtaRegistration | undefined, assessment: RegistrationAssessment | null | undefined): string {
  if (!registration) {
    return '<h2>Registratie</h2><p>Geen registratiegegevens ingevuld (projectgegevens → Registratie).</p>';
  }
  const issues = [...(assessment?.issues ?? []), ...(assessment?.plausibility ?? [])].map((item) =>
    `<tr>${cell(item.severity === 'error' ? 'fout' : item.severity === 'warning' ? 'plausibiliteit' : 'ontbreekt')}${cell(item.code)}${cell(item.path)}</tr>`).join('');
  const messageType = assessment?.messageType ?? registration.messageType ?? (registration.relabel ? 'relabel' : 'regular');
  const software = assessment?.software ?? registration.software;
  const softwareText = software
    ? `${software.name} ${software.version}, ${software.attestNumber ? `attest ${software.attestNumber}` : 'nog niet geattesteerd (BRL 9501)'}`
    : '—';
  const wlc = registration.wlcGwp;
  const wlcText = wlc?.valueKgCo2EqPerM2Year != null
    ? `${num(wlc.valueKgCo2EqPerM2Year, 2)} kg CO₂-eq/m²·jr${wlc.reportReference ? ` (${wlc.reportReference})` : ''}`
    : assessment?.wlcGwpRequired ? 'vereist, niet ingevuld' : 'niet vereist';
  const address = [registration.postcode, registration.houseNumber, registration.houseNumberAddition].filter(Boolean).join(' ');
  return `<h2>Registratie</h2><table><tbody>
    <tr><th>Doel</th>${cell(PURPOSE[registration.purpose ?? ''] ?? '—')}<th>Opname</th>${cell(SURVEY[registration.surveyType ?? ''] ?? '—')}</tr>
    <tr><th>Representativiteit</th>${cell(REPRESENTATION[registration.representation ?? ''] ?? '—')}<th>Berichttype</th>${cell(MESSAGE_TYPE[messageType] ?? messageType)}</tr>
    <tr><th>Adres (postcode, huisnummer)</th>${cell(address || '—')}<th>BAG-verblijfsobject</th>${cell(registration.bagObjectId ?? '—')}</tr>
    <tr><th>Bouwjaar</th>${cell(registration.constructionYear ?? '—')}<th>Woningtype / gebruiksfunctie</th>${cell(registration.buildingType ?? '—')}</tr>
    <tr><th>Opdrachtgever</th>${cell(registration.client ?? '—')}<th>Certificaatnummer</th>${cell(registration.certificateNumber ?? '—')}</tr>
    <tr><th>Opnemend adviseur</th>${cell(advisor(registration.surveyingAdvisor))}<th>Registrerend adviseur</th>${cell(advisor(registration.registeringAdvisor))}</tr>
    <tr><th>Opnamedatum</th>${cell(registration.surveyDate ?? '—')}<th>Registratiedatum</th>${cell(registration.registrationDate ?? '—')}</tr>
    <tr><th>Uiterste registratiedatum</th>${cell(assessment?.registrationDeadline ?? assessment?.relabelDeadline ?? '—')}<th>Geldig tot (opnamedatum + 10 jaar)</th>${cell(assessment?.validUntil ?? '—')}</tr>
    <tr><th>EP-Online-nummer</th>${cell(registration.epOnlineNumber ?? 'nog niet geregistreerd')}<th>Dossier compleet</th>${cell(assessment ? ((assessment.dossierComplete ?? assessment.issues.length === 0) ? 'ja' : 'nee') : '—')}</tr>
    <tr><th>Gereed voor registratie</th>${cell(readinessText(assessment))}<th>Rekenprogramma geattesteerd</th>${cell(assessment?.softwareAttested ? 'ja' : 'nee')}</tr>
    <tr><th>Rekenprogramma (Regeling art. 5)</th>${cell(softwareText)}<th>WLC-GWP</th>${cell(wlcText)}</tr>
    ${messageType === 'replacement' ? `<tr><th>Vervangt label</th>${cell(registration.replacedEpOnlineNumber ?? '—')}<th>Uiterste vervangdatum</th>${cell(assessment?.replacementDeadline ?? '—')}</tr>` : ''}
  </tbody></table>
  ${issues ? `<table><thead><tr><th>Soort</th><th>Code</th><th>Pad</th></tr></thead><tbody>${issues}</tbody></table>` : ''}
  ${assessment ? `<p>Bron termijnen: ${escapeHtml(assessment.source)}.</p>` : ''}`;
}

/** Label data of Regeling energieprestatie gebouwen art. 4. */
function labelDataSection(labelData: LabelData | null | undefined): string {
  if (!labelData) return '';
  const envelope = labelData.envelope.map((item) => `<tr>${cell(CATEGORY[item.category] ?? item.category)}<td class="n">${num(item.areaM2, 1)}</td>
    <td class="n">${num(item.meanUWPerM2k, 2)}</td><td class="n">${num(item.minRcM2kPerW, 2)}</td><td class="n">${num(item.maxRcM2kPerW, 2)}</td></tr>`).join('');
  const installations = labelData.installations;
  const list = (values: string[]) => values.length ? values.join(', ') : '—';
  return `<h2>Labelgegevens</h2>
    <table><thead><tr><th>Element</th><th>Oppervlakte [m²]</th><th>Gemiddelde U [W/m²K]</th><th>Laagste R<sub>c</sub></th><th>Hoogste R<sub>c</sub></th></tr></thead><tbody>${envelope || '<tr><td colspan="5">Geen scheidingsconstructies met een thermische grens.</td></tr>'}</tbody></table>
    <table><tbody>
      <tr><th>Verwarming</th>${cell((installations.heatingGenerators?.length ? installations.heatingGenerators.join(', ') : installations.heatingGenerator) ?? '—')}<th>Warm tapwater</th>${cell(installations.hotWaterGenerator ?? '—')}</tr>
      <tr><th>Ventilatie</th>${cell(list(installations.ventilationSystems))}<th>Koeling</th>${cell(list(installations.coolingGenerators))}</tr>
      <tr><th>PV-systemen</th>${cell(installations.pvSystemCount)}<th>Verlichtingszones</th>${cell(installations.lightingZoneCount)}</tr>
      <tr><th>Bron</th><td colspan="3">${escapeHtml(labelData.source)}</td></tr>
    </tbody></table>`;
}

/**
 * Calculation report for the unverified Rust chain. It repeats the kernel's
 * status, fingerprint, omitted corrections and sources so the printed result
 * cannot be mistaken for an attested BENG calculation or a registered label.
 */
export function generateNtaCalculationReportHTML(
  project: IProject,
  assessment: ProjectPerformanceAssessment,
  interpretations?: NtaInterpretationGroup[],
): string {
  const performance = assessment.performance;
  // Project plausibility and the kernel's own warnings (e.g. 10.15, 13.25).
  const warnings: Array<{ code: string; path: string; detail?: string | null }> = [
    ...(assessment.warnings ?? []),
    ...(performance?.warnings ?? []),
  ];
  const generatedAt = new Date().toISOString();
  const head = `<!doctype html><html lang="nl"><head><meta charset="utf-8">
    <title>NTA 8800-rekenrapport — ${escapeHtml(project.name)}</title>
    <style>body{font:14px/1.5 system-ui,sans-serif;max-width:1100px;margin:32px auto;padding:0 20px;color:#202530}
    h1,h2,h3{color:#173c67}table{border-collapse:collapse;width:100%;margin:12px 0 22px}th,td{border:1px solid #cdd5dd;padding:6px 8px;text-align:left;vertical-align:top}
    th{background:#edf3f8}td.n{text-align:right;font-variant-numeric:tabular-nums}.notice{border-left:5px solid #b45309;background:#fff8e7;padding:12px 16px}
    .kpi{display:grid;grid-template-columns:repeat(4,1fr);gap:10px}.kpi div{border:1px solid #cdd5dd;padding:10px}.kpi strong{display:block;font-size:22px}
    @media print{body{margin:12mm;padding:0;font-size:11px}table{break-inside:avoid}}</style></head><body>
    <h1>NTA 8800-rekenrapport</h1>
    <p>Project: ${escapeHtml(project.name)} · Project-ID: ${escapeHtml(project.id)} · Gegenereerd: ${escapeHtml(generatedAt)}</p>
    <div class="notice"><strong>Onverifieerde berekening — geen officieel energielabel, niet geattesteerd.</strong>
    Deze uitkomst komt uit de Rust-rekenkern van Open Energy Studio. De kern is getranscribeerd uit de gelicentieerde normtekst maar nog niet met referentiegevallen geverifieerd; niet-ondersteunde situaties worden afgewezen of als opgegeven waarde vermeld. Een energielabel wordt pas vastgesteld na registratie door een gecertificeerde adviseur met een BRL 9501-geattesteerd rekenprogramma (Omgevingsregeling art. 5.11/5.12).</div>
    <h2>Herleidbaarheid</h2><table><tbody>
      <tr><th>Doeluitgave</th>${cell(assessment.targetNormVersion)}<th>Kernelversie</th>${cell(assessment.kernelVersion)}</tr>
      <tr><th>Status</th>${cell(assessment.status)}<th>Atteststatus</th>${cell(assessment.attestStatus)}</tr>
      <tr><th>Invoervingerafdruk</th><td colspan="3"><code>${escapeHtml(assessment.inputFingerprint)}</code></td></tr>
      ${assessment.geometry ? `<tr><th>A<sub>g</sub> / A<sub>ls</sub></th><td>${num(assessment.geometry.usableFloorAreaM2, 1)} / ${num(assessment.geometry.lossAreaM2, 1)} m²</td>
        <th>A<sub>ls</sub>/A<sub>g</sub></th><td>${num(assessment.geometry.lossAreaRatio, 3)}</td></tr>` : ''}
    </tbody></table>
    ${registrationSection(project.registration, assessment.registration)}
    ${warnings.length > 0 ? `<h2>Plausibiliteit</h2><p>Deze meldingen houden de berekening niet tegen. De invoer of de uitkomst botst met de norm of met andere invoer, of is extreem volgens de letter van de norm.</p>
      <table><thead><tr><th>Code</th><th>Pad</th><th>Detail</th></tr></thead><tbody>${warnings
        .map((warning) => `<tr>${cell(warning.code)}${cell(warning.path)}${cell(warning.detail ?? '')}</tr>`).join('')}</tbody></table>` : ''}`;
  if (!performance || assessment.status !== 'calculated_unverified') {
    const gaps = assessment.gaps.map((gap) => `<tr>${cell(gap.code)}${cell(gap.path)}${cell(gap.detail ?? '')}</tr>`).join('');
    const issues = (performance?.issues ?? []).map((item) => `<tr>${cell(item.code)}${cell(item.path)}<td></td></tr>`).join('');
    return `${head}<h2>Geen uitkomst</h2><p>De rekenkern geeft geen uitkomst. Onderstaande invoergaten of afwijzingen moeten eerst worden opgelost.</p>
      <table><thead><tr><th>Code</th><th>Pad</th><th>Detail</th></tr></thead><tbody>${gaps}${issues}</tbody></table></body></html>`;
  }
  const heating = performance.spaceHeating;
  const zones = [heating.demand, ...(heating.additionalZoneDemands ?? [])];
  const carrierRows = MONTHS.map((month, index) => {
    const byCarrier = (carrier: string, field: 'usedKwh' | 'deliveredKwh') => performance.carriers
      .filter((item) => item.carrier === carrier && item.month === index + 1)
      .reduce((sum, item) => sum + item[field], 0);
    const balance = performance.electricityBalance[index];
    const cooling = zones.reduce((sum, zone) => sum + (zone.monthly[index]?.cooling.needKwh ?? 0), 0);
    return `<tr><th>${month}</th><td class="n">${num(heating.monthly[index]?.heatingNeedKwh)}</td><td class="n">${num(cooling)}</td>
      <td class="n">${num(byCarrier('gas', 'usedKwh'))}</td><td class="n">${num(byCarrier('dh', 'usedKwh'))}</td><td class="n">${num(byCarrier('bm', 'usedKwh'))}</td><td class="n">${num(byCarrier('el', 'usedKwh'))}</td>
      <td class="n">${num(balance?.producedKwh)}</td><td class="n">${num(balance?.selfUsedKwh)}</td><td class="n">${num(balance?.exportedKwh)}</td></tr>`;
  }).join('');
  const zoneRows = zones.map((zone) => `<tr>${cell(zone.monthly.length ? zone.transmission?.method : '—')}
    <td class="n">${num(zone.transmission?.conductanceWPerK, 2)}</td><td class="n">${num(zone.transmission?.groundHeatingAdjustedWPerK, 2)}</td>
    <td class="n">${num(zone.specificHeatCapacityKjPerM2k)}</td><td class="n">${num(zone.annualHeatingNeedKwh)}</td><td class="n">${num(zone.annualCoolingNeedKwh)}</td></tr>`).join('');
  const tojuliRows = performance.tojuli.flatMap((zone) => zone.activeCooling
    ? [`<tr>${cell(zone.zoneId)}${cell('actieve koeling (§5.7.1)')}<td class="n">0,00</td></tr>`]
    : zone.orientations.filter((item) => item.assessed).map((item) =>
      `<tr>${cell(zone.zoneId)}${cell(item.orientation)}<td class="n">${num(item.tojuliK, 2)}</td></tr>`)).join('');
  const bbl = performance.bblCheck;
  const extras = summarizeExtras(performance);
  const ventilationRows = extras.ventilation.map((item) => `<tr>${cell(item.zoneId)}<td class="n">${num(item.requiredJanuaryM3PerH)}</td>
    <td class="n">${num(item.infiltrationJanuaryM3PerH)}</td><td class="n">${num(item.conductanceJanuaryWPerK, 1)}</td>
    <td class="n">${num(item.fanKwh)}</td><td class="n">${num(item.frostProtectionKwh)}</td><td class="n">${num(item.grillePreheatingKwh)}</td></tr>`).join('');
  const beng1Basis = extras.beng1Basis === 'fixed_c1'
    ? `aparte run met vast ventilatiesysteem C1 (§5.4.2), Q<sub>H+C;nd</sub> = ${num(extras.fixedC1NeedKwh)} kWh`
    : extras.beng1Basis === 'confirmed' ? 'opgegeven ventilatie door de adviseur bevestigd als C1' : '—';
  const limits = [...new Set([...zones.flatMap((zone) => zone.omittedCorrections), ...heating.omittedTerms])]
    .map((item) => `<li>${escapeHtml(item)}</li>`).join('');
  return `${head}
    <h2>Indicatoren</h2>
    <div class="kpi">
      <div>BENG 1<strong>${num(performance.needIndicatorKwhPerM2Year, 2)}</strong>${performance.needIndicatorKwhPerM2Year == null ? 'vereist vast ventilatiesysteem C1 (§5.4)' : 'kWh/m²·jr'}</div>
      <div>BENG 2<strong>${num(performance.primaryFossilIndicatorKwhPerM2Year, 2)}</strong>kWh/m²·jr</div>
      <div>BENG 3<strong>${num(performance.renewableSharePercent, 1)}</strong>%</div>
      <div>Labelklasse<strong>${escapeHtml(performance.indicativeLabelClass ?? '—')}</strong>indicatief, niet geregistreerd</div>
    </div>
    <table><tbody>
      <tr><th>Primair fossiel (EPtot)</th><td class="n">${num(performance.annualPrimaryFossilKwh)} kWh</td><th>Hernieuwbaar (EPrenTot)</th><td class="n">${num(performance.annualRenewablePrimaryKwh)} kWh</td></tr>
      <tr><th>Warmte- en koudebehoefte</th><td class="n">${num(performance.annualHeatingAndCoolingNeedKwh)} kWh</td><th>Omgevingswarmte warmtepomp</th><td class="n">${num(performance.annualHeatPumpAmbientHeatKwh)} kWh</td></tr>
      <tr><th>CO<sub>2</sub>-emissie (§5.5.6.1, tabel 5.3)</th><td class="n">${num(performance.annualCo2Kg)} kg/jr</td><th>Per m² gebruiksoppervlakte</th><td class="n">${num(performance.co2KgPerM2, 1)} kg/m²·jr</td></tr>
      <tr><th>Opslagcorrectie (5.14a)</th><td class="n">${num(performance.annualStorageCorrectionKwh)} kWh</td><th>Hulpenergie verwarming</th><td class="n">${num(heating.annualAuxiliaryElectricityKwh)} kWh</td></tr>
      <tr><th>Basis BENG 1</th><td colspan="3">${beng1Basis}</td></tr>
      <tr><th>Terugwinbare systeemverliezen (7.3)</th>${cell(extras.recoverableLossesApplied ? 'verrekend' : 'niet verrekend')}<th>Q<sub>H;ls;rbl</sub></th><td class="n">${num(extras.recoverableLossKwh)} kWh</td></tr>
      ${extras.lightingKwh != null ? `<tr><th>Verlichting (hoofdstuk 14)</th><td class="n">${num(extras.lightingKwh)} kWh</td><th></th><td></td></tr>` : ''}
      <tr><th>Labelbron</th><td colspan="3">${escapeHtml(performance.labelSource)}</td></tr>
      <tr><th>Labelgegevens (Reg. art. 4)</th><td colspan="3">EP2 ${num(performance.primaryFossilIndicatorKwhPerM2Year, 2)} kWh/m²·jr · hernieuwbaar ${num(performance.renewableSharePercent, 1)} % · TO<sub>juli</sub> ${num(performance.tojuliMaxK, 2)} K · ${project.buildingFunction === 'residential' ? 'warmtebehoefte (BENG 1)' : 'energiebehoefte (BENG 1)'} ${num(performance.needIndicatorKwhPerM2Year, 2)} kWh/m²·jr</td></tr>
    </tbody></table>
    ${chapterFiveSection(performance.chapter5)}
    ${serviceEnergySection(performance)}
    ${zebSection(performance)}
    ${labelDataSection(assessment.labelData)}
    ${ventilationRows ? `<h2>Ventilatie (hoofdstuk 11)</h2><table><thead><tr><th>Zone</th><th>q<sub>V;ODA;req</sub> jan [m³/h]</th><th>Infiltratie jan [m³/h]</th>
      <th>H<sub>ve</sub> jan [W/K]</th><th>Ventilatoren [kWh/jr]</th><th>Vorstbeveiliging [kWh/jr]</th><th>Voorverwarming roosters [kWh/jr]</th></tr></thead>
      <tbody>${ventilationRows}</tbody></table>` : ''}
    ${bbl ? `<h2>Toets Bbl art. 4.149 (tabel 4.148A)</h2><table><tbody>
      <tr><th>Gebruiksfunctie</th>${cell(bbl.function)}<th>A<sub>ls</sub>/A<sub>g</sub></th><td class="n">${num(bbl.lossAreaRatio, 2)}</td></tr>
      <tr><th>BENG 1 ≤ ${num(bbl.limits.energyNeedMaxKwhPerM2, 1)}</th>${cell(meets(bbl.energyNeedMeets))}<th>BENG 2 ≤ ${num(bbl.limits.primaryFossilMaxKwhPerM2, 1)}</th>${cell(meets(bbl.primaryFossilMeets))}</tr>
      <tr><th>BENG 3 ≥ ${num(bbl.limits.renewableShareMinPercent)}%</th>${cell(meets(bbl.renewableShareMeets))}<th>Toeslag lichte bouw (lid 4)</th>${cell(bbl.limits.lightConstructionAllowanceApplied ? 'toegepast' : 'niet van toepassing')}</tr>
      <tr><th>Bron</th><td colspan="3">${escapeHtml(bbl.source)}</td></tr>
    </tbody></table>` : ''}
    ${performance.a0Check ? `<h2>Aanduiding A0 (Omgevingsregeling art. 5.11/5.12 lid 5)</h2><table><tbody>
      <tr><th>BENG 2 ≤ ${num(performance.a0Check.primaryFossilMaxKwhPerM2)} (bijlage IXa/Xa)</th>${cell(meets(performance.a0Check.primaryFossilMeets))}<th>Geen fossiele verbranding ter plaatse</th>${cell(meets(performance.a0Check.noOnSiteFossilCombustion))}</tr>
      <tr><th>BENG 1 / BENG 3 (tabel 4.148A)</th>${cell(`${meets(performance.a0Check.energyNeedMeets)} / ${meets(performance.a0Check.renewableShareMeets)}`)}<th>A0 mogelijk</th>${cell(performance.a0Check.eligible == null ? 'niet te toetsen' : performance.a0Check.eligible ? 'ja (onverifieerd)' : 'nee')}</tr>
      <tr><th>Bron</th><td colspan="3">${escapeHtml(performance.a0Check.source)}</td></tr>
    </tbody></table>` : ''}
    ${hotWaterSection(performance)}
    ${coolingSection(performance)}
    ${pvSection(performance)}
    <h2>TO<sub>juli</sub> (§5.7)</h2>
    ${tojuliRows ? `<table><thead><tr><th>Zone</th><th>Oriëntatie</th><th>TO<sub>juli</sub> [K]</th></tr></thead><tbody>${tojuliRows}</tbody></table>
      <p>Hoogste waarde: ${num(performance.tojuliMaxK, 2)} K — Bbl 4.149b (≤ 1,20): ${project.ntaCalculation?.calculationScope === 'utility' ? 'niet van toepassing (alleen woonfunctie)' : meets(performance.tojuliMeetsBblLimit)}.</p>` : '<p>Niet bepaald.</p>'}
    <h2>Rekenzones</h2>
    <table><thead><tr><th>Transmissieroute</th><th>H<sub>tr</sub> [W/K]</th><th>H<sub>H;g;adj</sub> [W/K]</th><th>D<sub>m</sub> [kJ/m²K]</th><th>Q<sub>H;nd</sub> [kWh]</th><th>Q<sub>C;nd</sub> [kWh]</th></tr></thead><tbody>${zoneRows}</tbody></table>
    <h2>Maandoverzicht [kWh]</h2>
    <table><thead><tr><th>Maand</th><th>Q<sub>H;nd</sub></th><th>Q<sub>C;nd</sub></th><th>Gas</th><th>Warmte (dh)</th><th>Biomassa</th><th>Elektriciteit</th><th>Opwekking</th><th>Eigen gebruik</th><th>Export</th></tr></thead><tbody>${carrierRows}</tbody></table>
    <h2>Niet meegenomen</h2><ul>${limits}</ul>
    <h2>Bronnen</h2><ul>
      <li>Klimaat: ${escapeHtml(heating.demand.climateSource)}</li>
      <li>Hoofdstuk 9 (afgifte, opwekking): ${escapeHtml(heating.chapter9Source)}</li>
      <li>Hoofdstuk 5 (primaire energie, indicatoren): ${escapeHtml(performance.chapter5Source)}</li>
      <li>Hoofdstukken 7–17: getranscribeerd uit NTA 8800:2025+C1:2026 (paginaverwijzingen in de kern); verificatie met referentiegevallen nog nodig.</li>
    </ul>
    ${interpretationsSection(interpretations)}</body></html>`;
}
