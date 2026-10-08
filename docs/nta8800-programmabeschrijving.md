# Globale beschrijving van het NTA 8800-rekenprogramma

**Stand:** 8 oktober 2026, op `nta8800-kernel`.
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

- **Normversies.** De rekenkern rekent standaard volgens NTA 8800:2025+C1:2026, de aangewezen versie. Voor vergelijking en herberekening rekent ze ook volgens 2024, 2023, 2022 en 2020+A1. De verschillen staan als genummerde schakelpunten, met paginaverwijzingen, in [normversies](nta8800-normversies.md). Een oudere versie geeft status `calculated_legacy_edition` en is niet registreerbaar.
- **Interpretaties.** Waar de norm meer dan één lezing toelaat, legt de kern haar keuze vast in `kernel_interpretations()`. Die keuzes staan in elk gedetailleerd rapport.
- **Open vragen aan NEN.** De open vragen, met name 10.15, 10.87 en f_prac, staan in [vragen aan NEN](nta8800-vragen-nen.md).
- **Vergelijking met andere programma's.** Hoe de uitkomsten zich verhouden tot die van andere programma's staat in de [vergelijking met openbare rapporten](nta8800-vergelijking-openbare-rapporten.md) en de [vergelijking met de RVO-voorbeeldwoningen](nta8800-vergelijking-rvo-voorbeeldwoningen.md).

## 5. Verificatie

De gate `scripts/verify-nta.sh` loopt bij elke wijziging. Er wordt alleen vastgelegd als hij slaagt. Hij draait:

- de unittests van de kern, met paginaverwijzingen naar de norm;
- de differentiële tests tussen de normversies;
- de **optiedekkingstest** (`crates/nta8800-core/tests/option_coverage.rs`). Die leidt elke invoeroptie af uit de invoertypen zelf en rekent haar door in alle vijf normversies. Elke run moet rekenen of weigeren met een benoemde code met tekst, zonder paniek en zonder niet-eindige getallen.
- de **openbare gevallen** A–F (`tests/public_comparison.rs`), nagerekend in hun eigen normversie;
- de robuustheidstests;
- de service- en API-tests;
- de tests van de desktop-opdrachten;
- de frontendtests, waaronder de controle dat elke meldcode van de kern een Nederlandse en Engelse tekst heeft.

Daarnaast zijn er:
- **MSRV-test.** De kern wordt ook getest met Rust 1.77.2, de oudste ondersteunde versie.
- **Onafhankelijke herberekening.** Elke route is apart herberekend; de stand staat in de [verificatiestatus](nta8800-verificatiestatus.md).
- **Referentieharnas.** Het [referentieprotocol](nta8800-referentieprotocol.md) beschrijft het harnas waarmee de ISSO 54-testset wordt gedraaid zodra die er is.
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

- **Geen attest.** Het programma is niet geattesteerd. Alle uitkomsten hebben status `calculated_unverified`.
- **Geen testset.** De ISSO 54-testset versie 5.0:2026 met verwachte uitkomsten is nog niet ingezien. Overeenstemming binnen de bandbreedtes is dus nog niet aangetoond (§4.2, p. 7).
- **Geen registratiebestand.** Registratie in EP-Online is niet mogelijk zonder het uploadformaat van RVO en een attest.
- **Open interpretatie koeling.** De koelroute volgt 10.15 en 10.87 letterlijk. Dat wijkt af van ten minste één ander programma, zie de vragen aan NEN.
- **Onvolledige routes.** Het uurklimaat van 17.3.8 en tabel 8 van ISO 6946 zijn niet volledig ([verificatiestatus](nta8800-verificatiestatus.md)).
- **Wat er nog openstaat voor het attest.** De volledige lijst staat in de [BRL 9501-gereedheid](nta8800-brl9501-gereedheid.md).
