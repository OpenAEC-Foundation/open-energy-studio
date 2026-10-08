//! Thermal resistance and transmittance of opaque constructions,
//! NTA 8800:2025+C1:2026 §8.2.2.2 (pp. 228–235), annex C (pp. 773–788) and
//! annex F (pp. 814–817).
//!
//! - Surface resistances R_si/R_se: table C.2.
//! - Layer resistances: materials via annex E ([`crate::materials`]), air
//!   layers via C.3 (tables C.3/C.4, C.12), narrow and tubular cavities via
//!   tables F.1–F.3, reflective foil systems (table E.13), attics (table
//!   C.5) and ventilated unheated spaces (C.15).
//! - R_T of homogeneous constructions (C.3) and of composite constructions
//!   (C.4–C.7, table C.1).
//! - Corrections ΔU = ΔU_a + ΔU_fa + ΔU_r (8.8–8.13, table 8.2) with the
//!   3 % rule; U_C = U_T/f_prac + ΔU (8.4, 8.6), R_C (C.1/C.2), R_eq
//!   (C.9/C.10).
//! - Tapered roof insulation (C.16–C.22) and the forfait linear-bridge
//!   supplement ΔU_for (8.3).
//!
//! Interpretation choices are listed in [`INTERPRETATIONS`].

use serde::{Deserialize, Serialize};

use crate::materials::{
    round_half_up, round_resistance_down, Ageing, Conductivity, InsulationMoisture,
    ReflectiveFoilSystem, TemperatureConversion,
};

/// Table C.2 R_se, (m²·K)/W.
pub const R_SE: f64 = 0.04;
/// R_si for downward heat flow (table C.2), m²K/W.
pub const R_SI_DOWNWARD: f64 = 0.17;
/// Table C.4: R_se for upward heat flow over a horizontal cavity with a
/// reflective layer (value in brackets).
pub const R_SE_UPWARD_REFLECTIVE: f64 = 0.05;
/// 8.4: practice factor for opaque parts.
pub const F_PRAC_OPAQUE: f64 = 1.0;
/// 8.2.2.2.1: ventilation grilles and silencer boxes.
pub const U_VENTILATION_GRILLE: f64 = 6.2;
/// 8.13: mean daily precipitation October–April, mm/day.
pub const PRECIPITATION_MM_PER_DAY: f64 = 2.105;

pub const INTERPRETATIONS: &[&str] = &[
    "C.12: with a known opening area A_V the weakly ventilated cavity interpolates between R_cav;nv and the R_cav;sv implied by the table value at A_V = 1 000 mm² (2·R_zv − R_nv)",
    "table C.4 downward heat flow: thicknesses between rows take the next lower tabulated thickness",
    "tables F.1–F.3 are interpolated bilinearly; values outside the table axes are clamped to the edge rows",
    "8.2.2.2.2: the 3 % rule compares the total ΔU with U_T",
    "8.9/8.11/8.13 in a composite construction: R_1 and R_T (C.3, thermal bridges neglected) come from the insulation section, by default the section with the highest C.3 R_T",
    "table C.4 footnote b: a reflective layer facing upward earns no bracket value unless the cavity is hermetically sealed; footnote b is applied to vertical cavities only when the input marks the layer as facing up",
    "table C.4: R_se 0,05 (bracket value) applies to upward heat flow when a horizontal cavity with an effective reflective layer is present; R_C subtracts the same R_se",
    "table F.1 (U ≤ 1,0) and tables F.2/F.3 (U > 1,0) are checked against the U_C of the whole construction",
    "tables C.3/C.4 footnotes c and e: an unventilated cavity thinner than 20 mm without an effective reflective layer follows D.2 of NEN-EN-ISO 6946:2017 with ε1 = ε2 = 0,9 and h_r0 at 10 °C (h_a = max(table value, λ_air/d), λ_air 0,025 W/(m·K)), rounded to 2 decimals like the table values; the same D.2 parameters reproduce every value of tables C.3 and C.4",
];

/// Direction of the heat flow through the element (table C.2 note 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatFlow {
    Upward,
    Horizontal,
    Downward,
}

impl HeatFlow {
    /// Table C.2 R_si.
    pub fn r_si(self) -> f64 {
        match self {
            Self::Upward => 0.10,
            Self::Horizontal => 0.13,
            Self::Downward => 0.17,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CavityVentilation {
    /// Openings < 500 mm²/m (or /m²).
    Unventilated,
    /// 500 ≤ A_V < 1 500; `None` uses the table value (A_V = 1 000).
    Weakly {
        #[serde(default, rename = "openingMm2")]
        opening_mm2: Option<f64>,
    },
    /// A_V ≥ 1 500: the cavity and everything outside it are disregarded.
    Strongly,
}

/// Table C.5 attic roofs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AtticRoof {
    TilesWithoutUnderlay,
    TilesOrSlatesWithUnderlay,
    TilesWithUnderlayAndReflectiveFoil,
    BoardingAndFelt,
}

impl AtticRoof {
    pub fn resistance(self) -> f64 {
        match self {
            Self::TilesWithoutUnderlay => 0.06,
            Self::TilesOrSlatesWithUnderlay => 0.2,
            Self::TilesWithUnderlayAndReflectiveFoil | Self::BoardingAndFelt => 0.3,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvelopePart {
    pub area_m2: f64,
    /// `None` uses the C.15 default of 2 W/(m²·K).
    #[serde(default)]
    pub u_value_w_per_m2k: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TubeOrientation {
    /// Table F.2.
    Horizontal,
    /// Table F.3.
    Vertical,
}

/// One layer from inside to outside.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Layer {
    Material {
        #[serde(rename = "thicknessM")]
        thickness_m: f64,
        conductivity: Conductivity,
    },
    /// R_calc of a product (E.4 already applied), e.g. from a DoP.
    Resistance {
        #[serde(rename = "resistanceM2KPerW")]
        resistance: f64,
        /// Needed only inside composite constructions.
        #[serde(default, rename = "thicknessM")]
        thickness_m: Option<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// E.4: R_calc = R_D/(F_T·F_M·F_A·F_conv), rounded down (E.2.1.1).
    DeclaredResistance {
        #[serde(rename = "resistanceDeclared")]
        resistance_declared: f64,
        moisture: InsulationMoisture,
        ageing: Ageing,
        #[serde(default)]
        temperature: Option<TemperatureConversion>,
        #[serde(default, rename = "convectionFactor")]
        convection_factor: Option<f64>,
        /// Needed only inside composite constructions.
        #[serde(default, rename = "thicknessM")]
        thickness_m: Option<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    ReflectiveFoil {
        system: ReflectiveFoilSystem,
    },
    /// C.3: air layer of at most 300 mm. Below 20 mm only unventilated
    /// without an effective reflective layer (footnotes c and e of tables
    /// C.3/C.4), or strongly ventilated.
    AirCavity {
        #[serde(rename = "thicknessMm")]
        thickness_mm: f64,
        ventilation: CavityVentilation,
        /// One bounding surface with ε ≤ 0,1 (values in brackets of tables
        /// C.3/C.4).
        #[serde(default, rename = "reflectiveSurface")]
        reflective_surface: bool,
        /// The reflective layer faces upward (bottom of the cavity); table
        /// C.4 footnote b grants no bracket value then.
        #[serde(default, rename = "reflectiveFacingUp")]
        reflective_facing_up: bool,
        /// Hermetically sealed cavity: footnote b allows the bracket value
        /// for an upward-facing reflective layer.
        #[serde(default, rename = "hermeticallySealed")]
        hermetically_sealed: bool,
    },
    /// Table F.1: unventilated narrow air layer or tube in a component with
    /// U ≤ 1,0 W/(m²·K).
    NarrowCavity {
        #[serde(rename = "thicknessMm")]
        thickness_mm: f64,
        #[serde(rename = "widthMm")]
        width_mm: f64,
    },
    /// Tables F.2/F.3: tubular cavity in a component with U > 1,0.
    TubularCavity {
        #[serde(rename = "thicknessMm")]
        thickness_mm: f64,
        #[serde(rename = "widthMm")]
        width_mm: f64,
        orientation: TubeOrientation,
    },
    /// Table C.5: naturally ventilated attic including the pitched roof;
    /// must be the outermost layer.
    Attic {
        roof: AtticRoof,
    },
    /// C.15: ventilated unheated space as outermost layer.
    VentilatedUnheatedSpace {
        /// Σ A_i between the heated and the unheated space, m².
        #[serde(rename = "separationAreaM2")]
        separation_area_m2: f64,
        /// Elements between the unheated space and outdoor air.
        #[serde(rename = "envelopeParts")]
        envelope_parts: Vec<EnvelopePart>,
        /// n in 1/h; `None` uses 0,3.
        #[serde(default, rename = "airChangeRate")]
        air_change_rate: Option<f64>,
        #[serde(rename = "volumeM3")]
        volume_m3: f64,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConstructionIssue {
    pub code: &'static str,
    pub path: String,
}

fn issue(code: &'static str, path: impl Into<String>) -> ConstructionIssue {
    ConstructionIssue {
        code,
        path: path.into(),
    }
}

fn positive(v: f64) -> bool {
    v.is_finite() && v > 0.0
}

/// Linear interpolation on a monotonic axis, clamped at both ends.
fn interpolate(axis: &[f64], values: &[f64], x: f64) -> f64 {
    let mut points: Vec<(f64, f64)> = axis.iter().copied().zip(values.iter().copied()).collect();
    points.sort_by(|a, b| a.0.total_cmp(&b.0));
    if x <= points[0].0 {
        return points[0].1;
    }
    for pair in points.windows(2) {
        let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
        if x <= x1 {
            return y0 + (x - x0) / (x1 - x0) * (y1 - y0);
        }
    }
    points[points.len() - 1].1
}

fn bilinear(rows: &[f64], cols: &[f64], table: &[&[f64]], row: f64, col: f64) -> f64 {
    let per_row: Vec<f64> = table.iter().map(|r| interpolate(cols, r, col)).collect();
    interpolate(rows, &per_row, row)
}

/// Table F.1 R in (m²·K)/W for narrow unventilated cavities.
pub fn narrow_cavity_resistance(thickness_mm: f64, width_mm: f64) -> f64 {
    const D: [f64; 6] = [2.0, 5.0, 7.0, 10.0, 15.0, 25.0];
    const RATIO: [f64; 8] = [10.0, 5.0, 3.0, 2.0, 1.0, 0.5, 0.3, 0.1];
    const T: [[f64; 8]; 6] = [
        [0.07, 0.07, 0.07, 0.07, 0.06, 0.06, 0.06, 0.06],
        [0.14, 0.14, 0.13, 0.13, 0.13, 0.12, 0.12, 0.11],
        [0.17, 0.17, 0.17, 0.16, 0.15, 0.14, 0.14, 0.13],
        [0.21, 0.21, 0.20, 0.20, 0.18, 0.17, 0.16, 0.15],
        [0.26, 0.25, 0.24, 0.24, 0.22, 0.20, 0.19, 0.17],
        [0.29, 0.28, 0.27, 0.26, 0.24, 0.22, 0.20, 0.18],
    ];
    let rows: Vec<&[f64]> = T.iter().map(|r| r.as_slice()).collect();
    bilinear(&D, &RATIO, &rows, thickness_mm, thickness_mm / width_mm)
}

/// Tables F.2/F.3 λ_equi in W/(m·K) of tubular cavities; the factor 1,2
/// (upward) or 0,8 (downward) applies to vertical heat flow with d > 10 mm.
pub fn tubular_cavity_lambda(
    thickness_mm: f64,
    width_mm: f64,
    orientation: TubeOrientation,
    heat_flow: HeatFlow,
) -> f64 {
    const AXIS: [f64; 8] = [5.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 80.0];
    // Rows b, columns d.
    const F2: [[f64; 8]; 8] = [
        [0.042, 0.055, 0.085, 0.124, 0.163, 0.202, 0.2242, 0.320],
        [0.044, 0.066, 0.100, 0.126, 0.151, 0.174, 0.197, 0.243],
        [0.046, 0.075, 0.133, 0.181, 0.217, 0.248, 0.277, 0.331],
        [0.047, 0.078, 0.138, 0.192, 0.242, 0.290, 0.336, 0.427],
        [0.047, 0.079, 0.142, 0.197, 0.249, 0.298, 0.346, 0.437],
        [0.047, 0.079, 0.144, 0.202, 0.255, 0.305, 0.354, 0.447],
        [0.047, 0.078, 0.146, 0.205, 0.260, 0.312, 0.361, 0.455],
        [0.048, 0.076, 0.147, 0.210, 0.267, 0.321, 0.372, 0.470],
    ];
    const F3: [[f64; 8]; 8] = [
        [0.042, 0.055, 0.085, 0.124, 0.163, 0.202, 0.2242, 0.320],
        [0.044, 0.059, 0.090, 0.130, 0.169, 0.208, 0.247, 0.326],
        [0.046, 0.063, 0.098, 0.139, 0.180, 0.219, 0.259, 0.337],
        [0.047, 0.066, 0.104, 0.147, 0.189, 0.229, 0.269, 0.348],
        [0.047, 0.067, 0.107, 0.153, 0.196, 0.238, 0.278, 0.358],
        [0.047, 0.068, 0.110, 0.157, 0.202, 0.245, 0.286, 0.368],
        [0.047, 0.068, 0.112, 0.161, 0.207, 0.251, 0.293, 0.376],
        [0.048, 0.069, 0.114, 0.166, 0.214, 0.260, 0.305, 0.391],
    ];
    let table = match orientation {
        TubeOrientation::Horizontal => &F2,
        TubeOrientation::Vertical => &F3,
    };
    let rows: Vec<&[f64]> = table.iter().map(|r| r.as_slice()).collect();
    let lambda = bilinear(&AXIS, &AXIS, &rows, width_mm, thickness_mm);
    let factor = match heat_flow {
        HeatFlow::Upward if thickness_mm > 10.0 => 1.2,
        HeatFlow::Downward if thickness_mm > 10.0 => 0.8,
        _ => 1.0,
    };
    lambda * factor
}

/// Tables C.3/C.4: (R_cav;nv, R_cav;zv) for an unventilated / weakly
/// ventilated cavity of at least 20 mm.
pub fn cavity_table(thickness_mm: f64, heat_flow: HeatFlow, reflective: bool) -> (f64, f64) {
    match (heat_flow, reflective) {
        (HeatFlow::Horizontal, false) => (0.18, 0.15),
        (HeatFlow::Horizontal, true) => (0.57, 0.40),
        (HeatFlow::Upward, false) => (0.16, 0.14),
        (HeatFlow::Upward, true) => (0.41, 0.31),
        (HeatFlow::Downward, reflective) => {
            const ROWS: [(f64, f64, f64, f64, f64); 5] = [
                (20.0, 0.18, 0.15, 0.57, 0.40),
                (25.0, 0.19, 0.15, 0.66, 0.44),
                (50.0, 0.21, 0.16, 0.99, 0.61),
                (100.0, 0.22, 0.17, 1.19, 0.71),
                (300.0, 0.23, 0.17, 1.40, 0.81),
            ];
            let row = ROWS
                .iter()
                .rev()
                .find(|r| r.0 <= thickness_mm + 1e-9)
                .unwrap_or(&ROWS[0]);
            if reflective {
                (row.3, row.4)
            } else {
                (row.1, row.2)
            }
        }
    }
}

/// Stefan–Boltzmann constant, W/(m²·K⁴).
const STEFAN_BOLTZMANN: f64 = 5.67e-8;
/// Thermal conductivity of still air behind the conduction limit of the
/// D.2 convection coefficient, W/(m·K).
const LAMBDA_AIR: f64 = 0.025;

/// R of an unventilated air layer by D.2 of NEN-EN-ISO 6946:2017 with the
/// parameters of footnote e of tables C.3/C.4 (2025+C1 p. 782 and 784):
/// ε1 = 0,9, ε2 = `emissivity_2`, h_r0 at 10 °C. Unrounded.
fn iso6946_d2_resistance(thickness_m: f64, heat_flow: HeatFlow, emissivity_2: f64) -> f64 {
    let conduction = LAMBDA_AIR / thickness_m;
    let h_a = match heat_flow {
        HeatFlow::Horizontal => conduction.max(1.25),
        HeatFlow::Upward => conduction.max(1.95),
        HeatFlow::Downward => conduction.max(0.12 * thickness_m.powf(-0.44)),
    };
    let e = 1.0 / (1.0 / 0.9 + 1.0 / emissivity_2 - 1.0);
    let h_r0 = 4.0 * STEFAN_BOLTZMANN * (273.15f64 + 10.0).powi(3);
    1.0 / (h_a + e * h_r0)
}

/// Footnotes c and e of tables C.3/C.4: R_cav;nv of an unventilated air
/// layer thinner than 20 mm without an effective reflective layer, rounded
/// to 2 decimals like the table values (table 8 of NEN-EN-ISO 6946:2017 is
/// built on the same D.2 method).
pub fn thin_cavity_resistance(thickness_mm: f64, heat_flow: HeatFlow) -> f64 {
    round_half_up(
        iso6946_d2_resistance(thickness_mm / 1000.0, heat_flow, 0.9),
        2,
    )
}

/// Exterior surface resistance at a strongly ventilated cavity (C.3.3,
/// footnotes b of tables C.3/C.4).
pub fn still_air_exterior_resistance(heat_flow: HeatFlow, reflective: bool) -> f64 {
    match (heat_flow, reflective) {
        (HeatFlow::Horizontal, false) => 0.12,
        (HeatFlow::Horizontal, true) => 0.22,
        (HeatFlow::Upward, _) => 0.10,
        (HeatFlow::Downward, false) => 0.19,
        (HeatFlow::Downward, true) => 0.83,
    }
}

impl Layer {
    /// R of the layer in (m²·K)/W; `None` for a strongly ventilated cavity
    /// (handled by truncation).
    pub fn resistance(&self, heat_flow: HeatFlow) -> Option<f64> {
        match self {
            Self::Material {
                thickness_m,
                conductivity,
            } => Some(thickness_m / conductivity.lambda_calc()),
            // E.2.1.1: R_calc rounded down.
            Self::Resistance { resistance, .. } => Some(round_resistance_down(*resistance)),
            Self::DeclaredResistance {
                resistance_declared,
                moisture,
                ageing,
                temperature,
                convection_factor,
                ..
            } => Some(round_resistance_down(
                resistance_declared
                    / (temperature
                        .as_ref()
                        .map_or(1.0, TemperatureConversion::factor)
                        * moisture.factor()
                        * ageing.factor()
                        * convection_factor.unwrap_or(1.0)),
            )),
            Self::ReflectiveFoil { system } => Some(system.resistance()),
            Self::AirCavity {
                thickness_mm,
                ventilation,
                ..
            } => {
                if *thickness_mm < 20.0 {
                    // Footnotes c and e of tables C.3/C.4; validation refuses
                    // the thin cases D.2 does not cover here.
                    return match ventilation {
                        CavityVentilation::Strongly => None,
                        _ => Some(thin_cavity_resistance(*thickness_mm, heat_flow)),
                    };
                }
                let (nv, zv) = cavity_table(*thickness_mm, heat_flow, self.effective_reflective());
                match ventilation {
                    CavityVentilation::Unventilated => Some(nv),
                    CavityVentilation::Weakly { opening_mm2: None } => Some(zv),
                    CavityVentilation::Weakly {
                        opening_mm2: Some(av),
                    } => {
                        let sv = 2.0 * zv - nv;
                        Some((1500.0 - av) / 1000.0 * nv + (av - 500.0) / 1000.0 * sv)
                    }
                    CavityVentilation::Strongly => None,
                }
            }
            Self::NarrowCavity {
                thickness_mm,
                width_mm,
            } => Some(narrow_cavity_resistance(*thickness_mm, *width_mm)),
            Self::TubularCavity {
                thickness_mm,
                width_mm,
                orientation,
            } => Some(
                thickness_mm
                    / 1000.0
                    / tubular_cavity_lambda(*thickness_mm, *width_mm, *orientation, heat_flow),
            ),
            Self::Attic { roof } => Some(roof.resistance()),
            Self::VentilatedUnheatedSpace {
                separation_area_m2,
                envelope_parts,
                air_change_rate,
                volume_m3,
            } => {
                let envelope: f64 = envelope_parts
                    .iter()
                    .map(|p| p.area_m2 * p.u_value_w_per_m2k.unwrap_or(2.0))
                    .sum();
                Some(
                    separation_area_m2
                        / (envelope + 0.33 * air_change_rate.unwrap_or(0.3) * volume_m3),
                )
            }
        }
    }

    /// A reflective cavity surface that earns the bracket values of tables
    /// C.3/C.4 (footnote b of table C.4).
    fn effective_reflective(&self) -> bool {
        matches!(
            self,
            Self::AirCavity {
                reflective_surface: true,
                reflective_facing_up,
                hermetically_sealed,
                ..
            } if !*reflective_facing_up || *hermetically_sealed
        )
    }

    /// A strongly ventilated air cavity (C.3.3).
    fn strongly_ventilated(&self) -> bool {
        matches!(
            self,
            Self::AirCavity {
                ventilation: CavityVentilation::Strongly,
                ..
            }
        )
    }

    /// Thickness in m for the C.6 layer split; `None` when unknown.
    fn thickness(&self) -> Option<f64> {
        match self {
            Self::Material { thickness_m, .. } => Some(*thickness_m),
            Self::Resistance { thickness_m, .. } | Self::DeclaredResistance { thickness_m, .. } => {
                *thickness_m
            }
            Self::AirCavity { thickness_mm, .. }
            | Self::NarrowCavity { thickness_mm, .. }
            | Self::TubularCavity { thickness_mm, .. } => Some(thickness_mm / 1000.0),
            Self::ReflectiveFoil { system } => match system {
                ReflectiveFoilSystem::FoilLayers { thickness_m } => Some(*thickness_m),
                _ => None,
            },
            Self::Attic { .. } | Self::VentilatedUnheatedSpace { .. } => None,
        }
    }

    fn outermost_only(&self) -> bool {
        matches!(
            self,
            Self::Attic { .. } | Self::VentilatedUnheatedSpace { .. }
        )
    }

    fn validate(&self, path: &str, issues: &mut Vec<ConstructionIssue>) {
        match self {
            Self::Material {
                thickness_m,
                conductivity,
            } => {
                if !positive(*thickness_m) {
                    issues.push(issue("thickness_invalid", format!("{path}.thicknessM")));
                }
                for m in conductivity.validate() {
                    issues.push(issue(m.code, format!("{path}.conductivity.{}", m.field)));
                }
            }
            Self::Resistance {
                resistance,
                thickness_m,
                source_reference,
            } => {
                if !(resistance.is_finite() && *resistance >= 0.0) {
                    issues.push(issue(
                        "resistance_invalid",
                        format!("{path}.resistanceM2KPerW"),
                    ));
                }
                if thickness_m.is_some_and(|t| !positive(t)) {
                    issues.push(issue("thickness_invalid", format!("{path}.thicknessM")));
                }
                if source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        format!("{path}.sourceReference"),
                    ));
                }
            }
            Self::DeclaredResistance {
                resistance_declared,
                temperature,
                convection_factor,
                thickness_m,
                source_reference,
                ..
            } => {
                if !(resistance_declared.is_finite() && *resistance_declared >= 0.0) {
                    issues.push(issue(
                        "resistance_invalid",
                        format!("{path}.resistanceDeclared"),
                    ));
                }
                if thickness_m.is_some_and(|t| !positive(t)) {
                    issues.push(issue("thickness_invalid", format!("{path}.thicknessM")));
                }
                if let Some(t) = temperature {
                    if !t.mean_temperature_c.is_finite() || !t.conversion_coefficient.is_finite() {
                        issues.push(issue(
                            "temperature_conversion_invalid",
                            format!("{path}.temperature"),
                        ));
                    }
                }
                if convection_factor.is_some_and(|f| !(f.is_finite() && f >= 1.0)) {
                    issues.push(issue(
                        "convection_factor_invalid",
                        format!("{path}.convectionFactor"),
                    ));
                }
                if source_reference.trim().is_empty() {
                    issues.push(issue(
                        "source_reference_required",
                        format!("{path}.sourceReference"),
                    ));
                }
            }
            Self::ReflectiveFoil { system } => {
                if let ReflectiveFoilSystem::FoilLayers { thickness_m } = system {
                    if !positive(*thickness_m) {
                        issues.push(issue(
                            "thickness_invalid",
                            format!("{path}.system.thicknessM"),
                        ));
                    }
                }
            }
            Self::AirCavity {
                thickness_mm,
                ventilation,
                ..
            } => {
                if !positive(*thickness_mm) {
                    issues.push(issue("thickness_invalid", format!("{path}.thicknessMm")));
                } else if *thickness_mm < 20.0
                    && (matches!(ventilation, CavityVentilation::Weakly { .. })
                        || self.effective_reflective())
                {
                    // Footnote c of tables C.3/C.4 covers only unventilated
                    // layers without reflective foil below 20 mm.
                    issues.push(issue(
                        "air_cavity_below_20_mm_unsupported",
                        format!("{path}.thicknessMm"),
                    ));
                }
                if *thickness_mm > 300.0 {
                    issues.push(issue(
                        "air_cavity_above_300_mm_requires_heat_balance",
                        format!("{path}.thicknessMm"),
                    ));
                }
                if let CavityVentilation::Weakly {
                    opening_mm2: Some(av),
                } = ventilation
                {
                    if !(500.0..1500.0).contains(av) {
                        issues.push(issue(
                            "weak_ventilation_opening_out_of_range",
                            format!("{path}.ventilation.openingMm2"),
                        ));
                    }
                }
            }
            Self::NarrowCavity {
                thickness_mm,
                width_mm,
            } => {
                if !(positive(*thickness_mm) && positive(*width_mm)) || *thickness_mm < 2.0 {
                    issues.push(issue("cavity_dimensions_invalid", path.to_string()));
                }
                if *thickness_mm >= 500.0 {
                    issues.push(issue("cavity_is_a_room", path.to_string()));
                }
            }
            Self::TubularCavity {
                thickness_mm,
                width_mm,
                ..
            } => {
                if !(positive(*thickness_mm) && positive(*width_mm)) {
                    issues.push(issue("cavity_dimensions_invalid", path.to_string()));
                }
            }
            Self::Attic { .. } => {}
            Self::VentilatedUnheatedSpace {
                separation_area_m2,
                envelope_parts,
                air_change_rate,
                volume_m3,
            } => {
                if !positive(*separation_area_m2)
                    || !positive(*volume_m3)
                    || envelope_parts.is_empty()
                {
                    issues.push(issue("unheated_space_invalid", path.to_string()));
                }
                if envelope_parts.iter().any(|p| {
                    !positive(p.area_m2) || p.u_value_w_per_m2k.is_some_and(|u| !positive(u))
                }) {
                    issues.push(issue(
                        "unheated_space_envelope_invalid",
                        format!("{path}.envelopeParts"),
                    ));
                }
                if air_change_rate.is_some_and(|n| !(n.is_finite() && n >= 0.0)) {
                    issues.push(issue(
                        "air_change_rate_invalid",
                        format!("{path}.airChangeRate"),
                    ));
                }
            }
        }
    }
}

/// Position of the first strongly ventilated cavity (C.3.3).
fn strong_cavity_index(layers: &[Layer]) -> Option<usize> {
    layers.iter().position(Layer::strongly_ventilated)
}

/// Sum of layer resistances with surface resistances (C.3), applying the
/// strongly ventilated cavity truncation of C.3.3.
pub fn total_resistance(layers: &[Layer], heat_flow: HeatFlow, exterior_air: bool) -> f64 {
    let mut total = heat_flow.r_si();
    for layer in layers {
        match layer.resistance(heat_flow) {
            Some(r) => total += r,
            None => {
                return total
                    + still_air_exterior_resistance(heat_flow, layer.effective_reflective());
            }
        }
    }
    total + exterior_resistance(layers, heat_flow, exterior_air)
}

/// R_se of table C.2, or 0,05 of table C.4 for upward heat flow with an
/// effective reflective cavity layer; 0 without exterior air.
pub fn exterior_resistance(layers: &[Layer], heat_flow: HeatFlow, exterior_air: bool) -> f64 {
    if !exterior_air {
        0.0
    } else if heat_flow == HeatFlow::Upward && layers.iter().any(Layer::effective_reflective) {
        R_SE_UPWARD_REFLECTIVE
    } else {
        R_SE
    }
}

/// Table C.1 classes for the weighting factor a′.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptionClass {
    /// λ > 0,30 without direct shielding by > 20 mm insulation.
    StonyUnshielded,
    /// 0,15 < λ ≤ 0,30 without direct shielding by > 20 mm insulation.
    WoodyUnshielded,
    /// Metal parts shielded on one side by 20–30 mm insulation.
    MetalOneSideShielded,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Section {
    pub id: String,
    /// Projected area of the section A_a (or its fraction).
    pub area: f64,
    pub layers: Vec<Layer>,
}

/// Table 8.2 levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AirVoidLevel {
    None,
    Weak,
    Strong,
}

impl AirVoidLevel {
    pub fn correction(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Weak => 0.01,
            Self::Strong => 0.04,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Fasteners {
    /// 8.10 with χ_fa from a 3D calculation (8.2.4.2).
    PointBridge {
        #[serde(rename = "countPerM2")]
        count_per_m2: f64,
        #[serde(rename = "chiWPerK")]
        chi_w_per_k: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// 8.11/8.12.
    Formula {
        #[serde(rename = "countPerM2")]
        count_per_m2: f64,
        #[serde(rename = "lambdaWPerMK")]
        lambda_w_per_mk: f64,
        #[serde(rename = "crossSectionM2")]
        cross_section_m2: f64,
        #[serde(rename = "penetrationDepthM")]
        penetration_depth_m: f64,
        #[serde(rename = "insulationThicknessM")]
        insulation_thickness_m: f64,
    },
}

/// 8.13 f·x values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvertedRoofDrainage {
    XpsGreenRoof,
    XpsRebatedEdges,
    XpsStraightEdges,
    XpsWithWaterproofVapourOpenLayer,
    OtherInsulation,
}

impl InvertedRoofDrainage {
    pub fn fx(self) -> f64 {
        match self {
            Self::XpsGreenRoof => 0.02,
            Self::XpsRebatedEdges => 0.03,
            Self::XpsStraightEdges => 0.04,
            Self::XpsWithWaterproofVapourOpenLayer => 0.01,
            Self::OtherInsulation => 0.05,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AirVoidCorrection {
    pub level: AirVoidLevel,
    /// Index of the insulation layer containing the voids (R_1).
    pub insulation_layer: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FastenerCorrection {
    pub fasteners: Fasteners,
    /// Index of the penetrated insulation layer (R_1).
    pub insulation_layer: usize,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InvertedRoofCorrection {
    pub drainage: InvertedRoofDrainage,
    /// Index of the insulation layer above the membrane (R_1).
    pub insulation_layer: usize,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Corrections {
    #[serde(default)]
    pub air_voids: Option<AirVoidCorrection>,
    #[serde(default)]
    pub fasteners: Option<FastenerCorrection>,
    #[serde(default)]
    pub inverted_roof: Option<InvertedRoofCorrection>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Build {
    /// C.3.
    Homogeneous { layers: Vec<Layer> },
    /// C.4–C.7 with sections sharing the same layer thicknesses.
    Composite {
        sections: Vec<Section>,
        interruption: InterruptionClass,
        /// Section id holding the insulation for R_1 and R_T of 8.9/8.11/
        /// 8.13; `None` takes the section with the highest C.3 R_T.
        #[serde(default, rename = "insulationSection")]
        insulation_section: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpaqueConstruction {
    pub heat_flow: HeatFlow,
    /// Exterior surface in contact with air (R_se of table C.2).
    #[serde(default = "yes")]
    pub exterior_air: bool,
    pub build: Build,
    #[serde(default)]
    pub corrections: Corrections,
    /// b_U for the equivalent resistance towards an unheated space
    /// (C.9/C.10).
    #[serde(default)]
    pub unheated_reduction_factor: Option<f64>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpaqueResult {
    pub r_t: f64,
    pub r_t_upper: Option<f64>,
    pub r_t_lower: Option<f64>,
    pub weighting_factor: Option<f64>,
    pub u_t: f64,
    pub delta_u_air_voids: f64,
    pub delta_u_fasteners: f64,
    pub delta_u_inverted_roof: f64,
    /// ΔU after the 3 % rule.
    pub delta_u: f64,
    pub u_c: f64,
    /// Presented U_C (2 decimals, 8.2.2.1).
    pub u_c_rounded: f64,
    /// R_C (C.2), presented with 2 decimals.
    pub r_c: f64,
    pub r_c_rounded: f64,
    pub r_equivalent: Option<f64>,
    /// Exterior resistance contained in R_T: R_se of table C.2 (or 0,05 of
    /// table C.4), 0 without exterior air (table C.2 note 1, p. 778), or
    /// the still-air value of C.3.3 behind a strongly ventilated cavity.
    /// A project U-value carries this value; 8.4.2.1 (p. 266) replaces it
    /// by R_si towards an unheated space.
    pub exterior_surface_resistance: f64,
}

impl OpaqueConstruction {
    pub fn validate(&self, path: &str) -> Vec<ConstructionIssue> {
        let mut issues = Vec::new();
        let check_layers = |layers: &[Layer], base: &str, issues: &mut Vec<ConstructionIssue>| {
            if layers.is_empty() {
                issues.push(issue("layers_required", base.to_string()));
            }
            for (i, layer) in layers.iter().enumerate() {
                layer.validate(&format!("{base}[{i}]"), issues);
                if layer.outermost_only() && i + 1 != layers.len() {
                    issues.push(issue("layer_must_be_outermost", format!("{base}[{i}]")));
                }
            }
        };
        match &self.build {
            Build::Homogeneous { layers } => {
                check_layers(layers, &format!("{path}.build.layers"), &mut issues)
            }
            Build::Composite { sections, .. } => {
                if sections.len() < 2 {
                    issues.push(issue(
                        "composite_requires_two_sections",
                        format!("{path}.build.sections"),
                    ));
                }
                for (s, section) in sections.iter().enumerate() {
                    let base = format!("{path}.build.sections[{s}].layers");
                    check_layers(&section.layers, &base, &mut issues);
                    if !positive(section.area) {
                        issues.push(issue(
                            "section_area_invalid",
                            format!("{path}.build.sections[{s}].area"),
                        ));
                    }
                    for (i, layer) in section.layers.iter().enumerate() {
                        if layer.thickness().is_none()
                            || (layer.resistance(self.heat_flow).is_none()
                                && !layer.strongly_ventilated())
                        {
                            issues.push(issue(
                                "composite_layer_requires_thickness",
                                format!("{base}[{i}]"),
                            ));
                        }
                    }
                }
                // C.3.3 truncation in C.5 and C.6: every section has its
                // strongly ventilated cavity at the same layer position.
                let cavity = sections
                    .first()
                    .and_then(|s| strong_cavity_index(&s.layers));
                for (s, section) in sections.iter().enumerate().skip(1) {
                    if strong_cavity_index(&section.layers) != cavity {
                        issues.push(issue(
                            "composite_strong_cavity_mismatch",
                            format!("{path}.build.sections[{s}].layers"),
                        ));
                    }
                }
                if let Some(first) = sections.first() {
                    let reference: Vec<Option<f64>> =
                        first.layers.iter().map(Layer::thickness).collect();
                    for (s, section) in sections.iter().enumerate().skip(1) {
                        let same = section.layers.len() == reference.len()
                            && section.layers.iter().zip(&reference).all(|(l, r)| {
                                match (l.thickness(), r) {
                                    (Some(a), Some(b)) => (a - b).abs() < 1e-6,
                                    _ => false,
                                }
                            });
                        if !same {
                            issues.push(issue(
                                "composite_sections_layer_split_mismatch",
                                format!("{path}.build.sections[{s}]"),
                            ));
                        }
                    }
                }
            }
        }
        if let Build::Composite {
            sections,
            insulation_section: Some(id),
            ..
        } = &self.build
        {
            if !sections.iter().any(|s| &s.id == id) {
                issues.push(issue(
                    "insulation_section_unknown",
                    format!("{path}.build.insulationSection"),
                ));
            }
        }
        let layer_count = match &self.build {
            Build::Homogeneous { layers } => layers.len(),
            Build::Composite { sections, .. } => sections.first().map_or(0, |s| s.layers.len()),
        };
        let index_ok = |i: usize| i < layer_count;
        if let Some(c) = &self.corrections.air_voids {
            if !index_ok(c.insulation_layer) {
                issues.push(issue(
                    "correction_layer_index_invalid",
                    format!("{path}.corrections.airVoids"),
                ));
            }
        }
        if let Some(FastenerCorrection {
            fasteners,
            insulation_layer,
        }) = &self.corrections.fasteners
        {
            if !index_ok(*insulation_layer) {
                issues.push(issue(
                    "correction_layer_index_invalid",
                    format!("{path}.corrections.fasteners"),
                ));
            }
            match fasteners {
                Fasteners::PointBridge {
                    count_per_m2,
                    chi_w_per_k,
                    source_reference,
                } => {
                    if !(count_per_m2.is_finite() && *count_per_m2 >= 0.0)
                        || !(chi_w_per_k.is_finite() && *chi_w_per_k >= 0.0)
                    {
                        issues.push(issue(
                            "fasteners_invalid",
                            format!("{path}.corrections.fasteners"),
                        ));
                    }
                    if source_reference.trim().is_empty() {
                        issues.push(issue(
                            "source_reference_required",
                            format!("{path}.corrections.fasteners.sourceReference"),
                        ));
                    }
                }
                Fasteners::Formula {
                    count_per_m2,
                    lambda_w_per_mk,
                    cross_section_m2,
                    penetration_depth_m,
                    insulation_thickness_m,
                } => {
                    if !(count_per_m2.is_finite() && *count_per_m2 >= 0.0)
                        || !positive(*lambda_w_per_mk)
                        || !positive(*cross_section_m2)
                        || !positive(*penetration_depth_m)
                        || !positive(*insulation_thickness_m)
                    {
                        issues.push(issue(
                            "fasteners_invalid",
                            format!("{path}.corrections.fasteners"),
                        ));
                    }
                    // 8.12: d_fa is the penetration depth into the layer.
                    if penetration_depth_m > insulation_thickness_m {
                        issues.push(issue(
                            "fastener_penetration_exceeds_insulation",
                            format!("{path}.corrections.fasteners.penetrationDepthM"),
                        ));
                    }
                }
            }
        }
        if let Some(c) = &self.corrections.inverted_roof {
            if !index_ok(c.insulation_layer) {
                issues.push(issue(
                    "correction_layer_index_invalid",
                    format!("{path}.corrections.invertedRoof"),
                ));
            }
        }
        if let Some(b) = self.unheated_reduction_factor {
            if !(b > 0.0 && b <= 1.0) {
                issues.push(issue(
                    "reduction_factor_invalid",
                    format!("{path}.unheatedReductionFactor"),
                ));
            }
        }
        if issues.is_empty() {
            // Table F.1 applies to components with U ≤ 1,0, tables F.2/F.3
            // to components with U > 1,0.
            let all_layers: Vec<&Layer> = match &self.build {
                Build::Homogeneous { layers } => layers.iter().collect(),
                Build::Composite { sections, .. } => {
                    sections.iter().flat_map(|s| s.layers.iter()).collect()
                }
            };
            let narrow = all_layers
                .iter()
                .any(|l| matches!(l, Layer::NarrowCavity { .. }));
            let tubular = all_layers
                .iter()
                .any(|l| matches!(l, Layer::TubularCavity { .. }));
            if narrow || tubular {
                let u = self.calculate().u_c;
                if narrow && u > 1.0 {
                    issues.push(issue(
                        "narrow_cavity_table_f1_requires_u_at_most_1",
                        format!("{path}.build"),
                    ));
                }
                if tubular && u <= 1.0 {
                    issues.push(issue(
                        "tubular_cavity_tables_f2_f3_require_u_above_1",
                        format!("{path}.build"),
                    ));
                }
            }
        }
        issues
    }

    /// Layers of the section used for R_1 and R_T in 8.9/8.11/8.13: the
    /// homogeneous build, the named insulation section, or the section with
    /// the highest C.3 R_T.
    fn insulation_layers(&self) -> &[Layer] {
        match &self.build {
            Build::Homogeneous { layers } => layers,
            Build::Composite {
                sections,
                insulation_section,
                ..
            } => {
                if let Some(id) = insulation_section {
                    if let Some(section) = sections.iter().find(|s| &s.id == id) {
                        return &section.layers;
                    }
                }
                sections
                    .iter()
                    .max_by(|a, b| {
                        let ra = total_resistance(&a.layers, self.heat_flow, self.exterior_air);
                        let rb = total_resistance(&b.layers, self.heat_flow, self.exterior_air);
                        ra.total_cmp(&rb)
                    })
                    .map_or(&[], |s| s.layers.as_slice())
            }
        }
    }

    /// R_se of the construction (tables C.2/C.4).
    fn r_se(&self) -> f64 {
        let reflective_upward = self.heat_flow == HeatFlow::Upward
            && match &self.build {
                Build::Homogeneous { layers } => layers.iter().any(Layer::effective_reflective),
                Build::Composite { sections, .. } => sections
                    .iter()
                    .any(|s| s.layers.iter().any(Layer::effective_reflective)),
            };
        if !self.exterior_air {
            0.0
        } else if reflective_upward {
            R_SE_UPWARD_REFLECTIVE
        } else {
            R_SE
        }
    }

    /// Call after `validate` returned no issues.
    pub fn calculate(&self) -> OpaqueResult {
        let (r_t, upper, lower, weight) = match &self.build {
            Build::Homogeneous { layers } => (
                total_resistance(layers, self.heat_flow, self.exterior_air),
                None,
                None,
                None,
            ),
            Build::Composite {
                sections,
                interruption,
                ..
            } => {
                let total_area: f64 = sections.iter().map(|s| s.area).sum();
                // C.5.
                let upper = total_area
                    / sections
                        .iter()
                        .map(|s| {
                            s.area / total_resistance(&s.layers, self.heat_flow, self.exterior_air)
                        })
                        .sum::<f64>();
                // C.6/C.7: λ″ per layer to three decimals. With a strongly
                // ventilated cavity (C.3.3) the layers from the cavity
                // outward are left out and the still-air R_se replaces R_se.
                let cavity = strong_cavity_index(&sections[0].layers);
                let mut lower = self.heat_flow.r_si()
                    + match cavity {
                        Some(index) => still_air_exterior_resistance(
                            self.heat_flow,
                            sections[0].layers[index].effective_reflective(),
                        ),
                        None => self.r_se(),
                    };
                for j in 0..cavity.unwrap_or(sections[0].layers.len()) {
                    let d = sections[0].layers[j].thickness().unwrap_or(0.0);
                    let lambda: f64 = sections
                        .iter()
                        .map(|s| {
                            let layer = &s.layers[j];
                            let r = layer.resistance(self.heat_flow).unwrap_or(0.0);
                            (d / r) * s.area
                        })
                        .sum::<f64>()
                        / total_area;
                    let lambda = round_half_up(lambda, 3);
                    lower += d / lambda;
                }
                // Table C.1.
                let weight = if upper <= 1.05 * lower {
                    0.0
                } else {
                    match interruption {
                        InterruptionClass::StonyUnshielded => 0.0,
                        InterruptionClass::WoodyUnshielded
                        | InterruptionClass::MetalOneSideShielded => 0.5,
                        InterruptionClass::Other => 1.0,
                    }
                };
                // C.4.
                let r_t = (weight * upper + lower) / (1.0 + 1.05 * weight);
                (r_t, Some(upper), Some(lower), Some(weight))
            }
        };
        let u_t = 1.0 / r_t;
        // 8.9/8.11/8.13: R_1 and R_T (C.3, thermal bridges neglected) of the
        // insulation section.
        let basis = self.insulation_layers();
        let basis_r_t = total_resistance(basis, self.heat_flow, self.exterior_air);
        let ratio = |index: usize| {
            let r_1 = basis
                .get(index)
                .and_then(|layer| layer.resistance(self.heat_flow))
                .unwrap_or(0.0);
            (r_1 / basis_r_t).powi(2)
        };
        let delta_a = self
            .corrections
            .air_voids
            .map_or(0.0, |c| c.level.correction() * ratio(c.insulation_layer));
        let delta_fa = self
            .corrections
            .fasteners
            .as_ref()
            .map_or(0.0, |c| match &c.fasteners {
                Fasteners::PointBridge {
                    count_per_m2,
                    chi_w_per_k,
                    ..
                } => count_per_m2 * chi_w_per_k,
                Fasteners::Formula {
                    count_per_m2,
                    lambda_w_per_mk,
                    cross_section_m2,
                    penetration_depth_m,
                    insulation_thickness_m,
                } => {
                    let alpha = 0.8 * penetration_depth_m / insulation_thickness_m
                        * (count_per_m2 * lambda_w_per_mk * cross_section_m2)
                        / insulation_thickness_m;
                    alpha * ratio(c.insulation_layer)
                }
            });
        let delta_r = self.corrections.inverted_roof.map_or(0.0, |c| {
            PRECIPITATION_MM_PER_DAY * c.drainage.fx() * ratio(c.insulation_layer)
        });
        let total = delta_a + delta_fa + delta_r;
        let delta_u = if total > 0.03 * u_t { total } else { 0.0 };
        let u_c = u_t / F_PRAC_OPAQUE + delta_u;
        let r_se = self.r_se();
        let first_layers = match &self.build {
            Build::Homogeneous { layers } => layers.as_slice(),
            Build::Composite { sections, .. } => sections[0].layers.as_slice(),
        };
        let exterior_surface_resistance = match strong_cavity_index(first_layers) {
            Some(index) => still_air_exterior_resistance(
                self.heat_flow,
                first_layers[index].effective_reflective(),
            ),
            None => r_se,
        };
        // C.2 with β = R_T·ΔU (C.8).
        let r_c = r_t / (1.0 + r_t * delta_u) - self.heat_flow.r_si() - r_se;
        let r_equivalent = self
            .unheated_reduction_factor
            .map(|b| 1.0 / (u_c * b) - self.heat_flow.r_si() - R_SE);
        OpaqueResult {
            r_t,
            r_t_upper: upper,
            r_t_lower: lower,
            weighting_factor: weight,
            u_t,
            delta_u_air_voids: delta_a,
            delta_u_fasteners: delta_fa,
            delta_u_inverted_roof: delta_r,
            delta_u,
            u_c,
            u_c_rounded: round_half_up(u_c, 2),
            r_c,
            r_c_rounded: round_half_up(r_c, 2),
            r_equivalent,
            exterior_surface_resistance,
        }
    }
}

/// Figure C.3 shapes of tapered insulation parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaperShape {
    /// Type 1.
    Rectangular,
    /// Type 2: triangle, thickest at the apex.
    TriangleThickestAtApex,
    /// Type 3: triangle, thickest along the base.
    TriangleThickestAtBase,
    /// Type 4: triangle with different thicknesses at the corners.
    TriangleVarying,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaperPart {
    pub shape: TaperShape,
    pub area_m2: f64,
    /// d_1 (type 4 only), m.
    #[serde(default)]
    pub intermediate_thickness_m: Option<f64>,
    /// d_2, m.
    pub max_thickness_m: f64,
}

/// C.17–C.20 for one part; r0 includes both surface resistances.
pub fn tapered_part_u(shape: TaperShape, r0: f64, r1: f64, r2: f64) -> f64 {
    if r2 < 1e-12 {
        return 1.0 / r0;
    }
    let ln = (1.0 + r2 / r0).ln();
    match shape {
        TaperShape::Rectangular => ln / r2,
        TaperShape::TriangleThickestAtApex => 2.0 / r2 * ((1.0 + r0 / r2) * ln - 1.0),
        TaperShape::TriangleThickestAtBase => 2.0 / r2 * (1.0 - r0 / r2 * ln),
        TaperShape::TriangleVarying => {
            if (r2 - r1).abs() < 1e-12 || r1 < 1e-12 {
                // Degenerates to type 2 (d1 = d2) or type 3 (d1 = 0).
                let shape = if r1 < 1e-12 {
                    TaperShape::TriangleThickestAtApex
                } else {
                    TaperShape::TriangleThickestAtBase
                };
                return tapered_part_u(shape, r0, r1, r2);
            }
            2.0 * (r0 * r1 * (1.0 + r2 / r0).ln() - r0 * r2 * (1.0 + r1 / r0).ln()
                + r1 * r2 * ((r0 + r2) / (r0 + r1)).ln())
                / (r1 * r2 * (r2 - r1))
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TaperedRoof {
    /// Base construction without the tapered part (C.3 → R_0).
    pub base_layers: Vec<Layer>,
    pub taper_conductivity: Conductivity,
    pub parts: Vec<TaperPart>,
    /// Slope of the tapered insulation; C.16 applies up to 5 %.
    pub slope_percent: f64,
}

impl TaperedRoof {
    pub fn validate(&self, path: &str) -> Vec<ConstructionIssue> {
        let mut issues = Vec::new();
        for (i, layer) in self.base_layers.iter().enumerate() {
            layer.validate(&format!("{path}.baseLayers[{i}]"), &mut issues);
        }
        for m in self.taper_conductivity.validate() {
            issues.push(issue(
                m.code,
                format!("{path}.taperConductivity.{}", m.field),
            ));
        }
        if !(self.slope_percent.is_finite() && self.slope_percent >= 0.0) {
            issues.push(issue("slope_invalid", format!("{path}.slopePercent")));
        } else if self.slope_percent > 5.0 {
            issues.push(issue(
                "tapered_roof_above_5_percent_requires_numerical_method",
                format!("{path}.slopePercent"),
            ));
        }
        if self.parts.is_empty() {
            issues.push(issue("taper_part_required", format!("{path}.parts")));
        }
        for (i, part) in self.parts.iter().enumerate() {
            let p = format!("{path}.parts[{i}]");
            if !positive(part.area_m2)
                || !(part.max_thickness_m.is_finite() && part.max_thickness_m >= 0.0)
            {
                issues.push(issue("taper_part_invalid", p.clone()));
            }
            if part.shape == TaperShape::TriangleVarying {
                match part.intermediate_thickness_m {
                    Some(d1) if d1 >= 0.0 && d1 <= part.max_thickness_m => {}
                    _ => issues.push(issue("taper_intermediate_thickness_invalid", p)),
                }
            }
        }
        issues
    }

    /// U_T (C.16), W/(m²·K).
    pub fn u_t(&self) -> f64 {
        let r0 = total_resistance(&self.base_layers, HeatFlow::Upward, true);
        let lambda = self.taper_conductivity.lambda_calc();
        let (sum_au, sum_a) = self.parts.iter().fold((0.0, 0.0), |(au, a), part| {
            let r1 = part.intermediate_thickness_m.unwrap_or(0.0) / lambda;
            let r2 = part.max_thickness_m / lambda;
            (
                au + part.area_m2 * tapered_part_u(part.shape, r0, r1, r2),
                a + part.area_m2,
            )
        });
        sum_au / sum_a
    }
}

/// 8.3: forfait linear-bridge supplement from the opaque parts (not
/// floors over crawlspaces or on ground, not panels).
pub fn forfait_bridge_supplement(parts: &[(f64, f64)]) -> f64 {
    let area: f64 = parts.iter().map(|(a, _)| a).sum();
    if area <= 0.0 {
        return 0.0;
    }
    let mean = parts.iter().map(|(a, u)| a * u).sum::<f64>() / area;
    (0.1 - 0.25 * (mean - 0.4)).max(0.0)
}

/// Note 2 of 8.2.2.1: U_C of a rooflight from U_rc (NEN-EN 1873).
pub fn rooflight_u(u_rc: f64, area_with_upstand_m2: f64, projected_area_m2: f64) -> f64 {
    u_rc * area_with_upstand_m2 / projected_area_m2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::materials::{Ageing, ForfaitMaterial, InsulationMoisture};

    fn material(thickness: f64, lambda: f64) -> Layer {
        Layer::Material {
            thickness_m: thickness,
            conductivity: Conductivity::Calculated {
                lambda_calc: lambda,
                source_reference: "test".into(),
            },
        }
    }

    fn cavity_wall() -> OpaqueConstruction {
        OpaqueConstruction {
            heat_flow: HeatFlow::Horizontal,
            exterior_air: true,
            build: Build::Homogeneous {
                layers: vec![
                    material(0.1, 1.0),
                    Layer::Material {
                        thickness_m: 0.15,
                        conductivity: Conductivity::ForfaitInsulation {
                            material: ForfaitMaterial::StoneWool,
                            moisture: InsulationMoisture::Regular,
                            ageing: Ageing::FactoryMade,
                        },
                    },
                    Layer::AirCavity {
                        thickness_mm: 40.0,
                        ventilation: CavityVentilation::Weakly { opening_mm2: None },
                        reflective_surface: false,
                        reflective_facing_up: false,
                        hermetically_sealed: false,
                    },
                    material(0.1, 1.16),
                ],
            },
            corrections: Corrections {
                air_voids: Some(AirVoidCorrection {
                    level: AirVoidLevel::Weak,
                    insulation_layer: 1,
                }),
                fasteners: None,
                inverted_roof: None,
            },
            unheated_reduction_factor: None,
        }
    }

    #[test]
    fn exterior_surface_resistance_follows_table_c2_note_1() {
        let wall = cavity_wall();
        assert_eq!(wall.calculate().exterior_surface_resistance, R_SE);
        let mut buried = cavity_wall();
        buried.exterior_air = false;
        assert_eq!(buried.calculate().exterior_surface_resistance, 0.0);
    }

    #[test]
    fn homogeneous_wall_follows_c3_and_8_4() {
        let wall = cavity_wall();
        assert!(wall.validate("e").is_empty());
        let r = wall.calculate();
        let r_t = 0.13 + 0.1 + 0.15 / 0.04 + 0.15 + 0.1 / 1.16 + 0.04;
        assert!((r.r_t - r_t).abs() < 1e-12);
        let delta_a = 0.01 * (3.75 / r_t).powi(2);
        assert!((r.delta_u_air_voids - delta_a).abs() < 1e-12);
        // ΔU_a ≈ 0,0078 exceeds 3 % of U_T ≈ 0,0070.
        assert!(delta_a > 0.03 / r_t);
        assert!((r.delta_u - delta_a).abs() < 1e-12);
        // Without corrections R_C = R_T − R_si − R_se.
        let mut plain = cavity_wall();
        plain.corrections = Corrections::default();
        let p = plain.calculate();
        assert_eq!(p.delta_u, 0.0);
        assert!((p.r_c - (r_t - 0.17)).abs() < 1e-12);
        // Weak voids in a thin layer stay under the 3 % threshold.
        let mut thin = cavity_wall();
        thin.corrections.air_voids = Some(AirVoidCorrection {
            level: AirVoidLevel::Weak,
            insulation_layer: 0,
        });
        assert_eq!(thin.calculate().delta_u, 0.0);
    }

    #[test]
    fn correction_applies_above_three_percent() {
        let mut wall = cavity_wall();
        wall.corrections.air_voids = Some(AirVoidCorrection {
            level: AirVoidLevel::Strong,
            insulation_layer: 1,
        });
        wall.corrections.fasteners = Some(FastenerCorrection {
            fasteners: Fasteners::PointBridge {
                count_per_m2: 6.0,
                chi_w_per_k: 0.004,
                source_reference: "3D".into(),
            },
            insulation_layer: 1,
        });
        let r = wall.calculate();
        let expected = 0.04 * (3.75 / r.r_t).powi(2) + 0.024;
        assert!((r.delta_u - expected).abs() < 1e-12);
        assert!((r.u_c - (1.0 / r.r_t + expected)).abs() < 1e-12);
        let beta = r.r_t * expected;
        assert!((r.r_c - (r.r_t / (1.0 + beta) - 0.17)).abs() < 1e-12);
        // Consistency: 1/(R_c + R_si + R_se) = U_C.
        assert!((1.0 / (r.r_c + 0.17) - r.u_c).abs() < 1e-12);
    }

    #[test]
    fn composite_construction_follows_c4_to_c7() {
        let section = |id: &str, area: f64, lambda: f64| Section {
            id: id.into(),
            area,
            layers: vec![
                material(0.012, 0.17),
                material(0.1, lambda),
                material(0.012, 0.17),
            ],
        };
        let wall = OpaqueConstruction {
            heat_flow: HeatFlow::Horizontal,
            exterior_air: true,
            build: Build::Composite {
                sections: vec![
                    section("insulation", 0.85, 0.035),
                    section("stud", 0.15, 0.3),
                ],
                interruption: InterruptionClass::WoodyUnshielded,
                insulation_section: None,
            },
            corrections: Corrections::default(),
            unheated_reduction_factor: None,
        };
        assert!(wall.validate("e").is_empty());
        let r = wall.calculate();
        let skin = 2.0 * 0.012 / 0.17;
        let ra = 0.17 + skin + 0.1 / 0.035;
        let rb = 0.17 + skin + 0.1 / 0.3;
        let upper = 1.0 / (0.85 / ra + 0.15 / rb);
        let lambda = round_half_up(0.85 * 0.035 + 0.15 * 0.3, 3);
        assert!(upper > 1.05 * (0.17 + skin + 0.1 / lambda));
        let lower = 0.17 + skin + 0.1 / lambda;
        assert!((r.r_t_upper.unwrap() - upper).abs() < 1e-12);
        assert!((r.r_t_lower.unwrap() - lower).abs() < 1e-12);
        assert_eq!(r.weighting_factor, Some(0.5));
        assert!((r.r_t - (0.5 * upper + lower) / (1.0 + 0.525)).abs() < 1e-12);
    }

    #[test]
    fn composite_with_strongly_ventilated_cavity_truncates_both_bounds() {
        // Rafter roof under ventilated tiles: C.3.3 drops the cavity and
        // the tiles and takes the still-air R_se (0,10 upward) in C.5 and C.6.
        let strong = || Layer::AirCavity {
            thickness_mm: 30.0,
            ventilation: CavityVentilation::Strongly,
            reflective_surface: false,
            reflective_facing_up: false,
            hermetically_sealed: false,
        };
        let section = |id: &str, area: f64, lambda: f64| Section {
            id: id.into(),
            area,
            layers: vec![
                material(0.012, 0.17),
                material(0.15, lambda),
                strong(),
                material(0.02, 1.0),
            ],
        };
        let roof = OpaqueConstruction {
            heat_flow: HeatFlow::Upward,
            exterior_air: true,
            build: Build::Composite {
                sections: vec![
                    section("insulation", 0.85, 0.035),
                    section("rafter", 0.15, 0.13),
                ],
                interruption: InterruptionClass::WoodyUnshielded,
                insulation_section: None,
            },
            corrections: Corrections::default(),
            unheated_reduction_factor: None,
        };
        assert!(roof.validate("r").is_empty(), "{:?}", roof.validate("r"));
        let r = roof.calculate();
        let skin = 0.012 / 0.17;
        let ra = 0.10 + skin + 0.15 / 0.035 + 0.10;
        let rb = 0.10 + skin + 0.15 / 0.13 + 0.10;
        let upper = 1.0 / (0.85 / ra + 0.15 / rb);
        let lambda = round_half_up(0.85 * 0.035 + 0.15 * 0.13, 3);
        let lower = 0.10 + 0.10 + skin + 0.15 / lambda;
        assert!((r.r_t_upper.unwrap() - upper).abs() < 1e-12);
        assert!((r.r_t_lower.unwrap() - lower).abs() < 1e-12);
        // The still-air value stands in for R_se in the project U-value.
        assert_eq!(r.exterior_surface_resistance, 0.10);

        // The cavity must sit at the same position in every section.
        let mut mixed = roof.clone();
        if let Build::Composite { sections, .. } = &mut mixed.build {
            sections[1].layers[2] = material(0.03, 0.025);
        }
        assert!(mixed
            .validate("r")
            .iter()
            .any(|issue| issue.code == "composite_strong_cavity_mismatch"));
    }

    #[test]
    fn cavities_and_special_layers() {
        assert_eq!(cavity_table(40.0, HeatFlow::Downward, false), (0.19, 0.15));
        assert_eq!(cavity_table(120.0, HeatFlow::Downward, true), (1.19, 0.71));
        let weak = Layer::AirCavity {
            thickness_mm: 30.0,
            ventilation: CavityVentilation::Weakly {
                opening_mm2: Some(1000.0),
            },
            reflective_surface: false,
            reflective_facing_up: false,
            hermetically_sealed: false,
        };
        assert!((weak.resistance(HeatFlow::Horizontal).unwrap() - 0.15).abs() < 1e-12);
        // Strongly ventilated: outer layers dropped, still-air R_se 0,12.
        let layers = vec![
            material(0.1, 0.04),
            Layer::AirCavity {
                thickness_mm: 30.0,
                ventilation: CavityVentilation::Strongly,
                reflective_surface: false,
                reflective_facing_up: false,
                hermetically_sealed: false,
            },
            material(0.1, 1.0),
        ];
        assert!(
            (total_resistance(&layers, HeatFlow::Horizontal, true) - (0.13 + 2.5 + 0.12)).abs()
                < 1e-12
        );
        assert!((narrow_cavity_resistance(10.0, 5.0) - 0.20).abs() < 1e-12);
        // Between d/b = 1 and 2 at d = 5: 0,13.
        assert!((narrow_cavity_resistance(5.0, 3.333_333) - 0.13).abs() < 1e-9);
        assert!(
            (tubular_cavity_lambda(20.0, 10.0, TubeOrientation::Horizontal, HeatFlow::Upward)
                - 0.12)
                .abs()
                < 1e-12
        );
        let space = Layer::VentilatedUnheatedSpace {
            separation_area_m2: 20.0,
            envelope_parts: vec![EnvelopePart {
                area_m2: 30.0,
                u_value_w_per_m2k: None,
            }],
            air_change_rate: None,
            volume_m3: 50.0,
        };
        assert!(
            (space.resistance(HeatFlow::Upward).unwrap() - 20.0 / (60.0 + 0.33 * 0.3 * 50.0)).abs()
                < 1e-12
        );
    }

    #[test]
    fn tapered_roof_follows_c16_to_c20() {
        let (r0, r2): (f64, f64) = (2.0, 4.0);
        let ln: f64 = (1.0 + r2 / r0).ln();
        assert!((tapered_part_u(TaperShape::Rectangular, r0, 0.0, r2) - ln / r2).abs() < 1e-12);
        assert!(
            (tapered_part_u(TaperShape::TriangleThickestAtApex, r0, 0.0, r2)
                - 2.0 / r2 * ((1.0 + r0 / r2) * ln - 1.0))
                .abs()
                < 1e-12
        );
        assert!(
            (tapered_part_u(TaperShape::TriangleThickestAtBase, r0, 0.0, r2)
                - 2.0 / r2 * (1.0 - r0 / r2 * ln))
                .abs()
                < 1e-12
        );
        // Type 4 approaches type 2 when d1 → 0.
        let type4 = tapered_part_u(TaperShape::TriangleVarying, r0, 1e-6, r2);
        let type2 = tapered_part_u(TaperShape::TriangleThickestAtApex, r0, 0.0, r2);
        assert!((type4 - type2).abs() < 1e-4);
        // A rectangular part lies between the thin and thick ends.
        let u = tapered_part_u(TaperShape::Rectangular, r0, 0.0, r2);
        assert!(u < 1.0 / r0 && u > 1.0 / (r0 + r2));
    }

    #[test]
    fn forfait_supplement_matches_table_8_1() {
        assert!((forfait_bridge_supplement(&[(10.0, 0.8)]) - 0.0).abs() < 1e-12);
        assert!((forfait_bridge_supplement(&[(10.0, 0.6)]) - 0.05).abs() < 1e-12);
        assert!((forfait_bridge_supplement(&[(5.0, 0.3), (5.0, 0.5)]) - 0.1).abs() < 1e-12);
        assert!((rooflight_u(1.5, 2.0, 1.6) - 1.875).abs() < 1e-12);
    }

    /// Footnote e of tables C.3/C.4 (2025+C1 p. 782 and 784): D.2 of
    /// NEN-EN-ISO 6946:2017 with ε1 = ε2 = 0,9 (ε2 = 0,1 for the bracket
    /// values) and h_r0 at 10 °C reproduces every unventilated value of
    /// tables C.3 and C.4, including the conduction branch of the downward
    /// rows of 20–50 mm.
    #[test]
    fn d2_method_reproduces_tables_c3_and_c4() {
        for heat_flow in [HeatFlow::Horizontal, HeatFlow::Upward, HeatFlow::Downward] {
            for d in [20.0, 25.0, 50.0, 100.0, 300.0] {
                for reflective in [false, true] {
                    let (table, _) = cavity_table(d, heat_flow, reflective);
                    let eps2 = if reflective { 0.1 } else { 0.9 };
                    let r = round_half_up(iso6946_d2_resistance(d / 1000.0, heat_flow, eps2), 2);
                    assert!(
                        (r - table).abs() < 1e-9,
                        "{heat_flow:?} {d} mm reflective {reflective}: {r} vs {table}"
                    );
                }
            }
        }
    }

    /// Footnote c of tables C.3/C.4: below 20 mm the still-air conduction
    /// λ/d governs h_a; R falls towards 0 with the thickness.
    #[test]
    fn thin_unventilated_cavity_follows_d2() {
        let h = |d| thin_cavity_resistance(d, HeatFlow::Horizontal);
        assert_eq!(h(5.0), 0.11);
        assert_eq!(h(7.0), 0.13);
        assert_eq!(h(10.0), 0.15);
        assert_eq!(h(15.0), 0.17);
        assert_eq!(thin_cavity_resistance(15.0, HeatFlow::Upward), 0.16);
        assert_eq!(thin_cavity_resistance(10.0, HeatFlow::Upward), 0.15);
        assert_eq!(thin_cavity_resistance(15.0, HeatFlow::Downward), 0.17);
        assert!(h(1.0) < h(5.0));
        // Continuous with the table at 20 mm.
        assert_eq!(thin_cavity_resistance(19.99, HeatFlow::Horizontal), 0.18);

        let layer = |thickness_mm, ventilation, reflective_surface| Layer::AirCavity {
            thickness_mm,
            ventilation,
            reflective_surface,
            reflective_facing_up: false,
            hermetically_sealed: false,
        };
        let codes = |layer: &Layer| {
            let mut issues = Vec::new();
            layer.validate("l", &mut issues);
            issues.into_iter().map(|i| i.code).collect::<Vec<_>>()
        };
        let unventilated = layer(10.0, CavityVentilation::Unventilated, false);
        assert_eq!(unventilated.resistance(HeatFlow::Horizontal), Some(0.15));
        assert!(codes(&unventilated).is_empty());
        // Strongly ventilated: truncation as for thicker layers.
        let strong = layer(10.0, CavityVentilation::Strongly, false);
        assert_eq!(strong.resistance(HeatFlow::Horizontal), None);
        assert!(codes(&strong).is_empty());
        // Weak ventilation and an effective reflective layer stay refused.
        for refused in [
            layer(10.0, CavityVentilation::Weakly { opening_mm2: None }, false),
            layer(10.0, CavityVentilation::Unventilated, true),
        ] {
            assert_eq!(codes(&refused), vec!["air_cavity_below_20_mm_unsupported"]);
        }
        let zero = layer(0.0, CavityVentilation::Unventilated, false);
        assert_eq!(codes(&zero), vec!["thickness_invalid"]);
    }

    #[test]
    fn validation_catches_unsupported_layers() {
        let wall = OpaqueConstruction {
            heat_flow: HeatFlow::Horizontal,
            exterior_air: true,
            build: Build::Homogeneous {
                layers: vec![
                    Layer::Attic {
                        roof: AtticRoof::BoardingAndFelt,
                    },
                    Layer::AirCavity {
                        thickness_mm: 10.0,
                        ventilation: CavityVentilation::Weakly { opening_mm2: None },
                        reflective_surface: false,
                        reflective_facing_up: false,
                        hermetically_sealed: false,
                    },
                ],
            },
            corrections: Corrections {
                air_voids: Some(AirVoidCorrection {
                    level: AirVoidLevel::Weak,
                    insulation_layer: 5,
                }),
                ..Corrections::default()
            },
            unheated_reduction_factor: Some(1.5),
        };
        let codes: Vec<_> = wall.validate("e").into_iter().map(|i| i.code).collect();
        for code in [
            "layer_must_be_outermost",
            "air_cavity_below_20_mm_unsupported",
            "correction_layer_index_invalid",
            "reduction_factor_invalid",
        ] {
            assert!(codes.contains(&code), "{code} missing in {codes:?}");
        }
    }

    fn composite_roof(first_insulation: bool) -> OpaqueConstruction {
        let section = |id: &str, area: f64, lambda: f64| Section {
            id: id.into(),
            area,
            layers: vec![
                material(0.012, 0.17),
                material(0.15, lambda),
                material(0.018, 0.13),
            ],
        };
        let mut sections = vec![
            section("insulation", 0.9, 0.04),
            section("rafter", 0.1, 0.13),
        ];
        if !first_insulation {
            sections.reverse();
        }
        OpaqueConstruction {
            heat_flow: HeatFlow::Upward,
            exterior_air: true,
            build: Build::Composite {
                sections,
                interruption: InterruptionClass::WoodyUnshielded,
                insulation_section: None,
            },
            corrections: Corrections {
                air_voids: Some(AirVoidCorrection {
                    level: AirVoidLevel::Weak,
                    insulation_layer: 1,
                }),
                ..Corrections::default()
            },
            unheated_reduction_factor: None,
        }
    }

    #[test]
    fn composite_delta_u_uses_the_insulation_section_c3() {
        let a = composite_roof(true);
        let b = composite_roof(false);
        assert!(a.validate("r").is_empty());
        let (ra, rb) = (a.calculate(), b.calculate());
        // Independent of the section order.
        assert!((ra.delta_u_air_voids - rb.delta_u_air_voids).abs() < 1e-15);
        // 8.9 with R_1 and the C.3 R_T of the insulation section.
        let r_1: f64 = 0.15 / 0.04;
        let rt_c3 = 0.10 + 0.012 / 0.17 + r_1 + 0.018 / 0.13 + 0.04;
        assert!((ra.delta_u_air_voids - 0.01 * (r_1 / rt_c3).powi(2)).abs() < 1e-12);
        // An explicit insulation section takes precedence.
        let mut named = composite_roof(true);
        if let Build::Composite {
            insulation_section, ..
        } = &mut named.build
        {
            *insulation_section = Some("rafter".into());
        }
        let r_rafter: f64 = 0.15 / 0.13;
        let rt_rafter = 0.10 + 0.012 / 0.17 + r_rafter + 0.018 / 0.13 + 0.04;
        assert!(
            (named.calculate().delta_u_air_voids - 0.01 * (r_rafter / rt_rafter).powi(2)).abs()
                < 1e-12
        );
        if let Build::Composite {
            insulation_section, ..
        } = &mut named.build
        {
            *insulation_section = Some("missing".into());
        }
        assert!(named
            .validate("r")
            .iter()
            .any(|i| i.code == "insulation_section_unknown"));
    }

    #[test]
    fn upward_reflective_layers_follow_table_c4() {
        let cavity = |facing_up: bool, sealed: bool| Layer::AirCavity {
            thickness_mm: 50.0,
            ventilation: CavityVentilation::Unventilated,
            reflective_surface: true,
            reflective_facing_up: facing_up,
            hermetically_sealed: sealed,
        };
        let roof = |layer: Layer| OpaqueConstruction {
            heat_flow: HeatFlow::Upward,
            exterior_air: true,
            build: Build::Homogeneous {
                layers: vec![material(0.1, 0.04), layer, material(0.02, 0.13)],
            },
            corrections: Corrections::default(),
            unheated_reduction_factor: None,
        };
        let base = 0.10 + 2.5 + 0.02 / 0.13;
        // Facing down: bracket values 0,41 and R_se 0,05.
        let down = roof(cavity(false, false)).calculate();
        assert!((down.r_t - (base + 0.41 + 0.05)).abs() < 1e-12);
        assert!((down.r_c - (down.r_t - 0.10 - 0.05)).abs() < 1e-12);
        // Facing up without sealing: no bonus, R_se 0,04.
        let up = roof(cavity(true, false)).calculate();
        assert!((up.r_t - (base + 0.16 + 0.04)).abs() < 1e-12);
        // Hermetically sealed: bonus again.
        let sealed = roof(cavity(true, true)).calculate();
        assert!((sealed.r_t - down.r_t).abs() < 1e-12);
    }

    #[test]
    fn declared_resistance_and_foils_round_down() {
        let layer = Layer::DeclaredResistance {
            resistance_declared: 2.5,
            moisture: InsulationMoisture::Regular,
            ageing: Ageing::InSitu {
                product: crate::materials::InSituProduct::EpsBeads,
                situation: crate::materials::InSituSituation::B,
                practice_tested: false,
            },
            temperature: None,
            convection_factor: None,
            thickness_m: None,
            source_reference: "DoP".into(),
        };
        // 2,5/(1,05·1,15) = 2,0704… → 2,07.
        assert_eq!(layer.resistance(HeatFlow::Horizontal), Some(2.07));
        let foil = ReflectiveFoilSystem::FoilLayers {
            thickness_m: 0.0101,
        };
        // 0,0101/0,03 = 0,3366… → 0,33.
        assert!((foil.resistance() - 0.33).abs() < 1e-12);
    }

    #[test]
    fn annex_f_tables_follow_the_u_value_and_fasteners_stay_in_the_layer() {
        let wall = |layers: Vec<Layer>| OpaqueConstruction {
            heat_flow: HeatFlow::Horizontal,
            exterior_air: true,
            build: Build::Homogeneous { layers },
            corrections: Corrections::default(),
            unheated_reduction_factor: None,
        };
        let narrow = Layer::NarrowCavity {
            thickness_mm: 10.0,
            width_mm: 5.0,
        };
        let tube = Layer::TubularCavity {
            thickness_mm: 20.0,
            width_mm: 40.0,
            orientation: TubeOrientation::Horizontal,
        };
        let codes = |w: OpaqueConstruction| -> Vec<&'static str> {
            w.validate("w").iter().map(|i| i.code).collect()
        };
        assert!(codes(wall(vec![material(0.1, 0.04), narrow.clone()])).is_empty());
        assert!(codes(wall(vec![material(0.1, 1.0), narrow]))
            .contains(&"narrow_cavity_table_f1_requires_u_at_most_1"));
        assert!(codes(wall(vec![material(0.01, 1.0), tube.clone()])).is_empty());
        assert!(codes(wall(vec![material(0.1, 0.04), tube]))
            .contains(&"tubular_cavity_tables_f2_f3_require_u_above_1"));
        let mut anchored = wall(vec![material(0.1, 0.04)]);
        anchored.corrections.fasteners = Some(FastenerCorrection {
            fasteners: Fasteners::Formula {
                count_per_m2: 4.0,
                lambda_w_per_mk: 17.0,
                cross_section_m2: 1.3e-5,
                penetration_depth_m: 0.12,
                insulation_thickness_m: 0.1,
            },
            insulation_layer: 0,
        });
        assert!(codes(anchored).contains(&"fastener_penetration_exceeds_insulation"));
    }
}
