# NTA 8800 maandelijkse warmte- en koudebehoefte (hoofdstuk 7)

Rust-module `monthly_demand` berekent per rekenzone de maandelijkse netto warmtebehoefte `Q_H;nd` en koudebehoefte `Q_C;nd` volgens de maandmethode van NTA 8800:2025+C1:2026 hoofdstuk 7. Dit is de eerste schakel van de rekenruggengraat: gebouw → behoefte → installaties → primaire energie → BENG. Het resultaat heeft status `calculated_unverified`. `referenceVerified`, `finalEditionVerified` en `bengCalculationAvailable` blijven `false`.

## Bron

De formules en constanten komen uit de normanalyses C1–C5 van Open Heatloss Studio (13 juli 2026). Die transcriberen de gelicentieerde pdf `NTA 8800_2025+C1_2026 nl.pdf` met paginaverwijzingen. De code is in Open Energy Studio opnieuw geschreven; Open Heatloss Studio staat onder de MIT-licentie. Een onafhankelijke review tegen de normtekst is nog nodig; de pdf staat op de netwerkshare `Z:\50_projecten\7_3BM_bouwkunde\000_Documentatie\98_normen\` en is niet op deze machine gemount.

| Onderdeel | Norm | Waarde / formule |
|---|---|---|
| Warmtebalans | 7.2.1 | `Q_H;nd = max(0, Q_H;ht − η_H;gn·Q_H;gn)` |
| Koudebalans met poort | 7.6/7.7 | `Q_C;ht/Q_C;gn > 2 → 0`, anders `Q_C;gn − η_C;ht·Q_C;ht` |
| Benutting | 7.46–7.55 | `η_H = (1−γ^a)/(1−γ^(a+1))`, `η_C = (1−γ^−a)/(1−γ^−(a+1))`, grens `a/(a+1)` bij γ = 1 |
| Parameter a | 7.51 | `a = 1,0 + τ/15 h` |
| Tijdconstante | 7.57 | `τ = (C_m/3600)/(H_tr + H_g;adj + H_ve)` |
| Warmtecapaciteit | tabel 7.10, 7.45 | `D_m` 55/80 … 250/450 kJ/(m²K); `C_m = D_m·1000·A_g` |
| Interne winst woning | 7.21–7.24 | `180·N_woon·N_P·0,001·t`; `N_P` per banden ≤30, 30–100, >100 m² |
| Zonwinst ramen | 7.32, 7.40 | `0,90·g_n·A·(1−F_F)·F_sh;obst·I_sol·t·0,001 − Q_sky` |
| Zonwinst opaak | 7.33, 7.6.6.3 | `0,6·R_se·U·A·I_sol·t·0,001 − Q_sky`, `F_sh;obst = 1` |
| Hemelstraling | 7.39, 7.6.6.4 | `F_sky·R_se·U·A·4,14·11·t·0,001`; `F_sky` 1 / 0,75 / 0,5 |
| Klimaat | tabel 17.1/17.2 | De Bilt; alleen hellingen 0° en 90° |
| Setpoints | tabel 7.13 | door de aanroeper met bron (woning 20 / 24 °C) |

## Invoer

De transmissie kent twee routes (`transmission.method`):

- `explicit`: `H_tr` exclusief grond (W/K) met bron, plus optioneel een grondroute (`H_g;adj` en twaalf maandwaarden voor verwarming en koeling);
- `components`: de module stelt `H_tr` zelf samen uit directe buitenelementen en koudebruggen (8.1, via `direct_transmission`) en onverwarmde ruimtes met opgegeven `b` (via `unheated_transmission`). Vloeren op grond gaan via het P/A-model van §8.3 (module `ground`: 8.30 `B' = A/(0,5·P)`, 8.32 `d = 0,5 + λ(R_si+R_c+0,04)`, λ = 2,0 (8.35), 8.40/8.41, 8.36 `H_g = A·U_fl`). Het grondverlies per maand is `H_g·(θ_int − θ_e;avg;an)·t` (7.14). Het jaargemiddelde is het ongewogen gemiddelde van tabel 17.1 (10,67 °C). De uitkomst geeft `transmission` met de deelcoëfficiënten.

Verder levert de aanroeper expliciet: `A_g` met bron, setpoints, ventilatiestromen met twaalf maandelijkse `H_ve` en optionele toevoertemperatuur, de massaklassen van vloer en wand plus de kolomkeuze van het plafond, de methode voor interne winst en de volledige lijsten met ramen en opake buitenvlakken. Elk onderdeel heeft een bronverwijzing. Een onvolledige inventaris, een ongeldige waarde, een onbekend veld of een niet ondersteunde helling geeft `invalid` zonder getallen.

## Niet toegepast (altijd meegeleverd als `omittedCorrections`)

- niet-continu verwarmen `a_H;red` (§7.9.2);
- temperatuurnivellering woningbouw (§7.9.4.2, formule 7.78);
- beweegbare zonwering en een afzonderlijke `g_gl;C`;
- hellingen anders dan 0° en 90°;
- detailberekening van de warmtecapaciteit volgens bijlage B;
- voetnoot c van tabel 7.10: de kolomkeuze ligt bij de aanroeper.

Een koelmaand waarin de warmteoverdracht voor koeling ≤ 0 is, wordt geweigerd (`cooling_heat_transfer_nonpositive_unsupported`) omdat die route niet is getranscribeerd. Met De Bilt en 24 °C komt dat niet voor.

## Interpretatiepunten voor de normreview

1. `τ` wordt per maand bepaald met de `H_ve` van die maand.
2. `H_tr` bevat de onverwarmde-ruimteroute met toegepaste b-factor; de exacte samenstelling uit §8 moet de aanroeper leveren.
3. Grond: alleen vloer op staal (z = 0); kruipruimte, kelder en randisolatie vallen buiten de module. Er is geen periodieke term (bijlage D). `H_g;an` wordt ook als `H_g;adj` in `τ` gebruikt. `θ_e;avg;an` is het ongewogen maandgemiddelde.

## Toetsing

Er zijn twaalf unittests: tabel 7.10, de bewonersbanden, de limieten van de benutting, een handmatig nagerekende januari, de koelpoort, het effect van massa en toevoertemperatuur, de grondroute en de foutpaden. Twee servicetests dekken HTTP. Het synthetische geval staat in `training-data/nta8800-monthly-demand-synthetic.json`. Er is nog geen onafhankelijk referentiegeval. De EDR/ISSO 54-invoer (EPW001) is bekend, maar de verwachte uitkomsten (bijlage 2, Excel) ontbreken.

## Aanroep

HTTP: `POST /v1/nta8800/demand/monthly/calculate` met `{ "input": { ... } }`. MCP en desktop: `calculate_monthly_demand`. TS: `calculateMonthlyDemandWithRust`.
