import type { IProject } from '../energy/types';
import type { ProjectPerformanceAssessment } from '../nta/KernelClient';
import { escapeHtml } from './HtmlEscaping';

const MONTHS = ['jan', 'feb', 'mrt', 'apr', 'mei', 'jun', 'jul', 'aug', 'sep', 'okt', 'nov', 'dec'];

function cell(value: unknown): string { return `<td>${escapeHtml(value)}</td>`; }
function num(value: number | null | undefined, digits = 0): string {
  return value == null || !Number.isFinite(value) ? '—' : value.toFixed(digits);
}
function meets(value: boolean | null | undefined): string {
  return value == null ? 'niet te toetsen' : value ? 'voldoet (onverifieerd)' : 'voldoet niet';
}

/**
 * Calculation report for the unverified Rust chain. It repeats the kernel's
 * status, fingerprint, omitted corrections and sources so the printed result
 * cannot be mistaken for an attested BENG calculation or a registered label.
 */
export function generateNtaCalculationReportHTML(
  project: IProject,
  assessment: ProjectPerformanceAssessment,
): string {
  const performance = assessment.performance;
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
    Deze uitkomst komt uit de Rust-rekenkern van Open Energy Studio. Delen van NTA 8800 zijn gebaseerd op transcripties en het openbare consultatieconcept; ventilatie, koeling en delen van tapwater zijn opgegeven waarden. Een energielabel wordt pas vastgesteld na registratie door een gecertificeerde adviseur met een BRL 9501-geattesteerd rekenprogramma (Omgevingsregeling art. 5.11/5.12).</div>
    <h2>Herleidbaarheid</h2><table><tbody>
      <tr><th>Doeluitgave</th>${cell(assessment.targetNormVersion)}<th>Kernelversie</th>${cell(assessment.kernelVersion)}</tr>
      <tr><th>Status</th>${cell(assessment.status)}<th>Atteststatus</th>${cell(assessment.attestStatus)}</tr>
      <tr><th>Invoervingerafdruk</th><td colspan="3"><code>${escapeHtml(assessment.inputFingerprint)}</code></td></tr>
      ${assessment.geometry ? `<tr><th>A<sub>g</sub> / A<sub>ls</sub></th><td>${num(assessment.geometry.usableFloorAreaM2, 1)} / ${num(assessment.geometry.lossAreaM2, 1)} m²</td>
        <th>A<sub>ls</sub>/A<sub>g</sub></th><td>${num(assessment.geometry.lossAreaRatio, 3)}</td></tr>` : ''}
    </tbody></table>`;
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
    <td class="n">${num(zone.transmission?.conductanceWPerK, 2)}</td><td class="n">${num(zone.transmission?.groundConductanceWPerK, 2)}</td>
    <td class="n">${num(zone.specificHeatCapacityKjPerM2k)}</td><td class="n">${num(zone.annualHeatingNeedKwh)}</td><td class="n">${num(zone.annualCoolingNeedKwh)}</td></tr>`).join('');
  const tojuliRows = performance.tojuli.flatMap((zone) => zone.activeCooling
    ? [`<tr>${cell(zone.zoneId)}${cell('actieve koeling (§5.7.1)')}<td class="n">0,00</td></tr>`]
    : zone.orientations.filter((item) => item.assessed).map((item) =>
      `<tr>${cell(zone.zoneId)}${cell(item.orientation)}<td class="n">${num(item.tojuliK, 2)}</td></tr>`)).join('');
  const bbl = performance.bblCheck;
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
      <tr><th>Labelbron</th><td colspan="3">${escapeHtml(performance.labelSource)}</td></tr>
    </tbody></table>
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
    <h2>TO<sub>juli</sub> (§5.7)</h2>
    ${tojuliRows ? `<table><thead><tr><th>Zone</th><th>Oriëntatie</th><th>TO<sub>juli</sub> [K]</th></tr></thead><tbody>${tojuliRows}</tbody></table>
      <p>Hoogste waarde: ${num(performance.tojuliMaxK, 2)} K — Bbl 4.149b (≤ 1,20): ${meets(performance.tojuliMeetsBblLimit)}.</p>` : '<p>Niet bepaald.</p>'}
    <h2>Rekenzones</h2>
    <table><thead><tr><th>Transmissieroute</th><th>H<sub>tr</sub> [W/K]</th><th>H<sub>g</sub> [W/K]</th><th>D<sub>m</sub> [kJ/m²K]</th><th>Q<sub>H;nd</sub> [kWh]</th><th>Q<sub>C;nd</sub> [kWh]</th></tr></thead><tbody>${zoneRows}</tbody></table>
    <h2>Maandoverzicht [kWh]</h2>
    <table><thead><tr><th>Maand</th><th>Q<sub>H;nd</sub></th><th>Q<sub>C;nd</sub></th><th>Gas</th><th>Warmte (dh)</th><th>Biomassa</th><th>Elektriciteit</th><th>Opwekking</th><th>Eigen gebruik</th><th>Export</th></tr></thead><tbody>${carrierRows}</tbody></table>
    <h2>Niet meegenomen</h2><ul>${limits}</ul>
    <h2>Bronnen</h2><ul>
      <li>Klimaat: ${escapeHtml(heating.demand.climateSource)}</li>
      <li>Hoofdstuk 9 (afgifte, opwekking): ${escapeHtml(heating.chapter9Source)}</li>
      <li>Hoofdstuk 5 (primaire energie, indicatoren): ${escapeHtml(performance.chapter5Source)}</li>
      <li>Hoofdstukken 7, 8, 13 en 16: transcripties uit de normanalyses van Open Heatloss Studio; review tegen de normtekst nog nodig.</li>
    </ul></body></html>`;
}
