# Bekende beperkingen

- **Uurlijks klimaat (17.3.8):** de uurwaarden van NEN 5060 zijn niet beschikbaar. Gedeclareerde maandfactoren met bron blijven mogelijk.
- **2025+C1-referentie:** er is nog geen openbaar rekenrapport in NTA 8800:2025+C1:2026 gevonden. Voor die uitgave zijn er alleen regressiegevallen: de openbare gevallen A–C, herberekend in 2025+C1, en de RVO-voorbeeldwoningen (band ±6 %).
- **ISSO 54 v5.0:** de testset met verwachte uitkomsten is niet in bezit. 260 van de 266 v2.0-deeltests zijn gecodeerd en rekenen, maar staan op *pending*.
- **Interpretaties:** de lezingen van onder meer 10.15/10.87 (koeling), de luchtstroom van de forfaitaire ventilatorenergie (vraag 5), de verdeling van gemeenschappelijke ruimten over tapwater en verlichting (vraag 6), f<sub>prac</sub>, A0 op de labelwaarde en labelelement h staan als vragen in [docs/nta8800-vragen-nen.md](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/nta8800-vragen-nen.md).
- **MWA-regels van 24-03-2026:** niet als bron beschikbaar, dus de verwijzingen ernaar zijn nog niet geverifieerd.
- **Uniec 3:** de import van een echte Uniec 3.4-export is getest op één geanonimiseerde export (een woning met één rekenzone, `training-data/uniec3-voorbeeldwoning.json`). Dakhellingen, de omtrek van de vloer op de grond en de belemmering en zonwering per raam komen mee; koudebruggen, leidingen, bronnen en ventilatiedebieten niet. Brede compatibiliteit met andere exports is niet aangetoond. De export is een nagebouwd invoerconcept (`.input-draft.uniec3`), geen bestand waarvan vaststaat dat Uniec het accepteert. Zie [hoofdstuk 9 van de handleiding](https://github.com/OpenAEC-Foundation/open-energy-studio/blob/main/docs/handleiding-nta8800/09-bestanden-en-uitwisseling.md).
- **VABI:** de veldcodes zijn afgeleid; er is geen echte VABI-export getest. Gegevens van vloeren op de grond (omtrek, kruipruimte, verwarmde kelder) gaan in import en export niet mee; de app meldt dat.
- **EP-Online:** de upload-XSD is niet openbaar.
- **Handleiding als PDF:** alleen als er bij de release een PDF-omzetter (chromium of wkhtmltopdf) is geïnstalleerd; anders HTML.
