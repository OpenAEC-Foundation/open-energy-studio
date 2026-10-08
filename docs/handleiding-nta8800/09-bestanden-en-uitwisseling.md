# 9. Bestanden en uitwisseling

## Projectbestanden (`.oes.json`)

**Opslaan.** Met **Opslaan** (Ctrl+S) in de bovenbalk of **Opslaan als** (Ctrl+Shift+S) in het menu Bestand schrijft de app het project als `<projectnaam>.oes.json`.
- In de desktopapp kies je de plaats in een opslagvenster; een project dat al een bestandspad heeft, wordt daar direct overschreven.
- In de browser wordt het bestand gedownload.

Het bestand bevat het volledige project, inclusief de NTA-invoer, de basisopname, het maatwerkadvies en de registratiegegevens. Daarnaast bevat het een stempel van de rekenkern: kernelversie, normversie en de invoervingerafdruk op het moment van opslaan.

**Openen.** Met **Openen** (Ctrl+O, menu Bestand of welkomstscherm) kies je een `.oes.json`- of `.json`-bestand; in de desktopapp met een bestandsvenster, in de browser met de bestandskiezer van de browser. Het project opent in een nieuw tabblad. Bij het openen:
- vergelijkt de app de opgeslagen stempel met de huidige rekenkern. Is de kern of de normversie anders, dan meldt ze dat; reken dan opnieuw en controleer de verschillen voordat je registreert. Verschilt alleen de invoervingerafdruk, dan is het bestand buiten de app gewijzigd en meldt ze dat ook.
- zet ze koppelingen van bewijs en foto's die nog op positie staan om naar koppelingen op id (zie [hoofdstuk 7](07-herlabelen-registratie-dossier.md));
- zet ze oude herlabelprojecten om naar de huidige vorm (zie [hoofdstuk 7](07-herlabelen-registratie-dossier.md)). Facturen zonder herlabelrol krijgen de rol "Te beoordelen", die nog niet als bewijs telt; de app meldt welke velden nog ontbreken.

## Voorbeeldprojecten

Het welkomstscherm (na het sluiten van alle tabbladen) biedt twee volledige voorbeelden: **Voorbeeld: tussenwoning** en **Voorbeeld: klein kantoor**. Ze openen ook via de adresparameter `?example=terraced_dwelling` of `?example=small_office`.

De voorbeelden zijn fictieve oefenprojecten. BENG en labelklasse zijn indicatief; er hoort geen geregistreerd energielabel bij. De bronbestanden staan in `training-data/nta8800-example-*.json`.

## UNIEC3-uitwisseling

De UNIEC3-knoppen staan onder Rapport & dossier › Exports (export), op het welkomstscherm en in het menu Bestand (import), en in het palet (Ctrl+K).

**UNIEC3 invoerconcept** exporteert het projectmodel als een zip-bestand `<projectnaam>.input-draft.uniec3`. De indeling is nagebouwd op basis van bestaande UNIEC3-bestanden van NTA 8800 v3.4. Het bestand bevat:
- gebouw, rekenzones, begrenzingsvlakken, constructies, ramen, koudebruggen en de luchtdoorlatendheid (q_v10);
- de installaties: verwarming, ventilatie, koeling, tapwater, PV en zonthermie.

Wat het **niet** bevat of garandeert:
- geen NTA-invoerblok (`ntaCalculation`), geen basisopname, geen maatwerkadvies en geen registratiegegevens;
- geen rekenuitkomsten (de samenvatting is leeg) en geen label;
- geen garantie dat een ander programma het bestand accepteert of de waarden op dezelfde manier leest. Controleer na het inlezen elders altijd de invoer.

**UNIEC3 import** leest een `.uniec3`-bestand en opent het als een nieuw project. Alleen het projectmodel komt mee: zones, vlakken, ramen, koudebruggen, constructies en installaties. De NTA-invoer, opname en registratie moet je daarna zelf invullen; tot dan geeft de rekenkern invoergaten.

## Exports onder Rapport & dossier

| Knop | Uitvoer |
|---|---|
| **NTA-invoerdossier exporteren** | de ingevoerde toestellen en het bewijs, zonder BENG-uitkomst of label |
| **NTA-rekenrapport exporteren** | het rapport van de rekenkern: indicatoren, labelklasse, Bbl-toets, TO-juli, maandwaarden, weggelaten correcties, bronnen en de bijlage Interpretaties |
| **Projectdossier exporteren (ZIP)** | het dossier met projectbestand, kernuitvoer, rekenrapport, bewijs, checklist en manifest (zie [hoofdstuk 7](07-herlabelen-registratie-dossier.md)) |

Het BENG-rapport exporteer je met **Exporteer rapport** op Rekenrapport of Exports, of via het palet (zie [hoofdstuk 4](04-uitvoer.md)).

## API en MCP-server

Andere programma's kunnen de rekenkern zonder de app gebruiken:
- de **HTTP-API** (binary `api`, standaard op `http://127.0.0.1:3007`, met een OpenAPI-document op `/v1/openapi.json`);
- de **MCP-server** (binary `mcp`, voor AI-assistenten zoals Claude Code en Claude Desktop).

Beide bieden dezelfde bewerkingen met dezelfde namen en uitkomsten als de app:
- de projectberekening, en daaruit los de energie per functie, de labelgegevens en de registratiestatus;
- de basisopnames;
- het maatwerkadvies, met maatregelen als JSON-patch;
- herlabelen;
- de interpretatielijst;
- de losse diagnoseroutes.

Zie [nta8800-api.md](../nta8800-api.md) en [nta8800-mcp.md](../nta8800-mcp.md) voor starten, registreren, foutcodes en voorbeelden. Ook hier geldt: de uitkomsten zijn onverifieerd zolang het programma geen BRL 9501-attest heeft.

## EP-Online

Registreren in EP-Online gebeurt met een registratiebestand dat alleen een geattesteerd rekenprogramma maakt (Omgevingsregeling art. 5.11/5.12 lid 2 en 3). Het formaat daarvan is niet openbaar en wordt na attestering via RVO verkregen; deze app maakt het dus niet. Het projectdossier bevat wel `ep-online-gegevensoverzicht.json`: dezelfde gegevens met de veldnamen van het openbare exportschema `EpbdExportTypesV4`, om het geregistreerde label achteraf te controleren (zie hoofdstuk 7).
