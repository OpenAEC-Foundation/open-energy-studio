# Globale beschrijving van het NTA 8800-rekenprogramma

**Stand:** 10 oktober 2026, op `nta8800-kernel`.
**Status:** werkdocument voor de attesteringsinstelling. Het programma is niet geattesteerd.

BRL 9501 van 29-05-2026 vraagt bij de aanvraag naast de handleiding een globale beschrijving van het rekenprogramma (§3.0, p. 5). Dit is die beschrijving. Ze geeft het overzicht en verwijst voor de details naar de deeldocumenten in `docs/`. Eisen uit de BRL worden alleen met artikel en pagina genoemd.

## 1. Wat het programma is

Open Energy Studio is een rekenprogramma voor de energieprestatie van gebouwen volgens NTA 8800:2025+C1:2026. Het maakt BENG-toetsen (Bbl art. 4.149 en 4.149b), energielabels (Omgevingsregeling art. 5.11–5.14) en maatwerkadviezen (BRL 9500-MWA). Het programma is open source.

**Wat de BRL attesteert.** De BRL attesteert de berekeningsmethode, dus de rekenkern; de in- en uitvoerinterface valt erbuiten, de uitvoereisen niet (§2.3, p. 3). Bij dit programma is de rekenkern de Rust-bibliotheek `crates/nta8800-core`. De versie ervan is `KERNEL_VERSION`.

**Gebruiksfuncties en deelgebieden** (§2.1, p. 2):
- alle gebruiksfuncties van de BRL, voor woningbouw en utiliteitsbouw;
- deelgebied 1, energieprestatie;
- deelgebied 2, energiegebruik en besparingsmaatregelen.

Deelgebied 3, financiële kengetallen, is in de BRL uitgesteld. Wat er per functie en route wel en niet is, staat in het [dekkingsregister](nta8800-dekkingsregister.md) en in hoofdstuk 01 van de [handleiding](handleiding-nta8800/01-reikwijdte-en-status.md).

## 2. Opbouw

| Laag | Waar | Taak |
|---|---|---|
| Rekenkern | `crates/nta8800-core` | De volledige NTA 8800-keten: hoofdstuk 5 tot en met 17 en de bijlagen, de normversies, de basis- en detailopname (`src/opname/`), het label, de registratiecontrole, het maatwerkadvies en het referentieharnas. Deterministisch, zonder netwerk of bestandstoegang. |
| Service | `crates/nta8800-service` | HTTP-API (OpenAPI 3.1 op `/v1/openapi.json`) en MCP-server op één register van operaties. Elke aanroep loopt via de rekenkern. Zie [API](nta8800-api.md) en [MCP](nta8800-mcp.md). |
| Desktop-app | `src-tauri` | Tauri-app met dezelfde operaties als opdrachten. De normversie wordt per aanroep op dezelfde manier toegepast als in de service (`norm_versions::request`). |
| Gebruikersinterface | `src/` (React) | Invoer per stap, de opname, het projectmodel, de rapporten, het projectdossier en de registratiecontrole. De interface rekent zelf niets uit dat in het label komt. |

De interface is alleen een invoer- en weergavelaag. Elke uitkomst die in een rapport, label of dossier komt, komt uit de rekenkern. Er is geen tweede rekenroute in TypeScript.

## 3. Van invoer tot registratie

1. **Invoer.** De adviseur voert het project in, of neemt een basis- of detailopname over in het projectmodel. Elk invoerveld kan een bron en bewijsstuk krijgen (Bron & bewijs). Een import uit een ander programma wordt in het projectbestand gelogd, zoals §4.3.1 opmerking (p. 8) vraagt.
2. **Berekening.** De app zet het project om naar de kerninvoer en roept `calculate_project_performance` aan. De kern geeft:
   - de indicatoren, met de tussenresultaten tot op rekenniveau;
   - de labelgegevens van art. 5.13 en 5.13a;
   - de Bbl-toets;
   - de invoergaten met hun pad;
   - een status (`calculated_unverified`, `incomplete`, `invalid`, of `calculated_legacy_edition` voor een oudere normversie).
3. **Stempel.** Elke berekening draagt `KERNEL_VERSION`, de normversie en een vingerafdruk van de invoer. Het projectbestand bewaart het stempel. Een herlabel met een andere rekenkern wordt geweigerd.
4. **Rapport.** Het rekenrapport heeft drie niveaus: samenvatting, standaard, en gedetailleerd tot op rekenniveau. Het bevat de bijlage "Interpretaties" (`src/core/report/EnergyPerformanceReport.ts`).
5. **Registratiecontrole.** De kern toetst het registratieblok aan BRL 9500-W/U en de Omgevingsregeling:
   - termijnen, berichttype en BAG;
   - WLC-GWP;
   - de verklaringen bij labelelementen k en l;
   - herlabelen en bewijsstukken.
   Een oudere normversie is nooit registreerbaar. Het programma moet een attestnummer dragen voordat registratie mogelijk is.
6. **Dossier.** De dossierexport bundelt invoer, kernuitvoer, rapport, bewijsstukken met SHA-256 en het EP-Online-gegevensoverzicht. Zie het [attestdossier](nta8800-attestdossier.md). Het uploadformaat van EP-Online is niet openbaar. Het gegevensoverzicht volgt daarom het openbare exportschema en is geen registratiebestand.

## 4. Normversies en interpretaties

- **Normversies.** De rekenkern rekent standaard volgens NTA 8800:2025+C1:2026, de aangewezen versie. Voor vergelijking en herberekening rekent ze ook volgens 2024, 2023, 2022 en 2020+A1. De verschillen staan als genummerde schakelpunten, met paginaverwijzingen, in [normversies](nta8800-normversies.md). Een oudere versie geeft status `calculated_legacy_edition` en is niet registreerbaar. Elk bronlabel in uitvoer, rapport en dossier noemt de berekende versie. Wie de versie van een project wisselt, krijgt eerst van de kern (`compare_project_editions`) wat de nieuwe versie weigert, eist, negeert of anders leest, elk met zijn schakelpunt; alleen veilige wijzigingen gaan automatisch. Op de openbare gevallen is elk verschil van meer dan 0,5 % tussen opeenvolgende versies herleid tot een schakelpunt.
- **Interpretaties.** Waar de norm meer dan één lezing toelaat, legt de kern haar keuze vast in `kernel_interpretations()`. Die keuzes staan in elk gedetailleerd rapport.
- **Open vragen aan NEN.** De open vragen, met name 10.15, 10.87, f_prac, de luchtstroom van de forfaitaire ventilatorenergie (vraag 5) en de verdeling van gemeenschappelijke ruimten over tapwater en verlichting (vraag 6), staan in [vragen aan NEN](nta8800-vragen-nen.md).
- **Vergelijking met andere programma's.** Hoe de uitkomsten zich verhouden tot die van andere programma's staat in de [vergelijking met openbare rapporten](nta8800-vergelijking-openbare-rapporten.md) en de [vergelijking met de RVO-voorbeeldwoningen](nta8800-vergelijking-rvo-voorbeeldwoningen.md).

## 5. Verificatie

De gate `scripts/verify-nta.sh` loopt bij elke wijziging. Er wordt alleen vastgelegd als hij slaagt. Hij draait:

- de unittests van de kern, met paginaverwijzingen naar de norm;
- de differentiële tests tussen de normversies, de controle dat elk bronlabel de berekende versie noemt (`tests/edition_labels.rs`) en het overzicht bij een versiewissel (`tests/edition_switch.rs`);
- de **optiedekkingstest** (`crates/nta8800-core/tests/option_coverage.rs`). Die leidt elke invoeroptie af uit de invoertypen zelf en rekent haar door in alle vijf normversies. Elke run moet rekenen of weigeren met een benoemde code met tekst, zonder paniek en zonder niet-eindige getallen.
- de **openbare gevallen** A–O, waarvan H–L, N en O utiliteitsgebouwen (`tests/public_comparison.rs`), nagerekend in hun eigen normversie;
- de **numerieke fuzz** (`tests/numeric_fuzz.rs`): elk getal uit de voorbeelden en openbare gevallen krijgt randwaarden, in alle vijf normversies, met controles op eindigheid, plausibiliteit en monotonie;
- de **referentiesuites** (`reference_gate`): de openbare gevallen en de RVO-voorbeeldwoningen met een oordeel, en de gecodeerde ISSO 54-deeltesten (260 van 266, versie 2.0) die rekenen zonder verwachting (`pending_expectation`);
- de **formulierbewaking**: ongeveer 2 400 toestanden van de projectformulieren en alle opnamesecties gaan door de kern; geen afgekeurde vorm, geen achtergebleven veld, elk gat met een route;
- de robuustheidstests, de schaaltests (hoofdstuk 8) en de gelijkwaardigheid van de zoektocht naar lege velden met de oude zoektocht (`tests/blank_search_equivalence.rs`);
- de service- en API-tests;
- de tests van de desktop-opdrachten;
- de frontendtests (139 bestanden), waaronder de controle dat elke meldcode van de kern een Nederlandse en Engelse tekst heeft, en een toegankelijkheidscontrole van alle NTA-formulieren en dialogen (namen, foutkoppeling, *Ga naar* en toetsenbord).

Daarnaast zijn er:
- **MSRV-test.** De kern wordt ook getest met Rust 1.77.2, de oudste ondersteunde versie.
- **Onafhankelijke herberekening.** Elke route is apart herberekend; de stand staat in de [verificatiestatus](nta8800-verificatiestatus.md).
- **Referentieharnas.** Het [referentieprotocol](nta8800-referentieprotocol.md) beschrijft het harnas. De gate draait het al op de gecodeerde ISSO 54-deeltesten, zonder oordeel; zodra de uitkomsten van versie 5.0 er zijn, worden die als verwachting ingevuld ([voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md)).
- **Testprotocol.** Het [testprotocol](nta8800-testprotocol.md) beschrijft alle testniveaus met de opdrachten, de handmatige acceptatietests vóór een vrijgave, de regressieprocedure en de codereviews; per vrijgave komt een testrapport in het archief.
- **Builds.** De build-verificaties per commit staan in `docs/nta8800-build-verificatie-*.md`, en de procedure in de [buildprocedure](nta8800-build-procedure.md).

## 6. Versiebeheer

- **Versienummers.** Het programma heeft een programmaversie en een rekenkernversie, beide volgens `MAJOR.MINOR.PATCH`. Hoe ze verhogen, hoe versies uitwisselbaar blijven en hoe oude versies bewaard worden, staat in [versiebeheer](nta8800-versiebeheer.md) (§4.3, p. 7; §5.2–5.3, p. 9).
- **Releasenotes.** Elke wijziging die een uitkomst of status verandert, staat in de [releasenotes](nta8800-releasenotes.md).
- **Kwaliteitssysteem.** Het [kwaliteitshandboek](kwaliteit/kwaliteitshandboek.md) bundelt de procedures voor hoofdstuk 5 en 6 van de BRL (p. 9–10): [melden van wijzigingen](kwaliteit/meldprocedure-wijzigingen.md), [klachten](kwaliteit/klachtenprocedure.md), het [register van licentiehouders](kwaliteit/register-licentiehouders.md) en het [archiefbeleid](kwaliteit/archiefbeleid.md). Het zijn concepten tot de attesthouder ze vaststelt.
- **Attestnummer.** Het programma leest zijn BRL 9501-attestnummer uit één accessor (`src/core/nta/Attest.ts`). Zolang dat leeg is:
  - toont de app geen NL-EPBD-merk;
  - meldt het rapport "niet geattesteerd";
  - is registratie niet mogelijk.

## 7. Beperkingen

- **Geen attest.** Het programma is niet geattesteerd. Alle uitkomsten hebben status `calculated_unverified`, of `calculated_legacy_edition` in een oudere normversie.
- **Geen testset.** De ISSO 54-testset versie 5.0:2026 met verwachte uitkomsten is nog niet ingezien. Overeenstemming binnen de bandbreedtes is dus nog niet aangetoond (§4.2, p. 7).
- **Geen registratiebestand.** Registratie in EP-Online is niet mogelijk zonder het uploadformaat van RVO en een attest.
- **Open interpretatie koeling.** De koelroute volgt 10.15 en 10.87 letterlijk. Dat wijkt af van ten minste één ander programma, zie de vragen aan NEN.
- **Onvolledige routes.** Het uurklimaat van 17.3.8 ontbreekt (daarvoor zijn de uurwaarden van NEN 5060 nodig). Dunne spouwen onder 20 mm rekenen via D.2, behalve zwak geventileerde spouwen en spouwen met reflecterende folie ([verificatiestatus](nta8800-verificatiestatus.md)).
- **Wat er nog openstaat voor het attest.** De volledige lijst staat in de [BRL 9501-gereedheid](nta8800-brl9501-gereedheid.md).

## 8. Grenzen en prestaties

Gemeten op 10 oktober 2026 (rekenkern 0.4.0, release-build, 8 kernen) met `crates/nta8800-core/tests/large_projects.rs`. Die test maakt deterministisch grote projecten:
- **utiliteit:** openbaar geval H (2 rekenzones, 18 ramen) n keer herhaald, met nieuwe ids voor elke kopie;
- **woningen:** de voorbeeld-rijwoning als n woningen van elk één rekenzone, met `dwellingCount` = n (6.2b, p. 160);
- **dicht utiliteitsgebouw:** geval H met zijn gemeenschappelijke ruimte als invoer (6.6.4), 25 keer herhaald tot 50 rekenzones. Elk raam is in vijf gesplitst (2 250 ramen, elk raam naar buiten met eigen zonwering, en een belemmering waar geval H er een heeft). Verder 1 200 lineaire koudebruggen, het PV-veld verdeeld over 30 systemen en de tweede helft van de zones op een extra verwarmingssysteem;
- **opname:** de opname van het kantoor uit 1985 met elk raam in n delen gesplitst.

| Project | Zones | Ramen | Rekentijd | Uitvoer (JSON) | Piekgeheugen testproces |
|---|---:|---:|---:|---:|---:|
| utiliteit | 2 | 18 | 22 ms | 207 kB | 14 MB |
| utiliteit | 20 | 180 | 76 ms | 1,5 MB | 25 MB |
| utiliteit | 40 | 360 | 150 ms | 2,9 MB | 40 MB |
| utiliteit | 80 | 720 | 343 ms | 5,8 MB | 67 MB |
| utiliteit | 200 | 1 800 | 883 ms | 14 MB | 144 MB |
| woningen | 50 | 100 | 97 ms | 2,7 MB | – |
| woningen | 200 | 400 | 425 ms | 10 MB | – |
| dicht utiliteitsgebouw, 2020+A1 | 50 | 2 250 | 423 ms | 7,5 MB | – |
| dicht utiliteitsgebouw, 2022 | 50 | 2 250 | 419 ms | 7,5 MB | – |
| dicht utiliteitsgebouw, 2023 | 50 | 2 250 | 424 ms | 7,5 MB | – |
| dicht utiliteitsgebouw, 2024 | 50 | 2 250 | 433 ms | 7,5 MB | – |
| dicht utiliteitsgebouw, 2025+C1 | 50 | 2 250 | 477 ms | 7,5 MB | – |
| geval H × 25, ramen × 1 | 50 | 450 | 207 ms | 3,7 MB | – |
| geval H × 25, ramen × 10 | 50 | 4 500 | 844 ms | 12 MB | – |
| geval H × 25, ramen × 20 | 50 | 9 000 | 1 511 ms | 21 MB | 257 MB |

- **Lineair.** Rekentijd, uitvoer en geheugen groeien evenredig met het aantal zones (ongeveer 4 ms, 72 kB uitvoer en 0,7 MB geheugen per utiliteitszone) en met het aantal ramen (ongeveer 0,15 ms per raam met zonwering). Alle vijf uitgaven rekenen het dichte gebouw even snel. Een complete berekening heeft geen kwadratisch gedrag.
- **Lege velden.** Een project met lege velden in het NTA-blok rekent niet; de kern zoekt dan elk leeg veld op (`blank_paths` in `project_performance.rs`). Die zoektocht leest het NTA-blok opnieuw voor elke poging. Sinds 10 oktober 2026 doet ze dat op een verkleinde kopie: elke lijst van objecten (ramen, zonwering, koudebruggen, …) houdt alleen haar eerste element en de elementen met een leeg veld. De weggelaten elementen bevatten geen leeg veld, de zoektocht verandert ze dus nooit, en elk lijstelement wordt los van de andere gelezen. Daarna controleert de kern de uitkomst op het hele blok: de gerepareerde kopie met de weggelaten elementen terug moet leesbaar zijn, elk gemeld leeg veld moet het blok op zichzelf laten falen (sinds review 27; eerst alleen alle gemelde samen), en alle andere lege velden samen niet. Slaagt een controle niet, of komt de verkleinde zoektocht niet rond, dan zoekt de kern op het hele blok zoals voorheen. In het dichte gebouw (release-build):

  | Lege velden | Hele blok | Verkleinde kopie |
  |---:|---:|---:|
  | 1 | 0,2 s | 0,14 s |
  | 10 | 1,1 s | 0,19 s |
  | 50 | 4,9 s | 0,33 s |
  | 200 | 19,6 s | 1,6 s |

  De controle per leeg veld kost ongeveer 0,4 s bij 200 lege velden. De tijd volgt nu het aantal lege velden en niet meer het aantal lege velden maal de grootte van het blok. De zoektocht op het hele blok blijft in de kern als orakel. De uitkomst is gelijk aan die van het orakel, getoetst op alle voorbeelden en openbare gevallen met hun varianten met lege velden, op 600 willekeurige sets lege velden (vast zaadgetal), op het dichte gebouw en op elke formulierstand van de projectformulierwacht (`blank_search_equivalence.rs`, `large_projects.rs`, `project_form_guard.rs`). Review 27 voegde lijstvormen toe (een leeg veld in het eerste element, geneste lijsten, een leeg element, een lege lijst), weggelaten elementen die op zichzelf fout zijn (de kern valt dan terug op het hele blok) en een lijst waarvan de elementen van elkaar afhangen. `custom_deserialization_in_the_kernel_is_known` faalt zodra de kern een eigen leesregel (`impl Deserialize`, `deserialize_with`, `untagged`, `flatten`, `from`) krijgt die nog niet bekeken is. De uitvoer van de 494 varianten van de snapshot is byte voor byte gelijk aan die van de vorige versie. Een klein project blijft snel: geval H (2 zones) met 40 lege velden rekent in 27 ms.
- **Opname.** De opname van het kantoor uit 1985 met 400 ramen rekent in 33 ms. In een meting met tijden per fase (904 ms in totaal voor 200 zones, door de meting zelf iets trager dan de 759 ms in de tabel) gaat 553 ms naar de gebouwberekening zelf; de rest is afleiding, controle en de eindige-getallencontrole van de uitkomst. Er was dus geen algoritmische correctie nodig.
- **Uitkomst bij schalen.** Een gebouw dat n keer herhaald wordt, houdt dezelfde energiebehoefte per m² (BENG 1). Verwarming, ventilatie en verlichting worden precies n keer zo groot. Tapwater, koeling en hulpenergie groeien minder, omdat ze vaste posten per systeem hebben (het verlies van het voorraadvat, de 87,6 kWh regelenergie van 10.87). De test controleert dit.
- **Invoergrootte.** Een utiliteitszone is ongeveer 11 kB invoer. De standaardlimiet van de HTTP-API (16 MB) laat dus ruwweg 1 500 zones toe. Geëxtrapoleerd kost zo'n verzoek ongeveer 7 s en 1 GB geheugen.
- **HTTP-API.** De API begrenst daarom het aantal gelijktijdige berekeningen (`--max-calculations`, standaard het aantal kernen) en de tijd per verzoek (`--calculation-timeout-s`, standaard 120 s). Daarna volgt 503 met `server_busy` of `calculation_timeout` (zie [HTTP-API](nta8800-api.md)). Wie de API buiten de eigen computer aanbiedt, stelt beide en `--body-limit-mb` lager in. Het dichte gebouw (68 000 JSON-waarden) blijft ver onder de standaardlimiet van 500 000 waarden en ver onder de rekentijdslimiet; ook 200 lege velden in dat gebouw (1,6 s) passen erin.
- **MCP en desktop.** De MCP-server en de desktop-app rekenen met dezelfde release-build, zonder eigen tijdslimiet: de desktop-app rekent in een aparte thread (`spawn_blocking`), zodat de interface blijft reageren terwijl een groot project rekent.
- **Bewaking in de gate.** `large_projects.rs` rekent in elke gate het dichte gebouw (budget 40 s in een debug-build; gemeten 3,4 s), 200 lege velden in dat gebouw (budget 60 s; gemeten 6,9 s) en de opname met 400 ramen (budget 15 s; gemeten 0,3 s). Een algoritme dat kwadratisch wordt, valt zo op.
- **Interface.** In de jsdom-testomgeving tekent de pagina Schil van een project met 40 zones en 360 ramen in ongeveer 3 s (0,4 s bij 2 zones), de overige invoer- en resultaatpagina's in minder dan 0,3 s. Ook dit groeit lineair. Een echte browser is sneller dan jsdom. De test `src/__tests__/large-project-ui.test.tsx` bewaakt dit met een ruime tijdsgrens.
- **Zelf meten.** `cargo test --release --offline --test large_projects -- --ignored --nocapture` in `crates/nta8800-core` drukt de tabel hierboven af.
