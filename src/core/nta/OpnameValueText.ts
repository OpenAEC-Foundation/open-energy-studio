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

/** Decimal points between digits as Dutch decimal commas ("1.47" → "1,47"). */
export function dutchDecimals(text: string): string {
  return text.replace(/(\d)\.(\d)/g, '$1,$2');
}

/** Use-function names in the kernel's value texts. */
const FUNCTION_NL: Record<string, string> = {
  office: 'kantoor', education: 'onderwijs', retail: 'winkel', sport: 'sport', lodging: 'logies', cell: 'cel',
  residential: 'wonen', assembly_without_day_care: 'bijeenkomst zonder kinderopvang',
  assembly_with_day_care: 'bijeenkomst met kinderopvang', healthcare_without_beds: 'gezondheidszorg zonder bed',
  healthcare_with_beds: 'gezondheidszorg met bed',
};

/** Value texts the kernel builds from numbers, with their Dutch form. */
const DUTCH_PATTERNS: [RegExp, (match: RegExpMatchArray) => string][] = [
  [/^(.+) m² of other functions \(≤ 25 %\) counted as (.+)$/, (m) => `${m[1]} m² van andere functies (≤ 25 %) geteld als ${FUNCTION_NL[m[2]] ?? m[2]}`],
  [/^(\d+) function\(s\) beyond the 25 % merge kept in a mixed calculation zone$/,
    (m) => `${m[1]} functie(s) boven de samenvoeging van 25 % apart gehouden in een gemengde rekenzone`],
  [/^(\d+) calculation zones; one heating, cooling and ventilation system serves all$/,
    (m) => `${m[1]} rekenzones; één verwarmings-, koel- en ventilatiesysteem bedient ze alle`],
  [/^outside \(A_g (.+) m² > 500 m²\)$/, (m) => `buiten de thermische zone (A_g ${m[1]} m² > 500 m²)`],
  [/^R_c (\S+) m²K\/W \((.+)\)$/, (m) => `R_c ${m[1]} m²K/W (${dutchDefaultValue(m[2]) ?? m[2]})`],
  [/^R_c (\S+)$/, (m) => `R_c ${m[1]}`],
  [/^R_bw (\S+) m²K\/W, U_xw (\S+) W\/\(m²K\)$/, (m) => `R_bw ${m[1]} m²K/W, U_xw ${m[2]} W/(m²K)`],
  [/^year class of (\d+)$/, (m) => `jaarklasse van ${m[1]}`],
  [/^year class before (\d+), at most R_c (.+)$/, (m) => `jaarklasse vóór ${m[1]}, ten hoogste R_c ${m[2]}`],
  [/^(.+) \(installation year (\d+)\)$/, (m) => `${m[1]} (installatiejaar ${m[2]})`],
  [/^(.+) \(installed (.+)\)$/, (m) => `${m[1]} (geïnstalleerd ${m[2]})`],
  [/^(.+) m² window$/, (m) => `${m[1]} m² raam`],
  [/^(\d+) uninsulated pipe\(s\), (\d+) storey\(s\) each$/, (m) => `${m[1]} ongeïsoleerde leiding(en), elk ${m[2]} bouwla(a)g(en)`],
  [/^(.+) mm reed \((.+) mm measured − 35 mm\)$/, (m) => `${m[1]} mm riet (${m[2]} mm gemeten − 35 mm)`],
  [/^(.+) kW \(largest surveyed heating or cooling system\)$/, (m) => `${m[1]} kW (grootste opgenomen verwarmings- of koelsysteem)`],
  [/^(.+) kW \(50 % of the total\)$/, (m) => `${m[1]} kW (50 % van het totaal)`],
  [/^(.+) kW > 25 kW$/, (m) => `${m[1]} kW > 25 kW`],
  [/^polycrystalline, installed (\d+)$/, (m) => `polykristallijn, geïnstalleerd ${m[1]}`],
  [/^table 14\.5 \(LED from 2017: (true|false)\)$/, (m) => `tabel 14.5 (LED vanaf 2017: ${m[1] === 'true' ? 'ja' : 'nee'})`],
  [/^the pool room lies in (.+), the only zone with sport$/, (m) => `de zwembadruimte ligt in ${m[1]}, de enige zone met sport`],
  [/^ΔU_for (\S+) W\/\(m²K\)$/, (m) => `ΔU_for ${m[1]} W/(m²K)`],
];

/**
 * The Dutch text of a default value: a fixed text from the table, a known
 * number-built text, or otherwise the text with Dutch decimal commas. Null when
 * nothing applies.
 */
export function dutchDefaultValue(value: string): string | null {
  if (Object.prototype.hasOwnProperty.call(DUTCH, value)) return DUTCH[value];
  for (const [pattern, build] of DUTCH_PATTERNS) {
    const match = value.match(pattern);
    if (match) return dutchDecimals(build(match));
  }
  const decimals = dutchDecimals(value);
  return decimals === value ? null : decimals;
}
