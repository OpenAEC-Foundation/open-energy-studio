//! Integration tests of the HTTP API: every operation route, the OpenAPI
//! document, the error envelope, CORS and the example projects.

use axum::{
    body::{to_bytes, Body},
    http::{header, Request, StatusCode},
    Router,
};
use nta8800_service::{
    app, app_with, openapi_document,
    operations::{operations, Method, ERROR_CODES},
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
    let v2023 = editions.iter().find(|item| item["id"] == "2023").unwrap();
    assert_eq!(v2023["implemented"], true);
    assert_eq!(v2023["registrationEligible"], false);
    let v2022 = editions.iter().find(|item| item["id"] == "2022").unwrap();
    assert_eq!(v2022["implemented"], true);
    assert_eq!(v2022["registrationEligible"], false);
    let v2020 = editions
        .iter()
        .find(|item| item["id"] == "2020+A1")
        .unwrap();
    assert_eq!(v2020["implemented"], true);
    assert_eq!(v2020["registrationEligible"], false);
}

#[tokio::test]
async fn edition_comparison_lists_the_changes_and_keeps_the_request() {
    let project = without_nulls(fixture("nta8800-example-office.json"));
    let (status, result) = post(
        "/v1/nta8800/project/editions/compare",
        json!({ "project": project, "normVersion": "2020+A1" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["fromNormVersion"], "2025+C1");
    assert_eq!(result["toNormVersion"], "2020+A1");
    assert_eq!(
        result["adjustedProject"]["ntaCalculation"]["normVersion"],
        "2020+A1"
    );
    let items = result["items"].as_array().unwrap();
    let panel = items
        .iter()
        .find(|item| item["path"].as_str().unwrap().ends_with("panelPeakPowerW"))
        .expect("PV panel route listed");
    assert_eq!(panel["points"], json!(["68"]));
    assert!(result["switchPoints"].as_array().unwrap().len() > 10);
    // Without a target the comparison is with the default edition.
    let mut legacy = project.clone();
    legacy["ntaCalculation"]["normVersion"] = json!("2024");
    let (status, result) = post(
        "/v1/nta8800/project/editions/compare",
        json!({ "project": legacy }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{result}");
    assert_eq!(result["fromNormVersion"], "2024");
    assert_eq!(result["toNormVersion"], "2025+C1");
    // A project without an NTA block cannot change edition.
    let (status, body) = post(
        "/v1/nta8800/project/editions/compare",
        json!({ "project": { "id": "x" } }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_envelope(&body, "invalid_project_shape");
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
    let (_, headers, _) = send(app_with(config.clone()), other).await;
    assert!(headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
    assert_eq!(headers[header::VARY], "Origin");

    // A refused preflight also varies on Origin, so a cache cannot hand it
    // to an allowed origin.
    let refused = Request::builder()
        .method("OPTIONS")
        .uri("/v1/nta8800/project/performance")
        .header(header::ORIGIN, "http://evil.example")
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
        .body(Body::empty())
        .unwrap();
    let (status, headers, _) = send(app_with(config), refused).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(headers.get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());
    assert_eq!(headers[header::VARY], "Origin");
}

/// A route the edition lacks is refused with 422 and a project gap at the
/// project input (the example office has LED lighting from 2017 and PV per
/// panel, neither of which NTA 8800:2020+A1 has).
#[tokio::test]
async fn refused_route_is_a_project_gap() {
    let mut office = without_nulls(fixture("nta8800-example-office.json"));
    office["ntaCalculation"]["normVersion"] = json!("2020+A1");
    let (status, body) = post(
        "/v1/nta8800/project/performance",
        json!({ "project": office }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    let gaps = body["gaps"].as_array().unwrap();
    assert!(
        gaps.iter().any(|gap| gap["code"] == "route_not_in_edition"
            && gap["path"] == "ntaCalculation.pvSystems[0].peakPower.panelPeakPowerW"
            && gap["detail"] == "derivedInput.pvSystems[0].peakPower.panelPeakPowerW"),
        "{body}"
    );
}

/// The option sweep of the kernel (crates/nta8800-core/tests/option_coverage.rs)
/// through the HTTP layer, for a few options of each outcome and every
/// edition: a calculated input answers 200 with a calculated status, a refused
/// one 422 with at least one code, a malformed one 400 with the error
/// envelope, and nothing answers 500 (non_finite_result, kernel_panic). The
/// first two options are the two NaN refusals the sweep found.
#[tokio::test]
async fn option_sweep_keeps_the_http_error_model() {
    let terraced = without_nulls(fixture("nta8800-example-terraced-dwelling.json"));
    let mut options: Vec<(&str, Value)> = Vec::new();
    let mut basement = terraced.clone();
    basement["ntaCalculation"]["groundFloors"][0]["heatedBasement"] = json!({
        "wallDepths": [{ "lengthM": 10.0, "depthM": 1.5 }],
        "wallResistanceM2kPerW": 2.5
    });
    options.push(("heated basement with depth per wall part", basement));
    let mut shutters = terraced.clone();
    shutters["ntaCalculation"]["windowSolar"]["movableShading"] = json!({
        "device": { "kind": "external_roller_shutter", "colour": "white" },
        "control": "manual_residential",
        "sourceReference": "table 7.5 device"
    });
    options.push(("table 7.5 shading device", shutters));
    options.push((
        "example office",
        without_nulls(fixture("nta8800-example-office.json")),
    ));
    let mut unknown = terraced.clone();
    unknown["ntaCalculation"]["generator"]["kind"] = json!("not_a_generator");
    options.push(("unknown generator kind", unknown));
    let mut without = terraced.clone();
    without.as_object_mut().unwrap().remove("ntaCalculation");
    options.push(("without ntaCalculation", without));
    for (label, project) in options {
        for edition in ["2025+C1", "2024", "2023", "2022", "2020+A1"] {
            let mut project = project.clone();
            if project.get("ntaCalculation").is_some() {
                project["ntaCalculation"]["normVersion"] = json!(edition);
            }
            let (status, body) = post(
                "/v1/nta8800/project/performance",
                json!({ "project": project }),
            )
            .await;
            let context = format!("{label} {edition}: {status} {body}");
            match status {
                StatusCode::OK => assert!(
                    body["status"].as_str().unwrap().starts_with("calculated"),
                    "{context}"
                ),
                StatusCode::UNPROCESSABLE_ENTITY => {
                    // The refusal names its cause in the project gaps, also
                    // for a route the building calculation refuses.
                    let codes: Vec<&Value> = body["gaps"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .map(|entry| &entry["code"])
                        .collect();
                    assert!(!codes.is_empty(), "{context}");
                    assert!(
                        codes.iter().all(|code| *code != "non_finite_result"),
                        "{context}"
                    );
                }
                _ => panic!("{context}"),
            }
        }
    }
    // A shape error of the survey (an unknown variant) is the error envelope.
    for edition in ["2025+C1", "2020+A1"] {
        let mut survey = fixture("nta8800-opname-1930-terraced.json");
        survey["normVersion"] = json!(edition);
        let (status, body) = post(
            "/v1/nta8800/opname/residential",
            json!({ "survey": survey.clone() }),
        )
        .await;
        // An older edition may refuse a route of the survey, with its codes.
        match status {
            StatusCode::OK => {}
            StatusCode::UNPROCESSABLE_ENTITY => assert!(
                body["issues"]
                    .as_array()
                    .is_some_and(|issues| !issues.is_empty()),
                "{edition}: {body}"
            ),
            _ => panic!("{edition}: {status} {body}"),
        }
        survey["dwelling"] = json!("not_a_dwelling");
        let (status, body) = post(
            "/v1/nta8800/opname/residential",
            json!({ "survey": survey }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{edition}: {body}");
        assert_envelope(&body, "invalid_request_shape");
    }
}

fn project_request(project: &Value) -> Request<Body> {
    Request::post("/v1/nta8800/project/performance")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "project": project }).to_string()))
        .unwrap()
}

/// A calculation that takes longer than the configured timeout is withheld
/// with 503 `calculation_timeout`, or `server_busy` when the request never
/// got a slot. With one slot and a normal timeout, simultaneous requests
/// queue and both calculate.
#[tokio::test]
async fn calculation_timeout_and_busy_use_the_error_envelope() {
    let project = without_nulls(fixture("nta8800-example-office.json"));
    let strict = app_with(HttpConfig {
        max_concurrent_calculations: 1,
        calculation_timeout: std::time::Duration::from_millis(1),
        ..HttpConfig::default()
    });
    let (first, second) = tokio::join!(
        send(strict.clone(), project_request(&project)),
        send(strict.clone(), project_request(&project)),
    );
    for (status, headers, body) in [first, second] {
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "{body}");
        let code = body["code"].as_str().unwrap_or_default().to_string();
        assert!(
            ["calculation_timeout", "server_busy"].contains(&code.as_str()),
            "{body}"
        );
        assert_envelope(&body, &code);
        let retry: u64 = headers[header::RETRY_AFTER]
            .to_str()
            .unwrap()
            .parse()
            .unwrap();
        assert!((1..=60).contains(&retry), "{retry}");
    }

    let queued = app_with(HttpConfig {
        max_concurrent_calculations: 1,
        ..HttpConfig::default()
    });
    let (first, second) = tokio::join!(
        send(queued.clone(), project_request(&project)),
        send(queued.clone(), project_request(&project)),
    );
    for (status, _, body) in [first, second] {
        assert_eq!(status, StatusCode::OK, "{}", body["gaps"]);
    }
}

/// The queue is bounded before the body is read: with one slot and no
/// waiting places, a second simultaneous request is refused at once with
/// `server_busy` and a `Retry-After`, while the first calculates.
#[tokio::test]
async fn full_queue_is_refused_before_the_body_is_read() {
    let project = without_nulls(fixture("nta8800-example-office.json"));
    let no_queue = app_with(HttpConfig {
        max_concurrent_calculations: 1,
        max_waiting_requests: 0,
        ..HttpConfig::default()
    });
    let (first, second) = tokio::join!(
        send(no_queue.clone(), project_request(&project)),
        send(no_queue.clone(), project_request(&project)),
    );
    assert_eq!(first.0, StatusCode::OK, "{}", first.2["gaps"]);
    assert_eq!(second.0, StatusCode::SERVICE_UNAVAILABLE, "{}", second.2);
    assert_envelope(&second.2, "server_busy");
    assert!(second.1.contains_key(header::RETRY_AFTER));
}

/// A body with more JSON values than the cap is refused before it is
/// parsed, whatever its byte size (a 16 MiB array of zeros is 8 million
/// values); a realistic project stays far below the default cap.
#[tokio::test]
async fn too_many_json_values_are_refused_before_parsing() {
    let small_cap = app_with(HttpConfig {
        max_json_values: 1_000,
        ..HttpConfig::default()
    });
    let body = json!({ "project": { "values": vec![0; 2_000] } });
    let (status, _, body) = send(
        small_cap,
        Request::post("/v1/nta8800/relabel/label-input-hash")
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_envelope(&body, "payload_too_large");
    assert_eq!(body["details"]["maxJsonValues"], 1_000);
    let (status, body) = post(
        "/v1/nta8800/project/performance",
        json!({ "project": without_nulls(fixture("nta8800-public-comparison-h.json")) }),
    )
    .await;
    assert_ne!(status, StatusCode::PAYLOAD_TOO_LARGE, "{body}");
}

/// A misspelled or unknown top-level member is refused, not ignored: a
/// `normversion` would otherwise calculate silently in the default edition.
#[tokio::test]
async fn unknown_request_members_are_refused() {
    let project = without_nulls(fixture("nta8800-example-office.json"));
    let (status, body) = post(
        "/v1/nta8800/project/performance",
        json!({ "project": project, "normversion": "2024" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_envelope(&body, "invalid_request_shape");
    assert_eq!(body["path"], "normversion");
    // Every request schema says so.
    let document = openapi_document();
    for op in operations().iter().filter(|op| op.method == Method::Post) {
        let schema = &document["paths"][op.path]["post"]["requestBody"]["content"]
            ["application/json"]["schema"];
        assert_eq!(schema["additionalProperties"], false, "{}", op.name);
    }
}

/// Relabel and the label-input hash need project objects.
#[tokio::test]
async fn relabel_and_hash_refuse_non_projects() {
    let (status, body) = post(
        "/v1/nta8800/relabel/assess",
        json!({ "original": 1, "current": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_envelope(&body, "invalid_project_shape");
    assert_eq!(body["path"], "original");
    let (status, body) = post(
        "/v1/nta8800/relabel/assess",
        json!({ "original": {}, "current": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["path"], "current");
    let (status, body) = post(
        "/v1/nta8800/relabel/label-input-hash",
        json!({ "project": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_envelope(&body, "invalid_project_shape");
    assert_eq!(body["path"], "project");
}

/// A reference case is compared, and stamped, in its own edition.
#[tokio::test]
async fn reference_case_is_stamped_with_its_own_edition() {
    let case = json!({
        "caseId": "edition-2024", "normVersion": "2024",
        "project": {"id":"house", "name":"House", "buildingFunction":"residential",
            "ntaCalculation": {"normVersion": "2024"},
            "zones":[{"id":"z", "floorArea":100, "volume":250,
                "surfaces":[{"id":"s", "area":50, "zoneId":"z", "windows":[]}]}]},
        "source": {"publisher":"synthetic", "documentId":"internal-1", "edition":"test",
            "usePermission":"internal", "independentReviewer":"test"},
        "expected": [{"path":"beng2", "value":1, "unit":"kWh/m2.year",
            "normReference":"internal test", "absoluteTolerance":0}]
    });
    let label_2024 = nta8800_core::norm_versions::NormVersion::V2024.label();
    for route in [
        "/v1/nta8800/reference/compare",
        "/v1/nta8800/reference/audit",
    ] {
        let (_, body) = post(route, json!({ "case": case })).await;
        assert_eq!(body["normVersion"], "2024", "{route}: {body}");
        assert_eq!(body["targetNormVersion"], label_2024, "{route}: {body}");
        // The same edition asked for explicitly is accepted.
        let (status, body) = post(route, json!({ "case": case, "normVersion": "2024" })).await;
        assert_ne!(status, StatusCode::BAD_REQUEST, "{route}: {body}");
    }
}

/// Capabilities name the calculations the kernel has, multi-zone, and the
/// editions, consistent with /v1/version.
#[tokio::test]
async fn capabilities_list_the_current_scope_and_editions() {
    let (_, capabilities) = get("/v1/nta8800/capabilities").await;
    let (_, version) = get("/v1/version").await;
    assert_eq!(capabilities["projectCalculationAvailable"], true);
    assert_eq!(
        capabilities["supportedNormVersions"],
        version["supportedNormVersions"]
    );
    let implemented: Vec<&str> = capabilities["implemented"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for scope in [
        "unverified_project_performance_multi_zone",
        "unverified_isso_75_1_basisopname_utility",
        "unverified_maatwerkadvies_isso_82_2_75_2",
        "relabel_comparison_brl_9500_annex_6a_6b",
        "registration_readiness_brl_9500_omgevingsregeling",
    ] {
        assert!(implemented.contains(&scope), "{scope}");
    }
    assert!(!implemented
        .iter()
        .any(|scope| scope.contains("adapter_single_zone")));
}

/// The energy-by-service projection keeps the identity and status fields
/// of the other projections.
#[tokio::test]
async fn energy_by_service_keeps_identity_fields() {
    let project = without_nulls(fixture("nta8800-example-office.json"));
    let (status, body) = post(
        "/v1/nta8800/project/energy-by-service",
        json!({ "project": project }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for key in [
        "status",
        "kernelVersion",
        "normVersion",
        "targetNormVersion",
        "registrationEligible",
        "attestStatus",
        "warnings",
        "inputFingerprint",
        "energyByService",
    ] {
        assert!(body.get(key).is_some(), "{key}: {body}");
    }
}

/// On a loopback service only the loopback host names are answered, so a
/// page that rebinds its own DNS name to 127.0.0.1 is refused.
#[tokio::test]
async fn host_check_refuses_rebound_names() {
    let config = HttpConfig {
        allowed_hosts: nta8800_service::http::LOOPBACK_HOSTS
            .iter()
            .map(|host| host.to_string())
            .collect(),
        ..HttpConfig::default()
    };
    let with_host = |host: &str| {
        Request::get("/health")
            .header(header::HOST, host)
            .body(Body::empty())
            .unwrap()
    };
    for host in [
        "127.0.0.1:3007",
        "localhost",
        "[::1]:3007",
        "LOCALHOST:3007",
    ] {
        let (status, _, _) = send(app_with(config.clone()), with_host(host)).await;
        assert_eq!(status, StatusCode::OK, "{host}");
    }
    let (status, _, body) = send(app_with(config), with_host("evil.example:3007")).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_envelope(&body, "host_not_allowed");
}

/// The OpenAPI document lists 422 only where an operation can answer it,
/// and serves itself relative to the service.
#[test]
fn openapi_lists_reachable_statuses_only() {
    let document = openapi_document();
    assert_eq!(document["servers"][0]["url"], "/");
    for (path, refuses) in [
        ("/v1/nta8800/validate", false),
        ("/v1/nta8800/relabel/assess", false),
        ("/v1/nta8800/relabel/label-input-hash", false),
        ("/v1/nta8800/reference/audit", false),
        ("/v1/nta8800/project/performance", true),
        ("/v1/nta8800/calculate", true),
    ] {
        let responses = &document["paths"][path]["post"]["responses"];
        assert_eq!(responses.get("422").is_some(), refuses, "{path}");
        assert!(responses.get("403").is_some(), "{path}");
    }
}

/// The queue wait and the calculation have their own timeouts: a request
/// that cannot get a slot within the queue timeout is refused with
/// `server_busy` and starts no calculation; the one that has the slot gets
/// the full calculation timeout and calculates.
#[tokio::test]
async fn queue_timeout_is_separate_from_the_calculation_timeout() {
    let project = without_nulls(fixture("nta8800-example-office.json"));
    let short_queue = app_with(HttpConfig {
        max_concurrent_calculations: 1,
        max_waiting_requests: 1,
        queue_timeout: std::time::Duration::from_millis(1),
        ..HttpConfig::default()
    });
    let (first, second) = tokio::join!(
        send(short_queue.clone(), project_request(&project)),
        send(short_queue.clone(), project_request(&project)),
    );
    assert_eq!(first.0, StatusCode::OK, "{}", first.2["gaps"]);
    assert_eq!(second.0, StatusCode::SERVICE_UNAVAILABLE, "{}", second.2);
    assert_envelope(&second.2, "server_busy");
    assert!(second.1.contains_key(header::RETRY_AFTER));
}

// ------------------------------------------------------------ contract probe

/// The machine codes the OpenAPI document lists for the error envelope.
fn documented_codes(document: &Value) -> Vec<String> {
    let description = document["components"]["schemas"]["Error"]["properties"]["code"]
        ["description"]
        .as_str()
        .unwrap();
    description
        .split(|c: char| !(c.is_ascii_lowercase() || c == '_'))
        .filter(|word| word.contains('_'))
        .map(str::to_string)
        .collect()
}

/// Checks one answer against the operation's documented responses: the
/// status is listed, an error envelope carries a listed code, a 200 has the
/// result schema's required members, and no message leaks a panic or a
/// source path.
fn assert_within_contract(
    document: &Value,
    path: &str,
    method: &str,
    probe: &str,
    status: StatusCode,
    body: &Value,
) {
    let responses = &document["paths"][path][method]["responses"];
    assert!(
        responses.get(status.as_str()).is_some(),
        "{method} {path} ({probe}) answered {status}, not documented: {body}"
    );
    let text = body.to_string();
    for leak in ["panicked", "/home/", "src/", ".rs:", "RUST_BACKTRACE"] {
        assert!(
            !text.contains(leak),
            "{path} ({probe}) leaks {leak}: {body}"
        );
    }
    if status == StatusCode::OK {
        let schema = &responses["200"]["content"]["application/json"]["schema"];
        let kind = if body.is_array() { "array" } else { "object" };
        assert!(
            body.is_object() || body.is_array(),
            "{path} ({probe}): {body}"
        );
        assert_eq!(schema["type"], kind, "{path} ({probe}) result type");
        for key in schema["required"].as_array().into_iter().flatten() {
            let key = key.as_str().unwrap();
            assert!(body.get(key).is_some(), "{path} ({probe}) lacks {key}");
        }
        return;
    }
    if status == StatusCode::UNPROCESSABLE_ENTITY && body.get("code").is_none() {
        // A kernel assessment, not an envelope.
        assert!(body["status"].is_string(), "{path} ({probe}): {body}");
        return;
    }
    let code = body["code"].as_str().unwrap_or("?").to_string();
    assert_envelope(body, &code);
    assert!(
        documented_codes(document).contains(&code),
        "{path} ({probe}) answered code {code}, not listed in the OpenAPI Error schema"
    );
}

/// Every code the service sources put in an error envelope (an
/// `error_body("…"` call or an edition refusal's `code: "…"`) is one of
/// [`ERROR_CODES`], so the OpenAPI list cannot fall behind the code.
#[test]
fn every_emitted_error_code_is_listed() {
    let root = env!("CARGO_MANIFEST_DIR");
    let sources = [
        "src/operations.rs",
        "src/http.rs",
        "src/bin/mcp.rs",
        "../nta8800-core/src/norm_versions/request.rs",
    ];
    let mut found = Vec::new();
    for source in sources {
        let text = std::fs::read_to_string(format!("{root}/{source}")).unwrap();
        for marker in ["error_body(", "code: "] {
            for (at, _) in text.match_indices(marker) {
                let rest = text[at + marker.len()..].trim_start();
                if let Some(literal) = rest.strip_prefix('"') {
                    let code = literal.split('"').next().unwrap();
                    found.push((source, code.to_string()));
                }
            }
        }
    }
    assert!(found.len() > 15, "{found:?}");
    for (source, code) in found {
        assert!(
            ERROR_CODES.contains(&code.as_str()),
            "{source} emits {code}, not in ERROR_CODES"
        );
    }
    let listed = documented_codes(&openapi_document());
    for code in ERROR_CODES {
        assert!(listed.iter().any(|item| item == code), "{code}");
    }
}

/// Every documented operation, probed in process with malformed and
/// edge-case requests, answers only the statuses and error codes its OpenAPI
/// entry lists.
#[tokio::test]
async fn every_operation_answers_within_its_documented_contract() {
    let document = openapi_document();
    let office = without_nulls(fixture("nta8800-example-office.json"));
    let survey = fixture("nta8800-opname-1930-terraced.json");
    for op in operations() {
        let path = op.path;
        if op.method == Method::Get {
            let (status, body) = get(path).await;
            assert_within_contract(&document, path, "get", "get", status, &body);
            let (status, body) = post(path, json!({})).await;
            assert_within_contract(&document, path, "get", "post on get", status, &body);
            continue;
        }
        let key = op.input.unwrap().key;
        let mut probes: Vec<(&str, Value)> = vec![
            ("empty", json!({})),
            ("unknown member", json!({ key: {}, "normversion": "2022" })),
            ("bad edition", json!({ key: {}, "normVersion": "1999" })),
            ("edition number", json!({ key: {}, "normVersion": 2022 })),
            ("older edition", json!({ key: {}, "normVersion": "2022" })),
        ];
        for (name, value) in [
            ("null", Value::Null),
            ("number", json!(1)),
            ("string", json!("x")),
            ("array", json!([])),
            ("object", json!({})),
            (
                "null members",
                json!({ "ntaCalculation": null, "normVersion": null }),
            ),
        ] {
            let mut body = json!({ key: value });
            if op.name == "assess_relabel" {
                body["current"] = body[key].clone();
            }
            probes.push((name, body));
        }
        if key == "project" {
            probes.push(("example office", json!({ "project": office.clone() })));
            probes.push((
                "example office 2022",
                json!({ "project": office.clone(), "normVersion": "2022" }),
            ));
        }
        if op.name == "assess_relabel" {
            probes.push((
                "example office pair",
                json!({ "original": office.clone(), "current": office.clone() }),
            ));
        }
        if key == "survey" {
            probes.push(("survey fixture", json!({ "survey": survey.clone() })));
        }
        for (probe, body) in probes {
            let (status, answer) = post(path, body).await;
            assert_within_contract(&document, path, "post", probe, status, &answer);
        }
        let (status, _, answer) = send(
            app(),
            Request::post(path)
                .header("content-type", "text/plain")
                .body(Body::from("{}"))
                .unwrap(),
        )
        .await;
        assert_within_contract(&document, path, "post", "text/plain", status, &answer);
        let (status, answer) = get(path).await;
        assert_within_contract(&document, path, "post", "get on post", status, &answer);
    }
}
