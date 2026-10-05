# NTA 8800 devbuild — broncommit e9419da

Deze build bevat de uitbreiding van het referentievergelijkingsharnas naar jaarlijkse primaire energie/CO₂ en maand- en jaarsommen per energiedienst en drager. De technische controles zijn op dezelfde stilstaande broncommit uitgevoerd.

| Controle | Resultaat |
| --- | --- |
| Rust-formatcontrole en Clippy voor kern en service, alle targets, waarschuwingen als fout | geslaagd |
| Rust-kern | 759 unittests en 3 integratietests geslaagd |
| HTTP/MCP-service | 56 tests geslaagd |
| `cargo build --manifest-path crates/nta8800-service/Cargo.toml --bins` | API en MCP gebouwd |
| `npm run tauri build -- --debug --bundles deb` | TypeScript, Vite, Rust-desktop en Linux-`.deb` geslaagd |
| Pakketmetadata | `open-energy-studio`, `0.1.6-alpha`, `amd64`; afhankelijk van `libwebkit2gtk-4.1-0` en `libgtk-3-0` |

Overdrachtsbestanden in `/home/aia04/Documents/Codex/2026-09-28/ku/outputs/`:

| Bestand | SHA-256 |
| --- | --- |
| `open-energy-studio_0.1.6-alpha_e9419da_debug_amd64.deb` | `28d7bff95b002f2f5177c8889942028894e6a5ebf48a365e7c9ced78f57361f0` |
| `oes-nta8800-api_e9419da_linux-amd64` | `634ad849c8236cabe16a9b04751f9e58d776dc705679df5e360bcafede51ba01` |
| `oes-nta8800-mcp_e9419da_linux-amd64` | `c058d200d70aadf8ed9a6754f10ba9c5c9c1506e9dcc1f063aa6f12f2c2996be` |

Vite meldde dezelfde waarschuwing voor de circa 2 MB hoofdbundel en de gemengde Tauri-import. Er is geen nieuwe frontendtestrun of visuele desktopacceptatie uitgevoerd; de laatste volledige frontendrun (452 tests) staat bij `7ac7e52`. De nieuwe referentietests gebruiken synthetische waarden uit de Rust-uitvoer om het harnas te testen. Officiële actuele EDR-uitkomsten en externe BRL 9501-attestering ontbreken.

De gebouwde binaries zijn ook als processen getest. De API op de tijdelijke loopbackpoort 3018 gaf voor de synthetische projectfixture BENG 2 = 8,17 en voor een manifest met precies deze aangeleverde verwachting `compared_pass`, `referenceVerified=false`. De MCP-stdio-handshake en `tools/list` slaagden (37 tools, inclusief `compare_reference_case`); `tools/call` gaf dezelfde manifestvingerafdruk en meetposten als HTTP, zonder referentieverklaring. Beide processen zijn na de controle gestopt. Deze pariteit toetst het transport, niet de onafhankelijkheid van de gekozen verwachting.
