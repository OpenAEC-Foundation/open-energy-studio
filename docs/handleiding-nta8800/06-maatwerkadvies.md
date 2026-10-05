# 6. Maatwerkadvies

Het paneel **Maatwerkadvies** staat op het tabblad Resultaten. Het werkt volgens BRL 9500-MWA-W/U van 24-03-2026, in werking per 29-05-2026 (§3.1, p. 13; §3.2 renovatiepaspoort, p. 14; §4.2.5, p. 21; §4.2.8 registratie, p. 23), en ISSO 82.2 (woningen) of ISSO 75.2 (utiliteit), beide 3e druk. De volledige technische beschrijving staat in [`docs/nta8800-maatwerkadvies.md`](../nta8800-maatwerkadvies.md).

## Werkwijze

1. Kies **Maatwerkadvies starten**.
2. Leg het **Gebruikersprofiel** vast (ISSO 82.2/75.2 §2.5, tabel 2.2). Kies een profiel of vul de velden vrij in: setpoints, bewoners, branduren, tapwater en praktijkfactoren voor ventilatie.
3. Leg het **Gemeten verbruik (fitprocedure)** vast, per jaar of per maand, voor de fitcontrole (ISSO 82.2 hoofdstuk 3, bijlage C.1). Leg ook de **Tarieven en rekenrente** vast, met een bron.
4. Voeg **maatregelen** toe (zie hieronder) en stel **pakketten** samen. Volgens §4.2.2 zijn er minimaal twee (ISSO 82.2 p. 52; ISSO 75.2 p. 64).
5. Kies **Doorrekenen**.

Het resultaat toont per variant (huidig, elke maatregel, elk pakket):
- de besparing per drager;
- CO₂;
- energiekosten;
- investering;
- terugverdientijd;
- netto contante waarde;
- het indicatieve label met BENG 1–3.

De resultaten zijn te sorteren op terugverdientijd of op netto contante waarde. ISSO 82.2 §6.2.4 (p. 82) schrijft geen rangorde voor.

## Maatregelen met een sjabloon

Kies bij **Soort maatregel** een sjabloon:

| Sjabloon | Wat je kiest |
|---|---|
| Isolatie dak, gevel of vloer | de vlakken en de nieuwe Rc of U |
| Beglazing | de ramen en de nieuwe U (en g) |
| Kierdichting | de streefwaarde q_v10 met bron; na uitvoering meten |
| Ventilatiesysteem | het nieuwe systeem (tabel 11.5) |
| Warmtepomp of andere opwekker | de opwekker; bij een forfaitaire warmtepomp de tabelrij |
| Tapwatertoestel | het toestel (§13.8) |
| PV | de systemen; de belemmering start conservatief (situatie e, tabel 17.3) |
| Zonneboiler | de systemen (§13.7) |
| Douche-WTW | de units |
| Verlichting (utiliteit) | de gewijzigde verlichtingszones |

Hoe het sjabloon werkt:
- **Patch:** het sjabloon maakt de wijziging zelf (een JSON-patch op het project) en toont die als voorbeeld. Elke berekening, rapportexport en dossierexport bouwt de patch opnieuw tegen het actuele project.
- **Blokkades:** een sjabloon met openstaande problemen blokkeert de maatregel (`measure_template_incomplete`), met de reden op de regel. Voorbeelden:
  - een ontbrekend piekvermogen;
  - vlakken die niet meer bestaan;
  - een ontbrekende aanvoertemperatuur.
- **Waarschuwingen:**
  - een isolatiemaatregel die niet beter is dan de huidige Rc;
  - een investering van 0 €.
- **Startwaarden:** levensduur en categorie zijn bewerkbare suggesties. ISSO geeft geen levensduurtabel.

## Renovatiepaspoort

Een renovatiepaspoort is een maatwerkadvies met drie gestapelde stappen. Stap 2 bevat de maatregelen van stap 1, stap 3 die van stap 1 en 2. Vink **Renovatiepaspoort opstellen** aan en kies per stap een pakket. Welke eisen gelden, hangt af van de gebouwfunctie van het project.

**Woningen (BRL 9500-MWA-W §3.2, p. 14):**
- **Stap 1, isolatie:** minstens de Standaard voor Woningisolatie. Vink dit aan, of laat het open: de kern vergelijkt dan zelf de warmtebehoefte van stap 1 met de standaard (NTA 8800 §5.3.2).
- **Vooroorlogse standaard:** kies deze als gevelisolatie technisch niet kan, met een motivatie. Een hybride warmtepomp is dan toegestaan.
- **Oververhitting:** vink de maatregelen in stap 1 aan die het risico beperken.
- **Stap 2:** een aardgasvrije hoofdverwarming waar dat realistisch mogelijk is. Kan dat niet, vul dan de motivatie in.
- **Stap 3:** voorstellen voor overige hernieuwbare energie. Vink ook aan of opslag is afgewogen (ISSO 82.2 §4.4.2).

**Utiliteit (BRL 9500-MWA-U §3.2, p. 14):**
- **Stap 1, isolatie:** het isolatieniveau is voldoende voor lagetemperatuurverwarming en hogetemperatuurkoeling (vinkje).
- **Gevelisolatie:** kan die technisch niet, vul dan de motivatie in.
- **Koelvraag:** vink de maatregelen in stap 1 aan die de koelvraag beperken.
- **EP2:** de EP2 van stap 3 moet op of onder de renovatiestandaard liggen (NTA 8800 tabel 5.7, zie ISSO 75.2). De kern toetst dit.
- **Stap 3:** voorstellen voor overige hernieuwbare energie.

Het resultaat toont per eis of die is gehaald. De regel **Registratie** zegt of het advies wordt geregistreerd als *Maatwerkadvies* of als *Maatwerkadvies met renovatiepaspoort* (§4.2.8). Dat laatste kan alleen als het paspoort aan alle eisen voldoet. Het paspoortdocument zelf maakt EP-Online bij de registratie.

## Handmatige maatregelen

Kies **Handmatig (patchregels, gevorderd)** om de wijziging zelf op te geven:
- **Regels:** elke regel heeft een bewerking (`replace`, `add`, `remove`), een pad (een JSON-pointer die met `/` begint) en een waarde.
- **Type:** het type van de waarde (getal, tekst, ja/nee, leeg of JSON) wordt bewaard.
- **Doel:** de wijziging werkt op het project, of op de afgeleide gebouwinvoer (`target: building`).

Handmatige paden gebruiken posities in lijsten. Een verwijdering in het projectmodel kan zo'n pad verschuiven:
- **Doel project:** het programma weigert de verwijdering en noemt de maatregelen die het betreft.
- **Doel gebouwinvoer:** de verwijderbevestiging noemt de maatregelen; controleer hun paden daarna.

Sjabloonmaatregelen gebruiken id's en hebben daar geen last van.

## Adviesrapport

Kies in het paneel **Adviesrapport (HTML)**. Het rapport is in het Nederlands, met:
- categorieën;
- bedragen gegroepeerd per duizendtal;
- de reden bij een ongeldige variant;
- de onderbouwing van een gunstige PV-belemmering.
