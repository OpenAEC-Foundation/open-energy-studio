//! MCP server (stdio) for the Open Energy Studio NTA 8800 kernel.
//!
//! Every tool is an operation from `nta8800_service::operations`, the same
//! registry the HTTP API serves, so tool `x` and route `x` always agree.
//! Resources expose the example projects, the survey fixtures, the kernel's
//! interpretation list, the Dutch user manual and the OpenAPI document.

use std::sync::Arc;

use nta8800_service::operations::{error_body, execute, operation, operations, Method};
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, Implementation,
        ListResourcesResult, ListToolsResult, PaginatedRequestParams, ReadResourceRequestParams,
        ReadResourceResponse, ReadResourceResult, Resource, ResourceContents, ServerCapabilities,
        ServerConfig, Tool, ToolAnnotations,
    },
    service::RequestContext,
    transport::stdio,
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use serde_json::{json, Map, Value};

/// A static resource: URI, name, description, MIME type and content.
struct StaticResource {
    uri: &'static str,
    name: &'static str,
    description: &'static str,
    mime: &'static str,
    content: fn() -> String,
}

macro_rules! file {
    ($path:literal) => {
        || include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../", $path)).to_string()
    };
}

static RESOURCES: &[StaticResource] = &[
    StaticResource { uri: "oes://examples/terraced-dwelling", name: "example-terraced-dwelling", description: "Fictional example project: terraced dwelling (tussenwoning) with complete NTA input; send as `project` to calculate_project_performance", mime: "application/json", content: file!("training-data/nta8800-example-terraced-dwelling.json") },
    StaticResource { uri: "oes://examples/office", name: "example-office", description: "Fictional example project: small office (kantoor) with complete NTA input; send as `project` to calculate_project_performance", mime: "application/json", content: file!("training-data/nta8800-example-office.json") },
    StaticResource { uri: "oes://surveys/residential-1930-terraced", name: "survey-residential-1930-terraced", description: "ISSO 82.1 dwelling basisopname fixture (1930 terraced); send as `survey` to assess_residential_survey", mime: "application/json", content: file!("training-data/nta8800-opname-1930-terraced.json") },
    StaticResource { uri: "oes://surveys/residential-1975-apartment", name: "survey-residential-1975-apartment", description: "ISSO 82.1 dwelling basisopname fixture (1975 apartment); send as `survey` to assess_residential_survey", mime: "application/json", content: file!("training-data/nta8800-opname-1975-apartment.json") },
    StaticResource { uri: "oes://surveys/residential-2015-detached", name: "survey-residential-2015-detached", description: "ISSO 82.1 dwelling basisopname fixture (2015 detached); send as `survey` to assess_residential_survey", mime: "application/json", content: file!("training-data/nta8800-opname-2015-detached.json") },
    StaticResource { uri: "oes://surveys/utility-1985-office", name: "survey-utility-1985-office", description: "ISSO 75.1 utility basisopname fixture (1985 office); send as `survey` to assess_utility_survey", mime: "application/json", content: file!("training-data/nta8800-opname-utility-1985-office.json") },
    StaticResource { uri: "oes://surveys/utility-2005-school", name: "survey-utility-2005-school", description: "ISSO 75.1 utility basisopname fixture (2005 school); send as `survey` to assess_utility_survey", mime: "application/json", content: file!("training-data/nta8800-opname-utility-2005-school.json") },
    StaticResource { uri: "oes://surveys/utility-1970-retail", name: "survey-utility-1970-retail", description: "ISSO 75.1 utility basisopname fixture (1970 retail); send as `survey` to assess_utility_survey", mime: "application/json", content: file!("training-data/nta8800-opname-utility-1970-retail.json") },
    StaticResource { uri: "oes://kernel/interpretations", name: "interpretations", description: "The kernel's documented readings of ambiguous NTA 8800 passages (interpretatielijst)", mime: "application/json", content: || json!(nta8800_core::interpretations::kernel_interpretations()).to_string() },
    StaticResource { uri: "oes://api/openapi.json", name: "openapi", description: "OpenAPI 3.1 document of the HTTP API; every operation is also an MCP tool with the same name", mime: "application/json", content: || nta8800_service::openapi_document().to_string() },
    StaticResource { uri: "oes://manual/index", name: "manual-index", description: "Gebruikershandleiding NTA 8800 (Dutch user manual): index", mime: "text/markdown", content: file!("docs/handleiding-nta8800/index.md") },
    StaticResource { uri: "oes://manual/01-reikwijdte-en-status", name: "manual-01-scope", description: "Handleiding 1: reikwijdte en status (scope, attest status)", mime: "text/markdown", content: file!("docs/handleiding-nta8800/01-reikwijdte-en-status.md") },
    StaticResource { uri: "oes://manual/02-basisopname", name: "manual-02-survey", description: "Handleiding 2: basisopname woningen (ISSO 82.1) en utiliteit (ISSO 75.1)", mime: "text/markdown", content: file!("docs/handleiding-nta8800/02-basisopname.md") },
    StaticResource { uri: "oes://manual/03-projectberekening", name: "manual-03-project", description: "Handleiding 3: projectberekening, invoersecties en normbasis", mime: "text/markdown", content: file!("docs/handleiding-nta8800/03-projectberekening.md") },
    StaticResource { uri: "oes://manual/04-uitvoer", name: "manual-04-output", description: "Handleiding 4: uitvoer (indicatoren, label, Bbl-toets, rapport)", mime: "text/markdown", content: file!("docs/handleiding-nta8800/04-uitvoer.md") },
    StaticResource { uri: "oes://manual/05-validatie", name: "manual-05-validation", description: "Handleiding 5: validatie, statussen, gaten en waarschuwingen", mime: "text/markdown", content: file!("docs/handleiding-nta8800/05-validatie.md") },
    StaticResource { uri: "oes://manual/06-maatwerkadvies", name: "manual-06-maatwerkadvies", description: "Handleiding 6: maatwerkadvies", mime: "text/markdown", content: file!("docs/handleiding-nta8800/06-maatwerkadvies.md") },
    StaticResource { uri: "oes://manual/07-herlabelen-registratie-dossier", name: "manual-07-relabel", description: "Handleiding 7: herlabelen, registratie en dossier", mime: "text/markdown", content: file!("docs/handleiding-nta8800/07-herlabelen-registratie-dossier.md") },
    StaticResource { uri: "oes://manual/08-versies-en-verwijzingen", name: "manual-08-versions", description: "Handleiding 8: versies en verwijzingen", mime: "text/markdown", content: file!("docs/handleiding-nta8800/08-versies-en-verwijzingen.md") },
    StaticResource { uri: "oes://manual/09-bestanden-en-uitwisseling", name: "manual-09-files", description: "Handleiding 9: bestanden, uitwisseling, API en MCP", mime: "text/markdown", content: file!("docs/handleiding-nta8800/09-bestanden-en-uitwisseling.md") },
];

fn tool_schema(op: &nta8800_service::operations::Operation) -> Arc<Map<String, Value>> {
    match nta8800_service::request_schema(op) {
        Value::Object(map) => Arc::new(map),
        _ => Arc::new(Map::new()),
    }
}

fn tools() -> Vec<Tool> {
    operations()
        .iter()
        .map(|op| {
            let description = match op.method {
                Method::Get => format!("{} (HTTP GET {})", op.description, op.path),
                Method::Post => format!("{} (HTTP POST {})", op.description, op.path),
            };
            Tool::new(op.name, description, tool_schema(op)).annotate(calculation_annotations())
        })
        .collect()
}

/// Every tool is a calculation: it reads its arguments and returns a
/// result, and changes nothing (no files, no network, no state between
/// calls). Clients may call it without asking for write permission.
fn calculation_annotations() -> ToolAnnotations {
    ToolAnnotations::new()
        .read_only(true)
        .destructive(false)
        .idempotent(true)
        .open_world(false)
}

/// Runs a tool and returns its result. Failures the caller can act on
/// (unknown tool, malformed arguments, refused input) are `isError` results
/// carrying the same envelope or assessment as the HTTP API.
pub fn call(name: &str, arguments: Option<Map<String, Value>>) -> CallToolResult {
    let Some(op) = operation(name) else {
        return CallToolResult::structured_error(error_body(
            "unknown_tool",
            format!("No tool named {name}"),
            None,
            Value::Null,
        ));
    };
    let body = Value::Object(arguments.unwrap_or_default());
    let outcome = execute(op, body);
    let is_error = outcome.is_error();
    // MCP structured content must be an object; the one list result
    // (list_interpretations) comes wrapped as `{"result": [...]}`.
    let body = if outcome.body.is_object() {
        outcome.body
    } else {
        json!({ "result": outcome.body })
    };
    if is_error {
        CallToolResult::structured_error(body)
    } else {
        CallToolResult::structured(body)
    }
}

/// Default number of tool calls that calculate at the same time: the same
/// default as the HTTP API's calculation slots.
fn max_calculations() -> usize {
    std::env::var("OES_MCP_MAX_CALCULATIONS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|slots| (1..=nta8800_service::http::MAX_CALCULATION_SLOTS).contains(slots))
        .unwrap_or_else(nta8800_service::http::default_max_concurrent_calculations)
}

#[derive(Clone)]
struct EnergyMcp {
    /// Tool calls that may calculate at once; further calls wait for a slot.
    slots: Arc<tokio::sync::Semaphore>,
}

impl ServerHandler for EnergyMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(
            Implementation::new("open-energy-studio-nta8800", env!("CARGO_PKG_VERSION"))
                .with_title("Open Energy Studio NTA 8800")
                .with_description(format!(
                    "Rust kernel {} for {}",
                    nta8800_core::KERNEL_VERSION,
                    nta8800_core::TARGET_NORM_VERSION
                )),
        )
        .with_instructions(
            "Dutch building energy performance (NTA 8800:2025+C1:2026, BENG, energielabel). \
             Start with calculate_project_performance on a project (see resources oes://examples/*), \
             or assess_residential_survey / assess_utility_survey for an ISSO 82.1/75.1 basisopname. \
             A result with status `incomplete` lists input gaps (gaten) with their JSON path. \
             Results are unverified: the program has no BRL 9501 attest yet. \
             A tool result is the JSON body the HTTP route returns; isError is set where the route answers 4xx or 5xx.",
        )
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(tools()))
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        tools().into_iter().find(|tool| tool.name == name)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let name = request.name.to_string();
        let arguments = request.arguments;
        // The semaphore is never closed, so acquiring only waits.
        let slot = self.slots.clone().acquire_owned().await.ok();
        let result = tokio::task::spawn_blocking(move || {
            let _slot = slot;
            call(&name, arguments)
        })
        .await
        .unwrap_or_else(|_| {
            CallToolResult::structured_error(error_body(
                "kernel_panic",
                "The kernel stopped unexpectedly; the result is withheld",
                None,
                Value::Null,
            ))
        });
        Ok(result.into())
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(
            RESOURCES
                .iter()
                .map(|resource| {
                    Resource::new(resource.uri, resource.name)
                        .with_description(resource.description)
                        .with_mime_type(resource.mime)
                })
                .collect(),
        ))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let Some(resource) = RESOURCES
            .iter()
            .find(|resource| resource.uri == request.uri)
        else {
            return Err(McpError::resource_not_found(
                format!("No resource {}", request.uri),
                None,
            ));
        };
        let contents = ResourceContents::text((resource.content)(), resource.uri);
        Ok(ReadResourceResult::new(vec![contents]).into())
    }
}

// ------------------------------------------------------------ stdin guard
//
// The MCP library reads one JSON-RPC message per line and leaves some lines
// unanswered: invalid or truncated JSON, JSON nested deeper than serde's 128
// levels, and an unbounded line is read into memory whole. A client then
// waits forever. The guard reads stdin first, caps the line length, answers
// those lines itself with a JSON-RPC error and passes the rest on unchanged.

/// Default cap on one JSON-RPC message line: 64 MiB.
const DEFAULT_MAX_LINE_BYTES: usize = 64 * 1024 * 1024;

/// JSON-RPC error codes (JSON-RPC 2.0 §5.1).
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const INVALID_PARAMS: i64 = -32602;

/// What to do with one input line.
#[derive(Debug, PartialEq)]
enum LineAction {
    /// Pass the line to the MCP library.
    Forward,
    /// Answer it with this JSON-RPC error line.
    Reply(String),
    /// Drop it (blank line, or an invalid notification, which gets no answer).
    Skip,
}

fn error_line(id: Value, code: i64, message: &str) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }).to_string()
}

/// The top-level `"id"` of a JSON object text, found without parsing the
/// whole text (it may be invalid or too deep). Only a string or a number id
/// is taken; anything else gives `null`, as JSON-RPC asks when the id cannot
/// be read.
fn top_level_id(bytes: &[u8]) -> Value {
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                let start = i + 1;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                let key = bytes.get(start..i.min(bytes.len())).unwrap_or(&[]);
                i += 1;
                if depth == 1 && key == b"id" {
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) != Some(&b':') {
                        continue;
                    }
                    j += 1;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    let start = j;
                    if bytes.get(j) == Some(&b'"') {
                        j += 1;
                        while j < bytes.len() && bytes[j] != b'"' {
                            j += if bytes[j] == b'\\' { 2 } else { 1 };
                        }
                        j += 1;
                    } else {
                        while j < bytes.len()
                            && matches!(bytes[j], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
                        {
                            j += 1;
                        }
                    }
                    return bytes
                        .get(start..j.min(bytes.len()))
                        .and_then(|token| serde_json::from_slice::<Value>(token).ok())
                        .filter(|id| id.is_string() || id.is_number())
                        .unwrap_or(Value::Null);
                }
                continue;
            }
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth = depth.saturating_sub(1),
            _ => {}
        }
        i += 1;
    }
    Value::Null
}

/// Decides what to do with one complete input line (without its newline).
/// A line with more than `max_json_values` JSON values is refused before it
/// is parsed, as the HTTP API refuses such a body (the parsed value takes
/// memory per value, not per byte).
fn classify_line(line: &[u8], max_json_values: usize) -> LineAction {
    let text = line.trim_ascii();
    if text.is_empty() {
        return LineAction::Skip;
    }
    if nta8800_service::http::json_value_count(text, max_json_values).is_none() {
        let mut reply: Value = serde_json::from_str(&error_line(
            top_level_id(text),
            INVALID_REQUEST,
            &format!("Invalid Request: the message has more than {max_json_values} JSON values"),
        ))
        .unwrap_or(Value::Null);
        reply["error"]["data"] =
            json!({ "code": "payload_too_large", "maxJsonValues": max_json_values });
        return LineAction::Reply(reply.to_string());
    }
    let message: Value = match serde_json::from_slice(text) {
        Ok(message) => message,
        Err(error) => {
            let id = top_level_id(text);
            return if error.to_string().contains("recursion limit") {
                LineAction::Reply(error_line(
                    id,
                    INVALID_REQUEST,
                    "Invalid Request: the message is nested more than 128 levels deep",
                ))
            } else {
                LineAction::Reply(error_line(
                    id,
                    PARSE_ERROR,
                    &format!("Parse error: {error}"),
                ))
            };
        }
    };
    let Some(object) = message.as_object() else {
        return LineAction::Forward;
    };
    // A notification (no id) is never answered, also when it is invalid.
    let Some(id) = object.get("id").cloned() else {
        return LineAction::Forward;
    };
    if object.get("method").and_then(Value::as_str) == Some("tools/call") {
        match object.get("params") {
            Some(Value::Object(params)) => match params.get("arguments") {
                None | Some(Value::Null) | Some(Value::Object(_)) => {}
                Some(_) => {
                    return LineAction::Reply(error_line(
                        id,
                        INVALID_PARAMS,
                        "Invalid params: `arguments` must be an object",
                    ))
                }
            },
            _ => {
                return LineAction::Reply(error_line(
                    id,
                    INVALID_PARAMS,
                    "Invalid params: tools/call needs a `params` object with `name`",
                ))
            }
        }
    }
    LineAction::Forward
}

/// The message for a line longer than the cap.
fn oversized_line(max_line_bytes: usize) -> String {
    error_line(
        Value::Null,
        INVALID_REQUEST,
        &format!("Invalid Request: the message is longer than {max_line_bytes} bytes"),
    )
}

/// The guard's caps on one input line.
#[derive(Debug, Clone, Copy)]
struct LineLimits {
    max_line_bytes: usize,
    max_json_values: usize,
}

/// Reads stdin line by line, at most `max_line_bytes` per line, answers the
/// lines [`classify_line`] refuses through `replies` and forwards the others
/// to the MCP library.
async fn guard_input(
    input: impl tokio::io::AsyncRead + Unpin,
    mut to_server: impl tokio::io::AsyncWrite + Unpin,
    replies: tokio::sync::mpsc::UnboundedSender<String>,
    limits: LineLimits,
) -> std::io::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let LineLimits {
        max_line_bytes,
        max_json_values,
    } = limits;
    let mut reader = BufReader::new(input);
    let mut line: Vec<u8> = Vec::new();
    let mut oversized = false;
    loop {
        let buffer = reader.fill_buf().await?;
        if buffer.is_empty() {
            break;
        }
        let (chunk, complete) = match buffer.iter().position(|&byte| byte == b'\n') {
            Some(at) => (&buffer[..at], at + 1),
            None => (buffer, buffer.len()),
        };
        let ends_line = complete > chunk.len();
        if !oversized {
            if line.len() + chunk.len() > max_line_bytes {
                oversized = true;
                line = Vec::new();
            } else {
                line.extend_from_slice(chunk);
            }
        }
        reader.consume(complete);
        if ends_line {
            let action = if oversized {
                LineAction::Reply(oversized_line(max_line_bytes))
            } else {
                classify_line(&line, max_json_values)
            };
            match action {
                LineAction::Forward => {
                    to_server.write_all(&line).await?;
                    to_server.write_all(b"\n").await?;
                    to_server.flush().await?;
                }
                LineAction::Reply(reply) => {
                    let _ = replies.send(reply);
                }
                LineAction::Skip => {}
            }
            line.clear();
            oversized = false;
        }
    }
    // A last line without a newline.
    if oversized {
        let _ = replies.send(oversized_line(max_line_bytes));
    } else if !line.is_empty() {
        match classify_line(&line, max_json_values) {
            LineAction::Forward => {
                to_server.write_all(&line).await?;
                to_server.write_all(b"\n").await?;
            }
            LineAction::Reply(reply) => {
                let _ = replies.send(reply);
            }
            LineAction::Skip => {}
        }
    }
    to_server.shutdown().await
}

/// Writes the MCP library's output lines and the guard's error lines to
/// stdout, one whole line at a time.
async fn merge_output(
    from_server: impl tokio::io::AsyncRead + Unpin,
    mut replies: tokio::sync::mpsc::UnboundedReceiver<String>,
    mut output: impl tokio::io::AsyncWrite + Unpin,
) -> std::io::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let mut lines = BufReader::new(from_server).lines();
    let mut server_open = true;
    let mut replies_open = true;
    while server_open || replies_open {
        tokio::select! {
            line = lines.next_line(), if server_open => match line? {
                Some(line) => {
                    output.write_all(line.as_bytes()).await?;
                    output.write_all(b"\n").await?;
                    output.flush().await?;
                }
                None => server_open = false,
            },
            reply = replies.recv(), if replies_open => match reply {
                Some(reply) => {
                    output.write_all(reply.as_bytes()).await?;
                    output.write_all(b"\n").await?;
                    output.flush().await?;
                }
                None => replies_open = false,
            },
        }
    }
    output.flush().await
}

fn max_line_bytes() -> usize {
    std::env::var("OES_MCP_MAX_LINE_BYTES")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|bytes| *bytes >= 1024)
        .unwrap_or(DEFAULT_MAX_LINE_BYTES)
}

/// Cap on the JSON values of one message: the HTTP API's default
/// (`--max-json-values`), or `OES_MCP_MAX_JSON_VALUES` (at least 1000).
fn max_json_values() -> usize {
    std::env::var("OES_MCP_MAX_JSON_VALUES")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|values| *values >= 1000)
        .unwrap_or(nta8800_service::http::DEFAULT_MAX_JSON_VALUES)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "--version") {
        println!("{}", nta8800_service::operations::version_value());
        return Ok(());
    }
    let (stdin, stdout) = stdio();
    // stdin → guard → (guard_end ⇄ server_in) → MCP library → (server_out ⇄
    // merge_end) → merge → stdout, with the guard's replies merged in.
    let (guard_end, server_in) = tokio::io::duplex(1 << 16);
    let (server_out, merge_end) = tokio::io::duplex(1 << 16);
    let (replies, replies_rx) = tokio::sync::mpsc::unbounded_channel();
    let limits = LineLimits {
        max_line_bytes: max_line_bytes(),
        max_json_values: max_json_values(),
    };
    let guard = tokio::spawn(guard_input(stdin, guard_end, replies, limits));
    let merge = tokio::spawn(merge_output(merge_end, replies_rx, stdout));
    let server = EnergyMcp {
        slots: Arc::new(tokio::sync::Semaphore::new(max_calculations())),
    };
    let service = server.serve((server_in, server_out)).await?;
    service.waiting().await?;
    guard.abort();
    let _ = merge.await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operation_is_a_tool_with_an_object_schema() {
        let tools = tools();
        assert_eq!(tools.len(), operations().len());
        for tool in &tools {
            assert_eq!(
                tool.input_schema.get("type"),
                Some(&json!("object")),
                "{}",
                tool.name
            );
        }
    }

    #[test]
    fn every_tool_says_it_only_calculates() {
        for tool in tools() {
            let annotations = tool.annotations.as_ref().expect("annotations");
            assert_eq!(annotations.read_only_hint, Some(true), "{}", tool.name);
            assert_eq!(annotations.destructive_hint, Some(false), "{}", tool.name);
            assert_eq!(annotations.idempotent_hint, Some(true), "{}", tool.name);
            assert_eq!(annotations.open_world_hint, Some(false), "{}", tool.name);
        }
    }

    #[test]
    fn every_resource_reads() {
        for resource in RESOURCES {
            let content = (resource.content)();
            assert!(!content.is_empty(), "{}", resource.uri);
            if resource.mime == "application/json" {
                serde_json::from_str::<Value>(&content).expect(resource.uri);
            }
        }
    }

    fn reply(action: LineAction) -> Value {
        match action {
            LineAction::Reply(line) => serde_json::from_str(&line).unwrap(),
            other => panic!("expected a reply, got {other:?}"),
        }
    }

    fn classify(line: &[u8]) -> LineAction {
        classify_line(line, nta8800_service::http::DEFAULT_MAX_JSON_VALUES)
    }

    #[test]
    fn guard_refuses_more_json_values_than_the_http_cap() {
        let arguments = format!("[{}]", vec!["0"; 2000].join(","));
        let line = format!(
            r#"{{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{{"name":"validate_project","arguments":{{"project":{{"x":{arguments}}}}}}}}}"#
        );
        let refused = reply(classify_line(line.as_bytes(), 1000));
        assert_eq!(refused["id"], 21);
        assert_eq!(refused["error"]["code"], INVALID_REQUEST);
        assert_eq!(refused["error"]["data"]["code"], "payload_too_large");
        assert_eq!(refused["error"]["data"]["maxJsonValues"], 1000);
        assert_eq!(classify_line(line.as_bytes(), 5000), LineAction::Forward);
    }

    #[test]
    fn a_list_result_is_wrapped_as_an_object() {
        // MCP structured content is an object; list_interpretations answers
        // a list over HTTP.
        let result = call("list_interpretations", None);
        assert_eq!(result.is_error, Some(false));
        let body = result.structured_content.unwrap();
        assert!(body["result"]
            .as_array()
            .is_some_and(|groups| !groups.is_empty()));
        for op in operations() {
            if op.method == Method::Get {
                let result = call(op.name, None);
                assert!(
                    result.structured_content.unwrap().is_object(),
                    "{}",
                    op.name
                );
            }
        }
    }

    #[test]
    fn a_tool_without_input_refuses_arguments() {
        // Review 38: the schema of a GET tool allows no members
        // (additionalProperties: false); an argument is refused, as an
        // unknown member is on the POST tools.
        for op in operations() {
            if op.method != Method::Get {
                continue;
            }
            let mut arguments = Map::new();
            arguments.insert("normVersion".to_string(), json!("2024"));
            let result = call(op.name, Some(arguments));
            assert_eq!(result.is_error, Some(true), "{}", op.name);
            let body = result.structured_content.unwrap();
            assert_eq!(body["error"], "invalid_request_shape", "{}", op.name);
            assert!(body["message"]
                .as_str()
                .is_some_and(|message| message.contains("takes no members")));
            assert_eq!(call(op.name, Some(Map::new())).is_error, Some(false));
        }
    }

    #[test]
    fn guard_answers_lines_the_library_would_leave_unanswered() {
        // Invalid and truncated JSON: parse error, with the id when readable.
        let bad = reply(classify(b"{bad json"));
        assert_eq!(bad["error"]["code"], PARSE_ERROR);
        assert_eq!(bad["id"], Value::Null);
        let truncated = reply(classify(
            br#"{"jsonrpc":"2.0","id":11,"method":"tools/list""#,
        ));
        assert_eq!(truncated["error"]["code"], PARSE_ERROR);
        assert_eq!(truncated["id"], 11);
        // Deeper than serde's 128 levels: invalid request, with the id.
        let deep = format!(
            r#"{{"jsonrpc":"2.0","id":"x7","method":"tools/call","params":{{"name":"get_version","arguments":{{"a":{}{}}}}}}}"#,
            "[".repeat(200),
            "]".repeat(200)
        );
        let deep = reply(classify(deep.as_bytes()));
        assert_eq!(deep["error"]["code"], INVALID_REQUEST);
        assert_eq!(deep["id"], "x7");
        // tools/call with non-object arguments or params: invalid params.
        let arguments = reply(classify(
            br#"{"jsonrpc":"2.0","id":12,"method":"tools/call","params":{"name":"get_version","arguments":[1]}}"#,
        ));
        assert_eq!(arguments["error"]["code"], INVALID_PARAMS);
        assert_eq!(arguments["id"], 12);
        let params = reply(classify(
            br#"{"jsonrpc":"2.0","id":13,"method":"tools/call","params":[1]}"#,
        ));
        assert_eq!(params["error"]["code"], INVALID_PARAMS);
        // Valid messages, notifications and blank lines pass or are dropped.
        assert_eq!(
            classify(br#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#),
            LineAction::Forward
        );
        assert_eq!(
            classify(br#"{"jsonrpc":"2.0","method":"tools/call","params":[1]}"#),
            LineAction::Forward
        );
        assert_eq!(classify(b"   "), LineAction::Skip);
    }

    #[test]
    fn top_level_id_ignores_nested_ids_and_strings() {
        assert_eq!(top_level_id(br#"{"a":{"id":1},"id":2"#), json!(2));
        assert_eq!(top_level_id(br#"{"a":"\"id\":3","id":"q"}"#), json!("q"));
        assert_eq!(top_level_id(br#"{"id":{"x":1}}"#), Value::Null);
        assert_eq!(top_level_id(b"[1,2]"), Value::Null);
    }

    #[tokio::test]
    async fn guard_caps_the_line_length() {
        use tokio::io::AsyncReadExt;
        let mut input = vec![b'x'; 5000];
        input.push(b'\n');
        input.extend_from_slice(br#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#);
        input.push(b'\n');
        let (to_server, mut server_side) = tokio::io::duplex(1 << 16);
        let (replies, mut replies_rx) = tokio::sync::mpsc::unbounded_channel();
        let limits = LineLimits {
            max_line_bytes: 4096,
            max_json_values: nta8800_service::http::DEFAULT_MAX_JSON_VALUES,
        };
        guard_input(&input[..], to_server, replies, limits)
            .await
            .unwrap();
        let reply: Value = serde_json::from_str(&replies_rx.recv().await.unwrap()).unwrap();
        assert_eq!(reply["error"]["code"], INVALID_REQUEST);
        let mut forwarded = String::new();
        server_side.read_to_string(&mut forwarded).await.unwrap();
        assert_eq!(
            forwarded,
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n"
        );
    }

    #[test]
    fn unknown_tool_and_missing_argument_are_errors() {
        let result = call("nope", None);
        assert_eq!(result.is_error, Some(true));
        let result = call("calculate_project_performance", None);
        assert_eq!(result.is_error, Some(true));
        let body = result.structured_content.unwrap();
        assert_eq!(body["code"], "missing_request_member");
        assert_eq!(body["path"], "project");
    }
}
