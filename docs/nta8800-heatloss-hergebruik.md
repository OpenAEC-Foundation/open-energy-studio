# Hergebruikonderzoek Open Heatloss Studio → Open Energy Studio

**Controle:** 29 september 2026. Open Energy Studio blijft het product voor NTA 8800; de zusterrepository is hier onderzocht als kandidaat-bron voor Rust-rekenmodules en historische testinvoer. Een geslaagde test in die repository verleent geen NTA-conformiteit of BRL 9501-attest aan Open Energy Studio.

## Feitelijke dekking

De zusterrepository `/home/aia04/Documenten/GitHub/open-heatloss-studio` bevat aparte Rust-crates voor onder meer `nta8800-model`, `tables`, `geometry`, `transmission`, `ventilation`, `demand`, `heating`, `dhw`, `cooling`, `pv` en `ep`. De workspace vermeldt MIT als licentie; vóór codeovername moet de concrete herkomst en licentietekst per bestand worden vastgelegd. Een vaste `path`-dependency naar de lokale zustercheckout is ongeschikt voor een zelfstandig bouwbare Open Energy Studio-release.

Een lokale `cargo test --locked` op `nta8800-demand`, `nta8800-dhw`, `nta8800-ep`, `nta8800-heating` en `nta8800-tables` slaagde: respectievelijk 84, 70, 67, 62 en 99 unit-tests; de doctests slaagden eveneens met één genegeerd voorbeeld. Dit bewijst dat die crates op deze checkout compileerbaar zijn en hun eigen assertions halen. Het is geen onafhankelijke vergelijking met actuele EDR-resultaten.

| Kandidaat | Bruikbaar als vertrekpunt | Controle vóór integratie |
| --- | --- | --- |
| `nta8800-model` en `geometry` | getypeerde zones, maanden, eenheden en meetfuncties | adapter van `.oes`, Ag/Als-regels, bronparagrafen, afronding en actuele referentiegevallen |
| `nta8800-tables` | klimaat-, materiaal- en andere tabellen met bronverwijzingen in code | volledige NTA 8800:2025+C1:2026 tegen elke over te nemen waarde controleren; distributie- en gebruiksrechten vastleggen |
| `nta8800-transmission`, `ventilation`, `demand` | meer routes dan de huidige OES-diagnose | bekende V1-vereenvoudigingen, onder meer aangeleverde reductiefactoren, ontbrekende transmissiecorrecties en schaduw-/koelroutes, oplossen en vergelijken met onafhankelijke deelresultaten |
| `nta8800-heating`, `dhw`, `cooling` | ketenmodellen en typed energiedragers | warmtepompen gebruiken in verwarming/tapwater een door gebruiker opgegeven **SCOP**; type-specifieke bijlage-Q/W-routes, hybride omschakeling, hulpenergie en prestatiecurven ontbreken. Niet als geverifieerde warmtepompberekening overnemen |
| `nta8800-ep` | indicatoren en labelmodel | eigen V1-documentatie noemt 2023-factoren, beperkte energiedragers en ontbrekende 2026-opslagcorrectie; label- en BENG-keten pas na exacte editie, volledige input en EDR-toets koppelen |

## Historische EDR-testen

De zusterrepository heeft zes EP-W-fixtures uit ISSO 54, versie 2.0 (12 mei 2022). De energie-uitkomsten in hun `expected.json` zijn `null` en gemarkeerd als geblokkeerd op de ontbrekende Excelbijlage. De bijbehorende EDR-resultaatassertions zijn genegeerd. Deze set levert dus **geen** actuele energie-goldens voor NTA 8800:2025+C1:2026.

De [publieke oorspronkelijke ISSO 54-testbeschrijving](https://documenten.isso.nl/s/Ym8-N6LW2khaIUJL0XEgRaFsscBPV5dn/ISSO%2054%20-%2012-05-2022.pdf) noemt op gedrukte pagina 5 voor EP-W001 wel onafhankelijk vastgelegde **invoer**: Ag 96 m², Ao 247,2 m², volume 259,2 m³ en ramen 24 m². Open Energy Studio bewaart hiervoor een kleine, opnieuw ingevoerde geometriefixture in `training-data/edr-2022-epw001-geometry.json`. Een Rust-test vergelijkt de OES-invoersommen met deze gepubliceerde totalen en bevestigt dat geen BENG-resultaat of buiten-transmissiegetal wordt afgeleid. Deze test controleert boekhouding van aangeleverde vlakken; hij verifieert geen NTA-meetmethode of prestatieuitkomst.

## Integratieregel

Voor elke over te nemen module: maak eerst een zelfstandige OES-crate of vendored module met vastgelegde herkomst en rechten; voeg vervolgens een adapter toe die de ontbrekende gegevens expliciet afwijst, versieer de gebruikte normtabellen en toets deelposten aan onafhankelijke verwachte waarden. Pas daarna mag die route BENG, TO-juli of label voeden. Een schijnbaar complete berekening op basis van generieke SCOP of V1-forfaits mag de bestaande `calculationAvailable=false` niet opheffen.
