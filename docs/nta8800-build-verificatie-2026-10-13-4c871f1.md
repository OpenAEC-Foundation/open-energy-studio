# NTA 8800 Linux-devbuild — `4c871f1`

Op 13 oktober 2026 bouwde `scripts/build-nta-dev.py` vanaf de schone broncommit `4c871f10be54cc6f6a7a3f91cb1f038c2468b171` een Linux amd64 Tauri-debugpakket en drie Rust-servicebinaries. `npm ci` installeerde de frontendafhankelijkheden opnieuw uit het lockbestand (0 gemelde kwetsbaarheden); de TypeScript- en Vite-productiebouw en de Cargo-bouw met `--locked` slaagden. De laatste volledige technische gate voor deze code is review 72 op `0dcd315`; sindsdien zijn alleen documentatiecommits toegevoegd. Deze bouw is geen nieuwe volledige testrun.

Het [manifest](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-13-4c871f1/oes-nta8800-build_4c871f1_linux-amd64.json) heeft SHA-256 `46e45d90ef82e116aaa876c5ea96b94ae85b881a2089c0e0836856ad515cfce1` en vermeldt Node 22.23.3, npm 10.9.9, Rust/Cargo 1.99.0, de vier dependency-lockhashes, pakketversie 0.2.0 en kernelversie 0.4.0.

| Artefact | SHA-256 |
| --- | --- |
| [Desktop-debugpakket](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-13-4c871f1/oes-nta8800-dev_4c871f1_linux-amd64.deb) | `5fdd31fcdd1016873eae91cf532be03c220b36e2ebf6fcf655aaa0ed2420b958` |
| [HTTP-API](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-13-4c871f1/oes-nta8800-api_4c871f1_linux-amd64) | `fd153dcd098acb3a83f507abf0aee113a3102fd2e36295ca0953c858e519bae0` |
| [MCP-server](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-13-4c871f1/oes-nta8800-mcp_4c871f1_linux-amd64) | `6d94cd31d47b298f0e655d7c7065fa9adc9d46941b54a593b229c154d88cb977` |
| [Referentiegate](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-13-4c871f1/oes-nta8800-reference-gate_4c871f1_linux-amd64) | `1a5b5281ee969d940e509c53b923e6e42990b315bf577f2fdd80c0a6d3c898fb` |

`scripts/verify-nta-build.py --source-repo .` bevestigde de artefactbytes, pakketmetadata en Git-bronmetadata. Het bewijst geen onafhankelijke bouwherkomst of bit-identieke tweede bouw. Het manifest houdt `referenceVerified=false` en `attestStatus=unattested`; officiële actuele referentie-uitkomsten, visuele desktopacceptatie en formele externe toets ontbreken.

De bewaarde HTTP-binary startte apart op `127.0.0.1:3017`: `/health` gaf HTTP 200 met kernel 0.4.0. Een POST van de voorbeeldtussenwoning naar `/v1/nta8800/project/performance` gaf HTTP 200, `calculated_unverified`, BENG 2 = 80,56 kWh/m²·jaar, indicatieve klasse A+ en `attestStatus=unattested`. De bewaarde MCP-binary onderhandelde protocol `2025-06-18` en gaf voor dezelfde projectinvoer via `calculate_project_performance` eveneens `calculated_unverified`, 80,56, A+ en `unattested`. Dit is een runtimecontrole van twee adapters met één interne voorbeeldinvoer, geen onafhankelijke normvergelijking.
