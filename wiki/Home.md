# Open Energy Studio — wiki

Open Energy Studio is open-source software voor de energieprestatie van woningen en utiliteitsgebouwen volgens **NTA 8800**. Het doel is een volledig, controleerbaar rekenprogramma dat volgens **BRL 9501** geattesteerd kan worden.

> **Status:** nog niet geattesteerd. Zie [[BRL 9501 status]].

## Versie

| Programma | Rekenkern (`KERNEL_VERSION`) | Handleiding | Releasetag |
|---|---|---|---|
| **2026.44.0** | **0.5.0** | gestempeld voor rekenkern 0.5.0 | `v2026.44.0` |
| 2026.43.0 | 0.4.0 | gestempeld voor rekenkern 0.4.0 | `v2026.43.0` |
| 0.2.0 | 0.3.0 | gestempeld voor rekenkern 0.3.0 | `oes-v0.2.0-kernel-v0.3.0` |

Programma 2026.44.0 · Rekenkern 0.5.0 · Handleiding gestempeld voor rekenkern 0.5.0. Het programmanummer volgt het OpenAEC-schema jaar.week.volgnummer.

De webapp draait `main`. Wijzigingen na de laatste uitgave staan onder "Onuitgebracht" in de releasenotes.

> **Niet geattesteerd.** Open Energy Studio heeft (nog) geen BRL 9501-attest. Uitkomsten zijn indicatief en niet bruikbaar voor een officieel energielabel, een BENG-toets bij een vergunning of registratie in EP-Online.

**Uitgaven van NTA 8800:**
- NTA 8800:2025+C1:2026: de standaard en de enige registreerbare uitgave;
- NTA 8800:2024 met INT-V1:2024, NTA 8800:2023, NTA 8800:2022 en NTA 8800:2020+A1:2020: oudere uitgaven, niet registreerbaar.

Zie [[Normversies]].

## Voor adviseurs

- [[Installatie]] en [[Snelstart]]
- [[Werkstroom]]: de stappen van het programma
- [[Basisopname]], [[Projectberekening]], [[Resultaten en rapport]]
- [[Maatwerkadvies]], [[Herlabelen]]
- [[Registratie, dossier en EP-Online|Registratie-dossier-en-EP-Online]], [[Bron en bewijs]]
- [[Import en export]]
- [[Veelgestelde vragen]], [[Bekende beperkingen]]

## Voor ontwikkelaars en de attesteringsinstelling

- [[Architectuur]], [[API en MCP]]
- [[Verificatie en testen]], [[Testprotocol]]
- [[Versiebeheer en releases]], [[BRL 9501 status]]

De volledige handleiding staat in de repository: [handleiding NTA 8800](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/handleiding-nta8800/index.md).
