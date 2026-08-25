use tauri::State;
use crate::application::commands::design_tasks::DesignUseCase;
use crate::domain::models::{DesignSpec, LintIssue};
use crate::domain::errors::AppError;

#[tauri::command]
pub fn list_designs(
    workspace_path: String,
    use_case: State<'_, DesignUseCase>,
) -> Result<Vec<DesignSpec>, AppError> {
    use_case.list_designs(&workspace_path).map_err(AppError::from)
}

#[tauri::command]
pub fn create_design(
    workspace_path: String,
    name: String,
    format: String,
    use_case: State<'_, DesignUseCase>,
) -> Result<DesignSpec, AppError> {
    use_case.create_design(&workspace_path, name, format).map_err(AppError::from)
}

#[tauri::command]
pub fn save_design(
    workspace_path: String,
    design: DesignSpec,
    use_case: State<'_, DesignUseCase>,
) -> Result<(), AppError> {
    use_case.save_design(&workspace_path, design).map_err(AppError::from)
}

#[tauri::command]
pub fn delete_design(
    workspace_path: String,
    design_id: String,
    use_case: State<'_, DesignUseCase>,
) -> Result<(), AppError> {
    use_case.delete_design(&workspace_path, &design_id).map_err(AppError::from)
}

#[tauri::command]
pub fn lint_spec(
    content: String,
    use_case: State<'_, DesignUseCase>,
) -> Result<Vec<LintIssue>, AppError> {
    Ok(use_case.lint_spec(&content))
}
