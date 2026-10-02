//! ISSO 82.1 (7e druk) chapter 15: PV in the basic survey.
//!
//! - PV type unknown: polycrystalline; installation year unknown: the
//!   construction year (a year before 2001 counts as 2000); amorphous of
//!   unknown type: multi-junction (table 15.7, p. 191).
//! - Building integration unknown: not ventilated (p. 191), the kernel's
//!   `f_perf` 0,76.
//! - East/west installations are two systems (p. 191).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::Recorder;

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
    /// F_sh;obst (one or twelve values); absent: 1,0.
    #[serde(default)]
    pub obstruction_factors: Option<Vec<f64>>,
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
    let year = pv.installation_year.unwrap_or_else(|| {
        let year = construction_year.max(2000);
        recorder.record(
            "pv_year_unknown_construction_year",
            &path,
            year.to_string(),
            "ISSO 82.1 p. 191 (table 15.7)",
        );
        year
    });
    let module = match pv.module_type {
        PvTypeAnswer::Monocrystalline => crystalline(true, year),
        PvTypeAnswer::Polycrystalline => crystalline(false, year),
        PvTypeAnswer::Unknown => {
            recorder.record(
                "pv_type_unknown_polycrystalline",
                &path,
                "polycrystalline".into(),
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
    json!({
        "id": pv.id,
        "peakPower": {"method": "table16_1", "moduleType": module, "panelAreaM2": pv.panel_area_m2},
        "azimuthDeg": pv.azimuth_deg,
        "tiltDeg": pv.tilt_deg,
        "mounting": mounting,
        "obstructionFactors": pv.obstruction_factors.clone().unwrap_or_else(|| vec![1.0]),
        "sourceReference": pv.source_reference,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(recorder.applied.len(), 3);
    }
}
