# Doeluitgave 2025+C1:2026 — wijzigingsimpact

**Broncontrole:** 29 september 2026. De [Staatscourant 2026, nr. 18113](https://zoek.officielebekendmakingen.nl/stcrt-2026-18113.html) wijst NTA 8800:2025+C1:2026 aan en noemt BRL 9501 van 14 oktober 2025. De toelichting benoemt onderstaande onderwerpen. Zij bevat niet de volledige rekenregels, invoerdefinities of officiële EDR-uitkomsten; daarvoor blijven de aangewezen NTA, BRL en ISSO-publicatie 54 nodig.

| Onderwerp uit de regeling | Huidige project-/Rust-dekking | Benodigd voor een toetsbare route |
| --- | --- | --- |
| Elektriciteits- en warmteopslag | Geen specifiek NTA-opslagmodel of Rust-rekenroute | Normdefinities, toepassingsgrenzen, vermogen/capaciteit, plaats in energiebalans, anti-dubbeltelling en onafhankelijke EDR-deeluitkomsten |
| Gebouwautomatiserings- en controlesystemen (GACS) | Afzonderlijke voorlopige Rust-diagnose voor `fBACS` uit het openbare hoofdstuk-5-concept; geen wettelijke verplichtingstoets of geverifieerde waarderingsroute | Definitieve gebouwtoepasselijkheid, bewijs van aanwezige functies en klassen, exacte NTA/BRL-invoer en utiliteitscases |
| Nieuwe energielabelindicatoren | Geen Rust-BENG- of labeluitvoer; bestaande TypeScript-uitvoer is indicatief | Jaarlijks finaal energiegebruik en operationele broeikasgasemissies per voorgeschreven bron en eenheid, plus alle overige verplichte indicatoren en officiële uitvoertoets |
| Zeer lage temperatuur bronnetten | `district_water` als warmtepompbron en vrije systeemkoppeling; geen netgegevens of bronbalans | Nettemperaturen, warmte-/koudelevering, hulpenergie, eigendoms-/systeemgrens en onafhankelijke combinatiegevallen |
| Platte-dak-PV, dakrandbeschaduwing, permanente zonwering, riet/biobased geleidbaarheid, interne warmtecapaciteit, bijlage AA | Bestaande UI-velden deels aanwezig, maar geen op de doeluitgave getoetste Rust-routes | Per wijziging exact NTA-artikel/tabel, datamodel, afronding en referentiegeval |
| EDR en attestering per release | Diagnostische structuur- en rekentests; geen actuele ISSO-54-set of attest | Volledige actuele W/U-testset en verwachte waarden, verplichte deel- en uitvoerposten, reproduceerbare run per release, resultaten naar certificerende instelling en besluit voor exacte release |

De regeling beschrijft globaal een positieve waardering voor bepaalde opslag en een gevolg voor gebouwen die niet aan een toepasselijke GACS-verplichting voldoen. De afzonderlijke [BACS-factor](nta8800-bacs-conceptdiagnose.md) 1,0/1,05 komt uitsluitend uit het **openbare concept** van §5.5.8 en is als voorlopige diagnose gemarkeerd; de definitieve normtekst, voorwaarden, invoer en rekenvolgorde zijn nog niet vastgesteld. Opslagpercentages zijn niet als codeconstanten overgenomen. Ook de historische ISSO-54-testen uit 2022 zijn geen actuele goldens voor 2025+C1:2026.

## Beslisvolgorde voor implementatie

1. Verkrijg rechtmatig de geconsolideerde NTA, de aangewezen BRL 9501 en actuele ISSO 54 met EDR-invoer en verwachte resultaten. Registreer editie, rechten en hash van elk bronbestand.
2. Maak per rij een expliciet Rust-invoercontract en een normprofiel met paragraaf-/tabelverwijzingen. Voeg geen standaardwaarde toe zonder normbron.
3. Bouw de volledige W- en U-rekenketen, inclusief warmtepompvarianten, opslag en GACS, en vergelijk deelposten vóór eindindicatoren.
4. Maak de actuele EDR-set blokkerend voor elke release. Archiveer invoerhash, uitvoerhash, normprofiel, kernelversie, tolerantiebron en reviewer.
5. Dien pas na volledige interne dekking de exacte softwareversie bij de attesteringsinstelling in; een ontwikkelbuild blijft `unattested`.

De [verificatiestatus](nta8800-verificatiestatus.md) en het [dekkingsregister](nta8800-dekkingsregister.md) geven de actuele codegrens weer.
