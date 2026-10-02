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
//! The kernel checks the data; it does not register anything. EP-Online
//! registration needs the RVO exchange specification.

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
    /// Relabel message type (§4.2.3/4.2.4).
    #[serde(default)]
    pub relabel: bool,
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
    /// needed before registration.
    pub severity: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationAssessment {
    pub source: &'static str,
    /// Survey date + 10 years (Bep art. 2.1 lid 7).
    pub valid_until: Option<String>,
    /// Last allowed registration date (§4.2.5); `None` for a relabel.
    pub registration_deadline: Option<String>,
    /// Last date for a relabel (§4.2.3).
    pub relabel_deadline: Option<String>,
    pub ready_for_registration: bool,
    pub issues: Vec<RegistrationIssue>,
}

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

pub fn assess_registration(registration: &Registration) -> RegistrationAssessment {
    let mut issues = Vec::new();
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
    if blank(&registration.bag_object_id) {
        issues.push(issue("bag_object_id_required", "bagObjectId", "missing"));
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
    if let Some(survey) = survey {
        if registration.relabel {
            // §4.2.3/4.2.4: improvements within 24 months of the original
            // survey, calculated with the original kernel.
            let deadline = survey.add_months(24);
            relabel_deadline = Some(deadline);
            if registered.is_some_and(|date| date > deadline) {
                issues.push(issue(
                    "relabel_deadline_exceeded",
                    "registrationDate",
                    "error",
                ));
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
    if registration.relabel && registration.purpose == Some(RegistrationPurpose::BblCheck) {
        issues.push(issue("relabel_not_for_bbl_check", "relabel", "error"));
    }
    check_detail_survey(registration, &mut issues);
    check_evidence(&registration.evidence, &mut issues);

    RegistrationAssessment {
        source: REGISTRATION_SOURCE,
        valid_until: valid_until.map(|date| date.to_string()),
        registration_deadline: registration_deadline.map(|date| date.to_string()),
        relabel_deadline: relabel_deadline.map(|date| date.to_string()),
        ready_for_registration: issues.is_empty(),
        issues,
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
) -> RegistrationAssessment {
    let mut result = assess_registration(registration);
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
        relabel.registration_date = Some("2027-12-01".into());
        let result = assess_registration(&relabel);
        assert_eq!(result.relabel_deadline.as_deref(), Some("2028-01-31"));
        assert!(result.registration_deadline.is_none());
        let found: Vec<_> = result.issues.iter().map(|item| item.code).collect();
        assert!(!found.contains(&"registration_deadline_exceeded"));
        assert!(found.contains(&"original_kernel_version_required"));
        relabel.original_kernel_version = Some("0.0.0-old".into());
        assert!(codes(&relabel).contains(&"relabel_kernel_version_differs"));
        relabel.original_kernel_version = Some(KERNEL_VERSION.into());
        relabel.registration_date = Some("2028-02-01".into());
        assert_eq!(codes(&relabel), vec!["relabel_deadline_exceeded"]);
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
        let result = assess_project_registration(&registration, &project);
        assert!(result.ready_for_registration == result.issues.is_empty());
        let unknown: Vec<_> = result
            .issues
            .iter()
            .filter(|item| item.code == "evidence_reference_unknown")
            .map(|item| item.path.as_str())
            .collect();
        assert_eq!(unknown, vec!["/constructions/0/evidenceReference"]);
        registration.evidence[0].linked_paths = vec!["/zones/3".into()];
        let result = assess_project_registration(&registration, &project);
        assert!(result
            .issues
            .iter()
            .any(|item| item.code == "evidence_link_path_unknown"));
        assert!(!result.ready_for_registration);
    }
}
