//! HTTP adapter: one route per [`crate::operations::Operation`], plus
//! `/health` and `/v1/openapi.json`, with a common error envelope, a body
//! size limit, a limit on simultaneous calculations, a bounded queue of
//! waiting requests, separate queue and calculation timeouts, optional CORS
//! and request logging.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Semaphore;

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, FromRequest, Request, State},
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
    /// Largest number of JSON values (array elements, object members,
    /// scalars) in a request body. The parsed body takes memory per value,
    /// not per byte (about 30 bytes for a compact array element, roughly
    /// 100 for an object member), so this caps a request's parsed body
    /// whatever its byte size. A body above it is refused with 413
    /// `payload_too_large` before it is parsed.
    pub max_json_values: usize,
    /// Origins allowed by CORS. Empty: no CORS headers. `*`: any origin.
    pub cors_origins: Vec<String>,
    /// Host names (the `Host` header without its port) the service answers.
    /// Empty: any. The API binary fills it with `localhost`, `127.0.0.1`
    /// and `::1` when it listens on a loopback address, so a web page that
    /// rebinds its own DNS name to 127.0.0.1 cannot reach the service; any
    /// other host gets 403 `host_not_allowed`.
    pub allowed_hosts: Vec<String>,
    /// Write one line per request to stderr.
    pub log_requests: bool,
    /// Calculations that may run at the same time. Further requests wait
    /// for a slot, up to [`HttpConfig::max_waiting_requests`].
    pub max_concurrent_calculations: usize,
    /// Requests that may wait for a calculation slot. A waiting request
    /// already holds its parsed body. The admitted requests (running plus
    /// waiting) each hold at most one body of at most
    /// [`HttpConfig::max_json_values`] values: no route copies it (see
    /// [`crate::operations::Operation::run`]). A request beyond the queue is
    /// refused at once with 503 `server_busy`, before its body is read.
    /// `0`: no queue.
    pub max_waiting_requests: usize,
    /// Longest wait for a free calculation slot. After it the client gets
    /// 503 `server_busy` and no calculation is started.
    pub queue_timeout: Duration,
    /// Longest calculation, counted from the moment it gets its slot. After
    /// it the client gets 503 `calculation_timeout`; the calculation
    /// finishes in the background and keeps its slot until then.
    pub calculation_timeout: Duration,
}

/// Default number of simultaneous calculations: the available cores,
/// within the 1–1024 range the API accepts.
pub fn default_max_concurrent_calculations() -> usize {
    std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .clamp(1, MAX_CALCULATION_SLOTS)
}

/// Upper bound of simultaneous calculations accepted by the API.
pub const MAX_CALCULATION_SLOTS: usize = 1024;

/// Default number of waiting requests: twice the calculation slots.
pub fn default_max_waiting_requests() -> usize {
    default_max_concurrent_calculations() * 2
}

/// Default cap on the JSON values of a request: 500 000. A realistic
/// project has about 650 values per calculation zone (public case H: 1 317
/// values for 2 zones, 48 kB), so this admits about 750 zones, nearly twice
/// the largest project the kernel benchmark runs (400 zones).
pub const DEFAULT_MAX_JSON_VALUES: usize = 500_000;

/// Host names a loopback-bound service answers by default.
pub const LOOPBACK_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "::1"];

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            body_limit_bytes: 16 * 1024 * 1024,
            max_json_values: DEFAULT_MAX_JSON_VALUES,
            cors_origins: Vec::new(),
            allowed_hosts: Vec::new(),
            log_requests: false,
            max_concurrent_calculations: default_max_concurrent_calculations(),
            max_waiting_requests: default_max_waiting_requests(),
            queue_timeout: Duration::from_secs(30),
            calculation_timeout: Duration::from_secs(120),
        }
    }
}

/// Shared limits of the calculation routes.
struct Limits {
    /// Calculations that may run at once.
    slots: Arc<Semaphore>,
    /// Requests that may be admitted at once: running plus waiting.
    admission: Arc<Semaphore>,
    queue_timeout: Duration,
    calculation_timeout: Duration,
    /// `Retry-After` of a 503, in seconds.
    retry_after_secs: u64,
    /// Cap on the JSON values of a request body.
    max_json_values: usize,
}

/// `Retry-After` hint of a 503: a quarter of the calculation timeout,
/// between 1 and 60 s.
fn retry_after_secs(config: &HttpConfig) -> u64 {
    (config.calculation_timeout.as_secs() / 4).clamp(1, 60)
}

fn respond(outcome: Outcome) -> Response {
    let status = StatusCode::from_u16(outcome.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, Json(outcome.body)).into_response()
}

fn outcome(status: u16, body: Value) -> Outcome {
    Outcome { status, body }
}

fn refusal(status: u16, body: Value) -> Response {
    respond(outcome(status, body))
}

/// Whether the request declares a JSON body (`application/json` or an
/// `application/*+json` type), as axum's JSON extractor requires.
fn is_json_content_type(headers: &HeaderMap) -> bool {
    let Some(value) = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    let mime = value
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    mime == "application/json" || (mime.starts_with("application/") && mime.ends_with("+json"))
}

/// An upper estimate of the JSON values in `bytes`: one per opening
/// bracket and per comma, plus one. `None` as soon as it exceeds `limit`, so
/// a huge body is refused without a full scan. Commas and brackets inside
/// strings do not count.
pub fn json_value_count(bytes: &[u8], limit: usize) -> Option<usize> {
    let mut count = 1usize;
    let mut in_string = false;
    let mut escaped = false;
    for &byte in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b',' | b'[' | b'{' => {
                count += 1;
                if count > limit {
                    return None;
                }
            }
            _ => {}
        }
    }
    Some(count)
}

/// Reads and parses a request body: JSON content type, at most the body
/// limit in bytes (the [`DefaultBodyLimit`] layer) and at most
/// `max_values` JSON values, a JSON object.
async fn read_json_body(request: Request, max_values: usize) -> Result<Value, Outcome> {
    if !is_json_content_type(request.headers()) {
        return Err(outcome(
            415,
            error_body(
                "unsupported_media_type",
                "Content-Type must be application/json",
                None,
                Value::Null,
            ),
        ));
    }
    let bytes = match Bytes::from_request(request, &()).await {
        Ok(bytes) => bytes,
        Err(rejection) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => {
            return Err(outcome(
                413,
                error_body(
                    "payload_too_large",
                    rejection.body_text(),
                    None,
                    Value::Null,
                ),
            ))
        }
        Err(rejection) => {
            return Err(outcome(
                400,
                error_body("invalid_json", rejection.body_text(), None, Value::Null),
            ))
        }
    };
    if json_value_count(&bytes, max_values).is_none() {
        return Err(outcome(
            413,
            error_body(
                "payload_too_large",
                format!("The request body has more than {max_values} JSON values"),
                None,
                json!({ "maxJsonValues": max_values }),
            ),
        ));
    }
    let body: Value = match serde_json::from_slice(&bytes) {
        Ok(body) => body,
        Err(error) => {
            return Err(outcome(
                400,
                error_body(
                    "invalid_json",
                    format!("The request body is not valid JSON: {error}"),
                    None,
                    Value::Null,
                ),
            ))
        }
    };
    if !body.is_object() {
        return Err(outcome(
            400,
            error_body(
                "invalid_request_shape",
                "The request body must be a JSON object",
                None,
                Value::Null,
            ),
        ));
    }
    Ok(body)
}

fn refuse_busy(code: &'static str, message: String, retry_after_secs: u64) -> Response {
    let mut response = respond(Outcome {
        status: 503,
        body: error_body(code, message, None, Value::Null),
    });
    response.headers_mut().insert(
        header::RETRY_AFTER,
        HeaderValue::from_str(&retry_after_secs.to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("1")),
    );
    response
}

async fn dispatch(op: &'static Operation, limits: Arc<Limits>, request: Request) -> Response {
    // Admission comes before the body is read, so the queue of waiting
    // requests (each holding its body) stays bounded.
    let Ok(_admitted) = limits.admission.clone().try_acquire_owned() else {
        return refuse_busy(
            "server_busy",
            "Too many requests are waiting for a calculation slot; try again later".to_string(),
            limits.retry_after_secs,
        );
    };
    let body = match read_json_body(request, limits.max_json_values).await {
        Ok(body) => body,
        Err(outcome) => return respond(outcome),
    };
    let slot = match tokio::time::timeout(
        limits.queue_timeout,
        limits.slots.clone().acquire_owned(),
    )
    .await
    {
        Ok(Ok(slot)) => slot,
        _ => {
            return refuse_busy(
                "server_busy",
                format!(
                    "No calculation slot became free within {} s; try again later",
                    limits.queue_timeout.as_secs_f64()
                ),
                limits.retry_after_secs,
            )
        }
    };
    // Kernel work is CPU-bound; keep it off the async workers. The slot (and
    // the body, moved into the closure) is released when the calculation
    // ends, also after a timeout. The calculation gets its full timeout,
    // however long the request waited for the slot.
    let work = tokio::task::spawn_blocking(move || {
        let _slot = slot;
        execute(op, body)
    });
    let joined = match tokio::time::timeout(limits.calculation_timeout, work).await {
        Ok(joined) => joined,
        Err(_) => {
            return refuse_busy(
                "calculation_timeout",
                format!(
                    "The calculation took longer than {} s; the result is withheld",
                    limits.calculation_timeout.as_secs_f64()
                ),
                limits.retry_after_secs,
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
    respond((op.run)(Value::Object(Default::default())))
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
    // Every answer depends on the Origin header once CORS is on, also a
    // refused preflight, so caches must key on it.
    let mut response = if request.method() == HttpMethod::OPTIONS {
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
        }
        response
    } else {
        let mut response = next.run(request).await;
        if let Some(origin) = origin {
            response
                .headers_mut()
                .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        }
        response
    };
    response
        .headers_mut()
        .append(header::VARY, HeaderValue::from_static("Origin"));
    response
}

/// The host name of a `Host` header or an allowed-host entry: without its
/// port, an IPv6 address without its brackets (`[::1]:3007` and `::1` are
/// both `::1`), in lower case.
fn host_name(host: &str) -> String {
    let host = host.trim();
    let name = if let Some(rest) = host.strip_prefix('[') {
        rest.split(']').next().unwrap_or("")
    } else if host.matches(':').count() > 1 {
        // A bare IPv6 address has no port.
        host
    } else {
        host.split(':').next().unwrap_or("")
    };
    name.to_ascii_lowercase()
}

/// Refuses a request whose `Host` is not an allowed host (DNS rebinding):
/// a page that points its own name at 127.0.0.1 still sends that name.
/// A request without `Host` (not a browser) passes.
async fn host_check(
    State(config): State<Arc<HttpConfig>>,
    request: Request,
    next: Next,
) -> Response {
    if config.allowed_hosts.is_empty() {
        return next.run(request).await;
    }
    if let Some(host) = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
    {
        let name = host_name(host);
        let allowed = config
            .allowed_hosts
            .iter()
            .any(|allowed| host_name(allowed) == name);
        if !allowed {
            return refusal(
                403,
                error_body(
                    "host_not_allowed",
                    format!(
                        "Host {name} is not served; allowed: {}",
                        config.allowed_hosts.join(", ")
                    ),
                    None,
                    Value::Null,
                ),
            );
        }
    }
    next.run(request).await
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
    let slots = config.max_concurrent_calculations.max(1);
    let limits = Arc::new(Limits {
        slots: Arc::new(Semaphore::new(slots)),
        admission: Arc::new(Semaphore::new(
            slots.saturating_add(config.max_waiting_requests),
        )),
        queue_timeout: config.queue_timeout,
        calculation_timeout: config.calculation_timeout,
        retry_after_secs: retry_after_secs(&config),
        max_json_values: config.max_json_values.max(1),
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
                    post(move |request: Request| dispatch(op, limits.clone(), request)),
                )
            }
        };
    }
    router
        .fallback(not_found)
        .layer(middleware::from_fn(method_not_allowed))
        .layer(DefaultBodyLimit::max(config.body_limit_bytes))
        .layer(middleware::from_fn_with_state(config.clone(), cors))
        .layer(middleware::from_fn_with_state(config.clone(), host_check))
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
            "description": "NTA 8800 edition to calculate with (default 2025+C1, the only registrable one). Written into the input's own edition (ntaCalculation.normVersion, survey or building normVersion, maatwerkadvies base, both relabel projects) when absent and must match it when present; diagnostic routes run with it active. Results record normVersion and targetNormVersion; an older edition gives status calculated_legacy_edition and is never registrable, except a relabel in the original's edition. A reference case is compared in its own case.normVersion (any implemented edition); a request edition must match it. The two diagnostic comparisons (direct-diagnostic, gas-chain-diagnostic) take only 2025+C1."
        }),
    );
    if op.name == "compare_project_editions" {
        properties[crate::operations::NORM_VERSION_MEMBER]["description"] = json!(
            "Target edition to compare the project with (default 2025+C1). The project keeps its own edition in ntaCalculation.normVersion; the result's adjustedProject carries the target."
        );
    }
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
        "additionalProperties": false,
    })
}

/// JSON Schema of an operation's 200 result. Most results are described in
/// docs/nta8800-api.md; the edition comparison has its shape here.
fn result_schema(op: &Operation) -> Value {
    if op.name == "list_interpretations" {
        // The one result that is not an object (an MCP tool result wraps it
        // as `{"result": [...]}`, see bin/mcp.rs).
        return json!({
            "type": "array",
            "items": { "type": "object", "required": ["module", "part", "items"], "properties": {
                "module": { "type": "string" },
                "part": { "type": "string" },
                "items": { "type": "array", "items": { "type": "string" } }
            } }
        });
    }
    if op.name != "compare_project_editions" {
        return json!({ "type": "object" });
    }
    let editions: Vec<&str> = nta8800_core::norm_versions::NormVersion::ALL
        .iter()
        .map(|version| version.id())
        .collect();
    json!({
        "type": "object",
        "required": ["fromNormVersion", "toNormVersion", "switchPoints", "items", "adjustedProject", "statusAfterAutomaticChanges", "remainingAfterAutomaticChanges"],
        "properties": {
            "fromNormVersion": { "type": "string", "enum": editions, "description": "The project's own edition" },
            "toNormVersion": { "type": "string", "enum": editions, "description": "The requested edition" },
            "fromLabel": { "type": "string" },
            "toLabel": { "type": "string" },
            "switchPoints": {
                "type": "array",
                "description": "Switch points of docs/nta8800-normversies.md whose values or routes differ between the two editions",
                "items": { "type": "object", "properties": {
                    "points": { "type": "array", "items": { "type": "string" } },
                    "subject": { "type": "string" },
                    "fields": { "type": "array", "items": { "type": "string" } },
                    "hasInputs": { "type": "boolean" }
                } }
            },
            "items": {
                "type": "array",
                "items": { "type": "object", "required": ["category", "code", "path", "points", "fix", "resolvedByAutomaticChanges"], "properties": {
                    "category": { "type": "string", "enum": ["refused", "required", "ignored", "reinterpreted", "warning"] },
                    "code": { "type": "string", "description": "Kernel code, or input_ignored_in_edition / input_reinterpreted_in_edition" },
                    "path": { "type": "string", "description": "Project input path, for example ntaCalculation.groundFloors[0].below.wallHeightAboveGroundM" },
                    "points": { "type": "array", "items": { "type": "string" }, "description": "Switch point numbers" },
                    "subject": { "type": ["string", "null"] },
                    "fix": { "type": "string", "enum": ["drop", "switchOff", "set", "enter", "choose", "review"], "description": "drop, switchOff and set are applied in adjustedProject; the others are left to the adviser" },
                    "value": { "description": "The value a set or switchOff writes; with choose, the value the target edition asks for (named, never written: the table 9.28 test standard is evidence)" },
                    "resolvedByAutomaticChanges": { "type": "boolean" }
                } }
            },
            "adjustedProject": { "type": "object", "description": "The project in the target edition with the automatic changes applied" },
            "statusAfterAutomaticChanges": { "type": "string" },
            "remainingAfterAutomaticChanges": { "type": "array", "items": { "type": "object" }, "description": "Gaps of adjustedProject the project did not have in its own edition" }
        }
    })
}

/// JSON Schema of an operation's request body (also the MCP tool input schema).
pub fn request_schema(op: &Operation) -> Value {
    input_schema(op)
}

/// The 405 answer of a route called with another method; `Allow` names the
/// route's own method.
fn method_not_allowed_response(method: Method, error: &Value) -> Value {
    let allow = match method {
        Method::Get => "GET, HEAD",
        Method::Post => "POST",
    };
    json!({
        "description": format!("Called with another method than the route's own (method_not_allowed); the Allow header gives {allow}"),
        "headers": { "Allow": { "description": "The method(s) the route takes", "schema": { "type": "string" } } },
        "content": { "application/json": { "schema": error } }
    })
}

/// OpenAPI 3.1 document built from the operation registry.
pub fn openapi_document() -> Value {
    let error = json!({ "$ref": "#/components/schemas/Error" });
    let host_refused = json!({ "description": "The Host header names a host the service does not answer (host_not_allowed; protection against DNS rebinding)", "content": { "application/json": { "schema": error } } });
    let mut paths = serde_json::Map::new();
    paths.insert(
        "/health".to_string(),
        json!({ "get": {
            "operationId": "health",
            "tags": ["service"],
            "summary": "Liveness check",
            "responses": {
                "200": { "description": "Service is up",
                    "content": { "application/json": { "schema": { "type": "object" } } } },
                "403": host_refused,
                "405": method_not_allowed_response(Method::Get, &error),
            }
        }}),
    );
    paths.insert(
        "/v1/openapi.json".to_string(),
        json!({ "get": {
            "operationId": "openapi",
            "tags": ["service"],
            "summary": "This OpenAPI 3.1 document",
            "responses": {
                "200": { "description": "OpenAPI document",
                    "content": { "application/json": { "schema": { "type": "object" } } } },
                "403": host_refused,
                "405": method_not_allowed_response(Method::Get, &error),
            }
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
            json!({ "description": "Result", "content": { "application/json": { "schema": result_schema(op) } } }),
        );
        responses.insert("403".into(), host_refused.clone());
        responses.insert("405".into(), method_not_allowed_response(op.method, &error));
        if op.method == Method::Post {
            responses.insert("400".into(), json!({ "description": "Malformed request: invalid JSON, an unknown or missing member, a wrong input shape or an edition that does not fit", "content": { "application/json": { "schema": error } } }));
            responses.insert("413".into(), json!({ "description": "Request body larger than the configured byte limit or with more JSON values than the configured value limit", "content": { "application/json": { "schema": error } } }));
            responses.insert("415".into(), json!({ "description": "Content-Type is not application/json", "content": { "application/json": { "schema": error } } }));
            if crate::operations::may_refuse(op) {
                responses.insert("422".into(), json!({ "description": "The kernel refuses or cannot complete the input; the body is the assessment with its status, gaps and issues", "content": { "application/json": { "schema": { "type": "object" } } } }));
            }
            responses.insert("500".into(), json!({ "description": "Result withheld (non_finite_result or serialization_failed) or kernel failure (kernel_panic)", "content": { "application/json": { "schema": error } } }));
            responses.insert("503".into(), json!({ "description": "Too many requests waiting or no calculation slot free within the queue timeout (server_busy), or the calculation exceeded the calculation timeout (calculation_timeout). The Retry-After header gives the seconds to wait before retrying.", "headers": { "Retry-After": { "description": "Seconds to wait before retrying", "schema": { "type": "integer", "minimum": 1 } } }, "content": { "application/json": { "schema": error } } }));
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
        // Relative: the document describes the service that serves it,
        // wherever it listens (by default http://127.0.0.1:3007).
        "servers": [{ "url": "/", "description": "The service serving this document (default http://127.0.0.1:3007)" }],
        "paths": paths,
        "components": { "schemas": { "Error": {
            "type": "object",
            "required": ["error", "code", "message"],
            "properties": {
                "error": { "type": "string", "description": "Machine code (same as code; kept for older clients)" },
                "code": { "type": "string", "description": format!(
                    "Machine code: {} (unknown_tool from the MCP server only)",
                    crate::operations::ERROR_CODES.join(", ")
                ) },
                "message": { "type": "string" },
                "path": { "type": ["string", "null"], "description": "JSON path of the offending input or result value" },
                "details": { "description": "Optional extra data" }
            }
        } } }
    })
}
