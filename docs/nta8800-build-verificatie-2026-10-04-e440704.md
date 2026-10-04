# NTA 8800 devbuild — commit e440704

De technische controle en Linux-debugbuild zijn uitgevoerd op de stilstaande commit `e440704`. Deze stand bevat de aanvullende eindigheidscontroles op kern-, HTTP/MCP- en desktoproutes en de bijgewerkte NTA-handleiding.

| Controle | Resultaat |
| --- | --- |
| Rust-kern `cargo test --manifest-path crates/nta8800-core/Cargo.toml --quiet` | 756 unittests en 3 integratietests geslaagd |
| Aanvullende robuustheid `ROBUST_TRIALS=100 cargo test --manifest-path crates/nta8800-core/Cargo.toml --test robustness --quiet` | 3 integratietests geslaagd; negen project-/opnamefixtures en een maatwerkadviesfixture |
| HTTP/MCP-service `cargo test --manifest-path crates/nta8800-service/Cargo.toml --quiet` | 54 tests geslaagd |
| Frontend `npm test -- --run --maxWorkers=4` | 420 tests in 65 bestanden geslaagd |
| Rust `cargo fmt --check` en `cargo clippy --all-targets -- -D warnings` | Kern en service geslaagd |
| `npm run build` en `npm run tauri build -- --debug --bundles deb` | TypeScript, Vite en Linux-`.deb` geslaagd |
| Pakketmetadata via `dpkg-deb -f` | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Pakket in de repository: `src-tauri/target/debug/bundle/deb/Open Energy Studio_0.1.6-alpha_amd64.deb`. Overdrachtskopie: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/open-energy-studio_0.1.6-alpha_e440704_debug_amd64.deb`. SHA-256 van beide bestanden: `ae5eed0372e8fe9153f1e8b5f9bdbd3b5cfe37dc319615bd176e415c25d8f555`.

Dit is een technische debugbuild. De mutaties testen foutafhandeling voor de genoemde gevallen; zij bewijzen geen normatieve juistheid. Een visuele UI-acceptatie, vergelijking met officiële actuele EDR-referentieuitkomsten en externe BRL 9501-attestering ontbreken. Deze build geeft geen geregistreerd energielabel af.
