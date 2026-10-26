# Productregisters (BCRG) en de registermomentopname

Issue #15. Stand 26 oktober 2026, rekenkern 0.6.0.

## Toegang tot het register

| Onderwerp | Stand |
|---|---|
| Register | BCRG energieprestatiedatabank (kwaliteitsverklaringen van producten) |
| Machinetoegang | BCRG noemt een API voor rekensoftware ("Koppeling software (API)"); een API-sleutel wordt via het secretariaat aangevraagd en per aanvraag beoordeeld |
| Contracteigenaar | BCRG; voor Open Energy Studio is nog geen sleutel aangevraagd |
| Eindpunten, schema, dekking, versies | onbekend: niet verkregen |
| Authenticatie, rechten, limieten | onbekend: niet verkregen |
| Foutgevallen (geweigerde sleutel, ingetrokken of gewijzigde verklaring) | alleen als begrip bekend; BCRG kan vermeldingen aanpassen of verwijderen (algemene voorwaarden 2026, art. 5.6 en 5.8). Exacte codes volgen pas uit het contract |
| Herdistributie van registergegevens door Open Energy Studio | niet geregeld |

Open Energy Studio raadt geen eindpunten en leest geen webpagina's van het register uit. Zolang er geen contract is, is de enige route **handmatig**: de adviseur neemt de waarden over en bevestigt ze.

## De registermomentopname

Een bewijsstuk van het soort *kwaliteitsverklaring* of *prestatieverklaring* kan een `registerSnapshot` dragen (`crates/nta8800-core/src/register_snapshot.rs`, `src/core/nta/registerSnapshot.ts`). Het bewijsstuk zelf (de pdf van de verklaring) heeft al zijn eigen SHA-256 en koppelingen naar invoer; de momentopname legt daarnaast vast:

| Veld | Inhoud |
|---|---|
| `register`, `route` | bijvoorbeeld `BCRG`; `manual` |
| `declarationNumber`, `revision` | verklaringnummer en revisie |
| `productName`, `manufacturer` | product en fabrikant |
| `scope` | normuitgave en toepassingsvoorwaarden zoals op de verklaring |
| `capturedOn` | datum van overnemen |
| `values[]` | grootheid, waarde (tekst zoals op de verklaring), eenheid en het invoerveld (JSON-pointer) dat de waarde vult |
| `valuesSha256` | SHA-256 over de waarden: per waarde `grootheid TAB waarde TAB eenheid TAB pad LF` |
| `confirmedBy`, `confirmedOn` | de adviseur die waarden en toepassing bevestigde, en wanneer |
| `registryCheck` | status in het register: `not_checked` (standaard), `current`, `changed`, `withdrawn`, met datum en opmerking |

**Twee aparte handelingen.** De momentopname is het historische verslag van wat de berekening gebruikte; heropenen van een project leest haar zoals ze is opgeslagen, zonder register. Controleren of de verklaring nog actueel is, is een aparte handeling (`registryCheck`) die de waarden niet verandert. Een nieuwe revisie of een intrekking wijzigt een bestaand project dus nooit stilzwijgend; de adviseur kiest zelf hoe een nieuwe berekening wordt gemaakt.

## Controles bij registratie

| Code | Ernst | Wanneer |
|---|---|---|
| `register_snapshot_unconfirmed` | blokkeert | geen bevestiging, of een wijziging na de bevestiging |
| `register_snapshot_identity_missing` | blokkeert | register, verklaringnummer of product leeg |
| `register_snapshot_values_missing`, `register_snapshot_value_missing` | blokkeert | geen waarden, of een lege waarde of grootheid |
| `register_snapshot_path_unknown` | blokkeert | het invoerveld bestaat niet |
| `register_snapshot_value_differs` | blokkeert | de invoer wijkt af van de overgenomen waarde |
| `register_snapshot_hash_mismatch` | blokkeert | de waarden zijn buiten de app gewijzigd |
| `register_snapshot_product_mismatch` | blokkeert | het product in de invoer (registratienummer bij de warmtepomp) is een ander dan het verklaringnummer |
| `register_declaration_withdrawn` | blokkeert | de adviseur stelde vast dat de verklaring is ingetrokken |
| `register_snapshot_kind`, `register_snapshot_date_invalid`, `register_check_date_invalid` | blokkeert | verkeerd soort bewijsstuk, of een ongeldige datum |
| `register_declaration_changed` | waarschuwing | de verklaring is gewijzigd in het register |
| `register_status_not_checked` | waarschuwing | de status in het register is niet gecontroleerd; dat is "niet gecontroleerd", nooit "geldig" |

Tests: `crates/nta8800-core/src/register_snapshot.rs` (ingetrokken, gewijzigd, niet gecontroleerd, ander product, ontbrekende waarde, afwijkende invoer, gewijzigde waarden zonder nieuwe hash, onbevestigd, offline heropenen) en `src/__tests__/register-snapshot.test.tsx` (hash gelijk aan die van de kern, vervallen bevestiging, dossier).

## Grenzen

- Een momentopname bewijst welke waarden uit welke verklaring zijn overgenomen. Zij bewijst niet dat de verklaring echt of geldig is; ook een berekende interpolatie uit een producttabel is dat niet. Elke uitkomst blijft `calculated_unverified`.
- De hash van het bewijsstuk bewijst dat het bestand niet is veranderd, niet dat het register het nog zo vermeldt.

## Wat er nodig is voor een koppeling

1. Een API-sleutel en de contractvoorwaarden van BCRG (aanvraag via het secretariaat; dat doet de organisatie, niet de app).
2. Daarna: zoeken op verklaringnummer, de waarden en de revisie ophalen als nieuwe momentopname met route `api`, de status in het register laten controleren, en bij geweigerde sleutel of storing "niet gecontroleerd" tonen in plaats van "actueel".
