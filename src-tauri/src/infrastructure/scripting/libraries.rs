// src-tauri/src/infrastructure/scripting/libraries.rs
//
// Curated script library registry (Phase 19). Bundles are vendored,
// MIT-licensed JavaScript files embedded at compile time and exposed to
// sandbox scripts through `require('name')`. See
// `assets/script-libs/THIRD-PARTY-NOTICES.md` for attribution.

pub struct ScriptLibrary {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub source: &'static str,
}

const LODASH_SOURCE: &str = include_str!("../../../assets/script-libs/lodash.min.js");
const DAYJS_SOURCE: &str = include_str!("../../../assets/script-libs/dayjs.min.js");
const CRYPTO_JS_SOURCE: &str = include_str!("../../../assets/script-libs/crypto-js.min.js");
const UUID_SOURCE: &str = include_str!("../../../assets/script-libs/uuid.min.js");

/// All registered libraries, in preload order.
pub fn registry() -> &'static [ScriptLibrary] {
    &[
        ScriptLibrary {
            name: "lodash",
            version: "4.17.21",
            description: "Utility belt for arrays, objects and functions.",
            source: LODASH_SOURCE,
        },
        ScriptLibrary {
            name: "dayjs",
            version: "1.11.13",
            description: "Immutable date-time formatting and manipulation.",
            source: DAYJS_SOURCE,
        },
        ScriptLibrary {
            name: "crypto-js",
            version: "4.2.0",
            description: "Hashes and ciphers (SHA, HMAC, AES, Base64...).",
            source: CRYPTO_JS_SOURCE,
        },
        ScriptLibrary {
            name: "uuid",
            version: "8.3.2",
            description: "RFC 4122 identifier generation (uuid.v4()).",
            source: UUID_SOURCE,
        },
    ]
}

/// Snippet evaluated before any bundle or user script. Polyfills what
/// vendored bundles expect from a browser host but QuickJS does not ship:
/// WebCrypto's `getRandomValues` (uuid@8 throws without it). The fallback
/// uses `Math.random`, which is fine for correlation ids but is NOT
/// cryptographically secure.
pub const SANDBOX_PRELOAD: &str = r#"
if (typeof globalThis.crypto === 'undefined' || typeof globalThis.crypto.getRandomValues !== 'function') {
    globalThis.crypto = {
        getRandomValues: function(typedArray) {
            for (var i = 0; i < typedArray.length; i++) {
                typedArray[i] = Math.floor(Math.random() * 256);
            }
            return typedArray;
        }
    };
}
"#;

/// Wraps a vendored UMD/CommonJS bundle so it evaluates to its exports.
/// Defining `module`/`exports` locals makes standard UMD builds select
/// their CommonJS branch instead of touching globals.
pub fn commonjs_wrap(source: &str) -> String {
    format!(
        "(function() {{ var module = {{ exports: {{}} }}; var exports = module.exports;\n{}\n;return module.exports; }})()",
        source
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_js::{Context, JsValue};

    fn evaluate(source: &str) -> Context {
        let context = Context::new().expect("quickjs context");
        context
            .eval(SANDBOX_PRELOAD)
            .expect("sandbox preload must evaluate");
        context
            .eval(&format!("var __module_exports = {};", commonjs_wrap(source)))
            .expect("bundle must evaluate inside the sandbox");
        context
    }

    fn eval_string(context: &Context, expression: &str) -> String {
        match context.eval(expression).expect("expression must evaluate") {
            JsValue::String(value) => value,
            other => panic!("expected string, got {:?}", other),
        }
    }

    fn eval_bool(context: &Context, expression: &str) -> bool {
        match context.eval(expression).expect("expression must evaluate") {
            JsValue::Bool(value) => value,
            other => panic!("expected bool, got {:?}", other),
        }
    }

    #[test]
    fn registry_has_four_mit_libraries() {
        let registry = registry();
        assert_eq!(registry.len(), 4);
        assert_eq!(
            registry.iter().map(|lib| lib.name).collect::<Vec<_>>(),
            vec!["lodash", "dayjs", "crypto-js", "uuid"]
        );
    }

    #[test]
    fn lodash_exports_functional_utilities() {
        let context = evaluate(LODASH_SOURCE);
        let chunked = match context
            .eval("__module_exports.chunk([1,2,3,4], 2).length")
            .expect("lodash smoke test")
        {
            JsValue::Int(length) => length,
            other => panic!("expected int, got {:?}", other),
        };
        assert_eq!(chunked, 2);
    }

    #[test]
    fn dayjs_formats_dates() {
        let context = evaluate(DAYJS_SOURCE);
        // Timezone-agnostic: dayjs must agree with the host Date object on
        // the same instant, regardless of the machine's local offset.
        let matches_host_date = eval_bool(
            &context,
            "(function(){ var d = new Date(Date.UTC(2020, 5, 15, 12, 0, 0)); \
             return __module_exports(d).year() === d.getFullYear(); })()",
        );
        assert!(matches_host_date);
    }

    #[test]
    fn crypto_js_computes_known_sha256_vector() {
        let context = evaluate(CRYPTO_JS_SOURCE);
        let digest = eval_string(&context, "__module_exports.SHA256('abc').toString()");
        // Well-known SHA-256 of "abc" (NIST vector).
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn uuid_generates_v4_identifiers() {
        let context = evaluate(UUID_SOURCE);
        let is_valid = eval_bool(
            &context,
            "(function(){ var id = __module_exports.v4(); \
             var pattern = /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/; \
             return pattern.test(id); })()",
        );
        assert!(is_valid);
    }
}
