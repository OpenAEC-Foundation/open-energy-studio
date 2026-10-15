# NTA 8800 bijlagen Q, V en W in de Rust-kern

Modules: `annex_q.rs`, `annex_v.rs`, `annex_w.rs` in `crates/nta8800-core`.
Bron: NTA 8800:2025+C1:2026, bijlage Q (p. 1027–1069), bijlage V (p. 1114–1116) en bijlage W (p. 1118–1128). Beeldformules zijn visueel van de pagina's gelezen. De normtekst staat niet in de repository.

Status: **ongeverifieerd**. Er zijn geen referentieberekeningen met productgegevens beschikbaar.

## Bijlage Q: warmtepomp met productgegevens

Generator `heat_pump_annex_q` in de ruimteverwarmingsketen.

| Stap | Normdeel | Uitvoering |
|---|---|---|
| Voorbewerking | Q.49–Q.83 | Gemiddelde condensor- en verdamperspreiding, COP- en vermogensvlakken door de condities 1–3 (L/W met alleen ventilatielucht: lijn door 1 en 2; L/L: lijn door 1 en 3), deellastconstanten per `plj`, carnotgrens voor L/L, `f_cor;bu;max` uit conditie 4 of 0,75. |
| Klassenberekening | Q.3–Q.48, tabellen Q.2–Q.7 | 27 buitentemperatuurklassen; gevraagd vermogen uit de knooppuntvraag en `f_Q;H` (WLE/WHE/ULE/UHE uit `Q_H;nd/A_g`); verdamperintrede volgens bron of opgave; condensorintrede uit tabel Q.5; aan/uit of modulerend met interpolatie tussen deellastpunten; afschakelcriteria; ontdooicorrectie. |
| Hulpenergie | Q.5–Q.8, Q.4.4 | Bronpomp van B/W en W/W, met voor- en nadraaitijd; W/W forfaitair 50 W per kW verdampervermogen. |
| Jaarwaarden | Q.1–Q.4 | `F_H;gen` (maximaal 1) en `η_H;gen;hp`. |
| Ventilatieluchtwarmtepomp | Q.84–Q.95, tabel Q.17 | Tijdfracties voor verwarming en tapwater, bronvermogen en gebruikt ventilatieluchtdebiet als functies. |

Interpretatiekeuzes (ook in de uitvoer `interpretations`):

1. Q.48 noemt −10 °C als nominale verdampertemperatuur voor L/L. L/L heeft geen waterdebiet; de kern gebruikt −10 °C voor L/W.
2. L/L-warmtepompen krijgen een condensorintrede van 20 °C (binnenconditie van tabel Q.10).
3. Bij `B < 0,15` (interval 5) verwijst de formule naar `plj = 5`, dat niet gemeten wordt. De kern gebruikt het 15 %-punt (`plj = 4`).
4. De jaarlijkse `F_H;gen` geldt voor elke maand.
5. Afschakelcriteria die niet zijn opgegeven, worden niet toegepast (opmerking 1 bij tabel Q.1/Q.3).
6. Het forfaitaire bronpompvermogen van een W/W-warmtepomp (Q.4.4, p. 1065) neemt het verdampervermogen bij conditie 1, P·(1 − 1/COP); de norm noemt de conditie niet.
7. Wordt elke klasse van tabel Q.6 gedekt, dan is `F_H;gen` = 1, met of zonder bijverwarming.
8. De bronpomp telt in `η_H;gen;hp` (Q.4) en daarnaast als hulpenergie volgens 9.6.3 (vraag 13 aan NEN).
9. Is in stap 2 van Q.5.3 de warmtevraag `Q_H;node,in(2)` nul, dan is de verhouding van Q.96 onbepaald en blijft f(3) = f(2) (Q.97, p. 1072).
10. Q.84 neemt de tapwatertijdfractie `f_W;t;hp-on` uit 13.149, zoals 13.8.2.4 en 11.23 voorschrijven, niet uit Q.86–Q.88 (vraag 14 aan NEN).

De herberekening van de warmtevraag bij overventilatie (Q.5.3, drie stappen) zit in de verwarmingsketen ([ventilatie](nta8800-ventilatie.md)). Niet uitgevoerd: luchtdebietinterpolatie bij variabel debiet (tabel Q.9). Bijlage Q geldt alleen voor ruimteverwarming; tapwaterwarmtepompen volgen hoofdstuk 13.

## Bijlage V: regeneratie

- Tabel V.1: `c_source` 1,00 / 1,02 / 1,04 bij een regeneratiegraad `R` onder 0,5, tot 0,75 en vanaf 0,75.
- `R` (V.1): vrije koeling uit dezelfde bron (V.2: de jaarlijkse koudebehoefte) plus zonneregeneratie (V.4/V.5, mei–september, rendement uit tabel V.2 via V.6 of opgegeven en afgerond volgens bijlage X), gedeeld door de aan de bron onttrokken warmte.
- Tabel V.3: collectieve grondwaterbron, recirculatie 1,00 en doublet 1,04 (functie; nog niet als keteninvoer gekoppeld).
- In de keten telt alleen de warmte van de ruimteverwarmingswarmtepomp in de noemer; tapwater op dezelfde bron is daar niet bekend.

## Bijlage W: boosterwarmtepomp

Opwekker `booster_heat_pump` in de tapwaterketen.

- COP per maand uit de brontemperatuur (W.11–W.13), gecorrigeerd voor de jaarlijkse tapmenge met het stilstandsverlies (W.14, methode 3).
- Elektriciteit (W.1, `f_prac` 0,95) en warmte uit het collectieve systeem (W.2) worden naar boven afgerond volgens bijlage X.
- Verdamperwarmte (W.8) en compensatie van het stilstandsverlies (W.9/W.10).
- Warmte uit het koelsysteem (W.3) bij opgave van `Q_C;HP;si;mi` per maand.
- De warmte uit het collectieve systeem telt op de drager van dat systeem: externe warmte (`dh`, η 1,0) of een opgegeven collectieve opwekker met rendement. De elektriciteit van de booster is hulpenergie.
- De brontemperatuur moet binnen 4 K van de gemeten temperaturen liggen (W.3.1).
- De koelkengetallen W.4–W.7 (p. 1121–1123) noemt de norm "ter indicatie". `annex_w::booster_cooling_indication` rekent ze uit, maar alleen ter rapportage: W.7 de looptijd t = Q [MJ] / P_bron [W] (P_bron bij een bron van 24 °C), W.6 de circulatiepomp met 2 W/m² over de gekoelde A_g, W.4 η = Q / E en W.5 F = Q / (Q + Q van de andere koudeopwekkers). De hulpenergie voor de koudedistributie volgt uit hoofdstuk 10; deze waarden gaan dus niet in de keten.
- Niet uitgevoerd: methoden 1 en 2 van W.3.3 (interpolatie tussen klassen, tabel 13.27).
