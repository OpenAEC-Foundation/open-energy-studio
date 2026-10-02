//! ISSO 82.1 (7e druk) chapter 13: domestic hot water in the basic survey.
//!
//! - No hot-water system: electric instantaneous heater (p. 164).
//! - Gas appliance (table 13.6, p. 168): type unknown → bath geyser,
//!   Gaskeur unknown → none, CW class unknown → CW-4/5/6 (the kernel's
//!   default class 4); a kitchen geyser above 13 kW counts as a bath
//!   geyser (p. 169).
//! - Tap pipe length: shortest horizontal plus vertical distance (table
//!   13.15, p. 178); it is measured, there is no fixed default.
//! - Shower heat recovery: unknown → shower not connected (table 13.16,
//!   p. 179).
//! - Electric boiler (§13.4, p. 172–174): its vessel losses are determined
//!   separately. Label unknown → manufacture year; year unknown → the
//!   construction year (p. 174); location unknown → outside the thermal
//!   zone (table 13.10); a boiler in a kitchen cabinet used only for the
//!   kitchen may take 10 l (p. 174). Connections are not distinguished
//!   (p. 173; NTA 13.6.3 factor 2).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::Recorder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GasApplianceType {
    BathGeyser,
    Combi,
    KitchenGeyser,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GaskeurAnswer {
    None,
    Gaskeur,
    GaskeurCw,
    GaskeurHrCw,
    Unknown,
}

/// CW class of a gas appliance with Gaskeur (table 13.6, p. 168).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CwClassAnswer {
    /// Kitchen use, CW-1 or CW-1+.
    Cw1,
    Cw2,
    Cw3,
    Cw4To6,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HotWaterGeneratorAnswer {
    /// No hot-water system in the dwelling.
    None,
    GasAppliance {
        #[serde(rename = "applianceType")]
        appliance_type: GasApplianceType,
        gaskeur: GaskeurAnswer,
        /// Burner load on the higher heating value, kW.
        #[serde(default, rename = "burnerLoadKw")]
        burner_load_kw: Option<f64>,
        /// CW class when a Gaskeur is present; omitted means unknown.
        #[serde(default, rename = "cwClass")]
        cw_class: Option<CwClassAnswer>,
    },
    ElectricBoiler,
    ElectricInstantaneous,
    HeatPump {
        #[serde(rename = "exhaustAirSource")]
        exhaust_air_source: bool,
    },
    DistrictHeat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShowerRecoveryAnswer {
    None,
    Vertical,
    Horizontal,
    /// Not determinable: shower not connected (table 13.16).
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TapsServed {
    KitchenAndBathroom,
    BathroomOnly,
    KitchenOnly,
}

/// Vessel of a residential electric boiler (ISSO 82.1 §13.4).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyBoilerVessel {
    /// Storage volume, l; `None` only for a kitchen-cabinet boiler (10 l).
    #[serde(default)]
    pub volume_l: Option<f64>,
    /// Built into a kitchen cabinet and used only for the kitchen (p. 174).
    #[serde(default)]
    pub kitchen_cabinet: bool,
    /// Energy label (vessels ≤ 500 l); `None` follows the manufacture year.
    #[serde(default)]
    pub label: Option<crate::domestic_hot_water::StorageLabel>,
    #[serde(default)]
    pub manufacture_year: Option<i32>,
    /// `None` unknown: outside the thermal zone (table 13.10).
    #[serde(default)]
    pub in_heated_zone: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyHotWater {
    pub generator: HotWaterGeneratorAnswer,
    pub served: TapsServed,
    #[serde(default)]
    pub kitchen_length_m: Option<f64>,
    #[serde(default)]
    pub bathroom_length_m: Option<f64>,
    #[serde(default = "one")]
    pub showers: u32,
    pub shower_heat_recovery: ShowerRecoveryAnswer,
    /// Vessel of an electric boiler (residential survey).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boiler_vessel: Option<SurveyBoilerVessel>,
    pub source_reference: String,
}

fn one() -> u32 {
    1
}

/// Storage of a residential electric boiler (table 13.10, p. 173–174).
pub fn boiler_storage(
    vessel: Option<&SurveyBoilerVessel>,
    construction_year: i32,
    reference: &str,
    recorder: &mut Recorder,
) -> Option<Value> {
    let default = SurveyBoilerVessel::default();
    let vessel = vessel.unwrap_or(&default);
    let path = "hotWater.boilerVessel";
    let volume = match vessel.volume_l {
        Some(volume) if volume.is_finite() && volume > 0.0 => volume,
        Some(_) => {
            recorder.issue("boiler_volume_invalid", format!("{path}.volumeL"));
            return None;
        }
        None if vessel.kitchen_cabinet => {
            recorder.record(
                "kitchen_boiler_forfait_10_l",
                path,
                "10 l".into(),
                "ISSO 82.1 p. 174",
            );
            10.0
        }
        None => {
            recorder.issue("boiler_volume_required", format!("{path}.volumeL"));
            return None;
        }
    };
    let loss = match vessel.label {
        Some(label) => json!({"method": "label", "label": label}),
        None => {
            let year = vessel.manufacture_year.unwrap_or_else(|| {
                recorder.record(
                    "vessel_year_unknown_construction_year",
                    path,
                    construction_year.to_string(),
                    "ISSO 82.1 p. 174",
                );
                construction_year
            });
            json!({"method": "unknown_label", "producedFrom2018": year >= 2018})
        }
    };
    let inside = vessel.in_heated_zone.unwrap_or_else(|| {
        recorder.record(
            "vessel_location_unknown_outside",
            path,
            "outside the thermal zone".into(),
            "ISSO 82.1 p. 173 (table 13.10)",
        );
        false
    });
    Some(json!({
        "id": "boiler",
        "volumeL": volume,
        "loss": loss,
        // NTA 13.6.3: electric boilers use 2 regardless of insulation.
        "connectionFactor": 2,
        "inHeatedZone": inside,
        "sourceReference": reference,
    }))
}

pub fn derive_hot_water(survey: &SurveyHotWater, recorder: &mut Recorder) -> Value {
    let reference = survey.source_reference.as_str();
    let generator = match &survey.generator {
        HotWaterGeneratorAnswer::None => {
            recorder.record(
                "no_hot_water_system_electric_instantaneous",
                "hotWater.generator",
                "electric_instantaneous".into(),
                "ISSO 82.1 p. 164",
            );
            json!({"kind": "electric_instantaneous"})
        }
        HotWaterGeneratorAnswer::GasAppliance {
            appliance_type,
            gaskeur,
            burner_load_kw,
            cw_class,
        } => {
            let mut kind = *appliance_type;
            if kind == GasApplianceType::Unknown {
                recorder.record(
                    "gas_appliance_type_unknown_bath_geyser",
                    "hotWater.generator.applianceType",
                    "bath_geyser".into(),
                    "ISSO 82.1 p. 168 (table 13.6)",
                );
                kind = GasApplianceType::BathGeyser;
            }
            if kind == GasApplianceType::KitchenGeyser && burner_load_kw.is_some_and(|kw| kw > 13.0)
            {
                recorder.record(
                    "kitchen_geyser_above_13_kw_bath_geyser",
                    "hotWater.generator.applianceType",
                    "bath_geyser".into(),
                    "ISSO 82.1 p. 169",
                );
                kind = GasApplianceType::BathGeyser;
            }
            let gaskeur = match gaskeur {
                GaskeurAnswer::Unknown => {
                    recorder.record(
                        "gaskeur_unknown_none",
                        "hotWater.generator.gaskeur",
                        "none".into(),
                        "ISSO 82.1 p. 168 (table 13.6)",
                    );
                    GaskeurAnswer::None
                }
                other => *other,
            };
            let appliance = match (kind, gaskeur) {
                (GasApplianceType::KitchenGeyser, _) => "kitchen_geyser",
                (_, GaskeurAnswer::None) => "without_gaskeur",
                (GasApplianceType::BathGeyser, GaskeurAnswer::Gaskeur) => "water_heater_gaskeur",
                (GasApplianceType::BathGeyser, _) => "water_heater_gaskeur_cw",
                (GasApplianceType::Combi, GaskeurAnswer::GaskeurHrCw) => "combi_gaskeur_hr_cw",
                (GasApplianceType::Combi, _) => "combi_gaskeur",
                (GasApplianceType::Unknown, _) => unreachable!("replaced above"),
            };
            // Table 13.6: the CW class counts only with a Gaskeur; unknown
            // is CW-4/5/6 (the kernel's class 4).
            let class = if gaskeur != GaskeurAnswer::None && kind != GasApplianceType::KitchenGeyser
            {
                match cw_class.unwrap_or(CwClassAnswer::Unknown) {
                    CwClassAnswer::Cw1 => Some("class1"),
                    CwClassAnswer::Cw2 => Some("class2"),
                    CwClassAnswer::Cw3 => Some("class3"),
                    CwClassAnswer::Cw4To6 => Some("class4"),
                    CwClassAnswer::Unknown => {
                        recorder.record(
                            "cw_class_unknown_cw_4_5_6",
                            "hotWater.generator.cwClass",
                            "class 4".into(),
                            "ISSO 82.1 p. 168 (table 13.6)",
                        );
                        Some("class4")
                    }
                }
            } else {
                None
            };
            let mut value = json!({
                "kind": "gas_appliance",
                "appliance": appliance,
                "kitchenOnly": survey.served == TapsServed::KitchenOnly,
            });
            if let Some(class) = class {
                value["measuredClass"] = json!(class);
            }
            value
        }
        HotWaterGeneratorAnswer::ElectricBoiler => json!({"kind": "electric_boiler"}),
        HotWaterGeneratorAnswer::ElectricInstantaneous => {
            json!({"kind": "electric_instantaneous"})
        }
        HotWaterGeneratorAnswer::HeatPump { exhaust_air_source } => {
            json!({"kind": "heat_pump", "exhaustAirSource": exhaust_air_source})
        }
        HotWaterGeneratorAnswer::DistrictHeat => json!({"kind": "external_heat"}),
    };
    let unit = match survey.shower_heat_recovery {
        ShowerRecoveryAnswer::None => None,
        ShowerRecoveryAnswer::Vertical => Some("vertical"),
        ShowerRecoveryAnswer::Horizontal => Some("horizontal"),
        ShowerRecoveryAnswer::Unknown => {
            recorder.record(
                "shower_heat_recovery_unknown_not_connected",
                "hotWater.showerHeatRecovery",
                "none".into(),
                "ISSO 82.1 p. 179 (table 13.16)",
            );
            None
        }
    };
    let served = match survey.served {
        TapsServed::KitchenAndBathroom => "kitchen_and_bathroom",
        TapsServed::BathroomOnly => "bathroom_only",
        TapsServed::KitchenOnly => "kitchen_only",
    };
    let mut emission = json!({
        "method": "residential",
        "served": served,
        "sourceReference": format!("{reference}; tap lengths per ISSO 82.1 p. 178"),
    });
    if let Some(length) = survey.kitchen_length_m {
        emission["kitchenLengthM"] = json!(length);
    }
    if let Some(length) = survey.bathroom_length_m {
        emission["bathroomLengthM"] = json!(length);
    }
    let mut system = json!({
        "need": {"method": "residential", "dwellingCount": 1, "sourceReference": reference},
        "emission": emission,
        "generator": generator,
        "equipmentReference": reference,
    });
    if let Some(unit) = unit {
        let mut showers = vec![json!({"unit": unit})];
        for _ in 1..survey.showers.max(1) {
            showers.push(json!({"unit": "none"}));
        }
        system["showerHeatRecovery"] = json!({
            "showers": showers,
            "connection": "unknown",
            "sourceReference": reference,
        });
    }
    system
}

#[cfg(test)]
mod tests {
    use super::*;

    fn survey(generator: HotWaterGeneratorAnswer) -> SurveyHotWater {
        SurveyHotWater {
            generator,
            served: TapsServed::KitchenAndBathroom,
            kitchen_length_m: Some(3.0),
            bathroom_length_m: Some(5.0),
            showers: 1,
            shower_heat_recovery: ShowerRecoveryAnswer::Unknown,
            boiler_vessel: None,
            source_reference: "survey".into(),
        }
    }

    #[test]
    fn electric_boiler_vessel_defaults() {
        let mut recorder = Recorder::default();
        let kitchen = SurveyBoilerVessel {
            kitchen_cabinet: true,
            ..SurveyBoilerVessel::default()
        };
        let vessel = boiler_storage(Some(&kitchen), 1975, "survey", &mut recorder).unwrap();
        assert_eq!(vessel["volumeL"], 10.0);
        assert_eq!(vessel["loss"]["producedFrom2018"], false);
        assert_eq!(vessel["inHeatedZone"], false);
        assert_eq!(vessel["connectionFactor"], 2);
        assert!(boiler_storage(None, 1975, "survey", &mut recorder).is_none());
        assert_eq!(recorder.issues[0].code, "boiler_volume_required");
    }

    #[test]
    fn unknowns_follow_table_13_6_and_13_16() {
        let mut recorder = Recorder::default();
        let system = derive_hot_water(
            &survey(HotWaterGeneratorAnswer::GasAppliance {
                appliance_type: GasApplianceType::Unknown,
                gaskeur: GaskeurAnswer::Unknown,
                burner_load_kw: None,
                cw_class: None,
            }),
            &mut recorder,
        );
        assert_eq!(system["generator"]["appliance"], "without_gaskeur");
        assert!(system.get("showerHeatRecovery").is_none());
        let rules: Vec<_> = recorder.applied.iter().map(|item| item.rule).collect();
        assert!(rules.contains(&"gas_appliance_type_unknown_bath_geyser"));
        assert!(rules.contains(&"gaskeur_unknown_none"));
        assert!(rules.contains(&"shower_heat_recovery_unknown_not_connected"));
    }

    #[test]
    fn no_system_and_large_kitchen_geyser() {
        let mut recorder = Recorder::default();
        let system = derive_hot_water(&survey(HotWaterGeneratorAnswer::None), &mut recorder);
        assert_eq!(system["generator"]["kind"], "electric_instantaneous");
        let geyser = derive_hot_water(
            &survey(HotWaterGeneratorAnswer::GasAppliance {
                appliance_type: GasApplianceType::KitchenGeyser,
                gaskeur: GaskeurAnswer::Gaskeur,
                burner_load_kw: Some(17.0),
                cw_class: None,
            }),
            &mut recorder,
        );
        assert_eq!(geyser["generator"]["appliance"], "water_heater_gaskeur");
        assert_eq!(geyser["generator"]["measuredClass"], "class4");
    }

    #[test]
    fn cw_class_sets_the_measured_class() {
        let mut recorder = Recorder::default();
        let combi = |cw_class| HotWaterGeneratorAnswer::GasAppliance {
            appliance_type: GasApplianceType::Combi,
            gaskeur: GaskeurAnswer::GaskeurHrCw,
            burner_load_kw: None,
            cw_class,
        };
        let known = derive_hot_water(&survey(combi(Some(CwClassAnswer::Cw2))), &mut recorder);
        assert_eq!(known["generator"]["appliance"], "combi_gaskeur_hr_cw");
        assert_eq!(known["generator"]["measuredClass"], "class2");
        assert!(!recorder
            .applied
            .iter()
            .any(|item| item.rule == "cw_class_unknown_cw_4_5_6"));
        let unknown = derive_hot_water(&survey(combi(None)), &mut recorder);
        assert_eq!(unknown["generator"]["measuredClass"], "class4");
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "cw_class_unknown_cw_4_5_6"));
        // Without a Gaskeur the CW class is not used.
        let plain = derive_hot_water(
            &survey(HotWaterGeneratorAnswer::GasAppliance {
                appliance_type: GasApplianceType::Combi,
                gaskeur: GaskeurAnswer::None,
                burner_load_kw: None,
                cw_class: Some(CwClassAnswer::Cw2),
            }),
            &mut recorder,
        );
        assert!(plain["generator"].get("measuredClass").is_none());
    }
}
