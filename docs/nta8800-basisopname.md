# Basisopname bestaande woningen (ISSO 82.1)

Module: `crates/nta8800-core/src/opname/` (`mod.rs` plus een submodule per ISSO-hoofdstuk)
Route: `POST /v1/nta8800/opname/residential` met `{ "survey": … }`, MCP-tool `assess_residential_survey` en Tauri-command `assess_residential_survey`.
TS-client: `assessResidentialSurveyWithRust` in `src/core/nta/KernelClient.ts`.
Bron: ISSO 82.1, 7e druk (2025), met het erratum van 6 januari 2026 en Wijzigingsdocument 82.1 2025 v1.1. Paginanummers verwijzen naar de pdf; de ISSO-tekst zelf staat niet in de repository.

**Bronvolgorde.** Het wijzigingsdocument v1.1 (15 oktober 2025) wijzigt de 6e druk. Volgens WD p. 6 zijn alle wijzigingen verwerkt in de geconsolideerde 7e druk, door ISSO vastgesteld op 24 september 2025 (printdatum 11 december 2025). Het erratum van 6 januari 2026 corrigeert die 7e druk. Waar de 7e druk en het wijzigingsdocument verschillen, geldt dus de 7e druk met het erratum. Het wijzigingsdocument dient alleen als toelichting. Dit raakt twee punten:

- het fabricagejaar van een ventilator dat onbekend is (tabel 11.15, p. 154);
- de g-waarde van zonwerend glas zonder productgegevens (p. 94).

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
| Thermische massa | Tabellen 7.5/7.6 (erratum §2) worden omgezet naar de klassen van NTA-tabel 7.10. Alleen `lighterCeiling` kiest de eerste kolom van tabel 7.4: een (zeer) zware vloer waarvan de bovenzijde zwaarder is dan het plafond erboven. Het criterium "gesloten of verlaagd plafond" (`closedOrSuspendedCeiling`) komt uit ISSO 75.1 en geeft in de woningopname `closed_ceiling_not_in_dwelling_survey` | p. 62 |
| Opake constructies | R_c volgens bouwjaar en isolatiestaat via NTA-bijlage I (`forfait_envelope`). Een nageïsoleerde spouw met onbekende breedte volgt tabel 8.26 (40/70/100 mm) | p. 84–93 |
| Renovatie of aanbouw | Bij "aanwezig, dikte onbekend" met `renovation`: is het jaar bekend en is er bewijs dat aan de eis van dat jaar is voldaan, dan geldt de jaarklasse van dat jaar. Is het jaar bekend zonder dat bewijs, dan de klasse ervóór, ten hoogste "1992 tot 2014" (R_c 2,5). Is het jaar onbekend, dan de klasse na die van het bouwjaar; vóór 1965 de kolom "(na)geïsoleerd". Zonder `renovation` blijft de jaarklasse van het bouwjaar gelden (§8.7.2, prioriteit 3) | p. 84–85 (§8.7.2.1, afb. 8.14) |
| Scheiding met onverwarmde ruimte | R_se is de R_si van de onverwarmde ruimte bij dezelfde warmtestroomrichting (0,13 / 0,10 / 0,17), niet 0,04 | NTA 8.4.2.1 |
| Sterk geventileerde ruimte | Een garage of andere sterk geventileerde ruimte (`strongly_ventilated`) telt voor het verlies als buitenlucht (zelfde R_c-rij, R_se, f_ls 1, ΔU_for), maar zonder zoninstraling op de delen die eraan grenzen | p. 41 (§6.3.4); WD 2025 p. 22–24; NTA 3.134, 6.3 |
| Beglazing | Gelijkstellingen: dubbel + voorzetraam = HR, enzovoort. U- en g-waarde uit tabel 8.14/8.15. Een raam zonder kozijn telt als hout/kunststof. Kozijnfractie 0,25 (NTA 7.6.6.2, methode B) | p. 93–95 |
| Deuren en panelen | Een deur met minder dan 65 % glas wordt gesplitst in een raam- en een deurdeel. Is niet vast te stellen of de deur geïsoleerd is, dan geldt ongeïsoleerd. Panelen volgen tabel 8.18–8.21 | p. 70, 95–97 |
| Koudebruggen | Forfaitair voor de hele woning: ΔU_for (NTA 8.2/8.3) op alle buitenvlakken; vloeren op grond krijgen 0,5·P | p. 79 |
| Aangrenzende onverwarmde ruimten | H_ue = 5·A (NTA I.8) en b_U = H_ue/(H_ue + H_iu) | NTA I.2.4, 8.4.1 |
| Belemmering | De adviseur kiest per raam een situatie (`shading`). Zonder situatie of opgegeven factoren geldt "minimale belemmering" | 82.1 p. 103 en 75.1 p. 105 (tabel 8.24/8.25), NTA §17.3.2 |
| Vloeren naar buitenlucht | Rij "daken en vloeren grenzend aan de buitenlucht" van tabel 8.9/8.10, R_si 0,17. Een plafond naar een onverwarmde zolder (AOR) is een zoldervloer (`attic_floor`: vloerrij, R_si 0,10) | p. 88–90; NTA I.4 |
| Thermokussens | R_c 1,95, ongeacht het antwoord over isolatie | p. 93; WD p. 37 |
| Rieten daken en gevels | Riet gemeten aan de onderzijde min 35 mm, afgerond op 50 mm. Zonder (of onbekende) isolatie tabel 8.12 (d/0,105). Met isolatie formule 8.5 (d_iso/0,045 + d_riet/0,105), 40 mm bij onbekende dikte | p. 92 (afb. 8.16) |
| Onverwarmde kelder | Het gat in de begane grondvloer is een fictieve ongeïsoleerde vloer (R_c 0,15), omtrek 0,01 m als die ontbreekt, R_bw van de gevel boven de rest van de vloer | p. 72 |
| Kruipruimtewand | R_bw is de R_c van de gevel met de laagste R_c, niet 1/U − 0,17 | p. 93 (tabel 8.13) |
| Leidingdoorvoeren | Onbekend → één ongeïsoleerde leiding per bouwlaag van de woning, elk met N_bouwlaag = aantal bouwlagen (formule 7.17, 1,8 W/K per bouwlaag). Een lege lijst betekent geen doorvoeren. De kern telt H_p mee in H_tr (7.16) | p. 63 (tabel 7.7); NTA 7.3.3 |
| Verwarming | Geen opwekker aanwezig → CR-ketel. Waakvlam onbekend → met waakvlam. Waterstofketel → HR-107. Watergedragen bron onbekend → bodem. Ontwerptemperatuur onbekend → tabel 9.9. Bij meerdere afgiftesystemen gaat oppervlakteverwarming voor. Inregeling onbekend → niet ingeregeld. Regeling onbekend → overig | p. 107–124 |
| Warmtepomp | Bodem/grondwater zonder zonneregeneratie → c_source 1,0 (NTA-tabel 9.27 voetnoot a); met regeneratie de opgegeven bijlage V-factor. Productgegevens volgens tabel 9.5 → rij tabel 9.28 met `highEfficiencyEvidence`. Een afvoerluchtwarmtepomp vraagt een tweede opwekker en wordt afgewezen (`exhaust_air_heat_pump_second_generator_required`) | p. 109–110; WD p. 43 |
| Warmtepompbron en tabel | Een warmtepomppaneel rekent met de rij buitenlucht (NTA p. 336, opmerking 3). Grondwater zonder bekende brontemperatuur rekent met de rij bodem (NTA p. 335). Met `sourceTemperatureC` onder 15 °C geldt de rij grondwater. Een collectieve bron (`collectiveSourceReference`, facturen of ontwerpgegevens) gaat naar `sourceSystem` collectief (9.62, f_cor.bron.col) en `externalSupply.collectiveHeatPumpSource` (9.6.8.1.1.2.3). Bij een collectieve grondwaterbron van het doublettype is c_source 1,04 (NTA-tabel V.3); recirculatie of onbekend geeft 1,00. Een hogetemperatuurbron is altijd collectief en krijgt de klasse van zijn temperatuur (15–20, 20–40 of ≥ 40 °C); bij 20 °C of meer is een kwaliteitsverklaring nodig, anders de rij grondwater. Collectief oppervlaktewater neemt in tabel 9.27 de rij grondwater (WD p. 45). Een collectieve installatie of een warmtepomp boven 25 kW rekent met tabel 9.29 (titel tabel 9.29, p. 337), met hulpenergie volgens 9.91 | p. 109–111 (tabel 9.6); WD p. 43–45 |
| Gasgestookte warmtepomp | `drive`: `gas_engine` of `gas_absorption` (p. 109) geeft `gas_heat_pump` met de GWP-rijen van tabel 9.27 (tot en met 25 kW) of 9.29 (boven 25 kW; altijd bij utiliteit). Het vermogen is verplicht (`gas_heat_pump_capacity_required`; NTA §9.6.3 en 9.91). Afvoerlucht, gecombineerde lucht en een hogetemperatuurbron zijn geen optie (`gas_heat_pump_source_not_allowed`, tabel 9.6 voetnoten 4–6). Een collectieve bron wordt voor gaswarmtepompen niet ondersteund (`gas_heat_pump_collective_source_unsupported`) | p. 109–111 |
| Olieketel | `boilerType: oil`: conventionele ketel op olie (definitie onder NTA-tabel 9.25), zonder waakvlam | p. 107–108 (tabel 9.3) |
| Lokale verwarming en stoomketel | `local_fired`: lokale gasverwarming inclusief waakvlam, olieverwarming of een stoomketel (brandstof verplicht). Met afvoer 0,65, zonder afvoer 0,10 (NTA-tabel 9.25, overige systemen). Elektriciteitsaansluiting onbekend → aanwezig (10 W, 9.6.8.2.3) | p. 108 (tabel 9.3) |
| Gasgestookte luchtverwarmer | `gas_air_heater`: conventioneel, VR, HR-100/104/107 (0,75–0,95, NTA-tabel 9.25). Waakvlam onbekend → aanwezig (695 kWh per toestel, §9.6.2.1). Aantal onbekend → het aantal uit tabel 9.16, anders 1 | p. 108 (tabel 9.3) |
| Biomassa | Bijlage R onbekend → niet conform | p. 111–112, p. 28 |
| Distributie bij externe warmte, elektrisch of biomassa met watergedragen afgifte | Pomp forfaitair. Geen warmtemeter: tabel 9.16a geldt alleen voor collectieve installaties | p. 117–122 |
| Leidingisolatie | `heating.pipeInsulation`: geïsoleerd ja/nee. Isolatiejaar onbekend → bouwjaar; dat geeft de periode van NTA-tabel 9.16 (vanaf 1995, 1980–1995, vóór 1980). Appendages en beugels onbekend → niet geïsoleerd (9.27a/9.27b). Zonder antwoord → niet geïsoleerd | p. 118 (tabel 9.12) |
| Eenpijpssysteem | `heating.distributionType`: tweepijps (ook Tichelmann), eenpijps met het aantal afgiftetoestellen, of gerenoveerd eenpijps. Bij een berekende pomp telt een eenpijpskring de weerstand van tabel 9.21 per afgiftetoestel (`onePipeEmitterCount`, NTA p. 317). Een gerenoveerd eenpijpssysteem rekent als tweepijps. Zonder antwoord → tweepijps | p. 115; WD p. 50 |
| Leidingen in onverwarmde ruimten | Bevat het gebouw een kruipruimte, kelder of andere onverwarmde ruimte en is niet vastgesteld dat er geen cv-leidingen lopen, dan zijn ze aanwezig met de forfaitaire lengte (15 % van L, 9.26). De distributie gaat dan naar de berekende route (9.26–9.40) | p. 120 (afb. 9.1), p. 121 |
| Tapwater | Geen systeem → elektrisch doorstroomtoestel. Gastoestel onbekend → badgeiser. Gaskeur onbekend → geen. CW-klasse (`cwClass`, alleen met Gaskeur): aanrecht/CW-1 → klasse 1, CW-2, CW-3, CW-4/5/6 of onbekend → klasse 4 (`measuredClass`). Keukengeiser boven 13 kW → badgeiser. DWTW onbekend → niet aangesloten | p. 164–180 (tabel 13.6) |
| Elektrische boiler | Vat via `boilerVessel`: volume verplicht, behalve een keukenkastboiler (10 l). Label onbekend → fabricagejaar; jaar onbekend → bouwjaar; plaats onbekend → buiten de zone; aansluitfactor 2 | p. 172–174 (tabel 13.10) |
| Luchtverwarming | `heating.airHeating` bij afgifte `air_heating` (tabel 9.16): direct (radiale ventilator bij onbekend), indirect (bij onbekend wisselstroom, hoger dan 8 m en zonder terugkeer van warme lucht) of via de LBK. Direct of indirect gaat naar `emission.airHeaters` (NTA-tabellen 9.12/9.13). Via de LBK geeft geen ventilatoren bij de afgifte. Type onbekend → niet van toepassing (`air_heating_type_unknown`). `airHeating` bij een andere afgifte → `air_heating_requires_air_heating_emitters` | p. 123 (tabel 9.16) |
| Passieve koeling | `ventilation.passiveCooling` alleen met een projectdocument van de leverancier (`evidenceReference`) dat een automatische sturing op gemeten binnen- én buitentemperatuur aantoont. Dit gaat naar `maximumCapacityForCooling` (τ_sysC) en, met `installedCapacityDm3PerS` uit het inregelrapport, naar `installedCapacity`. Is de capaciteit onbekend, dan geldt het eisdebiet. Bij systeem A volgt `passive_cooling_requires_mechanical_ventilation`; bij een WTW zonder aangetoonde bypass `passive_cooling_requires_bypass` | p. 146, 151–152 (§11.4.1, §11.5.4, §11.5.6) |
| Ventilatie | Zelfregelende roosters (tabel 11.3/11.5). Sturing onbekend → geen. WTW onbekend → geen. Tegenstroom met onbekend materiaal → aluminium. Toevoerkanaal en bypass volgens tabel 11.10–11.12. Kanaaldichtheid onbekend → 1,1. Ventilatormotor onbekend → wisselstroom (tot en met 2006) of gelijkstroom (vanaf 2007). Fabricagejaar ventilator onbekend → bouwjaar (tabel 11.15 gaat voor de algemene regel van p. 28) | p. 142–154 |
| PV | Kristallijn type bekend, jaar onbekend → bouwjaar (vóór 2001 telt als 2000). Type onbekend → polykristallijn met het installatiejaar, of "geplaatst vóór 2001" als ook dat onbekend is. Montage onbekend → niet geventileerd | p. 191 (tabel 15.7) |
| PV-beschaduwing | `pv[].shading` neemt de situaties van §15.4.7 over met de collectortabellen van NTA §17.3 (17.6/17.12/17.15): minimaal, zijbelemmering(en), dakranden (alleen platte daken), volledig of overig, of factoren uit de uitgebreide methode (17.3.8). Niets opgegeven → minimale belemmering (tabel 16.1, niet aanwezig), vastgelegd als toegepaste standaardwaarde. `shading` en `obstructionFactors` tegelijk → `pv_shading_declared_twice` | p. 192–193, 195–196 (tabel 16.1) |
| Energieopslag | Gebouwgebonden elektrische en thermische opslag (kWh), alleen met PV | p. 193 (§15.5) |
| Serre (AOS) | Grensvlak `sunroom`: in de basisopname telt een aangrenzende onverwarmde serre als buitenlucht, met zontoetreding. Vastgelegd als `sunroom_as_outdoor` | §6.3.4 p. 41 |
| Woonwagen en woonboot | `envelope.buildingKind`: `caravan` of `floating` (bestaande ligplaats van vóór 2018, of `newBerthSince2018`). De schil volgt dan de forfaits van NTA-tabellen I.5–I.7. Het grensvlak `water` (alleen een vloer van een woonboot, anders `water_boundary_requires_houseboat_floor`) geeft de romp (`floating_hull`), naar buitenlucht. Vloer en wand tellen voor de massa als licht (`houseboat_caravan_light_mass`) | p. 49, 61 |
| Zonwerend glas en zonwerende folie | `windows[].solarControl` met de g-waarde uit de productinformatie of de gecontroleerde kwaliteitsverklaring. Zonder die gegevens geldt de g-kolom van tabel 8.14. De kolom met g 0,4 uit het wijzigingsdocument (WD p. 38) staat niet in de 7e druk en wordt niet toegepast. Een lege bron geeft `solar_control_evidence_required` | p. 94 |
| Woningpositie in een woongebouw | `floor`: `ground_or_intermediate` (posities 1, 2, 5, 6), `top` (3, 7) of `roof_and_floor` (4, 8). NTA-tabel 11.14 heeft geen rij voor dak + vloer; het dak maakt het een bovenwoning (`apartment_roof_and_floor_top_type`) | p. 51 |
| Sturing ventilatie | `ventilation.controls` met CO₂-meting, CO₂-sturing, tijdsturing, zonering en (systeem C) afvoerpunten per verblijfsruimte, met een bewijsstuk. De combinatie wordt omgezet naar een woonrij van NTA-tabel 11.5 (B.1–B.3, C.1–C.5b, D.1–D.5c). Alleen combinaties die een rij van tabel 11.5 zijn, leveren die variant op; andere vallen terug op de variant zonder sturing. `declaredVariant` gaat voor. Centrale of decentrale WTW via `heatRecoveryLayout` | p. 143–145 (tabellen 11.4–11.6) |
| Gecombineerd systeem E | `ventilation.combined` met de verblijfsoppervlakte van het decentrale deel en de totale verblijfsoppervlakte. Het decentrale deel is D.5b met de WTW uit `heatRecovery`; `principle` beschrijft het andere deel (natuurlijk, mechanische toevoer of afvoer; balans geeft `combined_other_part_not_balanced`) | p. 145 (§11.3.6), NTA tabel 11.5 E.1 |
| Roosters met verwarmingslint | `ventilation.grilleHeatingStrips` met de vier regelgegevens; ontbreekt er een, dan geldt de terugvalwaarde 11.124. Is het aandeel onbekend, dan hebben alle roosters een lint | p. 145–146 (§11.3.7) |
| Bypass bij onbekende gegevens | Het fabricagejaar van de WTW-unit bepaalt de keuze tussen 100 %, 70 % en 0 %. Alleen als dat jaar onbekend is, telt het bouwjaar. De opname geeft de fractie expliciet door, omdat NTA 11.3.2.2 "bouw- of fabricagejaar" noemt | p. 151–152 (tabel 11.12) |
| Warmtepomp boven 70 °C | Een opgegeven klasse boven 70 °C (90/70) bij een warmtepomp vraagt `heating.heatPumpAbove70Declaration`, anders `heat_pump_above_70_requires_declaration` | p. 114 (tabel 9.9, erratum §4) |
| Lichtkoepels en daklichten | Met een gecontroleerde kwaliteitsverklaring (BCRG): `envelope.rooflights` met A_rc en U_rc uit de verklaring en de beglazing voor g, als raam in een dak (`rooflight_from_quality_declaration`). Zonder verklaring voer je ze in als raam of paneel; een lege verwijzing geeft `rooflight_quality_declaration_required`, een niet-dakvlak `rooflight_requires_roof` | p. 68 |

## Bekende bronfouten

- ISSO-tabel 7.4 drukt 350 af voor "zeer zware vloer, zware wand". NTA-tabel 7.10 en het wijzigingsdocument geven 450. De kern rekent met NTA-tabel 7.10, dus met 450.
- De R_c van thermokussens is 1,95 (WD p. 37) in plaats van 1,80. De opname rekent altijd met 1,95, ook bij een opgegeven dikte.
- λ riet is 0,105 (WD p. 36) in plaats van 0,2.

## Niet ondersteund

- Kwaliteitsverklaringen anders dan een gemeten q_v10 en lichtkoepels/daklichten (BCRG).
- Het nominale vermogen van een tapwaterwarmtepomp (§13.3.2.5): de kern toetst de capaciteit van tapwatertoestellen (13.8.2) nog niet.
- Een afvoerluchtwarmtepomp zonder tweede opwekker (WD p. 43): `exhaust_air_heat_pump_second_generator_required`. Met `additionalGenerators` wordt hij aanvaard.
- Detailopname-routes.

## Interpretatievragen

1. Een appartement met een onbekende positie aan de zijkant is als kop- of hoekligging ingedeeld. P. 51 vraagt een onderbouwde keuze en geeft geen invoer bij onbekend; de laag kiest conservatief (p. 28).
2. Een deur waarvan de isolatie niet vast te stellen is, telt als ongeïsoleerd. ISSO noemt dat niet expliciet; dit volgt uit de conservatieve regel op p. 28.
3. ISSO-klasse 70/50 heeft geen rij in NTA-tabel 9.14. De distributie gebruikt 70/60 (gelijke aanvoertemperatuur), en de ketel nu ook de gemiddelde temperatuur van die klasse (65 °C), zodat beide hetzelfde circuit beschrijven. De ketelforfait leest de temperatuur alleen tegen de grens van 50 °C, dus de uitkomst verandert niet.
4. ΔU_for (8.3) wordt niet toegepast op scheidingen met onverwarmde ruimten, omdat H_D;for alleen de buitenlucht betreft.
5. Een combitoestel met Gaskeur CW zonder HR telt als "combi met Gaskeur" (0,50). Tabel 13.25 kent alleen de combinatie HR + CW als hogere rij.
6. Formule 8.5 voor riet heeft geen R_ad en rondt half naar boven af op 50 mm. NTA I.2.1.4/I.3 telt R_ad wel mee en rondt naar beneden af. De opname volgt ISSO.
7. Leidingdoorvoeren (tabel 7.7): het aantal leidingen is het aantal bouwlagen, en elke leiding telt volgens 7.17 met alle bouwlagen van de zone. Een woning van twee bouwlagen krijgt dus 2 × 2 × 1,8 = 7,2 W/K.
8. Een nageïsoleerde spouw van vóór 1930 met onbekende breedte krijgt 40 mm. Tabel 8.26 begint bij 1930.
9. Een deels plat dak bij een niet-vrijstaande woning telt als hellend dak; ISSO kent deels plat alleen bij vrijstaande woningen.
10. Renovatie zonder bewijs volgt afb. 8.14 letterlijk, ook als de klasse vóór het renovatiejaar lager is dan die van het bouwjaar (bijvoorbeeld bij een aanbouw). "Aanwezig, dikte onbekend" zonder `renovation` geldt vanaf 1965 als oorspronkelijke isolatie (jaarklasse van het bouwjaar).
11. Een raam in een wand naar een sterk geventileerde ruimte telt in H_D als buitenraam (U voor buiten), zonder zonwinst.
12. Ventilatorfabricagejaar onbekend (tabel 11.15, p. 154): de 7e druk geeft het bouwjaar, het wijzigingsdocument v1.1 (WD p. 65) "onbekend". Volgens de bronvolgorde geldt de 7e druk, dus het bouwjaar (`fan_year_unknown_construction_year`).
13. Romp van een woonboot met bouwjaar 2014: ISSO-tabel 8.10 (p. 89) geeft R_c 3,5 voor 2014–2018, NTA-tabel I.7 (p. 831) 2,5 voor 2014–2015. De forfaitaire R_c is een rekenwaarde van de NTA; ISSO neemt die tabel over. De kern volgt NTA-tabel I.7. De ISSO-afwijking is gemeld als vermoedelijke overnamefout.
14. Een appartement met dak en vloer (posities 4 en 8) rekent voor de luchtdoorlatendheid als bovenwoning, omdat NTA-tabel 11.14 geen rij voor dak + vloer heeft.
15. Energieopslag in een woongebouw (erratum §6): alle opslag achter de hoofdmeter telt mee. De woningopname neemt de opgegeven capaciteit over. De adviseur geeft het totaal achter de hoofdmeter op, inclusief de opslag van andere opgenomen appartementen.
16. De GWP-rijen van tabel 9.27 (p. 334) noemen alleen een collectieve gebouwinstallatie, terwijl de titel van tabel 9.29 collectieve installaties ook omvat. Een gaswarmtepomp tot en met 25 kW in een collectieve installatie rekent met tabel 9.27; boven 25 kW rekent hij met tabel 9.29. Een individuele gaswarmtepomp tot en met 25 kW past in geen van beide tabellen en geeft `gas_heat_pump_individual_no_forfait_row`. Tabel 9.27 heeft geen GWP-rij oppervlaktewater, dus oppervlaktewater neemt daar de rij grondwater. Elektrische warmtepompen in een collectieve installatie rekenen met tabel 9.29.
17. Een gerenoveerd eenpijpssysteem (WD p. 50) heeft in NTA 8800 geen eigen regel. De opname rekent het als tweepijpssysteem.
18. Een tweepijpssysteem is aangenomen als het distributietype niet is opgegeven. ISSO geeft geen invoer bij onbekend; alleen de pompweerstand van een berekende pomp hangt ervan af.
19. Zonder P_H;gen (tabel 9.3 vraagt het niet) rekent 9.92 voor lokale verwarming en luchtverwarmers met een brander die de hele maand draait. Dat is de bovengrens die de begrenzing t_on ≤ t_mi toelaat.
20. Een collectieve bron van onbekende temperatuur valt voor de primaire factor in de klasse "≥ 20 °C, oppervlaktewater of onbekend" (9.6.8.1.1.2.3 b); voor de COP neemt hij de rij bodem (p. 335).
21. Bypass (tabel 11.12, p. 151–152): de jaarregel geldt alleen als de bypass of het percentage onbekend is. Een bypass die bij de opname afwezig is, telt als 0 %.
22. Systeem E (§11.3.6, p. 145) heeft in het decentrale deel altijd WTW, dus een onbekende wisselaar kan hier niet "geen WTW" worden (`combined_requires_heat_recovery`). Passieve koeling is ook mogelijk als het andere deel natuurlijk is (p. 152).

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

**Eén rekenzone, eventueel gemengd.** Het gebouw is één rekenzone met de grootste gebruiksfunctie als hoofdfunctie. De overige functies worden, de kleinste eerst, samengevoegd met de hoofdfunctie zolang ze samen niet meer dan 25 % van A_g beslaan (p. 39–40; `small_functions_merged_into_main`). Wat overblijft, blijft als aparte functie in een gemengde rekenzone (NTA §6.5.3; `larger_functions_kept_separate`): de laag geeft `functionAreas` door, met oppervlaktegewogen setpoints (`FunctionProfile::weighted`), ventilatie per functie, verlichting per functie en `labelFunctions` voor het label.

**Bevochtiging.** Bevochtiging gaat als `humidifiers`-post (zone `utiliteit`) mee in de verwarmingsketen: stoom (elektrisch of met een brandstof) en adiabatisch, met terugwinning bij een sorptiewiel. De latente last en de stoomenergie volgen dan uit hoofdstuk 12 in de kern; de tweede kernrun en de waarschuwing voor adiabatische bevochtiging zijn vervallen. Ontvochtiging (tabel 12.2) is nog niet gekoppeld.

**Meerdere tapwatersystemen.** `additionalHotWaterSystems` bevat verdere tapwatersystemen, elk met eigen opwekker(s), voorraadvaten en `servedAreas` (functie en oppervlakte). Het hoofdsysteem bedient de rest van het gebouw (13.20/13.20a). Een samengevoegde kleine functie telt als de hoofdfunctie. Een extra systeem zonder bediende oppervlakte geeft `hot_water_served_areas_required`; meer oppervlakte dan de functie heeft geeft `hot_water_served_areas_exceed_function`. `servedAreas` op het hoofdsysteem wordt genegeerd (`hot_water_main_served_areas_ignored`).

**LBK-batterijen.** `ventilation.ahu.heatingConnected` en `coolingConnected` worden de naverwarmer en koelbatterij van NTA-tabel 11.15 (`heatingCoil`, `coolingCoil`). Niet vast te stellen → niet aangesloten (`ahu_heating_unknown_none`, `ahu_cooling_unknown_none`; interpretatie van p. 149). Een koelbatterij zonder koelsysteem geeft `ahu_cooling_requires_cooling_system`.

## Geïmplementeerde regels

| Onderdeel | Regel | ISSO 75.1 |
|---|---|---|
| Functies | Andere functies tot samen 25 % van A_g worden, de kleinste eerst, samengevoegd met de hoofdfunctie; grotere blijven apart in een gemengde rekenzone (NTA §6.5.3). Een woonfunctie hoort in de woningopname | p. 39–40 |
| Gebouwtype | Eén- of meerlaags, ligging en daktype bepalen de rij van NTA-tabel 11.14. "Deels plat" geldt alleen bij vrijstaande gebouwen | p. 55–56 |
| Toestel- en renovatiejaar | Zoals bij woningen | p. 30, 57–59 |
| Zonwerende beglazing | Zichtbaar zonwerend glas of folie → g = 0,4 | p. 96 (tabel 8.14) |
| BACS | Zonder `bacs.systemPowerKw` beslissen de opgenomen vermogens: het grootste verwarmings- of koelsysteem, met de opwekkers van één systeem opgeteld en systemen niet bij elkaar (p. 63). Verwarming: `heatingInstallation.capacityKw`, anders de som van `heating.nominalPowerKw` en de extra opwekkers. Koeling: de som van `cooling.capacityKw` en de extra koudeopwekkers (`bacs_power_from_surveyed_systems`). Is een vermogen onbekend, dan geldt een systeem dat meer dan 2.500 m² bedient als systeem boven 290 kW. Is ook het bediende oppervlak onbekend, dan telt de A_g van het gebouw. BACS onbekend → afwezig. Automatisering onbekend → klasse D. Beheer onbekend → klasse C/D. Een systeem boven 290 kW zonder conform BACS krijgt f_BACS = 1,05 | p. 62–63 (tabel 7.3) |
| Grote installatie | Bedient een installatie meer dan 500 m², dan staat de technische ruimte (ketel, LBK) per definitie buiten de thermische zone | p. 17, 117 |
| Collectieve verwarming | Ketelrol collectief, met het nominale vermogen voor de hulpenergie (9.91). Warmtepomp in de utiliteitsscope van tabel 9.27 (zonder c_source en zonder rij 9.28). Berekende distributie (9.26–9.51) met forfaitaire pomp en warmtemeter aanwezig (tabel 9.16, alleen collectief). Leidingisolatie volgens tabel 9.12 (onbekend: niet geïsoleerd, isolatiejaar onbekend: bouwjaar, appendages onbekend: niet geïsoleerd) en een eenpijpssysteem met het aantal afgiftetoestellen (§9.4.2), zoals in de woningopname | p. 108–123 (tabel 9.12 p. 120, §9.4.2 p. 117) |
| Leidingdoorvoeren | Onbekend → één ongeïsoleerde leiding per toiletgroep (`toiletStacks`) door alle bouwlagen. Zonder opgave van toiletgroepen rekent de laag met één groep | p. 68 (tabel 7.8) |
| Leidingen in onverwarmde ruimten | Zoals bij woningen (afb. 9.1): met een kruipruimte, kelder of onverwarmde ruimte en zonder vastgestelde afwezigheid zijn de leidingen aanwezig met de forfaitaire lengte (15 % van L). Zijn ze afwezig, of is er geen onverwarmde ruimte, dan geeft de laag 0 m door; anders zou de berekende distributie 15 % van L in onverwarmde ruimte aannemen | p. 121–122 (afb. 9.1, tabel 9.14) |
| Renovatie, R_se naar onverwarmde ruimten, sterk geventileerde ruimten, thermische massa | Zoals bij woningen | p. 66 (tabel 7.5), 88–89 |
| Koeling | Vermogen onbekend → forfait. Ontwerptemperatuur onbekend → 6/12, bij alleen stralingskoeling 17/21. Inregeling onbekend → niet ingeregeld; statisch én dynamisch ingeregeld geven f_HB 1,0 (NTA-tabel 10.11). Gasmotor-koelmachine (`gas_engine_compression`, tabel 10.2): fabricagejaar onbekend → tot en met 2006, elektrisch vermogen verplicht (`gas_engine_power_required`). Extra koudeopwekkers op dezelfde distributie (`additionalGenerators`, §10.3.2) krijgen de prioriteit van NTA-tabel 10.15 en elk een vermogen (`cooling_generator_capacity_required`). Directe expansie in de ruimte of in de LBK (§10.4.1); in de LBK vraagt een LBK met koelbatterij. Appendages onbekend → niet geïsoleerd (tabel 10.9). Werkelijke leidinglengte en lengte door niet-gekoelde ruimten (tabel 10.10), anders forfait. Leidingen onbekend → ongeïsoleerd, isolatiejaar = bouwjaar. Koudemeters onbekend → aanwezig. Betonkernactivering telt als vloerkoeling. Binnendelen van split/VRF tellen als ventilatorconvectoren. Regeling onbekend → overig. WKO met onbekend jaar → vergunningsjaar, anders vóór 2013 | p. 131–138 |
| Ventilatie | Zelfregelende roosters en regeling zoals bij woningen. WTW volgens tabel 11.9, ook "koude laden met LBK" (NTA-tabel 11.18, 0,40). Centraal of decentraal (tabel 11.6). Buitenaansluiting: isolatie onbekend → niet geïsoleerd, lengte onbekend → kerndefault naar systeemtype (tabel 11.10). Constant volumeregeling onbekend → geen (tabel 11.11). Een gedeeltelijke bypass wordt naar beneden afgerond op tientallen; onbekend → de kernwaarde "onbekend" (tabel 11.12). Kanaaldichtheid LUKA A–C, LUKA D, geen kanaal of onbekend (tabel 11.13). Geïnstalleerde capaciteit (§11.4.1) voor B–E; bij een zwembad in de zone telt het debiet als onbekend (p. 148). Gecombineerd systeem E (§11.3.6) en roosters met verwarmingslint (§11.3.7) zoals bij woningen | p. 142–153 |
| Recirculatie | Aanwezig met onbekend percentage → de NTA-waarde x = 20 (11.60). Onbekend of aanwezig → geen. Een aangetoond hoger percentage wordt naar beneden afgerond op tientallen; lager dan 20 telt niet | p. 148 (tabel 11.7); NTA 11.60 |
| Debietregeling | Onbekend → geen. Minimumdebiet onbekend → 80 %. Een aangetoond lager percentage wordt naar boven afgerond; hoger dan 80 telt niet | p. 149 (tabel 11.8); NTA 11.61 |
| LBK-kanalen buiten de zone | Lengte onbekend → ≥ 40 m. Isolatie onbekend → R < 1,0. Situatie volgens NTA-tabel 11.19 | p. 153 (tabel 11.14) |
| Ventilatoren | Forfait. Motor onbekend → wisselstroom (tot en met 2006) of gelijkstroom (vanaf 2007). Fabricagejaar onbekend → bouwjaar | p. 154 (tabel 11.15) |
| Tapwater | Geen systeem → elektrisch doorstroomtoestel. Gastoestellen zoals bij woningen. Collectieve opwekker onbekend → overige direct verwarmde voorraadvaten. Gasboiler: jaar onbekend → bouwjaar, plaats onbekend → buiten de zone. Tappuntlengte onbekend → > 3 m. Circulatie forfaitair. DWTW alleen bij functies met douches | p. 164–178 |
| Voorraadvaten | Verplicht bij elektrische en indirect gestookte boilers. Label onbekend → volgt het fabricagejaar. Fabricagejaar onbekend → tot en met 2017. Aansluiting onbekend → ongeïsoleerd (f_sto;dis;ls = 5; elektroboilers 2). Plaats onbekend → buiten de zone | p. 172–173 (tabel 13.10) |
| Verlichting | Vermogen onbekend → forfait tabel 14.5. Bij forfait geldt centraal aan en geen daglichtregeling. Het forfait geldt dan voor alle verlichtingszones (NTA 14.3.4). De LED-waarde vraagt LED vanaf 2017 in alle zones: gemeten zones tellen alleen mee als al hun lampen LED zijn, een armaturenlijst telt als niet-LED. Alleen ruimteschakelaars → handmatig. Sensoren van onbekend type → automatisch aan, gedimd. Daglichtregeling van onbekend type → schakelend. Lamptype onbekend → toeslag 20 %. Parasitair vermogen forfaitair | p. 182–189 |
| Bevochtiging | Type stoom (elektrisch of niet-elektrisch) of adiabatisch, als bevochtiger in de verwarmingsketen. Terugwinning alleen bij een sorptiewiel | p. 161 |
| LBK-batterijen | Verwarming of koeling aangesloten op de LBK → naverwarmer of koelbatterij (tabel 11.15). Onbekend → niet aangesloten, behalve bij luchtverwarming via de LBK (`airHeating` `via_air_handling_unit`): dan is de naverwarmer aanwezig (`ahu_heating_from_air_heating`). "Niet aangesloten" naast luchtverwarming via de LBK geeft `air_heating_via_ahu_requires_heating_coil` | p. 149; p. 123 (tabel 9.16) |
| Luchtverwarming en passieve koeling | Zoals bij woningen | p. 123, 146, 151–152 |
| Ontvochtiging | Hoofdstuk 12 van 75.1 (p. 161) vraagt alleen naar bevochtiging. De ontvochtigingsbehoefte (NTA 12.5, tabel 12.2) volgt in de kern uit de ontwerptemperatuur van de koeling; de opname vraagt er niets extra voor | p. 161; NTA 12.5 |
| Warmtepomp die ook koelt | Compressiekoeling (`compression`, of `room_air_conditioner` bij directe expansie in de ruimte). Voor de verwarming is het een elektrische warmtepomp (§10.3.1.1). Vrije koeling op de bron van de verwarmingswarmtepomp gaat via `closed_ground_loop`/aquifer met `heatPumpSource` (NTA 10.84) | p. 126–129 |
| Rekenzones | Na het samenvoegen van p. 39–40 toetst de laag afb. 6.6 met tabel 6.4: setpoints meer dan 4 K uit elkaar (tenzij de grootste functie ≥ 90 % beslaat), of bij ventilatietype A, B, C of E een ventilatiecapaciteit die meer dan een factor 4 verschilt (tenzij de verblijfsgebieden in open verbinding staan, `openlyConnectedResidenceAreas`, of meer dan 80 % dezelfde eis heeft). Zonder `zones` stopt de opname dan met `calculation_zone_split_required`. Met `zones` (twee of meer) doorloopt elke rekenzone het schema opnieuw (p. 52); een zone die zelf nog gesplitst moet worden geeft `calculation_zone_criteria_not_met` | p. 52–54 (§6.5, afb. 6.6, tabel 6.4) |
| Fossiele brandstof op het perceel | `fossilFuelOnPlot` wordt `fossilAppliancesOutsideCalculation` (NTA §5.5.7) voor "lokaal koolstofemissievrij". Niet vastgesteld → alleen de berekende energiedragers beslissen | p. 61–62 (§7.1.7) |
| Sport- en zwemzalen | Bij een sportfunctie en A_g ≥ 1.000 m² gaat `sportHallAreaM2` naar A_g;si;sport van de circulatie (NTA 13.32a) van het tapwatersysteem dat de sportfunctie bedient. Onbekend → 0 m² (`sport_hall_area_unknown_0`). Een ruimte met zwembad (`swimmingPoolAreaM2`) wordt van de sportfunctie afgesplitst met de factor 2 van NTA §11.2.2.5.1 | p. 65 |
| PV | Zoals bij woningen | p. 196–197 |
| Energieopslag | Zoals bij woningen | §15.5 |

## Niet ondersteund

- Ventilatie per rekenzone: één ventilatiesysteem (het gebouwsysteem) bedient alle rekenzones; een gebouw met fysiek gescheiden ventilatiesystemen is per klimatiseringszone apart op te nemen (§6.4).
- Het criterium voor de specifieke interne warmtecapaciteit (afb. 6.6): de opname kent één constructie per gebouw.
- Passieve koeling, warmtepompen die ook koelen, ontvochtiging en luchtverwarming (ook via de LBK) worden nu ondersteund; zie de tabel hierboven. Hetzelfde geldt voor WKK, meerdere verwarmingsopwekkers, meerdere tapwatersystemen en zonneboilers.
- Daglichtsectoren. Een bekende daglichtregeling rekent met de forfaitaire daglichtmethode.

## Interpretatievragen

1. Hoeveel elektrisch aangesloten toestellen heeft een collectieve opwekker (9.91)? De laag neemt er één per opgenomen opwekker.
2. Staat een LBK binnen of buiten de thermische zone als dat onbekend is en de installatie klein is? De laag kiest buiten, als conservatieve keuze; ISSO geeft alleen de regel voor installaties boven 500 m². Hetzelfde geldt voor kanalen waarvan onbekend is of ze buiten de zone lopen (dan situatie volgens tabel 11.14 met de onbekend-waarden).
3. Is "80 % of meer" bij een onbekend minimumdebiet (tabel 11.8) op te vatten als x = 80 in NTA 11.61? De laag doet dat.
4. Wordt de stoomenergie (E_hum) niet met f_BACS vermenigvuldigd? Formule 5.20 past f_BACS alleen toe op E_H en E_C; de laag volgt dat.
5. Moet de forfaitaire circulatie (p. 174–175) bij een onbekende aanwezigheid worden opgenomen? De laag neemt geen circulatie op en geeft de waarschuwing `circulation_unknown_not_entered`.
6. De utiliteitsopname kent geen CW-klasse voor gastoestellen; die gaat altijd naar klasse 4.
7. Afb. 6.6 toetst op de functies ná het samenvoegen van p. 39–40 (p. 54: alleen de overgebleven functies). Daardoor kan de 90 %-uitzondering van het setpointcriterium in de basisopname niet meer optreden. Het criterium voor de interne warmtecapaciteit (factor 3) treedt niet op, omdat de opname één constructie per gebouw vastlegt. Voor de 80 %-regel van de ventilatiecapaciteit gebruikt de laag de A_g per functie als maat voor de verblijfsgebieden.
8. Is de sportfunctie samengevoegd met de hoofdfunctie, dan vervalt de zwembadfactor (`swimming_pool_in_merged_sport_function`). Het oppervlak van de sport- en zwemzalen telt dan nog wel in 13.32a voor het systeem dat de hoofdfunctie bedient.
9. Directe expansie in de LBK wordt in de koeling zonder distributie doorgegeven; de koude gaat via de koelbatterij van de LBK (NTA 11.116). Onbekend of de koeling op de LBK is aangesloten → aangesloten (`ahu_cooling_from_direct_expansion`).
10. Passieve koeling met ventilatoren staat in 75.1 in §11.5.6 (p. 152); §11.8 [DETAIL] (p. 154) gaat over natuurlijke ventilatieve koeling. De citaten volgen dat.
11. NTA-tabel 10.11 voetnoot a (p. 388): waterzijdig inregelen telt alleen met een verklaring volgens NEN-EN 14336. Statisch of dynamisch ingeregeld zonder `balancingEvidenceReference` rekent als niet ingeregeld (`cooling_balancing_without_declaration_none`). Dit geldt voor de woning- en de utiliteitsopname.
12. Tabel 7.3 (p. 63): het vermogen van een verwarmingssysteem is de som van de opwekkers van dat systeem. Met extra opwekkers telt de laag `heating.nominalPowerKw` (anders de installatiecapaciteit) plus de extra opwekkers. Bij één opwekker telt de installatiecapaciteit, anders het nominale vermogen.
13. Directe expansie in de LBK (p. 130, §10.4.1 p. 133): de koude bereikt de ruimten via de ventilatielucht. Er zijn dan geen afgiftetoestellen in de ruimte: de laag zet de afgifte op `other_or_unknown` zonder ventilatorconvectoren (`cooling_dx_ahu_no_room_emitters`). Een verborgen antwoord over directe expansie bij een watergevoerd systeem wordt genegeerd (`cooling_direct_expansion_ignored_water_based`).
14. De geïnstalleerde ventilatiecapaciteit (§11.4.1) wordt één keer opgegeven: bij ventilatie of bij passieve koeling (`installed_capacity_given_twice`). Met een zwembad in de zone telt hij als onbekend (p. 148), ook als hij uit de passieve koeling komt.
15. Een opgenomen bypasspercentage (tabel 11.12) groter dan 0 na afronding op tientallen telt voor passieve koeling als aanwezige bypass.
16. 13.32a (p. 65, "sport- en/of zwemzalen"): `sportHallAreaM2` is het oppervlak van de sportzalen zonder de zwembadruimte; `swimmingPoolAreaM2` wordt erbij opgeteld. Samen mogen ze niet groter zijn dan de sportfunctie.
17. Tabel 10.10 vraagt L en L_max. Elk onbekend gegeven krijgt zijn eigen forfaitaire waarde van 10.27 (§10.4.2.3, p. 386); het deel door niet-gekoelde ruimten is dan 15 %. De kern neemt een opgenomen L_max over (`maxPipeLengthM` van de pomp).
18. NTA-tabel 9.31 heeft geen waarden voor een gasmotor tot en met 2 kW uit 2006 of eerder. Bij een gasmotorkoelmachine meldt de opname `gas_engine_small_old_no_table_row` in plaats van een fout in de kern.
19. Inklapredenen van opgeslagen opnames met een hernoemd pad of een gesplitste regel blijven gekoppeld via een aliastabel (`COLLAPSE_REASON_ALIASES`): `cooling.distribution`, `cooling_fittings_unknown_uninsulated_meters_present`, `ventilation.ductsLukaAbc`, `ventilation.bypass` en de kanaal- en constant-volumeregels onder `ventilation.heatRecovery`.

## Meerdere rekenzones in de utiliteitsopname

Afb. 6.6 met tabel 6.4 (p. 52–54) kan een splitsing in rekenzones vragen; ook zonder die eis mag de adviseur splitsen (p. 54). De opname neemt dan een lijst `zones` aan.

1. **Zones.** Elke zone heeft een `id` en de gebruiksfuncties met hun A_g. Per functie tellen de zones op tot `functions` van het gebouw (tolerantie 0,05 m², anders `zone_function_areas_mismatch`). Zonder `zones`, of met één zone (waarschuwing `single_calculation_zone_whole_building`), is het gebouw één rekenzone zoals voorheen.
2. **Functies per zone (§6.6, p. 54).** Het samenvoegen van p. 39–40 gebeurt op gebouwniveau; in een zone komen alleen de functies voor die daarna over zijn. Een samengevoegde kleine functie telt in haar zone als hoofdfunctie. Setpoints en ventilatie-eisen volgen per zone met de oppervlaktegewogen waarden van §6.5.3.
3. **Afb. 6.6 per zone.** Elke zone doorloopt het schema opnieuw (p. 52: "voor elke rekenzone ... volledig doorlopen"). Moet een zone verder gesplitst worden, dan geeft de opname `calculation_zone_criteria_not_met` op `zones[i].functions`. `calculation_zone_split_required` komt alleen nog bij een opname zonder zones.
4. **Schil.** Een vlak in `envelope.surfaces` krijgt een optionele `zoneId`; ramen, deuren, panelen en daklichten volgen hun vlak. Een vlak zonder `zoneId` wordt naar A_g over de zones verdeeld (oppervlakte en blootgestelde omtrek, zodat B' van een vloer gelijk blijft), met zijn openingen. Vlakken tussen twee rekenzones neemt de opname niet op (geen warmte-uitwisseling tussen rekenzones). Een onbekende `zoneId` geeft `surface_zone_unknown`.
5. **Gebouwbrede waarden.** ΔU_for (8.3) volgt uit de hele schil (§8.2.1: de forfaitaire methode geldt voor het gehele gebouw) en geldt in elke zone. b_U van een onverwarmde ruimte volgt uit alle aangrenzende zones (8.53) en is in elke zone gelijk. De toegepaste standaardwaarden van de schil staan één keer in de uitvoer, met de paden van het hele gebouw.
6. **Installaties.** Eén verwarmingsketen bedient alle zones (§9.2): de eerste zone is de `demand`, de overige staan in `additionalZones` met dezelfde afgifte en distributie. Koeling, tapwater (13.20 per functie), BACS, PV en opslag blijven per gebouw. Het ventilatiesysteem is het gebouwsysteem en krijgt per zone de functies, A_g en het setpoint van die zone; de luchtdoorlatendheid geldt voor alle zones (p. 55: een blowerdoortest voor het hele gebouw mag voor alle rekenzones). Een opgegeven geïnstalleerde capaciteit wordt naar A_g verdeeld (`installed_capacity_split_by_area`). Een zwembadruimte hoort bij de zone met de meeste sportfunctie. Bevochtiging geldt in elke zone.
7. **Verlichting (NTA 14.3).** Elke verlichtingszone krijgt een `zoneId` (verplicht met zones: `lighting_zone_calculation_zone_required`). Per rekenzone dekken haar verlichtingszones haar A_g (`zone_lighting_area_mismatch`). De forfaitregel van 14.3.4 geldt per rekenzone.
8. **Leidingen.** De verticale leidingen (tabel 7.8) staan in de eerste zone.
9. **Inklapredenen en registratie.** De paden zijn die van het gebouw (zonder zoneprefix), zodat opgeslagen inklapredenen blijven passen. De registratie blijft per gebouw.

Interpretaties: het verdelen van vlakken zonder zone naar A_g en het verdelen van de geïnstalleerde capaciteit naar A_g zijn keuzes van de opname; ISSO 75.1 geeft er geen regel voor. Een ΔU_for per gebouw in plaats van per zone volgt uit "voor het gehele gebouw" van §8.2.1.

## Fixtures

Synthetische opnames in `training-data/` (geen echte gebouwen):

- `nta8800-opname-utility-1985-office.json`: kantoor met kantine, collectieve HR-104, WTW-LBK met recirculatie.
- `nta8800-opname-utility-2005-school.json`: school met mechanische afzuiging, sensoren en PV.
- `nta8800-opname-utility-1970-retail.json`: winkel met veel onbekenden, koeling, stoombevochtiging en een onbekend BACS-vermogen.

## Inklapreden per forfaitaire waarde

BRL 9500 §4.2.2 en bijlage 3 vragen een onderbouwing wanneer de adviseur terugvalt op een forfaitaire waarde ("inklappen"). Beide opnames, woning en utiliteit, nemen daarom het veld `inklapRedenen` aan: een object met per pad of regel van een toegepaste standaardwaarde de reden.

- De uitvoer zet die reden als `inklapReden` bij de bijbehorende `appliedDefaults`.
- Een sleutel zonder bijbehorende standaardwaarde geeft de waarschuwing `collapse_reason_unmatched`.
- De dossierchecklist markeert toegepaste standaardwaarden zonder reden als ontbrekend.

## Beschaduwingssituaties (tabel 8.24/8.25)

`windows[].shading.situation` volgt de situaties van tabel 8.24 (gevels) en tabel 8.25 (daken), te vinden in 82.1 op p. 103 en in 75.1 op p. 105. De kern rekent ze om naar NTA §17.3.2:

| Situatie ISSO | `situation` | Kern (`obstruction.method`) | Toegestaan in de basisopname |
|---|---|---|---|
| Minimale belemmering | `minimal` | `minimal` (a) | altijd |
| Belemmering met constante hoogte | `constant_height_obstruction` (h_b;⊥) | `parallel_obstruction` (b) | gevel, met koeling in de rekenzone |
| Constante overstek (balkon, galerij) | `constant_overhang` (h_o;⊥) | `overhang` (c) | gevel |
| Volledige belemmering | `full` | `full` (e) | met koeling in de rekenzone |
| Constante overstek en één of meer (zij)belemmering(en) | `overhang_with_obstructions` (h_o;⊥) | `other` met overstek (g) | gevel |
| Overige belemmering | `other` | `other` (g) | altijd |
| Zijbelemmering | `side_obstruction` | `side_obstruction` (d) | alleen detailopname |

- Een situatie die niet is toegestaan geeft `shading_situation_not_in_basic_survey`. Opgegeven maandfactoren (`obstruction`) samen met `shading` geven `window_obstruction_conflict`.
- In de woningopname geldt de rij "met koeling" alleen als `cooling` is ingevuld (hoofdstuk 10, zie onder). De utiliteitsopname gebruikt de rij "met koeling" zodra `cooling` is ingevuld.
- Bij volledige belemmering geldt de koudetabel 17.14 alleen met `coolingConditionsMet`. Zonder dat veld rekent de kern conservatief met 1,00.


## Meerdere opwekkers, collectieve installaties, WKK en zonneboilers

Deze routes gelden voor de woningopname (ISSO 82.1) en, waar vermeld, ook voor de utiliteitsopname (ISSO 75.1).

**Meerdere verwarmingsopwekkers (§9.3.2, p. 112–113; NTA 9.6.1).**
- Met `heating.additionalGenerators` worden alle ongelijke opwekkers opgenomen, elk met een nominaal vermogen volgens tabel 9.7. Ontbreekt een vermogen, dan geeft de opname `generator_power_required`.
- Er zijn twee uitzonderingen:
  - Externe warmtelevering met onbekend vermogen levert 50 % (tabel 9.7).
  - Een WKK met onbekend thermisch vermogen krijgt 1,5 × het elektrisch vermogen (vuistregel voor gasmotoren).
- De preferentie volgt de volgorde van p. 112:
  1. afvoerluchtwarmtepompen;
  2. overige warmtepompen;
  3. WKK;
  4. biomassa;
  5. externe warmte;
  6. elektrisch;
  7. ketels.

  De kern rekent met de opwekker `multiple`.
- Een bijgeplaatste preferente opwekker (§9.3.6) zet `addedPreferredGenerator` (NTA 9.58/9.59).
- Tabel 9.9 geldt per opwekker. Een warmtepomp in een hybride opstelling krijgt standaard de laagtemperatuurklasse (voetnoot 8). De distributie krijgt de klasse van de opwekker die geen warmtepomp is.
- In de utiliteitsopname gelden de collectieve rol, het warmtepompbereik en de hulpenergie per deelopwekker.

**Collectieve verwarming (p. 106, 121–122).**
- `heating.collective` maakt van de ketel een collectieve ketel. Het nominale vermogen is dan verplicht.
- Een warmtepomp wordt een collectieve gebouwinstallatie.
- De distributie wordt collectief, met de forfaitaire pomp.
- Warmtemeters onbekend: aanwezig (tabel 9.16).
- Het aangesloten gebruiksoppervlak (f_gebouw, `collectiveConnection`) is bij onbekend het aantal woningen × het oppervlak van de woning (p. 121).

**WKK (tabel 9.7, NTA 9.6.6.1, tabel 9.31).**
- `kind: chp` met het elektrisch vermogen (rij van tabel 9.31) en het fabricagejaar (vóór of na 2006).
- Het thermisch vermogen dient als brandervermogen voor 9.91.
- De WKK-stroom telt als eigen productie volgens 16.12.

**Collectief tapwater (p. 164, 176).**
- Collectieve opwekker onbekend: overige direct verwarmde voorraadvaten op gas (tabel 13.2).
- Het bediende gebruiksoppervlak is bij onbekend het aantal woningen × het oppervlak van de woning.
- `delivery_set_from_heating` is warm tapwater via een afleverset op het (collectieve) verwarmingssysteem (§13.3.4, NTA 13.8.4.9.3).

**Meerdere tapwateropwekkers (NTA 13.8.2).** `hotWater.nominalPowerKw` en `hotWater.additionalGenerators` worden doorgegeven aan de cascade van de kern.

**Zonneboilers (§15.3–15.4; NTA 13.7.2.2), woning en utiliteit.**

| Gegeven | Bij onbekend |
|---|---|
| Collectortype | onverglaasd (tabel 15.8) |
| Naverwarming | voorverwarmer met apart naverwarmingstoestel (tabel 15.4) |
| Back-upvolume | afgeleid van het totale volume (tabel 15.5, NTA 13.80) |
| Fabricagejaar van het vat | bouwjaar (§15.3.3/§13.3.2) |
| Beschaduwing | minimale belemmering (§15.4.7) |

- Een bruto-oppervlak van vacuümbuizen telt voor 60 % (p. 192).
- Rendement, collectorkring en pomp zijn forfaitair.
- `alsoSpaceHeating` maakt er een zonnecombi van.

## WKK-vermogen naar aandrijving en meerdere tapwateropwekkers in de utiliteitsopname

- **WKK zonder bekend thermisch vermogen (tabel 9.7, ISSO 82.1 p. 113).** Het nieuwe veld `engine` bepaalt de vuistregel op het elektrisch vermogen: gasmotor 1,5×, dieselmotor 1,2×, microturbine 2,5×. Is de aandrijving onbekend, dan rekent de opname met een gasmotor en legt dat vast als standaardwaarde (`chp_engine_unknown_gas`).
- **Utiliteit, meerdere tapwateropwekkers (NTA 13.8.2).** `hotWater.nominalPowerKw` en `hotWater.additionalGenerators` gebruiken de eigen utiliteitstypen, waaronder de gasboiler (p. 167) en een onbekend collectief toestel (tabel 13.2, p. 165). Standaardwaarden worden per opwekker vastgelegd met hun eigen pad.

## Basisopname in de app

Het projectscherm heeft een paneel "Basisopname (ISSO 82.1 / 75.1)". Een woning- of utiliteitsopname start vanuit de synthetische voorbeeldopnamen en wordt met het project bewaard (`basisopname`, buiten de vingerafdruk van het label).

Gestructureerde velden zijn er voor:
- algemene gegevens en verticale leidingen;
- de vlakken van de schil, met isolatie, thermoskussens en het (na-)isolatiejaar;
- ramen, met de belemmeringssituatie uit tabel 8.24;
- verwarming: opwekker inclusief WKK, nominaal vermogen, extra opwekkers, collectief en later bijgeplaatst;
- tapwater: opwekker, CW-klasse, collectief, boilervat, extra opwekkers en zonneboilers;
- PV, met de belemmeringssituatie.

Al het overige staat in een JSON-weergave. "Opname doorrekenen" stuurt de opname naar de Rust-kern en toont de status, de indicatieve labelklasse, BENG 1–3, de meldingen en de toegepaste standaardwaarden met ISSO-bron. Per standaardwaarde kan de adviseur de reden voor het inklappen invullen (`inklapRedenen`, BRL 9500 §4.2.2).

## Meerdere tapwatersystemen (ISSO 82.1 p. 164)

`additionalHotWaterSystems` in de woningopname beschrijft elk extra tapwatersysteem met dezelfde velden als `hotWater`. Een voorbeeld is een keukengeiser naast een combitoestel voor de badkamer. Per systeem leggen `connectedBathrooms` en `connectedKitchens` de aangesloten ruimten vast (NTA 13.19a). Zijn ze onbekend, dan volgen ze uit `served`: één badruimte en/of één keuken. Dat wordt vastgelegd als `hot_water_connected_*_from_served`. De utiliteitsopname kent deze lijst nog niet.

## Koeling in de woningopname (ISSO 82.1 hoofdstuk 10)

De woningopname kan nu een gebouwgebonden koelsysteem opnemen. Het veld `cooling` heeft dezelfde vorm als in de utiliteitsopname: opwekker, afgifte, aantal binnenunits, distributie via water, temperatuurtraject, inregeling, regeling en leidingisolatie. `coolingCollective` geeft aan of de koudeopwekker collectief is.

Bij onbekende gegevens gelden dezelfde regels als in 75.1, met de paginaverwijzingen van 82.1:

| Gegeven | Bij onbekend | Bron |
|---|---|---|
| Vermogen | forfaitair | tabel 10.3, p. 130 |
| Ontwerptemperatuur | 6/12 °C; bij alleen oppervlaktekoeling 17/21 °C | tabel 10.4, p. 130 |
| WKO-jaar | het vergunningsjaar, anders vóór 2013 | p. 129 |
| Inregeling | niet ingeregeld | tabel 10.6, p. 132 |
| Leidingisolatie | niet geïsoleerd; het isolatiejaar is het bouwjaar | tabel 10.7, p. 133 |
| Appendages | niet geïsoleerd | tabellen 10.9–10.11, p. 134–136 |
| Regeling | overige situaties | tabel 10.12, p. 137 |

Binnenunits van split- en VRF-systemen tellen als ventilatorconvectoren, en betonkernactivering als vloerkoeling (p. 136).

`coolingPresent` zonder `cooling` geeft `cooling_system_data_required`. Met koeling gebruiken de ramen de beschaduwingsrij "met koeling" (tabel 8.24/8.25). Het opnamepaneel heeft hiervoor een koelsectie.

**Koeling: correcties na de review.**
- Een aquifer van vóór 2013 krijgt bij woningen EER 14 (`aquifer_dwellings_before2013`) en bij utiliteit 16 (tabel 10.34).
- `balanced` is nu `none`/`static`/`dynamic` (tabel 10.6); `true`/`false` van oudere opnames blijven werken.
- `heatPumpSource` (NTA 10.84) volgt bij onbekend de verwarmingswarmtepomp van de opname op een bodem- of grondwaterbron. `groundAboveZeroDemonstrated` legt vast dat de bron aantoonbaar boven 0 °C blijft (ISSO 82.1 p. 129).
- Bij ventilatorconvectoren en splitunits is het aantal toestellen verplicht (`cooling_fan_coil_count_required`).
