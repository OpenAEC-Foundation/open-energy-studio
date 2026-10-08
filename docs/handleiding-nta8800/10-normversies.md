# 10. Normversies

Het programma rekent standaard volgens de aangewezen uitgave, **NTA 8800:2025+C1:2026**. Een project kan ook in een oudere uitgave rekenen: NTA 8800:2024 (met INT-V1:2024), 2023, 2022 of 2020+A1. Dat is bedoeld om een oude berekening na te rekenen of uitkomsten tussen uitgaven te vergelijken. De technische verantwoording, met elk schakelpunt en zijn paginaverwijzing, staat in [`docs/nta8800-normversies.md`](../nta8800-normversies.md).

## Een uitgave kiezen

- **Per project.** Op de stap **Project**, in het blok *Algemeen* van de NTA-invoer, staat de keuzelijst *Uitgave NTA 8800*. Een project zonder gekozen uitgave rekent in de aangewezen uitgave; de keuzelijst toont die dan ook. Een andere keuze gaat, zoals elke wijziging, eerst in het concept en geldt na **Toepassen** in de toepasbalk.
- **Voor nieuwe berekeningen.** Onder *Instellingen › Berekening* kies je de uitgave waarmee **NTA-invoer starten** een nieuw concept begint. Bestaande projecten houden hun eigen uitgave.

## Waarom een oudere uitgave niet registreerbaar is

Alleen een berekening volgens de aangewezen uitgave mag in EP-Online worden geregistreerd. Een uitkomst in een oudere uitgave krijgt daarom de status `calculated_legacy_edition`:
- het NTA-formulier toont bij de keuze "Oudere uitgave: de uitkomst dient alleen ter vergelijking en kan niet worden geregistreerd.";
- de projectstatus op de stap Project, het dashboard onder **Resultaten**, het rekenpaneel, de statusbalk en het NTA-rekenrapport melden "Oudere uitgave — niet voor registratie", met de uitgave waarin is gerekend;
- de registratiecontrole weigert het project met `legacy_edition_not_registrable`. De enige uitzondering is herlabelen in de uitgave van de oorspronkelijke berekening (zie hieronder).

## Invoer die alleen in een uitgave bestaat

Sommige invoer bestaat maar in een deel van de uitgaven. Het formulier toont die velden alleen onder de uitgaven die ze kennen. Voorbeelden:

| Invoer | Uitgaven | Waar |
|---|---|---|
| specifiek werkzame massa en dakoppervlak per ruimte (bijlage AA) | 2024 | Installaties › Koeling |
| keukenleiding ≤ 10 mm (tabel 13.2), afgifte-invoer van 2023 | 2023, 2022, 2020+A1 | Installaties › Warm tapwater, Verwarming, Koeling |
| massa per m² gebruiksoppervlakte (tabel 7.10) | 2022, 2020+A1 | Gebouw › Rekenzones, thermische massa |
| wandhoogte boven maaiveld bij kruipruimte of kelder (8.47) | 2022, 2020+A1 | Gebouw › Schil & ramen, vloeren op grond |
| elektrische boiler met geïsoleerde leidingen | 2022, 2020+A1 | Installaties › Warm tapwater |
| constante-lichtregeling (tabel 14.4) | 2022, 2020+A1 | Installaties › Verlichting |
| installatiejaar van de warmtepomp (9.85) | 2020+A1 | Installaties › Verwarming, opwekking |

Kies je een andere uitgave terwijl zo'n veld een waarde heeft, dan blijft de waarde staan en meldt het formulier dat alleen de genoemde uitgaven deze invoer kennen, met de knop **Verwijderen**. Laat je de waarde staan, dan weigert de rekenkern haar met `route_not_in_edition`.

Omgekeerd kent een oudere uitgave niet elke route van de aangewezen uitgave. Bijlage AA als bewijs van koelcapaciteit bestaat bijvoorbeeld niet in 2023, 2022 en 2020+A1, en een warmtepomp met een aanvoertemperatuur boven 55 °C volgens de forfaitaire tabellen bestaat niet in 2022. Zulke invoer geeft dan ook `route_not_in_edition`, met het pad van het veld en **Ga naar**.

## Andere onderdelen

- **Basisopname.** De opname rekent in de uitgave van het project. Het ISSO-opnameprotocol (7e druk) hoort bij de aangewezen uitgave; in een oudere uitgave is de uitkomst alleen ter vergelijking, met de melding `survey_protocol_edition_differs`.
- **Maatwerkadvies.** Het advies rekent in de uitgave van de basissituatie. Een maatregel die de uitgave verandert, is ongeldig.
- **Herlabelen.** De vergelijking rekent in de uitgave van het **oorspronkelijke** project. Een andere uitgave in het huidige project telt als niet toegestane wijziging.
- **Diagnoses en de constructie-editor.** De rekenhulpen onder Installaties (warmtepompen, ketels, BACS) en de U/R_c-berekening in de constructie-editor rekenen in de uitgave van het project, zowel in de desktopapp als in de browser.
- **Bestanden.** De uitgave staat in het projectbestand (`ntaCalculation.normVersion`). Een project zonder dat veld rekent in de aangewezen uitgave; de invoervingerafdruk blijft daardoor gelijk.
