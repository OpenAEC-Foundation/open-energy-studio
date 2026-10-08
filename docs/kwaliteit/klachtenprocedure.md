# Klachtenprocedure

**Status:** concept van de softwareleverancier, klaar om vast te stellen. De organisatie vult de namen en termijnen in en stelt de procedure vast in het [kwaliteitshandboek](kwaliteitshandboek.md).

**Grondslag:** BRL 9501 §6.5 (p. 10), taak C3 in de [gereedheidsanalyse](../nta8800-brl9501-gereedheid.md). In eigen woorden vraagt de BRL een vastgelegde procedure die beschrijft:
- hoe klachten worden afgehandeld, en binnen redelijke termijn onderzocht;
- wie verantwoordelijk is;
- hoe klachten worden geregistreerd (datum, aard, oplossing, oorzaak);
- de terugkoppeling aan de indiener, de interne terugkoppeling en de archivering.

## 1. Rollen

| Rol | Taak | Naam (in te vullen) |
|---|---|---|
| Klachtcoördinator | Ontvangt, registreert, bewaakt termijnen, koppelt terug | … |
| Behandelaar | Onderzoekt de klacht inhoudelijk (rekenkern, invoer, interface) | … |
| Releaseverantwoordelijke | Beslist over correctie en vrijgave; meldt aan de instelling | … |
| Interne reviewer | Controleert de analyse bij klachten met oorzaak "rekenfout" | … |

## 2. Wat een klacht is

Elke schriftelijke of mondelinge uiting van ontevredenheid over Open Energy Studio of de levering ervan, van een licentiehouder, adviseur, certificaathouder, de attesteringsinstelling of een derde. Een vraag zonder ontevredenheid is geen klacht, maar wordt wel geregistreerd als die een fout aan het licht brengt.

## 3. Verloop

| Stap | Wat | Termijn (voorstel) |
|---|---|---|
| 1. Ontvangst | Klacht komt binnen via [e-mailadres / formulier]. Mondelinge klachten legt de coördinator schriftelijk vast. | – |
| 2. Registratie | Regel in het klachtenregister (§5), met volgnummer. Ontvangstbevestiging aan de indiener. | 2 werkdagen |
| 3. Reproductie | De behandelaar vraagt zo nodig het projectbestand op. Hij reproduceert met dezelfde programma- en rekenkernversie (zie `KERNEL_VERSION` in het projectdossier; de oude versie staat in het [archief](archiefbeleid.md)). | 5 werkdagen |
| 4. Analyse | Oorzaak vaststellen volgens §4. | 10 werkdagen |
| 5. Besluit en actie | Zie §4. Bij een rekenfout: direct melden aan de attesteringsinstelling volgens de [meldprocedure](meldprocedure-wijzigingen.md). | afhankelijk van de oorzaak |
| 6. Terugkoppeling | Schriftelijk antwoord aan de indiener: oorzaak, oplossing, eventuele versie waarin het is opgelost. Vraag of de indiener tevreden is en leg het antwoord vast. | binnen 20 werkdagen na ontvangst, of een tussenbericht met nieuwe termijn |
| 7. Intern | Bespreken in het periodieke kwaliteitsoverleg; bij herhaling een structurele maatregel. | maandelijks |
| 8. Afsluiten en archiveren | Register bijwerken, stukken in het klachtendossier. | – |

## 4. Oorzaak en actie

| Oorzaak | Kenmerk | Actie |
|---|---|---|
| **Rekenfout** | De kern wijkt af van NTA 8800 (of de geldende uitgave) bij juiste invoer. | Test die de fout aantoont (eenheidstest of referentiegeval). Correctie volgens de wijzigingsprocedure: item in de releasenotes onder "Onuitgebracht", `KERNEL_VERSION` omhoog, gate groen, vrijgave met `scripts/release-nta.sh`. Melding aan de instelling en de licentiehouders, met de getroffen versies. Nagaan of geregistreerde labels geraakt kunnen zijn en dat in de melding vermelden. |
| **Interpretatieverschil** | De norm laat ruimte, of een ander programma leest de norm anders. | Vastleggen in [normversies](../nta8800-normversies.md) of de [vergelijkingsdocumenten](../nta8800-vergelijking-openbare-rapporten.md). Zo nodig een vraag aan NEN ([vragen aan NEN](../nta8800-vragen-nen.md)) of de TC 9501. Geen codewijziging zonder uitsluitsel. |
| **Fout in de gebruikersinterface of het rapport** | Uitkomst klopt, maar weergave, invoer of export niet. | Correctie met test; releasenote met "(geen rekenwijziging)". Melding als releasenote (§6.4). |
| **Gebruikersfout of onduidelijke handleiding** | De invoer was onjuist of onvolledig. | Uitleg aan de indiener. Bij onduidelijkheid: de [handleiding](../handleiding-nta8800/index.md) of een meldtekst verbeteren. |
| **Levering of dienstverlening** | Licentie, installatie, bereikbaarheid. | Afhandelen door de coördinator. |
| **Ongegrond** | Geen afwijking gevonden. | Onderbouwd antwoord aan de indiener. |

## 5. Klachtenregister

Het register is een tabel buiten de software (spreadsheet of CSV). Kolommen:

```csv
nummer,datum_ontvangst,indiener,organisatie,kanaal,programmaversie,kernel_version,norm_version,omschrijving,aard,oorzaak,oplossing,opgelost_in_versie,melding_instelling_datum,terugkoppeling_datum,tevredenheid,behandelaar,datum_afgesloten,dossier
2026-001,2026-10-09,…,…,e-mail,…,0.2.0,2025+C1,…,rekenfout|interpretatie|interface|gebruiker|levering|ongegrond,…,…,…,…,…,ja|nee|deels,…,…,klachten/2026-001/
```

- **aard** is de klacht zoals de indiener haar omschrijft; **oorzaak** is de uitkomst van de analyse (§4).
- Persoonsgegevens alleen voor zover nodig voor de afhandeling; zie de bewaartermijn hieronder.

## 6. Archivering

- Per klacht een map `klachten/<nummer>/` met de klacht, de correspondentie, het projectbestand (met toestemming van de indiener), de analyse en het antwoord.
- Bewaartermijn: minstens zolang het attest geldt plus 3 jaar, en niet korter dan de bewaartermijn van de betrokken versie in het [archiefbeleid](archiefbeleid.md).
- Het register en de dossiers zijn beschikbaar voor de attesteringsinstelling bij de periodieke beoordeling.

## 7. Controle

De jaarlijkse review in het [kwaliteitshandboek](kwaliteitshandboek.md) telt de klachten per oorzaak, controleert de termijnen en gaat na of elke rekenfout een test en een melding heeft.
