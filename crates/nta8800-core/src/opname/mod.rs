//! Basic survey (basisopname) of existing dwellings, ISSO 82.1 (7e druk,
//! 2025) with erratum 2026-01-06 and Wijzigingsdocument 82.1 2025 v1.1.
//!
//! A survey records what an energy advisor establishes on site, with
//! "unknown" where the advisor cannot determine a property. The layer
//! applies the ISSO recognition rules and defaults, builds the kernel's
//! [`BuildingPerformanceInput`] (demand with chapter 11 ventilation,
//! heating chain, hot water, PV) and lists every applied default with its
//! ISSO page for the dossier. ISSO prose is not reproduced; values are
//! transcribed and cited.
//!
//! Not covered (rejected with a code or reported as a warning): cooling,
//! collective installations, CHP, solar water heating, several heating
//! generators,
//! pipes in unheated spaces, sunrooms (AOS), detail-survey (detailopname)
//! routes and quality declarations other than a measured q_v10.

pub mod envelope;
pub mod general;
pub mod heating;
pub mod hot_water;
pub mod production;
pub mod ventilation;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::building_performance::{
    assess_building_performance, BuildingPerformanceAssessment, BuildingPerformanceInput,
};
use envelope::{SurfaceBoundary, SurveyEnvelope};
use general::{Construction, DwellingKind, DwellingType, Renovation};
use heating::SurveyHeating;
use hot_water::SurveyHotWater;
use production::SurveyPv;
use ventilation::SurveyVentilation;

pub const ISSO_SOURCE: &str =
    "ISSO 82.1 (7e druk, 2025) with erratum 2026-01-06 and Wijzigingsdocument 82.1 2025 v1.1";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedDefault {
    pub rule: &'static str,
    pub path: String,
    pub value: String,
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpnameWarning {
    pub code: &'static str,
    pub path: String,
    pub note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpnameIssue {
    pub code: &'static str,
    pub path: String,
}

/// Collects applied defaults, warnings and issues.
#[derive(Debug, Default)]
pub struct Recorder {
    pub applied: Vec<AppliedDefault>,
    pub warnings: Vec<OpnameWarning>,
    pub issues: Vec<OpnameIssue>,
}

impl Recorder {
    pub fn record(&mut self, rule: &'static str, path: &str, value: String, source: &'static str) {
        self.applied.push(AppliedDefault {
            rule,
            path: path.to_string(),
            value,
            source,
        });
    }

    pub fn warning(&mut self, code: &'static str, path: &str, note: &'static str) {
        self.warnings.push(OpnameWarning {
            code,
            path: path.to_string(),
            note,
        });
    }

    pub fn issue(&mut self, code: &'static str, path: impl Into<String>) {
        self.issues.push(OpnameIssue {
            code,
            path: path.into(),
        });
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeasuredInfiltration {
    /// NEN 2686, dm³/(s·m²) at 10 Pa; at most one year old unless
    /// registered (p. 55–56).
    pub qv10_dm3_per_s_m2: f64,
    pub source_reference: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResidentialSurvey {
    pub id: String,
    /// Year of the permit application, else of granting, else of
    /// completion (p. 52).
    pub construction_year: i32,
    #[serde(default)]
    pub renovation: Option<Renovation>,
    pub dwelling: DwellingType,
    /// A_g per NEN 2580 (p. 60).
    pub usable_floor_area_m2: f64,
    pub area_source_reference: String,
    /// Lowest ground level to the highest point (p. 57–59).
    pub building_height_m: f64,
    pub construction: Construction,
    #[serde(default)]
    pub measured_infiltration: Option<MeasuredInfiltration>,
    pub envelope: SurveyEnvelope,
    pub heating: SurveyHeating,
    pub hot_water: SurveyHotWater,
    pub ventilation: SurveyVentilation,
    #[serde(default)]
    pub pv: Vec<SurveyPv>,
    /// Building-bound cooling present; not covered by this layer yet.
    #[serde(default)]
    pub cooling_present: bool,
    pub source_reference: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpnameAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub source: &'static str,
    pub applied_defaults: Vec<AppliedDefault>,
    pub warnings: Vec<OpnameWarning>,
    pub issues: Vec<OpnameIssue>,
    pub derived_input: Option<BuildingPerformanceInput>,
    pub performance: Option<BuildingPerformanceAssessment>,
    pub reference_verified: bool,
}

fn validate(survey: &ResidentialSurvey, recorder: &mut Recorder) {
    if survey.id.trim().is_empty() {
        recorder.issue("survey_id_required", "id");
    }
    if !(1600..=2100).contains(&survey.construction_year) {
        recorder.issue("construction_year_invalid", "constructionYear");
    }
    if !(survey.usable_floor_area_m2.is_finite() && survey.usable_floor_area_m2 > 0.0) {
        recorder.issue("usable_floor_area_invalid", "usableFloorAreaM2");
    }
    if !(survey.building_height_m.is_finite() && survey.building_height_m > 0.0) {
        recorder.issue("building_height_invalid", "buildingHeightM");
    }
    if survey.cooling_present {
        recorder.issue("cooling_not_supported_in_basisopname", "coolingPresent");
    }
    for (field, value) in [
        ("sourceReference", &survey.source_reference),
        ("areaSourceReference", &survey.area_source_reference),
    ] {
        if value.trim().is_empty() {
            recorder.issue("source_reference_required", field);
        }
    }
    if let Some(renovation) = &survey.renovation {
        if renovation.evidence_reference.trim().is_empty() {
            recorder.issue(
                "renovation_evidence_required",
                "renovation.evidenceReference",
            );
        }
    }
    let hot = &survey.hot_water;
    let kitchen = matches!(
        hot.served,
        hot_water::TapsServed::KitchenAndBathroom | hot_water::TapsServed::KitchenOnly
    );
    let bathroom = matches!(
        hot.served,
        hot_water::TapsServed::KitchenAndBathroom | hot_water::TapsServed::BathroomOnly
    );
    if kitchen && hot.kitchen_length_m.is_none() {
        recorder.issue("tap_length_required", "hotWater.kitchenLengthM");
    }
    if bathroom && hot.bathroom_length_m.is_none() {
        recorder.issue("tap_length_required", "hotWater.bathroomLengthM");
    }
}

/// A_ls with f_ls of NTA 6.7.3 from the survey surfaces (gross areas).
fn loss_area(envelope: &SurveyEnvelope) -> f64 {
    envelope
        .surfaces
        .iter()
        .map(|surface| {
            let weight = match surface.boundary {
                SurfaceBoundary::Outdoor | SurfaceBoundary::UnheatedSpace { .. } => 1.0,
                SurfaceBoundary::Ground | SurfaceBoundary::Crawlspace => 0.7,
                SurfaceBoundary::AdjacentHeated => 0.0,
            };
            weight * surface.gross_area_m2
        })
        .sum()
}

/// Kernel input derived from the survey.
pub fn derive_residential_input(
    survey: &ResidentialSurvey,
    recorder: &mut Recorder,
) -> Option<Value> {
    validate(survey, recorder);
    if !recorder.issues.is_empty() {
        return None;
    }
    let year = survey.construction_year;
    let reference = survey.source_reference.as_str();
    let dwelling_kind = survey.dwelling.kind();
    let airtightness = general::airtightness_type(&survey.dwelling, recorder);
    let infiltration_year = general::infiltration_year(year, survey.renovation.as_ref(), recorder);
    let (floor, wall, ceiling) = general::thermal_mass(&survey.construction);
    let envelope = envelope::derive_envelope(&survey.envelope, year, recorder);
    let ventilation = ventilation::derive_ventilation(
        &survey.ventilation,
        dwelling_kind,
        year,
        infiltration_year,
        survey.usable_floor_area_m2,
        survey.building_height_m,
        envelope.floor_above_crawlspace,
        serde_json::to_value(airtightness)
            .ok()?
            .as_str()
            .unwrap_or_default(),
        survey
            .measured_infiltration
            .as_ref()
            .map(|item| item.qv10_dm3_per_s_m2),
        recorder,
    );
    let heating = heating::derive_heating(&survey.heating, year, recorder);
    let hot_water = hot_water::derive_hot_water(&survey.hot_water, recorder);
    let pv: Vec<Value> = survey
        .pv
        .iter()
        .map(|item| production::derive_pv(item, year, recorder))
        .collect();
    if !recorder.issues.is_empty() {
        return None;
    }
    let area = survey.usable_floor_area_m2;
    let demand = json!({
        "zoneId": "woning",
        "usableFloorAreaM2": area,
        "areaSourceReference": survey.area_source_reference,
        "usageFunction": "residential",
        "dwellingType": if dwelling_kind == DwellingKind::Apartment { "apartment_building" } else { "other" },
        "setpoints": {"heatingC": 20.0, "coolingC": 24.0, "sourceReference": "NTA 8800 table 7.13 residential"},
        "transmission": {
            "method": "components",
            "direct": {"elements": envelope.direct_elements, "linearBridges": [], "pointBridges": []},
            "unheated": envelope.unheated,
            "groundFloors": envelope.ground_floors,
            "groundInventoryConfirmed": true,
        },
        "ventilationFlows": [],
        "ventilation": ventilation.input,
        "thermalMass": {
            "floor": floor, "wall": wall, "ceiling": ceiling,
            "sourceReference": format!("{}; ISSO 82.1 tables 7.5/7.6", survey.construction.source_reference),
        },
        "internalGains": {"method": "residential", "dwellingCount": 1, "sourceReference": "one dwelling"},
        "windowInventoryComplete": true,
        "windows": envelope.windows,
        "opaqueInventoryComplete": true,
        "opaqueElements": envelope.opaque_elements,
    });
    let mut chain = json!({
        "demand": demand,
        "emission": heating.emission,
        "distribution": {"method": "heated_zone_only_space_heating", "sourceReference": format!("{reference}; basisopname: pipes in the heated zone")},
        "generator": heating.generator,
    });
    if let Some(system) = heating.distribution_system {
        chain["distributionSystem"] = system;
    }
    let mut input = json!({
        "calculationScope": "residential",
        "totalUsableFloorAreaM2": area,
        "areaSourceReference": survey.area_source_reference,
        "spaceHeating": chain,
        "bacsFactor": 1.0,
        "bacsSourceReference": "residential: f_BACS 1,0 (§5.5.8)",
        "useInventoryComplete": true,
        "declaredUses": [],
        "productionInventoryComplete": true,
        "onSiteProduction": [],
        "pvSystems": pv,
        "hotWater": hot_water,
        "lossAreaM2": loss_area(&survey.envelope),
        "lossAreaSourceReference": "basisopname: survey surfaces with f_ls (NTA 6.7.3)",
        "demandUsesFixedC1Ventilation": false,
        "batteryStoragePresent": false,
    });
    if let Some(renewable) = heating.heat_pump_renewable {
        input["heatPumpRenewable"] = renewable;
    }
    Some(input)
}

/// Survey → kernel input → building performance.
pub fn assess_residential_survey(survey: &ResidentialSurvey) -> OpnameAssessment {
    let mut recorder = Recorder::default();
    let derived = derive_residential_input(survey, &mut recorder);
    let derived_input =
        derived.and_then(
            |value| match serde_json::from_value::<BuildingPerformanceInput>(value) {
                Ok(input) => Some(input),
                Err(error) => {
                    recorder.issues.push(OpnameIssue {
                        code: "derived_input_shape_invalid",
                        path: error.to_string(),
                    });
                    None
                }
            },
        );
    let performance = derived_input.as_ref().map(assess_building_performance);
    let status = match &performance {
        Some(result) if result.status == "calculated_unverified" => "calculated_unverified",
        Some(_) => "derived_input_rejected",
        None => "invalid",
    };
    OpnameAssessment {
        status,
        scope: "isso_82_1_basisopname_residential_unverified",
        source: ISSO_SOURCE,
        applied_defaults: recorder.applied,
        warnings: recorder.warnings,
        issues: recorder.issues,
        derived_input,
        performance,
        reference_verified: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> ResidentialSurvey {
        let raw = match name {
            "1930" => include_str!("../../../../training-data/nta8800-opname-1930-terraced.json"),
            "1975" => include_str!("../../../../training-data/nta8800-opname-1975-apartment.json"),
            _ => include_str!("../../../../training-data/nta8800-opname-2015-detached.json"),
        };
        serde_json::from_str(raw).unwrap()
    }

    #[test]
    fn fixtures_calculate_with_documented_defaults() {
        for name in ["1930", "1975", "2015"] {
            let result = assess_residential_survey(&fixture(name));
            let performance = result.performance.as_ref();
            assert_eq!(
                result.status,
                "calculated_unverified",
                "{name}: {:?} {:?}",
                result.issues,
                performance.map(|item| &item.issues)
            );
            assert!(!result.applied_defaults.is_empty());
            let performance = performance.unwrap();
            assert!(
                performance
                    .primary_fossil_indicator_kwh_per_m2_year
                    .unwrap()
                    > 0.0
            );
            // Chapter 11 input gives the fixed C1 run, so BENG 1 is available.
            assert!(
                performance.need_indicator_kwh_per_m2_year.is_some(),
                "{name}"
            );
        }
    }

    #[test]
    fn older_dwelling_needs_more_than_newer() {
        let old = assess_residential_survey(&fixture("1930"));
        let new = assess_residential_survey(&fixture("2015"));
        let old_ep = old
            .performance
            .unwrap()
            .primary_fossil_indicator_kwh_per_m2_year
            .unwrap();
        let new_ep = new
            .performance
            .unwrap()
            .primary_fossil_indicator_kwh_per_m2_year
            .unwrap();
        assert!(old_ep > 2.0 * new_ep, "{old_ep} vs {new_ep}");
    }

    #[test]
    fn missing_tap_length_and_cooling_are_reported() {
        let mut survey = fixture("1975");
        survey.hot_water.kitchen_length_m = None;
        survey.cooling_present = true;
        let result = assess_residential_survey(&survey);
        assert_eq!(result.status, "invalid");
        let codes: Vec<_> = result.issues.iter().map(|item| item.code).collect();
        assert!(codes.contains(&"tap_length_required"));
        assert!(codes.contains(&"cooling_not_supported_in_basisopname"));
    }

    #[test]
    fn loss_area_weights_ground_with_0_7() {
        let survey = fixture("1930");
        let expected: f64 = survey
            .envelope
            .surfaces
            .iter()
            .map(|s| match s.boundary {
                SurfaceBoundary::Ground | SurfaceBoundary::Crawlspace => 0.7 * s.gross_area_m2,
                SurfaceBoundary::AdjacentHeated => 0.0,
                _ => s.gross_area_m2,
            })
            .sum();
        assert!((loss_area(&survey.envelope) - expected).abs() < 1e-9);
    }
}
