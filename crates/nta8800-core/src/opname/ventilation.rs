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
//!   unknown: none (table 11.11); bypass unknown per table 11.12
//!   (p. 151-152): the unit manufacture year governs, the construction
//!   year is only the fallback.
//! - Duct airtightness unknown: 1,1 (table 11.13, p. 153).
//! - Fans (table 11.15, p. 154): manufacture year unknown → construction
//!   year; motor type unknown → installed ≤ 2006 AC, ≥ 2007 DC.
//! - Flow: without a documented capacity the regulatory minimum applies
//!   (p. 147), which is the kernel's 11.56 route.
//! - Passive cooling (p. 146, 152; ISSO 75.1 likewise): only with a supplier
//!   project document proving automatic control on the measured indoor and
//!   outdoor temperature, for systems B–E; D with heat recovery (and E) also
//!   needs a bypass. Without it τ_sysC = 0. The installed capacity including
//!   the extra capacity for passive cooling comes from the commissioning
//!   report (p. 146).

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
    /// Cold loading with an air handling unit (ISSO 75.1 table 11.9; NTA
    /// table 11.18, η 0,40).
    ColdStorageWithAhu,
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
    #[serde(default)]
    pub passive_cooling: Option<SurveyPassiveCooling>,
    /// Central or decentral heat recovery (table 11.6); `None`: central.
    #[serde(default)]
    pub heat_recovery_layout: Option<RecoveryLayout>,
    /// Controls of tables 11.4–11.6 (p. 143–145); `None` or unknown
    /// answers: no control, no zoning. `declaredVariant` overrides them.
    #[serde(default)]
    pub controls: Option<SurveyControls>,
    /// System E (§11.3.6, p. 145): decentral balanced units with heat
    /// recovery and CO₂ control in part of the zone. `principle` then
    /// describes the other part and `heatRecovery` the decentral units.
    #[serde(default)]
    pub combined: Option<SurveyCombined>,
    /// Supply grilles with electric heating strips (§11.3.7, p. 145–146).
    #[serde(default)]
    pub grille_heating_strips: Option<SurveyGrilleHeatingStrips>,
    /// Table 11.13 (ISSO 82.1 p. 152–153): duct airtightness class from a
    /// measurement or the recognition rules (LUKA A, B or C; LUKA D); `None`
    /// or unknown: f_lea;du 1,1.
    #[serde(default)]
    pub duct_airtightness: Option<super::utility::DuctAirtightnessAnswer>,
    pub source_reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryLayout {
    Central,
    Decentral,
}

/// Where CO₂ is measured (tables 11.4–11.6), in increasing coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Co2Measurement {
    None,
    LivingRoom,
    LivingRoomAndMainBedroom,
    EveryHabitableRoom,
}

/// What a control acts on (tables 11.4–11.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlTarget {
    None,
    Supply,
    Extract,
    SupplyAndExtract,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyControls {
    #[serde(default)]
    pub co2_measurement: Option<Co2Measurement>,
    #[serde(default)]
    pub co2_control: Option<ControlTarget>,
    #[serde(default)]
    pub time_control: Option<ControlTarget>,
    #[serde(default)]
    pub zoning: Option<bool>,
    /// System C: separate extract points in every habitable room (C.5b).
    #[serde(default)]
    pub extract_per_habitable_room: Option<bool>,
    /// Product documentation or commissioning report for the controls.
    pub evidence_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyCombined {
    /// Residence area ventilated by the decentral units, m².
    pub decentral_area_m2: f64,
    /// Total residence area of the zone, m².
    pub total_residence_area_m2: f64,
}

/// §11.3.7: the grille settings from product data; without all four the
/// kernel's 11.124 fallback applies. Share unknown: all grilles (p. 146).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyGrilleHeatingStrips {
    #[serde(default)]
    pub max_power_w_per_dm3_per_s: Option<f64>,
    #[serde(default)]
    pub max_temperature_rise_k: Option<f64>,
    #[serde(default)]
    pub switch_on_below_c: Option<f64>,
    #[serde(default)]
    pub max_supply_temperature_c: Option<f64>,
    pub source_reference: String,
}

/// ISSO 82.1 §11.4.1/§11.5.6 (p. 146, 152): proven passive cooling.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyPassiveCooling {
    /// Supplier project document proving automatic control on the measured
    /// indoor and outdoor temperature.
    pub evidence_reference: String,
    /// Installed capacity including the extra capacity for passive cooling,
    /// from the commissioning report, dm³/s; `None`: regulatory flow.
    #[serde(default)]
    pub installed_capacity_dm3_per_s: Option<f64>,
}

/// Maps a passive-cooling answer onto the kernel's ventilation input:
/// `maximumCapacityForCooling` (11.2.2.3.2) and `installedCapacity` (11.62).
pub(crate) fn apply_passive_cooling(
    input: &mut Value,
    passive: Option<&SurveyPassiveCooling>,
    principle: VentilationPrinciple,
    combined: bool,
    bypass_present: Option<bool>,
    recorder: &mut Recorder,
) {
    let Some(passive) = passive else {
        return;
    };
    // p. 152: passive cooling occurs with systems B to E; system E has the
    // decentral balanced part even when the other part is natural.
    if principle == VentilationPrinciple::Natural && !combined {
        recorder.issue(
            "passive_cooling_requires_mechanical_ventilation",
            "ventilation.passiveCooling",
        );
        return;
    }
    if passive.evidence_reference.trim().is_empty() {
        recorder.issue(
            "passive_cooling_evidence_required",
            "ventilation.passiveCooling.evidenceReference",
        );
        return;
    }
    // p. 152: with heat recovery the bypass is a precondition (p. 151).
    let recovery = input["system"]["unit"].get("heatRecovery").is_some()
        || input["system"]["decentral"].get("heatRecovery").is_some();
    if recovery && bypass_present != Some(true) {
        recorder.issue(
            "passive_cooling_requires_bypass",
            "ventilation.bypassPresent",
        );
        return;
    }
    input["maximumCapacityForCooling"] = json!(passive.evidence_reference);
    match passive.installed_capacity_dm3_per_s {
        Some(total) => {
            input["installedCapacity"] = json!({
                "totalDm3PerS": total,
                "sourceReference": passive.evidence_reference,
            });
        }
        None => recorder.record(
            "passive_cooling_capacity_unknown_regulatory",
            "ventilation.passiveCooling.installedCapacityDm3PerS",
            "regulatory design flow (no extra capacity)".into(),
            "ISSO 82.1 p. 146–147",
        ),
    }
}

/// Table 11.9 exchanger with its unknown defaults (p. 150–151).
fn exchanger_kind(
    answer: Option<ExchangerAnswer>,
    recorder: &mut Recorder,
) -> Option<&'static str> {
    match answer {
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
        Some(ExchangerAnswer::ColdStorageWithAhu) => Some("run_around_coil_ahu"),
    }
}

/// The kernel `heatRecovery` with the tables 11.10–11.12 defaults.
fn recovery_input(
    survey: &SurveyVentilation,
    exchanger: &str,
    layout: &str,
    construction_year: i32,
    recorder: &mut Recorder,
) -> Value {
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
    // Table 11.12 with p. 152: the year defaults apply only while the bypass
    // or its percentage is unknown. A bypass surveyed as absent is 0 %. The
    // manufacture year of the unit governs; the construction year is only
    // the fallback. The NTA 11.3.2.2 list reads "bouw- of fabricagejaar", so
    // the explicit fraction is passed to keep a pre-2010 unit in a newer
    // dwelling off the 100 % default.
    let bypass = if survey.bypass_present == Some(false) {
        recorder.record(
            "bypass_surveyed_absent",
            "ventilation.bypassPresent",
            "0 %".into(),
            "ISSO 82.1 p. 151 (table 11.12)",
        );
        json!({"kind": "none"})
    } else {
        let (bypass_year, year_source) = match survey.unit_manufacture_year {
            Some(year) => (year, "unit manufacture year"),
            None => (
                construction_year,
                "construction year (manufacture year unknown)",
            ),
        };
        let (bypass, share) = if bypass_year >= 2010 {
            (json!({"kind": "full"}), "100 %")
        } else if survey.bypass_present == Some(true) {
            (json!({"kind": "partial", "fraction": 0.7}), "70 %")
        } else {
            (json!({"kind": "none"}), "0 %")
        };
        recorder.record(
            "bypass_table_11_12",
            "ventilation.bypass",
            format!("{share} ({year_source} {bypass_year})"),
            "ISSO 82.1 p. 151-152 (table 11.12)",
        );
        bypass
    };
    let mut recovery = json!({
        "efficiency": {"method": "table", "exchanger": exchanger},
        "bypass": bypass,
        "layout": layout,
        "supplyDuctInsulation": {"kind": "uninsulated"},
        "equipmentReference": survey.source_reference,
    });
    if let Some(year) = survey.unit_manufacture_year {
        recovery["manufactureYear"] = json!(year);
    }
    recovery
}

/// NTA table 11.5 variant (residential rows) from the controls of ISSO
/// tables 11.4–11.6 (p. 143–145). Only a combination that is a row of
/// table 11.5 earns that variant; anything else falls back to the variant
/// without control.
fn controls_variant(
    survey: &SurveyVentilation,
    construction_year: i32,
    central_recovery: bool,
    recovery: bool,
    recorder: &mut Recorder,
) -> String {
    let controls = survey.controls.clone().unwrap_or_default();
    if survey.controls.is_none() {
        recorder.record(
            "ventilation_controls_unknown_none",
            "ventilation",
            "no CO₂, time control or zoning".into(),
            "ISSO 82.1 p. 143–145 (tables 11.4–11.6)",
        );
    }
    let measurement = controls.co2_measurement.unwrap_or(Co2Measurement::None);
    let co2 = controls.co2_control.unwrap_or(ControlTarget::None);
    let time = controls.time_control.unwrap_or(ControlTarget::None);
    let zoning = controls.zoning.unwrap_or(false);
    let co2_active = co2 != ControlTarget::None && measurement != Co2Measurement::None;
    let living_and_bedroom = measurement >= Co2Measurement::LivingRoomAndMainBedroom;
    let variant: String = match survey.principle {
        VentilationPrinciple::Natural => pressure_variant(survey, construction_year, 'a', recorder),
        VentilationPrinciple::MechanicalSupply => {
            if co2 == ControlTarget::Supply
                && measurement == Co2Measurement::EveryHabitableRoom
                && zoning
            {
                "b3".into()
            } else if time == ControlTarget::Supply && !zoning {
                "b2".into()
            } else {
                "b1".into()
            }
        }
        VentilationPrinciple::MechanicalExtract => {
            let pressure = pressure_variant(survey, construction_year, 'c', recorder);
            let low_pressure = pressure == "c2a";
            let extract_co2 = co2_active
                && matches!(
                    co2,
                    ControlTarget::Extract | ControlTarget::SupplyAndExtract
                );
            if low_pressure && extract_co2 && living_and_bedroom && zoning {
                if controls.extract_per_habitable_room == Some(true) {
                    "c5b".into()
                } else {
                    "c5a".into()
                }
            } else if low_pressure
                && co2 == ControlTarget::SupplyAndExtract
                && co2_active
                && living_and_bedroom
                && !zoning
            {
                "c4b".into()
            } else if low_pressure && extract_co2 && living_and_bedroom && !zoning {
                "c4c".into()
            } else if low_pressure && extract_co2 && !zoning {
                "c4a".into()
            } else if time == ControlTarget::SupplyAndExtract && !zoning {
                "c3c".into()
            } else if low_pressure && time == ControlTarget::Extract && !zoning {
                "c3b".into()
            } else if time == ControlTarget::Extract && !zoning {
                "c3a".into()
            } else {
                pressure
            }
        }
        VentilationPrinciple::Balanced => {
            let decentral = recovery && !central_recovery;
            if co2_active && living_and_bedroom && zoning {
                if decentral {
                    "d5b".into()
                } else {
                    "d5a".into()
                }
            } else if co2_active && living_and_bedroom && !zoning && central_recovery {
                "d5c".into()
            } else if co2_active && !zoning && central_recovery {
                "d3".into()
            } else if time != ControlTarget::None && zoning {
                "d4b".into()
            } else if time != ControlTarget::None {
                "d4a".into()
            } else if recovery {
                "d2".into()
            } else {
                "d1".into()
            }
        }
    };
    if let Some(given) = &survey.controls {
        if given.evidence_reference.trim().is_empty() {
            recorder.issue(
                "ventilation_controls_evidence_required",
                "ventilation.controls.evidenceReference",
            );
        }
        recorder.record(
            "ventilation_controls_table_11_5",
            "ventilation.controls",
            variant.clone(),
            "ISSO 82.1 p. 143–145 (tables 11.4–11.6); NTA table 11.5",
        );
    }
    variant
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
    // System E (§11.3.6, p. 145): `principle` describes the other part.
    let combined = survey.combined.as_ref();
    if combined.is_some() && survey.principle == VentilationPrinciple::Balanced {
        recorder.issue("combined_other_part_not_balanced", "ventilation.combined");
    }
    let recovery_unit = survey.principle == VentilationPrinciple::Balanced || combined.is_some();
    let exchanger = if recovery_unit {
        exchanger_kind(survey.heat_recovery, recorder)
    } else {
        None
    };
    // §11.3.6 (p. 145): the decentral part of system E has heat recovery,
    // so the table 11.9 "none" default cannot apply to it.
    if combined.is_some() && exchanger.is_none() {
        recorder.issue(
            "combined_requires_heat_recovery",
            "ventilation.heatRecovery",
        );
    }
    let central_recovery = exchanger.is_some()
        && combined.is_none()
        && survey.heat_recovery_layout != Some(RecoveryLayout::Decentral);
    let variant = match &survey.declared_variant {
        Some(variant) => variant.clone(),
        None => controls_variant(
            survey,
            construction_year,
            central_recovery,
            exchanger.is_some(),
            recorder,
        ),
    };
    let ducts = |principle: VentilationPrinciple, recorder: &mut Recorder| {
        use super::utility::DuctAirtightnessAnswer;
        if principle == VentilationPrinciple::Natural {
            return "no_ducts";
        }
        match survey.duct_airtightness {
            Some(DuctAirtightnessAnswer::LukaAbc) => return "luka_a_b_c",
            Some(DuctAirtightnessAnswer::LukaD) => return "luka_d",
            Some(DuctAirtightnessAnswer::NoDucts) => return "no_ducts",
            Some(DuctAirtightnessAnswer::Unknown) | None => {}
        }
        recorder.record(
            "duct_airtightness_unknown",
            "ventilation.ductAirtightness",
            "unknown (f_lea;du 1,1)".into(),
            "ISSO 82.1 p. 153 (table 11.13)",
        );
        "unknown"
    };
    let mut unit = json!({
        "variant": variant,
        "ducts": ducts(survey.principle, recorder),
        "equipmentReference": reference,
    });
    let recovery = exchanger.map(|exchanger| {
        let layout = if combined.is_some()
            || survey.heat_recovery_layout == Some(RecoveryLayout::Decentral)
        {
            "decentral"
        } else {
            "central"
        };
        recovery_input(survey, exchanger, layout, construction_year, recorder)
    });
    let system = match combined {
        Some(combined) => {
            recorder.record(
                "combined_system_e1",
                "ventilation.combined",
                "decentral part D.5b, other part per principle".into(),
                "ISSO 82.1 p. 145 (§11.3.6); NTA table 11.5 E.1",
            );
            let mut decentral = json!({
                "variant": "d5b",
                "ducts": "no_ducts",
                "equipmentReference": reference,
            });
            if let Some(recovery) = recovery {
                decentral["heatRecovery"] = recovery;
            }
            json!({
                "kind": "combined",
                "decentralAreaM2": combined.decentral_area_m2,
                "totalResidenceAreaM2": combined.total_residence_area_m2,
                "decentral": decentral,
                "other": unit,
            })
        }
        None => {
            if let Some(recovery) = recovery {
                unit["heatRecovery"] = recovery;
            }
            json!({"kind": "single", "unit": unit})
        }
    };
    // Table 11.15: fan manufacture year unknown → construction year. This
    // specific rule takes precedence over the general installation-year
    // fallback (and is the conservative one).
    let fan_year = survey.unit_manufacture_year.unwrap_or_else(|| {
        recorder.record(
            "fan_year_unknown_construction_year",
            "ventilation.fans",
            construction_year.to_string(),
            "ISSO 82.1 p. 154 (table 11.15; specific rule over p. 28)",
        );
        construction_year
    });
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
    let mut input = json!({
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
        "system": system,
        "infiltration": infiltration,
        "fans": {"method": "forfait", "current": current, "manufactureYear": fan_year},
        "sourceReference": format!("{reference}; basisopname"),
    });
    apply_passive_cooling(
        &mut input,
        survey.passive_cooling.as_ref(),
        survey.principle,
        survey.combined.is_some(),
        survey.bypass_present,
        recorder,
    );
    if let Some(strips) = &survey.grille_heating_strips {
        let control = match (
            strips.max_power_w_per_dm3_per_s,
            strips.max_temperature_rise_k,
            strips.switch_on_below_c,
            strips.max_supply_temperature_c,
        ) {
            (Some(power), Some(rise), Some(switch_on), Some(supply)) => json!({
                "method": "specified",
                "maxPowerWPerDm3PerS": power,
                "maxTemperatureRiseK": rise,
                "switchOnBelowC": switch_on,
                "maxSupplyTemperatureC": supply,
            }),
            _ => {
                recorder.record(
                    "grille_heating_strip_settings_unknown",
                    "ventilation.grilleHeatingStrips",
                    "NTA 11.124 fallback".into(),
                    "ISSO 82.1 p. 146 (§11.3.7)",
                );
                json!({"method": "fallback"})
            }
        };
        // p. 146: share of grilles unknown → all grilles have a strip; the
        // basic survey records no installed capacity, so no split.
        recorder.record(
            "grille_heating_strips_all_grilles",
            "ventilation.grilleHeatingStrips",
            "all grilles".into(),
            "ISSO 82.1 p. 146 (§11.3.7)",
        );
        input["grillePreheating"] = json!({
            "control": control,
            "sourceReference": strips.source_reference,
        });
    }
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
            passive_cooling: None,
            heat_recovery_layout: None,
            controls: None,
            combined: None,
            grille_heating_strips: None,
            duct_airtightness: None,
            source_reference: "survey".into(),
        }
    }

    fn derive_with(survey: &SurveyVentilation) -> (Value, Recorder) {
        let mut recorder = Recorder::default();
        let input = derive_ventilation(
            survey,
            DwellingKind::SingleFamily,
            2015,
            None,
            100.0,
            9.0,
            false,
            "pitched_roof_terraced",
            None,
            &mut recorder,
        )
        .input;
        (input, recorder)
    }

    #[test]
    fn duct_airtightness_class_replaces_the_unknown_default() {
        use super::super::utility::DuctAirtightnessAnswer;
        let unknown = survey(VentilationPrinciple::Balanced);
        let (input, recorder) = derive_with(&unknown);
        assert!(input.to_string().contains("\"ducts\":\"unknown\""));
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "duct_airtightness_unknown"));
        let mut luka = survey(VentilationPrinciple::Balanced);
        luka.duct_airtightness = Some(DuctAirtightnessAnswer::LukaAbc);
        let (input, recorder) = derive_with(&luka);
        assert!(input.to_string().contains("\"ducts\":\"luka_a_b_c\""));
        assert!(!recorder
            .applied
            .iter()
            .any(|item| item.rule == "duct_airtightness_unknown"));
    }

    #[test]
    fn passive_cooling_needs_evidence_and_a_bypass_with_heat_recovery() {
        let passive = SurveyPassiveCooling {
            evidence_reference: "supplier project document".into(),
            installed_capacity_dm3_per_s: Some(90.0),
        };
        // Without the answer τ_sysC stays 0.
        let (plain, _) = derive_with(&survey(VentilationPrinciple::MechanicalExtract));
        assert!(plain.get("maximumCapacityForCooling").is_none());
        // System C with proven control.
        let mut extract = survey(VentilationPrinciple::MechanicalExtract);
        extract.passive_cooling = Some(passive.clone());
        let (input, recorder) = derive_with(&extract);
        assert!(recorder.issues.is_empty());
        assert_eq!(
            input["maximumCapacityForCooling"],
            "supplier project document"
        );
        assert_eq!(input["installedCapacity"]["totalDm3PerS"], 90.0);
        let kernel: crate::ventilation::VentilationInput =
            serde_json::from_value(input).expect("kernel input");
        assert!(kernel.maximum_capacity_for_cooling.is_some());
        // Capacity unknown: regulatory flow, recorded.
        extract
            .passive_cooling
            .as_mut()
            .unwrap()
            .installed_capacity_dm3_per_s = None;
        let (input, recorder) = derive_with(&extract);
        assert!(input.get("installedCapacity").is_none());
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "passive_cooling_capacity_unknown_regulatory"));
        // System A cannot cool passively with fans.
        let mut natural = survey(VentilationPrinciple::Natural);
        natural.passive_cooling = Some(passive.clone());
        let (input, recorder) = derive_with(&natural);
        assert!(input.get("maximumCapacityForCooling").is_none());
        assert_eq!(
            recorder.issues[0].code,
            "passive_cooling_requires_mechanical_ventilation"
        );
        // D with heat recovery needs the bypass (p. 152).
        let mut balanced = survey(VentilationPrinciple::Balanced);
        balanced.heat_recovery = Some(ExchangerAnswer::CounterFlowPlastic);
        balanced.passive_cooling = Some(passive);
        let (_, recorder) = derive_with(&balanced);
        assert_eq!(recorder.issues[0].code, "passive_cooling_requires_bypass");
        balanced.bypass_present = Some(true);
        let (input, recorder) = derive_with(&balanced);
        assert!(recorder.issues.is_empty());
        assert!(input.get("maximumCapacityForCooling").is_some());
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

    fn controls(
        measurement: Co2Measurement,
        co2: ControlTarget,
        time: ControlTarget,
        zoning: bool,
    ) -> Option<SurveyControls> {
        Some(SurveyControls {
            co2_measurement: Some(measurement),
            co2_control: Some(co2),
            time_control: Some(time),
            zoning: Some(zoning),
            extract_per_habitable_room: None,
            evidence_reference: "product sheet".into(),
        })
    }

    #[test]
    fn controls_map_tables_11_4_to_11_6_onto_table_11_5_rows() {
        use Co2Measurement as M;
        use ControlTarget as T;
        let variant = |survey: &SurveyVentilation, year| {
            derive(survey, year)["system"]["unit"]["variant"].clone()
        };
        // Table 11.4: system B.
        let mut supply = survey(VentilationPrinciple::MechanicalSupply);
        supply.controls = controls(M::None, T::None, T::Supply, false);
        assert_eq!(variant(&supply, 2015), "b2");
        supply.controls = controls(M::EveryHabitableRoom, T::Supply, T::None, true);
        assert_eq!(variant(&supply, 2015), "b3");
        // Not a row (living room only): no control credit.
        supply.controls = controls(M::LivingRoom, T::Supply, T::None, true);
        assert_eq!(variant(&supply, 2015), "b1");
        // Table 11.6: system D.
        let mut balanced = survey(VentilationPrinciple::Balanced);
        balanced.heat_recovery = Some(ExchangerAnswer::CounterFlowPlastic);
        balanced.controls = controls(M::LivingRoom, T::Extract, T::None, false);
        assert_eq!(variant(&balanced, 2015), "d3");
        balanced.controls = controls(M::LivingRoomAndMainBedroom, T::Supply, T::None, false);
        assert_eq!(variant(&balanced, 2015), "d5c");
        balanced.controls = controls(M::LivingRoomAndMainBedroom, T::Supply, T::None, true);
        assert_eq!(variant(&balanced, 2015), "d5a");
        balanced.heat_recovery_layout = Some(RecoveryLayout::Decentral);
        assert_eq!(variant(&balanced, 2015), "d5b");
        balanced.controls = controls(M::None, T::None, T::Supply, true);
        assert_eq!(variant(&balanced, 2015), "d4b");
        balanced.controls = controls(M::None, T::None, T::None, false);
        assert_eq!(variant(&balanced, 2015), "d2");
        // Table 11.5 (ISSO): system C with self-regulating vents ≤ 1 Pa.
        let mut extract = survey(VentilationPrinciple::MechanicalExtract);
        extract.self_regulating_vents = Some(true);
        extract.pressure_class = Some(PressureClass::AtMost1Pa);
        extract.controls = controls(M::LivingRoom, T::Extract, T::None, false);
        assert_eq!(variant(&extract, 2015), "c4a");
        extract.controls = controls(M::LivingRoomAndMainBedroom, T::Extract, T::None, true);
        assert_eq!(variant(&extract, 2015), "c5a");
        extract.controls = controls(M::None, T::None, T::Extract, false);
        assert_eq!(variant(&extract, 2015), "c3b");
        // Controls without evidence are rejected.
        let mut recorder = Recorder::default();
        let mut missing = survey(VentilationPrinciple::MechanicalSupply);
        missing.controls = controls(M::None, T::None, T::Supply, false);
        missing.controls.as_mut().unwrap().evidence_reference = " ".into();
        derive_ventilation(
            &missing,
            DwellingKind::SingleFamily,
            2015,
            None,
            100.0,
            9.0,
            false,
            "pitched_roof_terraced",
            None,
            &mut recorder,
        );
        assert_eq!(
            recorder.issues[0].code,
            "ventilation_controls_evidence_required"
        );
    }

    #[test]
    fn system_e_combines_decentral_d5b_with_the_other_principle() {
        let mut combined = survey(VentilationPrinciple::MechanicalExtract);
        combined.heat_recovery = Some(ExchangerAnswer::CounterFlowPlastic);
        combined.combined = Some(SurveyCombined {
            decentral_area_m2: 30.0,
            total_residence_area_m2: 80.0,
        });
        let input = derive(&combined, 2015);
        assert_eq!(input["system"]["kind"], "combined");
        assert_eq!(input["system"]["decentral"]["variant"], "d5b");
        assert_eq!(
            input["system"]["decentral"]["heatRecovery"]["layout"],
            "decentral"
        );
        assert_eq!(input["system"]["other"]["variant"], "c2a");
        assert!(input["system"]["other"].get("heatRecovery").is_none());
        let parsed: crate::ventilation::VentilationInput =
            serde_json::from_value(input).expect("kernel input");
        assert!(crate::ventilation::validate_ventilation(&parsed).is_empty());
        // The other part cannot be balanced.
        combined.principle = VentilationPrinciple::Balanced;
        let (_, recorder) = derive_with(&combined);
        assert_eq!(recorder.issues[0].code, "combined_other_part_not_balanced");
        // §11.3.6: the decentral part has heat recovery; "unknown" (table
        // 11.9 default none) cannot describe system E.
        combined.principle = VentilationPrinciple::MechanicalExtract;
        combined.heat_recovery = Some(ExchangerAnswer::Unknown);
        let (_, recorder) = derive_with(&combined);
        assert_eq!(recorder.issues[0].code, "combined_requires_heat_recovery");
        // p. 152: passive cooling also with system E next to natural
        // ventilation, given the bypass on the decentral heat recovery.
        combined.principle = VentilationPrinciple::Natural;
        combined.heat_recovery = Some(ExchangerAnswer::CounterFlowPlastic);
        combined.passive_cooling = Some(SurveyPassiveCooling {
            evidence_reference: "supplier project document".into(),
            installed_capacity_dm3_per_s: None,
        });
        let (_, recorder) = derive_with(&combined);
        assert_eq!(recorder.issues[0].code, "passive_cooling_requires_bypass");
        combined.bypass_present = Some(true);
        let (input, recorder) = derive_with(&combined);
        assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
        assert!(input.get("maximumCapacityForCooling").is_some());
    }

    #[test]
    fn grille_heating_strips_use_the_settings_or_the_fallback() {
        let mut natural = survey(VentilationPrinciple::Natural);
        natural.grille_heating_strips = Some(SurveyGrilleHeatingStrips {
            max_power_w_per_dm3_per_s: Some(20.0),
            max_temperature_rise_k: Some(10.0),
            switch_on_below_c: Some(5.0),
            max_supply_temperature_c: Some(15.0),
            source_reference: "grille data".into(),
        });
        let input = derive(&natural, 2015);
        assert_eq!(input["grillePreheating"]["control"]["method"], "specified");
        natural
            .grille_heating_strips
            .as_mut()
            .unwrap()
            .switch_on_below_c = None;
        let input = derive(&natural, 2015);
        assert_eq!(input["grillePreheating"]["control"]["method"], "fallback");
        let parsed: crate::ventilation::VentilationInput =
            serde_json::from_value(input).expect("kernel input");
        assert!(crate::ventilation::validate_ventilation(&parsed).is_empty());
    }

    #[test]
    fn bypass_follows_the_unit_manufacture_year_before_the_construction_year() {
        let mut wtw = survey(VentilationPrinciple::Balanced);
        wtw.heat_recovery = Some(ExchangerAnswer::CounterFlowUnknownMaterial);
        let bypass = |survey: &SurveyVentilation, year| {
            derive(survey, year)["system"]["unit"]["heatRecovery"]["bypass"].clone()
        };
        // Manufacture year unknown: the construction year decides (p. 152).
        assert_eq!(bypass(&wtw, 2012), json!({"kind": "full"}));
        assert_eq!(bypass(&wtw, 2005), json!({"kind": "none"}));
        // A pre-2010 unit in a 2012 dwelling: the unit year governs.
        wtw.unit_manufacture_year = Some(2008);
        assert_eq!(bypass(&wtw, 2012), json!({"kind": "none"}));
        wtw.bypass_present = Some(true);
        assert_eq!(
            bypass(&wtw, 2012),
            json!({"kind": "partial", "fraction": 0.7})
        );
        wtw.unit_manufacture_year = Some(2011);
        assert_eq!(bypass(&wtw, 1990), json!({"kind": "full"}));
        // Surveyed absent: the year defaults of table 11.12 do not apply.
        wtw.bypass_present = Some(false);
        assert_eq!(bypass(&wtw, 2015), json!({"kind": "none"}));
        wtw.unit_manufacture_year = None;
        assert_eq!(bypass(&wtw, 2015), json!({"kind": "none"}));
    }
}
