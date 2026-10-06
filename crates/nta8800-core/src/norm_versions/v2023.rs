//! NTA 8800:2023, as the difference to 2024 (cumulative: everything 2024
//! differs from 2025+C1 also holds here). Pages refer to the 2023 edition.
//! There is no interpretatiedocument for 2023.
//!
//! Not switched (see docs/nta8800-normversies.md): the detailed heating and
//! cooling emission tables of 9.3/10.3, table 13.4 by pipe class, the
//! panel-U build-year route of I.13/I.14 and the source-heat route of
//! 9.6.3.1.3. The kernel uses the 2024 route there.

use super::{AnnexAaVariant, HeatPumpSourceRoute, NormProfile, NormVersion, Table1318Lookup};

pub(super) const PROFILE: NormProfile = NormProfile {
    version: NormVersion::V2023,
    // No f_BACS and no storage credit (5.5, p. 84–91).
    bacs_factor_route: false,
    storage_credit: false,
    // Table 5.3 (p. 92–93), as 2024.
    k_co2_electricity: 0.34,
    k_co2_gas: 0.183,
    k_co2_oil: 0.260,
    k_co2_biomass: 0.372,
    k_co2_district_heat_forfait: 0.17,
    // Table 5.6 (p. 114).
    k_co2_waste_incineration: 0.113,
    indicators_2025: false,
    // Table 9.16 (p. 296), as 2024.
    psi_collective_combined_small: 2.0,
    collective_source_correction: false,
    heat_pump_source_route: HeatPumpSourceRoute::AnySourceFrom15C2024,
    roof_edge_obstruction: false,
    permanent_shading_routes: false,
    hot_water_building_share: false,
    // Table 13.18 (p. 617), as 2024.
    table_13_18: Table1318Lookup::LowerColumn,
    motor_efficiency_free_choice: false,
    // Unused: annex AA is no route in 2023 (`annex_aa_route`).
    annex_aa: AnnexAaVariant::Monthly2024,
    annex_aa_requirement_floor: false,
    // Table E.5 (p. 785).
    bio_based_ageing_one: false,
    bio_based_lambda_2025: false,
    // (I.3) (p. 815).
    reed_thatch_divisor: 0.2,
    booster_bounds: false,
    wide_collective_low_temperature_sources: false,
    // Tables 5.2/5.3/5.6 (p. 90–93, 114): bmA above 100 kW per installation.
    biomass_threshold_kw: 100.0,
    // Table 13.2 (p. 533): kitchen rows for d_in ≤ 8 mm and ≤ 10 mm.
    kitchen_diameter_rows: true,
    // Table 11.7 (p. 454).
    tau_ventilative_cooling: [
        0.00, 0.02, 0.00, 0.00, 0.12, 0.18, 0.28, 0.25, 0.17, 0.06, 0.01, 0.00,
    ],
    // 11.2.3.3.1 (p. 466).
    ventilative_cooling_operation: [0.5, 0.9],
    // 11.2.3.3.1 (p. 465) gives (11.71) without (11.71a).
    discharge_opening_route: false,
    // Table 11.8 (p. 460): f_τ 0,80 for the residential function.
    dwelling_occupancy_factor: Some(0.80),
    // Table 17.1 (p. 676): May–September differ from 2024.
    argii_temperature_c: [
        None,
        Some(13.97),
        Some(13.00),
        Some(13.70),
        Some(14.56),
        Some(15.62),
        Some(16.17),
        Some(16.90),
        Some(15.11),
        Some(15.04),
        Some(13.43),
        None,
    ],
    // 11.3.2.7 (p. 496).
    fan_temperature_rise_k: [1.0, 0.7, 1.5],
    // Table I.1 detail 17 (p. 804).
    psi_detail_17: [0.60, 0.90],
    // Table E.10 (p. 791).
    wood_fibre_cellulose_lambda: 0.050,
    // (P.25) (p. 950).
    reference_power_divisor: 8800.0,
    // P.6.5.4.8 (p. 960): forfait 20.
    geothermal_efficiency_fixed: true,
    // 5.7.1 (p. 103–104): active cooling counts as meeting TOjuli; no
    // annex AA.
    annex_aa_route: false,
    // No renovatiestandaard and no table 5.7 in chapter 5 (5.3.1, p. 70–72).
    renovation_standard: false,
};
