//! Calculation trace ("rekenspoor", feedback 9 Oct 2026): every step of the
//! result as quantity, formula, the values the kernel used, result, unit and
//! the NTA 8800 formula number, down to the input with its source. The steps
//! are built from the values the kernel computed and recorded (no second
//! calculation); the trace tests check that every sum equals the kernel's own
//! result.
//!
//! Modules: transmission (chapter 7/8): per zone `H_D` from the elements
//! (8.1), `H_tr` (7.16) and `Q_H;tr` per month (7.14); ventilation: per
//! month `H_H;ve` from the air flows with their supply-temperature factor
//! (7.19/7.20) and `Q_H;ve` (7.18); heat and cold balance: per month
//! `Q_H;nd` (7.1–7.5) and `Q_C;nd` (7.6–7.9) from the heat transfer, the
//! gains and the utilisation factor with its time constant (7.45–7.58);
//! installations: the energy use per energy function and carrier (5.20)
//! and the heating chain of chapter 9 per month (9.1–9.24); primary energy
//! and label: E_P;Tot per carrier (5.9–5.14a), EP2 (5.2), RER (5.3) and the
//! indicative label class; under the installations also the chains of hot
//! water (13.3–13.25), cooling (10.1–10.7), the fans (11.133) and lighting
//! (14.6/14.7), and the PV yield (16.2).

use serde::Serialize;

use crate::monthly_demand::{
    supply_temperature_factor, MonthlyDemandAssessment, MonthlyDemandInput, Transmission,
};

const MONTHS: [&str; 12] = [
    "jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
];

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
    if text == "-0" {
        "0".into()
    } else {
        text.to_string()
    }
}

fn leaf(
    key: String,
    quantity: String,
    formula: &str,
    values: String,
    result: f64,
    unit: &'static str,
    reference: &'static str,
) -> TraceStep {
    TraceStep {
        key,
        quantity,
        formula: formula.into(),
        values,
        result,
        unit,
        reference,
        source: None,
        uses: Vec::new(),
        children: Vec::new(),
    }
}

/// Transmission of one zone: `H_D` per element, `H_tr` and `Q_H;tr` per
/// month. `None` without a transmission result (an invalid zone).
pub fn transmission_trace(
    input: &MonthlyDemandInput,
    result: &MonthlyDemandAssessment,
) -> Option<TraceStep> {
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
                format!(
                    "{} × {}",
                    number(element.area_m2),
                    number(element.u_value_w_per_m2k)
                ),
                element.area_m2 * element.u_value_w_per_m2k,
                "W/K",
                "8.1",
            );
            step.source = Some(element.source_reference.clone());
            terms.push(format!(
                "{} × {}",
                number(element.area_m2),
                number(element.u_value_w_per_m2k)
            ));
            parts.push(step);
        }
        for bridge in &direct.linear_bridges {
            let mut step = leaf(
                format!("{zone}.transmission.hd.{}", bridge.id),
                bridge.id.clone(),
                "L·ψ",
                format!(
                    "{} × {}",
                    number(bridge.length_m),
                    number(bridge.psi_w_per_mk)
                ),
                bridge.length_m * bridge.psi_w_per_mk,
                "W/K",
                "8.1",
            );
            step.source = Some(bridge.source_reference.clone());
            terms.push(format!(
                "{} × {}",
                number(bridge.length_m),
                number(bridge.psi_w_per_mk)
            ));
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
        h_tr_children.push(leaf(
            format!("{zone}.transmission.hu"),
            "H_U".into(),
            "Σ b_U · H_iu",
            number(h_u),
            h_u,
            "W/K",
            "8.4",
        ));
    }
    if let Some(h_p) = summary.vertical_pipe_conductance_w_per_k {
        h_tr_terms.push(number(h_p));
        h_tr_children.push(leaf(
            format!("{zone}.transmission.hp"),
            "H_p".into(),
            "Σ H_p;j",
            number(h_p),
            h_p,
            "W/K",
            "7.17",
        ));
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
    let annual: f64 = result
        .monthly
        .iter()
        .map(|month| month.heating.transmission_kwh)
        .sum();
    let mut root = leaf(
        format!("{zone}.transmission"),
        format!("Transmissie {zone}"),
        "Σ Q_H;tr over de maanden",
        result
            .monthly
            .iter()
            .map(|month| number(month.heating.transmission_kwh))
            .collect::<Vec<_>>()
            .join(" + "),
        annual,
        "kWh",
        "7.14",
    );
    root.children = std::iter::once(h_tr).chain(months).collect();
    Some(root)
}

/// Ventilation of one zone: per month `H_H;ve` as the sum of the air flows,
/// each with its supply-temperature factor (7.19/7.20), and `Q_H;ve` (7.18).
/// The flows of a chapter 11 ventilation are the ones the kernel derived for
/// this result. `None` without a monthly result or without air flows.
pub fn ventilation_trace(
    input: &MonthlyDemandInput,
    result: &MonthlyDemandAssessment,
) -> Option<TraceStep> {
    let resolved = crate::monthly_demand::with_resolved_ventilation(input, result);
    let input = &resolved;
    if result.monthly.is_empty() || input.ventilation_flows.is_empty() {
        return None;
    }
    let zone = &input.zone_id;
    let setpoint = input.setpoints.heating_c;
    let months: Vec<TraceStep> = result
        .monthly
        .iter()
        .map(|month| {
            let name = MONTHS[usize::from(month.month - 1)];
            let outdoor = month.outdoor_temperature_c;
            let mut terms = Vec::new();
            let flows: Vec<TraceStep> = input
                .ventilation_flows
                .iter()
                .filter_map(|flow| {
                    let row = flow.months.iter().find(|row| row.month == month.month)?;
                    let supply = row.supply_temperature_c.unwrap_or(outdoor);
                    let factor = supply_temperature_factor(setpoint, supply, outdoor);
                    terms.push(format!(
                        "{} × {}",
                        number(row.conductance_w_per_k),
                        number(factor)
                    ));
                    let mut step = leaf(
                        format!("{zone}.ventilation.hve.{}.{}", month.month, flow.id),
                        flow.id.clone(),
                        "H_k · (θ_set − θ_sup) / (θ_set − θ_e)",
                        format!(
                            "{} × ({} − {}) / ({} − {})",
                            number(row.conductance_w_per_k),
                            number(setpoint),
                            number(supply),
                            number(setpoint),
                            number(outdoor)
                        ),
                        row.conductance_w_per_k * factor,
                        "W/K",
                        "7.19/7.20",
                    );
                    step.source = Some(flow.source_reference.clone());
                    Some(step)
                })
                .collect();
            let terms_heating = &month.heating;
            let mut h_ve = leaf(
                format!("{zone}.ventilation.hve.{}", month.month),
                format!("H_H;ve {name}"),
                "Σ H_k · b_v;k",
                terms.join(" + "),
                terms_heating.ventilation_conductance_w_per_k,
                "W/K",
                "7.19",
            );
            h_ve.children = flows;
            let mut step = leaf(
                format!("{zone}.ventilation.qhve.{}", month.month),
                format!("Q_H;ve {name}"),
                "H_H;ve · (θ_int;calc;H − θ_e) · t / 1000",
                format!(
                    "{} × ({} − {}) × {} / 1000",
                    number(terms_heating.ventilation_conductance_w_per_k),
                    number(terms_heating.calculation_temperature_c),
                    number(outdoor),
                    number(month.hours),
                ),
                terms_heating.ventilation_kwh,
                "kWh",
                "7.18",
            );
            step.children = vec![h_ve];
            step
        })
        .collect();
    let annual: f64 = result
        .monthly
        .iter()
        .map(|month| month.heating.ventilation_kwh)
        .sum();
    let mut root = leaf(
        format!("{zone}.ventilation"),
        format!("Ventilatie {zone}"),
        "Σ Q_H;ve over de maanden",
        result
            .monthly
            .iter()
            .map(|month| number(month.heating.ventilation_kwh))
            .collect::<Vec<_>>()
            .join(" + "),
        annual,
        "kWh",
        "7.18",
    );
    root.children = months;
    Some(root)
}

/// The heat balance (7.3) or the cold balance (7.7) of one zone: per month
/// the need from the heat transfer (7.10/7.11), the gains (7.12/7.13), the
/// balance ratio (7.50/7.55) and the utilisation factor (7.46–7.54) with its
/// time constant (7.57/7.58); with the recoverable system losses of 7.4/7.5
/// (7.8/7.9) when the kernel applied them. `None` without a monthly result.
pub fn balance_trace(
    input: &MonthlyDemandInput,
    result: &MonthlyDemandAssessment,
    heating: bool,
) -> Option<TraceStep> {
    if result.monthly.is_empty() {
        return None;
    }
    let summary = result.transmission.as_ref()?;
    let zone = &input.zone_id;
    let (s, tag) = if heating {
        ("H", "heating")
    } else {
        ("C", "cooling")
    };
    let eta = if heating { "η_H;gn" } else { "η_C;ht" };
    let ground_adjusted = if heating {
        summary.ground_heating_adjusted_w_per_k
    } else {
        summary.ground_cooling_adjusted_w_per_k
    };
    let capacity = result
        .specific_heat_capacity_kj_per_m2k
        .map(|d_m| (d_m, d_m * 1000.0 * input.usable_floor_area_m2));
    let months: Vec<TraceStep> = result
        .monthly
        .iter()
        .map(|month| {
            let terms = if heating { &month.heating } else { &month.cooling };
            let m = month.month;
            let name = MONTHS[usize::from(m - 1)];
            let key = |part: &str| format!("{zone}.{tag}.{part}.{m}");
            let outdoor = month.outdoor_temperature_c;

            // Q_ht = Q_tr + Q_ve (7.10/7.11).
            let mut transfer = leaf(
                key("qht"),
                format!("Q_{s};ht {name}"),
                &format!("Q_{s};tr + Q_{s};ve"),
                format!(
                    "{} + {}",
                    number(terms.transmission_kwh),
                    number(terms.ventilation_kwh)
                ),
                terms.heat_transfer_kwh,
                "kWh",
                if heating { "7.10" } else { "7.11" },
            );
            if heating {
                transfer.uses = vec![
                    format!("{zone}.transmission.qhtr.{m}"),
                    format!("{zone}.ventilation.qhve.{m}"),
                ];
            } else {
                transfer.children = vec![
                    leaf(
                        key("qtr"),
                        format!("Q_C;tr {name}"),
                        "H_tr · (θ_int;calc;C − θ_e) · t / 1000 + Q_C;tr;g",
                        format!(
                            "{} × ({} − {}) × {} / 1000 + {}",
                            number(month.transmission_conductance_w_per_k),
                            number(terms.calculation_temperature_c),
                            number(outdoor),
                            number(month.hours),
                            number(terms.ground_transmission_kwh),
                        ),
                        terms.transmission_kwh,
                        "kWh",
                        "7.15",
                    ),
                    leaf(
                        key("qve"),
                        format!("Q_C;ve {name}"),
                        "H_C;ve · (θ_int;calc;C − θ_e) · t / 1000",
                        format!(
                            "{} × ({} − {}) × {} / 1000",
                            number(terms.ventilation_conductance_w_per_k),
                            number(terms.calculation_temperature_c),
                            number(outdoor),
                            number(month.hours),
                        ),
                        terms.ventilation_kwh,
                        "kWh",
                        "7.18",
                    ),
                ];
            }

            // Q_gn = Q_int + Q_sol (7.12/7.13); Q_sol per window, opaque and sunroom (7.30/7.31).
            let (window_total, sunroom) = if heating {
                (month.window_solar_gains_kwh, month.sunroom_gains_kwh)
            } else {
                (month.window_solar_cooling_kwh, month.sunroom_cooling_gains_kwh)
            };
            let mut solar_children: Vec<TraceStep> = month
                .window_solar_by_window
                .iter()
                .map(|window| {
                    let value = if heating {
                        window.heating_kwh
                    } else {
                        window.cooling_kwh
                    };
                    leaf(
                        format!("{}.{}", key("qsol"), window.id),
                        window.id.clone(),
                        &format!("Q_{s};sol;wi"),
                        number(value),
                        value,
                        "kWh",
                        "7.32",
                    )
                })
                .collect();
            solar_children.push(leaf(
                format!("{}.opaque", key("qsol")),
                format!("Q_{s};sol;op"),
                &format!("Σ Q_{s};sol;op;k"),
                number(month.opaque_solar_gains_kwh),
                month.opaque_solar_gains_kwh,
                "kWh",
                "7.33",
            ));
            if sunroom != 0.0 {
                solar_children.push(leaf(
                    format!("{}.sunroom", key("qsol")),
                    format!("Q_{s};sol;ind"),
                    "Q_sol;ind via aangrenzende serre",
                    number(sunroom),
                    sunroom,
                    "kWh",
                    "7.30b",
                ));
            }
            let solar_value = window_total + month.opaque_solar_gains_kwh + sunroom;
            let mut solar = leaf(
                key("qsol"),
                format!("Q_{s};sol {name}"),
                &format!("Σ Q_{s};sol;wi + Σ Q_{s};sol;op + Q_{s};sol;ind"),
                solar_children
                    .iter()
                    .map(|step| number(step.result))
                    .collect::<Vec<_>>()
                    .join(" + "),
                solar_value,
                "kWh",
                "7.31",
            );
            solar.children = solar_children;
            let internal = leaf(
                key("qint"),
                format!("Q_{s};int {name}"),
                &format!("Q_{s};int"),
                number(month.internal_gains_kwh),
                month.internal_gains_kwh,
                "kWh",
                "7.21/7.25",
            );
            let mut gains = leaf(
                key("qgn"),
                format!("Q_{s};gn {name}"),
                &format!("Q_{s};int + Q_{s};sol"),
                format!(
                    "{} + {}",
                    number(month.internal_gains_kwh),
                    number(solar_value)
                ),
                terms.gains_kwh,
                "kWh",
                if heating { "7.12" } else { "7.13" },
            );
            gains.children = vec![internal, solar];

            // γ (7.50/7.55), τ (7.57/7.58) with C_m (7.45), a (7.51/7.56) and η (7.46–7.54).
            let gamma_step = terms.gamma.map(|gamma| {
                leaf(
                    key("gamma"),
                    format!("γ_{s} {name}"),
                    &format!("Q_{s};gn / Q_{s};ht"),
                    format!(
                        "{} / {}",
                        number(terms.gains_kwh),
                        number(terms.heat_transfer_kwh)
                    ),
                    gamma,
                    "-",
                    if heating { "7.50" } else { "7.55" },
                )
            });
            let conductances = format!(
                "{} + {} + {}",
                number(month.transmission_conductance_w_per_k),
                number(ground_adjusted),
                number(terms.ventilation_conductance_w_per_k)
            );
            let mut tau = leaf(
                key("tau"),
                format!("τ_{s} {name}"),
                &format!("C_m;int;eff / 3600 / (H_tr + H_{s};g;adj + H_{s};ve)"),
                match capacity {
                    Some((_, c_m)) => format!("{} / 3600 / ({conductances})", number(c_m)),
                    None => number(terms.time_constant_h),
                },
                terms.time_constant_h,
                "h",
                if heating { "7.57" } else { "7.58" },
            );
            if let Some((d_m, c_m)) = capacity {
                tau.children = vec![leaf(
                    format!("{zone}.{tag}.cm"),
                    "C_m;int;eff".into(),
                    "D_m;int;eff · 1000 · A_g",
                    format!(
                        "{} × 1000 × {}",
                        number(d_m),
                        number(input.usable_floor_area_m2)
                    ),
                    c_m,
                    "J/K",
                    "7.45",
                )];
            }
            let mut a = leaf(
                key("a"),
                format!("a_{s} {name}"),
                &format!("a_{s};0 + τ_{s} / τ_{s};0"),
                format!(
                    "{} + {} / {}",
                    number(crate::monthly_demand::A_0),
                    number(terms.time_constant_h),
                    number(crate::monthly_demand::TAU_0_H)
                ),
                terms.a,
                "-",
                if heating { "7.51" } else { "7.56" },
            );
            a.children = vec![tau];
            let (eta_formula, eta_values, eta_reference) = match terms.gamma {
                None => ("1".to_string(), "1".to_string(), if heating { "7.49" } else { "7.54" }),
                Some(gamma) if (gamma - 1.0).abs() < 1e-9 => (
                    format!("a_{s} / (a_{s} + 1)"),
                    format!("{0} / ({0} + 1)", number(terms.a)),
                    if heating { "7.47" } else { "7.53" },
                ),
                Some(gamma) if gamma <= 0.0 => {
                    if heating && terms.gains_kwh > 0.0 {
                        ("1 / γ_H".into(), format!("1 / {}", number(gamma)), "7.48")
                    } else {
                        ("1".into(), "1".into(), if heating { "7.49" } else { "7.54" })
                    }
                }
                Some(gamma) if heating => (
                    "(1 − γ_H^a_H) / (1 − γ_H^(a_H + 1))".into(),
                    format!(
                        "(1 − {0}^{1}) / (1 − {0}^({1} + 1))",
                        number(gamma),
                        number(terms.a)
                    ),
                    "7.46",
                ),
                Some(gamma) => (
                    "(1 − γ_C^−a_C) / (1 − γ_C^−(a_C + 1))".into(),
                    format!(
                        "(1 − {0}^−{1}) / (1 − {0}^−({1} + 1))",
                        number(gamma),
                        number(terms.a)
                    ),
                    "7.52",
                ),
            };
            let mut utilization = leaf(
                key("eta"),
                format!("{eta} {name}"),
                &eta_formula,
                eta_values,
                terms.utilization,
                "-",
                eta_reference,
            );
            utilization.children = vec![a];
            if gamma_step.is_some() {
                utilization.uses = vec![key("gamma")];
            }

            // Q_nd: the gates 7.1/7.2 (7.6), otherwise 7.3 (7.7).
            let net = terms.recoverable_net_kwh;
            let with_losses = result.recoverable_losses_applied;
            let mut extra = Vec::new();
            let gated = if heating {
                match terms.gamma {
                    Some(gamma) if gamma <= 0.0 && terms.gains_kwh > 0.0 => Some((
                        "γ_H ≤ 0 en Q_H;gn > 0 → 0".to_string(),
                        number(gamma),
                        "7.1",
                    )),
                    Some(gamma) if gamma > crate::monthly_demand::GAMMA_H_MAX => Some((
                        "γ_H > 2,0 → 0".to_string(),
                        number(gamma),
                        "7.2",
                    )),
                    None => Some(("Q_H;ht = 0 → 0".to_string(), "0".into(), "7.3")),
                    _ => None,
                }
            } else if terms.gains_kwh <= 0.0 {
                Some(("Q_C;gn ≤ 0 → 0".to_string(), number(terms.gains_kwh), "7.6"))
            } else {
                match terms.gamma {
                    Some(gamma) if gamma > 0.0 && 1.0 / gamma > 2.0 => Some((
                        "1 / γ_C > 2,0 → 0".to_string(),
                        format!("1 / {}", number(gamma)),
                        "7.6",
                    )),
                    _ => None,
                }
            };
            let (formula, values, reference) = match gated {
                Some(gate) => gate,
                None if heating && with_losses => {
                    let gains_incl = terms.gains_kwh + net;
                    let eta_incl = crate::monthly_demand::heating_utilization(
                        gains_incl / terms.heat_transfer_kwh,
                        terms.a,
                        gains_incl,
                    );
                    let delta = eta_incl - terms.utilization;
                    extra.push(rbl_step(key("rbl"), s, name, net));
                    extra.push(delta_step(key("deta"), eta, name, eta_incl, terms.utilization, "7.4"));
                    (
                        "Q_H;ht − η_H;gn · Q_H;gn − Δη_H;gn · Q_H;gn − η_H;gn · (Q_H;ls;rbl − Q_C;ls;rbl)".into(),
                        format!(
                            "{0} − {1} × {2} − {3} × {2} − {1} × {4}",
                            number(terms.heat_transfer_kwh),
                            number(terms.utilization),
                            number(terms.gains_kwh),
                            number(delta),
                            number(net)
                        ),
                        "7.3",
                    )
                }
                None if heating => (
                    "Q_H;ht − η_H;gn · Q_H;gn".into(),
                    format!(
                        "{} − {} × {}",
                        number(terms.heat_transfer_kwh),
                        number(terms.utilization),
                        number(terms.gains_kwh)
                    ),
                    "7.3",
                ),
                None if with_losses => {
                    let eta_incl = if terms.heat_transfer_kwh != 0.0 {
                        crate::monthly_demand::cooling_utilization(
                            (terms.gains_kwh + net) / terms.heat_transfer_kwh,
                            terms.a,
                        )
                    } else {
                        1.0
                    };
                    let delta = eta_incl - terms.utilization;
                    extra.push(rbl_step(key("rbl"), s, name, net));
                    extra.push(delta_step(key("deta"), eta, name, eta_incl, terms.utilization, "7.8"));
                    (
                        "a_C;red · (Q_C;gn − η_C;ht · Q_C;ht + (Q_H;ls;rbl − Q_C;ls;rbl) − Δη_C;ht · Q_C;ht)".into(),
                        format!(
                            "{0} × ({1} − {2} × {3} + {4} − {5} × {3})",
                            number(terms.reduction_factor),
                            number(terms.gains_kwh),
                            number(terms.utilization),
                            number(terms.heat_transfer_kwh),
                            number(net),
                            number(delta)
                        ),
                        "7.7",
                    )
                }
                None => (
                    "a_C;red · (Q_C;gn − η_C;ht · Q_C;ht)".into(),
                    format!(
                        "{} × ({} − {} × {})",
                        number(terms.reduction_factor),
                        number(terms.gains_kwh),
                        number(terms.utilization),
                        number(terms.heat_transfer_kwh)
                    ),
                    "7.7",
                ),
            };
            let mut need = leaf(
                key("qnd"),
                format!("Q_{s};nd {name}"),
                &formula,
                values,
                terms.need_kwh,
                "kWh",
                reference,
            );
            need.children = vec![transfer, gains];
            need.children.extend(gamma_step);
            need.children.push(utilization);
            if !heating {
                need.children.push(leaf(
                    key("ared"),
                    format!("a_C;red {name}"),
                    "(1 − f_C;red;wknd) + b_C;red;wknd · f_C;red;wknd",
                    number(terms.reduction_factor),
                    terms.reduction_factor,
                    "-",
                    "7.74",
                ));
            }
            need.children.extend(extra);
            need
        })
        .collect();
    let annual: f64 = months.iter().map(|step| step.result).sum();
    let mut root = leaf(
        format!("{zone}.{tag}"),
        format!(
            "{} {zone}",
            if heating {
                "Warmtebalans"
            } else {
                "Koudebalans"
            }
        ),
        &format!("Σ Q_{s};nd over de maanden"),
        months
            .iter()
            .map(|step| number(step.result))
            .collect::<Vec<_>>()
            .join(" + "),
        annual,
        "kWh",
        if heating { "7.3" } else { "7.7" },
    );
    root.children = months;
    Some(root)
}

/// `Q_H;ls;rbl − Q_C;ls;rbl` of one month (7.5/7.9).
fn rbl_step(key: String, s: &str, name: &str, net: f64) -> TraceStep {
    leaf(
        key,
        format!("Q_H;ls;rbl − Q_C;ls;rbl {name}"),
        "terugwinbare verliezen van de installaties",
        number(net),
        net,
        "kWh",
        if s == "H" { "7.5" } else { "7.9" },
    )
}

/// `Δη` of 7.4 (7.8): the utilisation with the recoverable losses minus without.
fn delta_step(
    key: String,
    eta: &str,
    name: &str,
    eta_incl: f64,
    eta_excl: f64,
    reference: &'static str,
) -> TraceStep {
    leaf(
        key,
        format!("Δ{eta} {name}"),
        &format!("{eta};incl.rbl − {eta}"),
        format!("{} − {}", number(eta_incl), number(eta_excl)),
        eta_incl - eta_excl,
        "-",
        reference,
    )
}

/// The energy functions of 5.20 with their names in the trace.
const FUNCTION_NAMES: [(&str, &str); 8] = [
    ("heating", "Verwarming"),
    ("humidification", "Bevochtiging"),
    ("ventilation", "Ventilatoren"),
    ("lighting", "Verlichting"),
    ("cooling", "Koeling"),
    ("hotWater", "Warm tapwater"),
    ("auxiliary", "Hulpenergie"),
    ("heatPumpSource", "Warmtepompbron"),
];

/// Installations of the building: the energy use per energy function and
/// carrier (`E_EPus;ci`, 5.20, from the §5.5.3 breakdown) with the months
/// behind every carrier, and under heating the chain of chapter 9 per month:
/// need, emission and distribution losses, the node and the generator output
/// (9.4/9.5/9.24) with the carriers the generators took in (9.1) and the
/// auxiliary energy (9.6). `None` without a breakdown.
pub fn installations_trace(
    performance: &crate::building_performance::BuildingPerformanceAssessment,
) -> Option<TraceStep> {
    let breakdown = &performance.energy_by_service;
    if breakdown.annual.is_empty() {
        return None;
    }
    let mut functions = Vec::new();
    for (service, name) in FUNCTION_NAMES {
        let carriers: Vec<TraceStep> = breakdown
            .annual
            .iter()
            .filter(|row| row.service == service && row.used_kwh != 0.0)
            .map(|row| {
                let months: Vec<String> = breakdown
                    .months
                    .iter()
                    .filter(|month| month.service == service && month.carrier == row.carrier)
                    .map(|month| number(month.used_kwh))
                    .collect();
                leaf(
                    format!("installations.{service}.{}", row.carrier),
                    format!("E_EPus;{}", row.carrier),
                    "Σ E_EPus;ci over de maanden",
                    months.join(" + "),
                    row.used_kwh,
                    "kWh",
                    "5.20",
                )
            })
            .collect();
        if carriers.is_empty() {
            continue;
        }
        let mut step = leaf(
            format!("installations.{service}"),
            name.into(),
            "Σ E_EPus;ci over de energiedragers",
            carriers
                .iter()
                .map(|carrier| number(carrier.result))
                .collect::<Vec<_>>()
                .join(" + "),
            carriers.iter().map(|carrier| carrier.result).sum(),
            "kWh",
            "5.20",
        );
        step.children = carriers;
        match service {
            "heating" => step.children.extend(heating_chain_trace(performance)),
            "hotWater" => step.children.extend(
                performance
                    .hot_water
                    .as_ref()
                    .and_then(hot_water_chain_trace),
            ),
            "cooling" => step
                .children
                .extend(performance.cooling.as_ref().and_then(cooling_chain_trace)),
            "ventilation" => step.children.extend(fans_trace(performance)),
            "lighting" => step.children.extend(lighting_trace(performance)),
            _ => {}
        }
        functions.push(step);
    }
    let mut root = leaf(
        "installations".into(),
        "Installaties".into(),
        "Σ E_EPus over de energiefuncties",
        functions
            .iter()
            .map(|step| number(step.result))
            .collect::<Vec<_>>()
            .join(" + "),
        functions.iter().map(|step| step.result).sum(),
        "kWh",
        "5.20",
    );
    root.children = functions;
    Some(root)
}

/// Chapter 9 per month: from the heating need over the emission and
/// distribution losses and the node to the generator output (9.4/9.5,
/// 9.24 per zone), and the carriers the generators took in (9.1) with the
/// auxiliary energy (9.6). The need links to the heat balance of each zone.
fn heating_chain_trace(
    performance: &crate::building_performance::BuildingPerformanceAssessment,
) -> Option<TraceStep> {
    let chain = &performance.space_heating;
    if chain.monthly.is_empty() {
        return None;
    }
    // The zones' heat balances in this trace (`{zone}.heating`).
    let zones: Vec<&str> = performance
        .trace
        .iter()
        .filter_map(|step| step.key.strip_suffix(".heating"))
        .collect();
    let months: Vec<TraceStep> = chain
        .monthly
        .iter()
        .map(|month| {
            let m = month.month;
            let name = MONTHS[usize::from(m - 1)];
            let key = |part: &str| format!("installations.heating.chain.{part}.{m}");
            let mut need = leaf(
                key("qnd"),
                format!("Q_H;nd {name}"),
                "Σ Q_H;nd;zi",
                number(month.heating_need_kwh),
                month.heating_need_kwh,
                "kWh",
                "9.10",
            );
            need.uses = zones
                .iter()
                .map(|zone| format!("{zone}.heating.qnd.{m}"))
                .collect();
            let terms: [(&str, &str, f64, &'static str); 6] = [
                ("emls", "Q_H;em;ls", month.emission_loss_kwh, "9.9"),
                ("disls", "Q_H;dis;ls", month.distribution_loss_kwh, "9.24"),
                (
                    "disaux",
                    "Q_H;dis;aux;rvd",
                    -month.distribution_auxiliary_to_medium_kwh,
                    "9.24",
                ),
                ("nodls", "Q_H;nod;ls", month.node_loss_kwh, "9.5"),
                (
                    "other",
                    "Q_H;AHU + Q_H;HUM + Q_W;H",
                    month.ahu_heating_load_kwh
                        + month.humidification_load_kwh
                        + month.hot_water_load_kwh,
                    "9.4",
                ),
                ("solar", "Q_H;nod;gns", -month.solar_gain_kwh, "9.5"),
            ];
            let mut parts = vec![need];
            let mut values = vec![number(month.heating_need_kwh)];
            for (part, quantity, value, reference) in terms {
                if value == 0.0 && part != "emls" && part != "disls" {
                    continue;
                }
                values.push(if value < 0.0 {
                    format!("− {}", number(-value))
                } else {
                    format!("+ {}", number(value))
                });
                parts.push(leaf(
                    key(part),
                    format!("{quantity} {name}"),
                    quantity,
                    number(value),
                    value,
                    "kWh",
                    reference,
                ));
            }
            let carriers: Vec<(&str, f64)> = [
                ("gas", month.natural_gas_kwh),
                ("el", month.generator_electricity_kwh),
                ("dh", month.district_heat_kwh),
                ("bm", month.biomass_kwh),
                ("oil", month.oil_kwh),
            ]
            .into_iter()
            .filter(|(_, value)| *value != 0.0)
            .collect();
            let mut input = leaf(
                key("genin"),
                format!("E_H;gen;in {name}"),
                "Σ E_H;gen;in;ci van de opwekkers",
                carriers
                    .iter()
                    .map(|(carrier, value)| format!("{carrier} {}", number(*value)))
                    .collect::<Vec<_>>()
                    .join(" + "),
                carriers.iter().map(|(_, value)| value).sum(),
                "kWh",
                "9.1",
            );
            input.children = carriers
                .iter()
                .map(|(carrier, value)| {
                    leaf(
                        format!("{}.{carrier}", key("genin")),
                        format!("E_H;gen;in;{carrier} {name}"),
                        "opwekker",
                        number(*value),
                        *value,
                        "kWh",
                        "9.1",
                    )
                })
                .collect();
            let mut output = leaf(
                key("genout"),
                format!("Q_H;gen;out {name}"),
                "Q_H;nd + Q_H;em;ls + Q_H;dis;ls − Q_H;dis;aux;rvd + Q_H;nod;ls + Q_H;AHU + Q_H;HUM + Q_W;H − Q_H;nod;gns",
                values.join(" "),
                month.generator_output_kwh,
                "kWh",
                "9.4/9.5",
            );
            output.children = parts;
            let mut step = leaf(
                key("month"),
                format!("Verwarmingsketen {name}"),
                "Q_H;gen;out → E_H;gen;in",
                format!(
                    "{} kWh warmte uit {} kWh energie",
                    number(month.generator_output_kwh),
                    number(input.result)
                ),
                input.result,
                "kWh",
                "9.1",
            );
            step.children = vec![output, input];
            if let Some(aux) = month.auxiliary_electricity_kwh {
                step.children.push(leaf(
                    key("aux"),
                    format!("W_H;aux {name}"),
                    "W_H;em;aux + W_H;dis;aux + W_H;sto;aux + W_H;gen;aux",
                    number(aux),
                    aux,
                    "kWh",
                    "9.6",
                ));
            }
            step
        })
        .collect();
    let mut root = leaf(
        "installations.heating.chain".into(),
        "Verwarmingsketen".into(),
        "Σ E_H;gen;in over de maanden",
        months
            .iter()
            .map(|step| number(step.result))
            .collect::<Vec<_>>()
            .join(" + "),
        months.iter().map(|step| step.result).sum(),
        "kWh",
        "9.1",
    );
    root.children = months;
    Some(root)
}

/// Primary energy and label: `E_P;Tot` from the delivered energy per carrier
/// and energy function times `f_P;del` (5.9–5.11) minus the export credit
/// (5.13) and the battery correction (5.14a), EP2 (5.2), RER (5.3) with
/// `E_Pren;Tot` (5.28) and the indicative label class (Omgevingsregeling
/// annex IX/X). `None` without indicators.
pub fn primary_energy_trace(
    performance: &crate::building_performance::BuildingPerformanceAssessment,
    area_m2: f64,
    residential: bool,
) -> Option<TraceStep> {
    let fossil = performance.annual_primary_fossil_kwh?;
    let ep2 = performance.primary_fossil_indicator_kwh_per_m2_year?;
    let breakdown = &performance.energy_by_service;
    let names: std::collections::HashMap<&str, &str> = FUNCTION_NAMES.into_iter().collect();

    // E_P;del;ci per carrier, with the energy functions behind it (5.11).
    let mut carriers = Vec::new();
    for carrier in crate::building_performance::BREAKDOWN_CARRIERS {
        let functions: Vec<TraceStep> = breakdown
            .annual
            .iter()
            .filter(|row| {
                row.carrier == carrier
                    && (row.delivered_kwh != 0.0 || row.primary_fossil_kwh != 0.0)
            })
            .map(|row| {
                let mut step = leaf(
                    format!("primary.del.{carrier}.{}", row.service),
                    names
                        .get(row.service)
                        .copied()
                        .unwrap_or(row.service)
                        .into(),
                    "E_del;ci · f_P;del;ci",
                    if row.delivered_kwh != 0.0 {
                        format!(
                            "{} × {}",
                            number(row.delivered_kwh),
                            number(row.primary_fossil_kwh / row.delivered_kwh)
                        )
                    } else {
                        number(row.primary_fossil_kwh)
                    },
                    row.primary_fossil_kwh,
                    "kWh",
                    "5.11",
                );
                step.uses = vec![format!("installations.{}.{carrier}", row.service)];
                step
            })
            .collect();
        if functions.is_empty() {
            continue;
        }
        let mut step = leaf(
            format!("primary.del.{carrier}"),
            format!("E_P;del;{carrier}"),
            "Σ E_P;del over de energiefuncties",
            functions
                .iter()
                .map(|function| number(function.result))
                .collect::<Vec<_>>()
                .join(" + "),
            functions.iter().map(|function| function.result).sum(),
            "kWh",
            "5.11",
        );
        step.children = functions;
        carriers.push(step);
    }
    let credit: f64 = breakdown
        .adjustments
        .iter()
        .map(|month| month.exported_electricity_credit_kwh)
        .sum();
    let storage: f64 = breakdown
        .adjustments
        .iter()
        .map(|month| month.storage_correction_kwh)
        .sum();
    let delivered: f64 = carriers.iter().map(|step| step.result).sum();
    let mut parts = carriers;
    if credit != 0.0 {
        parts.push(leaf(
            "primary.exp.el".into(),
            "E_P;exp;el".into(),
            "E_exp;el · f_P;exp;el",
            number(credit),
            credit,
            "kWh",
            "5.13",
        ));
    }
    if storage != 0.0 {
        parts.push(leaf(
            "primary.bat".into(),
            "E_P;BAT,out;tot".into(),
            "batterijcorrectie",
            number(storage),
            storage,
            "kWh",
            "5.14a",
        ));
    }
    let mut ep_tot = leaf(
        "primary.eptot".into(),
        "E_P;Tot".into(),
        "Σ E_P;del;ci − E_P;exp;el − E_P;BAT,out;tot",
        format!(
            "{} − {} − {}",
            number(delivered),
            number(credit),
            number(storage)
        ),
        fossil,
        "kWh",
        "5.9/5.10",
    );
    ep_tot.children = parts;
    let mut ep2_step = leaf(
        "primary.ep2".into(),
        "EP2 (E_wePTot)".into(),
        "E_P;Tot / A_g, naar boven afgerond op 0,01",
        format!("{} / {}", number(fossil), number(area_m2)),
        ep2,
        "kWh/m²",
        "5.2",
    );
    ep2_step.children = vec![
        ep_tot,
        leaf(
            "primary.ag".into(),
            "A_g".into(),
            "gebruiksoppervlakte",
            number(area_m2),
            area_m2,
            "m²",
            "5.2",
        ),
    ];
    let mut children = vec![ep2_step];

    // RER (5.3) with E_Pren;Tot (5.28).
    if let (Some(rer), Some(renewable)) = (
        performance.renewable_share_percent,
        performance.annual_renewable_primary_kwh,
    ) {
        let mut step = leaf(
            "primary.rer".into(),
            "RER (BENG 3)".into(),
            "E_Pren;Tot / (E_P;Tot + E_Pren;Tot) × 100, naar beneden afgerond op 0,1",
            format!("{0} / ({1} + {0}) × 100", number(renewable), number(fossil)),
            rer,
            "%",
            "5.3",
        );
        step.children = vec![leaf(
            "primary.epren".into(),
            "E_Pren;Tot".into(),
            "Σ E_Pren;H + E_Pren;C + E_Pren;W + E_Pren;el",
            number(renewable),
            renewable,
            "kWh",
            "5.28/5.29",
        )];
        step.uses = vec!["primary.eptot".into()];
        children.push(step);
    }

    // Indicative label class from the EP2 of the label scenario.
    if let Some(class) = performance.indicative_label_class {
        let label_ep2 = performance
            .label_primary_fossil_indicator_kwh_per_m2_year
            .unwrap_or(ep2);
        let function = crate::label_class::LabelFunction::Residential;
        let values = if residential {
            match crate::label_class::class_upper_bound(function, class) {
                Some(bound) => format!("{} ≤ {}", number(label_ep2), number(bound)),
                None => crate::label_class::class_upper_bound(function, "F")
                    .map_or(number(label_ep2), |bound| {
                        format!("{} > {}", number(label_ep2), number(bound))
                    }),
            }
        } else {
            number(label_ep2)
        };
        let mut step = leaf(
            "primary.label".into(),
            format!("Energielabel {class} (indicatief)"),
            "klassegrens bij EP2",
            values,
            label_ep2,
            "kWh/m²",
            if residential {
                "Omgevingsregeling bijlage IX"
            } else {
                "Omgevingsregeling bijlage X"
            },
        );
        if (label_ep2 - ep2).abs() > 1e-9 {
            step.children = vec![leaf(
                "primary.label.emgforf".into(),
                "EP2 label (EMGforf)".into(),
                "EP2 met forfaitaire waarden voor gebiedsmaatregelen",
                number(label_ep2),
                label_ep2,
                "kWh/m²",
                "Omgevingsregeling art. 5.11 lid 4",
            )];
        } else {
            step.uses = vec!["primary.ep2".into()];
        }
        children.push(step);
    }

    let mut root = leaf(
        "primary".into(),
        "Primaire energie en label".into(),
        "EP2 → energielabel",
        format!(
            "{} kWh/m²{}",
            number(ep2),
            performance
                .indicative_label_class
                .map(|class| format!(" → {class}"))
                .unwrap_or_default()
        ),
        ep2,
        "kWh/m²",
        "5.2",
    );
    root.children = children;
    Some(root)
}

/// A sum of signed terms as the trace shows it: `a + b − c`.
fn signed_terms(terms: &[f64]) -> String {
    let mut text = String::new();
    for (index, value) in terms.iter().enumerate() {
        if index == 0 {
            text.push_str(&number(*value));
        } else if *value < 0.0 {
            text.push_str(&format!(" − {}", number(-value)));
        } else {
            text.push_str(&format!(" + {}", number(*value)));
        }
    }
    text
}

/// The carriers a generator took in one month, as a step with a leaf per
/// carrier that is not zero.
fn carriers_step(
    key: String,
    quantity: String,
    carriers: &[(&str, f64)],
    reference: &'static str,
) -> TraceStep {
    let used: Vec<&(&str, f64)> = carriers.iter().filter(|(_, value)| *value != 0.0).collect();
    let mut step = leaf(
        key.clone(),
        quantity.clone(),
        "Σ E_in;ci van de opwekkers",
        used.iter()
            .map(|(carrier, value)| format!("{carrier} {}", number(*value)))
            .collect::<Vec<_>>()
            .join(" + "),
        used.iter().map(|(_, value)| value).sum(),
        "kWh",
        reference,
    );
    step.children = used
        .iter()
        .map(|(carrier, value)| {
            leaf(
                format!("{key}.{carrier}"),
                format!("{quantity} · {carrier}"),
                "opwekker",
                number(*value),
                *value,
                "kWh",
                reference,
            )
        })
        .collect();
    step
}

/// Chapter 13 per month: the need over the emission (13.9) and distribution
/// (13.25) to the heat the generators deliver (13.7, 13.4 after solar heat),
/// and the carriers they took in (13.3) with the auxiliary energy.
fn hot_water_chain_trace(
    hot_water: &crate::domestic_hot_water::HotWaterAssessment,
) -> Option<TraceStep> {
    if hot_water.months.is_empty() {
        return None;
    }
    let eta_em = hot_water.emission_efficiency;
    let months: Vec<TraceStep> = hot_water
        .months
        .iter()
        .map(|month| {
            let m = month.month;
            let name = MONTHS[usize::from(m - 1)];
            let key = |part: &str| format!("installations.hotWater.chain.{part}.{m}");
            let mut emission = leaf(
                key("qem"),
                format!("Q_W;em {name}"),
                "Q_W;nd / η_W;em − Q_W;rcd",
                format!(
                    "{} / {} − {}",
                    number(month.net_need_kwh),
                    number(eta_em),
                    number(month.recovered_kwh)
                ),
                month.emission_input_kwh,
                "kWh",
                "13.9",
            );
            emission.children.push(leaf(
                key("qnd"),
                format!("Q_W;nd {name}"),
                "netto warmtebehoefte warm tapwater",
                number(month.net_need_kwh),
                month.net_need_kwh,
                "kWh",
                "13.15/13.19",
            ));
            emission.children.push(leaf(
                key("etaem"),
                "η_W;em".into(),
                "afgifterendement",
                number(eta_em),
                eta_em,
                "-",
                "13.21–13.23",
            ));
            if month.recovered_kwh != 0.0 {
                emission.children.push(leaf(
                    key("qrcd"),
                    format!("Q_W;rcd {name}"),
                    "douchewarmteterugwinning",
                    number(month.recovered_kwh),
                    month.recovered_kwh,
                    "kWh",
                    "13.51",
                ));
            }
            let mut distribution = leaf(
                key("etadis"),
                format!("η_W;dis {name}"),
                "Q_W;em / (Q_W;em + Q_W;dis;ls − Q_W;dis;aux;rvd)",
                number(month.distribution_efficiency),
                month.distribution_efficiency,
                "-",
                "13.25",
            );
            if month.circulation_loss_kwh != 0.0 {
                distribution.children.push(leaf(
                    key("circ"),
                    format!("Q_W;dis;ls {name}"),
                    "circulatieverlies",
                    number(month.circulation_loss_kwh),
                    month.circulation_loss_kwh,
                    "kWh",
                    "13.26",
                ));
            }
            let storage = month.storage_loss_kwh + month.solar_backup_storage_loss_kwh;
            let terms = [
                month.emission_input_kwh / month.distribution_efficiency,
                storage,
                month.conversion_loss_kwh,
                -month.solar_renewable_kwh,
            ];
            let mut output = leaf(
                key("genout"),
                format!("Q_W;dis;nren {name}"),
                "Q_W;em / η_W;dis + Q_W;sto;ls;tot + Q_W;dis;conv;ls − Q_W;ren;sol",
                signed_terms(&terms),
                month.generator_output_kwh,
                "kWh",
                "13.4/13.7",
            );
            output.children = vec![emission, distribution];
            for (part, quantity, value, reference) in [
                ("sto", "Q_W;sto;ls;tot", storage, "13.8/13.58"),
                (
                    "conv",
                    "Q_W;dis;conv;ls",
                    month.conversion_loss_kwh,
                    "13.24",
                ),
                ("sol", "Q_W;ren;sol", month.solar_renewable_kwh, "13.4a"),
            ] {
                if value != 0.0 {
                    output.children.push(leaf(
                        key(part),
                        format!("{quantity} {name}"),
                        quantity,
                        number(value),
                        value,
                        "kWh",
                        reference,
                    ));
                }
            }
            let input = carriers_step(
                key("genin"),
                format!("E_W;gen;in {name}"),
                &[
                    ("el", month.electricity_kwh),
                    ("gas", month.natural_gas_kwh),
                    ("oil", month.oil_kwh),
                    ("dh", month.district_heat_kwh),
                    ("bm", month.biomass_kwh),
                ],
                "13.3",
            );
            let mut step = leaf(
                key("month"),
                format!("Tapwaterketen {name}"),
                "Q_W;dis;nren → E_W;gen;in",
                format!(
                    "{} kWh warmte uit {} kWh energie",
                    number(month.generator_output_kwh),
                    number(input.result)
                ),
                input.result,
                "kWh",
                "13.3",
            );
            step.children = vec![output, input];
            for (part, quantity, formula, value, reference) in [
                (
                    "aux",
                    "W_W;aux",
                    "hulpenergie pomp, elektronica en zonneboiler",
                    month.auxiliary_electricity_kwh,
                    "13.46/13.67",
                ),
                (
                    "amb",
                    "Q_W;amb",
                    "omgevingswarmte van de warmtepomp",
                    month.ambient_heat_kwh,
                    "13.3",
                ),
            ] {
                if value != 0.0 {
                    step.children.push(leaf(
                        key(part),
                        format!("{quantity} {name}"),
                        formula,
                        number(value),
                        value,
                        "kWh",
                        reference,
                    ));
                }
            }
            step
        })
        .collect();
    Some(month_root(
        "installations.hotWater.chain",
        "Tapwaterketen",
        "Σ E_W;gen;in over de maanden",
        months,
        "13.1",
    ))
}

/// Chapter 10 per month: the load on the cooling generators (10.5, less the
/// booster heat pump of 10.7) and the carriers they took in (10.2/10.3)
/// with the auxiliary energy (10.1).
fn cooling_chain_trace(cooling: &crate::space_cooling::CoolingAssessment) -> Option<TraceStep> {
    if cooling.months.is_empty() {
        return None;
    }
    let months: Vec<TraceStep> = cooling
        .months
        .iter()
        .map(|month| {
            let m = month.month;
            let name = MONTHS[usize::from(m - 1)];
            let key = |part: &str| format!("installations.cooling.chain.{part}.{m}");
            let parts = [
                ("qnd", "Q_C;nd", month.need_kwh, "10.5"),
                ("emls", "Q_C;em;ls", month.emission_loss_kwh, "10.5"),
                ("disls", "Q_C;dis;ls", month.distribution_loss_kwh, "10.21"),
                ("aux", "Q_C;dis;aux;rvd", month.pump_recovered_kwh, "10.45"),
                ("dhum", "Q_C;dhum", month.dehumidification_kwh, "12.5"),
                ("ahu", "Q_C;ahu;in;req", month.ahu_cooling_kwh, "11.116"),
                ("hp", "Q_C;HP", -month.booster_extraction_kwh, "10.7"),
            ];
            let shown: Vec<_> = parts
                .iter()
                .filter(|(part, _, value, _)| *value != 0.0 || *part == "qnd")
                .collect();
            let mut load = leaf(
                key("genin"),
                format!("Q_C;gen;in;req {name}"),
                "Q_C;nd + Q_C;em;ls + Q_C;dis;ls + Q_C;dis;aux;rvd + Q_C;dhum + Q_C;ahu − Q_C;HP",
                signed_terms(&shown.iter().map(|item| item.2).collect::<Vec<_>>()),
                month.generator_cold_kwh,
                "kWh",
                "10.5/10.7",
            );
            load.children = shown
                .iter()
                .map(|(part, quantity, value, reference)| {
                    leaf(
                        key(part),
                        format!("{quantity} {name}"),
                        quantity,
                        number(*value),
                        *value,
                        "kWh",
                        reference,
                    )
                })
                .collect();
            let input = carriers_step(
                key("in"),
                format!("E_C;gen;in {name}"),
                &[
                    ("el", month.electricity_kwh),
                    ("gas", month.natural_gas_kwh + month.chp_heat_kwh),
                    ("dh", month.district_heat_kwh),
                    ("dc", month.district_cold_kwh),
                ],
                "10.2/10.3",
            );
            let mut step = leaf(
                key("month"),
                format!("Koelketen {name}"),
                "Q_C;gen;in;req → E_C;gen;in",
                format!(
                    "{} kWh koude uit {} kWh energie",
                    number(month.generator_cold_kwh),
                    number(input.result)
                ),
                input.result,
                "kWh",
                "10.2",
            );
            step.children = vec![load, input];
            if month.auxiliary_electricity_kwh != 0.0 {
                step.children.push(leaf(
                    key("waux"),
                    format!("W_C;aux {name}"),
                    "W_C;aux;gen + W_C;aux;dis + W_C;aux;em",
                    number(month.auxiliary_electricity_kwh),
                    month.auxiliary_electricity_kwh,
                    "kWh",
                    "10.1",
                ));
            }
            step
        })
        .collect();
    Some(month_root(
        "installations.cooling.chain",
        "Koelketen",
        "Σ E_C;gen;in over de maanden",
        months,
        "10.2",
    ))
}

/// Chapter 11 per zone: the fan energy (11.133) with the frost protection
/// (11.105) and the preheating of grilles, per month.
fn fans_trace(
    performance: &crate::building_performance::BuildingPerformanceAssessment,
) -> Vec<TraceStep> {
    let chain = &performance.space_heating;
    std::iter::once(&chain.demand)
        .chain(&chain.additional_zone_demands)
        .filter_map(|demand| demand.ventilation.as_ref())
        .filter(|result| !result.months.is_empty())
        .map(|result| {
            let zone = &result.zone_id;
            let months: Vec<TraceStep> = result
                .months
                .iter()
                .map(|month| {
                    let terms = [
                        month.fan_electricity_kwh,
                        month.frost_protection_electricity_kwh,
                        month.grille_preheating_electricity_kwh,
                    ];
                    leaf(
                        format!("installations.ventilation.{zone}.{}", month.month),
                        format!("E_V {}", MONTHS[usize::from(month.month - 1)]),
                        "W_V;fan + E_V;eldf + E_V;preh",
                        signed_terms(&terms),
                        terms.iter().sum(),
                        "kWh",
                        "11.133/11.105",
                    )
                })
                .collect();
            month_root(
                &format!("installations.ventilation.{zone}"),
                &format!("Ventilatoren {zone}"),
                "Σ E_V over de maanden",
                months,
                "11.133",
            )
        })
        .collect()
}

/// Chapter 14 per zone: per lighting zone `W_t = W_L + W_P` (14.6) with the
/// installed power, the occupancy and daylight factors of `W_L` (14.7).
fn lighting_trace(
    performance: &crate::building_performance::BuildingPerformanceAssessment,
) -> Vec<TraceStep> {
    performance
        .lighting
        .iter()
        .filter(|zone| zone.annual_kwh != 0.0)
        .map(|zone| {
            let key = format!("installations.lighting.{}", zone.zone_id);
            let parts: Vec<TraceStep> = zone
                .lighting_zones
                .iter()
                .map(|item| {
                    let mut step = leaf(
                        format!("{key}.{}", item.id),
                        format!("W_t {}", item.id),
                        "W_L + W_P",
                        format!(
                            "{} + {}",
                            number(item.lighting_kwh),
                            number(item.parasitic_kwh)
                        ),
                        item.lighting_kwh + item.parasitic_kwh,
                        "kWh",
                        "14.6",
                    );
                    step.children = vec![
                        leaf(
                            format!("{key}.{}.wl", item.id),
                            format!("W_L {}", item.id),
                            "P_n · F_C · (t_D · F_o;D · F_D + t_N · F_o;N) / 1000",
                            format!(
                                "P_n {} W, F_o;D {}, F_o;N {}, F_D {}",
                                number(item.installed_power_w),
                                number(item.occupancy_day),
                                number(item.occupancy_night),
                                number(item.daylight_factor)
                            ),
                            item.lighting_kwh,
                            "kWh",
                            "14.7",
                        ),
                        leaf(
                            format!("{key}.{}.wp", item.id),
                            format!("W_P {}", item.id),
                            "parasitair energiegebruik",
                            number(item.parasitic_kwh),
                            item.parasitic_kwh,
                            "kWh",
                            "14.10–14.14",
                        ),
                    ];
                    step
                })
                .collect();
            let mut step = leaf(
                key,
                format!("Verlichting {}", zone.zone_id),
                "Σ W_t over de verlichtingszones, verdeeld over de maanden",
                zone.monthly_kwh
                    .iter()
                    .map(|value| number(*value))
                    .collect::<Vec<_>>()
                    .join(" + "),
                zone.annual_kwh,
                "kWh",
                "14.6",
            );
            step.children = parts;
            step
        })
        .collect()
}

/// Chapter 16: the electricity of every PV system per month (16.2/16.3).
pub fn pv_trace(
    performance: &crate::building_performance::BuildingPerformanceAssessment,
) -> Option<TraceStep> {
    if performance.pv_systems.is_empty() {
        return None;
    }
    let systems: Vec<TraceStep> = performance
        .pv_systems
        .iter()
        .map(|system| {
            leaf(
                format!("pv.{}", system.id),
                format!("E_el;PV;out {}", system.id),
                "E_sol · P_pk · f_perf · c_sh;PV · f_prac;PV / I_ref, per maand",
                system
                    .monthly_kwh
                    .iter()
                    .map(|value| number(*value))
                    .collect::<Vec<_>>()
                    .join(" + "),
                system.annual_kwh,
                "kWh",
                "16.2/16.3",
            )
        })
        .collect();
    let mut root = leaf(
        "pv".into(),
        "Zonnestroom (PV)".into(),
        "Σ E_el;PV;out over de systemen",
        systems
            .iter()
            .map(|step| number(step.result))
            .collect::<Vec<_>>()
            .join(" + "),
        systems.iter().map(|step| step.result).sum(),
        "kWh",
        "16.2",
    );
    root.children = systems;
    Some(root)
}

/// A root that sums its month steps.
fn month_root(
    key: &str,
    quantity: &str,
    formula: &str,
    months: Vec<TraceStep>,
    reference: &'static str,
) -> TraceStep {
    let mut root = leaf(
        key.into(),
        quantity.into(),
        formula,
        months
            .iter()
            .map(|step| number(step.result))
            .collect::<Vec<_>>()
            .join(" + "),
        months.iter().map(|step| step.result).sum(),
        "kWh",
        reference,
    );
    root.children = months;
    root
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
        let project: serde_json::Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-terraced-dwelling.json"
        ))
        .unwrap();
        let assessment = crate::project_performance::assess_project_performance(&project);
        let input = assessment.derived_input.as_ref().expect("derived input");
        let performance = assessment.performance.as_ref().expect("performance");
        for (zone, result) in input.zone_inputs().into_iter().zip(
            std::iter::once(&performance.space_heating.demand)
                .chain(&performance.space_heating.additional_zone_demands),
        ) {
            let trace = transmission_trace(zone, result).expect("trace");
            let summary = result.transmission.as_ref().unwrap();
            let h_tr = &trace.children[0];
            assert_eq!(h_tr.result, summary.conductance_w_per_k);
            // H_D is the sum of its elements, bridges and points.
            let h_d = h_tr
                .children
                .iter()
                .find(|step| step.quantity == "H_D")
                .expect("H_D");
            assert!(
                (children_sum(h_d) - h_d.result).abs() < 1e-9,
                "H_D {} vs Σ {}",
                h_d.result,
                children_sum(h_d)
            );
            assert!(h_d.children.iter().all(|step| step
                .source
                .as_deref()
                .is_some_and(|source| !source.is_empty())));
            // Every month: the recorded H_tr, temperatures and ground part give the kernel's Q_H;tr.
            for (step, month) in trace.children[1..].iter().zip(&result.monthly) {
                let rebuilt = month.transmission_conductance_w_per_k
                    * (month.heating.calculation_temperature_c - month.outdoor_temperature_c)
                    * month.hours
                    / 1000.0
                    + month.heating.ground_transmission_kwh;
                assert!(
                    (rebuilt - step.result).abs() < 1e-9,
                    "month {} {} vs {}",
                    month.month,
                    rebuilt,
                    step.result
                );
            }
            assert!(
                (trace.result
                    - trace.children[1..]
                        .iter()
                        .map(|step| step.result)
                        .sum::<f64>())
                .abs()
                    < 1e-9
            );

            // Ventilation: H_H;ve is the sum of its flows, and Q_H;ve follows from it.
            let ventilation = ventilation_trace(zone, result).expect("ventilation trace");
            for (step, month) in ventilation.children.iter().zip(&result.monthly) {
                let h_ve = &step.children[0];
                assert!(
                    (children_sum(h_ve) - h_ve.result).abs() < 1e-9,
                    "H_ve month {} {} vs Σ {}",
                    month.month,
                    h_ve.result,
                    children_sum(h_ve)
                );
                let rebuilt = h_ve.result
                    * (month.heating.calculation_temperature_c - month.outdoor_temperature_c)
                    * month.hours
                    / 1000.0;
                assert!(
                    (rebuilt - step.result).abs() < 1e-9,
                    "Q_ve month {} {} vs {}",
                    month.month,
                    rebuilt,
                    step.result
                );
            }
            assert!((ventilation.result - children_sum(&ventilation)).abs() < 1e-9);

            // Heat and cold balance: the gains are the sum of their parts, and each month's
            // need equals the kernel's; the annual sum is the kernel's annual need.
            let mut checked_with_losses = 0;
            for heating in [true, false] {
                let balance = balance_trace(zone, result, heating).expect("balance trace");
                for (step, month) in balance.children.iter().zip(&result.monthly) {
                    let terms = if heating {
                        &month.heating
                    } else {
                        &month.cooling
                    };
                    assert_eq!(step.result, terms.need_kwh);
                    let gains = step
                        .children
                        .iter()
                        .find(|child| child.key.contains(".qgn."))
                        .expect("Q_gn");
                    assert!(
                        (children_sum(gains) - gains.result).abs() < 1e-6,
                        "Q_gn month {} {} vs Σ {}",
                        month.month,
                        gains.result,
                        children_sum(gains)
                    );
                    let solar = &gains.children[1];
                    assert!((children_sum(solar) - solar.result).abs() < 1e-6);
                    if !heating {
                        let transfer = &step.children[0];
                        assert!((children_sum(transfer) - transfer.result).abs() < 1e-9);
                    }
                    // Without a gate, 7.3/7.7 rebuilt from the shown terms, with Δη and the
                    // recoverable losses when the kernel applied them.
                    let gated =
                        ["7.1", "7.2", "7.6"].contains(&step.reference) || terms.gamma.is_none();
                    if !gated {
                        let child = |part: &str| {
                            step.children
                                .iter()
                                .find(|child| child.key.contains(part))
                                .map_or(0.0, |child| child.result)
                        };
                        let (net, delta) = (child(".rbl."), child(".deta."));
                        if result.recoverable_losses_applied {
                            checked_with_losses += 1;
                        }
                        let rebuilt = if heating {
                            terms.heat_transfer_kwh
                                - terms.utilization * terms.gains_kwh
                                - delta * terms.gains_kwh
                                - terms.utilization * net
                        } else {
                            terms.reduction_factor
                                * (terms.gains_kwh - terms.utilization * terms.heat_transfer_kwh
                                    + net
                                    - delta * terms.heat_transfer_kwh)
                        };
                        assert!(
                            (rebuilt.max(0.0) - step.result).abs() < 1e-9,
                            "Q_nd month {} heating {heating}: {} vs {}",
                            month.month,
                            rebuilt,
                            step.result
                        );
                    }
                }
                let annual = if heating {
                    result.annual_heating_need_kwh
                } else {
                    result.annual_cooling_need_kwh
                };
                assert!((balance.result - annual.unwrap()).abs() < 1e-6);
            }
            // The example applies the recoverable losses: the full 7.3/7.7 was checked.
            assert!(!result.recoverable_losses_applied || checked_with_losses > 0);
        }
    }

    #[test]
    fn installations_trace_matches_the_kernel_results() {
        let project: serde_json::Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-terraced-dwelling.json"
        ))
        .unwrap();
        let assessment = crate::project_performance::assess_project_performance(&project);
        let performance = assessment.performance.as_ref().expect("performance");
        let trace = installations_trace(performance).expect("installations trace");
        // Σ functions = Σ E_EPus of the breakdown; every function is the sum of its carriers,
        // every carrier the sum of its months.
        let used: f64 = performance
            .energy_by_service
            .annual
            .iter()
            .map(|row| row.used_kwh)
            .sum();
        assert!(
            (trace.result - used).abs() < 1e-6,
            "{} vs {used}",
            trace.result
        );
        for function in &trace.children {
            let carriers: Vec<&TraceStep> = function
                .children
                .iter()
                .filter(|child| child.quantity.starts_with("E_EPus;"))
                .collect();
            let sum: f64 = carriers.iter().map(|carrier| carrier.result).sum();
            assert!(
                (function.result - sum).abs() < 1e-6,
                "{}",
                function.quantity
            );
            for carrier in carriers {
                let months: f64 = carrier
                    .values
                    .split(" + ")
                    .map(|value| value.parse::<f64>().unwrap())
                    .sum();
                assert!((months - carrier.result).abs() < 0.01, "{}", carrier.key);
            }
        }
        // The heating chain: the generator output is the sum of its terms (9.4/9.5) and the
        // input the sum of the carriers (9.1), each month.
        let heating = trace
            .children
            .iter()
            .find(|step| step.key == "installations.heating")
            .expect("heating");
        let chain = heating
            .children
            .iter()
            .find(|step| step.key == "installations.heating.chain")
            .expect("chain");
        assert_eq!(chain.children.len(), 12);
        for (month, row) in chain
            .children
            .iter()
            .zip(&performance.space_heating.monthly)
        {
            let output = &month.children[0];
            assert!(
                (children_sum(output) - row.generator_output_kwh).abs() < 1e-6,
                "month {}: Σ {} vs {}",
                row.month,
                children_sum(output),
                row.generator_output_kwh
            );
            let input = &month.children[1];
            assert!((children_sum(input) - input.result).abs() < 1e-9);
            // The need links to the zones' heat balance in the performance trace.
            assert!(output.children[0].uses.iter().all(|key| performance
                .trace
                .iter()
                .any(|step| key.starts_with(&step.key))));
        }
    }

    #[test]
    fn primary_energy_trace_matches_the_kernel_results() {
        let project: serde_json::Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-terraced-dwelling.json"
        ))
        .unwrap();
        let assessment = crate::project_performance::assess_project_performance(&project);
        let input = assessment.derived_input.as_ref().expect("derived input");
        let performance = assessment.performance.as_ref().expect("performance");
        let area = input.total_usable_floor_area_m2;
        let trace = primary_energy_trace(performance, area, true).expect("primary trace");
        let ep2 = &trace.children[0];
        let ep_tot = &ep2.children[0];
        // E_P;Tot: the carriers minus the export credit and the battery correction (5.10).
        let rebuilt: f64 = ep_tot
            .children
            .iter()
            .map(|step| {
                if step.key.starts_with("primary.del.") {
                    step.result
                } else {
                    -step.result
                }
            })
            .sum();
        assert!(
            (rebuilt - ep_tot.result).abs() < 1e-6,
            "{rebuilt} vs {}",
            ep_tot.result
        );
        for carrier in &ep_tot.children {
            if carrier.key.starts_with("primary.del.") {
                assert!((children_sum(carrier) - carrier.result).abs() < 1e-9);
            }
        }
        // EP2 = E_P;Tot / A_g rounded up to 0,01 (5.2).
        let expected = (ep_tot.result / area * 100.0 - 1e-9).ceil() / 100.0;
        assert!(
            (ep2.result - expected).abs() < 1e-9,
            "{} vs {expected}",
            ep2.result
        );
        // The label shown is the kernel's indicative class.
        let class = performance.indicative_label_class.expect("class");
        assert!(trace
            .children
            .iter()
            .any(|step| step.quantity == format!("Energielabel {class} (indicatief)")));
        // The links go to the installations steps.
        let installations = installations_trace(performance).expect("installations");
        let mut keys = std::collections::HashSet::new();
        fn collect(step: &TraceStep, keys: &mut std::collections::HashSet<String>) {
            keys.insert(step.key.clone());
            step.children.iter().for_each(|child| collect(child, keys));
        }
        collect(&installations, &mut keys);
        for carrier in ep_tot
            .children
            .iter()
            .filter(|step| step.key.starts_with("primary.del."))
        {
            for function in &carrier.children {
                assert!(
                    function.uses.iter().all(|key| keys.contains(key)),
                    "{:?}",
                    function.uses
                );
            }
        }
    }

    /// The chains of hot water, cooling, fans and lighting add up to the kernel's own
    /// monthly values, for the example dwelling and office.
    #[test]
    fn installation_chains_match_the_kernel_results() {
        for name in ["terraced-dwelling", "office"] {
            let text = std::fs::read_to_string(format!(
                "{}/../../training-data/nta8800-example-{name}.json",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap();
            let project: serde_json::Value = serde_json::from_str(&text).unwrap();
            let assessment = crate::project_performance::assess_project_performance(&project);
            let performance = assessment.performance.as_ref().expect("performance");
            if let Some(hot_water) = &performance.hot_water {
                let chain = hot_water_chain_trace(hot_water).expect("hot water chain");
                for (step, month) in chain.children.iter().zip(&hot_water.months) {
                    let output = &step.children[0];
                    let rebuilt = month.emission_input_kwh / month.distribution_efficiency
                        + month.storage_loss_kwh
                        + month.solar_backup_storage_loss_kwh
                        + month.conversion_loss_kwh
                        - month.solar_renewable_kwh;
                    assert!(
                        (rebuilt - output.result).abs() < 1e-6,
                        "{name} hot water {}: {rebuilt} vs {}",
                        month.month,
                        output.result
                    );
                    let emission = &output.children[0];
                    let rebuilt =
                        month.net_need_kwh / hot_water.emission_efficiency - month.recovered_kwh;
                    assert!(
                        (rebuilt - emission.result).abs() < 1e-6,
                        "{name} Q_W;em {}",
                        month.month
                    );
                    let input = &step.children[1];
                    assert!(
                        (input.result - month.carrier_input_kwh).abs() < 1e-6,
                        "{name} E_W;gen;in {}",
                        month.month
                    );
                }
            }
            if let Some(cooling) = &performance.cooling {
                let chain = cooling_chain_trace(cooling).expect("cooling chain");
                for (step, month) in chain.children.iter().zip(&cooling.months) {
                    let load = &step.children[0];
                    assert!(
                        (children_sum(load) - load.result).abs() < 1e-6,
                        "{name} cooling {}",
                        month.month
                    );
                }
            }
            for zone in fans_trace(performance) {
                assert!((children_sum(&zone) - zone.result).abs() < 1e-9);
            }
            for zone in lighting_trace(performance) {
                if !zone.children.is_empty() {
                    assert!(
                        (children_sum(&zone) - zone.result).abs() < 1e-6,
                        "{name} lighting {} vs {}",
                        children_sum(&zone),
                        zone.result
                    );
                }
            }
            // Every installations step still sums its carriers.
            let installations = installations_trace(performance).expect("installations");
            for function in &installations.children {
                let carriers: f64 = function
                    .children
                    .iter()
                    .filter(|child| child.quantity.starts_with("E_EPus;"))
                    .map(|child| child.result)
                    .sum();
                assert!((function.result - carriers).abs() < 1e-6);
            }
        }
    }
}
