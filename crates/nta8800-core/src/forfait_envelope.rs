//! Forfait (default) envelope values for existing buildings,
//! NTA 8800:2025+C1:2026 annex I (pp. 821–842).
//!
//! - ψ of building details: tables I.1 (low-rise) and I.2 (stacked
//!   buildings), columns A (additional conditions met) and B, with 0,5
//!   W/(m·K) for positions without a value.
//! - R_c of closed elements: decision tree figure I.4 with table I.4
//!   (before 1965), tables I.5–I.7 (from 1965; caravans; floating
//!   buildings) and formula I.2 for a known insulation thickness, U_c via
//!   I.1.
//! - U of windows and glazed doors (tables I.8/I.9), doors (table I.10)
//!   and panels in frames (tables I.11/I.12, I.15/I.16).
//! - H_ue of an adjacent unheated space in the basic survey (I.8).

use serde::{Deserialize, Serialize};

use crate::materials::round_half_up;
use crate::window_u::FrameGroup;

/// Table I.1/I.2 positions without a value.
pub const PSI_DEFAULT: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PsiColumn {
    /// Additional conditions of the table are met.
    A,
    B,
}

/// (position, variant, ψ_A, ψ_B) for tables I.1 (1–24) and I.2 (50–74).
const PSI_TABLE: &[(u16, u8, f64, f64)] = &[
    (1, 0, 0.27, 0.41),
    (2, 0, 0.45, 0.68),
    (3, 0, 0.60, 0.90),
    (4, 0, 0.00, 0.00),
    (5, 0, 0.15, 0.25),
    (6, 0, 0.09, 0.19),
    (7, 0, 0.10, 0.20),
    (8, 0, 0.10, 0.20),
    (9, 0, 0.14, 0.24),
    (10, 0, 0.09, 0.19),
    (11, 0, 0.15, 0.25),
    (12, 0, 0.00, 0.00),
    (13, 0, 0.16, 0.26),
    (14, 0, 0.03, 0.13),
    (15, 0, 0.13, 0.23),
    (16, 0, 0.05, 0.15),
    (17, 0, 0.06, 0.09),
    (18, 0, 0.50, 0.75),
    (19, 0, 0.13, 0.23),
    (20, 0, 0.12, 0.22),
    (21, 0, 0.14, 0.24),
    (22, 0, 0.12, 0.22),
    (23, 0, 0.24, 0.36),
    // 24: insulation interrupted at most by timber / by stainless lintels.
    (24, 0, 0.13, 0.23),
    (24, 1, 0.41, 0.62),
    (50, 0, 0.61, 0.92),
    (51, 0, 0.64, 0.96),
    (52, 0, 0.64, 0.96),
    (53, 0, 0.00, 0.00),
    (54, 0, 0.15, 0.25),
    (55, 0, 0.09, 0.19),
    (56, 0, 0.10, 0.20),
    (57, 0, 0.00, 0.00),
    // 58/59/73/74 variant 0: cast-on lugs or insulated stainless bars;
    // variant 1: uninterrupted insulation at the floor edge.
    (58, 0, 0.70, 1.05),
    (58, 1, 0.13, 0.23),
    (59, 0, 0.70, 1.05),
    (59, 1, 0.35, 0.53),
    (60, 0, 0.16, 0.26),
    (61, 0, 0.16, 0.26),
    (62, 0, 0.39, 0.59),
    (63, 0, 0.31, 0.47),
    (64, 0, 0.00, 0.00),
    (65, 0, 0.36, 0.54),
    (66, 0, 0.33, 0.50),
    (67, 0, 0.78, 1.17),
    (68, 0, 0.16, 0.26),
    (69, 0, 0.33, 0.50),
    (70, 0, 0.19, 0.29),
    (71, 0, 0.19, 0.29),
    (72, 0, 0.44, 0.66),
    (73, 0, 0.84, 1.26),
    (73, 1, 0.27, 0.41),
    (74, 0, 0.84, 1.26),
    (74, 1, 0.38, 0.57),
];

/// ψ_for for a detail position; `None` when the position/variant does not
/// exist (use [`PSI_DEFAULT`] only for genuinely missing positions).
pub fn forfait_psi(position: u16, variant: u8, column: PsiColumn) -> Option<f64> {
    // Detail 17: 0,06 / 0,09 from 2024 (p. 803), 0,60 / 0,90 in 2023 (p. 804).
    if (position, variant) == (17, 0) {
        let psi = crate::norm_versions::profile().psi_detail_17;
        return Some(match column {
            PsiColumn::A => psi[0],
            PsiColumn::B => psi[1],
        });
    }
    PSI_TABLE
        .iter()
        .find(|(p, v, _, _)| *p == position && *v == variant)
        .map(|(_, _, a, b)| match column {
            PsiColumn::A => *a,
            PsiColumn::B => *b,
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ElementType {
    /// Tilt from vertical at most 15°.
    Facade,
    /// Floors over crawlspaces or on ground (heat flow downward).
    Floor,
    /// Attic floors ("zoldervloeren"): the floor rows of tables I.4/I.5,
    /// with R_si 0,10 for the upward heat flow (table C.2).
    AtticFloor,
    /// Tilt from vertical at least 15°, bordering outdoor air.
    Roof,
    /// Hull of a floating building (table I.7).
    FloatingHull,
}

impl ElementType {
    /// Row of tables I.4–I.7 and formula I.2.
    fn table_row(self) -> Self {
        match self {
            Self::AtticFloor => Self::Floor,
            other => other,
        }
    }

    /// R_ad of formula I.2.
    fn additional_resistance(self) -> f64 {
        match self.table_row() {
            Self::Facade => 0.36,
            Self::Floor | Self::AtticFloor | Self::FloatingHull => 0.15,
            Self::Roof => 0.22,
        }
    }

    fn cavity_resistance(self) -> f64 {
        match self.table_row() {
            Self::Facade => 0.16,
            Self::Roof => 0.13,
            Self::Floor | Self::AtticFloor | Self::FloatingHull => 0.18,
        }
    }

    /// Table C.2 R_si by the usual heat-flow direction.
    fn default_r_si(self) -> f64 {
        match self {
            Self::Facade => 0.13,
            Self::Floor | Self::FloatingHull => 0.17,
            Self::Roof | Self::AtticFloor => 0.10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BuildingKind {
    Regular,
    Caravan,
    Floating {
        /// Berth that is new on 1 January 2018 (table I.7).
        #[serde(rename = "newBerthSince2018")]
        new_berth_since_2018: bool,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InsulationState {
    /// No insulation or unknown whether present.
    AbsentOrUnknown,
    /// (Post-)insulated with an undeterminable thickness.
    PresentUnknownThickness,
    /// I.2.1.4.
    KnownThickness {
        #[serde(rename = "thicknessMm")]
        thickness_mm: f64,
        /// Skip rounding to 10 mm when the product thickness is proven.
        #[serde(default, rename = "thicknessProven")]
        thickness_proven: bool,
        /// Known λ including anchors, moisture and ageing, used only when
        /// above 0,045 W/(m·K).
        #[serde(default, rename = "knownLambdaEquivalent")]
        known_lambda_equivalent: Option<f64>,
        /// Reed thatch thickness in m (I.3).
        #[serde(default, rename = "reedThicknessM")]
        reed_thickness_m: Option<f64>,
        /// Thermal cushions under a floor (+1,8).
        #[serde(default, rename = "thermalCushions")]
        thermal_cushions: bool,
    },
}

/// Renovation or later extension with insulation of undeterminable
/// thickness (ISSO 82.1/75.1 §8.7.2.1, afb. 8.14).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Renovation {
    /// Year of the renovation or extension; `None` when unknown.
    #[serde(default)]
    pub year: Option<i32>,
    /// Evidence (in the project dossier) that the insulation met the R_c
    /// requirement of that year.
    #[serde(default)]
    pub meets_requirements_of_year: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForfaitOpaque {
    pub element: ElementType,
    pub building: BuildingKind,
    pub construction_year: i32,
    pub insulation: InsulationState,
    /// Air cavity present.
    pub cavity: bool,
    /// Override of R_si when the heat flow differs from the usual one.
    #[serde(default)]
    pub r_si_override: Option<f64>,
    /// Bordering an unheated space: R_se becomes the space-side R_si of
    /// table C.2 for the same heat-flow direction (8.4.2.1, p. 266).
    ///
    /// Only for the basisopname route that feeds H_U directly
    /// (`transmission.unheated`). A project construction `uValue` must
    /// carry R_se = 0,04 (C.10, p. 777): the project route applies the
    /// 8.4.2.1 swap itself, so a U from this flag would be corrected twice.
    #[serde(default)]
    pub towards_unheated_space: bool,
    /// Post-insulation at a renovation or in an extension, thickness not
    /// determinable; only with `PresentUnknownThickness`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renovation: Option<Renovation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForfaitOpaqueResult {
    pub r_c: f64,
    pub u_c: f64,
    pub route: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ForfaitIssue {
    pub code: &'static str,
    pub path: String,
}

/// Table I.4.
fn table_i4(element: ElementType, cavity: bool, insulated: bool) -> Option<f64> {
    let (absent, present) = match (element.table_row(), cavity) {
        (ElementType::Facade, true) => (0.35, 0.85),
        (ElementType::Facade, false) => (0.19, 0.69),
        (ElementType::Floor, true) => (0.33, 0.83),
        (ElementType::Floor, false) => (0.15, 0.65),
        (ElementType::Roof, true) => (0.35, 0.85),
        (ElementType::Roof, false) => (0.22, 0.72),
        (ElementType::FloatingHull | ElementType::AtticFloor, _) => return None,
    };
    Some(if insulated { present } else { absent })
}

/// Tables I.5–I.7 for construction years from 1965.
/// Year classes of tables I.5/I.6 (regular buildings, caravans): start year
/// and R_c per class.
fn year_bounds(element: ElementType, building: BuildingKind) -> Option<&'static [(i32, f64)]> {
    const REGULAR_FACADE: &[(i32, f64)] = &[
        (1965, 0.43),
        (1975, 1.3),
        (1983, 1.3),
        (1988, 2.0),
        (1992, 2.5),
        (2014, 3.5),
        (2015, 4.5),
        (2021, 4.7),
    ];
    const REGULAR_FLOOR: &[(i32, f64)] = &[
        (1965, 0.17),
        (1975, 0.52),
        (1983, 1.3),
        (1988, 1.3),
        (1992, 2.5),
        (2014, 3.5),
        (2021, 3.7),
    ];
    const REGULAR_ROOF: &[(i32, f64)] = &[
        (1965, 0.86),
        (1975, 1.3),
        (1983, 1.3),
        (1988, 2.0),
        (1992, 2.5),
        (2014, 3.5),
        (2015, 6.0),
        (2021, 6.3),
    ];
    // 1965–1983: 0,19 for façades (0,04 for panels, see table I.6).
    const CARAVAN_FACADE: &[(i32, f64)] = &[
        (1965, 0.19),
        (1983, 1.3),
        (1992, 2.0),
        (2014, 2.5),
        (2021, 2.6),
    ];
    const CARAVAN_FLOOR: &[(i32, f64)] = &[
        (1965, 0.17),
        (1983, 1.3),
        (1992, 2.0),
        (2014, 2.5),
        (2021, 2.6),
    ];
    const CARAVAN_ROOF: &[(i32, f64)] = &[
        (1965, 0.22),
        (1983, 1.3),
        (1992, 2.0),
        (2014, 2.5),
        (2021, 2.6),
    ];
    match (building, element.table_row()) {
        (BuildingKind::Regular, ElementType::Facade) => Some(REGULAR_FACADE),
        (BuildingKind::Regular, ElementType::Floor) => Some(REGULAR_FLOOR),
        (BuildingKind::Regular, ElementType::Roof) => Some(REGULAR_ROOF),
        (BuildingKind::Caravan, ElementType::Facade) => Some(CARAVAN_FACADE),
        (BuildingKind::Caravan, ElementType::Floor) => Some(CARAVAN_FLOOR),
        (BuildingKind::Caravan, ElementType::Roof) => Some(CARAVAN_ROOF),
        _ => None,
    }
}

/// Index of the year class containing `year`; `None` before 1965.
fn year_class(bounds: &[(i32, f64)], year: i32) -> Option<usize> {
    bounds.iter().rposition(|(from, _)| year >= *from)
}

fn year_table(element: ElementType, building: BuildingKind, year: i32) -> Option<f64> {
    let element = element.table_row();
    let pick = |bounds: &[(i32, f64)]| {
        bounds
            .iter()
            .rev()
            .find(|(from, _)| year >= *from)
            .map(|(_, r)| *r)
    };
    match building {
        BuildingKind::Regular | BuildingKind::Caravan => {
            year_bounds(element, building).and_then(pick)
        }
        BuildingKind::Floating {
            new_berth_since_2018,
        } => {
            let new = new_berth_since_2018 && year >= 2018;
            match element {
                ElementType::Facade => {
                    if new {
                        return Some(if year >= 2021 { 4.7 } else { 4.5 });
                    }
                    pick(&[
                        (1965, 0.43),
                        (1975, 0.43),
                        (1983, 1.3),
                        (1988, 2.0),
                        (1992, 2.5),
                        (2014, 3.5),
                        (2015, 4.5),
                        (2018, 3.5),
                        (2021, 3.7),
                    ])
                }
                ElementType::FloatingHull => {
                    if new {
                        return Some(if year >= 2021 { 3.7 } else { 3.5 });
                    }
                    pick(&[
                        (1965, 0.17),
                        (1992, 2.5),
                        (2015, 3.5),
                        (2018, 2.5),
                        (2021, 2.6),
                    ])
                }
                ElementType::Roof => {
                    if new {
                        return Some(if year >= 2021 { 6.3 } else { 6.0 });
                    }
                    pick(&[
                        (1965, 0.35),
                        (1983, 1.3),
                        (1988, 2.0),
                        (1992, 2.5),
                        (2014, 3.5),
                        (2015, 6.0),
                        (2018, 4.5),
                        (2021, 4.7),
                    ])
                }
                ElementType::Floor | ElementType::AtticFloor => None,
            }
        }
    }
}

impl ForfaitOpaque {
    pub fn validate(&self, path: &str) -> Vec<ForfaitIssue> {
        let mut issues = Vec::new();
        if !(1000..=2100).contains(&self.construction_year) {
            issues.push(ForfaitIssue {
                code: "construction_year_invalid",
                path: format!("{path}.constructionYear"),
            });
        }
        if let InsulationState::KnownThickness {
            thickness_mm,
            known_lambda_equivalent,
            reed_thickness_m,
            thermal_cushions,
            ..
        } = &self.insulation
        {
            if !(thickness_mm.is_finite() && *thickness_mm >= 0.0) {
                issues.push(ForfaitIssue {
                    code: "insulation_thickness_invalid",
                    path: format!("{path}.insulation.thicknessMm"),
                });
            }
            if known_lambda_equivalent.is_some_and(|l| !(l.is_finite() && l > 0.0)) {
                issues.push(ForfaitIssue {
                    code: "lambda_invalid",
                    path: format!("{path}.insulation.knownLambdaEquivalent"),
                });
            }
            if reed_thickness_m.is_some_and(|d| !(0.1..=0.4).contains(&d)) {
                issues.push(ForfaitIssue {
                    code: "reed_thickness_out_of_range",
                    path: format!("{path}.insulation.reedThicknessM"),
                });
            }
            if *thermal_cushions && self.element != ElementType::Floor {
                issues.push(ForfaitIssue {
                    code: "thermal_cushions_floor_only",
                    path: format!("{path}.insulation.thermalCushions"),
                });
            }
            // I.2.1.4: with thermal cushions the floor is R_ad + 1,8 only.
            if *thermal_cushions && *thickness_mm != 0.0 {
                issues.push(ForfaitIssue {
                    code: "thermal_cushions_exclude_insulation_thickness",
                    path: format!("{path}.insulation.thicknessMm"),
                });
            }
        } else if self.forfait_rc_from_tables().is_none() {
            issues.push(ForfaitIssue {
                code: "no_forfait_value_for_element",
                path: path.to_string(),
            });
        }
        if let Some(renovation) = &self.renovation {
            if !matches!(self.insulation, InsulationState::PresentUnknownThickness) {
                issues.push(ForfaitIssue {
                    code: "renovation_requires_present_unknown_thickness",
                    path: format!("{path}.renovation"),
                });
            }
            if year_bounds(self.element, self.building).is_none() {
                issues.push(ForfaitIssue {
                    code: "renovation_year_classes_unavailable",
                    path: format!("{path}.renovation"),
                });
            }
            if renovation
                .year
                .is_some_and(|year| !(self.construction_year..=2100).contains(&year))
            {
                issues.push(ForfaitIssue {
                    code: "renovation_year_invalid",
                    path: format!("{path}.renovation.year"),
                });
            }
        }
        if self
            .r_si_override
            .is_some_and(|r| ![0.10, 0.13, 0.17].iter().any(|v| (v - r).abs() < 1e-9))
        {
            issues.push(ForfaitIssue {
                code: "r_si_not_in_table_c2",
                path: format!("{path}.rSiOverride"),
            });
        }
        issues
    }

    fn forfait_rc_from_tables(&self) -> Option<f64> {
        let insulated = matches!(self.insulation, InsulationState::PresentUnknownThickness);
        if let (true, Some(renovation)) = (insulated, &self.renovation) {
            return self.renovated_rc(renovation);
        }
        if self.construction_year < 1965 {
            // Table I.4 has no hull row: a houseboat hull before 1965 takes
            // the first class of table I.7 (interpretation).
            table_i4(self.element, self.cavity, insulated).or_else(|| {
                (self.element == ElementType::FloatingHull)
                    .then(|| year_table(self.element, self.building, 1965))
                    .flatten()
            })
        } else {
            year_table(self.element, self.building, self.construction_year)
        }
    }

    /// §8.7.2.1 / afb. 8.14 (ISSO 82.1 p. 84–85, 75.1 p. 88–89):
    /// - year known, evidence of that year's requirement: its year class;
    /// - year known, no evidence: the class before it, at most "1992 tot
    ///   2014" (R_c 2,5);
    /// - year unknown: the class after the original one; before 1965 the
    ///   "(na)geïsoleerd" column of table I.4.
    fn renovated_rc(&self, renovation: &Renovation) -> Option<f64> {
        let bounds = year_bounds(self.element, self.building)?;
        let insulated_pre_1965 = || table_i4(self.element, self.cavity, true);
        match renovation.year {
            Some(year) if renovation.meets_requirements_of_year => match year_class(bounds, year) {
                Some(index) => Some(bounds[index].1),
                None => insulated_pre_1965(),
            },
            Some(year) => match year_class(bounds, year) {
                Some(index) if index > 0 => Some(bounds[index - 1].1.min(2.5)),
                _ => insulated_pre_1965(),
            },
            None => match year_class(bounds, self.construction_year) {
                Some(index) => Some(bounds[(index + 1).min(bounds.len() - 1)].1),
                None => insulated_pre_1965(),
            },
        }
    }

    /// Call after `validate`.
    pub fn calculate(&self) -> ForfaitOpaqueResult {
        let (r_c, route) = match &self.insulation {
            InsulationState::KnownThickness {
                thickness_mm,
                thickness_proven,
                known_lambda_equivalent,
                reed_thickness_m,
                thermal_cushions,
            } => {
                let d_mm = if *thickness_proven {
                    *thickness_mm
                } else {
                    (thickness_mm / 10.0).round() * 10.0
                };
                let lambda = known_lambda_equivalent.unwrap_or(0.045).max(0.045);
                let mut r = d_mm / 1000.0 / lambda + self.element.additional_resistance();
                if self.cavity && d_mm <= 30.0 {
                    r += self.element.cavity_resistance();
                }
                if let Some(reed) = reed_thickness_m {
                    let d = ((reed / 0.05) + 1e-9).floor() * 0.05;
                    // (I.3): d/0,105 in 2025+C1 (p. 833), d/0,2 in 2024
                    // (p. 814).
                    r += d / crate::norm_versions::profile().reed_thatch_divisor;
                }
                if *thermal_cushions {
                    r += 1.8;
                }
                (r, "I.2.1.4")
            }
            InsulationState::PresentUnknownThickness if self.renovation.is_some() => {
                (self.forfait_rc_from_tables().unwrap_or(f64::NAN), "8.7.2.1")
            }
            _ => (
                self.forfait_rc_from_tables().unwrap_or(f64::NAN),
                if self.construction_year < 1965 {
                    "I.2.1.2"
                } else {
                    "I.2.1.3"
                },
            ),
        };
        let r_si = self
            .r_si_override
            .unwrap_or_else(|| self.element.default_r_si());
        // 8.4.2.1: towards an unheated space R_se is the space-side R_si.
        let r_se = if self.towards_unheated_space {
            r_si
        } else {
            0.04
        };
        // I.1, rounded to two decimals.
        let u_c = round_half_up(1.0 / (r_c + r_si + r_se), 2);
        ForfaitOpaqueResult { r_c, u_c, route }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForfaitGlass {
    TripleHr,
    HrPlusPlus,
    HrPlus,
    HrCoatedDouble,
    DoubleUncoated,
    SecondaryWindow,
    Single,
}

/// Tables I.8 (to outdoor air) and I.9 (not to outdoor air).
pub fn forfait_window_u(glass: ForfaitGlass, frame: FrameGroup, exterior: bool) -> f64 {
    let row: [f64; 3] = match (glass, exterior) {
        (ForfaitGlass::TripleHr, true) => [1.4, 1.9, 2.7],
        (ForfaitGlass::HrPlusPlus, true) => [1.8, 2.3, 3.1],
        (ForfaitGlass::HrPlus, true) => [2.0, 2.5, 3.3],
        (ForfaitGlass::HrCoatedDouble, true) => [2.3, 2.8, 3.6],
        (ForfaitGlass::DoubleUncoated | ForfaitGlass::SecondaryWindow, true) => [2.9, 3.3, 4.1],
        (ForfaitGlass::Single, true) => [5.1, 5.4, 6.2],
        (ForfaitGlass::TripleHr, false) => [1.3, 1.6, 2.2],
        (ForfaitGlass::HrPlusPlus, false) => [1.5, 1.9, 2.4],
        (ForfaitGlass::HrPlus, false) => [1.7, 2.1, 2.6],
        (ForfaitGlass::HrCoatedDouble, false) => [1.9, 2.2, 2.7],
        (ForfaitGlass::DoubleUncoated | ForfaitGlass::SecondaryWindow, false) => [2.3, 2.5, 3.0],
        (ForfaitGlass::Single, false) => [3.5, 3.6, 4.0],
    };
    row[frame_index(frame)]
}

fn frame_index(frame: FrameGroup) -> usize {
    match frame {
        FrameGroup::WoodOrPlastic => 0,
        FrameGroup::MetalWithThermalBreak => 1,
        FrameGroup::MetalWithoutThermalBreak => 2,
    }
}

/// Table I.10.
pub fn forfait_door_u(insulated: bool, exterior: bool) -> f64 {
    match (insulated, exterior) {
        (true, true) => 2.0,
        (true, false) => 1.7,
        (false, true) => 3.4,
        (false, false) => 2.7,
    }
}

/// I.2.2.1/I.2.2.2: a door with less than 65 % glazing combines the window
/// value for the glazed share with the door value for the rest.
pub fn forfait_glazed_door_u(
    glass_fraction: f64,
    glass: ForfaitGlass,
    frame: FrameGroup,
    door_insulated: bool,
    exterior: bool,
) -> f64 {
    let window = forfait_window_u(glass, frame, exterior);
    if glass_fraction >= 0.65 {
        window
    } else {
        glass_fraction * window + (1.0 - glass_fraction) * forfait_door_u(door_insulated, exterior)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PanelInsulation {
    AbsentOrUnknown,
    PresentUnknownThickness,
    KnownThickness {
        #[serde(rename = "thicknessMm")]
        thickness_mm: f64,
    },
}

/// NTA 8800:2023 tables I.13/I.14 (p. 819–820): forfait U of a panel in a
/// frame for a building (part) from 1965 whose data are missing, by
/// build-year class; `None` before 1965 (tables I.11/I.12 apply).
pub fn panel_u_by_build_year(build_year: i32, frame: FrameGroup, exterior: bool) -> Option<f64> {
    let row: [f64; 3] = match (exterior, build_year) {
        (_, ..=1964) => return None,
        (true, 1965..=1991) => [3.7, 4.1, 4.9],
        (true, 1992..=2012) => [3.7, 4.1, 4.2],
        (true, _) => [1.65, 1.65, 1.65],
        (false, 1965..=1991) => [2.8, 3.0, 3.4],
        (false, 1992..=2012) => [2.8, 3.0, 3.0],
        (false, _) => [1.4, 1.4, 1.4],
    };
    Some(row[frame_index(frame)])
}

/// Forfait panel U in the active edition: NTA 8800:2023 (I.2.2.4.1–2,
/// p. 818–820) takes tables I.13/I.14 for a building from 1965 without a
/// known insulation thickness; otherwise [`forfait_panel_u`].
pub fn forfait_panel_u_in_edition(
    insulation: PanelInsulation,
    cavity: bool,
    frame: FrameGroup,
    exterior: bool,
    build_year: Option<i32>,
) -> Option<f64> {
    if crate::norm_versions::profile().panel_build_year_tables
        && !matches!(insulation, PanelInsulation::KnownThickness { .. })
    {
        if let Some(u) = build_year.and_then(|year| panel_u_by_build_year(year, frame, exterior)) {
            return Some(u);
        }
    }
    forfait_panel_u(insulation, cavity, frame, exterior)
}

/// Tables I.11/I.12 and I.15/I.16 (thickness rounded to 10 mm, 10–300 mm).
pub fn forfait_panel_u(
    insulation: PanelInsulation,
    cavity: bool,
    frame: FrameGroup,
    exterior: bool,
) -> Option<f64> {
    let f = frame_index(frame);
    match insulation {
        PanelInsulation::AbsentOrUnknown | PanelInsulation::PresentUnknownThickness => {
            let insulated = matches!(insulation, PanelInsulation::PresentUnknownThickness);
            let row: [f64; 3] = match (exterior, insulated, cavity) {
                (true, false, false) => [3.7, 4.1, 4.9],
                (true, false, true) => [2.5, 2.8, 3.6],
                (true, true, false) => [1.5, 1.9, 2.7],
                (true, true, true) => [1.4, 1.7, 2.5],
                (false, false, false) => [2.8, 3.0, 3.4],
                (false, false, true) => [2.0, 2.3, 2.7],
                (false, true, false) => [1.3, 1.6, 2.2],
                (false, true, true) => [1.2, 1.5, 2.1],
            };
            Some(row[f])
        }
        PanelInsulation::KnownThickness { thickness_mm } => {
            let d = ((thickness_mm / 10.0).round() * 10.0) as i32;
            if d > 300 && exterior {
                return Some(panel_formula_u(f64::from(d) / 1000.0, frame));
            }
            if !(10..=300).contains(&d) {
                return None;
            }
            // (d, without cavity, with cavity) for d ≤ 30; one row above.
            const OUT_THIN: [(i32, [f64; 3], [f64; 3]); 3] = [
                (10, [2.0, 2.4, 3.2], [1.7, 2.0, 2.8]),
                (20, [1.5, 1.9, 2.7], [1.4, 1.7, 2.5]),
                (30, [1.3, 1.6, 2.4], [1.2, 1.5, 2.3]),
            ];
            const IN_THIN: [(i32, [f64; 3], [f64; 3]); 3] = [
                (10, [1.7, 2.0, 2.5], [1.5, 1.7, 2.3]),
                (20, [1.3, 1.6, 2.2], [1.2, 1.5, 2.1]),
                (30, [1.2, 1.4, 2.0], [1.1, 1.4, 1.9]),
            ];
            const OUT: [[f64; 3]; 27] = [
                [1.1, 1.5, 2.3],
                [1.0, 1.4, 2.2],
                [0.98, 1.3, 2.1],
                [0.93, 1.3, 2.1],
                [0.90, 1.2, 2.0],
                [0.87, 1.2, 2.0],
                [0.84, 1.2, 2.0],
                [0.82, 1.2, 2.0],
                [0.80, 1.2, 2.0],
                [0.79, 1.1, 1.9],
                [0.78, 1.1, 1.9],
                [0.77, 1.1, 1.9],
                [0.76, 1.1, 1.9],
                [0.75, 1.1, 1.9],
                [0.74, 1.1, 1.9],
                [0.73, 1.1, 1.9],
                [0.73, 1.1, 1.9],
                [0.72, 1.1, 1.9],
                [0.71, 1.1, 1.9],
                [0.71, 1.1, 1.9],
                [0.71, 1.1, 1.9],
                [0.70, 1.1, 1.9],
                [0.70, 1.0, 1.8],
                [0.69, 1.0, 1.8],
                [0.69, 1.0, 1.8],
                [0.69, 1.0, 1.8],
                [0.69, 1.0, 1.8],
            ];
            const IN: [[f64; 3]; 27] = [
                [1.0, 1.3, 1.9],
                [1.0, 1.2, 1.8],
                [0.9, 1.2, 1.8],
                [0.9, 1.2, 1.8],
                [0.8, 1.1, 1.7],
                [0.8, 1.1, 1.7],
                [0.8, 1.1, 1.7],
                [0.8, 1.1, 1.7],
                [0.8, 1.0, 1.7],
                [0.7, 1.0, 1.7],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.7, 1.0, 1.6],
                [0.6, 0.9, 1.6],
                [0.6, 0.9, 1.6],
            ];
            if d <= 30 {
                let rows = if exterior { &OUT_THIN } else { &IN_THIN };
                let row = rows.iter().find(|r| r.0 == d)?;
                Some(if cavity { row.2[f] } else { row.1[f] })
            } else {
                let rows = if exterior { &OUT } else { &IN };
                Some(rows[((d - 40) / 10) as usize][f])
            }
        }
    }
}

/// I.2.2.4.2 behind table I.15: R_c from I.4 (λ 0,035, R_ad 0,07), U_p
/// from I.1, 25 % frame with U_fr;for (I.5–I.7), ψ = 0; unrounded.
pub fn panel_formula_u(insulation_m: f64, frame: FrameGroup) -> f64 {
    let r_c = insulation_m / 0.035 + 0.07;
    let u_panel = 1.0 / (r_c + 0.13 + 0.04);
    let u_frame = match frame {
        FrameGroup::WoodOrPlastic => 2.4,
        FrameGroup::MetalWithThermalBreak => 3.8,
        FrameGroup::MetalWithoutThermalBreak => 7.0,
    };
    0.75 * u_panel + 0.25 * u_frame
}

/// I.8: H_ue = 5·A_T;iu in the basic survey of ISSO 82.1/75.1.
pub fn basic_survey_unheated_transfer(separation_area_m2: f64) -> f64 {
    5.0 * separation_area_m2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(year: i32, insulation: InsulationState) -> ForfaitOpaque {
        ForfaitOpaque {
            element: ElementType::Facade,
            building: BuildingKind::Regular,
            construction_year: year,
            insulation,
            cavity: true,
            r_si_override: None,
            towards_unheated_space: false,
            renovation: None,
        }
    }

    #[test]
    fn renovation_follows_afb_8_14() {
        let renovated = |year: Option<i32>, evidence: bool| {
            let mut facade = element(1978, InsulationState::PresentUnknownThickness);
            facade.renovation = Some(Renovation {
                year,
                meets_requirements_of_year: evidence,
            });
            assert!(facade.validate("f").is_empty());
            facade.calculate()
        };
        // Without renovation the 1975–1983 class: 1,3.
        let plain = element(1978, InsulationState::PresentUnknownThickness).calculate();
        assert_eq!(plain.r_c, 1.3);
        // Year unknown: the class after 1975–1983 (1983–1988): 1,3;
        // from 1983 the next class is 1988–1992: 2,0.
        assert_eq!(renovated(None, false).r_c, 1.3);
        let mut later = element(1985, InsulationState::PresentUnknownThickness);
        later.renovation = Some(Renovation {
            year: None,
            meets_requirements_of_year: false,
        });
        assert_eq!(later.calculate().r_c, 2.0);
        assert_eq!(later.calculate().route, "8.7.2.1");
        // 1995 with evidence: 1992–2014 (2,5); without: 1988–1992 (2,0).
        assert_eq!(renovated(Some(1995), true).r_c, 2.5);
        assert_eq!(renovated(Some(1995), false).r_c, 2.0);
        // Without evidence at most "1992 tot 2014": 2,5.
        assert_eq!(renovated(Some(2022), false).r_c, 2.5);
        assert_eq!(renovated(Some(2022), true).r_c, 4.7);
        // Before 1965 with unknown year: the "(na)geïsoleerd" column.
        let mut old = element(1950, InsulationState::PresentUnknownThickness);
        old.renovation = Some(Renovation {
            year: None,
            meets_requirements_of_year: false,
        });
        assert_eq!(old.calculate().r_c, 0.85);
        // Renovation needs "present, thickness unknown" and a year from
        // the construction year on.
        let mut wrong = element(1978, InsulationState::AbsentOrUnknown);
        wrong.renovation = Some(Renovation {
            year: Some(1970),
            meets_requirements_of_year: true,
        });
        let codes: Vec<_> = wrong.validate("f").iter().map(|i| i.code).collect();
        assert!(codes.contains(&"renovation_requires_present_unknown_thickness"));
        assert!(codes.contains(&"renovation_year_invalid"));
    }

    #[test]
    fn unheated_space_side_uses_r_si_for_r_se() {
        let mut wall = element(1970, InsulationState::AbsentOrUnknown);
        let outdoor = wall.calculate().u_c;
        wall.towards_unheated_space = true;
        let unheated = wall.calculate().u_c;
        // 8.4.2.1: R_se 0,04 becomes the horizontal R_si 0,13.
        assert_eq!(outdoor, round_half_up(1.0 / (0.43 + 0.13 + 0.04), 2));
        assert_eq!(unheated, round_half_up(1.0 / (0.43 + 0.13 + 0.13), 2));
    }

    #[test]
    fn attic_floors_use_upward_r_si_and_cushions_exclude_thickness() {
        let mut attic = element(1970, InsulationState::AbsentOrUnknown);
        attic.element = ElementType::AtticFloor;
        let r = attic.calculate();
        // Floor row of table I.5 (0,17) with R_si 0,10.
        assert_eq!(r.r_c, 0.17);
        assert_eq!(r.u_c, round_half_up(1.0 / (0.17 + 0.10 + 0.04), 2));
        let mut floor = element(
            1970,
            InsulationState::KnownThickness {
                thickness_mm: 50.0,
                thickness_proven: false,
                known_lambda_equivalent: None,
                reed_thickness_m: None,
                thermal_cushions: true,
            },
        );
        floor.element = ElementType::Floor;
        floor.cavity = false;
        assert!(floor
            .validate("f")
            .iter()
            .any(|i| i.code == "thermal_cushions_exclude_insulation_thickness"));
        floor.insulation = InsulationState::KnownThickness {
            thickness_mm: 0.0,
            thickness_proven: false,
            known_lambda_equivalent: None,
            reed_thickness_m: None,
            thermal_cushions: true,
        };
        assert!(floor.validate("f").is_empty());
        assert!((floor.calculate().r_c - (0.15 + 1.8)).abs() < 1e-12);
    }

    #[test]
    fn decision_tree_figure_i4() {
        let old = element(1950, InsulationState::AbsentOrUnknown).calculate();
        assert_eq!((old.r_c, old.route), (0.35, "I.2.1.2"));
        assert_eq!(old.u_c, round_half_up(1.0 / (0.35 + 0.17), 2));
        let old_insulated = element(1950, InsulationState::PresentUnknownThickness).calculate();
        assert_eq!(old_insulated.r_c, 0.85);
        assert_eq!(
            element(1990, InsulationState::AbsentOrUnknown)
                .calculate()
                .r_c,
            2.0
        );
        assert_eq!(
            element(2014, InsulationState::AbsentOrUnknown)
                .calculate()
                .r_c,
            3.5
        );
        assert_eq!(
            element(2022, InsulationState::PresentUnknownThickness)
                .calculate()
                .r_c,
            4.7
        );
        let known = element(
            1970,
            InsulationState::KnownThickness {
                thickness_mm: 54.0,
                thickness_proven: false,
                known_lambda_equivalent: None,
                reed_thickness_m: None,
                thermal_cushions: false,
            },
        )
        .calculate();
        // 50 mm / 0,045 + 0,36.
        assert!((known.r_c - (0.05 / 0.045 + 0.36)).abs() < 1e-12);
        let thin = element(
            1970,
            InsulationState::KnownThickness {
                thickness_mm: 20.0,
                thickness_proven: true,
                known_lambda_equivalent: Some(0.05),
                reed_thickness_m: None,
                thermal_cushions: false,
            },
        )
        .calculate();
        assert!((thin.r_c - (0.02 / 0.05 + 0.36 + 0.16)).abs() < 1e-12);
    }

    #[test]
    fn reed_thatch_divisor_follows_the_edition() {
        let mut roof = element(
            1950,
            InsulationState::KnownThickness {
                thickness_mm: 0.0,
                thickness_proven: true,
                known_lambda_equivalent: None,
                reed_thickness_m: Some(0.30),
                thermal_cushions: false,
            },
        );
        roof.element = ElementType::Roof;
        roof.cavity = false;
        // (I.3): d/0,105 in 2025+C1 (p. 833), d/0,2 in 2024 (p. 814).
        assert!((roof.calculate().r_c - (0.22 + 0.30 / 0.105)).abs() < 1e-12);
        let legacy =
            crate::norm_versions::with_version(crate::norm_versions::NormVersion::V2024, || {
                roof.calculate().r_c
            });
        assert!((legacy - (0.22 + 0.30 / 0.2)).abs() < 1e-12);
    }

    #[test]
    fn special_buildings_and_extras() {
        let mut hull = element(2019, InsulationState::AbsentOrUnknown);
        hull.element = ElementType::FloatingHull;
        hull.building = BuildingKind::Floating {
            new_berth_since_2018: true,
        };
        assert_eq!(hull.calculate().r_c, 3.5);
        hull.building = BuildingKind::Floating {
            new_berth_since_2018: false,
        };
        assert_eq!(hull.calculate().r_c, 2.5);
        let mut caravan = element(1980, InsulationState::AbsentOrUnknown);
        caravan.building = BuildingKind::Caravan;
        caravan.element = ElementType::Roof;
        assert_eq!(caravan.calculate().r_c, 0.22);
        let mut roof = element(
            1950,
            InsulationState::KnownThickness {
                thickness_mm: 0.0,
                thickness_proven: true,
                known_lambda_equivalent: None,
                reed_thickness_m: Some(0.27),
                thermal_cushions: false,
            },
        );
        roof.element = ElementType::Roof;
        roof.cavity = false;
        assert!((roof.calculate().r_c - (0.22 + 0.25 / 0.105)).abs() < 1e-12);
        // Before 1965 table I.4 has no hull row: the first class of I.7.
        let mut old_hull = element(1960, InsulationState::AbsentOrUnknown);
        old_hull.element = ElementType::FloatingHull;
        old_hull.building = BuildingKind::Floating {
            new_berth_since_2018: false,
        };
        assert_eq!(old_hull.calculate().r_c, 0.17);
        let mut hull_regular = element(1990, InsulationState::AbsentOrUnknown);
        hull_regular.element = ElementType::FloatingHull;
        let codes: Vec<_> = hull_regular
            .validate("e")
            .into_iter()
            .map(|i| i.code)
            .collect();
        assert!(codes.contains(&"no_forfait_value_for_element"));
    }

    #[test]
    fn windows_doors_panels_and_psi() {
        assert_eq!(
            forfait_window_u(ForfaitGlass::HrPlusPlus, FrameGroup::WoodOrPlastic, true),
            1.8
        );
        assert_eq!(
            forfait_window_u(
                ForfaitGlass::Single,
                FrameGroup::MetalWithoutThermalBreak,
                false
            ),
            4.0
        );
        assert_eq!(forfait_door_u(false, true), 3.4);
        assert!(
            (forfait_glazed_door_u(
                0.4,
                ForfaitGlass::HrPlusPlus,
                FrameGroup::WoodOrPlastic,
                false,
                true
            ) - (0.4 * 1.8 + 0.6 * 3.4))
                .abs()
                < 1e-12
        );
        assert_eq!(
            forfait_panel_u(
                PanelInsulation::AbsentOrUnknown,
                false,
                FrameGroup::WoodOrPlastic,
                true
            ),
            Some(3.7)
        );
        assert_eq!(
            forfait_panel_u(
                PanelInsulation::KnownThickness {
                    thickness_mm: 100.0
                },
                false,
                FrameGroup::WoodOrPlastic,
                true
            ),
            Some(0.84)
        );
        // Formula check of I.4/I.1 with 25 % frame at 2,4 for 100 mm.
        let u_panel: f64 = 1.0 / (0.1 / 0.035 + 0.07 + 0.17);
        assert!((0.75 * u_panel + 0.25 * 2.4 - 0.84).abs() < 0.005);
        assert_eq!(
            forfait_panel_u(
                PanelInsulation::KnownThickness { thickness_mm: 20.0 },
                true,
                FrameGroup::MetalWithThermalBreak,
                false
            ),
            Some(1.5)
        );
        assert_eq!(
            forfait_panel_u(
                PanelInsulation::KnownThickness {
                    thickness_mm: 400.0
                },
                false,
                FrameGroup::WoodOrPlastic,
                true
            )
            .map(crate::window_u::round_transparent),
            Some(0.66)
        );
        // The formula reproduces the last row of table I.15.
        for (frame, table) in [
            (FrameGroup::WoodOrPlastic, 0.69),
            (FrameGroup::MetalWithThermalBreak, 1.0),
            (FrameGroup::MetalWithoutThermalBreak, 1.8),
        ] {
            assert_eq!(
                crate::window_u::round_transparent(panel_formula_u(0.3, frame)),
                table
            );
        }
        // Table I.16 (not to outdoor air) is not extended by the formula.
        assert_eq!(
            forfait_panel_u(
                PanelInsulation::KnownThickness {
                    thickness_mm: 400.0
                },
                false,
                FrameGroup::WoodOrPlastic,
                false
            ),
            None
        );
        assert_eq!(forfait_psi(24, 1, PsiColumn::B), Some(0.62));
        assert_eq!(forfait_psi(58, 1, PsiColumn::A), Some(0.13));
        assert_eq!(forfait_psi(30, 0, PsiColumn::A), None);
        assert_eq!(basic_survey_unheated_transfer(12.0), 60.0);
    }
}
