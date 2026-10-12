# Uitvoereisen BRL 9500-W/U en BRL 9501

**Stand:** 10 oktober 2026, op `nta8800-kernel`.
**Status:** werkdocument, geen attest en geen verklaring van conformiteit.

Dit document zet per eis aan de *uitvoer* van het rekenprogramma op een rij welk document het programma levert en of de eis daarin is ingevuld. Het vult het [attestdossier](nta8800-attestdossier.md) en de [BRL 9501-gereedheid](nta8800-brl9501-gereedheid.md) aan; eisen aan de berekening zelf staan daar.

**Bronnen** (eisen in eigen woorden, geen tekst overgenomen; gedrukte paginanummers):
- [BRL 9501 van 29-05-2026](https://installq.nl/files/BRL-en/brl-9501-installq-2026-aangewezen.pdf), de aangewezen versie.
- BRL 9500-W en BRL 9500-U van 14-10-2025 ("bindend, nog niet vastgesteld"). De aangewezen versies van 29-05-2026 hebben dezelfde tekst en paginering (zie het [attestdossier](nta8800-attestdossier.md)).
- BRL 9500-MWA-W/U, Omgevingsregeling art. 5.11–5.14 en Bbl art. 6.29, zoals uitgewerkt in het attestdossier.

**Uitvoerdocumenten** (afkortingen in de tabel):

| Afkorting | Document | Code |
| --- | --- | --- |
| EPR | Rapportage Energieprestatie, niveaus samenvatting, standaard en gedetailleerd | `src/core/report/EnergyPerformanceReport.ts` |
| RR | NTA 8800-rekenrapport, de "uitdraai van de uitvoerfile" | `src/core/report/NtaCalculationReport.ts` |
| ID | Invoer- en bewijsoverzicht, het "opnameformulier (invoerfile)" | `src/core/report/NtaInputDossier.ts` |
| EPO | EP-Online-gegevensoverzicht, geen registratiebestand | `src/core/report/EpOnlineOverview.ts` |
| PD | Projectdossier: ZIP met project, kernuitvoer, rapport, bewijsregister, checklist en manifest | `src/core/report/ProjectDossier.ts` |
| MWA | Maatwerkadviesrapport | `src/core/report/MaatwerkadviesReport.ts` |
| LD | Leveringsdocument van een vrijgave | `scripts/nta-leveringsdocument.mjs` |

**Statussen:** *voldaan* (aanwezig en getest), *plaatshouder* (het programma toont uitdrukkelijk dat de waarde nog ontbreekt), *extern* (hangt af van RVO, ISSO of de attesteringsinstelling).

## Eisentabel

| # | Eis (eigen woorden) | Bron | Uitvoer | Status |
| --- | --- | --- | --- | --- |
| 1 | Het energieprestatierapport geeft de energiebehoefte (EP1, BENG 1), het primair fossiel energiegebruik (EP2, BENG 2) en het aandeel hernieuwbare energie (EP3, BENG 3). | 9500-W p. 7; 9500-U p. 4; 9501 §2.3, p. 3 | EPR (alle niveaus), RR, EPO | voldaan |
| 2 | Het rapport noemt de (indicatieve) labelletter, gebaseerd op EP2. | 9500-W p. 7; 9500-U p. 4 | EPR, RR, EPO | voldaan; zonder registratie "indicatief" |
| 3 | Voor woningen: de netto warmtevraag, het risico op oververhitting (TO<sub>juli</sub>) en hoe de netto warmtebehoefte zich verhoudt tot de Standaard voor Woningisolatie (NTA 8800 §5.3.2). | 9500-W p. 7; 9501 §2.3, p. 3 | EPR (alle niveaus, **sinds 10 oktober 2026 ook de samenvatting**), RR, EPO (`Warmtebehoefte`, `Temperatuuroverschrijding`) | voldaan |
| 4 | De namen van de opnemende en de registrerende adviseur staan op de uitdraai van de uitvoerfile, in het projectdossier en in de projectenregistratie. | 9500-W §4.2, p. 18; 9500-U §4.2, p. 14 | RR, EPR (**registratieblok sinds 10 oktober 2026 altijd aanwezig**; zonder gegevens noemt het wat ontbreekt), PD | voldaan |
| 5 | Uit de registratie blijken naam en vakbekwaamheidsnummer van beide adviseurs; bij representativiteit dezelfde namen als bij de referentiewoning. | 9500-W §4.2.5, p. 24 | RR, EPR (registratieblok), PD (`project.oes.json`) | voldaan |
| 6 | Het opnameformulier (de invoerfile) bevat de opnamedatum en naam en registratienummer van de opnemende adviseur. | 9500-W bijlage 3, p. 62; 9500-U bijlage 3, p. 54 | ID (**sinds 10 oktober 2026**; zonder gegevens "niet ingevuld"), PD | voldaan |
| 7 | De volledige rapportage (uitvoerfile), de registratiedatum, de adviseurs met nummer en het EP-Online-registratienummer horen in het projectdossier. | 9500-W bijlage 3, p. 62; 9500-U bijlage 3, p. 54 | PD (`rekenrapport.html`, `kernel-output.json`, registratieblok in RR) | voldaan; EP-Online-nummer "nog niet geregistreerd" tot registratie |
| 8 | Het energieprestatierapport wordt geleverd met een uitdraai van de uitvoerfile; op verzoek het volledige projectdossier. | 9500-W §4.2.6, p. 25; 9500-U §4.2.6, p. 20 | EPR + RR; PD | voldaan |
| 9 | Het rekenprogramma waarmee is gerekend, met zijn versie, staat bij de registratie. | Omgevingsregeling art. 5.14 lid 1 onder b | EPR, RR, MWA, ID, EPO (`berekening`), PD (`manifest.software`) — **programmaversie sinds 10 oktober 2026 apart van de rekenkernversie** | voldaan |
| 10 | Het attestnummer van het geattesteerde programma. | 9501 §6.1, p. 10; §8.4 opm., p. 15 | EPR (omslag, uitgangspunten, versiehoofdstuk), RR, MWA, ID, EPO, PD, LD | plaatshouder: "nog niet toegekend" tot het attest |
| 11 | Het deel van het versienummer dat bij de rekenkern hoort (`KERNEL_VERSION`), en de normversie. | 9501 §5.2, p. 9; §4.3, p. 7 | alle documenten; PD-manifest (`kernel`) | voldaan; registratie bij RVO extern |
| 12 | Versiebeleid: differentiatie, opbouw van het versienummer en uitwisselbaarheid tussen versies. | 9501 §4.3, p. 7 | [versiebeheer](nta8800-versiebeheer.md), LD | voldaan (document) |
| 13 | Bij registratie staat vermeld of, en met welk hulpmiddel, gegevens zijn ingelezen; automatisch inlezen mag niet. | 9501 §4.3.1 opm., p. 8 | RR en EPR (registratieblok, "Gegevens ingelezen"), project (`importLog`) | voldaan wanneer registratiegegevens zijn ingevuld |
| 14 | Gegevens van berekening en uitkomst worden geleverd volgens de XML-standaard voor EP-Online; invoer en uitkomst gaan naar EP-Online. | 9501 §4.3.1, p. 8; §5.3 opm. 2, p. 9 | EPO is een controleoverzicht in het openbare exportschema, **geen** registratiebestand | extern (XSD bij RVO) |
| 15 | Een gebruikshandleiding voor de licentiehouder. | 9501 §4.4, p. 8 | `docs/handleiding-nta8800/`, in de app en als HTML bij elke vrijgave | voldaan |
| 16 | Een leveringsdocument met het versienummer zoals in het attest. | 9501 §6.1, p. 10 | LD | voldaan; attestnummer als plaatshouder |
| 17 | Bij financiële kengetallen staan de invoergegevens erbij: tarieven, discontovoet, prijsstijging, looptijd, kostenbronnen. | 9501 §4.1.3, p. 7 | MWA ("Uitgangspunten"; **looptijd van de netto contante waarde, prijsstijging onderhoud en basisjaar sinds 10 oktober 2026**) | voldaan (deelgebied 3 nog niet in de BRL uitgewerkt) |
| 18 | Labelgegevens en verplichte labelelementen a–l, met de verklaringen k en l van de adviseur. | Omgevingsregeling art. 5.13 en 5.13a | EPR, RR, EPO (`labelStatements`) | voldaan; element m vult de uitgever |
| 19 | Geldigheid tien jaar vanaf de opnamedatum; registratietermijnen. | Bbl art. 6.29 lid 4; 9500-W §4.2.5, p. 24 | RR en EPR (registratieblok: geldig tot, uiterste registratiedatum) | voldaan |
| 20 | Het adres en het BAG-object van het gebouw. | Omgevingsregeling art. 5.14 lid 1 onder a | RR, EPR (registratieblok), EPO | voldaan |
| 21 | Het maatwerkadvies laat zien of het een maatwerkadvies of een maatwerkadvies met renovatiepaspoort is. | 9500-MWA-W/U §4.2.8, p. 23 | MWA (`registrationType`) | voldaan |
| 22 | Een reproduceerbaar en bewaard databestand: project, kernuitvoer en hashes. | 9500-W §4.2.7; bijlage 3, p. 62 | PD (manifest met SHA-256 per bestand, invoervingerafdruk) | voldaan |

**Telling:** 22 eisen. Voldaan: 20. Plaatshouder: 1 (rij 10, attestnummer). Extern: 1 (rij 14, XML-registratie bij EP-Online). Bij rij 11 is de uitvoer voldaan; de registratie van de rekenkernversie bij RVO is een stap van de attesthouder.

## Wat op 10 oktober 2026 is aangevuld

- **Programma apart van de rekenkern.** De omslag van de rapportage noemde de rekenkernversie als programmaversie ("Open Energy Studio 0.4.0"). Elke uitvoer noemt nu het rekenprogramma met de programmaversie, de rekenkernversie en het attestnummer apart (`src/core/report/ProgramIdentity.ts`).
- **Attestnummer als plaatshouder.** Zolang er geen attest is staat er "nog niet toegekend" in EPR, RR, MWA, ID, EPO (`berekening.attestnummerTekst`) en het PD-manifest (`software.attestNumberText`), in plaats van dat het veld ontbreekt.
- **Standaard voor Woningisolatie in elke rapportage van een woning**, ook de samenvatting (eis 3).
- **Registratieblok altijd in de rapportage**, ook zonder registratiegegevens; het noemt dan wat er ontbreekt (eis 4).
- **Opnamedatum en opnemend adviseur in het invoer- en bewijsoverzicht** (eis 6).
- **Looptijd van de netto contante waarde** in het maatwerkadviesrapport, met de prijsstijging van onderhoud en het basisjaar (eis 17).
- **Atteststatus in het rekenrapport in het Nederlands** in plaats van `unattested`.

De test `src/__tests__/brl-output-requirements.test.ts` legt deze punten vast.

## Nog extern

- **XML-registratie bij EP-Online** (eis 14): het registratieformaat is niet openbaar en wordt na attestering via RVO verkregen.
- **Attestnummer** (eis 10): komt met het attest; daarna `src/core/nta/attest.json` invullen. Elke uitvoer leest het daar.
- **Uitvoereisen van ISSO 54 versie 5.0** (9501 §4.1.1, p. 7): de publicatie is niet ingezien; zie de [gereedheid](nta8800-brl9501-gereedheid.md).
