//! NTA 8800:2024 with interpretatiedocument INT-V1:2024, as the difference to
//! 2025+C1. Pages refer to the 2024 edition; "INT" to INT-V1:2024.
//!
//! Not a difference, because INT-V1 brings 2024 in line with 2025+C1:
//! table 9.4 (INT p. 4: 2,5 / 2,0 / 1,5 / 2,5 K), the emission-loss formula
//! (9.16) (INT p. 5, = 2025 (9.12a)), the collective buffer vessel (INT
//! p. 3–4), the removed last term of (13.185) (INT p. 5) and θH,max in
//! (13.141b) (INT p. 5).

use super::{AnnexAaVariant, HeatPumpSourceRoute, NormProfile, NormVersion, Table1318Lookup};

pub(super) const PROFILE: NormProfile = NormProfile {
    version: NormVersion::V2024,
    // No f_BACS in 2024 (5.5.3, p. 87–88).
    bacs_factor_route: false,
    // No storage credit in 2024 (5.5.2, p. 84–86).
    storage_credit: false,
    // Table 5.3 (p. 94–95).
    k_co2_electricity: 0.34,
    k_co2_gas: 0.183,
    k_co2_oil: 0.260,
    k_co2_biomass: 0.372,
    k_co2_district_heat_forfait: 0.17,
    // Table 5.6 (p. 117).
    k_co2_waste_incineration: 0.113,
    indicators_2025: false,
    // Table 9.16 (p. 292).
    psi_collective_combined_small: 2.0,
    // (9.62) without f_cor.bron.col (p. 314–315).
    collective_source_correction: false,
    // 9.6.3.1.3 (p. 323) with INT p. 5.
    heat_pump_source_route: HeatPumpSourceRoute::AnySourceFrom15C2024,
    // 17.3.2 has no roof-edge situation (p. 678–687).
    roof_edge_obstruction: false,
    // 7.6.6.1.2/7.6.6.1.3 (p. 179–180): only the ISO 15099 route.
    permanent_shading_routes: false,
    // (13.157)/(13.160) without f_gebouw;si;W (p. 611–612).
    hot_water_building_share: false,
    // Table 13.18 without an interpolation rule (p. 614).
    table_13_18: Table1318Lookup::LowerColumn,
    // (11.136): a measured motor efficiency only when higher (p. 499).
    motor_efficiency_free_choice: false,
    // Annex AA 2024 (p. 1115–1127).
    annex_aa: AnnexAaVariant::Monthly2024,
    annex_aa_requirement_floor: false,
    // Table E.5 (p. 783): only mineral-wool flakes at 1,00.
    bio_based_ageing_one: false,
    // Tables E.11/E.12 (p. 791).
    bio_based_lambda_2025: false,
    // (I.3) (p. 814).
    reed_thatch_divisor: 0.2,
    // (W.9) (p. 1103) and W.3 (p. 1106) without the bounds.
    booster_bounds: false,
    // P.3 item 6 (p. 916): aquifer/groundwater and ground only.
    wide_collective_low_temperature_sources: false,
    // Tables 5.2/5.4 (p. 92–95).
    biomass_threshold_kw: 500.0,
    // Table 13.2 (2024 p. 527): only the "overig" kitchen row.
    kitchen_diameter_rows: false,
    // Table 11.7 (2024 p. 449).
    tau_ventilative_cooling: crate::ventilation::TAU_VENTILATIVE_COOLING,
    // 11.2.3.3.1 (2024 p. 461).
    ventilative_cooling_operation: [0.35, 0.50],
    // (11.71a) (2024 p. 460–461).
    discharge_opening_route: true,
    // Table 11.8 (2024 p. 455).
    dwelling_occupancy_factor: None,
    // Table 17.1 (2024 p. 674).
    argii_temperature_c: crate::climate::ARGII_TEMPERATURE_C,
    // 11.3.2.7 (2024 p. 491).
    fan_temperature_rise_k: [0.7, 0.4, 0.7],
    // Table I.1 detail 17 (2024 p. 803).
    psi_detail_17: [0.06, 0.09],
    // Table E.10 (2024 p. 790).
    wood_fibre_cellulose_lambda: 0.045,
    // (P.25) (2024 p. 948).
    reference_power_divisor: crate::annex_p::REFERENCE_POWER_DIVISOR,
    // P.6.5.4.8 (2024 p. 958) with the temperature-difference correction.
    geothermal_efficiency_fixed: false,
    // Annex AA (2024 p. 1115–1127).
    annex_aa_route: true,
    // Table 5.7 (2024 p. 74).
    renovation_standard: true,
    // Table 14.3 (2024 p. 646).
    led_2017_column: true,
    // (8.17)/(8.18) with glazing bars (2024 p. 219–222).
    glazing_bar_term: true,
    dwelling_shading_heating_off: true,
    residual_heat_pren_primary: false,
    rooflight_route: true,
    panel_build_year_tables: false,
    unspecified_screen_factor: 0.3,
    cross_area_roof_all_sectors: true,
    distribution_2024_rules: true,
    table_9_16_combined_rows: true,
    distribution_half_recoverable_route: false,
    table_13_4_system_rows: false,
    hot_water_series_routes: true,
    emission_tables_2023: false,
    cooling_emission_tables_2023: false,
    // From 2023 on; the 2022 values and both pages are in v2022.rs.
    heat_pump_tables_2022: false,
    flex_mode_route: true,
    preference_beta_building_share: true,
    design_classes_60_and_70: true,
    tested_emission_fan_power: true,
    unknown_shade_colour_rows: true,
    thermal_mass_by_kg_per_m2: false,
    crawl_wall_height_fixed: true,
    mineral_wool_flakes_ageing: 1.00,
    lambda_equi_ntr: 0.045,
    lambda_equi_known_route: true,
    swimming_pool_route: true,
    en_13053_route: true,
    electric_boiler_insulated_pipe_factor: false,
    storage_ambient_exhaust_air: true,
    mixed_air_route: true,
    lighting_maintenance_factor: false,
    pv_kpk_per_m2_floor: false,
    residual_heat_fixed_factors: false,
    small_system_forfait_route: true,
    unknown_beta_route: true,
    device_aux_a_from_2015_kwh: 43.8,
    heat_pump_aux_constants: true,
    fan_systype_combined: None,
    psi_columns_and_default: true,
    psi_detail_14: 0.03,
    residential_actual_pipe_length: true,
    declared_exhaust_air_flow_route: true,
    cold_recovery_route: true,
    heat_pump_high_test_standard: "NEN-EN 14511-2:2022",
};
