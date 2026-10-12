//! Edition-switch guidance (`compare_project_editions`) on the public cases
//! whose input differs between their own edition and 2025+C1: A, B, C
//! (crawlspace wall height h, point 59), D (groundwater heat pump), J and L,
//! and on both examples between every pair of editions.

use nta8800_core::edition_switch::{compare_project_editions, project_edition, SwitchCategory};
use nta8800_core::norm_versions::NormVersion;
use nta8800_core::project_performance::assess_project_performance;
use serde_json::{json, Value};
use std::collections::BTreeSet;

/// The cases with the edition of their report.
const CASES: [(&str, &str, &str); 6] = [
    (
        "A",
        "2020+A1",
        include_str!("../../../training-data/nta8800-public-comparison-a.json"),
    ),
    (
        "B",
        "2022",
        include_str!("../../../training-data/nta8800-public-comparison-b.json"),
    ),
    (
        "C",
        "2023",
        include_str!("../../../training-data/nta8800-public-comparison-c.json"),
    ),
    (
        "D",
        "2020+A1",
        include_str!("../../../training-data/nta8800-public-comparison-d.json"),
    ),
    (
        "J",
        "2020+A1",
        include_str!("../../../training-data/nta8800-public-comparison-j.json"),
    ),
    (
        "L",
        "2022",
        include_str!("../../../training-data/nta8800-public-comparison-l.json"),
    ),
];

type Key = (String, String);

fn case(index: usize) -> Value {
    let (_, edition, json) = CASES[index];
    let mut project: Value = serde_json::from_str(json).unwrap();
    project["ntaCalculation"]["normVersion"] = json!(edition);
    project
}

fn gap_keys(project: &Value) -> BTreeSet<Key> {
    assess_project_performance(project)
        .gaps
        .iter()
        .map(|gap| (gap.code.to_string(), gap.path.clone()))
        .collect()
}

fn in_edition(project: &Value, to: NormVersion) -> Value {
    let mut moved = project.clone();
    moved["ntaCalculation"]["normVersion"] = json!(to.id());
    moved
}

/// Checks one switch and returns a one-line summary.
fn check(name: &str, project: &Value, to: NormVersion) -> String {
    let from = project_edition(project);
    let report = compare_project_editions(project, to);
    let baseline = gap_keys(project);
    let label = format!("{name} {} → {}", from.id(), to.id());
    // Every edition refusal right after the switch is listed.
    let first: BTreeSet<Key> = gap_keys(&in_edition(project, to))
        .difference(&baseline)
        .cloned()
        .collect();
    let listed: BTreeSet<Key> = report
        .items
        .iter()
        .filter(|item| {
            matches!(
                item.category,
                SwitchCategory::Refused | SwitchCategory::Required
            )
        })
        .map(|item| (item.code.clone(), item.path.clone()))
        .collect();
    for key in first
        .iter()
        .filter(|(code, _)| code == "route_not_in_edition")
    {
        assert!(listed.contains(key), "{label}: {key:?} not listed");
    }
    // Every listed refusal or requirement is traced to a switch point.
    for item in &report.items {
        if matches!(
            item.category,
            SwitchCategory::Refused | SwitchCategory::Required
        ) {
            assert!(
                !item.points.is_empty(),
                "{label}: {} at {} has no switch point",
                item.code,
                item.path
            );
        }
    }
    // After the automatic changes, what is left is exactly what the list
    // leaves to the adviser.
    let open: BTreeSet<Key> = report
        .items
        .iter()
        .filter(|item| {
            matches!(
                item.category,
                SwitchCategory::Refused | SwitchCategory::Required
            )
        })
        .filter(|item| !item.resolved_by_automatic_changes)
        .map(|item| (item.code.clone(), item.path.clone()))
        .collect();
    let remaining: BTreeSet<Key> = report
        .remaining_after_automatic_changes
        .iter()
        .map(|gap| (gap.code.to_string(), gap.path.clone()))
        .collect();
    assert_eq!(open, remaining, "{label}");
    let recalculated: BTreeSet<Key> = gap_keys(&report.adjusted_project)
        .difference(&baseline)
        .cloned()
        .collect();
    assert_eq!(recalculated, remaining, "{label}: adjusted project");
    // The open items are never fixed automatically.
    for item in report
        .items
        .iter()
        .filter(|item| !item.resolved_by_automatic_changes)
    {
        if matches!(
            item.category,
            SwitchCategory::Refused | SwitchCategory::Required
        ) {
            assert!(
                !item.fix.automatic(),
                "{label}: {} left open with an automatic fix",
                item.path
            );
        }
    }
    format!(
        "{label}: {} items ({} automatic), {} open, status {}",
        report.items.len(),
        report
            .items
            .iter()
            .filter(|item| item.fix.automatic())
            .count(),
        remaining.len(),
        report.status_after_automatic_changes
    )
}

#[test]
fn switching_the_public_cases_lists_what_the_kernel_refuses_and_requires() {
    for (index, (name, _, _)) in CASES.iter().enumerate() {
        let project = case(index);
        let own = project_edition(&project);
        println!("{}", check(name, &project, NormVersion::V2025C1));
        // And back: from the adjusted project to the case's own edition.
        let report = compare_project_editions(&project, NormVersion::V2025C1);
        println!("{}", check(name, &report.adjusted_project, own));
    }
}

#[test]
fn a_crawlspace_wall_height_is_dropped_on_the_way_up_and_required_on_the_way_down() {
    // Case A in 2020+A1 with the actual crawlspace wall height (8.47).
    let mut a = case(0);
    a["ntaCalculation"]["groundFloors"][0]["below"]["wallHeightAboveGroundM"] = json!(0.3);
    let up = compare_project_editions(&a, NormVersion::V2025C1);
    let height = up
        .items
        .iter()
        .find(|item| item.path.ends_with("wallHeightAboveGroundM"))
        .expect("wall height listed");
    assert_eq!(height.category, SwitchCategory::Refused);
    assert_eq!(height.points, vec!["59"]);
    assert!(height.fix.automatic() && height.resolved_by_automatic_changes);
    let down = compare_project_editions(&up.adjusted_project, NormVersion::V2022);
    let height = down
        .items
        .iter()
        .find(|item| item.code == "ground_floor_wall_height_required")
        .expect("wall height required again");
    assert_eq!(height.category, SwitchCategory::Required);
    assert_eq!(height.points, vec!["59"]);
    assert!(!height.fix.automatic() && !height.resolved_by_automatic_changes);
}

#[test]
fn a_panel_route_without_its_member_is_left_to_the_adviser() {
    // 2020+A1 has no PV panel route (point 68). Dropping the panel power
    // would leave `peakPower.method: panels` without its member, so the
    // choice stays with the adviser.
    let office: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-example-office.json"
    ))
    .unwrap();
    let report = compare_project_editions(&office, NormVersion::V2020A1);
    let panel = report
        .items
        .iter()
        .find(|item| item.path.ends_with("panelPeakPowerW"))
        .expect("panel power listed");
    assert_eq!(panel.points, vec!["68"]);
    assert!(!panel.fix.automatic());
    assert!(report
        .adjusted_project
        .pointer("/ntaCalculation/pvSystems/0/peakPower/panelPeakPowerW")
        .is_some());
}

#[test]
fn the_table_9_28_test_standard_is_named_but_never_rewritten() {
    // Case D's heat pump on the table 9.28 row, valid in 2025+C1.
    let mut d = in_edition(&case(3), NormVersion::V2025C1);
    let pointer = "/ntaCalculation/generator/forfait/highEfficiencyEvidence/testStandardEdition";
    let slot = d
        .pointer_mut(pointer)
        .expect("case D has a table 9.28 test report");
    *slot = json!("NEN-EN 14511-2:2022");
    assert_test_standard(&d, pointer);
}

fn assert_test_standard(project: &Value, pointer: &str) {
    let report = compare_project_editions(project, NormVersion::V2022);
    let item = report
        .items
        .iter()
        .find(|item| item.code == "high_test_standard_invalid")
        .expect("test standard listed");
    assert_eq!(item.points, vec!["67a"]);
    // The standard the target asks for is named ...
    assert_eq!(item.value, Some(json!("NEN-EN 14511-2:2007")));
    // ... but the test report's own standard is evidence: the measured COPs
    // were tested to it, so the switch never rewrites it.
    assert!(!item.fix.automatic());
    assert!(!item.resolved_by_automatic_changes);
    assert_eq!(
        report.adjusted_project.pointer(pointer),
        project.pointer(pointer)
    );
    assert!(report
        .remaining_after_automatic_changes
        .iter()
        .any(|gap| gap.code == "high_test_standard_invalid"));
}

#[test]
fn every_edition_pair_of_the_examples_settles() {
    for json in [
        include_str!("../../../training-data/nta8800-example-office.json"),
        include_str!("../../../training-data/nta8800-example-terraced-dwelling.json"),
    ] {
        let project: Value = serde_json::from_str(json).unwrap();
        for from in NormVersion::ALL {
            let start = in_edition(&project, from);
            for to in NormVersion::ALL {
                println!("{}", check("example", &start, to));
            }
        }
    }
}
