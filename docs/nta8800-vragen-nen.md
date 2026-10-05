# Vragen aan NEN / NTA 8800-commissie

Concepten voor vragen over de interpretatie van NTA 8800:2025+C1:2026. Ze zijn in eigen woorden opgesteld en verwijzen alleen naar paginanummers. Aanleiding is de vergelijking met drie openbare BENG-rapporten van geattesteerde software ([nta8800-vergelijking-openbare-rapporten.md](nta8800-vergelijking-openbare-rapporten.md)).

## 1. Afgifteverlies koeling, formule 10.15 (p. 376)

**Situatie.** Formule 10.15 geldt als θ_int,inc − θ_e,comb kleiner is dan 0. Het verlies is dan de koudeafgifte maal het maximum van twee waarden: de verhouding Δθ_int,inc / (θ_int,inc − θ_e,comb) en 0,15. Bij vloerkoeling met forfaitaire waterzijdige inregeling en regeling volgt uit tabel 10.35 en tabel 10.5 een Δθ_int,inc van −3,05 tot −3,55 K. In de tussenmaanden (mei, september) ligt θ_e,comb maar net boven θ_int,inc, zodat de noemer klein is en de verhouding groot wordt.

**Gevolg bij letterlijke toepassing.** Over het jaar is het afgifteverlies 80–83 % van de koudebehoefte. In mei en september is het groter dan de behoefte zelf. In drie woningen leidt dit tot +1,5 tot +8,6 kWh/(m²·jr) BENG 2 ten opzichte van de gepubliceerde rapporten. Die rapporten impliceren een verlies van 16–42 % van de behoefte, dicht bij de ondergrens van 0,15.

**Vraag.**
- Is het de bedoeling dat de verhouding onbegrensd groot kan worden als θ_e,comb maar net boven θ_int,inc ligt, zodat het verlies groter wordt dan de behoefte?
- Of is een bovengrens bedoeld, bijvoorbeeld een MIN in plaats van een MAX met 0,15, of een maximum van het verlies als fractie van de behoefte?
- Als er een bedoelde begrenzing is: hoe moet die worden toegepast?

## 2. Regelenergie koeling, formule 10.87 (p. 425)

**Situatie.** Formule 10.87 rekent met een regelvermogen van 0,010 kW dat volgens de tekst altijd in bedrijf is. Dat geeft 87,6 kWh per jaar per koelopwekker, ook voor een omkeerbare warmtepomp die in de zomer koelt. De gepubliceerde rapporten tonen voor de hulpenergie van koeling circa 10 kWh per jaar, wat overeenkomt met alleen pompenergie.

**Vraag.**
- Geldt de regelenergie van 10.87 in alle twaalf maanden, ook buiten het koelseizoen?
- Geldt ze ook wanneer de koeling wordt geleverd door een omkeerbare warmtepomp waarvan de regeling al in de verwarmingsberekening (hulpenergie van de warmtepomp) is meegenomen?

## 3. f_prac bij een kwaliteitsverklaring van een warmtepomp (p. 337, 340, 615–617)

**Situatie.** Formule 9.62 (tabelroute) heeft f_prac 1; formule 9.63 (gegevens volgens NEN-EN 14511/14825 via bijlage Q) heeft 0,95. §9.1 (p. 285) laat een kwaliteitsverklaring de tabelwaarde vervangen. Op p. 615 staat dat een kwaliteitsverklaring voor verwarming is opgesteld volgens bijlage Q. Voor tapwater geeft 13.152 f_prac 1,0 aan de forfaitaire waarden van 13.8.4.5–13.8.4.7 en 0,95 in alle overige gevallen.

**Gekozen lezing.** Een gedeclareerde COP voor verwarming valt onder 9.63 met 0,95. Een gedeclareerd tapwaterrendement is geen forfaitaire waarde en krijgt 0,95. Dit komt tot op de kWh overeen met de gepubliceerde rapporten.

**Vraag.** Klopt deze lezing, of blijft een gedeclareerde waarde die de tabelwaarde vervangt binnen 9.62 en 13.8.4.7 met f_prac 1?
