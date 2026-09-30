# NTA 8800 — eerste gap-analyse Open Energy Studio

**Datum:** 28 september 2026  
**Methode:** statische inspectie van `main`, vergelijking met aanwezige tests en met het onderzoeksdossier van Open Heatloss Studio. Dit is een prioritering voor onderzoek, geen oordeel dat een specifieke formule juridisch onjuist is. Normconformiteit vergt toetsing aan NTA 8800:2025+C1:2026 en de aangewezen testset.

| Prioriteit | Onderdeel | Concreet bewijs in huidige code | Vereiste volgende stap |
|---|---|---|---|
| P0 | Normversie | `src/core/energy/Constants.ts` bevat losse constanten zonder een versieobject. | Maak `NormProfile` met tabel-, formule- en afrondingsherkomst. |
| P0 | Testbewijs | `training-data/validate-beng.ts` accepteert 10% voor BENG 1, 35% voor BENG 2, 12 procentpunt voor BENG 3 en eindigt succesvol bij 60% geslaagde checks. | Haal onafhankelijke EDR-referentiecases binnen; toets deelposten en maak falende normcases blokkerend. |
| P0 | UNIEC3-export | De exporter schrijft nu alleen invoer als gemarkeerd concept; `summary.json` is leeg en er is geen prestatie-entiteit. De oude ZIP-opbouw is gerepareerd en een OES-export/import-round-trip slaagt. De vaste NTA-versiemetadata uit 2024 beschrijft een geanalyseerd oud formaat; externe UNIEC3-interoperabiliteit en een actuele versie zijn niet bewezen. | Bevestig de actuele, gedocumenteerde uitwisselspecificatie en test met het doelprogramma. Voeg pas na normvalidatie prestatievelden toe. |
| P0 | Energielabel | `src/core/energy/EnergyLabel.ts` neemt alleen BENG 2 als parameter en bevat één tabel voor woonlabels; de preview riep deze voor alle gebouwfuncties aan. | Splits labelregels per functie en normprofiel; tot validatie alleen indicatieve labels tonen. |
| P0 | Gebruiksoppervlak | De oude vervanging van 0 door 1 m² is verwijderd; beide indicatieve calculators en TO-juli stoppen bij ongeldig oppervlak, en de preview meldt de invoerfout. | Leg de normatieve Ag-definitie, zonegrenzen en meetregels in het Rust-datamodel vast en toets tegen onafhankelijke cases. |
| P0 | BENG-eisen | `src/core/energy/Constants.ts` heeft één vaste BENG-limiet per globale gebouwfunctie. | Modelleer alle toepasselijke Bbl-afhankelijkheden, waaronder geometrie en uitzonderingen, met onafhankelijke voorbeelden. |
| P0 | Warmtepomp | `src/core/energy/types.ts` kent slechts `heat_pump_air`/`heat_pump_ground` met één `cop`; `PrimaryEnergy.ts` deelt de jaarvraag door die waarde. | Ontwerp bron/afgifte/configuratie, normroute, maandprestaties, hulpenergie en kwaliteitsverklaring. |
| P0 | Hybride regeling | `PrimaryEnergy.ts` normaliseert ingevoerde dekkingsfracties en verdeelt de vraag proportioneel over opwekkers. | Bereken normatieve bedrijfsvolgorde, bivalentie en dekkingsaandeel per maand. |
| P0 | Tapwaterwarmtepomp | `types.ts` heeft één generiek `heat_pump`-type; `PrimaryEnergy.ts` verdeelt vraag gelijk over systemen en gebruikt één rendement. | Modelleer bron, opslag, distributie, circulatie, naverwarming en productdata apart. |
| P1 | Distributie | `Constants.ts` gebruikt vaste toeslagen van 10% voor verwarming en 15% voor tapwater. | Vervang door normatieve takken per afgifte/distributie/opslag en bewijs met cases. |
| P1 | Ventilatie | `types.ts` biedt alleen `natural`, `type_c`, `type_d`; `Constants.ts` bevat vaste SFP- en WTW-defaults. | Maak een systeem- en componentenmatrix met normroute, bypass, regeling en ventilatorenergie. |
| P1 | Klimaat/zon | `Constants.ts` bevat losse graaddagen en vaste stralingswaarden per kompasrichting. | Herleid klimaatgegevens, helling, oriëntatie, beschaduwing en tijdstap tot de doelversie. |
| P1 | Gebouwfunctie | `types.ts` biedt `residential`, `office`, `education`, `healthcare`, `retail`, `industrial`, `other`. | Leg de volledige normatieve functie-indeling en zonegebonden menging vast. |
| P1 | Primaire energie | `BENGCalculatorMonthly.ts` gebruikt een vaste PV-kreditering en `Math.max(0, netPrimaryEnergy)`. | Toets energiedragers, opwekking en saldering per normversie en grenscase. |
| P1 | TO-juli | De uitkomst wordt als `GTO` met pass/fail getoond; de rekenroute en toepassingsvoorwaarden missen een externe golden. | Vergelijk deelresultaten en oriëntatie-/koelingsgevallen met officiële referenties. |
| P2 | Onderzoeksdocumentatie | `training-data/validate-beng.ts` zegt `no distribution losses`, terwijl `PrimaryEnergy.ts` inmiddels vaste distributietoeslagen toepast. | Werk validatiedocumentatie bij en leg code-/referentieversies vast. |

## Directe werkwijze

1. Leg per regel een normparagraaf, invoerfixture en verwachte deeluitkomst vast **voordat** de berekening wordt vervangen.
2. Introduceer de nieuwe bibliotheek als afzonderlijke, pure Rust-crate; laat de bestaande TypeScript-calculator tijdelijk bestaan als expliciet `legacy_indicative` profiel voor oude projecten.
3. Laat de UI een berekening pas als `extern gevalideerd` of `geattesteerd` tonen wanneer dat voor de exacte normversie en scope is bewezen.
4. Gebruik Open Heatloss Studio uitsluitend voor vergelijkende analyse. Daar zijn warmtepompen ook nog als generieke SCOP gemodelleerd; overname daarvan zou de P0-gap niet oplossen.
