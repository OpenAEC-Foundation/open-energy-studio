//! Windows, doors, shutters and thermal bridges,
//! NTA 8800:2025+C1:2026 §8.2.2.3–8.2.4 (pp. 235–247), annex G (p. 818)
//! and annex L (pp. 873–875).
//!
//! - U_W/U_D detailed (8.14, 8.18, 8.19) and simplified (8.15–8.17), the
//!   frame-type table 8.3, default doors (8.20/8.21) and shutters
//!   (8.22/8.23, table 8.4).
//! - U_gl of single glazing (8.24) and multiple glazing from table G.1.
//! - ψ_gl from tables L.1/L.2 with the spacer criterion (L.1).
//! - Numerical post-processing: U_fr (8.25), ψ (8.26), ψ_gl (8.27), ψ_p
//!   (8.28), χ (8.29), and the relevance test for point bridges (table 8.5).
//! - Presentation rounding of transparent components (8.2.2.1).

use serde::{Deserialize, Serialize};

/// f_prac for glazing (8.14 note 3).
pub const F_PRAC_GLAZING: f64 = 1.0;
/// 8.22: weighted time fraction of closed shutters.
pub const F_SHUT_WITH: f64 = 0.5;
pub const U_SINGLE_GLAZING: f64 = 5.8;
/// 8.20 and 8.21.
pub const U_DOOR_DEFAULT: f64 = 3.4;
pub const U_DOOR_INSULATING: f64 = 2.0;

/// 8.2.2.1: transparent U-values above 1,0 to one decimal, otherwise two.
pub fn round_transparent(u: f64) -> f64 {
    let decimals = if u > 1.0 { 1 } else { 2 };
    crate::materials::round_half_up(u, decimals)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GlazingLayers {
    Double,
    Triple,
}

/// Normal emissivity class of the coated pane(s) (table G.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoatingEmissivity {
    Uncoated,
    AtMost0_20,
    AtMost0_15,
    AtMost0_10,
    AtMost0_05,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GasFill {
    Air,
    Argon,
    Krypton,
    Sf6,
    Xenon,
}

/// Table G.1 (gas concentration > 90 %). Gap widths in mm: 6, 8, 12, 16,
/// 20 for double and 6, 8, 12 for triple glazing; other widths return
/// `None`.
pub fn table_g1(
    layers: GlazingLayers,
    coating: CoatingEmissivity,
    gap_mm: u32,
    gas: GasFill,
) -> Option<f64> {
    const DOUBLE: [[[f64; 5]; 5]; 5] = [
        [
            [3.3, 3.0, 2.8, 3.0, 2.6],
            [3.1, 2.9, 2.7, 3.1, 2.6],
            [2.8, 2.7, 2.6, 3.1, 2.6],
            [2.7, 2.6, 2.6, 3.1, 2.6],
            [2.7, 2.6, 2.6, 3.1, 2.6],
        ],
        [
            [2.7, 2.3, 1.9, 2.3, 1.6],
            [2.4, 2.1, 1.7, 2.4, 1.6],
            [2.0, 1.8, 1.6, 2.4, 1.6],
            [1.8, 1.6, 1.6, 2.5, 1.6],
            [1.8, 1.7, 1.6, 2.5, 1.7],
        ],
        [
            [2.6, 2.3, 1.8, 2.2, 1.5],
            [2.3, 2.0, 1.6, 2.3, 1.4],
            [1.9, 1.6, 1.5, 2.3, 1.5],
            [1.7, 1.5, 1.5, 2.4, 1.5],
            [1.7, 1.5, 1.5, 2.4, 1.5],
        ],
        [
            [2.6, 2.2, 1.7, 2.1, 1.4],
            [2.2, 1.9, 1.4, 2.2, 1.3],
            [1.8, 1.5, 1.3, 2.3, 1.3],
            [1.6, 1.4, 1.3, 2.3, 1.4],
            [1.6, 1.4, 1.4, 2.3, 1.4],
        ],
        [
            [2.5, 2.1, 1.5, 2.0, 1.2],
            [2.1, 1.7, 1.3, 2.1, 1.1],
            [1.7, 1.3, 1.1, 2.1, 1.2],
            [1.4, 1.2, 1.2, 2.2, 1.2],
            [1.5, 1.2, 1.2, 2.2, 1.2],
        ],
    ];
    const TRIPLE: [[[f64; 5]; 3]; 5] = [
        [
            [2.3, 2.1, 1.8, 1.9, 1.7],
            [2.1, 1.9, 1.7, 1.9, 1.6],
            [1.9, 1.8, 1.6, 2.0, 1.6],
        ],
        [
            [1.8, 1.5, 1.1, 1.3, 0.9],
            [1.5, 1.3, 1.0, 1.3, 0.8],
            [1.2, 1.0, 0.8, 1.3, 0.8],
        ],
        [
            [1.7, 1.4, 1.1, 1.2, 0.9],
            [1.5, 1.2, 0.9, 1.2, 0.8],
            [1.2, 1.0, 0.7, 1.3, 0.7],
        ],
        [
            [1.7, 1.3, 1.0, 1.1, 0.8],
            [1.4, 1.1, 0.8, 1.1, 0.7],
            [1.1, 0.9, 0.6, 1.2, 0.6],
        ],
        [
            [1.6, 1.2, 0.9, 1.1, 0.7],
            [1.3, 1.0, 0.7, 1.1, 0.5],
            [1.0, 0.8, 0.5, 1.1, 0.5],
        ],
    ];
    let c = coating as usize;
    let g = gas as usize;
    match layers {
        GlazingLayers::Double => {
            let i = [6, 8, 12, 16, 20].iter().position(|w| *w == gap_mm)?;
            Some(DOUBLE[c][i][g])
        }
        GlazingLayers::Triple => {
            let i = [6, 8, 12].iter().position(|w| *w == gap_mm)?;
            Some(TRIPLE[c][i][g])
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum GlazingU {
    Single,
    TableG1 {
        layers: GlazingLayers,
        coating: CoatingEmissivity,
        #[serde(rename = "gapMm")]
        gap_mm: u32,
        gas: GasFill,
    },
    /// NEN-EN 673/674/675.
    Declared {
        value: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

impl GlazingU {
    pub fn value(&self) -> Option<f64> {
        match self {
            Self::Single => Some(U_SINGLE_GLAZING),
            Self::TableG1 {
                layers,
                coating,
                gap_mm,
                gas,
            } => table_g1(*layers, *coating, *gap_mm, *gas),
            Self::Declared { value, .. } => Some(*value),
        }
    }

    fn validate(&self, path: &str, issues: &mut Vec<WindowIssue>) {
        match self {
            Self::Single => {}
            Self::TableG1 { .. } => {
                if self.value().is_none() {
                    issues.push(issue("glazing_not_in_table_g1", path));
                }
            }
            Self::Declared {
                value,
                source_reference,
            } => {
                if !positive(*value) {
                    issues.push(issue("u_value_invalid", format!("{path}.value")));
                }
                if source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        format!("{path}.sourceReference"),
                    ));
                }
            }
        }
    }
}

/// Frame groups of tables L.1/L.2, I.8–I.12 and table 8.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameGroup {
    WoodOrPlastic,
    MetalWithThermalBreak,
    MetalWithoutThermalBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Spacer {
    AluminiumOrSteel,
    /// Meets Σ(d·λ) ≤ 0,007 W/K (L.1).
    ThermallyImproved,
}

/// Formula L.1.
pub fn is_thermally_improved_spacer(paths: &[(f64, f64)]) -> bool {
    paths.iter().map(|(d, lambda)| d * lambda).sum::<f64>() <= 0.007
}

/// Tables L.1/L.2 ψ_g in W/(m·K).
pub fn table_psi_glazing(frame: FrameGroup, spacer: Spacer, low_e: bool) -> f64 {
    let (plain, coated) = match (spacer, frame) {
        (Spacer::AluminiumOrSteel, FrameGroup::WoodOrPlastic) => (0.06, 0.08),
        (Spacer::AluminiumOrSteel, FrameGroup::MetalWithThermalBreak) => (0.08, 0.11),
        (Spacer::AluminiumOrSteel, FrameGroup::MetalWithoutThermalBreak) => (0.02, 0.05),
        (Spacer::ThermallyImproved, FrameGroup::WoodOrPlastic) => (0.05, 0.06),
        (Spacer::ThermallyImproved, FrameGroup::MetalWithThermalBreak) => (0.06, 0.08),
        (Spacer::ThermallyImproved, FrameGroup::MetalWithoutThermalBreak) => (0.01, 0.04),
    };
    if low_e {
        coated
    } else {
        plain
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum EdgePsi {
    /// Single glazing: ψ = 0 (annex L note 2).
    SingleGlazing,
    Table {
        frame: FrameGroup,
        spacer: Spacer,
        #[serde(rename = "lowE")]
        low_e: bool,
    },
    /// 8.27/8.28 or NEN-EN-ISO 10077-2.
    Declared {
        value: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

impl EdgePsi {
    pub fn value(&self) -> f64 {
        match self {
            Self::SingleGlazing => 0.0,
            Self::Table {
                frame,
                spacer,
                low_e,
            } => table_psi_glazing(*frame, *spacer, *low_e),
            Self::Declared { value, .. } => *value,
        }
    }

    fn validate(&self, path: &str, issues: &mut Vec<WindowIssue>) {
        if let Self::Declared {
            value,
            source_reference,
        } = self
        {
            if !(value.is_finite() && *value >= 0.0) {
                issues.push(issue("psi_invalid", format!("{path}.value")));
            }
            if source_reference.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{path}.sourceReference"),
                ));
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GlazedPart {
    /// A_gl (K.2.1), m².
    pub area_m2: f64,
    pub glazing: GlazingU,
    /// ℓ_gl (K.2.2), m.
    pub perimeter_m: f64,
    pub edge: EdgePsi,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PanelPart {
    /// A_p (K.2.1), m².
    pub area_m2: f64,
    /// U_p (8.2.2.3.5.2), W/(m²·K).
    pub u_value_w_per_m2k: f64,
    pub perimeter_m: f64,
    pub psi_w_per_mk: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GlazingBars {
    pub length_m: f64,
    pub psi_w_per_mk: f64,
    pub source_reference: String,
}

/// Table 8.3 frame types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameType {
    /// A: wood or plastic (U_fr 2,4).
    A,
    /// B: metal with a thermal break meeting annex C (U_fr 3,8).
    B,
    /// C: metal without sufficient thermal break (U_fr 7,0).
    C,
}

/// Table 8.3 with linear interpolation on U_gl (note 4).
pub fn table_8_3(u_gl: f64, frame: FrameType) -> f64 {
    const UGL: [f64; 17] = [
        5.8, 3.3, 3.2, 3.0, 2.8, 2.6, 2.4, 2.2, 2.0, 1.8, 1.6, 1.4, 1.2, 1.0, 0.9, 0.7, 0.5,
    ];
    const A: [f64; 17] = [
        5.2, 3.3, 3.2, 3.0, 2.9, 2.8, 2.6, 2.5, 2.3, 2.2, 2.0, 1.9, 1.8, 1.6, 1.5, 1.4, 1.3,
    ];
    const B: [f64; 17] = [
        5.4, 3.6, 3.6, 3.4, 3.3, 3.2, 3.1, 2.9, 2.8, 2.6, 2.5, 2.4, 2.2, 2.1, 2.0, 1.9, 1.7,
    ];
    const C: [f64; 17] = [
        6.2, 4.5, 4.4, 4.2, 4.1, 4.0, 3.9, 3.7, 3.6, 3.5, 3.3, 3.2, 3.0, 2.9, 2.8, 2.7, 2.5,
    ];
    let column = match frame {
        FrameType::A => &A,
        FrameType::B => &B,
        FrameType::C => &C,
    };
    if u_gl >= UGL[0] {
        return column[0];
    }
    if u_gl <= UGL[16] {
        return column[16];
    }
    for i in 0..16 {
        let (hi, lo) = (UGL[i], UGL[i + 1]);
        if u_gl <= hi && u_gl >= lo {
            let t = (u_gl - lo) / (hi - lo);
            return column[i + 1] + t * (column[i] - column[i + 1]);
        }
    }
    column[16]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DoorDefault {
    /// 8.20: door without glazing, including frame.
    Opaque,
    /// 8.21: wood/plastic door with ≥ 65 % continuous insulation R_c ≥ 0,4.
    ThermallyInsulating,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum WindowMethod {
    /// 8.14 / 8.18 / 8.19.
    Detailed {
        #[serde(default)]
        glazing: Vec<GlazedPart>,
        #[serde(default)]
        panels: Vec<PanelPart>,
        #[serde(rename = "frameAreaM2")]
        frame_area_m2: f64,
        #[serde(rename = "frameUWPerM2K")]
        frame_u_w_per_m2k: f64,
        #[serde(rename = "frameSourceReference")]
        frame_source_reference: String,
        #[serde(default, rename = "glazingBars")]
        glazing_bars: Option<GlazingBars>,
    },
    /// 8.15–8.17.
    Simplified {
        glazing: GlazingU,
        #[serde(rename = "frameUWPerM2K")]
        frame_u_w_per_m2k: f64,
        #[serde(rename = "frameSourceReference")]
        frame_source_reference: String,
        edge: EdgePsi,
    },
    /// Table 8.3.
    FrameTable { glazing: GlazingU, frame: FrameType },
    /// 8.20/8.21.
    DoorDefault { door: DoorDefault },
    /// Product value (e.g. NEN-EN-ISO 10077, 12631, 12428).
    Declared {
        value: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShutterResistance {
    /// Table 8.4.
    InternalMetallised,
    InternalPlain,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum ShutterDeltaR {
    Table {
        kind: ShutterResistance,
    },
    /// NEN-EN 13125 for shutters, roller shutters and external screens.
    Declared {
        #[serde(rename = "deltaRM2KPerW")]
        delta_r: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Shutter {
    pub delta_r: ShutterDeltaR,
    /// Evidence for one of the 8.2.2.3.4 situations (automatic night
    /// closing, or dwelling-operated shutters covering window and frame).
    pub conditions_evidence: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WindowInput {
    pub method: WindowMethod,
    #[serde(default)]
    pub shutter: Option<Shutter>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WindowIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> WindowIssue {
    WindowIssue {
        code,
        path: path.into(),
    }
}

fn positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowResult {
    /// U_W without shutter.
    pub u_w: f64,
    /// U_W+shut (8.23).
    pub u_w_shut: Option<f64>,
    /// Effective U (8.22), or U_W without shutter.
    pub u_effective: f64,
    /// Presented value per 8.2.2.1.
    pub u_rounded: f64,
}

impl WindowInput {
    pub fn validate(&self, path: &str) -> Vec<WindowIssue> {
        let mut issues = Vec::new();
        let m = format!("{path}.method");
        match &self.method {
            WindowMethod::Detailed {
                glazing,
                panels,
                frame_area_m2,
                frame_u_w_per_m2k,
                frame_source_reference,
                glazing_bars,
            } => {
                if glazing.is_empty() && panels.is_empty() {
                    issues.push(issue("glazing_or_panel_required", m.clone()));
                }
                if !(frame_area_m2.is_finite() && *frame_area_m2 >= 0.0)
                    || !positive(*frame_u_w_per_m2k)
                {
                    issues.push(issue("frame_invalid", m.clone()));
                }
                if frame_source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        format!("{m}.frameSourceReference"),
                    ));
                }
                for (i, g) in glazing.iter().enumerate() {
                    let p = format!("{m}.glazing[{i}]");
                    if !positive(g.area_m2) || !(g.perimeter_m.is_finite() && g.perimeter_m >= 0.0)
                    {
                        issues.push(issue("glazing_geometry_invalid", p.clone()));
                    }
                    g.glazing.validate(&format!("{p}.glazing"), &mut issues);
                    g.edge.validate(&format!("{p}.edge"), &mut issues);
                }
                for (i, panel) in panels.iter().enumerate() {
                    let p = format!("{m}.panels[{i}]");
                    if !positive(panel.area_m2)
                        || !positive(panel.u_value_w_per_m2k)
                        || !(panel.perimeter_m.is_finite() && panel.perimeter_m >= 0.0)
                        || !(panel.psi_w_per_mk.is_finite() && panel.psi_w_per_mk >= 0.0)
                    {
                        issues.push(issue("panel_invalid", p.clone()));
                    }
                    if panel.source_reference.trim().is_empty() {
                        issues.push(issue(
                            "source_reference_required",
                            format!("{p}.sourceReference"),
                        ));
                    }
                }
                if let Some(bars) = glazing_bars {
                    if !(bars.length_m.is_finite() && bars.length_m >= 0.0)
                        || !(bars.psi_w_per_mk.is_finite() && bars.psi_w_per_mk >= 0.0)
                    {
                        issues.push(issue("glazing_bars_invalid", format!("{m}.glazingBars")));
                    }
                    if bars.source_reference.trim().is_empty() {
                        issues.push(issue(
                            "source_reference_required",
                            format!("{m}.glazingBars.sourceReference"),
                        ));
                    }
                }
            }
            WindowMethod::Simplified {
                glazing,
                frame_u_w_per_m2k,
                frame_source_reference,
                edge,
            } => {
                glazing.validate(&format!("{m}.glazing"), &mut issues);
                edge.validate(&format!("{m}.edge"), &mut issues);
                if !positive(*frame_u_w_per_m2k) {
                    issues.push(issue("frame_invalid", format!("{m}.frameUWPerM2K")));
                }
                if frame_source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        format!("{m}.frameSourceReference"),
                    ));
                }
            }
            WindowMethod::FrameTable { glazing, .. } => {
                glazing.validate(&format!("{m}.glazing"), &mut issues)
            }
            WindowMethod::DoorDefault { .. } => {}
            WindowMethod::Declared {
                value,
                source_reference,
            } => {
                if !positive(*value) {
                    issues.push(issue("u_value_invalid", format!("{m}.value")));
                }
                if source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        format!("{m}.sourceReference"),
                    ));
                }
            }
        }
        if let Some(shutter) = &self.shutter {
            if shutter.conditions_evidence.trim().is_empty() {
                issues.push(issue(
                    "source_reference_required",
                    format!("{path}.shutter.conditionsEvidence"),
                ));
            }
            if let ShutterDeltaR::Declared {
                delta_r,
                source_reference,
            } = &shutter.delta_r
            {
                if !(delta_r.is_finite() && *delta_r >= 0.0) {
                    issues.push(issue(
                        "shutter_resistance_invalid",
                        format!("{path}.shutter.deltaR"),
                    ));
                }
                if source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        format!("{path}.shutter.deltaR.sourceReference"),
                    ));
                }
            }
        }
        issues
    }

    /// Call after `validate`.
    pub fn calculate(&self) -> WindowResult {
        let u_w = match &self.method {
            WindowMethod::Detailed {
                glazing,
                panels,
                frame_area_m2,
                frame_u_w_per_m2k,
                glazing_bars,
                ..
            } => {
                let mut heat = frame_area_m2 * frame_u_w_per_m2k;
                let mut area = *frame_area_m2;
                for g in glazing {
                    heat += g.area_m2 * g.glazing.value().unwrap_or(f64::NAN) / F_PRAC_GLAZING
                        + g.perimeter_m * g.edge.value();
                    area += g.area_m2;
                }
                for p in panels {
                    heat += p.area_m2 * p.u_value_w_per_m2k + p.perimeter_m * p.psi_w_per_mk;
                    area += p.area_m2;
                }
                if let Some(bars) = glazing_bars {
                    heat += bars.length_m * bars.psi_w_per_mk;
                }
                heat / area
            }
            WindowMethod::Simplified {
                glazing,
                frame_u_w_per_m2k,
                edge,
                ..
            } => {
                let u_gl = glazing.value().unwrap_or(f64::NAN) / F_PRAC_GLAZING;
                let psi = edge.value();
                let u1 = 0.7 * u_gl + 0.3 * frame_u_w_per_m2k + 2.5 * psi;
                let u2 = 0.8 * u_gl + 0.2 * frame_u_w_per_m2k + 2.5 * psi;
                u1.max(u2)
            }
            WindowMethod::FrameTable { glazing, frame } => {
                table_8_3(glazing.value().unwrap_or(f64::NAN), *frame)
            }
            WindowMethod::DoorDefault { door } => match door {
                DoorDefault::Opaque => U_DOOR_DEFAULT,
                DoorDefault::ThermallyInsulating => U_DOOR_INSULATING,
            },
            WindowMethod::Declared { value, .. } => *value,
        };
        let u_w_shut = self.shutter.as_ref().map(|shutter| {
            let delta_r = match &shutter.delta_r {
                ShutterDeltaR::Table {
                    kind: ShutterResistance::InternalMetallised,
                } => 0.15,
                ShutterDeltaR::Table {
                    kind: ShutterResistance::InternalPlain,
                } => 0.05,
                ShutterDeltaR::Declared { delta_r, .. } => *delta_r,
            };
            1.0 / (1.0 / u_w + delta_r)
        });
        let u_effective = match u_w_shut {
            Some(shut) => (1.0 - F_SHUT_WITH) * u_w + F_SHUT_WITH * shut,
            None => u_w,
        };
        WindowResult {
            u_w,
            u_w_shut,
            u_effective,
            u_rounded: round_transparent(u_effective),
        }
    }
}

/// 8.25: U_fr from the 2D coupling coefficient of the frame with a
/// λ = 0,035 panel of thickness d_p and visible length b_p.
pub fn frame_u_from_l2d(
    l2d: f64,
    panel_thickness_m: f64,
    panel_length_m: f64,
    frame_width_m: f64,
) -> f64 {
    // U_p from 8.6 with R_T = R_p = d_p/λ_p (λ_p = 0,035).
    let u_p = 0.035 / panel_thickness_m;
    (l2d - u_p * panel_length_m) / frame_width_m
}

/// 8.26: ψ = L_2D − Σ ℓ_i·U_T;i.
pub fn linear_psi(l2d: f64, flanking: &[(f64, f64)]) -> f64 {
    l2d - flanking.iter().map(|(l, u)| l * u).sum::<f64>()
}

/// 8.27/8.28: ψ_gl (or ψ_p) = L_2D − U_fr·b_fr − U_gl·b_gl.
pub fn edge_psi(
    l2d: f64,
    u_frame: f64,
    frame_width_m: f64,
    u_infill: f64,
    infill_length_m: f64,
) -> f64 {
    l2d - u_frame * frame_width_m - u_infill * infill_length_m
}

/// 8.29: χ = L_3D − Σ A_i·U_T;i − Σ ℓ_j·ψ_j.
pub fn point_chi(l3d: f64, areas: &[(f64, f64)], lines: &[(f64, f64)]) -> f64 {
    l3d - areas.iter().map(|(a, u)| a * u).sum::<f64>()
        - lines.iter().map(|(l, p)| l * p).sum::<f64>()
}

/// Table 8.5 materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PointBridgeMaterial {
    Concrete,
    StainlessSteel,
    Steel,
    Aluminium,
}

/// 8.2.4.1: a point bridge counts when it occurs at least once per 20 m²,
/// the construction has R_c ≥ 3,5 and the cross-section exceeds table 8.5.
pub fn point_bridge_counts(
    material: PointBridgeMaterial,
    cross_section_m2: f64,
    occurrences_per_m2: f64,
    construction_rc: f64,
) -> bool {
    let minimum = match material {
        PointBridgeMaterial::Concrete => 0.040,
        PointBridgeMaterial::StainlessSteel => 0.006,
        PointBridgeMaterial::Steel => 0.002,
        PointBridgeMaterial::Aluminium => 0.0006,
    };
    occurrences_per_m2 >= 1.0 / 20.0 && construction_rc >= 3.5 && cross_section_m2 > minimum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hr_plus_plus() -> GlazingU {
        GlazingU::TableG1 {
            layers: GlazingLayers::Double,
            coating: CoatingEmissivity::AtMost0_05,
            gap_mm: 16,
            gas: GasFill::Argon,
        }
    }

    #[test]
    fn table_g1_and_rounding() {
        assert_eq!(hr_plus_plus().value(), Some(1.2));
        assert_eq!(
            table_g1(
                GlazingLayers::Triple,
                CoatingEmissivity::AtMost0_10,
                12,
                GasFill::Krypton
            ),
            Some(0.6)
        );
        assert_eq!(
            table_g1(
                GlazingLayers::Triple,
                CoatingEmissivity::AtMost0_10,
                16,
                GasFill::Air
            ),
            None
        );
        assert_eq!(round_transparent(1.349), 1.3);
        assert_eq!(round_transparent(0.876), 0.88);
    }

    #[test]
    fn detailed_window_follows_8_14() {
        let window = WindowInput {
            method: WindowMethod::Detailed {
                glazing: vec![GlazedPart {
                    area_m2: 1.2,
                    glazing: hr_plus_plus(),
                    perimeter_m: 4.6,
                    edge: EdgePsi::Table {
                        frame: FrameGroup::WoodOrPlastic,
                        spacer: Spacer::ThermallyImproved,
                        low_e: true,
                    },
                }],
                panels: vec![],
                frame_area_m2: 0.5,
                frame_u_w_per_m2k: 1.4,
                frame_source_reference: "10077-2".into(),
                glazing_bars: None,
            },
            shutter: None,
        };
        assert!(window.validate("w").is_empty());
        let r = window.calculate();
        let expected = (1.2 * 1.2 + 0.5 * 1.4 + 4.6 * 0.06) / 1.7;
        assert!((r.u_w - expected).abs() < 1e-12);
        assert_eq!(r.u_rounded, round_transparent(expected));
    }

    #[test]
    fn simplified_table_door_and_shutter() {
        let simplified = WindowInput {
            method: WindowMethod::Simplified {
                glazing: hr_plus_plus(),
                frame_u_w_per_m2k: 2.4,
                frame_source_reference: "table".into(),
                edge: EdgePsi::Table {
                    frame: FrameGroup::WoodOrPlastic,
                    spacer: Spacer::AluminiumOrSteel,
                    low_e: true,
                },
            },
            shutter: None,
        };
        // U1 = 0,84 + 0,72 + 0,2 = 1,76; U2 = 0,96 + 0,48 + 0,2 = 1,64.
        assert!((simplified.calculate().u_w - 1.76).abs() < 1e-12);
        assert!((table_8_3(1.2, FrameType::A) - 1.8).abs() < 1e-12);
        assert!((table_8_3(1.1, FrameType::B) - 2.15).abs() < 1e-12);
        let door = WindowInput {
            method: WindowMethod::DoorDefault {
                door: DoorDefault::Opaque,
            },
            shutter: Some(Shutter {
                delta_r: ShutterDeltaR::Table {
                    kind: ShutterResistance::InternalMetallised,
                },
                conditions_evidence: "BMS night closing".into(),
            }),
        };
        let r = door.calculate();
        let shut = 1.0 / (1.0 / 3.4 + 0.15);
        assert!((r.u_w_shut.unwrap() - shut).abs() < 1e-12);
        assert!((r.u_effective - (0.5 * 3.4 + 0.5 * shut)).abs() < 1e-12);
    }

    #[test]
    fn bridge_post_processing() {
        assert!((linear_psi(1.0, &[(1.0, 0.25), (1.2, 0.3)]) - 0.39).abs() < 1e-12);
        assert!((edge_psi(0.9, 1.5, 0.1, 1.1, 0.19) - (0.9 - 0.15 - 0.209)).abs() < 1e-12);
        assert!((point_chi(0.5, &[(1.0, 0.2)], &[(1.0, 0.1)]) - 0.2).abs() < 1e-12);
        let u_p = 0.035 / 0.024;
        assert!((frame_u_from_l2d(0.6, 0.024, 0.19, 0.1) - (0.6 - u_p * 0.19) / 0.1).abs() < 1e-12);
        assert!(is_thermally_improved_spacer(&[
            (0.0002, 17.0),
            (0.002, 0.2)
        ]));
        assert!(!is_thermally_improved_spacer(&[(0.0004, 160.0)]));
        assert!(point_bridge_counts(
            PointBridgeMaterial::Steel,
            0.003,
            0.1,
            4.7
        ));
        assert!(!point_bridge_counts(
            PointBridgeMaterial::Steel,
            0.003,
            0.01,
            4.7
        ));
        assert!(!point_bridge_counts(
            PointBridgeMaterial::Concrete,
            0.03,
            0.1,
            4.7
        ));
    }

    #[test]
    fn validation_reports_missing_sources() {
        let window = WindowInput {
            method: WindowMethod::FrameTable {
                glazing: GlazingU::TableG1 {
                    layers: GlazingLayers::Double,
                    coating: CoatingEmissivity::Uncoated,
                    gap_mm: 10,
                    gas: GasFill::Air,
                },
                frame: FrameType::A,
            },
            shutter: Some(Shutter {
                delta_r: ShutterDeltaR::Declared {
                    delta_r: 0.2,
                    source_reference: String::new(),
                },
                conditions_evidence: String::new(),
            }),
        };
        let codes: Vec<_> = window.validate("w").into_iter().map(|i| i.code).collect();
        assert!(codes.contains(&"glazing_not_in_table_g1"));
        assert_eq!(
            codes
                .iter()
                .filter(|c| **c == "source_reference_required")
                .count(),
            2
        );
    }
}
