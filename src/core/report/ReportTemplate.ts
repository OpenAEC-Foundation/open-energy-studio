import type { IProject, IBENGResult, IEnergyBreakdown } from '../energy/types';
import type { ProjectPerformanceAssessment } from '../nta/KernelClient';
import { escapeHtml } from './HtmlEscaping';
import { indicatorDecimals, kernelReportModel, type KernelReportModel } from './KernelReportModel';
import { kernelWithheld } from '../nta/KernelVerdict';

/** Languages of the standalone BENG report; every other UI language falls back to English. */
export type ReportLanguage = 'nl' | 'en';

export function reportLanguage(locale: string | null | undefined): ReportLanguage {
  return locale?.toLowerCase().startsWith('nl') ? 'nl' : 'en';
}

const TEXT = {
  nl: {
    titleKernel: 'Energieprestatierapport (NTA 8800, onverifieerd)',
    titleIndicative: 'Indicatief energierapport',
    generated: 'Gegenereerd op',
    projectInfo: 'Projectgegevens', projectName: 'Projectnaam', description: 'Omschrijving', address: 'Adres', city: 'Plaats',
    buildingFunction: 'Gebouwfunctie', floorArea: 'Totaal vloeroppervlak (Ag)', zoneCount: 'Aantal zones',
    envelope: 'Gebouwschil', zone: 'Zone', height: 'Hoogte', volume: 'Volume', airTightness: 'Luchtdichtheid (qv10)',
    surfaces: 'Oppervlakken', name: 'Naam', type: 'Type', orientation: 'Oriëntatie', area: 'Oppervlak', uValue: 'U-waarde',
    construction: 'Constructie', window: 'Raam', noSurfaces: 'Geen oppervlakken gedefinieerd',
    thermalBridges: 'Koudebruggen', psi: 'Ψ-waarde', length: 'Lengte', loss: 'Verlies',
    installations: 'Installaties', heating: 'Verwarming', efficiency: 'Rendement / COP', coverage: 'Dekking',
    ventilation: 'Ventilatie', heatRecovery: 'WTW-rendement', cooling: 'Koeling', hotWater: 'Warm tapwater',
    solarBoiler: 'Zonneboiler', yes: 'Ja', no: 'Nee',
    noHeating: 'Geen verwarmingssystemen gedefinieerd', noVentilation: 'Geen ventilatiesystemen gedefinieerd',
    noCooling: 'Geen koelsystemen gedefinieerd', noHotWater: 'Geen warm tapwater systemen gedefinieerd',
    renewables: 'Hernieuwbare energie', pv: 'Zonnepanelen (PV)', peakPower: 'Piekvermogen', tilt: 'Helling',
    noPv: 'Geen zonnepanelen gedefinieerd', solarThermal: 'Zonnecollectoren (thermisch)', noSolarThermal: 'Geen zonnecollectoren gedefinieerd',
    pvKernel: 'Zonnestroom volgens hoofdstuk 16', system: 'Systeem', production: 'Opwekking',
    bengKernel: 'BENG-indicatoren (NTA 8800-kern)',
    kernelNotice: 'Uit de Rust-rekenkern van Open Energy Studio (NTA 8800:2025+C1:2026). Onverifieerde berekening: geen officieel energielabel, niet geattesteerd en niet geregistreerd.',
    bengIndicative: 'Indicatieve BENG-waarden',
    withheldTitle: 'BENG-resultaten achtergehouden',
    withheldInvalid: 'De NTA-rekenkern keurt de invoer af. Dit rapport bevat daarom geen BENG-waarden, ook geen vereenvoudigde. Los eerst de meldingen in het NTA-paneel op.',
    withheldIncomplete: 'De NTA-invoer is nog onvolledig. Dit rapport bevat daarom geen BENG-waarden, ook geen vereenvoudigde. Vul eerst de ontbrekende gegevens aan.',
    indicativeNotice: 'Indicatief, niet volgens NTA 8800. Uitkomsten uit het oude vereenvoudigde rekenmodel. Geen geverifieerde NTA 8800-berekening, officieel energielabel of wettelijke toetsing.',
    indicator: 'Indicator', calculated: 'Berekend', limit: 'Eis', unit: 'Eenheid', status: 'Status',
    indicativeStatus: 'Indicatief', meets: 'voldoet (onverifieerd)', fails: 'voldoet niet', notTestable: 'niet te toetsen',
    beng1: 'BENG 1 – Energiebehoefte', beng2: 'BENG 2 – Primair fossiel energiegebruik', beng3: 'BENG 3 – Aandeel hernieuwbare energie',
    tojuli: 'TOjuli – Risico op oververhitting', labelClass: 'Indicatieve labelklasse',
    perM2Year: 'kWh/(m²·jaar)', perYear: 'kWh/jaar',
    monthly: 'Maandoverzicht', month: 'Maand', heatingNeed: 'Warmtebehoefte', coolingNeed: 'Koudebehoefte',
    solarGain: 'Zonnewinst', transmission: 'Transmissieverlies',
    balance: 'Energiebalans', losses: 'Verliezen', gains: 'Winsten', post: 'Post', value: 'Waarde',
    transmissionLoss: 'Transmissieverliezen', ventilationLoss: 'Ventilatieverliezen', ventilationInfiltration: 'Ventilatie- en infiltratieverliezen',
    infiltrationLoss: 'Infiltratieverliezen', internalGain: 'Interne warmtewinst', otherGain: 'Overige winst (serre 7.37 e.a.)',
    needAndUse: 'Energiebehoefte & -gebruik', heatingDemand: 'Verwarmingsbehoefte', coolingDemand: 'Koelbehoefte',
    heatingEnergy: 'Verwarming (geleverd)', coolingEnergy: 'Koeling (geleverd)', ventilationEnergy: 'Ventilatie (ventilatoren)',
    hotWaterEnergy: 'Warm tapwater (geleverd)', lightingEnergy: 'Verlichting', auxiliaryEnergy: 'Hulpenergie',
    totalPrimary: 'Totaal primaire fossiele energie', renewableProduction: 'Hernieuwbare opwekking',
    pvProduction: 'PV-opwekking', solarThermalProduction: 'Zonnecollectoren', totalRenewable: 'Totaal hernieuwbaar',
    deliveredUnavailable: 'Geleverde energie per energiefunctie ontbreekt in de kernuitkomst.',
    months: ['jan', 'feb', 'mrt', 'apr', 'mei', 'jun', 'jul', 'aug', 'sep', 'okt', 'nov', 'dec'],
    functions: { residential: 'Woonfunctie', office: 'Kantoorfunctie', education: 'Onderwijsfunctie', healthcare: 'Gezondheidszorgfunctie',
      retail: 'Winkelfunctie', industrial: 'Industriefunctie', other: 'Overig' } as Record<string, string>,
    heatingTypes: { hr107: 'HR107-ketel', hr_combi: 'HR-combiketel', heat_pump_air: 'Warmtepomp (lucht)', heat_pump_ground: 'Warmtepomp (bodem)',
      district_heating: 'Stadsverwarming', electric: 'Elektrisch', biomass: 'Biomassa' } as Record<string, string>,
    ventilationTypes: { natural: 'Natuurlijke ventilatie', type_c: 'Mechanische afvoer (systeem C)', type_d: 'Balansventilatie (systeem D)' } as Record<string, string>,
    coolingTypes: { none: 'Geen', split_unit: 'Split-unit', central_chiller: 'Centrale koelmachine', heat_pump_reversible: 'Warmtepomp (reversibel)' } as Record<string, string>,
    hotWaterTypes: { hr_combi: 'HR-combiketel', heat_pump: 'Warmtepomp', electric_boiler: 'Elektrische boiler', solar_boiler: 'Zonneboiler',
      district_heating: 'Stadsverwarming' } as Record<string, string>,
    surfaceTypes: { wall: 'Wand', roof: 'Dak', floor: 'Vloer', internal: 'Intern' } as Record<string, string>,
    orientations: { N: 'Noord', NE: 'Noordoost', E: 'Oost', SE: 'Zuidoost', S: 'Zuid', SW: 'Zuidwest', W: 'West', NW: 'Noordwest',
      horizontal: 'Horizontaal' } as Record<string, string>,
    solarThermalTypes: { flat_plate: 'Vlakke plaat', vacuum_tube: 'Vacuümbuis' } as Record<string, string>,
  },
  en: {
    titleKernel: 'Energy performance report (NTA 8800, unverified)',
    titleIndicative: 'Indicative energy report',
    generated: 'Generated on',
    projectInfo: 'Project information', projectName: 'Project name', description: 'Description', address: 'Address', city: 'City',
    buildingFunction: 'Building function', floorArea: 'Total floor area (Ag)', zoneCount: 'Number of zones',
    envelope: 'Building envelope', zone: 'Zone', height: 'Height', volume: 'Volume', airTightness: 'Air tightness (qv10)',
    surfaces: 'Surfaces', name: 'Name', type: 'Type', orientation: 'Orientation', area: 'Area', uValue: 'U-value',
    construction: 'Construction', window: 'Window', noSurfaces: 'No surfaces defined',
    thermalBridges: 'Thermal bridges', psi: 'Ψ-value', length: 'Length', loss: 'Loss',
    installations: 'Installations', heating: 'Heating', efficiency: 'Efficiency / COP', coverage: 'Coverage',
    ventilation: 'Ventilation', heatRecovery: 'Heat recovery efficiency', cooling: 'Cooling', hotWater: 'Domestic hot water',
    solarBoiler: 'Solar water heater', yes: 'Yes', no: 'No',
    noHeating: 'No heating systems defined', noVentilation: 'No ventilation systems defined',
    noCooling: 'No cooling systems defined', noHotWater: 'No hot water systems defined',
    renewables: 'Renewable energy', pv: 'Solar panels (PV)', peakPower: 'Peak power', tilt: 'Tilt',
    noPv: 'No solar panels defined', solarThermal: 'Solar collectors (thermal)', noSolarThermal: 'No solar collectors defined',
    pvKernel: 'Solar electricity per chapter 16', system: 'System', production: 'Production',
    bengKernel: 'BENG indicators (NTA 8800 kernel)',
    kernelNotice: 'From the Open Energy Studio Rust kernel (NTA 8800:2025+C1:2026). Unverified calculation: not an official energy label, not attested and not registered.',
    bengIndicative: 'Indicative BENG values',
    withheldTitle: 'BENG results withheld',
    withheldInvalid: 'The NTA kernel refuses the input. This report therefore contains no BENG values, not even simplified ones. Resolve the messages in the NTA panel first.',
    withheldIncomplete: 'The NTA input is still incomplete. This report therefore contains no BENG values, not even simplified ones. Complete the missing data first.',
    indicativeNotice: 'Indicative, not according to NTA 8800. Results of the old simplified model. Not a verified NTA 8800 calculation, official energy label or legal check.',
    indicator: 'Indicator', calculated: 'Calculated', limit: 'Limit', unit: 'Unit', status: 'Status',
    indicativeStatus: 'Indicative', meets: 'meets (unverified)', fails: 'does not meet', notTestable: 'cannot be tested',
    beng1: 'BENG 1 – Energy need', beng2: 'BENG 2 – Primary fossil energy', beng3: 'BENG 3 – Renewable share',
    tojuli: 'TOjuli – Overheating risk', labelClass: 'Indicative label class',
    perM2Year: 'kWh/(m²·yr)', perYear: 'kWh/yr',
    monthly: 'Monthly overview', month: 'Month', heatingNeed: 'Heating need', coolingNeed: 'Cooling need',
    solarGain: 'Solar gain', transmission: 'Transmission loss',
    balance: 'Energy balance', losses: 'Losses', gains: 'Gains', post: 'Item', value: 'Value',
    transmissionLoss: 'Transmission losses', ventilationLoss: 'Ventilation losses', ventilationInfiltration: 'Ventilation and infiltration losses',
    infiltrationLoss: 'Infiltration losses', internalGain: 'Internal gains', otherGain: 'Other gains (sunroom 7.37 etc.)',
    needAndUse: 'Energy need & use', heatingDemand: 'Heating need', coolingDemand: 'Cooling need',
    heatingEnergy: 'Heating (delivered)', coolingEnergy: 'Cooling (delivered)', ventilationEnergy: 'Ventilation (fans)',
    hotWaterEnergy: 'Hot water (delivered)', lightingEnergy: 'Lighting', auxiliaryEnergy: 'Auxiliary energy',
    totalPrimary: 'Total primary fossil energy', renewableProduction: 'Renewable production',
    pvProduction: 'PV production', solarThermalProduction: 'Solar collectors', totalRenewable: 'Total renewable',
    deliveredUnavailable: 'Delivered energy per energy function is missing from the kernel result.',
    months: ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'],
    functions: { residential: 'Residential', office: 'Office', education: 'Education', healthcare: 'Healthcare',
      retail: 'Retail', industrial: 'Industrial', other: 'Other' } as Record<string, string>,
    heatingTypes: { hr107: 'HR107 boiler', hr_combi: 'HR combi boiler', heat_pump_air: 'Heat pump (air)', heat_pump_ground: 'Heat pump (ground)',
      district_heating: 'District heating', electric: 'Electric', biomass: 'Biomass' } as Record<string, string>,
    ventilationTypes: { natural: 'Natural ventilation', type_c: 'Mechanical exhaust (system C)', type_d: 'Balanced ventilation (system D)' } as Record<string, string>,
    coolingTypes: { none: 'None', split_unit: 'Split unit', central_chiller: 'Central chiller', heat_pump_reversible: 'Heat pump (reversible)' } as Record<string, string>,
    hotWaterTypes: { hr_combi: 'HR combi boiler', heat_pump: 'Heat pump', electric_boiler: 'Electric boiler', solar_boiler: 'Solar water heater',
      district_heating: 'District heating' } as Record<string, string>,
    surfaceTypes: { wall: 'Wall', roof: 'Roof', floor: 'Floor', internal: 'Internal' } as Record<string, string>,
    orientations: { N: 'North', NE: 'North-east', E: 'East', SE: 'South-east', S: 'South', SW: 'South-west', W: 'West', NW: 'North-west',
      horizontal: 'Horizontal' } as Record<string, string>,
    solarThermalTypes: { flat_plate: 'Flat plate', vacuum_tube: 'Vacuum tube' } as Record<string, string>,
  },
};

export interface ReportOptions {
  /** The NTA kernel assessment; when calculated, the report shows its figures instead of the simplified engine. */
  kernel?: ProjectPerformanceAssessment | null;
  /** UI language (a BCP 47 tag); Dutch for `nl`, English otherwise. */
  locale?: string;
}

/** Generator efficiency term per heating type: a boiler has an efficiency η, a heat pump a COP. */
function efficiencyText(type: string, value: number, number: (v: number, d?: number) => string): string {
  return type.startsWith('heat_pump') ? `COP ${number(value, 2)}` : `η ${number(value, 3)}`;
}

/**
 * Generates the standalone BENG report. With a calculated kernel assessment the figures
 * come from the NTA kernel; otherwise from the simplified engine, marked
 * "indicatief, niet volgens NTA 8800". Returns an empty-result report when neither exists.
 */
export function generateReportHTML(project: IProject, result: IBENGResult | null, options: ReportOptions = {}): string {
  const language = reportLanguage(options.locale ?? 'nl');
  const L = TEXT[language];
  const tag = language === 'nl' ? 'nl-NL' : 'en-GB';
  const kernel: KernelReportModel | null = kernelReportModel(options.kernel);
  const date = new Date().toLocaleDateString(tag, { year: 'numeric', month: 'long', day: 'numeric' });

  function fmt(value: number | null | undefined, decimals = 1): string {
    if (value == null || !Number.isFinite(value)) return '–';
    return value.toLocaleString(tag, { minimumFractionDigits: decimals, maximumFractionDigits: decimals });
  }
  const label = (labels: Record<string, string>, key: string) => escapeHtml(labels[key] || key);

  function findConstruction(id: string): string {
    const c = project.constructions.find((con) => con.id === id);
    return c ? `${escapeHtml(c.name)} (U=${fmt(c.uValue, 2)} W/m²K)` : escapeHtml(id);
  }
  function findConstructionU(id: string): string {
    const c = project.constructions.find((con) => con.id === id);
    return c ? fmt(c.uValue, 2) : '-';
  }

  const floorArea = kernel?.usableFloorAreaM2 ?? result?.totalFloorArea ?? null;
  const projectInfoHTML = `
    <h2>${L.projectInfo}</h2>
    <table>
      <tr><th style="width:200px">${L.projectName}</th><td>${escapeHtml(project.name)}</td></tr>
      <tr><th>${L.description}</th><td>${escapeHtml(project.description || '-')}</td></tr>
      <tr><th>${L.address}</th><td>${escapeHtml(project.address || '-')}</td></tr>
      <tr><th>${L.city}</th><td>${escapeHtml(project.city || '-')}</td></tr>
      <tr><th>${L.buildingFunction}</th><td>${label(L.functions, project.buildingFunction)}</td></tr>
      <tr><th>${L.floorArea}</th><td>${fmt(floorArea)} m²</td></tr>
      <tr><th>${L.zoneCount}</th><td>${project.zones.length}</td></tr>
    </table>`;

  const zonesHTML = project.zones.map((zone) => {
    const surfaceRows = zone.surfaces.map((s) => {
      const windowRows = s.windows.map((w) => `
        <tr>
          <td style="padding-left:32px">└ ${escapeHtml(w.name)}</td>
          <td>${L.window}</td>
          <td>${label(L.orientations, w.orientation)}</td>
          <td>${fmt(w.area)} m²</td>
          <td>${fmt(w.uValue, 2)}</td>
          <td>g=${fmt(w.gValue, 2)}</td>
        </tr>`).join('');
      return `
        <tr>
          <td>${escapeHtml(s.name)}</td>
          <td>${label(L.surfaceTypes, s.type)}</td>
          <td>${label(L.orientations, s.orientation)}</td>
          <td>${fmt(s.area)} m²</td>
          <td>${findConstructionU(s.constructionId)}</td>
          <td>${findConstruction(s.constructionId)}</td>
        </tr>
        ${windowRows}`;
    }).join('');
    const thermalBridgeRows = zone.thermalBridges.map((tb) => `
      <tr>
        <td>${escapeHtml(tb.name)}</td>
        <td>${fmt(tb.psiValue, 3)} W/(m·K)</td>
        <td>${fmt(tb.length, 1)} m</td>
        <td>${fmt(tb.psiValue * tb.length, 2)} W/K</td>
      </tr>`).join('');
    return `
      <h3>${L.zone}: ${escapeHtml(zone.name)}</h3>
      <table>
        <tr><th style="width:200px">${L.floorArea}</th><td>${fmt(zone.floorArea)} m²</td></tr>
        <tr><th>${L.volume}</th><td>${fmt(zone.volume)} m³</td></tr>
        <tr><th>${L.height}</th><td>${fmt(zone.height)} m</td></tr>
        <tr><th>${L.airTightness}</th><td>${fmt(zone.airTightness.qv10, 2)} dm³/(s·m²)</td></tr>
      </table>
      <h4>${L.surfaces}</h4>
      <table>
        <tr><th>${L.name}</th><th>${L.type}</th><th>${L.orientation}</th><th>${L.area}</th><th>${L.uValue}</th><th>${L.construction}</th></tr>
        ${surfaceRows || `<tr><td colspan="6">${L.noSurfaces}</td></tr>`}
      </table>
      ${zone.thermalBridges.length > 0 ? `<h4>${L.thermalBridges}</h4>
        <table><tr><th>${L.name}</th><th>${L.psi}</th><th>${L.length}</th><th>${L.loss}</th></tr>${thermalBridgeRows}</table>` : ''}`;
  }).join('');

  const heatingRows = project.heatingSystems.map((h) => `
    <tr><td>${escapeHtml(h.name)}</td><td>${label(L.heatingTypes, h.type)}</td><td>${efficiencyText(h.type, h.cop, fmt)}</td><td>${fmt(h.coverageFraction * 100, 0)}%</td></tr>`).join('');
  const ventilationRows = project.ventilationSystems.map((v) => `
    <tr><td>${escapeHtml(v.name)}</td><td>${label(L.ventilationTypes, v.type)}</td><td>${fmt(v.heatRecoveryEfficiency * 100, 0)}%</td><td>${fmt(v.sfp, 1)} W/(dm³/s)</td></tr>`).join('');
  const coolingRows = project.coolingSystems.map((c) => `
    <tr><td>${escapeHtml(c.name)}</td><td>${label(L.coolingTypes, c.type)}</td><td>EER ${fmt(c.eer, 2)}</td></tr>`).join('');
  const hotWaterRows = project.hotWaterSystems.map((hw) => `
    <tr><td>${escapeHtml(hw.name)}</td><td>${label(L.hotWaterTypes, hw.type)}</td><td>${fmt(hw.efficiency * 100, 0)}%</td>
      <td>${hw.hasSolarBoiler ? `${L.yes} (${fmt(hw.solarBoilerFraction * 100, 0)}%)` : L.no}</td></tr>`).join('');
  const installationsHTML = `
    <h2>${L.installations}</h2>
    <h3>${L.heating}</h3>
    <table><tr><th>${L.name}</th><th>${L.type}</th><th>${L.efficiency}</th><th>${L.coverage}</th></tr>
      ${heatingRows || `<tr><td colspan="4">${L.noHeating}</td></tr>`}</table>
    <h3>${L.ventilation}</h3>
    <table><tr><th>${L.name}</th><th>${L.type}</th><th>${L.heatRecovery}</th><th>SFP</th></tr>
      ${ventilationRows || `<tr><td colspan="4">${L.noVentilation}</td></tr>`}</table>
    <h3>${L.cooling}</h3>
    <table><tr><th>${L.name}</th><th>${L.type}</th><th>EER</th></tr>
      ${coolingRows || `<tr><td colspan="3">${L.noCooling}</td></tr>`}</table>
    <h3>${L.hotWater}</h3>
    <table><tr><th>${L.name}</th><th>${L.type}</th><th>${L.efficiency}</th><th>${L.solarBoiler}</th></tr>
      ${hotWaterRows || `<tr><td colspan="4">${L.noHotWater}</td></tr>`}</table>`;

  const pvRows = project.solarPV.map((pv) => `
    <tr><td>${escapeHtml(pv.name)}</td><td>${fmt(pv.peakPower, 1)} kWp</td><td>${fmt(pv.area, 1)} m²</td>
      <td>${label(L.orientations, pv.orientation)}</td><td>${fmt(pv.tilt, 0)}°</td></tr>`).join('');
  const stRows = project.solarThermal.map((st) => `
    <tr><td>${escapeHtml(st.name)}</td><td>${label(L.solarThermalTypes, st.type)}</td><td>${fmt(st.collectorArea, 1)} m²</td>
      <td>${label(L.orientations, st.orientation)}</td><td>${fmt(st.tilt, 0)}°</td></tr>`).join('');
  // With a kernel result the PV systems of the NTA input are the ones calculated (chapter 16).
  const kernelPv = kernel && kernel.pvSystems.length > 0 ? `
    <h3>${L.pvKernel}</h3>
    <table><tr><th>${L.system}</th><th>${L.production}</th></tr>
      ${kernel.pvSystems.map((pv) => `<tr><td>${escapeHtml(pv.id)}</td><td>${fmt(pv.annualKwh, 0)} ${L.perYear}</td></tr>`).join('')}</table>` : '';
  const renewablesHTML = `
    <h2>${L.renewables}</h2>
    ${kernelPv}
    ${pvRows || !kernelPv ? `<h3>${L.pv}</h3>
    <table><tr><th>${L.name}</th><th>${L.peakPower}</th><th>${L.area}</th><th>${L.orientation}</th><th>${L.tilt}</th></tr>
      ${pvRows || `<tr><td colspan="5">${L.noPv}</td></tr>`}</table>` : ''}
    <h3>${L.solarThermal}</h3>
    <table><tr><th>${L.name}</th><th>${L.type}</th><th>${L.area}</th><th>${L.orientation}</th><th>${L.tilt}</th></tr>
      ${stRows || `<tr><td colspan="5">${L.noSolarThermal}</td></tr>`}</table>`;

  const status = (meets: boolean | null) => (meets == null ? L.notTestable : meets ? L.meets : L.fails);
  let bengHTML = '';
  let monthlyHTML = '';
  let breakdown: IEnergyBreakdown | null = null;
  if (kernel) {
    const titles = { beng1: L.beng1, beng2: L.beng2, beng3: L.beng3 };
    const rows = kernel.indicators.map((row) => `
      <tr><td>${titles[row.key]}</td><td>${fmt(row.value, indicatorDecimals(row.key))}</td>
        <td>${row.limit == null ? '–' : `${row.higherIsBetter ? '≥' : '≤'} ${fmt(row.limit, indicatorDecimals(row.key))}`}</td>
        <td>${row.key === 'beng3' ? '%' : L.perM2Year}</td><td>${status(row.meets)}</td></tr>`).join('');
    const tojuli = kernel.tojuli ? `
      <tr><td>${L.tojuli}</td><td>${fmt(kernel.tojuli.value, indicatorDecimals('tojuli'))}</td><td>≤ ${fmt(1.2, indicatorDecimals('tojuli'))}</td><td>K</td><td>${status(kernel.tojuli.meets)}</td></tr>` : '';
    bengHTML = `
      <h2>${L.bengKernel}</h2>
      <p class="verification-notice">${L.kernelNotice}</p>
      <table class="beng-table">
        <tr><th>${L.indicator}</th><th>${L.calculated}</th><th>${L.limit}</th><th>${L.unit}</th><th>${L.status}</th></tr>
        ${rows}${tojuli}
        <tr><td>${L.labelClass}</td><td>${escapeHtml(kernel.labelClass ?? '–')}</td><td></td><td></td><td>${L.notTestable}</td></tr>
      </table>`;
    monthlyHTML = `
      <h2>${L.monthly}</h2>
      <table><tr><th>${L.month}</th><th>${L.heatingNeed}</th><th>${L.coolingNeed}</th><th>${L.solarGain}</th><th>${L.transmission}</th></tr>
        ${kernel.monthly.map((row) => `<tr><td>${L.months[row.month - 1]}</td><td>${fmt(row.heatingNeedKwh, 0)} kWh</td>
          <td>${fmt(row.coolingNeedKwh, 0)} kWh</td><td>${fmt(row.solarGainKwh, 0)} kWh</td><td>${fmt(row.transmissionKwh, 0)} kWh</td></tr>`).join('')}
      </table>`;
    breakdown = kernel.breakdown;
  } else if (kernelWithheld(options.kernel)) {
    // The kernel refused the input: no simplified numbers in its place (shared rule, KernelVerdict).
    bengHTML = `
      <h2>${L.withheldTitle}</h2>
      <p class="verification-notice" data-withheld="true">${options.kernel?.status === 'incomplete' ? L.withheldIncomplete : L.withheldInvalid}</p>`;
  } else if (result) {
    bengHTML = `
      <h2>${L.bengIndicative}</h2>
      <p class="verification-notice">${L.indicativeNotice}</p>
      <table class="beng-table">
        <tr><th>${L.indicator}</th><th>${L.calculated}</th><th>${L.limit}</th><th>${L.unit}</th><th>${L.status}</th></tr>
        <tr><td>${L.beng1}</td><td>${fmt(result.beng1)}</td><td>≤ ${fmt(result.beng1Limit)}</td><td>${L.perM2Year}</td><td>${L.indicativeStatus}</td></tr>
        <tr><td>${L.beng2}</td><td>${fmt(result.beng2)}</td><td>≤ ${fmt(result.beng2Limit)}</td><td>${L.perM2Year}</td><td>${L.indicativeStatus}</td></tr>
        <tr><td>${L.beng3}</td><td>${fmt(result.beng3)}</td><td>≥ ${fmt(result.beng3Limit)}</td><td>%</td><td>${L.indicativeStatus}</td></tr>
      </table>`;
    breakdown = result.breakdown;
  }

  const bd = breakdown;
  const kwh = (value: number) => `${fmt(value, 0)} ${L.perYear}`;
  const breakdownHTML = bd ? `
    <h2>${L.balance}</h2>
    <h3>${L.losses}</h3>
    <table><tr><th>${L.post}</th><th>${L.value}</th></tr>
      <tr><td>${L.transmissionLoss}</td><td>${kwh(bd.transmissionLoss)}</td></tr>
      <tr><td>${kernel ? L.ventilationInfiltration : L.ventilationLoss}</td><td>${kwh(bd.ventilationLoss)}</td></tr>
      ${kernel ? '' : `<tr><td>${L.infiltrationLoss}</td><td>${kwh(bd.infiltrationLoss)}</td></tr>`}
    </table>
    <h3>${L.gains}</h3>
    <table><tr><th>${L.post}</th><th>${L.value}</th></tr>
      <tr><td>${L.solarGain}</td><td>${kwh(bd.solarGain)}</td></tr>
      <tr><td>${L.internalGain}</td><td>${kwh(bd.internalGain)}</td></tr>
      ${bd.otherGain ? `<tr><td>${L.otherGain}</td><td>${kwh(bd.otherGain)}</td></tr>` : ''}
    </table>
    <h3>${L.needAndUse}</h3>
    <table><tr><th>${L.post}</th><th>${L.value}</th></tr>
      <tr><td>${L.heatingDemand}</td><td>${kwh(bd.heatingDemand)}</td></tr>
      <tr><td>${L.coolingDemand}</td><td>${kwh(bd.coolingDemand)}</td></tr>
      ${bd.deliveredUnavailable ? `<tr><td colspan="2">${L.deliveredUnavailable}</td></tr>` : `
      <tr><td>${L.heatingEnergy}</td><td>${kwh(bd.heatingEnergy)}</td></tr>
      <tr><td>${L.coolingEnergy}</td><td>${kwh(bd.coolingEnergy)}</td></tr>
      <tr><td>${L.ventilationEnergy}</td><td>${kwh(bd.ventilationEnergy)}</td></tr>
      <tr><td>${L.hotWaterEnergy}</td><td>${kwh(bd.hotWaterEnergy)}</td></tr>
      <tr><td>${L.lightingEnergy}</td><td>${kwh(bd.lightingEnergy)}</td></tr>
      ${kernel ? `<tr><td>${L.auxiliaryEnergy}</td><td>${kwh(bd.auxiliaryEnergy)}</td></tr>` : ''}`}
      <tr style="font-weight:bold;background:#eff6ff"><td>${L.totalPrimary}</td><td>${kwh(bd.totalPrimaryEnergy)}</td></tr>
    </table>
    <h3>${L.renewableProduction}</h3>
    <table><tr><th>${L.post}</th><th>${L.value}</th></tr>
      <tr><td>${L.pvProduction}</td><td>${kwh(bd.pvProduction)}</td></tr>
      <tr><td>${L.solarThermalProduction}</td><td>${kwh(bd.solarThermalProduction)}</td></tr>
      <tr style="font-weight:bold;background:#eff6ff"><td>${L.totalRenewable}</td><td>${kwh(bd.renewableEnergy)}</td></tr>
    </table>` : '';

  const title = kernel ? L.titleKernel : L.titleIndicative;
  return `<!DOCTYPE html>
<html lang="${language}">
<head>
<meta charset="UTF-8">
<title>${title} - ${escapeHtml(project.name)}</title>
<style>
  body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 40px; color: #333; line-height: 1.5; font-size: 14px; }
  h1 { color: #1e40af; border-bottom: 3px solid #1e40af; padding-bottom: 8px; margin-bottom: 4px; font-size: 24px; }
  .subtitle { color: #666; font-size: 13px; margin-bottom: 24px; }
  h2 { color: #1e40af; margin-top: 32px; font-size: 18px; border-bottom: 1px solid #93c5fd; padding-bottom: 4px; }
  h3 { color: #1e3a5f; margin-top: 20px; font-size: 15px; }
  h4 { color: #555; margin-top: 16px; font-size: 13px; }
  table { width: 100%; border-collapse: collapse; margin: 12px 0; font-size: 13px; }
  th { background: #eff6ff; padding: 8px 12px; text-align: left; border: 1px solid #ddd; font-weight: 600; }
  td { padding: 8px 12px; border: 1px solid #ddd; }
  tr:nth-child(even) td { background: #fafafa; }
  .verification-notice { padding: 10px 12px; border-left: 4px solid #d97706; background: #fffbeb; color: #713f12; }
  .beng-table td:first-child { font-weight: 600; }
  .beng-table tr { height: 40px; }
  .footer { margin-top: 48px; padding-top: 12px; border-top: 1px solid #ddd; color: #999; font-size: 11px; display: flex; justify-content: space-between; }
  @media print {
    body { margin: 20px; font-size: 11px; }
    h1 { font-size: 20px; }
    h2 { font-size: 15px; break-before: auto; }
    table { page-break-inside: avoid; }
    .footer { position: fixed; bottom: 0; left: 0; right: 0; padding: 8px 20px; }
  }
</style>
</head>
<body>
  <h1>${title} – ${escapeHtml(project.name)}</h1>
  <div class="subtitle">${L.generated} ${date} | Open Energy Studio</div>
  ${projectInfoHTML}
  <h2>${L.envelope}</h2>
  ${zonesHTML}
  ${installationsHTML}
  ${renewablesHTML}
  ${bengHTML}
  ${monthlyHTML}
  ${breakdownHTML}
  <div class="footer">
    <span>${title} – ${escapeHtml(project.name)}</span>
    <span>${L.generated} ${date} — Open Energy Studio</span>
  </div>
</body>
</html>`;
}
