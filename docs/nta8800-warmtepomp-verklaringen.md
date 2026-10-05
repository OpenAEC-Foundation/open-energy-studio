# Warmtepompverklaringen: bron en invoerbewijs

**Onderzocht op 29 september 2026.** Dit is een bron- en implementatieanalyse voor Open Energy Studio, geen toestemming om een verklaring automatisch in een NTA-berekening te gebruiken.

## Primaire BCRG-bronnen

- [BCRG: API voor rekensoftware](https://bcrg.nl/nl/opnemen-register/api/) bevestigt dat gecontroleerde prestatiegegevens via een koppeling kunnen worden opgevraagd. Een API-key wordt na individuele aanvraag verstrekt; [BCRG noemt daarbij mogelijke kosten](https://bcrg.nl/nl/opstellen-energieprestatie-advies/api/). Er is op de publieke pagina geen endpoint-, authenticatie- of responsschema beschikbaar. Er is voor OES geen sleutel of contract vastgesteld.
- [BCRG over de Vabi-koppeling](https://bcrg.nl/nl/nieuws/gebruikers-van-vabi-software-profiteren-van-koppeling-met-bcrg-database/) meldt dat de koppeling voor softwareleveranciers niet exclusief is, maar dat bij warmtepompen alleen verklaringen met een NTA 8800-exportbestand in die koppeling beschikbaar waren. Beschikbaarheid van een PDF in de publieke databank bewijst dus nog geen API-dekking voor hetzelfde product.
- [BCRG: documenten voor fabrikanten/leveranciers](https://bcrg.nl/nl/energieprestatie/documenten-voor-fabrikantenleveranciers/) verwijst naar *Aanvraag Warmtepomp Versie 2026.1* en *Requirements Heatpumps NTA 8800 version 2025.5*.
- [Aanvraag Warmtepomp Versie 2026.1](https://bcrg.nl/media/filer_public/b2/5b/b25bb2a3-18c6-44d5-a64e-557bd14d58d1/aanvraag_warmtepomp_versie_20261.pdf) beschrijft de onderbouwing van gecontroleerde verklaringen. Het document noemt onder meer gemeten werkpunten, de gemeten aanvoertemperatuurrange, hulpenergie, tapwaterprofielen, bronvloeistof en bij bepaalde toepassingen ontdooicycli. Een warmtepomp met externe boiler vergt een getoetste combinatie of de toepasselijke combinatiemethode. Een boosterroute mag volgens dit aanvraagdocument interpoleren binnen het toepassingsgebied en niet extrapoleren.
- [BCRG-verklaringenbank voor warmtepompen](https://bcrg.nl/nl/databanken/energieprestaties/databank/kwaliteitsverklaringen-warmtepomp/) publiceert verklaringen voor concrete producten. Een [voorbeeldregistratie](https://bcrg.nl/nl/databanken/energieprestaties/databank/verklaring/851ec8ae-9b7a-4e41-b44a-8bd36383b72e/Vaillant-aroTHERM-pro-VWL-757.1-A-230V-%2B-VWZ-MEH-977-%2B-VIH-RW3003-BR-Vaillant-aroTHERM-pro-VWL-757.1-A-230V-%2B-VWZ-MEH-977-%2B-VIH-RW2502-B) vermeldt registratienummer, fabrikant, productcombinatie, toepassing en prestatiecategorieën. De pagina is JavaScript-afhankelijk; de volledige verklaring en alle tabellen zijn hier niet inhoudelijk geïmporteerd.
- [BCRG voor EP-adviseurs](https://bcrg.nl/nl/energieprestatie/) beschrijft het gebruik van gecontroleerde verklaringen als onderbouwing voor waarden buiten een NTA-forfait.

## Versie- en gebruiksgrens

De aanvraag *2026.1* is actueel gepubliceerd, maar verwijst in zijn tekst nog naar NTA 8800:2024 en oudere aangewezen testnormen. De Engelse *2025.5* noemt eveneens 2024. Deze documenten zijn daarom bruikbaar voor een **bewijsinventaris**, niet als vervanging van de geconsolideerde NTA 8800:2025+C1:2026. De volledige doeluitgave en de toepasbaarheid van een specifieke verklaring moeten vóór rekengebruik worden gecontroleerd. Registratie in de databank alleen bewijst niet dat een opgegeven toestel, combinatie, energiedienst, bouwfase of temperatuurbereik overeenkomt met het project.

## Vertaling naar het OES-datamodel

| Bewijsgegeven | Huidige invoer | Nog vast te leggen of toetsen |
| --- | --- | --- |
| Exacte verklaring en toestel(combinatie) | `performanceEvidence.reference` als vrije tekst; optioneel `registryRecord` met registratienummer, productnaam/combinatie, fabrikant en HTTPS-bronlink | Bronlink en producttoepassing daadwerkelijk controleren; raadpleegdatum, documentversie en eventuele intrekking vastleggen |
| Verwarmingswerkpunten | temperatuur, vermogen, drager en `testReference` | toepasselijke meetnorm/editie, gemeten range, modulatie/ontdooien, hulpenergiegrens en representativiteit voor dit product |
| Warm tapwater | dienst op een werkpunt | tappatroon S/M/L/XL, geteste vraag en ingaande energie, temperatuurinstelling, correcties en combinatie met vat |
| Koeling | reversibel kenmerk en werkpunt | gecontroleerde jaarlijkse koelprestatie, koelgrens en gebruiksbereik |
| Bron en systeemopbouw | acht broncategorieën, afgifte, hulpcomponenten en koppelingen | ventilatorcorrectie voor afvoerlucht, glycol/water, bronbewijs, boostergrenzen, hybride bedrijf en geldigheid van de combinatie |

Voordat een BCRG-waarde actief wordt, moet de software de exacte verklaring bewaren, product en gebruiksbereik laten controleren, de toegestane normroute kiezen en de deelresultaten tegen onafhankelijke verwachte waarden toetsen. Tot dan geven Rust, API en MCP alleen invoerdiagnoses en blijven `calculationAvailable=false` en `referenceVerified=false`. Zie [het dekkingsregister](nta8800-dekkingsregister.md) en [het bronnenregister](nta8800-bronnenregister.md).

De optionele registervelden worden alleen op aanwezigheid en HTTPS-vorm gecontroleerd. Een oude verklaring met alleen een vrije-tekstreferentie blijft leesbaar en krijgt `quality_declaration_registry_record_missing` als waarschuwing. Een deels ingevuld registerrecord is een invoerfout. De software haalt geen BCRG-data op, controleert geen registratie-/productovereenkomst en bepaalt geen prestatie op basis van deze velden.

**Integratiepad:** leg met BCRG voor OES de sleutel, kosten, rechten, actuele endpoints, versies, product-/combinatiesleutels, intrekkingen en exportbestanden vast. Ontwerp daarna een adapter die ruwe brondata met raadpleegdatum en bronhash bewaart, de exacte productmatch toont en alleen door de norm toegestane invoerwaarden doorgeeft aan Rust. Maak een offline/mock-contracttest én een geautoriseerde live-contracttest; geef bij ontbrekende sleutel of ontbrekende exportdata uitsluitend `onverifieerd` terug. Geen BCRG-API-respons mag zelfstandig `referenceVerified` of `attestStatus` opwaarderen.
