# NTA 8800 dekking: hoofdstukken 9 en 10 en bijlagen J–R (review 77)

Stand 15 oktober 2026, doeleditie NTA 8800:2025+C1:2026. Deel 2 van 3 van de dekkingsronde van review 77. Elke genummerde formule en elke tabel uit hoofdstuk 9 (p. 285–365), hoofdstuk 10 (p. 366–425) en de bijlagen J tot en met R (p. 843–1075) is ingedeeld. Dit document bevat alleen nummers, pagina's en kernbestanden, geen normtekst.

**Werkwijze.** De nummers zijn uit de tekstlaag van de gelicentieerde PDF gehaald; onleesbare formules en tabellen zijn op de gerenderde pagina gelezen. Per nummer is de plaats in `crates/nta8800-core/src` gezocht en nagegaan of een eerdere onafhankelijke herberekening (reviews 63–76, zie [verificatiestatus](nta8800-verificatiestatus.md)) de route raakt. Wat geïmplementeerd was maar door geen onafhankelijke toets werd geraakt, is in deze review met een eigen implementatie vanaf de normpagina's nagerekend, vóór het lezen van de formulecode van de kern. De herberekening staat als Rust-code in `crates/nta8800-core/tests/coverage_ch9_10_independent.rs` en neemt alleen de afgeleide invoer van de kern over die eerder is nagerekend (maandbehoefte, ϑ_e;avg, rekentemperatuur, genivelleerd setpoint, Q_H;ht, Δϑ_int;inc van 9.12, ontvochtiging van hoofdstuk 12, maand-EER van methode 2 en de warmtepompwarmte van r66). Tolerantie 10⁻⁶ relatief.

## Klassen

- **G+T**: geïmplementeerd en onafhankelijk getoetst (review en testbestand in de tabel).
- **G**: geïmplementeerd, niet onafhankelijk getoetst (hooguit een eenheidstest van de bouwer), of een term die volgens de tekst 0 is.
- **NG**: niet geïmplementeerd.
- **NVT**: niet van toepassing op de label- en BENG-berekening: informatief, een productdeclaratie of meetvoorschrift, een codelijst of een invoerbepaling die de gebruiker uitvoert.

## Telling

| Deel | G+T | G | NG | NVT | Totaal |
|---|---|---|---|---|---|
| Hoofdstuk 9 (verwarming) | 113 | 3 | 0 | 3 | 119 |
| Hoofdstuk 10 (koeling) | 107 | 11 | 0 | 4 | 122 |
| Bijlage J | 0 | 0 | 0 | 6 | 6 |
| Bijlage K | 0 | 0 | 0 | 1 | 1 |
| Bijlage L | 2 | 1 | 0 | 0 | 3 |
| Bijlage M | 34 | 0 | 0 | 2 | 36 |
| Bijlage N | 90 | 0 | 1 | 23 | 114 |
| Bijlage O | 2 | 0 | 0 | 3 | 5 |
| Bijlage P | 97 | 1 | 0 | 1 | 99 |
| Bijlage Q | 114 | 4 | 1 | 0 | 119 |
| Bijlage R | 0 | 0 | 0 | 2 | 2 |
| **Totaal** | **559** | **20** | **2** | **45** | **626** |

Formulenummers die in de tekst ontbreken (9.13–9.20, 9.52–9.55, 10.12–10.14, 10.39–10.42, N.51, P.5, tabellen 9.5–9.10 en 9.17, 10.2–10.3) bestaan niet in deze editie.

## Nieuw onafhankelijk getoetst (review 77)

Zestien tests, waarvan veertien slagen en twee als afwijking op `#[ignore]` staan:

| Test | Normplaats |
|---|---|
| `heating_distribution_in_an_unheated_space_with_a_labelled_pump` | 9.26–9.28 (stappen 2–5), 9.30–9.32b, tabellen 9.14–9.16 en 9.X, 9.39–9.47, 9.51, tabellen 9.18–9.22 (p. 302–321) |
| `collective_heating_distribution_with_calculated_psi_and_reference_pump` | 9.33, 9.35, 9.36, 9.37, 15 %-regel, f_gebouw, 9.43–9.46, 9.48/9.49 (p. 303–321) |
| `large_collective_pump_above_2_5_kw` | 9.50 met b = 2, EEI 0,25 (p. 319–320) |
| `heating_pipes_shared_with_hot_water` | zoneleidingen, hele maand, ϑ_int;calc;H, 9.38 (p. 303, 313) |
| `embedded_pipes_in_heating_and_cooling` | 9.34 en 10.25 (p. 311, 385) |
| `emission_fans_and_indirect_air_heaters`, `direct_fired_air_heaters` | 9.21–9.23, tabellen 9.11–9.13, schatting Q_h;b (p. 298–301) |
| `collective_buffer_vessel_in_an_unheated_space` | 9.2.3.5 met 13.58 en tabel 13.9 (p. 291–292, 567–570) |
| `cooling_water_distribution_in_a_dwelling` | koelgrens 10.19 met tabel 10.6, 10.10/10.11, 10.15/10.16, 10.17/10.18, 10.21–10.23c, tabellen 10.4–10.13, 10.28–10.38, 10.45, 10.87 (p. 370–394, 425) |
| `collective_cooling_with_free_cooling_before_a_chiller` | 10.49–10.52 met tabellen 10.15/10.16, 10.24, 10.76, 10.80, 10.83, 10.86, tabellen 10.29, 10.33, 10.34 (p. 384–424) |
| `floor_cooling_with_a_labelled_pump` | 10.33, tabel 10.8 letterlijk (p. 380, 391) |
| `booster_heat_pump_extraction_from_the_cooling_distribution` | 10.6–10.9 (p. 371–373) |
| `free_cooling_with_ground_regeneration_surcharge` | 10.84/10.85 (p. 423–424) |
| `water_cooled_chiller_condenser_auxiliaries` | 10.80, 10.82, tabellen 10.31/10.32, 10.83 (p. 420–422) |
| `collective_buffer_vessel_in_a_heated_space` (`#[ignore]`) | afwijking 1 hieronder |
| `free_cooling_energy_is_auxiliary_energy` (`#[ignore]`) | afwijking 2 hieronder |

## Afwijkingen van de kern

1. **Buffervat in een verwarmde ruimte (9.2.3.5, p. 292).** De tekst geeft ϑ_sto;amb = ϑ_int;set;H;zi,mi van 7.9.4, het genivelleerde setpoint van 7.76. De kern neemt de rekentemperatuur ϑ_int;op;H van 7.9.2 (`space_heating_chain.rs`, regel 2238: `zones[0].operative_c[month]`). In de testcase is het buffervatverlies daardoor in de stookmaanden tot 1,9 % hoger (januari 0,857 tegen 0,844 kWh). Voorstel: daar `levelled_c` (7.76) gebruiken, zoals bij het voorraadvat van 13.58.
2. **Hulpenergie van vrije koeling (10.79/10.86, p. 420–424).** W_fc;el;in hoort bij de hulpenergie van de koudeopwekking (10.79); 10.76 geldt alleen voor compressiemachines. De kern boekt W_fc in `electricityKwh` (E_C;el) in plaats van in `auxiliaryElectricityKwh` (`space_cooling.rs`, regel 2662, en de toeslag van 10.85 op regel 2711). Het totale elektriciteitsgebruik en de primaire energie veranderen niet; de verdeling over E_C;el en W_aux (en daarmee de ISSO-uitvoerpost Waux;tot) wel. Voorstel: W_fc bij de hulpenergie optellen.

## Niet geïmplementeerd en van toepassing

- **N.81 (p. 912):** de maximale belastingsfactor van een kachel of lokale verwarmer met automatische toevoer en een aansluiting op een watergedragen systeem. De kern rapporteert Q_H;gen;out;w (N.80) alleen en begrenst de belasting niet.
- **Q.92 (p. 1069):** het forfaitaire maximale thermische vermogen van een warmtepomp zonder productgegevens; de kern vraagt productgegevens. Daarnaast is de luchtdebietinterpolatie bij tabel Q.9 (p. 1058) niet uitgevoerd.

## Dubbelzinnigheden die de uitkomst meetbaar veranderen

- **Setpoint in 9.11 en 9.31 (p. 296, 306).** Beide verwijzen naar 7.9.4.1, waar ϑ_int;set;H;zi,mi het genivelleerde setpoint van 7.76 is (woning januari 17,04 °C). De kern en r63 nemen ϑ_int;set;H;stc (20 °C). Met het genivelleerde setpoint wordt het afgifteverlies van de testwoning in januari circa 17 % groter en dalen de watertemperaturen. De ISSO 54-referentiewaarde van E_H;ci voor EP-W001a (2026, buiten de repository) ligt 0,6 % onder de kern; de genivelleerde lezing zou de kern er ruim 2 % verder van af brengen. De lezing van de kern is aangehouden.
- **f_H;red;pmp;op in de ventilatorenergie (9.22/9.23 met 9.32a).** De pompfactor 0,10 van tabel 9.X werkt via t_H,op ook op de ventilatoren van ventilatorconvectoren en luchtverwarmers in een woning met een individuele installatie: een tiende van de stookuren.
- **Q_h;b,1 (p. 300).** ϑ_int;calc;H is gedrukt als „21 °C", terwijl Q_H;ht met de werkelijke rekentemperatuur is bepaald (woning januari 16,75 °C). De kern leest 21 °C; met de rekentemperatuur wordt het luchtverwarmerhulpvermogen in de testwoning circa 9 % hoger.
- **10.23c (p. 384).** Gedrukt staat ϑ_C,in = ϑ_C,out = ϑ_C;dis;in;flw;req; de kern neemt het gemiddelde van de in- en uitkolom van tabel 10.8 (lijst `interpretations` van de koelketen). Bij 6/12 met Δϑ_int;inc −4,5 K is dat 13,5 tegen 10,5 °C.
- **Tabel 10.8 en 10.21 bij vloerkoeling.** Met Δϑ_int;inc −4,8 K wordt het medium 23,8 °C, warmer dan de onverwarmde ruimte (19 °C); 10.21 geeft dan een negatief verlies. De kern houdt het gedrukte teken aan.
- **Pomp bij een boosterwarmtepomp (10.4.2.4 met 10.9).** De pompenergie is 0 in een maand met Q_C;gen;in = 0; de kern toetst op Q_C;nd > 0. Alleen van belang als de booster de hele koudevraag van een maand afneemt.
- Bestaand en elders vastgelegd: 10.15 met MAX(…; 0,15), 10.87 bij vrije koeling (vraag 2), f_prpr bij een verklaarde EER (vraag 12), Q.86–Q.88 tegenover 13.149 (vraag 14).

## Indeling per nummer

### Hoofdstuk 9 (verwarming): formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| 9.1 | 288 | G+T | generator_dispatch_draft.rs, space_heating_chain.rs | r63, r67 | per functie afzonderlijk berekend (9.6 en 13.8); de evenredige verdeling alleen bij een WKK-combi (13.8.4.8) |
| 9.2 | 289 | G+T | generator_dispatch_draft.rs, space_heating_chain.rs | r66 |  |
| 9.3 | 290 | G+T | generator_dispatch_draft.rs, space_heating_chain.rs | r66 |  |
| 9.4 | 290 | G+T | space_heating_chain.rs | r63, r66 | booster en afleverset in r66 |
| 9.5 | 291 | G+T | space_heating_chain.rs | r63 | knooppuntwinst van zonnecombisystemen (9.2.3.4) alleen eenheidstest |
| 9.6 | 293 | G+T | space_heating_chain.rs | r63 |  |
| 9.7 | 293 | G+T | space_heating_chain.rs | r63, r75, r77 (coverage_ch9_10_independent.rs) |  |
| 9.8 | 294 | G | space_heating_chain.rs | eenheidstest | verdeling naar oppervlakte over meerdere rekenzones niet onafhankelijk nagerekend |
| 9.9 | 294 | G+T | heating_emission.rs | r63 | ϑ_H;int;ini = ϑ_int;set;H;stc (zie dubbelzinnigheden) |
| 9.10 | 295 | G+T | heating_emission.rs | r63 | ϑ_H;int;ini = ϑ_int;set;H;stc (zie dubbelzinnigheden) |
| 9.11 | 296 | G+T | heating_emission.rs | r63 | ϑ_H;int;ini = ϑ_int;set;H;stc (zie dubbelzinnigheden) |
| 9.12 | 296 | G+T | heating_emission.rs | r63 | ϑ_H;int;ini = ϑ_int;set;H;stc (zie dubbelzinnigheden) |
| 9.12a | 297 | G+T | heating_emission.rs | r63 | ϑ_H;int;ini = ϑ_int;set;H;stc (zie dubbelzinnigheden) |
| 9.21 | 298 | G+T | heating_emission.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.22 | 298 | G+T | heating_emission.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.23 | 299 | G+T | heating_emission.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.24 | 301 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.25 | 302 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.26 | 302 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.27 | 303 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.27a | 304 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.27b | 304 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.28 | 304 | G+T | monthly_demand.rs, heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.29 | 304 | G | monthly_demand.rs, heating_distribution.rs | eenheidstest | luchtterm bij mechanische toevoer niet onafhankelijk nagerekend |
| 9.30 | 306 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.31 | 306 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.32 | 306 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.32a | 308 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.32b | 308 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.33 | 311 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.34 | 311 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.35 | 312 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.36 | 312 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.37 | 312 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.38 | 313 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.39 | 313 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.40 | 314 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.41 | 314 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.42 | 314 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.43 | 315 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.44 | 315 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.45 | 317 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.45b | 318 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.46 | 319 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.47 | 319 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.48 | 320 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.49 | 320 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.50 | 320 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.51 | 321 | G+T | heating_distribution.rs, space_heating_chain.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 9.56 | 323 | G+T | space_heating_chain.rs | r66 |  |
| 9.57 | 323 | G+T | space_heating_chain.rs | r66 |  |
| 9.58 | 323 | G+T | space_heating_chain.rs | r66 |  |
| 9.59 | 323 | G+T | space_heating_chain.rs | r66 |  |
| 9.60 | 324 | G+T | space_heating_chain.rs | r66 |  |
| 9.61 | 326 | G+T | boiler_forfait_draft.rs | r63 |  |
| 9.62 | 331 | G+T | forfait_heat_pump_monthly_draft.rs | r66 |  |
| 9.63 | 340 | G+T | annex_q.rs, space_heating_chain.rs | r68, r69 |  |
| 9.64 | 341 | G+T | space_heating_chain.rs | r66 |  |
| 9.65 | 342 | G+T | space_heating_chain.rs | r66 |  |
| 9.66 | 347 | G+T | micro_chp.rs | r67 |  |
| 9.67 | 348 | G+T | micro_chp.rs | r67 |  |
| 9.68 | 348 | G+T | micro_chp.rs | r67 |  |
| 9.69 | 349 | G+T | micro_chp.rs | r67 |  |
| 9.70 | 349 | G+T | micro_chp.rs | r67 |  |
| 9.71 | 349 | G+T | micro_chp.rs | r67 |  |
| 9.72 | 349 | G+T | micro_chp.rs | r67 |  |
| 9.73 | 350 | G+T | micro_chp.rs | r67 |  |
| 9.74 | 350 | G+T | micro_chp.rs | r67 |  |
| 9.75 | 350 | G+T | micro_chp.rs | r67 |  |
| 9.76 | 350 | G+T | micro_chp.rs | r67 |  |
| 9.77 | 350 | G+T | micro_chp.rs | r67 |  |
| 9.78 | 351 | G+T | micro_chp.rs | r67 |  |
| 9.79 | 351 | G+T | micro_chp.rs | r67 |  |
| 9.80 | 351 | G+T | micro_chp.rs | r67 |  |
| 9.81 | 351 | G+T | micro_chp.rs | r67 |  |
| 9.82 | 352 | G+T | micro_chp.rs | r67 |  |
| 9.83 | 352 | G+T | micro_chp.rs | r67 |  |
| 9.84 | 356 | G+T | space_heating_chain.rs, annex_p.rs | r66 |  |
| 9.85 | 359 | G+T | space_heating_chain.rs, heating_aux_draft.rs | r63, r66, r72 |  |
| 9.86 | 360 | G+T | annex_o.rs | r75 (bijlage O) |  |
| 9.87 | 360 | G+T | annex_o.rs | r75 (bijlage O) |  |
| 9.88 | 360 | G+T | annex_o.rs | r75 (bijlage O) |  |
| 9.89 | 361 | G+T | annex_o.rs | r75 (bijlage O) |  |
| 9.90 | 361 | G+T | annex_o.rs | r75 (bijlage O) |  |
| 9.91 | 364 | G+T | space_heating_chain.rs | r66 |  |
| 9.92 | 364 | G+T | space_heating_chain.rs | r66 |  |

### Hoofdstuk 9 (verwarming): tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel 9.1 | 289 | G+T | space_heating_chain.rs | r66 |  |
| tabel 9.2 | 296 | G+T | heating_emission.rs | r63 |  |
| tabel 9.3 | 297 | G+T | heating_emission.rs | r63 |  |
| tabel 9.4 | 297 | G+T | heating_emission.rs | r63 |  |
| tabel 9.11 | 299 | G+T | heating_emission.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.12 | 300 | G+T | heating_emission.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.13 | 301 | G+T | heating_emission.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.14 | 307 | G+T | heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.15 | 308 | G+T | heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.16 | 310 | G+T | heating_distribution.rs | r72, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.18 | 313 | G+T | heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.19 | 315 | G+T | heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.20 | 316 | G+T | heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.21 | 317 | G+T | heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.22 | 320 | G+T | heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 9.23 | 324 | G+T | space_heating_chain.rs | r66 |  |
| tabel 9.24 | 325 | NVT |  |  | invoerbepaling: testcondities voor het vermogen van een warmtepomp |
| tabel 9.25 | 327 | G+T | boiler_forfait_draft.rs | r63, r66 |  |
| tabel 9.26 | 329 | G+T | boiler_forfait_draft.rs | r63, r66 |  |
| tabel 9.27 | 333 | G+T | forfait_heat_pump_draft.rs | r66, r72 |  |
| tabel 9.28 | 335 | G+T | forfait_heat_pump_draft.rs | r66, r72 |  |
| tabel 9.29 | 337 | G | forfait_heat_pump_draft.rs | eenheidstest | utiliteitskolommen; r66 rekende de woningtabel 9.27 na |
| tabel 9.30 | 341 | G+T | space_heating_chain.rs | r66 |  |
| tabel 9.31 | 343 | G+T | space_heating_chain.rs | r66 |  |
| tabel 9.32 | 344 | NVT |  |  | beschrijvende invoerlijst WKK methode 1 |
| tabel 9.33 | 345 | G+T | micro_chp.rs | r67 |  |
| tabel 9.34 | 346 | NVT |  |  | beschrijvende invoerlijst WKK methode 1 |
| tabel 9.35 | 346 | G+T | micro_chp.rs | r67 |  |
| tabel 9.36 | 353 | G+T | micro_chp.rs | r67 |  |
| tabel 9.37 | 354 | G+T | micro_chp.rs | r67 |  |
| tabel 9.38 | 354 | G+T | micro_chp.rs | r67 |  |
| tabel 9.39 | 355 | G+T | micro_chp.rs | r67 |  |

### Hoofdstuk 10 (koeling): formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| 10.1 | 369 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.2 | 369 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.3 | 369 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.3a | 369 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.3b | 369 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.4 | 370 | G | space_cooling.rs |  | alle termen 0 volgens 10.4 (L_C;zi = 0) |
| 10.5 | 371 | G+T | space_cooling.rs | r66, r68 |  |
| 10.6 | 372 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | Q_C;HP wordt opgegeven (bijlage W) |
| 10.7 | 372 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | Q_C;HP wordt opgegeven (bijlage W) |
| 10.8 | 372 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | Q_C;HP wordt opgegeven (bijlage W) |
| 10.9 | 372 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | Q_C;HP wordt opgegeven (bijlage W) |
| 10.10 | 374 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.11 | 374 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.15 | 376 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.16 | 376 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.17 | 376 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.18 | 377 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.19 | 377 | G+T | space_cooling.rs, monthly_demand.rs | r66, r77 (coverage_ch9_10_independent.rs) |  |
| 10.20 | 377 | G | space_cooling.rs, monthly_demand.rs | eenheidstest | luchtterm bij mechanische toevoer niet onafhankelijk nagerekend |
| 10.21 | 382 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.22 | 382 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.22a | 382 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.22b | 382 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.23a | 383 | G | space_cooling.rs |  | niet gebruikt: L_C;zi = 0 voor gekoelde ruimten |
| 10.23b | 383 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.23c | 384 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.24 | 384 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.25 | 385 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.26 | 385 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.27 | 386 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) | 10.26 via dezelfde code als 9.35 |
| 10.28 | 387 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.29 | 388 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.30 | 388 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.31 | 389 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.32 | 390 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.32b | 390 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.33 | 391 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.34 | 391 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.35 | 391 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.36 | 391 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.37 | 392 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.38 | 392 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.43 | 393 | G | space_cooling.rs |  | 0 omdat L_C;zi = 0 |
| 10.44 | 393 | G | space_cooling.rs |  | op 0 gesteld volgens 10.4 |
| 10.45 | 394 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.46 | 398 | G+T | space_cooling.rs | r66 | sommaties |
| 10.47 | 398 | G+T | space_cooling.rs | r66 | sommaties |
| 10.48 | 398 | G+T | space_cooling.rs | r66 | sommaties |
| 10.49 | 400 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.50 | 400 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.51 | 400 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.52 | 401 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| 10.53 | 402 | G+T | space_cooling.rs | r68 |  |
| 10.54 | 402 | G+T | space_cooling.rs | r68 |  |
| 10.55 | 402 | G+T | space_cooling.rs | r68 |  |
| 10.56 | 403 | G+T | space_cooling.rs | r68 |  |
| 10.57 | 403 | G+T | space_cooling.rs | r68 |  |
| 10.58 | 403 | G+T | space_cooling.rs | r68 |  |
| 10.59 | 403 | G+T | space_cooling.rs | r68 |  |
| 10.60 | 404 | G+T | space_cooling.rs | r68 |  |
| 10.61 | 406 | G+T | space_cooling.rs | r68 |  |
| 10.62 | 406 | G+T | space_cooling.rs | r68 |  |
| 10.63 | 408 | G+T | space_cooling.rs | r68 |  |
| 10.64 | 409 | G+T | space_cooling.rs | r68 |  |
| 10.65 | 409 | G+T | space_cooling.rs | r67 |  |
| 10.66 | 409 | G | space_cooling.rs | eenheidstest | absorptie methode 2 |
| 10.67 | 410 | G+T | space_cooling.rs | r67 |  |
| 10.68 | 412 | G+T | space_cooling.rs | r67 |  |
| 10.69 | 412 | G+T | space_cooling.rs | r67 |  |
| 10.70 | 413 | G | space_cooling.rs | eenheidstest | alleen rapportage van de dekking |
| 10.71 | 413 | G | space_cooling.rs | eenheidstest | alleen rapportage van de dekking |
| 10.72 | 413 | G | space_cooling.rs | eenheidstest | alleen rapportage van de dekking |
| 10.73 | 414 | G+T | space_cooling.rs | r67 |  |
| 10.74 | 415 | G+T | space_cooling.rs | r67 |  |
| 10.75 | 416 | G+T | space_cooling.rs | r67 |  |
| 10.76 | 417 | G+T | space_cooling.rs | r66 |  |
| 10.77 | 417 | G+T | space_cooling.rs | r66 |  |
| 10.78 | 418 | G+T | space_cooling.rs | r66 |  |
| 10.79 | 420 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) | 10.79: W_fc als opwekkerelektriciteit geboekt (afwijking) |
| 10.80 | 420 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) | 10.79: W_fc als opwekkerelektriciteit geboekt (afwijking) |
| 10.80b | 420 | G | space_cooling.rs | eenheidstest | gasmotor-compressie |
| 10.81 | 421 | G+T | space_cooling.rs | r66 | absorptie |
| 10.82 | 421 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) | 10.79: W_fc als opwekkerelektriciteit geboekt (afwijking) |
| 10.83 | 422 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) | 10.79: W_fc als opwekkerelektriciteit geboekt (afwijking) |
| 10.84 | 423 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) | 10.79: W_fc als opwekkerelektriciteit geboekt (afwijking) |
| 10.85 | 423 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) | 10.79: W_fc als opwekkerelektriciteit geboekt (afwijking) |
| 10.86 | 423 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) | 10.79: W_fc als opwekkerelektriciteit geboekt (afwijking) |
| 10.87 | 425 | G+T | space_cooling.rs | r66, r77 (coverage_ch9_10_independent.rs) | 10.79: W_fc als opwekkerelektriciteit geboekt (afwijking) |

### Hoofdstuk 10 (koeling): tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel 10.1 | 370 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.4 | 375 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.5 | 375 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.5a | 376 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.6 | 379 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.7 | 379 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.8 | 380 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.9 | 386 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.10 | 388 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.11 | 388 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.12 | 389 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.13 | 393 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.14 | 394 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.15 | 399 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.16 | 401 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.17 | 405 | G+T | space_cooling.rs | r68 |  |
| tabel 10.18 | 407 | G+T | space_cooling.rs | r68 |  |
| tabel 10.19 | 411 | G+T | space_cooling.rs, monthly_demand.rs | r67 |  |
| tabel 10.20 | 411 | NVT |  |  | systeemcodes |
| tabel 10.21 | 412 | G+T | space_cooling.rs | r67 |  |
| tabel 10.22 | 414 | G+T | space_cooling.rs | r67 |  |
| tabel 10.23 | 415 | NVT |  |  | systeemcodes |
| tabel 10.24 | 415 | G+T | space_cooling.rs | r67 |  |
| tabel 10.25 | 416 | NVT |  |  | systeemcodes |
| tabel 10.26 | 416 | G+T | space_cooling.rs | r67 |  |
| tabel 10.27 | 416 | G | space_cooling.rs | eenheidstest | natteboltemperatuur (natte koeltoren, methode 2) |
| tabel 10.28 | 417 | NVT |  |  | systeemcodes |
| tabel 10.29 | 419 | G+T | space_cooling.rs | r66 |  |
| tabel 10.30 | 419 | G+T | space_cooling.rs | r66 |  |
| tabel 10.31 | 422 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.32 | 422 | G+T | space_cooling.rs | r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.33 | 423 | G+T | space_cooling.rs | r66 |  |
| tabel 10.34 | 424 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |
| tabel 10.35 | 375 | G+T | space_cooling.rs | r66/r67, r77 (coverage_ch9_10_independent.rs) |  |

### Bijlage J: formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| J.1 | 843 | NVT |  |  | productdeclaratie λ_D/R_D door de fabrikant |
| J.2 | 843 | NVT |  |  | productdeclaratie λ_D/R_D door de fabrikant |
| J.3 | 844 | NVT |  |  | productdeclaratie λ_D/R_D door de fabrikant |
| J.4 | 844 | NVT |  |  | productdeclaratie λ_D/R_D door de fabrikant |
| J.5 | 844 | NVT |  |  | productdeclaratie λ_D/R_D door de fabrikant |

### Bijlage J: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel J.1 | 845 | NVT |  |  | productdeclaratie |

### Bijlage K: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel K.1 | 848 | NVT |  |  | schematiseringsregels voor de opname van oppervlakken |

### Bijlage L: formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| L.1 | 868 | G | window_u.rs | eenheidstest | bewijsvoorwaarde voor een thermisch verbeterde afstandhouder |

### Bijlage L: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel L.1 | 867 | G+T | window_u.rs | r75 |  |
| tabel L.2 | 869 | G+T | window_u.rs | r75 |  |

### Bijlage M: formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| M.1 | 870 | G+T | annex_m.rs | r75 |  |
| M.2 | 871 | G+T | annex_m.rs | r75 |  |
| M.3 | 871 | G+T | annex_m.rs | r75 |  |
| M.4 | 871 | G+T | annex_m.rs | r75 |  |
| M.5 | 871 | G+T | annex_m.rs | r75 |  |
| M.6 | 872 | G+T | annex_m.rs | r75 |  |
| M.7 | 872 | G+T | annex_m.rs | r75 |  |
| M.8 | 873 | G+T | annex_m.rs | r75 |  |
| M.9 | 873 | G+T | annex_m.rs | r75 |  |
| M.10 | 874 | G+T | annex_m.rs | r75 |  |
| M.11 | 875 | G+T | annex_m.rs | r75 |  |
| M.12 | 875 | G+T | annex_m.rs | r75 |  |
| M.13 | 875 | G+T | annex_m.rs | r75 |  |
| M.14 | 875 | G+T | annex_m.rs | r75 |  |
| M.15 | 876 | G+T | annex_m.rs | r75 |  |
| M.16 | 876 | G+T | annex_m.rs | r75 |  |
| M.17 | 877 | G+T | annex_m.rs | r75 |  |
| M.18 | 877 | G+T | annex_m.rs | r75 |  |
| M.19 | 877 | G+T | annex_m.rs | r75 |  |
| M.20 | 877 | G+T | annex_m.rs | r75 |  |
| M.21 | 877 | G+T | annex_m.rs | r75 |  |
| M.22 | 878 | G+T | annex_m.rs | r75 |  |
| M.23 | 878 | G+T | annex_m.rs | r75 |  |
| M.24 | 878 | G+T | annex_m.rs | r75 |  |
| M.25 | 878 | G+T | annex_m.rs | r75 |  |
| M.26 | 878 | G+T | annex_m.rs | r75 |  |
| M.27 | 878 | G+T | annex_m.rs | r75 |  |
| M.28 | 879 | NVT |  |  | invoerbepaling P_n volgens NEN-EN 12831-1 |
| M.29 | 879 | G+T | annex_m.rs | r75 |  |

### Bijlage M: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel M.1 | 870 | NVT |  |  | overzicht van grootheden |
| tabel M.2 | 872 | G+T | annex_m.rs | r75 |  |
| tabel M.3 | 873 | G+T | annex_m.rs | r75 |  |
| tabel M.4 | 874 | G+T | annex_m.rs | r75 |  |
| tabel M.5 | 876 | G+T | annex_m.rs | r75 |  |
| tabel M.6 | 877 | G+T | annex_m.rs | r75 |  |
| tabel M.7 | 878 | G+T | annex_m.rs | r75 |  |

### Bijlage N: formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| N.1 | 883 | G+T | annex_n.rs | r76 |  |
| N.2 | 883 | G+T | annex_n.rs | r76 |  |
| N.3 | 883 | G+T | annex_n.rs | r76 |  |
| N.4 | 887 | G+T | annex_n.rs | r76 |  |
| N.5 | 887 | G+T | annex_n.rs | r76 |  |
| N.6 | 887 | G+T | annex_n.rs | r76 |  |
| N.7 | 888 | G+T | annex_n.rs | r76 |  |
| N.8 | 889 | G+T | annex_n.rs | r76 |  |
| N.9 | 889 | G+T | annex_n.rs | r76 |  |
| N.10 | 889 | G+T | annex_n.rs | r76 |  |
| N.11 | 892 | G+T | annex_n.rs | r76 |  |
| N.12 | 892 | G+T | annex_n.rs | r76 |  |
| N.13 | 892 | G+T | annex_n.rs | r76 |  |
| N.14 | 893 | G+T | annex_n.rs | r76 |  |
| N.15 | 893 | G+T | annex_n.rs | r76 |  |
| N.16 | 894 | G+T | annex_n.rs | r76 |  |
| N.17 | 894 | G+T | annex_n.rs | r76 |  |
| N.18 | 894 | G+T | annex_n.rs | r76 |  |
| N.19 | 895 | G+T | annex_n.rs | r76 |  |
| N.20 | 895 | G+T | annex_n.rs | r76 |  |
| N.21 | 895 | G+T | annex_n.rs | r76 |  |
| N.22 | 895 | G+T | annex_n.rs | r76 |  |
| N.23 | 895 | G+T | annex_n.rs | r76 |  |
| N.24 | 895 | G+T | annex_n.rs | r76 |  |
| N.25 | 895 | G+T | annex_n.rs | r76 |  |
| N.26 | 895 | G+T | annex_n.rs | r76 |  |
| N.27 | 895 | G+T | annex_n.rs | r76 |  |
| N.28 | 896 | G+T | annex_n.rs | r76 |  |
| N.29 | 896 | G+T | annex_n.rs | r76 |  |
| N.30 | 896 | G+T | annex_n.rs | r76 |  |
| N.31 | 896 | G+T | annex_n.rs | r76 |  |
| N.32 | 896 | G+T | annex_n.rs | r76 |  |
| N.33 | 897 | G+T | annex_n.rs | r76 |  |
| N.34 | 897 | G+T | annex_n.rs | r76 |  |
| N.35 | 897 | G+T | annex_n.rs | r76 |  |
| N.36 | 897 | G+T | annex_n.rs | r76 |  |
| N.37 | 898 | G+T | annex_n.rs | r76 |  |
| N.38 | 898 | G+T | annex_n.rs | r76 |  |
| N.39 | 898 | G+T | annex_n.rs | r76 |  |
| N.40 | 898 | G+T | annex_n.rs | r76 |  |
| N.41 | 899 | G+T | annex_n.rs | r76 |  |
| N.42 | 899 | G+T | annex_n.rs | r76 |  |
| N.43 | 899 | G+T | annex_n.rs | r76 |  |
| N.44 | 900 | G+T | annex_n.rs | r76 |  |
| N.45 | 902 | G+T | annex_n.rs | r76 |  |
| N.46 | 902 | G+T | annex_n.rs | r76 |  |
| N.47 | 902 | G+T | annex_n.rs | r76 |  |
| N.48 | 902 | G+T | annex_n.rs | r76 |  |
| N.49 | 902 | G+T | annex_n.rs | r76 |  |
| N.50 | 902 | G+T | annex_n.rs | r76 |  |
| N.52 | 903 | G+T | annex_n.rs | r76 |  |
| N.53 | 904 | G+T | annex_n.rs | r76 |  |
| N.54 | 904 | G+T | annex_n.rs | r76 |  |
| N.55 | 904 | G+T | annex_n.rs | r76 |  |
| N.56 | 905 | G+T | annex_n.rs | r76 |  |
| N.57 | 905 | G+T | annex_n.rs | r76 |  |
| N.58 | 905 | G+T | annex_n.rs | r76 |  |
| N.59 | 905 | G+T | annex_n.rs | r76 |  |
| N.60 | 906 | G+T | annex_n.rs | r76 |  |
| N.61 | 906 | G+T | annex_n.rs | r76 |  |
| N.62 | 906 | G+T | annex_n.rs | r76 |  |
| N.63 | 906 | G+T | annex_n.rs | r76 |  |
| N.64 | 906 | G+T | annex_n.rs | r76 |  |
| N.65 | 907 | G+T | annex_n.rs | r76 |  |
| N.66 | 907 | G+T | annex_n.rs | r76 |  |
| N.67 | 907 | G+T | annex_n.rs | r76 |  |
| N.68 | 907 | G+T | annex_n.rs | r76 |  |
| N.69 | 907 | G+T | annex_n.rs | r76 |  |
| N.70 | 907 | G+T | annex_n.rs | r76 |  |
| N.71 | 907 | G+T | annex_n.rs | r76 |  |
| N.72 | 907 | G+T | annex_n.rs | r76 |  |
| N.73 | 907 | G+T | annex_n.rs | r76 |  |
| N.74 | 908 | G+T | annex_n.rs | r76 |  |
| N.75 | 908 | G+T | annex_n.rs | r76 |  |
| N.76 | 908 | G+T | annex_n.rs | r76 |  |
| N.77 | 911 | G+T | annex_n.rs | r76 |  |
| N.78 | 911 | G+T | annex_n.rs | r76 |  |
| N.79 | 911 | G+T | annex_n.rs | r76 |  |
| N.80 | 911 | G+T | annex_n.rs | r76 |  |
| N.81 | 912 | NG |  |  | maximale belastingsfactor kachel met automatische toevoer en wateraansluiting |

### Bijlage N: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel N.1 | 880 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.2 | 881 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.3 | 884 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.4 | 886 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.5 | 890 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.6 | 891 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.7 | 891 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.8 | 899 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.9 | 910 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.10 | 911 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.11 | 912 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.12 | 913 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.13 | 913 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.14 | 914 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.15 | 914 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.16 | 914 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.17 | 915 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.18 | 915 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.19 | 915 | NVT |  |  | symbolen, invoer-/uitvoerlijsten en coderingen |
| tabel N.20 | 916 | G+T | annex_n.rs | r76 |  |
| tabel N.21 | 916 | G+T | annex_n.rs | r76 |  |
| tabel N.22 | 916 | G+T | annex_n.rs | r76 |  |
| tabel N.23 | 917 | G+T | annex_n.rs | r76 |  |
| tabel N.24 | 917 | G+T | annex_n.rs | r76 |  |
| tabel N.25 | 917 | G+T | annex_n.rs | r76 |  |
| tabel N.26 | 918 | G+T | annex_n.rs | r76 |  |
| tabel N.27 | 918 | G+T | annex_n.rs | r76 |  |
| tabel N.28 | 918 | G+T | annex_n.rs | r76 |  |
| tabel N.29 | 919 | G+T | annex_n.rs | r76 |  |
| tabel N.30 | 919 | G+T | annex_n.rs | r76 |  |
| tabel N.31 | 919 | NVT |  |  | keuzeprocedure gegevensbron (invoer) |
| tabel N.32 | 919 | NVT |  |  | keuzeprocedure gegevensbron (invoer) |
| tabel N.33 | 920 | NVT |  |  | keuzeprocedure gegevensbron (invoer) |
| tabel N.34 | 922 | NVT |  |  | keuzeprocedure gegevensbron (invoer) |

### Bijlage O: formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| O.1 | 926 | NVT |  |  | meetvoorschrift opgenomen vermogen |
| O.2 | 926 | G+T | annex_o.rs | r75 |  |
| O.3 | 929 | G+T | annex_o.rs | r75 |  |

### Bijlage O: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel O.1 | 927 | NVT |  |  | illustratie (O.2 als voorbeeld in r75) |
| tabel O.2 | 930 | NVT |  |  | illustratie (O.2 als voorbeeld in r75) |

### Bijlage P: formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| P.1 | 938 | G+T | annex_p.rs | r75, r76 |  |
| P.2 | 938 | G+T | annex_p.rs | r75, r76 |  |
| P.3 | 938 | G+T | annex_p.rs | r75, r76 |  |
| P.4 | 938 | G+T | annex_p.rs | r75, r76 |  |
| P.6 | 952 | G | annex_p.rs | eenheidstest | gemeten route |
| P.7 | 952 | G+T | annex_p.rs | r75, r76 |  |
| P.8 | 952 | G+T | annex_p.rs | r75, r76 |  |
| P.9 | 954 | G+T | annex_p.rs | r75, r76 |  |
| P.10 | 954 | G+T | annex_p.rs | r75, r76 |  |
| P.11 | 954 | G+T | annex_p.rs | r75, r76 |  |
| P.12 | 954 | G+T | annex_p.rs | r75, r76 |  |
| P.13 | 957 | G+T | annex_p.rs | r75, r76 |  |
| P.14 | 958 | G+T | annex_p.rs | r75, r76 |  |
| P.15 | 958 | G+T | annex_p.rs | r75, r76 |  |
| P.16 | 959 | G+T | annex_p.rs | r75, r76 |  |
| P.17 | 959 | G+T | annex_p.rs | r75, r76 |  |
| P.18 | 959 | G+T | annex_p.rs | r75, r76 |  |
| P.19 | 964 | G+T | annex_p.rs | r75, r76 |  |
| P.20 | 964 | G+T | annex_p.rs | r75, r76 |  |
| P.21 | 964 | G+T | annex_p.rs | r75, r76 |  |
| P.22 | 964 | G+T | annex_p.rs | r75, r76 |  |
| P.23 | 967 | G+T | annex_p.rs | r75, r76 |  |
| P.24 | 967 | G+T | annex_p.rs | r75, r76 |  |
| P.25 | 968 | G+T | annex_p.rs | r75, r76 |  |
| P.26 | 974 | G+T | annex_p.rs | r75, r76 |  |
| P.27 | 975 | G+T | annex_p.rs | r75, r76 |  |
| P.28 | 976 | G+T | annex_p.rs | r75, r76 |  |
| P.29 | 976 | G+T | annex_p.rs | r75, r76 |  |
| P.30 | 977 | G+T | annex_p.rs | r75, r76 |  |
| P.31 | 978 | NVT |  |  | invoerbepaling nominaal vermogen geothermie |
| P.32 | 981 | G+T | annex_p.rs | r75, r76 |  |
| P.33 | 981 | G+T | annex_p.rs | r75, r76 |  |
| P.34 | 985 | G+T | annex_p.rs | r75, r76 |  |
| P.35 | 986 | G+T | annex_p.rs | r75, r76 |  |
| P.36 | 986 | G+T | annex_p.rs | r75, r76 |  |
| P.37 | 987 | G+T | annex_p.rs | r75, r76 |  |
| P.38 | 987 | G+T | annex_p.rs | r75, r76 |  |
| P.39 | 987 | G+T | annex_p.rs | r75, r76 |  |
| P.40 | 989 | G+T | annex_p.rs | r75, r76 |  |
| P.41 | 989 | G+T | annex_p.rs | r75, r76 |  |
| P.42 | 989 | G+T | annex_p.rs | r75, r76 |  |
| P.43 | 990 | G+T | annex_p.rs | r75, r76 |  |
| P.44 | 990 | G+T | annex_p.rs | r75, r76 |  |
| P.45 | 991 | G+T | annex_p.rs | r75, r76 |  |
| P.46 | 992 | G+T | annex_p.rs | r75, r76 |  |
| P.47 | 996 | G+T | annex_p.rs | r75, r76 |  |
| P.48 | 996 | G+T | annex_p.rs | r75, r76 |  |
| P.49 | 997 | G+T | annex_p.rs | r75, r76 |  |
| P.50 | 997 | G+T | annex_p.rs | r75, r76 |  |
| P.51 | 999 | G+T | annex_p.rs | r75, r76 |  |
| P.52 | 999 | G+T | annex_p.rs | r75, r76 |  |
| P.53 | 1000 | G+T | annex_p.rs | r75, r76 |  |
| P.54 | 1000 | G+T | annex_p.rs | r75, r76 |  |
| P.55 | 1000 | G+T | annex_p.rs | r75, r76 |  |
| P.56 | 1004 | G+T | annex_p.rs | r75, r76 |  |
| P.57 | 1005 | G+T | annex_p.rs | r75, r76 |  |
| P.58 | 1006 | G+T | annex_p.rs | r75, r76 |  |
| P.59 | 1007 | G+T | annex_p.rs | r75, r76 |  |
| P.60 | 1007 | G+T | annex_p.rs | r75, r76 |  |
| P.61 | 1009 | G+T | annex_p.rs | r75, r76 |  |
| P.62 | 1010 | G+T | annex_p.rs | r75, r76 |  |
| P.63 | 1010 | G+T | annex_p.rs | r75, r76 |  |
| P.64 | 1012 | G+T | annex_p.rs | r75, r76 |  |
| P.65 | 1012 | G+T | annex_p.rs | r75, r76 |  |
| P.66 | 1013 | G+T | annex_p.rs | r75, r76 |  |
| P.67 | 1014 | G+T | annex_p.rs | r75, r76 |  |
| P.68 | 1014 | G+T | annex_p.rs | r75, r76 |  |
| P.69 | 1016 | G+T | annex_p.rs | r75, r76 |  |
| P.70 | 1016 | G+T | annex_p.rs | r75, r76 |  |
| P.71 | 1018 | G+T | annex_p.rs | r75, r76 |  |
| P.72 | 1019 | G+T | annex_p.rs | r75, r76 |  |
| P.73 | 1019 | G+T | annex_p.rs | r75, r76 |  |
| P.74 | 1020 | G+T | annex_p.rs | r75, r76 |  |
| P.75 | 1021 | G+T | annex_p.rs | r75, r76 |  |
| P.76 | 1021 | G+T | annex_p.rs | r75, r76 |  |
| P.77 | 1022 | G+T | annex_p.rs | r75, r76 |  |
| P.78 | 1022 | G+T | annex_p.rs | r75, r76 |  |
| P.79 | 1022 | G+T | annex_p.rs | r75, r76 |  |
| P.80 | 1023 | G+T | annex_p.rs | r75, r76 |  |
| P.81 | 1024 | G+T | annex_p.rs | r75, r76 |  |
| P.82 | 1024 | G+T | annex_p.rs | r75, r76 |  |
| P.83 | 1024 | G+T | annex_p.rs | r75, r76 |  |

### Bijlage P: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel P.0 | 961 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.1 | 962 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.2 | 966 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.3 | 969 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.4 | 970 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.5 | 973 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.6 | 975 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.7 | 992 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.8 | 998 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.9 | 1002 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.10 | 1003 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.11 | 1006 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.12 | 1011 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.13 | 1017 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.14 | 1023 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.15 | 1025 | G+T | annex_p.rs | r75, r76 |  |
| tabel P.16 | 1026 | G+T | annex_p.rs | r75, r76 |  |

### Bijlage Q: formules

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| Q.1 | 1028 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.2 | 1029 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.3 | 1029 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.4 | 1029 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.5 | 1030 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.6 | 1030 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.7 | 1030 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.8 | 1031 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.9 | 1031 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.10 | 1031 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.11 | 1032 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.12 | 1032 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.12A | 1034 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.12B | 1034 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.13 | 1034 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.14 | 1034 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.15 | 1034 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.16 | 1034 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.17 | 1034 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.18 | 1035 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.19 | 1035 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.20 | 1036 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.21 | 1036 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.22 | 1036 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.23 | 1036 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.24 | 1037 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.24A | 1037 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.25 | 1037 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.26 | 1037 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.27 | 1038 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.28 | 1038 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.29 | 1039 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.30 | 1039 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.31 | 1041 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.32 | 1041 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.33 | 1041 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.34 | 1041 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.35 | 1041 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.36 | 1042 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.37 | 1042 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.38 | 1042 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.39 | 1043 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.40 | 1043 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.41 | 1043 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.42 | 1043 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.43 | 1043 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.44 | 1044 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.45 | 1045 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.46 | 1047 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.47 | 1048 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.48 | 1049 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.49 | 1049 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.50 | 1050 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.51 | 1050 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.52 | 1050 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.53 | 1050 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.54 | 1051 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.55 | 1051 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.56 | 1051 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.57 | 1051 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.58 | 1051 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.59 | 1051 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.60 | 1051 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.61 | 1052 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.62 | 1052 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.63 | 1052 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.64 | 1052 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.65 | 1052 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.66 | 1052 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.67 | 1052 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.68 | 1052 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.69 | 1053 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.70 | 1053 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.71 | 1053 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.72 | 1053 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.73 | 1054 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.73A | 1054 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.73B | 1054 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.74 | 1054 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.75 | 1054 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.76 | 1054 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.77 | 1054 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.78 | 1054 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.79 | 1055 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.80 | 1055 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.81 | 1056 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.82 | 1056 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.83 | 1056 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.84 | 1066 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.85 | 1066 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.86 | 1066 | G | annex_q.rs | eenheidstest | niet in de keten gebruikt; 13.149 (vraag 14 aan NEN) |
| Q.87 | 1066 | G | annex_q.rs | eenheidstest | niet in de keten gebruikt; 13.149 (vraag 14 aan NEN) |
| Q.88 | 1067 | G | annex_q.rs | eenheidstest | niet in de keten gebruikt; 13.149 (vraag 14 aan NEN) |
| Q.89 | 1067 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.90 | 1067 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.91 | 1069 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.92 | 1069 | NG |  |  | forfaitair maximaal vermogen zonder productgegevens |
| Q.93 | 1070 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.94 | 1070 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.95 | 1070 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.96 | 1071 | G+T | annex_q.rs | r68, r69, r70 |  |
| Q.97 | 1072 | G+T | annex_q.rs | r68, r69, r70 |  |

### Bijlage Q: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel Q.1 | 1032 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.2 | 1038 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.3 | 1040 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.4 | 1044 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.5 | 1046 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.6 | 1047 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.7 | 1049 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.8 | 1057 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.9 | 1058 | G | annex_q.rs | r69 | luchtdebietinterpolatie niet uitgevoerd |
| tabel Q.10 | 1059 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.11 | 1060 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.12 | 1061 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.13 | 1062 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.14 | 1063 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.15 | 1064 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.16 | 1064 | G+T | annex_q.rs | r68, r69, r70 |  |
| tabel Q.17 | 1068 | G+T | annex_q.rs | r68, r69, r70 |  |

### Bijlage R: tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel R.1 | 1073 | NVT |  |  | productgrenswaarden; de kern vraagt de verklaring (bmB, tabel 5.2) |
| tabel R.2 | 1074 | NVT |  |  | productgrenswaarden; de kern vraagt de verklaring (bmB, tabel 5.2) |

### Hoofdstuk 9 (verwarming): tabellen

| Nr | p. | Klasse | Kernbestand | Toets | Opmerking |
|---|---|---|---|---|---|
| tabel 9.X | 309 | G+T | heating_distribution.rs | r77 (coverage_ch9_10_independent.rs) |  |
