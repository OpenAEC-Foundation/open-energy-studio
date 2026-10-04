# NTA 8800 devbuild — broncommit 47c84f3

Op broncommit `47c84f3` is het begrensde referentievergelijkingsharnas voor BENG 1/2/3 en TOjuliMax toegevoegd. De tests, adapters en Linux-debugbuild zijn daarna uitgevoerd zonder tussentijdse bronwijzigingen.

| Controle | Resultaat |
| --- | --- |
| Rust-formatcontrole voor kern en service | geslaagd |
| Clippy voor kern en service, alle targets, waarschuwingen als fout | geslaagd |
| Rust-kern | 758 unittests en 3 integratietests geslaagd |
| HTTP/MCP-service | 55 tests geslaagd |
| `npm run build` | TypeScript en Vite geslaagd |
| `npm run tauri build -- --debug --bundles deb` | TypeScript, Vite, Rust-desktop en `.deb` geslaagd |
| `cargo build --manifest-path crates/nta8800-service/Cargo.toml --bins` | HTTP-API en MCP-server gebouwd |
| Pakketmetadata via `dpkg-deb -f` | `open-energy-studio`, `0.1.6-alpha`, `amd64`; `libwebkit2gtk-4.1-0`, `libgtk-3-0` |

De desktop-overdrachtskopie staat op `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/open-energy-studio_0.1.6-alpha_47c84f3_debug_amd64.deb` (SHA-256 `69d2000bb9071f0914ac0e871aba27bfa9df6c685fd3962de4a0107770b04ee1`). De servicebinaries staan daarnaast als `oes-nta8800-api_47c84f3_linux-amd64` (SHA-256 `44bad4ba6ba6eb5684afc5e183e43c09ed84124fbec66e0b85037c4d8cd32f1d`) en `oes-nta8800-mcp_47c84f3_linux-amd64` (SHA-256 `8346a0365d51ee989909e0e261613d0b126658791c59498c66834795a2aaab60`) in dezelfde outputmap.

Vite meldde een grote hoofdbundel en gemengde statische/dynamische Tauri-import; de build slaagde. Er is op deze commit geen nieuwe volledige frontendtestrun of visuele acceptatie uitgevoerd; de laatste volledige frontendrun (452 tests) staat in het [vorige bouwdossier](nta8800-build-verificatie-2026-10-04-7ac7e52.md). De vergelijkingstests gebruiken synthetische verwachte waarden. Officiële actuele EDR-uitkomsten en externe BRL 9501-attestering ontbreken; `compared_pass` is geen normatieve vrijgave.
