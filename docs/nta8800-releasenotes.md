# NTA 8800-kern — releasenotes

Wijzigingen die de uitkomst of de status van bestaande, opgeslagen projecten veranderen. Normverwijzingen gaan naar NTA 8800:2025+C1:2026, met paragraaf-, formule- en paginanummers.

## 5 oktober 2026 — reviewcorrecties woningopname (ISSO 82.1)

### Opnames die nu `incomplete` worden

- **Individuele gaswarmtepomp tot en met 25 kW.** De GWP-rijen van NTA-tabel 9.27 (p. 334) gelden alleen voor een collectieve gebouwinstallatie. Tabel 9.29 geldt voor collectieve installaties en voor meer dan 25 kW. Een individuele gasmotor- of gasabsorptiewarmtepomp tot en met 25 kW heeft dus geen forfaitaire rij en geeft nu `gas_heat_pump_individual_no_forfait_row`. Tot nu toe rekende de opname met de GWP-rijen van tabel 9.27.
- **Collectieve warmtepompbron zonder bewijs.** Een aangevinkte collectieve bron met een lege verwijzing gaf stilzwijgend een individuele bron, zonder tabel V.3 en zonder 9.62. Volgens ISSO 82.1 p. 111 blijkt een collectieve bron uit facturen of ontwerpgegevens. De opname meldt nu `collective_source_reference_required`.
- **Systeem E zonder WTW.** Bij §11.3.6 (p. 145) heeft het decentrale deel altijd WTW. Daarom kan de standaardwaarde "geen WTW" van tabel 11.9 hier niet gelden. Een gecombineerd systeem met een onbekende of ontbrekende wisselaar geeft `combined_requires_heat_recovery`.
- **Lege g-waarde bij zonwerend glas.** Een lege `solarControl.gValue` gaf een leesfout van de hele opname. Nu geeft hij `solar_control_g_invalid` op dat veld.

### Opnames met een andere uitkomst

- **Bypass bij de opname afwezig.** De jaarregel van tabel 11.12 (p. 151–152) geldt alleen als de bypass of het bypasspercentage onbekend is. Een unit uit 2010 of later met `bypassPresent: false` kreeg ten onrechte 100 % bypass en krijgt nu 0 %.
- **Oppervlaktewater bij een collectieve installatie.** Volgens ISSO 82.1 p. 111 is oppervlaktewater een invoerkeuze bij een collectieve installatie. Dat geldt ook zonder collectieve bron. Zo'n installatie rekent nu met de rij oppervlaktewater van tabel 9.29 in plaats van met de rij bodem. Een gaswarmtepomp tot en met 25 kW in een collectieve installatie neemt de rij grondwater van tabel 9.27, want die tabel heeft geen rij oppervlaktewater.

### Nu toegestaan

- **Passieve koeling bij systeem E naast natuurlijke ventilatie.** Volgens p. 152 kan passieve koeling voorkomen bij de systemen B tot en met E. Het decentrale deel moet dan wel een bypass hebben.

### Formulier

- **Verborgen antwoorden worden gewist.** Kies je een andere bron of een ander toestel, dan wist het formulier de antwoorden die daarbij niet meer zichtbaar zijn: collectieve bron, grondwatersysteem, brontemperatuur, kwaliteitsverklaring en de brandstof van een stoomketel. Zo leveren ze geen `collective_source_water_based_only` of `local_heater_fuel_contradiction` meer op.
## 5 oktober 2026 — basisopname utiliteitsgebouwen volgens ISSO 75.1 (7e druk)

### Opnames die nu `incomplete` worden

- **Splitsing in rekenzones (afb. 6.6 met tabel 6.4, p. 53–54).** Na het samenvoegen van p. 39–40 stopt de opname met `calculation_zone_split_required` wanneer de setpoints van de overgebleven functies meer dan 4 K verschillen, of wanneer bij ventilatietype A, B, C of E de ventilatiecapaciteit meer dan een factor 4 verschilt. Een voorbeeld is onderwijs naast een sportfunctie van meer dan 25 %. De uitzondering voor verblijfsgebieden in open verbinding staat in `openlyConnectedResidenceAreas`.
- **Gasmotor-koelmachine zonder elektrisch vermogen** (`gas_engine_power_required`, tabel 10.2).
- **Meer koudeopwekkers zonder vermogen** (`cooling_generator_capacity_required`, §10.3.2 met NTA 10.49).
- **Directe expansie in de LBK** zonder LBK of met "niet aangesloten" op de koelbatterij.

### Opnames met een andere uitkomst

- **f_BACS uit de opgenomen vermogens (tabel 7.3, p. 62–63).** Zonder `bacs.systemPowerKw` beslist nu het opgenomen vermogen van verwarming en koeling. Een ketel van 350 kW in een gebouw kleiner dan 2.500 m² krijgt zo f_BACS 1,05 (eerder 1,0). Zijn alle systemen bekend en ten hoogste 290 kW, dan is het 1,0, ook boven 2.500 m².
- **Dynamisch ingeregelde koeldistributie** krijgt f_HB 1,0 (NTA-tabel 10.11) in plaats van 1,15. Dat geldt ook voor de koeling in de woningopname.
- **Collectieve verwarming.** De distributie leest nu de antwoorden van tabel 9.12 (leidingisolatie, isolatiejaar, appendages) en het eenpijpssysteem; eerder waren de leidingen altijd ongeïsoleerd.
- **Opgenomen koudeopwekkerpunten.** Opgegeven appendage-isolatie, koudemeters en leidinglengten gaan nu naar de kern.
- **Bronvermelding.** `derive_utility_input` geeft zelf al de ISSO 75.1-pagina's.

### Nieuwe opties

- Fossiele brandstof op het perceel, oppervlak van sport- en zwemzalen, ruimte met zwembad.
- Koeling: gasmotor-koelmachine, meer opwekkers met prioriteit, directe expansie in ruimte of LBK, koudemeters, appendages, leidinglengten.
- Ventilatie: LUKA D en geen kanaal, "koude laden met LBK", decentrale WTW, isolatie en lengte van de buitenaansluiting, constant volumeregeling, gedeeltelijke bypass in procenten, geïnstalleerde capaciteit, systeem E en roosters met verwarmingslint.

## 5 oktober 2026 — basisopname woningen volgens ISSO 82.1 (7e druk met erratum)

### Opnames die nu `incomplete` worden

- **Gesloten of verlaagd plafond in de woningopname.** `construction.closedOrSuspendedCeiling` komt uit ISSO 75.1. ISSO 82.1 tabel 7.4 (p. 62) kent alleen `lighterCeiling`. De woningopname meldt nu `closed_ceiling_not_in_dwelling_survey`.
- **Warmtepomp met een opgegeven klasse boven 70 °C.** Volgens tabel 9.9 en erratum §4 is dan een gecontroleerde verklaring nodig: `heating.heatPumpAbove70Declaration`, anders `heat_pump_above_70_requires_declaration`.
- **Bewijsstukken bij nieuwe opties.** Ventilatiesturing zonder bewijsstuk geeft `ventilation_controls_evidence_required`. Zonwerend glas zonder bron geeft `solar_control_evidence_required`.

### Opnames met een andere uitkomst

- **Bypass bij een onbekend bypassaandeel (tabel 11.12, p. 151–152).** Het fabricagejaar van de WTW-unit gaat nu voor het bouwjaar. Een unit van vóór 2010 in een woning van 2010 of later krijgt dus geen 100 %-bypass meer, maar 70 % (bypass aanwezig) of 0 %.
- **Ketel bij klasse 70/50.** De gemiddelde ontwerptemperatuur is nu 65 °C in plaats van 60 °C, gelijk aan de distributieklasse 70/60. De uitkomst verandert niet, omdat alleen de grens van 50 °C telt.

### Nieuwe opties

- Ventilatiesturing volgens de tabellen 11.4–11.6 zonder `declaredVariant`.
- Gecombineerd systeem E.
- Roosters met verwarmingslint.
- Woningpositie "dak + vloer".
- Zonwerend glas of folie met een g-waarde uit het product.
- Centrale of decentrale WTW.
## 4 oktober 2026 — verwarmingsopties in de basisopname (ISSO 82.1 hoofdstuk 9)

### Nieuwe, aanvullende invoer

- **Opwekkers (tabel 9.3).** `boilerType: oil` (olieketel, conventioneel), `local_fired` (lokale gasverwarming, olieverwarming of stoomketel, met of zonder afvoer) en `gas_air_heater` (direct gestookte luchtverwarmers). De kern rekent ze met NTA-tabel 9.25.
- **Warmtepompen (tabel 9.6).** `drive` (elektrisch, gasmotor, gasabsorptie), de bronnen `heat_pump_panel` en `high_temperature`, `groundwaterSystem` (doublet of recirculatie), `collectiveSourceReference`, `sourceTemperatureC` en `sourceQualityDeclarationReference`.
- **Distributie (§9.4.2, tabel 9.12).** `distributionType` (tweepijps, eenpijps met aantal afgiftetoestellen, gerenoveerd eenpijps) en `pipeInsulation` (geïsoleerd, isolatiejaar, appendages en beugels).
- **Kern.** `pump.onePipeEmitterCount` telt de weerstand per afgiftetoestel van een eenpijpskring (tabel 9.21, p. 317). `forfait_heater` mag zonder `nominalPowerKw`; 9.92 rekent dan met de bovengrens t_on = t_mi.

### Opnames met een andere uitkomst

- **Grondwaterwarmtepomp zonder brontemperatuur.** Deze rekent nu met de rij bodem (NTA p. 335). Eerder werd de rij grondwater gekozen, wat de kern zonder temperatuurbewijs afwees.
- **Collectieve warmtepomp in de woningopname.** Deze rekent nu met tabel 9.29 en krijgt hulpenergie volgens 9.91. Eerder volgde `table_scope_capacity_mismatch` en ontbrak de hulpenergie.
- **Warmtepomp boven 25 kW in de woningopname.** Deze rekent nu met tabel 9.29.

## 4 oktober 2026 — koelmethode 1 en bijlage Q

### Projecten die nu `invalid` worden

- **Vijfde meetpunt NEN-EN 14825 (10.63, p. 408–409).** Een vijfde punt moet de deellast van punt C en de condensorintredetemperatuur van punt A hebben (tolerantie 0,5 % of 0,5 K), anders `cooling_en14825_fifth_point_conditions`. De testpunten staan in de volgorde A, B, C, D.
- **Zonder vijfde punt (10.64).** De benadering met Δϑ_corr = 0 klopt alleen als de verdamperuittrede bij A en C gelijk is. Verschillen ze meer dan 0,5 K, dan volgt `cooling_en14825_fifth_point_required`.
- **Methode 1 alleen voor modulerende opwekkers (§10.5.4, p. 401).** Een minimumvermogen gelijk aan of boven het nominale vermogen geeft `cooling_en14825_modulating_required`. Een minimumvermogen boven het nominale gaf eerder `cooling_performance_invalid`.
- **Lucht/luchtwarmtepomp volgens bijlage Q.** Met waterafgifte (radiatoren, vloerverwarming, ventilatorradiatoren) of hydraulische distributiegegevens volgt `annex_q_air_air_hydronic_chain`.

### Projecten met een andere uitkomst

- **Bijlage Q met bijverwarming.** Dekt de warmtepomp elke temperatuurklasse van tabel Q.6, dan geldt nu F_H;gen = 1, ook als er bijverwarming is opgegeven. Eerder kreeg de bijverwarming door de afronding van tabel Q.6 een restaandeel van enkele honderdsten procent.

### Nieuwe waarschuwing

- **Deellast boven 100 % (10.56/10.58).** Komt f_C;PL in een temperatuurklasse boven 100 %, dan extrapoleert de kubische functie van 10.63 buiten het meetbereik. De kern houdt de letterlijke uitkomst aan en meldt `cooling_part_load_above_full_load`. Een te klein toestel kan zo gunstiger uitkomen.

## 4 oktober 2026 — dynamische ramen in de projectroute (bijlage A)

- **Nieuw, aanvullend veld** `ntaCalculation.dynamicWindows`: bijlage A (p. 766–770) per buitenraam, methode A of B, met de correctiefactoren van stap 2. Opgeslagen projecten zonder dit veld houden dezelfde uitkomst.
- In het NTA-formulier is dit invoerbaar onder "Dynamische ramen (bijlage A)". Eerder kon het alleen via de kerninvoer.
- **Nieuwe gap** `window_dynamic_and_shading_exclusive`: een dynamisch raam samen met beweegbare zonwering (7.42) telt de zonwering dubbel (§A.2, p. 767). Neem de zonwering op in de toestanden.
- **Lege waarden** in bijlage A (g, U, wegingen, correctiefactoren) geven `dynamic_value_missing` op hun eigen pad. Eerder blokkeerde één leeg veld het hele NTA-blok.
- τ_vis en τ_sol staan niet meer in het formulier: hoofdstuk 14 gebruikt ze niet (14.38, 14.41). Opgeslagen waarden blijven bewaard.

## 4 oktober 2026 — validatieregels en herberekeningsbevindingen

### Projecten die nu `incomplete` worden

- **Verticale leidingen (§7.3.3).**
  - Ontbreekt `verticalPipes` (project of zone), dan geldt dat als "onbekend" en volgt de gap `vertical_pipes_unknown`. Vul de leidingen in, of `[]` voor "geen".
  - Een meerzonig project zonder lijsten per zone krijgt dezelfde gap.
  - Staat bij een zone `[]` terwijl op projectniveau leidingen zijn opgegeven, dan volgt `vertical_pipes_conflicting`. Eerder vielen de projectleidingen dan stil weg.
- **Koudebrugmethode (§8.2.1, §8.3.3.1).**
  - Een forfaitaire vloerrand naast ψ-waarden geeft `thermal_bridge_methods_mixed`.
  - Forfaitair met een onverwarmde ruimte geeft `forfait_thermal_bridges_unheated_space_unsupported`.
- **Verwarmde kelder in de forfaitaire route (8.38).**
  - De kern vult ΔU_for voor de kelderwanden nu zelf in met de waarde van 8.3.
  - Een opgegeven waarde die daarvan afwijkt geeft `basement_forfait_delta_u_conflict`.

- **Ventilatie (11.60/11.61, p. 468).** Een terugregel-x gunstiger dan de standaard (recirculatie boven 20 %, debietregeling onder 80 %) zonder `flowReduction.evidenceReference` geeft `flow_reduction_evidence_required`.
- **Gemeten luchtdoorlatendheid.** Een q_v10 van 0 of lager geeft `infiltration_invalid`.

### Projecten met een andere uitkomst

- **Forfaitaire koudebruggen.**
  - H_D krijgt ΔU_for (8.2/8.3) op alle elementen, glas inbegrepen.
  - De zonwinst van dichte delen (7.33) en de uitstraling naar de hemel (7.39) houden U_c van 8.2.2, dus zonder ΔU_for.
- **Gemeten opslagverlies.**
  - H_sto;ls wordt naar boven afgerond volgens bijlage X (p. 568).
  - Nieuw is de route `measured_standby` (13.60).
  - Ongeldige waarden worden afgewezen, ook bij zonneboilervaten.
- **Koelmachine (10.73, tabel 10.8).**
  - De benodigde uittredetemperatuur is nu de aanvoertemperatuur min Δϑ_int;inc.
  - Bij verdamping in de ruimte is dat ϑ_C;int;inc (10.10).
  - Het EER en het elektriciteitsgebruik veranderen daardoor.
- **EN 16147-correcties (13.153b/13.153c).** De invoer `smartControlFactor`, `maxTestTemperatureC` en `designSetTemperatureC` is optioneel; zonder deze invoer verandert er niets.
- **BENG 1.** Wordt alleen getoond uit de run met vast ventilatiesysteem C1 (§5.4).
- **TOjuli bij utiliteit.** Er geldt geen Bbl-grenswaarde ("niet van toepassing").
- **Voorbeeldprojecten.** Tapwater wordt nu berekend in plaats van opgegeven. Zie `nta8800-voorbeeldproject-smoketest-2026-10-03.md`.

### Nieuwe waarschuwingen (de berekening loopt door)

- `declared_hot_water_efficiency_above_one`
- `declared_ventilation_below_required_flow`: alleen bij woningen, met de laagste f_ctrl·f_sys van tabel 11.5.
- `bacs_factor_without_capacity_evidence`
- `utility_open_ceiling_requires_evidence`
- `cooling_emission_loss_singular`
- `hot_water_circulation_defaults_low_efficiency`
- `lighting_large_office_group_without_office`: `largeOfficeGroup` in een zone zonder kantoorfunctie (§14.5.1, p. 664). Dit geeft F_o;D = 1, de minst gunstige waarde, en blokkeert dus niet.

Het rekenrapport toont zowel de projectmeldingen als de meldingen van de kern.
