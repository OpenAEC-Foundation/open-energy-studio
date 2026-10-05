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

| Geval | Gepubliceerd | Kern vóór (forfaitaire WP) | Kern na (kwaliteitsverklaring / 6.2b) |
|---|---|---|---|
| A | 92,99 / 25,19 / 80,4 | 94,00 / 75,59 / 53,8 | 94,00 / 33,84 / 76,0 |
| B | 54,61 / 27,10 / 64,8 | 52,96 / 38,04 / 51,5 | 52,96 / 24,10 / 67,3 |
| C, één zone | 64,47 / 28,70 / 70,9 | 63,41 / 30,18 / 70,0 | ongewijzigd |
| C, twee zones | 64,47 / 28,70 / 70,9 | BENG 1 niet berekenbaar / 31,44 / 69,1 | 64,70 / 31,44 / 69,1 |

A_g, A_ls en de PV-opbrengst komen in alle drie gevallen exact overeen. TOjuli is overal 0.

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

## Verklaring van de resterende verschillen

- **A, BENG 2 +34 % na de verklaring.** Verwarming (1 243 kWh) en tapwater (1 590 kWh) komen nu overeen met het rapport. Het verschil zit vooral in de koeling.
  - Koudeafgifte: formule 10.15 van de 2025-editie geeft bij vloerkoeling een veel groter verlies dan de 2020-editie die het rapport gebruikte (versie-effect).
  - Hulpenergie van ventilatoren en pompen: forfaitair, omdat productwaarden in het rapport ontbreken.
- **B, BENG 2 −11 % na de verklaring.** Wij rekenen met minder ventilatorenergie (129 tegen 360 kWh), omdat de BCRG-opgave van de ventilatie niet eenduidig te lezen is. Daar staat een groter koudeafgifteverlies tegenover. Overstekken en zijbelemmering zijn weggelaten en de thermische massa is aangenomen (warmtebehoefte −5,7 %).
- **C, BENG 2 +5 % (één zone) en +9,5 % (twee zones).**
  - Tapwaterleidingen: tabel 13.2 van de 2025-editie mist de binnendiameterkolom, met +7 % op tapwater (versie-effect).
  - Koeling: de regelenergie van 10.87.
  - CO₂: de elektriciteitsfactor 0,268 in tabel 5.2 van 2025, tegen 0,34, met −17 % (versie-effect).

Geen van de verschillen wijst op een formulefout in de kern. De gebruikte standaardwaarden per geval staan als bronvermelding (`sourceReference`) in de fixtures.
