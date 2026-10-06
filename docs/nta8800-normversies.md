# NTA 8800-uitgaven in de rekenkern

De rekenkern rekent standaard volgens de aangewezen uitgave, **NTA 8800:2025+C1:2026**. Een project kan ook een oudere uitgave kiezen. Dat is bedoeld om oude berekeningen na te rekenen en uitkomsten te vergelijken. Een berekening in een oudere uitgave is **nooit registreerbaar**.

Paginaverwijzingen gaan naar de gelicentieerde PDF's: NTA 8800:2023, NTA 8800:2024 (met het interpretatiedocument INT-V1:2024) en NTA 8800:2025+C1:2026. Voor 2023 bestaat geen interpretatiedocument; in de bronmap staan voor 2023 alleen wijzigingsdocumenten van ISSO en BRL. Deze repository bevat geen normtekst, alleen paragraaf-, formule-, tabel- en paginanummers.

## Kiezen van een uitgave

- **Projectbestand:** `ntaCalculation.normVersion`, met `"2025+C1"` (standaard), `"2024"` of `"2023"`. Ontbreekt het veld, dan rekent de kern in 2025+C1. De invoervingerafdruk van een project zonder het veld verandert daardoor niet.
- **App:** het NTA-invoerformulier heeft in het blok *Algemeen* het veld *Uitgave NTA 8800*. Bij een oudere uitgave tonen het rekenpaneel, het NTA-rekenrapport en de statusbalk "niet voor registratie".
- **API en MCP:** de projectbewerkingen lezen het veld uit het project. `GET /v1/version` (`get_version`) geeft onder `supportedNormVersions` de bekende uitgaven, met `implemented`, `registrationEligible`, `default` en de aanwijzingsperiode.
- **Bekend, niet geïmplementeerd:** `"2022"` en `"2020+A1"`. De kern weigert die met `edition_not_implemented` (status `invalid`). Een onbekende waarde is een invoergat.

## Wat er verandert bij een oudere uitgave

| Uitkomst | 2025+C1 | Oudere uitgave |
| --- | --- | --- |
| Status | `calculated_unverified` | `calculated_legacy_edition` |
| `targetNormVersion` | `NTA 8800:2025+C1:2026` | `NTA 8800:2024 met INT-V1:2024` of `NTA 8800:2023` |
| `normVersion`, `registrationEligible` | `2025+C1`, `true` | `2024` of `2023`, `false` |
| Registratiecontrole | volgens BRL 9500 | altijd de fout `legacy_edition_not_registrable` |
| Indicatoren die pas in 2025+C1 bestaan (§5.3.3–5.3.5, 5.5.6.1–2, 5.5.7, 5.6.4, 5.9, bijlage AB) | berekend | `null` |

Invoer voor een route die de gekozen uitgave niet kent, wordt geweigerd met `route_not_in_edition` op het pad van die invoer. Voorbeelden voor 2024: `bacsFactor` ≠ 1, een dakrand als belemmering van PV of collectoren, zonwerend glas (`glazingType: solar_control`), vaste lamellen, en `effectiveMassKgPerM2`/`roofAreaM2` van bijlage AA in 2025+C1. Voor 2023 daarnaast: bijlage AA als bewijs van koelcapaciteit, een spuiopening volgens (11.71a) (`area.method: discharge`), en omgekeerd `kitchenPipeDiameter` (tabel 13.2) in 2024 en 2025+C1.

## Hoe de kern de uitgave kiest

`crates/nta8800-core/src/norm_versions/` bevat:
- de enum `NormVersion` (oplopend: 2020+A1, 2022, 2023, 2024, 2025+C1);
- per geïmplementeerde uitgave één `NormProfile` (`v2025.rs`, `v2024.rs`, `v2023.rs`) met de getallen en routekeuzes, elk met paginaverwijzing. Het profiel van 2023 is cumulatief: alles waarin 2024 van 2025+C1 afwijkt, geldt ook in 2023, plus de eigen verschillen hieronder.

De invoer draagt de uitgave (`NtaCalculationInput.normVersion`, `BuildingPerformanceInput.normVersion`). `assess_building_performance` zet de uitgave voor de duur van de berekening als **thread-local** (`norm_versions::with_version`). Rekenfuncties lezen `norm_versions::profile()` op het punt waar de uitgaven verschillen.

Dat wijkt bewust af van "alle parameters doorgeven". De kern rekent synchroon op één thread, en de bewaker herstelt de vorige uitgave ook bij een paniek. Zo hoeven niet tientallen functiesignaturen en struct-initialisaties te veranderen voor een getal dat in één uitgave anders is. Andere threads (de service, parallelle tests) rekenen ongestoord in hun eigen uitgave. Een rekenfunctie die buiten `assess_building_performance` wordt aangeroepen (een conceptdiagnose of eenheidstest), rekent in 2025+C1.

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

Gelijk in 2023 en 2024, dus geen schakelpunt: K_CO2 (tabel 5.3, p. 92–93; AVI tabel 5.6, p. 114), f_P el 1,45 en gas 1,0 (tabel 5.2, p. 89), λequi;ntr 0,045 (p. 814–815; alleen 2022 had 0,06), (I.3) d/0,2 (p. 815), tabel 13.18 (p. 617) en EER_bron 23/16 voor een WKO-bron (9.6.8.1.1.2.3, p. 349).

## Besluiten bij de invoering van 2023

Vóór het programmeren nagelopen in de 2023-PDF:
- **Oude tabellen afgifte verwarming en productroute** (2023 p. 273–286 tegen 2024 p. 278–280): niet omgeschakeld. De kern rekent ook in 2023 met de drie samengevatte tabellen van 2024. De oude route bestaat uit losse correcties (Δθstr, Δθctr, Δθemb, Δθim, Δθroomaut, tabellen voor hoge ruimten) en een productroute (9.13)–(9.15). Daarvoor ontbreekt invoer in de kern. Dit is een bekende afwijking van een 2023-berekening.
- **Oude route afgifte koeling** (tabellen 10.2–10.5 en de productroute, 2023 p. 360–364 tegen tabel 10.35, 2024 p. 358–359): niet omgeschakeld, om dezelfde reden. Het effect op de openbare rapporten is klein (zie de toelichting bij 10.15 in de vergelijking).
- **Zomernachtventilatie:** tabel 11.7, f_argII, (11.71a), f_τ en θ_e;argII zijn omgeschakeld (punten 20–24). Niet omgeschakeld: de factor voor NEN 1087 (0,5 in 2023, 0,3 in 2024) en de voorwaarde β ≥ 60° voor dwarsventilatie, die 2023 niet stelt. Daarvoor heeft de kern geen eigen invoer.
- **Tabel 13.2 met de rijen ≤ 8 en ≤ 10 mm:** omgeschakeld, met de twee-derde-regel als omschrijving van de invoer (punt 19). Zonder invoer geldt de rij "overig of onbekend", net als in 2024.
- **Biomassagrens:** 100 kW in 2023 (punt 30).
- **λequi;ntr:** 0,045 in 2023 en 2024, geen schakelpunt.
- **Deler in bijlage P:** 8 800 in 2023 (punt 28).

## Niet omgeschakeld in 2023 (kern rekent met de 2024-route)

- Afgifte verwarming en koeling: zie de besluiten hierboven.
- Tabel 13.4 met de rijen "klein/overig" in plaats van de diameters van tabel 13.29 (2023 p. 542); de hotfill- en serieregels (13.141a–c, 13.8.4.10) en de regel L = 0 bij gecombineerde circulatie bestaan in 2023 niet.
- Distributie: L_zi ≠ 0 voor alleen-verwarmingsleidingen in verwarmde zones (2023 p. 290), geen volle maand t_H,op (p. 291), geen 65 °C-regel voor afleversets, f_dis;rbl 0,5 voor ongeïsoleerde leidingen in een ongeïsoleerde buitenmuur (p. 299), geen collectief-gecombineerde rijen in tabel 9.16 (p. 296).
- Beschaduwing: f_sh;with wordt in 2023 voor woningen niet op 0 gezet (p. 179–183).
- Bronwarmte: drempel ≥ 20 °C en de bronterm binnen (9.84) (p. 326, 343). De kern volgt de 2024-route van 9.6.3.1.3.
- Restwarmte f_Pren = 1 − f_P;del;rw (p. 116–121).
- Paneel-U volgens bouwjaar (I.13/I.14, p. 818–821); geen terugvalwaarde U_fr uit tabel 8.3 (p. 221); geen omrekening U_rc van lichtkoepels (p. 207). De kern rekent hier met de 2024-route.

## Geen schakelpunt: rechtgezet door INT-V1:2024

Het interpretatiedocument bij NTA 8800:2024 (INT-V1:2024) brengt deze punten al in lijn met 2025+C1. Daarom rekent de kern hier in beide uitgaven hetzelfde:
- **Tabel 9.4** (Δθctr + Δθroomaut): de 2024-tabel (p. 280) gaf 2 / −0,5 / −1 K. INT-V1 p. 4 corrigeert dat naar 2,5 / 2,0 / 1,5 / 2,5 K, gelijk aan 2025+C1 p. 297.
- **Afgifteverlies (9.16)/(9.12a)** met θe;comb: dit ontbrak in de 2024-tekst (p. 280–281). INT-V1 p. 5 voegt het weer toe, gelijk aan 2025+C1 p. 297–298.
- **Collectief buffervat** (9.2.3.5): INT-V1 p. 3–4 schrijft de 2025-werkwijze voor.
- **Laatste term van (13.185) en (13.96):** INT-V1 p. 5 schrapt die, net als 2025+C1 p. 653.
- **θH,max in (13.141b):** INT-V1 p. 5 definieert die, net als 2025+C1 p. 611.

## Interpretaties

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
- `crates/nta8800-core/tests/public_comparison.rs`: de openbare gevallen B en C (gepubliceerd in 2023) in 2023; zie [de vergelijking](nta8800-vergelijking-openbare-rapporten.md).
- `crates/nta8800-service/tests/api.rs`: `supportedNormVersions` en een berekening in 2024 via HTTP.
- `src/__tests__/nta-norm-versions.test.tsx`: de keuze in het formulier, de status als "berekend", en de melding in het rapport en de statusbalk.
