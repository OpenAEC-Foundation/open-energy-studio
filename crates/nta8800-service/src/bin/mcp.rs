//! MCP server (stdio) for the Open Energy Studio NTA 8800 kernel.
//!
//! Every tool is an operation from `nta8800_service::operations`, the same
//! registry the HTTP API serves, so tool `x` and route `x` always agree.
//! Resources expose the example projects, the survey fixtures, the kernel's
//! interpretation list, the Dutch user manual and the OpenAPI document.

use std::sync::Arc;

use nta8800_service::operations::{error_body, operation, operations, Method};
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResponse, CallToolResult, Implementation,
        ListResourcesResult, ListToolsResult, PaginatedRequestParams, ReadResourceRequestParams,
        ReadResourceResponse, ReadResourceResult, Resource, ResourceContents, ServerCapabilities,
        ServerConfig, Tool,
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
            Tool::new(op.name, description, tool_schema(op))
        })
        .collect()
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
    let outcome = (op.run)(&body);
    if outcome.is_error() {
        CallToolResult::structured_error(outcome.body)
    } else {
        CallToolResult::structured(outcome.body)
    }
}

#[derive(Clone)]
struct EnergyMcp;

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
        let result = tokio::task::spawn_blocking(move || call(&name, arguments))
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().any(|arg| arg == "--version") {
        println!("{}", nta8800_service::operations::version_value());
        return Ok(());
    }
    let service = EnergyMcp.serve(stdio()).await?;
    service.waiting().await?;
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
    fn every_resource_reads() {
        for resource in RESOURCES {
            let content = (resource.content)();
            assert!(!content.is_empty(), "{}", resource.uri);
            if resource.mime == "application/json" {
                serde_json::from_str::<Value>(&content).expect(resource.uri);
            }
        }
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
