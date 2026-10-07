//! NTA 8800:2022, as the difference to 2023 (cumulative: everything 2023
//! differs from 2024 and 2025+C1 also holds here). Pages: "2022 p." in the
//! 2022 edition, "2023 p." in the 2023 edition. There is no
//! interpretatiedocument for 2022.

use super::{HeatPumpSourceRoute, NormProfile, NormVersion};

pub(super) const PROFILE: NormProfile = NormProfile {
    version: NormVersion::V2022,
    // Tables 9.27/9.29 end at 50–55 °C and have no source-temperature rows
    // (2022 p. 314–317); above 55 °C annex Q applies (2022 p. 313). Table
    // P.5 likewise (2022 p. 941). 2023 p. 319–324, 955.
    heat_pump_tables_2022: true,
    // 5.20 and (9.84) without Q_HD;hp;in;bron, no 9.6.8.1.1.2.3
    // (2022 p. 94–96, 335, 341; 2023 p. 96–98, 343, 349).
    heat_pump_source_route: HeatPumpSourceRoute::None2022,
    // No flex mode (2022 p. 109, 914–948; 2023 p. 111, 963).
    flex_mode_route: false,
    // Tables 5.2/5.4: bmA is biomass under the Activiteitenbesluit, without
    // a kW limit (2022 p. 88–92; 2023 p. 90–94: 100 kW).
    biomass_threshold_kw: f64::INFINITY,
    // (9.58) without f_gebouw;si;H (2022 p. 305; 2023 p. 309).
    preference_beta_building_share: false,
    // Table 9.14 without 60/50 and 70/60 (2022 p. 290; 2023 p. 294).
    design_classes_60_and_70: false,
    // No NEN-EN 16430 fan power under table 9.11 (2022 p. 282; 2023 p. 286).
    tested_emission_fan_power: false,
    // Table 7.5 without "onbekende kleur" (2022 p. 176; 2023 p. 180).
    unknown_shade_colour_rows: false,
    // Table 7.10 by kg/m² (2022 p. 181–182; 2023 p. 185–186).
    thermal_mass_by_kg_per_m2: true,
    // (8.47) h is the actual height (2022 p. 236; 2023 p. 240: 0,125 m).
    crawl_wall_height_fixed: false,
    // Table E.5 (2022 p. 775; 2023 p. 785: 1,00).
    mineral_wool_flakes_ageing: 1.05,
    // (I.2) (2022 p. 805; 2023 p. 814: 0,045 or a known higher λ).
    lambda_equi_ntr: 0.06,
    lambda_equi_known_route: false,
    // No swimming pool in (11.57) (2022 p. 451; 2023 p. 459).
    swimming_pool_route: false,
    // No NEN-EN 13053 row (2022 p. 482; 2023 p. 490–491).
    en_13053_route: false,
    // f_sto;dis;ls 1,5 (2022 p. 550; 2023 p. 557–558).
    electric_boiler_insulated_pipe_factor: true,
    // (13.69)/(13.137) only (2022 p. 557, 584; 2023 p. 565, 592).
    storage_ambient_exhaust_air: false,
    // No C_W;mixed air (2022 p. 600–602; 2023 p. 609–611).
    mixed_air_route: false,
    // Table 14.4 MF (2022 p. 639; 2023 p. 649: MF = 1).
    lighting_maintenance_factor: true,
    // Table 9.28 refers to NEN-EN 14511-2, dated 2007 in the normative
    // references (2022 p. 14, 316; 2023 p. 322: NEN-EN 14511-2:2022).
    // 2020+A1 is the same (2020 p. 15, 314).
    heat_pump_high_test_standard: "NEN-EN 14511-2:2007",
    ..super::v2023::PROFILE
};
