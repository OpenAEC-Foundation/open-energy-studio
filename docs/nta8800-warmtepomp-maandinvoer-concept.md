# Warmtepomp maandelijkse generatorinvoer — consultatieconcept

Bron: [openbaar concept NTA 8800:2026 hoofdstuk 9](https://www.internetconsultatie.nl/epg2026/document/14150), §9.6.3.1, formule 9.62 en §9.6.8.1.1.2.3 (PDF-pagina 50–51 en 83–84), met tabellen 9.27/9.29. De onderzochte PDF heeft SHA-256 `89465ebe5cdfbba0a6d2da7596107e2d16bc26b0297ef3d7a2297fe8cd04d035`. Dit is geen gecontroleerde eindtekst van NTA 8800:2025+C1:2026.

De afzonderlijke Rust-diagnose combineert twee conceptregels voor iedere maand:

- Voor een collectieve warmtepompbron: `Q_HD,hp,in,bron = Q_H,gen,out × (1 − 1/COP_gi,mi)` volgens §9.6.8.1.1.2.3. Voor een individuele bron wordt deze collectieve leveringspost nul.
- Generator-elektriciteit: `E_H,gen,in = Q_H,gen,out / COP_gi,mi × f_prac − Q_HD,hp,in,bron × f_cor,bron,col` volgens 9.62, met `f_prac = 1` in dit concept.

De COP komt uit dezelfde Rust-tabeldiagnose als de overige forfaitaire invoer. Voor een individuele bron is de collectieve broncorrectie nul. Voor een collectieve bodembron is `f_cor,bron,col = 0,009 kWh_el/kWh_th`; voor collectief grondwater, oppervlaktewater of een bron van minstens 15 °C is dit `0,022 kWh_el/kWh_th`. De gebruiker moet het bronstelsel met een herleidbare bron expliciet kiezen. Onverenigbare bronklassen worden geweigerd. De grond-/grondwaterklasse met onbekend type of temperatuur kan niet als collectief bronstelsel worden gerekend zolang niet vaststaat welke factor van toepassing is.

De route vereist twaalf unieke maanden met niet-negatieve, eindige generatorwarmte `Q_H,gen,out` en een bronverwijzing. Ook de bronstelselclassificatie vereist een verwijzing. De collectieve bronwarmte wordt door Rust uit generatorwarmte en COP afgeleid; zij is geen vrije invoer. Onvolledige maanden, een ongeldige COP, negatieve of onmogelijke afgeleide bronwarmte en numerieke overloop geven geen gedeeltelijke elektriciteitsuitkomst. De **generatorwarmte wordt aangeleverd**. De onderliggende generatorverdeling uit §9.2.2.1.3 is niet geïmplementeerd.

Voorbeeld van de rekenstap: collectieve bron van 15–<20 °C, woningbouw, ontwerpaanvoer 35 °C, tabel-COP 4,8 en opgegeven generatorwarmte 1.000 kWh in een maand. De afgeleide collectieve bronwarmte is `1.000 × (1 − 1/4,8) = 791,666… kWh`. De ongecorrigeerde elektrische invoer is `1.000 / 4,8 = 208,333… kWh`; de collectieve broncorrectie is `791,666… × 0,022 = 17,416… kWh`; het conceptdeelresultaat is `190,916… kWh` voor die maand. Dit is een handvoorbeeld op aangeleverde generatorwarmte, geen onafhankelijke EDR-referentie-uitkomst.

HTTP: `POST /v1/nta8800/heat-pumps/forfait-monthly-draft/diagnose` met `{ "input": { ... } }`. MCP: `diagnose_forfait_heat_pump_monthly_draft` met dezelfde `input`. De response toont per maand aangeleverde generatorwarmte, afgeleide bronwarmte, ongecorrigeerde elektriciteit, broncorrectie en concept-elektriciteit. `generatorOutputDerived=false`, `auxiliariesIncluded=false`, `annualPerformanceAvailable=false`, `bengCalculationAvailable=false` en `finalEditionVerified=false` blijven expliciet; `collectiveSourceHeatDerived=true` alleen bij een geldige collectieve berekening. Hulpenergie uit §9.6.8.1, primaire energie, hernieuwbare fractie, BENG en energielabel worden hier niet berekend.

Voor normconforme vrijgave ontbreken de gecontroleerde eindtekst en correctiebladen, de upstream generatorverdeling, product-/bronbewijs, onafhankelijke actuele EDR-deeluitkomsten en externe attestering.
