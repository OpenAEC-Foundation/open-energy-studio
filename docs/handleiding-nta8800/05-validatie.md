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

Een leeg getal in het invoerblok wordt per veld gemeld als `nta_value_missing`. Een keuzeveld (type) dat leeg is, wordt als enige gemeld. De velden daaronder volgen pas als het type is gekozen. Fuzztests op drie voorbeeldprojecten bewaken dat geen veld vals of te weinig wordt gemeld.

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

Een dekkingstest (`crates/nta8800-core/tests/option_coverage.rs`) zet elke keuzemogelijkheid van de kerninvoer (elke variant en elk optioneel veld) een keer in de voorbeeldprojecten en rekent ze in alle vijf uitgaven. Elke run moet zonder paniek en met eindige getallen eindigen, als berekening of als weigering met een vertaalde meldcode. Een robuustheidstest (`crates/nta8800-core/tests/robustness.rs`) varieert de voorbeeldprojecten en opnames willekeurig. Hij faalt bij een paniek, een niet-eindig getal, of een ingreep van het vangnet.

## Plausibiliteit bij registratie

Bij registratie meldt het programma, zonder te blokkeren, onwaarschijnlijke uitkomsten (BRL 9500-W §7.2.2, p. 42):
- A+ of beter met een basisopname;
- een EP2 vlak onder een klassegrens;
- een sprong van drie of meer klassen;
- EP2, A_ls/A_g of A_g buiten een bandbreedte;
- een onwaarschijnlijke gemiddelde U;
- A_g met meer dan twee decimalen (`usable_floor_area_precision`, Praktijkhandboek p. 70).

Ook deze drempels zijn keuzes van het programma.
