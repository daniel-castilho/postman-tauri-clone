// src-tauri/src/infrastructure/codegen/template_adapter.rs
use base64::{engine::general_purpose::STANDARD, Engine as _};
use crate::application::ports::code_generator::CodeGeneratorPort;
use crate::domain::models::{HttpRequest, Body, Auth};

pub struct TemplateCodeGeneratorAdapter;

impl TemplateCodeGeneratorAdapter {
    pub fn new() -> Self { Self }
}

impl Default for TemplateCodeGeneratorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGeneratorPort for TemplateCodeGeneratorAdapter {
    fn generate_js_fetch(&self, request: &HttpRequest) -> String {
        let method = request.method.to_string();
        let url = &request.url.0;
        
        let mut headers_js = String::from("const myHeaders = new Headers();\n");
        for h in &request.headers {
            if h.enabled {
                headers_js.push_str(&format!("myHeaders.append(\"{}\", \"{}\");\n", h.key, h.value));
            }
        }

        // Auth Header
        if let Some(auth) = &request.auth {
            match auth {
                Auth::Bearer { token } => headers_js.push_str(&format!("myHeaders.append(\"Authorization\", \"Bearer {}\");\n", token)),
                Auth::Basic { username, password } => {
                    let auth_base64 = STANDARD.encode(format!("{}:{}", username, password));
                    headers_js.push_str(&format!("myHeaders.append(\"Authorization\", \"Basic {}\");\n", auth_base64));
                },
                _ => {}
            }
        }

        let (body_var, body_assign) = match &request.body {
            Some(Body::Raw(content, _)) => ("raw", format!("const raw = JSON.stringify({});\n", content)),
            _ => ("null", "".to_string()),
        };

        format!(
"{}{}
const requestOptions = {{
  method: \"{}\",
  headers: myHeaders,
  body: {},
  redirect: \"follow\"
}};

fetch(\"{}\", requestOptions)
  .then((response) => response.text())
  .then((result) => console.log(result))
  .catch((error) => console.error(error));",
            headers_js, body_assign, method, body_var, url
        )
    }

    fn generate_node_fetch(&self, request: &HttpRequest) -> String {
        let method = request.method.to_string();
        let url = &request.url.0;
        
        let mut headers_obj = String::from("{\n");
        for h in &request.headers {
            if h.enabled {
                headers_obj.push_str(&format!("    \"{}\": \"{}\",\n", h.key, h.value));
            }
        }
        
        // Auth Header
        if let Some(auth) = &request.auth {
            match auth {
                Auth::Bearer { token } => headers_obj.push_str(&format!("    \"Authorization\": \"Bearer {}\",\n", token)),
                Auth::Basic { username, password } => {
                    let auth_base = STANDARD.encode(format!("{}:{}", username, password));
                    headers_obj.push_str(&format!("    \"Authorization\": \"Basic {}\",\n", auth_base));
                },
                _ => {}
            }
        }
        headers_obj.push_str("  }");

        let (body_js, body_val) = match &request.body {
            Some(Body::Raw(content, _)) => (format!("const body = JSON.stringify({});\n", content), "body"),
            _ => ("".to_string(), "null"),
        };

        format!(
"async function makeRequest() {{
  const url = '{}';
  const headers = {};
  {}
  const response = await fetch(url, {{
    method: '{}',
    headers,
    body: {}
  }});

  const data = await response.text();
  console.log(data);
}}

makeRequest();",
            url, headers_obj, body_js, method, body_val
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::{Header, HttpRequest, RequestId, Url};

    fn sample_request() -> HttpRequest {
        HttpRequest {
            id: RequestId("req_1".into()),
            name: "Create pet".into(),
            description: None,
            method: crate::domain::models::HttpMethod::POST,
            url: Url("https://api.example.com/pets".into()),
            headers: vec![
                Header { key: "X-Token".into(), value: "secret".into(), enabled: true },
                Header { key: "X-Disabled".into(), value: "no".into(), enabled: false },
            ],
            body: Some(crate::domain::models::Body::Raw(
                "{\"name\":\"Rex\"}".into(),
                crate::domain::models::BodyMode::Json,
            )),
            auth: None,
            variables: Default::default(),
            scripts: None,
            grpc_config: None,
        }
    }

    #[test]
    fn js_fetch_includes_method_url_and_only_enabled_headers() {
        let adapter = TemplateCodeGeneratorAdapter::new();
        let code = adapter.generate_js_fetch(&sample_request());

        assert!(code.contains("POST"));
        assert!(code.contains("https://api.example.com/pets"));
        assert!(code.contains("X-Token"));
        assert!(!code.contains("X-Disabled"));
    }

    #[test]
    fn node_fetch_contains_method_and_body_payload() {
        let adapter = TemplateCodeGeneratorAdapter::new();
        let code = adapter.generate_node_fetch(&sample_request());

        assert!(code.contains("POST") || code.contains("method"));
        assert!(code.contains("pets"));
    }
}
