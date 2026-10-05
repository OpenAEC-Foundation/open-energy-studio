# NTA 8800 desktop-devbuild vastleggen

`scripts/build-nta-dev.py` bouwt op Linux amd64 het Tauri-debugpakket en de Rust-binaries voor HTTP, MCP en de referentiegate. De script weigert een niet-schone Git-werkboom, gebruikt `npm ci` en `cargo build --locked`, controleert de pakketversie en schrijft SHA-256, bestandsgrootte, broncommit, lockbestand-hashes en toolversies naar een JSON-manifest. De manifest bevat altijd `referenceVerified=false` en `attestStatus=unattested`.

Voor dezelfde commit weigert de script bestaande artefactnamen in de uitvoermap te overschrijven. Gebruik een nieuwe uitvoermap voor een herbouw die u naast de eerdere artefacten wilt vergelijken.

Controleer een bewaard pakket later met `python3 scripts/verify-nta-build.py /pad/naar/oes-nta8800-build_...json`. Deze lezer controleert de vier bestandsgroottes en SHA-256-hashes, veilige unieke bestandsnamen, de Linux-debpakketmetadata en de expliciet ongetoetste status. Een gekopieerd manifest kan echter een willekeurige broncommit claimen; vergelijk de manifesthash met een afzonderlijk bewaard dossier en controleer de bronstand apart. Een geslaagde bytecontrole bewijst geen normatieve juistheid.

Met `--source-repo /pad/naar/repo` toetst dezelfde lezer bovendien of de geclaimde commit in die Git-repository bestaat en of de vier lockbestanden, pakketversie, kernelversie en doeluitgave in die commit met het manifest overeenkomen. Deze koppeling bewijst nog niet dat de binaries werkelijk uit die bron zijn gecompileerd; daarvoor blijven een vertrouwde bouwomgeving en een reproduceerbare tweede bouw nodig.

Voer dit vanaf een gecommitte bronstand uit met Node/npm, Rust/Cargo, Tauri's Linux-ontwikkelbibliotheken en `dpkg-deb` op `PATH`:

```bash
python3 scripts/build-nta-dev.py --output-dir /pad/naar/bouwuitvoer
```

Voor een snellere lokale herbouw kan `--skip-npm-ci` worden gebruikt. Het manifest vermeldt dan `preexisting_node_modules`, zodat deze route niet als een verse lockbestandinstallatie wordt gepresenteerd. De script voert geen normatieve referentievergelijking of visuele UI-acceptatie uit. Verschillen in systeemlibraries, compiler, toolchain of buildtimestamps kunnen ook bij dezelfde broncommit andere binary-hashes opleveren; de manifest maakt die verschillen onderzoekbaar, maar bewijst geen bit-identieke herbouw of BRL 9501-attestering.

`scripts/verify-nta.sh` is de afzonderlijke technische testgate. Stel `NTA_REFERENCE_PLAN` en `NTA_REFERENCE_CASE_DIR` in zodra een rechtmatig verkregen, onafhankelijk vastgesteld actueel W/U-plan beschikbaar is; zonder deze variabelen meldt het script dat de referentiegate is overgeslagen.
