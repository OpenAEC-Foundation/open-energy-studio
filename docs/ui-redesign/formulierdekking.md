# Formulierdekking van de kerninvoer

Stand 9 oktober 2026. Elke keuze die de rekenkern kent, moet de gebruiker in een formulier kunnen maken, niet alleen in **Geavanceerd (JSON)**. Een certificeerbaar programma (BRL 9501 §3.0, de handleiding bij §4.4) laat de adviseur geen JSON schrijven voor gewone invoer. Dit document legt vast welke invoer een formulier heeft, en welke bewust alleen via JSON gaat, met de reden.

## Hoe de dekking wordt bewaakt

- **De catalogus.** `training-data/nta8800-input-options.json` bevat elke keuzewaarde (enum-variant) en elk optioneel veld van de projectinvoer (`ntaCalculation` en het projectmodel) en van beide basisopnamen. Serde levert de lijst, niet een handlijst: de test `input_option_catalogue_is_current` in `crates/nta8800-core/tests/option_coverage.rs` leidt hem af met dezelfde probe als de optiedekking. Komt er in de kern een variant of veld bij, dan faalt die test tot de catalogus is bijgewerkt (`UPDATE_INPUT_OPTIONS=1`).
- **De formuliercheck.** `src/__tests__/nta-form-coverage.test.ts` telt een optie alleen als gedekt wanneer de formulieren van **haar eigen route** haar **schrijven** (`FORM_SOURCES` in `src/core/nta/formCoverage.ts`, per route: project, woningopname, utiliteitsopname):
  - de naam staat op een schrijfplek: een keuzewaarde, een padelement of een sjabloonsleutel. Een lezing (`read(`), een vergelijking (`=== 'x'`, `case 'x'`) of commentaar telt niet; commentaar wordt eerst verwijderd;
  - bij een veld staat de ouder-sleutel bij die schrijfplek: binnen een paar regels, of in de definitie van de padvariabele (`const base = ['envelope', 'doors', index]`), of de component krijgt die padvariabele van zijn aanroeper. Zo dekt het `glass` van een raam niet het `glass` van een deur in hetzelfde bestand;
  - weergavepagina's (de shell) tellen niet mee; alleen het projectinfo-venster en de gebouwpagina staan erin. Het warmtepomppaneel staat erin omdat het de forfaitaire warmtepomp van het project invult (`FORM_BUILDERS`: het bouwt het hele object in één keer).

  Een optie die zo niet gedekt is, staat in `FORM_COVERAGE_EXEMPTIONS` met de reden en een soort: `form` (wel een formulier, niet op naam te zien), `json` (bewust alleen JSON), `missing` (nog geen formulier, te bouwen) of `parallel` (formulier in een parallelle wijziging). Een vrijstelling die inmiddels wel gedekt is, laat de test falen, zodat de lijst niet veroudert.
- **Grens van de check.** Het blijft tekstanalyse, geen render-test per veld. Wat de check als gedekt telt, schrijft wel echt die naam onder die ouder; of het veld op elke plek (elke zone, elk systeem) staat, toetsen de formuliertests (`nta-form-coverage-forms.test.tsx`).
- **Wat de formulieren schrijven, door de kern.** Dekking zegt niet dat de kern leest wat een formulier schrijft. Dat bewaakt het vangnet voor de projectformulieren: `src/__tests__/nta-project-form-guard.test.tsx` bedient elke getoonde NTA-sectie op het kantoor, de rijwoning en het kantoor met twee zones (elke keuze door al haar opties, elk vinkje, elke knop "toevoegen", daarna alles leeg) en legt elke toestand vast zoals "Toepassen" en de projectroute hem doorgeven. In de gate rekent `crates/nta8800-core/tests/project_form_guard.rs` ze door de kern en weigert een onleesbare vorm, een melding zonder route en een achtergebleven toestand uit `LEFTOVER_STATE`. Voor de opname doet `survey-null-paths.test.tsx` met `survey_form_shapes.rs` hetzelfde voor de lege waarden.

## Dekking

Tot 9 oktober 2026 telde de check een optie als gedekt zodra haar naam ergens in een formulierbestand stond, ook in een lezing, commentaar of een formulier van een andere route. Dat gaf 1 807 / 1 823 (99,1 %), te hoog: de zevende review vond opties die geen formulier aanbiedt (onder meer het glas van een deur, onverwarmde ruimten in de opname, en `bblFunction`-waarden die alleen in de lijst voor meerdere functies stonden). Met de strengere check:

| Route | Opties | Door een formulier geschreven | Vrijstelling `form` | `json` | `missing` | `parallel` |
|---|---|---|---|---|---|---|
| Projectinvoer | 1 098 | 1 074 | 8 | 16 | 0 | 0 |
| Woningopname | 316 | 316 | 0 | 0 | 0 | 0 |
| Utiliteitsopname | 409 | 409 | 0 | 0 | 0 | 0 |
| **Totaal** | **1 823** | **1 799 (98,7 %)** | **8** | **16** | **0** | **0** |

Met de vrijstellingen van soort `form` erbij heeft **1 807 / 1 823 (99,1 %)** een formulier; de overige 16 opties zijn bewust alleen JSON (zie hieronder). Geen enkele optie staat nog als `missing`. Na de strengere check stonden er 67 open (9 oktober 2026, ochtend); die kregen in de ronde van 9 oktober 2026 (middag) een formulier, zie "Formulieren van de tweede ronde".

Eerder telde de check 1 732 / 1 823 (95,0 %) door een formulier geschreven, met 67 opties `missing`.

In deze ronde kregen een formulier: de douche-WTW (§13.5.3, per douche met verbinding en opgegeven rendement), de kokendwaterkraan, het gasgestookte voorraadtoestel (13.165–13.175), de ontbrekende Bbl-functies (woonwagen, drijvende bouwwerken, logies in een logiesgebouw) en de berekende Ψ van de verwarmingsleidingen (9.33–9.35).

## Waar de invoer staat

| Invoer | Formulier |
|---|---|
| Functie, berekeningsgebied, woningtype (`labelFunction`, `bblFunction`, `usageFunction`, `calculationScope`, `dwellingType`) | Stap Project en Gebouw |
| Schil: vloeren op grond, hellingen, belemmering, zonwering en glas per raam, massa, interne warmte, setpoints | Gebouw › Schil & ramen, Gebouw › Zones |
| Verwarming: opwekker (rol van de ketel, bronsysteem van de warmtepomp), afgifte | Installaties › Verwarming |
| Distributie: Ψ forfaitair of berekend uit de leidinggegevens (9.33–9.35), pomp (EEI, ontwerpvolumestroom, eenpijpsysteem, vermogen, L_max), combinatieleidingen, b_U; bij een collectieve installatie de afleverset, en met een berekende Ψ ook het buffervat (tabel 9.16 opmerking 2) | Installaties › Verwarming › Distributie |
| Tapwater: alle opwekkers (ook boosterwarmtepomp bijlage W, warmtepompen in serie, warmtepomp met indirect vat, biomassa-combitoestel, groot direct gestookt vat, gasgestookt voorraadtoestel), kokendwaterkraan, douche-WTW, afleversets, b_U | Installaties › Tapwater |
| Koeling: opwekkers, afgifte, distributie met leidinglengte buiten de gekoelde zone en de circulatiepomp | Installaties › Koeling |
| Ventilatie: systeem, ventilatoren (opgegeven of motorvermogen), woningoppervlak bij meer zones | Installaties › Ventilatie |
| Verlichting, PV, GBS, opgegeven gebruik, hernieuwbaar | Installaties › Verlichting, Opwekking, Overig |
| Gegevens per rekenzone (`zoneData`): gebruiksfunctie, woningtype, delen met een andere functie, thermische massa, serres, zwembad in een sportfunctie | Gebouw › Zones › Gegevens per rekenzone (bij meer rekenzones) |
| Tapwater per systeem: circulatieleiding (isolatie, lengte, deel in onverwarmde ruimten, bouwlagen, pomp, opgegeven of berekende Ψ), extra opwekkers en serieopstelling ook bij een verder tapwatersysteem | Installaties › Tapwater |
| Collectieve koeling en gedeeld PV-systeem (aandeel naar de gebruiksoppervlakte van het gebouw), rekenzone van de verlichting | Installaties › Koeling, Opwekking, Verlichting |
| Woningopname: woningtype, ligging, daktype, constructie en plafond, gemeten q_v10, ventilatieprincipe en unit, bypass, ruimteregeling, waterzijdig inregelen, bouwlagen van de distributie, leidingen in onverwarmde ruimten, douches, tappunten en leidinglengtes, douche-WTW, deuren, panelen, glas (alle soorten van tabel 7.8), oriëntatie, helling, spouw en omtrek per vlak, opgegeven belemmeringsfactoren per raam, PV-ventilatie en belemmeringsfactoren, rietdak, kruipruimtebodem, onverwarmde ruimte als grens, bron van de gebruiksoppervlakte, opslag | Basisopname (woning) |
| Utiliteitsopname: gebouwtype, verlichtingszones (armaturen of lampen, lamptype, aanwezigheid, daglicht), recirculatie, volumestroomregeling, LUKA-kanalen, LBK-kanalen en LBK binnen de thermische zone, bevochtiging, GBS, collectieve verwarming, zonwerend glas, koeling (ontwerptemperatuur, regeling, leidingisolatie), circulatie en voorraadvaten, en verder dezelfde schil-, PV- en tapwatervragen als de woningopname | Basisopname (utiliteit) |

## Bewust alleen via JSON

| Invoer | Reden |
|---|---|
| `ntaCalculation.internalGains.lighting.method: resolved` | Interne waarde: de kern vult Φ_int;L zelf in uit hoofdstuk 14; geen invoerkeuze. |
| `lighting[].burningHoursFactor` | Alleen voor maatwerkadvies (ISSO 75.2 tabel 2.7); het maatwerkadvies zet hem, nooit voor het energielabel. |
| `ventilation.practice` (ook per zone) | Alleen voor maatwerkadvies (praktijkfactoren ISSO 82.2/75.2); het maatwerkadvies zet hem. |
| `ventilation.overventilation` (ook per zone) | Maandwaarden die de kern uit de warmtepompketen afleidt; alleen opgeven bij afwijkende meetgegevens. |
| `ventilationFlows[].months[].coolingConductanceWPerK`, `coolingSupplyTemperatureC` | Afwijkende koelwaarden per maand van een opgegeven luchtstroom; leeg is de verwarmingswaarde. |
| `thermalMass.annexBElements` (ook per zone) | Bijlage B per bouwdeel vervangt tabel 7.10 en vraagt een lagenopbouw per element. |
| `generator.auxiliaryMeasurements` | Componentmetingen volgens bijlage O (9.86–9.90) uit een meetrapport. De opgegeven A/B/C uit een kwaliteitsverklaring hebben wel een formulier. |
| `hotWater.generator.annexT`, `annexTConditions` | Beproevingsrapport volgens bijlage T (gaskeur-koppeling); zeldzaam, uit het rapport over te nemen. |
| `distributionSystem.unheatedPipeTransmittance` | Afwijkende Ψ voor het deel in onverwarmde ruimten; leeg geldt de Ψ van het hele systeem. |

**Wel een formulier, niet op naam te vinden:** het soort en de kleur van een zonweringstoestel. Zonwering per raam en de projectbrede zonwering kiezen het toestel als één samengestelde waarde "soort:kleur". `distributionSystem.usageFunction` neemt het sjabloon over van het blok.

**Bewust JSON, nieuw in de lijst:** de maandwaarden van de warmte die een boosterwarmtepomp aan de koeling onttrekt, en de maandfracties van de ventilatorregeling; beide uit een meetrapport.

**11 oktober 2026:** de optiedekking kreeg openbaar geval N (verwarmde kelder met een diepte per wanddeel, 8.42) als projectvoorbeeld. Daardoor staan de velden van `groundFloors[].heatedBasement` en zijn wanddelen (`wallDepths[]`: lengte en diepte) nu in de catalogus, en ook enkele blokken die alleen N heeft (de per-raamzonwering `windowShadings`, luchtverwarmers, armaturen per lamp, het koelafgifteblok van de editie 2023 en de LBK-kanalen buiten de zone). Die hebben alle een formulier. Twee kregen een vrijstelling:
- het soort en de kleur van het toestel in `windowShadings[].movableShading` (`form`): dezelfde samengestelde keuze "soort:kleur" als de projectbrede zonwering;
- `groundFloors[].heatedBasement.forfaitDeltaUWPerM2k` (`json`): de kern leidt ΔU_for van de verwarmde kelder af uit het projectmodel (8.38); alleen opgeven om daarvan af te wijken. De overname uit de opname laat een opgegeven waarde staan.

**11 oktober 2026, review 55:** de optiedekking kreeg invoer op de routes van reviews 40–54 (gedeelde serre per zone, stoombevochtiger op olie, collectieve aansluiting, koudebrugmethode van het gebouw, TOjuli-methode 3). De catalogus noemt nu ook die velden; alle hebben een formulier, behalve twee vrijstellingen (`json`):
- `zoneData[].sunrooms[].borderingZones`: de kern leidt de aangrenzende zones van een gedeelde serre af uit de zones met dezelfde serre-id (7.38, review 54); een opgegeven lijst wordt vervangen;
- `zoneData[].ventilationFlows[].months[].coolingConductanceWPerK`, `coolingSupplyTemperatureC`: dezelfde vrijstelling als voor de luchtstromen van het blok.

## Nieuwe formulieren in deze ronde

- **Tapwater:** boosterwarmtepomp met meetwaarden (bijlage W: beproevingen bij 24 en 40 °C, klasse, stilstandsverlies W.17, brontemperatuur als één waarde of per maand, bron van de warmte), warmtepompen in serie (bron van de laatste warmtepomp), warmtepomp met indirect verwarmd vat, biomassa-combitoestel (isolatie volgens bijlage R), overig direct gestookt voorraadtoestel; afleversets en b_U per tapwatersysteem.
- **Verwarming:** rol van de ketel (ook individuele bijverwarming), bronsysteem van de warmtepomp (9.62); in de distributie de pompgegevens, het buffervat, combinatieleidingen, de afleverset en b_U.
- **Koeling:** leidinglengte en temperatuur buiten de gekoelde zone, en de circulatiepomp (10.4.2.3).
- **Ventilatie:** vermogen van een ventilator uit het motorvermogen (11.135, tabel 11.20) en het oppervlak van de hele woning bij een woning in meer zones.
- **Interne warmte en verticale leidingen:** aandeel woningen in de zone (6.2b) en het aandeel gebruiksoppervlak per leiding.
- **Basisopname:** zie de tabel hierboven; daarnaast de volledige glaslijst van ISSO 82.1 tabel 7.8 / 75.1 tabel 7.9.

De formulieren schrijven precies de vorm van de kern; `src/__tests__/nta-form-coverage-forms.test.tsx` controleert dat per formulier (formulier → kern-JSON) en dat de invoergaten van de nieuwe velden naar de juiste stap leiden.

## Formulieren van de tweede ronde (9 oktober 2026, middag)

- **Gegevens per rekenzone:** een rekenzone krijgt desgewenst eigen gegevens; die beginnen als kopie van de interne warmte, de luchtstromen en de ventilatie van het blok, omdat de kern die bij een eigen `zoneData` alleen uit de zone haalt.
- **Circulatieleiding** voor het hoofdsysteem en elk verder tapwatersysteem (13.27–13.39); opgegeven en berekende Ψ sluiten elkaar uit.
- **Verder tapwatersysteem:** eigen nominaal vermogen, extra opwekkers en serieopstelling (de serie verdwijnt als er niet precies één extra opwekker is).
- **Collectieve koeling en gedeeld PV-systeem**, **zwembad** in een sportfunctie (§11.2.2.5.1), **rekenzone van de verlichting**.
- **Opname:** zie de tabel hierboven. Opslag (§15.5) wordt alleen met PV gevraagd (`storage_requires_pv`); opgegeven belemmeringsfactoren van een raam of PV-veld vervangen de situatie (die sluiten elkaar uit).
- **Boosterwarmtepomp bijlage W:** de twee beproevingen hebben elk een eigen naam ("beproeving 1/2").

`src/__tests__/survey-coverage-forms.test.tsx` en `src/__tests__/nta-form-coverage-project.test.tsx` controleren formulier → kern-JSON; `crates/nta8800-core/tests/survey_form_shapes.rs` en `project_form_shapes.rs` laten de kern dezelfde vormen lezen en doorrekenen.
