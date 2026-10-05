# NTA 8800 devbuild — bronstand `62a4a29`

Op 5 oktober 2026 is `scripts/build-nta-dev.py --skip-npm-ci` op de schone bronstand `62a4a29d57b65067ddcbeee617e9347d0db60e19` geslaagd. De Vite-productiebouw, Rust-servicebinaries en Linux amd64 Tauri-debugbundel zijn daadwerkelijk gebouwd. `--skip-npm-ci` betekent dat bestaande `node_modules` zijn gebruikt; de standaardroute met een verse `npm ci` is hiermee niet getest.

Het [machineleesbare bouwmanifest](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-build_62a4a29_linux-amd64.json) heeft SHA-256 `19e6592c961d991ff0f99624de0fffcebb4895a0df4a16f10872242fce0ebc16` en bevat de volledige broncommit, doelversie `NTA 8800:2025+C1:2026`, toolversies, lockbestandhashes, pakketmetadata en de volgende artefacten:

| Artefact | SHA-256 |
| --- | --- |
| [Linux desktop-debugpakket](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-dev_62a4a29_linux-amd64.deb) | `3f8e2abbdb4d58d1320a598b79130b2936aeee5cbc94999c94eb0d594d7126cc` |
| [HTTP-API](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-api_62a4a29_linux-amd64) | `df65d2e2fde1d5990fd7ad66ab841713487db332a0e05ef346dfcbbd45c3fecd` |
| [MCP-server](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-mcp_62a4a29_linux-amd64) | `f51d957574b8adbfeff5e76b4b22d094a681ddf973d0effbefff200bd69730b6` |
| [Referentiegate](/home/aia04/Documents/Codex/2026-09-28/ku/outputs/oes-nta8800-reference-gate_62a4a29_linux-amd64) | `9f1a60e37c69a0ef833092b2f6bf78bb4632d4fc4dc943096787eb389c30407d` |

De vier hashes en bestandsgroottes zijn onafhankelijk van de manifestcode opnieuw berekend en kwamen overeen. Een tweede aanroep voor dezelfde commit en uitvoermap werd vóór het bouwen geweigerd, zodat het bestaande dossier niet stilzwijgend wordt overschreven. De laatste volledige technische gate op de ongewijzigde kernel- en UI-broncode was op `e2d2a69`: 763 kern- en drie integratietests, 57 service- en vijf CLI-tests, 452 frontendtests, Rust 1.77.2-controle, format, Clippy, Tauri-check en TypeScript.

Het manifest meldt `referenceVerified=false` en `attestStatus=unattested`. Officiële actuele referentie-uitkomsten, visuele UI-acceptatie en formele externe toets ontbreken. Dit dossier bewijst uitsluitend deze lokale technische bouw en haar herleidbaarheid.
