# Collectieve bronwarmte van een gaswarmtepomp: conceptdiagnose

Deze Rust-deelroute gebruikt uitsluitend de openbare **consultatieconcepten 2026** van [hoofdstuk 5](https://www.internetconsultatie.nl/epg2026/document/14147) en [hoofdstuk 9](https://www.internetconsultatie.nl/epg2026/document/14150). Lokale controlebestanden: hoofdstuk 5 SHA-256 `b37f3cb9afd4747a156922d9c9f106808032134f476b816efa6e8f332def8fe4`; hoofdstuk 9 SHA-256 `89465ebe5cdfbba0a6d2da7596107e2d16bc26b0297ef3d7a2297fe8cd04d035`. De concepteditie en alle factoren moeten vóór gebruik voor certificering tegen de geldende NTA 8800 en bijbehorende tabellen worden vastgesteld.

## Afgebakende route

De invoer bevat de reeds gekoppelde gaswarmtepompketen voor twaalf maanden, een expliciete bronklasse met documentverwijzing en een bevestiging met bewijs dat er geen toepasselijke kwaliteitsverklaring is. De keten controleert dezelfde generator-ID en aangeleverde warmte voor conceptvergelijking 9.62 en toestelhulpenergie. Ontbrekende of tegenstrijdige gegevens geven `invalid`; dan verschijnen geen gedeeltelijke getallen.

De collectieve bronwarmte per maand is `Qout × (1 − 1/COP)` uit de forfaitaire keten. Zij krijgt de aparte energiedrager `dh`, zoals de bronterm in conceptvergelijking 5.20; zij is **geen** gasinput. Voor een bewezen bron onder 20 °C (met uitzondering van oppervlaktewater) past de route de conceptfactoren `fP = 1,45 / 23` en `fPren = 0,95` toe. Voor oppervlaktewater, of een collectieve grondwaterbron vanaf 20 °C, gebruikt de route zonder kwaliteitsverklaring de tabelwaarden `fP = 0,9`, `fPren = 0`. De bronklasse `unknown` wordt voor grond en grondwater niet doorgerekend. Voor oppervlaktewater volgt de route de afzonderlijke clausule in hoofdstuk 9, ongeacht de temperatuurklasse. Grond wordt uitsluitend als onder 20 °C aanvaard.

De uitkomst geeft per maand en jaar de `dh`-warmte en de **conceptbijdragen** aan fossiele en hernieuwbare primaire energie. Dit is geen totaal van de gebouwprestatie. De gasdrager van de 9.62-term is nog niet toegewezen. Bronpomp of bronventilator, eventuele andere systeemhulp, kwaliteitsverklaringen volgens bijlage P, CO₂-factor, volledige E_EPus-samenstelling, BENG en energielabel zijn niet inbegrepen. De UI, HTTP-route en MCP-tool tonen deze begrenzing; `finalEditionVerified`, `referenceVerified` en `bengCalculationAvailable` blijven `false`.

Via het optionele veld `gasCollectiveSourceEvidence` kan de [EPUS-conceptdiagnose](nta8800-epus-conceptdiagnose.md) deze maandelijkse `dh`-bronwarmte zelf afleiden en ongewogen in de `dh`-drager opnemen. Dat is een samenstelling van twee conceptstappen; de gasinput en de overige dragertoewijzing blijven daarbuiten.

## Koppelingen en toets

- HTTP: `POST /v1/nta8800/heat-pumps/gas-collective-source-draft/diagnose` met `{ "input": { ... } }`; ongeldige inhoud geeft 422.
- MCP: `diagnose_gas_collective_source_draft` met `{ "input": { ... } }`.
- Tauri: `diagnose_gas_collective_source_draft` met dezelfde getypeerde invoer.
- Project-UI: paneel voor gaswarmtepompmaandtermen, zichtbaar bij een collectief bronsysteem met opgeslagen hulpenergie. Bronklasse, bronverwijzing en controle op kwaliteitsverklaring zijn expliciete velden.
- Regressie: vier Rust-kernproeven, een HTTP-adapterproef en een UI-componentproef testen scheiding van `dh`, factorselectie, bewijsgrenzen en het wissen van verouderde uitkomsten. Dit zijn **interne** tests; er is geen onafhankelijke actuele EDR-referentiecase voor deze route.
