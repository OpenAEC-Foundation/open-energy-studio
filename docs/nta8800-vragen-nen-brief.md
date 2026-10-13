# Brief aan NEN en RVO: interpretatievragen NTA 8800

*Verzendklaar concept, opgesteld 10 oktober 2026 en bijgewerkt 11 oktober 2026 uit [nta8800-vragen-nen.md](nta8800-vragen-nen.md). Vul de velden tussen rechte haken in voor verzending. De brief citeert geen normtekst; hij verwijst alleen naar formule-, paragraaf- en paginanummers van NTA 8800:2025+C1:2026, tenzij een andere uitgave is genoemd.*

---

[organisatie]
[adres]
[postcode en plaats]

Aan: NEN, normcommissie NTA 8800 (energieprestatie van gebouwen)
Afschrift: Rijksdienst voor Ondernemend Nederland (RVO), team EP-Online / energielabel

Datum: [datum]
Contactpersoon: [contactpersoon], [e-mailadres], [telefoonnummer]
Betreft: interpretatievragen bij NTA 8800 in verband met een aanvraag van een BRL 9501-attest

Geachte leden van de normcommissie,

[organisatie] ontwikkelt Open Energy Studio, rekensoftware voor de energieprestatie van woningen en utiliteitsgebouwen volgens NTA 8800. Wij bereiden de aanvraag van een attest volgens BRL 9501 voor. De versie waar deze vragen over gaan is programmaversie 0.2.0 met rekenkern 0.4.0 (nog niet uitgebracht); de standaarduitgave is NTA 8800:2025+C1:2026.

Bij het narekenen van openbare BENG-berekeningen van geattesteerde software (vragen 1–7) bij een systematische controle van de rekenkern tegen de normtekst (vragen 8–12) en bij de BRL 9501-toetsset ISSO 54 (vragen 13 en 14) zijn wij veertien punten tegengekomen waar de tekst van de norm (of, bij vraag 4, de Omgevingsregeling) meer dan één lezing toelaat. Bij vijf daarvan verklaart het verschil in lezing een meetbaar verschil in BENG 2 tussen onze uitkomst en het gepubliceerde rapport; vragen 8–12 veranderen geen vergeleken uitkomst van de openbare gevallen. Wij volgen de letterlijke tekst, waar twee bepalingen elkaar tegenspreken (vragen 9 en 10) de lezing die aansluit op de rest van de berekening, en waar de toetsset ISSO 54 een andere lezing gebruikt dan de letterlijke tekst (vragen 13 en 14) de lezing van de toetsset, maar willen voor de attestering zeker weten welke lezing bedoeld is. Per vraag noemen wij de plaats in de norm, de twee lezingen, het gevolg in een openbaar geval en de concrete vraag. De openbare gevallen en de verschilanalyse per post staan in onze vergelijking van openbare rapporten; die sturen wij desgewenst mee.

Wij stellen een antwoord per vraag zeer op prijs, ook als het een verwijzing is naar een bestaande toelichting of een geplande correctie.

### Vraag 1. Afgifteverlies van koeling, formule 10.15 (p. 376)

- **Plaats.** Formule 10.15 (p. 376), met Δθ_int,inc uit tabel 10.35 en tabel 10.5. De formule is ongewijzigd sinds NTA 8800:2020 (p. 353).
- **Lezing A (letterlijk).** Het verlies is de koudeafgifte maal het grootste van de verhouding Δθ_int,inc / (θ_int,inc − θ_e,comb) en 0,15. Ligt θ_e,comb maar net boven θ_int,inc, zoals in mei en september, dan wordt de verhouding onbegrensd groot en het verlies groter dan de koudebehoefte zelf.
- **Lezing B.** Er is een bovengrens bedoeld, bijvoorbeeld een minimum in plaats van een maximum met 0,15, of een maximum van het verlies als fractie van de behoefte.
- **Gevolg in openbare gevallen.** De gepubliceerde rapporten impliceren een verlies van 16–42 % van de behoefte, dicht bij 0,15. Met lezing A ligt BENG 2 hoger dan in het rapport, in kWh/(m²·jr):
  - geval A +8,55, geval B +1,83 en geval C +1,46;
  - geval I +4,61;
  - geval M +18,26 (een woning van 43 m²; BENG 2 komt daarmee boven de eis van 30, het rapport blijft eronder).
- **Vraag.** Is het de bedoeling dat het verlies onbegrensd groot kan worden als θ_e,comb maar net boven θ_int,inc ligt? Zo nee: welke begrenzing is bedoeld, en hoe moet die worden toegepast?

### Vraag 2. Regelenergie van koeling, formule 10.87 (p. 425)

- **Plaats.** Formule 10.87 (p. 425): een regelvermogen van 0,010 kW dat volgens de tekst altijd in bedrijf is, samen 87,6 kWh per jaar per koelopwekker. Dezelfde tekst staat in NTA 8800:2020 (p. 403).
- **Lezing A (letterlijk).** De regelenergie telt in alle twaalf maanden, ook buiten het koelseizoen en ook bij een omkeerbare warmtepomp die in de zomer koelt.
- **Lezing B.** De regelenergie telt alleen in maanden met koudebehoefte, of niet als de regeling van een omkeerbare warmtepomp al in de hulpenergie voor verwarming zit. De gepubliceerde rapporten tonen voor de hulpenergie van koeling alleen pompenergie, circa 10 kWh per jaar.
- **Gevolg in openbare gevallen.** Met lezing A ligt BENG 2 hoger dan in het rapport, in kWh/(m²·jr):
  - geval A +1,66, geval B +0,94 en geval C +0,50;
  - geval I +0,29;
  - geval M +2,92.
- **Vraag.** Geldt de regelenergie van 10.87 in alle maanden? Geldt ze ook wanneer de koeling wordt geleverd door een omkeerbare warmtepomp waarvan de regeling al in de verwarmingsberekening is meegenomen?

### Vraag 3. Praktijkfactor f_prac bij een kwaliteitsverklaring van een warmtepomp (p. 337, 340, 615–617)

- **Plaats.** Formule 9.62 (tabelroute, f_prac 1) en 9.63 (gegevens volgens bijlage Q, f_prac 0,95); §9.1 (p. 285) over de kwaliteitsverklaring die de tabelwaarde vervangt; p. 615 over de kwaliteitsverklaring voor verwarming volgens bijlage Q; formule 13.152 voor tapwater.
- **Lezing A (onze keuze).** Een gedeclareerde COP voor verwarming valt onder 9.63 met f_prac 0,95. Een gedeclareerd tapwaterrendement is geen forfaitaire waarde en krijgt eveneens 0,95; dat geldt voor een warmtepomp, een gastoestel (tabel 13.25 of bijlage T) en een ketel met indirect verwarmd voorraadvat (tabel 13.28).
- **Lezing B.** Een gedeclareerde waarde die de tabelwaarde vervangt, blijft binnen 9.62 en 13.8.4.7, met f_prac 1.
- **Gevolg in openbare gevallen.** Lezing A komt tot op de kWh overeen met de gepubliceerde rapporten. Lezing B zou BENG 2 verlagen ten opzichte van die rapporten, in geval A met 2,82 en in geval B met 1,73 kWh/(m²·jr).
- **Vraag.** Klopt lezing A?

### Vraag 4. A0 en labelelement h bij een woning (Omgevingsregeling art. 5.11 en 5.13a)

Deze vraag gaat over de Omgevingsregeling (BWBR0045528, versie 2026-10-01), niet over NTA 8800. Wij leggen haar aan RVO voor, met het verzoek haar zo nodig door te zetten naar het ministerie.

- **A0, voorwaarde b.** Art. 5.11 lid 4 laat het primair fossiel energiegebruik van een woning bij energiemaatregelen op gebiedsniveau rekenen met forfaitaire waarden (EMGforf). Lid 5 onder b toetst het primair fossiel energiegebruik aan bijlage IXa.
  - **Lezing A (onze keuze).** Lid 5 onder b toetst dezelfde waarde als lid 4, dus de labelwaarde met EMGforf. Een woning met een warmtenet met kwaliteitsverklaring kan zo BENG 2 ≤ 27 hebben en toch geen A0 krijgen. Voor utiliteit (art. 5.12 lid 4) is de labelwaarde de waarde met kwaliteitsverklaring, dus BENG 2.
  - **Lezing B.** Lid 5 onder b toetst de BENG 2-waarde met kwaliteitsverklaring.
  - **Vraag.** Toetst lid 5 onder b de labelwaarde (EMGforf) of de BENG 2-waarde?
- **Labelelement h.** Art. 5.13a lid 1 onder h vraagt de productie van hernieuwbare energie in kWh.
  - **Lezing A (onze keuze).** Alleen de hernieuwbare elektriciteitsopwekking op eigen terrein: PV en andere hernieuwbare opwekking van hoofdstuk 16 van NTA 8800, zonder de elektriciteit van een (micro-)WKK. Zonnewarmte en omgevingswarmte zitten al in elementen c en i.
  - **Lezing B.** Ook hernieuwbare warmte, zoals zonnewarmte, in kWh.
  - **Vraag.** Bedoelt element h alleen hernieuwbare elektriciteit, of ook hernieuwbare warmte?
- **Gevolg in openbare gevallen.** Geen; de openbare rapporten tonen deze labelgegevens niet.

### Vraag 5. Luchtvolumestroom bij het forfaitaire ventilatorvermogen, formule 11.142 (p. 518; 2020+A1: 11.139, p. 495)

- **Plaats.** Formule 11.142 (p. 518) neemt q_V;ODA;req bepaald volgens 11.2.2.1. Bij 11.46–11.49 worden f_sys en f_ctrl, en daarmee q_V;ODA;req, voor de warmtebehoefte en de koudebehoefte apart berekend. Met maximale benutting van de geïnstalleerde capaciteit voor passieve koeling (tabel 11.6) is de waarde voor de koudebehoefte veel groter.
- **Lezing A (onze keuze).** q_V;ODA;req van de warmtebalans, in alle maanden.
- **Lezing B.** q_V;ODA;req van de koudebalans, of per maand de grootste van beide.
- **Gevolg in een openbaar geval.** Geval N (supermarkt, NTA 8800:2020+A1, ventilatiesysteem D.3 met CO₂-sturing en automatische passieve koelregeling) rekent in het rapport 5 164 kWh ventilatorenergie. Dat is tot op de kWh de forfaitaire formule met de luchtvolumestroom van de koudebalans (2 122 m³/h) over het hele jaar. Met de warmtebalans (938 m³/h) rekenen wij 2 283 kWh: −2,02 kWh/(m²·jr) op BENG 2.
- **Vraag.** Welke q_V;ODA;req geldt in 11.142: die van de warmtebalans, die van de koudebalans, of per maand de grootste? Als de koudebalans bedoeld is: ook in maanden zonder koudebehoefte?

### Vraag 6. Oppervlakte per gebruiksfunctie voor tapwater en verlichting bij een gemeenschappelijke ruimte (13.19, p. 538; hoofdstuk 14, p. 657 en 661; 6.6.4, p. 158)

- **Plaats.** 6.6.4 (p. 158) kent twee verdelingen van een gemeenschappelijke ruimte: voor de gebruiksfunctie in het gebouw over alle aangewezen functies (eerste alinea), en voor de gebruiksfunctie binnen een rekenzone alleen over de aangewezen functies die in die rekenzone liggen (laatste alinea). Formule 13.19 (p. 538) en de weging van t_D, t_N (p. 657) en P_n;spec (p. 661) wegen naar de gebruiksfuncties in de rekenzone. Dezelfde tekst staat in NTA 8800:2020+A1 (p. 515, 628–636), 2022 (p. 519, 633–641), 2023 (p. 527, 643–651) en 2024.
- **Lezing A (onze keuze).** De gebruiksoppervlakte binnen de rekenzone (6.6.4, laatste alinea). Formule 13.19 vermenigvuldigt met A_g van de rekenzone, en 14.4 (p. 656) laat de verlichtingszones samen A_g van de rekenzone dekken; een deel dat aan een functie buiten de rekenzone wordt toebedeeld, ligt niet in de rekenzone.
- **Lezing B.** De gebruiksoppervlakte in het gebouw (6.6.4, eerste alinea), omdat tabel 13.1 en de tabellen 14.1 en 14.3 waarden per gebruiksfunctie van het gebouw geven.
- **Gevolg in een openbaar geval.** Geval H (NTA 8800:2023) heeft een gemeenschappelijke ruimte van 49,20 m² in de rekenzone zonder sport, terwijl ook de sportfunctie erop is aangewezen. Het rapport volgt lezing B: 44,35 m² gaat naar sport, ook in de verlichting van de rekenzone zonder sport. Met lezing A rekenen wij 317 kWh minder tapwater en 248 kWh minder verlichting; BENG 2 wordt 80,74 in plaats van 81,86 kWh/(m²·jr) (rapport 80,01).
- **Vraag.** Geldt voor 13.19 en de weging in hoofdstuk 14 de gebruiksoppervlakte binnen de rekenzone of die in het gebouw? Ligt het deel van een gemeenschappelijke ruimte dat aan een functie buiten de rekenzone wordt toebedeeld, voor deze weging "in de rekenzone"?

### Vraag 7. Warmtecapaciteit van lucht in 11.115 en 11.119 (p. 505 en 506; 2022: p. 486 en 487)

- **Plaats.** Formule 11.119 (p. 506) geeft de warmte van de verwarmingsbatterij van een luchtbehandelingskast, 11.115 (p. 505) de koude van de koelbatterij. Bij beide staat c_a = 0,000 027 9 kWh/(kg·K). Elders rekent de norm met ρ_a·c_a = 1 211 J/(m³·K) (bij 8.49 en 8.57), dus c_a = 1 005 J/(kg·K) = 0,000 279 kWh/(kg·K). NTA 8800:2022 (p. 486 en 487) heeft dezelfde waarde.
- **Lezing A (onze keuze).** De gedrukte waarde 0,000 027 9 kWh/(kg·K).
- **Lezing B.** Een drukfout voor 0,000 279 kWh/(kg·K), zodat de batterij de warmte telt die de toevoerlucht volgens 11.3.2 en tabel 11.15 krijgt.
- **Gevolg in een openbaar geval.** Geval O (gezondheidszorgfunctie, NTA 8800:2022, D.3 met de luchtbehandelingskast in de zone): de warmtebehoefte van de zone daalt met ongeveer 1 600 kWh doordat de toevoerlucht op 18 °C komt, maar met lezing A telt de batterij 279 kWh, met lezing B 2 794 kWh. BENG 2 is 47,75 met lezing A en 49,80 met lezing B; het rapport geeft 49,49 kWh/(m²·jr). Bij geval L (school, NTA 8800:2022) is het 18,63 tegen 23,82, rapport 25,36.
- **Vraag.** Is c_a in 11.115 en 11.119 een drukfout voor 0,000 279 kWh/(kg·K)? Zo niet: waarom telt de batterij een tiende van de warmte die de toevoerlucht krijgt?

### Vraag 8. Gevel aan een aangrenzende onverwarmde serre bij TOjuli (5.7.2, p. 115–118)

- **Plaats.** Stap A van 5.7.2 (p. 115) neemt het geveldeel dat aan een aangrenzende onverwarmde serre grenst mee in A_T;or. Stap 2 (p. 116–117) telt H_C;D;juli;k en Q_C;sol;juli;k van constructies tussen de rekenzone en "de buitenlucht of serre". Stap 1 bepaalt H_C;D;juli;k volgens 8.2, terwijl een constructie naar een serre onder 8.4 valt (H_U met b_U); stap B noemt geen component voor H_U.
- **Lezing A (onze keuze tot uw antwoord).** Het serregeveldeel blijft buiten de verdeling per oriëntatie, zoals een gewone aangrenzende onverwarmde ruimte; de kern meldt dit bij elke rekenzone met een serre.
- **Lezing B.** Het geveldeel telt mee in A_T;or met b_U·H_zi;ztu (8.4) of U·A als transmissie en de indirecte zonnewinst van 7.30b als zonnewinst op zijn oriëntatie.
- **Gevolg.** Geen van de openbare gevallen heeft een serre. Ligt de serre op de enige gevel van een oriëntatie, dan wordt die oriëntatie met lezing A mogelijk niet beoordeeld (A_T;or ≤ 3 m²).
- **Vraag.** Welke warmteoverdrachtscoëfficiënt en welke zonnewinst horen in 5.40 en de julibalans bij het geveldeel naar een serre?

### Vraag 9. Scheidingsconstructie naar een aangrenzende onverwarmde ruimte bij TOjuli in de forfaitaire route (5.7.2, p. 115; 8.4.1, p. 266)

- **Plaats.** Stap A van 5.7.2 (p. 115) laat elementen naar een aangrenzende onverwarmde ruimte (AOR) buiten A_T;or. Bij de forfaitaire verrekening van 8.4.1 is H_U;for = 0 en zit de scheidingsconstructie in H_D;for (U_iu;equi volgens C.1.3); opmerking 5 (p. 266) beschouwt haar dan als uitwendige scheidingsconstructie, met de AOR vervangen door buitenlucht. Stap 1 van 5.7.2 neemt H_C;D;juli uit 8.2.
- **Lezing A (onze keuze).** In de forfaitaire route telt de scheidingsconstructie mee in A_T;or en H_C;D;juli van haar oriëntatie, zonder zonnewinst, zodat de verdeling aansluit op H_D;for.
- **Lezing B.** Stap A gaat voor: de scheidingsconstructie blijft ook in de forfaitaire route buiten A_T;or en buiten de verdeling per oriëntatie.
- **Gevolg.** Alleen geval O (gezondheidszorgfunctie, NTA 8800:2022) gebruikt de forfaitaire route met een AOR op een gevel; het rapport geeft geen TOjuli. Met lezing B wordt een oriëntatie met alleen zo'n gevel niet beoordeeld (A_T;or ≤ 3 m²) en krijgt een oriëntatie met ook buitengevels een kleiner aandeel in de verdeelde posten van stap B.
- **Vraag.** Geldt de scheidingsconstructie naar een AOR in de forfaitaire route voor stap A als uitwendige scheidingsconstructie, en hoort haar transmissie dan bij H_C;D;juli van haar oriëntatie?

### Vraag 10. RER_PrenTot;EMGforf bij een collectieve warmtepompbron onder 20 °C met kwaliteitsverklaring (5.3.1.2–5.3.1.3, p. 72–74; 9.6.8.1.1.2.3, p. 362)

- **Plaats.** Voor EwePTot;EMGforf geldt bij een collectieve warmtepompbron met ϑ < 20 °C dezelfde f_P;del;dh als bij EMGverklaring (p. 72 en p. 362). Voor RERPrenTot;EMGforf en EwePrenTot;EMGforf (p. 74) noemt 5.3.1.3 die uitzondering niet en verwijst naar de forfaitaire f_P;del en f_Pren van tabel 5.2 en 5.4. Voor de bron zelf geeft 9.6.8.1.1.2.3 a) forfaitaire waarden f_P;del;el/23 en f_Pren;dh = 0,95; tabel 5.4 geeft voor externe warmte zonder verklaring f_Pren = 0.
- **Lezing A (onze keuze).** De hele EMGforf-berekening houdt voor zo'n bron de waarden van de verklaring aan (f_P;del;dh, f_Pren;dh en K_CO2), zodat EPTot in EwePTot;EMGforf en in RERPrenTot;EMGforf gelijk is.
- **Lezing B.** Alleen f_P;del;dh blijft die van de verklaring; f_Pren;dh wordt de forfaitaire waarde (0,95 volgens 9.6.8.1.1.2.3 a), of 0 volgens tabel 5.4).
- **Gevolg.** Alleen de EMGforf-waarden van RERPrenTot en EwePrenTot, die bij een woning met gebiedsmaatregelen informatief op het label staan; BENG 3 rekent met EMGverklaring. Geen openbaar geval of ISSO 54-deeltest heeft zo'n bron met kwaliteitsverklaring.
- **Vraag.** Welke f_Pren;dh (en welke f_P;del;dh) hoort in RERPrenTot;EMGforf bij een collectieve warmtepompbron onder 20 °C met kwaliteitsverklaring?

### Vraag 11. Zonnewinst door de scheidingsconstructie naar een aangrenzende onverwarmde serre (7.6.1, p. 182; 7.6.4, p. 185; 7.6.6.1.1, p. 189)

- **Plaats.** 7.6.1 (p. 182) bepaalt bij de forfaitaire verrekening van 8.4.1 via een aangrenzende onverwarmde serre (AOS) de zonnewinst door de scheidingsconstructie, met de serre vervangen door buitenlucht; of 7.30b dan nog geldt, staat er niet bij. In de berekende route telt 7.34 alleen de niet-transparante vlakken in de serre (opmerking 2, p. 185), terwijl 7.6.6.1.1 (p. 189) straling die een raam pas na een serre bereikt, vermenigvuldigt met de zontoetredingsfactor van de serre.
- **Lezing A (onze keuze).** De scheidingsconstructie naar een serre krijgt in beide routes geen eigen zonnewinst, ook niet door haar ramen; de serre telt alleen via 7.30b.
- **Lezing B.** In de berekende route krijgen ramen in de scheiding directe zonnewinst volgens 7.32 met g_gl van het raam maal g_gl van de serrebeglazing, naast 7.30b; in de forfaitaire route krijgt de scheiding zonnewinst als uitwendige scheidingsconstructie (7.32/7.33) en vervalt 7.30b voor die serre.
- **Gevolg.** Geen openbaar geval of ISSO 54-deeltest heeft een serre. Lezing B verlaagt de warmtebehoefte en verhoogt de koudebehoefte van een zone met ramen naar een serre.
- **Vraag.** Krijgen ramen in de scheiding naar een AOS directe zonnewinst met de zontoetredingsfactor van de serre, en hoe wordt in de forfaitaire route de zonnewinst door die scheiding bepaald (7.32/7.33 of 7.30b, en met welke F_sky)?

### Vraag 12. Praktijkfactor f_prpr bij een kwaliteitsverklaring van een koudeopwekker (§10.1, p. 365; 10.5.4–10.5.6, p. 401–419)

- **Plaats.** §10.1 (p. 365) laat een kwaliteitsverklaring een waarde van tabel 10.29 of 10.30 vervangen. Voor de tabelwaarden geldt f_prpr = 1 (p. 419); methode 1 (10.54) en methode 2 (10.65/10.66) rekenen productwaarden met f_prpr 0,9, en methode 2 met 0,60 bij directe condensatie tegen de buitenlucht. Voor een gedeclareerde waarde in methode 3 noemt de norm geen f_prpr. Dezelfde tekst staat in 2022, 2023 en 2024.
- **Lezing A (onze keuze).** Een gedeclareerde EER of ζ is een productwaarde en krijgt f_prpr 0,9.
- **Lezing B.** Een gedeclareerde waarde vervangt de tabelwaarde en houdt f_prpr 1.
- **Gevolg.** Geen openbaar geval of ISSO 54-deeltest heeft een gedeclareerde koudeopwekker. Lezing B verlaagt het aandrijfenergiegebruik van zo'n opwekker met 10 %; een kamerairco met een verklaarde EER zou dan gunstiger uitkomen dan dezelfde airco met volledige NEN-EN 14511-gegevens in methode 2 (f_prpr 0,60).
- **Vraag.** Welke f_prpr hoort bij een gedeclareerde EER of ζ die een waarde van tabel 10.29 of 10.30 vervangt?

### Vraag 13. Klassecorrectie c_W;gen op de forfaitaire tapwaterwarmtepomp (tabellen 13.25 en 13.27, 13.8.4.7.3, p. 641–643)

- **Plaats.** Tabel 13.25 geeft een individuele warmtepomp 1,4 of 1,4·c_source, zonder c_W;gen in de cel; de toelichting onder de tabel vraagt c_W;gen bij de forfaitaire rendementen, klasse 4 als de klasse onbekend is. Tabel 13.27 begint bij 1 805 kWh.
- **Lezing A (onze keuze voor 2025+C1, zoals ISSO 54).** De forfaitaire warmtepomp krijgt c_W;gen van tabel 13.27, klasse 4 bij een onbekende klasse.
- **Lezing B (letterlijke cel, zoals de openbare rapporten in 2020+A1 en 2022).** De forfaitaire warmtepomp houdt 1,4 zonder correctie.
- **Gevolg.** ISSO 54 EPWReal B01: lezing A geeft 2 790 kWh voor tapwater (referentie 2 806 kWh) en BENG 2 73,65 (referentie 73,89); lezing B 1 339 kWh en BENG 2 49,33.
- **Vraag.** Geldt c_W;gen voor de forfaitaire warmtepomp, en welke waarde geldt onder 1 805 kWh?

### Vraag 14. Aantal fictieve verticale leidingen bij een onbekende situatie (7.3.3, formule 7.17)

- **Plaats.** Bij een onbekende situatie neemt 7.3.3 voor een woonfunctie buiten een woongebouw één ongeïsoleerde fictieve leiding per rekenzone per bouwlaag aan; opmerking 2 bij 7.17 telt per leiding het werkelijke aantal bouwlagen van de rekenzone.
- **Lezing A (onze keuze, zoals ISSO 54).** Eén leiding door alle N bouwlagen: N·1,8 W/K.
- **Lezing B (letterlijk).** N leidingen die elk N bouwlagen tellen: N²·1,8 W/K.
- **Gevolg.** Drie bouwlagen: 5,4 tegen 16,2 W/K. ISSO 54 EPWReal B03 (drie bouwlagen, "onbekend") sluit aan bij lezing A.
- **Vraag.** Welke van beide is bedoeld?

---

Wij danken u voor uw aandacht en zijn graag bereid de berekeningen per geval toe te lichten.

Met vriendelijke groet,

[naam]
[functie]
[organisatie]

*Bijlage op verzoek: vergelijking met openbare rapporten (gevallen A–O), met de verschilanalyse per post.*
