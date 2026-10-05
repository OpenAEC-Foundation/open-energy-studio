# NTA 8800 devbuild — commit cd46024

De technische controle en Linux-debugbuild zijn uitgevoerd op de stilstaande commit `cd46024`. De tests, formatcontrole, Clippy en bouw slaagden. De aanvullende robuustheidstest gebruikte 200 mutaties per elk van negen fixtures.

| Controle | Resultaat |
| --- | --- |
| Rust-kern `cargo test --manifest-path crates/nta8800-core/Cargo.toml --quiet` | 755 unittests en 3 integratietests geslaagd |
| Robuustheid `ROBUST_TRIALS=200 cargo test --manifest-path crates/nta8800-core/Cargo.toml --test robustness --quiet` | 3 integratietests geslaagd; 200 mutaties per fixture |
| HTTP/MCP-service `cargo test --manifest-path crates/nta8800-service/Cargo.toml --quiet` | 53 tests geslaagd |
| Frontend `npm test -- --run --maxWorkers=4` | 420 tests in 65 bestanden geslaagd |
| Rust `cargo fmt --check` en `cargo clippy --all-targets -- -D warnings` | Kern en service geslaagd |
| `npm run build` en `npm run tauri build -- --debug --bundles deb` | TypeScript, Vite en Linux-`.deb` geslaagd |
| Pakketmetadata via `dpkg-deb -I` | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Pakket in de repository: `src-tauri/target/debug/bundle/deb/Open Energy Studio_0.1.6-alpha_amd64.deb`. Overdrachtskopie: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/open-energy-studio_0.1.6-alpha_cd46024_debug_amd64.deb`. SHA-256 van beide bestanden: `5c78eed574c7037cb1c70115c3410d226dbb63b027a2ee5c0438077bc9174acf`.

Dit is een technische debugbuild. De extra mutaties tonen foutafhandeling voor de geteste gevallen. Een visuele UI-acceptatie, vergelijking met officiële actuele EDR-referentieuitkomsten en externe BRL 9501-attestering ontbreken. Deze build geeft geen geregistreerd energielabel af.
