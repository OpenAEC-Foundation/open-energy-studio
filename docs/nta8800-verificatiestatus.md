# NTA 8800 verificatiestatus — 2 oktober 2026

## Samenvatting stand 3 oktober 2026

**Wat de kern doet.** De Rust-kern rekent de volledige keten van NTA 8800:2025+C1:2026 door. Dat loopt van projectinvoer of basisopname tot en met:
- EP_Tot, EP_ren en RER;
- BENG 1/2/3 en TO-juli;
- CO2 en de indicatieve labelklasse;
- de Bbl- en A0-toets, ook voor gemengde functies (art. 4.149 lid 2).

Alle onderdelen zijn getranscribeerd uit de gelicentieerde normtekst, met paginaverwijzingen. De normtekst zelf staat niet in de repository.

| Onderdeel | Stand |
|---|---|
| H5 indicatoren, CO2 (tabel 5.3), opslag 5.14a, bijlage P met opslagrendement WD, 5.39g | geïmplementeerd |
| H6 A_ls met f_ls (6.7.3), zone-indeling §6.4/6.5.2, gemengde rekenzones §6.5.3 | geïmplementeerd |
| H7 incl. §7.9, serres (7.30b), bijlagen A/B/D, terugwinbare verliezen 7.3–7.8, leidingdoorvoeren H_p | geïmplementeerd |
| H8.2 constructies (bijlagen C, E–I, L), H8.3 grond incl. kruipruimte en kelders, H8.4 b_U opgegeven of afgeleid (8.53–8.59) | geïmplementeerd |
| H9 afgifte incl. ventilatorenergie (9.21/9.22), distributie 9.26–9.51, meerdere opwekkers (9.6.1), WKK methode 1 en 2 (9.6.6), terugwinbaar opwekkerverlies (9.7), bijlagen M/N/O/Q/V/W | geïmplementeerd |
| H10 methoden 1 (NEN-EN 14825), 2 (NEN-EN 14511) en 3, LBK-koeling, toevoerluchtterm 10.20 | geïmplementeerd; methode 2 voor absorptiekoeling volgt tabelwaarden |
| H11 ventilatie met drukbalans, C1-run, LBK-naverwarming en -koeling (tabel 11.15), herberekening Q.5.3 | geïmplementeerd |
| H12 incl. terugwinbaar verlies van stoombevochtigers | geïmplementeerd |
| H13 incl. bijlagen T/U/W, 13.164, zonneboilers §13.7 (berekend, getest systeem, PVT, zonnecombi) en meerdere opwekkers 13.8.2 | geïmplementeerd |
| H14, H16, §5.7 TO-juli incl. bijlage AA en dynamische beglazing | geïmplementeerd |
| §17.3 belemmering, situaties a–g (tabellen 17.4–17.15), ook voor collectoren; 17.3.8 als opgegeven factoren | geïmplementeerd |
| Basisopname ISSO 82.1/75.1, incl. renovatieklassen, sterk geventileerde ruimten, leidingen in onverwarmde ruimten | geïmplementeerd |
| BRL 9500: registratieblok, termijnen, labelgegevens, bewijsregister, dossierexport, detailopname-eis, herlabelen (6a/6b), versiestempel | geïmplementeerd |
| Maatwerkadvies (BRL 9500-MWA, ISSO 82.2/75.2): profielen, praktijkfactoren, fitcontrole volgens bijlage C.1, NCW, EPBD-systeemeisen, renovatiepaspoort | geïmplementeerd; ISSO-kostenmodel en locatieklimaat ontbreken (externe data) |
| Bijlage X (afronding) | geïmplementeerd |
| §5.9 finaal energiegebruik (5.57–5.60), 16.17 wind = 0 | geïmplementeerd |
| Bijlage AB ZEB-indicator (informatief) met operationele CO2 | geïmplementeerd |
| WKK voor tapwater (13.8.4.8), basisopname voor collectieve, meervoudige en hybride installaties, WKK en zonneboilers, opname-UI | geïmplementeerd |

**Dekkingsaudits.** Er zijn twee volledige audits gedaan van hoofdstuk 5 t/m 17 en alle bijlagen.
- Audit 1 vond tien open punten. Die zijn alle afgewerkt:
  - de indicatoren van hoofdstuk 5;
  - meerdere tapwater- en koelsystemen;
  - identieke installaties;
  - f_prac bij externe levering;
  - θ_ztu;
  - bijlage V;
  - gaswarmtepompen;
  - renovatiestandaard;
  - f_BACS.
- Audit 2 (alle 1 284 genummerde formules gecontroleerd) beoordeelt per hoofdstuk:

  | Beoordeling | Hoofdstukken en bijlagen |
  |---|---|
  | Compleet | Hoofdstuk 5, 7 en 10–16; bijlagen B–I, L–O, Q, R, T–V, X, Y, AA, AB |
  | Bijna compleet | Hoofdstuk 6, 8 en 17; bijlagen A en W |
  | Echte gaten | Hoofdstuk 9: één verwarmingssysteem per gebouw, hybride alleen nieuwbouw. Bijlage P: berekende route compleet sinds 3 oktober 2026 (zie onder). |

Na audit 2 zijn de echte gaten gedicht. De kern dekt nu:
- meerdere verwarmingssystemen;
- hybride warmtepompen in bestaande bouw;
- de letterlijke boeking van W_H;aux;hp;an;
- θ_int;op;C;
- de berekende route van bijlage P (P.13–P.83, tabellen P.1–P.16);
- bijlage A stap 2 met opgegeven correctiefactoren;
- W.4–W.7 ter indicatie;
- in de opname:
  - utiliteit: gemengde zones, meerdere tapwatersystemen, LBK-batterijen, bevochtiging, luchtverwarming en passieve koeling;
  - woningen: aangrenzende onverwarmde serre (AOS), daklichten, woonboot en caravan.

Wat buiten bereik blijft:
- 17.3.8: daarvoor zijn uurdata volgens NEN 5060 nodig. De kern accepteert opgegeven factoren.
- Spouwen dunner dan 20 mm: de waarden staan in NEN-EN-ISO 6946 tabel 8, die niet beschikbaar is. Zo'n spouw kan als opgegeven R-laag worden ingevoerd.
- Bijlage J: dit is een productbepaling.
- Bijlage K: dit zijn meetregels; de oppervlakken zijn invoer.

Interpretaties die een BRL 9501-beoordelaar waarschijnlijk aankaart:
- `θ_e;avg;an` (7.14/7.15/7.73, bijlage D): §17.2 geeft geen jaarwaarde; de kern gebruikt overal de 10,67 °C die D.4 (p. 791) noemt;
- §5.7.1 methode 3, tweede situatie: elke niet-beweegbare zonwering van tabel 7.4a/7.4b telt als beperking (eerste streepje), zonder g-toets;
- bijlage Q: F_H;gen = 1 bij volledige dekking (ook met bijverwarming); Q.48 bij L/W; bronpomp dubbel geteld (Q.4 en 9.6.3.2); Q.4.4 met het verdampervermogen bij conditie 1;
- koelmethode 1: f_C;PL boven 100 % niet begrensd (10.56/10.58), met waarschuwing;
- de teken-keuze van Δθ_fan in 9.29 tegenover 10.20;
- 10.15 en tabel 10.16;
- 10.23c: het gemiddelde van ϑ_in en ϑ_out van tabel 10.8, in plaats van tweemaal de aanvoertemperatuur;
- bijlage N: N.69, N.52 en N.61;
- micro-WKK: P_th;sb;
- TOjuli: AOR/AVR;
- θ_ztu = 13 °C zonder b_U;
- C.12;
- de afronding in 13.77 en 13.130;
- luchtbehandelingskast met verwarmer en koeler (11.100/11.101, p. 493–494): tabel 11.15 alleen als de batterij die in die balans actief is nodig is (zie hieronder);
- Q_H;ahu (11.119) en Q_C;ahu (11.115) bevatten de ventilatorwarmte ΔT_fan;
- daklichten (14.41, p. 673): de daglichtfactor maal 100, zodat hij in % is;
- de maandverdeling van verlichting via t_mi/t_an;
- W_t in de interne warmtelast van 7.28 is W_L + W_P (zie [maandbehoefte](nta8800-maandbehoefte.md)).
- bijlage P, maandprofielen: P.74 (p. 1020) per perceeldeel zonder maandwaarden, ook voor een sorptiedeel met alleen een jaarwaarde; P.78 (p. 1022) telt alleen E_C;dc;mi, dus ontvochtiging zit wel in de jaarwaarde (P.77) en niet in het maandprofiel;
- f_Pren;dc (5.49, §5.8.3.2, p. 129) wordt niet afgerond; alleen 5.42 en 5.50 schrijven afronding naar beneden op 0,01 voor;
- P.70 (p. 1016): de formule noemt Q_CD;dis;tot;an, de legenda Q_XD;in;tot; de legenda wordt gevolgd;
- P.27 zonder MAX(0): een WKK op biogas geeft een negatieve K_CO2;gen, met de waarschuwing `chp_co2_factor_negative`;
- 12.2.1 (p. 521): "grote opwekkers (A_g > 500 m²)" toetst de bediende oppervlakte van de bevochtiger (`servedAreaM2`), anders de oppervlakte aan het verwarmingssysteem;
- energie per energiefunctie (§5.5.3, 5.20): de eigen benutte opwekking (5.22) is naar rato van het elektriciteitsgebruik over de energiefuncties verdeeld; de norm bepaalt E_pr;EPus;el alleen per drager. Export (5.10/5.13), de opslagcorrectie (5.14a) en hernieuwbare elektriciteit (5.39a) blijven gebouwtermen;
- tapwater uit een combitoestel of afleverset op het verwarmingstoestel (13.184/13.185, p. 653): het aandeel is E_W, zonder f_BACS (5.20a, p. 89); het hulpenergiegebruik van het toestel blijft bij verwarming;
- opgegeven interne warmteproductie (`internalGains.method = declared`): tabel 7.2/7.3 (p. 179–180) en 7.21 (p. 177) zijn rekenwaarden zonder alternatief. Een afwijkende waarde geeft een waarschuwing en geen blokkade, omdat de opgegeven flux ook de termen van 7.28 en 7.29 kan bevatten. De gebruiksaanpassing van het maatwerkadvies (bijlage Z) wordt niet getoetst.

- bijlage Q in een meervoudige opwekking (§9.6.3, p. 331; 9.6.3.2, p. 340): de warmtepomp krijgt de energiefractie F_H;gen;gpref van bijlage Q op de hele knooplevering, niet die van tabel 9.23; hij moet als enige voorkeur 1 hebben en de overige toestellen delen 1 − F volgens 9.60 (voorkeuren hernummerd, geschatte β herschaald);
- bijlage Q met een gasketel als bijverwarming: het hulpenergiegebruik van 9.6.8.1 van die ketel wordt elke maand geboekt, ook als de ketel bij volledige dekking niets levert;
- bijlage M: lpg, steenkool en bruinkool staan in tabel M.3, maar tabel 5.2/5.3 (p. 94, p. 97) geven er geen f_P;del of K_CO2 voor; de keten weigert ze (`boiler_fuel_without_primary_factor`);
- bijlage M: β boven 1 (M.24/M.25) wordt geweigerd als capaciteitstekort, zoals bij bijlage N; de ondergrens P_int ≥ 0,05·P_n en de bovengrens f_gen;ls;P0 ≤ 0,1 zijn keuzes van dit programma;
- bijlage M: stand-byverliezen (M.4/M.6) verbruiken brandstof in elk uur van de maand, ook als de terugkoppeling van 9.7 de levering van een sterk overgedimensioneerde ketel tot 0 brengt; de norm wordt letterlijk gevolgd;
- biomassaklasse (tabel 5.2/5.4): een typeplaatvermogen boven 500 kW geeft bmA, ongeacht het opgegeven 500 kW-vinkje;
- V.1 (p. 1115) verwijst voor η_H;gen naar 14.6, dat in deze editie de daglichtafhankelijkheid van verlichting is; de kern neemt de geleverde COP met η_el = 1/f_P;del;el (onttrekking Q·(1 − η_el/COP)). Lezen als COP·η_el geeft Q·(1 − 1/COP) en een hogere R; de kernlezing is de behoudende.
Ze staan per module in `INTERPRETATIONS` en in de docs. `interpretations::kernel_interpretations()` verzamelt de lijsten; de API (`GET /v1/nta8800/interpretations`) en de desktop (`kernel_interpretations`) geven ze door, en het rekenrapport drukt ze af als bijlage.

**Onafhankelijke herberekeningen (3 oktober 2026).** Bij gebrek aan officiële referentiegevallen zijn beide voorbeeldprojecten opnieuw doorgerekend. De agent schreef daarvoor een eigen Python-implementatie, rechtstreeks vanuit de normtekst en de gerenderde pagina's, zonder de Rust-formules te lezen.

| Voorbeeld | Vergeleken | Grootste verschil met de kern | BENG 2 | RER | Label |
|---|---|---|---|---|---|
| Tussenwoning | 29 maandtermen | 5·10⁻¹³ | 17,13 | 57,5 % | A+++ |
| Kantoor | 23 maandtermen | 0 | 39,49 | 54,7 % | A++++ |

De vergeleken termen beslaan:
- hoofdstuk 7: winsten, setpoint-nivellering, tijdconstante, de grond volgens bijlage D, benuttingsfactoren, warmte- en koudebehoefte;
- hoofdstuk 9: afgifte en opwekking;
- hoofdstuk 14: verlichting;
- hoofdstuk 16: PV en export;
- hoofdstuk 5: indicatoren en label.

Elke afwijking die tijdens het opzetten werd gevonden, bleek een fout in de Python-versie te zijn; de kern volgde de norm.

De herberekeningen brachten wel invoer- en validatiepunten aan het licht:
- mengen van forfaitaire en gedetailleerde koudebruggen;
- verticale leidingen met de waarde "onbekend";
- f_BACS 1,0 zonder onderbouwing;
- een open plafond bij utiliteit;
- TOjuli bij utiliteit;
- onrealistische opgegeven gebruiken in de voorbeelden.

Deze punten zijn afgewerkt in `3eec124`. De herberekende waarden in de tabel gelden voor de voorbeeldinvoer van vóór die correctie. De actuele uitkomsten staan in `docs/nta8800-voorbeeldproject-smoketest-2026-10-03.md`.

**Herberekening van bijlage M, bijlage N, meervoudige opwekking, biomassa en bijlage V, 3 oktober 2026.** Eigen Python-implementatie vanuit de normpagina's (p. 870–923, 324, 331, 340–342, 1114–1116). De rekenkunde van de kern kwam in elk doorgerekend geval overeen. Eén inhoudelijk verschil is gecorrigeerd:
- **Bijlage Q in een meervoudige opwekking.** Een warmtepomp met bijlage Q naast een gasketel kreeg de energiefractie van tabel 9.23 (januari 0,563 bij β = 8/28, 450 kWh gas). §9.6.3 (p. 331) en 9.6.3.2 (p. 340) laten bijlage Q zowel de energiefractie als het rendement bepalen. De warmtepomp neemt nu F van bijlage Q op de hele knooplevering; bij volledige dekking levert de ketel niets.

Verder toegevoegd: weigering van lpg en kolen (geen factor in tabel 5.2/5.3), van een omgekeerd condenserend rendement, van een onrealistische stand-byverliesfactor of P_int, van β > 1 en van levering zonder bedrijfstijd, en de 500 kW-grens voor biomassa op het typeplaatvermogen.

**Herberekening van ventilatie (H11) en verlichting (H14), 3 oktober 2026.** Ook dit is gedaan met een eigen Python-implementatie vanuit de normpagina's. Getest zijn drie utiliteitsscenario's voor ventilatie (onder meer D5a met CO2-regeling, twee luchtstroomzones, een luchtbehandelingskast met verwarmer en koeler, en gedeclareerde en forfaitaire ventilatoren) en vier voor verlichting (installatie en forfait, daglichtsectoren met verticale ramen en daklichten, de forfaitaire daglichtmethode). Alle vergeleken maandtermen kwamen overeen tot op de rekenprecisie (≤ 1e-14). Wat er is vastgelegd of aangepast:
- **11.100/11.101 bij een kast met verwarmer en koeler (interpretatie).**
  - De kern past tabel 11.15 alleen toe als de batterij die in die balans actief is nodig is. De koudebalans rekent zonder naverwarming (θ_rh = 0) en de warmtebalans zonder koeling (Q_C;ahu = 0), zoals de slotalinea op p. 494 zegt.
  - Een letterlijke lezing van p. 493 ("koeling en naverwarming → tabel 11.15") zou altijd de tabelwaarde geven. De energie van de batterijen verandert daardoor niet; de warmtestroom per setpoint wel.
  - De verwijzing naar tabel 11.16 op p. 494 lezen we als drukfout voor tabel 11.15.
- **Ventilatorwarmte in de batterijlast (11.119/11.115).** De formules gebruiken θ_SUP;dis;in na de toevoerventilator, terwijl figuur 11.1 de batterij vóór de ventilator plaatst. De kern volgt de formule, dus de batterijlast bevat ΔT_fan. Dit is een normpunt, geen fout in de kern.
- **Daklichten (14.41, p. 673).** De legenda en tabel 14.8 rekenen in %, de gedrukte formule geeft een fractie. De kern vermenigvuldigt met 100.
- **Maandverdeling van verlichting.** Hoofdstuk 14 rekent alleen per jaar. De kern verdeelt volgens t_mi/t_an.
- **Tabel 14.7 onder D = 0,13 %.** De kern houdt de eerste tabelwaarde 0,12 aan; de norm geeft geen extrapolatieregel.
- **Nieuwe validatie.**
  - Een gemeten q_v10 ≤ 0 wordt afgewezen (`infiltration_invalid`): zonder lekpad sluit de massabalans van 11.3 niet.
  - Een terugregel-x die gunstiger is dan de standaard (recirculatie > 20 %, debietregeling < 80 %) vraagt `flowReduction.evidenceReference` (opmerkingen 1 en 2 bij 11.60/11.61, p. 468), anders `flow_reduction_evidence_required`. De basisopname geeft de bron van de ventilatieopname mee bij een opgenomen percentage.
  - `largeOfficeGroup` hoort alleen bij een zone met kantoorfunctie (§14.5.1, p. 664). Buiten een kantoorfunctie geeft het F_o;D = 1 (14.16), de minst gunstige waarde. Daarom is `lighting_large_office_group_without_office` een waarschuwing en blokkeert het de berekening niet.
  - `assess_zone_lighting` controleert de invoer zelf en weigert ongeldige invoer, zoals een gemengd forfait.

**Review van bijlage A in de projectroute, 3 oktober 2026.**
- **Dynamisch raam en beweegbare zonwering (§A.2, p. 767; 7.42).** Bijlage A rekent een raam met beweegbare luiken of zonwering zelf tot de dynamische elementen (A.2, figuur A.1). Een dynamisch raam met daarnaast de projectbrede beweegbare zonwering van 7.42 zou de zonweringstand dus twee keer tellen. De norm schrijft niet voor welke van de twee voorrang heeft. Daarom geeft de combinatie een gap (`window_dynamic_and_shading_exclusive`), in de projectroute op `dynamicWindows[i]` en in de kern op `windows[i].movableShading`. De zonwering hoort dan in de toestanden van bijlage A.
- **τ_vis en τ_sol van bijlage A (interpretatie).** Volgens p. 767 is τ_vis invoer voor hoofdstuk 14. Hoofdstuk 14 heeft daar echter geen ingang voor. 14.38 (verticale ramen) bevat geen doorlatingsfactor, en 14.41 zet τ_D65 vast op 0,6 voor daklichten (p. 673). De kern bewaart τ_vis en τ_sol daarom alleen als vastlegging. Het formulier vraagt ze niet meer, omdat ze de uitkomst niet veranderen.
- **Half ingevulde invoer.** Een lege waarde in bijlage A (g, U, een weging of een correctiefactor) geeft `dynamic_value_missing` op het eigen pad. Eerder maakte één leeg veld het hele NTA-blok onleesbaar (`nta_calculation_block_invalid`).
- **Ramen zonder grens.** Het formulier biedt alleen ramen aan in vlakken met een expliciete buitengrens. Een vlak zonder grens slaat de projectroute over (met een gap).

**Herberekening van de berekende routes voor tapwater (H13) en koeling (H10, methode 2), 3 oktober 2026.** Ook deze controle is gedaan met een eigen Python-implementatie vanuit de normpagina's. Vergeleken scenario's:
- tapwater: vijf scenario's (circulatie, voorraadvaten, warmtepompen volgens 13.160b, alle standaardwaarden);
- koeling: twee scenario's.

Hoofdstuk 13 kwam op elke maandterm overeen. In hoofdstuk 10 bleek één echte fout te zitten. Wat er is aangepast:
- **10.73: uittredetemperatuur van de opwekker (gecorrigeerd).** Volgens §10.3.4 (p. 379–380) geldt ϑ_C;gen;req;out = ϑ_C;gen;out;set = ϑ_C;dis;flw;set. In tabel 10.8 is dat de ontwerp-aanvoertemperatuur min Δϑ_int;inc.
  - De kern gebruikte de kale aanvoertemperatuur (6/12/17 °C) en ging zonder distributie uit van de kolom 6/12. Ze rekent nu met aanvoer − Δϑ_int;inc. Omdat Δϑ_int;inc negatief is, wordt de aanvoertemperatuur dus hoger.
  - Bij verdamping in de ruimte geldt nu ϑ_C;int;inc = 24 + Δϑ_int;inc (10.10) in plaats van 24 °C.
  - Het effect: een hogere EER. In het herberekende scenario met een luchtgekoelde 12/18-machine daalt het jaarverbruik met ongeveer 19 %.
- **10.23c: gemiddelde mediumtemperatuur (interpretatie, ongewijzigd).** De gedrukte formule zet ϑ_C,in en ϑ_C,out allebei gelijk aan ϑ_C;dis;in;flw;req. Dat zou het gemiddelde laten samenvallen met de aanvoertemperatuur. De tekst erboven (p. 383) zegt echter dat de gemiddelde temperatuur "uit tabel 10.8" volgt, en die tabel geeft een aparte ϑ_in en ϑ_out. De kern volgt die tabel: (ϑ_in + ϑ_out)/2 − Δϑ_int;inc. De tweede regel van 10.23c lezen we als drukfout; anders zou de ϑ_out-rij van tabel 10.8 nergens gebruikt worden.
- **10.15: afgifteverlies vlak bij de singulariteit (letterlijk, met waarschuwing).** Als ϑ_C;int;inc − ϑ_e;comb vlak onder nul ligt, wordt Δϑ_int;inc/(ϑ_C;int;inc − ϑ_e;comb) groot. Het afgifteverlies kan dan een veelvoud van de behoefte worden. De kern houdt de letterlijke uitkomst aan. Boven 3× de behoefte meldt ze de niet-blokkerende waarschuwing `cooling_emission_loss_singular` (pad `cooling.months[i]`), die het paneel toont.
- **Voorraadvat bij een getest toestel.** Opmerking 1 van §13.6.2 (p. 566) zet het vatverlies alleen op nul als een toestel volgens 13.8.4.2/13.8.4.3 samen met het vat is getest. Met `notInApplianceTest: true` per vat is een apart vat nu toegestaan bij `measured_two_profiles`, `heat_pump_en16147` of een gastoestel met bijlage T; het verlies wordt dan berekend. Bij de tabelwaarden (13.25, forfaitair) blijft een apart vat geblokkeerd.
- **Gemeten H_sto;ls.** Volgens p. 568 wordt H_sto;ls naar boven afgerond volgens bijlage X. Dat gebeurt nu ook bij `measured` (eerder alleen bij zonnevaten). Er is een nieuwe variant `measured_standby` voor 13.60, die H_sto;ls afleidt uit Q_stb;ls;ref, ϑ_sto;set;ref en ϑ_amb;ref. X.2 neemt letterlijk "het eerstvolgende hogere getal", ook als de waarde al in tabel X.1 staat (1,2 → 1,3).
- **NEN-EN 16147 bij één tappatroon (13.160b).** De correcties 13.153b (SCF, smart = 1 vanaf 0,07) en 13.153c (T_max;test, T_set;design, standaard 55 °C) zijn nu optionele invoer: `smartControlFactor`, `maxTestTemperatureC` en `designSetTemperatureC`. Zonder die invoer wordt `inputKwhPerDay` ongecorrigeerd gebruikt. C_W;mixed air hoort bij combiwarmtepompen op mengluchtbronnen; die route volgt 13.8.4.2 (`measured_two_profiles.mixedAir`).
- **Randen van tabel 13.18.**
  - Onder 765 kWh geldt de eerste kolom: de klasse wordt niet overschreden en de tabel heeft geen lagere kolom.
  - Boven de laatste kolom van het gemeten profiel geeft de kern `hot_water_heat_pump_class_exceeded`. P. 630 verbiedt gebruik in een hogere klasse dan de gemeten klasse, en voor profielen groter dan XL bestaan geen factoren.
- **Circulatie met standaardwaarden.** Ontbreekt de lengte (13.31) of de diameter/Ψ (13.29/tabel 13.4), en is het jaarlijkse η_W;dis lager dan 0,2? Dan meldt de kern de niet-blokkerende waarschuwing `hot_water_circulation_defaults_low_efficiency`. De berekening zelf blijft normconform.
- Niet nagerekend: koelmethode 1 (NEN-EN 14825, 10.53–10.64); zie de volgende alinea.

**Herberekening van bijlage Q (verwarming) en koelmethode 1 (H10), 3 oktober 2026.** Eigen Python-implementatie vanuit de normpagina's (p. 340, 1028–1065 en 402–409). Getest zijn negen warmtepompscenario's: lucht/water modulerend en aan/uit, brine/water forfaitair en modulerend, lucht/lucht en bivalente gevallen. Voor koeling zijn een kamerairco zonder en een koelmachine met vijfde meetpunt getest. Alle bin- en maandtermen kwamen overeen (≈ 1e-12). Vastgelegd of aangepast:
- **Bronpomp dubbel geteld (normpunt).** Q.4 (p. 1029) rekent W_H;aux;hp;an mee in η_H;gen;hp, en 9.6.3.2 (p. 340) boekt hem nogmaals als hulpenergie. De kern volgt beide letterlijk. Effect: ongeveer 5 % van het warmtepompverbruik bij brine/water aan/uit, 0,3 % bij een modulerende bronpomp.
- **Q.4.4 (p. 1065).** Het forfaitaire bronpompvermogen van water/water rekent met het verdampervermogen bij conditie 1, P·(1 − 1/COP). De norm noemt de conditie niet.
- **Q.1 bij volledige dekking.** Dekt de warmtepomp elke bin, dan geldt F_H;gen = 1, met of zonder bijverwarming. De afronding van tabel Q.6 zou anders een restaandeel van 0,007 % aan de bijverwarming geven.
- **Lucht/luchtwarmtepomp in een waterketen** wordt afgewezen (`annex_q_air_air_hydronic_chain`).
- **Deellast boven 100 % (10.56/10.58, p. 403, normpunt).** De norm begrenst f_C;PL niet. Bij een te klein toestel extrapoleert de kubische functie van 10.63 (in het testgeval f_EER 18,9 bij 324 % deellast). De kern houdt de letterlijke uitkomst aan en meldt `cooling_part_load_above_full_load`.
- **10.63/10.64 (p. 408–409).** Het vijfde meetpunt moet de deellast van C en de condensorintrede van A hebben. Zonder vijfde punt moet de verdamperuittrede bij A en C gelijk zijn, anders is 10.64 met Δϑ_corr = 0 niet consistent. Beide worden nu gecontroleerd (tolerantie 0,5).
- **§10.5.4 (p. 401).** Methode 1 is voor modulerende opwekkers; de kern eist een minimumvermogen onder het nominale vermogen.
- D.7 (p. 793) zoals gedrukt: de gewichten tellen niet op tot 1 en de breedte van de randisolatie ontbreekt (normfout, letterlijk gevolgd);
- tabel D.1 (p. 792): de grens R_n ≥ 2,0 alleen bij horizontale randisolatie;
- C.12: R_cav;sv = 2·R_zv − R_nv; tabel C.4 omlaag zonder interpolatie (lagere rij).

**Herberekening van PV (H16), zonneboilers en micro-WKK, 3 oktober 2026.** Eigen Python-implementatie vanuit de normpagina's. De tabellen 17.2, 17.12 en 17.15 zijn zelf uit de pdf gelezen. Getest zijn:
- PV: 16.2–16.4b, PVT (16.10) en de collectieve verdeling;
- zonneboilers: de berekende route (13.72–13.127) en de geteste route (13.128–13.140), voor voorverwarmer en geïntegreerde naverwarming;
- micro-WKK: 9.66–9.83 en 16.15.

Alle maandtermen kwamen overeen. Vastgelegd of aangepast:
- **9.66/16.15: warmte boven vollast (gecorrigeerd).** 9.66 (p. 347) begrenst P_th;gen;out op P_th;chp_100+sup_100. 16.15 (p. 687) rekent alleen P_el·t bij dat begrensde vermogen.
  - De kern boekte het overschot met η_el van het vollastpunt, en dus met elektriciteit. Voorbeeld: Stirling, 9 000 kWh in 300 h → 391 kWh in plaats van 313 kWh (+25 %).
  - Nu levert het overschot geen elektriciteit.
  - De norm wijst het overschot aan geen opwekker toe. Letterlijk zou het zelfs geen brandstof kosten. De kern boekt de invoer daarom als bijstookwarmte tegen f_prac·η_th;chp_100+sup_100 (interpretatie). Ze meldt dan de waarschuwing `micro_chp_capacity_exceeded`, ook in tapwater wanneer f_func van 13.183 begrenst.
- **16.15: f_gebouw bij een collectieve WKK (interpretatie, ongewijzigd).** De gedrukte 16.15 heeft geen f_gebouw. 9.66 rekent echter met de output van de hele installatie (Q_H;gen;j;out/f_gebouw), dus P_el·t is de productie van de hele installatie. Zonder f_gebouw zou één gebouwdeel alle elektriciteit krijgen.
- **Nummering van de methoden (normpunt).** Hoofdstuk 9 noemt de forfaitaire route methode 2 (9.6.6.1) en de NEN-EN 50465-route methode 1 (9.6.6.2); §16.4.2/16.4.3 nummeren andersom. De kern volgt hoofdstuk 9. §13.8.4.8.3/13.8.4.8.4 (p. 651) verwijzen voor de WKK naar 9.6.5.2; dat lezen we als 9.6.6.2.
- **Oriëntatie precies tussen twee tabelrichtingen (17.3.7, p. 749).** De norm staat de hoogste naastliggende belemmeringswaarde toe ("mag"). De kern nam altijd de rechtsom liggende buur en neemt nu per maand de hoogste, net als bij I_sol in 17.2 (p. 694, waar het "moet"). Voorbeeld: 22,5° bij 45° met een dakrand gaf 476,6 kWh; nu de hogere waarde van N en NO.
- **13.134: geteste zonneboiler met geïntegreerde naverwarming (normpunt, letterlijk met waarschuwing).** Q_W;ren = Q_W;use − f_dis·Q_W;bu;out (p. 602). Bij Σ f_dis < 1, dus een slechtere oriëntatie of meer belemmering dan zuid 45° vrij, krimpt het naverwarmingsdeel en stijgt de zonne-opbrengst. Volledig belemmerd zuid 37,5° geeft 1 920 in plaats van 931 kWh/jr. De kern houdt de formule aan en meldt `solar_tested_backup_distribution_below_one`.
- **Tabel 16.3 onder F_sh;obst 0,80 (interpretatie).** De tabel loopt van 0,80 tot 1,00 en geeft geen regel daarbuiten. De kern houdt 0,75 vast, zoals andere tabellen aan hun randen. De opbrengst van 16.2/16.3 daalt wel verder met F_sh;obst zelf.
- **Nieuwe controles:**
  - η_th + η_el ≤ 1,2 per meetpunt (tabel 9.33, `micro_chp_total_efficiency_invalid`);
  - P_el en η_el van één meetpunt moeten op 2 % na overeenkomen (`micro_chp_electric_values_inconsistent`);
  - een PVT-paneel en een PVT-zonneboiler moeten elkaar in het gebouw hebben, met een passende afdekking (`pvt_without_thermal_part`, `pvt_without_electric_part`, `pvt_cover_inconsistent`; niet-blokkerend, omdat de invoer geen koppeling heeft).

**Herberekening van hoofdstuk 8 (transmissie, grond, onverwarmde ruimten), 3 oktober 2026.** Eigen Python-implementatie vanuit de normpagina's, vergeleken via `assess_project_performance`. Getest zijn:
- grond: vloer op staal, kruipruimte (geventileerd en niet), onverwarmde en verwarmde kelder, randisolatie verticaal en horizontaal, gedetailleerde ψ (8.30–8.49, D.1–D.16);
- onverwarmde ruimten: b_U (8.53, 8.57–8.59), forfait H_ue (I.8), serre plus trappenhuis;
- thermische bruggen en constructies: ΔU_for (8.3), spouwmuren (C.3, tabel C.3), zolder (C.5), geventileerd trappenhuis (C.15), afschotdaken (C.17–C.20), samengestelde bovengrens (C.5);
- H_tr en de maandbehoefte van de tussenwoning.

Alles kwam op ongeveer 1e-6 overeen, op één punt na. Vastgelegd of aangepast:
- **U_f boven een kruipruimte of onverwarmde kelder (gecorrigeerd).** 8.43 (p. 258) neemt U_f volgens 8.2.2.2.1 (8.6, p. 229), dus met R_se = 0,04 uit tabel C.2 (p. 778). §8.3 geeft geen afwijking; de R_si-regel van 8.4.2.1 hoort bij H_D;zi,j;ztu. De kern gebruikte 0,17 en volgt nu de tekst. Geventileerde kruipruimte: H_g 13,40 → 13,59 W/K, Q_H;nd ongeveer +0,2 %.
- **H_D;zi,j;ztu met R_si (gecorrigeerd).** 8.4.2.1 (p. 266) vervangt R_se door de R_si van de onverwarmde ruimte. De projectroute gebruikte de U voor buitenlucht ongewijzigd. Ze rekent die nu om met R_si 0,13 (wand), 0,10 (dak/plafond) of 0,17 (vloer).
- **Samengestelde constructie met een sterk geventileerde spouw.** Werd geweigerd (`composite_layer_requires_thickness`). Nu toegestaan, met de afkapping van C.3.3 in beide grenzen.
- **Ongeldige grondcombinaties.** Een kruipruimte met randisolatie, of een kruipruimte met een verwarmde kelder, gaf alleen de status `invalid`. Nu staan ze als gat in `gaps`: `ground_floor_edge_insulation_slab_only` en `ground_floor_below_and_heated_basement`.
- **Nieuwe waarschuwingen:** `ground_floor_resistance_below_surface_resistance`, `ground_floor_perimeter_implausible`, `detailed_thermal_bridges_none_entered` en `sunroom_values_differ_from_unheated_space`.
- **D.7, tabel D.1, C.12 en tabel C.4:** vastgelegd als interpretatie (zie de lijst bovenaan, en `nta8800-maandbehoefte.md` en `nta8800-constructies.md`).
- **Voorbeeld tussenwoning:** het dak was 52 m² bij 45° op een plattegrond van 5 × 10 m. Dat moet 70,7 m² zijn. Het dak bestaat nu uit twee schilden van 35,36 m² (noord en zuid).

**Herberekening woningventilatie, BENG 1 en TOjuli, 3 oktober 2026.** Eigen Python-implementatie vanuit de normpagina's, 23 varianten (systemen A, C1, C.4b, D met volledige en gedeeltelijke bypass, E, roostervoorverwarming, forfaitaire infiltratie, actieve koeling met bijlage AA). Hoofdstuk 11 (11.1–11.142), 7.19/7.20, de maandbehoefte, BENG 1 met de vaste C1-run (§5.4.3.1), de grenswaarde en TOjuli (5.40–5.41c, bijlage AA) kwamen op afrondingsniveau overeen. Vastgelegd of aangepast:
- **`θ_e;avg;an` (interpretatie, gewijzigd).** 7.14/7.15/7.73 verwijzen naar §17.2, maar tabel 17.1 geeft alleen maandwaarden (p. 689). De enige jaarwaarde die de norm noemt, is θ_e = 10,67 °C onder D.4 (p. 791, op basis van NEN 5060). De transmissieterm en 7.73 gebruikten het ongewogen gemiddelde van tabel 17.1 (10,6717 °C), D.4 al 10,67 °C. Nu gebruikt alles 10,67 °C. Het effect is verwaarloosbaar (0,0017 K). Het uurgewogen gemiddelde (10,7023 °C) zou BENG 1 van de basistussenwoning met 0,02 kWh/m² verlagen.
- **Bijlage AA met onvoldoende capaciteit (gecorrigeerd).** §5.7.1 (p. 114–115): alleen een koelsysteem met voldoende capaciteit geeft vrijstelling; "voor alle andere systemen en situaties" moet TOjuli worden bepaald. De kern gaf TOjuli = geen waarde. Nu volgt TOjuli uit 5.40 zoals zonder actieve koeling, met de niet-blokkerende waarschuwing `annex_aa_capacity_insufficient` (in `warnings` van de rekenzone en van het gebouw).
- **Raam-id in bijlage AA (gecorrigeerd).** De afgeleide rekenzone noemt projectramen `window:<id>`; het projectraam-id werd niet gevonden. Beide schrijfwijzen worden nu herkend. Een afgewezen bewijs voor actieve koeling (bijvoorbeeld `annex_aa_window_unknown`) staat nu als gat in de projectuitkomst, met de rekenzone in `detail`.
- **Zonwering als bewijs (methode 3, tweede situatie).** Dit was alleen een verklaring. Nu toetst de kern de raamgegevens: meer dan 95 % van het beoordeelde glasoppervlak (oriëntatie 45°–315° en horizontaal) moet niet-beweegbare lamellen van tabel 7.4a/7.4b hebben, `g_gl ≤ 0,4` (7.40/7.41/7.41a/7.41b, juli, koeling) of `F_sh;obst;juli < 0,67`. Anders `solar_limitation_not_met`; een onvolledige raaminventaris geeft `window_inventory_incomplete`. Beweegbare zonwering telt niet mee: tabel 7.4a/7.4b gaan over niet-beweegbare zonwering.
- **`ventilation.months[].conductanceWPerK`.** Dit is ρ·c·Σq zonder b_v; de toevoertemperaturen dragen b_v. Het is dus niet de H_ve van 7.19 (bij D met WTW 68,8 tegen 32,1 W/K). Nieuw veld `weightedConductanceWPerK` geeft H_ve volgens 7.19/7.20. Het paneel en het rapport tonen nu die waarde.

**Herberekening van hoofdstuk 12 en bijlage P, 3 oktober 2026.** Eigen Python-implementatie vanuit de normpagina's. Bevochtiging (12.1–12.5, tabellen 12.1/12.2), een warmtenet met warmtepomp, WKK en ketel (P.13–P.18 leidingdelen, P.25 cascade, P.26–P.30, P.56–P.60, P.72–P.83) en een koudenet (P.51–P.55, P.66–P.70) kwamen op ongeveer 1e-12 overeen. Vastgelegd of aangepast:
- **P.78: ontvochtiging met alleen een jaarwaarde (gecorrigeerd).** De kern liet dan het hele maandprofiel van het koudenet vallen, zodat P.68 `monthly_cold_delivery_required` gaf en de standby van P.69 alle maanden liep. P.78 (p. 1022) telt alleen E_C;dc;mi; ontvochtiging telt nu alleen in de jaarwaarde (P.77).
- **P.73/P.74: één deel met alleen een jaarwaarde (gecorrigeerd).** Een sorptiedeel met alleen een jaarwaarde verving de maandwaarden van alle percelen door het profiel van P.74. Nu krijgt alleen dat deel P.74; de opgegeven maanden blijven.
- **f_Pren;dc (gewijzigd).** §5.8.3.2 (p. 129) schrijft bij 5.49 geen afronding voor, anders dan 5.42 en 5.50. De kern rondde naar beneden af; nu niet meer.
- **`-0,0` in de uitvoer (gecorrigeerd).** Een primaire factor 0 kwam als `-0.0` uit de afronding naar boven; nu 0.
- **Nieuwe meldingen:** `chp_co2_factor_negative` (P.27 met biogas), `cold_network_gain_outside_cooling_months` (leidingen van een koudenet die warmte winnen in maanden zonder koudelevering) en `humidifier_zone_duplicate` (§12.1: één systeem per rekenzone, blokkerend).
- **Nieuwe invoer:** `sourcePumpIncluded` bij een opgegeven warmtepomprendement (P.6.8.4.3, p. 1009: 0 W/kW als de bronpomp in het rendement zit) en `servedAreaM2` bij een bevochtiger (12.2.1).

**Onafhankelijke reviews.** Elk hoofdstuk is door een tweede, onafhankelijke controle tegen de gerenderde normpagina's gelegd. De fouten die daaruit kwamen, zijn hersteld en staan in de secties hieronder en in de moduledocumentatie. Voorbeelden:

- tabel 11.19 een kolom verschoven;
- het kwadraat in 11.137;
- 9.29 te ruim gelezen;
- tabelinterpolatie bij zonwering;
- het ontbreken van §7.9.

**Wat nog ontbreekt voor certificering.**

1. **Officiële referentie-uitkomsten.** De BRL 9501-testset of de bijlage van ISSO 54 met verwachte resultaten ontbreekt. De ISSO-boeken bevatten geen volledig doorgerekend voorbeeld. Alle uitkomsten zijn dus `referenceVerified: false`.
2. **Open interpretatievragen.** Ze staan per module in de documentatie en in de uitvoer `interpretations`. De belangrijkste: de letterlijke lezing van 9.29, de factor 15 % in formule 10.15, en de kolom "Juli/september" van tabel 10.16.
3. **Niet ondersteund (met expliciete foutcode).**
   - het specifieke gastoestel uit tabel 11.11, voetnoot a (de norm geeft geen rekenwaarden);
   - een wisselstroomventilator van na 2006 in de forfaitaire methode (tabel 11.23 geeft geen waarde);
   - de onderdelen met de status "in uitvoering" in de tabel hierboven.
   - Registratie in EP-Online vereist de uitwisselspecificatie van RVO.
4. **Visuele acceptatie** van de desktop-UI en de attestprocedure (BRL 9501:2026, ISSO 54 actuele editie).


## Tweede onafhankelijke review (2 oktober 2026, nacht)

Vier leesreviews hebben de kern opnieuw tegen de normtekst gelegd, zonder de code te wijzigen. Elke review rekende eigen voorbeelden onafhankelijk na in Python.

- **Constructies, bijlagen C, E, F, G, H, I en L.** Alle tabellen komen cel voor cel overeen, en alle uitgewerkte constructies zijn gelijk tot 1e-15. Wel zijn er afwijkingen gevonden:
  - R1 en RT voor ΔU bij samengestelde constructies;
  - de verouderingsfactor van in-situ isolatie;
  - de afronding van U in 8.1;
  - R_se bij een opwaartse warmtestroom;
  - de afronding van R_calc;
  - R_si van zoldervloeren;
  - thermische kussens;
  - een aantal ontbrekende validaties.
- **Bijlagen P, T, U, AA, A en B.** Alle tabellen en formules kloppen. De afwijkingen zitten in:
  - het opslagrendement van WD;
  - de forfaitaire voorwaarden van P.0 en P.5;
  - het hernieuwbare aandeel volgens 5.39g (absorptiekoeling en f_BACS);
  - WKK op biogas;
  - de klassecontroles van bijlage U en T;
  - dynamische beglazing in AA en TOjuli;
  - de interne warmte in TOjuli.
- **Bijlagen Q, V, W, M, N en O.** De modules zijn gelijk tot 1e-9. De afwijkingen zitten in de koppeling:
  - f_prac 0,95 ontbreekt in 9.63;
  - de brandstof van bijlage N staat op onderwaarde in plaats van bovenwaarde;
  - de dekkingstolerantie van Q.6;
  - de hulpenergie van de warmtepomp;
  - c_source;
  - 13.164;
  - de omgevingstemperatuur in M.12;
  - N.69.
- **Basisopname ISSO 82.1 en 75.1.** Zes fixtures geven plausibele labels. De drie belangrijkste fouten:
  - water- en bodemwarmtepompen worden afgewezen;
  - een elektrische boiler wordt afgewezen;
  - een vloer boven buitenlucht krijgt de verkeerde Rc-rij.

  Daarnaast ontbreken nog boekregels, zoals leidingdoorvoeren H_p, leidingen in onverwarmde ruimten en het naïsolatiejaar.

De vraag welke R_se onder een vloer boven een kruipruimte of kelder geldt, was beslist op 0,17 (R_si omlaag). Op 3 oktober 2026 is dat herzien naar de normtekst: 8.43 verwijst via 8.2.2.2.1 naar tabel C.2, en die geeft R_se = 0,04 zonder afwijking voor dit geval. Zie de herberekening van hoofdstuk 8 hierboven.

Het interpretatiedocument NTA 8800:2024/INT-V1:2024 is nagelopen. De correcties staan al in de doeleditie 2025+C1:2026:
- buffervat zonder terugwinbare verliezen bij 9.5;
- tabel 9.4 en 9.16;
- θ_H,max;si,mi in 13.141b;
- 13.96 en 13.185 zonder de vervallen termen.

Het document is daarom niet apart toegepast.

Elke review wordt in een eigen werkboom hersteld en via de poort samengevoegd. In deze ronde zijn ook toegevoegd: de LBK met naverwarmer en koelbatterij (tabel 11.15, 11.114–11.121) en een expliciete fout wanneer de drukbalans van 11.2.1.6 niet convergeert.

## Stand 2 oktober 2026: externe levering volgens §5.8 en bijlage P

De weigering van kwaliteitsverklaringen voor externe warmte en koude en van collectieve warmtepompbronnen is vervangen door de route van §5.8 en bijlage P (pagina's 128–138 en 940–1012), module `annex_p`:

- per drager dh, dw en dc een verklaarde, berekende (P.7/P.9) of gemeten (P.6) waarde voor `f_P;del`, `f_Pren` en `K_CO2`, met de afrondingen van P.6.1.2.1 en §5.8.3;
- EMGverklaring en EMGforf (§5.3.1) als scenariopaar in de indicatoren;
- collectieve warmtepompbron met `Q_HD;hp;in;bron` volgens 9.6.8.1.1.2.3 en de forfaits `f_P;el/23`, `K_CO2;el/23` en `f_Pren` 0,95 onder 20 °C;
- externe koude telt nu ook in de CO2-emissie (`K_CO2;el/3`), en tapwater uit een warmtenet heeft een eigen drager `dw`.

Interpretatievragen:

1. Formule 5.48 geeft `f_Pren` = 0,95 voor geothermie bij `η` = 20. De kern past `1 − 1/η` toe met het `η` van P.6.5.4.8 (`20·Δθ/40`), net als bij warmtepompen (5.44).
2. P.6.5.4.6 noemt voor warmte uit een AVI met elektriciteitsproductie `f_P;del;wi`. De kern gebruikt daar de WKK-met-derving-route met `f_P;del;wi` = 0,5 en `f_Pren` = 0,5 (5.46). Bij een stoomketel zonder elektriciteitsproductie gebruikt hij de verbrandingsroute.
3. Bij een gedeeltelijk directe hernieuwbare elektriciteitsvoorziening worden `f_P;del;el` en `K_CO2;el` naar het aandeel gewogen (P.4.5 "gewogen waarde"). Het aandeel telt als `W_gen;ren` en `W_aux;ren` in 5.42.
4. `Q_HD;hp;in;bron` gebruikt de jaargemiddelde COP van de keten in plaats van een COP per maand.
5. EMGforf voor een collectieve bron vanaf 20 °C met verklaring gebruikt tabel 5.2–5.4. De uitzondering in §5.3.1 geldt alleen onder 20 °C.
6. P.6 noemt geen `f_Pren`. Bij de gemeten route wordt `f_Pren;dX` opgegeven, volgens 5.42/5.49/5.50 zoals vastgelegd voor het systeem.

De onderdelen die hier nog ontbraken, zijn op 3 oktober 2026 toegevoegd (zie hieronder).

## Stand 3 oktober 2026: berekende route van bijlage P compleet

De berekende route rekent nu vanuit gebiedsgegevens (p. 957–1026). Opgegeven waarden gaan voor:

- behoefte van het gebied P.72–P.83 met tabellen P.14/P.15 en de maandverdeling P.74;
- leidingverliezen P.13–P.18 met tabellen P.1 en P.16, buffervaten P.43/P.44;
- het opslagrendement voor tapwater uit P.43–P.46 met tabel P.7;
- energiefracties: β met tabel P.2 en de cascade (P.23–P.25), tapwater P.40–P.42, koude P.51–P.55 met tabel P.8, en de collectieve zonnecollector P.32/P.33;
- forfaitaire rendementen: tabellen P.3/P.4 (ketels), P.6 (WKK), P.9/P.10 (koude), vaste biomassa P.6.5.4.3 en de flexmodus P.6.5.4.11 met tabellen 5.5/5.6;
- hulpenergie P.56–P.70 met tabellen P.11–P.13;
- P.71 als rapportage van de elektriciteitsproductie in het gebied.

Interpretatievragen:

7. P.62 (`t_on × ΣP` in W) en P.65 (`1 000 × P` in de noemer) zijn in de gedrukte norm dimensioneel anders dan P.57 en P.60. De kern rekent ze zoals P.57/P.60: kWh = h·W/1 000, en uren = kWh/kW.
8. P.74 geeft negatieve maandwaarden als `θ_e;avg;mi` boven 18 °C ligt (juli en augustus in tabel 17.1). De kern gebruikt `max(0, 18 − θ_e)` in teller en noemer, zodat het jaartotaal klopt. Hebben sommige percelen wel maandwaarden, dan verdeelt P.74 alleen de delen zonder maandwaarden; de opgegeven maanden blijven staan.
9. P.53 noemt het rendement "van de preferente koelmachine". De kern neemt per compressiekoelmachine haar eigen rendement. Bij een gasmotorkoelmachine geldt `η_CD;gen = COP·η_ge` op de brandstof; het asvermogen `P_in` krijgt daarom de COP van tabel P.9 zelf (p. 1000). Koelmachines vormen alleen één voorkeursgroep bij gelijk rendement én gelijke energiedrager (P.6.7.3).
10. De voorkeursregels a)–e) staan alleen bij warmte (P.6.5.3.2). De kern past ze ook toe op tapwater. Zonder rendement om op te rangschikken vraagt hij `priority`. Regel a) noemt hernieuwbare opwekkers met "zoals" (p. 966); vaste biomassa telt daarom als hernieuwbaar en gaat voor. Een elektrodeketel in flexmodus draait alleen in de flexuren en komt als laatste, in plaats van met rendement 0,99 regel e) te winnen. `priority` gaat voor beide keuzes.
11. Zonder maandwaarden verdeelt de kern de levering van tapwater naar maandlengte, net als P.82. Voor koude is er dan geen maandverdeling, en staat `f_on;mi` in elke maand op 1.
12. Bij de hulpenergie per opwekker krijgen restwarmte, geothermie, voorgeschakelde systemen, STEG en AVI geen eigen term. Hun hulpenergie zit al in de factor (P.6.5.4.7, P.6.8.4.1, P.6.9.4.1). Een warmtepomp met tabel P.5 heeft 0 W/kW voor de bronpomp; met een opgegeven rendement is dat 10 W/kW.
13. Bij de collectieve zonnecollector (13.7.2.2 op `Q_in;mi`) gelden `θ_X;low` als de omgevingstemperatuur van de opslag, `θ_X;high` als de retourtemperatuur en de bijstookset als de aanvoertemperatuur van het net. Hiervoor zijn de instellingen voor ruimteverwarming als voorbeeld genomen.
14. Een ketel met vollastrendement wordt bij tapwater alleen als niet-preferent toestel met 5 punten verlaagd (P.6.6.5.2). De norm schrijft "aftrek van 5 %"; de kern leest dat als 5 procentpunt, net als bij warmte. Een vollastrendement op een preferente tapwaterketel wordt geweigerd (`hot_water_full_load_requires_non_preferred`); die rekent met tabel P.3 HT. Een ketel buiten wordt bij warmte en tapwater eenmalig met 5 punten verlaagd.
15. P.71 staat los van P.4.5. De productie wordt gerapporteerd; het direct hernieuwbare aandeel van de aandrijving blijft een opgave.
16. P.25 rekent met `Q_HD;in;tot`, de warmte van alle opwekkers samen (p. 968). De kern neemt dat letterlijk, dus inclusief de bijdrage van collectieve zonnecollectoren.
17. De flexmodus begrenst de flexwarmte op 15 % van de productie van het net (5.8). Met `networkProductionKwh` geldt de opgegeven netproductie (warmte en tapwater samen); zonder die opgave neemt de kern de warmte van de functie die wordt berekend. `f_Pren` van een flexopwekker wordt op 0 begrensd: onder het omslagpunt is de elektriciteit buiten de flexuren groter dan de warmte, en dat is geen negatief hernieuwbaar aandeel.
18. Hulpenergie tapwater (P.6.9.4.3, p. 1013): met `alsoServesHeating` is de stand-by 0 W, omdat die bij verwarming hoort (P.6.8.4.1 a). Met `withoutAuxiliaryEnergy` (zoals een traditionele gasketel) zijn stand-by en ventilator 0 W en 0 W/kW.
19. Afwijkend van tabel P.1: het geval "één leiding in een sleuf" (`single_pipe_in_trench`, f_x 1,00) en de opgegeven `groundConductivity`/`surfaceCoefficient`. De norm noemt onder P.59 formule P.32 waar P.33 (`Q_XD;in;mi`) bedoeld is (p. 1008); de kern gebruikt P.33.

## Stand 2 oktober 2026: bijlagen AA, A en B, en TO-juli stap B

- **Bijlage AA** (`annex_aa`, p. 1135–1147) berekent per verblijfsruimte en per rekenzone de maatgevende koelbehoefte (AA.1–AA.9, tabellen AA.1–AA.3) en toetst de opwekker (AA.10/AA.11) en de afgifte per verblijfsruimte (AA.12/AA.13). Het capaciteitsbewijs `annex_aa` voor actieve koeling (§5.7.1) vraagt nu een `calculation`. Ontbreekt die, dan volgt `annex_aa_calculation_required`. Is de capaciteit onvoldoende, dan volgt `annex_aa_capacity_insufficient`. Het resultaat staat in `annexAa` van de TO-juli-uitkomst.
- **TO-juli stap B**: de juliwaarden van `Q_H;ls;rbl` (uit de verwarmingsketen) en `Q_C;ls;rbl` worden naar `A_T;or` verdeeld en gaan via 7.7/7.8 met Δη in de behoefte per oriëntatie. `Q_C;ls;rbl` is in de kern nog 0.
- **Bijlage A** (`annex_a`): dynamische ramen met maandgewogen g en U (methode A, stap 1) of één toestand (methode B).
- **Bijlage B** (`annex_b`): effectieve interne warmtecapaciteit als alternatief voor tabel 7.10.

Interpretatievragen:

1. AA.2.2.2 noemt `q_v;C;SUP;eff`. Met hoofdstuk 11-invoer gebruikt de kern infiltratie, natuurlijke toevoer en mechanische toevoer van de julikoudebalans. Zonder hoofdstuk 11 gebruikt hij de som van de koelgeleidingen van alle opgegeven stromen, ook spuien en verbrandingslucht.
2. Stap 1: bij gelijke zoninstraling op meerdere uren neemt de kern het vroegste uur. Een verblijfsruimte zonder ramen gebruikt het piekuur van de zone voor θ_e.
3. P_gl van de zone (AA.7) gebruikt θ_e op het piekuur van de zone; P_gl per verblijfsruimte gebruikt het eigen piekuur ("analoog").
4. AA.7: zonder opgegeven `U_w+shut` rekent de kern met `U_w`. Dat is conservatief, omdat θ_e > 24 °C.
5. AA.6b: `g_gl;C;juli` = F_w·g_gl;n (7.40). `F_C` is de reductiefactor van de beweegbare zonwering zonder de tijdfractie van tabel 7.7 (opmerking 3).
6. Tabel AA.2 heeft overlappende grenzen (≤ 1975, 1975 ≤ 1992, …). De kern hanteert ≤ 1975, 1976–1992, 1993–2015 en > 2015.
7. Bijlage A: de stap-2-correctiefactoren zijn 1, omdat er geen forfait is.
8. Bijlage B: een open vrijhangend plafond telt niet mee, voor de weerstand noch voor de massa.

## Stand 2 oktober 2026: bijlagen M, N en O

Toegevoegd uit de gelicentieerde normtekst (p. 359–361, 870–931): ketels met productwaarden (bijlage M), lokale verwarmers, luchtverwarmers, stralers en kachels (bijlage N), hulpenergie van individuele toestellen uit componentmetingen (bijlage O met 9.86–9.90) en de forfaitaire "overige systemen" van tabel 9.25. Het controleren van de forfaitroutes leverde het volgende op: tabel 9.25 (ketels) en tabel 9.30 (biomassa) komen overeen met de code. De luchtverwarmer- en lokale-verwarmingsrijen van tabel 9.25 ontbraken en zijn toegevoegd.

Interpretatievragen:

1. M.15 telt `Q_gen;aux;rbl` op bij het terugwinbare mantelverlies, en M.20 stelt die gelijk aan `Q_H;gen;aux;rvd` (naar het medium). Dat deel is al in M.1 afgetrokken. De kern rapporteert als terugwinbaar naar de ruimte daarom M.16 + M.19 (`Q_H;gen;aux;rbl`).
2. M.12 noemt `η_gen;Pn` zonder temperatuurcorrectie. Voor condenserende ketels is `η_gen;Pn;60` gebruikt.
3. `t_H;op;si;mi` in M.6, M.23 en M.25 is "gegeven in tabel 9.15". De kern neemt de uren van tabel 9.15 bij de stookgrens, zonder `f_H;red` en zonder de pompfactor van 9.32a. Bij meerdere zones gelden de langste uren en een oppervlaktegewogen retourtemperatuur.
4. Een collectieve ketel met productwaarden rekent de deellast met de output van de hele installatie (`/f_gebouw`) en schaalt de uitkomsten terug.
5. N.37 drukt `Q_lrh;blw;sby` in de teller; gelezen als `Q_lrh;sby;rh` (N.32). N.61 drukt `100 + (1 + …)` in de noemer, terwijl N.37 `100·(1 + …)` heeft; gelezen als `100·`.
6. N.69 interpoleert tussen `α_ch;ON;Pmin` en `α_ch;ON;Pn`. De kern interpoleert tussen de totale verliezen `α_ON;Pmin` en `α_ON;Pmax` (N.49/N.50), zodat ventilatie-, mantel- en condensatieterm meetellen. In de modulatiestand is de schoorsteencorrectie met β = 1 bepaald.
7. f_prac 0,95 (N.1.1, N.5) is toegepast als `E_H;gen;in / 0,95`.
8. Kachels: tabel N.33 verwijst voor `f_corr`, `ϑ_test` en `n` naar "forfaitaire waarden", maar tabel N.20 en N.22 hebben geen kachelrij. De kern gebruikt dan `f_corr` = 0 en `n` = 0, en hulpvermogens 0 tenzij opgegeven.
9. Een kachel met watergedragen aansluiting levert `Q_H;gen;out;w` (N.80) alleen als rapportage; die warmte wordt niet verrekend met een andere opwekker.
10. De invoer van bijlage M is op bovenwaarde (`f_Hs/Hi`, tabel M.3); de dragerhoeveelheden van de forfaitroute nemen de rendementen van tabel 9.25 en 9.30 ongewijzigd.

## Stand 2 oktober 2026: bijlagen Q, V en W

Bijlage Q (warmtepomp met productgegevens), bijlage V (regeneratie) en bijlage W (boosterwarmtepomp) zijn getranscribeerd (p. 1027–1069, 1114–1128) en gekoppeld aan de verwarmings- en tapwaterketen. Zie [bijlagen Q, V en W](nta8800-bijlagen-q-v-w.md).

Nieuwe interpretatievragen:

1. Q.48: −10 °C als nominale verdampertemperatuur voor L/W (de tekst noemt L/L).
2. L/L: condensorintrede 20 °C.
3. Interval 5 (B < 0,15) gebruikt het 15 %-deellastpunt.
4. De jaarlijkse energiefractie `F_H;gen` wordt per maand toegepast.
5. Bijlage V in de ruimteverwarmingsketen telt alleen de ruimteverwarmingswarmte in de noemer van V.1.
6. W.3: `Q_C;HP;si;mi` wordt opgegeven en niet uit hoofdstuk 10 overgenomen; de koelketen en de booster moeten dezelfde waarde krijgen.

## Stand 2 oktober 2026: review H7, H8 en H17 tegen de normtekst

Hoofdstuk 7, §8.3 met bijlage D en hoofdstuk 17 zijn nagelopen tegen de gelicentieerde normtekst (NTA 8800:2025+C1:2026, pagina's 164–220, 250–257, 690–715 en 790–795). Tabellen 17.1, 17.2, 17.4 (90°/45°), 7.7 en 7.9 en de meeste constanten klopten. De gevonden afwijkingen zijn hersteld:

- poort 7.2 (`γ_H > 2` → geen warmtebehoefte), 7.48 (`η_H = 1/γ`) en 7.54 (`η_C = 1` bij `γ_C ≤ 0`, in plaats van een weigering);
- aparte tijdconstanten `τ_H`/`τ_C` (7.57/7.58) met de seizoenswaarden `H_H;g;adj`/`H_C;g;adj` (D.2/D.3) en de maandelijkse `H_ve` inclusief `b_v` (7.19/7.20);
- `a_C;red` voor niet-continu koelen (7.7, 7.74/7.75);
- volledige §7.9: intermitterende verwarming 7.59–7.73 met tabel 7.14/7.15, nivellering woningbouw 7.78/7.79 en de rekentemperatuur in transmissie en ventilatie;
- bijlage D: maandelijkse grondoverdracht D.1–D.9 met tabel D.1, en de vloerrandterm van 8.36 (of forfait 8.37);
- tabel 17.4 met alle hellingskolommen (dichtstbijzijnde kolom), tabel 7.7–7.9 met interpolatie en de 180°-kolom, tabel 7.8, en zonwering op de warmtebalans voor utiliteit en niet-ingeregelde automatiek;
- tabel 17.1-kolommen `θ_e;argII`, `u_site` en `θ_ODA;preh;WTWC` als constanten voor hoofdstuk 11;
- TO-juli gebruikt de juli-`H_gr;an`, `H_C;g;adj` in de tijdconstante, `H_C;ve` met `b_v` en `a_C;red`.

Nieuwe verplichte invoer: `usageFunction` (en bij wonen `dwellingType`) per rekenzone en `edgeThermalBridges` per vloer op grond. De expliciete grondroute vraagt nu twaalf `H_g;an;mi` en de twee seizoenswaarden. Terugwinbare systeemverliezen (7.3–7.5, 7.7–7.9) blijven buiten beschouwing en staan in `omittedCorrections`. De interpretatiepunten staan in [maandbehoefte](nta8800-maandbehoefte.md).

Daarmee zijn interpretatievraag 1 en 8 hieronder beantwoord (zonwering op warmte: zie p. 197; `θ_e;avg;an`: toen ongewogen gemiddelde, D.4 gebruikt 10,67 °C; sinds 3 oktober 2026 overal 10,67 °C) en is open punt 6 (`a_H;red`/7.78) opgelost.

## Stand 2 oktober 2026: hoofdstuk 5 en 9 tegen de doeleditie

Een woordvergelijking van het consultatieconcept met NTA 8800:2025+C1:2026 (hoofdstuk 5, pagina's 72–124; hoofdstuk 9, pagina's 290–365) laat zien:

- In hoofdstuk 9 zijn alle formules, tabellen en getallen die de code gebruikt ongewijzigd. Alleen 9.16 heet nu 9.12a.
- In hoofdstuk 5 zijn vier onderdelen inhoudelijk gewijzigd: 5.14a, 5.20a/5.20b, tabel 5.3 en de collectieve warmtepompbron.

Wat daarop in de kern is gebouwd of hersteld:

- **Distributie volledig** (`heating_distribution`, `distributionSystem`):
  - 9.26–9.40: stookgrens met kleinste-kwadratenfit (9.28), bedrijfstijd 9.32a/9.32b met tabel 9.15 en tabel "9.X", watertemperaturen 9.30–9.32 met tabel 9.14, Ψ uit tabel 9.16 of 9.33–9.35, leidinglengtes 9.36/9.27, terugwinbaar deel 9.38;
  - pomp 9.41–9.51;
  - buffervat 9.2.3;
  - terugwinbare verliezen per zone (9.7) als uitvoer.
  - Het verlies wordt niet meer op 0 gezet in maanden zonder warmtebehoefte.
- **Hulpenergie**: 9.85-forfait voor individuele elektrische warmtepompen, ook in een hybride; 9.91/9.92 voor collectieve ketels, collectieve warmtepompen, externe warmte, elektrische verwarming en biomassa. De pompplicht van §9.4.4 wordt afgedwongen.
- **Opslagcorrectie 5.14a/5.14b**: batterijen en thermische opslag geven geen fout meer, maar `f_BAT;cor` = 0 of 1.
- **`f_BACS` = 1,05** alleen bij utiliteitsbouw (§5.5.8).
- **CO2-emissie** volgens §5.5.6.1 met tabel 5.3.
- **5.32**: warmtepomp met gecombineerde bron buitenlucht/ventilatieretourlucht.
- **Weigeringen en checks**:
  - een losse warmtepomp boven 55 °C wordt geweigerd (bijlage Q);
  - een biomassakachel telt alleen als enige verwarming van de ruimten die hij bedient (§9.6.5).

Nieuwe interpretatievragen:

1. Tabel 9.X: `f_H;red;pmp;op` = 0,10 voor een woning met individuele installatie. Letterlijk toegepast in 9.32a geldt die factor ook voor het leidingverlies, niet alleen voor de pomp. Bij individuele gasketels en warmtepompen zit de pomp echter in 9.85; de factor raakt dan alleen het leidingverlies.
2. 9.32a wordt gedrukt als `MAX(…)` met één argument; dat is gelezen als een product.
3. Stookgrens stap 5: "maximale snijpunt: θ_int;set;H;stc" is gelezen als een bovengrens gelijk aan het setpoint.
4. Een stookgrens die niet te bepalen is (stijgende lijn of minder dan twee punten) geeft een invoergat. De norm regelt dit geval niet.
5. Tabel 9.21 heeft geen rij voor een opwekker met `Δθ_g ≤ 10 K` en een afgifte met `Δθ_a > 10 K`. Daarvoor is de eerste rij gebruikt.
6. 9.45 vraagt `t_H;mi;max` en `Δθ_min` in de maand met de hoogste behoefte. Bij meerdere zones neemt de keten de langste bedrijfstijd over de zones.
7. Telt de afleverset van externe warmte als toestel voor de 10 W van 9.6.8.2.3? De gebruiker legt dat vast in `electricallyConnectedDevices`.
8. Losse warmtepomp boven 55 °C: §9.6.3 eist bijlage Q, terwijl tabel 9.27 kolommen tot 70 °C heeft. De kern weigert, conform de tekst.
9. `θ_int;op;H` (7.9.6) als omgevingstemperatuur van zoneleidingen is gelijkgesteld aan het verwarmingssetpoint, totdat §7.9 is gekoppeld.
10. De terugwinbare verliezen worden nog niet teruggekoppeld naar 7.2.1. Die terugkoppeling hoort bij de behoefteberekening; voor BENG 1 moet ze 0 blijven.

## Stand 2 oktober 2026: hoofdstukken 10, 13, 14, 16 en §5.7 tegen de normtekst

Een verificatie tegen de gelicentieerde NTA 8800:2025+C1:2026 vond afwijkingen in koeling, tapwater, PV en TO-juli. Hoofdstuk 14 ontbrak nog. Al deze modules zijn nu opnieuw opgebouwd vanuit de normtekst. Paginaverwijzingen staan in de modulekoppen en in het [bronnenregister](nta8800-bronnenregister.md).

| Module | Wat is veranderd |
|---|---|
| `space_cooling` | De snelkoppeling `Q/(η_em·η_dis·f_reg)` is vervangen door de optelketen van hoofdstuk 10. Die bestaat uit: <ul><li>afgifteverlies 10.15/10.16;</li><li>bedrijfsuren via de koelgrens en tabel 10.6;</li><li>watergedragen distributie 10.21 en pompenergie 10.28–10.38;</li><li>ventilatorconvectoren;</li><li>prioriteit en β volgens tabellen 10.15/10.16;</li><li>methode 3 met alle rijen van tabellen 10.29/10.30;</li><li>externe koude;</li><li>regeneratietoeslag 10.84/10.85;</li><li>hulpenergie 10.83/10.87;</li><li>onttrekking door een boosterwarmtepomp.</li></ul> |
| `domestic_hot_water` | Volgorde 13.9 (douche-WTW na `η_W;em`). Verder: <ul><li>tabellen 13.1–13.9 en 13.18/13.23–13.29;</li><li>circulatie en pomp 13.25–13.50;</li><li>voorraadvat 13.58/13.59;</li><li>afleversets;</li><li>gasboiler 13.165–13.175;</li><li>`f_prac` 13.152;</li><li>hulpenergie 13.181;</li><li>5.36/5.37.</li></ul> |
| `pv` | Tabellen 16.1–16.3; onbekende bevestiging telt als 0,76; `c_sh;PV` volgt uit `F_sh;obst` per maand; collectieve verdeling. |
| `tojuli` | <ul><li>AOR/AVR telt niet mee;</li><li>koudebruggen per oriëntatie;</li><li>`a_C;red`;</li><li>`Q_C;HP;juli`;</li><li>actieve koeling alleen met capaciteitsbewijs.</li></ul> |
| `lighting` (nieuw) | Hoofdstuk 14 voor utiliteit, inclusief daglicht 14.24–14.44 en afronding volgens bijlage X. |

**Interpretatievragen** (implementatie volgt de gedrukte tekst, tenzij anders vermeld):

1. **10.15.** `Q_C;em;ls = Q·MAX(Δϑ/(ϑ_int,inc − ϑ_e,comb); 0,15)` staat zo gedrukt (p. 376). Dat geeft een afgifteverlies van minimaal 15 % zodra er verlies is. Bij verwarming is 0,15 een bovengrens. Moet dit `MIN` zijn?
2. **Tabel 10.16.** De kolom "Juli/september" is gelezen als juli tot en met september.
3. **Externe koude** staat niet in tabel 10.15. De code rangschikt haar bij de centrale opwekkers.
4. **Hulpenergie methode 3.** §10.5.6 zegt dat de forfaitaire waarden de condensorhulpenergie en de regeling al bevatten. Toch noemt 10.5.7 alleen `W_hr = 0` voor methode 3. De code rekent:
   - de regeling 10.87 (0,010 kW, alle uren);
   - condensorwaterdistributie 10.83 voor watergekoelde machines.
   
   Bij vrije koeling telt alleen pompenergie.
5. **Afwijkende EER/ζ.** Voor een waarde uit een kwaliteitsverklaring gebruikt de code `f_prpr = 0,9` (10.54). Alleen voor tabelwaarden is 1,0 voorgeschreven.
6. **`Q_hr;out` (10.80) in methode 3** gebruikt de tabel-EER zonder `f_C;PL`/`f_EER;corr`, die alleen in methoden 1/2 bestaan.
7. **Koelgrens.**
   - Met minder dan twee bruikbare maanden of een niet-stijgende lijn kiest de code 25 °C (de minste uren).
   - De luchttoevoerterm van 10.20 ontbreekt (H11).
8. **14.41.** `D_SNA = 0,54·τ·A_Ca/A_D·η_R` geeft een fractie, terwijl tabel 14.8 in % is. De code vermenigvuldigt met 100; anders kan een daklicht nooit daglicht leveren.
9. **13.181** telt W/kW·kWh zoals gedrukt. De omgevingstemperatuur van verwarmde ruimten voor circulatie en vat is het naar oppervlakte gewogen stooksetpoint, niet `ϑ_int;calc`.
10. **Tabel 13.27, klasse 4.** Boven 3 890 kWh houdt de code 1,0 aan. Lagere klassen boven hun bereik worden geweigerd (p. 644).
11. **TO-juli.** Transmissie via AOR/AVR blijft ook buiten de juli-balans per oriëntatie, niet alleen buiten de noemer. Dat is conservatief.
12. **Tabel 16.3.** De laatste rij luidt "≤ 0,80 → 0,75". Lagere `F_sh;obst` wordt dus niet geweigerd.

**Nog open voor deze hoofdstukken:**

- methoden 1/2 voor koudeopwekkers;
- AHU-koeling en ontvochtiging;
- zonneboilers;
- meerdere tapwateropwekkers en boosterwarmtepompen;
- bijlage P-verklaringen voor `dh`/`dc`;
- terugwinbare verliezen in hoofdstuk 7;
- automatische koppeling van de verlichtingswinst (7.28) aan de interne winst van hoofdstuk 7.

## Stand 2 oktober 2026: bijlagen T en U, terugwinbare tapwaterverliezen

Uit NTA 8800:2025+C1:2026 (p. 525–655 en 1096–1113) zijn toegevoegd:

- **Bijlage T** (module `hot_water_tests`): testrapporten voor gastoestellen.
  - Warmwatertoestel: T.4/T.5.
  - Combitoestel met gemeten zomer- en winterrendement: T.12–T.14.
  - Combitoestel met forfaitaire omrekening: T.15–T.17, `K_f` = 0,5, alleen CW (≥ 0,40 op bovenwaarde).
  - Omrekening naar bovenwaarde met tabel T.7.
  - Het resultaat vervangt de tabelwaarde van tabel 13.25, afgerond op 0,025, met de gemeten klasse voor `c_W;gen`.
- **Bijlage U**: douche-WTW-rendement uit drie runs (U.2–U.6), afgerond op 0,025.
- **13.13**: terugwinbare tapwaterverliezen per maand (13.47, 13.49, 13.63, 13.179). Ze voeden Φ_int;W (7.29) van utiliteitszones, verdeeld naar oppervlakte (13.14).

Interpretatievragen:

1. Formule 13.63 kent geen zonevoorwaarde. Toch tellen alleen vaten in een verwarmde zone mee; het verlies van een vat in een onverwarmde ruimte komt niet in de rekenzone terecht.
2. Bijlage T definieert het rendement inclusief de primaire omrekening van hulpelektriciteit (`η_el;ow` = 1/`f_P;del;el` = 1/1,45). Hoofdstuk 13 gebruikt het als `η_W;gen` van het gastoestel, zoals bij een Gaskeur-verklaring. T.3 (elektrische toestellen), T.7/T.8 (bivalente warmtepompen) en T.9/T.10 (micro-WKK) zijn daarom niet als route opgenomen.
3. De uitzondering van de 500 m²-regel ("meerdere individuele toestellen voor een deel kleiner dan 500 m²") wordt alleen toegepast op het elektrische doorstroomtoestel (13.179). Een individuele keukenboiler in een groot gebouw verliest daardoor ten onrechte zijn terugwinbare vatverlies, totdat de invoer aangeeft dat het om afzonderlijke toestellen gaat.
4. Bij een collectief systeem wordt het terugwinbare verlies van het beoordeelde deel (al geschaald met `A_g;si`/`A_g;gebouw;W`) over de eigen zones verdeeld naar `A_g;zi`/Σ`A_g` (13.14).
5. Bijlage U: de testklasse wordt niet getoetst aan de toepassingsklasse van de opwekker (§13.5.3). Dat blijft een controle voor de adviseur.

## Stand 1 oktober 2026: rekenruggengraat

Er is nu een doorgaande, **onverifieerde** Rust-keten van `.oes`-project tot BENG 1/2/3, TO-juli, Bbl-toets en indicatieve labelklasse. Onderdelen en bronnen:

| Schakel | Module | Bron |
|---|---|---|
| Klimaat (volledige tabel 17.1/17.2, interpolatie) | `climate` | transcriptie Heatloss |
| Transmissie H8: direct, onverwarmd, P/A-grond | `monthly_demand` (components), `ground` | transcriptie Heatloss (C1) |
| Maandbehoefte H7 | `monthly_demand` | transcriptie Heatloss (C2–C5) |
| Meerdere rekenzones, gedeelde opwekker | `space_heating_chain`, `project_performance` | 9.2, 5.6/5.8 |
| Afgifte 9.3 | `heating_emission` | doeleditie p. 296–297 (gelijk aan consultatie) |
| Distributie 9.26–9.51, buffervat 9.2.3, terugwinbaar 9.2.5 | `heating_distribution`, `space_heating_chain` | doeleditie p. 290–321 |
| Opwekkers: gasketel, forfaitaire warmtepomp, hybride, externe warmtelevering, elektrisch, biomassa; hulpenergie 9.85/9.91 | `space_heating_chain` | doeleditie p. 323–365 |
| Koudeopwekking §10.5 | `space_cooling` | transcriptie Heatloss (F3b) |
| Belemmering en zonwering (§17.3, 7.42) | `solar_shading` | transcriptie Heatloss (F3d) |
| Tapwaterbehoefte H13 (rendementen opgegeven) | `domestic_hot_water` | referenties Heatloss |
| PV H16 | `pv` | transcriptie Heatloss (F3d-4) |
| Primaire/hernieuwbare energie, opslagcorrectie, CO2, indicatoren H5 | `building_performance` | doeleditie p. 72–124 |
| TO-juli §5.7 per oriëntatie | `tojuli` | transcriptie Heatloss (F3c) |

| Afgifte 9.3 | `heating_emission` | consultatie H9 |
| Opwekkers: gasketel, forfaitaire warmtepomp (met optioneel gemeten hulpenergie), hybride, externe warmtelevering, elektrisch, biomassa | `space_heating_chain` | consultatie H9 |
| Koeling H10 (methode 3, afgifte, distributie, prioriteit) | `space_cooling` | normtekst p. 366–426 (2 oktober 2026) |
| Belemmering en zonwering (§17.3, 7.42) | `solar_shading` | transcriptie Heatloss (F3d) |
| Tapwater H13 (één opwekker) | `domestic_hot_water` | normtekst p. 525–655 (2 oktober 2026) |
| Verlichting H14 (utiliteit) | `lighting` | normtekst p. 655–676 (2 oktober 2026) |
| PV H16 | `pv` | normtekst p. 678–682 (2 oktober 2026) |
| Primaire/hernieuwbare energie, indicatoren H5 | `building_performance` | consultatie H5 |
| TO-juli §5.7 per oriëntatie | `tojuli` | normtekst p. 113–120 (2 oktober 2026) |
| Labelklasse en A0 | `label_class`, `bbl_requirements` | Omgevingsregeling bijlagen IX/X/IXa/Xa (wettekst) |
| BENG-eisen | `bbl_requirements` | Bbl tabel 4.148A (wettekst) |
| Projectadapter, UI-paneel, formulier, rekenrapport | `project_performance`, `NtaPerformancePanel`, `NtaCalculationReport` | — |

Teststand bij commit 65de7ff: 218 kerntests (ook met Rust 1.77.2), 45 servicetests, 219 UI-tests; clippy en rustfmt schoon (`scripts/verify-nta.sh`). De projectroute is via de Vite-proxy tegen de draaiende API en in de echte browser gecontroleerd. Een debug-desktoppakket (`src-tauri/target/debug/bundle/deb/`) is na de reviewfixes opnieuw gebouwd.

**Onafhankelijke review (1 oktober 2026).** Een tweede, onafhankelijke controle van de keten vond vijf fouten, die alle vijf zijn hersteld:

- `f_BACS` ontbrak op stadsverwarming en biomassa (5.20);
- omgevingswarmte van een tapwaterwarmtepomp kon dubbel tellen;
- "actieve koeling" was mogelijk zonder koelsysteem (§5.7.1);
- een vloer boven buitenlucht kreeg helling 0° in plaats van 180°;
- lokale toestellen konden Δθ_hydr = 0 niet gebruiken (tabel 9.3, voetnoot a).

De volgende interpretatievragen moeten tegen de normtekst of wettekst worden beantwoord:

1. Beweegbare zonwering op de warmtebalans bij utiliteit: nu altijd 1,0. Volgens het concept is dat alleen voor woningen voorgeschreven.
2. TO-juli-noemer: telt de geleiding via een onverwarmde ruimte mee in `H_C;D`? Opgelost op 2 oktober 2026: nee, volgens §5.7.2 stap A (p. 116). Zie interpretatievraag 11 hierboven.
3. Dubbele PV-opgave: inmiddels opgelost; PV moet via één route lopen (`pv_route_mixed`).
4. A0-voorwaarde a: met of zonder de toeslag van Bbl 4.149 lid 4?
5. Bijlage Xa noemt voor onderwijs 64, waar het patroon 63 doet verwachten. De code volgt de gepubliceerde tekst.
6. Bewoners bij een woning die over meerdere zones is verdeeld: nu per zone uit zone-A_g/aantal woningen.
7. Een hernieuwbaar aandeel boven 100% bij negatieve `EPtot` wordt niet afgekapt.
8. `θ_e;avg;an` is het ongewogen maandgemiddelde.

**Open punten die certificering blokkeren:**

1. Review van alle transcripties tegen de gelicentieerde normtekst. De pdf staat op netwerkshare `Z:`, die op deze machine niet gemount is.
2. Hoofdstuk 11: ventilatie, infiltratie, C1-systeem voor BENG 1, zomerventilatie voor TO-juli (tabellen 11.5/11.6/11.8 ontbreken).
3. Hoofdstuk 10: koeling.
4. Hoofdstuk 13: rendementstabellen.
5. Distributie 9.26 is gebouwd; de koppeling met §7.9.2/7.9.6 (`f_H;red`, `θ_int;op;H`) en met de ventilatietermen van 9.28/9.29 loopt nog via opgaven.
6. Terugkoppeling van 9.2.5 naar 7.2.1, en de correcties `a_H;red`/7.78.

3. Hoofdstuk 10: methoden 1/2, AHU-koeling en ontvochtiging (methode 3 is nu verwerkt, zie boven).
4. Hoofdstuk 13: zonneboilers, meerdere opwekkers en boosterwarmtepompen (de tabellen zijn nu verwerkt, zie boven).
5. Distributie 9.26: stookgrens, tabel 7.11, §7.9.2 en 7.9.6.
6. 9.2.3/9.2.5 en correcties `a_H;red`/7.78.
7. Bijlage-P-verklaringen.
8. Weging van gemengde functies (Bbl lid 2).
9. Wijzigingsregeling 2026 (A0, bijlagen IXa/Xa).
10. EDR-testset met de uitkomsten uit bijlage 2.
11. BRL 9501-attest.

## Historische stand (tot 30 september 2026)

De tekst hieronder beschrijft de stand vóór de normtekst beschikbaar kwam en is achterhaald door de samenvatting bovenaan.

Voor de gekoppelde gaswarmtepompketen bestaat nu een getypeerd Rust-referentievergelijkingsharnas. Het eist 25 verwachte deelwaarden met brongegevens, berekeningsbasis en kWh-toleranties; ontbrekende of dubbele maandposten, ongeldige keteninvoer en numerieke overloop blokkeren de vergelijking. HTTP, MCP en Tauri delen dezelfde kern. Het projectscherm heeft nu een JSON-import- en vergelijkingspaneel dat verschillen en foutpaden laat zien en geen uitkomst in het project opslaat. De opnieuw gebouwde release-API gaf via de Vite-proxy `compared_pass` voor 25 synthetische waarden, `compared_fail` bij één gewijzigde verwachting en HTTP 422 met lege vergelijkingslijst bij een ontbrekende post. De release-MCP-server gaf dezelfde drie statussen, met `isError=true` voor de onvolledige case. Het meegeleverde voorbeeld is uitdrukkelijk synthetisch; een match houdt `referenceVerified=false`. De officiële, onafhankelijke resultaten zijn nog niet beschikbaar. Zie [referentieprotocol](nta8800-referentieprotocol.md).

De conceptgaswarmtepompdiagnoses voor vergelijking 9.62 en 9.91/9.92 zijn nu ook als één Rust-keten beschikbaar. Zij verlangt hetzelfde toestel, aandrijving, vermogen, forfaitaire COP-basis, bewijs voor generatorwarmte en twaalf identieke warmtewaarden. Bij een mismatch verdwijnen alle maanduitkomsten. Voor een synthetische individuele absorptiewarmtepomp met COP 1,6, 20 kW, 1.000 kWh warmte en 730 h per maand staan de niet-toegewezen 9.62-term van 625 kWh en de **aparte elektrische toestelhulpstroom** van 8,4 kWh naast elkaar. HTTP, MCP, Tauri en het maandpaneel zijn aangesloten. De opnieuw gebouwde release-API gaf via de Vite-proxy beide waarden en HTTP 422 zonder deelwaarden bij afwijkende maandwarmte; de release-MCP-server gaf hetzelfde geldige geval en `isError=true` bij de afwijking. De actuele definitieve norm en een onafhankelijke EDR-vergelijking ontbreken. Ook deze keten boekt geen gas en berekent geen BENG of label. Zie [gekoppeld contract](nta8800-gaswarmtepomp-gekoppelde-conceptketen.md).

Voor gasmotor- en absorptiewarmtepompen geeft een nieuwe Rust-route op basis van conceptvergelijking 9.62 twaalf maandtermen uit een opgeslagen tabel-COP en **aangeleverde** generatorwarmte. De UI kan een individueel of passend collectief bronstelsel kiezen en toont de afgeleide bronwarmte en invoerterm. Bij een individuele buitenluchtbron, COP 1,6 en 1.000 kWh generatorwarmte is de conceptterm 625 kWh; bij collectief grondwater, COP 2,1, volgt 523,8095 kWh bronwarmte en een correctie van 11,5238 kWh. HTTP, MCP en Tauri gebruiken dezelfde Rust-functie. De opnieuw gebouwde release-API gaf via de Vite-proxy 625 kWh bij de individuele bron en HTTP 422 zonder maandtermen bij een onjuiste collectieve bron; de release-MCP-server gaf 464,666667 kWh bij collectief grondwater. `carrierAllocationAvailable=false` en `gasInputEnergyAvailable=false` blijven expliciet: deze termen zijn geen geboekt gasverbruik en geen energielabel. Zie [maandtermencontract](nta8800-gaswarmtepomp-maandtermen-concept.md).

De opnieuw gebouwde debug-`.deb` is met `npm run tauri:build -- --debug --bundles deb` succesvol afgerond en bevat de nieuwe Rust-command. De release-API en MCP-binary zijn opnieuw gebouwd. Met opgeslagen gas-COP- en twaalfmaands-hulpstroominvoer geeft de gebouwde API via Vite-proxy `structurally_valid`, de afzonderlijke waarschuwing `gas_heat_pump_aux_draft_unverified` en `calculationAvailable=false`; een afwijkend thermisch vermogen geeft `invalid`. De API gaf via HTTP en de Vite-proxy 8,4 kWh; de MCP-tool gaf hetzelfde en meldde `bengCalculationAvailable=false`. Er is geen visuele desktopacceptatietest uitgevoerd.

De losse gaswarmtepomp-toestelhulpstroomdiagnose volgt conceptformules 9.91/9.92 met opgegeven maanduren, warmtelevering en bronverwijzingen. Bij forfaitaire COP eist zij 10 W stand-by, 1 W/kW branderterm, modulatie 1 en 0 W/kW oplossingspomp. De handcontrole van 20 kW, 1.000 kWh warmte en 730 h geeft 8,4 kWh elektriciteit. De Rust-, HTTP-, MCP- en Tauri-routes zijn nu via een formulier in de warmtepompinventaris bereikbaar; twaalf maanden en hun bronverwijzingen kunnen in het project worden bewaard of verwijderd. Rust controleert de koppeling met een passende opgeslagen gastabelselectie. Het invoerdossier toont deze invoer; de deelroute is niet verbonden met een jaarprestatie; zie [conceptcontract](nta8800-gaswarmtepomp-hulpenergie-concept.md). De volledige core- en service-suites tellen nu respectievelijk 139 en 38 tests.

De concept-COP-rijen voor gasmotor- en absorptiewarmtepompen zijn nu als afzonderlijke, aan een PDF-kopie met SHA-256 gekoppelde tabeltranscriptie vastgelegd. Een Rust-test controleert alle 48 cellen voor beide aandrijvingen (96 selecties); de volledige core-suite telt nu 139 geslaagde tests. Dit is bronreproductie, geen onafhankelijke EDR-rekenuitkomst. De openbare ISSO 54-uitgave 2022 verwijst voor de resultaten naar een afzonderlijke Excelbijlage; een herleidbare openbare kopie daarvan en de actuele 2025-testset zijn bij de broncontrole van 29 september 2026 niet gevonden. Zie [referentieprotocol](nta8800-referentieprotocol.md) en [gaswarmtepomp-concept](nta8800-gaswarmtepomp-forfait-concept.md).

Open Energy Studio heeft een nieuwe Rust-core voor projectstructuur, schilgeometrie en warmtepomp-invoercontrole. De desktop-app roept deze direct aan; de lokale HTTP-API en de MCP-server gebruiken dezelfde crate. Het project- en resultatenscherm tonen de controle, inclusief gevonden fouten, aandachtspunten, het aantal geclassificeerde warmtepompen, kernelversie en een SHA-256-vingerafdruk van de JSON-invoer. De installatieformulieren en een nieuwe losse warmtepompinventaris kunnen bron, afgifte, aandrijving, hybride/booster/reversibele kenmerken, bediende zones en prestatiebewijs opslaan. Deze vingerafdruk maakt invoersnapshots herkenbaar; hij bewijst geen correcte berekening of attestering.

`calculationAvailable=false` en `attestStatus=unattested` blijven van kracht. De Rust-core berekent **nog geen** BENG, TO-juli of energielabel. De HTTP-berekenroute retourneert 501 voor structureel geldige invoer. De bestaande TypeScript-uitkomsten worden in resultaten, preview, statusbalk, HTML-rapport en IFC-export als indicatief aangeduid; daar staan geen geslaagd/gefaald-badges meer. Een structureel geldige invoer is geen bewijs van een normconforme berekening.

De indicatieve TypeScript-rekenroute weigert nu een leeg project of een zone met nul, negatief of niet-eindig vloeroppervlak. De live preview toont dan een invoerfout. Ook de los aanroepbare TO-juli-functie rekent niet meer met een kunstmatige 1 m².

De Rust-invoeraudit en beide oude indicatieve TypeScript-rekenroutes weigeren nu ook een ontbrekende, niet-positieve of niet-eindige COP en een dekkingsfractie buiten 0–1 voor bestaande verwarmingswarmtepompen. Dit is een fysieke invoergrens, geen normatieve prestatiebeoordeling.

De Rust-audit wijst ook rekenkundige overloop af wanneer afzonderlijk eindige vloer-, vlak- of raamoppervlakten samen een niet-eindige som vormen. De API geeft voor zo'n project op `/calculate` HTTP 422 en geen prestatieuitkomst; een ongeldige oppervlaktesom krijgt geen schilsamenvatting. Dit is een invoerveiligheidscontrole, geen NTA-oppervlaktemethode.

De Rust-audit toont nu per zone en totaal de ingevoerde bruto schiloppervlakte, raamoppervlakte en het rekenkundige restant voor dichte delen. De samenvatting ontbreekt bij onvolledige of ongeldige vlak-/raamgeometrie, waaronder een ontbrekende ramenlijst. Dit zijn uitsluitend invoersommen; de NTA-meetregels, begrenzing van de thermische schil en transmissieberekening zijn niet geïmplementeerd.

De audit controleert ook aanwezige Rc-/U-waarden en laagdiktes/geleidingscoëfficiënten van constructies, U-/g-waarden van ramen, koudebruglengtes en Ψ-invoer, en qv10. Onfysische waarden zijn fouten; ontbrekende thermische waarden van oude constructies of ramen geven een waarschuwing. De geometriesom blijft beschikbaar als alleen thermische eigenschappen ongeldig zijn, zodat een oppervlaktesom niet ten onrechte als gevalideerde warmteverliesberekening wordt gezien. Er is geen normatieve relatie tussen Rc en U, koudebrugtoeslag of infiltratieroute berekend.

Wanneer een bestaande verwarmings- of tapwaterwarmtepomp prestatiepunten heeft, melden Rust en de UI expliciet dat hiervoor geen geverifieerde rekenroute bestaat. Beide oude indicatieve rekenroutes en de live preview stoppen dan, zodat geen uitkomst de punten stilzwijgend negeert. Iedere wijziging van projectinvoer wist nu ook een eerder opgeslagen indicatief resultaat; alleen UI-handelingen behouden het resultaat.

Een elektrisch aangedreven compressor met een prestatiepunt op een andere ingaande energiedrager dan elektriciteit krijgt nu `performance_point_carrier_mismatch`. De gedeelde werkpunteditor weigert zo'n punt en blokkeert ook opslaan na een latere wijziging van gasmotor naar elektrische compressor. Deze fysieke consistentiecontrole levert geen normatief warmtepomprendement op.

Voor structureel geldige projecten geeft de Rust-audit nu per werkpunt de dimensieloze verhouding nuttig/ingaand vermogen, inclusief toestel, dienst en energiedrager. Desktop, API en MCP ontvangen dezelfde `performancePointDiagnostics`; de UI toont haar met een expliciete waarschuwing dat het geen jaarprestatie of NTA-resultaat is. Een werkpunt waarvan de verhouding numeriek overloopt maakt de invoer ongeldig en levert geen diagnostisch getal op. Er is geen onafhankelijke toets van de verklaarde vermogens of de meetgrens met hulpcomponenten.

Voor latere prestatiecurven mogen twee werkpunten dezelfde temperaturen hebben als hun nuttige vermogens verschillen. Een exact dubbelzinnige toestand (dezelfde dienst, temperaturen, nuttig vermogen en energiedrager) wordt door Rust en de editor afgewezen; bij verschillende ingaande vermogens zou anders onduidelijk zijn welk punt geldt. Een onbekende meetgrens van hulpvermogen krijgt daarnaast een afzonderlijke auditwaarschuwing, zodat mogelijke dubbeltelling expliciet blijft. Dit levert nog geen deellastinterpolatie of jaar-COP op.

De UNIEC3-export schrijft nu uitsluitend een als invoerconcept gemarkeerd projectarchief, zonder indicatieve BENG-/TO-juli-prestatievelden. Een eigen export/import-round-trip controleert de ZIP-structuur. Externe import door UNIEC3 is niet geverifieerd; de oude vaste versiemetadata blijft een interoperabiliteitsvraag.

Open Heatloss Studio is opnieuw op crate- en testniveau onderzocht. Vijf relevante Rust-crates (`demand`, `dhw`, `ep`, `heating`, `tables`) slagen lokaal in totaal voor 382 unit-tests, maar hun documentatie benoemt V1-vereenvoudigingen en de verwarmings-/tapwaterwarmtepompen gebruiken een opgegeven seizoens-COP. De publieke historische ISSO 54-uitgave uit 2022 bevat voor EP-W001 onafhankelijk gepubliceerde geometrie-invoer, maar de officiële energie-uitkomsten staan in een ontbrekende Excelbijlage. OES heeft daarom alleen een [historische geometrie-regressie](nta8800-heatloss-hergebruik.md) toegevoegd: Ag 96 m², Ao 247,2 m², raamoppervlak 24 m² en V 259,2 m³. Zij controleert de som van ingevoerde vlakken, geen NTA:2026-meetregel of energieprestatie.

De Rust-kernel controleert nu losse `ntaHeatPumps`-toestellen met acht brontypen, dubbele IDs, bediende zoneverwijzingen en bewijsreferenties. Optionele prestatiepunten bewaren dienst, temperaturen (°C), vermogens (kW), energiedrager en testreferentie; de kern controleert waarden en toepasselijkheid, maar rekent er nog niet mee. De gedeelde editor kan werkpunten toevoegen, bewerken en verwijderen. Zoneverwijzingen worden ook bij warmtepompmetadata in bestaande verwarmings- en tapwatersystemen gecontroleerd; dubbele zones en metadata op niet-warmtepompsystemen worden afgekeurd. Elk los toestel krijgt een waarschuwing dat de rekenroute ontbreekt. Wanneer zulke toestellen aanwezig zijn, stopt de oude indicatieve berekening zodat geen uitkomst verschijnt die deze invoer heeft genegeerd.

De NTA-specifieke warmtepompobjecten en prestatiepunten weigeren nu onbekende velden; een typefout zoals `performancePoint` kan daardoor niet stilzwijgend als lege puntenlijst worden gelezen. De HTTP-API meldt een vormfout, de desktop-audit geeft bij ingebedde metadata de exacte parserdetails weer, en de MCP-tools gebruiken dezelfde machineleesbare foutcodes als HTTP.

Het rapportscherm kan nu ook zonder berekend resultaat een zelfstandig HTML-invoerdossier exporteren. Dat vermeldt per warmtepomp de invoercontext, bron/afgifte, energiediensten, registergegevens, prestatiepunten, hulpcomponenten en systeemkoppelingen. Het dossier noemt uitdrukkelijk dat bewijs en registergegevens niet extern geverifieerd zijn en bevat geen BENG- of labeluitkomst. Door gebruikers ingevoerde tekst wordt in dit dossier en het bestaande indicatieve HTML-rapport als HTML ge-escaped. De knop is via componenttest bediend; visuele desktopinteractie blijft hieronder als open controle staan.

Een gecontroleerde warmtepompverklaring kan nu naast de vrije-tekstreferentie een optioneel `registryRecord` bewaren met registratienummer, exacte product(combinatie), fabrikant en HTTPS-bronlink. De editor voorkomt een gedeeltelijk ingevuld record. Een oude verklaring zonder dit record blijft leesbaar en geeft in de Rust-audit een waarschuwing. De velden zijn door de gebruiker opgegeven en worden niet bij BCRG geauthenticeerd; toepasselijkheid en prestatie blijven ongeverifieerd. De [BCRG-bronnenanalyse](nta8800-warmtepomp-verklaringen.md) vermeldt dat aanvraagversie 2026.1 deels nog naar NTA 8800:2024 verwijst.

BCRG bevestigt een officiële [API-koppeling voor rekensoftware](https://bcrg.nl/nl/opnemen-register/api/), maar een sleutel wordt per aanvraag verstrekt en [kosten kunnen gelden](https://bcrg.nl/nl/opstellen-energieprestatie-advies/api/). Voor OES zijn sleutel, contract en schema nog niet aanwezig. Daarom blijft het registerrecord expliciet door de gebruiker ingevoerd en ongeverifieerd.

Warmtepompen kunnen nu ook afzonderlijke hulpcomponenten vastleggen: bronpomp, bronventilator, binnenventilator, distributiepomp, regeling/stand-by, ontdooien, bijverwarmer of overig. Elk component heeft een energiedienst, nominaal vermogen in W, energiedrager, bronreferentie en een expliciete meetgrens (`unknown`, `included_in_declared_performance` of `additional`). De Rust-kern valideert identiteit, positief vermogen, referentie en toepasselijkheid van de dienst. De UI kan componenten bewerken en bewaart ze ook bij heropenen van bestaande installaties. **Nominaal vermogen wordt niet omgezet in jaarenergie**: bedrijfsuren, seizoensweging en het voorkomen van dubbeltelling met verklaarde prestatie zijn nog niet normatief vastgesteld. Ingebedde systemen met zulke gegevens worden door de oude indicatieve rekenroute geweigerd.

Een warmtepomp kan nu een expliciete koppeling naar een bij-/reserveopwekker, voorgeschakelde warmtepomp of gedeelde bron opslaan. De Rust-kern controleert doeltype, doel-ID, bronreferentie, bestaande doelrecords en kringvormige voorgeschakelde warmtepompen. De UI biedt hiervoor een doelkeuze uit de projectinstallaties. Dit is uitsluitend systeemtopologie: er zijn geen omschakelmomenten, prioriteiten, energieaandelen of bronbalansen berekend. Ingebedde koppelingen stoppen de oude indicatieve calculator zodat hij ze niet negeert.

Een afvoerluchtwarmtepomp kan nu ook een bronkoppeling naar een bestaand ventilatiesysteem vastleggen. De Rust-kern waarschuwt als die ontbreekt en weigert een onbekende ventilatie-ID of verkeerd bron-/doeltype. De editor biedt bij afvoerlucht de ventilatiesystemen van het project aan. Ventilatiedebiet, warmtebeschikbaarheid en eventuele interactie met warmteterugwinning worden nog niet berekend.

De Rust-audit controleert nu ook dat verwarmings-, ventilatie-, koel- en tapwatersystemen objectrecords met een niet-lege unieke ID en een niet-leeg systeemtype zijn. Een verwijzing naar een ventilatiesysteem zonder type maakt de invoer ongeldig, ook als de ID bestaat. Dit is een structurele vormcontrole en geen verificatie van ventilatieprestatie of NTA-installatieroute.

Een Rust-referentiemanifestcontrole is toegevoegd voor bron, normeditie, onafhankelijke verwachte deelwaarden, eenheden en toleranties. HTTP en MCP bieden dezelfde administratieve audit. `manifestComplete` is uitsluitend een veldcontrole; `referenceVerified=false` en `calculationAvailable=false` blijven expliciet. Er zijn nog geen actuele onafhankelijke resultaatbestanden aanwezig.

De referentie-audit geeft nu naast de projectvingerafdruk ook een vingerafdruk van het **hele manifest**, inclusief bronmetadata, verwachte deelwaarden en toleranties. Het diagnostische W/K-vergelijkingsharnas geeft eveneens een afzonderlijke `caseFingerprint`. Een gewijzigde verwachting kan daardoor niet onder dezelfde case-identiteit verdwijnen; een hash bewijst echter niet dat de bron onafhankelijk of juist is.

Een aparte getypeerde case kan nu de vier posten van de directe-transmissiediagnose met aangeleverde W/K-verwachtingen en toleranties vergelijken via Rust, HTTP en MCP. Het harnas meldt verschillen per post en weigert ontbrekende waarden, eenheden en ongeldige invoer. `compared_pass` is een rekenkundige overeenkomst met door de aanroeper opgegeven waarden; de onafhankelijkheid en normatieve toepasbaarheid daarvan zijn niet vastgesteld. `referenceVerified=false` en `bengCalculationAvailable=false` blijven ook bij een geslaagde vergelijking staan.

Als eerste zuiver Rust-rekenblok is een afzonderlijke **diagnostische directe transmissiesom** toegevoegd: buitenvlakken `A·U`, lineaire koudebruggen `L·Ψ` en puntbijdragen `χ`, elk met expliciete eenheid, ID en herkomstveld. De API en MCP gebruiken dezelfde functie en geven bij ongeldige waarden geen getal. De response bevat deelposten, kernel-/doelnormversie en invoervingerafdruk, maar ook `referenceVerified=false` en `bengCalculationAvailable=false`. De functie kan vanuit expliciet geclassificeerde projectinvoer worden gevoed, maar is niet aan de BENG-route gekoppeld en is niet tegen een onafhankelijke NTA-deeluitkomst getoetst. Zie [het contract](nta8800-directe-transmissie.md).

De kern heeft nu ook een **diagnostische maandelijkse directe warmteflow**: twaalf expliciet aangeleverde maandrecords vermenigvuldigen de buiten-W/K met het getekende temperatuurverschil en de uren, met kWh per maand en als jaarsom. De API en MCP gebruiken diezelfde functie. Ongeldige of ontbrekende maanden en niet-eindige uitkomsten geven geen energie. De temperatuur-/urenset komt van de aanroeper; dit is geen NTA-klimaat, warmtebehoefte of BENG-resultaat. Zie [het contract](nta8800-maandtransmissie.md).

De oude indicatieve TypeScript-calculators en de preview worden geblokkeerd zodra het project een onverwarmde ruimte of zo’n thermische grens bevat; die route wordt daarin niet verwerkt. Een eerder projectresultaat verdwijnt bij wijziging van deze invoer.

Een tweede transmissieroute kan nu **diagnostisch** expliciet benoemde onverwarmde ruimtes optellen. Per ruimte vermenigvuldigt Rust de aangeleverde grensvlaksom `Σ(A·U)+Σ(L·Ψ)+Σχ` met een eveneens aangeleverde factor `b` binnen 0–1. API en MCP gebruiken dezelfde kernfunctie; ongeldige factoren, ontbrekende herkomst, foutieve componenten en overloop geven geen gedeeltelijk getal. De factor wordt niet uit NTA-gegevens afgeleid, de `.oes`-projectadapter en UI bewaren nu benoemde ruimtes, factoren met bron en expliciete grensvlakverwijzingen; een actuele onafhankelijke deeluitkomst ontbreekt nog. `referenceVerified=false` en `bengCalculationAvailable=false` blijven van kracht. Zie [het contract](nta8800-onverwarmde-transmissie.md).

Vlakken, lineaire koudebruggen en nieuwe puntkoudebruggen kunnen via de editors een expliciete thermische grens krijgen. De Rust-projectaudit meldt de classificatievoortgang en maakt **alleen bij volledig geclassificeerde, bruikbare invoer en expliciet bevestigde puntbruginventaris per zone** een beperkte directe buiten-diagnose zichtbaar. De adapter splitst bruto vlakken in dicht deel en ramen, telt buitenpunten als `Σχ` en houdt grond en aangrenzende ruimtes buiten deze som. Bestaande projecten krijgen geen veronderstelde grens, lege puntlijst of transmissiegetal. Bij toevoegen of verwijderen van een puntbrug moet de gebruiker de inventaris opnieuw bevestigen. Interne veldverwijzingen en door de gebruiker ingevulde detailreferenties zijn geen extern gecontroleerde bewijsstukken.

De warmtepompinvoer kan nu ook getypeerde dagelijkse tapwater-testprofielen uit een gecontroleerde kwaliteitsverklaring bewaren. Rust en de UI controleren onder meer de tapwaterafgifte, verklaring, unieke profielen, eenheden en broneditie. De projectaudit waarschuwt expliciet bij een afwijkende verklaringeditie en toont alleen de ruwe testenergieverhouding; die is geen praktijkrendement of jaarprestatie. De [BCRG-verklaring 20260044GK](https://mijn.bcrg.nl/media/documents/2026/GK/20260044GK.pdf) voegt een afvoerluchttoestel met debiet en droge-/natteboltemperaturen per profiel toe; ook deze in 2026 uitgegeven verklaring noemt een oudere normeditie (2024) en dient alleen als invoerregressie. De historische [BCRG-verklaring 20240123GK](https://mijn.bcrg.nl/media/documents/2024/GK/20240123GK.pdf) levert twee extern gepubliceerde invoerpunten (M/XL) voor een regressiefixture, maar noemt NTA 8800:2020 en geldt hier niet als actuele EDR- of normreferentie. Het invoerdossier toont de waarden met bron; de oude indicatieve calculator weigert ze. Zie [tapwater-verklaringsinvoer](nta8800-tapwater-verklaringsinvoer.md).

De Rust-kernel heeft nu ook een strikt begrensde, afzonderlijke tabelinterpolatie voor verklaarde ruimteverwarmingswaarden. Een expliciet gedeclareerde eerste klasse `θsup ≤ 30 °C` accepteert ook een lagere aanvoertemperatuur door precies de 30 °C-rij te kiezen; zonder deze klasse en boven de laatste rij blijft de vraag buiten bereik. Dit is aan de gepubliceerde BCRG-tabelvorm getoetst, geen algemene NTA-toepasselijkheidsregel. Vierpuntsfragmenten uit [BCRG 20250005GK](https://mijn.bcrg.nl/media/documents/2025/GK/20250005GK.pdf) en [BCRG 20260143GG](https://mijn.bcrg.nl/media/documents/2026/GG/20260143GG.pdf) geven controleerbare tussenwaarden voor rendement, voorkeursfractie en hulpenergie. De verklaringen noemen NTA 8800:2024 en 2025; selectie van de juiste tabel, systeemgrens, actuele normtoepassing en jaarbalans zijn niet geïmplementeerd. HTTP, MCP en een Tauri-command gebruiken dezelfde module; zie [verklaringstabel](nta8800-ruimteverwarming-verklaringstabel.md).

Een zelfstandige Rust-diagnose voor gedeclareerde tapwatertestprofielen is nu via HTTP, MCP en desktop beschikbaar. Zij controleert een volledig getypeerd warmtepomprecord en toont alleen de opgegeven dagelijkse energieën, de ruwe nuttig/ingaand-verhouding en brongegevens; ongeldige invoer levert geen profielen. De gebouwde API gaf voor BCRG 20260143GG profiel M 3,574209 en L 3,863321, en HTTP 422 zonder profielen na een nul-invoerenergie. De MCP-tool gaf dezelfde geldige profielen met `bengCalculationAvailable=false`. Deze testwaarden zijn geen praktijkrendement of jaarresultaat; zie [tapwater-testinvoer](nta8800-tapwater-verklaringsinvoer.md).

De publieke hoofdstuk-5-consultatie bevat formules 5.57–5.58 en 5.60 voor jaarlijks finaal energiegebruik. Een nieuwe afzonderlijke Rust-conceptmodule telt expliciet aangeleverde maandelijkse `E_EPus` per drager en zonneboilerpraktijkbijdragen op, met volledige twaalfmaands- en inventariscontrole. Een synthetisch geval geeft 3.600 kWh finaal en 3.720 kWh EED; met afzonderlijk opgegeven 100 m² berekent de module voorlopig 36,00 en 37,20 kWh/m²·jaar. Rust `rust_decimal` bewaart de optelling en de afronding naar boven op 0,01; 0,1 + 0,1 + 0,1 blijft 0,30, 0,001 extra wordt 0,31. De dependency is op 1.40.0 vastgepind en slaagt op Rust 1.77.2. Dit is geen definitieve normtekst, officiële EDR-test, BENG of label. HTTP, MCP en desktop gebruiken dezelfde kern; zie [conceptdiagnose](nta8800-finale-energie-conceptdiagnose.md).

Dezelfde openbare hoofdstuk-5-consultatie geeft ook conceptvergelijkingen 5.20–5.21 voor maandelijkse dienst- en hulpenergiecompositie per drager. Een tweede Rust-module neemt alle posten expliciet aan, past de opgegeven BACS-factor alleen op verwarming, koeling en hun hulpenergie toe en geeft de maandstroom door aan de finale-energiemodule. Een synthetisch elektrisch geval geeft 177,6 kWh/maand, 2.131,2 kWh/jaar en bij opgegeven 100 m² 21,32 kWh/m²·jaar. De diensthoofdstukken en actuele EDR-uitkomsten ontbreken; zie [maandelijkse conceptdiagnose](nta8800-epus-conceptdiagnose.md). Met optioneel collectief-bronbewijs leidt dezelfde module de `dh`-bronwarmte van een collectieve gaswarmtepomp zelf af (ongewogen door `fBACS`), eist een `dh`-drager en weigert een tweede handmatige bronpost als dubbeltelling.

Uit §5.5.8 van hetzelfde openbare concept is een afzonderlijke voorlopige BACS-factorcontrole toegevoegd. Zij onderscheidt woningbouw en utiliteit, telt generatorvermogens **per verwarmings-/koelsysteem**, herkent strikt meer dan 290 kW of een onbepaalbaar systeemvermogen, en beoordeelt ontbrekende BACS, regelklasse D of energiemanagementklasse C/D. Systeemgebonden klassebewijs heeft voorrang op gedeelde BACS-invoer; een bewezen overtreding bij één systeem volstaat voor de factor 1,05. Een onvolledige inventaris of zonder overtreding ontbrekend vereist klassebewijs geeft geen factor. Een weggelaten generatorvermogen is een HTTP-vormfout; alleen expliciet `null` betekent onbepaalbaar. De maandelijkse conceptdiagnose kan deze bewijsinvoer meekrijgen en weigert een afwijkende opgegeven factor. De wettelijke GACS-plicht, definitieve normeditie, BACS-functiebewijs en EDR-validatie zijn hiermee niet vastgesteld; zie [BACS-conceptdiagnose](nta8800-bacs-conceptdiagnose.md).

De conceptformules 5.1–5.3 zijn toegevoegd als een afzonderlijke indicatorendiagnose uit **aangeleverde** jaarlijkse C1-behoefte, fossiele en hernieuwbare primaire energie en gebruiksoppervlakte. De Rust-kern rondt de behoefte en fossiele indicator naar boven op 0,01 af, hernieuwbaar aandeel naar beneden op 0,1 procentpunt en hernieuwbare energie per m² naar beneden op 0,01. Bij externe-leveringsverklaringen moeten verklaring en forfait als een compleet paar worden opgegeven. De module berekent zelf geen upstream jaartotalen, BENG-toets of label; zie [conceptindicatoren](nta8800-indicatoren-conceptdiagnose.md).

Het openbare [hoofdstuk-9-concept](https://www.internetconsultatie.nl/epg2026/document/14150) is nu ook op warmtepomp-hulpenergie onderzocht. §9.6.8.1.1 geeft maandvergelijking 9.85 voor individuele toestellen en sluit bronpomp/-ventilator uit die bepaling. De algemene definitie van coëfficiënt `B` gebruikt kW, maar de forfaitregel voor elektrische warmtepompen vermeldt kWh. Deze tegenstrijdigheid en de ontbrekende definitieve norm/EDR zijn vastgelegd in [hulpenergie-onderzoek](nta8800-warmtepomp-hulpenergie-concept.md). Een afzonderlijke Rust-conceptmodule rekent vergelijking 9.85 nu met expliciet aangeleverde gemeten coëfficiënten en twaalf maanden generator-ingangsenergie: het synthetische geval geeft 15 kWh/maand en 180 kWh/jaar. Een tweede diagnose leidt A/B/C nu voorlopig af uit opgegeven stand-by- en afgiftepompvermogens, schakeltijden en modulatie via 9.86–9.88. Zij geeft bij het handvoorbeeld A=87,6, B=0,19 en C=0,5, gevolgd door 26,3 kWh/maand. De bronpomp/-ventilator en het dubbelzinnige forfait vallen buiten deze routes; jaarprestatie, BENG en label blijven onbeschikbaar.

Het openbare hoofdstuk 9 bevat ook forfaitaire COP-tabellen 9.27 en 9.29. Een nieuwe afzonderlijke Rust-diagnose leest hun basisrijen voor elektrische warmtepompen per expliciet gekozen woning-/utiliteitsscope, warmtebron en aanvoertemperatuur; de lucht/lucht-waarde 2,8 is apart afgedekt. Een lege cel, ontbrekende broncorrectie of temperatuur boven 70 °C geeft geen COP. De hogere woningbouwrijen zijn nu optioneel en vereisen alle bronafhankelijke COP-testpunten strikt boven de grenzen van tabel 9.28 en een opgegeven product- en testrapportbron; boven 55 °C bestaat geen hogere tabelcel. Authenticiteit van het rapport, productmatch, bijlagen V/Q, methode-2-energiegebruik en jaarlijkse BENG blijven buiten scope. Zie [forfaitaire COP-conceptdiagnose](nta8800-warmtepomp-forfait-cop-concept.md).

De losse elektrische warmtepompinventaris heeft hiervoor nu een formulier voor tabelscope, bronklasse, ontwerpaanvoer en bewijsverwijzingen. Deze invoer kan in `.oes` worden bewaard en heropend; het invoerdossier toont de opgegeven klasse en bron, zonder een COP als projectuitkomst te presenteren. De Rust-projectaudit toetst de opgeslagen invoer aan toestel, bron en gebouwfunctie en meldt de onbevestigde conceptstatus. De UI vraagt Rust-validatie aan voordat zij de tabelinvoer bewaart en biedt de lucht/lucht-rij uitsluitend bij een buitenluchtbron aan. Grondwater en collectieve bronklassen krijgen nu een expliciete brontemperatuur met bewijs; vanaf 20 °C kiest de diagnose bij ontbrekende bronkwaliteitsverklaring nu de grondwater-<15 °C-tabelrij en meldt die keuze expliciet. Bij woningbouw blijft de broncorrectie verplicht. Voor een opgegeven bodem-/grondwaterbron met onbekend type of temperatuur kiest de diagnose nu expliciet de bodemrij; de projectaudit waarschuwt voor die terugval. De herkomst van deze classificatie is niet onafhankelijk geverifieerd. Voor nieuwe invoer zijn nu ook opgegeven thermisch vermogen, bewijsbron en individueel/collectief verplicht; de Rust-projectaudit markeert dit bewijs als ontbrekend bij oudere dossiers en wijst een aantoonbaar verkeerde 25 kW-tabelscope af. De tabelselectie en invoer zijn daarmee controleerbaar, maar leveren nog geen geverifieerde jaarprestatie op.

De losse warmtepompinventaris biedt nu een UI-formulier voor de gemeten hulpenergie-conceptdiagnose bij een passend individueel elektrisch toestel. Alle twaalf maandwaarden en de bronverwijzingen zijn verplicht. De meetinvoer kan in het `.oes`-project worden bewaard en heropend; de Rust-projectaudit valideert die en meldt de conceptstatus. Het invoerdossier toont de opgeslagen bron- en meetwaarden. De uitkomst van de diagnose blijft tijdelijk en wordt niet als officieel projectresultaat opgeslagen.

De hybride-invoer bewaart nu ook productgebonden afschakelgrenzen uit [BCRG 20240234GK](https://mijn.bcrg.nl/media/documents/2024/GK/20240234GK.pdf): COP 2,0 en 55 °C voor Xtend 5 SE. Rust valideert en waarschuwt dat schakeling en normeditie niet geverifieerd zijn; de UI-editor, het invoerdossier en de oude-calculatorblokkering zijn aangepast. Zie [hybride grenzen](nta8800-hybride-afschakelgrenzen.md).

De Rust-kern heeft nu bovendien een voorlopige generatorverdeling uit openbaar hoofdstuk 9: standaardprioriteit tabel 9.1, nieuwbouw-β uit geïnstalleerde vermogens, winter-/zomerfracties en lineaire interpolatie uit tabel 9.23 en restvraag volgens 9.2–9.3. Een gekoppelde route leidt de maandelijkse warmtepompwarmte af en berekent daarmee de voorlopige generator-elektriciteit; het hybride UI-formulier vereist expliciete nieuwbouwbevestiging en herleidbare vermogens, ketelcategorie en afgiftetemperatuur. De knooppuntwarmte is nog aangeleverd; ketelaardgas volgt nu uit concepttabel 9.25 en formule 9.61. Individuele ketelhulpenergie volgt nu ook voorlopig uit formule 9.85, met bouwjaarklasse als expliciete invoer. Bij opgeslagen individuele meetcomponenten wordt warmtepomphulpenergie optioneel uit de in deze keten afgeleide maandstroom berekend; oude los ingevoerde maandwaarden worden vervangen. Waakvlam en bronpomp/-ventilator ontbreken; productgebonden afschakelgrenzen of een hybride ontwerpaanvoer boven 55 °C geven geen gekoppeld resultaat. Zie [de conceptanalyse](nta8800-generatorverdeling-concept.md).

## Uitgevoerde controles

De Rust-core is daarnaast daadwerkelijk met de gedeclareerde minimumversie Rust 1.77.2 getest: alle 139 tests slagen. Veertien aanroepen van `Option::is_none_or` vereisten Rust 1.82 en zijn door een gelijkwaardige, met 1.77.2 compatibele vorm vervangen. Rustfmt-controle en strenge Clippy-controle (`-D warnings`) slagen voor core en service. CI en de tag-releaseworkflow eisen nu TypeScript-controle, UI-tests, webbuild, Rust-tests, rustfmt, Clippy en de coretest op Rust 1.77.2 vóór pakketbouw of releasecreatie. Een door GitHub uitgevoerde workflowrun is nog niet gecontroleerd. Releases blijven als prerelease gemarkeerd zolang de NTA-rekenkern niet onafhankelijk is geverifieerd en geattesteerd.

| Onderdeel | Resultaat | Grens |
| --- | --- | --- |
| Gekoppelde gaswarmtepompconceptketen | Core 139, service 38 en UI 212 tests geslaagd; release-API en MCP geven bij 1.000 kWh aangeleverde warmte 625 kWh niet-toegewezen 9.62-term en 8,4 kWh aparte elektrische toestelhulpstroom. Een afwijkende maandwarmte geeft HTTP 422/MCP-fout zonder deeluitkomsten. Webbuild en debug-`.deb` geslaagd. | Geen definitieve normeditie, gasdragerboeking, bronpomp, onafhankelijke actuele EDR-case, BENG of energielabel; geen visuele desktopacceptatie. |
| Collectieve gaswarmtepompbron (concept) | Core 143 (ook Rust 1.77.2), service 39 en UI 213 tests geslaagd; strenge Clippy en rustfmt geslaagd; webbuild en debug-`.deb` gebouwd. Release-HTTP via Vite geeft voor 12 × 1.000 kWh 6.285,71 kWh afzonderlijke `dh`, `fP=1,45/23`, `fPren=0,95`; MCP geeft dezelfde bronuitkomst en weigert ontbrekend verklaringbewijs zonder deelwaarden. UI-componenttoets controleert bewijsvelden en vervallen uitkomst. Zie [scope en bronnen](nta8800-gaswarmtepomp-collectieve-bron-concept.md). | Alleen publieke consultatieconcepten; geen definitieve normeditie, gasinput, bronpomp/-ventilator, actuele onafhankelijke EDR-case, BENG, label of visuele desktopacceptatie. De volledige UI-suite had wisselende 5s-timeouts in bestaande tests; alle 213 slagen met een eenmalige 10s-testgrens. |
| EPUS met afgeleide collectieve gasbron (concept) | Twee Rust-coretests en één HTTP-servicetest toegevoegd: afgeleide `dh`-post 1.000 × (1 − 1/2,1) kWh per maand, `gasCollectiveSourceDerived=true`, bronvingerafdruk; ontbrekende `dh`-drager en handmatige dubbele bronpost geven `invalid` zonder maand- of jaaruitkomst. TS-types bijgewerkt; rustfmt-controle geslaagd. **Tests nog niet uitgevoerd**: in de afrondingsomgeving was crates.io niet bereikbaar; lokaal `cargo test`, Clippy, `npm test` en `npm run build` draaien vóór commit. | Geen gasdragerboeking van de 9.62-term, geen bronpomp/-ventilator, geen UI-paneel voor EPUS, geen onafhankelijke referentie |
| Gaswarmtepomp-referentievergelijking | 25 maand- en jaarposten verplicht; release-HTTP/MCP: `compared_pass`, `compared_fail` en onvolledige case als fout zonder deelvergelijking; invoer- en casevingerafdrukken apart. Projectscherm: JSON-import en verschillenoverzicht; vier gerichte UI-tests geslaagd. | Synthetisch voorbeeld, geen officiële EDR-waarden of onafhankelijk bronbewijs; `referenceVerified=false` ook bij match |
| Rust coretests | 139 geslaagd; opgeslagen forfaitaire COP-tabelinvoer met bron-/toestel-/gebouwscope en waarschuwing bij terugval van de collectieve bronklasse of onbekend bodem-/grondwatertype/-temperatuur; drie bestaande `.oes`-fixtures en de historische EP-W001-geometrie-invoersom, acht warmtepompbronnen, prestatiepunten inclusief elektrische dragercontrole en numerieke verhoudingsoverloop, drie BCRG-tapwater-testinvoerfixtures (M/XL) met broneditie en optionele bronluchtcondities, ruwe verhouding en ongeldige-sink-/verklaringstoets, optioneel registerrecord voor verklaringen, hulpcomponenten, systeemkoppelingen inclusief afvoerlucht/ventilatie en cycli, systeemrecordvorm en IDs, referentiemanifest- en casevingerafdrukken, getypeerde W/K-vergelijking inclusief verschiloverloop, geometrie en thermische invoercontroles inclusief somoverloop, handberekende directe transmissiesom en maandflow, projectadapter met buiten/grond-/puntuitsplitsing, benoemde onverwarmde ruimtes en provenance-fingerprint | Geen onafhankelijke actuele NTA-energie-referentiecases |
| Heatloss-hergebruikonderzoek | Vijf relevante crates lokaal getest: 84 demand, 70 dhw, 67 ep, 62 heating en 99 tables unit-tests geslaagd; historische EDR-fixtures en V1-scope geïnspecteerd | Zusterrepo-tests zijn geen OES-integratie of actuele externe attesttoets |
| Gebouwde API/MCP: werkpunt en meetgrens | API `/calculate` met twee punten op identieke dienst, temperaturen, nuttig vermogen en drager maar verschillend ingangsvermogen geeft HTTP 422 met `performance_point_condition_duplicate`; onbekende hulpmeetgrens geeft via API en MCP een waarschuwing, `calculationAvailable=false` | Invoer- en meetgrenscontrole; geen prestatiecurve, jaarenergie of normtoets |
| Onverwarmde-ruimtesom in Rust/HTTP/MCP | Handvoorbeeld `4,2 × 0,5 = 2,1 W/K`, factor/ID/bron- en componentcontrole; gebouwde API HTTP 200 en 422 zonder deelgetal, MCP-toolcall en foutresponse bevestigd | Aangeleverde `b`, geen NTA-factorafleiding of onafhankelijke referentiewaarde |
| Rust servicetests | 38 geslaagd; werkpuntverhouding via HTTP met `annualPerformanceAvailable=false`, geen onterecht BENG-resultaat, afwijzing van somoverloop met HTTP 422, losse warmtepomp inclusief hulpcomponent, systeem- en bronventilatiekoppeling, afwijzing van onvolledig ventilatierecord, verkeerd gespeld veld, onvolledige referentiecase, geldige/ongeldige directe transmissiediagnose en maandflow, W/K-vergelijking inclusief verschiloverloop met HTTP 422 en geclassificeerde projectdiagnose met puntbrug via HTTP | Geen normatieve resultaatvergelijking |
| React/Vitest | 212 geslaagd met `--maxWorkers=1`, inclusief opgeslagen forfaitaire COP-tabelinvoer en formulierdiagnose, opgeslagen gemeten warmtepomp-hulpenergie-invoer, bronverwijzingen en formulierdiagnose, werkpuntverhouding met scopewaarschuwing, blokkering van dubbelzinnige werkpunten, bronventilatiekoppeling, zelfstandig invoerdossier zonder berekening, HTML-escaping van projectgegevens, registerrecord-editor, grens- en puntbruginventariseditors, auditpaneel met buiten-diagnose, parserdetails, prestatiepunt- en dragercontrole, hulpcomponent- en koppelingeditor, behoud bij heropenen, weigering van genegeerde installatiedetails, resultaatwissen bij projectwijziging, warmtepompinventaris, onverwarmde-ruimteinvoer en blokkering van de oude calculator, schiloppervlaktetabel, invoercontrole, rapport-/IFC-uitvoer en UNIEC3-invoerconcept | Geautomatiseerde componentinteractie, geen visuele handcontrole; de volledige suite slaagde inclusief de JSON-import, verversing van oude resultaten en foutweergave in het gaswarmtepomp-referentiepaneel |
| Gasgedreven warmtepomp, concepttabellen 9.27/9.29 | Rust-core controleert beide aandrijvingen, vijf bronkeuzes in vier tabel-9.29-rijen en de drie woningbouwrijen van tabel 9.27, telkens zes temperatuurbanden. Tabel 9.27 vereist collectieve woningbouw tot en met 25 kW; bodem/grondwater vragen aangeleverde `csource` plus bewijs. Opgeslagen invoer wordt aan toestel-ID, aandrijving, bron, afgifte en gebouwfunctie getoetst. UI-inventaris bewaart en verwijdert de invoer; het invoerdossier toont bewijsbronnen met HTML-escaping. Gebouwde API en MCP geven voor absorptie/afvoerlucht bij 40 °C tabel-9.29-COP 2,4 en voor een collectieve woningbouw-bodemwarmtepomp van 25 kW bij 35 °C tabel-9.27-COP 1,3, met aangeleverde csource 1,1 gecorrigeerd tot 1,43; bij 55,01 °C of ontbrekend correctiebewijs HTTP 422/MCP-fout zonder COP. Opgeslagen projectinvoer levert bij beide adapters `structurally_valid`, een ongeverifieerd-waarschuwing en `calculationAvailable=false`; een toestelmismatch geeft `invalid`. Gasinput, geïntegreerde hulpenergie, definitieve editie en BENG blijven niet beschikbaar; een losse §9.6.8.2-diagnose berekent toestelhulpstroom uit aangeleverde maanden. | Tabelprioriteit 9.27/9.29 bij collectieve woningbouw, definitieve bijlage-V-factor, exacte definitieve norm, energiedragercorrecties, bronhulpenergie en onafhankelijke actuele EDR ontbreken. Geen energielabel. Zie [conceptcontract](nta8800-gaswarmtepomp-forfait-concept.md). |
| Gebouwde API en MCP: hybride grenzen | BCRG 20240234GK Xtend 5 SE met COP-grens 2,0 en maximaal 55 °C: beide gebouwde adapters melden `structurally_valid`, `calculationAvailable=false`, `heat_pump_operating_limits_unimplemented` en editiewaarschuwing; UI-editor en invoerdossier getest | Geen COP-curve, schakelstrategie, tweede-toestelberekening of actuele EDR-case |
| Gebouwde API/MCP-gemeten warmtepompcomponenten | Conceptvergelijkingen 9.86–9.88 leiden uit 10 W stand-by, 200/90 W afgiftepomp, 300+300/600 s, modulatie 0,5 en 2 kW elektrisch A=87,6, B=0,19 en C=0,5 af; bij 100 kWh generatoringang per maand volgt 26,3 kWh/maand en 315,6 kWh/jaar. Nul gemiddelde aan-tijd geeft HTTP 422 zonder coëfficiënten. | Aangeleverde metingen en bijlage-O-tijd; conceptfactoren 1,45/0,5, geen actuele EDR of definitieve norm |
| Gebouwde API/MCP: 25 kW-tabelgrens | Releasebinaries: individueel woongebouwtoestel met opgegeven 25,00 kW, bewijsbron en tabel 9.27 geeft via HTTP en MCP `structurally_valid` zonder waarschuwing voor ontbrekend scopebewijs; 25,01 kW met dezelfde tabel geeft `invalid` en `forfait_heat_pump_draft_input_invalid`. HTTP `/calculate` blijft 501. | Vermogen en individueel/collectief zijn door de invoerder opgegeven; de werkelijke thermische vermogensdefinitie en systeemgrens zijn niet extern geverifieerd |
| Gebouwde API/MCP met opgeslagen COP-tabelinvoer | Een woonproject met losse elektrische buitenlucht/waterwarmtepomp, 35 °C-ontwerpaanvoer en tabelbewijs geeft via de nieuwe releasebinaries op HTTP `/validate` en MCP `validate_project` `structurally_valid` met `forfait_heat_pump_draft_unverified` en `calculationAvailable=false`; HTTP `/calculate` geeft 501 `calculation_unavailable`. | De tabelscope en bron zijn opgegeven, maar capaciteit, definitieve norm, jaarprestatie en EDR-toets ontbreken |
| Gebouwde API/MCP: collectieve bronklasse vanaf 20 °C | Bij opgegeven 20 °C en klasse 20–<40 °C zonder bronkwaliteitsverklaring kiezen beide releaseadapters `selectedSource=groundwater_below15_c` met `sourceFallbackApplied=true`. De woningbouwdiagnose zonder broncorrectie geeft HTTP 422/MCP `isError=true` zonder COP; met opgegeven correctie 1,0 geven beide concept-COP 4,5 bij 35 °C. Met een opgegeven verklaringreferentie kiezen beide de collectieve rij en concept-COP 5,1. `bengCalculationAvailable=false` blijft gelden. | Verklaring, brontemperatuur en correctie zijn door de invoerder opgegeven; authenticiteit en fysieke bronkoppeling zijn niet onafhankelijk gecontroleerd. |
| Gebouwde API/MCP: warmtepomp-conceptformule 9.62 | Met aangeleverde twaalf maanden generatorwarmte van 1.000 kWh, een collectieve bron van 15–<20 °C en ontwerpaanvoer 35 °C selecteert Rust COP 4,8. Volgens §9.6.8.1.1.2.3 wordt de bronwarmte `791,666… kWh`; formule 9.62 geeft `1.000/4,8 − 791,666…×0,022 = 190,916… kWh` per maand. De opnieuw gebouwde release-API en MCP-server geven beide `190,91666666666669 kWh` voor maand 1 en weigeren negatieve generatorwarmte zonder maandgetallen. Het React-paneel vraagt twaalf maanden en bronverwijzingen en toont alleen conceptdeelwaarden. | Generatorwarmte is aangeleverd; de bronwarmte is afgeleid. Geen onafhankelijke actuele EDR-uitkomst, generatorverdeling, hulpenergie, jaarbalans, BENG of label. |
| Gebouwde API/MCP: generatorverdeling en hybride conceptketen | Bij 1.000 kWh aangeleverde knooppuntwarmte per maand, een warmtepomp van 4 kW en ketel van 6 kW verdelen beide releaseadapters januari als 750/250 kWh en mei als 980/20 kWh. Met tabel-COP 4,8 geeft de gekoppelde route in januari 143,1875 kWh warmtepompelektriciteit; een binnen opgestelde HR107-bijverwarmingsketel bij LT geeft `250/0,95 = 263,1579` kWh conceptaardgas. De keteltabelroute geeft via beide gebouwde adapters `η=0,95`. Met gemeten warmtepompcomponenten (10 W stand-by, 200/90 W afgiftepomp, 300+300/600 s en modulatie 0,5) is de gekoppelde warmtepomphulpstroom `87,6/12 + 0,19×143,1875/(0,5×2) = 34,505625` kWh in januari. De individuele forfaitaire hulpenergie bedraagt bij onbekend bouwjaar `87,6/12 + 0,132×263,1579/(0,4×24) = 10,9184` kWh elektriciteit in januari; bij opgegeven bouwjaar vanaf 2015 is dit 7,2684 kWh. Productgebonden afschakelgrenzen geven HTTP 422/MCP `isError=true` zonder deelresultaat. Het hybride React-formulier verlangt nieuwbouwbevestiging en bronverwijzingen, toont beide afzonderlijke hulpstromen wanneer de warmtepompmetingen beschikbaar zijn en vraagt expliciet of een waakvlam aanwezig is; die route wordt geweigerd. | Openbaar consultatieconcept, aangeleverde knooppuntwarmte/vermogens/rendementen; geen waakvlam, ontbrekende warmtepompmetingen of bronpomp/-ventilator, onafhankelijke actuele EDR, definitieve norm, BENG of label. Zie [generatorverdeling](nta8800-generatorverdeling-concept.md). |
| Gebouwde API/MCP: onbekend bodem-/grondwatertype of -temperatuur | Release-API en MCP kiezen bij expliciet `ground_or_groundwater_unknown` de bodemrij met `sourceFallbackReason=ground_or_groundwater_unknown`; bij 35 °C en correctie 1,0 is de woningbouw-COP 3,8. Voor utiliteit zonder broncorrectie is de tabel-COP 3,4. Beide geven `bengCalculationAvailable=false`. De projectaudit meldt de terugval en wijst deze klasse bij een buitenluchttoestel af. | Het bewijs dat de fysieke bron bodem of grondwater is, wordt alleen als invoerverwijzing bewaard. Geen onafhankelijke bronverificatie, jaarprestatie of definitieve normtoets. |
| Gebouwde API/MCP: hogere woningbouwrij 9.28 | Releasebinaries: buitenlucht/water bij 35 °C met opgegeven NEN-EN 14511-2:2022-rapport en COP-punten 2,76 / 2,86 / 1,91 geeft via HTTP 200 en MCP tabel-COP 3,35 met `applicabilityVerified=false` en `bengCalculationAvailable=false`. COP 2,75 bij A7(6)/W45 geeft HTTP 422 en MCP `isError=true`, beide zonder COP. | Testcijfers, product en rapport zijn opgegeven; rapportauthenticiteit, definitieve norm, seizoensprestatie en EDR zijn niet onafhankelijk gecontroleerd |
| Gebouwde API/MCP-forfaitaire COP-concepttabellen | De gebouwde API kiest bij buitenlucht/woning en 35 °C tabel-COP 2,40; net boven 35 °C 2,35; bij 71 °C HTTP 422 zonder COP. MCP kiest bij oppervlaktewater/utiliteit en 45 °C tabel 9.29 en COP 3,70, met `bengCalculationAvailable=false`. | Alleen openbare concepttabellen 9.27/9.29; tabel-/toesteltoepasselijkheid, jaarprestatie en actuele EDR niet vastgesteld |
| Gebouwde API/MCP met opgeslagen meetinvoer | Een volledig warmtepomprecord met twaalf maanden gemeten hulpenergie-invoer geeft bij HTTP `/validate` en MCP `validate_project` `structurally_valid`, `heating_aux_measured_draft_unverified` en `calculationAvailable=false`. De API-diagnose geeft 315,6 kWh/jaar met `bengCalculationAvailable=false`. | Synthetische meetinvoer; geen onafhankelijke EDR-deeluitkomst of attest |
| Gebouwde API/MCP-concepthulpenergie warmtepomp | A=60 kWh/jaar, B=0,1 kW, C=0,5, nominaal elektrisch 2 kW en 100 kWh generatoringang per maand geeft 15 kWh/maand en 180 kWh/jaar; nul C geeft HTTP 422 zonder deelgetal. | Alleen conceptvergelijking 9.85 en opgegeven coëfficiënten; bronpomp/ventilator uitgesloten, geen actueel EDR of definitieve norm |
| Gebouwde API/MCP-conceptindicatoren | Synthetische 100 m², 1.234,001 kWh C1-behoefte, 2.131,001 kWh `EPtot` en 1.000 kWh `EPrenTot`: 12,35 en 21,32 kWh/m²·jaar, 31,9% en 10,00 kWh/m²·jaar. Een onvolledig EMG-paar levert HTTP 422 zonder getallen. | Alle jaartotalen en `Ag;tot` zijn aangeleverd; geen actuele EDR of gecontroleerde einduitgave |
| Gebouwde API/MCP-conceptdiagnose BACS | Een utiliteitssysteem van 291 kW met regelklasse C en energiemanagementklasse C geeft voorlopig factor 1,05; ontbrekende klasse geeft geen factor; 0 kW geeft HTTP 422. Per-systeemgrens en onbekend vermogen door kerntests afgedekt. | Openbaar concept §5.5.8; geen definitieve norm-/EDR-verificatie of wettelijke plichttoets |
| Gebouwde API/MCP-conceptcompositie diensten | Synthetische elektriciteitsmaand met fBACS=1,05 geeft in beide adapters 177,6 kWh, na twaalf maanden 2.131,2 kWh en 21,32 kWh/m²·jaar bij opgegeven 100 m²; verkeerde drager geeft HTTP 422 zonder deeluitkomst | Conceptvergelijkingen; geen gecontroleerde definitieve NTA-editie of onafhankelijk EDR |
| Gebouwde API/MCP-conceptdiagnose finale energie | API: synthetisch 1.200 kWh elektriciteit plus 2.400 kWh gas en 120 kWh zonneboilerbijdrage geeft 3.600 kWh finaal en 3.720 kWh EED; onvolledige inventaris houdt EED leeg en dubbele maand geeft HTTP 422 zonder uitkomst. MCP-stdio-handshake/toolcall geeft 1.200 kWh en bij opgegeven 100 m² een voorlopige indicator 12,00 kWh/m²·jaar; `finalEditionVerified=false`. De gebouwde API bewaart 0,1 + 0,1 + 0,1 als 0,30 en rondt na 0,001 extra op tot 0,31; nul m² geeft HTTP 422. | Conceptformules; geen gecontroleerde definitieve NTA-editie of onafhankelijk EDR |
| Gebouwde API en MCP: tapwatertestinvoer | BCRG 20260143GG M/L geeft via beide adapters twee ruwe verhoudingen met `annualPerformanceAvailable=false`; HTTP 422 geeft geen profielen bij nul-invoerenergie | Alleen producttestinvoer volgens NTA 8800:2025; geen actuele C1:2026-praktijk- of jaarroute |
| Gebouwde API en MCP: open eerste temperatuurklasse | BCRG 20250005GK bij opgegeven 25 °C kiest expliciet de ≤30 °C-rij: η=6,155, geen extrapolatie; zonder die klasse HTTP 422. BCRG 20260143GG geeft via MCP voor dezelfde klasse η=7,507 met `bengCalculationAvailable=false` | Alleen tabeltoepassing op invoer, geen NTA-gebouwjaarresultaat of actuele EDR |
| Gebouwde API en MCP: ruimteverwarmingstabel | BCRG 20250005GK geeft uit vier punten η=5,68575, fractie=0,996 en hulpenergietabelwaarde=108,5 kWh/jaar; 20260143GG geeft voor water/water η=7,2615, fractie=0,8485 en 71,25 kWh/jaar; HTTP 200, MCP-toolcall geslaagd. Buiten bereik HTTP 422 en MCP `isError=true`, zonder getal. | Verklaringen NTA 8800:2024 en 2025, tabelfragmenten; geen bewezen C1:2026-toepassing, jaarbalans of EDR-bewijs |
| Gebouwde API en MCP: tapwater-testinvoer | BCRG M/XL- en M/L-invoer uit drie verklaringen door beide nieuwe binaries gevalideerd; de gebouwde API waarschuwt voor afwijkende normeditie en de ruwe verhouding is zichtbaar met `referenceVerified=false`, `annualPerformanceAvailable=false` en `calculationAvailable=false` | Verklaringen noemen NTA 8800:2020, 2024 en 2025; geen actuele EDR-case of jaarprestatie |
| Gebouwde API en MCP: werkpuntverhouding | Beide binaries met hetzelfde project getest: structureel geldig, verhouding 3,00, `annualPerformanceAvailable=false`, `calculationAvailable=false`; MCP-handshake en `validate_project` slagen | Geen jaarweging, bronverificatie of attest |
| Projectadapter onverwarmde ruimte via gebouwde API | Expliciet gekoppeld vlak plus koudebrug met b=0,5 geeft 2,1 W/K; onjuiste ruimteverwijzing geeft `invalid` zonder diagnose | Geleverde factor en gegevens zijn niet onafhankelijk geverifieerd |
| Productie-webbuild | `npm run build` geslaagd | Bundelwaarschuwing voor grote JS-chunk |
| Lokale API | `.oes`-fixture via Vite-proxy HTTP 200; herhaalde `/validate`-smoke op de Aalten-fixture HTTP 200, `structurally_valid` en bruto/raam/restant-invoersommen van 177,60/14,30/163,30 m²; opzettelijk te groot raam en ongeldige warmtepompafgifte op `/calculate` HTTP 422; nieuwe oppervlaktewaterwarmtepomp op `/validate` HTTP 200 met één geclassificeerd toestel en waarschuwing, `/calculate` HTTP 501; dubbele/onbekende zone en ongeldige COP/dekkingsfractie geven `status=invalid`, waarbij `/calculate` HTTP 422 retourneert; geldig prestatiepunt geeft `structurally_valid` en nulvermogen `performance_point_power_invalid`; ingebed prestatiepunt geeft waarschuwing en `/calculate` HTTP 501; verkeerd gespeld veld geeft HTTP 400 en de veldnaam; gebouwde API met twee afzonderlijk eindige vloeroppervlakten van `1e308` geeft op `/calculate` HTTP 422, `floor_area_sum_invalid` en `calculationAvailable=false` | Lokale ontwikkelroute; geen publieke auth/deployment |
| Thermische API-smoke | Bestaande Aalten-fixture op `/calculate`: HTTP 501 `calculation_unavailable`; dezelfde invoer met raam-g=1,4: HTTP 422 `invalid_project_input`, inclusief `window_g_invalid` | Fysieke invoercontrole; geen normatieve prestatieberekening |
| Directe transmissie API/MCP | Handvoorbeeld via HTTP 200: 4,37 W/K, `referenceVerified=false` en `bengCalculationAvailable=false`; U=0 via HTTP 422 zonder getal. MCP-handshake en tool-lijst slagen; geldige toolcall 2,00 W/K, ongeldige toolcall `isError=true` en geen getal | Alleen rekenkundige diagnose met opgegeven buitencomponenten; geen primaire normreview of onafhankelijke expected value |
| Maandflow API/MCP | Gebouwde API met twaalf opgegeven maanden: HTTP 200, 2 W/K, 2 kWh per maand en 24 kWh getekende jaarsom; elf maanden: HTTP 422, geen energie. MCP-handshake, tool-lijst en toolcall geven eveneens 24 kWh, `referenceVerified=false` | Door aanroeper opgegeven temperaturen/uren; geen NTA-vraag of onafhankelijke referentie |
| Registerrecord API/MCP | Gebouwde API `/validate`: oude verklaring zonder registerrecord geeft `structurally_valid` met `quality_declaration_registry_record_missing`; volledig record zonder die waarschuwing; leeg productveld geeft `invalid` met `quality_declaration_registry_field_required`. MCP `/validate` geeft dezelfde waarschuwing voor het oude record. | Vorm- en aanwezigheidstoets; geen live BCRG-controle, productmatch of normtoepassing |
| Diagnostische W/K-vergelijking | Rust vergelijkt vier aangeleverde deelwaarden; HTTP-test geeft `compared_fail` bij afwijkend totaal. De gebouwde lokale API meldt de capability en geeft `compared_pass` voor een passend handvoorbeeld; MCP-handshake/toolcall geeft dezelfde status met `referenceVerified=false` | Verwachtingen en toleranties zijn zelf opgegeven; geen bronverificatie of attest |
| W/K-verschiloverloop via gebouwde API/MCP | Twee afzonderlijk eindige waarden met een overlopend absoluut verschil geven in de gebouwde API HTTP 422, `invalid_case`, `metric_difference_overflow` en nul meetposten; MCP meldt `isError=true` met dezelfde issuecode en zonder meetposten | Numerieke veiligheid van de diagnostische vergelijker; geen normatieve validatie |
| Systeemkoppelingen via MCP | Geldige koppeling naar bestaande ketel: `systemLinkCount=1`, `structurally_valid`, `calculationAvailable=false`. Ontbrekend doel: `status=invalid` en `system_link_target_missing` | Topologiecontrole; geen hybride of cascaderoute |
| Bronventilatiekoppeling via gebouwde API/MCP | Een afvoerluchtwarmtepomp gekoppeld aan `vent-1` geeft in beide gebouwde binaries `structurally_valid`, `systemLinkCount=1`, `calculationAvailable=false`; API HTTP 200 en MCP-protocolhandshake slagen | ID- en typecontrole; geen debiet, warmtebeschikbaarheid of seizoensprestatie |
| Ongeldig ventilatierecord via gebouwde API/MCP | Een gekoppeld ventilatierecord met ID maar zonder type geeft in de gebouwde API op `/calculate` HTTP 422, `system_type_required` en `calculationAvailable=false`; MCP `/validate` geeft `status=invalid` met dezelfde code | Structurele systeeminvoer; geen controle van de werkelijke ventilatie-installatie |
| Projectgrens API/MCP-smoke | Bestaande Aalten-fixture op `/validate`: 0/7 vlakken en 0/3 lineaire bruggen geclassificeerd, puntinventaris onbevestigd, `directOutdoorDiagnostic=null`. HTTP-servicetest met bevestigde puntinventaris en een buitenpunt: 3,99 W/K, waarvan 0,04 W/K puntbijdrage; `referenceVerified=false`, `calculationAvailable=false` | Synthetische invoer; geen toets van werkelijke thermische grenzen of normresultaat |
| Puntinventaris via gebouwde API/MCP | Dezelfde synthetische buitenwand met χ=0,04 W/K: zonder bevestiging `directOutdoorDiagnostic=null`; na `pointBridgeInventoryComplete=true` geven API en MCP 2,04 W/K totaal en 0,04 W/K puntbijdrage, `referenceVerified=false` | De volledigheidsbevestiging is door de gebruiker opgegeven; geen onafhankelijke broncontrole |
| Prestatiepunt-drager via gebouwde API | Een elektrische compressor met een gaswerkpunt geeft op `/calculate` HTTP 422, `performance_point_carrier_mismatch` en `calculationAvailable=false` | Fysieke consistentie; geen normatieve warmtepompberekening |
| MCP stdio | Handshake, `calculate_beng` met `calculation_unavailable` en `unattested`, `validate_project` met `invalid_project_shape` en de verkeerd gespelde veldnaam, en `audit_reference_case` met zes ontbrekende-bewijsproblemen geslaagd | Geen NTA-rekenuitkomst of onafhankelijke verificatie |
| Referentie-API | `POST /v1/nta8800/reference/audit` HTTP 200 voor onvolledig diagnostisch geval; `manifestComplete=false`, `referenceVerified=false`, `calculationAvailable=false` | Alleen administratieve veldcontrole; geen resultatenvergelijking |
| Manifestvingerafdruk via gebouwde API/MCP | Gebouwde API houdt projecthash gelijk en wijzigt manifesthash bij gewijzigde verwachte waarde; MCP-handshake en `audit_reference_case` tonen `manifestFingerprint` en `referenceVerified=false` | Hash maakt een case herkenbaar, maar verifieert bron, rechten, normtoepassing of verwachte waarde niet |
| Desktop-devbuild | Linux `.deb` gebouwd; pakketomschrijving benoemt de experimentele invoeraudit | Debugpakket; geen releasecertificaat |
| Vernieuwde desktop-devbuild `fc5a8a5` | [Bouw- en testdossier](nta8800-build-verificatie-2026-10-03-fc5a8a5.md): schone tweede bouw, 703 kerntests, 53 servicetests en 10 gerichte frontendtests groen; SHA-256 en pakketmetadata vastgelegd | Debugpakket; geen visuele acceptatie, onafhankelijke actuele EDR-toets of attest |
| Visuele UI-controle | Niet uitgevoerd | Browsertoegang tot `127.0.0.1:3006` door opgeslagen gebruikersinstelling geblokkeerd; deze blokkade is niet omzeild |
| NTA-invoereditor bij projectwissel | Het prestatiepaneel sluit zowel het formulier als de JSON-editor bij een andere project-ID en wist de oude concepttekst. Gerichte componenttest: 7/7 geslaagd; TypeScript- en Vite-build geslaagd. | Geen visuele desktopcontrole; de Rust-kern valideert de opgeslagen invoer pas bij de volgende berekening. |
| Zijbalk met indicatief label bij projectwissel | De respons draagt de projectreferentie van de aanvraag; een oudere BENG-/labeluitkomst wordt bij een gewijzigde projectreferentie direct verborgen. Gerichte previewtests: 5/5 geslaagd; TypeScript- en Vite-build geslaagd. | Geen visuele desktopcontrole of onafhankelijke toets van de energie-uitkomst. |

## Nog nodig voor inhoudelijke en externe verificatie

1. ~~De volledige NTA 8800:2025+C1:2026 beschikbaar maken~~: gedaan op 2 oktober 2026 (gelicentieerde PDF, buiten de repository). Per formule en tabel zijn bron en pagina vastgelegd.
2. Officiële, onafhankelijk vastgestelde referentiegevallen per bouwfysische en installatieroute verkrijgen. Minstens één geval per warmtepompvariant, inclusief hybride, collectief, tapwater, reversibel en kwaliteitsverklaring.
3. ~~Rust-rekenmodules implementeren~~: gedaan, zie de samenvatting bovenaan.
4. Deelresultaten en grensgevallen toetsen tegen de referentiegevallen, toleranties verklaren en regressie/versieprovenance vastleggen.
5. Visuele en toetsenbordcontrole van de desktop-UI uitvoeren wanneer UI-toegang beschikbaar is; controleer kleine schermen, kleurcontrast, tabvolgorde en validatiemeldingen.
6. De toepasselijke editie van ISSO-publicatie 54 vaststellen: [InstallQ](https://installq.nl/controllers/brl) noemt 2024, [BouwZo](https://bouwzo.nl/search?Publisher=ISSO) vermeldt een uitgave van 14 november 2025. Verkrijg die publicatie en de definitieve [BRL 9501:2026](https://bouwzo.nl/reader/publicatie/brl-9501/2026), bevestig de verplichte W/U-testomvang en bereid daarna de attestaanvraag, het kwaliteitsproces en de externe toets voor. Pas na positief attest een geattesteerde scope publiceren. De bronstatus staat in het [bronnenregister](nta8800-bronnenregister.md).

Het onderscheid tussen de drie bestaande diagnostische `.oes`-gevallen en nog te verkrijgen normatieve referentiegevallen staat in [het referentieprotocol](nta8800-referentieprotocol.md).
De voor het formele dossier benodigde tests, versiegegevens, registratie en kwaliteitsbewaking staan in [het attestdossier](nta8800-attestdossier.md). Het publiek beschikbare InstallQ-document is voor deze inventaris gebruikt; de definitieve 2026-uitgave moet nog worden gecontroleerd.

## Lokaal reproduceren

```bash
cargo test --manifest-path crates/nta8800-core/Cargo.toml
cargo test --manifest-path crates/nta8800-service/Cargo.toml
cargo fmt --manifest-path crates/nta8800-core/Cargo.toml --check
cargo fmt --manifest-path crates/nta8800-service/Cargo.toml --check
cargo clippy --manifest-path crates/nta8800-core/Cargo.toml --all-targets -- -D warnings
cargo clippy --manifest-path crates/nta8800-service/Cargo.toml --all-targets -- -D warnings
CARGO_TARGET_DIR=target/msrv-core cargo +1.77.2 test --manifest-path crates/nta8800-core/Cargo.toml
npm test -- --run --maxWorkers=2
npm run build
npm run tauri build -- --debug --bundles deb
```

Start voor de browserontwikkelroute de lokale Rust-API op poort 3007 en daarna Vite op poort 3006. In de desktop-app gebruikt het controlescherm de Tauri-command en is de losse API niet nodig.

Het vernieuwde debug-`.deb`-pakket is tevens met `dpkg-deb -I` geïnspecteerd: `amd64`, versie `0.1.6-alpha`, met pakketafhankelijkheden `libwebkit2gtk-4.1-0` en `libgtk-3-0`. `ldd` vindt de WebKit/GTK-bibliotheken op de bouwhost. Dit is een pakket- en koppelingstoets, geen visuele desktopacceptatietest.

## Controle van geleverde Linux-bestanden

SHA-256 van de bij deze verificatiestatus geleverde bestanden:

| Bestand | SHA-256 |
| --- | --- |
| `open-energy-studio_0.1.6-alpha_debug_amd64.deb` | `92a22694a87dc4fadfb3e0345fe28c089b441dab5c50fb83c306d166d51d70cf` |
| `oes-nta8800-api-linux-amd64` | `631f6e33fa6d7596f1bb56e93657bc047e18e281ba4a39716cc0194c009d393f` |
| `oes-nta8800-mcp-linux-amd64` | `1ccb4587376e441e32fb13bbe45c1da3a99932cfa6f6333233f666530e89ea9f` |

Wijzigingen die de uitkomst of de status van opgeslagen projecten veranderen, staan in de [releasenotes](nta8800-releasenotes.md).
