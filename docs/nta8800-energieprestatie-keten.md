# NTA 8800 energieprestatieketen: van gebouw naar BENG 2/3

De Rust-module `building_performance` sluit de rekenruggengraat af voor een gebouw met één rekenzone:

1. [maandelijkse warmte- en koudebehoefte](nta8800-maandbehoefte.md) (hoofdstuk 7, transmissie uit hoofdstuk 8);
2. [keten ruimteverwarming](nta8800-verwarmingsketen.md) (afgifte, distributie, één opwekker);
3. gedeclareerde overige diensten: tapwater, ventilatoren, koeling, hulpenergie (hoofdstukken 10, 11 en 13 nog niet in Rust);
4. `E_EPus` per energiedrager (5.20/5.21, met `f_BACS` op verwarming, koeling en hun hulpenergie);
5. eigen elektriciteitsproductie: eigengebruik en export (5.22–5.26, met `E_nEPus;el = 0` volgens 5.27);
6. `EPTot` (5.9–5.14, tabel 5.2) en `EPrenTot` (5.29–5.31, 5.39, tabel 5.4);
7. indicatoren met de afronding van [indicatoren-conceptdiagnose](nta8800-indicatoren-conceptdiagnose.md): BENG 2 naar boven op 0,01, BENG 3 naar beneden op 0,1.

Status `calculated_unverified`, `attestStatus = unattested`, `labelAvailable = false`.

## Bron en factoren

Bron is het [openbare consultatieconcept van hoofdstuk 5](https://www.internetconsultatie.nl/epg2026/document/14147), §5.5–5.6. De factoren komen overeen met de analyse van de doeluitgave in Open Heatloss Studio (F3a):

| Factor | Waarde |
|---|---|
| `f_P;del;el` = `f_P;pr;us;el` = `f_P;exp;el` | 1,45 |
| `f_P;del` aardgas en stookolie | 1,0 |
| `f_Pren;renelect` (PV, PVT, wind) | 1,45 |
| `f_Pren;renheat` (omgevingswarmte) | 1,0 |

Omdat de drie elektriciteitsfactoren gelijk zijn, is het netto resultaat maandonafhankelijk: `(E_EPus;el − E_PV)·1,45`. De test controleert zowel de maandroute als deze identiteit. `EPTot` mag negatief worden; er wordt niet afgekapt.

## Warmtepomp als hernieuwbare bron

`Q_H;hp;in = Q_H;gen;out·(1 − 1/COP)` (5.31) telt alleen mee bij COP ≥ 1, een brontemperatuur onder 20 °C en geen afvoerlucht als bron. De aanroeper levert `heatPumpRenewable` met bron. Die opgave moet kloppen met de forfaitaire bronklasse:

- een afvoerluchtbron moet als afvoerlucht zijn opgegeven;
- een collectieve bron van ≥ 20 °C mag niet als "onder 20 °C" zijn opgegeven.

Als `COP_prac` wordt de door de forfaitaire module gecorrigeerde COP gebruikt; of daar de juiste `f_prac` in zit, moet de normreview nog bevestigen. Omgevingswarmte van een tapwaterwarmtepomp (5.35/5.36) is een aparte gedeclareerde post.

## BENG 1

Volgens §5.4 moet de energiebehoefte voor BENG 1 worden berekend met een vast ventilatiesysteem C1 (tabellen 11.5/11.6) en vaste interne warmtelasten. Hoofdstuk 11 zit nog niet in Rust. Daarom verschijnt `needIndicatorKwhPerM2Year` alleen als de aanroeper met `demandUsesFixedC1Ventilation = true` bevestigt dat de ventilatie-invoer het C1-systeem voorstelt. De jaarlijkse behoefte met de opgegeven ventilatie staat altijd in `annualHeatingAndCoolingNeedKwh`.

## Geweigerd of niet ondersteund

- batterijopslag (5.14a bevat in het concept nog een placeholder);
- collectieve warmtepompbron (vergt de `dh`-factorroute);
- externe warmte- en koudelevering, biomassa en export van warmte;
- verlichting bij woningbouw (volgens de opmerking bij 5.20 op 0);
- ventilatoren, verlichting en hulpenergie op een andere drager dan elektriciteit;
- onvolledige inventaris van diensten of productie.

Een energielabel wordt niet bepaald; de labelgrenzen en de EP-Online-uitvoer zijn niet geïmplementeerd.

## Synthetisch voorbeeld

`training-data/nta8800-building-performance-synthetic.json` beschrijft een woning van 100 m² met een HR107-combiketel. Gegevens:

- `H_tr` 80 W/K en `H_ve` 40 W/K;
- 150 kWh gas per maand voor tapwater en 20 kWh per maand voor ventilatoren;
- 1.020 kWh PV per jaar.

Uitkomsten:

- warmtebehoefte 4.848 kWh, koudebehoefte 802 kWh;
- 5.719 kWh gas voor ruimteverwarming;
- `EPTot` 6.623 kWh, dus BENG 2 = 66,24 kWh/m²·jr;
- `EPrenTot` 1.421 kWh, dus BENG 3 = 17,6%.

Dit is een rekenkundige consistentiecontrole, geen referentiegeval.

## Aanroep

HTTP: `POST /v1/nta8800/performance/calculate`. MCP en desktop: `calculate_building_performance`. TS: `calculateBuildingPerformanceWithRust`.
