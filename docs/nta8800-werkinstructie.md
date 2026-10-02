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

Het paneel **NTA 8800-berekening (Rust-kern)** toont daarna direct A_g, A_ls en A_ls/A_g. A_ls is de som van de vlakken die aan buitenlucht, grond of een onverwarmde ruimte grenzen.

## 2. NTA-invoer invullen

Kies **NTA-invoer starten**. Het formulier vraagt alleen wat het projectmodel nog niet heeft. Bij elke waarde hoort een bron, zoals een tekening, productblad, normtabel of opnamerapport. Een leeg veld blijft een invoergat. Er wordt niets stilzwijgend aangenomen.

| Onderdeel | Wat invullen | Normbasis |
|---|---|---|
| Algemeen | rekenscope, bron A_g, Bbl-gebruiksfunctie, actieve koeling (systeem plus capaciteitsbewijs), vergunningaanvraag na 29 mei 2026 | Bbl tabel 4.148A, §5.7.1, Omgevingsregeling 5.11 lid 5 |
| Setpoints | woning 20/24 °C, utiliteit volgens tabel | tabel 7.13 |
| Thermische massa | klasse vloer en wand, plafondkolom | tabel 7.10–7.12 |
| Interne winst | aantal woningen, of W/m² bij utiliteit | 7.21–7.24 |
| Ramen | kozijnfractie; belemmering (minimaal of opgegeven); beweegbare zonwering met F_c en bediening | 7.32, §17.3, 7.42/7.43 |
| Dakhellingen | helling per hellend dakvlak | tabel 17.2 |
| Vloeren op grond | blootgestelde omtrek P en R_si + R_c | 8.30–8.41 |
| Ventilatie | H_ve of debiet in m³/h, inclusief infiltratie | (hoofdstuk 11 nog niet in de kern) |
| Afgifte en distributie | afgiftesysteem, inregeling, regeling; distributie alleen in de verwarmde zone | tabel 9.2–9.4, §9.4.1 |
| Opwekker | gasketel, (hybride) warmtepomp, stadsverwarming, elektrisch of biomassa | §9.6 |
| Koeling | opwekker; afgifte (type, inregeling, regeling, aantal ventilatorconvectoren); watergedragen distributie (ontwerptemperatuur, leidingen) | tabellen 10.4–10.16, 10.29–10.35 |
| Tapwater | aantal woningen of gebruiksfunctie met oppervlakte; aangesloten tappunten en lengte van de uittapleidingen; toestel en gemeten toepassingsklasse | tabellen 13.1–13.3, 13.25–13.28 |
| PV | piekvermogen (panelen, opgegeven K_pk of tabel 16.1), azimut, helling, bevestiging, F_sh;obst | tabellen 16.1–16.3 |
| Bevestigingen | alle energieposten en alle eigen opwekking opgenomen; f_BACS; C1-ventilatie; batterij | §5.5 |

Deze invoer gaat via **Geavanceerd (JSON)**:

- maandwaarden en toevoertemperaturen;
- gegevens per zone (`zoneData` bij meerdere zones);
- hybride opwekkers en bijzondere belemmeringssituaties;
- meerdere koelopwekkers met vermogens;
- tapwatercirculatie, voorraadvaten en douche-WTW;
- verlichting bij utiliteit (`lighting`, hoofdstuk 14);
- oriëntaties van koudebruggen voor TO-juli.

## 3. Resultaat lezen

- **Invoergaten:** het paneel noemt per gat de code en het pad, bijvoorbeeld `ventilationFlows[0].months[3].conductanceWPerK`.
- **Afwijzing:** de kern controleert fysieke grenzen, bronnen en tegenstrijdigheden. Voorbeelden:
  - een warmtepompbron die volgens de tabel afvoerlucht is, maar niet zo is opgegeven;
  - dubbel opgegeven tapwater of koeling;
  - batterijopslag (nog niet ondersteund).
- **Uitkomst:**
  - BENG 2, BENG 3 en de indicatieve labelklasse;
  - BENG 1 alleen als de ventilatie-invoer het vaste C1-systeem voorstelt (§5.4);
  - TO-juli per oriëntatie met de toets aan 1,20, en de Bbl-toets met "voldoet", "voldoet niet" of "niet te toetsen";
  - de A0-toets als die van toepassing is;
  - een maandtabel, de lijst "niet meegenomen" en de invoervingerafdruk.
- **Rapport:** Rapport → **NTA-rekenrapport exporteren** geeft een HTML-rapport met dezelfde gegevens en alle bronnen, af te drukken als pdf.

## 4. Wat (nog) niet normatief is

Zie het [verificatiedossier](nta8800-verificatiestatus.md) en het [dekkingsregister](nta8800-dekkingsregister.md). In het kort:

- Hoofdstuk 11 (ventilatie, infiltratie, C1) en de distributie volgens 9.26 ontbreken in de kern.
- Hoofdstukken 10 (methode 3), 13, 14 en 16 en §5.7 zijn op 2 oktober 2026 tegen de normtekst herschreven. Wat daar nog ontbreekt, en de open interpretatievragen, staan in het verificatiedossier.
- Hoofdstukken 7, 8, 13, 16 en 17 zijn getranscribeerd uit normanalyses en nog niet onafhankelijk gereviewd tegen de normtekst. Hoofdstukken 5 en 9 volgen het consultatieconcept.
- Er zijn nog geen uitkomsten uit de EDR-testset (bijlage 2) vergeleken. Alleen A_g en A_ls van EP-W001 zijn getoetst.
