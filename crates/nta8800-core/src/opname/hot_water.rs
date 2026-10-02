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
    pub source_reference: String,
}

fn one() -> u32 {
    1
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
            if gaskeur != GaskeurAnswer::None && kind != GasApplianceType::KitchenGeyser {
                recorder.record(
                    "cw_class_unknown_cw_4_5_6",
                    "hotWater.generator",
                    "class 4".into(),
                    "ISSO 82.1 p. 168 (table 13.6)",
                );
            }
            json!({
                "kind": "gas_appliance",
                "appliance": appliance,
                "kitchenOnly": survey.served == TapsServed::KitchenOnly,
            })
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
            source_reference: "survey".into(),
        }
    }

    #[test]
    fn unknowns_follow_table_13_6_and_13_16() {
        let mut recorder = Recorder::default();
        let system = derive_hot_water(
            &survey(HotWaterGeneratorAnswer::GasAppliance {
                appliance_type: GasApplianceType::Unknown,
                gaskeur: GaskeurAnswer::Unknown,
                burner_load_kw: None,
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
            }),
            &mut recorder,
        );
        assert_eq!(geyser["generator"]["appliance"], "water_heater_gaskeur");
    }
}
