# NTA 8800 devbuild — commit 7ac7e52

De volledige interne tests en Linux-debugbuild zijn uitgevoerd op de stilstaande commit `7ac7e52`. Deze stand bevat de projectimportcontrole voor expliciet ongeldige `qv10`-waarden en de eerder geteste gedeelde regel voor achtergehouden BENG-getallen.

| Controle | Resultaat |
| --- | --- |
| Rust-kern `cargo test --manifest-path crates/nta8800-core/Cargo.toml --quiet` | 756 unittests en 3 integratietests geslaagd |
| HTTP/MCP-service `cargo test --manifest-path crates/nta8800-service/Cargo.toml --quiet` | 54 tests geslaagd |
| Frontend `npm test -- --run --maxWorkers=4` | 452 tests in 69 bestanden geslaagd |
| `npm run tauri build -- --debug --bundles deb` | TypeScript, Vite, Rust-desktop en Linux-`.deb` geslaagd |
| Pakketmetadata via `dpkg-deb -f` | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Pakket in de repository: `src-tauri/target/debug/bundle/deb/Open Energy Studio_0.1.6-alpha_amd64.deb`. Overdrachtskopie: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/open-energy-studio_0.1.6-alpha_7ac7e52_debug_amd64.deb`. SHA-256 van beide bestanden: `989d051bcc816d6bf1edb1ef64c833dea76f87595ebeccc0e2acb6841b83e923`.

Vite meldde een grote hoofdbundel (ongeveer 2,00 MB) en gemengde statische/dynamische import van de Tauri-core. De bouw slaagde; de opstartprestatie is niet gemeten. De frontendtestrun meldde dat jsdom `HTMLCanvasElement.getContext()` niet implementeert, maar alle tests slaagden.

Dit is een technische debugbuild. Een visuele UI-acceptatie, vergelijking met officiële actuele EDR-referentieuitkomsten en externe BRL 9501-attestering ontbreken. Deze build geeft geen geregistreerd energielabel af.
