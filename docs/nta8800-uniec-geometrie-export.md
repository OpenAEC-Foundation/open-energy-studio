# Export naar de geometrie-import van Uniec (concept)

Issue #14. Open Energy Studio schrijft de geometrie van een woning in het Excel-formaat dat Uniec 3 sinds versie 3.4.2.0 (28 mei 2026) kan importeren. De code staat in `src/core/io/UniecGeometryExport.ts`, de tests in `src/__tests__/uniec-geometry-export.test.ts` en `uniec-geometry-export-ui.test.tsx`. In de app: **Rapport & dossier › Exports › Uitwisseling › Uniec-geometrie (Excel, concept)**.

**Status: concept.** Het bestand is nog niet in Uniec geïmporteerd. Tot iemand met een Uniec-licentie dat heeft gedaan en de uitkomst hieronder heeft vastgelegd, is de export niet "ondersteund".

## Contract

| Bestand | Datum | SHA-256 | Gebruik |
|---|---|---|---|
| `import-beschrijving-2026-09-18.xlsx` (uniec3.nl/importfunctie) | 18-09-2026 | `312f5028cda667003609dd1eaeae4ee396e982378a8f476826999b1f247360cd` | kolommen, verplichte velden, keuzes, afrondingsregels en meldingen |
| `2026-05-28-voorbeeld-Excel-import-bestand-woning.xlsx` | 28-05-2026 | `1c21e68becfb2b354b1e092c9798ba4e7e485d976e5c99067c8d2de4c7e3ca71` | vergelijking (zie onder) |

De bestanden zelf staan niet in de repository; ze zijn van Uniec. De export volgt de kolomnamen en de kolomvolgorde van de specificatie. Het werkblad heet `algemene gegevens` (9 kolommen) en `geometrische gegevens` (74 kolommen, tot en met `Opmerking`).

**Verschillen tussen voorbeeld en specificatie.** Het voorbeeld is ouder dan de specificatie:
- het voorbeeld mist de kolommen `nbouwlaagUnit`, `AOR_AOS`, `Vloer`, `Aantal`, `Kruipruimteventilatie`, `AfstandMaaiveld`, `Rbw`, `RbfForf`, `RbfProd`, `RbfProdWaarde` en `Opmerking`;
- de volgorde verschilt: in het voorbeeld staan `Perimeter`, `Lengte` en `Oppervlakte` direct na `AantalTransparanteConstructies`;
- het voorbeeld schrijft `eigen waarde Dm;int;eff`, de specificatie `eigen waarde Dm;int;eff - tabel 7.10`.

Uniec leest de kolommen vermoedelijk op naam; dat is nog niet bevestigd.

## Reikwijdte van deze eerste versie

Wel: één grondgebonden woning (`TypeGebouw` = `grondgebonden woning`), nieuwbouw of bestaande bouw, met de rekenzones, de dichte delen (methodiek *vrije invoer* met R_c), ramen, deuren en panelen (U, g), vloeren direct op de grond met perimeter, en per raam de belemmering en zonwering waar het contract die zonder afstanden kan bevatten.

Niet (de export weigert met de reden, of noemt wat niet meegaat):

| Onderdeel | Gedrag |
|---|---|
| utiliteitsgebouw, appartement, appartementengebouw | geweigerd |
| vloer boven een kruipruimte of een verwarmde kelder, kelderwand | geweigerd (het contract vraagt Rbw, Rbf, kruipruimteventilatie, afstand tot maaiveld) |
| belemmering met afstanden en hoogtes (evenwijdig, overstek, zijbelemmering) of opgegeven factoren | geweigerd: het project kent relatieve hoogtes, het contract absolute maten |
| zonwering met alleen een reductiefactor F_c | geweigerd: het contract vraagt het type |
| koudebruggen (lineair en punt) | niet meegenomen: het contract vraagt per koudebrug de positie |
| ventilatieve koeling (openingen) | niet meegenomen; elk raam staat op `niet aanwezig` |
| kozijnfractie, dynamische beglazing (bijlage A), warmtecapaciteit volgens bijlage B | niet meegenomen |
| installaties, opname, registratie | buiten het contract (Uniec noemt de installatie-import een volgende stap) |

## Vertaling van projectgegevens naar het contract

### Algemene gegevens

| Kolom | Bron in het project | Afleiding |
|---|---|---|
| Omschrijving | `name` | alleen letters, cijfers, spatie en `- _ . ,`, tot 32 tekens (F002) |
| Plaats | `city` | verplicht; Uniec accepteert alleen namen uit de CBS-lijst van de specificatie, dat controleert de export niet |
| TypeGebouw | `buildingFunction` = `residential` | `grondgebonden woning` |
| SoortBouw | `registration.purpose` | `delivery`/`bbl_check` → `nieuwbouw`; anders `bestaande bouw - niet gerenoveerd` (het project legt geen renovatie vast) |
| Bouwjaar | `registration.constructionYear` | verplicht; naar boven afgerond op een heel jaar (R001) |
| Renovatiejaar, Eigendom | — | leeg (niet verplicht) |
| Opname | — | `detailopname` (de berekening is een detailberekening) |
| NaamExterneTool | — | `Open Energy Studio` |

### Geometrische gegevens, per rij

Elke rij herhaalt de zonekolommen. Per vlak komt een rij per raam of deur en een rij voor het dichte deel.

| Kolom | Bron in het project | Afleiding |
|---|---|---|
| WoningOmschrijving | `name` | als Omschrijving |
| TypeZone, NaamZone | rekenzone | `rekenzone`, `zone.name` |
| BouwwijzeVloeren / -Wanden | `ntaCalculation.thermalMass.floor` / `.wall` (tabel 7.10) | de klasse als een bouwwijze van het contract met die klasse: licht → `hsb, sfb, schuimbeton of hout (licht)` / `hsb, sfb of staalskeletbouw (licht)`, zwaar → `staal-beton, hout-beton of niet-massief beton (zwaar)` / `dragend metselwerk (zwaar)`, zeer zwaar → `massief beton (zeer zwaar)` / `betonnen wand-vloer skeletbouw (zeer zwaar)` |
| BouwwijzePlafonds | — | de klasse van de vloeren (het project legt plafonds niet apart vast) |
| Ag | `zone.floorArea` | R014 |
| NaamConstructie | `surface.name` | F002 |
| Vlak, Begrenzing | `surface.type`, `surface.thermalBoundary` | gevel/dak/vloer; buitenlucht (vloer → `vloer boven buitenlucht`), grond (alleen vloer), onverwarmde ruimte → `aor_for`, verwarmde buurruimte → `avr`; zonder grens: vloer → grond, anders buitenlucht |
| Orientatie | `surface.orientation` | N, NO, O, ZO, Z, ZW, W, NW; horizontaal dak → `HOR` |
| Helling | gevel 90; hellend dak uit `ntaCalculation.surfaceTilts` | R016; plat dak en vloer leeg, zoals het voorbeeld |
| Maaiveld | — | `opBovenMv` voor een vloer op de grond |
| Perimeter | `ntaCalculation.groundFloors[].exposedPerimeterM` | verplicht zonder lineaire rijen; R014 |
| NaamBibliotheek | constructie of kozijntype | één naam per combinatie van naam, soort en vlak (F214); een dubbele naam krijgt een volgnummer |
| TypeConstructie, Methodiek | — | `dicht` of `transparant`; `vrije invoer` |
| Rc | `construction.rcValue` | 0 < R_c ≤ 50; R014 |
| Oppervlakte (dicht) | bruto vlak min de ramen | R014 |
| TypeTransparant | kozijntype `kind` | raam, deur, paneel in kozijn |
| Uw/Ud | `window.uValue` | 0 < U ≤ 10; R012 |
| ggl;n | `window.gValue` | naar beneden op een veelvoud van 0,05 (R011) |
| A + AantalTransparanteConstructies, of Oppervlakte | kozijntype `unitArea` en `window.count`, of `window.area` | per kozijn als elk raam een type met oppervlakte per stuk en een aantal heeft; anders de totale oppervlakte. Eén manier voor alle rijen (F112) |
| Beschaduwing | `windowObstructions` of `windowSolar.obstruction` | minimaal → `minimale belemmering`, volledig → `volledige belemmering`, overig → `overige belemmering`; alleen bij buitenlucht en g > 0 |
| Zonwering, Regeling | `windowShadings` of `windowSolar.movableShading`; `windowGlazings` (lamellen) | tabel 7.5/7.6 naar de keuzes van het contract (een donker rolluik → `overige kleuren`); handbediend → `handbediend`, automatisch ISO 52016 → `automatisch intelligent`, automatisch → `automatisch overig`; zonder zonwering `geen zonwering` |
| GglAlt, GglDif | `windowGlazings[].glazing.diffusing` | bij `vaste lamellen - ISO 15099;2003` |
| VentilatieveKoeling | — | `niet aanwezig` bij elk raam naar buitenlucht |

## Vergelijking met het voorbeeld van Uniec (23 oktober 2026)

De test bouwt de woning van het voorbeeld na: één rekenzone van 96 m², vier gevels met R_c 6, op het zuiden 4 ramen van 6 m² en een deur, op het westen 2 ramen van 6 m² met witte rolluiken, een vloer op de grond (38 m², perimeter 20 m) en een plat dak van 48 m². Het gemaakte bestand is cel voor cel op kolomnaam vergeleken met het voorbeeld. Gelijk zijn: de algemene gegevens (op Eigendom, de naam van het hulpmiddel en een dubbele spatie in de omschrijving van het voorbeeld na), alle oppervlaktes van de dichte delen (43,2; 32,4; 16,7; 20,4; 38; 48 m²), de oriëntaties, de hellingen, R_c, U, g, A en het aantal per raam, de zonwering met regeling en de perimeter. Dat in het voorbeeld de zuidgevel bruto 43,2 m² is (16,7 + 4 × 6 + 2,5), bevestigt dat `A` de oppervlakte per kozijn is.

Verschillen, met de reden:

| Verschil | Reden |
|---|---|
| Bouwwijze: voorbeeld `eigen waarde Dm;int;eff` met D_m 250 en `nbouwlaag` 2 | de export schrijft de klasse van tabel 7.10; `nbouwlaag` is alleen verplicht voor projectwoningen |
| raam zuid: voorbeeld met constant overstek en ventilatieve koeling | de nagebouwde woning heeft die niet; een overstek weigert de export (zie boven) |
| vloer: voorbeeld met een lineaire rij `perimeter` (ψ 0,8, 8 m) | de export schrijft de perimeter en geen koudebruggen |
| dak: voorbeeld met `Beschaduwing` op het dichte dak | het contract vraagt beschaduwing alleen bij transparante delen |
| volgorde van ramen en dichte delen per gevel | de export zet de ramen vóór het dichte deel; het voorbeeld wisselt dat per gevel |

## Nog te doen vóór "ondersteund"

1. Het bestand van de nagebouwde voorbeeldwoning en van een echte projectwoning importeren in de actuele Uniec-versie (minstens 3.4.2.0) en de meldingen (F-codes) vastleggen.
2. Na de import in Uniec nalopen: aantal rekenzones en vlakken, oppervlaktes, oriëntaties, hellingen, R_c, U en g. Geen gelijkheid van het label claimen op grond van deze export.
3. Vastleggen of Uniec de kolommen op naam of op positie leest.
4. Daarna: kruipruimte en kelder, belemmering met maten, koudebruggen met positie, appartementen.
