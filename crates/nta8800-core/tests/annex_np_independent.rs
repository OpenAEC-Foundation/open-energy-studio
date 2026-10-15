//! Review 76: annex N (local heaters, stoves) and the parts of the calculated annex P route outside review 75 (pipe
//! losses P.13-P.18 with buffers, auxiliary energy P.56-P.70, area demand P.72-P.83, CHP P.26-P.30, flex mode, collective
//! solar collectors with declared contributions, hot-water storage P.34-P.46 with P.40-P.42, and the cold generators of
//! tables P.9/P.10 with P.51-P.55), recalculated independently from NTA 8800:2025+C1:2026. The expected values come from a
//! separate implementation written from the norm pages (pp. 121-132, 880-922, 952-1026) without first reading the kernel
//! code of the route; only numbers and page references are kept here. Tolerance 1e-6 relative.
//!
//! Readings where the text leaves a choice and the kernel's choice is kept (see docs/nta8800-verificatiestatus.md):
//! annex N: f_prac as E_H;gen;in / 0,95; N.37/N.61 with Q_lrh;sby;rh in the numerator and 100·(1 + Q_br;rh/Q) in the
//! denominator; ϑ_lrh;gen;air = ϑ_int; N.63 with the minimum-load auxiliaries; N.52 with β = 1 and N.69 with the total
//! losses α_ON (N.49/N.50) in modulation; β_mod limited to 0..1 and E_H;gen;in from N.70 with the last β_mod (step 10 as
//! repeated blocks of steps 6-9); table N.28 0,8 for a boiler room. Annex P: P.14 months not below 0 (cold networks count
//! gains only); buffer and other losses over the months by length, like the losses of the flow and forfait methods; a cold
//! network at 10 °C or more has no loss at all (P.6.2.2); P.74 per plot part with max(0, 18 − θ_e), hot water by month
//! length (P.82); without monthly cold the input is split by month length; P.53 with each chiller's own COP; table P.7
//! between rows with the next larger diameter and the next thinner insulation; K_CO2;WD;gen;tot divided by η_WD;gen;sto
//! like P.34; no P.59 standby for a collective solar collector (its auxiliary energy is W_sol).

use nta8800_core::annex_n::{heater_month, resolve_heater, HeaterMonth, LocalHeater};
use nta8800_core::annex_p::{assess_route, AnnexPRoute, SystemFunction};
use serde_json::Value;

fn close(label: &str, kernel: f64, expected: f64) {
    let diff = (kernel - expected).abs();
    let scale = kernel.abs().max(expected.abs()).max(1e-12);
    assert!(
        diff <= 1e-9 || diff / scale <= 1e-6,
        "{label}: kernel {kernel} independent {expected}"
    );
}

/// Months as ([Q_H;gen;out;req, t_lrh;gen, ϑ_int, ϑ_ext], [Q_H;gen;out, shortfall, E_H;gen;in/0,95, W_H;gen;aux,
/// Q_H;gen;out;w, β]).
fn annex_n(heater: &str, months: &[([f64; 4], [f64; 6])]) {
    let heater: LocalHeater = serde_json::from_str(heater).expect("heater");
    let resolved = resolve_heater(&heater, "heater").expect("resolved");
    for (index, (input, want)) in months.iter().enumerate() {
        let result = heater_month(
            &heater,
            &resolved,
            HeaterMonth {
                heat_required_kwh: input[0],
                operating_hours: input[1],
                indoor_c: input[2],
                outdoor_c: input[3],
            },
        );
        let got = [
            result.heat_output_kwh,
            result.shortfall_kwh,
            result.input_kwh,
            result.auxiliary_electricity_kwh,
            result.water_side_heat_kwh,
            result.load_factor,
        ];
        for (name, (kernel, expected)) in ["Q_out", "shortfall", "E_in", "W_aux", "Q_w", "β"]
            .iter()
            .zip(got.iter().zip(want))
        {
            close(&format!("month {} {name}", index + 1), *kernel, *expected);
        }
    }
}

fn annex_p(route: &str, function: SystemFunction, ids: &[&str], want: &[(&str, f64)]) {
    let route: AnnexPRoute = serde_json::from_str(route).expect("route");
    let result = assess_route(&route, function, "route").expect("annex P");
    let result: Value = serde_json::to_value(result).expect("serializable");
    let generators = result["generators"].as_array().expect("generators");
    assert_eq!(generators.len(), ids.len());
    for (generator, id) in generators.iter().zip(ids) {
        assert_eq!(generator["id"], *id);
    }
    for (pointer, expected) in want {
        let kernel = result
            .pointer(pointer)
            .and_then(Value::as_f64)
            .unwrap_or_else(|| panic!("{pointer} missing"));
        close(pointer, kernel, *expected);
    }
}

// ---------------------------------------------------------------------------------------------------
// Annex N (pp. 880-922): N.3 on/off, N.4 high-low and modulating, N.5 stoves, N.6 tables N.20-N.30
// ---------------------------------------------------------------------------------------------------

/// Air heater with atmospheric burner, on/off (N.3): table N.20 after 2005 (α_ch;on 13 %, f_corr 0,18, ϑ_test 20 °C),
/// table N.22 centrifugal (n 0,1, k_br 1,7 %); N.36/N.37 iteration, N.33/N.34 with a shortfall in month 2.
#[test]
fn annex_n_atmospheric_on_off() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"none","location":"heated_space_free","productionPeriod":"after2005","sourceReference":"x","heaterType":"air_heater_atmospheric","control":"on_off","fan":"centrifugal","envelopeInsulation":"old_average","product":{"inputFullKw":40}}"#,
        &[
            (
                [3000.0, 500.0, 18.0, 5.0],
                [
                    3000.0,
                    0.0,
                    3458.3904718008016,
                    55.85300611958295,
                    0.0,
                    0.16427354741053807,
                ],
            ),
            (
                [12000.0, 300.0, 18.0, -2.0],
                [
                    10687.199999999999,
                    1312.800000000001,
                    12602.178410316514,
                    203.52518132661172,
                    0.0,
                    0.9976724574833906,
                ],
            ),
            (
                [400.0, 200.0, 18.0, 12.0],
                [
                    400.0,
                    0.0,
                    455.693513705772,
                    7.359450246348217,
                    0.0,
                    0.05411360475256042,
                ],
            ),
        ],
    );
}

/// Dark radiator with flue, 1990-2005, pilot (table N.24: 2 %), ventilation not interlocked (N.21/N.22 with N.16-N.18
/// and table N.30), against a wall (table N.29: k_env 0,1; N.10 with table N.23).
#[test]
fn annex_n_radiant_flue_pilot_vent() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":true,"ventilation":"required","location":"heated_space_against_wall_or_roof","productionPeriod":"from1990_to2005","sourceReference":"x","heaterType":"radiant_tube_with_flue","control":"on_off","envelopeInsulation":"well_insulated_maintained","roomHeightM":8,"product":{"inputFullKw":30}}"#,
        &[
            (
                [2500.0, 400.0, 17.0, 3.0],
                [
                    2500.0,
                    0.0,
                    3813.647175003576,
                    9.057412040633492,
                    0.0,
                    0.3019137346877831,
                ],
            ),
            (
                [6000.0, 450.0, 17.0, -1.0],
                [
                    6000.0,
                    0.0,
                    8247.531036900034,
                    19.58788621263758,
                    0.0,
                    0.5803818137077802,
                ],
            ),
        ],
    );
}

/// Dark radiator > 60 kW (table N.22: n 0,15, k_blw 2 %, k_br 0,3 %), before 1990, interlocked ventilation (N.19/N.20),
/// under the roof outside the heated space (k_env 0,8, table N.28 k_aux;rh 0,8), product standby and pilot; month 3 without demand (N.29/N.31, t_ON = 0).
#[test]
fn annex_n_large_radiant_outside() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":true,"ventilation":"interlocked","location":"under_roof_outside_heated_space","productionPeriod":"before1990","sourceReference":"x","heaterType":"radiant_tube_with_flue","control":"on_off","envelopeInsulation":"old_poor","roomHeightM":12,"product":{"inputFullKw":80,"auxStandbyKw":0.05,"pilotLossPercent":1.5}}"#,
        &[
            (
                [9000.0, 600.0, 16.0, 4.0],
                [
                    9000.0,
                    0.0,
                    11468.540620988952,
                    1015.8758947761063,
                    0.0,
                    0.22698153312373964,
                ],
            ),
            (
                [60000.0, 700.0, 16.0, 0.0],
                [
                    43504.96549117926,
                    16495.03450882074,
                    58893.421273345375,
                    1287.8782817479855,
                    0.0,
                    0.9990848251728232,
                ],
            ),
            ([0.0, 100.0, 16.0, 10.0], [0.0, 0.0, 0.0, 165.0, 0.0, 0.0]),
        ],
    );
}

/// Condensing air heater on/off: table N.25 η 104 % gives α_cond 4 (N.26) next to α_ch;on 5 % (table N.20); boiler room.
#[test]
fn annex_n_condensing_on_off() {
    annex_n(
        r#"{"condensing":true,"pilotFlame":false,"ventilation":"none","location":"boiler_room","productionPeriod":"after2005","sourceReference":"x","heaterType":"condensing_air_heater","control":"on_off","fan":"axial","envelopeInsulation":"well_insulated_new_high_efficiency","product":{"inputFullKw":25}}"#,
        &[(
            [2000.0, 500.0, 20.0, 6.0],
            [
                2000.0,
                0.0,
                2109.8246095522263,
                18.039000411671534,
                0.0,
                0.1603466703259692,
            ],
        )],
    );
}

/// Modulating air heater with modulating combustion air: table N.26 (β_Pmin 0,7, α 6 %), N.4.4.5.3 at minimum load
/// (months 1 and 2) and N.4.4.5.4 modulation (month 3).
#[test]
fn annex_n_modulating_comb_air() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"none","location":"heated_space_against_wall_or_roof","productionPeriod":"after2005","sourceReference":"x","heaterType":"air_heater_modulating_combustion_air","control":"modulating","fan":"centrifugal","envelopeInsulation":"old_average","product":{"inputFullKw":50}}"#,
        &[
            (
                [4000.0, 400.0, 18.0, 5.0],
                [
                    4000.0,
                    0.0,
                    4342.707872225666,
                    100.192474480635,
                    0.0,
                    0.2946837484724559,
                ],
            ),
            (
                [12000.0, 450.0, 18.0, 0.0],
                [
                    12000.0,
                    0.0,
                    13096.200046428958,
                    302.1480439283252,
                    0.0,
                    0.7899295266100006,
                ],
            ),
            (
                [17000.0, 450.0, 18.0, -3.0],
                [
                    17000.0,
                    0.0,
                    18760.531518262436,
                    382.5,
                    0.0,
                    0.3070216667758719,
                ],
            ),
        ],
    );
}

/// High-low condensing air heater without modulating combustion air (table N.25: 102/90 %), boiler room, pilot. Month 2:
/// Q/t_gen (40 kW) lies below Φ_Pmin (42 kW) while N.61 converges above 1, so N.4.4.5.4 applies and the first β_mod of
/// N.66 is negative (clamped to 0). Step 10 (p. 907) repeats steps 6-9 until β_mod changes by less than 0,001; the kernel
/// compares the first β_mod with its start value 0 and stops after one pass (β_mod 0, E_H;gen;in 3 % too high).
#[test]
fn annex_n_modulation_first_beta_below_minimum() {
    annex_n(
        r#"{"condensing":true,"pilotFlame":true,"ventilation":"none","location":"boiler_room","productionPeriod":"from1990_to2005","sourceReference":"x","heaterType":"air_heater_modulating_no_combustion_air","control":"high_low","fan":"axial","envelopeInsulation":"old_poor","product":{"inputFullKw":60}}"#,
        &[
            (
                [6000.0, 500.0, 19.0, 5.0],
                [
                    6000.0,
                    0.0,
                    7964.672154611561,
                    97.28278131704121,
                    0.0,
                    0.360306597470523,
                ],
            ),
            (
                [20000.0, 500.0, 19.0, 1.0],
                [
                    20000.0,
                    0.0,
                    24837.316440265735,
                    270.0,
                    0.0,
                    0.28833075156171145,
                ],
            ),
        ],
    );
}

/// The first month of the same heater (N.4.4.5.3 at minimum load) is not affected.
#[test]
fn annex_n_highlow_condensing_minimum_load() {
    annex_n(
        r#"{"condensing":true,"pilotFlame":true,"ventilation":"none","location":"boiler_room","productionPeriod":"from1990_to2005","sourceReference":"x","heaterType":"air_heater_modulating_no_combustion_air","control":"high_low","fan":"axial","envelopeInsulation":"old_poor","product":{"inputFullKw":60}}"#,
        &[(
            [6000.0, 500.0, 19.0, 5.0],
            [
                6000.0,
                0.0,
                7964.672154611561,
                97.28278131704121,
                0.0,
                0.360306597470523,
            ],
        )],
    );
}

/// High-temperature radiator, high-low, ventilation required (N.21/N.22, N.50/N.51 divided by k_mod).
#[test]
fn annex_n_ht_radiant_highlow_vent() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"required","location":"heated_space_free","productionPeriod":"after2005","sourceReference":"x","heaterType":"high_temperature_radiant","control":"high_low","roomHeightM":10,"envelopeInsulation":"none","product":{"inputFullKw":20}}"#,
        &[
            (
                [1500.0, 300.0, 16.0, 4.0],
                [
                    1500.0,
                    0.0,
                    1839.6476201597332,
                    6.291594860946287,
                    0.0,
                    0.5825550797172488,
                ],
            ),
            (
                [4500.0, 350.0, 16.0, -1.0],
                [
                    4500.0,
                    0.0,
                    5215.243177303365,
                    12.6,
                    0.0,
                    0.41555530821341036,
                ],
            ),
        ],
    );
}

/// Fan-burner air heater with product values (N.3.3.1.2, table N.8: Φ_Pmin, α_ch;on;Pmin, Φ_aux;br/blw;Pmin), outdoors
/// (k_env 1); N.63 with the minimum-load auxiliaries and N.72 interpolation.
#[test]
fn annex_n_product_values_modulating() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"interlocked","location":"outdoors","productionPeriod":"after2005","sourceReference":"x","heaterType":"air_heater_fan_burner","control":"modulating","fan":"centrifugal","roomHeightM":6,"product":{"inputFullKw":45,"chimneyLossPercent":9,"chimneyCorrectionFactor":0.2,"testAirTemperatureC":15,"loadExponent":0.12,"auxBurnerKw":0.4,"auxAfterBurnerKw":0.15,"auxStandbyKw":0.02,"envelopeLossPercent":1.5,"inputMinKw":18,"chimneyLossMinPercent":5,"auxBurnerMinKw":0.2,"auxAfterBurnerMinKw":0.1}}"#,
        &[
            (
                [2500.0, 400.0, 18.0, 6.0],
                [
                    2500.0,
                    0.0,
                    3137.809961615746,
                    77.8091946353496,
                    0.0,
                    0.4140165921576332,
                ],
            ),
            (
                [12000.0, 420.0, 18.0, 1.0],
                [
                    12000.0,
                    0.0,
                    15415.356791787128,
                    191.59337124097732,
                    0.0,
                    0.6246987737235935,
                ],
            ),
        ],
    );
}

/// Solid-fuel stove (N.5.2): table N.27 η 50 %, N.74/N.75, α_env 0 (N.5.2.3).
#[test]
fn annex_n_stove_solid() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"none","location":"heated_space_free","productionPeriod":"after2005","sourceReference":"x","heaterType":"stove","control":"on_off","stoveKind":"solid_fuel_room_heater","product":{"outputFullKw":8}}"#,
        &[(
            [2000.0, 600.0, 20.0, 5.0],
            [
                2000.0,
                0.0,
                4210.526315789473,
                0.0,
                0.0,
                0.41666666666666663,
            ],
        )],
    );
}

/// Pellet stove with water connection (N.5.3): N.77-N.80, Q_H;gen;out;w = Q_H;gen;out·Φ_w/Φ_air.
#[test]
fn annex_n_stove_pellet_water() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"none","location":"heated_space_free","productionPeriod":"after2005","sourceReference":"x","heaterType":"stove","control":"on_off","stoveKind":"pellet","waterConnection":{"outputToAirKw":6,"outputToWaterKw":4},"product":{}}"#,
        &[(
            [1800.0, 500.0, 20.0, 4.0],
            [1800.0, 0.0, 4210.526315789474, 0.0, 1200.0, 0.6],
        )],
    );
}

/// Gas stove with ventilation required and product standby power.
#[test]
fn annex_n_gas_stove() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"required","location":"heated_space_free","productionPeriod":"after2005","sourceReference":"x","heaterType":"stove","control":"on_off","stoveKind":"gas_or_oil","roomHeightM":2.6,"product":{"outputFullKw":5,"combustionEfficiencyPercent":85,"auxStandbyKw":0.003}}"#,
        &[(
            [1200.0, 500.0, 20.0, 5.0],
            [
                1200.0,
                0.0,
                1649.6583866253868,
                0.7007405116800002,
                0.0,
                0.5328396588799998,
            ],
        )],
    );
}

/// Dark radiator with flue, modulating, combustion efficiency given: α_ch;on = 100 − η (N.6).
#[test]
fn annex_n_radiant_flue_modulating_eff() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"none","location":"heated_space_free","productionPeriod":"after2005","sourceReference":"x","heaterType":"radiant_tube_with_flue","control":"modulating","envelopeInsulation":"well_insulated_maintained","product":{"inputFullKw":35,"combustionEfficiencyPercent":92}}"#,
        &[
            (
                [3000.0, 300.0, 17.0, 5.0],
                [
                    3000.0,
                    0.0,
                    3369.881337682211,
                    11.43352596713607,
                    0.0,
                    0.435562893986136,
                ],
            ),
            (
                [9000.0, 400.0, 17.0, 0.0],
                [
                    9000.0,
                    0.0,
                    10171.034652669345,
                    34.5088675715567,
                    0.0,
                    0.98596764490162,
                ],
            ),
        ],
    );
}

/// Modulating condensing air heater with table N.25 (94/104 %: α_cond;Pmin 4, N.53), pilot and ventilation, outside
/// the heated space; all three modes.
#[test]
fn annex_n_modulating_condensing_min() {
    annex_n(
        r#"{"condensing":true,"pilotFlame":true,"ventilation":"required","location":"under_roof_outside_heated_space","productionPeriod":"after2005","sourceReference":"x","heaterType":"air_heater_modulating_combustion_air","control":"modulating","fan":"centrifugal","envelopeInsulation":"well_insulated_new_high_efficiency","roomHeightM":7,"product":{"inputFullKw":70}}"#,
        &[
            (
                [5000.0, 450.0, 18.0, 4.0],
                [
                    5000.0,
                    0.0,
                    7209.874313758283,
                    166.34210023885183,
                    0.0,
                    0.31062950558142266,
                ],
            ),
            (
                [16000.0, 500.0, 18.0, 1.0],
                [
                    16000.0,
                    0.0,
                    19168.503113508617,
                    442.24475040452023,
                    0.0,
                    0.7432684880748239,
                ],
            ),
            (
                [26000.0, 520.0, 18.0, -2.0],
                [
                    26000.0,
                    0.0,
                    30934.175595618144,
                    618.8,
                    0.0,
                    0.3578099741667138,
                ],
            ),
        ],
    );
}

/// Condensing gas stove: α_ch;ON = max(0, 100 − 104) (N.75 with p. 909) and α_cond 4.
#[test]
fn annex_n_condensing_gas_stove() {
    annex_n(
        r#"{"condensing":true,"pilotFlame":false,"ventilation":"none","location":"heated_space_free","productionPeriod":"after2005","sourceReference":"x","heaterType":"stove","control":"on_off","stoveKind":"gas_or_oil","product":{"outputFullKw":6,"combustionEfficiencyPercent":104}}"#,
        &[
            (
                [1500.0, 600.0, 20.0, 3.0],
                [
                    1500.0,
                    0.0,
                    1518.2186234817816,
                    0.0,
                    0.0,
                    0.4166666666666667,
                ],
            ),
            (
                [6000.0, 650.0, 20.0, -1.0],
                [3900.0, 2100.0, 3947.3684210526317, 0.0, 0.0, 1.0],
            ),
        ],
    );
}

/// High-low radiator with a shortfall (N.58) and an on/off month at minimum load with standby.
#[test]
fn annex_n_highlow_ht_radiant_shortfall() {
    annex_n(
        r#"{"condensing":false,"pilotFlame":false,"ventilation":"interlocked","location":"heated_space_against_wall_or_roof","productionPeriod":"after2005","sourceReference":"x","heaterType":"high_temperature_radiant","control":"high_low","roomHeightM":9,"envelopeInsulation":"old_average","product":{"inputFullKw":30,"auxStandbyKw":0.01}}"#,
        &[
            (
                [30000.0, 600.0, 15.0, -3.0],
                [
                    16841.15520134952,
                    13158.84479865048,
                    18947.368421052633,
                    32.4,
                    0.0,
                    1.0,
                ],
            ),
            (
                [3000.0, 600.0, 15.0, 6.0],
                [
                    3000.0,
                    0.0,
                    3381.419648614605,
                    15.422889420806031,
                    0.0,
                    0.35692762957598606,
                ],
            ),
        ],
    );
}

// ---------------------------------------------------------------------------------------------------
// Annex P (pp. 952-1026) with §5.8 (pp. 121-132)
// ---------------------------------------------------------------------------------------------------

/// Heat network with pipe losses per month (P.13/P.14): buried pipes (P.17, table P.1 λ_g 1,75 and a measured 1,2,
/// f_x 1,05/1,00), pipes in air (P.18, h_a 8 indoors and in the crawl space, 25 outdoors, a measured 12; f_x 0,90 default,
/// 0,80, 0,95), crawl space P.16, a supplied R, months out of operation, a buffer (P.43/P.44 at 80/12 °C) and other losses;
/// CHP without power loss from table P.6 (800 kWe after 2006, LT: 0,52/0,32; P.26/P.27); pumps 8 760 h (P.57); generator
/// auxiliaries P.59/P.60 (100 W, 1 W/kW for gas, t_on from P.33).
#[test]
fn annex_p_heat_pipes_monthly_chp() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":4000000,"distribution":{"method":"pipes","sourceReference":"x","otherLossKwh":50000,"segments":[{"lengthM":3000,"layers":[{"conductivityWPerMk":0.027,"innerDiameterM":0.1143,"outerDiameterM":0.2},{"conductivityWPerMk":0.4,"innerDiameterM":0.2,"outerDiameterM":0.21}],"placement":{"kind":"buried","coverDepthM":0.8,"ambient":{"kind":"outdoor"}},"correction":"two_pipes_in_trench"},{"lengthM":400,"layers":[{"conductivityWPerMk":0.035,"innerDiameterM":0.06,"outerDiameterM":0.12}],"placement":{"kind":"buried","coverDepthM":0.5,"groundConductivity":1.2,"ambient":{"kind":"crawlspace"}},"correction":"single_pipe_in_trench"},{"lengthM":600,"layers":[{"conductivityWPerMk":0.04,"innerDiameterM":0.05,"outerDiameterM":0.11}],"placement":{"kind":"in_air","ambient":{"kind":"indoor","temperatureC":18}}},{"lengthM":50,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"outdoor"}},"correction":"old_sliding_system"},{"lengthM":120,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"crawlspace"},"surfaceCoefficient":12},"correctionFactor":0.95},{"lengthM":200,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"buried","coverDepthM":1,"ambient":{"kind":"outdoor"}},"resistanceKmPerW":2.5}],"waterTemperature":{"method":"monthly","temperaturesC":[75,75,72,68,null,null,null,null,65,70,73,75]},"buffers":[{"volumeL":20000,"diameterAtLeast50Cm":true,"insulation":"at_least20_mm","waterTemperatureC":80,"ambientTemperatureC":12}]},"generators":[{"id":"chp","energyFraction":0.6,"nominalPowerKw":1000,"kind":{"kind":"chp_without_loss","carrier":{"kind":"natural_gas"},"tableP6":{"electricalPowerKw":800,"installedAfter2006":true,"temperatureLevel":"low"}}},{"id":"b","energyFraction":0.4,"nominalPowerKw":3000,"kind":{"kind":"boiler","carrier":{"kind":"natural_gas"},"efficiency":{"method":"table_p3","boiler":"hr107","temperatureLevel":"low"}}}],"auxiliary":{"distribution":{"method":"pumps","pumpPowersW":[5000,3000],"sourceReference":"x"}},"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["chp", "b"],
        &[
            ("/calculation/deliveredKwh", 4000000.0),
            ("/calculation/inputKwh", 4672919.435144627),
            ("/calculation/distributionLossKwh", 672919.4351446263),
            ("/distributionEfficiency", 0.8559959262118544),
            ("/generationPrimaryFactor", 1.0748237367802584),
            ("/calculation/auxiliaryElectricityKwh", 76972.21137865909),
            ("/calculation/auxiliary/distributionKwh", 70080.0),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 6892.211378659089),
            ("/factors/primaryFactor", 1.29),
            ("/factors/co2KgPerKwh", 0.30453971981129824),
            ("/factors/renewableFactor", 0.0),
            ("/calculation/monthlyInputKwh/0", 778624.6511274076),
            ("/calculation/monthlyInputKwh/1", 669340.1992910586),
            ("/calculation/monthlyInputKwh/2", 624431.5272319245),
            ("/calculation/monthlyInputKwh/3", 461954.87490515056),
            ("/calculation/monthlyInputKwh/4", 167270.90304677226),
            ("/calculation/monthlyInputKwh/5", 103798.15420442805),
            ("/calculation/monthlyInputKwh/6", 19424.175342465755),
            ("/calculation/monthlyInputKwh/7", 19424.175342465755),
            ("/calculation/monthlyInputKwh/8", 169859.75848808212),
            ("/calculation/monthlyInputKwh/9", 416548.177612699),
            ("/calculation/monthlyInputKwh/10", 527405.1864761077),
            ("/calculation/monthlyInputKwh/11", 714837.6520760644),
            ("/generators/0/energyFraction", 0.6),
            ("/generators/0/primaryFactor", 1.0706521739130435),
            ("/generators/0/co2KgPerKwh", 0.27),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 2803751.661086776),
            ("/generators/0/auxiliaryKwh", 3960.126827195453),
            ("/generators/1/energyFraction", 0.4),
            ("/generators/1/primaryFactor", 1.081081081081081),
            ("/generators/1/co2KgPerKwh", 0.23567567567567566),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 1869167.7740578507),
            ("/generators/1/efficiency", 0.925),
            ("/generators/1/auxiliaryKwh", 2932.0845514636353),
        ],
    );
}

/// Pipe losses per outdoor bin (P.15, table P.16) on a heating curve, off above 17 °C; CHP with power loss (P.28-P.30
/// with P.29 0,18; P.30 as printed with f_P;exp;el).
#[test]
fn annex_p_heat_pipes_bins_chp_loss() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":9000000,"distribution":{"method":"pipes","sourceReference":"x","otherLossKwh":0,"segments":[{"lengthM":3000,"layers":[{"conductivityWPerMk":0.027,"innerDiameterM":0.1143,"outerDiameterM":0.2},{"conductivityWPerMk":0.4,"innerDiameterM":0.2,"outerDiameterM":0.21}],"placement":{"kind":"buried","coverDepthM":0.8,"ambient":{"kind":"outdoor"}},"correction":"two_pipes_in_trench"},{"lengthM":400,"layers":[{"conductivityWPerMk":0.035,"innerDiameterM":0.06,"outerDiameterM":0.12}],"placement":{"kind":"buried","coverDepthM":0.5,"groundConductivity":1.2,"ambient":{"kind":"crawlspace"}},"correction":"single_pipe_in_trench"},{"lengthM":600,"layers":[{"conductivityWPerMk":0.04,"innerDiameterM":0.05,"outerDiameterM":0.11}],"placement":{"kind":"in_air","ambient":{"kind":"indoor","temperatureC":18}}},{"lengthM":50,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"outdoor"}},"correction":"old_sliding_system"},{"lengthM":120,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"crawlspace"},"surfaceCoefficient":12},"correctionFactor":0.95},{"lengthM":200,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"buried","coverDepthM":1,"ambient":{"kind":"outdoor"}},"resistanceKmPerW":2.5}],"waterTemperature":{"method":"outdoor_bins","curve":[{"outdoorC":-10,"waterC":90},{"outdoorC":0,"waterC":80},{"outdoorC":15,"waterC":65}],"offAboveOutdoorC":17},"buffers":[]},"generators":[{"id":"steg","energyFraction":1,"kind":{"kind":"chp_with_loss","carrier":{"kind":"natural_gas"}}}],"auxiliaryElectricityKwh":50000,"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["steg"],
        &[
            ("/calculation/deliveredKwh", 9000000.0),
            ("/calculation/inputKwh", 9547174.641411198),
            ("/calculation/distributionLossKwh", 547174.6414111983),
            ("/distributionEfficiency", 0.9426872701125829),
            ("/generationPrimaryFactor", 0.261),
            ("/calculation/auxiliaryElectricityKwh", 50000.0),
            ("/factors/primaryFactor", 0.29),
            ("/factors/co2KgPerKwh", 0.06184612697189048),
            ("/factors/renewableFactor", 0.0),
            ("/generators/0/energyFraction", 1.0),
            ("/generators/0/primaryFactor", 0.261),
            ("/generators/0/co2KgPerKwh", 0.056898),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 9547174.641411198),
        ],
    );
}

/// CHP variants: table P.6 rows 2-20 kW up to 2006, 200-500 kW HT after 2006, 1-25 MW on biogas (5.45), declared
/// conversion numbers on a 30 % biofuel mix, and a waste incinerator with power loss (declared ratio 0,25, table 5.5/5.6).
/// P.27 without MAX(0) gives a negative K for biogas (kernel warning).
#[test]
fn annex_p_chp_variants() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":6000000,"distribution":{"method":"flows","lossKwh":600000,"sourceReference":"x"},"generators":[{"id":"small","energyFraction":0.2,"kind":{"kind":"chp_without_loss","carrier":{"kind":"natural_gas"},"tableP6":{"electricalPowerKw":15,"installedAfter2006":false}}},{"id":"mid","energyFraction":0.2,"kind":{"kind":"chp_without_loss","carrier":{"kind":"natural_gas"},"tableP6":{"electricalPowerKw":300,"installedAfter2006":true}}},{"id":"decl","energyFraction":0.2,"kind":{"kind":"chp_without_loss","carrier":{"kind":"biofuel_mix","biofuelShare":0.3},"thermalEfficiency":0.45,"electricalEfficiency":0.4,"efficiencyReference":"x"}},{"id":"bio","energyFraction":0.1,"kind":{"kind":"chp_without_loss","carrier":{"kind":"biogas"},"tableP6":{"electricalPowerKw":1500,"installedAfter2006":true}}},{"id":"avi","energyFraction":0.3,"kind":{"kind":"chp_with_loss","carrier":{"kind":"waste_incineration"},"lossRatio":0.25,"lossRatioReference":"x"}}],"auxiliaryElectricityKwh":40000,"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["small", "mid", "decl", "bio", "avi"],
        &[
            ("/calculation/deliveredKwh", 6000000.0),
            ("/calculation/inputKwh", 6600000.0),
            ("/calculation/distributionLossKwh", 600000.0),
            ("/distributionEfficiency", 0.9090909090909091),
            ("/generationPrimaryFactor", 0.5407048245614036),
            ("/calculation/auxiliaryElectricityKwh", 40000.0),
            ("/factors/primaryFactor", 0.61),
            ("/factors/co2KgPerKwh", 0.12795418288349078),
            ("/factors/renewableFactor", 0.35),
            ("/generators/0/energyFraction", 0.2),
            ("/generators/0/primaryFactor", 1.092982456140351),
            ("/generators/0/co2KgPerKwh", 0.2602105263157895),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 1320000.0),
            ("/generators/1/energyFraction", 0.2),
            ("/generators/1/primaryFactor", 1.072),
            ("/generators/1/co2KgPerKwh", 0.26448),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 1320000.0),
            ("/generators/2/energyFraction", 0.2),
            ("/generators/2/primaryFactor", 0.26666666666666666),
            ("/generators/2/co2KgPerKwh", 0.10088888888888882),
            ("/generators/2/renewableFactor", 0.3),
            ("/generators/2/heatKwh", 1320000.0),
            ("/generators/3/energyFraction", 0.1),
            ("/generators/3/primaryFactor", 0.0),
            ("/generators/3/co2KgPerKwh", -0.25425641025641027),
            ("/generators/3/renewableFactor", 1.0),
            ("/generators/3/heatKwh", 660000.0),
            ("/generators/4/energyFraction", 0.3),
            ("/generators/4/primaryFactor", 0.18125),
            ("/generators/4/co2KgPerKwh", 0.050025),
            ("/generators/4/renewableFactor", 0.5),
            ("/generators/4/heatKwh", 1980000.0),
        ],
    );
}

/// Electrode boiler in flex mode (P.6.5.4.11, 5.8, tables 5.5/5.6, 5.8.3.1 g): 1 500 h at 50 MW capped at 15 % of
/// the network heat; η 0,99.
#[test]
fn annex_p_flex_electrode_forfait_cap() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":180000000,"distribution":{"method":"flows","lossKwh":20000000,"sourceReference":"x"},"generators":[{"id":"eb","energyFraction":0.3,"nominalPowerKw":50000,"kind":{"kind":"electric_flex","generator":{"kind":"electrode_boiler"},"connections":1200,"heatBuffer":true,"registrationReference":"x"}},{"id":"b","energyFraction":0.7,"kind":{"kind":"combustion","carrier":{"kind":"natural_gas"},"efficiency":0.92,"efficiencyReference":"x"}}],"auxiliaryElectricityKwh":1000000,"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["eb", "b"],
        &[
            ("/calculation/deliveredKwh", 180000000.0),
            ("/calculation/inputKwh", 200000000.0),
            ("/calculation/distributionLossKwh", 20000000.0),
            ("/distributionEfficiency", 0.9),
            ("/generationPrimaryFactor", 0.980566534914361),
            ("/calculation/auxiliaryElectricityKwh", 1000000.0),
            ("/factors/primaryFactor", 1.1),
            ("/factors/co2KgPerKwh", 0.23090625091494657),
            ("/factors/renewableFactor", 0.13),
            ("/generators/0/energyFraction", 0.3),
            ("/generators/0/primaryFactor", 0.7323232323232324),
            ("/generators/0/co2KgPerKwh", 0.13535353535353536),
            ("/generators/0/renewableFactor", 0.4949494949494949),
            ("/generators/0/heatKwh", 60000000.0),
            ("/generators/0/efficiency", 0.99),
            ("/generators/1/energyFraction", 0.7),
            ("/generators/1/primaryFactor", 1.0869565217391304),
            ("/generators/1/co2KgPerKwh", 0.23695652173913043),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 140000000.0),
            ("/generators/1/efficiency", 0.92),
        ],
    );
}

/// Flex mode with a declared network production for the 15 % cap and a heat pump with registered flex heat; 25 %
/// renewable auxiliary electricity (5.42).
#[test]
fn annex_p_flex_network_production_and_hp() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":180000000,"distribution":{"method":"flows","lossKwh":20000000,"sourceReference":"x"},"generators":[{"id":"eb","energyFraction":0.25,"nominalPowerKw":30000,"kind":{"kind":"electric_flex","generator":{"kind":"electrode_boiler","efficiency":0.98,"efficiencyReference":"x"},"networkProductionKwh":400000000,"connections":800,"heatBuffer":true,"registrationReference":"x"}},{"id":"hp","energyFraction":0.35,"nominalPowerKw":20000,"kind":{"kind":"electric_flex","generator":{"kind":"heat_pump","efficiency":{"method":"declared","value":3.2,"sourceReference":"x"}},"flexHeatKwh":9000000,"flexReference":"x","connections":800,"heatBuffer":true,"registrationReference":"x"}},{"id":"b","energyFraction":0.4,"kind":{"kind":"combustion","carrier":{"kind":"natural_gas"},"efficiency":0.92,"efficiencyReference":"x"}}],"auxiliaryElectricityKwh":1000000,"auxiliaryRenewableShare":0.25,"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["eb", "hp", "b"],
        &[
            ("/calculation/deliveredKwh", 180000000.0),
            ("/calculation/inputKwh", 200000000.0),
            ("/calculation/distributionLossKwh", 20000000.0),
            ("/distributionEfficiency", 0.9),
            ("/generationPrimaryFactor", 0.6099755296140195),
            ("/calculation/auxiliaryElectricityKwh", 1000000.0),
            ("/factors/primaryFactor", 0.69),
            ("/factors/co2KgPerKwh", 0.14240899265503304),
            ("/factors/renewableFactor", 0.43),
            ("/generators/0/energyFraction", 0.25),
            ("/generators/0/primaryFactor", 0.14795918367346939),
            ("/generators/0/co2KgPerKwh", 0.027346938775510202),
            ("/generators/0/renewableFactor", 0.8979591836734694),
            ("/generators/0/heatKwh", 50000000.0),
            ("/generators/0/efficiency", 0.98),
            ("/generators/1/energyFraction", 0.35),
            ("/generators/1/primaryFactor", 0.39486607142857144),
            ("/generators/1/co2KgPerKwh", 0.07298214285714287),
            ("/generators/1/renewableFactor", 0.7276785714285714),
            ("/generators/1/heatKwh", 70000000.0),
            ("/generators/1/efficiency", 3.2),
            ("/generators/2/energyFraction", 0.4),
            ("/generators/2/primaryFactor", 1.0869565217391304),
            ("/generators/2/co2KgPerKwh", 0.23695652173913043),
            ("/generators/2/renewableFactor", 0.0),
            ("/generators/2/heatKwh", 80000000.0),
            ("/generators/2/efficiency", 0.92),
        ],
    );
}

/// Derived fractions with heat pump and CHP preferred (P.6.5.3.2 d), table P.2 with P.24/P.25), constant 70 °C pipe
/// losses, table P.11 primary network over 3 km (0,0018·L), defaults of P.6.8.4.3 (10 W/kW source pump for a declared
/// heat pump, 1 W/kW gas, 10 W/kW solid biomass; P.6.5.4.3: η/1,08 − 0,05).
#[test]
fn annex_p_derived_hp_chp_aux_defaults() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":3000000,"distribution":{"method":"pipes","sourceReference":"x","otherLossKwh":0,"segments":[{"lengthM":3000,"layers":[{"conductivityWPerMk":0.027,"innerDiameterM":0.1143,"outerDiameterM":0.2},{"conductivityWPerMk":0.4,"innerDiameterM":0.2,"outerDiameterM":0.21}],"placement":{"kind":"buried","coverDepthM":0.8,"ambient":{"kind":"outdoor"}},"correction":"two_pipes_in_trench"},{"lengthM":400,"layers":[{"conductivityWPerMk":0.035,"innerDiameterM":0.06,"outerDiameterM":0.12}],"placement":{"kind":"buried","coverDepthM":0.5,"groundConductivity":1.2,"ambient":{"kind":"crawlspace"}},"correction":"single_pipe_in_trench"},{"lengthM":600,"layers":[{"conductivityWPerMk":0.04,"innerDiameterM":0.05,"outerDiameterM":0.11}],"placement":{"kind":"in_air","ambient":{"kind":"indoor","temperatureC":18}}},{"lengthM":50,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"outdoor"}},"correction":"old_sliding_system"},{"lengthM":120,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"crawlspace"},"surfaceCoefficient":12},"correctionFactor":0.95},{"lengthM":200,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"buried","coverDepthM":1,"ambient":{"kind":"outdoor"}},"resistanceKmPerW":2.5}],"waterTemperature":{"method":"constant","temperatureC":70},"buffers":[]},"generators":[{"id":"hp","nominalPowerKw":300,"kind":{"kind":"heat_pump","efficiency":{"method":"declared","value":3.5,"sourceReference":"x"},"drive":{"kind":"electricity","directRenewableShare":0.1}}},{"id":"chp","nominalPowerKw":250,"kind":{"kind":"chp_without_loss","carrier":{"kind":"natural_gas"},"tableP6":{"electricalPowerKw":180,"installedAfter2006":true}}},{"id":"b1","nominalPowerKw":1500,"kind":{"kind":"combustion","carrier":{"kind":"natural_gas"},"efficiency":0.95,"efficiencyReference":"x"}},{"id":"b2","nominalPowerKw":900,"kind":{"kind":"solid_biomass_boiler","carrier":{"kind":"biomass_above500_kw"},"netEfficiency":0.9,"sourceReference":"x"}}],"auxiliary":{"distribution":{"method":"forfait","network":"primary","farthestDistanceKm":5}},"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["hp", "chp", "b1", "b2"],
        &[
            ("/calculation/deliveredKwh", 3000000.0),
            ("/calculation/inputKwh", 3627565.4621161735),
            ("/calculation/distributionLossKwh", 627565.4621161732),
            ("/distributionEfficiency", 0.827000927021155),
            ("/generationPrimaryFactor", 0.09874078070901776),
            ("/calculation/auxiliaryElectricityKwh", 72919.68550941366),
            ("/calculation/auxiliary/distributionKwh", 32648.089159045558),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 40271.596350368105),
            ("/calculation/beta", 0.3721504171595197),
            ("/factors/primaryFactor", 0.16),
            ("/factors/co2KgPerKwh", 0.032762836992459396),
            ("/factors/renewableFactor", 0.87),
            ("/calculation/monthlyInputKwh/0", 580649.2628699197),
            ("/calculation/monthlyInputKwh/1", 498669.36398843414),
            ("/calculation/monthlyInputKwh/2", 466513.28245759703),
            ("/calculation/monthlyInputKwh/3", 346823.23046313657),
            ("/calculation/monthlyInputKwh/4", 161458.93481011645),
            ("/calculation/monthlyInputKwh/5", 111782.41018364535),
            ("/calculation/monthlyInputKwh/6", 48326.7111797827),
            ("/calculation/monthlyInputKwh/7", 48035.66103628732),
            ("/calculation/monthlyInputKwh/8", 128719.17517437339),
            ("/calculation/monthlyInputKwh/9", 311219.1757753762),
            ("/calculation/monthlyInputKwh/10", 392794.4497236841),
            ("/calculation/monthlyInputKwh/11", 532573.8044538201),
            ("/generators/0/energyFraction", 0.014967756039861113),
            ("/generators/0/primaryFactor", 0.37285714285714283),
            ("/generators/0/co2KgPerKwh", 0.06891428571428572),
            ("/generators/0/renewableFactor", 0.7142857142857143),
            ("/generators/0/heatKwh", 54296.51485558093),
            ("/generators/0/efficiency", 3.5),
            ("/generators/0/auxiliaryKwh", 1473.2616634113897),
            ("/generators/1/energyFraction", 0.012473130033217594),
            ("/generators/1/primaryFactor", 1.1530612244897958),
            ("/generators/1/co2KgPerKwh", 0.2808163265306122),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 45247.095712984104),
            ("/generators/1/auxiliaryKwh", 925.7718052842824),
            ("/generators/2/energyFraction", 0.07483878019930557),
            ("/generators/2/primaryFactor", 1.0526315789473684),
            ("/generators/2/co2KgPerKwh", 0.22947368421052633),
            ("/generators/2/renewableFactor", 0.0),
            ("/generators/2/heatKwh", 271482.57427790464),
            ("/generators/2/efficiency", 0.95),
            ("/generators/2/auxiliaryKwh", 1174.6308317056948),
            ("/generators/3/energyFraction", 0.8977203337276157),
            ("/generators/3/primaryFactor", 0.0),
            ("/generators/3/co2KgPerKwh", 0.0),
            ("/generators/3/renewableFactor", 1.0),
            ("/generators/3/heatKwh", 3256539.277269704),
            ("/generators/3/efficiency", 0.7833333333333332),
            ("/generators/3/auxiliaryKwh", 36697.93204996674),
        ],
    );
}

/// Area demand for heat (P.72-P.74, P.79-P.83, tables P.14/P.15): monthly and annual plots, hot water via a delivery
/// set, sorption cooling, a plot whose hot water is not on the heat network; table P.11 primary and secondary net.
#[test]
fn annex_p_area_heating() {
    annex_p(
        r#"{"method":"calculated","function":"heating","areaDemand":{"plots":[{"id":"a","heating":{"monthlyKwh":[9000,8000,7000,5000,2000,500,0,0,1500,4000,6500,8500]},"sourceReference":"x"},{"id":"b","heating":{"annualKwh":40000},"hotWater":{"annualKwh":12000},"hotWaterViaDeliverySet":true,"sourceReference":"x"},{"id":"c","usableAreaM2":120,"heatingForfait":"terraced_or_utility","hotWaterForfait":"dwelling_low_temperature","hotWaterViaDeliverySet":true,"sourceReference":"x"},{"id":"d","usableAreaM2":900,"heatingForfait":"corner_or_semi_detached","sorptionCooling":{"annualKwh":7000},"sourceReference":"x"},{"id":"e","heating":{"annualKwh":15000,"monthlyKwh":[]},"hotWater":{"annualKwh":5000},"sourceReference":"x"}]},"distribution":{"method":"flows","lossKwh":25000,"sourceReference":"x"},"generators":[{"id":"b","energyFraction":1,"nominalPowerKw":200,"kind":{"kind":"combustion","carrier":{"kind":"natural_gas"},"efficiency":0.9,"efficiencyReference":"x"}}],"auxiliary":{"distribution":{"method":"forfait","network":"primary_and_secondary","farthestDistanceKm":2.5}},"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["b"],
        &[
            ("/calculation/deliveredKwh", 168073.8),
            ("/calculation/inputKwh", 193073.8),
            ("/calculation/distributionLossKwh", 25000.0),
            ("/distributionEfficiency", 0.8705158338417744),
            ("/generationPrimaryFactor", 1.1111111111111112),
            ("/calculation/auxiliaryElectricityKwh", 2478.5125399999997),
            ("/calculation/auxiliary/distributionKwh", 1390.1313599999999),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 1088.3811799999999),
            ("/factors/primaryFactor", 1.3),
            ("/factors/co2KgPerKwh", 0.2822034502082353),
            ("/factors/renewableFactor", 0.0),
            ("/calculation/monthlyInputKwh/0", 30756.310966429297),
            ("/calculation/monthlyInputKwh/1", 25909.657370860175),
            ("/calculation/monthlyInputKwh/2", 24046.062611054593),
            ("/calculation/monthlyInputKwh/3", 18033.365977167403),
            ("/calculation/monthlyInputKwh/4", 9349.584279416751),
            ("/calculation/monthlyInputKwh/5", 5146.3087600316485),
            ("/calculation/monthlyInputKwh/6", 3439.76),
            ("/calculation/monthlyInputKwh/7", 3439.76),
            ("/calculation/monthlyInputKwh/8", 7388.502000678196),
            ("/calculation/monthlyInputKwh/9", 15371.059242681134),
            ("/calculation/monthlyInputKwh/10", 21708.633344636597),
            ("/calculation/monthlyInputKwh/11", 28484.795447044195),
            ("/generators/0/energyFraction", 1.0),
            ("/generators/0/primaryFactor", 1.1111111111111112),
            ("/generators/0/co2KgPerKwh", 0.24222222222222223),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 193073.8),
            ("/generators/0/efficiency", 0.9),
            ("/generators/0/auxiliaryKwh", 1088.3811799999999),
        ],
    );
}

/// Collective hot water: area demand (P.75/P.76, P.81-P.83, table P.15), η_WD;gen;sto from P.35 with P.43/P.44 (V/200,
/// α 10; measured standby at 45 K), P.45 with table P.7 and f_x 1,20, P.46 (0,2 W/kW); pipes at 65 °C (P.6.4); P.40-P.42
/// with CHP preferred; table P.12 and P.64/P.65 (no standby for a generator that also heats, P.6.9.4.3).
#[test]
fn annex_p_hot_water_calculated_storage() {
    annex_p(
        r#"{"method":"calculated","function":"hot_water","areaDemand":{"plots":[{"id":"o","usableAreaM2":2000,"hotWaterForfait":"office","sourceReference":"x"},{"id":"l","usableAreaM2":800,"hotWaterForfait":"lodging","sourceReference":"x"},{"id":"m","hotWater":{"monthlyKwh":[4000,3700,4000,3800,3700,3500,3400,3400,3600,3900,3900,4100]},"sourceReference":"x"},{"id":"y","hotWater":{"annualKwh":20000},"sourceReference":"x"}]},"hotWaterStorage":{"method":"calculated","sourceReference":"x","vessels":[{"volumeL":1500,"diameterAtLeast50Cm":false,"insulation":"none"},{"volumeL":3000,"diameterAtLeast50Cm":true,"standbyLossKwhPerDay":4.2,"standbyTestDifferenceK":45,"waterTemperatureC":65,"ambientTemperatureC":15}],"pipes":[{"lengthM":10,"outerDiameterMm":22,"insulationMm":15},{"lengthM":25,"outerDiameterMm":35,"insulationMm":20,"ambientTemperatureC":15},{"lengthM":4,"uValueWPerMk":0.3}],"exchanger":{"nominalPowerKw":300,"insulated":true}},"distribution":{"method":"pipes","sourceReference":"x","otherLossKwh":0,"segments":[{"lengthM":800,"layers":[{"conductivityWPerMk":0.035,"innerDiameterM":0.028,"outerDiameterM":0.068}],"placement":{"kind":"in_air","ambient":{"kind":"indoor","temperatureC":20}}},{"lengthM":150,"layers":[{"conductivityWPerMk":0.035,"innerDiameterM":0.042,"outerDiameterM":0.102}],"placement":{"kind":"in_air","ambient":{"kind":"crawlspace"}},"correction":"surface_or_recessed"}]},"generators":[{"id":"chp","nominalPowerKw":60,"kind":{"kind":"chp_without_loss","carrier":{"kind":"natural_gas"},"tableP6":{"electricalPowerKw":40,"installedAfter2006":true}}},{"id":"b","nominalPowerKw":400,"auxiliary":{"alsoServesHeating":true},"kind":{"kind":"boiler","carrier":{"kind":"natural_gas"},"efficiency":{"method":"table_p3","boiler":"hr104"}}}],"auxiliary":{"distribution":{"method":"forfait"}},"sourceReference":"x"}"#,
        SystemFunction::HotWater,
        &["chp", "b"],
        &[
            ("/calculation/deliveredKwh", 77780.0),
            ("/calculation/inputKwh", 171093.84850570245),
            ("/calculation/distributionLossKwh", 93313.84850570244),
            ("/distributionEfficiency", 0.454604304475667),
            ("/storageEfficiency", 0.8046451638913047),
            ("/generationPrimaryFactor", 1.433005846842518),
            ("/calculation/auxiliaryElectricityKwh", 1372.172160666537),
            ("/calculation/auxiliary/distributionKwh", 307.9689273102644),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 1064.2032333562724),
            ("/factors/primaryFactor", 3.18),
            ("/factors/co2KgPerKwh", 0.7724153644468189),
            ("/factors/renewableFactor", 0.0),
            ("/calculation/monthlyInputKwh/0", 14727.482331306406),
            ("/calculation/monthlyInputKwh/1", 13384.863846311277),
            ("/calculation/monthlyInputKwh/2", 14720.084206951717),
            ("/calculation/monthlyInputKwh/3", 14166.876914630839),
            ("/calculation/monthlyInputKwh/4", 14400.311038221906),
            ("/calculation/monthlyInputKwh/5", 13852.124056680135),
            ("/calculation/monthlyInputKwh/6", 14092.868076749914),
            ("/calculation/monthlyInputKwh/7", 14091.904078727941),
            ("/calculation/monthlyInputKwh/8", 13953.187130267761),
            ("/calculation/monthlyInputKwh/9", 14610.018274117607),
            ("/calculation/monthlyInputKwh/10", 14269.762400082964),
            ("/calculation/monthlyInputKwh/11", 14824.366151653976),
            ("/generators/0/energyFraction", 1.0),
            ("/generators/0/primaryFactor", 1.1530612244897958),
            ("/generators/0/co2KgPerKwh", 0.2808163265306122),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 212632.6686390358),
            ("/generators/0/auxiliaryKwh", 1064.2032333562724),
            ("/generators/1/energyFraction", 0.0),
            ("/generators/1/primaryFactor", 1.1428571428571428),
            ("/generators/1/co2KgPerKwh", 0.24914285714285714),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 0.0),
            ("/generators/1/efficiency", 0.875),
            ("/generators/1/auxiliaryKwh", 0.0),
        ],
    );
}

/// Hot water with forfait η_WD;gen;sto 0,80 (P.6.6.4.3), a non-preferred full-load boiler less 5 points (P.6.6.5.2)
/// without auxiliary energy, and pumps with 6 000 h (P.62).
#[test]
fn annex_p_hot_water_forfait_storage() {
    annex_p(
        r#"{"method":"calculated","function":"hot_water","deliveredKwh":500000,"hotWaterStorage":{"method":"forfait","insulation":"at_least10_mm"},"distribution":{"method":"flows","lossKwh":80000,"sourceReference":"x"},"generators":[{"id":"b","energyFraction":0.7,"nominalPowerKw":250,"auxiliary":{"withoutAuxiliaryEnergy":true},"kind":{"kind":"boiler","carrier":{"kind":"natural_gas"},"efficiency":{"method":"full_load","value":0.96,"sourceReference":"x"}}},{"id":"hp","energyFraction":0.3,"nominalPowerKw":80,"kind":{"kind":"heat_pump","efficiency":{"method":"declared","value":2.8,"sourceReference":"x","sourcePumpIncluded":true},"drive":{"kind":"electricity"}}}],"auxiliary":{"distribution":{"method":"pumps","pumpPowersW":[400,250],"operatingHours":6000,"sourceReference":"x"}},"sourceReference":"x"}"#,
        SystemFunction::HotWater,
        &["b", "hp"],
        &[
            ("/calculation/deliveredKwh", 500000.0),
            ("/calculation/inputKwh", 580000.0),
            ("/calculation/distributionLossKwh", 80000.0),
            ("/distributionEfficiency", 0.8620689655172413),
            ("/storageEfficiency", 0.8),
            ("/generationPrimaryFactor", 1.15573489010989),
            ("/calculation/auxiliaryElectricityKwh", 4776.0),
            ("/calculation/auxiliary/distributionKwh", 3900.0),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 875.9999999999999),
            ("/factors/primaryFactor", 1.36),
            ("/factors/co2KgPerKwh", 0.2873494964395604),
            ("/factors/renewableFactor", 0.17),
            ("/generators/0/energyFraction", 0.7),
            ("/generators/0/primaryFactor", 1.098901098901099),
            ("/generators/0/co2KgPerKwh", 0.23956043956043957),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 507500.0),
            ("/generators/0/efficiency", 0.9099999999999999),
            ("/generators/0/auxiliaryKwh", 0.0),
            ("/generators/1/energyFraction", 0.3),
            ("/generators/1/primaryFactor", 0.5178571428571429),
            ("/generators/1/co2KgPerKwh", 0.09571428571428572),
            ("/generators/1/renewableFactor", 0.6428571428571428),
            ("/generators/1/heatKwh", 217500.0),
            ("/generators/1/efficiency", 2.8),
            ("/generators/1/auxiliaryKwh", 875.9999999999999),
        ],
    );
}

/// Hot water with supplied storage and pipe losses (P.35) and table P.0 within one building at 90/60.
#[test]
fn annex_p_hot_water_losses_storage() {
    annex_p(
        r#"{"method":"calculated","function":"hot_water","deliveredKwh":300000,"hotWaterStorage":{"method":"losses","storageLossKwh":15000,"pipeLossKwh":6000,"sourceReference":"x"},"distribution":{"method":"small_system_forfait","connections":60,"connectionType":"within_building"},"generators":[{"id":"b","energyFraction":1,"nominalPowerKw":150,"kind":{"kind":"combustion","carrier":{"kind":"natural_gas"},"efficiency":0.88,"efficiencyReference":"x"}}],"auxiliaryElectricityKwh":3000,"sourceReference":"x"}"#,
        SystemFunction::HotWater,
        &["b"],
        &[
            ("/calculation/deliveredKwh", 300000.0),
            ("/calculation/inputKwh", 405000.0),
            ("/calculation/distributionLossKwh", 105000.0),
            ("/distributionEfficiency", 0.7407407407407407),
            ("/storageEfficiency", 0.9507042253521126),
            ("/generationPrimaryFactor", 1.1952861952861955),
            ("/calculation/auxiliaryElectricityKwh", 3000.0),
            ("/factors/primaryFactor", 1.63),
            ("/factors/co2KgPerKwh", 0.3544527272727273),
            ("/factors/renewableFactor", 0.0),
            ("/generators/0/energyFraction", 1.0),
            ("/generators/0/primaryFactor", 1.1363636363636365),
            ("/generators/0/co2KgPerKwh", 0.2477272727272727),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 426000.0),
            ("/generators/0/efficiency", 0.88),
        ],
    );
}

/// Cold network: area demand with dehumidification (P.77/P.78), pipe gains below 10 °C (P.14, months without water
/// temperature), table P.9 chillers (HT + wet 5, gas engine LT source 6·η_ge with table P.6), aquifer from 2013 (23)
/// preferred (P.6.7.3.2 a), β from P.53/P.55 with table P.8 and P.51; P.68 pumps in months with cold, P.69/P.70 with
/// table P.13 (dry cooler 45, open tower 18 W/kW) and the standby of P.6.10.4.3.
#[test]
fn annex_p_cold_derived_chillers() {
    annex_p(
        r#"{"method":"calculated","function":"cooling","areaDemand":{"plots":[{"id":"k","cooling":{"monthlyKwh":[0,0,5000,20000,60000,110000,150000,140000,80000,20000,0,0]},"dehumidification":{"annualKwh":30000},"sourceReference":"x"},{"id":"k2","cooling":{"monthlyKwh":[0,0,0,10000,30000,50000,70000,60000,30000,5000,0,0]},"sourceReference":"x"}]},"distribution":{"method":"pipes","sourceReference":"x","supplyBelow10C":true,"otherLossKwh":2000,"waterTemperature":{"method":"monthly","temperaturesC":[null,null,8,8,7,6,6,6,7,8,null,null]},"segments":[{"lengthM":1500,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.15,"outerDiameterM":0.25}],"placement":{"kind":"buried","coverDepthM":1,"ambient":{"kind":"outdoor"}},"correction":"two_pipes_in_trench_rigid"},{"lengthM":300,"layers":[{"conductivityWPerMk":0.035,"innerDiameterM":0.1,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"indoor","temperatureC":24}}}]},"generators":[{"id":"el","coolingPower":{"method":"compressor_shaft","shaftPowerKw":120},"auxiliary":{"heatRejection":"dry_cooler"},"kind":{"kind":"compression_chiller","variant":"high_temperature_emission_and_wet_cooling","drive":{"kind":"electricity"}}},{"id":"ge","coolingPower":{"method":"compressor_shaft","shaftPowerKw":80},"auxiliary":{"heatRejection":"open_cooling_tower","standbyW":25,"sourceReference":"x"},"kind":{"kind":"compression_chiller","variant":"low_temperature_source","drive":{"kind":"natural_gas"},"engineEfficiency":{"method":"table_p6","electricalPowerKw":80,"installedAfter2006":true}}},{"id":"aq","coolingPower":{"method":"aquifer","flowM3PerS":0.02,"supplyC":8,"returnC":16},"kind":{"kind":"free_cooling","source":"aquifer_storage_from2013","drive":{"kind":"electricity"}}}],"auxiliary":{"distribution":{"method":"pumps_monthly","pumpPowersW":[7500,2200],"sourceReference":"x"}},"sourceReference":"x"}"#,
        SystemFunction::Cooling,
        &["el", "ge", "aq"],
        &[
            ("/calculation/deliveredKwh", 870000.0),
            ("/calculation/inputKwh", 905553.5190067794),
            ("/calculation/distributionLossKwh", 35553.51900677937),
            ("/distributionEfficiency", 0.960738356971132),
            ("/generationPrimaryFactor", 0.19258486979680312),
            ("/calculation/auxiliaryElectricityKwh", 72012.90342837186),
            ("/calculation/auxiliary/distributionKwh", 57036.00000000001),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 14976.903428371861),
            ("/calculation/beta", 0.3829981718464351),
            ("/factors/primaryFactor", 0.33),
            ("/factors/co2KgPerKwh", 0.0624342379743993),
            ("/factors/renewableFactor", 0.6632715025363547),
            ("/calculation/monthlyInputKwh/0", 169.86301369863014),
            ("/calculation/monthlyInputKwh/1", 153.42465753424656),
            ("/calculation/monthlyInputKwh/2", 6045.543928550386),
            ("/calculation/monthlyInputKwh/3", 32255.09482579314),
            ("/calculation/monthlyInputKwh/4", 94849.18136458682),
            ("/calculation/monthlyInputKwh/5", 165664.7347815032),
            ("/calculation/monthlyInputKwh/6", 226580.6885993826),
            ("/calculation/monthlyInputKwh/7", 206742.69150602506),
            ("/calculation/monthlyInputKwh/8", 115020.89407970864),
            ("/calculation/monthlyInputKwh/9", 27737.155674654106),
            ("/calculation/monthlyInputKwh/10", 164.3835616438356),
            ("/calculation/monthlyInputKwh/11", 169.86301369863014),
            ("/generators/0/energyFraction", 0.20861263457241516),
            ("/generators/0/primaryFactor", 0.29),
            ("/generators/0/co2KgPerKwh", 0.0536),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 188909.9053463259),
            ("/generators/0/efficiency", 5.0),
            ("/generators/0/auxiliaryKwh", 11280.048377571758),
            ("/generators/1/energyFraction", 0.16689010765793214),
            ("/generators/1/primaryFactor", 0.5555555555555556),
            ("/generators/1/co2KgPerKwh", 0.12111111111111113),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 151127.92427706072),
            ("/generators/1/efficiency", 1.7999999999999998),
            ("/generators/1/auxiliaryKwh", 3638.0550508001033),
            ("/generators/2/energyFraction", 0.6244972577696527),
            ("/generators/2/primaryFactor", 0.06304347826086956),
            ("/generators/2/co2KgPerKwh", 0.01165217391304348),
            ("/generators/2/renewableFactor", 1.0),
            ("/generators/2/heatKwh", 565515.6893833928),
            ("/generators/2/efficiency", 23.0),
            ("/generators/2/auxiliaryKwh", 58.8),
        ],
    );
}

/// Cold with supplied fractions: sorption chillers on collective heat and on CHP (table P.10), other free source
/// (table P.9: 23) with 50 % direct renewable drive electricity (5.49), gas-engine chiller with declared η_ge;
/// 15 % forfait loss (P.6.4) and 0,009 kWh/kWh (P.6.10.3.3).
#[test]
fn annex_p_cold_supplied_sorption() {
    annex_p(
        r#"{"method":"calculated","function":"cooling","deliveredKwh":900000,"distribution":{"method":"small_cold_forfait","supplyBelow10C":true},"generators":[{"id":"sh","energyFraction":0.3,"nominalPowerKw":300,"kind":{"kind":"sorption_chiller","heat":{"source":"collective_heat","primaryFactor":0.62,"co2KgPerKwh":0.09,"sourceReference":"x"}}},{"id":"sc","energyFraction":0.2,"nominalPowerKw":200,"kind":{"kind":"sorption_chiller","heat":{"source":"chp","carrier":{"kind":"natural_gas"},"tableP6":{"electricalPowerKw":600,"installedAfter2006":false}}}},{"id":"fr","energyFraction":0.5,"nominalPowerKw":500,"kind":{"kind":"free_cooling","source":"other_low_temperature_source","drive":{"kind":"electricity","directRenewableShare":0.5}}},{"id":"ge","energyFraction":0,"nominalPowerKw":100,"kind":{"kind":"compression_chiller","variant":"high_temperature_emission_and_low_temperature_source","drive":{"kind":"natural_gas"},"engineEfficiency":{"method":"declared","value":0.34,"sourceReference":"x"}}}],"auxiliary":{"distribution":{"method":"forfait"}},"sourceReference":"x"}"#,
        SystemFunction::Cooling,
        &["sh", "sc", "fr", "ge"],
        &[
            ("/calculation/deliveredKwh", 900000.0),
            ("/calculation/inputKwh", 1035000.0),
            ("/calculation/distributionLossKwh", 135000.0),
            ("/distributionEfficiency", 0.8695652173913043),
            ("/generationPrimaryFactor", 0.5053387916431396),
            ("/calculation/auxiliaryElectricityKwh", 9665.4),
            ("/calculation/auxiliary/distributionKwh", 9315.0),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 350.4),
            ("/factors/primaryFactor", 0.6),
            ("/factors/co2KgPerKwh", 0.11550801146320348),
            ("/factors/renewableFactor", 0.4971189104243059),
            ("/generators/0/energyFraction", 0.3),
            ("/generators/0/primaryFactor", 0.8857142857142858),
            ("/generators/0/co2KgPerKwh", 0.1285714285714286),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 310500.0),
            ("/generators/0/auxiliaryKwh", 87.6),
            ("/generators/1/energyFraction", 0.2),
            ("/generators/1/primaryFactor", 1.1193181818181819),
            ("/generators/1/co2KgPerKwh", 0.2822727272727273),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 207000.0),
            ("/generators/1/auxiliaryKwh", 87.6),
            ("/generators/2/energyFraction", 0.5),
            ("/generators/2/primaryFactor", 0.03152173913043478),
            ("/generators/2/co2KgPerKwh", 0.00582608695652174),
            ("/generators/2/renewableFactor", 1.0),
            ("/generators/2/heatKwh", 517500.0),
            ("/generators/2/efficiency", 23.0),
            ("/generators/2/auxiliaryKwh", 87.6),
            ("/generators/3/energyFraction", 0.0),
            ("/generators/3/primaryFactor", 0.4201680672268907),
            ("/generators/3/co2KgPerKwh", 0.09159663865546216),
            ("/generators/3/renewableFactor", 0.0),
            ("/generators/3/heatKwh", 0.0),
            ("/generators/3/efficiency", 2.3800000000000003),
            ("/generators/3/auxiliaryKwh", 87.6),
        ],
    );
}

/// Collective solar collector with declared monthly contributions: F = ΣQ_sol/Q_in (P.32), factor 0, the boiler
/// takes the rest; W_sol supplied (P.56).
#[test]
fn annex_p_solar_declared() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":2000000,"distribution":{"method":"flows","lossKwh":200000,"sourceReference":"x"},"generators":[{"id":"sol","kind":{"kind":"collective_solar","contribution":{"method":"declared","monthlyKwh":[2000,5000,12000,20000,28000,30000,31000,27000,18000,9000,3000,1500],"sourceReference":"x"}}},{"id":"b","nominalPowerKw":1200,"kind":{"kind":"combustion","carrier":{"kind":"natural_gas"},"efficiency":0.93,"efficiencyReference":"x"}}],"auxiliary":{"distribution":{"method":"pumps","pumpPowersW":[2000],"sourceReference":"x"},"solarKwh":3500},"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["sol", "b"],
        &[
            ("/calculation/deliveredKwh", 2000000.0),
            ("/calculation/inputKwh", 2200000.0),
            ("/calculation/distributionLossKwh", 200000.0),
            ("/distributionEfficiency", 0.9090909090909091),
            ("/generationPrimaryFactor", 0.9841153470185728),
            ("/calculation/auxiliaryElectricityKwh", 24110.85),
            ("/calculation/auxiliary/distributionKwh", 17520.0),
            ("/calculation/auxiliary/solarKwh", 3500.0),
            ("/calculation/auxiliary/generatorsKwh", 3090.8499999999995),
            ("/factors/primaryFactor", 1.11),
            ("/factors/co2KgPerKwh", 0.23922171411505377),
            ("/factors/renewableFactor", 0.07),
            ("/calculation/monthlyInputKwh/0", 364900.84867403394),
            ("/calculation/monthlyInputKwh/1", 313296.5744908498),
            ("/calculation/monthlyInputKwh/2", 290299.28882323706),
            ("/calculation/monthlyInputKwh/3", 212663.06510526748),
            ("/calculation/monthlyInputKwh/4", 90909.66522201627),
            ("/calculation/monthlyInputKwh/5", 58938.63874604965),
            ("/calculation/monthlyInputKwh/6", 16986.301369863013),
            ("/calculation/monthlyInputKwh/7", 16986.301369863013),
            ("/calculation/monthlyInputKwh/8", 70015.84005722858),
            ("/calculation/monthlyInputKwh/9", 188795.95435957704),
            ("/calculation/monthlyInputKwh/10", 242729.75437846742),
            ("/calculation/monthlyInputKwh/11", 333477.7674035468),
            ("/generators/0/energyFraction", 0.08477272727272728),
            ("/generators/0/primaryFactor", 0.0),
            ("/generators/0/co2KgPerKwh", 0.0),
            ("/generators/0/renewableFactor", 1.0),
            ("/generators/0/heatKwh", 186500.0),
            ("/generators/0/auxiliaryKwh", 0.0),
            ("/generators/1/energyFraction", 0.9152272727272728),
            ("/generators/1/primaryFactor", 1.075268817204301),
            ("/generators/1/co2KgPerKwh", 0.23440860215053763),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 2013500.0),
            ("/generators/1/efficiency", 0.93),
            ("/generators/1/auxiliaryKwh", 3090.8499999999995),
        ],
    );
}

/// Table P.5 heat pump (outdoor air at 55 °C: 2,80) preferred over an outdoor oil boiler (full load less 5 points,
/// P.6.5.4.2); no source-pump term with the forfait efficiency (P.6.8.4.3); pumps with 5 000 h.
#[test]
fn annex_p_table_p5_heat_pump_aux() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":2500000,"distribution":{"method":"flows","lossKwh":300000,"sourceReference":"x"},"generators":[{"id":"hp","nominalPowerKw":400,"kind":{"kind":"heat_pump","efficiency":{"method":"table_p5","source":"electric_outdoor_air","supplyTemperatureC":55},"drive":{"kind":"electricity"}}},{"id":"b","nominalPowerKw":1000,"kind":{"kind":"boiler","carrier":{"kind":"oil"},"efficiency":{"method":"full_load","value":0.93,"outdoorInstallation":true,"sourceReference":"x"}}}],"auxiliary":{"distribution":{"method":"pumps","pumpPowersW":[1500,800],"operatingHours":5000,"sourceReference":"x"}},"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["hp", "b"],
        &[
            ("/calculation/deliveredKwh", 2500000.0),
            ("/calculation/inputKwh", 2800000.0),
            ("/calculation/distributionLossKwh", 300000.0),
            ("/distributionEfficiency", 0.8928571428571429),
            ("/generationPrimaryFactor", 0.6910389610389611),
            ("/calculation/auxiliaryElectricityKwh", 14114.4),
            ("/calculation/auxiliary/distributionKwh", 11500.0),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 2614.4),
            ("/calculation/beta", 0.21428571428571427),
            ("/factors/primaryFactor", 0.79),
            ("/factors/co2KgPerKwh", 0.19487160913454551),
            ("/factors/renewableFactor", 0.39),
            ("/generators/0/energyFraction", 0.72),
            ("/generators/0/primaryFactor", 0.5178571428571429),
            ("/generators/0/co2KgPerKwh", 0.09571428571428572),
            ("/generators/0/renewableFactor", 0.6428571428571428),
            ("/generators/0/heatKwh", 2016000.0),
            ("/generators/0/efficiency", 2.8),
            ("/generators/0/auxiliaryKwh", 875.9999999999999),
            ("/generators/1/energyFraction", 0.28),
            ("/generators/1/primaryFactor", 1.1363636363636365),
            ("/generators/1/co2KgPerKwh", 0.3704545454545455),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 784000.0000000001),
            ("/generators/1/efficiency", 0.88),
            ("/generators/1/auxiliaryKwh", 1738.4),
        ],
    );
}

/// CHP preferred over a biogas boiler with P_HD;gen;ref from records (P.25 alternative); table P.11 secondary net.
#[test]
fn annex_p_secondary_forfait_reference_power() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":2500000,"distribution":{"method":"flows","lossKwh":300000,"sourceReference":"x"},"referencePowerKw":1500,"generators":[{"id":"chp","nominalPowerKw":300,"kind":{"kind":"chp_without_loss","carrier":{"kind":"natural_gas"},"tableP6":{"electricalPowerKw":200,"installedAfter2006":false}}},{"id":"b","nominalPowerKw":1000,"kind":{"kind":"combustion","carrier":{"kind":"biogas"},"efficiency":0.9,"efficiencyReference":"x"}}],"auxiliary":{"distribution":{"method":"forfait","network":"secondary"}},"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["chp", "b"],
        &[
            ("/calculation/deliveredKwh", 2500000.0),
            ("/calculation/inputKwh", 2800000.0),
            ("/calculation/distributionLossKwh", 300000.0),
            ("/distributionEfficiency", 0.8928571428571429),
            ("/generationPrimaryFactor", 0.7887962962962962),
            ("/calculation/auxiliaryElectricityKwh", 9456.49477773597),
            ("/calculation/auxiliary/distributionKwh", 5040.0),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 4416.49477773597),
            ("/calculation/beta", 0.2),
            ("/factors/primaryFactor", 0.89),
            ("/factors/co2KgPerKwh", 0.21246143994387695),
            ("/factors/renewableFactor", 0.27),
            ("/generators/0/energyFraction", 0.7),
            ("/generators/0/primaryFactor", 1.1268518518518518),
            ("/generators/0/co2KgPerKwh", 0.26970370370370367),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 1959999.9999999998),
            ("/generators/0/auxiliaryKwh", 2616.4947777359707),
            ("/generators/1/energyFraction", 0.30000000000000004),
            ("/generators/1/primaryFactor", 0.0),
            ("/generators/1/co2KgPerKwh", 0.0),
            ("/generators/1/renewableFactor", 1.0),
            ("/generators/1/heatKwh", 840000.0000000001),
            ("/generators/1/efficiency", 0.9),
            ("/generators/1/auxiliaryKwh", 1800.0),
        ],
    );
}

/// Hot water: P.40 gives F ≥ 1, so F = 1 (P.41) for the preferred heat pump with 20 % direct renewable electricity.
#[test]
fn annex_p_hot_water_preferred_full() {
    annex_p(
        r#"{"method":"calculated","function":"hot_water","deliveredKwh":400000,"hotWaterStorage":{"method":"forfait","insulation":"at_least20_mm"},"distribution":{"method":"flows","lossKwh":50000,"sourceReference":"x"},"generators":[{"id":"hp","nominalPowerKw":200,"kind":{"kind":"heat_pump","efficiency":{"method":"declared","value":3,"sourceReference":"x"},"drive":{"kind":"electricity","directRenewableShare":0.2}}},{"id":"b","nominalPowerKw":300,"kind":{"kind":"combustion","carrier":{"kind":"natural_gas"},"efficiency":0.9,"efficiencyReference":"x"}}],"auxiliary":{"distribution":{"method":"forfait"}},"sourceReference":"x"}"#,
        SystemFunction::HotWater,
        &["hp", "b"],
        &[
            ("/calculation/deliveredKwh", 400000.0),
            ("/calculation/inputKwh", 450000.0),
            ("/calculation/distributionLossKwh", 50000.0),
            ("/distributionEfficiency", 0.8888888888888888),
            ("/storageEfficiency", 0.9),
            ("/generationPrimaryFactor", 0.4296296296296296),
            ("/calculation/auxiliaryElectricityKwh", 7512.000000000002),
            ("/calculation/auxiliary/distributionKwh", 810.0),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 6702.000000000002),
            ("/factors/primaryFactor", 0.52),
            ("/factors/co2KgPerKwh", 0.09436637333333335),
            ("/factors/renewableFactor", 0.64),
            ("/generators/0/energyFraction", 1.0),
            ("/generators/0/primaryFactor", 0.38666666666666666),
            ("/generators/0/co2KgPerKwh", 0.07146666666666668),
            ("/generators/0/renewableFactor", 0.6666666666666667),
            ("/generators/0/heatKwh", 500000.0),
            ("/generators/0/efficiency", 3.0),
            ("/generators/0/auxiliaryKwh", 5826.000000000002),
            ("/generators/1/energyFraction", 0.0),
            ("/generators/1/primaryFactor", 1.1111111111111112),
            ("/generators/1/co2KgPerKwh", 0.24222222222222223),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 0.0),
            ("/generators/1/efficiency", 0.9),
            ("/generators/1/auxiliaryKwh", 875.9999999999999),
        ],
    );
}

/// Cold network at 7 °C all year (gains only counted where positive), chillers from table P.9 with HT emission
/// preferred, the two other chillers by power (P.51); heat rejection supplied and closed tower (33 W/kW); pumps 3 000 h (P.67).
#[test]
fn annex_p_cold_equal_efficiency_group() {
    annex_p(
        r#"{"method":"calculated","function":"cooling","deliveredKwh":600000,"distribution":{"method":"pipes","sourceReference":"x","supplyBelow10C":true,"otherLossKwh":0,"waterTemperature":{"method":"constant","temperatureC":7},"segments":[{"lengthM":500,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.1,"outerDiameterM":0.18}],"placement":{"kind":"buried","coverDepthM":0.9,"ambient":{"kind":"outdoor"}},"correction":"two_pipes_in_trench"}]},"generators":[{"id":"c1","nominalPowerKw":300,"auxiliary":{"heatRejectionWPerKw":28,"sourceReference":"x"},"kind":{"kind":"compression_chiller","variant":"unspecified","drive":{"kind":"electricity"}}},{"id":"c2","nominalPowerKw":200,"auxiliary":{"heatRejection":"closed_cooling_tower"},"kind":{"kind":"compression_chiller","variant":"high_temperature_emission","drive":{"kind":"electricity"}}},{"id":"c3","nominalPowerKw":250,"kind":{"kind":"compression_chiller","variant":"wet_cooling","drive":{"kind":"electricity"}}}],"auxiliary":{"distribution":{"method":"pumps","pumpPowersW":[3000],"operatingHours":3000,"sourceReference":"x"}},"sourceReference":"x"}"#,
        SystemFunction::Cooling,
        &["c1", "c2", "c3"],
        &[
            ("/calculation/deliveredKwh", 600000.0),
            ("/calculation/inputKwh", 605646.4811231308),
            ("/calculation/distributionLossKwh", 5646.481123130731),
            ("/distributionEfficiency", 0.9906769356396494),
            ("/generationPrimaryFactor", 0.38183333333333336),
            ("/calculation/auxiliaryElectricityKwh", 23501.952535525554),
            ("/calculation/auxiliary/distributionKwh", 9000.0),
            ("/calculation/auxiliary/solarKwh", 0.0),
            ("/calculation/auxiliary/generatorsKwh", 14501.952535525554),
            ("/calculation/beta", 0.6),
            ("/factors/primaryFactor", 0.45),
            ("/factors/co2KgPerKwh", 0.08173502378997323),
            ("/factors/renewableFactor", 0.0),
            ("/calculation/monthlyInputKwh/0", 51438.46825977275),
            ("/calculation/monthlyInputKwh/1", 46460.55197656894),
            ("/calculation/monthlyInputKwh/2", 51438.46825977275),
            ("/calculation/monthlyInputKwh/3", 49779.16283203814),
            ("/calculation/monthlyInputKwh/4", 51438.46825977275),
            ("/calculation/monthlyInputKwh/5", 49779.16283203814),
            ("/calculation/monthlyInputKwh/6", 51438.46825977275),
            ("/calculation/monthlyInputKwh/7", 51438.46825977275),
            ("/calculation/monthlyInputKwh/8", 49779.16283203814),
            ("/calculation/monthlyInputKwh/9", 51438.46825977275),
            ("/calculation/monthlyInputKwh/10", 49779.16283203814),
            ("/calculation/monthlyInputKwh/11", 51438.46825977275),
            ("/generators/0/energyFraction", 0.15999999999999992),
            ("/generators/0/primaryFactor", 0.48333333333333334),
            ("/generators/0/co2KgPerKwh", 0.08933333333333333),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 96903.43697970087),
            ("/generators/0/efficiency", 3.0),
            ("/generators/0/auxiliaryKwh", 4067.1011452997154),
            ("/generators/1/energyFraction", 0.3733333333333334),
            ("/generators/1/primaryFactor", 0.3625),
            ("/generators/1/co2KgPerKwh", 0.067),
            ("/generators/1/renewableFactor", 0.0),
            ("/generators/1/heatKwh", 226108.01961930218),
            ("/generators/1/efficiency", 4.0),
            ("/generators/1/auxiliaryKwh", 10347.251390225838),
            ("/generators/2/energyFraction", 0.46666666666666673),
            ("/generators/2/primaryFactor", 0.3625),
            ("/generators/2/co2KgPerKwh", 0.067),
            ("/generators/2/renewableFactor", 0.0),
            ("/generators/2/heatKwh", 282635.0245241277),
            ("/generators/2/efficiency", 4.0),
            ("/generators/2/auxiliaryKwh", 87.6),
        ],
    );
}

/// Cold network at 10 °C or higher: no distribution loss (P.6.2.2), also no other loss.
#[test]
fn annex_p_cold_pipes_not_below_10() {
    annex_p(
        r#"{"method":"calculated","function":"cooling","deliveredKwh":600000,"distribution":{"method":"pipes","sourceReference":"x","supplyBelow10C":false,"otherLossKwh":12000,"waterTemperature":{"method":"constant","temperatureC":12},"segments":[{"lengthM":500,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.1,"outerDiameterM":0.18}],"placement":{"kind":"buried","coverDepthM":0.9,"ambient":{"kind":"outdoor"}},"correction":"two_pipes_in_trench"}]},"generators":[{"id":"c","energyFraction":1,"nominalPowerKw":300,"kind":{"kind":"compression_chiller","variant":"unspecified","drive":{"kind":"electricity"}}}],"auxiliaryElectricityKwh":5000,"sourceReference":"x"}"#,
        SystemFunction::Cooling,
        &["c"],
        &[
            ("/calculation/deliveredKwh", 600000.0),
            ("/calculation/inputKwh", 600000.0),
            ("/calculation/distributionLossKwh", 0.0),
            ("/distributionEfficiency", 1.0),
            ("/generationPrimaryFactor", 0.48333333333333334),
            ("/calculation/auxiliaryElectricityKwh", 5000.0),
            ("/factors/primaryFactor", 0.5),
            ("/factors/co2KgPerKwh", 0.09156666666666667),
            ("/factors/renewableFactor", 0.0),
            ("/generators/0/energyFraction", 1.0),
            ("/generators/0/primaryFactor", 0.48333333333333334),
            ("/generators/0/co2KgPerKwh", 0.08933333333333333),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 600000.0),
            ("/generators/0/efficiency", 3.0),
        ],
    );
}

/// Hot water with a vessel with given surface and loss factor, table P.7 between rows (next larger diameter, next
/// thinner insulation), a supplied U, an exchanger with 0,6 W/kW, θ_WD;circ 75 °C and f_x 1,1; VR boiler HT (table P.3).
#[test]
fn annex_p_wd_storage_lookup_exchanger() {
    annex_p(
        r#"{"method":"calculated","function":"hot_water","deliveredKwh":250000,"hotWaterStorage":{"method":"calculated","sourceReference":"x","circulationTemperatureC":75,"correctionFactor":1.1,"vessels":[{"volumeL":800,"surfaceM2":4.2,"lossFactorWPerM2k":1.8,"waterTemperatureC":62}],"pipes":[{"lengthM":12,"outerDiameterMm":20,"insulationMm":12},{"lengthM":6,"outerDiameterMm":54,"insulationMm":0,"ambientTemperatureC":10}],"exchanger":{"nominalPowerKw":150,"specificLossWPerKw":0.6}},"distribution":{"method":"flows","lossKwh":40000,"sourceReference":"x"},"generators":[{"id":"b","energyFraction":1,"nominalPowerKw":150,"kind":{"kind":"boiler","carrier":{"kind":"natural_gas"},"efficiency":{"method":"table_p3","boiler":"vr"}}}],"auxiliaryElectricityKwh":1000,"sourceReference":"x"}"#,
        SystemFunction::HotWater,
        &["b"],
        &[
            ("/calculation/deliveredKwh", 250000.0),
            ("/calculation/inputKwh", 290000.0),
            ("/calculation/distributionLossKwh", 40000.0),
            ("/distributionEfficiency", 0.8620689655172413),
            ("/storageEfficiency", 0.9639547559147753),
            ("/generationPrimaryFactor", 1.3831907827126435),
            ("/calculation/auxiliaryElectricityKwh", 1000.0),
            ("/factors/primaryFactor", 1.62),
            ("/factors/co2KgPerKwh", 0.35085328513237335),
            ("/factors/renewableFactor", 0.0),
            ("/generators/0/energyFraction", 1.0),
            ("/generators/0/primaryFactor", 1.3333333333333333),
            ("/generators/0/co2KgPerKwh", 0.2906666666666667),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 300843.99523999996),
            ("/generators/0/efficiency", 0.75),
        ],
    );
}

/// Pipe losses per outdoor bin without switch-off, the curve held constant outside its points, with buffer and other
/// losses (P.15 with P.43/P.44).
#[test]
fn annex_p_heat_bins_full_year_buffers() {
    annex_p(
        r#"{"method":"calculated","function":"heating","deliveredKwh":5000000,"distribution":{"method":"pipes","sourceReference":"x","otherLossKwh":10000,"segments":[{"lengthM":3000,"layers":[{"conductivityWPerMk":0.027,"innerDiameterM":0.1143,"outerDiameterM":0.2},{"conductivityWPerMk":0.4,"innerDiameterM":0.2,"outerDiameterM":0.21}],"placement":{"kind":"buried","coverDepthM":0.8,"ambient":{"kind":"outdoor"}},"correction":"two_pipes_in_trench"},{"lengthM":400,"layers":[{"conductivityWPerMk":0.035,"innerDiameterM":0.06,"outerDiameterM":0.12}],"placement":{"kind":"buried","coverDepthM":0.5,"groundConductivity":1.2,"ambient":{"kind":"crawlspace"}},"correction":"single_pipe_in_trench"},{"lengthM":600,"layers":[{"conductivityWPerMk":0.04,"innerDiameterM":0.05,"outerDiameterM":0.11}],"placement":{"kind":"in_air","ambient":{"kind":"indoor","temperatureC":18}}},{"lengthM":50,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"outdoor"}},"correction":"old_sliding_system"},{"lengthM":120,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"in_air","ambient":{"kind":"crawlspace"},"surfaceCoefficient":12},"correctionFactor":0.95},{"lengthM":200,"layers":[{"conductivityWPerMk":0.03,"innerDiameterM":0.08,"outerDiameterM":0.16}],"placement":{"kind":"buried","coverDepthM":1,"ambient":{"kind":"outdoor"}},"resistanceKmPerW":2.5}],"waterTemperature":{"method":"outdoor_bins","curve":[{"outdoorC":20,"waterC":70},{"outdoorC":-10,"waterC":85}]},"buffers":[{"volumeL":20000,"diameterAtLeast50Cm":true,"insulation":"at_least20_mm","waterTemperatureC":80,"ambientTemperatureC":12}]},"generators":[{"id":"b","energyFraction":1,"kind":{"kind":"combustion","carrier":{"kind":"natural_gas"},"efficiency":0.9,"efficiencyReference":"x"}}],"auxiliaryElectricityKwh":30000,"sourceReference":"x"}"#,
        SystemFunction::Heating,
        &["b"],
        &[
            ("/calculation/deliveredKwh", 5000000.0),
            ("/calculation/inputKwh", 5867407.702806479),
            ("/calculation/distributionLossKwh", 867407.7028064792),
            ("/distributionEfficiency", 0.852165087762423),
            ("/generationPrimaryFactor", 1.1111111111111112),
            ("/calculation/auxiliaryElectricityKwh", 30000.0),
            ("/factors/primaryFactor", 1.32),
            ("/factors/co2KgPerKwh", 0.2858513064915139),
            ("/factors/renewableFactor", 0.0),
            ("/generators/0/energyFraction", 1.0),
            ("/generators/0/primaryFactor", 1.1111111111111112),
            ("/generators/0/co2KgPerKwh", 0.24222222222222223),
            ("/generators/0/renewableFactor", 0.0),
            ("/generators/0/heatKwh", 5867407.702806479),
            ("/generators/0/efficiency", 0.9),
        ],
    );
}
