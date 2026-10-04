# NTA 8800 devbuild — commit af69ada

De interne controle en Linux-debugbuild zijn uitgevoerd op de stilstaande commit `af69ada`. Deze stand bevat de gedeelde regel die BENG-getallen achterhoudt wanneer de Rust-kern invoer weigert, ook in rapporten en IFC, en extra labels voor kernmeldingen.

| Controle | Resultaat |
| --- | --- |
| Rust-kern `cargo test --manifest-path crates/nta8800-core/Cargo.toml --quiet` | 756 unittests en 3 integratietests geslaagd |
| HTTP/MCP-service `cargo test --manifest-path crates/nta8800-service/Cargo.toml --quiet` | 54 tests geslaagd |
| Frontend `npm test -- --run --maxWorkers=4` | 451 tests in 69 bestanden geslaagd |
| `npm run build` en `npm run tauri build -- --debug --bundles deb` | TypeScript, Vite en Linux-`.deb` geslaagd |
| Pakketmetadata via `dpkg-deb -f` | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Pakket in de repository: `src-tauri/target/debug/bundle/deb/Open Energy Studio_0.1.6-alpha_amd64.deb`. Overdrachtskopie: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/open-energy-studio_0.1.6-alpha_af69ada_debug_amd64.deb`. SHA-256 van beide bestanden: `cbd6e1f0bc8084009f29f0ad889f12d9aed8d090c7e4cafa106dc4f76f95c60b`.

Vite meldde een grote hoofdbundel (ongeveer 2,00 MB) en gemengde statische/dynamische import van de Tauri-core. Dit blokkeerde de bouw niet; de opstartprestatie is hiermee niet gemeten.

Dit is een technische debugbuild. Een visuele UI-acceptatie, vergelijking met officiële actuele EDR-referentieuitkomsten en externe BRL 9501-attestering ontbreken. Deze build geeft geen geregistreerd energielabel af.
