//! Checks shared by the integration tests that run the kernel on many
//! inputs (`option_coverage.rs`, `numeric_fuzz.rs`): JSON-pointer edits,
//! the codes of a result, the finite-number guard, the routes of
//! `src/core/nta/gapRoutes.ts` and the NL/EN labels of `src/i18n`.
//!
//! Each test crate includes this module; not every crate uses every item.
#![allow(dead_code)]

use nta8800_core::finite::first_non_finite;
use serde_json::Value;
use std::collections::{BTreeSet, HashSet};
use std::path::Path;

pub fn unescape(segment: &str) -> String {
    segment.replace("~1", "/").replace("~0", "~")
}

pub fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// Sets (or inserts) the value at a JSON pointer whose parent exists.
pub fn set_pointer(root: &mut Value, pointer: &str, new: Value) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    let key = unescape(key);
    let target = if parent.is_empty() {
        root
    } else {
        root.pointer_mut(parent).unwrap()
    };
    match target {
        Value::Object(map) => {
            map.insert(key, new);
        }
        Value::Array(items) => items[key.parse::<usize>().unwrap()] = new,
        _ => panic!("{pointer}"),
    }
}

/// Removes the object member at a JSON pointer.
pub fn remove_pointer(root: &mut Value, pointer: &str) {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    if let Some(Value::Object(map)) = root.pointer_mut(parent) {
        map.remove(&unescape(key));
    }
}

/// The codes under every `gaps`, `warnings` and `issues` list.
pub fn codes(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                if matches!(key.as_str(), "gaps" | "warnings" | "issues") {
                    for entry in item.as_array().into_iter().flatten() {
                        if let Some(code) = entry.get("code").and_then(Value::as_str) {
                            out.insert(code.to_string());
                        }
                    }
                }
                codes(item, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                codes(item, out);
            }
        }
        _ => {}
    }
}

/// The path and detail reported with `code`, for the failure message.
pub fn entries(value: &Value, code: &str, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if map.get("code").and_then(Value::as_str) == Some(code) {
                out.push(format!(
                    "{} {}",
                    map.get("path").and_then(Value::as_str).unwrap_or(""),
                    map.get("detail").and_then(Value::as_str).unwrap_or("")
                ));
            }
            for item in map.values() {
                entries(item, code, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                entries(item, code, out);
            }
        }
        _ => {}
    }
}

pub fn checked<T: serde::Serialize>(result: &T) -> Result<Value, String> {
    match first_non_finite(result) {
        Some(path) => Err(format!("non-finite number at {path}")),
        None => Ok(serde_json::to_value(result).unwrap()),
    }
}

/// The prefixes of `src/core/nta/gapRoutes.ts`: the first member of every
/// `GAP_ROUTES` prefix and every member of `NTA_INPUT_ROUTES` (below
/// `ntaCalculation`). A gap path routes when its first member is one of the
/// former, or when it sits in the NTA input and its member is one of the
/// latter; the UI then opens the step page with that section.
pub struct GapRoutes {
    project: HashSet<String>,
    nta: HashSet<String>,
}

impl GapRoutes {
    pub fn routes(&self, path: &str) -> bool {
        let first =
            |path: &str| -> String { path.split(['.', '[']).next().unwrap_or("").to_string() };
        match path.strip_prefix("ntaCalculation") {
            Some("") => true,
            Some(rest) if rest.starts_with('.') => self.nta.contains(&first(&rest[1..])),
            _ => self.project.contains(&first(path)),
        }
    }
}

pub fn gap_routes() -> &'static GapRoutes {
    static ROUTES: std::sync::OnceLock<GapRoutes> = std::sync::OnceLock::new();
    ROUTES.get_or_init(|| {
        let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/core/nta/gapRoutes.ts");
        let text = std::fs::read_to_string(file).unwrap();
        let mut project = HashSet::new();
        let mut nta = HashSet::new();
        let mut in_nta = false;
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("export const NTA_INPUT_ROUTES") {
                in_nta = true;
                continue;
            }
            if in_nta {
                if line.starts_with("};") {
                    in_nta = false;
                    continue;
                }
                let key = line
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .trim_matches(|c: char| c == '\'' || c.is_whitespace());
                let member = key.split(['.', '[']).next().unwrap_or("");
                if !member.is_empty() {
                    nta.insert(member.to_string());
                }
            } else if let Some(rest) = line.split("prefix: '").nth(1) {
                let prefix = rest.split('\'').next().unwrap_or("");
                let member = prefix.split(['.', '[']).next().unwrap_or("");
                project.insert(member.to_string());
            }
        }
        assert!(nta.contains("generator") && project.contains("zones"));
        GapRoutes { project, nta }
    })
}

/// The label keys of `src/i18n`, Dutch and English apart: `nl.ts` and
/// `en.ts` whole, the other files per `export const …Nl` / `…En` table.
pub fn label_keys() -> (HashSet<String>, HashSet<String>) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/i18n");
    let (mut nl, mut en) = (HashSet::new(), HashSet::new());
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".ts") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let whole = match name.as_str() {
            "nl.ts" => Some(true),
            "en.ts" => Some(false),
            _ => None,
        };
        let mut table = whole;
        for line in text.lines() {
            if whole.is_none() && line.starts_with("export const ") {
                let ident = line["export const ".len()..]
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("");
                table = if ident.ends_with("Nl") {
                    Some(true)
                } else if ident.ends_with("En") {
                    Some(false)
                } else {
                    None
                };
            }
            let Some(dutch) = table else { continue };
            let Some(rest) = line.trim_start().strip_prefix('\'') else {
                continue;
            };
            let Some((key, after)) = rest.split_once('\'') else {
                continue;
            };
            if after.trim_start().starts_with(':') {
                if dutch { &mut nl } else { &mut en }.insert(key.to_string());
            }
        }
    }
    (nl, en)
}

pub const LABEL_PREFIXES: [&str; 7] = [
    "nta.gap.",
    "nta.warning.",
    "kernel.issue.",
    "opname.issue.",
    "opname.warning.",
    "mwa.issue.",
    "registration.issue.",
];

pub fn labelled(keys: &HashSet<String>, code: &str) -> bool {
    LABEL_PREFIXES
        .iter()
        .any(|prefix| keys.contains(&format!("{prefix}{code}")))
}

/// The rules every result must keep, given its JSON and its status:
/// - never refused with `non_finite_result` or `kernel_panic`;
/// - a non-calculated status only with at least one code;
/// - for the project route, a non-calculated status only with at least one
///   project gap, and every gap path routes in `gapRoutes.ts`.
///
/// Gives whether the result calculated and the codes it carries.
pub fn check_outcome(
    project: bool,
    value: &Value,
    status: &str,
) -> Result<(bool, BTreeSet<String>), String> {
    let mut found = BTreeSet::new();
    codes(value, &mut found);
    for generic in ["non_finite_result", "kernel_panic"] {
        if found.contains(generic) {
            let mut detail = Vec::new();
            entries(value, generic, &mut detail);
            return Err(format!("refused with {generic}: {}", detail.join("; ")));
        }
    }
    let calculated = status.starts_with("calculated");
    if !calculated && found.is_empty() {
        return Err(format!("status {status} without any code"));
    }
    if project && !calculated {
        let gaps = value["gaps"].as_array().cloned().unwrap_or_default();
        if gaps.is_empty() {
            return Err(format!("status {status} without any project gap"));
        }
        let routes = gap_routes();
        for gap in &gaps {
            let path = gap["path"].as_str().unwrap_or("");
            if !routes.routes(path) {
                return Err(format!(
                    "gap {} at {path} has no route in gapRoutes.ts",
                    gap["code"]
                ));
            }
        }
    }
    Ok((calculated, found))
}

/// The panic message of a caught panic.
pub fn panic_message(panic: &(dyn std::any::Any + Send)) -> String {
    panic
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default()
}
