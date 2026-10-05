# NTA 8800-uitgaven in de rekenkern

De rekenkern rekent standaard volgens de aangewezen uitgave, **NTA 8800:2025+C1:2026**. Een project kan ook een oudere uitgave kiezen. Dat is bedoeld om oude berekeningen na te rekenen en uitkomsten te vergelijken. Een berekening in een oudere uitgave is **nooit registreerbaar**.

Paginaverwijzingen gaan naar de gelicentieerde PDF's: NTA 8800:2024 (met het interpretatiedocument INT-V1:2024) en NTA 8800:2025+C1:2026. Deze repository bevat geen normtekst, alleen paragraaf-, formule-, tabel- en paginanummers.

## Kiezen van een uitgave

- **Projectbestand:** `ntaCalculation.normVersion`, met `"2025+C1"` (standaard) of `"2024"`. Ontbreekt het veld, dan rekent de kern in 2025+C1. De invoervingerafdruk van een project zonder het veld verandert daardoor niet.
- **App:** het NTA-invoerformulier heeft in het blok *Algemeen* het veld *Uitgave NTA 8800*. Bij een oudere uitgave tonen het rekenpaneel, het NTA-rekenrapport en de statusbalk "niet voor registratie".
- **API en MCP:** de projectbewerkingen lezen het veld uit het project. `GET /v1/version` (`get_version`) geeft onder `supportedNormVersions` de bekende uitgaven, met `implemented`, `registrationEligible`, `default` en de aanwijzingsperiode.
- **Bekend, niet geïmplementeerd:** `"2023"`, `"2022"` en `"2020+A1"`. De kern weigert die met `edition_not_implemented` (status `invalid`). Een onbekende waarde is een invoergat.

## Wat er verandert bij een oudere uitgave

| Uitkomst | 2025+C1 | Oudere uitgave |
| --- | --- | --- |
| Status | `calculated_unverified` | `calculated_legacy_edition` |
| `targetNormVersion` | `NTA 8800:2025+C1:2026` | `NTA 8800:2024 met INT-V1:2024` |
| `normVersion`, `registrationEligible` | `2025+C1`, `true` | `2024`, `false` |
| Registratiecontrole | volgens BRL 9500 | altijd de fout `legacy_edition_not_registrable` |
| Indicatoren die pas in 2025+C1 bestaan (§5.3.3–5.3.5, 5.5.6.1–2, 5.5.7, 5.6.4, 5.9, bijlage AB) | berekend | `null` |

Invoer voor een route die de gekozen uitgave niet kent, wordt geweigerd met `route_not_in_edition` op het pad van die invoer. Voorbeelden voor 2024: `bacsFactor` ≠ 1, een dakrand als belemmering van PV of collectoren, zonwerend glas (`glazingType: solar_control`), vaste lamellen, en `effectiveMassKgPerM2`/`roofAreaM2` van bijlage AA in 2025+C1.

## Hoe de kern de uitgave kiest

`crates/nta8800-core/src/norm_versions/` bevat:
- de enum `NormVersion` (oplopend: 2020+A1, 2022, 2023, 2024, 2025+C1);
- per geïmplementeerde uitgave één `NormProfile` (`v2025.rs`, `v2024.rs`) met de getallen en routekeuzes, elk met paginaverwijzing.

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
- **Biomassagrens:** 500 kW in beide uitgaven (2024 p. 92–95).
- **Bijlage AB** en de nieuwe indicatoren: alleen 2025+C1; in 2024 zijn ze `null`.

## Tests

- `crates/nta8800-core/src/norm_versions/tests.rs`: standaarduitgave, serde, het herstellen van de thread-local (ook na een paniek) en de profielwaarden met paginaverwijzing.
- Per schakelpunt een eenheidstest in de module van het schakelpunt, met de paginaverwijzing in de test.
- `crates/nta8800-core/tests/norm_versions.rs`: de projectroute (status, vingerafdruk, registratie, niet-geïmplementeerde en onbekende uitgaven) en een verschiltest van de voorbeeldprojecten in 2024 en 2025+C1. Daarin verandert alleen wat de lijst hierboven voorspelt.
- `crates/nta8800-service/tests/api.rs`: `supportedNormVersions` en een berekening in 2024 via HTTP.
- `src/__tests__/nta-norm-versions.test.tsx`: de keuze in het formulier, de status als "berekend", en de melding in het rapport en de statusbalk.
