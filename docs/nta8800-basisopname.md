# Basisopname bestaande woningen (ISSO 82.1)

Module: `crates/nta8800-core/src/opname/` (`mod.rs` plus een submodule per ISSO-hoofdstuk)
Route: `POST /v1/nta8800/opname/residential` met `{ "survey": … }`, MCP-tool `assess_residential_survey` en Tauri-command `assess_residential_survey`.
TS-client: `assessResidentialSurveyWithRust` in `src/core/nta/KernelClient.ts`.
Bron: ISSO 82.1, 7e druk (2025), met het erratum van 6 januari 2026 en Wijzigingsdocument 82.1 2025 v1.1. Paginanummers verwijzen naar de pdf; de ISSO-tekst zelf staat niet in de repository.

Status: **ongeverifieerd**. De laag is niet getoetst aan referentieberekeningen en is geen geattesteerde basisopname-software (BRL 9500/9501).

## Werking

De opname legt vast wat de adviseur ter plaatse vaststelt, met "onbekend" waar dat niet lukt. De laag past de ISSO-herkenningsregels en standaardwaarden toe en bouwt daaruit de kerninvoer (`BuildingPerformanceInput`):

- maandbehoefte met hoofdstuk 11-ventilatie;
- verwarmingsketen;
- warm tapwater;
- PV.

Daarna rekent de kern de energieprestatie en de indicatieve labelklasse. Elke toegepaste standaardwaarde verschijnt in `appliedDefaults` met regel, pad, waarde en ISSO-pagina, voor het dossier. `warnings` noemt benaderingen. `issues` noemt invoer die ontbreekt of niet wordt ondersteund.

## Geïmplementeerde regels

| Onderdeel | Regel | ISSO 82.1 |
|---|---|---|
| Algemeen | Jaar van een toestel: fabricagejaar, dan installatiejaar, dan bouwjaar | p. 28 |
| Renovatiejaar | Beslisschema afb. 7.3. Is het jaar onbekend, dan geldt het eerste jaar van de volgende jaarklasse | p. 52–55 |
| Woningtype | Een niet in te delen eengezinswoning is een hoekwoning. "Deels plat" geldt alleen bij vrijstaande woningen. Daarmee volgt het type van NTA-tabel 11.14 | p. 51–52 |
| Thermische massa | Tabellen 7.5/7.6 (erratum §2) worden omgezet naar de klassen van NTA-tabel 7.10. Een gesloten of verlaagd plafond (`closedOrSuspendedCeiling`) kiest de eerste kolom bij elk vloertype; `lighterCeiling` doet dat voor een (zeer) zware vloer met een lichter plafond | p. 62 |
| Opake constructies | R_c volgens bouwjaar en isolatiestaat via NTA-bijlage I (`forfait_envelope`). Een nageïsoleerde spouw met onbekende breedte volgt tabel 8.26 (40/70/100 mm) | p. 84–93 |
| Renovatie of aanbouw | Bij "aanwezig, dikte onbekend" met `renovation`: is het jaar bekend en is er bewijs dat aan de eis van dat jaar is voldaan, dan geldt de jaarklasse van dat jaar. Is het jaar bekend zonder dat bewijs, dan de klasse ervóór, ten hoogste "1992 tot 2014" (R_c 2,5). Is het jaar onbekend, dan de klasse na die van het bouwjaar; vóór 1965 de kolom "(na)geïsoleerd". Zonder `renovation` blijft de jaarklasse van het bouwjaar gelden (§8.7.2, prioriteit 3) | p. 84–85 (§8.7.2.1, afb. 8.14) |
| Scheiding met onverwarmde ruimte | R_se is de R_si van de onverwarmde ruimte bij dezelfde warmtestroomrichting (0,13 / 0,10 / 0,17), niet 0,04 | NTA 8.4.2.1 |
| Sterk geventileerde ruimte | Een garage of andere sterk geventileerde ruimte (`strongly_ventilated`) telt voor het verlies als buitenlucht (zelfde R_c-rij, R_se, f_ls 1, ΔU_for), maar zonder zoninstraling op de delen die eraan grenzen | p. 41 (§6.3.4); WD 2025 p. 22–24; NTA 3.134, 6.3 |
| Beglazing | Gelijkstellingen: dubbel + voorzetraam = HR, enzovoort. U- en g-waarde uit tabel 8.14/8.15. Een raam zonder kozijn telt als hout/kunststof. Kozijnfractie 0,25 (NTA 7.6.6.2, methode B) | p. 93–95 |
| Deuren en panelen | Een deur met minder dan 65 % glas wordt gesplitst in een raam- en een deurdeel. Is niet vast te stellen of de deur geïsoleerd is, dan geldt ongeïsoleerd. Panelen volgen tabel 8.18–8.21 | p. 70, 95–97 |
| Koudebruggen | Forfaitair voor de hele woning: ΔU_for (NTA 8.2/8.3) op alle buitenvlakken; vloeren op grond krijgen 0,5·P | p. 79 |
| Aangrenzende onverwarmde ruimten | H_ue = 5·A (NTA I.8) en b_U = H_ue/(H_ue + H_iu) | NTA I.2.4, 8.4.1 |
| Belemmering | De adviseur bepaalt de situatie per raam. Zonder opgegeven factoren geldt "minimale belemmering" | p. 102–103 (tabel 8.24/8.25) |
| Vloeren naar buitenlucht | Rij "daken en vloeren grenzend aan de buitenlucht" van tabel 8.9/8.10, R_si 0,17. Een plafond naar een onverwarmde zolder (AOR) is een zoldervloer (`attic_floor`: vloerrij, R_si 0,10) | p. 88–90; NTA I.4 |
| Thermokussens | R_c 1,95, ongeacht het antwoord over isolatie | p. 93; WD p. 37 |
| Rieten daken en gevels | Riet gemeten aan de onderzijde min 35 mm, afgerond op 50 mm. Zonder (of onbekende) isolatie tabel 8.12 (d/0,105). Met isolatie formule 8.5 (d_iso/0,045 + d_riet/0,105), 40 mm bij onbekende dikte | p. 92 (afb. 8.16) |
| Onverwarmde kelder | Het gat in de begane grondvloer is een fictieve ongeïsoleerde vloer (R_c 0,15), omtrek 0,01 m als die ontbreekt, R_bw van de gevel boven de rest van de vloer | p. 72 |
| Kruipruimtewand | R_bw is de R_c van de gevel met de laagste R_c, niet 1/U − 0,17 | p. 93 (tabel 8.13) |
| Leidingdoorvoeren | Onbekend → één ongeïsoleerde leiding per bouwlaag van de woning, elk met N_bouwlaag = aantal bouwlagen (formule 7.17, 1,8 W/K per bouwlaag). Een lege lijst betekent geen doorvoeren. De kern telt H_p mee in H_tr (7.16) | p. 63 (tabel 7.7); NTA 7.3.3 |
| Verwarming | Geen opwekker aanwezig → CR-ketel. Waakvlam onbekend → met waakvlam. Waterstofketel → HR-107. Watergedragen bron onbekend → bodem. Ontwerptemperatuur onbekend → tabel 9.9. Bij meerdere afgiftesystemen gaat oppervlakteverwarming voor. Inregeling onbekend → niet ingeregeld. Regeling onbekend → overig | p. 107–124 |
| Warmtepomp | Bodem/grondwater zonder zonneregeneratie → c_source 1,0 (NTA-tabel 9.27 voetnoot a); met regeneratie de opgegeven bijlage V-factor. Productgegevens volgens tabel 9.5 → rij tabel 9.28 met `highEfficiencyEvidence`. Een afvoerluchtwarmtepomp vraagt een tweede opwekker en wordt afgewezen (`exhaust_air_heat_pump_second_generator_required`) | p. 109–110; WD p. 43 |
| Biomassa | Bijlage R onbekend → niet conform | p. 111–112, p. 28 |
| Distributie bij externe warmte, elektrisch of biomassa met watergedragen afgifte | Pomp forfaitair, leidingen ongeïsoleerd. Geen warmtemeter: tabel 9.16a geldt alleen voor collectieve installaties | p. 117–122 |
| Leidingen in onverwarmde ruimten | Bevat het gebouw een kruipruimte, kelder of andere onverwarmde ruimte en is niet vastgesteld dat er geen cv-leidingen lopen, dan zijn ze aanwezig met de forfaitaire lengte (15 % van L, 9.26). De distributie gaat dan naar de berekende route (9.26–9.40) | p. 120 (afb. 9.1), p. 121 |
| Tapwater | Geen systeem → elektrisch doorstroomtoestel. Gastoestel onbekend → badgeiser. Gaskeur onbekend → geen. CW-klasse (`cwClass`, alleen met Gaskeur): aanrecht/CW-1 → klasse 1, CW-2, CW-3, CW-4/5/6 of onbekend → klasse 4 (`measuredClass`). Keukengeiser boven 13 kW → badgeiser. DWTW onbekend → niet aangesloten | p. 164–180 (tabel 13.6) |
| Elektrische boiler | Vat via `boilerVessel`: volume verplicht, behalve een keukenkastboiler (10 l). Label onbekend → fabricagejaar; jaar onbekend → bouwjaar; plaats onbekend → buiten de zone; aansluitfactor 2 | p. 172–174 (tabel 13.10) |
| Ventilatie | Zelfregelende roosters (tabel 11.3/11.5). Sturing onbekend → geen. WTW onbekend → geen. Tegenstroom met onbekend materiaal → aluminium. Toevoerkanaal en bypass volgens tabel 11.10–11.12. Kanaaldichtheid onbekend → 1,1. Ventilatormotor onbekend → wisselstroom (tot en met 2006) of gelijkstroom (vanaf 2007). Fabricagejaar ventilator onbekend → bouwjaar (tabel 11.15 gaat voor de algemene regel van p. 28) | p. 142–154 |
| PV | Kristallijn type bekend, jaar onbekend → bouwjaar (vóór 2001 telt als 2000). Type onbekend → polykristallijn met het installatiejaar, of "geplaatst vóór 2001" als ook dat onbekend is. Montage onbekend → niet geventileerd | p. 191 (tabel 15.7) |
| Energieopslag | Gebouwgebonden elektrische en thermische opslag (kWh), alleen met PV | p. 193 (§15.5) |

## Bekende bronfouten

- ISSO-tabel 7.4 drukt 350 af voor "zeer zware vloer, zware wand". NTA-tabel 7.10 en het wijzigingsdocument geven 450. De kern rekent met NTA-tabel 7.10, dus met 450.
- De R_c van thermokussens is 1,95 (WD p. 37) in plaats van 1,80. De opname rekent altijd met 1,95, ook bij een opgegeven dikte.
- λ riet is 0,105 (WD p. 36) in plaats van 0,2.

## Niet ondersteund

- Koeling: geeft de fout `cooling_not_supported_in_basisopname`.
- Collectieve installaties, WKK, zonneboilers en meerdere opwekkers.
- Serres (AOS), daklichten en woonboten/woonwagens.
- Situaties van overstekken, balkons en galerijen (tabel 8.24/8.25): de kern kent nog alleen "minimale belemmering" en opgegeven maandfactoren (§17.3-tabellen ontbreken).
- Het nominale vermogen van een tapwaterwarmtepomp (§13.3.2.5): de kern toetst de capaciteit van tapwatertoestellen (13.8.2) nog niet.
- Een afvoerluchtwarmtepomp: die vraagt een tweede opwekker (WD p. 43), en de opname kent één opwekker.
- Detailopname-routes en kwaliteitsverklaringen. De uitzondering is een gemeten q_v10.

## Interpretatievragen

1. Een appartement met een onbekende positie aan de zijkant is als kop- of hoekligging ingedeeld. P. 51 vraagt een onderbouwde keuze en geeft geen invoer bij onbekend; de laag kiest conservatief (p. 28).
2. Een deur waarvan de isolatie niet vast te stellen is, telt als ongeïsoleerd. ISSO noemt dat niet expliciet; dit volgt uit de conservatieve regel op p. 28.
3. ISSO-klasse 70/50 heeft geen rij in NTA-tabel 9.14. De distributie gebruikt 70/60 (gelijke aanvoertemperatuur). De ketel gebruikt de gemiddelde temperatuur 60 °C.
4. ΔU_for (8.3) wordt niet toegepast op scheidingen met onverwarmde ruimten, omdat H_D;for alleen de buitenlucht betreft.
5. Een combitoestel met Gaskeur CW zonder HR telt als "combi met Gaskeur" (0,50). Tabel 13.25 kent alleen de combinatie HR + CW als hogere rij.
6. Formule 8.5 voor riet heeft geen R_ad en rondt half naar boven af op 50 mm. NTA I.2.1.4/I.3 telt R_ad wel mee en rondt naar beneden af. De opname volgt ISSO.
7. Leidingdoorvoeren (tabel 7.7): het aantal leidingen is het aantal bouwlagen, en elke leiding telt volgens 7.17 met alle bouwlagen van de zone. Een woning van twee bouwlagen krijgt dus 2 × 2 × 1,8 = 7,2 W/K.
8. Een nageïsoleerde spouw van vóór 1930 met onbekende breedte krijgt 40 mm. Tabel 8.26 begint bij 1930.
9. Een deels plat dak bij een niet-vrijstaande woning telt als hellend dak; ISSO kent deels plat alleen bij vrijstaande woningen.
10. Renovatie zonder bewijs volgt afb. 8.14 letterlijk, ook als de klasse vóór het renovatiejaar lager is dan die van het bouwjaar (bijvoorbeeld bij een aanbouw). "Aanwezig, dikte onbekend" zonder `renovation` geldt vanaf 1965 als oorspronkelijke isolatie (jaarklasse van het bouwjaar).
11. Een raam in een wand naar een sterk geventileerde ruimte telt in H_D als buitenraam (U voor buiten), zonder zonwinst.

# Basisopname bestaande utiliteitsgebouwen (ISSO 75.1)

Module: `crates/nta8800-core/src/opname/utility.rs`.
Route: `POST /v1/nta8800/opname/utility` met `{ "survey": … }`, MCP-tool `assess_utility_survey` en Tauri-command `assess_utility_survey`.
TS-client: `assessUtilitySurveyWithRust` met het type `UtilitySurvey` in `src/core/nta/KernelClient.ts`.
Bron: ISSO 75.1, 7e druk (printdatum 11-12-2025). Paginanummers verwijzen naar de pdf; de ISSO-tekst zelf staat niet in de repository. Alleen basisopname-regels zijn toegepast, geen maatwerkadvies (ISSO 75.2).

Status: **ongeverifieerd**, net als de woninglaag.

## Werking

De utiliteitslaag hergebruikt de gedeelde delen van de woninglaag: schil (`envelope.rs`, met bijlage I via `forfait_envelope`), verwarming, tapwatertoestellen, PV, de `Recorder` en de lijst `appliedDefaults`. Regels die ISSO 75.1 gelijk aan ISSO 82.1 voorschrijft, krijgen in de uitvoer de ISSO 75.1-pagina. Utiliteitsspecifiek zijn:

- gebruiksfuncties met oppervlakten;
- collectieve verwarming;
- koeling;
- ventilatie met LBK, recirculatie en debietregeling;
- bevochtiging;
- utiliteitstapwater;
- verlichting;
- BACS.

**Eén rekenzone.** Het gebouw is één rekenzone met de grootste gebruiksfunctie als hoofdfunctie. Andere functies tot samen 25 % van A_g tellen mee als hoofdfunctie (p. 39–40). Liggen ze daarboven, dan meldt de laag `mixed_functions_require_zones`. Gebouwen met meer functies moeten dus nog per functie als aparte opname (één zone per functie) worden ingevoerd. De kern kan sinds §6.5.3 gemengde zones rekenen, maar de opnamelaag gebruikt dat nog niet.

**Bevochtiging.** Stoombevochtiging wordt in twee stappen berekend. De eerste kernrun levert de mechanische toevoerdebieten uit hoofdstuk 11. Daaruit volgt met 12.1–12.3 de maandelijkse stoomenergie. Die komt als `declaredUses`-post met de nieuwe dienst `humidification` (E_hum van 5.20, zonder f_BACS) in een tweede run. Adiabatische bevochtiging geeft de waarschuwing `adiabatic_humidification_load_not_in_heating_chain`: de latente last hoort bij de verwarming, maar de keten neemt die nog niet op. Ontvochtiging (tabel 12.2) is nog niet gekoppeld.

## Geïmplementeerde regels

| Onderdeel | Regel | ISSO 75.1 |
|---|---|---|
| Functies | Andere functies tot 25 % van A_g worden samengevoegd met de hoofdfunctie. Een woonfunctie hoort in de woningopname | p. 39–40 |
| Gebouwtype | Eén- of meerlaags, ligging en daktype bepalen de rij van NTA-tabel 11.14. "Deels plat" geldt alleen bij vrijstaande gebouwen | p. 55–56 |
| Toestel- en renovatiejaar | Zoals bij woningen | p. 30, 57–59 |
| Zonwerende beglazing | Zichtbaar zonwerend glas of folie → g = 0,4 | p. 96 (tabel 8.14) |
| BACS | Is het vermogen onbekend, dan geldt een systeem dat meer dan 2.500 m² bedient als systeem boven 290 kW. Is ook het bediende oppervlak onbekend, dan telt de A_g van het gebouw. BACS onbekend → afwezig. Automatisering onbekend → klasse D. Beheer onbekend → klasse C/D. Een systeem boven 290 kW zonder conform BACS krijgt f_BACS = 1,05 | p. 62–63 (tabel 7.3) |
| Grote installatie | Bedient een installatie meer dan 500 m², dan staat de technische ruimte (ketel, LBK) per definitie buiten de thermische zone | p. 17, 117 |
| Collectieve verwarming | Ketelrol collectief, met het nominale vermogen voor de hulpenergie (9.91). Warmtepomp in de utiliteitsscope van tabel 9.27 (zonder c_source en zonder rij 9.28). Berekende distributie (9.26–9.51) met forfaitaire pomp, leidingen ongeïsoleerd en warmtemeter aanwezig (tabel 9.16a, alleen collectief) | p. 108–123 |
| Leidingdoorvoeren | Onbekend → één ongeïsoleerde leiding per toiletgroep (`toiletStacks`) door alle bouwlagen. Zonder opgave van toiletgroepen rekent de laag met één groep | p. 68 (tabel 7.8) |
| Leidingen in onverwarmde ruimten | Zoals bij woningen (afb. 9.1): met een kruipruimte, kelder of onverwarmde ruimte en zonder vastgestelde afwezigheid zijn de leidingen aanwezig met de forfaitaire lengte (15 % van L). Zijn ze afwezig, of is er geen onverwarmde ruimte, dan geeft de laag 0 m door; anders zou de berekende distributie 15 % van L in onverwarmde ruimte aannemen | p. 121–122 (afb. 9.1, tabel 9.14) |
| Renovatie, R_se naar onverwarmde ruimten, sterk geventileerde ruimten, thermische massa | Zoals bij woningen | p. 66 (tabel 7.5), 88–89 |
| Koeling | Vermogen onbekend → forfait. Ontwerptemperatuur onbekend → 6/12, bij alleen stralingskoeling 17/21. Inregeling onbekend → niet ingeregeld. Leidingen onbekend → ongeïsoleerd, isolatiejaar = bouwjaar. Koudemeters onbekend → aanwezig. Betonkernactivering telt als vloerkoeling. Binnendelen van split/VRF tellen als ventilatorconvectoren. Regeling onbekend → overig. WKO met onbekend jaar → vergunningsjaar, anders vóór 2013 | p. 131–138 |
| Ventilatie | Zelfregelende roosters, regeling, WTW, bypass en kanaaldichtheid zoals bij woningen. Kanaaldichtheid wordt alleen bij bewezen LUKA A–C verlaagd | p. 142–153 |
| Recirculatie | Aanwezig met onbekend percentage → de NTA-waarde x = 20 (11.60). Onbekend of aanwezig → geen. Een aangetoond hoger percentage wordt naar beneden afgerond op tientallen; lager dan 20 telt niet | p. 148 (tabel 11.7); NTA 11.60 |
| Debietregeling | Onbekend → geen. Minimumdebiet onbekend → 80 %. Een aangetoond lager percentage wordt naar boven afgerond; hoger dan 80 telt niet | p. 149 (tabel 11.8); NTA 11.61 |
| LBK-kanalen buiten de zone | Lengte onbekend → ≥ 40 m. Isolatie onbekend → R < 1,0. Situatie volgens NTA-tabel 11.19 | p. 153 (tabel 11.14) |
| Ventilatoren | Forfait. Motor onbekend → wisselstroom (tot en met 2006) of gelijkstroom (vanaf 2007). Fabricagejaar onbekend → bouwjaar | p. 154 (tabel 11.15) |
| Tapwater | Geen systeem → elektrisch doorstroomtoestel. Gastoestellen zoals bij woningen. Collectieve opwekker onbekend → overige direct verwarmde voorraadvaten. Gasboiler: jaar onbekend → bouwjaar, plaats onbekend → buiten de zone. Tappuntlengte onbekend → > 3 m. Circulatie forfaitair. DWTW alleen bij functies met douches | p. 164–178 |
| Voorraadvaten | Verplicht bij elektrische en indirect gestookte boilers. Label onbekend → volgt het fabricagejaar. Fabricagejaar onbekend → tot en met 2017. Aansluiting onbekend → ongeïsoleerd (f_sto;dis;ls = 5; elektroboilers 2). Plaats onbekend → buiten de zone | p. 172–173 (tabel 13.10) |
| Verlichting | Vermogen onbekend → forfait tabel 14.5. Bij forfait geldt centraal aan en geen daglichtregeling. Het forfait geldt dan voor alle verlichtingszones (NTA 14.3.4). De LED-waarde vraagt LED vanaf 2017 in alle zones: gemeten zones tellen alleen mee als al hun lampen LED zijn, een armaturenlijst telt als niet-LED. Alleen ruimteschakelaars → handmatig. Sensoren van onbekend type → automatisch aan, gedimd. Daglichtregeling van onbekend type → schakelend. Lamptype onbekend → toeslag 20 %. Parasitair vermogen forfaitair | p. 182–189 |
| Bevochtiging | Type stoom (elektrisch of niet-elektrisch) of adiabatisch. Terugwinning alleen bij een sorptiewiel | p. 161 |
| PV | Zoals bij woningen | p. 196–197 |
| Energieopslag | Zoals bij woningen | §15.5 |

## Niet ondersteund

- Meer dan één rekenzone, en functiemengsels boven 25 %.
- Koeling via de LBK (DX of watergevoerd), passieve koeling met bypass, en koeling met warmtepompen die ook verwarmen.
- Ontvochtiging en de latente last van adiabatische bevochtiging.
- Meerdere opwekkers per dienst, WKK, zonneboilers, warmtapwater via afleversets en luchtverwarming via de LBK.
- Daglichtsectoren. Een bekende daglichtregeling rekent met de forfaitaire daglichtmethode.

## Interpretatievragen

1. Hoeveel elektrisch aangesloten toestellen heeft een collectieve opwekker (9.91)? De laag neemt er één per opgenomen opwekker.
2. Staat een LBK binnen of buiten de thermische zone als dat onbekend is en de installatie klein is? De laag kiest buiten, als conservatieve keuze; ISSO geeft alleen de regel voor installaties boven 500 m². Hetzelfde geldt voor kanalen waarvan onbekend is of ze buiten de zone lopen (dan situatie volgens tabel 11.14 met de onbekend-waarden).
3. Is "80 % of meer" bij een onbekend minimumdebiet (tabel 11.8) op te vatten als x = 80 in NTA 11.61? De laag doet dat.
4. Wordt de stoomenergie (E_hum) niet met f_BACS vermenigvuldigd? Formule 5.20 past f_BACS alleen toe op E_H en E_C; de laag volgt dat.
5. Moet de forfaitaire circulatie (p. 174–175) bij een onbekende aanwezigheid worden opgenomen? De laag neemt geen circulatie op en geeft de waarschuwing `circulation_unknown_not_entered`.
6. De utiliteitsopname kent geen CW-klasse voor gastoestellen; die gaat altijd naar klasse 4.

## Fixtures

Synthetische opnames in `training-data/` (geen echte gebouwen):

- `nta8800-opname-utility-1985-office.json`: kantoor met kantine, collectieve HR-104, WTW-LBK met recirculatie.
- `nta8800-opname-utility-2005-school.json`: school met mechanische afzuiging, sensoren en PV.
- `nta8800-opname-utility-1970-retail.json`: winkel met veel onbekenden, koeling, stoombevochtiging en een onbekend BACS-vermogen.
