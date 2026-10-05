//! ISSO 82.1 (7e druk) chapter 15: PV in the basic survey.
//!
//! - Crystalline type known, installation year unknown: the construction
//!   year (a year before 2001 counts as 2000). Type unknown: polycrystalline
//!   with the installation year, or placed before 2001 when that year is
//!   unknown too. Amorphous of unknown type: multi-junction (table 15.7,
//!   p. 191).
//! - Building integration unknown: not ventilated (p. 191), the kernel's
//!   `f_perf` 0,76.
//! - East/west installations are two systems (p. 191).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::Recorder;

/// Building-bound energy storage (§15.5, p. 193): only with a PV system,
/// fixed to the installation (no plug-in batteries). In an apartment
/// building every storage system behind the main meter counts (erratum
/// §6 on §15.5.1/§15.5.2): the capacities are the totals behind that
/// meter, including those of other apartments surveyed with it.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyStorage {
    #[serde(default)]
    pub electrical_kwh: f64,
    #[serde(default)]
    pub thermal_kwh: f64,
    pub source_reference: String,
}

/// `batteryStoragePresent` and `storage` of the kernel input.
pub fn derive_storage(
    storage: Option<&SurveyStorage>,
    pv_present: bool,
    recorder: &mut Recorder,
) -> (bool, Option<Value>) {
    let Some(storage) = storage else {
        return (false, None);
    };
    if !pv_present {
        recorder.issue("storage_requires_pv", "storage");
        return (false, None);
    }
    (
        true,
        Some(json!({
            "buildingBoundElectricalKwh": storage.electrical_kwh,
            "buildingBoundThermalKwh": storage.thermal_kwh,
            "sourceReference": storage.source_reference,
        })),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PvTypeAnswer {
    Monocrystalline,
    Polycrystalline,
    AmorphousSingleJunction,
    AmorphousMultiJunction,
    AmorphousUnknown,
    Cigs,
    CdTe,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MountingAnswer {
    NotVentilated,
    ModeratelyVentilated,
    StronglyVentilated,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyPv {
    pub id: String,
    pub panel_area_m2: f64,
    pub module_type: PvTypeAnswer,
    #[serde(default)]
    pub installation_year: Option<i32>,
    /// 0 = north, clockwise.
    pub azimuth_deg: f64,
    pub tilt_deg: f64,
    pub mounting: MountingAnswer,
    /// F_sh;obst (one or twelve values); exclusive with `shading`.
    #[serde(default)]
    pub obstruction_factors: Option<Vec<f64>>,
    /// ISSO 82.1 §15.4.7 / 75.1 §15.4.7 with table 16.1: minimal, side
    /// obstruction(s), roof edges (flat roofs), full or other obstruction,
    /// or factors of the extended method (NTA 17.3.8). Absent without
    /// `obstructionFactors`: minimal (table 16.1, nothing present).
    #[serde(default)]
    pub shading: Option<crate::solar_shading::CollectorObstruction>,
    pub source_reference: String,
}

fn crystalline(mono: bool, year: i32) -> &'static str {
    match (mono, year) {
        (true, y) if y < 2001 => "monocrystalline_before2001",
        (true, y) if y <= 2010 => "monocrystalline2001_to2010",
        (true, y) if y <= 2014 => "monocrystalline2011_to2014",
        (true, y) if y <= 2017 => "monocrystalline2015_to2017",
        (true, _) => "monocrystalline_from2018",
        (false, y) if y < 2001 => "multicrystalline_before2001",
        (false, y) if y <= 2010 => "multicrystalline2001_to2010",
        (false, y) if y <= 2014 => "multicrystalline2011_to2014",
        (false, y) if y <= 2017 => "multicrystalline2015_to2017",
        (false, _) => "multicrystalline_from2018",
    }
}

pub fn derive_pv(pv: &SurveyPv, construction_year: i32, recorder: &mut Recorder) -> Value {
    let path = format!("pv[{}]", pv.id);
    let crystalline_year = |recorder: &mut Recorder| {
        pv.installation_year.unwrap_or_else(|| {
            let year = construction_year.max(2000);
            recorder.record(
                "pv_year_unknown_construction_year",
                &path,
                year.to_string(),
                "ISSO 82.1 p. 191 (table 15.7)",
            );
            year
        })
    };
    let module = match pv.module_type {
        PvTypeAnswer::Monocrystalline => crystalline(true, crystalline_year(recorder)),
        PvTypeAnswer::Polycrystalline => crystalline(false, crystalline_year(recorder)),
        PvTypeAnswer::Unknown => {
            let year = pv.installation_year.unwrap_or(2000);
            recorder.record(
                "pv_type_unknown_polycrystalline",
                &path,
                match pv.installation_year {
                    Some(year) => format!("polycrystalline, installed {year}"),
                    None => "polycrystalline, placed before 2001".into(),
                },
                "ISSO 82.1 p. 191 (table 15.7)",
            );
            crystalline(false, year)
        }
        PvTypeAnswer::AmorphousSingleJunction => "amorphous_single_junction",
        PvTypeAnswer::AmorphousMultiJunction => "amorphous_multi_junction",
        PvTypeAnswer::AmorphousUnknown => {
            recorder.record(
                "pv_amorphous_unknown_multi_junction",
                &path,
                "multi-junction".into(),
                "ISSO 82.1 p. 191 (table 15.7)",
            );
            "amorphous_multi_junction"
        }
        PvTypeAnswer::Cigs => "cigs",
        PvTypeAnswer::CdTe => "cd_te",
    };
    let mounting = match pv.mounting {
        MountingAnswer::NotVentilated => "not_ventilated",
        MountingAnswer::ModeratelyVentilated => "moderately_ventilated",
        MountingAnswer::StronglyVentilated => "strongly_ventilated",
        MountingAnswer::Unknown => {
            recorder.record(
                "pv_mounting_unknown_not_ventilated",
                &path,
                "not_ventilated".into(),
                "ISSO 82.1 p. 191",
            );
            "not_ventilated"
        }
    };
    let mut value = json!({
        "id": pv.id,
        "peakPower": {"method": "table16_1", "moduleType": module, "panelAreaM2": pv.panel_area_m2},
        "azimuthDeg": pv.azimuth_deg,
        "tiltDeg": pv.tilt_deg,
        "mounting": mounting,
        "sourceReference": pv.source_reference,
    });
    match (&pv.shading, &pv.obstruction_factors) {
        (Some(_), Some(_)) => {
            recorder.issue("pv_shading_declared_twice", format!("{path}.shading"))
        }
        (Some(shading), None) => {
            value["obstruction"] = serde_json::to_value(shading).expect("serializable");
        }
        (None, Some(factors)) => value["obstructionFactors"] = json!(factors),
        (None, None) => {
            recorder.record(
                "pv_shading_not_entered_minimal",
                &path,
                "minimal".into(),
                "ISSO 82.1 p. 195-196 (table 16.1)",
            );
            value["obstruction"] = json!({"method": "minimal"});
        }
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pv_shading_maps_onto_the_collector_situations() {
        use crate::solar_shading::CollectorObstruction;
        let survey = |shading: Option<CollectorObstruction>, factors: Option<Vec<f64>>| SurveyPv {
            id: "pv".into(),
            panel_area_m2: 10.0,
            module_type: PvTypeAnswer::Unknown,
            installation_year: Some(2020),
            azimuth_deg: 180.0,
            tilt_deg: 15.0,
            mounting: MountingAnswer::NotVentilated,
            obstruction_factors: factors,
            shading,
            source_reference: "survey".into(),
        };
        // Nothing entered: minimal obstruction (table 16.1).
        let mut recorder = Recorder::default();
        let value = derive_pv(&survey(None, None), 2000, &mut recorder);
        assert_eq!(value["obstruction"]["method"], "minimal");
        assert!(value.get("obstructionFactors").is_none());
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "pv_shading_not_entered_minimal"));
        // A roof edge of 0,8 m at 0,5 m passes to the kernel unchanged.
        let edge = CollectorObstruction::RoofEdge {
            height_m: 0.8,
            distance_m: 0.5,
        };
        let mut recorder = Recorder::default();
        let value = derive_pv(&survey(Some(edge), None), 2000, &mut recorder);
        assert_eq!(value["obstruction"]["method"], "roof_edge");
        assert_eq!(value["obstruction"]["heightM"], 0.8);
        let system: crate::pv::PvSystem = serde_json::from_value(value).unwrap();
        assert!(crate::pv::validate_pv(&system, "pv").is_empty());
        // Both routes at once are an issue.
        let mut recorder = Recorder::default();
        derive_pv(
            &survey(Some(CollectorObstruction::Full), Some(vec![0.9])),
            2000,
            &mut recorder,
        );
        assert!(recorder
            .issues
            .iter()
            .any(|item| item.code == "pv_shading_declared_twice"));
    }

    #[test]
    fn storage_requires_pv() {
        let mut recorder = Recorder::default();
        let storage = SurveyStorage {
            electrical_kwh: 10.0,
            thermal_kwh: 0.0,
            source_reference: "survey".into(),
        };
        let (present, value) = derive_storage(Some(&storage), true, &mut recorder);
        assert!(present);
        assert_eq!(value.unwrap()["buildingBoundElectricalKwh"], 10.0);
        let (present, _) = derive_storage(Some(&storage), false, &mut recorder);
        assert!(!present);
        assert_eq!(recorder.issues[0].code, "storage_requires_pv");
    }

    #[test]
    fn unknown_type_year_and_mounting_follow_table_15_7() {
        let mut recorder = Recorder::default();
        let pv = SurveyPv {
            id: "pv".into(),
            panel_area_m2: 16.0,
            module_type: PvTypeAnswer::Unknown,
            installation_year: None,
            azimuth_deg: 180.0,
            tilt_deg: 35.0,
            mounting: MountingAnswer::Unknown,
            obstruction_factors: None,
            shading: None,
            source_reference: "survey".into(),
        };
        let value = derive_pv(&pv, 1995, &mut recorder);
        assert_eq!(
            value["peakPower"]["moduleType"],
            "multicrystalline_before2001"
        );
        assert_eq!(value["mounting"], "not_ventilated");
        let system: crate::pv::PvSystem = serde_json::from_value(value).unwrap();
        assert!((system.peak_power.kw() - 115.0 * 16.0 / 1000.0).abs() < 1e-9);
        // Type and year, mounting and the minimal shading of table 16.1.
        assert_eq!(recorder.applied.len(), 3);
        // Type and year unknown: placed before 2001, whatever the
        // construction year.
        let newer = derive_pv(&pv, 2015, &mut recorder);
        assert_eq!(
            newer["peakPower"]["moduleType"],
            "multicrystalline_before2001"
        );
        // Known crystalline type: the construction year.
        let mono = SurveyPv {
            module_type: PvTypeAnswer::Monocrystalline,
            ..pv
        };
        let value = derive_pv(&mono, 2015, &mut recorder);
        assert_eq!(
            value["peakPower"]["moduleType"],
            "monocrystalline2015_to2017"
        );
    }
}
