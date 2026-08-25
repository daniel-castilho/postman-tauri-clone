// src-tauri/src/infrastructure/scripting/quickjs_runner.rs
use crate::application::ports::script_runner::ScriptRunnerPort;
use crate::domain::errors::DomainError;
use crate::domain::models::{HttpRequest, HttpResponse, TestResult, ScriptLog};
use quick_js::{Context, JsValue};

pub struct QuickJsScriptRunner {}

impl QuickJsScriptRunner {
    pub fn new() -> Self {
        Self {}
    }
}

impl ScriptRunnerPort for QuickJsScriptRunner {
    fn execute_pre_request(
        &self, 
        script: &str, 
        _request: &mut HttpRequest, 
        env_vars: &mut std::collections::HashMap<String, String>,
        global_vars: &mut std::collections::HashMap<String, String>,
        session_vars: &mut std::collections::HashMap<String, String>,
    ) -> Result<Vec<ScriptLog>, DomainError> {
        let (_tests, logs, updated_env, updated_globals, updated_session) = self.run_js_script(
            script, 
            None, 
            env_vars.clone(), 
            global_vars.clone(), 
            session_vars.clone()
        )?;
        *env_vars = updated_env;
        *global_vars = updated_globals;
        *session_vars = updated_session;
        Ok(logs)
    }

    fn execute_test(
        &self, 
        script: &str, 
        response: &HttpResponse, 
        env_vars: &mut std::collections::HashMap<String, String>,
        global_vars: &mut std::collections::HashMap<String, String>,
        session_vars: &mut std::collections::HashMap<String, String>,
    ) -> Result<(Vec<TestResult>, Vec<ScriptLog>), DomainError> {
        let (tests, logs, updated_env, updated_globals, updated_session) = self.run_js_script(
            script, 
            Some(response), 
            env_vars.clone(), 
            global_vars.clone(), 
            session_vars.clone()
        )?;
        *env_vars = updated_env;
        *global_vars = updated_globals;
        *session_vars = updated_session;
        Ok((tests, logs))
    }
}

impl QuickJsScriptRunner {
    fn run_js_script(
        &self, 
        script: &str, 
        response: Option<&HttpResponse>,
        mut env_vars: std::collections::HashMap<String, String>,
        mut global_vars: std::collections::HashMap<String, String>,
        mut session_vars: std::collections::HashMap<String, String>,
    ) -> Result<(Vec<TestResult>, Vec<ScriptLog>, std::collections::HashMap<String, String>, std::collections::HashMap<String, String>, std::collections::HashMap<String, String>), DomainError> {
        let context = Context::new().map_err(|e| DomainError::ScriptError(format!("Failed to create JS context: {}", e)))?;
        
        let env_json = serde_json::to_string(&env_vars).unwrap_or_else(|_| "{}".to_string());
        let global_json = serde_json::to_string(&global_vars).unwrap_or_else(|_| "{}".to_string());
        let session_json = serde_json::to_string(&session_vars).unwrap_or_else(|_| "{}".to_string());

        // SAFETY: Use JSON serialization for proper escaping to prevent injection attacks
        let resp_script = if let Some(res) = response {
            let body_str = res.body.clone().unwrap_or_default();

            // Properly escape the body for JavaScript string context using JSON serialization
            let escaped_body = serde_json::to_string(&body_str)
                .unwrap_or_else(|_| "\"\"".to_string());

            format!(
                r#"
                pm.response = {{
                    json: function() {{ return JSON.parse({}); }},
                    status: {},
                    text: function() {{ return {}; }}
                }};"#,
                escaped_body,
                res.status,
                escaped_body
            )
        } else { "".to_string() };

        let setup_script = format!(
            r#"
            var pm = {{
                test_results: [],
                logs: [],
                environment: {{
                    values: {},
                    set: function(key, val) {{ this.values[key] = String(val); }},
                    get: function(key) {{ return this.values[key]; }}
                }},
                globals: {{
                    values: {},
                    set: function(key, val) {{ this.values[key] = String(val); }},
                    get: function(key) {{ return this.values[key]; }}
                }},
                variables: {{
                    values: {},
                    set: function(key, val) {{ this.values[key] = String(val); }},
                    get: function(key) {{ return this.values[key]; }}
                }}
            }};
            var console = {{
                log: function(...args) {{ pm.logs.push({{ level: 'info', content: args.map(String).join(' ') }}); }},
                error: function(...args) {{ pm.logs.push({{ level: 'error', content: args.map(String).join(' ') }}); }},
                warn: function(...args) {{ pm.logs.push({{ level: 'warn', content: args.map(String).join(' ') }}); }}
            }};
            pm.test = function(name, callback) {{
                try {{
                    callback();
                    pm.test_results.push({{ name: name, passed: true, error: null }});
                }} catch (e) {{
                    pm.test_results.push({{ name: name, passed: false, error: e.toString() }});
                }}
            }};
            var expect = function(val) {{
                return {{
                    to: {{
                        equal: function(other) {{
                            if (val != other) throw new Error("Expected " + val + " to equal " + other);
                        }},
                        include: function(sub) {{
                            if (String(val).indexOf(sub) === -1) throw new Error("Expected " + val + " to include " + sub);
                        }},
                        be: {{
                            a: function(type) {{
                                if (typeof val !== type) throw new Error("Expected " + val + " to be a " + type);
                            }}
                        }}
                    }}
                }};
            }};
            {}
            "#,
            env_json,
            global_json,
            session_json,
            resp_script
        );

        context.eval(&setup_script).map_err(|e| DomainError::ScriptError(format!("JS Setup Error: {}", e)))?;
        context.eval(script).map_err(|e| DomainError::ScriptError(format!("Script Execution Error: {}", e)))?;
        
        let pm_val: JsValue = context.eval_as("pm").map_err(|e| DomainError::ScriptError(e.to_string()))?;
        let mut tests = vec![];
        let mut logs = vec![];
        
        if let JsValue::Object(obj) = pm_val {
            // Extract tests
            if let Some(JsValue::Array(arr)) = obj.get("test_results") {
                for v in arr {
                    if let JsValue::Object(res_obj) = v {
                        let name = match res_obj.get("name") {
                            Some(JsValue::String(s)) => s.clone(),
                            _ => "Unknown".to_string(),
                        };
                        let passed = match res_obj.get("passed") {
                            Some(JsValue::Bool(b)) => *b,
                            _ => false,
                        };
                        let error = match res_obj.get("error") {
                            Some(JsValue::String(s)) => Some(s.clone()),
                            _ => None,
                        };
                        tests.push(TestResult { name, passed, error });
                    }
                }
            }

            // Extract logs
            if let Some(JsValue::Array(arr)) = obj.get("logs") {
                for v in arr {
                    if let JsValue::Object(log_obj) = v {
                        let level = match log_obj.get("level") {
                            Some(JsValue::String(s)) => s.clone(),
                            _ => "info".to_string(),
                        };
                        let content = match log_obj.get("content") {
                            Some(JsValue::String(s)) => s.clone(),
                            _ => "".to_string(),
                        };
                        logs.push(ScriptLog { level, content });
                    }
                }
            }

            // Extract updated values for all scopes
            let scopes = ["environment", "globals", "variables"];
            for scope in scopes {
                if let Some(JsValue::Object(env_obj)) = obj.get(scope) {
                    if let Some(JsValue::Object(vals_obj)) = env_obj.get("values") {
                        let target_map = match scope {
                            "environment" => &mut env_vars,
                            "globals" => &mut global_vars,
                            "variables" => &mut session_vars,
                            _ => unreachable!(),
                        };
                        for (k, v) in vals_obj {
                            if let JsValue::String(s) = v {
                                target_map.insert(k.clone(), s.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok((tests, logs, env_vars, global_vars, session_vars))
    }
}
