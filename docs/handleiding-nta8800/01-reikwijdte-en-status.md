# 1. Reikwijdte en status

## Wat het programma berekent

De rekenkern (`crates/nta8800-core`, Rust) rekent de energieprestatie volgens **NTA 8800:2025+C1:2026**, de aangewezen uitgave. Ter vergelijking kan een project ook in NTA 8800:2024, 2023, 2022 of 2020+A1 rekenen; zo'n uitkomst is niet registreerbaar (zie [hoofdstuk 10](10-normversies.md)). Dat gaat van invoer tot indicatoren:

| Onderdeel | Normbasis |
|---|---|
| Indicatoren EP1 (BENG 1), EP2 (BENG 2), EP3 / RER (BENG 3), TOjuli, CO₂, finaal energiegebruik | hoofdstuk 5, §5.7, §5.9 |
| Gebruiksoppervlakte, verliesoppervlak A_ls, rekenzones | hoofdstuk 6 |
| Maandelijkse warmte- en koudebehoefte, interne winst, zonwinst, dynamische beglazing, serres | hoofdstuk 7, bijlagen A, B en D |
| Transmissie, constructies, grond, onverwarmde ruimten | hoofdstuk 8, bijlagen C, E–I en L |
| Ruimteverwarming: afgifte, distributie, opwekking (ketel, warmtepomp, hybride, WKK, biomassa, externe warmte) | hoofdstuk 9, bijlagen M, N, O, Q, V en W |
| Koeling (methoden 1, 2 en 3) | hoofdstuk 10 |
| Ventilatie, infiltratie, luchtbehandeling | hoofdstuk 11 |
| Bevochtiging en ontvochtiging | hoofdstuk 12 |
| Warm tapwater, inclusief zonneboilers en meerdere opwekkers | hoofdstuk 13, bijlagen T, U en W |
| Verlichting (utiliteit) | hoofdstuk 14 |
| PV en windenergie | hoofdstuk 16 |
| Klimaat en belemmering | hoofdstuk 17 |
| Externe levering (warmte- en koudenetten) | bijlage P |
| Afronding | bijlage X |
| ZEB-indicator (informatief) | bijlage AB |

Daarnaast bevat het programma:
- **Basisopname:** voor bestaande woningen volgens ISSO 82.1 (7e druk, met erratum) en voor utiliteitsgebouwen volgens ISSO 75.1 (7e druk). Zie [hoofdstuk 2](02-basisopname.md).
- **Labelgegevens en labelklasse:** volgens de Omgevingsregeling, art. 5.11–5.13a en bijlagen IX/X (tot 2024 de Regeling energieprestatie gebouwen).
- **Bbl-toets:** de BENG-grenswaarden, ook voor gemengde functies.
- **Maatwerkadvies:** volgens BRL 9500-MWA en ISSO 82.2/75.2, met het renovatiepaspoort voor woningen en utiliteit. Zie [hoofdstuk 6](06-maatwerkadvies.md).
- **Registratie, herlabelen en dossier:** registratiegegevens, herlabelen (BRL 9500 bijlagen 6a/6b) en een projectdossier met manifest. Zie [hoofdstuk 7](07-herlabelen-registratie-dossier.md).

## Wat het programma niet doet

**Buiten de reikwijdte**
- Productbepaling (bijlage J) en meetregels (bijlage K). Productwaarden en meetresultaten zijn invoer, met een bronvermelding.
- Windenergie rekent het programma op nul (16.17).
- Detailopname-routes van ISSO 82.1/75.1. Gebruik de projectberekening (hoofdstuk 3) voor een detailopname.
- Ventilatie per rekenzone in de utiliteitsopname. Eén gebouwsysteem bedient alle zones; zie [hoofdstuk 2](02-basisopname.md).

**Afhankelijk van externe gegevens die niet beschikbaar zijn**
- Uurwaarden van het klimaat voor §17.3.8. Belemmeringsfactoren worden daarom als opgegeven waarden aanvaard.
- NEN-EN-ISO 6946 tabel 8, voor luchtspouwen dunner dan 20 mm. Geef zo'n laag op als R-waarde.
- De kostenmodelbeschrijving van ISSO (rapport 110293) en een locatieklimaat voor het maatwerkadvies.
- Het uitwisselformaat (XSD) van EP-Online voor registratie. Het programma registreert niet zelf.
- Officiële referentiegevallen: de EDR-testen van ISSO-publicatie 54 versie 5.0:2026, de testset waar BRL 9501 naar verwijst, met hun verwachte uitkomsten.

## Attesteringsstatus

- Het programma heeft **geen BRL 9501-attest**. De uitvoer draagt `attestStatus: "unattested"`.
- Elke berekening krijgt de status `calculated_unverified`, of `calculated_legacy_edition` in een oudere uitgave. Er is nog geen toets tegen officiële referentiegevallen; wel is de kern per uitgave vergeleken met openbare BENG-rapporten ([`docs/nta8800-vergelijking-openbare-rapporten.md`](../nta8800-vergelijking-openbare-rapporten.md)).
- `readyForRegistration` blijft onwaar tot er een attestnummer is ingevuld (`SOFTWARE_ATTEST_NUMBER` in `src/core/nta/Registration.ts`). De uitvoer toont los daarvan of het dossier compleet is (`dossierComplete`).
- De kern is wel onafhankelijk nagerekend: met eigen implementaties vanuit de normpagina's en per maand vergeleken. De stand staat in [`docs/nta8800-verificatiestatus.md`](../nta8800-verificatiestatus.md).
- Waar de norm onduidelijk of strijdig is, legt de kern de gekozen lezing vast. Die lezingen staan in de bijlage "Interpretaties" van het rekenrapport (zie [hoofdstuk 4](04-uitvoer.md)).
