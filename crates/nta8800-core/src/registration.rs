//! Registration data of an energy performance report and the deadline
//! checks an EP adviser works under.
//!
//! Sources (page numbers only, no text): BRL 9500-W draft 14-10-2025
//! §4.2.3 (improvements within 24 months, p. 23), §4.2.4 (relabelling with
//! the original software version and survey date, p. 23–24), §4.2.5
//! (registration within three months, six for serial projects; names and
//! competence numbers of both advisers; toets Bbl: survey date equals
//! registration date, p. 24); Besluit energieprestatie gebouwen art. 2.1
//! lid 7 (label valid ten years from the survey date, p. 4); Regeling
//! energieprestatie gebouwen art. 4 and 5 (label and registration data,
//! p. 5–6); BRL 9500-W §3.1 (mandatory detailed survey, p. 14–15; BRL
//! 9500-U §3.1 p. 11) and Bijlage 3 (project dossier with evidence,
//! p. 61–63). BRL 9500-U mirrors these clauses.
//!
//! Added on 3 October 2026:
//! - the attested software of the registration (Regeling art. 5 lid 1 onder b,
//!   p. 6; Regeling art. 2/3 require a BRL 9501-attested program, p. 4–5);
//! - the separate relabel message type and the replacement of an incorrect
//!   label within 24 months (BRL 9500-W §4.2.5 opmerking 4 and 5, p. 24–25);
//! - the WLC-GWP result for new buildings over 1000 m² checked against the
//!   Bbl from 1-1-2028 (BRL 9500-W p. 18, 21 and 62);
//! - the BAG addressable object as the lowest registration level of a
//!   residential label (Praktijkhandboek v2 p. 46); other buildings may use
//!   the pand or verblijfsobject id (Regeling art. 5 lid 1 onder a, p. 6);
//!   A_g to two decimals (Praktijkhandboek p. 70);
//! - plausibility warnings modelled on the dossier selection of BRL
//!   9500-W §7.2.2 (p. 42). Their thresholds are this program's own choice.
//!
//! The kernel checks the data; it does not register anything. EP-Online
//! registration needs the RVO exchange specification.

use crate::label_class::{class_rank, class_upper_bound, LabelFunction};
use crate::label_data::{ElementCategory, EnvelopeSummary};
use crate::KERNEL_VERSION;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

pub const REGISTRATION_SOURCE: &str = "BRL 9500-W/U (14-10-2025) §4.2.3–4.2.5; Besluit energieprestatie gebouwen art. 2.1 lid 7; Regeling energieprestatie gebouwen art. 4–5";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationPurpose {
    /// Existing building (label).
    ExistingBuilding,
    /// Report at completion (oplevering).
    Delivery,
    /// Check against the Bbl requirements (toets Bbl).
    BblCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SurveyType {
    Basic,
    Detailed,
}

/// Representativity of BRL 9500-W §4.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Representation {
    Unique,
    Reference,
    Similar,
}

/// Message type of the registration (BRL 9500-W §4.2.5 opmerking 4 and 5,
/// p. 24–25): a regular registration, the separate relabel message
/// (improvements within 24 months, §4.2.3) or the replacement of an
/// incorrect label through EP-Online replacement rights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Regular,
    Relabel,
    Replacement,
}

/// The program that made the calculation (Regeling art. 5 lid 1 onder b, p. 6).
/// The application fills it in; the attest number stays empty until the
/// program is attested under BRL 9501.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SoftwareIdentity {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub attest_number: Option<String>,
}

/// Outcome of the WLC-GWP calculation that the EP adviser enters (BRL
/// 9500-W p. 21). The calculation itself is outside the BRL scope.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WlcGwp {
    /// kg CO2-eq per m² usable floor area per year.
    #[serde(default)]
    pub value_kg_co2_eq_per_m2_year: Option<f64>,
    /// The WLC-GWP report in the project dossier.
    #[serde(default)]
    pub report_reference: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Advisor {
    #[serde(default)]
    pub name: String,
    /// Vakbekwaamheidsnummer.
    #[serde(default)]
    pub competence_number: String,
}

/// Situations of BRL 9500-W/U §3.1 that make the detailed survey
/// mandatory, besides toets Bbl and delivery (purpose) and a completion
/// after 1-1-2021 (`completionDate`/`constructionYear`).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DetailSurveyTriggers {
    /// Renewal after demolition keeping only foundations and floors.
    #[serde(default)]
    pub rebuilt_after_demolition: bool,
    /// Full renovation with new-build requirements after 1-1-2021.
    #[serde(default)]
    pub full_renovation_with_new_build_requirements: bool,
    /// Energy performance fee (EPV), residential only.
    #[serde(default)]
    pub energy_performance_fee: bool,
    /// Showing that an improvement reaches a BENG requirement.
    #[serde(default)]
    pub beng_requirement_proof: bool,
    /// An earlier report with a detailed survey was registered.
    #[serde(default)]
    pub previous_detailed_registration: bool,
    /// Dwelling added after 2021 to a building realised before 2021.
    #[serde(default)]
    pub added_after_2021: bool,
}

impl DetailSurveyTriggers {
    fn any(&self) -> bool {
        self.rebuilt_after_demolition
            || self.full_renovation_with_new_build_requirements
            || self.energy_performance_fee
            || self.beng_requirement_proof
            || self.previous_detailed_registration
            || self.added_after_2021
    }
}

/// Type of evidence in the project dossier (Bijlage 3 and 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    PhotoOverview,
    PhotoDetail,
    Invoice,
    Drawing,
    Datasheet,
    DeclarationOfPerformance,
    QualityDeclaration,
    /// Data supplied by the client (Bijlage 4 form or e-mail).
    ClientStatement,
    Other,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GpsPosition {
    pub latitude: f64,
    pub longitude: f64,
}

/// One evidence file. The kernel keeps its identity (hash) and metadata;
/// the file itself goes into the dossier export.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvidenceItem {
    pub id: String,
    pub kind: EvidenceKind,
    pub file_name: String,
    /// SHA-256 of the file, lowercase hex.
    pub sha256: String,
    /// Date the photo or document was made, YYYY-MM-DD.
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub gps: Option<GpsPosition>,
    /// Who supplied it (adviser, client, installer, …).
    #[serde(default)]
    pub source_party: Option<String>,
    /// Adviser who checked it (BRL 9500 §4.2.7).
    #[serde(default)]
    pub checked_by: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// JSON pointers into the project that this evidence supports.
    #[serde(default)]
    pub linked_paths: Vec<String>,
    /// Local copy of the file in the desktop app.
    #[serde(default)]
    pub stored_path: Option<String>,
}

/// All fields are optional so projects without registration data stay
/// valid; the assessment lists what is still missing for registration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Registration {
    #[serde(default)]
    pub purpose: Option<RegistrationPurpose>,
    #[serde(default)]
    pub survey_type: Option<SurveyType>,
    #[serde(default)]
    pub representation: Option<Representation>,
    /// Reference dwelling for `similar`.
    #[serde(default)]
    pub reference_object_id: Option<String>,
    /// BAG verblijfsobject id.
    #[serde(default)]
    pub bag_object_id: Option<String>,
    #[serde(default)]
    pub postcode: Option<String>,
    #[serde(default)]
    pub house_number: Option<String>,
    #[serde(default)]
    pub house_number_addition: Option<String>,
    #[serde(default)]
    pub construction_year: Option<u32>,
    /// Dwelling type or use function as shown on the label (Reg. art 4).
    #[serde(default)]
    pub building_type: Option<String>,
    #[serde(default)]
    pub client: Option<String>,
    #[serde(default)]
    pub certificate_number: Option<String>,
    #[serde(default)]
    pub surveying_advisor: Option<Advisor>,
    #[serde(default)]
    pub registering_advisor: Option<Advisor>,
    /// Opnamedatum, YYYY-MM-DD. For a relabel the original survey date.
    #[serde(default)]
    pub survey_date: Option<String>,
    /// Registratiedatum, YYYY-MM-DD (planned or actual).
    #[serde(default)]
    pub registration_date: Option<String>,
    /// Serial new build or renovation project: six months (§4.2.5).
    #[serde(default)]
    pub serial_project: bool,
    /// Relabel message type (§4.2.3/4.2.4). Kept for saved projects;
    /// `messageType` supersedes it.
    #[serde(default)]
    pub relabel: bool,
    /// Message type; without it `relabel` decides between regular and
    /// relabel.
    #[serde(default)]
    pub message_type: Option<MessageType>,
    /// Replacement: EP-Online number of the label that is replaced.
    #[serde(default)]
    pub replaced_ep_online_number: Option<String>,
    /// The program that made the calculation (Regeling art. 5 lid 1 onder b).
    #[serde(default)]
    pub software: Option<SoftwareIdentity>,
    /// WLC-GWP result (BRL 9500-W p. 18, 21, 62).
    #[serde(default)]
    pub wlc_gwp: Option<WlcGwp>,
    /// Delivery: date of the toets Bbl the delivery follows, YYYY-MM-DD.
    /// The WLC-GWP duty from 1-1-2028 follows that check (BRL 9500-W p. 21,
    /// 62).
    #[serde(default)]
    pub bbl_check_date: Option<String>,
    /// A_g of the whole building, m², when the calculation covers only part
    /// of it (one dwelling); the WLC-GWP threshold is per building.
    #[serde(default)]
    pub building_usable_floor_area_m2: Option<f64>,
    /// Class of the label that was registered before, for the plausibility
    /// check on large class jumps.
    #[serde(default)]
    pub previous_label_class: Option<String>,
    /// Relabel: date of the improvement, YYYY-MM-DD, from the quote with
    /// order or the specified invoice (§4.2.3, p. 23); it must lie within
    /// 24 months of the original survey date.
    #[serde(default)]
    pub improvement_date: Option<String>,
    /// Kernel version of the original calculation, for a relabel.
    #[serde(default)]
    pub original_kernel_version: Option<String>,
    /// EP-Online number, entered after registration.
    #[serde(default)]
    pub ep_online_number: Option<String>,
    /// Opleverdatum, YYYY-MM-DD (§3.1: after 1-1-2021 detailed survey).
    #[serde(default)]
    pub completion_date: Option<String>,
    #[serde(default)]
    pub detail_survey_triggers: DetailSurveyTriggers,
    /// Evidence register of the project dossier (Bijlage 3).
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationIssue {
    pub code: &'static str,
    pub path: String,
    /// `error`: registration not allowed as entered; `missing`: data still
    /// needed before registration; `warning`: plausibility finding that
    /// does not block registration.
    pub severity: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationAssessment {
    pub source: &'static str,
    /// Effective message type (`messageType`, else `relabel`).
    pub message_type: MessageType,
    /// Survey date + 10 years (Bep art. 2.1 lid 7).
    pub valid_until: Option<String>,
    /// Last allowed registration date (§4.2.5); `None` for a relabel or a
    /// replacement.
    pub registration_deadline: Option<String>,
    /// Last date for a relabel (§4.2.3).
    pub relabel_deadline: Option<String>,
    /// Last date for a replacement (§4.2.5 opmerking 5).
    pub replacement_deadline: Option<String>,
    /// Whether the WLC-GWP result is required (new building > 1000 m²,
    /// toets Bbl or delivery from 1-1-2028); `None` when undecidable.
    pub wlc_gwp_required: Option<bool>,
    /// The program of the registration: the stored identity, or this
    /// program for projects saved before the identity was recorded.
    pub software: SoftwareIdentity,
    /// Whether that program carries a BRL 9501 attest number (Regeling
    /// art. 2/3, p. 4–5). Reported apart from `issues`: it is a property of
    /// the program, not of the dossier.
    pub software_attested: bool,
    pub ready_for_registration: bool,
    pub issues: Vec<RegistrationIssue>,
    /// Plausibility findings (severity `warning`); they never block
    /// registration.
    pub plausibility: Vec<RegistrationIssue>,
}

/// Calculation results the registration checks need; filled by the
/// project route. Every field is optional so the block can be checked on
/// its own.
#[derive(Debug, Clone, Default)]
pub struct RegistrationContext {
    /// A_g of the calculation, m².
    pub usable_floor_area_m2: Option<f64>,
    /// A_g per zone as entered, to check the two-decimal measuring rule.
    pub zone_floor_areas_m2: Vec<f64>,
    /// A_ls/A_g.
    pub loss_area_ratio: Option<f64>,
    pub residential: bool,
    /// Label function when the building has one (borderline check).
    pub label_function: Option<LabelFunction>,
    /// EP2 rounded to 0,01 kWh/m²·yr.
    pub primary_fossil_kwh_per_m2: Option<f64>,
    pub label_class: Option<&'static str>,
    pub envelope: Vec<EnvelopeSummary>,
}

/// Name of this program in the registration (Regeling art. 5 lid 1 onder b).
pub const SOFTWARE_NAME: &str = "Open Energy Studio";

/// First day on which the WLC-GWP result is required (BRL 9500-W p. 18).
const WLC_GWP_FROM: Date = Date {
    year: 2028,
    month: 1,
    day: 1,
};

/// Usable floor area above which the WLC-GWP result is required, m².
const WLC_GWP_AREA_M2: f64 = 1000.0;

/// Calendar date without time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if is_leap(year) => 29,
        _ => 28,
    }
}

impl Date {
    /// `YYYY-MM-DD`.
    pub fn parse(text: &str) -> Option<Date> {
        let mut parts = text.trim().split('-');
        let year: i32 = parts.next()?.parse().ok()?;
        let month: u32 = parts.next()?.parse().ok()?;
        let day: u32 = parts.next()?.parse().ok()?;
        if parts.next().is_some()
            || !(1900..=9999).contains(&year)
            || !(1..=12).contains(&month)
            || day == 0
            || day > days_in_month(year, month)
        {
            return None;
        }
        Some(Date { year, month, day })
    }

    /// Calendar months later; the day is clamped to the month end.
    pub fn add_months(self, months: u32) -> Date {
        let index = self.year * 12 + self.month as i32 - 1 + months as i32;
        let year = index.div_euclid(12);
        let month = index.rem_euclid(12) as u32 + 1;
        Date {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn issue(code: &'static str, path: &str, severity: &'static str) -> RegistrationIssue {
    RegistrationIssue {
        code,
        path: format!("registration.{path}"),
        severity,
    }
}

fn blank(value: &Option<String>) -> bool {
    value.as_deref().map_or(true, |text| text.trim().is_empty())
}

fn check_advisor(advisor: &Option<Advisor>, path: &str, issues: &mut Vec<RegistrationIssue>) {
    match advisor {
        None => issues.push(issue("advisor_required", path, "missing")),
        Some(advisor) => {
            if advisor.name.trim().is_empty() {
                issues.push(issue(
                    "advisor_name_required",
                    &format!("{path}.name"),
                    "missing",
                ));
            }
            if advisor.competence_number.trim().is_empty() {
                issues.push(issue(
                    "advisor_competence_number_required",
                    &format!("{path}.competenceNumber"),
                    "missing",
                ));
            }
        }
    }
}

impl Registration {
    /// `messageType`, else `relabel` (saved projects).
    pub fn effective_message_type(&self) -> MessageType {
        self.message_type.unwrap_or(if self.relabel {
            MessageType::Relabel
        } else {
            MessageType::Regular
        })
    }
}

/// BAG ids are 16 digits; digits 5–6 give the object type. Praktijkhandboek
/// v2 p. 46: a residential label is registered on an addressable object,
/// a verblijfsobject (01), ligplaats (02) or standplaats (03). Regeling
/// art. 5 lid 1 onder a (p. 6) also allows the pand id (10), which other
/// buildings may use.
fn check_bag_object_id(id: &str, residential: bool, issues: &mut Vec<RegistrationIssue>) {
    let id = id.trim();
    let allowed: &[&str] = if residential {
        &["01", "02", "03"]
    } else {
        &["01", "02", "03", "10"]
    };
    if id.len() != 16 || !id.chars().all(|c| c.is_ascii_digit()) {
        issues.push(issue("bag_object_id_invalid", "bagObjectId", "error"));
    } else if !allowed.contains(&&id[4..6]) {
        issues.push(issue(
            "bag_object_id_not_addressable",
            "bagObjectId",
            "error",
        ));
    }
}

/// The program of the registration (Regeling art. 5 lid 1 onder b, p. 6).
/// Projects saved before the identity was recorded get this program with
/// the kernel version; blank fields are filled the same way.
fn effective_software(software: &Option<SoftwareIdentity>) -> SoftwareIdentity {
    let stored = software.clone().unwrap_or_default();
    SoftwareIdentity {
        name: if stored.name.trim().is_empty() {
            SOFTWARE_NAME.to_owned()
        } else {
            stored.name
        },
        version: if stored.version.trim().is_empty() {
            KERNEL_VERSION.to_owned()
        } else {
            stored.version
        },
        attest_number: stored
            .attest_number
            .filter(|number| !number.trim().is_empty()),
    }
}

/// BRL 9500-W p. 18, 21 and 62: from 1-1-2028 a new building over
/// 1000 m² checked against the Bbl (and its later delivery) needs a
/// WLC-GWP calculation, whose result the adviser enters.
fn check_wlc_gwp(
    registration: &Registration,
    context: &RegistrationContext,
    issues: &mut Vec<RegistrationIssue>,
    warnings: &mut Vec<RegistrationIssue>,
) -> Option<bool> {
    if let Some(wlc) = &registration.wlc_gwp {
        if wlc
            .value_kg_co2_eq_per_m2_year
            .is_some_and(|value| !value.is_finite() || value <= 0.0)
        {
            issues.push(issue(
                "wlc_gwp_invalid",
                "wlcGwp.valueKgCo2EqPerM2Year",
                "error",
            ));
        }
    }
    // The duty follows the toets Bbl: a delivery inherits it from the
    // check it follows (p. 21, 62), so it is keyed on that check's date.
    let check_date = match registration.purpose {
        Some(RegistrationPurpose::BblCheck) => registration
            .registration_date
            .as_deref()
            .or(registration.survey_date.as_deref()),
        Some(RegistrationPurpose::Delivery) => match registration.bbl_check_date.as_deref() {
            Some(date) => Some(date),
            None => {
                let late = registration
                    .registration_date
                    .as_deref()
                    .or(registration.survey_date.as_deref())
                    .and_then(Date::parse)
                    .is_some_and(|date| date >= WLC_GWP_FROM);
                if late {
                    warnings.push(warning(
                        "wlc_gwp_bbl_check_date_unknown",
                        "registration.bblCheckDate",
                    ));
                }
                return None;
            }
        },
        _ => return Some(false),
    };
    let date = check_date.and_then(Date::parse)?;
    if date < WLC_GWP_FROM {
        return Some(false);
    }
    // The threshold is per building; a dwelling's own A_g decides only when
    // it already exceeds it.
    let area = match registration.building_usable_floor_area_m2 {
        Some(area) => area,
        None => {
            let area = context.usable_floor_area_m2?;
            if context.residential && area <= WLC_GWP_AREA_M2 {
                warnings.push(warning(
                    "wlc_gwp_building_area_unknown",
                    "registration.buildingUsableFloorAreaM2",
                ));
                return None;
            }
            area
        }
    };
    if area <= WLC_GWP_AREA_M2 {
        return Some(false);
    }
    let wlc = registration.wlc_gwp.clone().unwrap_or_default();
    if wlc.value_kg_co2_eq_per_m2_year.is_none() {
        issues.push(issue(
            "wlc_gwp_required",
            "wlcGwp.valueKgCo2EqPerM2Year",
            "missing",
        ));
    }
    if blank(&wlc.report_reference) {
        issues.push(issue(
            "wlc_gwp_reference_required",
            "wlcGwp.reportReference",
            "missing",
        ));
    }
    Some(true)
}

fn warning(code: &'static str, path: &str) -> RegistrationIssue {
    RegistrationIssue {
        code,
        path: path.to_owned(),
        severity: "warning",
    }
}

/// A label class as entered, trimmed and upper case ("a+" is "A+").
fn normalised_class(text: &str) -> String {
    text.trim().to_uppercase()
}

/// Plausibility warnings after BRL 9500-W §7.2.2 (p. 42): the certification
/// body selects dossiers on A+ labels made with the basic survey, on
/// borderline labels and on impossible or unrealistic values. The
/// thresholds below are this program's own; they never block registration.
pub fn plausibility_warnings(
    registration: &Registration,
    context: &RegistrationContext,
) -> Vec<RegistrationIssue> {
    let mut found = Vec::new();
    let a_plus = class_rank("A+").unwrap_or(usize::MAX);
    if let Some(class) = context.label_class {
        if registration.survey_type == Some(SurveyType::Basic)
            && class_rank(class).is_some_and(|rank| rank <= a_plus)
        {
            found.push(warning(
                "plausibility_high_class_basic_survey",
                "registration.surveyType",
            ));
        }
        if let (Some(function), Some(ep2)) =
            (context.label_function, context.primary_fossil_kwh_per_m2)
        {
            if let Some(bound) = class_upper_bound(function, class) {
                let margin = (0.01 * bound.abs()).max(1.0);
                if ep2 <= bound && bound - ep2 < margin {
                    found.push(warning(
                        "plausibility_borderline_label",
                        "performance.primaryFossilIndicatorKwhPerM2Year",
                    ));
                }
            }
        }
        if let Some(previous) = registration.previous_label_class.as_deref() {
            match (class_rank(&normalised_class(previous)), class_rank(class)) {
                (Some(before), Some(now)) if before >= now + 3 => found.push(warning(
                    "plausibility_label_class_jump",
                    "registration.previousLabelClass",
                )),
                _ => {}
            }
        }
    }
    if let Some(ep2) = context.primary_fossil_kwh_per_m2 {
        if !(-500.0..=1500.0).contains(&ep2) {
            found.push(warning(
                "plausibility_ep2_out_of_range",
                "performance.primaryFossilIndicatorKwhPerM2Year",
            ));
        }
    }
    if let Some(ratio) = context.loss_area_ratio {
        if !(0.2..=5.0).contains(&ratio) {
            found.push(warning(
                "plausibility_loss_area_ratio",
                "geometry.lossAreaRatio",
            ));
        }
    }
    if let Some(area) = context.usable_floor_area_m2 {
        let implausible = if context.residential {
            !(15.0..=1000.0).contains(&area)
        } else {
            area < 1.0
        };
        if implausible {
            found.push(warning(
                "plausibility_usable_floor_area",
                "geometry.usableFloorAreaM2",
            ));
        }
    }
    // Praktijkhandboek v2 p. 70: A_g is measured to two decimals.
    if context
        .zone_floor_areas_m2
        .iter()
        .any(|area| ((area * 100.0).round() - area * 100.0).abs() > 1e-6)
    {
        found.push(warning("usable_floor_area_precision", "zones"));
    }
    for summary in &context.envelope {
        let Some(u) = summary.mean_u_w_per_m2k else {
            continue;
        };
        let range = match summary.category {
            ElementCategory::Glazing => 0.4..=6.0,
            _ => 0.08..=4.5,
        };
        if !range.contains(&u) {
            found.push(warning(
                "plausibility_u_value",
                match summary.category {
                    ElementCategory::Facade => "labelData.envelope.facade",
                    ElementCategory::Roof => "labelData.envelope.roof",
                    ElementCategory::Floor => "labelData.envelope.floor",
                    ElementCategory::Glazing => "labelData.envelope.glazing",
                },
            ));
        }
    }
    found
}

pub fn assess_registration(registration: &Registration) -> RegistrationAssessment {
    assess_registration_with(registration, &RegistrationContext::default())
}

pub fn assess_registration_with(
    registration: &Registration,
    context: &RegistrationContext,
) -> RegistrationAssessment {
    let mut issues = Vec::new();
    let message_type = registration.effective_message_type();
    if registration.relabel && message_type != MessageType::Relabel {
        issues.push(issue("message_type_conflict", "messageType", "error"));
    }
    if registration.purpose.is_none() {
        issues.push(issue("purpose_required", "purpose", "missing"));
    }
    if registration.survey_type.is_none() {
        issues.push(issue("survey_type_required", "surveyType", "missing"));
    }
    match registration.representation {
        None => issues.push(issue(
            "representation_required",
            "representation",
            "missing",
        )),
        Some(Representation::Similar) if blank(&registration.reference_object_id) => issues.push(
            issue("reference_object_required", "referenceObjectId", "missing"),
        ),
        _ => {}
    }
    match registration.bag_object_id.as_deref() {
        Some(id) if !id.trim().is_empty() => {
            check_bag_object_id(id, context.residential, &mut issues)
        }
        _ => issues.push(issue("bag_object_id_required", "bagObjectId", "missing")),
    }
    if blank(&registration.postcode) {
        issues.push(issue("postcode_required", "postcode", "missing"));
    }
    if blank(&registration.house_number) {
        issues.push(issue("house_number_required", "houseNumber", "missing"));
    }
    if blank(&registration.client) {
        issues.push(issue("client_required", "client", "missing"));
    }
    if blank(&registration.certificate_number) {
        issues.push(issue(
            "certificate_number_required",
            "certificateNumber",
            "missing",
        ));
    }
    check_advisor(
        &registration.surveying_advisor,
        "surveyingAdvisor",
        &mut issues,
    );
    check_advisor(
        &registration.registering_advisor,
        "registeringAdvisor",
        &mut issues,
    );

    let survey = match registration.survey_date.as_deref() {
        None => {
            issues.push(issue("survey_date_required", "surveyDate", "missing"));
            None
        }
        Some(text) => {
            let date = Date::parse(text);
            if date.is_none() {
                issues.push(issue("survey_date_invalid", "surveyDate", "error"));
            }
            date
        }
    };
    let registered = match registration.registration_date.as_deref() {
        None => {
            issues.push(issue(
                "registration_date_required",
                "registrationDate",
                "missing",
            ));
            None
        }
        Some(text) => {
            let date = Date::parse(text);
            if date.is_none() {
                issues.push(issue(
                    "registration_date_invalid",
                    "registrationDate",
                    "error",
                ));
            }
            date
        }
    };

    let valid_until = survey.map(|date| date.add_months(120));
    let mut registration_deadline = None;
    let mut relabel_deadline = None;
    let mut replacement_deadline = None;
    if message_type == MessageType::Replacement && blank(&registration.replaced_ep_online_number) {
        issues.push(issue(
            "replaced_ep_online_number_required",
            "replacedEpOnlineNumber",
            "missing",
        ));
    }
    if let Some(survey) = survey {
        if message_type == MessageType::Replacement {
            // §4.2.5 opmerking 5 (p. 25): an incorrect label is replaced
            // through EP-Online replacement rights within 24 months of the
            // original survey; the survey date stays the original one.
            let deadline = survey.add_months(24);
            replacement_deadline = Some(deadline);
            if registered.is_some_and(|date| date > deadline) {
                issues.push(issue(
                    "replacement_deadline_exceeded",
                    "registrationDate",
                    "error",
                ));
            }
        } else if message_type == MessageType::Relabel {
            // §4.2.3/4.2.4: improvements within 24 months of the original
            // survey, calculated with the original kernel.
            // §4.2.3 (p. 23): the improvement, proven by a quote with
            // order or an invoice, must be made within 24 months of the
            // original survey date; the registration date does not count.
            let deadline = survey.add_months(24);
            relabel_deadline = Some(deadline);
            match registration.improvement_date.as_deref() {
                None => issues.push(issue(
                    "improvement_date_required",
                    "improvementDate",
                    "missing",
                )),
                Some(text) => match Date::parse(text) {
                    None => issues.push(issue(
                        "improvement_date_invalid",
                        "improvementDate",
                        "error",
                    )),
                    Some(date) if date > deadline => issues.push(issue(
                        "relabel_deadline_exceeded",
                        "improvementDate",
                        "error",
                    )),
                    Some(date) if date < survey => issues.push(issue(
                        "improvement_before_survey",
                        "improvementDate",
                        "error",
                    )),
                    Some(_) => {}
                },
            }
            match registration.original_kernel_version.as_deref() {
                None => issues.push(issue(
                    "original_kernel_version_required",
                    "originalKernelVersion",
                    "missing",
                )),
                Some(version) if version != KERNEL_VERSION => issues.push(issue(
                    "relabel_kernel_version_differs",
                    "originalKernelVersion",
                    "error",
                )),
                _ => {}
            }
        } else {
            // §4.2.5: three months, six for serial projects.
            let deadline = survey.add_months(if registration.serial_project { 6 } else { 3 });
            registration_deadline = Some(deadline);
            if registered.is_some_and(|date| date > deadline) {
                issues.push(issue(
                    "registration_deadline_exceeded",
                    "registrationDate",
                    "error",
                ));
            }
        }
        if let Some(registered) = registered {
            if registered < survey {
                issues.push(issue(
                    "registration_before_survey",
                    "registrationDate",
                    "error",
                ));
            }
            // §4.2.5 opmerking 2.
            if registration.purpose == Some(RegistrationPurpose::BblCheck) && registered != survey {
                issues.push(issue("bbl_check_dates_differ", "registrationDate", "error"));
            }
        }
    }
    // §4.2.3 (W p. 23, U p. 18): relabelling only for existing buildings.
    if message_type == MessageType::Relabel
        && registration
            .purpose
            .is_some_and(|purpose| purpose != RegistrationPurpose::ExistingBuilding)
    {
        issues.push(issue(
            "relabel_only_for_existing_buildings",
            "relabel",
            "error",
        ));
    }
    check_detail_survey(registration, &mut issues);
    check_evidence(&registration.evidence, &mut issues);
    let software = effective_software(&registration.software);
    let mut plausibility = plausibility_warnings(registration, context);
    let wlc_gwp_required = check_wlc_gwp(registration, context, &mut issues, &mut plausibility);
    // Only used for the class-jump warning, so an unknown class warns.
    if registration
        .previous_label_class
        .as_deref()
        .is_some_and(|class| {
            !class.trim().is_empty() && class_rank(&normalised_class(class)).is_none()
        })
    {
        plausibility.push(warning(
            "previous_label_class_invalid",
            "registration.previousLabelClass",
        ));
    }

    RegistrationAssessment {
        source: REGISTRATION_SOURCE,
        message_type,
        valid_until: valid_until.map(|date| date.to_string()),
        registration_deadline: registration_deadline.map(|date| date.to_string()),
        relabel_deadline: relabel_deadline.map(|date| date.to_string()),
        replacement_deadline: replacement_deadline.map(|date| date.to_string()),
        wlc_gwp_required,
        software_attested: software.attest_number.is_some(),
        software,
        ready_for_registration: issues.is_empty(),
        issues,
        plausibility,
    }
}

/// BRL 9500-W/U §3.1: situations in which a basic survey is not allowed.
fn check_detail_survey(registration: &Registration, issues: &mut Vec<RegistrationIssue>) {
    if registration.survey_type != Some(SurveyType::Basic) {
        return;
    }
    let start_2021 = Date {
        year: 2021,
        month: 1,
        day: 1,
    };
    let completed_after_2021 = registration
        .completion_date
        .as_deref()
        .and_then(Date::parse)
        .is_some_and(|date| date > start_2021)
        || registration
            .construction_year
            .is_some_and(|year| year >= 2021);
    let reasons = [
        (
            matches!(
                registration.purpose,
                Some(RegistrationPurpose::BblCheck | RegistrationPurpose::Delivery)
            ),
            "detail_survey_required_for_purpose",
            "purpose",
        ),
        (
            completed_after_2021,
            "detail_survey_required_completed_after_2021",
            "completionDate",
        ),
        (
            registration.detail_survey_triggers.any(),
            "detail_survey_required",
            "detailSurveyTriggers",
        ),
    ];
    for (applies, code, path) in reasons {
        if applies {
            issues.push(issue(code, path, "error"));
        }
    }
}

fn check_evidence(evidence: &[EvidenceItem], issues: &mut Vec<RegistrationIssue>) {
    let mut ids = HashSet::new();
    for (index, item) in evidence.iter().enumerate() {
        let path = format!("evidence[{index}]");
        if item.id.trim().is_empty() || !ids.insert(item.id.as_str()) {
            issues.push(issue("evidence_id_invalid", &format!("{path}.id"), "error"));
        }
        if item.file_name.trim().is_empty() {
            issues.push(issue(
                "evidence_file_name_required",
                &format!("{path}.fileName"),
                "error",
            ));
        }
        let hex = item.sha256.len() == 64
            && item
                .sha256
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
        if !hex {
            issues.push(issue(
                "evidence_hash_invalid",
                &format!("{path}.sha256"),
                "error",
            ));
        }
        if item
            .date
            .as_deref()
            .is_some_and(|date| Date::parse(date).is_none())
        {
            issues.push(issue(
                "evidence_date_invalid",
                &format!("{path}.date"),
                "error",
            ));
        }
        if let Some(gps) = &item.gps {
            let valid =
                (-90.0..=90.0).contains(&gps.latitude) && (-180.0..=180.0).contains(&gps.longitude);
            if !valid {
                issues.push(issue(
                    "evidence_gps_invalid",
                    &format!("{path}.gps"),
                    "error",
                ));
            }
        }
        if blank(&item.checked_by) {
            // §4.2.7: the adviser checks evidence supplied by others.
            issues.push(issue(
                "evidence_check_required",
                &format!("{path}.checkedBy"),
                "missing",
            ));
        }
        for (link, pointer) in item.linked_paths.iter().enumerate() {
            if !pointer.starts_with('/') {
                issues.push(issue(
                    "evidence_link_path_invalid",
                    &format!("{path}.linkedPaths[{link}]"),
                    "error",
                ));
            }
        }
    }
}

/// Marks an evidence id inside a `…Reference` text, e.g.
/// `"factuur, evidence:ev-3"`.
pub const EVIDENCE_REFERENCE_PREFIX: &str = "evidence:";

/// Cross-checks the evidence register against the project: linked paths
/// must exist, and every `evidence:<id>` in a `…Reference` field must name
/// a registered item. The registration block itself is skipped.
pub fn check_evidence_links(
    project: &Value,
    registration: &Registration,
) -> Vec<RegistrationIssue> {
    let mut issues = Vec::new();
    let ids: HashSet<&str> = registration
        .evidence
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    for (index, item) in registration.evidence.iter().enumerate() {
        for (link, pointer) in item.linked_paths.iter().enumerate() {
            if pointer.starts_with('/') && project.pointer(pointer).is_none() {
                issues.push(issue(
                    "evidence_link_path_unknown",
                    &format!("evidence[{index}].linkedPaths[{link}]"),
                    "error",
                ));
            }
        }
    }
    let mut stack: Vec<(String, &Value)> = vec![(String::new(), project)];
    while let Some((pointer, value)) = stack.pop() {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    if pointer.is_empty() && key == "registration" {
                        continue;
                    }
                    let child_pointer =
                        format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1"));
                    match child {
                        Value::String(text) if key.ends_with("Reference") => {
                            for id in referenced_evidence(text) {
                                if !ids.contains(id) {
                                    issues.push(RegistrationIssue {
                                        code: "evidence_reference_unknown",
                                        path: child_pointer.clone(),
                                        severity: "error",
                                    });
                                }
                            }
                        }
                        _ => stack.push((child_pointer, child)),
                    }
                }
            }
            Value::Array(items) => {
                for (index, child) in items.iter().enumerate() {
                    stack.push((format!("{pointer}/{index}"), child));
                }
            }
            _ => {}
        }
    }
    issues
}

fn referenced_evidence(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '(' | ')'))
        .filter_map(|token| token.strip_prefix(EVIDENCE_REFERENCE_PREFIX))
        .map(|id| id.trim_end_matches('.'))
        .filter(|id| !id.is_empty())
}

/// [`assess_registration`] plus the evidence cross-check against the
/// project the block belongs to.
pub fn assess_project_registration(
    registration: &Registration,
    project: &Value,
    context: &RegistrationContext,
) -> RegistrationAssessment {
    let mut result = assess_registration_with(registration, context);
    result
        .issues
        .extend(check_evidence_links(project, registration));
    result.ready_for_registration = result.issues.is_empty();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete() -> Registration {
        Registration {
            purpose: Some(RegistrationPurpose::ExistingBuilding),
            survey_type: Some(SurveyType::Basic),
            representation: Some(Representation::Unique),
            bag_object_id: Some("0363010000000001".into()),
            postcode: Some("1011AB".into()),
            house_number: Some("1".into()),
            client: Some("Eigenaar".into()),
            certificate_number: Some("K12345".into()),
            surveying_advisor: Some(Advisor {
                name: "A".into(),
                competence_number: "V1".into(),
            }),
            registering_advisor: Some(Advisor {
                name: "B".into(),
                competence_number: "V2".into(),
            }),
            survey_date: Some("2026-01-31".into()),
            registration_date: Some("2026-04-30".into()),
            software: Some(SoftwareIdentity {
                name: SOFTWARE_NAME.into(),
                version: "0.1.6-alpha".into(),
                attest_number: Some("TEST-ATTEST".into()),
            }),
            ..Registration::default()
        }
    }

    fn codes(registration: &Registration) -> Vec<&'static str> {
        assess_registration(registration)
            .issues
            .iter()
            .map(|item| item.code)
            .collect()
    }

    #[test]
    fn dates_and_month_arithmetic() {
        assert!(Date::parse("2026-02-29").is_none());
        assert!(Date::parse("2028-02-29").is_some());
        assert!(Date::parse("2026-13-01").is_none());
        let date = Date::parse("2026-01-31").unwrap();
        assert_eq!(date.add_months(1).to_string(), "2026-02-28");
        assert_eq!(date.add_months(3).to_string(), "2026-04-30");
        assert_eq!(date.add_months(120).to_string(), "2036-01-31");
        assert_eq!(
            Date::parse("2028-02-29")
                .unwrap()
                .add_months(120)
                .to_string(),
            "2038-02-28"
        );
    }

    #[test]
    fn complete_registration_within_three_months() {
        let result = assess_registration(&complete());
        assert!(result.ready_for_registration, "{:?}", result.issues);
        assert_eq!(result.valid_until.as_deref(), Some("2036-01-31"));
        assert_eq!(result.registration_deadline.as_deref(), Some("2026-04-30"));
    }

    #[test]
    fn deadlines_serial_bbl_and_relabel() {
        let mut late = complete();
        late.registration_date = Some("2026-05-01".into());
        assert!(codes(&late).contains(&"registration_deadline_exceeded"));
        late.serial_project = true;
        assert!(!codes(&late).contains(&"registration_deadline_exceeded"));

        let mut bbl = complete();
        bbl.purpose = Some(RegistrationPurpose::BblCheck);
        assert!(codes(&bbl).contains(&"bbl_check_dates_differ"));
        bbl.registration_date = bbl.survey_date.clone();
        assert!(!codes(&bbl).contains(&"bbl_check_dates_differ"));

        let mut relabel = complete();
        relabel.relabel = true;
        // The registration date may lie after the 24 months; the
        // improvement date decides (§4.2.3).
        relabel.registration_date = Some("2028-03-01".into());
        let result = assess_registration(&relabel);
        assert_eq!(result.relabel_deadline.as_deref(), Some("2028-01-31"));
        assert!(result.registration_deadline.is_none());
        let found: Vec<_> = result.issues.iter().map(|item| item.code).collect();
        assert!(!found.contains(&"registration_deadline_exceeded"));
        assert!(found.contains(&"original_kernel_version_required"));
        assert!(found.contains(&"improvement_date_required"));
        relabel.original_kernel_version = Some("0.0.0-old".into());
        assert!(codes(&relabel).contains(&"relabel_kernel_version_differs"));
        relabel.original_kernel_version = Some(KERNEL_VERSION.into());
        relabel.improvement_date = Some("2028-01-31".into());
        assert!(codes(&relabel).is_empty(), "{:?}", codes(&relabel));
        relabel.improvement_date = Some("2028-02-01".into());
        assert_eq!(codes(&relabel), vec!["relabel_deadline_exceeded"]);
        relabel.improvement_date = Some("2025-12-01".into());
        assert_eq!(codes(&relabel), vec!["improvement_before_survey"]);
        // Only for existing buildings.
        relabel.improvement_date = Some("2027-06-01".into());
        relabel.purpose = Some(RegistrationPurpose::Delivery);
        assert!(codes(&relabel).contains(&"relabel_only_for_existing_buildings"));
    }

    #[test]
    fn empty_registration_lists_missing_data_only() {
        let result = assess_registration(&Registration::default());
        assert!(!result.ready_for_registration);
        assert!(result.issues.iter().all(|item| item.severity == "missing"));
        assert!(result.valid_until.is_none());
        let mut similar = complete();
        similar.representation = Some(Representation::Similar);
        assert_eq!(codes(&similar), vec!["reference_object_required"]);
    }

    #[test]
    fn software_bag_and_message_types() {
        // Regeling art. 5 lid 1 onder b: the attested program is part of
        // the data. The attest is a property of the program, reported apart
        // from the dossier issues, so it does not block readiness.
        let mut unattested = complete();
        unattested.software.as_mut().unwrap().attest_number = None;
        let result = assess_registration(&unattested);
        assert!(result.ready_for_registration, "{:?}", result.issues);
        assert!(!result.software_attested);
        assert!(assess_registration(&complete()).software_attested);
        // Saved before the identity was recorded: this program, kernel version.
        unattested.software = None;
        let result = assess_registration(&unattested);
        assert!(result.issues.is_empty(), "{:?}", result.issues);
        assert_eq!(result.software.name, SOFTWARE_NAME);
        assert_eq!(result.software.version, KERNEL_VERSION);

        // Praktijkhandboek p. 46: a residential label on an addressable
        // object; Regeling art. 5 lid 1 onder a (p. 6) also allows the pand
        // id, which a utility building may use.
        let dwelling = RegistrationContext {
            residential: true,
            ..RegistrationContext::default()
        };
        let utility = RegistrationContext::default();
        let bag_codes = |registration: &Registration, context: &RegistrationContext| {
            assess_registration_with(registration, context)
                .issues
                .iter()
                .map(|item| item.code)
                .collect::<Vec<_>>()
        };
        let mut bag = complete();
        bag.bag_object_id = Some("0363100000000001".into());
        assert_eq!(
            bag_codes(&bag, &dwelling),
            vec!["bag_object_id_not_addressable"]
        );
        assert!(bag_codes(&bag, &utility).is_empty());
        bag.bag_object_id = Some("0363200000000001".into());
        assert_eq!(
            bag_codes(&bag, &utility),
            vec!["bag_object_id_not_addressable"]
        );
        bag.bag_object_id = Some("0363020000000001".into());
        assert!(bag_codes(&bag, &dwelling).is_empty());
        bag.bag_object_id = Some("36301000000001".into());
        assert_eq!(codes(&bag), vec!["bag_object_id_invalid"]);

        // §4.2.5 opmerking 4/5: relabel and replacement message types.
        let mut relabel = complete();
        relabel.message_type = Some(MessageType::Relabel);
        let result = assess_registration(&relabel);
        assert_eq!(result.message_type, MessageType::Relabel);
        assert!(result.relabel_deadline.is_some());
        let mut conflict = complete();
        conflict.relabel = true;
        conflict.message_type = Some(MessageType::Regular);
        assert!(codes(&conflict).contains(&"message_type_conflict"));

        let mut replacement = complete();
        replacement.message_type = Some(MessageType::Replacement);
        replacement.registration_date = Some("2027-06-01".into());
        let result = assess_registration(&replacement);
        assert_eq!(result.replacement_deadline.as_deref(), Some("2028-01-31"));
        assert!(result.registration_deadline.is_none());
        assert_eq!(
            codes(&replacement),
            vec!["replaced_ep_online_number_required"]
        );
        replacement.replaced_ep_online_number = Some("EP-123".into());
        assert!(codes(&replacement).is_empty());
        replacement.registration_date = Some("2028-02-01".into());
        assert_eq!(codes(&replacement), vec!["replacement_deadline_exceeded"]);
    }

    #[test]
    fn wlc_gwp_from_2028_for_new_buildings_over_1000_m2() {
        let mut bbl = complete();
        bbl.purpose = Some(RegistrationPurpose::BblCheck);
        bbl.survey_type = Some(SurveyType::Detailed);
        bbl.survey_date = Some("2028-01-10".into());
        bbl.registration_date = Some("2028-01-10".into());
        let large = RegistrationContext {
            usable_floor_area_m2: Some(1200.0),
            ..RegistrationContext::default()
        };
        let result = assess_registration_with(&bbl, &large);
        assert_eq!(result.wlc_gwp_required, Some(true));
        let found: Vec<_> = result.issues.iter().map(|item| item.code).collect();
        assert_eq!(
            found,
            vec!["wlc_gwp_required", "wlc_gwp_reference_required"]
        );
        bbl.wlc_gwp = Some(WlcGwp {
            value_kg_co2_eq_per_m2_year: Some(7.5),
            report_reference: Some("evidence:wlc".into()),
        });
        assert!(assess_registration_with(&bbl, &large).issues.is_empty());
        // Not required before 2028, at 1000 m² or for an existing building.
        let small = RegistrationContext {
            usable_floor_area_m2: Some(1000.0),
            ..RegistrationContext::default()
        };
        assert_eq!(
            assess_registration_with(&bbl, &small).wlc_gwp_required,
            Some(false)
        );
        bbl.wlc_gwp = None;
        bbl.survey_date = Some("2027-12-31".into());
        bbl.registration_date = Some("2027-12-31".into());
        assert_eq!(
            assess_registration_with(&bbl, &large).wlc_gwp_required,
            Some(false)
        );
        assert_eq!(
            assess_registration_with(&complete(), &large).wlc_gwp_required,
            Some(false)
        );
        // A delivery follows its toets Bbl (p. 21, 62): a 2028 delivery after
        // a 2027 check needs no WLC-GWP; without the check date it warns.
        let mut delivery = bbl.clone();
        delivery.purpose = Some(RegistrationPurpose::Delivery);
        delivery.survey_date = Some("2028-02-01".into());
        delivery.registration_date = Some("2028-02-01".into());
        delivery.bbl_check_date = Some("2027-11-01".into());
        let result = assess_registration_with(&delivery, &large);
        assert_eq!(result.wlc_gwp_required, Some(false));
        assert!(result.issues.is_empty(), "{:?}", result.issues);
        delivery.bbl_check_date = None;
        let result = assess_registration_with(&delivery, &large);
        assert_eq!(result.wlc_gwp_required, None);
        assert!(result.issues.is_empty(), "{:?}", result.issues);
        assert!(result
            .plausibility
            .iter()
            .any(|item| item.code == "wlc_gwp_bbl_check_date_unknown"));
        delivery.bbl_check_date = Some("2028-01-05".into());
        assert_eq!(
            assess_registration_with(&delivery, &large).wlc_gwp_required,
            Some(true)
        );
        // The threshold is per building: one dwelling of 90 m² does not
        // decide it, the building's A_g does.
        let dwelling = RegistrationContext {
            usable_floor_area_m2: Some(90.0),
            residential: true,
            ..RegistrationContext::default()
        };
        let mut apartment = bbl.clone();
        apartment.survey_date = Some("2028-01-10".into());
        apartment.registration_date = Some("2028-01-10".into());
        let result = assess_registration_with(&apartment, &dwelling);
        assert_eq!(result.wlc_gwp_required, None);
        assert!(result
            .plausibility
            .iter()
            .any(|item| item.code == "wlc_gwp_building_area_unknown"));
        apartment.building_usable_floor_area_m2 = Some(4800.0);
        assert_eq!(
            assess_registration_with(&apartment, &dwelling).wlc_gwp_required,
            Some(true)
        );
        // Undecidable without A_g.
        bbl.survey_date = Some("2028-03-01".into());
        bbl.registration_date = Some("2028-03-01".into());
        assert_eq!(assess_registration(&bbl).wlc_gwp_required, None);
        bbl.wlc_gwp = Some(WlcGwp {
            value_kg_co2_eq_per_m2_year: Some(-1.0),
            report_reference: None,
        });
        assert!(codes(&bbl).contains(&"wlc_gwp_invalid"));
    }

    #[test]
    fn plausibility_warnings_never_block() {
        let mut registration = complete();
        registration.previous_label_class = Some("E".into());
        let context = RegistrationContext {
            usable_floor_area_m2: Some(12.0),
            zone_floor_areas_m2: vec![12.005],
            loss_area_ratio: Some(6.0),
            residential: true,
            label_function: Some(LabelFunction::Residential),
            primary_fossil_kwh_per_m2: Some(104.5),
            label_class: Some("A+"),
            envelope: vec![EnvelopeSummary {
                category: ElementCategory::Facade,
                area_m2: 50.0,
                mean_u_w_per_m2k: Some(0.05),
                min_rc_m2k_per_w: None,
                max_rc_m2k_per_w: None,
            }],
        };
        let result = assess_registration_with(&registration, &context);
        assert!(result.ready_for_registration, "{:?}", result.issues);
        let found: Vec<_> = result.plausibility.iter().map(|item| item.code).collect();
        for code in [
            "plausibility_high_class_basic_survey",
            "plausibility_borderline_label",
            "plausibility_label_class_jump",
            "plausibility_loss_area_ratio",
            "plausibility_usable_floor_area",
            "usable_floor_area_precision",
            "plausibility_u_value",
        ] {
            assert!(found.contains(&code), "{code}: {found:?}");
        }
        assert!(result
            .plausibility
            .iter()
            .all(|item| item.severity == "warning"));
        let calm = RegistrationContext {
            usable_floor_area_m2: Some(96.25),
            zone_floor_areas_m2: vec![96.25],
            loss_area_ratio: Some(1.8),
            residential: true,
            label_function: Some(LabelFunction::Residential),
            primary_fossil_kwh_per_m2: Some(150.0),
            label_class: Some("A"),
            envelope: Vec::new(),
        };
        registration.previous_label_class = Some("B".into());
        assert!(assess_registration_with(&registration, &calm)
            .plausibility
            .is_empty());
        // Case does not matter; an unknown class only warns.
        registration.previous_label_class = Some("e".into());
        let result = assess_registration_with(&registration, &context);
        assert!(result
            .plausibility
            .iter()
            .any(|item| item.code == "plausibility_label_class_jump"));
        registration.previous_label_class = Some("Z".into());
        let result = assess_registration(&registration);
        assert!(result.ready_for_registration, "{:?}", result.issues);
        assert!(result
            .plausibility
            .iter()
            .any(|item| item.code == "previous_label_class_invalid"));
    }

    fn evidence(id: &str) -> EvidenceItem {
        EvidenceItem {
            id: id.into(),
            kind: EvidenceKind::Invoice,
            file_name: "factuur.pdf".into(),
            sha256: "a".repeat(64),
            date: Some("2026-01-20".into()),
            gps: None,
            source_party: Some("opdrachtgever".into()),
            checked_by: Some("A".into()),
            description: None,
            linked_paths: vec!["/zones/0".into()],
            stored_path: None,
        }
    }

    #[test]
    fn basic_survey_rejected_where_section_3_1_requires_detail() {
        assert!(codes(&complete()).is_empty());
        let mut delivery = complete();
        delivery.purpose = Some(RegistrationPurpose::Delivery);
        assert!(codes(&delivery).contains(&"detail_survey_required_for_purpose"));
        delivery.survey_type = Some(SurveyType::Detailed);
        assert!(!codes(&delivery).contains(&"detail_survey_required_for_purpose"));

        let mut new = complete();
        new.completion_date = Some("2021-06-01".into());
        assert!(codes(&new).contains(&"detail_survey_required_completed_after_2021"));
        new.completion_date = None;
        new.construction_year = Some(2022);
        assert!(codes(&new).contains(&"detail_survey_required_completed_after_2021"));

        let mut fee = complete();
        fee.detail_survey_triggers.energy_performance_fee = true;
        assert_eq!(codes(&fee), vec!["detail_survey_required"]);
    }

    #[test]
    fn evidence_register_is_validated_and_cross_checked() {
        let mut registration = complete();
        registration.evidence = vec![evidence("ev-1"), evidence("ev-1")];
        registration.evidence[1].sha256 = "xyz".into();
        registration.evidence[1].checked_by = None;
        registration.evidence[1].gps = Some(GpsPosition {
            latitude: 95.0,
            longitude: 5.0,
        });
        let found = codes(&registration);
        for code in [
            "evidence_id_invalid",
            "evidence_hash_invalid",
            "evidence_check_required",
            "evidence_gps_invalid",
        ] {
            assert!(found.contains(&code), "{code}: {found:?}");
        }

        let mut registration = complete();
        registration.evidence = vec![evidence("ev-1")];
        let project = serde_json::json!({
            "zones": [{"sourceReference": "survey; evidence:ev-1"}],
            "constructions": [{"evidenceReference": "evidence:ev-9."}],
            "registration": {"note": "evidence:ignored"}
        });
        let result =
            assess_project_registration(&registration, &project, &RegistrationContext::default());
        assert!(result.ready_for_registration == result.issues.is_empty());
        let unknown: Vec<_> = result
            .issues
            .iter()
            .filter(|item| item.code == "evidence_reference_unknown")
            .map(|item| item.path.as_str())
            .collect();
        assert_eq!(unknown, vec!["/constructions/0/evidenceReference"]);
        registration.evidence[0].linked_paths = vec!["/zones/3".into()];
        let result =
            assess_project_registration(&registration, &project, &RegistrationContext::default());
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "evidence_link_path_unknown"));
        assert!(!result.ready_for_registration);
    }
}
