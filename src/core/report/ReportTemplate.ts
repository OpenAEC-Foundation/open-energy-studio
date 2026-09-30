import type { IProject, IBENGResult } from '../energy/types';
import { escapeHtml } from './HtmlEscaping';

/**
 * Generates a standalone report of indicative legacy energy estimates.
 */
export function generateReportHTML(project: IProject, result: IBENGResult): string {
  const date = new Date().toLocaleDateString('nl-NL', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });

  const buildingFunctionLabels: Record<string, string> = {
    residential: 'Woonfunctie',
    office: 'Kantoorfunctie',
    education: 'Onderwijsfunctie',
    healthcare: 'Gezondheidszorgfunctie',
    retail: 'Winkelfunctie',
    industrial: 'Industriefunctie',
    other: 'Overig',
  };

  const heatingTypeLabels: Record<string, string> = {
    hr107: 'HR107 ketel',
    hr_combi: 'HR combi-ketel',
    heat_pump_air: 'Warmtepomp (lucht)',
    heat_pump_ground: 'Warmtepomp (bodem)',
    district_heating: 'Stadsverwarming',
    electric: 'Elektrisch',
    biomass: 'Biomassa',
  };

  const ventilationTypeLabels: Record<string, string> = {
    natural: 'Natuurlijke ventilatie',
    type_c: 'Mechanisch (Type C)',
    type_d: 'Balansventilatie (Type D)',
  };

  const coolingTypeLabels: Record<string, string> = {
    none: 'Geen',
    split_unit: 'Split-unit',
    central_chiller: 'Centrale koelmachine',
    heat_pump_reversible: 'Warmtepomp (reversibel)',
  };

  const hotWaterTypeLabels: Record<string, string> = {
    hr_combi: 'HR combi-ketel',
    heat_pump: 'Warmtepomp',
    electric_boiler: 'Elektrische boiler',
    solar_boiler: 'Zonneboiler',
    district_heating: 'Stadsverwarming',
  };

  const surfaceTypeLabels: Record<string, string> = {
    wall: 'Wand',
    roof: 'Dak',
    floor: 'Vloer',
    internal: 'Intern',
  };

  const orientationLabels: Record<string, string> = {
    N: 'Noord',
    NE: 'Noordoost',
    E: 'Oost',
    SE: 'Zuidoost',
    S: 'Zuid',
    SW: 'Zuidwest',
    W: 'West',
    NW: 'Noordwest',
    horizontal: 'Horizontaal',
  };

  const solarThermalTypeLabels: Record<string, string> = {
    flat_plate: 'Vlakke plaat',
    vacuum_tube: 'Vacuumbuis',
  };

  function fmt(value: number, decimals = 1): string {
    return value.toFixed(decimals);
  }

  function findConstruction(id: string): string {
    const c = project.constructions.find((con) => con.id === id);
    return c ? `${escapeHtml(c.name)} (U=${fmt(c.uValue, 2)} W/m\u00b2K)` : escapeHtml(id);
  }

  function findConstructionU(id: string): string {
    const c = project.constructions.find((con) => con.id === id);
    return c ? fmt(c.uValue, 2) : '-';
  }

  // ---- Project info section ----
  const projectInfoHTML = `
    <h2>Projectgegevens</h2>
    <table>
      <tr><th style="width:200px">Projectnaam</th><td>${escapeHtml(project.name)}</td></tr>
      <tr><th>Omschrijving</th><td>${escapeHtml(project.description || '-')}</td></tr>
      <tr><th>Adres</th><td>${escapeHtml(project.address || '-')}</td></tr>
      <tr><th>Plaats</th><td>${escapeHtml(project.city || '-')}</td></tr>
      <tr><th>Gebouwfunctie</th><td>${escapeHtml(buildingFunctionLabels[project.buildingFunction] || project.buildingFunction)}</td></tr>
      <tr><th>Totaal vloeroppervlak (Ag)</th><td>${fmt(result.totalFloorArea)} m\u00b2</td></tr>
      <tr><th>Aantal zones</th><td>${project.zones.length}</td></tr>
    </table>`;

  // ---- Building envelope per zone ----
  const zonesHTML = project.zones
    .map((zone) => {
      const surfaceRows = zone.surfaces
        .map((s) => {
          const windowRows = s.windows
            .map(
              (w) => `
              <tr>
                <td style="padding-left:32px">\u2514 ${escapeHtml(w.name)}</td>
                <td>Raam</td>
                <td>${escapeHtml(orientationLabels[w.orientation] || w.orientation)}</td>
                <td>${fmt(w.area)} m\u00b2</td>
                <td>${fmt(w.uValue, 2)}</td>
                <td>g=${fmt(w.gValue, 2)}</td>
              </tr>`
            )
            .join('');

          return `
            <tr>
              <td>${escapeHtml(s.name)}</td>
              <td>${escapeHtml(surfaceTypeLabels[s.type] || s.type)}</td>
              <td>${escapeHtml(orientationLabels[s.orientation] || s.orientation)}</td>
              <td>${fmt(s.area)} m\u00b2</td>
              <td>${findConstructionU(s.constructionId)}</td>
              <td>${findConstruction(s.constructionId)}</td>
            </tr>
            ${windowRows}`;
        })
        .join('');

      const thermalBridgeRows = zone.thermalBridges
        .map(
          (tb) => `
          <tr>
            <td>${escapeHtml(tb.name)}</td>
            <td>${fmt(tb.psiValue, 3)} W/(m\u00b7K)</td>
            <td>${fmt(tb.length, 1)} m</td>
            <td>${fmt(tb.psiValue * tb.length, 2)} W/K</td>
          </tr>`
        )
        .join('');

      return `
        <h3>Zone: ${escapeHtml(zone.name)}</h3>
        <table>
          <tr><th style="width:200px">Vloeroppervlak (Ag)</th><td>${fmt(zone.floorArea)} m\u00b2</td></tr>
          <tr><th>Volume</th><td>${fmt(zone.volume)} m\u00b3</td></tr>
          <tr><th>Hoogte</th><td>${fmt(zone.height)} m</td></tr>
          <tr><th>Luchtdichtheid (qv10)</th><td>${fmt(zone.airTightness.qv10, 2)} dm\u00b3/(s\u00b7m\u00b2)</td></tr>
        </table>
        <h4>Oppervlakken</h4>
        <table>
          <tr><th>Naam</th><th>Type</th><th>Ori\u00ebntatie</th><th>Oppervlak</th><th>U-waarde</th><th>Constructie</th></tr>
          ${surfaceRows || '<tr><td colspan="6">Geen oppervlakken gedefinieerd</td></tr>'}
        </table>
        ${
          zone.thermalBridges.length > 0
            ? `<h4>Koudebruggen</h4>
               <table>
                 <tr><th>Naam</th><th>\u03a8-waarde</th><th>Lengte</th><th>Verlies</th></tr>
                 ${thermalBridgeRows}
               </table>`
            : ''
        }`;
    })
    .join('');

  // ---- Installations section ----
  const heatingRows = project.heatingSystems
    .map(
      (h) => `
      <tr>
        <td>${escapeHtml(h.name)}</td>
        <td>${escapeHtml(heatingTypeLabels[h.type] || h.type)}</td>
        <td>${fmt(h.cop, 2)}</td>
        <td>${fmt(h.coverageFraction * 100, 0)}%</td>
      </tr>`
    )
    .join('');

  const ventilationRows = project.ventilationSystems
    .map(
      (v) => `
      <tr>
        <td>${escapeHtml(v.name)}</td>
        <td>${escapeHtml(ventilationTypeLabels[v.type] || v.type)}</td>
        <td>${fmt(v.heatRecoveryEfficiency * 100, 0)}%</td>
        <td>${fmt(v.sfp, 1)} W/(dm\u00b3/s)</td>
      </tr>`
    )
    .join('');

  const coolingRows = project.coolingSystems
    .map(
      (c) => `
      <tr>
        <td>${escapeHtml(c.name)}</td>
        <td>${escapeHtml(coolingTypeLabels[c.type] || c.type)}</td>
        <td>${fmt(c.eer, 2)}</td>
      </tr>`
    )
    .join('');

  const hotWaterRows = project.hotWaterSystems
    .map(
      (hw) => `
      <tr>
        <td>${escapeHtml(hw.name)}</td>
        <td>${escapeHtml(hotWaterTypeLabels[hw.type] || hw.type)}</td>
        <td>${fmt(hw.efficiency * 100, 0)}%</td>
        <td>${hw.hasSolarBoiler ? `Ja (${fmt(hw.solarBoilerFraction * 100, 0)}%)` : 'Nee'}</td>
      </tr>`
    )
    .join('');

  const installationsHTML = `
    <h2>Installaties</h2>
    <h3>Verwarming</h3>
    <table>
      <tr><th>Naam</th><th>Type</th><th>COP / rendement</th><th>Dekking</th></tr>
      ${heatingRows || '<tr><td colspan="4">Geen verwarmingssystemen gedefinieerd</td></tr>'}
    </table>
    <h3>Ventilatie</h3>
    <table>
      <tr><th>Naam</th><th>Type</th><th>WTW rendement</th><th>SFP</th></tr>
      ${ventilationRows || '<tr><td colspan="4">Geen ventilatiesystemen gedefinieerd</td></tr>'}
    </table>
    <h3>Koeling</h3>
    <table>
      <tr><th>Naam</th><th>Type</th><th>EER</th></tr>
      ${coolingRows || '<tr><td colspan="3">Geen koelsystemen gedefinieerd</td></tr>'}
    </table>
    <h3>Warm tapwater</h3>
    <table>
      <tr><th>Naam</th><th>Type</th><th>Rendement</th><th>Zonneboiler</th></tr>
      ${hotWaterRows || '<tr><td colspan="4">Geen warm tapwater systemen gedefinieerd</td></tr>'}
    </table>`;

  // ---- Renewables section ----
  const pvRows = project.solarPV
    .map(
      (pv) => `
      <tr>
        <td>${escapeHtml(pv.name)}</td>
        <td>${fmt(pv.peakPower, 1)} kWp</td>
        <td>${fmt(pv.area, 1)} m\u00b2</td>
        <td>${escapeHtml(orientationLabels[pv.orientation] || pv.orientation)}</td>
        <td>${fmt(pv.tilt, 0)}\u00b0</td>
      </tr>`
    )
    .join('');

  const stRows = project.solarThermal
    .map(
      (st) => `
      <tr>
        <td>${escapeHtml(st.name)}</td>
        <td>${escapeHtml(solarThermalTypeLabels[st.type] || st.type)}</td>
        <td>${fmt(st.collectorArea, 1)} m\u00b2</td>
        <td>${escapeHtml(orientationLabels[st.orientation] || st.orientation)}</td>
        <td>${fmt(st.tilt, 0)}\u00b0</td>
      </tr>`
    )
    .join('');

  const renewablesHTML = `
    <h2>Hernieuwbare energie</h2>
    <h3>Zonnepanelen (PV)</h3>
    <table>
      <tr><th>Naam</th><th>Piekvermogen</th><th>Oppervlak</th><th>Ori\u00ebntatie</th><th>Helling</th></tr>
      ${pvRows || '<tr><td colspan="5">Geen zonnepanelen gedefinieerd</td></tr>'}
    </table>
    <h3>Zonnecollectoren (thermisch)</h3>
    <table>
      <tr><th>Naam</th><th>Type</th><th>Oppervlak</th><th>Ori\u00ebntatie</th><th>Helling</th></tr>
      ${stRows || '<tr><td colspan="5">Geen zonnecollectoren gedefinieerd</td></tr>'}
    </table>`;

  // ---- BENG results section ----
  const bengResultsHTML = `
    <h2>Indicatieve BENG-waarden</h2>
    <p class="verification-notice">Indicatieve uitkomsten uit het oude vereenvoudigde rekenmodel. Geen geverifieerde NTA 8800-berekening, officieel energielabel of wettelijke toetsing.</p>
    <table class="beng-table">
      <tr>
        <th>Indicator</th>
        <th>Berekend</th>
        <th>Eis</th>
        <th>Eenheid</th>
        <th>Status</th>
      </tr>
      <tr>
        <td>BENG 1 \u2013 Energiebehoefte</td>
        <td>${fmt(result.beng1)}</td>
        <td>\u2264 ${fmt(result.beng1Limit)}</td>
        <td>kWh/(m\u00b2\u00b7jaar)</td>
        <td>Indicatief</td>
      </tr>
      <tr>
        <td>BENG 2 \u2013 Primair fossiel energiegebruik</td>
        <td>${fmt(result.beng2)}</td>
        <td>\u2264 ${fmt(result.beng2Limit)}</td>
        <td>kWh/(m\u00b2\u00b7jaar)</td>
        <td>Indicatief</td>
      </tr>
      <tr>
        <td>BENG 3 \u2013 Aandeel hernieuwbare energie</td>
        <td>${fmt(result.beng3)}</td>
        <td>\u2265 ${fmt(result.beng3Limit)}</td>
        <td>%</td>
        <td>Indicatief</td>
      </tr>
    </table>`;

  // ---- Energy breakdown section ----
  const bd = result.breakdown;
  const breakdownHTML = `
    <h2>Energiebalans</h2>
    <h3>Verliezen</h3>
    <table>
      <tr><th>Post</th><th>Waarde</th></tr>
      <tr><td>Transmissieverliezen</td><td>${fmt(bd.transmissionLoss, 0)} kWh/jaar</td></tr>
      <tr><td>Ventilatieverliezen</td><td>${fmt(bd.ventilationLoss, 0)} kWh/jaar</td></tr>
      <tr><td>Infiltratieverliezen</td><td>${fmt(bd.infiltrationLoss, 0)} kWh/jaar</td></tr>
    </table>
    <h3>Winsten</h3>
    <table>
      <tr><th>Post</th><th>Waarde</th></tr>
      <tr><td>Zonnewinst</td><td>${fmt(bd.solarGain, 0)} kWh/jaar</td></tr>
      <tr><td>Interne warmtewinst</td><td>${fmt(bd.internalGain, 0)} kWh/jaar</td></tr>
    </table>
    <h3>Energiebehoefte &amp; -gebruik</h3>
    <table>
      <tr><th>Post</th><th>Waarde</th></tr>
      <tr><td>Verwarmingsbehoefte</td><td>${fmt(bd.heatingDemand, 0)} kWh/jaar</td></tr>
      <tr><td>Koelbehoefte</td><td>${fmt(bd.coolingDemand, 0)} kWh/jaar</td></tr>
      <tr><td>Verwarming (geleverd)</td><td>${fmt(bd.heatingEnergy, 0)} kWh/jaar</td></tr>
      <tr><td>Koeling (geleverd)</td><td>${fmt(bd.coolingEnergy, 0)} kWh/jaar</td></tr>
      <tr><td>Ventilatie (ventilatoren)</td><td>${fmt(bd.ventilationEnergy, 0)} kWh/jaar</td></tr>
      <tr><td>Warm tapwater (geleverd)</td><td>${fmt(bd.hotWaterEnergy, 0)} kWh/jaar</td></tr>
      <tr><td>Verlichting</td><td>${fmt(bd.lightingEnergy, 0)} kWh/jaar</td></tr>
      <tr style="font-weight:bold;background:#eff6ff">
        <td>Totaal primaire energie</td><td>${fmt(bd.totalPrimaryEnergy, 0)} kWh/jaar</td>
      </tr>
    </table>
    <h3>Hernieuwbare opwekking</h3>
    <table>
      <tr><th>Post</th><th>Waarde</th></tr>
      <tr><td>PV-opwekking</td><td>${fmt(bd.pvProduction, 0)} kWh/jaar</td></tr>
      <tr><td>Zonnecollectoren</td><td>${fmt(bd.solarThermalProduction, 0)} kWh/jaar</td></tr>
      <tr style="font-weight:bold;background:#eff6ff">
        <td>Totaal hernieuwbaar</td><td>${fmt(bd.renewableEnergy, 0)} kWh/jaar</td>
      </tr>
    </table>`;

  // ---- Assemble full document ----
  return `<!DOCTYPE html>
<html lang="nl">
<head>
<meta charset="UTF-8">
<title>Indicatief energierapport - ${escapeHtml(project.name)}</title>
<style>
  body {
    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
    margin: 40px;
    color: #333;
    line-height: 1.5;
    font-size: 14px;
  }
  h1 {
    color: #1e40af;
    border-bottom: 3px solid #1e40af;
    padding-bottom: 8px;
    margin-bottom: 4px;
    font-size: 24px;
  }
  .subtitle {
    color: #666;
    font-size: 13px;
    margin-bottom: 24px;
  }
  h2 {
    color: #1e40af;
    margin-top: 32px;
    font-size: 18px;
    border-bottom: 1px solid #93c5fd;
    padding-bottom: 4px;
  }
  h3 {
    color: #1e3a5f;
    margin-top: 20px;
    font-size: 15px;
  }
  h4 {
    color: #555;
    margin-top: 16px;
    font-size: 13px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    margin: 12px 0;
    font-size: 13px;
  }
  th {
    background: #eff6ff;
    padding: 8px 12px;
    text-align: left;
    border: 1px solid #ddd;
    font-weight: 600;
  }
  td {
    padding: 8px 12px;
    border: 1px solid #ddd;
  }
  tr:nth-child(even) td {
    background: #fafafa;
  }
  .verification-notice {
    padding: 10px 12px;
    border-left: 4px solid #d97706;
    background: #fffbeb;
    color: #713f12;
  }
  .beng-table td:first-child {
    font-weight: 600;
  }
  .beng-table tr {
    height: 40px;
  }
  .footer {
    margin-top: 48px;
    padding-top: 12px;
    border-top: 1px solid #ddd;
    color: #999;
    font-size: 11px;
    display: flex;
    justify-content: space-between;
  }
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
  <h1>Indicatief energierapport \u2013 ${escapeHtml(project.name)}</h1>
  <div class="subtitle">Gegenereerd op ${date} | Open Energy Studio</div>

  ${projectInfoHTML}

  <h2>Gebouwschil</h2>
  ${zonesHTML}

  ${installationsHTML}

  ${renewablesHTML}

  ${bengResultsHTML}

  ${breakdownHTML}

  <div class="footer">
    <span>Indicatief energierapport \u2013 ${escapeHtml(project.name)}</span>
    <span>Gegenereerd op ${date} met Open Energy Studio</span>
  </div>
</body>
</html>`;
}
