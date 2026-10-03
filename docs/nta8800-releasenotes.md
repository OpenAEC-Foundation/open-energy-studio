# NTA 8800-kern — releasenotes

Wijzigingen die de uitkomst of de status van bestaande, opgeslagen projecten veranderen. Normverwijzingen gaan naar NTA 8800:2025+C1:2026, met paragraaf-, formule- en paginanummers.

## 3 oktober 2026 — hoofdstuk 12 en bijlage P na de herberekening

### Uitkomsten die veranderen

- **Koudenet met ontvochtiging als jaarwaarde (P.77/P.78, p. 1022).** Het maandprofiel blijft nu bestaan; ontvochtiging telt alleen in de jaarwaarde. Pompenergie per maand (P.68) is daardoor mogelijk, en de standby van P.69 loopt alleen in de maanden met koudelevering.
- **Warmtenet met een deel zonder maandwaarden (P.73/P.74, p. 1020).** Alleen dat deel volgt P.74; de opgegeven maandwaarden van de andere percelen blijven. Dit verandert vooral de looptijden van P.60.
- **f_Pren;dc (5.49, p. 129)** wordt niet meer naar beneden afgerond.
- **Een tweede bevochtiger in dezelfde rekenzone** geeft nu het gat `humidifier_zone_duplicate` (§12.1, p. 520).

### Nieuw

- `sourcePumpIncluded` bij een opgegeven warmtepomprendement in bijlage P: de bronpomp of -ventilator zit in het rendement, dus 0 W/kW in plaats van 10 W/kW (P.6.8.4.3, p. 1009).
- `servedAreaM2` bij een stoombevochtiger: de bediende oppervlakte voor de 500 m²-grens van 12.2.1 (p. 521). Zonder waarde blijft de oppervlakte aan het verwarmingssysteem gelden.
- Waarschuwingen `chp_co2_factor_negative` (P.27) en `cold_network_gain_outside_cooling_months` (P.13–P.18 voor koude).
- Een primaire factor 0 heet in de uitvoer nu `0` in plaats van `-0`.
## 3 oktober 2026 — review van de registratie, lege waarden en bijlage P-formulieren

### Status van opgeslagen projecten

- **Gereed voor registratie.** `readyForRegistration` is nu waar als het dossier compleet is én het rekenprogramma een BRL 9501-attest heeft (Regeling art. 2 en 3, p. 4–5). Zolang `SOFTWARE_ATTEST_NUMBER` leeg is, is het dus overal onwaar. Het nieuwe veld `dossierComplete` beoordeelt alleen het dossier; het rapport toont beide.
- **Herlabelen.** Alleen een herlabeling houdt het opgeslagen rekenprogramma (BRL 9500-W §4.2.4, p. 24). Het programma legt nu ook de kernversie vast (`software.kernelVersion`). Wijkt die af van de rekenkern die nu rekent, dan meldt de registratie `relabel_software_kernel_differs`. Een vervanging is een nieuwe berekening en krijgt het huidige programma (p. 23).
- **WLC-GWP.** Een oplevering vóór 2028 zonder datum van de toets Bbl geeft weer "niet vereist" in plaats van "onbekend". Een vrijstaande woning met één woning in de berekening is het hele gebouw: haar eigen A_g beslist, zonder waarschuwing.
- **Bijlage P.** Een maandreeks van de netwatertemperatuur zonder enige maand in bedrijf geeft `network_temperature_required` (P.14). Een berekende opslag (P.35) zonder vat, laadleiding of warmtewisselaar geeft `storage_components_required`; zonder onderdelen kwam η op 1, gunstiger dan het forfait. Het formulier begint de berekende opslag nu met één vat.
- **Maatwerkadvies.** Een wijziging met een leeg pad of een pad zonder `/` wordt geweigerd met `measure_patch_failed` op die regel. Voorheen verving een leeg pad het hele project.

### Lege waarden

`nta_value_missing` vindt nu ook twee of meer lege verplichte velden in één type (bijvoorbeeld lengte en gronddekking van één leidingsegment). Toegestane lege maanden in reeksen als `temperaturesC` worden niet meer gemeld.

### Formulieren

- Elke wijziging in een maatregel heeft nu een soort waarde (getal, tekst, ja/nee, leeg of JSON), zodat tekst "2" tekst blijft.
- Het wissen van "overig verlies" verwijdert het veld (de kern neemt dan 0). Een gebieds-PV-systeem begint zonder belemmeringsfactor.
- De dode JSON-controle bij het opslaan van het NTA-formulier is verwijderd.

## 3 oktober 2026 — formulieren zonder JSON-editors (bijlage P, hoofdstuk 14, randdelen, maatwerkadvies)

Rekenuitkomsten en opgeslagen projecten veranderen niet; dezelfde invoer is nu met gewone velden te maken:

- **Bijlage P:** de laatste JSON-vakken in het formulier voor externe levering zijn vervangen door velden:
  - leidingdelen (P.13–P.18) met lagen, ligging, omgeving, correctie van tabel P.1, watertemperatuur van het net en buffervaten (P.43/P.44);
  - de berekende tapwateropslag (P.35 met P.43–P.46): vaten, laadleidingen en externe wisselaar;
  - de opwekkers collectieve zonnewarmte (opgegeven of volgens 13.7.2.2, P.33), elektrische flexmodus (5.8) en sorptiekoeling (tabel P.10);
  - PV in het gebied (P.7/P.71) met de velden van hoofdstuk 16.

  Bij het wisselen van soort, ligging of bron worden de velden van de vorige keuze gewist.
- **Verlichting (hoofdstuk 14):** meerdere armaturengroepen (14.8 systeemvermogen of 14.9 lampvermogen met tabel 14.2), f_dyn, geïnstalleerd parasitair vermogen (14.10–14.12), daglichtsectoren (verticale ramen, daklichten en hellende ramen van bijlage Y) en extra verlichtingszones. Alleen verlichting per rekenzone bij meerdere rekenzones loopt nog via Geavanceerd (JSON).
- **Vloerranden (8.36):** de ψ-waarden per randdeel hebben eigen velden.
- **PV:** het wisselen van de route voor P_pk wist de velden van de vorige route; voorheen bleef bijvoorbeeld `panelCount` staan naast `panelAreaM2`, wat de kern als onbekend veld weigerde.
- **Maatwerkadvies:** de wijzigingen van een maatregel (RFC 6902) staan per regel met bewerking, pad en waarde in plaats van in één JSON-tekstvak.

De volledige JSON-weergave van het NTA-blok en van de basisopname blijft bestaan als geavanceerde invoer.
## 3 oktober 2026 — reviewcorrecties registratie en combi-tapwater

### Uitkomsten die veranderen

- **Combi-aandeel per systeem (13.184/13.185, p. 653; 9.1, p. 288).** Het tapwateraandeel van een combi- of afleverset wordt nu berekend met de opwekkeroutput van het hoofdsysteem, het enige systeem met de tapwaterbelasting, en alleen op de dragers van dat systeem toegepast. Een extra verwarmingssysteem (bijvoorbeeld een gasketel naast een warmtepomp-combi) blijft volledig E_H × f_BACS. Projecten met meerdere verwarmingssystemen, een combi en f_BACS ≠ 1 krijgen daardoor een ander gebruik en EPTot. Projecten met één verwarmingssysteem veranderen niet.

### Invoer

- **Lege waarden in getagde enums.** Een lege waarde (null) binnen bijvoorbeeld een opwekker of een PV-piekvermogen gaf nog `nta_calculation_block_invalid`. De route zoekt de lege velden nu in de JSON zelf en meldt `nta_value_missing` op het exacte pad. Een optioneel veld dat leeg mag zijn wordt niet gemeld.

### Registratie

- **Attest apart van de dossiercontrole.** `software_attest_number_missing` en `software_required` zijn geen registratiepunten meer. De beoordeling geeft `software` (het opgeslagen programma, of dit programma met de kernversie voor projecten zonder) en `softwareAttested`. `readyForRegistration` hangt alleen nog van het dossier af; het rapport noemt een ontbrekend attest naast "gereed".
- **Herlabelen en vervangen** houden bij opslaan het programma van de oorspronkelijke berekening (BRL 9500-W §4.2.4, p. 23–24); een gewone registratie krijgt het huidige programma.
- **Dossierchecklist.** De punten van Bijlage 6a/6b verschijnen ook bij `messageType: relabel` zonder het oude veld `relabel`.
- **BAG-id.** Een pand-id (type 10) is toegestaan voor andere gebouwen dan woningen (Regeling art. 5 lid 1 onder a, p. 6). Voor een woninglabel blijft het adresseerbare object verplicht (Praktijkhandboek p. 46).
- **WLC-GWP bij oplevering.** De plicht volgt de toets Bbl (BRL 9500-W p. 21, 62). Een oplevering gebruikt de nieuwe datum `bblCheckDate`. Zonder die datum is de plicht onbeslist en volgt een plausibiliteitsmelding (`wlc_gwp_bbl_check_date_unknown`) in plaats van een blokkerende melding. De grens van 1000 m² geldt per gebouw: `buildingUsableFloorAreaM2`; bij één woning onder de grens zonder gebouw-A_g volgt `wlc_gwp_building_area_unknown`.
- **Vorige labelklasse.** Hoofdletters doen er niet toe ("a+" is A+); een onbekende klasse geeft een plausibiliteitsmelding in plaats van een fout.
- **Lokaal BAG-register.** Alleen een geregistreerd label (met EP-Online-nummer) komt in het register. Het leegmaken van het BAG-id of het nummer verwijdert de regel. Een label waarvan de geldigheid (opnamedatum + 10 jaar) vóór de nieuwe opname of registratie verliep, telt niet als conflict.
- **Bronverwijzing.** "Regeling art. 5 lid b" is nu "art. 5 lid 1 onder b".

## 3 oktober 2026 — registratie: rekenprogramma, berichttypen, WLC-GWP, BAG en plausibiliteit

Rekenuitkomsten veranderen niet. De registratiecontrole wordt strenger:

- **Rekenprogramma (Regeling art. 5 lid 1 onder b, p. 6):** de app schrijft `registration.software` bij het opslaan van de projectgegevens. Zolang het programma niet volgens BRL 9501 is geattesteerd, meldt de kern `software_attest_number_missing` (ontbreekt); `readyForRegistration` blijft dan `false`. Opgeslagen projecten zonder dit blok krijgen `software_required` tot de projectgegevens opnieuw worden opgeslagen.
- **BAG-id:** een id dat niet uit 16 cijfers bestaat of geen verblijfsobject, ligplaats of standplaats is, geeft nu een fout (`bag_object_id_invalid`, `bag_object_id_not_addressable`; Praktijkhandboek v2 p. 46).
- **Berichttype (BRL 9500-W p. 24–25):** nieuw veld `messageType` met `regular`, `relabel` en `replacement`. Het oude `relabel: true` blijft werken. Vervangen vraagt het EP-Online-nummer van het vervangen label en moet binnen 24 maanden na de oorspronkelijke opname.
- **WLC-GWP (BRL 9500-W p. 18, 21, 62):** vanaf 1-1-2028 is bij toets Bbl en oplevering van een gebouw > 1000 m² de uitkomst met rapportverwijzing verplicht.
- **Plausibiliteit (BRL 9500-W p. 42):** nieuwe, nooit blokkerende lijst `registration.plausibility`, ook in het rapport.
- **A<sub>g</sub> (Praktijkhandboek v2 p. 70):** labelgegevens tonen A<sub>g</sub> op twee decimalen; meer decimalen in de invoer geeft een plausibiliteitsmelding.
- **Bronnen:** de laatste verwijzingen naar de internetconsultatie (afgifte H9, hulpenergie warmtepompen, gaswarmtepompen, opwekkerverdeling en de collectieve bron) zijn vervangen door paginanummers van NTA 8800:2025+C1:2026.
## 3 oktober 2026 — reviewcorrecties energie per energiefunctie en invoerformulieren

### Uitkomsten die veranderen

- **Tapwater uit een combitoestel of afleverset op het verwarmingstoestel (13.184/13.185, p. 653; 5.20a, p. 89).** Het tapwateraandeel van het toestel is E_W en telt nu onder warm tapwater, zonder f_BACS. Alleen E_H krijgt f_BACS. Bij een utiliteitsgebouw met f_BACS = 1,05 en zo'n toestel dalen het gebruik, EPtot en BENG 2 met 0,05 × het tapwateraandeel. Woningen (f_BACS = 1) houden dezelfde totalen; in de verdeling per energiefunctie verschuift het aandeel van verwarming naar warm tapwater. Het hulpenergiegebruik van het toestel blijft bij verwarming (p. 653).

### Opgeslagen projecten

- **Leeg getal in een lijst.** Een lege maandwaarde of een leeg pompvermogen gaf `nta_calculation_block_invalid` voor het hele NTA-blok. Nu is dat het gat `nta_value_missing` op het pad van die waarde, bijvoorbeeld `ntaCalculation.declaredUses[0].monthlyKwh[3]`.
- **Gecombineerde buitenlucht en afvoerlucht uitgevinkt.** Het formulier wist nu de verborgen `outdoorAirHeatFraction` en `outdoorAirFractionReference`. Die gaven anders het blokkerende `outdoor_air_fraction_without_combined_source`. Een opgeslagen project met zulke restwaarden houdt dat gat tot het vakje opnieuw wordt aan- en uitgevinkt.

### Formulieren en uitvoer

- Een ander koudeopwekkertype vervangt alleen de eerste opwekker; verdere opwekkers blijven staan. De keuzelijst kent nu ook de compressiekoelmachine met gasmotor (tabel 10.29) en de absorptiekoelmachine op een WKK (tabel 10.30).
- Waarschuwingen over opgegeven interne warmte wijzen nu naar de zone, bijvoorbeeld `spaceHeating.demand.internalGains.heatFluxWPerM2`. Een zone waarvan alle gebruiksfuncties woonfuncties zijn, krijgt `internal_gains_declared_residential`.
- De bijlage met interpretaties bevat nu ook hoofdstuk 5 (`building_performance::INTERPRETATIONS`).

## 3 oktober 2026 — energie per energiefunctie, rapportaanvullingen en opgegeven interne warmte

Uitkomsten veranderen niet; er komen uitvoervelden en waarschuwingen bij.

- **Energie per energiefunctie (§5.5.3, 5.20, p. 89–90):** Nieuwe uitvoer `energyByService`, met per energiefunctie (verwarming, warm tapwater, koeling, bevochtiging, ventilatie, verlichting, hulpenergie volgens 5.21, collectieve warmtepompbron), drager en maand:
  - het gebruik E_EPus;ci;
  - de afgenomen energie;
  - de primaire fossiele energie;
  - de hernieuwbare energie (5.39).

  Export, opslagcorrectie en hernieuwbare elektriciteit staan apart als gebouwtermen. De som per drager is gelijk aan `carriers`; de som met de gebouwtermen is gelijk aan EPtot en EPrenTot (getoetst voor beide voorbeeldprojecten).
- **Nieuwe uitvoer:** `cooling` (hoofdstuk 10, per koelsysteem in `systems`) en `pvSystems` (hoofdstuk 16, E_pr;el per systeem).
- **Paneel en rapport:** een tabel per energiefunctie. Het rapport toont daarnaast warm tapwater (behoefte, verliezen, opwekker), koeling per systeem, PV per systeem, de ZEB-indicator (bijlage AB) en een bijlage met alle interpretaties van de kern.
- **Waarschuwingen bij opgegeven interne warmte (§7.5.3.1/7.5.3.2, p. 179–180):**
  - `internal_gains_declared_below_table`: lager dan q_Oc·f_τ + q_A van tabel 7.2/7.3;
  - `internal_gains_declared_differs_from_table`: gelijk aan noch de tabelwaarde, noch de tabelwaarde met q_L van §5.4.2;
  - `internal_gains_declared_residential`: een woonfunctie, waar 7.21 de formule voorschrijft.
## 3 oktober 2026 — formulieren voor externe levering, serres, bevochtiging, koelsystemen en opgegeven stromen

Alleen de invoer verandert; opgeslagen projecten rekenen hetzelfde.

- **Externe levering (§5.8, bijlage P):** het JSON-veld is vervangen door een formulier per drager (dh, dw, dc):
  - forfaitair (geen route), kwaliteitsverklaring, gemeten stromen (P.6) of berekend systeem (P.7, P.9);
  - bij het berekende systeem: levering of percelen (P.72–P.83), distributie (jaarstromen, tabel P.0 of klein koudesysteem), opwekkers met hun soort, vermogen, fractie en voorkeur, hulpenergie (P.56–P.70) en de tapwateropslag (P.34/P.35);
  - collectieve warmtepompbron en elektriciteit in het gebied (P.7, P.71).

  Leidingdelen, berekende opslag en de opwekkers collectieve zonnewarmte, elektrische flex en sorptiekoeling houden een JSON-veld binnen het formulier.
- **Aangrenzende onverwarmde serres (7.30b)** met beglazing, b_U, H_zi;ztu en vlakken.
- **Bevochtiging (hoofdstuk 12)** per rekenzone: verstuivend of stoom, met warmtewiel.
- **Meerdere koelsystemen (§10.2):** een project met meer rekenzones kan per koelsysteem de bediende zones kiezen; dat sluit één koelsysteem voor het hele gebouw uit.
- **Zonneverwarming zonder tapwatersysteem (§13.7).**
- **Collectieve installatie (9.6.1)** en het bewijs voor het hernieuwbare aandeel van een warmtepomp (5.31/5.32).
- **Opgegeven stromen per maand:** gebruik buiten de berekening (§5.5), hernieuwbare tapwaterwarmte (5.35/5.36) en opwekking op eigen perceel (hoofdstuk 16).
- **Bronvermelding in de uitvoer:** de velden `draftSource` en `consultationSource` van de tabellen 9.25, 9.27 en 9.29, §5.3.1, §5.5.8, 5.20–5.21 en §5.9 noemen nu de paragraaf en pagina's van de eindtekst in plaats van de internetconsultatie.

## 3 oktober 2026 — invoerformulieren voor toestellen met productgegevens en de grondvloer

Alleen de invoer verandert; opgeslagen projecten rekenen hetzelfde.

- **Warmtepomp met productgegevens (bijlage Q):**
  - bron;
  - maximaal vermogen bij condities 1–4;
  - aan/uit of modulerend met de deellastreeksen van tabel Q.15;
  - uitschakelcriteria van tabellen Q.1/Q.3;
  - bronpomp (Q.4.4);
  - verdamperintrede (Q.2.14.2);
  - bijverwarming (elektrisch of forfaitaire gasketel);
  - regeneratie bij een bodembron (bijlage V).
- **Ketel met productgegevens (bijlage M), lokale, lucht- en stralingsverwarmers (bijlage N) en overige verwarmers volgens tabel 9.25:** nu als formulier in plaats van alleen via de API.
- **Grondvloer (§8.3, bijlage D):**
  - kruipruimte of onverwarmde kelder eronder (8.3.4.2);
  - verwarmde ruimte onder maaiveld (8.3.3.2);
  - randisolatie (tabel D.1).
- **BCRG-verklaringstabel:** invoer met de BCRG-code. De kern interpoleert η, F en W_aux binnen de tabel. De uitkomst is informatief en wordt niet in het project opgeslagen.
## 3 oktober 2026 — TOjuli, bijlage AA en de jaargemiddelde buitentemperatuur

### Uitkomsten die veranderen

- **TOjuli bij onvoldoende koelcapaciteit (bijlage AA).** Was: geen TOjuli en geen Bbl-toets. Nu: TOjuli volgens 5.40, zoals zonder actieve koeling (§5.7.1, p. 114–115), met de waarschuwing `annex_aa_capacity_insufficient`.
- **`θ_e;avg;an`.** Overal 10,67 °C (D.4, p. 791) in plaats van het ongewogen gemiddelde 10,6717 °C in 7.14/7.15/7.73. Verschillen in de orde van 0,01 kWh/m².

### Projecten die nu een gat krijgen

- **Zonwering als bewijs voor actieve koeling** (`criterion: shaded_glazing`). De raamgegevens moeten de verklaring dragen: meer dan 95 % van het beoordeelde glas met lamellen van tabel 7.4a/7.4b, `g_gl ≤ 0,4` of `F_sh;obst;juli < 0,67`. Anders `solar_limitation_not_met`; bij een onvolledige raaminventaris `window_inventory_incomplete`.
- **Afgewezen bewijs voor actieve koeling** staat nu ook als gat in `gaps` (pad `ntaCalculation.activeCooling…`). Eerder viel TOjuli stil weg.

### Overig

- Een raam in bijlage AA mag met het projectraam-id worden opgegeven (`win-S`), naast de afgeleide naam `window:win-S`.
- Nieuw uitvoerveld `weightedConductanceWPerK` per ventilatiemaand: H_ve volgens 7.19/7.20 met b_v. `conductanceWPerK` blijft ρ·c·Σq zonder b_v.

## 3 oktober 2026 — R_se en R_si bij grenzen met een onverwarmde ruimte (8.4.2.1)

Volgens 8.4.2.1 (p. 266) wordt bij een grens met een onverwarmde ruimte R_se vervangen door de R_si van tabel C.2 (p. 778). Een projectconstructie draagt R_se = 0,04 (C.10, p. 777). Deze release regelt de gevallen waarin dat niet zo is.

### Projecten die nu `invalid` worden

- **Horizontaal `internal` vlak naar een onverwarmde ruimte.** De warmtestroomrichting en daarmee R_si (0,10 omhoog, 0,17 omlaag) is niet bekend. Dit geeft nu `unheated_surface_direction_required`; kies het type vloer of dak. Een niet-horizontaal `internal` vlak telt als wand (R_si 0,13, tabel C.2 opmerking 3).
- **Ongeldige `exteriorSurfaceResistance`.** Een negatieve waarde, of een waarde van ten minste 1/U, geeft `construction_exterior_resistance_invalid`.

### Uitkomsten die veranderen

- **Constructie zonder buitenlucht** (tabel C.2 opmerking 1). Wordt de kernberekening van een constructie toegepast terwijl "Buitenvlak in contact met lucht" uit staat, dan bewaart het project `exteriorSurfaceResistance: 0`. Naar een onverwarmde ruimte trekt de kern dan geen 0,04 meer af, maar telt hij alleen R_si op. Hetzelfde geldt voor de stilstaande-luchtwaarde achter een sterk geventileerde spouw (C.3.3). Bestaande constructies zonder dit veld blijven op 0,04.
- **VABI-import.** Een geïmporteerde constructie krijgt de R_si van haar elementtype: wand 0,13, dak 0,10, vloer 0,17. Dat was altijd 0,17. Een naam die op een wand en op een dak voorkomt, geeft nu twee constructies.
- **Eenvoudige U-berekening voor `internal`.** Die gebruikt nu R_se 0,04 in plaats van een tweede R_si van 0,13. Zo gaat de 8.4.2.1-correctie in de kern niet dubbel.

### Overig

- **Het label van het vinkje is aangepast.** "Grenst aan buitenlucht" heet nu "Buitenvlak in contact met lucht". Het hoort ook aan te staan bij een grens met een onverwarmde ruimte.
- **`towardsUnheatedSpace` in het forfaitaire envelopmodel geldt alleen voor de basisopname.** Een U-waarde uit die route hoort niet in een projectconstructie thuis.
## 3 oktober 2026 — reviewcorrecties rekenzones in de utiliteitsopname

### Opnames die nu `incomplete` worden

- **Zwembad met meerdere sportzones.** Hebben meerdere rekenzones sport en is de zwembadruimte niet per zone opgegeven, dan volgt `swimming_pool_zone_required` (p. 65). Eerder koos de opname de zone met de meeste sport.
- **Nieuwe controles op de waarden per zone:** `zone_installed_capacity_invalid`, `zone_installed_capacity_exceeds_total`, `zone_swimming_pool_area_invalid`, `zone_swimming_pool_areas_mismatch`, `zone_combined_without_system` en `zone_combined_areas_mismatch`.

### Opnames met een andere uitkomst

- **Eén oppervlaktebron.** Bij meerdere zones komen de functiegroepen, het tapwater, `labelFunctions`, BACS en de totale A_g uit de zonesommen. Een afwijking binnen 0,05 m² gaf eerder `derived_input_rejected`; nu rekent de opname door.
- **Geïnstalleerde capaciteit per zone (p. 147).** Met `zones[i].installedCapacityDm3PerS` krijgt die zone haar eigen capaciteit; alleen de rest wordt naar A_g verdeeld.
- **Systeem E per zone (p. 145).** Met `zones[i].combined` krijgt alleen die zone een decentraal deel, met haar eigen oppervlakken.

Opnames zonder `zones` geven dezelfde invoer als voorheen.

## 3 oktober 2026 — hoofdstuk 8 na de herberekening

### Projecten die nu `incomplete` worden

- **Kruipruimte of onverwarmde kelder met randisolatie.** Randisolatie (D.7/D.8, tabel D.1) hoort alleen bij een vloer op de grond: `ground_floor_edge_insulation_slab_only` op `ntaCalculation.groundFloors[i]`. Dit gaf eerder alleen de status `invalid`.
- **Kruipruimte én verwarmde kelder onder dezelfde vloer:** `ground_floor_below_and_heated_basement`. Ook dit gaf eerder alleen `invalid`.

### Projecten met een andere uitkomst

- **Vloer boven een kruipruimte of onverwarmde kelder.** U_f rekent nu met R_se = 0,04 (8.43 → 8.2.2.2.1 → tabel C.2) in plaats van 0,17. H_g stijgt iets; Q_H;nd ongeveer +0,2 %.
- **Transmissie naar een onverwarmde ruimte.** H_D;zi,j;ztu gebruikt nu R_si aan de kant van de onverwarmde ruimte in plaats van R_se (8.4.2.1). U wordt omgerekend met 0,13 (wand), 0,10 (dak of plafond) of 0,17 (vloer). H_D;iu daalt iets; b_U en H_U veranderen mee.
- **Samengestelde constructie met een sterk geventileerde spouw.** Werd geweigerd en wordt nu berekend (C.3.3 in C.5 en C.6).
- **Voorbeeld tussenwoning:** het dak is nu 2 × 35,36 m² in plaats van 52 m². BENG 2 80,56 kWh/m²·jr, aandeel hernieuwbaar 22,3 %, label A+.

### Nieuwe waarschuwingen (niet-blokkerend)

- `ground_floor_resistance_below_surface_resistance`: R_si + R_c van de vloer onder 0,17.
- `ground_floor_perimeter_implausible`: P groter dan 2·A/1 m + 2 m.
- `detailed_thermal_bridges_none_entered`: gedetailleerde methode zonder ψ-waarden naar buitenlucht.
- `sunroom_values_differ_from_unheated_space`: b_U of H_zi;ztu van een serre wijkt meer dan 10 % af van de onverwarmde ruimte met dezelfde id.
## 3 oktober 2026 — meerdere rekenzones in de utiliteitsopname (ISSO 75.1 §6.5)

### Nieuwe invoer

- **`zones`** in de utiliteitsopname: twee of meer rekenzones met hun gebruiksfuncties. Een opname die met `calculation_zone_split_required` stopte, kan nu worden afgemaakt. Vlakken (`envelope.surfaces[].zoneId`) en verlichtingszones (`lighting[].zoneId`) krijgen hun zone; vlakken zonder zone worden naar A_g verdeeld.
- In de woningopname is `zoneId` op een vlak niet toegestaan: `surface_zone_not_in_dwelling_survey`.

### Opnames die nu `incomplete` worden

Alleen opnames met `zones`:

- `zone_function_areas_mismatch`, `zone_function_area_invalid`, `zone_id_required`, `zone_id_duplicate`;
- `surface_zone_unknown`, `lighting_zone_calculation_zone_required`, `lighting_zone_calculation_zone_unknown`, `zone_lighting_area_mismatch`;
- `calculation_zone_criteria_not_met`: een zone die volgens afb. 6.6 zelf nog gesplitst moet worden.

### Opnames met een andere uitkomst

Geen: zonder `zones` is de afleiding ongewijzigd (gecontroleerd op de drie utiliteitsfixtures: invoer, standaardwaarden en uitkomst identiek).

## 3 oktober 2026 — micro-WKK, PV en zonneboilers na de herberekening

### Projecten die nu `invalid` worden

- **Micro-WKK met een totaalrendement boven 1,2.** Tabel 9.33 begrenst η_th + η_el per meetpunt op [0; 1,2]: `micro_chp_total_efficiency_invalid`.
- **Micro-WKK met tegenstrijdige elektrische meetwaarden.** Zijn P_el en η_el allebei opgegeven, dan moeten ze op 2 % na passen bij P_th·η_el/η_th: `micro_chp_electric_values_inconsistent`.

### Projecten met een andere uitkomst

- **Micro-WKK boven vollast (9.66/16.15).** Warmte boven P_th;chp_100+sup_100·t levert geen eigen elektriciteit meer op. De invoer van dat overschot blijft geboekt als bijstookwarmte. Lagere opgewekte elektriciteit betekent een hogere primaire fossiele energie.
- **PV met een belemmering, precies tussen twee oriëntaties (17.3.7).** Bij een azimut van 22,5°, 67,5° enzovoort geldt nu per maand de hoogste belemmeringsfactor van de twee buren. Eerder gold altijd de rechtsom liggende buur.

### Nieuwe waarschuwingen (de berekening loopt door)

- `micro_chp_capacity_exceeded`: de micro-WKK levert in een maand meer warmte dan zijn vollastvermogen (verwarming of tapwater).
- `solar_tested_backup_distribution_below_one`: geteste zonneboiler met geïntegreerde naverwarming en Σ f_dis < 1; 13.134 verhoogt dan de zonne-opbrengst.
- `pvt_without_thermal_part`, `pvt_without_electric_part` en `pvt_cover_inconsistent`: het elektrische en het thermische deel van een PVT-systeem passen niet bij elkaar.
## 3 oktober 2026 — reviewcorrecties utiliteitsopname (ISSO 75.1) en waterzijdig inregelen van koeling

### Opnames met een andere uitkomst

- **Waterzijdig inregelen van koeling zonder verklaring (woning- en utiliteitsopname).** NTA-tabel 10.11 voetnoot a (p. 388) vraagt een verklaring volgens NEN-EN 14336. Zonder `cooling.balancingEvidenceReference` rekent statisch of dynamisch ingeregeld nu als niet ingeregeld: f_HB 1,15 in plaats van 1,00, en geen waterzijdige inregeling bij de afgifte. Opgeslagen opnames met "statisch" of "dynamisch" hebben deze verwijzing nog niet en krijgen dus een hoger pompenergiegebruik.
- **f_BACS bij meerdere verwarmingsopwekkers.** Het systeemvermogen van tabel 7.3 (p. 63) is nu de som van de hoofdopwekker en de extra opwekkers. Daarvoor telde alleen de installatiecapaciteit. Een ketel van 200 kW met een warmtepomp van 150 kW geeft nu f_BACS 1,05 in plaats van 1,0 als BACS onbekend is.
- **Directe expansie in de LBK.** Er zijn geen afgiftetoestellen in de ruimte meer: de afgifte is `other_or_unknown` zonder ventilatorconvectoren. Opgenomen split-binnendelen gaven ventilatorenergie voor toestellen die er niet zijn.
- **Zwembadruimte in 13.32a.** Het oppervlak van de zwembadruimte telt nu mee met de sportzalen (p. 65). `sportHallAreaM2` is voortaan het oppervlak zonder de zwembadruimte.
- **L_max van koelleidingen.** Een opgenomen `cooling.maxPipeLengthM` vervangt de forfaitaire L_max van 10.27 in de pompberekening.
- **Geïnstalleerde capaciteit met een zwembad.** Komt de capaciteit uit de passieve koeling, dan telt hij bij een zwembad in de zone nu ook als onbekend (p. 148).

### Opnames die nu `incomplete` worden

- **Capaciteit dubbel opgegeven.** `ventilation.installedCapacityDm3PerS` naast `ventilation.passiveCooling.installedCapacityDm3PerS` geeft `installed_capacity_given_twice`. Voorheen won stilzwijgend de waarde van de passieve koeling.
- **Gasmotor tot en met 2 kW uit 2006 of eerder.** NTA-tabel 9.31 heeft hiervoor geen waarden. De opname meldt nu `gas_engine_small_old_no_table_row`; voorheen faalde de kern zonder duidelijke melding.
- **Zwembad plus sportzalen groter dan de sportfunctie.** Dit geeft `sport_hall_area_invalid`.

### Nu toegestaan

- **Passieve koeling bij systeem E naast natuurlijke ventilatie** in de utiliteitsopname (p. 152). De woningopname stond dit al toe.
- **Bypasspercentage als bypass.** Een opgenomen bypasspercentage van 10 % of meer telt voor passieve koeling als aanwezige bypass.
- **Verborgen directe-expansieantwoord.** Bij een watergevoerd systeem wordt het genegeerd in plaats van `cooling_direct_expansion_not_water_based`. Het formulier wist het antwoord bij het aanvinken van "watergevoerd".

### Inklapredenen

- Opgeslagen inklapredenen met een hernoemd pad of een gesplitste regel blijven gekoppeld (aliastabel), in plaats van een waarschuwing `collapse_reason_unmatched` te geven.

### Rekenzones

- `calculation_zone_split_required` toont in het paneel nu de vervolgstap. Het ontwerp voor meerdere rekenzones in de opname staat in `docs/nta8800-basisopname.md`.

## 3 oktober 2026 — reviewcorrecties woningopname (ISSO 82.1)

### Opnames die nu `incomplete` worden

- **Individuele gaswarmtepomp tot en met 25 kW.** De GWP-rijen van NTA-tabel 9.27 (p. 334) gelden alleen voor een collectieve gebouwinstallatie. Tabel 9.29 geldt voor collectieve installaties en voor meer dan 25 kW. Een individuele gasmotor- of gasabsorptiewarmtepomp tot en met 25 kW heeft dus geen forfaitaire rij en geeft nu `gas_heat_pump_individual_no_forfait_row`. Tot nu toe rekende de opname met de GWP-rijen van tabel 9.27.
- **Collectieve warmtepompbron zonder bewijs.** Een aangevinkte collectieve bron met een lege verwijzing gaf stilzwijgend een individuele bron, zonder tabel V.3 en zonder 9.62. Volgens ISSO 82.1 p. 111 blijkt een collectieve bron uit facturen of ontwerpgegevens. De opname meldt nu `collective_source_reference_required`.
- **Systeem E zonder WTW.** Bij §11.3.6 (p. 145) heeft het decentrale deel altijd WTW. Daarom kan de standaardwaarde "geen WTW" van tabel 11.9 hier niet gelden. Een gecombineerd systeem met een onbekende of ontbrekende wisselaar geeft `combined_requires_heat_recovery`.
- **Lege g-waarde bij zonwerend glas.** Een lege `solarControl.gValue` gaf een leesfout van de hele opname. Nu geeft hij `solar_control_g_invalid` op dat veld.

### Opnames met een andere uitkomst

- **Bypass bij de opname afwezig.** De jaarregel van tabel 11.12 (p. 151–152) geldt alleen als de bypass of het bypasspercentage onbekend is. Een unit uit 2010 of later met `bypassPresent: false` kreeg ten onrechte 100 % bypass en krijgt nu 0 %.
- **Oppervlaktewater bij een collectieve installatie.** Volgens ISSO 82.1 p. 111 is oppervlaktewater een invoerkeuze bij een collectieve installatie. Dat geldt ook zonder collectieve bron. Zo'n installatie rekent nu met de rij oppervlaktewater van tabel 9.29 in plaats van met de rij bodem. Een gaswarmtepomp tot en met 25 kW in een collectieve installatie neemt de rij grondwater van tabel 9.27, want die tabel heeft geen rij oppervlaktewater.

### Nu toegestaan

- **Passieve koeling bij systeem E naast natuurlijke ventilatie.** Volgens p. 152 kan passieve koeling voorkomen bij de systemen B tot en met E. Het decentrale deel moet dan wel een bypass hebben.

### Formulier

- **Verborgen antwoorden worden gewist.** Kies je een andere bron of een ander toestel, dan wist het formulier de antwoorden die daarbij niet meer zichtbaar zijn: collectieve bron, grondwatersysteem, brontemperatuur, kwaliteitsverklaring en de brandstof van een stoomketel. Zo leveren ze geen `collective_source_water_based_only` of `local_heater_fuel_contradiction` meer op.
## 3 oktober 2026 — basisopname utiliteitsgebouwen volgens ISSO 75.1 (7e druk)

### Opnames die nu `incomplete` worden

- **Splitsing in rekenzones (afb. 6.6 met tabel 6.4, p. 53–54).** Na het samenvoegen van p. 39–40 stopt de opname met `calculation_zone_split_required` wanneer de setpoints van de overgebleven functies meer dan 4 K verschillen, of wanneer bij ventilatietype A, B, C of E de ventilatiecapaciteit meer dan een factor 4 verschilt. Een voorbeeld is onderwijs naast een sportfunctie van meer dan 25 %. De uitzondering voor verblijfsgebieden in open verbinding staat in `openlyConnectedResidenceAreas`.
- **Gasmotor-koelmachine zonder elektrisch vermogen** (`gas_engine_power_required`, tabel 10.2).
- **Meer koudeopwekkers zonder vermogen** (`cooling_generator_capacity_required`, §10.3.2 met NTA 10.49).
- **Directe expansie in de LBK** zonder LBK of met "niet aangesloten" op de koelbatterij.

### Opnames met een andere uitkomst

- **f_BACS uit de opgenomen vermogens (tabel 7.3, p. 62–63).** Zonder `bacs.systemPowerKw` beslist nu het opgenomen vermogen van verwarming en koeling. Een ketel van 350 kW in een gebouw kleiner dan 2.500 m² krijgt zo f_BACS 1,05 (eerder 1,0). Zijn alle systemen bekend en ten hoogste 290 kW, dan is het 1,0, ook boven 2.500 m².
- **Dynamisch ingeregelde koeldistributie** krijgt f_HB 1,0 (NTA-tabel 10.11) in plaats van 1,15. Dat geldt ook voor de koeling in de woningopname.
- **Collectieve verwarming.** De distributie leest nu de antwoorden van tabel 9.12 (leidingisolatie, isolatiejaar, appendages) en het eenpijpssysteem; eerder waren de leidingen altijd ongeïsoleerd.
- **Opgenomen koudeopwekkerpunten.** Opgegeven appendage-isolatie, koudemeters en leidinglengten gaan nu naar de kern.
- **Bronvermelding.** `derive_utility_input` geeft zelf al de ISSO 75.1-pagina's.

### Nieuwe opties

- Fossiele brandstof op het perceel, oppervlak van sport- en zwemzalen, ruimte met zwembad.
- Koeling: gasmotor-koelmachine, meer opwekkers met prioriteit, directe expansie in ruimte of LBK, koudemeters, appendages, leidinglengten.
- Ventilatie: LUKA D en geen kanaal, "koude laden met LBK", decentrale WTW, isolatie en lengte van de buitenaansluiting, constant volumeregeling, gedeeltelijke bypass in procenten, geïnstalleerde capaciteit, systeem E en roosters met verwarmingslint.

## 3 oktober 2026 — basisopname woningen volgens ISSO 82.1 (7e druk met erratum)

### Opnames die nu `incomplete` worden

- **Gesloten of verlaagd plafond in de woningopname.** `construction.closedOrSuspendedCeiling` komt uit ISSO 75.1. ISSO 82.1 tabel 7.4 (p. 62) kent alleen `lighterCeiling`. De woningopname meldt nu `closed_ceiling_not_in_dwelling_survey`.
- **Warmtepomp met een opgegeven klasse boven 70 °C.** Volgens tabel 9.9 en erratum §4 is dan een gecontroleerde verklaring nodig: `heating.heatPumpAbove70Declaration`, anders `heat_pump_above_70_requires_declaration`.
- **Bewijsstukken bij nieuwe opties.** Ventilatiesturing zonder bewijsstuk geeft `ventilation_controls_evidence_required`. Zonwerend glas zonder bron geeft `solar_control_evidence_required`.

### Opnames met een andere uitkomst

- **Bypass bij een onbekend bypassaandeel (tabel 11.12, p. 151–152).** Het fabricagejaar van de WTW-unit gaat nu voor het bouwjaar. Een unit van vóór 2010 in een woning van 2010 of later krijgt dus geen 100 %-bypass meer, maar 70 % (bypass aanwezig) of 0 %.
- **Ketel bij klasse 70/50.** De gemiddelde ontwerptemperatuur is nu 65 °C in plaats van 60 °C, gelijk aan de distributieklasse 70/60. De uitkomst verandert niet, omdat alleen de grens van 50 °C telt.

### Nieuwe opties

- Ventilatiesturing volgens de tabellen 11.4–11.6 zonder `declaredVariant`.
- Gecombineerd systeem E.
- Roosters met verwarmingslint.
- Woningpositie "dak + vloer".
- Zonwerend glas of folie met een g-waarde uit het product.
- Centrale of decentrale WTW.
## 3 oktober 2026 — verwarmingsopties in de basisopname (ISSO 82.1 hoofdstuk 9)

### Nieuwe, aanvullende invoer

- **Opwekkers (tabel 9.3).** `boilerType: oil` (olieketel, conventioneel), `local_fired` (lokale gasverwarming, olieverwarming of stoomketel, met of zonder afvoer) en `gas_air_heater` (direct gestookte luchtverwarmers). De kern rekent ze met NTA-tabel 9.25.
- **Warmtepompen (tabel 9.6).** `drive` (elektrisch, gasmotor, gasabsorptie), de bronnen `heat_pump_panel` en `high_temperature`, `groundwaterSystem` (doublet of recirculatie), `collectiveSourceReference`, `sourceTemperatureC` en `sourceQualityDeclarationReference`.
- **Distributie (§9.4.2, tabel 9.12).** `distributionType` (tweepijps, eenpijps met aantal afgiftetoestellen, gerenoveerd eenpijps) en `pipeInsulation` (geïsoleerd, isolatiejaar, appendages en beugels).
- **Kern.** `pump.onePipeEmitterCount` telt de weerstand per afgiftetoestel van een eenpijpskring (tabel 9.21, p. 317). `forfait_heater` mag zonder `nominalPowerKw`; 9.92 rekent dan met de bovengrens t_on = t_mi.

### Opnames met een andere uitkomst

- **Grondwaterwarmtepomp zonder brontemperatuur.** Deze rekent nu met de rij bodem (NTA p. 335). Eerder werd de rij grondwater gekozen, wat de kern zonder temperatuurbewijs afwees.
- **Collectieve warmtepomp in de woningopname.** Deze rekent nu met tabel 9.29 en krijgt hulpenergie volgens 9.91. Eerder volgde `table_scope_capacity_mismatch` en ontbrak de hulpenergie.
- **Warmtepomp boven 25 kW in de woningopname.** Deze rekent nu met tabel 9.29.

## 3 oktober 2026 — koelmethode 1 en bijlage Q

### Projecten die nu `invalid` worden

- **Vijfde meetpunt NEN-EN 14825 (10.63, p. 408–409).** Een vijfde punt moet de deellast van punt C en de condensorintredetemperatuur van punt A hebben (tolerantie 0,5 % of 0,5 K), anders `cooling_en14825_fifth_point_conditions`. De testpunten staan in de volgorde A, B, C, D.
- **Zonder vijfde punt (10.64).** De benadering met Δϑ_corr = 0 klopt alleen als de verdamperuittrede bij A en C gelijk is. Verschillen ze meer dan 0,5 K, dan volgt `cooling_en14825_fifth_point_required`.
- **Methode 1 alleen voor modulerende opwekkers (§10.5.4, p. 401).** Een minimumvermogen gelijk aan of boven het nominale vermogen geeft `cooling_en14825_modulating_required`. Een minimumvermogen boven het nominale gaf eerder `cooling_performance_invalid`.
- **Lucht/luchtwarmtepomp volgens bijlage Q.** Met waterafgifte (radiatoren, vloerverwarming, ventilatorradiatoren) of hydraulische distributiegegevens volgt `annex_q_air_air_hydronic_chain`.

### Projecten met een andere uitkomst

- **Bijlage Q met bijverwarming.** Dekt de warmtepomp elke temperatuurklasse van tabel Q.6, dan geldt nu F_H;gen = 1, ook als er bijverwarming is opgegeven. Eerder kreeg de bijverwarming door de afronding van tabel Q.6 een restaandeel van enkele honderdsten procent.

### Nieuwe waarschuwing

- **Deellast boven 100 % (10.56/10.58).** Komt f_C;PL in een temperatuurklasse boven 100 %, dan extrapoleert de kubische functie van 10.63 buiten het meetbereik. De kern houdt de letterlijke uitkomst aan en meldt `cooling_part_load_above_full_load`. Een te klein toestel kan zo gunstiger uitkomen.

## 3 oktober 2026 — dynamische ramen in de projectroute (bijlage A)

- **Nieuw, aanvullend veld** `ntaCalculation.dynamicWindows`: bijlage A (p. 766–770) per buitenraam, methode A of B, met de correctiefactoren van stap 2. Opgeslagen projecten zonder dit veld houden dezelfde uitkomst.
- In het NTA-formulier is dit invoerbaar onder "Dynamische ramen (bijlage A)". Eerder kon het alleen via de kerninvoer.
- **Nieuwe gap** `window_dynamic_and_shading_exclusive`: een dynamisch raam samen met beweegbare zonwering (7.42) telt de zonwering dubbel (§A.2, p. 767). Neem de zonwering op in de toestanden.
- **Lege waarden** in bijlage A (g, U, wegingen, correctiefactoren) geven `dynamic_value_missing` op hun eigen pad. Eerder blokkeerde één leeg veld het hele NTA-blok.
- τ_vis en τ_sol staan niet meer in het formulier: hoofdstuk 14 gebruikt ze niet (14.38, 14.41). Opgeslagen waarden blijven bewaard.

## 3 oktober 2026 — validatieregels en herberekeningsbevindingen

### Projecten die nu `incomplete` worden

- **Verticale leidingen (§7.3.3).**
  - Ontbreekt `verticalPipes` (project of zone), dan geldt dat als "onbekend" en volgt de gap `vertical_pipes_unknown`. Vul de leidingen in, of `[]` voor "geen".
  - Een meerzonig project zonder lijsten per zone krijgt dezelfde gap.
  - Staat bij een zone `[]` terwijl op projectniveau leidingen zijn opgegeven, dan volgt `vertical_pipes_conflicting`. Eerder vielen de projectleidingen dan stil weg.
- **Koudebrugmethode (§8.2.1, §8.3.3.1).**
  - Een forfaitaire vloerrand naast ψ-waarden geeft `thermal_bridge_methods_mixed`.
  - Forfaitair met een onverwarmde ruimte geeft `forfait_thermal_bridges_unheated_space_unsupported`.
- **Verwarmde kelder in de forfaitaire route (8.38).**
  - De kern vult ΔU_for voor de kelderwanden nu zelf in met de waarde van 8.3.
  - Een opgegeven waarde die daarvan afwijkt geeft `basement_forfait_delta_u_conflict`.

- **Ventilatie (11.60/11.61, p. 468).** Een terugregel-x gunstiger dan de standaard (recirculatie boven 20 %, debietregeling onder 80 %) zonder `flowReduction.evidenceReference` geeft `flow_reduction_evidence_required`.
- **Gemeten luchtdoorlatendheid.** Een q_v10 van 0 of lager geeft `infiltration_invalid`.

### Projecten met een andere uitkomst

- **Forfaitaire koudebruggen.**
  - H_D krijgt ΔU_for (8.2/8.3) op alle elementen, glas inbegrepen.
  - De zonwinst van dichte delen (7.33) en de uitstraling naar de hemel (7.39) houden U_c van 8.2.2, dus zonder ΔU_for.
- **Gemeten opslagverlies.**
  - H_sto;ls wordt naar boven afgerond volgens bijlage X (p. 568).
  - Nieuw is de route `measured_standby` (13.60).
  - Ongeldige waarden worden afgewezen, ook bij zonneboilervaten.
- **Koelmachine (10.73, tabel 10.8).**
  - De benodigde uittredetemperatuur is nu de aanvoertemperatuur min Δϑ_int;inc.
  - Bij verdamping in de ruimte is dat ϑ_C;int;inc (10.10).
  - Het EER en het elektriciteitsgebruik veranderen daardoor.
- **EN 16147-correcties (13.153b/13.153c).** De invoer `smartControlFactor`, `maxTestTemperatureC` en `designSetTemperatureC` is optioneel; zonder deze invoer verandert er niets.
- **BENG 1.** Wordt alleen getoond uit de run met vast ventilatiesysteem C1 (§5.4).
- **TOjuli bij utiliteit.** Er geldt geen Bbl-grenswaarde ("niet van toepassing").
- **Voorbeeldprojecten.** Tapwater wordt nu berekend in plaats van opgegeven. Zie `nta8800-voorbeeldproject-smoketest-2026-10-03.md`.

### Nieuwe waarschuwingen (de berekening loopt door)

- `declared_hot_water_efficiency_above_one`
- `declared_ventilation_below_required_flow`: alleen bij woningen, met de laagste f_ctrl·f_sys van tabel 11.5.
- `bacs_factor_without_capacity_evidence`
- `utility_open_ceiling_requires_evidence`
- `cooling_emission_loss_singular`
- `hot_water_circulation_defaults_low_efficiency`
- `lighting_large_office_group_without_office`: `largeOfficeGroup` in een zone zonder kantoorfunctie (§14.5.1, p. 664). Dit geeft F_o;D = 1, de minst gunstige waarde, en blokkeert dus niet.

Het rekenrapport toont zowel de projectmeldingen als de meldingen van de kern.
