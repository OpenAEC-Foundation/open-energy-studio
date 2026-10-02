//! HTTP adapter for `nta8800-core`.
//! Only loopback serving is supported by the bundled binary; network exposure
//! needs a separate authentication and deployment design.

use axum::{
    extract::Json,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize)]
pub struct ProjectRequest {
    pub project: Value,
}

#[derive(Deserialize)]
pub struct ReferenceRequest {
    pub case: nta8800_core::reference::ReferenceCase,
}

#[derive(Deserialize)]
pub struct DirectTransmissionRequest {
    pub input: nta8800_core::direct_transmission::DirectTransmissionInput,
}

#[derive(Deserialize)]
pub struct MonthlyDirectRequest {
    pub input: nta8800_core::monthly_direct_transmission::MonthlyDirectInput,
}

#[derive(Deserialize)]
pub struct MonthlyDemandRequest {
    pub input: nta8800_core::monthly_demand::MonthlyDemandInput,
}

#[derive(Deserialize)]
pub struct VentilationRequest {
    pub input: nta8800_core::ventilation::VentilationInput,
}

#[derive(Deserialize)]
pub struct SpaceHeatingChainRequest {
    pub input: nta8800_core::space_heating_chain::SpaceHeatingChainInput,
}

#[derive(Deserialize)]
pub struct BuildingPerformanceRequest {
    pub input: nta8800_core::building_performance::BuildingPerformanceInput,
}

#[derive(Deserialize)]
pub struct UnheatedTransmissionRequest {
    pub input: nta8800_core::unheated_transmission::UnheatedTransmissionInput,
}

#[derive(Deserialize)]
pub struct DeclaredHeatingTableRequest {
    pub input: nta8800_core::declared_heating_table::DeclaredHeatingTableInput,
}

#[derive(Deserialize)]
pub struct ForfaitHeatPumpDraftRequest {
    pub input: nta8800_core::forfait_heat_pump_draft::ForfaitHeatPumpDraftInput,
}

#[derive(Deserialize)]
pub struct GasHeatPumpForfaitDraftRequest {
    pub input: nta8800_core::gas_heat_pump_forfait_draft::GasHeatPumpForfaitDraftInput,
}

#[derive(Deserialize)]
pub struct GasHeatPumpAuxDraftRequest {
    pub input: nta8800_core::gas_heat_pump_aux_draft::GasHeatPumpAuxDraftInput,
}

#[derive(Deserialize)]
pub struct GasHeatPumpMonthlyDraftRequest {
    pub input: nta8800_core::gas_heat_pump_monthly_draft::GasHeatPumpMonthlyDraftInput,
}

#[derive(Deserialize)]
pub struct GasHeatPumpChainDraftRequest {
    pub input: nta8800_core::gas_heat_pump_chain_draft::GasHeatPumpChainDraftInput,
}

#[derive(Deserialize)]
pub struct GasCollectiveSourceDraftRequest {
    pub input: nta8800_core::gas_collective_source_draft::GasCollectiveSourceDraftInput,
}

#[derive(Deserialize)]
pub struct ForfaitHeatPumpMonthlyDraftRequest {
    pub input: nta8800_core::forfait_heat_pump_monthly_draft::ForfaitHeatPumpMonthlyDraftInput,
}

#[derive(Deserialize)]
pub struct GeneratorDispatchDraftRequest {
    pub input: nta8800_core::generator_dispatch_draft::GeneratorDispatchDraftInput,
}

#[derive(Deserialize)]
pub struct HybridHeatPumpMonthlyDraftRequest {
    pub input: nta8800_core::hybrid_heat_pump_monthly_draft::HybridHeatPumpMonthlyDraftInput,
}

#[derive(Deserialize)]
pub struct BoilerForfaitDraftRequest {
    pub input: nta8800_core::boiler_forfait_draft::BoilerForfaitDraftInput,
}

#[derive(Deserialize)]
pub struct BoilerForfaitMonthlyDraftRequest {
    pub input: nta8800_core::boiler_forfait_draft::BoilerForfaitMonthlyDraftInput,
}

#[derive(Deserialize)]
pub struct DeclaredDhwRequest {
    pub input: nta8800_core::heat_pumps::HeatPumpInput,
}

#[derive(Deserialize)]
pub struct FinalEnergyDraftRequest {
    pub input: nta8800_core::final_energy_draft::FinalEnergyDraftInput,
}

#[derive(Deserialize)]
pub struct EpusDraftRequest {
    pub input: nta8800_core::epus_draft::EpusDraftInput,
}

#[derive(Deserialize)]
pub struct BacsDraftRequest {
    pub input: nta8800_core::bacs_draft::BacsDraftInput,
}

#[derive(Deserialize)]
pub struct IndicatorsDraftRequest {
    pub input: nta8800_core::indicators_draft::IndicatorsDraftInput,
}

#[derive(Deserialize)]
pub struct HeatingAuxDraftRequest {
    pub input: nta8800_core::heating_aux_draft::HeatingAuxDraftInput,
}

#[derive(Deserialize)]
pub struct HeatingAuxMeasuredDraftRequest {
    pub input: nta8800_core::heating_aux_draft::HeatingAuxMeasuredDraftInput,
}

#[derive(Deserialize)]
pub struct DirectDiagnosticCaseRequest {
    pub case: nta8800_core::diagnostic_reference::DirectDiagnosticCase,
}

#[derive(Deserialize)]
pub struct GasChainDiagnosticCaseRequest {
    pub case: nta8800_core::gas_heat_pump_chain_reference::GasChainDiagnosticCase,
}

pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/nta8800/capabilities", get(capabilities))
        .route("/v1/nta8800/validate", post(validate))
        .route("/v1/nta8800/calculate", post(calculate))
        .route(
            "/v1/nta8800/transmission/direct/diagnose",
            post(diagnose_direct_transmission),
        )
        .route(
            "/v1/nta8800/transmission/direct/monthly-diagnose",
            post(diagnose_monthly_direct),
        )
        .route(
            "/v1/nta8800/demand/monthly/calculate",
            post(calculate_monthly_demand),
        )
        .route(
            "/v1/nta8800/heating/space-heating-chain/calculate",
            post(calculate_space_heating_chain),
        )
        .route(
            "/v1/nta8800/ventilation/calculate",
            post(calculate_ventilation),
        )
        .route(
            "/v1/nta8800/project/performance",
            post(calculate_project_performance),
        )
        .route(
            "/v1/nta8800/performance/calculate",
            post(calculate_building_performance),
        )
        .route(
            "/v1/nta8800/transmission/unheated/diagnose",
            post(diagnose_unheated_transmission),
        )
        .route(
            "/v1/nta8800/heat-pumps/declared-heating-table/diagnose",
            post(diagnose_declared_heating_table),
        )
        .route(
            "/v1/nta8800/heat-pumps/forfait-cop-draft/diagnose",
            post(diagnose_forfait_heat_pump_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/gas-forfait-cop-draft/diagnose",
            post(diagnose_gas_heat_pump_forfait_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/gas-aux-draft/diagnose",
            post(diagnose_gas_heat_pump_aux_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/gas-forfait-monthly-draft/diagnose",
            post(diagnose_gas_heat_pump_monthly_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/gas-chain-draft/diagnose",
            post(diagnose_gas_heat_pump_chain_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/gas-collective-source-draft/diagnose",
            post(diagnose_gas_collective_source_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/forfait-monthly-draft/diagnose",
            post(diagnose_forfait_heat_pump_monthly_draft),
        )
        .route(
            "/v1/nta8800/heating/generator-dispatch-draft/diagnose",
            post(diagnose_generator_dispatch_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/hybrid-monthly-draft/diagnose",
            post(diagnose_hybrid_heat_pump_monthly_draft),
        )
        .route(
            "/v1/nta8800/boilers/forfait-draft/diagnose",
            post(diagnose_boiler_forfait_draft),
        )
        .route(
            "/v1/nta8800/boilers/forfait-monthly-draft/diagnose",
            post(diagnose_boiler_forfait_monthly_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/declared-dhw/diagnose",
            post(diagnose_declared_dhw),
        )
        .route(
            "/v1/nta8800/energy/final-draft/diagnose",
            post(diagnose_final_energy_draft),
        )
        .route(
            "/v1/nta8800/energy/epus-draft/diagnose",
            post(diagnose_epus_draft),
        )
        .route(
            "/v1/nta8800/energy/bacs-draft/diagnose",
            post(diagnose_bacs_draft),
        )
        .route(
            "/v1/nta8800/energy/indicators-draft/diagnose",
            post(diagnose_indicators_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/heating-aux-draft/diagnose",
            post(diagnose_heating_aux_draft),
        )
        .route(
            "/v1/nta8800/heat-pumps/heating-aux-measured-draft/diagnose",
            post(diagnose_heating_aux_measured_draft),
        )
        .route("/v1/nta8800/reference/audit", post(audit_reference))
        .route(
            "/v1/nta8800/reference/direct-diagnostic/compare",
            post(compare_direct_diagnostic),
        )
        .route(
            "/v1/nta8800/reference/gas-chain-diagnostic/compare",
            post(compare_gas_heat_pump_chain_diagnostic),
        )
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "kernel": "rust" }))
}

async fn capabilities() -> Json<Value> {
    Json(serde_json::to_value(nta8800_core::capabilities()).expect("capabilities serialize"))
}

async fn validate(Json(request): Json<ProjectRequest>) -> (StatusCode, Json<Value>) {
    match nta8800_core::assess_json(request.project) {
        Ok(assessment) => (StatusCode::OK, Json(json!(assessment))),
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_project_shape", "message": message })),
        ),
    }
}

async fn calculate(Json(request): Json<ProjectRequest>) -> (StatusCode, Json<Value>) {
    match nta8800_core::assess_json(request.project) {
        Err(message) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_project_shape", "message": message })),
        ),
        Ok(assessment) if assessment.status == "invalid" => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "invalid_project_input", "assessment": assessment })),
        ),
        Ok(_) => (
            StatusCode::NOT_IMPLEMENTED,
            Json(json!({
                "error": "calculation_unavailable",
                "message": "The Rust NTA 8800 calculation is not validated or available yet",
                "capabilities": nta8800_core::capabilities()
            })),
        ),
    }
}

async fn audit_reference(Json(request): Json<ReferenceRequest>) -> Json<Value> {
    Json(json!(nta8800_core::reference::audit_reference_case(
        request.case
    )))
}

async fn compare_direct_diagnostic(
    Json(request): Json<DirectDiagnosticCaseRequest>,
) -> (StatusCode, Json<Value>) {
    let result = nta8800_core::diagnostic_reference::compare_direct_diagnostic(request.case);
    let status = if result.status == "invalid_case" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(result)))
}

async fn compare_gas_heat_pump_chain_diagnostic(
    Json(request): Json<GasChainDiagnosticCaseRequest>,
) -> (StatusCode, Json<Value>) {
    let result =
        nta8800_core::gas_heat_pump_chain_reference::compare_gas_heat_pump_chain_diagnostic(
            request.case,
        );
    let status = if result.status == "invalid_case" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(result)))
}

async fn diagnose_direct_transmission(
    Json(request): Json<DirectTransmissionRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::direct_transmission::assess_direct_transmission(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_monthly_direct(
    Json(request): Json<MonthlyDirectRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::monthly_direct_transmission::assess_monthly_direct(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn calculate_monthly_demand(
    Json(request): Json<MonthlyDemandRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::monthly_demand::assess_monthly_demand(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn calculate_ventilation(
    Json(request): Json<VentilationRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::ventilation::assess_ventilation(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn calculate_space_heating_chain(
    Json(request): Json<SpaceHeatingChainRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::space_heating_chain::assess_space_heating_chain(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn calculate_project_performance(
    Json(request): Json<ProjectRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::project_performance::assess_project_performance(&request.project);
    let status = match assessment.status {
        "calculated_unverified" => StatusCode::OK,
        _ => StatusCode::UNPROCESSABLE_ENTITY,
    };
    (status, Json(json!(assessment)))
}

async fn calculate_building_performance(
    Json(request): Json<BuildingPerformanceRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::building_performance::assess_building_performance(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_unheated_transmission(
    Json(request): Json<UnheatedTransmissionRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::unheated_transmission::assess_unheated_transmission(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_declared_heating_table(
    Json(request): Json<DeclaredHeatingTableRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::declared_heating_table::assess_declared_heating_table(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_forfait_heat_pump_draft(
    Json(request): Json<ForfaitHeatPumpDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::forfait_heat_pump_draft::assess_forfait_heat_pump_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_gas_heat_pump_forfait_draft(
    Json(request): Json<GasHeatPumpForfaitDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::gas_heat_pump_forfait_draft::assess_gas_heat_pump_forfait_draft(
        &request.input,
    );
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_gas_heat_pump_aux_draft(
    Json(request): Json<GasHeatPumpAuxDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::gas_heat_pump_aux_draft::assess_gas_heat_pump_aux_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_gas_heat_pump_monthly_draft(
    Json(request): Json<GasHeatPumpMonthlyDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::gas_heat_pump_monthly_draft::assess_gas_heat_pump_monthly_draft(
        &request.input,
    );
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_gas_heat_pump_chain_draft(
    Json(request): Json<GasHeatPumpChainDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::gas_heat_pump_chain_draft::assess_gas_heat_pump_chain_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_gas_collective_source_draft(
    Json(request): Json<GasCollectiveSourceDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::gas_collective_source_draft::assess_gas_collective_source_draft(
        &request.input,
    );
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_forfait_heat_pump_monthly_draft(
    Json(request): Json<ForfaitHeatPumpMonthlyDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::forfait_heat_pump_monthly_draft::assess_forfait_heat_pump_monthly_draft(
            &request.input,
        );
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_generator_dispatch_draft(
    Json(request): Json<GeneratorDispatchDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::generator_dispatch_draft::assess_generator_dispatch_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_hybrid_heat_pump_monthly_draft(
    Json(request): Json<HybridHeatPumpMonthlyDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::hybrid_heat_pump_monthly_draft::assess_hybrid_heat_pump_monthly_draft(
            &request.input,
        );
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_boiler_forfait_draft(
    Json(request): Json<BoilerForfaitDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::boiler_forfait_draft::assess_boiler_forfait_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_boiler_forfait_monthly_draft(
    Json(request): Json<BoilerForfaitMonthlyDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::boiler_forfait_draft::assess_boiler_forfait_monthly_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_declared_dhw(
    Json(request): Json<DeclaredDhwRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::declared_dhw::assess_declared_dhw(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_final_energy_draft(
    Json(request): Json<FinalEnergyDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::final_energy_draft::assess_final_energy_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_epus_draft(Json(request): Json<EpusDraftRequest>) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::epus_draft::assess_epus_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_bacs_draft(Json(request): Json<BacsDraftRequest>) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::bacs_draft::assess_bacs_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_indicators_draft(
    Json(request): Json<IndicatorsDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::indicators_draft::assess_indicators_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_heating_aux_draft(
    Json(request): Json<HeatingAuxDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment = nta8800_core::heating_aux_draft::assess_heating_aux_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

async fn diagnose_heating_aux_measured_draft(
    Json(request): Json<HeatingAuxMeasuredDraftRequest>,
) -> (StatusCode, Json<Value>) {
    let assessment =
        nta8800_core::heating_aux_draft::assess_heating_aux_measured_draft(&request.input);
    let status = if assessment.status == "invalid" {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        StatusCode::OK
    };
    (status, Json(json!(assessment)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn measured_heating_aux_route_derives_coefficients_without_forfait() {
        let months: Vec<Value> = (1..=12)
            .map(|month| json!({"month":month,"generatorInputElectricityKwh":100}))
            .collect();
        let mut input = json!({
            "generatorId":"hp-1", "generatorSourceReference":"schedule",
            "measurements":{"standbyElectronicsW":10,
                "deliveryPumpDuringCompressorW":200,"deliveryPumpPrePostW":90,
                "pumpPreRunSeconds":300,"pumpPostRunSeconds":300,
                "averageCompressorOnSeconds":600,"meanCompressorModulation":0.5,
                "nominalElectricDriveKw":2,"measurementSourceReference":"meter",
                "timingSourceReference":"cycle"},
            "inputEnergySourceReference":"generator metering", "months":months
        });
        let request = |input: &Value| {
            Request::post("/v1/nta8800/heat-pumps/heating-aux-measured-draft/diagnose")
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["derivedCoefficients"]["aAnnualKwh"], 87.6);
        assert_eq!(value["derivedCoefficients"]["bKw"], 0.19);
        assert_eq!(
            value["auxiliary"]["monthlyAuxiliaryElectricityKwh"][0]["electricityKwh"],
            26.3
        );
        assert_eq!(value["auxiliary"]["annualAuxiliaryElectricityKwh"], 315.6);
        assert_eq!(value["bengCalculationAvailable"], false);
        input["measurements"]["averageCompressorOnSeconds"] = json!(0);
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(value["derivedCoefficients"].is_null() && value["auxiliary"].is_null());
    }

    #[tokio::test]
    async fn heating_aux_draft_endpoint_uses_generator_input_and_no_forfait() {
        let months: Vec<Value> = (1..=12)
            .map(|month| json!({"month":month,"generatorInputElectricityKwh":100}))
            .collect();
        let mut input = json!({
            "generatorId":"hp-1", "generatorSourceReference":"schedule",
            "coefficients":{"aAnnualKwh":60,"bKw":0.1,"cDimensionless":0.5,
                "nominalElectricDriveKw":2,"sourceReference":"measured record"},
            "inputEnergySourceReference":"generator electricity", "months":months
        });
        let request = |input: &Value| {
            Request::post("/v1/nta8800/heat-pumps/heating-aux-draft/diagnose")
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(
            value["monthlyAuxiliaryElectricityKwh"][0]["electricityKwh"],
            15.0
        );
        assert_eq!(value["annualAuxiliaryElectricityKwh"], 180.0);
        assert_eq!(value["forfaitUsed"], false);
        assert_eq!(value["bengCalculationAvailable"], false);
        input["coefficients"]["cDimensionless"] = json!(0);
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(value["monthlyAuxiliaryElectricityKwh"]
            .as_array()
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn draft_indicator_endpoint_requires_emg_pair_and_never_returns_a_label() {
        let mut input = json!({
            "calculationScope":"utility", "totalUsableFloorAreaM2":100,
            "areaSourceReference":"measured", "annualNeedC1Kwh":1234.001,
            "needSourceReference":"separate annual need",
            "scenarios":[{"kind":"emg_declaration","annualPrimaryFossilKwh":2131.001,
                "annualRenewableKwh":1000,"sourceReference":"declared annual totals"},
                {"kind":"emg_forfait","annualPrimaryFossilKwh":3000,
                "annualRenewableKwh":500,"sourceReference":"forfait annual totals"}]
        });
        let request = |input: &Value| {
            Request::post("/v1/nta8800/energy/indicators-draft/diagnose")
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["needIndicatorKwhPerM2Year"], 12.35);
        assert_eq!(
            value["scenarios"][0]["primaryFossilIndicatorKwhPerM2Year"],
            21.32
        );
        assert_eq!(value["scenarios"][0]["renewableSharePercent"], 31.9);
        assert_eq!(value["labelAvailable"], false);
        input["scenarios"].as_array_mut().unwrap().pop();
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(value["scenarios"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn bacs_draft_endpoint_uses_each_system_and_preserves_incomplete_evidence() {
        let mut input = json!({
            "buildingUse":"utility", "systemInventoryComplete":true,
            "systems":[{"id":"heating-a","service":"heating","sourceReference":"system schedule",
                "generators":[{"id":"g1","nominalThermalCapacityKw":291,"sourceReference":"nameplate"}]}],
            "bacs":{"present":true,"automaticControlsClass":"C","energyManagementClass":"C",
                "sourceReference":"controls inspection"}
        });
        let request = |input: &Value| {
            Request::post("/v1/nta8800/energy/bacs-draft/diagnose")
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["factor"], 1.05);
        assert_eq!(value["referenceVerified"], false);
        input["bacs"]["energyManagementClass"] = Value::Null;
        let response = app().oneshot(request(&input)).await.unwrap();
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["status"], "incomplete");
        assert!(value["factor"].is_null());
        input["systems"][0]["generators"][0]["nominalThermalCapacityKw"] = json!(0);
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn epus_draft_endpoint_composes_bacs_weighted_months_without_enabling_beng() {
        let months: Vec<Value> = (1..=12)
            .map(|month| {
                json!({
                    "month":month,
                    "spaceHeatingKwh":100,"humidificationKwh":0,"ventilationKwh":5,
                    "lightingKwh":0,"spaceCoolingKwh":20,"dehumidificationKwh":0,
                    "domesticHotWaterKwh":30,"collectiveSourceHeatKwh":0,
                    "auxiliary":{"spaceHeatingKwh":10,"humidificationKwh":0,
                        "spaceCoolingKwh":2,"dehumidificationKwh":0,
                        "domesticHotWaterKwh":3,"solarPvKwh":1}
                })
            })
            .collect();
        let mut input = json!({
            "bacsFactor":1.05,"bacsSourceReference":"synthetic factor",
            "carrierInventoryComplete":true,"solarThermalInventoryComplete":true,
            "totalUsableFloorAreaM2":100,"areaSourceReference":"synthetic area",
            "carriers":[{"carrierCode":"el","sourceReference":"synthetic services","months":months}],
            "solarThermal":[]
        });
        let request = |input: &Value| {
            Request::post("/v1/nta8800/energy/epus-draft/diagnose")
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["monthlyByCarrierKwh"][0]["energyKwh"], 177.6);
        assert_eq!(value["finalEnergy"]["finalEnergyKwhPerYear"], 2131.2);
        assert_eq!(
            value["finalEnergy"]["finalEnergyIndicatorKwhPerM2Year"],
            21.32
        );
        assert_eq!(value["bengCalculationAvailable"], false);
        input["carriers"][0]["carrierCode"] = json!("gas");
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["monthlyByCarrierKwh"], json!([]));
        assert!(value["finalEnergy"].is_null());
    }

    #[tokio::test]
    async fn epus_draft_endpoint_derives_collective_gas_source_heat_on_dh_once() {
        let service_months: Vec<Value> = (1..=12)
            .map(|month| {
                json!({
                    "month":month,
                    "spaceHeatingKwh":100,"humidificationKwh":0,"ventilationKwh":5,
                    "lightingKwh":0,"spaceCoolingKwh":20,"dehumidificationKwh":0,
                    "domesticHotWaterKwh":30,"collectiveSourceHeatKwh":0,
                    "auxiliary":{"spaceHeatingKwh":10,"humidificationKwh":0,
                        "spaceCoolingKwh":2,"dehumidificationKwh":0,
                        "domesticHotWaterKwh":3,"solarPvKwh":1}
                })
            })
            .collect();
        let dh_months: Vec<Value> = (1..=12)
            .map(|month| {
                json!({
                    "month":month,
                    "spaceHeatingKwh":0,"humidificationKwh":0,"ventilationKwh":0,
                    "lightingKwh":0,"spaceCoolingKwh":0,"dehumidificationKwh":0,
                    "domesticHotWaterKwh":0,"collectiveSourceHeatKwh":0,
                    "auxiliary":{"spaceHeatingKwh":0,"humidificationKwh":0,
                        "spaceCoolingKwh":0,"dehumidificationKwh":0,
                        "domesticHotWaterKwh":0,"solarPvKwh":0}
                })
            })
            .collect();
        let output_months = (1..=12)
            .map(|month| json!({"month":month,"energyKwh":1000.0}))
            .collect::<Vec<_>>();
        let aux_months = (1..=12)
            .map(|month| json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0}))
            .collect::<Vec<_>>();
        let evidence = json!({
            "chain": {
                "monthly": {"forfait": {"generatorId":"ga-1","drive":"absorption","application":"utility",
                    "applicationReference":"office","collectiveBuildingInstallation":false,"externalHeatSupply":false,
                    "thermalCapacityKw":20.0,"capacityReference":"plate","source":"groundwater_aquifer",
                    "sourceReference":"source plan","designSupplyTemperatureC":35.0,"designSupplyReference":"design"},
                    "generatorOutputKwh":output_months,"generatorOutputReference":"heat meter",
                    "sourceSystem":"collective_groundwater_surface_or_at_least15_c","sourceSystemReference":"invoice"},
                "auxiliary": {"generatorId":"ga-1","drive":"absorption","nominalThermalCapacityKw":20.0,
                    "capacityReference":"plate","standbyElectronicsW":10.0,"burnerAuxiliaryWPerKw":1.0,
                    "solutionPumpWPerKw":0.0,"coefficientsReference":"draft","meanModulation":1.0,
                    "modulationReference":"draft","buildingShare":1.0,"buildingShareReference":"whole building",
                    "forfaitCopUsed":true,"monthHoursReference":"schedule","generatorOutputReference":"heat meter","months":aux_months}
            },
            "sourceTemperatureClass":"below20_c","sourceTemperatureReference":"source design",
            "noQualityDeclarationConfirmed":true,"noQualityDeclarationReference":"register check"
        });
        let mut input = json!({
            "bacsFactor":1.05,"bacsSourceReference":"synthetic factor",
            "carrierInventoryComplete":true,"solarThermalInventoryComplete":true,
            "totalUsableFloorAreaM2":100,"areaSourceReference":"synthetic area",
            "carriers":[
                {"carrierCode":"el","sourceReference":"synthetic services","months":service_months},
                {"carrierCode":"dh","sourceReference":"source invoice","months":dh_months}
            ],
            "gasCollectiveSourceEvidence":evidence,
            "solarThermal":[]
        });
        let request = |input: &Value| {
            Request::post("/v1/nta8800/energy/epus-draft/diagnose")
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["gasCollectiveSourceDerived"], true);
        assert!(value["gasCollectiveSourceFingerprint"].is_string());
        let dh: Vec<&Value> = value["monthlyByCarrierKwh"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|item| item["carrierCode"] == "dh")
            .collect();
        assert_eq!(dh.len(), 12);
        let expected = 1000.0 * (1.0 - 1.0 / 2.1);
        assert!((dh[0]["energyKwh"].as_f64().unwrap() - expected).abs() < 1e-8);
        assert_eq!(value["bengCalculationAvailable"], false);
        input["carriers"][1]["months"][0]["collectiveSourceHeatKwh"] = json!(100.0);
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["monthlyByCarrierKwh"], json!([]));
        assert_eq!(value["gasCollectiveSourceDerived"], false);
        assert!(value["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "gas_collective_source_double_count"));
    }

    #[tokio::test]
    async fn final_energy_draft_endpoint_requires_complete_monthly_inputs() {
        let months: Vec<Value> = (1..=12)
            .map(|month| json!({"month":month,"energyKwh":100}))
            .collect();
        let input = json!({
            "carrierInventoryComplete":true,
            "solarThermalInventoryComplete":true,
            "totalUsableFloorAreaM2":100,
            "areaSourceReference":"synthetic area supplied by test",
            "carriers":[{"carrierCode":"el","sourceReference":"synthetic 5.5.3 input",
                "monthlyEpusKwh":months}],
            "solarThermal":[]
        });
        let request = |input: &Value| {
            Request::post("/v1/nta8800/energy/final-draft/diagnose")
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["finalEnergyKwhPerYear"], 1200.0);
        assert_eq!(value["eedFinalEnergyKwhPerYear"], 1200.0);
        assert_eq!(value["finalEnergyIndicatorKwhPerM2Year"], 12.0);
        assert_eq!(value["finalEditionVerified"], false);
        assert_eq!(value["bengCalculationAvailable"], false);
        let mut invalid = input;
        invalid["carriers"][0]["monthlyEpusKwh"][1]["month"] = json!(1);
        let response = app().oneshot(request(&invalid)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(value["finalEnergyKwhPerYear"].is_null());
    }

    #[tokio::test]
    async fn declared_dhw_endpoint_reports_only_raw_test_values() {
        let mut input: Value = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20260143gg-dhw-test-input.json"
        ))
        .unwrap();
        let request = |input: &Value| {
            Request::post("/v1/nta8800/heat-pumps/declared-dhw/diagnose")
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["profiles"][0]["tapProfile"], "M");
        assert_eq!(value["annualPerformanceAvailable"], false);
        assert_eq!(value["referenceVerified"], false);
        input["dhwTestPoints"][0]["inputEnergyKwhPerDay"] = json!(0);
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["profiles"], json!([]));
    }

    #[tokio::test]
    async fn declared_heating_table_endpoint_interpolates_only_within_supplied_grid() {
        let mut input: Value = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20250005gk-heating-grid-sample.json"
        ))
        .unwrap();
        input["grossHeatDemandKwhPerYear"] = json!(8333.5);
        input["designSupplyTemperatureC"] = json!(32.5);
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/heat-pumps/declared-heating-table/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!((result["generationEfficiency"].as_f64().unwrap() - 5.68575).abs() < 1e-12);
        assert_eq!(result["interpolation"]["demandWeight"], 0.5);
        assert_eq!(result["interpolation"]["temperatureWeight"], 0.5);
        assert_eq!(result["declarationNormVersion"], "NTA 8800:2024");
        assert_eq!(result["annualPerformanceAvailable"], false);
        assert_eq!(result["bengCalculationAvailable"], false);
        input["designSupplyTemperatureC"] = json!(25.0);
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/heat-pumps/declared-heating-table/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["interpolation"]["temperatureLowerC"], 30.0);
        assert_eq!(result["interpolation"]["temperatureUpperC"], 30.0);
        assert_eq!(result["interpolation"]["temperatureWeight"], 0.0);
        input["designSupplyTemperatureC"] = json!(60.0);
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/heat-pumps/declared-heating-table/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(result["generationEfficiency"].is_null());
        assert!(result["interpolation"].is_null());
    }

    #[tokio::test]
    async fn calculate_never_returns_an_unverified_beng_result() {
        let body = json!({"project": {
            "id": "house", "name": "House", "buildingFunction": "residential",
            "zones": [{"id": "z1", "floorArea": 100.0, "volume": 250.0,
                "surfaces": [{"id": "wall", "area": 50.0, "zoneId": "z1", "windows": []}]}]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/calculate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["error"], "calculation_unavailable");
        assert_eq!(result["capabilities"]["attestStatus"], "unattested");
    }

    #[tokio::test]
    async fn calculate_rejects_overflowed_project_area_without_a_result() {
        let body = json!({"project": {
            "id":"large", "name":"Large", "buildingFunction":"residential",
            "zones":[
                {"id":"z1", "floorArea":1e308, "volume":250.0,
                    "surfaces":[{"id":"s1", "area":10.0, "windows":[]}]},
                {"id":"z2", "floorArea":1e308, "volume":250.0,
                    "surfaces":[{"id":"s2", "area":10.0, "windows":[]}]}
            ]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/calculate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["error"], "invalid_project_input");
        assert!(result["assessment"]["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "floor_area_sum_invalid"));
        assert_eq!(result["assessment"]["calculationAvailable"], false);
    }

    fn monthly_demand_sample() -> Value {
        serde_json::from_str(include_str!(
            "../../../training-data/nta8800-monthly-demand-synthetic.json"
        ))
        .unwrap()
    }

    async fn post_json(uri: &str, body: Value) -> (StatusCode, Value) {
        let response = app()
            .oneshot(
                Request::post(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn monthly_demand_route_returns_unverified_need() {
        let (status, result) = post_json(
            "/v1/nta8800/demand/monthly/calculate",
            json!({"input": monthly_demand_sample()}),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["status"], "calculated_unverified");
        assert_eq!(result["monthly"].as_array().unwrap().len(), 12);
        assert!(result["annualHeatingNeedKwh"].as_f64().unwrap() > 0.0);
        assert_eq!(result["referenceVerified"], false);
        assert_eq!(result["bengCalculationAvailable"], false);
    }

    #[tokio::test]
    async fn ventilation_route_returns_actual_and_c1_runs() {
        let input: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-ventilation-synthetic.json"
        ))
        .unwrap();
        let (status, result) = post_json(
            "/v1/nta8800/ventilation/calculate",
            json!({ "input": input }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["status"], "calculated_unverified");
        assert_eq!(result["actual"]["months"].as_array().unwrap().len(), 12);
        assert!(result["fixedC1"]["demandFlows"].as_array().unwrap().len() >= 2);
        assert_eq!(result["referenceVerified"], false);
    }

    #[tokio::test]
    async fn space_heating_chain_route_returns_gas_per_month() {
        let input: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-space-heating-chain-synthetic.json"
        ))
        .unwrap();
        let (status, result) = post_json(
            "/v1/nta8800/heating/space-heating-chain/calculate",
            json!({ "input": input }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["status"], "calculated_unverified");
        assert!(result["annualNaturalGasKwh"].as_f64().unwrap() > 0.0);
        assert_eq!(result["demand"]["status"], "calculated_unverified");
        assert_eq!(result["bengCalculationAvailable"], false);
    }

    #[tokio::test]
    async fn project_performance_route_reports_gaps_or_results() {
        let project: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-project-performance-synthetic.json"
        ))
        .unwrap();
        let (status, result) = post_json(
            "/v1/nta8800/project/performance",
            json!({ "project": project.clone() }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["status"], "calculated_unverified");
        assert!(result["performance"]["primaryFossilIndicatorKwhPerM2Year"].is_number());
        let mut incomplete = project;
        incomplete.as_object_mut().unwrap().remove("ntaCalculation");
        let (status, result) = post_json(
            "/v1/nta8800/project/performance",
            json!({ "project": incomplete }),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(result["status"], "incomplete");
        assert_eq!(result["gaps"][0]["code"], "nta_calculation_block_missing");
    }

    #[tokio::test]
    async fn performance_route_returns_unverified_indicators_without_label() {
        let input: Value = serde_json::from_str(include_str!(
            "../../../training-data/nta8800-building-performance-synthetic.json"
        ))
        .unwrap();
        let (status, result) = post_json(
            "/v1/nta8800/performance/calculate",
            json!({ "input": input }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(result["status"], "calculated_unverified");
        assert!(result["primaryFossilIndicatorKwhPerM2Year"].is_number());
        assert!(result["renewableSharePercent"].is_number());
        assert!(result["needIndicatorKwhPerM2Year"].is_null());
        assert_eq!(result["labelAvailable"], false);
        assert_eq!(result["attestStatus"], "unattested");
    }

    #[tokio::test]
    async fn monthly_demand_route_rejects_unsupported_tilt_without_numbers() {
        let mut input = monthly_demand_sample();
        input["windows"][0]["tiltDeg"] = json!(200.0);
        let (status, result) = post_json(
            "/v1/nta8800/demand/monthly/calculate",
            json!({"input": input}),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(result["issues"][0]["code"], "tilt_unsupported");
        assert!(result["annualHeatingNeedKwh"].is_null());
    }

    #[tokio::test]
    async fn monthly_direct_route_reports_signed_flow_without_a_beng_claim() {
        let months = (1..=12)
            .map(|month| {
                json!({
                    "month":month, "indoorTemperatureC":20.0,
                    "outdoorTemperatureC":10.0, "hours":100.0
                })
            })
            .collect::<Vec<_>>();
        let body = json!({"input":{
            "direct":{"elements":[{"id":"wall", "areaM2":10.0,
                "uValueWPerM2k":0.2,"sourceReference":"drawing"}]},
            "months":months
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/transmission/direct/monthly-diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["annualSignedHeatFlowKwh"], 24.0);
        assert_eq!(result["directConductanceWPerK"], 2.0);
        assert_eq!(result["referenceVerified"], false);
        assert_eq!(result["bengCalculationAvailable"], false);
    }

    #[tokio::test]
    async fn monthly_direct_route_rejects_incomplete_months_without_energy() {
        let body = json!({"input":{
            "direct":{"elements":[{"id":"wall", "areaM2":10.0,
                "uValueWPerM2k":0.2,"sourceReference":"drawing"}]},
            "months":[{"month":1,"indoorTemperatureC":20.0,
                "outdoorTemperatureC":10.0,"hours":100.0}]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/transmission/direct/monthly-diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(result["annualSignedHeatFlowKwh"].is_null());
        assert!(result["monthly"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn unheated_route_reduces_explicit_space_conductance_without_a_beng_claim() {
        let body = json!({"input":{"spaces":[{
            "id":"garage", "reductionFactor":0.5,
            "factorSourceReference":"supplied-assumption-1",
            "boundary":{"elements":[{"id":"wall", "areaM2":10,
                "uValueWPerM2k":0.4, "sourceReference":"drawing-1"}],
                "linearBridges":[{"id":"edge", "lengthM":2,
                    "psiWPerMk":0.1, "sourceReference":"detail-1"}]}
        }]}});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/transmission/unheated/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["totalReducedConductanceWPerK"], 2.1);
        assert_eq!(result["spaces"][0]["unreducedConductanceWPerK"], 4.2);
        assert_eq!(result["referenceVerified"], false);
        assert_eq!(result["bengCalculationAvailable"], false);
        let mut invalid = body;
        invalid["input"]["spaces"][0]["reductionFactor"] = json!(1.2);
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/transmission/unheated/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(invalid.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["totalReducedConductanceWPerK"], Value::Null);
    }

    #[tokio::test]
    async fn validate_accepts_classified_standalone_heat_pump_without_claiming_calculation() {
        let body = json!({"project": {
            "id": "house", "name": "House", "buildingFunction": "residential",
            "zones": [{"id": "z1", "floorArea": 100.0, "volume": 250.0,
                "surfaces": [{"id": "wall", "area": 50.0, "zoneId": "z1", "windows": []}]}],
            "heatingSystems": [{"id":"boiler-1", "type":"hr107"}],
            "ntaHeatPumps": [{
                "id": "hp-water", "servedZoneIds": ["z1"],
                "source": "surface_water", "sink": "hydronic",
                "drive": "electric_compression", "reversible": true,
                "hybrid": false, "booster": false,
                "performanceEvidence": {"kind": "normative_default", "reference": null},
                "forfaitHeatPumpDraft": {
                    "generatorId":"hp-water", "classificationSourceReference":"design sheet",
                    "scope":"utility_collective_or_over25_kw", "source":"surface_water", "sink":"hydronic",
                    "designSupplyTemperatureC":45.0, "sourceCorrectionFactor":null,
                    "sourceCorrectionReference":null
                },
                "heatingAuxMeasuredDraft": {
                    "generatorId": "hp-water", "generatorSourceReference": "schedule",
                    "measurements": {
                        "standbyElectronicsW": 10, "deliveryPumpDuringCompressorW": 200,
                        "deliveryPumpPrePostW": 90, "pumpPreRunSeconds": 300,
                        "pumpPostRunSeconds": 300, "averageCompressorOnSeconds": 600,
                        "meanCompressorModulation": 0.5, "nominalElectricDriveKw": 2,
                        "measurementSourceReference": "meter", "timingSourceReference": "cycle"
                    },
                    "inputEnergySourceReference": "monthly meter",
                    "months": (1..=12).map(|month| json!({"month": month,
                        "generatorInputElectricityKwh": 100})).collect::<Vec<_>>()
                },
                "auxiliaryComponents": [{
                    "id":"source-pump", "kind":"source_pump", "service":"space_heating",
                    "nominalPowerW":80, "energyCarrier":"electricity",
                    "measurementBoundary":"additional", "evidenceReference":"datasheet-1"
                }],
                "systemLinks": [{"id":"backup", "role":"backup_generator",
                    "targetKind":"heating_system", "targetId":"boiler-1",
                    "evidenceReference":"scheme-1"}]
            }]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/validate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["summary"]["classifiedHeatPumpCount"], 1);
        assert_eq!(result["summary"]["auxiliaryComponentCount"], 1);
        assert_eq!(result["summary"]["systemLinkCount"], 1);
        assert_eq!(result["calculationAvailable"], false);
        assert_eq!(result["issues"][0]["code"], "heat_pump_route_unimplemented");
        assert!(result["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "heating_aux_measured_draft_unverified"));
        assert!(result["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "forfait_heat_pump_draft_unverified"));
    }

    #[tokio::test]
    async fn forfait_cop_draft_uses_explicit_table_scope_without_annual_result() {
        let mut input = json!({
            "generatorId":"hp-air", "classificationSourceReference":"design sheet",
            "scope":"residential_at_most25_kw", "source":"outdoor_air", "sink":"hydronic",
            "designSupplyTemperatureC":35.0,
            "sourceCorrectionFactor":null, "sourceCorrectionReference":null
        });
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/heat-pumps/forfait-cop-draft/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["tableCop"], 2.4);
        assert_eq!(result["annualPerformanceAvailable"], false);
        assert_eq!(result["bengCalculationAvailable"], false);

        input["designSupplyTemperatureC"] = json!(71.0);
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/heat-pumps/forfait-cop-draft/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(result["correctedCop"].is_null());

        input["designSupplyTemperatureC"] = json!(35.0);
        input["rowVariant"] = json!("table_9_28_high_efficiency");
        input["highEfficiencyEvidence"] = json!({
            "productReference":"tested model A", "testReportReference":"lab report p4",
            "testStandardEdition":"NEN-EN 14511-2:2022", "points":[
                {"condition":"a7_wet6_w45", "measuredCop":2.76},
                {"condition":"a7_wet6_w35", "measuredCop":2.86},
                {"condition":"a_minus7_wet_minus8_w45", "measuredCop":1.91}
            ]
        });
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/heat-pumps/forfait-cop-draft/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["tableCop"], 3.35);
        assert_eq!(result["applicabilityVerified"], false);
        input["highEfficiencyEvidence"]["points"][0]["measuredCop"] = json!(2.75);
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/heat-pumps/forfait-cop-draft/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(result["correctedCop"].is_null());
        assert_eq!(
            result["issues"][0]["code"],
            "high_test_cop_below_requirement"
        );
    }

    #[tokio::test]
    async fn forfait_monthly_draft_derives_source_heat_without_enabling_beng() {
        let months = (1..=12)
            .map(|month| json!({"month":month,"energyKwh":1000.0}))
            .collect::<Vec<_>>();
        let mut input = json!({
            "forfait":{
                "generatorId":"hp", "classificationSourceReference":"source design",
                "scope":"residential_at_most25_kw", "source":"collective15_to20_c",
                "sink":"hydronic", "designSupplyTemperatureC":35.0,
                "sourceCorrectionFactor":null, "sourceCorrectionReference":null,
                "sourceTemperatureC":15.0,
                "sourceTemperatureEvidenceReference":"source meter"
            },
            "generatorOutputKwh":months,
            "generatorOutputReference":"heat meter",
            "sourceSystem":"collective_groundwater_surface_or_at_least15_c",
            "sourceSystemReference":"source design"
        });
        let endpoint = "/v1/nta8800/heat-pumps/forfait-monthly-draft/diagnose";
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["status"], "diagnostic_valid");
        assert_eq!(result["monthly"].as_array().unwrap().len(), 12);
        assert!(
            (result["monthly"][0]["generatorInputElectricityKwh"]
                .as_f64()
                .unwrap()
                - (1000.0 / 4.8 - 1000.0 * (1.0 - 1.0 / 4.8) * 0.022))
                .abs()
                < 1e-9
        );
        assert_eq!(result["collectiveSourceHeatDerived"], true);
        assert_eq!(result["bengCalculationAvailable"], false);
        input["generatorOutputKwh"][0]["energyKwh"] = json!(-1.0);
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let invalid: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(invalid["monthly"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn generator_dispatch_endpoint_interpolates_hybrid_months_and_rejects_missing_evidence() {
        let mut input = json!({
            "nodeInputKwh":(1..=12).map(|month| json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
            "nodeInputReference":"node balance",
            "designContext":"new_build",
            "generators":[
                {"id":"hp","class":"heat_pump","classificationReference":"design schedule","nominalThermalPowerKw":4.5,
                 "powerReference":"plate","priorityEfficiency":4.0,"efficiencyReference":"COP"},
                {"id":"boiler","class":"other_boiler","classificationReference":"design schedule","nominalThermalPowerKw":5.5,
                 "powerReference":"plate","priorityEfficiency":0.9,"efficiencyReference":"declaration"}
            ]
        });
        let endpoint = "/v1/nta8800/heating/generator-dispatch-draft/diagnose";
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(
            result["monthly"][0]["generators"][0]["deliveredHeatKwh"],
            810.0
        );
        assert_eq!(
            result["monthly"][4]["generators"][0]["deliveredHeatKwh"],
            990.0
        );
        assert_eq!(result["bengCalculationAvailable"], false);
        input["generators"][0]["powerReference"] = json!("");
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let invalid: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(invalid["status"], "invalid");
        assert!(invalid["monthly"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn hybrid_endpoint_derives_generator_heat_and_rejects_cop_mismatch() {
        let mut input = json!({
            "dispatch": {
                "nodeInputKwh":(1..=12).map(|month| json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
                "nodeInputReference":"node balance", "designContext":"new_build",
                "generators":[
                    {"id":"hp","class":"heat_pump","classificationReference":"design schedule","nominalThermalPowerKw":4.0,"powerReference":"plate",
                     "priorityEfficiency":4.8,"efficiencyReference":"table"},
                    {"id":"boiler","class":"other_boiler","classificationReference":"design schedule","nominalThermalPowerKw":6.0,"powerReference":"plate",
                     "priorityEfficiency":0.95,"efficiencyReference":"table 9.25"}
                ]
            },
            "forfait":{"generatorId":"hp","classificationSourceReference":"source design",
                "scope":"residential_at_most25_kw","source":"collective15_to20_c",
                "sink":"hydronic","designSupplyTemperatureC":35.0,
                "thermalCapacityKw":4.0,"capacitySourceReference":"plate","collectiveBuildingInstallation":false,
                "sourceCorrectionFactor":null,"sourceCorrectionReference":null,
                "sourceTemperatureC":15.0,"sourceTemperatureEvidenceReference":"source meter"},
            "sourceSystem":"collective_groundwater_surface_or_at_least15_c",
            "sourceSystemReference":"source design",
            "boiler":{"generatorId":"boiler","role":"individual_supplementary",
                "location":"inside_thermal_boundary","kind":"hr107","fuel":"natural_gas",
                "averageDesignEmissionTemperatureC":45.0,"emissionCircuit":"direct",
                "equipmentReference":"boiler plate","locationReference":"site inspection",
                "temperatureAndCircuitReference":"heating design","pilotFlamePresent":false},
            "heatPumpAuxiliaryMeasurements":{"generatorId":"hp","generatorSourceReference":"hp plate",
                "measurements":{"standbyElectronicsW":10.0,"deliveryPumpDuringCompressorW":200.0,
                    "deliveryPumpPrePostW":90.0,"pumpPreRunSeconds":300.0,"pumpPostRunSeconds":300.0,
                    "averageCompressorOnSeconds":600.0,"meanCompressorModulation":0.5,
                    "nominalElectricDriveKw":2.0,"measurementSourceReference":"power measurements",
                    "timingSourceReference":"cycle measurements"}},
            "declaredOperatingLimitsPresent":false
        });
        let endpoint = "/v1/nta8800/heat-pumps/hybrid-monthly-draft/diagnose";
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(
            result["heatPump"]["monthly"][0]["generatorOutputKwh"],
            750.0
        );
        assert!(
            (result["heatPump"]["monthly"][0]["generatorInputElectricityKwh"]
                .as_f64()
                .unwrap()
                - (750.0 / 4.8 - 750.0 * (1.0 - 1.0 / 4.8) * 0.022))
                .abs()
                < 1e-9
        );
        assert_eq!(result["boilerInputEnergyAvailable"], true);
        assert_eq!(result["boilerAuxiliaryElectricityAvailable"], true);
        assert_eq!(result["heatPumpAuxiliaryElectricityAvailable"], true);
        assert!(
            (result["heatPumpAuxiliary"]["auxiliary"]["monthlyAuxiliaryElectricityKwh"][0]
                ["electricityKwh"]
                .as_f64()
                .unwrap()
                - (87.6 / 12.0 + 0.19 * 143.1875))
                .abs()
                < 1e-9
        );
        assert!(
            (result["boiler"]["monthly"][0]["auxiliaryElectricityKwh"]
                .as_f64()
                .unwrap()
                - (87.6 / 12.0 + 0.132 * (250.0 / 0.95) / (0.4 * 24.0)))
                .abs()
                < 1e-9
        );
        assert!(
            (result["boiler"]["monthly"][0]["inputNaturalGasKwh"]
                .as_f64()
                .unwrap()
                - 250.0 / 0.95)
                .abs()
                < 1e-9
        );
        input["dispatch"]["generators"][0]["priorityEfficiency"] = json!(5.0);
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let invalid: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(invalid["heatPump"].is_null() && invalid["dispatch"].is_null());
    }

    #[tokio::test]
    async fn boiler_endpoints_use_table_925_and_reject_pilot_flame() {
        let mut boiler = json!({
            "generatorId":"boiler", "role":"individual_supplementary",
            "location":"inside_thermal_boundary", "kind":"hr107", "fuel":"natural_gas",
            "averageDesignEmissionTemperatureC":45.0, "emissionCircuit":"direct",
            "equipmentReference":"boiler plate", "locationReference":"site inspection",
            "temperatureAndCircuitReference":"heating design", "pilotFlamePresent":false
        });
        let endpoint = "/v1/nta8800/boilers/forfait-draft/diagnose";
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":boiler}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["generationEfficiency"], 0.95);
        let monthly = json!({"boiler":boiler,
            "generatorOutputKwh":(1..=12).map(|month| json!({"month":month,"energyKwh":250.0})).collect::<Vec<_>>(),
            "generatorOutputReference":"dispatch fingerprint"});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/boilers/forfait-monthly-draft/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":monthly}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(
            (result["monthly"][0]["inputNaturalGasKwh"].as_f64().unwrap() - 250.0 / 0.95).abs()
                < 1e-9
        );
        assert!(
            result["monthly"][0]["auxiliaryElectricityKwh"]
                .as_f64()
                .unwrap()
                > 10.0
        );
        boiler["pilotFlamePresent"] = json!(true);
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":boiler}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn gas_pump_table_929_endpoint_selects_absorption_without_claiming_gas_use() {
        let mut input = json!({
            "generatorId":"gwp-1", "drive":"absorption", "application":"collective_building",
            "applicationReference":"building installation schedule", "collectiveBuildingInstallation":true,
            "externalHeatSupply":false, "thermalCapacityKw":40.0, "capacityReference":"plate",
            "source":"exhaust_air", "sourceReference":"product schedule",
            "designSupplyTemperatureC":40.0, "designSupplyReference":"heating design"
        });
        let endpoint = "/v1/nta8800/heat-pumps/gas-forfait-cop-draft/diagnose";
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["forfaitCop"], 2.4);
        assert_eq!(result["gasInputEnergyAvailable"], false);
        input["designSupplyTemperatureC"] = json!(55.01);
        let response = app()
            .oneshot(
                Request::post(endpoint)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({"input":input}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(result["forfaitCop"].is_null());
    }

    #[tokio::test]
    async fn gas_pump_aux_endpoint_separates_electricity_and_rejects_solution_pump_with_forfait() {
        let mut input = json!({
            "generatorId":"ga-1", "drive":"absorption", "nominalThermalCapacityKw":20.0,
            "capacityReference":"plate", "standbyElectronicsW":10.0,
            "burnerAuxiliaryWPerKw":1.0, "solutionPumpWPerKw":0.0,
            "coefficientsReference":"draft 9.6.8.2.3", "meanModulation":1.0,
            "modulationReference":"draft 9.6.8.2.3", "buildingShare":1.0,
            "buildingShareReference":"whole building", "forfaitCopUsed":true,
            "monthHoursReference":"hour schedule", "generatorOutputReference":"generator ledger",
            "months":(1..=12).map(|month| json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0})).collect::<Vec<_>>()
        });
        let endpoint = "/v1/nta8800/heat-pumps/gas-aux-draft/diagnose";
        let request = |input: &Value| {
            Request::post(endpoint)
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(
            (value["monthly"][0]["auxiliaryElectricityKwh"]
                .as_f64()
                .unwrap()
                - 8.4)
                .abs()
                < 1e-12
        );
        assert_eq!(value["gasInputEnergyAvailable"], false);
        assert_eq!(value["bengCalculationAvailable"], false);
        input["solutionPumpWPerKw"] = json!(10.0);
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(value["monthly"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn gas_pump_monthly_endpoint_exposes_962_terms_without_carrier_claim() {
        let mut input = json!({
            "forfait":{
                "generatorId":"ga-1", "drive":"absorption", "application":"utility",
                "applicationReference":"office design", "collectiveBuildingInstallation":false,
                "externalHeatSupply":false, "thermalCapacityKw":40.0, "capacityReference":"plate",
                "source":"groundwater_aquifer", "sourceReference":"source design",
                "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design"
            },
            "generatorOutputKwh":(1..=12).map(|month| json!({"month":month,"energyKwh":1000.0})).collect::<Vec<_>>(),
            "generatorOutputReference":"heat meter",
            "sourceSystem":"collective_groundwater_surface_or_at_least15_c",
            "sourceSystemReference":"source invoices"
        });
        let endpoint = "/v1/nta8800/heat-pumps/gas-forfait-monthly-draft/diagnose";
        let request = |input: &Value| {
            Request::post(endpoint)
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        let heat = 1000.0 * (1.0 - 1.0 / 2.1);
        assert!(
            (value["monthly"][0]["equation962InputTermKwh"]
                .as_f64()
                .unwrap()
                - (1000.0 / 2.1 - heat * 0.022))
                .abs()
                < 1e-10
        );
        assert_eq!(value["carrierAllocationAvailable"], false);
        assert_eq!(value["gasInputEnergyAvailable"], false);
        input["sourceSystem"] = json!("collective_ground");
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(value["monthly"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn gas_pump_chain_endpoint_links_heat_and_suppresses_mismatches() {
        let months = (1..=12)
            .map(|month| json!({"month":month,"energyKwh":1000.0}))
            .collect::<Vec<_>>();
        let aux_months = (1..=12)
            .map(|month| json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0}))
            .collect::<Vec<_>>();
        let mut input = json!({
            "monthly":{
                "forfait":{
                    "generatorId":"ga-1", "drive":"absorption", "application":"utility",
                    "applicationReference":"office", "collectiveBuildingInstallation":false,
                    "externalHeatSupply":false, "thermalCapacityKw":20.0, "capacityReference":"plate",
                    "source":"outdoor_air", "sourceReference":"plan", "designSupplyTemperatureC":35.0,
                    "designSupplyReference":"design"
                },
                "generatorOutputKwh":months, "generatorOutputReference":"meter",
                "sourceSystem":"individual", "sourceSystemReference":"plan"
            },
            "auxiliary":{
                "generatorId":"ga-1", "drive":"absorption", "nominalThermalCapacityKw":20.0,
                "capacityReference":"plate", "standbyElectronicsW":10.0,
                "burnerAuxiliaryWPerKw":1.0, "solutionPumpWPerKw":0.0,
                "coefficientsReference":"draft 9.6.8.2.3", "meanModulation":1.0,
                "modulationReference":"draft 9.6.8.2.3", "buildingShare":1.0,
                "buildingShareReference":"whole building", "forfaitCopUsed":true,
                "monthHoursReference":"schedule", "generatorOutputReference":"meter", "months":aux_months
            }
        });
        let endpoint = "/v1/nta8800/heat-pumps/gas-chain-draft/diagnose";
        let request = |input: &Value| {
            Request::post(endpoint)
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(
            value["monthly"][0]["equation962UnallocatedInputTermKwh"],
            625.0
        );
        assert_eq!(value["monthly"][0]["equipmentAuxiliaryElectricityKwh"], 8.4);
        assert_eq!(value["carrierAllocationAvailable"], false);
        input["auxiliary"]["months"][0]["generatorOutputKwh"] = json!(900.0);
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(value["monthly"].as_array().unwrap().is_empty());
        assert_eq!(value["annualEquipmentAuxiliaryElectricityKwh"], Value::Null);
    }

    #[tokio::test]
    async fn collective_gas_source_endpoint_keeps_dh_separate_and_rejects_missing_declaration_evidence(
    ) {
        let months = (1..=12)
            .map(|month| json!({"month":month,"energyKwh":1000.0}))
            .collect::<Vec<_>>();
        let aux_months = (1..=12)
            .map(|month| json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0}))
            .collect::<Vec<_>>();
        let mut input = json!({
            "chain": {
                "monthly": {"forfait": {"generatorId":"ga-1","drive":"absorption","application":"utility",
                    "applicationReference":"office","collectiveBuildingInstallation":false,"externalHeatSupply":false,
                    "thermalCapacityKw":20.0,"capacityReference":"plate","source":"groundwater_aquifer",
                    "sourceReference":"source plan","designSupplyTemperatureC":35.0,"designSupplyReference":"design"},
                    "generatorOutputKwh":months,"generatorOutputReference":"heat meter",
                    "sourceSystem":"collective_groundwater_surface_or_at_least15_c","sourceSystemReference":"invoice"},
                "auxiliary": {"generatorId":"ga-1","drive":"absorption","nominalThermalCapacityKw":20.0,
                    "capacityReference":"plate","standbyElectronicsW":10.0,"burnerAuxiliaryWPerKw":1.0,
                    "solutionPumpWPerKw":0.0,"coefficientsReference":"draft","meanModulation":1.0,
                    "modulationReference":"draft","buildingShare":1.0,"buildingShareReference":"whole building",
                    "forfaitCopUsed":true,"monthHoursReference":"schedule","generatorOutputReference":"heat meter","months":aux_months}
            },
            "sourceTemperatureClass":"below20_c","sourceTemperatureReference":"source design",
            "noQualityDeclarationConfirmed":true,"noQualityDeclarationReference":"register check"
        });
        let endpoint = "/v1/nta8800/heat-pumps/gas-collective-source-draft/diagnose";
        let request = |input: &Value| {
            Request::post(endpoint)
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["sourceEnergyCarrier"], "dh");
        assert_eq!(value["primaryFossilFactor"], 1.45 / 23.0);
        assert_eq!(value["primaryRenewableFactor"], 0.95);
        assert_eq!(value["gasInputEnergyAvailable"], false);
        assert_eq!(value["bengCalculationAvailable"], false);
        input["noQualityDeclarationReference"] = json!("");
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert!(value["monthly"].as_array().unwrap().is_empty());
        assert!(value["annualDeliveredSourceHeatKwh"].is_null());
    }

    #[tokio::test]
    async fn gas_chain_reference_endpoint_requires_complete_metrics_and_never_attests() {
        let monthly = (1..=12)
            .map(|month| json!({"month":month,"energyKwh":1000.0}))
            .collect::<Vec<_>>();
        let aux_monthly = (1..=12)
            .map(|month| json!({"month":month,"hours":730.0,"generatorOutputKwh":1000.0}))
            .collect::<Vec<_>>();
        let mut expected = Vec::new();
        for month in 1..=12 {
            expected.push(json!({"metric":"equation962_unallocated_input_term","month":month,
                "valueKwh":625.0,"absoluteToleranceKwh":1e-9,"calculationBasis":"synthetic arithmetic"}));
            expected.push(json!({"metric":"equipment_auxiliary_electricity","month":month,
                "valueKwh":8.4,"absoluteToleranceKwh":1e-9,"calculationBasis":"synthetic arithmetic"}));
        }
        expected.push(json!({"metric":"annual_equipment_auxiliary_electricity","month":null,
            "valueKwh":100.8,"absoluteToleranceKwh":1e-9,"calculationBasis":"synthetic arithmetic"}));
        let mut case = json!({
            "caseId":"synthetic","normVersion":nta8800_core::TARGET_NORM_VERSION,
            "source":{"publisher":"internal test","documentId":"synthetic hand calculation",
                "edition":"draft","usePermission":"internal test","independentReviewer":"unverified"},
            "input":{
                "monthly":{"forfait":{"generatorId":"ga-1","drive":"absorption","application":"utility",
                    "applicationReference":"office","collectiveBuildingInstallation":false,
                    "externalHeatSupply":false,"thermalCapacityKw":20.0,"capacityReference":"plate",
                    "source":"outdoor_air","sourceReference":"plan","designSupplyTemperatureC":35.0,
                    "designSupplyReference":"design"},"generatorOutputKwh":monthly,
                    "generatorOutputReference":"meter","sourceSystem":"individual","sourceSystemReference":"plan"},
                "auxiliary":{"generatorId":"ga-1","drive":"absorption","nominalThermalCapacityKw":20.0,
                    "capacityReference":"plate","standbyElectronicsW":10.0,"burnerAuxiliaryWPerKw":1.0,
                    "solutionPumpWPerKw":0.0,"coefficientsReference":"draft","meanModulation":1.0,
                    "modulationReference":"draft","buildingShare":1.0,"buildingShareReference":"whole building",
                    "forfaitCopUsed":true,"monthHoursReference":"schedule","generatorOutputReference":"meter",
                    "months":aux_monthly}},
            "expected":expected
        });
        let request = |case: &Value| {
            Request::post("/v1/nta8800/reference/gas-chain-diagnostic/compare")
                .header("content-type", "application/json")
                .body(Body::from(json!({"case":case}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&case)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["status"], "compared_pass");
        assert_eq!(result["metrics"].as_array().unwrap().len(), 25);
        assert_eq!(result["referenceVerified"], false);
        case["expected"][0]["valueKwh"] = json!(626.0);
        let response = app().oneshot(request(&case)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["status"], "compared_fail");
        case["expected"].as_array_mut().unwrap().pop();
        let response = app().oneshot(request(&case)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["status"], "invalid_case");
        assert!(result["metrics"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn gas_pump_residential_table_927_requires_sourced_correction() {
        let mut input = json!({
            "generatorId":"gwp-home", "drive":"gas_engine",
            "application":"residential_collective_at_most25_kw",
            "applicationReference":"building schedule", "collectiveBuildingInstallation":true,
            "externalHeatSupply":false, "thermalCapacityKw":25.0, "capacityReference":"plate",
            "source":"ground", "sourceReference":"ground loop plan",
            "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design",
            "sourceCorrectionFactor":1.1, "sourceCorrectionReference":"appendix V calculation"
        });
        let endpoint = "/v1/nta8800/heat-pumps/gas-forfait-cop-draft/diagnose";
        let request = |input: &Value| {
            Request::post(endpoint)
                .header("content-type", "application/json")
                .body(Body::from(json!({"input":input}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["table"], "9.27");
        assert_eq!(value["forfaitCop"], 1.3);
        assert!((value["correctedCop"].as_f64().unwrap() - 1.43).abs() < 1e-12);
        assert_eq!(value["gasInputEnergyAvailable"], false);
        input["sourceCorrectionReference"] = Value::Null;
        let response = app().oneshot(request(&input)).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn validate_saved_gas_pump_table_input_without_enabling_calculation() {
        let project = json!({
            "id":"gas-project", "name":"Office gas pump", "buildingFunction":"office",
            "zones":[{"id":"z1","floorArea":100.0,"volume":250.0,
                "surfaces":[{"id":"wall","area":50.0,"zoneId":"z1","windows":[]}]}],
            "ntaHeatPumps":[{
                "id":"gwp-1", "source":"outdoor_air", "sink":"hydronic",
                "drive":"gas_engine", "reversible":false, "hybrid":false, "booster":false,
                "performanceEvidence":{"kind":"normative_default","reference":null},
                "gasHeatPumpForfaitDraft":{
                    "generatorId":"gwp-1", "drive":"gas_engine", "application":"utility",
                    "applicationReference":"office schedule", "collectiveBuildingInstallation":false,
                    "externalHeatSupply":false, "thermalCapacityKw":20.0, "capacityReference":"plate",
                    "source":"outdoor_air", "sourceReference":"system design",
                    "designSupplyTemperatureC":35.0, "designSupplyReference":"heating design"
                }
            }]
        });
        let request = |project: Value| {
            Request::post("/v1/nta8800/validate")
                .header("content-type", "application/json")
                .body(Body::from(json!({"project":project}).to_string()))
                .unwrap()
        };
        let response = app().oneshot(request(project.clone())).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["status"], "structurally_valid");
        assert_eq!(value["calculationAvailable"], false);
        assert!(value["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "gas_heat_pump_forfait_draft_unverified"));
        let mut mismatch = project;
        mismatch["ntaHeatPumps"][0]["drive"] = json!("absorption");
        let response = app().oneshot(request(mismatch)).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let value: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(value["status"], "invalid");
        assert!(value["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "gas_heat_pump_forfait_draft_scope_invalid"));
    }

    #[tokio::test]
    async fn validate_rejects_misspelled_heat_pump_fields() {
        let body = json!({"project": {
            "id": "house", "name": "House", "buildingFunction": "residential",
            "zones": [{"id": "z1", "floorArea": 100.0, "volume": 250.0,
                "surfaces": [{"id": "wall", "area": 50.0, "zoneId": "z1", "windows": []}]}],
            "ntaHeatPumps": [{
                "id": "hp", "source": "outdoor_air", "sink": "hydronic",
                "drive": "electric_compression", "reversible": false,
                "hybrid": false, "booster": false,
                "performanceEvidence": {"kind": "normative_default", "reference": null},
                "performancePoint": []
            }]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/validate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["error"], "invalid_project_shape");
        assert!(result["message"]
            .as_str()
            .unwrap()
            .contains("performancePoint"));
    }

    #[tokio::test]
    async fn reference_audit_does_not_promote_incomplete_case_to_verified() {
        let body = json!({"case": {
            "caseId": "diagnostic", "normVersion": "NTA 8800:2025+C1:2026",
            "project": {"id":"house", "name":"House", "buildingFunction":"residential",
                "zones":[{"id":"z", "floorArea":100, "volume":250,
                    "surfaces":[{"id":"s", "area":50, "zoneId":"z", "windows":[]}]}]},
            "source": {"publisher":"", "documentId":"", "edition":"",
                "usePermission":"", "independentReviewer":""},
            "expected": []
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/reference/audit")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["manifestComplete"], false);
        assert_eq!(result["referenceVerified"], false);
        assert_eq!(result["calculationAvailable"], false);
        assert!(result["manifestFingerprint"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
    }

    #[tokio::test]
    async fn direct_transmission_endpoint_returns_diagnostic_without_enabling_beng() {
        let body = json!({"input": {
            "elements": [{"id":"wall", "areaM2":10, "uValueWPerM2k":0.2,
                "sourceReference":"drawing-1"}],
            "linearBridges": [{"id":"edge", "lengthM":3, "psiWPerMk":0.05,
                "sourceReference":"detail-1"}]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/transmission/direct/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["status"], "input_valid");
        assert_eq!(result["totalDirectConductanceWPerK"], 2.15);
        assert_eq!(result["referenceVerified"], false);
        assert_eq!(result["bengCalculationAvailable"], false);
    }

    #[tokio::test]
    async fn direct_transmission_endpoint_suppresses_number_for_invalid_u_value() {
        let body = json!({"input": {
            "elements": [{"id":"wall", "areaM2":10, "uValueWPerM2k":0,
                "sourceReference":"drawing-1"}]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/transmission/direct/diagnose")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["status"], "invalid");
        assert!(result["totalDirectConductanceWPerK"].is_null());
        assert_eq!(result["issues"][0]["code"], "direct_u_invalid");
    }

    #[tokio::test]
    async fn direct_diagnostic_comparison_reports_difference_without_attesting() {
        let body = json!({"case": {
            "caseId":"diagnostic-case", "normVersion":"NTA 8800:2025+C1:2026",
            "source":{"publisher":"Example", "documentId":"manual-1", "edition":"2026",
                "usePermission":"internal", "independentReviewer":"Reviewer"},
            "input":{"elements":[{"id":"wall", "areaM2":10, "uValueWPerM2k":0.2,
                "sourceReference":"drawing-1"}]},
            "expected":[
                {"path":"elementConductanceWPerK", "value":2, "unit":"W/K",
                    "absoluteTolerance":0, "calculationBasis":"Manual"},
                {"path":"linearBridgeConductanceWPerK", "value":0, "unit":"W/K",
                    "absoluteTolerance":0, "calculationBasis":"Manual"},
                {"path":"pointBridgeConductanceWPerK", "value":0, "unit":"W/K",
                    "absoluteTolerance":0, "calculationBasis":"Manual"},
                {"path":"totalDirectConductanceWPerK", "value":3, "unit":"W/K",
                    "absoluteTolerance":0.01, "calculationBasis":"Manual"}
            ]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/reference/direct-diagnostic/compare")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["status"], "compared_fail");
        assert_eq!(result["allMetricsWithinTolerance"], false);
        assert_eq!(result["referenceVerified"], false);
        assert_eq!(result["bengCalculationAvailable"], false);
        assert!(result["caseFingerprint"]
            .as_str()
            .unwrap()
            .starts_with("sha256:"));
    }

    #[tokio::test]
    async fn direct_diagnostic_comparison_rejects_difference_overflow() {
        let body = json!({"case": {
            "caseId":"overflow", "normVersion":"NTA 8800:2025+C1:2026",
            "source":{"publisher":"Example", "documentId":"manual-1", "edition":"2026",
                "usePermission":"internal", "independentReviewer":"Reviewer"},
            "input":{"elements":[{"id":"wall", "areaM2":1e308, "uValueWPerM2k":1,
                "sourceReference":"drawing-1"}]},
            "expected":[
                {"path":"elementConductanceWPerK", "value":-1e308, "unit":"W/K",
                    "absoluteTolerance":0, "calculationBasis":"Manual"},
                {"path":"linearBridgeConductanceWPerK", "value":0, "unit":"W/K",
                    "absoluteTolerance":0, "calculationBasis":"Manual"},
                {"path":"pointBridgeConductanceWPerK", "value":0, "unit":"W/K",
                    "absoluteTolerance":0, "calculationBasis":"Manual"},
                {"path":"totalDirectConductanceWPerK", "value":-1e308, "unit":"W/K",
                    "absoluteTolerance":0, "calculationBasis":"Manual"}
            ]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/reference/direct-diagnostic/compare")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["status"], "invalid_case");
        assert!(result["metrics"].as_array().unwrap().is_empty());
        assert!(result["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "metric_difference_overflow"));
    }

    #[tokio::test]
    async fn project_validation_exposes_only_explicit_outdoor_diagnostic() {
        let body = json!({"project": {
            "id":"p", "name":"House", "buildingFunction":"residential",
            "constructions":[{"id":"c", "rcValue":5, "uValue":0.2, "layers":[]}],
            "zones":[{"id":"z", "floorArea":100, "volume":250,
                "thermalBridges":[{"id":"b", "length":3, "psiValue":0.05,
                    "zoneId":"z", "thermalBoundary":"outdoor"}],
                "pointThermalBridges":[{"id":"p", "chiValue":0.04, "zoneId":"z",
                    "thermalBoundary":"outdoor", "sourceReference":"detail-1"}],
                "pointBridgeInventoryComplete":true,
                "surfaces":[{"id":"s", "area":10, "constructionId":"c",
                    "zoneId":"z", "thermalBoundary":"outdoor", "windows":[
                        {"id":"w", "area":2, "uValue":1.1, "gValue":0.4, "surfaceId":"s"}
                    ]}]
            }]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/validate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["summary"]["thermalBoundaries"]["complete"], true);
        assert_eq!(
            result["summary"]["directOutdoorDiagnostic"]["totalDirectConductanceWPerK"],
            3.99
        );
        assert_eq!(
            result["summary"]["directOutdoorDiagnostic"]["pointBridgeConductanceWPerK"],
            0.04
        );
        assert_eq!(
            result["summary"]["directOutdoorDiagnostic"]["referenceVerified"],
            false
        );
        assert_eq!(result["calculationAvailable"], false);
    }

    #[tokio::test]
    async fn validate_exhaust_air_source_link_without_claiming_a_calculation() {
        let body = json!({"project": {
            "id":"p", "name":"House", "buildingFunction":"residential",
            "zones":[{"id":"z", "floorArea":100, "volume":250,
                "surfaces":[{"id":"s", "area":10, "zoneId":"z", "windows":[]}]}],
            "ventilationSystems":[{"id":"vent-1", "type":"type_d"}],
            "ntaHeatPumps":[{"id":"hp", "servedZoneIds":["z"], "source":"exhaust_air",
                "sink":"hydronic", "drive":"electric_compression", "reversible":false,
                "hybrid":false, "booster":false,
                "performanceEvidence":{"kind":"normative_default", "reference":null},
                "systemLinks":[{"id":"source", "role":"source_ventilation",
                    "targetKind":"ventilation_system", "targetId":"vent-1",
                    "evidenceReference":"drawing-1"}]}]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/validate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["status"], "structurally_valid");
        assert_eq!(result["summary"]["systemLinkCount"], 1);
        assert_eq!(result["calculationAvailable"], false);
    }

    #[tokio::test]
    async fn validate_exposes_only_an_instantaneous_heat_pump_power_ratio() {
        let body = json!({"project": {
            "id":"p", "name":"House", "buildingFunction":"residential",
            "zones":[{"id":"z", "floorArea":100, "volume":250,
                "surfaces":[{"id":"s", "area":10, "zoneId":"z", "windows":[]}]}],
            "ntaHeatPumps":[{"id":"hp", "servedZoneIds":["z"],
                "source":"groundwater", "sink":"hydronic", "drive":"electric_compression",
                "reversible":false, "hybrid":false, "booster":false,
                "performanceEvidence":{"kind":"normative_default", "reference":null},
                "performancePoints":[{"id":"p1", "service":"space_heating",
                    "sourceTemperatureC":10, "sinkTemperatureC":35,
                    "usefulCapacityKw":6, "inputPowerKw":2,
                    "inputEnergyCarrier":"electricity", "testReference":"lab-1"}]}]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/validate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        let point = &result["summary"]["performancePointDiagnostics"][0];
        assert_eq!(point["heatPumpId"], "hp");
        assert_eq!(point["pointId"], "p1");
        assert_eq!(point["instantaneousUsefulToInputRatio"], 3.0);
        assert_eq!(point["referenceVerified"], false);
        assert_eq!(point["annualPerformanceAvailable"], false);
        assert_eq!(result["calculationAvailable"], false);
    }

    #[tokio::test]
    async fn validate_preserves_historical_declared_dhw_test_data_without_annual_result() {
        for (fixture, first_ratio) in [
            (
                include_str!("../../../training-data/bcrg-20240123gk-dhw-test-input.json"),
                5.865 / 2.520,
            ),
            (
                include_str!("../../../training-data/bcrg-20260044gk-dhw-test-input.json"),
                6.13 / 2.269,
            ),
            (
                include_str!("../../../training-data/bcrg-20260143gg-dhw-test-input.json"),
                5.876 / 1.644,
            ),
        ] {
            let pump: Value = serde_json::from_str(fixture).unwrap();
            let body = json!({"project": {
                "id":"p", "name":"House", "buildingFunction":"residential",
                "zones":[{"id":"z", "floorArea":100, "volume":250,
                    "surfaces":[{"id":"s", "area":10, "zoneId":"z", "windows":[]}]}],
                "ntaHeatPumps":[pump]
            }});
            let response = app()
                .oneshot(
                    Request::post("/v1/nta8800/validate")
                        .header("content-type", "application/json")
                        .body(Body::from(body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let result: Value =
                serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                    .unwrap();
            assert_eq!(result["status"], "structurally_valid");
            assert_eq!(
                result["summary"]["dhwTestDiagnostics"][0]["tapProfile"],
                "M"
            );
            let ratio = result["summary"]["dhwTestDiagnostics"][0]["declaredUsefulToInputRatio"]
                .as_f64()
                .unwrap();
            assert!((ratio - first_ratio).abs() < 1e-12);
            assert_eq!(
                result["summary"]["dhwTestDiagnostics"][0]["referenceVerified"],
                false
            );
            assert_eq!(
                result["summary"]["dhwTestDiagnostics"][0]["annualPerformanceAvailable"],
                false
            );
            assert_eq!(result["calculationAvailable"], false);
            assert!(result["issues"]
                .as_array()
                .unwrap()
                .iter()
                .any(|issue| issue["code"] == "dhw_declaration_norm_edition_unverified"));
        }
    }

    #[tokio::test]
    async fn validate_preserves_hybrid_shutoff_limits_without_switching_energy() {
        let pump: Value = serde_json::from_str(include_str!(
            "../../../training-data/bcrg-20240234gk-hybrid-limit-input.json"
        ))
        .unwrap();
        let body = json!({"project": {
            "id":"p", "name":"House", "buildingFunction":"residential",
            "zones":[{"id":"z", "floorArea":100, "volume":250,
                "surfaces":[{"id":"s", "area":10, "zoneId":"z", "windows":[]}]}],
            "ntaHeatPumps":[pump]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/validate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(result["status"], "structurally_valid");
        assert_eq!(result["calculationAvailable"], false);
        assert!(result["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "heat_pump_operating_limits_unimplemented"));
        assert!(result["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["code"] == "operating_limits_norm_edition_unverified"));
    }

    #[tokio::test]
    async fn calculate_rejects_malformed_linked_ventilation_system() {
        let body = json!({"project": {
            "id":"p", "name":"House", "buildingFunction":"residential",
            "zones":[{"id":"z", "floorArea":100, "volume":250,
                "surfaces":[{"id":"s", "area":10, "zoneId":"z", "windows":[]}]}],
            "ventilationSystems":[{"id":"vent-1"}],
            "ntaHeatPumps":[{"id":"hp", "servedZoneIds":["z"], "source":"exhaust_air",
                "sink":"hydronic", "drive":"electric_compression", "reversible":false,
                "hybrid":false, "booster":false,
                "performanceEvidence":{"kind":"normative_default", "reference":null},
                "systemLinks":[{"id":"source", "role":"source_ventilation",
                    "targetKind":"ventilation_system", "targetId":"vent-1",
                    "evidenceReference":"drawing-1"}]}]
        }});
        let response = app()
            .oneshot(
                Request::post("/v1/nta8800/calculate")
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let result: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(result["error"], "invalid_project_input");
        assert!(result["assessment"]["issues"]
            .as_array()
            .unwrap()
            .iter()
            .any(|issue| issue["code"] == "system_type_required"));
        assert_eq!(result["assessment"]["calculationAvailable"], false);
    }
}
