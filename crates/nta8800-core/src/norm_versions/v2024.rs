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
};
