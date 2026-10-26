//! Snapshot of a register declaration (issue #15): what the adviser took
//! over from a quality declaration in a product register such as the BCRG
//! energieprestatiedatabank, kept with the evidence item of that
//! declaration.
//!
//! There is no machine contract with the register yet (BCRG issues API keys
//! on request; endpoints, rights and limits are not known), so the only
//! route is `manual`: the adviser copies the values and confirms them. The
//! snapshot keeps the declaration number, revision, product, the scope the
//! declaration states, the date of copying, the values with the project
//! input each one fills, and a SHA-256 over those values. It is the
//! historical record of what the calculation used and never changes by
//! itself. Whether the declaration is still current is a separate check
//! (`registryCheck`) the adviser records; until then its status is "not
//! checked", never "valid".
//!
//! A snapshot proves which values were copied from which declaration; it is
//! no proof that the declaration is authentic or valid, and the result stays
//! `calculated_unverified` like every other result of this program.

use crate::registration::{
    resolve_evidence_pointer, Date, EvidenceItem, EvidenceKind, Registration, RegistrationIssue,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// How the values were obtained. Only `manual` exists: no register API
/// contract has been obtained (docs/nta8800-bcrg-register.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotRoute {
    Manual,
}

/// The adviser's record of the declaration's current status in the register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryState {
    NotChecked,
    /// Checked: the register shows the same revision.
    Current,
    /// Checked: the register shows another revision or other values.
    Changed,
    /// Checked: the declaration is withdrawn.
    Withdrawn,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegistryCheck {
    pub state: RegistryState,
    /// YYYY-MM-DD of the check.
    #[serde(default)]
    pub checked_on: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// One value copied from the declaration.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SnapshotValue {
    /// What the declaration calls it, e.g. "COP A7/W35".
    pub label: String,
    /// The value as on the declaration, e.g. "4,21".
    pub value: String,
    #[serde(default)]
    pub unit: Option<String>,
    /// JSON pointer of the project input this value fills, when it fills one.
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegisterSnapshot {
    /// The register, e.g. "BCRG".
    pub register: String,
    pub route: SnapshotRoute,
    pub declaration_number: String,
    #[serde(default)]
    pub revision: Option<String>,
    pub product_name: String,
    #[serde(default)]
    pub manufacturer: Option<String>,
    /// NTA 8800 edition and application conditions as the declaration states them.
    #[serde(default)]
    pub scope: Option<String>,
    /// YYYY-MM-DD on which the values were copied.
    pub captured_on: String,
    pub values: Vec<SnapshotValue>,
    /// SHA-256 over the values ([`snapshot_values_sha256`]).
    pub values_sha256: String,
    /// The adviser who confirmed the values and their scope, and when.
    #[serde(default)]
    pub confirmed_by: Option<String>,
    #[serde(default)]
    pub confirmed_on: Option<String>,
    #[serde(default)]
    pub registry_check: Option<RegistryCheck>,
}

/// SHA-256 (lowercase hex) over the values: per value one line
/// `label TAB value TAB unit TAB path LF`, absent fields empty. The app
/// computes the same (src/core/nta/registerSnapshot.ts).
pub fn snapshot_values_sha256(values: &[SnapshotValue]) -> String {
    let mut hasher = Sha256::new();
    for item in values {
        hasher.update(item.label.as_bytes());
        hasher.update(b"\t");
        hasher.update(item.value.as_bytes());
        hasher.update(b"\t");
        hasher.update(item.unit.as_deref().unwrap_or("").as_bytes());
        hasher.update(b"\t");
        hasher.update(item.path.as_deref().unwrap_or("").as_bytes());
        hasher.update(b"\n");
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn issue(code: &'static str, path: String, severity: &'static str) -> RegistrationIssue {
    RegistrationIssue {
        code,
        path,
        severity,
    }
}

fn blank(text: &Option<String>) -> bool {
    text.as_deref()
        .map_or(true, |value| value.trim().is_empty())
}

/// A declaration value as a number: "4,21", "4.21" or "1 234,5".
fn number(text: &str) -> Option<f64> {
    let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    let normal = if compact.contains(',') {
        compact.replace('.', "").replace(',', ".")
    } else {
        compact
    };
    normal.parse::<f64>().ok().filter(|value| value.is_finite())
}

/// Whether the project input equals the copied value.
fn same_value(project: &Value, copied: &str) -> bool {
    match project {
        Value::Number(number_value) => match (number_value.as_f64(), number(copied)) {
            (Some(a), Some(b)) => (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0),
            _ => false,
        },
        Value::String(text) => text.trim() == copied.trim(),
        Value::Bool(flag) => copied.trim() == flag.to_string(),
        _ => false,
    }
}

/// Where a product carries its register identity: the heat-pump input
/// (`ntaHeatPump.performanceEvidence.registryRecord`), at or above a pointer.
const PRODUCT_RECORDS: [&str; 2] = [
    "/performanceEvidence/registryRecord/registrationNumber",
    "/ntaHeatPump/performanceEvidence/registryRecord/registrationNumber",
];

/// The registration number of a product record at or above `pointer`, if any.
fn product_record_number<'a>(project: &'a Value, pointer: &str) -> Option<&'a str> {
    let mut current = pointer.to_owned();
    loop {
        let node = project.pointer(&current);
        if let Some(number) = PRODUCT_RECORDS
            .iter()
            .filter_map(|record| node.and_then(|node| node.pointer(record)))
            .filter_map(Value::as_str)
            .find(|number| !number.trim().is_empty())
        {
            return Some(number);
        }
        let cut = current.rfind('/')?;
        current.truncate(cut);
    }
}

fn check_one(
    project: &Value,
    item: &EvidenceItem,
    snapshot: &RegisterSnapshot,
    path: &str,
    issues: &mut Vec<RegistrationIssue>,
    plausibility: &mut Vec<RegistrationIssue>,
) {
    let at = |field: &str| format!("{path}.registerSnapshot.{field}");
    if !matches!(
        item.kind,
        EvidenceKind::QualityDeclaration | EvidenceKind::DeclarationOfPerformance
    ) {
        issues.push(issue(
            "register_snapshot_kind",
            format!("{path}.kind"),
            "error",
        ));
    }
    if snapshot.register.trim().is_empty()
        || snapshot.declaration_number.trim().is_empty()
        || snapshot.product_name.trim().is_empty()
    {
        issues.push(issue(
            "register_snapshot_identity_missing",
            at("declarationNumber"),
            "missing",
        ));
    }
    if Date::parse(&snapshot.captured_on).is_none() {
        issues.push(issue(
            "register_snapshot_date_invalid",
            at("capturedOn"),
            "error",
        ));
    }
    if snapshot.values.is_empty() {
        issues.push(issue(
            "register_snapshot_values_missing",
            at("values"),
            "missing",
        ));
    }
    for (index, value) in snapshot.values.iter().enumerate() {
        if value.label.trim().is_empty() || value.value.trim().is_empty() {
            issues.push(issue(
                "register_snapshot_value_missing",
                at(&format!("values[{index}].value")),
                "missing",
            ));
            continue;
        }
        let Some(pointer) = value
            .path
            .as_deref()
            .filter(|pointer| !pointer.trim().is_empty())
        else {
            continue;
        };
        let resolved = resolve_evidence_pointer(project, pointer);
        match resolved
            .as_deref()
            .and_then(|resolved| project.pointer(resolved))
        {
            None => issues.push(issue(
                "register_snapshot_path_unknown",
                at(&format!("values[{index}].path")),
                "error",
            )),
            // The calculation uses another value than the declaration: the
            // snapshot never follows the project, nor the project the snapshot.
            Some(current) if !same_value(current, &value.value) => issues.push(issue(
                "register_snapshot_value_differs",
                at(&format!("values[{index}]")),
                "error",
            )),
            Some(_) => {}
        }
    }
    if snapshot_values_sha256(&snapshot.values) != snapshot.values_sha256 {
        issues.push(issue(
            "register_snapshot_hash_mismatch",
            at("valuesSha256"),
            "error",
        ));
    }
    // The product the values fill must be the product of the declaration.
    let pointers = item.linked_paths.iter().chain(
        snapshot
            .values
            .iter()
            .filter_map(|value| value.path.as_ref()),
    );
    for pointer in pointers {
        let Some(resolved) = resolve_evidence_pointer(project, pointer) else {
            continue;
        };
        if let Some(number) = product_record_number(project, &resolved) {
            if number.trim() != snapshot.declaration_number.trim() {
                issues.push(issue(
                    "register_snapshot_product_mismatch",
                    at("declarationNumber"),
                    "error",
                ));
                break;
            }
        }
    }
    if blank(&snapshot.confirmed_by)
        || snapshot
            .confirmed_on
            .as_deref()
            .and_then(Date::parse)
            .is_none()
    {
        issues.push(issue(
            "register_snapshot_unconfirmed",
            at("confirmedBy"),
            "missing",
        ));
    }
    match snapshot.registry_check.as_ref().map(|check| check.state) {
        Some(RegistryState::Withdrawn) => issues.push(issue(
            "register_declaration_withdrawn",
            at("registryCheck"),
            "error",
        )),
        Some(RegistryState::Changed) => plausibility.push(issue(
            "register_declaration_changed",
            at("registryCheck"),
            "warning",
        )),
        Some(RegistryState::Current) => {
            if snapshot
                .registry_check
                .as_ref()
                .and_then(|check| check.checked_on.as_deref())
                .and_then(Date::parse)
                .is_none()
            {
                issues.push(issue(
                    "register_check_date_invalid",
                    at("registryCheck.checkedOn"),
                    "error",
                ));
            }
        }
        Some(RegistryState::NotChecked) | None => plausibility.push(issue(
            "register_status_not_checked",
            at("registryCheck"),
            "warning",
        )),
    }
}

/// Checks every register snapshot in the evidence register against the
/// project: blocking findings go to `issues`, the status findings that do
/// not block registration (not checked, changed revision) to `plausibility`.
pub fn check_register_snapshots(
    project: &Value,
    registration: &Registration,
    issues: &mut Vec<RegistrationIssue>,
    plausibility: &mut Vec<RegistrationIssue>,
) {
    for (index, item) in registration.evidence.iter().enumerate() {
        if let Some(snapshot) = &item.register_snapshot {
            check_one(
                project,
                item,
                snapshot,
                &format!("registration.evidence[{index}]"),
                issues,
                plausibility,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn value(label: &str, value: &str, path: Option<&str>) -> SnapshotValue {
        SnapshotValue {
            label: label.into(),
            value: value.into(),
            unit: None,
            path: path.map(Into::into),
        }
    }

    fn project() -> Value {
        json!({
            "heatingSystems": [{
                "id": "hp",
                "cop": 4.21,
                "ntaHeatPump": { "performanceEvidence": { "kind": "controlled_quality_declaration", "reference": "evidence:ev-1",
                    "registryRecord": { "registrationNumber": "20230256GK", "productName": "X", "manufacturer": "Y", "sourceUrl": "" } } }
            }]
        })
    }

    fn snapshot() -> RegisterSnapshot {
        let values = vec![value("COP A7/W35", "4,21", Some("/heatingSystems/0/cop"))];
        RegisterSnapshot {
            register: "BCRG".into(),
            route: SnapshotRoute::Manual,
            declaration_number: "20230256GK".into(),
            revision: Some("2".into()),
            product_name: "Warmtepomp X".into(),
            manufacturer: Some("Y".into()),
            scope: Some("NTA 8800:2025+C1:2026, woningbouw".into()),
            captured_on: "2026-10-26".into(),
            values_sha256: snapshot_values_sha256(&values),
            values,
            confirmed_by: Some("A. Adviseur".into()),
            confirmed_on: Some("2026-10-26".into()),
            registry_check: Some(RegistryCheck {
                state: RegistryState::Current,
                checked_on: Some("2026-10-26".into()),
                note: None,
            }),
        }
    }

    fn registration(snapshot: RegisterSnapshot) -> Registration {
        let mut item: EvidenceItem = serde_json::from_value(json!({
            "id": "ev-1", "kind": "quality_declaration", "fileName": "verklaring.pdf",
            "sha256": "a".repeat(64), "checkedBy": "A. Adviseur", "linkedPaths": ["/heatingSystems/0"]
        }))
        .unwrap();
        item.register_snapshot = Some(snapshot);
        Registration {
            evidence: vec![item],
            ..Registration::default()
        }
    }

    fn findings(snapshot: RegisterSnapshot) -> (Vec<&'static str>, Vec<&'static str>) {
        let mut issues = Vec::new();
        let mut plausibility = Vec::new();
        check_register_snapshots(
            &project(),
            &registration(snapshot),
            &mut issues,
            &mut plausibility,
        );
        (
            issues.iter().map(|item| item.code).collect(),
            plausibility.iter().map(|item| item.code).collect(),
        )
    }

    #[test]
    fn a_confirmed_current_snapshot_that_matches_the_project_is_clean() {
        assert_eq!(findings(snapshot()), (vec![], vec![]));
    }

    #[test]
    fn the_hash_is_over_the_values_in_a_fixed_form() {
        // label TAB value TAB unit TAB path LF; the app computes the same.
        let values = vec![value("COP", "4,21", None)];
        assert_eq!(
            snapshot_values_sha256(&values),
            format!("{:x}", Sha256::digest(b"COP\t4,21\t\t\n"))
        );
    }

    #[test]
    fn a_withdrawn_declaration_blocks_and_a_changed_one_warns() {
        let mut withdrawn = snapshot();
        withdrawn.registry_check.as_mut().unwrap().state = RegistryState::Withdrawn;
        assert_eq!(
            findings(withdrawn).0,
            vec!["register_declaration_withdrawn"]
        );
        let mut changed = snapshot();
        changed.registry_check.as_mut().unwrap().state = RegistryState::Changed;
        assert_eq!(
            findings(changed),
            (vec![], vec!["register_declaration_changed"])
        );
        // Unchecked is "not checked", never "valid".
        let mut unchecked = snapshot();
        unchecked.registry_check = None;
        assert_eq!(
            findings(unchecked),
            (vec![], vec!["register_status_not_checked"])
        );
    }

    #[test]
    fn another_product_is_refused() {
        let mut other = snapshot();
        other.declaration_number = "20210163GK".into();
        assert_eq!(
            findings(other).0,
            vec!["register_snapshot_product_mismatch"]
        );
    }

    #[test]
    fn a_missing_value_and_a_changed_project_value_are_found() {
        let mut empty = snapshot();
        empty.values = vec![value("COP A7/W35", " ", None)];
        empty.values_sha256 = snapshot_values_sha256(&empty.values);
        assert_eq!(findings(empty).0, vec!["register_snapshot_value_missing"]);
        let mut differs = snapshot();
        differs.values = vec![value("COP A7/W35", "4,5", Some("/heatingSystems/0/cop"))];
        differs.values_sha256 = snapshot_values_sha256(&differs.values);
        assert_eq!(findings(differs).0, vec!["register_snapshot_value_differs"]);
    }

    #[test]
    fn edited_values_without_a_new_hash_and_an_unconfirmed_snapshot_block() {
        let mut edited = snapshot();
        edited.values[0].label = "COP A2/W35".into();
        assert_eq!(findings(edited).0, vec!["register_snapshot_hash_mismatch"]);
        let mut unconfirmed = snapshot();
        unconfirmed.confirmed_by = None;
        assert_eq!(
            findings(unconfirmed).0,
            vec!["register_snapshot_unconfirmed"]
        );
    }

    #[test]
    fn a_snapshot_reads_back_unchanged_offline() {
        // Reopening a saved project reads the snapshot as it was stored; no
        // register is consulted and the status stays what the adviser recorded.
        let stored = serde_json::to_value(snapshot()).unwrap();
        let reopened: RegisterSnapshot = serde_json::from_value(stored.clone()).unwrap();
        assert_eq!(serde_json::to_value(&reopened).unwrap(), stored);
        assert_eq!(findings(reopened), (vec![], vec![]));
    }
}
