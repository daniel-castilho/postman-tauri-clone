// src-tauri/src/application/commands/report_tasks.rs
//
// Thin IPC adapter over the infrastructure run-report renderers. The GUI
// delegates HTML/Markdown generation here so the CLI and the desktop app
// share a single rendering implementation.

use crate::domain::models::{CollectionRunReport, RunReportFormat};

/// Renders a finished collection run as a self-contained document.
#[tauri::command]
pub fn render_run_report(
    collection_name: Option<String>,
    report: CollectionRunReport,
    duration_ms: u64,
    format: RunReportFormat,
) -> String {
    let name = collection_name
        .unwrap_or_else(|| "Collection Run".to_string());
    match format {
        RunReportFormat::Html => {
            crate::infrastructure::reporting::html_reporter::render_html(&name, &report, duration_ms)
        }
        RunReportFormat::Markdown => {
            crate::infrastructure::reporting::markdown_reporter::render_markdown(
                &name, &report, duration_ms,
            )
        }
    }
}
