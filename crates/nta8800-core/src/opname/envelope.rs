//! ISSO 82.1 (7e druk) chapter 8: envelope of the basic survey.
//!
//! - Opaque R_c by construction year and insulation state: NTA annex I
//!   (ISSO tables 8.9–8.11, p. 88–91) via [`crate::forfait_envelope`];
//!   post-insulated cavity of unknown width: table 8.26 (erratum §3,
//!   p. 84): 40 mm (1930–1970), 70 mm (1970–1985), 100 mm (after 1985).
//! - Glazing equivalences (p. 94) and U/g of tables 8.14/8.15 (p. 94–95,
//!   NTA tables I.8/I.9); frame fraction 0,25 (NTA 7.6.6.2 method B);
//!   glass without a frame counts as a wood/plastic frame (p. 94).
//! - Doors: below 65 % glass split into a window and a door part (p. 70);
//!   door U of table 8.16/8.17 (p. 95); an undeterminable insulation is
//!   taken as uninsulated (conservative choice, p. 28).
//! - Panels: tables 8.18–8.21 (p. 97) via annex I.
//! - Thermal bridges: forfait route for the whole building (p. 79, NTA
//!   8.2/8.3 with table 8.1); floors on ground get 0,5·P (8.37).
//! - Adjacent unheated spaces: H_ue = 5·A_T;iu (NTA I.8, basic survey) with
//!   b_U = H_ue/(H_ue + H_iu) (8.53 with H_V;iu = 0).
//! - Obstruction: the default situation is "none" (p. 195), NTA §17.3.2a.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::Recorder;
use crate::climate::Orientation;
use crate::forfait_envelope::{
    basic_survey_unheated_transfer, forfait_door_u, forfait_panel_u, forfait_window_u,
    BuildingKind, ElementType, ForfaitGlass, ForfaitOpaque, InsulationState, PanelInsulation,
};
use crate::window_u::FrameGroup;

pub const FRAME_FRACTION_FORFAIT: f64 = 0.25;
pub const GLAZED_DOOR_THRESHOLD: f64 = 0.65;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceElement {
    Facade,
    Roof,
    Floor,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SurfaceBoundary {
    Outdoor,
    Ground,
    /// Floor over a crawlspace.
    Crawlspace,
    UnheatedSpace {
        #[serde(rename = "spaceId")]
        space_id: String,
    },
    /// Adjacent heated space or other dwelling: no transmission (8.5).
    AdjacentHeated,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum InsulationAnswer {
    /// No insulation, or not determinable whether present.
    NoneOrUnknown,
    /// (Post-)insulated, thickness not determinable.
    PresentUnknownThickness,
    /// Cavity filled afterwards, cavity width unknown (table 8.26).
    CavityFilledUnknownWidth,
    Thickness {
        #[serde(rename = "thicknessMm")]
        thickness_mm: f64,
        /// Product thickness proven (no rounding to 10 mm).
        #[serde(default)]
        proven: bool,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveySurface {
    pub id: String,
    pub element: SurfaceElement,
    pub boundary: SurfaceBoundary,
    /// Gross area including the openings placed in it, m².
    pub gross_area_m2: f64,
    #[serde(default)]
    pub orientation: Option<Orientation>,
    /// Tilt from horizontal; default 90° (façade), 0° (roof), 180° (floor
    /// over outdoor air).
    #[serde(default)]
    pub tilt_deg: Option<f64>,
    pub cavity: bool,
    pub insulation: InsulationAnswer,
    /// Thermal cushions under a floor (p. 93, R_c 1,95 per WD p. 37).
    #[serde(default)]
    pub thermal_cushions: bool,
    /// Exposed perimeter of a floor on ground or crawlspace, m.
    #[serde(default)]
    pub exposed_perimeter_m: Option<f64>,
    /// Crawlspace bottom insulated without a declaration: R_bf 0,5
    /// (table 8.13, p. 93); `None`/false: uninsulated.
    #[serde(default)]
    pub crawlspace_bottom_insulated: Option<bool>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GlassAnswer {
    TripleHr,
    HrPlusPlus,
    HrPlus,
    Hr,
    /// Low-e coating present, type not determinable: HR (p. 94).
    CoatedTypeUnknown,
    Double,
    /// Double glazing with a low-e coating: HR (p. 94).
    DoubleWithCoating,
    /// Double glazing with a secondary window: HR (p. 94).
    DoubleWithSecondary,
    /// HR with a secondary window: HR++ (p. 94).
    HrWithSecondary,
    /// HR++ with a secondary window: triple HR (p. 94).
    HrPlusPlusWithSecondary,
    SecondaryWindow,
    Single,
    /// Leaded light not in double glazing: single (p. 94).
    LeadedLight,
    /// Glass blocks: double (p. 94).
    GlassBlocks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameAnswer {
    WoodOrPlastic,
    MetalWithThermalBreak,
    Metal,
    /// Glazing without a frame: wood/plastic (p. 94).
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyWindow {
    pub id: String,
    pub surface_id: String,
    pub area_m2: f64,
    pub glass: GlassAnswer,
    pub frame: FrameAnswer,
    /// Declared obstruction factors per §17.3 (12 heating, 12 cooling);
    /// absent: no obstruction (p. 195).
    #[serde(default)]
    pub obstruction: Option<DeclaredObstruction>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeclaredObstruction {
    pub heating: Vec<f64>,
    pub cooling: Vec<f64>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyDoor {
    pub id: String,
    pub surface_id: String,
    pub area_m2: f64,
    /// `None`: not determinable.
    #[serde(default)]
    pub insulated: Option<bool>,
    #[serde(default)]
    pub glass_fraction: f64,
    #[serde(default)]
    pub glass: Option<GlassAnswer>,
    pub frame: FrameAnswer,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyPanel {
    pub id: String,
    pub surface_id: String,
    pub area_m2: f64,
    pub insulation: PanelInsulation,
    pub cavity: bool,
    pub frame: FrameAnswer,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyUnheatedSpace {
    pub id: String,
    pub description: String,
}

/// Envelope part of the survey.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyEnvelope {
    pub surfaces: Vec<SurveySurface>,
    #[serde(default)]
    pub windows: Vec<SurveyWindow>,
    #[serde(default)]
    pub doors: Vec<SurveyDoor>,
    #[serde(default)]
    pub panels: Vec<SurveyPanel>,
    #[serde(default)]
    pub unheated_spaces: Vec<SurveyUnheatedSpace>,
}

pub fn frame_group(frame: FrameAnswer, recorder: &mut Recorder, path: &str) -> FrameGroup {
    match frame {
        FrameAnswer::WoodOrPlastic => FrameGroup::WoodOrPlastic,
        FrameAnswer::MetalWithThermalBreak => FrameGroup::MetalWithThermalBreak,
        FrameAnswer::Metal => FrameGroup::MetalWithoutThermalBreak,
        FrameAnswer::None => {
            recorder.record(
                "glazing_without_frame_wood_plastic",
                path,
                "wood_or_plastic".into(),
                "ISSO 82.1 p. 94",
            );
            FrameGroup::WoodOrPlastic
        }
    }
}

/// p. 94 equivalences to the rows of table 8.14.
pub fn glass_row(glass: GlassAnswer, recorder: &mut Recorder, path: &str) -> ForfaitGlass {
    let (row, rule) = match glass {
        GlassAnswer::TripleHr => (ForfaitGlass::TripleHr, None),
        GlassAnswer::HrPlusPlus => (ForfaitGlass::HrPlusPlus, None),
        GlassAnswer::HrPlus => (ForfaitGlass::HrPlus, None),
        GlassAnswer::Hr => (ForfaitGlass::HrCoatedDouble, None),
        GlassAnswer::CoatedTypeUnknown => (
            ForfaitGlass::HrCoatedDouble,
            Some("glazing_coating_type_unknown_hr"),
        ),
        GlassAnswer::Double => (ForfaitGlass::DoubleUncoated, None),
        GlassAnswer::DoubleWithCoating => (
            ForfaitGlass::HrCoatedDouble,
            Some("glazing_double_with_coating_hr"),
        ),
        GlassAnswer::DoubleWithSecondary => (
            ForfaitGlass::HrCoatedDouble,
            Some("glazing_double_with_secondary_hr"),
        ),
        GlassAnswer::HrWithSecondary => (
            ForfaitGlass::HrPlusPlus,
            Some("glazing_hr_with_secondary_hr_plus_plus"),
        ),
        GlassAnswer::HrPlusPlusWithSecondary => (
            ForfaitGlass::TripleHr,
            Some("glazing_hr_plus_plus_with_secondary_triple"),
        ),
        GlassAnswer::SecondaryWindow => (ForfaitGlass::SecondaryWindow, None),
        GlassAnswer::Single => (ForfaitGlass::Single, None),
        GlassAnswer::LeadedLight => (ForfaitGlass::Single, Some("glazing_leaded_light_single")),
        GlassAnswer::GlassBlocks => (
            ForfaitGlass::DoubleUncoated,
            Some("glazing_glass_blocks_double"),
        ),
    };
    if let Some(rule) = rule {
        recorder.record(rule, path, format!("{row:?}"), "ISSO 82.1 p. 94");
    }
    row
}

/// Table 8.14 g-value (to outdoor air; 0 otherwise, table 8.15).
pub fn glass_g(row: ForfaitGlass) -> f64 {
    match row {
        ForfaitGlass::TripleHr => 0.5,
        ForfaitGlass::HrPlusPlus | ForfaitGlass::HrPlus | ForfaitGlass::HrCoatedDouble => 0.6,
        ForfaitGlass::DoubleUncoated | ForfaitGlass::SecondaryWindow => 0.75,
        ForfaitGlass::Single => 0.85,
    }
}

/// Table 8.26 (erratum §3): cavity width for a post-insulated cavity.
pub fn cavity_width_mm(construction_year: i32) -> f64 {
    if construction_year < 1970 {
        40.0
    } else if construction_year <= 1985 {
        70.0
    } else {
        100.0
    }
}

/// 8.3 with Ū of the opaque outdoor elements.
pub fn delta_u_forfait(area_weighted_u: f64) -> f64 {
    (0.1 - 0.25 * (area_weighted_u - 0.4)).max(0.0)
}

/// Result of the envelope translation, as kernel JSON fragments.
pub struct DerivedEnvelope {
    pub direct_elements: Vec<Value>,
    pub windows: Vec<Value>,
    pub opaque_elements: Vec<Value>,
    pub ground_floors: Vec<Value>,
    pub unheated: Option<Value>,
    pub floor_above_crawlspace: bool,
}

struct OpaquePart {
    id: String,
    area: f64,
    u: f64,
    orientation: Orientation,
    tilt: f64,
    outdoor: bool,
    /// Counts in Ū of 8.3 (opaque, not a panel, not a ground floor).
    in_mean: bool,
    reference: String,
}

struct WindowPart {
    id: String,
    area: f64,
    u: f64,
    g: f64,
    orientation: Orientation,
    tilt: f64,
    obstruction: Value,
    reference: String,
}

fn default_tilt(surface: &SurveySurface) -> f64 {
    surface.tilt_deg.unwrap_or(match surface.element {
        SurfaceElement::Facade => 90.0,
        SurfaceElement::Roof => 0.0,
        SurfaceElement::Floor => 180.0,
    })
}

/// Row of tables 8.9/8.10 (p. 88–90) and the R_si override:
/// - a floor bordering outdoor air takes the "daken en vloeren grenzend aan
///   de buitenlucht" row, with R_si 0,17 (downward heat flow, table C.2);
/// - a ceiling to an unheated space (attic floor, AOR) takes the floor row
///   ("scheiden van de grond of een AOR"; NTA I.4 zoldervloeren), with
///   R_si 0,10 (upward heat flow).
fn element_type(surface: &SurveySurface) -> (ElementType, Option<f64>) {
    match (surface.element, &surface.boundary) {
        (SurfaceElement::Facade, _) => (ElementType::Facade, None),
        (SurfaceElement::Floor, SurfaceBoundary::Outdoor) => (ElementType::Roof, Some(0.17)),
        (SurfaceElement::Floor, _) => (ElementType::Floor, None),
        (SurfaceElement::Roof, SurfaceBoundary::UnheatedSpace { .. }) => {
            (ElementType::Floor, Some(0.10))
        }
        (SurfaceElement::Roof, _) => (ElementType::Roof, None),
    }
}

/// Translate the envelope; issues are pushed as (code, path).
pub fn derive_envelope(
    envelope: &SurveyEnvelope,
    construction_year: i32,
    recorder: &mut Recorder,
) -> DerivedEnvelope {
    let mut opaque: Vec<OpaquePart> = Vec::new();
    let mut windows: Vec<WindowPart> = Vec::new();
    let mut ground_floors = Vec::new();
    let mut partitions: Vec<(String, f64, f64, String)> = Vec::new();
    let mut floor_above_crawlspace = false;
    let mut crawl_floors: Vec<usize> = Vec::new();
    let mut facade_parts: Vec<(f64, f64)> = Vec::new();

    for (index, surface) in envelope.surfaces.iter().enumerate() {
        let path = format!("envelope.surfaces[{index}]");
        if surface.gross_area_m2 <= 0.0 || !surface.gross_area_m2.is_finite() {
            recorder.issue("surface_area_invalid", format!("{path}.grossAreaM2"));
            continue;
        }
        let exterior = matches!(surface.boundary, SurfaceBoundary::Outdoor);
        let orientation = match (surface.orientation, surface.element) {
            (Some(orientation), _) => orientation,
            (None, SurfaceElement::Facade) => {
                recorder.issue(
                    "surface_orientation_required",
                    format!("{path}.orientation"),
                );
                Orientation::South
            }
            (None, _) => Orientation::South,
        };
        let tilt = default_tilt(surface);
        if surface.element == SurfaceElement::Roof
            && tilt > 5.0
            && surface.orientation.is_none()
            && exterior
        {
            recorder.issue(
                "surface_orientation_required",
                format!("{path}.orientation"),
            );
        }

        // Openings on this surface.
        let mut openings_area = 0.0;
        for (w_index, window) in envelope.windows.iter().enumerate() {
            if window.surface_id != surface.id {
                continue;
            }
            let w_path = format!("envelope.windows[{w_index}]");
            openings_area += window.area_m2;
            let row = glass_row(window.glass, recorder, &w_path);
            let frame = frame_group(window.frame, recorder, &w_path);
            let u = forfait_window_u(row, frame, exterior);
            push_window_or_partition(
                &mut windows,
                &mut partitions,
                surface,
                WindowPart {
                    id: window.id.clone(),
                    area: window.area_m2,
                    u,
                    g: glass_g(row),
                    orientation,
                    tilt,
                    obstruction: obstruction(window.obstruction.as_ref()),
                    reference: window.source_reference.clone(),
                },
            );
        }
        for (d_index, door) in envelope.doors.iter().enumerate() {
            if door.surface_id != surface.id {
                continue;
            }
            let d_path = format!("envelope.doors[{d_index}]");
            openings_area += door.area_m2;
            let insulated = door.insulated.unwrap_or_else(|| {
                recorder.record(
                    "door_insulation_unknown_uninsulated",
                    &format!("{d_path}.insulated"),
                    "false".into(),
                    "ISSO 82.1 p. 28 (conservative), p. 95",
                );
                false
            });
            let frame = frame_group(door.frame, recorder, &d_path);
            let glass_area = door.glass_fraction.clamp(0.0, 1.0) * door.area_m2;
            let door_part_area = if door.glass_fraction >= GLAZED_DOOR_THRESHOLD {
                0.0
            } else {
                door.area_m2 - glass_area
            };
            let window_area = door.area_m2 - door_part_area;
            if window_area > 0.0 {
                let Some(glass) = door.glass else {
                    recorder.issue("door_glass_required", format!("{d_path}.glass"));
                    continue;
                };
                if door_part_area > 0.0 {
                    recorder.record(
                        "door_split_window_and_door",
                        &d_path,
                        format!("{window_area:.2} m² window"),
                        "ISSO 82.1 p. 70",
                    );
                }
                let row = glass_row(glass, recorder, &d_path);
                push_window_or_partition(
                    &mut windows,
                    &mut partitions,
                    surface,
                    WindowPart {
                        id: format!("{}-glass", door.id),
                        area: window_area,
                        u: forfait_window_u(row, frame, exterior),
                        g: glass_g(row),
                        orientation,
                        tilt,
                        obstruction: obstruction(None),
                        reference: door.source_reference.clone(),
                    },
                );
            }
            if door_part_area > 0.0 {
                let part = OpaquePart {
                    id: door.id.clone(),
                    area: door_part_area,
                    u: forfait_door_u(insulated, exterior),
                    orientation,
                    tilt,
                    outdoor: exterior,
                    in_mean: false,
                    reference: door.source_reference.clone(),
                };
                push_opaque_or_partition(&mut opaque, &mut partitions, surface, part);
            }
        }
        for (p_index, panel) in envelope.panels.iter().enumerate() {
            if panel.surface_id != surface.id {
                continue;
            }
            let p_path = format!("envelope.panels[{p_index}]");
            openings_area += panel.area_m2;
            let frame = frame_group(panel.frame, recorder, &p_path);
            match forfait_panel_u(panel.insulation, panel.cavity, frame, exterior) {
                Some(u) => push_opaque_or_partition(
                    &mut opaque,
                    &mut partitions,
                    surface,
                    OpaquePart {
                        id: panel.id.clone(),
                        area: panel.area_m2,
                        u,
                        orientation,
                        tilt,
                        outdoor: exterior,
                        in_mean: false,
                        reference: panel.source_reference.clone(),
                    },
                ),
                None => recorder.issue("panel_thickness_out_of_range", p_path),
            }
        }
        let net = surface.gross_area_m2 - openings_area;
        if net < -1e-9 {
            recorder.issue("openings_exceed_surface_area", path.clone());
            continue;
        }
        if matches!(surface.boundary, SurfaceBoundary::AdjacentHeated) {
            continue;
        }

        // Opaque part.
        let insulation = match &surface.insulation {
            InsulationAnswer::NoneOrUnknown => InsulationState::AbsentOrUnknown,
            InsulationAnswer::PresentUnknownThickness => InsulationState::PresentUnknownThickness,
            InsulationAnswer::CavityFilledUnknownWidth => {
                let width = cavity_width_mm(construction_year);
                recorder.record(
                    "cavity_width_unknown_table_8_26",
                    &format!("{path}.insulation"),
                    format!("{width} mm"),
                    "ISSO 82.1 p. 84 (table 8.26, erratum §3)",
                );
                InsulationState::KnownThickness {
                    thickness_mm: width,
                    thickness_proven: false,
                    known_lambda_equivalent: None,
                    reed_thickness_m: None,
                    thermal_cushions: false,
                }
            }
            InsulationAnswer::Thickness {
                thickness_mm,
                proven,
            } => InsulationState::KnownThickness {
                thickness_mm: *thickness_mm,
                thickness_proven: *proven,
                known_lambda_equivalent: None,
                reed_thickness_m: None,
                thermal_cushions: surface.thermal_cushions,
            },
        };
        let (element, r_si_override) = element_type(surface);
        if r_si_override.is_some() {
            recorder.record(
                "surface_table_row_by_boundary",
                &path,
                format!("{element:?} row, R_si {:.2}", r_si_override.unwrap_or_default()),
                "ISSO 82.1 p. 88–90 (tables 8.9/8.10); NTA I.4, table C.2",
            );
        }
        // p. 93 (WD p. 37): thermal cushions under a floor give R_c 1,95
        // for the floor construction, whatever else is answered.
        let cushions = surface.thermal_cushions && element == ElementType::Floor;
        let insulation = if cushions {
            recorder.record(
                "thermal_cushions_rc_1_95",
                &format!("{path}.thermalCushions"),
                "R_c 1,95 m²K/W".into(),
                "ISSO 82.1 p. 93; WD 2025 p. 37",
            );
            InsulationState::KnownThickness {
                thickness_mm: 0.0,
                thickness_proven: true,
                known_lambda_equivalent: None,
                reed_thickness_m: None,
                thermal_cushions: true,
            }
        } else {
            if surface.thermal_cushions {
                recorder.issue("thermal_cushions_floor_only", format!("{path}.thermalCushions"));
            }
            match insulation {
                InsulationState::KnownThickness {
                    thickness_mm,
                    thickness_proven,
                    known_lambda_equivalent,
                    reed_thickness_m,
                    ..
                } => InsulationState::KnownThickness {
                    thickness_mm,
                    thickness_proven,
                    known_lambda_equivalent,
                    reed_thickness_m,
                    thermal_cushions: false,
                },
                other => other,
            }
        };
        let forfait = ForfaitOpaque {
            element,
            building: BuildingKind::Regular,
            construction_year,
            insulation,
            // The flat 1,95 (= 0,15 + 1,8) has no cavity term.
            cavity: surface.cavity && !cushions,
            r_si_override,
        };
        let problems = forfait.validate(&path);
        if !problems.is_empty() {
            for problem in problems {
                recorder.issue(problem.code, problem.path);
            }
            continue;
        }
        let result = forfait.calculate();
        recorder.record(
            "opaque_rc_forfait_annex_i",
            &path,
            format!("R_c {:.2} m²K/W ({})", result.r_c, result.route),
            "ISSO 82.1 p. 84–93 (tables 8.9–8.11), NTA annex I",
        );
        match &surface.boundary {
            SurfaceBoundary::Ground | SurfaceBoundary::Crawlspace => {
                let Some(perimeter) = surface.exposed_perimeter_m else {
                    recorder.issue(
                        "exposed_perimeter_required",
                        format!("{path}.exposedPerimeterM"),
                    );
                    continue;
                };
                // The kernel takes R_si + R_c (R_si 0,17 for a floor, C.2).
                let mut floor = json!({
                    "id": surface.id,
                    "areaM2": net,
                    "exposedPerimeterM": perimeter,
                    "constructionResistanceM2kPerW": result.r_c + 0.17,
                    "edgeThermalBridges": {"method": "forfait"},
                    "sourceReference": format!("{}; basisopname R_c {} ({})", surface.source_reference, result.r_c, result.route),
                });
                if matches!(surface.boundary, SurfaceBoundary::Crawlspace) {
                    floor_above_crawlspace = true;
                    let insulated = surface.crawlspace_bottom_insulated.unwrap_or(false);
                    recorder.record(
                        "crawlspace_bottom",
                        &path,
                        if insulated {
                            "R_bf 0,5 m²K/W (insulated, no declaration)".to_string()
                        } else {
                            "uninsulated bottom, R_bf 0".to_string()
                        },
                        "ISSO 82.1 p. 93 table 8.13",
                    );
                    floor["below"] = json!({
                        "kind": "crawlspace",
                        "floorResistanceM2kPerW": if insulated { 0.5 } else { 0.0 },
                        "depthClass": "other",
                        // Filled in from the façade after the loop.
                        "wallResistanceM2kPerW": 0.0,
                        "wallUValueWPerM2k": 0.0,
                    });
                    crawl_floors.push(ground_floors.len());
                }
                ground_floors.push(floor);
            }
            _ => {
                if matches!(surface.element, SurfaceElement::Facade) && exterior {
                    facade_parts.push((result.u_c, result.r_c));
                }
                if net > 1e-9 {
                    push_opaque_or_partition(
                        &mut opaque,
                        &mut partitions,
                        surface,
                        OpaquePart {
                            id: surface.id.clone(),
                            area: net,
                            u: result.u_c,
                            orientation,
                            tilt,
                            outdoor: exterior,
                            in_mean: true,
                            reference: format!(
                                "{}; basisopname R_c {} ({})",
                                surface.source_reference, result.r_c, result.route
                            ),
                        },
                    );
                }
            }
        }
    }

    // Table 8.13 (p. 93): crawlspace wall R_bw = R_c of the façade above
    // (the lowest of several); U_xw = U of that façade (8.47 note 3).
    if !crawl_floors.is_empty() {
        // The façade with the lowest R_c (highest U); R_bw is its R_c
        // itself, not one recomputed from the rounded U.
        match facade_parts
            .iter()
            .cloned()
            .fold(None, |lowest: Option<(f64, f64)>, part| match lowest {
                Some(best) if best.1 <= part.1 => Some(best),
                _ => Some(part),
            }) {
            Some((u_facade, r_c)) => {
                for index in &crawl_floors {
                    ground_floors[*index]["below"]["wallResistanceM2kPerW"] = json!(r_c);
                    ground_floors[*index]["below"]["wallUValueWPerM2k"] = json!(u_facade);
                }
                recorder.record(
                    "crawlspace_wall_from_facade",
                    "envelope",
                    format!("R_bw {r_c:.2} m²K/W, U_xw {u_facade:.2} W/(m²K)"),
                    "ISSO 82.1 p. 93 table 8.13; NTA 8.34, 8.47",
                );
            }
            None => recorder.issue("crawlspace_requires_facade", "envelope.surfaces"),
        }
    }

    // 8.3: ΔU_for from the opaque outdoor elements (not ground floors or panels).
    let (sum_au, sum_a) = opaque
        .iter()
        .filter(|part| part.outdoor && part.in_mean)
        .fold((0.0, 0.0), |(au, a), part| {
            (au + part.area * part.u, a + part.area)
        });
    let delta_u = if sum_a > 0.0 {
        delta_u_forfait(sum_au / sum_a)
    } else {
        0.0
    };
    recorder.record(
        "thermal_bridges_forfait_delta_u",
        "envelope",
        format!("ΔU_for {delta_u:.3} W/(m²K)"),
        "ISSO 82.1 p. 79; NTA 8.2/8.3",
    );

    let mut direct_elements = Vec::new();
    let mut opaque_elements = Vec::new();
    let mut window_values = Vec::new();
    for part in opaque.iter().filter(|part| part.outdoor) {
        let u = part.u + delta_u;
        direct_elements.push(json!({
            "id": part.id, "areaM2": part.area, "uValueWPerM2k": u,
            "sourceReference": part.reference,
        }));
        opaque_elements.push(json!({
            "id": part.id, "areaM2": part.area, "orientation": part.orientation,
            "tiltDeg": part.tilt, "uValueWPerM2k": u, "sourceReference": part.reference,
        }));
    }
    for window in &windows {
        let u = window.u + delta_u;
        direct_elements.push(json!({
            "id": window.id, "areaM2": window.area, "uValueWPerM2k": u,
            "sourceReference": window.reference,
        }));
        window_values.push(json!({
            "id": window.id, "areaM2": window.area, "orientation": window.orientation,
            "tiltDeg": window.tilt, "gPerpendicular": window.g,
            "frameFraction": FRAME_FRACTION_FORFAIT, "uValueWPerM2k": u,
            "obstruction": window.obstruction, "sourceReference": window.reference,
        }));
    }
    if !windows.is_empty() {
        recorder.record(
            "frame_fraction_forfait",
            "envelope.windows",
            FRAME_FRACTION_FORFAIT.to_string(),
            "NTA 7.6.6.2 method B; ISSO 82.1 p. 93",
        );
    }

    // Adjacent unheated spaces, basic survey.
    let unheated = if partitions.is_empty() {
        None
    } else {
        let mut spaces = Vec::new();
        for space in &envelope.unheated_spaces {
            let parts: Vec<&(String, f64, f64, String)> = partitions
                .iter()
                .filter(|(id, _, _, _)| *id == space.id)
                .collect();
            if parts.is_empty() {
                continue;
            }
            let area: f64 = parts.iter().map(|p| p.1).sum();
            let h_iu: f64 = parts.iter().map(|p| p.1 * p.2).sum();
            let h_ue = basic_survey_unheated_transfer(area);
            let b = h_ue / (h_ue + h_iu);
            recorder.record(
                "unheated_space_basic_survey_h_ue",
                &format!("envelope.unheatedSpaces[{}]", space.id),
                format!("H_ue {h_ue:.1} W/K, b {b:.3}"),
                "NTA I.8 (I.2.4) and 8.53; ISSO 82.1 basisopname",
            );
            let elements: Vec<Value> = parts
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    json!({"id": format!("{}-{}", space.id, i), "areaM2": p.1,
                           "uValueWPerM2k": p.2, "sourceReference": p.3})
                })
                .collect();
            spaces.push(json!({
                "id": space.id,
                "reductionFactor": b,
                "factorSourceReference": "basisopname: H_ue = 5·A_T;iu (NTA I.8), b_U per 8.53",
                "boundary": {"elements": elements},
            }));
        }
        for (id, _, _, _) in &partitions {
            if !envelope.unheated_spaces.iter().any(|space| &space.id == id) {
                recorder.issue(
                    "unheated_space_unknown",
                    format!("envelope.unheatedSpaces[{id}]"),
                );
            }
        }
        Some(json!({ "spaces": spaces }))
    };

    DerivedEnvelope {
        direct_elements,
        windows: window_values,
        opaque_elements,
        ground_floors,
        unheated,
        floor_above_crawlspace,
    }
}

fn obstruction(declared: Option<&DeclaredObstruction>) -> Value {
    match declared {
        Some(item) => json!({
            "method": "declared", "heating": item.heating, "cooling": item.cooling,
            "sourceReference": item.source_reference,
        }),
        None => json!({"method": "minimal"}),
    }
}

fn push_window_or_partition(
    windows: &mut Vec<WindowPart>,
    partitions: &mut Vec<(String, f64, f64, String)>,
    surface: &SurveySurface,
    part: WindowPart,
) {
    match &surface.boundary {
        SurfaceBoundary::Outdoor => windows.push(part),
        SurfaceBoundary::UnheatedSpace { space_id } => {
            partitions.push((space_id.clone(), part.area, part.u, part.reference))
        }
        _ => {}
    }
}

fn push_opaque_or_partition(
    opaque: &mut Vec<OpaquePart>,
    partitions: &mut Vec<(String, f64, f64, String)>,
    surface: &SurveySurface,
    part: OpaquePart,
) {
    match &surface.boundary {
        SurfaceBoundary::Outdoor => opaque.push(part),
        SurfaceBoundary::UnheatedSpace { space_id } => {
            partitions.push((space_id.clone(), part.area, part.u, part.reference))
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface(id: &str, element: SurfaceElement, boundary: SurfaceBoundary) -> SurveySurface {
        SurveySurface {
            id: id.into(),
            element,
            boundary,
            gross_area_m2: 40.0,
            orientation: Some(Orientation::South),
            tilt_deg: None,
            cavity: true,
            insulation: InsulationAnswer::NoneOrUnknown,
            thermal_cushions: false,
            exposed_perimeter_m: Some(16.0),
            crawlspace_bottom_insulated: None,
            source_reference: "survey".into(),
        }
    }

    #[test]
    fn glazing_equivalences_follow_p_94() {
        let mut recorder = Recorder::default();
        assert_eq!(
            glass_row(GlassAnswer::DoubleWithSecondary, &mut recorder, "w"),
            ForfaitGlass::HrCoatedDouble
        );
        assert_eq!(
            glass_row(GlassAnswer::HrPlusPlusWithSecondary, &mut recorder, "w"),
            ForfaitGlass::TripleHr
        );
        assert_eq!(
            glass_row(GlassAnswer::LeadedLight, &mut recorder, "w"),
            ForfaitGlass::Single
        );
        assert_eq!(glass_g(ForfaitGlass::Single), 0.85);
        assert_eq!(glass_g(ForfaitGlass::TripleHr), 0.5);
        assert_eq!(recorder.applied.len(), 3);
        assert_eq!(
            frame_group(FrameAnswer::None, &mut recorder, "w"),
            FrameGroup::WoodOrPlastic
        );
    }

    #[test]
    fn cavity_width_follows_table_8_26() {
        assert_eq!(cavity_width_mm(1935), 40.0);
        assert_eq!(cavity_width_mm(1970), 70.0);
        assert_eq!(cavity_width_mm(1985), 70.0);
        assert_eq!(cavity_width_mm(1986), 100.0);
    }

    #[test]
    fn delta_u_follows_table_8_1() {
        assert!((delta_u_forfait(0.8) - 0.0).abs() < 1e-12);
        assert!((delta_u_forfait(0.6) - 0.05).abs() < 1e-12);
        assert!((delta_u_forfait(0.4) - 0.1).abs() < 1e-12);
    }

    #[test]
    fn envelope_splits_doors_adds_delta_u_and_derives_b() {
        let mut recorder = Recorder::default();
        let envelope = SurveyEnvelope {
            surfaces: vec![
                surface("gevel", SurfaceElement::Facade, SurfaceBoundary::Outdoor),
                surface("vloer", SurfaceElement::Floor, SurfaceBoundary::Ground),
                surface(
                    "garagewand",
                    SurfaceElement::Facade,
                    SurfaceBoundary::UnheatedSpace {
                        space_id: "garage".into(),
                    },
                ),
            ],
            windows: vec![SurveyWindow {
                id: "raam".into(),
                surface_id: "gevel".into(),
                area_m2: 8.0,
                glass: GlassAnswer::Hr,
                frame: FrameAnswer::WoodOrPlastic,
                obstruction: None,
                source_reference: "survey".into(),
            }],
            doors: vec![SurveyDoor {
                id: "voordeur".into(),
                surface_id: "gevel".into(),
                area_m2: 2.0,
                insulated: None,
                glass_fraction: 0.25,
                glass: Some(GlassAnswer::Single),
                frame: FrameAnswer::WoodOrPlastic,
                source_reference: "survey".into(),
            }],
            panels: Vec::new(),
            unheated_spaces: vec![SurveyUnheatedSpace {
                id: "garage".into(),
                description: "garage".into(),
            }],
        };
        let derived = derive_envelope(&envelope, 1975, &mut recorder);
        assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
        // Façade R_c 1975: 1,30 → U = 1/(1,30 + 0,13 + 0,04) = 0,68.
        let facade_u = 0.68;
        let delta = delta_u_forfait(facade_u);
        let wall = derived
            .direct_elements
            .iter()
            .find(|e| e["id"] == "gevel")
            .unwrap();
        assert!((wall["areaM2"].as_f64().unwrap() - 30.0).abs() < 1e-9);
        assert!((wall["uValueWPerM2k"].as_f64().unwrap() - (facade_u + delta)).abs() < 1e-9);
        // Door: 0,5 m² single glass window, 1,5 m² uninsulated door (3,4).
        let door = derived
            .direct_elements
            .iter()
            .find(|e| e["id"] == "voordeur")
            .unwrap();
        assert!((door["uValueWPerM2k"].as_f64().unwrap() - (3.4 + delta)).abs() < 1e-9);
        assert_eq!(derived.windows.len(), 2);
        assert_eq!(derived.ground_floors.len(), 1);
        let space = &derived.unheated.unwrap()["spaces"][0];
        let h_iu = 40.0 * 0.68;
        let b = 200.0 / (200.0 + h_iu);
        assert!((space["reductionFactor"].as_f64().unwrap() - b).abs() < 1e-9);
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "door_insulation_unknown_uninsulated"));
    }

    #[test]
    fn crawlspace_floor_takes_the_facade_for_its_walls() {
        let mut recorder = Recorder::default();
        let envelope = SurveyEnvelope {
            surfaces: vec![
                surface("gevel", SurfaceElement::Facade, SurfaceBoundary::Outdoor),
                surface("vloer", SurfaceElement::Floor, SurfaceBoundary::Crawlspace),
            ],
            windows: Vec::new(),
            doors: Vec::new(),
            panels: Vec::new(),
            unheated_spaces: Vec::new(),
        };
        let derived = derive_envelope(&envelope, 1975, &mut recorder);
        assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
        let floor = &derived.ground_floors[0];
        assert_eq!(floor["below"]["kind"], "crawlspace");
        assert_eq!(floor["below"]["floorResistanceM2kPerW"], 0.0);
        let u_facade = floor["below"]["wallUValueWPerM2k"].as_f64().unwrap();
        let r_bw = floor["below"]["wallResistanceM2kPerW"].as_f64().unwrap();
        // R_bw is the façade R_c itself (table 8.13), not 1/U − 0,17.
        let facade = ForfaitOpaque {
            element: ElementType::Facade,
            building: BuildingKind::Regular,
            construction_year: 1975,
            insulation: InsulationState::AbsentOrUnknown,
            cavity: true,
            r_si_override: None,
        }
        .calculate();
        assert_eq!(r_bw, facade.r_c);
        assert_eq!(u_facade, facade.u_c);
        // R_si 0,17 is added for the kernel's R_si + R_c.
        let r = floor["constructionResistanceM2kPerW"].as_f64().unwrap();
        assert!(r > 0.17);
        assert!(derived.floor_above_crawlspace);
    }

    #[test]
    fn floor_to_outdoor_air_and_thermal_cushions() {
        let mut recorder = Recorder::default();
        let mut cushions = surface("kussen", SurfaceElement::Floor, SurfaceBoundary::Crawlspace);
        cushions.thermal_cushions = true;
        cushions.exposed_perimeter_m = Some(10.0);
        let envelope = SurveyEnvelope {
            surfaces: vec![
                surface("gevel", SurfaceElement::Facade, SurfaceBoundary::Outdoor),
                surface("overstek", SurfaceElement::Floor, SurfaceBoundary::Outdoor),
                cushions,
            ],
            windows: Vec::new(),
            doors: Vec::new(),
            panels: Vec::new(),
            unheated_spaces: Vec::new(),
        };
        let derived = derive_envelope(&envelope, 1970, &mut recorder);
        assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
        // Table 8.10: floors to outdoor air take the roof row.
        let roof_row = ForfaitOpaque {
            element: ElementType::Roof,
            building: BuildingKind::Regular,
            construction_year: 1970,
            insulation: InsulationState::AbsentOrUnknown,
            cavity: true,
            r_si_override: Some(0.17),
        }
        .calculate();
        let overhang = derived
            .opaque_elements
            .iter()
            .find(|item| item["id"] == "overstek")
            .unwrap();
        let rule = recorder
            .applied
            .iter()
            .find(|item| item.rule == "opaque_rc_forfait_annex_i" && item.path == "envelope.surfaces[1]")
            .unwrap();
        assert!(rule.value.starts_with(&format!("R_c {:.2}", roof_row.r_c)), "{}", rule.value);
        assert!(overhang.is_object());
        // p. 93: thermal cushions give R_c 1,95 whatever the insulation answer.
        let floor = &derived.ground_floors[0];
        let r = floor["constructionResistanceM2kPerW"].as_f64().unwrap();
        assert!((r - (1.95 + 0.17)).abs() < 1e-9, "{r}");
    }
}
