# NTA 8800 herhaalbouw Linux-devpakket — `face280`

Op 15 oktober 2026 zijn vanuit dezelfde schone commit `face28079b27de544f13aed656c9c8f32d1cdfde` twee Linux amd64 devbuilds uitgevoerd, elk met een verse `npm ci` en `cargo --locked`. De bouwschrijver legt `SOURCE_DATE_EPOCH=1792092572` vast, zet dit voor de Tauri-bouw en bouwt de gemaakte debbundel opnieuw met vaste eigenaar, gzip-instelling en die tijdgrens. De gerichte Python-tests voor de schrijver en manifestlezer slaagden (5 tests). De volledige technische gate op dezelfde reken- en UI-code was al geslaagd op `b98ad9c`; latere wijzigingen betroffen alleen bouwbewijs en documentatie.

| Artefact | SHA-256, in beide runs gelijk |
| --- | --- |
| [Desktop-debugpakket, eerste run](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-face280-first/oes-nta8800-dev_face280_linux-amd64.deb) | `984f9a8e61d2c15dd38e7ebc257c605a1ce02265f3d73fa22e3ad73320de68af` |
| HTTP-API | `fd153dcd098acb3a83f507abf0aee113a3102fd2e36295ca0953c858e519bae0` |
| MCP-server | `6d94cd31d47b298f0e655d7c7065fa9adc9d46941b54a593b229c154d88cb977` |
| Referentiegate | `1a5b5281ee969d940e509c53b923e6e42990b315bf577f2fdd80c0a6d3c898fb` |

De [eerste](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-face280-first/oes-nta8800-build_face280_linux-amd64.json) en [tweede](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-face280-second/oes-nta8800-build_face280_linux-amd64.json) manifesten zijn bytegelijk: SHA-256 `ff2772c61161a205fca76e56d53af1b5d47107f0623d27dbcbf945f354053ec2`. De afzonderlijke manifestlezer bevestigde voor beide de vier artefacten, debmetadata, lockhashes en Git-bronmetadata. Beide builds gebruikten dezelfde host, toolchain (Node 24.21.0, npm 12.2.0, Rust/Cargo 1.99.0) en bestaande Cargo-cache. Dit is een herhaalbouw, geen onafhankelijke bouwomgeving of volledige cross-host-reproduceerbaarheid.

De later toegevoegde `--compare-with`-controle heeft deze twee bewaarde manifesten en alle acht artefactbestanden opnieuw gelezen en bytegelijkheid voor dezelfde commit bevestigd. Een negatieve controle met de twee eerdere builds op `a4e55b4` wees het toen afwijkende debpakket af. Dit is een vergelijking van bewaarde bytes, geen bewijs dat beide builds op onafhankelijke hosts zijn gemaakt.

Een tarvergelijking met de vorige bouw op `79148ab` vond dezelfde inhoud en metadata buiten tijdstempels in twee controlbestanden en vijf geïnstalleerde bestanden. De nieuwe debindeling bevat extra root-taringangen door het opnieuw verpakken. Het manifest houdt `referenceVerified=false` en `attestStatus=unattested`; actuele officiële referentiegevallen, visuele desktopacceptatie en formele externe toets ontbreken nog.
