# 5. Validatie en meldingen

## Statussen

Elke projectberekening krijgt één status:

| Status | Betekenis | Wat te doen |
|---|---|---|
| `calculated_unverified` | De berekening is volledig doorgerekend. "Unverified" betekent dat het programma niet geattesteerd is en niet tegen officiële referentiegevallen is getoetst. | Uitkomst gebruiken als onverifieerde berekening. |
| `calculated_legacy_edition` | Volledig doorgerekend in een oudere uitgave van NTA 8800 (2024, 2023, 2022 of 2020+A1). Alleen ter vergelijking; de registratiecontrole weigert het project met `legacy_edition_not_registrable`. | Voor registratie de aangewezen uitgave kiezen (zie [hoofdstuk 10](10-normversies.md)). |
| `incomplete` | Er ontbreekt invoer: de kern kon geen rekeninvoer afleiden. De gaten staan in de uitvoer. | Gaten aanvullen. |
| `invalid` | De invoer is wel compleet, maar de kern weigert haar: fysiek onmogelijk, tegenstrijdig, buiten de blokkerende grenzen, of met een niet-eindige uitkomst. | Meldingen lezen en de invoer corrigeren. |

De basisopname kent daarnaast `derived_input_rejected`. Dan is de opname afgeleid, maar weigert de kern de afgeleide invoer.

## Gaten en waarschuwingen

- **Gat** (`gaps`): blokkeert de berekening. Elk gat heeft:
  - een code;
  - een pad naar het veld, zoals `ntaCalculation.ventilationFlows[0].months[3].conductanceWPerK`;
  - soms een detail.
- **Waarschuwing** (`warnings`): de berekening gaat door. Een waarschuwing wijst op iets ongebruikelijks dat de adviseur moet controleren.
- **Kernmelding** (`issues`): een weigering door de kern zelf, bijvoorbeeld bij een tegenstrijdige combinatie.

Een leeg getal in het invoerblok dat de kern nodig heeft, wordt per veld gemeld als `nta_value_missing`. Een leeg veld met een standaardwaarde in de kern (bijvoorbeeld de montage van PV of het aantal gedeelde zones van een verticale leiding) houdt die standaardwaarde: leeg telt als niet ingevuld. Een keuzeveld (type) dat leeg is, wordt als enige gemeld. De velden daaronder volgen pas als het type is gekozen. Fuzztests op drie voorbeeldprojecten bewaken dat geen veld vals of te weinig wordt gemeld.

## Invoergrenzen (keuzes van dit programma)

Deze grenzen zijn **geen normwaarden**. Ze vangen invoer af die geen bestaand gebouw kan beschrijven.

| Gegeven | Waarschuwing boven | Blokkerend boven |
|---|---|---|
| A_g van een zone | 10⁶ m² | 10⁷ m² |
| Oppervlak van een vlak of raam | 5·10⁵ m² | 10⁷ m² |
| U-waarde | 10 W/(m²·K) | 100 W/(m²·K) |
| q_v10 | 10 dm³/(s·m²) | 1000 dm³/(s·m²) |
| Opgegeven maandgebruik | 5000 kWh per m² per maand | 10⁶ kWh per m² per maand |

**Waarschuwingen zonder blokkade:**
- A_g van een zone onder 1 m²;
- minder dan 10 m² per woning;
- A_ls/A_g boven 20.

**Aantallen in de opnames en de kern:**
- bouwlagen tot 200;
- woningen en andere aantallen tot 100.000;
- gebouwhoogte tot 1000 m.

Alles daarboven wordt geweigerd voordat er gerekend wordt.

**Maatwerkadvies:**
- een levensduur van meer dan 100 jaar wordt geweigerd;
- elk getal in afgeleide gebouwinvoer boven 10¹² wordt geweigerd.

## Vangnet voor niet-eindige uitkomsten

Een uitkomst mag nooit oneindig of NaN zijn. De kern controleert elke uitkomst voordat die wordt uitgegeven:
- **Project, opnames en maatwerkadvies:** de status wordt `invalid` met de melding `non_finite_result` en het pad van het getal. De berekende delen worden achtergehouden.
- **API:** elke route geeft dan HTTP 500 met `{"error":"non_finite_result","path":…}`.
- **Desktop-app:** elk kerncommando geeft dan een foutmelding met het pad.

## Hoe de kern wordt bewaakt

Elke wijziging van het programma doorloopt dezelfde controle (`scripts/verify-nta.sh`). Een wijziging komt alleen in de code als alle onderdelen slagen. De onderdelen die voor de gebruiker van belang zijn:

- **Versie en handleiding.** De controle faalt als de releasenotes een wijziging van uitkomsten noemen zonder dat de kernversie omhoog is gegaan (`scripts/nta-kernel-version.mjs check`). Een item zonder rekenwijziging eindigt precies op "(geen rekenwijziging)". Ook faalt ze als deze handleiding niet is bijgewerkt voor de huidige kernversie (`scripts/nta-manual.mjs check`).
- **Dekking van alle opties** (`crates/nta8800-core/tests/option_coverage.rs`). Elke keuzemogelijkheid van de kerninvoer (elke variant en elk optioneel veld) gaat een keer door de voorbeeldprojecten en de openbare gevallen, in alle vijf uitgaven. Elke run moet zonder paniek en met eindige getallen eindigen, als berekening of als weigering met een vertaalde meldcode. Een geweigerde berekening moet ten minste één invoergat hebben met een **Ga naar**. De lijst met opties komt uit de invoertypen zelf, dus een nieuwe optie valt er vanzelf onder.
- **Numerieke fuzz** (`crates/nta8800-core/tests/numeric_fuzz.rs`). Elk getal in de voorbeeldprojecten en de openbare gevallen krijgt randwaarden (0, negatief, heel klein, heel groot, net buiten de grens), in alle vijf uitgaven. Er gelden dezelfde eisen als bij de dekkingstest. Daarnaast is een berekende uitkomst plausibel: geen negatieve energiebehoefte, en een hogere U-waarde of meer infiltratie verlaagt de warmtebehoefte nooit.
- **Robuustheid** (`crates/nta8800-core/tests/robustness.rs`). De voorbeeldprojecten en opnames worden willekeurig gevarieerd. De test faalt bij een paniek, een niet-eindig getal, of een ingreep van het vangnet.
- **Grote projecten** (`crates/nta8800-core/tests/large_projects.rs`). Een utiliteitsgebouw met tientallen rekenzones en een woongebouw met tientallen woningen moeten binnen een tijdslimiet rekenen, met uitkomsten die evenredig met de grootte meeschalen. De gemeten tijden staan in de [programmabeschrijving](../nta8800-programmabeschrijving.md#8-grenzen-en-prestaties).
- **Referentiesuites** (zie hieronder).

## Referentiesuites

De referentiesuites in `training-data/reference-suites/` rekenen vaste projecten door en vergelijken de uitkomst met een verwachting en een bandbreedte (absoluut, relatief, of de ruimste van beide):
- **Openbare gevallen A–O**: herbouwde openbare BENG-rapporten, elk in de uitgave waarin het rapport is gemaakt. Zie [`nta8800-vergelijking-openbare-rapporten.md`](../nta8800-vergelijking-openbare-rapporten.md).
- **RVO-voorbeeldwoningen**: de warmtebehoefte binnen de bandbreedte van RVO.
- **ISSO 54 versie 2.0**: de deeltesten uit de openbare uitgave van ISSO-publicatie 54, door ons gecodeerd. De verwachte uitkomsten zijn niet openbaar, dus deze gevallen staan op "geen verwachting": ze moeten rekenen, maar worden nog niet vergeleken. Zie [`nta8800-isso54-voorbereiding.md`](../nta8800-isso54-voorbereiding.md).

Het rapport (`reference-report.md` en `.json`) noemt per geval de uitkomst, het verschil en het oordeel, met de commit, de kernversie en de SHA-256 van invoer en uitvoer. Het eindoordeel:
- *geslaagd*: alle vergeleken gevallen liggen binnen hun band (afsluitcode 0);
- *mislukt*: een geval ligt buiten zijn band of rekent niet (afsluitcode 1);
- *geen vergelijking*: er is niets vergeleken, bijvoorbeeld bij een run met alleen ISSO 54 (afsluitcode 3).

Een rapport met alleen "geen verwachting" is dus geen bewijs van overeenstemming. Dat ontstaat pas met de verwachte uitkomsten van ISSO 54 versie 5.0:2026, waar BRL 9501 naar verwijst.

### Testsetdossier in het releasearchief

Bij elke vrijgave schrijft `scripts/release-nta.sh` de gecodeerde ISSO 54-deeltesten als testsetdossier in het archief, in de map `testset/` (BRL 9501 §3.3, p. 5–6):
- `projecten/<deeltest>.oes.json`: elke deeltest als gewoon projectbestand. Je opent het met **Openen** (zie [hoofdstuk 9](09-bestanden-en-uitwisseling.md)) en ziet dan precies de invoer waarmee de deeltest is gerekend. Zo kan de certificerende instelling een deeltest in het programma bekijken of zelf opnieuw invoeren.
- `testset.md`, `testset.csv` en `testset.json`: per deeltest het onderwerp, de uitgave, de status, BENG 1, 2 en 3, TO<sub>juli</sub>, de geleverde energie per dienst en de SHA-256 van projectbestand en antwoord.

De kolommen *verwacht* en *oordeel* vullen zich zodra een deeltest een verwachting heeft. Het oordeel komt uit de vergelijking van de suite: *binnen de band*, *buiten de band* (ook als alleen de labelklasse afwijkt) of *niet vergeleken*, met de status, als een deeltest met verwachting niet rekent. Zolang de verwachte uitkomsten van ISSO 54 versie 5.0 ontbreken, blijven beide kolommen leeg.

Het dossier noemt de commit waarmee het is gemaakt. Is het gemaakt vanuit een werkkopie met niet-vastgelegde wijzigingen (alleen bij een proefrun), dan staat daar "(met lokale wijzigingen)" achter. Een echte vrijgave kan alleen vanuit een schone werkkopie. Het manifest telt de projectbestanden en `SHA256SUMS` dekt alle bestanden. De technische beschrijving staat in het [testprotocol](../nta8800-testprotocol.md), §3.

## Plausibiliteit bij registratie

Bij registratie meldt het programma, zonder te blokkeren, onwaarschijnlijke uitkomsten (BRL 9500-W §7.2.2, p. 42):
- A+ of beter met een basisopname;
- een EP2 vlak onder een klassegrens;
- een sprong van drie of meer klassen;
- EP2, A_ls/A_g of A_g buiten een bandbreedte;
- een onwaarschijnlijke gemiddelde U;
- een opgegeven PV-piekvermogen boven 300 W/m² (`plausibility_pv_peak_power`). NTA 8800 16.4 stelt geen maximum; de kern weigert pas boven 400 W/m², als invoerfout;
- A_g met meer dan twee decimalen (`usable_floor_area_precision`, Praktijkhandboek p. 70).

Ook deze drempels zijn keuzes van het programma.
