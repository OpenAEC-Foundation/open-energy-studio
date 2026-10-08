# Meldprocedure wijzigingen

**Status:** concept van de softwareleverancier, klaar om vast te stellen. De organisatie vult de namen in de rollentabel in en stelt de procedure vast in het [kwaliteitshandboek](kwaliteitshandboek.md). Tot dat moment geldt zij niet.

**Grondslag:** BRL 9501 §5.2 (p. 9), §6.2 (p. 10) en §6.4 (p. 10). In eigen woorden:
- een wijziging die het rekenresultaat kan beïnvloeden, wordt direct schriftelijk gemeld aan de attesteringsinstelling en de licentiehouders;
- na elke wijziging van de berekeningsmethode of de gebruikersinterface volgt een releasenote, die de instelling en de licentiehouders krijgen.

## 1. Rollen

| Rol | Taak in deze procedure | Naam (in te vullen) |
|---|---|---|
| Releaseverantwoordelijke | Stelt vast of een vrijgave gemeld moet worden, ondertekent de melding | … |
| Interne reviewer | Controleert de releasenote en de melding vóór verzending (vier-ogenprincipe) | … |
| Contactpersoon attesteringsinstelling | Ontvangt en bevestigt meldingen namens de instelling | … (bij de instelling) |

## 2. Wanneer melden

De bron is het releasenotesbestand [docs/nta8800-releasenotes.md](../nta8800-releasenotes.md) met zijn afspraken bovenaan, en het [versiebeleid](../nta8800-versiebeheer.md).

| Situatie | Kenmerk in de repository | Melden aan instelling | Melden aan licentiehouders |
|---|---|---|---|
| **Rekenwijziging** | Een item onder "Onuitgebracht" zonder "(geen rekenwijziging)" in de titel. `KERNEL_VERSION` gaat minstens een MINOR-stap omhoog; de gate (`scripts/nta-kernel-version.mjs check`) dwingt dat af. | Ja, vóór of uiterlijk op de dag van de vrijgave | Ja, met de vrijgave |
| **Alleen gebruikersinterface** | Items met "(geen rekenwijziging)"; `KERNEL_VERSION` gelijk, programmaversie omhoog. | Ja, als releasenote (§6.4) | Ja, als releasenote |
| **Normwijziging** | Nieuwe `TARGET_NORM_VERSION` of een nieuwe uitgave in `norm_versions/`; MAJOR- of MINOR-stap. | Ja, vooraf aangekondigd, met het plan voor het bewaren van de oude versie ([archiefbeleid](archiefbeleid.md)) | Ja, vooraf aangekondigd |
| **Fout gevonden in een uitgebrachte versie** | Klacht of eigen bevinding met oorzaak "rekenfout" ([klachtenprocedure](klachtenprocedure.md)). | Ja, binnen 5 werkdagen na vaststelling, ook als de correctie nog niet klaar is | Ja, met de getroffen versies en een advies |

"Direct" (§5.2) wordt hier uitgelegd als: niet later dan het moment waarop de licentiehouder de gewijzigde versie kan gebruiken. Een wijziging die nog niet is uitgegeven, raakt geen licentiehouder en hoeft nog niet gemeld te worden.

## 3. Werkwijze per vrijgave

1. **Releasenotes afronden.** Alle items voor deze vrijgave staan onder "Onuitgebracht". De interne reviewer leest ze na op volledigheid en begrijpelijkheid voor een adviseur.
2. **Vrijgave maken.** `scripts/release-nta.sh` zet de items onder `## Rekenkern X.Y.Z — <datum>`, draait de gate en de referentiesuites, en schrijft het archief `release/<tag>/` met het leveringsdocument, `manifest.json`, `SHA256SUMS`, het gatelog, het referentierapport en de handleiding.
3. **Melding opstellen** met het sjabloon in §4. Bijlagen:
   - de releasenotesectie van deze versie (als PDF of als tekst);
   - het leveringsdocument;
   - het referentierapport (`reference-report.md`) uit het archief;
   - bij een rekenwijziging: de lijst van referentiegevallen waarvan de uitkomst veranderde, met oud en nieuw.
4. **Verzenden** aan de instelling en aan elke licentiehouder in het [register van licentiehouders](register-licentiehouders.md) met een geldige licentie.
5. **Vastleggen.** Zie §5.

## 4. Sjabloon

> **Onderwerp:** Open Energy Studio [programmaversie] — rekenkern [KERNEL_VERSION] — [rekenwijziging / wijziging gebruikersinterface / normwijziging]
>
> Geachte [naam],
>
> Hierbij melden wij, als houder van BRL 9501-attest [attestnummer], een nieuwe versie van Open Energy Studio.
>
> - Programmaversie: [x.y.z]
> - Rekenkernversie (`KERNEL_VERSION`): [x.y.z], vorige versie: [x.y.z]
> - Norm: [TARGET_NORM_VERSION]
> - Git-tag: [oes-v…-kernel-v…]
> - Datum beschikbaar voor licentiehouders: [datum]
>
> **Verandert de uitkomst van berekeningen?** [Ja/Nee]. [Bij ja: welke onderdelen, welke projecten geraakt worden, en de orde van grootte op de referentiegevallen.]
>
> **Wat moet de licentiehouder doen?** [Bijvoorbeeld: niets / opnieuw rekenen vóór registratie / herlabelen alleen met de oorspronkelijke rekenkern.]
>
> Bijgevoegd: de releasenotes van deze versie, het leveringsdocument en het rapport van de referentieberekeningen.
>
> Met vriendelijke groet,
> [naam releaseverantwoordelijke], [organisatie]

## 5. Vastleggen

Elke verzonden melding komt in het meldarchief, naast de vrijgave:

```
release/<tag>/meldingen/
  YYYY-MM-DD-instelling.pdf        (verzonden bericht, met bijlagen)
  YYYY-MM-DD-licentiehouders.pdf
  ontvangstbevestigingen/          (bevestiging van de instelling)
```

Per melding wordt in het meldregister vastgelegd:

| Datum | Tag | Soort | Ontvangers (instelling / aantal licentiehouders) | Verzonden door | Ontvangst bevestigd (datum) |
|---|---|---|---|---|---|
| … | … | … | … | … | … |

Het archief en het register vallen onder het [archiefbeleid](archiefbeleid.md). De map `release/` staat niet in git; zij hoort op de archieflocatie die daar is vastgelegd.

## 6. Controle

De jaarlijkse review uit het [kwaliteitshandboek](kwaliteitshandboek.md) vergelijkt de lijst van tags met het meldregister. Elke tag met een rekenwijziging moet een melding aan de instelling hebben.
