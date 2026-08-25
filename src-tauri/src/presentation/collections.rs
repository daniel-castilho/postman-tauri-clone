// src-tauri/src/presentation/collections.rs
use tauri::State;
use crate::application::commands::workspace::WorkspaceUseCase;
use crate::domain::models::Collection;
use crate::domain::errors::AppError;

#[tauri::command]
pub fn load_collections(
    workspace_path: String,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<Vec<Collection>, AppError> {
    use_case.load_collections(&workspace_path).map_err(AppError::from)
}

#[tauri::command]
pub fn save_collection(
    workspace_path: String,
    collection: Collection,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<(), AppError> {
    use_case.save_collection(&workspace_path, collection).map_err(AppError::from)
}

#[tauri::command]
pub fn delete_collection(
    workspace_path: String,
    collection_id: String,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<(), AppError> {
    use_case.delete_collection(&workspace_path, &collection_id).map_err(AppError::from)
}

#[tauri::command]
pub fn load_environments(
    workspace_path: String,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<Vec<crate::domain::models::Environment>, AppError> {
    use_case.load_environments(&workspace_path).map_err(AppError::from)
}

#[tauri::command]
pub fn save_environments(
    workspace_path: String,
    environments: Vec<crate::domain::models::Environment>,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<(), AppError> {
    use_case.save_environments(&workspace_path, environments).map_err(AppError::from)
}

#[tauri::command]
pub fn import_collection_by_path(
    collection_path: String,
    workspace_path: String,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<Collection, AppError> {
    let content = std::fs::read_to_string(&collection_path)
        .map_err(|e| AppError::persistence_error(format!("Error reading external file: {}", e)))?;
    
    let mut collection: Collection = serde_json::from_str(&content)
        .map_err(|e| AppError::persistence_error(format!("Invalid collection format: {}", e)))?;

    // Guarantee a unique ID to avoid conflicts (defensive measure)
    collection.id = format!("col_imp_{}", uuid::Uuid::new_v4());

    use_case.save_collection(&workspace_path, collection.clone()).map_err(AppError::from)?;
    
    Ok(collection)
}

#[tauri::command]
pub fn export_workspace(
    workspace_path: String,
    export_path: String,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<(), AppError> {
    let collections = use_case.load_collections(&workspace_path).map_err(AppError::from)?;
    let environments = use_case.load_environments(&workspace_path).map_err(AppError::from)?;
    let globals = use_case.load_globals(&workspace_path).ok();

    let bundle = crate::domain::models::WorkspaceBundle {
        collections,
        environments,
        globals,
        export_date: chrono::Utc::now().to_rfc3339(),
        app_version: "0.1.0".to_string(),
    };
    
    let content = serde_json::to_string_pretty(&bundle)
        .map_err(|e| AppError::persistence_error(format!("Error serializing bundle: {}", e)))?;
        
    std::fs::write(&export_path, content)
        .map_err(|e| AppError::persistence_error(format!("Error writing export file: {}", e)))?;
        
    Ok(())
}

#[tauri::command]
pub fn load_globals(
    workspace_path: String,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<crate::domain::models::GlobalVariables, AppError> {
    use_case.load_globals(&workspace_path).map_err(AppError::from)
}

#[tauri::command]
pub fn save_globals(
    workspace_path: String,
    globals: crate::domain::models::GlobalVariables,
    use_case: State<'_, WorkspaceUseCase>,
) -> Result<(), AppError> {
    use_case.save_globals(&workspace_path, globals).map_err(AppError::from)
}
