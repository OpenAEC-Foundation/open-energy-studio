//! One entry point for the envelope element calculations of NTA 8800
//! §8.2 and annexes C, E–I and L: opaque constructions, tapered roofs,
//! windows and doors, forfait values for existing buildings, rooflights,
//! ventilation grilles, numerically determined U-values, forfait linear
//! thermal bridges (tables I.1/I.2) and the forfait supplement ΔU_for (8.3).
//!
//! The results are unverified until reference cases are reproduced.

use serde::{Deserialize, Serialize};

use crate::constructions::{
    forfait_bridge_supplement, rooflight_u, OpaqueConstruction, OpaqueResult, TaperedRoof,
    U_VENTILATION_GRILLE,
};
use crate::forfait_envelope::{
    forfait_door_u, forfait_glazed_door_u, forfait_panel_u, forfait_panel_u_in_edition,
    forfait_psi, forfait_window_u, ForfaitGlass, ForfaitOpaque, ForfaitOpaqueResult,
    PanelInsulation, PsiColumn, PSI_DEFAULT,
};
use crate::materials::round_half_up;
use crate::window_u::{round_transparent, FrameGroup, WindowInput, WindowMethod, WindowResult};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ElementKind {
    Opaque {
        construction: OpaqueConstruction,
    },
    TaperedRoof {
        roof: TaperedRoof,
    },
    Window {
        window: WindowInput,
    },
    ForfaitOpaque {
        element: ForfaitOpaque,
    },
    ForfaitWindow {
        glass: ForfaitGlass,
        frame: FrameGroup,
        exterior: bool,
    },
    ForfaitDoor {
        insulated: bool,
        exterior: bool,
        /// Glazed share of a door with transparent parts.
        #[serde(default, rename = "glassFraction")]
        glass_fraction: Option<f64>,
        #[serde(default)]
        glass: Option<ForfaitGlass>,
        #[serde(default)]
        frame: Option<FrameGroup>,
    },
    ForfaitPanel {
        insulation: PanelInsulation,
        cavity: bool,
        frame: FrameGroup,
        exterior: bool,
        /// Build year of the building (part), NTA 8800:2023 tables I.13/I.14
        /// only (p. 819–820).
        #[serde(default, rename = "buildYear", skip_serializing_if = "Option::is_none")]
        build_year: Option<i32>,
    },
    /// Note 2 of 8.2.2.1 (NEN-EN 1873).
    Rooflight {
        #[serde(rename = "uRcWPerM2K")]
        u_rc: f64,
        #[serde(rename = "areaWithUpstandM2")]
        area_with_upstand_m2: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 8.2.2.2.1: ventilation grilles and silencer boxes.
    VentilationGrille,
    /// 8.5: U_T = L_C/A_con from a numerical calculation (8.6), plus ΔU
    /// for air voids or an inverted roof when applicable.
    Numerical {
        #[serde(rename = "couplingWPerK")]
        coupling_w_per_k: f64,
        #[serde(rename = "constructionAreaM2")]
        construction_area_m2: f64,
        #[serde(default, rename = "deltaUWPerM2K")]
        delta_u: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvelopeElement {
    pub id: String,
    /// A_T (K.1.2), m²; required for rooflights and ΔU_for.
    #[serde(default)]
    pub projected_area_m2: Option<f64>,
    /// Counts for ΔU_for (opaque, not a floor over a crawlspace or on
    /// ground, not a panel).
    #[serde(default)]
    pub in_forfait_supplement: bool,
    pub element: ElementKind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForfaitBridge {
    pub id: String,
    /// Detail position of table I.1/I.2; `None` uses 0,5 W/(m·K).
    #[serde(default)]
    pub position: Option<u16>,
    #[serde(default)]
    pub variant: u8,
    pub column: PsiColumn,
    pub length_m: f64,
    /// The detail separates two zones or buildings: half the loss here.
    #[serde(default)]
    pub shared: bool,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvelopeInput {
    pub elements: Vec<EnvelopeElement>,
    #[serde(default)]
    pub forfait_bridges: Vec<ForfaitBridge>,
    /// 8.2: use ΔU_for for the whole building instead of ψ-values.
    #[serde(default)]
    pub forfait_supplement: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnvelopeIssue {
    pub code: &'static str,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementResult {
    pub id: String,
    pub route: &'static str,
    /// U used in H_D (8.1) and ΔU_for (8.3): rounded per 8.2.2.1 (opaque
    /// 2 decimals; transparent 1 decimal above 1,0, else 2), W/(m²·K).
    pub u_value: f64,
    /// Same as `u_value`; kept for clients of the presented value.
    pub u_rounded: f64,
    /// U before the 8.2.2.1 rounding, W/(m²·K).
    pub u_unrounded: f64,
    pub r_c: Option<f64>,
    pub r_c_rounded: Option<f64>,
    pub opaque: Option<OpaqueResult>,
    pub window: Option<WindowResult>,
    pub forfait: Option<ForfaitOpaqueResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeResult {
    pub id: String,
    pub psi_w_per_mk: f64,
    /// ℓ·ψ, halved for shared details, W/K.
    pub coefficient_w_per_k: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvelopeAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub issues: Vec<EnvelopeIssue>,
    pub elements: Vec<ElementResult>,
    pub bridges: Vec<BridgeResult>,
    pub delta_u_forfait: Option<f64>,
    pub interpretations: Vec<&'static str>,
    pub reference_verified: bool,
}

fn positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

fn validate(input: &EnvelopeInput) -> Vec<EnvelopeIssue> {
    let mut issues = Vec::new();
    let mut push = |code, path: String| issues.push(EnvelopeIssue { code, path });
    if input.elements.is_empty() {
        push("element_required", "elements".into());
    }
    for (i, e) in input.elements.iter().enumerate() {
        let path = format!("elements[{i}]");
        if e.id.trim().is_empty() {
            push("element_id_required", format!("{path}.id"));
        }
        if e.projected_area_m2.is_some_and(|a| !positive(a)) {
            push("projected_area_invalid", format!("{path}.projectedAreaM2"));
        }
        if e.in_forfait_supplement && e.projected_area_m2.is_none() {
            push("projected_area_required", format!("{path}.projectedAreaM2"));
        }
        let el = format!("{path}.element");
        match &e.element {
            ElementKind::Opaque { construction } => {
                for x in construction.validate(&format!("{el}.construction")) {
                    push(x.code, x.path);
                }
            }
            ElementKind::TaperedRoof { roof } => {
                for x in roof.validate(&format!("{el}.roof")) {
                    push(x.code, x.path);
                }
            }
            ElementKind::Window { window } => {
                for x in window.validate(&format!("{el}.window")) {
                    push(x.code, x.path);
                }
                if e.in_forfait_supplement {
                    push(
                        "transparent_element_not_in_supplement",
                        format!("{path}.inForfaitSupplement"),
                    );
                }
            }
            ElementKind::ForfaitOpaque { element } => {
                for x in element.validate(&format!("{el}.element")) {
                    push(x.code, x.path);
                }
            }
            ElementKind::ForfaitWindow { .. } => {
                if e.in_forfait_supplement {
                    push(
                        "transparent_element_not_in_supplement",
                        format!("{path}.inForfaitSupplement"),
                    );
                }
            }
            ElementKind::ForfaitDoor {
                glass_fraction,
                glass,
                frame,
                ..
            } => {
                if let Some(f) = glass_fraction {
                    if !(0.0..=1.0).contains(f) || glass.is_none() || frame.is_none() {
                        push("glazed_door_invalid", el.clone());
                    }
                }
                // 8.3: only opaque category a) elements (8.2.2.1).
                if e.in_forfait_supplement {
                    push(
                        "door_or_panel_not_in_supplement",
                        format!("{path}.inForfaitSupplement"),
                    );
                }
            }
            ElementKind::ForfaitPanel {
                insulation,
                cavity,
                frame,
                exterior,
                build_year,
            } => {
                if forfait_panel_u(*insulation, *cavity, *frame, *exterior).is_none() {
                    push("panel_thickness_outside_table", el.clone());
                }
                // Tables I.13/I.14 by build year exist in NTA 8800:2023 only;
                // there a panel without a known thickness needs the year.
                let tables = crate::norm_versions::profile().panel_build_year_tables;
                if build_year.is_some() && !tables {
                    push("route_not_in_edition", format!("{el}.buildYear"));
                }
                if tables
                    && build_year.is_none()
                    && !matches!(insulation, PanelInsulation::KnownThickness { .. })
                {
                    push("panel_build_year_required", format!("{el}.buildYear"));
                }
                if e.in_forfait_supplement {
                    push(
                        "door_or_panel_not_in_supplement",
                        format!("{path}.inForfaitSupplement"),
                    );
                }
            }
            ElementKind::Rooflight {
                u_rc,
                area_with_upstand_m2,
                source_reference,
            } => {
                // NTA 8800:2023 8.2.2.1 (p. 207) has no rooflight category and
                // no U_rc conversion (2024 p. 210).
                if !crate::norm_versions::profile().rooflight_route {
                    push("route_not_in_edition", el.clone());
                }
                if !positive(*u_rc) || !positive(*area_with_upstand_m2) {
                    push("rooflight_invalid", el.clone());
                }
                if e.projected_area_m2.is_none() {
                    push("projected_area_required", format!("{path}.projectedAreaM2"));
                }
                if source_reference.trim().is_empty() {
                    push("source_reference_required", format!("{el}.sourceReference"));
                }
                if e.in_forfait_supplement {
                    push(
                        "transparent_element_not_in_supplement",
                        format!("{path}.inForfaitSupplement"),
                    );
                }
            }
            ElementKind::VentilationGrille => {}
            ElementKind::Numerical {
                coupling_w_per_k,
                construction_area_m2,
                delta_u,
                source_reference,
            } => {
                if !positive(*coupling_w_per_k)
                    || !positive(*construction_area_m2)
                    || !(delta_u.is_finite() && *delta_u >= 0.0)
                {
                    push("numerical_coupling_invalid", el.clone());
                }
                if source_reference.trim().is_empty() {
                    push("source_reference_required", format!("{el}.sourceReference"));
                }
            }
        }
    }
    // 8.2.2.3.1: one choice between 8.14 and 8.15 for all windows.
    let method_of = |e: &EnvelopeElement| match &e.element {
        ElementKind::Window { window } => match window.method {
            WindowMethod::Detailed { .. } => Some(true),
            WindowMethod::Simplified { .. } => Some(false),
            _ => None,
        },
        _ => None,
    };
    let detailed = input.elements.iter().filter_map(method_of).any(|d| d);
    let simplified = input.elements.iter().filter_map(method_of).any(|d| !d);
    if detailed && simplified {
        push("window_formula_8_14_and_8_15_mixed", "elements".into());
    }
    for (i, b) in input.forfait_bridges.iter().enumerate() {
        let path = format!("forfaitBridges[{i}]");
        if !(b.length_m.is_finite() && b.length_m >= 0.0) {
            push("bridge_length_invalid", format!("{path}.lengthM"));
        }
        if b.description.trim().is_empty() {
            push("source_reference_required", format!("{path}.description"));
        }
        if let Some(position) = b.position {
            if forfait_psi(position, b.variant, b.column).is_none() {
                push("bridge_position_not_in_table", format!("{path}.position"));
            }
        }
    }
    if input.forfait_supplement && !input.forfait_bridges.is_empty() {
        // 8.2: forfait and non-forfait bridge methods may not be mixed.
        push(
            "forfait_supplement_mixed_with_bridges",
            "forfaitSupplement".into(),
        );
    }
    issues
}

fn element_result(e: &EnvelopeElement) -> ElementResult {
    let base = |route, u: f64, rounded: f64| ElementResult {
        id: e.id.clone(),
        route,
        u_value: rounded,
        u_rounded: rounded,
        u_unrounded: u,
        r_c: None,
        r_c_rounded: None,
        opaque: None,
        window: None,
        forfait: None,
    };
    match &e.element {
        ElementKind::Opaque { construction } => {
            let r = construction.calculate();
            ElementResult {
                r_c: Some(r.r_c),
                r_c_rounded: Some(r.r_c_rounded),
                opaque: Some(r),
                ..base("8.2.2.2", r.u_c, r.u_c_rounded)
            }
        }
        ElementKind::TaperedRoof { roof } => {
            let u = roof.u_t();
            base("C.4.4", u, round_half_up(u, 2))
        }
        ElementKind::Window { window } => {
            let r = window.calculate();
            ElementResult {
                window: Some(r),
                ..base("8.2.2.3", r.u_effective, r.u_rounded)
            }
        }
        ElementKind::ForfaitOpaque { element } => {
            let r = element.calculate();
            ElementResult {
                r_c: Some(r.r_c),
                r_c_rounded: Some(round_half_up(r.r_c, 2)),
                forfait: Some(r),
                ..base(r.route, r.u_c, r.u_c)
            }
        }
        ElementKind::ForfaitWindow {
            glass,
            frame,
            exterior,
        } => {
            let u = forfait_window_u(*glass, *frame, *exterior);
            base("I.2.2.1", u, u)
        }
        ElementKind::ForfaitDoor {
            insulated,
            exterior,
            glass_fraction,
            glass,
            frame,
        } => {
            let u = match (glass_fraction, glass, frame) {
                (Some(f), Some(g), Some(fr)) => {
                    forfait_glazed_door_u(*f, *g, *fr, *insulated, *exterior)
                }
                _ => forfait_door_u(*insulated, *exterior),
            };
            base("I.2.2.3", u, round_transparent(u))
        }
        ElementKind::ForfaitPanel {
            insulation,
            cavity,
            frame,
            exterior,
            build_year,
        } => {
            let u =
                forfait_panel_u_in_edition(*insulation, *cavity, *frame, *exterior, *build_year)
                    .unwrap_or(f64::NAN);
            base("I.2.2.4", u, round_transparent(u))
        }
        ElementKind::Rooflight {
            u_rc,
            area_with_upstand_m2,
            ..
        } => {
            let u = rooflight_u(
                *u_rc,
                *area_with_upstand_m2,
                e.projected_area_m2.unwrap_or(f64::NAN),
            );
            base("8.2.2.1 e", u, round_transparent(u))
        }
        ElementKind::VentilationGrille => {
            base("8.2.2.2.1", U_VENTILATION_GRILLE, U_VENTILATION_GRILLE)
        }
        ElementKind::Numerical {
            coupling_w_per_k,
            construction_area_m2,
            delta_u,
            ..
        } => {
            let u = coupling_w_per_k / construction_area_m2 + delta_u;
            base("8.5", u, round_half_up(u, 2))
        }
    }
}

pub fn assess_envelope(input: &EnvelopeInput) -> EnvelopeAssessment {
    let scope = "nta8800_chapter_8_2_envelope_elements_unverified";
    let mut interpretations: Vec<&'static str> = crate::constructions::INTERPRETATIONS.to_vec();
    interpretations.extend_from_slice(INTERPRETATIONS);
    let issues = validate(input);
    if !issues.is_empty() {
        return EnvelopeAssessment {
            status: "invalid",
            scope,
            issues,
            elements: Vec::new(),
            bridges: Vec::new(),
            delta_u_forfait: None,
            interpretations,
            reference_verified: false,
        };
    }
    let elements: Vec<ElementResult> = input.elements.iter().map(element_result).collect();
    let bridges = input
        .forfait_bridges
        .iter()
        .map(|b| {
            let psi = b
                .position
                .and_then(|p| forfait_psi(p, b.variant, b.column))
                .unwrap_or(PSI_DEFAULT);
            let share = if b.shared { 0.5 } else { 1.0 };
            BridgeResult {
                id: b.id.clone(),
                psi_w_per_mk: psi,
                coefficient_w_per_k: b.length_m * psi * share,
            }
        })
        .collect();
    let delta_u_forfait = input.forfait_supplement.then(|| {
        let parts: Vec<(f64, f64)> = input
            .elements
            .iter()
            .zip(&elements)
            .filter(|(e, _)| e.in_forfait_supplement)
            .map(|(e, r)| (e.projected_area_m2.unwrap_or(0.0), r.u_value))
            .collect();
        forfait_bridge_supplement(&parts)
    });
    EnvelopeAssessment {
        status: "calculated_unverified",
        scope,
        issues: Vec::new(),
        elements,
        bridges,
        delta_u_forfait,
        interpretations,
        reference_verified: false,
    }
}

pub const INTERPRETATIONS: &[&str] = &[
    "8.2.2.1: the U in H_D and in ΔU_for (8.3) is the rounded value; forfait doors and panels follow the transparent rounding",
    "8.3: only opaque category a) elements (opaque constructions, tapered roofs, forfait opaque values, numerical U, ventilation grilles of 8.2.2.2.1) may enter ΔU_for",
    "table E.1, E.14–E.17: densities between table rows use the nearest row (E.1) or the next higher density (masonry)",
    "8.25: U_p of the substitute panel uses R_T = R_p = d_p/λ_p without surface resistances, as written",
    "table I.6 caravans 1965–1983: 0,19 for façades; the 0,04 panel value is not applied automatically",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_assesses_all_element_kinds() {
        let input: EnvelopeInput = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-constructions-synthetic.json"
        ))
        .unwrap();
        let a = assess_envelope(&input);
        assert_eq!(a.status, "calculated_unverified", "{:?}", a.issues);
        assert_eq!(a.elements.len(), input.elements.len());
        assert!(a
            .elements
            .iter()
            .all(|e| e.u_value.is_finite() && e.u_value > 0.0));
        let grille = a.elements.iter().find(|e| e.route == "8.2.2.2.1").unwrap();
        assert_eq!(grille.u_value, 6.2);
        assert!(a
            .bridges
            .iter()
            .any(|b| (b.psi_w_per_mk - 0.5).abs() < 1e-12));
    }

    #[test]
    fn mixing_supplement_and_bridges_is_rejected() {
        let input = EnvelopeInput {
            elements: vec![EnvelopeElement {
                id: "g".into(),
                projected_area_m2: Some(1.0),
                in_forfait_supplement: true,
                element: ElementKind::VentilationGrille,
            }],
            forfait_bridges: vec![ForfaitBridge {
                id: "b".into(),
                position: Some(5),
                variant: 0,
                column: PsiColumn::A,
                length_m: 2.0,
                shared: false,
                description: "sill".into(),
            }],
            forfait_supplement: true,
        };
        let a = assess_envelope(&input);
        assert_eq!(a.status, "invalid");
        assert!(a
            .issues
            .iter()
            .any(|i| i.code == "forfait_supplement_mixed_with_bridges"));
        let ok = EnvelopeInput {
            forfait_bridges: vec![],
            ..input
        };
        let a = assess_envelope(&ok);
        // U = 6,2 → ΔU_for = max(0; 0,1 − 0,25·5,8) = 0.
        assert_eq!(a.delta_u_forfait, Some(0.0));
    }

    #[test]
    fn rounded_u_feeds_h_d_and_supplement_excludes_doors_and_panels() {
        use crate::window_u::{EdgePsi, GlazingU, Spacer};
        let window = |method: WindowMethod| ElementKind::Window {
            window: WindowInput {
                method,
                shutter: None,
            },
        };
        let simplified = || {
            window(WindowMethod::Simplified {
                glazing: GlazingU::Declared {
                    value: 1.2,
                    source_reference: "glass".into(),
                },
                frame_u_w_per_m2k: 1.6,
                frame_source_reference: "frame".into(),
                edge: EdgePsi::Table {
                    frame: FrameGroup::WoodOrPlastic,
                    spacer: Spacer::AluminiumOrSteel,
                    low_e: true,
                    spacer_paths: Vec::new(),
                    spacer_source_reference: None,
                },
            })
        };
        let element = |id: &str, supplement: bool, element: ElementKind| EnvelopeElement {
            id: id.into(),
            projected_area_m2: Some(2.0),
            in_forfait_supplement: supplement,
            element,
        };
        let input = EnvelopeInput {
            elements: vec![
                element("w", false, simplified()),
                element(
                    "n",
                    true,
                    ElementKind::Numerical {
                        coupling_w_per_k: 0.5678,
                        construction_area_m2: 2.0,
                        delta_u: 0.0,
                        source_reference: "2D".into(),
                    },
                ),
            ],
            forfait_bridges: Vec::new(),
            forfait_supplement: true,
        };
        let a = assess_envelope(&input);
        assert_eq!(a.status, "calculated_unverified", "{:?}", a.issues);
        let w = &a.elements[0];
        assert_eq!(w.u_value, round_transparent(w.u_unrounded));
        assert_eq!(w.u_value, w.u_rounded);
        let n = &a.elements[1];
        assert!((n.u_unrounded - 0.2839).abs() < 1e-12);
        assert_eq!(n.u_value, 0.28);
        // ΔU_for from the rounded U_C (8.3): 0,1 − 0,25·(0,28 − 0,4) = 0,13.
        assert!((a.delta_u_forfait.unwrap() - 0.13).abs() < 1e-12);

        // Doors, panels and transparent forfait elements may not enter 8.3.
        let mut bad = input.clone();
        bad.elements.push(element(
            "d",
            true,
            ElementKind::ForfaitDoor {
                insulated: true,
                exterior: true,
                glass_fraction: None,
                glass: None,
                frame: None,
            },
        ));
        bad.elements.push(element(
            "p",
            true,
            ElementKind::ForfaitPanel {
                insulation: PanelInsulation::AbsentOrUnknown,
                cavity: false,
                frame: FrameGroup::WoodOrPlastic,
                exterior: true,
                build_year: None,
            },
        ));
        bad.elements.push(element(
            "fw",
            true,
            ElementKind::ForfaitWindow {
                glass: ForfaitGlass::HrPlusPlus,
                frame: FrameGroup::WoodOrPlastic,
                exterior: true,
            },
        ));
        let codes: Vec<_> = assess_envelope(&bad)
            .issues
            .iter()
            .map(|i| i.code)
            .collect();
        assert_eq!(
            codes
                .iter()
                .filter(|c| **c == "door_or_panel_not_in_supplement")
                .count(),
            2
        );
        assert!(codes.contains(&"transparent_element_not_in_supplement"));

        // 8.2.2.3.1: 8.14 and 8.15 may not be mixed.
        let mut mixed = input;
        mixed.elements.push(element(
            "w2",
            false,
            window(WindowMethod::Detailed {
                glazing: Vec::new(),
                panels: vec![crate::window_u::PanelPart {
                    area_m2: 1.0,
                    u_value_w_per_m2k: 0.5,
                    perimeter_m: 4.0,
                    psi_w_per_mk: 0.0,
                    source_reference: "panel".into(),
                }],
                frame_area_m2: 0.3,
                frame_u_w_per_m2k: 1.6,
                frame_source_reference: "frame".into(),
                glazing_bars: None,
            }),
        ));
        assert!(assess_envelope(&mixed)
            .issues
            .iter()
            .any(|i| i.code == "window_formula_8_14_and_8_15_mixed"));
    }
}
