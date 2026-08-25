// src-tauri/src/infrastructure/http/reqwest_adapter.rs
use async_trait::async_trait;
use reqwest::{Client, Method as ReqwestMethod, header::{HeaderMap, HeaderName, HeaderValue}};
use crate::application::ports::http_client::HttpClientPort;
use crate::domain::models::{HttpRequest, HttpResponse, HttpMethod, Header, Body, Auth};
use crate::domain::errors::DomainError;
use std::time::Instant;
use std::str::FromStr;

pub struct ReqwestHttpClientAdapter {
    client: Client,
    pub jar: std::sync::Arc<reqwest::cookie::Jar>,
}

impl ReqwestHttpClientAdapter {
    pub fn new() -> Self {
        let jar = std::sync::Arc::new(reqwest::cookie::Jar::default());
        Self {
            client: Client::builder()
                .cookie_provider(std::sync::Arc::clone(&jar))
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            jar
        }
    }

    fn map_method(method: &HttpMethod) -> ReqwestMethod {
        match method {
            HttpMethod::GET => ReqwestMethod::GET,
            HttpMethod::POST => ReqwestMethod::POST,
            HttpMethod::PUT => ReqwestMethod::PUT,
            HttpMethod::DELETE => ReqwestMethod::DELETE,
            HttpMethod::PATCH => ReqwestMethod::PATCH,
            HttpMethod::HEAD => ReqwestMethod::HEAD,
            HttpMethod::OPTIONS => ReqwestMethod::OPTIONS,
            HttpMethod::WS => ReqwestMethod::GET, // WS starts with GET upgrade
            HttpMethod::GRPC => ReqwestMethod::POST,
            HttpMethod::CUSTOM(m) => ReqwestMethod::from_bytes(m.as_bytes()).unwrap_or(ReqwestMethod::GET),
        }
    }
}

impl Default for ReqwestHttpClientAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HttpClientPort for ReqwestHttpClientAdapter {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, DomainError> {
        let url = request.url.0.clone();
        
        let method = Self::map_method(&request.method);
        let mut builder = self.client.request(method, &url);
        
        // Headers
        let mut headers_map = HeaderMap::new();
        for header in request.headers.iter().filter(|h| h.enabled) {
            if let (Ok(name), Ok(value)) = (HeaderName::from_str(&header.key), HeaderValue::from_str(&header.value)) {
                headers_map.insert(name, value);
            }
        }
        builder = builder.headers(headers_map);

        // Auth
        if let Some(auth) = &request.auth {
            match auth {
                Auth::NoAuth => {},
                Auth::Bearer { token } => {
                    builder = builder.bearer_auth(token);
                },
                Auth::Basic { username, password } => {
                    builder = builder.basic_auth(username, Some(password));
                },
                Auth::ApiKey { key, value, in_header } => {
                    if *in_header {
                        builder = builder.header(key, value);
                    } else {
                        builder = builder.query(&[(key, value)]);
                    }
                },
                Auth::OAuth2 { access_token, header_prefix } => {
                   let prefix = header_prefix.as_deref().unwrap_or("Bearer");
                   builder = builder.header("Authorization", format!("{} {}", prefix, access_token));
                },
                Auth::AWSSig4 { .. } => {
                    // Placeholder: Logging/Monitoring can be added here
                    println!("AWSSig4 selected but full signing logic is pending.");
                }
            }
        }

        // Body
        if let Some(body) = request.body {
            match body {
                Body::Raw(content, _) => {
                    builder = builder.body(content);
                },
                Body::UrlEncoded(pairs) => {
                    let map: std::collections::HashMap<&str, &str> = pairs.iter()
                        .filter(|p| p.enabled)
                        .map(|p| (p.key.as_str(), p.value.as_str()))
                        .collect();
                    builder = builder.form(&map);
                },
                Body::FormData(fields) => {
                    let mut form = reqwest::multipart::Form::new();
                    for field in fields.iter().filter(|f| f.enabled) {
                        if let Some(path) = &field.file {
                            if !path.is_empty() {
                                let file_name = std::path::Path::new(path)
                                    .file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "file".to_string());
                                
                                match tokio::fs::read(path).await {
                                    Ok(file_bytes) => {
                                        let part = reqwest::multipart::Part::bytes(file_bytes).file_name(file_name);
                                        form = form.part(field.key.clone(), part);
                                    },
                                    Err(e) => {
                                        println!("Warning: Could not read file for multipart: {}", e);
                                        // Proceed with an empty field or error? Postman usually fails silently or sends a string.
                                        form = form.text(field.key.clone(), format!("[Error reading file: {}]", path));
                                    }
                                }
                            } else {
                                form = form.text(field.key.clone(), field.value.clone());
                            }
                        } else {
                            form = form.text(field.key.clone(), field.value.clone());
                        }
                    }
                    builder = builder.multipart(form);
                },
                Body::Binary(data) => {
                    builder = builder.body(data);
                },
                Body::GraphQL { query, variables } => {
                    let mut payload = std::collections::HashMap::new();
                    payload.insert("query", serde_json::Value::String(query.clone()));
                    
                    if !variables.is_empty() {
                        if let Ok(vars_json) = serde_json::from_str::<serde_json::Value>(variables.as_str()) {
                            payload.insert("variables", vars_json);
                        }
                    }
                    builder = builder.json(&payload);
                }
            }
        }

        let start_time = Instant::now();
        
        // Executing HTTP Call
        let reqwest_response = builder.send().await.map_err(|e| DomainError::NetworkError(e.to_string()))?;
        
        let time_ms = start_time.elapsed().as_millis() as u64;
        let status = reqwest_response.status().as_u16();
        let status_text = reqwest_response.status().to_string();
        
        // Re-binding headers back to Domain model
        let mut out_headers = Vec::new();
        for (key, value) in reqwest_response.headers() {
            out_headers.push(Header {
                key: key.to_string(),
                value: value.to_str().unwrap_or("").to_string(),
                enabled: true
            });
        }
        
        // Controlled body mapping bytes to avoid blowing up memory with giant binary payloads
        let bytes = reqwest_response.bytes().await.map_err(|e| DomainError::NetworkError(e.to_string()))?;
        let size_bytes = bytes.len();
        let string_body = String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| "[Binary Content]".to_string());
        
        Ok(HttpResponse {
            status,
            status_text,
            headers: out_headers,
            body: Some(string_body),
            time_ms,
            size_bytes,
            tests_results: vec![],
            logs: vec![],
        })
    }
}
