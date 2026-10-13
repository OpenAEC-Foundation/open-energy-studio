//! The switch points between editions, keyed by the [`NormProfile`] fields
//! that carry them.
//!
//! Every field two profiles can differ in belongs to exactly one entry here
//! (a test walks the serialized profiles). An entry carries the numbers of
//! docs/nta8800-normversies.md ("Verschillen … die de kern omschakelt") and
//! the input members the switch acts on, so a refusal or requirement the
//! kernel reports after a change of edition can be traced to its switch
//! point. Entries without input members change values only.

use super::{NormProfile, NormVersion};
use serde::Serialize;
use serde_json::Value;

/// One switch point: the profile fields it consists of and the input
/// members it accepts, refuses or requires per edition.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPoint {
    /// Row numbers in docs/nta8800-normversies.md.
    pub points: &'static [&'static str],
    /// Short subject, as in the documentation table.
    pub subject: &'static str,
    /// [`NormProfile`] fields of the switch point.
    pub fields: &'static [&'static str],
    /// Input members the switch acts on, as the trailing members of a gap
    /// path without indices (`area.method`, `wallHeightAboveGroundM`).
    pub inputs: &'static [&'static str],
    /// Codes the switch point gives on an enclosing path (a requirement
    /// reported on the floor rather than on its member).
    pub codes: &'static [&'static str],
    /// The input keeps its place but means something else per edition
    /// (`biomassAbove500Kw`: above the edition's own threshold).
    pub reinterpreted: bool,
    /// Whether an edition has the route the inputs belong to. An input the
    /// target edition has no route for and does not refuse is ignored there.
    #[serde(skip)]
    pub has_route: Option<fn(&NormProfile) -> bool>,
}

const fn values(
    points: &'static [&'static str],
    subject: &'static str,
    fields: &'static [&'static str],
) -> SwitchPoint {
    SwitchPoint {
        points,
        subject,
        fields,
        inputs: &[],
        codes: &[],
        reinterpreted: false,
        has_route: None,
    }
}

const fn inputs(
    points: &'static [&'static str],
    subject: &'static str,
    fields: &'static [&'static str],
    inputs: &'static [&'static str],
) -> SwitchPoint {
    SwitchPoint {
        points,
        subject,
        fields,
        inputs,
        codes: &[],
        reinterpreted: false,
        has_route: None,
    }
}

const fn routed(
    points: &'static [&'static str],
    subject: &'static str,
    fields: &'static [&'static str],
    inputs: &'static [&'static str],
    has_route: fn(&NormProfile) -> bool,
) -> SwitchPoint {
    SwitchPoint {
        points,
        subject,
        fields,
        inputs,
        codes: &[],
        reinterpreted: false,
        has_route: Some(has_route),
    }
}

/// Every switch point, in the order of the documentation.
pub const SWITCH_POINTS: &[SwitchPoint] = &[
    routed(
        &["1"],
        "f_BACS 1,05 (5.5.8/15.3)",
        &["bacs_factor_route"],
        &["bacsFactor"],
        |p| p.bacs_factor_route,
    ),
    values(&["2"], "opslagcorrectie (5.14a/b)", &["storage_credit"]),
    values(
        &["3"],
        "K_CO2 tabel 5.3",
        &[
            "k_co2_electricity",
            "k_co2_gas",
            "k_co2_oil",
            "k_co2_biomass",
            "k_co2_district_heat_forfait",
        ],
    ),
    values(&["4"], "K_CO2 AVI tabel 5.6", &["k_co2_waste_incineration"]),
    values(&["5"], "nieuwe indicatoren", &["indicators_2025"]),
    values(
        &["6"],
        "Ψ tabel 9.16 collectief ≤ 500 m²",
        &["psi_collective_combined_small"],
    ),
    values(
        &["7"],
        "f_cor.bron.col (9.62)",
        &["collective_source_correction"],
    ),
    inputs(
        &["8", "46", "50"],
        "bronwarmte warmtepomp",
        &["heat_pump_source_route"],
        &["realisedFrom2013"],
    ),
    routed(
        &["9"],
        "dakrand als belemmering (17.3.2 f)",
        &["roof_edge_obstruction"],
        &["obstruction"],
        |p| p.roof_edge_obstruction,
    ),
    routed(
        &["10"],
        "zonwerend glas, vaste en draaibare lamellen",
        &["permanent_shading_routes"],
        &["glazingType", "fixedLouvres", "fixedLouvres.control"],
        |p| p.permanent_shading_routes,
    ),
    values(
        &["10a"],
        "TOjuli-uitzondering 5.7.1 methode 3: zonwering tabel 7.5/7.6",
        &["solar_limitation_movable_shading"],
    ),
    values(
        &["11"],
        "f_gebouw;si;W (13.157)/(13.160)",
        &["hot_water_building_share"],
    ),
    values(&["12"], "tabel 13.18 c_W,EU;gen", &["table_13_18"]),
    values(
        &["13"],
        "gemeten motorrendement (11.136)",
        &["motor_efficiency_free_choice"],
    ),
    inputs(
        &["14"],
        "bijlage AA",
        &["annex_aa", "annex_aa_requirement_floor"],
        &["effectiveMassKgPerM2", "roofAreaM2"],
    ),
    values(
        &["15"],
        "F_A;iso biobased (tabel E.5)",
        &["bio_based_ageing_one"],
    ),
    values(
        &["16"],
        "biobased λ, riet (tabel E.11/E.12)",
        &["bio_based_lambda_2025"],
    ),
    values(&["17"], "rieten dak (I.3)", &["reed_thatch_divisor"]),
    values(&["18"], "boosterwarmtepomp (W.9)", &["booster_bounds"]),
    values(
        &["18a"],
        "LT-bronnen als collectieve bron (P.3 item 6)",
        &["wide_collective_low_temperature_sources"],
    ),
    routed(
        &["19"],
        "tabel 13.2 keukenrijen",
        &["kitchen_diameter_rows"],
        &["kitchenPipeDiameter"],
        |p| p.kitchen_diameter_rows,
    ),
    values(&["20"], "τ_argII tabel 11.7", &["tau_ventilative_cooling"]),
    values(&["21"], "f_argII", &["ventilative_cooling_operation"]),
    inputs(
        &["22"],
        "spuiopening uit C_d en C_e (11.71a)",
        &["discharge_opening_route"],
        &["area.method"],
    ),
    values(
        &["23"],
        "f_τ woonfunctie tabel 11.8",
        &["dwelling_occupancy_factor"],
    ),
    values(&["24"], "θ_e;argII tabel 17.1", &["argii_temperature_c"]),
    values(&["25"], "ΔT_fan", &["fan_temperature_rise_k"]),
    values(&["26"], "Ψ detail 17 tabel I.1", &["psi_detail_17"]),
    values(
        &["27"],
        "λ houtvezel en cellulose tabel E.10",
        &["wood_fibre_cellulose_lambda"],
    ),
    values(
        &["28", "70"],
        "referentievermogen (P.25)",
        &["reference_power_divisor"],
    ),
    values(
        &["29"],
        "rendement geothermie",
        &["geothermal_efficiency_fixed"],
    ),
    SwitchPoint {
        reinterpreted: true,
        ..inputs(
            &["30", "53"],
            "biomassagrens bmA",
            &["biomass_threshold_kw"],
            &["biomassAbove500Kw"],
        )
    },
    inputs(
        &["31"],
        "bijlage AA en zoninstralingstoets als bewijs koelcapaciteit",
        &["cooling_capacity_methods_route"],
        &["capacity.method"],
    ),
    values(
        &["32"],
        "renovatiestandaard tabel 5.7",
        &["renovation_standard"],
    ),
    routed(
        &["33"],
        "tabel 14.3 led vanaf 2017",
        &["led_2017_column"],
        &["ledFrom2017"],
        |p| p.led_2017_column,
    ),
    routed(
        &["34"],
        "roeden in U_w (8.17)/(8.18)",
        &["glazing_bar_term"],
        &["glazingBars"],
        |p| p.glazing_bar_term,
    ),
    inputs(
        &["35"],
        "f_sh;with woningen (7.6.6.1.4)",
        &["dwelling_shading_heating_off"],
        &["movableShading.control"],
    ),
    values(
        &["36"],
        "restwarmte f_Pren (5.47)/(5.55)",
        &["residual_heat_pren_primary"],
    ),
    inputs(
        &["37"],
        "lichtkoepels U_rc → U_C",
        &["rooflight_route"],
        &["elements"],
    ),
    SwitchPoint {
        codes: &["panel_build_year_required"],
        ..routed(
            &["38"],
            "paneel-U naar bouwjaar (I.13/I.14)",
            &["panel_build_year_tables"],
            &["buildYear"],
            |p| p.panel_build_year_tables,
        )
    },
    values(
        &["39"],
        "NEN 1087-factor zonder roosterspecificatie",
        &["unspecified_screen_factor"],
    ),
    values(
        &["40"],
        "A_w;cros (11.77)",
        &["cross_area_roof_all_sectors"],
    ),
    values(
        &["41"],
        "distributie verwarming 9.4",
        &["distribution_2024_rules"],
    ),
    values(
        &["42"],
        "tabel 9.16 gecombineerde leidingen",
        &["table_9_16_combined_rows"],
    ),
    routed(
        &["43"],
        "f_H;dis;rbl 0,5 in ongeïsoleerde schil",
        &["distribution_half_recoverable_route"],
        &["uninsulatedPipesInUninsulatedShell"],
        |p| p.distribution_half_recoverable_route,
    ),
    values(
        &["44"],
        "tabel 13.4 onbekende diameter",
        &["table_13_4_system_rows"],
    ),
    inputs(
        &["45"],
        "serieschakeling tapwater",
        &["hot_water_series_routes"],
        &["series", "kind"],
    ),
    routed(
        &["47"],
        "afgifte verwarming 9.3 (2023)",
        &["emission_tables_2023"],
        &["emission.edition2023"],
        |p| p.emission_tables_2023,
    ),
    routed(
        &["48"],
        "afgifte koeling 10.3.3 (2023)",
        &["cooling_emission_tables_2023"],
        &["cooling.emission.edition2023"],
        |p| p.cooling_emission_tables_2023,
    ),
    SwitchPoint {
        // From 2023 a (ground)water source needs its temperature class.
        codes: &["source_temperature_class_mismatch"],
        ..inputs(
            &["49", "51"],
            "tabellen 9.27/9.29 en P.5",
            &["heat_pump_tables_2022"],
            &[
                "designSupplyTemperatureC",
                "source",
                "sourceTemperatureC",
                "sourceTemperatureEvidenceReference",
                "efficiency.source",
                "efficiency.supplyTemperatureC",
            ],
        )
    },
    inputs(
        &["52"],
        "flexmodus",
        &["flex_mode_route"],
        &["kind", "generators"],
    ),
    values(
        &["54"],
        "bijgeplaatste preferente opwekker (9.58)",
        &["preference_beta_building_share"],
    ),
    inputs(
        &["55"],
        "tabel 9.14 ontwerptemperatuurklassen",
        &["design_classes_60_and_70"],
        &["designTemperatureClass"],
    ),
    inputs(
        &["56"],
        "ventilatorvermogen NEN-EN 16430",
        &["tested_emission_fan_power"],
        &["testedPowerW"],
    ),
    inputs(
        &["57"],
        "tabel 7.5 onbekende kleur",
        &["unknown_shade_colour_rows"],
        &["colour"],
    ),
    routed(
        &["58"],
        "tabel 7.10 naar massa per m²",
        &["thermal_mass_by_kg_per_m2"],
        &["massKgPerM2"],
        |p| p.thermal_mass_by_kg_per_m2,
    ),
    SwitchPoint {
        codes: &["ground_floor_wall_height_required"],
        ..inputs(
            &["59"],
            "hoogte h boven maaiveld (8.47)",
            &["crawl_wall_height_fixed"],
            &["wallHeightAboveGroundM"],
        )
    },
    values(
        &["60"],
        "F_A;iso minerale-wolvlokken",
        &["mineral_wool_flakes_ageing"],
    ),
    inputs(
        &["61"],
        "bekende hogere λ_equi (I.2)",
        &["lambda_equi_known_route"],
        &["knownLambdaEquivalent"],
    ),
    inputs(
        &["62"],
        "zwembad (11.57)",
        &["swimming_pool_route"],
        &["swimmingPool"],
    ),
    inputs(
        &["63"],
        "η_hr volgens NEN-EN 13053",
        &["en_13053_route"],
        &["efficiency.standard"],
    ),
    routed(
        &["64"],
        "f_sto;dis;ls elektroboiler",
        &["electric_boiler_insulated_pipe_factor"],
        &["electricBoilerInsulatedPipe"],
        |p| p.electric_boiler_insulated_pipe_factor,
    ),
    values(
        &["65"],
        "ϑ_sto;amb ventilatieretourluchtwarmtepomp",
        &["storage_ambient_exhaust_air"],
    ),
    values(
        &["80"],
        "c_W;gen forfaitaire tapwaterwarmtepomp",
        &["forfait_heat_pump_class_correction"],
    ),
    inputs(
        &["66"],
        "C_W;mixed air (13.153b)",
        &["mixed_air_route"],
        &["mixedAir"],
    ),
    routed(
        &["67"],
        "onderhoudsfactor MF tabel 14.4",
        &["lighting_maintenance_factor"],
        &["constantIlluminance"],
        |p| p.lighting_maintenance_factor,
    ),
    SwitchPoint {
        codes: &["high_test_standard_invalid"],
        ..inputs(
            &["67a"],
            "beproeving hoge-COP-rij tabel 9.28",
            &["heat_pump_high_test_standard"],
            &["testStandardEdition"],
        )
    },
    inputs(
        &["68"],
        "piekvermogen PV (16.4)",
        &["pv_kpk_per_m2_floor"],
        &["panelPeakPowerW", "peakPower.method"],
    ),
    values(&["69"], "λ_equi;ntr (I.2)", &["lambda_equi_ntr"]),
    inputs(
        &["71"],
        "restwarmte bijlage P",
        &["residual_heat_fixed_factors"],
        &["auxiliarySpecific"],
    ),
    inputs(
        &["72"],
        "kleine systemen bijlage P",
        &["small_system_forfait_route"],
        &["distribution", "auxiliary", "network"],
    ),
    SwitchPoint {
        codes: &["generator_power_required"],
        ..values(
            &["73"],
            "β preferente opwekker onbekend",
            &["unknown_beta_route"],
        )
    },
    values(
        &["74"],
        "forfaitaire hulpenergie (9.85)",
        &["device_aux_a_from_2015_kwh", "heat_pump_aux_constants"],
    ),
    values(&["75"], "f_systype E1 (11.142)", &["fan_systype_combined"]),
    inputs(
        &["76"],
        "tabellen I.1/I.2",
        &["psi_columns_and_default", "psi_detail_14"],
        &["column", "position"],
    ),
    inputs(
        &["77"],
        "werkelijke leidinglengte woningen",
        &["residential_actual_pipe_length"],
        &["actualPipeLengthM"],
    ),
    inputs(
        &["78"],
        "debiet uit kwaliteitsverklaring (13.148a)",
        &["declared_exhaust_air_flow_route"],
        &["declaredFlowM3PerH"],
    ),
    inputs(
        &["79"],
        "koudeterugwinning met 100 % bypass",
        &["cold_recovery_route"],
        &["coldRecoveryEvidence"],
    ),
];

/// The profile as a field map; `version` is left out.
fn fields_of(version: NormVersion) -> serde_json::Map<String, Value> {
    let mut map = match serde_json::to_value(version.profile()) {
        Ok(Value::Object(map)) => map,
        _ => serde_json::Map::new(),
    };
    map.remove("version");
    map
}

/// The profile fields in which two editions differ, in field order.
pub fn differing_fields(a: NormVersion, b: NormVersion) -> Vec<String> {
    let left = fields_of(a);
    let right = fields_of(b);
    left.iter()
        .filter(|(key, value)| right.get(*key) != Some(*value))
        .map(|(key, _)| key.clone())
        .collect()
}

/// Every field name of [`NormProfile`] except `version`.
pub fn profile_field_names() -> Vec<String> {
    fields_of(NormVersion::V2025C1).keys().cloned().collect()
}

/// The switch points whose fields differ between two editions.
pub fn switch_points_between(a: NormVersion, b: NormVersion) -> Vec<&'static SwitchPoint> {
    let differing = differing_fields(a, b);
    SWITCH_POINTS
        .iter()
        .filter(|point| {
            point
                .fields
                .iter()
                .any(|field| differing.iter().any(|d| d == field))
        })
        .collect()
}

/// The members of a gap path without indices: `a.b[2].c` → `["a", "b", "c"]`.
pub fn path_members(path: &str) -> Vec<&str> {
    path.split('.')
        .map(|part| part.split('[').next().unwrap_or(part))
        .filter(|part| !part.is_empty())
        .collect()
}

/// Whether `suffix` (`area.method`) ends the member list of `path`.
pub fn path_ends_with(path: &str, suffix: &str) -> bool {
    let members = path_members(path);
    let wanted: Vec<&str> = suffix.split('.').collect();
    members.len() >= wanted.len() && members[members.len() - wanted.len()..] == wanted[..]
}

/// The switch point between two editions a finding belongs to: of the
/// switch points whose fields differ, the one that names the code, else the
/// one with the longest matching input suffix.
pub fn attribute(
    code: &str,
    path: &str,
    a: NormVersion,
    b: NormVersion,
) -> Option<&'static SwitchPoint> {
    let between = switch_points_between(a, b);
    if let Some(point) = between.iter().find(|point| point.codes.contains(&code)) {
        return Some(point);
    }
    between
        .into_iter()
        .filter_map(|point| {
            point
                .inputs
                .iter()
                .filter(|input| path_ends_with(path, input))
                .map(|input| input.split('.').count())
                .max()
                .map(|length| (length, point))
        })
        .max_by_key(|(length, _)| *length)
        .map(|(_, point)| point)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_profile_field_belongs_to_exactly_one_switch_point() {
        for field in profile_field_names() {
            let owners: Vec<_> = SWITCH_POINTS
                .iter()
                .filter(|point| point.fields.contains(&field.as_str()))
                .collect();
            assert_eq!(
                owners.len(),
                1,
                "{field} belongs to {} switch points",
                owners.len()
            );
        }
        let names = profile_field_names();
        for point in SWITCH_POINTS {
            for field in point.fields {
                assert!(
                    names.iter().any(|name| name == field),
                    "{field} is not a profile field"
                );
            }
        }
    }

    #[test]
    fn every_field_that_differs_between_editions_has_a_switch_point() {
        for (i, a) in NormVersion::ALL.iter().enumerate() {
            for b in &NormVersion::ALL[i + 1..] {
                for field in differing_fields(*a, *b) {
                    assert!(
                        SWITCH_POINTS
                            .iter()
                            .any(|point| point.fields.contains(&field.as_str())),
                        "{field} differs between {} and {} without a switch point",
                        a.id(),
                        b.id()
                    );
                }
            }
        }
    }

    #[test]
    fn every_switch_point_is_a_row_of_the_documentation_and_back() {
        let doc = include_str!("../../../../docs/nta8800-normversies.md");
        let mut rows = Vec::new();
        for line in doc.lines() {
            let Some(rest) = line.strip_prefix("| ") else {
                continue;
            };
            let Some((first, _)) = rest.split_once(" |") else {
                continue;
            };
            let numbered = first.chars().next().is_some_and(|c| c.is_ascii_digit())
                && first.chars().all(|c| c.is_ascii_alphanumeric());
            if numbered {
                rows.push(first.to_string());
            }
        }
        for point in SWITCH_POINTS {
            for id in point.points {
                assert!(
                    rows.iter().any(|row| row == id),
                    "point {id} has no row in docs/nta8800-normversies.md"
                );
            }
        }
        for row in &rows {
            assert!(
                SWITCH_POINTS
                    .iter()
                    .any(|point| point.points.contains(&row.as_str())),
                "row {row} of docs/nta8800-normversies.md has no switch point"
            );
        }
    }

    #[test]
    fn a_switch_point_with_a_route_differs_in_that_route_somewhere() {
        for point in SWITCH_POINTS
            .iter()
            .filter(|point| point.has_route.is_some())
        {
            let route = point.has_route.unwrap();
            let values: Vec<bool> = NormVersion::ALL
                .iter()
                .map(|v| route(v.profile()))
                .collect();
            assert!(
                values.contains(&true) && values.contains(&false),
                "{:?}",
                point.points
            );
        }
    }

    #[test]
    fn attribution_takes_the_longest_input_suffix() {
        let point = attribute(
            "route_not_in_edition",
            "ntaCalculation.cooling.emission.edition2023",
            NormVersion::V2023,
            NormVersion::V2025C1,
        )
        .expect("attributed");
        assert_eq!(point.points, &["48"]);
        let point = attribute(
            "route_not_in_edition",
            "ntaCalculation.groundFloors[0].below.wallHeightAboveGroundM",
            NormVersion::V2022,
            NormVersion::V2025C1,
        )
        .expect("attributed");
        assert_eq!(point.points, &["59"]);
        assert!(attribute(
            "route_not_in_edition",
            "ntaCalculation.groundFloors[0].below.wallHeightAboveGroundM",
            NormVersion::V2024,
            NormVersion::V2025C1,
        )
        .is_none());
        let point = attribute(
            "ground_floor_wall_height_required",
            "ntaCalculation.groundFloors[0]",
            NormVersion::V2025C1,
            NormVersion::V2022,
        )
        .expect("attributed by code");
        assert_eq!(point.points, &["59"]);
    }
}
