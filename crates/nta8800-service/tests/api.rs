//! Integration tests of the HTTP API: every operation route, the OpenAPI
//! document, the error envelope, CORS and the example projects.

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    Router,
};
use nta8800_service::{
    app, app_with, openapi_document,
    operations::{operations, Method},
    HttpConfig,
};
use serde_json::{json, Value};
use tower::ServiceExt;

fn fixture(name: &str) -> Value {
    let path = format!("{}/../../training-data/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {path}"));
    serde_json::from_str(&text).unwrap()
}

/// The desktop client leaves null object members out before calling the kernel.
fn without_nulls(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(_, item)| !item.is_null())
                .map(|(key, item)| (key, without_nulls(item)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(without_nulls).collect()),
        other => other,
    }
}

async fn send(
    router: Router,
    request: Request<Body>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into()))
    };
    (status, headers, body)
}

async fn post(uri: &str, body: Value) -> (StatusCode, Value) {
    let (status, _, body) = send(
        app(),
        Request::post(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await;
    (status, body)
}

async fn get(uri: &str) -> (StatusCode, Value) {
    let (status, _, body) = send(app(), Request::get(uri).body(Body::empty()).unwrap()).await;
    (status, body)
}

fn assert_envelope(body: &Value, code: &str) {
    assert_eq!(body["code"], code, "{body}");
    assert_eq!(body["error"], code, "{body}");
    assert!(body["message"].is_string(), "{body}");
    assert!(
        body.get("path").is_some() && body.get("details").is_some(),
        "{body}"
    );
}

#[tokio::test]
async fn every_operation_has_a_route() {
    for op in operations() {
        match op.method {
            Method::Get => {
                let (status, body) = get(op.path).await;
                assert_eq!(status, StatusCode::OK, "{} {body}", op.path);
            }
            Method::Post => {
                let (status, body) = post(op.path, json!({})).await;
                assert_eq!(status, StatusCode::BAD_REQUEST, "{} {body}", op.path);
                assert_envelope(&body, "missing_request_member");
                assert_eq!(body["path"], op.input.unwrap().key, "{}", op.path);
            }
        }
    }
}

#[tokio::test]
async fn openapi_lists_exactly_the_routes() {
    let (status, document) = get("/v1/openapi.json").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(document, openapi_document());
    assert_eq!(document["openapi"], "3.1.0");
    let paths = document["paths"].as_object().unwrap();
    // Every operation is documented, under its own method and operationId.
    for op in operations() {
        let method = if op.method == Method::Get {
            "get"
        } else {
            "post"
        };
        let entry = &paths[op.path][method];
        assert_eq!(entry["operationId"], op.name, "{}", op.path);
        assert_eq!(entry["x-mcp-tool"], op.name);
    }
    // Every documented path answers (no 404) and nothing else is documented.
    assert_eq!(paths.len(), operations().len() + 2);
    for (path, item) in paths {
        let method = item.as_object().unwrap().keys().next().unwrap().clone();
        let request = if method == "get" {
            Request::get(path.as_str()).body(Body::empty()).unwrap()
        } else {
            Request::post(path.as_str())
                .header("content-type", "application/json")
                .body(Body::from("{}"))
                .unwrap()
        };
        let (status, _, _) = send(app(), request).await;
        assert_ne!(status, StatusCode::NOT_FOUND, "{path}");
    }
    // Operation names are unique.
    let mut names: Vec<_> = operations().iter().map(|op| op.name).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), operations().len());
}

#[tokio::test]
async fn health_and_version_identify_the_build() {
    let (status, health) = get("/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(health["status"], "ok");
    let (status, version) = get("/v1/version").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(version["apiVersion"], "v1");
    assert_eq!(version["kernelVersion"], nta8800_core::KERNEL_VERSION);
    assert_eq!(version["targetNormVersion"], "NTA 8800:2025+C1:2026");
    assert_eq!(version["buildFingerprint"].as_str().unwrap().len(), 64);
    let editions = version["supportedNormVersions"].as_array().unwrap();
    assert_eq!(editions.len(), 5);
    let current = editions
        .iter()
        .find(|item| item["default"] == true)
        .unwrap();
    assert_eq!(current["id"], "2025+C1");
    assert_eq!(current["registrationEligible"], true);
    let v2024 = editions.iter().find(|item| item["id"] == "2024").unwrap();
    assert_eq!(v2024["implemented"], true);
    assert_eq!(v2024["registrationEligible"], false);
}

#[tokio::test]
async fn older_edition_calculates_but_is_not_registrable() {
    let mut project = without_nulls(fixture("nta8800-example-terraced-dwelling.json"));
    project["ntaCalculation"]["normVersion"] = json!("2024");
    let (status, result) = post(
        "/v1/nta8800/project/performance",
        json!({ "project": project }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", result["gaps"]);
    assert_eq!(result["status"], "calculated_legacy_edition");
    assert_eq!(result["normVersion"], "2024");
    assert_eq!(result["registrationEligible"], false);
    assert_eq!(result["targetNormVersion"], "NTA 8800:2024 met INT-V1:2024");
}

#[tokio::test]
async fn example_projects_calculate() {
    for name in [
        "nta8800-example-terraced-dwelling.json",
        "nta8800-example-office.json",
    ] {
        let project = without_nulls(fixture(name));
        let (status, result) = post(
            "/v1/nta8800/project/performance",
            json!({ "project": project.clone() }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{name}: {}", result["gaps"]);
        assert_eq!(result["status"], "calculated_unverified");
        let beng2 = result["performance"]["primaryFossilIndicatorKwhPerM2Year"]
            .as_f64()
            .unwrap();
        assert!(beng2 > 0.0 && beng2 < 200.0, "{name}: {beng2}");

        let (status, energy) = post(
            "/v1/nta8800/project/energy-by-service",
            json!({ "project": project.clone() }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(energy["energyByService"].is_object(), "{energy}");
        assert_eq!(energy["inputFingerprint"], result["inputFingerprint"]);

        let (status, label) = post(
            "/v1/nta8800/label/data",
            json!({ "project": project.clone() }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(label["labelData"], result["labelData"]);

        let (status, registration) = post(
            "/v1/nta8800/registration/assess",
            json!({ "project": project.clone() }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(registration["registration"], result["registration"]);

        let (status, hash) = post(
            "/v1/nta8800/relabel/label-input-hash",
            json!({ "project": project.clone() }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(hash["labelInputSha256"].as_str().unwrap().len(), 64);

        let (status, relabel) = post(
            "/v1/nta8800/relabel/assess",
            json!({ "original": project.clone(), "current": project }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{relabel}");
    }
}

#[tokio::test]
async fn surveys_calculate() {
    for (route, name) in [
        (
            "/v1/nta8800/opname/residential",
            "nta8800-opname-1930-terraced.json",
        ),
        (
            "/v1/nta8800/opname/residential",
            "nta8800-opname-1975-apartment.json",
        ),
        (
            "/v1/nta8800/opname/utility",
            "nta8800-opname-utility-1985-office.json",
        ),
    ] {
        let (status, result) = post(route, json!({ "survey": fixture(name) })).await;
        assert_eq!(status, StatusCode::OK, "{name}: {}", result["issues"]);
        assert_eq!(result["status"], "calculated_unverified");
    }
}

#[tokio::test]
async fn a_project_without_nta_input_is_incomplete_with_gaps() {
    let mut project = without_nulls(fixture("nta8800-example-terraced-dwelling.json"));
    project.as_object_mut().unwrap().remove("ntaCalculation");
    let (status, result) = post(
        "/v1/nta8800/project/performance",
        json!({ "project": project }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(result["status"], "incomplete");
    assert_eq!(result["gaps"][0]["code"], "nta_calculation_block_missing");
    assert!(result.get("performance").is_none_or(Value::is_null));
}

#[tokio::test]
async fn malformed_requests_use_the_error_envelope() {
    // Invalid JSON.
    let (status, _, body) = send(
        app(),
        Request::post("/v1/nta8800/validate")
            .header("content-type", "application/json")
            .body(Body::from("{not json"))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_envelope(&body, "invalid_json");

    // Wrong content type.
    let (status, _, body) = send(
        app(),
        Request::post("/v1/nta8800/validate")
            .header("content-type", "text/plain")
            .body(Body::from("{}"))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_envelope(&body, "unsupported_media_type");

    // Not an object.
    let (status, body) = post("/v1/nta8800/validate", json!([1, 2])).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_envelope(&body, "invalid_request_shape");

    // Wrong input shape, with the JSON path of the mismatch.
    let (status, body) = post(
        "/v1/nta8800/opname/residential",
        json!({ "survey": { "constructionYear": "nineteen thirty" } }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_envelope(&body, "invalid_request_shape");
    assert!(
        body["path"].as_str().unwrap().starts_with("survey"),
        "{body}"
    );

    // Unknown route and wrong method.
    let (status, body) = get("/v1/nta8800/nothing-here").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_envelope(&body, "not_found");
    let (status, body) = get("/v1/nta8800/project/performance").await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_envelope(&body, "method_not_allowed");

    // Body larger than the limit.
    let small = app_with(HttpConfig {
        body_limit_bytes: 64,
        ..HttpConfig::default()
    });
    let big = json!({ "project": { "padding": "x".repeat(500) } }).to_string();
    let (status, _, body) = send(
        small,
        Request::post("/v1/nta8800/validate")
            .header("content-type", "application/json")
            .body(Body::from(big))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_envelope(&body, "payload_too_large");
}

#[tokio::test]
async fn cors_is_off_by_default_and_configurable() {
    let request = || {
        Request::get("/health")
            .header(header::ORIGIN, "http://localhost:5173")
            .body(Body::empty())
            .unwrap()
    };
    let (_, headers, _) = send(app(), request()).await;
    assert!(headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());

    let config = HttpConfig {
        cors_origins: vec!["http://localhost:5173".into()],
        ..HttpConfig::default()
    };
    let (_, headers, _) = send(app_with(config.clone()), request()).await;
    assert_eq!(
        headers[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "http://localhost:5173"
    );

    let preflight = Request::builder()
        .method("OPTIONS")
        .uri("/v1/nta8800/project/performance")
        .header(header::ORIGIN, "http://localhost:5173")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .body(Body::empty())
        .unwrap();
    let (status, headers, _) = send(app_with(config.clone()), preflight).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(headers[header::ACCESS_CONTROL_ALLOW_METHODS]
        .to_str()
        .unwrap()
        .contains("POST"));

    let other = Request::get("/health")
        .header(header::ORIGIN, "http://evil.example")
        .body(Body::empty())
        .unwrap();
    let (_, headers, _) = send(app_with(config), other).await;
    assert!(headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
}
