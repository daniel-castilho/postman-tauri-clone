// src-tauri/src/application/commands/export_ts_bindings.rs
//
// Automated TypeScript bindings exporter (Zero Type-Drift epic, story S6).
//
// Every IPC-crossing domain type derives `ts_rs::TS` with `#[ts(export)]`,
// which also generates an individual export test. This module provides a
// single deterministic entry point (`cargo test export_ts_bindings`) that
// regenerates ALL bindings in one go, so CI and developers can refresh
// `src/types/generated/` with one command.
//
// The output directory is configured once via `TS_RS_EXPORT_DIR`
// (see `.cargo/config.toml`) and MUST NOT be edited by hand.
//
// NOTE: when a new DTO is added to `domain/models.rs` or `domain/errors.rs`,
// register it in the list below so it is exported and drift-checked.

#[cfg(test)]
mod tests {
    use ts_rs::TS;
    use crate::domain::errors::AppError;
    use crate::domain::models::{
        Auth, Body, BodyMode, Collection, CollectionItem,
        CollectionRunReport, DesignSpec, Environment, EnvironmentVariable, FormField,
        GlobalVariables, GrpcConfig, GrpcMetadata, Header, HttpScripts, HttpRequest,
        HttpResponse, HttpMethod, KeyValue, LintIssue, LintSeverity, LoadTestConfig,
        LoadTestReport, MemberRole, MockRule, MockServerStatus, MonitorDefinition,
        MonitorReport,         RequestId, RequestRunResult, ScriptLibraryInfo, ScriptLog, SendRequestOutput, SyncChange,
        TestResult, Url, VariableType, WorkspaceBundle, WorkspaceMember,
    };

    #[test]
    fn export_ts_bindings() {
        // Reads TS_RS_EXPORT_DIR / TS_RS_LARGE_INT (see .cargo/config.toml)
        let cfg = ts_rs::Config::from_env();

        // Value objects & primitives of the request/response contract
        RequestId::export_all(&cfg).unwrap();
        Url::export_all(&cfg).unwrap();
        Header::export_all(&cfg).unwrap();
        HttpMethod::export_all(&cfg).unwrap();
        Body::export_all(&cfg).unwrap();
        BodyMode::export_all(&cfg).unwrap();
        FormField::export_all(&cfg).unwrap();
        KeyValue::export_all(&cfg).unwrap();
        Auth::export_all(&cfg).unwrap();

        // HTTP execution contract
        HttpRequest::export_all(&cfg).unwrap();
        HttpScripts::export_all(&cfg).unwrap();
        HttpResponse::export_all(&cfg).unwrap();
        ScriptLog::export_all(&cfg).unwrap();
        TestResult::export_all(&cfg).unwrap();
        SendRequestOutput::export_all(&cfg).unwrap();

        // Collections & workspace persistence
        Collection::export_all(&cfg).unwrap();
        CollectionItem::export_all(&cfg).unwrap();
        WorkspaceBundle::export_all(&cfg).unwrap();

        // Environments & variables
        Environment::export_all(&cfg).unwrap();
        EnvironmentVariable::export_all(&cfg).unwrap();
        VariableType::export_all(&cfg).unwrap();
        GlobalVariables::export_all(&cfg).unwrap();

        // Collection runner reports
        CollectionRunReport::export_all(&cfg).unwrap();
        RequestRunResult::export_all(&cfg).unwrap();

        // Mock server
        MockRule::export_all(&cfg).unwrap();
        MockServerStatus::export_all(&cfg).unwrap();

        // gRPC configuration
        GrpcConfig::export_all(&cfg).unwrap();
        GrpcMetadata::export_all(&cfg).unwrap();

        // Load testing
        LoadTestConfig::export_all(&cfg).unwrap();
        LoadTestReport::export_all(&cfg).unwrap();

        // Monitors (also emitted as the `monitor-check` event payload)
        MonitorDefinition::export_all(&cfg).unwrap();
        MonitorReport::export_all(&cfg).unwrap();

        // Collaborative sync (also emitted as the `sync-change` event payload)
        MemberRole::export_all(&cfg).unwrap();
        WorkspaceMember::export_all(&cfg).unwrap();
        SyncChange::export_all(&cfg).unwrap();

        // Script library registry (Phase 19 Package Manager)
        ScriptLibraryInfo::export_all(&cfg).unwrap();

        // SpecHub design & governance linting
        DesignSpec::export_all(&cfg).unwrap();
        LintSeverity::export_all(&cfg).unwrap();
        LintIssue::export_all(&cfg).unwrap();

        // IPC error envelope
        AppError::export_all(&cfg).unwrap();
    }
}
