# Voorbeeldprojecten via de gebouwde NTA-API — 3 oktober 2026

Bij broncommit `548715b` zijn beide fictieve projecten uit `training-data` via de lokaal gebouwde Rust-API ingediend op `POST /v1/nta8800/project/performance`. Net als de desktopadapter zijn `null`-objectvelden vóór verzending weggelaten; arrayposities bleven behouden.

| Project | HTTP | Kernelstatus | Gaten | Fouten | Indicatieve labelklasse | BENG 2 (kWh/m²·jr) |
| --- | --- | --- | ---: | ---: | --- | ---: |
| `nta8800-example-terraced-dwelling.json` | 200 | `calculated_unverified` | 0 | 0 | A+++ | 17,13 |
| `nta8800-example-office.json` | 200 | `calculated_unverified` | 0 | 0 | A++++ | 39,49 |

De API-proef bevestigt dat de voorbeelden technisch door de projectroute lopen. De voorbeelden zijn fictief. Hun resultaten zijn niet tegen onafhankelijke actuele NTA 8800-referentiegevallen getoetst; de labels zijn niet geregistreerd of geattesteerd. De startpagina meldt dit nu expliciet.

## Herhaling na de validatieregels — 4 oktober 2026

Na de merge van de validatieregels zijn beide voorbeelden aangepast (`3eec124`): één koudebrugmethode, opgegeven verticale leidingen, ventilatie volgens hoofdstuk 11 en berekend warm tapwater in plaats van een opgegeven gebruik. Ze zijn opnieuw ingediend, ook na de correctie dat ΔU_for alleen in H_D telt (merge van de reviewfixes), via `POST /v1/nta8800/project/performance`.

| Project | Kernelstatus | Gaten | Waarschuwingen | Indicatieve labelklasse | BENG 2 (kWh/m²·jr) | Aandeel hernieuwbare energie |
| --- | --- | ---: | ---: | --- | ---: | ---: |
| `nta8800-example-terraced-dwelling.json` | `calculated_unverified` | 0 | 0 | A+ | 76,56 | 23,2 % |
| `nta8800-example-office.json` | `calculated_unverified` | 0 | 0 | A+++ | 48,33 | 49,6 % |

De tussenwoning stijgt vooral doordat het tapwater nu wordt berekend met het tabelrendement van de HR-combiketel. Dat is realistisch voor een woning met een gasketel. De vorige waarde van 17,13 berustte op een opgegeven tapwatergebruik dat fysiek niet kon (η > 1).
