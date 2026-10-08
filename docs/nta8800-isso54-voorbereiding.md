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
| Oordeel zonder verwachtingen | Een run met alleen deze suite vergelijkt niets: het rapport zegt "geen vergelijking (0 vergeleken, 52 zonder verwachting)" en de afsluitcode is 3, niet 0. In de gate lopen de openbare gevallen en de RVO-woningen mee; die vergelijken wel. |

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
- **EP-U002j.** 30 % kantoor en 70 % overige bijeenkomst in één rekenzone. Ventilatie, tapwater en verlichting zijn per oppervlak verdeeld. De gebruiksfunctie van de zone, voor setpoints en interne warmte, is de grootste: overige bijeenkomst.

## Stand van de codering

52 van de 266 deeltesten uit bijlage 1 zijn gecodeerd: 38 voor woningen en 14 voor utiliteit. Elke gecodeerde deeltest rekent in de gate. Bij de open deeltesten heeft de kern in vrijwel alle gevallen de route al. Ze zijn nog niet gecodeerd, omdat geometrie of productgegevens uit de beschrijving moeten worden afgeleid.

Twee groepen zijn te controleren, omdat het niet zeker is dat de route bestaat:
- EP-W003c: rolluik met extra warmteweerstand;
- EP-W016: deuren.

De 11 realistische gebouwen (EPWRealB01 tot en met EPWRealD05 en EPURealB01) zijn niet te coderen zolang de bijlagen 3A–3K niet in bezit zijn.

| Test | Onderwerp | Deeltesten | Gecodeerd | Open | Waarom open |
|---|---|---|---|---|---|
| EPW001 | Referentie | 1 | EPW001 | – |  |
| EPW002 | Isolatie | 3 | – | a, b, c | route aanwezig: per element een eigen constructie, glas en bouwjaar; c: lineaire bruggen per aansluiting (`thermalBridges`) |
| EPW003 | Raameigenschappen | 3 | b | a, c | a: U-raam uit U_gl, U_kozijn en Ψ_glas via 8.15 (`window_u`), route aanwezig; c: rolluik met extra R 0,2: te controleren welke route NTA 8800:2022 hiervoor heeft |
| EPW004 | Oriëntatie | 7 | a, b, c, d, e, f, g | – |  |
| EPW005 | Thermische massa | 3 | a, b, c | – |  |
| EPW006 | Begrenzing begane grondvloer | 8 | – | a, b, c, d, e, f, g, h | route aanwezig voor kruipruimte (a–e), deels ingegraven gevels (f, g) en AOR (h); geometrie en kruipruimtegegevens nog te coderen |
| EPW007 | Infiltratie | 3 | a, b, c | – |  |
| EPW008 | Overstek | 5 | – | a, b, c, d, e | overstekken per raam (`windowObstructions`): hoeken uit de figuren 2–3 nog af te leiden |
| EPW009 | Zijbelemmeringen | 6 | – | a, b, c, d, e, f | zijbelemmeringen per raam (`windowObstructions`): hoeken uit figuur 4 nog af te leiden |
| EPW010 | Belemmering | 5 | – | a, b, c, d, e | belemmering (`windowSolar.obstruction`): nog te coderen |
| EPW011 | Zonwering | 2 | – | a, b | beweegbare zonwering (tabel 7.5/7.6): nog te coderen |
| EPW012 | Zonwering en oriëntatie | 6 | – | a, b, c, d, e, f | zonwering met oriëntatie: nog te coderen |
| EPW013 | Gebruiksoppervlak | 5 | – | a, b, c, d, e | andere gebruiksoppervlakte: geometrie nog te coderen |
| EPW014 | Dakvorm | 1 | – | a | hellend dak (`surfaceTilts`): nog te coderen |
| EPW015 | Vertikale leidingen | 2 | – | a, b | verticale leidingen (`verticalPipes`): nog te coderen |
| EPW016 | Deuren | 3 | – | a, b, c | deuren: te controleren hoe een deur in het projectmodel wordt ingevoerd (als schildeel met eigen U) |
| EPW101 | Ventilatiesysteem | 23 | a, b, d, e, f, g, h, i, j, k, l, m, n, o, p, q, t, u, v | c, r, s, w | c: B1 met bouwjaar 1985 en wisselstroomventilatoren; r/s: WTW-rendement uit een verklaring (EN 13141-7 / EN 13142); w: E.1 met twee delen. Routes aanwezig, nog te coderen |
| EPW102 | Voorverwarming nat. toev.vent. | 2 | – | a, b | voorverwarming natuurlijke toevoer (`grillePreheating`): nog te coderen |
| EPW103 | Zomernachtventilatie | 3 | – | a, b, c | zomernachtventilatie (`ventilativeCooling`) met doorlaten en hoogten: nog te coderen |
| EPW104 | Ventilatie overig | 6 | – | a, b, c, d, f, g | open verbrandingstoestellen, badgeiser, ventilatorvermogen, D.4a in een meergezinswoning: routes aanwezig, nog te coderen |
| EPW201 | Afgifte | 5 | a, e | b, c, d | b: radiatoren met netwerkregeling; c/d: convectoren met boosterventilatoren (`emission.fans`): nog te coderen |
| EPW202 | Distributie | 6 | – | a, b, c, d, e, f | leidingen in onverwarmde ruimten, pomp, leidingen in de constructie (e: te controleren), collectieve ketel (f): nog te coderen |
| EPW203 | Opwekking | 16 | b, c | a, d, e, f, g, h, i, j, l, m, n, o, p, q | lokale gasverwarming (a), warmtepompen (d–i, l–q, deels met kwaliteitsverklaring uit bijlagen 4–6), ketel met verklaring (j), meer rekenzones (l, m), luchtverwarmer (o): routes aanwezig, nog te coderen |
| EPW204 | Opwekking2 | 9 | – | a, b, c, d, e, f, g, h, i | warmtelevering, biomassa, micro-WKK: routes aanwezig, nog te coderen |
| EPW205 | Opwekking gemeenschappelijk | 2 | – | a, c | collectieve opwekking: nog te coderen |
| EPW206 | Opwekking woongebouw | 3 | – | a, b, c | opwekking in een woongebouw: nog te coderen |
| EPW301 | Afgifte | 3 | – | a, b, d | koeling: de referentiewoning heeft geen koeling; af te leiden van EP-U001, nog te coderen |
| EPW302 | Distributie | 6 | – | a, b, c, d, e, f | koeldistributie: nog te coderen |
| EPW303 | Opwekking | 5 | – | a, b, c, d, e | koelopwekking: nog te coderen |
| EPW401 | Afgifte | 2 | – | a, b | tapwaterafgifte: nog te coderen |
| EPW402 | Distributie | 7 | – | a, b, c, d, e, f, g | tapwaterdistributie en circulatie: nog te coderen |
| EPW403 | Douche WTW | 4 | – | a, b, c, d | douche-WTW: nog te coderen |
| EPW404 | Voorraadvat | 4 | – | a, b, c, d | voorraadvat: nog te coderen |
| EPW405 | Zonneboiler | 6 | – | a, b, c, d, e, f | zonneboiler (`solarThermal`): nog te coderen |
| EPW406 | Opwekking | 21 | – | a, b, c, d, e, f, g, i, j, k, l, m, n, o, p, q, t, u, v, w, x | tapwateropwekkers, deels met kwaliteitsverklaring (bijlagen 5 en 6): nog te coderen |
| EPW407 | Opwekking gemeensch./woong. | 9 | – | a, b, c, d, e, f, g, h, i | collectieve tapwateropwekking: nog te coderen |
| EPW501 | PV-panelen | 4 | – | a, b, c, d | PV-panelen (`pvSystems`): nog te coderen |
| EPU001 | Referentie | 1 | EPU001 | – |  |
| EPU002 | Gebruiksfunctie | 10 | a, b, c, d, e, f, g, h, i, j | – |  |
| EPU102 | Zonwering | 3 | a, b, c | – |  |
| EPU201 | Ventilatiesysteem | 2 | – | a, c | geïnstalleerde capaciteit en recirculatie: nog te coderen |
| EPU202 | AHU | 3 | – | a, b, c | luchtbehandelingskast in een groter gebouw (8 bouwlagen, nieuwe geometrie): nog te coderen |
| EPU203 | Ventilatoren | 3 | – | a, b, c | ventilatorvermogen en terugregeling: nog te coderen |
| EPU301 | Afgifte | 1 | – | a | hoge ruimte (10 m, `edition2023` high_room): nog te coderen |
| EPU302 | Distributie | 2 | – | a, b | leidinglengte en bouwjaar 1950: nog te coderen |
| EPU303 | Opwekking | 7 | – | a, b, c, d, e, f, g | opwekkers utiliteit: nog te coderen |
| EPU401 | Afgifte | 1 | – | a | koelafgifte: nog te coderen |
| EPU402 | Distributie | 1 | – | a | koeldistributie: nog te coderen |
| EPU403 | Opwekking | 1 | – | a | koelopwekking: nog te coderen |
| EPU501 | Afgifte | 1 | – | a | tapwaterafgifte: nog te coderen |
| EPU502 | Distributie | 3 | – | a, b, c | tapwaterdistributie: nog te coderen |
| EPU503 | Douche wtw | 1 | – | EPU503 | douche-WTW: nog te coderen |
| EPU504 | Opwekking | 1 | – | EPU504 | tapwateropwekking: nog te coderen |
| EPU601 | Vermogen | 3 | – | a, b, c | verlichtingsvermogen: nog te coderen |
| EPU602 | Regeling | 9 | – | a, b, c, d, e, f, g, h, i | verlichtingsregelingen: nog te coderen |
| EPU701 | Bevochtiging | 4 | – | a, b, c, d | bevochtiging (`humidification`): nog te coderen |
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

