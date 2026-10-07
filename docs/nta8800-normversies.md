# NTA 8800-uitgaven in de rekenkern

De rekenkern rekent standaard volgens de aangewezen uitgave, **NTA 8800:2025+C1:2026**. Een project kan ook een oudere uitgave kiezen. Dat is bedoeld om oude berekeningen na te rekenen en uitkomsten te vergelijken. Een berekening in een oudere uitgave is **nooit registreerbaar**.

Paginaverwijzingen gaan naar de gelicentieerde PDF's: NTA 8800:2020+A1:2020 (de geconsolideerde uitgave), NTA 8800:2022, NTA 8800:2023, NTA 8800:2024 (met het interpretatiedocument INT-V1:2024) en NTA 8800:2025+C1:2026. Voor 2022 en 2023 bestaat geen interpretatiedocument; in de bronmap staan daarvoor alleen wijzigingsdocumenten van ISSO en BRL. Deze repository bevat geen normtekst, alleen paragraaf-, formule-, tabel- en paginanummers.

## Kiezen van een uitgave

- **Projectbestand:** `ntaCalculation.normVersion`, met `"2025+C1"` (standaard), `"2024"`, `"2023"`, `"2022"` of `"2020+A1"`. Ontbreekt het veld, dan rekent de kern in 2025+C1. De invoervingerafdruk van een project zonder het veld verandert daardoor niet.
- **App:** het NTA-invoerformulier heeft in het blok *Algemeen* het veld *Uitgave NTA 8800*. Bij een oudere uitgave tonen het rekenpaneel, het NTA-rekenrapport en de statusbalk "niet voor registratie".
- **API en MCP:** elke `POST`-bewerking neemt het optionele verzoeklid `normVersion`; zie [Uitgave per route](#uitgave-per-route). Zonder dat lid lezen de projectbewerkingen het veld uit het project. `GET /v1/version` (`get_version`) geeft onder `supportedNormVersions` de bekende uitgaven, met `implemented`, `registrationEligible`, `default` en de aanwijzingsperiode.
- **Alle bekende uitgaven rekenen.** `edition_not_implemented` blijft bestaan voor een uitgave zonder profiel, maar geen bekende uitgave valt daar nu onder. Een onbekende waarde is een invoergat.

## Uitgave per route

Elke ingang van de kern rekent in een gekozen uitgave en zet die op de uitkomst (`normVersion`, `targetNormVersion`; waar een registratie mogelijk is ook `registrationEligible`).

| Route | Waar de uitgave staat | Uitkomst in een oudere uitgave |
| --- | --- | --- |
| Project (`assess_project_performance`, registratie, labeldata, energie per dienst) | `ntaCalculation.normVersion` | `calculated_legacy_edition`, `legacy_edition_not_registrable` |
| Gebouw (`assess_building_performance`) | `normVersion` van de gebouwinvoer | `registrationEligible: false` (in de service ook status `calculated_legacy_edition`) |
| Basisopname woning/utiliteit (`assess_residential_survey`, `assess_utility_survey`) | `normVersion` van de opname; de app neemt de uitgave van het project | `calculated_legacy_edition`, waarschuwing `survey_protocol_edition_differs` |
| Maatwerkadvies (`assess_maatwerkadvies`) | de basissituatie: `ntaCalculation.normVersion` van het project of `normVersion` van de gebouwinvoer | `calculated_legacy_edition` als alle varianten rekenen; een maatregel die de uitgave wijzigt, is ongeldig (`measure_changes_norm_version`) |
| Herlabelen (`assess_relabel`) | `ntaCalculation.normVersion` van het **oorspronkelijke** project | een andere uitgave in het huidige project is *niet toegestaan* (6b-achtig, eigen cluster) |
| Constructies en diagnoses (API, MCP en desktop-app) | verzoeklid `normVersion`; de kern rekent met die uitgave actief. De diagnosepanelen en de constructie-editor sturen de uitgave van het project mee | `calculated_unverified` wordt `calculated_legacy_edition` |
| Referentiegevallen | het geval zelf (alleen 2025+C1) | `normVersion` ≠ `2025+C1` geeft `norm_version_not_applicable` |
| Label-invoer-hash | — (hangt niet van de uitgave af) | ongewijzigd |

**Basisopname in een oudere uitgave.** Het opnameprotocol (ISSO 82.1 en 75.1, 7e druk 2025) hoort bij NTA 8800:2025+C1. Een oudere uitgave is in de kern alleen toegestaan om te **vergelijken**: de kern past dezelfde ISSO-opnameregels en forfaits toe en rekent het gebouw daarna in de gekozen uitgave. De uitkomst krijgt de waarschuwing `survey_protocol_edition_differs`, de status `calculated_legacy_edition` en `registrationEligible: false`. Een oude opname die volgens een eerdere druk van ISSO 82.1/75.1 is gedaan, wordt dus niet nagebootst; de kern kent alleen de 7e druk.

**Herlabelen.** Een herlabeling gebruikt de methodiek en de softwareversie van de oorspronkelijke opname (BRL 9500-W 2026 §4.2.4 p. 23–24; BRL 9500-U 2026 p. 19–20). Daarom:
- vergelijkt `assess_relabel` in de uitgave van het oorspronkelijke project (`normVersion`) en meldt ook die van het huidige project (`currentNormVersion`). Ontbreekt het veld, dan geldt 2025+C1; een ontbrekend en een expliciet `"2025+C1"` zijn gelijk;
- is een andere uitgave in het huidige project een wijziging die niet is toegestaan (`/ntaCalculation/normVersion`);
- weigert de registratiecontrole een herlabeling in een oudere uitgave **niet** met `legacy_edition_not_registrable`, als die uitgave gelijk is aan die van het oorspronkelijke project volgens de eigen hervergelijking van de kern. Elke andere registratie in een oudere uitgave blijft geweigerd. Het projectveld `registrationEligible` blijft `false`: het zegt iets over een gewone registratie in die uitgave.

**Service.** Het verzoeklid `normVersion` wordt in de eigen plek van de invoer geschreven als die ontbreekt (zie de tabel; bij herlabelen in beide projecten). Staat er al een andere uitgave, dan antwoordt de service 400 `norm_version_conflict`. Ontbreekt de plek (bijvoorbeeld een project zonder `ntaCalculation`), dan volgt 400 `norm_version_not_applicable`. De standaarduitgave wordt nooit in de invoer geschreven; zo veranderen invoervingerafdruk en label-invoer-hash niet. Een onbekende waarde geeft 400 `invalid_norm_version`. Een diagnose in een uitgave zonder profiel geeft 422 `edition_not_implemented`.

## Wat er verandert bij een oudere uitgave

| Uitkomst | 2025+C1 | Oudere uitgave |
| --- | --- | --- |
| Status | `calculated_unverified` | `calculated_legacy_edition` |
| `targetNormVersion` | `NTA 8800:2025+C1:2026` | `NTA 8800:2024 met INT-V1:2024`, `NTA 8800:2023` of `NTA 8800:2022` |
| `normVersion`, `registrationEligible` | `2025+C1`, `true` | `2024`, `2023` of `2022`, `false` |
| Registratiecontrole | volgens BRL 9500 | altijd de fout `legacy_edition_not_registrable` |
| Indicatoren die pas in 2025+C1 bestaan (§5.3.3–5.3.5, 5.5.6.1–2, 5.5.7, 5.6.4, 5.9, bijlage AB) | berekend | `null` |

Invoer voor een route die de gekozen uitgave niet kent, wordt geweigerd met `route_not_in_edition` op het pad van die invoer. Voorbeelden voor 2024: `bacsFactor` ≠ 1, een dakrand als belemmering van PV of collectoren, zonwerend glas (`glazingType: solar_control`), vaste lamellen, en `effectiveMassKgPerM2`/`roofAreaM2` van bijlage AA in 2025+C1. Voor 2023 daarnaast: bijlage AA als bewijs van koelcapaciteit, een spuiopening volgens (11.71a) (`area.method: discharge`), en omgekeerd `kitchenPipeDiameter` (tabel 13.2) in 2024 en 2025+C1.

## Hoe de kern de uitgave kiest

`crates/nta8800-core/src/norm_versions/` bevat:
- de enum `NormVersion` (oplopend: 2020+A1, 2022, 2023, 2024, 2025+C1);
- per geïmplementeerde uitgave één `NormProfile` (`v2025.rs`, `v2024.rs`, `v2023.rs`, `v2022.rs`) met de getallen en routekeuzes, elk met paginaverwijzing. De oudere profielen zijn cumulatief: alles waarin 2024 van 2025+C1 afwijkt, geldt ook in 2023, en alles waarin 2023 afwijkt ook in 2022, plus de eigen verschillen hieronder. Invoer die alleen "2023" heette (tabel 13.2-keukenrijen, `emission.edition2023`, `cooling.emission.edition2023`, paneel-`buildYear`, `uninsulatedPipesInUninsulatedShell`), geldt daardoor ook in 2022.

De invoer draagt de uitgave (`NtaCalculationInput.normVersion`, `BuildingPerformanceInput.normVersion`, `ResidentialSurvey`/`UtilitySurvey.normVersion`, de basis van het maatwerkadvies, het oorspronkelijke project bij herlabelen). Elke ingang (`assess_project_performance`, `assess_building_performance`, de opnames, `assess_maatwerkadvies`, `assess_relabel`) zet de uitgave voor de duur van de berekening als **thread-local** (`norm_versions::with_version`). De service doet dat ook voor de constructies en diagnoses. Rekenfuncties lezen `norm_versions::profile()` op het punt waar de uitgaven verschillen.

Dat wijkt bewust af van "alle parameters doorgeven". De kern rekent synchroon op één thread, en de bewaker herstelt de vorige uitgave ook bij een paniek. Zo hoeven niet tientallen functiesignaturen en struct-initialisaties te veranderen voor een getal dat in één uitgave anders is. Andere threads (de service, parallelle tests) rekenen ongestoord in hun eigen uitgave. Een rekenfunctie die zonder `with_version` wordt aangeroepen (een eenheidstest), rekent in 2025+C1. De service draait elk verzoek in een eigen blokkerende taak; een test met parallelle verzoeken in 2024 en 2025+C1 laat zien dat de uitgave niet tussen verzoeken lekt.

## Verschillen 2024 → 2025+C1 die de kern omschakelt

| # | Onderwerp | 2024 (pagina) | 2025+C1 (pagina) | Kern |
| --- | --- | --- | --- | --- |
| 1 | 5.5.8/15.3 f_BACS 1,05 | bestaat niet (p. 87–88) | p. 89–91, 99–101, 676 | `bacsFactor` ≠ 1 → `route_not_in_edition`; projectroute zet 1,0 |
| 2 | 5.14a/b opslagcorrectie | geen (5.10, p. 84–86) | p. 85–87 | `storage_correction_factor` = 0 |
| 3 | Tabel 5.3 K_CO2 el/gas/olie/biomassa/externe warmte | 0,34 / 0,183 / 0,260 / 0,372 / 0,17 (p. 94–95) | 0,268 / 0,218 / 0,326 / 0,104 / 0,09 (p. 96–98) | `NormProfile`; ook koude-forfait K_el/3 |
| 4 | Tabel 5.6 AVI | 0,113 (p. 117) | 0,138 (p. 125) | `SystemCarrier::co2` |
| 5 | Nieuwe indicatoren (5.3.3–5.3.5, 5.5.6.1–2, 5.5.7, 5.6.4, 5.9, AB) | bestaan niet | p. 76–77, 96–99, 110–112, 133, 1148–1163 | `null` |
| 6 | Tabel 9.16 Ψ collectief verwarming + tapwater ≤ 500 m² | 2,0 (p. 292) | 1,0 (p. 310) | `PipeTransmittance::value` |
| 7 | (9.62) f_cor.bron.col | geen term (p. 314–315) | 0,022 / 0,009 (p. 331–332) | `SourceSystem::correction_factor`, gaswarmtepomp |
| 8 | Bronwarmte warmtepomp (5.20) | elke tabelbron ≥ 15 °C, f_P 0,9 of bijlage P, plus Q/EER (p. 93, 323, 346; INT-V1 p. 5) | alleen collectieve bron, < 20 °C f_P;el/23 (p. 72, 90, 362–363) | `SystemHeatPump::source_heat_booked`, `source_factors`, `realisedFrom2013` |
| 9 | 17.3.2 f) dakrand als belemmering | bestaat niet (p. 678–687) | p. 695–708 | `collector_obstruction_factor` → geen waarde; PV-invoer `route_not_in_edition` |
| 10 | 7.6.6.1.2–7.6.6.1.3.4 zonwerend glas, vaste en draaibare lamellen (ook TOjuli-uitzondering 5.7.1) | bestaan niet (p. 179–180, 106) | p. 190–196, 114–115 | invoer `route_not_in_edition` |
| 11 | (13.157)/(13.160) f_gebouw;si;W | /365, hulpenergie zonder f_gebouw (p. 611–612) | /(365·f_gebouw), × f_gebouw (p. 627–628) | `book_generator` |
| 12 | Tabel 13.18 c_W,EU;gen | kolom (p. 614) | lineair geïnterpoleerd (p. 630–631) | `european_profile_correction` |
| 13 | (11.136) gemeten motorrendement | alleen als hoger dan tabel 11.20 (p. 499) | vrije keuze (p. 515) | `nominal_fan_power` |
| 14 | Bijlage AA | SWM, tabel AA.2, juli-maandinstraling 17.2, geen ondergrens (p. 1115–1127) | uurinstraling tabel AA.3, ondergrens 0 kW (p. 1136–1147) | `assess_annex_aa`, `effectiveMassKgPerM2`, `roofAreaM2` |
| 15 | Tabel E.5 F_A;iso cellulose, gespoten vlas | overig 1,30 (p. 783) | 1,00 (p. 803) | `ForfaitMaterial::in_situ_product` |
| 16 | Tabel E.11/E.12 biobased λ, riet | vlas/schapenwol/katoen 0,050, kokos 0,055, stro 0,060, hennep 0,100; riet 0,200 geen isolatie (p. 791) | p. 810 | `ForfaitMaterial::lambda` |
| 17 | (I.3) rieten dak | d/0,2 (p. 814) | d/0,105 (p. 833) | `ForfaitOpaque::calculate` |
| 18 | (W.9) ondergrens noemer boosterwarmtepomp | geen (p. 1103) | ≥ E_ls (p. 1123) | `calculate_booster` |

## Verschillen 2023 → 2024 die de kern omschakelt

Bovenop de achttien punten hierboven. Pagina's links uit NTA 8800:2023, rechts uit NTA 8800:2024.

| # | Onderwerp | 2023 (pagina) | 2024 (pagina) | Kern |
| --- | --- | --- | --- | --- |
| 19 | Tabel 13.2 keuken, binnendiameter ≤ 8 mm of ≤ 10 mm over twee derde van de lengte | eigen rijen, bv. 6–8 m: 0,67 / 0,55 / overig 0,43 (p. 533) | alleen "overig" (p. 527) | invoer `hotWater.emission.kitchenPipeDiameter` (`up_to_8_mm`, `up_to_10_mm`, `other`); `kitchen_emission_for`; buiten 2023 `route_not_in_edition` |
| 20 | Tabel 11.7 τ_argII mei–nov | 0,12 / 0,18 / 0,28 / 0,25 / 0,17 / 0,06 / 0,01 (p. 454) | 0,46 / 0,75 / 0,81 / 0,79 / 0,75 / 0,26 / 0,05 (p. 449) | `ventilative_cooling_flows` |
| 21 | f_argII handmatig / automatisch | 0,5 / 0,9 (p. 466) | 0,35 / 0,50 (p. 461) | `CoolingOperation::factor` |
| 22 | (11.71a) spuiopening uit C_d en C_e | bestaat niet (p. 465) | p. 460–461 | `area.method: discharge` → `route_not_in_edition` |
| 23 | Tabel 11.8 f_τ woonfunctie | 0,80 (p. 460) | min(0,38 + 0,006·A_g; 0,8) (p. 455) | `VentilationFunction::occupancy_factor` |
| 24 | Tabel 17.1 θ_e;argII mei–sep | 14,56 / 15,62 / 16,17 / 16,90 / 15,11 (p. 676) | 16,42 / 16,76 / 17,51 / 18,24 / 16,74 (p. 674) | `ventilative_cooling_flows`, toevoertemperatuur |
| 25 | ΔT_fan verwarming / koeling woning / koeling utiliteit | 1 / 0,7 / 1,5 K (p. 496) | 0,7 / 0,4 / 0,7 K (p. 491) | toevoertemperatuur (11.129/11.130) |
| 26 | Tabel I.1 detail 17 (kozijn dakkapel) | 0,60 / 0,90 (p. 804) | 0,06 / 0,09 (p. 803) | `forfait_psi` |
| 27 | Tabel E.10 houtvezel en cellulose | 0,050 (p. 791) | 0,045 (p. 790) | `ForfaitMaterial::lambda` |
| 28 | (P.25) referentievermogen | Q/8 800 × 3,6 (p. 950) | Q/5 400 × 3,6 (p. 948) | `annex_p` |
| 29 | P.6.5.4.8 rendement geothermie | forfait 20 (p. 960) | met temperatuurcorrectie (p. 958) | `geothermal_efficiency` |
| 30 | Biomassagrens bmA | 100 kW (p. 90–93) | 500 kW (p. 92–95) | `biomass_class_limit_kw`; het invoerveld `biomassAbove500Kw` betekent "boven de grens van de uitgave" |
| 31 | Bijlage AA als bewijs koelcapaciteit (5.7.1) | bestaat niet; actieve koeling met voldoende capaciteit geldt als voldaan (p. 103–104) | p. 106, 1115–1127 | `capacity.method: annex_aa` → `route_not_in_edition` |
| 32 | Renovatiestandaard (tabel 5.7) | bestaat niet (p. 70–72) | p. 73–74 | `null` (hoofdstuk-5-indicatoren zijn in beide oudere uitgaven al `null`) |
| 33 | Tabel 14.3 kolom "led vanaf 2017" | één kolom (p. 648) | 16 / 12 W/m² (p. 646) | `ledFrom2017: true` → `route_not_in_edition` |
| 34 | Roeden in U_w (8.17)/(8.18) | geen term (p. 214–218) | p. 219–222 | `glazingBars` → `route_not_in_edition` |
| 35 | f_sh;with woningen (7.6.6.1.4) | tabel 7.7 ook op de warmtebalans; automatische zonwering van woningen tabel 7.7 (p. 179–181) | 0 op de warmtebalans; automatisch tabel 7.9 (p. 181, 186) | `dwelling_shading_heating_off`; `automatic` in een woning → `route_not_in_edition` |
| 36 | Restwarmte f_Pren (5.47)/(5.55) | 1 − f_rw;aux;spec·f_P;del;el = 0,8985 (p. 116, 121, 959) | 1 − f_rw;aux;spec = 0,93 (p. 119, 124) | `residual_heat_pren_primary` |
| 37 | Lichtkoepels U_rc → U_C (8.2.2.1 e) | bestaat niet (p. 207) | p. 210 | element `rooflight` → `route_not_in_edition` |
| 38 | Paneel-U naar bouwjaar (I.13/I.14) | vanaf 1965 per bouwjaarklasse (p. 818–820) | alleen I.11/I.12 (p. 817–818) | `buildYear` op `forfait_panel` (alleen 2023; verplicht zonder bekende dikte → `panel_build_year_required`); de basisopname geeft het bouwjaar door |
| 39 | NEN 1087-factor zonder roosterspecificatie (11.71) | 0,5 (p. 466) | 0,3 (p. 461) | `screenUnspecified` op `opening_angle` (alle edities) |
| 40 | A_w;cros (11.77) | alleen naar oriëntatie (p. 469) | dakopening β < 60° in elke sector (11.77a/b, p. 464) | `cross_area_roof_all_sectors` |
| 41 | Distributie verwarming 9.4 | L_zi ≠ 0 ook voor alleen-verwarming, t_H;op uit tabel 9.15, geen 65 °C en Δϑ_H,ontw bij afleversets (p. 290–294, 304) | L_zi = 0, volle maand, ≥ 65 °C (p. 284–290, 300) | `distribution_2024_rules` |
| 42 | Tabel 9.16 gecombineerde collectieve leidingen | geen eigen rijen (p. 296) | p. 292 | `table_9_16_combined_rows` |
| 43 | f_H;dis;rbl 0,5 in ongeïsoleerde schil (9.4.3) | p. 299 | bestaat niet (leidingen tellen als onverwarmd, p. 285) | `uninsulatedPipesInUninsulatedShell` (alleen 2023) |
| 44 | Tabel 13.4 onbekende diameter | rijen "klein"/"overig" (p. 542) | tabel 13.29 en 35/80 mm (p. 537–538) | `table_13_4_system_psi`; "klein" = ten hoogste 500 m² aangesloten (interpretatie) |
| 45 | Serieschakeling tapwater 13.141a–d en 13.8.4.10 | bestaat niet | p. 595, 638 | `series` en `heat_pump_series` → `route_not_in_edition` |
| 46 | Bronwarmte warmtepomp | alleen bron ≥ 20 °C als externe warmte (p. 326, 343); EER_bron blijft (p. 349) | elke bron ≥ 15 °C (p. 323) | `HeatPumpSourceRoute::From20C2023` |
| 47 | Afgifte verwarming 9.3 | tabellen 9.2–9.10, (9.17)–(9.20) (p. 273–285) | tabellen 9.2–9.4 (p. 279–280) | `emission.edition2023` (alleen 2023); zonder die invoer de onbekende waarden van 2023 |
| 48 | Afgifte koeling 10.3.3 | tabellen 10.2–10.5 (p. 360–364) | tabellen 10.35/10.4/10.5 (p. 359) | `cooling.emission.edition2023` (alleen 2023); zonder die invoer afgeleid van de 2024-velden |

Gelijk in 2023 en 2024, dus geen schakelpunt: K_CO2 (tabel 5.3, p. 92–93; AVI tabel 5.6, p. 114), f_P el 1,45 en gas 1,0 (tabel 5.2, p. 89), λequi;ntr 0,045 (p. 814–815; alleen 2022 had 0,06), (I.3) d/0,2 (p. 815), tabel 13.18 (p. 617) en EER_bron 23/16 voor een WKO-bron (9.6.8.1.1.2.3, p. 349).

## Verschillen 2022 → 2023 die de kern omschakelt

Bovenop de punten 1–48. Pagina's links uit NTA 8800:2022, rechts uit NTA 8800:2023. Het verschil is opnieuw uit beide PDF's afgeleid (genormaliseerde tekstvergelijking plus symboolvergelijking).

| # | Onderwerp | 2022 (pagina) | 2023 (pagina) | Kern |
| --- | --- | --- | --- | --- |
| 49 | Tabellen 9.27/9.29 warmtepompen | kolommen tot 55 °C, geen bronrijen ≥ 15 °C, 9.27 voor alle woningen; boven 55 °C bijlage Q (p. 313–317) | tot 70 °C, bronrijen 15–20/20–40/≥ 40 °C, grens 25 kW/collectief (p. 319–324) | `heat_pump_tables_2022`: `designSupplyTemperatureC` > 55 en `source` `collective*` → `route_not_in_edition`; geen `table_scope_capacity_mismatch`; grondwater zonder temperatuurgrens |
| 50 | Bronwarmte Q_HD;hp;in;bron (5.20, 9.84, 9.6.8.1.1.2.3) | bestaat niet (p. 94–96, 335, 341) | p. 96–98, 343, 349 | `HeatPumpSourceRoute::None2022` |
| 51 | Tabel P.5 | tot 55 °C, geen bronrijen (p. 941) | tot 75 °C met bronrijen (p. 955) | `efficiency.source`/`supplyTemperatureC` → `route_not_in_edition` |
| 52 | Flexmodus (prijsplafond) | bestaat niet | 5.8, P.6.5.4.11 (p. 111, 963) | `electric_flex` → `route_not_in_edition` |
| 53 | Biomassagrens bmA | geen kW-grens, "valt onder het Activiteitenbesluit" (p. 88–92) | 100 kW (p. 90–94) | `biomass_threshold_kw` = ∞; het veld `biomassAbove500Kw` betekent in 2022 "valt onder het Activiteitenbesluit" |
| 54 | (9.58) bijgeplaatste preferente opwekker | Σ Φ / Φ_H;tot (p. 305) | Σ Φ · f_gebouw;si;H / Φ_H;tot (p. 309) | `preference_beta_building_share` |
| 55 | Tabel 9.14 ontwerptemperatuurklassen | zonder 60/50 en 70/60 (p. 290) | met (p. 294) | `designTemperatureClass` `60_50`/`70_60` → `route_not_in_edition` |
| 56 | Ventilatorvermogen getest volgens NEN-EN 16430 | bestaat niet (p. 282) | p. 286 | `emission.fans.testedPowerW` → `route_not_in_edition` |
| 57 | Tabel 7.5 "onbekende kleur" | bestaat niet (p. 176) | p. 180 | `colour: unknown` → `route_not_in_edition` |
| 58 | Tabel 7.10 specifieke interne warmtecapaciteit | naar massa per m² (< 250, 250–500, 500–750, > 750 kg/m²) (p. 181–182) | naar vloer- en wandtype, tabellen 7.10–7.12 (p. 185–186) | `thermalMass.massKgPerM2` (alleen 2022); zonder die invoer de vloer/wandklassen (zelfde D_m-waarden; interpretatie) |
| 59 | (8.47) hoogte h boven maaiveld | werkelijke hoogte (p. 236) | vast 0,125 m (p. 240) | `wallHeightAboveGroundM` op kruipruimte/onverwarmde kelder: verplicht in 2022 (`ground_floor_wall_height_required`), daarna `route_not_in_edition` |
| 60 | Tabel E.5 minerale-wolvlokken F_A;iso | 1,05 (p. 775) | 1,00 (p. 785) | `mineral_wool_flakes_ageing` |
| 61 | (I.2) λ_equi;ntr | 0,06 (p. 805) | 0,045 of een bekende hogere λ (p. 814) | `knownLambdaEquivalent` → `route_not_in_edition` |
| 62 | Zwembad in (11.57) | bestaat niet (p. 451) | q_usi;spec sport × 2 (p. 459) | `swimmingPool` → `route_not_in_edition` |
| 63 | η_hr volgens NEN-EN 13053 | geen rij (p. 482) | p. 490–491 | `standard: en13053` → `route_not_in_edition` |
| 64 | f_sto;dis;ls = 1,5 elektroboiler met geïsoleerde leiding | p. 550 | vervallen (p. 557–558) | `electricBoilerInsulatedPipe` (alleen 2022) |
| 65 | ϑ_sto;amb met ventilatieretourluchtwarmtepomp (13.69a/13.137a) | bestaat niet (p. 557, 584) | p. 565, 592 | `storage_ambient_exhaust_air` |
| 66 | C_W;mixed air (13.153b) | bestaat niet (p. 600–602) | p. 609–611 | `mixedAir` → `route_not_in_edition` |
| 67a | Tabel 9.28 beproeving voor de hoge-COP-rij | NEN-EN 14511-2, gedateerd 2007 (p. 14, 316) | NEN-EN 14511-2:2022 (p. 322) | `heat_pump_high_test_standard`; een andere `testStandardEdition` → `high_test_standard_invalid` |
| 67 | Tabel 14.4 onderhoudsfactor MF | 0,8 lineair TL / 0,7 led L80 met nieuwwaardecompensatie, F_C = 1 − ½(1 − MF) (p. 639) | MF = 1 (p. 649) | `lightingZones[].constantIlluminance` (alleen 2022) |

Gelijk in 2022 en 2023, dus geen schakelpunt: tabellen 5.2/5.3 behalve flexmodus en biomassa, (P.25) met 8 800 (2022 p. 937; de deler 4 000 is die van 2020+A1, punt 70), tabel I.1 details 6, 7 en 17, de paneeltabellen I.13/I.14, de afgiftetabellen 9.2–9.10 en 10.2–10.5, tabel 11.5 (de rijkeuze bij een onbekend roostertype is invoer) en de bijlagen S–Z.

**Acceptatie met een openbaar rapport.** Openbaar rapport B (rijwoning, Uniec 3.1.6.2, berekend 22-03-2023) valt in de periode van 2022 en rekent nu in 2022: 52,96 / 29,10 / 62,5, gelijk aan 2023 (zie [de vergelijking](nta8800-vergelijking-openbare-rapporten.md#a-b-en-d-in-hun-eigen-editie-7-oktober-2026)). Het woongebouw met 28 appartementen in Schagen (r1) is niet nagebouwd: het vraagt een meerzonige schematisering van 28 woningen met veel aannames. Een tweede 2022-rapport (vrijstaande woning, 31-01-2023) is gevonden maar niet nagebouwd; de reden staat in de vergelijking.

Geen invoer in de kern, dus niet omgeschakeld: de standaard voor woningisolatie (2023 p. 74; de hoofdstuk-5-indicatoren zijn in oudere uitgaven al `null`), gemeenschappelijke ruimten in woongebouwen en kelderkasten tot 4 m² (schematisering), het gecombineerde circulatiesysteem met cv-water (2022 p. 531–541), het keukenvat van 10 l (2023 p. 559, opnameregel) en ramen/deuren met minder dan 65 % glas (2023 p. 815–816).

## Verschillen 2020+A1 → 2022 die de kern omschakelt

Het profiel van 2020+A1 is cumulatief op 2022: alles waarin 2022 van 2023 verschilt (punten 49–67) geldt ook hier. De invoer die alleen 2022 kent (`thermalMass.massKgPerM2`, `wallHeightAboveGroundM`, `electricBoilerInsulatedPipe`, `lightingZones[].constantIlluminance`) en de 2023-invoer gelden dus ook onder 2020+A1. Bron: de geconsolideerde uitgave NTA 8800:2020+A1:2020 tegen NTA 8800:2022, met een tekstvergelijking per pagina. Het afzonderlijke interpretatiedocument bij 2020+A1 is niet verwerkt.

| # | Onderwerp | 2020+A1 | 2022 | Kern |
|---|---|---|---|---|
| 68 | (16.4) piekvermogen PV | K_pk per m² naar beneden afgerond op 5 W/m², oppervlakte zoals opgegeven, geen paneelroute (p. 651) | piekvermogen per paneel afgerond op 5 W, K_pk en oppervlakte op 2 decimalen (p. 656–657) | `pv_kpk_per_m2_floor`; `peakPower.method: panels` → `route_not_in_edition` |
| 69 | (I.2) λ_equi;ntr | 0,045 (p. 796) | 0,06 (p. 805) | `lambda_equi_ntr`; een bekende hogere λ blijft geweigerd |
| 70 | (P.25) deler referentievermogen | 4 000, benuttingsgraad 0,13 (p. 925) | 8 800, benuttingsgraad 0,28 (p. 937) | `reference_power_divisor` |
| 71 | Restwarmte | η = 1, f_P;del;rw 0,1 (tabel 5.5), K_CO2 0,034 (tabel 5.6), f_Pren 0,9 (5.47) (p. 109–113, 933) | f_rw;aux;spec 0,07 met elektrische hulpenergie (p. 945) | `residual_heat_fixed_factors`; `auxiliarySpecific` → `route_not_in_edition` |
| 72 | Kleine systemen bijlage P | geen tabel P.0, geen kleine-systeemwaarde bij P.11, geen forfait 0,009 0 voor koudedistributie (p. 920, 960, 969) | p. 930, 972, 981 | `small_system_forfait_route`: `distribution.method: small_system_forfait`, `network: small_system` en de forfaitaire koude-hulpenergie → `route_not_in_edition` |
| 73 | β van de preferente opwekker onbekend | geen waarde; vermogens vereist (p. 925) | β = 0,5 (p. 936) | `unknown_beta_route`: zonder vermogen `generator_power_required` |
| 74 | (9.85) forfaitaire hulpenergie | één set voor alle toestellen, ook warmtepompen: A 87,6 (vóór 2015 of onbekend) of 13,0 kWh (vanaf 2015), B 0,132, C 1,44/3,6, B_nom 24 kW (p. 334–336) | A 43,8 vanaf 2015; warmtepompen A 43,8, B 0,132, C 0,7, B_nom 3 (p. 338) | `device_aux_a_from_2015_kwh`, `heat_pump_aux_constants` |
| 75 | (11.142) f_systype van E1 | 1,5 op de hele zone (p. 495) | gesplitst naar oppervlakte (11.139–11.141, p. 498–499) | `fan_systype_combined` |
| 76 | Tabellen I.1/I.2 | één Ψ-kolom; detail 14 is 0,70; geen standaard 0,5 voor een positie zonder waarde (p. 786–788) | kolommen A en B, detail 14 0,03/0,13, standaard 0,5 (p. 793–796) | `psi_columns_and_default`, `psi_detail_14`; kolom B en een brug zonder positie → `route_not_in_edition` |
| 77 | Werkelijke leidinglengte (9.4.2.3, 10.4.2.3) | alleen utiliteitsbouw; woningen forfaitair (p. 292, 363) | ook woningen (p. 294, 367) | `residential_actual_pipe_length`: `distributionSystem.actualPipeLengthM` bij een woonfunctie → `route_not_in_edition` |
| 78 | (13.148a) debiet uit een kwaliteitsverklaring | bestaat niet (p. 590) | p. 594 | `declaredFlowM3PerH` → `route_not_in_edition`; de eis uit opmerking 3 vervalt |
| 79 | (11.106a) koudeterugwinning met 100 % bypass | bestaat niet (p. 476) | p. 479 | `bypass.coldRecoveryEvidence` → `route_not_in_edition` |

**Punt 74, bouwjaar warmtepomp.** De forfaitaire warmtepompinvoer heeft een optioneel `installationYear` met `installationYearReference` (dezelfde controles als bij de gasketel: 1900–2026, bron verplicht). Onder 2020+A1 geeft een bouwjaar vanaf 2015 A = 13,0 kWh, anders of zonder bouwjaar A = 87,6 kWh ("vóór 2015 of onbekend", 2020 p. 336). Vanaf 2022 hebben warmtepompen eigen constanten zonder bouwjaar (2022 p. 338; 2025+C1 p. 360); daar verandert het bouwjaar niets. Het formulier vraagt het bouwjaar alleen onder 2020+A1 en biedt een achtergebleven waarde onder een andere uitgave ter verwijdering aan. Een gasketel met `installationYear` vanaf 2015 krijgt onder 2020+A1 13,0 kWh.

**Niet omgeschakeld (geen invoer of geen route in de kern):**
- de indicator E_wePRenTot (2022 p. 72) bestaat in 2020+A1 niet; de kern geeft hem alleen in het hoofdstuk-5-blok van 2025+C1, dat in oudere uitgaven `null` is;
- de afrondingsregels van bijlage C (Rc op vier decimalen afgekapt in 2020+A1, p. 742; twee decimalen rekenkundig in 2022, p. 747), de tabellen C.3/C.4 voor sterk geventileerde spouwen (2020 p. 748–750) en de λ-afronding van tabel E.14 (2022 p. 784): de constructie-invoer van de kern levert Rc al als gegeven of rekent niet met deze rijen;
- de belemmeringsregels voor koeling bij meerdere belemmeringen (2022 p. 677–678): de keuze van de methode is invoer;
- de boosterwarmtepomp met zonneboiler (0,55 × Q_W;sol;us, 2022 p. 561–576): de kern heeft geen invoer voor die combinatie;
- de aanwezigheidsfactor F_o;D voor kantoortuinen groter dan 30 m² (2022 p. 640): de regel is in 2022 verduidelijkt; de keuze `largeOfficeGroup` is invoer;
- voorraadvaten groter dan 2 000 l (2022 p. 549): 2020+A1 noemt alleen vaten tot 2 000 l, zonder andere route;
- de tapwaterwarmtepomp met "type vóór 2021" (2020 p. 584–594) en de klasse-eis bij warmtepompen (2022 p. 607): de keuze van de methode is invoer;
- de forfaitaire Ψ-voorwaarden (Rc ≥ 4,5 in 2020+A1, ≥ 4,7 in 2022, detail 1): de adviseur toetst de voorwaarde.

**Acceptatie met openbare rapporten.** Twee rapporten uit de periode van 2020+A1 rekenen in deze editie (zie [de vergelijking](nta8800-vergelijking-openbare-rapporten.md#a-b-en-d-in-hun-eigen-editie-7-oktober-2026)):
- A (vrijstaand, plat dak, Uniec 3.0.16): 94,00 / 36,06 / 74,4 tegen 92,99 / 25,19 / 80,4. De PV-afronding van punt 68 geeft de 2 437 kWh van het rapport; het verschil in BENG 2 is 10.15 en 10.87.
- D (vrijstaande vakantiewoning, Uniec 3.0.10.0): 86,35 / 39,07 / 83,6 tegen 86,72 / 39,19 / 83,5. Tapwater, ventilatoren, PV en, met het installatiejaar 2021, de hulpenergie van de warmtepomp (A = 13,0 kWh, punt 74) zijn gelijk; de overstek op het oostglas is een belemmering per raam (h_o;⊥ 0,25 aangenomen).
- E (vrijstaande woning, Uniec 3.1.5.0, 2022): 88,77 / 31,73 / 72,6 tegen 86,82 / 28,89 / 74,1, met belemmeringen per raam en twee tapwatersystemen; zie [nta8800-vergelijking-openbare-rapporten.md](nta8800-vergelijking-openbare-rapporten.md).

## Besluiten bij de invoering van 2023

Vóór het programmeren nagelopen in de 2023-PDF:
- **Afgifte verwarming** (2023 p. 273–285, punt 47): de forfaitaire tabellen 9.2–9.10 met (9.17)–(9.20), ook voor ruimten hoger dan 4 m. Invoer `emission.edition2023`. Zonder die invoer leidt de kern de 2023-waarden af uit de 2024-velden: eigenschappen die de 2024-invoer niet kent, krijgen de waarde "onbekend" (de hoogste van de categorie, of (9.18a)), de regeling is niet gecertificeerd (Δθctr,1), een onbekend leidingsysteem telt als eenpijps, "individuele ruimtethermostaten" geeft Δθroomaut −0,5, luchtverwarming volgt 9.3.3.4. Tabel 9.3 drukt 2,5 K voor centrale regeling alleen in de kolom Δθctr,2; de kern leest die voor beide kolommen. De productroute (9.13)–(9.15) met CA-waarden is niet als aparte route opgenomen: de gecertificeerde kolom Δθctr,2 (`certifiedControl`) dekt de gangbare toepassing.
- **Afgifte koeling** (2023 p. 360–364, punt 48): tabellen 10.2–10.5. Invoer `cooling.emission.edition2023`. Zonder die invoer: Δϑctr,1, de rij van tabel 10.4 die bij de 2024-keuze hoort (statisch → "per afgiftesysteem zonder groepen", dynamisch of niet van toepassing → 0) en Δϑroomaut +0,5 ("standalone") voor elke regeling per ruimte, 0 voor onbekend.
- **Zomernachtventilatie:** tabel 11.7, f_argII, (11.71a), f_τ en θ_e;argII (punten 20–24), de factor voor NEN 1087 (punt 39) en de sectorindeling van (11.77) (punt 40). De voorwaarde "dak met een hoek van ten hoogste 60°" voor dwarsventilatie staat in beide uitgaven gelijk (2023 p. 467, 2024 p. 460); het verschil zit in (11.77a/b).
- **Tabel 13.2 met de rijen ≤ 8 en ≤ 10 mm:** omgeschakeld, met de twee-derde-regel als omschrijving van de invoer (punt 19). Zonder invoer geldt de rij "overig of onbekend", net als in 2024.
- **Biomassagrens:** 100 kW in 2023 (punt 30).
- **λequi;ntr:** 0,045 in 2023 en 2024, geen schakelpunt.
- **Deler in bijlage P:** 8 800 in 2023 (punt 28).

## Niet omgeschakeld in 2023

Alle eerder hier genoemde punten zijn schakelpunten geworden (35–48). Twee punten vragen geen schakelpunt, omdat de kern de 2024-regel niet als eigen route heeft:
- **Terugvalwaarde U_fr uit tabel 8.3** (2024 p. 227, niet in 2023 p. 221–222): de kern rekent U_fr nooit zelf uit tabel 8.3; `frameUWPerM2K` is altijd opgegeven met een bron. Onder 2023 hoort die bron NEN-EN-ISO 10077-2 te zijn. De route `frame_table` (U_W uit tabel 8.3 als geheel) bestaat in beide uitgaven.
- **L = 0 bij een gecombineerd circulatiesysteem voor tapwater en verwarming** (2024 p. 533, niet in 2023 p. 539): de kern heeft geen invoer voor zo'n gecombineerd circulatiesysteem; de circulatieleiding wordt in beide uitgaven volledig gerekend.

## Geen schakelpunt: rechtgezet door INT-V1:2024

Het interpretatiedocument bij NTA 8800:2024 (INT-V1:2024) brengt deze punten al in lijn met 2025+C1. Daarom rekent de kern hier in beide uitgaven hetzelfde:
- **Tabel 9.4** (Δθctr + Δθroomaut): de 2024-tabel (p. 280) gaf 2 / −0,5 / −1 K. INT-V1 p. 4 corrigeert dat naar 2,5 / 2,0 / 1,5 / 2,5 K, gelijk aan 2025+C1 p. 297.
- **Afgifteverlies (9.16)/(9.12a)** met θe;comb: dit ontbrak in de 2024-tekst (p. 280–281). INT-V1 p. 5 voegt het weer toe, gelijk aan 2025+C1 p. 297–298.
- **Collectief buffervat** (9.2.3.5): INT-V1 p. 3–4 schrijft de 2025-werkwijze voor.
- **Laatste term van (13.185) en (13.96):** INT-V1 p. 5 schrapt die, net als 2025+C1 p. 653.
- **θH,max in (13.141b):** INT-V1 p. 5 definieert die, net als 2025+C1 p. 611.

## Interpretaties

- **Tabel 13.4 in 2023 (p. 542):** de rij "klein" heeft geen definitie. De kern leest "klein" als een systeem met ten hoogste 500 m² aangesloten gebruiksoppervlakte, de grens die 2024 (p. 538) voor utiliteitsgebouwen gebruikt.
- **Tabel 13.18 in 2024 (p. 614):** de tabel geeft geen interpolatieregel. De kern neemt de kolom van de grootste tabelwaarde die niet groter is dan de jaarlijkse tapwarmtevraag. 2025+C1 (p. 630–631) interpoleert lineair.
- **Bronwarmte in 2024:** 9.6.3.1.3 (p. 323, aangepast door INT-V1 p. 5) telt de bronwarmte van elke warmtepomp uit tabel 9.27/9.29 met een bron van ten minste 15 °C mee in (5.20). De kern leest dat af aan de bronrij (15–20, 20–40, ≥ 40 °C). Voetnoot e van tabel 9.27/9.29 (p. 316–317, 320–321), die een bron vanaf 20 °C als externe warmte behandelt, past daarbinnen. De bronwarmte krijgt f_P;del 0,9 van tabel 5.2 (p. 93) en K_CO2 0,17 van tabel 5.3 (p. 95), of de waarden uit bijlage P. Voor een (grond)water- of aquiferbron van 15–20 °C komt daar de hulpenergie van het bronsysteem bij: Q/EER, met EER 23 voor een bron vanaf 2013 en anders 16 (9.6.8.1.1.2.3, p. 346). Het invoerveld is `externalSupply.collectiveHeatPumpSource.realisedFrom2013`; zonder dat veld geldt 16.
- **Bijlage AA in 2024:**
  - SWM (stap 1, p. 1116) is invoer (`effectiveMassKgPerM2`), via tabel AA.1 of formule (AA.1).
  - t_max (stap 2, p. 1117) volgt tabel AA.2 per vertrek en voor de zone. Tussenliggende oriëntaties worden lineair geïnterpoleerd. Gelijke grootste glasoppervlakken krijgen het gemiddelde.
  - Een vertrek met alleen noordglas krijgt t_max van "horizontaal".
  - θe tussen hele uren wordt lineair geïnterpoleerd.
  - De optionele verlaging van de opwekkereis (AA.3.2.3, p. 1125) past de kern niet toe. Dat is aan de veilige kant.
  - (AA.10) deelt in de 2024-tekst door A_g;zi, maar de legenda noemt A_g;vr;zi,j. De kern volgt de legenda, net als 2025+C1 (AA.9).
- **Opslagcorrectie (5.14a/b)** bestaat niet in 2024. Een opgegeven batterij geeft daar geen correctie en ook geen melding.

## Niet omgeschakeld (geen invoer of geen route in de kern)

- **Tabel 7.10 voetnoot c en tabel 7.11** (2024 p. 188–189): de kolomkeuze is invoer van de adviseur.
- **P.3 item 6 / P.5** (2024 p. 916, 921): de regeneratie en de ruimere lijst van LT-bronnen hebben geen eigen invoer in de kern.
- **W.3-plafond** (2025+C1 p. 1126): de formule geeft bij kleinere hoeveelheden al een lagere COP.
- **Biomassagrens:** 500 kW in 2024 en 2025+C1 (2024 p. 92–95); 100 kW in 2023 (punt 30).
- **Bijlage AB** en de nieuwe indicatoren: alleen 2025+C1; in 2024 zijn ze `null`.

## Tests

- `crates/nta8800-core/src/norm_versions/tests.rs`: standaarduitgave, serde, het herstellen van de thread-local (ook na een paniek) en de profielwaarden met paginaverwijzing.
- Per schakelpunt een eenheidstest in de module van het schakelpunt, met de paginaverwijzing in de test.
- `crates/nta8800-core/tests/norm_versions.rs`: de projectroute (status, vingerafdruk, registratie, niet-geïmplementeerde en onbekende uitgaven) en verschiltests van de voorbeeldprojecten in 2024 tegen 2025+C1 en 2023 tegen 2024. Daarin verandert alleen wat de lijsten hierboven voorspellen: in 2023 tegen 2024 verandert de woning niet en het kantoor alleen in TOjuli (ΔT_fan, punt 25). Het voorbeeldkantoor gebruikt de ledkolom van tabel 14.3; die invoer wordt in 2023 geweigerd (punt 33), dus de verschiltest zet hem in beide uitgaven op "overig". De invoer `kitchenPipeDiameter` wordt buiten 2023 geweigerd.
- `crates/nta8800-core/src/norm_versions/tests.rs` (`switch_points_2023`), `ventilation.rs`, `annex_p.rs` en `tojuli.rs`: per schakelpunt van 2023 de waarde met de pagina in beide uitgaven.
- `crates/nta8800-core/src/norm_versions/tests.rs` (`switch_points_2022`), `forfait_heat_pump_draft.rs`, `annex_p.rs`, `ground.rs`, `forfait_envelope.rs`, `lighting.rs` en `tests/norm_versions.rs` (`edition_2022_*`, `example_projects_2022_*`, `thermal_mass_by_kg_per_m2_is_a_2022_route`): de schakelpunten van 2022. Op de voorbeeldprojecten verandert in 2022 tegen 2023 niets: geen van beide heeft invoer op een route die verschilt.
- `crates/nta8800-core/src/norm_versions/tests.rs` (`switch_points_2020`), `annex_p.rs` (restwarmte) en `tests/norm_versions.rs` (`edition_2020a1_*`, `example_projects_2020a1_*`): de schakelpunten van 2020+A1. De verschiltest zet de PV-paneelroute van de voorbeelden om naar een opgegeven K_pk van 200 W/m² met hetzelfde piekvermogen. Tegen 2022 veranderen in beide voorbeelden alleen de grootheden die de hulpenergie van de warmtepomp volgen (punt 74): primair fossiel, CO2 en het hernieuwbare aandeel. Behoefte, TOjuli en omgevingswarmte blijven gelijk.
- `crates/nta8800-core/tests/public_comparison.rs`: de openbare gevallen B en C in 2023, A en D in 2020+A1 en B in 2022; zie [de vergelijking](nta8800-vergelijking-openbare-rapporten.md).
- `crates/nta8800-service/tests/api.rs`: `supportedNormVersions` en een berekening in 2024 via HTTP.
- Per route 2024 tegen 2025+C1 (verschil, stempel, status): `opname/mod.rs` en `opname/utility.rs` (`*_in_its_edition`), `maatwerkadvies.rs` (`variants_follow_the_base_edition`), `relabel.rs` (`relabel_keeps_the_original_edition`) en `registration.rs` (`relabel_in_the_original_edition_is_not_refused_as_legacy`).
- `crates/nta8800-service/src/lib.rs`: `every_route_kind_takes_and_stamps_the_edition`, `every_post_operation_documents_norm_version` en de gelijktijdigheidstest `parallel_requests_keep_their_own_edition` (32 parallelle verzoeken, afwisselend 2024 en 2025+C1).
- `src/__tests__/nta-norm-versions.test.tsx`: de keuze in het formulier, de status als "berekend", en de melding in het rapport en de statusbalk.
