# Vergelijking met de RVO-voorbeeldwoningen 2022

**Dit is geen officiële referentietoets.** RVO publiceert voor de voorbeeldwoningen geen EP1/EP2/EP3 of label per woningtype, alleen de netto warmtebehoefte Q_H,nd en de Standaard voor woningisolatie. De vergelijking laat zien of de woningopname en de rekenkern op dezelfde orde uitkomen als de validatietool waarmee RVO rekende. Voor de BRL 9501-attestering geldt alleen de toetsset van ISSO-publicatie 54.

## Bronnen

- RVO, [Voorbeeldwoningen bestaande bouw](https://www.rvo.nl/onderwerpen/wetten-en-regels-gebouwen/voorbeeldwoningen-bestaande-bouw): brochure en Excel met de gegevens en rekenresultaten per woningtype (251 tabbladen, per type vijf maatregelpakketten).
- RVO, [Verantwoordingsrapportage voorbeeldwoningen bestaande bouw 2022](https://www.rvo.nl/sites/default/files/2023-01/verantwoordingsrapportage-voorbeeldwoningen-bestaande-bouw-2022.pdf).
- RVO rekende met de validatietool "Rekentool NTA 8800 MWA v220119", dus met NTA 8800:2022. De kern rekent met NTA 8800:2025+C1:2026.

## Opzet

Per woningtype is het pakket "huidig" twee keer doorgerekend:

1. **Basisopname** (`survey` in de fixture): de woning als ISSO 82.1-basisopname met de standaardwaarden van de opname, inclusief de forfaitaire koudebrugtoeslag ΔU_for (NTA 8800 §8.2.1, formule 8.3).
2. **RVO-typering** (`performanceInput` in de fixture): dezelfde rekeninvoer, maar met de U- en Rc-waarden en de raamoriëntaties van RVO en zonder ΔU_for. RVO's uitkomsten passen alleen bij ΔU_for = 0, dus de validatietool rekende zonder koudebrugtoeslag.

Aannames waar RVO niets opgeeft: thermische massa, de omtrek van de begane-grondvloer, de lengte van de tapleidingen en de regeling van de verwarming.

## Uitkomsten (pakket "huidig")

| Woningtype | Q_H,nd RVO | Q_H,nd RVO-typering | Verschil | Q_H,nd basisopname | Standaard RVO | Standaard kern |
|---|---:|---:|---:|---:|---:|---:|
| Rijwoning tussen 1965–1974 | 141,9 | 146,0 | +2,9 % | 147,5 | 57,5 | 58 |
| Rijwoning hoek 1946–1964 | 208,2 | 219,1 | +5,2 % | 262,1 | 84,2 | 84 |
| Vrijstaand 1975–1991 | 157,3 | 150,2 | −4,6 % | 154,0 | 90,3 | 90 |
| Twee-onder-een-kap 1992–2005 | 108,0 | 104,9 | −2,9 % | 114,1 | 78,0 | 78 |
| Portiek tussen/midden 1965–1974 | 90,7 | 93,9 | +3,5 % | 94,7 | 45 | 45 |
| Galerij tussen/midden 1975–1991 | 70,0 | 68,7 | −1,9 % | 69,9 | 45 | 45 |

Waarden in kWh/m². Over alle 29 doorgerekende combinaties van type en pakket ligt de RVO-typering gemiddeld +0,8 % boven RVO (standaardafwijking 4,1 %). Met ΔU_for erbij is dat +8,4 % (tot +24 % bij goed geïsoleerde pakketten).

## Verklaring van de verschillen

- **Koudebrugtoeslag.** De kern telt ΔU_for op bij elke buitenconstructie, zoals formule 8.3 voorschrijft; die formule is in NTA 8800:2022 en 2025+C1:2026 gelijk. RVO's uitkomsten passen bij ΔU_for = 0. Bij geïsoleerde pakketten is de toeslag ongeveer 0,1 W/(m²·K), goed voor +6 tot +24 % warmtebehoefte. Dit is een keuze in RVO's typering, geen rekenfout.
- **Balansventilatie (pakket 4) +7,5 tot +8,6 %.** De besparing door warmteterugwinning zelf komt binnen 0,3–0,6 kWh/m² overeen; het percentage is groot omdat de basis klein is.
- **Hoekwoning en vrijstaande woning (±5 %).** Dit valt binnen de onzekerheid van de eigen aannames: lichte in plaats van zware bouw geeft 4–5 %, de vloeromtrek ±2–3 %.
- **Basisopname hoekwoning 1946–1964 (262 tegen 208).** De opname gebruikt de standaardwaarden van ISSO 82.1 voor het bouwjaar, RVO de gemiddelde waarden uit WoON 2018. Waar RVO zelf de jaarklassewaarden gebruikt (tussenwoning 1965–1974), komt de basisopname binnen +3,4 %.

## Wat deze vergelijking aan de kern veranderde

- De woningopname gaf het bouwjaar niet door aan de rekenkern. Daardoor ontbrak de Standaard voor woningisolatie (§5.3.2) bij elke opname. Het bouwjaar gaat nu mee, ook in de utiliteitsopname, en de Standaard komt voor alle zes typen overeen met RVO.
- Een individuele warmtepomp zonder opgegeven vermogen liet de opname vastlopen zonder melding. ISSO 82.1 tabel 9.6 (p. 110) vraagt dat vermogen niet; de opname rekent dan met NTA-tabel 9.27 (woningen tot en met 25 kW) en legt dat vast. Bij een collectieve warmtepomp vraagt de opname het vermogen (`heat_pump_capacity_required`), omdat tabel 9.29 het nodig heeft.
- Een opname die de rekenkern weigert, toont nu altijd waarom: de meldingen van de kern komen mee onder `derivedInput.…`.

## Regressietest

De zes woningtypen staan als fixtures in `training-data/nta8800-rvo-voorbeeldwoningen-*.json`, met de RVO-waarden, de basisopname en de RVO-typering. De test `rvo_voorbeeldwoningen_stay_within_the_documented_band` (in `crates/nta8800-core/src/opname/mod.rs`) controleert:

- de Standaard voor woningisolatie binnen 0,5 kWh/m² van RVO;
- Q_H,nd van de RVO-typering binnen ±6 % van RVO;
- beide Q_H,nd-waarden binnen 0,5 % van de vastgelegde waarde, zodat een wijziging in de kern opvalt.
