# Voorbereiding op de testset ISSO 54

Dit document legt vast hoe de rekenkern klaarstaat voor de testset van BRL 9501 (ISSO-publicatie 54). Zodra ISSO 54 versie 5.0:2026 met de verwachte uitkomsten er is, moeten we alleen nog de verwachte waarden invullen. De gate draait de deeltesten al.

Zie ook de [BRL 9501-gereedheid](nta8800-brl9501-gereedheid.md) §2, het [referentieprotocol](nta8800-referentieprotocol.md) en het [attestdossier](nta8800-attestdossier.md).

## Bron

- **ISSO 54 versie 2.0** "Testen voor het deelgebied EDR attest energieprestatie", vastgesteld door het CCvD van InstallQ op 12-05-2022, voor NTA 8800 (januari 2022). Openbaar via documenten.isso.nl.
  - Hoofdstuk 2 beschrijft de woningtesten (referentietest EP-W001, p. 4–6), hoofdstuk 3 de utiliteitstesten (EP-U001, p. 44) en hoofdstuk 4 de realistische gebouwen.
  - Hoofdstuk 5 (p. 53) noemt de uitvoer die een programma moet tonen.
  - Bijlage 1 (p. 54–66) somt de te attesteren tests op.
- **De verwachte uitkomsten** staan in bijlage 2, een apart resultatendocument. Dat is niet in bezit.
- **De afkeurgrens** is 1,0 % afwijking naar boven of beneden per deeltest (bijlage 2, p. 67).
- **De invoer van de realistische gebouwen** staat in de bijlagen 3A–3K, ook aparte documenten. Die zijn evenmin in bezit.

De pdf wordt niet in de repository opgenomen. De invoer is in eigen woorden gecodeerd, met test-id en pagina.

## Hoe het werkt

| Onderdeel | Plaats |
|---|---|
| Referentiewoning EP-W001 als project, NTA 8800:2022 | `training-data/isso54/EPW001.json` |
| Referentiekantoor EP-U001 (EP-W001 als kantoor met koeling) | `training-data/isso54/EPU001.json` |
| Deeltesten als patch op de referentie | `training-data/reference-suites/isso54-v2.json`, gegenereerd door `scripts/isso54-suite.py` |
| Uitvoering in elke gate | `reference_gate` via `scripts/verify-nta.sh` (alle suites in `training-data/reference-suites/`) |
| Oordeel zonder verwachtingen | Een run met alleen deze suite vergelijkt niets: het rapport zegt "geen vergelijking (0 vergeleken, 227 zonder verwachting)" en de afsluitcode is 3, niet 0. In de gate lopen de openbare gevallen en de RVO-woningen mee; die vergelijken wel. |

- **Patches.** Een deeltest verandert een of enkele kenmerken van de referentie. De suite geeft daarom per deeltest een `projectPatch` (`set`/`remove` op een JSON-pointer) op het referentieproject, in plaats van een volledig projectbestand. Een patch waarvan het doel ontbreekt, laat de gate falen, dus een wijziging van de referentie kan een deeltest niet stilletjes leegmaken.
- **Zonder verwachting.** Zonder bijlage 2 heeft elke deeltest een blok `pending` in plaats van `expected`:
  - de reden;
  - de vast te leggen indicatoren: BENG 1, BENG 2, BENG 3 en, voor woningen, TOjuli;max;
  - de band van 1 % die straks geldt.
  
  De gate rekent de deeltest door en legt de uitkomsten vast met status `pending_expectation`. In het rapport staat dan "geen verwachting", zonder oordeel. Zo'n geval laat de gate slagen, maar bewijst niets over de uitkomst.
- **Wel een harde eis.** Een deeltest die niet rekent (status `calculation_unavailable` of `invalid_case`), laat de gate falen. Zo blijft elke gecodeerde deeltest rekenbaar.
- **Geen vermenging.** Een geval met `pending` mag geen `expected` en geen verwachte labelklasse hebben (`pending_with_expected_values`).

## Interpretaties bij het coderen

Deze keuzes volgen uit de testbeschrijving of uit NTA 8800:2022. Ze zijn het eerst te controleren zodra de verwachte uitkomsten er zijn.

- **EP-W001 afgifte.** Vloerverwarming met een deklaag van 1,8 cm valt onder "systeem met dunne deklaag (< 2 cm)" van tabel 9.4 (2022 p. 275). De inregeling per veld met dynamische groepsbalans is de rij "statisch per paneel met dynamische groepsbalans" van tabel 9.2. De regeling per ruimte geeft Δθ_roomaut −0,5 K, zoals de test noemt. De regelaars zijn niet als gecertificeerd (NEN-EN 215/15500) opgegeven, omdat de test dat niet zegt.
- **EP-W001 distributie.** "Geen aanvullende pomp" is de pomp in de ketel (`included_in_generator_auxiliary`). De leidingen zijn geïsoleerd vanaf 1995: bouwjaar 2021, en de referentieregel van EP-W202 (p. 24) noemt ze geïsoleerd.
- **EP-W001 infiltratie.** Forfaitair met tabel 11.13, rij "plat dak, vrijstaand". Dat geeft de q_v10 van 0,7 en f_type 1,4 van de test.
- **EP-W001 ventilatie.**
  - De 3-standenschakelaar is niet apart ingevoerd: de forfaitaire ventilatorroute (tabel 11.23) heeft daar geen invoer voor.
  - "Maximale benutting van de ventilatiecapaciteit" is als passieve koeling vastgelegd, zonder extra geïnstalleerde capaciteit, want de test noemt er geen.
  - Bij systemen A, B en C (EP-W101) is dat weggelaten, omdat passieve koeling via het systeem mechanische toevoer vraagt.
- **EP-W001 tapwater.** Tapklasse CW5 is toepassingsklasse 4 (tabel 13.24, 2022 p. 617). De uittapleiding naar de keuken is > 10 mm, dus de rij "overig" van tabel 13.2.
- **EP-W007a/b.** De test laat het bouwjaar 1950 ook gelden voor de ketel, de ventilatoren en de isolatie van de verwarmingsleidingen. Gecodeerd:
  - het installatiejaar van de ketel en het fabricagejaar van de ventilatoren op 1950;
  - de leidingisolatie in de periode "vóór 1980".
- **EP-U001 koeling.** De test noemt vloerkoeling met ontwerptemperatuur 12/16. NTA 8800:2022 tabel 10.8 opmerking 2 (p. 361) schrijft voor een afgiftesysteem van alleen vloerkoeling 17/21 voor, en de kern dwingt dat af (`cooling_design_temperature_inconsistent`). Gecodeerd is daarom 17/21.
- **EP-U001 verlichting.** "LED, vermogen forfaitair": NTA 8800:2022 heeft geen aparte LED-kolom (die is van 2024), dus de forfaitaire waarde van de functie.
- **EP-W003a/c en EP-W016.** De U-waarde is met de formules van `window_u` berekend en afgerond volgens 8.2.2.1 (één decimaal boven 1,0):
  - EP-W003a: 8.15, max(0,7·2,0 + 0,3·3,4; 0,8·2,0 + 0,2·3,4) + 2,5·0,11 = 2,695 → 2,7;
  - EP-W003c: U_W+shut = 1/(1/1,8 + 0,2) = 1,324 (8.23), effectief 0,5·1,8 + 0,5·1,324 = 1,562 → 1,6 (8.22, f_shut;with 0,5). Alleen de extra weerstand varieert; het rolluik is niet als zonwering ingevoerd;
  - EP-W016: de deur is een raamelement van 2 m² in de noordgevel (bruto 43,2, netto 41,2 m²), opaak U 3,4 (8.20) en g 0. Met glas is U oppervlaktegewogen over glas en deur (geen ψ opgegeven): 3,1 (50 %) en 2,9 (80 %). De kozijnfractie is projectbreed 0,25, dus het glasaandeel staat in een equivalente g: 0,7 · aandeel / 0,75.
- **EP-W012.** Zonwering die van binnenuit wordt bediend, is `manual_residential` (tabel 7.7). In NTA 8800:2022 telt die zonwering ook op de verwarmingsbalans (tabel 7.7, maart t/m oktober); pas vanaf 2024 is f_sh;with daar 0 voor woningen. Een donker screen verhoogt daarom BENG 1.
- **EP-W404.** f_sto;dis;ls volgt 2022 p. 549–550: T-stukken geïsoleerd 2, ongeïsoleerd 5, gemeten H_sto;ls 1, elektroboiler met geïsoleerde warmwaterleiding 1,5 (`electricBoilerInsulatedPipe`). EP-W404c rekent de stilstandsproef (2 kWh bij 60/20 °C) via 13.60.
- **EP-W405.**
  - Het opgegeven oppervlak is de referentieoppervlakte van de collector. Collector- en vatgegevens die niet zijn gegeven, zijn forfaitair (tabel 13.14).
  - EP-W405d noemt een vat van 150 l met een back-updeel van 150 l. Het zonnedeel is dan 0, zoals de forfaitaire waarde van 13.80 onder 80 l ook geeft, en de zonne-opbrengst is vrijwel nul (BENG 3 1,2 %).
  - EP-W405d noemt het "toestel voor tapwater": een indirect gestookt vat dat niet ook verwarmt. EP-W405e zegt "tapwater en verwarming".
- **EP-W203/EP-U303 warmtepompen.**
  - "Voldoet aan tabel 9.28" vraagt meetwaarden; de test geeft die niet, dus de punten staan net boven de minima van tabel 9.28 (fictief, als zodanig vermeld).
  - De bron is tabel 9.27/9.29 met c_source 1,0 bij bodem en grondwater voor woningen; bij utiliteit (tabel 9.29) geldt die voetnoot niet.
  - Elke warmtepomp heeft het bewijs voor hernieuwbare energie van bijlage P (`heatPumpRenewable`).
  - Bij retourluchtwarmtepompen (EP-W203p, EP-U303c) is het ventilatiesysteem C1 zonder passieve koeling via het systeem.
- **EP-W203a, EP-W204c.** Lokale toestellen zonder watergedragen distributie: afgifte `local_heater` zonder waterzijdige inregeling, en een opgegeven distributieverlies van 0. De kern heeft geen aparte invoer "geen distributiesysteem".
- **EP-W204a, EP-W204e.** Bij warmtelevering en een biomassaketel hoort de distributiepomp niet bij de hulpenergie van 9.85. De pomp is berekend (9.41–9.51) met onbekend vermogen en EEI.
- **EP-U601/602.** Het lampvermogen is één groep (14.9, tabel 14.2). Nieuwwaardecompensatie is tabel 14.4 van 2022 (`constantIlluminance`). EP-U602a "centraal aan" is handbediend met centrale aanschakeling.
- **EP-U002j.** 30 % kantoor en 70 % overige bijeenkomst in één rekenzone. Ventilatie, tapwater en verlichting zijn per oppervlak verdeeld. De gebruiksfunctie van de zone, voor setpoints en interne warmte, is de grootste: overige bijeenkomst.
- **Referenties: geen leidingen in onverwarmde ruimten.** EP-W001 en EP-U001 noemen geen verwarmingsleidingen door onverwarmde ruimten, maar de codering liet `unheatedPipeLengthM` leeg. Dan neemt 9.36 15 % van de lengte als onverwarmd. Gecorrigeerd naar 0 (9 oktober 2026): BENG 2 van EP-W001 daalt van 105,18 naar 105,07. EP-W202a (lengte onbekend) is nu de variant met 15 %.
- **EP-W201b–d.** Rij van de overtemperatuur in tabel 9.3 (2022 p. 273) naar de gemiddelde watertemperatuur min 20 °C: 55/47 is 31 K (rij 30 K), 50/42 is 26 K (rij 30 K), 90/70 is 60 K. Bij convectoren met boosterventilatoren de rij "radiatoren met ventilator". De 40 W per ventilator vraagt de route van NEN-EN 16430, die NTA 8800:2022 niet kent (schakelpunt 56): gerekend is met 10 W per ventilator uit tabel 9.11. De gemiddelde ontwerptemperatuur van de ketel volgt de ontwerpklasse.
- **Ventilatie EP-W101c, r, s, w.**
  - EP-W101c: het bouwjaar 1985 werkt volgens de test op infiltratie, de ventilatoren en de isolatieperiode van de verwarmingsleidingen (1980–1995), niet op de ketel.
  - EP-W101r: verklaring van 85 % met dissipatie verdisconteerd, als NEN-EN 13141-7. EP-W101s: 85 % volgens NEN-EN 13142, zonder dissipatie.
  - EP-W101w: E.1 met D.5b op 40 m² en C.1 op 30 m² verblijfsgebied, zonder bypass, zonder passieve koeling via het systeem.
- **EP-W102b.** q_v;inst 60 dm³/s natuurlijke toevoer, waarvan 30 dm³/s (108 m³/h) voorverwarmd.
- **EP-W103a/b.** De vier ramen van de zuidgevel, elk 2,5 m² netto (8,33 m² × 0,30 gaas), openingshoek 90°, middens op 1,2 m en 3,9 m, openingshoogte 2 m. De voorwaarden van 11.2.3.3 (inbraak, insecten, regen) zijn als vervuld aangenomen; de test noemt ze niet.
- **Raamposities EP-W008–W010.** Uit de figuren 2–6: ramen van 3,0 × 2,0 m, borstwering 0,5 m, verdiepingshoogte 2,7 m, horizontaal 0,5–3,5 m en 4,5–7,5 m. Raammiddens op 1,5 m en 4,2 m. `raam-zuid-1`/`-3` zijn als de linker ramen (beneden, boven) genomen, `-2`/`-4` als de rechter. Links en rechts zijn gezien van binnen naar buiten (west is rechts).
  - EP-W008a–c: overstek direct boven elk raam, h_o = 1,0 / diepte (2022 p. 672).
  - EP-W008d/e: het doorlopende dak (5,4 m) over de linker 4 m dekt alleen de linker ramen.
  - EP-W009a/b: b_b = 1,5 / diepte aan de westzijde (rechts). EP-W009c–f: één belemmering aan de oostzijde (links), 0,5 m naast de rechter ramen: b_b = 2,0 / diepte rechts en 6,0 / diepte links (p. 673).
  - EP-W010: h_b = (h − raammidden) / x (p. 671); boven het midden van een raam is er geen belemmering (minimaal). De koelvoorwaarden van tabel 17.14 noemt de test niet, dus tabel 17.5 bij EP-W010e.
- **Koeling EP-W301–303.** De gekoelde referentiewoning neemt de koeling van EP-U001 over: vloerkoeling, inregeling en regeling onbekend, geïsoleerde leidingen, pomp, geen warmtemeter, individuele compressiekoelmachine, 17/21 (tabel 10.8 opmerking 2).
  - De afgiftetabellen zijn die van 2023 (2022 p. 354–356). EP-W301b/d: ventilatorconvectoren, dus 12/16 (b) en 12/18 (d). Het aantal convectoren van EP-W301b is niet gegeven (n_fan 0); bij EP-W301d is 8 W één convector van 10 W (10.18).
  - EP-W303e: EER 4,2 gemeten volgens NEN-EN 14825 zonder deellastpunten, ingevoerd als opgegeven rendement (§10.1) in plaats van tabel 10.29.
- **Appartement (EP-W202f, EP-W205, EP-W302f, EP-W402).** De referentiewoning als hoekappartement op de bovenste verdieping van een gebouw van 4 woningen en 4 bouwlagen, 10,8 m hoog: de begane-grondvloer vervalt, infiltratie tabel 11.14 "gestapeld, hoek, boven", woningtype woongebouw. Het collectieve vloeroppervlak is 4 × 96 m² (EP-W205c: 10 woningen).
  - EP-W205: de HR107 is als voorkeursopwekker genomen (9.6.1), de VR-ketels daarna, elk 10 kW.
  - EP-W402: de afleverset van de test hoort bij warmtelevering (§13.4.2) en is bij een ketel niet ingevoerd; zo ook bij EP-U502a.
- **EP-W204b/i.** Warmtelevering met een kwaliteitsverklaring (bijlage P): f_P 0,5, f_Pren 0,3, K_CO2 0,1, b gemeten (`measuredOnly`), i berekend. De distributiepomp valt buiten 9.85 en is berekend.
- **EP-W204f/g.** Micro-WKK onder 2 kW (tabel 9.31; elke waarde tot 2 kW is dezelfde rij). Het thermisch vermogen voor 9.91 is niet gegeven: 6 kW aangenomen (fictief, als zodanig vermeld).
- **EP-W203g.** Lucht/luchtwarmtepomp met luchtverwarming: geen watergedragen distributie, distributieverlies 0.
- **EP-W406.**
  - d, u, v: ventilatiewarmtepomp van 2 kW met C1. Bij v ook voor verwarming (f_combi 1 in oktober–maart, 13.144a). Afvoerlucht telt niet als hernieuwbare bron (5.37), dus BENG 3 is 0.
  - m, n: twee tapwatersystemen verdeeld naar aangesloten keukens en badkamers (13.19a). Bij n is het tweede toestel "gesloten toestel met Gaskeur en CW4" als waterverwarmer met Gaskeur CW gelezen.
- **Geometrie (EP-W002, W006, W013, W014, W103c, EP-U202, U502c).** Een deeltest die het gebouw verbouwt, staat in de generator als gewijzigde kopie van de referentie en wordt als set/remove-patch vastgelegd. Gevelvlakken zijn bruto (netto uit tabel 3 plus beglazing). Elk oppervlakteveld (ventilatie, tapwater, verlichting) volgt de gebruiksoppervlakte.
  - EP-W002a/b: gevels nemen R_si + R_se 0,17 en het dak 0,14 (tabel 1, p. 5); de vloer op grond R_c + 0,17, zoals de referentie (6,17 bij R_c 6,0). Het bouwjaar werkt op infiltratie, ventilatoren en de isolatieperiode van de leidingen, niet op de ketel (zoals EP-W101c).
  - EP-W002c: de verticale hoeken zijn 4 × 5,4 = 21,6 m (de test drukt 20,8 en 21,6 af). De aansluiting vloer-gevel (ψ −0,18) is de vloerrand van 8.36.
  - EP-W006a–e: R_bw is de R_c van de gevel met zijn U 0,162 (8.47 staat de gevel-U toe). h van (8.47), in 2022 verplicht, is 0 voor een vloer op maaiveld en 0,25 m waar de test dat noemt. De vloer boven een kruipruimte telt ook voor de infiltratie (tabel 11.1, alleen vóór 1992).
  - EP-W006g: de dijkwoning is gerekend als verwarmde ruimte onder maaiveld (8.3.3.2, 8.42/D.12): het noordelijke wanddeel van 8 m ligt 5,4 m diep, de andere delen op maaiveld. De noordgevel is daarmee geen buitenvlak meer.
  - EP-W013b: vier woningen in één gebouw, één berekening met N_woon 4 (internalGains, ventilatie, tapwater); de infiltratierij blijft die van de referentie.
  - EP-W013d: 10 modules van 2 bouwlagen, 54 m hoog, 10 woningen met elk een eigen installatie (2 aangesloten bouwlagen per woning), infiltratie "meerlaags, hele gebouw". EP-W013e: de appartementvariant in een gebouw van 54 m; "bestaande bouw" verandert de berekening niet.
  - EP-W014a: lessenaarsdak van 45° op het zuiden, dak 67,9 m², zijgevels 50,4 m², noordgevel 91,2 m², volume 403,2 m³; Ag blijft 96 m². "Vrijstaand met puntdak" is in tabel 11.14 "hellend dak, vrijstaand".
  - EP-W103c: raam 2 (beneden) en raam 4 (boven) gaan naar de oostgevel; dubbelzijdig volgt uit azimutverschil ≥ 90° (11.77).
- **EP-W203l–n.** Twee rekenzones van 48 m² (beneden met de vloer en ramen 1–2, boven met het dak en ramen 3–4), elk een halve woning (6.2b, `dwellingShare` 0,5) en een eigen ventilatie-invoer voor 48 m². Hybride: lucht/waterwarmtepomp 10 kW (tabel 9.27 COP 2,3 bij 45 °C in 2022) met HR107 CW4 35 kW (tabel 9.25 η 0,95); de prioriteitsrendementen van tabel 9.1 zijn deze tabelwaarden. Bij m heeft de bovenzone een tweede hybride en een eigen combi voor de badkamer, de onderzone een combi voor de keuken (13.19a).
- **EP-W303c/d.** Vrije koeling met opslag (1 of 2 kW) naast een compressiekoelmachine (9 of 8 kW); de warmtepomp gebruikt de opslag als bron (10.84). Aquifer is grondwater onder 15 °C (tabel 9.27). De test geeft geen regeneratiegraad R, dus c_source 1,00 (tabel V.1 laagste rij; bij c ook tabel V.3 recirculatie). Bij c is het tapwater een warmtepomp op dezelfde aquifer.
- **EP-W406i/k.** De boosterwarmtepomp zonder meetgegevens rekent met de forfaitaire waarden van 13.162/13.163 (2022 p. 607–609), nieuw in de kern, op de aanvoer 45 °C van de collectieve HR107 (de waarde bij 44 °C geldt). Bij k zijn A/B/C/B_nom van de test de constanten van 9.85 voor een individueel toestel; de collectieve ketel valt onder 9.91, dus ze zijn niet ingevoerd. De regeling in het hoofdvertrek is "centraal met ruimteregeling" van tabel 10.4.
- **EP-U202, U502c, U701d.** EP-U202: 8 modules, Ag 384 m², 16 ramen. a: AHU in de verwarmde zone met verwarmings- en koelbatterij. b: terugregeling tot 60 % (11.61, x 60 met ontwerpbewijs) en toerenregeling (tabel 11.22). c: AHU buiten de zone, kanalen 20–40 m geïsoleerd: tabel 11.19 situatie 2 (2022 p. 489). EP-U502c: 32 × 24 m, Ag 1536 m², sport met 500 m² sportzaal (13.32a). EP-U701d: op EP-U202a met warmtewiel (tabel 11.18 "rotary") en stoombevochtiger op gas.
- **Utiliteit.**
  - EP-U201/203: recirculatie verlaagt de buitenlucht, niet het toerental: tabel 11.22 "overige" (f_regfan 1). EP-U203b: motor 80 W met gemeten U·I·e = 220 × 0,6 × 1 = 132 W (11.136).
  - EP-U302a: L_max 40 m geldt alleen voor een berekende pomp; de ketelpomp valt onder 9.85. EP-U302b: het bouwjaar 1950 werkt volgens de test ook op de hulpenergie van de ketel.
  - EP-U303g: WKK van 20–200 kW elektrisch (100 kW als klassewaarde), 45/40 als lage temperatuur.
  - EP-U401a: wandkoeling is stralingskoeling, dus 17/21. EP-U403a: gasmotor van 30 kW, gebouwd na 2006.
  - EP-U502b: L_max 40 m heeft geen invoer bij de circulatiepomp (13.39).
  - EP-U503a: tien douches, waarvan 2 met verticale en 4 met horizontale douche-WTW, collectieve opstelling ("gedeelde units", tabel 13.8), toewijzing onbekend (p. 564).

## Stand van de codering

227 van de 266 deeltesten uit bijlage 1 zijn gecodeerd: 172 voor woningen en 55 voor utiliteit (9 oktober 2026; eerst 52, daarna 115 en 199). Elke gecodeerde deeltest rekent in de gate. Bij de open deeltesten heeft de kern in vrijwel alle gevallen de route al. Ze zijn nog niet gecodeerd, omdat geometrie of productgegevens uit de beschrijving moeten worden afgeleid.

EP-W003c (rolluik met extra warmteweerstand) en EP-W016 (deuren) waren te controleren. Beide routes bestaan: `window_u` rekent U_W+shut (8.22/8.23) en de deur (8.20, 8.18/8.19) als rekenhulp, en het project neemt de uitkomst als U-waarde van het raam of de deur. Ze zijn gecodeerd.

Niet te coderen met de huidige kern of de norm:
- **EP-W204d** (pelletkachel die niet aan bijlage R voldoet): tabel 9.30 (2022 p. 337) geeft alleen rendementen voor toestellen die aan bijlage R voldoen. Voor andere toestellen is er geen forfaitaire route; de kern weigert met `biomass_class_unsupported`.
- **EP-U602g/h**: de daglichtfactoren worden direct gegeven, zonder sectorgeometrie (zie de tabel).
- **EP-W006f** (vloer 0,5 m onder maaiveld boven een kruipruimte): de kern kent een vloer boven een kruipruimte (8.3.4.2) en een verwarmde ruimte onder maaiveld (8.3.3.2), niet beide tegelijk (`ground_floor_below_and_heated_basement`).
- **EP-W006h** (oostgevel grenst aan een AOR): met de forfaitaire thermische bruggen van de referentie weigert de kern een onverwarmde aangrenzende ruimte (`forfait_thermal_bridges_unheated_space_unsupported`, 8.4 met U_iu;equi van C.1.3 wordt niet afgeleid).
- **EP-W406j** (booster met COP 3,5 en P_ls 0,03 kW): bijlage W vraagt de COP bij twee brontemperaturen en een gemeten klasse; met één COP is er geen route, en het forfait van 13.163 kent geen opgegeven COP.

De 11 realistische gebouwen (EPWRealB01 tot en met EPWRealD05 en EPURealB01) zijn niet te coderen zolang de bijlagen 3A–3K niet in bezit zijn.

| Test | Onderwerp | Deeltesten | Gecodeerd | Open | Waarom open |
|---|---|---|---|---|---|
| EPW001 | Referentie | 1 | EPW001 | – |  |
| EPW002 | Isolatie | 3 | a, b, c | – |  |
| EPW003 | Raameigenschappen | 3 | a, b, c | – |  |
| EPW004 | Oriëntatie | 7 | a, b, c, d, e, f, g | – |  |
| EPW005 | Thermische massa | 3 | a, b, c | – |  |
| EPW006 | Begrenzing begane grondvloer | 8 | a, b, c, d, e, g | f, h | f: vloer onder maaiveld boven een kruipruimte, geen route; h: AOR met forfaitaire bruggen, geen route (zie boven) |
| EPW007 | Infiltratie | 3 | a, b, c | – |  |
| EPW008 | Overstek | 5 | a, b, c, d, e | – |  |
| EPW009 | Zijbelemmeringen | 6 | a, b, c, d, e, f | – |  |
| EPW010 | Belemmering | 5 | a, b, c, d, e | – |  |
| EPW011 | Zonwering | 2 | – | a, b | route nu aanwezig: `ntaCalculation.windowGlazings` geeft per raam de waarden van 7.41 (a: g_gl,alt 0,045 en g_gl,dif 0,2), een glastype van tabel 7.4 of vaste lamellen; nog te coderen. b vraagt de waarden uit de kwaliteitsverklaring van figuur 7 en U_w volgens 8.15 |
| EPW012 | Zonwering en oriëntatie | 6 | a, b, c, d, e, f | – |  |
| EPW013 | Gebruiksoppervlak | 5 | a, b, c, d, e | – |  |
| EPW014 | Dakvorm | 1 | a | – |  |
| EPW015 | Vertikale leidingen | 2 | a, b | – |  |
| EPW016 | Deuren | 3 | a, b, c | – |  |
| EPW101 | Ventilatiesysteem | 23 | a t/m w (alle) | – |  |
| EPW102 | Voorverwarming nat. toev.vent. | 2 | a, b | – |  |
| EPW103 | Zomernachtventilatie | 3 | a, b, c | – |  |
| EPW104 | Ventilatie overig | 6 | – | a, b, c, d, f, g | open verbrandingstoestellen, badgeiser, ventilatorvermogen, D.4a in een meergezinswoning: routes aanwezig, nog te coderen |
| EPW201 | Afgifte | 5 | a, b, c, d, e | – |  |
| EPW202 | Distributie | 6 | a, b, c, d, f | e | e: route nu aanwezig: Ψ uit de geometrie met 9.34 (`pipeTransmittance` `insulated_embedded`); nog te coderen |
| EPW203 | Opwekking | 16 | a, b, c, d, e, f, g, h, i, l, m, n, p | j, o, q | j: route nu aanwezig: A, B, C en B_nom van 9.85 uit de kwaliteitsverklaring (`declaredAuxiliaryConstants`, §9.1); nog te coderen. o: luchtverwarmer met axiale en radiale recirculatieventilatoren; q: kwaliteitsverklaring uit bijlage 5 (niet in bezit) |
| EPW204 | Opwekking2 | 9 | a, b, c, e, f, g, i | d, h | d: niet te coderen, zie boven; h: route nu aanwezig: ε_chp;th en ε_chp;el uit de kwaliteitsverklaring in plaats van tabel 9.31 (`declaredEfficiencies`, §9.1); nog te coderen |
| EPW205 | Opwekking gemeenschappelijk | 2 | a, c | – |  |
| EPW206 | Opwekking woongebouw | 3 | – | a, b, c | opwekking in een woongebouw: nog te coderen |
| EPW301 | Afgifte | 3 | a, b, d | – |  |
| EPW302 | Distributie | 6 | a, b, c, d, f | e | e: route nu aanwezig: Ψ uit de geometrie met 10.25 (`pipe` `calculated`); nog te coderen |
| EPW303 | Opwekking | 5 | a, b, c, d, e | – |  |
| EPW401 | Afgifte | 2 | a, b | – |  |
| EPW402 | Distributie | 7 | a, b, c, d, e, g | f | f: route nu aanwezig: Ψ uit de geometrie met 13.28 (`circulation.calculatedPsi`); nog te coderen |
| EPW403 | Douche WTW | 4 | a, b, c, d | – |  |
| EPW404 | Voorraadvat | 4 | a, b, c, d | – |  |
| EPW405 | Zonneboiler | 6 | a, b, c, d, e, f | – |  |
| EPW406 | Opwekking | 21 | a, b, c, d, e, f, g, i, k, m, n, q, u, v | j, l, o, p, t, w, x | j: booster met één COP, geen route (zie boven); l: biomassa die niet aan bijlage R voldoet (geen route, zie EP-W204d); o, p: warmtepompboiler met kwaliteitsverklaring; t: warmtepomp met ketel en een opgegeven aandeel; w, x: verklaringen uit bijlagen 6 en 7 (niet in bezit). Routes deels aanwezig, nog te coderen |
| EPW407 | Opwekking gemeensch./woong. | 9 | – | a, b, c, d, e, f, g, h, i | collectieve tapwateropwekking: nog te coderen |
| EPW501 | PV-panelen | 4 | a, b, c, d | – |  |
| EPU001 | Referentie | 1 | EPU001 | – |  |
| EPU002 | Gebruiksfunctie | 10 | a, b, c, d, e, f, g, h, i, j | – |  |
| EPU102 | Zonwering | 3 | a, b, c | – |  |
| EPU201 | Ventilatiesysteem | 2 | a, c | – |  |
| EPU202 | AHU | 3 | a, b, c | – |  |
| EPU203 | Ventilatoren | 3 | a, b, c | – |  |
| EPU301 | Afgifte | 1 | a | – |  |
| EPU302 | Distributie | 2 | a, b | – |  |
| EPU303 | Opwekking | 7 | a, b, c, d, e, f, g | – |  |
| EPU401 | Afgifte | 1 | a | – |  |
| EPU402 | Distributie | 1 | a | – |  |
| EPU403 | Opwekking | 1 | a | – |  |
| EPU501 | Afgifte | 1 | a | – |  |
| EPU502 | Distributie | 3 | a, b, c | – |  |
| EPU503 | Douche wtw | 1 | EPU503 | – |  |
| EPU504 | Opwekking | 1 | EPU504 | – |  |
| EPU601 | Vermogen | 3 | a, b, c | – |  |
| EPU602 | Regeling | 9 | a, b, c, d, e, f, i | g, h | g, h: de test geeft de daglichtfactoren F_D;S, F_D;dayl en F_D direct; de kern leidt ze af uit de sectorgeometrie (14.25–14.42) en kent geen invoer van opgegeven daglichtfactoren. Niet te coderen zonder geometrie |
| EPU701 | Bevochtiging | 4 | a, b, c, d | – |  |
| EPWReal/EPUReal | Realistische gebouwen | 11 | – | alle | invoer in bijlagen 3A–3K, niet in bezit |

## Versie 5.0:2026 inpluggen

1. **Verkrijgen.** ISSO 54 versie 5.0:2026 en het resultatendocument rechtmatig verkrijgen, met de gebruiksrechten (extern, taak E1 in de [gereedheid](nta8800-brl9501-gereedheid.md)).
2. **Uitgave.** Versie 5.0 hoort bij NTA 8800:2025+C1. Daarom:
   - in `EPW001.json` en `EPU001.json` het veld `ntaCalculation.normVersion` weghalen (dan geldt de standaarduitgave 2025+C1);
   - in `scripts/isso54-suite.py` de `normVersion` van de gevallen op `2025+C1` zetten.
   
   De invoer die alleen in 2022 bestaat, vraagt de kern dan als `route_not_in_edition` aan; die invoer omzetten. Zie [normversies](nta8800-normversies.md) voor de verschillen tussen de uitgaven.
3. **Referenties en deeltesten nalopen.** De referentiegebouwen en deeltesten van 5.0 vergelijken met 2.0. Gewijzigde invoer in de bouwer of de patches aanpassen, nieuwe deeltesten toevoegen en vervallen deeltesten schrappen.
4. **Verwachte waarden invullen.** Per deeltest de metrieken van `pending.metrics` verplaatsen naar `expected`, met:
   - de waarde uit het resultatendocument;
   - `relativeTolerance` 0,01, of de band die 5.0 per deeltest noemt;
   - `absoluteTolerance` 0.
   
   Daarna het blok `pending` weghalen. Een geval met beide wordt geweigerd. Dit kan in `scripts/isso54-suite.py` met een tabel per test-id.
5. **Meer grootheden vergelijken.** Grootheden die 5.0 per deeltest vraagt en die de kern al levert, zoals de energie per dienst en drager, toevoegen. Het referentieharnas kent daarvoor de paden `serviceAnnual/<dienst>/<drager>/usedKwh` en verwante paden (`crates/nta8800-core/src/reference.rs`).
6. **Draaien.** De gate draait de suite. `reference-report.md` in `nta-evidence/gate/reference/` geeft per deeltest de verwachting, de uitkomst, het verschil en het oordeel, met de commit, `KERNEL_VERSION` en de SHA-256 van invoer en uitvoer. Dat rapport is het bewijs voor BRL 9501 §3.3, §4.2, §6.2 en §6.3.
7. **Afwijkingen verklaren.** Elke afwijking boven de band verklaren en vastleggen. Ligt de oorzaak in de testset, dan dat melden aan InstallQ en ISSO.

## Verschillen tussen versie 2.0 en 5.0

Over de inhoud van versie 5.0:2026 is niets openbaar gevonden (gezocht op 9 oktober 2026). Bekend is alleen wat BRL 9501 (29-05-2026) er zelf over zegt: de indeling in deelgebieden, hoofdgroepen, tests en deeltesten, en een bandbreedte per (deel)test (§3.3 opm., p. 6).

Te verwachten, maar niet bevestigd:
- de overgang van NTA 8800:2022 naar 2025+C1, met de schakelpunten in [normversies](nta8800-normversies.md);
- deeltesten voor routes die sinds 2022 zijn bijgekomen, zoals de bronregels van warmtepompen, flexmodus en de LED-kolom.

