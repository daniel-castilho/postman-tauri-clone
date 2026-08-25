// src-tauri/src/presentation/commands.rs
use tauri::State;
use crate::application::commands::send_request::SendRequestUseCase;
use crate::application::commands::run_collection::RunCollectionUseCase;
use crate::application::commands::ai_tasks::AITasksUseCase;
use crate::application::commands::mock_server_tasks::MockServerUseCase;
use crate::application::commands::generate_code::GenerateCodeUseCase;
use crate::application::commands::import_tasks::ImportUseCase;
use crate::application::commands::docs_tasks::DocsUseCase;
use crate::application::commands::load_test::LoadTestUseCase;
use crate::application::commands::monitor_tasks::MonitorUseCase;
use crate::application::commands::sync_tasks::SyncUseCase;
use crate::domain::models::{
    HttpRequest, HttpResponse, Environment, GlobalVariables, Collection, 
    AIRequest, AIResponse, MockRule, MockServerStatus, 
    LoadTestConfig, LoadTestReport, MonitorDefinition, MonitorReport,
    WorkspaceMember, MemberRole
};
use crate::domain::errors::AppError;
use std::collections::HashMap;

#[tauri::command]
pub async fn send_request(
    request: HttpRequest,
    environment: Environment,
    globals: GlobalVariables,
    session_vars: HashMap<String, String>,
    use_case: State<'_, SendRequestUseCase>,
) -> Result<(HttpResponse, Environment, GlobalVariables, HashMap<String, String>), AppError> {
    use_case.execute(request, &environment, &globals, &session_vars)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn run_collection(
    collection: Collection,
    environment: Environment,
    globals: GlobalVariables,
    use_case: State<'_, RunCollectionUseCase>,
) -> Result<Vec<HttpResponse>, AppError> {
    use_case.execute(collection, environment, globals)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn explain_request(
    request: AIRequest,
    use_case: State<'_, AITasksUseCase>,
) -> Result<AIResponse, AppError> {
    use_case.explain_request(request).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn start_mock_server(
    port: u16,
    rules: Vec<MockRule>,
    use_case: State<'_, MockServerUseCase>,
) -> Result<(), AppError> {
    use_case.start_server(port, rules).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn stop_mock_server(
    use_case: State<'_, MockServerUseCase>,
) -> Result<(), AppError> {
    use_case.stop_server().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_mock_server_status(
    use_case: State<'_, MockServerUseCase>,
) -> Result<MockServerStatus, AppError> {
    Ok(use_case.get_status().await)
}

#[tauri::command]
pub async fn generate_js_code(
    request: HttpRequest,
    target: String,
    use_case: State<'_, GenerateCodeUseCase>,
) -> Result<String, AppError> {
    use_case.generate_js(request, &target).map_err(AppError::from)
}

#[tauri::command]
pub async fn import_openapi(
    content: String,
    use_case: State<'_, ImportUseCase>,
) -> Result<Collection, AppError> {
    use_case.import_openapi(content).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn read_file_text(
    path: String,
) -> Result<String, AppError> {
    let lower_path = path.to_lowercase();
    if !lower_path.ends_with(".json") && !lower_path.ends_with(".yaml") && !lower_path.ends_with(".yml") {
        return Err(AppError::Persistence("Access denied: only JSON or YAML files are permitted for import.".to_string()));
    }
    std::fs::read_to_string(path).map_err(|e| AppError::Persistence(e.to_string()))
}

#[tauri::command]
pub fn generate_docs(
    collection: Collection,
    use_case: State<'_, DocsUseCase>,
) -> Result<String, AppError> {
    use_case.generate_markdown(&collection).map_err(AppError::from)
}

#[tauri::command]
pub async fn run_load_test(
    request: HttpRequest,
    config: LoadTestConfig,
    environment: Environment,
    globals: GlobalVariables,
    use_case: State<'_, LoadTestUseCase>,
) -> Result<LoadTestReport, AppError> {
    use_case.execute(request, config, environment, globals).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn start_monitor(
    monitor: MonitorDefinition,
    use_case: State<'_, MonitorUseCase>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use_case.start_monitor(monitor, app_handle).await;
    Ok(())
}

#[tauri::command]
pub async fn stop_monitor(
    monitor_id: String,
    use_case: State<'_, MonitorUseCase>,
) -> Result<(), String> {
    use_case.stop_monitor(&monitor_id).await;
    Ok(())
}

#[tauri::command]
pub async fn invite_user(
    email: String,
    role: MemberRole,
    use_case: State<'_, SyncUseCase>,
) -> Result<WorkspaceMember, String> {
    use_case.invite_user(email, role).await
}

#[tauri::command]
pub async fn get_members(
    use_case: State<'_, SyncUseCase>,
) -> Result<Vec<WorkspaceMember>, String> {
    Ok(use_case.get_members().await)
}

#[tauri::command]
pub async fn sync_resource_change(
    resource_type: String,
    resource_id: String,
    operation: String,
    data: String,
    use_case: State<'_, SyncUseCase>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use_case.push_change(app_handle, resource_type, resource_id, operation, data).await;
    Ok(())
}
