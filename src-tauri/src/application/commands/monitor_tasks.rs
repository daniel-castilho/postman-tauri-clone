use crate::domain::models::{MonitorDefinition, MonitorReport, HttpRequest, HttpMethod, Url, RequestId};
use crate::application::ports::http_client::HttpClientPort;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use tokio::task::JoinHandle;
use tauri::Emitter;
use chrono::Local;

pub struct MonitorUseCase {
    http_client: Arc<dyn HttpClientPort>,
    active_monitors: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl MonitorUseCase {
    pub fn new(http_client: Arc<dyn HttpClientPort>) -> Self {
        Self {
            http_client,
            active_monitors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_monitor(&self, monitor: MonitorDefinition, app_handle: tauri::AppHandle) {
        let monitor_id = monitor.id.clone();
        let mut active = self.active_monitors.lock().await;
        
        // If one already exists, cancel it
        if let Some(handle) = active.remove(&monitor_id) {
            handle.abort();
        }

        let client = Arc::clone(&self.http_client);
        let mon = monitor.clone();

        let handle = tokio::spawn(async move {
            loop {
                // Prepare a simple GET request for the monitor
                let request = HttpRequest {
                    id: RequestId(format!("mon_{}", mon.id)),
                    name: format!("Monitor: {}", mon.name),
                    description: None,
                    method: HttpMethod::GET,
                    url: Url(mon.url.clone()),
                    headers: vec![],
                    body: None,
                    auth: None,
                    variables: HashMap::new(),
                    scripts: None,
                    grpc_config: None,
                };

                let start_time = std::time::Instant::now();
                let result = client.send(request).await;
                let duration = start_time.elapsed().as_millis() as u64;

                let report = match result {
                    Ok(res) => MonitorReport {
                        monitor_id: mon.id.clone(),
                        last_check: Local::now().to_rfc3339(),
                        status: res.status,
                        response_time_ms: duration,
                        is_healthy: res.status >= 200 && res.status < 400,
                    },
                    Err(_) => MonitorReport {
                        monitor_id: mon.id.clone(),
                        last_check: Local::now().to_rfc3339(),
                        status: 500,
                        response_time_ms: duration,
                        is_healthy: false,
                    },
                };

                // Emit the event to the frontend
                let _ = app_handle.emit("monitor-check", report);

                // Wait for the interval
                tokio::time::sleep(tokio::time::Duration::from_secs(mon.interval_seconds)).await;
            }
        });

        active.insert(monitor_id, handle);
    }

    pub async fn stop_monitor(&self, monitor_id: &str) {
        let mut active = self.active_monitors.lock().await;
        if let Some(handle) = active.remove(monitor_id) {
            handle.abort();
        }
    }
}
