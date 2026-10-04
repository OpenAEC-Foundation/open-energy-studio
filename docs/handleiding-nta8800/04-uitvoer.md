# 4. Uitvoer

Na het opslaan van de NTA-invoer rekent de kern zelf. Dat gebeurt een korte tijd na de laatste wijziging. De uitkomsten verschijnen op drie plekken:
- het paneel *NTA 8800-berekening (Rust-kern)*;
- het tabblad Resultaten;
- het tabblad Rapport.

Alle getoonde BENG-waarden komen uit de kern. De oude, vereenvoudigde berekening verschijnt alleen als er geen kernresultaat is. Dan draagt ze het label "Indicatief, niet volgens NTA 8800".

## Indicatoren

| Indicator | Betekenis | Afronding |
|---|---|---|
| BENG 1 (EP1) | energiebehoefte in kWh/m²·jr, uit een aparte run met het vaste C1-ventilatiesysteem (§5.4) | omhoog op 0,01 |
| BENG 2 (EP2) | primair fossiel energiegebruik in kWh/m²·jr | omhoog op 0,01 |
| BENG 3 (RER) | aandeel hernieuwbare energie in % | omlaag op 0,1 |
| TOjuli | temperatuuroverschrijding per oriëntatie, alleen bij woningbouw (§5.7) | omhoog op 0,01 |
| CO₂ | emissie per drager (tabel 5.3) | — |
| Finaal energiegebruik | per drager (§5.9) | — |
| ZEB-indicator | informatief (bijlage AB) | — |

**Bijzondere gevallen**
- Als EPTot en de hernieuwbare energie samen nul zijn, blijft EP2 bestaan en is het aandeel hernieuwbaar leeg (`null`).
- Een negatieve EPTot kan een aandeel boven 100 % geven. Dan volgt een waarschuwing.

## Label en labelgegevens

- **Klasse:** de labelklasse volgt uit de afgeronde EP2 en de klassegrenzen van de Regeling (bijlagen I/Ia, p. 15–17). Bij gemengde functies zijn de grenzen naar oppervlakte gewogen.
- **Woningen:** het label gebruikt de forfaitaire EMG-waarden (Regeling art. 2 lid 3, p. 4). BENG 2 en 3 en de Bbl-toets blijven op de kwaliteitsverklaring. De uitvoer heeft daarom aparte velden: `labelPrimaryFossilIndicatorKwhPerM2Year` en `labelRenewableSharePercent`.
- **Indicatief:** de klasse is altijd indicatief, omdat het programma niet geattesteerd is.
- **Labelgegevens (Regeling art. 4):**
  - algemene gegevens: functie, bouwjaar, A_g op twee decimalen, woningtype;
  - isolatie per element;
  - installaties;
  - EP2, het aandeel hernieuwbaar en TOjuli.

  Het bouwjaar komt eerst uit de registratie en anders uit de NTA-invoer.

## Bbl-toets

- **Woningen:** de toets vergelijkt BENG 1, 2 en 3 met de grenswaarden. BENG 1 hangt af van A_ls/A_g.
- **Utiliteit:** de grenswaarden zijn per functie naar oppervlakte gewogen (art. 4.149 lid 2). Woonfuncties en utiliteitsfuncties mogen niet samen worden gewogen.
- **TOjuli:** de grens van 1,20 geldt alleen voor woningbouw.
- **Rapport:** het rapport toont elke grens met hetzelfde aantal decimalen als de waarde.

De grenswaarden komen uit de Bbl-tekst, niet uit de NTA. Zie het bronnenregister in [hoofdstuk 8](08-versies-en-verwijzingen.md).

## Energie per energiefunctie

Per functie, drager en maand toont de uitvoer (`energyByService`) het gebruik, de levering, het primair fossiele en het hernieuwbare deel (§5.5.3, 5.20/5.21). De functies zijn:
- verwarming;
- warm tapwater;
- koeling;
- bevochtiging;
- ventilatie;
- verlichting;
- hulpenergie;
- collectieve bron.

Over de verdeling:
- **Eigen opwekking:** eigen PV-stroom die zelf wordt gebruikt, is naar rato van het elektriciteitsgebruik over de functies verdeeld. Dat is een interpretatie.
- **Combi-tapwater:** het tapwaterdeel van een combitoestel telt als tapwater, zonder f_BACS (13.184/13.185).
- **Totalen:** de som van de functies sluit aan op de totalen per drager.

Het rapport toont daarnaast:
- warm tapwater in detail;
- elke koelinstallatie apart;
- PV per systeem.

## Meldingen bij de uitkomst

- **Invoergaten:** wat ontbreekt; de berekening stopt.
- **Waarschuwingen:** wat ongebruikelijk is; de berekening gaat door. Voorbeelden:
  - opgegeven gebruik met een rendement boven 1;
  - f_BACS 1,0 zonder BACS-gegevens;
  - een zeer laag distributierendement door standaardwaarden voor de circulatieleiding;
  - een emissieverlies dat bijna oneindig wordt (10.15).
- **Codes en paden:** elke melding heeft een vertaalde tekst, met de code en het pad klein erbij.

Zie [hoofdstuk 5](05-validatie.md) voor de statussen.

## Bijlage Interpretaties

Het rekenrapport heeft een bijlage met alle interpretaties van de kern. Dat zijn de plaatsen waar de norm onduidelijk of strijdig is en de kern een lezing kiest. De lijst komt uit één functie (`kernel_interpretations()`) en is ook op te vragen:
- API: `GET /v1/nta8800/interpretations`;
- desktop-app: commando `kernel_interpretations`.

De lijst is in het Engels; het rapport vermeldt dat het de eigen tekst van de kern is.

## Rapporten

| Rapport | Inhoud | Taal |
|---|---|---|
| BENG-rapport (tab Rapport, Export Report) | indicatoren, grenzen, maandoverzicht, energiebalans | volgt de taal van de interface |
| NTA-rekenrapport | volledige uitkomst, labelgegevens, registratie, interpretaties | Nederlands (BRL 9500-document) |
| NTA-invoerdossier | de invoer | Nederlands |
| Adviesrapport maatwerkadvies | zie [hoofdstuk 6](06-maatwerkadvies.md) | Nederlands |

**Opmaak in de Nederlandse documenten**
- getallen met een decimale komma;
- tijden als `<time datetime="…">`, gelijk aan het manifest van het dossier.
