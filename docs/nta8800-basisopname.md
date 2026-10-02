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
| Thermische massa | Tabellen 7.5/7.6 (erratum §2) worden omgezet naar de klassen van NTA-tabel 7.10 | p. 62 |
| Opake constructies | R_c volgens bouwjaar en isolatiestaat via NTA-bijlage I (`forfait_envelope`). Een nageïsoleerde spouw met onbekende breedte volgt tabel 8.26 (40/70/100 mm) | p. 84–93 |
| Beglazing | Gelijkstellingen: dubbel + voorzetraam = HR, enzovoort. U- en g-waarde uit tabel 8.14/8.15. Een raam zonder kozijn telt als hout/kunststof. Kozijnfractie 0,25 (NTA 7.6.6.2, methode B) | p. 93–95 |
| Deuren en panelen | Een deur met minder dan 65 % glas wordt gesplitst in een raam- en een deurdeel. Is niet vast te stellen of de deur geïsoleerd is, dan geldt ongeïsoleerd. Panelen volgen tabel 8.18–8.21 | p. 70, 95–97 |
| Koudebruggen | Forfaitair voor de hele woning: ΔU_for (NTA 8.2/8.3) op alle buitenvlakken; vloeren op grond krijgen 0,5·P | p. 79 |
| Aangrenzende onverwarmde ruimten | H_ue = 5·A (NTA I.8) en b_U = H_ue/(H_ue + H_iu) | NTA I.2.4, 8.4.1 |
| Belemmering | Standaard "geen" (minimale belemmering) | p. 195 |
| Verwarming | Geen opwekker aanwezig → CR-ketel. Waakvlam onbekend → met waakvlam. Waterstofketel → HR-107. Watergedragen bron onbekend → bodem. Ontwerptemperatuur onbekend → tabel 9.9. Bij meerdere afgiftesystemen gaat oppervlakteverwarming voor. Inregeling onbekend → niet ingeregeld. Regeling onbekend → overig | p. 107–124 |
| Distributie bij externe warmte, elektrisch of biomassa met watergedragen afgifte | Pomp forfaitair, warmtemeter aanwezig, leidingen ongeïsoleerd | p. 117–122 |
| Tapwater | Geen systeem → elektrisch doorstroomtoestel. Gastoestel onbekend → badgeiser. Gaskeur onbekend → geen. CW-klasse onbekend → CW-4/5/6. Keukengeiser boven 13 kW → badgeiser. DWTW onbekend → niet aangesloten | p. 164–180 |
| Ventilatie | Zelfregelende roosters (tabel 11.3/11.5). Sturing onbekend → geen. WTW onbekend → geen. Tegenstroom met onbekend materiaal → aluminium. Toevoerkanaal en bypass volgens tabel 11.10–11.12. Kanaaldichtheid onbekend → 1,1. Ventilatormotor onbekend → wisselstroom (tot en met 2006) of gelijkstroom (vanaf 2007) | p. 142–154 |
| PV | Type onbekend → polykristallijn. Jaar onbekend → bouwjaar (vóór 2001 telt als 2000). Montage onbekend → niet geventileerd | p. 191 |

## Bekende bronfouten

- ISSO-tabel 7.4 drukt 350 af voor "zeer zware vloer, zware wand". NTA-tabel 7.10 en het wijzigingsdocument geven 450. De kern rekent met NTA-tabel 7.10, dus met 450.
- De R_c van thermokussens is 1,95 (WD p. 37) in plaats van 1,80. Bij een opgegeven dikte rekent de kern via bijlage I.
- λ riet is 0,105 (WD p. 36) in plaats van 0,2. Riet heeft nog geen invoerroute in de opname.

## Niet ondersteund

- Koeling: geeft de fout `cooling_not_supported_in_basisopname`.
- Collectieve installaties, WKK, zonneboilers en meerdere opwekkers.
- Leidingen in onverwarmde ruimten, serres (AOS), daklichten, riet en woonboten/woonwagens.
- Detailopname-routes en kwaliteitsverklaringen. De uitzondering is een gemeten q_v10.
- Kruipruimtevloeren worden benaderd als vloer op grond (waarschuwing `crawlspace_floor_approximated_as_slab_on_ground`). De kern mist NTA 8.3.4.2 en bijlage D.2.2.4.

## Interpretatievragen

1. Een appartement met een onbekende positie aan de zijkant is als kop- of hoekligging ingedeeld, naar analogie van de hoekwoningregel op p. 51.
2. Een deur waarvan de isolatie niet vast te stellen is, telt als ongeïsoleerd. ISSO noemt dat niet expliciet; dit volgt uit de conservatieve regel op p. 28.
3. ISSO-klasse 70/50 heeft geen rij in NTA-tabel 9.14. De distributie gebruikt 70/60 (gelijke aanvoertemperatuur). De ketel gebruikt de gemiddelde temperatuur 60 °C.
4. ΔU_for (8.3) wordt niet toegepast op scheidingen met onverwarmde ruimten, omdat H_D;for alleen de buitenlucht betreft.
5. Een combitoestel met Gaskeur CW zonder HR telt als "combi met Gaskeur" (0,50). Tabel 13.25 kent alleen de combinatie HR + CW als hogere rij.
