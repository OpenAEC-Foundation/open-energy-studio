# NTA 8800 devbuild — commit fc5a8a5

De schone werkboom op commit `fc5a8a5` is opnieuw gebouwd met `npm run tauri:build -- --debug --bundles deb`. De eerste bouw liep samen met een nieuwe commit; daarom is de bouw vanaf de daarna schone werkboom herhaald. Alleen de tweede bouw geldt voor dit dossier.

| Controle | Resultaat |
| --- | --- |
| Rust-kern `cargo test --lib --quiet` | 703 geslaagd, 0 mislukt |
| HTTP/MCP-service `cargo test --quiet` | 53 geslaagd, 0 mislukt |
| Gerichte frontendtests voor NTA-prestatiepaneel en energierapport | 10 geslaagd, 0 mislukt |
| TypeScript, Vite en Tauri debug-`.deb` | Bouw geslaagd |
| Pakketmetadata | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Pakket in de repository: `src-tauri/target/debug/bundle/deb/Open Energy Studio_0.1.6-alpha_amd64.deb`. Overdrachtskopie: `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/open-energy-studio_0.1.6-alpha_fc5a8a5_debug_amd64.deb`. SHA-256 van beide: `af46a866c80cfdfec647c51e0ae16047c7d335fbc259bad969d7a20ec34f3c02`.

Dit is een technische debugbuild. De UI is niet visueel geaccepteerd; officiële actuele referentie-uitkomsten en externe BRL 9501-attestering ontbreken. De build geeft geen geregistreerd energielabel af.
