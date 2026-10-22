# NTA 8800 Linux-devbuild — 16 oktober 2026

Broncommit: `f70749da90495bf967f815e28a0a3dd6b809eb6e`. Twee opeenvolgende builds op dezelfde Linux-host, elk met `npm ci`, Cargo `--locked`, TypeScript/Vite en Tauri-debug-`.deb`, slaagden. Beide manifesten zijn met `scripts/verify-nta-build.py --source-repo .` tegen de broncommit gecontroleerd. De vergelijking met `--compare-with` bevestigde bytegelijke vier artefacten en manifesten; exit 0. De Git-werkboom bleef schoon. Bestaande Cargo-buildcache was beschikbaar.

| Artefact | Grootte (bytes) | SHA-256 (beide builds) |
| --- | ---: | --- |
| Desktop `.deb` | 103227930 | `54526d7a0c34f3b5cf5fdb3dae73df674a68784852ef91c71e79bb66165a3ecf` |
| HTTP-API | 237391496 | `fd153dcd098acb3a83f507abf0aee113a3102fd2e36295ca0953c858e519bae0` |
| MCP | 270062536 | `6d94cd31d47b298f0e655d7c7065fa9adc9d46941b54a593b229c154d88cb977` |
| Referentiegate | 229930176 | `1a5b5281ee969d940e509c53b923e6e42990b315bf577f2fdd80c0a6d3c898fb` |
| Manifest | — | `83ba739f97c10dc20c74ea52e0e1289a72adcb5687e15a9aae6402725c921062` |

De manifesten liggen lokaal onder `nta-evidence/local-f70749d-a/` en `nta-evidence/local-f70749d-b/` (Git genegeerd). Ze leggen Node 24.21.0, npm 12.2.0, rustc/Cargo 1.99.0, `sourceDateEpoch=1792117637`, lockhashes, pakketversie 0.2.0, `referenceVerified=false` en `attestStatus=unattested` vast. De herziene manifestlezer bevestigde bij deze echte builds de commitgebonden epoch en gaf de juiste broncontrolemelding.

Deze herhaalproef op één host bewijst geen onafhankelijke reproduceerbaarheid, NTA-rekenjuistheid, actuele officiële referentieuitkomst, visuele desktopacceptatie of attestering. De afzonderlijke GitHub-runnerworkflow is nog niet uitgevoerd.
