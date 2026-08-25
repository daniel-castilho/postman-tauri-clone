// src/types/ipc.ts
//
// Single stable import point for every Tauri IPC type.
//
// The files under `./generated/` are produced by ts-rs from the Rust domain
// models (source of truth: `src-tauri/src/domain/models.rs` and `errors.rs`)
// and MUST NOT be edited by hand. Regenerate them with:
//
//   cargo test export_ts_bindings
//
// This barrel exists so application code never imports generated file paths
// directly; it also gives us one place to audit which contract types are
// actually consumed by the frontend.

export type { AppError } from './generated/AppError';
export type { Auth } from './generated/Auth';
export type { Body } from './generated/Body';
export type { BodyMode } from './generated/BodyMode';
export type { Collection } from './generated/Collection';
export type { CollectionItem } from './generated/CollectionItem';
export type { CollectionRunReport } from './generated/CollectionRunReport';
export type { DesignSpec } from './generated/DesignSpec';
export type { Environment } from './generated/Environment';
export type { EnvironmentVariable } from './generated/EnvironmentVariable';
export type { FormField } from './generated/FormField';
export type { GlobalVariables } from './generated/GlobalVariables';
export type { GrpcConfig } from './generated/GrpcConfig';
export type { GrpcMetadata } from './generated/GrpcMetadata';
export type { Header } from './generated/Header';
export type { HttpScripts } from './generated/HttpScripts';
export type { HttpRequest } from './generated/HttpRequest';
export type { HttpResponse } from './generated/HttpResponse';
export type { HttpMethod } from './generated/HttpMethod';
export type { KeyValue } from './generated/KeyValue';
export type { LintIssue } from './generated/LintIssue';
export type { LintSeverity } from './generated/LintSeverity';
export type { LoadTestConfig } from './generated/LoadTestConfig';
export type { LoadTestReport } from './generated/LoadTestReport';
export type { MemberRole } from './generated/MemberRole';
export type { MockRule } from './generated/MockRule';
export type { MockServerStatus } from './generated/MockServerStatus';
export type { MonitorDefinition } from './generated/MonitorDefinition';
export type { MonitorReport } from './generated/MonitorReport';
export type { RequestId } from './generated/RequestId';
export type { RequestRunResult } from './generated/RequestRunResult';
export type { ScriptLog } from './generated/ScriptLog';
export type { ScriptLibraryInfo } from './generated/ScriptLibraryInfo';
export type { SendRequestOutput } from './generated/SendRequestOutput';
export type { SyncChange } from './generated/SyncChange';
export type { TestResult } from './generated/TestResult';
export type { Url } from './generated/Url';
export type { VariableType } from './generated/VariableType';
export type { WorkspaceBundle } from './generated/WorkspaceBundle';
export type { WorkspaceMember } from './generated/WorkspaceMember';
