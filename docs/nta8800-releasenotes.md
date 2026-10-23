# NTA 8800-kern — releasenotes

Wijzigingen die de uitkomst of de status van bestaande, opgeslagen projecten veranderen. Normverwijzingen gaan naar NTA 8800:2025+C1:2026, met paragraaf-, formule- en paginanummers.

## Afspraken voor dit bestand

Dit bestand is de wijzigingsregistratie van de rekenkern (BRL 9501 §5.2, §6.2 en §6.4). `scripts/nta-kernel-version.mjs check` controleert het in de gate tegen `KERNEL_VERSION` in `crates/nta8800-core/Cargo.toml`.

- **Nieuwe items** komen bovenaan, onder "Onuitgebracht", als `### <datum> — <titel>`. Een item met de oude kop `## <datum> — <titel>` boven de eerste versiesectie telt ook als onuitgebracht.
- **Uitkomstwijziging.** Een item verandert uitkomsten of de status van opgeslagen projecten, tenzij de titel "(geen rekenwijziging)" vermeldt.
- **Versie ophogen.** Staat er onder "Onuitgebracht" een item dat uitkomsten verandert, dan moet `KERNEL_VERSION` minstens een MINOR-stap boven de laatste uitgave staan (zie [versiebeheer](nta8800-versiebeheer.md)). Eén ophoging per uitgave volstaat; latere items in dezelfde ronde vragen geen nieuwe.
- **Uitgave.** `scripts/release-nta.sh` zet de onuitgebrachte items onder `## Rekenkern X.Y.Z — <datum>` met de regel `<!-- kernel-version: X.Y.Z -->` eronder. Versies lopen af door het bestand.

## Onuitgebracht

### 22 oktober 2026 — TOjuli narekenbaar en NEN-vragen per uitgave (geen rekenwijziging)
- **TOjuli (§5.7, issue #8):** de uitkomst geeft per oriëntatie ook Q_ht, Q_gn en Q_rbl van stap B (`heatTransferJulyKwh`, `gainsJulyKwh`, `recoverableJulyKwh`) en per rekenzone de vaste waarden en zonetermen van 5.40 (`july`: t_juli, θ_e;juli, setpoint, a_C;red, interne warmtelast, H_C;ve, H_gr;an, warmtecapaciteit, gebruiksoppervlakte, minimale oppervlakte, grenswaarde). De resultatenpagina toont ze, met de rekenregel; TOjuli zelf verandert niet.
- **NEN-vragen:** [nta8800-vragen-nen.md](nta8800-vragen-nen.md) heeft een tabel *Stand per uitgave* met per vraag de status bij NEN en de lezing in de rekenkern; een test bewaakt dat elke vraag erin staat en dat de tabel bij de huidige rekenkernversie hoort.

### 22 oktober 2026 — Registratie: ingelezen gegevens bevestigen (issue #8)
- **Rekenkern.** `KERNEL_VERSION` 0.4.0 → 0.5.0; de handleiding is opnieuw gestempeld.
- **Registratiecontrole (BRL 9501 §4.3.1, p. 8):** een import uit UNIEC3 of VABI telt pas mee als de adviseur heeft bevestigd dat de ingelezen gegevens zijn gecontroleerd (`importLog[i].reviewedAt`). Zonder die bevestiging meldt de kern `import_not_reviewed` en is het dossier onvolledig; `dataImport.allReviewed` geeft aan of alles is bevestigd. Een opgeslagen project met een import zonder bevestiging is daardoor niet meer gereed voor registratie tot de adviseur bevestigt; berekende waarden veranderen niet.
- **App:** de registratiepagina heeft een blok *Ingelezen gegevens* met per import de knop **Ingelezen gegevens gecontroleerd** (en **Bevestiging intrekken**); het importverslag toont de bevestiging.

### 22 oktober 2026 — Automatisch programmaversienummer (geen rekenwijziging)
- **Versie ophogen:** `npm run bump` zonder argument kiest zelf het volgende nummer jaar.week.volgnummer (ISO-week van vandaag, na de bestaande tags van die week) en zet het ook in `package-lock.json` en, als `26.W.N`, in de MSI-versie (`bundle.windows.wix.version`). Daarmee is het laatste punt van issue #10 afgerond.

### 22 oktober 2026 — Opname: aansluiting douche-wtw en vlakken hernoemen (geen rekenwijziging)
- **Aansluiting douche-wtw (tabel 13.8):** de opname legt vast waarop de douchewarmteterugwinning is aangesloten (douchemengkraan én toestel 1,00, alleen mengkraan 0,85, alleen de inlaat van het toestel of gedeelde units 0,75). Zonder antwoord blijft het "niet vast te stellen" (0,75), zoals tot nu toe; opgeslagen opnamen rekenen gelijk.
- **Vlakken hernoemen:** een vlak in de opname kan een eigen naam krijgen ("Voorgevel"); ramen, deuren, dakramen, panelen, de vloer van een grondwand, de dichte delen en de fotokoppelingen gaan mee.

## Rekenkern 0.4.0 — 22 oktober 2026
<!-- kernel-version: 0.4.0 -->

### 15 oktober 2026 — Review 77: laatste routes en dekking per normdeel
- **Herberekening laatste routes:** berekende collectieve zonne-energie op een warmte- of tapwaternet (P.32/P.33 met 13.7.2.2), de gemeten route van bijlage P (P.6) en vliesgevels; `crates/nta8800-core/tests/remaining_independent.rs`.
- **Rekenwijziging (13.114, p. 595):** bij collectieve zonne-energie voor verwarming telt het opslagverlies van het back-updeel Q_H;bu;sto;ls mee in het energiegebruik van het zonnedeel (was weggelaten); klein effect.
- **Rekenwijziging (P.6.6.5.10, tabel 13.12, p. 579):** collectieve zonne-energie voor warm tapwater rekende met de instellingen voor verwarming (retour- en aanvoertemperatuur van het net). Nu de tapwaterinstellingen van tabel 13.12: ϑ_low 10 °C (koud water), ϑ_high en de back-upinstelling 65 °C (circulatie); alleen ϑ_ref wordt 70 °C. Voorbeeld: 14 990 in plaats van 17 540 kWh zonnewarmte.
- **Dekking hoofdstukken 5–8 en bijlagen A–I en AA:** 329 formules en 79 tabellen per stuk nagelopen (`docs/nta8800-dekking-ch5-8.md`, `crates/nta8800-core/tests/coverage_ch5_8_independent.rs`); geen afwijking. Niet uitgevoerd: 7.44 (kozijnfractie methode A; methode B 0,25 werkt), 8.50/8.51 (ψ_gr uit een numeriek model; opgeven mag) en D.17/D.18 (gewogen h en z over kelderwanddelen; één gemiddelde invoeren).
- **Dekking hoofdstukken 9–10 en bijlagen J–R** (`docs/nta8800-dekking-ch9-10.md`) en **11–17 en bijlagen S–Z** (`docs/nta8800-dekking-ch11-17.md`): per formule en tabel nagelopen, met tests in `coverage_ch9_10_independent.rs` en `coverage_ch11_17_independent.rs`.
- **Rekenwijziging (13.168, p. 634):** het stilstandsverlies van een gasboiler rekent met (ϑ_Wc;mn + 5 − ϑ_W;amb)/45; de +5 ontbrak. Daarnaast telt Q_W;S;ls nu als terugwinbaar verlies volgens 13.6.5 (p. 636); dat ontbrak in een verwarmde zone.
- **Rekenwijziging (p. 644):** een tapwatertoestel in serie met andere neemt c_W;gen van tabel 13.26/13.27 bij zijn eigen deel van de warmtebehoefte, niet bij de totale Q_W;dis;nren;an.
- **Rekenwijziging (12.1, p. 520):** bevochtiging rekent met q_V;SUP;dis;out van 11.87, niet met q_V;SUP;eff; verschil bij toevoersystemen (B).
- **Rekenwijziging (9.2.3.5, p. 292):** een buffervat in een verwarmde ruimte rekent met de genivelleerde setpointtemperatuur θ_int;set;H (7.9.4), niet met de rekentemperatuur.
- **Boeking (10.79, p. 420):** de pompenergie van vrije koeling en de regeneratietoeslag van 10.85 staan nu als hulpenergie, niet als E_C;el; de totale elektriciteit en de primaire energie blijven gelijk.
- **Vraag 28 aan NEN:** de kern rondt een opgegeven U-waarde niet opnieuw af vóór 8.1 (p. 228); met die afronding wijken dertien openbare gevallen af van hun gepubliceerde rapport.

### 15 oktober 2026 — Review 76: bijlagen C, E, I, N en P, numerieke route en opnamestandaarden onafhankelijk nagerekend
- **Herberekening:** hellende lagen (C.4.4), reflecterende spouwen en folies (C.3.5, tabellen C.3/C.4, E.13), de λ-omrekening van bijlage E (E.3–E.10, tabellen E.1–E.17, H.1), bijlage I (tabellen I.1–I.16, I.2/I.3, figuur I.4) en de numerieke route (8.5, 8.25–8.29, tabel 8.5); circa 1 210 grootheden (`crates/nta8800-core/tests/constructions_independent.rs`). Daarnaast circa 45 standaardwaarden van de basisopname tegen NTA 8800 gelegd (`crates/nta8800-core/tests/opname_defaults_independent.rs`).
- **Rekenwijziging (figuur I.4, p. 827):** "is er isolatie aanwezig? nee" leidt naar I.2.1.2 (tabel I.4, kolom zonder isolatie), ongeacht het bouwjaar. De kern kende alleen "geen of onbekend" en nam vanaf 1965 tabel I.5. Nieuw: `absent` in de forfaitaire constructie en `none` in de opname ("geen isolatie, vastgesteld"); "onbekend" blijft per bouwjaar I.2.1.2 of I.2.1.3. Voorbeeld: spouwgevel 1970 zonder isolatie R_c 0,35 (was 0,43). Opgeslagen invoer houdt de oude betekenis "onbekend".
- **Bijlagen N en P:** lokale verwarmers (alle typen en regelingen, N.37/N.61/N.66–N.72, tabellen N.20–N.30) en van de berekende route van bijlage P de leidingverliezen (P.13–P.18), hulpenergie (P.56–P.70), oppervlaktevraag (P.72–P.83), WKK (P.26–P.30), flexmodus, collectieve zonne-energie, tapwater (P.34–P.46) en koudeopwekkers (tabellen P.9/P.10); circa 940 grootheden (`crates/nta8800-core/tests/annex_np_independent.rs`).
- **Rekenwijziging (N.4.4.5.4 stap 10, p. 907):** de iteratie van β_mod stopte na één slag als de eerste β_mod 0 was (gemiddelde belasting onder het minimale vermogen), omdat de startwaarde 0 als vorige waarde gold. Nu wordt altijd minstens één keer herhaald; voorbeeld hoog-laag condenserende luchtverwarmer: E_in 24 837 in plaats van 25 480 kWh.
- **Vragen aan NEN 25–27:** N.69 (α_ch of α_ON bij modulatie), P.38 (K_CO2 met of zonder η_sto) en P.14 (teken van het verlies van een koudenet).
- **Rekenwijziging (7.17a, p. 173):** in de utiliteitsopname loopt de fictieve verticale leiding per toiletgroep bij onbekende leidingen door ⌊H/3⌋ bouwlagen (minimaal 1) en wordt H_p naar gebruiksoppervlakte over alle rekenzones verdeeld. De opname nam het opgegeven aantal bouwlagen en boekte alles in de eerste zone (winkel 1970, H = 6 m: 2 in plaats van 1 bouwlaag).

### 15 oktober 2026 — TOjuli bij een niet gehaald criterium van methode 3; vragen 20–24 aan NEN
- **Rekenwijziging (§5.7.1, p. 115):** voldoet de rekenzone niet aan het opgegeven criterium voor beperking van de zoninstraling (methode 3), dan blokkeerde de kern TOjuli met `solar_limitation_not_met`. De tekst zegt dat voor "alle andere systemen en situaties" TOjuli wordt bepaald; de kern berekent TOjuli nu, met `solar_limitation_not_met` als waarschuwing, zoals al bij een te kleine capaciteit volgens bijlage AA.
- **Vragen aan NEN 20–24:** uit review 75 de punten die een uitkomst meetbaar veranderen: M.25 met of zonder f_ctr;ls, η van geothermie in 5.48, afronding van de U naar een onverwarmde ruimte, U_x;T in D.15/D.16 en ΔU_for over kelderwanden. De overige lezingen staan in de verificatiestatus.

### 15 oktober 2026 — Review 75: grondverlies, schil, hoofdstuk 7, indicatoren en bijlagen M, O en P onafhankelijk nagerekend
- **Herberekening:** kruipruimte, onverwarmde en verwarmde kelder (8.38/8.39, 8.45), deels verdiepte vloeren, randisolatie (D.7–D.9), forfaitaire en gedetailleerde randkoudebruggen, H_g;an per maand, H_H/C;g;adj en de julibehandeling voor TOjuli (5.40) nagerekend vanaf de normpagina's; circa 1 980 grootheden in 22 gevallen (`crates/nta8800-core/tests/ground_independent.rs`), alle gelijk na één herstel.
- **Rekenwijziging (8.49, p. 263):** de ventilatie van een onverwarmde kelder rekent met ρ_a·c_a = 1211 J/(m³·K) zoals gedrukt, niet 1,205·1005 = 1211,025; H_g verandert 1–3·10⁻⁶ relatief.
- **Julibehandeling:** H_gr;an;juli en H_C;g;adj gaan per oriëntatie met het aandeel mee, zoals 5.7.2 stap B; dit verklaart de TOjuli-verschillen van ISSO 54 B03, B05, D02 en D04 niet.
- **Hoofdstuk 7 en 17.3:** belemmering (tabellen 17.3–17.15, ook collectoren en PV), beweegbare zonwering (7.42/7.43, tabellen 7.5–7.9), benuttingsfactoren (7.46–7.54), de maandbalans van 58 synthetische zones (g-waarden, lamellen en verstrooiend glas, dynamische beglazing van bijlage A, daklichten, opake zonwinst, interne warmte van woningen en alle utiliteitsfuncties, tabel 7.10 en bijlage B) en bijlage AA nagerekend; 88 710 grootheden (`crates/nta8800-core/tests/solar_independent.rs`), gelijk na één herstel.
- **Rekenwijziging (17.3.2 a en d, p. 698 en 701):** een zijbelemmering met relatieve breedte b_b van 3,73 m of meer valt onder situatie a (minimale belemmering), want situatie d vraagt b_b < 3,73 m. De kern nam de kolom b_b ≥ 1,0 van tabel 17.10/17.12; nu tabel 17.4 voor ramen en 1,00 (tabel 17.6) voor collectoren en PV. Alleen de warmtebalans en de zonne-energie veranderen; tabel 17.11 is in die kolommen al 1,00.
- **Schil (bijlage C, 8.2, 8.4, 7.30b, 5.7.2):** constructies uit lagen (luchtlagen, samengestelde lagen C.4–C.7, ΔU-correcties), ramen en deuren (8.14–8.23, tabellen 8.3/8.4, G.1, L.1/L.2), ΔU_for en forfaitaire ψ (tabel I.1), onverwarmde ruimten (b_U, R_se → R_si), serres en de TOjuli-verdeling per oriëntatie nagerekend; circa 230 grootheden (`crates/nta8800-core/tests/envelope_independent.rs`), gelijk na één herstel.
- **Rekenwijziging (5.7.2 stap A, p. 115):** een vloer boven buitenlucht of, in de forfaitaire route, boven een onverwarmde ruimte (helling 180°) telde bij TOjuli onder zuid. Horizontale elementen gaan naar rato over de oriëntaties; dat geldt nu ook voor vloeren (helling vanaf 175°).
- **Indicatoren en bijlagen:** hoofdstuk 12 (bevochtiging), de f_BACS-beslissing (§5.5.8), de elektriciteitsbalans en indicatoren van hoofdstuk 5 (5.9–5.27, opslag, CO2, EED), collectieve bronnen onder en vanaf 20 °C, gemeenschappelijke ruimten (6.6.4), renovatiestandaard en Bbl-grenzen, f_ls, de zoneringsregels, bijlage M, bijlage O en de berekende route van bijlage P nagerekend; circa 2 100 grootheden (`crates/nta8800-core/tests/indicators_independent.rs`), gelijk na twee herstellen.
- **Rekenwijziging (§5.7.1, p. 114):** "A_w kleiner dan 0,2 · A_g;tot" is strikt. Bij precies 0,2 · A_g (EPW301a: 19,20 m² op 96 m²) gaf het drijvendekommaproduct 19,200000000000003 de vrijstelling; nu wordt TOjuli berekend.
- **Rekenwijziging (bijlage O, O.1.5 en O.2.3.4, p. 924 en 930):** de gemiddelde aantijd t_a;gem en de gemiddelde belasting voldoen nu aan belasting · aantijd = 120 s, met de belasting lineair in de tijd tussen de punten van de curve (tabel O.2: 283,7 s en 0,4229 in plaats van 282,4 s en 0,4206, product 118,75 s). B, C en de hulpenergie van 9.85–9.88 veranderen 0,3–0,4 %.
- **Vraag 19 aan NEN:** in een rekenzone met meer gebruiksfuncties weegt de kern q_Oc · f_τ per gebruiksfunctie, zoals openbare gevallen H en K; de letterlijke lezing van 6.5.3 weegt beide apart.

### 15 oktober 2026 — Opname: q_v10 uit kwaliteitsborging (geen rekenwijziging)
- **Herkomst q_v10 (§11.2.5):** naast een meting (NEN 2686) legt de opname nu ook een q_v10 vast die binnen een kwaliteitsborgingsprocedure is vastgelegd; §11.2.5 schrijft voor dat die waarde wordt gebruikt. ISSO 54 EPWReal D05 geeft "qv,10-waarde gemeten: nee" met 0,62 dm³/(s·m²); met die waarde komen alle zes uitkomsten binnen de tolerantie. Zonder `basis` blijft het een meting, dus opgeslagen opnamen rekenen gelijk.
- **Bron van de afgeleide infiltratie:** de afgeleide invoer neemt de bron van de q_v10 zelf over (het meetrapport of de kwaliteitsborging, met de route), niet meer de bron van het ventilatiesysteem.
- **Vragen aan NEN hernummerd:** na samenvoegen met reviews 69 en 70 (vragen 13 en 14, bijlage Q) staan de vragen uit de BRL 9501-toetsset ISSO 54 nu als 15–18 (c_W;gen 15, verticale leidingen 16, hernieuwbare tapwaterwarmte 17, TOjuli utiliteit 18); de brief telt achttien vragen.

### 14 oktober 2026 — Review 74: documentatie bijgewerkt na de onafhankelijke herberekeningen (geen rekenwijziging)
- **Verificatiestatus:** bovenaan een overzicht van reviews 63–73 (wat per hoofdstuk, bijlage en uitgave is nagerekend, met aantallen grootheden) en wat niet onafhankelijk is nagerekend.
- **Overzicht kern 0.4.0:** de invoerafhankelijke wijzigingen van reviews 70 en 72 toegevoegd; de open interpretaties lopen nu tot vraag 14.
- **Overige documenten:** gereedheid, dekkingsregister, testprotocol en README noemen de herberekeningen en de testbestanden met onafhankelijk berekende waarden.

### 14 oktober 2026 — Review 73: overige schakelpunten van oudere uitgaven nagelopen (geen rekenwijziging)
- **Nagelopen:** alle schakelpunten die review 72 open liet, tegen de tekst van beide uitgaven. Tabel 13.4 van 2023 en tabel 7.10 van 2022 nu als kerntest met de waarden uit de tekst, plus een kerntest op de waarde per uitgave van de schakelpunten 7, 13, 29, 30/53, 37, 54, 59, 62, 73, 75 en 77 (`edition_switches_independent.rs`). Geen afwijking met de kern.
- **Opgemerkt:** in 2024 spreken de definitie bij (5.20) (bron onder 20 °C telt niet) en 9.6.3.1.3 (bronwarmte vanaf 15 °C) elkaar tegen; de kern volgt 9.6.3.1.3 met het interpretatiedocument. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 13 oktober 2026 — Utiliteitsopname: koeling per zone, 290 kW-antwoord, TOjuli 0
- **Koeling per zone (§10.2):** de utiliteitsopname kent `zoneIds` per koelsysteem en `additionalCoolingSystems`; de kern rekent dan met `coolingSystems`. ISSO 54 EPUReal B01: koudelevering in zone 1 en multi-split op elektriciteit in zone 2 (18 942 kWh, referentie circa 19 483).
- **f_BACS (tabel 7.3):** `bacs.above290Kw` legt het antwoord van het formulier vast ("geen installatie met vermogen > 290 kW") en gaat voor het vermogen.
- **TOjuli utiliteit (§5.7):** een utiliteitsgebouw registreert TOjuli 0, zoals de toetsset voor alle 59 EPU-deeltests (vraag 18); de waarden per oriëntatie blijven ter informatie.

### 13 oktober 2026 — ISSO 54 D-gevallen en utiliteit: vier rekenkernpunten
- **Hernieuwbare tapwaterwarmte (5.36):** de forfaitaire tapwaterwarmtepomp met c_W;gen boekt onder 2025+C1 E_W;gen;in · (1,4·c_source − 1) als hernieuwbaar, zoals de toetsset (vraag 17). EPWReal B01: RER 58,0 % (referentie 57,8 %), was 54,7 %.
- **ΔU_for alleen in H_D (8.3):** de basisopname gaf de forfaitaire koudebrugtoeslag mee in de U van de zonnewinsttermen 7.33/7.39; die rekenen nu met U_c en de toeslag gaat apart mee (`forfaitDeltaUWPerM2k`). TOjuli EPWReal B06 en B07 nu exact (1,99 en 2,30).
- **Tabel 9.28:** het bewijs voor de hoge rij mag de verklaring van het testrapport zijn dat de warmtepomp aan tabel 9.28 voldoet (`complianceStated`), met product- en rapportverwijzing maar zonder COP-waarden, zoals de ISSO 54-formulieren het vastleggen.
- **Waterzijdig inregelen verwarming:** de basisopname kent naast ja/nee nu `none`, `static` en `dynamic` ("dynamisch gebalanceerd", EPWReal B01); verwarming van B01 nu op 0,05 % van de referentie.

### 13 oktober 2026 — Opname: dubbele vlakcodes en ramen zonder vlak worden geweigerd
- **Schil (opname):** een vlakcode die twee keer voorkomt geeft nu `survey_surface_id_duplicate`, en een raam, deur, dakraam of paneel waarvan het vlak niet bestaat `opening_surface_unknown`. Tot nu toe trok de kern zo'n raam van beide vlakken af en telde het dubbel, of liet hij het zonder melding weg. Een opgeslagen opname met zo'n fout wordt nu geweigerd tot de code is hersteld; de app maakt geen dubbele codes meer.

### 13 oktober 2026 — ISSO 54-toets: c_W;gen op de forfaitaire tapwaterwarmtepomp en geen waakvlam bij een HR-ketel
- **Uitgangspunt:** waar de BRL 9501-toetsset ISSO 54 en de letterlijke tekst uiteenlopen, volgt de kern de toetsset en staat het punt als vraag aan NEN in de [vragenlijst](nta8800-vragen-nen.md) (vragen 15 en 16).
- **Tapwaterwarmtepomp (tabellen 13.25/13.27):** onder 2025+C1 krijgt de forfaitaire warmtepomp (1,4 en 1,4·c_source) weer c_W;gen van tabel 13.27, klasse 4 bij een onbekende klasse. ISSO 54 EPWReal B01: tapwater 2 790 kWh (referentie 2 806), BENG 2 73,65 (referentie 73,89); zonder correctie was het 1 339 kWh en 49,33. Oudere uitgaven rekenen zoals de openbare rapporten J en M zonder correctie (normversies, punt 80).
- **Waakvlam (9.6.2.1):** een HR 100-, HR 104-, HR 107- of waterstofketel zonder opgave krijgt in de opname geen waakvlam meer (regel `pilot_flame_unknown_absent_condensing`); een conventionele of VR-ketel houdt de standaard van ISSO 82.1. ISSO 54 EPWReal B03: BENG 2 158,15 (referentie 157,77), was 163,74.
- **Verticale leidingen (7.3.3):** ongewijzigd één leiding door alle bouwlagen, zoals ISSO 54 EPWReal B03; nu vastgelegd als vraag 16.
### 12 oktober 2026 — Review 72: schakelpunten van oudere uitgaven nagelopen; miscanthus/graswol en NEN-EN 13053 in 2022
- **Nagelopen:** de schakelpunten die review 71 niet raakte, tegen de tekst van elke uitgave. Elf schakelpunten (6, 12, 15, 16, 19, 26, 27, 38, 60, 74 en 76) nu ook als kerntest met met de hand berekende waarden (`edition_switches_independent.rs`); een groot deel van de rest tekstueel. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.
- **Miscanthus en graswol vóór 2025+C1:** tabel E.11 van 2020+A1 t/m 2024 heeft voor deze materialen geen waarde. De kern gaf ze de 2025+C1-waarde; nu `route_not_in_edition` op het materiaal. Invoer die al rekende verandert niet.
- **η_hr volgens NEN-EN 13053 in 2022 en 2020+A1:** werd geweigerd, maar mag volgens 2022 p. 479/482. Zonder de rij van tabel 11.17 (die pas in 2023 komt) geldt "onbekend/overig": f_rend;cond 0,10. Nieuwe route in een oudere uitgave; geen invoer die al rekende verandert.

### 12 oktober 2026 — Review 71: oudere uitgaven onafhankelijk nagerekend (geen rekenwijziging)
- **Herberekening:** de voorbeeldwoning, het voorbeeldkantoor en de 31 ventilatievarianten van review 65 in 2024, 2023, 2022 en 2020+A1 nagerekend met een eigen implementatie per uitgave (hoofdstukken 5, 7, 9, 11, 13, 14, 16, TOjuli en de vaste C1-run van 5.4.3). Per uitgave 14 639 grootheden vergeleken tegen 10⁻⁶ relatief (2025+C1: 14 640); alle gelijk.
- **Wat de uitgaven hier veranderen en dat is nagerekend:** K_CO2 van tabel 5.3 (2024), θ_e;argII van tabel 17.1, τ_argII van tabel 11.7, f_argII, de NEN 1087-factor 0,5, (11.77) zonder dakregel, ΔT_C;fan 0,7/1,5 K en de afgiftetabellen 9.2/9.3 (2023), en in 2020+A1 de afronding van K_pk op 5 W/m² (16.4) en A = 13,0 kWh in (9.85). Geen kernfout en geen nieuwe vraag aan NEN. Welke schakelpunten deze invoer niet raakt, staat in de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 70: laatste onderdelen van bijlage Q nagerekend; opgegeven tapwaterdebiet bij overventilatie niet onder het vereiste buitenluchtdebiet
- **Herberekening:** in 2025+C1 nagerekend vanaf de normpagina's: interval 5 (B < 0,15; tabel Q.2, Q.16), het forfaitaire bronpompvermogen van een water/waterwarmtepomp (Q.4.4), modulerende lucht/luchtwarmtepompen met de carnotbegrenzing (Q.12A/Q.12B, Q.73A/Q.73B, Q.74–Q.78) en de herberekening van de warmtevraag bij overventilatie door een afvoerluchtwarmtepomp (Q.5.3: Q.84, Q.90/Q.91, Q.96/Q.97) met het buitenluchtdebiet van 11.23/11.23a. 1 034 grootheden in zeven varianten vergeleken tegen 10⁻⁶ relatief; na het herstel hieronder alle gelijk.
- **Rekenwijziging (11.23, p. 449; 13.148/13.148a, p. 615):** een opgegeven tapwaterdebiet `ventilation.overventilation.hotWaterFlowM3PerH` lager dan het vereiste buitenluchtdebiet q_V;ODA;req (11.22) verlaagde het buitenluchtdebiet. De kern verhoogt zo'n waarde nu tot q_V;ODA;req van de warmtebehoefte, zoals 13.148/13.148a voorschrijven. Dit verandert alleen projecten met zo'n opgegeven tapwaterdebiet; de afgeleide debieten en alle referentiegevallen (openbare gevallen A–O, ISSO 54) blijven gelijk.
- **Vraag 14 aan NEN:** 13.8.2.4 en 11.23 bepalen de tapwatertijdfractie met 13.149, bijlage Q.5.2.2 geeft daarvoor Q.86–Q.88. De kern volgt 13.149. Vijf gevallen blijven als test in `crates/nta8800-core/tests/installations_independent.rs`. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 69: overige bijlage Q-warmtepompen onafhankelijk nagerekend (geen rekenwijziging)
- **Herberekening:** in 2025+C1 nagerekend vanaf de normpagina's (p. 1027–1069): brine/water en water/water met bronpomp (tabel Q.4, Q.44, Q.5–Q.8), alleen afvoerlucht (Q.41, Q.54–Q.56, Q.63–Q.65), gecombineerde buiten- en afvoerlucht (Q.43), lucht/lucht (Q.57–Q.59, Q.66–Q.68), een gasketel als bijverwarming, modulerende lucht/water- en brine/waterwarmtepompen (Q.2.6, Q.6, Q.71–Q.78, tabellen Q.1, Q.2, Q.12, Q.13 en Q.16) en de tijdfractie in bedrijf per maand (Q.90, Q.91, Q.93, tabel Q.17).
- **Uitkomst:** 2 214 grootheden in negen varianten vergeleken tegen 10⁻⁶ relatief; alle gelijk. Geen kernfout, geen gewijzigde uitkomst. Zeven varianten blijven als test in `crates/nta8800-core/tests/installations_independent.rs`.
- **Vraag 13 aan NEN:** de bronpomp van een brine/water- of water/waterwarmtepomp telt letterlijk twee keer: in η_H;gen;hp via Q.4 en nog eens als hulpenergie via 9.6.3 (p. 340). De kern volgt de tekst tot er een antwoord is. Niet nagerekend: interval 5, modulerende lucht/lucht met carnotbegrenzing, de herberekening bij overventilatie (Q.94–Q.97) en de tapwaterfractie (Q.86–Q.88). Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 68: koelmethode 1 en bijlage Q-warmtepompen onafhankelijk nagerekend (geen rekenwijziging)
- **Herberekening:** in 2025+C1 nagerekend vanaf de normpagina's: koelmethode 1 met NEN-EN 14825-deellastgegevens (10.53–10.64; met en zonder vijfde meetpunt, met een grens voor de condensorintrede) en aan/uit-lucht/waterwarmtepompen met productwaarden volgens bijlage Q (Q.1–Q.9, Q.29–Q.53, Q.60–Q.62, Q.69/Q.70, Q.81–Q.83, tabellen Q.3 en Q.5–Q.7, met 9.63), ook een te kleine warmtepomp met afschakeling en elektrische bijverwarming.
- **Uitkomst:** 492 grootheden vergeleken tegen 10⁻⁶ relatief; alle gelijk. Geen kernfout, geen gewijzigde uitkomst. Vier gevallen blijven als test in `crates/nta8800-core/tests/installations_independent.rs`. De lezingen van de kern zijn gevolgd: de afgedrukte f_t;tot-rij van tabel 10.18, alle gemeten condities in Q.49/Q.50, ϑ_evap;nom −10 °C voor lucht/water (Q.48) en F_H;gen = 1 als elke bin wordt gedekt. Niet nagerekend: modulerende warmtepompen, brine/water en water/water, afvoerlucht, lucht/lucht, de maandverdeling Q.90–Q.97 en hybride met bijlage Q. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 67: zonneboilers, koelmethode 2, gemeten booster en WKK methode 1 onafhankelijk nagerekend (geen rekenwijziging)
- **Herberekening:** in 2025+C1 nagerekend vanaf de normpagina's: zonneboilers voor tapwater (13.7.2.2; voorverwarmer en geïntegreerde naverwarming), koelmethode 2 voor een split-airco en een watergekoelde koelmachine op een droge koeler (10.5.5), de gemeten boosterwarmtepomp (bijlage W, met de correctie W.14) en micro-WKK methode 1 (9.66–9.83).
- **Uitkomst:** 304 grootheden vergeleken tegen 10⁻⁶ relatief; alle gelijk. Geen kernfout, geen gewijzigde uitkomst. Zes gevallen blijven als test in `crates/nta8800-core/tests/installations_independent.rs`. Opgemerkt: bij WKK methode 1 rekent tabel 9.35 met t_H;op van 9.32a, inclusief de pompfactor 0,10 voor een woning met een individuele installatie; de kern volgt de letterlijke tekst. Niet nagerekend: koelmethode 1 en bijlage Q-productwaarden. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 66: installatieroutes buiten de voorbeeldprojecten onafhankelijk nagerekend (geen rekenwijziging)
- **Herberekening:** in 2025+C1 nagerekend vanaf de normpagina's met 37 ISSO 54-deeltesten en 15 varianten: externe warmte met en zonder kwaliteitsverklaring (f_prac, EMGforf), biomassa, warmtepompen forfaitair en met gedeclareerde COP, hybride, meerdere opwekkers met prioriteit, WKK methode 2, elektrische verwarming, tapwatertoestellen met afleverset en voorraadvat (13.59, 13.60), douche-wtw, boosterwarmtepomp, koelopwekking methode 3 (tabel-EER, gedeclareerd, absorptie, externe koude, koeltoren) en PV met batterij.
- **Uitkomst:** 2 417 grootheden vergeleken tegen 10⁻⁶ relatief; alle gelijk. Geen kernfout, geen gewijzigde uitkomst. Vijftien gevallen blijven als test in `crates/nta8800-core/tests/installations_independent.rs`. Niet nagerekend: zonneboilers, bijlage Q-productwaarden, WKK methode 1, de gemeten booster (bijlage W) en koelmethoden 1 en 2. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 65: overige routes van hoofdstuk 11 onafhankelijk nagerekend (geen rekenwijziging)
- **Herberekening:** de routes die de voorbeeldprojecten niet bereiken, zijn met 29 synthetische ventilatie-invoeren nagerekend vanaf de normpagina's: ventilatieve koeling, enkelzijdig en dwars met halfopen sectoren (11.70–11.79), open verbrandingstoestellen (11.80–11.83, tabellen 11.11 en 11.12), luchtstroomzones vanaf 15 m (tabel 11.1, 11.6–11.10, tabel 11.3), de kruipruimterijen van tabel 11.1, roosterverwarming (11.123–11.128), vorstbeveiliging bij een woning met balansventilatie (tabel 11.16, 11.105/11.106) en de luchtbehandelingskast met batterijen (tabellen 11.15 en 11.19, f_lea;ahu, f_ins;ahu, 11.114–11.121 met c_a als gedrukt).
- **Uitkomst:** 12 008 grootheden vergeleken tegen 10⁻⁶ relatief; alle gelijk. Geen kernfout, geen gewijzigde uitkomst. Twaalf varianten blijven als test in `crates/nta8800-core/tests/ventilation_independent.rs`. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 64: hoofdstuk 11 onafhankelijk nagerekend (geen rekenwijziging)
- **Herberekening:** hoofdstuk 11 is voor de tussenwoning (C1) en het kantoor (D2 met wtw) in 2025+C1 opnieuw afgeleid vanaf de normpagina's, vanaf de afgeleide invoer van de kern. Vergeleken zijn de benodigde buitenluchtvolumestroom (11.22, 11.56, tabellen 11.5, 11.8, 11.9), infiltratie en spuien (11.68, 11.84, 11.85), het luchtstroommodel met de routine voor de interne referentiedruk (11.1–11.21), de toevoertemperatuur met wtw en bypass (11.99–11.108), H_ve met b_v (7.19, 7.20) en het ventilatorgebruik (11.132, 11.142), plus de run met vast systeem C1 (§5.4.2, §5.4.3) door hoofdstuk 7.
- **Uitkomst:** 1 041 grootheden vergeleken tegen 10⁻⁶ relatief; alle gelijk, ook BENG 1 van het kantoor (74,55) uit de eigen C1-run. Geen kernfout, geen gewijzigde uitkomst. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 63: voorbeeldprojecten onafhankelijk nagerekend (geen rekenwijziging)
- **Herberekening:** de tussenwoning en het kantoor uit `training-data/` zijn in 2025+C1 opnieuw nagerekend met een eigen implementatie vanaf de normpagina's, vanaf de afgeleide invoer van de kern. Vergeleken zijn transmissie en grond (8.2, 8.3, bijlage D), interne en zonnewinst (7.5, 7.6), de maandbalans met nivellering en onderbreking (7.1–7.79), afgifte, opwekking en hulpenergie (9.9–9.12a, 9.61, 9.85), tapwater met voorraadvat (13.15–13.26, 13.59), verlichting (14.7–14.14), PV (16.2–16.4b), primair fossiel en hernieuwbaar (§5.3) en TOjuli per oriëntatie (5.40–5.41).
- **Uitkomst:** 782 grootheden vergeleken tegen 10⁻⁶ relatief; alle gelijk. BENG 1 van het kantoor komt volgens §5.4 uit de run met vast systeem C1, zoals de kern rekent. Geen kernfout, geen gewijzigde uitkomst.
- **Niet opnieuw afgeleid:** hoofdstuk 11 (luchtstromen, H_ve, ventilatoren); die waarden komen van de kern. Zie de [verificatiestatus](nta8800-verificatiestatus.md), stand 12 oktober.

### 12 oktober 2026 — Review 62: de zes open ISSO 54-deeltesten opnieuw nagegaan (geen rekenwijziging)
- **EP-W204d en EP-W406l** (biomassa die niet aan bijlage R voldoet): ook in 2025+C1 geeft tabel 9.30 (p. 341) alleen rendementen voor toestellen die aan bijlage R voldoen, en methode 1 vraagt meetgegevens die de deeltest niet geeft. Geen route; de kern weigert terecht.
- **EP-W406j** (booster met één COP): 13.8.4.4 en bijlage W (2025+C1 p. 631–632, 1118–1120) vragen nog steeds meetgegevens bij twee brontemperaturen of het forfait zonder opgegeven COP. Geen route.
- **EP-W203q, EP-W406w en EP-W406x:** de bijlagedocumenten met de kwaliteitsverklaringen zijn nog steeds niet in bezit.
- De telling blijft 260 van de 266; zie [voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md).

### 12 oktober 2026 — Review 61: vrijgavegereedschap weigert een gate-log die geen bewijs is (geen rekenwijziging)
- **Mislukte of afgebroken gate:** `scripts/nta-gate-summary.mjs` geeft elke samenvatting een status (geslaagd, niet geslaagd, onvolledig) en markeert de stap waar de gate stopte. Een stap zonder tellingen heette eerder altijd "geslaagd", ook in een mislukte gate; een log met mislukte tests en toch de slotregel telt niet meer als geslaagd.
- **Vrijgave weigert:** `release-nta.sh` laat de samenvatting met `--require-pass` controleren en stopt bij een mislukte of onvolledige gate. Buiten een proefvrijgave stopt hij ook als een verplichte gatestap ontbreekt, en vooraf als `NTA_SKIP_MSRV` of `NTA_SKIP_KERNEL_E2E` aanstaat. Na de gate, het testsetdossier en de build moet de werkmap nog schoon zijn.
- **GitHub-releasetekst:** zegt bij een niet-geslaagde gate dat die niet slaagde, in plaats van "passed".
- **Gate-log:** `verify-nta.sh` meldt bij een fout "Gate FAILED (exit N) at step: …", zet "(failed, exit N)" achter die stap in de tijdentabel en houdt de afsluitcode. In de parallelle modus stopt hij bij een fout de hele procesboom van elke achtergrondpijplijn (cargo, rustc, npm), niet alleen de directe kinderen.
- **Kleurcodes:** ANSI-kleurcodes in de log (bijvoorbeeld met `FORCE_COLOR`) veranderen de tellingen niet meer.

### 12 oktober 2026 — Review 60: leverdocumenten halen hun aantallen uit de gate (geen rekenwijziging)
- **Gate-samenvatting:** `scripts/release-nta.sh` schrijft na de gate `gate-samenvatting.json` met per gatestap de Rust- en Vitest-aantallen en de duur, en per referentiesuite het aantal gevallen, vergeleken en zonder verwachting, gelezen uit `gate.log` en `reference-report.json` (`scripts/nta-gate-summary.mjs`). Het bestand staat in het manifest en in `SHA256SUMS`.
- **Testrapport:** krijgt een automatisch ingevulde sectie "Gate-uitvoer" met die tabel, en de niveaus n (app tegen de echte rekenkern) en o (formulierstanden en gewiste velden door de kern).
- **GitHub-releasetekst:** noemt onder Verification de aantallen uit de samenvatting in plaats van alleen "passed".
- **Geen lokale paden of valse "lokale wijzigingen":** de samenvatting noemt de stap van de referentiesuites zonder het pad op de bouwmachine, en een gekoppelde `node_modules` in een werkmap telt niet meer als wijziging, zodat het leveringsdocument niet ten onrechte "(met lokale wijzigingen)" achter de commit zet.
- **Testprotocol:** de verouderde aantallen (~950 kerntests, 1 300 tests in 139 bestanden) zijn bijgewerkt en verwijzen naar de samenvatting per vrijgave; de niveaus n en o en de duur van ongeveer 30 minuten staan erin.

### 12 oktober 2026 — Poort: tijd per stap en optioneel parallel (geen rekenwijziging)
- **Tijd per stap:** `scripts/verify-nta.sh` toont na elke stap de duur en aan het eind een overzicht, ook als een stap faalt. Meting op 12 oktober (koude build, 8 kernen): 1798 s in totaal, met als grootste stappen de kern op Rust 1.77.2 (426 s), de UI-tests (385 s), de kerntests (357 s) en de twee Tauri-stappen (247 s); zie het testprotocol §5.
- **Optioneel parallel (`NTA_GATE_PARALLEL=1`):** de Rust 1.77.2-run en de frontendbundel met de Tauri-stappen kunnen naast de kernstappen lopen en worden vóór de UI-tests opgehaald; een mislukte pijplijn laat de poort falen. Standaard staat dit uit: drie gelijktijdige cargo-builds liepen op deze machine (13 GB) tegen een geheugentekort aan. Zonder de variabele doet de poort dezelfde controles, alleen staat TypeScript nu vóór de Rust 1.77.2-run en de Tauri-stappen.

### 12 oktober 2026 — Review 59: codereview van reviews 55–58 en de overname, de statusbalk en de app-run tegen de echte kern (geen rekenwijziging)
- **Overnemen in projectmodel bij meer zones:** het warm tapwater van een opname dient het hele gebouw (13.20: de functieoppervlakken samen ten hoogste A_g van het gebouw). Bij een project met meer rekenzones schaalde de overname de tapwateroppervlakken naar de eerste zone, zodat de tapwatervraag te laag uitviel. Ze volgen nu de A_g van alle zones samen; de verlichting (14.4) en bevochtigers blijven bij de zone. Bij één zone verandert niets. Test: `src/__tests__/review59-takeover-building-area.test.ts`.
- **Poort en schermafdrukken met `CARGO_TARGET_DIR`:** de stap "app against the real kernel" en `scripts/manual-screenshots/` zochten `kernel_once` altijd in `crates/nta8800-service/target`; met een gezette `CARGO_TARGET_DIR` faalde de poort. Ze zoeken het programma nu waar cargo het neerzet; een mislukte start noemt de oorzaak. `gate-order.test.ts` bewaakt dit.
- **Nagelopen zonder gebrek:** de keuzes die in review 58 geen "—" meer tonen (de bijbehorende kernvelden zijn verplicht; waar "geen" een keuze is, is het een eigen optie), het weglaten van vervolgpunten van een geweigerd ventilatie- of verlichtingsblok (alleen zolang dat blok zelf een open punt heeft), de BACS- en forfaitpaden, het wissen van de BACS-inventaris bij overname (zichtbaar in de voorbeschouwing, ongedaan te maken), `kernel_once` (leest alleen stdin, schrijft geen bestanden) en de versiechip in de statusbalk.

### 12 oktober 2026 — Review 58: één handeling, één open punt bij het veld dat de adviseur wijzigde (geen rekenwijziging)
- **Bewaking:** een nieuwe veegtest (`src/__tests__/nta-clear-field-sweep.test.tsx`) wist elk veld dat een adviseur kan leegmaken (getal, keuze met "—", tekst) apart, in elke NTA-sectie van de voorbeeldkantoor, de voorbeeldrijwoning en openbaar geval N, via de eigen wijzigfunctie van het formulier. De kern (`every_cleared_field_is_one_gap_at_that_field` in `tests/project_form_guard.rs`, in de poort) eist dat elk nieuw open punt op of onder dat veld ligt, en de UI-test dat **Ga naar** voor dat veld naar de pagina van de sectie leidt. 262 velden in ongeveer 15 s.
- **Gevonden en verholpen:**
  - **"—" in een keuze met varianten koos stil een variant.** Bij onder meer de interne warmtelast, de bepaling van het piekvermogen van PV, de infiltratie, het ventilatiesysteem, het WTW-rendement, bypass, toevoerkanaalisolatie, ventilatoren, distributieroute, opslagverlies, verlichting (vermogen, parasitair, daglicht, armaturen), belemmering en de afgiftesoort van 2023 schreef "—" het sjabloon van een andere variant, zodat het open punt bij een ander veld uitkwam. Zulke keuzes (`SelectField required`) bieden geen "—" meer zodra er een waarde is.
  - **Een geweigerd ventilatie- of verlichtingsblok gaf een tweede open punt** (`ventilation_flow_required` bij `ventilationFlows`, `lighting_gain_requires_chapter_14` bij de interne warmtelast). Heeft dat blok zelf een open punt, dan vervalt het gevolgpunt.
  - **De bron van de oppervlakte** (`areaSourceReference`) werd bij `zones[0]` gemeld; nu bij het veld zelf.
  - **GBS-inventarisatie:** een lege id, bron of opwekkergegeven gaf alleen `bacs_factor_undetermined` op het hele blok; nu noemt de kern het veld (`source_required`, `generator_id_required_or_duplicate` enz.). Het samenvattende punt blijft wanneer geen veld de oorzaak is.
  - **Forfaitaire warmtepomp (2020+A1):** een gewist installatiejaar naast zijn bron gaf `forfait_lookup_invalid` op de opwekker en een melding bij de bron; nu één punt, `installation_year_reference_without_year`, bij het jaar. De samenvatting `forfait_lookup_invalid` verschijnt alleen nog als de tabelopzoeking zelf geen veld noemt (ook bij de hybride warmtepomp).
  - **Belemmeringsfactor van PV:** wissen schreef een lege lijst, die de kern zonder belemmeringssituatie weigert (`pv_obstruction_factor_invalid` op de lijst). Nu blijft het veld leeg staan en meldt de kern het daar; met een situatie vervalt de factor zoals voorheen. Een nieuw PV-systeem in bijlage P begint ook met een leeg veld in plaats van een lege lijst.
- **Bewust:** een gewist nominaal opwekkervermogen in de GBS-inventarisatie telt volgens §5.5.8 als boven 290 kW, zodat het systeem zijn GBS-klassen vraagt (`bacs_evidence_incomplete`); dat staat als bewuste uitzondering in de test. De keuze "eigen F_c" (lege waarde met eigen tekst) bij zonwering is een gewone keuze, geen wissen.
- **Tests:** `a_cleared_field_is_named_where_it_was_cleared` in de kern; `refused_routes_are_project_gaps` en de overloop-test van de forfaitaire warmtepomp volgen de nieuwe regel. Volledige invoer rekent precies als voorheen.

### 12 oktober 2026 — Eén open punt voor een gewiste instelwaarde (geen rekenwijziging)
- **Wat er mis was:** het formulier neemt de instelwaarden van de zone over in de ventilatie van hoofdstuk 11 (`ventilation.heatingSetpointC`, `ventilation.coolingSetpointC`), en de kern eist dat die gelijk zijn (`ventilation_setpoint_mismatch`). Wie **Instelwaarde verwarming** of **Instelwaarde koeling** wiste, kreeg daardoor twee open punten voor één handeling, waarvan één naar een veld dat het formulier niet toont.
- **Nu:** staat de instelwaarde zelf ook leeg, dan noemt de kern alleen `setpoints.heatingC` of `setpoints.coolingC`, met **Ga naar** naar het gewiste veld. Een lege kopie naast een ingevulde instelwaarde blijft een open punt. Een volledige invoer rekent precies als voorheen.
- **Tests:** `a_cleared_setpoint_is_one_gap_not_two` in de kern; de app-run met de echte kern eist nu één open punt na het wissen van de instelwaarde.

### 12 oktober 2026 — Review 57: Overnemen in projectmodel volledig voor alle voorbeeldopnamen (geen rekenwijziging)
- **Nieuwe controle over alle zes voorbeeldopnamen** (drie woningen, drie utiliteitsgebouwen). `crates/nta8800-core/tests/survey_takeover_completeness.rs` legt de NTA-invoer vast die de kern uit elke opname afleidt (`training-data/nta8800-survey-takeover-derived.json`). `src/__tests__/review57-takeover-completeness.test.ts` eist dat elk onderdeel daarvan wordt overgenomen of met reden in `NOT_TAKEN_OVER` staat, en legt vast wat het overnemen schrijft, in een leeg concept en in het concept van het voorbeeldproject, zoals **Toepassen** het aan de kern geeft (`training-data/nta8800-survey-takeover-written.json`). De Rust-test rekent dat door via de projectroute en controleert:
  - dat er geen open punt overblijft op een overgenomen onderdeel;
  - dat de projectroute, met de geometrie van de opname teruggezet, dezelfde invoer afleidt als de opname;
  - dat beide routes dan dezelfde BENG 1–3 en labelklasse geven.
- **Gevonden en opgelost:**
  - **Bevochtiging** (`spaceHeating.humidifiers`, de stoombevochtiger van de winkel uit 1970) ging niet mee.
  - **Bouwjaar** ging niet mee; het is nodig voor de Standaard voor woningisolatie (§5.3.2). Ook de **label- en Bbl-functie** gaan nu mee als de opname die geeft; een woningopname zonder labelfunctie laat die van het project staan.
  - **Verlichting en bevochtiging** hielden de zone van de opname (`utiliteit`), en de verlichtings- en tapwateroppervlakten bleven die van de opname. Bij een utiliteitsopname weigerde de kern het project daardoor (`lighting_gain_requires_chapter_14`, `lighting_zone_areas_invalid`, `hot_water_function_areas_invalid`, `hot_water_served_area_exceeds_building`). Ze krijgen nu de eerste projectzone, en de oppervlakten worden naar verhouding omgerekend naar die zone, zoals de ventilatie bij **Toepassen** al deed.
  - **BACS:** een eigen BACS-inventaris van het project (§5.5.8) ging vóór de f_BACS uit de opname, zodat de winkel uit 1970 met 1,00 rekende in plaats van 1,05. Het overnemen maakt die inventaris nu leeg; die hoort bij de installaties die het overnemen vervangt.
- **Verwachte verschillen** tussen beide routes, alle geometrie: oppervlakten, verliesoppervlak, vlakken en ramen, de zone en de oppervlakten van ventilatie, verlichting en tapwater. Verder blijft een label- of Bbl-functie die de opname niet geeft die van het project. De NTA 8800-uitgave en de vraag naar de vergunningsaanvraag na 29 mei 2026 blijven die van het project.
- Handleiding hoofdstuk 2 noemt wat nu meegaat.

### 12 oktober 2026 — Adviseursstappen doorlopen met de echte rekenkern; basisopname neemt het verdeelsysteem over (geen rekenwijziging)
- **Vijf stappen tegen de echte rekenkern:** `src/__tests__/real-kernel-e2e.test.tsx` doorloopt nu ook de basisopname (doorrekenen, overnemen in het projectmodel, project opnieuw berekend), het maatwerkadvies (de tabel met varianten, inclusief TO<sub>juli</sub> en warmte), de wissel naar 2024 en terug (de vergelijking van de uitgaven in het venster), het rapport en het EP-Online-gegevensoverzicht (BENG-waarden, rekenkernversie en uitgave, ook in het dossier-ZIP) en een leeg verplicht veld (het open punt van de rekenkern met werkende **Ga naar**). Elke stap vergelijkt wat het scherm toont met het antwoord van `kernel_once` op precies het verzoek dat het programma stuurde. Het hele bestand duurt ongeveer 55 seconden.
- **Gevonden en opgelost: verdeelsysteem bij het overnemen van de basisopname.** De opname leidt bij een berekende verdeling (9.4) ook het verdeelsysteem af (`spaceHeating.distributionSystem`), maar *Overnemen in projectmodel* nam dat niet over. Na het overnemen van bijvoorbeeld de voorbeeldopname uit 1930 weigerde de rekenkern het project met `distribution_system_required`. Het verdeelsysteem (en het aantal identieke installaties, als de opname dat afleidt) gaat nu mee. Test: `src/__tests__/survey-takeover-distribution.test.ts`. De berekening zelf verandert niet; alleen de overname vult nu het blok dat de kern al vroeg.

### 12 oktober 2026 — Statusbalk toont rekenkern en uitgave bij elke vensterbreedte; programma doorgerekend met de echte rekenkern in de poort (geen rekenwijziging)
- **Statusbalk:** de versie van de rekenkern en de uitgave van NTA 8800 stonden alleen in een venster van minstens 1360 pixels breed in de statusbalk, en pas na het antwoord van de rekenkern. Ze staan er nu bij elke breedte: compact ("Kern 0.4.0 · NTA 8800 2025+C1"), in een breed venster met de volledige naam van de uitgave. Voor het antwoord van de rekenkern staat de uitgave van het project er al. De zin in hoofdstuk 0 van de handleiding beschrijft dit nu precies, en de schermafdrukken van het hele venster zijn opnieuw gemaakt. Test: `src/__tests__/status-bar-version.test.tsx`.
- **Programma tegen de echte rekenkern:** `src/__tests__/real-kernel-e2e.test.tsx` opent het voorbeeld rijwoning en kantoor in het volledige programma en laat elke aanroep naar `/api` beantwoorden door `kernel_once` (zonder API-server). Het controleert dat de statusbalk de BENG-waarden en de versie van de rekenkern toont, en dat een wijziging (een overstek boven het zuidraam) het resultaat als verouderd markeert tot het nieuwe antwoord er is. `scripts/verify-nta.sh` bouwt `kernel_once` en draait deze test na de UI-tests; `NTA_SKIP_KERNEL_E2E=1` slaat hem over. Een gewone `vitest run` slaat het bestand over zolang `OES_KERNEL_ONCE` niet is gezet.

### 12 oktober 2026 — Schermafdrukken van het hele venster opnieuw, met een vaste opnameopstelling (geen rekenwijziging)
- **Opnieuw gemaakt:** de acht afbeeldingen van het hele venster (`welkom.png`, `projectoverzicht.png`, `verwarming.png`, `palet.png`, `controle.png`, `resultaten.png`, `rapport.png`, `registratie.png`, hoofdstuk 0). Ze tonen nu programmaversie 0.2.0 in de titelbalk in plaats van 0.1.6-alpha, en de huidige pagina's (onder meer de knop **Handleiding** en het blok *Algemeen* met de uitgave). Het punt "Bekend, niet aangepast" bij review 56 is daarmee opgelost.
- **Opnameopstelling in de repository:** `scripts/manual-screenshots/` maakt deze afbeeldingen opnieuw met `node scripts/manual-screenshots/capture.mjs`. Het start een tijdelijke lokale vite-server op een vrije poort, laat Firefox headless het echte programma fotograferen en stopt de server weer. De aanroepen van het programma naar de rekenkern beantwoordt het in hetzelfde proces met het nieuwe voorbeeld `kernel_once` van `crates/nta8800-service`; er wordt geen API-server gestart. De README noemt per afbeelding hoe ze gemaakt is.

### 11 oktober 2026 — Review 56: schermafdrukken van de handleiding bij de wijzigingen van reviews 35–55 (geen rekenwijziging)
- **Opnieuw gemaakt:** `overnemen-vloeren.png` (hoofdstuk 2). Het venster *Opname overnemen in het projectmodel* toont sinds de vervolgstappen van review 53 ook het deel *Koudebrugmethode van het gebouw*; de oude afbeelding liet dat niet zien.
- **Nieuw, in hoofdstuk 3:** de keuze *Lineaire koudebruggen van het gebouw* (`koudebrugmethode.png`), twee rekenzones die één serre delen met de melding *Gedeelde serre, ook bij …* (`gedeelde-serre.png`), een verwarmde kelder met de diepte per wanddeel (`kelder-wanddelen.png`) en een zwak geventileerde smalle luchtlaag in *Constructie bewerken* (`smalle-luchtlaag.png`).
- **Nagelopen, ongewijzigd:** de andere 24 afbeeldingen tonen geen onderdeel dat in reviews 35–55 is veranderd. De vetgedrukte knop- en veldnamen in de hoofdstukken komen overeen met de huidige Nederlandse teksten van het programma.
- **Bekend, niet aangepast:** de afbeeldingen van het hele venster (`welkom.png`, `projectoverzicht.png`, `controle.png`, `resultaten.png`, `rapport.png`, `registratie.png` en andere) tonen in de titelbalk nog programmaversie 0.1.6-alpha. Die opnieuw maken vraagt het hele programma met een draaiende rekenkern; dat kon hier niet.
- Zoals de eerdere afbeeldingen zijn ze gemaakt met een tijdelijke lokale ontwikkelserver die het onderdeel los toont, en `firefox --headless --screenshot`; de rekenkern is daarvoor niet gestart.

### 11 oktober 2026 — Review 55: robuustheidsbewaking op de routes van reviews 40–54 (geen rekenwijziging)
De robuustheidsfuzz (`tests/robustness.rs`), de numerieke fuzz (`tests/numeric_fuzz.rs`), de optiedekking (`tests/option_coverage.rs`) en de fuzz van de zoektocht naar lege velden (`blank_fuzz`) rekenden alleen met voorbeelden zonder de invoer die reviews 40–54 toevoegden. Ze krijgen nu invoer op die routes (`tests/support/new_inputs.rs`): twee zones met een gedeelde serre (7.38), een collectieve installatie met afleverset en twee identieke toestellen, een bovenwoning met de forfaitaire koudebrugmethode van het gebouw, een stoombevochtiger op olie bij een ketel voor 800 m², TOjuli-methode 3 met screens in 2024, de verwarmde kelder met wanddelen van geval N en een zwak geventileerde smalle luchtlaag (F.2). De optiedekking vindt daardoor 14 posities meer (504), alle bereikt; de catalogus `training-data/nta8800-input-options.json` noemt nu ook de stoomdrager, de koudebrugmethode, de collectieve aansluiting en de serres per zone.

Gevonden en hersteld, elk met een test die op de oude code faalt:

- **Aangesloten oppervlakte zonder bereikgrens.** Een collectieve installatie met `collectiveConnection.connectedUsableAreaM2` van 10³⁰⁰ m² gaf een niet-eindig hulpenergiegebruik (9.4), dat het vangnet als `non_finite_result` weigerde in plaats van de invoer te noemen. De aangesloten oppervlakte krijgt nu de grenzen van de zone-oppervlakte: waarschuwing boven 1.000.000 m², blokkade boven 10.000.000 m², met de nieuwe melding `connected_area_out_of_range` op het veld (NL/EN, Ga naar).
- **Leeg tagveld met eenheidsvarianten.** Bij een lege soort van een bevochtiger naast een lege drager, of een lege soort zonwering naast een lege kleur, noemde de zoektocht naar lege velden ook de drager of de kleur. Of die gevraagd wordt hangt van de soort af (een verneveling of metaalhoudend doek vraagt er geen), en volgens de regel voor lege tags wordt dan alleen de soort genoemd. De zoektocht herkende de soort niet als tag, omdat een variant zonder velden (verneveling, doek, uitvalscherm) de andere velden negeert in plaats van te weigeren. Nu wordt alleen de soort genoemd; een lege drager bij een gekozen stoombevochtiger blijft een ontbrekend gegeven.

Fuzzresultaten: robuustheidsfuzz met 4000 pogingen per voorbeeld (16 voorbeelden) zonder panic of niet-eindig getal; numerieke fuzz over 23 voorbeelden (3619 getallen, 12174 runs) zonder bevinding; de fuzz van lege velden met 3000 pogingen op twee seeds over vijf blokken zonder valse of gemiste melding. De referentierapporten van alle suites zijn byte-identiek voor en na.

### 11 oktober 2026 — Review 54: kritische codereview van reviews 44–53 (geen rekenwijziging)
Codereview van alles wat na review 43 is samengevoegd (reviews 44–53, de gedeelde serre van 7.38 en het vervolg op review 53). Gevonden en hersteld, elk met een test die op de oude code faalt:

- **Lijst van aangrenzende zones niet gecontroleerd.** `borderingZones` van een serre (7.38) komt ook via de directe invoer van de maandbehoefte (API en MCP) binnen. Een negatieve H_zi,j;ztu, een zone die twee keer voorkomt, een onzinnige instelwaarde of waarden die samen overlopen (1e308 en −1e308 geven ∞ − ∞) gaven een ongeldige of NaN-begrenzing. De kern weigert zo'n lijst nu met `sunroom_invalid` op `sunrooms[i].borderingZones`.
- **Ingevulde lijst in de projectroute bleef staan.** De projectroute vulde `borderingZones` alleen aan als de lijst leeg was, zodat een in het project ingevulde lijst (met niet-bestaande zones of andere instelwaarden) de begrenzing bepaalde. De projectroute leidt de lijst nu altijd zelf af uit de zones met dezelfde serre-id, en maakt hem leeg voor een serre van één zone.
- **Lege serre-id gedeeld.** Twee zones met een serre zonder id telden als één gedeelde serre. Een lege id deelt nu nooit.
- **Dezelfde serre twee keer in één lijst.** Een zone kon dezelfde serre (dezelfde onverwarmde ruimte) twee keer kiezen; die telde dan dubbel in 7.30b en in 7.38. Dat is nu het ontbrekende gegeven `sunroom_id_duplicate` op de tweede vermelding, met "Ga naar".
- **"Ga naar" bij gemengde koudebrugmethoden.** Met de koudebrugmethode van het gebouw op forfaitair en ψ-waarden in een zone wees `thermal_bridge_methods_mixed` naar `ntaCalculation.groundFloors`, ook als het project geen vloer op de grond heeft (bovenwoning). De melding wijst nu naar `zones[i].thermalBridges` van de eerste zone met ψ-waarden; een gedetailleerde vloerrand houdt de melding op de vloeren.
- **Diepte per wanddeel: ingevulde diepte ging verloren.** Bij **Diepte per wanddeel opgeven** verdween de al ingevulde diepte z, en bij het verwijderen van het laatste wanddeel kwam z leeg terug. Het eerste wanddeel begint nu met de ingevulde diepte, en het laatste verwijderde wanddeel geeft zijn diepte terug aan z.

Gecontroleerd zonder bevinding: de gas/olie-keuze van de stoomopwekker met de oude waarde `gas_or_oil` bij opslaan en openen, de deling door het aantal identieke toestellen bij de grens van 500 m², nulwaarden bij de wanddelen (z = 0 wordt geweigerd), het verplaatsen van meldingen naar hun invoer na het herordenen van zones (op zone-id, niet op volgorde), en de overname van de koudebrugmethode met ongedaan maken. Projecten met geldige invoer rekenen gelijk; de referentierapporten zijn ongewijzigd.

### 11 oktober 2026 — Overname van de koudebrugmethode en de verwarmde kelder in de optiedekking (geen rekenwijziging)
Vervolg op review 53.

- **Overnemen in projectmodel stelt de koudebrugmethode voor.** De opname (ISSO 82.1/75.1) rekent met forfaitaire lineaire koudebruggen (ΔU_for, 8.3), maar de overname nam de methode van het gebouw (`ntaCalculation.thermalBridgeRoute`) niet mee. Een bovenwoning zonder vloer op de grond hield in het project zo de gedetailleerde route tot de adviseur forfaitair koos. Het venster heeft nu het deel *Koudebrugmethode van het gebouw*: forfaitair, aangevinkt als het project nog geen methode heeft, niet aangevinkt als het project een andere methode, een gedetailleerde vloerrand of ψ-waarden in de zones heeft. Ongedaan maken werkt zoals bij de andere overgenomen waarden. Een opname die niets over de methode zegt, verandert niets.
- **Verwarmde kelder in de optiedekking.** Openbaar geval N (verwarmde kelder met een diepte per wanddeel, 8.42) is een projectvoorbeeld van de optiedekking, zodat de catalogus `training-data/nta8800-input-options.json` de velden van `groundFloors[].heatedBasement` en de wanddelen (`wallDepths[]`) noemt; ook enkele andere blokken van N staan er nu in, alle met een formulier. De formulierbewaking (`nta-project-form-guard`) loopt het formulier van de vloeren op de grond ook door met een verwarmde kelder met twee wanddelen, en de kern rekent elke stand door.

### 11 oktober 2026 — Review 53: nieuwe invoer en meldingen van reviews 40–52 bereikbaar in de workflow (geen rekenwijziging)
Review van de invoer en meldingen die reviews 40–52 en de gedeelde serre toevoegden: formulieren, Ga naar, lijsten en handleiding.

- **Meldingen van de gebouwberekening in *Open punten*.** Waarschuwingen van de berekening zelf (`performance.warnings`, bijvoorbeeld `generator_large_system_outside`, `tojuli_sunroom_facade_left_out`, `annex_aa_capacity_insufficient`) stonden alleen in het kernpaneel en het rapport, zonder Ga naar en zonder telling in de stappen. De projectroute zet ze nu ook in `warnings`, op het pad van de projectinvoer (bijvoorbeeld `ntaCalculation.generator.boiler.location`, `ntaCalculation.sunrooms`, `ntaCalculation.constructionYear`), met het pad in de berekening als toelichting. Het kernpaneel en de rapporten tonen elke melding één keer.
- **Gedeelde serre in het formulier.** Twee rekenzones delen één serre als ze dezelfde serre-id hebben; dat was nergens te zien, en twee zones die elk hun eerste serre toevoegden kregen allebei `serre-1` en deelden dus ongewild. Nu:
  - krijgt een nieuwe serre een id dat nog geen zone gebruikt;
  - kies je de serre uit de onverwarmde ruimten van het project als die er zijn;
  - toont elke zone met een gedeelde serre *Gedeelde serre, ook bij …*, met een uitleg onder de lijst.
- **Kelderwand per wanddeel.** De diepte per wanddeel (8.42/D.12, review 44) kon de kern rekenen maar niet in het formulier worden opgegeven. **Diepte per wanddeel opgeven** vervangt nu de ene diepte z door wanddelen met lengte en diepte; het laatste wanddeel verwijderen zet de ene diepte terug.
- **Gecontroleerd zonder bevinding:** de koudebrugmethode van het gebouw, de zwak geventileerde smalle luchtlaag, de stoomopwekker gas/olie, de collectieve afleverset en de nieuwe meldcodes hebben Nederlandse en Engelse teksten en een Ga naar naar hun veld.
- Handleiding hoofdstuk 0 en 3 en `docs/nta8800-maandbehoefte.md` bijgewerkt.

### 11 oktober 2026 — Overzicht: welke uitkomsten in rekenkern 0.4.0 veranderen (geen rekenwijziging)
Samenvatting van de onuitgebrachte items hieronder die uitkomsten veranderen; de items zelf geven de details en de normverwijzingen. Dit item verandert zelf niets.

- **Referentiegevallen die bewegen.** Van de 290 gevallen in de referentiepoort veranderen er drie:
  - **openbaar geval N** (supermarkt, 2020+A1): BENG 1/2/3 van 54,57 / −24,31 / 122,6 naar 54,52 / −24,34 / 122,7 door de kelderwanden per wanddeel (review 44);
  - **ISSO 54 EP-W406q**: BENG 2 van 101,80 naar 104,13 door f_prac 0,95 op een verklaard tapwaterrendement van een gastoestel of indirect gestookt vat (review 41);
  - **ISSO 54 EP-W006g**: BENG 1 69,33 → 69,25, BENG 2 100,96 → 100,69, TOjuli 5,67 → 5,73 door de kelderwanden per wanddeel (review 44).

  Beide ISSO 54-deeltests hebben nog geen verwachte waarden. De overige openbare gevallen A–O, de RVO-voorbeeldwoningen en de andere deeltests zijn byte-gelijk.
- **Uitkomsten die alleen bij bepaalde invoer veranderen** (geen referentiegeval heeft die invoer):
  - **TOjuli:** methode 3 met beweegbare zonwering van tabel 7.5/7.6 in 2024; methode 3 en bijlage AA geweigerd in 2023 en ouder; de serregevel buiten de verdeling per oriëntatie (review 40).
  - **Tapwater:** de circulatieleiding bij een onbekend aantal woningen volgens tabel 13.29 (review 41).
  - **Ruimteverwarming:** het leidingverlies in de zone bij een collectieve afleverset en tabel 9.16; een ketel voor meer dan 500 m² buiten de thermische schil (review 42).
  - **Ventilatie:** een opening op een sectorgrens bij dwarsventilatie telt in één sector (review 43).
  - **Transmissie:** forfaitaire koudebruggen zonder grondvloer (nieuwe keuze, bestaande projecten ongewijzigd); een zonekoudebrug naar de grond is nu een blokkerend gat in plaats van stil weggelaten (review 44).
  - **Serre:** de begrenzing van de serrewinst bij een gedeelde serre (review 46), met het setpoint van elke aangrenzende zone (vervolg op review 52).
  - **Koeling:** de condensorwarmte bij een verklaarde EER of ζ met waterzijdige warmteafvoer (review 48).
  - **Forfaitaire R_c:** thermokussens onder een vloer met een spouw krijgen geen spouwwaarde meer (review 50).
  - **Constructies:** nieuwe keuze voor een zwak geventileerde smalle luchtlaag, met de halve waarde van tabel F.1 (review 51; bestaande projecten ongewijzigd).
  - **Overventilatie met een afvoerluchtwarmtepomp:** een opgegeven tapwaterdebiet onder het vereiste buitenluchtdebiet wordt opgehoogd tot q_V;ODA;req (13.148/13.148a; review 70).
  - **Oudere uitgaven:** miscanthus en graswol worden vóór 2025+C1 geweigerd (tabel E.11), en η_hr volgens NEN-EN 13053 is in 2022 en 2020+A1 nu een route met aftrek 0,10 (review 72). Invoer die al rekende verandert niet.
  - **Eerdere items van 9 oktober:** PV boven 300/400 W/m², WKK met voorraadvat, opgegeven F_D;dayl, kelderwanden in A_ls, wanden en daken aan de grond, gemeenschappelijke ruimten (6.6.4).
- **Open interpretaties.** De keuzes achter vragen 1–14 aan NEN veranderen niet tot er een antwoord is ([vragen aan NEN](nta8800-vragen-nen.md), [brief](nta8800-vragen-nen-brief.md)).

### 11 oktober 2026 — Gedeelde serre: begrenzing 7.38 met het setpoint van elke zone
Formule 7.38 (p. 187) begrenst de winst van een serre waaraan meerdere rekenzones grenzen met de som van H_zi,j;ztu·(θ_int;set;H;stc;zi − θ_e) over al die zones, elk met het eigen setpoint. De kern nam tot nu toe H_zi;ztu / F (7.36) met het setpoint van de eigen zone; dat is alleen exact bij gelijke setpoints (kanttekening van review 52).
- **Wat de kern nu doet:** komt dezelfde serre-id voor bij meer dan één zone in `zoneData`, dan krijgt elke kopie de lijst van alle aangrenzende zones met hun H_zi;ztu en verwarmingssetpoint (`borderingZones`), en de begrenzing rekent met die lijst. Een serre van één zone rekent zoals voorheen met 7.37.
- **Gevolg:** uitkomsten veranderen alleen bij een gedeelde serre tussen zones met verschillende setpoints volgens tabel 7.13, bijvoorbeeld een woonfunctie (20 °C) naast een sportfunctie (16 °C). Bij gelijke setpoints is de uitkomst gelijk. Geen referentiegeval heeft een serre; de referentierapporten zijn voor en na byte-gelijk.
- **Tests:** `a_shared_sunroom_caps_with_each_zone_setpoint_7_38` (handberekening met 6 W/K bij 20 °C en 14 W/K bij 16 °C, februari) en `a_shared_sunroom_lists_every_bordering_zone` (projectroute, twee zones).

### 11 oktober 2026 — Review 52: onafhankelijke herberekening van de wijzigingen uit reviews 40–51 (geen rekenwijziging)
Elke regel die in reviews 40–51 veranderde, is opnieuw uitgerekend met een eigen implementatie, geschreven vanaf de normpagina's zonder eerst de kerncode van die regel te lezen, en op synthetische invoer met grenswaarden met de kern vergeleken. Alle elf regels komen overeen (tot 1e-9). De gevallen staan nu als vaste tests in de kern (`review52_*`).
- **TOjuli methode 3 in 2024** (p. 106): schermen en zonneschermen van tabel 7.5/7.6 tellen, alleen "> 95 %" volstaat, noordramen (tussen NW en NO) tellen niet mee, horizontale ramen en een noordoostraam wel.
- **Tabel 13.29** (p. 553): de banden ≤ 10, 11–20 en > 20 woningen uit A_g;gebouw;W / A_g;tot rekenen gelijk aan dezelfde invoer met de diameter ingevuld. Bij een quotiënt op ,5 rondt de kern naar boven (10,5 → 11 woningen → 35 mm), zoals de interpretatielijst zegt; de norm noemt geen afronding.
- **13.152** (p. 616–617, afronding p. 627/645): f_prac 0,95 op een verklaard rendement van een gastoestel (0,737 → 0,725) en een indirect gestookt vat (0,949 → 0,925). Bij het gastoestel blijft c_W;gen van tabel 13.26 staan (p. 640–643); de eerste eigen berekening vergat die factor, de kern niet.
- **Tabel 9.16 en 9.26/9.27a** (p. 302–304, 310): ongeïsoleerde leidingen bij een afleverset Ψ 1,0 tot 500 m² en 2,0 daarboven, alleen verwarming boven 500 m² 3,0, en het juliverlies met L_equi = 0,15/Ψ·L.
- **Tabel 9.25 met §9.6** (p. 322, 327–328): 500 m² is nog een klein systeem, 500,5 m² niet; twee gelijke systemen op 1 000 m² zijn klein.
- **11.77–11.79** (p. 480–481): de halfopen sectoren, ook met vier ramen in verschillende sectoren en met alle ramen in één sector (A_w;cros 0).
- **8.38/8.39 met 8.32–8.35, 8.40–8.42 en 8.45** (p. 251–260): kelderwanden per wanddeel op eigen diepte, een ongeïsoleerde kelder op één diepte en de forfaitaire route met ΣA_T;bw verdeeld naar z_j·ℓ_j.
- **7.30b en 7.34–7.38** (p. 182, 185–187): een gedeelde serre met F = 0,3, twee niet-transparante vlakken, de begrenzing in februari en geen begrenzing bij koeling. **Kanttekening:** de kern neemt in 7.38 het setpoint van de eigen zone voor alle grenzende zones (zo staat het in de interpretatielijst). Bij gelijke setpoints is dat exact; grenzen zones met een ander setpoint aan dezelfde serre, dan wijkt de begrenzing af. Bijvoorbeeld 16 °C in plaats van 20 °C in de andere zone geeft in een synthetisch voorbeeld 17,9 kWh volgens 7.38 tegen 20,2 kWh in de kern. De invoer per zone kent de andere zones niet, dus dit blijft een beperking; geen referentiegeval heeft een serre.
- **10.76, 10.80 en 10.83 met §10.1** (p. 365, 417–423): verklaarde EER 5,04 → 5,00 (droge koeler) en 3,18 → 3,15 (open koeltoren) met Q_hr = Q_C·(1 + 1/EER) zonder f_prpr, 10.82 nul in methode 3, en de tabelwaarde 3,00 met f_prpr 1.
- **I.2.1.4** (p. 832–833) en **F.2** (p. 814–815): thermokussens 1,95 met en zonder spouw, de spouwwaarde bij ≤ 30 mm, en de halve tabel F.1-waarde op drie tabelknopen.

### 11 oktober 2026 — Review 51: rest van bijlage P (P.13–P.18, P.72–P.83) en de zwak geventileerde smalle luchtlaag (F.2)
Review 51 sloot de twee open punten van review 50.
- **Bijlage P zonder verschil nagerekend** tegen 2025+C1 en vergeleken met 2024 en 2023:
  - de distributieverliezen per maand en per buitentemperatuur (P.13–P.15, p. 957–958), de kruipruimtetemperatuur P.16 en de leidingweerstand in de grond en in lucht P.17/P.18 met tabel P.1 (p. 959–963) en tabel P.16 (p. 1026);
  - de warmte-, tapwater- en koudebehoefte van het gebied per jaar en per maand (P.72–P.78, met θ_in;ref 18 °C in P.74) en de rekenwaarden per perceel P.79–P.83 met tabellen P.14 en P.15 (p. 1019–1025).
  - De rekenregels zijn in 2024 en 2023 gelijk. Alleen de rij "één leiding in een sleuf" (f_x 1,00) van tabel P.1 is nieuw in 2025+C1; de kern gebruikte die factor al en houdt hem in de oudere uitgaven aan. De broncommentaar en punt 19 van de interpretatielijst in de verificatiestatus zeggen dat nu zo.
- **Zwak geventileerde smalle luchtlaag (F.2, p. 814).** Een zwak geventileerde luchtlaag krijgt de helft van de waarde van tabel F.1. Die regel had geen invoer, dus de kern paste hem nooit toe. De laag `narrow_cavity` heeft nu `weaklyVentilated` (standaard uit, dus bestaande invoer rekent gelijk), en **Constructie bewerken** biedt de laagsoort *smalle luchtlaag (tabel F.1)* met dikte, breedte en **zwak geventileerd**. **Gevolg:** alleen nieuwe invoer met die keuze; de referentierapporten zijn byte-gelijk.
- Tests: de laagweerstand in `constructions.rs` (d 10 mm, d/b 2: 0,20, zwak geventileerd 0,10; zonder het veld 0,20), die op de oude code faalt omdat het veld werd geweigerd, en de vertaling naar de kern in `nta-form-models.test.ts`.

### 11 oktober 2026 — Review 50: hoofdstuk 17, bijlage AB, de WKK-methoden, bijlage P en de constructiebijlagen; thermokussens zonder spouwwaarde (I.2.1.4)
Review 50 rondde de controle per hoofdstuk af met wat in de stand van 11 oktober nog niet opnieuw naast de tekst van 2025+C1 lag. Zonder verschil nagerekend:
- **Hoofdstuk 17:** tabel 17.1 (maandlengte, buitentemperatuur, ϑ_e;argII, windsnelheid, ϑ_ODA;preh;WTWC) en tabel 17.2 (alle hellingen en oriëntaties) met de keuzeregels voor oriëntatie en helling (p. 689–694); de beschaduwingstabellen 17.4 en 17.7–17.15 cel voor cel tegen de tekstlaag (7 584 cellen, p. 708–763); tabellen 17.5 en 17.6 zijn overal 1,00. De uurroute van 17.3.8 blijft onbeschikbaar zonder de uurgegevens van NEN 5060.
- **Bijlage AB (ZEB):** AB.1, AB.9–AB.15, AB.61–AB.73 met η_BAT 0,85 en f_BAT;el;corr vanaf 5 kWh, tabel AB.1 met de oppervlaktegewogen fractie, de factoren van tabel AB.2 en de emissiefactoren van tabel AB.3 (p. 1148–1159).
- **WKK:** methode 2 (9.65 met tabel 9.31, p. 342–343) en methode 1 (9.66–9.83 met tabellen 9.36–9.38, p. 347–354), en voor tapwater 13.182/13.183 met de afronding op 0,025 (p. 649–650).
- **Bijlage P:** tabellen P.0–P.12 en P.16, P.19–P.30 (energiefactor, energiefractie, β, WKK met en zonder derving) en de restwarmte van P.6.5.4.7 (p. 961–1003).
- **Constructies:** tabellen C.1–C.5 en C.15–C.22 (afschotisolatie), bijlage E (tabellen E.1–E.6 en E.10–E.17, de afrondingsregels), F.1–F.3, G.1, H.1, bijlage I (tabellen I.1/I.2 en I.4–I.16, I.1–I.7) en L.1/L.2 met criterium L.1 (p. 773–869).

- **Thermokussens onder een vloer (I.2.1.4, p. 833).** De kussenregel staat "in afwijking van het voorgaande": de vloer krijgt R_ad 0,15 plus 1,8, samen 1,95. Was de vloer ook als constructie met spouw ingevoerd, dan telde de kern daar de spouwwaarde 0,18 van de regel voor dunne isolatie bij op, dus 2,13. Die spouwwaarde valt nu weg; zo staat het ook in de opname (R_c 1,95, ISSO 82.1 p. 93). 2023 en 2024 hebben dezelfde tekst. **Gevolg:** alleen een forfaitaire vloer met thermokussens en een spouw. Geen referentiegeval heeft thermokussens; de referentierapporten zijn byte-gelijk.

### 11 oktober 2026 — Review 48: formules van bijlage M en bijlage Q, koudeopwekkers van hoofdstuk 10, condensorwarmte bij een verklaarde EER (10.80)
Review 48 legde de formules van bijlage M (M.1–M.29, p. 870–879), de formules van bijlage Q buiten de tabellen die review 47 al deed (Q.1–Q.16, Q.20–Q.31, Q.35–Q.53, Q.60–Q.97, p. 1027–1072) en de koudeopwekkers van hoofdstuk 10 (10.46–10.87 met tabellen 10.15–10.34, p. 397–425) naast de tekst van 2025+C1. Zonder verschil nagerekend:
- **Bijlage M:** het rendement bij vollast M.7/M.8 met tabel M.2, bij deellast M.10 met tabel M.4, de correctie uit een aanvullende test M.13/M.14, de verliezen M.9, M.11 en M.12 met f_prac 0,95 en tabel M.3, de interpolatie M.4/M.5 en M.21/M.22, M.6 en M.23 over de bedrijfstijd van tabel 9.15, de mantel- en hulpenergieverliezen M.16–M.19 met tabellen M.5/M.6, de regelingsfactor M.24 met tabel M.7 en P_int = 0,3·P_n (M.29). M.25 staat zo in alle uitgaven vanaf 2022.
- **Bijlage Q:** de energiefractie en het opwekkingsrendement Q.1–Q.4, de bronpomp Q.5–Q.8 (ook de derde macht van Q.6 en 50 W/kW bij water/water), de looptijd Q.9/Q.10, de modulerende warmtepomp Q.11–Q.16 en Q.20–Q.28 met tabel Q.2 en de carnotgrens Q.12A/Q.12B, de aan/uit-warmtepomp Q.29–Q.31 en Q.35–Q.40, de verdamperintrede Q.41–Q.44, Q.45–Q.48, de vlakken door de meetpunten Q.49–Q.70, de deellastconstanten Q.71–Q.80, f_cor;bu;max Q.81–Q.83 en de tijdfracties en herberekening van een afvoerluchtwarmtepomp Q.84–Q.97.
- **Hoofdstuk 10:** de prioritering en β van 10.49–10.52 met tabellen 10.15/10.16, methode 1 (10.53–10.64 met tabellen 10.17/10.18 en T_0;abs 273,16 K), methode 2 (10.65–10.75 met tabellen 10.19–10.27, f_prpr 0,60 of 0,9 en PLV 0,95 voor absorptie), methode 3 (10.76–10.78 met tabellen 10.29/10.30), de condensorwarmte en hulpenergie 10.79–10.83 met tabellen 10.31–10.33, vrije koeling 10.84–10.86 met tabel 10.34 en de regeling 10.87.

- **Condensorwarmte bij een verklaarde EER of ζ (10.80/10.81, p. 420).** De afgevoerde warmte van een koudeopwekker is Q·(1 + 1/EER) met de EER van het toestel zelf, zonder praktijkfactor; zo rekende de kern al bij methoden 1 en 2. Bij een verklaarde waarde in methode 3 (§10.1) nam de kern hier de verklaarde waarde maal f_prpr 0,9. Nu gebruikt ze de verklaarde waarde, naar beneden afgerond. **Gevolg:** alleen bij een verklaarde EER of ζ met watergekoelde condensatie (koeltoren, droge koeler, bodem of oppervlaktewater) daalt de pompenergie van het condensorwatercircuit (10.83), bij een verklaarde EER van 4,37 met ongeveer 2 %. Geen openbaar geval, RVO-voorbeeldwoning of ISSO 54-deeltest heeft een verklaarde koudeopwekker; de referentierapporten zijn ongewijzigd.
- **Vraag 12 aan NEN.** Voor de tabelwaarden van tabel 10.29/10.30 geldt f_prpr = 1 (p. 419); voor een verklaarde waarde die zo'n tabelwaarde vervangt (§10.1, p. 365), zegt de norm het niet. De kern rekent de aandrijfenergie met f_prpr 0,9, als een productwaarde in methode 1 en 2; met f_prpr 1 zou die 10 % lager zijn. De vraag staat in [vragen aan NEN](nta8800-vragen-nen.md) en in de [brief](nta8800-vragen-nen-brief.md), die nu twaalf vragen heeft, en in de interpretatielijst van hoofdstuk 10.
- **Interpretaties vastgelegd** in de lijst van hoofdstuk 9 (bijlage M): β van M.25 gebruikt de levering met de regelingsfactor van M.24 (M.25 noemt Q_H;gen;out, een factor f_ctr;ls 1,00–1,06 lager); het terugwinbare verlies naar de ruimte is M.16 plus M.19 (letterlijk zou M.20 de hulpenergie die in M.1 al aan het medium terugkomt nog eens meetellen); η_gen;Pn in M.12 is bij een condenserende ketel η_gen;Pn;60.
- Test: `a_declared_eer_rejects_heat_without_the_practice_factor` (space_cooling.rs), die op de oude code faalt.

### 11 oktober 2026 — Review 47: hoofdstuk 12 en de bijlagen, stoombevochtiger op olie (geen rekenwijziging)
Review 47 legde hoofdstuk 12 (bevochtiging en ontvochtiging, p. 520–524) en de bijlagen die de kern uitvoert naast de tekst van 2025+C1, en vergeleek hoofdstuk 12 en bijlage W woord voor woord met 2024 en 2023. Zonder verschil nagerekend:
- **Hoofdstuk 12:** 12.1 met h_we 2 538,2 kJ/kg, η_HU;rvd 0,55 voor een warmtewiel en ρ_a 1,205; tabel 12.1 (woonfunctie en sportfunctie 0); verneveling via de verwarming (E_hum = 0) en stoom 12.3 met η 0,8 of 0,6; het terugwinbare verlies 12.4 met de grens van 500 m²; 12.5 met tabel 12.2, directe expansie en een onbekend ontwerp als 6/12 °C, en geen ontvochtiging bij uitsluitend vloer-, wand- of plafondkoeling (10.5, p. 372). Hoofdstuk 12 is in 2023, 2024 en 2025+C1 gelijk.
- **Bijlage B** (B.1, werkzame dikte met 0,25 m²K/W, 100 mm en de halve dikte, open plafond), **bijlage M** (tabellen M.2–M.7), **bijlage N** (tabellen N.20, N.22, N.23, N.25 en N.26), **bijlage Q** (tabellen Q.4–Q.7, Q.16 en Q.17, de ontdooicorrectie Q.17–Q.19), **bijlage V** (tabellen V.1–V.3, V.1–V.6), **bijlage W** (W.1–W.14 met f_prac, de afronding naar boven, de ondergrens in W.9 vanaf 2025+C1 en tabel 13.23), **bijlage X** (tabel X.1 en de voorbeelden) en **bijlage AA** (AA.1–AA.13 met de tabellen AA.1–AA.3 en de aftrek van 35 W/m²).

- **Stoombevochtiger op olie.** 12.3 (p. 521) geeft gas en olie hetzelfde rendement 0,6, maar tabel 5.2 (p. 94) houdt aardgas en stookolie als aparte energiedragers, met een eigen CO2-factor (0,218 en 0,326 kg/kWh). De kern had één keuze "gas of olie" en boekte die als aardgas. De stoomopwekker heeft nu de keuzes elektrisch, gas en olie (`carrier`: `electricity`, `gas`, `oil`); olie telt als drager `oil` in het energiegebruik, het finale energiegebruik en de CO2-uitstoot. **Gevolg:** opgeslagen projecten veranderen niet: de oude waarde `gas_or_oil` en het antwoord "niet-elektrische stoom" uit de basisopname (die de brandstof niet vraagt) tellen als gas, zoals voorheen, en het formulier toont de oude keuze zolang die er staat. Wie olie gebruikt, kiest nu **Olie**; dan verschuift dat verbruik van gas naar olie en stijgt de CO2-uitstoot. De ISSO 54-deeltests EP-U701c en EP-U701d (stoom op gas) en de referentierapporten zijn ongewijzigd.
- **Interpretaties vastgelegd** in de lijst van hoofdstuk 12: tabel 12.1 wordt in een zone met meer gebruiksfuncties naar oppervlakte gewogen, en de brandstof van een stoomopwekker telt als zijn eigen drager.
- Tests: `an_oil_fired_steam_generator_books_oil` (humidification.rs) en `an_oil_fired_steam_humidifier_uses_oil` (project_performance.rs); die laatste faalt op de oude code, die `oil` niet kende. In het formulier: `offers gas and oil steam generators and keeps an earlier gas-or-oil choice visible`.

### 11 oktober 2026 — Review 46: begrenzing van de serrewinst bij een gedeelde serre (7.38), zonnewinst door de scheiding naar een serre (vraag 11)
Review 46 legde hoofdstuk 6 (zonering, gebruiksoppervlakte, A_ls en A_r, p. 156–163) en hoofdstuk 7 (maandbalans, p. 164–221) naast de tekst van 2025+C1 en vergeleek hoofdstuk 7 woord voor woord met 2024 en 2023. Zonder verschil nagerekend: de balans 7.1–7.13 met de terugwinbare verliezen (7.3–7.5, 7.7–7.9), transmissie en ventilatie 7.14–7.20 met b_v op het standaardsetpoint, verticale leidingen (tabel 7.1, 7.17a), interne winst 7.21–7.29 met de tabellen 7.2/7.3 en f_L, zonnewinst 7.31–7.33 met de hemelstraling 7.39 (h_lr;e 4,14, Δθ_sky 11 K, F_sky naar helling, α_sol 0,6), tabel 7.4 (met het zonwerende glas van 2025+C1 alleen bij utiliteit), 7.40–7.43 met de afronding van g_gl;n en F_c, de tabellen 7.4a/7.4b en 7.5–7.9, de kozijnfractie, tabel 7.10 (en de massakolommen van 2022), de benuttingsfactoren 7.46–7.58, de rekentemperatuur en niet-continu verwarmen en koelen 7.59–7.75, de tabellen 7.13–7.15, de nivellering 7.78/7.79, 7.80/7.81, tabel 17.1, de weging per gebruiksfunctie (6.5.3), N_woon;zi (6.2a/6.2b) en f_ls (6.7.3). Hoofdstuk 7 verschilt tussen 2023, 2024 en 2025+C1 alleen in tabel 7.4, de vaste lamellen (7.41a/7.41b, verticale lamellen) en de voetnoten van tabel 7.10/7.11; die hebben al een uitgavewissel of zijn invoerkeuzes.

- **Gedeelde serre (7.38).** Grenzen meerdere rekenzones aan één serre, dan begrenst 7.38 de serrewinst in de verwarmingsstand op b_U·Σ H_zi,j;ztu·(θ_int;set;H;stc − θ_e)·t over al die zones (p. 187). De kern nam alleen H_zi;ztu van de eigen zone, waardoor de begrenzing bij een verdeelfactor F < 1 te streng was. Nu telt Σ H_zi,j;ztu = H_zi;ztu / F (7.36), met het setpoint van de eigen zone. **Gevolg:** alleen bij een serre met F < 1 waarvan de winst de begrenzing raakt; de serrewinst en dus de warmtebehoefte van die zone kunnen dan tot een factor 1/F hoger respectievelijk lager uitvallen. Geen openbaar geval, geen RVO-voorbeeldwoning en geen ISSO 54-deeltest heeft een serre; de referentierapporten zijn ongewijzigd. Een zone met F = 0 krijgt geen serrewinst.
- **Interpretatie, vraag 11 aan NEN.** 7.6.1 (p. 182) bepaalt in de forfaitaire route van 8.4.1 de zonnewinst door de scheiding naar een serre; 7.6.6.1.1 (p. 189) laat straling die een raam pas na een serre bereikt, met de g van de serre doorgaan, terwijl 7.34 (opmerking 2, p. 185) alleen niet-transparante vlakken telt. De kern geeft de scheiding naar een serre in beide routes geen eigen zonnewinst, ook niet door haar ramen, en telt de serre alleen via 7.30b. Een project met zo'n scheiding krijgt de melding `sunroom_partition_gain_interpreted` (in de forfaitaire route altijd, in de berekende route bij ramen in de scheiding). De vraag staat in [vragen aan NEN](nta8800-vragen-nen.md) en in de [brief](nta8800-vragen-nen-brief.md), die nu elf vragen heeft; beide keuzes staan in de interpretatielijst van hoofdstuk 7.
- Tests: `a_shared_sunroom_caps_on_all_bordering_zones_7_38` (monthly_demand.rs) faalt op de oude code; `a_sunroom_partition_reports_the_gain_interpretation` (project_performance.rs) legt de melding vast.

### 11 oktober 2026 — Review 45: verlichting, BACS en de optelling in hoofdstuk 5 (geen rekenwijziging)
Review 45 legde verlichting (hoofdstuk 14, p. 655–675), f_BACS (5.5.8, p. 99–101) en de optelling van hoofdstuk 5 naast de tekst van 2025+C1 en vergeleek hoofdstuk 14 en 5.5–5.6 woord voor woord met 2024 en 2023. Zonder verschil nagerekend:
- **Verlichting:** 14.1–14.14 met de tabellen 14.1–14.3 (branduren, voorschakelpercentages, P_n;spec met de ledkolom vanaf 2024), afronding van P_n, P_em en P_pc naar boven volgens bijlage X, F_C = 1 (14.15), de aanwezigheidsfactoren 14.16–14.23 met de tabellen 14.4/14.5, de daglichtfactoren 14.24–14.42 met de tabellen 14.6–14.9, de forfaitaire methode 14.43/14.44 en f_L van 7.28. Hoofdstuk 14 is in 2024 en 2025+C1 gelijk; de verschillen met 2023 en 2022 hadden al een omschakeling.
- **f_BACS:** de 290 kW-toets per systeem, een onbekend vermogen, klasse D voor automatische regelingen of C/D voor energiemanagement, en de weging van E_H, E_C, W_H;aux en W_C;aux (5.20a/5.20b, 5.21).
- **Hoofdstuk 5:** tabel 5.2 (in alle uitgaven gelijk), tabel 5.3 per uitgave, tabel 5.4, 5.9–5.27 met de opslagcorrectie 5.14a/5.14b, de hernieuwbare energie 5.28–5.39h (warmtepompen met de grens van 20 °C en ventilatieretourlucht, biomassa bmA/bmB/bmC, koude met EER ≥ 8, warmtenet en collectieve bron), de afronding van BENG 1, BENG 2, RER, EwePrenTot, E_H;nd, E_C;nd en EweFinal, de standaard voor woningisolatie (5.3.2), de renovatiestandaard (tabel 5.7), het finale energiegebruik (5.57–5.60) en de labelklassen van de Omgevingsregeling.

- **Geen melding meer over het bouwjaar in een oudere uitgave.** Een oudere uitgave geeft geen blok `chapter5`, dus ook geen standaard voor woningisolatie; de melding `standard_insulation_construction_year_missing` verscheen toch. Die komt nu alleen nog in 2025+C1. De beschrijving in [normversies](nta8800-normversies.md) zegt nu dat het hele blok vervalt, ook de posten die de oudere uitgave wel kent (E_H;nd, de standaard voor woningisolatie in 2023/2024, de renovatiestandaard in 2024, EwePrenTot).
- **Interpretatie, vraag 10 aan NEN.** Bij een collectieve warmtepompbron onder 20 °C met kwaliteitsverklaring houdt de EMGforf-berekening de hele verklaring aan (f_P;del;dh, f_Pren;dh, K_CO2). 5.3.1.2 en 9.6.8.1.1.2.3 (p. 72 en 362) schrijven dat alleen voor f_P;del;dh in EwePTot;EMGforf voor; 5.3.1.3 (p. 74) noemt voor RERPrenTot;EMGforf geen uitzondering. Het gaat alleen om de EMGforf-waarden van RER en EwePrenTot, die bij een woning met gebiedsmaatregelen informatief op het label staan. De vraag staat in [vragen aan NEN](nta8800-vragen-nen.md) en in de [brief](nta8800-vragen-nen-brief.md), die nu tien vragen heeft; de keuze staat in de interpretatielijst van bijlage P.
- Tests: `standard_insulation_needs_the_construction_year` (building_performance.rs) controleert nu ook 2024, 2023 en 2022 en faalt op de oude code; `source_below_20_uses_one_value_in_both_scenarios` (annex_p.rs) legt de gekozen lezing van vraag 10 vast.

### 11 oktober 2026 — Review 44: kelderwanden per wanddeel, forfaitaire koudebruggen zonder grondvloer, koudebrug naar de grond
Review 44 legde de transmissie (hoofdstuk 8, p. 223–270) en bijlage D (p. 790–796) naast de tekst van 2025+C1 en vergeleek hoofdstuk 8 woord voor woord met 2024, 2023, 2022 en 2020+A1. Zonder verschil nagerekend: H_D met ΔU_for (8.1–8.3, tabel 8.1, p. 224–225, met de uitsluiting van vloeren op grond of boven een kruipruimte, panelen en deuren), de afronding van U (8.2.2.1, p. 228), U_C met de toeslagen (8.4–8.13, tabel 8.2, de 3 %-regel, p. 228–235), de grond (8.30–8.51, p. 250–264: B'_f, de equivalente diktes, λ_gr 2,0, 8.36–8.38, 8.40–8.49 met h 0,125 m vanaf 2023), de onverwarmde ruimte (8.52–8.59, p. 264–269, met H_V;iu = 0, ρ_a·c_a 1 211 en de forfaitaire route met U_iu;eq = U_C·b_U van C.10, p. 777), H_A = 0 (8.5, p. 269), f_ls (6.7.3, p. 162) en de maandwaarden D.1–D.18 (U_x;T in D.15/D.16 is daar de U van de wand boven maaiveld, p. 796). Hoofdstuk 8 is in 2024 en 2025+C1 inhoudelijk gelijk; de verschillen met 2023 en ouder (h 0,125 m, de eenheid van ε, de raamformules, de voorwaarde voor H_U;adj) zijn invoer of hebben al een schakelpunt. Drie punten wijken af:
- **Kelderwanden per wanddeel.** 8.39 telt Σz_j·ℓ_j·U_bw;j en 8.38 ΣA_T;bw·(U_bw;j + ΔU_for), met U_bw;j van 8.45 op de eigen diepte z_j van wanddeel j (p. 253–255 en 260; gelijk in 2020+A1–2024). De kern nam voor alle wanddelen U_bw bij de gemiddelde z van 8.42. Nu rekent elk wanddeel van `heatedBasement.wallDepths` met zijn eigen z_j; de vloer houdt de gemiddelde z (8.40/8.41 met 8.42), de periodieke termen die van D.12. In 8.38 wordt een opgegeven ΣA_T;bw naar z_j·ℓ_j over de wanddelen verdeeld. **Gevolg:** alleen bij een verwarmde kelder met verschillende diepten per wanddeel; U_bw daalt met de diepte, dus een diep wanddeel verliest minder dan bij de gemiddelde z. Openbaar geval N (31,80 m op 3,65 m, de rest op 0 m): 54,57 / −24,31 / 122,6 → 54,52 / −24,34 / 122,7, tegen 55,98 / −27,18 / 126,9 in het rapport (warmtebehoefte 80 201 → 80 023 kWh). ISSO 54 EP-W006g (dijkwoning, nog zonder verwachte waarde): 69,33 / 100,96 → 69,25 / 100,69, TOjuli 5,67 → 5,73. De andere 288 gevallen van de suites veranderen niet.
- **Forfaitaire koudebruggen zonder vloer op de grond.** 8.2.1 (p. 224–225) laat de forfaitaire ΔU_for toe voor elk gebouw, mits voor het hele gebouw. De kern koos die route alleen via een forfaitaire vloerrand, dus een gebouw zonder vloer op de grond (een bovenwoning) moest elke ψ invoeren of rekende zonder koudebruggen. Nieuw veld `ntaCalculation.thermalBridgeRoute` (`forfait` of `detailed`; leeg: de vloerrand beslist, zoals voorheen), met een keuzelijst in *Gebouw → Schil & ramen*. Kiezen gebouw en vloerrand verschillend, dan volgt `thermal_bridge_methods_mixed` op `ntaCalculation.thermalBridgeRoute`. Bestaande projecten veranderen niet.
- **Een koudebrug naar de grond in de zonelijst viel weg.** H_D, H_U en H_g namen een lineaire of puntvormige koudebrug met begrenzing *grond* uit de koudebruggenlijst van de zone niet mee; ψ_gr van de vloerrand hoort bij de vloer op grond (8.36/8.39, p. 253 en 255–256). Zo'n koudebrug geeft nu het invoergat `thermal_bridge_ground_on_floor_edge` op haar begrenzing. **Gevolg:** een opgeslagen project met zo'n koudebrug rekent pas weer als de koudebrug bij de vloerrand staat of een andere begrenzing heeft. Geen fixture of suitegeval heeft er een. Daarnaast waarschuwt de kern bij een vloerrand met *ψ-waarden per randdeel* zonder randdelen (`ground_floor_edge_bridges_none_entered`): H_g heeft dan geen randverlies.
- Tests: `heated_basement_follows_8_39_and_d10_d11` (ground.rs) rekent de wanddelen nu met elk hun eigen U_bw en faalt op de oude code; `forfait_bridges_without_a_ground_floor`, `a_zone_bridge_to_the_ground_is_refused_not_dropped` en `a_detailed_floor_edge_without_psi_is_flagged` in `project_performance.rs`; `src/__tests__/thermal-bridge-route.test.tsx` voor het formulier. De handleiding (hoofdstuk 3, *Koudebrugmethode van het gebouw*) en [maandbehoefte](nta8800-maandbehoefte.md) beschrijven de regels; de vergelijking van N en de tabel in [normversies](nta8800-normversies.md) zijn bijgewerkt.

### 11 oktober 2026 — Review 43: sectorgrens bij dwarsventilatie (11.77)
Review 43 legde ventilatie en infiltratie (hoofdstuk 11, p. 426–519) en het gebruik daarvan in de warmtebalans (7.19/7.20, p. 174–175) naast de tekst van 2025+C1 en, waar de kern uitgaven onderscheidt, 2024, 2023, 2022 en 2020+A1. Zonder verschil nagerekend: het luchtstroommodel (11.1–11.21, tabellen 11.1–11.3, p. 426–447, met de routine van 11.2.1.6 en de nauwkeurigheid 11.14), de benodigde en de ontwerpvolumestroom (11.22–11.24 en 11.55–11.65, tabellen 11.5–11.9, p. 449–472, met overventilatie, f_terugregel, de ondergrens van 35 dm³/s en q_V;inst = 0 bij vraagsturing in woningen), de systeemvarianten A–E (11.25–11.54, p. 452–465), spuien (11.66–11.69, tabel 11.10, p. 473–474), ventilatieve koeling (11.70–11.76, p. 475–479, met f_argII, τ_argII en de stroomrichtingen), open verbrandingstoestellen (11.80–11.83, tabellen 11.11/11.12, p. 483–484), infiltratie (11.84–11.86, tabellen 11.13/11.14, p. 485–488), de distributie (11.87–11.98, p. 489–492), de toevoertemperatuur (11.99–11.104, tabel 11.15, p. 493–496), vorstbeveiliging (11.105/11.106, tabel 11.16, p. 497), warmteterugwinning met bypass en koudeterugwinning (11.106a–11.109, tabellen 11.17/11.18, p. 498–503), recirculatie (11.110–11.113), de LBK-batterijen (11.114–11.121, met c_a zoals gedrukt, vraag 7), de ventilatorwarmte (11.3.2.7), de kanalen (11.122, tabel 11.19, p. 507–508), voorverwarming in roosters (11.123–11.128, p. 509–511), de afvoertemperatuur (11.129/11.130, p. 511) en de ventilatoren (11.131–11.142, tabellen 11.20–11.23, p. 512–519), plus het vaste C1-systeem van 5.4.3. De tabellen 11.5, 11.8, 11.10, 11.12–11.23 zijn per uitgave vergeleken; waar ze verschillen (tabel 11.7, 11.8 en 11.17, de f_argII, de NEN 1087-factor, (11.71a) en (11.77a)) heeft de kern al een schakelpunt. Eén punt wijkt af:
- **Een opening op een sectorgrens telde in twee sectoren.** (11.77)/(11.77b) (p. 480; 2024 p. 464; 2023 p. 469) delen de openingen voor A_w;cros in vier sectoren van 90° in, met α_min ≤ α_w;k < α_max: een halfopen sector. De kern telde een opening tot en met 45° van de sectoras mee, dus een opening precies op een grens (op noord, oost, zuid of west in de diagonale indeling, of op noordoost, zuidoost, zuidwest of noordwest in de gewone) in beide aangrenzende sectoren. Nu telt zo'n opening in één sector, de sector die met de klok mee volgt. **Gevolg:** alleen bij dwarsventilatie (ventilatieve koeling met openingen in twee gevels die minstens 90° verschillen, of in een gevel en het dak) met een opening op zo'n grens, en alleen als de windterm van (11.75)/(11.76) groter is dan de trekterm: A_w;cros en dus q_V;argII in de koudebehoefte veranderen. Voorbeeld: een raam op noord en een raam op zuidoost van elk 1 m² gaven A_w;cros 0,530 m², nu 0,354 m²; een noordraam met een dakluik (β 30°) gaf 0,354 m², nu 0,530 m². Alle uitgaven. Openbare gevallen A–O, de RVO-voorbeeldwoningen en de ISSO 54-suite veranderen niet: de rapporten van alle 290 gevallen zijn voor en na gelijk.
- Test: `an_opening_on_a_sector_boundary_counts_in_one_sector` in `ventilation.rs` faalt op de oude code; `cross_ventilation_area_follows_11_77_to_11_79` rekent het dakluik nu met drie sectoren per ronde. De beschrijving in [ventilatie](nta8800-ventilatie.md) en de handleiding (hoofdstuk 3) noemen de regel.

### 11 oktober 2026 — Review 42: leidingen naar een afleverset, ketels van systemen boven 500 m²
Review 42 legde de ruimteverwarmingsketen (hoofdstuk 9) naast de tekst van 2025+C1 en, waar de kern uitgaven onderscheidt, 2024, 2023, 2022 en 2020+A1. Zonder verschil nagerekend: de afgifte (9.11–9.12a met de tabellen 9.2–9.4, p. 296–298; de grens 0,15 in 9.12a), de ventilatoren en luchtverwarmers in de afgifte (9.21–9.23, tabellen 9.11–9.13, p. 298–301, met de schatting van Q_h;b), de distributie (9.26–9.32a, tabellen 9.14–9.16 en 9.X, p. 302–310), de energiefractie bij meer opwekkers (9.56–9.60, tabel 9.23, p. 322–324), de ketelrendementen (tabel 9.25, p. 327–328), de warmtepompen (9.62/9.63, tabellen 9.27–9.29, p. 331–337), biomassa (tabel 9.30, p. 341), WKK (tabel 9.31, p. 343), externe warmtelevering (9.84 met f_prac, p. 356) en de hulpenergie van ketels en warmtepompen (9.85, p. 359–360). Twee punten wijken af:
- **Distributieleidingen naar een tapwater-afleverset.** 9.4.1 en 9.27 (p. 302 en 304; 2024 p. 284 en 286) zetten L_zi alleen op 0 als het distributiesysteem uitsluitend voor ruimteverwarming dient. Een collectieve installatie met een afleverset voert via dezelfde leidingen ook warmte voor tapwater; 13.4 c) (p. 545; 2024 p. 529) legt het verlies van die leidingen uitdrukkelijk in 9.4.2, en tabel 9.16 (p. 310; 2024 p. 292) heeft voor deze leidingen een eigen rij. De kern nam bij "afleverset" wel de volle maand en ϑ_H,mean ≥ 65 °C, maar alleen met het vinkje "leidingen voeren ook tapwater" ook L_zi en de gecombineerde rij. Nu telt een afleverset als zodanig. **Gevolg** (2024 en 2025+C1, alleen met afleverset zonder dat vinkje): het distributieverlies in de verwarmde zone telt mee, het hele jaar op 65 °C, met Ψ uit de gecombineerde rij (geïsoleerd 0,4 in plaats van 0,3); het verwarmingsgebruik stijgt, deels gecompenseerd door de terugwinbare verliezen. 2023 en ouder kennen deze regels niet en veranderen niet. Geen openbaar geval of suitegeval gebruikt deze combinatie.
- **Ketel van een systeem voor meer dan 500 m².** §9.6 (p. 322; 2024 p. 304; 2022 p. 303; 2020+A1 p. 301) en opmerking b bij tabel 9.25 (p. 328) stellen dat de opwekkers van een systeem voor meer dan 500 m² per definitie in een aangrenzende ruimte staan, dus buiten de thermische begrenzing. De kern nam de ingevoerde opstelling over. Nu rekent een gasketel (ook als bijverwarming van een hybride of bijlage Q-warmtepomp) of een centrale biomassaketel die "binnen" staat, bij meer dan 500 m² per toestel met de rij "buiten", met de waarschuwing `generator_large_system_outside`. **Gevolg:** bijvoorbeeld een HR-107-ketel als hoofdverwarming, LT: 0,925 in plaats van 0,975, dus 5,4 % meer gas; een centrale biomassaketel 0,75 in plaats van 0,80. Alle uitgaven. Geen openbaar geval of suitegeval verandert: in de ISSO 54-suite staan de grote systemen al buiten of zijn per woning gemodelleerd. De test `fifty_dwellings_calculate_like_one` modelleert de vijftig woningen nu met één ketel per woning (identieke systemen).
- **Geval O als toets.** Het verschil in verwarming van O (warmtenet, 2022) zit na vraag 7 (c_a in 11.115/11.119) nog op +385 kWh warmte voor het systeem (17 841 tegen 17 456 kWh, BENG 2 +0,31). Afgifte (Δθ_int;inc 3,2 K, 1 794 kWh), distributie (2022 nog zonder L_zi = 0, 1 526 kWh), pompenergie (389 kWh, gelijk aan het rapport) en f_prac 0,95 volgen de norm; de rest ligt in dezelfde orde als het verschil in BENG 1 (+1,1 %) uit de aannames voor b_U en de kruipruimte. Vraag 7 verklaart dus het verschil; er is geen tweede oorzaak gevonden.
- Tests: `a_delivery_set_keeps_the_zone_pipes_and_the_combined_row` en `a_boiler_for_more_than_500_m2_takes_the_outside_row` in `space_heating_chain.rs`; beide falen op de oude code. De handleiding (hoofdstuk 3) beschrijft beide regels.

### 11 oktober 2026 — Review 41: f_prac van gedeclareerde gas- en ketelrendementen voor tapwater, diameter van de circulatieleiding, TOjuli bij een AOR in de forfaitaire route
Review 41 legde de tapwaterketen (hoofdstuk 13, p. 525–655) en de PV-keten (16.2, p. 677–682, met 5.10–5.14b, p. 85–87) naast de tekst van 2025+C1 en, waar de kern uitgaven onderscheidt, 2024 en 2023. Zonder verschil nagerekend: de behoefte (13.15–13.19, tabel 13.1), de afgifte (tabellen 13.2/13.3, 13.21–13.23), de douche-WTW (13.51–13.53, tabellen 13.7/13.8), de circulatie (13.26–13.44, tabellen 13.4 en 13.6), de voorraadvaten (13.58–13.60a, tabel 13.9), de tabellen 13.22 en 13.25–13.28, de hulpenergie (13.181), externe warmtelevering (13.8.4.9), en bij PV 16.2–16.4b, de tabellen 16.1–16.3, f_prac;PV, de collectieve verdeling en de opslagcorrectie 5.14a/5.14b. In 16.2 en 13.2–13.6 verschillen 2024 en 2025+C1 alleen in redactie. Twee punten wijken af:
- **f_prac van een gedeclareerd tapwaterrendement van een gastoestel of ketel.** 13.152 (p. 616–617) geeft f_prac 1,0 alleen aan de forfaitaire waarden van 13.8.4.5–13.8.4.7; in alle overige gevallen 0,95. De kern paste dat (vraag 3 aan NEN) al toe op een gedeclareerde warmtepomp, maar gaf een gedeclareerd of volgens bijlage T gemeten rendement van een gastoestel (tabel 13.25) en een gedeclareerd rendement van een ketel met indirect verwarmd voorraadvat (tabel 13.28, p. 645) nog f_prac 1. Nu 0,95. **Gevolg:** het tapwatergebruik van zo'n toestel stijgt met 1/0,95 (+5,3 %). Openbare gevallen A–O veranderen niet; in de ISSO 54-suite alleen EP-W406q (2022, gastoestel met verklaring, nog zonder verwachte waarde): BENG 2 101,80 → 104,13 kWh/(m²·jr), BENG 1 en TOjuli gelijk.
- **Diameter van de circulatieleiding bij een onbekend aantal woningen.** Tabel 13.29 (p. 553; 2024 p. 538) kiest de diameter naar het aantal aangesloten woonfuncties; is dat onbekend, dan is het A_g;gebouw;W / A_g;tot. De kern nam dan één woning (28 mm). Nu rekent ze het aantal uit de oppervlakten (afgerond op hele woningen). **Gevolg:** bij een collectief tapwatersysteem met circulatie zonder opgegeven diameter en aantal woningen wordt de diameter 35 mm (11–20 woningen) of 42 mm (meer dan 20), dus een hogere Ψ en een groter circulatieverlies. 2023 en ouder (rijen "klein"/"overig" van tabel 13.4) veranderen niet. Geen openbaar geval of suitegeval gebruikt deze route.
- **Interpretatie, vraag 9 aan NEN.** In de forfaitaire route van 8.4.1 (H_U;for = 0, opmerking 5 op p. 266) telt de kern een scheidingsconstructie naar een AOR als uitwendige scheidingsconstructie mee in A_T;or en H_C;D;juli van haar oriëntatie (zonder zonnewinst), terwijl stap A van 5.7.2 (p. 115) een AOR-element buiten A_T;or laat. Dat blijft zo, zodat de verdeling aansluit op H_D;for; de vraag staat in [vragen aan NEN](nta8800-vragen-nen.md) en in de [brief](nta8800-vragen-nen-brief.md), die nu negen vragen heeft. Alleen geval O gebruikt deze route; het rapport geeft geen TOjuli.
- De kern noemt deze keuzes nu ook in haar lijst met interpretaties (`tojuli` en `domestic_hot_water`). Tests: `declared_gas_and_indirect_boiler_values_take_f_prac_0_95` en `table_13_29_counts_dwellings_from_the_areas_when_unknown` in `domestic_hot_water.rs`; beide falen op de oude code.

### 11 oktober 2026 — Review 40: TOjuli-uitzondering 5.7.1 per uitgave, Bbl-oordeel per zone, gevel naar een serre
Review 40 legde TOjuli (§5.7, p. 113–120) en de koelketen (hoofdstuk 10, p. 366–426) naast de tekst van 2025+C1, 2024, 2023 en 2022. De rekenregels van 5.7.2 zijn in die vier uitgaven gelijk en de kern volgt ze; in de koelketen zijn 10.4 (10.21–10.22b, tabel 10.9, 10.27), de pompenergie (10.28–10.38, tabellen 10.10–10.13) en de terugwinbare energie (10.4, 10.43–10.45) nagerekend zonder verschil. Drie punten wijken af:
- **Methode 3 van 5.7.1 verschilt per uitgave.** NTA 8800:2024 (p. 106) laat een opening met zonwering van tabel 7.5 of 7.6 (screens, jaloezieën, rolluiken, gemetalliseerde weefsels, uitval- en knikarmschermen) meetellen als beperking van de zoninstraling; 2025+C1 (p. 114) noemt daar de vaste lamellen van tabel 7.4a/7.4b. De kern telde in 2024 alleen glas met g_gl ≤ 0,4 en belemmering. Nu telt in 2024 een raam met een tabelzonwering (`movableShading.device`); een gedeclareerde F_c noemt geen tabeltype en telt niet. Nieuw schakelpunt 10a (`solar_limitation_movable_shading`). **Gevolg:** een 2024-project met actieve koeling en de toets `shaded_glazing` dat eerder `solar_limitation_not_met` kreeg en een berekende TOjuli, krijgt nu TOjuli 0 als meer dan 95 % van de beoordeelde ramen zo'n zonwering heeft. 2025+C1 verandert niet.
- **Geen capaciteitsmethoden in 2023 en ouder.** 5.7.1 van NTA 8800:2023 (p. 103–104) en 2022 kent de drie methoden niet; de kern weigerde al bijlage AA, maar accepteerde methode 3 en toetste dan de raamgegevens. Nu geeft `capacity.method: solar_limitation` daar `route_not_in_edition`, net als bijlage AA (schakelpunt 31, veld nu `cooling_capacity_methods_route`). **Gevolg:** een project in 2023 of ouder met deze methode wordt `invalid` voor TOjuli tot de methode wordt gewijzigd (bijvoorbeeld een dynamische koellastberekening). Geen openbaar geval gebruikt haar.
- **Bbl-oordeel alleen voor een woonfunctie, ook per zone.** Bbl art. 4.149b stelt de TOjuli-grens alleen voor woonfuncties; op gebouwniveau gold dat al, maar per zone gaf `tojuli[].meetsBblLimit` ook bij een utiliteitszone `true`/`false`. Dat is nu `null`. Getallen veranderen niet.
- **Gevel naar een aangrenzende onverwarmde serre (vraag 8 aan NEN).** Stap A telt het geveldeel naar een serre mee in A_T;or, maar stap 1 geeft er via 8.2 geen H_C;D voor. De kern laat het geveldeel, zijn transmissie en de serrewinst van 7.30b buiten de verdeling, zoals bij een gewone aangrenzende onverwarmde ruimte, en meldt dat met `tojuli_sunroom_facade_left_out`. De vraag staat in [vragen aan NEN](nta8800-vragen-nen.md) en in de [brief](nta8800-vragen-nen-brief.md), die nu acht vragen heeft. Geen openbaar geval heeft een serre.
- Openbare gevallen A–O en de ISSO 54-suite veranderen niet (geen van hen gebruikt methode 3 of een serre). Tests: `movable_shading_of_tables_7_5_and_7_6_counts_in_2024_only`, `solar_limitation_is_not_in_2023`, `the_bbl_limit_is_for_dwellings_only` en `a_zone_with_a_sunroom_reports_the_left_out_facade` in `tojuli.rs`; alle vier falen op de oude code.

### 11 oktober 2026 — Uitwisseling: een waarde van 0 en de helling van een UNIEC3-dak (geen rekenwijziging)
- **Een COP, EER of rendement van 0 veranderde stil.** De export schreef 0, de UNIEC3- en VABI-import namen dat niet over en gaven de standaardwaarde (EER 3, COP en rendement 1). Nu:
  - een EER van 0 bij koeling van de soort *geen* (bijvoorbeeld de Reddingspost Kijkduin) is niet in gebruik en komt ongewijzigd terug, zonder punt;
  - een COP, EER of rendement van 0 bij een werkend systeem (bijvoorbeeld het tapwaterrendement in het EDR-project) blijft uit het bestand en staat in de exportmelding (`heatingSystems[*].cop`, `coolingSystems[*].eer`, `hotWaterSystems[*].efficiency`); bij het weer inlezen staat de aangenomen waarde als punt in het importverslag.
- **Helling van een hellend UNIEC3-dak.** De VABI-import vroeg bij een hellend dak zonder `Hellingshoek` om de hoek in de NTA-invoer, de UNIEC3-import niet. Een UNIEC3-begrenzing heeft hier geen hellingshoek; een dak met een oriëntatie krijgt nu hetzelfde punt (`zones[*].surfaces[*].tilt`, *dropped*). Een plat dak niet.
- Tests: vier nieuwe tests in `src/__tests__/import-fidelity-review39.test.ts`; alle vier falen op de oude code. Handleiding hoofdstuk 9 bijgewerkt.

### 11 oktober 2026 — Review 39: getrouwheid van de UNIEC3- en VABI-uitwisseling (geen rekenwijziging)
- **Exportmelding was onvolledig.** De melding "niet alles overgenomen" keek alleen naar zones, vlakken, ramen, koudebruggen en constructies. Ze noemt nu ook:
  - velden van installaties die geen van beide formaten bevat, zoals de NTA-warmtepompgegevens (`heatingSystems[*].ntaHeatPump`) en een opmerking bij een PV-veld (`solarPV[*].comment`), die stil wegvielen;
  - bij UNIEC3 een constructie die geen vlak gebruikt (`constructions[unused]`): UNIEC3 schrijft een constructie alleen bij de vlakken die haar gebruiken;
  - bij VABI een raam met een andere oriëntatie dan zijn vlak: een VABI-deelvlak heeft geen eigen oriëntatie, dus het raam kwam terug met de oriëntatie van het vlak.
- **VABI-export van een vlak vol ramen.** Bedekten de ramen het hele vlak, dan schreef de export geen dicht deelvlak en ging de constructie van het vlak bij het weer inlezen verloren (openbaar geval F, de dakkapelgevels). Het dichte deelvlak staat er nu altijd, met oppervlakte 0 als de ramen het vlak vullen.
- **Niet-ingelezen gegevens worden genoemd.** De import sloeg alles over waar hij geen veld voor heeft, zonder punt in het importverslag. Nu staat het er als *Niet overgenomen*, één punt per soort:
  - UNIEC3: entiteiten die niet gelezen zijn (een andere soort, een entiteit die niet gekoppeld is waar de import zoekt, een tweede constructie of luchtdichtheid onder één element) en per entiteitsoort de eigenschappen zonder veld;
  - VABI: per soort element de onderliggende elementen zonder veld;
  - niet mee tellen bibliotheekkopieën en een lege instellingen-entiteit; een export van dit programma geeft bij het weer inlezen geen zulke punten.
- **Bronversie en editie.** De import las `meta.json` van UNIEC3 en het blok `Algemeen` van VABI niet. Het importverslag noemt nu de **bronversie** (programma en NTA 8800-versie volgens het bestand), en elke import heeft een punt bij `normVersion`: de editie en de rekeninstellingen van het andere programma worden niet overgenomen, het project rekent met de editie van de NTA-invoer. Zo valt een bestand uit een oudere Uniec 3 op voordat je registreert.
- **Rondgang.** Een nieuwe test exporteert elk trainingsproject naar UNIEC3 en VABI, leest het weer in en eist dat elke gewijzigde waarde in de exportmelding of het importverslag staat.
- Tests: `src/__tests__/import-fidelity-review39.test.ts` (veertien tests; elf falen op de oude code). Handleiding hoofdstuk 9 bijgewerkt.

### 11 oktober 2026 — Maatwerkadvies: resultaat per document en kolommen als het rapport (geen rekenwijziging)
- **Resultaat blijft staan.** Het paneel hield het resultaat van **Doorrekenen** zelf vast, dus na het verlaten van de stap Maatwerkadvies stond er weer "niet berekend". Het resultaat wordt nu per document bewaard (`MaatwerkadviesResultProvider`), met de regel van review 38: alleen getoond zolang het document nog de kernelinvoer heeft waaruit het is berekend, nooit bij een ander document, en niet als het binnenkomt na een wijziging. Het wordt niet in het projectbestand opgeslagen.
- **Kolommen.** De resultatentabel op het scherm miste de kolommen TO<sub>juli</sub> [K] en Warmte [kWh] die het rapport wel heeft. Ze staan er nu, met dezelfde afronding (2 en 0 decimalen); de tabel scrolt horizontaal op een smal scherm.
- Tests: `src/__tests__/mwa-result-persist.test.tsx` (vier tests; drie falen op de oude code). Handleiding hoofdstuk 6 bijgewerkt.

### 11 oktober 2026 — Review 38: maatwerkadvies (geen rekenwijziging)
- **Resultaat hoort bij zijn invoer.** Het paneel hield het laatste resultaat vast tot het advies zelf werd gewijzigd. Na een wissel van normuitgave, ongedaan maken, een ander document of een wijziging tijdens het doorrekenen bleef zo een resultaat staan dat niet bij de getoonde invoer hoorde; document B kon het resultaat van document A tonen. Het resultaat wordt nu bewaard met de kernelinvoer waaruit het is berekend en alleen getoond zolang het document die invoer nog heeft. Terug in het eerste document staat het weer.
- **Adviesrapport.**
  - Zonder geadviseerd pakket stond er "— (automatisch: hoogste netto contante waarde, door adviseur te bevestigen)". Er staat nu "geen pakket geadviseerd", zoals op het scherm.
  - Een gedeeltelijk berekend advies noemde de meldingen van de rekenkern alleen op het scherm. Het rapport heeft nu een hoofdstuk **Meldingen** met code, pad en toelichting.
- **MCP.** Een tool zonder invoer (GET-operatie) negeerde argumenten, terwijl het schema geen leden toestaat. Elk argument wordt nu geweigerd met `invalid_request_shape`, zoals een onbekend lid bij de andere tools. De HTTP-routes geven deze operaties nooit een body en veranderen niet.
- Tests: `src/__tests__/mwa-review38.test.tsx` (vier tests, alle vier falen op de oude code) en `a_tool_without_input_refuses_arguments` in de MCP-server. Handleiding hoofdstuk 6 en `docs/nta8800-mcp.md` bijgewerkt.

### 11 oktober 2026 — Review 37: contract van HTTP-API en MCP-server (geen rekenwijziging)
- **OpenAPI-document.**
  - Elke route kan 405 `method_not_allowed` geven (de verkeerde methode), maar het document noemde 405 nergens. Het staat nu bij elke route, met de `Allow`-header; 403 `host_not_allowed` staat nu ook bij `/health` en `/v1/openapi.json`.
  - `GET /v1/nta8800/interpretations` geeft een lijst, maar het document beschreef elke uitkomst als object. Het beschrijft nu de lijst met per groep `module`, `part` en `items`.
  - De lijst van foutcodes bij `Error.code` miste `edition_not_implemented` (en `unknown_tool` van de MCP-server). De lijst komt nu uit één constante (`ERROR_CODES`); een test leest de bronnen van de service en de editieweigeringen van de kern en faalt bij een code die er niet in staat.
- **MCP-server.**
  - `list_interpretations` gaf een lijst als `structuredContent`, terwijl MCP daar een object vraagt; een strikte client kon de uitkomst weigeren. De lijst komt nu verpakt als `{"result": [...]}`.
  - De HTTP-API weigert een body met meer dan 500 000 JSON-waarden voordat hij wordt ingelezen; de MCP-server las elke regel tot 64 MiB in, wat een veelvoud daarvan aan geheugen kon vragen. De poortwachter weigert nu ook een regel met meer waarden dan de grens (`-32600`, `data.code` `payload_too_large`); `OES_MCP_MAX_JSON_VALUES` stelt een andere grens in.
  - Tools rekenden onbeperkt tegelijk. Er rekenen er nu hoogstens zoveel als de HTTP-API standaard rekenplaatsen heeft; verdere aanroepen wachten. `OES_MCP_MAX_CALCULATIONS` stelt een ander aantal in.
- **Documentatie.** `docs/nta8800-api.md` noemde vier bewerkingen die nooit 422 geven; `compare_project_editions` is de vijfde. `docs/nta8800-mcp.md` beschrijft de verpakking, de waardengrens en de rekenplaatsen.
- **Test.** `every_operation_answers_within_its_documented_contract` (`crates/nta8800-service/tests/api.rs`) stuurt elke bewerking in het proces lege, verkeerde en onbekende leden, een onbekende, numerieke en oudere `normVersion`, de voorbeeldprojecten en -opnamen, een verkeerd `Content-Type` en de verkeerde methode, en controleert dat elk antwoord een gedocumenteerde status en foutcode heeft, dat een 200 de vereiste leden van het resultaatschema heeft en dat geen bericht een paniek of bronpad noemt.

### 11 oktober 2026 — Uitkomst van de basisopname blijft staan na het verlaten van de stap (geen rekenwijziging)
- **Opname doorrekenen** gaf een uitkomst die verdween zodra je naar een andere stap ging: bij terugkomst stond er "Nog niet doorgerekend". De uitkomst wordt nu per document bewaard, met de opname en de NTA 8800-uitgave waarmee ze is berekend, en alleen getoond zolang de opname en de uitgave daarmee overeenkomen. Een wijziging in de opname (een reden bij een standaardwaarde uitgezonderd), een andere uitgave, het verwijderen van de opname of een ander document geeft geen uitkomst; een ander document toont nooit de uitkomst van het eerste. Handleiding hoofdstuk 2 bijgewerkt.

### 11 oktober 2026 — Review 36: basisopname op locatie: vlakken, namen, verwijderen en foto's (geen rekenwijziging)
- **Vlakken van de schil.** De opname had geen knop om een vlak toe te voegen, te hernoemen of te verwijderen: elke opname bleef de zes vlakken van de voorbeeldwoning houden (ook de woningscheidende wand), en een ander vlak kon alleen via de JSON-weergave. Er is nu **Vlak toevoegen**, **Naam van het vlak** en **Vlak verwijderen**. Bij een nieuwe naam gaan ramen, deuren, panelen, lichtkoepels, de vloer bij een geveldeel tegen de grond en de gekoppelde foto's mee; een vlak met ramen, deuren, panelen of lichtkoepels kan niet weg.
- **Namen die halverwege leeg of dubbel zijn.** Wie de naam van een rekenzone wiste en opnieuw typte, of onderweg de naam van een andere zone passeerde (`zone1` op weg naar `zone12`), liet de verwijzingen van vlakken en verlichtingszones achter bij een naam die niet meer bestond. De verwijzingen blijven nu bij de laatste goede naam en volgen zodra de naam weer uniek is; het formulier meldt een lege of dubbele naam.
- **Opname verwijderen** vraagt nu eerst om bevestiging; één klik wiste de hele opname zonder vraag. De koppelingen van foto's naar de opname vervallen; de foto's blijven in het bewijsregister als "niet gekoppeld", zoals bij het verwijderen van één onderdeel.
- **JSON-weergave.** JSON die geen object is (`null`, een lijst, een getal) werd overgenomen en brak daarna het formulier; die wordt nu geweigerd met een melding. Is de opname gewijzigd sinds de JSON-bewerking begon, dan zegt de weergave dat toepassen die wijzigingen overschrijft, met **JSON opnieuw laden**.
- **Foto's.** HEIF-, TIFF- en BMP-foto's tellen als foto (ze konden niet via *Bestaand bewijsstuk koppelen* aan een opname-onderdeel worden gekoppeld); een foto die de browser niet kan tonen (HEIC, TIFF) toont een paperclip in plaats van een gebroken afbeelding.
- Handleiding hoofdstuk 2 bijgewerkt. Test: `src/__tests__/survey-review36.test.tsx`.

### 11 oktober 2026 — Bewijsregister: koppeling vanuit het contextpaneel blijft bewaard bij opslaan van het registratieformulier (geen rekenwijziging)
- Had het registratieformulier niet-opgeslagen wijzigingen in het bewijsregister en koppelde de adviseur intussen een bewijsstuk vanuit het contextpaneel, dan won bij *Opslaan* de lijst uit het formulier en ging de nieuwe koppeling stil verloren (vervolg op review 35). Het formulier voegt het bewijsregister nu per bewijsstuk samen: koppelingen en bestanden die elders zijn toegevoegd blijven, de wijzigingen en verwijderingen van de adviseur ook. Is hetzelfde bewijsstuk aan beide kanten gewijzigd, dan blijft de versie van de adviseur staan en toont het formulier een melding met de bestandsnaam.
- Handleiding hoofdstuk 7 bijgewerkt.

### 11 oktober 2026 — Review 35: registratiegegevens blijven bewaard buiten de stap, bewijsregister en checklist (geen rekenwijziging)
- **Registratieformulier:** de niet-opgeslagen gegevens in werkstap 10 horen nu bij het document, zoals het NTA-concept. Ze gingen verloren zodra de adviseur een andere stap opende; nu blijven ze staan. Sluiten van het tabblad of het programma vraagt eerst of je zonder opslaan wilt sluiten. Registratiegegevens die een andere stap opslaat (de herlabelvergelijking, een koppeling vanuit het contextpaneel) laten de wijzigingen in het formulier staan.
- **Geen schijnwijzigingen:** een veld dat je invult en weer leegmaakt, een vinkje aan en weer uit, of een verklaring k/l die je terugzet naar *niet beantwoord*, telt niet meer als wijziging; *Opslaan* en *Verwerpen* blijven dan uit.
- **Bewijsregister:** in *Onderbouwt (JSON-paden, komma-gescheiden)* kon je geen tweede pad typen, omdat het veld de komma direct weghaalde (`/zones/0, /zones/1` werd één pad `/zones/0/zones/1`). Het veld houdt nu de getypte tekst vast. Het GPS-veld en dit veld tonen na *Verwerpen* weer de opgeslagen waarde.
- **Checklist BRL 9500:** onder het EP-Online-gegevensoverzicht staan nu de antwoorden op de verklaringen k en l, met *niet beantwoord* voor een open vraag. Het exportschema heeft er geen veld voor; de ZIP had ze al.
- Handleiding hoofdstukken 0 en 7 bijgewerkt.

### 11 oktober 2026 — Openbaar geval O: gezondheidszorgfunctie op een warmtenet; vraag 7 aan NEN (geen rekenwijziging)

- **Nieuw geval O.** Een openbaar BENG-rapport van een gezondheidszorgfunctie zonder bedgebied van 451,50 m² op de begane grond van een woongebouw (Uniec 3.1.5.0, 24-02-2023, NTA 8800:2022) is nagebouwd: `training-data/nta8800-public-comparison-o.json`, de test `case_o_healthcare_on_district_heat_under_nta_8800_2022` en `openbaar-o-2022` in de suite van de openbare gevallen (30 vergeleken gevallen, 290 in totaal). Het is het eerste openbare geval met externe warmtelevering met een kwaliteitsverklaring voor verwarming en tapwater, met een gezondheidszorgfunctie en met een aangrenzende onverwarmde ruimte. O rekent ook in de edities-tests mee (`edition_labels.rs`, `edition_deltas.rs`; rij in [normversies](nta8800-normversies.md)).
- **Uitkomst.** Kern 88,25 / 47,75 / 44,2 tegen rapport 87,26 / 49,49 / 45,1, label A+++ in beide. Eis, A_ls/A_g, tapwater, hulpenergie, ventilatoren, verlichting en PV kloppen tot op de kWh; f_prac 0,95 op de verklaring zit in beide. Het hele verschil in BENG 2 (−1,74) zit in de verwarming.
- **Vraag 7 aan NEN.** De verwarmingsbatterij van de LBK rekent met c_a = 0,000 027 9 kWh/(kg·K) zoals gedrukt in 11.115/11.119 (2022 p. 486–487, 2025+C1 p. 505–506), een tiende van de 1 005 J/(kg·K) die de norm elders gebruikt. Met de fysische waarde zou O 49,80 geven (rapport 49,49) en L 23,82 in plaats van 18,63 (rapport 25,36). De kern houdt de gedrukte waarde tot NEN antwoordt; de vraag staat in [vragen aan NEN](nta8800-vragen-nen.md) en in de [brief](nta8800-vragen-nen-brief.md), die nu zeven vragen heeft.
- **Gezocht.** Ook op 11 oktober 2026 is geen openbaar rapport met Uniec 3.4 (uitgave 2025+C1) gevonden; de zoekronde staat in de [vergelijking](nta8800-vergelijking-openbare-rapporten.md).

### 11 oktober 2026 — Review 34: onbetrouwbare invoer bij openen en importeren (geen rekenwijziging)

- **Projectbestand met BOM.** Een projectbestand dat met een Windows-editor opnieuw is opgeslagen begint met een UTF-8 byte-order mark. De browserversie las dat al, maar de desktopapp weigerde het met "Unexpected token". Beide openen het nu.
- **JSON zonder object.** Een bestand met `null`, een lijst of een los getal gaf een onduidelijke JavaScript-fout. Het geeft nu "Invalid file format: expected a JSON object".
- **Te groot importbestand.** Een UNIEC3- of VABI-import weigert nu een bestand boven 256 MB voordat het wordt ingelezen, met een NL/EN-melding die bestand en grens noemt. Zo vult een per ongeluk gekozen video of schijfkopie het geheugen niet.
- **Nagelopen zonder bevinding:** de begrenzing van HTTP- en MCP-verzoeken (grootte, aantal JSON-waarden, regellengte), de uitpakgrenzen van archieven, het lezen van getallen en codes bij UNIEC3 en VABI, de vormcontrole bij openen (een verkeerd type op een veld van de voorbeeldprojecten buiten het NTA-blok geeft nooit een onduidelijke JavaScript-fout) en de robuustheidsfuzz van de rekenkern met 4 000 proeven per fixture.

### 11 oktober 2026 — Review 33: handleiding bijgewerkt voor het testsetdossier en de lege lijst Open punten (geen rekenwijziging)

- **Testsetdossier.** Hoofdstuk 5 beschrijft nu het testsetdossier in het releasearchief: de deeltesten als projectbestanden die je in het programma opent, het overzicht met de kolommen *verwacht* en *oordeel*, en de markering "(met lokale wijzigingen)" bij een proefrun.
- **Lege lijst.** Hoofdstuk 0 noemt nu wat een lege lijst *Open punten* op Controle en Project betekent. "Geen open punten van de rekenkern." staat er alleen als de kern heeft geantwoord; anders staat er dat de kern nog rekent, niet bereikbaar is of nog geen antwoord heeft gegeven. Het contextpaneel heeft eigen teksten, en die staan nu bij *Controle*.
- **Nagelopen zonder bevinding:**
  - verwijzingen, ankers en afbeeldingen;
  - vetgedrukte veld- en knopnamen tegen de Nederlandse teksten van de app;
  - de onderdelen van de afgelopen dagen: wissel van uitgave, gemeenschappelijke ruimten per zone en gebouw, *Overnemen* van vloeren op de grond, opgegeven F_D;dayl, WKK met voorraadvat, wandoppervlakte van de verwarmde kelder, Technische details en de programmaversie in de uitvoer.

### 11 oktober 2026 — Review 32: oordeel in het testsetdossier gelijk aan de vergelijking, dossiermap beveiligd (geen rekenwijziging)

- **Oordeel.** Het testsetdossier telde alleen de getallen na. Een afwijkende labelklasse (`compared_fail` bij getallen binnen de band) stond er daardoor als *binnen de band*. Een deeltest met verwachting die niet rekende, stond er met een leeg oordeel, alsof de verwachting nog ontbrak. Het oordeel komt nu uit de status van de vergelijking: *binnen de band*, *buiten de band* of *niet vergeleken (status)*. De verwachte labelklasse staat in de kolom *verwacht*.
- **Dossiermap.** `reference_gate --testset-dir` weigert nu een `caseId` die geen gewone bestandsnaam is (zoals `../x`), een dubbele `caseId` en een map `projecten/` die al bestanden bevat. Zo komt geen bestand buiten het dossier terecht, overschrijft geen deeltest een andere, en telt het manifest alleen de bestanden van deze run.
- **Commit.** Bij een proefvrijgave uit een werkmap met wijzigingen schrijft `release-nta.sh` in het dossier nu "(met lokale wijzigingen)" achter de commit, net als de gate.
- **Releasetekst.** `github-release.md` kortte de commit af tot 12 tekens en liet daarbij "(met lokale wijzigingen)" weg. Het korte commitnummer staat er nu met die vermelding, zoals in het leveringsdocument, het testrapport en het manifest (`shortCommit`, test in `nta-github-release.test.ts`).
- **Getoetst.** Twee runs van het dossier over alle 260 deeltesten geven bytegelijke bestanden. Tests: `testset.rs` (labelklasse, niet berekend, bestandsnamen), `reference_gate` (oude bestanden, dubbele en onveilige `caseId`) en `ci-parity.test.ts` (commitstempel).

### 10 oktober 2026 — Testsetdossier ISSO 54 voor de attesteringsinstelling (geen rekenwijziging)

- **Waarom.** BRL 9501 §3.3 (p. 5–6): de instelling controleert een deel van de uitkomsten en voert minstens één test per deelgebied zelf in; invoer en uitkomst van elke deeltest moeten beschikbaar zijn.
- **Wat.** `reference_gate --testset-dir <map> [--testset-topics <bestand>]` schrijft per suitegeval een projectbestand dat het programma opent (`projecten/<caseId>.oes.json`, het eigen bestandsformaat met rekenkernstempel) en een uitkomstenoverzicht (`testset.md`, `.csv`, `.json`) met onderwerp, uitgave, status, BENG 1/2/3, TOjuli, geleverde energie per dienst, SHA-256 van invoer en uitvoer, en de kolommen *verwacht* en *oordeel*, die zich vullen zodra een deeltest een verwachting heeft. De onderwerpen per testreeks staan in eigen woorden in `training-data/isso54/onderwerpen.json`.
- **Bij elke vrijgave.** `scripts/release-nta.sh` maakt het dossier in `release/<tag>/testset/` (ook bij `--dry-run`); het manifest telt de projectbestanden en geeft de SHA-256 van het overzicht.
- **Getoetst.** De getallen komen van het projectbestand zoals het is weggeschreven en weer geopend, via de bewerking van de HTTP-route; voor alle 260 deeltesten gelijk aan het referentierapport. `crates/nta8800-service/tests/testset_roundtrip.rs` (twaalf deeltesten, woning en utiliteit) en `src/__tests__/testset-project-files.test.ts` (de app opent drie bestanden en stuurt het project ongewijzigd naar de kern).

### 10 oktober 2026 — UI-tests bestand tegen een belaste machine (geen rekenwijziging)

- **Wat er misging.** In gates naast andere gates faalden UI-tests die los wel slaagden: `ux-findings.test.tsx` (findBy met de standaard 1 s, terwijl een kernantwoord al 400 ms debounce heeft), `workflow-e2e.test.tsx` (rapportvoorbeeld via waitFor met 1 s), `workflow-e2e-040.test.tsx` (testlimiet van 60 s voor een hele werkstroom) en `large-project-ui.test.tsx` (vaste tijdsbudgetten).
- **Gedeelde wachttijden.** `src/__tests__/timeouts.ts`: findBy/waitFor wachten standaard 5 s (`UI_WAIT_MS`, ingesteld in `setup.ts`), de limiet per test is 60 s (`TEST_TIMEOUT_MS`, in `vitest.config.ts`; de gate geeft geen eigen `--testTimeout=20000` meer mee), en lange werkstroomtests hebben één limiet van 180 s (`SLOW_UI_TEST_MS`) in plaats van losse 30 en 60 s. Onder zware belasting overschreed ook de formulierdekkingstest (`nta-form-coverage.test.ts`) de oude 20 s.
- **Tijdsbudget met een basislijn.** `large-project-ui.test.tsx` meet eerst dezelfde pagina voor geval H (2 zones) en staat voor het 20-voudige project het maximum van 15 s en 80 × die basislijn toe. Lineair werk groeit ongeveer 20-voudig, een kwadratische opzoeking per rij ongeveer 400-voudig en faalt dus nog steeds.
- **Getoetst.** De hele UI-suite slaagde drie keer onder kunstmatige belasting (8 cpu-belasters naast 4 Vitest-workers, en twee suites tegelijk), waar hij vóór deze wijziging op twee tests faalde.

### 10 oktober 2026 — Gelijke actieversies in beide workflows; verzendklare brief met de NEN-vragen (geen rekenwijziging)

- **Actieversies.** `release.yml` gebruikt nu dezelfde versies als `ci.yml`: `actions/checkout` en `actions/setup-node` v6, `actions/upload-artifact` v7 en `azure/trusted-signing-action` v1. Bij v1 heet de invoer voor het ondertekeningsaccount `signing-account-name`; die naam staat nu ook in `release.yml`. `src/__tests__/ci-parity.test.ts` eist dat beide workflows elke gedeelde actie met dezelfde versie vastleggen.
- **Brief aan NEN en RVO.** [docs/nta8800-vragen-nen-brief.md](nta8800-vragen-nen-brief.md) zet de zes open vragen uit [nta8800-vragen-nen.md](nta8800-vragen-nen.md) in een formele brief, met per vraag de plaats in de norm (alleen nummers), de twee lezingen, het gevolg in de openbare gevallen en de vraag zelf. De gegevens van de afzender staan als invulvelden. De gereedheidslijst (E3) verwijst ernaar.

### 10 oktober 2026 — CI gelijk aan een verse lokale gate; github-release.md door het vrijgavescript (geen rekenwijziging)

- **CI zonder eigen frontendbuild.** `ci.yml` (gate) en `release.yml` (verify) bouwden de frontend vóór `scripts/verify-nta.sh`. Daardoor bleef verborgen dat de gate in een verse checkout op de Tauri-stap vastliep. Die aparte stap is weg: CI draait nu alleen het gatescript, net als een lokale checkout. `src/__tests__/ci-parity.test.ts` bewaakt dat.
- **Uitgebrachte releasenotes ook in CI.** De verify-stap van `release.yml` controleert nu, net als `scripts/release-nta.sh`, dat de uitgebrachte secties van de releasenotes sinds de vorige tag ongewijzigd zijn.
- **Het `.deb` uit het vrijgavescript blijft staan.** De build-stap van `release.yml` uploadde met `--clobber`, zodat het Linux-`.deb` dat de gebruiker met het leveringsdocument had geüpload werd vervangen door een ander bestand met een andere SHA-256. Bestanden die al bij de release staan, worden nu niet meer vervangen.
- **github-release.md uit het script.** `scripts/release-nta.sh` schrijft de tekst voor de GitHub-release zelf (`scripts/nta-github-release.mjs`): versies, attest-status, de items van de rekenkernsectie en de SHA-256 van pakketten, handleiding, leveringsdocument en testrapport. Stap 5 van de vrijgavelijst in `docs/nta8800-versiebeheer.md` is daarmee een controle in plaats van handwerk.
- **MSRV-cache.** De gate-cache in `ci.yml` neemt `target/msrv-core/` mee.

### 10 oktober 2026 — Proefuitgave 0.3.0 / rekenkern 0.4.0: gate bouwt de frontend vóór Tauri (geen rekenwijziging)

- **Gate in een verse checkout.** `scripts/verify-nta.sh` bouwde de frontend pas als laatste stap, terwijl de Tauri-stappen `dist/` al nodig hebben (`generate_context!`). In een verse checkout, en dus bij `scripts/release-nta.sh` vanaf een schone clone, liep de gate daardoor vast. De stap "frontend bundle" staat nu vóór "Tauri check"; `src/__tests__/gate-order.test.ts` bewaakt die volgorde. De CI-workflows bouwden al vooraf en werken ongewijzigd.
- **Stappenplan.** `docs/nta8800-versiebeheer.md` heeft nu de stappen voor de uitgave programma 0.3.0 / rekenkern 0.4.0, inclusief push en GitHub-release, na een proefrun met de opgehoogde versie (archief, handleiding, pakket en leveringsdocument droegen 0.3.0 / 0.4.0).

### 10 oktober 2026 — Review 31: programmaversie op één plek bijgehouden (geen rekenwijziging)

- **Versie ophogen.** `npm run bump` (`scripts/bump-version.js`) zette de programmaversie in `package.json`, `tauri.conf.json` en `src-tauri/Cargo.toml`, maar niet in `src-tauri/Cargo.lock`. De gate bouwt met `--locked` en zou na het ophogen falen. Het script werkt nu ook de lockfile bij.
- **Eén versie overal.** Nieuwe test `review31-output-identity.test.ts`: de programmaversie is gelijk in `package.json`, het desktoppakket, `Cargo.toml` en `Cargo.lock`, en gelijk aan wat de rapporten en het leveringsdocument noemen. Het leveringsdocument gebruikt voor een ontbrekend attestnummer dezelfde woorden als de rapporten.
- **Getoetst en in orde:** escaping van namen en vakbekwaamheidsnummers van de adviseurs in de rapportage, het rekenrapport en het invoeroverzicht; de Standaard voor Woningisolatie alleen bij woningen; het blok `berekening` laat de XML-vorm van het EP-Online-overzicht ongemoeid. Het attestdossier en hoofdstuk 7 van de handleiding beschrijven het blok `berekening` nu.

### 10 oktober 2026 — Uitvoereisen BRL 9500-W/U en BRL 9501 nagelopen (geen rekenwijziging)

- **Overzicht.** Nieuw document [uitvoereisen](nta8800-uitvoereisen.md): 22 eisen aan de uitvoer, per document met bron (paragraaf en pagina) en status. 20 voldaan, het attestnummer als plaatshouder, de XML-registratie bij EP-Online extern.
- **Programma apart van de rekenkern.** De omslag van de rapportage noemde de rekenkernversie als versie van het programma. Rapportage, rekenrapport, maatwerkadviesrapport en invoeroverzicht noemen nu het rekenprogramma met zijn programmaversie, de rekenkernversie en het BRL 9501-attestnummer apart (Omgevingsregeling art. 5.14 lid 1 onder b; BRL 9501 §5.2 en §6.1).
- **Attestnummer als plaatshouder.** Zolang er geen attest is staat er "nog niet toegekend", ook in het EP-Online-gegevensoverzicht (nieuw blok `berekening`) en in het manifest van het projectdossier (nieuw veld `software`).
- **Standaard voor Woningisolatie in elke rapportage van een woning,** ook in de samenvatting, naast de nettowarmtebehoefte (BRL 9500-W p. 7).
- **Registratieblok altijd in de rapportage.** Zonder registratiegegevens noemt het welke gegevens ontbreken, zoals de adviseurs met hun vakbekwaamheidsnummer (BRL 9500-W §4.2, p. 18).
- **Invoer- en bewijsoverzicht** met opnamedatum en opnemend adviseur, of "niet ingevuld" (BRL 9500-W bijlage 3, p. 62).
- **Maatwerkadviesrapport** noemt de looptijd van de netto contante waarde, de prijsstijging van onderhoud en het basisjaar bij de andere financiële uitgangspunten (BRL 9501 §4.1.3, p. 7).
- **Rekenrapport** toont de atteststatus in het Nederlands.
- Test: `src/__tests__/brl-output-requirements.test.ts`.

### 10 oktober 2026 — Review 30 van de UI-wijzigingen van 10 oktober (geen rekenwijziging)

- **Brede keuzelijsten.** Een keuzelijst die alleen een andere optietekst krijgt (zonder dat de sectie opnieuw tekent) krijgt nu ook twee kolommen; de waarnemer kijkt naar tekstwijzigingen, niet alleen naar toegevoegde elementen.
- **Meervoud.** De samenvatting bij het wisselen van uitgave ("1 punt", "3 punten") en het aantal warmtepompen op Installaties › Overzicht gebruiken nu meervoudsvormen in plaats van "punt(en)" en "warmtepomp(en)".
- **Registratie opslaan.** Is de registratie opgeslagen met een andere rekenkernversie, dan blijft **Opslaan** beschikbaar en zegt de balk "Opgeslagen met een andere rekenkernversie; opslaan legt de huidige vast". Zonder dat bleef het oude programmakenmerk staan zolang er geen veld veranderde. **Verwerpen** blijft uit: er zijn geen gewijzigde velden.
- Test: `src/__tests__/review30-ui.test.tsx`.

### 10 oktober 2026 — UX-bevindingen 1–9 uit de rondgang als nieuwe adviseur (geen rekenwijziging)

- **Technische foutmelding:** de ruwe fout (bijvoorbeeld "Rust API: HTTP 500") staat achter een dichtgeklapte uitklapper *Technische details*; de Nederlandse melding zegt wat er is en wat je kunt doen.
- **Resultaten zonder uitkomst:** **Herberekenen**, en **Ga naar Controle › Controleoverzicht** als de kern de invoer onvolledig of ongeldig vindt.
- **Rapport zonder uitkomst:** één melding; exporteren en afdrukken van het rapport staan uit met uitleg (ook onder Exports), het invoerdossier blijft beschikbaar.
- **Bron & bewijs:** elke regel toont het formulierlabel van het bronveld; het kernpad staat in de tooltip.
- **Aantallen:** meervoudsvormen ("geen fouten", "1 fout", "3 fouten") in plaats van "fout(en)".
- **Voorbeeldprojecten:** de bronteksten zijn Nederlands (alleen tekstvelden).
- **Instelwaarde:** *instelwaarde* in plaats van *setpoint* in de Nederlandse interface en meldingen.
- **Registratie:** **Opslaan** staat uit zolang er niets is gewijzigd.
- **Gaswarmtepomp-referentievergelijking:** de uitgeschakelde knop zegt waarom.

### 10 oktober 2026 — Rondgang als nieuwe adviseur (geen rekenwijziging)

- **Open punten.** De lijst zegt "Geen open punten van de rekenkern" alleen nog als de kern heeft geantwoord; daarvoor staat er "Nog geen antwoord van de rekenkern", "Berekenen…" of "Rekenkern niet bereikbaar".
- **Getallen in de taal van de interface** in de warmtepompdimensionering, de constructie-editor, de installatie-eigenschappen, de vlakeditor en de maandgrafiek (was een decimale punt).
- **Eén naam voor de kern:** *rekenkern* en *rekenkernversie* in de Nederlandse interface, de rapporten en de handleiding. De melding bij een onbereikbare kern noemt Herberekenen; meldingen die naar het oude NTA-paneel verwezen, noemen nu Resultaten, Controle › Controleoverzicht of de stap Project.
- **Handleiding:** paden gelijk aan het programma (Gebouw › Rekenzones, Controle › Controleoverzicht, vloeren op grond onder Gebouw › Schil & ramen).
- Overige bevindingen die een ontwerpbesluit vragen staan in `docs/ui-redesign/ontwerp.md`, "UX-bevindingen (10 oktober 2026)". Test: `src/__tests__/walkthrough-review.test.tsx`.

### 10 oktober 2026 — Keuzelijsten met lange teksten (geen rekenwijziging)

- **Uitgave NTA 8800.** De keuzelijst in het blok *Algemeen* toonde "NTA 8800:2025+C1:2026 (aangew…": de tekst paste niet in één kolom van het formulier. Alleen keuzelijsten van `SelectField` met een tekst boven 34 tekens kregen twee kolommen, en deze tekst telde precies 34 tekens.
- **Alle keuzelijsten.** Elke sectie en de basisopname markeren nu na het tekenen de keuzelijsten waarvan de langste keuze meer dan 26 tekens telt (`markWideSelects` in `NtaFormFields.tsx`). Zo'n keuzelijst krijgt twee kolommen, ook als de sectie haar eigen keuzelijst bouwt; dat waren er ruim dertig, waaronder het type opwekker, de koudeopwekker, de serieopstelling, de ventilatie-invoer en de beweegbare zonwering. Keuzelijsten in een tabelcel houden hun kolom. Onder 720 px vensterbreedte blijft het één kolom.
- **Handleiding.** De schermafdruk `uitgave-keuze.png` (hoofdstuk 10) is opnieuw gemaakt en toont nu het hele blok *Algemeen* met de volledige uitgave.
- **Controle.** `select-width.test.tsx` rendert elke NTA-sectie op beide voorbeelden, in het Nederlands en het Engels, en beide opnames, en faalt op een lange keuzelijst buiten een tabel die niet twee kolommen krijgt. De test faalt zonder de markering. `responsive-layout.test.ts` bewaakt de CSS-regel.

### 10 oktober 2026 — Laatste indelingspunten en de laatste schermafdrukken (geen rekenwijziging)

- **Installaties › Overzicht.** Bij 1024 px liep de rij installatiekaarten door naar een tweede regel die met een losse "›" begon. Het scheidingsteken staat nu achter een kaart; een kaart aan het eind van een regel staat tegen de rand van de lijst, zodat zijn teken wegvalt.
- **Bron & bewijs.** Een regel bevat het pad, de waarde, de bestandskeuze en **Ga naar**, maar het raster had drie kolommen; **Ga naar** stond daardoor op een eigen regel onder elk item. Het raster heeft nu vier kolommen.
- **Opname overnemen.** In *Vloeren op de grond* liep "hoort bij" tegen de keuzelijst aan, en de opsommingstekens onder *Niet overgenomen* stonden buiten het venster. Beide zijn rechtgezet. Het venster is 720 px breed en ziet er bij 1024, 1280, 1366 en 1920 px gelijk uit.
- **Handleiding.** Twee schermafdrukken erbij: de gedeclareerde f_ctrl bij het ventilatiesysteem (hoofdstuk 3) en de keuzelijst *Uitgave NTA 8800* (hoofdstuk 10). Daarmee heeft elk onderdeel in de lijst van de inhoudsopgave een schermafdruk.
- **Controle.** `responsive-layout.test.ts` bewaakt het scheidingsteken en de vier kolommen van *Bron & bewijs*.

### 10 oktober 2026 — Indeling bij smalle en brede vensters (geen rekenwijziging)

- **Bovenbalk onder 1280 px.** De documenttabs werden tot hun eerste letter samengedrukt en de knop voor een nieuw tabblad tot een stip. Zoeken, Opslaan en Herberekenen tonen daar nu alleen hun pictogram; hun tekst blijft de toegankelijke naam. De knop voor een nieuw tabblad krimpt niet meer.
- **Stappenbalk.** In een smal werkgebied liepen de verbindingslijnen door de namen van de stappen (Installaties › Verwarming bij 1024 px). Een stap krimpt niet meer onder zijn naam; de balk schuift horizontaal.
- **Tabellen per raam.** *Belemmering per raam*, *Zonwering per raam* en *Glas per raam* stonden in één veldkolom van 200 px, ook bij 1920 px, met afgebroken raam- en vlaknamen. Ze nemen nu de hele breedte van het blok.
- **Getalvelden in formulieren.** Een getalveld in een formulierregel (bijvoorbeeld Bouwjaar op Registratie) kreeg een tweede rand binnen de eigen rand. Het heeft nu één rand.
- **Controle.** `responsive-layout.test.ts` bewaakt deze regels in de stijlbladen. De handmatige test H19 in het testprotocol doorloopt de schermen bij 1024, 1280, 1366 en 1920 px.

### 10 oktober 2026 — Plaatshouders in teksten en het welkomstscherm in smalle vensters (geen rekenwijziging)

- **Melding na het openen van een oud herlabelproject.** De melding toonde letterlijk "{count}" en "{fields}" in plaats van het aantal gemarkeerde facturen en de nog in te vullen velden. De tekst gebruikt nu de plaatshouders die i18next invult.
- **Vaste controle op plaatshouders.** `i18n-placeholders.test.ts` leest elke `t()`-aanroep in de broncode. Hij faalt als een label met enkele accolades via `t()` wordt ingevuld, als een `{{naam}}` niet wordt meegegeven of een variabele niet wordt gebruikt, en als Nederlands, Engels of een andere taal andere plaatshouders heeft. `i18n-placeholders-rendered.test.tsx` toont elke NTA-sectie, elke werkstap, elk hoofdstuk van de handleiding en de rapporten in het Nederlands en het Engels, en faalt op elke zichtbare `{{…}}` of `{naam}`.
- **Welkomstscherm in vensters smaller dan 1360 px.** Een regel voor smalle vensters zette de kolommen van het venster terug naar die van een open project. Het welkomstscherm kwam dan in de smalle navigatiekolom terecht. De regel voor het lege venster gaat nu altijd voor; `shell-empty-layout.test.ts` bewaakt dat.
- **Handleiding.** De schermafdruk van het welkomstscherm (hoofdstuk 0) is opnieuw gemaakt: de huidige versie, de volle breedte en drie recente projecten.

### 10 oktober 2026 — Negen schermafdrukken erbij in de handleiding; versies in de melding van de handleiding (geen rekenwijziging)

- **Schermafdrukken.** De handleiding toont nu ook de melding "Oudere uitgave — niet voor registratie" onder Resultaten, het formulier van bijlage AA, de voortgangslijst en de uitkomst van de basisopname, *ψ uit de leidinggegevens*, de ketelhulpenergie en WKK-omzettingsgetallen uit een kwaliteitsverklaring, *Bron & bewijs*, *Verklaringen voor het energielabel* en het venster *Handleiding*. De uitkomsten in de afbeeldingen zijn vastgelegde antwoorden van de rekenkern uit `training-data/`. Nog open: de gedeclareerde f_ctrl en de keuzelijst *Uitgave NTA 8800*.
- **Melding in het venster Handleiding.** Bij een andere rekenkern dan waarvoor de handleiding is nagelopen stond er letterlijk "rekenkern {manual} … rekenkern {kernel}", en de verwijzing naar een document toonde "{path}". De teksten gebruiken nu de schrijfwijze van i18next (`{{manual}}`), zodat de versies en het pad erin staan. Test in `manual-view.test.tsx`.

### 10 oktober 2026 — Leesbare waarden en keuzes in formulieren (geen rekenwijziging)

- **Zonwering per raam.** De soorten zonwering met een kleur (screen, uitvalscherm, rolluik) toonden alleen `dark`, `white` of `other`. i18next las de dubbele punt in sleutels als `device.external_screen:dark` als een naamruimte. Sleutels met een dubbele punt werken nu (`nsSeparator: false`).
- **Overnemen in projectmodel, vloeren op de grond.** Getallen staan afgerond in de taal van het programma (R_c 2,99 in plaats van 2,986667; Ψ en U op drie decimalen). Een verwarmde kelder, kruipruimte, randisolatie of randkoudebrug staat per onderdeel met naam in plaats van als JSON. De kolom met de vloerwaarde krijgt zijn ruimte.
- **Bijlage AA.** De keuze van een raam noemt de oriëntatie als woord ("West") in plaats van de code (`W`).
- **Resultaten.** De tabellen *Finaal energiegebruik* en *Energie per energiefunctie* noemen de energiedrager met naam en code ("Aardgas (gas)") in plaats van alleen de code van tabel 5.2.
- **Test.** `select-option-labels.test.tsx` loopt alle keuzelijsten van alle NTA-onderdelen langs, op de voorbeelden en de openbare gevallen A–N, in het Nederlands en Engels, plus de basisopname. Een optie mag geen onvertaalde sleutel, het deel na een dubbele punt of (in het Nederlands) de ruwe waarde tonen. `takeover-value-format.test.tsx` legt de opmaak van de overnamewaarden vast.
- **Handleiding.** `overnemen-vloeren.png` en `zonwering-per-raam.png` staan in de to-dolijst om opnieuw te maken.

### 10 oktober 2026 — Schermafdrukken in de handleiding (geen rekenwijziging)

- **Nieuw in `docs/handleiding-nta8800/img/`:**
  - het venster bij een wissel van uitgave (hoofdstuk 10);
  - de gemeenschappelijke ruimte van een rekenzone (hoofdstuk 3);
  - *Vloeren op de grond* bij *Overnemen in projectmodel* (hoofdstuk 2);
  - de tabellen *Belemmering*, *Zonwering* en *Glas per raam* (hoofdstuk 3).
- **Hoe gemaakt.** Elk onderdeel is los weergegeven met de voorbeeldprojecten of de openbare gevallen D en G, in Firefox zonder scherm. De wissel van uitgave toont een vastgelegd antwoord van de rekenkern op het voorbeeldkantoor (2025+C1 naar 2020+A1).
- **Handleiding.** De afbeeldingen zitten in de HTML-handleiding en in het venster *Handleiding* in de app. De lijst "Nog te doen: schermafdrukken" in de inhoudsopgave is bijgewerkt; de overige onderdelen vragen het volledige venster met een rekenkern en staan er nog.
- **Opnieuw gemaakt na de leesbare waarden.** `overnemen-vloeren.png` toont de afgeronde waarden en de verwarmde kelder per onderdeel, `zonwering-per-raam.png` de naam van de zonwering ("Screen buiten, donker …") in plaats van `dark`. In de tabel *Vloeren op de grond* breekt een afgerond getal niet meer over twee regels (3,87 stond als "3,8 / 7").

### 10 oktober 2026 — Documentatie bijgewerkt na het werk van 10 oktober (geen rekenwijziging)

- **Testprotocol.** Niveau a noemt ~950 unittests van de kern, niveau k ruim 1 300 frontendtests in 139 bestanden. De tests van bronlabels per uitgave, het overzicht bij een uitgavewissel, het overnemen van vloeren op de grond, de toegankelijkheidscontrole en de doorlooptests van kern 0.4.0 staan in de lijsten. Acceptatietest H4 vraagt het overzicht bij een wissel van uitgave; nieuw is H18, de gemeenschappelijke ruimte (ook in het sjabloon van het testrapport).
- **README en wiki.** Gemeenschappelijke ruimten, kelder en grond, opgegeven F_D;dayl, het overnemen van vloeren op de grond, een geveldeel tegen de grond, het overzicht bij een uitgavewissel, bronlabels per uitgave, de 46 bewerkingen van API en MCP, rekentijden en de NEN-vragen 5 en 6.
- **Programmabeschrijving en BRL 9501-gereedheid.** Stand 10 oktober 2026, met de uitgavewissel, de schaal- en gelijkwaardigheidstests, de toegankelijkheidscontrole en de NEN-vragen 5 en 6 (E3).
- **Handleiding.** De inhoudsopgave noemt de gemeenschappelijke ruimten, het overnemen van vloeren op de grond, toegankelijkheid en het overzicht bij een uitgavewissel, plus drie nog te maken schermafdrukken.

### 10 oktober 2026 — Review 29: uitgave wijzigen herschrijft geen bewijs meer (geen rekenwijziging)

- **Beproevingsnorm van tabel 9.28 (punt 67a).** "Automatisch aanpassen" zette de gedateerde NEN-EN 14511-2 van een testrapport op de versie van de nieuwe uitgave. Die norm zegt volgens welke versie de gemeten COP's zijn bepaald; het testrapport verandert niet mee met de uitgave. Het overzicht noemt nu de norm die de nieuwe uitgave vraagt (`choose` met `value`), laat de invoer staan en laat de keuze (een rapport volgens die norm of een andere tabelrij) aan de adviseur. Test `the_table_9_28_test_standard_is_named_but_never_rewritten`.
- **Een veld dat de nieuwe uitgave extra vraagt.** **Ga naar** sloot het venster zonder de uitgave te wijzigen, terwijl zo'n veld vaak pas in de nieuwe uitgave verschijnt (de hoogte h van 8.47 in 2022). Bij die punten staat nu **Wijzigen en ga naar**: dat wijzigt alleen de uitgave (één stap, ongedaan te maken) en gaat dan naar het veld. Test in `nta-edition-switch.test.tsx`.
- **Nagelopen zonder bevinding.** Het overzicht verandert het project in de aanvraag niet en elk automatisch verwijderd of uitgezet lid hoort bij het genoemde omschakelpunt (alle openbare gevallen en beide voorbeelden tussen elk paar uitgaven); de lus is begrensd en zet een teruggezet lid niet opnieuw weg; het nieuwe pad van 8.47 komt overal goed aan (Ga naar, routes, opname); de route valt onder dezelfde grenzen als de andere rekenroutes en de Tauri-opdracht rekent buiten de UI-thread.

### 10 oktober 2026 — Uitgave wijzigen met overzicht vooraf (geen rekenwijziging)

- **Wat.** Wie de uitgave van een project wijzigt, ziet eerst wat de nieuwe uitgave weigert, extra vraagt, niet gebruikt of anders leest. Elke regel noemt de melding, het pad, het omschakelpunt uit `docs/nta8800-normversies.md` en de aanpassing, met **Ga naar**.
- **Automatisch aanpassen.** Alleen veilige, mechanische aanpassingen: een geweigerd lid verwijderen of een keuze uitzetten (de beproevingsnorm van tabel 9.28 wordt sinds review 29 alleen genoemd). Een lid dat daarna verplicht blijkt of waarvan het weglaten het blok onleesbaar maakt, blijft staan. Verplichte waarden worden nooit ingevuld. Wijzigen is één stap in het concept; ongedaan maken herstelt alles.
- **Kern en service.** Nieuwe module `edition_switch.rs` rekent het project in beide uitgaven door en zet veilige aanpassingen door tot er niets nieuws meer komt. `norm_versions/switches.rs` koppelt elk veld van `NormProfile` aan één omschakelpunt; tests bewaken dat elk veld dat tussen uitgaven verschilt en elke genummerde rij van de documentatie een omschakelpunt heeft. Punt 18a (P.3 item 6) is als rij toegevoegd. Nieuwe route `POST /v1/nta8800/project/editions/compare`, MCP-tool en Tauri-opdracht `compare_project_editions`, met een resultaatschema in het OpenAPI-document.
- **Pad van de 8.47-meldingen.** `route_not_in_edition` en `ground_floor_wall_height_required` van een vloer op grond staan nu op `groundFloors[i].below.wallHeightAboveGroundM` in plaats van op de hele vloer, zodat **Ga naar** het veld zelf vindt.
- **Tests.** `tests/edition_switch.rs`: de openbare gevallen A, B, C, D, J en L van hun eigen uitgave naar 2025+C1 en terug, en beide voorbeelden tussen elk paar uitgaven. Elke weigering direct na de omschakeling staat in de lijst, elke regel heeft een omschakelpunt, en na de automatische aanpassingen blijft precies over wat de lijst aan de adviseur laat. Daarnaast `src/__tests__/nta-edition-switch.test.tsx` (venster, beide wijzigknoppen, ongedaan maken, onbereikbare kern, toegankelijkheid) en een API- en Tauri-test.

### 10 oktober 2026 — Verschillen per uitgave op de openbare gevallen nagelopen (geen rekenwijziging)

- **Wat.** De openbare gevallen A–N, de voorbeeldprojecten en ISSO 54 EPW001/EPU001 zijn in alle vijf uitgaven doorgerekend. Elke afwijking van meer dan 0,5 % op een BENG-waarde tussen twee opeenvolgende uitgaven is herleid tot een schakelpunt; de tabel staat in [normversies](nta8800-normversies.md), "Verschillen per uitgave op de openbare gevallen". De genegeerde test `tests/edition_deltas.rs` maakt de tabel opnieuw.
- **Geen rekenfout gevonden.** Elk verschil heeft een schakelpunt in de juiste uitgave en op de juiste post. Voorbeelden:
  - de hulpenergie van 9.85 tussen 2020+A1 en 2022;
  - f_sh;with = 0 bij handbediende zonwering vanaf 2024 (geval F, behoefte −9 %);
  - f_τ van kleine woningen vanaf 2024 (geval M);
  - de afgiftetabellen van 2024 (geval I, koelenergie +21 %).
- **Melding f_BACS.** `bacs_factor_without_capacity_evidence` (§5.5.8) verscheen ook bij een berekening in 2024, 2023, 2022 of 2020+A1, die geen f_BACS kennen. De melding geldt nu alleen in 2025+C1.
- **Reden van een geweigerde forfaitaire warmtepomp.** Wordt de tabelrij ongeldig, dan gaf de projectroute alleen `forfait_lookup_invalid` op `generator.forfait`. Nu staat er ook de oorzaak op het eigen veld, met "Ga naar" naar dat veld. Voorbeelden:
  - `source_temperature_class_mismatch` op `sourceTemperatureC` (een grondwaterbron vraagt vanaf 2023 een brontemperatuur);
  - `high_test_standard_invalid` op `testStandardEdition`.
  Dat geldt ook voor de hybride warmtepomp.
- **Testnorm van tabel 9.28 in het diagnosepaneel.** Het forfaitaire-COP-paneel schreef bij elke uitgave NEN-EN 14511-2:2022 in het testbewijs. NTA 8800:2020+A1 en 2022 noemen de versie van 2007 (2022 p. 14, 316), dus de kern weigerde de hoge-COP-rij daar altijd. Het paneel volgt nu de uitgave van het project. De meldingtekst en het label van het testrapport noemen geen jaartal meer vast.

### 10 oktober 2026 — Review 28: formulieren en rapporten sinds uitgave 0.3.0 (geen rekenwijziging)

- **Getallen bij "Overnemen in projectmodel".** De voorvertoning en de koppeling van vloeren toonden getallen met een decimale punt (38.5); ze volgen nu de taal van het programma (38,5).
- **Kelderwand in de basisopname.** Werd een gevelvlak tegen de grond een ander element of kreeg het een andere grens, dan bleven H1 en de gekozen vloer verborgen in de opname staan. Ze vervallen nu met die wijziging.
- **Gemeenschappelijke ruimte.** De lijst "bedient" bood ook de woonfunctie aan; een woongedeelte is een aparte berekening, dus de lijst biedt nu alleen utiliteitsfuncties, net als de functieoppervlakten.
- **Nagelopen zonder bevinding.** Lijsten met index als sleutel (de getalvelden volgen hun waarde zodra ze niet bewerkt worden), leeg tegenover weggelaten in wat naar de kern gaat, opslaan en openen (het NTA-blok gaat ongewijzigd mee), ongedaan maken, de exportmelding voor Uniec 3/VABI (het NTA-blok staat al in de lijst), ontbrekende vertalingen (de bestaande i18n-toetsen) en HTML-escaping van bronvermeldingen in de rapporten.

### 10 oktober 2026 — Review 27: zoektocht naar lege velden strenger gecontroleerd op het hele blok (geen rekenwijziging)

- **Gevonden.** De controle op het hele blok toetste alle gemelde lege velden alleen samen. Bij een lijst waarvan de elementen van elkaar afhangen (een element dat de volgende elementen andere regels geeft), kon de verkleinde kopie daardoor een leeg veld melden dat het hele blok toestaat. Een invoertype van de kern met zo'n lijst bestaat niet; de unittest `reduced_blank_search_holds_for_a_list_whose_elements_depend_on_each_other` maakt er een na en faalde.
- **Opgelost.** Elk gemeld leeg veld moet het hele blok nu op zichzelf laten falen; anders zoekt de kern op het hele blok. De controle gebeurt ter plekke, zonder kopie per veld: 200 lege velden in het dichte gebouw kosten 1,6 s in plaats van 1,2 s (release-build).
- **Nieuwe toetsen** in `blank_search_equivalence.rs`: lijstvormen (een leeg veld in het eerste element, geneste lijsten, een leeg element of object, elk element leeg, een lege lijst, een leeg type-veld), weggelaten elementen die op zichzelf fout zijn (onbekend veld, ontbrekend verplicht veld, verkeerd type, onbekende variant; de kern valt dan aantoonbaar terug op het hele blok) en `custom_deserialization_in_the_kernel_is_known`, die faalt zodra de kern een eigen leesregel krijgt die nog niet bekeken is. De tellers van de verkleinde zoektocht gelden nu per thread, zodat elke toets alleen zijn eigen zoektochten telt.
- **Nagelopen zonder bevinding.** Geen invoertype van het NTA-blok heeft een eigen `Deserialize`, `deserialize_with` om een lijst, `untagged`, `flatten` of `from`/`try_from` (alleen een scalair BACS-veld en een opnameantwoord); `deny_unknown_fields` en getagde enums werken per element. Controles die elementen vergelijken (unieke id's, verwijzingen naar zones of vloeren, bewijsverwijzingen per index) draaien na het lezen en spelen in de zoektocht geen rol. Het terugzetten van paden (`Shape`) klopt voor geneste lijsten, lege lijsten en een leeg veld in het eerste element.

### 10 oktober 2026 — Lege velden zoeken op een verkleinde kopie van het NTA-blok (geen rekenwijziging)

- **Wat.** De zoektocht naar lege velden (`blank_paths`) draait nu op een kopie van het NTA-blok waarin elke lijst van objecten alleen haar eerste element en de elementen met een leeg veld houdt. Daarna controleert de kern de uitkomst op het hele blok; slaagt dat niet, dan zoekt ze op het hele blok zoals voorheen.
- **Sneller.** In het dichte gebouw van de schaalmeting (release-build): 200 lege velden van 19,6 s naar 1,2 s, 50 van 4,9 s naar 0,26 s. De tijd groeit niet meer met de grootte van het blok.
- **Dezelfde meldingen.** De zoektocht op het hele blok blijft als orakel in de kern. Beide geven dezelfde paden in dezelfde volgorde: getoetst op alle projectfixtures met hun varianten met lege velden, 600 willekeurige sets lege velden, het dichte gebouw en elke formulierstand van de projectformulierwacht. De uitvoer van de 494 snapshotvarianten is byte voor byte gelijk.
- **Gate.** `blanks_in_a_dense_building_are_found_within_budget` zoekt nu 200 lege velden (budget 60 s in een debug-build; gemeten 6,9 s) in plaats van 10. Nieuw: `tests/blank_search_equivalence.rs`.

### 10 oktober 2026 — Schaal en prestaties van de rekenkern (geen rekenwijziging)

- **Gemeten.** Een dicht utiliteitsgebouw is gemeten, in alle vijf uitgaven: 50 rekenzones, 2 250 ramen met eigen zonwering en belemmeringen, 1 200 koudebruggen, 30 PV-systemen, een extra verwarmingssysteem en een gemeenschappelijke ruimte. Het rekent in 0,42–0,48 s (release-build).
  - De rekentijd groeit lineair met zones en ramen: 9 000 ramen kosten 1,5 s.
  - Een opname met 400 ramen rekent in 33 ms.
- **Knelpunt: het opzoeken van lege velden.** Voor elke poging werd het hele NTA-blok gekopieerd en opnieuw gelezen. De zoektocht leest het blok nu zonder kopie. Dat halveert de tijd: 200 lege velden in het dichte gebouw kosten 19,6 s in plaats van 34,6 s.
  - De tijd groeit nog steeds met het aantal lege velden maal de grootte van het blok; de programmabeschrijving (§8) noemt de getallen.
  - De uitvoer van alle voorbeelden, openbare gevallen en hun varianten met lege velden (494 berekeningen) is byte voor byte gelijk gebleven.
- **Bewaking.** In de gate controleert `large_projects.rs` nu ook drie gevallen met een ruim tijdsbudget voor een debug-build: het dichte gebouw, tien lege velden daarin en de opname met 400 ramen.
  - Een eenheidstest bewaakt dat het geleende en het gekopieerde blok dezelfde eerste fout geven.
  - De benchmark (`--ignored`) drukt de nieuwe tabellen af.

### 10 oktober 2026 — Toegankelijkheid en toetsenbord van formulieren en vensters (geen rekenwijziging)

- **Vaste controle.** Een nieuwe test (`src/__tests__/a11y-forms-dialogs.test.tsx`, met de controle in `a11yAudit.ts`) loopt elk NTA-onderdeel op het voorbeeldkantoor en de tussenwoning door (lijsten geopend), beide opnamen, het venster *Overnemen in projectmodel*, de registratiepagina, de instellingen-, project- en opdrachtvensters en alle invoervensters. Hij eist een toegankelijke naam op elk veld en elke knop, unieke id's, aria-verwijzingen die bestaan, `role="dialog"` met `aria-modal` en een titel op elk venster, en een foutmelding die aan het veld gekoppeld is. Nieuwe onderdelen in `NTA_SECTIONS` vallen er vanzelf onder.
- **Ga naar.** Voor elk gelabeld veld van elk onderdeel controleert de test dat **Ga naar** de focus in dat veld zet (een uitgeschakeld veld kan geen focus krijgen en telt niet mee).
- **Vensters en toetsenbord.** Tests op de focusval (Tab en Shift+Tab blijven in het venster), Escape en het terugzetten van de focus op de knop die het venster opende.
- **Gevonden en opgelost.**
  - De velden van de vensters *Luchtdichtheid*, *Koudebrug* en *Feedback* hadden een label dat niet aan het veld gekoppeld was, zodat een schermlezer het veld zonder naam voorlas; het berichtveld, de bijlageknop en de categorieknoppen (nu met `aria-pressed`) van *Feedback* hadden geen naam.
  - Foutmeldingen in de invoervensters (bijvoorbeeld "naam verplicht") waren niet aan hun veld gekoppeld. Het venster koppelt ze nu met `aria-invalid` en `aria-describedby`, en haalt de koppeling weg zodra de fout verholpen is.
  - Het opdrachtvenster verwees met `aria-controls` naar een lijst die er niet is als niets gevonden wordt.

### 10 oktober 2026 — Review 26: overnemen van vloeren op de grond (geen rekenwijziging)

- **Volgorde is geen verschil.** Het venster vergeleek de waarden van de opname en het project als tekst, zodat dezelfde kruipruimte, randkoudebruggen of verwarmde kelder met de leden in een andere volgorde als "het project heeft een andere waarde" verscheen. De vergelijking negeert nu de volgorde.
- **ΔU_for van de adviseur blijft.** De opname neemt ΔU_for van een verwarmde kelder niet mee (de kern leidt die af uit het projectmodel, 8.38). Had de adviseur bij de projectvloer zelf een ΔU_for ingevuld, dan gaf dat een schijnconflict, en overnemen wiste die waarde. Die waarde blijft nu staan en telt niet als verschil.
- **Eén opnamevloer per projectvloer.** Twee opnamevloeren konden aan dezelfde projectvloer gekoppeld worden; de tweede overschreef dan de eerste. Een projectvloer die al bij een opnamevloer hoort, is nu in de keuzelijst van de andere uitgeschakeld (met de naam van die vloer), en een dubbele koppeling wordt genegeerd.
- **Bronvermelding.** Een vloer met de id `begane-grondvloer` werd als al vermeld gezien als de bron al `basisopname, vloer begane-grondvloer-2` noemde. De herkenning vergelijkt nu de hele vermelding.
- **Leesbare namen.** De regels in het venster noemen de waarde in gewone taal ("omtrek P (m)", "verwarmde kelder") in plaats van de veldnaam.
- **Gecontroleerd en in orde:** de koppeling volgt dezelfde vloeren als de kern (grens grond, geen wand of dak); id's van opnamevloeren zijn uniek (`survey_item_id_duplicate`); de waarden hebben in opname en project dezelfde vorm en eenheid; overnemen gaat via het concept en is ongedaan te maken; een opname die niet rekent, biedt geen overname aan.
- **Toetsing.** Vijf nieuwe tests in `src/__tests__/survey-takeover-ground.test.tsx`.

### 9 oktober 2026 — Overnemen in projectmodel neemt vloeren op de grond per waarde mee (geen rekenwijziging)

- **Wat ontbrak.** "Overnemen in projectmodel" laat de geometrie bij het projectmodel, en nam daardoor ook de vloergegevens van de opname niet mee: de omtrek P, een kruipruimte of onverwarmde kelder, en de verwarmde kelder die de opname sinds vandaag maakt van een geveldeel tegen de grond (ISSO 82.1 §8.4, NTA 8800 8.3.3.2). De adviseur moest ze opnieuw invoeren.
- **Nu.** Het venster koppelt elke opnamevloer aan een vloer van het projectmodel (dezelfde id, de enige vloer, of de enige met dezelfde oppervlakte; anders kiest de adviseur) en toont per waarde wat er in `ntaCalculation.groundFloors` verandert. Nieuwe waarden staan aangevinkt; een andere waarde in het project of het verwijderen van een kruipruimte, randisolatie of kelder gaat alleen mee na aanvinken. ΔU_for van de kelderwanden blijft uit het projectmodel komen (8.38 met 8.2.1). De bronverwijzing krijgt "basisopname, vloer <id>" met de bron van de opnamevloer.
- **Toetsing.** `crates/nta8800-core/tests/survey_takeover.rs` controleert dat de opname de vloer in `training-data/nta8800-survey-takeover-ground.json` afleidt en dat een projectvloer van dezelfde oppervlakte met de overgenomen invoer met verwarmde kelder rekent en dezelfde warmteoverdracht naar de grond heeft. `src/__tests__/survey-takeover-ground.test.tsx` controleert dat het venster precies die invoer schrijft en een afwijkende omtrek niet ongevraagd vervangt.
- **Uitkomsten.** Bestaande invoer rekent hetzelfde; alleen wat de adviseur in het venster aanvinkt, verandert het concept.

### 9 oktober 2026 — Doorlooptests van de invoer van kern 0.4.0; drie gaten in scherm en rapport (geen rekenwijziging)

- **Doorlooptests.** `src/__tests__/workflow-e2e-040.test.tsx` doorloopt in de hele app, via de stappagina's en de toepasbalk, de invoer die sinds 0.3.0 is toegevoegd: een gemeenschappelijke ruimte (6.6.4) met door de kern gevulde tapwater- en verlichtingslijsten, de wandoppervlakte van een verwarmde kelder (A_ls en BENG 1-eis), een opgegeven F_D;dayl en de weigering van 1,2, K_pk 350 W/m² met de plausibiliteitsmelding op de registratiepagina, een geveldeel tegen de grond in de basisopname, en een gevel aan de grond in het project (`ground_surface_not_floor`). De kern is in deze tests vervangen door antwoorden in de vorm die de kern teruggeeft; de rekenkant staat in de Rust-tests die per geval genoemd zijn.
- **Eigen gegevens bij één utiliteitsrekenzone.** Het onderdeel "Gegevens per rekenzone" verscheen alleen bij meer dan één rekenzone of als er al eigen gegevens waren. Bij een utiliteitsgebouw met één rekenzone, het gewone geval, waren functieoppervlakten en gemeenschappelijke ruimten (6.6.4) daardoor in de app niet in te voeren. Het onderdeel staat nu bij elk utiliteitsproject.
- **Verdeling van de gemeenschappelijke ruimten in de rapportage.** De verdeling volgens 6.6.4 en de lijsten die de kern invulde stonden alleen in het losse rekenrapport. Ze staan nu ook in de rapportage energieprestatie, in het hoofdstuk "Gebouw en rekenzones" (standaard en gedetailleerd).
- **Invoeroverzicht reikt tot opgegeven waarden in geneste lijsten.** Het invoeroverzicht van de rapportage stopte acht niveaus diep, waardoor bijvoorbeeld een opgegeven F_D;dayl (`lighting[1].lightingZones[1].daylight.sectors[1].daylightDependencyFactor`) ontbrak. Het gaat nu tien niveaus diep.

### 9 oktober 2026 — Review 25: invulling van tapwater en verlichting uit de gemeenschappelijke ruimten nagelopen

- **Lezing bevestigd.** 13.19 en de weging van t_D, t_N en P_n;spec in hoofdstuk 14 nemen de gebruiksoppervlakte binnen de rekenzone, zoals de laatste alinea van 6.6.4 die definieert (2025+C1 p. 158, 538, 656–661; 2023 p. 145, 527, 647–651). NEN-vraag 6 noemt nu de argumenten voor beide lezingen.
- **Ingevulde lijsten dekken de rekenzone.** De eigen oppervlakten plus de gemeenschappelijke ruimten mogen 1 % van de zone afwijken (`common_space_area_mismatch`). Een door de kern ingevulde verlichtings- of tapwaterlijst gaf dan `lighting_function_areas_invalid` of `hot_water_function_areas_invalid` op een veld dat de adviseur niet had ingevuld. De kern schaalt de verdeling nu naar A_g van de rekenzone (14.4, 13.19). Test `filled_hot_water_and_lighting_lists_cover_the_zone_within_the_tolerance`.
- **Geen tapwatervinkje naast meer systemen.** Met extra tapwatersystemen vult de kern de lijst niet (13.20); het formulier biedt het vinkje dan niet meer aan.
- **Rekenrapport.** Het rapport toont de verdeling van de gemeenschappelijke ruimten en welke lijsten de kern heeft ingevuld (paragraaf "Gemeenschappelijke ruimten (6.6.4)").
- **Afgedekt.** Een weggelaten lijst zonder gemeenschappelijke ruimten blijft geweigerd op zijn eigen pad (test `omitted_hot_water_and_lighting_lists_without_common_spaces_are_refused`); de numerieke fuzz bereikt nu ook de door de kern ingevulde lijsten.

### 9 oktober 2026 — Gemeenschappelijke ruimten vullen ook tapwater en verlichting (geen rekenwijziging)

- **Wat.** Met gemeenschappelijke ruimten (`zoneData[].commonSpaces`) vult de kern nu ook een lege lijst `lighting[].functions` en een lege `hotWater.need.areas`, met de verdeling binnen de rekenzone. Een ingevulde lijst gaat voor. De lijsten mogen daarvoor leeg zijn of ontbreken; zonder gemeenschappelijke ruimten blijft een lege lijst geweigerd (`lighting_function_areas_invalid`, `hot_water_function_areas_invalid`).
- **Norm.** 13.19 weegt Q_W;nd;spec naar "de gebruiksfuncties usi in de beschouwde rekenzone" (2025+C1 p. 538). t_D, t_N (p. 657) en P_n;spec (p. 661) wegen naar "de gebruiksfuncties die in de rekenzone zijn gelegen". Dat is de verdeling binnen de rekenzone van 6.6.4 (p. 158). Dezelfde tekst staat in 2020+A1, 2022, 2023 en 2024.
- **Tapwater.** Het tapwater krijgt de verdeling binnen de zone, opgeteld over de zones (`commonSpaces.withinZones`). Dat gebeurt alleen bij één tapwatersysteem: met meer systemen bedient elk zijn eigen deel (13.20).
- **Herleidbaar.** `commonSpaces.filled` noemt elke lijst die de kern invulde, ook `bblFunctions` en `labelFunctions`.
- **Formulier.** Bij tapwater en verlichting staat met gemeenschappelijke ruimten de keuze "Oppervlakten per functie uit de gemeenschappelijke ruimten laten verdelen (6.6.4)".
- **Uniec.** Uniec 3.2.9.2 neemt in geval H de gebouwverdeling. Dat geeft daar 317 kWh meer tapwater en 248 kWh meer verlichting dan de verdeling binnen de zone. De vraag ligt bij NEN ([vraag 6](nta8800-vragen-nen.md)); de fixture van H houdt de gebouwverdeling van het rapport aan.
- **Bestaande projecten.** Die rekenen gelijk: een lege lijst werd eerder geweigerd.
- **Test.** `case_h_common_space_fills_hot_water_and_lighting_within_the_zone`.

### 9 oktober 2026 — Vrijgavecontrole voor rekenkern 0.4.0 (geen rekenwijziging)

- **Releasenotes.** Elke merge sinds `oes-v0.2.0-kernel-v0.3.0` heeft een item onder "Onuitgebracht"; de aanduiding "(geen rekenwijziging)" klopt per item.
- **Versies.** README en wiki noemen naast de uitgave 0.3.0 dat de tak rekenkern 0.4.0 (onuitgebracht) draagt; `docs/nta8800-versiebeheer.md` en het testprotocol noemen 0.4.0 als huidige, onuitgebrachte rekenkern.
- **Aantallen.** Het sjabloon van het testrapport en het kwaliteitshandboek noemen openbare gevallen A–N; de BRL 9501-gereedheid noemt veertien nagerekende rapporten.
- **Handleiding.** De meldingentabel in hoofdstuk 3 noemt `ground_surface_not_floor`.

### 9 oktober 2026 — Review 24: gemeenschappelijke ruimten (6.6.4) nagelopen

- **Labelfuncties.** De kern vulde met de gebouwverdeling alleen de Bbl-functies. Het label (Omgevingsregeling bijlage X) wordt per gebruiksfunctie van het gebouw bepaald, net als de Bbl-eisen; zonder opgegeven labelfuncties rekende het label nu met één functie over het hele gebouw. De kern vult ze nu ook, met dezelfde verdeling (`label_functions_follow_the_building_split`, en in `case_h_common_space_input_equals_the_hand_split`).
- **Dubbele telling.** Functie-oppervlakten waarin de gemeenschappelijke ruimte al zat, telden haar twee keer en werden pas later geweigerd als `function_areas_invalid` op de functies. Nu weigert de kern dit bij de gemeenschappelijke ruimte zelf met `common_space_area_mismatch`: de eigen oppervlakten plus de gemeenschappelijke ruimten moeten de gebruiksoppervlakte van de rekenzone zijn (6.6.3, 2025+C1 p. 158).
- **Zones zonder gemeenschappelijke ruimte.** Een ongeldige functie-oppervlakte in zo'n zone gaf `common_space_exceeds_zone`, soms op het pad van een andere zone. Die zone houdt nu haar eigen melding (`function_areas_invalid`).
- **Zwembad.** Een sportfunctie die in twee ventilatiedelen stond, met en zonder zwembad, kreeg na de verdeling over het hele sportdeel het zwembad (dubbele q_usi;spec, 11.2.2.5.1). De delen houden nu hun verhouding (`allocated_ventilation_keeps_the_swimming_pool_part`).
- **Formulier.** Functie-oppervlakten en gemeenschappelijke ruimten staan alleen in de eigen gegevens van een rekenzone, en die waren bij één rekenzone niet te openen. Dat kan nu. Een zone die een woonfunctie krijgt, houdt geen verborgen gemeenschappelijke ruimten meer.
- **Gecontroleerd en in orde.** Dezelfde tekst van 6.6.4 staat in 2020+A1 (p. 140), 2022 (p. 141), 2023 (p. 145) en 2025+C1 (p. 158); de 90 %-toets volgt na de verdeling; een aangewezen functie die niet in het gebouw zit, valt terecht weg; Uniec 3/VABI en EP-Online dragen geen functie-oppervlakten.

### 9 oktober 2026 — Gemeenschappelijke ruimten als invoer, verdeeld door de rekenkern volgens 6.6.4 (geen rekenwijziging)

- **Waarom.** Openbaar geval H liet zien dat een met de hand verdeelde gemeenschappelijke ruimte makkelijk fout gaat: binnen een rekenzone gaat zij alleen naar de functies van die zone, voor de Bbl-eisen naar alle functies die erop zijn aangewezen (6.2, p. 142; 6.6.4, p. 158).
- **Nieuwe invoer.** `zoneData[].commonSpaces[]` met `areaM2`, `servedFunctions` (leeg = alle functies van het gebouw) en `sourceReference`. Met gemeenschappelijke ruimten zijn de `functionAreas` van de zone de eigen oppervlakten van de functies.
- **Wat de kern doet.** Hij verdeelt elke ruimte naar rato van de eigen oppervlakten: binnen de zone over de aangewezen functies van die zone, voor het gebouw over alle aangewezen functies. Hij schrijft de verdeling in `functionAreas` en de ventilatiefuncties van de zone (een zwembad blijft bij zijn functie) en, als ze ontbreken, in `bblFunctions`. De verdeling staat in de uitkomst onder `commonSpaces`.
- **Weigeringen.** `common_space_invalid`, `common_space_function_outside_zone` en `common_space_exceeds_zone`, met tekst in nl en en.
- **Formulier.** Onder Eigen gegevens voor deze rekenzone kunt u een gemeenschappelijke ruimte toevoegen; de handleiding (hoofdstuk 3) beschrijft het.
- **Gelijkwaardigheid.** `case_h_common_space_input_equals_the_hand_split` rekent geval H met deze invoer en krijgt dezelfde BENG-waarden en Bbl-eisen als met de handmatige verdeling van de fixture. Bestaande projecten zonder `commonSpaces` rekenen ongewijzigd.

### 9 oktober 2026 — Openbaar geval H: de gemeenschappelijke ruimte binnen RZ-1 alleen naar de functies van die zone (geen rekenwijziging)

- **Oorzaak.** De verwarming van H lag 9 % onder het rapport (3 989 tegen 4 382 kWh). De fixture verdeelde de gemeenschappelijke ruimte van 49,20 m² ook binnen RZ-1 over alle vier functies, met 44,35 m² sport in RZ-1. 6.6.4 (2023 p. 145) deelt haar binnen een rekenzone alleen toe aan de functies die in die zone liggen; de sportfunctie ligt in RZ-2.
- **Nieuwe invoer van RZ-1.** Kantoor 25,24, bijeenkomst overig 45,16 en gezondheidszorg overig 47,40 m², alle drie 21 °C (6.5.3, 2023 p. 143–144, en tabel 7.13, p. 197), in plaats van een gewogen 19,12 °C. De ventilatie van RZ-1 rekent met dezelfde functies. De gebouwverdeling over vier functies blijft voor de Bbl-grenswaarden, het tapwater (13.2.3) en de verlichtingszone van de gemeenschappelijke ruimte; daar komen kern en rapport al op de kWh overeen.
- **Gevolg.**
  - De warmte is nu 13 301 tegen 13 366 kWh, de elektriciteit 4 361 tegen 4 382, de hulpenergie 318 tegen 319 en de omgevingswarmte 8 940 tegen 8 984 kWh.
  - H: 30,20 / 81,86 / 30,0 tegen 29,08 / 80,01 / 30,5 (was 27,29 / 80,63 / 29,7).
  - BENG 2 ligt nu 1,85 hoger door koeling (10.15, +1,56), de regelenergie van 10.87 (+0,17) en de ventilatoren (+0,17). BENG 1 ligt 3,9 % hoger; het rapport geeft geen behoefte per zone of dienst om dat te splitsen.
- **Wat verandert.** De fixture van H, de test `case_h_utility_under_nta_8800_2023` (nu ook warmte, hulpenergie, omgevingswarmte en de behoefte van RZ-1), de suite `openbare-gevallen.json` en de documentatie. De rekenkern niet.

### 9 oktober 2026 — Openbaar geval N: warmtebehoefte van het werkelijke systeem verklaard (geen rekenwijziging)

- **Ventilatie.** Elke ventilatie-invoer van de uitdraai is naast de fixture gelegd (systeem, f_ctrl, WTW, LBK, terugregeling, debieten). Ze kloppen. Een eigen herberekening van de massabalans (11.1–11.14) geeft dezelfde infiltratie als de kern: 629,5 m³/h in januari.
- **Infiltratie.** De fixture geeft de infiltratie nu via de referentieroute van 11.86, want de uitdraai zegt "geen meetwaarde". Dat is meerlaags gebouw als geheel, 0,5 × 1,2 × 0,7 = 0,42; het getal blijft gelijk.
- **Oorzaak van het verschil.** De +13 % in de verwarming zit in de verlichtingswarmte. De kern rekent die buiten BENG 1 met 7.28 (f_L × W_t), zoals opmerking 3 bij 5.4.1 voorschrijft. Uniec 3.0.x lijkt daar de vaste q_L van 5.4.2 te gebruiken.
  - Met die waarde komt de kern bij N op −2,3 % en bij I op +1,3 % (nu +13,1 % en −7,1 %).
  - Bij de rapporten uit 2022 en 2023 (H, K) wordt het daarmee juist slechter.
  - De norm is duidelijk, dus de kern verandert niet en er is geen vraag aan NEN nodig.
- **Wat verandert.** De fixture van N, de test `case_n_supermarket_under_nta_8800_2020_a1` (nu ook behoefte, infiltratie, toevoer en verlichtingswarmte) en de documentatie in `nta8800-vergelijking-openbare-rapporten.md`. De rekenkern niet.

### 9 oktober 2026 — Openbare gevallen M en N per term verklaard; N rekent de afgifte van de hoge ruimte met 9.3.3.7 (geen rekenwijziging)

- **M, BENG 3.** E_ren klopt per term met het rapport: omgevingswarmte verwarming 3 817 tegen 3 793 kWh, tapwater 386 in beide, koeling 0 in beide, PV 4 446 in beide; samen 8 649 tegen 8 625 kWh. Met de E_P van het rapport (1 140 kWh) geeft de E_ren van de kern 88,35 % tegen 88,3 %. Het verschil op BENG 3 (80,6 tegen 88,3) volgt dus geheel uit BENG 2: de koeling volgens 10.15 en 10.87. De test controleert nu E_ren, de omgevingswarmte van verwarming en tapwater en BENG 3 met de E_P van het rapport.
- **N, afgifte.** De fixture rekende de luchtverwarming van de 6–8 m hoge winkel met 9.3.3.4 (woningen tot 4 m, 2,0 K). Voor een ruimte hoger dan 4 m gelden de waarden van 9.3.3.7 (2020+A1 p. 277–280): Δθ_ctr,1 2,5 K van tabel 9.10 (gelijk aan de uitdraai) en Δθ_str van 9.19 met de hoogste gradiënt van tabel 9.8 (toevoerpositie onbekend), bij h_R 7 m samen 4,0 K. Het afgifteverlies staat daarmee in elke maand op de grens 0,15 van 9.16 (p. 268), zodat h_R binnen de klasse niet uitmaakt. N gaat van 54,57 / −24,99 / 123,6 naar 54,57 / −24,31 / 122,6, tegen 55,98 / −27,18 / 126,9 in het rapport. Het verschil in verwarming (nu 13,1 % meer warmte) zit daarmee in de warmtebehoefte van het werkelijke systeem, niet in de afgifte; de uitdraai geeft geen behoefte om het verder te verklaren.
- Alleen de fixture van N, de verwachte waarden in `public_comparison.rs` en de suite `openbare-gevallen.json` veranderen; de rekenkern niet.

### 9 oktober 2026 — Opname: geveldeel tegen de grond als kelderwand; grondgegevens bij Uniec 3/VABI genoemd

- **Opname (ISSO 82.1/75.1).** Een geveldeel met grens *grond* werd als eigen vloer op grond doorgerekend (met een omtrek die het formulier niet vroeg). Het gaat nu, zoals ISSO 82.1 7e druk §8.4 (p. 74–75; ISSO 75.1 7e druk p. 78–79) vraagt, samen met zijn vloer: nieuwe antwoorden `depthBelowGroundM` (H1) en `groundFloorId`. De vloer krijgt een `heatedBasement` (NTA 8800 8.3.3.2, 8.38) met z volgens 8.42 uit H1 (ℓ_j = A_j/z_j), het wandoppervlak ΣA_T;bw (ook in A_ls, 6.7.2), de R_c van het geveldeel (oppervlaktegewogen 1/R_c) en ΔU_for. Een dak met grens *grond* wordt geweigerd (`ground_surface_not_floor`), het formulier biedt die grens bij een dak niet meer aan. Nieuwe codes `depth_below_ground_required`/`_invalid`, `ground_wall_floor_required`/`_unknown`. Tests: `a_facade_part_against_the_ground_makes_its_floor_a_heated_basement`, `a_ground_wall_needs_h1_and_its_floor_and_a_roof_cannot_border_the_ground` en `a_facade_part_against_the_ground_calculates_with_its_floor_in_every_edition` (vloer op grond en boven kruipruimte, vijf uitgaven), plus formulier- en null-padtests. Uitkomsten veranderen alleen voor een opname met een geveldeel tegen de grond.
- **Uniec 3 en VABI.** Geen van beide formaten bevat, voor zover hun bestanden bekend zijn, de grondroute (grens per vlak, perimeter, kruipruimte, verwarmde kelder); zonder echte voorbeeldbestanden zijn er geen veldnamen om op af te beelden. De exportmelding noemt nu `ntaCalculation.groundFloors` apart en het importpunt noemt de vloeren op grond; handleiding hoofdstuk 9 en de wiki zeggen het.

### 9 oktober 2026 — Review 23: een wand of dak aan de grond wordt geweigerd

- **Wand of dak met begrenzing "grond".** Hoofdstuk 8.3 kent alleen een grondroute voor vloeren. Wanden onder het maaiveld zijn de wanden van een verwarmde kelder (8.3.3.2, 2025+C1 p. 255) en worden bij de kelder van de vloer ingevoerd. Een wand- of dakvlak met begrenzing grond kreeg tot nu toe de vloerformules. Naast een verwarmde kelder telde het dubbel mee in H_g en in A_ls. De kern weigert zo'n vlak nu met `ground_surface_not_floor` op de begrenzing van het vlak ("Ga naar" leidt naar de gebouwschil). Een project dat zo'n vlak had, rekent pas weer als de begrenzing is aangepast. Test: `a_wall_or_roof_to_the_ground_is_refused`.
- **Formulier.** Het veld voor ΣA_T;bw zegt nu dat de oppervlakte in A_ls en in 8.38 telt, en dat 8.39 met z × P rekent.
- **Gecontroleerd en in orde:** de bronvermeldingen per uitgave (`cite`, `leak_once`: de sleutels botsen niet), het EP-Online-overzicht in een oudere uitgave, en de kelderwanden in A_ls bij meer zones en bij een onverwarmde kelder.

### 9 oktober 2026 — Bronvermeldingen noemen de doorgerekende uitgave (geen rekenwijziging)

- **Wat was er mis.** In een berekening in een oudere uitgave noemden `chapter5Source`, `chapter9Source`, `draftSource` en `consultationSource` de paragraaf- en paginanummers van NTA 8800:2025+C1:2026, bijvoorbeeld "NTA 8800:2025+C1:2026, §9.3 … (p. 294–297)" bij een 2020+A1-berekening. De opname noemde bij H2 onder 2020+A1 de bron van 2022, en bij f_BACS onder een oudere uitgave de 2025+C1-paragraaf als bron.
- **Nu.** In de aangewezen uitgave blijft de verwijzing gelijk. In een oudere uitgave noemt ze die uitgave en het hoofdstuk ("NTA 8800:2020+A1:2020, hoofdstuk 9"): de hoofdstukken zijn in alle uitgaven gelijk, de paragraaf-, formule- en paginanummers niet (`norm_versions::cite`). Bij f_BACS staat in een uitgave zonder 5.5.8 dat die uitgave geen f_BACS kent.
- **Rapporten.** Het voorblad van de energieprestatierapportage, de kernmelding van het BENG-rapport, het maatwerkadvies, de doeluitgave in het invoerdossier en de EP-Online-toelichting noemen de uitgave van de uitkomst, niet vast NTA 8800:2025+C1:2026.
- **Test.** `tests/edition_labels.rs` rekent de voorbeelden, de openbare gevallen A–N en ISSO 54 EP-W001/EP-U001 in alle vijf uitgaven, de 260 ISSO 54-deeltesten in 2022 en de zes opnames in alle vijf uitgaven, en loopt de hele uitvoer door: geen tekst noemt "NTA 8800:<jaar>" van een andere uitgave. Bronverwijzingen die de adviseur zelf invoerde, blijven buiten beschouwing. Frontend: `report-edition-labels.test.ts`.

### 9 oktober 2026 — Openbaar geval I per term verklaard: BENG 2 en BENG 3 komen uit de koeling (geen rekenwijziging)

- **Uitkomst.** Geval I (Uniec 3.0.9.3, 2020+A1) blijft 75,51 / −19,41 / 119,4 tegen 75,26 / −36,81 / 143,8; de fixture had geen codeerfout en de kern geen fout.
- **BENG 2** (+17,40): de ontvochtiging Q_C;dhum van 12.5 is 13 885 kWh koude (+15,29), het verlies van 10.15 letterlijk 4 186 kWh (+4,61) en de voelbare behoefte −1,78 tegen de 12 069 kWh koude van het rapport. Directe expansie in de ruimte ontvochtigt volgens 12.3 en OPMERKING 2 bij de koudeafgifte (2025+C1 p. 371 en 523–524; 2020+A1 p. 347 en 500–501); de uitdraai wijst erop dat Uniec 3.0.9.3 daar geen ontvochtiging telde. Geen open lezing, dus geen nieuwe vraag aan NEN.
- **BENG 3**: de hernieuwbare termen komen overeen (E_ren 52 443 tegen 53 054 kWh; PV gelijk, omgevingswarmte 8 042 tegen 8 653). Met de E_P van het rapport gaf de kern 144,5 % tegen 143,8 %; het verschil loopt dus volledig via E_P uit de koeling.
- `case_i_utility_under_nta_8800_2020_a1` controleert nu ook de koudetermen, E_ren en de omgevingswarmte. Zie [vergelijking openbare rapporten](nta8800-vergelijking-openbare-rapporten.md), geval I.

### 9 oktober 2026 — Opgegeven kelderwandoppervlakte ΣA_T;bw in A_ls en 8.38; afronding van P_n gecontroleerd

- **Kelderwandoppervlakte (6.7.1/6.7.2, 2025+C1 p. 161–162; 8.38, p. 253–254; 2020+A1 p. 144 en 225–226).** A_ls telt de geprojecteerde oppervlakte A_T volgens bijlage K, en de forfaitaire route 8.38 rekent de kelderwanden over ΣA_T;bw volgens K.1.2. Nieuw is het optionele veld `heatedBasement.wallAreaM2`: is het opgegeven, dan telt het in A_ls (f_ls 0,7) en in beide wandtermen van 8.38; zonder opgave blijft z·P gelden. De gedetailleerde 8.39 en de maandtermen van bijlage D blijven z·P gebruiken. Een opgave van 0 of minder wordt geweigerd. Formulier: "Oppervlakte kelderwanden onder maaiveld ΣA_T;bw" bij de verwarmde kelder. Dit verandert de uitkomst van projecten met een verwarmde kelder in de forfaitaire route die de oppervlakte opgeven.
  - Openbaar geval G geeft nu de 130,45 m² uit de uitdraai: A_ls 694,21 → 670,80 m² en eis BENG 1 66,42 → 64,52, beide gelijk aan het rapport. BENG 65,42 / 29,71 / 72,1 → 65,07 / 29,31 / 72,5 (rapport 64,52 / 29,44 / 70,6).
  - Tests: `heated_basement_forfait_takes_the_entered_wall_area` (ground.rs), uitgebreid `heated_basement_walls_count_in_the_loss_area` (project_performance.rs), `case_g_under_nta_8800_2024` controleert nu ook de eis, en een formuliertest in `nta-product-generators.test.tsx`.
- **Afronding van P_n gecontroleerd, geen wijziging.** Bij geval N rondt de kern P_n naar boven af (17 546 → 18 000 W) en Uniec niet (+0,99 op BENG 2). De zin "Rond de aldus bepaalde waarde Pn naar boven af op twee significante cijfers volgens bijlage X" staat in 14.3.2, de route met het werkelijk geïnstalleerde vermogen, in elke uitgave (2020+A1 p. 631, 2023 p. 646, 2025+C1 p. 660). De afronding geldt dus juist voor een opgegeven vermogen. Dat is geen open lezing; de kern blijft afronden.

### 9 oktober 2026 — Wanden van een verwarmde kelder in A_ls; openbaar geval N, een supermarkt in 2020+A1

- **A_ls met kelderwanden (6.7.3 c, p. 162; 2020+A1 p. 144).** De wanden van een verwarmde kelder (`groundFloors[].heatedBasement`, z·P van 8.39) telden niet mee in de verliesoppervlakte. Ze grenzen aan de grond en tellen nu mee met f_ls 0,7, in de Bbl-toets en in de geometrie voor de registratiecontroles. Projecten met een verwarmde kelder krijgen een hogere A_ls/A_g, en dus een andere eis voor BENG 1 (bij een woning ook een andere energiebehoefte-eis). De BENG-waarden zelf veranderen niet. Openbaar geval G: A_ls 579,48 → 694,21 m² (rapport 670,80), eis BENG 1 57,10 → 66,42 (rapport 64,52). Test: `heated_basement_walls_count_in_the_loss_area` (project_performance.rs).
- **Nieuw geval N.** Een openbaar BENG-rapport van een supermarkt van 2 064,26 m² boven een sterk geventileerde parkeerkelder (Uniec 3.0.7.1, 02-02-2021, NTA 8800:2020+A1) is nagebouwd: `training-data/nta8800-public-comparison-n.json`, de test `case_n_supermarket_under_nta_8800_2020_a1` en `openbaar-n-2020a1` in de suite van de openbare gevallen (29 vergeleken gevallen, 289 in totaal).
- **Uitkomst N.** De kern rekent 54,57 / −24,99 / 123,6 tegen 55,98 / −27,18 / 126,9. A_ls/A_g (2,64), de Bbl-eisen, tapwater, de COP 2,80 en PV op de meter kloppen. Op BENG 2: verwarming +2,03 (9,9 % meer warmte, oorzaak niet uit de uitdraai te halen), koeling +1,07, verlichting +0,99 (P_n naar boven afgerond volgens bijlage X), 10.87 +0,06 en ventilatoren −2,02.
- **Nieuwe vraag aan NEN.** Uniec rekent de forfaitaire ventilatoren (11.142; 2020+A1 11.139) met q_V;ODA;req van de koudebalans, de kern met die van de warmtebalans. Vraag 5 in [vragen aan NEN](nta8800-vragen-nen.md).

### 9 oktober 2026 — Handleiding en formulier bijgewerkt voor rekenkern 0.4.0 (geen rekenwijziging)

- **Handleiding, hoofdstuk 3.**
  - Een WKK voor tapwater met een indirect verwarmd vat: waar je het vat toevoegt, en dat de kern het vatverlies één keer meetelt (13.8.4.7.4).
  - De PV-grenzen voor een opgegeven K_pk: een waarschuwing boven 300 W/m² en een weigering boven 400 W/m², met het datablad als bewijs.
  - De keuze **opgegeven F_D;dayl** bij de daglichtsectoren: welk bewijs nodig is, en dat er geen keuze voor de daglichtregeling is.
  - `pv_peak_power_invalid` en `lighting_daylight_sector_invalid` staan nu in de tabel met meldingen.
- **Formulier en meldingen (nl/en).**
  - Het veld K_pk noemt het datablad en beide grenzen.
  - De waarschuwing `plausibility_pv_peak_power` zegt wat je controleert.
  - De factor F_D;dayl zegt dat de daglichtregeling erin zit.
  - `lighting_daylight_sector_invalid` dekt ook een opgegeven factor.
  - Test: `nta-kernel-040-messages.test.ts`.
- **Aantallen.** Het testprotocol en de BRL 9501-gereedheid noemden nog 240 ISSO 54-deeltesten; het zijn er 260 (203 woning, 57 utiliteit).

### 9 oktober 2026 — Openbaar geval M: kleine vrijstaande woning met koeling in 2022 (geen rekenwijziging)

- **Nieuw geval.** Een openbaar BENG-rapport van een vrijstaande woning met kap van 43,44 m² (Uniec 3.1.3.1, 15-09-2022, NTA 8800:2022) is nagebouwd als geval M: `training-data/nta8800-public-comparison-m.json`, de test `case_m_small_dwelling_with_cooling_under_nta_8800_2022` en `openbaar-m-2022` in de suite van de openbare gevallen (28 vergeleken gevallen, 288 in totaal).
- **Uitkomst.** De kern rekent 170,87 / 47,85 / 80,6 tegen 171,29 / 26,25 / 88,3. Verwarming, tapwater, ventilatoren, PV en de Bbl-eisen kloppen. Het verschil op BENG 2 komt geheel uit de koeling: 10.15 letterlijk (+18,26) en de regelenergie van 10.87 (+2,92). De koude van het rapport is de behoefte van de kern plus 16 %.
- **Zoekronde.** Ook nu geen openbaar rapport in 2025+C1 (Uniec 3.4). Een supermarkt in 2020+A1 met volledige uitdraai is gevonden ([vergelijking](nta8800-vergelijking-openbare-rapporten.md)); die is later nagebouwd als geval N.

### 9 oktober 2026 — Review 22: EP-U602h-daklichten zonder afronding, WKK-vat en opgegeven daglichtsector nagelopen (geen rekenwijziging)

- **ISSO 54 EP-U602h.** De bovenste zone (daklichten, F_D;S 0,68) was als opgegeven F_D;dayl gecodeerd, waardoor de kern 0,5036 afrondde naar 0,50 en F_D 0,75 gaf in plaats van 0,7518. F_D;S 0,68 is de waarde van tabel 14.8 voor 2 % ≤ D < 4 % (2025+C1 p. 674; tabel 14.9 in 2022, p. 650), dus die sector gaat nu door de gewone daklichtroute (14.31–14.36, 14.41/14.42) en geeft 0,7518 exact. Alleen F_D;S 0,3 (EP-U602g en de onderste zone van h) ligt buiten bereik van 14.37 en blijft op de route van NEN-EN 15193-1 (F_D 0,89 tegen 0,8905). Test: `isso54_u602h_rooflight_sector_gives_the_unrounded_14_36_factor` (lighting.rs).
- **Opgegeven daglichtsector.** Een `declared` sector met een daglichtregeling wordt geweigerd in plaats van stil genegeerd: de regeling zit al in F_D;dayl (14.36, 2025+C1 p. 671–672). Test: `declared_sector_takes_no_daylight_control`. De afronding naar beneden op twee significante cijfers en de samenstelling van F_D (14.24–14.26) kloppen met de norm.
- **WKK met voorraadvat.** Nagelopen tegen 13.8.4.7.4 (2025+C1 p. 643–644): het vatverlies telt één keer mee in de opwekkeruitvoer (13.7), en ε_th en ε_el van de WKK gelden voor die uitvoer, zoals bij een ketel. De test `hot_water_chp_method_2_uses_the_ht_conversion_factors` legt dat nu exact vast.

### 9 oktober 2026 — Opgegeven F_D;dayl per daglichtsector (NEN-EN 15193-1 bijlage F); ISSO 54: 260 van de 266 deeltesten

- **Nieuwe daglichtroute.** Onder de tabel met de daglichtregelingfactor F_D;C staat een alternatief: F_D;dayl;i van een daglichtsector mag worden bepaald met bijlage F van NEN-EN 15193-1:2017, naar beneden afgerond op twee significante cijfers (2025+C1 p. 672; 2022 p. 648; dezelfde tekst in 2020+A1, 2023 en 2024). De kern kende die route niet. Een daglichtsector kan nu van soort `declared` zijn, met de sectoroppervlakte `areaM2`, `daylightDependencyFactor` (0–1) en een verplichte `sourceReference`. De kern rondt naar beneden af op twee cijfers (0,781 wordt 0,78); tabel X.1 geldt hier niet, omdat de alinea bijlage X niet noemt (interpretatie in `lighting.rs`). Bestaande berekeningen veranderen niet.
  - Formulier: "opgegeven F_D;dayl" bij de soort daglichtsector, zonder keuze voor de daglichtregeling (die zit in de opgegeven factor).
  - Tests: `declared_sector_takes_the_15193_annex_f_factor_rounded_down` (lighting.rs), een route in de numerieke fuzz, een donor in de optiedekking en een formuliertest.
- **ISSO 54 v2.0.** EP-U602g en h zijn met deze route gecodeerd: 260 van de 266 deeltesten, alle `pending_expectation`. Te toetsen tegen versie 5.0: de test geeft F_D;dayl 0,781 en 0,5036 onafgerond, de kern rekent met 0,78 en 0,50 ([voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md)). Open blijven zes deeltesten: EP-W203q, EP-W406w en EP-W406x (kwaliteitsverklaring niet in bezit) en EP-W204d, EP-W406j en EP-W406l (geen route in de norm).
- **Voorraadvat bij een WKK in het formulier.** Het tapwaterformulier biedt een voorraadvat bij elke opwekker, dus ook bij een WKK (13.8.4.7.4); een formuliertest legt dat vast.

### 9 oktober 2026 — ISSO 54: 258 van de 266 deeltesten; voorraadvat bij een WKK voor tapwater

- **Voorraadvat bij een WKK voor tapwater.** De kern weigerde een indirect verwarmd voorraadvat naast een WKK voor tapwater (`hot_water_storage_in_generator_efficiency`). Volgens 13.8.4.7.4 (2022 p. 621) volgen de verliezen van zo'n vat uit 13.6, en opmerking 1 van 13.6.2 (2022 p. 547) zondert een WKK niet uit. Het vat wordt nu toegestaan en gerekend, zoals bij een ketel. Een berekening die al rekende, verandert niet; alleen de eerder geweigerde invoer rekent nu. Test: `hot_water_chp_method_2_uses_the_ht_conversion_factors`.
- **ISSO 54 v2.0.** EP-W104a–g, EP-W406o, p en t en EP-W407a–i zijn gecodeerd: 258 van de 266 deeltesten, alle `pending_expectation`. De acht open deeltesten zijn niet te coderen:
  - EP-W203q, EP-W406w en EP-W406x vragen een kwaliteitsverklaring uit een apart bijlagedocument dat niet in bezit is;
  - EP-W204d, EP-W406j en EP-W406l hebben geen route in de norm;
  - EP-U602g en h geven daglichtfactoren zonder geometrie.

  Codering en aannames: [voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md).

### 9 oktober 2026 — Review 21: PV-grenzen in het interpretatieregister, CI op Node 22 (geen rekenwijziging)

- **Interpretatieregister.** De grenzen voor een opgegeven K_pk (weigeren boven 400 W/m², waarschuwen boven 300 W/m²) zijn keuzes van het programma; 16.4 kent geen maximum (2025+C1 p. 679–680). Ze staan nu in de interpretatielijst van hoofdstuk 16 (rapportbijlage en attestdossier) en in de drempels van de plausibiliteitscontrole in [het attestdossier](nta8800-attestdossier.md).
- **Test over de projectroute.** `declared_pv_peak_power_warns_on_the_project_route` (project_performance.rs): 290 W/m² rekent zonder melding, 310 W/m² rekent en geeft `plausibility_pv_peak_power` op het veld, 401 W/m² wordt geweigerd (`pv_peak_power_invalid`).
- **CI.** `ci.yml` draait de gate nu met Node 22, net als de lokale gate en `release.yml`; README en wiki zeggen dat.

### 9 oktober 2026 — Openbare gevallen A–H, K en L naast hun uitdraai gecontroleerd (geen rekenwijziging)

- **Wat.** Na de rijfouten van tabel 7.10 bij I, J en K is van elk ander openbaar geval de letterlijke invoer vergeleken met de uitdraai:
  - bouwwijze en plafond;
  - Rc, U en g;
  - q_v;10;
  - ventilatievariant en rendement van de warmteterugwinning;
  - opwekkers en tapwater;
  - PV;
  - gebruiksoppervlakten.

  Geen geval staat op een verkeerde rij, en er is geen invoerfout die de uitkomst raakt.
- **Aangepast, zonder effect op de uitkomst:**
  - De klassen van tabel 7.11 en 7.12 volgen nu letterlijk de uitdraai. D, E en J hebben wand zwaar (dragend metselwerk); E en H hebben vloer zeer zwaar (massieve betonvloer). Bij D en E beslist de massa per m², bij H en J blijft de rij gelijk.
  - Het projectmodel van E heeft nu q_v10 0,63, zoals de rekeninvoer en de uitdraai. Dit raakt alleen de Uniec 3-export.
- **Uitkomsten.** Alle vastgelegde uitkomsten zijn ongewijzigd. Zie [vergelijking openbare rapporten](nta8800-vergelijking-openbare-rapporten.md), "Controle van de letterlijke invoer".

### 9 oktober 2026 — Openbare gevallen I en J: rij van tabel 7.10 gecorrigeerd (geen rekenwijziging)

Alleen de herbouw van twee openbare gevallen (2020+A1) is gecorrigeerd; de rekenkern is niet gewijzigd. Dezelfde soort fout als eerst bij K: de bouwwijze uit de uitdraai was op de verkeerde rij van tabel 7.10 gezet (2020+A1 p. 179–180).

| Geval | Bouwwijze in het rapport | Rij was → is | Rapport (BENG 1 / 2 / 3) | Kern was | Kern nu |
|---|---|---|---|---|---|
| **I**, kantoor met bijeenkomstfunctie | staalskelet met staalbeton of niet-massieve betonnen vloeren, 110 kJ/(m²K) | < 250 → 250–500 kg/m² | 75,26 / −36,81 / 143,8 | 81,02 / −13,31 / 112,3 | 75,51 / −19,41 / 119,4 |
| **J**, winkelruimte | dragend metselwerk met massieve betonnen vloeren | 500–750 → > 750 kg/m² | 63,92 / 46,11 / 30,1 | 67,19 / 54,49 / 26,7 | 63,75 / 51,46 / 27,9 |

- **BENG 1** van beide gevallen ligt nu binnen 0,3 % van het rapport (was +7,7 % en +5,1 %).
- **BENG 2** blijft hoger dan het rapport door koeling (ontvochtiging bij directe expansie en de letterlijke lezing van 10.15); zie [vergelijking openbare rapporten](nta8800-vergelijking-openbare-rapporten.md).
- **Suite en tests.** `openbare-gevallen.json` (`openbaar-i-2020a1`, `openbaar-j-2020a1`) en `case_i_utility_under_nta_8800_2020_a1`, `case_j_retail_under_nta_8800_2020_a1` hebben de nieuwe waarden.

### 9 oktober 2026 — PV: opgegeven piekvermogen per m² tot 400 W/m², waarschuwing boven 300 W/m²

- **Wat.** De kern weigerde een opgegeven K_pk boven 300 W/m² (`pv_peak_power_invalid`). Die grens stond niet in de norm: 16.4 kent geen maximum (2020+A1 p. 651, 2022 p. 656–657, 2025+C1 p. 679–680). Openbaar geval L (Uniec 3.1.5.0) gaf 310 Wp/m² op.
- **Nu.** De kern weigert pas boven 400 W/m² (40 % moduulrendement, een invoerfout). Tussen 300 en 400 W/m² rekent hij en geeft de registratiecontrole de plausibiliteitswaarschuwing `plausibility_pv_peak_power`; die blokkeert niet. Beide grenzen zijn keuzes van het programma.
- **Gevolg voor opgeslagen projecten.** Een project met een opgegeven K_pk tussen 300 en 400 W/m² was onvolledig en rekent nu. Andere uitkomsten veranderen niet.
- **Geval L** neemt nu letterlijk 310 Wp/m² over 203 m² over (was 300 W/m² over 209,77 m²): PV 46 017 kWh, gelijk aan het rapport (was 46 018). BENG 1/2/3 veranderen niet.
- **Rekenkern.** `KERNEL_VERSION` 0.3.0 → 0.4.0; de handleiding is opnieuw gestempeld.
- **Tests.** `declared_peak_power_has_no_norm_maximum` (pv.rs) en de uitgebreide `plausibility_warnings_never_block` (registration.rs).

### 9 oktober 2026 — Openbare gevallen K en L: sporthal en basisschool in 2022 (geen rekenwijziging)

Twee nieuwe utiliteitsgevallen uit de periode van NTA 8800:2022 (Uniec 3.1.5.0). Ze zijn als regressie op de eigen uitkomst opgenomen, met de verschilanalyse per post in [vergelijking openbare rapporten](nta8800-vergelijking-openbare-rapporten.md).

| Geval | Rapport (BENG 1 / 2 / 3) | Kern | Eisen |
|---|---|---|---|
| **K**, sporthal met kantoor en bijeenkomstfunctie | 29,68 / −0,79 / 101,0 | 29,57 / −0,90 / 101,1 | gelijk aan het rapport |
| **L**, basisschool met gymzaal (eerste onderwijsfunctie) | 111,52 / 25,36 / 76,9 | 111,44 / 18,63 / 80,2 | gelijk aan het rapport |

- **Eisen.** In beide gevallen geeft de kern dezelfde Bbl-eisen per gebruiksfunctie als het rapport, en dezelfde A_ls/A_g.
- **Verschil bij L.** BENG 2 van L ligt 6,73 lager door de verwarming. Dat verschil is uit de uitdraai niet te herleiden (vertrekhoogte "> 8 m", luchtbehandelingskast); een kernfout is niet gevonden.
- **Rekenkern.** Geen wijziging, `KERNEL_VERSION` blijft 0.3.0.
- **Suites en tests.** De referentiesuite `openbare-gevallen.json` heeft nu 14 gevallen; de gate vergelijkt 27 gevallen. De tests zijn `case_k_sports_hall_under_nta_8800_2022` en `case_l_school_under_nta_8800_2022`.

### 9 oktober 2026 — GitHub Actions volgen de vrijgaveprocedure (geen rekenwijziging)

- **`release.yml`** start nu op tags `oes-v*` (oude `v*`-tags blijven werken). Het controleert dat de tag hoort bij `package.json` en `KERNEL_VERSION` en draait de volledige gate (`scripts/verify-nta.sh`, met Rust 1.77.2). Daarna bouwt het de pakketten voor Linux, macOS en Windows en hangt ze met `SHA256SUMS-installers.txt` aan de release. Een bestaande release houdt zijn tekst; anders komt er een concept (pre-release) dat handmatig wordt gepubliceerd.
- **`ci.yml`** draait dezelfde gate bij push en pull request naar `main` en `nta8800-kernel`, en maakt geen releases meer vanaf `main`.
- [Versiebeheer](nta8800-versiebeheer.md) en de wiki beschrijven de stappen na het pushen van een tag.

### 9 oktober 2026 — README, wiki en testprotocol gecorrigeerd na de feitencontrole (geen rekenwijziging)

Alleen documentatie (review 20).
- **Rust-versies:** 1.77.2 is de minimumversie voor de rekenkern; de desktop-app en de API/MCP-service vragen 1.88.
- **Pakketten:** een release levert alleen een `.deb`; installers voor andere platforms bouw je met `npm run tauri:build` op dat platform.
- **Webversie:** een statische `dist/`-build heeft een doorverwijzing `/api` → `http://127.0.0.1:3007` nodig.
- **Herlabelen:** de termijn van 24 maanden geldt voor de verbeteringen, gerekend vanaf de oorspronkelijke opnamedatum; het oordeel heet "te beoordelen".
- **Openbare gevallen:** zijn een regressie op de vastgelegde uitkomsten van de kern; de verschillen met de gepubliceerde waarden zijn per post verklaard. ISSO 54: 240 van de 266 deeltests.
- **Overig:** het BRL 9501-attest ontbreekt nog (README), de releasevolgorde, de controle met `sha256sum -c --ignore-missing`, het benchmarkcommando, de plek van `KERNEL_VERSION` in de app, en de publicatie van de wiki vanuit de hoofdmap.

### 9 oktober 2026 — ISSO 54-codering gecorrigeerd na review 19 (geen rekenwijziging)

De rekenkern verandert niet; alleen de codering van de ISSO 54 v2.0-deeltesten in `training-data/` (zie `docs/nta8800-isso54-voorbereiding.md`, "Correcties na review 19").
- **EP-W203l–n.** De splitsing in twee rekenzones telde een verticale leiding van 2 bouwlagen per zone en ventilatie voor twee woningen van 48 m². Nu 1 bouwlaag per zone en ventilatie voor de hele woning van 96 m²; zonder hybride rekent de splitsing als EP-W001 (nieuwe test `isso54_controls.rs`). W203l: BENG 2 105,81 → 98,48.
- **Utiliteitsreferentie EP-U001.** Nu met de fictieve verticale leiding per toiletgroep (7.17a); alle 55 utiliteitsdeeltesten stijgen ongeveer 1 %.
- **EP-W009c/d.** Ramen met b_b ≥ 3,73 zijn minimale belemmering (17.3.2 situatie a).
- **EP-W013b/d.** Een eigen installatie per woning: `identicalSystems` 4 en 10.
- **Overig.** EP-W402 met onbekend vatlabel (rekent als C), een eigen vloerconstructie bij EP-W002, en paginaverwijzingen naar de rij van de deeltest. EP-W102b en EP-W003c zijn als open interpretatie vastgelegd.

## Rekenkern 0.3.0 — 9 oktober 2026
<!-- kernel-version: 0.3.0 -->

### 10 oktober 2026 — testprotocol en testrapport per vrijgave (geen rekenwijziging)

- Nieuw [testprotocol](nta8800-testprotocol.md). Het beschrijft:
  - de testniveaus van de gate, met opdrachten, criteria, bewijs en duur;
  - de procedure voor de ISSO 54-testset;
  - 17 handmatige acceptatietests vóór elke vrijgave;
  - de regressieprocedure per soort wijziging, met de end-to-endvergelijking;
  - de codereviews 1–19.
- Nieuw sjabloon [testrapport](templates/nta8800-testrapport.md). `scripts/release-nta.sh` zet het vooringevuld (versies, tag, commit, datum, gatestatus) als `testrapport.md` in elk vrijgavearchief; het valt onder `SHA256SUMS`.

### 10 oktober 2026 — README vernieuwd en wiki-documentatie (geen rekenwijziging)

- **README:** de README beschrijft de huidige stand. Hij vermeldt:
  - de rekenkern in Rust;
  - de vijf uitgaven met hun exacte namen;
  - de versies: programma 0.2.0, rekenkern 0.3.0, handleiding gestempeld voor 0.3.0;
  - de toolchain: Rust 1.77.2 als MSRV (de service vraagt 1.88), Node.js 22 en Tauri 2;
  - de gate, versiebeheer en release;
  - de documentatiekaart, met een verwijzing naar het testprotocol.
- **Wiki:** nieuwe map `wiki/` met Nederlandstalige wikipagina's per onderwerp, klaar om naar de GitHub-wiki te publiceren (zie `wiki/README-publiceren.md`). De pagina's vatten samen en verwijzen naar de handleiding en de documenten in de repository.

### 10 oktober 2026 — EP-Online-gegevensoverzicht, dossiercontrole en A0-tabel na de achttiende review

- **`Op_basis_van_referentiegebouw` stond omgekeerd.** Het veld is nu `true` voor de gelijkende woning, die een referentie-energieprestatierapport krijgt, en `false` voor de bezochte referentiewoning (BRL 9500-W §4.3 p. 26; RVO-handleiding EP-online bijlage 2).
- **Woning of niet volgt de berekening.** Het overzicht neemt de rekenomvang van de kern (`calculationScope`); een logiesfunctie die als woning is berekend, krijgt Gebouwklasse W met TOjuli, warmtebehoefte en de EMG-velden (Omgevingsregeling art. 5.11 lid 1, 5.13 onder a en d). Zonder kernuitvoer beslist de gebruiksfunctie.
- **Veldlengtes uit het exportschema.** Huisnummertoevoeging (4), huisletter (1), Gebouwtype en Gebouwsubtype (75), Detailaanduiding (25), SBI-code (20) en Status (256). Een te lange waarde komt niet meer in de XML maar staat met reden bij de weggelaten velden. Een huisnummer dat niet te ontleden is, verdwijnt niet meer stil maar wordt gemeld.
- **Dossiercontrole (BRL 9500 Bijlage 3).**
  - De registratiegegevens vragen ook het registratienummer van de registrerende adviseur (W p. 62, U p. 54).
  - Kwaliteits- en gelijkwaardigheidsverklaringen gelden bij elk doel zodra de invoer een verklaarde productwaarde gebruikt (kwaliteitsverklaring warmtepomp, gedeclareerde f_ctrl, A/B/C van de ketel, ε van de WKK, opgegeven opwekkerwaarden), met de verklaring en een foto of factuur (W p. 63, §4.2.7 p. 25; U p. 21).
  - "Het geleverde energieprestatie-rapport" is ontbrekend als het dossier geen rapport bevat (W p. 61, U p. 53).
  - Een utiliteitsdossier vraagt ook het type bevochtiging en/of ontvochtiging en het type verlichting (U p. 53).
- **A0-tabel in het rapport.** De rij heet nu "b. Primair fossiel energiegebruik ≤ … (bijlage IXa/Xa)" in plaats van "BENG 2 ≤", en de voorwaarden a (energiebehoefte) en c (aandeel hernieuwbare energie) staan er ook in.
- Rekenuitkomsten van de kern veranderen niet; de waarden in het EP-Online-overzicht en de dossierstatus wel.

### 10 oktober 2026 — labelelement h, A0 bij woningen en het woningtype op het label

- **Labelelement h (Omgevingsregeling art. 5.13a lid 1 onder h).** De "productie van hernieuwbare energie" telde de elektriciteit van een (micro-)WKK mee. Die is wel eigen opwekking, maar niet hernieuwbaar (5.14b). De elektriciteitsbalans heeft nu per maand `producedRenewableKwh`, en element h is de som daarvan. `producedKwh` blijft de totale eigen opwekking voor de balans van 5.10/5.13. Zonnewarmte telt niet mee in h; dat is een lezing, zie [vragen aan NEN](nta8800-vragen-nen.md) §4.
- **A0 bij een woning (art. 5.11 lid 4 en lid 5 onder b).** Voorwaarde b toetste BENG 2. Ze toetst nu de labelwaarde, die bij een woning met energiemaatregelen op gebiedsniveau met forfaitaire waarden (EMGforf) wordt berekend. Een woning met een warmtenet met kwaliteitsverklaring en BENG 2 ≤ 27 maar EMGforf > 27 krijgt geen A0 meer. Utiliteit blijft ongewijzigd: daar is de labelwaarde de waarde met kwaliteitsverklaring. Ook dit is een lezing (§4).
- **Woningtype op het label (art. 5.11 lid 1 en 5.13 onder a).** Of het label een woningtype draagt, volgt nu de rekensoort van de kern (woning of utiliteit) en niet de gebouwfunctie van het project. Een logiesfunctie niet in een logiesgebouw, gerekend als woning, krijgt zo haar woningtype. Zonder kerninvoer beslist de gebouwfunctie.
- **Effect op de cases.** De openbare gevallen A–J, de RVO-woningen en de ISSO 54-cases rekenen ongewijzigd. Geen van die cases heeft een micro-WKK met A0-toets, een EMG-woning met A0-toets of een logiesfunctie als woning.

### 10 oktober 2026 — oudere uitgave ook in het geneste prestatieblok; klimaatbron per uitgave (geen rekenwijziging)

- **Status van het geneste blok.** Bij een berekening in een oudere uitgave kreeg alleen de bovenste status `calculated_legacy_edition`. Het geneste `performance`-blok van de project- en opnameroute bleef `calculated_unverified`. Nu krijgt ook dat blok de status van de oudere uitgave, en `registrationEligible: false` als het die heeft. Wie alleen het geneste blok leest, ziet zo geen registreerbaar resultaat meer. Het rapport, het rapportmodel en het EP-Online-overzicht behandelen het resultaat zoals eerder: berekend in een oudere uitgave, niet voor registratie.
- **Klimaatbron.** `climateSource` noemde altijd NTA 8800:2025+C1:2026, ook bij een uitkomst in 2022 of 2020+A1. Nu noemt hij de tabellen 17.1/17.2 van de uitgave waarin is gerekend (2025+C1 p. 690, 2024 p. 674, 2023 p. 676, 2022 p. 666, 2020+A1 p. 662).
- **Gecontroleerd:** tabel 17.2 is in alle vijf de uitgaven gelijk. Tabel 17.1 verschilt alleen in θ_e;argII voor mei tot en met september (2023, 2022 en 2020+A1 tegenover 2024 en later). Die waarden wisselde de kern al per uitgave (`argii_temperature_c` in het profiel), dus de uitkomsten veranderen niet. 2020+A1 heeft geen kolom ϑ_ODA;preh;WTWC; koudeterugwinning weigert de kern daar al (omslagpunt 79).

### 10 oktober 2026 — import en export van UNIEC3- en VABI-bestanden na de zestiende review

Ingelezen VABI-projecten rekenen anders dan voorheen: de koudebruggen, U-waarden en hellingshoeken veranderen. Een eerder ingelezen project verandert pas als je het bestand opnieuw inleest.
- **Koudebruggen.** De VABI-export schreef de koudebruggen van een zone onder elk vlak; de import telde ze dan per vlak op, zodat ΣψL na een rondgang met het aantal vlakken vermenigvuldigde. Nu staan ze één keer per rekenzone. Een ouder bestand met dezelfde lijst onder elk vlak wordt één keer ingelezen, met een punt in het importverslag. Een ontbrekende ψ wordt een punt.
- **U-waarde.** Een constructie zonder R_c (alleen een U ingevoerd) kwam na een VABI-rondgang terug met U = 0. Nu schrijft de export `UWaarde`, leest de import die eerst, dan de bibliotheek met dezelfde naam, en pas daarna R_c met de R_si van het vlaktype. Een U van 0 wordt nooit overgenomen, ook niet bij UNIEC3: het vlak blijft dan zonder constructie, met een punt.
- **Getallen.** `1.234,5` is 1234,5 (was 1,2345); tekst, NaN en waarden buiten het bereik worden niet meer stil 0 of NaN maar een punt met de standaardwaarde of de grens. Elk ingelezen project gaat door dezelfde controle als een geopend projectbestand. Een q_v10 van 0 of zonder waarde krijgt de forfaitaire waarde met het kenmerk *aangenomen*.
- **Codes.** Codes als `constructor` of `__proto__` worden niet meer als functie, vlaktype of oriëntatie overgenomen.
- **UNIEC3-export** schrijft getallen met volle precisie in plaats van op twee decimalen (q_v10 0,987 werd 0,99).
- **Hellingshoek.** VABI `Hellingshoek` 0 en 180 zijn horizontaal (een plat dak werd een noordgevel); een andere hoek van een dak komt in de NTA-invoer (`surfaceTilts`). De export schrijft de hellingshoek.
- **Deelvlakken.** Een VABI-vlak met meer dichte deelvlakken wordt gesplitst, elk met zijn eigen constructie uit de bibliotheek met dezelfde naam; eerder kreeg het hele vlak de waarde van het eerste deelvlak en verdubbelde de constructielijst.
- **Meer gebouwen en dubbele ids.** Meer dan één gebouw, GEB of UNIT, dubbele ids en verwijzingen naar onbekende elementen worden punten. In een EPA-archief met meer XML-bestanden wordt `project.xml` gelezen.
- **Tekencodering.** VABI-bestanden in UTF-16 of windows-1252, en met BOM, worden goed gelezen; een bestand met `DOCTYPE` wordt geweigerd. Ongeldige JSON of een verkeerde opbouw van een UNIEC3-bestand geeft een duidelijke melding in het Nederlands of Engels.
- **Importverslag.** Elke import legt bestandsnaam, SHA-256 en de punten om na te lopen vast in het importlogboek; de app toont ze na het inlezen en in Projectinformatie. Het logboek blijft buiten de vingerafdruk van de labelinvoer, gelijk aan de rekenkern.
- **Verlies bij export.** De UNIEC3- en VABI-export melden welke projectgegevens (zoals de NTA-invoer) niet in het bestand komen; bij opnieuw inlezen staat dat als punt in het verslag.

### 10 oktober 2026 — dossier, bewijsstukken en webview afgeschermd (geen rekenwijziging)

Vijftiende review (desktoplaag en bestanden).
- **Content Security Policy.** De desktopapp had geen CSP (`"csp": null`). Nu staat er een strikte: alleen eigen scripts (`script-src 'self'`), geen plug-ins, en netwerk alleen naar de app zelf, de IPC, de lokale API en de feedbackdienst. Lettertypen van Google Fonts en `blob:`-afbeeldingen (miniaturen) blijven toegestaan. In ontwikkeling geldt geen CSP (`devCsp: null`), zodat Vite blijft werken.
- **Rapportvoorbeelden zonder scripts.** De iframes van de rapportbouwer en het afdrukvoorbeeld, en het verborgen afdrukframe, hebben nu een `sandbox` zonder `allow-scripts`. Rapport-HTML wordt al ontsnapt; dit voorkomt dat één gemiste ontsnapping toegang tot de IPC zou geven.
- **Dossier niet opgeslagen is een fout.** Mislukt het wegschrijven van het projectdossier in de desktopapp (schijf vol, geen schrijfrechten), dan toont de app de fout. Eerder viel ze terug op een browserdownload die in de webview niets doet, waarna de export als gelukt gold.
- **Grootte van het dossier.** Een dossier boven 2 GB wordt geweigerd met een melding (de ZIP-bibliotheek schrijft geen ZIP64; boven 4 GB zou de ZIP onbruikbaar zijn). Foto's, pdf's en andere al gecomprimeerde bestanden worden niet opnieuw gecomprimeerd.
- **Bewijsstukken.** Per bestand maximaal 200 MB, en alleen pdf, afbeeldingen, tekeningen (dwg, dxf, ifc), kantoordocumenten en e-mails; anders een melding, zonder het bestand te lezen. De hash rekent zonder extra kopie van het bestand. De desktopapp houdt een bestand na het wegschrijven in de app-map niet meer ook in het geheugen, en verwijderen uit het register geeft het geheugen vrij.
- **Gelijktijdig toevoegen.** Bestanden die in twee blokken tegelijk worden toegevoegd, krijgen elk een eigen id; het toevoegen gebeurt op het register zoals het op dat moment is (`ADD_EVIDENCE`), zodat tussentijdse wijzigingen in de registratie niet verloren gaan.
- **Dubbele ids en namen.** Een dubbel id in het register krijgt bij het openen een nieuw id, met een notitie in de beschrijving. Twee bewijsstukken met dezelfde naam in de ZIP krijgen een volgnummer, zodat de SHA-256 in het manifest overeenkomt met het gearchiveerde bestand. Namen in de ZIP houden hun extensie bij inkorten en verliezen afsluitende punten en spaties.
- **Import van UNIEC3 en VABI.** Alleen de benodigde bestanden uit het archief worden uitgepakt, met een grens op de opgegeven grootte (64 MB per bestand, 256 MB samen), zodat een zip-bom het geheugen niet kan vullen.

### 10 oktober 2026 — opslaan, afsluiten en openen zonder verlies van werk (geen rekenwijziging)

- **Afsluiten vraagt eerst.** Het venster sluiten (de sluitknop, Alt+F4, afsluiten vanuit het besturingssysteem) gooide alle niet-opgeslagen tabbladen zonder vraag weg. Nu stelt de app per project met niet-opgeslagen wijzigingen of niet-toegepaste NTA-invoer dezelfde vraag als bij een tabblad (Opslaan · Niet opslaan · Annuleren); bij Annuleren of een mislukte opslag blijft het venster open. In de browser vraagt de browser of je de pagina wilt verlaten.
- **Mislukt opslaan is geen opslag.** Lukte het schrijven niet (schijf vol, map alleen-lezen), dan viel de desktopapp terug op een browserdownload die daar niets doet en zette het project op opgeslagen; bij sluiten ging het tabblad dicht. Nu meldt de app **Opslaan mislukt** met de reden, blijft het project gewijzigd en blijft het tabblad open.
- **Veilig opslaan.** De desktopapp schrijft via de nieuwe opdracht `save_project` eerst een tijdelijk bestand naast het project, maakt dat vast op schijf en zet het dan op de plaats van het oude; de vorige versie blijft als `.bak`. Een stroomstoring of volle schijf tijdens het opslaan laat het oude bestand heel. De opdracht schrijft alleen `.json`-bestanden die in een venster zijn gekozen of op de lijst met recente projecten staan.
- **Recente projecten na een herstart.** Het bestandsvenster geeft alleen voor de lopende sessie toegang tot een gekozen bestand, dus na een herstart mislukte "recent openen" en verdween het project van de lijst. De desktopapp houdt nu zelf een lijst bij van bestanden die via een venster zijn geopend of opgeslagen (`read_project_file`) en leest die ook na een herstart. Een project verdwijnt alleen van de lijst als het bestand er niet meer is.
- **Bestandsversie.** Een bestand met een nieuwere hoofdversie (2.x en hoger) werd zonder melding geopend en bij opslaan teruggeschreven als 1.0. Nu weigert de app het met een duidelijke melding. Een nieuwer 1.x-bestand opent wel, met een melding welke velden bij opslaan wegvallen; een bestand zonder versie geldt als 1.0.
- **Stempelvergelijking bij openen.** De vingerafdruk wordt nu vergeleken met het project zoals het in het bestand staat, vóór de standaardwaarden van het openen; een ouder bestand met stempel meldt dan geen onterechte "invoer gewijzigd".
- **Rekenen buiten de hoofdthread.** De rekenopdrachten van de desktopapp lopen nu op een achtergrondthread, zodat het venster niet bevriest tijdens een berekening, en een paniek in de rekenkern wordt een foutmelding `kernel_panic` in plaats van het afsluiten van de app met alle open projecten (de release gebruikt nu `panic = "unwind"`). De normversie blijft per berekening op dezelfde thread gezet en hersteld.

### 9 oktober 2026 — basisopname rekent in NTA 8800:2022 en 2020+A1

- **Hoogte H2 bij een kruipruimte.** NTA 8800:2022 en 2020+A1 rekenen in (8.47) met de werkelijke hoogte h van de kruipruimtewand boven het maaiveld (2022 p. 236); vanaf 2023 is h vast 0,125 m (p. 240). De basisopname vulde h nooit in, dus een woning- of utiliteitsopname met een vloer boven een kruipruimte of onverwarmde kelder werd in die uitgaven geweigerd met `ground_floor_wall_height_required`, op een veld dat de opname niet kon zetten. De opname vraagt nu H2, de hoogte van de bovenkant van de begane grondvloer tot het maaiveld (ISSO 82.1 4e druk afb. 8.23, p. 73; ISSO 75.1 4e druk dezelfde paragraaf), als `floorHeightAboveGroundM` op die vloer. In 2022 en 2020+A1 gaat de waarde door als h; zonder antwoord meldt de opname `floor_height_above_ground_required` op het opnameveld. Een forfaitaire waarde voor een onbekende hoogte geven de bronnen niet. Vanaf 2023 wordt H2 niet gebruikt en verandert er niets. Het veld staat in de opname alleen bij die twee uitgaven.
- **f_BACS in de utiliteitsopname.** De opname gaf f_BACS 1,05 door in uitgaven zonder 5.5.8 (2024 en ouder), waar de kern dat weigert (`route_not_in_edition` op `bacsFactor`). De opname houdt daar nu 1,0 aan, met de regel `bacs_factor_not_in_edition`.
- **Test.** `crates/nta8800-core/tests/survey_editions.rs` rekent de zes opnamevoorbeelden in alle vijf uitgaven: met H2 rekenen ze overal, zonder H2 alleen in 2022 en 2020+A1 een melding op het opnameveld; vanaf 2023 verandert H2 niets.

### 9 oktober 2026 — HTTP-API en MCP-server na de veertiende review (geen rekenwijziging)

Geen uitkomst van een berekening verandert; wel het gedrag van de service bij grote, verkeerde of onbekende verzoeken.

- **Geheugen per verzoek begrensd.** Een verzoek van 16 MiB met alleen nullen kostte ongeveer 1 GB, omdat de body twee keer werd gekopieerd en elke JSON-waarde tientallen bytes kost; acht tegelijk liepen op tot 8,6 GB. Nu kopieert geen route de body meer (het invoerlid gaat in zijn geheel naar de kern), en de service telt de JSON-waarden voordat ze de body inleest: boven `--max-json-values` (standaard 500 000) volgt 413 `payload_too_large`. Een ingelezen body kost zo hoogstens ongeveer 70 MB; de documentatie geeft de gemeten bytes per waarde en de grens voor alle toegelaten verzoeken samen.
- **Referentiegevallen in hun eigen uitgave.** `reference/compare` en `reference/audit` stempelden de uitkomst altijd met 2025+C1, ook bij een geval in 2024. Nu krijgt de uitkomst de uitgave van `case.normVersion`; een afwijkende `normVersion` in het verzoek geeft `norm_version_conflict`.
- **Herlabelen en label-hash** weigeren iets anders dan een projectobject (400 `invalid_project_shape` met het pad), in plaats van `{"original":1,"current":2}` als toegestaan te beoordelen.
- **De verouderde route `/v1/nta8800/calculate`** geeft bij een ongeldig project de beoordeling zelf als 422-body, zoals de andere routes; de code `invalid_project_input` vervalt.
- **Onbekende leden in een verzoek** worden geweigerd (400 `invalid_request_shape`), zodat een verschreven `normversion` niet ongemerkt in 2025+C1 rekent. De schema's in het OpenAPI-document hebben `additionalProperties: false`.
- **MCP-server antwoordt altijd.** Ongeldige of afgebroken JSON, JSON dieper dan 128 niveaus, een regel langer dan 64 MiB (`OES_MCP_MAX_LINE_BYTES`) en `tools/call` met verkeerde `arguments` bleven onbeantwoord; nu volgt een JSON-RPC-fout (−32700, −32600 of −32602) en antwoordt de server daarna gewoon verder.
- **`capabilities`** noemt de huidige reikwijdte: de projectberekening met meer rekenzones, de basisopnames, maatwerkadvies, herlabelen en registratie, `projectCalculationAvailable` en de uitgaven (`supportedNormVersions`). `calculationAvailable` gaat alleen over de verouderde route.
- **Hostcontrole.** Op een loopback-adres beantwoordt de API alleen `localhost`, `127.0.0.1` en `::1` (plus `--allowed-host`); een andere `Host` krijgt 403 `host_not_allowed`. Dit houdt een webpagina met DNS rebinding buiten.
- **Kleiner:** elk CORS-antwoord heeft `Vary: Origin`, ook een geweigerde preflight; `energy-by-service` geeft dezelfde identiteitsvelden als de andere projectdelen (`registrationEligible`, `attestStatus`, `warnings`, `normVersion`); het OpenAPI-document noemt 422 alleen waar die kan voorkomen, heeft een relatieve `servers`-regel en noemt 403; `validate` antwoordt bewust altijd 200 (de app leest de beoordeling ook bij `invalid`). Nieuwe meldcodes met Nederlandse en Engelse tekst: `invalid_json`, `payload_too_large`, `unsupported_media_type`, `host_not_allowed`.

### 9 oktober 2026 — basisopname en maatwerkadvies na de dertiende review

- **Renovatie met onbekende isolatiedikte (ISSO §8.7.2.1).** "De klasse ervóór" en "de klasse erna" tellen nu op de klassen van ISSO-tabel 8.10 (82.1 p. 90, 75.1 p. 89). Bij gewone gevels is dat één klasse "van 1975 tot 1988"; de kern telde op de splitsing bij 1983 uit NTA-tabel I.5. Gevolgen: gebouwd 1970 en gerenoveerd 1985 zonder bewijs geeft R_c 0,43 (was 1,3); gebouwd 1978 met onbekend renovatiejaar geeft 2,0 (was 1,3). Vloeren en daken hebben in beide tabellen de splitsing bij 1983 en veranderen niet.
- **Romp van een woonboot uit 2014.** De basisopname volgt ISSO-tabel 8.10 (82.1 p. 89): R_c 3,5 ("van 2014 tot 2018"). De rechtstreekse forfaitaire route blijft NTA-tabel I.7 (p. 831) volgen: 2,5.
- **Woningpaspoort, stap 1.** Zonder verklaring van de adviseur toetst de kern de warmtebehoefte van stap 1 nu aan de Standaard voor Woningisolatie "na 1945" (ISSO 82.2 §1.10.2 p. 22, §4.4.2 p. 75). De vooroorlogse standaard geldt alleen met `prewarStandard` en een motivering. Een opgegeven grens wordt vergeleken met de warmtebehoefte E_H;nd, niet met E_H+C;nd (NTA 8800 §5.3.2 Opmerking 2, p. 75). Hoofdstuk 5 geeft daarvoor ook `standardInsulationPost1945KwhPerM2`.
- **Gebruikersprofielen utiliteit (ISSO 75.2 tabellen 2.3 p. 42 en 2.4 p. 43).** Elke rekenzone krijgt het profiel ten opzichte van haar eigen NTA-waarden (tabellen 7.13–7.15). Eerder kregen alle zones de setpoints en uren van de hoofdzone.
- **Afwijking van het elektriciteitsgebruik.** De gerapporteerde afwijking (jaar en per maand) en het berekende maandgemiddelde gebruiken nu de netto levering (levering min teruglevering), zoals de passingstoets en de meting (ISSO 82.2 Bijlage C.1 p. 105, C.3 p. 107). Bij teruglevering liet het rapport eerder een te grote afwijking zien.
- **NCW-horizon.** Zonder opgegeven horizon gebruiken alle varianten en pakketten één gemeenschappelijke horizon: de langste levensduur van alle maatregelen. Zo vergelijkt de automatische keuze op de hoogste NCW pakketten over dezelfde periode.
- **Bronnen.** De verwijzingen naar BRL 9500-MWA-W/U van 24-03-2026 zijn niet gecontroleerd; die tekst ontbreekt hier (gereedheid E7).

### 9 oktober 2026 — herlabelen na de dertiende review

Het oordeel bij herlabelen verandert in vier gevallen (BRL 9500-W bijlage 6a/6b, p. 67–68; U p. 58–60; ISSO 82.1 en 75.1 bijlage A):

- **Energielabel van een vat.** Een ander label van een voorraadvat (`hotWater.storage[].loss.label`) of buffervat (`distributionSystem.bufferVessel.label`) viel weg uit de vergelijking, terwijl de hash wel veranderde. Het telt nu als één-op-één wijziging (6a). Alleen een `label` die weergavetekst is, blijft buiten de vergelijking. Ook de manier waarop het stilstandsverlies is bepaald (label of meting) is een eigenschap van het vat en geen systeemwijziging meer.
- **Gastoestel door elektrische boiler.** Een gasgestookt warmwatertoestel (`gas_appliance`, `gas_storage_heater`) vervangen door een elektrische boiler is volgens ISSO 82.1 bijlage A, cluster 1 (p. 209) en ISSO 75.1 (p. 215) een één-op-één vervanging. Het was "niet toegestaan"; nu is het toegestaan (6a). Andere wissels, zoals een gastoestel door een warmtepomp, blijven een systeemwijziging (6b).
- **Luchtdichtheid.** Een andere q<sub>v10</sub> of infiltratie gold als toegestane installatiewijziging. Bijlage 6a en 6b noemen luchtdichtheid niet; het is nu "te beoordelen", met het meetrapport als bewijs.
- **Koeling.** Koeling aanbrengen of verwijderen was "te beoordelen". Het is een systeemwijziging ("het aanbrengen van koeling", ISSO 82.1 bijlage A, cluster 6, p. 210; 6b W p. 68) en is nu niet toegestaan.

De indeling herkent de eigenschappen van ramen (`windowSolar`) niet meer als zonne-energie. De clusters "uitgave van NTA 8800 gewijzigd" en die van deze wijziging hebben nu een Nederlandse en Engelse tekst; een test leest alle clusters en notities uit de kern en eist een tekst.

### 9 oktober 2026 — getalvelden na de twaalfde review (geen rekenwijziging)

- **Geen ander getal dan getypt.** Een getalveld overschreef de getypte tekst als het formulier een tussenstand weigerde (de "0" van "0,04" bij een λ die groter dan nul moet zijn). Daarna las het "0.8,04" als 8,04 en sloeg dat op. Zolang het veld focus heeft, blijft de getypte tekst nu staan; een geweigerde stand is gemarkeerd en bij het verlaten toont het veld weer de opgeslagen waarde. Gemengde scheidingstekens tellen alleen als getal als het ene teken per drie cijfers groepeert ("1.234,5"); "0.8,04" is geen getal. Een overloop zoals "1e999" ook niet.
- **Pijltjestoetsen binnen de grenzen.** ↑/↓ houden zich aan `min` en `max` van het veld, zoals een gewoon getalveld (hellingshoek niet onder 0, dekking niet boven 100 %).
- **Perzisch, Arabisch en volle breedte.** Getallen staan in elke taal in Latijnse cijfers, zodat het veld bewerkbaar blijft; Perzische, Arabisch-Indische en volle-breedtecijfers en -scheidingstekens en richtingstekens worden bij het typen gelezen.
- **Opmaak.** De monospace-letter geldt alleen voor de nieuwe getalvelden, niet meer voor elk getalveld van de NTA-formulieren.
- **Hulptechnologie.** Een getalveld meldt zijn waarde en grenzen (`aria-valuenow`, `aria-valuemin`, `aria-valuemax`).
- **Vertaalsleutels.** De controle op ongebruikte sleutels liet een hele naamruimte door zodra een los woord zoals `'overview'` in de code stond. Een letterlijke tekst telt nu alleen als voorvoegsel als hij een punt bevat. Daarmee zijn 15 ongebruikte sleutels gevonden en verwijderd, onder meer `envelope.windowsOf`, `installations.blockCount` en `overview.issueCount`.
- Toetsen: `src/__tests__/decimal-input-typing.test.tsx`.

### 9 oktober 2026 — decimale komma in alle getalvelden, ongebruikte vertaalsleutels (geen rekenwijziging)

- **Decimale komma overal.** De 81 resterende kale getalvelden (`type="number"`) buiten de NTA-formulieren accepteren nu een komma: de oudere dialogen (zone, schildeel, raam, constructie, koudebruggen, installaties, luchtdichtheid), het maatwerkadvies, de rekenhulpen en de diagnosepanelen voor warmtepompen. Een kaal getalveld weigerde "0,35" zonder melding. Ze gebruiken het nieuwe `DecimalInput`. Dat neemt de waarde en de afhandeling van het veld ongewijzigd over (leeg, minimum, afwijzen van een negatief getal) en geeft de waarde met een punt door. Tekst die geen getal is, wordt gemarkeerd en niet doorgegeven.
- **Geen nieuwe kale getalvelden.** `src/__tests__/decimal-input.test.tsx` faalt als `type="number"` weer in `src/components` verschijnt. Hij toetst ook de komma, het lege veld en het terugzetten van een geweigerde waarde, en de invoer met een komma in de zonedialoog, de constructiedialoog, de verwarmingsdialoog en het maatwerkadvies.
- **Vertaalsleutels.** 14 sleutels die nergens meer werden gebruikt zijn uit alle talen verwijderd, onder meer `theme.blue` en oude teksten van de projectverkenner en de bewijskoppeling. `src/__tests__/i18n-orphan-keys.test.ts` faalt bij een sleutel die de code niet gebruikt. Sleutels die de code opbouwt, zoals `kernel.issue.${code}` of `options(t, 'nta.annexP.carrier', …)`, tellen als gebruikt.

### 9 oktober 2026 — toegankelijkheid en taal van de formulieren (geen rekenwijziging)

- **Lijstonderdelen als benoemde groep.** Elk vlak, raam, deur, opwekker, vat, lichtzone, functie, verticale leiding, randisolatie, armaturengroep en elk deel van een E.1-systeem is een groep met een eigen naam. De groep heet naar het onderdeel (bijvoorbeeld "gevel-2" of "Opwekker 1"), zodat een schermlezer de velden van verschillende onderdelen uit elkaar kan houden.
- **Eenheden.** Een eenheid in een invoerveld wordt voorgelezen na een scheiding ("Lampvermogen, W"), niet vastgeplakt ("LampvermogenW").
- **Bevindingen op het veld.** Een veld met een fout of aandachtspunt van de rekenkern krijgt naast de rode of gele rand een zichtbaar teken (⚠). Het invoerelement krijgt `aria-invalid` (bij een fout) en de melding als `aria-description`, zodat een schermlezer haar voorleest. Fouten en aandachtspunten in de werkstroom verschillen ook in teken, niet alleen in kleur.
- **Decimale komma.** De maandvelden (dynamische ramen, maandwaarden), de terugwinbare tapwaterwarmte, de constante ventilatie en de getallen in Registratie (bouwjaar, WLC-GWP, gebruiksoppervlakte) gebruiken nu het getalveld van de app: een komma of een punt werkt overal.
- **Labels.** Dubbele namen binnen één groep zijn opgeheven: de bron van de leidinglengtes van tapwater, de opstelling en de toestelbron van de WTW, en in het Engels "System arrangement" naast "Heat recovery layout". Engelse teksten met Nederlandse normtermen ("afb.", "Bijlage 6a") zijn vertaald ("fig.", "BRL 9500 annex 6a").
- **Test.** `src/__tests__/nta-form-accessibility.test.tsx` rendert elke NTA-sectie op het voorbeeldkantoor en de tussenwoning en beide opnames. Hij eist een naam voor elk invoerelement en elke knop, een legenda voor elke fieldset en geen dubbele naam binnen één groep. Verder toetst hij de eenheid in de naam, de decimale komma, de aankondiging van bevindingen en dat er geen Nederlandse normtermen in Engelse teksten staan (en omgekeerd).

### 9 oktober 2026 — tapwaterwarmtepomp: opgegeven klasse zonder kwaliteitsverklaring

- **Klasse alleen bij een opgegeven rendement.** Een forfaitaire tapwaterwarmtepomp met een toepassingsklasse maar zonder kwaliteitsverklaring kreeg nog c_W;gen van tabel 13.27, en bij klasse 1 boven 1 805 kWh de fout `hot_water_heat_pump_class_exceeded`. Een forfaitaire waarde heeft geen gemeten klasse (tabel 13.25 en 13.8.4.7.3 c); 2025+C1 p. 641/643, 2020+A1 p. 612/615, 2022 p. 617/620, 2023 p. 627/630, 2024 p. 625/627). De kern corrigeert nu alleen een opgegeven rendement met de klasse waarbij het is gemeten. De klasse blijft gelden voor de douche (13.30).
- **Resultaten die veranderen.** Projecten met een forfaitaire tapwaterwarmtepomp, zonder kwaliteitsverklaring en met een opgegeven klasse, rekenen nu met 1,4 (of 1,4 · c_source). Een project dat met klasse 1 op `hot_water_heat_pump_class_exceeded` stuitte, rekent nu.

### 9 oktober 2026 — tapwaterwarmtepomp forfaitair zonder c_W;gen; openbare gevallen I en J

- **Tapwaterwarmtepomp forfaitair.** De kern vermenigvuldigde het forfaitaire opwekkingsrendement van een tapwaterwarmtepomp (1,4, of 1,4 · c_source) met c_W;gen van tabel 13.27 voor klasse 4: bij een kleine bruto tapwatervraag 0,45, dus 0,63 in plaats van 1,4. Tabel 13.25 geeft de warmtepompen zonder c_W;gen; alleen de gasrijen dragen c_W;gen (p. 641; 2020+A1 p. 612). 13.8.4.7.3 c) neemt c_W;gen uit de klasse waarbij het rendement van de warmtepomp is gemeten (p. 643). De kern corrigeert nu alleen bij een opgegeven gemeten klasse (`measuredClass`). Test `forfait_heat_pump_takes_no_class_correction`.
- **Resultaten die veranderen.** Projecten met een forfaitaire tapwaterwarmtepomp zonder gemeten klasse en een bruto tapwatervraag onder 3 890 kWh rekenen met minder tapwaterelektriciteit, dus een lagere BENG 2 en een hoger aandeel hernieuwbaar. Een productverklaring (`declared`) en de boosterwarmtepomp (13.162/13.163) veranderen niet. De openbare gevallen A–H en de RVO-voorbeeldwoningen veranderen niet.
- **Openbare gevallen I en J.** Twee utiliteitsgevallen uit de periode van 2020+A1, nagebouwd uit openbare Uniec-uitdraaien: I een kantoor met bijeenkomstfunctie (439,04 m²), J een winkelruimte onder appartementen (89,80 m²). De eisen van de kern zijn gelijk aan die van de rapporten. Fixtures `training-data/nta8800-public-comparison-{i,j}.json`, tests in `crates/nta8800-core/tests/public_comparison.rs`, suitegevallen `openbaar-i-2020a1` en `openbaar-j-2020a1`. De verschillen per post staan in `docs/nta8800-vergelijking-openbare-rapporten.md`: bij I vooral de ontvochtiging bij directe expansie (12.5) en 10.15, bij J 10.15 en 10.87.

### 9 oktober 2026 — openbare rapporten onder 2025+C1 gezocht (geen rekenwijziging)

- Opnieuw gezocht naar openbare BENG-rapporten uit de periode van NTA 8800:2025+C1 (Uniec 3.4, rekendatum vanaf 29 mei 2026), voor woningen en utiliteitsgebouwen. Er is er nog geen gevonden; de zoekopdrachten en de gevonden, niet bruikbare rapporten staan in `docs/nta8800-vergelijking-openbare-rapporten.md`.

### 9 oktober 2026 — bewijskoppelingen op id bij de registratiecontrole (tiende review)

- **Bewijskoppeling.** De app schrijft een koppeling naar een bouwdeel op id (`/zones/@z1/surfaces/@gevel-n`, sinds 7 oktober), maar de registratiecontrole van de kern kon die vorm niet lezen. Elk zo gekoppeld bewijsstuk gaf `evidence_link_path_unknown`, en het project werd daardoor nooit klaar voor registratie (`readyForRegistration` onwaar). De kern lost `@id`-delen nu op zoals de app: het ene element met die id, anders onbekend (ontbrekend of dubbel). De oude vorm met posities blijft werken.
- **Resultaten die veranderen.** Alleen de registratiestatus: een project met zulke koppelingen kan nu wel klaar voor registratie zijn. Rekenuitkomsten veranderen niet.
- **Bewaking.** Een gedeelde lijst (`training-data/nta8800-evidence-pointer-cases.json`) wordt door de app en de kern opgelost; beide moeten hetzelfde geven. Een eind-tot-eindtest koppelt in de app een foto, bewaart en opent het project en laat de kern het verzoek controleren.

### 9 oktober 2026 — EP-Online-overzicht en labelgegevens in het rapport (geen rekenwijziging)

- **Oudere uitgave in het EP-Online-overzicht.** Bij een berekening in een oudere uitgave van NTA 8800 vulde het gegevensoverzicht toch de energieklasse en de indicatoren in, zonder vermelding van de uitgave. Die blijven nu leeg met de reden "oudere uitgave (NTA 8800:…), niet registreerbaar", en de toelichting noemt de uitgave. Het rapport zei dit al.
- **Labelgegevens in het rapport (Omgevingsregeling art. 5.13a).** Twee elementen verschenen met hun Engelse sleutel (`renewableProductionKwh`, `wlcGwpKgCo2EqPerM2`); ze hebben nu hun Nederlandse naam. Energiedrager en hernieuwbare bron staan in het Nederlands (aardgas, elektriciteit, …), jaarhoeveelheden in hele kWh. Een test leest de velden en codes uit de kern, zodat een nieuw element niet ongenoemd blijft.
- **Documentatie.** Achterhaalde tellingen en standen bijgewerkt in de BRL 9501-gereedheid, de programmabeschrijving en de handleiding (240 van 266 ISSO 54-deeltesten, openbare gevallen A–H, hoofdstukken 00–10).

### 9 oktober 2026 — een leeg veld met een standaardwaarde houdt die waarde (negende review)

- **Null is ontbrekend.** Sinds de vorige wijziging kreeg de projectroute de lege velden als null. Een veld met een standaardwaarde in de kern (bijvoorbeeld de bevestiging van PV, het aantal zones langs een verticale leiding, het aantal ventilatorconvectoren, het buitengebouwaandeel van de ventilatoren, het aantal zonnecollectoren) gaf dan `nta_value_missing` en status `incomplete`, waar het weggelaten veld gewoon rekende. Dat gold voor 76 velden die de formulieren kunnen leegmaken. De kern leest een null-lid in het NTA-blok nu als ontbrekend, zoals de invoervingerafdruk: het veld houdt zijn standaardwaarde. `nta_value_missing` volgt alleen nog voor een veld dat de kern echt nodig heeft. De enige uitzondering is `nominalThermalCapacityKw` van een GBS-opwekker, waar null bewust "niet te bepalen" betekent; die null telt ook in de vingerafdruk mee. De ruwe controles op `ntaHeatPump` en `layers` behandelen null nu ook als ontbrekend.
- **Formulieren.** De velden met een standaardwaarde laten bij leegmaken het lid weg in plaats van null te schrijven, en de bevestiging van PV heeft geen lege keuze meer.
- **Verticale leidingen.** Een leiding houdt alleen de velden van haar route (7.17a): de bouwlagen voor een woonfunctie, de gebouwhoogte H en het oppervlakteaandeel voor een andere gebruiksfunctie. De route volgt, net als in de kern, de gebruiksfunctie van de zone (een eigen gebruiksfunctie in de zonegegevens gaat voor). Een achtergebleven veld van de andere route verdwijnt bij Toepassen, en een ontbrekend veld van de eigen route wordt leeg gezet, zodat de melding op een getoond veld staat.
- **Vangnet.** De vangnettest van de projectformulieren weigert nu ook een `nta_value_missing` op een veld waarvan het weglaten wel rekent. Een nieuwe test zet elk weglaatbaar veld van de voorbeeldprojecten op null en eist dezelfde status, vingerafdruk en indicatoren als zonder het veld.

### 9 oktober 2026 — lege velden per veld gemeld, vingerafdruk zonder null-leden

- **Lege velden in de NTA-invoer.** De app liet de lege velden (null) van het NTA-blok weg voor de projectroute. Een verplicht veld ontbrak dan, en de kern kon alleen het hele blok weigeren: één melding `nta_calculation_block_invalid` op `ntaCalculation`, zonder veld. De projectroute krijgt de lege velden nu als null; de zoektocht naar lege waarden in de kern meldt elk veld dat nog leeg is als `nta_value_missing` op zijn eigen pad, met "Ga naar". Van de formuliertoestanden van de nieuwe vangnettest (hieronder) gaf eerst 1 484 van de 2 412 de blokmelding; nu geen. Status en uitkomst van een volledig ingevuld project veranderen niet.
- **Invoervingerafdruk.** Een object-lid met de waarde null telt niet meer mee, net als een ontbrekend lid (uitzondering: `nominalThermalCapacityKw`, zie hieronder). De vingerafdruk van projecten die de app opslaat, blijft daardoor gelijk. Wat wel verandert:
  - Een API-aanroep die null-leden meestuurt, krijgt een andere vingerafdruk dan voorheen.
  - Ook de rekenhulpen die een vingerafdruk van hun getypte invoer maken (onder meer `bacs_draft`, `epus_draft`, `final_energy_draft`, `heating_aux_draft` en `hybrid_heat_pump_monthly_draft`) krijgen voor elke aanroeper een andere vingerafdruk: een leeg optioneel veld werd als `null` meegeteld en valt nu weg. Niets controleert die vingerafdrukken later; ze dienen alleen als herkomst.
  - De valideer- en de projectroute geven alleen dezelfde vingerafdruk zolang het project geen `maatwerkadvies` of `basisopname` bevat: de projectroute laat die blokken weg voor de vingerafdruk, de valideerroute niet (dat was al zo).
  - Een project dat met een ontwikkelversie van rekenkern 0.3.0 vóór deze wijziging is opgeslagen, met null-leden buiten `ntaCalculation` (bijvoorbeeld in `basisopname`), kan bij openen "invoer gewijzigd" melden. Uitgebrachte versies hebben dit niet.
- **Verticale leiding in een utiliteitsgebouw.** Zonder gebouwhoogte H (7.17a) volgde `vertical_pipe_storeys_invalid` op de bouwlagen, een veld dat het utiliteitsformulier niet toont. Nu `vertical_pipe_building_height_invalid` op `buildingHeightM`; een woning houdt de melding op de bouwlagen.

### 9 oktober 2026 — vangnet voor de projectformulieren (geen rekenwijziging)

- **Vangnet.** `src/__tests__/nta-project-form-guard.test.tsx` opent elke NTA-sectie die de stappagina's tonen op drie projecten (het kantoor, de rijwoning en het kantoor met twee rekenzones). Elke keuze gaat door al haar opties, elk vinkje aan en uit, elke knop "toevoegen" één keer, en tot slot wordt elk getal leeggemaakt en elke lege optie gekozen. Na elke stap gaat de invoer door wat "Toepassen" en de aanroep van de projectroute doen; zo'n 2 400 verschillende toestanden. `crates/nta8800-core/tests/project_form_guard.rs` rekent ze in de gate door de kern en weigert een vorm die de kern niet kan lezen (`nta_calculation_block_invalid`, `derived_input_*`), een melding zonder route in `gapRoutes.ts` en een achtergebleven toestand die het formulier niet meer laat wissen (de lijst `LEFTOVER_STATE`, met de fouten van review 7 en 8). Een melding die alleen om een waarde vraagt, is de verwachte uitkomst van een leeg veld.
- **Gevonden en verholpen:**
  - "—" bij de tapwateropwekker schreef `kind: ""`; de kern weigerde het hele blok. Nu blijft de soort leeg (null) en noemt de kern het veld.
  - Een andere gebruiksfunctie dan woonfunctie in de algemene gegevens liet het woningtype staan (`dwelling_type_not_applicable`); het verdwijnt nu.
  - Een ander afgiftesysteem liet de hydraulische inregeling staan (`emission_balancing_inconsistent`). Luchtverwarming en lokale verwarming krijgen nu "n.v.t.", een watersysteem verliest "n.v.t."; de keuze biedt alleen wat tabel 9.3 (voetnoot a) bij het systeem toestaat.
  - Een utiliteitsgebouw kreeg de varianten C4a, C5a, C5b en D5c aangeboden, waarvoor tabel 11.5 geen f_ctrl voor utiliteit kent (`variant_not_applicable_to_category`). Ze staan er nu niet meer bij.
  - Een woning kreeg "verlichting" als opgegeven gebruik (§5.5) aangeboden (`residential_lighting_must_be_omitted`); die keuze is er nu alleen voor utiliteit.
  - "GBS aanwezig" uitvinken liet de klassen staan (`absent_bacs_has_classes`). De klassen verdwijnen nu en staan alleen bij een aanwezig GBS.
  - Opgegeven maandwaarden begonnen als lege lijst (`monthly_values_invalid`); nu als twaalf lege maanden, zodat de kern elke lege maand noemt.

### 9 oktober 2026 — projectformulieren na de achtste review (geen rekenwijziging)

De kern is ongewijzigd; de formulieren schrijven nu invoer die de kern leest.

- **Zwembad bij sportfunctie.** "Toepassen" bouwde de ventilatiefuncties opnieuw op en liet `swimmingPool` vallen, zodat het vinkje nooit werd bewaard (11.57). De vlag blijft nu staan zolang de functie sport blijft.
- **Serieschakeling.** Een tweede verdere opwekker toevoegen liet de serieschakeling staan terwijl de keuze verdween; de kern weigerde met `hot_water_series_requires_two_generators`. Toevoegen haalt de serieschakeling nu weg, voor het hoofdsysteem en voor verdere systemen.
- **Eigen gegevens per rekenzone.**
  - De kopie van de ventilatie van het blok kreeg de oppervlakte van zone 1 (`ventilation_area_mismatch`). Ze krijgt nu de id en de gebruiksoppervlakte van de eigen zone.
  - Zonder interne warmte in het blok schreef het sjabloon `{}`; nu een geldige standaard (woonfunctie of utiliteit).
  - Een eigen woonfunctie neemt het woningtype van het blok over; een andere functie haalt een achtergebleven woningtype weg (`dwelling_type_not_applicable`).
  - Delen met een andere functie bieden geen woonfunctie meer en verdwijnen bij een woonzone (`function_areas_residential_mixed`).
  - De toelichting zegt nu dat met meer zones elke zone eigen gegevens nodig heeft; verwijderen vraagt dan eerst om bevestiging.
- **Kleiner.** "Ga naar" vindt de berekende Ψ van de circulatie van een verder tapwatersysteem (`additionalHotWaterSystems[0]…`). Bij meerdere koelsystemen staat het collectieve aandeel per systeem.

### 9 oktober 2026 — opnameformulieren na de achtste review (geen rekenwijziging)

- **Lijsten met getallen.** Bij de belemmeringsfactoren van PV en ramen schreef een leeggemaakt veld `[0]` en een afsluitende ";" een extra 0. Bij PV gaf dat zonder melding een opbrengst van nul. Lege delen vallen nu weg, en een leeg veld laat de lijst weg.
- **Lege verplichte getallen.** De oppervlakte en isolatiedikte van een paneel en de inhoud van een voorraadvat schreven leeg `null`, waarna de kern de hele opname weigerde met een kale melding. "Opname doorrekenen" noemt die velden nu eerst. De opslagcapaciteiten (elektrisch, thermisch) hebben in de kern de standaard 0 en worden leeg weggelaten. De keuze van het vlak en het kozijn van een paneel heeft geen lege optie meer.
- **Een vangnet tegen deze fout.** `src/__tests__/survey-null-paths.test.tsx` maakt in elk opnameblok elk getal leeg en kiest elke lege optie. Elke `null` die overblijft, moet `emptyRequiredSurveyFields` noemen of in `training-data/nta8800-survey-null-tolerant-paths.json` staan: antwoorden die de kern als "niet beantwoord" leest. `crates/nta8800-core/tests/survey_form_shapes.rs` controleert die lijst tegen de kern, en dat een verplicht getal `null` weigert.
- **Bypass bij systeem E.** De vraag naar de bypass staat nu ook bij een gecombineerd systeem (E), waar de kern hem leest (tabel 11.12 en `passive_cooling_requires_bypass`).
- **Koeling in de utiliteitsopname.** De ontwerptemperatuur en de leidingisolatie staan alleen bij watergedragen koeling, de enige waarbij de kern ze leest, en worden gewist bij de overstap naar directe expansie.

### 9 oktober 2026 — formulieren voor de laatste invoer die alleen via JSON kon (geen rekenwijziging)

De formulierdekking ([formulierdekking](ui-redesign/formulierdekking.md)) telt geen open opties meer: 1 799 van de 1 823 kernopties schrijft een formulier, 8 zet een formulier op een manier die de check niet op naam ziet, 16 blijven bewust JSON. De kern en zijn uitkomsten zijn ongewijzigd; `crates/nta8800-core/tests/survey_form_shapes.rs` en `project_form_shapes.rs` laten de kern de nieuwe formuliervormen lezen en doorrekenen zonder nieuwe meldingen.

- **Gegevens per rekenzone** (Gebouw › Zones, bij meer rekenzones): eigen gebruiksfunctie, woningtype, delen met een andere functie (§6.5.3), thermische massa, serres en een zwembad in een sportfunctie. Een rekenzone met eigen gegevens begint als kopie van de interne warmte, de luchtstromen en de ventilatie van het blok, want de kern haalt die dan alleen uit de zone (zonder kopie: `ventilation_flow_required`).
- **Circulatieleiding** voor het hoofdtapwatersysteem en elk verder systeem (13.27–13.39). Voorheen had alleen de berekende Ψ een veld, en dat verscheen alleen als er via JSON al een circulatie stond.
- **Verder tapwatersysteem:** nominaal vermogen, extra opwekkers en serieopstelling (die verdwijnt als er niet precies één extra opwekker is).
- **Collectieve koeling, gedeeld PV-systeem, zwembad, rekenzone van de verlichting.**
- **Opname:** bron van de gebruiksoppervlakte; opslag (alleen met PV, `storage_requires_pv`); oriëntatie, helling, spouw en omtrek per vlak; panelen; opgegeven belemmeringsfactoren per raam en per PV-veld; waterzijdig inregelen en bouwlagen van de distributie; bypass; tappunten en leidinglengtes; douche-WTW; PV-ventilatie en amorfe modules; in de utiliteitsopname de koeling (ontwerptemperatuur, regeling, leidingisolatie), circulatie en voorraadvaten, en de luchtbehandelingskast binnen de thermische zone.
- **Boosterwarmtepomp (bijlage W):** de twee beproevingen hebben elk een eigen naam voor schermlezers.

### 9 oktober 2026 — opnameformulieren leveren invoer die de kern leest (geen rekenwijziging)

Na de zevende review. Elk punt hieronder liet de kern de hele opname weigeren of gaf een melding die de gebruiker niet kon oplossen.

- **Ramen met zonwerend glas.** Het laatste vinkje uitzetten schreef `null` in plaats van een lege lijst; nu `[]`. Een verwijderd raam verdwijnt ook uit deze lijst, zodat geen `solar_control_window_unknown` achterblijft.
- **Onverwarmde ruimte als begrenzing.** De opname heeft een lijst *Onverwarmde ruimten* (id en omschrijving). De begrenzing "onverwarmde ruimte" staat pas in de keuze als die lijst een ruimte heeft, en neemt dan de eerste als `spaceId`. Een verwijderde ruimte zet de vlakken die eraan grensden terug op buitenlucht.
- **Deur met glas.** Bij een glasaandeel boven 0 vraagt het formulier de glassoort (`door_glass_required` was niet op te lossen). De oppervlakte heet nu *Oppervlakte van de deur*.
- **Variant van tabel 11.5** is een keuzelijst met de codes van de kern; leeg laat het veld weg (vrije tekst gaf bij leeg of hoofdletters een geweigerde opname).
- **Nieuwe verlichtingszones en deuren** krijgen een id die nog niet bestaat, ook na het verwijderen van een eerdere.
- **Recirculatiepercentage** staat er alleen bij "aanwezig" en verdwijnt bij "geen" of "onbekend"; de kern geeft een ingevuld percentage voorrang op het antwoord (ISSO 75.1 p. 148, NTA 11.60).
- **Daglichtregeling** toont "geen" als standaard en laat het veld weg (de keuze "—" schreef `null`).
- **Lege verplichte getallen** (gemeten q_v10, oppervlakte en vermogens van verlichtingszones, de onverwarmde ruimte van een vlak) worden vóór het doorrekenen genoemd in plaats van als kale weigering van de kern.
- **Gebouwautomatisering:** het veld heet nu *Nominaal vermogen van het grootste verwarmings- of koelsysteem (opgeteld)*.
- Een gesloten of verlaagd plafond blijft alleen bij utiliteit: de kern weigert het in de woningopname (`closed_ceiling_not_in_dwelling_survey`, ISSO 82.1 tabel 7.4 kent die kolom niet).

### 9 oktober 2026 — formulierdekking eerlijk geteld, buffervat en nieuwe formulieren (geen rekenwijziging)

- **Buffervat in de distributie.** Het formulier bood het buffervat aan bij elke installatie met een forfaitaire Ψ, terwijl de kern het alleen accepteert bij een collectieve installatie met een berekende Ψ (tabel 9.16 opmerking 2: de forfaitaire Ψ bevat het bufferverlies al). Daardoor was een buffervat uit het formulier nooit geldig. Nu biedt de distributie de keuze "Ψ forfaitair of berekend uit de leidinggegevens (9.33–9.35)", en verschijnt het buffervat pas bij een collectieve installatie met een berekende Ψ. De afleverset per woning staat alleen nog bij een collectieve installatie.
- **Warmtepompen in serie** bieden de bron "bodem of grondwater onbekend" niet meer aan: tabel 9.29 heeft daarvoor geen waarde bij 65–70 °C.
- **Eigen labels** voor de bronvelden van het buffervat, de koelpomp, de afleversets en de collectieve opwekker van de boosterwarmtepomp, en duidelijkere teksten voor het stilstandsverlies van het buffervat, het woningaandeel per zone en de onderbouwing van het bronsysteem.
- **Formuliercheck.** Telt een optie alleen nog als gedekt als een formulier van haar eigen route haar schrijft, met de ouder-sleutel erbij; lezingen, vergelijkingen en commentaar tellen niet. De echte dekking is 95,2 % (was als 99,1 % gemeld); de rest staat met een reden in `formCoverage.ts` (zie `docs/ui-redesign/formulierdekking.md`).
- **Nieuwe formulieren:** douche-WTW per douche, kokendwaterkraan, gasgestookt voorraadtoestel en de ontbrekende Bbl-functies.

### 9 oktober 2026 — formulieren voor alle gewone invoer; bewaakte formulierdekking (geen rekenwijziging)

- **Formulierdekking bewaakt.** De kern levert een catalogus van elke keuzewaarde en elk optioneel veld van de projectinvoer en beide basisopnamen (`training-data/nta8800-input-options.json`, bijgehouden door een test in `option_coverage.rs`). Een nieuwe test (`nta-form-coverage.test.ts`) faalt als een optie geen formulier heeft en ook niet als bewust-JSON met reden is vastgelegd. Dekking op naam: van 1 651 naar 1 807 van de 1 823 opties; de overige 16 staan met reden in [Formulierdekking](ui-redesign/formulierdekking.md).
- **Nieuwe formulieren, projectinvoer:** boosterwarmtepomp met meetwaarden (bijlage W), warmtepompen in serie, warmtepomp met indirect vat, biomassa-combitoestel, groot direct gestookt voorraadtoestel; afleversets en b_U per tapwatersysteem; rol van de ketel en bronsysteem van de warmtepomp; pomp, buffervat, combinatieleidingen, afleverset en b_U in de distributie; leidinglengte buiten de gekoelde zone en de circulatiepomp van de koeling; ventilatorvermogen uit het motorvermogen; woningoppervlak bij meer zones; aandeel woningen per zone; aandeel per verticale leiding.
- **Nieuwe formulieren, basisopname:** woningtype en ligging, gebouwtype, constructie en plafond, gemeten q_v10, ventilatieprincipe en unit, ruimteregeling, leidingen in onverwarmde ruimten, douches en tappunten, deuren, alle glassoorten, rietdak, kruipruimtebodem, aangetoonde isolatie, onverwarmde ruimte als grens; bij utiliteit de verlichtingszones, recirculatie, volumestroomregeling, LUKA- en LBK-kanalen, bevochtiging, GBS, collectieve verwarming, zonwerend glas en koelleidingisolatie.

### 9 oktober 2026 — formulier voor de forfaitaire boosterwarmtepomp; meldteksten bij velden (geen rekenwijziging)

- De forfaitaire boosterwarmtepomp (13.162/13.163) heeft een formulier bij het tapwatertoestel: de ontwerpaanvoertemperatuur van het voedende verwarmingssysteem (leeg is onbekend) en de bron van de warmte volgens figuur W.1 (externe warmtelevering, het verwarmingssysteem of een collectieve opwekker met rendement en energiedrager). Eerder kon deze route alleen via Geavanceerd (JSON).
- Een veld met een melding toont bij aanwijzen de tekst van de melding in plaats van de kale code. Ook het controlepaneel van de rekenkern zoekt de tekst bij elke meldcode op.
- Een test controleert dat elke code die de kern kan geven, in de lijst met invoergaten een Nederlandse en Engelse tekst heeft.

### 9 oktober 2026 — handleiding bijgewerkt voor de routes sinds 8 oktober (geen rekenwijziging)

- Hoofdstuk 3 beschrijft de nieuwe invoer: dunne luchtspouwen (bijlage D.2 van NEN-EN-ISO 6946), forfaitaire koudebruggen met een onverwarmde ruimte, de verwarmde vloer onder maaiveld boven een kruipruimte, luchtverwarmers van meer typen, ketelhulpenergie en WKK-omzettingsgetallen uit een kwaliteitsverklaring (ook voor tapwater), ψ uit de leidinggegevens met de eenheden en de vaste h_a bij tapwater, en de forfaitaire boosterwarmtepomp. Een nieuwe paragraaf zegt welk bewijs bij opgegeven productwaarden hoort, en de meldingentabel noemt de nieuwe meldcodes.
- Hoofdstuk 2 noemt het fabricage- en installatiejaar van de warmtepomp onder 2020+A1 en het fabricagejaar van ventilatoren in een gebouw van vóór 1900.
- Hoofdstuk 5 beschrijft de controle bij elke wijziging (versie- en handleidingcontrole, dekking van alle opties, numerieke fuzz, robuustheid, grote projecten) en de referentiesuites met het oordeel *geen vergelijking* (afsluitcode 3).
- Hoofdstuk 7 beschrijft de verklaringen k en l bij het energielabel.
- Hoofdstuk 1 noemt de gecodeerde deeltesten van ISSO 54 versie 2.0; de lijst met ontbrekende schermafdrukken in de inhoudsopgave is aangevuld.

### 9 oktober 2026 — luchtverwarmers, onverwarmde ruimten en EP-W006h na review 6

- **Luchtverwarmers van meer typen vragen het aantal van het eerste type.** Met `emission.airHeaters.further` en zonder `count` nam de kern 1 aan. 9.23 met tabel 9.12/9.13 telt in n_H,aux alle luchtverwarmers in de zone (2025+C1 p. 299–301; 2022 p. 282), dus het aantal is dan nodig: nieuw invoergat `air_heater_count_required` op `emission.airHeaters.count`. Eén type zonder `count` blijft "alle luchtverwarmers in de zone".
- **Forfaitaire thermische bruggen met een onverwarmde ruimte die geen vlak raakt.** Sinds de forfaitaire route met een aangrenzende onverwarmde ruimte (8.4.1 opmerking 5, C.10) moest elke onverwarmde ruimte van het project een b_U hebben, ook een ruimte waar geen vlak aan grenst; zonder die b_U kwam `unheated_transmission_unresolved`. Nu bepaalt de kern b_U alleen voor ruimten waar een vlak aan grenst, en een zone zonder scheidingsvlak naar een onverwarmde ruimte vraagt er geen. Zo'n project rekent weer zoals voorheen.
- **Vlak zonder lijst `windows`.** Een vlak zonder `windows` heeft geen ramen. Bij een scheiding naar een onverwarmde ruimte gaf dat `unheated_transmission_unresolved`, bij een buitenvlak `direct_transmission_unresolved`; nu rekent het vlak zonder ramen.
- **ISSO 54 EP-W006h** gebruikt de normatieve H_ue = 5 × A_T;iu van formule I.8 (2025+C1 p. 841) in plaats van een opbouw uit opmerking 1 van I.2.4 (≈ 5,11 × A_T;iu). Aanname, zie [ISSO 54-voorbereiding](nta8800-isso54-voorbereiding.md).
- **Interface.** Bij luchtverwarmers zijn het aantal en de andere typen nu in te voeren; een ander eerste type kiezen bewaart het aantal, de andere typen, Q_h;b en de bron.

### 9 oktober 2026 — verwarmde vloer onder maaiveld boven een kruipruimte

- **Vloer onder maaiveld boven een kruipruimte of onverwarmde kelder rekent.** Een vloer met zowel `below` als `heatedBasement` gaf `ground_floor_below_and_heated_basement`. De norm staat de combinatie toe: 8.39 met U_fl van 8.43 en U_x;T = 0 (2025+C1 p. 255, 256 en 262; 2022 p. 229, 230 en 236; gelijk in alle uitgaven). De wanden onder maaiveld tellen met z·P·U_bw. Bijlage D heeft voor de combinatie geen formule; de vloer wordt in een vloerdeel (D.13/D.14, of D.15/D.16 zonder h·P·U_xw) en een wanddeel (wandtermen van D.10/D.11) gesplitst, elk met de faseverschuiving van tabel D.1. De meldcode `ground_floor_below_and_heated_basement` vervalt. h van (8.47) wordt bij deze vloer niet gebruikt: in 2022 is hij daar niet verplicht, vanaf 2023 wordt een opgegeven h geweigerd (`route_not_in_edition`).
- **ISSO 54 EP-W006f** is gecodeerd: 240 van de 266 deeltesten.
- **Dekking.** Een afgeleid geval C met de kruipruimtevloer 0,5 m onder maaiveld loopt mee in de numerieke fuzz; de fuzz faalt als hij `heatedBasement/depthM` niet meer bereikt.

### 9 oktober 2026 — forfaitaire koudebruggen met een onverwarmde ruimte, luchtverwarmers van meer typen; ISSO 54: 239 deeltesten gecodeerd

- **Forfaitaire thermische bruggen met een aangrenzende onverwarmde ruimte.** De kern weigerde deze combinatie (`forfait_thermal_bridges_unheated_space_unsupported`); die code vervalt. Volgens 8.4.1 opmerking 5 (2025+C1 p. 266, 2022 p. 240) telt de onverwarmde ruimte dan als buitenlucht en is H_U;for = 0. De scheiding hoort in H_D;for met U_iu;equi = U_C·b_U (C.10), en krijgt ΔU_for zoals elk element van H_D.
  - U_C is de U-waarde van het project (R_se 0,04, niet de R_si van 8.4.2.1). b_U is die van de ruimte in `unheatedSpaces`, opgegeven of afgeleid (8.53–8.59).
  - De scheiding telt mee in Ū van 8.3. In de TOjuli-splitsing staat ze op haar eigen oriëntatie, zonder zonnewinst.
  - Zonder b_U voor de ruimte volgt `unheated_transmission_unresolved`.
  - Projecten die eerder werden geweigerd, rekenen nu. Projecten zonder onverwarmde ruimte of met gedetailleerde bruggen rekenen gelijk.
- **Luchtverwarmers van verschillende typen in één zone (9.23).** `emission.airHeaters` heeft `count` en `further`. P_H;aux = factor·Q_h;b/n_H;aux per toestel (tabel 9.12/9.13), opgeteld over de typen. Eén axiale en drie radiale luchtverwarmers geven (0,014 + 3·0,022)/4·Q_h;b. Nieuwe code `air_heater_count_invalid` bij een aantal van 0. Bestaande invoer rekent gelijk.
- **ISSO 54 v2.0: 239 van de 266 deeltesten gecodeerd** (was 227). Nieuw: EP-W011a/b, W202e, W203j, W203o, W204h, W206a–c, W302e, W402f en W006h. De interpretaties staan in `docs/nta8800-isso54-voorbereiding.md`.
  - EP-W006f blijft open: de norm staat een vloer onder maaiveld boven een kruipruimte toe (8.38/8.39), maar de periodieke coëfficiënten van bijlage D zijn voor die combinatie nog niet uitgewerkt.
  - EP-W406j heeft geen route: een booster met één COP, waar bijlage W twee meetpunten vraagt.

### 9 oktober 2026 — correcties na de vijfde review: glastype per raam, oliehulpenergie, WKK-verklaring voor tapwater, leidinggegevens

- **Glastype per raam rekent.** Een glastype uit tabel 7.4 in `windowGlazings` gaf altijd `window_g_declared_and_table`, omdat het raam ook zijn eigen g-waarde meegaf. g_gl;n komt uit NEN-EN 410 óf tabel 7.4 (2025+C1 p. 190): het gekozen glastype vervangt nu de g-waarde van dat raam.
- **Hulpenergie van een oliegestookte ketel.** Met opgegeven constanten (§9.1) of metingen volgens bijlage O nam 9.85 alleen het gasverbruik als E_H;ci, bij olie dus 0: alleen A/12 telde. Nu telt de olie-invoer mee, zoals in de forfaitaire route (p. 359).
- **WKK-verklaring ook voor tapwater.** ε_chp;th en ε_chp;el uit een kwaliteitsverklaring (§9.1) golden alleen voor ruimteverwarming. Ze vervangen nu ook voor tapwater de rij van tabel 9.31 (§13.8.4.7.4, p. 343), met invoerveld, bronverwijzing en dezelfde controles; bij methode 1 geweigerd. Met een verklaring geeft een rij zonder tabelwaarde ("n.v.t.") niet meer `chp_class_invalid`.
- **ψ van tapwaterleidingen: h_a ligt vast.** 13.27 en 13.29 rekenen voor tapwater met h_a = 8,0 W/(m²·K) (p. 551). Een eigen warmteovergangscoëfficiënt wordt geweigerd (`hot_water_pipe_surface_coefficient_fixed`); verwarming (p. 311) en koeling (p. 385) staan een andere waarde wel toe.
- **Leidingmaten in meters.** Een diameter of diepte boven 1 m geeft `pipe_dimension_not_in_metres`, met een eenhedenhint, in plaats van een aannemelijke ψ uit de verhoudingen (millimeters in een meterveld).

### 9 oktober 2026 — routes uit ISSO 54: glas per raam, opgegeven hulpenergie- en WKK-waarden, ψ uit de leidinggegevens

Het coderen van de ISSO 54-deeltesten liet vier invoerroutes zien die de rekenkern kende maar de projectinvoer niet doorgaf, of die ontbraken. Ze gelden in alle uitgaven. Projecten zonder deze invoer rekenen gelijk.

- **Glas per raam** (`ntaCalculation.windowGlazings`, 7.6.6.1.2/7.6.6.1.3, 2025+C1 p. 189–193). Per raam een glastype van tabel 7.4, vaste lamellen (7.41a/7.41b) of verstrooiende beglazing of vaste zonwering met de ISO 15099-waarden van 7.41. Een raam dat niet in de lijst staat, rekent met zijn eigen g-waarde. Nieuwe meldcodes: `window_glazing_duplicate`, `window_glazing_without_window`, `window_glazing_not_outdoor`, `window_glazing_reference_required`. Fouten in de glaswaarden komen als invoergat bij het eigen item. Formulier: Gebouw › Schil › "Glas per raam". ISSO 54 EP-W011a.
- **Hulpenergie van de ketel uit een kwaliteitsverklaring** (`generator.declaredAuxiliaryConstants`, §9.1 p. 285 met 9.85 p. 359–360). A, B, C en B_nom vervangen de forfaitaire constanten. A en C worden naar beneden afgerond op één decimaal en B op drie, zoals de forfaitaire waarden. Niet samen met de metingen van bijlage O (`boiler_auxiliary_declared_and_measured`); ongeldige waarden geven `auxiliary_constant_invalid`. ISSO 54 EP-W203j.
- **WKK-omzettingsgetallen uit een kwaliteitsverklaring** (`generator.declaredEfficiencies`, §9.1 met methode 2, 9.65 en tabel 9.31, p. 342–343). ε_chp;th en ε_chp;el vervangen de tabelregel, naar beneden afgerond op twee decimalen; ze moeten groter dan 0 zijn en samen ten hoogste 1 (`chp_declared_efficiency_invalid`). Bij methode 1 weigert de kern ze (`chp_declared_efficiency_method1`). ISSO 54 EP-W204h.
- **ψ uit de leidinggegevens voor koeling en tapwater.** Verwarming kende 9.33–9.35 al (`pipeTransmittance`). Nu ook koeling met 10.24–10.26 (p. 384–385, `pipe` `{kind: "calculated", geometry}`) en de tapwatercirculatie met 13.27–13.29 (p. 551, `circulation.calculatedPsi`), onder meer voor leidingen in de constructie. Nieuwe meldcodes: `cooling_pipe_geometry_invalid`, `hot_water_pipe_geometry_invalid`, `hot_water_psi_declared_and_calculated`. ISSO 54 EP-W202e, EP-W302e, EP-W402f.

### 9 oktober 2026 — boosterwarmtepomp forfaitair (13.162/13.163); ISSO 54: 227 deeltesten gecodeerd

- **Boosterwarmtepomp zonder meetgegevens.** §13.8.4.4 laat de warmte uit het verwarmingssysteem (13.162) en het elektriciteitsgebruik (13.163) van een boosterwarmtepomp ook forfaitair bepalen (2025+C1 p. 631–633, 2022 p. 607–609); de kern kende alleen bijlage W met metingen. Nieuwe opwekker `booster_heat_pump_forfait` met de ontwerpaanvoertemperatuur van het voedende verwarmingssysteem: η_gen;hj 1,15 bij 24 °C en 1,05 bij 40 °C, COP 3 en 4, geïnterpoleerd en tussen 20 en 44 °C geëxtrapoleerd, daarbuiten de waarde bij 20 of 44 °C; onbekend 1,15 en 3. c_W;gen volgt klasse 4 van tabel 13.27 op de bruto warmtebehoefte, f_prac;gi is 1,0 en er zijn geen terugwinbare verliezen (13.164). Een invoer die eerder geen route had, rekent nu. Nieuwe meldcode `booster_supply_temperature_invalid`.
- **28 deeltesten erbij,** nu 227 van de 266 (172 woning, 55 utiliteit). Alle rekenen in de gate, zonder verwachting.
  - Woning: isolatie (EP-W002a–c), kruipruimte en dijkwoning (W006a–e/g), gebruiksoppervlak (W013a–e), lessenaarsdak (W014a), dubbelzijdige zomernachtventilatie (W103c), twee rekenzones met hybride warmtepomp (W203l–n), vrije koeling met opslag (W303c/d) en de boosterwarmtepomp in een woongebouw (W406i/k).
  - Utiliteit: luchtbehandelingskast in een kantoor van 8 bouwlagen (EP-U202a–c), sportgebouw van 1536 m² (U502c) en bevochtiging met warmtewiel (U701d).
- **Niet te coderen:** EP-W006f (vloer onder maaiveld boven een kruipruimte), EP-W006h (AOR bij forfaitaire bruggen) en EP-W406j (booster met één COP). Zie de [voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md).

### 9 oktober 2026 — ISSO 54: 199 deeltesten gecodeerd (geen rekenwijziging)

- **84 deeltesten erbij,** nu 199 van de 266 (149 woning, 50 utiliteit). Alle rekenen in de gate, zonder verwachting.
  - Woning: koeling (EP-W301a/b/d, W302a–d/f, W303a/b/e), afgifte en distributie verwarming (W201b–d, W202a–d/f), ventilatie (W101c/r/s/w, W102a/b, W103a/b), opwekking (W203g, W204b/f/g/i, W205a/c), tapwater (W402a–e/g, W406d/e/f/m/n/u/v) en overstekken en belemmeringen (W008a–e, W009a–f, W010a–e).
  - Utiliteit: ventilatie (EP-U201a/c, U203a–c), verwarming (U301a, U302a/b, U303f/g), koeling (U401a, U402a, U403a), tapwater (U501a, U502a/b, U503a, U504a) en bevochtiging (U701a–c).
- **Referenties zonder leidingen in onverwarmde ruimten.** EP-W001 en EP-U001 noemen geen verwarmingsleidingen door onverwarmde ruimten, maar de codering liet `unheatedPipeLengthM` leeg, zodat 9.36 15 % als onverwarmd nam. Nu 0. Dit verandert alleen de vastgelegde uitkomsten van de ISSO-gevallen (EP-W001 BENG 2 105,18 → 105,07), niet de rekenkern.
- Interpretaties en de nog open deeltesten staan in de [voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md).

### 9 oktober 2026 — ISSO 54: 115 deeltesten gecodeerd; zonneboiler met geïntegreerde naverwarming

- **Zonneboiler met geïntegreerde naverwarming.** Een indirect gestookt vat (ketel of warmtepomp) die het back-updeel van een zonneboilervat verwarmt, vroeg een eigen voorraadvat (`hot_water_storage_required`). §13.6.2 (2025+C1 p. 566, 2022 p. 547) zegt dat het verlies van het vat en het naverwarmerdeel bij een zonne-energiesysteem volgens 13.7 gaat. Bij een zonneboiler met `integrated_backup` voor tapwater is een apart vat nu niet meer vereist. Bij een voorverwarmer blijft het vat van de ketel nodig.
- **Back-updeel gelijk aan het hele vat.** Een opgegeven `backupVolumeL` gelijk aan `totalVolumeL` werd geweigerd (`hot_water_storage_volume_invalid`). De forfaitaire waarde van 13.80 (2025+C1 p. 581) geeft onder 80 l zelf V_sto;bu = V_sto, dus dat is geldig. Alleen meer dan het vat wordt nog geweigerd.
- **ISSO 54 versie 2.0.** 63 deeltesten erbij, nu 115 van de 266 (86 woning, 29 utiliteit): EP-W003a/c, W012, W015, W016, W203a/d–f/h/i/p, W204a/c/e, W401, W403–W405, W406a/b/c/g/q, W501, EP-U303a–e, U601 en U602a–f/i. Alle rekenen in de gate, zonder verwachting.
  - EP-W201e verandert alleen de afgifte, de NEN-EN 215-verklaring en de inregeling; de regeling per ruimte blijft die van EP-W001.
  - Niet te coderen: EP-W204d (tabel 9.30 heeft geen rendement voor biomassa die niet aan bijlage R voldoet) en EP-U602g/h (opgegeven daglichtfactoren zonder geometrie). Zie [voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md).

### 9 oktober 2026 — wachtrij en tijdslimieten van de API, oordeel van de referentierun (geen rekenwijziging)

- **Geen "geslaagd" zonder vergelijking.** Een referentierun waarin elk geval `pending_expectation` is, vergeleek niets maar meldde "geslaagd" en afsluitcode 0. Het rapport heeft nu `comparisonVerdict` (`passed`, `failed`, `no_comparison`) en `comparedCases`; zonder vergeleken geval is `numericComparisonPassed` onwaar, zegt het rapport "geen vergelijking (0 vergeleken, 52 zonder verwachting)" en is de afsluitcode 3. De gate draait alle suites samen en vergelijkt de openbare gevallen en de RVO-woningen, dus die blijft groen.
- **Een geval zonder verwachting met een niet-eindige waarde** legt die waarde niet meer vast: het krijgt status `calculation_unavailable` met code `metric_not_finite`.
- **Wachttijd en berekening hebben elk een eigen tijdslimiet.** Eerder deelden ze één limiet: een verzoek dat bijna de hele tijd op een rekenplaats had gewacht, startte toch een berekening die meteen werd afgebroken maar haar plaats bleef bezetten. Nu geeft een wachttijd boven `--queue-timeout-s` (standaard 30 s) 503 `server_busy` zonder berekening, en krijgt de berekening de volle `--calculation-timeout-s`.
- **Begrensde wachtrij.** `--max-waiting` (standaard twee keer het aantal rekenplaatsen) begrenst de wachtende verzoeken; het verzoek wordt toegelaten of geweigerd voordat de body wordt gelezen, zodat wachtende bodies het geheugen niet onbegrensd vullen.
- **`Retry-After`** op elke 503 (een kwart van de rekentijdslimiet, 1–60 s), ook in het OpenAPI-document.
- **Standaardaantal rekenplaatsen** wordt begrensd op 1024, zodat de API op een machine met meer kernen zonder vlag start.

### 9 oktober 2026 — voorbereiding op de testset ISSO 54 (geen rekenwijziging)

- **Gevallen zonder verwachting in het referentieharnas.** Een referentiegeval mag nu `pending` geven in plaats van `expected`: de reden en de metrieken die het vastlegt, met de band die straks geldt. Zo'n geval wordt doorgerekend en krijgt status `pending_expectation`, zonder oordeel. Het laat de gate slagen, maar een geval dat niet rekent, laat hem falen. Het rapport toont "geen verwachting" en het aantal van zulke gevallen (`pendingExpectationCases`). `pending` naast verwachte waarden of een labelklasse wordt geweigerd (`pending_with_expected_values`); verder zijn er de codes `pending_reason_required` en `pending_metrics_required`. Bestaande gevallen en uitkomsten veranderen niet.
- **ISSO 54 versie 2.0 gecodeerd.** De referentiewoning EP-W001 en het referentiekantoor EP-U001 staan als volledige projecten in `training-data/isso54/` en rekenen in NTA 8800:2022. 52 deeltesten staan als patch op de referentie in `training-data/reference-suites/isso54-v2.json`, gegenereerd door `scripts/isso54-suite.py`:
  - woning: isolatie en raam (EP-W003b), oriëntatie, thermische massa, infiltratie, 19 ventilatiesystemen, afgifte en opwekking;
  - utiliteit: tien gebruiksfuncties en zonwering.
  
  De gate rekent ze alle door. De verwachte uitkomsten (bijlage 2 van ISSO 54) zijn niet in bezit. De codering, de interpretaties en de open deeltesten staan in [voorbereiding ISSO 54](nta8800-isso54-voorbereiding.md).

### 9 oktober 2026 — grote projecten: metingen, rekenplaatsen en tijdslimiet in de API (geen rekenwijziging)

- **Grote projecten gemeten.** Nieuwe test `crates/nta8800-core/tests/large_projects.rs` maakt projecten tot 200 utiliteitszones (1 800 ramen) en 200 woningen. Ze rekenen allemaal, met eindige uitkomsten. Rekentijd, uitvoer en geheugen groeien lineair: ongeveer 4 ms, 72 kB en 0,7 MB per zone, en 0,76 s voor 200 zones. Er was geen algoritmische correctie nodig, dus de uitkomsten blijven gelijk.
- **Controle bij schalen.** Een n keer herhaald gebouw houdt dezelfde BENG 1, en verwarming, ventilatie en verlichting worden precies n keer zo groot. Zie [Grenzen en prestaties](nta8800-programmabeschrijving.md#8-grenzen-en-prestaties).
- **HTTP-API.** Nieuwe opties `--max-calculations` (standaard het aantal kernen) en `--calculation-timeout-s` (standaard 120 s). Verzoeken boven het aantal rekenplaatsen wachten. Na de tijdslimiet volgt 503 met de nieuwe codes `server_busy` of `calculation_timeout`, in de gewone foutenvelop. Zo kan een groot verzoek (16 MB is ongeveer 1 500 zones) de service niet onbeperkt geheugen laten gebruiken.
- **Interface.** Nieuwe test `src/__tests__/large-project-ui.test.tsx`: de invoerpagina's van een project met 40 zones en 360 ramen, en alle resultaattabbladen van een antwoord met 40 zones, tekenen binnen een ruime tijdsgrens.

### 9 oktober 2026 — derde review: opname vóór 1900, lucht/lucht-warmtepomp, zonwering per raam

- **Opname van vóór 1900 met mechanische ventilatie rekent weer.** Bij een onbekend fabricagejaar van de ventilator vult de opname het bouwjaar in (tabel 11.15). Sinds de ondergrens 1900 voor het fabricagejaar weigerde de kern dan een bouwjaar van 1800–1899 (`manufacture_year_invalid`). De opname neemt nu het bouwjaar, maar niet vroeger dan 1900. Dat verandert de uitkomst niet: tabel 11.20 en 11.23 zetten elk jaar tot en met 1900 in hun oudste rij (2025+C1 p. 516, 519). Een ingevuld fabricagejaar buiten 1900–2100 geeft nu `manufacture_year_invalid` op het opnameveld `ventilation.unitManufactureYear`, in plaats van diep in de afgeleide invoer.
- **Kwaliteitsverklaring van een lucht/lucht-warmtepomp wordt gecontroleerd.** Bij afgifte aan de binnenlucht ging de verklaring zonder de controles van §9.1 (p. 285) de berekening in: een lege bron werd geaccepteerd, een waarde onder 0,05 gaf COP 0, 10⁶ werd COP 10⁶ en −1 werd stil de forfaitaire 2,8. Nu gelden dezelfde controles als bij waterzijdige afgifte (`source_required`, `heat_pump_declared_efficiency_invalid`, `heat_pump_declared_fraction_invalid`, `heat_pump_declared_auxiliary_invalid`). Een geldige verklaring rekent als voorheen.
- **Zonwering per raam, raam niet meer aan buitenlucht:** het formulier noemt het raam bij naam met "niet meer in een buitenvlak" en de code `window_shading_not_outdoor`, zoals de kern, in plaats van "ontbreekt".
- **Referentiesuite:** de band van BENG 2 bij geval H is weer ±0,05, zoals in `public_comparison.rs` (de relatieve band verbreedde hem tot ±0,08). De kop noemt A–H.
- **Bijlage V in 2020+A1 gecontroleerd:** tabel V.1 en V.3 hebben dezelfde getallen als in de latere uitgaven (2020+A1 p. 1069, 1072); de toegestane c_source 1,00 / 1,02 / 1,04 geldt dus in elke uitgave.

### 9 oktober 2026 — bronfactor gaswarmtepomp, ontwerpafgiftetemperatuur ketel en negatieve projectgegevens

Invoer die eerder rekende, wordt nu geweigerd. Geldige projecten rekenen als voorheen.

- **Bronfactor van de gaswarmtepomp.** De losse diagnose van de gaswarmtepomp (tabellen 9.27/9.29) nam elke positieve c_source aan, bijvoorbeeld 1,1. Bijlage V, tabel V.1 (2025+C1 p. 1114) geeft alleen 1,00, 1,02 en 1,04, en de elektrische warmtepomp volgde dat al. Een andere waarde geeft nu `source_correction_invalid`; het formulier weigert haar ook.
- **Gemiddelde ontwerpafgiftetemperatuur van de ketel.** Toegestaan was −30 tot 120 °C. Tabel 9.26 noot a (2025+C1 p. 329) definieert θem;avg als het gemiddelde van de ontwerpaanvoer- en retourtemperatuur van het verwarmingswater (voorbeelden 45/38 tot 90/70). Nu geldt 20 °C < θem;avg ≤ 100 °C: daaronder verwarmt het water niet boven de binnentemperatuur, daarboven kookt het. Een waarde daarbuiten geeft `average_emission_temperature_invalid`.
- **Negatieve projectgegevens.** Een projectbestand met een negatieve of niet-eindige vloeroppervlakte, inhoud of hoogte van een zone, R_c of U van een constructie, of COP of dekkingsgraad (boven 1) van een verwarmingssysteem wordt bij het openen geweigerd. De NTA-berekening leest deze getallen niet, maar de editor, de vereenvoudigde berekening en de export wel. Het zone-, constructie- en verwarmingsvenster negeren een getypt minteken.
- **Numerieke fuzz.** Bereikt nu ook de gedeclareerde f_ctrl en een gedeclareerde F_c per raam; de test faalt als hij die posities niet meer raakt.

### 9 oktober 2026 — gedeclareerde f_ctrl en zonwering per raam

Twee nieuwe invoerroutes. Projecten zonder deze invoer rekenen precies als voorheen.

- **Gedeclareerde f_ctrl** (`ventilation.system.unit.declaredControlFactor`: `value` en `declarationReference`). Een kwaliteitsverklaring van het toegepaste ventilatiesysteem, zoals een BCRG-gelijkwaardigheidsverklaring, mag de rekenwaarde van tabel 11.5 vervangen (2025+C1 p. 7 en p. 460). De waarde geldt als f_ctrl;tabel 11.5 in 11.48/11.49, bij E.1 als f_ctrl;overig. Nieuwe meldcodes: `declared_control_factor_invalid` (niet groter dan 0 of boven 2) en `declared_control_factor_not_applicable` (op het decentrale deel van E.1).
- **Zonwering per raam** (`windowShadings`: per raam `windowId`, `movableShading` en `sourceReference`). 7.42 en 7.43 gelden per raam (2025+C1 p. 196–198). Een raam in de lijst krijgt zijn eigen zonwering of geen; de andere ramen houden `windowSolar.movableShading`. Nieuwe meldcodes: `window_shading_duplicate`, `window_shading_without_window`, `window_shading_not_outdoor` en `window_shading_reference_required`; een ongeldige F_c of een lege bron is een invoergat op het eigen pad.
- **Formulier:** een vinkje "Gedeclareerde f_ctrl" bij het ventilatiesysteem en een tabel "Zonwering per raam" onder Gebouw › Schil & ramen.
- **Openbare gevallen:** A rekent met de f_ctrl 0,50 en G met de f_ctrl 0,51 en de screens op het dakraam, zoals de rapporten.
  - A: BENG 2 35,15 → 34,37 (2025+C1) en 36,06 → 35,29 (2020+A1).
  - G: 65,67 / 30,89 / 71,8 → 65,42 / 29,71 / 72,1; het rapport geeft 64,52 / 29,44 / 70,6.

### 9 oktober 2026 — numerieke fuzz van de projectinvoer; rendementen die op 0 afronden, c_source buiten bijlage V en onmogelijke fabricagejaren geweigerd

Invoer die eerder rekende of als niet-eindig resultaat werd geweigerd, krijgt nu een invoergat met een eigen code. Bestaande projecten met geldige invoer rekenen hetzelfde.

- **Gedeclareerd tapwaterrendement onder één afrondingsstap.** §13.8.4.7.2 (p. 640) rondt een gedeclareerde waarde naar beneden af op 0,05 (elektrisch) of 0,025 (gas). Een warmtepomp met een gedeclareerde waarde onder 0,05 kwam zo op 0 uit; de energie-inzet deelde door nul en het project werd geweigerd met `non_finite_result`. Nu volgt `hot_water_efficiency_invalid` op `hotWater.generator.declared.value` (warmtepomp onder 0,05; gastoestel en indirect verwarmd toestel onder 0,025). Elke opwekker die op een rendement van 0 uitkomt (ook de meetroute volgens NEN-EN 16147) wordt geweigerd in plaats van door te rekenen.
- **Gedeclareerd opwekkingsrendement van een warmtepomp voor verwarming** onder 0,05 rondt (§9.1, p. 285) ook op 0 af. Het gaf de misleidende melding `cop_overflow` op de bronfactor; nu `heat_pump_declared_efficiency_invalid` op `qualityDeclaration.generationEfficiency`.
- **Bronfactor c_source** van een elektrische warmtepomp op bodem of grondwater: bijlage V (tabel V.1, p. 1114) kent alleen 1,00, 1,02 en 1,04. Elke andere positieve waarde werd aanvaard (ook 10¹²); nu `source_correction_invalid`, zoals al bij tapwater en de gasabsorptiewarmtepomp.
- **Fabricagejaar** van ventilatoren (tabel 11.23, p. 519, en tabel 11.20) en van de warmteterugwinunit: een jaar buiten 1900–2100 (bijvoorbeeld −1) koos stilzwijgend de oudste rij. Nu `manufacture_year_invalid`; leeg blijft "onbekend". De tekst van `installation_year_invalid` noemt nu ook de grens 2100 in plaats van 2026.
- **Nieuwe test `crates/nta8800-core/tests/numeric_fuzz.rs`** naast de optiedekking. Elk getal in de voorbeeldprojecten en de openbare gevallen A–H (1 724 getallen op 117 posities) krijgt randwaarden: 0, −0, −1, de tegengestelde waarde, 10⁻¹², 10¹², 10³⁰⁰, een breuk voor een geheel getal en de randen die de naam aangeeft (jaar, fractie, hoek, temperatuur). Daarnaast 200 vaste willekeurige combinaties (factor 0,5–2) en monotonie: hogere U of infiltratie of lagere Rc verlaagt de warmtebehoefte nooit. Elke berekening rekent plausibel (behoefte niet negatief, aandeel hernieuwbaar volgens (5.3)) of weigert met een benoemde code en een invoergat met route; samen 7 689 berekeningen in vijf uitgaven.

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

### 8 oktober 2026 — opname: luchtdichtheid van ventilatiekanalen; herlabelen vergelijkt de opname

- **Luchtdichtheid ventilatiekanalen in de opname** (`ventilation.ductAirtightness`): LUKA A/B/C, LUKA D, geen kanalen of onbekend; onbekend geeft de standaardwaarde f_lea;du 1,1.
- **Herlabelen van een opnameproject.** Een project zonder eigen NTA-invoer met een basisopname (bestaande bouw) vergelijkt en hasht nu de antwoorden van de opname, zonder de redenen bij standaardwaarden. Eerder sloeg de vergelijking de opname over en zag zij geen enkele wijziging. Opnamesleutels voor het ventilatiesysteem, glas en kozijn en het paneeloppervlak van PV vallen onder bijlage 6a/6b. Gedeelde hashgevallen bijgewerkt.

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

### 6 oktober 2026 — basisopname: alle ontwerptemperatuurklassen en tabel 9.28

Uitkomsten veranderen alleen voor opnames die een nieuwe klasse of tabel 9.28 gebruiken.
- **Ontwerptemperatuurklasse.** De opname biedt nu alle klassen van NTA-tabel 9.14 (30/27 tot en met 90/70) plus 60/45 en 70/50 van de ISSO-formulieren. Aanvoer- en gemiddelde temperatuur komen uit die tabel; 60/45 en 70/50 nemen de regel met dezelfde aanvoertemperatuur (60/50, 70/60). De verklaring voor een warmtepomp boven 70 °C wordt ook bij 75/65 en 80/60 gevraagd.
- **Tabel 9.28 in de opname-wizard.** Bij een elektrische warmtepomp met bodem, grondwater of buitenlucht als bron kan de adviseur aangeven dat het toestel aan tabel 9.28 voldoet, met product, meetrapport, meetnorm en de gemeten COP per testconditie. Voorheen kon dat alleen in de JSON-weergave.
- **Release.** Versie 0.1.7-alpha; de release-job slaat een bestaande release over in plaats van te falen.
- **Leidingdoorvoeren onbekend (7.3.3).** Eén ongeïsoleerde doorvoer per bouwlaag in plaats van elke leiding door alle bouwlagen: een woning van drie lagen krijgt 5,4 in plaats van 16,2 W/K. Dit verlaagt de warmtebehoefte van grondgebonden opnames met meer dan één bouwlaag zonder opgegeven leidingen. De RVO-regressiewaarden zijn bijgewerkt; de tussenwoning en twee-onder-een-kap liggen nu dichter bij RVO (+0,8 % en +2,2 %).
- **EDR-effect (ISSO 54).** EPWReal B01 BENG 2 van +12,2 % naar −0,3 %, B05 van +2,0 % naar +0,4 %, B02 naar +0,9 %, B03 warmtebehoefte van +4,6 % naar +0,3 %.

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

### 5 oktober 2026 — rekenkern in de browser via WebAssembly (geen rekenwijziging)

Rekenuitkomsten veranderen niet. De webversie rekent nu met dezelfde Rust-kern als de desktop-app en de HTTP-API, in de browser zelf.
- **Nieuwe crates** `nta8800-operations` (het operatieregister, losgemaakt van de servicecrate) en `nta8800-wasm` (de WebAssembly-adapter met `run`, `version` en `list_operations`).
- **Eén aanroeplaag** in de UI, `kernelTransport.ts`: desktop-app via Tauri, browser via wasm, ontwikkelserver via de `/api`-proxy.
- **Gebouwd pakket** in `src/kernel-wasm/` (`npm run build:wasm`); het wasm-bestand van circa 8,5 MB laadt bij de eerste kernaanroep.
- **Referentiegate** accepteert een basisopname als invoer; `scripts/edr-manifests.js` maakt manifesten uit de EDR-testen van ISSO 54 (zie het referentieprotocol).
- Uitleg in [nta8800-wasm.md](nta8800-wasm.md).

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
