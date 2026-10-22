# NTA 8800 vaste-layout-bouwproef — 16 oktober 2026

Broncommit: `0d4e2390c1ea2fe245d73da7d408f056d3774c00`. De bouw van [deze commit](https://github.com/Erika98-be/open-energy-studio/commit/0d4e2390c1ea2fe245d73da7d408f056d3774c00) gebruikt een eigen checkout en Cargo-cache onder `/tmp/oes-nta8800-evidence`, een vast Rustup-pad en `CARGO_INCREMENTAL=0`. De Rust-compiler remapt deze vaste paden naar `/workspace`, `/cargo` en `/rustup`. De Python-bouwbewijssuite slaagde met negen tests; de CI-workflow is als YAML geparseerd.

Twee lokale Linux-debugbuilds zijn elk met een verse `npm ci`, eigen checkout en schone Cargo-cache uitgevoerd. De manifestlezer bevestigde de pakket-, byte- en bronmetadata tegen deze commit en vond pakket, drie servicebinaries en zelfs beide manifesten bytegelijk. Hun manifest-SHA-256 is `b35409552bdf0aeb75d1612130134f940a35fee60ac2c27e9a5270d9abd5961d`. De manifesten registreren `buildLayout=fixed_tmp_v1`, Node 24.21.0, npm 12.2.0, rustc/Cargo 1.99.0, de vier lockhashes, `sourceDateEpoch=1792157811`, `referenceVerified=false` en `attestStatus=unattested`.

| Artefact | Lokale bytes | Lokale SHA-256 |
| --- | ---: | --- |
| API | 229021208 | `e8f0a69035d8d5e86deb3a8981aaac10498475cdb9bccbcc8a9144300027148e` |
| Desktop `.deb` | 89036464 | `b582570385f80881aa9675d7720fd27a618efcc9b86e310d35258f99dd10bdbe` |
| MCP | 259189784 | `91d5f0f4bbaeaf34f8a0f1f5ce4424143d79e6b3b386434aa6d3780baaad5c59` |
| Referentiegate | 222875336 | `bcb2c94333a736612caa447170f8f953d3aedfb65b8df7d439fa72e1a5f3f6d7` |

De lokale API bevat geen `/home/aia04`- of `/tmp/oes-nta8800-evidence`-padbytes; `/workspace` en `/cargo` komen wel voor. Alle vijf jobs van [GitHub-run 38909208019](https://github.com/Erika98-be/open-energy-studio/actions/runs/38909208019) slaagden. De runner gebruikte dezelfde geregistreerde compiler-, Node-, npm-, lock- en layoutgegevens. Na download en controle zijn de API, MCP en referentiegate **bytegelijk tussen de lokale host en de runner**. Alleen het desktop-`.deb` verschilt: lokaal 89 036 464 bytes en SHA-256 `b582570385f80881aa9675d7720fd27a618efcc9b86e310d35258f99dd10bdbe`, op de runner 89 036 476 bytes en SHA-256 `c5b27d211c1c82b53690e211e411b216741a91fbab5005113d3c00db8e700014`. De manifestvergelijker meldt daarom terecht `Build artifact differs: desktop`.

De twee uitgepakte pakketten hebben dezelfde zeven bestanden op dezelfde paden. Alleen `usr/bin/open-energy-studio` en het daarvan afgeleide `DEBIAN/md5sums` verschillen. De ELF-bestanden zijn even groot (399 223 920 bytes); 1 036 bytes verschillen in 39 aaneengesloten gebieden. Afgezien van de ELF-build-ID betreft dit een reeks van 25 CSP-SHA-256-hashes in `.rodata`: de **verzameling hashes is identiek, de volgorde verschilt**. Dit wijst op niet-deterministische iteratie in Tauri's asset-codegeneratie; het is een technische oorzaak van de pakketafwijking en geen verschil in frontend-inhoud. De bron van `tauri-codegen` 2.5.5 bewaart assets en per-HTML-hashes in `HashMap` en gebruikt directoryvolgorde voor de globale hashes. Een lokale patch sorteert deze gegevens voor codegeneratie; een nieuwe onafhankelijke bouw moet aantonen of daarmee ook het desktoppakket bytegelijk wordt.

Deze bouw levert geen officiële NTA-referentietoets of extern attest op.
