//! HTTP adapter: one route per [`crate::operations::Operation`], plus
//! `/health` and `/v1/openapi.json`, with a common error envelope, a body
//! size limit, a limit on simultaneous calculations, a calculation timeout,
//! optional CORS and request logging.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Semaphore;

use axum::{
    extract::{rejection::JsonRejection, DefaultBodyLimit, Request, State},
    http::{header, HeaderMap, HeaderValue, Method as HttpMethod, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::operations::{error_body, execute, operations, Method, Operation, Outcome};

/// Runtime options of the HTTP adapter.
#[derive(Debug, Clone)]
pub struct HttpConfig {
    /// Largest accepted request body in bytes.
    pub body_limit_bytes: usize,
    /// Origins allowed by CORS. Empty: no CORS headers. `*`: any origin.
    pub cors_origins: Vec<String>,
    /// Write one line per request to stderr.
    pub log_requests: bool,
    /// Calculations that may run at the same time. A project takes memory
    /// roughly in proportion to its zones (about 0,7 MB per zone), so this
    /// bounds the memory of the service. Further requests wait for a slot.
    pub max_concurrent_calculations: usize,
    /// Longest wait for a free calculation slot plus the calculation itself.
    /// After it the client gets 503 (`server_busy` while waiting,
    /// `calculation_timeout` while calculating); a calculation that is
    /// already running finishes in the background and keeps its slot.
    pub calculation_timeout: Duration,
}

/// Default number of simultaneous calculations: the available cores.
pub fn default_max_concurrent_calculations() -> usize {
    std::thread::available_parallelism().map_or(4, |n| n.get())
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            body_limit_bytes: 16 * 1024 * 1024,
            cors_origins: Vec::new(),
            log_requests: false,
            max_concurrent_calculations: default_max_concurrent_calculations(),
            calculation_timeout: Duration::from_secs(120),
        }
    }
}

/// Shared limits of the calculation routes.
struct Limits {
    slots: Arc<Semaphore>,
    timeout: Duration,
}

fn respond(outcome: Outcome) -> Response {
    let status = StatusCode::from_u16(outcome.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(outcome.body)).into_response()
}

fn rejection_outcome(rejection: JsonRejection) -> Outcome {
    let status = rejection.status();
    let (http, code) = match status {
        StatusCode::PAYLOAD_TOO_LARGE => (413, "payload_too_large"),
        StatusCode::UNSUPPORTED_MEDIA_TYPE => (415, "unsupported_media_type"),
        _ => (400, "invalid_json"),
    };
    Outcome {
        status: http,
        body: error_body(code, rejection.body_text(), None, Value::Null),
    }
}

fn busy(code: &'static str, message: String) -> Response {
    respond(Outcome {
        status: 503,
        body: error_body(code, message, None, Value::Null),
    })
}

async fn dispatch(
    op: &'static Operation,
    limits: Arc<Limits>,
    payload: Result<Json<Value>, JsonRejection>,
) -> Response {
    let body = match payload {
        Ok(Json(body)) => body,
        Err(rejection) => return respond(rejection_outcome(rejection)),
    };
    if !body.is_object() {
        return respond(Outcome {
            status: 400,
            body: error_body(
                "invalid_request_shape",
                "The request body must be a JSON object",
                None,
                Value::Null,
            ),
        });
    }
    let started = Instant::now();
    let slot =
        match tokio::time::timeout(limits.timeout, limits.slots.clone().acquire_owned()).await {
            Ok(Ok(slot)) => slot,
            _ => {
                return busy(
                    "server_busy",
                    format!(
                        "No calculation slot became free within {} s; try again later",
                        limits.timeout.as_secs_f64()
                    ),
                )
            }
        };
    let remaining = limits.timeout.saturating_sub(started.elapsed());
    // Kernel work is CPU-bound; keep it off the async workers. The slot is
    // released when the calculation ends, also after a timeout.
    let work = tokio::task::spawn_blocking(move || {
        let _slot = slot;
        execute(op, &body)
    });
    let joined = match tokio::time::timeout(remaining, work).await {
        Ok(joined) => joined,
        Err(_) => {
            return busy(
                "calculation_timeout",
                format!(
                    "The calculation took longer than {} s; the result is withheld",
                    limits.timeout.as_secs_f64()
                ),
            )
        }
    };
    match joined {
        Ok(outcome) => respond(outcome),
        Err(_) => respond(Outcome {
            status: 500,
            body: error_body(
                "kernel_panic",
                "The kernel stopped unexpectedly; the result is withheld",
                None,
                Value::Null,
            ),
        }),
    }
}

async fn dispatch_get(op: &'static Operation) -> Response {
    respond((op.run)(&Value::Object(Default::default())))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "kernel": "rust", "kernelVersion": nta8800_core::KERNEL_VERSION }))
}

async fn openapi() -> Json<Value> {
    Json(openapi_document())
}

async fn not_found(request: Request) -> Response {
    respond(Outcome {
        status: 404,
        body: error_body(
            "not_found",
            format!(
                "No route {} {}; see /v1/openapi.json",
                request.method(),
                request.uri().path()
            ),
            None,
            Value::Null,
        ),
    })
}

async fn method_not_allowed(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        let mut wrapped = respond(Outcome {
            status: 405,
            body: error_body(
                "method_not_allowed",
                format!("{method} is not allowed on {path}"),
                None,
                Value::Null,
            ),
        });
        if let Some(allow) = response.headers().get(header::ALLOW) {
            wrapped.headers_mut().insert(header::ALLOW, allow.clone());
        }
        return wrapped;
    }
    response
}

fn allowed_origin(config: &HttpConfig, headers: &HeaderMap) -> Option<HeaderValue> {
    let origin = headers.get(header::ORIGIN)?;
    if config.cors_origins.iter().any(|allowed| allowed == "*") {
        return Some(HeaderValue::from_static("*"));
    }
    let text = origin.to_str().ok()?;
    config
        .cors_origins
        .iter()
        .any(|allowed| allowed == text)
        .then(|| origin.clone())
}

async fn cors(State(config): State<Arc<HttpConfig>>, request: Request, next: Next) -> Response {
    if config.cors_origins.is_empty() {
        return next.run(request).await;
    }
    let origin = allowed_origin(&config, request.headers());
    if request.method() == HttpMethod::OPTIONS {
        let mut response = StatusCode::NO_CONTENT.into_response();
        if let Some(origin) = origin {
            let headers = response.headers_mut();
            headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("GET, POST, OPTIONS"),
            );
            headers.insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("content-type"),
            );
            headers.insert(
                header::ACCESS_CONTROL_MAX_AGE,
                HeaderValue::from_static("600"),
            );
            headers.insert(header::VARY, HeaderValue::from_static("origin"));
        }
        return response;
    }
    let mut response = next.run(request).await;
    if let Some(origin) = origin {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        response
            .headers_mut()
            .insert(header::VARY, HeaderValue::from_static("origin"));
    }
    response
}

async fn log_requests(
    State(config): State<Arc<HttpConfig>>,
    request: Request,
    next: Next,
) -> Response {
    if !config.log_requests {
        return next.run(request).await;
    }
    let started = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    eprintln!(
        "{method} {path} {} {:.1} ms",
        response.status().as_u16(),
        started.elapsed().as_secs_f64() * 1000.0
    );
    response
}

/// The HTTP application with default options (no CORS, no logging, 16 MiB).
pub fn app() -> Router {
    app_with(HttpConfig::default())
}

/// The HTTP application with the given options.
pub fn app_with(config: HttpConfig) -> Router {
    let limits = Arc::new(Limits {
        slots: Arc::new(Semaphore::new(config.max_concurrent_calculations.max(1))),
        timeout: config.calculation_timeout,
    });
    let config = Arc::new(config);
    let mut router = Router::new()
        .route("/health", get(health))
        .route("/v1/openapi.json", get(openapi));
    for op in operations() {
        router = match op.method {
            Method::Get => router.route(op.path, get(move || dispatch_get(op))),
            Method::Post => {
                let limits = limits.clone();
                router.route(
                    op.path,
                    post(move |payload: Result<Json<Value>, JsonRejection>| {
                        dispatch(op, limits.clone(), payload)
                    }),
                )
            }
        };
    }
    router
        .fallback(not_found)
        .layer(middleware::from_fn(method_not_allowed))
        .layer(DefaultBodyLimit::max(config.body_limit_bytes))
        .layer(middleware::from_fn_with_state(config.clone(), cors))
        .layer(middleware::from_fn_with_state(config, log_requests))
}

fn input_schema(op: &Operation) -> Value {
    let Some(input) = op.input else {
        return json!({ "type": "object", "properties": {}, "additionalProperties": false });
    };
    let mut properties = serde_json::Map::new();
    properties.insert(
        input.key.to_string(),
        json!({ "type": "object", "description": input.description }),
    );
    let mut required = vec![input.key];
    let editions: Vec<&str> = nta8800_core::norm_versions::NormVersion::ALL
        .iter()
        .map(|version| version.id())
        .collect();
    properties.insert(
        crate::operations::NORM_VERSION_MEMBER.to_string(),
        json!({
            "type": "string",
            "enum": editions,
            "default": nta8800_core::norm_versions::NormVersion::default().id(),
            "description": "NTA 8800 edition to calculate with (default 2025+C1, the only registrable one). Written into the input's own edition (ntaCalculation.normVersion, survey or building normVersion, maatwerkadvies base, both relabel projects) when absent and must match it when present; diagnostic routes run with it active. Results record normVersion and targetNormVersion; an older edition gives status calculated_legacy_edition and is never registrable, except a relabel in the original's edition. Reference cases accept only 2025+C1."
        }),
    );
    if op.name == "assess_relabel" {
        properties.insert(
            "current".to_string(),
            json!({ "type": "object", "description": "The current project, compared with `original`." }),
        );
        required.push("current");
    }
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
    })
}

/// JSON Schema of an operation's request body (also the MCP tool input schema).
pub fn request_schema(op: &Operation) -> Value {
    input_schema(op)
}

/// OpenAPI 3.1 document built from the operation registry.
pub fn openapi_document() -> Value {
    let error = json!({ "$ref": "#/components/schemas/Error" });
    let mut paths = serde_json::Map::new();
    paths.insert(
        "/health".to_string(),
        json!({ "get": {
            "operationId": "health",
            "tags": ["service"],
            "summary": "Liveness check",
            "responses": { "200": { "description": "Service is up",
                "content": { "application/json": { "schema": { "type": "object" } } } } }
        }}),
    );
    paths.insert(
        "/v1/openapi.json".to_string(),
        json!({ "get": {
            "operationId": "openapi",
            "tags": ["service"],
            "summary": "This OpenAPI 3.1 document",
            "responses": { "200": { "description": "OpenAPI document",
                "content": { "application/json": { "schema": { "type": "object" } } } } }
        }}),
    );
    for op in operations() {
        let summary = op
            .description
            .split(['.', ';', ':'])
            .next()
            .unwrap_or(op.description);
        let mut responses = serde_json::Map::new();
        responses.insert(
            "200".into(),
            json!({ "description": "Result", "content": { "application/json": { "schema": { "type": "object" } } } }),
        );
        if op.method == Method::Post {
            responses.insert("400".into(), json!({ "description": "Malformed request: invalid JSON, missing member or wrong input shape", "content": { "application/json": { "schema": error } } }));
            responses.insert("413".into(), json!({ "description": "Request body larger than the configured limit", "content": { "application/json": { "schema": error } } }));
            responses.insert("415".into(), json!({ "description": "Content-Type is not application/json", "content": { "application/json": { "schema": error } } }));
            responses.insert("422".into(), json!({ "description": "The kernel refuses or cannot complete the input; the body is the assessment with its status, gaps and issues", "content": { "application/json": { "schema": { "type": "object" } } } }));
            responses.insert("500".into(), json!({ "description": "Result withheld (non_finite_result or serialization_failed) or kernel failure", "content": { "application/json": { "schema": error } } }));
            responses.insert("503".into(), json!({ "description": "No calculation slot became free in time (server_busy) or the calculation exceeded the timeout (calculation_timeout)", "content": { "application/json": { "schema": error } } }));
        }
        if op.name == "calculate_beng" {
            responses.insert("501".into(), json!({ "description": "Legacy endpoint, calculation unavailable", "content": { "application/json": { "schema": error } } }));
        }
        let mut operation = json!({
            "operationId": op.name,
            "tags": [op.category],
            "summary": summary,
            "description": op.description,
            "x-mcp-tool": op.name,
            "responses": responses,
        });
        if op.method == Method::Post {
            operation["requestBody"] = json!({
                "required": true,
                "content": { "application/json": { "schema": input_schema(op) } }
            });
        }
        let method = match op.method {
            Method::Get => "get",
            Method::Post => "post",
        };
        paths.insert(op.path.to_string(), json!({ method: operation }));
    }
    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Open Energy Studio NTA 8800 API",
            "version": env!("CARGO_PKG_VERSION"),
            "description": format!(
                "HTTP API of the Open Energy Studio Rust kernel for {}. Results are unverified: the program has no BRL 9501 attest yet. Kernel version {}.",
                nta8800_core::TARGET_NORM_VERSION,
                nta8800_core::KERNEL_VERSION
            ),
            "license": { "name": "LGPL-3.0-or-later" }
        },
        "servers": [{ "url": "http://127.0.0.1:3007" }],
        "paths": paths,
        "components": { "schemas": { "Error": {
            "type": "object",
            "required": ["error", "code", "message"],
            "properties": {
                "error": { "type": "string", "description": "Machine code (same as code; kept for older clients)" },
                "code": { "type": "string", "description": "Machine code, e.g. invalid_json, invalid_request_shape, missing_request_member, invalid_project_shape, payload_too_large, unsupported_media_type, not_found, method_not_allowed, non_finite_result, serialization_failed, kernel_panic, calculation_unavailable" },
                "message": { "type": "string" },
                "path": { "type": ["string", "null"], "description": "JSON path of the offending input or result value" },
                "details": { "description": "Optional extra data" }
            }
        } } }
    })
}
