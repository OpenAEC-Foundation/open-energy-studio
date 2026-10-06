//! NTA 8800:2025+C1:2026, the designated edition and the kernel's base.
//! Pages refer to this edition.

use super::{AnnexAaVariant, HeatPumpSourceRoute, NormProfile, NormVersion, Table1318Lookup};

pub(super) const PROFILE: NormProfile = NormProfile {
    version: NormVersion::V2025C1,
    // 5.5.8 (p. 99–101), 15.3 (p. 676).
    bacs_factor_route: true,
    // 5.14a/5.14b (p. 86–87).
    storage_credit: true,
    // Table 5.3 (p. 96–98).
    k_co2_electricity: 0.268,
    k_co2_gas: 0.218,
    k_co2_oil: 0.326,
    k_co2_biomass: 0.104,
    k_co2_district_heat_forfait: 0.09,
    // Table 5.6 (p. 125).
    k_co2_waste_incineration: 0.138,
    // 5.3.3–5.3.5 (p. 76–77), 5.6.4 (p. 110–112), 5.9 (p. 133), annex AB.
    indicators_2025: true,
    // Table 9.16 (p. 310).
    psi_collective_combined_small: 1.0,
    // (9.62) (p. 331–332).
    collective_source_correction: true,
    // 9.6.8.1.1.2.3 (p. 362–363).
    heat_pump_source_route: HeatPumpSourceRoute::CollectiveOnly2025,
    // 17.3.2 f) (p. 702), table 17.15.
    roof_edge_obstruction: true,
    // 7.6.6.1.2–7.6.6.1.3.4 (p. 190–196).
    permanent_shading_routes: true,
    // (13.157)/(13.160) (p. 627–628).
    hot_water_building_share: true,
    // Table 13.18 (p. 630–631).
    table_13_18: Table1318Lookup::Interpolate,
    // (11.136) (p. 515).
    motor_efficiency_free_choice: true,
    // Annex AA (p. 1136–1147).
    annex_aa: AnnexAaVariant::Hourly2025,
    annex_aa_requirement_floor: true,
    // Table E.5 (p. 803).
    bio_based_ageing_one: true,
    // Tables E.11/E.12 (p. 810).
    bio_based_lambda_2025: true,
    // (I.3) (p. 833).
    reed_thatch_divisor: 0.105,
    // (W.9) (p. 1123), W.3 (p. 1126).
    booster_bounds: true,
    // P.3 item 6 (p. 936).
    wide_collective_low_temperature_sources: true,
    // Tables 5.2/5.4 (p. 94–100).
    biomass_threshold_kw: 500.0,
    // Below: unchanged since 2024; pages of the 2024 edition.
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
};
