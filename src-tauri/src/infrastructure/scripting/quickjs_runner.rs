// src-tauri/src/infrastructure/scripting/quickjs_runner.rs
use super::libraries::{commonjs_wrap, registry, SANDBOX_PRELOAD};
use crate::application::ports::script_runner::ScriptRunnerPort;
use crate::domain::errors::DomainError;
use crate::domain::models::{HttpRequest, HttpResponse, TestResult, ScriptLog};
use crate::infrastructure::persistence::fs_script_settings_repository::FsScriptSettingsRepository;
use quick_js::{Context, JsValue};

use std::sync::RwLock;

pub struct QuickJsScriptRunner {
    /// Workspace directory holding `script-libraries.json`. `None` keeps
    /// every registered library enabled (headless runs without a workspace).
    settings_dir: RwLock<Option<String>>,
    script_settings: FsScriptSettingsRepository,
}

impl QuickJsScriptRunner {
    pub fn new() -> Self {
        Self {
            settings_dir: RwLock::new(None),
            script_settings: FsScriptSettingsRepository::new(),
        }
    }

    /// Points the runner at the workspace whose library settings apply.
    pub fn set_settings_dir(&self, path: Option<String>) {
        let mut guard = match self.settings_dir.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        *guard = path;
    }

    fn disabled_libraries(&self) -> Vec<String> {
        let guard = match self.settings_dir.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        match guard.as_deref() {
            Some(dir) => self.script_settings.load_disabled(dir),
            None => Vec::new(),
        }
    }

    /// Builds the module bootstrap evaluated before every user script:
    /// sandbox polyfills, preloaded (enabled) bundles and the `require`
    /// resolver.
    fn build_modules_code(&self) -> String {
        let disabled = self.disabled_libraries();
        let mut code = String::from(SANDBOX_PRELOAD);
        code.push_str("\nvar __tyny_modules = {};\n");
        for library in registry() {
            if !disabled.iter().any(|name| name == library.name) {
                code.push_str(&format!(
                    "__tyny_modules['{}'] = {};\n",
                    library.name,
                    commonjs_wrap(library.source)
                ));
            }
        }
        code.push_str(
            "var require = function(name) { \
             if (Object.prototype.hasOwnProperty.call(__tyny_modules, name)) { \
             return __tyny_modules[name]; } \
             throw new Error(\"Module '\" + name + \"' is not available. Installed modules: \" + Object.keys(__tyny_modules).join(', ')); };\n",
        );
        code
    }
}

impl Default for QuickJsScriptRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated mutable variable scopes returned by a sandboxed script execution.
pub type ScriptVariableScopes = (
    Vec<TestResult>,
    Vec<ScriptLog>,
    std::collections::HashMap<String, String>,
    std::collections::HashMap<String, String>,
    std::collections::HashMap<String, String>,
);

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
    ) -> Result<ScriptVariableScopes, DomainError> {
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
        let modules_code = self.build_modules_code();
        context.eval(&modules_code).map_err(|e| DomainError::ScriptError(format!("JS Modules Error: {}", e)))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_resolves_bundled_library_inside_test_script() {
        let runner = QuickJsScriptRunner::new();
        let mut env = std::collections::HashMap::new();
        let script = r#"
            var _ = require('lodash');
            var crypto = require('crypto-js');
            pm.test('lodash chunks pairs', function() {
                if (_.chunk([1,2,3,4], 2).length !== 2) throw new Error('chunk failed');
            });
            pm.test('sha256 vector', function() {
                if (crypto.SHA256('abc').toString() !== 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad') throw new Error('hash failed');
            });
        "#;
        let (tests, _logs, _e, _g, _s) = runner
            .run_js_script(script, None, env.clone(), env.clone(), env)
            .expect("script must run");
        assert_eq!(tests.len(), 2);
        assert!(tests.iter().all(|t| t.passed), "failures: {:?}", tests);
    }

    #[test]
    fn unknown_module_inside_test_is_reported() {
        let runner = QuickJsScriptRunner::new();
        let mut env = std::collections::HashMap::new();
        let script = r#"
            pm.test('uses missing module', function() {
                require('not-a-real-module');
            });
        "#;
        let (tests, _logs, _e, _g, _s) = runner
            .run_js_script(script, None, env.clone(), env.clone(), env)
            .expect("script must run");
        assert_eq!(tests.len(), 1);
        assert!(!tests[0].passed);
        let error = tests[0].error.as_deref().unwrap_or_default();
        assert!(error.contains("not available"), "unexpected error: {}", error);
        assert!(error.contains("lodash"), "error should list installed modules: {}", error);
    }

    #[test]
    fn disabled_library_is_not_preloaded() {
        let workspace_dir = {
            let dir = std::env::temp_dir().join(format!(
                "tyny-runner-libs-{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&dir).expect("temp dir");
            let settings = crate::infrastructure::persistence::fs_script_settings_repository::FsScriptSettingsRepository::new();
            settings
                .save_disabled(dir.to_str().expect("utf8 path"), &["lodash".to_string()])
                .expect("save disabled");
            dir
        };

        let runner = QuickJsScriptRunner::new();
        runner.set_settings_dir(Some(workspace_dir.to_string_lossy().to_string()));

        let mut env = std::collections::HashMap::new();
        let script = r#"
            pm.test('dayjs still available', function() {
                if (typeof require('dayjs') !== 'function') throw new Error('dayjs missing');
            });
            pm.test('lodash disabled', function() {
                try { require('lodash'); throw new Error('should have thrown'); }
                catch (e) { if (String(e.message || e).indexOf('not available') === -1) throw e; }
            });
        "#;
        let (tests, _logs, _e, _g, _s) = runner
            .run_js_script(script, None, env.clone(), env.clone(), env)
            .expect("script must run");
        assert_eq!(tests.len(), 2);
        assert!(tests.iter().all(|t| t.passed), "failures: {:?}", tests);

        std::fs::remove_dir_all(workspace_dir).ok();
    }
}
