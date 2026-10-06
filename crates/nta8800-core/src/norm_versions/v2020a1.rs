//! NTA 8800:2020+A1:2020, as the difference to 2022 (cumulative: everything
//! 2022 differs from 2023, 2024 and 2025+C1 also holds here). Pages:
//! "2020 p." in the consolidated 2020+A1 edition, "2022 p." in the 2022
//! edition. The separate interpretatiedocument of 2020+A1 is not modelled.

use super::{NormProfile, NormVersion};

pub(super) const PROFILE: NormProfile = NormProfile {
    version: NormVersion::V2020A1,
    // (16.4): K_pk per m² rounded down to a multiple of 5 W/m²; no panel
    // route 16.4b and no two-decimal area (2020 p. 651; 2022 p. 656–657).
    pv_kpk_per_m2_floor: true,
    // (I.2): λ_equi;ntr 0,045 (2020 p. 796; 2022 p. 805: 0,06).
    lambda_equi_ntr: 0.045,
    // (P.25) divisor 4 000 (2020 p. 925; 2022 p. 937: 8 800).
    reference_power_divisor: 4000.0,
    // Residual heat: η = 1 with f_P;del;rw 0,1 (table 5.5) and K_CO2 0,034
    // (table 5.6), f_Pren = 1 − 0,1 (5.47) (2020 p. 109–113, 933; 2022
    // p. 945: f_rw;aux;spec 0,07).
    residual_heat_fixed_factors: true,
    // No table P.0, no small-system value under P.11 and no 0,009 0 cold
    // distribution forfait (2020 p. 920, 960, 969; 2022 p. 930, 972, 981).
    small_system_forfait_route: false,
    // No β 0,5 for an unknown power ratio (2020 p. 925; 2022 p. 936).
    unknown_beta_route: false,
    // 9.6.8.1.1.2.1: A = 13,0 kWh from 2015 and one forfait set for all
    // devices, heat pumps included (2020 p. 334–336; 2022 p. 338).
    device_aux_a_from_2015_kwh: 13.0,
    heat_pump_aux_constants: false,
    // (11.142) f_systype = 1,5 for E1 (2020 p. 495; 2022 p. 498–499).
    fan_systype_combined: Some(1.5),
    // Tables I.1/I.2 with one Ψ column, no 0,5 default (2020 p. 786–788;
    // 2022 p. 793–796); detail 14 is 0,70 (2020 p. 786).
    psi_columns_and_default: false,
    psi_detail_14: 0.70,
    // Detail 17 has the one value 0,60 (2020 p. 787), as column A of 2022.
    // Dwellings use the forfait pipe length (2020 p. 292, 363; 2022 p. 294,
    // 367).
    residential_actual_pipe_length: false,
    // No (13.148a) (2020 p. 590; 2022 p. 594).
    declared_exhaust_air_flow_route: false,
    // No (11.106a) (2020 p. 476; 2022 p. 479).
    cold_recovery_route: false,
    ..super::v2022::PROFILE
};
