# NTA 8800 HTTP-API

De HTTP-API stelt de Rust-rekenkern van Open Energy Studio beschikbaar voor andere programma's. Elke bewerking bestaat precies één keer, in het register `crates/nta8800-service/src/operations.rs`. Daaruit worden drie dingen afgeleid:
- de HTTP-route;
- de MCP-tool met dezelfde naam (zie [nta8800-mcp.md](nta8800-mcp.md));
- het OpenAPI-document.

Daardoor kunnen API, MCP-server en documentatie niet uit elkaar lopen. Een test controleert dat elke bewerking een route en een OpenAPI-pad heeft, en omgekeerd.

Uitkomsten zijn **onverifieerd**: het programma heeft nog geen BRL 9501-attest. Labels uit de API zijn indicatief en niet geregistreerd.

## Starten

```
cargo run --release --manifest-path crates/nta8800-service/Cargo.toml --bin api
```

| Optie | Omgevingsvariabele | Standaard |
|---|---|---|
| `--bind ADRES` | `OES_API_BIND` | `127.0.0.1` |
| `--port POORT` | `OES_API_PORT` | `3007` |
| `--cors-origin ORIGIN` (herhaalbaar, `*` voor alles) | `OES_API_CORS_ORIGINS` (kommagescheiden) | geen CORS |
| `--allowed-host HOST` (herhaalbaar) | `OES_API_ALLOWED_HOSTS` (kommagescheiden) | op een loopback-adres: `localhost`, `127.0.0.1` en `::1`, plus de opgegeven namen; op een ander adres alleen de opgegeven namen (geen: geen controle) |
| `--body-limit-mb N` | `OES_API_BODY_LIMIT_MB` | `16` (1–1024) |
| `--max-json-values N` | `OES_API_MAX_JSON_VALUES` | `500000` (1000–100000000): het grootste aantal JSON-waarden (array-elementen, objectleden, getallen, teksten) in een verzoek |
| `--max-calculations N` | `OES_API_MAX_CALCULATIONS` | aantal processorkernen (1–1024): zoveel berekeningen lopen tegelijk, de rest wacht |
| `--max-waiting N` | `OES_API_MAX_WAITING` | twee keer het aantal rekenplaatsen (0–65536): zoveel verzoeken mogen op een rekenplaats wachten; een verzoek daarboven krijgt meteen 503 `server_busy`, nog voordat de body is gelezen |
| `--queue-timeout-s N` | `OES_API_QUEUE_TIMEOUT_S` | `30` (1–86400): langste wachttijd op een rekenplaats; daarna 503 `server_busy` en er start geen berekening |
| `--calculation-timeout-s N` | `OES_API_CALCULATION_TIMEOUT_S` | `120` (1–86400): langste duur van de berekening zelf, gerekend vanaf het moment dat zij een rekenplaats krijgt |
| `--log` / `--no-log` | `OES_API_LOG` (`1`/`0`) | aan: één regel per verzoek op stderr |
| `--version` | | drukt de versie-informatie af en stopt |

Een vlag wint van de omgevingsvariabele. Op een niet-loopback-adres meldt de server een waarschuwing: de API heeft geen authenticatie en is bedoeld voor lokaal gebruik. SIGINT en SIGTERM stoppen netjes: de server neemt geen nieuwe verbindingen meer aan en laat lopende verzoeken afmaken.

**Geheugen.** Een ingelezen body kost geheugen per JSON-waarde, niet per byte. Gemeten (`crates/nta8800-service/tests/memory_bound.rs`):
- een compact array-element kost ongeveer 42 bytes;
- een waarde van een echt project (openbaar geval H) ongeveer 136 bytes, dat is 3,8 bytes per byte JSON.

Een body van 16 MiB met alleen nullen (`[0,0,…]`) zou zo ruim 300 MB innemen. Daarom telt de service eerst de waarden en weigert een verzoek boven `--max-json-values` met 413 `payload_too_large`, nog voordat de JSON wordt ingelezen. Met de standaard van 500 000 waarden kost een ingelezen body hoogstens ongeveer 70 MB (136 bytes × 500 000). Een echt project heeft ongeveer 650 waarden per rekenzone; de standaard laat dus ongeveer 750 zones toe, bijna twee keer het grootste project uit de prestatiemeting (400 zones).

Geen route kopieert de body: het invoerlid gaat in zijn geheel door naar de kern (de test controleert dit). Een toegelaten verzoek houdt dus één ingelezen body vast. Het geheugen van alle toegelaten verzoeken samen is daarmee ten hoogste:

`(max-calculations + max-waiting) × (ca. 136 bytes × max-json-values)` plus het rekengeheugen van de lopende berekeningen (circa 0,7 MB per rekenzone, zie [Grenzen en prestaties](nta8800-programmabeschrijving.md#8-grenzen-en-prestaties)).

Met 8 rekenplaatsen, 16 wachtplaatsen en de standaarden is dat ongeveer 1,7 GB in het slechtste geval. Verlaag `--max-json-values` of `--max-waiting` op een machine met minder geheugen. Het verzoek wordt toegelaten of geweigerd voordat de body wordt gelezen; de bytes van een body (hoogstens `--body-limit-mb`) worden losgelaten zodra de JSON is ingelezen.

De wachttijd en de berekening hebben elk hun eigen tijdslimiet. Een verzoek dat binnen `--queue-timeout-s` geen rekenplaats krijgt, krijgt 503 `server_busy` en start geen berekening; een verzoek dat lang heeft gewacht, krijgt dus nooit een berekening die meteen weer wordt afgebroken. Een berekening die langer duurt dan `--calculation-timeout-s`, geeft 503 `calculation_timeout`; de service maakt haar op de achtergrond af en houdt zolang haar rekenplaats bezet. Elke 503 heeft een `Retry-After`-header: een kwart van de rekentijdslimiet, tussen 1 en 60 seconden.

Hoe lang een berekening duurt, staat in [programmabeschrijving §8](nta8800-programmabeschrijving.md#8-grenzen-en-prestaties). Een utiliteitsgebouw van 50 rekenzones met 2 250 ramen rekent in ongeveer 0,5 s (release-build). Een project met lege velden duurt iets langer, omdat de kern elk leeg veld opzoekt: 200 lege velden in dat gebouw kosten ongeveer 1,6 s. Beide blijven ruim binnen de standaardrekentijd van 120 s.

**Hostnamen.** Een webpagina kan haar eigen domeinnaam naar 127.0.0.1 laten wijzen (*DNS rebinding*) en zo een lokale service bereiken. Op een loopback-adres beantwoordt de service daarom alleen `localhost`, `127.0.0.1` en `::1` (met of zonder poort) plus de namen uit `--allowed-host`. Een andere `Host` krijgt 403 `host_not_allowed`. Een verzoek zonder `Host`-header (geen browser) gaat door.

**CORS.** Met `--cors-origin` krijgt elk antwoord `Vary: Origin`, ook een geweigerde preflight, zodat een cache een antwoord voor de ene origin niet aan een andere geeft.

De ontwikkelserver (`npm run dev`) stuurt `/api/*` door naar poort 3007; de desktop-app roept de kern rechtstreeks aan en gebruikt de API niet.

## Versies en identiteit

- Alle bewerkingen staan onder `/v1`. Een wijziging die bestaande clients breekt, krijgt een nieuw voorvoegsel; toevoegingen (nieuwe routes, nieuwe velden in antwoorden) blijven binnen `v1`.
- `GET /v1/version` geeft:
  - `serviceVersion` en `apiVersion`;
  - `kernelVersion` (het rekenkerngedeelte van het versienummer, BRL 9501 §5.2);
  - `targetNormVersion` (NTA 8800:2025+C1:2026);
  - `supportedNormVersions`: de bekende edities van NTA 8800 (`id`, `label`, `implemented`, `registrationEligible`, `default`, aanwijzingsperiode). Een project kiest een editie met `ntaCalculation.normVersion` (`"2020+A1"`, `"2022"`, `"2023"`, `"2024"` of `"2025+C1"`, standaard `"2025+C1"`); zie [nta8800-normversies.md](nta8800-normversies.md);
  - `buildCommit` (uit `OES_BUILD_COMMIT` tijdens het bouwen, anders `null`);
  - `buildFingerprint`, een SHA-256 over versies en bewerkingentabel. Twee builds met dezelfde vingerafdruk hebben dezelfde kern en dezelfde routes.
- Elke rekenuitkomst bevat zelf ook `kernelVersion`, `targetNormVersion` en `inputFingerprint`; projectuitkomsten ook `normVersion` en `registrationEligible`. Een berekening in een oudere editie heeft status `calculated_legacy_edition` (HTTP 200) en is nooit registreerbaar (uitzondering: een herlabeling in de editie van het oorspronkelijke project).
- **Editie per verzoek.** Elke `POST`-bewerking (en elke MCP-tool) neemt naast het invoerlid het optionele lid `normVersion` (`"2020+A1"`, `"2022"`, `"2023"`, `"2024"`, `"2025+C1"`; standaard `"2025+C1"`). De service schrijft het in de eigen plek van de invoer als die ontbreekt (`ntaCalculation.normVersion`, `survey.normVersion`, `input.normVersion`, de basis van het maatwerkadvies, beide projecten bij herlabelen) en rekent de constructies en diagnoses met die editie actief. Elke uitkomst krijgt `normVersion` en `targetNormVersion`. Een referentiegeval (`reference/audit`, `reference/compare`) rekent in zijn eigen `case.normVersion` (elke geïmplementeerde editie) en krijgt die editie als stempel; een `normVersion` in het verzoek moet daarmee overeenkomen, anders 400 `norm_version_conflict`. De twee diagnosevergelijkingen (`reference/direct-diagnostic/compare`, `reference/gas-chain-diagnostic/compare`) nemen alleen `"2025+C1"`. Details en de betekenis per route: [nta8800-normversies.md](nta8800-normversies.md#uitgave-per-route).
- `GET /v1/nta8800/capabilities` geeft de reikwijdte van de kern:
  - `implemented`: de onderdelen, waaronder de projectberekening met meer rekenzones, de basisopnames, het maatwerkadvies, herlabelen en registratie;
  - `projectCalculationAvailable` (`true`): de projectberekening rekent BENG, TOjuli en het indicatieve label;
  - `calculationAvailable` (`false`): alleen de verouderde route `/v1/nta8800/calculate` rekent niet; het veld blijft voor oudere clients;
  - `supportedNormVersions`: dezelfde lijst als `/v1/version`.
- `GET /health` geeft `{"status":"ok","kernel":"rust","kernelVersion":"…"}`.
- `GET /v1/openapi.json` geeft het OpenAPI 3.1-document. Daarin staat per bewerking `operationId` en `x-mcp-tool` (de MCP-toolnaam). De `servers`-regel is relatief (`/`): het document beschrijft de service die het levert, op welk adres die ook luistert (standaard `http://127.0.0.1:3007`).

## Statuscodes en foutmodel

| Code | Betekenis | Body |
|---|---|---|
| 200 | Uitkomst | het resultaat van de kern |
| 400 | Ongeldig verzoek: geen geldige JSON (ook: dieper genest dan 128 niveaus), geen object, een onbekend of ontbrekend lid, verkeerde vorm of een `normVersion` die niet past | foutenvelop |
| 403 | De `Host`-header noemt een naam die de service niet beantwoordt (`host_not_allowed`) | foutenvelop |
| 404 / 405 | Onbekende route of verkeerde methode | foutenvelop |
| 413 | Body groter dan de bytelimiet of met meer JSON-waarden dan `--max-json-values` (`payload_too_large`) | foutenvelop |
| 415 | `Content-Type` is geen `application/json` | foutenvelop |
| 422 | De kern weigert of kan niet afmaken (status `invalid`, `incomplete`, `derived_input_rejected`, `invalid_case`, …) | de **beoordeling zelf**, met `status`, `gaps` en `issues`, ook bij de verouderde route `/v1/nta8800/calculate` |
| 500 | Uitkomst achtergehouden (`non_finite_result`, `serialization_failed`) of kernfout (`kernel_panic`) | foutenvelop |
| 501 | Alleen de verouderde route `/v1/nta8800/calculate` | foutenvelop |
| 503 | Wachtrij vol of geen rekenplaats vrij binnen de wachttijdslimiet (`server_busy`), of de berekening duurde langer dan de rekentijdslimiet (`calculation_timeout`) | foutenvelop, met `Retry-After` |

De foutenvelop:

```json
{
  "error": "invalid_request_shape",
  "code": "invalid_request_shape",
  "message": "`survey.constructionYear`: invalid type: string \"nineteen thirty\", expected …",
  "path": "survey.constructionYear",
  "details": null
}
```

`error` en `code` zijn gelijk; `error` blijft bestaan voor oudere clients. `path` is het JSON-pad van de foute invoer of, bij `non_finite_result`, van de niet-eindige uitkomst. Codes:
- `invalid_json`, `invalid_request_shape` (ook: een onbekend lid naast het invoerlid en `normVersion`), `missing_request_member`, `invalid_project_shape` (ook: `original`, `current` of `project` bij herlabelen is geen object), `invalid_maatwerkadvies_shape`;
- `invalid_norm_version` (onbekende editie; `details.supportedNormVersions`), `norm_version_conflict` (de invoer of het referentiegeval heeft al een andere editie), `norm_version_not_applicable` (geen plek voor de editie, of een diagnosevergelijking buiten 2025+C1), `edition_not_implemented` (een editie zonder profiel van de kern; geen enkele bekende editie heeft dat nu);
- `payload_too_large` (`details.maxJsonValues` als het aantal waarden te groot is), `unsupported_media_type`, `host_not_allowed`, `not_found`, `method_not_allowed`;
- `non_finite_result`, `serialization_failed`, `kernel_panic`, `calculation_unavailable`, `server_busy`, `calculation_timeout`;
- alleen via MCP: `unknown_tool`.

Een 422 is geen fout van de client of de server, maar een rekenuitkomst: lees `gaps` (ontbrekende invoer met pad) en `issues` (strijdige invoer). De betekenis van elke code staat in de app en in [hoofdstuk 5 van de handleiding](handleiding-nta8800/05-validatie.md).

Vijf bewerkingen antwoorden nooit 422; hun bevindingen staan in de body bij 200:
- `validate_project` (status `invalid` of `structurally_valid`; de app leest de beoordeling ook bij `invalid`);
- `assess_relabel`;
- `get_label_input_hash`;
- `audit_reference_case`;
- `compare_project_editions` (wat de doeluitgave weigert of vraagt, staat in `items` en `remainingAfterAutomaticChanges`).

Het OpenAPI-document noemt 422 alleen bij de bewerkingen die het kunnen geven, en 403 en 405 bij elke route. Elke code die de service kan geven, staat in de beschrijving van `Error.code`; een test leest de bronnen en houdt die lijst volledig. Een tweede test stuurt elke bewerking lege, verkeerde en onbekende leden, een onbekende of oudere `normVersion`, een verkeerd `Content-Type` en de verkeerde methode, en controleert dat elk antwoord een gedocumenteerde status en code heeft en geen paniek- of bronpad noemt.

`GET /v1/nta8800/interpretations` geeft als enige een lijst (per module een groep), geen object; het OpenAPI-document beschrijft die vorm.

## Bewerkingen

Elke `POST` verwacht een JSON-object met één lid (soms twee), genoemd in de derde kolom, plus het optionele lid `normVersion`. Andere leden worden geweigerd (400 `invalid_request_shape`), zodat een verschreven `normversion` niet ongemerkt in de standaardeditie rekent; de schema's in het OpenAPI-document hebben `additionalProperties: false`. `null`-leden van objecten mag je weglaten; de desktop-app doet dat ook.

| Route | MCP-tool | Invoerlid | Wat |
|---|---|---|---|
| `GET /v1/version` | `get_version` | — | Report the service, kernel and target norm version (NTA 8800:2025+C1:2026), the API version and a build fingerprint |
| `GET /v1/nta8800/capabilities` | `get_capabilities` | — | Show the Rust NTA 8800 kernel scope, target version and attest status |
| `GET /v1/nta8800/interpretations` | `list_interpretations` | — | List the kernel's documented readings of ambiguous or contradictory NTA 8800 passages (interpretatielijst), grouped per module |
| `POST /v1/nta8800/validate` | `validate_project` | `project` | Validate an Open Energy Studio project structure |
| `POST /v1/nta8800/calculate` | `calculate_beng` | `project` | Legacy endpoint |
| `POST /v1/nta8800/project/performance` | `calculate_project_performance` | `project` | Calculate a saved .oes project with its ntaCalculation block per NTA 8800:2025+C1:2026 |
| `POST /v1/nta8800/project/editions/compare` | `compare_project_editions` | `project` | Compare a project between its own edition and the request's `normVersion` (zie *Uitgave wijzigen* hieronder) |
| `POST /v1/nta8800/project/energy-by-service` | `get_energy_by_service` | `project` | Return delivered, primary fossil and renewable energy per service (verwarming, tapwater, koeling, bevochtiging, ventilatoren, verlichting, hulpenergie), carrier and month (NTA 8800 §5.5.3) |
| `POST /v1/nta8800/label/data` | `get_label_data` | `project` | Return the label data (labelgegevens) per Omgevingsregeling art. 5.13 and 5.13a |
| `POST /v1/nta8800/registration/assess` | `assess_registration` | `project` | Check a project's registration block for EP-Online registration per BRL 9500-W/U (29-05-2026) and the Omgevingsregeling |
| `POST /v1/nta8800/maatwerkadvies` | `assess_maatwerkadvies` | `input` | Calculate a maatwerkadvies (BRL 9500-MWA, ISSO 82.2/75.2) |
| `POST /v1/nta8800/relabel/assess` | `assess_relabel` | `original`, `current` | Compare an original and a current project for herlabelen per BRL 9500-W/U Bijlage 6a/6b |
| `POST /v1/nta8800/relabel/label-input-hash` | `get_label_input_hash` | `project` | Compute the canonical SHA-256 of a project's label input as stored with a relabel comparison |
| `POST /v1/nta8800/transmission/direct/diagnose` | `diagnose_direct_transmission` | `input` | Diagnose the direct-to-outdoor A·U + L·psi + chi sum |
| `POST /v1/nta8800/transmission/direct/monthly-diagnose` | `diagnose_monthly_direct` | `input` | Diagnose signed monthly direct-to-outdoor heat flow from supplied conductance, temperatures and hours |
| `POST /v1/nta8800/demand/monthly/calculate` | `calculate_monthly_demand` | `input` | Calculate the unverified NTA 8800 chapter 7 monthly heating and cooling need of one zone with De Bilt climate |
| `POST /v1/nta8800/heating/space-heating-chain/calculate` | `calculate_space_heating_chain` | `input` | Calculate the unverified monthly space-heating chain (need, emission, distribution, one generator) and energy per carrier |
| `POST /v1/nta8800/ventilation/calculate` | `calculate_ventilation` | `input` | Calculate unverified NTA 8800 chapter 11 ventilation for one zone |
| `POST /v1/nta8800/opname/residential` | `assess_residential_survey` | `survey` | Translate an ISSO 82.1 basic survey (basisopname) of an existing dwelling into NTA 8800 kernel input, list every applied default with its ISSO page, and calculate the unverified building performance and indicative label |
| `POST /v1/nta8800/opname/utility` | `assess_utility_survey` | `survey` | Translate an ISSO 75.1 basic survey (basisopname) of an existing utility building into NTA 8800 kernel input (one calculation zone |
| `POST /v1/nta8800/constructions/calculate` | `calculate_constructions` | `input` | Calculate unverified NTA 8800 8.2 envelope element U- and Rc-values (annexes C, E-I, L) |
| `POST /v1/nta8800/performance/calculate` | `calculate_building_performance` | `input` | Calculate unverified building energy indicators, BENG and an indicative label class when the supplied input is complete |
| `POST /v1/nta8800/transmission/unheated/diagnose` | `diagnose_unheated_transmission` | `input` | Diagnose conductance via unheated spaces using caller-supplied factors |
| `POST /v1/nta8800/heat-pumps/declared-heating-table/diagnose` | `diagnose_declared_heating_table` | `input` | Interpolate a supplied space-heating product declaration table within its bounds |
| `POST /v1/nta8800/heat-pumps/forfait-cop-draft/diagnose` | `diagnose_forfait_heat_pump_draft` | `input` | Look up a base electric heat-pump COP in public draft tables 9.27/9.29 |
| `POST /v1/nta8800/heat-pumps/gas-forfait-cop-draft/diagnose` | `diagnose_gas_heat_pump_forfait_draft` | `input` | Diagnose a gas-engine or gas-absorption heat-pump COP from public draft tables 9.27/9.29 |
| `POST /v1/nta8800/heat-pumps/gas-aux-draft/diagnose` | `diagnose_gas_heat_pump_aux_draft` | `input` | Diagnose gas heat-pump generator auxiliary electricity from draft 9.91/9.92 |
| `POST /v1/nta8800/heat-pumps/gas-forfait-monthly-draft/diagnose` | `diagnose_gas_heat_pump_monthly_draft` | `input` | Diagnose draft 9.62 monthly input terms for a gas-engine or absorption heat pump |
| `POST /v1/nta8800/heat-pumps/gas-chain-draft/diagnose` | `diagnose_gas_heat_pump_chain_draft` | `input` | Link gas heat-pump draft 9.62 and 9.91/9.92 monthly terms by generator, heat and evidence |
| `POST /v1/nta8800/heat-pumps/gas-collective-source-draft/diagnose` | `diagnose_gas_collective_source_draft` | `input` | Diagnose the separate dh heat and draft fossil/renewable primary contribution of a collective gas heat-pump source using public consultation chapters 5 and 9 |
| `POST /v1/nta8800/heat-pumps/forfait-monthly-draft/diagnose` | `diagnose_forfait_heat_pump_monthly_draft` | `input` | Diagnose draft equation 9.62 from supplied monthly heat and a draft COP |
| `POST /v1/nta8800/heating/generator-dispatch-draft/diagnose` | `diagnose_generator_dispatch_draft` | `input` | Diagnose new-build heating generator dispatch from public draft tables 9.1/9.23 and equations 9.2/9.3/9.56/9.60 |
| `POST /v1/nta8800/heat-pumps/hybrid-monthly-draft/diagnose` | `diagnose_hybrid_heat_pump_monthly_draft` | `input` | Diagnose a new-build hybrid heat-pump chain |
| `POST /v1/nta8800/boilers/forfait-draft/diagnose` | `diagnose_boiler_forfait_draft` | `input` | Diagnose draft gas water-boiler efficiency from consultation table 9.25 |
| `POST /v1/nta8800/boilers/forfait-monthly-draft/diagnose` | `diagnose_boiler_forfait_monthly_draft` | `input` | Diagnose draft gas input and individual-boiler forfait auxiliary electricity with equations 9.61/9.85 from supplied monthly boiler heat |
| `POST /v1/nta8800/heat-pumps/declared-dhw/diagnose` | `diagnose_declared_dhw` | `input` | Check declared tap-profile test energies and report their raw ratios |
| `POST /v1/nta8800/energy/final-draft/diagnose` | `diagnose_final_energy_draft` | `input` | Sum supplied monthly carrier energy and solar thermal contributions using public chapter-5 draft arithmetic |
| `POST /v1/nta8800/energy/epus-draft/diagnose` | `diagnose_epus_draft` | `input` | Compose monthly E_EPus per carrier from supplied service terms and BACS factor using public draft equations 5.20–5.21, then provisionally sum final energy |
| `POST /v1/nta8800/energy/bacs-draft/diagnose` | `diagnose_bacs_draft` | `input` | Diagnose provisional fBACS from per-system heating/cooling power and class evidence under public consultation §5.5.8 |
| `POST /v1/nta8800/energy/indicators-draft/diagnose` | `diagnose_indicators_draft` | `input` | Provisional chapter-5 indicator ratios and directional rounding from separately supplied annual totals |
| `POST /v1/nta8800/heat-pumps/heating-aux-draft/diagnose` | `diagnose_heating_aux_draft` | `input` | Provisional chapter-9 equation 9.85 auxiliary electricity for one individual electric heat pump from supplied measured coefficients and input energy |
| `POST /v1/nta8800/heat-pumps/heating-aux-measured-draft/diagnose` | `diagnose_heating_aux_measured_draft` | `input` | Provisional chapter-9 equations 9.86–9.88 derive A/B/C for one electric heat pump from supplied measured powers and cycle inputs, then apply 9.85 |
| `POST /v1/nta8800/reference/audit` | `audit_reference_case` | `case` | Audit a reference-case manifest for missing evidence |
| `POST /v1/nta8800/reference/compare` | `compare_reference_case` | `case` | Compare submitted BENG/TOjuli expectations to the unverified Rust project result |
| `POST /v1/nta8800/reference/direct-diagnostic/compare` | `compare_direct_diagnostic` | `case` | Compare four direct-transmission diagnostic terms to supplied W/K expectations |
| `POST /v1/nta8800/reference/gas-chain-diagnostic/compare` | `compare_gas_heat_pump_chain_diagnostic` | `case` | Compare all twelve gas heat-pump draft 9.62 and equipment-electricity values plus annual equipment electricity to supplied expectations |

### Uitgave wijzigen (`compare_project_editions`)

Bij deze route is `normVersion` de **doeluitgave** (standaard 2025+C1); het project behoudt zijn eigen uitgave in `ntaCalculation.normVersion`. Een project zonder `ntaCalculation` geeft 400 `invalid_project_shape`. De route antwoordt altijd 200 en verandert niets aan het verzoek. De kern rekent het project in beide uitgaven door (`crates/nta8800-core/src/edition_switch.rs`):

| Veld | Inhoud |
|---|---|
| `fromNormVersion`, `toNormVersion` | eigen uitgave en doeluitgave |
| `switchPoints` | de omschakelpunten van [`nta8800-normversies.md`](nta8800-normversies.md) waarvan de profielwaarden tussen de twee uitgaven verschillen (`points`, `subject`, `fields`, `hasInputs`) |
| `items[]` | per invoer: `category` (`refused`, `required`, `ignored`, `reinterpreted`, `warning`), de kerncode, het projectpad, de omschakelpunten (`points`), `fix` en `resolvedByAutomaticChanges` |
| `adjustedProject` | het project in de doeluitgave met de automatische aanpassingen |
| `statusAfterAutomaticChanges`, `remainingAfterAutomaticChanges` | status en nieuwe gaten van `adjustedProject`: precies de punten die voor de adviseur overblijven |

`fix` is `drop`, `switchOff` of `set` (veilig, toegepast in `adjustedProject`), of `enter`, `choose` of `review` (aan de adviseur). Automatisch verandert alleen een lid dat het omschakelpunt noemt, nooit een omsluitend object of lijstelement. Een geweigerd lid dat na het weglaten verplicht blijkt, of waarvan het weglaten het blok onleesbaar maakt, wordt teruggezet en krijgt `choose`. Verplichte waarden worden nooit verzonnen. De gedateerde NEN-EN 14511-2 van een testrapport voor tabel 9.28 (punt 67a) is bewijs en wordt nooit herschreven: het item krijgt `choose` met in `value` de norm die de doeluitgave vraagt. Het OpenAPI-document beschrijft de vorm van het resultaat.

De koppeling van een melding aan een omschakelpunt volgt uit de profielen zelf: elk veld van `NormProfile` hoort bij precies één omschakelpunt (`norm_versions::switches`), en een test controleert dat elk veld dat tussen twee uitgaven verschilt en elke genummerde rij van `nta8800-normversies.md` een omschakelpunt heeft.

### Keuzes bij de pariteit met de desktop-app

- **Projectberekening.** Alles wat de app uit één projectberekening toont, komt uit `calculate_project_performance`: BENG, TOjuli, Bbl-toets, label, labelgegevens (art. 5.13/5.13a), energie per functie, registratiestatus en waarschuwingen. `get_energy_by_service`, `get_label_data` en `assess_registration` geven daar een deel van, voor clients die alleen dat deel nodig hebben.
- **Maatwerkadvies-sjablonen.** De sjablonen (isolatie, beglazing, warmtepomp, PV, …) zetten in de app keuzes om in JSON-patches (`src/core/nta/MwaTemplates.ts`). De API rekent met die patches: `assess_maatwerkadvies` neemt maatregelen met een `patch`. De sjabloongenerator is niet naar Rust overgezet. Hij is UI-logica (formulieren, selecties, controles) en de kern hoeft hem niet te kennen; een API-client stelt de patch zelf samen.
- **Rapporten en EP-Online-overzicht.** Het BENG-rapport, het rekenrapport en `ep-online-gegevensoverzicht.json` maakt de app zelf, uit de uitkomst van `calculate_project_performance`. De API levert die uitkomst; opmaak en dossier blijven in de client.
- **Gedetailleerde uitvoer.** Komt er een kerntrace (tussenwaarden per maand) bij, dan is dat één nieuwe regel in het register: een route en een MCP-tool tegelijk.
- **Diagnose- en conceptroutes** (`…-draft/diagnose`, `reference/…`) zijn onderdelen die de kern ook los kan doorrekenen, voor controle en referentievergelijkingen.

## Voorbeelden

Versie en gezondheid:

```
curl -s http://127.0.0.1:3007/v1/version
curl -s http://127.0.0.1:3007/health
```

Een voorbeeldproject doorrekenen. Het Python-fragment laat de `null`-leden weg en zet het project in de envelop `{"project": …}`:

```
python3 -c '
import json, sys
def strip(v):
    if isinstance(v, dict): return {k: strip(x) for k, x in v.items() if x is not None}
    if isinstance(v, list): return [strip(x) for x in v]
    return v
print(json.dumps({"project": strip(json.load(open(sys.argv[1])))}))
' training-data/nta8800-example-terraced-dwelling.json \
  | curl -s -X POST http://127.0.0.1:3007/v1/nta8800/project/performance \
      -H 'content-type: application/json' -d @-
```

Het antwoord bevat onder meer `status`, `performance.primaryFossilIndicatorKwhPerM2Year` (BENG 2) en `performance.indicativeLabelClass`.

Een basisopname (ISSO 82.1):

```
python3 -c 'import json, sys; print(json.dumps({"survey": json.load(open(sys.argv[1]))}))' \
  training-data/nta8800-opname-1930-terraced.json \
  | curl -s -X POST http://127.0.0.1:3007/v1/nta8800/opname/residential \
      -H 'content-type: application/json' -d @-
```

Herlabelen: origineel tegen huidig project:

```
curl -s -X POST http://127.0.0.1:3007/v1/nta8800/relabel/assess \
  -H 'content-type: application/json' \
  -d '{"original": {...}, "current": {...}}'
```

## Tests

`crates/nta8800-service/tests/api.rs` controleert:
- elke route;
- het OpenAPI-document tegen het register;
- de foutenvelop bij ongeldige JSON, verkeerde content-type, verkeerde vorm, onbekende route, verkeerde methode en een te grote body;
- de grens op het aantal JSON-waarden, onbekende leden, herlabelen en hash met iets anders dan een project, de editie van een referentiegeval, de reikwijdte in `capabilities`, de velden van `energy-by-service`, de hostcontrole en de 422-regels in het OpenAPI-document;
- CORS, met `Vary: Origin`;
- versie en gezondheid;
- beide voorbeeldprojecten via alle projectroutes;
- drie basisopnames;
- een project zonder NTA-invoer (422 met gaten).

`crates/nta8800-service/tests/memory_bound.rs` meet met een tellende allocator wat een ingelezen body per JSON-waarde kost, en controleert dat een bewerking de body niet kopieert.

## Build

De release-binaries van 5 oktober 2026, gebouwd uit deze bronstand met `cargo build --release --bin api --bin mcp`:

| Binary | SHA-256 |
|---|---|
| `api` | `b32a44e9147de5fed99c80742bd7368395f32d009ee2feaa445c91fad167fd04` |
| `mcp` | `5afc2f119f46fc68c8dc6885e8bb95b5df2a99a501c868f79aae7f26fc4d7f1c` |

Het `mcp`-binary bevat de handleiding en de fixtures als resources; elke wijziging daarin geeft dus een andere hash.

De hash hangt af van compiler, platform en bronstand; leg hem per release vast (BRL 9501 §4.3).
