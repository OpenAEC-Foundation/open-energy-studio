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
| Belemmering | §17.3, tabel 17.4/17.5 (p. 708–715) | `minimal`: warmte tabel 17.4 met dichtstbijzijnde hellingskolom 0–180° (bij gelijke afstand de hoogste), koude 1,00; of `declared` |
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
- **Bijlage A (p. 766–771)**: `windows[].dynamic` geeft een dynamisch transparant element. Methode A (`weighted_states`) middelt g en U per maand over de toestanden. Daarvoor geef je per maand het aandeel van `Σ I_sol·Δt` (A.2) en van `Σ Δθ·Δt` (A.1) per toestand op. Methode B (`single_state`) gebruikt één toestand. De maandelijkse g werkt in de zonwinst; het verschil met de nominale U van het raam wordt per maand bij `H_D` opgeteld.

Elk onderdeel heeft een bronverwijzing. Een onvolledige inventaris, een ongeldige waarde, een onbekend veld of een helling buiten 0–180° geeft `invalid` zonder getallen.

## Uitkomst per maand

Per balans: setpoint na nivellering, reductiefactor (`a_H;red` of `a_C;red`), rekentemperatuur, `H_ve` met `b_v`, tijdconstante, `a`, transmissie, ventilatie, warmteoverdracht, winst, `γ`, benutting en behoefte. Daarnaast de maandelijkse `H_g;an;mi`. De transmissiesamenvatting geeft `H_g` stationair, de twaalf maandwaarden en de seizoenswaarden.

## Niet toegepast (altijd meegeleverd als `omittedCorrections`)

- terugwinbare systeemverliezen `Q_H;ls;rbl`/`Q_C;ls;rbl` en de Δη-termen van 7.3–7.5 en 7.7–7.9 (komen uit hoofdstuk 9 en 10);
- `H_p` (verticale leidingen, 7.3.3) en `H_A` (aangrenzende verwarmde ruimten, 8.5);
- belemmeringssituaties b–g van §17.3 worden opgegeven, niet afgeleid;
- voetnoot c van tabel 7.10: de kolomkeuze ligt bij de aanroeper;
- bijlage D voor andere vloeren dan vloer op grond (kruipruimte, kelder);
- één gebruiksfunctie per rekenzone (tabel 7.13–7.15).

## Interpretatiepunten voor de normreview

1. `θ_e;avg;an` in 7.14/7.15 en D.1–D.3 is het ongewogen gemiddelde van tabel 17.1 (10,6725 °C); D.4 gebruikt de vaste 10,67 °C.
2. D.7/D.8 staan in de NTA met exponenten in `d_f;equi/δ`; NEN-EN-ISO 13370 gebruikt daar de breedte of diepte van de randisolatie. De module volgt de NTA zoals gedrukt.
3. Tabel D.1: verticale randisolatie geeft altijd β = 2; horizontale randisolatie alleen vanaf R_c ≥ 2,0 m²K/W.
4. 7.66–7.68: als `dθ_set − dθ_float ≤ 0` geldt `f_low = 1` vóór de regel `dθ_float = 1 → f_low = 0`.
5. `b_v` (7.20) is ongedefinieerd als `θ_set;stc = θ_e`; de module neemt dan 1. Met tabel 17.1 en tabel 7.13 komt dat niet voor.
6. Een maand met `H_tr + H_g;adj + H_ve ≤ 0` (bijvoorbeeld door warme toevoerlucht met negatieve `b_v`) wordt geweigerd; de tijdconstante is dan niet gedefinieerd.
7. TOjuli (§5.7) gebruikt in de balans en in 5.40 de juliwaarde `H_gr;an`, en in de tijdconstante `H_C;g;adj`.
8. Bijlage B: een vrijhangend plafond met ten minste 15 % open oppervlak telt niet mee voor de weerstand vanaf het binnenoppervlak. Omdat het geen bouwconstructie is, telt de module ook de massa ervan niet mee.
9. Bijlage A: de correctiefactoren van stap 2 hebben geen forfaitaire waarde en zijn 1. De kern heeft geen uurklimaat, dus de gewichten van stap 1 worden opgegeven. De nominale U van het raam moet in de transmissie-invoer staan, omdat de module per maand alleen het verschil corrigeert. TOjuli gebruikt per oriëntatie nog de nominale U.

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

De vloerconstructie zelf krijgt aan de onderzijde R_si = 0,17 (C.2). Randisolatie volgens D.7/D.8 hoort alleen bij een vloer direct op de grond. Een verwarmde kelder krijgt `heatedBasement` met de werkelijke diepte z en de R_c van de kelderwanden:

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

## Reductiefactor b_U in de projectroute

Een onverwarmde ruimte in het project (`unheatedSpaces`) heeft óf een opgegeven `reductionFactor` met bron, óf `outside` met de verliezen van de ruimte naar buiten. Met `outside` leidt de kern b_U af volgens 8.53–8.59:

- `transmission` is H_D;ue;
- `ventilation` is 8.57 (debiet) of 8.58 (0,5·H_D;ue).

Wordt per zone gerekend, dan telt de kern H_zi,j;ztu van de andere projectzones die aan dezelfde ruimte grenzen automatisch op bij Σ_j, bovenop `otherZonesConductanceWPerK`.

Beide velden tegelijk geeft `unheated_factor_declared_and_derived`. Geen van beide geeft `unheated_reduction_factor_required`.
