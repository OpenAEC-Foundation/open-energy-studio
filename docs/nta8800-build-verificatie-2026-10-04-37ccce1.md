# NTA 8800 devbuild — broncommit 37ccce1

Deze build voegt geselecteerde H9-maandtermen toe aan het bestaande referentievergelijkingsharnas. De technische controle is op dezelfde stilstaande broncommit uitgevoerd.

| Controle | Resultaat |
| --- | --- |
| Rust-formatcontrole en Clippy van de gewijzigde kern, alle targets, waarschuwingen als fout | geslaagd |
| Rust-kern | 760 unittests en 3 integratietests geslaagd |
| HTTP/MCP-service | 56 tests geslaagd |
| `cargo build --manifest-path crates/nta8800-service/Cargo.toml --bins` | API en MCP gebouwd |
| `npm run tauri build -- --debug --bundles deb` | TypeScript, Vite, Rust-desktop en Linux-`.deb` geslaagd |
| Pakketmetadata | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Overdrachtsbestanden in `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/`:

| Bestand | SHA-256 |
| --- | --- |
| `open-energy-studio_0.1.6-alpha_37ccce1_debug_amd64.deb` | `8a35a4e2249176cd8e99682d5e251c09fb7162b7b31afa654388330ba35af08f` |
| `oes-nta8800-api_37ccce1_linux-amd64` | `3b127333a961d374abf72f5cc33c12ef1eb9868efce9354be18d11fc1bd6cc59` |
| `oes-nta8800-mcp_37ccce1_linux-amd64` | `313ec27321f8c3d708b2cfb6c029a824bf65aae9d733f1c61d32da8b36cd6208` |

Vite meldde dezelfde waarschuwingen voor de grote hoofdbundel en gemengde Tauri-import. De frontendcode is ongewijzigd; de laatste volledige frontendtestrun (452 tests) is van `7ac7e52`. Visuele UI-acceptatie, actuele officiële EDR-referentieuitkomsten en externe BRL 9501-attestering ontbreken. De nieuwe H9-vergelijkingstests gebruiken synthetische verwachtingen en geven geen normatieve vrijgave.
