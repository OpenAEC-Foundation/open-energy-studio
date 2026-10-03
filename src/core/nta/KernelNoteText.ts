/**
 * The kernel lists what it leaves out or reads in a particular way
 * (`omittedCorrections` of the monthly need, `omittedTerms` of the heating
 * chain) as fixed English texts. In Dutch they are translated here; a text
 * without a translation is shown as written.
 */
const DUTCH: Record<string, string> = {
  '7.3–7.5 and 7.7–7.9 recoverable losses are applied by the heating chain (apply_recoverable_losses); Q_C;ls;rbl = 0 is prescribed by 10.4/10.43':
    'De terugwinbare verliezen van 7.3–7.5 en 7.7–7.9 worden in de verwarmingsketen verwerkt (apply_recoverable_losses); Q_C;ls;rbl = 0 volgt uit 10.4/10.43.',
  '8.5: H_A = 0 for adjacent heated spaces is prescribed by the norm':
    '8.5: H_A = 0 voor aangrenzende verwarmde ruimten is door de norm voorgeschreven.',
  '§17.3.8 extended obstruction method (hourly NEN 5060) enters as declared factors':
    '§17.3.8 uitgebreide belemmeringsmethode (uurwaarden NEN 5060) wordt als opgegeven factoren ingevoerd.',
  "table 7.10 footnote c is the caller's column choice":
    'Tabel 7.10 voetnoot c: de kolomkeuze ligt bij de invoer.',
  '9.6.1: generators with the same preference share their energy by nominal power; product-specific hybrid switching and domestic hot water priority are not modelled':
    '9.6.1: opwekkers met dezelfde voorkeur delen de energie naar nominaal vermogen; productspecifiek hybride schakelen en tapwatervoorrang zijn niet gemodelleerd.',
  '7.82: ϑ_ztu of the unheated space follows from distributionSystem.unheatedReductionFactor (b_U); without it and without entered values 13 °C is used':
    '7.82: ϑ_ztu van de onverwarmde ruimte volgt uit distributionSystem.unheatedReductionFactor (b_U); zonder die factor en zonder opgegeven waarden geldt 13 °C.',
  'annex Q: c_source (annex V) is not applied to method 1 (9.63 has no c_source; tables 9.27/9.29 only); the degree of regeneration is reported':
    'Bijlage Q: c_source (bijlage V) geldt niet voor methode 1 (9.63 kent geen c_source; alleen tabellen 9.27/9.29); de regeneratiegraad wordt wel gerapporteerd.',
  'annex Q: W_H;gen;aux = W_H;aux;hp;an/(12·3,6) is booked literally per 9.6.3.2 (p. 340); Q.4 (p. 1029) already includes the source pump in η_H;gen;hp, so the norm counts it twice (norm issue, followed as written)':
    'Bijlage Q: W_H;gen;aux = W_H;aux;hp;an/(12·3,6) wordt letterlijk volgens 9.6.3.2 (p. 340) geboekt; Q.4 (p. 1029) neemt de bronpomp al op in η_H;gen;hp, zodat de norm die dubbel telt (normpunt, letterlijk gevolgd).',
  'annex Q: F_H;gen = 1 (Q.1) is taken as met when every bin of table Q.6 is fully covered; the rounded table hours sum to 277,757 instead of 277,778':
    'Bijlage Q: F_H;gen = 1 (Q.1) geldt als elke klasse van tabel Q.6 volledig gedekt is; de afgeronde tabeluren tellen op tot 277,757 in plaats van 277,778.',
  'annex Q: V.1 hot-water term not needed, because c_source is not applied to method 1':
    'Bijlage Q: de tapwaterterm van V.1 is niet nodig, omdat c_source niet voor methode 1 geldt.',
  'annex N: E_H;gen;in converted to the gross calorific value (N.3) with f_Hs/Hi of table M.3 (biomass as wood 1,08)':
    'Bijlage N: E_H;gen;in omgerekend naar bovenwaarde (N.3) met f_Hs/Hi van tabel M.3 (biomassa als hout 1,08).',
  'annex M: ϑ_brm (M.12) from 9.4.2: heating setpoint for a heated space, ϑ_ztu of the distribution system for an installation room; table M.6 otherwise':
    'Bijlage M: ϑ_brm (M.12) volgens 9.4.2: de verwarmingssetpoint in een verwarmde ruimte, ϑ_ztu van het distributiesysteem in een installatieruimte, anders tabel M.6.',
  '9.6.1 with §9.6.3 (p. 331) and 9.6.3.2 (p. 340): an annex Q heat pump in a multiple set takes F_H;gen;gpref from annex Q on the whole node output instead of table 9.23; it must be the only first preference, and the other generators share 1 − F by 9.60 with their preferences renumbered from 1 (estimated β rebased on the share left by the heat pump)':
    '9.6.1 met §9.6.3 (p. 331) en 9.6.3.2 (p. 340): een warmtepomp volgens bijlage Q in een meervoudige set krijgt F_H;gen;gpref uit bijlage Q op de hele knooppuntvraag in plaats van tabel 9.23; hij moet de enige eerste voorkeur zijn, en de overige opwekkers delen 1 − F volgens 9.60, met hun voorkeuren hernummerd vanaf 1 (geschatte β herleid op het deel dat de warmtepomp overlaat).',
  'annex Q with a gas backup: the 9.6.8.1 auxiliary energy of the backup boiler is booked in every month, also when full coverage leaves the boiler without output (literal stand-by term)':
    'Bijlage Q met gasbijstook: de hulpenergie van 9.6.8.1 van de bijstookketel wordt elke maand geboekt, ook als volledige dekking de ketel zonder levering laat (letterlijke stand-byterm).',
  'annex M: LPG, hard coal and lignite of table M.3 are refused (boiler_fuel_without_primary_factor), because tables 5.2/5.3 (p. 94, p. 97) give them no f_P;del or K_CO2':
    'Bijlage M: lpg, steenkool en bruinkool van tabel M.3 worden geweigerd (boiler_fuel_without_primary_factor), omdat tabellen 5.2/5.3 (p. 94, p. 97) er geen f_P;del of K_CO2 voor geven.',
  'annex M: β above 1 (M.24/M.25) is refused as a capacity shortfall, as annex N does; the plausibility bounds P_int ≥ 0,05·P_n and f_gen;ls;P0 ≤ 0,1 are program choices':
    'Bijlage M: β boven 1 (M.24/M.25) wordt als te klein vermogen geweigerd, zoals bijlage N dat doet; de plausibiliteitsgrenzen P_int ≥ 0,05·P_n en f_gen;ls;P0 ≤ 0,1 zijn keuzes van het programma.',
  'annex M: stand-by losses (M.4/M.6) burn fuel in every hour of the month even when the 9.7 feedback of recoverable losses brings the generator output to 0 (a grossly oversized boiler in a small zone); the norm is followed literally':
    'Bijlage M: stand-byverliezen (M.4/M.6) verbruiken elk uur van de maand brandstof, ook als de terugkoppeling van terugwinbare verliezen (9.7) de levering van de opwekker op 0 brengt (een sterk overgedimensioneerde ketel in een kleine zone); de norm wordt letterlijk gevolgd.',
  'tables 5.2/5.4 (biomass classes): a type-plate power above 500 kW classes the boiler as bmA regardless of the entered 500 kW flag':
    'Tabellen 5.2/5.4 (biomassaklassen): een typeplaatvermogen boven 500 kW deelt de ketel in als bmA, ongeacht de opgegeven 500 kW-aanduiding.',
  "tables 5.2/5.4 (p. 94, 'per installatie'): the biomass class follows how the stoves or boilers are modelled. The generators of one multiple set add up; identical systems (§9.1, p. 287, for example one stove per dwelling) and separate heating systems (for example local heaters in different zones) are separate installations and are classed one by one":
    "Tabellen 5.2/5.4 (p. 94, 'per installatie'): de biomassaklasse volgt uit hoe de kachels of ketels zijn gemodelleerd. De opwekkers van één meervoudige set tellen op; identieke systemen (§9.1, p. 287, bijvoorbeeld één kachel per woning) en aparte verwarmingssystemen (bijvoorbeeld lokale toestellen in verschillende zones) zijn aparte installaties en worden elk apart ingedeeld.",
  '9.56 (p. 323) with remark 4 (p. 324): when preference 1 of an annex Q set is estimated at β ≥ 1, the remaining preferences are weighted by their entered nominal powers; only when all of them are missing does each count with the same power (program choice); a partly known set must complete its powers (remark 1, p. 323)':
    '9.56 (p. 323) met opmerking 4 (p. 324): als voorkeur 1 van een set met bijlage Q op β ≥ 1 is geschat, wegen de overige voorkeuren naar hun opgegeven nominale vermogen; alleen als die allemaal ontbreken telt elk met hetzelfde vermogen (keuze van het programma); een deels bekende set moet de vermogens aanvullen (opmerking 1, p. 323).',
  "annex V, V.1 (p. 1115): η_H;gen is referred to 14.6, which in this edition is the lighting daylight factor; the kernel uses the delivered COP with η_el = 1/f_P;del;el, so extraction is Q·(1 − η_el/COP). Reading η_H;gen as COP·η_el gives Q·(1 − 1/COP) and a higher R; the kernel's reading is the conservative one":
    'Bijlage V, V.1 (p. 1115): voor η_H;gen wordt naar 14.6 verwezen, in deze uitgave de daglichtfactor van verlichting; de kern gebruikt de geleverde COP met η_el = 1/f_P;del;el, zodat de onttrekking Q·(1 − η_el/COP) is. η_H;gen lezen als COP·η_el geeft Q·(1 − 1/COP) en een hogere R; de lezing van de kern is de behoudende.',
};

/** A kernel note in the UI language: the Dutch translation for a Dutch UI, otherwise the text as written. */
export function kernelNote(text: string, locale: string): string {
  if (!locale.toLowerCase().startsWith('nl')) return text;
  return Object.prototype.hasOwnProperty.call(DUTCH, text) ? DUTCH[text] : text;
}
