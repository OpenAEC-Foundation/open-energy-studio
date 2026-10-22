# NTA 8800 Linux-devbuild — `966fea8`

De Linux amd64 devbuild op schone commit `966fea84a1dc5bcac230dcb3d3d1b58824056d77` slaagde met verse `npm ci` en Cargo-lockbestanden. De bouwer zet nu ook tijdstempels van uitgepakte bestanden en symbolische links expliciet op de committijd, inclusief invoer die ouder is dan `SOURCE_DATE_EPOCH`. De vijf gerichte Python-bouwbewijstests slaagden, met een fixture die zowel oudere als nieuwere ingangstijden en een symbolische link bevat.

Het [manifest](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-966fea8/oes-nta8800-build_966fea8_linux-amd64.json) heeft SHA-256 `fe1d6c4405c56424e284b0a8c2e8dcdfd7eae294df03724d5d92b10beb29ef52` en legt `SOURCE_DATE_EPOCH=1792095769` vast. De afzonderlijke lezer bevestigde de vier artefacten en de Git-bronmetadata. Alle drie control- en achttien data-taringangen in het debpakket hebben die committijd.

| Artefact | SHA-256 |
| --- | --- |
| [Desktop-debugpakket](/home/aia04/Documenten/GitHub/open-energy-studio-builds/2026-10-15-966fea8/oes-nta8800-dev_966fea8_linux-amd64.deb) | `db6f86f9050c3143acc1a07a42979a5a7dae7096c50178b4f5c242d53744bfe6` |
| HTTP-API | `fd153dcd098acb3a83f507abf0aee113a3102fd2e36295ca0953c858e519bae0` |
| MCP-server | `6d94cd31d47b298f0e655d7c7065fa9adc9d46941b54a593b229c154d88cb977` |
| Referentiegate | `1a5b5281ee969d940e509c53b923e6e42990b315bf577f2fdd80c0a6d3c898fb` |

Dit is één lokale bouw op dezelfde host als de eerdere herhaalbuilds, met bestaande Cargo-cache. Het bewijs voor bytegelijke herhaalbuilds op commit `face280` blijft afzonderlijk; voor `966fea8` is geen tweede bouw uitgevoerd. Onafhankelijke cross-host-reproductie, actuele officiële referentiegevallen, visuele desktopacceptatie en formele externe toets blijven open. Het manifest vermeldt `referenceVerified=false` en `attestStatus=unattested`.
