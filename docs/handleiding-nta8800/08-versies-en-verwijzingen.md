# 8. Versies en verwijzingen

## Versie en reproduceerbaarheid

Elke berekening draagt drie gegevens:
- **Normversie** (`targetNormVersion`): de uitgave waarin is gerekend. Standaard is dat NTA 8800:2025+C1:2026; een project kan ook in 2024, 2023, 2022 of 2020+A1 rekenen (zie [Normversies](10-normversies.md)). Een uitkomst in een oudere uitgave heeft de status `calculated_legacy_edition` en is niet registreerbaar.
- **Kernversie** (`kernelVersion`): de versie van de rekenkern (`KERNEL_VERSION`, gelijk aan de crateversie van `nta8800-core`).
- **Invoervingerafdruk** (`inputFingerprint`): een hash van de projectinvoer. Het maatwerkadvies en de bewaarde basisopname tellen niet mee, omdat ze de energieprestatie van het project niet veranderen.

Het projectbestand bewaart deze gegevens. Het rapport en het dossiermanifest tonen ze. Met dezelfde kernversie en dezelfde invoer geeft een herberekening dezelfde uitkomst.

## Oudere builds bewaren

Herlabelen moet met de rekenkern van de oorspronkelijke berekening (BRL 9500-W §4.2.4, p. 24; 9500-U p. 20). Het programma legt bij elke berekening de kernversie vast. Herlabelen met een andere kern geeft deze meldingen:
- `relabel_kernel_version_differs`;
- `relabel_software_kernel_differs`.

Een nieuwere build kan dus geen labels van een oudere build herlabelen. De certificaathouder bewaart daarom elke gebruikte geattesteerde build minstens 24 maanden na de opname. BRL 9501 §5.3 (p. 9) vraagt daarnaast dat de attesthouder een oude rekenversie minstens drie jaar na een normwijziging beschikbaar houdt. Het versiebeleid (opbouw van het versienummer, uitwisselbaarheid tussen versies, registratie van de rekenkernversie bij RVO) staat in [`docs/nta8800-versiebeheer.md`](../nta8800-versiebeheer.md). Zie ook [`docs/nta8800-attestdossier.md`](../nta8800-attestdossier.md).

## Wijzigingen

Wijzigingen die de uitkomst of de status van opgeslagen projecten veranderen, staan per datum in de [releasenotes](../nta8800-releasenotes.md). Lees ze bij elke nieuwe versie. Dat geldt vooral voor regels die opgeslagen projecten `incomplete` of `invalid` maken.

## Verdere documentatie

| Document | Inhoud |
|---|---|
| [nta8800-verificatiestatus.md](../nta8800-verificatiestatus.md) | stand van de verificatie, onafhankelijke herberekeningen, interpretatielijst |
| [nta8800-dekkingsregister.md](../nta8800-dekkingsregister.md) | dekking per normonderdeel |
| [nta8800-attestdossier.md](../nta8800-attestdossier.md) | eisen uit BRL 9501, BRL 9500, het Bbl en de Omgevingsregeling en de status per eis |
| [nta8800-versiebeheer.md](../nta8800-versiebeheer.md) | versiebeleid volgens BRL 9501 §4.3: versienummers, uitwisselbaarheid, RVO-registratie en bewaartermijnen |
| [nta8800-releasenotes.md](../nta8800-releasenotes.md) | wijzigingen per datum |
| [nta8800-normversies.md](../nta8800-normversies.md) | de uitgaven van NTA 8800 in de kern, met elk schakelpunt en zijn paginaverwijzing |
| [nta8800-vergelijking-openbare-rapporten.md](../nta8800-vergelijking-openbare-rapporten.md) | vergelijking met openbare BENG-rapporten per uitgave |
| [nta8800-werkinstructie.md](../nta8800-werkinstructie.md) | korte werkinstructie voor de projectberekening |
| [nta8800-energieprestatie-keten.md](../nta8800-energieprestatie-keten.md) | de rekenketen van hoofdstuk 5, 9, 10, 13, 14 en 16 |
| [nta8800-maandbehoefte.md](../nta8800-maandbehoefte.md) | hoofdstuk 7 en de maandbehoefte |
| [nta8800-constructies.md](../nta8800-constructies.md) | hoofdstuk 8 en de constructies |
| [nta8800-ventilatie.md](../nta8800-ventilatie.md) | hoofdstuk 11 |
| [nta8800-basisopname.md](../nta8800-basisopname.md) | regels en keuzes van de basisopname ISSO 82.1/75.1 |
| [nta8800-maatwerkadvies.md](../nta8800-maatwerkadvies.md) | maatwerkadvies, sjablonen, NCW, fitcontrole |
| [nta8800-bronnenregister.md](../nta8800-bronnenregister.md) | gebruikte bronnen en open punten |
| [nta8800-voorbeeldproject-smoketest-2026-10-03.md](../nta8800-voorbeeldproject-smoketest-2026-10-03.md) | uitkomsten van de fictieve voorbeeldprojecten |

## Bronnen

De rekenkern is gebouwd op gelicentieerde bronnen:
- NTA 8800:2025+C1:2026, en voor de oudere uitgaven NTA 8800:2024, 2023, 2022 en 2020+A1;
- ISSO 82.1 (7e druk, met erratum), ISSO 75.1 (7e druk), ISSO 82.2 en ISSO 75.2 (3e druk);
- BRL 9500-W en BRL 9500-U van 29 mei 2026 (aangewezen; tekst en paginering gelijk aan de versie van 14 oktober 2025), BRL 9500-MWA-W/U van 24 maart 2026 (in werking per 29 mei 2026) en BRL 9501 van 29 mei 2026; paginaverwijzingen gelden voor deze versies;
- het Besluit bouwwerken leefomgeving (art. 4.149 en 6.29) en de Omgevingsregeling (art. 5.11–5.14, bijlagen IX–Xa); het Besluit en de Regeling energieprestatie gebouwen zijn per 1 januari 2024 ingetrokken;
- het Praktijkhandboek v2.

Deze bronnen staan niet in de repository. Code en documentatie verwijzen alleen naar paragrafen, formules, tabellen en pagina's.
