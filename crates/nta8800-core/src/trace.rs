//! Calculation trace ("rekenspoor", feedback 9 Oct 2026): every step of the
//! result as quantity, formula, the values the kernel used, result, unit and
//! the NTA 8800 formula number, down to the input with its source. The steps
//! are built from the values the kernel computed and recorded (no second
//! calculation); the trace tests check that every sum equals the kernel's own
//! result.
//!
//! First module: transmission (chapter 7/8): per zone `H_D` from the
//! elements (8.1), `H_tr` (7.16) and `Q_H;tr` per month (7.14).

use serde::Serialize;

use crate::monthly_demand::{MonthlyDemandAssessment, MonthlyDemandInput, Transmission};

const MONTHS: [&str; 12] = ["jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec"];

/// One step of the calculation trace.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceStep {
    /// Stable id within the trace, e.g. `z1.transmission.hd.gevel-1`.
    pub key: String,
    /// Quantity or item, e.g. `H_D` or the element id.
    pub quantity: String,
    /// Formula in symbols, e.g. `Σ A·U + Σ L·ψ + Σ χ`.
    pub formula: String,
    /// The formula with the values the kernel used (decimal point).
    pub values: String,
    pub result: f64,
    pub unit: &'static str,
    /// NTA 8800 formula, table or paragraph number.
    pub reference: &'static str,
    /// Source of an input value (survey answer, table, declaration).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Keys of steps elsewhere in the trace this step uses (shared steps are
    /// listed once and referred to).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub uses: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TraceStep>,
}

/// A number as the trace shows it: at most three decimals, no trailing zeros.
pub fn number(value: f64) -> String {
    let text = format!("{value:.3}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" { "0".into() } else { text.to_string() }
}

fn leaf(key: String, quantity: String, formula: &str, values: String, result: f64, unit: &'static str, reference: &'static str) -> TraceStep {
    TraceStep { key, quantity, formula: formula.into(), values, result, unit, reference, source: None, uses: Vec::new(), children: Vec::new() }
}

/// Transmission of one zone: `H_D` per element, `H_tr` and `Q_H;tr` per
/// month. `None` without a transmission result (an invalid zone).
pub fn transmission_trace(input: &MonthlyDemandInput, result: &MonthlyDemandAssessment) -> Option<TraceStep> {
    let summary = result.transmission.as_ref()?;
    let zone = &input.zone_id;
    let mut h_tr_children = Vec::new();
    if let Transmission::Components(components) = &input.transmission {
        let direct = &components.direct;
        let mut parts = Vec::new();
        let mut terms = Vec::new();
        for element in &direct.elements {
            let mut step = leaf(
                format!("{zone}.transmission.hd.{}", element.id),
                element.id.clone(),
                "A·U",
                format!("{} × {}", number(element.area_m2), number(element.u_value_w_per_m2k)),
                element.area_m2 * element.u_value_w_per_m2k,
                "W/K",
                "8.1",
            );
            step.source = Some(element.source_reference.clone());
            terms.push(format!("{} × {}", number(element.area_m2), number(element.u_value_w_per_m2k)));
            parts.push(step);
        }
        for bridge in &direct.linear_bridges {
            let mut step = leaf(
                format!("{zone}.transmission.hd.{}", bridge.id),
                bridge.id.clone(),
                "L·ψ",
                format!("{} × {}", number(bridge.length_m), number(bridge.psi_w_per_mk)),
                bridge.length_m * bridge.psi_w_per_mk,
                "W/K",
                "8.1",
            );
            step.source = Some(bridge.source_reference.clone());
            terms.push(format!("{} × {}", number(bridge.length_m), number(bridge.psi_w_per_mk)));
            parts.push(step);
        }
        for bridge in &direct.point_bridges {
            let mut step = leaf(
                format!("{zone}.transmission.hd.{}", bridge.id),
                bridge.id.clone(),
                "χ",
                number(bridge.chi_w_per_k),
                bridge.chi_w_per_k,
                "W/K",
                "8.1",
            );
            step.source = Some(bridge.source_reference.clone());
            terms.push(number(bridge.chi_w_per_k));
            parts.push(step);
        }
        if let Some(h_d) = summary.direct_conductance_w_per_k {
            let mut step = leaf(
                format!("{zone}.transmission.hd"),
                "H_D".into(),
                "Σ A·U + Σ L·ψ + Σ χ",
                terms.join(" + "),
                h_d,
                "W/K",
                "8.1",
            );
            step.children = parts;
            h_tr_children.push(step);
        }
    }
    let mut h_tr_terms = Vec::new();
    if let Some(h_d) = summary.direct_conductance_w_per_k {
        h_tr_terms.push(number(h_d));
    }
    if let Some(h_u) = summary.unheated_conductance_w_per_k {
        h_tr_terms.push(number(h_u));
        h_tr_children.push(leaf(format!("{zone}.transmission.hu"), "H_U".into(), "Σ b_U · H_iu", number(h_u), h_u, "W/K", "8.4"));
    }
    if let Some(h_p) = summary.vertical_pipe_conductance_w_per_k {
        h_tr_terms.push(number(h_p));
        h_tr_children.push(leaf(format!("{zone}.transmission.hp"), "H_p".into(), "Σ H_p;j", number(h_p), h_p, "W/K", "7.17"));
    }
    let h_tr_key = format!("{zone}.transmission.htr");
    let mut h_tr = leaf(
        h_tr_key.clone(),
        "H_tr".into(),
        "H_D + H_U + H_p",
        h_tr_terms.join(" + "),
        summary.conductance_w_per_k,
        "W/K",
        "7.16",
    );
    h_tr.children = h_tr_children;

    let months: Vec<TraceStep> = result
        .monthly
        .iter()
        .map(|month| {
            let terms = &month.heating;
            let mut step = leaf(
                format!("{zone}.transmission.qhtr.{}", month.month),
                format!("Q_H;tr {}", MONTHS[usize::from(month.month - 1)]),
                "H_tr · (θ_int;calc;H − θ_e) · t / 1000 + Q_H;tr;g",
                format!(
                    "{} × ({} − {}) × {} / 1000 + {}",
                    number(month.transmission_conductance_w_per_k),
                    number(terms.calculation_temperature_c),
                    number(month.outdoor_temperature_c),
                    number(month.hours),
                    number(terms.ground_transmission_kwh),
                ),
                terms.transmission_kwh,
                "kWh",
                "7.14",
            );
            step.uses = vec![h_tr_key.clone()];
            step
        })
        .collect();
    let annual: f64 = result.monthly.iter().map(|month| month.heating.transmission_kwh).sum();
    let mut root = leaf(
        format!("{zone}.transmission"),
        format!("Transmissie {zone}"),
        "Σ Q_H;tr over de maanden",
        result.monthly.iter().map(|month| number(month.heating.transmission_kwh)).collect::<Vec<_>>().join(" + "),
        annual,
        "kWh",
        "7.14",
    );
    root.children = std::iter::once(h_tr).chain(months).collect();
    Some(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sum of the children's results, for the consistency checks.
    fn children_sum(step: &TraceStep) -> f64 {
        step.children.iter().map(|child| child.result).sum()
    }

    #[test]
    fn numbers_trim_trailing_zeros() {
        assert_eq!(number(1.250), "1.25");
        assert_eq!(number(54.85), "54.85");
        assert_eq!(number(2.0), "2");
        assert_eq!(number(-0.0001), "0");
    }

    #[test]
    fn transmission_trace_matches_the_kernel_results() {
        let project: serde_json::Value = serde_json::from_str(include_str!("../../../training-data/nta8800-example-terraced-dwelling.json")).unwrap();
        let assessment = crate::project_performance::assess_project_performance(&project);
        let input = assessment.derived_input.as_ref().expect("derived input");
        let performance = assessment.performance.as_ref().expect("performance");
        for (zone, result) in input.zone_inputs().into_iter().zip(std::iter::once(&performance.space_heating.demand).chain(&performance.space_heating.additional_zone_demands)) {
            let trace = transmission_trace(zone, result).expect("trace");
            let summary = result.transmission.as_ref().unwrap();
            let h_tr = &trace.children[0];
            assert_eq!(h_tr.result, summary.conductance_w_per_k);
            // H_D is the sum of its elements, bridges and points.
            let h_d = h_tr.children.iter().find(|step| step.quantity == "H_D").expect("H_D");
            assert!((children_sum(h_d) - h_d.result).abs() < 1e-9, "H_D {} vs Σ {}", h_d.result, children_sum(h_d));
            assert!(h_d.children.iter().all(|step| step.source.as_deref().is_some_and(|source| !source.is_empty())));
            // Every month: the recorded H_tr, temperatures and ground part give the kernel's Q_H;tr.
            for (step, month) in trace.children[1..].iter().zip(&result.monthly) {
                let rebuilt = month.transmission_conductance_w_per_k
                    * (month.heating.calculation_temperature_c - month.outdoor_temperature_c)
                    * month.hours
                    / 1000.0
                    + month.heating.ground_transmission_kwh;
                assert!((rebuilt - step.result).abs() < 1e-9, "month {} {} vs {}", month.month, rebuilt, step.result);
            }
            assert!((trace.result - trace.children[1..].iter().map(|step| step.result).sum::<f64>()).abs() < 1e-9);
        }
    }
}
