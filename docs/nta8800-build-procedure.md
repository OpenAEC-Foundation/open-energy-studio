# NTA 8800 desktop-devbuild vastleggen

`scripts/build-nta-dev.py` bouwt op Linux amd64 het Tauri-debugpakket en de Rust-binaries voor HTTP, MCP en de referentiegate. De script weigert een niet-schone Git-werkboom, gebruikt `npm ci` en `cargo build --locked`, controleert de pakketversie en schrijft SHA-256, bestandsgrootte, broncommit, lockbestand-hashes en toolversies naar een JSON-manifest. De manifest bevat altijd `referenceVerified=false` en `attestStatus=unattested`.

Voor de desktopbundel zet de schrijver `SOURCE_DATE_EPOCH` op de Git-committijd en legt die waarde in het manifest vast. Na Tauri pakt hij de debbundel uit, sorteert de regels in `DEBIAN/md5sums` op pakketpad, zet alle uitgepakte tijdstempels op die committijd en bouwt deze opnieuw met `dpkg-deb --root-owner-group -Zgzip -z1`: het pakket krijgt zo stabiele controlesomvolgorde, eigenaars, archief- en tar-tijdstempels en compressie-instellingen. De bronrepositorycontrole vergelijkt de vastgelegde waarde met de commit. Toolchain, systeemlibraries en andere bronbestandsmetadata kunnen nog steeds verschillen.

Voor dezelfde commit weigert de script bestaande artefactnamen in de uitvoermap te overschrijven. Gebruik een nieuwe uitvoermap voor een herbouw die u naast de eerdere artefacten wilt vergelijken.

De schrijver maakt artefacten en het manifest exclusief aan. Ook een bestaande symbolische link naar een ontbrekend bestand blokkeert de bouwuitvoer; een doel dat tijdens de bouw verschijnt wordt niet vervangen. Dit beschermt bewaard bouwbewijs tegen onbedoeld overschrijven, maar maakt de bouw niet onafhankelijk of bit-identiek reproduceerbaar.

De technische gate voert de Python-tests van de schrijver en manifestlezer uit met Python 3.11. CI zet die versie expliciet klaar; de lokale bouwhost heeft dezelfde minimumversie nodig vanwege `tomllib`.

Controleer een bewaard pakket later met `python3 scripts/verify-nta-build.py /pad/naar/oes-nta8800-build_...json`. Deze lezer controleert de vier bestandsgroottes en SHA-256-hashes, veilige unieke bestandsnamen zonder symbolische links, de Linux-debpakketmetadata en de expliciet ongetoetste status. Een gekopieerd manifest kan echter een willekeurige broncommit claimen; vergelijk de manifesthash met een afzonderlijk bewaard dossier en controleer de bronstand apart. Een geslaagde bytecontrole bewijst geen normatieve juistheid.

Met `--source-repo /pad/naar/repo` toetst dezelfde lezer bovendien of de geclaimde SHA een Git-commit in die repository is en of de vier lockbestanden, pakketversie, kernelversie, doeluitgave en vastgelegde `SOURCE_DATE_EPOCH` (indien aanwezig) in die commit met het manifest overeenkomen. Een boom- of tagobject telt niet als broncommit. Deze koppeling bewijst nog niet dat de binaries werkelijk uit die bron zijn gecompileerd; daarvoor blijven een vertrouwde bouwomgeving en een reproduceerbare tweede bouw nodig.

Met `--compare-with /pad/naar/tweede-manifest.json` controleert de lezer beide artefactsets en vereist hij dezelfde broncommit, norm- en kernelversie, pakketmetadata, lockhashes en bytegelijke hashes en groottes van pakket, API, MCP en referentiegate. Combineer dit met `--source-repo` om beide manifesten tegen dezelfde bron te controleren. De opdracht toont of de bytes gelijk zijn; leg afzonderlijk vast op welke hosts en met welke toolchains de builds daadwerkelijk zijn gemaakt.

De GitHub Actions-job `nta-build-evidence` bouwt dezelfde vier artefacten op een Ubuntu-runner zonder hergebruik van de lokale Cargo-cache, controleert het manifest tegen de uitgecheckte commit en bewaart de bestanden 30 dagen als workflow-artefact. Download die set samen met een lokale build van **dezelfde commit** en voer de vergelijking met `--compare-with` uit. De workflowdefinitie op zichzelf is geen bouwbewijs; leg de geslaagde run, host-/toolchaingegevens en byteuitkomst vast.

Voer dit vanaf een gecommitte bronstand uit met Node/npm, Rust/Cargo, Tauri's Linux-ontwikkelbibliotheken en `dpkg-deb` op `PATH`:

```bash
python3 scripts/build-nta-dev.py --output-dir /pad/naar/bouwuitvoer
```

Voor een cross-host-bytevergelijking gebruikt de CI-job `scripts/build-nta-fixed-layout.py`. Dit bouwt vanuit een schone kopie van de commit onder `/tmp/oes-nta8800-evidence`, met een eigen Cargo-cache, vast Rustup-pad en `CARGO_INCREMENTAL=0`. Gebruik lokaal dezelfde wrapper met een nieuwe uitvoermap. Het manifest meldt `buildLayout=fixed_tmp_v1`; de vergelijker weigert verschillende layouts. De lokale en runner-toolchainversies moeten eveneens overeenkomen. De [proef op `7436583`](nta8800-crosshost-build-2026-10-18-7436583.md) leverde met deze procedure vier bytegelijke Linux-debug-artefacten en bytegelijke manifesten op de lokale host en een GitHub-runner.

Voor een snellere lokale herbouw kan `--skip-npm-ci` worden gebruikt. Het manifest vermeldt dan `preexisting_node_modules`, zodat deze route niet als een verse lockbestandinstallatie wordt gepresenteerd. De script voert geen normatieve referentievergelijking of visuele UI-acceptatie uit. Verschillen in systeemlibraries, compiler, toolchain of buildtimestamps kunnen ook bij dezelfde broncommit andere binary-hashes opleveren; de manifest maakt die verschillen onderzoekbaar, maar bewijst geen bit-identieke herbouw of BRL 9501-attestering.

`scripts/verify-nta.sh` is de afzonderlijke technische testgate. Alle Cargo-tests, Clippy-controles, Tauri-checks en de optionele referentiegate gebruiken de vastgelegde Cargo-lockbestanden (`--locked`). Stel `NTA_REFERENCE_PLAN` en `NTA_REFERENCE_CASE_DIR` in zodra een rechtmatig verkregen, onafhankelijk vastgesteld actueel W/U-plan beschikbaar is; zonder deze variabelen meldt het script dat de referentiegate is overgeslagen.
