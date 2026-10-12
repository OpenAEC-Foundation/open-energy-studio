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
| Oordeel zonder verwachtingen | Een run met alleen deze suite vergelijkt niets: het rapport zegt "geen vergelijking (0 vergeleken, 260 zonder verwachting)" en de afsluitcode is 3, niet 0. In de gate lopen de openbare gevallen en de RVO-woningen mee; die vergelijken wel. |

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
- **EP-U001 verticale leidingen.** Hetzelfde gebouw als EP-W001 (leidingdoorvoeren forfaitair) met 1 toiletgroep: één ongeïsoleerde fictieve verticale leiding per toiletgroep, N = H/3 (NTA 8800:2022 7.3.3, formule 7.17a, p. 155–156), met H 5,4 m en bij EP-U202 en EP-U701d 21,6 m.
- **EP-U001 verlichting.** "LED, vermogen forfaitair": NTA 8800:2022 heeft geen aparte LED-kolom (die is van 2024), dus de forfaitaire waarde van de functie.
- **EP-W003a/c en EP-W016.** De U-waarde is met de formules van `window_u` berekend en afgerond volgens 8.2.2.1 (één decimaal boven 1,0):
  - EP-W003a: 8.15, max(0,7·2,0 + 0,3·3,4; 0,8·2,0 + 0,2·3,4) + 2,5·0,11 = 2,695 → 2,7;
  - EP-W003c: U_W+shut = 1/(1/1,8 + 0,2) = 1,324 (8.23), effectief 0,5·1,8 + 0,5·1,324 = 1,562 → 1,6 (8.22, f_shut;with 0,5). Alleen de extra weerstand varieert; het rolluik is niet als zonwering ingevoerd. Dit is een open interpretatie, te toetsen aan versie 5.0: als zonwering (buitenrolluik, gekleurd, handbediend) stijgt BENG 2 met ongeveer 12 % en daalt TOjuli van 8,91 naar 2,28. De alternatieve codering staat als commentaar in `scripts/isso54-suite.py`;
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
- **EP-U602g/h (opgegeven daglichtfactoren).** De test geeft F_D;S, F_D;dayl en F_D direct, zonder sectorgeometrie (p. 50–51). De norm kent geen invoer van een opgegeven F_D;S: die volgt uit 14.37–14.42. Wel mag F_D;dayl;i van een daglichtsector worden bepaald met bijlage F van NEN-EN 15193-1:2017, naar beneden afgerond op twee significante cijfers (de alinea onder tabel 14.7; 2020+A1 p. 643, 2022 p. 648, 2023 p. 657, 2025+C1 p. 672; ook in 2024). Die route heeft de kern nu: een sector van soort `declared` met de oppervlakte A_D;i, F_D;dayl;i en een bronvermelding. De deeltesten zijn zo gecodeerd:
  - **g:** één verlichtingszone van 96 m², 768 W, voorschakeling onbekend, geen nieuwwaardecompensatie, automatisch aan/automatisch uit, en een sector van 48 m² met F_D;dayl 0,781 (= 1 − 0,73 × 0,3; de test geeft F_D 0,8905 = (48 × 0,781 + 48)/96).
  - **h:** twee verlichtingszones van 48 m² met elk 384 W. De onderste heeft een sector van 24 m² met F_D;dayl 0,781 (F_D 0,8905 zoals g). De bovenste heeft daklichten met F_D;S 0,68. Dat is de waarde van tabel 14.8 (2025+C1 p. 674; tabel 14.9 in 2022, p. 650) voor een daglichtfactor van 2 tot 4 %, dus die sector gaat door de gewone daklichtroute (14.31–14.36, 14.41/14.42), zonder afronding: een daklicht van 2 × 4 m in de hoek van een ruimte van 8 × 6 m, vrije hoogte 2,75 m, opening 3 m², armaturen 2 m boven het werkvlak. Dat geeft A_D 24 m², D ongeveer 3,2 %, F_D;dayl 1 − 0,73 × 0,68 = 0,5036 en F_D 0,7518, gelijk aan de test (review 22; test `isso54_u602h_rooflight_sector_gives_the_unrounded_14_36_factor`).
  - **Te toetsen.** F_D;S 0,3 van g en van de onderste zone van h ligt buiten bereik van 14.37 voor verticale ramen (minstens 0,65 × 0,12 + 0,25 = 0,328), dus alleen daar is de route van NEN-EN 15193-1 nodig. De test rekent 0,781 met 14.36, zonder afronding; via de 15193-1-route rondt de kern af naar 0,78, zoals de norm voor die route voorschrijft, en wordt F_D 0,89 in plaats van 0,8905 (0,06 %). Het afronden gaat naar beneden op twee cijfers, niet naar een klasse van tabel X.1: waar de norm die klassen bedoelt, noemt ze bijlage X of tabel X.1 (zoals bij P_n, 2025+C1 p. 660), en deze alinea doet dat niet.
- **EP-U002j.** 30 % kantoor en 70 % overige bijeenkomst in één rekenzone. Ventilatie, tapwater en verlichting zijn per oppervlak verdeeld. De gebruiksfunctie van de zone, voor setpoints en interne warmte, is de grootste: overige bijeenkomst.
- **Referenties: geen leidingen in onverwarmde ruimten.** EP-W001 en EP-U001 noemen geen verwarmingsleidingen door onverwarmde ruimten, maar de codering liet `unheatedPipeLengthM` leeg. Dan neemt 9.36 15 % van de lengte als onverwarmd. Gecorrigeerd naar 0 (9 oktober 2026): BENG 2 van EP-W001 daalt van 105,18 naar 105,07. EP-W202a (lengte onbekend) is nu de variant met 15 %.
- **EP-W201b–d.** Rij van de overtemperatuur in tabel 9.3 (2022 p. 273) naar de gemiddelde watertemperatuur min 20 °C: 55/47 is 31 K (rij 30 K), 50/42 is 26 K (rij 30 K), 90/70 is 60 K. Bij convectoren met boosterventilatoren de rij "radiatoren met ventilator". De 40 W per ventilator vraagt de route van NEN-EN 16430, die NTA 8800:2022 niet kent (schakelpunt 56): gerekend is met 10 W per ventilator uit tabel 9.11. De gemiddelde ontwerptemperatuur van de ketel volgt de ontwerpklasse.
- **Ventilatie EP-W101c, r, s, w.**
  - EP-W101c: het bouwjaar 1985 werkt volgens de test op infiltratie, de ventilatoren en de isolatieperiode van de verwarmingsleidingen (1980–1995), niet op de ketel.
  - EP-W101r: verklaring van 85 % met dissipatie verdisconteerd, als NEN-EN 13141-7. EP-W101s: 85 % volgens NEN-EN 13142, zonder dissipatie.
  - EP-W101w: E.1 met D.5b op 40 m² en C.1 op 30 m² verblijfsgebied, zonder bypass, zonder passieve koeling via het systeem.
- **EP-W102b.** q_v;inst 60 dm³/s natuurlijke toevoer, waarvan 30 dm³/s (108 m³/h) voorverwarmd. De kern gebruikt q_v;inst (11.62) ook voor het ontwerpdebiet (11.55/11.58), niet alleen voor het voorverwarmde aandeel van 11.128; het voorverwarmde debiet vraagt bovendien een geïnstalleerde natuurlijke capaciteit. Tegen C1 komt daardoor ongeveer 6,4 kWh/m² BENG 2 van de capaciteit en 1,3 van de voorverwarming. De kern heeft geen route om de 60 dm³/s alleen aan de voorverwarming te geven; de codering volgt de testtekst letterlijk. Open interpretatie, te toetsen aan versie 5.0.
- **EP-W103a/b.** De vier ramen van de zuidgevel, elk 2,5 m² netto (8,33 m² × 0,30 gaas), openingshoek 90°, middens op 1,2 m en 3,9 m, openingshoogte 2 m. De voorwaarden van 11.2.3.3 (inbraak, insecten, regen) zijn als vervuld aangenomen; de test noemt ze niet.
- **Raamposities EP-W008–W010.** Uit de figuren 2–6: ramen van 3,0 × 2,0 m, borstwering 0,5 m, verdiepingshoogte 2,7 m, horizontaal 0,5–3,5 m en 4,5–7,5 m. Raammiddens op 1,5 m en 4,2 m. `raam-zuid-1`/`-3` zijn als de linker ramen (beneden, boven) genomen, `-2`/`-4` als de rechter; links en rechts van de ramen zijn zoals in de figuren, van buiten op de zuidgevel gezien (oost is rechts). De zijde van een zijbelemmering (`side` in de kern) is gezien vanuit het raam naar buiten, het zichtveld van 17.3.2: west is rechts, oost is links.
  - EP-W008a–c: overstek direct boven elk raam, h_o = 1,0 / diepte (2022 p. 672).
  - EP-W008d/e: het doorlopende dak (5,4 m) over de linker 4 m dekt alleen de linker ramen.
  - EP-W009a/b: b_b = 1,5 / diepte aan de westzijde (rechts). EP-W009c–f: één belemmering aan de oostzijde (in het zichtveld links), 0,5 m naast de rechter ramen: b_b = 2,0 / diepte voor de rechter en 6,0 / diepte voor de linker ramen (p. 673). Een b_b van 3,73 of meer is minimale belemmering (17.3.2 situatie a, p. 674): bij EP-W009c (b_b 10,0) en EP-W009d (b_b 5,0) zijn de linker ramen daarom `minimal`.
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
  - EP-W013b: vier woningen in één gebouw, één berekening met N_woon 4 (internalGains, ventilatie, tapwater); de infiltratierij blijft die van de referentie. Elke woning heeft een eigen installatie: `identicalSystems` 4 (§9.1, zoals EP-W206c).
  - EP-W013d: 10 modules van 2 bouwlagen, 54 m hoog, 10 woningen met elk een eigen installatie (2 aangesloten bouwlagen per woning, `identicalSystems` 10), infiltratie "meerlaags, hele gebouw". EP-W013e: de appartementvariant in een gebouw van 54 m; "bestaande bouw" verandert de berekening niet.
  - EP-W014a: lessenaarsdak van 45° op het zuiden, dak 67,9 m², zijgevels 50,4 m², noordgevel 91,2 m², volume 403,2 m³; Ag blijft 96 m². "Vrijstaand met puntdak" is in tabel 11.14 "hellend dak, vrijstaand".
  - EP-W103c: raam 2 (beneden) en raam 4 (boven) gaan naar de oostgevel; dubbelzijdig volgt uit azimutverschil ≥ 90° (11.77).
- **EP-W203l–n.** Twee rekenzones van 48 m² (beneden met de vloer en ramen 1–2, boven met het dak en ramen 3–4), elk een halve woning (6.2b, `dwellingShare` 0,5) en een eigen ventilatie-invoer voor 48 m², gemaakt voor de hele woning van 96 m² (`wholeDwellingAreaM2`, voor f_τ van tabel 11.8 en het minimum van 11.65). Elke zone is één bouwlaag, dus de forfaitaire verticale leiding telt per zone 1 bouwlaag (7.3.3). Zonder hybride opwekker rekent de splitsing als EP-W001; de test `crates/nta8800-core/tests/isso54_controls.rs` bewaakt dat met `training-data/isso54/EPW001-two-zones-control.json`. Hybride: lucht/waterwarmtepomp 10 kW (tabel 9.27 COP 2,3 bij 45 °C in 2022) met HR107 CW4 35 kW (tabel 9.25 η 0,95); de prioriteitsrendementen van tabel 9.1 zijn deze tabelwaarden. Bij m heeft de bovenzone een tweede hybride en een eigen combi voor de badkamer, de onderzone een combi voor de keuken (13.19a).
- **EP-W303c/d.** Vrije koeling met opslag (1 of 2 kW) naast een compressiekoelmachine (9 of 8 kW); de warmtepomp gebruikt de opslag als bron (10.84). Aquifer is grondwater onder 15 °C (tabel 9.27). De test geeft geen regeneratiegraad R, dus c_source 1,00 (tabel V.1 laagste rij; bij c ook tabel V.3 recirculatie). Bij c is het tapwater een warmtepomp op dezelfde aquifer.
- **EP-W406i/k.** De boosterwarmtepomp zonder meetgegevens rekent met de forfaitaire waarden van 13.162/13.163 (2022 p. 607–609), nieuw in de kern, op de aanvoer 45 °C van de collectieve HR107 (de waarde bij 44 °C geldt). Bij k zijn A/B/C/B_nom van de test de constanten van 9.85 voor een individueel toestel; de collectieve ketel valt onder 9.91, dus ze zijn niet ingevoerd. De regeling in het hoofdvertrek is "centraal met ruimteregeling" van tabel 10.4.
- **EP-W011a.** Alle vier ramen krijgen 7.41 met g_gl;alt 0,045 en g_gl;dif 0,2 (`windowGlazings`, `diffusing`).
- **EP-W011b.** De kwaliteitsverklaring van figuur 7 (p. 15) geeft g 0,63 en U_g 1,1. g_gl;n 0,63 is de g-waarde van het raam. U_w met de vereenvoudigde 8.15 zoals EP-W003a: max(0,7·1,1 + 0,3·2,4; 0,8·1,1 + 0,2·2,4) + 2,5·0,08 = 1,69 → 1,7.
- **EP-W203j.** Het opgegeven rendement 0,975 en de fractie 1 zijn gelijk aan tabel 9.25 (2022) voor een HR107 binnen de schil bij 45/40; ze zijn niet ingevoerd. A, B, C en B_nom staan in `declaredAuxiliaryConstants`. De W;H;aux van 106,48/111,55 kWh in de test is een uitkomst, geen invoer.
- **EP-W204h.** Als EP-W204g (micro-WKK < 2 kW, LT, 45/40), nu met HRe-label en ε_th 0,8 en ε_el 0,12 uit de kwaliteitsverklaring (methode 2, §9.1).
- **EP-W202e, W302e, W402f.** De "binnendiameter 0,02 m" van de test is d_i van 9.34/10.25/13.28; die formules definiëren d_i als de leidingdiameter zonder isolatie (2022 p. 293). d_a 0,04 m, diepte 0,03 m, λ isolatie 0,04, λ constructie 2. EP-W202e geeft dezelfde uitkomst als EP-W001: er lopen geen leidingen door onverwarmde ruimten.
- **EP-W203o.** HR104-luchtverwarmer van tabel 9.25, luchtverwarming zonder distributiesysteem (opgegeven verlies 0). 9.23 telt de ventilatoren per type: één axiale (0,014·Q_h;b) en drie radiale (0,022·Q_h;b) over n_H;aux = 4 geeft (0,014 + 3·0,022)/4·Q_h;b = 0,020·Q_h;b; de kern kent daarvoor `airHeaters.count` en `further` (nieuw). Die hulpenergie hoort bij de afgifte (2022 p. 282), dus de opwekker zelf heeft geen elektrisch aangesloten toestel. Q_h;b is geschat volgens tabel 9.12. Regeling per ruimte als de referentie.
- **EP-W206a–c.** 48 appartementen: 8 modules naast elkaar en 6 boven elkaar, 12 bouwlagen, 32,4 m. De zuidgevel van de tabel (921,6 m²) is netto; bruto 2073,6 m² met 192 ramen van 6 m². Nieuwbouwberekening voor het hele gebouw: individuele ventilatie per appartement (de referentie, geschaald), een elektrische doorstromer per appartement. De ketel van 10 kW staat volgens de test op het gebouw; de opstelplaats is niet gegeven, dus die van de referentie (binnen de schil) blijft staan; voor een collectieve ketel hangt de rij van tabel 9.25 daar niet van af. Leidingen door onverwarmde ruimte met onbekende lengte (9.36).
  - b: twee rekenzones, de onderste vier appartementlagen (8 bouwlagen, 32 appartementen, 2/3 van de gevels en de vloer) op de collectieve ketel, de bovenste 16 appartementen met elk een HR107 als extra verwarmingssysteem (§9.2) met 16 gelijke toestellen. BENG 1 ligt 1 % hoger dan bij a door de verdeling over twee zones.
  - c: 12 gelijke installaties (§9.1 `identicalSystems`) die samen het hele gebouw bedienen, elk 2 bouwlagen.
- **EP-W006h.** Met de forfaitaire thermische bruggen van de referentie geldt 8.4.1 opmerking 5 met C.10 (2022 p. 240, p. 748): de scheiding met de AOR telt in H_D;for als U_C·b_U, H_U;for = 0. Die route is nieuw in de kern. De AOR is "zonder verdere specificatie"; b_U volgt uit 8.53 met de normatieve H_ue = 5 × A_T;iu van formule I.8 (2025+C1 p. 841), ingevoerd als één transmissie-element met U 5,0 over A_T;iu (de oostgevel van 32,4 m²) zonder ventilatie; opmerking 2 bij I.8 geeft daarmee b = 1/(1 + U/5). Dat is een aanname: formeel hoort bijlage I.2.4 bij de forfaitaire Rc- en U-waarden van I.2.1–I.2.3 (2022 p. 240) en in 2025+C1 bij de basisopname (p. 266). Een eerdere codering bouwde H_ue op uit de onderdelen van opmerking 1 (A_T;ue 1,44 × A_T;iu met U 3,2 en 1,5 × A_T;iu m³/h), wat ≈ 5,11 × A_T;iu gaf in plaats van 5,00. De gevels hebben R_c 0,35 (U 1,923).
- **EP-U202, U502c, U701d.** EP-U202: 8 modules, Ag 384 m², 16 ramen. a: AHU in de verwarmde zone met verwarmings- en koelbatterij. b: terugregeling tot 60 % (11.61, x 60 met ontwerpbewijs) en toerenregeling (tabel 11.22). c: AHU buiten de zone, kanalen 20–40 m geïsoleerd: tabel 11.19 situatie 2 (2022 p. 489). EP-U502c: 32 × 24 m, Ag 1536 m², sport met 500 m² sportzaal (13.32a). EP-U701d: op EP-U202a met warmtewiel (tabel 11.18 "rotary") en stoombevochtiger op gas.
- **Utiliteit.**
  - EP-U201/203: recirculatie verlaagt de buitenlucht, niet het toerental: tabel 11.22 "overige" (f_regfan 1). EP-U203b: motor 80 W met gemeten U·I·e = 220 × 0,6 × 1 = 132 W (11.136).
  - EP-U302a: L_max 40 m geldt alleen voor een berekende pomp; de ketelpomp valt onder 9.85. EP-U302b: het bouwjaar 1950 werkt volgens de test ook op de hulpenergie van de ketel.
  - EP-U303g: WKK van 20–200 kW elektrisch (100 kW als klassewaarde), 45/40 als lage temperatuur.
  - EP-U401a: wandkoeling is stralingskoeling, dus 17/21. EP-U403a: gasmotor van 30 kW, gebouwd na 2006.
  - EP-U502b: L_max 40 m heeft geen invoer bij de circulatiepomp (13.39).
  - EP-U503a: tien douches, waarvan 2 met verticale en 4 met horizontale douche-WTW, collectieve opstelling ("gedeelde units", tabel 13.8), toewijzing onbekend (p. 564).

## Codering EP-W104, EP-W406o/p/t en EP-W407 (9 oktober 2026)

- **EP-W104a** (p. 22–23). Systeem A1 zonder kanalen. Een open CR-ketel (conventioneel) met waakvlam binnen de schil, 90/70, radiatoren tegen de buitenwand, regeling in het hoofdvertrek, geen waterzijdige inregeling. Tapwater: een badgeiser met Gaskeur CW3 (tabel 13.25, doorstroomtoestel met Gaskeur CW, klasse 3). Beide toestellen zijn open en halen verbrandingslucht uit de woning: tabel 11.11 gastoestel type B en tabel 11.12 cv-ketel en badgeiser, met de nominale belasting van 35 en 15 kW. De 35 kW werkt alleen daar: een individuele ketel rekent zijn hulpenergie met de constanten van 9.85.
- **EP-W104b, c, d en f** (p. 23) zijn wijzigingen op EP-W001. b: geïnstalleerde capaciteit 150 dm³/s (11.62). c: nominaal ventilatorvermogen 100 W. d: motor 80 W met U·I·cos φ = 220 × 0,6 × 0,8 = 105,6 W (11.136, eenfasewisselstroommotor). f: C4b met 100 W.
- **EP-W104g** (p. 23). Een hoekwoning op de onderste laag van een meergezinswoning: de referentie zonder dak en oostgevel (verwarmde buren), perimeter 22 m, gebouwhoogte 10,8 m, infiltratie "laag, eind, onder of tussen" (tabel 11.14), bouwjaar 1945. "Geïsoleerd vóór 1995" valt in de rij "vóór 1980 of onbekend": de test zegt niet of het vóór 1980 was. D4a met een verklaard rendement van 85 % (NEN-EN 13142, dissipatie verdisconteerd), Luka C, geïsoleerd kanaal van 1 m, centraal, bypass 100 % met koudeterugwinning op de verklaring (11.106a, in 2022 aanwezig). Forfaitaire gelijkstroomventilatoren van 2020, geen volumeregeling, geen passieve koeling.
- **EP-W406o en p** (p. 39). Een warmtepompboiler op ventilatieretourlucht, alleen tapwater, met een kwaliteitsverklaring: COP 3 als verklaard η_W;gen. De kern past daarop f_prac 0,95 toe (13.152 voor een niet-forfaitaire waarde); dat is de f_prac die de test geeft. P_nom 5 en 0,3 kW (13.141) en q_ve;hp;W 200 m³/h uit de verklaring (13.148a). Het "aandeel hernieuwbare energie 1000 kWh" is geen invoer: retourlucht is zonder opgave van het buitenluchtaandeel niet hernieuwbaar (5.32). EP-W406p heeft bijverwarming nodig. Gecodeerd als elektrische doorstroomverwarmer (tabel 13.25, 0,95), omdat de test geen vat noemt. Het vermogen ontbreekt ook; 13.141 vraagt er een, dus 2 kW is aangenomen (fictief, genoeg voor de rest).
- **EP-W406t** (p. 39). Een buitenluchtwarmtepomp (voldoet aan tabel 9.28) voor verwarming en tapwater, met een los vat van 150 l, label D, T-stukken geïsoleerd, binnen de schil, 0,3 kW. Ze levert 64,3 % van het tapwater (13.146, één punt). Een HR107 met CW4 van 10 kW levert alleen tapwater, de rest.
- **EP-W407** (p. 41–42). Het gebouw van EP-W206: 48 appartementen, 12 bouwlagen, A_g 4608 m².
  - a: elk appartement heeft de installaties van EP-W001 (48 gelijke systemen).
  - b: een gemeenschappelijke HR107 van 100 kW buiten de verwarmde zone, voor verwarming en tapwater, met een indirect gestookt vat van 2000 l (label niet gegeven, vanaf 2018 zoals EP-W402) en geïsoleerde T-stukken (f_sto;dis;ls 2). Circulatie 28/25 met 20 mm isolatie, geen leidingen door onverwarmde ruimten, kleppen en beugels geïsoleerd, pomp onbekend zonder regeling, geen afleversets.
  - c: tapwater uit een direct gestookt vat boven 150 kW (13.8.4.5.4, 0,50); verwarming als b.
  - d: als b met een VR-ketel binnen de verwarmde zone, zonder waakvlam.
  - e–h: dezelfde installaties voor één tussen-midden-appartement. Alleen de voor- en achtergevel grenzen aan buiten; infiltratie "laag, midden, onder of tussen". De collectieve oppervlakken blijven die van het hele gebouw.
  - i: een WKK van 15 kW elektrisch en 30 kW thermisch, LT, met gemeten ε_th 0,6 en ε_el 0,3 in plaats van de rij van tabel 9.31. De WKK verwarmt ook een vat van 2000 l met geïsoleerde T-stukken. Gebouwd na 2006 is aangenomen; de gemeten waarden vervangen de rij toch.
- **Kernwijziging voor EP-W407i.** De kern weigerde een voorraadvat naast een WKK voor tapwater (`hot_water_storage_in_generator_efficiency`). 13.8.4.7.4 (2022 p. 621) bepaalt de verliezen van indirect verwarmde voorraadvaten in 13.6, en opmerking 1 van 13.6.2 (2022 p. 547) zondert een WKK niet uit. Het vat wordt nu toegestaan en gerekend, zoals bij een ketel.

## Correcties na review 19 (10 oktober 2026)

Een steekproef van 41 deeltesten tegen de testtekst vond vier codeerfouten. Ze zijn gecorrigeerd; de kern is niet veranderd.

| Correctie | Deeltesten | Effect (BENG 1 / BENG 2) |
|---|---|---|
| Splitsing in twee rekenzones: 1 bouwlaag per zone voor de verticale leiding, ventilatie voor de hele woning van 96 m² | EP-W203l–n | W203l 75,96 → 74,47 / 105,81 → 98,48; W203m BENG 2 114,90 → 107,58; W203n 98,39 → 91,76 |
| Fictieve verticale leiding per toiletgroep (7.17a) in de utiliteitsreferentie | alle 55 EP-U | EP-U001 94,55 → 95,43 / 135,32 → 136,28; EP-U202a 88,40 → 89,74 / 122,04 → 123,38 |
| Linker ramen met b_b ≥ 3,73 als minimale belemmering | EP-W009c/d | 75,73 → 74,92 / 106,50 → 105,76 |
| Eigen installatie per woning: `identicalSystems` 4 en 10 | EP-W013b/d | BENG 2 103,71 → 104,70 en 86,35 → 86,94 |

Verder, zonder effect op de uitkomst:
- **EP-W402a–g.** De test geeft geen label van het vat (p. 33): nu `unknown_label` met productie vanaf 2018, wat 13.59 als label C rekent. Bij EP-U502a staat label C wel in de test (p. 49).
- **EP-W002.** De vloer op grond had de constructie van het dak; nu een eigen vloerconstructie. De U van de vloer komt uit `groundFloors`.
- **Paginaverwijzingen.** Ongeveer 60 deeltesten noemden de pagina vóór (of na) de rij van de deeltest; ze wijzen nu naar de pagina van de deeltest zelf. EP-W406q (p. 39) en EP-U601c (p. 50, in de testtekst als "EP-W601c") waren goed.

Open interpretaties, te toetsen zodra versie 5.0 met uitkomsten er is: EP-W102b (q_v;inst ook in het ontwerpdebiet) en EP-W003c (rolluik alleen als extra weerstand), zie hierboven.

## Stand van de codering

260 van de 266 deeltesten uit bijlage 1 zijn gecodeerd: 203 voor woningen en 57 voor utiliteit (9 oktober 2026; eerst 52, daarna 115, 199, 227, 239, 240 en 258). Elke gecodeerde deeltest rekent in de gate. De zes open deeltesten zijn niet te coderen: drie vragen een kwaliteitsverklaring uit een apart bijlagedocument dat niet in bezit is, drie hebben geen route in de norm (zie hieronder en de tabel).

EP-W003c (rolluik met extra warmteweerstand) en EP-W016 (deuren) waren te controleren. Beide routes bestaan: `window_u` rekent U_W+shut (8.22/8.23) en de deur (8.20, 8.18/8.19) als rekenhulp, en het project neemt de uitkomst als U-waarde van het raam of de deur. Ze zijn gecodeerd.

Niet te coderen met de huidige kern of de norm:
- **EP-W203q, EP-W406w, EP-W406x**: de kwaliteitsverklaringen (Inventum Modul AIR Solo, Modul Air combi 180 en Modul Air red) staan in aparte bijlagedocumenten bij ISSO 54 (p. 66–67). Die zijn niet in bezit; zonder de verklaarde waarden is er niets in te voeren.
- **EP-W406l** (biomassa die niet aan bijlage R voldoet): geen forfaitaire route, zoals EP-W204d hieronder.
- **EP-W204d** (pelletkachel die niet aan bijlage R voldoet): tabel 9.30 (2022 p. 337) geeft alleen rendementen voor toestellen die aan bijlage R voldoen. Voor andere toestellen is er geen forfaitaire route; de kern weigert met `biomass_class_unsupported`.
- **EP-W406j** (booster met COP 3,5 en P_ls 0,03 kW): 13.8.4.4 (2022 p. 607) kent meetgegevens volgens bijlage W of de forfaitaire waarden van 13.162/13.163. Bijlage W leidt de COP-lijn af uit twee meetpunten bij 24 (of 16) °C en 40 °C (W.11–W.13, 2022 p. 1089–1090). Met één COP is er geen route; het forfait kent geen opgegeven COP. De kern weigert terecht.

**Hercontrole 12 oktober 2026 (review 62).** Na de uitbreidingen van reviews 40–61 opnieuw nagegaan tegen 2025+C1, de uitgave waarin nieuwe routes zouden staan:
- **EP-W204d en EP-W406l.** 9.6.5 (2025+C1 p. 340–341) kent voor kachels en ketels met vaste biobrandstof methode 1 (meetgegevens volgens NEN-EN 15316-4-1/-4-8) en methode 2 (tabel 9.30, p. 341). Tabel 9.30 geldt nog steeds alleen voor toestellen die aan bijlage R voldoen; de deeltest geeft geen meetgegevens voor methode 1. Voor tapwater (13.8.4.6, tabel 13.22, p. 637) geldt hetzelfde. Het interpretatiedocument (2024) en het Praktijkhandboek v2 geven geen andere route. De kern weigert terecht met `biomass_class_unsupported`.
- **EP-W406j.** 13.8.4.4 (2025+C1 p. 631–632) en bijlage W (p. 1118–1120) zijn inhoudelijk gelijk aan 2022: meetgegevens bij twee brontemperaturen (W.1) of het forfait van 13.162 zonder opgegeven COP. Eén COP blijft zonder route.
- **EP-W203q, EP-W406w en EP-W406x.** De bijlagedocumenten met de kwaliteitsverklaringen van de Inventum Modul-toestellen zijn nog steeds niet in bezit (ook niet lokaal te vinden). Wat ontbreekt: per toestel de verklaarde COP- of rendementswaarden en de bijbehorende tappatronen.

De telling blijft 260 van de 266.

**EP-W006f (vloer onder maaiveld boven een kruipruimte).** De kern weigerde deze combinatie (`ground_floor_below_and_heated_basement`); de norm staat haar toe en de kern rekent haar nu:
- **Stationair.** 8.39 (2022 p. 229; 2025+C1 p. 255) neemt U_fl "volgens 8.3.4", voor een vloer boven een kruipruimte dus 8.43 (2022 p. 230; 2025+C1 p. 256). Voor een vloer boven een kruipruimte of onverwarmde kelder die zelf onder maaiveld ligt, geldt U_x;T = 0 (2022 p. 236; 2025+C1 p. 262): U_x is alleen U_x;V, en h van (8.47) wordt niet gebruikt. De wanden van de verwarmde ruimte onder maaiveld geven z·P·U_bw met de werkelijke diepte z (8.3.3.2); d_1 van 8.45 vergelijkt met d_bf;equi van de kruipruimtevloer (8.33), zoals 8.45 die noemt. Dezelfde tekst staat in 2020+A1, 2023 en 2024: geen schakelpunt.
- **Bijlage D.** Bijlage D heeft geen formule voor deze combinatie. Beslissing: de vloer wordt in twee delen gerekend, naar de opbouw van 8.39 (vloer- en wandterm opgeteld) en opmerking 3 van 8.3.2 (een deels onder maaiveld gelegen vloer wordt gesplitst en elk deel apart gerekend). Het vloerdeel volgt D.13/D.14 (kruipruimte, tabel D.1 (0, 0)) of D.15/D.16 zonder de term h·P·U_xw (onverwarmde kelder, (0, 1)), met U_x zonder U_x;T. Het wanddeel volgt de wandtermen van D.10 en D.11 (2022 p. 766; 2025+C1 p. 794), tabel D.1 "alle overige gevallen" (0, 1), zoals bij een verwarmde kelder. De randtermen van 8.36/8.37/8.38 horen bij het vloerdeel, ΔU_for van 8.38 over z·P bij het wanddeel. De vloerterm e^(−z/δ) van D.11 vervalt: de kruipruimtevloer staat al in D.14.
- **Codering.** De vloer van EP-W006a (R_c 0,15, R_bf 6, openingen 0,0012 m²/m) met `heatedBasement` z = 0,5 m en R_c 6,0 van de gevel. De onderste 0,5 m van elke gevel grenst aan grond en zit in de wandterm; de bruto geveloppervlakken worden 0,5 m × de gevellengte kleiner. De ramen van de onderste bouwlaag beginnen op 0,5 m en blijven buitenlucht.

De 11 realistische gebouwen (EPWRealB01 tot en met EPWRealD05 en EPURealB01) zijn niet te coderen zolang de bijlagen 3A–3K niet in bezit zijn.

| Test | Onderwerp | Deeltesten | Gecodeerd | Open | Waarom open |
|---|---|---|---|---|---|
| EPW001 | Referentie | 1 | EPW001 | – |  |
| EPW002 | Isolatie | 3 | a, b, c | – |  |
| EPW003 | Raameigenschappen | 3 | a, b, c | – |  |
| EPW004 | Oriëntatie | 7 | a, b, c, d, e, f, g | – |  |
| EPW005 | Thermische massa | 3 | a, b, c | – |  |
| EPW006 | Begrenzing begane grondvloer | 8 | a, b, c, d, e, f, g, h | – |  |
| EPW007 | Infiltratie | 3 | a, b, c | – |  |
| EPW008 | Overstek | 5 | a, b, c, d, e | – |  |
| EPW009 | Zijbelemmeringen | 6 | a, b, c, d, e, f | – |  |
| EPW010 | Belemmering | 5 | a, b, c, d, e | – |  |
| EPW011 | Zonwering | 2 | a, b | – |  |
| EPW012 | Zonwering en oriëntatie | 6 | a, b, c, d, e, f | – |  |
| EPW013 | Gebruiksoppervlak | 5 | a, b, c, d, e | – |  |
| EPW014 | Dakvorm | 1 | a | – |  |
| EPW015 | Vertikale leidingen | 2 | a, b | – |  |
| EPW016 | Deuren | 3 | a, b, c | – |  |
| EPW101 | Ventilatiesysteem | 23 | a t/m w (alle) | – |  |
| EPW102 | Voorverwarming nat. toev.vent. | 2 | a, b | – |  |
| EPW103 | Zomernachtventilatie | 3 | a, b, c | – |  |
| EPW104 | Ventilatie overig | 6 | a, b, c, d, f, g | – |  |
| EPW201 | Afgifte | 5 | a, b, c, d, e | – |  |
| EPW202 | Distributie | 6 | a, b, c, d, e, f | – |  |
| EPW203 | Opwekking | 16 | a, b, c, d, e, f, g, h, i, j, l, m, n, o, p | q | q: kwaliteitsverklaring uit een apart bijlagedocument (niet in bezit) |
| EPW204 | Opwekking2 | 9 | a, b, c, e, f, g, h, i | d | d: niet te coderen, zie boven |
| EPW205 | Opwekking gemeenschappelijk | 2 | a, c | – |  |
| EPW206 | Opwekking woongebouw | 3 | a, b, c | – |  |
| EPW301 | Afgifte | 3 | a, b, d | – |  |
| EPW302 | Distributie | 6 | a, b, c, d, e, f | – |  |
| EPW303 | Opwekking | 5 | a, b, c, d, e | – |  |
| EPW401 | Afgifte | 2 | a, b | – |  |
| EPW402 | Distributie | 7 | a, b, c, d, e, f, g | – |  |
| EPW403 | Douche WTW | 4 | a, b, c, d | – |  |
| EPW404 | Voorraadvat | 4 | a, b, c, d | – |  |
| EPW405 | Zonneboiler | 6 | a, b, c, d, e, f | – |  |
| EPW406 | Opwekking | 21 | a, b, c, d, e, f, g, i, k, m, n, o, p, q, t, u, v | j, l, w, x | j: booster met één COP, geen route (zie boven); l: biomassa die niet aan bijlage R voldoet (geen route, zie EP-W204d); w, x: verklaringen uit aparte bijlagedocumenten (niet in bezit) |
| EPW407 | Opwekking gemeensch./woong. | 9 | a, b, c, d, e, f, g, h, i | – |  |
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
| EPU602 | Regeling | 9 | a, b, c, d, e, f, g, h, i | – |  |
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
7. **Testsetdossier.** `scripts/release-nta.sh` schrijft per vrijgave `release/<tag>/testset/` met elke deeltest als projectbestand en een uitkomstenoverzicht (zie het [testprotocol](nta8800-testprotocol.md), §3). Zodra de suite `expected` heeft, vult het overzicht de kolommen *verwacht* en *oordeel* zelf. Bij een nieuwe testreeks het onderwerp toevoegen aan `training-data/isso54/onderwerpen.json` (eigen woorden).
8. **Afwijkingen verklaren.** Elke afwijking boven de band verklaren en vastleggen. Ligt de oorzaak in de testset, dan dat melden aan InstallQ en ISSO.

## Verschillen tussen versie 2.0 en 5.0

Over de inhoud van versie 5.0:2026 is niets openbaar gevonden (gezocht op 9 oktober 2026). Bekend is alleen wat BRL 9501 (29-05-2026) er zelf over zegt: de indeling in deelgebieden, hoofdgroepen, tests en deeltesten, en een bandbreedte per (deel)test (§3.3 opm., p. 6).

Te verwachten, maar niet bevestigd:
- de overgang van NTA 8800:2022 naar 2025+C1, met de schakelpunten in [normversies](nta8800-normversies.md);
- deeltesten voor routes die sinds 2022 zijn bijgekomen, zoals de bronregels van warmtepompen, flexmodus en de LED-kolom.

