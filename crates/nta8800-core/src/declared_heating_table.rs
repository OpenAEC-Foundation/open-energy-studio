//! Linear interpolation of user-supplied product declaration table values.
//! The caller selects the applicable table and supplies already determined
//! annual gross heat demand and design supply temperature. This does not
//! establish product applicability or calculate an NTA building result.

use crate::{KERNEL_VERSION, TARGET_NORM_VERSION};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredHeatingTableInput {
    pub declaration_id: String,
    pub declaration_norm_version: String,
    pub source_reference: String,
    pub table_scope: String,
    pub gross_heat_demand_kwh_per_year: f64,
    pub design_supply_temperature_c: f64,
    /// The first row explicitly represents all temperatures up to its listed upper limit.
    #[serde(default)]
    pub first_row_covers_lower_temperatures: bool,
    pub rows: Vec<TemperatureRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemperatureRow {
    pub supply_temperature_c: f64,
    pub points: Vec<DeclaredHeatingPoint>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredHeatingPoint {
    pub gross_heat_demand_kwh_per_year: f64,
    pub generation_efficiency: f64,
    pub preferred_energy_fraction: f64,
    pub auxiliary_electricity_kwh_per_year: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeclaredHeatingTableAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub target_norm_version: &'static str,
    pub kernel_version: &'static str,
    pub input_fingerprint: String,
    pub declaration_id: String,
    pub declaration_norm_version: String,
    pub source_reference: String,
    pub table_scope: String,
    pub declaration_edition_matches_target: bool,
    pub reference_verified: bool,
    pub annual_performance_available: bool,
    pub beng_calculation_available: bool,
    pub interpolation: Option<InterpolationBracket>,
    pub generation_efficiency: Option<f64>,
    pub preferred_energy_fraction: Option<f64>,
    pub auxiliary_electricity_kwh_per_year: Option<f64>,
    pub issues: Vec<TableIssue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterpolationBracket {
    pub demand_lower_kwh_per_year: f64,
    pub demand_upper_kwh_per_year: f64,
    pub demand_weight: f64,
    pub temperature_lower_c: f64,
    pub temperature_upper_c: f64,
    pub temperature_weight: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> TableIssue {
    TableIssue {
        code,
        path: path.into(),
    }
}

fn interpolate(a: f64, b: f64, weight: f64) -> f64 {
    a * (1.0 - weight) + b * weight
}

fn bracket(nodes: &[f64], query: f64) -> Option<(usize, usize, f64)> {
    if query < *nodes.first()? || query > *nodes.last()? {
        return None;
    }
    if query == nodes[0] {
        return Some((0, 0, 0.0));
    }
    for high in 1..nodes.len() {
        if query <= nodes[high] {
            let low = high - 1;
            return Some((low, high, (query - nodes[low]) / (nodes[high] - nodes[low])));
        }
    }
    None
}

pub fn assess_declared_heating_table(
    input: &DeclaredHeatingTableInput,
) -> DeclaredHeatingTableAssessment {
    let fingerprint = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(input).expect("typed input serializes"))
    );
    let mut issues = Vec::new();
    for (field, value) in [
        ("declarationId", &input.declaration_id),
        ("declarationNormVersion", &input.declaration_norm_version),
        ("sourceReference", &input.source_reference),
        ("tableScope", &input.table_scope),
    ] {
        if value.trim().is_empty() {
            issues.push(issue("declaration_provenance_required", field));
        }
    }
    if input.rows.len() < 2 {
        issues.push(issue("declaration_table_two_rows_required", "rows"));
    }
    let mut temperature_nodes = Vec::new();
    let mut demand_nodes = Vec::new();
    for (row_index, row) in input.rows.iter().enumerate() {
        let row_path = format!("rows[{row_index}]");
        if !row.supply_temperature_c.is_finite()
            || temperature_nodes
                .last()
                .is_some_and(|last| row.supply_temperature_c <= *last)
        {
            issues.push(issue(
                "declaration_temperature_axis_invalid",
                format!("{row_path}.supplyTemperatureC"),
            ));
        }
        temperature_nodes.push(row.supply_temperature_c);
        if row.points.len() < 2 {
            issues.push(issue(
                "declaration_table_two_points_required",
                format!("{row_path}.points"),
            ));
        }
        if row_index > 0 && row.points.len() != demand_nodes.len() {
            issues.push(issue(
                "declaration_table_shape_mismatch",
                format!("{row_path}.points"),
            ));
        }
        let mut previous = None;
        for (point_index, point) in row.points.iter().enumerate() {
            let path = format!("{row_path}.points[{point_index}]");
            let demand = point.gross_heat_demand_kwh_per_year;
            if !demand.is_finite() || demand <= 0.0 || previous.is_some_and(|last| demand <= last) {
                issues.push(issue(
                    "declaration_demand_axis_invalid",
                    format!("{path}.grossHeatDemandKwhPerYear"),
                ));
            }
            previous = Some(demand);
            if row_index == 0 {
                demand_nodes.push(demand);
            } else if demand_nodes.get(point_index) != Some(&demand) {
                issues.push(issue(
                    "declaration_demand_axis_mismatch",
                    format!("{path}.grossHeatDemandKwhPerYear"),
                ));
            }
            if !point.generation_efficiency.is_finite() || point.generation_efficiency <= 0.0 {
                issues.push(issue(
                    "declaration_efficiency_invalid",
                    format!("{path}.generationEfficiency"),
                ));
            }
            if !point.preferred_energy_fraction.is_finite()
                || !(0.0..=1.0).contains(&point.preferred_energy_fraction)
            {
                issues.push(issue(
                    "declaration_fraction_invalid",
                    format!("{path}.preferredEnergyFraction"),
                ));
            }
            if !point.auxiliary_electricity_kwh_per_year.is_finite()
                || point.auxiliary_electricity_kwh_per_year < 0.0
            {
                issues.push(issue(
                    "declaration_auxiliary_invalid",
                    format!("{path}.auxiliaryElectricityKwhPerYear"),
                ));
            }
        }
    }
    let demand_bracket = if input.gross_heat_demand_kwh_per_year.is_finite() {
        bracket(&demand_nodes, input.gross_heat_demand_kwh_per_year)
    } else {
        None
    };
    if demand_bracket.is_none() {
        issues.push(issue(
            "declaration_demand_out_of_range",
            "grossHeatDemandKwhPerYear",
        ));
    }
    let temperature_bracket = if input.design_supply_temperature_c.is_finite() {
        if input.first_row_covers_lower_temperatures
            && temperature_nodes
                .first()
                .is_some_and(|first| input.design_supply_temperature_c <= *first)
        {
            Some((0, 0, 0.0))
        } else {
            bracket(&temperature_nodes, input.design_supply_temperature_c)
        }
    } else {
        None
    };
    if temperature_bracket.is_none() {
        issues.push(issue(
            "declaration_temperature_out_of_range",
            "designSupplyTemperatureC",
        ));
    }
    let mut values = None;
    let mut interpolation = None;
    if issues.is_empty() {
        let (d0, d1, dw) = demand_bracket.expect("validated demand bracket");
        let (t0, t1, tw) = temperature_bracket.expect("validated temperature bracket");
        let per_row = |index: usize| {
            let a = &input.rows[index].points[d0];
            let b = &input.rows[index].points[d1];
            (
                interpolate(a.generation_efficiency, b.generation_efficiency, dw),
                interpolate(a.preferred_energy_fraction, b.preferred_energy_fraction, dw),
                interpolate(
                    a.auxiliary_electricity_kwh_per_year,
                    b.auxiliary_electricity_kwh_per_year,
                    dw,
                ),
            )
        };
        let a = per_row(t0);
        let b = per_row(t1);
        let result = (
            interpolate(a.0, b.0, tw),
            interpolate(a.1, b.1, tw),
            interpolate(a.2, b.2, tw),
        );
        if [result.0, result.1, result.2]
            .iter()
            .all(|value| value.is_finite())
        {
            values = Some(result);
            interpolation = Some(InterpolationBracket {
                demand_lower_kwh_per_year: demand_nodes[d0],
                demand_upper_kwh_per_year: demand_nodes[d1],
                demand_weight: dw,
                temperature_lower_c: temperature_nodes[t0],
                temperature_upper_c: temperature_nodes[t1],
                temperature_weight: tw,
            });
        } else {
            issues.push(issue("declaration_interpolation_overflow", "rows"));
        }
    }
    DeclaredHeatingTableAssessment {
        status: if issues.is_empty() {
            "input_valid"
        } else {
            "invalid"
        },
        scope: "declared_space_heating_table_interpolation_only",
        target_norm_version: crate::norm_versions::current_label(),
        kernel_version: KERNEL_VERSION,
        input_fingerprint: fingerprint,
        declaration_id: input.declaration_id.clone(),
        declaration_norm_version: input.declaration_norm_version.clone(),
        source_reference: input.source_reference.clone(),
        table_scope: input.table_scope.clone(),
        declaration_edition_matches_target: input.declaration_norm_version == TARGET_NORM_VERSION,
        reference_verified: false,
        annual_performance_available: false,
        beng_calculation_available: false,
        interpolation,
        generation_efficiency: values.map(|value| value.0),
        preferred_energy_fraction: values.map(|value| value.1),
        auxiliary_electricity_kwh_per_year: values.map(|value| value.2),
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> DeclaredHeatingTableInput {
        serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20250005gk-heating-grid-sample.json"
        ))
        .unwrap()
    }

    #[test]
    fn published_table_corners_and_midpoint_interpolate_without_annual_claim() {
        let mut input = fixture();
        let at_corner = assess_declared_heating_table(&input);
        assert_eq!(at_corner.status, "input_valid");
        assert_eq!(at_corner.generation_efficiency, Some(6.155));
        assert_eq!(at_corner.preferred_energy_fraction, Some(1.0));
        assert_eq!(at_corner.auxiliary_electricity_kwh_per_year, Some(93.0));
        input.gross_heat_demand_kwh_per_year = 8333.5;
        input.design_supply_temperature_c = 32.5;
        let midpoint = assess_declared_heating_table(&input);
        assert!((midpoint.generation_efficiency.unwrap() - 5.68575).abs() < 1e-12);
        assert!((midpoint.preferred_energy_fraction.unwrap() - 0.996).abs() < 1e-12);
        assert_eq!(midpoint.auxiliary_electricity_kwh_per_year, Some(108.5));
        let bracket = midpoint.interpolation.unwrap();
        assert_eq!(bracket.demand_lower_kwh_per_year, 5556.0);
        assert_eq!(bracket.demand_upper_kwh_per_year, 11111.0);
        assert_eq!(bracket.demand_weight, 0.5);
        assert_eq!(bracket.temperature_weight, 0.5);
        assert!(!midpoint.declaration_edition_matches_target);
        assert!(
            !midpoint.reference_verified
                && !midpoint.annual_performance_available
                && !midpoint.beng_calculation_available
        );
    }

    #[test]
    fn invalid_grid_or_extrapolation_produces_no_values() {
        let mut input = fixture();
        input.gross_heat_demand_kwh_per_year = 100.0;
        assert!(assess_declared_heating_table(&input)
            .generation_efficiency
            .is_none());
        input.gross_heat_demand_kwh_per_year = 5556.0;
        input.rows[1].points[1].preferred_energy_fraction = 1.01;
        let result = assess_declared_heating_table(&input);
        assert_eq!(result.status, "invalid");
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.code == "declaration_fraction_invalid"));
        assert!(result.generation_efficiency.is_none());
        assert!(result.interpolation.is_none());
        input.rows[1].points[1].preferred_energy_fraction = 0.992;
        input.rows[1].points[1].gross_heat_demand_kwh_per_year = 10000.0;
        assert!(assess_declared_heating_table(&input)
            .issues
            .iter()
            .any(|issue| issue.code == "declaration_demand_axis_mismatch"));
    }

    #[test]
    fn published_2025_water_water_excerpt_uses_its_own_table_and_stays_unverified() {
        let mut input: DeclaredHeatingTableInput = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20260143gg-heating-grid-sample.json"
        ))
        .unwrap();
        let corner = assess_declared_heating_table(&input);
        assert_eq!(corner.generation_efficiency, Some(7.507));
        assert_eq!(corner.preferred_energy_fraction, Some(0.974));
        assert_eq!(corner.auxiliary_electricity_kwh_per_year, Some(63.0));
        input.gross_heat_demand_kwh_per_year = 8333.5;
        input.design_supply_temperature_c = 32.5;
        let midpoint = assess_declared_heating_table(&input);
        assert!((midpoint.generation_efficiency.unwrap() - 7.2615).abs() < 1e-12);
        assert!((midpoint.preferred_energy_fraction.unwrap() - 0.8485).abs() < 1e-12);
        assert_eq!(midpoint.auxiliary_electricity_kwh_per_year, Some(71.25));
        assert!(!midpoint.declaration_edition_matches_target);
        assert!(!midpoint.reference_verified && !midpoint.annual_performance_available);
    }

    #[test]
    fn explicitly_open_first_temperature_class_uses_first_row_without_extrapolation() {
        let mut input = fixture();
        input.design_supply_temperature_c = 25.0;
        let result = assess_declared_heating_table(&input);
        assert_eq!(result.status, "input_valid");
        assert_eq!(result.generation_efficiency, Some(6.155));
        let bracket = result.interpolation.unwrap();
        assert_eq!(bracket.temperature_lower_c, 30.0);
        assert_eq!(bracket.temperature_upper_c, 30.0);
        assert_eq!(bracket.temperature_weight, 0.0);
        input.first_row_covers_lower_temperatures = false;
        let bounded = assess_declared_heating_table(&input);
        assert_eq!(bounded.status, "invalid");
        assert_eq!(bounded.generation_efficiency, None);
        assert!(bounded
            .issues
            .iter()
            .any(|issue| issue.code == "declaration_temperature_out_of_range"));
    }
}
