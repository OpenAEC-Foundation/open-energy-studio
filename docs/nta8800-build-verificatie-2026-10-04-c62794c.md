# NTA 8800 devbuild — commit c62794c

De schone werkboom op `c62794c` is gebouwd met `npm run tauri build -- --debug --bundles deb`. De commit bleef tijdens de tests en bouw gelijk.

| Controle | Resultaat |
| --- | --- |
| Rust-kern `cargo test --manifest-path crates/nta8800-core/Cargo.toml --quiet` | 753 geslaagd, 0 mislukt |
| HTTP/MCP-service `cargo test --manifest-path crates/nta8800-service/Cargo.toml --quiet` | 53 geslaagd, 0 mislukt |
| Volledige frontendtest `npm test -- --run --maxWorkers=4` | 399 geslaagd in 63 bestanden, 0 mislukt |
| TypeScript, Vite en Tauri debug-`.deb` | Bouw geslaagd |
| Pakketmetadata | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Pakket in de repository: `src-tauri/target/debug/bundle/deb/Open Energy Studio_0.1.6-alpha_amd64.deb`. Overdrachtskopie: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/open-energy-studio_0.1.6-alpha_c62794c_debug_amd64.deb`. SHA-256 van beide: `fcab82bf7b511844794ad20e8883032700888ee041bf690a310cc607b5cd78b2`.

Dit is een technische debugbuild. De UI is niet visueel geaccepteerd; officiële actuele referentie-uitkomsten en externe BRL 9501-attestering ontbreken. De build geeft geen geregistreerd energielabel af.
