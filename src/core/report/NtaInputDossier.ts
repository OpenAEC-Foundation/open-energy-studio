import type { INtaHeatPumpInput, IProject } from '../energy/types';
import { escapeHtml } from './HtmlEscaping';

type PumpEntry = { context: string; pump: INtaHeatPumpInput };

function allHeatPumps(project: IProject): PumpEntry[] {
  return [
    ...project.heatingSystems.flatMap((system) => system.ntaHeatPump
      ? [{ context: `Verwarming: ${system.name || system.id}`, pump: system.ntaHeatPump }] : []),
    ...project.hotWaterSystems.flatMap((system) => system.ntaHeatPump
      ? [{ context: `Warm tapwater: ${system.name || system.id}`, pump: system.ntaHeatPump }] : []),
    ...(project.ntaHeatPumps ?? []).map((pump) => ({ context: 'Los toestel', pump })),
  ];
}

function cell(value: unknown): string { return `<td>${escapeHtml(value)}</td>`; }

function pumpHtml({ context, pump }: PumpEntry): string {
  const evidence = pump.performanceEvidence;
  const record = evidence.registryRecord;
  const pointRows = (pump.performancePoints ?? []).map((point) => `<tr>
    ${cell(point.id)}${cell(point.service)}${cell(point.sourceTemperatureC)}${cell(point.sinkTemperatureC)}
    ${cell(point.usefulCapacityKw)}${cell(point.inputPowerKw)}${cell(point.inputEnergyCarrier)}${cell(point.testReference)}
  </tr>`).join('');
  const dhwRows = (pump.dhwTestPoints ?? []).map((point) => `<tr>
    ${cell(point.tapProfile)}${cell(point.usefulEnergyKwhPerDay)}${cell(point.inputEnergyKwhPerDay)}
    ${cell(point.nominalCapacityKw)}${cell(point.practiceFactor)}${cell(point.testSetpointC)}
    ${cell(point.designSetpointC)}${cell(point.sourceAirFlowM3PerHour ?? '—')}
    ${cell(point.sourceAirDryBulbC ?? '—')}${cell(point.sourceAirWetBulbC ?? '—')}
    ${cell(point.declarationNormVersion)}${cell(point.sourceReference)}
  </tr>`).join('');
  const auxiliaryRows = (pump.auxiliaryComponents ?? []).map((item) => `<tr>
    ${cell(item.id)}${cell(item.kind)}${cell(item.service)}${cell(item.nominalPowerW)}
    ${cell(item.energyCarrier)}${cell(item.measurementBoundary)}${cell(item.evidenceReference)}
  </tr>`).join('');
  const linkRows = (pump.systemLinks ?? []).map((link) => `<tr>
    ${cell(link.id)}${cell(link.role)}${cell(link.targetKind)}${cell(link.targetId)}${cell(link.evidenceReference)}
  </tr>`).join('');
  const measured = pump.heatingAuxMeasuredDraft;
  const measuredRows = measured ? `<table><tbody>
    <tr><th>Generatorbron</th>${cell(measured.generatorSourceReference)}<th>Meetbron</th>${cell(measured.measurements.measurementSourceReference)}</tr>
    <tr><th>Schakelbron</th>${cell(measured.measurements.timingSourceReference)}<th>Maandenergiebron</th>${cell(measured.inputEnergySourceReference)}</tr>
    <tr><th>Stand-by-elektronica W</th>${cell(measured.measurements.standbyElectronicsW)}<th>Afgiftepomp tijdens compressor W</th>${cell(measured.measurements.deliveryPumpDuringCompressorW)}</tr>
    <tr><th>Afgiftepomp voor-/naloop W</th>${cell(measured.measurements.deliveryPumpPrePostW)}<th>Voor-/naloop s</th>${cell(`${measured.measurements.pumpPreRunSeconds} / ${measured.measurements.pumpPostRunSeconds}`)}</tr>
    <tr><th>Gemiddelde compressortijd s</th>${cell(measured.measurements.averageCompressorOnSeconds)}<th>Gemiddelde modulatie</th>${cell(measured.measurements.meanCompressorModulation)}</tr>
    <tr><th>Nominaal elektrisch ingangsvermogen kW</th>${cell(measured.measurements.nominalElectricDriveKw)}<th>Maandwaarden kWh</th>${cell(measured.months.map((month) => `${month.month}: ${month.generatorInputElectricityKwh}`).join('; '))}</tr>
  </tbody></table>` : '<p>Geen gemeten conceptinvoer vastgelegd.</p>';
  const forfait = pump.forfaitHeatPumpDraft;
  const forfaitRows = forfait ? `<table><tbody>
    <tr><th>Tabelklasse</th>${cell(forfait.scope)}<th>Bronklasse</th>${cell(forfait.source)}</tr>
    <tr><th>Brontemperatuur °C</th>${cell(forfait.sourceTemperatureC ?? '—')}<th>Bewijs brontemperatuur</th>${cell(forfait.sourceTemperatureEvidenceReference ?? '—')}</tr>
    <tr><th>Kwaliteitsverklaring bron</th>${cell(forfait.sourceQualityDeclarationReference ?? '—')}<th>Verklaring gecontroleerd</th>${cell('Nee')}</tr>
    <tr><th>Tabelrij</th>${cell(forfait.rowVariant ?? 'base')}<th>Testnorm</th>${cell(forfait.highEfficiencyEvidence?.testStandardEdition ?? '—')}</tr>
    <tr><th>Afgifte</th>${cell(forfait.sink)}<th>Ontwerpaanvoer °C</th>${cell(forfait.designSupplyTemperatureC ?? '—')}</tr>
    <tr><th>Bewijs classificatie</th>${cell(forfait.classificationSourceReference)}<th>Broncorrectie csource</th>${cell(forfait.sourceCorrectionFactor ?? '—')}</tr>
    <tr><th>Thermisch vermogen kW</th>${cell(forfait.thermalCapacityKw ?? '—')}<th>Bewijs vermogen</th>${cell(forfait.capacitySourceReference ?? '—')}</tr>
    <tr><th>Collectieve gebouwinstallatie</th>${cell(forfait.collectiveBuildingInstallation == null ? 'Niet vastgelegd' : forfait.collectiveBuildingInstallation ? 'Ja' : 'Nee')}<th>Toepasselijkheid</th>${cell('Ongeverifieerd')}</tr>
    <tr><th>Bewijs broncorrectie</th>${cell(forfait.sourceCorrectionReference ?? '—')}<th>Generator-ID</th>${cell(forfait.generatorId)}</tr>
    <tr><th>Beproefd product</th>${cell(forfait.highEfficiencyEvidence?.productReference ?? '—')}<th>Testrapport</th>${cell(forfait.highEfficiencyEvidence?.testReportReference ?? '—')}</tr>
    <tr><th>Testcondities en COP</th>${cell(forfait.highEfficiencyEvidence?.points.map((point) => `${point.condition}: ${point.measuredCop}`).join('; ') ?? '—')}<th>Verificatie</th>${cell('Niet onafhankelijk gecontroleerd')}</tr>
  </tbody></table>` : '<p>Geen forfaitaire COP-conceptinvoer vastgelegd.</p>';
  const gasDraft = pump.gasHeatPumpForfaitDraft;
  const gasRows = gasDraft ? `<table><tbody>
    <tr><th>Aandrijving</th>${cell(gasDraft.drive)}<th>Toepassing tabel 9.29</th>${cell(gasDraft.application)}</tr>
    <tr><th>Bewijs toepassing</th>${cell(gasDraft.applicationReference)}<th>Collectieve gebouwinstallatie</th>${cell(gasDraft.collectiveBuildingInstallation ? 'Ja' : 'Nee')}</tr>
    <tr><th>Externe bronwarmte</th>${cell(gasDraft.externalHeatSupply ? 'Ja' : 'Nee')}<th>Thermisch vermogen kW</th>${cell(gasDraft.thermalCapacityKw)}</tr>
    <tr><th>Bewijs vermogen</th>${cell(gasDraft.capacityReference)}<th>Bronklasse</th>${cell(gasDraft.source)}</tr>
    <tr><th>Bewijs bron</th>${cell(gasDraft.sourceReference)}<th>Ontwerpaanvoer °C</th>${cell(gasDraft.designSupplyTemperatureC)}</tr>
    <tr><th>Bewijs aanvoer</th>${cell(gasDraft.designSupplyReference)}<th>Generator-ID</th>${cell(gasDraft.generatorId)}</tr>
    <tr><th>Aangeleverde broncorrectie csource</th>${cell(gasDraft.sourceCorrectionFactor ?? '—')}<th>Bewijs broncorrectie</th>${cell(gasDraft.sourceCorrectionReference ?? '—')}</tr>
  </tbody></table>` : '<p>Geen gaswarmtepomp-tabelinvoer vastgelegd.</p>';
  const gasAux = pump.gasHeatPumpAuxDraft;
  const gasAuxRows = gasAux ? `<table><tbody>
    <tr><th>Generator-ID</th>${cell(gasAux.generatorId)}<th>Aandrijving</th>${cell(gasAux.drive)}</tr>
    <tr><th>Thermisch vermogen kW</th>${cell(gasAux.nominalThermalCapacityKw)}<th>Bewijs vermogen</th>${cell(gasAux.capacityReference)}</tr>
    <tr><th>Stand-by W</th>${cell(gasAux.standbyElectronicsW)}<th>Brander W/kW</th>${cell(gasAux.burnerAuxiliaryWPerKw)}</tr>
    <tr><th>Oplossingspomp W/kW</th>${cell(gasAux.solutionPumpWPerKw)}<th>Bewijs coëfficiënten</th>${cell(gasAux.coefficientsReference)}</tr>
    <tr><th>Modulatie</th>${cell(gasAux.meanModulation)}<th>Bewijs modulatie</th>${cell(gasAux.modulationReference)}</tr>
    <tr><th>Gebouwdeel</th>${cell(gasAux.buildingShare)}<th>Bewijs gebouwdeel</th>${cell(gasAux.buildingShareReference)}</tr>
    <tr><th>Forfaitaire COP</th>${cell(gasAux.forfaitCopUsed ? 'Ja' : 'Nee')}<th>Bewijs maanduren</th>${cell(gasAux.monthHoursReference)}</tr>
    <tr><th>Bewijs generatorwarmte</th>${cell(gasAux.generatorOutputReference)}<th>Maandinvoer (h; kWh)</th>${cell(gasAux.months.map((month) => `${month.month}: ${month.hours}; ${month.generatorOutputKwh}`).join(' | '))}</tr>
  </tbody></table>` : '<p>Geen gaswarmtepomp-hulpstroominvoer vastgelegd.</p>';
  return `<section class="pump">
    <h3>${escapeHtml(context)} — ${escapeHtml(pump.id)}</h3>
    <table><tbody>
      <tr><th>Bron</th>${cell(pump.source)}<th>Afgifte</th>${cell(pump.sink)}</tr>
      <tr><th>Aandrijving</th>${cell(pump.drive)}<th>Bediende zones</th>${cell(pump.servedZoneIds?.join(', ') || 'Niet vastgelegd')}</tr>
      <tr><th>Reversibel</th>${cell(pump.reversible ? 'Ja' : 'Nee')}<th>Hybride / booster</th>${cell(`${pump.hybrid ? 'Ja' : 'Nee'} / ${pump.booster ? 'Ja' : 'Nee'}`)}</tr>
      <tr><th>Prestatiebewijs</th>${cell(evidence.kind)}<th>Referentie</th>${cell(evidence.reference || 'Niet vastgelegd')}</tr>
    </tbody></table>
    <h4>Registergegevens — door gebruiker ingevoerd, niet geverifieerd</h4>
    ${record ? `<table><tbody>
      <tr><th>Registratienummer</th>${cell(record.registrationNumber)}<th>Product/combinatie</th>${cell(record.productName)}</tr>
      <tr><th>Fabrikant/leverancier</th>${cell(record.manufacturer)}<th>Bronlink</th>${cell(record.sourceUrl)}</tr>
    </tbody></table>` : '<p>Geen registerrecord vastgelegd.</p>'}
    <h4>Prestatiepunten</h4>
    <table><thead><tr><th>ID</th><th>Dienst</th><th>Bron °C</th><th>Afgifte °C</th><th>Nuttig kW</th><th>Ingaand kW</th><th>Drager</th><th>Testbron</th></tr></thead><tbody>
      ${pointRows || '<tr><td colspan="8">Geen prestatiepunten vastgelegd.</td></tr>'}
    </tbody></table>
    <h4>Tapwater-testprofielen — overgenomen invoer, geen normatieve jaarprestatie</h4>
    <table><thead><tr><th>Profiel</th><th>Nuttig kWh/d</th><th>Ingaand kWh/d</th><th>Nominaal kW</th><th>Praktijkfactor</th><th>Testsetpunt °C</th><th>Ontwerpsetpunt °C</th><th>Brondebiet m³/h</th><th>Bron droog °C</th><th>Bron nat °C</th><th>Normeditie</th><th>Bron</th></tr></thead><tbody>
      ${dhwRows || '<tr><td colspan="12">Geen tapwater-testprofielen vastgelegd.</td></tr>'}
    </tbody></table>
    <h4>Verklaarde afschakelgrenzen — geen schakelberekening</h4>
    ${pump.declaredOperatingLimits ? `<table><tbody>
      <tr><th>Minimale bedrijfs-COP</th>${cell(pump.declaredOperatingLimits.minimumOperatingCop ?? '—')}
        <th>Maximale aanvoer °C</th>${cell(pump.declaredOperatingLimits.maximumSupplyTemperatureC ?? '—')}</tr>
      <tr><th>Normeditie</th>${cell(pump.declaredOperatingLimits.declarationNormVersion)}
        <th>Bron</th>${cell(pump.declaredOperatingLimits.sourceReference)}</tr>
    </tbody></table>` : '<p>Geen afschakelgrenzen vastgelegd.</p>'}
    <h4>Hulpcomponenten</h4>
    <table><thead><tr><th>ID</th><th>Type</th><th>Dienst</th><th>Nominaal W</th><th>Drager</th><th>Meetgrens</th><th>Bron</th></tr></thead><tbody>
      ${auxiliaryRows || '<tr><td colspan="7">Geen hulpcomponenten vastgelegd.</td></tr>'}
    </tbody></table>
    <h4>Gemeten hulpenergie-invoer voor conceptdiagnose — geen geverifieerde uitkomst</h4>
    ${measuredRows}
    <h4>Forfaitaire COP-tabelinvoer uit consultatieconcept — geen geverifieerde COP</h4>
    ${forfaitRows}
    <h4>Gasmotor-/gasabsorptie-tabelinvoer — geen gasgebruik of geverifieerde COP</h4>
    ${gasRows}
    <h4>Gaswarmtepomp-toestelhulpstroominvoer — geen geverifieerde jaarprestatie</h4>
    ${gasAuxRows}
    <h4>Systeemkoppelingen</h4>
    <table><thead><tr><th>ID</th><th>Rol</th><th>Doeltype</th><th>Doel-ID</th><th>Bron</th></tr></thead><tbody>
      ${linkRows || '<tr><td colspan="5">Geen systeemkoppelingen vastgelegd.</td></tr>'}
    </tbody></table>
  </section>`;
}

type SourceRow = { path: string; source: string; values: string };

/** Every object in the NTA block that carries a source reference, with its own scalar values. */
function ntaSourceRows(value: unknown, path = 'ntaCalculation'): SourceRow[] {
  if (Array.isArray(value)) return value.flatMap((item, index) => ntaSourceRows(item, `${path}[${index}]`));
  if (!value || typeof value !== 'object') return [];
  const entries = Object.entries(value as Record<string, unknown>);
  const own = entries.filter(([key]) => /Reference$/.test(key));
  const scalars = entries
    .filter(([key, item]) => !/Reference$/.test(key) && (item === null || typeof item !== 'object'))
    .map(([key, item]) => `${key}: ${item === null ? '—' : String(item)}`);
  const rows = own.map(([key, source]) => ({
    path: `${path}.${key}`,
    source: typeof source === 'string' ? source : '',
    values: scalars.join('; '),
  }));
  return [...rows, ...entries
    .filter(([, item]) => item && typeof item === 'object')
    .flatMap(([key, item]) => ntaSourceRows(item, `${path}.${key}`))];
}

/** Standalone input/provenance dossier; deliberately has no energy result. */
export function generateNtaInputDossierHTML(project: IProject): string {
  const pumps = allHeatPumps(project);
  const generatedAt = new Date().toISOString();
  return `<!doctype html><html lang="nl"><head><meta charset="utf-8">
    <title>NTA 8800 invoer- en bewijsoverzicht — ${escapeHtml(project.name)}</title>
    <style>body{font:14px/1.5 system-ui,sans-serif;max-width:1100px;margin:32px auto;padding:0 20px;color:#202530}
    h1,h2,h3,h4{color:#173c67}table{border-collapse:collapse;width:100%;margin:12px 0 22px}th,td{border:1px solid #cdd5dd;padding:6px 8px;text-align:left;vertical-align:top}
    th{background:#edf3f8}.notice{border-left:5px solid #b45309;background:#fff8e7;padding:12px 16px}.pump{break-inside:avoid;margin:28px 0}
    @media print{body{margin:12mm;padding:0;font-size:11px}table{break-inside:avoid}}</style></head><body>
    <h1>NTA 8800 invoer- en bewijsoverzicht</h1>
    <p>Project: ${escapeHtml(project.name)} · Project-ID: ${escapeHtml(project.id)} · Gegenereerd: ${escapeHtml(generatedAt)}</p>
    <div class="notice"><strong>Invoer alleen — niet geattesteerd.</strong> Dit document bevat geen geverifieerde NTA 8800-berekening, BENG-uitkomst of officieel energielabel. Registervelden en bronverwijzingen zijn door de gebruiker ingevoerd en niet extern gecontroleerd.</div>
    <h2>Project en doeluitgave</h2><table><tbody>
      <tr><th>Doeluitgave</th>${cell('NTA 8800:2025+C1:2026')}<th>Gebouwfunctie</th>${cell(project.buildingFunction)}</tr>
      <tr><th>Adres</th>${cell(project.address || 'Niet vastgelegd')}<th>Plaats</th>${cell(project.city || 'Niet vastgelegd')}</tr>
      <tr><th>Rekenzones</th>${cell(project.zones.length)}<th>Warmtepompregistraties</th>${cell(pumps.length)}</tr>
    </tbody></table>
    <h2>NTA-rekeninvoer en bronnen</h2>
    ${project.ntaCalculation ? `<table><thead><tr><th>Pad</th><th>Bron</th><th>Waarden</th></tr></thead><tbody>
      ${ntaSourceRows(project.ntaCalculation).map((row) => `<tr>${cell(row.path)}${cell(row.source.trim() || 'BRON ONTBREEKT')}${cell(row.values)}</tr>`).join('')}
    </tbody></table>` : '<p>Geen NTA-rekeninvoer vastgelegd.</p>'}
    <h2>Warmtepompen en bewijs</h2>
    ${pumps.length ? pumps.map(pumpHtml).join('') : '<p>Geen geclassificeerde warmtepompen vastgelegd.</p>'}
    <p>De geldigheid van verklaringen, productmatch, bronrechten en toepasselijkheid van normroutes moeten afzonderlijk worden gecontroleerd.</p>
    </body></html>`;
}
