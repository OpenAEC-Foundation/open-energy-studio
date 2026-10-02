//! Design values of thermal conductivity and resistance of building
//! materials, NTA 8800:2025+C1:2026 annex E (pp. 788–801) and annex H
//! (p. 820).
//!
//! - Insulation and reflective foils: λ_calc = λ_D·F_T·F_M·F_A·F_conv (E.3)
//!   and R_calc = R_D/(F_T·F_M·F_A·F_conv) (E.4), with F_T (E.7), F_M
//!   (tables E.2–E.4), F_A (E.10, table E.5).
//! - Masonry: area-weighted joints and units (E.5) or tables E.14–E.17.
//! - Other materials: λ_calc = λ_D·F_MA (E.6, table E.1).
//! - Forfait values: tables E.10 (existing buildings), E.11, E.12, E.13 and
//!   table H.1 for window and frame materials.
//! - Rounding of λ (up) and R (down) per the NEN-EN-ISO 10456 rules quoted
//!   in E.2.1.1.
//!
//! The norm text is not reproduced here; values are transcribed from the
//! licensed edition with page references.

use serde::{Deserialize, Serialize};

/// E.2.1.1: round λ up to 0,001 / 0,005 / 0,01 / 0,1 W/(m·K).
pub fn round_lambda_up(lambda: f64) -> f64 {
    let step = if lambda <= 0.08 {
        0.001
    } else if lambda <= 0.20 {
        0.005
    } else if lambda <= 2.00 {
        0.01
    } else {
        0.1
    };
    // Guard against representation error (0.035 / 0.001 = 35.000000000000004).
    ((lambda / step) - 1e-9).ceil() * step
}

/// E.2.1.1: round R down to at most two decimals or three significant
/// figures.
pub fn round_resistance_down(resistance: f64) -> f64 {
    let step = if resistance >= 10.0 { 0.1 } else { 0.01 };
    ((resistance / step) + 1e-9).floor() * step
}

/// Arithmetic rounding to `decimals` (half away from zero), as required for
/// presented U- and R_c-values (8.2.2.1, C.1.2).
pub fn round_half_up(value: f64, decimals: i32) -> f64 {
    let factor = 10f64.powi(decimals);
    let scaled = value * factor;
    (scaled + 1e-9 * scaled.signum()).round() / factor
}

/// Moisture conditions of tables E.2 (insulation); regular Dutch practice
/// has F_M = 1,00 (E.2.1.3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InsulationMoisture {
    Regular,
    PerimeterGeneral,
    PerimeterXpsFullyBonded,
    PerimeterXpsPointBonded,
    InvertedRoofGeneral,
    InvertedRoofXpsSlopeUpTo1Percent,
    InvertedRoofXpsSlopeAbove1Percent,
    InvertedRoofXpsGreenRoof,
}

impl InsulationMoisture {
    /// Table E.2.
    pub fn factor(self) -> f64 {
        match self {
            Self::Regular | Self::PerimeterXpsFullyBonded => 1.00,
            Self::PerimeterGeneral | Self::InvertedRoofGeneral => 1.25,
            Self::PerimeterXpsPointBonded | Self::InvertedRoofXpsSlopeAbove1Percent => 1.02,
            Self::InvertedRoofXpsSlopeUpTo1Percent => 1.04,
            Self::InvertedRoofXpsGreenRoof => 1.07,
        }
    }
}

/// Table E.5 product groups for in-situ insulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InSituProduct {
    /// MW flakes, WW, ICB, WF, LFCI, plant or animal fibres, straw bales.
    FibresAndFlakes,
    EpsBeads,
    Polyurethane,
    UreaFormaldehyde,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InSituSituation {
    /// Construction designed for in-situ insulation, new-build like.
    A,
    /// Existing construction with obstacles or defects.
    B,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Ageing {
    /// Factory-made products: F_A = 1.
    FactoryMade,
    /// E.10 with table E.5.
    InSitu {
        product: InSituProduct,
        situation: InSituSituation,
        /// Footnote b: PU with a practice-oriented test, F_A;iso = 1,05.
        #[serde(default, rename = "practiceTested")]
        practice_tested: bool,
    },
}

impl Ageing {
    pub fn factor(&self) -> f64 {
        match self {
            Self::FactoryMade => 1.0,
            Self::InSitu {
                product,
                situation,
                practice_tested,
            } => {
                let iso = match product {
                    InSituProduct::FibresAndFlakes => 1.00,
                    InSituProduct::EpsBeads => 1.05,
                    InSituProduct::Polyurethane if *practice_tested => 1.05,
                    InSituProduct::Polyurethane => 1.10,
                    InSituProduct::UreaFormaldehyde => 1.25,
                    InSituProduct::Other => 1.30,
                };
                let appl = match situation {
                    InSituSituation::A => 1.00,
                    InSituSituation::B => 1.15,
                };
                iso * appl
            }
        }
    }
}

/// E.7: F_T from the mean material temperature; 1,00 between 5 and 20 °C.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemperatureConversion {
    pub mean_temperature_c: f64,
    /// f_T from annex A of NEN-EN-ISO 10456, 1/K.
    pub conversion_coefficient: f64,
    pub source_reference: String,
}

impl TemperatureConversion {
    pub fn factor(&self) -> f64 {
        if self.mean_temperature_c > 5.0 && self.mean_temperature_c < 20.0 {
            1.0
        } else {
            (self.conversion_coefficient * (self.mean_temperature_c - 10.0)).exp()
        }
    }
}

/// Table E.10 (existing-building column λ_for), E.11 and E.12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForfaitMaterial {
    GlassWool,
    StoneWool,
    MineralWoolFlakes,
    EpsBoard,
    EpsWhiteBeads,
    EpsGreyBeads,
    Xps,
    PurPirBoard,
    PurSprayedOpenCell,
    PurSprayedClosedCell,
    PhenolicFoam,
    CellularGlass,
    WoodWool,
    ExpandedPerlite,
    ExpandedCork,
    WoodFibre,
    UreaFormaldehyde,
    CelluloseLoose,
    FlaxBoard,
    FlaxSprayed,
    SheepWool,
    Cotton,
    Coconut,
    Straw,
    HempBoard,
    HempOther,
    Peat,
    Miscanthus,
    GrassWool,
    Reed,
    CementBoundPerlite,
    CementBoundEps,
    Shells,
    ClayGranules,
}

impl ForfaitMaterial {
    /// λ_for in W/(m·K), before the E.2.1 conversion factors.
    pub fn lambda(self) -> f64 {
        use ForfaitMaterial::*;
        match self {
            GlassWool | StoneWool | EpsBoard | Xps => 0.040,
            MineralWoolFlakes | EpsWhiteBeads | CellularGlass | WoodFibre | CelluloseLoose => 0.045,
            EpsGreyBeads | PurSprayedClosedCell => 0.035,
            PurPirBoard | PhenolicFoam => 0.030,
            PurSprayedOpenCell => 0.045,
            WoodWool => 0.100,
            ExpandedPerlite | ExpandedCork => 0.050,
            UreaFormaldehyde => 0.060,
            FlaxBoard | SheepWool | Cotton | HempBoard | GrassWool => 0.045,
            FlaxSprayed | Coconut => 0.050,
            Straw => 0.055,
            HempOther | Miscanthus => 0.060,
            Peat | Reed => 0.100,
            CementBoundPerlite | CementBoundEps => 0.150,
            Shells => 0.200,
            ClayGranules => 0.300,
        }
    }

    /// E.2.2.1 note 1: insulation materials have λ ≤ 0,100 W/(m·K).
    pub fn is_insulation(self) -> bool {
        self.lambda() <= 0.100
    }
}

/// Table E.1 material classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OtherMaterialClass {
    Inorganic,
    Glass,
    Organic,
    Plastic,
}

/// Table E.1 F_MA. Densities between rows take the row with the nearest
/// density, the higher factor on a tie.
pub fn moisture_ageing_factor(class: OtherMaterialClass, density_kg_m3: f64) -> f64 {
    let rows: &[(f64, f64)] = match class {
        OtherMaterialClass::Inorganic => &[
            (2500.0, 1.25),
            (2300.0, 1.25),
            (2100.0, 1.25),
            (1900.0, 1.25),
            (1600.0, 1.30),
            (1300.0, 1.30),
            (1000.0, 1.35),
            (700.0, 1.40),
            (400.0, 1.40),
        ],
        OtherMaterialClass::Glass => &[(2500.0, 1.00)],
        OtherMaterialClass::Organic => {
            &[(1000.0, 1.20), (700.0, 1.20), (500.0, 1.20), (400.0, 1.25)]
        }
        OtherMaterialClass::Plastic => &[(1500.0, 1.00)],
    };
    if class == OtherMaterialClass::Organic && density_kg_m3 <= 400.0 {
        return 1.25;
    }
    let nearest = rows
        .iter()
        .map(|(d, _)| (d - density_kg_m3).abs())
        .fold(f64::INFINITY, f64::min);
    rows.iter()
        .filter(|(d, _)| ((d - density_kg_m3).abs() - nearest).abs() < 1e-9)
        .map(|(_, f)| *f)
        .fold(f64::NEG_INFINITY, f64::max)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MasonryKind {
    Brick,
    Concrete,
    CalciumSilicate,
    AeratedConcrete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MasonryEnvironment {
    DryIndoor,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MasonryJoints {
    /// Glued joints ≤ 3 mm.
    Glued,
    Mortar,
}

/// Tables E.14–E.17: λ_calc for masonry including joints. Densities between
/// rows take the next higher tabulated density (conservative); densities
/// outside the table return `None`.
pub fn masonry_table_lambda(
    kind: MasonryKind,
    density_kg_m3: f64,
    environment: MasonryEnvironment,
    joints: MasonryJoints,
) -> Option<f64> {
    // (ρ, glued dry, glued other, mortar dry, mortar other)
    const BRICK: [(f64, f64, f64, f64, f64); 19] = [
        (600.0, 0.160, 0.300, 0.190, 0.320),
        (700.0, 0.190, 0.360, 0.220, 0.380),
        (800.0, 0.220, 0.420, 0.260, 0.440),
        (900.0, 0.260, 0.480, 0.300, 0.500),
        (1000.0, 0.290, 0.540, 0.340, 0.570),
        (1100.0, 0.320, 0.600, 0.380, 0.630),
        (1200.0, 0.350, 0.660, 0.420, 0.700),
        (1300.0, 0.390, 0.720, 0.460, 0.760),
        (1400.0, 0.430, 0.800, 0.510, 0.850),
        (1500.0, 0.460, 0.860, 0.540, 0.910),
        (1600.0, 0.500, 0.940, 0.600, 0.990),
        (1700.0, 0.550, 1.030, 0.650, 1.080),
        (1800.0, 0.590, 1.110, 0.700, 1.160),
        (1900.0, 0.640, 1.210, 0.760, 1.270),
        (2000.0, 0.680, 1.290, 0.810, 1.350),
        (2100.0, 0.740, 1.390, 0.870, 1.460),
        (2200.0, 0.790, 1.490, 0.940, 1.560),
        (2300.0, 0.850, 1.590, 1.000, 1.670),
        (2400.0, 0.900, 1.690, 1.070, 1.780),
    ];
    // (ρ, dry, other)
    const CONCRETE: [(f64, f64, f64); 9] = [
        (1600.0, 1.033, 1.164),
        (1700.0, 1.091, 1.231),
        (1800.0, 1.185, 1.336),
        (1900.0, 1.279, 1.442),
        (2000.0, 1.396, 1.575),
        (2100.0, 1.526, 1.720),
        (2200.0, 1.666, 1.879),
        (2300.0, 1.831, 2.064),
        (2400.0, 2.018, 2.276),
    ];
    const CALCIUM_SILICATE: [(f64, f64, f64); 3] = [
        (1750.0, 0.870, 1.530),
        (1850.0, 1.000, 1.750),
        (2200.0, 1.570, 2.760),
    ];
    const AERATED: [(f64, f64, f64); 5] = [
        (400.0, 0.119, 0.129),
        (500.0, 0.141, 0.153),
        (600.0, 0.173, 0.188),
        (700.0, 0.195, 0.211),
        (800.0, 0.227, 0.246),
    ];
    let dry = environment == MasonryEnvironment::DryIndoor;
    let value = match kind {
        MasonryKind::Brick => BRICK
            .iter()
            .find(|row| row.0 + 1e-9 >= density_kg_m3)
            .map(|row| match (joints, dry) {
                (MasonryJoints::Glued, true) => row.1,
                (MasonryJoints::Glued, false) => row.2,
                (MasonryJoints::Mortar, true) => row.3,
                (MasonryJoints::Mortar, false) => row.4,
            }),
        MasonryKind::Concrete => pick(&CONCRETE, density_kg_m3, dry),
        MasonryKind::CalciumSilicate => pick(&CALCIUM_SILICATE, density_kg_m3, dry),
        MasonryKind::AeratedConcrete => pick(&AERATED, density_kg_m3, dry),
    }?;
    Some(round_lambda_up(value))
}

fn pick(rows: &[(f64, f64, f64)], density: f64, dry: bool) -> Option<f64> {
    rows.iter()
        .find(|row| row.0 + 1e-9 >= density)
        .map(|row| if dry { row.1 } else { row.2 })
}

/// Tables E.3/E.4 F_M for masonry units and mortar.
pub fn masonry_moisture_factor(kind: Option<MasonryKind>, environment: MasonryEnvironment) -> f64 {
    let dry = environment == MasonryEnvironment::DryIndoor;
    match kind {
        Some(MasonryKind::Brick) => {
            if dry {
                1.07
            } else {
                2.01
            }
        }
        Some(MasonryKind::AeratedConcrete) => {
            if dry {
                1.08
            } else {
                1.17
            }
        }
        Some(MasonryKind::CalciumSilicate) => {
            if dry {
                1.14
            } else {
                2.01
            }
        }
        Some(MasonryKind::Concrete) => {
            if dry {
                1.13
            } else {
                1.32
            }
        }
        // Mortar (table E.4).
        None => {
            if dry {
                1.17
            } else {
                1.82
            }
        }
    }
}

/// Table H.1 window and frame materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowMaterial {
    RedCopper,
    Aluminium,
    Copper,
    Steel,
    StainlessSteel,
    Pvc,
    Hardwood,
    Softwood,
    GlassFibrePolyester,
    SodaLimeGlass,
    Acrylic,
    Pmma,
    Polycarbonate,
    Polyamide,
    Polyamide66GlassFibre,
    PolyethyleneHd,
    PolyethyleneLd,
    Polypropylene,
    PolypropyleneGlassFibre,
    PolyurethaneBreak,
    PvcRigid,
    Neoprene,
    Epdm,
    SiliconeSeal,
    PvcFlexible,
    Mohair,
    FoamRubber,
    PurFoamSealant,
    Butyl,
    Polysulfide,
    SiliconeSealant,
    Polyisobutylene,
    PolyesterResin,
    SilicaGel,
    SiliconeFoamLd,
    SiliconeFoamMd,
}

impl WindowMaterial {
    pub fn lambda(self) -> f64 {
        use WindowMaterial::*;
        match self {
            RedCopper => 380.0,
            Aluminium => 160.0,
            Copper => 120.0,
            Steel => 50.0,
            StainlessSteel => 17.0,
            Pvc | PvcRigid => 0.17,
            Hardwood => 0.18,
            Softwood => 0.13,
            GlassFibrePolyester => 0.30,
            SodaLimeGlass => 1.0,
            Acrylic => 0.20,
            Pmma => 0.18,
            Polycarbonate => 0.20,
            Polyamide => 0.25,
            Polyamide66GlassFibre => 0.30,
            PolyethyleneHd => 0.52,
            PolyethyleneLd => 0.33,
            Polypropylene => 0.22,
            PolypropyleneGlassFibre | PolyurethaneBreak => 0.25,
            Neoprene => 0.23,
            Epdm => 0.25,
            SiliconeSeal | SiliconeSealant => 0.35,
            PvcFlexible | Mohair => 0.14,
            FoamRubber => 0.06,
            PurFoamSealant => 0.30,
            Butyl => 0.24,
            Polysulfide => 0.40,
            Polyisobutylene => 0.20,
            PolyesterResin => 0.19,
            SilicaGel => 0.13,
            SiliconeFoamLd => 0.12,
            SiliconeFoamMd => 0.17,
        }
    }
}

/// How a material layer's design conductivity is obtained.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Conductivity {
    /// λ_calc already determined (e.g. NEN-EN-ISO 10456 8.3 for other
    /// materials, E.2.2.3.1 last paragraph).
    Calculated {
        #[serde(rename = "lambdaCalc")]
        lambda_calc: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// E.3 from a declared λ_D (CE / DoP / quality statement).
    DeclaredInsulation {
        #[serde(rename = "lambdaDeclared")]
        lambda_declared: f64,
        moisture: InsulationMoisture,
        ageing: Ageing,
        #[serde(default)]
        temperature: Option<TemperatureConversion>,
        /// F_conv when the modified Rayleigh number exceeds table E.6;
        /// `None` means 1,00.
        #[serde(default, rename = "convectionFactor")]
        convection_factor: Option<f64>,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// E.3 from a forfait λ_for (tables E.10–E.12).
    ForfaitInsulation {
        material: ForfaitMaterial,
        moisture: InsulationMoisture,
        ageing: Ageing,
    },
    /// Tables E.14–E.17.
    MasonryTable {
        kind: MasonryKind,
        #[serde(rename = "densityKgM3")]
        density_kg_m3: f64,
        environment: MasonryEnvironment,
        joints: MasonryJoints,
    },
    /// E.5 from declared λ_D of units and mortar (tables E.3/E.4).
    MasonryDeclared {
        kind: MasonryKind,
        #[serde(rename = "unitLambdaDeclared")]
        unit_lambda_declared: f64,
        #[serde(rename = "mortarLambdaDeclared")]
        mortar_lambda_declared: f64,
        /// A_voeg/(A_voeg + A_steen); 0 for glued joints ≤ 3 mm.
        #[serde(rename = "jointAreaFraction")]
        joint_area_fraction: f64,
        environment: MasonryEnvironment,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// E.6 with table E.1.
    DeclaredOther {
        #[serde(rename = "lambdaDeclared")]
        lambda_declared: f64,
        class: OtherMaterialClass,
        #[serde(rename = "densityKgM3")]
        density_kg_m3: f64,
        #[serde(rename = "sourceReference")]
        source_reference: String,
    },
    /// Table H.1 (window and frame materials).
    WindowMaterial { material: WindowMaterial },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MaterialIssue {
    pub code: &'static str,
    pub field: &'static str,
}

impl Conductivity {
    pub fn validate(&self) -> Vec<MaterialIssue> {
        let mut issues = Vec::new();
        let mut push = |code, field| issues.push(MaterialIssue { code, field });
        let positive = |v: f64| v.is_finite() && v > 0.0;
        match self {
            Self::Calculated {
                lambda_calc,
                source_reference,
            } => {
                if !positive(*lambda_calc) {
                    push("lambda_invalid", "lambdaCalc");
                }
                if source_reference.trim().is_empty() {
                    push("source_reference_required", "sourceReference");
                }
            }
            Self::DeclaredInsulation {
                lambda_declared,
                temperature,
                convection_factor,
                source_reference,
                ..
            } => {
                if !positive(*lambda_declared) {
                    push("lambda_invalid", "lambdaDeclared");
                }
                if source_reference.trim().is_empty() {
                    push("source_reference_required", "sourceReference");
                }
                if let Some(t) = temperature {
                    if !t.mean_temperature_c.is_finite() || !t.conversion_coefficient.is_finite() {
                        push("temperature_conversion_invalid", "temperature");
                    }
                    if t.source_reference.trim().is_empty() {
                        push("source_reference_required", "temperature.sourceReference");
                    }
                }
                if let Some(f) = convection_factor {
                    if !(*f >= 1.0 && f.is_finite()) {
                        push("convection_factor_invalid", "convectionFactor");
                    }
                }
            }
            Self::ForfaitInsulation { .. } | Self::WindowMaterial { .. } => {}
            Self::MasonryTable {
                kind,
                density_kg_m3,
                environment,
                joints,
            } => {
                if masonry_table_lambda(*kind, *density_kg_m3, *environment, *joints).is_none() {
                    push("masonry_density_outside_table", "densityKgM3");
                }
            }
            Self::MasonryDeclared {
                unit_lambda_declared,
                mortar_lambda_declared,
                joint_area_fraction,
                source_reference,
                ..
            } => {
                if !positive(*unit_lambda_declared) || !positive(*mortar_lambda_declared) {
                    push("lambda_invalid", "unitLambdaDeclared");
                }
                if !(0.0..1.0).contains(joint_area_fraction) {
                    push("joint_area_fraction_invalid", "jointAreaFraction");
                }
                if source_reference.trim().is_empty() {
                    push("source_reference_required", "sourceReference");
                }
            }
            Self::DeclaredOther {
                lambda_declared,
                density_kg_m3,
                source_reference,
                ..
            } => {
                if !positive(*lambda_declared) {
                    push("lambda_invalid", "lambdaDeclared");
                }
                if !positive(*density_kg_m3) {
                    push("density_invalid", "densityKgM3");
                }
                if source_reference.trim().is_empty() {
                    push("source_reference_required", "sourceReference");
                }
            }
        }
        issues
    }

    /// λ_calc in W/(m·K), rounded up per E.2.1.1. Call after `validate`.
    pub fn lambda_calc(&self) -> f64 {
        match self {
            Self::Calculated { lambda_calc, .. } => *lambda_calc,
            Self::DeclaredInsulation {
                lambda_declared,
                moisture,
                ageing,
                temperature,
                convection_factor,
                ..
            } => round_lambda_up(
                lambda_declared
                    * temperature
                        .as_ref()
                        .map_or(1.0, TemperatureConversion::factor)
                    * moisture.factor()
                    * ageing.factor()
                    * convection_factor.unwrap_or(1.0),
            ),
            Self::ForfaitInsulation {
                material,
                moisture,
                ageing,
            } => round_lambda_up(material.lambda() * moisture.factor() * ageing.factor()),
            Self::MasonryTable {
                kind,
                density_kg_m3,
                environment,
                joints,
            } => masonry_table_lambda(*kind, *density_kg_m3, *environment, *joints)
                .unwrap_or(f64::NAN),
            Self::MasonryDeclared {
                kind,
                unit_lambda_declared,
                mortar_lambda_declared,
                joint_area_fraction,
                environment,
                ..
            } => {
                let unit = round_lambda_up(
                    unit_lambda_declared * masonry_moisture_factor(Some(*kind), *environment),
                );
                if *joint_area_fraction == 0.0 {
                    return unit;
                }
                let mortar = round_lambda_up(
                    mortar_lambda_declared * masonry_moisture_factor(None, *environment),
                );
                round_lambda_up(joint_area_fraction * mortar + (1.0 - joint_area_fraction) * unit)
            }
            Self::DeclaredOther {
                lambda_declared,
                class,
                density_kg_m3,
                ..
            } => round_lambda_up(lambda_declared * moisture_ageing_factor(*class, *density_kg_m3)),
            Self::WindowMaterial { material } => material.lambda(),
        }
    }

    /// Insulation in the sense of E.2.2.1 note 1 (λ ≤ 0,100 W/(m·K)).
    pub fn is_insulation(&self) -> bool {
        match self {
            Self::DeclaredInsulation { .. } => true,
            Self::ForfaitInsulation { material, .. } => material.is_insulation(),
            other => other.lambda_calc() <= 0.100,
        }
    }
}

/// Table E.13 reflective foil systems.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReflectiveFoilSystem {
    /// Multi-foil (bubble foil etc.): R = d/0,03 with d the guaranteed
    /// thickness in m.
    FoilLayers {
        #[serde(rename = "thicknessM")]
        thickness_m: f64,
    },
    /// Foil as facing: R = 0.
    Facing,
    TwoFoilsWithAirLayer,
    ThreeFoilsWithAirLayers,
}

impl ReflectiveFoilSystem {
    pub fn resistance(self) -> f64 {
        match self {
            Self::FoilLayers { thickness_m } => thickness_m / 0.03,
            Self::Facing => 0.0,
            Self::TwoFoilsWithAirLayer => 1.80,
            Self::ThreeFoilsWithAirLayers => 2.90,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_follows_iso_10456() {
        assert!((round_lambda_up(0.0351) - 0.036).abs() < 1e-12);
        assert!((round_lambda_up(0.035) - 0.035).abs() < 1e-12);
        assert!((round_lambda_up(0.081) - 0.085).abs() < 1e-12);
        assert!((round_lambda_up(1.033) - 1.04).abs() < 1e-12);
        assert!((round_lambda_up(2.018) - 2.1).abs() < 1e-12);
        assert!((round_resistance_down(4.379) - 4.37).abs() < 1e-12);
        assert!((round_resistance_down(12.37) - 12.3).abs() < 1e-12);
        assert!((round_half_up(4.65, 1) - 4.7).abs() < 1e-12);
        assert!((round_half_up(1.2513, 2) - 1.25).abs() < 1e-12);
    }

    #[test]
    fn insulation_conversion_follows_e_3() {
        let forfait = Conductivity::ForfaitInsulation {
            material: ForfaitMaterial::EpsWhiteBeads,
            moisture: InsulationMoisture::Regular,
            ageing: Ageing::InSitu {
                product: InSituProduct::EpsBeads,
                situation: InSituSituation::B,
                practice_tested: false,
            },
        };
        // 0,045 × 1,05 × 1,15 = 0,0543 → 0,055.
        assert!((forfait.lambda_calc() - 0.055).abs() < 1e-12);
        let declared = Conductivity::DeclaredInsulation {
            lambda_declared: 0.035,
            moisture: InsulationMoisture::InvertedRoofXpsSlopeUpTo1Percent,
            ageing: Ageing::FactoryMade,
            temperature: None,
            convection_factor: None,
            source_reference: "DoP".into(),
        };
        // 0,035 × 1,04 = 0,0364 → 0,037.
        assert!((declared.lambda_calc() - 0.037).abs() < 1e-12);
        let cold = TemperatureConversion {
            mean_temperature_c: 0.0,
            conversion_coefficient: 0.003,
            source_reference: "ISO 10456".into(),
        };
        assert!((cold.factor() - (-0.03f64).exp()).abs() < 1e-12);
    }

    #[test]
    fn masonry_and_other_materials() {
        assert_eq!(
            masonry_table_lambda(
                MasonryKind::Brick,
                1800.0,
                MasonryEnvironment::Other,
                MasonryJoints::Mortar
            ),
            Some(1.16)
        );
        // Between rows: next higher density (1 900).
        assert_eq!(
            masonry_table_lambda(
                MasonryKind::Brick,
                1850.0,
                MasonryEnvironment::Other,
                MasonryJoints::Mortar
            ),
            Some(1.27)
        );
        // 1,033 rounds up to 1,04.
        assert_eq!(
            masonry_table_lambda(
                MasonryKind::Concrete,
                1600.0,
                MasonryEnvironment::DryIndoor,
                MasonryJoints::Mortar
            ),
            Some(1.04)
        );
        assert_eq!(
            masonry_table_lambda(
                MasonryKind::AeratedConcrete,
                900.0,
                MasonryEnvironment::DryIndoor,
                MasonryJoints::Glued
            ),
            None
        );
        let declared = Conductivity::MasonryDeclared {
            kind: MasonryKind::CalciumSilicate,
            unit_lambda_declared: 0.76,
            mortar_lambda_declared: 0.83,
            joint_area_fraction: 0.1,
            environment: MasonryEnvironment::DryIndoor,
            source_reference: "EN 1745".into(),
        };
        // unit 0,76×1,14 = 0,8664 → 0,87; mortar 0,83×1,17 = 0,9711 → 0,98;
        // 0,1×0,98 + 0,9×0,87 = 0,881 → 0,89.
        assert!((declared.lambda_calc() - 0.89).abs() < 1e-12);
        assert_eq!(
            moisture_ageing_factor(OtherMaterialClass::Inorganic, 2400.0),
            1.25
        );
        assert_eq!(
            moisture_ageing_factor(OtherMaterialClass::Inorganic, 850.0),
            1.40
        );
        assert_eq!(
            moisture_ageing_factor(OtherMaterialClass::Organic, 350.0),
            1.25
        );
        assert_eq!(WindowMaterial::Aluminium.lambda(), 160.0);
        assert!(
            (ReflectiveFoilSystem::FoilLayers { thickness_m: 0.006 }.resistance() - 0.2).abs()
                < 1e-12
        );
    }
}
