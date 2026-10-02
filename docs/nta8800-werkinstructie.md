# Werkinstructie: NTA 8800-berekening met de Rust-rekenkern

Deze instructie beschrijft hoe je in Open Energy Studio van een project naar een **onverifieerde** NTA 8800-uitkomst komt: BENG 1/2/3, TO-juli, Bbl-toets, A0 en een indicatieve labelklasse.

De uitkomst is geen energielabel en geen bewijs voor een omgevingsvergunning. Een label wordt pas vastgesteld na registratie door een gecertificeerde adviseur (BRL 9500), met een rekenprogramma dat volgens BRL 9501 is geattesteerd (Omgevingsregeling art. 5.11/5.12). Dit programma is (nog) niet geattesteerd.

## 1. Project opbouwen

1. Maak per rekenzone een zone met gebruiksoppervlakte (A_g) en volume.
2. Leg alle begrenzingsvlakken vast: gevel, dak en vloer. Geef bij elk vlak:
   - de **thermische begrenzing**: buitenlucht, grond, onverwarmde ruimte, aangrenzende verwarmde ruimte of intern;
   - de **oriëntatie** (`horizontal` voor platte vlakken);
   - de constructie, met U-waarde.
3. Leg ramen vast met oppervlakte, U en g (loodrecht, `g_gl;n`).
4. Leg lineaire en puntkoudebruggen vast, met begrenzing. Bevestig dat de puntbruginventaris compleet is.
5. Leg onverwarmde ruimtes vast met reductiefactor b en bron, en koppel de vlakken eraan.

Bij een constructie (**Constructie bewerken**) berekent de sectie *NTA 8800 U/R_c* de waarde met de kern: uit de lagenopbouw (λ_calc met bron, luchtspouwen, warmtestroomrichting) of forfaitair volgens bijlage I. Met **Toepassen** vervangt die uitkomst de eenvoudige Σd/λ-waarde van het project.

Het paneel **NTA 8800-berekening (Rust-kern)** toont daarna direct A_g, A_ls en A_ls/A_g. A_ls is de gewogen som volgens 6.7.3: vlakken naar buitenlucht of een onverwarmde ruimte tellen volledig, vlakken naar de grond of een kruipruimte voor 0,7.

## 2. NTA-invoer invullen

Kies **NTA-invoer starten**. Het formulier vraagt alleen wat het projectmodel nog niet heeft. Bij elke waarde hoort een bron, zoals een tekening, productblad, normtabel of opnamerapport. Een leeg veld blijft een invoergat. Er wordt niets stilzwijgend aangenomen.

| Onderdeel | Wat invullen | Normbasis |
|---|---|---|
| Algemeen | rekenscope, bron A_g, gebruiksfunctie van de rekenzone (en bij wonen het woningtype: woongebouw of overige woning), Bbl-gebruiksfunctie, actieve koeling (systeem plus capaciteitsbewijs), vergunningaanvraag na 29 mei 2026 | tabel 7.13–7.15, 7.78, Bbl tabel 4.148A, §5.7.1, Omgevingsregeling 5.11 lid 5 |
| Setpoints | woning 20/24 °C, utiliteit volgens tabel; moet gelijk zijn aan tabel 7.13 voor de gebruiksfunctie | tabel 7.13 |
| Thermische massa | klasse vloer en wand, plafondkolom | tabel 7.10–7.12 |
| Interne winst | aantal woningen; bij utiliteit de tabelmethode (personen en apparatuur uit tabel 7.2/7.3, verlichting uit hoofdstuk 14 of opgegeven W_t, terugwinbaar tapwaterverlies) of een opgegeven W/m² (dan geen BENG 1-run) | 7.21–7.29, §5.4.2 |
| Ramen | kozijnfractie; belemmering (minimaal of opgegeven); beweegbare zonwering met F_c en bediening (woning handbediend of automatiek volgens ISO 52016-3, overige automatiek, utiliteit handbediend met of zonder lichtwering) | 7.32, §17.3, 7.42/7.43, tabel 7.7–7.9 |
| Dakhellingen | helling per hellend dakvlak | tabel 17.2 |
| Vloeren op grond | blootgestelde omtrek P, R_si + R_c, vloerrand (ψ per randdeel of forfait 0,5·P) en eventuele randisolatie (via JSON) | 8.30–8.41, bijlage D |
| Ventilatie | kies **Hoofdstuk 11**: gebouwhoogte, bouwjaar, kruipruimte, systeemvariant (tabel 11.5) of gecombineerd systeem E.1, WTW (rendement, bypass, opstelling, toevoerkanaal), kanalen, LBK, infiltratie (gemeten q_v10 of tabel 11.13/11.14), open verbrandingstoestellen, ventilatieve koeling met openingen, voorverwarming in roosters en ventilatoren. Oppervlakte, setpoints en functie neemt het formulier zelf over. Of kies **H_ve zelf opgeven** | hoofdstuk 11, zie [ventilatie](nta8800-ventilatie.md) |
| Afgifte en distributie | afgiftesysteem, inregeling, regeling; distributie alleen in de verwarmde zone of berekend, met installatie, ontwerptemperatuur, leidingisolatie en -lengtes en circulatiepomp | tabel 9.2–9.4, 9.26–9.51 |
| Opwekker | gasketel, (hybride) warmtepomp, stadsverwarming, elektrisch of biomassa; bij de laatste drie ook toestellen en vermogen voor de hulpenergie | §9.6, 9.85, 9.91 |
| Koeling | opwekker; afgifte (type, inregeling, regeling, aantal ventilatorconvectoren); watergedragen distributie (ontwerptemperatuur, leidingen) | tabellen 10.4–10.16, 10.29–10.35 |
| Tapwater | aantal woningen of gebruiksfunctie met oppervlakte; aangesloten tappunten en lengte van de uittapleidingen; toestel en gemeten toepassingsklasse | tabellen 13.1–13.3, 13.25–13.28 |
| PV | piekvermogen (panelen, opgegeven K_pk of tabel 16.1), azimut, helling, bevestiging, F_sh;obst | tabellen 16.1–16.3 |
| Verlichting (utiliteit) | gebruiksfunctie en oppervlakte, per verlichtingszone vermogen (forfait of armaturen), schakeling, centrale aan-schakeling, daglicht (geen of forfait), afzuiging | hoofdstuk 14 |
| Bevestigingen | alle energieposten en alle eigen opwekking opgenomen; f_BACS (1,05 alleen utiliteit); C1-ventilatie; opslag met capaciteit | §5.5, 5.14a |

Deze invoer gaat via **Geavanceerd (JSON)**:

- maandwaarden en toevoertemperaturen;
- gegevens per zone (`zoneData` bij meerdere zones);
- hybride opwekkers en bijzondere belemmeringssituaties;
- meerdere koelopwekkers met vermogens;
- tapwatercirculatie, voorraadvaten en douche-WTW;
- daglichtsectoren en meerdere armatuurtypen bij verlichting;
- hoofdstuk 11-invoer per zone (`zoneData[].ventilation`) bij meerdere zones;
- oriëntaties van koudebruggen voor TO-juli.

## 3. Resultaat lezen

- **Invoergaten:** het paneel noemt per gat de code en het pad, bijvoorbeeld `ventilationFlows[0].months[3].conductanceWPerK`.
- **Afwijzing:** de kern controleert fysieke grenzen, bronnen en tegenstrijdigheden. Voorbeelden:
  - een warmtepompbron die volgens de tabel afvoerlucht is, maar niet zo is opgegeven;
  - dubbel opgegeven tapwater of koeling;
  - opslag zonder capaciteit, of `f_BACS` 1,05 bij een woning.
- **Uitkomst:**
  - BENG 2, BENG 3 en de indicatieve labelklasse;
  - BENG 1 alleen als de ventilatie-invoer het vaste C1-systeem voorstelt (§5.4);
  - TO-juli per oriëntatie met de toets aan 1,20, en de Bbl-toets met "voldoet", "voldoet niet" of "niet te toetsen";
  - de A0-toets als die van toepassing is;
  - een maandtabel, de lijst "niet meegenomen" en de invoervingerafdruk.
- **Rapport:** Rapport → **NTA-rekenrapport exporteren** geeft een HTML-rapport met dezelfde gegevens en alle bronnen, af te drukken als pdf.

## 4. Wat (nog) niet normatief is

Zie het [verificatiedossier](nta8800-verificatiestatus.md) en het [dekkingsregister](nta8800-dekkingsregister.md). In het kort:

- Hoofdstuk 11 (ventilatie, infiltratie, C1) zit in de kern en voedt de maandberekening en de BENG 1-run. De distributie volgens 9.26 is er; het formulier vult die nog niet in.
- Hoofdstuk 7, §8.3 met bijlage D en hoofdstuk 17 zijn op 2 oktober 2026 nagelopen tegen de normtekst. Hoofdstukken 10 (methode 3), 13, 14 en 16 en §5.7 zijn op dezelfde dag tegen de normtekst herschreven, en hoofdstukken 5 en 9 zijn tegen de doeleditie gecontroleerd. Wat nog ontbreekt, en de open interpretatievragen, staan in het verificatiedossier.
- Er zijn nog geen uitkomsten uit de EDR-testset (bijlage 2) vergeleken. Alleen A_g en het omhullend oppervlak A_o van EP-W001 zijn getoetst.
