# NTA 8800 cross-host-bouwproef met gelijke toolchain — 16 oktober 2026

Broncommit: `ee7ec3fe7742b9a1fd660665a561cedb34be5b08`. De lokale debug-bouw en [GitHub-run 38888068521](https://github.com/Erika98-be/open-energy-studio/actions/runs/38888068521) zijn elk met een verse `npm ci` uitgevoerd. De CI-gate, Linux-, Windows- en macOS-installateurs en de afzonderlijke `nta-build-evidence`-job slaagden. De manifestlezer bevestigde van beide debug-bouwsets de hashes, pakketmetadata en broncommit voordat de bytevergelijking begon.

Beide manifesten vermelden dezelfde locks, `sourceDateEpoch=1792146833`, rustc/Cargo 1.99.0, Node 24.21.0 en npm 12.2.0. Beide hosts gebruiken een Ubuntu-24.04-basis. Toch faalt `python3 scripts/verify-nta-build.py <lokaal-manifest> --compare-with <runner-manifest> --source-repo .` met exit 1: `Build artifact differs: api`. Alle vier artefacten verschillen:

| Artefact | Lokaal (bytes) | Runner (bytes) | Lokaal SHA-256 | Runner SHA-256 |
| --- | ---: | ---: | --- | --- |
| API | 237391496 | 229032592 | `fd153dcd098acb3a83f507abf0aee113a3102fd2e36295ca0953c858e519bae0` | `44ece5ac126100c4d9d1e3dcff696084792ee7b6b173ea3bb1031890478a5d15` |
| Desktop `.deb` | 103227954 | 89040476 | `0cf84febf83f6b75267091468b07490d3203bb843918ee7aa168d8d3d48a99a8` | `e2137f593555ec472d16d106d741c238858767dd2b17dbba329cebc0bc49b36e` |
| MCP | 270062536 | 259199848 | `6d94cd31d47b298f0e655d7c7065fa9adc9d46941b54a593b229c154d88cb977` | `5e50db005cc9dc1e4db8a059f5cb5c2d78ddd33dc92363087b3727a04d973a2f` |
| Referentiegate | 229930176 | 222886856 | `1a5b5281ee969d940e509c53b923e6e42990b315bf577f2fdd80c0a6d3c898fb` | `66d0bf7b3581fb8a3c4e765788774a9931fd4c21e4d1e82f97bca7f89f36162d` |

`readelf -p .comment` op de API toont op beide hosts rustc 1.99.0 en LLD 23.1.1, maar lokaal GCC `13.3.0-6ubuntu2~24.04` en op de runner `13.3.0-6ubuntu2~24.04.1`. In de debug-binaries zijn ook verschillende absolute Cargo-registerpaden aanwezig (`/home/aia04` en `/home/runner`). Een onderzoekskopie van de API is op beide hosts van debugsymbolen en `.comment` ontdaan; ook dan verschillen de SHA-256 en nog 63 016 bytes in bestandsgrootte. De precieze oorzaken en een passende pad-/omgevingnormalisatie zijn dus nog niet vastgesteld. Deze proef levert geen bytegelijke cross-host-bouw op.

De geldige manifesten blijven `referenceVerified=false` en `attestStatus=unattested`. De geslaagde CI-jobs leveren technisch bouwbewijs, geen officiële NTA-referentie-uitkomst of extern attest.
