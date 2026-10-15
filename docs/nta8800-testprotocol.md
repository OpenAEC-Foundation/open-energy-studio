# Testprotocol NTA 8800-rekenprogramma

*Stand: 9 oktober 2026. Programma 0.2.0, rekenkern 0.4.0 (onuitgebracht; laatste uitgave 0.3.0).*

Dit protocol beschrijft hoe Open Energy Studio wordt getest: bij elke wijziging en bij elke vrijgave. Het is bedoeld voor de organisatie (ontwikkelaar, interne reviewer, vrijgever) en voor de attesteringsinstelling. Het sluit aan op BRL 9501 §3.0 (p. 5), §4.2 (p. 7), §6.2–6.3 (p. 10) en §7.1.3–7.2 (p. 11–13); de stand per eis staat in de [BRL 9501-gereedheid](nta8800-brl9501-gereedheid.md). Normtekst van NEN, ISSO of de BRL staat niet in dit document; verwijzingen gaan naar artikel en pagina.

Verwante documenten: [kwaliteitshandboek](kwaliteit/kwaliteitshandboek.md), [referentieprotocol](nta8800-referentieprotocol.md), [voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md), [versiebeheer](nta8800-versiebeheer.md), [releasenotes](nta8800-releasenotes.md), [archiefbeleid](kwaliteit/archiefbeleid.md), [programmabeschrijving](nta8800-programmabeschrijving.md).

## 1. Doel, reikwijdte en versies

**Doel.** Aantonen dat de rekenkern de NTA 8800 correct toepast, dat de invoer, uitvoer en rapportage die uitkomst ongewijzigd doorgeven, en dat een wijziging geen onbedoelde uitkomstwijziging veroorzaakt.

**Reikwijdte.**

| Onderdeel | Plaats |
|---|---|
| Rekenkern | `crates/nta8800-core` (Rust) |
| Service, HTTP-API, MCP-server, referentierunner | `crates/nta8800-service` |
| Desktop-opdrachten en bestandsopslag | `src-tauri` |
| Gebruikersinterface, rapporten, dossier, EP-Online-overzicht, import en export | `src/` |

**Versies.** Elk testrapport noemt:

- de programmaversie (`package.json`, nu 0.2.0);
- `KERNEL_VERSION` (`crates/nta8800-core/Cargo.toml`, nu 0.3.0);
- de commit.

**Normversies.** De kern rekent in vijf uitgaven:

- NTA 8800:2025+C1 is de enige registreerbare;
- 2024, 2023, 2022 en 2020+A1 zijn oudere uitgaven, met status `calculated_legacy_edition` ([normversies](nta8800-normversies.md)).

Een test die per uitgave loopt, loopt in alle vijf.

**Deelgebieden** (BRL 9501 §4.1): woningbouw en utiliteitsbouw, nieuwbouw en bestaande bouw, inclusief basisopname (ISSO 82.1/75.1), maatwerkadvies (ISSO 82.2/75.2), herlabelen (BRL 9500 bijlage 6a/6b) en registratie.

## 2. Testniveaus

Alle automatische niveaus zitten in de gate `scripts/verify-nta.sh`. De gate stopt bij de eerste fout. Er wordt alleen vastgelegd en vrijgegeven als de gate slaagt (afsluitcode 0).

**De volledige gate:**

```sh
CARGO_NET_OFFLINE=true scripts/verify-nta.sh            # met MSRV-run
CARGO_NET_OFFLINE=true NTA_SKIP_MSRV=1 scripts/verify-nta.sh   # zonder, voor tussentijdse runs
```

- **Duur:** ongeveer 30 minuten op een machine met 8 kernen (zie §5, doorlooptijd van de gate).
- **Aantallen:** de aantallen tests hieronder zijn afgerond en verouderen; de geldende aantallen per vrijgave staan in `gate-samenvatting.json` en in het testrapport, uit het gatelog van die vrijgave.
- **Bewijs:** de gate schrijft het referentierapport en de formulierdocumenten naar `$NTA_EVIDENCE_DIR`, standaard `nta-evidence/gate/` (niet in git). `scripts/release-nta.sh` zet dit per vrijgave in `release/<tag>/evidence/` en het gatelog in `release/<tag>/gate.log`.

| | Niveau | Wat het dekt | Uitvoeren | Geslaagd als | Bewijs | Duur |
|---|---|---|---|---|---|---|
| a | Unittests van de kern | Elke formule, tabel en invoercontrole, met paginaverwijzing naar de norm in de testnaam of het commentaar | `cargo test --locked --manifest-path crates/nta8800-core/Cargo.toml --lib` | alle tests groen (bijna 1 000) | gatelog | ~1 min |
| b | Differentiële tests per uitgave | Per schakelpunt verandert alleen de verwachte grootheid tussen opeenvolgende uitgaven; elk bronlabel noemt de berekende uitgave (`tests/edition_labels.rs`); een wissel van uitgave noemt elke weigering en eis met zijn schakelpunt (`tests/edition_switch.rs`). De verschillentabel per uitgave op de openbare gevallen staat in `tests/edition_deltas.rs` (`--ignored`). | `--test norm_versions`, `--test edition_labels`, `--test edition_switch` (zelfde manifest) | alle tests groen | gatelog | < 1 min |
| c | Openbare gevallen A–O | Vijftien gepubliceerde BENG-berekeningen nagebouwd en in hun eigen uitgave doorgerekend; de banden en de verschilanalyse per post staan in [vergelijking openbare rapporten](nta8800-vergelijking-openbare-rapporten.md) | `--test public_comparison` | elke waarde binnen de vastgelegde band | gatelog | < 1 min |
| d | Referentiesuites | De suites in `training-data/reference-suites/`: `openbare-gevallen.json` (A–O), `rvo-voorbeeldwoningen.json` (±6 % op Q_H;nd) en `isso54-v2.json` (260 deeltesten zonder verwachting) | gatestap "reference suites"; los: `cargo run --manifest-path crates/nta8800-service/Cargo.toml --bin reference_gate -- --suite <suite> --report-dir <map>` | afsluitcode 0. Een suite die alleen `pending_expectation` bevat, geeft los gedraaid afsluitcode 3 (`no_comparison`), wat geen geslaagd oordeel is. In de gate vergelijken de openbare gevallen en RVO wel, en is de afsluitcode 0. | `reference-report.json` en `.md` met commit, `KERNEL_VERSION`, uitgave, oordeel en SHA-256 per invoer en uitvoer | ~1 min |
| e | Optiedekking | Elke enumvariant en elk optioneel veld van de invoer, afgeleid uit de invoertypen, in alle vijf uitgaven. Elke run rekent, of weigert met een benoemde code met NL/EN-tekst en een gat met een route. | `--test option_coverage` | geen paniek, geen niet-eindig getal, geen code zonder tekst, geen gat zonder route | gatelog | 1–2 min |
| f | Numerieke fuzz | Elk getal uit de voorbeelden en openbare gevallen krijgt randwaarden (0, negatief, klein, groot, net buiten bereik), plus een geseede combinatieronde en monotoniecontroles | `--test numeric_fuzz` | idem als (e), plus plausibiliteit en monotonie | gatelog | ~1,5 min |
| g | Grote projecten en prestaties | Tot 200 zones en 1 800 ramen, een dicht utiliteitsgebouw (50 zones, 2 250 ramen met zonwering, 1 200 koudebruggen, 30 PV-systemen), 200 lege velden daarin (en dezelfde meldingen als de zoektocht op het hele blok) en een opname met 400 ramen; lineaire schaal en een tijdsbudget | `--test large_projects`, `--test blank_search_equivalence`; benchmark: `cargo test --release --offline --manifest-path crates/nta8800-core/Cargo.toml --test large_projects -- --ignored --nocapture` | binnen het budget; uitkomsten schalen zoals verwacht | gatelog, [programmabeschrijving §8](nta8800-programmabeschrijving.md) | < 1 min |
| h | Robuustheid | Afwijkende en kapotte invoer, achtergehouden resultaten, null gelijk aan afwezig (`tests/robustness.rs`, `tests/null_as_absent.rs`), opname in alle uitgaven (`tests/survey_editions.rs`), overnemen van vloeren op de grond uit de opname (`tests/survey_takeover.rs`), opname- en formuliervormen (`tests/survey_form_shapes.rs`, `tests/project_form_shapes.rs`), bewijskoppelingen (`tests/evidence_link_registration.rs`) | `cargo test --locked --manifest-path crates/nta8800-core/Cargo.toml` | alle tests groen | gatelog | ~1 min |
| i | Service, API en MCP | Operaties, foutmodel, gelijktijdigheid en wachtrij, time-outs, `Retry-After`, Host-controle, onbekende velden (`tests/api.rs`); MCP over stdio inclusief kapotte JSON-RPC (`tests/mcp_stdio.rs`); geheugengrens per verzoek (`tests/memory_bound.rs`) | `cargo test --locked --manifest-path crates/nta8800-service/Cargo.toml` | alle tests groen | gatelog | ~1 min |
| j | Desktop-opdrachten | Kernopdrachten met uitgave en paniekafvang, atomair opslaan met `.bak`, recente bestanden na herstart (`src-tauri/src/lib.rs`, `src-tauri/src/project_files.rs`) | `cargo test --locked --manifest-path src-tauri/Cargo.toml --lib` | alle tests groen | gatelog | < 1 min |
| k | Frontendtests (Vitest) | Ruim 1 600 tests in ruim 170 bestanden in `src/__tests__/`, met de veegtest op gewiste velden (`nta-clear-field-sweep.test.tsx`); zie de lijst hieronder | `npx vitest run --maxWorkers=4` (limiet per test 60 s uit `vitest.config.ts`) | alle tests groen | gatelog | 5–10 min |
| l | MSRV | De kern op Rust 1.77.2, de oudste ondersteunde versie | `cd crates/nta8800-core && CARGO_TARGET_DIR=target/msrv-core cargo +1.77.2 test --locked --offline` | alle tests groen | gatelog (stap "core on Rust 1.77.2") | ~3 min |
| m | Versie- en handleidingstempel | Een uitkomstwijziging heeft een MINOR-stap van `KERNEL_VERSION`, uitgebrachte secties van de releasenotes zijn ongewijzigd, en de handleiding is gestempeld voor de huidige kern | `node scripts/nta-kernel-version.mjs check` en `node scripts/nta-manual.mjs check` | beide melden overeenstemming | gatelog (eerste twee stappen) | seconden |
| n | App tegen de echte rekenkern | De hele app met elke `/api`-aanroep beantwoord door `crates/nta8800-service/examples/kernel_once.rs`, zonder server: openen en rekenen, opname overnemen, maatwerkadvies, uitgave wisselen, rapport en EP-Online-overzicht, een gat met "Ga naar"; wat het scherm toont moet gelijk zijn aan het antwoord van de kern (`real-kernel-e2e.test.tsx`) | gatestap "app against the real kernel (kernel_once)"; los: `OES_KERNEL_ONCE=<pad> npx vitest run src/__tests__/real-kernel-e2e.test.tsx` | alle tests groen | gatelog | ~1 min |
| o | Formulierstanden en gewiste velden door de kern | De toestanden van de formulierbewaking en de veegtest (elk wisbaar veld afzonderlijk gewist) gaan door de kern; elk gewist veld geeft precies één gat op dat veld (`tests/project_form_guard.rs`) | gatestap "project-form states through the kernel" | alle tests groen | gatelog, `evidence/form-guard/` | ~1 min |

**Ook in de gate:**

- rustfmt en clippy (`-D warnings`) op kern en service;
- de TypeScript-controle (`npx tsc --noEmit`);
- de frontendbundel (`npm run build`), vóór de Tauri-stappen; CI roept alleen `scripts/verify-nta.sh` aan en bouwt de frontend niet apart (`ci-parity.test.ts`);
- **bestand tegen belasting:** UI-tests wachten via gedeelde limieten in `src/__tests__/timeouts.ts` (findBy/waitFor 5 s, per test 60 s, werkstroomtests 180 s); tijdsbudgetten schalen met een basislijn uit dezelfde run (`large-project-ui.test.tsx`). Een test die alleen onder belasting faalt, is een fout in de test of de app en wordt niet opgelost door hem los opnieuw te draaien;
- de stap "project-form states through the kernel": de documenten die de formulierbewaking schreef, gaan door `tests/project_form_guard.rs`.
- **onafhankelijk berekende waarden** (onderdeel van a/h): `tests/ventilation_independent.rs` (12 tests, review 65), `tests/installations_independent.rs` (37 tests, reviews 66–70) en `tests/edition_switches_independent.rs` (12 tests, reviews 72–73) leggen varianten vast met waarden die buiten de kern vanaf de normpagina's zijn berekend; de herberekeningen zelf staan in de [verificatiestatus](nta8800-verificatiestatus.md).

**Belangrijkste frontendtests (k):**

- **Formulierbewaking:**
  - `nta-project-form-guard.test.tsx` doorloopt alle secties van de projectformulieren (~2 400 toestanden); die toestanden gaan door de kern.
  - `survey-null-paths.test.tsx` leegt elk getal in alle elf opnamesecties.
  - `nta-project-form-guard-fixes.test.tsx` test de afzonderlijke formulierherstellingen.
- **Formulierdekking:** `nta-form-coverage.test.ts`, met elke optie per route ([formulierdekking](ui-redesign/formulierdekking.md)), en `nta-form-coverage-forms.test.tsx`, `nta-form-coverage-project.test.tsx`, `survey-coverage-forms.test.tsx`.
- **Teksten:** `kernel-code-labels.test.ts` (elke code van kern en service heeft NL/EN), `i18n-coverage.test.ts` en `i18n-orphan-keys.test.ts`.
- **Toegankelijkheid en invoer:** `nta-form-accessibility.test.tsx`, `a11y-forms-dialogs.test.tsx` (namen, foutkoppeling, "Ga naar" en toetsenbord in alle NTA-secties, de opname en de dialogen), `decimal-input.test.tsx` en `decimal-input-typing.test.tsx`.
- **Werkstroom van begin tot eind:** `workflow-e2e.test.tsx` en `workflow-e2e-040.test.tsx` (gemeenschappelijke ruimte, kelderwand, opgegeven F_D;dayl, PV-waarschuwing, grondwand in opname en project).
- **Normversie wisselen:** `nta-edition-switch.test.tsx` (het overzicht bij een wissel, beide knoppen, terugdraaien, onbereikbare kern).
- **Overnemen uit de opname:** `survey-takeover-ground.test.tsx` (vloeren op de grond per waarde, conflicten alleen op keuze).
- **Dossier, bewijs en opslag:** `dossier-evidence-hardening.test.tsx`, `dossier-desktop-save.test.ts`, `project-dossier.test.ts`, `ep-online-dossier-review18.test.ts`, `project-files.test.ts` en `app-file-safety.test.tsx`.
- **Import en export:** `foreign-import-review16.test.ts`.
- **Versies en handleiding:** `nta-kernel-version.test.ts`, `nta-manual.test.ts` en `manual-view.test.tsx`.

## 3. Testset ISSO 54

**BRL 9501 §3.0 en §4.2 (p. 5 en 7):** de aanvrager voert de deeltesten van ISSO 54 zelf uit. Elke deeltest ligt binnen de bandbreedte (1 % per deeltest). Versie 5.0:2026 met de verwachte uitkomsten is nog niet in bezit. De voorbereiding staat in de [voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md).

**Nu.** 260 van de 266 deeltesten van versie 2.0 zijn gecodeerd:

- generator: `scripts/isso54-suite.py`;
- suite: `training-data/reference-suites/isso54-v2.json`;
- basisprojecten: `training-data/isso54/`.

Ze rekenen in elke gate mee als `pending_expectation`. Een deeltest die niet rekent, laat de gate falen.

**Zodra versie 5.0 er is:**

1. Codeer de deeltesten van 5.0 in een nieuwe suite `isso54-v5.json`, of werk de 2.0-codering bij waar de testomschrijving gelijk is. Doe dat in uitgave 2025+C1. Leg elke interpretatie vast in de voorbereiding.
2. Vul per deeltest de verwachte waarden uit bijlage 2 in onder `expected`, met `relativeTolerance` 0,01. Verwijder `pending`.
3. Draai de gate. Het referentierapport is het bewijs voor §3.0, §4.2, §6.2 en §6.3.
4. Een deeltest buiten de band wordt onderzocht en hersteld (rekenkernfout), of vastgelegd als interpretatie en voorgelegd aan NEN of TC 9501 ([vragen aan NEN](nta8800-vragen-nen.md)).
5. **Vervolgonderzoek (§7.1.3, p. 11):** elke twee jaar een steekproef van minstens 10 % van de tests per deelgebied, op de dan geldende vrijgave.
6. **Herattestering na een rekenkernwijziging (§7.2, p. 12–13):** alle nieuwe en gewijzigde deeltesten. De instelling neemt een steekproef van 25 % van de nieuwe en gewijzigde, en 10 % van de ongewijzigde per hoofdgroep.

Het rapport (`reference-report.md`) wordt per vrijgave gearchiveerd. Het testrapport (§7) verwijst ernaar.

**Testsetdossier voor de instelling (§3.3, p. 5–6).** `scripts/release-nta.sh` schrijft bij elke vrijgave (ook bij `--dry-run`) `release/<tag>/testset/`:

- `projecten/<caseId>.oes.json`: elke gecodeerde deeltest als projectbestand dat het programma opent, met het rekenkernstempel van de berekening. De instelling kan de invoer zo in het programma bekijken en een deeltest zelf opnieuw invoeren;
- `testset.md`, `testset.csv` en `testset.json`: per deeltest het onderwerp (in eigen woorden), de uitgave, de status, BENG 1/2/3, TOjuli, de geleverde energie per dienst en de SHA-256 van projectbestand en antwoord. De kolommen *verwacht* en *oordeel* vullen zich zodra de deeltest in de suite een verwachting heeft. Het oordeel is dat van de vergelijking in `vergelijking/`: *binnen de band* (`compared_pass`), *buiten de band* (`compared_fail`, ook bij een afwijkende labelklasse) of *niet vergeleken (status)* als een deeltest met verwachting niet rekent. Een eerdere inhoud van `projecten/`, een dubbele of onveilige `caseId` laat het maken van het dossier mislukken;
- `vergelijking/`: het referentierapport van alleen deze suite.

De getallen zijn het antwoord op het projectbestand zoals het is weggeschreven, via dezelfde bewerking als de HTTP-route (`calculate_project_performance`). `crates/nta8800-service/tests/testset_roundtrip.rs` laat zien dat twaalf deeltesten (woning en utiliteit) na openen precies rekenen als de suite, en `src/__tests__/testset-project-files.test.ts` dat de app die bestanden opent en ongewijzigd naar de kern stuurt. Het manifest telt de projectbestanden en geeft de SHA-256 van het overzicht; `SHA256SUMS` dekt alle bestanden. Los: `cargo run --manifest-path crates/nta8800-service/Cargo.toml --bin reference_gate -- --suite training-data/reference-suites/isso54-v2.json --testset-dir <map> --testset-topics training-data/isso54/onderwerpen.json` (afsluitcode 3 zolang geen deeltest een verwachting heeft).

## 4. Handmatige acceptatietests

Uit te voeren vóór elke vrijgave, op het gebouwde desktoppakket (`release/<tag>/pakketten/`), door iemand die de wijzigingen niet zelf heeft gemaakt. Leg het resultaat vast in het testrapport (§7).

| Nr | Test | Stappen | Verwacht resultaat | Geslaagd |
|---|---|---|---|---|
| H1 | Installatie | Installeer het `.deb` op een schone machine en start de app. | De app start; Instellingen › Over toont de programmaversie en de handleidingstempel, de statusbalk toont `KERNEL_VERSION`; beide gelijk aan het leveringsdocument. | ☐ |
| H2 | Woning van opname tot label | Nieuw project, woning. Vul de basisopname in, "Opname doorrekenen", "Overnemen in projectmodel", Toepassen, Rekenen. | Het label en de BENG-waarden verschijnen. Het rapport, het EP-Online-overzicht en de labeldata tonen dezelfde getallen. | ☐ |
| H3 | Utiliteit van opname tot label | Als H2, met een kantoor (utiliteitsopname). | Idem; klassengrenzen volgens bijlage X van de Omgevingsregeling. | ☐ |
| H4 | Wisselen van normversie | Open een rekenend project en kies achtereenvolgens 2024, 2023, 2022 en 2020+A1, en dan terug naar 2025+C1. | Bij een oudere uitgave: melding "oudere uitgave, niet registreerbaar", status `calculated_legacy_edition`, bij elke wissel eerst het overzicht van wat de nieuwe uitgave weigert of eist, met schakelpunt en "Ga naar"; "Automatisch aanpassen" wijzigt alleen veilige punten en één keer ongedaan maken herstelt alles. Uitgavevelden verschijnen of verdwijnen. Terug in 2025+C1: registreerbaar, en achtergebleven velden krijgen een verwijderknop. | ☐ |
| H5 | Invoer per raam | Geef één raam een belemmering, zonwering en glas per raam. Verwijder daarna een ander raam. | Alleen dat raam wijzigt in de uitkomst. Na verwijderen blijven geen verweesde regels achter. "Ga naar" landt op het juiste veld. | ☐ |
| H6 | Bijlage AA | Kies bijlage AA voor koelvermogen, voeg ruimten en ramen toe, en maak een tweede gekoelde zone aan. | Per zone alleen de eigen ramen. Geen `annex_aa_window_unknown`. | ☐ |
| H7 | Bron en bewijs, dossier | Koppel een foto aan een raam en een PDF aan een bronveld. Exporteer het dossier. | Het ZIP bevat de bestanden onder `evidence/`. `manifest.json` noemt per bestand de SHA-256, en die komt overeen met `sha256sum` van het uitgepakte bestand. | ☐ |
| H8 | Herlabelen | Leg een origineel vast, wijzig een wand-U (toegestaan), een opwekker (niet toegestaan) en q_v10 (te beoordelen). | De oordelen zijn respectievelijk toegestaan, niet toegestaan en te beoordelen. Een wijziging na de vergelijking geeft "vergelijk opnieuw". | ☐ |
| H9 | Verklaringen k en l | Vul bij registratie de verklaringen k en l in (ja/nee). Sla op en heropen. | De antwoorden blijven bewaard en staan in het rapport en het EP-Online-overzicht als ja/nee. Zonder antwoord: melding bij registratie. | ☐ |
| H10 | EP-Online-overzicht | Maak het overzicht voor een woning en voor een gelijkende woning. | `Op_basis_van_referentiegebouw` is onwaar voor de bezochte en waar voor de gelijkende woning. Velden binnen de XSD-lengtes. Bij een oudere uitgave geen energieklasse. | ☐ |
| H11 | Opslaan en sluiten | Wijzig een project zonder op te slaan en sluit het venster. Annuleer, sla op, sluit opnieuw. | Het venster vraagt per document Opslaan / Niet opslaan / Annuleren. Opslaan schrijft het bestand en een `.bak`. | ☐ |
| H12 | Recent na herstart | Sluit de app, start opnieuw en open het project via "Recent". | Het project opent. De lijst blijft staan. | ☐ |
| H13 | Nieuwere bestandsversie | Open een projectbestand met `"version": "2.0"`. | Duidelijke weigering (NL/EN). Niets gaat stil verloren. | ☐ |
| H14 | Import Uniec 3 en VABI | Importeer een Uniec 3- en een VABI-bestand. | Een melding met het aantal importpunten. Projectinformatie › Importverslag toont bestandsnaam, SHA-256 en de punten. Geen U = 0. ΣψL is gelijk aan de bron. | ☐ |
| H15 | Handleiding | Open de handleiding via Ctrl+K en via de knop in een stap. | De juiste hoofdstukken openen. Versies in het zijpaneel gelijk aan programma en kern. | ☐ |
| H16 | API | Start `api` en vraag `/v1/version` op. Reken een voorbeeldproject (`POST /v1/nta8800/performance/calculate`). Stuur meer gelijktijdige verzoeken dan `--max-calculations` plus `--max-waiting`. | Versie gelijk aan het leveringsdocument; berekening 200; overloop 503 `server_busy` met `Retry-After`. | ☐ |
| H17 | MCP | Start `mcp` over stdio. Roep `tools/list` aan, een rekentool, en stuur een kapotte JSON-regel. | Toolslijst en uitkomst correct. Kapotte regel geeft −32700. De server blijft antwoorden. | ☐ |
| H18 | Gemeenschappelijke ruimte | Utiliteitsgebouw met één rekenzone: voeg bij Eigen gegevens per rekenzone een gemeenschappelijke ruimte toe met bediende functies en bron, laat tapwater en verlichting verdelen, en reken. | Het rapport toont de verdeling binnen de zone en over het gebouw (6.6.4) en welke lijsten de kern vulde. Eigen oppervlakten plus gemeenschappelijke ruimte ongelijk aan de zone geeft `common_space_area_mismatch`. | ☐ |
| H19 | Vensterbreedtes | Zet het venster op 1024, 1280, 1366 en 1920 px breed (Tauri-minimum 800 × 600). Bekijk bij elke breedte het welkomstscherm, Project, Gebouw › Schil &amp; ramen (tot en met *Belemmering per raam*), Gebouw › Rekenzones, Installaties › Verwarming (de stappenbalk), Controle, Resultaten (Overzicht, Per dienst, Maandwaarden), Registratie, Rapport &amp; dossier, Basisopname, Gereedschap › Handleiding, en de dialogen Instellingen en Uitgave wijzigen. | Niets valt buiten beeld of over elkaar: de documenttabs blijven leesbaar en de knop voor een nieuw tabblad zichtbaar (onder 1280 px tonen zoeken, Opslaan en Herberekenen alleen hun pictogram); de stappenbalk en brede tabellen schuiven horizontaal in plaats van te overlappen; tabellen per raam nemen de hele breedte van het blok; elk invoerveld heeft één rand. | ☐ |

## 5. Regressieprocedure bij een wijziging

**Tests per soort wijziging:**

| Soort wijziging | Minimaal | Aanvullend |
|---|---|---|
| Rekenkern (formule, tabel, route) | volledige gate met MSRV | paginaverwijzing in de unittest; differentiële test per uitgave; openbare gevallen en referentiesuites bijwerken met verklaring; end-to-endvergelijking (hieronder) |
| Invoertypen van de kern | volledige gate | optiedekking en formulierdekking (de catalogus `training-data/nta8800-input-options.json` bijwerken met `UPDATE_INPUT_OPTIONS=1`); formulier en vertaling |
| Formulieren en UI | gate zonder MSRV | formulierbewaking; acceptatietest van het betreffende scherm (§4) |
| Rapport, dossier, EP-Online | gate zonder MSRV | H7, H9, H10 |
| Service, API, MCP | gate zonder MSRV | H16, H17 |
| Desktop en bestanden | gate en desktopbuild (`npm run tauri:build -- --debug --bundles deb`) | H11–H13 |
| Import en export | gate zonder MSRV | H14 |
| Alleen documentatie | versie- en handleidingscontrole (m) | — |

**Versieverhoging.** Verandert een wijziging uitkomsten, statussen of meldcodes van de kern, dan krijgt ze een item onder "## Onuitgebracht" in de [releasenotes](nta8800-releasenotes.md). Zonder "(geen rekenwijziging)" aan het eind van de titel vraagt de gate bij de volgende vrijgave een MINOR-stap van `KERNEL_VERSION` ([versiebeheer](nta8800-versiebeheer.md)). Zolang een kernversie niet is uitgebracht, is één stap per vrijgave genoeg. De handleiding wordt opnieuw gestempeld als de kernversie verandert.

**End-to-endvergelijking.** Bij een wijziging van de rekenkern, en altijd vóór een vrijgave, gaan alle voorbeeldprojecten door kern, rapport, EP-Online-overzicht en dossier, in elke uitgave die ze ondersteunen:

- de openbare gevallen A–O;
- de RVO-voorbeeldwoningen;
- de opnamevoorbeelden;
- een steekproef van ISSO 54-deeltesten.

Elk veranderd getal wordt toegeschreven aan een item in de releasenotes: BENG 1/2/3, labelklasse, TOjuli of aandeel hernieuwbaar. Een verandering zonder verklaring is een mogelijke regressie en houdt de vrijgave tegen.

Deze werkwijze is toegepast in review 10 (basis) en review 17 (herhaling, zonder onverklaarde verschillen). De harnassen staan beschreven in die reviews: per project een regel met alle indicatoren, vóór en na naast elkaar.

**Doorlooptijd van de gate.** `scripts/verify-nta.sh` drukt per stap de duur af en sluit af met een overzicht. Meting op 12 oktober 2026, koude build in een verse werkmap, 8 kernen, 13 GB:

| Stap | Tijd (s) |
|---|---|
| kerntests (`crates/nta8800-core`) | 357 |
| clippy kern | 49 |
| servicetests | 123 |
| clippy service | 41 |
| referentiesuites | 11 |
| kern op Rust 1.77.2 | 426 |
| frontendbundel | 33 |
| Tauri check | 109 |
| Tauri-commandotests | 138 |
| TypeScript | 25 |
| UI-tests | 385 |
| app tegen de echte kern (`kernel_once`) | 55 |
| projectformulieren door de kern | 43 |
| **totaal** | **1798** |

Tests per run: de kern 985 unittests plus de integratietests, de UI 1614 tests in 172 bestanden (7 overgeslagen), de app-run 7 tests. Met `NTA_GATE_PARALLEL=1` lopen de Rust 1.77.2-run en de frontend/Tauri-pijplijn naast de kernstappen. Dat staat standaard uit: op deze machine gaf het een geheugentekort, en een parallelle meting is daarom nog niet gedaan. De kerntests besteden het grootste deel aan een paar binaries met twee of drie zware tests (82 s en 57 s); daar valt de meeste winst te halen.

## 6. Codereviews

Naast de geautomatiseerde tests wordt elke wijziging gereviewd. Grote wijzigingen krijgen daarnaast een gerichte review.

**Werkwijze:**
1. Een review per gebied, alleen lezend, tegen de broncode en de normtekst. De reviewer bevestigt een vermoeden met een eigen test in een kopie van de code.
2. Elke bevinding komt met bestand, regel, scenario en voorgestelde oplossing.
3. Elke bevestigde bevinding wordt hersteld met een test die zonder de oplossing faalt. Daarna draait de gate.

Gerichte reviews tot nu toe:

| Ronde | Gebied |
|---|---|
| 1 | Belemmering per raam, installatiejaar warmtepomp, uitgavebehandeling in service en desktop, bewijskoppelingen op id, bijlage AA-formulier |
| 2 | Geweigerde routes als invoergat, labelelementen k/l, opname-id's, referentierunner en toleranties, versiescripts, handleidingweergave |
| 3 | Opgegeven f_ctrl, zonwering per raam, bereikcontroles uit de fuzz, openbare gevallen G/H |
| 4 | Gelijktijdigheid en time-out van de HTTP-API, referentiegevallen zonder verwachting, ISSO 54-codering |
| 5 | Nieuwe kernroutes: glas per raam, ketel A/B/C, WKK-rendementen, Ψ uit leidinggeometrie, boosterwarmtepomp |
| 6 | Forfaitaire koudebruggen met onverwarmde ruimte, gemengde luchtverwarmers |
| 7 | Nieuwe formulieren (formulierdekking) |
| 8 | Overige formulieren: zonedata, circulatie, verdere tapwatersystemen, opnamevelden |
| 9 | Lege waarden naar de kern, invoervingerafdruk |
| 10 | Van begin tot eind: voorbeelden door kern, rapport, EP-Online-overzicht en dossier |
| 11 | c_W;gen van de forfaitaire tapwaterwarmtepomp, ontvochtiging bij directe expansie |
| 12 | Decimale-kommavelden in de oudere schermen |
| 13 | Basisopname, maatwerkadvies en herlabelen tegen ISSO 82.1/75.1, 82.2/75.2 en BRL 9500 bijlage 6a/6b |
| 14 | HTTP-API en MCP-server |
| 15 | Desktoplaag en bestandsopslag |
| 16 | Import en export Uniec 3 en VABI |
| 17 | Herhaling van ronde 10 na alle herstellingen |
| 18 | Uitvoer tegen Omgevingsregeling, BRL 9500 en EP-Online-XSD |
| 19 | Steekproef van 41 ISSO 54-coderingen |

De herstellingen staan per ronde in de [releasenotes](nta8800-releasenotes.md).

## 7. Vastlegging

**Bij elke vrijgave.** `scripts/release-nta.sh` maakt het archief `release/<tag>/` met:

- het gatelog (`gate.log`) en de samenvatting daarvan (`gate-samenvatting.json`: per gatestap de Rust- en Vitest-aantallen en de duur, en per referentiesuite de gevallen en het oordeel), waaruit het testrapport en `github-release.md` hun aantallen halen;
- het referentierapport (`evidence/reference/reference-report.json` en `.md`);
- de formulierdocumenten (`evidence/form-guard/`);
- de pakketten, de handleiding, het leveringsdocument en `manifest.json`;
- de tekst voor de GitHub-release (`github-release.md`), met de SHA-256 van de bestanden die bij de release horen;
- `SHA256SUMS`;
- een vooringevuld testrapport (`testrapport.md`) volgens het sjabloon [nta8800-testrapport](templates/nta8800-testrapport.md), met versies, commit, tag, datum, gatestatus en de gate-uitvoer per stap al ingevuld.

De tester vult daarin de resultaten per niveau, de handmatige tests (§4), de afwijkingen en de aftekening in. Het ingevulde rapport wordt bewaard bij het archief, volgens het [archiefbeleid](kwaliteit/archiefbeleid.md).

**Bij een wijziging zonder vrijgave.** Bewaar het gatelog bij de commit, of geef in de commit aan dat de gate slaagde.

## 8. Rollen en frequentie

De rollen staan in het [kwaliteitshandboek](kwaliteit/kwaliteitshandboek.md).

| Wat | Wie | Wanneer |
|---|---|---|
| Gate | ontwikkelaar | bij elke wijziging, vóór het vastleggen |
| Gerichte review | interne reviewer | bij elke grote of rekenkern-wijziging |
| End-to-endvergelijking | ontwikkelaar, gecontroleerd door de reviewer | bij elke rekenkernwijziging en vóór elke vrijgave |
| Handmatige acceptatietests (§4) | tester, niet de maker van de wijziging | vóór elke vrijgave |
| Testrapport en aftekening | vrijgever | bij elke vrijgave |
| ISSO 54-run met oordeel | ontwikkelaar | bij elke vrijgave zodra versie 5.0 er is; steekproef om de twee jaar (§7.1.3) |
| Evaluatie van dit protocol | kwaliteitsverantwoordelijke | in de jaarlijkse review van het kwaliteitshandboek |
