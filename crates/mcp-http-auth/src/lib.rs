//! HTTP transport authentication product kernels.
//!
//! Parity with `src/transports/auth.ts`. Pure/sync decision + payload helpers;
//! no network I/O. TS remains a read-only oracle for dual-oracle unit tests.
//!
//! Authority: Rust product path for auth decision logic (WAVE4). Full HTTP
//! transport binding remains a separate crate/surface.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// JSON-RPC error code for unauthorized (server-defined range).
pub const UNAUTHORIZED_CODE: i64 = -32_001;

/// Header advertised on 401 so clients know how to authenticate.
pub const WWW_AUTHENTICATE: &str = r#"Bearer realm="mcp", charset="UTF-8""#;

/// Opt-in API-key auth configuration (no custom hook — pure path).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AuthOptions {
    /// Expected `X-API-Key` value. When set (and no custom hook at the
    /// transport layer), MCP requests require a matching header.
    pub api_key: Option<String>,
}

impl AuthOptions {
    #[must_use]
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
        }
    }

    #[must_use]
    pub fn none() -> Self {
        Self { api_key: None }
    }
}

/// True when any authentication is configured (apiKey path).
#[must_use]
pub fn is_auth_enabled(options: &AuthOptions) -> bool {
    options.api_key.is_some()
}

/// SHA-256 digest of UTF-8 bytes (parity with Node `createHash("sha256")`).
#[must_use]
pub fn sha256_bytes(value: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let out = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

/// Constant-time equality for equal-length byte slices.
#[must_use]
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

/// Constant-time API key comparison.
///
/// Both sides are SHA-256 hashed so buffers always share length and raw key
/// bytes never drive timing. Empty/missing provided keys always reject.
#[must_use]
pub fn verify_api_key(expected: &str, provided: Option<&str>) -> bool {
    match provided {
        None | Some("") => false,
        Some(p) => constant_time_eq(&sha256_bytes(expected), &sha256_bytes(p)),
    }
}

/// Read a header from a flat string map (case-insensitive key lookup).
///
/// Handles multi-value by taking the first entry when the map stores comma-
/// joined or when callers pass the first of a string[].
#[must_use]
pub fn read_header_map(
    headers: &std::collections::BTreeMap<String, String>,
    name: &str,
) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    for (k, v) in headers {
        if k.eq_ignore_ascii_case(&lower) {
            // First of comma-separated list (Node string[] → join often first).
            let first = v.split(',').next().unwrap_or(v).trim();
            if first.is_empty() {
                return None;
            }
            return Some(first.to_string());
        }
    }
    None
}

/// Read first value from a multi-value header map (parity: string | string[]).
#[must_use]
pub fn read_header_multi(
    headers: &std::collections::BTreeMap<String, Vec<String>>,
    name: &str,
) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    for (k, values) in headers {
        if k.eq_ignore_ascii_case(&lower) {
            let first = values.first()?.trim();
            if first.is_empty() {
                return None;
            }
            return Some(first.to_string());
        }
    }
    None
}

/// Pure apiKey authorization decision (no custom authenticate hook).
///
/// Precedence at the product layer for pure path:
/// - no apiKey configured → allow
/// - apiKey configured → verify `X-API-Key` header
#[must_use]
pub fn is_authorized_api_key(
    options: &AuthOptions,
    x_api_key: Option<&str>,
) -> bool {
    match options.api_key.as_deref() {
        None => true,
        Some(expected) => verify_api_key(expected, x_api_key),
    }
}

/// Canonical JSON-RPC error body returned on auth failure.
#[must_use]
pub fn unauthorized_body() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": null,
        "error": {
            "code": UNAUTHORIZED_CODE,
            "message": "Unauthorized",
        }
    })
}

/// HTTP status for unauthorized MCP requests.
pub const UNAUTHORIZED_STATUS: u16 = 401;

/// Decide whether a path is always open (health + CORS preflight leave open).
#[must_use]
pub fn is_auth_bypass_path(path: &str) -> bool {
    let p = path.trim_end_matches('/');
    p.ends_with("/health") || p.eq_ignore_ascii_case("/health")
}

/// HTTP method that is always open for CORS preflight.
#[must_use]
pub fn is_cors_preflight(method: &str) -> bool {
    method.eq_ignore_ascii_case("OPTIONS")
}

/// Combined gate: bypass health/OPTIONS; else apply apiKey decision.
#[must_use]
pub fn authorize_request(
    options: &AuthOptions,
    method: &str,
    path: &str,
    x_api_key: Option<&str>,
) -> AuthDecision {
    if is_cors_preflight(method) || is_auth_bypass_path(path) {
        return AuthDecision::Allow;
    }
    if is_authorized_api_key(options, x_api_key) {
        AuthDecision::Allow
    } else {
        AuthDecision::Deny
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthDecision {
    Allow,
    Deny,
}

impl AuthDecision {
    #[must_use]
    pub fn is_allow(self) -> bool {
        matches!(self, Self::Allow)
    }

    #[must_use]
    pub fn is_deny(self) -> bool {
        matches!(self, Self::Deny)
    }
}

/// Response headers for a 401 denial.
#[must_use]
pub fn unauthorized_headers() -> Vec<(String, String)> {
    vec![
        (
            "WWW-Authenticate".into(),
            WWW_AUTHENTICATE.into(),
        ),
        ("Content-Type".into(), "application/json".into()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const API_KEY: &str = "s3cret-key";

    #[test]
    fn verify_api_key_accepts_matching() {
        assert!(verify_api_key(API_KEY, Some(API_KEY)));
    }

    #[test]
    fn verify_api_key_rejects_wrong() {
        assert!(!verify_api_key(API_KEY, Some("wrong")));
    }

    #[test]
    fn verify_api_key_rejects_missing_or_empty() {
        assert!(!verify_api_key(API_KEY, None));
        assert!(!verify_api_key(API_KEY, Some("")));
    }

    #[test]
    fn constant_time_eq_length_mismatch() {
        assert!(!constant_time_eq(b"ab", b"a"));
        assert!(constant_time_eq(b"ab", b"ab"));
        assert!(!constant_time_eq(b"ab", b"ac"));
    }

    #[test]
    fn read_header_map_case_insensitive() {
        let mut m = BTreeMap::new();
        m.insert("X-API-Key".into(), API_KEY.into());
        assert_eq!(read_header_map(&m, "x-api-key").as_deref(), Some(API_KEY));
        assert_eq!(read_header_map(&m, "X-API-KEY").as_deref(), Some(API_KEY));
        assert!(read_header_map(&m, "authorization").is_none());
    }

    #[test]
    fn read_header_multi_takes_first() {
        let mut m = BTreeMap::new();
        m.insert("x-api-key".into(), vec![API_KEY.into(), "other".into()]);
        assert_eq!(read_header_multi(&m, "X-API-Key").as_deref(), Some(API_KEY));
        let empty: BTreeMap<String, Vec<String>> = BTreeMap::new();
        assert!(read_header_multi(&empty, "x-api-key").is_none());
    }

    #[test]
    fn is_authorized_api_key_precedence() {
        let with = AuthOptions::with_api_key(API_KEY);
        let none = AuthOptions::none();
        assert!(is_authorized_api_key(&none, None));
        assert!(!is_authorized_api_key(&with, None));
        assert!(!is_authorized_api_key(&with, Some("wrong")));
        assert!(is_authorized_api_key(&with, Some(API_KEY)));
        assert!(is_auth_enabled(&with));
        assert!(!is_auth_enabled(&none));
    }

    #[test]
    fn unauthorized_body_shape() {
        let body = unauthorized_body();
        assert_eq!(body["jsonrpc"], "2.0");
        assert!(body["id"].is_null());
        assert_eq!(body["error"]["code"], UNAUTHORIZED_CODE);
        assert_eq!(body["error"]["message"], "Unauthorized");
    }

    #[test]
    fn authorize_request_health_and_options_open() {
        let opts = AuthOptions::with_api_key(API_KEY);
        assert!(authorize_request(&opts, "GET", "/mcp/health", None).is_allow());
        assert!(authorize_request(&opts, "OPTIONS", "/mcp", None).is_allow());
        assert!(authorize_request(&opts, "POST", "/mcp", None).is_deny());
        assert!(authorize_request(&opts, "POST", "/mcp", Some(API_KEY)).is_allow());
        assert!(authorize_request(&AuthOptions::none(), "POST", "/mcp", None).is_allow());
    }

    #[test]
    fn unauthorized_headers_include_www_authenticate() {
        let h = unauthorized_headers();
        assert!(h.iter().any(|(k, v)| k == "WWW-Authenticate" && v == WWW_AUTHENTICATE));
        assert_eq!(UNAUTHORIZED_STATUS, 401);
    }

    #[test]
    fn wave4_product_http_auth_surface() {
        // Dense smoke: hash stability + dual reject paths.
        let a = sha256_bytes("a");
        let b = sha256_bytes("a");
        assert!(constant_time_eq(&a, &b));
        assert!(!verify_api_key("x", Some("y")));
        assert!(is_auth_bypass_path("/mcp/health/"));
        assert!(is_cors_preflight("options"));
    }
}
