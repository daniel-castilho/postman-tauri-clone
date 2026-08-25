use crate::application::ports::design_repository::DesignRepositoryPort;
use crate::domain::models::DesignSpec;
use crate::domain::errors::DomainError;
use std::fs;
use std::path::Path;

pub struct FsDesignRepository;

impl DesignRepositoryPort for FsDesignRepository {
    fn list_designs(&self, workspace_path: &str) -> Result<Vec<DesignSpec>, DomainError> {
        let designs_path = Path::new(workspace_path).join("designs");
        if !designs_path.exists() {
            return Ok(Vec::new());
        }

        let mut designs = Vec::new();
        for entry in fs::read_dir(designs_path).map_err(|e| DomainError::PersistenceError(e.to_string()))? {
            let entry = entry.map_err(|e| DomainError::PersistenceError(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path).map_err(|e| DomainError::PersistenceError(e.to_string()))?;
                if let Ok(design) = serde_json::from_str::<DesignSpec>(&content) {
                    designs.push(design);
                }
            }
        }
        Ok(designs)
    }

    fn save_design(&self, workspace_path: &str, design: &DesignSpec) -> Result<(), DomainError> {
        let designs_path = Path::new(workspace_path).join("designs");
        fs::create_dir_all(&designs_path).map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        
        let file_path = designs_path.join(format!("{}.json", design.id));
        let content = serde_json::to_string_pretty(design).map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        fs::write(file_path, content).map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        Ok(())
    }

    fn delete_design(&self, workspace_path: &str, design_id: &str) -> Result<(), DomainError> {
        let file_path = Path::new(workspace_path).join("designs").join(format!("{}.json", design_id));
        if file_path.exists() {
            fs::remove_file(file_path).map_err(|e| DomainError::PersistenceError(e.to_string()))?;
        }
        Ok(())
    }
}
