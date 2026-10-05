# NTA-desktopdevbuild na afhankelijkheidsupdate — `dc1b917`

De Linux amd64 debugbuild op broncommit `dc1b9172beb7132988f6ce1c3465ba5e41300cac` is geslaagd na het exact vastzetten van de Tauri-JavaScriptpakketten op de bijbehorende Rust-minorversies. De Rust-servicebinaries, TypeScript-/Vite-productiebouw, Tauri-debugapp en `.deb`-bundel zijn gebouwd. De live UI-devserver op `http://127.0.0.1:3006/` gaf daarna HTTP 200.

Vlak voor de bundelbouw is `npm ci` zonder extra vlaggen op hetzelfde lockbestand geslaagd met 0 npm-auditmeldingen. De bouwscript is daarna met `--skip-npm-ci` uitgevoerd om deze al geïnstalleerde dependencies te gebruiken; het [machineleesbare manifest](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-build_dc1b917_linux-amd64.json) registreert daarom correct `preexisting_node_modules`. Het manifest heeft SHA-256 `4ad5ec7e03b0f0c533cd0ba0fb596c5d44b5dc99538e1c302f31fbf46ce39810`.

| Artefact | SHA-256 |
| --- | --- |
| [Desktop-debugpakket](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-dev_dc1b917_linux-amd64.deb) | `ebadf197e7c040e90b6b2285a4390774f94cbbfe1f36d5eaec8a2e5ffe5bbceb` |
| [HTTP-API](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-api_dc1b917_linux-amd64) | `b20b31502b50f00bf44e466b9e5a4fc62c5a158160d692c56b9aad683b845349` |
| [MCP-server](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-mcp_dc1b917_linux-amd64) | `3d4b6f027e8c369dc66a220c49a2d6f4097b40a006565999c979dc24f8509015` |
| [Referentiegate](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_dc1b917_linux-amd64) | `2ebf75f4b4963d9aaf1a9e4bbfe223f879cc337f536f4f1783fbd58d7139814c` |

`scripts/verify-nta-build.py --source-repo .` controleerde opnieuw de vier artefacten, het debpakket en de in Git vastgelegde lock- en versiemetadata. De frontend op `bb7c1bd` met de vernieuwde generieke dependencies doorliep eerder 505 tests; na de Tauri-pins slaagden een schone `npm ci`, `npm run build` en de desktopbouw. De volledige UI-suite is na de Tauri-pins niet nogmaals uitgevoerd. Officiële actuele referentie-uitkomsten, visuele UI-acceptatie en formele attestering ontbreken; het manifest meldt `referenceVerified=false` en `attestStatus=unattested`.
