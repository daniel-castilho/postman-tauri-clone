use crate::domain::models::DesignSpec;
use crate::domain::errors::DomainError;

pub trait DesignRepositoryPort: Send + Sync {
    fn list_designs(&self, workspace_path: &str) -> Result<Vec<DesignSpec>, DomainError>;
    fn save_design(&self, workspace_path: &str, design: &DesignSpec) -> Result<(), DomainError>;
    fn delete_design(&self, workspace_path: &str, design_id: &str) -> Result<(), DomainError>;
}
