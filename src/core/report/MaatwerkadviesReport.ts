import type { IProject } from '../energy/types';
import type { MaatwerkadviesAssessment, MwaVariantResult, NtaMaatwerkadvies } from '../nta/KernelClient';
import { escapeHtml } from './HtmlEscaping';
import { dutchCodeCell, dutchNumber, dutchTimeHtml } from './DutchReportText';
import { measureEvidenceNotes } from '../nta/MwaTemplates';

function cell(value: unknown): string { return `<td>${escapeHtml(value)}</td>`; }
function num(value: number | null | undefined, digits = 0): string {
  return dutchNumber(value, digits);
}
function n(value: number | null | undefined, digits = 0): string { return `<td class="n">${num(value, digits)}</td>`; }

const SYSTEM: Record<string, string> = {
  space_heating: 'Ruimteverwarming',
  space_cooling: 'Ruimtekoeling',
  hot_water: 'Warm tapwater',
  ventilation: 'Ventilatie',
  lighting: 'Ingebouwde verlichting',
};

const REQUIREMENT: Record<string, string> = {
  three_steps_present: 'Drie stappen aanwezig (beperken vraag, duurzame installaties, opwekking)',
  insulation_standard: 'Standaard voor Woningisolatie gehaald (stap 1)',
  prewar_standard_motivated: 'Vooroorlogse standaard gemotiveerd',
  overheating_measures: 'Maatregelen tegen oververhitting in stap 1',
  natural_gas_free_main_heating: 'Hoofdverwarming aardgasvrij (stap 2)',
  emission_free_result: 'Geen verbranding van fossiele brandstof op het perceel (stap 3)',
  renewable_production: 'Hernieuwbare opwekking toegevoegd (stap 3)',
  storage_considered: 'Opslagcapaciteit afgewogen (stap 3)',
};

function verdict(value: boolean | null | undefined): string {
  return value == null ? 'niet te beoordelen' : value ? 'voldoet' : 'voldoet niet';
}

const PROFILE: Record<string, string> = {
  nta: 'NTA 8800',
  energy_conscious: 'energiebewust',
  average: 'gemiddeld',
  not_energy_conscious: 'niet energiebewust',
};

function variantRow(result: MwaVariantResult): string {
  const use = result.actualUse;
  const savings = result.savings;
  return `<tr>${cell(result.name)}${cell(result.label.labelClass ?? '—')}${n(result.label.primaryFossilIndicatorKwhPerM2, 1)}${n(result.label.tojuliMaxK, 2)}
    ${n(use?.gasM3)}${n(use ? use.electricityImportKwh - use.electricityExportKwh : null)}${n(use?.districtHeatKwh)}${n(use?.co2Kg)}${n(use?.energyCostEur)}
    ${n(savings?.energyCostEur)}${n(result.investmentEur)}${n(result.simplePaybackYears, 1)}${n(result.netPresentValueEur)}</tr>`;
}

/** BRL 9500-MWA-W/U §3.1 report: current use, packages, savings, costs and advice. */
export function generateMaatwerkadviesReportHTML(
  project: IProject,
  definition: NtaMaatwerkadvies,
  assessment: MaatwerkadviesAssessment,
): string {
  const generatedTime = dutchTimeHtml();
  const head = `<!doctype html><html lang="nl"><head><meta charset="utf-8">
    <title>Maatwerkadvies — ${escapeHtml(project.name)}</title>
    <style>body{font:14px/1.5 system-ui,sans-serif;max-width:1100px;margin:32px auto;padding:0 20px;color:#202530}
    h1,h2,h3{color:#173c67}table{border-collapse:collapse;width:100%;margin:12px 0 22px}th,td{border:1px solid #cdd5dd;padding:6px 8px;text-align:left;vertical-align:top}
    th{background:#edf3f8}td.n{text-align:right;font-variant-numeric:tabular-nums}.notice{border-left:5px solid #b45309;background:#fff8e7;padding:12px 16px}
    @media print{body{margin:12mm;padding:0;font-size:11px}table{break-inside:avoid}}</style></head><body>
    <h1>Maatwerkadvies</h1>
    <p>Project: ${escapeHtml(project.name)} · ${escapeHtml(project.address ?? '')} ${escapeHtml(project.city ?? '')} · Gegenereerd: ${generatedTime}</p>
    <div class="notice"><strong>Onverifieerde berekening, niet geattesteerd.</strong> De energieberekening komt uit de Rust-rekenkern van Open Energy Studio (NTA 8800:2025+C1:2026). Werkelijke besparingen hangen af van gebruik, uitvoering en energieprijzen. TO<sub>juli</sub> is slechts een indicatie en is voor bestaande bouw niet gevalideerd (ISSO 82.2 §4.2.3).</div>
    <h2>Herleidbaarheid</h2><table><tbody>
      <tr><th>Doeluitgave</th>${cell(assessment.targetNormVersion)}<th>Kernelversie</th>${cell(assessment.kernelVersion)}</tr>
      <tr><th>Status</th>${cell(assessment.status)}<th>Atteststatus</th>${cell(assessment.attestStatus)}</tr>
      <tr><th>Invoervingerafdruk</th><td colspan="3"><code>${escapeHtml(assessment.inputFingerprint)}</code></td></tr>
    </tbody></table>`;
  if (assessment.status === 'invalid' || !assessment.current) {
    const issues = assessment.issues.map((item) => `<tr>${dutchCodeCell(item.code)}${cell(item.path)}${cell(item.detail ?? '')}</tr>`).join('');
    return `${head}<h2>Geen uitkomst</h2><table><thead><tr><th>Code</th><th>Pad</th><th>Detail</th></tr></thead><tbody>${issues}</tbody></table></body></html>`;
  }
  const current = assessment.current;
  const use = definition.currentUse;
  const future = definition.futureUse;
  const tariffs = definition.tariffs;
  const fit = assessment.fitCheck;
  const advice = assessment.advice;
  const evidenceNote = (measure: (typeof definition.measures)[number]) => measureEvidenceNotes(measure)
    .map((note) => `<br><small>PV ${escapeHtml(note.id)}: belemmeringssituatie a) (minimaal, NTA 8800 tabel 17.3) — onderbouwing: ${escapeHtml(note.source)}</small>`).join('');
  const measureRows = definition.measures.map((measure) => `<tr><td>${escapeHtml(measure.name)}${evidenceNote(measure)}</td>${cell(measure.category)}${n(measure.investmentEur)}${cell(measure.costSource)}
    ${n(measure.lifetimeYears)}${n(measure.maintenanceEurPerYear ?? 0)}${cell(measure.phaseYear ?? '—')}</tr>`).join('');
  const packageRows = definition.packages.map((item) => {
    const names = item.measureIds.map((id) => definition.measures.find((m) => m.id === id)?.name ?? id).join(', ');
    return `<tr>${cell(item.name)}${cell(names)}</tr>`;
  }).join('');
  const savingsRows = assessment.packages.map((result) => {
    const s = result.savings;
    return `<tr>${cell(result.name)}${n(s?.gasM3)}${n(s?.electricityKwh)}${n(s?.heatKwh)}${n(s?.co2Kg)}${n(s?.primaryFossilKwh)}${n(s?.energyCostEur)}</tr>`;
  }).join('');
  const phasingRows = assessment.packages.flatMap((result) => result.phasing.map((step) =>
    `<tr>${cell(result.name)}${cell(step.year ?? 'direct')}${cell(step.measureIds.map((id) => definition.measures.find((m) => m.id === id)?.name ?? id).join(', '))}</tr>`)).join('');
  const chosen = assessment.packages.find((item) => item.id === advice?.packageId);
  const passport = assessment.renovationPassport;
  const list = (items: string[]) => items.length ? `<ul>${items.map((item) => `<li>${escapeHtml(item)}</li>`).join('')}</ul>` : '<p>Geen.</p>';
  return `${head}
    <h2>Huidige situatie</h2>
    <table><tbody>
      <tr><th>Energielabel (indicatief, NTA 8800)</th>${cell(current.label.labelClass ?? '—')}<th>EP2 [kWh/m²·jr]</th>${n(current.label.primaryFossilIndicatorKwhPerM2, 1)}</tr>
      <tr><th>Gebruikersprofiel nu</th>${cell(use ? `${PROFILE[use.profile] ?? use.profile} — ${use.sourceReference}` : 'NTA 8800')}<th>Profiel na maatregelen</th>${cell(future ? `${PROFILE[future.profile] ?? future.profile} — ${future.sourceReference}` : 'gelijk aan nu')}</tr>
      <tr><th>Gas [m³/jr]</th>${n(current.actualUse?.gasM3)}<th>Elektriciteit netto [kWh/jr]</th>${n(current.actualUse ? current.actualUse.electricityImportKwh - current.actualUse.electricityExportKwh : null)}</tr>
      <tr><th>CO<sub>2</sub> [kg/jr]</th>${n(current.actualUse?.co2Kg)}<th>Energiekosten [€/jr]</th>${n(current.actualUse?.energyCostEur)}</tr>
    </tbody></table>
    ${fit ? `<h3>Vergelijking met gemeten gebruik (fitprocedure, ISSO 82.2/75.2 hoofdstuk 3)</h3><table><tbody>
      <tr><th>Afwijking gas</th>${n(fit.gasDeviationPercent, 1)}<th>Afwijking elektriciteit</th>${n(fit.electricityDeviationPercent, 1)}</tr>
      <tr><th>Gemeten helling gas [m³/K] / stookgrens [°C]</th><td>${num(fit.measuredGasLine?.slopeM3PerK, 2)} / ${num(fit.measuredGasLine?.heatingLimitC, 1)}</td>
        <th>Berekende helling [m³/K] / stookgrens [°C]</th><td>${num(fit.calculatedGasLine?.slopeM3PerK, 2)} / ${num(fit.calculatedGasLine?.heatingLimitC, 1)}</td></tr>
      <tr><th>Basislast gas gemeten / berekend [m³/mnd]</th><td>${num(fit.measuredGasBaseLoad, 1)} / ${num(fit.calculatedGasBaseLoad, 1)}</td>
        <th>Afwijking warmte</th>${n(fit.heatDeviationPercent, 1)}</tr>
      <tr><th>Gemeten helling elektriciteit [kWh/K] / stookgrens [°C]</th><td>${num(fit.measuredElectricityLine?.slopeM3PerK, 1)} / ${num(fit.measuredElectricityLine?.heatingLimitC, 1)}</td>
        <th>Berekende helling elektriciteit [kWh/K] / stookgrens [°C]</th><td>${num(fit.calculatedElectricityLine?.slopeM3PerK, 1)} / ${num(fit.calculatedElectricityLine?.heatingLimitC, 1)}</td></tr>
      <tr><th>Fitcriteria ISSO 82.2 bijlage C.1 (jaar ±5 %, helling ±5 %, stookgrens ±1 °C, basislijn ±5 %)</th><td colspan="3">${escapeHtml(verdict(fit.criteria.withinCriteria))}
        (jaar gas ${escapeHtml(verdict(fit.criteria.annualGas))}, elektriciteit ${escapeHtml(verdict(fit.criteria.annualElectricity))}, warmte ${escapeHtml(verdict(fit.criteria.annualHeat))};
        helling gas ${escapeHtml(verdict(fit.criteria.gasSlope))}, stookgrens gas ${escapeHtml(verdict(fit.criteria.gasHeatingLimit))}, basislijn gas ${escapeHtml(verdict(fit.criteria.gasBaseLine))})</td></tr>
    </tbody></table>` : ''}
    <h2>Maatregelen</h2>
    <table><thead><tr><th>Maatregel</th><th>Soort</th><th>Investering [€]</th><th>Kostenbron</th><th>Levensduur [jr]</th><th>Onderhoud [€/jr]</th><th>Fasering</th></tr></thead><tbody>${measureRows}</tbody></table>
    <h2>Maatregelpakketten</h2>
    <table><thead><tr><th>Pakket</th><th>Maatregelen</th></tr></thead><tbody>${packageRows}</tbody></table>
    <h2>Resultaten</h2>
    <table><thead><tr><th>Variant</th><th>Label</th><th>EP2</th><th>TO<sub>juli</sub></th><th>Gas [m³]</th><th>Elektr. netto [kWh]</th><th>Warmte [kWh]</th><th>CO<sub>2</sub> [kg]</th>
      <th>Kosten [€/jr]</th><th>Besparing [€/jr]</th><th>Investering [€]</th><th>Terugverdientijd [jr]</th><th>NCW [€]</th></tr></thead>
      <tbody>${variantRow(current)}${assessment.measures.map(variantRow).join('')}${assessment.packages.map(variantRow).join('')}</tbody></table>
    <h3>Besparing per pakket (jaarlijks)</h3>
    <table><thead><tr><th>Pakket</th><th>Gas [m³]</th><th>Elektriciteit [kWh]</th><th>Warmte [kWh]</th><th>CO<sub>2</sub> [kg]</th><th>Primair fossiel [kWh]</th><th>Energiekosten [€]</th></tr></thead><tbody>${savingsRows}</tbody></table>
    ${phasingRows ? `<h3>Fasering</h3><table><thead><tr><th>Pakket</th><th>Jaar</th><th>Maatregelen</th></tr></thead><tbody>${phasingRows}</tbody></table>` : ''}
    ${chosen && chosen.systemChecks?.length ? `<h3>Systeemeisen Bbl art. 4.248 voor het geadviseerde pakket (ISSO 82.2 §5.2)</h3>
    <table><thead><tr><th>Systeem</th><th>Waarde</th><th>Eis</th><th>Eenheid</th><th>Oordeel</th><th>Toelichting</th></tr></thead><tbody>
    ${chosen.systemChecks.map((check) => `<tr>${cell(SYSTEM[check.system] ?? check.system)}${n(check.value, 2)}${n(check.limit, 2)}${cell(check.unit)}${cell(check.limit == null ? 'geen eis' : verdict(check.meets))}${cell(check.note ?? '')}</tr>`).join('')}
    </tbody></table>
    <p>De systeemeisen gelden wanneer de opwekker, de ventilatie-unit of een derde van de afgiftelichamen of armaturen wordt geïnstalleerd, vervangen of verbeterd.</p>` : ''}
    ${passport ? `<h2>Renovatiepaspoort (ISSO 82.2 §1.10 en §4.4)</h2>
    <p><strong>Voldoet aan de eisen:</strong> ${escapeHtml(verdict(passport.eligible))}</p>
    <table><thead><tr><th>Stap</th><th>Label (NTA 8800)</th><th>EP2</th><th>Gas [m³]</th><th>Elektr. netto [kWh]</th><th>CO<sub>2</sub> [kg]</th><th>Energiekosten [€/jr]</th></tr></thead><tbody>
    ${passport.steps.map((step) => `<tr>${cell(step.name)}${cell(step.label.labelClass ?? '—')}${n(step.label.primaryFossilIndicatorKwhPerM2, 1)}${n(step.actualUse?.gasM3)}${n(step.actualUse ? step.actualUse.electricityImportKwh - step.actualUse.electricityExportKwh : null)}${n(step.actualUse?.co2Kg)}${n(step.actualUse?.energyCostEur)}</tr>`).join('')}
    </tbody></table>
    <p>De labels zijn gestapeld: stap 2 bevat de maatregelen van stap 1, stap 3 die van stap 1 en 2. Ze volgen de standaard NTA 8800-berekening en kunnen afwijken van het energiegebruik op het paspoort.</p>
    <table><thead><tr><th>Eis</th><th>Oordeel</th><th>Toelichting</th></tr></thead><tbody>
    ${passport.requirements.map((item) => `<tr>${cell(REQUIREMENT[item.code] ?? item.code)}${cell(verdict(item.met))}${cell(item.detail ?? '')}</tr>`).join('')}
    </tbody></table>
    <h3>Over oververhitting</h3><ul>
      <li>TO<sub>juli</sub>, GTO en ATG geven alleen een indicatie en zijn beperkt voor bestaande bouw.</li>
      <li>Let op risico's die de berekening niet laat zien, zoals de noodzaak van zonwering of ventilatie.</li>
      <li>De berekening gebruikt verouderde klimaatgegevens en geeft geen garantie voor het werkelijke risico.</li>
      <li>Het gebouw kan in de praktijk gevoeliger zijn voor oververhitting; aanvullende (dynamische) studies kunnen zinvol zijn.</li>
    </ul>` : ''}
    <h2>Advies</h2>
    <p><strong>Best passend pakket:</strong> ${escapeHtml(chosen?.name ?? '—')} (${advice?.chosenBy === 'adviser' ? 'keuze adviseur' : 'automatisch: hoogste netto contante waarde, door adviseur te bevestigen'})</p>
    ${advice?.motivation ? `<p>${escapeHtml(advice.motivation)}</p>` : ''}
    <h3>Waarschuwingen</h3>${list(advice?.warnings ?? [])}
    <h3>Uitwerking door een specialist</h3>${list(advice?.specialistNotes ?? [])}
    ${advice?.notes.length ? `<h3>Opmerkingen</h3>${list(advice.notes)}` : ''}
    <h2>Uitgangspunten</h2><table><tbody>
      <tr><th>Gastarief</th><td>€ ${num(tariffs.gasEurPerM3, 3)}/m³</td><th>Elektriciteit levering / teruglevering</th><td>€ ${num(tariffs.electricityEurPerKwh, 3)} / € ${num(tariffs.electricityExportEurPerKwh ?? 0, 3)} per kWh</td></tr>
      <tr><th>Warmte</th><td>€ ${num(tariffs.districtHeatEurPerKwh ?? 0, 3)}/kWh</td><th>Bron tarieven</th>${cell(tariffs.sourceReference)}</tr>
      <tr><th>Discontovoet / prijsstijging</th><td>${num((definition.economics?.discountRate ?? 0.03) * 100, 1)} % / ${num((definition.economics?.energyPriceChange ?? 0) * 100, 1)} %</td><th>Bron</th>${cell(definition.economics?.sourceReference ?? '—')}</tr>
    </tbody></table>
    <h3>Interpretaties</h3>${list(assessment.interpretations)}
    </body></html>`;
}
