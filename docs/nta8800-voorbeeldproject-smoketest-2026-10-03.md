# Voorbeeldprojecten via de gebouwde NTA-API — 3 oktober 2026

Bij broncommit `548715b` zijn beide fictieve projecten uit `training-data` via de lokaal gebouwde Rust-API ingediend op `POST /v1/nta8800/project/performance`. Net als de desktopadapter zijn `null`-objectvelden vóór verzending weggelaten; arrayposities bleven behouden.

| Project | HTTP | Kernelstatus | Gaten | Fouten | Indicatieve labelklasse | BENG 2 (kWh/m²·jr) |
| --- | --- | --- | ---: | ---: | --- | ---: |
| `nta8800-example-terraced-dwelling.json` | 200 | `calculated_unverified` | 0 | 0 | A+++ | 17,13 |
| `nta8800-example-office.json` | 200 | `calculated_unverified` | 0 | 0 | A++++ | 39,49 |

De API-proef bevestigt dat de voorbeelden technisch door de projectroute lopen. De voorbeelden zijn fictief. Hun resultaten zijn niet tegen onafhankelijke actuele NTA 8800-referentiegevallen getoetst; de labels zijn niet geregistreerd of geattesteerd. De startpagina meldt dit nu expliciet.
