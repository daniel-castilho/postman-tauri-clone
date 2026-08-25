// src-tauri/src/infrastructure/websocket/tungstenite_adapter.rs
use async_trait::async_trait;
use crate::application::ports::websocket::WebSocketPort;
use crate::domain::errors::DomainError;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};

pub struct TungsteniteWebSocketAdapter {
    app_handle: AppHandle,
    connections: Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>>>,
}

impl TungsteniteWebSocketAdapter {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WebSocketPort for TungsteniteWebSocketAdapter {
    async fn connect(&self, id: String, url: String) -> Result<(), DomainError> {
        let (ws_stream, _) = connect_async(&url).await.map_err(|e| DomainError::NetworkError(e.to_string()))?;
        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
        
        let app_handle = self.app_handle.clone();
        let conn_id = id.clone();
        let conns = self.connections.clone();
        
        // Writer Task
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if write.send(msg).await.is_err() { break; }
            }
        });
        
        // Reader Task
        let app_handle_inner = app_handle.clone();
        let conn_id_inner = id.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(msg) = msg {
                    // Emit messages to frontend
                    let payload = serde_json::json!({
                        "connectionId": conn_id,
                        "message": if msg.is_text() || msg.is_binary() { msg.to_string() } else { "".to_string() },
                        "type": if msg.is_text() { "text" } else { "binary" },
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    let _ = app_handle_inner.emit("ws-message", payload);
                }
            }
            // Cleanup on close
            conns.lock().await.remove(&conn_id_inner);
            let _ = app_handle_inner.emit("ws-status", serde_json::json!({ "connectionId": conn_id_inner, "status": "disconnected" }));
        });
        
        self.connections.lock().await.insert(id.clone(), tx);
        let _ = app_handle.emit("ws-status", serde_json::json!({ "connectionId": id, "status": "connected" }));
        
        Ok(())
    }

    async fn send(&self, id: String, message: String) -> Result<(), DomainError> {
        let conns = self.connections.lock().await;
        if let Some(tx) = conns.get(&id) {
            tx.send(Message::Text(message.into())).map_err(|e| DomainError::NetworkError(e.to_string()))?;
            Ok(())
        } else {
            Err(DomainError::NetworkError("Connection not found".to_string()))
        }
    }

    async fn disconnect(&self, id: String) -> Result<(), DomainError> {
        let mut conns = self.connections.lock().await;
        if let Some(tx) = conns.remove(&id) {
            let _ = tx.send(Message::Close(None));
            Ok(())
        } else {
            Ok(())
        }
    }
}
