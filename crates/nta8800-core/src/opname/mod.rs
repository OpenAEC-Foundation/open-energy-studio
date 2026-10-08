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
//! Sunrooms (AOS) count as outdoor air (ISSO 82.1 §6.3.4); caravans and
//! houseboats take the forfaits of NTA tables I.5–I.7 and light mass
//! (p. 62). Quality declarations: a measured q_v10 and rooflights with a
//! BCRG declaration (p. 68). Not covered: detail-survey (detailopname)
//! routes.

pub mod envelope;
pub mod general;
pub mod heating;
pub mod hot_water;
pub mod production;
pub mod utility;
pub mod ventilation;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::norm_versions::{self, NormVersion};

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
    /// The adviser's reason for falling back on this forfait ("inklappen",
    /// BRL 9500-W §4.2.2 p. 19 and Bijlage 3 p. 61), from the survey's
    /// `inklapRedenen` by path or rule.
    #[serde(rename = "inklapReden", skip_serializing_if = "Option::is_none")]
    pub collapse_reason: Option<String>,
}

/// Attaches the adviser's collapse reasons to the applied defaults; a key
/// matches the default's path or its rule. Unmatched keys are warned.
/// Collapse-reason keys (path or rule) of saved surveys that were renamed
/// or split; each old key also matches the listed new paths or rules.
const COLLAPSE_REASON_ALIASES: &[(&str, &[&str])] = &[
    (
        "cooling_fittings_unknown_uninsulated_meters_present",
        &[
            "cooling_fittings_unknown_uninsulated",
            "cooling_meters_unknown_present",
        ],
    ),
    (
        "cooling.distribution",
        &[
            "cooling.fittingsInsulated",
            "cooling.coldMeters",
            "cooling.pipeLengthM",
        ],
    ),
    (
        "ventilation.ductsLukaAbc",
        &["ventilation.ductAirtightness"],
    ),
    ("ventilation.bypass", &["ventilation.bypassPercent"]),
    (
        "ventilation.heatRecovery",
        &[
            "ventilation.supplyDuctInsulation",
            "ventilation.supplyDuctLengthM",
            "ventilation.constantVolumeControl",
        ],
    ),
];

fn key_matches(key: &str, path: &str, rule: &str) -> bool {
    key == path
        || key == rule
        || COLLAPSE_REASON_ALIASES
            .iter()
            .any(|(old, new)| *old == key && new.iter().any(|item| *item == path || *item == rule))
}

pub(crate) fn apply_collapse_reasons(
    recorder: &mut Recorder,
    reasons: &std::collections::BTreeMap<String, String>,
) {
    for item in recorder.applied.iter_mut() {
        item.collapse_reason = reasons
            .get(&item.path)
            .or_else(|| reasons.get(item.rule))
            .or_else(|| {
                reasons
                    .iter()
                    .find(|(key, _)| key_matches(key, &item.path, item.rule))
                    .map(|(_, reason)| reason)
            })
            .map(|reason| reason.trim().to_string())
            .filter(|reason| !reason.is_empty());
    }
    let unmatched: Vec<String> = reasons
        .keys()
        .filter(|key| {
            !recorder
                .applied
                .iter()
                .any(|item| key_matches(key, &item.path, item.rule))
        })
        .cloned()
        .collect();
    for key in unmatched {
        recorder.warning(
            "collapse_reason_unmatched",
            &format!("inklapRedenen.{key}"),
            "no applied default has this path or rule",
        );
    }
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
            collapse_reason: None,
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

/// Vertical pipe through the thermal envelope (§7.2.4, table 7.7).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SurveyVerticalPipe {
    /// `None`: not determinable, uninsulated (table 7.7).
    #[serde(default)]
    pub insulated: Option<bool>,
    /// Zones or adjacent heated spaces along the part bordering this zone,
    /// this zone included; `None`: not shared (NTA 7.3.3).
    #[serde(default)]
    pub shared_zones: Option<u32>,
}

/// §7.2.4 table 7.7 and NTA 7.3.3: the vertical pipes, with one
/// uninsulated pipe per storey of the zone when their number is unknown.
/// `default_count` is the number of uninsulated pipes when the pipes are
/// not determinable, with the rule recorded by the caller.
pub(crate) fn vertical_pipes(
    pipes: Option<&[SurveyVerticalPipe]>,
    storeys: u32,
    default_count: u32,
    reference: &str,
    recorder: &mut Recorder,
) -> Vec<Value> {
    let storeys = storeys.max(1);
    let defaulted;
    let pipes = match pipes {
        Some(pipes) => pipes,
        None => {
            // The validation bounds the counts; the cap keeps a missed
            // count from exhausting memory.
            defaulted = vec![SurveyVerticalPipe::default(); default_count.min(MAX_COUNT) as usize];
            &defaulted
        }
    };
    pipes
        .iter()
        .enumerate()
        .map(|(index, pipe)| {
            let insulated = pipe.insulated.unwrap_or_else(|| {
                recorder.record(
                    "vertical_pipe_insulation_unknown_uninsulated",
                    &format!("verticalPipes[{index}].insulated"),
                    "false".into(),
                    "ISSO 82.1 p. 63 (table 7.7)",
                );
                false
            });
            json!({
                "id": format!("leiding-{}", index + 1),
                "storeys": storeys,
                "insulated": insulated,
                "sharedZones": pipe.shared_zones.unwrap_or(1),
                "sourceReference": format!("{reference}; basisopname §7.2.4"),
            })
        })
        .collect()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResidentialSurvey {
    pub id: String,
    /// NTA 8800 edition to calculate with; default 2025+C1:2026. The ISSO
    /// survey protocol is always the 2025 edition, so an older edition gives
    /// a comparison only (`survey_protocol_edition_differs`).
    #[serde(default, skip_serializing_if = "NormVersion::is_default")]
    pub norm_version: NormVersion,
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
    /// Storeys of the dwelling (zone); `None`: the storeys served by the
    /// heating distribution.
    #[serde(default)]
    pub storeys: Option<u32>,
    /// Vertical pipes through the envelope (§7.2.4); `None`: not
    /// determinable (one uninsulated pipe per storey), empty: none present.
    #[serde(default)]
    pub vertical_pipes: Option<Vec<SurveyVerticalPipe>>,
    pub envelope: SurveyEnvelope,
    pub heating: SurveyHeating,
    pub hot_water: SurveyHotWater,
    /// Further hot-water systems of the dwelling, for example a kitchen
    /// geyser next to the bathroom appliance (ISSO 82.1 p. 164, NTA §13.2.4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub additional_hot_water_systems: Vec<SurveyHotWater>,
    pub ventilation: SurveyVentilation,
    #[serde(default)]
    pub pv: Vec<SurveyPv>,
    /// Building-bound electrical or thermal storage (§15.5).
    #[serde(default)]
    pub storage: Option<production::SurveyStorage>,
    /// Building-bound cooling present. Without `cooling` the survey is
    /// rejected, because chapter 10 needs the system data.
    #[serde(default)]
    pub cooling_present: bool,
    /// ISSO 82.1 chapter 10: the main cooling system of the dwelling.
    #[serde(default)]
    pub cooling: Option<utility::SurveyCooling>,
    /// The cooling generator is collective (several dwellings).
    #[serde(default)]
    pub cooling_collective: bool,
    pub source_reference: String,
    /// Reason per applied default (path or rule) for falling back on the
    /// forfait (BRL 9500-W §4.2.2).
    #[serde(default, rename = "inklapRedenen")]
    pub collapse_reasons: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpnameAssessment {
    pub status: &'static str,
    pub scope: &'static str,
    pub source: &'static str,
    /// Label of the NTA 8800 edition the survey was calculated with.
    pub target_norm_version: &'static str,
    /// Edition the survey was calculated with.
    pub norm_version: NormVersion,
    /// Only a 2025+C1:2026 calculation may be registered.
    pub registration_eligible: bool,
    pub applied_defaults: Vec<AppliedDefault>,
    pub warnings: Vec<OpnameWarning>,
    pub issues: Vec<OpnameIssue>,
    pub derived_input: Option<BuildingPerformanceInput>,
    pub performance: Option<BuildingPerformanceAssessment>,
    pub reference_verified: bool,
}

/// Upper bound for storey counts in a survey. A program choice, not a norm
/// value: the counts size the derived input (one vertical pipe per storey),
/// so an unbounded count could exhaust memory.
pub(crate) const MAX_STOREYS: u32 = 200;
/// Upper bound for other survey counts (dwellings, showers, emitters,
/// heaters, collectors). A program choice, not a norm value.
pub(crate) const MAX_COUNT: u32 = 100_000;

/// Records `code` at `path` when the count exceeds `max`.
pub(crate) fn check_count(
    recorder: &mut Recorder,
    value: Option<u32>,
    max: u32,
    code: &'static str,
    path: &str,
) {
    if value.is_some_and(|value| value > max) {
        recorder.issue(code, path.to_string());
    }
}

/// Range checks of the counts in the heating answers, shared by the
/// residential and the utility survey.
pub(crate) fn validate_heating_counts(heating: &SurveyHeating, recorder: &mut Recorder) {
    check_count(
        recorder,
        Some(heating.storeys),
        MAX_STOREYS,
        "storeys_out_of_range",
        "heating.storeys",
    );
    if let Some(collective) = &heating.collective {
        check_count(
            recorder,
            collective.connected_storeys,
            MAX_STOREYS,
            "storeys_out_of_range",
            "heating.collective.connectedStoreys",
        );
        check_count(
            recorder,
            collective.connected_dwellings,
            MAX_COUNT,
            "count_out_of_range",
            "heating.collective.connectedDwellings",
        );
    }
    if let Some(heating::DistributionTypeAnswer::OnePipe { emitter_count }) =
        &heating.distribution_type
    {
        check_count(
            recorder,
            Some(*emitter_count),
            MAX_COUNT,
            "count_out_of_range",
            "heating.distributionType.emitterCount",
        );
    }
    if let heating::HeatingGenerator::GasAirHeater { count, .. } = &heating.generator {
        check_count(
            recorder,
            *count,
            MAX_COUNT,
            "count_out_of_range",
            "heating.generator.count",
        );
    }
    match &heating.air_heating {
        Some(heating::AirHeatingAnswer::Direct { count, .. })
        | Some(heating::AirHeatingAnswer::Indirect { count, .. }) => {
            check_count(
                recorder,
                *count,
                MAX_COUNT,
                "count_out_of_range",
                "heating.airHeating.count",
            );
        }
        _ => {}
    }
}

/// Largest area a survey can describe (A_g, a surface or a window), m², and
/// the largest building height, m. Program choices, not norm values: above
/// them the input cannot describe a building.
pub(crate) const MAX_AREA_M2: f64 = 1.0e7;
pub(crate) const MAX_BUILDING_HEIGHT_M: f64 = 1000.0;

/// Upper bounds of the envelope areas, shared by both surveys.
pub(crate) fn validate_envelope_bounds(envelope: &SurveyEnvelope, recorder: &mut Recorder) {
    for (index, surface) in envelope.surfaces.iter().enumerate() {
        if surface.gross_area_m2 > MAX_AREA_M2 {
            recorder.issue(
                "area_out_of_range",
                format!("envelope.surfaces[{index}].grossAreaM2"),
            );
        }
    }
    for (index, window) in envelope.windows.iter().enumerate() {
        if window.area_m2 > MAX_AREA_M2 {
            recorder.issue(
                "area_out_of_range",
                format!("envelope.windows[{index}].areaM2"),
            );
        }
    }
}

/// Range checks of the vertical pipes, shared by both surveys.
pub(crate) fn validate_vertical_pipe_counts(
    pipes: Option<&[SurveyVerticalPipe]>,
    recorder: &mut Recorder,
) {
    let Some(pipes) = pipes else { return };
    if pipes.len() > MAX_COUNT as usize {
        recorder.issue("count_out_of_range", "verticalPipes");
    }
    for (index, pipe) in pipes.iter().enumerate() {
        check_count(
            recorder,
            pipe.shared_zones,
            MAX_COUNT,
            "count_out_of_range",
            &format!("verticalPipes[{index}].sharedZones"),
        );
    }
}

/// The optional ids of one survey list (`list` is its path): a given id is
/// not blank and names one item only, so an evidence or photo link by id
/// (`/…/@id`) finds exactly that item. Ids are not used in the calculation.
pub(crate) fn validate_item_ids<'a>(
    ids: impl IntoIterator<Item = Option<&'a str>>,
    list: &str,
    recorder: &mut Recorder,
) {
    let mut seen: Vec<&str> = Vec::new();
    for (index, id) in ids.into_iter().enumerate() {
        let Some(id) = id else { continue };
        if id.trim().is_empty() {
            recorder.issue("survey_item_id_blank", format!("{list}[{index}].id"));
        } else if seen.contains(&id) {
            recorder.issue("survey_item_id_duplicate", format!("{list}[{index}].id"));
        } else {
            seen.push(id);
        }
    }
}

/// The ids of the survey lists without a natural key: further heating and
/// hot-water generators and further hot-water systems.
fn validate_residential_item_ids(survey: &ResidentialSurvey, recorder: &mut Recorder) {
    validate_item_ids(
        survey.heating.additional_generators.iter().map(|item| item.id.as_deref()),
        "heating.additionalGenerators",
        recorder,
    );
    validate_item_ids(
        survey.hot_water.additional_generators.iter().map(|item| item.id.as_deref()),
        "hotWater.additionalGenerators",
        recorder,
    );
    validate_item_ids(
        survey.additional_hot_water_systems.iter().map(|item| item.id.as_deref()),
        "additionalHotWaterSystems",
        recorder,
    );
    for (index, system) in survey.additional_hot_water_systems.iter().enumerate() {
        validate_item_ids(
            system.additional_generators.iter().map(|item| item.id.as_deref()),
            &format!("additionalHotWaterSystems[{index}].additionalGenerators"),
            recorder,
        );
    }
}

fn validate(survey: &ResidentialSurvey, recorder: &mut Recorder) {
    validate_residential_item_ids(survey, recorder);
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
    if survey.usable_floor_area_m2 > MAX_AREA_M2 {
        recorder.issue("area_out_of_range", "usableFloorAreaM2");
    }
    if survey.building_height_m > MAX_BUILDING_HEIGHT_M {
        recorder.issue("building_height_invalid", "buildingHeightM");
    }
    validate_envelope_bounds(&survey.envelope, recorder);
    check_count(
        recorder,
        survey.storeys,
        MAX_STOREYS,
        "storeys_out_of_range",
        "storeys",
    );
    validate_heating_counts(&survey.heating, recorder);
    validate_vertical_pipe_counts(survey.vertical_pipes.as_deref(), recorder);
    let hot = &survey.hot_water;
    check_count(
        recorder,
        Some(hot.showers),
        MAX_COUNT,
        "count_out_of_range",
        "hotWater.showers",
    );
    check_count(
        recorder,
        hot.connected_bathrooms,
        MAX_COUNT,
        "count_out_of_range",
        "hotWater.connectedBathrooms",
    );
    check_count(
        recorder,
        hot.connected_kitchens,
        MAX_COUNT,
        "count_out_of_range",
        "hotWater.connectedKitchens",
    );
    if let Some(collective) = &hot.collective {
        check_count(
            recorder,
            collective.connected_dwellings,
            MAX_COUNT,
            "count_out_of_range",
            "hotWater.collective.connectedDwellings",
        );
    }
    for (index, solar) in hot.solar.iter().enumerate() {
        check_count(
            recorder,
            Some(solar.collector_count),
            MAX_COUNT,
            "count_out_of_range",
            &format!("hotWater.solar[{index}].collectorCount"),
        );
    }
    // The dwelling survey derives one calculation zone.
    for (index, surface) in survey.envelope.surfaces.iter().enumerate() {
        if surface.zone_id.is_some() {
            recorder.issue(
                "surface_zone_not_in_dwelling_survey",
                format!("envelope.surfaces[{index}].zoneId"),
            );
        }
    }
    if survey.construction.closed_or_suspended_ceiling {
        // ISSO 82.1 table 7.4 (p. 62) has no closed-ceiling column: only a
        // (very) heavy floor whose top is heavier than the ceiling above
        // takes the first column (`lighterCeiling`). The closed or
        // suspended ceiling is the ISSO 75.1 utility criterion.
        recorder.issue(
            "closed_ceiling_not_in_dwelling_survey",
            "construction.closedOrSuspendedCeiling",
        );
    }
    if survey.cooling_present && survey.cooling.is_none() {
        // ISSO 82.1 §10.2: a cooled zone needs the cooling system data.
        recorder.issue("cooling_system_data_required", "cooling");
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
    let systems = std::iter::once((&survey.hot_water, "hotWater".to_string())).chain(
        survey
            .additional_hot_water_systems
            .iter()
            .enumerate()
            .map(|(index, hot)| (hot, format!("additionalHotWaterSystems[{index}]"))),
    );
    for (hot, path) in systems {
        let kitchen = matches!(
            hot.served,
            hot_water::TapsServed::KitchenAndBathroom | hot_water::TapsServed::KitchenOnly
        );
        let bathroom = matches!(
            hot.served,
            hot_water::TapsServed::KitchenAndBathroom | hot_water::TapsServed::BathroomOnly
        );
        if kitchen && hot.kitchen_length_m.is_none() {
            recorder.issue("tap_length_required", format!("{path}.kitchenLengthM"));
        }
        if bathroom && hot.bathroom_length_m.is_none() {
            recorder.issue("tap_length_required", format!("{path}.bathroomLengthM"));
        }
    }
}

/// A_ls with f_ls of NTA 6.7.3 from the survey surfaces (gross areas).
pub(crate) fn loss_area(envelope: &SurveyEnvelope) -> f64 {
    envelope
        .surfaces
        .iter()
        .map(|surface| {
            let weight = match surface.boundary {
                SurfaceBoundary::Outdoor
                | SurfaceBoundary::StronglyVentilated
                | SurfaceBoundary::Sunroom
                | SurfaceBoundary::Water
                | SurfaceBoundary::UnheatedSpace { .. } => 1.0,
                SurfaceBoundary::Ground
                | SurfaceBoundary::Crawlspace
                | SurfaceBoundary::UnheatedCellar => 0.7,
                SurfaceBoundary::AdjacentHeated => 0.0,
            };
            weight * surface.gross_area_m2
        })
        .sum()
}

/// One hot-water system of the residential survey as kernel input.
fn survey_hot_water(
    survey: &ResidentialSurvey,
    hot: &SurveyHotWater,
    path: &str,
    recorder: &mut Recorder,
) -> Value {
    let year = survey.construction_year;
    let mut system = hot_water::derive_hot_water(hot, recorder);
    hot_water::apply_extensions(
        &mut system,
        hot,
        year,
        survey.usable_floor_area_m2,
        false,
        recorder,
    );
    hot_water::apply_exhaust_air_use(
        &mut system,
        survey.ventilation.principle,
        survey.ventilation.heat_recovery.is_some(),
    );
    if matches!(
        hot.generator,
        hot_water::HotWaterGeneratorAnswer::ElectricBoiler
    ) {
        if let Some(vessel) = hot_water::boiler_storage(
            hot.boiler_vessel.as_ref(),
            year,
            &hot.source_reference,
            recorder,
        ) {
            system["storage"] = json!([vessel]);
        }
    } else if hot.boiler_vessel.is_some() {
        recorder.issue(
            "boiler_vessel_not_applicable",
            format!("{path}.boilerVessel"),
        );
    }
    system
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
    let (mut floor, mut wall, ceiling) = general::thermal_mass(&survey.construction);
    // p. 62: houseboats and caravans count as light floor and light wall.
    if survey
        .envelope
        .building_kind
        .is_some_and(|kind| !matches!(kind, crate::forfait_envelope::BuildingKind::Regular))
        && (floor != crate::monthly_demand::MassClass::Light
            || wall != crate::monthly_demand::MassClass::Light)
    {
        recorder.record(
            "houseboat_caravan_light_mass",
            "construction",
            "light floor, light wall".into(),
            "ISSO 82.1 p. 62",
        );
        floor = crate::monthly_demand::MassClass::Light;
        wall = crate::monthly_demand::MassClass::Light;
    }
    // Table 8.24/8.25: the shading rows depend on cooling in the zone.
    let envelope = envelope::derive_envelope_with_cooling(
        &survey.envelope,
        year,
        survey.cooling.is_some(),
        recorder,
    );
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
    let mut heating = heating::derive_heating(&survey.heating, year, recorder);
    let unheated_spaces = survey.envelope.surfaces.iter().any(|surface| {
        matches!(
            surface.boundary,
            SurfaceBoundary::Crawlspace
                | SurfaceBoundary::UnheatedCellar
                | SurfaceBoundary::UnheatedSpace { .. }
        )
    });
    let calculated_distribution =
        heating::apply_unheated_pipes(&survey.heating, &mut heating, unheated_spaces, recorder);
    let mut hot_water = survey_hot_water(survey, &survey.hot_water, "hotWater", recorder);
    let mut additional_hot_water = Vec::new();
    if !survey.additional_hot_water_systems.is_empty() {
        // NTA 13.19a: the share of each system follows its taps.
        hot_water["connectedTaps"] =
            hot_water::connected_taps(&survey.hot_water, "hotWater", recorder);
        for (index, hot) in survey.additional_hot_water_systems.iter().enumerate() {
            let path = format!("additionalHotWaterSystems[{index}]");
            let mut system = survey_hot_water(survey, hot, &path, recorder);
            system["connectedTaps"] = hot_water::connected_taps(hot, &path, recorder);
            additional_hot_water.push(system);
        }
    }
    let pv: Vec<Value> = survey
        .pv
        .iter()
        .map(|item| production::derive_pv(item, year, recorder))
        .collect();
    let (storage_present, storage) =
        production::derive_storage(survey.storage.as_ref(), !survey.pv.is_empty(), recorder);
    if !recorder.issues.is_empty() {
        return None;
    }
    let area = survey.usable_floor_area_m2;
    let storeys = survey.storeys.unwrap_or(survey.heating.storeys).max(1);
    if survey.vertical_pipes.is_none() {
        recorder.record(
            "vertical_pipes_unknown_one_per_storey",
            "verticalPipes",
            format!("{storeys} uninsulated pipe(s), {storeys} storey(s) each"),
            "ISSO 82.1 p. 63 (table 7.7); NTA 8800 7.3.3",
        );
    }
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
            "verticalPipes": vertical_pipes(
                survey.vertical_pipes.as_deref(),
                storeys,
                storeys,
                reference,
                recorder,
            ),
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
        "distribution": calculated_distribution.unwrap_or_else(|| json!({"method": "heated_zone_only_space_heating", "sourceReference": format!("{reference}; basisopname: pipes in the heated zone")})),
        "generator": heating.generator,
    });
    if let Some(system) = heating.distribution_system {
        chain["distributionSystem"] = system;
    }
    if let Some(connection) =
        heating::collective_connection(&survey.heating, survey.usable_floor_area_m2, recorder)
    {
        chain["collectiveConnection"] = connection;
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
        "additionalHotWaterSystems": additional_hot_water,
        "lossAreaM2": loss_area(&survey.envelope),
        "lossAreaSourceReference": "basisopname: survey surfaces with f_ls (NTA 6.7.3)",
        "demandUsesFixedC1Ventilation": false,
        "batteryStoragePresent": storage_present,
    });
    // §5.3.2 (Standaard voor woningisolatie) and the label data need the
    // construction year; the survey records it, so it is passed on.
    if let Ok(year) = u32::try_from(survey.construction_year) {
        if year > 0 {
            input["constructionYear"] = json!(year);
        }
    }
    if let Some(storage) = storage {
        input["storage"] = storage;
    }
    if let Some(renewable) = heating.heat_pump_renewable {
        input["heatPumpRenewable"] = renewable;
    }
    if let Some(source) = heating.collective_heat_pump_source {
        input["externalSupply"] = json!({"collectiveHeatPumpSource": source});
    }
    // ISSO 82.1 chapter 10: the main cooling system of the zone.
    if let Some(cooling) = &survey.cooling {
        let mut value = utility::cooling_value(
            cooling,
            year,
            storeys,
            utility::CoolingBook::Residential,
            !survey.cooling_collective,
            recorder,
        );
        utility::apply_cooling_heat_pump_source(
            &mut value,
            cooling,
            &input["spaceHeating"]["generator"],
            utility::CoolingBook::Residential,
            recorder,
        );
        input["cooling"] = value;
    }
    Some(input)
}

/// Survey → kernel input → building performance, in the survey's edition.
pub fn assess_residential_survey(survey: &ResidentialSurvey) -> OpnameAssessment {
    norm_versions::with_version(survey.norm_version, || {
        assess_residential_in_edition(survey)
    })
}

fn assess_residential_in_edition(survey: &ResidentialSurvey) -> OpnameAssessment {
    let mut recorder = Recorder::default();
    let derived = derive_residential_input(survey, &mut recorder);
    let derived_input = parse_derived_input(derived, survey.norm_version, &mut recorder);
    let performance = derived_input.as_ref().map(assess_building_performance);
    let status = match &performance {
        Some(result) if result.status == "calculated_unverified" => "calculated_unverified",
        Some(_) => "derived_input_rejected",
        None => "invalid",
    };
    surface_rejection(status, performance.as_ref(), &mut recorder.issues);
    apply_collapse_reasons(&mut recorder, &survey.collapse_reasons);
    let status = edition_status(status, survey.norm_version, &mut recorder);
    refuse_non_finite(OpnameAssessment {
        status,
        scope: "isso_82_1_basisopname_residential_unverified",
        source: ISSO_SOURCE,
        target_norm_version: survey.norm_version.label(),
        norm_version: survey.norm_version,
        registration_eligible: survey.norm_version.registration_eligible(),
        applied_defaults: recorder.applied,
        warnings: recorder.warnings,
        issues: recorder.issues,
        derived_input,
        performance,
        reference_verified: false,
    })
}

/// Parses the derived kernel input, carrying the survey's edition into it
/// (the building route applies the edition of its own input).
pub(crate) fn parse_derived_input(
    derived: Option<Value>,
    version: NormVersion,
    recorder: &mut Recorder,
) -> Option<BuildingPerformanceInput> {
    derived.and_then(|mut value| {
        if !version.is_default() {
            value["normVersion"] = json!(version);
        }
        match serde_json::from_value::<BuildingPerformanceInput>(value) {
            Ok(input) => Some(input),
            Err(error) => {
                recorder.issues.push(OpnameIssue {
                    code: "derived_input_shape_invalid",
                    path: error.to_string(),
                });
                None
            }
        }
    })
}

/// A survey in an older edition: the ISSO 82.1/75.1 protocol (7e druk,
/// 2025) belongs to 2025+C1, so the result is a comparison only. A
/// calculation keeps the legacy status and is never registrable.
pub(crate) fn edition_status(
    status: &'static str,
    version: NormVersion,
    recorder: &mut Recorder,
) -> &'static str {
    if version.is_default() {
        return status;
    }
    recorder.warning(
        "survey_protocol_edition_differs",
        "normVersion",
        "the ISSO survey protocol (7e druk, 2025) belongs to NTA 8800:2025+C1; a survey calculated in an older edition is for comparison only and cannot be registered",
    );
    if status == "calculated_unverified" {
        "calculated_legacy_edition"
    } else {
        status
    }
}

/// A survey whose derived input the kernel refuses must say why: when the
/// survey itself recorded no issue, the kernel's own issues are surfaced
/// (prefixed with `derivedInput.`), and a refusal without any reason gets
/// `derived_input_rejected_without_reason`. A `derived_input_rejected`
/// status never comes with an empty issue list.
pub(crate) fn surface_rejection(
    status: &str,
    performance: Option<&crate::building_performance::BuildingPerformanceAssessment>,
    issues: &mut Vec<OpnameIssue>,
) {
    if status != "derived_input_rejected" || !issues.is_empty() {
        return;
    }
    if let Some(performance) = performance {
        issues.extend(performance.issues.iter().map(|issue| OpnameIssue {
            code: issue.code,
            path: format!("derivedInput.{}", issue.path),
        }));
        if issues.is_empty() {
            issues.push(OpnameIssue {
                code: "derived_input_rejected_without_reason",
                path: performance.status.to_string(),
            });
        }
    }
}

/// Safety net: a survey result with a non-finite number (NaN, ±∞) is never
/// handed out (serde_json would write `null`). The status becomes `invalid`
/// with `non_finite_result` at the first such path, and the computed parts
/// are withheld.
pub(crate) fn refuse_non_finite(mut assessment: OpnameAssessment) -> OpnameAssessment {
    let Some(path) = crate::finite::first_non_finite(&assessment) else {
        return assessment;
    };
    assessment.status = "invalid";
    assessment.performance = None;
    assessment.derived_input = None;
    assessment.issues.push(OpnameIssue {
        code: "non_finite_result",
        path,
    });
    assessment
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
    fn dwelling_survey_rejects_the_utility_closed_ceiling_criterion() {
        // 82.1 table 7.4 (p. 62) only knows the lighter-ceiling criterion.
        let mut survey = fixture("1975");
        survey.construction.closed_or_suspended_ceiling = true;
        let result = assess_residential_survey(&survey);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "closed_ceiling_not_in_dwelling_survey"));
    }

    #[test]
    fn roof_and_floor_apartment_calculates_as_a_top_storey_dwelling() {
        // Positions 4/8 of afb. 7.1 (p. 51).
        let mut survey = fixture("1975");
        survey.dwelling = DwellingType::Apartment {
            floor: general::ApartmentFloor::RoofAndFloor,
            side: general::ApartmentSide::Middle,
        };
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert!(result
            .applied_defaults
            .iter()
            .any(|item| item.rule == "apartment_roof_and_floor_top_type"));
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
    fn collapse_reasons_attach_by_path_or_rule() {
        let mut survey = fixture("1930");
        let plain = assess_residential_survey(&survey);
        let first = plain.applied_defaults[0].clone();
        let second = plain.applied_defaults.last().unwrap().clone();
        assert!(first.collapse_reason.is_none());
        survey
            .collapse_reasons
            .insert(first.path.clone(), "niet zichtbaar".into());
        survey
            .collapse_reasons
            .insert(second.rule.to_string(), "geen factuur".into());
        survey.collapse_reasons.insert("nergens".into(), "x".into());
        let result = assess_residential_survey(&survey);
        let applied = &result.applied_defaults;
        // A path match wins over a rule match.
        assert_eq!(
            applied[0].collapse_reason.as_deref(),
            Some("niet zichtbaar")
        );
        assert!(applied.last().unwrap().collapse_reason.is_some());
        assert!(applied
            .iter()
            .filter(|item| item.path != first.path && item.rule != second.rule)
            .all(|item| item.collapse_reason.is_none()));
        let json = serde_json::to_value(&applied[0]).unwrap();
        assert!(json.get("inklapReden").is_some());
        assert!(result
            .warnings
            .iter()
            .any(|item| item.code == "collapse_reason_unmatched"
                && item.path == "inklapRedenen.nergens"));
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
        assert!(codes.contains(&"cooling_system_data_required"));
    }

    #[test]
    fn caravans_and_houseboats_have_light_floors_and_walls() {
        let mut survey = fixture("1930");
        let mut recorder = Recorder::default();
        let input = derive_residential_input(&survey, &mut recorder).unwrap();
        let regular = input["spaceHeating"]["demand"]["thermalMass"].clone();
        survey.envelope.building_kind = Some(crate::forfait_envelope::BuildingKind::Caravan);
        let mut recorder = Recorder::default();
        let input = derive_residential_input(&survey, &mut recorder).unwrap();
        let mass = &input["spaceHeating"]["demand"]["thermalMass"];
        assert_eq!(mass["floor"], "light");
        assert_eq!(mass["wall"], "light");
        assert_eq!(mass["ceiling"], regular["ceiling"]);
        assert_eq!(
            regular["floor"] != "light" || regular["wall"] != "light",
            recorder
                .applied
                .iter()
                .any(|item| item.rule == "houseboat_caravan_light_mass")
        );
    }

    #[test]
    fn vertical_pipes_default_to_one_per_storey() {
        let mut survey = fixture("1930");
        survey.storeys = Some(2);
        let mut recorder = Recorder::default();
        let input = derive_residential_input(&survey, &mut recorder).unwrap();
        let pipes = &input["spaceHeating"]["demand"]["transmission"]["verticalPipes"];
        assert_eq!(pipes.as_array().unwrap().len(), 2);
        assert_eq!(pipes[0]["storeys"], 2);
        assert_eq!(pipes[0]["insulated"], false);
        // Determined absent: no pipes.
        survey.vertical_pipes = Some(Vec::new());
        let mut recorder = Recorder::default();
        let input = derive_residential_input(&survey, &mut recorder).unwrap();
        let pipes = &input["spaceHeating"]["demand"]["transmission"]["verticalPipes"];
        assert!(pipes.as_array().unwrap().is_empty());
    }

    #[test]
    fn kitchen_geyser_next_to_bathroom_appliance_splits_the_need() {
        let mut survey = fixture("1930");
        survey.hot_water.served = hot_water::TapsServed::BathroomOnly;
        survey.hot_water.kitchen_length_m = None;
        survey.additional_hot_water_systems = vec![serde_json::from_value(json!({
            "generator": {"kind": "gas_appliance", "applianceType": "kitchen_geyser", "gaskeur": "none"},
            "served": "kitchen_only",
            "kitchenLengthM": 3.0,
            "showers": 0,
            "showerHeatRecovery": "none",
            "sourceReference": "survey"
        }))
        .unwrap()];
        let mut recorder = Recorder::default();
        let input = derive_residential_input(&survey, &mut recorder).expect("derived input");
        assert!(recorder.issues.is_empty(), "{:?}", recorder.issues);
        // p. 164 / NTA 13.19a: the connected taps follow the served taps.
        assert_eq!(
            input["hotWater"]["connectedTaps"],
            json!({"bathrooms": 1, "kitchens": 0})
        );
        assert_eq!(
            input["additionalHotWaterSystems"][0]["connectedTaps"],
            json!({"bathrooms": 0, "kitchens": 1})
        );
        assert!(recorder
            .applied
            .iter()
            .any(|item| item.rule == "hot_water_connected_kitchens_from_served"));
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
    }

    #[test]
    fn dwelling_with_split_airco_is_cooled_per_isso_82_1_chapter_10() {
        let mut survey = fixture("2015");
        let base = assess_residential_survey(&survey);
        survey.cooling_present = true;
        survey.cooling = Some(
            serde_json::from_value(json!({
                "generator": "room_air_conditioner",
                "emitter": "split_indoor_units_on_wall",
                "fanCoilCount": 3,
                "waterBased": false,
                "sourceReference": "survey photos"
            }))
            .unwrap(),
        );
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        let input = result.derived_input.as_ref().unwrap();
        assert!(input.cooling.is_some());
        // Split indoor units count as fan convectors, cited from ISSO 82.1.
        assert!(result.applied_defaults.iter().any(|item| {
            item.rule == "split_indoor_units_fan_coils" && item.source.starts_with("ISSO 82.1")
        }));
        // Cooling adds electricity to the dwelling.
        let electricity = |assessment: &OpnameAssessment| -> f64 {
            assessment
                .performance
                .as_ref()
                .unwrap()
                .carriers
                .iter()
                .filter(|row| row.carrier == "el")
                .map(|row| row.used_kwh)
                .sum()
        };
        assert!(electricity(&result) > electricity(&base));
        // Without the system data the survey is rejected.
        survey.cooling = None;
        let missing = assess_residential_survey(&survey);
        assert!(missing
            .issues
            .iter()
            .any(|item| item.code == "cooling_system_data_required"));
    }

    #[test]
    fn fixture_indicators_stay_plausible() {
        // EP2 ranges typical for the labels of these dwellings (F–G, B–C,
        // A++ and better); crawlspaces without inspection carry heating
        // pipes in unheated spaces (afb. 9.1).
        for (name, low, high, distribution) in [
            ("1930", 250.0, 450.0, "calculated"),
            ("1975", 140.0, 240.0, "calculated"),
            ("2015", 20.0, 80.0, "heated_zone_only_space_heating"),
        ] {
            let result = assess_residential_survey(&fixture(name));
            let performance = result.performance.unwrap();
            let ep2 = performance
                .primary_fossil_indicator_kwh_per_m2_year
                .unwrap();
            assert!((low..high).contains(&ep2), "{name}: EP2 {ep2}");
            let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
            assert_eq!(
                input["spaceHeating"]["distribution"]["method"], distribution,
                "{name}"
            );
        }
    }

    fn heat_pump(source: heating::HeatPumpSource) -> heating::HeatingGenerator {
        heating::HeatingGenerator::HeatPump {
            source,
            air_sink: false,
            high_temperature: false,
            capacity_kw: Some(5.0),
            source_regeneration_factor: None,
            high_efficiency_evidence: None,
            drive: Default::default(),
            groundwater_system: None,
            collective_source_reference: None,
            source_temperature_c: None,
            source_temperature_reference: None,
            source_quality_declaration_reference: None,
            manufacture_year: None,
            installation_year: None,
        }
    }

    fn boiler() -> heating::HeatingGenerator {
        heating::HeatingGenerator::Boiler {
            boiler_type: heating::BoilerType::Hr107,
            pilot_flame: Some(false),
            inside_thermal_boundary: true,
            manufacture_year: Some(2015),
            installation_year: None,
        }
    }

    #[test]
    fn hybrid_heat_pump_becomes_multiple_generators() {
        let mut survey = fixture("1975");
        survey.heating.generator = boiler();
        survey.heating.nominal_power_kw = Some(24.0);
        survey.heating.additional_generators = vec![heating::AdditionalHeatingGenerator {
            id: None,
            generator: heat_pump(heating::HeatPumpSource::OutdoorAir),
            nominal_power_kw: Some(4.0),
        }];
        survey.heating.added_preferred_generator = true;
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(generator["kind"], "multiple");
        assert_eq!(generator["addedPreferredGenerator"], true);
        // p. 112: the heat pump is preferred over the boiler.
        let parts = generator["generators"].as_array().unwrap();
        let preference = |kind: &str| {
            parts
                .iter()
                .find(|part| part["generator"]["kind"] == kind)
                .unwrap()["preference"]
                .clone()
        };
        assert_eq!(preference("heat_pump_forfait"), 1);
        assert_eq!(preference("gas_boiler"), 2);
        // Table 9.9: the boiler decides the design class with radiators.
        assert!(result
            .applied_defaults
            .iter()
            .any(|item| item.rule == "several_generators_preference_order"));
        // Without the boiler's power the survey is incomplete (table 9.7).
        survey.heating.nominal_power_kw = None;
        let missing = assess_residential_survey(&survey);
        assert!(missing
            .issues
            .iter()
            .any(|item| item.code == "generator_power_required"));
    }

    /// Ids of further generators are links only: they leave the derived input
    /// unchanged, a blank or repeated id within one list is an issue.
    #[test]
    fn survey_item_ids_are_unique_and_do_not_change_the_calculation() {
        let mut survey = fixture("1975");
        survey.heating.generator = boiler();
        survey.heating.nominal_power_kw = Some(24.0);
        let extra = |id: Option<&str>| heating::AdditionalHeatingGenerator {
            id: id.map(str::to_string),
            generator: heat_pump(heating::HeatPumpSource::OutdoorAir),
            nominal_power_kw: Some(4.0),
        };
        survey.heating.additional_generators = vec![extra(None)];
        let plain = assess_residential_survey(&survey);
        survey.heating.additional_generators = vec![extra(Some("wp-1"))];
        let with_id = assess_residential_survey(&survey);
        assert_eq!(with_id.status, plain.status);
        assert_eq!(
            serde_json::to_value(with_id.derived_input.as_ref().unwrap()).unwrap(),
            serde_json::to_value(plain.derived_input.as_ref().unwrap()).unwrap()
        );
        // The id round-trips through the survey JSON.
        let json = serde_json::to_value(&survey).unwrap();
        assert_eq!(json["heating"]["additionalGenerators"][0]["id"], "wp-1");
        survey.heating.additional_generators = vec![extra(Some("wp-1")), extra(Some("wp-1"))];
        let repeated = assess_residential_survey(&survey);
        assert!(repeated.issues.iter().any(|item| item.code == "survey_item_id_duplicate"
            && item.path == "heating.additionalGenerators[1].id"));
        survey.heating.additional_generators = vec![extra(Some(" "))];
        let blank = assess_residential_survey(&survey);
        assert!(blank.issues.iter().any(|item| item.code == "survey_item_id_blank"
            && item.path == "heating.additionalGenerators[0].id"));
    }

    #[test]
    fn exhaust_air_heat_pump_is_accepted_with_a_second_generator() {
        let mut survey = fixture("2015");
        survey.heating.generator = heat_pump(heating::HeatPumpSource::ExhaustAir);
        let alone = assess_residential_survey(&survey);
        assert!(alone
            .issues
            .iter()
            .any(|item| item.code == "exhaust_air_heat_pump_second_generator_required"));
        survey.heating.nominal_power_kw = Some(1.5);
        survey.heating.additional_generators = vec![heating::AdditionalHeatingGenerator {
            id: None,
            generator: heating::HeatingGenerator::Electric {
                connected_devices: 1,
            },
            nominal_power_kw: Some(3.0),
        }];
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
    }

    #[test]
    fn collective_boiler_uses_connected_area_and_heat_meters() {
        let mut survey = fixture("1975");
        survey.heating.generator = boiler();
        survey.heating.nominal_power_kw = Some(300.0);
        survey.heating.collective = Some(heating::CollectiveHeating {
            connected_usable_area_m2: None,
            connected_dwellings: Some(40),
            connected_storeys: Some(4),
            heat_meters_present: None,
        });
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        let chain = &input["spaceHeating"];
        assert_eq!(chain["generator"]["boiler"]["role"], "collective");
        assert_eq!(
            chain["collectiveConnection"]["connectedUsableAreaM2"],
            40.0 * survey.usable_floor_area_m2
        );
        assert_eq!(chain["distributionSystem"]["installation"], "collective");
        assert_eq!(
            chain["distributionSystem"]["pump"]["heatMeterPresent"],
            true
        );
        let rules: Vec<_> = result
            .applied_defaults
            .iter()
            .map(|item| item.rule)
            .collect();
        assert!(rules.contains(&"collective_area_dwellings_times_area"));
        assert!(rules.contains(&"heat_meters_unknown_present"));
        // §13.3.4: hot water through a delivery set on that system
        // (NTA 13.8.4.9.3) loads the heating node instead of a carrier.
        survey.hot_water.generator = hot_water::HotWaterGeneratorAnswer::DeliverySetFromHeating;
        let delivery = assess_residential_survey(&survey);
        assert_eq!(
            delivery.status,
            "calculated_unverified",
            "{:?} {:?}",
            delivery.issues,
            delivery.performance.as_ref().map(|item| &item.issues)
        );
        let input = serde_json::to_value(delivery.derived_input.as_ref().unwrap()).unwrap();
        assert_eq!(input["hotWater"]["generator"]["kind"], "heating_system");
    }

    fn solar(backup: hot_water::SolarBackupAnswer) -> hot_water::SurveySolarWaterHeater {
        hot_water::SurveySolarWaterHeater {
            id: "zb".into(),
            collector: hot_water::CollectorAnswer::Unknown,
            collector_area_m2: 2.4,
            gross_area: true,
            collector_count: 1,
            orientation: crate::climate::Orientation::South,
            tilt_deg: 45.0,
            shading: None,
            backup,
            storage_volume_l: 150.0,
            backup_volume_l: None,
            storage_label: None,
            storage_manufacture_year: None,
            also_space_heating: false,
            pvt: None,
            source_reference: "survey photo".into(),
        }
    }

    #[test]
    fn solar_water_heater_follows_tables_15_4_and_15_8() {
        let mut survey = fixture("2015");
        let base = assess_residential_survey(&survey);
        survey.hot_water.solar = vec![solar(hot_water::SolarBackupAnswer::Unknown)];
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        let heater = &input["hotWater"]["solar"][0];
        assert_eq!(heater["method"]["solarType"], "preheater");
        assert_eq!(
            heater["method"]["collectors"]["efficiency"]["collector"],
            "unglazed_or_unknown"
        );
        assert_eq!(
            heater["method"]["collectors"]["obstruction"]["method"],
            "minimal"
        );
        let rules: Vec<_> = result
            .applied_defaults
            .iter()
            .map(|item| item.rule)
            .collect();
        for rule in [
            "solar_collector_unknown_unglazed",
            "solar_backup_unknown_preheater",
            "solar_vessel_year_unknown_construction_year",
            "solar_shading_not_entered_minimal",
        ] {
            assert!(rules.contains(&rule), "{rule}");
        }
        // A glazed collector on a gas-heated dwelling lowers the primary
        // fossil energy (the unglazed forfait, a1 = 15, yields little).
        let ep = |result: &OpnameAssessment| {
            result
                .performance
                .as_ref()
                .unwrap()
                .primary_fossil_indicator_kwh_per_m2_year
                .unwrap()
        };
        let mut gas = fixture("1930");
        let gas_base = assess_residential_survey(&gas);
        let mut glazed = solar(hot_water::SolarBackupAnswer::SeparateHeater);
        glazed.collector = hot_water::CollectorAnswer::Glazed;
        glazed.storage_manufacture_year = Some(2020);
        gas.hot_water.solar = vec![glazed];
        let gas_solar = assess_residential_survey(&gas);
        assert_eq!(
            gas_solar.status, "calculated_unverified",
            "{:?}",
            gas_solar.issues
        );
        assert!(
            ep(&gas_solar) < ep(&gas_base),
            "{} vs {}",
            ep(&gas_solar),
            ep(&gas_base)
        );
        let _ = base;
        // An evacuated-tube gross area counts 60 % (p. 192).
        let mut tube = solar(hot_water::SolarBackupAnswer::IntegratedElectric);
        tube.collector = hot_water::CollectorAnswer::EvacuatedTube;
        survey.hot_water.solar = vec![tube];
        let mut recorder = Recorder::default();
        let input = derive_residential_input(&survey, &mut recorder).unwrap();
        let collectors = &input["hotWater"]["solar"][0]["method"]["collectors"];
        assert!((collectors["moduleAreaM2"].as_f64().unwrap() - 1.44).abs() < 1e-9);
        assert_eq!(
            input["hotWater"]["solar"][0]["method"]["solarType"],
            "integrated_backup"
        );
    }

    #[test]
    fn collective_hot_water_and_a_second_generator() {
        let mut survey = fixture("1975");
        survey.hot_water.generator = hot_water::HotWaterGeneratorAnswer::CollectiveUnknown;
        survey.hot_water.collective = Some(hot_water::CollectiveHotWaterAnswer {
            building_usable_area_m2: None,
            connected_dwellings: Some(24),
        });
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        assert_eq!(
            input["hotWater"]["generator"]["kind"],
            "large_direct_storage"
        );
        assert_eq!(
            input["hotWater"]["collective"]["buildingUsableFloorAreaM2"],
            24.0 * survey.usable_floor_area_m2
        );
        // A heat-pump boiler with an electric heater behind it (13.8.2).
        let mut survey = fixture("2015");
        survey.hot_water.generator = hot_water::HotWaterGeneratorAnswer::HeatPump {
            exhaust_air_source: false,
        };
        survey.hot_water.nominal_power_kw = Some(1.5);
        survey.hot_water.additional_generators = vec![hot_water::AdditionalHotWaterAnswer {
            id: None,
            generator: hot_water::HotWaterGeneratorAnswer::ElectricInstantaneous,
            nominal_power_kw: Some(6.0),
        }];
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        assert_eq!(
            input["hotWater"]["additionalGenerators"][0]["generator"]["kind"],
            "electric_instantaneous"
        );
    }

    #[test]
    fn loss_area_weights_ground_with_0_7() {
        let survey = fixture("1930");
        let expected: f64 = survey
            .envelope
            .surfaces
            .iter()
            .map(|s| match s.boundary {
                SurfaceBoundary::Ground
                | SurfaceBoundary::Crawlspace
                | SurfaceBoundary::UnheatedCellar => 0.7 * s.gross_area_m2,
                SurfaceBoundary::AdjacentHeated => 0.0,
                _ => s.gross_area_m2,
            })
            .sum();
        assert!((loss_area(&survey.envelope) - expected).abs() < 1e-9);
    }

    fn calculated(survey: &ResidentialSurvey) -> (OpnameAssessment, Value) {
        let result = assess_residential_survey(survey);
        assert_eq!(
            result.status,
            "calculated_unverified",
            "{:?} {:?}",
            result.issues,
            result.performance.as_ref().map(|item| &item.issues)
        );
        let input = serde_json::to_value(result.derived_input.as_ref().unwrap()).unwrap();
        (result, input)
    }

    fn rules(result: &OpnameAssessment) -> Vec<&'static str> {
        result
            .applied_defaults
            .iter()
            .map(|item| item.rule)
            .collect()
    }

    fn efficiency(result: &OpnameAssessment) -> f64 {
        result
            .performance
            .as_ref()
            .unwrap()
            .space_heating
            .generation_efficiency
            .unwrap()
    }

    #[test]
    fn oil_boiler_is_a_conventional_boiler_on_oil() {
        let mut survey = fixture("1930");
        survey.heating.generator = heating::HeatingGenerator::Boiler {
            boiler_type: heating::BoilerType::Oil,
            pilot_flame: None,
            inside_thermal_boundary: true,
            manufacture_year: Some(1990),
            installation_year: None,
        };
        let (result, input) = calculated(&survey);
        let boiler = &input["spaceHeating"]["generator"]["boiler"];
        assert_eq!(boiler["kind"], "conventional");
        assert_eq!(boiler["fuel"], "oil");
        assert_eq!(boiler["pilotFlamePresent"], false);
        // Table 9.25: conventional, inside the boundary, 0,75.
        assert!((efficiency(&result) - 0.75).abs() < 1e-12);
        assert!(!rules(&result).contains(&"pilot_flame_unknown_present"));
    }

    #[test]
    fn local_and_air_heaters_follow_table_9_25_other_systems() {
        let mut survey = fixture("1930");
        survey.heating.emitters = heating::Emitters::LocalHeaters;
        survey.heating.generator = heating::HeatingGenerator::LocalFired {
            appliance: heating::LocalFiredAppliance::GasHeater,
            fuel: None,
            flue_gas_exhaust: true,
            electricity_connected: None,
        };
        let (result, input) = calculated(&survey);
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(generator["kind"], "forfait_heater");
        assert_eq!(generator["heaterKind"], "local_with_flue");
        assert_eq!(generator["fuel"], "natural_gas");
        assert!((efficiency(&result) - 0.65).abs() < 1e-12);
        assert!(rules(&result).contains(&"fired_heater_electricity_unknown_connected"));

        // Without a flue: 0,10.
        survey.heating.generator = heating::HeatingGenerator::LocalFired {
            appliance: heating::LocalFiredAppliance::OilHeater,
            fuel: None,
            flue_gas_exhaust: false,
            electricity_connected: Some(false),
        };
        let (result, input) = calculated(&survey);
        assert_eq!(input["spaceHeating"]["generator"]["fuel"], "oil");
        assert!((efficiency(&result) - 0.10).abs() < 1e-12);

        // A steam boiler needs its fuel.
        survey.heating.generator = heating::HeatingGenerator::LocalFired {
            appliance: heating::LocalFiredAppliance::SteamBoiler,
            fuel: None,
            flue_gas_exhaust: true,
            electricity_connected: Some(true),
        };
        let missing = assess_residential_survey(&survey);
        assert!(missing
            .issues
            .iter()
            .any(|item| item.code == "steam_boiler_fuel_required"));

        // Three HR-107 air heaters with unknown pilot flames (table 9.3) and
        // their count from table 9.16.
        survey.heating.emitters = heating::Emitters::AirHeating;
        survey.heating.air_heating = Some(heating::AirHeatingAnswer::Direct {
            radial_fan: Some(false),
            count: Some(3),
        });
        survey.heating.generator = heating::HeatingGenerator::GasAirHeater {
            heater_type: heating::AirHeaterType::Hr107,
            pilot_flame: None,
            count: None,
        };
        let (result, input) = calculated(&survey);
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(generator["heaterKind"], "air_heater_hr107");
        assert_eq!(generator["pilotFlames"], 3);
        assert_eq!(generator["auxiliary"]["electricallyConnectedDevices"], 3);
        assert!((efficiency(&result) - 0.95).abs() < 1e-12);
        assert!(rules(&result).contains(&"pilot_flame_unknown_present"));
    }

    #[test]
    fn heat_pump_sources_follow_table_9_6() {
        let mut survey = fixture("2015");
        // Heat-pump panel: the outdoor-air row (NTA p. 336 note 3).
        survey.heating.generator = heat_pump(heating::HeatPumpSource::HeatPumpPanel);
        let (result, input) = calculated(&survey);
        assert_eq!(
            input["spaceHeating"]["generator"]["forfait"]["source"],
            "outdoor_air"
        );
        assert!(rules(&result).contains(&"heat_pump_panel_outdoor_air_row"));

        // Groundwater without a known temperature: the ground row (p. 335);
        // with 11 °C from design data: the groundwater row.
        survey.heating.generator = heat_pump(heating::HeatPumpSource::Groundwater);
        let (result, input) = calculated(&survey);
        assert_eq!(
            input["spaceHeating"]["generator"]["forfait"]["source"],
            "ground"
        );
        assert!(rules(&result).contains(&"source_temperature_unknown_ground"));
        if let heating::HeatingGenerator::HeatPump {
            source_temperature_c,
            ..
        } = &mut survey.heating.generator
        {
            *source_temperature_c = Some(11.0);
        }
        let (_, input) = calculated(&survey);
        let forfait = &input["spaceHeating"]["generator"]["forfait"];
        assert_eq!(forfait["source"], "groundwater_below15_c");
        assert_eq!(forfait["sourceTemperatureC"], 11.0);
        // Individual source: no doublet correction.
        assert_eq!(forfait["sourceCorrectionFactor"], 1.0);
    }

    #[test]
    fn collective_groundwater_doublet_and_high_temperature_sources() {
        let mut survey = fixture("2015");
        let collective = |source, temperature, system| heating::HeatingGenerator::HeatPump {
            source,
            air_sink: false,
            high_temperature: false,
            capacity_kw: Some(6.0),
            source_regeneration_factor: None,
            high_efficiency_evidence: None,
            drive: Some(heating::HeatPumpDrive::Electric),
            groundwater_system: system,
            collective_source_reference: Some("invoice heat-pump source".into()),
            source_temperature_c: temperature,
            source_temperature_reference: Some("design data".into()),
            source_quality_declaration_reference: Some("declaration".into()),
            manufacture_year: None,
            installation_year: None,
        };
        survey.heating.generator = collective(
            heating::HeatPumpSource::Groundwater,
            Some(12.0),
            Some(heating::GroundwaterSystem::Doublet),
        );
        let (result, input) = calculated(&survey);
        let generator = &input["spaceHeating"]["generator"];
        // Table V.3: doublet 1,04; 9.62 with f_cor.bron.col 0,022.
        assert_eq!(generator["forfait"]["sourceCorrectionFactor"], 1.04);
        assert_eq!(
            generator["sourceSystem"],
            "collective_groundwater_surface_or_at_least15_c"
        );
        assert_eq!(
            input["externalSupply"]["collectiveHeatPumpSource"]["temperatureClass"],
            "below20_c"
        );
        // Table 9.27 groundwater row × 1,04 (45 °C: 4,1; 55 °C: 3,7).
        let supply = generator["forfait"]["designSupplyTemperatureC"]
            .as_f64()
            .unwrap();
        let row = if supply > 45.0 { 3.7 } else { 4.1 };
        assert!(
            (efficiency(&result) - row * 1.04).abs() < 1e-9,
            "{supply} {}",
            efficiency(&result)
        );

        // A source of 25 °C with a quality declaration: the 20–40 °C row.
        survey.heating.generator =
            collective(heating::HeatPumpSource::HighTemperature, Some(25.0), None);
        let (result, input) = calculated(&survey);
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(generator["forfait"]["source"], "collective20_to40_c");
        assert_eq!(input["heatPumpRenewable"]["sourceBelow20C"], false);
        assert_eq!(
            input["externalSupply"]["collectiveHeatPumpSource"]["temperatureClass"],
            "at_least20_c_or_surface_water_or_unknown"
        );
        let supply = generator["forfait"]["designSupplyTemperatureC"]
            .as_f64()
            .unwrap();
        let row = if supply > 45.0 { 4.0 } else { 4.5 };
        assert!((efficiency(&result) - row).abs() < 1e-9, "{supply}");

        // An individual high-temperature source does not exist (p. 111).
        if let heating::HeatingGenerator::HeatPump {
            collective_source_reference,
            ..
        } = &mut survey.heating.generator
        {
            *collective_source_reference = None;
        }
        let invalid = assess_residential_survey(&survey);
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "high_temperature_source_collective_only"));

        // A ticked collective source without its evidence (p. 111) is an
        // issue, not a silent individual source.
        survey.heating.generator = collective(
            heating::HeatPumpSource::Groundwater,
            Some(12.0),
            Some(heating::GroundwaterSystem::Doublet),
        );
        if let heating::HeatingGenerator::HeatPump {
            collective_source_reference,
            ..
        } = &mut survey.heating.generator
        {
            *collective_source_reference = Some("  ".into());
        }
        let invalid = assess_residential_survey(&survey);
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "collective_source_reference_required"));
    }

    #[test]
    fn large_or_collective_heat_pumps_use_table_9_29() {
        let mut survey = fixture("1975");
        survey.heating.generator = heat_pump(heating::HeatPumpSource::OutdoorAir);
        survey.heating.nominal_power_kw = Some(120.0);
        survey.heating.collective = Some(heating::CollectiveHeating {
            connected_usable_area_m2: None,
            connected_dwellings: Some(40),
            connected_storeys: Some(4),
            heat_meters_present: Some(true),
        });
        if let heating::HeatingGenerator::HeatPump { capacity_kw, .. } =
            &mut survey.heating.generator
        {
            *capacity_kw = Some(120.0);
        }
        let (result, input) = calculated(&survey);
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(
            generator["forfait"]["scope"],
            "utility_collective_or_over25_kw"
        );
        assert_eq!(generator["auxiliary"]["nominalPowerKw"], 120.0);
        assert!(rules(&result).contains(&"heat_pump_table_9_29"));
        // Table 9.29 outdoor air at 55 °C: 2,80.
        assert!((efficiency(&result) - 2.8).abs() < 1e-9);

        // p. 111: surface water is a choice for a collective installation,
        // also without a collective source (table 9.29 surface-water row).
        survey.heating.generator = heat_pump(heating::HeatPumpSource::SurfaceWater);
        if let heating::HeatingGenerator::HeatPump { capacity_kw, .. } =
            &mut survey.heating.generator
        {
            *capacity_kw = Some(120.0);
        }
        let (result, input) = calculated(&survey);
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(generator["forfait"]["source"], "surface_water");
        assert!(!rules(&result).contains(&"individual_surface_water_as_ground"));
        let supply = generator["forfait"]["designSupplyTemperatureC"]
            .as_f64()
            .unwrap();
        // Table 9.29 surface water: 3,3 at 55 °C, 3,7 at 45 °C.
        let row = if supply > 45.0 { 3.3 } else { 3.7 };
        assert!((efficiency(&result) - row).abs() < 1e-9, "{supply}");
    }

    #[test]
    fn gas_heat_pumps_use_the_gwp_rows() {
        let mut survey = fixture("2015");
        survey.heating.generator = heating::HeatingGenerator::HeatPump {
            source: heating::HeatPumpSource::Ground,
            air_sink: false,
            high_temperature: false,
            capacity_kw: Some(20.0),
            source_regeneration_factor: None,
            high_efficiency_evidence: None,
            drive: Some(heating::HeatPumpDrive::GasAbsorption),
            groundwater_system: None,
            collective_source_reference: None,
            source_temperature_c: None,
            source_temperature_reference: None,
            source_quality_declaration_reference: None,
            manufacture_year: None,
            installation_year: None,
        };
        // NTA p. 334: the table 9.27 GWP rows cover a collective building
        // installation only; an individual dwelling unit up to 25 kW has no
        // forfait row.
        let individual = assess_residential_survey(&survey);
        assert!(individual
            .issues
            .iter()
            .any(|item| item.code == "gas_heat_pump_individual_no_forfait_row"));
        // Above 25 kW: table 9.29.
        if let heating::HeatingGenerator::HeatPump { capacity_kw, .. } =
            &mut survey.heating.generator
        {
            *capacity_kw = Some(30.0);
        }
        let (_, input) = calculated(&survey);
        assert_eq!(
            input["spaceHeating"]["generator"]["table"],
            "utility_collective_or_above25_kw"
        );
        // A collective installation up to 25 kW: the table 9.27 GWP rows.
        if let heating::HeatingGenerator::HeatPump { capacity_kw, .. } =
            &mut survey.heating.generator
        {
            *capacity_kw = Some(20.0);
        }
        survey.heating.collective = Some(heating::CollectiveHeating {
            connected_usable_area_m2: None,
            connected_dwellings: Some(4),
            connected_storeys: Some(2),
            heat_meters_present: Some(true),
        });
        let (result, input) = calculated(&survey);
        let generator = &input["spaceHeating"]["generator"];
        assert_eq!(generator["kind"], "gas_heat_pump");
        assert_eq!(generator["table"], "residential_at_most25_kw");
        let class = input["spaceHeating"]["generator"]["designSupplyTemperatureC"]
            .as_f64()
            .unwrap();
        // Table 9.27 GWP ground row: 1,1 at 55 °C, 1,2 at 45 °C.
        let expected = if class > 45.0 { 1.1 } else { 1.2 };
        assert!((efficiency(&result) - expected).abs() < 1e-9);

        // Table 9.6 footnotes 4–6: no exhaust air for gas heat pumps.
        if let heating::HeatingGenerator::HeatPump { source, .. } = &mut survey.heating.generator {
            *source = heating::HeatPumpSource::ExhaustAir;
        }
        let invalid = assess_residential_survey(&survey);
        assert!(invalid
            .issues
            .iter()
            .any(|item| item.code == "gas_heat_pump_source_not_allowed"));
    }

    #[test]
    fn pipe_insulation_and_one_pipe_loop_reach_the_distribution() {
        let mut survey = fixture("1975");
        survey.heating.generator = boiler();
        survey.heating.nominal_power_kw = Some(300.0);
        survey.heating.collective = Some(heating::CollectiveHeating {
            connected_usable_area_m2: None,
            connected_dwellings: Some(40),
            connected_storeys: Some(4),
            heat_meters_present: Some(true),
        });
        let (plain, _) = calculated(&survey);
        survey.heating.pipe_insulation = Some(heating::PipeInsulationAnswer {
            insulated: true,
            insulation_year: Some(1990),
            fittings_insulated: Some(true),
        });
        survey.heating.distribution_type =
            Some(heating::DistributionTypeAnswer::OnePipe { emitter_count: 8 });
        let (result, input) = calculated(&survey);
        let system = &input["spaceHeating"]["distributionSystem"];
        assert_eq!(
            system["pipeTransmittance"]["insulation"]["period"],
            "from1980_to1995"
        );
        assert_eq!(system["valvesInsulated"], true);
        assert_eq!(system["pump"]["onePipeEmitterCount"], 8);
        assert!(!rules(&result).contains(&"pipe_insulation_unknown_uninsulated"));
        // Insulated pipes lose less; the one-pipe loop needs more pump energy.
        let chain = |item: &OpnameAssessment| {
            let months = &item.performance.as_ref().unwrap().space_heating.monthly;
            let loss: f64 = months.iter().map(|month| month.distribution_loss_kwh).sum();
            let pump: f64 = months
                .iter()
                .map(|month| month.distribution_auxiliary_electricity_kwh)
                .sum();
            (loss, pump)
        };
        let (plain_loss, plain_pump) = chain(&plain);
        let (loss, pump) = chain(&result);
        assert!(loss < plain_loss, "{loss} vs {plain_loss}");
        assert!(pump > plain_pump, "{pump} vs {plain_pump}");

        // Insulation year unknown: the construction year (1975 → before 1980).
        survey.heating.pipe_insulation = Some(heating::PipeInsulationAnswer {
            insulated: true,
            insulation_year: None,
            fittings_insulated: None,
        });
        survey.heating.distribution_type = Some(heating::DistributionTypeAnswer::RenovatedOnePipe);
        let (result, input) = calculated(&survey);
        let system = &input["spaceHeating"]["distributionSystem"];
        assert_eq!(
            system["pipeTransmittance"]["insulation"]["period"],
            "before1980_or_unknown"
        );
        assert_eq!(system["valvesInsulated"], false);
        assert!(system["pump"]["onePipeEmitterCount"].is_null());
        let applied = rules(&result);
        assert!(applied.contains(&"pipe_insulation_year_unknown_construction_year"));
        assert!(applied.contains(&"pipe_fittings_unknown_uninsulated"));
        assert!(applied.contains(&"renovated_one_pipe_as_two_pipe"));
    }

    /// RVO Voorbeeldwoningen 2022 (pakket "huidig"), see
    /// docs/nta8800-vergelijking-rvo-voorbeeldwoningen.md. Not an official
    /// reference test: the survey route must reproduce RVO's Standaard voor
    /// woningisolatie (§5.3.2), and RVO's typification (U values, ΔU_for 0)
    /// must stay within the documented ±6 % of RVO's Q_H,nd.
    #[test]
    fn rvo_voorbeeldwoningen_stay_within_the_documented_band() {
        let fixtures = [
            include_str!(
                "../../../../training-data/nta8800-rvo-voorbeeldwoningen-tussenwoning-1965-1974.json"
            ),
            include_str!(
                "../../../../training-data/nta8800-rvo-voorbeeldwoningen-hoekwoning-1946-1964.json"
            ),
            include_str!(
                "../../../../training-data/nta8800-rvo-voorbeeldwoningen-vrijstaand-1975-1991.json"
            ),
            include_str!(
                "../../../../training-data/nta8800-rvo-voorbeeldwoningen-twee-onder-een-kap-1992-2005.json"
            ),
            include_str!(
                "../../../../training-data/nta8800-rvo-voorbeeldwoningen-portiekwoning-1965-1974.json"
            ),
            include_str!(
                "../../../../training-data/nta8800-rvo-voorbeeldwoningen-galerijwoning-1975-1991.json"
            ),
        ];
        for raw in fixtures {
            let doc: Value = serde_json::from_str(raw).unwrap();
            let title = doc["title"].as_str().unwrap();
            let number = |value: &Value| value.as_f64().unwrap();
            let rvo_need = number(&doc["rvo"]["heatingNeedKwhPerM2"]);
            let rvo_standard = number(&doc["rvo"]["standardInsulationKwhPerM2"]);

            let survey: ResidentialSurvey = serde_json::from_value(doc["survey"].clone()).unwrap();
            let result = assess_residential_survey(&survey);
            assert_eq!(
                result.status, "calculated_unverified",
                "{title}: {:?}",
                result.issues
            );
            let performance = result.performance.as_ref().unwrap();
            assert!(
                !performance
                    .warnings
                    .iter()
                    .any(|item| item.code == "standard_insulation_construction_year_missing"),
                "{title}"
            );
            let chapter5 = performance.chapter5.as_ref().unwrap();
            let standard = chapter5.standard_insulation_kwh_per_m2.unwrap();
            assert!(
                (standard - rvo_standard).abs() <= 0.5,
                "{title}: {standard} vs {rvo_standard}"
            );
            let survey_need = number(&doc["regression"]["surveyHeatingNeedKwhPerM2"]);
            assert!(
                (chapter5.heating_need_kwh_per_m2 - survey_need).abs() <= 0.005 * survey_need,
                "{title}: survey {} vs recorded {survey_need}",
                chapter5.heating_need_kwh_per_m2
            );

            let input: BuildingPerformanceInput =
                serde_json::from_value(doc["performanceInput"].clone()).unwrap();
            let typified = assess_building_performance(&input);
            assert_eq!(typified.status, "calculated_unverified", "{title}");
            let need = typified.chapter5.as_ref().unwrap().heating_need_kwh_per_m2;
            assert!(
                (need - rvo_need).abs() <= 0.06 * rvo_need,
                "{title}: {need} vs RVO {rvo_need}"
            );
            let recorded = number(&doc["regression"]["performanceInputHeatingNeedKwhPerM2"]);
            assert!(
                (need - recorded).abs() <= 0.005 * recorded,
                "{title}: {need} vs {recorded}"
            );
        }
    }

    #[test]
    fn survey_passes_the_construction_year_to_the_kernel() {
        for name in ["1930", "1975", "2015"] {
            let survey = fixture(name);
            let mut recorder = Recorder::default();
            let input = derive_residential_input(&survey, &mut recorder).unwrap();
            assert_eq!(
                input["constructionYear"].as_u64(),
                u64::try_from(survey.construction_year).ok(),
                "{name}"
            );
            let result = assess_residential_survey(&survey);
            assert!(!result
                .performance
                .as_ref()
                .unwrap()
                .warnings
                .iter()
                .any(|item| item.code == "standard_insulation_construction_year_missing"));
        }
    }

    fn without_heat_pump_capacity(survey: &mut ResidentialSurvey) {
        if let heating::HeatingGenerator::HeatPump { capacity_kw, .. } =
            &mut survey.heating.generator
        {
            *capacity_kw = None;
        } else {
            panic!("fixture has no heat pump");
        }
        survey.heating.nominal_power_kw = None;
    }

    #[test]
    fn individual_heat_pump_without_capacity_uses_table_9_27() {
        let mut survey = fixture("2015");
        without_heat_pump_capacity(&mut survey);
        let result = assess_residential_survey(&survey);
        assert_eq!(
            result.status, "calculated_unverified",
            "{:?}",
            result.issues
        );
        assert!(rules(&result).contains(&"heat_pump_capacity_unknown_table_9_27"));
    }

    #[test]
    fn collective_heat_pump_without_capacity_names_the_field() {
        let mut survey = fixture("2015");
        without_heat_pump_capacity(&mut survey);
        survey.heating.collective = Some(heating::CollectiveHeating {
            connected_usable_area_m2: None,
            connected_dwellings: Some(20),
            connected_storeys: Some(4),
            heat_meters_present: Some(true),
        });
        let result = assess_residential_survey(&survey);
        assert_ne!(result.status, "calculated_unverified");
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "heat_pump_capacity_required"
                && item.path == "heating.generator.capacityKw"));
    }

    #[test]
    fn a_rejected_derived_input_always_carries_a_reason() {
        let survey = fixture("1930");
        let mut recorder = Recorder::default();
        let mut input = derive_residential_input(&survey, &mut recorder).unwrap();
        input["bacsFactor"] = json!(-1.0);
        let input: BuildingPerformanceInput = serde_json::from_value(input).unwrap();
        let performance = assess_building_performance(&input);
        assert_ne!(performance.status, "calculated_unverified");
        let mut issues = Vec::new();
        surface_rejection("derived_input_rejected", Some(&performance), &mut issues);
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .all(|item| item.path.starts_with("derivedInput.")
                || item.code == "derived_input_rejected_without_reason"));
        // A survey that already explains itself keeps its own issues.
        let mut own = vec![OpnameIssue {
            code: "own",
            path: "x".into(),
        }];
        surface_rejection("derived_input_rejected", Some(&performance), &mut own);
        assert_eq!(own.len(), 1);
    }

    /// The survey follows its edition: 2024 gives the legacy status, the
    /// protocol warning and a 2024 building run; 2025+C1 is unchanged.
    #[test]
    fn survey_is_calculated_in_its_edition() {
        let current = assess_residential_survey(&fixture("1975"));
        let mut survey = fixture("1975");
        survey.norm_version = NormVersion::V2024;
        let legacy = assess_residential_survey(&survey);
        assert_eq!(current.status, "calculated_unverified");
        assert_eq!(current.norm_version, NormVersion::V2025C1);
        assert!(current.registration_eligible);
        assert!(!current
            .warnings
            .iter()
            .any(|item| item.code == "survey_protocol_edition_differs"));
        assert_eq!(
            legacy.status, "calculated_legacy_edition",
            "{:?}",
            legacy.issues
        );
        assert_eq!(legacy.norm_version, NormVersion::V2024);
        assert_eq!(legacy.target_norm_version, "NTA 8800:2024 met INT-V1:2024");
        assert!(!legacy.registration_eligible);
        assert!(legacy
            .warnings
            .iter()
            .any(|item| item.code == "survey_protocol_edition_differs"));
        let input = legacy.derived_input.as_ref().unwrap();
        assert_eq!(input.norm_version, NormVersion::V2024);
        let (now, then) = (
            current.performance.as_ref().unwrap(),
            legacy.performance.as_ref().unwrap(),
        );
        assert_eq!(then.norm_version, NormVersion::V2024);
        assert!(!then.registration_eligible);
        // 2025+C1 adds the chapter 5 indicators (EP, final energy).
        assert!(now.chapter5.is_some());
        assert!(then.chapter5.is_none());
        // The serialized survey carries the edition only when it is older.
        let value = serde_json::to_value(&survey).unwrap();
        assert_eq!(value["normVersion"], "2024");
        assert!(serde_json::to_value(fixture("1975"))
            .unwrap()
            .get("normVersion")
            .is_none());
        // The thread's edition is restored after the run.
        assert_eq!(norm_versions::current(), NormVersion::V2025C1);
    }

    /// An edition without a kernel profile is refused with the building
    /// route's reason.
    #[test]
    fn survey_in_unimplemented_edition_is_refused() {
        let mut survey = fixture("1930");
        survey.norm_version = NormVersion::V2020A1;
        let result = assess_residential_survey(&survey);
        if NormVersion::V2020A1.implemented() {
            return;
        }
        assert_eq!(result.status, "derived_input_rejected");
        assert!(
            result
                .issues
                .iter()
                .any(|item| item.code == "edition_not_implemented"),
            "{:?}",
            result.issues
        );
        assert!(!result.registration_eligible);
    }
}
