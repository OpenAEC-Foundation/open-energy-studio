# Testrapport NTA 8800-rekenprogramma

Volgens het [testprotocol](../nta8800-testprotocol.md). `scripts/release-nta.sh` vult de velden tussen dubbele accolades in en zet dit rapport in `release/<tag>/testrapport.md`; de rest vult de tester in.

## Identificatie

| Veld | Waarde |
|---|---|
| Programmaversie | {{PROGRAM_VERSION}} |
| Rekenkernversie (`KERNEL_VERSION`) | {{KERNEL_VERSION}} |
| Tag | {{TAG}} |
| Commit | {{COMMIT}} |
| Datum vrijgave | {{RELEASE_DATE}} |
| Gate (automatisch) | {{GATE_STATUS}} |
| Proefvrijgave | {{DRY_RUN}} |
| Tester | |
| Testmachine en besturingssysteem | |

## Automatische testniveaus (§2)

Het gatelog staat in `gate.log`, het referentierapport in `evidence/reference/`. De tabel onder "Gate-uitvoer" is door `scripts/release-nta.sh` uit die twee bestanden gehaald (ook als `gate-samenvatting.json`); neem de aantallen per niveau daaruit over.

| | Niveau | Resultaat (aantal geslaagd / totaal) | Opmerking |
|---|---|---|---|
| a | Unittests kern | | |
| b | Differentiële tests per uitgave | | |
| c | Openbare gevallen A–O | | |
| d | Referentiesuites (openbare gevallen, RVO, ISSO 54) | | oordeel: |
| e | Optiedekking | | |
| f | Numerieke fuzz | | |
| g | Grote projecten | | |
| h | Robuustheid | | |
| i | Service, API, MCP | | |
| j | Desktop-opdrachten | | |
| k | Frontendtests | | |
| l | MSRV 1.77.2 | | |
| m | Versie- en handleidingstempel | | |
| n | App tegen de echte rekenkern (`kernel_once`) | | |
| o | Formulierstanden en gewiste velden door de kern | | |

### Gate-uitvoer (automatisch ingevuld)

{{GATE_SUMMARY}}

## ISSO 54 (§3)

| Veld | Waarde |
|---|---|
| Suite en versie | |
| Deeltesten met verwachting / binnen 1 % | |
| Afwijkingen en vervolg | |

## End-to-endvergelijking (§5)

| Veld | Waarde |
|---|---|
| Vergeleken met vrijgave of commit | |
| Aantal gewijzigde getallen | |
| Alle wijzigingen verklaard door de releasenotes? | ja / nee |
| Onverklaarde wijzigingen | |

## Handmatige acceptatietests (§4)

| Nr | Test | Geslaagd | Opmerking |
|---|---|---|---|
| H1 | Installatie | ☐ | |
| H2 | Woning van opname tot label | ☐ | |
| H3 | Utiliteit van opname tot label | ☐ | |
| H4 | Wisselen van normversie | ☐ | |
| H5 | Invoer per raam | ☐ | |
| H6 | Bijlage AA | ☐ | |
| H7 | Bron en bewijs, dossier | ☐ | |
| H8 | Herlabelen | ☐ | |
| H9 | Verklaringen k en l | ☐ | |
| H10 | EP-Online-overzicht | ☐ | |
| H11 | Opslaan en sluiten | ☐ | |
| H12 | Recent na herstart | ☐ | |
| H13 | Nieuwere bestandsversie | ☐ | |
| H14 | Import Uniec 3 en VABI | ☐ | |
| H15 | Handleiding | ☐ | |
| H16 | API | ☐ | |
| H17 | MCP | ☐ | |
| H18 | Gemeenschappelijke ruimte | ☐ | |
| H19 | Vensterbreedtes | ☐ | |

## Afwijkingen

| Nr | Omschrijving | Ernst | Besluit (herstellen vóór vrijgave / bekende beperking / geen actie) |
|---|---|---|---|
| | | | |

## Aftekening

| Rol | Naam | Datum | Akkoord |
|---|---|---|---|
| Tester | | | ☐ |
| Interne reviewer | | | ☐ |
| Vrijgever | | | ☐ |
