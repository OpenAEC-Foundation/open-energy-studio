# Vergelijking met openbare BENG-rapporten — 5 oktober 2026

Dit is **geen officiële referentietoets**. Vier openbaar gepubliceerde BENG-rapporten zijn in de app nagebouwd en met de rekenkern doorgerekend. Die rapporten zijn gemaakt met geattesteerde software (Uniec) onder oudere NTA 8800-edities. Verschillen zijn daarom deels verwacht. De officiële toetsing voor BRL 9501 loopt via ISSO-publicatie 54 versie 5.0:2026, die niet openbaar is.

De herbouwde invoer staat als fictieve, geanonimiseerde fixtures in `training-data/nta8800-public-comparison-{a,b,c,d,f}.json`. Adressen en namen zijn verwijderd. De test `crates/nta8800-core/tests/public_comparison.rs` legt de huidige uitkomsten vast als regressie; de gepubliceerde waarden zijn alleen context.

## Bronnen

| Geval | Bron | Software, rekendatum en NTA-editie |
|---|---|---|
| A | [openbaar rapport A (vrijstaande woning, plat dak)](https://www.starlinehome.nl/uploads/20210923-Beng_berekening_21.164_002.pdf) | Uniec 3.0.16, 13-04-2021, NTA 8800:2020+A1 |
| B | [openbaar rapport B (rijwoning)](https://repository.officiele-overheidspublicaties.nl/Bijlagen/TerInzageLegging/2024/til-2024-415/1/bijlage/26.2220837_-_Rap._BENG-berekening.pdf) | Uniec 3.1.6.2, 22-03-2023, NTA 8800:2022 |
| C | [openbaar rapport C (vrijstaande woning, twee zones)](https://www.oud-osdorp.nl/wp-content/uploads/2024/03/Rap.-BENG-berekening-V1.0_18-03-2024.pdf) | Uniec 3.2.7.0, 18-03-2024, NTA 8800:2023 |
| D | [openbaar rapport D (vrijstaande vakantiewoning met kap)](https://www.planviewer.nl/imro/files/NL.IMRO.1640.OV22HoHouterhof8-VG01/b_NL.IMRO.1640.OV22HoHouterhof8-VG01_bd1.pdf) | Uniec 3.0.10.0, 30-03-2021, NTA 8800:2020+A1 |
| F | [openbaar rapport F (vrijstaande woning met kap en plat dak)](https://www.planviewer.nl/imro/files/NL.IMRO.1640.OV22KoKelperveen24-VG01/b_NL.IMRO.1640.OV22KoKelperveen24-VG01_bd10.pdf) | Uniec 3.0.19.4, 08-03-2022, NTA 8800:2020+A1 (zie [F in 2020+A1](#f-in-2020a1-7-oktober-2026)) |

De rapporten noemen de editie niet zelf. De editie volgt uit de rekendatum en de aanwijzingsperiode (`NormVersion::designation_period`): 2020+A1 tot 1 juni 2022, 2022 tot 1 juli 2023, 2023 tot 1 juli 2024. De Uniec-versie past daarbij (3.0 voor 2020+A1, 3.1 voor 2022, 3.2 voor 2023).

Andere gevonden rapporten zijn niet nagebouwd (zie [Gezochte rapporten](#gezochte-rapporten-7-oktober-2026)).

## Uitkomsten

BENG 1 en 2 in kWh/(m²·jr), BENG 3 in %.

| Geval | Gepubliceerd | Kern, eerste herbouw (forfaitaire WP) | Kern met kwaliteitsverklaring en 6.2b | Kern na regel-voor-regel-reconciliatie |
|---|---|---|---|---|
| A | 92,99 / 25,19 / 80,4 | 94,00 / 75,59 / 53,8 | 94,00 / 33,84 / 76,0 | 94,00 / 35,15 / 74,9 |
| B | 54,61 / 27,10 / 64,8 | 52,96 / 38,04 / 51,5 | 52,96 / 24,10 / 67,3 | 52,96 / 28,84 / 62,6 |
| C, twee zones | 64,47 / 28,70 / 70,9 | BENG 1 niet berekenbaar / 31,44 / 69,1 | 64,70 / 31,44 / 69,1 | 64,70 / 31,54 / 69,0 |

De laatste kolom is wat `tests/public_comparison.rs` vastlegt. Daarin zijn de invoercorrecties uit de reconciliatie verwerkt (zie hieronder) en rekent een gedeclareerd warmtepomprendement met f_prac 0,95. A_g, A_ls en de PV-opbrengst komen in alle drie gevallen exact overeen. TOjuli is overal 0.

## Wat er is aangepast

1. **Kwaliteitsverklaring warmtepomp (§9.1, p. 285; §13.8.4.7.2, p. 640).** De rapporten A en B gebruikten productwaarden uit een BCRG-verklaring: COP 4,35 / 1,30 (A) en 4,05 / 1,55 (B). De kern kon die niet opnemen in de projectberekening, waardoor de forfaitaire tabel 9.27 gold.
   - Nu kan bij de forfaitaire warmtepomp een `qualityDeclaration` worden ingevuld, met:
     - het nummer van de verklaring;
     - de COP;
     - de energiefractie;
     - de jaarlijkse hulpenergie.
   - Bij tapwater kan een gedeclareerd rendement worden ingevuld.
   - Beide hebben een eigen sectie in het NTA-formulier.
   - De COP wordt naar beneden afgerond op 0,05, zoals §9.1 voorschrijft.
2. **Eén woning in meerdere rekenzones (6.2b, p. 160; 7.21–7.24, p. 177).** Elke zone telde eerst een hele woning, zodat de bewoners dubbel meetelden. De W/m²-omweg blokkeerde BENG 1.
   - Nu krijgt elke zone N_woon;zi = A_g;zi / Σ A_g;zi × N_woon.
   - BENG 1 is weer berekenbaar: 64,70, tegen 64,47 gepubliceerd.
3. **Koudeafgifteverlies (10.15).** Er is een zachtere melding bijgekomen, `cooling_emission_loss_exceeds_need`, voor maanden waarin het verlies groter is dan de behoefte. Dat speelt in geval A, met een factor 1,56. De harde melding boven 3× blijft.
4. **Hulpenergie koeling (geval C, circa 97 kWh tegen 10 kWh).** Dit is nagelopen en is geen fout. Bijna alles is de regelenergie van formule 10.87 (p. 425): 0,010 kW die altijd in bedrijf is, samen 87,6 kWh per jaar. De pompenergie zelf is klein. Het staat nu in de interpretatielijst.

## Regel-voor-regel-reconciliatie

Per post is het elektriciteitsgebruik uit het rapport naast de kern gelegd. Tussen haakjes staat de geleverde warmte of koude. Het effect op BENG 2 is Δ × 1,45 / A_g. De optelsom van de posten verklaart het verschil in BENG 2 tot op ±0,03 kWh/(m²·jr), de afronding van de hele kWh in de rapporten.

Elektriciteit in kWh/jr, rapport → kern, vóór de correcties in deze versie:

| Post | A | B | C |
|---|---|---|---|
| Warmtepomp verwarming (warmte) | 1 269 → 1 243 (5 242 → 5 407) | 1 186 → 1 057 (4 564 → 4 282) | 4 541 → 4 574 (15 212 → 15 323) |
| Hulpenergie verwarming | 53 → 52 | 171 → 135 | 329 → 331 |
| Tapwater (warmte) | 1 673 → 1 590 (2 067 = 2 067) | 2 136 → 2 029 (3 145 = 3 145) | 3 359 → 3 598 (4 703 → 5 037) |
| Ventilatoren incl. vorstbeveiliging | 12 → 91 | 360 → 129 | 354 → 354 |
| Koeling (opgewekte koude) | 745 → 1 174 (2 236 → 3 522) | 312 → 446 (936 → 1 339) | 539 → 663 (1 618 → 1 989) |
| Hulpenergie koeling | 10 → 98 | 10 → 98 | 10 → 97 |
| PV | 2 437 → 2 467 | 1 646 = 1 646 | 4 065 = 4 065 |

Bijdrage per oorzaak aan het verschil kern − rapport, in BENG 2 kWh/(m²·jr) / BENG 3 %-punt. De bijdragen zijn stap voor stap bepaald; de verdeling over BENG 3 hangt licht af van de volgorde.

| Oorzaak | Soort | A | B | C | In deze versie |
|---|---|---|---|---|---|
| Lezing van de rapportinvoer (ventilatoren, leidingen, hulpenergie) | invoer | +1,52 / −0,8 | −3,00 / +2,7 | −0,10 / +0,1 | gecorrigeerd in de fixtures |
| f_prac 0,95 bij een gedeclareerd WP-rendement | modellering | −2,82 / +1,8 | −1,73 / +2,0 | – | kern aangepast (zie onder) |
| Regelenergie koeling 10.87 (87,6 kWh/jr) | interpretatie | +1,66 / −0,9 | +0,94 / −0,8 | +0,50 / −0,3 | blijft, vraag aan NEN |
| Koudeafgifteverlies 10.15 | interpretatie | +8,55 / −5,0 | +1,83 / −1,6 | +1,46 / −1,0 | blijft, vraag aan NEN |
| PV: afronding K_pk (editie 2020, formule 16.4) | versie | −0,57 / +0,4 | – | – | versieverschil |
| Tapwater: kolom binnendiameter in tabel 13.2 (editie 2023) | versie | – | – | +1,35 / −0,9 | verdwijnt in de editie 2023 (zie onder) |
| Rest (warmte- en koudebehoefte) | onverklaard | +0,30 / 0,0 | −1,04 / +0,2 | −0,47 / +0,4 | open |
| **Totaal** | | **+8,65 / −4,4** | **−3,00 / +2,5** | **+2,74 / −1,8** | |

Na de twee correcties (invoer en f_prac) blijft over, in BENG 2 kWh/(m²·jr): A +9,96 (35,15 tegen 25,19), B +1,74 (28,84 tegen 27,10), C +2,84 (31,54 tegen 28,70). Dat is precies de som van 10.87, 10.15, de versieverschillen en de rest.

### Toelichting per oorzaak

1. **Invoer.**
   - A: het rapport geeft de ventilator als productwaarde, P 8,5 W en f 0,147. Met 11.132 (p. 512, f_prac;vent 0,9) geeft dat 12,2 kWh, gelijk aan het rapport. De eerste herbouw rekende forfaitair (91 kWh).
   - B: vier units van 21,7 W met f 0,364 geven 307,5 kWh, plus 52,6 kWh vorstbeveiliging: 360,1 kWh, gelijk aan het rapport. Ook de gedeclareerde hulpenergie van de warmtepomp van 146 kWh ontbrak.
   - Alle drie: volgens de rapporten liggen er geen leidingen buiten de verwarmde of gekoelde zone. De herbouw nam de standaard van 15 % aan.
   - Gevoeligheid: de thermische massa is bepalend voor BENG 1. A als zwaar geeft 76,0 in plaats van 94,0; B als licht geeft 63,9 in plaats van 53,0. Een kozijnfractie van 0,15 geeft A 98,8 en B 53,2.
2. **f_prac bij een kwaliteitsverklaring.** In A en B is de elektriciteit van het rapport tot op de kWh gelijk aan warmte / (COP × 0,95). Formule 9.62 (tabelroute, p. 337) heeft f_prac 1; 9.63 (gegevens volgens bijlage Q, p. 340) heeft 0,95. Een kwaliteitsverklaring voor verwarming is opgesteld volgens bijlage Q (p. 615), dus de kern rekent een gedeclareerde COP nu met 0,95. Voor tapwater geeft 13.152 (p. 616–617) f_prac 1,0 alleen aan de forfaitaire waarden van 13.8.4.5–13.8.4.7; een gedeclareerde waarde valt onder "alle overige gevallen", 0,95. Beide stonden in de vorige versie op 1,0.
3. **10.87.** De tekst is in 2020 (p. 403) en 2025 (p. 425) gelijk: 0,010 kW regelenergie die altijd in bedrijf is. De rapporten tellen alleen pompenergie (circa 10 kWh).
4. **10.15.** De formule is sinds 2020 ongewijzigd (2020 p. 353, 2025 p. 376). De tabellen voor Δθ zijn in 2024 vervangen (−4,0/−4,0/−3,3 K oud, −3,55/−3,55/−3,05 K nieuw), wat de letterlijke uitkomst minder dan 0,5 % verandert. Letterlijk toegepast is het verlies 80–83 % van de behoefte, en meer dan 100 % in mei en september, omdat θ_int,inc dicht bij θ_e,comb ligt. Bij onze behoefte impliceren de rapporten 16 % (A), 21 % (B) en 42 % (C). 10.15 neemt het maximum van de verhouding en 0,15 en geldt alleen als θ_int,inc − θ_e,comb negatief is. Rekenen met de verhouding begrensd op 0,15 reproduceert A en B binnen 40–50 kWh koude; C houdt 309 kWh over. Welke regel de andere software toepast, is uit de rapporten niet vast te stellen.
5. **Versieverschillen.**
   - PV (A): de editie 2020 rondt K_pk in 16.4 (p. 651) af naar beneden op een veelvoud van 5 W/m²; 2025 voegt 16.4b toe (p. 679, Wp per paneel). Met een aangenomen paneel van 1,646 m² en 195 W/m² geeft dat factor 0,98784, en 2 437 tegen 2 467 kWh.
   - Tapwater (C): de editie 2023 (p. 533) had η_k = 0,55 voor een binnendiameter tot 10 mm; 2025 (p. 543) geeft alleen 0,43. Met de oude waarde wordt de tapwaterwarmte 4 703,1 kWh, gelijk aan het rapport.
   - CO₂ (geen BENG): 0,34 in tabel 5.2 van 2023 tegen 0,268 in 2025. Met 0,34 komen de 451, 860 en 1 723 kg uit de rapporten exact terug.
6. **Rest.** Netto warmtebehoefte rapport → kern: A 59,10 → 61,39 (+3,9 %), B 29,48 → 27,79 (−5,7 %), C 51,62 → 52,33. Dit is bouwfysica en invoerdetail (belemmering, thermische massa); in C ook het resterende verschil in koudebehoefte.

### B en C in de editie 2023

C is berekend volgens NTA 8800:2023; B valt in de periode van 2022 (zie [A, B en D in hun eigen editie](#a-b-en-d-in-hun-eigen-editie-7-oktober-2026)). 2022 verschilt voor B niet van 2023. De kern rekent 2023 nu ook (`normVersion: "2023"`, zie [nta8800-normversies.md](nta8800-normversies.md)). Uitkomsten BENG 1 / BENG 2 / BENG 3:

| Geval | Rapport | Kern 2024 | Kern 2023 |
|---|---|---|---|
| B | 54,61 / 27,10 / 64,8 | 52,96 / 28,84 / 62,6 | 52,96 / 29,10 / 62,5 |
| C (keukenleiding ≤ 10 mm) | 64,47 / 28,70 / 70,9 | 64,70 / 31,54 / 69,0 | 64,70 / 30,44 / 69,8 |

- C: met `kitchenPipeDiameter: "up_to_10_mm"` geeft tabel 13.2 (2023 p. 533) η_W;em;k 0,55 in plaats van 0,43. BENG 2 daalt precies 1,35 en BENG 3 stijgt 0,9: de versiepost van de tabel hierboven verdwijnt. Wat overblijft (+1,49 BENG 2) is 10.87 (+0,50), 10.15 (+1,46) en de rest (−0,47).
- B: de schakelpunten van 2023 verhogen BENG 2 met 0,26; BENG 1 blijft gelijk. Opgebouwd uit:
  - **+0,11: ΔT_C;fan** (11.3.2.7). Voor de koudebehoefte van woningen geldt 0,7 K in 2023 (p. 496) tegen 0,4 K in 2024 (p. 491). B heeft een WTW-unit met volledige bypass, dus de uitzondering "WTW zonder bypass: 0 K" geldt niet. De warmere toevoerlucht verhoogt de koudebehoefte van de werkelijke ventilatie en daarmee de koeling in BENG 2. BENG 1 rekent met het vaste systeem C1 zonder ventilatoren in de toevoer en verandert niet. Nagegaan door elk schakelpunt afzonderlijk op de 2024-waarde te zetten: alleen de koelwaarde van ΔT_fan geeft het verschil (de verwarmingswaarde is voor B 0 K in beide uitgaven, omdat het WTW-rendement de dissipatie bevat). De waarden zijn juist overgenomen.
  - **+0,13: afgifte verwarming** volgens tabel 9.4 van 2023 (vloerverwarming, onbekende isolatie → (9.18a), Δθctr,1 2,5, eenpijpskolom 0,7, Δθroomaut −0,5, Δθim −0,2: 3,2 K tegen 2,5 K in 2024).
  - **+0,02: afgifte koeling** volgens tabellen 10.2–10.5 (vloerkoeling: −4,0 K tegen −3,55 K).
- C: −1,35 door tabel 13.2 (≤ 10 mm), +0,24 afgifte verwarming en +0,02 afgifte koeling; ΔT_C;fan heeft voor C geen effect. Uitkomst 30,44 (was 30,19 vóór de afgifteschakelpunten).
- B heeft geen gegevens over de binnendiameter, dus tabel 13.2 blijft op "overig". Het verschil met het rapport blijft verklaard door 10.87, 10.15 en de rest.
- De afgifteroutes van 2023 (tabellen 9.2–9.10 en 10.2–10.5) zijn nu omgeschakeld. Omdat de rapporten de 2023-afgiftegegevens niet tonen, rekent de kern met de onbekende waarden van 2023; dat verhoogt BENG 2 en vergroot het verschil met de rapporten iets. Met de werkelijke afgiftegegevens (`emission.edition2023`) kan dat dalen.

De test staat in `crates/nta8800-core/tests/public_comparison.rs` (`cases_b_and_c_under_nta_8800_2023`).

De vragen over 10.15 en 10.87 staan in [nta8800-vragen-nen.md](nta8800-vragen-nen.md).

## A, B en D in hun eigen editie (7 oktober 2026)

Sinds de kern NTA 8800:2022 en 2020+A1 rekent, lopen A en D in 2020+A1 en B in 2022. De test is `cases_a_and_b_under_their_own_editions` en `case_d_under_nta_8800_2020_a1` in `crates/nta8800-core/tests/public_comparison.rs`. BENG 1 / BENG 2 / BENG 3:

| Geval | Rapport | Kern 2024 | Kern in de eigen editie |
|---|---|---|---|
| A (2020+A1) | 92,99 / 25,19 / 80,4 | 94,00 / 35,15 / 74,9 | 94,00 / 36,06 / 74,4 |
| B (2022) | 54,61 / 27,10 / 64,8 | 52,96 / 28,84 / 62,6 | 52,96 / 29,10 / 62,5 |
| D (2020+A1) | 86,72 / 39,19 / 83,5 | – | 86,35 / 39,07 / 83,6 |
| E (2022) | 86,82 / 28,89 / 74,1 | – | 88,61 / 31,68 / 72,6 |

**Invoer die de oudere edities anders vragen.**
- (8.47): 2022 en 2020+A1 rekenen met de werkelijke hoogte h van de vloer boven maaiveld (2022 p. 236). A en B geven 0,10 m. Dat verandert BENG 1 niet op twee decimalen.
- 2020+A1 kent geen werkelijke leidinglengte bij woningen (punt 77). De 48,84 m van A is de forfaitaire lengte die Uniec bij "leidinggegevens onbekend" toont; 9.36 geeft dezelfde lengte (BENG 2 35,15 → 35,14 in 2024).
- 2020+A1 rondt K_pk af naar beneden op 5 W/m² (punt 68). A: 325 Wp op 9 panelen van samen 15,21 m² is 192,3 → 190 W/m²; de kern geeft dan de 2 437 kWh van het rapport. In 2024 telde de kern 2 467 kWh.

**A in 2020+A1.** BENG 2 stijgt van 35,15 naar 36,06: +0,56 door de PV-afronding (de versiepost van de tabel hierboven verdwijnt) en +0,36 door de schakelpunten van 2023 (afgifte verwarming en koeling, ΔT_C;fan), die ook in 2022 en 2020+A1 gelden. De overige punten van 2020+A1 raken A niet: de warmtepomp heeft een gedeclareerde hulpenergie, dus punt 74 speelt niet. Wat tegen het rapport overblijft, is 10.15 (+8,55), 10.87 (+1,66) en de rest.

**B in 2022.** Gelijk aan 2023 (29,10): geen van de punten 49–67 raakt B. De luchtwarmtepomp levert 45 °C, onder de grens van 55 °C van punt 49.

**D in 2020+A1.** Een vrijstaande vakantiewoning ("andere logiesfunctie", woningbouw volgens tabel 6.1) van 79,70 m² met één rekenzone. Het rapport is een volledige Uniec-uitdraai. Herbouw:
- Gevels W en O 47,33 m² bruto, N en Z 23,75 m² (Rc 4,70); twee dakvlakken van 47,48 m² onder 30° (Rc 6,30); vloer op grond 82,23 m² (Rc 3,70), omtrek 40,09 m (de funderingslengtes). Ramen U 1,4 / g 0,60, deur U 1,6. Lineaire bruggen per vlak zoals opgegeven.
- Massa: "dragend metselwerk met massieve betonnen vloeren", tabel 7.10 rij > 750 kg/m² (`massKgPerM2` 800).
- Infiltratie forfaitair: de kern geeft 1,4 × 0,7 × 1,0 = 0,98 dm³/(s·m²), gelijk aan q_v10;lea;ref van het rapport.
- Grondwaterwarmtepomp "voldoet aan tabel 9.28", 35 °C, COP 5,00: de rij van tabel 9.28 met c_bron 1. Het rapport geeft geen meetwaarden; de fixture zet ze net boven het minimum van tabel 9.28.
- Vloerverwarming in een vertrek van 4–6 m (afgifteroute voor hoge ruimten, 5,0 m), regeling in het hoofdvertrek.
- Ventilatie C.2c met forfaitaire ventilatoren, geïnstalleerde capaciteit 117 dm³/s.
- Elektrische boiler van 200 l, label C, met geïsoleerde warme aansluiting: f_sto;dis;ls 1,5 (2020 p. 545, punt 64).
- 17 PV-panelen van 350 Wp, west, 30°. Het paneeloppervlak staat niet in het rapport; de fixture neemt 1,75 m² (200 W/m²), zodat de afronding van punt 68 niets afhaalt.

Per post, elektriciteit in kWh/jr (het rapport geeft primaire energie; gedeeld door 1,45):

| Post | Rapport | Kern | Verschil in BENG 2 |
|---|---|---|---|
| Warmtepomp verwarming (warmte) | 2 427 (12 136) | 2 421 (12 103) | −0,12 |
| Hulpenergie verwarming | 46 | 46 | 0,00 |
| Tapwater | 3 552 | 3 552 | 0 |
| Ventilatoren | 401 | 401 | 0 |
| PV (op de meter) | 4 273 | 4 273 | 0 |
| **BENG 2** | **39,19** | **39,07** | **−0,12** |

- **Hulpenergie.** Het rapport rekent 9.85 met A = 13,0 kWh (toestel vanaf 2015): 13,0 + 0,132 × 2 427 / (0,4 × 24) = 46,4. De fixture geeft de warmtepomp het installatiejaar 2021 (`installationYear`); de kern neemt dan ook A = 13,0 en rekent 46 kWh op zijn eigen 2 421 kWh. Zonder installatiejaar zou de kern A = 87,6 nemen (vóór 2015 of onbekend, punt 74 in [nta8800-normversies.md](nta8800-normversies.md)): 119 kWh en BENG 2 38,42. Wat tegen het rapport overblijft, is 6 kWh warmtepompelektriciteit (hieronder).
- **Ventilatoren.** Tabel 11.23 geeft voor ventilatoren vanaf 2007 0,45 W/(m³/h). Met dat fabricagejaar (nieuwbouw 2021) is de uitkomst gelijk aan het rapport. Een eerste herbouw met een onbekend fabricagejaar nam de oudste rij (4,00 W/(m³/h) voor wisselstroom) en gaf 3 567 kWh; dat was een invoerfout in de herbouw, geen fout in de kern.
- **Warmte en BENG 1.** Het rapport geeft het glas op de oostgevel (9,84 m²: vier ramen en de glazen deur) een constante overstek, maar niet de maat. Sinds de kern een belemmering per raam kent (`windowObstructions`), staat die overstek alleen op die ramen. De klassen van tabel 17.8/17.9 geven (BENG 1 / BENG 2 / TOjuli):

  | h_o;⊥ | BENG 1 | BENG 2 | TOjuli |
  |---|---|---|---|
  | minimale belemmering (vóór 7 oktober) | 82,49 | 37,06 | 0,67 |
  | ≤ 0,4 | 86,35 | 39,07 | 0,27 |
  | 0,5–0,75 | 85,56 | 38,65 | 0,43 |
  | ≥ 1,0 | 84,78 | 38,01 | 0,67 |

  Alleen de klasse h_o;⊥ ≤ 0,4 geeft de TOjuli van het rapport (0,29) en vrijwel zijn BENG 1. De fixture neemt h_o;⊥ 0,25 en vermeldt dat als aanname. BENG 1 ligt dan 0,4 % onder het rapport, BENG 2 0,12.
- **Tabel 9.28.** De kern eiste voor de rij van tabel 9.28 een beproeving volgens NEN-EN 14511-2:2022. Dat is de gedateerde verwijzing van 2023 (p. 322). 2022 (p. 14, 316) en 2020+A1 (p. 15, 314) verwijzen naar NEN-EN 14511-2:2007. De kern volgt nu de verwijzing van de editie (`heat_pump_high_test_standard`).

D in 2022 (alleen ter vergelijking; D valt in de periode van 2020+A1): 86,35 / 41,79 / 82,6. Het verschil met 2020+A1 is de hulpenergie van de warmtepomp: 2022 rekent een warmtepomp met de eigen constanten van 9.85 (A 43,8, punt 74), 2020+A1 met A 13,0 voor een toestel vanaf 2015.

**E in 2022.** Een vrijstaande woning van 209,40 m² met één rekenzone (Uniec 3.1.5.0, 31-01-2023, [rapport](https://www.boekel.nl/data/downloadables/2/2/1/8/7564003_1675349122509_20230131-bb_beng_mpg-22-323.pdf)). Gepubliceerd 86,82 / 28,89 / 74,1 en TOjuli 0 (actieve koeling). De kern: 88,61 / 31,68 / 72,6. De test is `case_e_under_nta_8800_2022`. Herbouw:
- Vier gevels met elk hun constructies (spouwmuur Rc 5,99, betimmering 5,19, zijwang dakkapel 4,70), vier dakvlakken onder 50° (Rc 6,30) met dakramen, vloer op grond 139,08 m² (Rc 3,98), omtrek 65,53 m. De bruto gevelvlakken sluiten op de ramen (58,52 / 50,75 / 51,73 / 60,29 m²).
- Geen lineaire bruggen in de uitdraai: de forfaitaire toeslag ΔU_for van 8.2/8.3. Zonder die toeslag gaf de herbouw 72,67 voor BENG 1.
- Belemmering per raam (`windowObstructions`): zijbelemmering op D en L (voorgevel) en op I en H (noordgevel), volledige belemmering op F (13,19 m² oost) en G. De uitdraai geeft per zijbelemmering de hoogte, de afstand en de breedte, in een blok dat bij tekstextractie wegvalt maar in de opgemaakte pagina staat. Met b_b = afstand / breedte (2022 p. 673): D 3,31 / 1,10 = 3,01, L 2,33 / 1,75 = 1,33, I 4,61 / 12,85 = 0,36 en H 1,49 / 12,85 = 0,12, alle lager dan 2,5 m. Een eerdere versie van de fixture nam b_b 0,5 voor alle vier (88,77 / 31,73 / 72,6). Voor de volledige belemmering neemt de fixture aan dat de koelvoorwaarden niet vaststaan (tabel 17.5); met de voorwaarden vervuld daalt BENG 1 met 1,10 en BENG 2 met 0,83.
- Lucht/waterwarmtepomp 9 kW met productspecifiek COP 4,15, energiefractie 0,992 en 281 kWh hulpenergie; aanvullende distributiepomp 164 W (EEI 0,23); 20,10 m geïsoleerde leiding buiten de verwarmde zone.
- Twee tapwatersystemen, verdeeld met 13.19a: de warmtepomp (productspecifiek 1,60) op de badruimte, een kokendwaterkraan met een vat van 7 l (H 0,12 W/K) op het aanrecht. Het rapport geeft 2 962 en 625 kWh behoefte.
- Ventilatie D.2 met wtw 0,893, volledige bypass met koudeterugwinning, ventilatoren 170,8 W met f 0,364; infiltratie gemeten 0,63.
- Compressiekoeling forfaitair met vloerkoeling en 134,02 m leiding.
- 10 panelen van 405 Wp, zuid, 50°; paneeloppervlak aangenomen (200 W/m²).

Per post, elektriciteit in kWh/jr:

| Post | Rapport | Kern |
|---|---|---|
| Verwarming | 3 915 | 4 010 |
| Hulpenergie verwarming | 341 | 342 |
| Tapwater | 2 574 | 2 608 |
| Koeling | 186 | 372 |
| Hulpenergie koeling | 8 | 95 |
| Ventilatoren | 690 | 690 |
| PV (op de meter) | 3 543 | 3 543 |

- Ventilatoren, PV en de hulpenergie van de verwarming zijn gelijk aan het rapport.
- Koeling: de letterlijke lezing van 10.15 (zie de gevallen A–C) verdubbelt hier de koude-elektriciteit. Ook de hulpenergie van de koeling ligt hoger.
- Verwarming en BENG 1 (+2,1 %): de aanname voor de volledige belemmering en de forfaitaire toeslag, waarvan het rapport de uitkomst niet toont.

### Gezochte rapporten (7 oktober 2026)

Gezocht is op openbare BENG-rapporten bij vergunningen (repository.officiele-overheidspublicaties.nl, planviewer.nl, gemeentesites) met rekendata in de periodes van 2020+A1 en 2022. Gevonden en niet nagebouwd (het rapport van Boekel is sindsdien geval E, dat van Kelperveen geval F):

| Rapport | Periode | Waarom niet |
|---|---|---|
| [vrijstaande woning 291 m², Vabi EPA, 1-8-2022](https://repository.officiele-overheidspublicaties.nl/externebijlagen/exb-2022-66238/1/bijlage/exb-2022-66238.pdf) | 2022 (rapportdatum) | ander programma en geen rekendatum in de uitdraai; editie niet vast te stellen |
| [woongebouw met 28 appartementen, Uniec 3.1.3.1, 12-07-2022](https://www.schagen.nl/sites/default/files/2022-12/05%20Beng%20berekening.pdf) | 2022 | meerdere zones en woningen; herbouw uit de uitdraai vraagt veel aannames |
| [woning, Uniec 3.2.4.1, 24-11-2023](https://www.boekel.nl/data/downloadables/5/7/7/3/8290459_1703078780501_2248-og10-beng-berekening-geanonimiseerd.pdf), [appartementen, Uniec 3.2.3.0, 9-11-2023](https://repository.officiele-overheidspublicaties.nl/Bijlagen/TerInzageLegging/2024/til-2024-31972/1/bijlage/06._2022288.beng.wd.b0.pdf), [woning, 23-2-2024](https://repository.officiele-overheidspublicaties.nl/Bijlagen/TerInzageLegging/2024/til-2024-13030/1/bijlage/23-73_BENG-berekening_28.pdf) | 2023 | niet in een van de gezochte edities |

## Conclusie

Geen van de verschillen wijst op een formulefout in de rekenformules van de kern. Eén editieregel is rechtgezet: de beproevingsnorm bij tabel 9.28 in 2022 en 2020+A1. Na correctie van de invoer en f_prac zijn de resterende verschillen herleid tot twee bewuste letterlijke lezingen van de norm (10.15 en 10.87), drie versieverschillen tussen de edities en een klein bouwfysisch restant. De gebruikte standaardwaarden per geval staan als bronvermelding (`sourceReference`) in de fixtures.

## F in 2020+A1 (7 oktober 2026)

Een vrijstaande woning van 231,61 m² met twee bouwlagen, een kap met dakkapellen en een plat dak, berekend op 08-03-2022 met Uniec 3.0.19.4. De rekendatum valt in de periode van 2020+A1. De fixture is `training-data/nta8800-public-comparison-f.json`, de test `case_f_under_nta_8800_2020_a1`.

| | BENG 1 | BENG 2 | BENG 3 |
|---|---|---|---|
| Rapport | 74,69 | 2,59 | 97,5 |
| Kern in 2020+A1 | 74,06 | 3,79 | 96,3 |
| Kern zonder belemmering per raam (eerste herbouw) | 73,49 | 3,68 | 96,4 |

**Herbouw.**
- Gevels NO en ZW 65,92 m² bruto, ZO en NW 47,17 m² (Rc 4,70); hellende daken NO en ZW 56,29 m² onder 35°, plat dak 63,80 m² en plat dak van de dakkapel 4,25 m² (Rc 6,30); dakkapelgevels en -wangen; vloer op grond 155,89 m² (Rc 3,70), z ≤ 0,3, h 0,00 m, omtrek 61,72 m (de funderingslengtes 01 en 02). Ramen U 1,4 / g 0,60, deuren U 1,6 / g 0. Alle lineaire bruggen per vlak zoals opgegeven. De opgetelde verliesoppervlakte is 567,70 m²; het rapport noemt 520,93 m² (compactheid 2,25). Dat verschil is niet te herleiden uit de uitdraai, maar A_ls telt alleen in de eis, niet in de indicatoren.
- Massa: "hsb, sfb of staalskeletbouw met staalbeton of niet-massieve betonnen vloeren", tabel 7.10 rij 250 tot 500 kg/m² (2020 p. 180), D_m 180 kJ/(m²·K) zonder plafond.
- Infiltratie: meetwaarde q_v10 0,40 dm³/(s·m²) per gebouw, gebouwhoogte 7,03 m.
- Zonwering: witte buitenrolluiken op alle ramen (tabel 7.5), handbediend (tabel 7.7).
- Lucht/water-warmtepomp met een Kiwa-verklaring (91850/03), voor verwarming en tapwater. Verwarming: COP 4,60, energiefractie 0,986 en hulpenergie 55 kWh, zoals het rapport ze uit tabel 2.2 van de verklaring neemt (30–35 °C, Q_H;nd/A_g > 150 MJ/m²); de rest levert een elektrisch element. Tapwater: COP 1,95. Installatiejaar 2022.
- Tweepijps, 35 °C, leidinggegevens onbekend (de kern geeft dezelfde forfaitaire 148,23 m), geïsoleerd, geen aanvullende pomp. Vloerverwarming, regeling per ruimte met handmatig overrulen.
- Balansventilatie D.2 forfaitair, enthalpiewisselaar (tabel 11.18, 0,75), 100 % bypass, constant-volumeregeling, toevoerkanaal onbekend, automatische passieve koelregeling. Ventilatoren forfaitair.
- Compressiekoeling forfaitair met vloerkoeling, watergedragen 17/21 °C, pomp 33 W met EEI 0,23, twee bouwlagen.
- PV: 24 panelen JA-Solar JAM60S21-360-HC BK van 360 Wp, zuidwest, 35°, matig geventileerd. BCRG-verklaring 20201714GK: 190 W/m² bij 1,86 m² per paneel.

Per post, elektriciteit in kWh/jr (BENG 2-effect = verschil × 1,45 / 231,61):

| Post | Rapport | Kern | Verschil in BENG 2 |
|---|---|---|---|
| Verwarming, warmtepomp en element (warmte) | 3 326 (13 864) | 3 265 (circa 13 620) | −0,39 |
| Hulpenergie verwarming | 55 | 55 | 0 |
| Tapwater | 3 157 | 3 157 | 0 |
| Ventilatoren, met vorstbeveiliging | 987 | 987 | 0 |
| Koeling, opwekker | 112 | 146 | +0,22 |
| Hulpenergie koeling | 1 | 88 | +0,54 |
| PV (op de meter) | 7 226 | 7 094 | +0,83 |
| **BENG 2** | **2,59** | **3,79** | **+1,20** |

- **Verwarming en tapwater.** Beide rapportposten volgen uit de verklaarde rendementen met f_prac 0,95: 13 664 / (4,60 × 0,95) + 200 = 3 326 kWh en 5 849 / (1,95 × 0,95) = 3 157 kWh. De kern rekent zo ook; het verschil bij verwarming komt alleen uit de lagere warmtevraag.
- **PV.** Uniec telt 24 × 360 Wp = 8 640 Wp. De kern volgt (16.4) met K_pk naar beneden afgerond op 5 W/m² (2020 p. 651): 190 × 44,64 = 8 482 Wp. Dat is ook het vermogen dat het energielabel in hetzelfde rapport noemt. Met 8 640 Wp geeft de kern precies de 7 226 kWh van het rapport. De tekst van 2020+A1 zegt dat het piekvermogen met (16.4) "kan" worden berekend; geval A (Uniec 3.0.16) volgde wel de afronding, geval F (Uniec 3.0.19.4) niet. De kern houdt de afronding van punt 68 aan.
- **Koeling.** De kern rekent een netto koudebehoefte van 274 kWh met een afgifteverlies volgens 10.15 van 176 kWh (64 %). Het rapport noemt 337 kWh koude voor het systeem; bij dezelfde netto behoefte is dat een verlies van ongeveer 23 %. De hulpenergie is de regelenergie van 10.87 (87,6 kWh). Dit zijn de bekende vragen bij 10.15 en 10.87 (zie [nta8800-vragen-nen.md](nta8800-vragen-nen.md)).
- **BENG 1.** De kern rekent 0,8 % minder (74,06 tegen 74,69). Acht ramen hebben in het rapport een zijbelemmering: twee op NO, vijf op ZW (onder meer de pui van 7,59 m²) en één op NW, samen 19,83 m² van 45,48 m² glas (44 %). De uitdraai geeft per raam de hoogte, de afstand en de breedte, in een blok dat bij tekstextractie wegvalt. Met b_b = afstand / breedte (2020 p. 669) staan ze per raam in `windowObstructions`: V0.5 0,32 en V0.6 0,83 (beide ≥ 2,5 m hoog, dus met de koelvoorwaarde van p. 672), A0.4 0,37, A0.5 0,88, A0.6 1,83, A0.7b 2,44, A0.8 3,32 (< 2,5 m) en R0.1 0,12 (links, < 2,5 m). De eerste herbouw, van vóór de belemmering per raam, rekende met minimale belemmering: 73,49 / 3,68 / 96,4. TOjuli is 0, net als in het rapport (actieve koeling).

**Gevonden fout in de kern.** De witte rolluiken zijn een apparaat uit tabel 7.5 (`movableShading.device`). Zonder eigen `reductionFactor` bleef dat veld NaN en kwam het zo in de afgeleide invoer, waarna de controle op eindige getallen de hele berekening weigerde (`non_finite_result`). Elk project met een zonwering uit tabel 7.5 of 7.6 kon daardoor niet rekenen. Het veld wordt nu weggelaten als het niet is opgegeven (test `shading_device_serialises_without_reduction_factor`).
