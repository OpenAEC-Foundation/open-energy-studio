# Versiebeheer van het NTA 8800-rekenprogramma

Dit document beschrijft het versiebeleid dat BRL 9501 van 29-05-2026 in §4.3 (p. 7) van de documentatie van een rekenprogramma vraagt. Het gaat om vier punten:
- hoe versienummers verschillen naar omvang en aard van een wijziging;
- het versienummer zelf;
- hoe de uitwisselbaarheid tussen versies is geborgd;
- hoe het versienummer is opgebouwd.

Daarnaast behandelt het de registratie van de rekenkernversie bij RVO (§5.2, p. 9) en de bewaartermijn van oude versies (§5.3, p. 9). Dit is een werkdocument: het programma is niet geattesteerd.

De [BRL 9501-gereedheid](nta8800-brl9501-gereedheid.md) loopt alle eisen van BRL 9501 na en geeft wat een attest nog tegenhoudt. De [globale beschrijving van het rekenprogramma](nta8800-programmabeschrijving.md) geeft het overzicht voor de attesteringsinstelling.

## Opbouw van het versienummer

Een berekening draagt twee versienummers.

| Deel | Waar vastgelegd | Voorbeeld | Betekenis |
| --- | --- | --- | --- |
| Programmaversie | `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` | `0.2.0` | De versie van de desktop-app en de gebruikersinterface. |
| Rekenkernversie (`KERNEL_VERSION`) | `crates/nta8800-core/Cargo.toml`, in de kern beschikbaar als `KERNEL_VERSION` | `0.4.0` (onuitgebracht; laatste uitgave 0.3.0) | De versie van de rekenkern: het deel van het versienummer dat BRL 9501 §5.2 aan de NTA 8800-versie koppelt. |
| Normversie (`TARGET_NORM_VERSION`) | `crates/nta8800-core/src/lib.rs` | `NTA 8800:2025+C1:2026` | De normtekst waarop de rekenkern is gebaseerd. |

Beide versienummers volgen het schema `MAJOR.MINOR.PATCH`, eventueel met een achtervoegsel voor een voorlopige uitgave (`-alpha`, `-beta`).

| Soort wijziging | Rekenkernversie | Programmaversie |
| --- | --- | --- |
| Nieuwe normversie, wijzigingsblad of correctieblad van NTA 8800, of een nieuwe bindende interpretatie die uitkomsten verandert | MAJOR | MAJOR of MINOR |
| Rekenwijziging binnen dezelfde normversie die een uitkomst kan veranderen (foutherstel, nieuwe route, nieuwe validatie die invoer afwijst) | MINOR | MINOR |
| Wijziging zonder invloed op uitkomsten (documentatie, meldteksten, prestaties) | PATCH | PATCH |
| Alleen gebruikersinterface, rapport of export | ongewijzigd | MINOR of PATCH |

Elke wijziging staat in de [releasenotes](nta8800-releasenotes.md), gegroepeerd per rekenkernversie. BRL 9501 §5.2 vraagt dat de attesthouder de attesteringsinstelling en de licentiehouders hierover direct informeert. §6.4 (p. 10) vraagt een releasenote bij elke wijziging van de berekeningsmethode of de gebruikersinterface.

## Afdwingen in de gate

`scripts/nta-kernel-version.mjs check` draait als eerste stap van `scripts/verify-nta.sh`. Het controleert de releasenotes tegen `KERNEL_VERSION`:
- elke uitgegeven versie is een sectie `## Rekenkern X.Y.Z — <datum>` met de regel `<!-- kernel-version: X.Y.Z -->`, aflopend door het bestand;
- `KERNEL_VERSION` is nooit lager dan de laatste uitgave;
- staat er onder "Onuitgebracht" een item dat uitkomsten verandert (elk item, tenzij de titel "(geen rekenwijziging)" vermeldt), dan moet `KERNEL_VERSION` minstens een MINOR-stap boven de laatste uitgave staan;
- een verhoogde `KERNEL_VERSION` vraagt minstens één onuitgebracht item.

Daarna controleert `scripts/nta-manual.mjs check` de handleiding: de stempel `<!-- handleiding: rekenkern X.Y.Z -->` in `docs/handleiding-nta8800/index.md` moet gelijk zijn aan `KERNEL_VERSION`. Wie de rekenkernversie ophoogt, loopt de handleiding na voor die versie en past de stempel (en de zichtbare regel eronder) aan; zo gaat een nieuwe kern nooit uit met een handleiding die er niet voor is nagelopen (BRL 9501 §4.4).

Eén ophoging per uitgaveronde volstaat: latere uitkomstwijzigingen vóór de vrijgave vallen onder dezelfde nieuwe versie. Een nieuwe normversie, een wijzigingsblad of een bindende interpretatie vraagt een MAJOR-stap. De gate kan dat onderscheid niet zelf maken; dat blijft een beoordeling bij het ophogen.

**Waarom 0.2.0.** `KERNEL_VERSION` stond van 30 september tot 8 oktober 2026 op 0.1.0, terwijl in die periode veel wijzigingen uitkomsten veranderden (onder meer de oudere uitgaven en herstelde routes). Achteraf is niet meer per commit vast te stellen welke stand een "0.1.0"-stempel had. Daarom bundelt 0.2.0 alle wijzigingen van die periode, en begint het afgedwongen beleid daar. Een project met een 0.1.0-stempel van na 30 september moet bij twijfel opnieuw worden berekend.

## Vrijgeven

`scripts/release-nta.sh` maakt een vrijgave (BRL 9501 §5.3, §6.1–6.3):
1. de onuitgebrachte items gaan onder "Rekenkern `KERNEL_VERSION`" in de releasenotes, in één commit (alleen als de rekenkernversie nieuw is);
2. de volledige gate draait; het log en het rapport van de referentiesuites komen in het archief;
3. de desktoppakketten worden offline gebouwd;
4. de handleiding gaat mee als één HTML-bestand (`handleiding/handleiding-nta8800-<programmaversie>.html`, afbeeldingen ingebed) met de programma- en rekenkernversie erin, en als PDF wanneer chromium of wkhtmltopdf op de buildmachine staat;
5. het [leveringsdocument](templates/nta8800-leveringsdocument.md) wordt gevuld uit `src/core/nta/attest.json`, de versienummers en de SHA-256 van de pakketten en de handleiding; daarnaast een `manifest.json` en `SHA256SUMS`;
6. er komt een lokale, geannoteerde git-tag `oes-v<programmaversie>-kernel-v<rekenkernversie>`.

Het archief staat in `release/<tag>/` (niet in git). Pushen van de tag en publiceren van het archief zijn een aparte, handmatige stap. `--dry-run` doet alles zonder commit en tag, en schrijft naar `release/dry-run-<tag>/`. Draai een echte vrijgave zonder `NTA_SKIP_MSRV`, zodat de MSRV-stap in het gate-log staat.

### Uitgave 0.3.0 / rekenkern 0.4.0 — stappen

Op 10 oktober 2026 is deze uitgave als proef doorlopen: programmaversie opgehoogd naar 0.3.0 en `scripts/release-nta.sh --dry-run` met de Rust 1.77.2-stap. Dat leverde `release/dry-run-oes-v0.3.0-kernel-v0.4.0/` op met `handleiding-nta8800-0.3.0.html`, `Open Energy Studio_0.3.0_amd64.deb`, een leveringsdocument en testrapport met 0.3.0 / 0.4.0 en `SHA256SUMS`. De proef vond één fout: in een verse checkout liep de gate vast op de Tauri-stap omdat `dist/` nog niet bestond; de gate bouwt de frontend nu eerst.

Stappen voor de echte uitgave, in deze volgorde, vanaf een schone `nta8800-kernel`:

1. `node scripts/bump-version.js 0.3.0` — zet `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` en `src-tauri/Cargo.lock` op 0.3.0.
2. Werk de versietabel bij in `README.md`, `wiki/Home.md`, `wiki/Versiebeheer-en-releases.md` en `wiki/_Footer.md` (programma 0.3.0, rekenkern 0.4.0, handleiding gestempeld voor rekenkern 0.4.0, tag `oes-v0.3.0-kernel-v0.4.0`) en de voorbeeldwaarde in de tabel "Opbouw van het versienummer" hierboven.
3. `git commit -am "Program version 0.3.0"` (met de gebruikelijke Co-Authored-By-regel als dat de werkwijze is).
4. `scripts/release-nta.sh` — zonder `NTA_SKIP_MSRV`; dit draait de volledige gate, bouwt het `.deb`, zet de onuitgebrachte releasenotes onder "Rekenkern 0.4.0" (eigen commit) en maakt de lokale tag `oes-v0.3.0-kernel-v0.4.0`. Het archief komt in `release/oes-v0.3.0-kernel-v0.4.0/`.
5. Controleer `release/oes-v0.3.0-kernel-v0.4.0/github-release.md`. Het script schrijft die tekst voor de GitHub-release zelf: versies, attest-status, de items van "Rekenkern 0.4.0" en de SHA-256 van het `.deb`, de handleiding, het leveringsdocument en het testrapport. Pas hem zo nodig aan.
6. `git push origin nta8800-kernel` en `git push origin oes-v0.3.0-kernel-v0.4.0` — vraagt GitHub-credentials; dat doet de gebruiker zelf.
7. `gh release create oes-v0.3.0-kernel-v0.4.0 --verify-tag --prerelease --title "Open Energy Studio 0.3.0, NTA 8800 kernel 0.4.0" --notes-file release/oes-v0.3.0-kernel-v0.4.0/github-release.md release/oes-v0.3.0-kernel-v0.4.0/pakketten/*.deb release/oes-v0.3.0-kernel-v0.4.0/handleiding/*.html release/oes-v0.3.0-kernel-v0.4.0/leveringsdocument.md release/oes-v0.3.0-kernel-v0.4.0/testrapport.md release/oes-v0.3.0-kernel-v0.4.0/SHA256SUMS` — of maak de release via de GitHub-website. Doe dit direct na het pushen van de tag: de workflow maakt na de gate (ruim een kwartier) zelf een concept-release als er nog geen is. Bestaat de release al als concept, gebruik dan `gh release edit oes-v0.3.0-kernel-v0.4.0 --notes-file release/oes-v0.3.0-kernel-v0.4.0/github-release.md` en `gh release upload oes-v0.3.0-kernel-v0.4.0` met dezelfde bestanden. Daarna bouwt `.github/workflows/release.yml` de pakketten voor Windows en macOS (zie hieronder). Het `.deb` dat je zelf uploadt blijft staan: de workflow vervangt geen bestand dat al bij de release staat, zodat de SHA-256 in het leveringsdocument en `SHA256SUMS` klopt.
8. Controleer in de Actions-run dat **verify** slaagt (tag = `oes-v<package.json>-kernel-v<KERNEL_VERSION>`).

### Na het pushen van de tag: GitHub Actions

Een gepushte tag `oes-v*` (of een oude `v*`-tag) start `.github/workflows/release.yml`:
1. **verify** controleert dat de tag hoort bij de broncode: `oes-v<versie in package.json>-kernel-v<KERNEL_VERSION>`. Daarna draaien de controle van de releasenotes en de handleidingstempel, de controle dat de uitgebrachte secties van de releasenotes sinds de vorige tag ongewijzigd zijn (dezelfde als in `scripts/release-nta.sh`), en de volledige gate (`scripts/verify-nta.sh`, ook de rekenkern op Rust 1.77.2). Het gate-bewijs komt als artefact bij de run.
2. **release** gebruikt de GitHub-release van de tag als die al bestaat, bijvoorbeeld met `release/<tag>/github-release.md` als tekst; de tekst blijft dan ongemoeid. Bestaat hij niet, dan maakt de workflow een concept (draft, pre-release) met een korte tekst die je vóór publiceren vervangt.
3. **build** bouwt per platform de installatiepakketten en hangt ze aan de release: Linux (`.deb`, `.rpm`, AppImage), macOS (universele `.dmg`) en Windows (NSIS `-setup.exe` en `.msi`). Windows-pakketten worden ondertekend als de Azure-geheimen in de repository staan. Een bestand dat al bij de release staat, zoals het `.deb` uit `scripts/release-nta.sh`, wordt niet vervangen; wil je het opnieuw bouwen, verwijder het dan eerst van de release.
4. **checksums** zet `SHA256SUMS-installers.txt` over alle geüploade pakketten bij de release.

Een concept dat de workflow zelf maakte, blijft een concept: publiceren is een handmatige beslissing. Met *Run workflow* (invoer: een bestaande tag) bouw je de pakketten opnieuw. `.github/workflows/ci.yml` draait dezelfde gate bij elke push en pull request naar `main` en `nta8800-kernel`, en maakt zelf geen releases meer. Beide workflows roepen voor de gate alleen `scripts/verify-nta.sh` aan, zonder eigen voorbereidende stappen zoals een aparte frontendbuild, zodat CI zich gedraagt als een verse lokale checkout (`src/__tests__/ci-parity.test.ts`).

Het attestnummer, de identificatiecode en de attesteringsinstelling staan op één plek: `src/core/nta/attest.json`. De app (registratieblok, rapport) en het leveringsdocument lezen ze daar.

## Vastleggen per berekening

Elke berekening legt de versie vast waarmee zij is gemaakt:
- **Projectbestand:** het `.oes.json`-bestand bewaart een kernelstempel met `kernelVersion`, `targetNormVersion` en `inputFingerprint` (`src/core/io/ProjectSerializer.ts`).
- **Openen van een project:** de app vergelijkt de bewaarde stempel met de huidige kern. Ze waarschuwt bij een andere kern- of normversie, en ook bij gewijzigde invoer met dezelfde versie.
- **Registratieblok:** dit blok bewaart de programma-identiteit (`registration.software`): naam, programmaversie, rekenkernversie en, na attestering, het attestnummer (Regeling art. 5 lid 1 onder b).
- **Herlabelen:** bij herlabelen blijft de identiteit van de oorspronkelijke berekening behouden. De kern weigert herlabelen met een andere rekenkern (`relabel_kernel_version_differs`, `relabel_software_kernel_differs`).
- **Uitvoer:** het rekenrapport, de kernuitvoer in het projectdossier en het manifest (`manifest.json`) vermelden de normversie en de vingerafdruk van de invoer.

## Uitwisselbaarheid tussen versies

Een nieuwere programmaversie moet projecten van een oudere versie kunnen lezen, zonder dat de vastgelegde berekening verandert.

- **Toevoegingen in het invoerformaat:** nieuwe velden worden toegevoegd met een standaardwaarde (`#[serde(default)]` in de kern). Een bestaand bestand blijft daardoor leesbaar.
- **Onbekende velden:** de invoerblokken van de kern weigeren velden die zij niet kennen (`deny_unknown_fields`). Een bestand uit een nieuwere versie wordt daardoor niet stil verkeerd gelezen door een oudere.
- **Openen van ontbrekende onderdelen:** bij het openen vult `normalizeProject` ontbrekende lijsten aan. Een volledig bestand blijft ongewijzigd, zodat de vingerafdruk gelijk blijft.
- **Migraties:** een migratie die invoer aanpast, staat in de releasenotes en is getest. Een voorbeeld is `migrateLegacyRelabel` voor herlabelprojecten van vóór 3 oktober 2026.
- **Nieuwe blokkerende regels:** de releasenotes noemen in de rubriek "Opgeslagen projecten die nu onvolledig worden" welke bestaande projecten na een update een gat krijgen.
- **Uitkomsten bij een nieuwe rekenkern:** een nieuwe rekenkern kan andere uitkomsten geven voor dezelfde invoer. Het kernelstempel maakt dat zichtbaar. Een geregistreerd label blijft verbonden aan de rekenkern waarmee het is berekend.

## Registratie bij RVO en bewaren van oude versies

- **Registratie bij RVO (BRL 9501 §5.2, p. 9):** de rekenkernversie wordt bij RVO geregistreerd. Dat is een organisatorische taak van de attesthouder. Het in te dienen nummer is `KERNEL_VERSION` samen met `TARGET_NORM_VERSION`.
- **Bewaren na een normwijziging (BRL 9501 §5.3, p. 9):** de attesthouder bewaart de oude rekenversie minstens drie jaar na een normwijziging. Hij houdt die beschikbaar voor de licentiehouders of voert op verzoek de controleberekening zelf uit.
- **Bewaren voor herlabelen (BRL 9500-W §4.2.4, p. 24; 9500-U p. 20):** de certificaathouder houdt de oorspronkelijke rekenkern minstens 24 maanden na de opname beschikbaar.
- **Hoe dit hier wordt bewaard:** de broncode van elke versie staat in de git-geschiedenis, en de releasenotes leggen de wijzigingen per rekenkernversie vast. `scripts/release-nta.sh` maakt per vrijgave een git-tag en een archief met de pakketten en hun SHA-256. Waar die archieven minstens drie jaar worden bewaard, moet de attesthouder nog vastleggen.
- **Leveringsdocument (BRL 9501 §6.1, p. 10):** het programma wordt geleverd met een leveringsdocument met het versienummer uit het attest. Het wordt bij elke vrijgave gegenereerd; het attestnummer en de identificatiecode staan er als "nog niet toegekend" in tot het attest er is.

## Registraties van de attesthouder (BRL 9501 §6.2–6.3, p. 10)

- **Wijzigingslogboek:** de git-geschiedenis met de releasenotes.
- **Testresultaten per wijziging:**
  - de lokale gate `scripts/verify-nta.sh`, met Rust-tests, Clippy, de minimale Rust-versie, frontendtests en de productiebouw;
  - de vaste fuzz- en robuustheidstests;
  - de referentiesuites in `training-data/reference-suites/` (openbare gevallen A–O, RVO-voorbeeldwoningen, ISSO 54 versie 2.0). Elke gate schrijft hun rapport, met commit, `KERNEL_VERSION` en SHA-256 per invoer en uitvoer, naar `nta-evidence/gate/reference/`; een vrijgave bewaart het in haar archief.
- **Nog niet aanwezig:** de EDR-testen van ISSO-publicatie 54 versie 5.0:2026 (BRL 9501 §4.2, p. 7), waarvan de uitkomsten bij elke vrijgave moeten worden bewaard. Zie het [attestdossier](nta8800-attestdossier.md).
