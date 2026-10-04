# Gebruikershandleiding NTA 8800 — Open Energy Studio

Deze handleiding beschrijft het NTA 8800-deel van Open Energy Studio:
- de rekenkern;
- de invoerformulieren;
- de basisopname;
- het maatwerkadvies;
- herlabelen;
- het projectdossier.

Ze is geschreven als programmadocumentatie voor een softwareattest (BRL 9501). Ze volgt de code van de branch `nta8800-kernel` per 4 oktober 2026.

De handleiding citeert geen tekst uit NTA 8800, ISSO-publicaties of BRL-documenten. Verwijzingen noemen alleen paragraaf-, formule-, tabel- en paginanummers. Houd de bronnen zelf bij de hand.

## Inhoud

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

## Belangrijk vooraf

- Het programma is **niet geattesteerd** volgens BRL 9501. Uitkomsten hebben de status `calculated_unverified` en zijn geen geregistreerd energielabel.
- Een energielabel ontstaat pas na registratie door een gecertificeerd adviseur (BRL 9500), met een geattesteerd rekenprogramma.
- De voorbeeldprojecten in `training-data/` zijn fictief.
