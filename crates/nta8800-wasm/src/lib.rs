//! WebAssembly adapter for the NTA 8800 kernel.
//!
//! The browser build of Open Energy Studio loads this module and calls
//! [`run`] with the same route and request body it would send to the HTTP
//! API. The dispatch goes through `nta8800-operations`, the registry that the
//! HTTP API and the MCP server use, so the three transports expose the same
//! operations and status rules. Nothing here touches the network or the file
//! system; the calculation runs inside the page.

use nta8800_operations::{error_body, operation, operations, version_value, Method, Outcome};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

fn outcome_json(outcome: Outcome) -> String {
    json!({ "status": outcome.status, "body": outcome.body }).to_string()
}

/// Finds an operation by its HTTP route (`/v1/...`, optionally prefixed with
/// `/api` as the browser client writes it) or by its MCP tool name.
fn find(route: &str) -> Option<&'static nta8800_operations::Operation> {
    let path = route.strip_prefix("/api").unwrap_or(route);
    let path = path.split('?').next().unwrap_or(path);
    operations()
        .iter()
        .find(|op| op.path == path)
        .or_else(|| operation(route))
}

/// Runs one kernel operation. `route` is the HTTP route or the MCP name,
/// `body` the JSON request body (empty for GET operations). The result is a
/// JSON object `{"status": <http status>, "body": <response body>}`, so the
/// client can treat it exactly like an HTTP response.
#[wasm_bindgen]
pub fn run(route: &str, body: &str) -> String {
    let Some(op) = find(route) else {
        return outcome_json(Outcome {
            status: 404,
            body: error_body(
                "unknown_operation",
                format!("No kernel operation at `{route}`"),
                None,
                Value::Null,
            ),
        });
    };
    let body: Value = if body.trim().is_empty() {
        json!({})
    } else {
        match serde_json::from_str(body) {
            Ok(value) => value,
            Err(error) => {
                return outcome_json(Outcome {
                    status: 400,
                    body: error_body("invalid_json", error.to_string(), None, Value::Null),
                })
            }
        }
    };
    outcome_json((op.run)(&body))
}

/// Kernel, norm and build identity, the same object as `GET /v1/version`.
#[wasm_bindgen]
pub fn version() -> String {
    version_value().to_string()
}

/// Every operation with its method, route and MCP name, as a JSON array.
#[wasm_bindgen]
pub fn list_operations() -> String {
    let list: Vec<Value> = operations()
        .iter()
        .map(|op| {
            json!({
                "name": op.name,
                "method": match op.method { Method::Get => "GET", Method::Post => "POST" },
                "path": op.path,
                "category": op.category,
            })
        })
        .collect();
    serde_json::to_string(&list).unwrap_or_else(|_| "[]".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: String) -> Value {
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn version_route_answers_with_the_kernel_identity() {
        let outcome = parse(run("/api/v1/version", ""));
        assert_eq!(outcome["status"], 200);
        assert_eq!(
            outcome["body"]["kernelVersion"],
            nta8800_core::KERNEL_VERSION
        );
        assert_eq!(
            parse(version())["targetNormVersion"],
            nta8800_core::TARGET_NORM_VERSION
        );
    }

    #[test]
    fn routes_resolve_with_and_without_the_api_prefix_and_by_name() {
        let survey = include_str!("../../../training-data/nta8800-opname-1930-terraced.json");
        let body = format!("{{\"survey\": {survey}}}");
        for route in [
            "/api/v1/nta8800/opname/residential",
            "/v1/nta8800/opname/residential",
            "assess_residential_survey",
        ] {
            let outcome = parse(run(route, &body));
            assert_eq!(outcome["status"], 200, "{route}");
            assert_eq!(outcome["body"]["status"], "calculated_unverified");
        }
    }

    #[test]
    fn errors_follow_the_http_status_rules() {
        assert_eq!(parse(run("/v1/nope", ""))["status"], 404);
        assert_eq!(
            parse(run("/v1/nta8800/opname/residential", "{"))["status"],
            400
        );
        let missing = parse(run("/v1/nta8800/opname/residential", "{}"));
        assert_eq!(missing["status"], 400);
        assert_eq!(missing["body"]["code"], "missing_request_member");
        let list = parse(list_operations());
        assert!(list.as_array().unwrap().len() > 40);
    }
}
