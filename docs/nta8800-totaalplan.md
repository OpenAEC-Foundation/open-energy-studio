# Open Energy Studio — totaalplan NTA 8800

**Onderzoeksdatum:** 28 september 2026; bronregister bijgewerkt op 29 september 2026  
**Doel:** een zelfstandig toetsbare NTA 8800-rekenbibliotheek in Open Energy Studio, met volledige invoer- en resultatenworkflow voor woningen, woongebouwen en utiliteitsbouw, gevolgd door BRL 9501-attestering.  
**Status:** onderzoeks- en uitvoeringsplan; geen conformiteitsverklaring.

## 1. Normatieve basis en afbakening

1. Leg één doelversie vast: **NTA 8800:2025+C1:2026**. NEN vermeldt dat deze de uitgave 2025 vervangt en dat de volledige norm kosteloos via een gratis NEN Connect-account in te zien is; de losse PDF staat op € 0,00 maar blijft auteursrechtelijk beschermd. Verkrijg een rechtmatige werkkopie voor formule-review en bewaar daarnaast historische versies als expliciete rekenprofielen. Bron: [NEN](https://www.nen.nl/nta-8800-2025-c1-2026-nl-349740).
2. Maak voor iedere formule, tabel, default, invoerregel en afronding een normregister: normversie, hoofdstuk/paragraaf, toepassingsgebied, codefunctie, test, eigenaar en reviewstatus. De normtekst blijft buiten de openbare repository tenzij hergebruik is toegestaan.
3. Richt attestering op het **NL-EPBD/EDR-attest Energieprestatie onder BRL 9501:2026**. Controleer de definitieve tekst en testset rechtstreeks bij InstallQ en de attesteringsinstelling. BRL 9500 en opnameprotocollen zijn nodig voor het werkproces van EP-adviseurs, maar zijn een apart certificeringstraject. Bronnen: [InstallQ BRL-overzicht](https://installq.nl/controllers/brl), [InstallQ attestering](https://installq.nl/energiediagnose-referentie), [RVO](https://www.rvo.nl/onderwerpen/wetten-en-regels-gebouwen/informatie-epa).
4. Het product toont vóór attestering uitsluitend **indicatieve** uitkomsten. Het mag geen officieel energielabel of wettelijke conformiteit suggereren. Voer registratie in EP-Online pas als aparte integratie uit zodra rollen, autorisatie, formaat en attest zijn bevestigd.
5. Houd **NTA 8800-energieprestatie** gescheiden van warmteverliesdimensionering volgens ISSO 51 / EN 12831. Een dimensioneringslast is geen BENG-energiebehoefte.

## 2. Nulmeting van de repositories

**Open Energy Studio (`main`, v0.1.6-alpha):** React/TypeScript/Tauri. De huidige calculator heeft een maandbalans, BENG 1/2/3 en TO-juli, maar gebruikt forfaitaire vereenvoudigingen. Het verwarmingsmodel kent `heat_pump_air` en `heat_pump_ground` met één invoer-COP; tapwater heeft één generiek warmtepomptype. De huidige validatiescripttekst noemt een vereenvoudigd model, drie Uniec-cases, toleranties tot 35% op BENG 2 en een slaaggrens van 60% van checks. De labelklassefunctie gebruikt één woontabel zonder expliciete gebouwfunctie. De reguliere Vitest-suite bevat vooral UI-tests; voor de rekenkern ontbreken onafhankelijke normcases op certificeringsniveau.

**Open Heatloss Studio (`master`):** bevat veel `nta8800-*`-Rust-crates, een BENG-orchestrator, analyses met paragraafverwijzingen en referentiecases. De BENG/Uniec-goldens zijn deels bewust `#[ignore]`, met gedocumenteerde afwijkingen. De verwarmingscrate heeft eveneens één generieke `HeatPump { scop }`, geen volledige warmtepompmethode. Gebruik de analyses, testopzet en onderbouwde tabellen als **referentie**; alle ontwikkeling, validatie en productintegratie vinden plaats in Open Energy Studio. Controleer licenties en auteurschap voordat code wordt hergebruikt. De PDF-opzet van Heatloss Studio is een nuttig voorbeeld voor rapportprovenance en automatische PDF-smoketests.

## 3. Doelarchitectuur

**Bibliotheekgrens.** Bouw een pure, versieerbare **Rust-crate `nta8800-core`** zonder React/Tauri-afhankelijkheden. Invoer: een gevalideerd `CalculationProject` met gebouw, rekenzones, installaties en bewijsbronnen. Uitvoer: `CalculationResult` met indicatoren, maand- en deelresultaten, aannames, waarschuwingen, bronverwijzingen en een reproduceerbare fingerprint van normversie en invoer. De Tauri-app, HTTP-API, MCP-server, import/export en rapportgenerator zijn adapters rond precies deze crate. Er komt geen tweede TypeScript-rekenkern voor nieuwe normfunctionaliteit.

**Datamodel.** Scheid (a) gebouwopname en bewijs, (b) normatieve rekeninvoer, (c) afgeleide tussenwaarden en (d) gepresenteerde resultaten. Gebruik eenheden en expliciete enums voor onbekend/niet-toepasselijk/forfaitair/productverklaring. Vermijd stilzwijgende defaults op verplichte velden. Iedere gekende default heeft een normbron en een zichtbare herkomst. Bewaar migraties voor bestaande `.oes`-projecten.

**Rekenketen.** Geometrie en zonering → transmissie/grond/koudebruggen → ventilatie/infiltratie → zoninstraling en beschaduwing → interne warmte en thermische massa → maandelijkse verwarmings- en koelbehoefte → afgifte/distributie/opwekking per energiedienst → PV/zonne-energie/overige opwekking → primaire en hernieuwbare energie → BENG, TO-juli, labelklasse en Bbl-toetsing. Geen indicator mag op een alternatieve verborgen formule worden berekend.

**Normprofielen.** `NormProfile` bevat NTA-versie, correctie- en interpretatiebesluiten, aangewezen regelgeving, klimaatgegevens, tabellen, labelgrenzen en afrondingsregels. De app toont de gebruikte versie bij ieder resultaat en in ieder rapport.

## 4. Volledige inhoudelijke dekking

| Werkstroom | Vereiste dekking | Acceptatiebewijs |
|---|---|---|
| Gebouw en geometrie | Woningen, woongebouwen, utiliteitsfuncties; thermische begrenzing, Ag/Als, rekenzones, aangrenzende ruimtes, onverwarmde ruimtes, grond, openingen, oriëntatie | Normcases voor oppervlakten, grenzen en gemengde functies |
| Schil | Rc/U/Uw, koudebruggen, ramen, deuren, zonwering, beschaduwing, thermische massa, luchtdichtheid | Tabel- en grenswaardetests plus onafhankelijke casussen |
| Klimaat en energievraag | Aangewezen klimaatdata, maandmethode, verwarming, koeling, zomercomfort/TO-juli | Deelpostvergelijking per maand en per zone |
| Ventilatie | Relevante systeemvarianten, WTW, bypass, vraagsturing, infiltratie, ventilatorenergie | Afzonderlijke systeemcases en massabalans |
| Verwarming en koeling | Afgifte, regeling, distributie, opslag, opwekkers, hulpenergie, meerdere opwekkers, actieve/passieve koeling | Referentiecases per tak en gecombineerde cases |
| Tapwater | Vraag, leiding- en opslagverliezen, circulatie, DWTW, zonneboiler, collectieve systemen, tapwaterwarmtepompen | Tapwaterdeelposten per systeemvariant |
| Overige energie | Utiliteitsverlichting, PV, zonnecollectoren, energiedragers, externe warmte/koude, hernieuwbare fracties | Deelposten, saldering en grensgevallen |
| Opslag en gebouwautomatisering | Elektriciteits- en warmteopslag, GACS-toepasselijkheid en aangetoonde functies volgens de doeluitgave | Actuele EDR-deelgevallen en bewijs van systeemgrenzen, waardering en anti-dubbeltelling |
| Zeer lage temperatuur bronnetten | Nettemperaturen, warmte-/koudelevering, collectieve grens en gekoppelde warmtepompen | Onafhankelijke gecombineerde net- en warmtepompcases |
| Einduitkomsten | BENG 1/2/3, TO-juli, labelklasse, toepasselijke Bbl-eisen, jaarlijks finaal energiegebruik, operationele broeikasgasemissies en overige verplichte velden | Identieke invoer geeft deterministische, herleidbare uitkomst en voldoet aan de actuele uitvoertests |

De matrix wordt per normparagraaf uitgewerkt. Een UI-keuze telt pas als ondersteund wanneer zowel normberekening als onafhankelijke test beschikbaar is.

## 5. Warmtepompen: volledige variantenmatrix

Maak geen lijst met alleen `lucht` en `bodem`. Modelleer **warmtebron, nuttige afgifte, bedrijfstoestand en systeemopbouw** afzonderlijk. De exacte normroute per combinatie wordt vóór implementatie aan de toepasselijke NTA-paragraaf en bijlagen gekoppeld; niet iedere productbenaming is een zelfstandige normcategorie.

| Dimensie | Te inventariseren varianten |
|---|---|
| Bron | Buitenlucht; ventilatie-/retourlucht; bodemlus; grondwater/WKO; oppervlaktewater; restwarmte of bronnet; externe warmtelevering waar toepasselijk |
| Afgifte/dienst | Lucht/lucht; lucht/water; water/water; ruimteverwarming; warm tapwater; gecombineerde ruimteverwarming en tapwater; reversibele verwarming/koeling; boosterwarmtepomp |
| Aandrijving/techniek | Elektrische compressie; thermisch aangedreven/absorptie waar de norm deze behandelt; productgebonden systeemverklaring |
| Systeemconfiguratie | Monovalent; mono-energetisch; bivalent/hybride met ketel; cascade; collectief; meerdere rekenzones; individuele naverwarming |
| Operationele details | Ontwerp- en afgiftetemperaturen, deellast, temperatuurafhankelijk rendement, ontdooien, elektrisch element, bronpomp, ventilator, regel- en stand-by-energie, opslag/distributie, bivalentiepunt, dekkingsaandeel, prioriteit tapwater, zomerbedrijf |

Voor iedere ondersteunde combinatie: definieer vereiste invoer, geldige forfaitaire route, route voor gecontroleerde kwaliteitsverklaring (bijvoorbeeld BCRG), formulevolgorde, maandelijkse energiestromen, primaire energie, hernieuwbare fractie en minimaal één onafhankelijke referentiecase. Maak een apart **dekkingsoverzicht** met `niet onderzocht`, `geïmplementeerd`, `intern gevalideerd`, `extern gevalideerd`, `geattesteerd`. Geen generieke SCOP invullen als die de verplichte normroute vervangt. RVO noemt onder andere hoogrendement-, booster- en absorptiewarmtepompen en het gebruik van kwaliteitsverklaringen: [RVO innovatieve opties BENG](https://www.rvo.nl/sites/default/files/2023-01/innovatieve-opties-beng.pdf).

Voor gasmotor- en gasabsorptiewarmtepompen bestaat nu een [begrensde concept-COP-selectie](nta8800-gaswarmtepomp-forfait-concept.md) uit tabellen 9.27 en 9.29. Tabel 9.27 is beperkt tot collectieve woningbouw tot en met 25 kW met expliciet aangeleverde `csource` voor bodem/grondwater. De tabelprioriteit en omzetting naar gas- en hulpenergiestromen zijn open verificatiepunten; deze varianten leveren nog geen BENG- of labelresultaat.

## 6. Test- en kwaliteitsstrategie

1. **Brontrouw:** elke formule en tabel krijgt een onafhankelijke review tegen de exacte normuitgave. Een test die alleen de huidige code-uitkomst als verwacht resultaat opslaat, bewijst geen conformiteit.
2. **Unitniveau:** tabelhoeken, interpolatie, eenheden, afronding, nul- en grensgevallen, bron-/afgiftecombinaties en foutmeldingen.
3. **Ketenniveau:** een woningcase per relevante installatievariant, daarna woongebouwen en utiliteitsfuncties. Vergelijk deelresultaten, niet alleen het label.
4. **Externe referentie:** officiële EDR/ISSO 54-testset en door de attesteringsinstelling voorgeschreven resultaten zijn leidend; commerciële softwarevergelijkingen zijn diagnostische tweede laag. De bestaande Uniec-fixtures met benaderde geometrie blijven daarvoor bruikbaar maar zijn geen certificeringsbewijs.
5. **Gates:** geen genegeerde tests in de aangevraagde attestscope, geen verruimde toleranties zonder normverklaring, nul ongedocumenteerde defaults, reproduceerbare CI-artifacts en een traceerbare wijzigingsgeschiedenis.
6. **Regressie:** iedere bug krijgt een minimaal invoergeval plus extern gemotiveerde verwachte uitkomst. Release bouwt dezelfde rekenkern voor desktop en testharnas. De actuele EDR-set wordt bij iedere wijziging en iedere marktversie opnieuw uitgevoerd; resultaten worden volgens de aangewezen BRL aan de certificerende instelling geleverd. Zie [de officiële wijzigingsregeling](https://zoek.officielebekendmakingen.nl/stcrt-2026-18113.html) en de [wijzigingsimpact](nta8800-2026-wijzigingen.md).

## 7. UI en rapportage

De UI volgt de opname- en rekenvolgorde: project → gebouw → zones en schil → ventilatie → verwarming/koeling → tapwater → opwekking → invoercontrole → berekenen → resultaten → rapport. Per veld: eenheid, herkomst, normversie, verplichtheid en geldigheidsmelding. Complexe installaties krijgen een begeleide editor met bron, afgifte, opwekker(s), dekkingsaandeel en hulpenergie in aparte stappen.

Het resultatenscherm toont: berekenstatus (`indicatief`/`gevalideerd`/`geattesteerd`, nooit automatisch opwaarderen), normprofiel, BENG/TO-juli/label, deelposten per dienst en maand, waarschuwingen, gekozen forfaits en bewijsreferenties. Rapport en export krijgen dezelfde status en fingerprint. Maak overzichtelijke validatiefouten vóór berekening, niet alleen een numerieke uitkomst achteraf.

## 8. Werkpakketten en volgorde

| Fase | Resultaat | Exitcriterium |
|---|---|---|
| 0. Baseline | Herhaalbare frontend- en desktop-devbuild, bestaande tests, builddocumentatie | Schone lokale build en CI |
| 1. Normregister | Volledige hoofdstuk-/tabelmatrix en doelversie, bronbestanden en testset bevestigd | Iedere berekende post heeft een bron en eigenaar |
| 2. Canoniek model | Gebouw, zones, systemen, bewijsbronnen, eenheden en migratie uit `.oes` | Round-trip en invoervalidatie werken |
| 3. Bouwfysische vraag | Geometrie, schil, ventilatie, zon, massa, verwarming/koeling, TO-juli | Onafhankelijke deelresultaten groen |
| 4. Installaties | Verwarming, volledige warmtepompmatrix, koeling, tapwater, verlichting, hulpenergie | Variantcases en hybride/collectieve cases groen |
| 5. Eindenergie en label | PV, primaire/hernieuwbare energie, BENG, Bbl en energielabel | End-to-end referentietests groen |
| 6. Productworkflow | Invoerassistent, validatie, resultaten, rapport, projectversies | Adviseur kan complete case reproduceren |
| 7. Attestering | Kwaliteitshandboek, release-/klachtenproces, testdossier, externe toets | BRL 9501-attest voor afgebakende scope |

Begin intern met **woningen** als eerste ontwikkel- en referentiescope. Ga er voor de formele attestering niet van uit dat een attest voor alleen woningen mogelijk is: de publiek beschikbare [ISSO 54-errata bij de uitgave 2020](https://documenten.isso.nl/s/UoQ7KSAd2EgbXfm-WrdPTayzG-pn1jE6/Errata%20publicatie%2054%2001-07-2020.pdf) vermeldt expliciet dat een rekenprogramma destijds zowel woningen/woongebouwen als utiliteitsgebouwen moest dekken. [InstallQ](https://installq.nl/controllers/brl) verwijst naar ISSO 54 (2024), terwijl [BouwZo](https://bouwzo.nl/search?Publisher=ISSO) een uitgave van 14 november 2025 vermeldt. De toepasselijke editie, inhoud en [BRL 9501:2026](https://bouwzo.nl/reader/publicatie/brl-9501/2026) moeten met de attesteringsinstelling worden bevestigd voordat de aanvraagomvang wordt vastgelegd. Zie het [bronnenregister](nta8800-bronnenregister.md). Bouw de generieke bibliotheek vanaf het begin voor beide gebruiksgebieden en leg per release vast welke scope werkelijk extern is getoetst.

## 9. Eerste uitvoeringsweek

1. Maak een reproduceerbare desktop-devbuild en leg de systeemeisen vast.
2. Zet de huidige resultaten en rapporten expliciet op `indicatief`.
3. Maak het normregister en de dekkingsmatrix aan, te beginnen met verwarming/warmtepompen en de huidige calculator.
4. Leg de officiële EDR-testset, de actuele BRL en eventuele aanvullende attesteringseisen vast met bron- en licentiegegevens.
5. Definieer de eerste woningfixture en leg zowel invoer als verwachte deelresultaten vast vóór wijziging van de rekenkern.
6. Ontwerp het nieuwe installatie-datamodel en een migratieroute; start pas daarna met de eerste warmtepomprekenroute.

## 10. Open beslissingen

- Welke **scope van het eerste softwareattest** schrijft de actuele ISSO 54/BRL 9501 voor? Controleer expliciet of woningen en utiliteit gezamenlijk moeten worden getoetst; EP-W basis/detail zijn adviseurs-/opnamescopes en mogen niet zonder bron als aparte softwareattesten worden behandeld.
- Wie levert de definitieve EDR-testresultaten en stemt de toetsaanpak af met de attesteringsinstelling?
- Welke product- en kwaliteitsverklaringsdata mogen in de openbare repository worden opgenomen?
- Welke historische normversies moeten vanaf de eerste release reproduceerbaar blijven?

Deze beslissingen blokkeren de baseline, normmatrix en eerste modelverbeteringen niet; ze bepalen wel de formele attestscope en testdata.

## 11. Uitvoeringsstand: Rust-kernel en interfaces

De nieuwe `crates/nta8800-core` is de enige beoogde plaats voor de toekomstige normberekening. Tauri roept deze crate aan voor projectvalidatie. `crates/nta8800-service` biedt een lokale HTTP-API en een stdio MCP-server, beide op dezelfde kernel. De service documenteert routes, tools en foutstatussen in zijn README. De warmtepomptaxonomie modelleert bron, afgifte, aandrijving, reversibiliteit, hybride/boosteropties en herkomst van prestatiegegevens. Daarnaast kunnen gemeten of verklaarde prestatiepunten per dienst, temperaturen, vermogens en energiedrager worden vastgelegd. Hulpcomponenten bewaren nominaal vermogen, energiedrager, dienst, bewijs en meetgrens, zodat de latere normroute dubbeltelling kan voorkomen. Systeemkoppelingen leggen bij-/reserveopwekkers, voorgeschakelde warmtepompen en gedeelde bronnen expliciet vast; de audit controleert doelen en cycli. Dit is een invoermodel; geen van deze gegevens activeert al een normatieve warmtepomprekenroute, schakelstrategie of jaarenergiepost.

De Rust-kernel levert momenteel structuur- en schilgeometrievalidatie en een samenvatting van bestaande `.oes`-projecten. De invoeraudit meldt verwarmings- en tapwaterwarmtepompen met alleen een generieke prestatiewaarde en controleert bron, afgifte en bewijsreferentie bij aanvullende `ntaHeatPump`-metadata. Er is nu ook een afzonderlijke Rust-diagnose voor directe buiten-transmissie (`A·U + L·Ψ + χ`) met API- en MCP-adapter. Een projectadapter levert alleen een beperkte buiten-diagnose wanneer schilvlakken en lineaire én puntkoudebruggen expliciet zijn geclassificeerd en de puntbruginventaris per zone is bevestigd; de UI kan deze gegevens opslaan. Beide vormen zijn uitsluitend rekenkundig getest en niet in de BENG-keten opgenomen. `calculationAvailable=false` en `attestStatus=unattested` blijven expliciet; de HTTP-BENG-route retourneert 501. De oudere TypeScript-uitkomsten blijven zichtbaar als indicatie en mogen niet als NTA 8800- of attestresultaat worden aangeduid. De volgende inhoudelijke stap is een normregister per formule/tabel, toepassing en correcties voor alle thermische grenzen en een onafhankelijk referentiegeval voor het eerste bouwfysische deelmodel. Pas daarna kan een Rust-berekenroute stapsgewijs worden vrijgegeven.

De UI toont de Rust-invoercontrole op het project- en resultatenscherm. Verwarmings- en tapwaterdialogen kunnen de warmtepompclassificatie vastleggen; de browserontwikkelserver gebruikt een lokale API-proxy en de desktop-app de Tauri-command. Zie `docs/nta8800-verificatiestatus.md` voor de actuele testresultaten en open verificatiepunten, `docs/nta8800-dekkingsregister.md` voor de status per warmtepompvariant en deelmodel, `docs/nta8800-referentieprotocol.md` voor het vereiste bewijs per rekenroute en `docs/nta8800-attestdossier.md` voor de formele test- en kwaliteitsvereisten.

Het openbare hoofdstuk-9-concept is inmiddels ook gebruikt voor een afzonderlijke Rust-generatorverdeling bij nieuwbouw, inclusief tabel 9.1, beta-/energiedeel uit tabel 9.23 en de restvraag uit formules 9.2–9.3. Een gekoppelde hybride diagnose voert de zo afgeleide warmtepompwarmte door naar de forfaitaire maandformule. De UI vraagt hiervoor expliciete nieuwbouwbevestiging, twaalf knooppuntmaanden en bewijs van vermogens, ketelcategorie en afgiftetemperatuur. Tabel 9.25 en formule 9.61 geven nu een voorlopige ketel-aardgasinput; voor individuele ketels geeft forfaitaire formule 9.85 ook elektrische hulpenergie. Met opgeslagen meetcomponenten wordt ook de individuele warmtepomphulpstroom uit de afgeleide maandenergie berekend. Dit blijft een conceptdeelmodel: de knooppuntvraag is aangeleverd, waakvlam en bronpomp/-ventilator ontbreken, renovatie en productgebonden afschakelgrenzen worden geweigerd, en er is geen onafhankelijke actuele EDR-verificatie. Zie [de generatorverdelingsanalyse](nta8800-generatorverdeling-concept.md).

## 12. Uitvoeringsstand 1 oktober 2026: rekenruggengraat

Er is een doorgaande, onverifieerde Rust-keten van `.oes`-project tot:

- BENG 1/2/3, TO-juli en de Bbl-toets;
- de A0-aanduiding en de indicatieve labelklasse.

De keten dekt hoofdstukken 7, 8, 9 (afgifte; opwekking met ketel, (hybride) warmtepomp, stadsverwarming, elektrisch en biomassa), 10 (koudeopwekking), 13 (tapwaterbehoefte), 16 (PV), 17 (klimaat, belemmering) en 5 (primaire en hernieuwbare energie), met meerdere rekenzones. Projectadapter, formulier, paneel en rekenrapport zijn beschikbaar in de app, via HTTP, MCP en Tauri.

Werkinstructie: [nta8800-werkinstructie.md](nta8800-werkinstructie.md). Stand en blokkades: [verificatiestatus](nta8800-verificatiestatus.md).

De volgende stappen vereisen externe input:

1. de normtekst NTA 8800:2025+C1:2026, voor de review van de transcripties en voor hoofdstuk 11, distributie 9.26 en de rendementstabellen;
2. de actuele EDR-testset met de uitkomsten uit bijlage 2;
3. afstemming over de attestscope met InstallQ.
