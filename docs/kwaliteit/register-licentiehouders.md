# Register van licentiehouders

**Status:** concept van de softwareleverancier, klaar om vast te stellen. Het register zelf staat buiten de software en buiten git (het bevat persoons- en klantgegevens). Dit document beschrijft wat erin staat en hoe het wordt bijgehouden.

**Grondslag:** BRL 9501 §6.2.1 (p. 10): de attesthouder houdt een registratie bij van de licentiehouders van de geattesteerde berekeningsmethode. Het register is ook de verzendlijst voor de [meldprocedure](meldprocedure-wijzigingen.md) (§5.2, §6.4).

## 1. Beheer

| Rol | Taak | Naam (in te vullen) |
|---|---|---|
| Registerbeheerder | Voert nieuwe licenties, leveringen en beëindigingen in | … |
| Releaseverantwoordelijke | Gebruikt het register als verzendlijst bij elke melding | … |

## 2. Wat per licentiehouder wordt vastgelegd

**Licentiehouder** (één regel per organisatie):

| Veld | Toelichting |
|---|---|
| `licentienummer` | Eigen volgnummer, bijvoorbeeld `LH-2026-001` |
| `organisatie` | Naam zoals in de KvK |
| `kvk` | KvK-nummer |
| `contactpersoon`, `e-mail`, `telefoon` | Voor meldingen en releasenotes |
| `certificaathouder_brl9500` | Ja/nee, met het certificaatnummer en het deelgebied (W/U/MWA) indien bekend |
| `licentie_start`, `licentie_einde` | Leeg einde = lopend |
| `aantal_gebruikers` | Indien de licentie dat beperkt |

**Leveringen** (één regel per geleverde versie per licentiehouder):

| Veld | Toelichting |
|---|---|
| `licentienummer` | Verwijzing naar de licentiehouder |
| `datum_levering` | Datum waarop de versie beschikbaar kwam |
| `tag` | Git-tag van de vrijgave, `oes-v<programma>-kernel-v<kern>` |
| `programmaversie`, `kernel_version`, `norm_version` | Zoals in het leveringsdocument |
| `leveringsdocument_sha256` | SHA-256 van `leveringsdocument.md` uit `release/<tag>/` (staat in `SHA256SUMS`) |
| `pakket_sha256` | SHA-256 van het geleverde installatiepakket |
| `melding_verzonden` | Datum van de melding volgens de [meldprocedure](meldprocedure-wijzigingen.md) |

## 3. Sjabloon

```csv
# licentiehouders.csv
licentienummer,organisatie,kvk,contactpersoon,email,telefoon,certificaathouder_brl9500,licentie_start,licentie_einde,aantal_gebruikers
LH-2026-001,…,…,…,…,…,ja (W, nr …),2026-..-..,,…
```

```csv
# leveringen.csv
licentienummer,datum_levering,tag,programmaversie,kernel_version,norm_version,leveringsdocument_sha256,pakket_sha256,melding_verzonden
LH-2026-001,2026-..-..,oes-v…-kernel-v…,…,…,2025+C1,…,…,2026-..-..
```

## 4. Koppeling met het releasearchief

- Elke vrijgave staat in `release/<tag>/` met `leveringsdocument.md`, `manifest.json` en `SHA256SUMS` (gemaakt door `scripts/release-nta.sh`; zie het [archiefbeleid](archiefbeleid.md)).
- De hashes in `leveringen.csv` worden overgenomen uit `SHA256SUMS` van die tag. Daarmee is per licentiehouder aan te tonen welke exacte bestanden hij kreeg.
- Bij een klacht (zie [klachtenprocedure](klachtenprocedure.md)) wordt via het register vastgesteld welke versie de indiener gebruikt en welke andere licentiehouders dezelfde versie hebben.

## 5. Controle

- Bij elke vrijgave: elke lopende licentie krijgt een leveringsregel of een vastgelegde reden waarom niet.
- Jaarlijks in de review uit het [kwaliteitshandboek](kwaliteitshandboek.md): steekproef van leveringsregels tegen `SHA256SUMS` in het archief, en beëindigde licenties nalopen.
- Bewaartermijn van het register: zolang het attest geldt plus 3 jaar.
