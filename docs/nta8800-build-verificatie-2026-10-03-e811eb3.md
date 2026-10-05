# NTA 8800 devbuild — commit e811eb3

De schone werkboom op `e811eb3` is gebouwd met `npm run tauri build -- --debug --bundles deb`. De commit bleef tijdens de bouw gelijk.

| Controle | Resultaat |
| --- | --- |
| Rust-kern `cargo test --manifest-path crates/nta8800-core/Cargo.toml --quiet` | 747 geslaagd, 0 mislukt |
| HTTP/MCP-service `cargo test --manifest-path crates/nta8800-service/Cargo.toml --quiet` | 53 geslaagd, 0 mislukt |
| Gerichte frontendtests voor relabelen en maatwerkadvies | 30 geslaagd, 0 mislukt |
| Volledige frontendtestrun `npm test -- --run --maxWorkers=4` | 380 geslaagd in 61 bestanden, 0 mislukt |
| TypeScript, Vite en Tauri debug-`.deb` | Bouw geslaagd |
| Pakketmetadata | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Pakket in de repository: `src-tauri/target/debug/bundle/deb/Open Energy Studio_0.1.6-alpha_amd64.deb`. Overdrachtskopie: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/open-energy-studio_0.1.6-alpha_e811eb3_debug_amd64.deb`. SHA-256 van beide: `74ee176201a2626f2e048c46cb37d83ea17b4204c6cab99d44d31022937de861`.

Dit is een technische debugbuild. De UI is niet visueel geaccepteerd; officiële actuele referentie-uitkomsten en externe BRL 9501-attestering ontbreken. De build geeft geen geregistreerd energielabel af.
