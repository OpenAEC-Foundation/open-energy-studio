# NTA 8800 cross-host-bouwproef — 16 oktober 2026

Broncommit: `99466ee68b1a30f342c2b122f5ea388bdb0ed6ad`. De lokale Linux Mint 22.3-build en [GitHub-run 38881147491](https://github.com/Erika98-be/open-energy-studio/actions/runs/38881147491) zijn apart met een verse `npm ci` gebouwd. Op GitHub slaagden de volledige gate, de drie platforminstallateurs en de extra `nta-build-evidence`-job. Het runner-artefact is gedownload. Beide debug-bouwmanifesten slaagden afzonderlijk voor de SHA-256-, pakket- en broncommitcontrole met `--source-repo .`; norm- en kernelversie, pakketmetadata, lockhashes en `sourceDateEpoch` komen overeen.

De vergelijking `--compare-with` gaf exit 1: `Build artifact differs: api`. Alle vier artefacten verschillen in hash en grootte:

| Artefact | Lokaal (bytes) | GitHub-runner (bytes) | Lokaal SHA-256 | Runner SHA-256 |
| --- | ---: | ---: | --- | --- |
| API | 237391496 | 229078816 | `fd153dcd098acb3a83f507abf0aee113a3102fd2e36295ca0953c858e519bae0` | `e0fd13cac1a195e90ca4d6aa303a458f8896df23947e83cd944275f90739de17` |
| Desktop `.deb` | 103227964 | 88101122 | `66d16ac509a169566cab06ffc3d4ff53418ee80c385b074a276de721777e5c57` | `c3aba4f0f246fd5feb12fe105e6b0d928f4af3876f3849352c74d02aac2abcbb` |
| MCP | 270062536 | 258712208 | `6d94cd31d47b298f0e655d7c7065fa9adc9d46941b54a593b229c154d88cb977` | `7172748a724c001d29710f3c721c3a5fd3539c53afb3d7517b1cfa70b7b5c68a` |
| Referentiegate | 229930176 | 222928504 | `1a5b5281ee969d940e509c53b923e6e42990b315bf577f2fdd80c0a6d3c898fb` | `9041ecd1ce0e6bd1b6bcd19432e50829251bbfbc389a7e51811af844360f17ae` |

De lokale build gebruikte rustc/Cargo 1.99.0, Node 24.21.0 en npm 12.2.0 op een Ubuntu-24.04-basis. De runner gebruikte rustc/Cargo 1.99.1, Node 24.21.0 en npm 11.19.0 op Ubuntu 22.04. De ELF-commentsectie bevestigt de verschillende rustc- en GCC-versies; in beide servicebinaries staan bovendien verschillende absolute bronpaden (`/home/aia04` tegenover `/home/runner`). Dit zijn waargenomen verschillen, geen bewezen volledige oorzaakanalyse. De CI-bouwbewijsjob is daarom voor de volgende gemarkeerde proef op Ubuntu 24.04, Rust 1.99.0, Node 24.21.0 en npm 12.2.0 vastgezet. Padvastlegging en eventuele overige hostafhankelijkheden blijven te onderzoeken.

Dit is geen bytegelijke cross-host-bouw. Evenmin is het een officiële NTA-referentietoets of extern attest; beide manifesten blijven `referenceVerified=false` en `attestStatus=unattested`.
