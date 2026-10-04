# 8. Versies en verwijzingen

## Versie en reproduceerbaarheid

Elke berekening draagt drie gegevens:
- **Normversie** (`targetNormVersion`): NTA 8800:2025+C1:2026.
- **Kernversie** (`kernelVersion`): de versie van de rekenkern (`KERNEL_VERSION`, gelijk aan de crateversie van `nta8800-core`).
- **Invoervingerafdruk** (`inputFingerprint`): een hash van de projectinvoer. Het maatwerkadvies en de bewaarde basisopname tellen niet mee, omdat ze de energieprestatie van het project niet veranderen.

Het projectbestand bewaart deze gegevens. Het rapport en het dossiermanifest tonen ze. Met dezelfde kernversie en dezelfde invoer geeft een herberekening dezelfde uitkomst.

## Oudere builds bewaren

Herlabelen moet met de rekenkern van de oorspronkelijke berekening (BRL 9500-W §4.2.4, p. 24; 9500-U p. 20). Het programma legt bij elke berekening de kernversie vast. Herlabelen met een andere kern geeft deze meldingen:
- `relabel_kernel_version_differs`;
- `relabel_software_kernel_differs`.

Een nieuwere build kan dus geen labels van een oudere build herlabelen. De certificaathouder bewaart daarom elke gebruikte geattesteerde build minstens 24 maanden na de opname. BRL 9501 §5.3 vraagt daarnaast dat een oude rekenversie beschikbaar blijft na een normwijziging. Zie [`docs/nta8800-attestdossier.md`](../nta8800-attestdossier.md).

## Wijzigingen

Wijzigingen die de uitkomst of de status van opgeslagen projecten veranderen, staan per datum in de [releasenotes](../nta8800-releasenotes.md). Lees ze bij elke nieuwe versie. Dat geldt vooral voor regels die opgeslagen projecten `incomplete` of `invalid` maken.

## Verdere documentatie

| Document | Inhoud |
|---|---|
| [nta8800-verificatiestatus.md](../nta8800-verificatiestatus.md) | stand van de verificatie, onafhankelijke herberekeningen, interpretatielijst |
| [nta8800-dekkingsregister.md](../nta8800-dekkingsregister.md) | dekking per normonderdeel |
| [nta8800-attestdossier.md](../nta8800-attestdossier.md) | eisen uit BRL 9501, BRL 9500, Bep en Regeling en de status per eis |
| [nta8800-releasenotes.md](../nta8800-releasenotes.md) | wijzigingen per datum |
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
- NTA 8800:2025+C1:2026;
- ISSO 82.1 (7e druk, met erratum), ISSO 75.1 (7e druk), ISSO 82.2 en ISSO 75.2 (3e druk);
- BRL 9500-W en BRL 9500-U in de versie van 14 oktober 2025 (bindend verklaard, nog niet vastgesteld), en BRL 9500-MWA-W/U van 19 juni 2024; paginaverwijzingen gelden voor deze versies;
- het Besluit en de Regeling energieprestatie gebouwen;
- het Praktijkhandboek v2.

Deze bronnen staan niet in de repository. Code en documentatie verwijzen alleen naar paragrafen, formules, tabellen en pagina's.
