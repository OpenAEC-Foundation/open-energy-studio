# NTA 8800 hoofdstuk 11: ventilatie in de Rust-kern

Module: `crates/nta8800-core/src/ventilation.rs`
Route: `POST /v1/nta8800/ventilation/calculate`, MCP-tool `calculate_ventilation` en Tauri-command `calculate_ventilation`.
Bron: NTA 8800:2025+C1:2026, pagina 436–520. De normtekst zelf staat niet in de repository.

Status: **ongeverifieerd**. Er zijn nog geen referentiegevallen vergeleken (zie het [referentieprotocol](nta8800-referentieprotocol.md)).

## Wat de module berekent

De module rekent per rekenzone en per maand, apart voor de warmtebehoefte en de koudebehoefte (§11.2). Elke maand doorloopt dus twee keer het stappenplan van 11.2.1.1.

| Stap | Normdeel | Uitvoering |
|---|---|---|
| Benodigde buitenluchtvolumestroom | 11.22, 11.23 | f_ctrl uit tabel 11.5 en tabel 11.6 met τ_sysC (tabel 11.7). f_sys is 1,00. f_prac;req is 0,95 en ε_V is 1. Overventilatie voor een warmtepomp op ventilatieretourlucht verloopt via 11.23/11.23a. f_buitenlucht volgt uit 11.24. |
| Ontwerpvolumestroom | 11.55–11.65, tabel 11.8 en 11.9 | Rekenwaarde of geïnstalleerde capaciteit (de hoogste telt). Bij vraagsturing in woningen is de warmtebehoefte q_V;inst = 0. f_terugregel geldt voor recirculatie en debietregeling. De ondergrens is 35 dm³/s per woning, of naar rato bij een opgesplitste woning. Daarnaast gelden f_lea;du en f_lea;ahu. |
| Systeemvarianten | 11.2.2.2, tabel 11.5 | A, B, C, D en het gecombineerde systeem E.1 (decentraal D.5b plus een ander systeem, 11.29–11.45 en 11.50–11.54). |
| Spuiventilatie | 11.66–11.69, tabel 11.10 | τ_argI is 0,01, of komt uit tabel 11.7 bij de koudebehoefte van woningen. |
| Ventilatieve koeling | 11.70–11.79 | Alleen bij de koudebehoefte. Enkelzijdige ventilatie en dwarsventilatie worden automatisch bepaald uit de openingen. De nettodoorlaat volgt uit 11.71a/b. De bediening bepaalt f_argII. |
| Open verbrandingstoestellen | 11.80–11.83, tabel 11.11 en 11.12 | Toestellen met en zonder afvoer. f_τ;verbr verschilt voor warmte en koude. P_h;fi is minimaal P_h;fi;min. |
| Infiltratie | 11.84–11.86, tabel 11.13 en 11.14 | Een gemeten waarde (NEN 2686) of de rekenwaarde. |
| Luchtstroommodel | 11.1–11.18, tabel 11.1–11.3 | Openingen per luchtstroomzone (H < 15 m, 15–50 m of > 50 m), met een kruipruimtevloer vóór 1992. Externe druk via 11.1. Massastromen met dichtheid 11.4. De interne referentiedruk volgt de voorgeschreven routine (stappen 1–12, bisectie) tot de nauwkeurigheid van 11.14. |
| Effectieve volumestromen | 11.19–11.21, tabel 11.4 | Infiltratie, natuurlijke toevoer, spuien, verbrandingslucht en mechanische toevoer, elk met de eigen temperatuur. |
| Toevoertemperatuur | 11.99–11.130 | Vorstbeveiliging (tabel 11.16), WTW met f_prac;hr (11.107–11.109, tabel 11.17 en 11.18), bypass en koudeterugwinning (11.106a), recirculatie (11.110–11.113), ventilatorwarmte (11.3.2.7), kanaalverlies (tabel 11.19) en elektrische voorverwarming in roosters (11.123/11.124). |
| Hulpenergie | 11.105/11.106, 11.125–11.128, 11.131–11.142 | Vorstbeveiliging, voorverwarming in roosters en ventilatoren. Het ventilatorvermogen is forfaitair (tabel 11.23) of opgegeven (11.133–11.138, tabel 11.20–11.22). |

De uitvoer `demandFlows` geeft per luchtstroom k en per maand de warmteoverdrachtscoëfficiënt H_ve;k = ρ_a·c_a·q_V;k/3600 en de toevoertemperatuur, voor verwarming en voor koeling. Hoofdstuk 7 rekent daarmee de ventilatieverliezen (7.18–7.20).

### Vast C1-systeem voor BENG 1

`fixedC1` is een tweede run met het vaste ventilatiesysteem C1 volgens §5.4.3:

- EXTRACT_OP, met f_ctrl 1,00 voor woningen en 1,32 voor utiliteitsgebouwen;
- f_lea;du = 1,05;
- geen luchtbehandelingskast (LBK), geen geïnstalleerde capaciteit en geen ondergrens 11.63–11.65;
- geen verbrandingslucht, geen overventilatie en geen voorverwarming in roosters;
- geen maximale benutting van de capaciteit voor koeling.

Infiltratie, spuien en de bouwkundige voorzieningen voor ventilatieve koeling blijven gelijk aan de werkelijke situatie.

## Niet ondersteund (expliciete afwijzing)

- Een luchtbehandelingskast die de toevoerlucht verwarmt of koelt (tabel 11.15, 11.100/11.101, 11.114–11.121). Foutcode: `ahu_supply_air_conditioning_unsupported`.
- Een specifiek gastoestel met afvoer gelijktijdig met de mechanische afvoer (tabel 11.11, voetnoot a). Foutcode: `specific_gas_appliance_unsupported`.
- Een wisselstroomventilator van na 2006 in de forfaitaire methode (tabel 11.23 geeft geen waarde). Foutcode: `forfait_fan_ac_after_2006_unsupported`.

## Interpretatiekeuzes

Deze keuzes staan ook in de uitvoer (`interpretations`) en in het verificatiedossier:

1. Voor woningen gebruikt f_τ (tabel 11.8) de gemiddelde woninggrootte A_g;zi/N_woon. Bij een zone met meerdere woningen zou de totale A_g altijd 0,8 geven.
2. De ventilatorenergie (11.132) gebruikt q_V;ODA;req van de warmtebehoefte. De norm noemt aparte berekeningen voor warmte en koude, maar levert één energiegebruik op.
3. Formule 11.137 wordt letterlijk toegepast, met de waarden uit tabel 11.21: f_regfan = Σ f_q;k·t_d;k = 0,58. Opmerking 11.138 suggereert dat de verhoudingen gekwadrateerd worden. Dat geeft 0,364 als f_q de debietverhouding is.
4. Ventilatieve koeling wordt voor de hele zone berekend. Bij meerdere luchtstroomzones wordt de stroom verdeeld via 11.6–11.10.
5. ΔC_p bij dwarsventilatie gebruikt de klasse uit tabel 11.3 die hoort bij de gebouwhoogte.
6. De nauwkeurigheid volgens 11.14 gebruikt zonetotalen, ook per luchtstroomzone.
7. Bij precies H = 50 m wordt de zone in twee luchtstroomzones verdeeld (11.6/11.7). Tabel 11.1 en 11.8 verschillen op deze grens.

## Koppeling

De koppeling aan hoofdstuk 7 (`monthly_demand`, met aparte warmte- en koudewaarden per stroom) en aan de energieprestatieketen (ventilatorenergie, vorstbeveiliging en de aparte BENG 1-run) volgt zodra de herziening van hoofdstuk 7 is samengevoegd.
