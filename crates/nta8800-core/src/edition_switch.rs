//! What changes for a project when its NTA 8800 edition changes.
//!
//! The comparison is not a hand-kept list. The project is calculated in its
//! own edition and in the target edition; what the target refuses or newly
//! requires is what the kernel reports there and not in the source edition.
//! Safe mechanical changes are applied and the project is calculated again,
//! until nothing new turns up:
//! - a member the target refuses (`route_not_in_edition`) is dropped, or a
//!   flag that switches the route on is switched off, unless dropping it
//!   makes the member required.
//!
//! The dated NEN-EN 14511-2 of a table 9.28 test report is evidence: it
//! says what the measured COPs were tested to. It is never rewritten; the
//! item names the standard the target asks for and leaves the choice (a
//! report to that standard, or another table row) to the adviser.
//!
//! Required values are never invented; they stay listed. Every finding is
//! traced to its switch point through [`crate::norm_versions::switches`].
//! Members of a route the target does not have and does not refuse (a flag
//! left on, a 2023-only member) are listed as ignored.

use crate::norm_versions::switches::{self, SwitchPoint, SWITCH_POINTS};
use crate::norm_versions::NormVersion;
use crate::project_performance::{assess_project_performance, InputGap};
use serde::Serialize;
use serde_json::{json, Value};

/// At most this many recalculations while the automatic changes settle.
const MAX_ROUNDS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SwitchCategory {
    /// The target edition refuses the input.
    Refused,
    /// The target edition needs an input the project does not give.
    Required,
    /// The target edition does not use the input.
    Ignored,
    /// The input keeps its place but means something else in the target.
    Reinterpreted,
    /// A plausibility warning the target edition adds.
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SwitchFix {
    /// Remove the member (applied automatically).
    Drop,
    /// Switch the flag off (applied automatically).
    SwitchOff,
    /// Set the member to `value` (applied automatically). No switch point
    /// produces it at present: the one value that differs by edition, the
    /// table 9.28 test standard, is evidence and is left to the adviser.
    Set,
    /// Enter a value; the kernel does not invent one.
    Enter,
    /// Choose another value or route.
    Choose,
    /// Check the input; nothing is refused.
    Review,
}

impl SwitchFix {
    pub fn automatic(self) -> bool {
        matches!(self, Self::Drop | Self::SwitchOff | Self::Set)
    }
}

/// One input that changes meaning, validity or need with the edition.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditionSwitchItem {
    pub category: SwitchCategory,
    pub code: String,
    /// Project input path (`ntaCalculation.groundFloors[0].below.wallHeightAboveGroundM`).
    pub path: String,
    /// Switch point numbers of docs/nta8800-normversies.md.
    pub points: Vec<&'static str>,
    pub subject: Option<&'static str>,
    pub fix: SwitchFix,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    /// The automatic changes make this finding disappear (the item itself,
    /// or a member it hung under, was dropped or set).
    pub resolved_by_automatic_changes: bool,
}

/// A switch point whose profile fields differ between the two editions.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchPointSummary {
    pub points: Vec<&'static str>,
    pub subject: &'static str,
    pub fields: Vec<&'static str>,
    /// The switch point acts on input members (otherwise on values only).
    pub has_inputs: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditionSwitchReport {
    pub from_norm_version: NormVersion,
    pub to_norm_version: NormVersion,
    pub from_label: &'static str,
    pub to_label: &'static str,
    pub switch_points: Vec<SwitchPointSummary>,
    pub items: Vec<EditionSwitchItem>,
    /// The project in the target edition with the automatic changes applied.
    pub adjusted_project: Value,
    /// Status of `adjusted_project`.
    pub status_after_automatic_changes: &'static str,
    /// Gaps of `adjusted_project` the project did not have in its own
    /// edition: exactly the listed items that need the adviser.
    pub remaining_after_automatic_changes: Vec<InputGap>,
}

/// The edition a project is calculated in (`ntaCalculation.normVersion`).
pub fn project_edition(project: &Value) -> NormVersion {
    project
        .pointer("/ntaCalculation/normVersion")
        .filter(|value| !value.is_null())
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default()
}

fn key(gap: &InputGap) -> (String, String) {
    (gap.code.to_string(), gap.path.clone())
}

fn is_required(code: &str) -> bool {
    code == "nta_value_missing" || code.ends_with("_required") || code.ends_with("_missing")
}

/// JSON pointer of a gap path (`a.b[2].c` → `/a/b/2/c`).
pub fn pointer_of(path: &str) -> String {
    let mut pointer = String::new();
    for part in path.split('.') {
        let mut pieces = part.split('[');
        if let Some(member) = pieces.next() {
            pointer.push('/');
            pointer.push_str(&member.replace('~', "~0").replace('/', "~1"));
        }
        for index in pieces {
            pointer.push('/');
            pointer.push_str(index.trim_end_matches(']'));
        }
    }
    pointer
}

/// Removes the value at `pointer`; returns it.
fn remove_at(value: &mut Value, pointer: &str) -> Option<Value> {
    let (parent, last) = pointer.rsplit_once('/')?;
    match value.pointer_mut(parent)? {
        Value::Object(map) => map.remove(last),
        Value::Array(items) => {
            let index: usize = last.parse().ok()?;
            (index < items.len()).then(|| items.remove(index))
        }
        _ => None,
    }
}

fn set_edition(project: &mut Value, version: NormVersion) {
    if let Some(Value::Object(map)) = project.get_mut("ntaCalculation") {
        map.insert("normVersion".into(), json!(version.id()));
    }
}

fn point_of(
    code: &str,
    path: &str,
    from: NormVersion,
    to: NormVersion,
) -> Option<&'static SwitchPoint> {
    switches::attribute(code, path, from, to)
}

/// The automatic change for a finding of the target edition, if any.
/// Only a member the switch point names is changed automatically, never an
/// enclosing object or a list element.
fn automatic_fix(
    gap: &InputGap,
    point: Option<&SwitchPoint>,
    project: &Value,
) -> Option<(SwitchFix, Option<Value>)> {
    let point = point?;
    if gap.path.ends_with(']')
        || !point
            .inputs
            .iter()
            .any(|input| switches::path_ends_with(&gap.path, input))
    {
        return None;
    }
    let pointer = pointer_of(&gap.path);
    let current = project.pointer(&pointer)?;
    match gap.code {
        "route_not_in_edition" => match current {
            Value::Bool(true) => Some((SwitchFix::SwitchOff, Some(json!(false)))),
            _ => Some((SwitchFix::Drop, None)),
        },
        _ => None,
    }
}

/// The value the target edition asks for, shown with a finding the adviser
/// resolves (the table 9.28 test standard); never applied automatically.
fn suggestion(gap: &InputGap, to: NormVersion) -> Option<Value> {
    (gap.code == "high_test_standard_invalid")
        .then(|| json!(to.profile().heat_pump_high_test_standard))
}

fn apply(project: &mut Value, path: &str, fix: SwitchFix, value: &Option<Value>) -> Option<Value> {
    let pointer = pointer_of(path);
    match fix {
        SwitchFix::Drop => remove_at(project, &pointer),
        SwitchFix::SwitchOff | SwitchFix::Set => {
            let slot = project.pointer_mut(&pointer)?;
            let old = slot.clone();
            *slot = value.clone().unwrap_or(Value::Null);
            Some(old)
        }
        _ => None,
    }
}

fn restore(project: &mut Value, path: &str, old: Value) {
    let pointer = pointer_of(path);
    let Some((parent, last)) = pointer.rsplit_once('/') else {
        return;
    };
    match project.pointer_mut(parent) {
        Some(Value::Object(map)) => {
            map.insert(last.to_string(), old);
        }
        Some(Value::Array(items)) => {
            if let Ok(index) = last.parse::<usize>() {
                if index <= items.len() {
                    items.insert(index, old);
                } else {
                    items.push(old);
                }
            }
        }
        _ => {}
    }
}

/// Whether `outer` is an enclosing member of `inner` (`a.b` of `a.b[0].c`).
fn encloses(outer: &str, inner: &str) -> bool {
    inner.len() > outer.len()
        && inner.starts_with(outer)
        && matches!(inner.as_bytes()[outer.len()], b'.' | b'[')
}

fn new_gaps(result: &[InputGap], baseline: &[(String, String)]) -> Vec<InputGap> {
    result
        .iter()
        .filter(|gap| !baseline.contains(&key(gap)))
        .cloned()
        .collect()
}

/// Walks `value` and lists the paths of the members named `name` whose
/// value is set (not null and not `false`).
fn member_paths(value: &Value, prefix: &str, name: &str, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                if key == name && !matches!(child, Value::Null | Value::Bool(false)) {
                    out.push(path.clone());
                }
                member_paths(child, &path, name, out);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                member_paths(child, &format!("{prefix}[{index}]"), name, out);
            }
        }
        _ => {}
    }
}

/// Compares a project between its own edition and `to`.
pub fn compare_project_editions(project: &Value, to: NormVersion) -> EditionSwitchReport {
    let from = project_edition(project);
    let source = assess_project_performance(project);
    let baseline: Vec<(String, String)> = source.gaps.iter().map(key).collect();
    let baseline_warnings: Vec<(String, String)> = source.warnings.iter().map(key).collect();

    let mut candidate = project.clone();
    set_edition(&mut candidate, to);
    let mut items: Vec<EditionSwitchItem> = Vec::new();
    // Paths whose automatic drop turned out to make them required.
    let mut kept: Vec<String> = Vec::new();
    let mut applied: Vec<(String, Value)> = Vec::new();

    let mut result = assess_project_performance(&candidate);
    for _ in 0..MAX_ROUNDS {
        let mut changed = false;
        for gap in new_gaps(&result.gaps, &baseline) {
            if items
                .iter()
                .any(|item| item.code == gap.code && item.path == gap.path)
            {
                continue;
            }
            let point = point_of(gap.code, &gap.path, from, to);
            let automatic = if kept.contains(&gap.path) {
                None
            } else {
                automatic_fix(&gap, point, &candidate)
            };
            let (fix, value) = match automatic {
                Some((fix, value)) => {
                    if let Some(old) = apply(&mut candidate, &gap.path, fix, &value) {
                        applied.push((gap.path.clone(), old));
                        changed = true;
                    }
                    (fix, value)
                }
                None if is_required(gap.code) => (SwitchFix::Enter, None),
                None => (SwitchFix::Choose, suggestion(&gap, to)),
            };
            items.push(EditionSwitchItem {
                category: if is_required(gap.code) {
                    SwitchCategory::Required
                } else {
                    SwitchCategory::Refused
                },
                code: gap.code.to_string(),
                path: gap.path.clone(),
                points: point.map(|p| p.points.to_vec()).unwrap_or_default(),
                subject: point.map(|p| p.subject),
                fix,
                value,
                resolved_by_automatic_changes: false,
            });
        }
        result = assess_project_performance(&candidate);
        // A dropped member that the target then requires, or whose absence
        // makes its block unreadable, was not optional: put it back and
        // leave the choice to the adviser.
        let fresh = new_gaps(&result.gaps, &baseline);
        let mut reverted = false;
        for item in items.iter_mut().filter(|item| item.fix == SwitchFix::Drop) {
            let broken = fresh.iter().any(|gap| {
                gap.code != item.code && (gap.path == item.path || encloses(&gap.path, &item.path))
            });
            if broken {
                if let Some(index) = applied.iter().rposition(|(path, _)| path == &item.path) {
                    let (path, old) = applied.remove(index);
                    restore(&mut candidate, &path, old);
                    kept.push(path);
                    item.fix = SwitchFix::Choose;
                    reverted = true;
                }
            }
        }
        if reverted {
            result = assess_project_performance(&candidate);
            continue;
        }
        if !changed {
            break;
        }
    }

    // Members the target has no route for and does not refuse.
    for point in SWITCH_POINTS {
        let Some(has_route) = point.has_route else {
            continue;
        };
        if has_route(to.profile()) || !has_route(from.profile()) {
            continue;
        }
        for input in point.inputs {
            let name = input.rsplit('.').next().unwrap_or(input);
            let mut paths = Vec::new();
            if let Some(nta) = candidate.get("ntaCalculation") {
                member_paths(nta, "ntaCalculation", name, &mut paths);
            }
            for path in paths
                .into_iter()
                .filter(|path| switches::path_ends_with(path, input))
            {
                if items.iter().any(|item| item.path == path) {
                    continue;
                }
                items.push(EditionSwitchItem {
                    category: SwitchCategory::Ignored,
                    code: "input_ignored_in_edition".into(),
                    path,
                    points: point.points.to_vec(),
                    subject: Some(point.subject),
                    fix: SwitchFix::Review,
                    value: None,
                    resolved_by_automatic_changes: false,
                });
            }
        }
    }
    // Members that keep their place but mean something else.
    for point in switches::switch_points_between(from, to)
        .into_iter()
        .filter(|point| point.reinterpreted)
    {
        for input in point.inputs {
            let mut paths = Vec::new();
            if let Some(nta) = candidate.get("ntaCalculation") {
                member_paths(nta, "ntaCalculation", input, &mut paths);
            }
            for path in paths {
                items.push(EditionSwitchItem {
                    category: SwitchCategory::Reinterpreted,
                    code: "input_reinterpreted_in_edition".into(),
                    path,
                    points: point.points.to_vec(),
                    subject: Some(point.subject),
                    fix: SwitchFix::Review,
                    value: None,
                    resolved_by_automatic_changes: false,
                });
            }
        }
    }
    for warning in result
        .warnings
        .iter()
        .filter(|warning| !baseline_warnings.contains(&key(warning)))
    {
        let point = point_of(warning.code, &warning.path, from, to);
        items.push(EditionSwitchItem {
            category: SwitchCategory::Warning,
            code: warning.code.to_string(),
            path: warning.path.clone(),
            points: point.map(|p| p.points.to_vec()).unwrap_or_default(),
            subject: point.map(|p| p.subject),
            fix: SwitchFix::Review,
            value: None,
            resolved_by_automatic_changes: false,
        });
    }

    let remaining = new_gaps(&result.gaps, &baseline);
    // A summary finding on an enclosing object (`forfait_lookup_invalid` on
    // the generator) takes the switch point of the cause under it.
    for index in 0..items.len() {
        if !items[index].points.is_empty() {
            continue;
        }
        let cause = items
            .iter()
            .find(|other| !other.points.is_empty() && encloses(&items[index].path, &other.path))
            .map(|other| (other.points.clone(), other.subject));
        if let Some((points, subject)) = cause {
            items[index].points = points;
            items[index].subject = subject;
        }
    }
    // A finding without a switch point that the automatic changes made
    // disappear was a consequence of another one (a chapter that could not
    // be calculated while a refused member was there); it is not listed.
    items.retain(|item| {
        !item.points.is_empty()
            || item.fix.automatic()
            || !matches!(
                item.category,
                SwitchCategory::Refused | SwitchCategory::Required
            )
            || remaining
                .iter()
                .any(|gap| gap.code == item.code && gap.path == item.path)
    });
    for item in items.iter_mut().filter(|item| {
        matches!(
            item.category,
            SwitchCategory::Refused | SwitchCategory::Required
        )
    }) {
        item.resolved_by_automatic_changes = !remaining
            .iter()
            .any(|gap| gap.code == item.code && gap.path == item.path);
    }

    EditionSwitchReport {
        from_norm_version: from,
        to_norm_version: to,
        from_label: from.label(),
        to_label: to.label(),
        switch_points: switches::switch_points_between(from, to)
            .into_iter()
            .map(|point| SwitchPointSummary {
                points: point.points.to_vec(),
                subject: point.subject,
                fields: point.fields.to_vec(),
                has_inputs: !point.inputs.is_empty(),
            })
            .collect(),
        items,
        status_after_automatic_changes: result.status,
        remaining_after_automatic_changes: remaining,
        adjusted_project: candidate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_of_a_gap_path() {
        assert_eq!(
            pointer_of("ntaCalculation.groundFloors[0].below.h"),
            "/ntaCalculation/groundFloors/0/below/h"
        );
        assert_eq!(pointer_of("a.b[2][3].c"), "/a/b/2/3/c");
    }

    #[test]
    fn the_same_edition_changes_nothing() {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-example-office.json"
        ))
        .unwrap();
        let report = compare_project_editions(&project, project_edition(&project));
        assert!(report.items.is_empty(), "{:?}", report.items);
        assert!(report.switch_points.is_empty());
        assert!(report.remaining_after_automatic_changes.is_empty());
    }
}
