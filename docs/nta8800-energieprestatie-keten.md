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

## PV (hoofdstuk 16)

Module `pv` rekent 16.3 `E_sol = I_sol·t·F_sh;obst/1000` en 16.2 `E_PV = E_sol·P_pk·f_perf·c_sh;PV·0,95/1`. Helling en oriëntatie werken alleen via `I_sol` uit tabel 17.2 (volledig, met interpolatie). De tabellen 16.1–16.3 zijn niet als waarden getranscribeerd. Daarom zijn `P_pk` (kW) en `c_sh;PV` (0,75–1) bronverplichte invoer, en is `f_perf` beperkt tot de in de analyse genoemde 0,76/0,80/0,82. PV kan via `pvSystems` worden berekend of via `onSiteProduction` worden opgegeven; beide tellen op. Een zuiddak van 30° geeft circa 900 kWh/kWp per jaar.

## Tapwater (hoofdstuk 13, gedeeltelijk)

Module `domestic_hot_water` rekent de netto behoefte uit:

- woningbouw volgens 13.15: 856 kWh per bewoner per jaar (§13.2.3.1), met bewoners volgens 13.16–13.18 (dezelfde banden als 7.22–7.24), verdeeld naar `t_mi/8760`;
- utiliteit: een opgegeven waarde uit tabel 13.1 met bron.

Een optionele maandelijkse douche-WTW-bijdrage (13.51) wordt afgetrokken. De module deelt door de opgegeven `η_W;em` en `η_W;dis` en daarna door `η_W;gen` of de seizoens-COP. De tabellen voor afgifte, distributie en opwekking zijn niet getranscribeerd, dus die rendementen zijn bronverplichte invoer. Omgevingswarmte van een tapwaterwarmtepomp telt bij `renewableHeatPump=true` als hernieuwbaar (5.35/5.36). Dat mag alleen met elektriciteit en COP ≥ 1. `hotWater` en een gedeclareerde tapwaterpost tegelijk geeft `hot_water_double_count`. Bron van de getallen: de referenties van Open Heatloss Studio `nta8800-dhw`; review tegen de normtekst is nog nodig.

## Warmtepomp als hernieuwbare bron

`Q_H;hp;in = Q_H;gen;out·(1 − 1/COP)` (5.31) telt alleen mee bij COP ≥ 1, een brontemperatuur onder 20 °C en geen afvoerlucht als bron. De aanroeper levert `heatPumpRenewable` met bron. Die opgave moet kloppen met de forfaitaire bronklasse:

- een afvoerluchtbron moet als afvoerlucht zijn opgegeven;
- een collectieve bron van ≥ 20 °C mag niet als "onder 20 °C" zijn opgegeven.

Als `COP_prac` wordt de door de forfaitaire module gecorrigeerde COP gebruikt; of daar de juiste `f_prac` in zit, moet de normreview nog bevestigen. Omgevingswarmte van een tapwaterwarmtepomp (5.35/5.36) is een aparte gedeclareerde post.

## Indicatieve labelklasse

Module `label_class` zet de naar boven afgeronde BENG 2 om met de tabellen uit de Omgevingsregeling (art. 5.11/5.12 lid 4, [BWBR0045528](https://wetten.overheid.nl/BWBR0045528/2026-01-01), versie 1 januari 2026, opgehaald 1 oktober 2026):

- **Woningen, bijlage IX:** A++++ ≤ 0, A+++ ≤ 50, A++ ≤ 75, A+ ≤ 105, A ≤ 160, B ≤ 190, C ≤ 250, D ≤ 290, E ≤ 335, F ≤ 380, G > 380 kWh/m²·jr.
- **Utiliteit, bijlage X:** tien kolommen per gebruiksfunctie, met A+++++ tot en met G. Bij utiliteit is `labelFunction` nodig; zonder die opgave ontbreekt de klasse.

De uitkomst heet `indicativeLabelClass`; `labelAvailable` blijft `false`. Het echte energielabel wordt volgens art. 5.11/5.12 lid 3 pas na registratie door een gecertificeerde adviseur (BRL 9500) vastgesteld, met een rekenprogramma dat volgens BRL 9501 is geattesteerd.

De oude TypeScript-labelfunctie gebruikte onjuiste grenzen (A+++ ≤ 20, F ≤ 340). Die zijn gecorrigeerd naar bijlage IX. De wijzigingsregeling van 24 april 2026 ([Stcrt. 2026, 18123](https://zoek.officielebekendmakingen.nl/stcrt-2026-18123.html)) voegt onder meer een A0-aanduiding (bijlagen IXa/Xa) en nieuwe labelgegevens toe. Die zijn nog niet verwerkt; de datum van inwerkingtreding moet worden gecontroleerd.

## Toets Bbl art. 4.149

Module `bbl_requirements` bevat tabel 4.148A van het Besluit bouwwerken leefomgeving ([BWBR0041297](https://wetten.overheid.nl/BWBR0041297/2026-01-01), versie 1 januari 2026, opgehaald 1 oktober 2026). De tabel geeft per gebruiksfunctie:

- de maximale energiebehoefte (BENG 1), afhankelijk van `A_ls/A_g`;
- het maximale primair fossiele energiegebruik (BENG 2);
- het minimale aandeel hernieuwbare energie (BENG 3).

Grondgebonden woning (1e): 55 bij ≤ 1,5; `55 + 30·(x − 1,5)` tot 3,0; `100 + 50·(x − 3,0)` daarboven; BENG 2 ≤ 30; BENG 3 ≥ 50%. Woongebouw (1a): 65 bij ≤ 1,83; BENG 2 ≤ 50; BENG 3 ≥ 40%.

Lid 4 verhoogt de behoefte-eis met 5 kWh/m²·jr bij een gewogen `D_m` ≤ 180 kJ/m²K, maar alleen voor de rijen waar de tabel dat lid aanwijst: 1a, 1e en 7b. Een indicator die niet beschikbaar is, bijvoorbeeld BENG 1 zonder C1-ventilatie, geeft "niet te toetsen". Weging bij gemengde functies (lid 2) is niet geïmplementeerd.

De projectadapter leidt `A_ls` af als som van de bruto vlakken die aan buitenlucht, grond of een onverwarmde ruimte grenzen. Of die definitie exact aansluit op de verliesoppervlakte van NTA 8800 moet de normreview nog bevestigen. De gebruiksfunctie (`bblFunction`) moet de gebruiker zelf kiezen; het sjabloon vult die niet in.

Relevante bepalingen uit de Omgevingsregeling:

- art. 5.31a/5.31b: de BENG-waarden voor nieuwbouw worden bepaald door een bedrijf met BRL 9500-detailopname, met een programma dat volgens BRL 9501 is geattesteerd;
- art. 5.50 lid 2: de koelbehoefte van woningen gaat via de "Rekentool Koelbehoefte NTA 8800" in plaats van bijlage AA.

## TO-juli (§5.7)

Module `tojuli` bepaalt formule 5.40 per oriëntatie met een eigen koudebalans voor juli (7.2.2):

- Ramen en opake vlakken met een helling boven 5° blijven bij hun oriëntatie, met eigen `A·U` en zonwinst. Een hellend dak telt dus mee in zijn oriëntatie.
- Horizontale vlakken (≤ 5°, §7.6.6.4), koudebruggen, transmissie via onverwarmde ruimte, grond, ventilatie, interne winst en `C_m` worden naar rato van `A_T;or/ΣA_T` verdeeld.
- Oriëntaties met `A_T;or` ≤ 3 m² worden niet beoordeeld.
- De uitkomst wordt naar boven afgerond op 0,01 K. De hoogste waarde wordt getoetst aan 1,20 (Bbl 4.149b lid 1).
- Bij voldoende actieve koeling (`activeCoolingPresent`, §5.7.1) geldt TO-juli = 0.

De module vereist de `components`-transmissieroute, waarin ramen en opake vlakken precies de directe elementgeleiding dekken. Een afwijking geeft `tojuli_envelope_inconsistent`.

**Afhankelijkheid van ventilatie.** In de norm bevat de koudebalans van juli de zomerventilatie uit hoofdstuk 11 (onder meer spuiventilatie `q_V;argI`). Zolang hoofdstuk 11 niet in Rust zit, moet de opgegeven ventilatiegeleiding voor juli die stromen al bevatten. Het synthetische project heeft constant 35 W/K zonder spuiventilatie en komt daardoor op 6,36 K. Dat is rekenkundig consistent met de invoer, maar geen realistische waarde voor een woning. Verder niet uitgewerkt: de boosterwarmtepompterm `Q_C;HP;juli` (5.41a–c, bij afwezigheid 0), de splitsing van lineaire koudebruggen per oriëntatie (nu naar rato) en de geprojecteerde oppervlakte van hellende vlakken (nu bruto). De bron is de F3c-analyse van Open Heatloss Studio.

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

## Projectadapter en UI

Module `project_performance` leidt de volledige invoer af uit een `.oes`-project met één rekenzone:

- Uit het project zelf komen:
  - vlakken met thermische begrenzing en oriëntatie;
  - ramen (oppervlakte, U, g);
  - constructie-U;
  - lineaire en puntkoudebruggen;
  - onverwarmde ruimtes met b-factor.
- Uit het strikte blok `ntaCalculation` (onbekende velden worden geweigerd) komen alle gegevens die het oude model niet bevat, elk met bron:
  - rekenscope en oppervlaktebron, setpoints, massaklassen en interne winst;
  - hellingen van dakvlakken, kozijnfractie en belemmering;
  - perimeter en `R_si+R_c` van grondvloeren;
  - maandelijkse ventilatiegeleiding;
  - afgifte, distributie en opwekker;
  - `f_BACS`, overige diensten, PV en de C1-bevestiging.

Wat ontbreekt, verschijnt als invoergat (`gaps`) met code en pad. Een ongeldig blok geeft het exacte veldpad, bijvoorbeeld `ventilationFlows[0].months[3].conductanceWPerK`. Status `incomplete` (HTTP 422), `invalid` (422) of `calculated_unverified` (200). De afgeleide invoer wordt meegeleverd als `derivedInput`, zodat elke waarde herleidbaar is.

Regels van de adapter:

- Een gevel krijgt helling 90°. Een vlak met oriëntatie `horizontal` krijgt 0°. Voor een dakvlak met oriëntatie is een expliciete helling verplicht.
- Iedere grondvloer moet een vermelding in `groundFloors` hebben.

HTTP: `POST /v1/nta8800/project/performance` met `{ "project": ... }`. MCP en desktop: `calculate_project_performance`. TS: `calculateProjectPerformanceWithRust`.

Het paneel **NTA 8800-berekening (Rust-kern)** staat op het project- en resultatenscherm. Het toont:

- invoergaten of afwijzingen;
- bij een volledige berekening BENG 1/2/3 met het label "Onverifieerd";
- BENG 1 alleen met bevestigde C1-ventilatie;
- jaartotalen, een maandtabel, de weggelaten correcties en de invoervingerafdruk.

Via **NTA-invoer starten** opent een sjabloon. Daarin zijn alleen normvaste waarden ingevuld, zoals 20/24 °C voor woningbouw en de kolom van tabel 7.10. Projectspecifieke waarden staan op `null` en alle bronvermeldingen zijn leeg, zodat de rekenkern een invoergat blijft melden tot alles is ingevuld en onderbouwd. `training-data/nta8800-project-performance-synthetic.json` is een volledig synthetisch project. Het levert via de devserverproxy BENG 2 = 11,09 en BENG 3 = 77,7% op; dat is een consistentiecontrole, geen referentiegeval.
