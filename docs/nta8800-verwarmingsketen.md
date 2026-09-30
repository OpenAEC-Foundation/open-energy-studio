# NTA 8800 keten ruimteverwarming: behoefte → afgifte → distributie → opwekker

De Rust-modules `heating_emission` en `space_heating_chain` voeren per maand de warmtebehoefte uit hoofdstuk 7 ([maandbehoefte](nta8800-maandbehoefte.md)) door een afgiftesysteem, een distributieroute en één opwekker. Het resultaat is het energiegebruik per energiedrager. Status: `calculated_unverified`. `bengCalculationAvailable` blijft `false`.

## Bron

De afgifte- en distributieregels komen uit het [openbare consultatieconcept van hoofdstuk 9](https://www.internetconsultatie.nl/epg2026/document/14150), pagina's 13–15 en 16–17. Formule 9.16 staat daar als afbeelding en is visueel gelezen. Dit concept is geen gecontroleerde NTA 8800:2025+C1:2026.

## Afgifte (§9.3)

- 9.9: `Q_H;em;in = Q_H;em;out + Q_H;em;ls`; 9.10: `Q_H;em;out = Q_H;nd`.
- 9.11/9.12: `θ_H;int;inc = θ_H;set + Δθ_int;inc`. `Δθ_int;inc` is de som van:
  - tabel 9.2, systeem: radiatoren/convectoren 0,35; vloerverwarming 0,3; ventilatorgedreven −0,15; luchtverwarming 0,0; overig/onbekend 0,35;
  - tabel 9.3, waterzijdig inregelen: geen/onbekend 0,7; statisch 0,4; dynamisch 0,2; lokaal of luchtverwarming 0;
  - tabel 9.4, regeling: hoofdvertrek 2,5; centraal met naregeling 2,0; per ruimte 1,5; overig/onbekend 2,5.
- 9.16: bij `θ_H;set − θ_e;avg;mi > 0` geldt `Q_H;em;ls = Q_H;em;out · Min(Δθ_int;inc / (θ_H;int;inc − θ_e;avg;mi); 0,15)`, anders 0.

Bij radiatoren zonder inregeling en met een hoofdvertrekthermostaat (3,55 K) grijpt het plafond van 15% in vrijwel elke stookmaand. Luchtverwarming vereist `balancing = not_applicable`; een watersysteem mag die keuze niet hebben. Ventilatorgedreven afgifte wordt geweigerd, omdat de ventilatorenergie (9.21/9.22) nog niet is gemodelleerd.

## Distributie (§9.4)

- `heated_zone_only_space_heating`: volgens §9.4.1 wordt het verlies van leidingen in verwarmde ruimte verwaarloosd, mits het systeem uitsluitend voor ruimteverwarming dient. Het verlies is dan 0.
- `declared`: twaalf maandwaarden voor `Q_H;dis;ls` met bron. Ze tellen alleen mee in maanden met warmtebehoefte.

De volledige formule 9.26 (Ψ, leidinglengtes, temperaturen, bedrijfstijd 9.32a, forfait 15% in onverwarmde ruimte) is nog niet geïmplementeerd.

## Opwekker (§9.6)

Bij één opwekker geeft tabel 9.1 `β = 1`, dus 100% dekking van `Q_H;gen;out = Q_H;em;in + Q_H;dis;ls` (knooppuntverliezen weggelaten).

- `gas_boiler`: bestaande [ketelmodule](nta8800-ketel-forfait-concept.md) (tabel 9.25, 9.61, hulpenergie 9.85).
- `heat_pump_forfait`: bestaande [forfaitaire warmtepompmodule](nta8800-warmtepomp-maandinvoer-concept.md) (tabellen 9.27/9.29, 9.62, collectieve-broncorrectie). Hulpenergie van de warmtepomp ontbreekt.

- `hybrid_heat_pump`: warmtepomp met individuele bijverwarmingsketel. De bestaande [generatorverdeling](nta8800-generatorverdeling-concept.md) splitst de knooppuntvraag van de keten volgens tabel 9.1/9.23 (alleen nieuwbouw, installatievermogens). Daarna volgen de forfaitaire COP-route voor het warmtepompdeel en tabel 9.25/9.61/9.85 voor het ketelaandeel. Gemeten hulpenergie van de warmtepomp (9.85–9.88) is optioneel en telt op bij de hulpenergie van de ketel. Weigering volgt, net als in de onderliggende module, bij een ontwerpaanvoer boven 55 °C (bijlage Q), bij productgebonden afschakelgrenzen en bij een afwijkende ketelrol. `heatPumpOutputKwh` per maand geeft het warmtepompdeel; alleen daarover telt omgevingswarmte als hernieuwbaar (5.31).

## Niet meegenomen (`omittedTerms`)

Knooppuntverliezen en -winst (9.2.3), terugwinbare systeemverliezen (9.2.5), ventilatorenergie van de afgifte (9.21), hulpenergie van een enkele warmtepomp en bronpomp/-ventilator, meer dan twee opwekkers, productgebonden hybride schakeling en tapwaterprioriteit.

## Aanroep

HTTP: `POST /v1/nta8800/heating/space-heating-chain/calculate` met `{ "input": { demand, emission, distribution, generator } }`. MCP en desktop: `calculate_space_heating_chain`. TS: `calculateSpaceHeatingChainWithRust`. Synthetisch voorbeeld: `training-data/nta8800-space-heating-chain-synthetic.json`.

## Toetsing

Er zijn zeven unittests: tabelsommen, de ratio- en plafondtak, ketel- en warmtepompketen, gedeclareerde distributie, afwijzingen en foutprefixen. Een servicetest dekt de HTTP-route. Er is geen onafhankelijk referentiegeval.
