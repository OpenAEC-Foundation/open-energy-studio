# NTA 8800 desktopdevbuild — `413e932`

De schone broncommit `413e932bffaa9f0938f148743e115095323545cb` is gebouwd met `scripts/build-nta-dev.py --skip-npm-ci`. De servicebuild, TypeScript/Vite-bundel en Linux amd64 Tauri-debugbundel slaagden. Het bestaande `node_modules` is als `preexisting_node_modules` in het manifest geregistreerd; deze run bewijst geen verse npm-installatie. De voorafgaande volledige technische gate op dezelfde UI- en kerncode slaagde met 797 kern-, 57 service- en 744 frontendtests plus integratiegroepen. Na de servicewijziging slaagden 58 servicetests plus integratiegroepen, formatcontrole en Clippy.

Het [bouwmanifest](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-build_413e932_linux-amd64.json) heeft SHA-256 `42067e487ff6ed77a1ff55270e186a9424c914f1454530989ed4251fbc50b8f7`.

| Artefact | SHA-256 |
| --- | --- |
| [Desktop-debugpakket](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-dev_413e932_linux-amd64.deb) | `81e3c857c02aceab5c07c205eccd409fbb17f7b59935775a4330ef19ee2cf540` |
| [HTTP-API](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-api_413e932_linux-amd64) | `960131d93b5662cd6463af2a52857d602e1edf06b38b4ef11ed821ee57e3c29b` |
| [MCP-server](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-mcp_413e932_linux-amd64) | `fc26e435928a36a6d76ee02f215f9df8333a899f0804e090f2725cd6186f8586` |
| [Referentiegate](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_413e932_linux-amd64) | `c4e92fb289dcbf8609472b827b0ced7c1b2e527c1f4fd075ec453ff764501faf` |

De afzonderlijke `verify-nta-build.py --source-repo .` controleerde de bytes, debmetadata, lockhashes en Git-bronmetadata. De officiële actuele referentiebatch, visuele UI-acceptatie en externe BRL 9501-toets zijn niet uitgevoerd. Het manifest houdt `referenceVerified=false` en `attestStatus=unattested`.
