//! A link to an element made in the app is stored by id
//! (`/zones/@z1/surfaces/@wall-N`, `stablePointer` in
//! src/core/nta/EvidenceLinks.ts). The fixture is the project-route request
//! of such a project after save and reload, written by
//! src/__tests__/evidence-link-registration.test.tsx. The registration
//! check must resolve the link, not report it as unknown.

use nta8800_core::project_performance::assess_project_performance;
use serde_json::Value;

#[test]
fn an_app_made_id_link_passes_the_registration_check() {
    let project: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-evidence-link-project.json"
    ))
    .unwrap();
    let links = &project["registration"]["evidence"][0]["linkedPaths"];
    assert!(
        links[0].as_str().unwrap().contains("/@"),
        "the fixture must carry an id link: {links}"
    );
    let result = assess_project_performance(&project);
    let registration = result.registration.expect("registration assessed");
    let link_issues: Vec<_> = registration
        .issues
        .iter()
        .filter(|item| item.code.starts_with("evidence_link"))
        .map(|item| format!("{} @ {}", item.code, item.path))
        .collect();
    assert!(link_issues.is_empty(), "{link_issues:?}");
}

#[test]
fn a_dangling_id_link_is_still_reported() {
    let mut project: Value = serde_json::from_str(include_str!(
        "../../../training-data/nta8800-evidence-link-project.json"
    ))
    .unwrap();
    project["registration"]["evidence"][0]["linkedPaths"] =
        serde_json::json!(["/zones/@z1/surfaces/@removed-wall"]);
    let registration = assess_project_performance(&project)
        .registration
        .expect("registration assessed");
    assert!(registration
        .issues
        .iter()
        .any(|item| item.code == "evidence_link_path_unknown"));
    assert!(!registration.ready_for_registration);
}
