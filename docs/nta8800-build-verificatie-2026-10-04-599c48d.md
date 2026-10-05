# Technische buildcontrole op `599c48d`

Datum: 4 oktober 2026. Broncommit: `599c48d` (`nta8800-kernel`). Doeluitgave: NTA 8800:2025+C1:2026.

De volledige Rust-kern (761 tests en drie integratietests) en service (57 tests en vijf CLI-tests) slaagden. Rust-formatcontrole, Clippy met waarschuwingen als fouten, TypeScript en de gerichte NL/EN-kerncodelabeltest slaagden. De gerichte labelvergelijking slaagde ook onder Rust 1.77.2. Een HTTP-route-test controleerde dat de klasse als `indicativeLabelClass` verschijnt met `referenceVerified=false` en `attestStatus=unattested`. De gebouwde batch-CLI gaf met één synthetische numerieke BENG-post en een passende indicatieve klasse `plannedCoveragePassed=true`; dit is geen officiële referentiecase.

`npm run tauri build -- --debug --bundles deb` en `cargo build --manifest-path crates/nta8800-service/Cargo.toml --bins` slaagden. Het Linux amd64 pakket is een **debug/devbuild** (`open-energy-studio`, `0.1.6-alpha`).

| Artefact | Pad | SHA-256 |
| --- | --- | --- |
| Desktop-devbuild | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-dev_599c48d_linux-amd64.deb` | `214fae5a5b819066b18c024f01b578f43142213eac520546217226a7d9cb798d` |
| Referentiegate | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_599c48d_linux-amd64` | `67a3fd3d7659288c6b06274bb05d7b70b9f8f4e63346ef645b3293f258e2dc8b` |
| HTTP-API | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-api_599c48d_linux-amd64` | `de1160c6d42ef079642846c1b6e9512e68bc892757b678af1e5e016a4c6bced9` |
| MCP-server | `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-mcp_599c48d_linux-amd64` | `935ed6160bcdfdcb60ed29964ef9a0549748fa38d59f0a50db0c80528f2ea3bd` |

De UI is niet visueel geaccepteerd. De verwachte waarden zijn intern synthetisch; officiële actuele W/U-uitkomsten, onafhankelijke brontoets en BRL 9501-attestering blijven open.
