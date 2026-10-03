/**
 * The basisopname records each applied default with a short value text written in
 * English by the kernel. In Dutch these fixed texts are translated here; texts built
 * from numbers (areas, years, powers) stay as written.
 */
const DUTCH: Record<string, string> = {
  '+20 %': '+20 %',
  '0 %': '0 %',
  '0 m in unheated spaces': '0 m in onverwarmde ruimten',
  '0 m² (no reduction of A_g in 13.32a)': '0 m² (geen verlaging van A_g in 13.32a)',
  '0 m² sport halls; the swimming-pool room counts': '0 m² sportzalen; de zwembadruimte telt mee',
  '1 electrically connected device (10 W stand-by)': '1 elektrisch aangesloten toestel (10 W stand-by)',
  '1 uninsulated pipe (interpretation: at least one toilet group)': '1 ongeïsoleerde leiding (interpretatie: ten minste één toiletgroep)',
  '< 20 %: NTA default x = 20': '< 20 %: NTA-standaardwaarde x = 20',
  'AC fans, room higher than 8 m, without warm-air return': 'AC-ventilatoren, ruimte hoger dan 8 m, zonder warmeluchtterugvoer',
  'NTA 11.124 fallback': 'terugvaloptie NTA 11.124',
  'all grilles': 'alle roosters',
  'central on-control, no daylight control': 'centrale aanschakeling, geen daglichtregeling',
  'class 4': 'klasse 4',
  'class C or D': 'klasse C of D',
  'class D': 'klasse D',
  'construction year': 'bouwjaar',
  'conventional boiler': 'conventionele ketel',
  'cooling coil: direct expansion in the AHU': 'koelbatterij: directe expansie in de LBK',
  'decentral part D.5b, other part per principle': 'decentraal deel D.5b, overig deel volgens het principe',
  'f_sto;dis;ls 5': 'f_sto;dis;ls 5',
  false: 'nee',
  true: 'ja',
  'true (10 W stand-by)': 'ja (10 W stand-by)',
  'forfait (14.14)': 'forfaitair (14.14)',
  'forfait (9.41–9.51)': 'forfaitair (9.41–9.51)',
  'forfait power for all lighting zones': 'forfaitair vermogen voor alle verlichtingszones',
  forfait: 'forfaitair',
  'heat pumps, CHP, biomass, external heat, electric, boilers': 'warmtepompen, WKK, biomassa, externe warmte, elektrisch, ketels',
  'length, insulation and diameter forfait; fittings uninsulated': 'lengte, isolatie en diameter forfaitair; appendages ongeïsoleerd',
  'light floor, light wall': 'lichte vloer, lichte wand',
  'manufactured up to 2017': 'geproduceerd tot en met 2017',
  minimal: 'minimaal',
  'multi-junction': 'multi-junctie',
  'no CO₂, time control or zoning': 'geen CO₂-, tijdsturing of zonering',
  'no cooling coil (interpretation: not determinable is not connected)': 'geen koelbatterij (interpretatie: niet vast te stellen is niet aangesloten)',
  'no per-zone system E areas: every zone keeps the building ratio': 'geen oppervlakken systeem E per zone: elke zone houdt de verhouding van het gebouw',
  'no reheating coil (interpretation: not determinable is not connected)': 'geen naverwarmer (interpretatie: niet vast te stellen is niet aangesloten)',
  'no time control, CO₂ measurement or control, no zoning': 'geen tijdsturing, CO₂-meting of -sturing, geen zonering',
  none: 'geen',
  'not applicable: no air-heater fan energy': 'niet van toepassing: geen ventilatorenergie luchtverwarmer',
  'not established: only the calculated carriers decide': 'niet vastgesteld: alleen de berekende dragers bepalen',
  'not insulated': 'niet geïsoleerd',
  'not insulated (R < 0,3)': 'niet geïsoleerd (R < 0,3)',
  'other_or_unknown, no fan convectors': 'overig of onbekend, geen ventilatorconvectoren',
  'outdoor air (basisopname)': 'buitenlucht (basisopname)',
  'outside the thermal zone': 'buiten de thermische zone',
  'per-zone flows not established: the rest split over those zones by A_g': 'debieten per zone niet vastgesteld: de rest naar A_g over die zones verdeeld',
  pitched: 'hellend',
  present: 'aanwezig',
  'present, forfait length (15 % of L, 9.26)': 'aanwezig, forfaitaire lengte (15 % van L, 9.26)',
  'present, forfait length (9.26)': 'aanwezig, forfaitaire lengte (9.26)',
  'radial recirculation fan': 'radiale recirculatieventilator',
  'recirculation (c_source 1,00)': 'recirculatie (c_source 1,00)',
  'regulatory design flow (no extra capacity)': 'wettelijk ontwerpdebiet (geen extra capaciteit)',
  'reheating coil: heating is air heating via the AHU': 'naverwarmer: verwarming is luchtverwarming via de LBK',
  'sport merged into the main function: no pool factor': 'sport samengevoegd met de hoofdfunctie: geen zwembadfactor',
  'surfaces without zoneId split over the zones by A_g': 'vlakken zonder zone naar A_g over de zones verdeeld',
  'switching (forfait daylight method 14.43/14.44)': 'schakelen (forfaitaire daglichtmethode 14.43/14.44)',
  "the zone sums per function replace the building's function areas": 'de zonesommen per functie vervangen de functieoppervlakken van het gebouw',
  'top storey (table 11.14)': 'bovenste bouwlaag (tabel 11.14)',
  unglazed: 'onafgedekt',
  uninsulated: 'ongeïsoleerd',
  'uninsulated ground floor, R_c 0,15': 'ongeïsoleerde begane grondvloer, R_c 0,15',
  'uninsulated; length by kernel default (4 m / ½ H)': 'ongeïsoleerd; lengte volgens kernstandaard (4 m / ½ H)',
  unknown: 'onbekend',
  'unknown (NTA 11.3.2.2 defaults)': 'onbekend (standaardwaarden NTA 11.3.2.2)',
  'unknown (f_lea;du 1,1)': 'onbekend (f_lea;du 1,1)',
  'unknown (regulatory flow): swimming pool in the zone': 'onbekend (wettelijk debiet): zwembad in de zone',
  '≥ 40 m': '≥ 40 m',
  '> 3 m': '> 3 m',
};

/** PascalCase enum names (`HrCoatedDouble`) as snake_case (`hr_coated_double`). */
export function snakeCase(value: string): string {
  return value.replace(/([a-z0-9])([A-Z])/g, '$1_$2').replace(/([A-Z])([A-Z][a-z])/g, '$1_$2').toLowerCase();
}

/** The Dutch text of a fixed default value, or null when there is none. */
export function dutchDefaultValue(value: string): string | null {
  return Object.prototype.hasOwnProperty.call(DUTCH, value) ? DUTCH[value] : null;
}
