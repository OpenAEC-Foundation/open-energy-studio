# NTA 8800 HTTP-API en MCP — runtime-smoketest 4 oktober 2026

Bronstand: `c6d8a01`. De Rust-kern en service verschillen niet van de volledig geteste bronstand `e440704`. `cargo build --manifest-path crates/nta8800-service/Cargo.toml --bins --quiet` slaagde. De gebouwde binaries zijn daarna als aparte processen via HTTP en MCP-stdio aangeroepen.

| Invoer | HTTP-API `/v1/nta8800/project/performance` | MCP `calculate_project_performance` |
| --- | --- | --- |
| `training-data/nta8800-project-performance-synthetic.json` in de vereiste `project`-envelope | HTTP 200, `calculated_unverified`, geen gaten, BENG 2 = 8,17 kWh/(m²·jr) | `isError=false`, dezelfde status en BENG 2 = 8,17 |
| Dezelfde fixture zonder `ntaCalculation` | HTTP 422, `incomplete`, eerste gat `nta_calculation_block_missing`, geen `performance` | `isError=true`, dezelfde status en eerste gat, geen `performance` |

De MCP-stdio-handshake (`initialize` met protocol `2025-06-18`) en `tools/list` slaagden; de server adverteerde 36 tools, waaronder `calculate_project_performance` en `validate_project`. De API meldde via `/health` `{"kernel":"rust","status":"ok"}`. De tijdelijke API op `127.0.0.1:3017` is na de controle gestopt.

SHA-256 van de gebouwde binaries: `api` = `fde22db4f64a50e01f200fd6ce01cb9a163d8053e3eccc91405ec8a7cb2f6c07`; `mcp` = `3d2d4cc430246bf47054b2ae45d13aff47ba98c87852240fabae8d6f653440b3`.

Dit toetst de transporten en dezelfde kernuitkomst voor twee invoervarianten. De verwachte BENG-waarde is een interne fixturewaarde; dit is geen officiële actuele EDR-referentietoets of attestering.
