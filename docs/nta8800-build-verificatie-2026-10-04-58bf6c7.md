# Technische NTA-buildcontrole op `58bf6c7`

Datum: 4 oktober 2026. Broncommit: `58bf6c7` (`nta8800-kernel`). Doeluitgave: NTA 8800:2025+C1:2026.

`scripts/verify-nta.sh` slaagde met `NTA_SKIP_MSRV=1` en een synthetisch referentiegeval plus passend dekkingsplan: 760 kerntests en drie integratietests, 56 servicetests en vier CLI-tests, rustfmt, Clippy met waarschuwingen als fouten, de geplande numerieke referentiegate, Tauri-check, TypeScript en 452 frontendtests in 69 bestanden. De geplande gate gaf `plannedCoveragePassed=true` en `referenceVerified=false`. Een aanvankelijke run faalde op vier ontbrekende NL/EN-labels voor nieuwe referentiefouten; na toevoeging slaagde de volledige herhaling. De MSRV-test is in deze run overgeslagen.

`npm run tauri build -- --debug --bundles deb` slaagde daarna. Het pakket is een Linux amd64 **debug/devbuild**, met pakketnaam `open-energy-studio` en versie `0.1.6-alpha`. De gebundelde frontend en Rust-desktop komen uit de genoemde broncommit. Een visuele desktopacceptatie is hiermee niet uitgevoerd.

| Artefact | Pad | SHA-256 |
| --- | --- | --- |
| Linux desktop-devbuild | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-dev_58bf6c7_linux-amd64.deb` | `cee3f92c10c55884e7a5c9e29958fc609904dd5dacf8493a3cb8d4a0e2f3a089` |
| Geplande referentie-CLI | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_58bf6c7_linux-amd64` | `bd488f88075f9e607663bd0a3514faea96e484dd5f2c489f3cf6deaeb15b496f` |

De verwachte BENG-waarde in de proef komt uit een interne synthetische fixture. Deze controle bevestigt de technische bouw en het gedrag van de gate, niet normconformiteit, onafhankelijke actuele EDR-dekking of BRL 9501-attestering.
