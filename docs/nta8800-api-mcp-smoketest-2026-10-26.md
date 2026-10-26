# NTA 8800 HTTP-API en MCP — runtime-smoketest 26 oktober 2026

Bronstand: `1c11c3a2` (programma 2026.44.0, rekenkern 0.5.0) met de MCP-annotaties van issue #9. Gebouwd met `cargo build --manifest-path crates/nta8800-service/Cargo.toml --bins --release` (rustc 1.99.0, Windows x64). De binaries zijn als aparte processen aangeroepen, zonder desktop-app of browser: de API via HTTP op `127.0.0.1:3017` (`--body-limit-mb 1`), de MCP-server via stdio door een eigen JSON-RPC-client. De vorige test is van [4 oktober](nta8800-api-mcp-smoketest-2026-10-04.md).

## HTTP-API

| Verzoek | Antwoord |
|---|---|
| `GET /health` | 200, `{"kernel":"rust","kernelVersion":"0.5.0","status":"ok"}` |
| `GET /v1/version` | 200, API `v1`, rekenkern 0.5.0, NTA 8800:2025+C1:2026, `buildFingerprint` `ceb867200653…` |
| `GET /v1/openapi.json` | 200, OpenAPI 3.1.0, 48 paden |
| `GET /v1/nta8800/capabilities` | 200, `projectCalculationAvailable: true` |
| `POST /v1/nta8800/project/performance`, `training-data/nta8800-project-performance-synthetic.json` als `project` | 200, `calculated_unverified`, BENG 2 = 8,17 kWh/(m²·jr) (gelijk aan 4 oktober) |
| dezelfde fixture zonder `ntaCalculation` | 422, `incomplete`, eerste gat `nta_calculation_block_missing` |
| geen geldige JSON | 400 `invalid_json` |
| onbekend lid naast `project` | 400 `invalid_request_shape` |
| onbekende route | 404 `not_found` |
| `Content-Type: text/plain` | 415 `unsupported_media_type` |
| body van 1,2 MB bij een limiet van 1 MB | 413 `payload_too_large` |
| `Host: evil.example` | 403 `host_not_allowed` |

**Authenticatie.** De API heeft geen authenticatie. Ze luistert standaard alleen op `127.0.0.1` en beantwoordt alleen de hostnamen van dat adres (DNS-rebinding wordt zo geweigerd, zie de laatste regel). Op een ander adres waarschuwt de server bij het starten. Voor gebruik buiten de eigen computer hoort er een proxy met authenticatie voor ([nta8800-api.md](nta8800-api.md)).

## MCP (stdio)

| Stap | Uitkomst |
|---|---|
| `initialize`, protocol `2025-06-18` | server `open-energy-studio-nta8800` 0.1.0, protocol `2025-06-18` |
| `tools/list` | 46 tools, waaronder `calculate_project_performance` en `validate_project`; alle 46 met `readOnlyHint: true` |
| `resources/list` | 20 resources (voorbeeldprojecten, opnamefixtures, interpretatielijst, handleiding, OpenAPI) |
| `calculate_project_performance` met de fixture | `isError: false`, `calculated_unverified`, BENG 2 = 8,17; `structuredContent` is byte voor byte gelijk aan het API-antwoord, en de tekstinhoud gelijk aan `structuredContent` |
| idem zonder `ntaCalculation` | `isError: true`, `incomplete`, eerste gat `nta_calculation_block_missing` |
| onbekende tool | `isError: true` (`unknown_tool`) |

**Rechten en terugdraaien.** Elke tool is een berekening: hij leest zijn argumenten, schrijft geen bestanden, gebruikt geen netwerk en bewaart niets tussen aanroepen. Er is dus niets terug te draaien. De annotaties (`readOnlyHint`, `destructiveHint: false`, `idempotentHint`, `openWorldHint: false`) zeggen dat sinds deze test; een test in `crates/nta8800-service/src/bin/mcp.rs` bewaakt het.

**Claude Desktop en Claude Code.** De registratie staat in [nta8800-mcp.md](nta8800-mcp.md). Een installeerbare bundel (`.mcpb`) en een interactieve Claude-connector zijn er nog niet; de server wordt als binary uit de repository gebouwd.

## Bibliotheek

`cargo run --manifest-path crates/nta8800-operations/Cargo.toml --example external_consumer` rekent het voorbeeldproject (tussenwoning) via het bewerkingenregister door: status 200, `calculated_unverified`, BENG 1 = 51,37, BENG 2 = 80,56, BENG 3 = 22,3 %, indicatief label A+. Zie [nta8800-platforms-en-koppelingen.md](nta8800-platforms-en-koppelingen.md).

## Binaries

SHA-256: `api.exe` = `8c174ce4bd35dae3c0031650667e2255f129029cc068fc471919da4a7bb8046b`; `mcp.exe` = `2c62b8757845b27bc6f0632280bd4e9007ea381a27ea1206ab36f7cb83939d14`.

Dit toetst de transporten, het foutmodel en dezelfde kernuitkomst via API, MCP en bibliotheek. De BENG-waarden zijn interne fixturewaarden; dit is geen referentietoets en geen attestering.
