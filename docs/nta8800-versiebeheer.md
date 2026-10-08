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
| Programmaversie | `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` | `0.1.6-alpha` | De versie van de desktop-app en de gebruikersinterface. |
| Rekenkernversie (`KERNEL_VERSION`) | `crates/nta8800-core/Cargo.toml`, in de kern beschikbaar als `KERNEL_VERSION` | `0.1.0` | De versie van de rekenkern: het deel van het versienummer dat BRL 9501 §5.2 aan de NTA 8800-versie koppelt. |
| Normversie (`TARGET_NORM_VERSION`) | `crates/nta8800-core/src/lib.rs` | `NTA 8800:2025+C1:2026` | De normtekst waarop de rekenkern is gebaseerd. |

Beide versienummers volgen het schema `MAJOR.MINOR.PATCH`, eventueel met een achtervoegsel voor een voorlopige uitgave (`-alpha`, `-beta`).

| Soort wijziging | Rekenkernversie | Programmaversie |
| --- | --- | --- |
| Nieuwe normversie, wijzigingsblad of correctieblad van NTA 8800, of een nieuwe bindende interpretatie die uitkomsten verandert | MAJOR | MAJOR of MINOR |
| Rekenwijziging binnen dezelfde normversie die een uitkomst kan veranderen (foutherstel, nieuwe route, nieuwe validatie die invoer afwijst) | MINOR | MINOR |
| Wijziging zonder invloed op uitkomsten (documentatie, meldteksten, prestaties) | PATCH | PATCH |
| Alleen gebruikersinterface, rapport of export | ongewijzigd | MINOR of PATCH |

Elke wijziging die een uitkomst kan veranderen, staat in de [releasenotes](nta8800-releasenotes.md) onder de kop "Resultaten die veranderen". BRL 9501 §5.2 vraagt dat de attesthouder de attesteringsinstelling en de licentiehouders hierover direct informeert. §6.4 (p. 10) vraagt een releasenote bij elke wijziging van de berekeningsmethode of de gebruikersinterface.

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
- **Hoe dit hier wordt bewaard:** de broncode van elke versie staat in de git-geschiedenis, en de releasenotes leggen de wijzigingen vast. Een vast bewaarbeleid voor de gebouwde releasebestanden (de `.deb` en het installatiepakket), met een git-tag per vrijgave, moet de attesthouder nog vastleggen.
- **Leveringsdocument (BRL 9501 §6.1, p. 10):** het programma wordt geleverd met een leveringsdocument met het versienummer uit het attest. Dat document ontbreekt nog, omdat er geen attest is.

## Registraties van de attesthouder (BRL 9501 §6.2–6.3, p. 10)

- **Wijzigingslogboek:** de git-geschiedenis met de releasenotes.
- **Testresultaten per wijziging:**
  - de lokale gate `scripts/verify-nta.sh`, met Rust-tests, Clippy, de minimale Rust-versie, frontendtests en de productiebouw;
  - de vaste fuzz- en robuustheidstests.
- **Nog niet aanwezig:** de EDR-testen van ISSO-publicatie 54 versie 5.0:2026 (BRL 9501 §4.2, p. 7), waarvan de uitkomsten bij elke vrijgave moeten worden bewaard. Zie het [attestdossier](nta8800-attestdossier.md).
