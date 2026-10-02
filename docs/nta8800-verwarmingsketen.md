# NTA 8800 keten ruimteverwarming: behoefte → afgifte → distributie → opwekker

De Rust-modules `heating_emission`, `heating_distribution` en `space_heating_chain` voeren per maand de warmtebehoefte uit hoofdstuk 7 ([maandbehoefte](nta8800-maandbehoefte.md)) door een afgiftesysteem, een distributiesysteem, het knooppunt en één opwekker. Het resultaat is het energiegebruik per energiedrager plus de hulpenergie. Status: `calculated_unverified`. `bengCalculationAvailable` blijft `false`.

## Bron

Hoofdstuk 9 is gecontroleerd tegen NTA 8800:2025+C1:2026. Afgifte (pagina's 296–297) en de opwekkerregels zijn gelijk aan het consultatieconcept; alleen de oude formule 9.16 heet nu 9.12a. De distributie (pagina's 302–321), het knooppunt (290–292), de terugwinbare verliezen (293–294) en de hulpenergie (359–365) zijn rechtstreeks uit de doeleditie overgenomen. Formules die in de tekstlaag onleesbaar zijn (9.26, 9.33–9.35, 9.45, 9.50, 9.51) zijn visueel van de pagina gelezen. Er staat geen normtekst in de repository.

## Afgifte (§9.3)

- 9.9: `Q_H;em;in = Q_H;em;out + Q_H;em;ls`; 9.10: `Q_H;em;out = Q_H;nd`.
- 9.11/9.12: `θ_H;int;inc = θ_H;set + Δθ_int;inc`. `Δθ_int;inc` is de som van:
  - tabel 9.2, systeem: radiatoren/convectoren 0,35; vloerverwarming 0,3; ventilatorgedreven −0,15; luchtverwarming 0,0; overig/onbekend 0,35;
  - tabel 9.3, waterzijdig inregelen: geen/onbekend 0,7; statisch 0,4; dynamisch 0,2; lokaal of luchtverwarming 0;
  - tabel 9.4, regeling: hoofdvertrek 2,5; centraal met naregeling 2,0; per ruimte 1,5; overig/onbekend 2,5.
- 9.12a: bij `θ_H;set − θ_e;avg;mi > 0` geldt `Q_H;em;ls = Q_H;em;out · Min(Δθ_int;inc / (θ_H;int;inc − θ_e;avg;mi); 0,15)`, anders 0.

Bij radiatoren zonder inregeling en met een hoofdvertrekthermostaat (3,55 K) grijpt het plafond van 15% in vrijwel elke stookmaand. Luchtverwarming vereist `balancing = not_applicable`; een watersysteem mag die keuze niet hebben. Ventilatorgedreven afgifte wordt geweigerd, omdat de ventilatorenergie (9.21/9.22) nog niet is gemodelleerd.

## Distributie (§9.4)

Per rekenzone kies je één route:

- `heated_zone_only_space_heating`: volgens §9.4.1 wordt het verlies van leidingen in verwarmde ruimte verwaarloosd, mits het systeem uitsluitend voor ruimteverwarming dient. Het verlies is dan 0. Leidingen in onverwarmde ruimten zijn met deze route niet afgedekt.
- `declared`: twaalf maandwaarden voor `Q_H;dis;ls` met bron. Ze tellen in elke maand mee: de doeleditie zet het verlies buiten het stookseizoen niet op nul, want de bedrijfstijd van tabel 9.15 is ook in de zomer groter dan 0.
- `calculated`: formule 9.26 met de systeemgegevens in `distributionSystem`.

### Systeemgegevens (`distributionSystem`)

Deze gegevens gelden voor alle zones van de keten:

- ontwerptemperatuurklasse (tabel 9.14; zonder opgave 90/70), individuele of collectieve installatie, gebruiksfunctie voor `f_H;red` (tabel 7.15) en aantal aangesloten bouwlagen;
- Ψ: forfaitair uit tabel 9.16 (geïsoleerd per periode, ongeïsoleerd of onbekend, met de collectieve rijen voor leidingen die ook tapwater voeren), of berekend met 9.33 (geïsoleerd in lucht), 9.34 (ingestort) of 9.35 (ongeïsoleerd, `h_a` = 8);
- leidinglengte: werkelijk of 9.36 (`L_si = 0,64·A_g`), lengte in onverwarmde ruimte werkelijk of 15% van `L_si`, en de toeslag voor kleppen en beugels volgens 9.27a/9.27b;
- temperatuur in de onverwarmde ruimte per maand, of 13 °C;
- vlaggen voor leidingen die ook tapwater voeren en voor een collectieve installatie met afleverset;
- een collectief buffervat en de pompgegevens.

`collectiveConnection.connectedUsableAreaM2` is `A_g;gebouw;H`. Daarmee volgt `f_gebouw;si;H` = zone-oppervlak / `A_g;gebouw;H`. Zonder dit blok rekent de keten het gebouw als geheel (`f_gebouw` = 1).

### Rekenregels

- Stookgrens (9.28 stap 2–5): kleinste-kwadratenlijn door de maanden met minstens 10% van de maximale behoefte, snijpunt met de temperatuuras, afgerond, niet boven het setpoint, begrensd op 8–16 °C. De ventilatietermen van 9.28/9.29 (`E_V;eldf`, `E_V;elvv`, luchtterm) geef je per zone op in `heatingLimitExtraKwh`; zonder opgave zijn ze 0. Lukt de lijn niet (stijgend of minder dan twee punten), dan volgt `heating_limit_undetermined`.
- Bedrijfstijd (9.32a/9.32b): `t_H,op = t_H(tabel 9.15) · f_H;red · f_H;red;pmp;op`, met `f_H;red = 1 − f_day − f_wknd` (7.62/7.63, tabel 7.15) en de factor van tabel "9.X": 0,10 voor een woonfunctie met individuele installatie, anders 1,0. Bij leidingen die ook tapwater voeren of bij een afleverset is `t_H,op` de hele maand.
- Watertemperaturen: 9.31/9.32 met `θ_H,e;ontw` = −10 °C, `θ_mean` volgens 9.30. Bij een afleverset geldt `θ_mean ≥ 65 °C`.
- Verlies (9.26): `Ψ_zi·(θ_mean − θ_amb,zi)·L_zi·t/1000 + Ψ_j·(θ_mean − θ_amb,j)·L_j·t/1000·A_g;zi/ΣA_g`, maal `f_gebouw`. `L_zi` = 0 als de leidingen alleen voor ruimteverwarming dienen (§9.4.1). Voor `θ_amb,zi` gebruikt de keten het verwarmingssetpoint; de `θ_int;op;H` van 7.9.6 is nog niet gekoppeld.
- Terugwinbaar (9.38): het zone-deel van 9.26 met `f_H;dis;rbl` = 1. Verlies in onverwarmde ruimte is niet terugwinbaar.

### Pomp (9.41–9.51)

`pump.method`:

- `included_in_generator_auxiliary`: de pomp zit in de hulpenergie van 9.85. Dit mag alleen bij een individuele gasketel of een individuele elektrische warmtepomp.
- `none_on_site`: er staat geen pomp op het perceel, bijvoorbeeld bij externe warmte zonder warmtewisselaar (§9.4.4).
- `calculated`:
  - `Δp = 1,4 · 0,10 · L_max + Δp_add` (9.44), met `L_max` werkelijk of volgens 9.37 (`35 + 6n + 0,13·A_g/n`);
  - `Δp_add` volgens tabel 9.21: afgifte 2 of 4,5 kPa (de hoogste over de zones), warmtemeter 10 kPa, en de opwekkerweerstand uit `Δθ_h;a;ontw` en `Δθ_h;g;ontw` (warmtepomp 10, externe warmte = afgifte, overig 20);
  - debiet volgens 9.45 in de maand met de hoogste behoefte voor de stookgrens, of opgegeven;
  - `P_hydr = max(Δp·V/3600; 0,01)` (9.43);
  - `ε = f_e·EEI/0,25`, met `f_e` uit 9.47, 9.48/9.49 of 9.50 (b = 2) en EEI 0,23/0,25 of opgegeven;
  - `f_HB` = 1,00 bij statisch of dynamisch ingeregelde watersystemen, anders 1,15 (tabel 9.19);
  - `W = P_hydr·t_H,op·f_HB·ε·f_gebouw` (9.41, 9.51), verdeeld over de zones naar oppervlak.
  - Een kwart is terugwinbaar naar de zone (9.39). Driekwart gaat naar het medium (9.40) en verlaagt `Q_H;dis;in` (9.24, per zone begrensd op 0).

Een pomp opgeven is verplicht zodra een watervoerend afgiftesysteem aanwezig is en de opwekker niet onder 9.85 valt (collectieve ketel, collectieve warmtepomp, externe warmte, elektrisch, biomassa): code `distribution_pump_input_required`.

### Knooppunt (9.2.3)

Een collectief buffervat geeft een knooppuntverlies volgens 13.58 met `f_sto;bac;acc` = 1 en `H = S_sto;ls/45`. `S` komt van het label, uit tabel 13.9 met de labelklasse, of standaard uit klasse C (vanaf 2018) of G (eerder). De settemperatuur is `θ_H,a;ontw` bij een systeem met constante temperatuur, anders de hoogste `θ_H,in` van de zones. De omgeving is het setpoint of 13 °C. Het buffervat mag alleen bij een berekende Ψ; de forfaitaire Ψ van tabel 9.16 bevat het buffervatverlies al (opmerking 2). Zonthermische knooppuntwinst, LBK, bevochtiging, boosterwarmtepompen en afleversets als knooppuntvraag zijn niet gemodelleerd.

### Terugwinbare verliezen (9.2.5)

`zoneRecoverableLosses` geeft per zone `Q_H;ls;rbl` = 9.38 + 9.39. Afgifteverliezen en opwekkerverliezen zijn in alle forfaitaire routes 0 (pagina's 297, 323, 331, 341 en 345). De koppeling terug naar 7.2.1 hoort bij de behoefteberekening. Die koppeling is hier nog niet gelegd: het veld is de interface. Voor BENG 1 moet deze term 0 blijven (§5.4.2).

## Opwekker (§9.6)

Bij één opwekker geeft tabel 9.1 `β = 1`, dus 100% dekking van `Q_H;gen;out` = Σ zones `Q_H;dis;in` + knooppuntverlies.

- `gas_boiler`: [ketelmodule](nta8800-ketel-forfait-concept.md) (tabel 9.25, 9.61). Een individuele ketel gebruikt hulpenergie volgens 9.85 (gas-forfait). Een collectieve ketel vereist `auxiliary` voor 9.91/9.92: aantal toestellen (10 W stand-by), nominaal vermogen (1 W/kW branderbedrijf) en bron.
- `heat_pump_forfait`: [forfaitaire warmtepompmodule](nta8800-warmtepomp-maandinvoer-concept.md) (tabellen 9.27/9.29, 9.62).
  - Hulpenergie van een individueel toestel: gemeten (9.85–9.88 via `auxiliaryMeasurements`) of forfaitair 9.85 met A = 43,8 kWh, B = 0,132 kW, C = 0,7 en Bnom = 3 kW over de maandelijkse elektriciteit.
  - Een collectieve warmtepomp gebruikt 9.91 met alleen de stand-by-term.
  - Een ontwerpaanvoer boven 55 °C wordt geweigerd (`heat_pump_above55_requires_annex_q`). Volgens §9.6.3 (pagina 331) moet dan bijlage Q, ook voor een warmtepomp zonder ketel. Tabel 9.27 heeft wel kolommen tot 70 °C; die tegenstrijdigheid staat in het verificatiedossier.
- `hybrid_heat_pump`: warmtepomp met individuele bijverwarmingsketel, gesplitst met de [generatorverdeling](nta8800-generatorverdeling-concept.md) volgens tabel 9.1/9.23 (alleen nieuwbouw). Hulpenergie: ketel 9.85, warmtepomp gemeten of 9.85-forfait.
- `external_heat`: 9.84 met `η = 1,0` en `f_prac = 1`, drager `dh`. De hulpenergie loopt via 9.91 (§9.6.7.2). Of een afleverset als "toestel" voor de 10 W telt, staat in `electricallyConnectedDevices`; dit is een interpretatie (zie het verificatiedossier). Een kwaliteitsverklaring (bijlage P) wordt geweigerd.
- `electric_resistance`: COP 1,0 (tabel 9.27). Hulpenergie volgens 9.91: 10 W per toestel of paneel.
- `biomass`: 9.64 met tabel 9.30, alleen klasse bmB (bijlage R).
  - Een kachel telt alleen als hij de enige verwarming is in de ruimten die hij bedient (§9.6.5). Dat bevestig je met `soleHeatingInServedRooms`; zonder bevestiging volgt een invoergat, en bij `false` volgt weigering.
  - Hulpenergie volgens 9.91: 10 W stand-by, en 10 W/kW bij automatische brandstoftoevoer. Centrale ketels gelden als automatisch gestookt.

- `gas_boiler` met `auxiliaryMeasurements` (alleen individueel): de constanten A, B en C van 9.85 worden uit componentmetingen berekend volgens bijlage O en 9.86–9.90 (p. 359–361, 923–931). Dat omvat de rechte lijnen voor ventilator en modulerende pomp (O.2/O.3, kleinste kwadraten) en de gemiddelde aantijd uit het belastingsverloop (O.1.5: 120 s bij vollast, 1 800 s voor bivalente toestellen).
- `product_boiler`: ketel met productwaarden volgens bijlage M (methode 1 van 9.6.2.2 en 9.6.5.2, p. 870–879). Brandstof gas, olie of hout; hout vraagt bijlage R (bmB).
  - Deellastverhouding M.25 over de bedrijfstijd van tabel 9.15 bij de stookgrens (zonder de reductiefactoren van 9.32a).
  - Temperatuurgecorrigeerde rendementen M.7/M.8/M.10 met tabel M.2/M.4, of M.13/M.14 uit een extra test.
  - Verliezen M.9/M.11/M.12, lineair geïnterpoleerd (M.4/M.5), bij `ϑ_Hc;mn = ϑ_H,out` van 9.32.
  - Hulpenergie M.21–M.23, waarvan 0,75 naar het medium gaat (M.18). Invoer M.1 met de regelfactor van tabel M.7.
  - De retourtemperatuur gebruikt de ontwerpklasse van `distributionSystem` of `designTemperatureClass`. De pomp zit niet in de ketelhulpenergie, dus een watergedragen afgifte vraagt `distributionSystem`.
  - Terugwinbare verliezen naar de ruimte (M.16/M.19) staan in `generatorRecoverableLossKwh`. Ze worden niet teruggekoppeld naar de behoefte.
- `local_heater`: lokale verwarmer, luchtverwarmer, straler of kachel volgens bijlage N (p. 880–922).
  - Aan/uit-toestellen via N.3 met de iteratie N.36/N.37.
  - Hoog-laag en modulerend via N.4: aan/uit op minimumlast (N.60/N.61) of modulatie (N.65–N.72).
  - Kachels via N.5, zonder mantelverlies; een watergedragen aansluiting geeft `waterSideHeat` alleen als rapportage.
  - Ontbrekende productwaarden komen uit tabellen N.20 en N.22–N.30. Waar geen standaardwaarde bestaat, volgt `local_heater_value_required`.
  - De bedrijfstijd is tabel 9.15 bij de stookgrens van de zonebehoefte. De invoer wordt gedeeld door f_prac 0,95. Onvoldoende vermogen geeft `local_heater_capacity_insufficient`.
- `forfait_heater`: de "overige systemen" van tabel 9.25. Lokale gas- of olieverwarming met afvoer 0,65, zonder afvoer 0,10, en direct gestookte gasluchtverwarmers van conventioneel 0,75 tot HR-107 0,95. Hulpenergie volgens 9.91 met 10 W stand-by en 1 W/kW branderbedrijf.

Olie-invoer (`oilKwh`) telt in de energieprestatie als drager `oil`, gewogen met f_BACS.

`auxiliaryElectricityKwh` per maand is de som van opwekker- en distributiehulpenergie (9.6). Het distributiedeel staat ook apart in `distributionAuxiliaryElectricityKwh`.

## Niet meegenomen (`omittedTerms`)

- Zonthermische knooppuntwinst en de overige knooppuntvragen (9.2.3).
- Terugkoppeling van de terugwinbare verliezen naar de behoefte.
- Ventilatorenergie van de afgifte (9.21).
- Meer dan twee opwekkers, productgebonden hybride schakeling en tapwaterprioriteit.
- `θ_int;op;H` (7.9.6) als omgevingstemperatuur van de leidingen in de zone.

## Aanroep

HTTP: `POST /v1/nta8800/heating/space-heating-chain/calculate` met `{ "input": { demand, emission, distribution, generator, distributionSystem?, collectiveConnection? } }`. MCP en desktop: `calculate_space_heating_chain`. TS: `calculateSpaceHeatingChainWithRust`. Synthetische voorbeelden: `training-data/nta8800-space-heating-chain-synthetic.json` (individuele ketel) en `training-data/nta8800-space-heating-chain-collective-synthetic.json` (collectieve ketel met berekende distributie en pomp).

## Toetsing

Unittests rekenen met de hand na: tabel 9.15, `f_H;red`, de stookgrensfit, 9.31/9.32, tabel 9.16 en 9.33–9.35, 9.37–9.50, tabel 13.9 met het buffervat, 9.26/9.38–9.40/9.51 in de keten, 9.85-forfait, 9.91/9.92 met en zonder deel-gebouw, en de nieuwe afwijzingen. Er is geen onafhankelijk referentiegeval.
