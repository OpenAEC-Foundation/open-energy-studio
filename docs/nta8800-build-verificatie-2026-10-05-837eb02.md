# NTA 8800 technische gate en desktopdevbuild — `837eb02`

Op de schone broncommit `837eb0286badede2828d45ef340eacc719d01da9` is `scripts/verify-nta.sh` volledig geslaagd met twee Vitest-workers. Rust-formatcontrole, Clippy met waarschuwingen als fouten, Rust-kern en service, de kerntests op Rust 1.77.2, Tauri-check, TypeScript, 744 frontendtests in 87 bestanden en de Vite-productiebouw slaagden. De kern had 797 gewone tests plus de bestaande integratiegroepen (6, 2 en 3); de service had 57 gewone tests en de CLI-/transportgroepen (2, 3, 5, 9 en 1). De geplande externe referentiebatch werd expliciet overgeslagen omdat geen officieel actueel plan met cases is ingesteld.

Daarna bouwde `scripts/build-nta-dev.py --skip-npm-ci` vanaf dezelfde schone commit de Rust-servicebinaries en Linux amd64 Tauri-debugbundel. Het bestaande `node_modules` kwam uit een eerder geslaagde schone `npm ci` op hetzelfde ongewijzigde lockbestand. Het [manifest](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-build_837eb02_linux-amd64.json) registreert dit als `preexisting_node_modules` en heeft SHA-256 `82269397ea590733a4bd68a47128d0b99c730d210dfb61b0bcdab04b3eb0687a`.

| Artefact | SHA-256 |
| --- | --- |
| [Desktop-debugpakket](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-dev_837eb02_linux-amd64.deb) | `6e6ec7b161b7b7ae272aa88b143440c77b846df5ddd1f12e41491d9e0be4e48f` |
| [HTTP-API](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-api_837eb02_linux-amd64) | `a63a7f97b7a8b39a1b774bc178a7bb6390f743a32920f21631542d596fca7c52` |
| [MCP-server](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-mcp_837eb02_linux-amd64) | `fe5ec49c5e7497056c6cf1957c424d4ba03622d33f9daecca56f11da3da3fe9a` |
| [Referentiegate](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_837eb02_linux-amd64) | `c4e92fb289dcbf8609472b827b0ced7c1b2e527c1f4fd075ec453ff764501faf` |

De afzonderlijke `verify-nta-build.py --source-repo .` controleerde deze bytes, pakketmetadata, lockhashes en in Git aanwezige bronmetadata. De UI-devserver reageerde met HTTP 200 op `http://127.0.0.1:3006/`; dit is geen visuele acceptatietest. Het manifest meldt `referenceVerified=false` en `attestStatus=unattested`. Officiële actuele EDR-uitkomsten en formele externe toets blijven vereist.
