# NTA 8800 energieprestatieketen: van gebouw naar BENG 2/3

De Rust-module `building_performance` sluit de rekenruggengraat af voor een gebouw met één rekenzone:

1. [maandelijkse warmte- en koudebehoefte](nta8800-maandbehoefte.md) (hoofdstuk 7, transmissie uit hoofdstuk 8);
2. [keten ruimteverwarming](nta8800-verwarmingsketen.md) (afgifte, distributie, één opwekker);
3. berekende koeling (H10), tapwater (H13), verlichting bij utiliteit (H14) en PV (H16); ventilatoren (H11) en overige posten als gedeclareerde diensten;
4. `E_EPus` per energiedrager (5.20/5.21, met `f_BACS` op verwarming, koeling en hun hulpenergie);
5. eigen elektriciteitsproductie: eigengebruik en export (5.22–5.26, met `E_nEPus;el = 0` volgens 5.27);
6. `EPTot` (5.9–5.14a, tabel 5.2) en `EPrenTot` (5.29–5.32, 5.39, tabel 5.4);
7. de operationele CO2-emissie (§5.5.6.1, tabel 5.3);
8. indicatoren met de afronding van [indicatoren-conceptdiagnose](nta8800-indicatoren-conceptdiagnose.md): BENG 2 naar boven op 0,01, BENG 3 naar beneden op 0,1.

Status `calculated_unverified`, `attestStatus = unattested`, `labelAvailable = false`.

## Bron en factoren

Bron is het [openbare consultatieconcept van hoofdstuk 5](https://www.internetconsultatie.nl/epg2026/document/14147), §5.5–5.6. De factoren komen overeen met de analyse van de doeluitgave in Open Heatloss Studio (F3a):

| Factor | Waarde |
|---|---|
| `f_P;del;el` = `f_P;pr;us;el` = `f_P;exp;el` | 1,45 |
| `f_P;del` aardgas en stookolie | 1,0 |
| `f_Pren;renelect` (PV, PVT, wind) | 1,45 |
| `f_Pren;renheat` (omgevingswarmte) | 1,0 |

## Controle tegen NTA 8800:2025+C1:2026

- **Opslagcorrectie 5.14a/5.14b** (pagina 85):
  - Per maand geldt `E_P;BAT,out;tot = MIN(E_pr;el;ren;tot; E_EPus;el) · 0,05 · f_BAT;cor`. De term wordt zonder primaire factor afgetrokken in 5.10.
  - `f_BAT;cor` = 1 als de gebouwgebonden elektrische én thermische opslag samen minstens 5 kWh is, anders 0. Een stekkerbatterij telt niet.
  - Invoer: `batteryStoragePresent` plus `storage` (capaciteiten in kWh, met bron).
  - Alle gemodelleerde producenten (PV, PVT, wind) zijn hernieuwbaar; WKK is er niet.
  - Uitvoer: `annualStorageCorrectionKwh`.
- **`f_BACS`** (§5.5.8, pagina 100): 1,05 mag alleen bij utiliteitsbouw. Bij woningbouw volgt `bacs_factor_residential_invalid`. De factor weegt de opwekkerenergie en de hulpenergie van verwarming (inclusief distributiepomp) en van koeling.
- **CO2** (§5.5.6.1, tabel 5.3, peildatum januari 2025):
  - Factoren in kg/kWh: elektriciteit 0,268 (afgenomen en geëxporteerd), aardgas 0,218 (ook voor waterstof), stookolie 0,326, biomassa bmB 0,5 × 0,104 en externe warmte zonder verklaring 0,09.
  - `m_CO2` volgt de opbouw van 5.10 met `K_CO2` in plaats van `f_P;del`, zonder de opslagcorrectie.
  - Uitvoer: `annualCo2Kg` en `co2KgPerM2`.
  - Externe koude zonder verklaring telt met `K_CO2;el / 3`. Met een verklaring of een berekening volgens bijlage P gelden de waarden uit `externalSupply` (zie hieronder).
- **Externe warmte, tapwater en koude** (§5.8 en bijlage P, module `annex_p`):
  - Per drager (`dh` verwarming, `dw` tapwater, `dc` koude) gelden zonder verklaring de forfaitaire waarden: dh/dw `f_P` 0,9, `f_Pren` 0 en `K_CO2` 0,09; dc `f_P;el/3`, `f_Pren` 0 en `K_CO2;el/3`.
  - Met `externalSupply.heating`, `.hotWater` of `.cooling` gelden waarden volgens bijlage P, via drie routes:
    - `declared`: de waarden op de kwaliteitsverklaring;
    - `calculated`: P.7/P.9 met het distributierendement P.10–P.12 (twee van de drie jaarstromen) of het forfaitaire verlies per aansluiting (tabel P.0; koude 15 % onder 10 °C). De opwekkerfactoren volgen P.19–P.22 voor brandstoffen en warmtepompen (tabel P.5 of opgegeven rendement), P.26–P.30 voor WKK, P.6.5.4.7 voor restwarmte (0,07 kWh_e/kWh) en P.6.5.4.8 voor geothermie (`η = 20·Δθ/40`). Daarbij komt `f_Pren;dX` volgens 5.42/5.49/5.50 met 5.43–5.56;
    - `measured`: P.6 met gemeten jaarstromen.
  - Tapwater (`calculated` met `function: hot_water`) vraagt `hotWaterStorage`: `η_WD;gen;sto` volgens P.35 (verliezen van voorraad en leidingwerk, P.6.6.4.1/P.6.6.4.2) of forfaitair 0,90 / 0,80 / 0,50 naar isolatie (P.6.6.4.3). `f_WD;gen;tot` wordt daardoor gedeeld (P.34) en de opwekkers leveren `Q_in/η_sto`. Het kleine-systeemforfait voor tapwater rekent alleen met 90/60 (tabel P.0 voetnoot b; `hot_water_small_system_requires_90_60`). Tabel P.5 geldt niet voor tapwater (P.6.6.5.4 kent geen forfait; `table_p5_heating_only`).
  - WKK op biogas of biomassa telt `f_Pren` 1, op een bio-mengsel het biogene aandeel en op AVI 0,5 (5.45 met opmerking 6, 5.46). `K_CO2` volgt P.27 letterlijk zonder MAX(0) en kan bij een hernieuwbare brandstof negatief worden.
  - `f_P;XD;tot` wordt naar boven afgerond op 0,01 en `f_Pren;dX` naar beneden.
  - Hernieuwbare energie telt per drager als geleverde energie maal `f_Pren;dX` (5.39). Voor externe warmte gebruikt 5.39g `Q_H;gen;out` zonder f_BACS (f_BACS weegt wel EP_Tot), en voor een absorptiekoelmachine op externe warmte `Q_C;gen;out × f_Pren;dh` (de koudeproductie, niet de warmte-input `Q/ζ`; `districtHeatColdKwh` in de koelmaand).
  - Met een kwaliteitsverklaring rekent de kern twee keer (§5.3.1): EMGverklaring (de hoofduitkomst en het eerste scenario in `indicators`) en EMGforf met de forfaitaire waarden. `externalSupply` in de uitvoer geeft beide factorsets en EP_Tot, EP_ren en CO2 van EMGforf.
  - Bij een `ExternalHeat`-opwekker moet `qualityDeclarationPresent` overeenkomen met `externalSupply.heating` (`external_heat_declaration_mismatch`).
- **Collectieve warmtepompbron** (5.20, 9.6.8.1.1.2.3):
  - Een warmtepomp met een collectieve bron vraagt `externalSupply.collectiveHeatPumpSource` met de temperatuurklasse en een factuur of ontwerpbron.
  - De kern telt `Q_HD;hp;in;bron = Q_out·(1 − 1/COP)` per maand. Die energie telt zonder f_BACS tegen de factoren van de bron mee (drager `dh_hp_source`).
  - Onder 20 °C gelden forfaitair `f_P;el/23`, `K_CO2;el/23` en `f_Pren` 0,95, of de waarden volgens bijlage P. Dezelfde waarde geldt dan ook in EMGforf.
  - Vanaf 20 °C, bij oppervlaktewater of een onbekende bron, gelden tabel 5.2–5.4.
  - De omgevingswarmte van de warmtepomp telt dan niet nog eens als `renheat`.
- **Warmtepomp op buitenlucht én ventilatieretourlucht** (5.32):
  - Je geeft `combinedOutdoorAndExhaustAir` op, waarbij de bron in de COP-tabel buitenlucht moet zijn (tabel 9.27 voetnoot c).
  - Alleen het buitenluchtaandeel `f_H;buitenlucht` telt als omgevingswarmte. Zonder kwaliteitsverklaring is dat aandeel 0.

Omdat de drie elektriciteitsfactoren gelijk zijn, is het netto resultaat maandonafhankelijk: `(E_EPus;el − E_PV)·1,45`. De test controleert zowel de maandroute als deze identiteit. `EPTot` mag negatief worden; er wordt niet afgekapt.

## PV (hoofdstuk 16)

Module `pv` volgt §16.2 van NTA 8800:2025+C1:2026 (p. 678–682):

- 16.3: `E_sol = I_sol·t·F_sh;obst/1000`. `F_sh;obst` mag één waarde of twaalf maandwaarden zijn.
- 16.2: `E_PV = E_sol·P_pk·f_perf·c_sh;PV·0,95/1`.
- `P_pk` komt uit 16.4a of 16.4b:
  - 16.4a: `K_pk·A_PV/1000`, met `K_pk` uit tabel 16.1 (moduletype en jaar van plaatsing) of uit een verklaring;
  - 16.4b: piekvermogen per paneel, naar beneden afgerond op 5 W, maal het aantal panelen.
- `f_perf` komt uit tabel 16.2 via de bevestigingswijze. Een onbekende bevestiging telt als "niet geventileerd" (0,76).
- `c_sh;PV` wordt per maand afgeleid uit `F_sh;obst` met tabel 16.3 en lineaire interpolatie. Bij `F_sh;obst` ≤ 0,80 geldt 0,75.
- Een collectief systeem wordt verdeeld naar `A_g;tot/A_g;gebouw;PV` (p. 678).

Helling en oriëntatie werken alleen via `I_sol` uit tabel 17.2. PV loopt via `pvSystems` of via `onSiteProduction`, niet via beide.

## Tapwater (hoofdstuk 13)

Module `domestic_hot_water` rekent één tapwatersysteem met één opwekker (p. 525–655). Per maand:

1. **Nettobehoefte.**
   - Woningen: 13.15–13.18, 856 kWh per bewoner.
   - Utiliteit: 13.19 met tabel 13.1 per gebruiksfunctie (`areas`).
2. **Douche-WTW.** 13.51/13.52 met tabellen 13.7 en 13.8. Het rendement is het gemiddelde over alle douches (13.53); een douche zonder unit telt als 0. Per douche: forfait (verticaal, horizontaal, onbekend), een opgegeven rendement, of een testrapport volgens bijlage U (`annex_u`): drie runs per klasse (tabel U.1), als energieën of als meetreeks met dichtheid en enthalpie volgens U.5/U.6. Het gemiddelde wordt naar beneden afgerond op 0,025; een opgegeven rendement ook. De klasse moet passen (§13.5.3, `annex_u_test_class_mismatch`): woningen de toepassingsklasse van het tapwatertoestel (klasse 4 als die onbekend is), utiliteit altijd klasse 4. Tabel U.1 kent geen klasse 1; een klasse 1-toestel gebruikt de meting bij klasse 2 (interpretatie). Bij een opgegeven rendement wordt de klasse alleen getoetst als `testClass` is ingevuld.
3. **Afgifte.** `Q_W;em = Q_W;nd/η_W;em − Q_W;rcd` (13.9). De terugwinning gaat er dus ná het afgifterendement af. `η_W;em` komt uit tabel 13.2 (woningen, 13.21–13.23) of tabel 13.3 (utiliteit).
4. **Circulatie.**
   - Verlies 13.26, met Ψ uit tabel 13.4 (dichtstbijzijnde diameter) en de diameter uit tabel 13.29 als die onbekend is.
   - Lengte `L = 0,3·A_red + 10` (13.31), waarvan 15 % in onverwarmde ruimte bij 13 °C.
   - Pompenergie 13.34–13.44 met tabel 13.6. Van de pompenergie komt 80 % in het tapwater (13.50).
   - Distributierendement 13.25.
5. **Voorraadvat.** 13.58 (gemeten `H_sto;ls`) of 13.59 (label, tabel 13.9). Zonder label: C vanaf 2018, anders G.
6. **Afleversets.** Verlies 13.24/13.24a en elektronica 13.46.
7. **Opwekking.** `E = Q_W;dis/(f_prac·η_W;gen)` (13.3, 13.152). Ondersteunde opwekkers:
   - gastoestellen met tabel 13.25 en `c_W;gen` uit tabel 13.26;
   - warmtepompen 1,4·`c_source` met tabel 13.27, of een EN 16147-meting bij één tappatroon (13.160b, tabel 13.18);
   - elektrisch doorstroomtoestel (0,95) en elektroboiler (1,0);
   - gasboiler tot 150 kW (13.165–13.175);
   - overige direct verwarmde vaten (0,50);
   - ketel of warmtepomp met indirect vat (tabel 13.28, 1,4);
   - externe warmtelevering (η 1,0, drager `dh`).
   
   Een gastoestel kan in plaats van de tabelwaarde een testrapport volgens bijlage T meekrijgen (`annexT`, de Gaskeur-koppeling, p. 1096–1109):
   - een warmwatertoestel zonder cv-functie: T.4 en T.5;
   - een combitoestel met forfaitaire omrekening van het winterrendement: T.15–T.17 met `K_f` = 0,5, alleen bij een tapwaterzijdig rendement van ten minste 0,40 op bovenwaarde (CW);
   - een combitoestel met gemeten zomer- en winterrendement: T.12–T.14.

   Het resultaat gaat met tabel T.7 naar bovenwaarde. Het rapport vraagt de gemeten klasse; `c_W;gen` (tabel 13.26) werkt daarna zoals bij een verklaring, ook voor een toestel zonder Gaskeur of van onbekend type (tabel 13.25 opmerking 3 d: elke gemeten waarde hoort bij een klasse). De route vraagt `annexTConditions` (§13.8.4.3): het type werd al vóór 2021 geleverd en het toestel staat binnen; anders `annex_t_not_applicable`.

   Verklaarde rendementen worden naar beneden afgerond op 0,025 (gas) of 0,05 (elektrisch). Een warmtepomp mag niet in een hogere toepassingsklasse rekenen dan waarin hij is gemeten (`hot_water_heat_pump_class_exceeded`).
8. **Hulpenergie.** 13.181 voor opwekkers waarvan de hulpenergie niet in het rendement zit.
9. **Omgevingswarmte.** 5.36. Bij een afvoerluchtbron telt alleen het opgegeven buitenluchtdeel mee (5.37).
10. **Terugwinbare verliezen** `Q_W;ls;rbl` (13.13), per maand in `recoverableLossKwh`:
    - circulatie met `f_W;dis;rbl` = 1 als alle leidingen in verwarmde zones liggen, anders 0,85 (13.47);
    - afleversets, met `f_W;conv;rbl` = 1;
    - 20 % van de circulatiepompenergie (13.49);
    - voorraadvaten in een verwarmde zone (13.63), maar niet bij een gebouw boven 500 m²;
    - het opwekkingsverlies van een elektrisch doorstroomtoestel (13.179).

    Bij een utiliteitszone met `internalGains.method = utility` en een lege `hotWaterRecoverableKwh` vult de energieprestatieketen deze verliezen in als Φ_int;W (7.29), verdeeld naar gebruiksoppervlakte (13.14).

Niet in de kern:

- zonneboilers (§13.7, `Q_W;ren;sol = 0`);
- meerdere opwekkers per systeem (13.8.2);
- boosterwarmtepompen (bijlage W);
- afleversets op een collectief verwarmingssysteem (13.8.4.9.3);
- terugwinbare opwekkingsverliezen van warmtepompen en combitoestellen met geïntegreerd vat (13.160a), en de bijlage T-routes voor elektrische toestellen, bivalente warmtepompen en micro-WKK (T.3, T.7–T.10).

## Koeling (hoofdstuk 10, methode 3)

Module `space_cooling` (p. 366–426) rekent per maand `Q_C;gen;in = Σ(Q_C;nd + Q_C;em;ls + Q_C;dis;ls + Q_C;dis;rvd) − Q_C;HP` (10.5, 10.7–10.9). Onderdelen:

- **Afgifteverlies.** 10.10/10.11 en 10.15/10.16, met tabellen 10.35, 10.4, 10.5 en 10.5a (8 K woning, 12 K utiliteit).
- **Bedrijfsuren.** `t_C;mi` uit de koelgrens (10.19, stappen 1–6) en tabel 10.6.
- **Watergedragen distributie.**
  - Verlies 10.21/10.22, alleen in niet-gekoelde ruimten. Standaard is dat 15 % van `L_si = 0,64·A_g` bij 19 °C.
  - Ψ uit tabel 10.9, mediumtemperatuur uit tabel 10.8.
  - Pompenergie 10.28–10.38 met tabellen 10.10–10.13, plus teruggewonnen pompwarmte (10.45).
  - Bij directe expansie is er geen distributieverlies.
- **Ventilatorconvectoren.** 10 W per stuk tijdens de bedrijfsuren (10.17/10.18, tabel 10.7).
- **Verdeling over opwekkers.** Voorrang volgens tabel 10.15. `β` wordt naar boven afgerond op 0,1; de energiefractie komt uit tabel 10.16 (10.49–10.52). Bij meer dan één prioriteit zijn de vermogens verplicht.
- **Opwekking (methode 3, §10.5.6).**
  - Tabel 10.29: compressie 3,00, en gasmotor 3,00·`η_ge` met `ε_chp;el` uit tabel 9.31.
  - Tabel 10.30: gasabsorptie 0,80, absorptie op externe warmte 0,70.
  - Externe koude (10.78): drager `dc`, `f_P` = 1,45/3, `f_Pren` = 0.
  - Vrije koeling (10.86, tabel 10.34).
  - Regeneratietoeslag voor een bodemopslag die als warmtepompbron dient (10.84/10.85).
  - Verklaarde EER-waarden worden afgerond op 0,05 (elektrisch) of 0,025 (gas).
- **Hulpenergie opwekking.** Volgens 10.79:
  - geen condensorventilator in methode 3;
  - condensorwaterdistributie volgens tabel 10.33 bij watergekoelde machines;
  - regeling 0,010 kW gedurende alle uren;
  - bij vrije koeling alleen pompenergie.
- **Omgevingskoude.** 5.34: koude van vrije koeling met EER ≥ 8.

Absorptie op een WKK wordt afgewezen (`cooling_chp_unsupported`), omdat WKK niet in de verwarmingsketen zit. Niet gemodelleerd: koeling in de luchtbehandelingskast (H11), ontvochtiging (H12) en methoden 1 en 2 (meetgegevens volgens EN 14825/14511).

## Verlichting (hoofdstuk 14)

Module `lighting` (p. 655–676):

- **Woningen:** `W_L = 0` voor de indicatoren. Verlichtingsinvoer bij woningbouw wordt geweigerd.
- **Utiliteit, per verlichtingszone:** `W_L = P_n·F_C·(t_D·F_o;D·F_D + t_N·F_o;N)/1000` (14.7), met:
  - branduren uit tabel 14.1;
  - vermogen uit 14.8/14.9 met tabel 14.2, naar boven afgerond volgens bijlage X, of het forfait 14.13 met tabel 14.3;
  - parasitair vermogen 14.10–14.14;
  - `F_C = 1`;
  - aanwezigheid 14.16–14.23 met tabellen 14.4/14.5;
  - daglicht 14.24–14.44 met tabellen 14.6–14.9, voor gevelramen, daklichten of de forfaitaire methode.
  
  Bij forfaitair vermogen geldt `F_D = 1`.
- **Maandverdeling:** `t_mi/t_an`, zoals in 7.28.
- **Interne winst:** `f_L·W_t·1000/t_an` (7.28) staat per zone in de uitkomst. Neem die op in de opgegeven interne warmtelast van hoofdstuk 7; de koppeling gebeurt nog niet automatisch.

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

**Aanduiding A0.** Sinds de wijzigingsregeling van 24 april 2026 ([Stcrt. 2026, 18123](https://zoek.officielebekendmakingen.nl/stcrt-2026-18123.html), in werking in mei 2026; bijlage IX zelf is niet gewijzigd) mag bij een vergunningaanvraag na 29 mei 2026 de aanduiding A0 naast de labelklasse staan. Voorwaarden (art. 5.11/5.12 lid 5):

- a) BENG 1 voldoet aan tabel 4.148A;
- b) BENG 2 ≤ bijlage IXa/Xa, bijvoorbeeld andere woonfunctie 27, woongebouw 45, kantoor 36;
- c) BENG 3 voldoet aan tabel 4.148A;
- d) geen koolstofemissie uit fossiele brandstoffen ter plaatse.

Module `bbl_requirements::a0_check` toetst die voorwaarden, bij `permitApplicationAfter20260529 = true`. Voorwaarde d is afgeleid als "geen gas- of oliegebruik". Zonder BENG 1 (geen C1) is de uitkomst "niet te toetsen".

De oude TypeScript-labelfunctie gebruikte onjuiste grenzen (A+++ ≤ 20, F ≤ 340). Die zijn gecorrigeerd naar bijlage IX. De verplichte nieuwe labelgegevens van art. 5.13a (onder meer operationele broeikasgasemissies) zijn nog niet uitgewerkt.

## Toets Bbl art. 4.149

Module `bbl_requirements` bevat tabel 4.148A van het Besluit bouwwerken leefomgeving ([BWBR0041297](https://wetten.overheid.nl/BWBR0041297/2026-01-01), versie 1 januari 2026, opgehaald 1 oktober 2026). De tabel geeft per gebruiksfunctie:

- de maximale energiebehoefte (BENG 1), afhankelijk van `A_ls/A_g`;
- het maximale primair fossiele energiegebruik (BENG 2);
- het minimale aandeel hernieuwbare energie (BENG 3).

Grondgebonden woning (1e): 55 bij ≤ 1,5; `55 + 30·(x − 1,5)` tot 3,0; `100 + 50·(x − 3,0)` daarboven; BENG 2 ≤ 30; BENG 3 ≥ 50%. Woongebouw (1a): 65 bij ≤ 1,83; BENG 2 ≤ 50; BENG 3 ≥ 40%.

Lid 4 verhoogt de behoefte-eis met 5 kWh/m²·jr bij een gewogen `D_m` ≤ 180 kJ/m²K, maar alleen voor de rijen waar de tabel dat lid aanwijst: 1a, 1e en 7b. Een indicator die niet beschikbaar is, bijvoorbeeld BENG 1 zonder C1-ventilatie, geeft "niet te toetsen". Weging bij gemengde functies (lid 2) is niet geïmplementeerd.

De projectadapter leidt `A_ls` af volgens 6.3 en 6.7.3: vlakken naar buitenlucht en onverwarmde ruimten wegen 1, vlakken naar de grond 0,7, en scheidingen met aangrenzende verwarmde ruimten 0. Een vloer boven een kruipruimte moet als grondvlak zijn ingevoerd om de factor 0,7 te krijgen. Het ongewogen omhullend oppervlak staat apart als `envelopeAreaM2`. De gebruiksfunctie (`bblFunction`) moet de gebruiker zelf kiezen; het sjabloon vult die niet in.

Relevante bepalingen uit de Omgevingsregeling:

- art. 5.31a/5.31b: de BENG-waarden voor nieuwbouw worden bepaald door een bedrijf met BRL 9500-detailopname, met een programma dat volgens BRL 9501 is geattesteerd;
- art. 5.50 lid 2: de koelbehoefte van woningen gaat via de "Rekentool Koelbehoefte NTA 8800" in plaats van bijlage AA. Volgens de verificatie van 2 oktober 2026 vervalt die aanwijzing zodra NTA 8800:2025 is aangewezen. Bijlage AA in de doeluitgave (p. 1135–1147) is de gecorrigeerde versie die de rekentool al volgde. Dit moet nog tegen de geconsolideerde regeling worden bevestigd.

## TO-juli (§5.7)

Module `tojuli` bepaalt formule 5.40 per oriëntatie met een eigen koudebalans voor juli (7.2.2), volgens p. 113–120:

- **Stap A.** Ramen en opake vlakken naar buitenlucht met een helling boven 5° blijven bij hun oriëntatie, met eigen `A·U` en zonwinst. Vlakken naar een aangrenzende onverwarmde of verwarmde ruimte (AOR/AVR) doen niet mee. Zij zitten niet in `A_T` en niet in `H_C;D`, en stap B noemt geen component voor hun transmissie.
- **Stap B, stap 2.** Een koudebrug met expliciete ψ/χ hoort bij de oriëntatie van zijn constructiedelen (`orientations` op de brug, in het project als `N`…`NW`/`horizontal`). Een brug over meerdere oriëntaties wordt gelijk verdeeld.
- **Verdeling naar rato.** Bruggen zonder oriëntatie, horizontale vlakken (≤ 5°), grond, ventilatie, interne winst en `C_m` worden verdeeld naar `A_T;or/ΣA_T`.
- **Niet beoordeeld.** Oriëntaties met `A_T;or` ≤ 3 m².
- **Correcties.** `a_C;red` (7.74/7.75, tabel 7.15 via de gebruiksfunctie) vermenigvuldigt de julibehoefte. De onttrekking door een boosterwarmtepomp `Q_C;HP;juli` (10.6) wordt per zone naar oppervlakte en per oriëntatie naar behoefte verdeeld (5.41a–c).
- **Afronding en toets.** Naar boven op 0,01 K; toets aan 1,20 (Bbl 4.149b lid 1).
- **Actieve koeling.** TO-juli = 0 alleen met `activeCooling`. Daarvoor is nodig:
  - een systeem uit de lijst van §5.7.1;
  - een capaciteitsbewijs: dynamische koellast, bijlage AA, of beperking van de zoninstraling.

  Controles:
  - `A_w < 0,2·A_g` wordt tegen de ramen getoetst;
  - dauwpuntskoeling met bevochtigde afvoerlucht vereist methode 1 of 2;
  - "overig" is alleen toegestaan bij utiliteit;
  - het systeem moet passen bij de berekende koelopwekker.

De module vereist de `components`-transmissieroute, waarin ramen en opake vlakken precies de directe elementgeleiding dekken. Een afwijking geeft `tojuli_envelope_inconsistent`.

TO-juli rekent met dezelfde zone-invoer als de verwarmingsketen: de interne winst van hoofdstuk 14-verlichting (7.28) en de terugwinbare tapwaterverliezen (7.29) zijn daarin al ingevuld. Een dynamisch raam (bijlage A) telt in juli met `U_jul` in `H_C;D` en met `g_jul` in de zonwinst; bijlage AA gebruikt eveneens `g_gl;C;juli` (AA.6b) en `U_jul`. `Q_C;ls;rbl` is 0: hoofdstuk 10 kent geen terugwinbare koudeverliezen (`L_C;zi` = 0, pompwarmte gaat naar de last).

De nominale `g_gl;n` wordt overal naar beneden afgerond op 0,05 (§7.6.6.1.2 onder 7.40). De maandwaarde van een dynamisch raam (A.2) wordt niet afgerond.

**Nog niet uitgewerkt:**

- het geveldeel aan een aangrenzende onverwarmde serre (AOS);
- de geprojecteerde oppervlakte van hellende vlakken (nu bruto).

De zomerventilatie van hoofdstuk 11 moet in de opgegeven ventilatiegeleiding zitten.

## Warmtepomp volgens bijlage Q

Een `heat_pump_annex_q` telt voor 5.30/5.31 net als een forfaitaire warmtepomp: het omgevingswarmte-aandeel is `Q_hp·(1 − 1/η)` met het bijlage Q-rendement, als `heatPumpRenewable` de bron bevestigt. Een bron met alleen ventilatielucht telt niet. Bij gecombineerde bronnen telt alleen het buitenluchtaandeel (5.32). De bronconsistentie wordt gecontroleerd tegen het bijlage Q-brontype. De bronwarmte telt ook mee in de regeneratietoets van 10.84.

### Herziening na de review van bijlagen Q, M en N (2 oktober 2026)

- **f_prac (9.63, p. 340).** Het elektriciteitsgebruik is `Q_out / (COP · 0,95)`. `practiceFactor` staat in de uitvoer en `correctedEfficiency` is `COP · 0,95`.
- **Geen c_source bij methode 1.** Formule 9.63 kent geen broncorrectie; c_source (bijlage V) staat alleen in de forfaitaire tabellen 9.27/9.29 en de tapwatertabel. Een opgegeven `regeneration` wordt nog gevalideerd en de regeneratiegraad wordt gerapporteerd, maar `sourceCorrection` is altijd 1. Daardoor is ook de tapwaterterm van V.1 niet nodig.
- **Hulpenergie bronpomp (9.6.3.2).** Interpretatie: `W_H;aux;hp;an` zit volgens Q.4 al in de noemer van η_H;gen;hp (de COP van 9.63). Een tweede boeking van `W_aux/(12·3,6)` zou de bronpomp dubbel tellen; de kern boekt hem daarom niet opnieuw.
- **F = 1 zonder bijverwarming (Q.1).** De afgeronde uren van tabel Q.6 sommeren tot 277,757 in plaats van 277,778. Daardoor blijft de letterlijke F net onder 1, ook als de warmtepomp elke bin dekt. Interpretatie: als elke bin volledig is gedekt, geldt F = 1 en is geen bijverwarming nodig. Met bijverwarming blijft de letterlijke F gelden.
- **COP ≤ 0.** Een bin met geleverde warmte en een COP ≤ 0 (extrapolatie) wordt geweigerd met `annex_q_cop_not_positive`. Voorheen werd die bin stil overgeslagen.
- **Bijlage N, bovenwaarde (N.3).** Bijlage N rekent op de onderwaarde; hoofdstuk 5 telt brandstof op de calorische bovenwaarde. De keten vermenigvuldigt `E_H;gen;in` met f_Hs/Hi uit tabel M.3: gas 1,11, olie 1,06, biomassa als hout 1,08. Bijlage N geeft zelf geen verhouding.
- **Bijlage N, maand zonder vraag.** De hulpenergie na de brander (N.29) en in stand-by (N.31) loopt over t_gen door, met t_ON = 0.
- **Bijlage N, interpretaties.** Deze staan in `annex_n::INTERPRETATIONS`:
  - N.69 interpoleert de totale verliezen α_ON (N.49/N.50), omdat N.73 de mantelverliezen in de modulatieberekening meeneemt;
  - N.52 rekent in modulatie met β_cmb;min = 1;
  - de noemer van N.61 wordt gelezen als 100·(1 + Q_br/Q).
- **Bijlage M, ϑ_brm (M.12).** De omgevingstemperatuur volgt 9.4.2:
  - in een verwarmde ruimte het stookpunt (zoals bij de leidingen, voor θ_int;op;H van 7.9.6);
  - in een installatieruimte ϑ_ztu van het distributiesysteem, als die is ingevoerd;
  - anders tabel M.6.

Nog open:
- het terugwinbare verlies van de boosterwarmtepomp (13.164, hoofdstuk 13);
- de koppeling van de afvoerluchtfuncties van Q.5 aan hoofdstuk 11 (overventilatie, Q.96/Q.97).

## BENG 1

Volgens §5.4 rekent BENG 1 met een vast ventilatiesysteem C1 (§5.4.3) en, bij utiliteit, met vaste interne warmtelasten (§5.4.2). Als elke zone hoofdstuk 11-invoer heeft (`ventilation`), voert de kern die aparte run zelf uit (`fixedC1` per zone). BENG 1 en `annualHeatingAndCoolingNeedKwh` gebruiken dan de som van die runs. Zonder hoofdstuk 11-invoer verschijnt `needIndicatorKwhPerM2Year` alleen als de aanroeper met `demandUsesFixedC1Ventilation = true` bevestigt dat de opgegeven ventilatie het C1-systeem voorstelt.

De ventilatoren (11.132), de vorstbeveiliging (11.105) en de voorverwarming in roosters (11.125) uit hoofdstuk 11 tellen als elektriciteit zonder f_BACS. Ventilatorenergie mag dan niet ook als opgegeven post staan (`ventilation_fans_double_count`).

## Geweigerd of niet ondersteund

- opslag zonder opgegeven capaciteit (`storage_capacity_required`);
- collectieve warmtepompbron (vergt de `dh`-factorroute);
- export van warmte en absorptiekoeling op WKK;
- verlichting bij woningbouw (14.2.1: `W_L;spec = 0`);
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

**Meerdere rekenzones.** Elke zone krijgt een eigen maandbehoefte met alleen de eigen vlakken, ramen, koudebruggen en onverwarmde-ruimtegrenzen, en een eigen afgifte en distributie. De opwekker voorziet de som (9.2), en de behoefte voor BENG 1 is de som over de zones (5.6/5.8). Bij meer dan één zone is voor iedere zone een `zoneData`-regel verplicht. Die bevat ventilatiestromen en interne winst, en optioneel setpoints, massaklassen, afgifte en distributie. Wat niet is opgegeven, valt terug op de waarden op blokniveau. `A_g;tot` is de som van de zone-oppervlaktes. TO-juli wordt per zone en oriëntatie bepaald; de toets gebruikt de hoogste waarde. `D_m` voor Bbl lid 4 wordt naar gebruiksoppervlak gewogen.

Regels van de adapter:

- Een gevel krijgt helling 90°. Een vlak met oriëntatie `horizontal` krijgt 0°. Voor een dakvlak met oriëntatie is een expliciete helling verplicht.
- Iedere grondvloer moet een vermelding in `groundFloors` hebben.

HTTP: `POST /v1/nta8800/project/performance` met `{ "project": ... }`. MCP en desktop: `calculate_project_performance`. TS: `calculateProjectPerformanceWithRust`.

Het paneel **NTA 8800-berekening (Rust-kern)** staat op het project- en resultatenscherm. Het toont:

- invoergaten of afwijzingen;
- bij een volledige berekening BENG 1/2/3 met het label "Onverifieerd";
- BENG 1 alleen met bevestigde C1-ventilatie;
- jaartotalen, een maandtabel, de weggelaten correcties en de invoervingerafdruk.

Via **NTA-invoer starten** opent een gestructureerd formulier met secties die de rekenvolgorde volgen: algemeen, setpoints, massa, interne winst, ramen, dakhellingen, grondvloeren, ventilatie, afgifte/distributie, opwekker, tapwater, PV en bevestigingen. Een leeg getal wordt `null`, zodat de rekenkern het als invoergat meldt. Maandwaarden, toevoertemperaturen, zoneData en overige details gaan via **Geavanceerd (JSON)**. Beide editors starten vanuit een sjabloon. Daarin zijn alleen normvaste waarden ingevuld, zoals 20/24 °C voor woningbouw en de kolom van tabel 7.10. Projectspecifieke waarden staan op `null` en alle bronvermeldingen zijn leeg, zodat de rekenkern een invoergat blijft melden tot alles is ingevuld en onderbouwd. `training-data/nta8800-project-performance-synthetic.json` is een volledig synthetisch project. Met minimale belemmering (§17.3.2a) levert het via de devserverproxy BENG 2 = 14,37, BENG 3 = 72,9% en een warmtebehoefte van 2.643 kWh op; dat is een consistentiecontrole, geen referentiegeval.

## Bevochtiging en ontvochtiging (hoofdstuk 12)

- **Ontvochtiging:** zit in de koelketen. Q_C;dhum = f_DHU;C · Q_C;nd (12.5, tabel 12.2) komt bij de opwekkerlast. De factor volgt uit de ontwerptemperatuur van de koudedistributie; zonder distributie (directe expansie) of bij een onbekend ontwerp telt 6/12 °C. Bij stralingsafgifte (vloer, wand, plafond) is Q_C;dhum 0.
- **Bevochtiging:** `humidifiers` per zone, in de verwarmingsketen (of `ntaCalculation.humidifiers`). Vereist hoofdstuk 11-invoer, want 12.1 gebruikt de mechanische toevoer van de warmtebalans.
  - Een vernevelaar levert zijn latente warmte als last aan het knooppunt van de verwarmingsketen (9.4).
  - Een stoombevochtiger gebruikt elektriciteit (η 0,8) of gas (η 0,6).
  - Het terugwinbare verlies van stoombevochtiging (12.4) wordt nog niet teruggekoppeld naar de behoefte.

## Ventilatoren in de afgifte (9.21/9.22, tabel 9.11)

`emission.fans` beschrijft de ventilatoren voor luchtcirculatie in de ruimte. Bij `fan_assisted_radiators_or_convectors` is dit veld verplicht; zonder het veld geeft de kern `emission_fans_required`. Voor andere afgiftesystemen, zoals fancoils of de binnenunit van een split, is het optioneel.

- **Vermogen per ventilator** volgt tabel 9.11:
  - 10 W voor een ventilatorconvector of elektrische verwarming;
  - 12 W voor dynamische warmteopslag;
  - 12 W bij onbekend type (de hoogste waarde).
- **Getest vermogen**: een volgens NEN-EN 16430 getest vermogen (`testedPowerW`) vervangt de tabelwaarde.
- **Energie**: W_fan = Σ P·n·t_H;op;si;mi / 1000.
- **Bedrijfstijd**: t_H;op;si;mi is de langste bedrijfstijd (9.32a) van de zones op het systeem.
- **Boeking**: de energie komt in `emissionFanElectricityKwh` en telt mee in de hulpenergie van de verwarming.
