# NTA 8800 dekking hoofdstukken 5–8 en bijlagen A–I en AA (review 77)

Stand 15 oktober 2026, doeluitgave NTA 8800:2025+C1:2026. Deel 1 van 3 van de dekkingsronde van review 77. Per genummerde formule en per tabel van hoofdstuk 5, 6, 7 en 8 en de bijlagen A tot en met I en AA staat hieronder of de kern (`crates/nta8800-core`) haar uitvoert en of een test haar raakt. Het document bevat alleen nummers, paragraafnummers en bestandsnamen, geen normtekst.

**Werkwijze.** De formule- en tabelnummers zijn uit de tekst van de norm gehaald (regels die eindigen op een formulenummer, kopjes "Tabel x.y"). Per nummer is in de kern gezocht naar de verwijzing in het commentaar en is nagegaan welke test de route raakt; daarbij tellen de onafhankelijke herberekeningen van reviews 63–76 (zie [verificatiestatus](nta8800-verificatiestatus.md)). Waar een uitgevoerde bepaling met een rekenwaarde nog geen onafhankelijke test had, is een eigen berekening vanaf de normpagina gemaakt en als test opgenomen in `crates/nta8800-core/tests/coverage_ch5_8_independent.rs` (15 tests, tolerantie 10⁻⁶ relatief, alle gelijk).

**Status.**

- **GT** — uitgevoerd en onafhankelijk getest (eigen berekening vanaf de normpagina in een test, of in een eerdere review vergeleken).
- **GE** — uitgevoerd, alleen met een eigen eenheidstest van de kern.
- **NG** — niet uitgevoerd in de kern.
- **NVT** — niet van toepassing: informatief (opmerking), classificatie of invoerbeslissing van de opnemer, of een berekening die bewust buiten de kern blijft.

Formule 5.59 komt in de tekst niet voor (5.58 wordt gevolgd door 5.60); 8.7 evenmin. Bijlagen F, G en H hebben alleen tabellen. Tabellen I.3, I.13 en I.14 bestaan niet in deze uitgave.

**Tellingen.** Formules (329): GT 303, GE 6, NG 5, NVT 15. Tabellen (79): GT 67, GE 1, NG 0, NVT 11.

## Formules

| Nummer | Paragraaf | Status | Kernlocatie (`crates/nta8800-core/src`) | Test (`crates/nta8800-core/tests`) |
|---|---|---|---|---|
| Formule 5.1 | 5.3.1.1 | GT | building_performance.rs, indicators_draft.rs | coverage_ch5_8_independent.rs (beng_one_from_the_fixed_c1_run); review 64 |
| Formule 5.2 | 5.3.1.2 | GT | building_performance.rs, indicators_draft.rs | indicators_independent.rs |
| Formule 5.3 | 5.3.1.3 | GT | building_performance.rs, indicators_draft.rs | indicators_independent.rs |
| Formule 5.3a | 5.3.2 | GT | building_performance.rs, indicators_draft.rs | coverage_ch5_8_independent.rs (need_indicators); indicators_independent.rs |
| Formule 5.3b | 5.3.2 | GT | building_performance.rs, monthly_demand.rs | coverage_ch5_8_independent.rs (need_indicators) |
| Formule 5.3c | 5.3.2 | GT | building_performance.rs, monthly_demand.rs | coverage_ch5_8_independent.rs (need_indicators) |
| Formule 5.3d | 5.3.3 | GT | building_performance.rs, monthly_demand.rs | coverage_ch5_8_independent.rs (need_indicators) |
| Formule 5.3e | 5.3.3 | GT | building_performance.rs, monthly_demand.rs | coverage_ch5_8_independent.rs (need_indicators) |
| Formule 5.3f | 5.3.3 | GT | building_performance.rs, monthly_demand.rs | coverage_ch5_8_independent.rs (need_indicators) |
| Formule 5.3g | 5.3.4 | GT | building_performance.rs, monthly_demand.rs | coverage_ch5_8_independent.rs (need_indicators) |
| Formule 5.3h | 5.3.5.1 | GT | building_performance.rs, final_energy_draft.rs | indicators_independent.rs |
| Formule 5.3i | 5.3.5.2 | GT | building_performance.rs, final_energy_draft.rs | indicators_independent.rs |
| Formule 5.4 | 5.4.2 | GT | building_performance.rs, monthly_demand.rs (vaste C1-run, q_L van 5.4.2) | coverage_ch5_8_independent.rs (beng_one_from_the_fixed_c1_run, fixed_lighting_flux_of_the_c1_run); review 64 |
| Formule 5.5 | 5.4.2 | GT | building_performance.rs, monthly_demand.rs (vaste C1-run, q_L van 5.4.2) | coverage_ch5_8_independent.rs (beng_one_from_the_fixed_c1_run, fixed_lighting_flux_of_the_c1_run); review 64 |
| Formule 5.6 | 5.4.2 | GT | building_performance.rs, monthly_demand.rs (vaste C1-run, q_L van 5.4.2) | coverage_ch5_8_independent.rs (beng_one_from_the_fixed_c1_run, fixed_lighting_flux_of_the_c1_run); review 64 |
| Formule 5.7 | 5.4.2 | GT | building_performance.rs, monthly_demand.rs (vaste C1-run, q_L van 5.4.2) | coverage_ch5_8_independent.rs (beng_one_from_the_fixed_c1_run, fixed_lighting_flux_of_the_c1_run); review 64 |
| Formule 5.8 | 5.4.2 | GT | building_performance.rs, monthly_demand.rs (vaste C1-run, q_L van 5.4.2) | coverage_ch5_8_independent.rs (beng_one_from_the_fixed_c1_run, fixed_lighting_flux_of_the_c1_run); review 64 |
| Formule 5.9 | 5.5.2 | GT | building_performance.rs | indicators_independent.rs; installations_independent.rs |
| Formule 5.10 | 5.5.2 | GT | building_performance.rs | indicators_independent.rs; installations_independent.rs |
| Formule 5.11 | 5.5.2 | GT | building_performance.rs | indicators_independent.rs; installations_independent.rs |
| Formule 5.12 | 5.5.2 | GT | building_performance.rs | indicators_independent.rs; installations_independent.rs |
| Formule 5.13 | 5.5.2 | GT | building_performance.rs | indicators_independent.rs; installations_independent.rs |
| Formule 5.14 | 5.5.2 | GT | building_performance.rs (E_nEPus;el = 0 volgens 5.27) | indicators_independent.rs |
| Formule 5.14a | 5.5.2 | GT | building_performance.rs | indicators_independent.rs (indicators_storage_correction) |
| Formule 5.14b | 5.5.2 | GT | building_performance.rs | indicators_independent.rs (indicators_storage_correction) |
| Formule 5.15 | 5.5.2 | GT | building_performance.rs | indicators_independent.rs |
| Formule 5.16 | 5.5.2 | GT | building_performance.rs | indicators_independent.rs |
| Formule 5.17a | 5.5.2 | GT | building_performance.rs | indicators_independent.rs |
| Formule 5.17b | 5.5.2 | GT | building_performance.rs | indicators_independent.rs |
| Formule 5.18a | 5.5.2 | GT | building_performance.rs | indicators_independent.rs |
| Formule 5.18b | 5.5.2 | GT | building_performance.rs | indicators_independent.rs |
| Formule 5.19a | 5.5.2 | GT | building_performance.rs | indicators_independent.rs |
| Formule 5.19b | 5.5.2 | GT | building_performance.rs | indicators_independent.rs |
| Formule 5.20 | 5.5.3 | GT | building_performance.rs, epus_draft.rs | indicators_independent.rs |
| Formule 5.20a | 5.5.3 | GT | building_performance.rs, epus_draft.rs | indicators_independent.rs |
| Formule 5.20b | 5.5.3 | GT | building_performance.rs, epus_draft.rs | indicators_independent.rs |
| Formule 5.21 | 5.5.3 | GT | building_performance.rs, epus_draft.rs | indicators_independent.rs |
| Formule 5.22 | 5.5.4.1 | GT | building_performance.rs | installations_independent.rs; indicators_independent.rs |
| Formule 5.23 | 5.5.4.1 | GT | building_performance.rs | installations_independent.rs; indicators_independent.rs |
| Formule 5.24 | 5.5.4.1 | GT | building_performance.rs | installations_independent.rs; indicators_independent.rs |
| Formule 5.25 | 5.5.4.1 | GT | building_performance.rs | installations_independent.rs; indicators_independent.rs |
| Formule 5.26 | 5.5.4.1 | GT | building_performance.rs | installations_independent.rs; indicators_independent.rs |
| Formule 5.27 | 5.5.4.2 | GT | building_performance.rs (0 W/m² voor de indicatoren) | indicators_independent.rs |
| Formule 5.28 | 5.6.2.0 | GT | building_performance.rs | coverage_ch5_8_independent.rs; indicators_independent.rs |
| Formule 5.29 | 5.6.2.0 | GT | building_performance.rs | coverage_ch5_8_independent.rs; indicators_independent.rs |
| Formule 5.30 | 5.6.2.1 | GT | building_performance.rs, space_heating_chain.rs | coverage_ch5_8_independent.rs; indicators_independent.rs |
| Formule 5.31 | 5.6.2.1 | GT | building_performance.rs | coverage_ch5_8_independent.rs (heat_pump_space_heating_renewable); installations_independent.rs |
| Formule 5.32 | 5.6.2.1 | GT | building_performance.rs | coverage_ch5_8_independent.rs (heat_pump_partly_exhaust_air_renewable) |
| Formule 5.33 | 5.6.2.1 | GT | building_performance.rs | indicators_independent.rs (collective_heat_pump_source_from_20_c) |
| Formule 5.34 | 5.6.2.2 | GT | building_performance.rs, space_cooling.rs | coverage_ch5_8_independent.rs (free_cooling_renewable, external_cold_declared_renewable) |
| Formule 5.35 | 5.6.2.3 | GT | building_performance.rs, domestic_hot_water.rs | coverage_ch5_8_independent.rs; installations_independent.rs |
| Formule 5.36 | 5.6.2.3 | GT | domestic_hot_water.rs | coverage_ch5_8_independent.rs (hot_water_heat_pump_renewable; lezing NEN-vraag 17) |
| Formule 5.37 | 5.6.2.3 | GT | domestic_hot_water.rs | coverage_ch5_8_independent.rs (hot_water_heat_pump_renewable; lezing NEN-vraag 17) |
| Formule 5.38 | 5.6.2.3 | GT | domestic_hot_water.rs | coverage_ch5_8_independent.rs (hot_water_heat_pump_renewable) |
| Formule 5.39 | 5.6.2.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.39a | 5.6.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.39b | 5.6.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.39c | 5.6.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.39d | 5.6.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.39e | 5.6.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.39f | 5.6.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.39g | 5.6.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.39h | 5.6.4 | GT | building_performance.rs (RenewableByCarrier) | coverage_ch5_8_independent.rs |
| Formule 5.40 | 5.7.2 | GT | tojuli.rs | envelope_independent.rs; ground_independent.rs |
| Formule 5.41 | 5.7.2 | GT | tojuli.rs | envelope_independent.rs; ground_independent.rs |
| Formule 5.41a | 5.7.2 | GE | tojuli.rs, space_cooling.rs (Q_C;HP) | eenheidstest tojuli.rs |
| Formule 5.41b | 5.7.2 | GE | tojuli.rs, space_cooling.rs (Q_C;HP) | eenheidstest tojuli.rs |
| Formule 5.41c | 5.7.2 | GE | tojuli.rs, space_cooling.rs (Q_C;HP) | eenheidstest tojuli.rs |
| Formule 5.42 | 5.8.3.1 | GT | annex_p.rs | annex_np_independent.rs; indicators_independent.rs |
| Formule 5.43 | 5.8.3.1 | GT | annex_p.rs | annex_np_independent.rs; indicators_independent.rs |
| Formule 5.44 | 5.8.3.1 | GT | annex_p.rs | annex_np_independent.rs; indicators_independent.rs |
| Formule 5.45 | 5.8.3.1 | GT | annex_p.rs | annex_np_independent.rs; indicators_independent.rs |
| Formule 5.46 | 5.8.3.1 | GT | annex_p.rs | annex_np_independent.rs; indicators_independent.rs |
| Formule 5.47 | 5.8.3.1 | GT | annex_p.rs | annex_np_independent.rs; indicators_independent.rs |
| Formule 5.48 | 5.8.3.1 | GT | annex_p.rs | annex_np_independent.rs; indicators_independent.rs |
| Formule 5.49 | 5.8.3.2 | GT | annex_p.rs | annex_np_independent.rs; indicators_independent.rs |
| Formule 5.50 | 5.8.3.3 | GT | annex_p.rs | annex_np_independent.rs |
| Formule 5.51 | 5.8.3.3 | GT | annex_p.rs | annex_np_independent.rs |
| Formule 5.52 | 5.8.3.3 | GT | annex_p.rs | annex_np_independent.rs |
| Formule 5.53 | 5.8.3.3 | GT | annex_p.rs | annex_np_independent.rs |
| Formule 5.54 | 5.8.3.3 | GT | annex_p.rs | annex_np_independent.rs |
| Formule 5.55 | 5.8.3.3 | GT | annex_p.rs | annex_np_independent.rs |
| Formule 5.56 | 5.8.3.3 | GT | annex_p.rs | annex_np_independent.rs |
| Formule 5.57 | 5.9.2 | GT | building_performance.rs, final_energy_draft.rs | indicators_independent.rs |
| Formule 5.58 | 5.9.2 | GT | building_performance.rs, final_energy_draft.rs | indicators_independent.rs |
| Formule 5.60 | 5.9.3 | GT | building_performance.rs, final_energy_draft.rs | indicators_independent.rs |
| Formule 6.1 | 6.6.5 | GT | domestic_hot_water.rs (A_g per woning bij tapwater) | installations_independent.rs; review 66 |
| Formule 6.2a | 6.6.6 | GT | monthly_demand.rs, project_performance.rs | solar_independent.rs |
| Formule 6.2b | 6.6.6 | GT | monthly_demand.rs, project_performance.rs | solar_independent.rs |
| Formule 6.3 | 6.7.2 | GT | project_performance.rs, opname/envelope.rs | indicators_independent.rs (loss_area_weighting) |
| Formule 6.4 | 6.8.2 | NVT | - | A_r wordt in de NTA niet verder gebruikt |
| Formule 7.1 | 7.2.1 | GT | monthly_demand.rs | solar_independent.rs; indicators_independent.rs; review 63 |
| Formule 7.2 | 7.2.1 | GT | monthly_demand.rs | solar_independent.rs; indicators_independent.rs; review 63 |
| Formule 7.3 | 7.2.1 | GT | monthly_demand.rs | solar_independent.rs; indicators_independent.rs; review 63 |
| Formule 7.4 | 7.2.1 | GT | monthly_demand.rs | solar_independent.rs; indicators_independent.rs; review 63 |
| Formule 7.5 | 7.2.1 | GT | monthly_demand.rs | solar_independent.rs; indicators_independent.rs; review 63 |
| Formule 7.6 | 7.2.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; review 63 |
| Formule 7.7 | 7.2.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; review 63 |
| Formule 7.8 | 7.2.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; review 63 |
| Formule 7.9 | 7.2.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; review 63 |
| Formule 7.10 | 7.2.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; envelope_independent.rs |
| Formule 7.11 | 7.2.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; envelope_independent.rs |
| Formule 7.12 | 7.2.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; envelope_independent.rs |
| Formule 7.13 | 7.2.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; envelope_independent.rs |
| Formule 7.14 | 7.3.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; envelope_independent.rs |
| Formule 7.15 | 7.3.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; envelope_independent.rs |
| Formule 7.16 | 7.3.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; envelope_independent.rs |
| Formule 7.17 | 7.3.3 | GT | monthly_demand.rs, opname/utility.rs | opname_defaults_independent.rs; review 63 |
| Formule 7.17a | 7.3.3 | GT | monthly_demand.rs, opname/utility.rs | opname_defaults_independent.rs; review 63 |
| Formule 7.18 | 7.4.2 | GT | monthly_demand.rs, ventilation.rs | ventilation_independent.rs; review 64 |
| Formule 7.19 | 7.4.3 | GT | monthly_demand.rs, ventilation.rs | ventilation_independent.rs; review 64 |
| Formule 7.20 | 7.4.4 | GT | monthly_demand.rs, ventilation.rs | ventilation_independent.rs; review 64 |
| Formule 7.21 | 7.5.2.1 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.22 | 7.5.2.1 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.23 | 7.5.2.1 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.24 | 7.5.2.1 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.25 | 7.5.2.2 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.26 | 7.5.3.1 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.27 | 7.5.3.2 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.28 | 7.5.3.3 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.29 | 7.5.3.4 | GT | monthly_demand.rs, lighting.rs | solar_independent.rs |
| Formule 7.30a | 7.6.2 | GT | monthly_demand.rs | solar_independent.rs; envelope_independent.rs |
| Formule 7.30b | 7.6.2 | GT | monthly_demand.rs | solar_independent.rs; envelope_independent.rs |
| Formule 7.31 | 7.6.3 | GT | monthly_demand.rs | solar_independent.rs |
| Formule 7.32 | 7.6.3 | GT | monthly_demand.rs | solar_independent.rs |
| Formule 7.33 | 7.6.3 | GT | monthly_demand.rs | solar_independent.rs |
| Formule 7.34 | 7.6.4 | GT | monthly_demand.rs | envelope_independent.rs (serres) |
| Formule 7.35 | 7.6.4 | GT | monthly_demand.rs | envelope_independent.rs (serres) |
| Formule 7.35a | 7.6.4 | GT | monthly_demand.rs | envelope_independent.rs (serres) |
| Formule 7.36 | 7.6.4 | GT | monthly_demand.rs | envelope_independent.rs (serres) |
| Formule 7.37 | 7.6.4 | GT | monthly_demand.rs | envelope_independent.rs (serres) |
| Formule 7.38 | 7.6.4 | GT | monthly_demand.rs | envelope_independent.rs (serres) |
| Formule 7.39 | 7.6.5 | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Formule 7.40 | 7.6.6.1.2 | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Formule 7.41 | 7.6.6.1.3.1 | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Formule 7.41a | 7.6.6.1.3.2 | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Formule 7.41b | 5.7.1 | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Formule 7.42 | 7.6.6.1.4 | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Formule 7.43 | 7.6.6.1.4 | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Formule 7.44 | 7.6.6.2 | NG | monthly_demand.rs neemt F_fr als invoer (zone per raam, project één waarde) | methode A niet berekend; methode B 0,25 als invoer |
| Formule 7.45 | 7.7 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.46 | 7.8.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.47 | 7.8.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.48 | 7.8.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.49 | 7.8.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.50 | 7.8.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.51 | 7.8.2 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.52 | 7.8.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.53 | 7.8.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.54 | 7.8.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.55 | 7.8.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.56 | 7.8.3 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.57 | 7.8.4 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.58 | 7.8.4 | GT | monthly_demand.rs | solar_independent.rs; ground_independent.rs; coverage_ch5_8_independent.rs (need_indicators) |
| Formule 7.59 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.60 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.61 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.62 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.63 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.64 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.65 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.66 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.67 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.68 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.69 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.70 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.71 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.72 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.73 | 7.9.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.74 | 7.9.3 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.75 | 7.9.3 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.76 | 7.9.4.1 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.77 | 7.9.4.1 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.78 | 7.9.4.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.79 | 7.9.4.2 | GT | monthly_demand.rs | solar_independent.rs; review 63 |
| Formule 7.80 | 7.9.6 | GT | monthly_demand.rs (uitvoer, als gedrukt) | coverage_ch5_8_independent.rs (cooling_operative_temperature) |
| Formule 7.81 | 7.9.6 | GT | monthly_demand.rs (uitvoer, als gedrukt) | coverage_ch5_8_independent.rs (cooling_operative_temperature) |
| Formule 7.82 | 7.9.7 | GE | space_heating_chain.rs, domestic_hot_water.rs | eenheidstests domestic_hot_water.rs |
| Formule 8.1 | 8.2.1 | GT | envelope_elements.rs, constructions.rs | envelope_independent.rs; coverage_ch5_8_independent.rs (tabel 8.1) |
| Formule 8.2 | 8.2.1 | GT | envelope_elements.rs, constructions.rs | envelope_independent.rs; coverage_ch5_8_independent.rs (tabel 8.1) |
| Formule 8.3 | 8.2.1 | GT | envelope_elements.rs, constructions.rs | envelope_independent.rs; coverage_ch5_8_independent.rs (tabel 8.1) |
| Formule 8.4 | 8.2.2.2.1 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.5 | 8.2.2.2.1 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.6 | 8.2.2.2.1 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.8 | 8.2.2.2.2.1 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.9 | 8.2.2.2.2.2 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.10 | 8.2.2.2.2.3 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.11 | 8.2.2.2.2.3 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.12 | 8.2.2.2.2.3 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.13 | 8.2.2.2.2.4 | GT | constructions.rs, envelope_elements.rs | envelope_independent.rs |
| Formule 8.14 | 8.2.2.3.1 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.15 | 8.2.2.3.1 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.16 | 8.2.2.3.1 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.17 | 8.2.2.3.1 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.18 | 8.2.2.3.2 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.19 | 8.2.2.3.3 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.20 | 8.2.2.3.3 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.21 | 8.2.2.3.3 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.22 | 8.2.2.3.4 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.23 | 8.2.2.3.4 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.24 | 8.2.2.3.5.1 | GT | window_u.rs | envelope_independent.rs; constructions_independent.rs |
| Formule 8.25 | 8.2.2.3.5.3 | GT | window_u.rs | constructions_independent.rs |
| Formule 8.26 | 8.2.3.1 | GT | window_u.rs | constructions_independent.rs |
| Formule 8.27 | 8.2.3.2 | GT | window_u.rs | constructions_independent.rs |
| Formule 8.28 | 8.2.3.3 | GT | window_u.rs | constructions_independent.rs |
| Formule 8.29 | 8.2.4.2 | GT | window_u.rs | constructions_independent.rs |
| Formule 8.30 | 8.3.2.2 | GT | ground.rs | ground_independent.rs |
| Formule 8.31 | 8.3.2.2 | GT | ground.rs | ground_independent.rs |
| Formule 8.32 | 8.3.2.3 | GT | ground.rs | ground_independent.rs |
| Formule 8.33 | 8.3.2.3 | GT | ground.rs | ground_independent.rs |
| Formule 8.34 | 8.3.2.3 | GT | ground.rs | ground_independent.rs |
| Formule 8.35 | 8.3.2.4.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.36 | 8.3.3.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.37 | 8.3.3.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.38 | 8.3.3.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.39 | 8.3.3.2 | GT | ground.rs | ground_independent.rs |
| Formule 8.40 | 8.3.4.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.41 | 8.3.4.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.42 | 8.3.4.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.43 | 8.3.4.2 | GT | ground.rs | ground_independent.rs |
| Formule 8.44 | 8.3.5.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.45 | 8.3.5.3 | GT | ground.rs | ground_independent.rs |
| Formule 8.46 | 8.3.5.4.1 | GT | ground.rs | ground_independent.rs |
| Formule 8.47 | 8.3.5.4.2 | GT | ground.rs | ground_independent.rs |
| Formule 8.48 | 8.3.5.4.3 | GT | ground.rs | ground_independent.rs |
| Formule 8.49 | 8.3.5.4.3 | GT | ground.rs | ground_independent.rs |
| Formule 8.50 | 8.3.6 | NG | - | ψ_gr alleen opgegeven of uit bijlage I |
| Formule 8.51 | 8.3.6 | NG | - | ψ_gr alleen opgegeven of uit bijlage I |
| Formule 8.52 | 8.4.1 | GT | unheated_transmission.rs, lib.rs | envelope_independent.rs |
| Formule 8.53 | 8.4.1 | GT | unheated_transmission.rs, lib.rs | envelope_independent.rs |
| Formule 8.54 | 8.4.1 | GT | unheated_transmission.rs, lib.rs | envelope_independent.rs |
| Formule 8.55 | 8.4.1 | GT | unheated_transmission.rs, lib.rs | envelope_independent.rs |
| Formule 8.56 | 8.4.3 | GT | unheated_transmission.rs, lib.rs | envelope_independent.rs |
| Formule 8.57 | 8.4.3 | GT | unheated_transmission.rs, lib.rs | envelope_independent.rs |
| Formule 8.58 | 8.4.3 | GT | unheated_transmission.rs, lib.rs | envelope_independent.rs |
| Formule 8.59 | 8.4.3 | GT | unheated_transmission.rs, lib.rs | envelope_independent.rs |
| Formule 8.60 | 8.5 | NVT | monthly_demand.rs (H_A = 0) | opmerking; H_A = 0 getest in envelope_independent.rs |
| Formule 8.61 | 8.5 | NVT | monthly_demand.rs (H_A = 0) | opmerking; H_A = 0 getest in envelope_independent.rs |
| Formule 8.62 | 8.6.1.1 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.63 | 8.6.1.2 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.64 | 8.6.2.3.1 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.65 | 8.6.3.1 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.66 | 8.6.3.1 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.67 | 8.6.3.2 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.68 | 8.6.5 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.69 | 8.6.6.1 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.70 | 8.6.6.1 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.71 | 8.6.6.1 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.72 | 8.6.6.2 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule 8.73 | 8.6.6.2 | NVT | envelope_elements.rs (L_C als invoer, 8.5) | numerieke berekening buiten de kern; constructions_independent.rs |
| Formule AA.1 | AA.2.2.1 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.2 | AA.2.2.1 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.3a | AA.2.2.1 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.3b | AA.2.2.1 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.4 | AA.2.2.2 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.5 | AA.2.2.3 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.6a | AA.2.2.4 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.6b | AA.2.2.4 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.7 | AA.2.2.5 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.8 | AA.2.3.1 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.9 | AA.2.3.2 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.10 | AA.3.2.3 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.11 | AA.3.2.3 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.12 | AA.3.2.4 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule AA.13 | AA.3.2.4 | GT | annex_aa.rs, tojuli.rs | solar_independent.rs |
| Formule A.1 | A.3 | GT | annex_a.rs (gewichten als invoer) | solar_independent.rs |
| Formule A.2 | A.3 | GT | annex_a.rs (gewichten als invoer) | solar_independent.rs |
| Formule A.3 | A.3 | GE | annex_a.rs (alleen uitvoer) | eenheidstest annex_a.rs |
| Formule A.4 | A.3 | GE | annex_a.rs (alleen uitvoer) | eenheidstest annex_a.rs |
| Formule B.1 | B.1 | GT | annex_b.rs | solar_independent.rs (thermal_mass_annex_b) |
| Formule C.1 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.2 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.3 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.4 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.5 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.6 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.7 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.8 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.9 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.10 | C.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.11 | C.1 | GT | constructions.rs (via C.15 als laag) | envelope_independent.rs |
| Formule C.12 | C.3 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.13 | C.3.5.1 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.14 | C.3.5.2 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.15 | C.4 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.16 | C.4 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.17 | C.4 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.18 | C.4 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.19 | C.4 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.20 | C.4 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.21 | C.4 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule C.22 | C.4 | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Formule D.1 | D.1 | GT | ground.rs | ground_independent.rs |
| Formule D.2 | D.1 | GT | ground.rs | ground_independent.rs |
| Formule D.3 | D.1 | GT | ground.rs | ground_independent.rs |
| Formule D.4 | D.2 | GT | ground.rs | ground_independent.rs |
| Formule D.5 | D.2.2.2 | GT | ground.rs | ground_independent.rs |
| Formule D.6 | D.2.2.2 | GT | ground.rs | ground_independent.rs |
| Formule D.7 | D.2.2.2 | GT | ground.rs | ground_independent.rs |
| Formule D.8 | D.2.2.2 | GT | ground.rs | ground_independent.rs |
| Formule D.9 | D.2.2.2 | GT | ground.rs | ground_independent.rs |
| Formule D.10 | D.2.2.3 | GT | ground.rs | ground_independent.rs |
| Formule D.11 | D.2.2.3 | GT | ground.rs | ground_independent.rs |
| Formule D.12 | D.2.2.3 | GT | ground.rs | ground_independent.rs |
| Formule D.13 | D.2.2.4 | GT | ground.rs | ground_independent.rs |
| Formule D.14 | D.2.2.4 | GT | ground.rs | ground_independent.rs |
| Formule D.15 | D.2.2.5 | GT | ground.rs | ground_independent.rs |
| Formule D.16 | D.2.2.5 | GT | ground.rs | ground_independent.rs |
| Formule D.17 | D.2.2.5 | NG | ground.rs (één h en één z als invoer) | gewogen gemiddelde over wanddelen niet berekend |
| Formule D.18 | D.2.2.5 | NG | ground.rs (één h en één z als invoer) | gewogen gemiddelde over wanddelen niet berekend |
| Formule E.1 | E.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.2 | E.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.3 | E.2.1.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.4 | E.2.1.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.5 | E.2.1.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.6 | E.2.1.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.7 | E.2.1.2 | GT | materials.rs | constructions_independent.rs |
| Formule E.8 | E.2.1.3.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.9 | E.2.1.3.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.10 | E.2.1.4.1 | GT | materials.rs | constructions_independent.rs |
| Formule E.11 | E.2.1.5 | GT | materials.rs | constructions_independent.rs |
| Formule I.1 | I.2.1.1 | GT | forfait_envelope.rs, envelope_elements.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Formule I.2 | I.2.1.4 | GT | forfait_envelope.rs, envelope_elements.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Formule I.3 | I.2.1.4 | GT | forfait_envelope.rs, envelope_elements.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Formule I.4 | I.2.2.4.2 | GT | forfait_envelope.rs, envelope_elements.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Formule I.5 | I.2.2.4.2 | GT | forfait_envelope.rs, envelope_elements.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Formule I.6 | I.2.2.4.2 | GT | forfait_envelope.rs, envelope_elements.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Formule I.7 | I.2.2.4.2 | GT | forfait_envelope.rs, envelope_elements.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Formule I.8 | I.2.2.4.2 | GT | forfait_envelope.rs, envelope_elements.rs | constructions_independent.rs; opname_defaults_independent.rs |

## Tabellen

| Nummer | | Status | Kernlocatie (`crates/nta8800-core/src`) | Test (`crates/nta8800-core/tests`) |
|---|---|---|---|---|
| Tabel 5.1 |  | NVT | project_performance.rs (gebruiksfunctie/Bbl-functie als invoer) | indeling, geen rekenwaarde |
| Tabel 5.2 |  | GT | building_performance.rs, norm_versions | indicators_independent.rs; coverage_ch5_8_independent.rs |
| Tabel 5.3 |  | GT | building_performance.rs, norm_versions | indicators_independent.rs; coverage_ch5_8_independent.rs |
| Tabel 5.4 |  | GT | building_performance.rs, norm_versions | indicators_independent.rs; coverage_ch5_8_independent.rs |
| Tabel 5.5 |  | GT | annex_p.rs | annex_np_independent.rs |
| Tabel 5.6 |  | GT | annex_p.rs | annex_np_independent.rs |
| Tabel 5.7 |  | GT | label_class.rs | indicators_independent.rs |
| Tabel 6.1 |  | GE | building_performance.rs | eenheidstest building_performance.rs |
| Tabel 6.2 |  | NVT | - | terminologie en informatieve tabellen |
| Tabel 6.3 |  | NVT | - | terminologie en informatieve tabellen |
| Tabel 6.4 |  | NVT | - | terminologie en informatieve tabellen |
| Tabel 6.5 |  | NVT | opname/utility.rs | schematiseringsbeslissing (invoer) |
| Tabel 7.1 |  | GT | monthly_demand.rs | opname_defaults_independent.rs; review 63 |
| Tabel 7.2 |  | GT | monthly_demand.rs | solar_independent.rs |
| Tabel 7.3 |  | GT | monthly_demand.rs | solar_independent.rs |
| Tabel 7.4 |  | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Tabel 7.4a |  | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Tabel 7.4b |  | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Tabel 7.5 |  | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Tabel 7.6 |  | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Tabel 7.7 |  | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Tabel 7.8 |  | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Tabel 7.9 |  | GT | monthly_demand.rs, solar_shading.rs | solar_independent.rs |
| Tabel 7.10 |  | GT | monthly_demand.rs | solar_independent.rs |
| Tabel 7.11 |  | NVT | monthly_demand.rs (MassClass als invoer) | classificatie door de opnemer |
| Tabel 7.12 |  | NVT | monthly_demand.rs (MassClass als invoer) | classificatie door de opnemer |
| Tabel 7.13 |  | GT | monthly_demand.rs | solar_independent.rs |
| Tabel 7.14 |  | GT | monthly_demand.rs | solar_independent.rs |
| Tabel 7.15 |  | GT | monthly_demand.rs | solar_independent.rs |
| Tabel 8.1 |  | GT | constructions.rs (8.3) | coverage_ch5_8_independent.rs (table_8_1_examples_of_formula_8_3) |
| Tabel 8.2 |  | GT | constructions.rs, window_u.rs | envelope_independent.rs |
| Tabel 8.3 |  | GT | constructions.rs, window_u.rs | envelope_independent.rs |
| Tabel 8.4 |  | GT | constructions.rs, window_u.rs | envelope_independent.rs |
| Tabel 8.5 |  | GT | window_u.rs | constructions_independent.rs |
| Tabel 8.6 |  | NVT | - | modelleringsregels numerieke berekening |
| Tabel AA.1 |  | GT | annex_aa.rs | solar_independent.rs |
| Tabel AA.2 |  | GT | annex_aa.rs | solar_independent.rs |
| Tabel AA.3 |  | GT | annex_aa.rs | solar_independent.rs |
| Tabel C.1 |  | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Tabel C.2 |  | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Tabel C.3 |  | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Tabel C.4 |  | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Tabel C.5 |  | GT | constructions.rs | envelope_independent.rs; constructions_independent.rs |
| Tabel D.1 |  | GT | ground.rs | ground_independent.rs |
| Tabel E.1 |  | GT | materials.rs | constructions_independent.rs |
| Tabel E.2 |  | GT | materials.rs | constructions_independent.rs |
| Tabel E.3 |  | GT | materials.rs | constructions_independent.rs |
| Tabel E.4 |  | GT | materials.rs | constructions_independent.rs |
| Tabel E.5 |  | GT | materials.rs | constructions_independent.rs |
| Tabel E.6 |  | GT | materials.rs | constructions_independent.rs |
| Tabel E.7 |  | NVT | - | overzicht productnormen |
| Tabel E.8 |  | NVT | - | overzicht productnormen |
| Tabel E.9 |  | NVT | - | overzicht productnormen |
| Tabel E.10 |  | GT | materials.rs, constructions.rs | constructions_independent.rs |
| Tabel E.11 |  | GT | materials.rs, constructions.rs | constructions_independent.rs |
| Tabel E.12 |  | GT | materials.rs, constructions.rs | constructions_independent.rs |
| Tabel E.13 |  | GT | materials.rs, constructions.rs | constructions_independent.rs |
| Tabel E.14 |  | GT | materials.rs, constructions.rs | constructions_independent.rs |
| Tabel E.15 |  | GT | materials.rs, constructions.rs | constructions_independent.rs |
| Tabel E.16 |  | GT | materials.rs, constructions.rs | constructions_independent.rs |
| Tabel E.17 |  | GT | materials.rs, constructions.rs | constructions_independent.rs |
| Tabel F.1 |  | GT | constructions.rs | coverage_ch5_8_independent.rs (annex_f_air_cavities) |
| Tabel F.2 |  | GT | constructions.rs | coverage_ch5_8_independent.rs (annex_f_air_cavities) |
| Tabel F.3 |  | GT | constructions.rs | coverage_ch5_8_independent.rs (annex_f_air_cavities) |
| Tabel G.1 |  | GT | window_u.rs | envelope_independent.rs |
| Tabel H.1 |  | GT | materials.rs | constructions_independent.rs |
| Tabel I.1 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.2 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.4 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.5 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.6 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.7 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.8 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.9 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.10 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.11 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.12 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.15 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |
| Tabel I.16 |  | GT | forfait_envelope.rs, window_u.rs | constructions_independent.rs; opname_defaults_independent.rs |

## Niet uitgevoerd en relevant voor BENG of label

| Bepaling | Pagina | Wat ontbreekt | Gevolg |
|---|---|---|---|
| 7.44 (7.6.6.2, methode A) | 203 | De kozijnfractie per raam uit A_gl/A_w wordt niet berekend. In de projectroute geldt één F_fr voor alle ramen (`windowSolar.frameFraction`); in de zone-invoer kan per raam een waarde worden opgegeven. | Zonnewinst (7.32) en TOjuli bij ramen met verschillende kozijnfracties; methode B (0,25) is wel mogelijk. Gedeeltelijke afwijking alleen bij methode A met ongelijke ramen. |
| 8.50/8.51 (8.3.6) | 263 | ψ_gr uit een numeriek tweedimensionaal model (L_2D, U_fl;8) wordt niet berekend; ψ_gr komt uit opgave of uit bijlage I. | Geen, zolang de opnemer ψ_gr zelf bepaalt of bijlage I gebruikt (de norm staat dat toe). |
| D.17/D.18 (D.2.2.5) | 796 | Het lengtegewogen gemiddelde van h en z over de wanddelen van een onverwarmde kelder wordt niet berekend; de kern neemt één h en één z. | Geen bij één wanddeel; bij meerdere moet de opnemer het gewogen gemiddelde invoeren. |

**Bewust buiten de kern (NVT):** de numerieke rekenmethode van 8.6 (8.62–8.73, tabel 8.6); de kern neemt L_C, L_2D en L_3D als opgegeven invoer (8.5, 8.25–8.29). H_A van 8.60/8.61 staat in een opmerking; normatief is H_A = 0 (8.5). A_r (6.4) wordt nergens in de NTA verder gebruikt. De uurweging van bijlage A (A.1–A.4) vraagt uurwaarden volgens NEN 5060; de kern krijgt de gewichten per maand als invoer.

## Alleen eigen eenheidstest (GE)

- 5.41a–5.41c: verdeling van de onttrekking door een boosterwarmtepomp in juli over de oriëntaties (TOjuli). Eenheidstest in `tojuli.rs`; een route met een booster op het koudedistributiesysteem zit niet in de testsets.
- 7.82: temperatuur van een aangrenzende onverwarmde ruimte als uitvoer, gebruikt voor leidingverliezen in hoofdstuk 9 en 13. Eenheidstests in `domestic_hot_water.rs`.
- A.3/A.4: τ_sol en τ_vis van dynamische beglazing; alleen uitvoer (hoofdstuk 14 gebruikt ze niet).
- Tabel 6.1: toepassingsoverzicht, in de kern alleen gebruikt voor de kolomkeuze van bijlage AB.

## Nieuwe onafhankelijke tests (review 77)

| Test | Bepalingen | Gevallen |
|---|---|---|
| `heat_pump_space_heating_renewable` | 5.31, 5.39c, 5.28 | EPW203d, per maand |
| `heat_pump_partly_exhaust_air_renewable` | 5.32 (f_H;buitenlucht 0 en 0,4) | EPW203d met buitenluchtbron |
| `biomass_renewable`, `external_heat_declared_renewable`, `solar_water_heater_renewable` | 5.30, 5.35, 5.39d, 5.39f, 5.39g, tabel 5.4 | EPW204c, EPW204b, EPW405a |
| `pv_renewable` | 5.39, 5.39a, tabel 5.4 | EPW501a, voorbeeldwoning, voorbeeldkantoor |
| `free_cooling_renewable`, `external_cold_declared_renewable` | 5.34, 5.39e, 5.39h | EPW301a met vrije koeling (EER 10, tabel 10.34) en met externe koude met verklaring |
| `hot_water_heat_pump_renewable` | 5.36, 5.37, 5.38 | EPW406d (buitenlucht, deels afvoerlucht, afvoerlucht), EPW406i (booster) |
| `need_indicators` | 5.3a–5.3g met 7.1–7.7 en 7.46–7.56 | voorbeeldkantoor en -woning |
| `beng_one_from_the_fixed_c1_run`, `fixed_lighting_flux_of_the_c1_run` | 5.1, 5.4–5.8, vaste q_L van 5.4.2 | voorbeeldkantoor en -woning |
| `cooling_operative_temperature` | 7.80/7.81 | voorbeeldkantoor, twaalf maanden |
| `table_8_1_examples_of_formula_8_3` | 8.3, tabel 8.1 | zes gemiddelde U-waarden |
| `annex_f_air_cavities` | tabellen F.1–F.3 met de factor 1,2/0,8 | roosterpunten en één tussenwaarde |

Er is in deze ronde geen afwijking van de kern gevonden; alle 15 tests zijn gelijk zonder `#[ignore]`. De verwachte waarden worden in de test berekend uit de uitvoer van de hoofdstukken 7, 9, 10, 13 en 16, zodat een herstel elders (zoals de afronding van U volgens 8.2.2.1, p. 228, die tijdens deze review in de projectroute werd aangebracht) de tests niet ongeldig maakt.

## Lezingen en dubbelzinnigheden

- **5.36 bij de forfaitaire tapwaterwarmtepomp (NEN-vraag 17).** De kern boekt E_W;gen;in·(1,4·c_source − 1), zoals de ISSO 54-toetsset. Letterlijk gelezen geeft 5.36 Q_W;gen;out·(1 − 1/η_W;gen;prac) met η inclusief c_W;gen. Voor EPW406d met een buitenluchtbron is dat 310,28 tegenover 967,35 kWh hernieuwbaar per jaar; dat verandert RER meetbaar. De kern volgt ISSO 54; de vraag ligt bij NEN.
- **7.80 als gedrukt.** De formule telt Q_C;nd op bij de winst (p. 221); fysisch zou de koudebehoefte de temperatuur verlagen. Het is alleen een uitvoergrootheid zonder invloed op de indicatoren; de kern volgt de gedrukte formule.
- **Tabellen F.1–F.3.** De bijlage geeft geen interpolatieregel; de kern interpoleert lineair tussen de gedrukte punten en neemt boven 25 mm de rij "25 tot 500". De waarde 0,224 2 in de eerste rij van F.2 en F.3 is als 0,2242 overgenomen.
- **5.3b/5.3c en 5.3e/5.3f.** De netto behoefte laat Δη·Q_gn en de terugwinbare systeemverliezen weg (p. 75–76); a_C;red blijft in de koudebehoefte staan. Kern en herberekening lezen dat gelijk.
- **8.5 (H_A).** De binnentemperatuur van utiliteitsfuncties verwijst in de tekst naar tabel 7.11; bedoeld is tabel 7.13. Zonder gevolg, omdat H_A = 0.
- Eerder vastgelegde NEN-vragen voor deze hoofdstukken blijven staan: 8, 9 en 18 (TOjuli), 10 (RER bij EMGforf), 11 (serre), 16 (verticale leidingen), 19 (bezettingstijd), 21 (5.48 en 5.56, geothermie), 22 (afronding U naar AOR), 23 (U_x;T in D.15/D.16) en 24 (ΔU_for bij kelders).

## Bepalingen zonder formule- of tabelnummer

Deze zijn niet per regel opgenomen. Een steekproef bevestigt dat ze in eerdere reviews zijn getest: de zoneringsregels van 6.4/6.5 (`indicators_independent.rs`, zoning_boundaries), de gebruiksoppervlakte en gemeenschappelijke ruimten van 6.6 (common_space-tests), de standaard voor woningisolatie in 5.3.2 en de renovatiestandaard (tabel 5.7), de BACS-factor van 5.5.8 en de TOjuli-methoden van 5.7.1 en bijlage AA. WLC-GWP (5.5.6.2) en de koolstofemissievrije gebouwen van 5.5.7 vallen buiten deze telling.
