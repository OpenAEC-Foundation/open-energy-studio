# Doeluitgave 2025+C1:2026 — wijzigingsimpact

**Broncontrole:** 29 september 2026; implementatiestand bijgewerkt op 15 oktober 2026. De [Staatscourant 2026, nr. 18113](https://zoek.officielebekendmakingen.nl/stcrt-2026-18113.html) wijst NTA 8800:2025+C1:2026 aan en noemt BRL 9501 van 14 oktober 2025. De toelichting benoemt onderstaande onderwerpen. Zij bevat niet de volledige rekenregels, invoerdefinities of officiële EDR-uitkomsten. De gelicentieerde doeluitgave en de [aangewezen BRL 9501](nta8800-brl9501-gereedheid.md) zijn beschikbaar; de Rust-routes zijn tegen de doeluitgave nagelopen. Actuele officiële ISSO-54-uitkomsten, de toepasselijke volledige testset en de formele externe toets ontbreken nog. Dit overzicht volgt het [dekkingsregister](nta8800-dekkingsregister.md) en is geen conformiteitsverklaring.

| Onderwerp uit de regeling | Huidige project-/Rust-dekking | Benodigd voor een toetsbare route |
| --- | --- | --- |
| Elektriciteits- en warmteopslag | `storage`-invoer voor batterij of thermische opslag met capaciteit en de gebouwgebonden route van 5.14a in de Rust-kern; nog onverifieerd | Toepassingsgrenzen, systeemgrens en anti-dubbeltelling met actuele EDR-deeluitkomsten toetsen |
| Gebouwautomatiserings- en controlesystemen (GACS) | Rust-route voor `f_BACS` volgens §5.5.8, ook uit opgenomen systeemvermogens; afzonderlijke invoeraudit en grensgevallen | Wettelijke plicht en bewijs van functies/klassen apart vaststellen; actuele utiliteitsreferenties vergelijken |
| Nieuwe energielabelindicatoren | Rust-keten levert BENG, indicatieve labelklasse, jaarlijkse finale energie en operationele CO₂; HTTP, MCP en desktop gebruiken dezelfde kern | Volledige uitvoer en eenheden tegen actuele officiële W/U-gevallen en de registratie-eisen toetsen |
| Zeer lage temperatuur bronnetten | `district_water`-classificatie, gedeeltelijke collectieve warmtepompbronroute en bijlage P; geen volledig netmodel | Nettemperaturen, warmte-/koudelevering, hulpenergie, eigendoms-/systeemgrens en onafhankelijke combinatiegevallen |
| Platte-dak-PV, dakrandbeschaduwing, permanente zonwering, riet/biobased geleidbaarheid, interne warmtecapaciteit, bijlage AA | Rust-routes voor PV, beschaduwing, constructies, capaciteit en bijlage AA zijn tegen de gelicentieerde doeluitgave nagelopen; de rekenrouten blijven onverifieerd | Actuele onafhankelijke gevallen per gewijzigde tak en grenswaarde vergelijken |
| EDR en attestering per release | Reproduceerbare technische gate, referentieharnas en bewaarde debugbuilds; interne herberekeningen, maar geen actuele officiële ISSO-54-set of attest | Volledige actuele W/U-testset en verwachte waarden, verplichte deel- en uitvoerposten, reproduceerbare run per release, resultaten naar certificerende instelling en besluit voor exacte release |

De regeling beschrijft globaal een positieve waardering voor bepaalde opslag en een gevolg voor gebouwen die niet aan een toepasselijke GACS-verplichting voldoen. De [BACS-diagnose](nta8800-bacs-conceptdiagnose.md) ontstond uit het openbare concept; de latere kernroute is tegen §5.5.8 van de gelicentieerde doeluitgave nagelopen. Wettelijke toepasselijkheid en werkelijk aanwezige functies zijn daarmee nog niet bewezen. Ook de historische ISSO-54-testen uit 2022 zijn geen actuele goldens voor 2025+C1:2026.

## Beslisvolgorde voor implementatie

1. Behoud de rechtmatig verkregen geconsolideerde NTA buiten de repository en de aangewezen BRL 9501 als eisenkader; verkrijg de actuele ISSO 54 met EDR-invoer en verwachte resultaten. Registreer editie, rechten en hash van elk bronbestand.
2. Maak per rij een expliciet Rust-invoercontract en een normprofiel met paragraaf-/tabelverwijzingen. Voeg geen standaardwaarde toe zonder normbron.
3. Sluit de open W- en U-varianten en invoergrenzen in de Rust-keten; vergelijk deelposten vóór eindindicatoren met onafhankelijke actuele gevallen.
4. Maak de actuele EDR-set blokkerend voor elke release. Archiveer invoerhash, uitvoerhash, normprofiel, kernelversie, tolerantiebron en reviewer.
5. Dien pas na volledige interne dekking de exacte softwareversie bij de attesteringsinstelling in; een ontwikkelbuild blijft `unattested`.

De [verificatiestatus](nta8800-verificatiestatus.md) en het [dekkingsregister](nta8800-dekkingsregister.md) geven de actuele codegrens weer.
