# Technische buildcontrole op `dba3ca2`

Datum: 4 oktober 2026. Broncommit: `dba3ca2` (`nta8800-kernel`). Doeluitgave: NTA 8800:2025+C1:2026.

De referentievergelijking accepteert nu ook `labelPrimaryFossil` (`kWh/m2.year`) en `labelRenewableShare` (`%`). Zij vergelijken de afzonderlijke labelscenario-indicatoren, die van BENG 2 en 3 kunnen verschillen. Een gerichte synthetische kerntest controleerde match, mismatch en een verkeerde eenheid. `cargo fmt`, Clippy voor alle Rust-targets met waarschuwingen als fouten, `cargo check` voor Tauri, `cargo build` voor alle servicebinaries en `npm run tauri build -- --debug --bundles deb` slaagden. De volledige kern-, service- en frontendtests zijn op deze commit niet opnieuw uitgevoerd; de vorige volledige teststand staat in de verificatiestatus.

Het pakket is een Linux amd64 **debug/devbuild** met pakketnaam `open-energy-studio` en versie `0.1.6-alpha`. De bestanden zijn uit dezelfde bronstand gebouwd en met SHA-256 vastgelegd.

| Artefact | Pad | SHA-256 |
| --- | --- | --- |
| Desktop-devbuild | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-dev_dba3ca2_linux-amd64.deb` | `4e58475dc3f6e05ae6da2734d97c9414246a6fe36561ba6ef7eab608b39a9bea` |
| Referentiegate | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_dba3ca2_linux-amd64` | `3ccf79aa6f8522ea173d26fd9c83d3239e024a7288fad2b73085f185c8446fea` |
| HTTP-API | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-api_dba3ca2_linux-amd64` | `92d5822e9fcb985785af219562c6343786a1e0cc46388c00c5a7bfc89e11d093` |
| MCP-server | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-mcp_dba3ca2_linux-amd64` | `9dd23d2bc85d3641f4f2f0e6d8133b94df591caf31336fceda840d435f053882` |

De gerichte test bewijst de extractie en vergelijking, geen onafhankelijke normatieve juistheid. Visuele UI-acceptatie, actuele officiële W/U-referentiegevallen en externe BRL 9501-attestering staan open.
