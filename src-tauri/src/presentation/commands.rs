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
use crate::application::ports::websocket::WebSocketPort;
use crate::infrastructure::http::reqwest_adapter::ReqwestHttpClientAdapter;
use crate::domain::models::{
    HttpRequest, Environment, GlobalVariables, Collection, CollectionItem,
    CollectionRunReport, MockRule, MockServerStatus,
    LoadTestConfig, LoadTestReport, MonitorDefinition,
    WorkspaceMember, MemberRole, SendRequestOutput
};
use crate::domain::errors::{AppError, DomainError};
use std::collections::HashMap;
use std::sync::Arc;

#[tauri::command]
pub async fn send_request(
    request: HttpRequest,
    environment: Environment,
    globals: GlobalVariables,
    session_vars: HashMap<String, String>,
    use_case: State<'_, SendRequestUseCase>,
) -> Result<SendRequestOutput, AppError> {
    let (response, updated_environment, updated_globals, updated_session) = use_case
        .execute(request, &environment, &globals, &session_vars)
        .await
        .map_err(AppError::from)?;

    Ok(SendRequestOutput {
        response,
        environment: updated_environment,
        globals: updated_globals,
        session_vars: updated_session,
    })
}

#[tauri::command]
pub async fn run_collection(
    items: Vec<CollectionItem>,
    environment: Environment,
    globals: GlobalVariables,
    session_vars: HashMap<String, String>,
    use_case: State<'_, RunCollectionUseCase>,
) -> Result<CollectionRunReport, AppError> {
    use_case.execute(items, &environment, &globals, &session_vars)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn ai_generate_tests(
    url: String,
    response_body: String,
    use_case: State<'_, AITasksUseCase>,
) -> Result<String, String> {
    use_case.generate_tests(&url, &response_body)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ai_explain_response(
    response_body: String,
    use_case: State<'_, AITasksUseCase>,
) -> Result<String, String> {
    use_case.explain_response(&response_body)
        .await
        .map_err(|e| e.to_string())
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
    Ok(use_case.get_server_status().await)
}

#[tauri::command]
pub async fn generate_js_code(
    request: HttpRequest,
    target: String,
    use_case: State<'_, GenerateCodeUseCase>,
) -> Result<String, AppError> {
    if target == "node" {
        Ok(use_case.generate_node_fetch(&request))
    } else {
        Ok(use_case.generate_js_fetch(&request))
    }
}

#[tauri::command]
pub async fn import_openapi(
    content: String,
    workspace_path: String,
    use_case: State<'_, ImportUseCase>,
) -> Result<Collection, AppError> {
    use_case.import_openapi(&content, &workspace_path).map_err(AppError::from)
}

#[tauri::command]
pub async fn read_file_text(
    path: String,
) -> Result<String, AppError> {
    let lower_path = path.to_lowercase();
    if !lower_path.ends_with(".json") && !lower_path.ends_with(".yaml") && !lower_path.ends_with(".yml") {
        return Err(AppError::persistence_error(
            "Access denied: only JSON or YAML files are permitted for import.",
        ));
    }
    std::fs::read_to_string(path).map_err(|e| AppError::persistence_error(e.to_string()))
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
pub async fn ws_connect(
    id: String,
    url: String,
    adapter: State<'_, Arc<dyn WebSocketPort>>,
) -> Result<(), String> {
    adapter.connect(id, url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ws_send(
    id: String,
    message: String,
    adapter: State<'_, Arc<dyn WebSocketPort>>,
) -> Result<(), String> {
    adapter.send(id, message).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ws_disconnect(
    id: String,
    adapter: State<'_, Arc<dyn WebSocketPort>>,
) -> Result<(), String> {
    adapter.disconnect(id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cookies(
    url: String,
    http_client: State<'_, Arc<ReqwestHttpClientAdapter>>,
) -> Result<String, AppError> {
    use reqwest::cookie::CookieStore;

    let parsed_url = reqwest::Url::parse(&url)
        .map_err(|e| AppError::from(DomainError::InvalidUrl(e.to_string())))?;
    let cookie_string = match http_client.jar.cookies(&parsed_url) {
        Some(value) => value.to_str().unwrap_or_default().to_string(),
        None => String::new(),
    };
    Ok(cookie_string)
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

// --- Script library management (Phase 19) ---

use crate::domain::models::ScriptLibraryInfo;
use crate::infrastructure::persistence::fs_script_settings_repository::FsScriptSettingsRepository;
use crate::infrastructure::scripting::libraries::registry;
use crate::infrastructure::scripting::quickjs_runner::QuickJsScriptRunner;

fn build_library_list(workspace_path: &str) -> Vec<ScriptLibraryInfo> {
    let repository = FsScriptSettingsRepository::new();
    let disabled = repository.load_disabled(workspace_path);
    registry()
        .iter()
        .map(|library| ScriptLibraryInfo {
            name: library.name.to_string(),
            version: library.version.to_string(),
            description: library.description.to_string(),
            enabled: !disabled.iter().any(|name| name == library.name),
        })
        .collect()
}

#[tauri::command]
pub fn configure_script_engine(
    workspace_path: String,
    engine: State<'_, Arc<QuickJsScriptRunner>>,
) -> Result<(), AppError> {
    engine.set_settings_dir(Some(workspace_path));
    Ok(())
}

#[tauri::command]
pub fn list_script_libraries(workspace_path: String) -> Vec<ScriptLibraryInfo> {
    build_library_list(&workspace_path)
}

#[tauri::command]
pub fn set_script_library_enabled(
    workspace_path: String,
    name: String,
    enabled: bool,
) -> Result<Vec<ScriptLibraryInfo>, AppError> {
    if !registry().iter().any(|library| library.name == name) {
        return Err(AppError {
            code: "VALIDATION_ERROR".to_string(),
            message: format!("Unknown script library '{}'", name),
        });
    }
    let repository = FsScriptSettingsRepository::new();
    let mut disabled = repository.load_disabled(&workspace_path);
    if enabled {
        disabled.retain(|disabled_name| disabled_name != &name);
    } else if !disabled.contains(&name) {
        disabled.push(name.clone());
    }
    repository
        .save_disabled(&workspace_path, &disabled)
        .map_err(AppError::from)?;
    Ok(build_library_list(&workspace_path))
}
