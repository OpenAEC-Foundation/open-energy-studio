# Vergelijking met openbare BENG-rapporten — 5 oktober 2026

Dit is **geen officiële referentietoets**. Drie openbaar gepubliceerde BENG-rapporten zijn in de app nagebouwd en met de rekenkern doorgerekend. Die rapporten zijn gemaakt met geattesteerde software (Uniec) onder oudere NTA 8800-edities. Verschillen zijn daarom deels verwacht. De officiële toetsing voor BRL 9501 loopt via ISSO-publicatie 54 versie 5.0:2026, die niet openbaar is.

De herbouwde invoer staat als fictieve, geanonimiseerde fixtures in `training-data/nta8800-public-comparison-{a,b,c}.json`. Adressen en namen zijn verwijderd. De test `crates/nta8800-core/tests/public_comparison.rs` legt de huidige uitkomsten vast als regressie; de gepubliceerde waarden zijn alleen context.

## Bronnen

| Geval | Bron | Software en NTA-editie |
|---|---|---|
| A | [openbaar rapport A (vrijstaande woning, plat dak)](https://www.starlinehome.nl/uploads/20210923-Beng_berekening_21.164_002.pdf) | Uniec 3.0.16 (2021), NTA 8800:2020 |
| B | [openbaar rapport B (rijwoning)](https://repository.officiele-overheidspublicaties.nl/Bijlagen/TerInzageLegging/2024/til-2024-415/1/bijlage/26.2220837_-_Rap._BENG-berekening.pdf) | Uniec 3.1.6.2 (2023), NTA 8800:2022 |
| C | [openbaar rapport C (vrijstaande woning, twee zones)](https://www.oud-osdorp.nl/wp-content/uploads/2024/03/Rap.-BENG-berekening-V1.0_18-03-2024.pdf) | Uniec 3.2.7.0 (2024), NTA 8800:2023 |

Twee andere rapporten zijn niet gebruikt: een woongebouw met 28 appartementen in meerdere zones, en een verhalend rapport zonder volledige invoerlijst.

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
| Tapwater: kolom binnendiameter in tabel 13.2 (editie 2023) | versie | – | – | +1,35 / −0,9 | versieverschil |
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

De vragen over 10.15 en 10.87 staan in [nta8800-vragen-nen.md](nta8800-vragen-nen.md).

## Conclusie

Geen van de verschillen wijst op een formulefout in de kern. Na correctie van de invoer en f_prac zijn de resterende verschillen herleid tot twee bewuste letterlijke lezingen van de norm (10.15 en 10.87), drie versieverschillen tussen de edities en een klein bouwfysisch restant. De gebruikte standaardwaarden per geval staan als bronvermelding (`sourceReference`) in de fixtures.
