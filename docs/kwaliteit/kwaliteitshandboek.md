# Kwaliteitshandboek Open Energy Studio — NTA 8800

**Status:** concept van de softwareleverancier, klaar om vast te stellen (taak C2 in de [gereedheidsanalyse](../nta8800-brl9501-gereedheid.md)). Het handboek geldt pas na vaststelling door de attesthouder. Vul de rollen in, kies de archieflocatie en teken de vaststelling onderaan.

Dit handboek bundelt de procedures waarmee de attesthouder voldoet aan BRL 9501 hoofdstuk 5 en 6 (p. 9–10). Het verwijst naar de technische procedures die al in de repository staan en naar de organisatorische procedures in deze map. BRL-tekst wordt niet overgenomen; verwijzingen zijn naar artikel en pagina.

## 1. Rollen

| Rol | Verantwoordelijk voor | Naam (in te vullen) | Vervanger |
|---|---|---|---|
| Directie / attesthouder | Vaststelling van dit handboek, contract met de attesteringsinstelling, KvK (§5.1) | … | … |
| Releaseverantwoordelijke | Vrijgaven, versienummers, meldingen (§5.2, §6.3, §6.4) | … | … |
| Interne reviewer | Review van wijzigingen in de rekenkern en van releasenotes (vier-ogenprincipe) | … | … |
| Klachtcoördinator | [Klachtenprocedure](klachtenprocedure.md) (§6.5) | … | … |
| Registerbeheerder | [Register van licentiehouders](register-licentiehouders.md) (§6.2.1), archief | … | … |
| Vertegenwoordiger TC 9501 | Deelname aan de technische commissie van InstallQ (§5.5) | … | … |

Eén persoon mag meer rollen hebben, behalve dat de interne reviewer niet de auteur van de gereviewde wijziging is.

## 2. Procedures

| Onderwerp | BRL 9501 | Procedure | Bewijs |
|---|---|---|---|
| Wijzigen van de rekenkern | §4.3 (p. 7), §6.2 (p. 10) | Elke wijziging via git met review; een uitkomstwijziging krijgt een item in de [releasenotes](../nta8800-releasenotes.md) onder "Onuitgebracht"; [versiebeleid](../nta8800-versiebeheer.md) | git-geschiedenis, releasenotes |
| Versiediscipline | §4.3 (p. 7), §8.2 (p. 15) | `scripts/nta-kernel-version.mjs check` in de gate: geen uitkomstwijziging zonder MINOR-stap van `KERNEL_VERSION` | gatelog |
| Technische gate | §6.2–6.3 (p. 10) | `scripts/verify-nta.sh`: kern-, service-, Tauri- en frontendtests, optiedekking over alle uitgaven (`crates/nta8800-core/tests/option_coverage.rs`), openbare gevallen, MSRV, build, handleidingstempel | gatelog per vrijgave |
| Referentieberekeningen | §4.2 (p. 7), §6.3 (p. 10), §7.2 (p. 12–13) | Suites in `training-data/reference-suites/` (openbare gevallen A–F, RVO-voorbeeldwoningen; later ISSO 54), gedraaid door `reference_gate` in de gate | `reference-report.json`/`.md` per vrijgave |
| Vrijgave | §6.1, §6.3 (p. 10) | `scripts/release-nta.sh`: releasenotes per versie, gate, pakket, [leveringsdocument](../templates/nta8800-leveringsdocument.md), `SHA256SUMS`, lokale tag | `release/<tag>/` |
| Handleiding | §3.0 (p. 5), §4.4 (p. 8) | [Handleiding](../handleiding-nta8800/index.md) in de app en als HTML in elke vrijgave; versiestempel gecontroleerd door `scripts/nta-manual.mjs check` | gatelog, `release/<tag>/handleiding/` |
| Melden van wijzigingen | §5.2 (p. 9), §6.4 (p. 10) | [Meldprocedure wijzigingen](meldprocedure-wijzigingen.md) | meldregister, `release/<tag>/meldingen/` |
| Registratie rekenkern bij RVO | §5.2 (p. 9) | Na de eerste vrijgave met attest: `KERNEL_VERSION` registreren bij RVO; elke nieuwe rekenkernversie opnieuw (taak C5) | bevestiging RVO in het meldarchief |
| Bewaren | §5.3 (p. 9), §6.2–6.3 (p. 10) | [Archiefbeleid](archiefbeleid.md) | archief met `SHA256SUMS` |
| Licentiehouders | §6.2.1 (p. 10) | [Register van licentiehouders](register-licentiehouders.md) | register |
| Klachten | §6.5 (p. 10) | [Klachtenprocedure](klachtenprocedure.md) | klachtenregister |
| Minimale registratie | §5.4 (p. 9) | Afspraak met minstens één certificaathouder over een jaarlijkse registratie volgens BRL 9500-W en -U (taak C8) | afspraak en registratiebewijs |
| Interpretaties | §4.1 (p. 7) | Vastleggen in [normversies](../nta8800-normversies.md) en de vergelijkingsdocumenten; open vragen naar NEN ([vragen aan NEN](../nta8800-vragen-nen.md)) of TC 9501 | documenten in git |

## 3. Jaarlijkse review

Eén keer per jaar, en na elke normwijziging, voert de directie met de interne reviewer een review uit. Agenda:

1. Vrijgaven van het afgelopen jaar: is elke tag met een rekenwijziging gemeld ([meldprocedure](meldprocedure-wijzigingen.md) §6)?
2. Archief: `sha256sum -c` op beide kopieën, één oude versie herbouwd ([archiefbeleid](archiefbeleid.md) §5).
3. Klachten: aantal per oorzaak, termijnen, structurele maatregelen ([klachtenprocedure](klachtenprocedure.md) §7).
4. Register van licentiehouders: volledig en actueel ([register](register-licentiehouders.md) §5).
5. Referentieberekeningen en ISSO 54: oordeel per deeltest in de laatste vrijgave; nieuwe of gewijzigde deeltesten (§7.1.3, §7.2).
6. Open interpretatievragen en antwoorden van NEN of TC 9501.
7. [Gereedheidsanalyse](../nta8800-brl9501-gereedheid.md) bijwerken.
8. Dit handboek: nog actueel? Wijzigingen vastleggen met datum.

Het verslag gaat in het archief en is beschikbaar voor de attesteringsinstelling.

## 4. Vaststelling

| Versie | Datum | Wijziging | Vastgesteld door |
|---|---|---|---|
| concept | 9 oktober 2026 | Opgesteld door de softwareleverancier | — (nog niet vastgesteld) |
