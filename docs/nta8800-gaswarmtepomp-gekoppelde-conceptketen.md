# Gaswarmtepomp: gekoppelde concepttermen 9.62 en 9.91/9.92

De Rust-module `gas_heat_pump_chain_draft` koppelt de al afzonderlijk aanwezige [maandterm](nta8800-gaswarmtepomp-maandtermen-concept.md) en [toestelhulpstroom](nta8800-gaswarmtepomp-hulpenergie-concept.md). Bron voor beide is het [openbare consultatieconcept van hoofdstuk 9](https://www.internetconsultatie.nl/epg2026/document/14150), §§9.6.3.1 en 9.6.8.2.2–9.6.8.2.3, vergelijkingen 9.62, 9.91 en 9.92. De voor deze lezing bewaarde PDF heeft SHA-256 `92053f92d0490bc2fc5ae9fac3c99904f4d80aaf80a3bacc0f6c00f41661118a`.

De diagnose eist één toestel-ID, aandrijving, thermisch vermogen, forfaitaire COP-basis, letterlijk dezelfde bewijsreferentie voor generatorwarmte en twaalf gelijke, afzonderlijk gevalideerde maandwaarden in beide invoeren. Bij elke afwijking worden **alle maanduitkomsten onderdrukt**. Daarmee kan een verschillend warmtevolume niet stilzwijgend in de twee conceptformules terechtkomen. De uitvoer toont per maand afzonderlijk:

- de aangeleverde generatorwarmte en eventuele afgeleide collectieve bronwarmte;
- de nog **niet aan een energiedrager toegewezen** invoerterm uit 9.62;
- de begrensde bedrijfsuren en elektrische toestelhulpstroom uit 9.91/9.92.

Bij een collectieve bodembron loopt het temperatuur- en bronbewijs uit de maandterm mee door deze keten. Een ontbrekende temperatuur verhindert alle gekoppelde deelwaarden; bij 15 °C verandert de correctie van 0,009 naar 0,022, terwijl de apart berekende toestelhulpstroom gelijk blijft. Een gerichte Rust-regressie dekt deze koppeling.

Bij individuele buitenlucht, absorptie-COP 1,6, 20 kW vermogen, 1.000 kWh aangeleverde warmte en 744 normuren in januari geeft de synthetische handcontrole 625 kWh voor de **niet toegewezen** 9.62-term en 8,54 kWh toestelhulpstroom. De laatste is apart elektriciteit. Dit zijn interne rekencontroles, geen onafhankelijke EDR-uitkomsten.

Voor de getypeerde vergelijking staan nu twee afzonderlijke synthetische JSON-cases klaar: een individuele buitenluchtbron met 25 posten en een collectieve grondwaterbron met 37 posten. De tweede gebruikt bij 1.000 kWh generatorwarmte en concept-COP 2,1 de afgeleide bronwarmte en 0,022-correctie in 9.62. De cases zijn uitsluitend invoer- en transportvoorbeelden; zij bevestigen geen normtoepassing of onafhankelijk resultaat.

HTTP: `POST /v1/nta8800/heat-pumps/gas-chain-draft/diagnose`; MCP en Tauri: `diagnose_gas_heat_pump_chain_draft`. Het maandpaneel biedt de gekoppelde knop wanneer gas-hulpstroominvoer in het project is opgeslagen. De diagnose slaat geen resultaat op in het project. `carrierAllocationAvailable`, `gasInputEnergyAvailable`, `sourcePumpOrFanIncluded`, `finalEditionVerified`, `referenceVerified` en `bengCalculationAvailable` blijven `false`. Deze keten berekent geen gasverbruik, bronpomp/-ventilator, BENG of energielabel. Voor zulke uitkomsten ontbreken de definitieve normroute, energiedragertoedeling met primaire elektriciteitsfactor en onafhankelijke actuele referentiecases.
