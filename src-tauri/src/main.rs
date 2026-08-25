#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod domain;
mod application;
mod infrastructure;
mod presentation;

// ...
use tauri::Manager;
use application::commands::send_request::SendRequestUseCase;
use application::commands::workspace::WorkspaceUseCase;
use infrastructure::http::reqwest_adapter::ReqwestHttpClientAdapter;
use infrastructure::environment::variable_resolver_adapter::RealVariableResolver;
use infrastructure::scripting::quickjs_runner::QuickJsScriptRunner;
use infrastructure::persistence::fs_collection_repository::FsCollectionRepository;
use infrastructure::persistence::fs_design_repository::FsDesignRepository;

use std::sync::Arc;
use application::commands::run_collection::RunCollectionUseCase;
use application::commands::design_tasks::DesignUseCase;
use application::ports::websocket::WebSocketPort;

use infrastructure::websocket::tungstenite_adapter::TungsteniteWebSocketAdapter;
use infrastructure::ai::gemini_adapter::GeminiAIAdapter;
use application::commands::ai_tasks::AITasksUseCase;
use infrastructure::mock::axum_adapter::AxumMockServerAdapter;
use application::commands::mock_server_tasks::MockServerUseCase;

use infrastructure::codegen::template_adapter::TemplateCodeGeneratorAdapter;
use application::commands::generate_code::GenerateCodeUseCase;
use infrastructure::importers::openapi_adapter::OpenApiImporterAdapter;
use application::commands::import_tasks::ImportUseCase;
use infrastructure::docs::markdown_adapter::MarkdownDocsAdapter;
use application::commands::docs_tasks::DocsUseCase;
use application::commands::load_test::LoadTestUseCase;
use application::commands::monitor_tasks::MonitorUseCase;
use application::commands::sync_tasks::SyncUseCase;
use infrastructure::grpc::mock_adapter::MockGrpcClientAdapter;

fn main() {
    // Headless mode: recognized CLI subcommands run without booting the
    // desktop shell, reusing the same application layer and adapters.
    let argv: Vec<String> = std::env::args().collect();
    if let Some(invocation) = presentation::cli::headless_command(&argv) {
        std::process::exit(presentation::cli::invoke(invocation));
    }

    let http_client = Arc::new(ReqwestHttpClientAdapter::new());
    let variable_resolver = Arc::new(RealVariableResolver::new());
    let script_runner = Arc::new(QuickJsScriptRunner::new());
    let grpc_client = Arc::new(MockGrpcClientAdapter);
    let send_request_usecase = SendRequestUseCase::new(http_client.clone(), grpc_client.clone(), variable_resolver.clone(), script_runner.clone());
    let run_collection_usecase = RunCollectionUseCase::new(send_request_usecase.clone());

    let ai_key = std::env::var("GEMINI_API_KEY").unwrap_or_else(|_| "MOCK_KEY".to_string());
    let ai_adapter = Arc::new(GeminiAIAdapter::new(ai_key));
    let ai_tasks_usecase = AITasksUseCase::new(ai_adapter);

    let mock_adapter = Arc::new(AxumMockServerAdapter::new());
    let mock_server_usecase = MockServerUseCase::new(mock_adapter);

    let codegen_adapter = Arc::new(TemplateCodeGeneratorAdapter::new());
    let generate_code_usecase = GenerateCodeUseCase::new(codegen_adapter);

    let fs_collection_repo = Arc::new(FsCollectionRepository::new());
    let workspace_usecase = WorkspaceUseCase::new(Box::new(FsCollectionRepository::new()))
        .expect("Failed to initialize workspace use case");
    
    let import_adapter = Arc::new(OpenApiImporterAdapter::new());
    let import_usecase = ImportUseCase::new(import_adapter, fs_collection_repo.clone());
    
    let docs_adapter = Arc::new(MarkdownDocsAdapter::new());
    let docs_usecase = DocsUseCase::new(docs_adapter);

    let load_test_usecase = LoadTestUseCase::new(http_client.clone(), variable_resolver.clone());
    let monitor_usecase = MonitorUseCase::new(http_client.clone());
    let sync_usecase = SyncUseCase::new();

    let design_usecase = DesignUseCase::new(Box::new(FsDesignRepository));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            println!("🚀 Tyny Pulse - Clean Architecture iniciado!");
            let ws_adapter: Arc<dyn WebSocketPort> =
                Arc::new(TungsteniteWebSocketAdapter::new(app.handle().clone()));
            app.manage(ws_adapter);
            app.manage(http_client.clone());
            Ok(())
        })
        .manage(send_request_usecase)
        .manage(run_collection_usecase)
        .manage(workspace_usecase)
        .manage(ai_tasks_usecase)
        .manage(mock_server_usecase)
        .manage(generate_code_usecase)
        .manage(import_usecase)
        .manage(docs_usecase)
        .manage(load_test_usecase)
        .manage(monitor_usecase)
        .manage(sync_usecase)
        .manage(design_usecase)
        .invoke_handler(tauri::generate_handler![
            presentation::commands::send_request,
            presentation::commands::run_collection,
            presentation::commands::ws_connect,
            presentation::commands::ws_send,
            presentation::commands::ws_disconnect,
            presentation::commands::get_cookies,
            presentation::commands::ai_generate_tests,
            presentation::commands::ai_explain_response,
            presentation::commands::start_mock_server,
            presentation::commands::stop_mock_server,
            presentation::commands::get_mock_server_status,
            presentation::commands::generate_js_code,
            presentation::commands::import_openapi,
            presentation::commands::read_file_text,
            presentation::commands::generate_docs,
            presentation::commands::run_load_test,
            presentation::commands::start_monitor,
            presentation::commands::stop_monitor,
            presentation::commands::invite_user,
            presentation::commands::get_members,
            presentation::commands::sync_resource_change,
            presentation::collections::load_collections,
            presentation::collections::save_collection,
            presentation::collections::delete_collection,
            presentation::collections::load_environments,
            presentation::collections::save_environments,
            presentation::collections::load_globals,
            presentation::collections::save_globals,
            presentation::collections::import_collection_by_path,
            presentation::collections::export_workspace,
            presentation::designs::list_designs,
            presentation::designs::create_design,
            presentation::designs::save_design,
            presentation::designs::delete_design,
            presentation::designs::lint_spec
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
