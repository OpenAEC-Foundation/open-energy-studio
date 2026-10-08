# Archiefbeleid

**Status:** concept van de softwareleverancier, klaar om vast te stellen. De organisatie legt de archieflocatie en de verantwoordelijke vast en stelt het beleid vast in het [kwaliteitshandboek](kwaliteitshandboek.md).

**Grondslag:**
- BRL 9501 §5.3 (p. 9): na een wijziging van de berekeningsmethode door een normwijziging blijft de oude versie minstens 3 jaar beschikbaar, als programma, als versie om mee te rekenen, of doordat de attesthouder op verzoek rekent.
- BRL 9501 §6.2 en §6.3 (p. 10): wijzigingslogboek en testresultaten bij elke wijziging bewaren; na elke vrijgave aantonen dat hoofdstuk 4 voldaan is en dat onderzoek bewaren.
- BRL 9500-W §4.2.4 (p. 24) en 9500-U (p. 20): voor herlabelen blijft de oorspronkelijke rekenkern 24 maanden beschikbaar (zie het [attestdossier](../nta8800-attestdossier.md)).

## 1. Wat wordt bewaard, en hoe lang

| Onderdeel | Bron | Bewaartermijn | Wie |
|---|---|---|---|
| Broncode en volledige geschiedenis | git (`nta8800-kernel`, `main`) met de tags `oes-v…-kernel-v…` | onbeperkt | attesthouder |
| Vrijgavearchief per tag | `release/<tag>/` van `scripts/release-nta.sh`: installatiepakket(ten), `leveringsdocument.md`, `manifest.json`, `SHA256SUMS`, gatelog, referentierapport (`reference-report.json`/`.md`), handleiding als HTML | minstens 3 jaar na de volgende normwijziging (§5.3), en minstens 24 maanden na de laatste levering van die kern (herlabelen) — de langste termijn geldt | attesthouder |
| Releasenotes | [docs/nta8800-releasenotes.md](../nta8800-releasenotes.md) in git, per versie | onbeperkt (git) | attesthouder |
| Meldingen aan instelling en licentiehouders | `release/<tag>/meldingen/` ([meldprocedure](meldprocedure-wijzigingen.md)) | als het vrijgavearchief | attesthouder |
| Testresultaten bij elke wijziging (§6.2) | gatelog en referentierapport per vrijgave; tussentijdse gate-runs zijn reproduceerbaar uit git | als het vrijgavearchief | attesthouder |
| ISSO 54-resultaten per vrijgave (§6.3) | referentierapport zodra de testset in `training-data/reference-suites/` staat | als het vrijgavearchief | attesthouder |
| Register van licentiehouders | [register](register-licentiehouders.md) | attest + 3 jaar | attesthouder |
| Klachtendossiers | [klachtenprocedure](klachtenprocedure.md) | attest + 3 jaar | attesthouder |
| Projectdossiers van berekeningen en labels | dossier-ZIP uit de app (invoer, uitkomst, bewijs, `KERNEL_VERSION`) | 15 jaar | **certificaathouder** (BRL 9500), niet de attesthouder |

## 2. Waar

| Kopie | Locatie (in te vullen) | Toegang |
|---|---|---|
| Primair | … (bijvoorbeeld een beheerde bestandsserver of objectopslag met versiebeheer) | releaseverantwoordelijke, schrijf; overige rollen, lees |
| Secundair | … (fysiek gescheiden, bijvoorbeeld een tweede datacenter of offline medium) | registerbeheerder |

`release/` en `nta-evidence/` staan in `.gitignore`. Na elke vrijgave kopieert de releaseverantwoordelijke `release/<tag>/` naar beide locaties.

## 3. Integriteit

- `SHA256SUMS` in elke vrijgavemap bevat de hash van elk bestand in die map, behalve zichzelf.
- **Na het kopiëren** en **bij de jaarlijkse review** op beide locaties:

  ```sh
  cd <archief>/<tag> && sha256sum -c SHA256SUMS
  ```

  Elke regel moet `OK` geven. Een afwijking wordt hersteld uit de andere kopie en vastgelegd in het kwaliteitsoverleg.
- De hash van het leveringsdocument staat ook in het [register van licentiehouders](register-licentiehouders.md), zodat een geleverde versie onafhankelijk van het archief te controleren is.

## 4. Een oude versie terughalen

Volgorde van voorkeur:

1. **Uit het archief.** Het installatiepakket uit `release/<tag>/`, na `sha256sum -c SHA256SUMS`.
2. **Uit git.** Elke vrijgave heeft een geannoteerde tag (`git tag -l 'oes-v*'`). `scripts/release-nta.sh` maakt die tag lokaal; publiceren naar de centrale repository is een aparte, handmatige stap en hoort bij elke vrijgave te gebeuren. Daarna:

   ```sh
   git checkout <tag>
   CARGO_NET_OFFLINE=true NTA_SKIP_MSRV=1 scripts/verify-nta.sh   # zelfde gate als bij de vrijgave
   npm run tauri:build -- --bundles deb
   ```

   De gate van die tag moet weer groen zijn en het referentierapport hetzelfde oordeel geven als het gearchiveerde rapport. De afhankelijkheden liggen vast in de `Cargo.lock`-bestanden en `package-lock.json`; voor een offline bouw moet ook de toolchain van dat moment beschikbaar zijn (MSRV staat in [versiebeheer](../nta8800-versiebeheer.md)).
3. **Rekenen in de huidige kern met een oudere uitgave.** De kern rekent oudere NTA 8800-uitgaven als `calculated_legacy_edition` ([normversies](../nta8800-normversies.md)). Dat is een hulpmiddel voor controleberekeningen, **geen** vervanging van de geattesteerde oude kern: de uitgavenprofielen zijn nagebouwd en niet dezelfde code.

Voor herlabelen geldt alleen route 1 of 2: BRL 9500 vraagt dezelfde rekenkern, en de app weigert herlabelen met een andere `KERNEL_VERSION` (`relabel_kernel_version_differs`).

## 5. Controle

De jaarlijkse review in het [kwaliteitshandboek](kwaliteitshandboek.md):
- `sha256sum -c` op beide kopieën van alle vrijgaven binnen de bewaartermijn;
- één oude tag volgens §4 route 2 herbouwen en het referentierapport vergelijken;
- vrijgaven waarvan de termijn verstreken is, pas verwijderen na een vastgelegd besluit.
