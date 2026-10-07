//! The edition of one request: the optional `normVersion` of an API, MCP or
//! desktop call, written into the input's own edition slot, and the edition
//! stamped on the result. The service and the desktop app share these rules.

use super::NormVersion;
use serde_json::{json, Value};

/// Name of the request member and of the input slots.
pub const NORM_VERSION_MEMBER: &str = "normVersion";

/// A request whose edition cannot be used; `code` is a kernel code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditionError {
    pub code: &'static str,
    pub message: String,
    pub path: String,
}

/// The requested edition, if any: absent or `null` is none, anything that
/// is not an edition identifier is `invalid_norm_version`.
pub fn parse_requested(value: Option<&Value>) -> Result<Option<NormVersion>, EditionError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(|_| EditionError {
                code: "invalid_norm_version",
                message: format!(
                    "`normVersion` must be one of the edition identifiers, not {value}"
                ),
                path: NORM_VERSION_MEMBER.into(),
            }),
    }
}

/// Writes `version` at `pointer` of `input` (called `member` in messages).
/// An edition already there must be the same (`norm_version_conflict`); the
/// default edition is never written, since that would change the input's
/// fingerprint and label-input hash.
pub fn place_in(
    input: &mut Value,
    member: &str,
    pointer: &str,
    version: NormVersion,
) -> Result<(), EditionError> {
    let path = format!("{member}{}", pointer.replace('/', "."));
    match input.pointer(pointer).filter(|value| !value.is_null()) {
        Some(found) => {
            let found: Option<NormVersion> = serde_json::from_value(found.clone()).ok();
            if found == Some(version) {
                Ok(())
            } else {
                Err(EditionError {
                    code: "norm_version_conflict",
                    message: format!(
                        "`normVersion` {} differs from the edition in `{path}`",
                        version.id()
                    ),
                    path,
                })
            }
        }
        None if version.is_default() => Ok(()),
        None => {
            let (parent, key) = pointer.rsplit_once('/').expect("slot pointer has a key");
            match input.pointer_mut(parent).and_then(Value::as_object_mut) {
                Some(object) => {
                    object.insert(key.to_string(), json!(version));
                    Ok(())
                }
                None => Err(EditionError {
                    code: "norm_version_not_applicable",
                    message: format!(
                        "`normVersion` {} cannot be placed: `{path}` has no parent object",
                        version.id()
                    ),
                    path,
                }),
            }
        }
    }
}

/// The edition in the slot at `pointer` of `input`; absent is the default.
pub fn edition_at(input: &Value, pointer: &str) -> NormVersion {
    input
        .pointer(pointer)
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}

/// Records the edition a result was calculated with: `normVersion` and
/// `targetNormVersion` are added when the kernel did not set them, and a
/// calculation in an older edition gets the legacy status (never
/// registrable). Error envelopes and non-objects stay as they are.
pub fn stamp(result: &mut Value, version: NormVersion) {
    let Some(object) = result.as_object_mut() else {
        return;
    };
    if object.contains_key("error") {
        return;
    }
    let version = object
        .get(NORM_VERSION_MEMBER)
        .and_then(|value| serde_json::from_value::<NormVersion>(value.clone()).ok())
        .unwrap_or(version);
    object
        .entry(NORM_VERSION_MEMBER)
        .or_insert_with(|| json!(version));
    object
        .entry("targetNormVersion")
        .or_insert_with(|| json!(version.label()));
    if !version.registration_eligible() {
        if object.get("status").and_then(Value::as_str) == Some("calculated_unverified") {
            object.insert("status".into(), json!("calculated_legacy_edition"));
        }
        if object.contains_key("registrationEligible") {
            object.insert("registrationEligible".into(), json!(false));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requested_edition_is_parsed_or_refused() {
        assert_eq!(parse_requested(None), Ok(None));
        assert_eq!(parse_requested(Some(&Value::Null)), Ok(None));
        assert_eq!(
            parse_requested(Some(&json!("2022"))),
            Ok(Some(NormVersion::V2022))
        );
        let error = parse_requested(Some(&json!("2019"))).unwrap_err();
        assert_eq!(error.code, "invalid_norm_version");
        assert_eq!(error.path, "normVersion");
    }

    #[test]
    fn edition_is_placed_once_and_never_contradicted() {
        let mut project = json!({ "ntaCalculation": {} });
        let slot = "/ntaCalculation/normVersion";
        place_in(&mut project, "project", slot, NormVersion::V2025C1).unwrap();
        assert!(project.pointer(slot).is_none());
        place_in(&mut project, "project", slot, NormVersion::V2023).unwrap();
        assert_eq!(edition_at(&project, slot), NormVersion::V2023);
        let error = place_in(&mut project, "project", slot, NormVersion::V2024).unwrap_err();
        assert_eq!(error.code, "norm_version_conflict");
        assert_eq!(error.path, "project.ntaCalculation.normVersion");
        let error = place_in(&mut json!({}), "project", slot, NormVersion::V2023).unwrap_err();
        assert_eq!(error.code, "norm_version_not_applicable");
        assert_eq!(edition_at(&json!({}), slot), NormVersion::V2025C1);
    }

    #[test]
    fn older_edition_result_is_stamped_legacy() {
        let mut result = json!({ "status": "calculated_unverified", "registrationEligible": true });
        stamp(&mut result, NormVersion::V2020A1);
        assert_eq!(result["status"], "calculated_legacy_edition");
        assert_eq!(result["registrationEligible"], false);
        assert_eq!(result["normVersion"], "2020+A1");
        assert_eq!(result["targetNormVersion"], "NTA 8800:2020+A1:2020");
        let mut current = json!({ "status": "calculated_unverified" });
        stamp(&mut current, NormVersion::V2025C1);
        assert_eq!(current["status"], "calculated_unverified");
        let mut error = json!({ "error": { "code": "x" } });
        stamp(&mut error, NormVersion::V2022);
        assert!(error.get("normVersion").is_none());
    }
}
