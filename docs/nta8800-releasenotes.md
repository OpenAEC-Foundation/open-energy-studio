# NTA 8800-kern — releasenotes

Wijzigingen die de uitkomst of de status van bestaande, opgeslagen projecten veranderen. Normverwijzingen gaan naar NTA 8800:2025+C1:2026, met paragraaf-, formule- en paginanummers.

## Afspraken voor dit bestand

Dit bestand is de wijzigingsregistratie van de rekenkern (BRL 9501 §5.2, §6.2 en §6.4). `scripts/nta-kernel-version.mjs check` controleert het in de gate tegen `KERNEL_VERSION` in `crates/nta8800-core/Cargo.toml`.

- **Nieuwe items** komen bovenaan, onder "Onuitgebracht", als `### <datum> — <titel>`. Een item met de oude kop `## <datum> — <titel>` boven de eerste versiesectie telt ook als onuitgebracht.
- **Uitkomstwijziging.** Een item verandert uitkomsten of de status van opgeslagen projecten, tenzij de titel "(geen rekenwijziging)" vermeldt.
- **Versie ophogen.** Staat er onder "Onuitgebracht" een item dat uitkomsten verandert, dan moet `KERNEL_VERSION` minstens een MINOR-stap boven de laatste uitgave staan (zie [versiebeheer](nta8800-versiebeheer.md)). Eén ophoging per uitgave volstaat; latere items in dezelfde ronde vragen geen nieuwe.
- **Uitgave.** `scripts/release-nta.sh` zet de onuitgebrachte items onder `## Rekenkern X.Y.Z — <datum>` met de regel `<!-- kernel-version: X.Y.Z -->` eronder. Versies lopen af door het bestand.

## Onuitgebracht

### 9 oktober 2026 — gedeclareerde f_ctrl en zonwering per raam

Twee nieuwe invoerroutes. Projecten zonder deze invoer rekenen precies als voorheen.

- **Gedeclareerde f_ctrl** (`ventilation.system.unit.declaredControlFactor`: `value` en `declarationReference`). Een kwaliteitsverklaring van het toegepaste ventilatiesysteem, zoals een BCRG-gelijkwaardigheidsverklaring, mag de rekenwaarde van tabel 11.5 vervangen (2025+C1 p. 7 en p. 460). De waarde geldt als f_ctrl;tabel 11.5 in 11.48/11.49, bij E.1 als f_ctrl;overig. Nieuwe meldcodes: `declared_control_factor_invalid` (niet groter dan 0 of boven 2) en `declared_control_factor_not_applicable` (op het decentrale deel van E.1).
- **Zonwering per raam** (`windowShadings`: per raam `windowId`, `movableShading` en `sourceReference`). 7.42 en 7.43 gelden per raam (2025+C1 p. 196–198). Een raam in de lijst krijgt zijn eigen zonwering of geen; de andere ramen houden `windowSolar.movableShading`. Nieuwe meldcodes: `window_shading_duplicate`, `window_shading_without_window`, `window_shading_not_outdoor` en `window_shading_reference_required`; een ongeldige F_c of een lege bron is een invoergat op het eigen pad.
- **Formulier:** een vinkje "Gedeclareerde f_ctrl" bij het ventilatiesysteem en een tabel "Zonwering per raam" onder Gebouw › Schil & ramen.
- **Openbare gevallen:** A rekent met de f_ctrl 0,50 en G met de f_ctrl 0,51 en de screens op het dakraam, zoals de rapporten.
  - A: BENG 2 35,15 → 34,37 (2025+C1) en 36,06 → 35,29 (2020+A1).
  - G: 65,67 / 30,89 / 71,8 → 65,42 / 29,71 / 72,1; het rapport geeft 64,52 / 29,44 / 70,6.

### 9 oktober 2026 — invoergaten van geweigerde routes per zone, raam en systeem; strengere controle van de releasenotes (geen rekenwijziging)

Uitkomsten, status en meldcodes veranderen niet; alleen het pad van een invoergat en de controles rond de vrijgave.

- **Invoergaten in projecten met meer zones.** Een geweigerde route in de tweede zone kwam terecht bij de eerste zone of het projectblok. Nu volgt het pad de afleiding:
  - een vloer op grond via zijn oppervlak-id naar `ntaCalculation.groundFloors[i]` (eerder via de volgorde binnen de zone, wat bij meer zones of een andere volgorde een andere vloer aanwees);
  - afgifte, distributie, interne warmtelast, thermische massa, setpoints, woningtype, zonnekamers en verticale leidingen naar `ntaCalculation.zoneData[k]` als die zone ze daar zelf opgeeft, anders naar het projectblok;
  - een raam naar zijn eigen belemmering (`windowObstructions`), zijn bijlage A-gegevens (`dynamicWindows`), het blok `windowSolar` of het raam zelf in de zone (oppervlak, U, g);
  - de collectieve aansluiting en het aantal identieke systemen van een extra verwarmingssysteem naar `ntaCalculation.additionalHeatingSystems[i]`.
  
  "Ga naar" brengt je daarmee naar het juiste veld. Afgifte en distributie per zone gaan naar *Installaties › Verwarming*, zonnekamers per zone naar *Onverwarmde ruimten*.
- **Controle van de releasenotes.** Een kop boven de eerste versiesectie die bijna een item was (en-streepje of koppelteken, maand met hoofdletter of afgekort, ISO-datum, `####`, of een bestand met CRLF-regeleinden) werd overgeslagen, zodat een uitkomstwijziging zonder ophoging door de gate kwam. Nu is elke kop daar een structuurkop, een item in exact de vorm `### <d> <maand> <jjjj> — <titel>` of een subkop van een item; al het andere is een fout. Alleen het exacte slot "(geen rekenwijziging)" maakt een item vrij.
- **Vrijgave.** `scripts/release-nta.sh` vergelijkt de uitgebrachte versiesecties met de vorige vrijgavetag (`check-released`): een item dat achteraf in een uitgebrachte sectie wordt gezet, stopt de vrijgave. De releasenotes worden pas na de gate en de build gecommit; mislukt een vrijgave vóór de tag, dan gaat het archief opzij en kan de vrijgave opnieuw.
- **Handleiding.** Een link met een stuurteken (zoals `java<tab>script:`) wordt niet meer als link getoond.

### 9 oktober 2026 — openbare gevallen G (NTA 8800:2024) en H (eerste utiliteitsgebouw) (geen rekenwijziging)

- **G**, vrijstaande woning met verwarmde kelder, Uniec 3.3.5.0 (29-04-2025), het eerste geval in de editie 2024: kern 65,67 / 30,89 / 71,8 tegen 64,52 / 29,44 / 70,6. Tapwater (5 545 kWh), PV, ventilatoren en de COP zijn gelijk. Het verschil zit in de product-f_ctrl 0,51 uit een BCRG-verklaring (de kern rekent tabel 11.5, 0,59) en in de letterlijke 10.15 bij koeling.
- **H**, utiliteitsgebouw met kantoor, bijeenkomst, zorg en sportschool, 743,80 m² in twee zones, Uniec 3.2.9.2 (29-05-2024, editie 2023): kern 27,29 / 80,63 / 29,7 tegen 29,08 / 80,01 / 30,5. Verlichting, tapwater, PV, de COP en de Bbl-grenswaarden van het gemengde gebouw (44,94 / 86,22 / 30,4) zijn gelijk.
- Beide staan in `tests/public_comparison.rs` en in de referentiesuite `openbare-gevallen.json`. Zie [de vergelijking](nta8800-vergelijking-openbare-rapporten.md#g-in-2024-en-h-het-eerste-utiliteitsgebouw-in-2023-9-oktober-2026).
- Bevinding zonder codewijziging: de kern kent geen gedeclareerde f_ctrl (gelijkwaardigheidsverklaring) en geen zonwering op één raam.

### 9 oktober 2026 — procedures voor het kwaliteitssysteem (geen rekenwijziging)

- **Nieuwe map `docs/kwaliteit/`** met concepten die de attesthouder alleen nog hoeft in te vullen en vast te stellen (BRL 9501 hoofdstuk 5 en 6, taken C2, C3, C4 en C6):
  - [kwaliteitshandboek](kwaliteit/kwaliteitshandboek.md): rollen, overzicht van alle procedures met hun bewijs, jaarlijkse review;
  - [meldprocedure wijzigingen](kwaliteit/meldprocedure-wijzigingen.md) (§5.2, §6.4): wanneer en hoe instelling en licentiehouders bericht krijgen, gekoppeld aan "Onuitgebracht" en `KERNEL_VERSION`, met sjabloon en meldregister;
  - [klachtenprocedure](kwaliteit/klachtenprocedure.md) (§6.5): verloop met termijnen, oorzaakanalyse (rekenfout, interpretatie, interface, gebruiker) en een registersjabloon;
  - [register van licentiehouders](kwaliteit/register-licentiehouders.md) (§6.2.1): velden en CSV-sjablonen, gekoppeld aan `SHA256SUMS` per vrijgave;
  - [archiefbeleid](kwaliteit/archiefbeleid.md) (§5.3, §6.2–6.3): bewaartermijnen, twee kopieën, integriteitscontrole en het terughalen van een oude versie.
- De [gereedheidsanalyse](nta8800-brl9501-gereedheid.md) telt nu 6 voldaan, 14 deels, 6 open, 7 extern en 1 n.v.t.

### 9 oktober 2026 — luchtspouw dunner dan 20 mm rekent; rekenkern 0.3.0

- **Dunne luchtspouw** (tabel C.3/C.4, voetnoot c en e, p. 782 en 784). Een niet-geventileerde luchtspouw dunner dan 20 mm zonder werkzame reflecterende laag gaf `air_cavity_below_20_mm_unsupported`. Hij rekent nu met bijlage D.2 van NEN-EN-ISO 6946:2017 en de parameters van voetnoot e: ε1 = ε2 = 0,9 en h_r0 bij 10 °C. Onder 20 mm bepaalt de geleiding van stilstaande lucht (λ 0,025 W/(m·K) gedeeld door de dikte) de convectieve term. De waarde wordt afgerond op 2 decimalen, net als de tabelwaarden. Voorbeelden, horizontale warmtestroom: 5 mm 0,11, 10 mm 0,15, 15 mm 0,17 (m²·K)/W.
  - **Controle.** Dezelfde methode geeft elke waarde van tabel C.3 en C.4 terug, ook de waarden tussen haakjes (ε2 = 0,1) en de geleidingstak van de neerwaartse rijen van 20 tot 50 mm. Test `d2_method_reproduces_tables_c3_and_c4`.
  - **Nog geweigerd.** Een zwak geventileerde dunne spouw en een dunne spouw met een werkzame reflecterende laag houden `air_cavity_below_20_mm_unsupported`: voetnoot c dekt alleen spouwen zonder reflecterende folie, en C.12 vraagt een tabelwaarde voor zwakke ventilatie. Een sterk geventileerde dunne spouw rekent zoals een dikkere (C.3.3). Een dikte van 0 of minder geeft nu `thickness_invalid`.
  - **Alle uitgaven.** Voetnoot c en de waarden van tabel C.3/C.4 zijn gelijk in 2020+A1 (p. 749–751), 2022 (p. 754–756), 2023 (p. 764–766), 2024 (p. 762–764) en 2025+C1; er is geen schakelpunt.
  - **Effect.** Projecten die eerder rekenden, veranderen niet. Een constructie die eerder werd geweigerd, rekent nu. Daarom is `KERNEL_VERSION` verhoogd naar 0.3.0.
- **Optiedekking.** De sweep `option_coverage.rs` loopt nu ook de constructies en schildelen door (route `assess_envelope`, onder elke uitgave met `with_version`), plus vier dunne-spouwgevallen. Totaal 9 000 berekeningen zonder fout; alle codes hebben een tekst.
- **Uurklimaat 17.3.8** blijft als opgegeven factoren. De uitgebreide methode vraagt de uurwaarden van NEN 5060:2018+A1:2021 (zonnestand, totale straling en het deel direct plus circumsolair, per uur); die norm is niet beschikbaar. Zie [verificatiestatus](nta8800-verificatiestatus.md).

### 9 oktober 2026 — handleiding in de app en bij elke vrijgave (geen rekenwijziging)

- **Handleiding in de app** (BRL 9501 §4.4, taak B7). *Gereedschap › Handleiding* toont de gebruikershandleiding uit `docs/handleiding-nta8800/`. De hoofdstukken en afbeeldingen zitten in de build, dus de desktop-app toont ze ook offline. Links tussen hoofdstukken blijven in de viewer; verwijzingen naar andere documenten tonen hun pad. Naast de inhoud staan de programmaversie, de rekenkernversie waarvoor de handleiding is nagelopen en de rekenkern van deze installatie; verschillen die, dan meldt de viewer dat.
- **Contextgevoelig.** Elke werkstap heeft een knop *Handleiding* naar het hoofdstuk over die stap. *Instellingen › Over* toont de handleidingversie en opent de handleiding, ook zonder open project.
- **Bij de vrijgave.** `scripts/release-nta.sh` zet de handleiding als één HTML-bestand in het archief, met beide versies erin en de SHA-256 in het leveringsdocument, `manifest.json` en `SHA256SUMS`. Een PDF volgt alleen als chromium of wkhtmltopdf aanwezig is.
- **Gatecontrole.** `scripts/nta-manual.mjs check` faalt als de stempel in `index.md` niet de huidige `KERNEL_VERSION` noemt.
- De Markdown wordt door één eigen, kleine parser gelezen (`scripts/nta-manual-markdown.mjs`), zonder nieuwe afhankelijkheid. Alle tekst wordt als tekst getoond; onveilige links (`javascript:` en dergelijke) worden niet gevolgd.

### 9 oktober 2026 — labelelementen k en l in de labelgegevens van de kern; ids voor extra opwekkers in de opname

Rekenkern 0.3.0. Berekende getallen veranderen niet; de labelgegevens en de controle van de opname wel.

- **Labelelementen k en l** (Omgevingsregeling art. 5.13a lid 1 onder k en l). De kern neemt de verklaringen van de adviseur uit `registration.labelStatements` over in `labelData.indicators.elements` (`respondsToExternalSignals`, `lowTemperatureHeating`). Tot nu toe bleven die `null`. Een onbeantwoorde verklaring blijft `null`. Het rapport en het EP-Online-overzicht nemen de waarden van de kern; zonder labelgegevens (geen berekend resultaat) vallen ze terug op de registratie. De algemene rijen met labelelementen in het rapport tonen k en l niet nog eens.
- **Ids in de opname.** Extra verwarmingsopwekkers (`heating.additionalGenerators`), extra tapwateropwekkers (`hotWater.additionalGenerators`, ook binnen een extra tapwatersysteem) en extra tapwatersystemen (`additionalHotWaterSystems`) kunnen een `id` hebben, in de woning- en de utiliteitsopname. De kern gebruikt hem niet in de berekening; een lege id geeft `survey_item_id_blank`, dezelfde id twee keer in één lijst `survey_item_id_duplicate`. De opname geeft een nieuw onderdeel een id (`opwekker-1`, `tapwateropwekker-1`, `tapwatersysteem-1`). Een project uit een eerdere versie krijgt de ontbrekende ids bij het openen; koppelingen van bewijs en foto's op positie gaan dan mee naar de id.

### 9 oktober 2026 — uitkomst van de opname in het onderdeel; snellere interfacetests (geen rekenwijziging)

- **Opname doorrekenen** geeft nu terugkoppeling in het onderdeel waar je staat: status, indicatief label en EP₂, de fouten van dat onderdeel met **Ga naar het veld**, het aantal fouten elders en **Volledige uitkomst**. Eerder verscheen na doorrekenen vanuit Algemeen niets op de pagina zelf; de uitkomst stond alleen onder Uitkomst & forfaitair. De kaart *Uitkomst opname* heeft nu ook **Overnemen in projectmodel**, zoals het ontwerp vraagt.
- Interfacetests zoeken formuliervelden op hun label (`getByLabelText`) in plaats van op rol en naam. `getByRole` met een naam berekent voor elk kandidaat-element de toegankelijke naam via de stijlcascade van jsdom; in de traagste opnametest was dat 20,8 van 37 s rekentijd, nu 5,7 s. Keuzerondjes en tests die op de rol zelf toetsen houden `getByRole`.

## Rekenkern 0.2.0 — 8 oktober 2026
<!-- kernel-version: 0.2.0 -->

Eerste uitgave onder het afgedwongen versiebeleid. Ze bundelt alle wijzigingen sinds 0.1.0 (30 september 2026), waaronder veel die uitkomsten veranderen: de oudere uitgaven 2024, 2023, 2022 en 2020+A1, de herstelde routes en de controles hieronder. `KERNEL_VERSION` stond in die periode ten onrechte nog op 0.1.0; een project dat met "0.1.0" is gestempeld en na 30 september is berekend, kan met elke stand van deze reeks zijn gemaakt.

### 8 oktober 2026 — versiebeleid, referentiesuites en vrijgave (geen rekenwijziging)

- **Rekenkernversie 0.2.0.** `KERNEL_VERSION` gaat van 0.1.0 naar 0.2.0 (zie de inleiding van deze sectie en [versiebeheer](nta8800-versiebeheer.md)). Een herlabelregistratie met de oorspronkelijke kern 0.1.0 meldt nu `relabel_kernel_version_differs`, zoals het beleid voor een andere kern vraagt.
- **Releasenotes per versie.** Dit bestand is ingedeeld per rekenkernversie. De gate (`scripts/nta-kernel-version.mjs check`) faalt als een onuitgebracht item uitkomsten verandert zonder MINOR-ophoging.
- **Referentieharnas.**
  - Een verwachte waarde mag naast `absoluteTolerance` een `relativeTolerance` hebben; de ruimste band geldt.
  - Een referentiegeval mag elke uitgave noemen die de kern rekent (label of id), mits de invoer dezelfde uitgave draagt. Eerder weigerde het harnas alles behalve 2025+C1 met `norm_version_mismatch`.
  - `inputKind: "buildingPerformance"` vergelijkt een gebouwinvoer, met de nieuwe grootheid `chapter5HeatingNeed`.
  - `reference_gate` leest suites (`--suite`) en schrijft een rapport met commit, `KERNEL_VERSION` en SHA-256 per invoer en uitvoer (`--report-dir`).
- **Suites in elke gate.** `training-data/reference-suites/` bevat de openbare gevallen A–F in hun eigen uitgave en de zes RVO-voorbeeldwoningen (±6 %-band rond RVO en de eigen regressie). De gedocumenteerde uitkomsten blijven gelijk.
- **Openbare gevallen A en D.** Het lege object `airTightness: {}` in de zone is weggelaten; de structurele controle weigerde het. Uitkomsten veranderen niet.
- **Vrijgave.** `scripts/release-nta.sh` maakt een archief met gatelog, referentierapport, pakketten, leveringsdocument, manifest en `SHA256SUMS`, en een lokale tag. Het attestnummer en de identificatiecode staan in `src/core/nta/attest.json`.

### 8 oktober 2026 — geweigerde routes als invoergat; toesteljaar warmtepomp in de opname

- **Geweigerde route als invoergat.** Weigerde de gebouwberekening een route (bijvoorbeeld `route_not_in_edition` voor LED vanaf 2017 of PV per paneel onder 2020+A1), dan gaf het project status `invalid` met een lege lijst invoergaten; de code stond alleen in `performance.issues`. Nu wordt elke melding van een niet-berekende gebouwberekening een invoergat op de projectinvoer die haar voedt, bijvoorbeeld `ntaCalculation.pvSystems[0].peakPower.panelPeakPowerW` of `ntaCalculation.internalGains.lighting`. Het pad in de afgeleide invoer staat in `detail`. "Ga naar" opent zo de juiste stap en het juiste veld. De uitkomst van berekende projecten verandert niet.
- **Labelfunctie.** `ntaCalculation.labelFunction` heeft een veld in de stap Project en een route voor "Ga naar"; een labelfunctie die niet bij het berekeningstype past (`label_function_scope_mismatch`) wees eerder naar het controleoverzicht.
- **Controle.** De dekkingstest van alle opties (`option_coverage.rs`) eist nu ook dat een niet-berekend project minstens één invoergat heeft en dat elk pad een route heeft in `gapRoutes.ts`. Een service-test controleert het 422-antwoord met het invoergat.
- **Opname, warmtepomp.** Het opnameformulier vraagt bij een warmtepomp het fabricagejaar en het installatiejaar. Alleen NTA 8800:2020+A1 gebruikt ze (hulpenergie, formule 9.85; volgorde volgens ISSO 82.1 p. 28).

### 8 oktober 2026 — verklaringen k en l bij het label; attestnummer en NL-EPBD-merk

- **Verklaringen k en l (Omgevingsregeling art. 5.13a lid 1).** De registratie heeft nu `labelStatements` met twee ja/nee-verklaringen van de adviseur:
  - `respondsToExternalSignals` (k): kan het gebouw reageren op externe signalen en zijn energiegebruik aanpassen;
  - `lowTemperatureHeating` (l): is het verwarmingsdistributiesysteem ontworpen voor lage temperatuur.

  Voor een registratie vanaf 29 mei 2026 meldt de kern `label_statement_external_signals_required` en `label_statement_low_temperature_required` (ernst `missing`). De registratiedatum beslist, anders de opnamedatum; zonder datum worden ze gevraagd. **Gevolg voor opgeslagen projecten:** een registratie vanaf 29 mei 2026 zonder deze verklaringen is niet meer volledig (`dossierComplete: false`) tot ze zijn ingevuld. Uitkomsten veranderen niet.

  Het rapport toont de verklaringen bij de labelgegevens. Het EP-Online-gegevensoverzicht zet ze in een apart blok `labelStatements`, omdat het openbare exportschema er geen velden voor heeft.
- **Attestnummer en NL-EPBD-merk (BRL 9501 §8.2 en §8.4).** Het attestnummer wordt op één plaats gelezen (`src/core/nta/Attest.ts`). Alleen met een attestnummer tonen het venster "Over" en het rapport het nummer en het vak voor het NL-EPBD-merk. Het officiële beeldmerk staat niet in de repository. Zonder attestnummer verandert er niets: "niet geattesteerd" en geen merk.
- **Programmabeschrijving.** Nieuw: [globale beschrijving van het rekenprogramma](nta8800-programmabeschrijving.md) voor de attesteringsinstelling (BRL 9501 §3.0).

### 8 oktober 2026 — werkstroom van begin tot eind getest; uitgave en bewijs in de interface

Geen wijziging van uitkomsten of statussen. Wel in de interface:
- De keuzelijst *Uitgave NTA 8800* toont bij een project zonder uitgave de aangewezen uitgave (2025+C1), waarin de kern dan ook rekent, in plaats van "—". De lege keuze is vervallen.
- Een uitkomst in een oudere uitgave (`calculated_legacy_edition`) toont op de stap Project nu een eigen statustekst (voorheen de ruwe sleutel), en het dashboard van Resultaten meldt "Oudere uitgave — niet voor registratie".
- Het veld *massa per m² gebruiksoppervlakte* (alleen 2022 en 2020+A1) bleef onder een latere uitgave zichtbaar; het volgt nu de regel van de andere 2022-velden, met een melding en **Verwijderen**.
- Een bewijsstuk toevoegen of koppelen in *Bron & bewijs* vervangt de tekst van het bronveld niet meer: `evidence:<id>` komt achter de bestaande omschrijving.
- Een element selecteren opent in het contextpaneel het tabblad *Eigenschappen*, ook als *Voorbeeld* openstond.
- Nieuwe test `src/__tests__/workflow-e2e.test.tsx` loopt de werkstroom door in de volledige app, met antwoorden die de echte kern op dezelfde verzoeken gaf. De handleiding is bijgewerkt en heeft een hoofdstuk *Normversies*.

### 8 oktober 2026 — kelder met diepte per wanddeel rekent weer; dekking van alle opties

- Een verwarmde kelder met de diepte per wanddeel (`groundFloors[].heatedBasement.wallDepths`, 8.42/D.12) zonder `depthM` gaf status `invalid` met `non_finite_result`. De lege `depthM` kwam als NaN in de afgeleide invoer. Hij blijft nu weg, net als de lege `reductionFactor` van 7 oktober.
- Dezelfde vorm, nog zonder gevolg: een raam van de gebouwinvoer zonder `gPerpendicular` (g_gl;n uit tabel 7.4) schreef NaN weg. Ook die blijft nu weg.
- 129 meldcodes hadden geen Nederlandse of Engelse tekst en verschenen als kale code. Het gaat vooral om bijlage P, tapwater, de tweede verwarmingssystemen en de rekenzones, bijvoorbeeld `zone_data_missing`, `heating_system_zone_unknown`, `table_p11_no_value` en `hot_water_heat_pump_class_exceeded`. De controle op teksten las elk bronbestand maar tot de eerste testregel (`#[cfg(test)]`). Een testhulp halverwege een bestand verborg zo de code erna, en codes via `ok_or("…")` vond zij niet. Alle codes hebben nu een tekst.
- Nieuwe test `crates/nta8800-core/tests/option_coverage.rs` loopt elke optie van de invoer door, in alle vijf uitgaven:
  - elke variant van elke keuzelijst: 1 203;
  - elk optioneel veld, gevuld: 1 130;
  - elk ingevuld optioneel veld, weggelaten: 299.
  Ze worden niet met de hand bijgehouden; de invoertypen zelf noemen ze. Elke berekening rekent, of weigert met een benoemde code met tekst. Geen enkele valt om of geeft een niet-eindig getal. Een nieuwe variant of een nieuw veld valt er vanzelf onder.

### 8 oktober 2026 — bijlage AA per rekenzone, belemmerings- en bouwjaarcontroles

- **Bijlage AA per gekoelde rekenzone** (2025+C1 p. 1135). Nieuw naast `activeCooling.capacity.calculation`: `zoneCalculations`, één berekening per gekoelde rekenzone met `zoneId`. Elke zone toetst haar eigen ruimten tegen haar eigen ramen. Met één gekoelde zone blijft de enkele berekening zonder `zoneId` werken; bestaande projecten met één zone rekenen hetzelfde. Een project met meer gekoelde zones gaf eerder altijd `annex_aa_window_unknown` en kon bijlage AA dus nooit halen.
  - Nieuwe meldcodes: `annex_aa_zone_id_required`, `annex_aa_zone_unknown`, `annex_aa_zone_duplicate` en `annex_aa_zone_calculation_required`.
  - De rekenkern weigert nu ook een lege of dubbele ruimtenaam (`annex_aa_room_id_required`, `annex_aa_room_id_duplicate`), zoals het formulier al deed.
  - Formulier: bij meer dan één rekenzone een berekening per zone, met alleen de ramen van die zone. Een berekening zonder zone kan aan een zone worden toegewezen. De velden voor SWM en dakoppervlak volgen dezelfde editieregel als de kern.
- **Belemmering met opgegeven factoren zonder bron.** Een belemmering per raam (`windowObstructions`) met methode `declared` en een lege `sourceReference` gaf status `invalid` zonder invoergat. Nu is het een gat `source_reference_required` op `ntaCalculation.windowObstructions[i].obstruction.sourceReference`; het raam rekent met de standaardsituatie. Voor `windowSolar.obstruction` staat het gat op `ntaCalculation.windowSolar.obstruction`.
- **Belemmering voor een bestaand raam.** `window_obstruction_without_window` volgt alleen nog als geen raam die id heeft. Een raam dat niet meer aan buitenlucht grenst, krijgt `window_obstruction_not_outdoor`; een raam met onvolledige gegevens alleen zijn eigen gat `window_data_missing`.
- **Installatiejaar.** De bovengrens is een vaste 2100 in plaats van 2026, zodat een toestel dat na het bouwen van de kern is geïnstalleerd geldig blijft (gasketel en warmtepomp). Het installatiejaar van een warmtepomp en de bron daarvan worden alleen onder 2020+A1 gecontroleerd, waar 9.85 ze gebruikt; onder een andere uitgave maakt een achtergebleven waarde de opwekker niet meer ongeldig.
- **Opname, warmtepomp onder 2020+A1.** De opname geeft een warmtepomp het toesteljaar volgens ISSO 82.1 p. 28 (fabricagejaar, installatiejaar, anders het bouwjaar), zoals bij de gasketel (`manufactureYear`, `installationYear`). Onder 2022 en later blijft de afgeleide invoer gelijk.

### 7 oktober 2026 — openbare gevallen E en F met de belemmeringsmaten uit de uitdraai

Geen wijziging in de kern. De Uniec-uitdraaien van E en F geven per zijbelemmering de hoogte, de afstand en de breedte, in een blok dat bij tekstextractie wegvalt. Met b_b = afstand / breedte (2020 p. 669, 2022 p. 673) staan die nu per raam in `windowObstructions`:
- F (2020+A1): acht ramen met zijbelemmering, twee daarvan met de koelvoorwaarde (≥ 2,5 m). 74,06 / 3,79 / 96,3 tegen gepubliceerd 74,69 / 2,59 / 97,5 (was 73,49 / 3,68 / 96,4 met minimale belemmering).
- E (2022): de vier zijbelemmeringen krijgen hun eigen b_b in plaats van de aangenomen 0,5. 88,61 / 31,68 / 72,6 tegen gepubliceerd 86,82 / 28,89 / 74,1 (was 88,77 / 31,73 / 72,6).

### 7 oktober 2026 — bewijskoppelingen op id

Geen wijziging in uitkomst of status. Een koppeling van een bewijsstuk of opnamefoto (`linkedPaths`) noemt een element nu bij zijn id: `/zones/@woonzone/surfaces/@gevel-noord`, `/basisopname/survey/pv/@pv-1`. Daardoor blijft de koppeling bij het element als een ander element wordt verwijderd of verschoven.
- Een element zonder (unieke) id blijft op positie gekoppeld. Dat zijn de extra opwekkers en extra tapwatersystemen van de opname. Bij verwijderen in de opname schuiven de koppelingen van de onderdelen erna mee, en vervallen de koppelingen naar het verwijderde onderdeel.
- Een project met koppelingen op positie krijgt bij het openen koppelingen op id. De omzetting is idempotent. Een koppeling die niet meer naar een element wijst, blijft ongewijzigd en wordt gemeld.
- `supports` in `manifest.json` geeft de koppelingen in de vorm met id.
- De Checklist BRL 9500 meldt koppelingen naar elementen die niet meer in het project staan.
- Het bewijsregister valt buiten de invoervingerafdruk; de omzetting verandert die dus niet.

### 7 oktober 2026 — zonwering uit tabel 7.5/7.6 rekent weer; openbaar geval F

- Een beweegbare zonwering met een apparaat uit tabel 7.5 of 7.6 (`windowSolar.movableShading.device`, bijvoorbeeld rolluiken of screens) zonder eigen `reductionFactor` gaf status `invalid` met `non_finite_result`. Het lege veld kwam als NaN in de afgeleide invoer. Het veld wordt nu weggelaten. Zulke projecten rekenen nu; de F_c volgt uit de tabel (§7.5). Projecten met een opgegeven `reductionFactor` zijn ongewijzigd.
- Nieuw openbaar vergelijkingsgeval F (vrijstaande woning, Uniec 3.0.19.4, NTA 8800:2020+A1), zie `docs/nta8800-vergelijking-openbare-rapporten.md`.

### 7 oktober 2026 — Belemmering per raam

- Nieuw in `ntaCalculation`: `windowObstructions`, een lijst van `{ windowId, obstruction, sourceReference }`. Een raam in deze lijst krijgt zijn eigen belemmeringssituatie van §17.3 (7.13 neemt F_sh;obst per raam, p. 183–184; 17.3.2 één situatie per raam). Ramen buiten de lijst houden `windowSolar.obstruction`.
- Bestaande projecten rekenen hetzelfde. Een lijst die alleen de standaardsituatie herhaalt, geeft dezelfde getallen; een test bewaakt dat.
- Nieuwe meldcodes: `window_obstruction_duplicate`, `window_obstruction_without_window` en `window_obstruction_reference_required`. Een ongeldige situatie meldt de bestaande codes (bijvoorbeeld `obstruction_geometry_invalid`) op het pad `ntaCalculation.windowObstructions[i].obstruction`; het raam rekent dan met de standaardsituatie.
- Invoer: Gebouw › Schil › **Belemmering per raam**, een tabel met alle buitenramen ("Zoals project" of een eigen situatie). Verwijdert u een raam, dan verdwijnt ook zijn regel.
- Openbaar geval D (2020+A1) heeft nu de overstek op het oostglas per raam: 86,35 / 39,07 / 83,6 tegen gepubliceerd 86,72 / 39,19 / 83,5 (was 82,49 / 37,06 / 83,9). Nieuw is geval E (vrijstaande woning, 2022) met belemmeringen per raam en twee tapwatersystemen: 88,77 / 31,73 / 72,6 tegen 86,82 / 28,89 / 74,1.

### 7 oktober 2026 — Bron & bewijs, opnamefoto's en overnemen in projectmodel

Geen wijziging in uitkomst of status. Wel nieuw in het dossier en de invoer:
- `manifest.json` van de dossier-ZIP heeft een inhoudsopgave van het bewijs (`evidence`). Die geeft per bewijsstuk het pad in de ZIP, de SHA-256 en de invoer die het onderbouwt (`supports`, JSON-pointers uit `linkedPaths` en uit `evidence:<id>`-bronverwijzingen).
- Bronvelden, gebouwelementen en opname-onderdelen krijgen bewijs en foto's uit het bewijsregister. Een opnamefoto is een registeritem met `linkedPaths` `/basisopname/survey/...`; de opname zelf verandert niet.
- **Overnemen in projectmodel** schrijft de uit de opname afgeleide NTA-invoer in het concept. Geometrie, de zonegegevens van een opname met meer zones en aanvullende verwarmingssystemen worden niet overgenomen. De gebruiker past het concept zelf toe.

### 7 oktober 2026 — Bijlage AA als formulier (geen rekenwijziging)

De berekening voor bijlage AA (`activeCooling.capacity.calculation`) vul je nu in een formulier in, onder Installaties › Koeling, in plaats van in de JSON-editor. Het formulier bevat:
- een tabel met de ruimten;
- per ruimte de ramen uit het project;
- dupliceren en verwijderen van ruimten;
- meldingen bij het veld, met de meldcodes van de kern;
- een knop "JSON bekijken" voor wie de JSON zelf wil bewerken.

Het invoerformaat is ongewijzigd en bestaande projecten rekenen hetzelfde. Er zijn drie nieuwe meldcodes, die alleen het formulier gebruikt: `annex_aa_construction_year_required`, `annex_aa_room_id_required` en `annex_aa_room_id_duplicate`.

### 7 oktober 2026 — bouwjaar warmtepomp en uitgave in de desktop-app

- De forfaitaire warmtepomp (`forfait` van `heat_pump_forfait` en `hybrid_heat_pump`) heeft een optioneel `installationYear` met `installationYearReference`. Onder NTA 8800:2020+A1 geeft een bouwjaar vanaf 2015 in (9.85) A = 13,0 kWh in plaats van 87,6 kWh (2020 p. 336). Andere uitgaven: geen verschil. Nieuwe meldingen zijn de bestaande codes `installation_year_invalid`, `installation_year_reference_required` en `installation_year_reference_without_year`.
- Alle NTA-opdrachten van de desktop-app nemen een optioneel `normVersion`, met dezelfde regels als de service (één implementatie in `nta8800_core::norm_versions::request`): onbekend → `invalid_norm_version`; een invoer met een eigen uitgave krijgt de gevraagde uitgave en mag die niet tegenspreken (`norm_version_conflict`); diagnoses rekenen met de uitgave actief; het resultaat krijgt `normVersion`, `targetNormVersion` en in een oudere uitgave `calculated_legacy_edition`. De diagnosepanelen en de constructie-editor sturen de uitgave van het project mee. Resultaten in 2025+C1 zijn ongewijzigd.

### 7 oktober 2026 — openbare rapporten in hun eigen editie

- Tabel 9.28 (hoge-COP-rij van de forfaitaire warmtepomp): onder 2022 en 2020+A1 is de beproeving nu NEN-EN 14511-2:2007, de gedateerde verwijzing van die edities (2022 p. 14, 316; 2020 p. 15, 314). Eerder eiste de kern daar NEN-EN 14511-2:2022, die pas vanaf 2023 geldt (p. 322), en gaf `high_test_standard_invalid` op een correcte beproeving. Projecten onder 2023, 2024 en 2025+C1 zijn ongewijzigd.
- Openbare rapporten in de editie van hun rekendatum: A en een nieuw geval D in 2020+A1, B in 2022 (docs/nta8800-vergelijking-openbare-rapporten.md). D (vrijstaande vakantiewoning, Uniec 3.0.10.0) komt overeen in tapwater, ventilatoren en PV; de hulpenergie van de warmtepomp bevestigt (9.85) met A 13,0 kWh vanaf 2015.

### 6 oktober 2026 — NTA 8800:2020+A1

De uitgave `"2020+A1"` rekent nu (eerder `edition_not_implemented`). Het profiel is cumulatief op 2022, plus de punten 68–79 in docs/nta8800-normversies.md:
- PV (16.4): K_pk per m² naar beneden afgerond op 5 W/m²; de paneelroute (`peakPower.method: panels`) bestaat niet;
- λ_equi;ntr 0,045; (P.25) met deler 4 000; restwarmte met f_P;del 0,1, K_CO2 0,034 en f_Pren 0,9;
- (9.85): A = 13,0 kWh voor toestellen vanaf 2015 en dezelfde forfaitaire set voor warmtepompen (A 87,6, B 0,132, C 0,4, B_nom 24);
- (11.142): f_systype 1,5 voor systeem E1; tabel I.1 met één kolom en detail 14 = 0,70;
- geweigerd met `route_not_in_edition`: tabel P.0 en de kleine-systeemwaarden van bijlage P, β = 0,5 bij onbekende vermogens, kolom B en de standaard-Ψ van 0,5, de werkelijke leidinglengte bij woningen, (13.148a) en (11.106a).

De 2022-invoer (massa per m², wandhoogte, elektroboiler met geïsoleerde leiding, constante-lichtsterkteregeling) heeft nu velden in het formulier en is zichtbaar onder 2022 en 2020+A1. Resultaten onder 2022, 2023, 2024 en 2025+C1 zijn ongewijzigd.

### 6 oktober 2026 — NTA 8800:2022

De uitgave `"2022"` rekent nu (eerder `edition_not_implemented`). Het profiel is cumulatief op 2023, plus de punten 49–67 in docs/nta8800-normversies.md:
- warmtepomptabellen 9.27/9.29 en P.5 tot 55 °C, zonder bronrijen en zonder 25 kW-grens; daarboven bijlage Q; geen bronwarmte Q_HD;hp;in;bron en geen flexmodus;
- biomassa zonder kW-grens, (9.58) zonder f_gebouw;si;H, tabel 9.14 zonder 60/50 en 70/60;
- tabel E.5 minerale-wolvlokken 1,05, λ_equi;ntr 0,06, tabel 7.5 zonder "onbekende kleur";
- nieuwe invoer die alleen onder 2022 geldt: `thermalMass.massKgPerM2` (tabel 7.10 naar massa), `wallHeightAboveGroundM` voor (8.47), `electricBoilerInsulatedPipe` (f_sto;dis;ls 1,5) en `lightingZones[].constantIlluminance` (tabel 14.4);
- zwembadventilatie, η_hr volgens NEN-EN 13053, C_W;mixed air, het getest ventilatorvermogen (NEN-EN 16430) en 13.69a/13.137a bestaan in 2022 niet.

De invoer die tot nu toe "alleen 2023" was, geldt ook in 2022; het formulier toont die velden onder beide edities. Nieuwe codes: `thermal_mass_invalid`, `ground_floor_wall_height_required`. Resultaten onder 2023, 2024 en 2025+C1 zijn ongewijzigd.

### 6 oktober 2026 — NTA 8800:2023 compleet

De uitgave 2023 rekent nu ook de punten die eerder op de 2024-route terugvielen (docs/nta8800-normversies.md, punten 35–48):
- afgifte verwarming volgens tabellen 9.2–9.10 (ook ruimten hoger dan 4 m) en afgifte koeling volgens tabellen 10.2–10.5, met een eigen 2023-invoer die alleen onder 2023 zichtbaar is;
- distributie: L_zi ook voor alleen-verwarmingsleidingen, bedrijfstijd uit tabel 9.15, geen 65 °C-regel voor afleversets, tabel 9.16 zonder gecombineerde rijen, f_H;dis;rbl 0,5 voor leidingen in een ongeïsoleerde schil (nieuwe 2023-invoer);
- f_sh;with van woningen, restwarmte f_Pren, bronwarmte vanaf 20 °C, paneel-U naar bouwjaar (tabellen I.13/I.14, nieuwe 2023-invoer `buildYear`), tabel 13.4 "klein/overig", de NEN 1087-factor 0,5 en (11.77) zonder dak-in-elke-sector;
- lichtkoepels (U_rc) en de serieschakelingen voor tapwater worden onder 2023 geweigerd (`route_not_in_edition`).

Nieuw voor alle edities: `screenUnspecified` op een opening met openingshoek (NEN 1087-doorlaat × 0,3, onder 2023 × 0,5).

Openbare rapporten onder 2023: B 52,96 / 29,10 / 62,5 (was 28,95), C 64,70 / 30,44 / 69,8 (was 30,19). De resultaten onder 2024 en 2025+C1 zijn ongewijzigd.

### 5 oktober 2026 — NTA 8800-editie op elke route

Opgeslagen projecten zonder `ntaCalculation.normVersion` rekenen ongewijzigd in 2025+C1; hun vingerafdruk en label-invoer-hash veranderen niet. Wat verandert voor een project of verzoek in een oudere editie (zie [nta8800-normversies.md](nta8800-normversies.md#uitgave-per-route)):
- **Basisopname** (woning en utiliteit) rekent in de editie van het project (veld `normVersion` van de opname). In een oudere editie: status `calculated_legacy_edition`, `registrationEligible: false` en de waarschuwing `survey_protocol_edition_differs`, omdat het ISSO-opnameprotocol (7e druk, 2025) bij 2025+C1 hoort. Vergelijken mag, registreren niet. Voorheen rekende de opname altijd in 2025+C1.
- **Maatwerkadvies** rekent de basissituatie en alle varianten in de editie van de basis. Een maatregel die de editie wijzigt, is ongeldig (`measure_changes_norm_version`). Alle varianten geldig in een oudere editie: status `calculated_legacy_edition`. Nieuw in de uitkomst: `normVersion` en `registrationEligible`.
- **Herlabelen** vergelijkt in de editie van het oorspronkelijke project (BRL 9500-W §4.2.4 p. 23–24, U p. 19–20). Een andere editie in het huidige project is niet toegestaan. Een herlabeling in de oudere editie van het origineel wordt in de registratiecontrole niet meer geweigerd met `legacy_edition_not_registrable`. Nieuw in de uitkomst: `normVersion`, `targetNormVersion`, `currentNormVersion`.
- **API en MCP:** elke `POST`-bewerking neemt het lid `normVersion` en stempelt de editie op de uitkomst (`normVersion`, `targetNormVersion`). Nieuwe foutcodes: `invalid_norm_version`, `norm_version_conflict`, `norm_version_not_applicable` (400) en `edition_not_implemented` (422). Constructies en diagnoses in een oudere editie krijgen status `calculated_legacy_edition`. Het OpenAPI-document en de MCP-toolschema's beschrijven het lid.
- **App:** de basisopname neemt de editie van het project en toont die in de resultaatkaart (met "niet voor registratie" bij een oudere editie). Het maatwerkadvies toont de editie van alle varianten. De desktopopdracht voor constructies neemt een optionele editie.

### 5 oktober 2026 — NTA 8800:2023 als oudere uitgave

Opgeslagen projecten zonder `normVersion` of met `"2025+C1"` of `"2024"` rekenen ongewijzigd.
- **`normVersion: "2023"`** rekent nu, met status `calculated_legacy_edition` en `targetNormVersion` `NTA 8800:2023`. Zo'n berekening is nooit registreerbaar. Eerder gaf `"2023"` `edition_not_implemented`.
- Het profiel van 2023 is het profiel van 2024 plus zestien eigen schakelpunten, elk met de pagina's in beide uitgaven (docs/nta8800-normversies.md, punten 19–34):
  - tabel 13.2 met de keukenrijen voor ≤ 8 en ≤ 10 mm;
  - zomernachtventilatie: τ_argII, f_argII, f_τ van woningen, θ_e;argII en geen (11.71a);
  - ΔT_fan;
  - Ψ van detail 17;
  - λ van houtvezel en cellulose;
  - deler 8 800 in (P.25) en het vaste rendement 20 van geothermie;
  - biomassagrens 100 kW;
  - geen bijlage AA als koelcapaciteitsbewijs;
  - geen renovatiestandaard.
- **Nieuwe invoer, alleen in 2023:** `hotWater.emission.kitchenPipeDiameter` (`up_to_8_mm`, `up_to_10_mm`, `other`). In 2024 en 2025+C1 geeft een waarde ≤ 8 of ≤ 10 mm `route_not_in_edition`.
- **Ook geweigerd in 2023 (`route_not_in_edition`):** een spuiopening volgens (11.71a), bijlage AA als bewijs voor de koelcapaciteit, roeden (`glazingBars`) en `ledFrom2017: true` bij verlichting.
- **Niet omgeschakeld** (de kern rekent hier met de 2024-route): de oude afgiftetabellen voor verwarming en koeling, tabel 13.4 naar leidingklasse, enkele distributieregels, de bronwarmteroute, restwarmte en een aantal routes van bijlage I en hoofdstuk 8. De lijst staat in docs/nta8800-normversies.md.
- **Openbare gevallen:** C in 2023 met een keukenleiding ≤ 10 mm komt op BENG 2 30,19 (2024: 31,54; rapport 28,70). De versiepost van 1,35 verdwijnt.
- **App:** het veld *Uitgave NTA 8800* biedt nu ook NTA 8800:2023. Het veld voor de binnendiameter van de keukenleiding verschijnt alleen in 2023. De invoer *Collectieve bron gerealiseerd vanaf 2013* geldt in 2023 en 2024 (9.6.8.1.1.2.3, 2023 p. 349).

### 5 oktober 2026 — UI-herontwerp, fase F10: thema's, toegankelijkheid, vertaling en afronding (geen rekenwijziging)

Rekenuitkomsten veranderen niet; opgeslagen projecten openen ongewijzigd. Het UI-herontwerp is hiermee afgerond.
- **Contrast.** Elke tekst haalt in donker, licht en hoog contrast minstens 4,5:1. Invoerranden en statusmarkeringen halen minstens 3:1. Een paar kleuren zijn daarvoor bijgesteld: tekst op het lichte thema iets donkerder, de onverifieerd-kleur en secundaire tekst in het donkere thema iets lichter. Op lichte labelkleuren (A, A+, E) staat de letter nu donker in plaats van wit.
- **Toetsenbord en schermlezer.**
  - Projecttabbladen zijn met Tab bereikbaar.
  - Alle invoervelden van de rekenhulpen hebben een naam.
  - Oudere invoervelden tonen bij focus dezelfde amberkleurige focusrand als de rest van het programma.
  - De koppen op enkele pagina's volgen nu een correcte volgorde.
- **Vertaling.** De vensterknoppen en het sluitkruisje van een tabblad zijn vertaald. De eenheden op het resultatendashboard en de basisopname volgen de taal (`kWh/m²·yr` in het Engels). Talen naast Nederlands en Engels worden pas geladen als je ze kiest.
- **Sluiten met een niet-toegepast concept.** Sluit je een projecttabblad terwijl de NTA-invoer wijzigingen heeft die nog niet zijn toegepast, dan vraagt het programma eerst of je wilt sluiten zonder toe te passen.
- **Installaties › Overzicht** toont per verwarmingssysteem de keten opwekking › distributie › afgifte › regeling. Een groene stip betekent ingevuld; een klik opent dat deel van de NTA-invoer.
- **Basisopname:** de subpagina Rekenzones staat alleen bij een utiliteitsopname in de navigatie en het palet.
- **Recente projecten** tonen de indicatieve labelklasse van de laatste doorrekening bij het opslaan.
- **Sneller starten.** Het 3D-model, de rapportopbouw, het afdrukvoorbeeld en de rekenhulpen laden pas bij eerste gebruik. De startbundel is ongeveer 10 % kleiner.
- **Handleiding:** nieuw hoofdstuk 0 *Werken met het programma* (werkstappen, Ctrl+K, contextpaneel, Controle en Ga naar, Basis/Alle velden, toepasbalk, Gereedschap, uitgavekeuze, sneltoetsen), met schermafdrukken.

### 5 oktober 2026 — UI-herontwerp, fase F9: oplevering, registratie, welkom en instellingen (geen rekenwijziging)

Rekenuitkomsten veranderen niet; opgeslagen projecten openen ongewijzigd.
- **Rapport & dossier** heeft vier subpagina's:
  - *Rekenrapport*: de rapportopbouw (niveau, taal, hoofdstukken, voorbeeld, export en afdrukken);
  - *Invoerdossier*: de export van het NTA-invoerdossier en het invoeroverzicht;
  - *Checklist BRL 9500*: de dossiercheck per groep met de status compleet, ontbreekt, controleren, bezig (kernrun loopt nog) of n.v.t., het bewijsregister, het EP-Online-gegevensoverzicht en de export van het projectdossier (ZIP);
  - *Exports*: BENG-rapport, NTA-rekenrapport, invoerdossier, projectdossier, UNIEC3, VABI en IFC.
- **Registratie** is een eigen pagina. Bovenaan staan de gereedheid met de redenen (dossier niet compleet, geen attest), het rekenprogramma met attest-status en een banner zolang er geen BRL 9501-attest is. Daaronder staan de openstaande punten en de plausibiliteit, elk met Ga naar, en het formulier met berichttype, BAG met grootboekwaarschuwing, opname en adviseurs, WLC-GWP, opnametriggers en bewijs. Opslaan werkt zoals voorheen in de dialoog: `cleanRegistration`, het behoud van de programma-identiteit bij herlabelen en het BAG-grootboek zijn ongewijzigd.
- **Projectgegevens** bevat alleen nog naam, omschrijving, gebruiksfunctie, adres en plaats. Het palet en Ga naar sturen registratievelden naar de stap Registratie.
- **Welkomstscherm:** nieuwe woning of nieuw utiliteitsgebouw, Openen, UNIEC3- en VABI-import, recente projecten (desktop: bestanden die geopend of opgeslagen zijn) en de twee voorbeelden.
- **Instellingen:** de tabbladen Algemeen (thema, taal), Berekening (live voorbeeld per project en de editie voor nieuwe berekeningen) en Over. Kiest de adviseur NTA 8800:2024 als editie, dan begint nieuwe NTA-invoer met `normVersion: "2024"`; bestaande projecten houden hun editie.
### 5 oktober 2026 — UI-herontwerp, fase F8: basisopname, maatwerkadvies en herlabelen (geen rekenwijziging)

Rekenuitkomsten veranderen niet. Opgeslagen opnames, maatwerkadviezen en herlabelvergelijkingen blijven zoals ze zijn.
- **Basisopname** is een wizard met de onderdelen Algemeen, Rekenzones (alleen utiliteit), Thermische schil, Verwarming, Warm tapwater, Ventilatie, Koeling, Zonnestroom en Uitkomst & forfaitair.
  - Links staat de voortgang: per onderdeel het aantal fouten uit de laatste doorrekening.
  - Rechts staat de kaart *Uitkomst opname* (indicatief, los van de projectberekening): label, EP₂, status, fouten en forfaitaire waarden.
  - Elke fout heeft Ga naar, dat het onderdeel opent en het veld focust.
  - De tabel met forfaitaire waarden, bron en inklapreden staat onder Uitkomst & forfaitair.
  - Een opname die de kern niet kan lezen (bijv. een leeg verplicht veld) geeft nu een foutmelding in plaats van een lege uitkomst.
- **Maatwerkadvies** heeft de tabbladen Maatregelen & pakketten, Gemeten verbruik, Woningpas en Advies & rapport.
  - Maatregelen staan op kaarten met sjabloonkeuze, de status compleet of onvolledig (met de open punten), en 1/2/3 voor de pakketdeelname. De volledige editor opent in een zijpaneel.
  - Na het doorrekenen toont het tabblad het labelpad (EP₂ en label per variant, het geadviseerde pakket in amber) en de pakketvergelijking. Sorteren gaat op NCW, terugverdientijd of invoervolgorde.
  - Advies & rapport toont de variantentabel, het advies, het gekozen pakket (label van → naar, zes kerncijfers, fasering) en de rapportexport.
- **Herlabelen** is een stepper: oorspronkelijk bestand → vergelijking → wijzigingen → bewijsrollen → gereedheid.
  - De wijzigingentabel filtert op 6a, 6b en te beoordelen, met leesbare onderdeelnamen en Ga naar.
  - Facturen van vóór de herlabelrollen staan als melding bovenaan.
  - De gereedheid toont de hercontrole van de rekenkern tegen het bewaarde origineel en de openstaande herlabelpunten.
- **Navigatie.** De paden `basisopname.*` en `maatwerkadvies.*` openen nu het juiste onderdeel of tabblad (bijv. `basisopname.envelope…` → Thermische schil, `maatwerkadvies.tariffs…` → Gemeten verbruik).

### 5 oktober 2026 — keuze van de NTA 8800-uitgave

Rekenuitkomsten van bestaande projecten veranderen niet: zonder `ntaCalculation.normVersion` rekent de kern zoals voorheen in NTA 8800:2025+C1:2026, met dezelfde invoervingerafdruk.
- **Nieuw invoerveld** `ntaCalculation.normVersion`: `"2025+C1"` (standaard) of `"2024"`. In de app staat het in het NTA-invoerformulier, blok *Algemeen*.
- **Nieuwe status** `calculated_legacy_edition` voor een berekening in een oudere uitgave. De uitkomst draagt `normVersion` en `registrationEligible: false`, en `targetNormVersion` noemt de gekozen uitgave. De registratiecontrole geeft altijd `legacy_edition_not_registrable`. Het rekenpaneel, het NTA-rekenrapport en de statusbalk melden "niet voor registratie". De API geeft 200, net als bij `calculated_unverified`.
- **Nieuwe codes:**
  - `edition_not_implemented`: 2023, 2022 en 2020+A1 zijn bekend maar niet geïmplementeerd;
  - `route_not_in_edition`: invoer voor een route die de gekozen uitgave niet kent.
- **Indicatoren** die pas in 2025+C1 bestaan, zijn in een oudere uitgave `null`.
- **API:** `GET /v1/version` geeft `supportedNormVersions`.
- **NTA 8800:2024 met INT-V1:2024** is geïmplementeerd met 18 schakelpunten. Daarvan veranderen in de voorbeeldprojecten alleen de CO2-uitkomsten en de 2025-indicatoren. Nieuwe invoer, alleen in 2024: `effectiveMassKgPerM2` en `roofAreaM2` (bijlage AA), en `externalSupply.collectiveHeatPumpSource.realisedFrom2013`. Nieuwe codes: `annex_aa_effective_mass_required` en `annex_aa_effective_mass_invalid`.
- **Uitleg:** [nta8800-normversies.md](nta8800-normversies.md) beschrijft de uitgaven, de schakelpunten (met paginanummers in beide uitgaven), de interpretaties en waarom de uitgave als thread-local wordt gekozen.

### 5 oktober 2026 — f_prac bij gedeclareerde warmtepomprendementen en reconciliatie met openbare rapporten

Uitkomsten veranderen voor projecten met een kwaliteitsverklaring van een warmtepomp:
- **Ruimteverwarming.** Een gedeclareerde COP (`qualityDeclaration` bij de forfaitaire warmtepomp) rekent nu met f_prac 0,95 volgens 9.63 (p. 340), in plaats van 1. Een kwaliteitsverklaring voor verwarming is opgesteld volgens bijlage Q (p. 615). Het warmtepompdeel van de elektriciteit stijgt met 1/0,95 (+5,3 %); de bijverwarming bij een energiefractie onder 1 verandert niet.
- **Warm tapwater.** Een gedeclareerd warmtepomprendement (§13.8.4.7.2) rekent nu met f_prac 0,95 volgens 13.152 (p. 616–617), in plaats van 1,0. Alleen de forfaitaire waarden houden 1,0.
- **Vergelijkingsfixtures.** `training-data/nta8800-public-comparison-{a,b,c}.json` zijn gecorrigeerd naar de rapportinvoer: productwaarden voor de ventilatoren (A: 8,5 W, f 0,147; B: 4 units van 21,7 W, f 0,364), de gedeclareerde hulpenergie van 146 kWh bij B, en geen leidingen buiten de verwarmde of gekoelde zone. Vastgelegde uitkomsten: A 94,00 / 35,15 / 74,9; B 52,96 / 28,84 / 62,6; C 64,70 / 31,54 / 69,0.
- **Documentatie.** De regel-voor-regel-reconciliatie staat in `docs/nta8800-vergelijking-openbare-rapporten.md`. Vragen over 10.15, 10.87 en f_prac staan in `docs/nta8800-vragen-nen.md`.

### 5 oktober 2026 — UI-herontwerp, fase F7: resultatendashboard (geen rekenwijziging)

Rekenuitkomsten veranderen niet. Resultaten toont de kernuitkomst nu als dashboard met de tabbladen Overzicht, Per dienst, Per zone, Maandwaarden en Herkomst.
- **Overzicht:**
  - de labelklasse met schaal;
  - BENG 1/2/3 en TO<sub>juli</sub> als meter tegen de Bbl-eis, met oordeel en marge. TO<sub>juli</sub> staat alleen bij woonfuncties;
  - de maandelijkse energie gestapeld per dienst, per drager of als primair fossiel, met PV en exportcredit onder de nullijn;
  - de netto warmte- en koudebehoefte per maand;
  - aandachtspunten met Ga naar;
  - de kerngetallen (primair fossiel, hernieuwbaar, finaal, CO₂, ZEB).
- **Grafieken.** Elke grafiek heeft een tabelweergave en focusbare maanden met een tooltip. De grafiektotalen zijn gelijk aan de kerntotalen.
- **Herkomst** toont kernversie, normversie, status, attest, vingerafdruk, bronnen van label en Bbl-eisen, en de interpretaties van de kern.
- **Ingehouden en indicatief.** Een ingehouden of indicatieve uitkomst toont het dashboard niet; daar blijft het bestaande gedrag gelden.
- **Paginatitel.** De titel noemt nu de subpagina, bijvoorbeeld Constructies of Per dienst.

### 5 oktober 2026 — UI-herontwerp, fase F6: NTA-invoer in de stappen (geen rekenwijziging)

Rekenuitkomsten veranderen niet. Opgeslagen projecten openen ongewijzigd; het NTA-blok heeft dezelfde vorm.
- **De NTA-invoer staat bij de stap waar hij hoort.** Project: uitgave, rekenomvang en de andere algemene gegevens. Gebouw: gebruiksfuncties, setpoints, massa en interne warmte bij Rekenzones; ramen, dynamische ramen, dakhellingen en vloeren op grond bij Schil & ramen; serres bij Onverwarmde ruimten. Installaties: Verwarming in vijf plus één delen (Opwekking, Distributie, Afgifte, Regeling & BCRG, Hulpenergie, Zonneverwarming), Warm tapwater, Ventilatie, Koeling, Bevochtiging, Verlichting (nieuw, utiliteit), Opwekking (PV, externe levering en bijlage P, opgegeven stromen, opslag) en Gebouwautomatisering. Controle › NTA-invoer houdt de bevestigingen, het volledige formulier en de JSON-editor.
- **Eén concept, één keer toepassen.** Alle stappen bewerken hetzelfde concept; van stap wisselen verliest niets. De balk onderaan toont het aantal wijzigingen, Ongedaan maken, Vorige stap en Toepassen en verder. Toepassen doet wat Opslaan in het formulier deed.
- **Basis / Alle velden.** Minder gebruikte secties staan in Basis achter "Geavanceerd" en klappen vanzelf open bij invoer, een kernmelding of Ga naar.
- **Bron & bewijs per sectie** met de bronvelden van die sectie, ingevuld of open, en Ga naar.
- **Getallen** accepteren een komma of een punt en tonen de waarde in de taal van de app.
- **Controle in het contextpaneel**: fouten, aandachtspunten en weggelaten correcties, de punten van de open pagina eerst, met Ga naar naar het veld. Een veld met een kernmelding heeft een rode of gele rand.
- **Ga naar** opent nu de juiste stap, subpagina, het verwarmingsdeel en zo nodig het ingeklapte blok, en zet de focus in het veld. Een ongeldig NTA-blok (`nta_calculation_block_invalid`) wijst naar het veld uit de kernmelding, bijvoorbeeld `ntaCalculation.setpoints.heatingC`. Daardoor tellen NTA-meldingen in de navigatie bij Gebouw of Installaties in plaats van bij Controle.
- **Alleen bij de editie 2024** staan de velden effectieve massa en dakoppervlak per vertrek (bijlage AA) en "collectieve bron gerealiseerd vanaf 2013" (bijlage P). Bij een andere editie meldt de app achtergebleven waarden en biedt Verwijderen aan.

### 5 oktober 2026 — UI-herontwerp, fase F5: Gebouw en Installaties per onderdeel (geen rekenwijziging)

Rekenuitkomsten veranderen niet. Opgeslagen projecten openen ongewijzigd; ids van zones, vlakken, ramen en systemen blijven gelijk.
- **Gebouw** heeft subpagina's: Schil & ramen, Rekenzones, Constructies, Koudebruggen, Luchtdichtheid, Onverwarmde ruimten en 3D-model.
  - Schil & ramen is één tabel met de ramen als onderliggende regels. Filteren kan op type (alle/gevels/daken/vloeren) en op naam of constructie. Groeperen kan op oriëntatie, type of constructie. De kolommen zijn grenst aan, helling (uit de NTA-invoer, anders 90° voor wanden en 0° voor vloeren), bruto A, constructie, U en het aantal ramen.
  - Onder de tabel staan het indicatieve aandeel in H<sub>T</sub> per elementtype (U·A en ψ·l) en de controles van de schil: thermische begrenzing, vlakken zonder constructie, forfaitaire koudebruggen en de kernmeldingen van de stap met Ga naar.
  - De luchtdoorlatendheid q<sub>v;10</sub> is direct op de pagina te wijzigen.
- **Contextpaneel bij een geselecteerd element.** Het paneel bewerkt een vlak, raam, rekenzone of koudebrug direct met eenheidsvelden. Voor een vlak zijn dat bruto oppervlak, oriëntatie, toegewezen constructie en thermische grens. Netto oppervlak, helling, U, R<sub>c</sub> en H<sub>T</sub> zijn afgeleid. Het paneel toont ook de ramen in het vlak en linkt naar de U-waardecalculator. Een leeg verplicht veld wordt niet weggeschreven.
- **Editors als zijlade.** Bewerken en toevoegen openen de bestaande editors rechts als zijlade in plaats van als venster. Velden, controles en opslaan blijven gelijk.
- **Installaties** heeft subpagina's:
  - Overzicht;
  - Verwarming, Warm tapwater, Ventilatie, Koeling, Bevochtiging, Opwekking (PV), Warmtepompen en Gebouwautomatisering.
  - Het overzicht toont per aanwezige dienst een ketenkaart met de systemen en hun kerngetallen, de ingevulde NTA-blokken en de status uit de kernmeldingen. Afwezige diensten staan samen in één gestippelde kaart. Zonder selectie toont het contextpaneel de energie per dienst uit de laatste kernberekening.
  - Elke dienstpagina toont de systemen van het projectmodel in een tabel, de open punten van die dienst en welke NTA-invoerblokken ingevuld zijn. Een link opent ze in Controle › NTA-invoer; de NTA-invoer zelf wordt in F6 opgesplitst.
  - De gaswarmtepomp-referentie staat als inklapbare diagnose onder Warmtepompen.
- **Ga naar** kiest nu de juiste subpagina. Voorbeelden: koudebruggen bij Koudebruggen, `constructions` bij Constructies, `solarPV` bij Opwekking (PV).
- **3D-model.** Een lege of niet-tekenbare geometrie toont een leeg- of foutstatus. De vlaknamen zonder eigen naam zijn vertaald.

### 5 oktober 2026 — UI-herontwerp, fase F3 en F4: werkstroom in plaats van ribbon (geen rekenwijziging)

Rekenuitkomsten veranderen niet. Opgeslagen projecten openen ongewijzigd.
- **Geen ribbon meer.** Links staat een genummerde werkstroom: Project → Gebouw → Installaties → Controle → Resultaten → Basisopname → Maatwerkadvies → Herlabelen → Rapport & dossier → Registratie. Elke stap toont zijn status uit de kerncontrole: compleet, aantal fouten, aantal aandachtspunten of nog te doen. Bij nieuwbouw (oplevering of Bbl-toets) zijn de stappen voor bestaande bouw gedimd.
- **Alle ribbonacties blijven bereikbaar:**
  - toevoegen bij de stap waar ze horen (Gebouw, Installaties);
  - exports bij Rapport & dossier;
  - importeren op het projectoverzicht;
  - de rekenhulpen, UNIEC3/VABI en IFC 3D in het menu Gereedschap onderaan de navigatie;
  - Nieuw/Openen/Opslaan als in het menu Bestand;
  - alles ook in het opdrachtpalet (Ctrl K).
- **Bovenbalk.** Documenttabbladen, zoeken (Ctrl K), Bestand, Opslaan, Herberekenen (Ctrl ↵) en het contextpaneel (Ctrl .).
- **Contextpaneel.** Eén paneel rechts vervangt Eigenschappen en Live preview. Het live voorbeeld staat ook in Instellingen (Ctrl ,).
- **Statusbalk.** Toont of het resultaat actueel of verouderd is, de kernversie, BENG 1–3, TO<sub>juli</sub>, het indicatieve label en "onverifieerd · geen attest".
- **Eén kernberekening per document.** Navigatie, statusbalk, resultaten, rapport en afdrukvoorbeeld lezen dezelfde uitkomst, zodat "verouderd" en "bezig" overal gelijk zijn.
- **Ga naar.** Een kernmelding met pad opent de juiste stap, selecteert het element en zet de focus op de rij.
- **Sneltoetsen.** Alt ↑/↓ wisselt van stap, Alt ←/→ van subpagina. Ctrl N/O/S/Shift S/W werken zoals voorheen. De URL-hash volgt de stap (`#/gebouw`).
- **Toegankelijkheid.** Een skiplink "Naar inhoud" en de landmarks banner, navigatie, main, contextpaneel en statusbalk. De actieve stap heeft `aria-current="page"`. Na navigatie gaat de focus naar de paginatitel.
- **Rapport.**
  - De inhoudsopgave en de hoofdstuktitels tonen subscripts (H<sub>D</sub>, H<sub>g</sub>, H<sub>U</sub> en H<sub>tr</sub>) in plaats van liggende streepjes.
  - De noot onder tabel 3 noemt de gebruiksfunctie in het Nederlands, bijvoorbeeld "woonfunctie (niet in een woongebouw)", in plaats van `other_residential`.

### 5 oktober 2026 — HTTP-API en MCP-server

Rekenuitkomsten veranderen niet. Wat wel verandert voor clients van de API of de MCP-server:
- **Eén register.** Alle bewerkingen staan in `crates/nta8800-service/src/operations.rs`. Elke bewerking is een HTTP-route én een MCP-tool met dezelfde naam (45 in totaal) en staat in het OpenAPI 3.1-document (`GET /v1/openapi.json`).
- **Nieuwe routes en tools:**
  - `GET /v1/version` / `get_version`;
  - `list_interpretations` (de route bestond al);
  - `assess_maatwerkadvies` en `assess_relabel` als MCP-tool (de routes bestonden al);
  - `POST /v1/nta8800/project/energy-by-service` / `get_energy_by_service`;
  - `POST /v1/nta8800/label/data` / `get_label_data`;
  - `POST /v1/nta8800/registration/assess` / `assess_registration`;
  - `POST /v1/nta8800/relabel/label-input-hash` / `get_label_input_hash`.
- **Foutmodel.** Fouten van client en server hebben één envelop: `error`, `code`, `message`, `path` en `details`. `error` en `message` blijven bestaan, zodat bestaande clients blijven werken.
  - **Nieuw:** een verkeerde invoervorm gaf eerder 422 met platte tekst van axum. Nu is dat 400 met code `invalid_request_shape` en het JSON-pad van de fout.
  - **Nieuwe codes:**
    - ongeldige JSON geeft 400 `invalid_json`;
    - een ontbrekend invoerlid geeft 400 `missing_request_member`;
    - een verkeerde content-type geeft 415;
    - een te grote body geeft 413;
    - onbekende routes en methoden geven 404/405.
  - **Ongewijzigd:** een weigering van de kern blijft 422 met de beoordeling als body.
- **Server-opties:**
  - `--bind`, `--port`, `--cors-origin`, `--body-limit-mb` en `--log`/`--no-log`, ook als omgevingsvariabelen `OES_API_*`;
  - netjes stoppen bij SIGINT/SIGTERM;
  - één logregel per verzoek;
  - standaard alleen op loopback.
- **MCP-server:**
  - **Resources:** de voorbeeldprojecten, de opnamefixtures, de interpretatielijst, de handleiding en het OpenAPI-document.
  - **Tool-uitkomsten:** dezelfde JSON-body als de route, ook als `structuredContent`.
  - **`isError`:** dit staat waar de route 4xx/5xx geeft. De oude tools gaven bij een vormfout eigen codes (`invalid_*_shape`); die zijn nu `invalid_request_shape` met pad, behalve `invalid_project_shape` en `invalid_maatwerkadvies_shape`.
- **Documentatie:** [nta8800-api.md](nta8800-api.md), [nta8800-mcp.md](nta8800-mcp.md) en hoofdstuk 9 van de handleiding.
### 5 oktober 2026 — woningopname na de vergelijking met de RVO-voorbeeldwoningen

- **Bouwjaar naar de rekenkern.** De woning- en utiliteitsopname geven het bouwjaar nu door. Opnames krijgen daardoor de Standaard voor woningisolatie (§5.3.2) en de waarschuwing `standard_insulation_construction_year_missing` verdwijnt. De labelgegevens krijgen het bouwjaar ook. Andere uitkomsten veranderen niet.
- **Warmtepomp zonder vermogen.** Een individuele warmtepomp zonder opgegeven vermogen rekent nu met NTA-tabel 9.27 (woningen tot en met 25 kW), vastgelegd als `heat_pump_capacity_unknown_table_9_27` (ISSO 82.1 tabel 9.6, p. 110). Zo'n opname liep eerder vast. Een collectieve warmtepomp zonder vermogen geeft `heat_pump_capacity_required` op `heating.generator.capacityKw`.
- **Geweigerde opnames tonen de reden.** Weigert de kern de afgeleide invoer, dan staan de meldingen van de kern bij de opname, onder `derivedInput.…`. Een weigering zonder melding geeft `derived_input_rejected_without_reason`.
- **Bronvermelding koudebrugtoeslag.** ΔU_for verwijst naar NTA 8800 §8.2.1, formule 8.3.
- **Vergelijkingstest.** Zes RVO-voorbeeldwoningen staan als fixtures in `training-data/nta8800-rvo-voorbeeldwoningen-*.json`; zie `docs/nta8800-vergelijking-rvo-voorbeeldwoningen.md`. Dit is geen officiële referentietoets.
### 5 oktober 2026 — Rapportage Energieprestatie met detailniveaus

- **Nieuw rapport:** "Rapportage Energieprestatie (NTA 8800)" op het tabblad Rapport, met de niveaus samenvatting, standaard en gedetailleerd. Bij gedetailleerd kies je per hoofdstuk welke berekeningen tot op maandniveau worden getoond, met formulenummers en uitgewerkte rekenstappen. Export als HTML (desktop: opslagvenster) en afdrukken of opslaan als PDF, opgemaakt voor A4.
- **Kernuitvoer uitgebreid (alleen extra velden):** elke maand van de warmtebehoefte bevat nu `windowSolarByWindow` (zonnewinst per raam, verwarming en koeling; samen gelijk aan `windowSolarGainsKwh`) en `sunroomGainsKwh`/`sunroomCoolingGainsKwh` (7.30b). Bestaande velden en uitkomsten veranderen niet.
- **Testfixtures:** de kernuitvoer van beide voorbeeldprojecten staat in `training-data/nta8800-example-*.kernel-output.json`; de rapporttests gebruiken die.
### 5 oktober 2026 — UI-herontwerp, fase F1 en F2 (geen rekenwijziging)

Eerste fasen van het UI-herontwerp (`docs/ui-redesign/ontwerp.md`). Rekenkern, invoer en uitkomsten zijn ongewijzigd.
- **Huisstijl en leesbaarheid:** nieuwe designtokens; Inter, Space Grotesk en JetBrains Mono worden met de app meegeleverd, dus ook offline in de desktopversie. Gedempte tekst haalt nu WCAG AA (4,7 : 1 donker, 5,0 : 1 licht). De statusbalk is neutraal in plaats van amber; de scrollbalk volgt het thema; "Onverifieerd" heeft een eigen violette kleur in plaats van de waarschuwingskleur.
- **Meldingen:** foutmeldingen bij openen en importeren verschijnen als melding rechtsonder in plaats van een browservenster. Sluiten van een tabblad met niet-opgeslagen wijzigingen vraagt in de app-taal "Opslaan · Niet opslaan · Annuleren", ook in de browserversie.
- **Taal:** de feedbackknop en het feedbackformulier zijn Nederlands; bestandskeuzes tonen "Bestand kiezen…" in plaats van de Engelse browsertekst; de teksten van het 3D-model ontbraken en zijn toegevoegd.
- **Bouwstenen:** knoppen, invoervelden met eenheid (komma en punt worden beide geaccepteerd), keuzeknoppen, kaarten, tabellen, statuslabels, dialogen en zijpanelen voor de volgende fasen.
### 5 oktober 2026 — kwaliteitsverklaring warmtepomp, woning in meerdere zones, vergelijking met openbare rapporten

Aanleiding: drie openbare BENG-rapporten zijn nagebouwd ([vergelijking](nta8800-vergelijking-openbare-rapporten.md)).

#### Nieuwe invoer

- **Kwaliteitsverklaring van een warmtepomp voor ruimteverwarming (§9.1, p. 285).** De forfaitaire warmtepomp neemt nu een optionele `forfait.qualityDeclaration` aan:
  - `declarationReference`: het nummer en de uitgever van de verklaring;
  - `generationEfficiency`: de COP. Die vervangt de waarde uit tabel 9.27/9.29 en wordt naar beneden afgerond op 0,05. c_source blijft gelden.
  - `energyFraction`: bij een waarde onder 1 levert de geïntegreerde elektrische bijverwarming de rest, met rendement 1.
  - `auxiliaryKwhPerYear`: vervangt het forfait van 9.85.

  Een fractie onder 1 in een set van meerdere opwekkers geeft `heat_pump_declared_fraction_in_multiple_set`.
- **Gedeclareerd tapwaterrendement van een warmtepomp (§13.8.4.7.2, p. 640).** `declared` bij een tapwaterwarmtepomp vervangt 1,4·c_source en wordt afgerond op 0,05. Zonder toepassingsklasse geldt de waarde zoals in de verklaring geïnterpoleerd; met een klasse wordt de correctie c_W;gen toegepast.
- Beide hebben een eigen sectie in het NTA-formulier.

#### Resultaten die veranderen

- **Woning of woongebouw in meerdere rekenzones (6.2b, p. 160).** Per zone geldt nu N_woon;zi = A_g;zi / Σ A_g;zi × N_woon. Dat raakt de interne warmtewinst (7.21–7.24) en BENG 1.
  - Voorheen telde elke zone het volledige `dwellingCount`, zodat bewoners per zone dubbel meetelden.
  - Een eigen aandeel kan worden opgegeven met `internalGains.dwellingShare`.
  - Projecten met één zone veranderen niet.

#### Nieuwe melding

- `cooling_emission_loss_exceeds_need` (niet-blokkerend): het koudeafgifteverlies van formule 10.15 is groter dan de koudebehoefte in ten minste één maand.

#### Vastgelegd als interpretatie

- **Regelenergie koeling (10.87, p. 425).** 0,010 kW, altijd in bedrijf, ook bij een omkeerbare warmtepomp: 87,6 kWh per jaar.

### 5 oktober 2026 — aansluiting op de definitieve BRL-versies van 29 mei 2026

De applicatie was gebouwd tegen de concepten van BRL 9500-W/U en BRL 9501 van 14-10-2025 en tegen BRL 9500-MWA-W/U van 19-06-2024. Inmiddels zijn de aangewezen versies openbaar.
- **BRL 9500-W/U van 29-05-2026:** tekst en paginering zijn gelijk aan de concepten; alleen de datum en de verwijzing naar NTA 8800:2025+C1:2026 verschillen. De registratie- en herlabelregels blijven ongewijzigd; alleen de bronvermelding is bijgewerkt.
- **BRL 9501 van 29-05-2026:** tekst en paginering zijn gelijk aan het concept, op de datum, de NTA-verwijzing en de deelgebieden na. Het derde deelgebied, financiële kengetallen, wordt alleen nog beoogd (§2.1, p. 2). De testset is ISSO-publicatie 54 versie 5.0:2026 (§11, p. 18).
- **BRL 9500-MWA-W/U van 24-03-2026** (in werking per 29-05-2026): voegt het renovatiepaspoort toe (§3.2, p. 14) en de registratie als maatwerkadvies met of zonder paspoort (§4.2.8, p. 23).

#### Resultaten die veranderen

- **Renovatiepaspoort utiliteit:** het paspoort volgt BRL 9500-MWA-U §3.2. De woningeisen (Standaard voor Woningisolatie, aardgasvrij, opslag) gelden niet meer voor utiliteitsgebouwen. Daarvoor in de plaats komen: isolatie geschikt voor LTV en HTK (`lowTemperatureReady`), motivatie bij onmogelijke gevelisolatie, maatregelen tegen de koelvraag (`coolingMeasureIds`) en een EP2 van stap 3 op of onder de renovatiestandaard (`renovation_standard_ep2`). Een bewaard utiliteitspaspoort krijgt daardoor andere eisen en vaak het oordeel "onbeslist" tot de nieuwe velden zijn ingevuld.
- **Renovatiepaspoort woningen, stap 1:** zonder verklaring van de adviseur vergelijkt de kern nu zelf de warmtebehoefte van stap 1 met de Standaard voor Woningisolatie (NTA 8800 §5.3.2). Eerder bleef de eis dan onbeslist.
- **Renovatiepaspoort woningen, stap 2:** aardgasvrij geldt "waar realistisch mogelijk". Een motivatie (`gasFreeNotRealisticMotivation`) voldoet aan stap 2.

#### Nieuw

- **Registratietype maatwerkadvies:** `registrationType` in de uitvoer van het maatwerkadvies: `maatwerkadvies`, of `maatwerkadvies_met_renovatiepaspoort` als het paspoort aan alle eisen voldoet. Paneel en adviesrapport tonen het.
- **Ingelezen gegevens:** BRL 9501 §4.3.1, opmerking (p. 8), vraagt te vermelden of, en met welk hulpmiddel, de adviseur gegevens heeft ingelezen.
  - De UNIEC3- en VABI-import schrijven een regel in `importLog` van het project.
  - De registratiecontrole geeft `dataImport` (`dataImported`, `tools`), en het rekenrapport toont de regel *Gegevens ingelezen*.
  - Een regel zonder hulpmiddel geeft `import_log_invalid`.
  - `importLog` telt niet mee in de labelinvoer voor herlabelen, in de kern en in de app.
- **Versiebeleid:** het versiebeleid volgens BRL 9501 §4.3 (p. 7) staat in [`docs/nta8800-versiebeheer.md`](nta8800-versiebeheer.md). Het omvat de registratie van de rekenkernversie bij RVO (§5.2) en het bewaren van een oude rekenversie minstens drie jaar na een normwijziging (§5.3, p. 9).
### 5 oktober 2026 — Omgevingsregeling als bron, labelelementen art. 5.13a, EP-Online-overzicht

Geen rekenwijziging in de kern.

- **Bronverwijzingen.** De Regeling en het Besluit energieprestatie gebouwen zijn per 1 januari 2024 ingetrokken. De code, de meldingen en de documentatie verwijzen nu naar de opvolgers: Omgevingsregeling art. 5.11–5.14 en bijlagen IX–Xa (BWBR0045528, versie 2026-10-01) en Bbl art. 6.29 lid 4 voor de geldigheid van tien jaar. De klassegrenzen (bijlagen IX en X), de A0-waarden (bijlagen IXa en Xa met de voorwaarden van lid 5), het forfaitaire scenario voor woningen (art. 5.11 lid 4) en de kwaliteitsverklaringen voor utiliteit (art. 5.12 lid 4) zijn nagelopen tegen die versie en kloppen.
- **Labelelementen (art. 5.13a lid 1, sinds 29 mei 2026).** `labelData.indicators.elements` geeft de elementen die de berekening kan leveren: operationele CO₂ per m², WLC-GWP uit de registratie, finaal energiegebruik per m², het jaarlijkse primair fossiele, hernieuwbare primaire en finale energiegebruik, de hernieuwbare productie en de belangrijkste energiedrager en hernieuwbare bron. Reageren op externe signalen en een afgiftesysteem voor lage temperaturen zijn verklaringen en blijven leeg.
- **EP-Online-gegevensoverzicht.** Het projectdossier bevat `ep-online-gegevensoverzicht.json` met de veldnamen van het openbare exportschema `EpbdExportTypesV4`. Het is een controleoverzicht, geen registratiebestand: het uploadformaat van EP-Online is niet openbaar.

### 4 oktober 2026 — achtergehouden resultaten overal, volledige codelijst, luchtdichtheid bij openen

Geen rekenwijziging in de kern.

- **Eén regel voor vereenvoudigde getallen** (`src/core/nta/KernelVerdict.ts`). Keurt de NTA-kern de invoer af (ongeldig of onvolledig), dan tonen het BENG-rapport (tabblad, HTML-export, afdrukken en afdrukvoorbeeld), de IFC-export en het resultatenoverzicht geen vereenvoudigde BENG-waarden meer. Het rapport meldt "BENG-resultaten achtergehouden"; de IFC-export weigert met dezelfde melding. Alleen zonder NTA-invoer of zonder antwoord van de kern blijft de indicatieve schatting zichtbaar.
- **IFC met kernwaarden.** Heeft de kern gerekend, dan bevat de IFC-export de kernwaarden met de Bbl-toets (MEETS / DOES_NOT_MEET / NOT_TESTABLE) in plaats van de vereenvoudigde waarden.
- **Geen flits van vereenvoudigde getallen.** Na een afgekeurde berekening blijft het resultatenoverzicht "achtergehouden" tonen terwijl de volgende berekening loopt.
- **Codelijst compleet.** De test die controleert of elke kerncode een Nederlandse en Engelse tekst heeft, vindt nu ook codes uit hulpfuncties (`push`/`add`), meerregelige `issue(…)`-aanroepen, tupels en `unwrap_or`. Daarmee zijn 148 extra codes gevonden en vertaald (o.a. bijlage M/V/W, micro-WKK, koeling, PV, zonneboiler, verlichting, zone-indeling). Twee teksten zijn gecorrigeerd: bijlage Q verdamperintrede (1 waarde of 27 waarden) en ontwerpaanvoertemperatuur (groter dan 0 en hoogstens 75 °C).
- **Luchtdichtheid bij openen.** Een bestand zonder luchtdichtheid krijgt nu q_v10 = 0,4 dm³/(s·m²) (de importstandaard), gemarkeerd als aangenomen, in plaats van 0 (volkomen luchtdicht). De gebouwschilweergave vraagt de waarde te controleren.
- **Details.** Getallen in technische details tonen hoogstens 2 decimalen; ook de details bij afgekeurde invoer zijn vertaald.
- **Editors.** Opslaan zonder wijziging voegt geen lege velden meer toe aan vlakken en installaties.

### 4 oktober 2026 — vertaalde meldingen en rekenhulpen

Geen rekenwijziging; alleen de weergave verandert.

- **Elke kernmelding vertaald.** Alle 608 meldingscodes van de rekenkern en de basisopname die nog als ruwe code verschenen, hebben nu een Nederlandse en Engelse tekst die zegt wat er mis is en wat te doen, met de normparagraaf waar dat helpt. Ze staan in `src/i18n/kernelCodeLabels.ts`. Een code zonder eigen tekst op de plek waar hij verschijnt, valt terug op deze algemene tekst.
- **Bewaking.** De test `kernel-code-labels.test.ts` haalt alle meldingscodes uit de Rust-bronnen en faalt zodra een code geen Nederlandse of Engelse tekst heeft. Een nieuwe code kan dus niet onvertaald worden uitgeleverd.
- **Technische details.** De Engelse toelichting die de kern bij sommige meldingen meegeeft (zoals "7.3.3: zone list [] …" of "A_ls/A_g 200001.4"), wordt voor de bekende vormen vertaald, met getallen in de notatie van de gekozen taal. Een onbekende toelichting blijft zichtbaar, gemarkeerd als technisch detail. Dat geldt in het paneel, de invoercontrole, het maatwerkadvies en de Nederlandse rapporten.
- **Rekenhulpen.** De U-waardecalculator en de koudebrugcalculator toonden ruwe sleutels (`uvalue.*`, `tb.*`); ze hebben nu Nederlandse en Engelse teksten en tonen getallen in de notatie van de gekozen taal.
### 4 oktober 2026 — UI-doorloop 4: geen gegevensverlies bij bewerken, achtergehouden resultaten, openen

- **Constructie bewerken verliest niets meer.** Een constructie zonder lagen (zoals alle constructies in beide voorbeelden) kreeg bij openen Rc 0 en U 5,882, en opslaan schreef die waarden weg, ook na alleen hernoemen. De bewerker houdt nu de opgeslagen Rc, U en R_se-basis vast tot u een laag wijzigt of een kernresultaat toepast. Openen en opslaan zonder wijziging laat het project ongewijzigd; dat geldt nu ook aantoonbaar voor de zone-, vlak-, raam-, verwarmings-, ventilatie-, tapwater-, koel-, PV- en zonthermiebewerkers. De verwarmings- en tapwaterbewerker gooiden bij opslaan opgeslagen warmtepompconcepten (onder meer `forfaitHeatPumpDraft`) weg; die blijven nu staan. Een percentageveld (dekking, WTW-rendement, zonnefractie) wordt alleen teruggerekend als u het wijzigt.
- **Resultaten achtergehouden.** Keurt de kern de invoer af (`invalid`) of is de NTA-invoer onvolledig, dan toont het resultatentabblad geen vereenvoudigde BENG-waarden meer maar de melding "Resultaten achtergehouden". Alleen een project zonder enige NTA-invoer (`nta_calculation_block_missing`) toont nog de indicatieve schatting. Het NTA-paneel toont bij `invalid` nu ook de blokkerende gaten.
- **Project openen.** Een projectbestand waarin lijsten ontbreken (zoals `thermalBridges` of `windows`) wordt bij openen aangevuld met lege lijsten; een volledig bestand blijft ongewijzigd. Een fout in één weergave maakt de applicatie niet meer leeg: die weergave toont een melding met "Opnieuw proberen". In de browserversie opent "Openen" nu een bestandskiezer.
- **Verwijderen.** De bevestiging noemt de NTA-invoer die meegaat met naam (zone, vlak, raam, systeem) in plaats van JSON-paden, en zegt hoeveel vlakken en ramen met een zone of vlak meegaan.
- **Maatregelsjablonen** noemen het ontbrekende veld ("Ontbreekt: rendement warmteterugwinning").
- **Kleinere punten.** Het projectoverzicht telt kWp uit de NTA-PV-invoer; de GTO-grens volgt de taalinstelling; formule- en tabelnummers in opnamewaarden blijven "(11.109)"; bronnen van opnamestandaarden en de herlabelbron zijn in het Nederlands; "geen variant geadviseerd" als niets is geadviseerd; de U-waardecalculator past op 1280 px.

### 4 oktober 2026 — robuustheid: vangnet op alle uitvoerroutes en bijgestelde grenzen

- **Vangnet op elke kernuitvoer.** De HTTP-dienst en de desktopapp controleren nu elk kernresultaat op NaN en oneindig voordat het wordt geserialiseerd. Zo'n resultaat wordt niet uitgegeven: de dienst antwoordt met HTTP 500 en `{"error": "non_finite_result", "path": …}`, de desktopapp met de fout `non_finite_result: <pad>`. Dat geldt ook voor de diagnostische routes (H7, H9, H11, de `*_draft`-modules) die het projectvangnet niet hadden.
- **Maatwerkadvies.** Een maatwerkadvies met een niet-eindig getal krijgt status `invalid` met `non_finite_result`; de varianten worden achtergehouden. Een gebouwinvoer als basis, en elke gebouwinvoer na maatregelen op het gebouw, wordt nu gecontroleerd op:
  - gebruiksoppervlakte boven 10.000.000 m² (`area_out_of_range`);
  - opgegeven maandgebruik boven 1.000.000 kWh per m² per maand (`declared_use_out_of_range`);
  - elk getal groter dan 10¹² (`value_out_of_range`).

  Zo'n variant wordt niet berekend. Projectmaatregelen liepen al via de projectcontrole.
- **Waarschuwingsgrenzen verhoogd.** De waarschuwing voor een vlak of raam gaat nu af boven 500.000 m² (was 100.000 m²): grote distributiecentra en terminals hebben daken van meer dan 100.000 m². De waarschuwing voor opgegeven maandgebruik gaat af boven 5000 kWh per m² per maand (was 1000): een datacentrum haalt ruim 1000. De blokkerende grenzen zijn ongewijzigd.
- **Teksten.** Een gebouwhoogte boven 1000 m heeft nu een eigen melding. De meldingen over levensduur en opgegeven gebruik noemen de grens.

### 4 oktober 2026 — robuustheid: aantallen, niet-eindige getallen en bereikgrenzen

Een robuustheidstest met ongeveer 49.000 verstoorde invoeren vond geen paniek, maar wel één crash, twee niet-eindige uitkomsten en uitkomsten die als berekend werden gemeld bij onzinnige invoer. De grenzen hieronder zijn keuzes van het programma, geen normwaarden.

#### Opnames en projecten die nu `invalid` worden

- **Onbegrensde aantallen in de opname.** Een opname met `heating.storeys` = 4.294.967.295 en zonder verticale leidingen probeerde 51,5 GB geheugen te reserveren en brak het proces af. Nu geldt:
  - bouwlagen (`storeys`, `heating.storeys`, `heating.collective.connectedStoreys`) hoogstens 200, anders `storeys_out_of_range`;
  - andere aantallen hoogstens 100.000, anders `count_out_of_range`. Dat geldt voor woningen op een collectieve installatie, douches, collectoren, afgiftetoestellen op een eenpijpssysteem, luchtverwarmers, ventilatorconvectoren, `toiletStacks` en gedeelde zones van verticale leidingen;
  - oppervlakten (A_g, functies, vlakken, ramen) hoogstens 10.000.000 m², anders `area_out_of_range`; gebouwhoogte hoogstens 1000 m.
- **Kern.** Verticale leidingen hoogstens 200 bouwlagen of 1000 m gebouwhoogte (`vertical_pipe_storeys_invalid`). Bouwlagen van tapwatercirculatie en koeldistributie hoogstens 200. Het aantal woningen hoogstens 100.000 (`dwelling_count_invalid`). De levensduur van een maatregel in het maatwerkadvies hoogstens 100 jaar (`measure_lifetime_invalid`), omdat de standaardhorizon anders de netto contante waarde eindeloos laat doorrekenen.
- **Projectinvoer buiten elk denkbaar gebouw** blokkeert de berekening; de uitkomsten worden achtergehouden:
  - gebruiksoppervlakte van een zone boven 10.000.000 m² (`zone_area_out_of_range`);
  - oppervlakte van een vlak of raam boven 10.000.000 m² (`surface_area_out_of_range`);
  - U-waarde boven 100 W/m²K (`u_value_out_of_range`);
  - q_v10 boven 1000 dm³/s·m² (`qv10_out_of_range`);
  - opgegeven maandgebruik boven 1.000.000 kWh per m² per maand (`declared_use_out_of_range`).
- **Vangnet voor niet-eindige getallen.** Bevat een project- of opnameresultaat toch NaN of oneindig, dan wordt het niet uitgegeven (serde_json zou er `null` van maken). De status wordt `invalid` met `non_finite_result` en het pad van het eerste getal. `geometry.lossAreaRatio` en de gemiddelde U van de labelgegevens zijn nu alleen nog eindig.

#### Nieuwe waarschuwingen (de berekening loopt door)

- Zone-A_g boven 1.000.000 m² of onder 1 m² (`zone_area_out_of_range`, `zone_area_implausible`).
- Vlak of raam boven 500.000 m² (`surface_area_out_of_range`).
- U-waarde boven 10 W/m²K (`u_value_out_of_range`).
- q_v10 boven 10 dm³/s·m² (`qv10_out_of_range`).
- Opgegeven maandgebruik boven 5000 kWh per m² per maand (`declared_use_out_of_range`).
- Minder dan 10 m² per woning (`dwelling_count_implausible`).
- A_ls/A_g boven 20 (`loss_area_ratio_implausible`).

TOjuli leidt zijn invoer af uit dezelfde oppervlakten, U-waarden, q_v10 en woningaantallen, dus de grenzen gelden daar ook.

#### Test

`crates/nta8800-core/tests/robustness.rs` verstoort in elke testrun 30 keer per fixture één tot drie getallen van de drie projectvoorbeelden en de zes opnamefixtures. De test eist: geen paniek, geen niet-eindig getal en geen ingreep van het vangnet. Met `ROBUST_TRIALS` en `ROBUST_SEED` kan een lokale run breder; 3 × 1500 proeven per fixture gaven geen bevinding.

### 4 oktober 2026 — verwijderen: warmtepompzones, alle onderdelen, gebouwmaatregelen

- **Bediende zones van warmtepompen.** Verwijder je een zone, dan haalt de app die zone ook uit `servedZoneIds` van `heatingSystems[].ntaHeatPump`, `hotWaterSystems[].ntaHeatPump` en `ntaHeatPumps[]`. De bevestiging noemt die onderdelen. Voorheen meldde de kern daarna `heat_pump_zone_missing`, wat blokkeert. Een lijst die leeg wordt, blijft bestaan; de kern geeft dan alleen de waarschuwing `heat_pump_zones_missing`.
- **Elke verwijdering wordt gecontroleerd op handmatige maatregelen.** De weigering bij een verschuivende positie gold alleen voor zones, vlakken en ramen. Hij geldt nu ook voor:
  - lineaire en puntkoudebruggen;
  - constructies;
  - verwarmings-, ventilatie-, koel- en tapwatersystemen;
  - PV en zonthermische systemen.

  De app bepaalt de toestand na de verwijdering met dezelfde reducer als de echte verwijdering. Een verwijdering van een onderdeel dat niet bestaat, doet niets.
- **Handmatige maatregelen op de afgeleide rekeninvoer** (`target: 'building'`). De bevestiging noemt ze voortaan ter controle. Of hun posities verschuiven, is vooraf niet te bepalen. Verwijderen blijft mogelijk.
- **Dossier.** Een geëxporteerde checklist bevat nooit de status "bezig", ook niet als de aanroeper die doorgeeft. In de rapportweergave tellen onderdelen met "bezig" mee als open.
- **Geneste dialogen.** Een dialoog die binnen een andere dialoog staat, krijgt altijd de focus en de Tab- en Escape-toetsen, ook als beide in één keer verschijnen.

### 4 oktober 2026 — verwijderen zonder losse verwijzingen, tapwatervaten, dialogen en dossierchecklist

- **Verwijderen werkt door in de NTA-invoer.** Verwijder je een zone, vlak of raam, dan verdwijnt ook de NTA-invoer die er met id naar verwijst. Het gaat om `zoneData`, `dynamicWindows`, `groundFloors`, `surfaceTilts`, `lighting`, `humidifiers` en elk ander element met een `zoneId`, `surfaceId` of `windowId`. Een `zoneIds`-lijst van een verwarmings- of koelsysteem wordt ingekort. Een systeem dat alleen verwijderde zones bediende, verdwijnt. Voorheen bleven die verwijzingen staan en stopte de berekening met `zone_data_without_zone`, `dynamic_window_without_window`, `ground_floor_data_without_ground_surface` of `heating_system_zone_unknown`. De bevestiging noemt nu de NTA-onderdelen die meegaan. Een herlabelvergelijking van vóór de verwijdering raakt verouderd.
- **Handmatige maatwerkadviesmaatregelen.** Een handmatige maatregel wijst het project aan op positie (`/zones/0/surfaces/2/…`). Zou een verwijdering die positie laten verschuiven, dan weigert de app de verwijdering en noemt de maatregelen. Anders zou de maatregel zonder foutmelding een ander onderdeel wijzigen. Sjabloonmaatregelen werken met id's en hebben hier geen last van.
- **Tapwatervaten in het tapwatersjabloon (§13.6.2, opmerking 1, p. 566).** Het sjabloon beslist nu net als de kern over het hoofd- én de extra toestellen:
  - een vat blijft bij een elektrische of indirecte boiler, een indirecte warmtepomp of externe warmte;
  - bij een getest toestel blijven alleen de vaten die buiten de test vallen (`notInApplianceTest`); de andere vervallen, ook bij een mix;
  - voorheen keek het sjabloon alleen naar het nieuwe hoofdtoestel en verwijderde het alle vaten.
- **Dialogen.** Tab en Escape werken via een documentbrede afhandeling voor alleen het bovenste open dialoogvenster. Dat blijft werken als het veld met focus verdwijnt. Escape in een geneste dialoog sluit alleen die dialoog.
- **Dossierchecklist.**
  - De checklist in het tabblad en die in de export krijgen nu allebei de beoordeling van de opgeslagen basisopname. Daarmee wordt de inklapreden van toegepaste forfaitaire waarden echt getoetst.
  - Terwijl de kern of de opname nog rekent, tonen de onderdelen die ervan afhangen "bezig" in plaats van "ontbreekt".
- **Samenvoegen per zone gedocumenteerd.** Een functie die apart blijft omdat zij een eigen zone heeft, blijft ook apart in de zone met de hoofdfunctie. Die zone moet dan afb. 6.6 doorstaan; zie `docs/nta8800-basisopname.md`.

### 4 oktober 2026 — bestaande schildelen bewerken en toetsenbord in dialogen

Het bewerken zelf verandert geen rekenregel. Het verwijderen van een zone, vlak of raam doet dat wel: de NTA-invoer die ernaar verwijst, gaat mee (zie de sectie hierboven).

- **Bewerken en verwijderen.** Bestaande zones, vlakken, ramen, koudebruggen, puntkoudebruggen en constructies zijn te bewerken en te verwijderen vanuit:
  - de schilweergave, die nu ook een ramentabel per zone heeft;
  - de projectboom, met dubbelklik;
  - het eigenschappenpaneel.
- **Stabiele ids.** De bestaande dialogen werken het onderdeel ter plekke bij. Id en volgorde blijven gelijk, zodat herlabelvergelijkingen en maatwerkadviessjablonen hun verwijzingen houden.
- **Verwijderen.** Elke verwijdering vraagt om bevestiging. Een constructie die nog door vlakken wordt gebruikt, kan niet worden verwijderd; de melding noemt die vlakken.
- **Toetsenbord in dialogen.** Een dialoog zet de focus in het eerste veld en houdt Tab binnen de dialoog. Escape sluit de dialoog en de focus keert terug naar de knop die hem opende.
- **Labels.** Alle velden in de zone-, raam-, vlak-, constructie- en installatiedialogen hebben een gekoppeld label.
- **Getallen.** De schilweergave toont getallen volgens de taalinstelling.
### 4 oktober 2026 — derde UI-doorloop: utiliteitsopname, dossiercheck en maatregelsjablonen

#### Uitkomst die verandert

- **Kleine functie in een eigen rekenzone (ISSO 75.1 p. 39–40, §6.6 p. 54).** Het samenvoegen van kleine functies met de hoofdfunctie is een toegestane vereenvoudiging. Staat een functie in een rekenzone zonder de hoofdfunctie, dan houdt die functie nu haar eigen gebruiksfunctie. Voorbeeld: een sporthal van 200 m² naast 1 400 m² onderwijs, in een eigen zone, rekent nu als sport in plaats van als onderwijs. Zonder rekenzones blijft het samenvoegen zoals het was.
- **Maatregelsjabloon tapwater.** Bij een opwekker die zijn opslag in het opwekkingsrendement draagt (bijvoorbeeld de forfaitaire tapwaterwarmtepomp) haalt het sjabloon het bestaande voorraadvat weg. Zonder die stap weigerde de kern de maatregel (`hot_water_storage_in_generator_efficiency`). Een opwekker die een apart vat nodig heeft, terwijl er geen vat is, geeft het nieuwe probleem `storageRequired`.

#### Nieuwe controles in de sjablonen

- Ventilatie: een leeg WTW-rendement, een leeg bypassaandeel of lege isolatiegegevens geven `valueRequired`, net als ontbrekende testwaarden van een getest tapwatertoestel.
- Isolatie: de nieuwe constructie heet nu naar de nieuwe Rc ("Gevel Rc 4,7 → Rc 6 (m1)"). Een Rc die niet beter is dan de huidige constructie geeft de waarschuwing `notImproved`; die blokkeert de maatregel niet.
- Verlichting: in een maatregel zijn functie, oppervlakken en verlichtingszones niet meer te wijzigen. Die horen bij het project en werden tot nu toe stil genegeerd.

#### Weergave

- De utiliteitsopname heeft een veld voor de gebruiksfuncties van het gebouw en per verlichtingszone een veld voor de oppervlakte.
- De dossierchecklist op het tabblad Rapport gebruikt dezelfde kernuitvoer en dezelfde labelinvoer-hash als de dossierexport.
- Het maatwerkadviesrapport is consequent Nederlands: soort maatregel, status, getallen met duizendtallen en de reden bij niet-berekende varianten. De interpretaties van de rekenkern staan er herkenbaar in het Engels.
- Opgenomen jaartallen staan zonder duizendtalteken; vaste en uit getallen opgebouwde waardeteksten zijn vertaald. De "Wordt"-kolom van herlabelen toont een samenvatting in plaats van JSON. De kernaudit, het projectoverzicht en de lijst "Niet meegenomen" volgen de taal.

### 3 oktober 2026 — herlabelvergelijking: getallen, anker van het origineel en dossieroordeel

#### Registraties die nu een fout of ontbrekend gegeven melden

- **Oorspronkelijk project gekoppeld aan het oorspronkelijke label (BRL 9500-W §4.2.3–4.2.4, p. 23–24; U p. 18–20).** De SHA-256 van het bewaarde origineel is nu verplicht (`relabel_original_project_hash_required`). Het registratieblok in het origineel moet het opgegeven EP-Online-nummer, de certificaathouder en de opnamedatum bevatten; anders volgt `relabel_original_project_anchor_missing` of `relabel_original_project_anchor_mismatch`. Bewaar het oorspronkelijke project dus na registratie, met het EP-Online-nummer.
- **Kernelhash van de labelinvoer.** Getallen tellen nu in één notatie (`100.0` = `100`, `-0` = `0`, `1e-7` = `0.0000001`), zowel in de vergelijking als in de hash. Een vergelijking die vóór deze wijziging is bewaard, meldt daardoor `relabel_comparison_outdated`: vergelijk opnieuw. De app gebruikt nu dezelfde hash als de kern.
- **Oudere herlabelprojecten.** Alleen bestanden zonder een van de nieuwe herlabelvelden worden bij openen bijgewerkt. Een factuur zonder rol krijgt nu de rol ‘te beoordelen’ (`review`), die niet als bewijs telt; de melding verschijnt altijd als er facturen zijn gemarkeerd. Eerder bijgewerkte bestanden met de rol gespecificeerde factuur blijven ongewijzigd.

#### Indeling van wijzigingen die verandert

- **Ventilatie bij woningen (W bijlage 6a, p. 67).** Een wijziging in de distributie van ventilatie is nu toegestaan (6a), naast het afgiftesysteem. Bij utiliteit blijft dit 6b (U p. 60).

#### Dossier

- Het herlabeloordeel in de checklist en in `herlabel-vergelijking.json` komt uit de nieuwe kernvergelijking bij registratie (`relabelAssessment` in de registratiebeoordeling), niet uit het bewaarde oordeel. Zonder kernuitvoer toont de checklist ‘controleren’.
- `project.oes.json` in het dossier bevat het bewaarde origineel niet meer; dat staat alleen in `herlabel-origineel.oes.json`.

### 3 oktober 2026 — herlabelvergelijking opnieuw gecontroleerd bij registratie

#### Registraties die nu een fout of ontbrekend gegeven melden

- **Herlabelvergelijking (BRL 9500-W §4.2.3–4.2.4 p. 23–24, bijlage 3 p. 63; U p. 18–20, p. 54).** De registratiecontrole vertrouwt het bewaarde oordeel niet meer. De vergelijking bewaart nu het oorspronkelijke projectbestand (`relabelComparison.originalProjectText`); de kern controleert het tegen `originalSha256` (`relabel_original_project_hash_mismatch`), vergelijkt het opnieuw met het huidige project en gebruikt dat oordeel. Is het project na de vergelijking gewijzigd, dan meldt de kern `relabel_comparison_outdated`. De vergelijking gebeurt op een kernelhash van de labelinvoer met gesorteerde sleutels en zonder null-velden, dus een andere sleutelvolgorde maakt haar niet verouderd. Een vergelijking zonder het oorspronkelijke bestand geeft `relabel_original_project_required`: vergelijk opnieuw. Het dossier bevat het origineel als `herlabel-origineel.oes.json`.
- **Oudere herlabelprojecten.** Bij openen krijgt een factuur zonder herlabelrol de rol gespecificeerde factuur; een eenmalige melding noemt de gegevens die nog ontbreken.

#### Indeling van wijzigingen die verandert

- **Ruimteverwarming in het NTA-blok.** `ntaCalculation/emission`, `/distribution`, `/distributionSystem` en `zoneData/*/emission|distribution` tellen nu als verwarming: bij woningen 6a (W p. 67) in plaats van "ter beoordeling".
- **PV en zonthermie in het NTA-blok.** Een gewijzigde helling, oriëntatie, module- of collectoroppervlakte of piekvermogen (`tiltDeg`, `azimuthDeg`, `moduleAreaM2`, `collectorAreaM2`, `peakPower`, oud `collectorArea`) is nu "ter beoordeling" in plaats van een eigenschap die mag.

#### Maatwerkadvies

- Een verlichtingsmaatregel uit een eerdere vorm blijft geblokkeerd (`migrationReview`) tot de adviseur bevestigt; bewerken heft dat niet meer op.
- Een oude ventilatiemaatregel zonder ventilatiesysteem geeft `ventilationSystemRequired` in plaats van een patch die het systeem leegmaakt.
- De onderbouwing van minimale belemmering bij PV (tabel 17.3 situatie a, p. 706–707) staat nu in het maatwerkadviesrapport en in de dossierchecklist.

### 3 oktober 2026 — herlabelen volgens BRL 9500 §4.2.3 en bijlage 6a/6b

#### Registraties die nu een ontbrekend gegeven melden

- **Herlabelen (berichttype `relabel`).** De kern vraagt nu het certificaatnummer en het EP-Online-nummer van het oorspronkelijke label (`original_certificate_number_required`, `original_ep_online_number_required`), de bewaarde herlabelvergelijking (`relabel_comparison_required`) en een bewijsstuk met de rol offerte met opdracht of gespecificeerde factuur (`relabel_proof_required`; W §4.2.3 p. 23, U p. 19). Een ander certificaatnummer dan dat van het oorspronkelijke label geeft `relabel_certificate_holder_differs`; een vergelijking met een 6b-wijziging geeft `relabel_changes_not_allowed`. Bij een wijziging aan PV of zonthermie zijn een foto met beschaduwing en de bevestiging van de fysieke aansluiting nodig; bij utiliteit bevestigt de adviseur dat er geen 6b-wijzigingen zijn (U p. 19). Bestaande herlabelprojecten zijn dus niet meer gereed voor registratie tot deze gegevens zijn ingevuld.

#### Indeling van wijzigingen die verandert

- Alleen een gewijzigde gebruiksoppervlakte (`floorArea`) is nu 6b in plaats van beoordeling.
- Grootte, helling en oriëntatie van PV of zonthermie vragen beoordeling in plaats van 6b.
- De g-waarde van beglazing is 6a in plaats van beoordeling.
- Bij woningen volgen distributie, afgifte en regeling bijlage 6a per functie (p. 67): bij ventilatie en tapwater geldt alleen het afgiftesysteem als 6a, de rest vraagt beoordeling.
- Het aandeel van een opwekker naast een toegevoegde opwekker vraagt beoordeling.
- Wijzigingen in `maatwerkadvies` en `basisopname` worden niet meer als wijziging getoond.

#### Dossier

- De herlabelvergelijking wordt bij de registratie bewaard en komt als `herlabel-vergelijking.json` in het projectdossier. De checklist toetst nu de verbeterdatum aan de 24 maanden, vraagt een bewijsstuk met de herlabelrol en markeert een vergelijking als verouderd als het project daarna is gewijzigd.
### 3 oktober 2026 — maatwerkadvies: maatregelsjablonen na review

#### Uitkomsten die veranderen

- **Patches altijd vers.** De rapportexport en het dossier rekenden met de patch van de laatste berekening in het paneel. Na een toegevoegd of verwijderd vlak kon die een ander element wijzigen. `buildMaatwerkadviesInput` genereert de sjabloonpatches nu altijd opnieuw.
- **Ventilatie en verlichting.** Deze sjablonen bewaren alleen hun wijzigingen (systeem, of per verlichtingszone vermogen, parasitair vermogen, aanwezigheid, daglicht en afzuigarmaturen). Latere wijzigingen in het project worden niet meer teruggezet en niet meer als besparing van de maatregel geboekt.
- **PV.** Nieuwe PV-systemen starten op belemmering situatie e) (tabel 17.3 met opmerking 23, p. 706–707). Situatie a) vraagt een onderbouwing.
- **Dakisolatie.** Een dak steiler dan 60° krijgt R_si 0,13 (tabel C.2 opmerking 3, p. 778).
- **Warmtepomp.** De bronvlaggen van 5.31/5.32 volgen uit de gekozen bron; een collectieve bron van 20–40 °C of ≥ 40 °C geeft geen `heat_pump_source_contradiction` meer.

#### Maatregelen die nu niet worden doorgerekend

- Een sjabloon met openstaande problemen gaat met `incomplete` naar de kern; elke variant met die maatregel meldt `measure_template_incomplete`. Nieuwe problemen:
  - `selectionStale`: gekozen vlakken, ramen of verlichtingszones bestaan niet meer;
  - `supplyTemperatureRequired`: warmtepomp met afgifte via water zonder ontwerpaanvoertemperatuur;
  - `peakPowerRequired`: PV zonder piekvermogen;
  - `obstructionEvidenceRequired`: PV met minimale belemmering zonder onderbouwing;
  - `migrationReview`: verlichtingssjabloon in de oude vorm, tot de adviseur de wijzigingen bevestigt.
- Gevelwanden tegen de grond worden niet meer als isolatievlak aangeboden.

#### Formulier

- Bij forfaitair verlichtingsvermogen is daglichtregeling uitgeschakeld (14.24, p. 667); bij centrale aan-schakeling staat een toelichting (14.16/14.17, p. 664).
- Wisselen van sjabloonsoort houdt een zelf ingevulde levensduur en categorie.
- Een investering van 0 € geeft een waarschuwing.

### 3 oktober 2026 — reviewcorrecties rapportafronding, restset, bouwjaar en setpoints

#### Projecten die nu `incomplete` worden

- **Restset naast een bijlage Q-warmtepomp met β ≥ 1 (opmerking 1, p. 323).** Is maar een deel van de nominale vermogens van de overige opwekkers ingevuld, dan meldt de projectroute het gat `rest_set_power_partial` bij elk ontbrekend vermogen, met het volledige projectpad (`ntaCalculation.generator.generators[i].nominalPowerKw` of `ntaCalculation.additionalHeatingSystems[k].generator…`). De status is `incomplete` en de melding zegt wat te doen: alle vermogens invullen of alle leegmaken. Rechtstreekse kerninvoer krijgt dezelfde code als kernmelding.

#### Uitkomsten die veranderen

- **Bouwjaar (Regeling art. 4 a, p. 5; NTA §5.3.2, p. 75–76).** Het bouwjaar van de labelgegevens en van de standaard voor woningisolatie is nu het registratiejaar, anders het bouwjaar in het NTA-blok. Het ventilatiejaar valt niet meer terug als bouwjaar: tabel 11.13 (p. 486) vraagt een bouw- of renovatiejaar. Een project zonder registratie- en NTA-bouwjaar krijgt dus geen bouwjaar meer uit de ventilatie; beide voorbeeldprojecten hebben nu een bouwjaar in het NTA-blok (2020 en 2021).
- **Waarschuwingen.** `construction_year_mismatch` vergelijkt nu het NTA-blok met de registratie. Het nieuwe `ventilation_year_before_construction_year` meldt alleen een ventilatiejaar vóór het bouwjaar; een later jaar is een renovatiejaar.
- **Rekenrapport.** De regel ‘Bouwjaar’ toont hetzelfde opgeloste jaar als de labelgegevens.

#### Weergave

- **BENG-rapport.** Eisen staan met dezelfde decimalen als de waarde: BENG 1 en 2 en TOjuli op 0,01, BENG 3 op 0,1. De kern toetst onafgeronde waarden; een rij als ‘74,11 | ≤ 74,1 | voldoet’ of ‘32,6 | ≥ 33 | voldoet’ kan niet meer voorkomen.
- **Tabel 7.13 in het formulier.** De controle loopt nu, net als de kern, over de rekenzones van het project in plaats van over `zoneData`. De knop ‘Tabelwaarden gebruiken’ neemt de bronverwijzing van het blok over, anders ‘NTA 8800 tabel 7.13’, zodat de kern de setpoints niet weigert om een lege bron. De projectroute kent geen gebruiksaanpassing; die tak is uit de controle verwijderd.
- **Tijdstempels.** Rekenrapport, invoerdossier en maatwerkadviesrapport tonen de generatietijd als `<time datetime="…">` met de exacte ISO-tijd. In het projectdossier is dat dezelfde tijd als `generatedAt` in `manifest.json`.
### 3 oktober 2026 — maatregelsjablonen in het maatwerkadvies

- **Maatregelen via sjablonen (ISSO 82.2/75.2 §4.3).** Een nieuwe maatregel start als sjabloon: isolatie van dak, gevel of vloer, beglazing, kierdichting, ventilatiesysteem, warmtepomp (forfaitair of bijlage Q), tapwatertoestel, PV, zonneboiler, douche-WTW en verlichting. Het sjabloon genereert de JSON-patch en toont die als voorbeeld; de patchregels blijven beschikbaar onder "Handmatig".
- **Opgeslagen maatregelen.** Een maatregel krijgt het optionele veld `template`; de kern negeert het. Bestaande maatregelen zonder sjabloon openen als "Handmatig" en rekenen ongewijzigd.
- **Patches opnieuw opgebouwd.** Bij het doorrekenen worden de patches van sjabloonmaatregelen opnieuw gegenereerd tegen het huidige project, zodat een gewijzigde volgorde van vlakken of ramen de paden niet laat verouderen.

### 3 oktober 2026 — vakantiewoning in ZEB, reservevermogens, één bouwjaar en lege enum-tag

- **ZEB-indicator (bijlage AB).** Een vakantiewoning (‘andere logiesfunctie’) telt in tabel AB.1 nu in de kolom woningbouw (f_du januari 0,75 in plaats van 0,55), zoals tabel 6.1 (p. 136) haar als woonfunctie behandelt. ZEB-uitkomsten van projecten met deze functie veranderen.
- **Resterende opwekkers bij β ≥ 1 (9.56, p. 323; opmerking 1).** Een set waarin een bijlage Q-warmtepomp als preferentie 1 op β ≥ 1 is geschat, rekent de rest alleen nog met gelijke delen als **alle** resterende vermogens ontbreken. Ontbreekt een deel, dan meldt de projectroute het gat `rest_set_power_partial` bij elk ontbrekend vermogen en worden zulke opgeslagen projecten `incomplete`; de kern zelf meldt dezelfde code (status `invalid` bij rechtstreekse kerninvoer).
- **Eén bouwjaar.** Het ventilatiebouwjaar wordt ook per rekenzone gelezen (`zoneData[i].ventilation.constructionYear`). De labelgegevens (Regeling art. 4 a) gebruiken nu hetzelfde bouwjaar als de standaard voor woningisolatie (§5.3.2). Een registratiebouwjaar dat afwijkt van het ventilatiebouwjaar geeft de waarschuwing `construction_year_mismatch`.
- **Registratiewaarschuwingen.** `plausibility_borderline_label` en `plausibility_ep2_out_of_range` wijzen nu naar `performance.labelPrimaryFossilIndicatorKwhPerM2Year`.
- **Lege enum-tag.** Is de tag van een variantblok leeg (bijvoorbeeld `method` of `kind`), dan meldt de kern alleen die tag; lege velden in hetzelfde blok worden pas beoordeeld als de variant gekozen is. Een tag is een enum-veld waarvan een andere variant de toegestane of verplichte velden van het blok verandert.
### 3 oktober 2026 — BENG-rapport uit de kern, taal van exports en tweede UI-doorloop

Geen wijziging in de rekenkern; wel in wat rapporten en schermen tonen.

- **BENG-rapport.** Het tabblad Rapport, "Rapport exporteren" en de afdrukvoorbeeld gebruiken nu de NTA 8800-kernuitkomst: BENG 1/2/3 met de Bbl-eisen en het oordeel van de kern, TOjuli (alleen woonfunctie), indicatieve labelklasse, maandoverzicht (warmte-/koudebehoefte, zonwinst, transmissie), energiebalans en PV uit hoofdstuk 16. Het vereenvoudigde rekenmodel verschijnt alleen zonder kernuitkomst, gemarkeerd "Indicatief, niet volgens NTA 8800". Bij een ketel heet het rendement nu η in plaats van COP.
- **Taal.** Het BENG-rapport volgt de taal van de interface (Nederlands, anders Engels): `lang`, decimalen, datums en teksten. Het NTA 8800-rekenrapport, het invoer- en projectdossier en het maatwerkadviesrapport blijven Nederlandse BRL 9500-documenten, maar gebruiken nu overal een decimale komma, Nederlandse datums en vertaalde meldingscodes (met de code ernaast). Registratiecodes (`client_required`, `plausibility_*` en andere) hebben Nederlandse en Engelse omschrijvingen.
- **Maatwerkadvies.** Een variant die niet te berekenen is, toont nu de redenen van de kern in plaats van alleen "⚠". In het Engels worden de vaste adviesteksten van de kern vertaald; eigen teksten van de adviseur blijven zoals ingevoerd. "Huidige situatie" volgt de interfacetaal.
- **Herlabelen.** Clusters en toelichtingen van Bijlage 6a/6b zijn vertaald, getallen volgen de taal, en een wijziging toont de namen van zone, vlak en raam in plaats van een JSON-pointer (de pointer staat in de tooltip).
- **Basisopname.** Toegepaste standaardwaarden tonen ja/nee, getallen volgens de taal, PascalCase- en snake_case-waarden als vertaald label met de code, en de vaste Engelse waardeteksten van de kern in het Nederlands. De BENG-waarden van de opname volgen de taal.
- **Projectgegevens.** Alle velden hebben een gekoppeld label; het venster heeft `role="dialog"` met `aria-modal` en is ingedeeld in secties met twee kolommen. Velden voor herlabelen en vervangen verschijnen alleen bij dat berichttype.
- **Setpointcontrole (tabel 7.13).** De controle in het formulier volgt de kern: per zone van `zoneData` de eigen functie of het oppervlaktegewogen gemiddelde van `functionAreas` (§6.5.3), tegen de setpoints die de zone gebruikt; met een gebruiksaanpassing (bijlage Z) geen controle. "Tabelwaarden gebruiken" schrijft naar de zone zelf als zones met gedeelde setpoints verschillende tabelwaarden nodig hebben.
- **Energiebalans.** De winsten volgen nu Q_H;gn van de kern; serrewinst (7.37) en andere termen staan als "overige winst". Ontbreekt de energie per energiefunctie, dan meldt de grafiek dat in plaats van nulwaarden.
- **Verouderde uitkomsten.** Alle uitkomsten van het vereenvoudigde model (kaarten, GTO, grafieken, vloeroppervlak) staan in één gemarkeerd blok met een duidelijke melding.
- **Kernaanroepen.** De resultatenweergave wacht 400 ms na een wijziging (zoals het live-voorbeeld) en deelt een lopende kernberekening met het live-voorbeeld, het rapport en de afdruk.
- **Overig.** Keuzelijsten met lange teksten krijgen twee kolommen en de volledige tekst als tooltip; schuifbalken volgen het donkere thema.

### 3 oktober 2026 — resultatenweergave en bouwjaar

- **Bouwjaar.** Zonder `ntaCalculation.constructionYear` neemt de projectroute het bouwjaar uit de registratie, en anders het bouwjaar van de ventilatiesectie (tabel 11.13, dezelfde grootheid). Projecten die alleen het ventilatiebouwjaar hadden, krijgen nu de standaard voor woningisolatie (§5.3.2) in plaats van de melding `standard_insulation_construction_year_missing`.
- **Resultaten.** Met een berekende NTA-kernuitkomst tonen de BENG-kaarten, TOjuli en de energiebalans (inclusief PV-opwek uit hoofdstuk 16) de kernwaarden; de vereenvoudigde rekenmethode verschijnt alleen nog als de kern geen volledige uitkomst heeft. Rekenresultaten blijven na een projectwijziging zichtbaar als verouderd, zodat het maatwerkadvies- en herlabelpaneel niet verdwijnen.
### 3 oktober 2026 — reviewcorrecties biomassa, Bbl-functies, labelgegevens en lege velden

#### Uitkomsten die veranderen

- **Identieke systemen en biomassa (§9.1, p. 287; tabel 5.2/5.4, p. 94).** Identieke systemen (bijvoorbeeld één pelletkachel per woning) zijn afzonderlijke fysieke installaties. Hun vermogen wordt niet meer vermenigvuldigd met het aantal: 40 woningen met elk een kachel van 15 kW blijven bmB in plaats van bmA (f_P = 0). Alleen de opwekkers van één meervoudige set tellen op.
- **Eén grote lokale biomassaverwarmer.** Een kachel met een eigen vollastvermogen boven 500 kW wordt nu ook als bmA gerekend; voorheen kreeg hij alleen de waarschuwing.
- **Restset bij β₁ ≥ 1 (9.56, p. 323; opmerking 4, p. 324).** Opgegeven nominale vermogens van de overige voorkeuren worden weer gebruikt; gelijke aandelen gelden alleen als die vermogens ontbreken.
- **Vakantiewoning (p. 136).** Bbl-functie 'andere logiesfunctie' telt als woningbouw en past nu bij een woningberekening.
- **Registratiecontroles.** De grenslabelwaarschuwing en de andere registratiecontroles gebruiken de label-EP2 (EMGforf, Regeling art. 2 lid 3).
- **Rapport.** De labelregel toont het hernieuwbaar aandeel van het labelscenario, zonder terug te vallen op het aandeel met verklaring.

#### Gaten die waarschuwingen worden

- `bbl_function_areas_sum_mismatch` en `label_function_areas_sum_mismatch` blokkeren de berekening niet meer; het zijn nu waarschuwingen. Functieoppervlakten komen vaak uit vergunning of BAG (NEN 2580), A_g;tot volgt §6.6.2–6.6.4 (p. 158). Labelfuncties worden nu ook bij een woningberekening getoetst.

#### Lege velden

- De zoektocht naar lege verplichte velden is uitgebreid getest. Proeven maken nu ook hele objecten, lijsten en lijstelementen leeg, naast losse waarden, in drie voorbeelden: tussenwoning, kantoor, en kantoor met een meervoudige opwekkerset en een berekend warmtenet (bijlage P). Getoetst wordt op 0 onterechte meldingen, 0 gemiste velden en geen restfout. De kosten worden begrensd op aantal deserialisaties in plaats van tijd.

### 3 oktober 2026 — hoofdstuk 5, label, Bbl-functies en maatwerkadvies

#### Uitkomsten die veranderen

- **Woninglabel met gebiedsmaatregelen (Regeling art. 2 lid 3, p. 4).** Met een kwaliteitsverklaring voor externe levering rekent het woninglabel nu met het forfaitaire scenario (EMGforf); de labelklasse kan daardoor lager uitvallen. BENG 2 en BENG 3 blijven op het scenario met verklaring. Utiliteit volgt art. 3 lid 3 en blijft op de verklaring. Nieuwe uitvoer: `labelPrimaryFossilIndicatorKwhPerM2Year` en `labelRenewableSharePercent`; de labelgegevens (art. 4) en het maatwerkadvies gebruiken die.
- **Netto contante waarde.** Een maatregel die pas na de horizon wordt uitgevoerd, kreeg een restwaarde zonder investering; die restwaarde vervalt.
- **Brandurenfactor (bijlage Z, p. 1133).** Een maatwerkadviesrun met een brandurenfactor op de verlichting geeft geen labelklasse meer.
- **EPTot + EPrenTot = 0.** Dit maakte de hele indicatorset ongeldig. Nu blijft EP2 bepaald (0 geeft A++++) en is het aandeel hernieuwbare energie `null`.

#### Invoer die nu een gat geeft

- `bbl_functions_residential_utility_mixed`: woon- en utiliteitsfuncties in één `bblFunctions`-lijst (§5.3.1, p. 70).
- `bbl_functions_scope_mismatch`: een Bbl-functie die niet bij de berekening (woningbouw of utiliteit) past.
- `bbl_function_areas_sum_mismatch` en `label_function_areas_sum_mismatch`: functieoppervlakten die niet binnen 0,5 % (of 0,5 m²) optellen tot A_g;tot.
- `label_functions_residential_in_utility`: een woonfunctie in de labelfuncties van een utiliteitsgebouw (Regeling bijlage Ia).

#### Nieuwe waarschuwingen

- `renewable_share_above_100_negative_primary_fossil`: RER boven 100 % door een negatieve EPTot (5.3 letterlijk).
- `standard_insulation_construction_year_missing`: een woning zonder bouwjaar, zodat de standaard voor woningisolatie (§5.3.2) ontbreekt.

#### Overig

- Het maatwerkadviespaneel sorteert maatregelen en pakketten desgewenst op terugverdientijd of netto contante waarde; ISSO 82.2 §6.2.4 schrijft geen rangorde voor.
### 3 oktober 2026 — lege velden getoetst met willekeurige proeven, biomassa per installatie

#### Status die verandert

- **Lege velden.** De zoektocht naar lege waarden werkt nu in twee stappen. Eerst wordt een kopie gerepareerd tot die inleest: elke fout krijgt op zijn plek een waarde die de kern accepteert, bij een keuzeveld de variant die alle naastgelegen velden van de invoer kent. Daarna wordt elke lege waarde afzonderlijk teruggezet. Een veld dat leeg mag zijn (ook een `null` dat via `deserialize_with` "onbepaald" betekent, zoals `nominalThermalCapacityKw` in het BACS-blok) wordt niet meer als ontbrekend gemeld. Een leeg keuzeveld (`method`, `kind`) verbergt de lege velden daarachter niet meer en geeft geen extra `nta_calculation_block_invalid`. Een vaste test zet in beide voorbeeldprojecten 300 keer 1 of 2 willekeurige waarden leeg en daarna alle waarden tegelijk: geen onterechte en geen gemiste meldingen.
- **Biomassa boven 500 kW met bijlage R-vinkje.** Dat blokkeert niet meer. De installatie rekent als bmA (tabel 5.2, p. 94; p. 95) en `biomass_class_conflict` is nu een waarschuwing.

#### Uitkomsten die veranderen

- **Biomassaklasse per installatie** (tabel 5.2 en 5.4, "per installatie", p. 94–95). Het vermogen van alle vaste-biomassatoestellen in één verwarmingssysteem wordt opgeteld: de toestellen van een meervoudige opwekking, en N gelijke toestellen (§9.1) als N × het vermogen. Twee houtketels van 300 kW zijn samen bmA, ook als elke ketel onder bijlage R valt.
- **Geschatte β bij een warmtepomp met bijlage Q.** Als voorkeur 1 met β = 1 geschat is, krijgen de overige toestellen elk een gelijk aandeel. Eerder vroeg de kern dan alsnog om nominale vermogens (`generator_nominal_power_invalid`).

### 3 oktober 2026 — reviewcorrecties lege velden, herlabeling en WLC-GWP

#### Status die verandert

- **Lege velden.** De zoektocht naar lege waarden (`nta_value_missing`) is herschreven. Ook een leeg veld met `#[serde(default)]` dat geen keuze toelaat, een leeg keuzeveld (enum) naast een ander leeg veld in hetzelfde blok en een lege waarde in een samengevoegde (flatten) structuur geven nu een eigen gat op hun pad. Toegestane lege waarden (maanden buiten bedrijf, optionele velden) worden niet gemeld. Een echt ontbrekend verplicht veld wordt niet meer verward met een lege waarde met dezelfde naam elders; het blok krijgt dan naast de gaten ook `nta_calculation_block_invalid`.
- **Hele gebouw (WLC-GWP).** Een woning telt alleen als het hele gebouw als die vrijstaand is vastgelegd: met de vrijstaande typen van tabel 11.14 in de infiltratie-invoer, of met een woningtype dat "vrijstaand" noemt (niet half vrijstaand of twee-onder-een-kap), en niet in een woongebouw. Een tussenwoning zonder ingevoerde woningscheidende wanden telde eerder als hele gebouw; nu geldt weer `buildingUsableFloorAreaM2` of de waarschuwing `wlc_gwp_building_area_unknown`.
- **Herlabeling (BRL 9500-W §4.2.4, p. 24).** Bij het opslaan legt de app nu de rekenkern van de kernelstempel vast (`software.kernelVersion`). Een herlabeling houdt de opgeslagen identiteit en neemt, als die geen kern noemt, de opgegeven `originalKernelVersion` over. `relabel_software_kernel_differs` meldt nu een bewaarde kern die afwijkt van de opgegeven oorspronkelijke kern (of, zonder die opgave, van de huidige).

#### Uitkomsten die veranderen

- **P.78 met alleen ontvochtiging.** Een perceel met alleen een jaarwaarde voor ontvochtiging gaf een maandprofiel van nullen dat als bekend gold, waardoor P.68/P.69 nul werden. Dan gelden de maanden nu als onbekend. De maandwaarden van de koude-invoer (`monthlyInputKwh`) kunnen volgens P.78 lager optellen dan het jaartotaal; dat staat nu bij het veld.

#### Rapport en invoer

- "Gereed voor registratie" noemt alle redenen: dossier onvolledig en/of rekenprogramma nog niet geattesteerd.
- Maatwerkadvies: elke wijzigingsregel houdt na het verwijderen van een andere regel zijn eigen waarde en type, en een opgeslagen `null` opent weer als type null.
### 3 oktober 2026 — bijlage M, meervoudige opwekking en biomassa na de herberekening

#### Uitkomsten die veranderen

- **Warmtepomp met bijlage Q in een meervoudige opwekking** (§9.6.3, p. 331; 9.6.3.2, p. 340). De warmtepomp krijgt de energiefractie van bijlage Q op de hele knooplevering in plaats van tabel 9.23; de overige toestellen leveren de rest. Bij volledige dekking verbruikt een bijgeplaatste ketel geen brandstof meer.
- **Biomassaketel boven 500 kW** (tabel 5.2/5.4). Het typeplaatvermogen bepaalt de klasse: boven 500 kW is het bmA, ook zonder het vinkje.

#### Projecten die nu `incomplete` of ongeldig worden

- `annex_q_heat_pump_first_preference`: een warmtepomp met bijlage Q in een meervoudige opwekking moet als enige voorkeur 1 hebben.
- `boiler_fuel_without_primary_factor`: een ketel met productgegevens (bijlage M) op lpg, steenkool of bruinkool. Tabel M.3 kent ze, tabel 5.2/5.3 geven er geen factor voor.
- `boiler_condensing_efficiencies_inverted`: condenserend vollastrendement bij 60 °C hoger dan bij 30 °C (M.8).
- `boiler_standby_loss_invalid` boven f_gen;ls;P0 = 0,1 en `boiler_power_invalid` voor P_int onder 0,05·P_n.
- `product_boiler_capacity_insufficient` / `product_boiler_output_without_operating_hours`: de ketel kan de maandlevering niet binnen de bedrijfstijd van tabel 9.15 leveren (M.24/M.25).
- `biomass_class_conflict`: bijlage R-vinkje bij een typeplaatvermogen boven 500 kW.

### 3 oktober 2026 — hoofdstuk 12 en bijlage P na de herberekening

#### Uitkomsten die veranderen

- **Koudenet met ontvochtiging als jaarwaarde (P.77/P.78, p. 1022).** Het maandprofiel blijft nu bestaan; ontvochtiging telt alleen in de jaarwaarde. Pompenergie per maand (P.68) is daardoor mogelijk, en de standby van P.69 loopt alleen in de maanden met koudelevering.
- **Warmtenet met een deel zonder maandwaarden (P.73/P.74, p. 1020).** Alleen dat deel volgt P.74; de opgegeven maandwaarden van de andere percelen blijven. Dit verandert vooral de looptijden van P.60.
- **f_Pren;dc (5.49, p. 129)** wordt niet meer naar beneden afgerond.
- **Een tweede bevochtiger in dezelfde rekenzone** geeft nu het gat `humidifier_zone_duplicate` (§12.1, p. 520).

#### Nieuw

- `sourcePumpIncluded` bij een opgegeven warmtepomprendement in bijlage P: de bronpomp of -ventilator zit in het rendement, dus 0 W/kW in plaats van 10 W/kW (P.6.8.4.3, p. 1009).
- `servedAreaM2` bij een stoombevochtiger: de bediende oppervlakte voor de 500 m²-grens van 12.2.1 (p. 521). Zonder waarde blijft de oppervlakte aan het verwarmingssysteem gelden.
- Waarschuwingen `chp_co2_factor_negative` (P.27) en `cold_network_gain_outside_cooling_months` (P.13–P.18 voor koude).
- Een primaire factor 0 heet in de uitvoer nu `0` in plaats van `-0`.
### 3 oktober 2026 — review van de registratie, lege waarden en bijlage P-formulieren

#### Status van opgeslagen projecten

- **Gereed voor registratie.** `readyForRegistration` is nu waar als het dossier compleet is én het rekenprogramma een BRL 9501-attest heeft (Regeling art. 2 en 3, p. 4–5). Zolang `SOFTWARE_ATTEST_NUMBER` leeg is, is het dus overal onwaar. Het nieuwe veld `dossierComplete` beoordeelt alleen het dossier; het rapport toont beide.
- **Herlabelen.** Alleen een herlabeling houdt het opgeslagen rekenprogramma (BRL 9500-W §4.2.4, p. 24). Het programma legt nu ook de kernversie vast (`software.kernelVersion`). Wijkt die af van de rekenkern die nu rekent, dan meldt de registratie `relabel_software_kernel_differs`. Een vervanging is een nieuwe berekening en krijgt het huidige programma (p. 23).
- **WLC-GWP.** Een oplevering vóór 2028 zonder datum van de toets Bbl geeft weer "niet vereist" in plaats van "onbekend". Een vrijstaande woning met één woning in de berekening is het hele gebouw: haar eigen A_g beslist, zonder waarschuwing.
- **Bijlage P.** Een maandreeks van de netwatertemperatuur zonder enige maand in bedrijf geeft `network_temperature_required` (P.14). Een berekende opslag (P.35) zonder vat, laadleiding of warmtewisselaar geeft `storage_components_required`; zonder onderdelen kwam η op 1, gunstiger dan het forfait. Het formulier begint de berekende opslag nu met één vat.
- **Maatwerkadvies.** Een wijziging met een leeg pad of een pad zonder `/` wordt geweigerd met `measure_patch_failed` op die regel. Voorheen verving een leeg pad het hele project.

#### Lege waarden

`nta_value_missing` vindt nu ook twee of meer lege verplichte velden in één type (bijvoorbeeld lengte en gronddekking van één leidingsegment). Toegestane lege maanden in reeksen als `temperaturesC` worden niet meer gemeld.

#### Formulieren

- Elke wijziging in een maatregel heeft nu een soort waarde (getal, tekst, ja/nee, leeg of JSON), zodat tekst "2" tekst blijft.
- Het wissen van "overig verlies" verwijdert het veld (de kern neemt dan 0). Een gebieds-PV-systeem begint zonder belemmeringsfactor.
- De dode JSON-controle bij het opslaan van het NTA-formulier is verwijderd.

### 3 oktober 2026 — formulieren zonder JSON-editors (bijlage P, hoofdstuk 14, randdelen, maatwerkadvies)

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
### 3 oktober 2026 — reviewcorrecties registratie en combi-tapwater

#### Uitkomsten die veranderen

- **Combi-aandeel per systeem (13.184/13.185, p. 653; 9.1, p. 288).** Het tapwateraandeel van een combi- of afleverset wordt nu berekend met de opwekkeroutput van het hoofdsysteem, het enige systeem met de tapwaterbelasting, en alleen op de dragers van dat systeem toegepast. Een extra verwarmingssysteem (bijvoorbeeld een gasketel naast een warmtepomp-combi) blijft volledig E_H × f_BACS. Projecten met meerdere verwarmingssystemen, een combi en f_BACS ≠ 1 krijgen daardoor een ander gebruik en EPTot. Projecten met één verwarmingssysteem veranderen niet.

#### Invoer

- **Lege waarden in getagde enums.** Een lege waarde (null) binnen bijvoorbeeld een opwekker of een PV-piekvermogen gaf nog `nta_calculation_block_invalid`. De route zoekt de lege velden nu in de JSON zelf en meldt `nta_value_missing` op het exacte pad. Een optioneel veld dat leeg mag zijn wordt niet gemeld.

#### Registratie

- **Attest apart van de dossiercontrole.** `software_attest_number_missing` en `software_required` zijn geen registratiepunten meer. De beoordeling geeft `software` (het opgeslagen programma, of dit programma met de kernversie voor projecten zonder) en `softwareAttested`. `readyForRegistration` hangt alleen nog van het dossier af; het rapport noemt een ontbrekend attest naast "gereed".
- **Herlabelen en vervangen** houden bij opslaan het programma van de oorspronkelijke berekening (BRL 9500-W §4.2.4, p. 23–24); een gewone registratie krijgt het huidige programma.
- **Dossierchecklist.** De punten van Bijlage 6a/6b verschijnen ook bij `messageType: relabel` zonder het oude veld `relabel`.
- **BAG-id.** Een pand-id (type 10) is toegestaan voor andere gebouwen dan woningen (Regeling art. 5 lid 1 onder a, p. 6). Voor een woninglabel blijft het adresseerbare object verplicht (Praktijkhandboek p. 46).
- **WLC-GWP bij oplevering.** De plicht volgt de toets Bbl (BRL 9500-W p. 21, 62). Een oplevering gebruikt de nieuwe datum `bblCheckDate`. Zonder die datum is de plicht onbeslist en volgt een plausibiliteitsmelding (`wlc_gwp_bbl_check_date_unknown`) in plaats van een blokkerende melding. De grens van 1000 m² geldt per gebouw: `buildingUsableFloorAreaM2`; bij één woning onder de grens zonder gebouw-A_g volgt `wlc_gwp_building_area_unknown`.
- **Vorige labelklasse.** Hoofdletters doen er niet toe ("a+" is A+); een onbekende klasse geeft een plausibiliteitsmelding in plaats van een fout.
- **Lokaal BAG-register.** Alleen een geregistreerd label (met EP-Online-nummer) komt in het register. Het leegmaken van het BAG-id of het nummer verwijdert de regel. Een label waarvan de geldigheid (opnamedatum + 10 jaar) vóór de nieuwe opname of registratie verliep, telt niet als conflict.
- **Bronverwijzing.** "Regeling art. 5 lid b" is nu "art. 5 lid 1 onder b".

### 3 oktober 2026 — registratie: rekenprogramma, berichttypen, WLC-GWP, BAG en plausibiliteit

Rekenuitkomsten veranderen niet. De registratiecontrole wordt strenger:

- **Rekenprogramma (Regeling art. 5 lid 1 onder b, p. 6):** de app schrijft `registration.software` bij het opslaan van de projectgegevens. Zolang het programma niet volgens BRL 9501 is geattesteerd, meldt de kern `software_attest_number_missing` (ontbreekt); `readyForRegistration` blijft dan `false`. Opgeslagen projecten zonder dit blok krijgen `software_required` tot de projectgegevens opnieuw worden opgeslagen.
- **BAG-id:** een id dat niet uit 16 cijfers bestaat of geen verblijfsobject, ligplaats of standplaats is, geeft nu een fout (`bag_object_id_invalid`, `bag_object_id_not_addressable`; Praktijkhandboek v2 p. 46).
- **Berichttype (BRL 9500-W p. 24–25):** nieuw veld `messageType` met `regular`, `relabel` en `replacement`. Het oude `relabel: true` blijft werken. Vervangen vraagt het EP-Online-nummer van het vervangen label en moet binnen 24 maanden na de oorspronkelijke opname.
- **WLC-GWP (BRL 9500-W p. 18, 21, 62):** vanaf 1-1-2028 is bij toets Bbl en oplevering van een gebouw > 1000 m² de uitkomst met rapportverwijzing verplicht.
- **Plausibiliteit (BRL 9500-W p. 42):** nieuwe, nooit blokkerende lijst `registration.plausibility`, ook in het rapport.
- **A<sub>g</sub> (Praktijkhandboek v2 p. 70):** labelgegevens tonen A<sub>g</sub> op twee decimalen; meer decimalen in de invoer geeft een plausibiliteitsmelding.
- **Bronnen:** de laatste verwijzingen naar de internetconsultatie (afgifte H9, hulpenergie warmtepompen, gaswarmtepompen, opwekkerverdeling en de collectieve bron) zijn vervangen door paginanummers van NTA 8800:2025+C1:2026.
### 3 oktober 2026 — reviewcorrecties energie per energiefunctie en invoerformulieren

#### Uitkomsten die veranderen

- **Tapwater uit een combitoestel of afleverset op het verwarmingstoestel (13.184/13.185, p. 653; 5.20a, p. 89).** Het tapwateraandeel van het toestel is E_W en telt nu onder warm tapwater, zonder f_BACS. Alleen E_H krijgt f_BACS. Bij een utiliteitsgebouw met f_BACS = 1,05 en zo'n toestel dalen het gebruik, EPtot en BENG 2 met 0,05 × het tapwateraandeel. Woningen (f_BACS = 1) houden dezelfde totalen; in de verdeling per energiefunctie verschuift het aandeel van verwarming naar warm tapwater. Het hulpenergiegebruik van het toestel blijft bij verwarming (p. 653).

#### Opgeslagen projecten

- **Leeg getal in een lijst.** Een lege maandwaarde of een leeg pompvermogen gaf `nta_calculation_block_invalid` voor het hele NTA-blok. Nu is dat het gat `nta_value_missing` op het pad van die waarde, bijvoorbeeld `ntaCalculation.declaredUses[0].monthlyKwh[3]`.
- **Gecombineerde buitenlucht en afvoerlucht uitgevinkt.** Het formulier wist nu de verborgen `outdoorAirHeatFraction` en `outdoorAirFractionReference`. Die gaven anders het blokkerende `outdoor_air_fraction_without_combined_source`. Een opgeslagen project met zulke restwaarden houdt dat gat tot het vakje opnieuw wordt aan- en uitgevinkt.

#### Formulieren en uitvoer

- Een ander koudeopwekkertype vervangt alleen de eerste opwekker; verdere opwekkers blijven staan. De keuzelijst kent nu ook de compressiekoelmachine met gasmotor (tabel 10.29) en de absorptiekoelmachine op een WKK (tabel 10.30).
- Waarschuwingen over opgegeven interne warmte wijzen nu naar de zone, bijvoorbeeld `spaceHeating.demand.internalGains.heatFluxWPerM2`. Een zone waarvan alle gebruiksfuncties woonfuncties zijn, krijgt `internal_gains_declared_residential`.
- De bijlage met interpretaties bevat nu ook hoofdstuk 5 (`building_performance::INTERPRETATIONS`).

### 3 oktober 2026 — energie per energiefunctie, rapportaanvullingen en opgegeven interne warmte

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
### 3 oktober 2026 — formulieren voor externe levering, serres, bevochtiging, koelsystemen en opgegeven stromen

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

### 3 oktober 2026 — invoerformulieren voor toestellen met productgegevens en de grondvloer

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
### 3 oktober 2026 — TOjuli, bijlage AA en de jaargemiddelde buitentemperatuur

#### Uitkomsten die veranderen

- **TOjuli bij onvoldoende koelcapaciteit (bijlage AA).** Was: geen TOjuli en geen Bbl-toets. Nu: TOjuli volgens 5.40, zoals zonder actieve koeling (§5.7.1, p. 114–115), met de waarschuwing `annex_aa_capacity_insufficient`.
- **`θ_e;avg;an`.** Overal 10,67 °C (D.4, p. 791) in plaats van het ongewogen gemiddelde 10,6717 °C in 7.14/7.15/7.73. Verschillen in de orde van 0,01 kWh/m².

#### Projecten die nu een gat krijgen

- **Zonwering als bewijs voor actieve koeling** (`criterion: shaded_glazing`). De raamgegevens moeten de verklaring dragen: meer dan 95 % van het beoordeelde glas met lamellen van tabel 7.4a/7.4b, `g_gl ≤ 0,4` of `F_sh;obst;juli < 0,67`. Anders `solar_limitation_not_met`; bij een onvolledige raaminventaris `window_inventory_incomplete`.
- **Afgewezen bewijs voor actieve koeling** staat nu ook als gat in `gaps` (pad `ntaCalculation.activeCooling…`). Eerder viel TOjuli stil weg.

#### Overig

- Een raam in bijlage AA mag met het projectraam-id worden opgegeven (`win-S`), naast de afgeleide naam `window:win-S`.
- Nieuw uitvoerveld `weightedConductanceWPerK` per ventilatiemaand: H_ve volgens 7.19/7.20 met b_v. `conductanceWPerK` blijft ρ·c·Σq zonder b_v.

### 3 oktober 2026 — R_se en R_si bij grenzen met een onverwarmde ruimte (8.4.2.1)

Volgens 8.4.2.1 (p. 266) wordt bij een grens met een onverwarmde ruimte R_se vervangen door de R_si van tabel C.2 (p. 778). Een projectconstructie draagt R_se = 0,04 (C.10, p. 777). Deze release regelt de gevallen waarin dat niet zo is.

#### Projecten die nu `invalid` worden

- **Horizontaal `internal` vlak naar een onverwarmde ruimte.** De warmtestroomrichting en daarmee R_si (0,10 omhoog, 0,17 omlaag) is niet bekend. Dit geeft nu `unheated_surface_direction_required`; kies het type vloer of dak. Een niet-horizontaal `internal` vlak telt als wand (R_si 0,13, tabel C.2 opmerking 3).
- **Ongeldige `exteriorSurfaceResistance`.** Een negatieve waarde, of een waarde van ten minste 1/U, geeft `construction_exterior_resistance_invalid`.

#### Uitkomsten die veranderen

- **Constructie zonder buitenlucht** (tabel C.2 opmerking 1). Wordt de kernberekening van een constructie toegepast terwijl "Buitenvlak in contact met lucht" uit staat, dan bewaart het project `exteriorSurfaceResistance: 0`. Naar een onverwarmde ruimte trekt de kern dan geen 0,04 meer af, maar telt hij alleen R_si op. Hetzelfde geldt voor de stilstaande-luchtwaarde achter een sterk geventileerde spouw (C.3.3). Bestaande constructies zonder dit veld blijven op 0,04.
- **VABI-import.** Een geïmporteerde constructie krijgt de R_si van haar elementtype: wand 0,13, dak 0,10, vloer 0,17. Dat was altijd 0,17. Een naam die op een wand en op een dak voorkomt, geeft nu twee constructies.
- **Eenvoudige U-berekening voor `internal`.** Die gebruikt nu R_se 0,04 in plaats van een tweede R_si van 0,13. Zo gaat de 8.4.2.1-correctie in de kern niet dubbel.

#### Overig

- **Het label van het vinkje is aangepast.** "Grenst aan buitenlucht" heet nu "Buitenvlak in contact met lucht". Het hoort ook aan te staan bij een grens met een onverwarmde ruimte.
- **`towardsUnheatedSpace` in het forfaitaire envelopmodel geldt alleen voor de basisopname.** Een U-waarde uit die route hoort niet in een projectconstructie thuis.
### 3 oktober 2026 — reviewcorrecties rekenzones in de utiliteitsopname

#### Opnames die nu `incomplete` worden

- **Zwembad met meerdere sportzones.** Hebben meerdere rekenzones sport en is de zwembadruimte niet per zone opgegeven, dan volgt `swimming_pool_zone_required` (p. 65). Eerder koos de opname de zone met de meeste sport.
- **Nieuwe controles op de waarden per zone:** `zone_installed_capacity_invalid`, `zone_installed_capacity_exceeds_total`, `zone_swimming_pool_area_invalid`, `zone_swimming_pool_areas_mismatch`, `zone_combined_without_system` en `zone_combined_areas_mismatch`.

#### Opnames met een andere uitkomst

- **Eén oppervlaktebron.** Bij meerdere zones komen de functiegroepen, het tapwater, `labelFunctions`, BACS en de totale A_g uit de zonesommen. Een afwijking binnen 0,05 m² gaf eerder `derived_input_rejected`; nu rekent de opname door.
- **Geïnstalleerde capaciteit per zone (p. 147).** Met `zones[i].installedCapacityDm3PerS` krijgt die zone haar eigen capaciteit; alleen de rest wordt naar A_g verdeeld.
- **Systeem E per zone (p. 145).** Met `zones[i].combined` krijgt alleen die zone een decentraal deel, met haar eigen oppervlakken.

Opnames zonder `zones` geven dezelfde invoer als voorheen.

### 3 oktober 2026 — hoofdstuk 8 na de herberekening

#### Projecten die nu `incomplete` worden

- **Kruipruimte of onverwarmde kelder met randisolatie.** Randisolatie (D.7/D.8, tabel D.1) hoort alleen bij een vloer op de grond: `ground_floor_edge_insulation_slab_only` op `ntaCalculation.groundFloors[i]`. Dit gaf eerder alleen de status `invalid`.
- **Kruipruimte én verwarmde kelder onder dezelfde vloer:** `ground_floor_below_and_heated_basement`. Ook dit gaf eerder alleen `invalid`.

#### Projecten met een andere uitkomst

- **Vloer boven een kruipruimte of onverwarmde kelder.** U_f rekent nu met R_se = 0,04 (8.43 → 8.2.2.2.1 → tabel C.2) in plaats van 0,17. H_g stijgt iets; Q_H;nd ongeveer +0,2 %.
- **Transmissie naar een onverwarmde ruimte.** H_D;zi,j;ztu gebruikt nu R_si aan de kant van de onverwarmde ruimte in plaats van R_se (8.4.2.1). U wordt omgerekend met 0,13 (wand), 0,10 (dak of plafond) of 0,17 (vloer). H_D;iu daalt iets; b_U en H_U veranderen mee.
- **Samengestelde constructie met een sterk geventileerde spouw.** Werd geweigerd en wordt nu berekend (C.3.3 in C.5 en C.6).
- **Voorbeeld tussenwoning:** het dak is nu 2 × 35,36 m² in plaats van 52 m². BENG 2 80,56 kWh/m²·jr, aandeel hernieuwbaar 22,3 %, label A+.

#### Nieuwe waarschuwingen (niet-blokkerend)

- `ground_floor_resistance_below_surface_resistance`: R_si + R_c van de vloer onder 0,17.
- `ground_floor_perimeter_implausible`: P groter dan 2·A/1 m + 2 m.
- `detailed_thermal_bridges_none_entered`: gedetailleerde methode zonder ψ-waarden naar buitenlucht.
- `sunroom_values_differ_from_unheated_space`: b_U of H_zi;ztu van een serre wijkt meer dan 10 % af van de onverwarmde ruimte met dezelfde id.
### 3 oktober 2026 — meerdere rekenzones in de utiliteitsopname (ISSO 75.1 §6.5)

#### Nieuwe invoer

- **`zones`** in de utiliteitsopname: twee of meer rekenzones met hun gebruiksfuncties. Een opname die met `calculation_zone_split_required` stopte, kan nu worden afgemaakt. Vlakken (`envelope.surfaces[].zoneId`) en verlichtingszones (`lighting[].zoneId`) krijgen hun zone; vlakken zonder zone worden naar A_g verdeeld.
- In de woningopname is `zoneId` op een vlak niet toegestaan: `surface_zone_not_in_dwelling_survey`.

#### Opnames die nu `incomplete` worden

Alleen opnames met `zones`:

- `zone_function_areas_mismatch`, `zone_function_area_invalid`, `zone_id_required`, `zone_id_duplicate`;
- `surface_zone_unknown`, `lighting_zone_calculation_zone_required`, `lighting_zone_calculation_zone_unknown`, `zone_lighting_area_mismatch`;
- `calculation_zone_criteria_not_met`: een zone die volgens afb. 6.6 zelf nog gesplitst moet worden.

#### Opnames met een andere uitkomst

Geen: zonder `zones` is de afleiding ongewijzigd (gecontroleerd op de drie utiliteitsfixtures: invoer, standaardwaarden en uitkomst identiek).

### 3 oktober 2026 — micro-WKK, PV en zonneboilers na de herberekening

#### Projecten die nu `invalid` worden

- **Micro-WKK met een totaalrendement boven 1,2.** Tabel 9.33 begrenst η_th + η_el per meetpunt op [0; 1,2]: `micro_chp_total_efficiency_invalid`.
- **Micro-WKK met tegenstrijdige elektrische meetwaarden.** Zijn P_el en η_el allebei opgegeven, dan moeten ze op 2 % na passen bij P_th·η_el/η_th: `micro_chp_electric_values_inconsistent`.

#### Projecten met een andere uitkomst

- **Micro-WKK boven vollast (9.66/16.15).** Warmte boven P_th;chp_100+sup_100·t levert geen eigen elektriciteit meer op. De invoer van dat overschot blijft geboekt als bijstookwarmte. Lagere opgewekte elektriciteit betekent een hogere primaire fossiele energie.
- **PV met een belemmering, precies tussen twee oriëntaties (17.3.7).** Bij een azimut van 22,5°, 67,5° enzovoort geldt nu per maand de hoogste belemmeringsfactor van de twee buren. Eerder gold altijd de rechtsom liggende buur.

#### Nieuwe waarschuwingen (de berekening loopt door)

- `micro_chp_capacity_exceeded`: de micro-WKK levert in een maand meer warmte dan zijn vollastvermogen (verwarming of tapwater).
- `solar_tested_backup_distribution_below_one`: geteste zonneboiler met geïntegreerde naverwarming en Σ f_dis < 1; 13.134 verhoogt dan de zonne-opbrengst.
- `pvt_without_thermal_part`, `pvt_without_electric_part` en `pvt_cover_inconsistent`: het elektrische en het thermische deel van een PVT-systeem passen niet bij elkaar.
### 3 oktober 2026 — reviewcorrecties utiliteitsopname (ISSO 75.1) en waterzijdig inregelen van koeling

#### Opnames met een andere uitkomst

- **Waterzijdig inregelen van koeling zonder verklaring (woning- en utiliteitsopname).** NTA-tabel 10.11 voetnoot a (p. 388) vraagt een verklaring volgens NEN-EN 14336. Zonder `cooling.balancingEvidenceReference` rekent statisch of dynamisch ingeregeld nu als niet ingeregeld: f_HB 1,15 in plaats van 1,00, en geen waterzijdige inregeling bij de afgifte. Opgeslagen opnames met "statisch" of "dynamisch" hebben deze verwijzing nog niet en krijgen dus een hoger pompenergiegebruik.
- **f_BACS bij meerdere verwarmingsopwekkers.** Het systeemvermogen van tabel 7.3 (p. 63) is nu de som van de hoofdopwekker en de extra opwekkers. Daarvoor telde alleen de installatiecapaciteit. Een ketel van 200 kW met een warmtepomp van 150 kW geeft nu f_BACS 1,05 in plaats van 1,0 als BACS onbekend is.
- **Directe expansie in de LBK.** Er zijn geen afgiftetoestellen in de ruimte meer: de afgifte is `other_or_unknown` zonder ventilatorconvectoren. Opgenomen split-binnendelen gaven ventilatorenergie voor toestellen die er niet zijn.
- **Zwembadruimte in 13.32a.** Het oppervlak van de zwembadruimte telt nu mee met de sportzalen (p. 65). `sportHallAreaM2` is voortaan het oppervlak zonder de zwembadruimte.
- **L_max van koelleidingen.** Een opgenomen `cooling.maxPipeLengthM` vervangt de forfaitaire L_max van 10.27 in de pompberekening.
- **Geïnstalleerde capaciteit met een zwembad.** Komt de capaciteit uit de passieve koeling, dan telt hij bij een zwembad in de zone nu ook als onbekend (p. 148).

#### Opnames die nu `incomplete` worden

- **Capaciteit dubbel opgegeven.** `ventilation.installedCapacityDm3PerS` naast `ventilation.passiveCooling.installedCapacityDm3PerS` geeft `installed_capacity_given_twice`. Voorheen won stilzwijgend de waarde van de passieve koeling.
- **Gasmotor tot en met 2 kW uit 2006 of eerder.** NTA-tabel 9.31 heeft hiervoor geen waarden. De opname meldt nu `gas_engine_small_old_no_table_row`; voorheen faalde de kern zonder duidelijke melding.
- **Zwembad plus sportzalen groter dan de sportfunctie.** Dit geeft `sport_hall_area_invalid`.

#### Nu toegestaan

- **Passieve koeling bij systeem E naast natuurlijke ventilatie** in de utiliteitsopname (p. 152). De woningopname stond dit al toe.
- **Bypasspercentage als bypass.** Een opgenomen bypasspercentage van 10 % of meer telt voor passieve koeling als aanwezige bypass.
- **Verborgen directe-expansieantwoord.** Bij een watergevoerd systeem wordt het genegeerd in plaats van `cooling_direct_expansion_not_water_based`. Het formulier wist het antwoord bij het aanvinken van "watergevoerd".

#### Inklapredenen

- Opgeslagen inklapredenen met een hernoemd pad of een gesplitste regel blijven gekoppeld (aliastabel), in plaats van een waarschuwing `collapse_reason_unmatched` te geven.

#### Rekenzones

- `calculation_zone_split_required` toont in het paneel nu de vervolgstap. Het ontwerp voor meerdere rekenzones in de opname staat in `docs/nta8800-basisopname.md`.

### 3 oktober 2026 — reviewcorrecties woningopname (ISSO 82.1)

#### Opnames die nu `incomplete` worden

- **Individuele gaswarmtepomp tot en met 25 kW.** De GWP-rijen van NTA-tabel 9.27 (p. 334) gelden alleen voor een collectieve gebouwinstallatie. Tabel 9.29 geldt voor collectieve installaties en voor meer dan 25 kW. Een individuele gasmotor- of gasabsorptiewarmtepomp tot en met 25 kW heeft dus geen forfaitaire rij en geeft nu `gas_heat_pump_individual_no_forfait_row`. Tot nu toe rekende de opname met de GWP-rijen van tabel 9.27.
- **Collectieve warmtepompbron zonder bewijs.** Een aangevinkte collectieve bron met een lege verwijzing gaf stilzwijgend een individuele bron, zonder tabel V.3 en zonder 9.62. Volgens ISSO 82.1 p. 111 blijkt een collectieve bron uit facturen of ontwerpgegevens. De opname meldt nu `collective_source_reference_required`.
- **Systeem E zonder WTW.** Bij §11.3.6 (p. 145) heeft het decentrale deel altijd WTW. Daarom kan de standaardwaarde "geen WTW" van tabel 11.9 hier niet gelden. Een gecombineerd systeem met een onbekende of ontbrekende wisselaar geeft `combined_requires_heat_recovery`.
- **Lege g-waarde bij zonwerend glas.** Een lege `solarControl.gValue` gaf een leesfout van de hele opname. Nu geeft hij `solar_control_g_invalid` op dat veld.

#### Opnames met een andere uitkomst

- **Bypass bij de opname afwezig.** De jaarregel van tabel 11.12 (p. 151–152) geldt alleen als de bypass of het bypasspercentage onbekend is. Een unit uit 2010 of later met `bypassPresent: false` kreeg ten onrechte 100 % bypass en krijgt nu 0 %.
- **Oppervlaktewater bij een collectieve installatie.** Volgens ISSO 82.1 p. 111 is oppervlaktewater een invoerkeuze bij een collectieve installatie. Dat geldt ook zonder collectieve bron. Zo'n installatie rekent nu met de rij oppervlaktewater van tabel 9.29 in plaats van met de rij bodem. Een gaswarmtepomp tot en met 25 kW in een collectieve installatie neemt de rij grondwater van tabel 9.27, want die tabel heeft geen rij oppervlaktewater.

#### Nu toegestaan

- **Passieve koeling bij systeem E naast natuurlijke ventilatie.** Volgens p. 152 kan passieve koeling voorkomen bij de systemen B tot en met E. Het decentrale deel moet dan wel een bypass hebben.

#### Formulier

- **Verborgen antwoorden worden gewist.** Kies je een andere bron of een ander toestel, dan wist het formulier de antwoorden die daarbij niet meer zichtbaar zijn: collectieve bron, grondwatersysteem, brontemperatuur, kwaliteitsverklaring en de brandstof van een stoomketel. Zo leveren ze geen `collective_source_water_based_only` of `local_heater_fuel_contradiction` meer op.
### 3 oktober 2026 — basisopname utiliteitsgebouwen volgens ISSO 75.1 (7e druk)

#### Opnames die nu `incomplete` worden

- **Splitsing in rekenzones (afb. 6.6 met tabel 6.4, p. 53–54).** Na het samenvoegen van p. 39–40 stopt de opname met `calculation_zone_split_required` wanneer de setpoints van de overgebleven functies meer dan 4 K verschillen, of wanneer bij ventilatietype A, B, C of E de ventilatiecapaciteit meer dan een factor 4 verschilt. Een voorbeeld is onderwijs naast een sportfunctie van meer dan 25 %. De uitzondering voor verblijfsgebieden in open verbinding staat in `openlyConnectedResidenceAreas`.
- **Gasmotor-koelmachine zonder elektrisch vermogen** (`gas_engine_power_required`, tabel 10.2).
- **Meer koudeopwekkers zonder vermogen** (`cooling_generator_capacity_required`, §10.3.2 met NTA 10.49).
- **Directe expansie in de LBK** zonder LBK of met "niet aangesloten" op de koelbatterij.

#### Opnames met een andere uitkomst

- **f_BACS uit de opgenomen vermogens (tabel 7.3, p. 62–63).** Zonder `bacs.systemPowerKw` beslist nu het opgenomen vermogen van verwarming en koeling. Een ketel van 350 kW in een gebouw kleiner dan 2.500 m² krijgt zo f_BACS 1,05 (eerder 1,0). Zijn alle systemen bekend en ten hoogste 290 kW, dan is het 1,0, ook boven 2.500 m².
- **Dynamisch ingeregelde koeldistributie** krijgt f_HB 1,0 (NTA-tabel 10.11) in plaats van 1,15. Dat geldt ook voor de koeling in de woningopname.
- **Collectieve verwarming.** De distributie leest nu de antwoorden van tabel 9.12 (leidingisolatie, isolatiejaar, appendages) en het eenpijpssysteem; eerder waren de leidingen altijd ongeïsoleerd.
- **Opgenomen koudeopwekkerpunten.** Opgegeven appendage-isolatie, koudemeters en leidinglengten gaan nu naar de kern.
- **Bronvermelding.** `derive_utility_input` geeft zelf al de ISSO 75.1-pagina's.

#### Nieuwe opties

- Fossiele brandstof op het perceel, oppervlak van sport- en zwemzalen, ruimte met zwembad.
- Koeling: gasmotor-koelmachine, meer opwekkers met prioriteit, directe expansie in ruimte of LBK, koudemeters, appendages, leidinglengten.
- Ventilatie: LUKA D en geen kanaal, "koude laden met LBK", decentrale WTW, isolatie en lengte van de buitenaansluiting, constant volumeregeling, gedeeltelijke bypass in procenten, geïnstalleerde capaciteit, systeem E en roosters met verwarmingslint.

### 3 oktober 2026 — basisopname woningen volgens ISSO 82.1 (7e druk met erratum)

#### Opnames die nu `incomplete` worden

- **Gesloten of verlaagd plafond in de woningopname.** `construction.closedOrSuspendedCeiling` komt uit ISSO 75.1. ISSO 82.1 tabel 7.4 (p. 62) kent alleen `lighterCeiling`. De woningopname meldt nu `closed_ceiling_not_in_dwelling_survey`.
- **Warmtepomp met een opgegeven klasse boven 70 °C.** Volgens tabel 9.9 en erratum §4 is dan een gecontroleerde verklaring nodig: `heating.heatPumpAbove70Declaration`, anders `heat_pump_above_70_requires_declaration`.
- **Bewijsstukken bij nieuwe opties.** Ventilatiesturing zonder bewijsstuk geeft `ventilation_controls_evidence_required`. Zonwerend glas zonder bron geeft `solar_control_evidence_required`.

#### Opnames met een andere uitkomst

- **Bypass bij een onbekend bypassaandeel (tabel 11.12, p. 151–152).** Het fabricagejaar van de WTW-unit gaat nu voor het bouwjaar. Een unit van vóór 2010 in een woning van 2010 of later krijgt dus geen 100 %-bypass meer, maar 70 % (bypass aanwezig) of 0 %.
- **Ketel bij klasse 70/50.** De gemiddelde ontwerptemperatuur is nu 65 °C in plaats van 60 °C, gelijk aan de distributieklasse 70/60. De uitkomst verandert niet, omdat alleen de grens van 50 °C telt.

#### Nieuwe opties

- Ventilatiesturing volgens de tabellen 11.4–11.6 zonder `declaredVariant`.
- Gecombineerd systeem E.
- Roosters met verwarmingslint.
- Woningpositie "dak + vloer".
- Zonwerend glas of folie met een g-waarde uit het product.
- Centrale of decentrale WTW.
### 3 oktober 2026 — verwarmingsopties in de basisopname (ISSO 82.1 hoofdstuk 9)

#### Nieuwe, aanvullende invoer

- **Opwekkers (tabel 9.3).** `boilerType: oil` (olieketel, conventioneel), `local_fired` (lokale gasverwarming, olieverwarming of stoomketel, met of zonder afvoer) en `gas_air_heater` (direct gestookte luchtverwarmers). De kern rekent ze met NTA-tabel 9.25.
- **Warmtepompen (tabel 9.6).** `drive` (elektrisch, gasmotor, gasabsorptie), de bronnen `heat_pump_panel` en `high_temperature`, `groundwaterSystem` (doublet of recirculatie), `collectiveSourceReference`, `sourceTemperatureC` en `sourceQualityDeclarationReference`.
- **Distributie (§9.4.2, tabel 9.12).** `distributionType` (tweepijps, eenpijps met aantal afgiftetoestellen, gerenoveerd eenpijps) en `pipeInsulation` (geïsoleerd, isolatiejaar, appendages en beugels).
- **Kern.** `pump.onePipeEmitterCount` telt de weerstand per afgiftetoestel van een eenpijpskring (tabel 9.21, p. 317). `forfait_heater` mag zonder `nominalPowerKw`; 9.92 rekent dan met de bovengrens t_on = t_mi.

#### Opnames met een andere uitkomst

- **Grondwaterwarmtepomp zonder brontemperatuur.** Deze rekent nu met de rij bodem (NTA p. 335). Eerder werd de rij grondwater gekozen, wat de kern zonder temperatuurbewijs afwees.
- **Collectieve warmtepomp in de woningopname.** Deze rekent nu met tabel 9.29 en krijgt hulpenergie volgens 9.91. Eerder volgde `table_scope_capacity_mismatch` en ontbrak de hulpenergie.
- **Warmtepomp boven 25 kW in de woningopname.** Deze rekent nu met tabel 9.29.

### 3 oktober 2026 — koelmethode 1 en bijlage Q

#### Projecten die nu `invalid` worden

- **Vijfde meetpunt NEN-EN 14825 (10.63, p. 408–409).** Een vijfde punt moet de deellast van punt C en de condensorintredetemperatuur van punt A hebben (tolerantie 0,5 % of 0,5 K), anders `cooling_en14825_fifth_point_conditions`. De testpunten staan in de volgorde A, B, C, D.
- **Zonder vijfde punt (10.64).** De benadering met Δϑ_corr = 0 klopt alleen als de verdamperuittrede bij A en C gelijk is. Verschillen ze meer dan 0,5 K, dan volgt `cooling_en14825_fifth_point_required`.
- **Methode 1 alleen voor modulerende opwekkers (§10.5.4, p. 401).** Een minimumvermogen gelijk aan of boven het nominale vermogen geeft `cooling_en14825_modulating_required`. Een minimumvermogen boven het nominale gaf eerder `cooling_performance_invalid`.
- **Lucht/luchtwarmtepomp volgens bijlage Q.** Met waterafgifte (radiatoren, vloerverwarming, ventilatorradiatoren) of hydraulische distributiegegevens volgt `annex_q_air_air_hydronic_chain`.

#### Projecten met een andere uitkomst

- **Bijlage Q met bijverwarming.** Dekt de warmtepomp elke temperatuurklasse van tabel Q.6, dan geldt nu F_H;gen = 1, ook als er bijverwarming is opgegeven. Eerder kreeg de bijverwarming door de afronding van tabel Q.6 een restaandeel van enkele honderdsten procent.

#### Nieuwe waarschuwing

- **Deellast boven 100 % (10.56/10.58).** Komt f_C;PL in een temperatuurklasse boven 100 %, dan extrapoleert de kubische functie van 10.63 buiten het meetbereik. De kern houdt de letterlijke uitkomst aan en meldt `cooling_part_load_above_full_load`. Een te klein toestel kan zo gunstiger uitkomen.

### 3 oktober 2026 — dynamische ramen in de projectroute (bijlage A)

- **Nieuw, aanvullend veld** `ntaCalculation.dynamicWindows`: bijlage A (p. 766–770) per buitenraam, methode A of B, met de correctiefactoren van stap 2. Opgeslagen projecten zonder dit veld houden dezelfde uitkomst.
- In het NTA-formulier is dit invoerbaar onder "Dynamische ramen (bijlage A)". Eerder kon het alleen via de kerninvoer.
- **Nieuwe gap** `window_dynamic_and_shading_exclusive`: een dynamisch raam samen met beweegbare zonwering (7.42) telt de zonwering dubbel (§A.2, p. 767). Neem de zonwering op in de toestanden.
- **Lege waarden** in bijlage A (g, U, wegingen, correctiefactoren) geven `dynamic_value_missing` op hun eigen pad. Eerder blokkeerde één leeg veld het hele NTA-blok.
- τ_vis en τ_sol staan niet meer in het formulier: hoofdstuk 14 gebruikt ze niet (14.38, 14.41). Opgeslagen waarden blijven bewaard.

### 3 oktober 2026 — validatieregels en herberekeningsbevindingen

#### Projecten die nu `incomplete` worden

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

#### Projecten met een andere uitkomst

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

#### Nieuwe waarschuwingen (de berekening loopt door)

- `declared_hot_water_efficiency_above_one`
- `declared_ventilation_below_required_flow`: alleen bij woningen, met de laagste f_ctrl·f_sys van tabel 11.5.
- `bacs_factor_without_capacity_evidence`
- `utility_open_ceiling_requires_evidence`
- `cooling_emission_loss_singular`
- `hot_water_circulation_defaults_low_efficiency`
- `lighting_large_office_group_without_office`: `largeOfficeGroup` in een zone zonder kantoorfunctie (§14.5.1, p. 664). Dit geeft F_o;D = 1, de minst gunstige waarde, en blokkeert dus niet.

Het rekenrapport toont zowel de projectmeldingen als de meldingen van de kern.

## Rekenkern 0.1.0 — 30 september 2026
<!-- kernel-version: 0.1.0 -->

Eerste versie van de rekenkern. Voor deze versie zijn geen afzonderlijke releasenotes bijgehouden.
