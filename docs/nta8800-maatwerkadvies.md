# Maatwerkadvies (BRL 9500-MWA-W/U)

**Status:** onverifieerd, niet geattesteerd. Module `crates/nta8800-core/src/maatwerkadvies.rs`, route `POST /v1/nta8800/maatwerkadvies` (`{ "input": … }`), Tauri-commando `assess_maatwerkadvies`, TS `assessMaatwerkadviesWithRust`, paneel "Maatwerkadvies" in de resultatenweergave en een adviesrapport in HTML.

Bronnen (alleen paginaverwijzingen, geen tekst overgenomen):

- ISSO 82.2, 3e druk (woningen) en ISSO 75.2, 3e druk (utiliteitsgebouwen): de methode.
- BRL 9500-MWA-W en -U (19-06-2024): de eisen aan het advies, §3.1 (p. 11) en §4.2.5 (p. 17).

## Opbouw

Het maatwerkadvies rekent elke variant twee keer:

1. **NTA 8800-berekening.** Hieruit komen het (indicatieve) label, BENG 1–3 en TO<sub>juli</sub>. Deze run gebruikt geen gebruikersparameters.
2. **Berekening op werkelijk gebruik.** Dezelfde invoer, aangevuld met het gebruiksprofiel. Hieruit komen het jaarverbruik per energiedrager, het primair fossiel energiegebruik, de CO₂-uitstoot en de energiekosten.

Een run met gebruikersparameters geeft nooit een labelklasse: `indicativeLabelClass` is dan leeg.

Varianten:

- **Huidige situatie:** het project zonder maatregelen, met het huidige gebruik (`currentUse`).
- **Elke maatregel afzonderlijk.** Dit geeft inzicht in de kosten-batenverhouding (ISSO 82.2 §4.2.3, p. 53).
- **Elk pakket.** Volgens ISSO 82.2/75.2 §4.2.2 (p. 52) zijn er minimaal twee; met minder pakketten volgt een waarschuwing.

Na maatregelen geldt `futureUse`, of anders `currentUse`. Zo kan een gewijzigd gebruik na de maatregelen worden doorgerekend, zoals de opmerking bij BRL 9500-MWA-W §4.2.5 vraagt.

## Gebruiksprofiel (ISSO 82.2/75.2 §2.5, tabel 2.2)

Het profiel is `nta`, `energy_conscious`, `average` of `not_energy_conscious`. Elk veld kan daarnaast vrij worden ingevuld; de vrije invoer gaat voor het profiel.

**Woningen:**

| Parameter | Bron (ISSO 82.2) | Energiebewust | Gemiddeld | Niet energiebewust |
|---|---|---|---|---|
| θ<sub>int;set;H</sub> / θ<sub>int;set;C</sub> [°C] | tabel 2.3, p. 36 | 18 / 26 | 20 / 24 | 22 / 22 |
| Verlaagd setpoint [°C] | tabel 2.4, p. 37 | 14 | 16 | 18 |
| t<sub>H;red;day</sub> / t<sub>H;red;wknd</sub> [h] | tabel 2.4, p. 37 | 12 / 8 | 10 / 0 | 8 / 0 |
| f<sub>mod;sp</sub> [-] | tabel 2.4, p. 37 | 0,8 | 0,5 (woongebouw) / 0,6 | 0 |
| Q<sub>W;nd;spec</sub> [kWh per persoon per jaar] | tabel 2.5, p. 37 | 409 | 545 | 681 |
| q<sub>int;tot</sub> per persoon [W] | tabel 2.6, p. 38 | 152 | 180 | 208 |

**Utiliteitsgebouwen** (ISSO 75.2 tabellen 2.3–2.5, p. 42–43) gaan uit van de NTA-waarde met een correctie:

- setpoint verwarmen en koelen ±2 K;
- verlaagd setpoint ±2 K;
- uren verlaagd per dag ±20 %;
- weekenduren ±24 h, begrensd op 48 h;
- warmtapwaterbehoefte ±20 %.

"Gemiddeld" is gelijk aan NTA 8800. Tabel 2.4 vermeldt voor beide afwijkende profielen "NTA −20 %" bij de dagelijkse uren. De kern leest dit als +20 % voor energiebewust en −20 % voor niet energiebewust, in lijn met de weekendkolom.

**Kernaansluiting:**

- `MonthlyDemandInput.usageFit` (hoofdstuk 7): het verlaagde setpoint en de uren gaan via `function_profile`. f<sub>mod;sp</sub> gaat in 7.78. Het aantal bewoners en de warmte per persoon gaan in 7.21 voor woningen. Voor utiliteit gaat q<sub>Oc</sub>·f<sub>τ</sub> + q<sub>A</sub> in 7.25.
- Bij een fit mag het setpoint afwijken van tabel 7.13.
- `BuildingPerformanceInput.hotWaterNeedFit`: de werkelijke jaarlijkse netto warmtapwaterbehoefte Q<sub>W;nd</sub>. Dit is `Q_W;nd;spec × N_p` of een factor op de NTA-behoefte.
- De uitkomst van hoofdstuk 13 wordt lineair in de behoefte geschaald. Circulatie- en opslagverlies blijven daarbij gelijk (interpretatie).

## Maatregelen en pakketten

Een maatregel bestaat uit:

- een naam en een soort;
- een JSON-patch (RFC 6902: `replace`, `add`, `remove` met een JSON-pointer) op het project (`target: "project"`) of op de afgeleide gebouwinvoer (`target: "building"`);
- de investering, met verplichte kostenbron (BRL 9500-MWA-W §4.2.5);
- de levensduur en het verschil in onderhoudskosten;
- optioneel het jaar van uitvoering (fasering) en een notitie voor een specialist.

Een pakket bestaat uit een naam, de maatregelen en optioneel een waarschuwing voor gedeeltelijke of latere uitvoering.

Voorbeelden:

```json
{"op": "replace", "path": "/constructions/0/uValue", "value": 0.15}
{"op": "add", "path": "/ntaCalculation/pvSystems/-", "value": {"…": "…"}}
```

### Maatregelsjablonen

In het paneel kiest de adviseur een soort maatregel. Het sjabloon zet de keuzes om in de patch; de patch blijft wat de kern rekent. De soorten volgen het overzicht van mogelijke maatregelen in ISSO 82.2 §4.3 (p. 54–62) en ISSO 75.2 §4.3 (p. 65–78; verlichting p. 78):

| Sjabloon | Invoer | Patch op het project |
|---|---|---|
| Isolatie dak, gevel of vloer | vlakken, nieuwe Rc of U | nieuwe constructie (`/constructions/-`) met U = 1/(R_si + Rc + R_se), R_si 0,10/0,13/0,17 en R_se 0,04 (tabel C.2, p. 778); een dak steiler dan 60° krijgt R_si 0,13 (opmerking 3, helling uit `surfaceTilts`) en een eigen constructie; `constructionId` van de vlakken; bij een vloer met grondinvoer ook `constructionResistanceM2kPerW` = Rc + 0,17. Wanden tegen de grond worden niet aangeboden: de kern rekent een verwarmde kelder met `heatedBasement.wallResistanceM2kPerW` (8.38), niet met de constructie van de wand |
| Beglazing | ramen, nieuwe U en eventueel g | `uValue`/`gValue` van de ramen |
| Kierdichting | streefwaarde qv10 en bron | `infiltration` van de hoofdstuk 11-invoer (ook per zone) en `airTightness.qv10` |
| Ventilatiesysteem | systeemunit (tabel 11.5) | alleen `ntaCalculation.ventilation.system` (en `zoneData[].ventilation.system`); debieten, regeling en infiltratie van het project blijven |
| Warmtepomp of andere opwekker | opwekker; bij een forfaitaire warmtepomp de tabelrij (9.27/9.29) | `ntaCalculation.generator` en `heatPumpRenewable` (5.31/5.32); de vlaggen bron < 20 °C en ventilatieretourlucht volgen uit de gekozen bron. Bij afgifte via water is de ontwerpaanvoertemperatuur verplicht |
| Tapwatertoestel | toestel (§13.8) | `ntaCalculation.hotWater.generator` |
| PV | systemen (hoofdstuk 16) | toegevoegd aan `pvSystems`, id voorafgegaan door de maatregel-id; zonder opgegeven factoren start de belemmering op situatie e) (volledig), de conservatieve keuze voor PV (tabel 17.3 met opmerking 23, p. 706–707). Situatie a) (minimaal) vraagt een onderbouwing (`obstructionSourceReference`, alleen in het sjabloon). Een leeg piekvermogen blokkeert de maatregel |
| Zonneboiler | systemen (§13.7) | toegevoegd aan `hotWater.solar` |
| Douche-WTW | units per douche, aansluiting, bron (§13.6, bijlage U) | `hotWater.showerHeatRecovery` |
| Verlichting (utiliteit) | verlichtingszones (hoofdstuk 14) | per verlichtingszone (`zoneId/id`) alleen de gewijzigde leden `power`, `parasitic`, `occupancy`, `daylight` en `extractedLuminaires`, toegepast op de actuele verlichting. Bij forfaitair vermogen is daglichtregeling uitgeschakeld (F_D = 1, 14.24, p. 667); bij centrale aan-schakeling meldt het formulier dat F_o;D = F_o;N = 1 (14.16/14.17, p. 664) |

Het sjabloon wordt in de maatregel bewaard (`template`); de kern negeert het. `buildMaatwerkadviesInput` genereert de patches van alle sjabloonmaatregelen opnieuw tegen het project zoals het dan is, zodat de array-indices in de paden kloppen. Dat geldt voor het paneel, de rapportexport en het dossier.

Een sjabloon met openstaande problemen (bijvoorbeeld gekozen vlakken die niet meer bestaan, `selectionStale`, of een ontbrekend piekvermogen) gaat met `incomplete` naar de kern. De kern rekent geen variant met zo'n maatregel en meldt `measure_template_incomplete` op de regel; er wordt dus nooit met een halve patch gerekend.

**Migratie van oude sjablonen.** Ventilatie- en verlichtingssjablonen die vóór 3 oktober 2026 zijn bewaard, hielden een kopie van het hele blok; de patch zette daardoor latere projectwijzigingen terug. Bij het openen worden ze omgezet:

- Ventilatie houdt alleen het systeem (`system`). De editor wijzigde alleen de unit, dus dit is verliesvrij.
- Verlichting wordt omgezet naar de verschillen met het actuele project. Die kopie kan een maatregel niet onderscheiden van een latere projectwijziging, dus de maatregel blokkeert (`migrationReview`) tot de adviseur de wijzigingen bevestigt of bewerkt.

Bij het wisselen van sjabloonsoort blijven een zelf ingevulde levensduur en categorie staan; alleen de startwaarden van de vorige soort worden vervangen. Een investering van 0 € geeft een waarschuwing (ISSO 82.2 p. 82, 75.2 p. 98). "Handmatig" laat het sjabloon los en houdt de patchregels bewerkbaar. De startwaarde voor de levensduur is een bewerkbare suggestie; ISSO 82.2/75.2 geven geen levensduurtabel (alleen ketels 15–20 jaar, 82.2 p. 90). De kerntest `template_measures_run_on_the_example_projects` rekent de sjablonen op beide voorbeeldprojecten door (`training-data/nta8800-mwa-template-measures.json`); elke maatregel verlaagt daar EP2.

## Energiekosten en economie

**Kosten.** De jaarlijkse kosten volgen uit de tarieven van de adviseur. Die tarieven hebben een verplichte bron.

- Gas wordt omgerekend met 35,17 MJ/m³, de bovenwaarde van Groningen-equivalent.
- Elektriciteit wordt berekend als afname minus eigen gebruik, maal het leveringstarief, minus de teruglevering maal de terugleververgoeding.
- Warmte, koude, olie en biomassa gaan per kWh.
- Vaste gas- en warmteaansluitkosten tellen mee zolang de drager wordt gebruikt.

**Besparing.** De besparing is de huidige situatie min de variant.

**Terugverdientijd** (ISSO 82.2 §5.1, p. 79 en §6.2.4, p. 82): investering / (kostenbesparing − extra onderhoud).

**Netto contante waarde** (geëist in BRL 9500-MWA-W §3.1):

```
NCW = −Σ I + Σ_t (S·(1+e)^(t−1) − M)/(1+r)^t − herinvesteringen na de levensduur + lineaire restwaarde op de horizon
```

- r is de discontovoet, standaard 3 %.
- e is de energieprijsstijging, standaard 0.
- De horizon is standaard de langste levensduur in de variant.
- De rekenregels van de ISSO-modelbeschrijving (rapport 110293) zijn niet beschikbaar. Deze formule is daarom een interpretatie en de parameters komen van de adviseur.

## Advies

- **Best passend pakket.** Dit is de keuze van de adviseur (`advisedPackageId` met `adviceMotivation`). Zonder keuze stelt de kern het geldige pakket met de hoogste NCW voor, gemarkeerd als automatisch.
- **Waarschuwingen:**
  - minder dan twee pakketten;
  - de waarschuwing per pakket;
  - een ongeldige berekening;
  - een stijgende TO<sub>juli</sub> (ISSO 82.2 §4.2.3);
  - stijgende energiekosten.
- **Specialistnotities** (ISSO 82.2 §6.2.3, p. 82):
  - de notitie per maatregel;
  - de vermogensbepaling van warmtepompen en koelsystemen;
  - de samenstelling van isolatiepakketten bij condensatierisico.
- **Fasering.** Per pakket worden de maatregelen per uitvoeringsjaar gegroepeerd.

## Fitcontrole (ISSO 82.2/75.2 hoofdstuk 3)

Met `measured` vergelijkt de kern het berekende verbruik bij het huidige gebruik met het gemeten verbruik:

- de afwijking in % voor gas, elektriciteit en warmte;
- bij maandelijkse gasstanden: de regressielijn van het gasverbruik tegen de maandgemiddelde buitentemperatuur voor maanden onder 15 °C, voor zowel meting als berekening. De lijn geeft de helling in m³/K, het snijpunt en de stookgrens (§3.2.3).

Het fitten zelf blijft het werk van de adviseur, die het gebruiksprofiel aanpast (§3.2.4–3.2.8).

## Herlabelen

Het paneel "Herlabelen" laadt het projectbestand van het oorspronkelijke label en toont per wijziging het oordeel van `relabel.rs`. De mogelijke oordelen zijn:

- toegestaan (BRL 9500 bijlage 6a);
- niet toegestaan (bijlage 6b);
- te beoordelen.

## Nog niet ondersteund

- **Locatiespecifieke klimaatgegevens en beschaduwing (§2.6).** De kern heeft geen NEN 5060-uurwaarden of KNMI-reeksen. Wel kan de adviseur bij de fitcontrole de lokale maandtemperaturen van de meetperiode opgeven.
- **Een exacte warmtapwaterfit in hoofdstuk 13 zelf.** Die wordt nu lineair geschaald.
- **De kostenrekenregels van de ISSO-modelbeschrijving (rapport 110293).**
- **Monitoringbestanden voor MWA-registratie.** Daarvoor is een externe specificatie nodig.
- **De systeemeis voor koeling.** Hoofdstuk 10 rapporteert de primaire energie niet per systeem, dus de kern kan deze eis niet toetsen.

## Praktijkfactoren voor ventilatie

De praktijkfactoren voor ventilatie staan in ISSO 82.2 tabel 2.7 (p. 39) en 75.2 tabel 2.8 (p. 45).

- **Standaardprofielen:** de berekening voor het werkelijk gebruik past de factoren toe bij de profielen energiebewust, gemiddeld en niet energiebewust.
- **NTA-optie:** hier gelden ze alleen als ze zijn ingevuld, want tabel 2.2 (p. 35) geeft voor de NTA-optie "–".
- **Eigen waarden:** met `ventilationPractice` kunnen de factoren worden overschreven.
- **Opgegeven ventilatiestromen:** zones zonder de route van hoofdstuk 11 kunnen niet per deel worden gecorrigeerd. Het advies krijgt dan de waarschuwing `ventilation_practice_not_applied`.

| Factor | Werkt op | Standaardwaarde |
|---|---|---|
| f_prac;vent;sys | systeemgebonden debiet per deel van het systeem | A 0,25; C 0,5; D 0,75 |
| f_prac;argl | spuiventilatie | 0,5 |
| f_prac;lea | infiltratie | 0,5 |

Systeem B staat niet in de tabel en krijgt, als interpretatie, de waarde van C. De factoren lopen via `usageFit.ventilationPractice` naar hoofdstuk 11 (`VentilationInput.practice`). De labelberekening en de vaste C1-berekening voor BENG 1 gebruiken ze niet. De ventilatorenergie blijft gebaseerd op het eisdebiet.


## Utiliteit: personen en branduren verlichting

**Interne warmte per persoon (ISSO 75.2 tabel 2.6, p. 44).** Met `persons` (N_p van het gebouw) wordt q_Oc·f_τ per zone N_p·aandeel·q_oc;p·f_t/A_g:
- het aandeel is het aandeel in de gebruiksoppervlakte;
- q_oc;p is standaard 80 W (`heatPerPersonW`);
- f_t komt uit NTA tabel 7.2 (`occupancyTimeFraction`).

Daarbij komt q_A uit NTA tabel 7.3 (`applianceWPerM2`). Een vrij ingevulde `occupancyApplianceWPerM2` gaat voor.

**Branduren verlichting (75.2 tabel 2.7).** De branduren t_D en t_N van tabel 14.1 worden per profiel geschaald:

| Profiel | Factor |
|---|---|
| energiebewust | 0,8 |
| gemiddeld / NTA | 1,0 |
| niet energiebewust | 1,2 |

`lightingHoursFactor` overschrijft de factor. In hoofdstuk 14 werkt de factor via `ZoneLighting.burningHoursFactor`; de labelberekening zet die nooit.

## Warm tapwater bij de NTA-optie

ISSO 82.2 §2.5.3 (p. 37) noemt een praktijkcorrectie voor de NTA-optie, maar geeft geen getal. De kern neemt daarom de waarde van het gemiddelde profiel: 545 kWh per bewoner. Dit is een interpretatie.

## Fitcontrole (ISSO 82.2 hoofdstuk 3 en bijlage C.1)

**Meetgegevens.** Naast de jaarwaarden kunnen maandreeksen worden opgegeven voor:
- gas, in m³;
- elektriciteit, als netto afname min teruglevering (C.3);
- warmte;
- de lokale gemiddelde buitentemperatuur van de meetperiode (§3.2.1).

Ontbreekt de jaarwaarde, dan telt de kern een volledige maandreeks op.

**Regressielijnen.** Per drager bepaalt de kern een lijn door de maanden onder 15 °C:
- de gemeten lijn met de lokale temperaturen;
- de berekende lijn met het klimaatjaar van NTA 8800.

**Basislast.** Dit is het gemiddelde van de maanden vanaf 15 °C, het verbruik voor warm tapwater en koken.

**Stookgrens.** Dit is het knikpunt waar de lijn de basislast raakt: θ = (a − basis)/(−b) (§3.2.7).

**Criteria (bijlage C.1, p. 105).** De kern geeft per criterium voldoet, voldoet niet of niet te beoordelen, plus een totaaloordeel:

| Criterium | Tolerantie |
|---|---|
| jaarverbruik per drager | ±5 % |
| helling | ±5 % |
| stookgrens | ±1 °C |
| basislijn | ±5 % |

## Systeemeisen EPBD (ISSO 82.2 §5.2)

Elke variant krijgt `systemChecks` volgens Bbl art. 4.248 (tabel 4.248) en Omgevingsregeling art. 5.2 met bijlage VIII. De kern rekent ze op de standaard NTA 8800-berekening van de variant.

| Systeem | Waarde | Eis |
|---|---|---|
| Ruimteverwarming | (E_H − E_H;WKK)/Q_H;nd, met Q_H;nd zonder terugwinbare verliezen | ≤ 1,31 |
| Warm tapwater | (E_W − E_W;WKK)/Q_W;nd | ≤ 3,45 |
| Ventilatie (utiliteit) | E_V/q_V;ODA;req | ≤ 3,8 kWh/(m³/h) |
| Ingebouwde verlichting (utiliteit) | E_L/A_g | ≤ 75 kWh_prim/m² |

Uitgangspunten:
- De primaire energie volgt de f_P;del-factoren van tabel 5.2 van NTA 8800. Externe warmte telt forfaitair.
- WKK-bijdragen zijn 0, omdat de keten geen WKK voor verwarming of tapwater kent.
- Voor verlichting volgt de kern het Bbl (75). ISSO 82.2 tabel 5.1 drukt 17 af.

Het advies waarschuwt bij een pakket dat een eis niet haalt. De eisen gelden alleen als een systeem wordt geïnstalleerd, vervangen of verbeterd.

## Renovatiepaspoort (ISSO 82.2 §1.10 en §4.4)

Met `renovationPassport` wijst de adviseur drie pakketten aan:
1. beperken van de warmte- en koudevraag;
2. duurzame installaties;
3. opwekking en opslag.

De stappen zijn gestapeld: stap 2 bevat de maatregelen van stap 1, en stap 3 die van stap 1 en 2. Per stap rapporteert de kern het NTA 8800-label en het energiegebruik bij het werkelijk gebruik.

De eisen die de kern beoordeelt:
- alle drie de stappen aanwezig;
- Standaard voor Woningisolatie: een verklaring van de adviseur, of de BENG 1-waarde van stap 1 tegen een opgegeven grens;
- een motivatie bij de vooroorlogse standaard;
- maatregelen tegen oververhitting in stap 1;
- aardgasvrije hoofdverwarming na stap 2: geen gas- of oliegebruik, of hybride bij de vooroorlogse standaard;
- geen fossiele verbranding na stap 3;
- hernieuwbare opwekking in stap 3;
- opslagcapaciteit afgewogen.

Het rapport neemt de verplichte kanttekeningen over oververhitting op (§1.10.2), in eigen woorden.

## Netto contante waarde

- **Fasering.** Met `economics.baseYear` begint de investering van een maatregel in zijn `phaseYear`. De besparing van een pakket begint in jaar 1.
- **Onderhoud.** De onderhoudskosten stijgen jaarlijks met `maintenancePriceChange`.
