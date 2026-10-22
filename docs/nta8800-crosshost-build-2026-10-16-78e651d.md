# NTA 8800 bronpadproef — 16 oktober 2026

Broncommit: `78e651d766c774d7c73be88952d0c70c4624b2b2`. De bouwschrijver geeft de Rust-compiler vaste bronpaden voor de werkboom (`/workspace`), Cargo-cache (`/cargo`) en Rustup-cache (`/rustup`). Vooraf ingestelde `RUSTFLAGS` en `CARGO_ENCODED_RUSTFLAGS` worden geweigerd. De gerichte Python-bouwbewijssuite slaagde met negen tests.

Twee lokale Linux-debugbuilds uit deze commit zijn elk met een verse `npm ci` uitgevoerd. De eerste compileerde de Rust-afhankelijkheden opnieuw vanwege de gewijzigde compileropties; de tweede gebruikte de resulterende Cargo-cache. Beide manifesten slaagden afzonderlijk voor de byte- en broncontrole. `--compare-with` bevestigde dat pakket, API, MCP, referentiegate en zelfs de manifesten bytegelijk zijn.

| Artefact | Bytes | SHA-256 in beide lokale builds |
| --- | ---: | --- |
| API | 237363968 | `7fbcb718890eab506e72d91df7fec31f64c59bd795752c055187b5a101e44f21` |
| Desktop `.deb` | 103219202 | `3d56e99ec39acfea95b07c091300976e9191e7b940be275d0dd31793d75626f5` |
| MCP | 270032304 | `e4362eebaabc6825f674cd6dba13e76e7f1cf39063d151418d4657c05a4fd1b1` |
| Referentiegate | 229906496 | `c9ca729dd9a82b9d87ff436b57439ba524c4da8d6b8f2e6ff6916f5912135d03` |
| Manifest | — | `1707f8246ac2af2dbaa77bd538f94b898854c4f783e181382530b009835cfa3e` |

De lokale API bevat geen `/home/aia04`-padbytes meer; de vaste `/workspace`- en `/cargo`-paden zijn aanwezig. De lokale `.comment`-sectie noemt GCC `13.3.0-6ubuntu2~24.04.1`, net als de runner. De manifesten registreren Node 24.21.0, npm 12.2.0, rustc/Cargo 1.99.0 en de commitgebonden epoch.

De aparte [GitHub-run 38901586617](https://github.com/Erika98-be/open-energy-studio/actions/runs/38901586617) is volledig geslaagd: de technische gate, de drie platformbuilds en `nta-build-evidence` zijn groen. Het gedownloade runner-manifest slaagt ook voor byte-, pakket- en broncontrole. De vergelijking met de lokale set faalt alsnog met `Build artifact differs: api`; alle vier artefacten verschillen:

| Artefact | Runnerbytes | Runner SHA-256 |
| --- | ---: | --- |
| API | 229018528 | `ad9c7603011c1904321c1c798637132556e6ff3520ce24f49952668089e061f3` |
| Desktop `.deb` | 89012990 | `dce2352efe6f0d515219ea3eadd785c952f294f727da1a936ca5f14dfb9ff04d` |
| MCP | 259184160 | `a8108745ef6f2d98639a2155a8e69bf9574fdce0df1124bd88a65996c9169a1c` |
| Referentiegate | 222875976 | `36df3f501d6c2dedfce0c4baa69639f97ccb26d191860dd1f1a2ad6c6ce1f026` |

De twee manifesten hebben identieke toolchainmetadata, locks, norm-/kernelversie en broncommit. Beide API's bevatten geen hostspecifieke `/home`-padbytes; de lokale bevat wel meer `/workspace`- en `/cargo`-vermeldingen dan die van de runner. Het verschil zit vooral in `.debug_info` (lokaal 47 661 206 tegenover runner 39 846 284 bytes), maar ook `.text` en `.rodata` verschillen. De absolute bronpaden binnen de compileroptie `--remap-path-prefix` zijn op de hosts verschillend en kunnen crate-identiteiten en symboolnamen beïnvloeden; dat is een onderzoekshypothese, geen bewezen volledige oorzaak. Een volgende gecontroleerde proef moet de bron- en Cargo-cachepaden tijdens het compileren zelf gelijk maken en incrementele bouw uitsluiten.

Beide manifesten blijven `referenceVerified=false` en `attestStatus=unattested`. Deze bouwproef levert geen bytegelijke cross-host-bouw, officiële NTA-referentietoets of extern attest.
