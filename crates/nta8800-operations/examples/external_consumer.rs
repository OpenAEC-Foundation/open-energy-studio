//! How another Rust program uses the kernel as a library (issue #9): through
//! the operation registry, with the same names, inputs and statuses as the
//! HTTP API and the MCP tools. In another crate:
//!
//! ```toml
//! [dependencies]
//! nta8800-operations = { git = "https://github.com/OpenAEC-Foundation/open-energy-studio", tag = "v2026.44.0" }
//! serde_json = "1"
//! ```
//!
//! Run from the repository: `cargo run --manifest-path crates/nta8800-operations/Cargo.toml --example external_consumer`
//! (optionally with the path of a project file).

use nta8800_operations::{operation, version_value};
use serde_json::{json, Value};

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../training-data/nta8800-example-terraced-dwelling.json"
        )
        .to_string()
    });
    let project: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("project file"))
            .expect("project JSON");

    let version = version_value();
    println!(
        "kernel {} · {}",
        version["kernelVersion"], version["targetNormVersion"]
    );

    // The same operation as POST /v1/nta8800/project/performance and the MCP tool.
    let op = operation("calculate_project_performance").expect("operation");
    let outcome = (op.run)(json!({ "project": project }));
    let body = &outcome.body;
    println!(
        "HTTP-equivalent status {} · kernel status {}",
        outcome.status, body["status"]
    );
    if outcome.is_error() {
        for gap in body["gaps"].as_array().into_iter().flatten().take(5) {
            println!("  gap {} at {}", gap["code"], gap["path"]);
        }
        std::process::exit(1);
    }
    let performance = &body["performance"];
    println!(
        "BENG 1 {} · BENG 2 {} · BENG 3 {} % · indicative label {}",
        performance["needIndicatorKwhPerM2Year"],
        performance["primaryFossilIndicatorKwhPerM2Year"],
        performance["renewableSharePercent"],
        performance["indicativeLabelClass"]
    );
    // The result is unverified until the program holds a BRL 9501 attest.
    assert_eq!(body["status"], "calculated_unverified");
}
