# Koelreferentie via HTTP en MCP — runtimecontrole

Datum: 4 oktober 2026. Broncode en gebouwde binaries: `e2d2a69`. De binaryhashes staan in het [bouwdossier](nta8800-build-verificatie-2026-10-04-e2d2a69.md).

Een kopie van `training-data/nta8800-project-performance-synthetic.json` kreeg voor deze controle een synthetische compressiekoeler in `ntaCalculation.cooling`. Het referentiemanifest vroeg `coolingMonth/7/generatorColdKwh` (`kWh`) op met een opzettelijk afwijkende verwachting van 1.000.000.000 kWh en tolerantie nul. De input is niet als nieuwe golden of normreferentie opgeslagen.

De gebouwde HTTP-API draaide tijdelijk op `127.0.0.1:3018`. `POST /v1/nta8800/reference/compare` gaf HTTP 200, `compared_fail` en de berekende waarde 754,6801159198358 kWh. De gebouwde MCP-stdio-server accepteerde de `initialize`-handshake voor protocol `2025-06-18`; `tools/call` voor `compare_reference_case` gaf `isError=false` en exact dezelfde meetpost, werkelijke waarde, afwijking en manifestvingerafdruk. Beide antwoorden hielden `referenceVerified=false` en `attestStatus=unattested`. De tijdelijke processen zijn gestopt.

Deze controle bevestigt de gedeelde transportkoppeling voor één synthetische koeldetailpost. De verwachting is bewust onjuist; dit is geen onafhankelijke NTA 8800-uitkomst, visuele UI-test of externe attestering.
