use crate::domain::models::{SyncChange, WorkspaceMember, Environment};
use tauri::Emitter;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use uuid::Uuid;

pub struct SyncUseCase {
    members: Arc<Mutex<Vec<WorkspaceMember>>>,
    last_changes: Arc<Mutex<Vec<SyncChange>>>,
}

impl SyncUseCase {
    pub fn new() -> Self {
        Self {
            members: Arc::new(Mutex::new(vec![
                WorkspaceMember { 
                    user_id: "u1".into(), 
                    email: "owner@example.com".into(), 
                    role: crate::domain::models::MemberRole::Admin 
                }
            ])),
            last_changes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn invite_user(&self, email: String, role: crate::domain::models::MemberRole) -> Result<WorkspaceMember, String> {
        let mut members = self.members.lock().await;
        let new_user = WorkspaceMember {
            user_id: Uuid::new_v4().to_string(),
            email,
            role,
        };
        members.push(new_user.clone());
        Ok(new_user)
    }

    pub async fn get_members(&self) -> Vec<WorkspaceMember> {
        self.members.lock().await.clone()
    }

    pub async fn push_change(&self, app_handle: tauri::AppHandle, resource_type: String, resource_id: String, operation: String, mut data: String) {
        // Lógica de Segurança p/ Secrets
        if resource_type == "Environment" && operation == "Update" {
            if let Ok(mut env) = serde_json::from_str::<Environment>(&data) {
                for var in &mut env.variables {
                    // Nunca envia o valor local (current) p/ a nuvem/broadcast
                    var.current_value = String::new();
                    
                    // Se for Secret, talvez queira omitir o initial tb se for ultra-safe
                    // No Postman oficial o initial_value é compartilhado.
                }
                data = serde_json::to_string(&env).unwrap_or(data);
            }
        }

        let change = SyncChange {
            id: Uuid::new_v4().to_string(),
            resource_type,
            resource_id,
            operation,
            data,
            timestamp: Utc::now().to_rfc3339(),
        };

        // Cache local (simulando "cloud store")
        self.last_changes.lock().await.push(change.clone());

        // Broadcast local para outras abas ou simulação de peer-check
        let _ = app_handle.emit("sync-change", change);
    }
}
