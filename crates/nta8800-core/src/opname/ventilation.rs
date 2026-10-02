//! ISSO 82.1 (7e druk) chapter 11: ventilation in the basic survey.
//!
//! - Dwellings without (adequate) ventilation provisions, or with only a
//!   bathroom fan on the light switch, are system A (p. 142–143).
//! - Self-regulating supply vents (tables 11.3/11.5, p. 142–144): presence
//!   unknown → installation year ≤ 2012 standard, ≥ 2013 Δp ≤ 1 Pa; present
//!   with unknown pressure class → ≤ 2003 5 < Δp ≤ 10 Pa, ≥ 2004 Δp ≤ 1 Pa.
//! - Controls unknown: no CO₂ measurement or control, no time control, no
//!   zoning (tables 11.4–11.6, p. 143–145).
//! - Heat recovery unknown: none (table 11.9, p. 150); counterflow of
//!   unknown material: aluminium (p. 151); supply-duct insulation unknown:
//!   not insulated, length unknown: 4 m (single family) or half the
//!   building height (apartment) (table 11.10, p. 150); constant volume
//!   unknown: none (table 11.11); bypass unknown per table 11.12 (p. 151).
//! - Duct airtightness unknown: 1,1 (table 11.13, p. 153).
//! - Fans (table 11.15, p. 154): manufacture year unknown → construction
//!   year; motor type unknown → installed ≤ 2006 AC, ≥ 2007 DC.
//! - Flow: without a documented capacity the regulatory minimum applies
//!   (p. 147), which is the kernel's 11.56 route.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::general::DwellingKind;
use super::Recorder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VentilationPrinciple {
    /// Natural supply and exhaust, or no adequate provisions.
    Natural,
    MechanicalSupply,
    MechanicalExtract,
    Balanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureClass {
    AtMost1Pa,
    From1To5Pa,
    From5To10Pa,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExchangerAnswer {
    CounterFlowAluminium,
    CounterFlowPlastic,
    /// Counterflow, material unknown: aluminium (p. 151).
    CounterFlowUnknownMaterial,
    CrossFlow,
    PlateOrTube,
    Rotary,
    Enthalpy,
    HeatPipe,
    TwoElement,
    /// Type not determinable: no heat recovery (table 11.9).
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MotorAnswer {
    Ac,
    Dc,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyVentilation {
    pub principle: VentilationPrinciple,
    /// Explicit table 11.5 variant (e.g. "c4a") when controls are proven.
    #[serde(default)]
    pub declared_variant: Option<String>,
    /// All supply vents self-regulating; `None` when unknown.
    #[serde(default)]
    pub self_regulating_vents: Option<bool>,
    #[serde(default)]
    pub pressure_class: Option<PressureClass>,
    /// Installation year of the vents/unit.
    #[serde(default)]
    pub installation_year: Option<i32>,
    #[serde(default)]
    pub heat_recovery: Option<ExchangerAnswer>,
    /// Bypass present (`None` unknown).
    #[serde(default)]
    pub bypass_present: Option<bool>,
    #[serde(default)]
    pub unit_manufacture_year: Option<i32>,
    #[serde(default)]
    pub motor: Option<MotorAnswer>,
    pub source_reference: String,
}

fn pressure_variant(
    survey: &SurveyVentilation,
    construction_year: i32,
    prefix: char,
    recorder: &mut Recorder,
) -> String {
    let year = survey.installation_year.unwrap_or(construction_year);
    let class = match (survey.self_regulating_vents, survey.pressure_class) {
        (Some(false), _) => None,
        (Some(true), Some(class)) => Some(class),
        (Some(true), None) => {
            let class = if year <= 2003 {
                PressureClass::From5To10Pa
            } else {
                PressureClass::AtMost1Pa
            };
            recorder.record(
                "vent_pressure_class_unknown",
                "ventilation.pressureClass",
                format!("{class:?} (installation year {year})"),
                "ISSO 82.1 p. 142/144 (tables 11.3/11.5)",
            );
            Some(class)
        }
        (None, _) => {
            let class = (year >= 2013).then_some(PressureClass::AtMost1Pa);
            recorder.record(
                "self_regulating_vents_unknown",
                "ventilation.selfRegulatingVents",
                format!("{class:?} (installation year {year})"),
                "ISSO 82.1 p. 142/144 (tables 11.3/11.5)",
            );
            class
        }
    };
    let suffix = match class {
        None => "1",
        Some(PressureClass::AtMost1Pa) => "2a",
        Some(PressureClass::From1To5Pa) => "2b",
        Some(PressureClass::From5To10Pa) => "2c",
    };
    format!("{prefix}{suffix}")
}

pub struct DerivedVentilation {
    pub input: Value,
    pub exhaust_air_heat_pump_possible: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn derive_ventilation(
    survey: &SurveyVentilation,
    dwelling: DwellingKind,
    construction_year: i32,
    infiltration_year: Option<i32>,
    usable_floor_area_m2: f64,
    building_height_m: f64,
    floor_above_crawlspace: bool,
    airtightness_type: &str,
    measured_qv10: Option<f64>,
    recorder: &mut Recorder,
) -> DerivedVentilation {
    let reference = survey.source_reference.as_str();
    let variant = match &survey.declared_variant {
        Some(variant) => variant.clone(),
        None => {
            recorder.record(
                "ventilation_controls_unknown_none",
                "ventilation",
                "no CO₂, time control or zoning".into(),
                "ISSO 82.1 p. 143–145 (tables 11.4–11.6)",
            );
            match survey.principle {
                VentilationPrinciple::Natural => {
                    pressure_variant(survey, construction_year, 'a', recorder)
                }
                VentilationPrinciple::MechanicalExtract => {
                    pressure_variant(survey, construction_year, 'c', recorder)
                }
                VentilationPrinciple::MechanicalSupply => "b1".into(),
                VentilationPrinciple::Balanced => "d1".into(),
            }
        }
    };
    let mechanical = survey.principle != VentilationPrinciple::Natural;
    let ducts = if mechanical {
        recorder.record(
            "duct_airtightness_unknown",
            "ventilation.ducts",
            "unknown (f_lea;du 1,1)".into(),
            "ISSO 82.1 p. 153 (table 11.13)",
        );
        "unknown"
    } else {
        "no_ducts"
    };
    let mut unit = json!({
        "variant": variant,
        "ducts": ducts,
        "equipmentReference": reference,
    });
    if survey.principle == VentilationPrinciple::Balanced {
        let exchanger = match survey.heat_recovery {
            None | Some(ExchangerAnswer::Unknown) => {
                recorder.record(
                    "heat_recovery_unknown_none",
                    "ventilation.heatRecovery",
                    "none".into(),
                    "ISSO 82.1 p. 150 (table 11.9)",
                );
                None
            }
            Some(ExchangerAnswer::CounterFlowUnknownMaterial) => {
                recorder.record(
                    "counterflow_material_unknown_aluminium",
                    "ventilation.heatRecovery",
                    "counter_flow_aluminium".into(),
                    "ISSO 82.1 p. 151",
                );
                Some("counter_flow_aluminium")
            }
            Some(ExchangerAnswer::CounterFlowAluminium) => Some("counter_flow_aluminium"),
            Some(ExchangerAnswer::CounterFlowPlastic) => Some("counter_flow_plastic"),
            Some(ExchangerAnswer::CrossFlow) => Some("cross_flow"),
            Some(ExchangerAnswer::PlateOrTube) => Some("plate_or_tube"),
            Some(ExchangerAnswer::Rotary) => Some("rotary"),
            Some(ExchangerAnswer::Enthalpy) => Some("enthalpy"),
            Some(ExchangerAnswer::HeatPipe) => Some("heat_pipe"),
            Some(ExchangerAnswer::TwoElement) => Some("two_element"),
        };
        if let Some(exchanger) = exchanger {
            if survey.declared_variant.is_none() {
                unit["variant"] = json!("d2");
            }
            recorder.record(
                "supply_duct_insulation_unknown",
                "ventilation.heatRecovery",
                "uninsulated; length by kernel default (4 m / ½ H)".into(),
                "ISSO 82.1 p. 150 (table 11.10)",
            );
            recorder.record(
                "constant_volume_unknown_none",
                "ventilation.heatRecovery",
                "false".into(),
                "ISSO 82.1 p. 151 (table 11.11)",
            );
            let bypass =
                json!({"kind": "unknown", "bypassPresent": survey.bypass_present.unwrap_or(false)});
            recorder.record(
                "bypass_table_11_12",
                "ventilation.bypass",
                "unknown → 100 % from 2010, 70 % with bypass, else 0 %".into(),
                "ISSO 82.1 p. 151 (table 11.12)",
            );
            let mut recovery = json!({
                "efficiency": {"method": "table", "exchanger": exchanger},
                "bypass": bypass,
                "layout": "central",
                "supplyDuctInsulation": {"kind": "uninsulated"},
                "equipmentReference": reference,
            });
            if let Some(year) = survey.unit_manufacture_year {
                recovery["manufactureYear"] = json!(year);
            }
            unit["heatRecovery"] = recovery;
        }
    }
    let fan_year = super::general::device_year(
        survey.unit_manufacture_year,
        survey.installation_year,
        construction_year,
        recorder,
        "ventilation.fans",
    );
    let current = match survey.motor.unwrap_or(MotorAnswer::Unknown) {
        MotorAnswer::Ac => "ac",
        MotorAnswer::Dc => "dc",
        MotorAnswer::Unknown => {
            let installed = survey.installation_year.unwrap_or(construction_year);
            let current = if installed <= 2006 { "ac" } else { "dc" };
            recorder.record(
                "fan_motor_unknown",
                "ventilation.motor",
                format!("{current} (installed {installed})"),
                "ISSO 82.1 p. 154 (table 11.15)",
            );
            current
        }
    };
    let infiltration = match measured_qv10 {
        Some(qv10) => {
            json!({"method": "measured", "qv10DmPerSM2": qv10, "sourceReference": reference})
        }
        None => {
            let mut value = json!({"method": "reference", "buildingType": airtightness_type});
            if let Some(year) = infiltration_year {
                value["renovationYear"] = json!(year);
            }
            value
        }
    };
    let input = json!({
        "zoneId": "woning",
        "usableFloorAreaM2": usable_floor_area_m2,
        "category": "residential",
        "functions": [{"function": "residential", "areaM2": usable_floor_area_m2}],
        "dwellingCount": 1,
        "apartmentBuilding": dwelling == DwellingKind::Apartment,
        "buildingHeightM": building_height_m,
        "constructionYear": construction_year,
        "floorAboveCrawlspace": floor_above_crawlspace,
        "heatingSetpointC": 20.0,
        "coolingSetpointC": 24.0,
        "system": {"kind": "single", "unit": unit},
        "infiltration": infiltration,
        "fans": {"method": "forfait", "current": current, "manufactureYear": fan_year},
        "sourceReference": format!("{reference}; basisopname"),
    });
    DerivedVentilation {
        input,
        exhaust_air_heat_pump_possible: matches!(
            survey.principle,
            VentilationPrinciple::MechanicalExtract | VentilationPrinciple::Balanced
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn survey(principle: VentilationPrinciple) -> SurveyVentilation {
        SurveyVentilation {
            principle,
            declared_variant: None,
            self_regulating_vents: None,
            pressure_class: None,
            installation_year: None,
            heat_recovery: None,
            bypass_present: None,
            unit_manufacture_year: None,
            motor: None,
            source_reference: "survey".into(),
        }
    }

    fn derive(survey: &SurveyVentilation, year: i32) -> Value {
        let mut recorder = Recorder::default();
        derive_ventilation(
            survey,
            DwellingKind::SingleFamily,
            year,
            None,
            100.0,
            9.0,
            false,
            "pitched_roof_terraced",
            None,
            &mut recorder,
        )
        .input
    }

    #[test]
    fn vent_rules_follow_tables_11_3_and_11_5() {
        let old = derive(&survey(VentilationPrinciple::MechanicalExtract), 1975);
        assert_eq!(old["system"]["unit"]["variant"], "c1");
        let new = derive(&survey(VentilationPrinciple::MechanicalExtract), 2015);
        assert_eq!(new["system"]["unit"]["variant"], "c2a");
        let mut present = survey(VentilationPrinciple::Natural);
        present.self_regulating_vents = Some(true);
        present.installation_year = Some(2000);
        assert_eq!(derive(&present, 1975)["system"]["unit"]["variant"], "a2c");
    }

    #[test]
    fn heat_recovery_and_fans_follow_tables_11_9_and_11_15() {
        let unknown = derive(&survey(VentilationPrinciple::Balanced), 2005);
        assert_eq!(unknown["system"]["unit"]["variant"], "d1");
        assert!(unknown["system"]["unit"].get("heatRecovery").is_none());
        assert_eq!(unknown["fans"]["current"], "ac");
        let mut wtw = survey(VentilationPrinciple::Balanced);
        wtw.heat_recovery = Some(ExchangerAnswer::CounterFlowUnknownMaterial);
        let derived = derive(&wtw, 2012);
        assert_eq!(derived["system"]["unit"]["variant"], "d2");
        assert_eq!(
            derived["system"]["unit"]["heatRecovery"]["efficiency"]["exchanger"],
            "counter_flow_aluminium"
        );
        assert_eq!(derived["fans"]["current"], "dc");
        assert_eq!(derived["system"]["unit"]["ducts"], "unknown");
    }
}
