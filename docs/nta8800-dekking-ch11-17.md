# NTA 8800 dekking hoofdstukken 11–17 en bijlagen S–Z (review 77, deel 3 van 3)

Uitgave NTA 8800:2025+C1:2026. Per formule en tabel: geïmplementeerd en getest, geïmplementeerd maar niet getest, niet geïmplementeerd, of niet van toepassing. Alleen nummers, pagina's en kernbestanden; geen normtekst. Nieuwe onafhankelijke tests staan in `crates/nta8800-core/tests/coverage_ch11_17_independent.rs` (modules `part_a`, `part_b`, `part_c`; 50 groen, 5 `#[ignore]` voor bevestigde kernafwijkingen). Verwachte waarden komen uit een eigen herberekening vanaf de normpagina's, geschreven vóór het lezen van de formulecode van de kern; tolerantie 1·10⁻⁶ relatief.

## Samenvatting

| Deel | Bereik | + getest | niet getest | niet geïmplementeerd | n.v.t. |
|---|---|---|---|---|---|
| A | H11, H12, bijlage S | 61 regels | 2 (11.131, 12.2) | 1 (11.134) | 1 (bijlage S, informatief) |
| B | H13, bijlagen T, U, W | 249 formules/tabellen | 0 | 8 (13.151; T.3, T.7–T.11, tabel T.1) | 18 |
| C | H14–H17, bijlagen V, X, Y, Z | 61 regels | 0 | 0 | 6 |

Bevestigde afwijkingen (test met `#[ignore]`):

1. **12.1 met 11.87 bij toevoersystemen (B)**, p. 489 en 520: de kern voedt 12.1 met q_V;SUP;eff in plaats van q_V;SUP;dis;out (`src/space_heating_chain.rs:1102-1106`; q_V;SUP;dis;out wordt al bepaald in `src/ventilation.rs:2961`). Voorbeeld januari 117,69 tegen 112,09 kWh.
2. **13.168**, p. 634: de kern laat de +5 K weg (`src/domestic_hot_water.rs:3393`); stilstandsverlies van een gasvoorraadtoestel zonder gemeten waarde daardoor 35/40 van de normwaarde. Drie tests.
3. **c_W;gen bij een toestel in serie**, p. 644 met tabellen 13.26/13.27: de kern leest de klassecorrectie bij de systeemtotale Q_W;dis;nren;an in plaats van de eigen Q_W;gen;gi;out;an van het toestel (`src/domestic_hot_water.rs:4493-4502`).


## Deel A


Testbestand: `crates/nta8800-core/tests/coverage_ch11_17_independent.rs (module part_a)` (15 tests, waarvan 1 `#[ignore]` voor een kernafwijking). "r77" = nieuwe test in dat bestand; "r64/r65" = `tests/ventilation_independent.rs` en de voorbeeldprojecten; "r70" = `tests/installations_independent.rs`; "bron" = `#[cfg(test)]` in het bronbestand. Paginanummers zijn de gedrukte nummers (gelijk aan de PDF-index).

| Formule/tabel | Pagina | Status | Kernbestand | Test | Opmerking |
|---|---|---|---|---|---|
| 11.1–11.5 | 439–441 | geïmplementeerd + getest | src/ventilation.rs | r64/r65; r77 A, B, E.1 | externe druk, dichtheid, massabalans |
| 11.6–11.10 | 442–443 | geïmplementeerd + getest | src/ventilation.rs | r65 (20/49/50/60 m) | verdeling over luchtstroomzones |
| 11.11–11.21 | 443–447 | geïmplementeerd + getest | src/ventilation.rs | r64/r65; r77 A, B, E.1 (drukroutine exact gevolgd) | 11.20/11.21 som over alle paden van een soort (ook "uit"-paden die instromen) |
| tabel 11.1, 11.2, 11.3 | 430–440 | geïmplementeerd + getest | src/ventilation.rs | r64/r65 (kruipruimte, ≥ 15 m); r77 (H < 15 m) | |
| tabel 11.4 | 448 | geïmplementeerd + getest | src/ventilation.rs | r64 | temperaturen per stroom |
| 11.22 | 449 | geïmplementeerd + getest | src/ventilation.rs | r64; bron | |
| 11.23, 11.23a | 449 | geïmplementeerd + getest | src/ventilation.rs | r70; r77 F | kern past 11.23 ook in de koelbalans toe (opgegeven f_H) |
| 11.24 | 451 | geïmplementeerd + getest | src/ventilation.rs | r77 F | alleen als uitvoer; bijlage Q (Q.43/Q.95) gebruikt een opgegeven f_buitenlucht, geen koppeling |
| 11.25, 11.26 | 452 | geïmplementeerd + getest | src/ventilation.rs | r77 A (A.2b) | nieuw vastgelegd |
| 11.27 | 453 | geïmplementeerd + getest | src/ventilation.rs | r77 B (B.1) | q_SUP;eff = q_ODA;req (incl. f_lea) |
| 11.28 | 454 | geïmplementeerd + getest | src/ventilation.rs | r64 (C.1) | |
| 11.29–11.33 | 455–456 | geïmplementeerd + getest | src/ventilation.rs | r77 E.1 | 11.32 met f_BAL-DEC één keer (zie dubbelzinnigheden) |
| 11.34–11.39 | 457 | geïmplementeerd + getest | src/ventilation.rs | r77 E.1 met A.2a en B.2 | |
| 11.40–11.42 | 457 | geïmplementeerd + getest | src/ventilation.rs | r77 E.1 met C.1 | |
| 11.43–11.45 | 458 | geïmplementeerd + getest | src/ventilation.rs | r77 E.1 met D.1 | |
| 11.46–11.49 | 459 | geïmplementeerd + getest | src/ventilation.rs | r77 D (11.48), E.1 maximale capaciteit | f_sys is overal 1,00; de 11.49-tak (tabel 11.6 < 11.5) niet apart |
| tabel 11.5 | 460–462 | geïmplementeerd + getest | src/ventilation.rs | r64 (C.1, D.2); r77 (A.2a/b, B.1/2, C.1 util, D.1, D.5a util, D.5b); bron | alle rijen tekstueel gelijk aan de PDF |
| 11.50, 11.51 | 463 | geïmplementeerd + getest | src/ventilation.rs | r77 E.1 (4 varianten) | |
| 11.52 | 463 | geïmplementeerd + getest | src/ventilation.rs | r77 E.1 utiliteit | |
| tabel 11.6, 11.53, 11.54 | 464–465 | geïmplementeerd + getest | src/ventilation.rs | r77 D, E.1 maximale capaciteit | |
| tabel 11.7 | 465 | geïmplementeerd + getest | src/ventilation.rs | r64/r65 (τ_argIC, τ_argII); r77 (τ_sysC, τ_bypass, τ_C) | |
| 11.55–11.58 | 466 | geïmplementeerd + getest | src/ventilation.rs | r64 (11.56/11.57); r77 D (11.55/11.58) | |
| 11.59–11.61 | 467–468 | geïmplementeerd + getest | src/ventilation.rs | r77 D (30 %/60 % met bewijs); bron (validatie) | |
| 11.62 | 468 | geïmplementeerd + getest | src/ventilation.rs | r77 D | som 1a–1d als invoer (totaal + natuurlijk deel) |
| 11.63 | 469 | geïmplementeerd + getest | src/ventilation.rs | bron | |
| 11.64 | 469 | geïmplementeerd + getest | src/ventilation.rs | r77 B (3 woningen) | nieuw vastgelegd |
| 11.65 | 469 | geïmplementeerd + getest | src/ventilation.rs | r77 A (40 van 60 m²) | f_τ op hele woning (bestaande interpretatie) |
| tabel 11.8 | 471 | geïmplementeerd + getest | src/ventilation.rs | r64 (woon, kantoor); bron | overige rijen tekstueel gelijk; industriefunctie niet in de kern |
| tabel 11.9 | 472 | geïmplementeerd + getest | src/ventilation.rs | r64; r77 (onbekend 1,10, LUKA D, zonder kanaal; hoogste waarde bij E.1) | |
| 11.66–11.69, tabel 11.10 | 473–474 | geïmplementeerd + getest | src/ventilation.rs | r64; r77 A/B/E.1 | kinderopvang 4,8 niet apart |
| 11.70–11.79 (incl. 11.71a/b, 11.77a/b) | 476–481 | geïmplementeerd + getest | src/ventilation.rs | r65; review 52 (bron) | |
| 11.80–11.83, tabel 11.11, 11.12 | 481–484 | geïmplementeerd + getest | src/ventilation.rs | r65 | |
| 11.84–11.86 | 485 | geïmplementeerd + getest | src/ventilation.rs | r64; bron; r77 (7 rijen, renovatiejaar) | |
| tabel 11.13, 11.14 | 486–489 | geïmplementeerd + getest | src/ventilation.rs | r77 (spotcheck 8 combinaties) | |
| 11.87–11.92 | 489–491 | geïmplementeerd + getest | src/ventilation.rs | r64; r77 | |
| 11.93–11.98 | 491–492 | geïmplementeerd + getest | src/ventilation.rs | r77 E.1 (C.1- en D.1-deel) | 11.93/11.97 met 11.32/11.44: f één keer |
| 11.99 | 493 | geïmplementeerd + getest | src/ventilation.rs | r64/r65 | |
| 11.100–11.104, tabel 11.15 | 493–496 | geïmplementeerd + getest | src/ventilation.rs | r65 (LBK); bron | |
| tabel 11.16, 11.105, 11.106 | 497–498 | geïmplementeerd + getest | src/ventilation.rs | r64/r65; r77 E.1 (alleen D.5b-deel); bron | geen vorstsprong bij SUPPLY_OP (zie dubbelzinnigheden) |
| 11.106a | 498 | geïmplementeerd + getest | src/ventilation.rs | r77 E (mei–sep) | nieuw vastgelegd |
| 11.107, 11.108 | 499 | geïmplementeerd + getest | src/ventilation.rs | r64; r77 (gedeeltelijke bypass, f_lea;ahu 1,02, decentraal, constantvolume) | |
| 11.109 | 500 | geïmplementeerd + getest | src/ventilation.rs | bron (4 m); r77 (opgegeven isolatie, 0,5·H appartement) | f_prac;hr-plafond 0,9 getest |
| tabel 11.17 | 501 | geïmplementeerd + getest | src/ventilation.rs | r77 (≥ 85 %, tabelroute 0) | overige rijen niet apart |
| tabel 11.18 | 503 | geïmplementeerd + getest | src/ventilation.rs | r64 (kunststof); r77 (roterend) | overige rijen tekstueel gelijk |
| 11.110–11.113 | 503 | geïmplementeerd + getest | src/ventilation.rs | r77 D | alleen in de warmtebalans (zie dubbelzinnigheden) |
| 11.114–11.121 | 504–506 | geïmplementeerd + getest | src/ventilation.rs | r65 | c_a als gedrukt (vraag 7) |
| 11.122, tabel 11.19 | 507–508 | geïmplementeerd + getest | src/ventilation.rs | r65; bron | |
| 11.123–11.128 | 509–511 | geïmplementeerd + getest | src/ventilation.rs | r65 | |
| 11.129, 11.130 | 511 | geïmplementeerd + getest | src/ventilation.rs | r65 | |
| 11.131 | 512 | geïmplementeerd, niet getest | src/building_performance.rs | – | som over rekenzones; eenvoudige optelling |
| 11.132 | 512 | geïmplementeerd + getest | src/ventilation.rs | r64; r77 | gebruikt q_ODA;req van de warmtebalans (interpretatie) |
| 11.133 | 513 | geïmplementeerd + getest | src/ventilation.rs | r77 D, opgegeven ventilatoren (f_gebouw 0,8/0,5) | |
| 11.134 | 514 | niet geïmplementeerd | – | – | opgegeven ventilatoren gelden per rekenzone; verdeling van een gedeelde ventilator naar q-aandeel ontbreekt |
| 11.135, 11.136 | 515 | geïmplementeerd + getest | src/ventilation.rs | bron; r77 (gemeten 0,75 exact, afronding) | |
| tabel 11.20 | 516 | geïmplementeerd + getest | src/ventilation.rs | bron; r77 (6 rijen) | |
| 11.137, 11.138, tabel 11.21 | 516–517 | geïmplementeerd + getest | src/ventilation.rs | bron (tabel 11.21); r77 (opgegeven f_regfan per maand) | |
| tabel 11.22 | 517 | geïmplementeerd + getest | src/ventilation.rs | bron; r77 (4 cellen × f_τ) | |
| 11.139–11.141 | 517–518 | geïmplementeerd + getest | src/ventilation.rs | r77 E.1 (f_systype per deel 0/1/2) | |
| 11.142, tabel 11.23 | 518–519 | geïmplementeerd + getest | src/ventilation.rs | r64; bron; r77 (6 rijen) | |
| 12.1 | 520 | geïmplementeerd + getest | src/humidification.rs; src/space_heating_chain.rs | bron; r77 (zorg, warmtewiel); r77 ignore | **afwijking** in de keten bij SUPPLY_OP (zie onder) |
| 12.2 | 521 | geïmplementeerd, niet getest | src/building_performance.rs | – | boeking per zone en drager, eenvoudige som |
| 12.3, 12.4 | 521–522 | geïmplementeerd + getest | src/humidification.rs | bron; r77 (> 500 m² en 400 m²) | |
| tabel 12.1 | 522 | geïmplementeerd + getest | src/humidification.rs | bron; r77 (algemeen en zorg, alle maanden) | woonfunctie 0 |
| 12.5, tabel 12.2 | 523–524 | geïmplementeerd + getest | src/humidification.rs | bron (12/16, 17/21); r77 (6/12, 12/18) | |
| bijlage S (S.1–S.2.5, figuren S.1–S.24) | 1075–1091 | n.v.t. | – | – | informatief: toelichting op de systeemvarianten van tabel 11.5; geen formules of tabellen |

### Aantallen (rijen hierboven)

- geïmplementeerd + getest: 61 (waarvan 12.1 met een afwijking in de projectketen)
- geïmplementeerd, niet getest: 2 (11.131, 12.2)
- niet geïmplementeerd: 1 (11.134)
- n.v.t.: 1 (bijlage S)

### Niet geïmplementeerd en relevant voor BENG/label

- 11.134 (p. 514): het effectieve vermogen van ventilatoren van de thermische zone wordt naar rato van q_ODA;req over de rekenzones verdeeld. De kern rekent opgegeven ventilatoren per rekenzone; het gebouwtotaal klopt alleen als de gebruiker een gedeelde ventilator één keer (of zelf verdeeld) invoert. Geen controle op dubbeltelling.
- (koppeling) 11.24 (p. 451) wordt berekend maar niet doorgegeven aan bijlage Q (Q.43/Q.95 verwijzen naar 11.2.2.1.2); daar is f_buitenlucht een opgave met bewijs.

### Bevestigde afwijking

- **12.1 (p. 520–521) met 11.87 (p. 489), SUPPLY_OP.** 12.1 neemt q_V;SUP;dis;out "zoals bepaald in 11.3.1.1"; voor B-systemen is dat 11.87: q_ODA;req/(f_lea;du·f_lea;ahu). De keten geeft `heating.mechanical_supply_m3_per_h` door (`src/space_heating_chain.rs:1102-1106`), dat bij SUPPLY_OP q_V;SUP;eff = q_ODA;req is (`src/ventilation.rs:2868-2872`). Kantoor B.1, 100 m², stoombevochtiger elektrisch, januari: kern 117,692 kWh, norm 112,088 kWh (factor f_lea;du·f_lea;ahu = 1,05). Voorstel: per balans q_V;SUP;dis;out (zoals al voor 9.29/10.20 in `src/ventilation.rs:2961-2962` berekend) in `BalanceFlows` opnemen en die in de keten aan 12.1 geven. Test: `humidification_uses_the_supply_flow_of_11_87_for_a_supply_system` (ignore).

### Dubbelzinnigheden met meetbaar effect

1. **11.32/11.44 met 11.93/11.97 (p. 456, 458, 491–492).** 11.93 geeft q_ETA;dis;in al met f_BAL-DEC, 11.32 vermenigvuldigt nog eens met f_BAL-DEC (idem 11.97/11.44 met f_overig). De kern past de factor één keer toe. Letterlijk (twee keer) gaf in r77 E.1 januari een afvoer van 119,9 in plaats van 156,6 m³/h en een referentiedruk van −5,29 in plaats van −5,75 Pa. NEN-vraag waard.
2. **Vorstbeveiliging bij SUPPLY_OP (p. 495 tegenover p. 497).** 11.3.2.1.1 noemt elke mechanische toevoer; de legenda bij 11.103 zet ΔT_preh = 0 voor SUPPLY_OP en BALANCED_OP zonder WTW. De kern volgt de legenda (geen sprong, geen 11.105-energie). Anders: +0,2 K en ca. 14 kWh/maand (B.1, 3 woningen, 284 m³/h) in jan, feb, okt–dec.
3. **12.1 en q_SUP bij SUPPLY_OP:** 11.2.2.2.2 (q_SUP;eff = q_ODA;req) en 11.87 (gedeeld door f_lea) spreken elkaar tegen; zie de afwijking hierboven (factor 1,05–1,12).
4. **Recirculatie in de koelbalans (11.111, p. 503).** "f_terugregel = 1 bij koudebehoefte" noemt het product; de kern zet ook f_terugregel;recirculatie op 1 (geen recirculatiesprong in de koelbalans). Letterlijk met 0,7: toevoertemperatuur juli 21,49 in plaats van 20,55 °C (r77 D).
5. **Ventilatorenergie bij maximale benutting voor koeling (11.132/11.142).** De kern gebruikt q_ODA;req van de warmtebalans (bestaande interpretatie); met τ_sysC is q_ODA;req in de koelbalans veel hoger (r77 D juli 2 273,7 tegen 639,8 m³/h). Bij forfaitaire ventilatoren maakt de keuze verschil.
6. **11.23 in de koelbalans:** de kern past de overventilatie ook in de koelbalans toe (r77 F: 180,4 tegen 159,2 m³/h); 11.23 noemt geen balans, f_H;τ;hp-on is een verwarmingsfractie.

### Toegevoegde tests (`tests/coverage_ch11_17_independent.rs (module part_a)`)

| Test | Bepaling | Pagina |
|---|---|---|
| natural_a2b_split_dwelling_with_reference_infiltration | 11.1–11.21, 11.25/11.26, 11.65, 11.86, tabellen 11.13/11.14 | 439–487 |
| mechanical_supply_b1_apartment_zone | 11.2.2.2.2, 11.27, 11.64, 11.3.2.7, 11.142 | 452–519 |
| combined_e1_decentral_d5b_with_c1 | 11.29–11.33, 11.40–11.42, 11.50/11.51, 11.87/11.88, 11.93–11.96, 11.105–11.109, tabel 11.17, 11.139–11.141 | 455–518 |
| combined_e1_with_maximum_capacity_for_cooling | 11.48, 11.53/11.54, tabel 11.6/11.7 | 459–465 |
| combined_e1_with_natural_supply_or_balanced_other_part | 11.34–11.39, 11.43–11.45, 11.97/11.98, tabel 11.9 (hoogste) | 457–492 |
| combined_e1_utility_control_factor | 11.52 | 463 |
| utility_installed_capacity_recirculation_and_declared_fans | 11.48, 11.55–11.61, 11.107, 11.110–11.113, 11.133/11.135, tabellen 11.20/11.22 | 459–517 |
| cold_recovery_with_full_bypass_and_apartment_duct_length | 11.106a, 11.109 (0,5·H) | 498–500 |
| overventilation_and_outdoor_air_fraction | 11.23/11.23a, 11.24 | 449–451 |
| reference_infiltration_rows_of_tables_11_13_and_11_14 | 11.85/11.86, tabellen 11.13/11.14 | 485–489 |
| declared_fans_motor_rows_and_monthly_regulation | 11.133, 11.135, 11.136, tabel 11.20, f_gebouw | 513–516 |
| forfait_fan_rows_of_table_11_23 | 11.142, tabel 11.23 | 518–519 |
| utility_fan_regulation_rows_of_table_11_22 | tabel 11.22 × f_τ | 517 |
| humidification_and_dehumidification_spot_rows | 12.1, 12.3, 12.4, 12.5, tabellen 12.1/12.2 | 520–524 |
| humidification_uses_the_supply_flow_of_11_87_for_a_supply_system (ignore) | 12.1 met 11.87 | 489, 520 |

## Deel B


Paginanummers zijn gedrukte paginanummers (PDF-pagina = gedrukte pagina, gecontroleerd op p. 553). "Nieuw" = test toegevoegd in `tests/coverage_ch11_17_independent.rs (module part_b)` (review 77). "Bouwerstest" = in-source `#[cfg(test)]` in het genoemde bestand. "R63/R66/R67/R70" = onafhankelijke herberekening uit eerdere reviews met vaste test (`tests/installations_independent.rs` e.a.). DHW = `src/domestic_hot_water.rs`.

Nummers die in de norm niet voorkomen (gecontroleerd met pdftotext per pagina en visueel op p. 531 en 560): 13.5, 13.30, 13.45, 13.48, 13.61, 13.62, 13.70, 13.71, 13.147, 13.155, 13.171–13.173, 13.176–13.178, 13.180; tabellen 13.5 en 13.19–13.21. 13.166/13.167 staan wel in de norm (p. 634, als verwijzing naar NEN-EN 15316-4-1). Ze zijn hieronder niet als formule meegeteld.

| Formule/tabel | Pagina | Status | Kernbestand (src/xxx.rs) | Test | Opmerking (kort, geen normtekst) |
|---|---|---|---|---|---|
| 13.1 | 528 | geïmplementeerd + getest | DHW (HotWaterMonth per drager), building_performance.rs | R63, R66; nieuw (gas/elek per drager in alle ketentests) | som per drager |
| 13.1a | 529 | geïmplementeerd + getest | DHW:4175–4185 (f_building) | nieuw `collective_circulation_share_of_one_dwelling`; bouwerstest | f_gebouw |
| 13.2 | 529 | geïmplementeerd + getest | DHW (auxiliary_electricity_kwh) | nieuw (alle ketentests), R66 | som hulpenergie |
| 13.3 | 530 | geïmplementeerd + getest | DHW:3309 e.v. (book_generator) | nieuw, R63, R66, R67 | |
| 13.4, 13.4a, 13.6 | 531 | geïmplementeerd + getest | DHW:4437–4465 | R67; bouwerstest `solar_preheater_is_used_first_per_13_4` | zon eerst, begrensd op Q_W;dis |
| 13.7 | 531 | geïmplementeerd + getest | DHW:4423–4460 | nieuw (circulatie, afleversets), R66 | |
| 13.8 | 531 | geïmplementeerd + getest | DHW, solar_thermal.rs | R67 (naverwarmerdeel) | × f_gebouw |
| 13.9, 13.9a | 532 | geïmplementeerd + getest | DHW:4203–4206 | nieuw, R63, R66 (EPW403), bouwerstest | DWTW na η_W;em |
| 13.10–13.12 | 533–534 | geïmplementeerd + getest | DHW:4176–4200 (annual_need, unsplit) | bouwerstest `shower_recovery_uses_the_dwelling_need_under_a_13_19a_split` | |
| 13.13, 13.14 | 534–535 | geïmplementeerd + getest | DHW:4129–4147, 4338, 4584; building_performance.rs:3145 | nieuw (circulatie A/C, afleversets U, gasboiler); bouwerstests | 500 m²-regel: zie dubbelzinnigheid 2 |
| 13.15–13.18 | 537–538 | geïmplementeerd + getest | monthly_demand.rs (occupants_per_dwelling), DHW:2878 | nieuw `residential_need_13_15_to_13_18`; R63 | |
| 13.19 | 538 | geïmplementeerd + getest | DHW:2878 | nieuw (utiliteit), R63 | |
| Tabel 13.1 | 539 | geïmplementeerd + getest | DHW:92 | nieuw `tables_13_1_and_13_7_per_function` (alle rijen) | |
| 13.19a | 540 | geïmplementeerd + getest | DHW:1615 | nieuw `need_fractions_13_19a`; building_performance-bouwerstest | |
| 13.19b, 13.19c | 540 | geïmplementeerd + getest | DHW (A_g;si;W = context-oppervlakte) | nieuw (circulatie B) | definitie; zie dubbelzinnigheid 1 |
| 13.20, 13.20a | 541 | geïmplementeerd + getest | DHW:2029, 4212 | bouwerstest `utility_circulation_uses_the_served_area_of_13_20a` | |
| 13.21–13.23 | 542 | geïmplementeerd + getest | DHW:2900 | bouwerstest `emission_follows_13_23_and_table_13_2`; nieuw (13.23); R63 | |
| Tabel 13.2 | 543 | geïmplementeerd + getest | DHW:122–168 | nieuw `table_13_2_bands`; edition_switches_independent (2023-rijen) | grens 2,0 m in hogere band |
| Tabel 13.3 | 544 | geïmplementeerd + getest | DHW:2900 | nieuw (0,8 bij 4 m, 1,0 bij 2,5 m) | |
| 13.24, 13.24a | 546 | geïmplementeerd + getest | DHW:76–78 en afleversetblok | nieuw `utility_delivery_sets_13_24a`, `residential_delivery_sets_13_24`; R66 | |
| 13.25 | 548 | geïmplementeerd + getest | DHW:4423 e.v. | nieuw (4 circulatietests) | |
| 13.26, 13.26a | 549–550 | geïmplementeerd + getest | DHW:4208–4296 | nieuw (4 circulatietests) | 15 % onverwarmd bij 13 °C, c_ls;kb 0,03/0,15 |
| 13.27–13.29 | 551 | geïmplementeerd + getest | heating_distribution.rs:424–520 (PipeGeometry) | nieuw `calculated_psi_13_27_to_13_29`; bouwerstest 13.28 | h_a 8,0 vast |
| Tabel 13.4 | 553 | geïmplementeerd + getest | DHW:249 | nieuw `table_13_4_rows_and_nearest_diameter` (alle cellen, dichtstbijzijnde, gelijke afstand); edition_switches_independent | |
| Tabel 13.29 | 553 | geïmplementeerd + getest | DHW:4233–4250 | nieuw (band > 20, ook uit A_gebouw/A_tot); bouwerstest review52 (alle banden) | |
| 13.31, 13.32, 13.32a | 554 | geïmplementeerd + getest | DHW:4224–4290 | nieuw (forfaitaire lengte, sporthal ≥ 1 000 m²) | zie dubbelzinnigheid 1 |
| 13.33 | 555 | geïmplementeerd + getest | DHW:4432 | nieuw | |
| 13.34–13.38 | 556–557 | geïmplementeerd + getest | DHW:4299–4325 | nieuw (4 circulatietests) | |
| 13.39 | 558 | geïmplementeerd + getest | DHW:4306 | nieuw `utility_circulation_with_sport_hall_and_label_power` | |
| 13.40, 13.41 | 559 | geïmplementeerd + getest | DHW:4308 | nieuw (circulatie A/B) | |
| 13.42 | 559 | geïmplementeerd + getest | DHW:4312 | nieuw `large_circulation_pump_above_2_5_kw` | was ongetest |
| 13.43, 13.44 | 558–559 | geïmplementeerd + getest | DHW:4320–4325 | nieuw | |
| Tabel 13.6 | 558 | geïmplementeerd + getest | DHW:4314 | nieuw (beide rijen) | |
| 13.46 | 560 | geïmplementeerd + getest | DHW:80, 4432 | nieuw, R66 | 10 W per set |
| 13.47 | 560 | geïmplementeerd + getest | DHW:4584 | nieuw (0,85 en f_conv;rbl 1) | |
| 13.49, 13.50 | 561 | geïmplementeerd + getest | DHW:82 | nieuw | |
| 13.51–13.53 | 563 | geïmplementeerd + getest | DHW:2856, 4190 | R66 (EPW403a–d), bouwerstest | |
| 13.54–13.57 | 564 | n.v.t. | — | — | voorbeelden van 13.53 |
| Tabel 13.7 | 564 | geïmplementeerd + getest | DHW:106 | nieuw (alle rijen) | |
| Tabel 13.8 | 565 | geïmplementeerd + getest | DHW:484 | nieuw (alle rijen), R66 | |
| 13.58 | 567 | geïmplementeerd + getest | DHW:4330–4390 | R66 (gemeten, kokendwaterkraan 90 °C); bouwerstest | |
| 13.59 | 567 | geïmplementeerd + getest | DHW:4356–4382 | nieuw (labels B, C, D, A, onbekend vanaf 2018), R63, R66 | S_conn label C buiten A+/A/B |
| 13.60 | 568 | geïmplementeerd + getest | DHW:424 | R66, bouwerstest (afronding bijlage X) | |
| 13.60a | 569 | n.v.t. | — | — | zonder effect: bij gemeten H_sto;ls geldt f_sto;dis;ls = 1 |
| Tabel 13.9 | 570 | geïmplementeerd + getest | DHW:375 | nieuw (alle labels bij 200 l), R63, R66 | |
| 13.63 | 570 | geïmplementeerd + getest | DHW:4338 | nieuw (hotfill, gasboiler), bouwerstest | |
| 13.64, 13.65 | 571 | n.v.t. | — | — | W_W;aux;sto = 0 (13.6.4.2), dus altijd 0 |
| 13.66 | 574 | geïmplementeerd + getest | solar_thermal.rs, DHW:3874 | R67 | f_prac;sol 0,95, f_PVT;th |
| 13.66a | 574 | geïmplementeerd + getest | DHW, solar_thermal.rs | bouwerstest `solar_combi_splits_by_f_use_and_feeds_the_node` | verwarmingsdeel alleen bouwerstest |
| 13.67, 13.68 | 575 | geïmplementeerd + getest | DHW:4450–4460 | R67 (13.67), bouwerstest | |
| 13.69, 13.69a | 576 | geïmplementeerd + getest | solar_thermal.rs, DHW:3703 | R67 (13.69), bouwerstest `solar_storage_ambient_follows_13_69_and_13_69a` | |
| 13.72–13.84 | 576–582 | geïmplementeerd + getest | solar_thermal.rs:113–270, 716–830 | R67, bouwerstests | |
| Tabellen 13.10, 13.11 | 578 | geïmplementeerd + getest | solar_thermal.rs:37–41 | R67 | |
| Tabel 13.12 | 579 | geïmplementeerd + getest | solar_thermal.rs (ServiceSettings) | R67 | |
| 13.85–13.91 | 583–586 | geïmplementeerd + getest | solar_thermal.rs:743–790 | bouwerstest; building_performance-bouwerstests | alleen bouwerstest (geen onafhankelijke herberekening) |
| Tabel 13.13 | 583 | geïmplementeerd + getest | solar_thermal.rs | bouwerstest | idem |
| 13.92–13.109 | 586–593 | geïmplementeerd + getest | solar_thermal.rs:716–830 | R67 | |
| Tabellen 13.14, 13.15 | 589–591 | geïmplementeerd + getest | solar_thermal.rs:33, 113 | R67 | |
| 13.110–13.127 | 594–600 | geïmplementeerd + getest | solar_thermal.rs:716–830 | bouwerstests | alleen bouwerstest |
| 13.128–13.140 (incl. 13.137a/b) | 600–604 | geïmplementeerd + getest | solar_thermal.rs:842–975 | bouwerstests `tested_preheater_interpolates_and_distributes`, `tested_backup_flags_a_distribution_below_one`; DHW `tested_solar_system_outside_its_range_is_rejected` | alleen bouwerstest; 13.134 letterlijk (interpretatie) |
| Tabel 13.16 | 604 | geïmplementeerd + getest | solar_thermal.rs:380 | nieuw `pvt_reduction_table_13_16` | was ongetest |
| 13.141 | 611 | geïmplementeerd + getest | DHW:4002–4120 | nieuw (P_nom, f_func 0,6) | |
| 13.141a | 611 | geïmplementeerd + getest | DHW:4058–4100 | nieuw `hotfill_electric_boiler_after_a_combi` | c_W;gen-basis: afwijking 2 |
| 13.141b–13.141d | 611 | geïmplementeerd + getest | DHW:4058–4100 | nieuw `collective_series_with_f_pref`; bouwerstest | |
| 13.142 | 612 | geïmplementeerd + getest | DHW:2345, 4087 | nieuw (externe warmte zonder P_nom), R66 | |
| 13.143a, 13.144b, 13.145 | 613 | geïmplementeerd + getest | DHW:4113 | nieuw `three_generators_dispatch_by_category_and_efficiency`, `declared_generator_share_is_interpolated` | inclusief volgorde 13.8.2.1 |
| 13.144a | 613 | geïmplementeerd + getest | DHW:1505, 4047 | nieuw `exhaust_air_heat_pump_with_f_combi_and_extra_electric_heater`; bouwerstest | extra doorstroomtoestel p. 614 |
| 13.146 | 614 | geïmplementeerd + getest | DHW:1471, 4077 | nieuw `declared_generator_share_is_interpolated`; bouwerstest | buiten bereik: kern houdt eindwaarde aan |
| 13.148, 13.148a | 615 | geïmplementeerd + getest | building_performance.rs:3051–3140 | bouwerstest `exhaust_air_hot_water_heat_pump_feeds_13_148_and_13_149_into_chapter_11`; R70 (13.148a) | |
| 13.149 | 615 | geïmplementeerd + getest | DHW:3181, 4534 | nieuw (twee tests) | |
| 13.150 | 616 | geïmplementeerd + getest | DHW:3158 | nieuw | |
| 13.151 | 616 | niet geïmplementeerd | — | — | t_W;use wordt nergens verder gebruikt; geen effect op BENG/label |
| 13.152 | 616–617 | geïmplementeerd + getest | DHW:2997, 3040, 1313 | nieuw (1,0 forfait; 0,95 meting/NEN-EN 89), bouwerstest | |
| Tabel 13.17 | 620 | geïmplementeerd + getest | DHW:742–756 (ProfileTest) | nieuw (twee-profieltests) | invoerdefinities |
| 13.153 | 621 | geïmplementeerd + getest | DHW:1050 | nieuw `test_profile_energies_and_corrections` | hulpfunctie, niet in de keten (Q_gas;p is invoer) |
| 13.153a | 621 | geïmplementeerd + getest | DHW:1183 | nieuw (gas twee profielen) | |
| 13.153b, 13.153c | 622 | geïmplementeerd + getest | DHW:1183, 1932 | nieuw (SCF, T_max;test) | |
| 13.153d–13.153i | 623–624 | geïmplementeerd + getest | DHW:1064–1096 | nieuw `mixed_air_correction_per_month`; bouwerstest in de keten | |
| 13.154, 13.154a | 625 | geïmplementeerd + getest | DHW:1241–1257 | nieuw (incl. `gas_appliance_below_the_lowest_profile_13_154a`) | |
| 13.156a, 13.156b | 627 | geïmplementeerd + getest | DHW:1200 | nieuw `gas_combi_with_vessel_and_pfhrd` | |
| 13.157, 13.158 | 627–628 | geïmplementeerd + getest | DHW:3335–3355, 1259 | nieuw; bouwerstest f_gebouw | |
| 13.159, 13.160 | 628 | geïmplementeerd + getest | DHW:1322, 3667 | nieuw | |
| 13.160a | 629 | geïmplementeerd + getest | DHW:1339, 3676 | nieuw (afvoerlucht-wp en combi met vat) | met ongecorrigeerde testenergie |
| 13.160b | 630 | geïmplementeerd + getest | DHW:3059 | bouwerstest; edition_switches_independent `switch_12_table_13_18_interpolation` | |
| Tabel 13.18 | 630 | geïmplementeerd + getest | DHW:1953 | nieuw `class_correction_tables`; edition_switches_independent | |
| 13.161 | 631 | geïmplementeerd + getest | annex_w.rs, DHW:3521 | R67 | |
| 13.162, 13.163 | 632–633 | geïmplementeerd + getest | annex_w.rs | R66 | |
| 13.164 | 633 | geïmplementeerd + getest | DHW:3533 | bouwerstest (DHW:4953) | alleen bouwerstest |
| 13.165–13.170, 13.174, 13.175 | 634–636 | geïmplementeerd + getest, afwijking | DHW:3362–3398, 3079 | nieuw (3 tests, #[ignore]); bouwerstest | 13.168 zonder +5 K: afwijking 1 |
| Tabel 13.22 | 637 | geïmplementeerd + getest | DHW:2960 | nieuw `tables_13_22_and_13_28_rows` (alle rijen) | |
| 13.179 | 639 | geïmplementeerd + getest | DHW:3512, 4568 | nieuw (extra doorstroomtoestel), R66 | |
| Tabel 13.23 | 640 | geïmplementeerd + getest | DHW:585, annex_w.rs (BoosterClass) | nieuw (klassen in 13.26/13.27), R67 (W.14) | |
| Tabel 13.24 | 640 | n.v.t. | — | — | informatief (Gaskeur-koppeling) |
| Tabel 13.25 | 641 | geïmplementeerd + getest, één rij niet | DHW:620, 3020–3060 | R63, R66, opname_defaults_independent, nieuw (0,675, 0,95, 1,4) | rij "(combi)toestel met microWKK" (c_W;gen × ε_W;chp;th) niet als tabelrij |
| Tabellen 13.26, 13.27 | 643–644 | geïmplementeerd + getest | DHW:1896, 1909 | nieuw `class_correction_tables`; R63, R66 | |
| Tabel 13.28 | 645 | geïmplementeerd + getest | DHW:3091–3106 | nieuw (alle rijen + olie), R66 | |
| 13.181 | 647–648 | geïmplementeerd + getest | DHW:3420 | nieuw (10 W, 1 W/kW), R66 | |
| 13.182, 13.183 | 649–650 | geïmplementeerd + getest | DHW:3573, 3637 | bouwerstest `hot_water_micro_chp_follows_13_182_and_13_183` | alleen bouwerstest |
| 13.184 | 653 | geïmplementeerd + getest | building_performance.rs | bouwerstest `hot_water_from_the_heating_system_is_split_per_13_184` | |
| 13.185 | 652–653 | geïmplementeerd + getest | DHW:3329 | bouwerstests | |
| Tabel T.1 | 1095 | niet geïmplementeerd | — | — | alleen voor T.8 (bivalente wp met externe bijstook); bewust niet aangeboden |
| Tabellen T.2–T.6 | 1096–1099 | n.v.t. | — | — | tappatronen voor de laboratoriumproef |
| T.1, T.2 | 1101–1102 | n.v.t. | — | — | laboratoriumprocedure (Q_gewenst) |
| T.3 | 1102 | niet geïmplementeerd | hot_water_tests.rs (kop) | — | elektrisch toestel; bewust niet aangeboden (elektrische rendementen op geleverde energie) |
| T.4, T.5 | 1102–1103 | geïmplementeerd + getest | hot_water_tests.rs:55, 202 | nieuw `annex_t_efficiencies`; bouwerstest | |
| T.6 | 1103 | geïmplementeerd + getest | hot_water_tests.rs | nieuw | |
| T.7–T.11 | 1104–1105 | niet geïmplementeerd | — | — | bivalente wp met externe bijstook en microWKK; bewust niet aangeboden (route via verklaring 13.146 of Chp) |
| Tabel T.7 | 1105 | geïmplementeerd + getest | hot_water_tests.rs:45 | nieuw (aardgas, propaan, butaan) | |
| T.12–T.17 | 1106–1108 | geïmplementeerd + getest | hot_water_tests.rs | nieuw `annex_t_efficiencies`; bouwerstests | |
| Tabel U.1 | 1110 | geïmplementeerd + getest | hot_water_tests.rs:315 | nieuw `annex_u_efficiency` | |
| U.1 | 1112 | n.v.t. | — | — | laboratoriumprocedure |
| U.2–U.6 | 1112–1113 | geïmplementeerd + getest | hot_water_tests.rs:292–460 | nieuw `annex_u_efficiency`; bouwerstests | gemiddelde van drie runs, naar beneden op 0,025 |
| W.1, W.2 | 1120 | geïmplementeerd + getest | annex_w.rs | R67 | |
| W.3 | 1120 | geïmplementeerd + getest | annex_w.rs:372 | bouwerstest `cooling_source_reduces_the_heating_system_heat` | alleen bouwerstest |
| W.4–W.7 | 1121–1122 | geïmplementeerd + getest | annex_w.rs:390–420 | bouwerstest `booster_cooling_indication_follows_w4_to_w7` | ter indicatie, geen BENG-effect |
| W.8–W.15 | 1123–1125 | geïmplementeerd + getest | annex_w.rs | R67 | methode 3 van W.3.3 |
| W.16, W.17 | 1126–1127 | n.v.t. | annex_w.rs (invoer) | — | laboratoriumbepaling; COP_ki;j en P_ls zijn invoer uit het testrapport |

### Telling

Per afzonderlijke formule of tabel (nummers die niet in de norm staan, tellen niet mee). Hoofdstuk 13 heeft 202 formules (incl. 13.166/13.167) en 25 tabellen; bijlage T 17 formules en 7 tabellen; bijlage U 6 formules en 1 tabel; bijlage W 17 formules. Samen 275.

| Klasse | H13 formules | H13 tabellen | Bijlage T | Bijlage U | Bijlage W | Totaal |
|---|---|---|---|---|---|---|
| geïmplementeerd + getest | 194 | 24 | 10 (T.4–T.6, T.12–T.17, tabel T.7) | 6 (U.2–U.6, tabel U.1) | 15 | 249 |
| waarvan met bevestigde afwijking (test #[ignore]) | 13.165–13.175 (via 13.168); 13.141a/b en tabellen 13.26/13.27 (c_W;gen-basis bij serie) | | | | | 2 regels |
| geïmplementeerd, niet getest | 0 | 0 | 0 | 0 | 0 | 0 (13.42, 13.27–13.29 en tabel 13.16 waren ongetest; nu getest) |
| niet geïmplementeerd | 1 (13.151) | 0 (één rij van tabel 13.25) | 7 (T.3, T.7–T.11, tabel T.1) | 0 | 0 | 8 |
| n.v.t. | 7 (13.54–13.57, 13.60a, 13.64, 13.65) | 1 (tabel 13.24) | 7 (T.1, T.2, tabellen T.2–T.6) | 1 (U.1) | 2 (W.16, W.17) | 18 |

"Getest" telt ook bouwerstests mee, zoals de opdracht voorschrijft. Alleen een bouwerstest, zonder onafhankelijke herberekening: zonnecombi verwarmingsdeel (13.66a, 13.85–13.91, 13.110–13.127, tabel 13.13), geteste zonneboilers (13.128–13.140), 13.164, 13.182/13.183, 13.184/13.185, W.3–W.7.

## Deel C


Testbestand : `crates/nta8800-core/tests/coverage_ch11_17_independent.rs (module part_c)` (7 tests, groen, rustfmt- en clippy-schoon).
Testnamen afgekort: T1 = `lighting_installed_power_daylight_and_occupancy_14_7_to_14_44`, T2 = `lighting_forfait_power_weighted_tables_14_1_14_3_14_4`, T3 = `lighting_table_rows_and_occupancy_branches`, T4 = `annex_x_table_x1_classes_over_five_decades`, T5 = `pvt_electricity_table_16_4_bounds_and_orientation_midway`, T6 = `climate_table_17_1_columns_and_table_17_2_orientation_rule`, T7 = `annex_v_regeneration_degree_and_correction`.

| Formule/tabel | Pagina | Status | Kernbestand (src/xxx.rs) | Test | Opmerking (kort, geen normtekst) |
|---|---|---|---|---|---|
| 14.1–14.3 (woonfunctie) | 655 | geïmplementeerd + getest | building_performance.rs, lighting.rs | in-source `building_performance` (residential_lighting_must_be_omitted) | W_L;spec = 0 en W_P = 0: verlichtingsinvoer bij woning geweigerd |
| 14.4 (Σ A_g;vz = A_g) | 656 | geïmplementeerd + getest | lighting.rs (validate_lighting) | in-source validatietest; T1, T2 | samenvoegregel 70 % is schematisering door gebruiker, geen kerncontrole |
| 14.5–14.7 | 657 | geïmplementeerd + getest | lighting.rs | in-source; T1, T2 (nieuw: meerdere verlichtingszones, gewogen t_D/t_N) | maandverdeling t_mi/t_an is kerninterpretatie |
| Tabel 14.1 | 658 | geïmplementeerd + getest | lighting.rs (burning_hours) | T3 alle rijen; T1/T2 oppervlaktegewogen | |
| 14.8 (incl. f_dyn, afronding bijlage X) | 659 | geïmplementeerd + getest | lighting.rs | in-source; T1 (nieuw: f_dyn 0,9, afronding na f_dyn) | |
| 14.9 | 659 | geïmplementeerd + getest | lighting.rs | in-source (T5); T1 alle technologieën | |
| Tabel 14.2 | 660 | geïmplementeerd + getest | lighting.rs (LampTechnology) | T3 alle rijen | |
| 14.10–14.12 (incl. afronding P_em, P_pc) | 660–661 | geïmplementeerd + getest | lighting.rs | in-source; T1 (8,2→8,5; 0,95→1,0; 3,0→3,2) | |
| 14.13 | 661 | geïmplementeerd + getest | lighting.rs | in-source; T2 (gewogen, beide kolommen) | geen afronding voorgeschreven, kern rondt niet af |
| Tabel 14.3 | 662 | geïmplementeerd + getest | lighting.rs (specific_power) | T3 alle rijen incl. voetnoot a; T2 | |
| 14.14 | 662 | geïmplementeerd + getest | lighting.rs | opname_defaults_independent; in-source; T2 | |
| 14.15 (F_C) | 663 | geïmplementeerd + getest | lighting.rs | in-source (2022-route); T1/T2 met F_C = 1 | MF = F_CC = 1 in 2025+C1 |
| 14.16–14.17 | 664 | geïmplementeerd + getest | lighting.rs | in-source (waarschuwing); T1 (centrale aan-regeling, grote kantoorgroep) | |
| 14.18–14.19 | 664–665 | geïmplementeerd + getest | lighting.rs (occupancy_factor) | in-source; T1, T2, T3 | |
| 14.20 | 665 | geïmplementeerd + getest | lighting.rs (occupancy_factor) | T3 (nieuw) | met tabel 14.4 (F_A ≤ 0,5) in label/BENG niet bereikbaar |
| 14.21–14.22 | 665 | geïmplementeerd + getest | lighting.rs | in-source; T1, T2 | |
| 14.23 | 665 | geïmplementeerd + getest | lighting.rs | T3 (nieuw) | als 14.20 niet bereikbaar met tabel 14.4 |
| Tabel 14.4 | 666 | geïmplementeerd + getest | lighting.rs (absence_factors) | T3 alle rijen; T1/T2 gewogen | |
| Tabel 14.5 | 666 | geïmplementeerd + getest | lighting.rs (SwitchControl) | T3 alle rijen | |
| 14.24 | 667 | geïmplementeerd + getest | lighting.rs | in-source; opname_defaults_independent; T2 | |
| 14.25–14.26 | 667 | geïmplementeerd + getest | lighting.rs | in-source; T1 (twee sectoren) | |
| 14.27–14.29 | 668–669 | geïmplementeerd + getest | lighting.rs (sector_geometry) | in-source; T1 | breedte volgens figuur 14.1 is invoer |
| 14.30 (alternatief) | 669 | geïmplementeerd + getest | lighting.rs | T1 (nieuw) | |
| 14.31–14.35 | 670 | geïmplementeerd + getest | lighting.rs | in-source; T1 | breedte op dezelfde manier als diepte |
| 14.36 (+ alternatief NEN-EN 15193-1 bijlage F) | 671–672 | geïmplementeerd + getest | lighting.rs | in-source (U602g/h); T1 | |
| Tabel 14.6 | 672 | geïmplementeerd + getest | lighting.rs (DaylightControl) | in-source; T3 | |
| 14.37–14.40 | 672 | geïmplementeerd + getest | lighting.rs | in-source; T1 (I_Sh 0,2) | |
| Tabel 14.7 | 673 | geïmplementeerd + getest | lighting.rs (vertical_daylight_supply) | in-source; T3 (interpolatie op 11 punten) | onder 0,13 % vastgehouden (gedocumenteerde interpretatie) |
| 14.41 | 673 | geïmplementeerd + getest | lighting.rs | in-source; T1 | ×100 (gedocumenteerde interpretatie) |
| 14.42 | 674 | geïmplementeerd + getest | lighting.rs | in-source; T1 (k tussen tabelpunten) | |
| Tabel 14.8 | 674 | geïmplementeerd + getest | lighting.rs (rooflight_supply) | in-source; T3 (klassegrenzen) | |
| Tabel 14.9 | 674 | geïmplementeerd + getest | lighting.rs (rooflight_utilance) | in-source; T3 (interpolatie) | |
| 14.43–14.44 | 675 | geïmplementeerd + getest | lighting.rs, building_performance.rs (lighting_context) | T1 (nieuw; met en zonder daglichtregeling) | eerder alleen herrekend (3 okt), niet vastgepind |
| §15.1–15.3 (geen formules) | 676 | n.v.t. | — (f_BACS: building_performance.rs, bacs_draft.rs) | f_BACS in indicators_independent (5.20a/b) | alleen beschrijvend; f_BACS hoort bij 5.5.8 (buiten deze scope) |
| 16.0 | 677 | geïmplementeerd + getest | building_performance.rs | installations_independent (PV+batterij, 5.10) | som van PV, PVT, WKK, wind |
| 16.1–16.4b, f_prac;PV | 677–682 | geïmplementeerd + getest | pv.rs | installations_independent (review 66); in-source; T5 | |
| Tabel 16.1 | 680 | geïmplementeerd + getest | pv.rs | opname_defaults_independent (kristallijn); T5 (dunne film) | |
| Tabel 16.2 | 681 | geïmplementeerd + getest | pv.rs | opname_defaults_independent; T5 | |
| Tabel 16.3 | 681 | geïmplementeerd + getest | pv.rs (shading_correction) | installations_independent; T5 (0,87 → 0,848) | onder 0,80 vastgehouden op 0,75 (gedocumenteerd) |
| 16.5–16.8 | 683 | n.v.t. | — | — | nulbepalingen (geen thermische output, hulpenergie, verliezen); kern heeft geen term |
| 16.9–16.10 | 684 | geïmplementeerd + getest | pv.rs | in-source; T5 (nieuw, onafhankelijk) | |
| Tabel 16.4 | 685 | geïmplementeerd + getest | pv.rs (PvtCover) | in-source; T5 (grenzen 0,015 en 0,03) | |
| 16.11–16.13 | 686 | geïmplementeerd + getest | building_performance.rs, space_heating_chain, domestic_hot_water.rs | installations_independent (16.12); in-source dhw (16.13) | |
| 16.14–16.16 | 687 | geïmplementeerd + getest | micro_chp.rs, domestic_hot_water.rs | installations_independent (16.15); in-source dhw (16.16) | kernnaam "method1" = normmethode 2 |
| 16.17 (wind = 0) | 688 | geïmplementeerd + getest | building_performance.rs | in-source (final_energy_sums_carriers_and_wind_is_zero) | opgegeven windproductie geweigerd |
| Tabel 17.1 | 690 | geïmplementeerd + getest | climate.rs | envelope_independent (t_mi, θ_e); T6 (alle kolommen) | |
| θ_e;argII-definitie (17.2) | 689 | n.v.t. | climate.rs | — | afleiding uit NEN 5060-uurwaarden; kern gebruikt tabelwaarden 17.1 |
| Tabel 17.2 (incl. oriëntatie- en hellingsregel) | 691–694 | geïmplementeerd + getest | climate.rs (irradiance_at) | envelope/solar_independent; T6 (middenoriëntaties, helling 37,5°) | zie ambiguïteit 1 |
| 17.3.1–17.3.2, tabel 17.3 | 694–707 | geïmplementeerd + getest | solar_shading.rs | solar_independent (review 75) | |
| Tabel 17.4–17.6 | 708–715 | geïmplementeerd + getest | solar_shading_tables.rs | solar_independent | |
| Tabel 17.7 | 716–720 | geïmplementeerd + getest | solar_shading_tables.rs | solar_independent | |
| Tabel 17.8–17.9 | 720–730 | geïmplementeerd + getest | solar_shading_tables.rs | solar_independent | |
| Tabel 17.10–17.11 | 731–743 | geïmplementeerd + getest | solar_shading_tables.rs | solar_independent | |
| Tabel 17.12 | 744–751 | geïmplementeerd + getest | solar_shading_tables.rs | solar_independent | |
| Tabel 17.13–17.14 | 752–759 | geïmplementeerd + getest | solar_shading_tables.rs | solar_independent | |
| Tabel 17.15 | 760–762 | geïmplementeerd + getest | solar_shading_tables.rs | solar_independent; installations_independent | |
| 17.3.8 (uitgebreide uurmethode) | 763–765 | n.v.t. | solar_shading.rs (opgegeven factoren met bron) | — | NEN 5060-uurwaarden niet beschikbaar; alternatief, niet verplicht |
| Tabel V.1 | 1114 | geïmplementeerd + getest | annex_v.rs | in-source; T7 | |
| V.1–V.3 | 1114–1115 | geïmplementeerd + getest | annex_v.rs, building_performance.rs | in-source; T7 (met en zonder vrije koeling) | η_H;gen-verwijzing naar 14.6 (gedocumenteerd) |
| V.4–V.5 | 1115 | geïmplementeerd + getest | annex_v.rs | in-source; T7 (vijf F_sh-waarden, helling 37,5°) | /1000 voor kWh (dimensioneel) |
| Tabel V.2, V.6 (+ afronding opgegeven rendement) | 1116 | geïmplementeerd + getest | annex_v.rs | in-source; T7 (interpolatie, 0,47 → 0,44) | |
| Tabel V.3 | 1117 | geïmplementeerd + getest | annex_v.rs (functie); keten via opgegeven c_source | in-source | ketenkoppeling als opgegeven waarde (1,00/1,02/1,04 met bron) |
| X.2, tabel X.1 | 1129–1130 | geïmplementeerd + getest | significant_figures.rs, lighting.rs (round_up_annex_x) | in-source (alle voorbeelden); T4 (alle klassen, 6 decaden, beide routines) | twee implementaties van naar boven afronden; T4 bewaakt gelijkheid |
| Bijlage Y verticale projectie (A1/A2) | 1131 | geïmplementeerd + getest | lighting.rs (annex_y_projection) | in-source | |
| Bijlage Y horizontale projectie, > 105° | 1132 | geïmplementeerd + getest | lighting.rs | in-source | |
| Tabel Z.1 | 1133 | n.v.t. | building_performance.rs (vaste waarden) | in-source (tabel 14.1-factor alleen maatwerkadvies) | overzicht van beleidsfactoren; geen rekenregel |
| Tabel Z.2 | 1134 | n.v.t. | — | — | overzicht van paragrafen in H9/11/13 (andere scope) |

### Aantallen

- geïmplementeerd + getest: 61 rijen
- geïmplementeerd, niet getest: 0 (alle eerder ongepinde posten zijn in T1–T7 vastgelegd: 14.20, 14.23, 14.30, 14.43–14.44, f_dyn, rijen tabel 14.1–14.5, tabel 16.4-grenzen, tabel 17.1-kolommen, oriëntatieregel 17.2, V.1–V.6 onafhankelijk, X.1 volledig)
- niet geïmplementeerd: 0
- n.v.t.: 6 rijen (§15, 16.5–16.8, θ_e;argII-afleiding, 17.3.8, tabel Z.1, tabel Z.2)

### Ambiguïteiten

1. Tabel 17.2, middenoriëntatie gecombineerd met hellingsinterpolatie (p. 679, 694): de kern neemt per hellingskolom de hoogste naastliggende waarde en interpoleert daarna. Andersom (eerst per oriëntatie interpoleren, dan de hoogste) geeft bijvoorbeeld bij 202,5°/37,5° in juni 205,0 i.p.v. 205,25 W/m² (0,12 %). Klein, alleen bij PV/collector/regeneratie met een vrij azimut exact midden tussen twee kolommen.
2. 14.44 (p. 675): ΣA_g en ΣA_w "van alle rekenzones op het perceel"; de kern gebruikt de gebruiksoppervlakte en ramen van het beoordeelde gebouw(deel). Bij een deelberekening kan de verhouding afwijken.
3. Bekend en gedocumenteerd: 14.41 ×100, tabel 14.7 onder 0,13 %, tabel 16.3 onder 0,80, V.1 η_H;gen-verwijzing naar 14.6 en η_W;gen naar "19.7".

## Dubbelzinnigheden met meetbaar effect

Deel A (H11/H12):

1. 11.32/11.44 met 11.93/11.97: f_BAL-DEC (of f_overig) staat in beide; de kern past de factor één keer toe. Dubbel toegepast: E.1-voorbeeld januari afvoer 119,9 in plaats van 156,6 m³/h. Kandidaat voor een NEN-vraag.
2. Vorstbeveiliging bij toevoersystemen (B): 11.3.2.1.1 (p. 497) tegenover de legenda van 11.103 (p. 495, ΔT_preh = 0); de kern volgt de legenda.
3. 11.111 in de koelbalans: de kern zet f_terugregel;recirculatie ook daar op 1 (letterlijk 0,7: julitoevoertemperatuur 21,49 in plaats van 20,55 °C).
4. 11.132/11.142: ventilatorenergie met q_ODA;req van de verwarmingsbalans (bestaande lezing); bij maximale koelcapaciteit is het koeldebiet veel hoger.
5. 11.23 (overventilatie) ook in de koelbalans toegepast.
6. 11.134 (p. 514, verdeling van gedeclareerde ventilatoren over rekenzones) ontbreekt; een gedeelde ventilator moet één keer worden ingevoerd. 11.24 wordt wel berekend maar niet door bijlage Q gebruikt (f_buitenlucht is daar invoer).

Deel B (H13):

1. 13.31/13.32 bij een woning op een collectief systeem (p. 554): de kern bepaalt de forfaitaire circulatielengte uit A_g van het gebouw en schaalt daarna met f_gebouw (in lijn met 13.38); met A_g;si;W van de woning zou de lengte veel kleiner zijn. Nog geen vermelding in `INTERPRETATIONS`; kandidaat voor een NEN-vraag.
2. 500 m²-regel van 13.13: een systeem voor het hele woongebouw zonder `collective` rekent als individuele toestellen; alleen terugwinbare verliezen veranderen, zonder effect op BENG bij woningen.
3. 13.146 boven de hoogste gedeclareerde klasse houdt de eindwaarde aan; bij parallelle opwekkers c_W;gen op het systeemtotaal.

Deel C (H14–H17):

1. Tabel 17.2 bij een azimut precies tussen twee kolommen in combinatie met interpolatie over de helling (p. 679, 694): de kern neemt per hellingsrij de hogere kolom en interpoleert daarna; andersom 0,12 % verschil (`src/climate.rs:273-318`).
2. 14.44 (p. 675): de kern gebruikt het oppervlak en de ramen van het beoordeelde gebouw, niet alle zones op het perceel; verschil bij een deelbeoordeling.
3. Bekende lezingen blijven: 14.41 × 100, tabel 14.7 onder D = 0,13 %, tabel 16.3 op 0,75 onder F_sh;obst 0,80, verouderde verwijzingen in V.1, V.5 gedeeld door 1000.

## Alleen bouwerstests, geen onafhankelijke herberekening

Zonnecombi verwarmingsdeel (13.66a, 13.85–13.91, 13.110–13.127, tabel 13.13), geteste zonneboilers (13.128–13.140), 13.164, 13.182/13.183, 13.184/13.185, W.3–W.7, 11.131 en 12.2 (eenvoudige sommen in `src/building_performance.rs`).
