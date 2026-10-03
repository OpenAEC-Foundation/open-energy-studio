# NTA 8800 maandelijkse warmte- en koudebehoefte (hoofdstuk 7)

Rust-module `monthly_demand` berekent per rekenzone de maandelijkse netto warmtebehoefte `Q_H;nd` en koudebehoefte `Q_C;nd` volgens de maandmethode van NTA 8800:2025+C1:2026 hoofdstuk 7. Dit is de eerste schakel van de rekenruggengraat: gebouw → behoefte → installaties → primaire energie → BENG. Het resultaat heeft status `calculated_unverified`. `referenceVerified`, `finalEditionVerified` en `bengCalculationAvailable` blijven `false`.

## Bron

De module is op 2 oktober 2026 nagelopen tegen de gelicentieerde normtekst (NTA 8800:2025+C1:2026, pdf in de lokale normenmap; niet in de repository). Formules die in de tekstlaag onleesbaar zijn, zijn gecontroleerd op de gerenderde pagina's. Paginanummers hieronder zijn de gedrukte nummers; die zijn gelijk aan de pdf-pagina's. Tabellen 17.4, 7.7, 7.8 en 7.9 zijn uit de normtekst gegenereerd en steekproefsgewijs nagerekend.

| Onderdeel | Norm (pagina) | Waarde / formule |
|---|---|---|
| Warmtebalans | 7.1–7.3 (p. 165) | `γ_H ≤ 0` met `Q_H;gn > 0` → 0; `γ_H > 2,0` → 0; anders `max(0, Q_H;ht − η_H;gn·Q_H;gn)` |
| Koudebalans | 7.6/7.7 (p. 166) | `1/γ_C > 2` → 0; anders `a_C;red·(Q_C;gn − η_C;ht·Q_C;ht)`, ondergrens 0 |
| Benutting | 7.46–7.55 (p. 207–209) | 7.48 `η_H = 1/γ` bij `γ ≤ 0` en positieve winst (gerapporteerd; behoefte 0), 7.49 `η = 1`; 7.54 `η_C = 1` bij `γ_C ≤ 0` |
| Parameter a | 7.51/7.56 | `a_H/C = 1,0 + τ_H/C/15 h` |
| Tijdconstanten | 7.57/7.58 (p. 210) | `τ_H = (C_m/3600)/(H_tr + H_H;g;adj + H_H;ve)`, `τ_C` met `H_C;g;adj` en `H_C;ve` |
| Transmissie | 7.14/7.15 (p. 170) | `(H_tr·(θ_calc − θ_e;mi) + H_g;an;mi·(θ_calc − θ_e;an))·0,001·t` |
| Ventilatie | 7.18–7.20 (p. 174–176) | `H_ve = Σ H_k·b_v;k`, `b_v = (θ_set;stc − θ_sup)/(θ_set;stc − θ_e)`, `Q_ve = H_ve·(θ_calc − θ_e)·0,001·t`, apart voor warmte en koude |
| Rekentemperatuur verwarming | 7.59–7.73 (p. 211–215) | `θ_calc;H = a_H;red·(θ_set;H − θ_e) + θ_e` met nacht- en weekendverlaging per gebruiksfunctie |
| Nivellering woningbouw | 7.76–7.79 (p. 216) | `Δθ = (0,8·f_sp)(f_sp·H_e;spec)(θ_stc − θ_e)/(f_sp·H_e;spec + 2,0)`, `f_sp` 0,5 woongebouw / 0,6 overige woning |
| Niet-continu koelen | 7.74/7.75 (p. 215) | `a_C;red = 1 − f + 0,3·f`, `f = t_C;red;wknd/168` |
| Gebruiksfunctie | tabel 7.13–7.15 (p. 216–220) | setpoints (woning 20, sport 16, zorg met bed 22, overig 21; koeling 24 °C), verlaagd setpoint (16 °C, sport 14 °C) en uren verlaging |
| Warmtecapaciteit | tabel 7.10, 7.45 | `D_m` 55/80 … 250/450 kJ/(m²K); `C_m = D_m·1000·A_g` |
| Interne winst woning | 7.21–7.24 | `180·N_woon·N_P·0,001·t`; `N_P` per banden ≤30, 30–100, >100 m² |
| Zonwinst ramen | 7.32, 7.40, 7.42/7.43 | `0,90·g_n·A·(1−F_F)·F_sh;obst·r_sh·I_sol·t·0,001 − Q_sky`, apart voor warmte- en koudebalans |
| Belemmering | §17.3, tabel 17.3–17.15 (p. 698–763) | Eén situatie per raam (zie hieronder); hellingen nemen de dichtstbijzijnde kolom van tabel 17.4 (bij gelijke afstand de hoogste); onder 15° geldt het zichtveld zuid; of `declared` |
| Beweegbare zonwering | 7.42/7.43, tabel 7.7–7.9 (p. 197–202) | `r_sh = (1−f_sh;with)+f_sh;with·F_c`; lineaire interpolatie tussen 0/45/90/180°; `f_sh;with = 0` voor warmte alleen bij woningen met handbediening of automatiek volgens ISO 52016-3 |
| Zonwinst opaak | 7.33, 7.6.6.3 | `0,6·R_se·U·A·I_sol·t·0,001 − Q_sky`, `F_sh;obst = 1` |
| Hemelstraling | 7.39, 7.6.6.4 | `F_sky·R_se·U·A·4,14·11·t·0,001`; `F_sky` 1 (≤5°) / 0,75 (≤75°) / 0,5 (≤90°) / 0 (overhellend) |
| Klimaat | tabel 17.1/17.2 | De Bilt; ook `θ_e;argII`, `u_site` en `θ_ODA;preh;WTWC` als constanten voor hoofdstuk 11 |

## Invoer

Verplicht naast de bestaande velden: `usageFunction` (tabel 7.13–7.15) en, bij de woonfunctie, `dwellingType` (`apartment_building` of `other`). De opgegeven setpoints moeten gelijk zijn aan tabel 7.13 voor de gebruiksfunctie (`setpoints_table_7_13_mismatch`). Interne winst `residential` mag alleen bij de woonfunctie. De zonweringsregeling moet bij de functie passen.

De transmissie kent twee routes (`transmission.method`):

- `explicit`: `H_tr` exclusief grond (W/K) met bron, plus optioneel een grondroute met twaalf maandwaarden `H_g;an;mi` (bijlage D.1) en de seizoenswaarden `H_H;g;adj` (D.2) en `H_C;g;adj` (D.3);
- `components`: de module stelt `H_tr` zelf samen uit directe buitenelementen en koudebruggen (8.1) en onverwarmde ruimtes met opgegeven `b`. Vloeren op grond gaan via §8.3 en bijlage D (module `ground`):
  - stationair: 8.30, 8.32, λ = 2,0, 8.40/8.41 en de vloerrand: `edgeThermalBridges` `detailed` (8.36, `Σ ℓ·ψ_gr`) of `forfait` (8.37, `+ 0,5·P`);
  - periodiek: D.5 `H_pi`, D.6 `H_pe` zonder randisolatie, D.7 (horizontaal) of D.8 (verticaal) met D.9 `d'` bij `edgeInsulation`, het minimum bij beide; δ = 3 m;
  - D.4 maandwarmtestroom met `θ̄_i` = setpoint verwarming, `θ̄_e` = 10,67 °C, amplitudes 2 en 7,9 K, τ = 1 en tabel D.1 (randgeïsoleerd: β = 2, anders β = 1);
  - D.1 `H_g;an;mi = Φ_mi/(θ_set;H − θ_e;an)`, D.2/D.3 de seizoenswaarden.

Ventilatiestromen hebben per maand een geleiding en optionele toevoertemperatuur voor de warmtebalans, en optioneel `coolingConductanceWPerK` en `coolingSupplyTemperatureC` voor de koudebalans (leeg = gelijk aan de warmtewaarden). Hoofdstuk 11 levert deze straks per balans.

Twee optionele bijlagen (beide met bron):

- **Bijlage B (p. 772)**: `thermalMass.annexBElements` vervangt tabel 7.10. Per constructieonderdeel geef je de oppervlakte en de lagen vanaf de zonezijde (dikte, λ, ρ, c). De werkzame dikte loopt vanaf het binnenoppervlak zolang R < 0,25 m²K/W, met een maximum van 100 mm en van de halve constructiedikte. Een binnenwand met `bothSides` telt van twee kanten. `D_m = Σ ρ·c·d·A / A_g`.
- **Bijlage A (p. 766–771)**: `windows[].dynamic` geeft een dynamisch transparant element. Methode A (`weighted_states`) middelt g en U per maand over de toestanden. Daarvoor geef je per maand het aandeel van `Σ I_sol·Δt` (A.2) en van `Σ Δθ·Δt` (A.1) per toestand op. Methode B (`single_state`) gebruikt één toestand. De maandelijkse g werkt in de zonwinst; het verschil met de nominale U van het raam wordt per maand bij `H_D` opgeteld. Een toestand mag ook `tauSolar` en `tauVisual` hebben (τ_sol en τ_vis, A.3/A.4). De kern middelt die dan met de zonnegewichten (`tau_solar_for_month`, `tau_visual_for_month`). Ontbreekt τ bij één toestand, dan is er geen maandwaarde. Hoofdstuk 14 heeft geen ingang voor τ_vis: 14.38 bevat geen doorlatingsfactor en 14.41 zet τ_D65 op 0,6 (p. 673). De waarden zijn dus alleen vastlegging en het formulier vraagt ze niet. Een raam met beweegbare zonwering valt zelf onder bijlage A (A.2). Een dynamisch raam mag daarom niet ook `movableShading` (7.42) hebben (`window_dynamic_and_shading_exclusive`).
- **Bijlage A stap 2 (p. 770)**: de norm zegt alleen dat correctiefactoren voor dynamische effecten uit uurberekeningen "kunnen worden afgeleid". Hij geeft geen waarden en geen manier om de aandelen per toestand uit maandgegevens af te leiden. `correction` (bij beide methoden) neemt daarom twaalf opgegeven factoren voor U (`uFactors`) en voor g (`gFactors`) met een bron (`sourceReference`). Die factoren vermenigvuldigen de maandwaarden. Minder of meer dan twaalf waarden, of een waarde die niet positief is, geeft `dynamic_correction_invalid`; zonder bron volgt `source_reference_required`. Zonder `correction` zijn de factoren 1.
- **Bijlage A in de projectroute**: `ntaCalculation.dynamicWindows` koppelt per project-raam-id (`windowId`) een `dynamic`-blok met methode A of B en eventueel `correction` aan een buitenraam. De `uValue` en `gValue` van het projectraam blijven de nominale waarden in H_D. De kern vervangt ze per maand door U_mi;mn en g_mi;mn maal de stap 2-factoren. In het NTA-formulier staat dit onder "Dynamische ramen (bijlage A)", met toestanden, de twaalf maandrijen met gewichten per toestand en de correctiefactoren per maand. Fouten geven een gap op het pad in `dynamicWindows`: `dynamic_window_duplicate`, `dynamic_window_without_window` (geen buitenraam met die id), `dynamic_weights_invalid`, `dynamic_state_invalid`, `dynamic_state_required`, `dynamic_correction_invalid`, `source_reference_required`, `dynamic_value_missing` (een leeg veld, op zijn eigen pad) en `window_dynamic_and_shading_exclusive` (naast de projectbrede beweegbare zonwering).

Elk onderdeel heeft een bronverwijzing. Een onvolledige inventaris, een ongeldige waarde, een onbekend veld of een helling buiten 0–180° geeft `invalid` zonder getallen.

## Uitkomst per maand

Per balans: setpoint na nivellering, reductiefactor (`a_H;red` of `a_C;red`), rekentemperatuur, `H_ve` met `b_v`, tijdconstante, `a`, transmissie, ventilatie, warmteoverdracht, winst, `γ`, benutting en behoefte. Daarnaast de maandelijkse `H_g;an;mi`. De transmissiesamenvatting geeft `H_g` stationair, de twaalf maandwaarden en de seizoenswaarden.

## Verticale leidingen (7.3.3)

`transmission.verticalPipes` (route `components`) geeft per leiding het aantal bouwlagen van de zone, geïsoleerd of niet en het aantal zones waarlangs de leiding loopt. `H_p = Σ N_bouwlaag;j · H_p;spec;j / n_zones` met tabel 7.1 (1,8 W/K ongeïsoleerd, 0,5 W/K geïsoleerd). `H_p` telt mee in `H_tr` (7.16) en staat apart in de transmissiesamenvatting (`verticalPipeConductanceWPerK`).

Bij een fictieve leiding per toiletgroep in de utiliteitsbouw (7.17a) mag in plaats van `storeys` ook `buildingHeightM` (H van 11.2.1.2) worden opgegeven. Dan geldt N_bouwlaag = ⌊H/3⌋, met een minimum van 1. `areaShare` verdeelt H_p naar rato van de gebruiksoppervlakte over de zones van het gebouw.

`verticalPipes: []` betekent dat er aantoonbaar geen doorvoeren zijn (tabel 7.1, 0 W/K). Ontbreekt de lijst, dan is de aanwezigheid onbekend en geeft de kern `vertical_pipes_unknown`. 7.3.3 schrijft dan fictieve ongeïsoleerde leidingen voor: per bouwlaag van de rekenzone (woning buiten een woongebouw), één per woonfunctie (woongebouw) of één per toiletgroep met N = ⌊H/3⌋, verdeeld naar gebruiksoppervlakte (utiliteit). De kern leidt die niet zelf af, omdat het aantal bouwlagen, woningen of toiletgroepen niet in de invoer staat; de adviseur voert ze in. De basisopname past de standaardwaarde wel zelf toe (`vertical_pipes_unknown_one_per_storey`, `vertical_pipes_unknown_one_per_toilet_group`).

## Lineaire thermische bruggen: één methode (8.2.1, 8.3.3.1)

De forfaitaire verrekening (ΔU_for in 8.2, 0,5·P in 8.37/8.38) mag alleen voor het hele gebouw; vermenging met ψ-waarden is niet toegestaan. De kern geeft `thermal_bridge_methods_mixed` als een vloerrand forfaitair is (`edgeThermalBridges.method = forfait`) terwijl er elders ψ-waarden staan: lineaire bruggen in H_D, in de begrenzing met een onverwarmde ruimte of in een gedetailleerde vloerrand met ψ_gr. Een aansluiting vloer–gevel van een vloer op grond hoort volgens opmerking 10 van 8.2.1 niet in H_D maar in de vloerrand (8.36, ψ_gr).

## Zontoetreding: tabelwaarden (7.41, tabel 7.4–7.6, 7.4a/b)

Invoer voor de zontoetreding per raam:
- `glazing.glazingType` geeft g_gl;n uit tabel 7.4 in plaats van `gPerpendicular`. Opgeven van allebei is een fout.
- `movableShading.device` geeft F_c uit tabel 7.5/7.6. Voor zonneschermen hangt die af van de oriëntatie. Een expliciete `reductionFactor` blijft mogelijk; die wordt naar boven afgerond op 0,01.
- `glazing.fixedLouvres` vermenigvuldigt g met F_c;lam: 0,27 bij 90°, 0,15 schuin, 0,27/0,06 draaibaar open/dicht. Bij draaibare lamellen weegt `f_sh;with` uit de regeling.
- `glazing.diffusing` rekent met g = 0,75·g_alt + 0,25·g_dif (7.41).
- Zonwerend glas staat in tabel 7.4 alleen voor de utiliteitsbouw.

## Kelderdiepte per wanddeel (8.42/D.12) en vochtfactor (E.8/E.9)

- `heatedBasement.wallDepths` geeft z_j per wanddeel. Dan is z = Σℓ_j·z_j/Σℓ_j, en Σℓ_j moet gelijk zijn aan de omtrek P. `depthM` mag dan niet ook worden opgegeven.
- Voor kruipruimten en onverwarmde kelders zijn z_j een klasse (0 of 0,5 m) en h_j vast 0,125 m (8.47). D.17 en D.18 vallen daar samen met de enkele waarde.
- `moistureConversion` bij een opgegeven isolatie-λ_D rekent F_M = e^{f·(x₂ − 0)}: E.8 voor Ψ, E.9 voor u. De coëfficiënt komt uit tabel 4 van NEN-EN-ISO 10456. Deze F_M vervangt tabel E.2.

## Niet toegepast (altijd meegeleverd als `omittedCorrections`)

- terugwinbare systeemverliezen `Q_H;ls;rbl`/`Q_C;ls;rbl` en de Δη-termen van 7.3–7.5 en 7.7–7.9 (komen uit hoofdstuk 9 en 10);
- `H_A` (aangrenzende verwarmde ruimten, 8.5);
- de uitgebreide methode van §17.3.8 (uurwaarden NEN 5060) komt binnen als `declared`-factoren;
- voetnoot c van tabel 7.10: de kolomkeuze ligt bij de aanroeper;
- bijlage D voor andere vloeren dan vloer op grond (kruipruimte, kelder);
- één gebruiksfunctie per rekenzone (tabel 7.13–7.15).

## Interpretatiepunten voor de normreview

1. `θ_e;avg;an` in 7.14/7.15/7.73 en D.1–D.4 is overal 10,67 °C. §17.2 geeft geen jaarwaarde; D.4 (p. 791) noemt deze waarde "op basis van NEN 5060". Het ongewogen gemiddelde van tabel 17.1 is 10,6717 °C, het uurgewogen 10,7023 °C.
2. D.7/D.8 (p. 793) staan in de NTA met exponenten in `d_f;equi/δ`; NEN-EN-ISO 13370 (H.5.2) gebruikt daar de breedte D of diepte van de randisolatie. De gewichten (1 − e^(−d_f/δ)) en e^(−2·d_f/δ) tellen niet op tot 1, en de breedte van de randisolatie komt in de formule niet voor. Dat lezen we als normfout. De module volgt de NTA zoals gedrukt.
3. Tabel D.1 (p. 792): verticale randisolatie geeft altijd β = 2; horizontale randisolatie alleen vanaf R_n ≥ 2,0 m²K/W. De tekst kan ook zo gelezen worden dat de grens van 2,0 voor beide soorten geldt. Dat is dubbelzinnig; de kern houdt de grens alleen bij horizontale randisolatie aan.
4. 7.66–7.68: als `dθ_set − dθ_float ≤ 0` geldt `f_low = 1` vóór de regel `dθ_float = 1 → f_low = 0`. De norm noemt beide voorwaarden zonder rangorde, en bij `dθ_float = 1` gelden ze allebei (dubbelzinnig). De kern kiest `f_H;red;low = 1`. Dat is de enige fysische lezing: bij `dθ_float = 1` dekken de winsten het hele verlies, de temperatuur zakt in de verlaagde periode niet onder het verlaagde setpoint, en 7.64 geeft dan `a_H;red = 1` (geen verlaging). Met `f = 0` zou 7.65 een verlaging opleveren voor een periode waarin de zone niet afkoelt. Omdat `dθ_set` volgens 7.70–7.72 hooguit 1 is, valt elk geval met `dθ_float = 1` onder de eerste voorwaarde; de regel van 7.68 heeft in de kern dus geen eigen effect.
5. `b_v` (7.20) is ongedefinieerd als `θ_set;stc = θ_e`; de module neemt dan 1. Met tabel 17.1 en tabel 7.13 komt dat niet voor.
6. Een maand met `H_tr + H_g;adj + H_ve ≤ 0` (bijvoorbeeld door warme toevoerlucht met negatieve `b_v`) wordt geweigerd; de tijdconstante is dan niet gedefinieerd.
7. TOjuli (§5.7) gebruikt in de balans en in 5.40 de juliwaarde `H_gr;an`, en in de tijdconstante `H_C;g;adj`.
8. Bijlage B: een vrijhangend plafond met ten minste 15 % open oppervlak telt niet mee voor de weerstand vanaf het binnenoppervlak. Omdat het geen bouwconstructie is, telt de module ook de massa ervan niet mee.
9. Bijlage A: de correctiefactoren van stap 2 hebben geen forfaitaire waarde. Zonder opgegeven `correction` zijn ze 1. De kern heeft geen uurklimaat, dus de gewichten van stap 1 worden opgegeven. De nominale U van het raam moet in de transmissie-invoer staan, omdat de module per maand alleen het verschil corrigeert. TOjuli gebruikt per oriëntatie de juliwaarde `U_jul` (`transmission_u_for_month`).
10. 7.28: De legenda beschrijft `W_t` als het verlichtingsgebruik voor de vereiste lichtniveaus, wat op `W_L` (14.7) lijkt, maar het symbool en de verwijzing naar 14.2.2 wijzen naar het totaal `W_t = W_L + W_P` (14.6). De kern volgt het symbool en de verwijzing: de interne warmtelast van verlichting is `f_L·(W_L + W_P)·1000/t_an`, dus inclusief parasitaire energie (`internal_gain_w` in `lighting.rs`).

## Toetsing

Unittests met handberekeningen voor: tabel 7.10, bewonersbanden, benutting inclusief 7.48/7.54, tabellen 7.13–7.15, beide takken van de intermitterende verwarming (7.64 en 7.65) en de weekendverlaging, vrij zweven en warme maanden, nivellering 7.78, een volledig nagerekende januari, de poort 7.2, de koelpoort, warme toevoerlucht (7.54), `a_C;red` bij utiliteit, `b_v` per balans, de expliciete en samengestelde grondroute, D.4–D.9 en tabel D.1, en de foutpaden. Het synthetische geval staat in `training-data/nta8800-monthly-demand-synthetic.json`. Er is nog geen onafhankelijk referentiegeval.

## Aanroep

HTTP: `POST /v1/nta8800/demand/monthly/calculate` met `{ "input": { ... } }`. MCP en desktop: `calculate_monthly_demand`. TS: `calculateMonthlyDemandWithRust`.

## Vloer boven kruipruimte of onverwarmde kelder (8.3.4.2, bijlage D.2.2.4/D.2.2.5)

Een vloer op grond kan een `below` krijgen:

- **Kruipruimte:** U_fl volgens 8.43 met U_g (8.44, 8.45) en U_x (8.46–8.48: wand boven maaiveld met h = 0,125 m, kruipruimteventilatie met ε = 0,001 2 m²/m als forfait, u10 = 5 m/s, f_u = 0,05). Maandwaarden volgens D.13/D.14 met tabel D.1 (0, 0).
- **Onverwarmde kelder:** U_x;V volgens 8.49 met n = 0,3 als forfait. Maandwaarden volgens D.15/D.16 met tabel D.1 (0, 1).

Invoer per vloer:

- de diepteklasse z: 0 m op zand, anders 0,5 m;
- R_bf van de kruipruimte- of keldervloer (0 als ongeïsoleerd);
- R_bw en U_xw: de gevel erboven (8.34, 8.47 opmerking 3).

De vloerconstructie zelf krijgt aan de onderzijde R_se = 0,04 (8.43 via 8.2.2.2.1 en tabel C.2; zie de interpretatie hieronder). Randisolatie volgens D.7/D.8 hoort alleen bij een vloer direct op de grond. Een verwarmde kelder krijgt `heatedBasement` met de werkelijke diepte z en de R_c van de kelderwanden:

- H_g volgens 8.39, met U_fl uit 8.40/8.41 bij diepte z en U_bw uit 8.45;
- forfaitair volgens 8.38: 0,5·P plus ΔU_for over de wandoppervlakte z·P (ΔU_for opgeven);
- maandwaarden volgens D.10/D.11.

## Rekenzone met meerdere gebruiksfuncties (§6.5.3)

`functionAreas` geeft de gebruiksfuncties met hun oppervlakte. De volgende rekenwaarden worden dan naar gebruiksoppervlakte gewogen:

- de setpoints (tabel 7.13) en het verlaagde setpoint (tabel 7.14);
- de reductie-uren (tabel 7.15) en daarmee a_C;red;
- de interne warmte van personen en apparatuur (tabellen 7.2/7.3);
- de vaste q_L van de C1-run.

Voorwaarden:

- `usageFunction` moet de grootste functie zijn;
- de oppervlaktes tellen op tot A_g (1 % tolerantie);
- een woonfunctie mag niet met andere functies worden gemengd (§6.5.2).

De toetsen op setpointverschil, ventilatiecapaciteit en warmtecapaciteit staan in `zoning.rs`.

## Aangrenzende onverwarmde serre (7.30b, §7.6.4)

`sunrooms` geeft per serre de gegevens voor de indirecte zonnewinst:

- de absorberende oppervlakken (α, oppervlakte, oriëntatie, helling);
- g en kozijnfractie van de serrebeglazing, apart voor verwarming en koeling;
- b_U en H_zi;ztu uit §8.4, en de verdeelfactor (7.36).

De winst is (1 − b_U)·F·f_gn;max·Q_sol;ztu. Q_sol;ztu volgt uit 7.34/7.35 met F_sh;obst = 1. De begrenzing f_gn;max (7.37) geldt alleen in de verwarmingsstand. Het warmteverlies via de serre loopt via de onverwarmde ruimte van §8.4. De vereenvoudigde route uit de norm (de serre negeren) blijft mogelijk door geen `sunrooms` op te geven.

## H_D naar een onverwarmde ruimte (8.4.2.1)

H_D;zi,j;ztu (p. 266) wordt berekend als H_D van 8.2.1, met R_se vervangen door de R_si van de onverwarmde ruimte (tabel C.2). De U-waarden van het project zijn bepaald met R_se = 0,04 (C.1.2). De projectroute rekent ze daarom om: U_iu = 1/(1/U − 0,04 + R_si). Daarbij is R_si 0,13 bij een wand of een vlak zonder type, 0,10 bij een dak of plafond (warmtestroom omhoog) en 0,17 bij een vloer boven de onverwarmde ruimte. Dit geldt voor opake delen en ramen; ψ- en χ-waarden blijven ongewijzigd. Bij een binnenwand met U 0,4 wordt dat 0,386 W/(m²K).

## Reductiefactor b_U in de projectroute

Een onverwarmde ruimte in het project (`unheatedSpaces`) heeft óf een opgegeven `reductionFactor` met bron, óf `outside` met de verliezen van de ruimte naar buiten. Met `outside` leidt de kern b_U af volgens 8.53–8.59:

- `transmission` is H_D;ue;
- `ventilation` is 8.57 (debiet) of 8.58 (0,5·H_D;ue).

Wordt per zone gerekend, dan telt de kern H_zi,j;ztu van de andere projectzones die aan dezelfde ruimte grenzen automatisch op bij Σ_j, bovenop `otherZonesConductanceWPerK`.

Beide velden tegelijk geeft `unheated_factor_declared_and_derived`. Geen van beide geeft `unheated_reduction_factor_required`.

## Leidingdoorvoeren in de projectroute

`ntaCalculation.verticalPipes` geeft de verticale leidingen van 7.3.3 (H_p, 7.17) voor een project met één rekenzone. Bij meer zones staan ze per zone in `zoneData[].verticalPipes`; een projectlijst geeft dan `vertical_pipes_per_zone_required`, zodat dezelfde leiding niet in elke zone wordt meegeteld. Een ontbrekende lijst (geen projectlijst bij één zone, geen zonelijst bij meer zones) geeft de lacune `vertical_pipes_unknown` met de standaardregel van 7.3.3 in `detail`; `[]` betekent geen doorvoeren.

## Forfaitaire thermische bruggen in de projectroute

Een forfaitaire vloerrand kiest de forfaitaire route voor het hele gebouw. De kern berekent dan ΔU_for volgens 8.3 uit de ondoorschijnende vlakken naar buitenlucht (oppervlak min ramen, U van de constructie) en telt die op bij elke U in H_D, ramen inbegrepen (opmerking 7 bij 8.3). Ook de zoninstraling op ondoorschijnende delen en de ramen gebruiken die U. Staan er dan nog ψ-waarden in `zones[].thermalBridges` of een gedetailleerde vloerrand met ψ_gr, dan geeft de kern `thermal_bridge_methods_mixed`. In de forfaitaire route is H_U;for = 0 en hoort de scheiding met een onverwarmde ruimte in H_D;for met U_iu;equi (C.1.3). Dat leidt de projectroute niet af; een grensvlak met een onverwarmde ruimte geeft dan `forfait_thermal_bridges_unheated_space_unsupported`.

## Plausibiliteitswaarschuwingen in de projectroute

`warnings` in de projectbeoordeling houdt de berekening niet tegen, maar meldt opgegeven waarden die met de norm of andere invoer botsen:

- `declared_hot_water_efficiency_above_one`: een opgegeven brandstofgebruik voor warm tapwater is lager dan Q_W;nd (13.15 voor woningen, tabel 13.1 voor utiliteit). Dat impliceert een opwekkingsrendement boven 1 op bovenwaarde.
- `declared_ventilation_below_required_flow`: alleen bij woningen. Het gaat om een mechanisch systeem zonder warmteterugwinning waarvan de opgegeven H_ve onder ρc·q_V;ODA;req ligt.
  - De ondergrens volgt 11.22 met de laagste f_ctrl·f_sys van tabel 11.5 voor het systeemtype (B 0,57; C en D 0,52), zonder de ondergrens van 11.63.
  - Een vraaggestuurde regeling geeft daardoor geen valse melding.
  - Bij utiliteit is het ontwerpdebiet tijdgemiddeld over de bedrijfstijd. Die weging maakt deze indicatieve toets niet, dus daar geeft de toets geen melding.
  - Infiltratie komt nog bij de ondergrens.
- `bacs_factor_without_capacity_evidence`: een utiliteitsgebouw met f_BACS < 1,05 zonder `bacs`-blok. Volgens §5.5.8 is f_BACS 1,05 tenzij alle verwarmings- en koelsystemen aantoonbaar ten hoogste 290 kW zijn, of de gebouwautomatisering voldoet.
- `utility_open_ceiling_requires_evidence`: tabel 7.10 voetnoot a. Utiliteitsbouw rekent met de kolom "gesloten of verlaagd plafond", tenzij een vrijhangend plafond ten minste 15 % open is.
- `ground_floor_resistance_below_surface_resistance`: `constructionResistanceM2kPerW` is R_si + R_c (8.32, 8.43). Een waarde onder R_si = 0,17 betekent een negatieve R_c.
- `ground_floor_perimeter_implausible`: de blootgestelde omtrek P is groter dan 2·A/w + 2·w met w = 1 m. Een vloer met een gemiddelde breedte van minstens 1 m kan zo'n omtrek niet hebben.
- `detailed_thermal_bridges_none_entered`: de gedetailleerde route (geen forfaitaire vloerrand, 8.2.1) zonder één lineaire thermische brug naar buitenlucht. H_D krijgt dan geen ψ·ℓ. Voer de bruggen in of kies de forfaitaire ΔU_for (8.3).
- `sunroom_values_differ_from_unheated_space`: een serre (`sunrooms`) met dezelfde id als een onverwarmde ruimte van het project, waarvan de opgegeven b_U of H_zi;ztu meer dan 10 % afwijkt van de afgeleide waarde (8.4.1, 8.53–8.59).

BENG 1 (`needIndicatorKwhPerM2Year`, ook in `indicators`) staat alleen ingevuld als de behoefte uit de vaste C1-ventilatierun van §5.4 komt. E_H+C;nd met de opgegeven ventilatie staat in het hoofdstuk 5-blok (`chapter5.heatingAndCoolingNeedKwhPerM2`). `tojuliMeetsBblLimit` geldt alleen voor woonfuncties (Bbl art. 4.149b) en is bij utiliteit leeg.

## Overgangsweerstand onder een vloer boven een kruipruimte of kelder (5 oktober 2026 herzien)

U_f in 8.43 (p. 258) verwijst via 8.2.2.2.1 en C.1.2 (8.6, p. 229) naar tabel C.2 (p. 778). Die geeft R_se = 0,04, "tenzij bij desbetreffende formules anders is aangegeven". Bij 8.43 staat geen afwijking. De vervanging van R_se door R_si in 8.4.2.1 geldt alleen voor H_D;zi,j;ztu naar een onverwarmde ruimte, en een kruipruimte valt onder §8.3. De kern rekende eerder met R_si 0,17 (naar analogie met 7.2 van NEN-EN-ISO 13370). Ze volgt nu de normtekst: U_f = 1/(R_si + R_c + 0,04).

Effect: H_g van een geventileerde kruipruimte 13,40 → 13,59 W/K in de herberekening, ongeveer +0,2 % Q_H;nd voor een tussenwoning.

## Belemmeringssituaties (§17.3.2, tabel 17.3)

`obstruction.method` kiest één situatie per raam. De situaties zijn alternatieven; ze worden niet met elkaar vermenigvuldigd. Wat tabel 17.3 voor een balans "niet beschikbaar" noemt, valt terug op situatie g. Voor koeling is dat tabel 17.5 (1,00), behalve met een overstek evenwijdig aan een verticaal raam (tabel 17.9).

| `method` | Situatie | Warmte | Koude |
|---|---|---|---|
| `minimal` | a | tabel 17.4 | tabel 17.5 (1,00) |
| `parallel_obstruction` (`relativeHeight` = h_b;⊥) | b, alleen verticaal | tabel 17.7 | niet beschikbaar → 1,00 |
| `overhang` (`relativeHeight` = h_o;⊥) | c, alleen verticaal | tabel 17.8 | tabel 17.9 |
| `side_obstruction` (`side`, `relativeWidth` = b_b) | d, alleen verticaal | tabel 17.10 | tabel 17.11 met `coolingHeightCondition` (≥ 2,5 m boven de bovenkant van het raam), anders 1,00 |
| `full` | e | tabel 17.13 | tabel 17.14 met `coolingConditionsMet`, anders 1,00 |
| `other` (`overhangRelativeHeight` optioneel) | g | tabel 17.13 | tabel 17.9 bij een overstek aan een verticaal raam, anders 1,00 |

- De kolommen voor relatieve hoogte zijn < 0,5, 0,5–1,0 en ≥ 1,0. Voor de relatieve breedte zijn het < 1,0 en ≥ 1,0. Bij zijbelemmeringen aan beide zijden telt de kleinste b_b.
- Verticaal betekent: de dichtstbijzijnde hellingskolom van tabel 17.4 is 90°, dus 82,5° ≤ helling < 97,5°. Situaties b–d op een niet-verticaal vlak geven `obstruction_situation_requires_vertical`.
- Het zichtveld onder 15° helling is zuid (17.3.1). Dat geldt nu ook voor tabel 17.4: een vlak van 7,5–15° op het noorden neemt de zuidwaarde van de 15°-kolom.
- Zonnecollectoren en PV (x = P) gebruiken `solar_shading::collector_obstruction_factor`:
  - `minimal`: tabel 17.6, 1,00;
  - `side_obstruction`: tabel 17.12; aan beide zijden met b_b < 1 op een schuin paneel geldt tabel 17.15 (voetnoot c);
  - `full` en `other`: tabel 17.15;
  - `roof_edge`: tabel 17.15 alleen als h_dakrand > 0,5 m én l_dakrand < h_dakrand, anders 1,00;
  - `declared`.

  De koppeling aan de zonneboiler- en PV-routes moet nog worden gemaakt.
- De tabelwaarden van 17.7–17.15 zijn uit de tekstlaag van de gelicentieerde norm gegenereerd en steekproefsgewijs vergeleken met de gerenderde pagina's (733, 727, 750).
