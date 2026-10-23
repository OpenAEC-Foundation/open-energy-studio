# 9. Bestanden en uitwisseling

## Projectbestanden (`.oes.json`)

**Opslaan.** Met **Opslaan** (Ctrl+S) in de bovenbalk of **Opslaan als** (Ctrl+Shift+S) in het menu Bestand schrijft de app het project als `<projectnaam>.oes.json`.
- In de desktopapp kies je de plaats in een opslagvenster; een project dat al een bestandspad heeft, wordt daar direct overschreven.
- In de browser wordt het bestand gedownload.

De desktopapp slaat veilig op: ze schrijft eerst een tijdelijk bestand naast het project, zet dat pas daarna op de plaats van het oude en bewaart de vorige versie als `<naam>.oes.json.bak`. Valt de stroom uit of is de schijf vol tijdens het opslaan, dan blijft het oude bestand heel. Mislukt het opslaan (bijvoorbeeld een map zonder schrijfrechten), dan meldt de app **Opslaan mislukt** met de reden. Het project blijft dan als gewijzigd gemarkeerd en het tabblad blijft open; kies **Opslaan als** voor een andere plek.

**Afsluiten.** Sluit je het venster (de sluitknop, Alt+F4 of afsluiten vanuit het besturingssysteem) terwijl een project niet-opgeslagen wijzigingen of niet-toegepaste NTA-invoer heeft, dan stelt de app per project dezelfde vraag als bij het sluiten van een tabblad: **Opslaan**, **Niet opslaan** of **Annuleren**. Bij **Annuleren**, of als het opslaan niet lukt, blijft het venster open. In de browser vraagt de browser zelf of je de pagina wilt verlaten.

Het bestand bevat het volledige project, inclusief de NTA-invoer, de basisopname, het maatwerkadvies en de registratiegegevens. Daarnaast bevat het een stempel van de rekenkern: rekenkernversie, normversie en de invoervingerafdruk op het moment van opslaan.

**Openen.** Met **Openen** (Ctrl+O, menu Bestand of welkomstscherm) kies je een `.oes.json`- of `.json`-bestand; in de desktopapp met een bestandsvenster, in de browser met de bestandskiezer van de browser. Het project opent in een nieuw tabblad. Bij het openen:
- vergelijkt de app de opgeslagen stempel met de huidige rekenkern. Is de kern of de normversie anders, dan meldt ze dat; reken dan opnieuw en controleer de verschillen voordat je registreert. Verschilt alleen de invoervingerafdruk, dan is het bestand buiten de app gewijzigd en meldt ze dat ook.
- geeft ze extra opwekkers en extra tapwatersystemen in de opname een id en zet ze koppelingen van bewijs en foto's die nog op positie staan om naar koppelingen op id (zie [hoofdstuk 7](07-herlabelen-registratie-dossier.md));
- zet ze oude herlabelprojecten om naar de huidige vorm (zie [hoofdstuk 7](07-herlabelen-registratie-dossier.md)). Facturen zonder herlabelrol krijgen de rol "Te beoordelen", die nog niet als bewijs telt; de app meldt welke velden nog ontbreken.
- controleert ze de bestandsversie. Een bestand uit een nieuwere versie van Open Energy Studio met een andere hoofdversie (2.x en hoger) opent ze niet; open het met die nieuwere versie. Een nieuwer bestand met dezelfde hoofdversie (1.x) opent ze wel, maar ze noemt de velden die deze versie niet kent: die vallen weg bij opslaan. Een bestand zonder versie geldt als versie 1.0.

**Recente projecten.** Het welkomstscherm toont de projecten die je in de desktopapp hebt geopend of opgeslagen. Die kun je ook na een herstart weer openen: de app onthoudt welke bestanden je via een venster hebt gekozen en leest alleen die. Een project verdwijnt alleen van de lijst als het bestand er niet meer is; bij een andere fout (bijvoorbeeld geen leesrechten) blijft het staan.

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

**UNIEC3 import** leest een `.uniec3`-bestand en opent het als een nieuw project. De import herkent twee soorten bestanden:

- **Een echte export van Uniec 3.4** (relaties met `ParentId`/`ChildId` en een bibliotheek van constructies). Mee komen: de rekenzones met gebruiksoppervlakte, de begrenzingsvlakken met constructie, de kozijnrijen met kozijntype, aantal en oppervlakte, q_v10, de installaties en een deel van de NTA-invoer: de hellingshoek van daken, de omtrek van de vloer op de grond en de belemmering en zonwering per raam. Niet mee komen onder meer koudebruggen, leidingen, bronnen, ventilatiedebieten en de verklaringen uit de productlijst van Uniec. Bevat het bestand meer gebouwen, dan leest de import het gebouw met het laagste nummer en meldt de andere. Deze route is getest op één geanonimiseerde export van een woning met één rekenzone; dat is geen bewijs dat elke export goed gaat.
- **Het eigen invoerconcept** (`.input-draft.uniec3` hierboven): alleen het projectmodel komt mee. Een begrenzing heeft daarin geen hellingshoek; bij een hellend dak zet de import, net als de VABI-import zonder `Hellingshoek`, een punt in het importverslag: vul de hellingshoek in bij de NTA-invoer.

In beide gevallen vul je de rest van de NTA-invoer, de opname en de registratie daarna zelf in; tot dan geeft de rekenkern invoergaten. Uitwisseling met Uniec is nooit verliesvrij: elk punt dat de import aannam of liet liggen, staat met soort en veld in het importverslag, en de registratie vraagt per import een bevestiging dat de ingelezen gegevens zijn gecontroleerd ([hoofdstuk 7](07-herlabelen-registratie-dossier.md)).

Getallen worden met volle precisie geschreven (geen afronding op twee decimalen).

**Uniec-geometrie (Excel, concept)** schrijft de geometrie van een grondgebonden woning in het Excel-formaat van de geometrie-import van Uniec (specificatie van 18-09-2026): de werkbladen *algemene gegevens* en *geometrische gegevens*, met de rekenzones, de dichte delen met R_c, de ramen en deuren met U en g, de vloer op de grond met perimeter en per raam de belemmering en zonwering. Waarden worden afgerond zoals de specificatie voorschrijft (bijvoorbeeld g naar beneden op 0,05). Vraagt het contract iets dat het project niet heeft of dat deze versie nog niet ondersteunt (plaats, bouwjaar, bouwwijze, perimeter, een kruipruimte, een belemmering met maten), dan maakt de app geen bestand en noemt hij elk punt; wat niet meegaat (koudebruggen, ventilatieve koeling) en wat is afgeleid, staat in dezelfde melding. Het bestand is nog niet in Uniec getest: controleer na de import in Uniec de aantallen, oppervlaktes, oriëntaties en constructies. De volledige vertaling staat in [nta8800-uniec-geometrie-export.md](../nta8800-uniec-geometrie-export.md).

## VABI-uitwisseling

**VABI export** schrijft het projectmodel als een EPA-bestand (zip met `project.xml`). Per vlak staan de U-waarde en de R_c van de constructie (ook bij een vlak dat helemaal uit ramen bestaat, dan met oppervlakte 0), de hellingshoek (`Hellingshoek`: 0 voor een vloer of plat dak, 90 voor een wand, de hoek uit de NTA-invoer voor een hellend dak) en de ramen. De koudebruggen staan één keer per rekenzone.

**VABI import** leest een `.epa`-bestand of een losse `project.xml`:
- de tekencodering volgt het bestand (UTF-8, UTF-16 of windows-1252, met of zonder BOM); een bestand met een `DOCTYPE` wordt om veiligheidsredenen geweigerd;
- staan er meer XML-bestanden in het archief, dan wordt `project.xml` gelezen;
- de U-waarde komt uit het deelvlak, anders uit de constructiebibliotheek met dezelfde naam, en pas daarna uit R_c met de R_si van het vlaktype (NTA 8800 tabel C.2); zonder U en R_c wordt het vlak niet aan een constructie gekoppeld. Een U van 0 neemt de import nooit over;
- een vlak met meer dichte deelvlakken wordt in delen gesplitst, elk met een eigen constructie;
- hellingshoek 0 of 180 is horizontaal; een andere hoek van een dak komt in de NTA-invoer (`surfaceTilts`), met het bestand als bron;
- koudebruggen die een oudere export onder elk vlak herhaalde, worden één keer ingelezen.

## Wat de import aanneemt: het importverslag

De UNIEC3- en VABI-import lezen alle getallen streng: een komma of punt als decimaalteken, punten als duizendtallen (`1.234,5` is 1234,5), en geen tekst, NaN of oneindig. Een ontbrekende of ongeldige waarde, een waarde buiten het toegestane bereik, een onbekende code of verwijzing, een dubbel id, meer dan één gebouw, en elk gegeven dat niet kon worden overgenomen, wordt een **punt om na te lopen**. Een ontbrekende q_v10 krijgt de forfaitaire waarde en het kenmerk *aangenomen*. Een ontbrekende ψ krijgt 0,05 W/(m·K), met een punt.

Na het inlezen toont de app hoeveel punten er zijn, per soort, met de eerste paar. De volledige lijst staat in **Projectinformatie › Importverslag**, met per import:
- het hulpmiddel (UNIEC3 of VABI), de bestandsnaam en het tijdstip;
- de SHA-256 van het ingelezen bestand, zodat het bronbestand later aan te wijzen is;
- de **bronversie**: het programma en de NTA 8800-versie die het bestand zelf noemt (bij UNIEC3 uit `meta.json`, bij VABI uit `Algemeen`);
- elk punt met het pad in het project (bijvoorbeeld `zones[0].airTightness.qv10`), de soort, de aangenomen waarde en een toelichting.

**Editie.** De import neemt de NTA 8800-editie en de rekeninstellingen van het andere programma nooit over: alleen het projectmodel komt mee. Elke import heeft daarom een punt bij `normVersion` met de bronversie (of de melding dat het bestand geen versie noemt). Een bestand uit een oudere Uniec 3 is onder een oudere editie gemaakt; het project rekent met de editie die bij de NTA-invoer staat. Controleer die voordat je registreert.

**Niet ingelezen.** Wat in het bestand staat maar waar het programma geen veld voor heeft, verdwijnt niet stil: het importverslag noemt het, per soort, als *Niet overgenomen*:
- bij UNIEC3 de entiteiten die niet gelezen zijn (een andere soort, of een entiteit die niet gekoppeld is waar de import zoekt, zoals een tweede constructie onder één vlak) onder `entities`, en per entiteitsoort de eigenschappen zonder veld (bijvoorbeeld `BEGR`);
- bij VABI per soort element de onderliggende elementen zonder veld (bijvoorbeeld `Rekenzone` of `Verwarming/Opwekker`).

Een export van dit programma zelf geeft bij het weer inlezen geen zulke punten.

Het importverslag is herkomst, geen labelinvoer: het telt niet mee in de vingerafdruk van de labelinvoer (`labelInputSha256`, gelijk aan de rekenkern), zodat herlabelen er niet op reageert.

Een bestand dat helemaal niet te lezen is (geen geldige JSON, een verkeerde opbouw, geen gebouw, een onbekende tekencodering) geeft een melding in het Nederlands of Engels met de naam van het bestand.

## Wat bij export verloren gaat

UNIEC3 en VABI kunnen niet alles bevatten wat het project bevat: de NTA-invoer (`ntaCalculation`), de basisopname, het maatwerkadvies, de registratie, bij installaties de NTA-warmtepompgegevens (`heatingSystems[*].ntaHeatPump`) en een opmerking bij een PV-veld, bij UNIEC3 de lagen van de constructies en een constructie die geen vlak gebruikt (`constructions[unused]`), bij VABI de oriëntatie van een raam dat anders staat dan zijn vlak (een VABI-deelvlak heeft geen eigen oriëntatie), en in beide formaten een COP, EER of rendement van 0: dat is geen waarde die een import overneemt, dus die blijft uit het bestand (bij het weer inlezen staat een aangenomen waarde met een punt in het importverslag). Een EER van 0 bij koeling van de soort *geen* gaat wel mee, want die wordt niet gebruikt. Bij het exporteren toont de app welke onderdelen niet in het bestand komen. Lees je zo'n bestand later weer in, dan staat de ontbrekende NTA-invoer als punt in het importverslag.

**Vloeren op grond.** Geen van beide formaten bevat, voor zover hun bestanden hier bekend zijn, de grondroute van NTA 8800 8.3: de grens van een vlak (grond, kruipruimte, onverwarmde kelder), de perimeter P, de ruimte onder de vloer en een verwarmde kelder met diepte z en wandoppervlak ΣA_T;bw. Een vloer komt als gewoon vlak met zijn R_c in het bestand. De exportmelding noemt daarom `ntaCalculation.groundFloors` apart, en na een import vul je de vloeren op grond opnieuw in bij de NTA-invoer (Gebouw › Schil & ramen › Vloeren op grond). Zonder echte voorbeeldbestanden van Uniec 3 en VABI met deze gegevens kan het programma geen veldnamen kiezen die die pakketten zouden lezen.

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
