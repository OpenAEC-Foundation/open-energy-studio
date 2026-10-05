# Frontendafhankelijkheden voor de NTA-desktopbuild — 5 oktober 2026

Op een schone snapshot van broncommit `bb7c1bd0b982c5debf713e6707439a35961d7d1a` zijn de frontendafhankelijkheden opnieuw vastgezet. Direct bijgewerkt: `fflate` naar `^0.8.3`, `sharp` naar `^0.35.5`, `vite` naar `^7.3.6` en `vitest` naar `^4.1.11`. Het lockbestand werkt ook onderliggende pakketten bij. Hiermee valt de directe ZIP64-kwetsbaarheid van [fflate](https://github.com/advisories/GHSA-px8p-9vwx-vf98) buiten de vastgezette versie; de overige updates zijn gekozen na de lokale npm-audit en vervolgens als geheel getest.

De eerste schone installatie van het oude lockbestand gaf 13 npm-auditmeldingen (2 laag, 4 middel, 7 hoog). `npm audit fix` faalde hier door een fout in npm 10.9.9 (`edgesOut` op `null`). Daarom zijn de vier directe pakketten afzonderlijk opgewaardeerd en is daarna de resterende transitieve boom vernieuwd. De nieuwe schone `npm ci` meldde **0 kwetsbaarheden op dat controlemoment**. Dit is geen permanente veiligheidsverklaring; de audit is afhankelijk van de actuele adviesdatabase.

Met het nieuwe `package.json` en `package-lock.json` op de volledige bronstand `bb7c1bd` slaagden in een aparte checkout:

- `npm ci` zonder extra vlaggen: 187 pakketten geïnstalleerd, 0 auditmeldingen;
- `npm run build`: TypeScript en Vite 7.3.6 geslaagd;
- `npx vitest run --testTimeout=20000 --maxWorkers=2`: 75 testbestanden en 505 tests geslaagd.

Deze controles betreffen de frontendafhankelijkheden en UI. Rust-kern, normatieve referentiegevallen en formele attestering zijn hierdoor niet opnieuw geverifieerd. Een eerdere volledige testpoging met vier workers is wegens gelijktijdige zware testprocessen afgebroken; de latere volledige run met twee workers slaagde. De runtimewaarschuwingen over enkele React-testupdates en een niet-unieke UI-sleutel bleven niet-fataal en staan los van de pakketinstallatie.

De eerste Tauri-bouw met het breed vernieuwde lockbestand werd geweigerd omdat de door npm gekozen Tauri-JavaScriptpakketten andere minorversies hadden dan de Rust-crates. Daarom zijn `@tauri-apps/api` op de beschikbare `2.10.1`, `@tauri-apps/cli` op `2.10.1`, `@tauri-apps/plugin-dialog` op `2.6.0` en `@tauri-apps/plugin-fs` op `2.4.5` exact vastgezet. Een npm-override en deduplicatie houden ook de twee indirecte API-kopieën op `2.10.1`; `npm ls` bevestigde één gedeelde versie. Een nieuwe `npm ci`, npm-audit zonder meldingen, TypeScript-/Vite-bouw en de [desktopdevbuild op `dc1b917`](nta8800-build-verificatie-2026-10-05-dc1b917.md) slaagden. De eerdere weigering is in het verificatiedossier bewaard.
