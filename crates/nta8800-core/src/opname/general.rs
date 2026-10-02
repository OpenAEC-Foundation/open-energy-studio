//! ISSO 82.1 (7e druk) chapters 5–7: general data of the dwelling.
//!
//! - Year of a device: manufacture year, else installation year, else the
//!   construction year of the dwelling (p. 28).
//! - Renovation year for infiltration: decision scheme of afb. 7.3 (p. 54);
//!   an applicable but unknown year is the first year of the next year
//!   class above the construction-year class (p. 55).
//! - Dwelling position: a dwelling that fits no other type is a corner
//!   dwelling (p. 51).
//! - Infiltration: NTA table 11.14 type from building type, position and
//!   roof type (p. 55–56).
//! - Thermal mass: floor and wall type of tables 7.5/7.6 (p. 62 and erratum
//!   §2) mapped to the NTA table 7.10 classes; the kernel uses table 7.10, so
//!   the printed 350 for "zeer zwaar/zwaar" in ISSO table 7.4 (p. 62) does
//!   not apply (NTA and Wijzigingsdocument give 450).

use serde::{Deserialize, Serialize};

use super::Recorder;
use crate::monthly_demand::{CeilingColumn, MassClass};
use crate::ventilation::AirtightnessType;

/// Year classes of ISSO 82.1 p. 55 (same as NTA table 11.13).
const YEAR_CLASS_STARTS: [i32; 5] = [1970, 1980, 1990, 2000, 2010];

/// First year of the next year class above the class of `year` (p. 55).
pub fn next_year_class_start(year: i32) -> i32 {
    YEAR_CLASS_STARTS
        .iter()
        .copied()
        .find(|start| *start > year)
        .unwrap_or(year.max(2010))
}

/// p. 28: manufacture year → installation year → construction year.
pub fn device_year(
    manufacture: Option<i32>,
    installation: Option<i32>,
    construction_year: i32,
    recorder: &mut Recorder,
    path: &str,
) -> i32 {
    match (manufacture, installation) {
        (Some(year), _) => year,
        (None, Some(year)) => {
            recorder.record(
                "device_year_from_installation",
                path,
                year.to_string(),
                "ISSO 82.1 p. 28",
            );
            year
        }
        (None, None) => {
            recorder.record(
                "device_year_from_construction_year",
                path,
                construction_year.to_string(),
                "ISSO 82.1 p. 28",
            );
            construction_year
        }
    }
}

/// Answers of the renovation decision scheme (afb. 7.3).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Renovation {
    /// At least 90 % of the envelope to outdoor air (or the attic floor)
    /// post-insulated after construction.
    pub envelope_post_insulated: bool,
    /// At least 90 % of the window area got insulating glass and draught
    /// strips at the opening parts.
    pub glazing_replaced_with_draught_strips: bool,
    /// At least 90 % of the window area got new frames.
    pub frames_replaced: bool,
    /// At least 90 % of the window area sealed between wall and frame, or
    /// the joint sealed by the façade insulation.
    pub frame_joints_sealed: bool,
    /// Year of the measures; for roof and façade in different years the
    /// roof year (p. 55). `None` when unknown.
    #[serde(default)]
    pub year: Option<i32>,
    pub evidence_reference: String,
}

/// Year used for the infiltration reference value (p. 52–55).
pub fn infiltration_year(
    construction_year: i32,
    renovation: Option<&Renovation>,
    recorder: &mut Recorder,
) -> Option<i32> {
    let renovation = renovation?;
    let applicable = renovation.envelope_post_insulated
        && renovation.glazing_replaced_with_draught_strips
        && (renovation.frames_replaced || renovation.frame_joints_sealed);
    if !applicable {
        recorder.record(
            "renovation_year_not_applicable",
            "renovation",
            "construction year".into(),
            "ISSO 82.1 p. 54 (afb. 7.3)",
        );
        return None;
    }
    Some(match renovation.year {
        Some(year) => year,
        None => {
            let year = next_year_class_start(construction_year);
            recorder.record(
                "renovation_year_unknown_next_class",
                "renovation.year",
                year.to_string(),
                "ISSO 82.1 p. 55",
            );
            year
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DwellingKind {
    SingleFamily,
    Apartment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SingleFamilyPosition {
    Terraced,
    EndOrCorner,
    Detached,
    /// Fits no other type: corner dwelling (p. 51).
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoofType {
    Pitched,
    /// At least 50 % flat; only for detached dwellings (p. 52).
    PartlyFlat,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApartmentFloor {
    GroundOrIntermediate,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApartmentSide {
    Middle,
    EndOrCorner,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DwellingType {
    SingleFamily {
        position: SingleFamilyPosition,
        #[serde(rename = "roofType")]
        roof_type: RoofType,
    },
    Apartment {
        floor: ApartmentFloor,
        side: ApartmentSide,
    },
}

impl DwellingType {
    pub fn kind(&self) -> DwellingKind {
        match self {
            Self::SingleFamily { .. } => DwellingKind::SingleFamily,
            Self::Apartment { .. } => DwellingKind::Apartment,
        }
    }
}

/// NTA table 11.14 type for the dwelling (ISSO 82.1 p. 51–56).
pub fn airtightness_type(dwelling: &DwellingType, recorder: &mut Recorder) -> AirtightnessType {
    match dwelling {
        DwellingType::SingleFamily {
            position,
            roof_type,
        } => {
            let position = match position {
                SingleFamilyPosition::Unknown => {
                    recorder.record(
                        "dwelling_position_unknown_corner",
                        "dwelling.position",
                        "end_or_corner".into(),
                        "ISSO 82.1 p. 51",
                    );
                    SingleFamilyPosition::EndOrCorner
                }
                other => *other,
            };
            let roof = match (roof_type, position) {
                (RoofType::PartlyFlat, SingleFamilyPosition::Detached) => RoofType::PartlyFlat,
                (RoofType::PartlyFlat, _) => {
                    recorder.record(
                        "partly_flat_roof_only_detached",
                        "dwelling.roofType",
                        "pitched".into(),
                        "ISSO 82.1 p. 52 (partly flat exists for detached dwellings only; interpretation: pitched)",
                    );
                    RoofType::Pitched
                }
                (other, _) => *other,
            };
            match (roof, position) {
                (RoofType::Pitched, SingleFamilyPosition::Terraced) => {
                    AirtightnessType::PitchedRoofTerraced
                }
                (RoofType::Pitched, SingleFamilyPosition::Detached) => {
                    AirtightnessType::PitchedRoofDetached
                }
                (RoofType::Pitched, _) => AirtightnessType::PitchedRoofEndOrCorner,
                (RoofType::PartlyFlat, _) => AirtightnessType::PitchedRoofDetachedPartlyFlat,
                (RoofType::Flat, SingleFamilyPosition::Terraced) => {
                    AirtightnessType::FlatRoofTerraced
                }
                (RoofType::Flat, SingleFamilyPosition::Detached) => {
                    AirtightnessType::FlatRoofDetached
                }
                (RoofType::Flat, _) => AirtightnessType::FlatRoofEndOrCorner,
            }
        }
        DwellingType::Apartment { floor, side } => {
            let side = match side {
                ApartmentSide::Unknown => {
                    recorder.record(
                        "apartment_side_unknown_end",
                        "dwelling.side",
                        "end_or_corner".into(),
                        "ISSO 82.1 p. 51 asks a substantiated choice; interpretation: end/corner (conservative, p. 28)",
                    );
                    ApartmentSide::EndOrCorner
                }
                other => *other,
            };
            match (floor, side) {
                (ApartmentFloor::GroundOrIntermediate, ApartmentSide::Middle) => {
                    AirtightnessType::StoreyMiddleLowerOrIntermediate
                }
                (ApartmentFloor::GroundOrIntermediate, _) => {
                    AirtightnessType::StoreyEndLowerOrIntermediate
                }
                (ApartmentFloor::Top, ApartmentSide::Middle) => AirtightnessType::StoreyMiddleTop,
                (ApartmentFloor::Top, _) => AirtightnessType::StoreyEndTop,
            }
        }
    }
}

/// Table 7.5 floor types (p. 62).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FloorConstruction {
    /// Timber, timber-frame, steel-frame, aerated concrete, or any floor
    /// insulated on top.
    Light,
    /// Steel-concrete, hollow-core or cassette, timber-concrete, light
    /// floors with a screed, very heavy floors with insulation and screed.
    Heavy,
    /// Solid concrete.
    VeryHeavy,
}

/// Table 7.6 wall types (erratum §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WallConstruction {
    /// Timber or steel frame, steel skeleton, walls insulated inside.
    Light,
    /// Load-bearing masonry, concrete column-beam skeleton.
    Heavy,
    /// Concrete wall-floor skeleton.
    VeryHeavy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Construction {
    pub floor: FloorConstruction,
    pub wall: WallConstruction,
    /// (Very) heavy floor with a lighter (suspended) ceiling.
    #[serde(default)]
    pub lighter_ceiling: bool,
    pub source_reference: String,
}

/// Kernel thermal-mass classes for the ISSO construction types.
pub fn thermal_mass(construction: &Construction) -> (MassClass, MassClass, CeilingColumn) {
    let floor = match construction.floor {
        FloorConstruction::Light => MassClass::Light,
        FloorConstruction::Heavy => MassClass::Heavy,
        FloorConstruction::VeryHeavy => MassClass::VeryHeavy,
    };
    let wall = match construction.wall {
        WallConstruction::Light => MassClass::Light,
        WallConstruction::Heavy => MassClass::Heavy,
        WallConstruction::VeryHeavy => MassClass::VeryHeavy,
    };
    let ceiling = if construction.lighter_ceiling && floor != MassClass::Light {
        CeilingColumn::ClosedOrSuspended
    } else {
        CeilingColumn::OpenOrNone
    };
    (floor, wall, ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renovation_year_follows_afb_7_3_and_p_55() {
        let mut recorder = Recorder::default();
        let mut renovation = Renovation {
            envelope_post_insulated: true,
            glazing_replaced_with_draught_strips: true,
            frames_replaced: false,
            frame_joints_sealed: true,
            year: None,
            evidence_reference: "invoices".into(),
        };
        assert_eq!(
            infiltration_year(1965, Some(&renovation), &mut recorder),
            Some(1970)
        );
        assert_eq!(next_year_class_start(1985), 1990);
        assert_eq!(next_year_class_start(2012), 2012);
        renovation.frame_joints_sealed = false;
        assert_eq!(
            infiltration_year(1965, Some(&renovation), &mut recorder),
            None
        );
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "renovation_year_not_applicable"));
    }

    #[test]
    fn unknown_position_is_a_corner_dwelling() {
        let mut recorder = Recorder::default();
        let kind = airtightness_type(
            &DwellingType::SingleFamily {
                position: SingleFamilyPosition::Unknown,
                roof_type: RoofType::Pitched,
            },
            &mut recorder,
        );
        assert_eq!(kind, AirtightnessType::PitchedRoofEndOrCorner);
        let partly = airtightness_type(
            &DwellingType::SingleFamily {
                position: SingleFamilyPosition::Terraced,
                roof_type: RoofType::PartlyFlat,
            },
            &mut recorder,
        );
        assert_eq!(partly, AirtightnessType::PitchedRoofTerraced);
        let flat = airtightness_type(
            &DwellingType::Apartment {
                floor: ApartmentFloor::Top,
                side: ApartmentSide::Middle,
            },
            &mut recorder,
        );
        assert_eq!(flat, AirtightnessType::StoreyMiddleTop);
    }

    #[test]
    fn device_year_falls_back_to_construction_year() {
        let mut recorder = Recorder::default();
        assert_eq!(device_year(None, None, 1975, &mut recorder, "x"), 1975);
        assert_eq!(
            device_year(None, Some(2004), 1975, &mut recorder, "x"),
            2004
        );
        assert_eq!(recorder.applied.len(), 2);
    }

    #[test]
    fn thermal_mass_maps_tables_7_5_and_7_6() {
        let (floor, wall, ceiling) = thermal_mass(&Construction {
            floor: FloorConstruction::VeryHeavy,
            wall: WallConstruction::Heavy,
            lighter_ceiling: true,
            source_reference: "survey".into(),
        });
        assert_eq!(floor, MassClass::VeryHeavy);
        assert_eq!(wall, MassClass::Heavy);
        assert_eq!(ceiling, CeilingColumn::ClosedOrSuspended);
    }
}
