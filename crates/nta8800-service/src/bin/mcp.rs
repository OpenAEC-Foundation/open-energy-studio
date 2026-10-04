use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    schemars, tool, tool_router,
    transport::stdio,
    ServiceExt,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ProjectArgs {
    /// Legacy Open Energy Studio project object, not the outer .oes metadata wrapper.
    project: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ReferenceArgs {
    /// Reference manifest with source provenance, project input and independently expected metrics.
    case: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DirectTransmissionArgs {
    /// Explicit outdoor elements and thermal bridges with units and property references.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MonthlyDirectArgs {
    /// Explicit direct-to-outdoor conductance components and twelve supplied monthly temperature/hour records.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct MonthlyDemandArgs {
    /// One calculation zone: floor area, usage function (tables 7.13–7.15) and dwelling type, setpoints, transmission (explicit or chapter 8 components with annex D ground), ventilation conductances per balance, thermal-mass classes, internal-gain method, windows and opaque elements with source references.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ConstructionsArgs {
    /// Envelope elements (layered opaque constructions, tapered roofs, windows/doors with optional shutters, forfait existing-building values, rooflights, grilles, numerical U) plus forfait thermal bridges, with source references.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ResidentialSurveyArgs {
    /// ISSO 82.1 basic survey of one existing dwelling: construction year, dwelling type, envelope surfaces with insulation answers, glazing, heating, hot water, ventilation and PV, with "unknown" options.
    survey: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct UtilitySurveyArgs {
    /// ISSO 75.1 basic survey of one existing utility building: use functions with areas, building type, envelope, heating installation, cooling, ventilation with AHU, recirculation and flow control, humidification, hot water, lighting zones, PV and BACS, with "unknown" options.
    survey: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct VentilationArgs {
    /// One zone: use functions, height, system variant (table 11.5), heat recovery, infiltration, combustion appliances, ventilative cooling openings and fans, with source references.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SpaceHeatingChainArgs {
    /// Monthly demand input plus emission system, distribution route and one generator (gas boiler or forfait heat pump).
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BuildingPerformanceArgs {
    /// Space-heating chain, declared other services, on-site production, BACS factor and floor area.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct UnheatedTransmissionArgs {
    /// Named unheated spaces, explicit boundary terms and caller-supplied reduction factors.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DeclaredHeatingTableArgs {
    /// User-supplied product declaration table excerpt and query within its bounds.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ForfaitHeatPumpDraftArgs {
    /// Explicit scope, source class and supply temperature for consultation tables 9.27/9.29.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GasHeatPumpForfaitDraftArgs {
    /// Gas-engine or absorption pump, table-9.29 application, source, capacity and design-temperature evidence.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GasHeatPumpAuxDraftArgs {
    /// Gas generator, explicit auxiliary coefficients and twelve supplied month records.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GasHeatPumpMonthlyDraftArgs {
    /// Gas-engine/absorption table selection and twelve supplied generator-output months.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GasHeatPumpChainDraftArgs {
    /// Matching draft 9.62 and equipment-auxiliary inputs for one gas generator and twelve months.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GasCollectiveSourceDraftArgs {
    /// Linked gas heat-pump chain, collective-source temperature evidence and confirmed absence of a quality declaration.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ForfaitHeatPumpMonthlyDraftArgs {
    /// Draft table lookup plus twelve supplied generator-output months and source-system evidence.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GeneratorDispatchDraftArgs {
    /// New-build heating-node input, typed generators, rated thermal power and efficiency provenance.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct HybridHeatPumpMonthlyDraftArgs {
    /// New-build generator dispatch plus a matching electric heat pump forfait table and source-system evidence.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BoilerForfaitDraftArgs {
    /// Gas water-boiler class, placement and emission-circuit evidence for draft table 9.25.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BoilerForfaitMonthlyDraftArgs {
    /// Draft table-9.25 boiler selection and twelve supplied generator-output months.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DeclaredDhwArgs {
    /// Classified heat pump with controlled declaration and EN 16147 tap-profile inputs.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FinalEnergyDraftArgs {
    /// Complete monthly E_EPus per carrier and optional solar thermal practice contributions.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EpusDraftArgs {
    /// Monthly service terms by carrier and BACS factor from the public chapter-5 draft.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BacsDraftArgs {
    /// Building use, complete heating/cooling system inventory, generator powers and BACS class evidence.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct IndicatorsDraftArgs {
    /// Ag, C1 annual need, and ordinary or paired EMG annual primary/renewable totals.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct HeatingAuxDraftArgs {
    /// Supplied measured A/B/C, nominal electric drive power and twelve monthly generator input electricity values.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct HeatingAuxMeasuredDraftArgs {
    /// Measured standby and delivery-pump powers, supplied cycle times and modulation, and twelve monthly generator electricity values.
    input: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct DirectDiagnosticCaseArgs {
    /// Diagnostic comparison case with explicit inputs and four independently supplied W/K terms.
    case: Value,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GasChainDiagnosticCaseArgs {
    /// Gas-chain input plus 25 supplied monthly and annual expected diagnostic values with provenance.
    case: Value,
}

#[derive(Clone)]
struct EnergyMcp;

#[tool_router(server_handler)]
impl EnergyMcp {
    #[tool(
        description = "Diagnose a gas-engine or gas-absorption heat-pump COP from public draft tables 9.27/9.29; no gas input, auxiliaries, BENG or label"
    )]
    fn diagnose_gas_heat_pump_forfait_draft(
        &self,
        Parameters(args): Parameters<GasHeatPumpForfaitDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::gas_heat_pump_forfait_draft::GasHeatPumpForfaitDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(json!({"error":"invalid_gas_heat_pump_forfait_draft_shape","message":message.to_string()}).to_string())]),
            Ok(input) => {
                let result = nta8800_core::gas_heat_pump_forfait_draft::assess_gas_heat_pump_forfait_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) } else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Diagnose gas heat-pump generator auxiliary electricity from draft 9.91/9.92; no gas input, source pump, BENG or label"
    )]
    fn diagnose_gas_heat_pump_aux_draft(
        &self,
        Parameters(args): Parameters<GasHeatPumpAuxDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::gas_heat_pump_aux_draft::GasHeatPumpAuxDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(json!({"error":"invalid_gas_heat_pump_aux_draft_shape","message":message.to_string()}).to_string())]),
            Ok(input) => {
                let result = nta8800_core::gas_heat_pump_aux_draft::assess_gas_heat_pump_aux_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) } else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Diagnose draft 9.62 monthly input terms for a gas-engine or absorption heat pump; no carrier allocation, gas use, BENG or label"
    )]
    fn diagnose_gas_heat_pump_monthly_draft(
        &self,
        Parameters(args): Parameters<GasHeatPumpMonthlyDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::gas_heat_pump_monthly_draft::GasHeatPumpMonthlyDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(json!({"error":"invalid_gas_heat_pump_monthly_draft_shape","message":message.to_string()}).to_string())]),
            Ok(input) => {
                let result = nta8800_core::gas_heat_pump_monthly_draft::assess_gas_heat_pump_monthly_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) } else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Link gas heat-pump draft 9.62 and 9.91/9.92 monthly terms by generator, heat and evidence; no carrier allocation, gas use, BENG or label"
    )]
    fn diagnose_gas_heat_pump_chain_draft(
        &self,
        Parameters(args): Parameters<GasHeatPumpChainDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::gas_heat_pump_chain_draft::GasHeatPumpChainDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(json!({"error":"invalid_gas_heat_pump_chain_draft_shape","message":message.to_string()}).to_string())]),
            Ok(input) => {
                let result = nta8800_core::gas_heat_pump_chain_draft::assess_gas_heat_pump_chain_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) } else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Diagnose the separate dh heat and draft fossil/renewable primary contribution of a collective gas heat-pump source using public consultation chapters 5 and 9; no gas allocation, BENG or label"
    )]
    fn diagnose_gas_collective_source_draft(
        &self,
        Parameters(args): Parameters<GasCollectiveSourceDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::gas_collective_source_draft::GasCollectiveSourceDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(json!({"error":"invalid_gas_collective_source_draft_shape","message":message.to_string()}).to_string())]),
            Ok(input) => {
                let result = nta8800_core::gas_collective_source_draft::assess_gas_collective_source_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) } else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Diagnose draft gas water-boiler efficiency from consultation table 9.25; no pilot flame, auxiliaries, BENG or label"
    )]
    fn diagnose_boiler_forfait_draft(
        &self,
        Parameters(args): Parameters<BoilerForfaitDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::boiler_forfait_draft::BoilerForfaitDraftInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_boiler_forfait_draft_shape","message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result =
                    nta8800_core::boiler_forfait_draft::assess_boiler_forfait_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Diagnose draft gas input and individual-boiler forfait auxiliary electricity with equations 9.61/9.85 from supplied monthly boiler heat; no pilot flame, BENG or label"
    )]
    fn diagnose_boiler_forfait_monthly_draft(
        &self,
        Parameters(args): Parameters<BoilerForfaitMonthlyDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::boiler_forfait_draft::BoilerForfaitMonthlyDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(json!({"error":"invalid_boiler_forfait_monthly_draft_shape","message":message.to_string()}).to_string())]),
            Ok(input) => {
                let result = nta8800_core::boiler_forfait_draft::assess_boiler_forfait_monthly_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) } else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Diagnose a new-build hybrid heat-pump chain: chapter-9 draft dispatch, heat-pump electricity, boiler gas and boiler auxiliary electricity; optional measured heat-pump auxiliaries use linked months; no BENG or label"
    )]
    fn diagnose_hybrid_heat_pump_monthly_draft(
        &self,
        Parameters(args): Parameters<HybridHeatPumpMonthlyDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::hybrid_heat_pump_monthly_draft::HybridHeatPumpMonthlyDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_hybrid_heat_pump_monthly_draft_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::hybrid_heat_pump_monthly_draft::assess_hybrid_heat_pump_monthly_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) }
                else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Diagnose new-build heating generator dispatch from public draft tables 9.1/9.23 and equations 9.2/9.3/9.56/9.60; supplied node demand and powers, no BENG or label"
    )]
    fn diagnose_generator_dispatch_draft(
        &self,
        Parameters(args): Parameters<GeneratorDispatchDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::generator_dispatch_draft::GeneratorDispatchDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_generator_dispatch_draft_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::generator_dispatch_draft::assess_generator_dispatch_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) }
                else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Diagnose draft equation 9.62 from supplied monthly heat and a draft COP; no dispatch, auxiliaries, BENG or label"
    )]
    fn diagnose_forfait_heat_pump_monthly_draft(
        &self,
        Parameters(args): Parameters<ForfaitHeatPumpMonthlyDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::forfait_heat_pump_monthly_draft::ForfaitHeatPumpMonthlyDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_forfait_heat_pump_monthly_draft_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::forfait_heat_pump_monthly_draft::assess_forfait_heat_pump_monthly_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) }
                else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Look up a base electric heat-pump COP in public draft tables 9.27/9.29; applicability and final edition unverified, no annual performance or BENG"
    )]
    fn diagnose_forfait_heat_pump_draft(
        &self,
        Parameters(args): Parameters<ForfaitHeatPumpDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_forfait_heat_pump_draft_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::forfait_heat_pump_draft::assess_forfait_heat_pump_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) }
                else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(description = "Show the Rust NTA 8800 kernel scope, target version and attest status")]
    fn get_capabilities(&self) -> String {
        serde_json::to_string(&nta8800_core::capabilities()).expect("capabilities serialize")
    }

    #[tool(
        description = "Validate an Open Energy Studio project structure; does not calculate an energy label"
    )]
    fn validate_project(&self, Parameters(args): Parameters<ProjectArgs>) -> CallToolResult {
        match nta8800_core::assess_json(args.project) {
            Ok(assessment) => CallToolResult::success(vec![ContentBlock::text(
                serde_json::to_string(&assessment).expect("assessment serialize"),
            )]),
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({
                    "error": "invalid_project_shape", "message": message
                })
                .to_string(),
            )]),
        }
    }

    #[tool(
        description = "Request a Rust NTA 8800 BENG calculation; reports unavailable until independently validated"
    )]
    fn calculate_beng(&self, Parameters(args): Parameters<ProjectArgs>) -> CallToolResult {
        match nta8800_core::assess_json(args.project) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({
                    "error": "invalid_project_shape", "message": message
                })
                .to_string(),
            )]),
            Ok(assessment) if assessment.status == "invalid" => {
                CallToolResult::error(vec![ContentBlock::text(
                    json!({
                        "error": "invalid_project_input", "assessment": assessment
                    })
                    .to_string(),
                )])
            }
            Ok(_) => {
                CallToolResult::error(vec![ContentBlock::text(json!({
                "error": "calculation_unavailable",
                "message": "The Rust NTA 8800 calculation is not validated or available yet",
                "capabilities": nta8800_core::capabilities()
            }).to_string())])
            }
        }
    }

    #[tool(
        description = "Audit a reference-case manifest for missing evidence; never verifies its expected values"
    )]
    fn audit_reference_case(&self, Parameters(args): Parameters<ReferenceArgs>) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::reference::ReferenceCase>(args.case) {
            Ok(case) => CallToolResult::success(vec![ContentBlock::text(
                json!(nta8800_core::reference::audit_reference_case(case)).to_string(),
            )]),
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({
                    "error": "invalid_reference_shape", "message": message.to_string()
                })
                .to_string(),
            )]),
        }
    }

    #[tool(
        description = "Compare submitted BENG/TOjuli expectations to the unverified Rust project result; never attests or verifies source independence"
    )]
    fn compare_reference_case(
        &self,
        Parameters(args): Parameters<ReferenceArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::reference::ReferenceCase>(args.case) {
            Ok(case) => {
                let result = nta8800_core::reference::compare_reference_case(case);
                if result.status == "invalid_case" || result.status == "calculation_unavailable" {
                    CallToolResult::error(vec![ContentBlock::text(json!(result).to_string())])
                } else {
                    CallToolResult::success(vec![ContentBlock::text(json!(result).to_string())])
                }
            }
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error": "invalid_reference_shape", "message": message.to_string()})
                    .to_string(),
            )]),
        }
    }

    #[tool(
        description = "Diagnose the direct-to-outdoor A·U + L·psi + chi sum; does not produce NTA BENG or a verified label"
    )]
    fn diagnose_direct_transmission(
        &self,
        Parameters(args): Parameters<DirectTransmissionArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::direct_transmission::DirectTransmissionInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({
                    "error": "invalid_direct_transmission_shape", "message": message.to_string()
                })
                .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::direct_transmission::assess_direct_transmission(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Diagnose signed monthly direct-to-outdoor heat flow from supplied conductance, temperatures and hours; not NTA demand or BENG"
    )]
    fn diagnose_monthly_direct(
        &self,
        Parameters(args): Parameters<MonthlyDirectArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::monthly_direct_transmission::MonthlyDirectInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_monthly_direct_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result =
                    nta8800_core::monthly_direct_transmission::assess_monthly_direct(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Calculate the unverified NTA 8800 chapter 7 monthly heating and cooling need of one zone with De Bilt climate; lists omitted corrections; no BENG or label"
    )]
    fn calculate_monthly_demand(
        &self,
        Parameters(args): Parameters<MonthlyDemandArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::monthly_demand::MonthlyDemandInput>(args.input)
        {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_monthly_demand_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::monthly_demand::assess_monthly_demand(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Calculate unverified NTA 8800 8.2 envelope element U- and Rc-values (annexes C, E-I, L): layered and composite constructions with dU corrections, tapered roofs, windows and doors, forfait values for existing buildings, forfait psi and the dU_for supplement"
    )]
    fn calculate_constructions(
        &self,
        Parameters(args): Parameters<ConstructionsArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::envelope_elements::EnvelopeInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_constructions_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::envelope_elements::assess_envelope(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Calculate unverified NTA 8800 chapter 11 ventilation for one zone: required and effective flows from the pressure balance, supply temperatures, chapter 7 conductances per balance, fan and frost-protection electricity, and the fixed C1 run for BENG 1"
    )]
    fn calculate_ventilation(
        &self,
        Parameters(args): Parameters<VentilationArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::ventilation::VentilationInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_ventilation_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::ventilation::assess_ventilation(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Translate an ISSO 82.1 basic survey (basisopname) of an existing dwelling into NTA 8800 kernel input, list every applied default with its ISSO page, and calculate the unverified building performance and indicative label"
    )]
    fn assess_residential_survey(
        &self,
        Parameters(args): Parameters<ResidentialSurveyArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::opname::ResidentialSurvey>(args.survey) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_survey_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(survey) => {
                let result = nta8800_core::opname::assess_residential_survey(&survey);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "calculated_unverified" {
                    CallToolResult::success(vec![content])
                } else {
                    CallToolResult::error(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Translate an ISSO 75.1 basic survey (basisopname) of an existing utility building into NTA 8800 kernel input (one calculation zone; other functions up to 25 % merged), list every applied default with its ISSO page, and calculate the unverified building performance and indicative label"
    )]
    fn assess_utility_survey(
        &self,
        Parameters(args): Parameters<UtilitySurveyArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::opname::utility::UtilitySurvey>(args.survey) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_survey_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(survey) => {
                let result = nta8800_core::opname::utility::assess_utility_survey(&survey);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "calculated_unverified" {
                    CallToolResult::success(vec![content])
                } else {
                    CallToolResult::error(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Calculate the unverified monthly space-heating chain (need, emission, distribution, one generator) and energy per carrier; lists omitted terms; no BENG or label"
    )]
    fn calculate_space_heating_chain(
        &self,
        Parameters(args): Parameters<SpaceHeatingChainArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::space_heating_chain::SpaceHeatingChainInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_space_heating_chain_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::space_heating_chain::assess_space_heating_chain(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Calculate unverified building energy indicators, BENG and an indicative label class when the supplied input is complete; no registered label or attest"
    )]
    fn calculate_building_performance(
        &self,
        Parameters(args): Parameters<BuildingPerformanceArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::building_performance::BuildingPerformanceInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_building_performance_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result =
                    nta8800_core::building_performance::assess_building_performance(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Derive unverified BENG and an indicative label class from a saved .oes project with its ntaCalculation block; returns input gaps when data is missing; no registered label or attest"
    )]
    fn calculate_project_performance(
        &self,
        Parameters(args): Parameters<ProjectArgs>,
    ) -> CallToolResult {
        let result = nta8800_core::project_performance::assess_project_performance(&args.project);
        let content = ContentBlock::text(json!(result).to_string());
        if result.status == "calculated_unverified" {
            CallToolResult::success(vec![content])
        } else {
            CallToolResult::error(vec![content])
        }
    }

    #[tool(
        description = "Interpolate a supplied space-heating product declaration table within its bounds; does not calculate annual building performance, BENG or a label"
    )]
    fn diagnose_declared_heating_table(
        &self,
        Parameters(args): Parameters<DeclaredHeatingTableArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::declared_heating_table::DeclaredHeatingTableInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_declared_heating_table_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::declared_heating_table::assess_declared_heating_table(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Check declared tap-profile test energies and report their raw ratios; never produces NTA practice efficiency, annual BENG or a label"
    )]
    fn diagnose_declared_dhw(
        &self,
        Parameters(args): Parameters<DeclaredDhwArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::heat_pumps::HeatPumpInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_declared_dhw_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::declared_dhw::assess_declared_dhw(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Sum supplied monthly carrier energy and solar thermal contributions using public chapter-5 draft arithmetic; final norm edition and EDR unverified, no BENG or label"
    )]
    fn diagnose_final_energy_draft(
        &self,
        Parameters(args): Parameters<FinalEnergyDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::final_energy_draft::FinalEnergyDraftInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_final_energy_draft_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::final_energy_draft::assess_final_energy_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Compose monthly E_EPus per carrier from supplied service terms and BACS factor using public draft equations 5.20–5.21, then provisionally sum final energy; no verified NTA result or label"
    )]
    fn diagnose_epus_draft(&self, Parameters(args): Parameters<EpusDraftArgs>) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::epus_draft::EpusDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_epus_draft_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::epus_draft::assess_epus_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Diagnose provisional fBACS from per-system heating/cooling power and class evidence under public consultation §5.5.8; no verified NTA result or label"
    )]
    fn diagnose_bacs_draft(&self, Parameters(args): Parameters<BacsDraftArgs>) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::bacs_draft::BacsDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_bacs_draft_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::bacs_draft::assess_bacs_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Provisional chapter-5 indicator ratios and directional rounding from separately supplied annual totals; no verified NTA calculation or label"
    )]
    fn diagnose_indicators_draft(
        &self,
        Parameters(args): Parameters<IndicatorsDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::indicators_draft::IndicatorsDraftInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_indicators_draft_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::indicators_draft::assess_indicators_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Provisional chapter-9 equation 9.85 auxiliary electricity for one individual electric heat pump from supplied measured coefficients and input energy; excludes source pump/fan, no BENG"
    )]
    fn diagnose_heating_aux_draft(
        &self,
        Parameters(args): Parameters<HeatingAuxDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::heating_aux_draft::HeatingAuxDraftInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_heating_aux_draft_shape", "message":message.to_string()})
                    .to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::heating_aux_draft::assess_heating_aux_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Provisional chapter-9 equations 9.86–9.88 derive A/B/C for one electric heat pump from supplied measured powers and cycle inputs, then apply 9.85; no verified NTA or BENG"
    )]
    fn diagnose_heating_aux_measured_draft(
        &self,
        Parameters(args): Parameters<HeatingAuxMeasuredDraftArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::heating_aux_draft::HeatingAuxMeasuredDraftInput>(args.input) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_heating_aux_measured_draft_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::heating_aux_draft::assess_heating_aux_measured_draft(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" { CallToolResult::error(vec![content]) }
                else { CallToolResult::success(vec![content]) }
            }
        }
    }

    #[tool(
        description = "Diagnose conductance via unheated spaces using caller-supplied factors; not NTA demand or BENG"
    )]
    fn diagnose_unheated_transmission(
        &self,
        Parameters(args): Parameters<UnheatedTransmissionArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::unheated_transmission::UnheatedTransmissionInput>(
            args.input,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_unheated_transmission_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(input) => {
                let result = nta8800_core::unheated_transmission::assess_unheated_transmission(&input);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Compare four direct-transmission diagnostic terms to supplied W/K expectations; never attests NTA compliance"
    )]
    fn compare_direct_diagnostic(
        &self,
        Parameters(args): Parameters<DirectDiagnosticCaseArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::diagnostic_reference::DirectDiagnosticCase>(
            args.case,
        ) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(
                json!({"error":"invalid_direct_diagnostic_case_shape", "message":message.to_string()}).to_string(),
            )]),
            Ok(case) => {
                let result = nta8800_core::diagnostic_reference::compare_direct_diagnostic(case);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid_case" {
                    CallToolResult::error(vec![content])
                } else {
                    CallToolResult::success(vec![content])
                }
            }
        }
    }

    #[tool(
        description = "Compare all twelve gas heat-pump draft 9.62 and equipment-electricity values plus annual equipment electricity to supplied expectations; never attests provenance or NTA compliance"
    )]
    fn compare_gas_heat_pump_chain_diagnostic(
        &self,
        Parameters(args): Parameters<GasChainDiagnosticCaseArgs>,
    ) -> CallToolResult {
        match serde_json::from_value::<nta8800_core::gas_heat_pump_chain_reference::GasChainDiagnosticCase>(args.case) {
            Err(message) => CallToolResult::error(vec![ContentBlock::text(json!({"error":"invalid_gas_chain_diagnostic_case_shape","message":message.to_string()}).to_string())]),
            Ok(case) => {
                let result = nta8800_core::gas_heat_pump_chain_reference::compare_gas_heat_pump_chain_diagnostic(case);
                let content = ContentBlock::text(json!(result).to_string());
                if result.status == "invalid_case" { CallToolResult::error(vec![content]) } else { CallToolResult::success(vec![content]) }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = EnergyMcp.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
