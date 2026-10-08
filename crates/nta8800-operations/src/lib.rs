//! One registry of kernel operations shared by the HTTP API, the MCP server,
//! the WebAssembly module, the OpenAPI document and the tests. Every operation
//! has exactly one HTTP route and one MCP tool with the same name, so the
//! adapters cannot drift. This crate depends only on the kernel and serde, so
//! it compiles for `wasm32-unknown-unknown` as well as for the servers.

use nta8800_core::norm_versions::{self, NormVersion};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

/// HTTP method of an operation's route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// The single top-level request member that carries an operation's input.
#[derive(Debug, Clone, Copy)]
pub struct InputSpec {
    pub key: &'static str,
    pub description: &'static str,
}

/// How a kernel status maps to an HTTP status code (and MCP `isError`).
#[derive(Debug, Clone, Copy)]
pub enum StatusRule {
    /// `status == "invalid"` is 422, everything else 200.
    Invalid,
    /// Only `calculated_unverified` (and `calculated_legacy_edition`, a
    /// calculation in an older edition) is 200, everything else 422.
    Calculated,
    /// Always 200.
    Always,
    /// `status == "invalid_case"` is 422.
    InvalidCase,
    /// `invalid_case` or `calculation_unavailable` is 422.
    ReferenceCompare,
}

impl StatusRule {
    pub fn status(self, body: &Value) -> u16 {
        let status = body.get("status").and_then(Value::as_str).unwrap_or("");
        let refused = match self {
            StatusRule::Invalid => status == "invalid",
            StatusRule::Calculated => {
                status != "calculated_unverified" && status != "calculated_legacy_edition"
            }
            StatusRule::Always => false,
            StatusRule::InvalidCase => status == "invalid_case",
            StatusRule::ReferenceCompare => {
                status == "invalid_case" || status == "calculation_unavailable"
            }
        };
        if refused {
            422
        } else {
            200
        }
    }
}

/// Result of running an operation: an HTTP status and a JSON body.
///
/// 200 is a success. 422 is a kernel assessment that refuses or cannot complete
/// the input; the body is then the assessment itself, with its `status`,
/// `gaps` and `issues`. 400 is a malformed request and 500 a withheld
/// non-finite or unserializable result; both use the [`error_body`] envelope,
/// as do the HTTP adapter's own 404, 405, 413 and 415 answers.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub status: u16,
    pub body: Value,
}

impl Outcome {
    /// Whether an MCP client should see this as a failed tool call.
    pub fn is_error(&self) -> bool {
        self.status >= 400
    }
}

/// One kernel capability, exposed as `METHOD path` and as MCP tool `name`.
pub struct Operation {
    pub name: &'static str,
    pub method: Method,
    pub path: &'static str,
    pub description: &'static str,
    pub input: Option<InputSpec>,
    /// Functional group, used as OpenAPI tag.
    pub category: &'static str,
    pub run: fn(&Value) -> Outcome,
}

/// The common error envelope. `error` and `code` carry the same machine code
/// (`error` is kept for older clients), `message` is human readable, `path`
/// is the JSON path of the offending input when known, `details` is optional.
pub fn error_body(
    code: &str,
    message: impl Into<String>,
    path: Option<String>,
    details: Value,
) -> Value {
    json!({
        "error": code,
        "code": code,
        "message": message.into(),
        "path": path,
        "details": details,
    })
}

/// Serializes a kernel result, withholding it if it contains a non-finite
/// number or cannot be represented as JSON.
pub fn finite_value<T: Serialize + ?Sized>(value: &T) -> Result<Value, Outcome> {
    match nta8800_core::finite::first_non_finite(value) {
        None => serde_json::to_value(value).map_err(|_| Outcome {
            status: 500,
            body: error_body(
                "serialization_failed",
                "The kernel result could not be serialized as JSON; the result is withheld",
                None,
                Value::Null,
            ),
        }),
        Some(path) => Err(Outcome {
            status: 500,
            body: error_body(
                "non_finite_result",
                "The kernel produced a non-finite number; the result is withheld",
                Some(path),
                Value::Null,
            ),
        }),
    }
}

fn missing_member(key: &str) -> Outcome {
    Outcome {
        status: 400,
        body: error_body(
            "missing_request_member",
            format!("The request body must be a JSON object with member `{key}`"),
            Some(key.to_string()),
            Value::Null,
        ),
    }
}

/// Deserializes `body[key]` into `T`, reporting the JSON path of a mismatch.
pub fn member<T: DeserializeOwned>(body: &Value, key: &str) -> Result<T, Outcome> {
    let value = body.get(key).ok_or_else(|| missing_member(key))?;
    serde_path_to_error::deserialize::<_, T>(value).map_err(|error| {
        let inner = error.path().to_string();
        let path = if inner == "." || inner.is_empty() {
            key.to_string()
        } else {
            format!("{key}.{inner}")
        };
        Outcome {
            status: 400,
            body: error_body(
                "invalid_request_shape",
                format!("`{path}`: {}", error.inner()),
                Some(path),
                Value::Null,
            ),
        }
    })
}

/// Runs a typed kernel function on `body[key]` and maps its status.
pub fn typed<T: DeserializeOwned, R: Serialize>(
    body: &Value,
    key: &str,
    run: impl FnOnce(T) -> R,
    rule: StatusRule,
) -> Outcome {
    let input = match member::<T>(body, key) {
        Ok(input) => input,
        Err(outcome) => return outcome,
    };
    let result = run(input);
    match finite_value(&result) {
        Ok(value) => Outcome {
            status: rule.status(&value),
            body: value,
        },
        Err(outcome) => outcome,
    }
}

fn ok(value: Value) -> Outcome {
    Outcome {
        status: 200,
        body: value,
    }
}

fn project_member(body: &Value) -> Result<Value, Outcome> {
    body.get("project")
        .cloned()
        .ok_or_else(|| missing_member("project"))
}

fn invalid_project_shape(message: String) -> Outcome {
    Outcome {
        status: 400,
        body: error_body(
            "invalid_project_shape",
            message,
            Some("project".into()),
            Value::Null,
        ),
    }
}

fn run_validate(body: &Value) -> Outcome {
    let project = match project_member(body) {
        Ok(project) => project,
        Err(outcome) => return outcome,
    };
    match nta8800_core::assess_json(project) {
        Ok(assessment) => match finite_value(&assessment) {
            Ok(value) => ok(value),
            Err(outcome) => outcome,
        },
        Err(message) => invalid_project_shape(message),
    }
}

fn run_calculate_beng(body: &Value) -> Outcome {
    let project = match project_member(body) {
        Ok(project) => project,
        Err(outcome) => return outcome,
    };
    match nta8800_core::assess_json(project) {
        Err(message) => invalid_project_shape(message),
        Ok(assessment) if assessment.status == "invalid" => Outcome {
            status: 422,
            body: json!({
                "error": "invalid_project_input",
                "code": "invalid_project_input",
                "assessment": assessment
            }),
        },
        Ok(_) => {
            let mut body = error_body(
                "calculation_unavailable",
                "The Rust NTA 8800 calculation is not validated or available yet; use calculate_project_performance",
                None,
                Value::Null,
            );
            body["capabilities"] = json!(nta8800_core::capabilities());
            Outcome { status: 501, body }
        }
    }
}

fn run_project_performance(body: &Value) -> Result<Value, Outcome> {
    let project = project_member(body)?;
    let assessment = nta8800_core::project_performance::assess_project_performance(&project);
    finite_value(&assessment)
}

fn run_calculate_project_performance(body: &Value) -> Outcome {
    match run_project_performance(body) {
        Ok(value) => Outcome {
            status: StatusRule::Calculated.status(&value),
            body: value,
        },
        Err(outcome) => outcome,
    }
}

/// The project assessment reduced to its identity and status fields plus
/// `members`, so a client can ask for one part without the rest.
fn project_projection(body: &Value, members: &[&str]) -> Outcome {
    match run_project_performance(body) {
        Err(outcome) => outcome,
        Ok(value) => {
            let mut out = serde_json::Map::new();
            for key in [
                "status",
                "kernelVersion",
                "targetNormVersion",
                "normVersion",
                "registrationEligible",
                "inputFingerprint",
                "attestStatus",
                "gaps",
                "warnings",
            ] {
                if let Some(item) = value.get(key) {
                    out.insert(key.to_string(), item.clone());
                }
            }
            for key in members {
                out.insert(
                    (*key).to_string(),
                    value.get(*key).cloned().unwrap_or(Value::Null),
                );
            }
            let value = Value::Object(out);
            Outcome {
                status: StatusRule::Calculated.status(&value),
                body: value,
            }
        }
    }
}

fn run_assess_registration(body: &Value) -> Outcome {
    project_projection(body, &["registration"])
}

fn run_label_data(body: &Value) -> Outcome {
    project_projection(body, &["labelData"])
}

fn run_energy_by_service(body: &Value) -> Outcome {
    match run_project_performance(body) {
        Err(outcome) => outcome,
        Ok(value) => {
            let energy = value
                .get("performance")
                .and_then(|performance| performance.get("energyByService"))
                .cloned()
                .unwrap_or(Value::Null);
            let out = json!({
                "status": value.get("status"),
                "kernelVersion": value.get("kernelVersion"),
                "inputFingerprint": value.get("inputFingerprint"),
                "gaps": value.get("gaps"),
                "energyByService": energy,
            });
            Outcome {
                status: StatusRule::Calculated.status(&out),
                body: out,
            }
        }
    }
}

fn run_label_input_hash(body: &Value) -> Outcome {
    match project_member(body) {
        Err(outcome) => outcome,
        Ok(project) => ok(json!({
            "labelInputSha256": nta8800_core::relabel::label_input_hash(&project),
            "kernelVersion": nta8800_core::KERNEL_VERSION,
        })),
    }
}

fn run_relabel(body: &Value) -> Outcome {
    let (Some(original), Some(current)) = (body.get("original"), body.get("current")) else {
        let key = if body.get("original").is_none() {
            "original"
        } else {
            "current"
        };
        return missing_member(key);
    };
    match finite_value(&nta8800_core::relabel::assess_relabel(original, current)) {
        Ok(value) => ok(value),
        Err(outcome) => outcome,
    }
}

fn run_maatwerkadvies(body: &Value) -> Outcome {
    let Some(input) = body.get("input").cloned() else {
        return missing_member("input");
    };
    match nta8800_core::maatwerkadvies::assess_maatwerkadvies_json(input) {
        Ok(assessment) => match finite_value(&assessment) {
            Ok(value) => Outcome {
                status: StatusRule::Invalid.status(&value),
                body: value,
            },
            Err(outcome) => outcome,
        },
        Err(message) => Outcome {
            status: 400,
            body: error_body(
                "invalid_maatwerkadvies_shape",
                message,
                Some("input".into()),
                Value::Null,
            ),
        },
    }
}

// ---------------------------------------------------------------- editions

/// Request member that selects the NTA 8800 edition of any operation.
pub const NORM_VERSION_MEMBER: &str = norm_versions::request::NORM_VERSION_MEMBER;

/// How an operation takes the edition of a request.
enum EditionRoute {
    /// The input carries its own edition at these (member, JSON pointer)
    /// places; the request's edition is written there when absent and must
    /// match when present.
    Slots(Vec<(&'static str, &'static str)>),
    /// The kernel function runs with the edition active (constructions and
    /// diagnostic routes).
    Active,
    /// The result does not depend on the edition (label-input hash).
    Independent,
    /// Reference cases are compared in 2025+C1:2026 only.
    Fixed,
}

fn edition_route(op: &Operation, body: &Value) -> EditionRoute {
    const PROJECT: &str = "/ntaCalculation/normVersion";
    match op.name {
        "validate_project"
        | "calculate_beng"
        | "calculate_project_performance"
        | "get_energy_by_service"
        | "get_label_data"
        | "assess_registration" => EditionRoute::Slots(vec![("project", PROJECT)]),
        "assess_residential_survey" | "assess_utility_survey" => {
            EditionRoute::Slots(vec![("survey", "/normVersion")])
        }
        "calculate_building_performance" => EditionRoute::Slots(vec![("input", "/normVersion")]),
        // Every variant is calculated in the base situation's edition.
        "assess_maatwerkadvies" => EditionRoute::Slots(vec![(
            "input",
            match body.pointer("/input/base/kind").and_then(Value::as_str) {
                Some("building") => "/base/input/normVersion",
                _ => "/base/project/ntaCalculation/normVersion",
            },
        )]),
        // A relabel stays in the original's edition (BRL 9500-W §4.2.4).
        "assess_relabel" => EditionRoute::Slots(vec![("original", PROJECT), ("current", PROJECT)]),
        "get_label_input_hash" => EditionRoute::Independent,
        "audit_reference_case"
        | "compare_reference_case"
        | "compare_direct_diagnostic"
        | "compare_gas_heat_pump_chain_diagnostic" => EditionRoute::Fixed,
        _ => EditionRoute::Active,
    }
}

fn edition_error(code: &str, message: String, path: String) -> Outcome {
    let supported: Vec<&str> = NormVersion::ALL
        .iter()
        .map(|version| version.id())
        .collect();
    Outcome {
        status: 400,
        body: error_body(
            code,
            message,
            Some(path),
            json!({ "supportedNormVersions": supported }),
        ),
    }
}

/// Refusal of a request's edition, as an error envelope.
fn edition_refusal(error: norm_versions::request::EditionError) -> Outcome {
    edition_error(error.code, error.message, error.path)
}

/// The request's `normVersion`, if any.
fn requested_norm_version(body: &Value) -> Result<Option<NormVersion>, Outcome> {
    norm_versions::request::parse_requested(body.get(NORM_VERSION_MEMBER)).map_err(edition_refusal)
}

/// Writes `version` into the input's own edition slots, refusing a request
/// that contradicts the input.
fn place_edition(
    slots: &[(&'static str, &'static str)],
    body: &mut Value,
    version: NormVersion,
) -> Result<(), Outcome> {
    for (member, pointer) in slots {
        if let Some(input) = body.get_mut(*member) {
            norm_versions::request::place_in(input, member, pointer, version)
                .map_err(edition_refusal)?;
        }
    }
    Ok(())
}

/// Records the edition a result was calculated with (see
/// [`norm_versions::request::stamp`]).
fn stamp_edition(outcome: &mut Outcome, version: NormVersion) {
    norm_versions::request::stamp(&mut outcome.body, version);
}

/// Runs an operation as the HTTP and MCP adapters do: with the request's
/// optional `normVersion` applied (written into the input's own edition, or
/// active while the kernel runs) and the edition stamped on the result.
pub fn execute(op: &Operation, body: &Value) -> Outcome {
    if op.method == Method::Get {
        return (op.run)(body);
    }
    let requested = match requested_norm_version(body) {
        Ok(requested) => requested,
        Err(outcome) => return outcome,
    };
    let mut version = requested.unwrap_or_default();
    let mut body = body.clone();
    match edition_route(op, &body) {
        EditionRoute::Slots(slots) => {
            if let Some(version) = requested {
                if let Err(outcome) = place_edition(&slots, &mut body, version) {
                    return outcome;
                }
            }
            // The input's own edition (for a relabel the original's) is the
            // one the kernel applies.
            version = slots
                .first()
                .and_then(|(member, pointer)| body.get(*member)?.pointer(pointer))
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or_default();
        }
        EditionRoute::Active if !version.implemented() => {
            return Outcome {
                status: 422,
                body: error_body(
                    "edition_not_implemented",
                    format!("The kernel has no profile for {}", version.label()),
                    Some(NORM_VERSION_MEMBER.into()),
                    Value::Null,
                ),
            };
        }
        EditionRoute::Fixed if !version.is_default() => {
            return edition_error(
                "norm_version_not_applicable",
                "Reference cases are compared in NTA 8800:2025+C1:2026 only".into(),
                NORM_VERSION_MEMBER.into(),
            );
        }
        EditionRoute::Active | EditionRoute::Independent | EditionRoute::Fixed => {}
    }
    let mut outcome = norm_versions::with_version(version, || (op.run)(&body));
    // The legacy status is a success under every status rule, so stamping
    // never changes the HTTP status.
    if outcome.status == 200 || outcome.status == 422 {
        stamp_edition(&mut outcome, version);
    }
    outcome
}

/// Kernel, norm and service identity of this build.
pub fn version_value() -> Value {
    json!({
        "service": "open-energy-studio-nta8800",
        "serviceVersion": env!("CARGO_PKG_VERSION"),
        "apiVersion": "v1",
        "kernelVersion": nta8800_core::KERNEL_VERSION,
        "targetNormVersion": nta8800_core::TARGET_NORM_VERSION,
        "supportedNormVersions": nta8800_core::norm_versions::supported_editions(),
        "buildCommit": option_env!("OES_BUILD_COMMIT"),
        "buildFingerprint": build_fingerprint(),
    })
}

/// SHA-256 over the identity fields and the operation table: two builds with
/// the same fingerprint expose the same kernel version and the same surface.
pub fn build_fingerprint() -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(nta8800_core::KERNEL_VERSION.as_bytes());
    hasher.update(nta8800_core::TARGET_NORM_VERSION.as_bytes());
    hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
    hasher.update(option_env!("OES_BUILD_COMMIT").unwrap_or("").as_bytes());
    for op in operations() {
        hasher.update(op.name.as_bytes());
        hasher.update(op.path.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

const PROJECT_DOC: &str = "Open Energy Studio project object: the `project` member of a saved .oes.json file (without the outer metadata wrapper), including its `ntaCalculation` block. Null object members may be left out.";

/// Every kernel operation, in a stable order.
pub fn operations() -> &'static [Operation] {
    OPERATIONS
}

/// Looks up an operation by its MCP tool name.
pub fn operation(name: &str) -> Option<&'static Operation> {
    OPERATIONS.iter().find(|op| op.name == name)
}

static OPERATIONS: &[Operation] = &[
    Operation {
        name: "get_version",
        method: Method::Get,
        path: "/v1/version",
        description: "Report the service, kernel and target norm version (NTA 8800:2025+C1:2026), the API version and a build fingerprint",
        input: None,
        category: "service",
        run: |_| ok(version_value()),
    },
    Operation {
        name: "get_capabilities",
        method: Method::Get,
        path: "/v1/nta8800/capabilities",
        description: "Show the Rust NTA 8800 kernel scope, target version and attest status",
        input: None,
        category: "service",
        run: |_| ok(json!(nta8800_core::capabilities())),
    },
    Operation {
        name: "list_interpretations",
        method: Method::Get,
        path: "/v1/nta8800/interpretations",
        description: "List the kernel's documented readings of ambiguous or contradictory NTA 8800 passages (interpretatielijst), grouped per module; the calculation report prints the same list as appendix",
        input: None,
        category: "service",
        run: |_| ok(json!(nta8800_core::interpretations::kernel_interpretations())),
    },
    Operation {
        name: "validate_project",
        method: Method::Post,
        path: "/v1/nta8800/validate",
        description: "Validate an Open Energy Studio project structure; does not calculate an energy label",
        input: Some(InputSpec { key: "project", description: PROJECT_DOC }),
        category: "project",
        run: run_validate,
    },
    Operation {
        name: "calculate_beng",
        method: Method::Post,
        path: "/v1/nta8800/calculate",
        description: "Legacy endpoint: validates the project and answers 501 calculation_unavailable. Use calculate_project_performance for BENG 1/2/3, TOjuli and the indicative label",
        input: Some(InputSpec { key: "project", description: PROJECT_DOC }),
        category: "project",
        run: run_calculate_beng,
    },
    Operation {
        name: "calculate_project_performance",
        method: Method::Post,
        path: "/v1/nta8800/project/performance",
        description: "Calculate a saved .oes project with its ntaCalculation block per NTA 8800:2025+C1:2026: BENG 1/2/3 (energiebehoefte, primair fossiel energiegebruik, aandeel hernieuwbare energie), TOjuli, Bbl check, indicative label class, label data (Omgevingsregeling art. 5.13/5.13a), energy per service and carrier, registration readiness and warnings. Returns status incomplete with input gaps (gaten) when data is missing. Unverified: no registered label and no attest",
        input: Some(InputSpec { key: "project", description: PROJECT_DOC }),
        category: "project",
        run: run_calculate_project_performance,
    },
    Operation {
        name: "get_energy_by_service",
        method: Method::Post,
        path: "/v1/nta8800/project/energy-by-service",
        description: "Return delivered, primary fossil and renewable energy per service (verwarming, tapwater, koeling, bevochtiging, ventilatoren, verlichting, hulpenergie), carrier and month (NTA 8800 §5.5.3); a part of calculate_project_performance",
        input: Some(InputSpec { key: "project", description: PROJECT_DOC }),
        category: "project",
        run: run_energy_by_service,
    },
    Operation {
        name: "get_label_data",
        method: Method::Post,
        path: "/v1/nta8800/label/data",
        description: "Return the label data (labelgegevens) of a project per Omgevingsregeling art. 5.13 and 5.13a: general data, envelope and installation summary and indicators; a part of calculate_project_performance",
        input: Some(InputSpec { key: "project", description: PROJECT_DOC }),
        category: "registration",
        run: run_label_data,
    },
    Operation {
        name: "assess_registration",
        method: Method::Post,
        path: "/v1/nta8800/registration/assess",
        description: "Check a project's registration block for EP-Online registration per BRL 9500-W/U (29-05-2026) and the Omgevingsregeling: dossier completeness, deadlines and validity, message type (regulier, herlabelen, vervanging), relabel re-check, plausibility and readiness (gereed voor registratie); a part of calculate_project_performance",
        input: Some(InputSpec { key: "project", description: PROJECT_DOC }),
        category: "registration",
        run: run_assess_registration,
    },
    Operation {
        name: "assess_maatwerkadvies",
        method: Method::Post,
        path: "/v1/nta8800/maatwerkadvies",
        description: "Calculate a maatwerkadvies (BRL 9500-MWA, ISSO 82.2/75.2): base situation, measures as JSON patches on the project, packages, tariffs, usage fit, net present value, payback time and renovation passport. Measure templates (isolatie, beglazing, warmtepomp, PV, ...) are generated client-side into these patches",
        input: Some(InputSpec { key: "input", description: "Maatwerkadvies input: the base project or building input, measures (each with a JSON patch, investment, cost source and lifetime), packages, tariffs and optional usage profile and fit. Null members may be left out." }),
        category: "maatwerkadvies",
        run: run_maatwerkadvies,
    },
    Operation {
        name: "assess_relabel",
        method: Method::Post,
        path: "/v1/nta8800/relabel/assess",
        description: "Compare an original and a current project for herlabelen per BRL 9500-W/U Bijlage 6a/6b: classify each change as allowed (6a), needs review or not allowed (6b), with clusters and the canonical label-input hashes of both projects",
        input: Some(InputSpec { key: "original", description: "The originally registered project; the request also needs member `current` with the current project. Both are Open Energy Studio project objects." }),
        category: "relabel",
        run: run_relabel,
    },
    Operation {
        name: "get_label_input_hash",
        method: Method::Post,
        path: "/v1/nta8800/relabel/label-input-hash",
        description: "Compute the canonical SHA-256 of a project's label input (sorted keys, nulls removed, numbers normalised; registration, maatwerkadvies, basisopname and import log excluded), as stored with a relabel comparison",
        input: Some(InputSpec { key: "project", description: PROJECT_DOC }),
        category: "relabel",
        run: run_label_input_hash,
    },
    Operation {
        name: "diagnose_direct_transmission",
        method: Method::Post,
        path: "/v1/nta8800/transmission/direct/diagnose",
        description: "Diagnose the direct-to-outdoor A·U + L·psi + chi sum; does not produce NTA BENG or a verified label",
        input: Some(InputSpec { key: "input", description: "Explicit outdoor elements and thermal bridges with units and property references." }),
        category: "transmission",
        run: |body| typed::<nta8800_core::direct_transmission::DirectTransmissionInput, _>(body, "input", |input| nta8800_core::direct_transmission::assess_direct_transmission(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_monthly_direct",
        method: Method::Post,
        path: "/v1/nta8800/transmission/direct/monthly-diagnose",
        description: "Diagnose signed monthly direct-to-outdoor heat flow from supplied conductance, temperatures and hours; not NTA demand or BENG",
        input: Some(InputSpec { key: "input", description: "Explicit direct-to-outdoor conductance components and twelve supplied monthly temperature/hour records." }),
        category: "transmission",
        run: |body| typed::<nta8800_core::monthly_direct_transmission::MonthlyDirectInput, _>(body, "input", |input| nta8800_core::monthly_direct_transmission::assess_monthly_direct(&input), StatusRule::Invalid),
    },
    Operation {
        name: "calculate_monthly_demand",
        method: Method::Post,
        path: "/v1/nta8800/demand/monthly/calculate",
        description: "Calculate the unverified NTA 8800 chapter 7 monthly heating and cooling need of one zone with De Bilt climate; lists omitted corrections; no BENG or label",
        input: Some(InputSpec { key: "input", description: "One calculation zone: floor area, usage function (tables 7.13–7.15) and dwelling type, setpoints, transmission (explicit or chapter 8 components with annex D ground), ventilation conductances per balance, thermal-mass classes, internal-gain method, windows and opaque elements with source references." }),
        category: "demand",
        run: |body| typed::<nta8800_core::monthly_demand::MonthlyDemandInput, _>(body, "input", |input| nta8800_core::monthly_demand::assess_monthly_demand(&input), StatusRule::Invalid),
    },
    Operation {
        name: "calculate_space_heating_chain",
        method: Method::Post,
        path: "/v1/nta8800/heating/space-heating-chain/calculate",
        description: "Calculate the unverified monthly space-heating chain (need, emission, distribution, one generator) and energy per carrier; lists omitted terms; no BENG or label",
        input: Some(InputSpec { key: "input", description: "Monthly demand input plus emission system, distribution route and one generator (gas boiler or forfait heat pump)." }),
        category: "heating",
        run: |body| typed::<nta8800_core::space_heating_chain::SpaceHeatingChainInput, _>(body, "input", |input| nta8800_core::space_heating_chain::assess_space_heating_chain(&input), StatusRule::Invalid),
    },
    Operation {
        name: "calculate_ventilation",
        method: Method::Post,
        path: "/v1/nta8800/ventilation/calculate",
        description: "Calculate unverified NTA 8800 chapter 11 ventilation for one zone: required and effective flows from the pressure balance, supply temperatures, chapter 7 conductances per balance, fan and frost-protection electricity, and the fixed C1 run for BENG 1",
        input: Some(InputSpec { key: "input", description: "One zone: use functions, height, system variant (table 11.5), heat recovery, infiltration, combustion appliances, ventilative cooling openings and fans, with source references." }),
        category: "ventilation",
        run: |body| typed::<nta8800_core::ventilation::VentilationInput, _>(body, "input", |input| nta8800_core::ventilation::assess_ventilation(&input), StatusRule::Invalid),
    },
    Operation {
        name: "assess_residential_survey",
        method: Method::Post,
        path: "/v1/nta8800/opname/residential",
        description: "Translate an ISSO 82.1 basic survey (basisopname) of an existing dwelling into NTA 8800 kernel input, list every applied default with its ISSO page, and calculate the unverified building performance and indicative label",
        input: Some(InputSpec { key: "survey", description: "ISSO 82.1 basic survey of one existing dwelling: construction year, dwelling type, envelope surfaces with insulation answers, glazing, heating, hot water, ventilation and PV, with \"unknown\" options." }),
        category: "opname",
        run: |body| typed::<nta8800_core::opname::ResidentialSurvey, _>(body, "survey", |input| nta8800_core::opname::assess_residential_survey(&input), StatusRule::Calculated),
    },
    Operation {
        name: "assess_utility_survey",
        method: Method::Post,
        path: "/v1/nta8800/opname/utility",
        description: "Translate an ISSO 75.1 basic survey (basisopname) of an existing utility building into NTA 8800 kernel input (one calculation zone; other functions up to 25 % merged), list every applied default with its ISSO page, and calculate the unverified building performance and indicative label",
        input: Some(InputSpec { key: "survey", description: "ISSO 75.1 basic survey of one existing utility building: use functions with areas, building type, envelope, heating installation, cooling, ventilation with AHU, recirculation and flow control, humidification, hot water, lighting zones, PV and BACS, with \"unknown\" options." }),
        category: "opname",
        run: |body| typed::<nta8800_core::opname::utility::UtilitySurvey, _>(body, "survey", |input| nta8800_core::opname::utility::assess_utility_survey(&input), StatusRule::Calculated),
    },
    Operation {
        name: "calculate_constructions",
        method: Method::Post,
        path: "/v1/nta8800/constructions/calculate",
        description: "Calculate unverified NTA 8800 8.2 envelope element U- and Rc-values (annexes C, E-I, L): layered and composite constructions with dU corrections, tapered roofs, windows and doors, forfait values for existing buildings, forfait psi and the dU_for supplement",
        input: Some(InputSpec { key: "input", description: "Envelope elements (layered opaque constructions, tapered roofs, windows/doors with optional shutters, forfait existing-building values, rooflights, grilles, numerical U) plus forfait thermal bridges, with source references." }),
        category: "constructions",
        run: |body| typed::<nta8800_core::envelope_elements::EnvelopeInput, _>(body, "input", |input| nta8800_core::envelope_elements::assess_envelope(&input), StatusRule::Invalid),
    },
    Operation {
        name: "calculate_building_performance",
        method: Method::Post,
        path: "/v1/nta8800/performance/calculate",
        description: "Calculate unverified building energy indicators, BENG and an indicative label class when the supplied input is complete; no registered label or attest",
        input: Some(InputSpec { key: "input", description: "Space-heating chain, declared other services, on-site production, BACS factor and floor area." }),
        category: "performance",
        run: |body| typed::<nta8800_core::building_performance::BuildingPerformanceInput, _>(body, "input", |input| nta8800_core::building_performance::assess_building_performance(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_unheated_transmission",
        method: Method::Post,
        path: "/v1/nta8800/transmission/unheated/diagnose",
        description: "Diagnose conductance via unheated spaces using caller-supplied factors; not NTA demand or BENG",
        input: Some(InputSpec { key: "input", description: "Named unheated spaces, explicit boundary terms and caller-supplied reduction factors." }),
        category: "transmission",
        run: |body| typed::<nta8800_core::unheated_transmission::UnheatedTransmissionInput, _>(body, "input", |input| nta8800_core::unheated_transmission::assess_unheated_transmission(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_declared_heating_table",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/declared-heating-table/diagnose",
        description: "Interpolate a supplied space-heating product declaration table within its bounds; does not calculate annual building performance, BENG or a label",
        input: Some(InputSpec { key: "input", description: "User-supplied product declaration table excerpt and query within its bounds." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::declared_heating_table::DeclaredHeatingTableInput, _>(body, "input", |input| nta8800_core::declared_heating_table::assess_declared_heating_table(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_forfait_heat_pump_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/forfait-cop-draft/diagnose",
        description: "Look up a base electric heat-pump COP in public draft tables 9.27/9.29; applicability and final edition unverified, no annual performance or BENG",
        input: Some(InputSpec { key: "input", description: "Explicit scope, source class and supply temperature for consultation tables 9.27/9.29." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput, _>(body, "input", |input| nta8800_core::forfait_heat_pump_draft::assess_forfait_heat_pump_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_gas_heat_pump_forfait_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/gas-forfait-cop-draft/diagnose",
        description: "Diagnose a gas-engine or gas-absorption heat-pump COP from public draft tables 9.27/9.29; no gas input, auxiliaries, BENG or label",
        input: Some(InputSpec { key: "input", description: "Gas-engine or absorption pump, table-9.29 application, source, capacity and design-temperature evidence." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::gas_heat_pump_forfait_draft::GasHeatPumpForfaitDraftInput, _>(body, "input", |input| nta8800_core::gas_heat_pump_forfait_draft::assess_gas_heat_pump_forfait_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_gas_heat_pump_aux_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/gas-aux-draft/diagnose",
        description: "Diagnose gas heat-pump generator auxiliary electricity from draft 9.91/9.92; no gas input, source pump, BENG or label",
        input: Some(InputSpec { key: "input", description: "Gas generator, explicit auxiliary coefficients and twelve supplied month records." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::gas_heat_pump_aux_draft::GasHeatPumpAuxDraftInput, _>(body, "input", |input| nta8800_core::gas_heat_pump_aux_draft::assess_gas_heat_pump_aux_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_gas_heat_pump_monthly_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/gas-forfait-monthly-draft/diagnose",
        description: "Diagnose draft 9.62 monthly input terms for a gas-engine or absorption heat pump; no carrier allocation, gas use, BENG or label",
        input: Some(InputSpec { key: "input", description: "Gas-engine/absorption table selection and twelve supplied generator-output months." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::gas_heat_pump_monthly_draft::GasHeatPumpMonthlyDraftInput, _>(body, "input", |input| nta8800_core::gas_heat_pump_monthly_draft::assess_gas_heat_pump_monthly_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_gas_heat_pump_chain_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/gas-chain-draft/diagnose",
        description: "Link gas heat-pump draft 9.62 and 9.91/9.92 monthly terms by generator, heat and evidence; no carrier allocation, gas use, BENG or label",
        input: Some(InputSpec { key: "input", description: "Matching draft 9.62 and equipment-auxiliary inputs for one gas generator and twelve months." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::gas_heat_pump_chain_draft::GasHeatPumpChainDraftInput, _>(body, "input", |input| nta8800_core::gas_heat_pump_chain_draft::assess_gas_heat_pump_chain_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_gas_collective_source_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/gas-collective-source-draft/diagnose",
        description: "Diagnose the separate dh heat and draft fossil/renewable primary contribution of a collective gas heat-pump source using public consultation chapters 5 and 9; no gas allocation, BENG or label",
        input: Some(InputSpec { key: "input", description: "Linked gas heat-pump chain, collective-source temperature evidence and confirmed absence of a quality declaration." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::gas_collective_source_draft::GasCollectiveSourceDraftInput, _>(body, "input", |input| nta8800_core::gas_collective_source_draft::assess_gas_collective_source_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_forfait_heat_pump_monthly_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/forfait-monthly-draft/diagnose",
        description: "Diagnose draft equation 9.62 from supplied monthly heat and a draft COP; no dispatch, auxiliaries, BENG or label",
        input: Some(InputSpec { key: "input", description: "Draft table lookup plus twelve supplied generator-output months and source-system evidence." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::forfait_heat_pump_monthly_draft::ForfaitHeatPumpMonthlyDraftInput, _>(body, "input", |input| nta8800_core::forfait_heat_pump_monthly_draft::assess_forfait_heat_pump_monthly_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_generator_dispatch_draft",
        method: Method::Post,
        path: "/v1/nta8800/heating/generator-dispatch-draft/diagnose",
        description: "Diagnose new-build heating generator dispatch from public draft tables 9.1/9.23 and equations 9.2/9.3/9.56/9.60; supplied node demand and powers, no BENG or label",
        input: Some(InputSpec { key: "input", description: "New-build heating-node input, typed generators, rated thermal power and efficiency provenance." }),
        category: "heating",
        run: |body| typed::<nta8800_core::generator_dispatch_draft::GeneratorDispatchDraftInput, _>(body, "input", |input| nta8800_core::generator_dispatch_draft::assess_generator_dispatch_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_hybrid_heat_pump_monthly_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/hybrid-monthly-draft/diagnose",
        description: "Diagnose a new-build hybrid heat-pump chain: chapter-9 draft dispatch, heat-pump electricity, boiler gas and boiler auxiliary electricity; optional measured heat-pump auxiliaries use linked months; no BENG or label",
        input: Some(InputSpec { key: "input", description: "New-build generator dispatch plus a matching electric heat pump forfait table and source-system evidence." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::hybrid_heat_pump_monthly_draft::HybridHeatPumpMonthlyDraftInput, _>(body, "input", |input| nta8800_core::hybrid_heat_pump_monthly_draft::assess_hybrid_heat_pump_monthly_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_boiler_forfait_draft",
        method: Method::Post,
        path: "/v1/nta8800/boilers/forfait-draft/diagnose",
        description: "Diagnose draft gas water-boiler efficiency from consultation table 9.25; no pilot flame, auxiliaries, BENG or label",
        input: Some(InputSpec { key: "input", description: "Gas water-boiler class, placement and emission-circuit evidence for draft table 9.25." }),
        category: "boilers",
        run: |body| typed::<nta8800_core::boiler_forfait_draft::BoilerForfaitDraftInput, _>(body, "input", |input| nta8800_core::boiler_forfait_draft::assess_boiler_forfait_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_boiler_forfait_monthly_draft",
        method: Method::Post,
        path: "/v1/nta8800/boilers/forfait-monthly-draft/diagnose",
        description: "Diagnose draft gas input and individual-boiler forfait auxiliary electricity with equations 9.61/9.85 from supplied monthly boiler heat; no pilot flame, BENG or label",
        input: Some(InputSpec { key: "input", description: "Draft table-9.25 boiler selection and twelve supplied generator-output months." }),
        category: "boilers",
        run: |body| typed::<nta8800_core::boiler_forfait_draft::BoilerForfaitMonthlyDraftInput, _>(body, "input", |input| nta8800_core::boiler_forfait_draft::assess_boiler_forfait_monthly_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_declared_dhw",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/declared-dhw/diagnose",
        description: "Check declared tap-profile test energies and report their raw ratios; never produces NTA practice efficiency, annual BENG or a label",
        input: Some(InputSpec { key: "input", description: "Classified heat pump with controlled declaration and EN 16147 tap-profile inputs." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::heat_pumps::HeatPumpInput, _>(body, "input", |input| nta8800_core::declared_dhw::assess_declared_dhw(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_final_energy_draft",
        method: Method::Post,
        path: "/v1/nta8800/energy/final-draft/diagnose",
        description: "Sum supplied monthly carrier energy and solar thermal contributions using public chapter-5 draft arithmetic; final norm edition and EDR unverified, no BENG or label",
        input: Some(InputSpec { key: "input", description: "Complete monthly E_EPus per carrier and optional solar thermal practice contributions." }),
        category: "energy",
        run: |body| typed::<nta8800_core::final_energy_draft::FinalEnergyDraftInput, _>(body, "input", |input| nta8800_core::final_energy_draft::assess_final_energy_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_epus_draft",
        method: Method::Post,
        path: "/v1/nta8800/energy/epus-draft/diagnose",
        description: "Compose monthly E_EPus per carrier from supplied service terms and BACS factor using public draft equations 5.20–5.21, then provisionally sum final energy; no verified NTA result or label",
        input: Some(InputSpec { key: "input", description: "Monthly service terms by carrier and BACS factor from the public chapter-5 draft." }),
        category: "energy",
        run: |body| typed::<nta8800_core::epus_draft::EpusDraftInput, _>(body, "input", |input| nta8800_core::epus_draft::assess_epus_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_bacs_draft",
        method: Method::Post,
        path: "/v1/nta8800/energy/bacs-draft/diagnose",
        description: "Diagnose provisional fBACS from per-system heating/cooling power and class evidence under public consultation §5.5.8; no verified NTA result or label",
        input: Some(InputSpec { key: "input", description: "Building use, complete heating/cooling system inventory, generator powers and BACS class evidence." }),
        category: "energy",
        run: |body| typed::<nta8800_core::bacs_draft::BacsDraftInput, _>(body, "input", |input| nta8800_core::bacs_draft::assess_bacs_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_indicators_draft",
        method: Method::Post,
        path: "/v1/nta8800/energy/indicators-draft/diagnose",
        description: "Provisional chapter-5 indicator ratios and directional rounding from separately supplied annual totals; no verified NTA calculation or label",
        input: Some(InputSpec { key: "input", description: "Ag, C1 annual need, and ordinary or paired EMG annual primary/renewable totals." }),
        category: "energy",
        run: |body| typed::<nta8800_core::indicators_draft::IndicatorsDraftInput, _>(body, "input", |input| nta8800_core::indicators_draft::assess_indicators_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_heating_aux_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/heating-aux-draft/diagnose",
        description: "Provisional chapter-9 equation 9.85 auxiliary electricity for one individual electric heat pump from supplied measured coefficients and input energy; excludes source pump/fan, no BENG",
        input: Some(InputSpec { key: "input", description: "Supplied measured A/B/C, nominal electric drive power and twelve monthly generator input electricity values." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::heating_aux_draft::HeatingAuxDraftInput, _>(body, "input", |input| nta8800_core::heating_aux_draft::assess_heating_aux_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "diagnose_heating_aux_measured_draft",
        method: Method::Post,
        path: "/v1/nta8800/heat-pumps/heating-aux-measured-draft/diagnose",
        description: "Provisional chapter-9 equations 9.86–9.88 derive A/B/C for one electric heat pump from supplied measured powers and cycle inputs, then apply 9.85; no verified NTA or BENG",
        input: Some(InputSpec { key: "input", description: "Measured standby and delivery-pump powers, supplied cycle times and modulation, and twelve monthly generator electricity values." }),
        category: "heat-pumps",
        run: |body| typed::<nta8800_core::heating_aux_draft::HeatingAuxMeasuredDraftInput, _>(body, "input", |input| nta8800_core::heating_aux_draft::assess_heating_aux_measured_draft(&input), StatusRule::Invalid),
    },
    Operation {
        name: "audit_reference_case",
        method: Method::Post,
        path: "/v1/nta8800/reference/audit",
        description: "Audit a reference-case manifest for missing evidence; never verifies its expected values",
        input: Some(InputSpec { key: "case", description: "Reference manifest with source provenance, project input and independently expected metrics." }),
        category: "reference",
        run: |body| typed::<nta8800_core::reference::ReferenceCase, _>(body, "case", nta8800_core::reference::audit_reference_case, StatusRule::Always),
    },
    Operation {
        name: "compare_reference_case",
        method: Method::Post,
        path: "/v1/nta8800/reference/compare",
        description: "Compare submitted BENG/TOjuli expectations to the unverified Rust project result; never attests or verifies source independence",
        input: Some(InputSpec { key: "case", description: "Reference manifest with source provenance, project input and independently expected metrics." }),
        category: "reference",
        run: |body| typed::<nta8800_core::reference::ReferenceCase, _>(body, "case", nta8800_core::reference::compare_reference_case, StatusRule::ReferenceCompare),
    },
    Operation {
        name: "compare_direct_diagnostic",
        method: Method::Post,
        path: "/v1/nta8800/reference/direct-diagnostic/compare",
        description: "Compare four direct-transmission diagnostic terms to supplied W/K expectations; never attests NTA compliance",
        input: Some(InputSpec { key: "case", description: "Diagnostic comparison case with explicit inputs and four independently supplied W/K terms." }),
        category: "reference",
        run: |body| typed::<nta8800_core::diagnostic_reference::DirectDiagnosticCase, _>(body, "case", nta8800_core::diagnostic_reference::compare_direct_diagnostic, StatusRule::InvalidCase),
    },
    Operation {
        name: "compare_gas_heat_pump_chain_diagnostic",
        method: Method::Post,
        path: "/v1/nta8800/reference/gas-chain-diagnostic/compare",
        description: "Compare all twelve gas heat-pump draft 9.62 and equipment-electricity values plus annual equipment electricity to supplied expectations; never attests provenance or NTA compliance",
        input: Some(InputSpec { key: "case", description: "Gas-chain input plus 25 supplied monthly and annual expected diagnostic values with provenance." }),
        category: "reference",
        run: |body| typed::<nta8800_core::gas_heat_pump_chain_reference::GasChainDiagnosticCase, _>(body, "case", nta8800_core::gas_heat_pump_chain_reference::compare_gas_heat_pump_chain_diagnostic, StatusRule::InvalidCase),
    },
];
