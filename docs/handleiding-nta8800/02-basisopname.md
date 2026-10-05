# 2. Basisopname

De basisopname staat in het paneel **Basisopname**, onder Project. Er zijn twee soorten:
- **Woning-opname starten:** voor bestaande woningen, volgens ISSO 82.1, 7e druk met erratum van 6 januari 2026.
- **Utiliteit-opname starten:** voor bestaande utiliteitsgebouwen, volgens ISSO 75.1, 7e druk.

De opname wordt bij het project bewaard (blok `basisopname`). De kern zet de opname om in NTA 8800-invoer en rekent die door met **Opname doorrekenen**. Het resultaat staat los van de projectberekening: het paneel toont het als een aparte, op de opname gebaseerde uitkomst.

**Volgorde van bronnen.** Volgens het wijzigingsdocument v1.1 (p. 6) zitten de wijzigingen daarvan in de 7e druk. Waar de bronnen verschillen, geldt daarom de 7e druk met het erratum. De keuzes per tabel staan in [`docs/nta8800-basisopname.md`](../nta8800-basisopname.md).

## Woningopname (ISSO 82.1)

De opname heeft deze onderdelen:

| Onderdeel | Inhoud | Bron |
|---|---|---|
| Algemeen | bouwjaar, woningtype en ligging (ook appartement met dak en vloer), gebruiksoppervlakte, bouwlagen | ISSO 82.1 hoofdstuk 6 |
| Thermische schil | vlakken met isolatie (aanwezig, dikte, renovatiejaar), ramen en deuren met beglazing, zonwerende beglazing met productgegevens | hoofdstuk 8, tabellen 8.9–8.17 |
| Verwarming | opwekker (ketel, warmtepomp, lokale verwarming, luchtverwarmer, collectief), ontwerpklasse (tabel 9.9), distributie (leidingisolatie, eenpijps), afgifte | hoofdstuk 9 |
| Ventilatie | systeem (A–E), regelingen (tabellen 11.4–11.6), WTW met bypass, ventilatoren, roosterverwarming, passieve koeling | hoofdstuk 11 |
| Warm tapwater | toestel, opslag, douche-WTW | hoofdstuk 13 |
| Ruimtekoeling | opwekker en afgifte | hoofdstuk 10 |
| Zonnestroom (PV) | panelen, oriëntatie, helling | ISSO-hoofdstuk over zonne-energie; NTA hoofdstuk 16 |

**Standaardwaarden.** Een vraag zonder antwoord krijgt de standaardwaarde uit de ISSO-tabel. Elke toegepaste standaardwaarde verschijnt in de lijst "Toegepaste standaardwaarden". Leg per regel de reden vast in het veld **Reden**. Het projectdossier toetst die redenen; zie [hoofdstuk 7](07-herlabelen-registratie-dossier.md).

**Meldingen die vaak voorkomen**
- **Warmtepomp boven 70 °C** (`heat_pump_above_70_requires_declaration`, erratum §4): geef een gecontroleerde verklaring op.
- **Collectieve bron zonder bewijs** (`collective_source_reference_required`): vul de bronvermelding in.
- **Gesloten of verlaagd plafond** in de woningopname (`closed_ceiling_not_in_dwelling_survey`): dit is een ISSO 75.1-gegeven en hoort hier niet.
- **Systeem E zonder warmteterugwinning** (`combined_requires_heat_recovery`, §11.3.6).

## Utiliteitsopname (ISSO 75.1)

**Algemeen**
- Bouwjaar, aantal bouwlagen en gebouwhoogte.
- **Gebruiksfuncties van het gebouw**: functies met oppervlakte en het totaal.
- Fossiele brandstof op het perceel.
- Oppervlakte van sport- en zwemzalen.

De overige onderdelen volgen ISSO 75.1:
- thermische schil;
- verwarming, met BACS-bepaling uit de opgenomen vermogens (tabel 7.3, p. 62–63);
- ventilatie met luchtbehandelingskast;
- koeling;
- warm tapwater;
- verlichting, met een oppervlakte per verlichtingszone;
- bevochtiging;
- PV.

### Rekenzones (§6.5, afb. 6.6 met tabel 6.4)

Kleine functies (tot 25 %) worden samengevoegd met de hoofdfunctie (p. 39–40). Volgens afb. 6.6 en tabel 6.4 (p. 53–54) moet een gebouw soms toch in meer rekenzones worden opgedeeld:
- bij een verschil in setpoint van meer dan 4 K;
- bij een verschil in ventilatiecapaciteit van meer dan een factor 4.

Zonder zones meldt de opname dan `calculation_zone_split_required`.

Zo deel je het gebouw op:
1. Open **Rekenzones (§6.5)** en voeg per zone een id en de gebruiksfuncties met oppervlakte toe. De zones moeten samen de functies van het gebouw dekken.
2. Ken vlakken toe aan een zone met de zonekeuze per vlak. Een vlak zonder zone wordt naar A_g over de zones verdeeld.
3. Geef per verlichtingszone de rekenzone op.
4. Geef per zone eventueel de geïnstalleerde ventilatiecapaciteit op (p. 147). Een zone zonder eigen waarde krijgt naar A_g een deel van het restant.
5. Geef per zone eventueel het oppervlak van de zwemzaal (p. 65) en het decentrale deel van systeem E op (p. 145).

Elke zone moet zelf weer aan afb. 6.6 voldoen (`calculation_zone_criteria_not_met`). Een functie die een eigen zone heeft, wordt ook in de zone van de hoofdfunctie niet samengevoegd. Dat kan in die zone een extra zonesplitsing vragen.

**Beperkingen**
- Eén ventilatiesysteem bedient alle zones.
- Het criterium voor de specifieke interne warmtecapaciteit wordt niet getoetst, omdat de opname één constructie per gebouw kent.

**Meldingen die vaak voorkomen**
- **Ontbrekend bewijs voor inregeling van koeling** (NTA 8800 tabel 10.11, voetnoot a, p. 388): geef een verklaring op, anders telt de inregeling als niet uitgevoerd.
- **Oppervlak zwemzaal zonder zone** (`swimming_pool_zone_required`): bij meerdere sportzones moet de zone worden gekozen.
- **Kleine oude gasmotor** (`gas_engine_small_old_no_table_row`): tabel 9.31 heeft hiervoor geen rij.

## Invoergrenzen

Beide opnames weigeren onmogelijke aantallen en maten:

| Gegeven | Grens | Melding |
|---|---|---|
| bouwlagen | meer dan 200 | `storeys_out_of_range` |
| andere aantallen | meer dan 100.000 | `count_out_of_range` |
| oppervlakten | meer dan 10⁷ m² | `area_out_of_range` |
| gebouwhoogte | ontbreekt, 0 of meer dan 1000 m | `building_height_invalid` |

Dit zijn keuzes van het programma, geen normwaarden.
