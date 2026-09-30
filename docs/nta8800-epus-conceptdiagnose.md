# Conceptdiagnose maandelijkse energie per energiedrager

Het [openbare consultatieconcept van hoofdstuk 5](https://www.internetconsultatie.nl/epg2026/document/14147), §5.5.3, bevat vergelijking 5.20 voor `E_EPus;ci` en vergelijking 5.21 voor de elektrische hulpenergie. De afzonderlijke energiefuncties komen uit hoofdstukken 9–16. Dit concept is geen gecontroleerde NTA 8800:2025+C1:2026.

Rust-module `epus_draft` neemt per energiedrager twaalf expliciet opgegeven maandrecords met ruimteverwarming, bevochtiging, ventilatie, verlichting, koeling, ontvochtiging, warm tapwater, collectieve bronwarmte en zes hulpenergieposten. Zij vermenigvuldigt uitsluitend ruimteverwarming, koeling en hun twee hulpenergieposten met de door de aanroeper opgegeven `fBACS`. De overige posten worden eenmaal opgeteld. Het concept noemt als waarden voor deze factor 1,0 en 1,05; de module accepteert alleen die twee en eist een bronverwijzing. Optionele `bacsEvidence` wordt met de aparte [BACS-conceptdiagnose](nta8800-bacs-conceptdiagnose.md) beoordeeld: een afwijkende of niet bepaalbare factor blokkeert de maanduitkomst. Een losse factor zonder dat bewijs blijft voorlopige invoer; de module toetst geen wettelijke GACS-plicht.

De concepttekst onderscheidt `el`, `gas`, `bm`, `dh`, `dw`, `dc` en `oil`. De module weigert ventilatie, verlichting en hulpenergie op een andere drager dan elektriciteit; de collectieve warmtepompbronpost mag alleen onder `dh` staan. Voor de in het concept niet uitgewerkte ontvochtigingsenergie en de hulpenergie voor bevochtiging/ontvochtiging eist zij nul. Zonder aanvullend bewijs moet de caller de voorwaarde voor collectieve bronwarmte, waaronder temperatuur en eventuele bronfactor, vooraf beoordelen; de diagnose kiest of corrigeert een handmatig opgegeven bronpost niet. Ook de woningbouwregel voor verlichting vergt gebouwcontext en wordt hier niet afgeleid.

Na validatie gebruikt de module de [conceptoptelling voor jaarlijks finaal energiegebruik](nta8800-finale-energie-conceptdiagnose.md) op haar maanduitvoer. Daarmee staan de twee conceptstappen in één Rust-pad. Ongeldige drager, dubbele of ontbrekende maand, ontbrekende herkomst, niet-eindige/negatieve waarde, verboden nulpost of overloop geven geen maand- of jaaruitkomst. Een onvolledige drager- of zonneboilerinventaris behoudt een expliciete status `incomplete`.

## Afgeleide collectieve bronwarmte van een gaswarmtepomp

Optioneel veld `gasCollectiveSourceEvidence` neemt dezelfde invoer als de [collectieve-bronconceptdiagnose voor gaswarmtepompen](nta8800-gaswarmtepomp-collectieve-bron-concept.md). Is die diagnose `diagnostic_valid`, dan vult de module per maand `collectiveSourceHeatKwh` van de `dh`-drager met de afgeleide `Qout × (1 − 1/COP)`. Die post wordt, net als in vergelijking 5.20, **niet** met `fBACS` vermenigvuldigd. De uitkomst meldt `gasCollectiveSourceDerived=true` en de vingerafdruk van de onderliggende bronbeoordeling (`gasCollectiveSourceFingerprint`), zodat herleidbaar is welke keten is gebruikt.

Grenzen die tot `invalid` zonder gedeeltelijke getallen leiden:

- ongeldige of onvolledige bronbeoordeling: `gas_collective_source_evidence_invalid`;
- geen `dh`-drager in de invoer: `gas_collective_source_dh_carrier_required`;
- een handmatig opgegeven `collectiveSourceHeatKwh` ≠ 0 op `dh` naast het bewijs: `gas_collective_source_double_count`.

De gasinput van de 9.62-term en de toestelhulpenergie worden hier **niet** automatisch op `gas` of `el` geboekt; die dragertoewijzing blijft een open verificatiepunt. Bronpomp of -ventilator zit evenmin in de afgeleide post.

HTTP: `POST /v1/nta8800/energy/epus-draft/diagnose` met `{ "input": { ... } }`; MCP en desktop gebruiken `diagnose_epus_draft`. `finalEditionVerified`, `referenceVerified` en `bengCalculationAvailable` blijven `false`. `/calculate` blijft voor geldige projecten HTTP 501.

Een synthetische maand met 100 kWh verwarming, 20 kWh koeling, 10 en 2 kWh bijbehorende hulpenergie, 5 kWh ventilatie, 30 kWh tapwater, 3 kWh tapwaterhulp en 1 kWh PV-hulp geeft bij `fBACS=1,05` precies 177,6 kWh elektriciteit. Twaalf zulke maanden geven 2.131,2 kWh en bij opgegeven 100 m² een voorlopige indicator van 21,32 kWh/m²·jaar. Dit is een handmatig rekenkundig geval, geen onafhankelijk EDR- of productreferentiegeval. Met daarnaast een lege `dh`-drager en collectief grondwaterbewijs onder 20 °C (absorptie, 20 kW, 1.000 kWh/maand, concept-COP 2,1) geeft de `dh`-drager 1.000 × (1 − 1/2,1) ≈ 523,81 kWh per maand; ook dat is alleen een interne synthetische toets.

Voor vrijgave zijn de definitieve hoofdstuk-5-tekst en BACS-factorregel, de afzonderlijke serviceberekeningen, een volgens §6.6 bepaalde oppervlakte en actuele onafhankelijke verwachte deelresultaten noodzakelijk.
