use crate::domain::models::{HttpRequest, HttpResponse, Header};
use crate::domain::errors::DomainError;
use crate::application::ports::grpc_client::GrpcClientPort;
use async_trait::async_trait;

pub struct MockGrpcClientAdapter;

#[async_trait]
impl GrpcClientPort for MockGrpcClientAdapter {
    async fn call(&self, request: HttpRequest) -> Result<HttpResponse, DomainError> {
        let grpc = request.grpc_config.ok_or(DomainError::ValidationError("gRPC configuration missing".into()))?;
        
        // Mocking the gRPC call
        let response_body = format!(
            "{{\n  \"message\": \"gRPC simulation successful!\",\n  \"service\": \"{}\",\n  \"method\": \"{}\",\n  \"proto\": \"{}\"\n}}",
            grpc.service, grpc.method, grpc.proto_path
        );

        Ok(HttpResponse {
            status: 0, // gRPC doesn't use HTTP status in the same way, usually 0 is OK
            status_text: "OK (gRPC)".into(),
            headers: vec![
                Header { key: "content-type".into(), value: "application/grpc".into(), enabled: true }
            ],
            body: Some(response_body),
            time_ms: 42,
            size_bytes: 156,
            tests_results: vec![],
            logs: vec![]
        })
    }
}
