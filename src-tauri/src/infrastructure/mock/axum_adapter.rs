// src-tauri/src/infrastructure/mock/axum_adapter.rs
use std::sync::Arc;
use tokio::sync::{RwLock, oneshot};
use async_trait::async_trait;
use axum::{
    routing::any,
    Router,
    response::{Response, IntoResponse},
    http::{StatusCode, HeaderName, HeaderValue, Method},
    extract::{Request, State as AxumState},
};
use crate::application::ports::mock_server::MockServerPort;
use crate::domain::models::{MockRule, MockServerStatus, HttpMethod};
use crate::domain::errors::DomainError;

struct ServerState {
    rules: Vec<MockRule>,
}

pub struct AxumMockServerAdapter {
    state: Arc<RwLock<ServerState>>,
    stop_tx: Arc<RwLock<Option<oneshot::Sender<()>>>>,
    is_running: Arc<RwLock<bool>>,
    current_port: Arc<RwLock<u16>>,
}

impl AxumMockServerAdapter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ServerState { rules: Vec::new() })),
            stop_tx: Arc::new(RwLock::new(None)),
            is_running: Arc::new(RwLock::new(false)),
            current_port: Arc::new(RwLock::new(0)),
        }
    }
}

impl Default for AxumMockServerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

async fn handle_mock(
    AxumState(state): AxumState<Arc<RwLock<ServerState>>>,
    req: Request,
) -> impl IntoResponse {
    let state = state.read().await;
    let path = req.uri().path();
    let method = req.method();

    // Find matching rule
    let matching_rule = state.rules.iter().find(|r| {
        let path_matches = r.path == path;
        let method_matches = matches!(
            (&r.method, method),
            (HttpMethod::GET, &Method::GET)
                | (HttpMethod::POST, &Method::POST)
                | (HttpMethod::PUT, &Method::PUT)
                | (HttpMethod::DELETE, &Method::DELETE)
                | (HttpMethod::PATCH, &Method::PATCH)
        );
        path_matches && method_matches
    });

    if let Some(rule) = matching_rule {
        let mut response = Response::builder()
            .status(StatusCode::from_u16(rule.status).unwrap_or(StatusCode::OK))
            .body(axum::body::Body::from(rule.body.clone()))
            .unwrap();

        for header in &rule.headers {
            if let (Ok(name), Ok(val)) = (HeaderName::from_bytes(header.key.as_bytes()), HeaderValue::from_str(&header.value)) {
                response.headers_mut().insert(name, val);
            }
        }
        response
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("No mock rule found for this path/method"))
            .unwrap()
    }
}

#[async_trait]
impl MockServerPort for AxumMockServerAdapter {
    async fn start(&self, port: u16, rules: Vec<MockRule>) -> Result<(), DomainError> {
        self.stop().await?; // Ensure previous is stopped

        let state = self.state.clone();
        {
            let mut s = state.write().await;
            s.rules = rules;
        }

        let (tx, rx) = oneshot::channel::<()>();
        *self.stop_tx.write().await = Some(tx);
        *self.is_running.write().await = true;
        *self.current_port.write().await = port;

        let app = Router::new()
            .fallback(any(handle_mock))
            .with_state(state);

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| DomainError::NetworkError(e.to_string()))?;

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    rx.await.ok();
                })
                .await
                .ok();
        });

        Ok(())
    }

    async fn stop(&self) -> Result<(), DomainError> {
        if let Some(tx) = self.stop_tx.write().await.take() {
            let _ = tx.send(());
        }
        *self.is_running.write().await = false;
        *self.current_port.write().await = 0;
        Ok(())
    }

    async fn get_status(&self) -> MockServerStatus {
        MockServerStatus {
            is_running: *self.is_running.read().await,
            port: *self.current_port.read().await,
            active_rules: self.state.read().await.rules.len(),
        }
    }
}
