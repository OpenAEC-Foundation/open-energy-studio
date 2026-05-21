//! `schema-export` — dump JSON Schemas for project and result types to disk.
//!
//! Invoked from the npm `generate-types` script as a build-time step that
//! produces the inputs to `json-schema-to-typescript`. Run via:
//!
//! ```sh
//! cargo run --bin schema-export -- --out ../schemas
//! ```
//!
//! The output directory defaults to `schemas/` relative to the workspace root
//! and contains:
//!
//! - `project.schema.json`  (from `isso51_core::project_schema`)
//! - `result.schema.json`   (from `isso51_core::result_schema`)
//!
//! Phase B will extend this binary to also export `ProjectV2` /
//! `OesProjectResult` schemas once the BENG orchestrator lands.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let out_dir = parse_out(&args).unwrap_or_else(|| PathBuf::from("schemas"));

    fs::create_dir_all(&out_dir)?;

    let project_path = out_dir.join("project.schema.json");
    fs::write(&project_path, isso51_core::project_schema())?;
    eprintln!("wrote {}", project_path.display());

    let result_path = out_dir.join("result.schema.json");
    fs::write(&result_path, isso51_core::result_schema())?;
    eprintln!("wrote {}", result_path.display());

    Ok(())
}

fn parse_out(args: &[String]) -> Option<PathBuf> {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "--out" {
            return iter.next().map(PathBuf::from);
        }
        if let Some(rest) = arg.strip_prefix("--out=") {
            return Some(PathBuf::from(rest));
        }
    }
    None
}
