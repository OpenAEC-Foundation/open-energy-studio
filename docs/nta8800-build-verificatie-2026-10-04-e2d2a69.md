# Technische buildcontrole op `e2d2a69`

Datum: 4 oktober 2026. Broncommit: `e2d2a69` (`nta8800-kernel`). Doeluitgave: NTA 8800:2025+C1:2026.

Deze bronstand voegt de `coolingMonth`-meetpaden voor de hoofdstuk-10-koelketen aan het referentieharnas toe. De gerichte synthetische Rust-test controleerde een berekende koelmaand, ontbrekende koeling, een afwijking en een ongeldig maandpad. `cargo fmt`, Clippy voor alle Rust-kern-targets met waarschuwingen als fouten, `cargo build --manifest-path crates/nta8800-service/Cargo.toml --bins` en `npm run tauri build -- --debug --bundles deb` slaagden. De laatste volledige kern-, service-, MSRV- en frontendrun was op de voorafgaande bronstand `dba3ca2`; die is niet als test van deze wijziging opgevoerd.

Het pakket is een Linux amd64 **debug/devbuild** (`open-energy-studio`, `0.1.6-alpha`). De bestanden zijn uit dezelfde bronstand gebouwd.

| Artefact | Pad | SHA-256 |
| --- | --- | --- |
| Desktop-devbuild | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-dev_e2d2a69_linux-amd64.deb` | `345f4f214cdc412cbb58ac9c223b1cfa932458636c14592adb43d9a6783f57b6` |
| Referentiegate | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_e2d2a69_linux-amd64` | `9f1a60e37c69a0ef833092b2f6bf78bb4632d4fc4dc943096787eb389c30407d` |
| HTTP-API | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-api_e2d2a69_linux-amd64` | `df65d2e2fde1d5990fd7ad66ab841713487db332a0e05ef346dfcbbd45c3fecd` |
| MCP-server | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-mcp_e2d2a69_linux-amd64` | `f51d957574b8adbfeff5e76b4b22d094a681ddf973d0effbefff200bd69730b6` |

De testwaarden komen uit dezelfde interne synthetische projectfixture en zijn geen onafhankelijke normuitkomsten. Visuele UI-acceptatie, actuele officiële W/U-referenties en externe BRL 9501-attestering staan open.
