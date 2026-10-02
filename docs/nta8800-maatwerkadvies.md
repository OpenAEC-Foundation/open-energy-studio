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

- Locatiespecifieke klimaatgegevens en beschaduwing (§2.6).
- De branduren van verlichting voor utiliteit (75.2 tabel 2.7, p. 44).
- De interne warmte per persoon voor utiliteit (80 W, 75.2 tabel 2.6).
- Maandelijkse elektriciteits- en warmtemeting in de fitcontrole.
- Een exacte warmtapwaterfit in hoofdstuk 13 zelf. Nu is die lineair geschaald, omdat `domestic_hot_water.rs` buiten deze wijziging viel.
- Het renovatiepaspoort (ISSO 82.2 §1.10 en §4.4).
- De EPBD-systeemeisen (ISSO 82.2 §5.2, tabel 5.1) als automatische controle.
- De kostenrekenregels van de ISSO-modelbeschrijving (rapport 110293).
- Monitoringbestanden voor MWA-registratie: daarvoor is een externe specificatie nodig.


## Praktijkfactoren voor ventilatie

Bij de berekening voor het werkelijk gebruik worden altijd de praktijkfactoren voor ventilatie toegepast (ISSO 82.2 tabel 2.7, p. 39; 75.2 tabel 2.8, p. 45). In het gebruiksprofiel kunnen ze worden overschreven met `ventilationPractice`.

| Factor | Werkt op | Standaardwaarde |
|---|---|---|
| f_prac;vent;sys | systeemgebonden debiet per deel van het systeem | A 0,25; C 0,5; D 0,75 |
| f_prac;argl | spuiventilatie | 0,5 |
| f_prac;lea | infiltratie | 0,5 |

Systeem B staat niet in de tabel en krijgt, als interpretatie, de waarde van C. De factoren lopen via `usageFit.ventilationPractice` naar hoofdstuk 11 (`VentilationInput.practice`). De labelberekening en de vaste C1-berekening voor BENG 1 gebruiken ze niet. De ventilatorenergie blijft gebaseerd op het eisdebiet.
