//! Runs one kernel operation in-process, as the HTTP API would, without a server.
//!
//! `kernel_once <METHOD> <PATH>` reads the request body (JSON, may be empty)
//! from stdin and prints `{"status": …, "body": …}` on stdout. The manual
//! screenshot harness (`scripts/manual-screenshots/`) answers the app's
//! `/api/…` calls with it, so no API server is started and no port is opened.

use std::io::Read;

use nta8800_service::operations::{execute, operations, Method};
use serde_json::{json, Value};

fn main() {
    let mut args = std::env::args().skip(1);
    let method = args.next().unwrap_or_default().to_ascii_uppercase();
    let path = args.next().unwrap_or_default();
    let path = path.split('?').next().unwrap_or_default();
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);

    let (status, body) = if path == "/health" {
        (
            200,
            json!({ "status": "ok", "kernel": "rust", "kernelVersion": nta8800_core::KERNEL_VERSION }),
        )
    } else {
        let wanted = if method == "GET" {
            Method::Get
        } else {
            Method::Post
        };
        match operations()
            .iter()
            .find(|op| op.path == path && op.method == wanted)
        {
            None => (
                404,
                json!({ "error": "not_found", "code": "not_found", "message": path }),
            ),
            Some(op) => {
                let body = if raw.trim().is_empty() {
                    json!({})
                } else {
                    match serde_json::from_str::<Value>(&raw) {
                        Ok(value) => value,
                        Err(error) => {
                            print_outcome(
                                400,
                                json!({ "error": "invalid_json", "code": "invalid_json", "message": error.to_string() }),
                            );
                            return;
                        }
                    }
                };
                let outcome = execute(op, body);
                (outcome.status, outcome.body)
            }
        }
    };
    print_outcome(status, body);
}

fn print_outcome(status: u16, body: Value) {
    println!("{}", json!({ "status": status, "body": body }));
}
