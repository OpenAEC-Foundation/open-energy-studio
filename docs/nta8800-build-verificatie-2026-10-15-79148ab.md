# NTA 8800 Linux-devbuild — `79148ab`

Op 15 oktober 2026 bouwde `scripts/build-nta-dev.py` vanaf de schone broncommit `79148ab6022126f7ccfa8bc03f3dc48c0f3756df` opnieuw het Linux amd64 Tauri-debugpakket en de drie Rust-servicebinaries. `npm ci` installeerde vanuit het lockbestand (0 gemelde kwetsbaarheden); Cargo gebruikte `--locked`. TypeScript, Vite en de desktopbundel slaagden. De technische gate op `b98ad9c` slaagde eerder op dezelfde reken- en UI-code; tussen die commit en deze bouw veranderde alleen het verificatiedossier.

Het [manifest](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-79148ab/oes-nta8800-build_79148ab_linux-amd64.json) heeft SHA-256 `2a20ac3fcf3fdaa703382a315a27624ce9c06b4019eef469e4152f4f50eea5dd`. Het vermeldt Node 24.21.0, npm 12.2.0, Rust/Cargo 1.99.0, de vier dependency-lockhashes, pakketversie 0.2.0 en kernelversie 0.4.0.

| Artefact | SHA-256 |
| --- | --- |
| [Desktop-debugpakket](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-79148ab/oes-nta8800-dev_79148ab_linux-amd64.deb) | `4e2e17764762d4727d30b5298279a06fe64b284a2454ede687c31c5b9e0c6f51` |
| [HTTP-API](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-79148ab/oes-nta8800-api_79148ab_linux-amd64) | `fd153dcd098acb3a83f507abf0aee113a3102fd2e36295ca0953c858e519bae0` |
| [MCP-server](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-79148ab/oes-nta8800-mcp_79148ab_linux-amd64) | `6d94cd31d47b298f0e655d7c7065fa9adc9d46941b54a593b229c154d88cb977` |
| [Referentiegate](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-79148ab/oes-nta8800-reference-gate_79148ab_linux-amd64) | `1a5b5281ee969d940e509c53b923e6e42990b315bf577f2fdd80c0a6d3c898fb` |

`verify-nta-build.py --source-repo .` bevestigde artefactbytes, debmetadata en Git-bronmetadata. De drie servicebinaries zijn bytegelijk aan de bewaarde bouw van `4c871f1`. Een aanvullende vergelijking met `dpkg-deb -x` en `dpkg-deb -e` vond vijf bytegelijke geïnstalleerde bestanden en twee bytegelijke controlbestanden. De `.deb`-archieven hebben wel verschillende hashes: hun `ar`-leden en de 19 tar-ingangen hebben verschillende bouwtijdstempels; de vergeleken tar-metadata buiten tijdstempels is gelijk. Dit verklaart waarom de pakketbytes geen bit-identieke herbouw aantonen, ondanks gelijke uitgepakte inhoud. Deze herbouw gebeurde op dezelfde host, met bestaande Cargo-artefacten en andere Node-/npm-versies. Zij is daarom geen onafhankelijke tweede desktopbouw. Het manifest houdt `referenceVerified=false` en `attestStatus=unattested`; officiële actuele referentie-uitkomsten, visuele desktopacceptatie en externe toets ontbreken.
