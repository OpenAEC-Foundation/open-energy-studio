# Gebruikershandleiding NTA 8800 — Open Energy Studio

<!-- handleiding: rekenkern 0.2.0 -->
*Versie: bij rekenkern 0.2.0. De app en de handleiding van een release tonen daarnaast de programmaversie.*

Deze handleiding beschrijft het NTA 8800-deel van Open Energy Studio:
- de rekenkern;
- de invoerformulieren;
- de basisopname;
- het maatwerkadvies;
- herlabelen;
- het projectdossier.

Ze is geschreven als programmadocumentatie voor een softwareattest (BRL 9501). Ze volgt de code van de branch `nta8800-kernel` per 8 oktober 2026, na het UI-herontwerp (werkstappen in plaats van het lint) en met de oudere uitgaven van NTA 8800 in de kern.

De handleiding citeert geen tekst uit NTA 8800, ISSO-publicaties of BRL-documenten. Verwijzingen noemen alleen paragraaf-, formule-, tabel- en paginanummers. Houd de bronnen zelf bij de hand.

## Inhoud

0. [Werken met het programma](00-werken-met-het-programma.md): welkomstscherm, werkstappen, Ctrl+K, contextpaneel, Controle en Ga naar, Basis/Alle velden, de toepasbalk, Gereedschap, thema's en sneltoetsen.
1. [Reikwijdte en status](01-reikwijdte-en-status.md): wat het programma berekent, wat niet, en de attestering.
2. [Basisopname](02-basisopname.md): woningen (ISSO 82.1) en utiliteitsgebouwen (ISSO 75.1), met rekenzones.
3. [Projectberekening](03-projectberekening.md): het projectmodel en alle onderdelen van het NTA-invoerformulier.
4. [Uitvoer](04-uitvoer.md): indicatoren, label, Bbl-toets, energie per functie, meldingen en de bijlage met interpretaties.
5. [Validatie en meldingen](05-validatie.md): statussen, gaten en waarschuwingen, invoergrenzen en het vangnet voor niet-eindige uitkomsten.
6. [Maatwerkadvies](06-maatwerkadvies.md): sjablonen, handmatige wijzigingen, pakketten en het adviesrapport.
7. [Herlabelen, registratie en dossier](07-herlabelen-registratie-dossier.md):
   - de BRL 9500-bijlagen 6a/6b, het origineel en de vergelijking;
   - het bewijs;
   - de registratiegegevens en de dossierexport.
8. [Versies en verwijzingen](08-versies-en-verwijzingen.md): kernversie, invoervingerafdruk, het bewaren van oudere builds en de overige documentatie.
9. [Bestanden en uitwisseling](09-bestanden-en-uitwisseling.md): opslaan en openen van `.oes.json`, de voorbeeldprojecten, UNIEC3-export en -import en de exports onder Rapport & dossier › Exports.
10. [Normversies](10-normversies.md): rekenen in NTA 8800:2025+C1, 2024, 2023, 2022 of 2020+A1, invoer per uitgave en waarom een oudere uitgave niet registreerbaar is.

## Belangrijk vooraf

- Het programma is **niet geattesteerd** volgens BRL 9501. Uitkomsten hebben de status `calculated_unverified` (of `calculated_legacy_edition` in een oudere uitgave) en zijn geen geregistreerd energielabel.
- Een energielabel ontstaat pas na registratie door een gecertificeerd adviseur (BRL 9500), met een geattesteerd rekenprogramma.
- De voorbeeldprojecten in `training-data/` zijn fictief.

## Nog te doen: schermafdrukken

Alle afbeeldingen waar de hoofdstukken naar verwijzen staan in `img/`. Ze zijn gemaakt vóór 8 oktober 2026. Voor deze onderdelen is nog geen schermafdruk:
- de keuzelijst *Uitgave NTA 8800* en de melding "Oudere uitgave — niet voor registratie" onder Resultaten ([hoofdstuk 10](10-normversies.md));
- *Gebouw › Schil & ramen* met de tabel *Belemmering per raam*;
- *Installaties › Koeling* met het formulier van bijlage AA;
- de basisopname met de voortgangslijst en *Overnemen in projectmodel*;
- het blok *Bron & bewijs* in het contextpaneel.
