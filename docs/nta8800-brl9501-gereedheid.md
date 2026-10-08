# BRL 9501-gereedheid van het NTA 8800-rekenprogramma

**Stand:** 8 oktober 2026, op `nta8800-kernel` (commit `4cb71cf`).
**Status:** werkdocument, geen attest en geen verklaring van conformiteit.

Dit document loopt BRL 9501 eis voor eis na en zet ernaast wat het programma nu aantoonbaar doet. Het doel is te laten zien wat een attest nog in de weg staat.

## Gebruikte bronnen

- [BRL 9501 van 29-05-2026](https://installq.nl/files/BRL-en/brl-9501-installq-2026-aangewezen.pdf), de aangewezen versie. Paginanummers zijn de gedrukte nummers ("Pagina - n -").
- BRL 9500-W/U en BRL 9500-MWA-W/U. Die zijn per eis al uitgewerkt in het [attestdossier](nta8800-attestdossier.md), dat hieronder alleen wordt samengevat.

De eisen zijn in eigen woorden weergegeven. Er is geen tekst uit de BRL overgenomen.

**Statussen:**
- **voldaan:** aantoonbaar aanwezig in code of documentatie.
- **deels:** aanwezig, maar onvolledig of niet aantoonbaar zoals de BRL het vraagt.
- **open:** ontbreekt en kan door de ontwikkelaar zelf worden ingevuld.
- **extern:** hangt af van een derde (ISSO, RVO, NEN, de attesteringsinstelling).
- **n.v.t.:** niet van toepassing op dit moment.

Waar de BRL een eis aan de **attesthouder als organisatie** stelt (hoofdstukken 5 en 6), is de status ook "open" als het om een organisatorische stap gaat. De prioriteitenlijst onderaan maakt het onderscheid tussen bouwen, extern en organisatie.

**Wat het attest omvat.** De BRL attesteert de *berekeningsmethode*: de rekenkern. De in- en uitvoerinterface valt daarbuiten, maar de uitvoereisen horen er wel bij (§2.3, p. 3). Voor dit programma is dat `crates/nta8800-core`, met daaromheen de service, de desktop-app en de rapporten.

## 1. Eisentabel BRL 9501

### Reikwijdte en procedure (hoofdstukken 2 en 3)

| Eis (BRL 9501) | Inhoud in eigen woorden | Status | Bewijs in de repo | Wat ontbreekt |
|---|---|---|---|---|
| §2.1, p. 2 | Het attest dekt de gebruiksfuncties woonfunctie, andere logiesfunctie, bijeenkomst (overig en kinderopvang), cel, gezondheidszorg (klinisch en niet-klinisch), kantoor, logies in een logiesgebouw, onderwijs, sport en winkel. Er zijn twee deelgebieden: 1 energieprestatie, 2 energiegebruik. Het derde deelgebied, financiële kengetallen, is uitgesteld. | voldaan (functies) | `UsageFunction` in `crates/nta8800-core/src/monthly_demand.rs` kent alle functies. Andere logiesfunctie rekent als woningbouw volgens tabel 6.1 (openbaar geval D, [vergelijking](nta8800-vergelijking-openbare-rapporten.md)). | Met de attesteringsinstelling vastleggen voor welke deelgebieden en eventuele uitsluitingen (§2.4, §8.3) het attest wordt aangevraagd. |
| §3.0, p. 5 | De aanvrager voert alle deeltesten van de gekozen deelgebieden zelf uit. Hij stelt het programma, de documentatie en de resultaten ter beschikking. | extern | Er is een administratief referentieharnas: `crates/nta8800-core/src/reference.rs`, `POST /v1/nta8800/reference/audit` en de MCP-tool `audit_reference_case` ([referentieprotocol](nta8800-referentieprotocol.md)). Daarnaast zijn er openbare gevallen A–F (`crates/nta8800-core/tests/public_comparison.rs`). | ISSO 54 versie 5.0:2026 met de verwachte uitkomsten, en een batchrun die alle deeltesten draait en de resultaten bewaart (taak B1). |
| §3.0, p. 5 | Bij het programma hoort documentatie: een handleiding die ook de licentiehouders krijgen, en een globale beschrijving van het programma. | voldaan | [Handleiding](handleiding-nta8800/index.md) (hoofdstukken 00–09) en de [globale beschrijving van het rekenprogramma](nta8800-programmabeschrijving.md) (taak B6, 8 oktober 2026). | – |
| §3.3, p. 5–6 | De instelling controleert per deelgebied minstens 25 % van de uitkomsten, en van elke test minimaal één. Ze voert zelf in. Van alle (deel)testen moeten invoer en uitkomst beschikbaar zijn. | deels / extern | Alle normatieve invoer heeft een formulier, zodat de instelling zelf kan invoeren ([verificatiestatus](nta8800-verificatiestatus.md)). Het projectdossier bundelt invoer, kernuitvoer en rapport met SHA-256 (`src/core/report/ProjectDossier.ts`). | Projectbestanden en een uitkomstenoverzicht per deeltest van ISSO 54. Dat kan pas met de testset. |
| §3.6, p. 6 | Het attest volgt pas als hoofdstukken 4–6 voldaan zijn en er een attesteringsovereenkomst is. | open | – | Een attesteringsinstelling kiezen en een overeenkomst sluiten. |

### Berekeningsmethode, documenten en uitvoer (hoofdstuk 4)

| Eis (BRL 9501) | Inhoud in eigen woorden | Status | Bewijs in de repo | Wat ontbreekt |
|---|---|---|---|---|
| §4.1.1, p. 7 | De energieprestatie wordt volledig volgens NTA 8800 berekend, nauwkeurig genoeg volgens ISSO 54. | deels | De volledige keten van hoofdstuk 5 tot en met 17 en de bijlagen zit in de kern ([reikwijdte](handleiding-nta8800/01-reikwijdte-en-status.md)). Elke route is onafhankelijk herberekend ([verificatiestatus](nta8800-verificatiestatus.md)). De optiedekkingstest draait elke invoeroptie in vijf uitgaven (`tests/option_coverage.rs`, 7 650 runs). Zes openbare rapporten zijn nagerekend ([vergelijking](nta8800-vergelijking-openbare-rapporten.md)). Status `calculated_unverified`. | Toetsing tegen ISSO 54 versie 5.0. Twee routes blijven open: het uurklimaat van 17.3.8 en ISO 6946 tabel 8. Twee interpretaties wijken af van Uniec: 10.15 en 10.87 bij koeling ([vragen aan NEN](nta8800-vragen-nen.md)). |
| §4.1.1, p. 7 | De uitvoer van de indicatoren voldoet aan ISSO 54 en §4.3. | deels / extern | Het rekenrapport heeft een samenvatting, standaard- en detailniveau tot op rekenniveau (`src/core/report/EnergyPerformanceReport.ts`). Het bevat de labelgegevens (`label_data.rs`) en de bijlage "Interpretaties" (`kernel_interpretations()`). | De uitvoereisen van ISSO 54 versie 5.0 zijn niet ingezien en kunnen dus niet worden afgevinkt. |
| §4.1.2, p. 7 | Deelgebied 2: de energiestromen van besparingsmaatregelen worden nauwkeurig genoeg berekend, inclusief het renovatiepaspoort. | deels | `crates/nta8800-core/src/maatwerkadvies.rs`, de afstemming op het werkelijke gebruik (ISSO 82.2/75.2) en `renovation_passport` met schema W en U ([attestdossier](nta8800-attestdossier.md), MWA-tabel). | De ISSO 54-tests voor deelgebied 2 en hun uitkomsten. Lokale klimaatdata (NEN 5060-uurwaarden) ontbreken voor de afstemming. |
| §4.1.3, p. 7 | Deelgebied 3: financiële kengetallen, met de gebruikte invoer bij de uitkomsten. | n.v.t. | Het deelgebied is uitgesteld (§2.1). Er is al een NCW en terugverdientijd per pakket, met een verplichte `costSource` per maatregel. | Het rekenmodel van ISSO-rapport 110293 is niet beschikbaar. Pas nodig als het deelgebied wordt ingevoerd. |
| §4.2, p. 7 | Elke deeltest van ISSO 54 ligt binnen de bandbreedte die voor die deeltest geldt. | extern | Openbare gevallen A–F en de RVO-voorbeeldwoningen (zie §2 hieronder). Het harnas vergelijkt met een absolute én een relatieve tolerantie per grootheid; de ruimste geldt (`reference.rs`, `relativeTolerance`, taak B2). De suites `training-data/reference-suites/` lopen in elke gate mee (taak B1). | De testset zelf. |
| §4.3, p. 7 | De documentatie beschrijft hoe versienummers verschillen naar omvang en aard van een wijziging, het versienummer, de uitwisselbaarheid tussen versies en de opbouw van het nummer. | voldaan | [Versiebeheer](nta8800-versiebeheer.md) beschrijft alle vier. Het kernelstempel per projectbestand en `normalizeProject` borgen de uitwisselbaarheid. Sinds 8 oktober 2026 dwingt de gate het beleid af: `scripts/nta-kernel-version.mjs check` faalt als onuitgebrachte uitkomstwijzigingen in de releasenotes staan zonder MINOR-ophoging van `KERNEL_VERSION` (nu 0.2.0, taak B3). | – |
| §4.3.1, p. 8 | De berekening en de uitkomst worden geleverd in de vastgelegde XML-standaard voor upload naar de RVO-database. | extern | Er is een `ep-online-gegevensoverzicht.json` volgens het openbare *export*schema. Dat is uitdrukkelijk geen registratiebestand ([attestdossier](nta8800-attestdossier.md), [bronnenregister](nta8800-bronnenregister.md)). | Het XSD voor aanlevering of registratie, op te vragen bij RVO, en daarna de adapter. |
| §4.3.1, p. 8 | Voldoen aan de eisen van RVO voor inloggen met eHerkenning. | extern | – | De eisen van RVO, en eHerkenning voor de attesthouder. |
| §4.3.1 opm., p. 8 | Bij invoer voor registratie wordt vermeld of, en met welk hulpmiddel, de adviseur gegevens heeft ingelezen. Automatisch inlezen mag niet. | voldaan | `src/core/io/importLog.ts`: elke UNIEC3- of VABI-import schrijft een regel. De registratiecontrole geeft `dataImport`, en het rapport toont dat. Een import start altijd de adviseur zelf. | – |
| §4.4, p. 8 | De attesthouder geeft de licentiehouders schriftelijk een handleiding. Zonder licenties hoeft dat niet, maar een beknopte beschrijving voor de instelling wordt aangeraden. | voldaan | De [handleiding](handleiding-nta8800/index.md) zit in het programma (Gereedschap › Handleiding, ook via Instellingen › Over en een knop *Handleiding* op elke werkstap) en gaat bij elke vrijgave als één HTML-bestand mee, met SHA-256 in het leveringsdocument en `SHA256SUMS` (taak B7). De stempel in `index.md` noemt de rekenkernversie; de gate controleert die tegen `KERNEL_VERSION`. | Een PDF komt alleen mee als op de buildmachine chromium of wkhtmltopdf staat; anders alleen HTML. |

### Eisen aan de attesthouder (hoofdstuk 5)

| Eis (BRL 9501) | Inhoud in eigen woorden | Status | Bewijs in de repo | Wat ontbreekt |
|---|---|---|---|---|
| §5.1, p. 9 | De attesthouder staat in het handelsregister van een EU-lidstaat, met een uittreksel van maximaal een jaar oud. | open | – | Vaststellen welke rechtspersoon attesthouder wordt, en een uittreksel opvragen. |
| §5.2, p. 9 | Wijzigingen die het resultaat kunnen beïnvloeden worden direct schriftelijk gemeld aan de instelling en de licentiehouders. | deels | De [releasenotes](nta8800-releasenotes.md) leggen elke uitkomstwijziging vast, gegroepeerd per rekenkernversie (taak B3). | Een meldprocedure naar de instelling en de licentiehouders (organisatorisch). |
| §5.2, p. 9 | Het rekenkerndeel van het versienummer wordt bij RVO geregistreerd. | open | `KERNEL_VERSION` en `TARGET_NORM_VERSION` bestaan ([versiebeheer](nta8800-versiebeheer.md)). | De registratie bij RVO, na de eerste echte release van de kern. |
| §5.3, p. 9 | Na een normwijziging blijft de oude versie minstens 3 jaar bruikbaar: het oude programma zelf, een versie om mee te rekenen, of de attesthouder rekent op verzoek. | deels | De broncode van elke versie staat in git. De kern rekent bovendien oudere uitgaven (2024, 2023, 2022, 2020+A1) als `calculated_legacy_edition` ([normversies](nta8800-normversies.md)). | `scripts/release-nta.sh` maakt per vrijgave een git-tag en een archief met het installatiepakket en SHA-256 (taak B4). Nog vast te leggen: waar de archieven minstens 3 jaar worden bewaard (organisatorisch). Let op: de uitgavenprofielen in de huidige kern zijn niet hetzelfde als de geattesteerde oude kern, en vervangen die dus niet. |
| §5.3 opm. 2, p. 9 | Zowel de invoer als de uitkomst van de berekening gaat naar EP-Online. | extern | Het projectdossier bevat beide. | Het registratieformaat (zie §4.3.1). |
| §5.4, p. 9 | Elk jaar registreert minstens één certificaathouder een berekening volgens BRL 9500-W en een volgens 9500-U. | open | – | Kan pas na het attest en de EP-Online-koppeling. Daarvoor zijn afspraken met een certificaathouder nodig. |
| §5.5, p. 9 | De attesthouder neemt deel aan TC 9501 van InstallQ. | open | – | Aanmelden bij InstallQ. |

### Interne kwaliteitsbewaking (hoofdstuk 6)

| Eis (BRL 9501) | Inhoud in eigen woorden | Status | Bewijs in de repo | Wat ontbreekt |
|---|---|---|---|---|
| §6.1, p. 10 | Het programma wordt geleverd met een leveringsdocument dat het versienummer uit het attest draagt. | deels | `scripts/release-nta.sh` vult bij elke vrijgave het sjabloon `docs/templates/nta8800-leveringsdocument.md` uit `src/core/nta/attest.json`, de versienummers en de SHA-256 van de pakketten (taak B5). | Het attestnummer en de identificatiecode in `attest.json`, na het attest. |
| §6.2, p. 10 | Er is een logboek van elke wijziging in de software zoals de licentiehouder die gebruikt. | voldaan | De git-geschiedenis, de [releasenotes](nta8800-releasenotes.md) per rekenkernversie (taak B3) en per vrijgave een git-tag met archief (taak B4). | – |
| §6.2, p. 10 | De testresultaten worden bij elke wijziging van de berekeningsmethode of de gebruikersinterface bewaard. | deels | De gate `scripts/verify-nta.sh` draait de kern-, service-, Tauri- en frontendtests, de optiedekking, de openbare gevallen, de MSRV-controle en de build. Voor een aantal commits staat het resultaat vast in `docs/nta8800-build-verificatie-*.md`. De gate schrijft het rapport van de referentiesuites met commit, `KERNEL_VERSION` en SHA-256 per invoer en uitvoer; `scripts/release-nta.sh` bewaart het gatelog en dat rapport per vrijgave (taken B1 en B4). | De ISSO 54-tests ontbreken in de gate. |
| §6.2.1, p. 10 | Er is een register van licentiehouders. | open | – | Een register opzetten, buiten de software. |
| §6.3, p. 10 | Na elke release toont de attesthouder aan dat hoofdstuk 4 nog voldaan is, en bewaart hij dat onderzoek. | deels | Dezelfde gate als hierboven. | De ISSO 54-run per release. Zonder de testset is §4.2 niet aan te tonen. |
| §6.4, p. 10 | Na elke wijziging van de berekeningsmethode of de gebruikersinterface volgt een releasenote, die naar de instelling en de licentiehouders gaat. | deels | [Releasenotes](nta8800-releasenotes.md). | Een releasenote per release-versie, ook voor wijzigingen die alleen de gebruikersinterface raken. De releasenotes moeten ook naar de instelling en de licentiehouders worden verspreid. |
| §6.5, p. 10 | Er is een gedocumenteerde klachtenprocedure: afhandeling, verantwoordelijken, registratie (datum, aard, oplossing, oorzaak), terugkoppeling aan de klager en intern, en archivering. | open | Alleen genoemd in het [totaalplan](nta8800-totaalplan.md), stap 7. | Een procedure en een klachtenregister (taak C3). |

### Attest en attestmerk (hoofdstukken 7–9)

| Eis (BRL 9501) | Inhoud in eigen woorden | Status | Bewijs in de repo | Wat ontbreekt |
|---|---|---|---|---|
| §7.1.3, p. 11 | Elke twee jaar steekproeven op minstens 10 % van de ISSO 54-tests per deelgebied. | extern | – | De testset en een herhaalbare run (taak B1). |
| §7.2, p. 12–13 | Bij een rekenkernwijziging voert de attesthouder alle nieuwe en gewijzigde (deel)testen zelf uit. | extern | – | Hetzelfde. Daarnaast moet bekend zijn welke tests ISSO 54 als gewijzigd aanmerkt. |
| §8.2, p. 15 | Een nieuwe rekenkern krijgt een nieuw attest. Het versienummer van de rekenkern staat op het attest. | deels | `KERNEL_VERSION` wordt per berekening vastgelegd en weigert herlabelen met een andere kern (`relabel_kernel_version_differs`). | De versiediscipline staat (taak B3). Het attest moet straks de dan geldende `KERNEL_VERSION` noemen (organisatorisch). |
| §8.4 opm., p. 15 | Een geattesteerd programma draagt het NL-EPBD-merk. | deels | Taak B8 (8 oktober 2026). Eén accessor, `src/core/nta/Attest.ts`, leest het attestnummer. Alleen met een attestnummer tonen het venster "Over" en het rapport (voorblad en kopregel) het nummer en de plaats van het merk. Zonder attestnummer staat er "niet geattesteerd" en geen merk. Getest in `src/__tests__/brl9501-attest-label-statements.test.tsx`. | Het officiële NL-EPBD-beeldmerk, na het attest en de licentie op het merk. Het hoort in `src/assets/nl-epbd/` en vervangt de tijdelijke tekst in het merkvak. |
| §9, p. 16 | De licentiehouder krijgt bij de opdracht een volledig exemplaar van het attest. Gepubliceerd wordt alleen volledig. | open | – | Organisatorisch, na het attest. |

## 2. Testset ISSO 54

**Wat de BRL vraagt.** BRL 9501 verwijst voor volledigheid, nauwkeurigheid, uitvoer en de bandbreedte per deeltest naar ISSO-publicatie 54 versie 5.0:2026 (§4.1, §4.2, §11 ref. [1]). Die publicatie deelt de tests per deelgebied in hoofdgroepen, tests en deeltesten in, en geeft per (deel)test de maximaal toegestane bandbreedte (§3.3 opm., p. 6). De instelling controleert:
- bij toelating minstens 25 % van alle uitkomsten (§3.3);
- bij een herattestering 25 % van de nieuwe en gewijzigde tests, en 10 % van de ongewijzigde per hoofdgroep (§7.2.3);
- in het vervolgonderzoek elke twee jaar 10 % (§7.1.3).

**Wat bekend is.** Versie 5.0:2026 is niet openbaar en is hier niet ingezien.
- Openbaar is alleen versie 2.0 van 12-05-2022, voor NTA 8800:2022. Die bevat de testbeschrijvingen (testgebouw EPW001, deeltesten per onderwerp, realistische gebouwen), maar niet de verwachte uitkomsten. Die staan in een apart Excel-bestand.
- De afkeurgrens in versie 2.0 is 1 % per deeltest. Zie het [attestdossier](nta8800-attestdossier.md) en het [referentieprotocol](nta8800-referentieprotocol.md).
- De testgeometrie van EPW001 staat als historische invoerregressie in `training-data/edr-2022-epw001-*.json`, zonder verwachte uitkomsten.

**Wat al kan, als voorbereiding en niet als bewijs.**
- **Openbare rapporten A–F** (Uniec, uitgaven 2020+A1 tot 2024), in `training-data/nta8800-public-comparison-{a..f}.json` en `crates/nta8800-core/tests/public_comparison.rs`. Na verklaring per post resten vooral twee oorzaken: de lezing van 10.15 en 10.87 bij koeling, en aannames waar een uitdraai gegevens mist. Twee voorbeelden:
  - D (2020+A1): 86,35 / 39,07 / 83,6 tegen 86,72 / 39,19 / 83,5;
  - F (2020+A1): 74,06 / 3,79 / 96,3 tegen 74,69 / 2,59 / 97,5.
  
  Een bandbreedte van 1 % halen deze gevallen niet op BENG 2 waar koeling meetelt. Dat is het grootste inhoudelijke risico voor de testset ([vergelijking](nta8800-vergelijking-openbare-rapporten.md), [vragen aan NEN](nta8800-vragen-nen.md)).
- **RVO-voorbeeldwoningen.** Zes woningtypen; de warmtebehoefte ligt binnen ±6 % ([vergelijking](nta8800-vergelijking-rvo-voorbeeldwoningen.md)).
- **Interne controles.**
  - onafhankelijke herberekening van elke route;
  - de optiedekkingstest in vijf uitgaven;
  - fuzz- en robuustheidstests;
  - het referentieharnas `audit_reference_case`, met per case de normversie, de bron, het gebruiksrecht, de onafhankelijke reviewer en de verwachte waarden.

**Wat nodig is zodra de testset er is.**
1. ISSO 54 versie 5.0:2026 en het uitkomstenbestand rechtmatig verkrijgen, met de gebruiksrechten.
2. Elke (deel)test als projectbestand vastleggen (invoer) en als manifest voor `audit_reference_case` (verwachte uitkomst en bandbreedte, absoluut en relatief). Een suite in `training-data/reference-suites/` kan per case naar een projectbestand verwijzen.
3. De suite toevoegen aan `training-data/reference-suites/`. De gate draait dan elke deeltest en schrijft het verschil, het oordeel, de commit, `KERNEL_VERSION` en de SHA-256 van invoer en uitvoer weg (`reference_gate`, taak B1). Het bewijs dient voor §3.0, §6.2, §6.3 en §7.2.
4. Afwijkingen per deeltest verklaren. Een tegenstrijdigheid of omissie in de testset melden aan InstallQ en ISSO; de instelling doet dat ook (§7.7, p. 14).
5. De koelinterpretatie (10.15 en 10.87) vastzetten op wat de testset verwacht, zodra die bekend is of NEN antwoordt.

## 3. Wat BRL 9500 en MWA van de software vragen

Het [attestdossier](nta8800-attestdossier.md) loopt BRL 9500-W/U, BRL 9500-MWA-W/U, het Bbl en de Omgevingsregeling per eis na. Daar staan de bronnen en de code. Samengevat staat het overgrote deel als "aanwezig", met deze uitzonderingen:

| Eis | Status | Toelichting |
|---|---|---|
| EP-Online-registratie (BRL 9500 §4.2.5; Omgevingsregeling art. 5.11/5.12 lid 3 en 5.14) | extern | Het uploadformaat is niet openbaar. |
| Verplichte labelelementen k en l (Omgevingsregeling art. 5.13a lid 1) | aanwezig | Taak B9 (8 oktober 2026). Dit zijn verklaringen van de adviseur, met ja of nee, in `registration.labelStatements`. Voor een registratie vanaf 29 mei 2026 vraagt de kern ze (`label_statement_external_signals_required`, `label_statement_low_temperature_required`). Het rapport toont ze bij de labelgegevens. Het EP-Online-gegevensoverzicht zet ze apart, omdat het exportschema er geen velden voor heeft. Element m vult de uitgever van het label in. |
| Rekenregels voor de kostenberekening (ISSO-modelbeschrijving, rapport 110293) | extern | Het rapport is niet beschikbaar. De NCW gebruikt de parameters van de adviseur. |
| Bewaren van de oorspronkelijke rekenkern voor herlabelen (24 maanden) en van het dossier (15 jaar) | organisatorisch | Hangt samen met taak B4. Het bewaren van dossiers ligt bij de certificaathouder. |
| Maatwerkadvies, afstemming op werkelijk gebruik, renovatiepaspoort | deels | Aanwezig maar niet geverifieerd. Lokale klimaatdata ontbreken. |

## 4. Wat het attest nog tegenhoudt

### (a) Zelf te bouwen

Afgerond op 8 oktober 2026: B1–B5 (gebouwd, zie de rijen) en B6, B8 en B9 (✔).

| Taak | Eis | Omschrijving |
|---|---|---|
| **B1** | §3.0, §6.2, §6.3, §7.2 | Een testsetrunner die alle referentiecases in een map draait, met `audit_reference_case` per case. Hij schrijft een rapport (Markdown en JSON) met de commit, `KERNEL_VERSION`, `TARGET_NORM_VERSION`, het oordeel per deeltest en de SHA-256 van invoer en uitvoer. De gate roept hem aan, zodat de openbare gevallen en later ISSO 54 elke run meelopen. **Gebouwd (8 oktober 2026):** `reference_gate --suite … --report-dir …` met `reference-report.json` en `.md`; suites voor de openbare gevallen A–F en de RVO-voorbeeldwoningen; draait in elke gate. |
| **B2** | §4.2 | Bandbreedtes in het referentiemanifest: naast `absoluteTolerance` ook `relativeTolerance` (bijvoorbeeld 1 %), en per metriek de regel "absoluut of relatief, welke het ruimst is". ISSO 54 versie 2.0 werkt met een percentage per deeltest. |
| **B7** ✔ | §4.4 | De handleiding bij elke release meeleveren, als PDF of in de app, met dezelfde versie als het programma. **Gebouwd (9 oktober 2026):** in de app via `src/core/manual/` en `ManualView`; bij de vrijgave `scripts/nta-manual.mjs html`; gatecontrole `scripts/nta-manual.mjs check`. |
| **B8** ✔ | §8.4 | Het NL-EPBD-merk en het attestnummer tonen in de app en op het rapport, zodra `SOFTWARE_ATTEST_NUMBER` gevuld is. |
| **B9** ✔ | Omgevingsregeling art. 5.13a | Invoervelden voor de verklaringen van de adviseur bij labelelementen k en l. |

### (b) Extern

| Punt | Bij wie | Waarom het blokkeert |
|---|---|---|
| **E1** ISSO 54 versie 5.0:2026 met verwachte uitkomsten en bandbreedtes | ISSO | Zonder de testset is §4.2 niet aan te tonen. Er is geen attest mogelijk. |
| **E2** XSD voor aanlevering en registratie bij EP-Online, en de eHerkenningseisen | RVO, EP-Online-team | §4.3.1. Ook nodig voor §5.4 en voor de registratie van labels door certificaathouders. |
| **E3** Antwoord over 10.15, 10.87 en f_prac | NEN | De koelroute wijkt af van Uniec en raakt waarschijnlijk de 1 %-grens bij BENG 2 ([vragen aan NEN](nta8800-vragen-nen.md)). |
| **E4** Een attesteringsinstelling | aangewezen instelling met InstallQ-overeenkomst | §3.6: toelatingsonderzoek, overeenkomst en afgifte. |
| **E5** Uurklimaat 17.3.8 en ISO 6946 tabel 8 | NEN / normtekst | Twee routes zijn nog niet volledig; zie de [verificatiestatus](nta8800-verificatiestatus.md). |
| **E6** ISSO-rapport 110293 (kostenmodel) | ISSO | Alleen voor deelgebied 3 en de NCW-regels van het maatwerkadvies. |

### (c) Organisatorisch

| Punt | Eis |
|---|---|
| **C1** De rechtspersoon van de attesthouder vaststellen, met een KvK-uittreksel van hooguit een jaar oud | §5.1, §8.1 |
| **C2** Een kwaliteitshandboek met wijzigings- en releaseprocedure, meldprocedure richting instelling en licentiehouders, en een interne reviewer | §5.2, §6.2–6.4 |
| **C3** Een klachtenprocedure en een klachtenregister | §6.5 |
| **C4** Een register van licentiehouders | §6.2.1 |
| **C5** De rekenkernversie registreren bij RVO | §5.2 |
| **C6** Bewaartermijnen vastleggen: 3 jaar na een normwijziging, 24 maanden voor herlabelen | §5.3; BRL 9500 §4.2.4 |
| **C7** Deelnemen aan TC 9501 | §5.5 |
| **C8** Afspraken met minstens één certificaathouder over een jaarlijkse W- en U-registratie | §5.4 |

### De tien belangrijkste blokkades, op volgorde

1. **E1:** ISSO 54 versie 5.0:2026 met de uitkomsten (extern).
2. **E3:** de koelinterpretatie 10.15 en 10.87; risico op de 1 %-grens (extern).
3. **E2:** het EP-Online-registratieformaat en eHerkenning (extern).
4. ~~**B1 + B2:** een testsetrunner met relatieve bandbreedtes~~ (gebouwd 8 oktober 2026).
5. ~~**B3:** de discipline voor de rekenkernversie, met releasenotes per versie~~ (gebouwd 8 oktober 2026).
6. **E4:** een attesteringsinstelling en de overeenkomst (extern).
7. **C2:** het kwaliteitshandboek met de wijzigings-, release- en meldprocedures (organisatorisch).
8. ~~**B4 + B5:** een releasearchief en het leveringsdocument~~ (gebouwd 8 oktober 2026; de bewaarplaats van de archieven is organisatorisch).
9. **C1 + C3 + C4:** KvK, klachtenprocedure, register van licentiehouders (organisatorisch).
10. **E5:** het uurklimaat 17.3.8 en ISO 6946 tabel 8 (extern).

## Telling

Telling over de eisentabel in §1 (34 rijen). Een rij met twee statussen telt bij de eerste.

| Status | Aantal |
|---|---|
| voldaan | 6 |
| deels | 12 |
| open | 8 |
| extern | 7 |
| n.v.t. | 1 |

Bijgewerkt na taken B1–B5 en B6, B8, B9 (8 oktober 2026): §3.0, §4.3 en §6.2 (logboek) gingen naar voldaan, §6.1 en §8.4 van open naar deels. Na taak B7 (9 oktober 2026) ging §4.4 van deels naar voldaan.
