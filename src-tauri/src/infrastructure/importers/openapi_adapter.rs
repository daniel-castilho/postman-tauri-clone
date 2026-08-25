// src-tauri/src/infrastructure/importers/openapi_adapter.rs
use serde_json::Value;
use crate::application::ports::import_port::ImportPort;
use crate::domain::models::{Collection, CollectionItem, HttpRequest, HttpMethod, Url};
use crate::domain::errors::DomainError;
use uuid::Uuid;

pub struct OpenApiImporterAdapter;

impl OpenApiImporterAdapter {
    pub fn new() -> Self { Self }
}

impl ImportPort for OpenApiImporterAdapter {
    fn parse_openapi(&self, content: &str) -> Result<Collection, DomainError> {
        let v: Value = if content.trim().starts_with('{') {
            serde_json::from_str(content).map_err(|e| DomainError::SerializationError(e.to_string()))?
        } else {
            serde_yaml::from_str(content).map_err(|e| DomainError::SerializationError(e.to_string()))?
        };

        let title = v["info"]["title"].as_str().unwrap_or("Imported Collection").to_string();
        let mut items = Vec::new();

        if let Some(paths) = v["paths"].as_object() {
            for (path, methods) in paths {
                if let Some(methods_obj) = methods.as_object() {
                    for (method, details) in methods_obj {
                        let http_method = match method.to_uppercase().as_str() {
                            "GET" => HttpMethod::GET,
                            "POST" => HttpMethod::POST,
                            "PUT" => HttpMethod::PUT,
                            "DELETE" => HttpMethod::DELETE,
                            "PATCH" => HttpMethod::PATCH,
                            _ => continue,
                        };

                        let summary = details["summary"].as_str().unwrap_or(path);
                        
                        items.push(CollectionItem::Request(HttpRequest {
                            id: Uuid::new_v4().to_string(),
                            name: summary.to_string(),
                            method: http_method,
                            url: Url(format!("{{{{base_url}}}}{}", path)),
                            headers: Vec::new(),
                            body: None,
                            auth: None,
                            variables: std::collections::HashMap::new(),
                            scripts: None,
                        }));
                    }
                }
            }
        }

        Ok(Collection {
            id: Uuid::new_v4().to_string(),
            name: title,
            items,
        })
    }
}
