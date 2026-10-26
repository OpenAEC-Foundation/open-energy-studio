# Gebruikershandleiding NTA 8800 — Open Energy Studio

<!-- handleiding: rekenkern 0.6.0 -->
*Versie: bij rekenkern 0.6.0. De app en de handleiding van een release tonen daarnaast de programmaversie.*

Deze handleiding beschrijft het NTA 8800-deel van Open Energy Studio:
- de rekenkern;
- de invoerformulieren;
- de basisopname;
- het maatwerkadvies;
- herlabelen;
- het projectdossier.

Ze is geschreven als programmadocumentatie voor een softwareattest (BRL 9501). Ze volgt de code van de branch `nta8800-kernel` per 11 oktober 2026, na het UI-herontwerp (werkstappen in plaats van het lint) en met de oudere uitgaven van NTA 8800 in de kern.

De handleiding citeert geen tekst uit NTA 8800, ISSO-publicaties of BRL-documenten. Verwijzingen noemen alleen paragraaf-, formule-, tabel- en paginanummers. Houd de bronnen zelf bij de hand.

## Inhoud

0. [Werken met het programma](00-werken-met-het-programma.md): welkomstscherm, werkstappen, Ctrl+K, contextpaneel, Controle en Ga naar, Basis/Alle velden, de toepasbalk, Gereedschap, thema's, toegankelijkheid en sneltoetsen.
1. [Reikwijdte en status](01-reikwijdte-en-status.md): wat het programma berekent, wat niet, en de attestering.
2. [Basisopname](02-basisopname.md): woningen (ISSO 82.1) en utiliteitsgebouwen (ISSO 75.1), met rekenzones, een geveldeel tegen de grond, en *Overnemen in projectmodel*, ook voor vloeren op de grond.
3. [Projectberekening](03-projectberekening.md): het projectmodel, alle onderdelen van het NTA-invoerformulier en de gemeenschappelijke ruimten (6.6.4).
4. [Uitvoer](04-uitvoer.md): indicatoren, label, Bbl-toets, energie per functie, meldingen en de bijlage met interpretaties.
5. [Validatie en meldingen](05-validatie.md): statussen, gaten en waarschuwingen, invoergrenzen en het vangnet voor niet-eindige uitkomsten.
6. [Maatwerkadvies](06-maatwerkadvies.md): sjablonen, handmatige wijzigingen, pakketten en het adviesrapport.
7. [Herlabelen, registratie en dossier](07-herlabelen-registratie-dossier.md):
   - de BRL 9500-bijlagen 6a/6b, het origineel en de vergelijking;
   - het bewijs;
   - de registratiegegevens en de dossierexport.
8. [Versies en verwijzingen](08-versies-en-verwijzingen.md): kernversie, invoervingerafdruk, het bewaren van oudere builds en de overige documentatie.
9. [Bestanden en uitwisseling](09-bestanden-en-uitwisseling.md): opslaan en openen van `.oes.json`, de voorbeeldprojecten, UNIEC3-export en -import en de exports onder Rapport & dossier › Exports.
10. [Normversies](10-normversies.md): rekenen in NTA 8800:2025+C1, 2024, 2023, 2022 of 2020+A1, invoer per uitgave, het overzicht bij een wissel van uitgave en waarom een oudere uitgave niet registreerbaar is.

## Belangrijk vooraf

- Het programma is **niet geattesteerd** volgens BRL 9501. Uitkomsten hebben de status `calculated_unverified` (of `calculated_legacy_edition` in een oudere uitgave) en zijn geen geregistreerd energielabel.
- Een energielabel ontstaat pas na registratie door een gecertificeerd adviseur (BRL 9500), met een geattesteerd rekenprogramma.
- De voorbeeldprojecten in `training-data/` zijn fictief.

## Schermafdrukken

Alle afbeeldingen waar de hoofdstukken naar verwijzen staan in `img/`. De meeste zijn gemaakt vóór 8 oktober 2026. Op 10 oktober 2026 kwamen erbij: het venster bij een wissel van uitgave (`uitgave-wisselen.png`, hoofdstuk 10), de gemeenschappelijke ruimte van een rekenzone (`gemeenschappelijke-ruimte.png`, hoofdstuk 3), *Vloeren op de grond* bij het overnemen van een opname (`overnemen-vloeren.png`, hoofdstuk 2) en de tabellen *Belemmering*, *Zonwering* en *Glas per raam* (`belemmering-per-raam.png`, `zonwering-per-raam.png`, `glas-per-raam.png`, hoofdstuk 3). Die tonen het onderdeel los, zonder de rest van het venster, met de voorbeeldprojecten of de openbare gevallen D en G uit `training-data/` en, bij de wissel van uitgave, een vastgelegd antwoord van de rekenkern. Later op 10 oktober 2026 kwamen er op dezelfde manier nog negen bij:
- de melding "Oudere uitgave — niet voor registratie" onder Resultaten (`resultaten-oudere-uitgave.png`, hoofdstuk 10), met het vastgelegde antwoord van de rekenkern voor de voorbeeldtussenwoning in de uitgave 2022;
- het formulier van bijlage AA (`bijlage-aa.png`, hoofdstuk 3);
- de basisopname met de voortgangslijst en *Overnemen in projectmodel* (`opname-voortgang.png`) en de uitkomst bovenaan een onderdeel (`opname-uitkomst-onderdeel.png`, beide hoofdstuk 2), met het vastgelegde antwoord van de rekenkern voor de opname uit 1930;
- *ψ uit de leidinggegevens* (`psi-leidinggegevens.png`) en de ketelhulpenergie en WKK-omzettingsgetallen uit een kwaliteitsverklaring (`kwaliteitsverklaring-ketel-wkk.png`, beide hoofdstuk 3);
- *Bron & bewijs* (`bron-en-bewijs.png`) en *Verklaringen voor het energielabel* (`verklaringen-label.png`, beide hoofdstuk 7);
- het venster *Handleiding* (`handleiding-venster.png`, hoofdstuk 0).

Nog later op 10 oktober 2026 kwamen op dezelfde manier de gedeclareerde f_ctrl bij het ventilatiesysteem (`fctrl-gedeclareerd.png`, hoofdstuk 3) en de keuzelijst *Uitgave NTA 8800* in het blok *Algemeen* (`uitgave-keuze.png`, hoofdstuk 10) erbij, beide met het voorbeeldkantoor. Daarmee heeft elk onderdeel een schermafdruk.

Op 11 oktober 2026 is `overnemen-vloeren.png` (hoofdstuk 2) opnieuw gemaakt, omdat het venster nu ook het deel *Koudebrugmethode van het gebouw* toont. Op dezelfde manier kwamen erbij, alle in hoofdstuk 3: de keuze *Lineaire koudebruggen van het gebouw* (`koudebrugmethode.png`), twee rekenzones die één serre delen (`gedeelde-serre.png`), een verwarmde kelder met de diepte per wanddeel (`kelder-wanddelen.png`) en een smalle luchtlaag die zwak geventileerd is in *Constructie bewerken* (`smalle-luchtlaag.png`). Ze gebruiken de voorbeeldtussenwoning; de serre, de wanddelen en de lagen zijn voor de afbeelding ingevuld.

Op 12 oktober 2026 zijn de acht afbeeldingen van het hele venster in hoofdstuk 0 opnieuw gemaakt (`welkom.png`, `projectoverzicht.png`, `verwarming.png`, `palet.png`, `controle.png`, `resultaten.png`, `rapport.png`, `registratie.png`), met de voorbeeldtussenwoning en de uitkomst van de huidige rekenkern. Ze zijn opnieuw te maken met `scripts/manual-screenshots/` (zie de README daar).
