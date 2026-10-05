# 7. Herlabelen, registratie en dossier

## Registratiegegevens

De registratiegegevens staan in **Projectgegevens** → *Registratie (BRL 9500)*. Ze worden bewaard in het projectblok `registration`. Alle velden zijn optioneel: de kern meldt wat nog ontbreekt. Het gaat om:
- **Gebouw:**
  - doel (toets Bbl, oplevering, bestaande bouw);
  - opnametype en representativiteit;
  - adres en BAG-id;
  - bouwjaar en gebouwtype.
- **Partijen en data:**
  - opdrachtgever en certificaatnummer;
  - opnemend en registrerend adviseur, elk met vakbekwaamheidsnummer;
  - opname- en registratiedatum.
- **Berichttype:** `regular`, `relabel` (herlabelen) of `replacement` (vervanging van een onjuist label binnen 24 maanden, BRL 9500-W p. 24–25).
- **WLC-GWP:** voor nieuwe gebouwen boven 1000 m² bij een toets Bbl vanaf 1-1-2028 en de oplevering daarna (BRL 9500-W p. 18, 21, 62).
- **Vorige labelklasse:** alleen voor de plausibiliteitscontrole.

Bij opslaan schrijft de app ook de identiteit van het rekenprogramma: naam, versie, kernversie en attestnummer (Omgevingsregeling art. 5.14 lid 1 onder b). Bij een herlabeling is dat anders: dan blijft de opgeslagen identiteit van de oorspronkelijke berekening staan en schrijft de app niet de huidige (BRL 9500-W §4.2.4, p. 24).

**Controles van de kern**
- **Termijn:** registratie binnen drie maanden na de opname, of zes bij seriematige projecten (BRL §4.2.5).
- **Detailopname:** bij toets Bbl en oplevering is een detailopname verplicht (BRL 9500-W §3.1).
- **BAG-id:** het BAG-id heeft 16 cijfers. Een woning vraagt een verblijfsobject, ligplaats of standplaats. Utiliteit mag ook een pand-id gebruiken (Omgevingsregeling art. 5.14 lid 1 onder a; Praktijkhandboek p. 46).
- **Eén label per adres:** de app houdt lokaal bij welke labels er in deze installatie geregistreerd zijn. Ze waarschuwt als hetzelfde adres al een geldig woninglabel heeft. Labels die buiten deze installatie zijn geregistreerd, kan ze niet zien.

**Status**
- `dossierComplete` geeft aan of het dossier compleet is.
- `readyForRegistration` is alleen waar als het dossier compleet is **en** het programma geattesteerd is (Omgevingsregeling art. 5.11 en 5.12 lid 2. Zolang er geen attestnummer is, blijft dit onwaar.

## Bewijsregister

**Projectgegevens** → *Bewijsregister (BRL 9500 bijlage 3)*. Leg per bestand vast:
- soort, bestandsnaam en SHA-256;
- datum, optioneel GPS;
- aanleverende partij en controlerende adviseur;
- de JSON-paden die het bestand onderbouwt.

Verwijs vanuit een bronveld naar een bewijsstuk met `evidence:<id>`. Bij herlabelen heeft een bewijsstuk een **rol bij herlabelen**:
- offerte met opdracht;
- gespecificeerde factuur van de verbetering op dit adres;
- foto van PV of zonthermie met beschaduwing.

## Herlabelen (BRL 9500 bijlagen 6a/6b)

Herlabelen kan alleen bij bestaande bouw. Het geldt voor verbeteringen die binnen 24 maanden na de oorspronkelijke opnamedatum zijn aangebracht. Er wordt gerekend met dezelfde rekenkern en dezelfde opnamedatum (BRL 9500-W §4.2.3–4.2.4, p. 23–24; 9500-U p. 18–20).

**Werkwijze**
1. Bewaar na de oorspronkelijke registratie het projectbestand, met het EP-Online-nummer in de registratiegegevens. Dat bestand wordt het **origineel**.
2. Breng de verbeteringen aan in het project. Bewerk bestaande elementen en voeg geen nieuwe toe als het om een vervanging gaat.
3. Zet het berichttype op *herlabelen*. Vul in:
   - de verbeteringsdatum;
   - het oorspronkelijke certificaatnummer en EP-Online-nummer;
   - de **Kernelversie oorspronkelijke opname (herlabelen)**; zonder dit veld meldt de kern `original_kernel_version_required`;
   - eventueel de verwijzing naar het oorspronkelijke dossier;
   - bij gewijzigde PV of zonthermie het vinkje **PV of zonthermie exclusief en fysiek verbonden met dit gebouw (herlabelen)** (BRL 9500-W p. 23);
   - bij utiliteit het vinkje **Utiliteit: met opdrachtgever en stukken vastgesteld dat er geen 6b-wijzigingen zijn (herlabelen)** (BRL 9500-U p. 19).
4. Kies in het paneel **Herlabelen (BRL 9500 bijlage 6a/6b)**, op het tabblad Resultaten, het oorspronkelijke projectbestand.

**De vergelijking.** Het paneel deelt elke wijziging in:
- toegestaan (6a);
- niet toegestaan (6b);
- te beoordelen door de adviseur.

De indeling volgt W bijlage 6a/6b (p. 67–68) en U bijlage 6a/6b (p. 58–60):

| Wijziging | Woningen (W) | Utiliteit (U) |
|---|---|---|
| Geometrie, thermische grens, gebruiksoppervlakte | 6b | 6b |
| Ander systeemtype | 6b | 6b |
| Isolatie en installaties, één op één | 6a | 6a |
| Distributie, afgifte, regeling | volgt 6a per functie | 6b |
| PV of zonthermie: grootte, helling, oriëntatie | beoordelen | beoordelen |
| g-waarde van beglazing | 6a | 6a |
| Verlichting (staat in geen van de bijlagen) | beoordelen | beoordelen |

**Bewaren.** De vergelijking en het originele bestand worden bij de registratie bewaard (`registration.relabelComparison`). Na een latere projectwijziging staat er een melding dat de vergelijking verouderd is; vergelijk dan opnieuw.

**Controle bij registratie.** De kern vertrouwt de bewaarde uitkomst niet:
- **Echtheid van het origineel:** ze toetst het origineel aan zijn SHA-256 en aan het EP-Online-nummer, de certificaathouder en de opnamedatum in de registratie.
- **Opnieuw vergelijken:** ze vergelijkt opnieuw en gebruikt die uitkomst.
- **Actualiteit:** ze blokkeert als de huidige invoer sinds de vergelijking is gewijzigd (`relabel_comparison_outdated`).
- **Overige eisen:**
  - dezelfde certificaathouder;
  - een bewijsstuk met de rol offerte of factuur;
  - bij PV of zonthermie een foto en een bevestiging van de fysieke aansluiting;
  - bij utiliteit een bevestiging dat er geen 6b-wijzigingen zijn (U p. 19).

**Oude projecten.** Herlabelprojecten van vóór deze regels worden bij het openen eenmalig omgezet. Facturen zonder rol worden "te beoordelen" en tellen nog niet als bewijs. Een melding noemt de velden die nog ontbreken.

**Privacy.** Het projectbestand bevat het volledige origineel, met adres en opdrachtgever. Deel het alleen binnen het dossier van de certificaathouder.

## Projectdossier

Kies **Rapport** → **Projectdossier exporteren (ZIP)**. De ZIP bevat:

| Bestand | Inhoud |
|---|---|
| `project.oes.json` | het project, zonder het opgeslagen origineel |
| `kernel-output.json` | de volledige kernuitvoer |
| `ep-online-gegevensoverzicht.json` | de gegevens zoals EP-Online ze publiceert (veldnamen van het openbare exportschema `EpbdExportTypesV4`), om het geregistreerde label achteraf te controleren; **geen registratiebestand** |
| `rekenrapport.html` | het NTA-rekenrapport |
| `basisopname-output.json` | de opname-uitkomst, bij een basisopname |
| `herlabel-vergelijking.json` | de vergelijking, met de uitkomst van de nieuwe kernvergelijking (`kernelRecheck`), bij herlabelen |
| `herlabel-origineel.oes.json` | het origineel, byte voor byte, bij herlabelen |
| bewijsbestanden | uit het bewijsregister |
| `dossier-checklist.json` | de volledigheidscontrole (BRL 9500 bijlage 3, p. 61–63) |
| `manifest.json` | moment, projectnaam, kernversie met normversie en invoervingerafdruk, attestatus, per bestand pad, SHA-256 en grootte, het aantal ontbrekende bewijsstukken (`missingEvidence`) en de checklist |

**Checklist.** De checklist hangt af van het doel, het opnametype, de representativiteit en herlabelen. Bij een basisopname toetst ze de redenen voor de toegepaste standaardwaarden. Het tabblad Rapport toont dezelfde checklist als de export. Terwijl de berekening loopt, staan punten op "bezig"; in een export komt die status nooit voor.

**EP-Online.** De app kan een label niet zelf registreren. Het uploadformaat van EP-Online is niet openbaar en is alleen voor geattesteerde rekenprogramma's via RVO verkrijgbaar. Het gegevensoverzicht in de ZIP gebruikt de veldnamen van het openbare exportschema, zodat u na registratie kunt nagaan of EP-Online dezelfde waarden toont als de berekening. `missingRequired` noemt de velden die voor een registratie nog ontbreken, zoals de registratiedatum of het certificaathoudernummer.

**Bewaren.** Bewaar het dossier en het databestand vijftien jaar (Praktijkhandboek p. 47). Dat is een taak van de certificaathouder.
