# Versiebeheer en releases

## Huidige versie

| Programma | Rekenkern (`KERNEL_VERSION`) | Handleiding | Releasetag |
|---|---|---|---|
| **2026.44.0** | **0.5.0** | gestempeld voor rekenkern 0.5.0 | `v2026.44.0` |
| 2026.43.0 | 0.4.0 | gestempeld voor rekenkern 0.4.0 | `v2026.43.0` |
| 0.2.0 | 0.3.0 | gestempeld voor rekenkern 0.3.0 | `oes-v0.2.0-kernel-v0.3.0` |

Programma 2026.44.0 · Rekenkern 0.5.0 · Handleiding gestempeld voor rekenkern 0.5.0. Het programmanummer volgt het OpenAEC-schema jaar.week.volgnummer.

De webapp draait `main`. Wijzigingen na de laatste uitgave staan onder "Onuitgebracht" in de releasenotes.

> **Niet geattesteerd.** Open Energy Studio heeft (nog) geen BRL 9501-attest. Uitkomsten zijn indicatief en niet bruikbaar voor een officieel energielabel, een BENG-toets bij een vergunning of registratie in EP-Online.

**Uitgaven:**
- NTA 8800:2025+C1:2026 (registreerbaar);
- NTA 8800:2024 met INT-V1:2024;
- NTA 8800:2023;
- NTA 8800:2022;
- NTA 8800:2020+A1:2020.

**Toolchain:**
- Rust 1.77.2 (minimum voor de rekenkern; de desktop-app en de API/MCP-service vragen 1.88);
- Node.js 22;
- Tauri 2.

## Regels

- **Kernversie:** `KERNEL_VERSION` is de versie van `crates/nta8800-core`, en elk resultaat draagt hem. Een wijziging die uitkomsten verandert, vraagt een nieuwe kernversie; de gate bewaakt dat.
- **Releasenotes:**
  - ingedeeld per kernversie;
  - nieuwe items komen onder `## Onuitgebracht`;
  - een item zonder rekeneffect eindigt op `(geen rekenwijziging)`.
- **Handleiding:** gestempeld voor een kernversie. De gate controleert dat de stempel gelijk is aan `KERNEL_VERSION`.
- **Release:** `scripts/release-nta.sh` doet het volgende:
  - draait de gate en bouwt de pakketten;
  - exporteert de handleiding;
  - schrijft het leveringsdocument, het manifest en `SHA256SUMS`;
  - maakt een lokale tag `oes-v<programma>-kernel-v<kern>`.

  Het script pusht nooit. Met `--dry-run` doet het een proefrun zonder commit of tag.
- **Na het pushen van de tag:** GitHub Actions (`release.yml`) controleert dat de tag bij `package.json` en `KERNEL_VERSION` hoort en draait de volledige gate. Daarna bouwt het de installatiepakketten voor Linux (`.deb`, `.rpm`, AppImage), macOS (`.dmg`) en Windows (`-setup.exe`, `.msi`) en hangt ze met `SHA256SUMS-installers.txt` aan de GitHub-release van de tag. Een bestaande release houdt zijn tekst; anders maakt de workflow een concept dat je zelf publiceert.
- **CI:** elke push en pull request naar `main` en `nta8800-kernel` draait dezelfde gate (`ci.yml`).

Details: [docs/nta8800-versiebeheer.md](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/nta8800-versiebeheer.md) en de [releasenotes](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/nta8800-releasenotes.md).
