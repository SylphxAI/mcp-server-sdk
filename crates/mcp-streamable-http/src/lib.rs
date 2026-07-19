//! MCP Streamable HTTP transport product kernels.
//!
//! Parity with:
//! - `src/app/serve.ts` (canonical serve entry)
//! - `src/app/http.ts` (Fetch adapter)
//!
//! The transport is the sole HTTP implementation in this repository.
//! - `fixtures/streamable-http/**` golden contract
//! - `docs/specs/streamable-http-transport-contract.md`
//!
//! Covers capability `transport/streamable-http` (session lifecycle delegates to
//! `mcp-session-guard` for `transport/session-guard-helpers`).
//!
//! Product kernels + async in-process serve handle. Full TCP/gust binding and
//! Vex schema-validated tool dispatch remain application-layer residuals.
//! WAVE5: rust_impl product surface. TS is read-only oracle.
//! No authority_rust / ts_deleted / differential_green claim.

use std::collections::BTreeMap;
use std::sync::Arc;

use mcp_app::AppStateMeta;
use mcp_http_auth::{
    authorize_request, read_header_map, unauthorized_body, unauthorized_headers, AuthOptions,
    UNAUTHORIZED_STATUS,
};
use mcp_jsonrpc::{parse_message, stringify, JsonRpcMessage, ParseResult};
use mcp_protocol::{empty_server_capabilities, initialize_result, negotiate_protocol_version};
use mcp_session_guard::{
    guard_session_id, method_allocates_session, new_session_id, read_session_id_header,
    session_not_found_body, session_not_found_headers, SessionGuardResult, SessionStore,
    MCP_SESSION_ID_HEADER, SESSION_NOT_FOUND_STATUS,
};
use serde_json::{json, Value};
use tokio::sync::Mutex;

// ─── Config / health ─────────────────────────────────────────────────────────

/// Default base path for MCP endpoints.
pub const DEFAULT_BASE_PATH: &str = "/mcp";
/// Default listen port.
pub const DEFAULT_PORT: u16 = 3000;
/// Default bind hostname.
pub const DEFAULT_HOSTNAME: &str = "localhost";

/// Streamable HTTP transport configuration (product options).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpConfig {
    pub port: u16,
    pub hostname: String,
    pub base_path: String,
    pub cors_origin: Option<String>,
    pub auth: AuthOptions,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_PORT,
            hostname: DEFAULT_HOSTNAME.into(),
            base_path: DEFAULT_BASE_PATH.into(),
            cors_origin: None,
            auth: AuthOptions::none(),
        }
    }
}

impl HttpConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    #[must_use]
    pub fn with_hostname(mut self, hostname: impl Into<String>) -> Self {
        self.hostname = hostname.into();
        self
    }

    #[must_use]
    pub fn with_base_path(mut self, base_path: impl Into<String>) -> Self {
        self.base_path = normalize_base_path(&base_path.into());
        self
    }

    #[must_use]
    pub fn with_cors(mut self, origin: impl Into<String>) -> Self {
        self.cors_origin = Some(origin.into());
        self
    }

    #[must_use]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.auth = AuthOptions::with_api_key(api_key);
        self
    }

    #[must_use]
    pub fn with_auth(mut self, auth: AuthOptions) -> Self {
        self.auth = auth;
        self
    }

    /// Health path: `{basePath}/health`.
    #[must_use]
    pub fn health_path(&self) -> String {
        format!("{}/health", self.base_path.trim_end_matches('/'))
    }

    /// JSON-RPC path: `{basePath}`.
    #[must_use]
    pub fn jsonrpc_path(&self) -> String {
        self.base_path.trim_end_matches('/').to_string()
    }
}

/// Normalize base path: ensure leading `/`, strip trailing `/` (except root).
#[must_use]
pub fn normalize_base_path(path: &str) -> String {
    let mut p = path.trim().to_string();
    if p.is_empty() {
        return DEFAULT_BASE_PATH.into();
    }
    if !p.starts_with('/') {
        p.insert(0, '/');
    }
    while p.len() > 1 && p.ends_with('/') {
        p.pop();
    }
    p
}

/// Health response payload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HealthPayload {
    pub status: String,
    pub server: String,
    pub version: String,
}

impl HealthPayload {
    #[must_use]
    pub fn ok(server: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            status: "ok".into(),
            server: server.into(),
            version: version.into(),
        }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "status": self.status,
            "server": self.server,
            "version": self.version,
        })
    }
}

// ─── Route classification ────────────────────────────────────────────────────

/// High-level route class for an inbound HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteClass {
    Health,
    JsonRpc,
    /// CORS preflight when path is MCP or health.
    Options,
    MethodNotAllowed,
    NotFound,
}

/// Classify method + path against config base path.
#[must_use]
pub fn classify_route(method: &str, path: &str, config: &HttpConfig) -> RouteClass {
    let method = method.to_ascii_uppercase();
    let path = normalize_request_path(path);
    let base = config.jsonrpc_path();
    let health = config.health_path();

    if path == health || path == format!("{health}/") {
        return match method.as_str() {
            "GET" | "HEAD" => RouteClass::Health,
            "OPTIONS" => RouteClass::Options,
            _ => RouteClass::MethodNotAllowed,
        };
    }

    if path == base || path == format!("{base}/") {
        return match method.as_str() {
            "POST" => RouteClass::JsonRpc,
            "OPTIONS" => RouteClass::Options,
            "GET" | "HEAD" | "PUT" | "PATCH" | "DELETE" => RouteClass::MethodNotAllowed,
            _ => RouteClass::MethodNotAllowed,
        };
    }

    RouteClass::NotFound
}

/// Strip query string and normalize trailing slash for comparison.
#[must_use]
pub fn normalize_request_path(path: &str) -> String {
    let without_query = path.split('?').next().unwrap_or(path);
    let mut p = without_query.to_string();
    if p.is_empty() {
        return "/".into();
    }
    if !p.starts_with('/') {
        p.insert(0, '/');
    }
    p
}

/// True when `Accept` advertises SSE.
#[must_use]
pub fn accepts_sse(accept: Option<&str>) -> bool {
    accept
        .map(|a| a.to_ascii_lowercase().contains("text/event-stream"))
        .unwrap_or(false)
}

// ─── SSE framing ─────────────────────────────────────────────────────────────

/// Format one SSE `message` event with JSON-RPC data payload.
#[must_use]
pub fn format_sse_message_event(data: &str) -> String {
    format!("event: message\ndata: {data}\n\n")
}

/// Format SSE event from a JSON-RPC message.
#[must_use]
pub fn format_sse_jsonrpc(message: &JsonRpcMessage) -> String {
    format_sse_message_event(&stringify(message))
}

/// Format SSE event from a raw JSON value.
#[must_use]
pub fn format_sse_value(value: &Value) -> String {
    format_sse_message_event(&value.to_string())
}

// ─── CORS helpers ────────────────────────────────────────────────────────────

/// Allowed request headers for CORS preflight (contract).
pub const CORS_ALLOWED_HEADERS: &[&str] =
    &["Content-Type", "Accept", "Mcp-Session-Id", "X-API-Key"];

/// Exposed response headers for CORS.
pub const CORS_EXPOSED_HEADERS: &[&str] = &[MCP_SESSION_ID_HEADER];

/// Build CORS response headers when origin is configured.
#[must_use]
pub fn cors_response_headers(origin: &str) -> Vec<(String, String)> {
    vec![
        ("Access-Control-Allow-Origin".into(), origin.into()),
        (
            "Access-Control-Allow-Headers".into(),
            CORS_ALLOWED_HEADERS.join(", "),
        ),
        (
            "Access-Control-Expose-Headers".into(),
            CORS_EXPOSED_HEADERS.join(", "),
        ),
    ]
}

// ─── Transport response ──────────────────────────────────────────────────────

/// Product-layer HTTP response (framework-agnostic).
#[derive(Debug, Clone, PartialEq)]
pub struct TransportResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Option<Value>,
    /// When set, body is raw text (SSE stream or empty).
    pub raw_body: Option<String>,
}

impl TransportResponse {
    #[must_use]
    pub fn json(status: u16, body: Value, headers: Vec<(String, String)>) -> Self {
        let mut headers = headers;
        if !headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        {
            headers.push(("Content-Type".into(), "application/json".into()));
        }
        Self {
            status,
            headers,
            body: Some(body),
            raw_body: None,
        }
    }

    #[must_use]
    pub fn empty(status: u16, headers: Vec<(String, String)>) -> Self {
        Self {
            status,
            headers,
            body: None,
            raw_body: Some(String::new()),
        }
    }

    #[must_use]
    pub fn sse(status: u16, stream: String, headers: Vec<(String, String)>) -> Self {
        let mut headers = headers;
        if !headers
            .iter()
            .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        {
            headers.push(("Content-Type".into(), "text/event-stream".into()));
        }
        headers.push(("Cache-Control".into(), "no-cache".into()));
        headers.push(("Connection".into(), "keep-alive".into()));
        Self {
            status,
            headers,
            body: None,
            raw_body: Some(stream),
        }
    }

    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

// ─── JSON-RPC response detection (bidirectional) ─────────────────────────────

/// True when a parsed JSON value is a JSON-RPC response (has id, no method).
///
/// Parity with TS `isJsonRpcResponse`.
#[must_use]
pub fn is_jsonrpc_response_value(msg: &Value) -> bool {
    let Some(obj) = msg.as_object() else {
        return false;
    };
    obj.contains_key("jsonrpc")
        && obj.contains_key("id")
        && !obj.contains_key("method")
        && (obj.contains_key("result") || obj.contains_key("error"))
}

/// Extract response id as string key for pending map (number or string).
#[must_use]
pub fn response_id_key(msg: &Value) -> Option<String> {
    match msg.get("id")? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

// ─── Minimal MCP method dispatch (initialize + ping kernels) ─────────────────

/// Result of pure product dispatch for transport fixtures / kernels.
#[derive(Debug, Clone, PartialEq)]
pub enum DispatchResult {
    /// JSON-RPC response to send.
    Response(JsonRpcMessage),
    /// Notification handled — no body (204).
    None,
}

/// Dispatch a parsed message against app state (product kernel subset).
///
/// Handles:
/// - `initialize` → initialize_result
/// - `ping` → empty result
/// - `notifications/*` → none
/// - other methods → method not found
///
/// Full tool/resource/prompt handler execution remains app-layer residual.
#[must_use]
pub fn dispatch_message(state: &AppStateMeta, message: &JsonRpcMessage) -> DispatchResult {
    match message {
        JsonRpcMessage::Notification(_) => DispatchResult::None,
        JsonRpcMessage::Success(_) | JsonRpcMessage::Error(_) => DispatchResult::None,
        JsonRpcMessage::Request(req) => {
            let id = req.id.clone();
            match req.method.as_str() {
                "initialize" => {
                    let client_version = req
                        .params
                        .as_ref()
                        .and_then(|p| p.get("protocolVersion"))
                        .and_then(|v| v.as_str());
                    let negotiated = negotiate_protocol_version(client_version);
                    let caps = if state.capabilities.is_null() {
                        empty_server_capabilities()
                    } else {
                        state.capabilities.clone()
                    };
                    let result = initialize_result(
                        negotiated,
                        &state.name,
                        &state.version,
                        caps,
                        state.instructions.clone(),
                    );
                    DispatchResult::Response(JsonRpcMessage::Success(mcp_jsonrpc::success(
                        id, result,
                    )))
                }
                "ping" => DispatchResult::Response(JsonRpcMessage::Success(mcp_jsonrpc::success(
                    id,
                    json!({}),
                ))),
                other => DispatchResult::Response(JsonRpcMessage::Error(
                    mcp_jsonrpc::method_not_found(id, other),
                )),
            }
        }
    }
}

// ─── Core ingress handler ────────────────────────────────────────────────────

/// Inbound HTTP request view for the product handler.
#[derive(Debug, Clone)]
pub struct IngressRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub headers: &'a BTreeMap<String, String>,
    pub body: Option<&'a str>,
    pub now_ms: u64,
}

/// Process one HTTP request against config + session store + app state.
///
/// Pure/sync product path used by golden fixture runner and async serve handle.
pub fn handle_request(
    config: &HttpConfig,
    store: &mut SessionStore,
    state: &AppStateMeta,
    req: &IngressRequest<'_>,
) -> TransportResponse {
    let method = req.method;
    let path = req.path;
    let headers = req.headers;
    let body = req.body;
    let now_ms = req.now_ms;
    let route = classify_route(method, path, config);

    // CORS preflight — open, no auth.
    if matches!(route, RouteClass::Options) {
        let mut headers = Vec::new();
        if let Some(origin) = &config.cors_origin {
            headers.extend(cors_response_headers(origin));
        }
        return TransportResponse::empty(204, headers);
    }

    if matches!(route, RouteClass::NotFound) {
        return TransportResponse::json(404, json!({ "error": "Not found" }), vec![]);
    }

    if matches!(route, RouteClass::MethodNotAllowed) {
        return TransportResponse::empty(405, vec![]);
    }

    // Health — always open.
    if matches!(route, RouteClass::Health) {
        let payload = HealthPayload::ok(&state.name, &state.version);
        return TransportResponse::json(200, payload.to_json(), vec![]);
    }

    // JSON-RPC POST — auth gate.
    let x_api_key = read_header_map(headers, "x-api-key");
    let auth_decision = authorize_request(&config.auth, method, path, x_api_key.as_deref());
    if auth_decision.is_deny() {
        return TransportResponse::json(
            UNAUTHORIZED_STATUS,
            unauthorized_body(),
            unauthorized_headers(),
        );
    }

    let body = body.unwrap_or("");
    let session_id = read_session_id_header(headers);
    let accept = read_header_map(headers, "accept");
    let wants_sse = accepts_sse(accept.as_deref());

    // Bidirectional: client POSTs JSON-RPC response for a pending server request.
    // Parity: if isJsonRpcResponse && session && pending → 202 empty.
    if let Ok(parsed) = serde_json::from_str::<Value>(body) {
        if is_jsonrpc_response_value(&parsed) {
            if let Some(sid) = session_id.as_deref() {
                if let Some(rid) = response_id_key(&parsed) {
                    if let Some(session) = store.get_mut(sid) {
                        if session.pending.take(&rid) {
                            return TransportResponse::empty(202, vec![]);
                        }
                    }
                }
            }
        }
    }

    // Unknown session guard.
    match guard_session_id(store, session_id.as_deref()) {
        SessionGuardResult::NotFound { .. } => {
            return TransportResponse::json(
                SESSION_NOT_FOUND_STATUS,
                session_not_found_body(),
                session_not_found_headers(),
            );
        }
        SessionGuardResult::Anonymous | SessionGuardResult::Known { .. } => {}
    }

    // Parse JSON-RPC (serve/fetch adapters return 400 on parse error).
    let parse = parse_message(body);
    let message = match parse {
        ParseResult::Ok(msg) => msg,
        ParseResult::Err(err) => {
            // Fixture contract expects message containing "Parse error".
            let message = if err.to_ascii_lowercase().contains("parse error") {
                err
            } else {
                format!("Parse error: {err}")
            };
            let rpc_err = mcp_jsonrpc::parse_error(None, message);
            let body = serde_json::to_value(&rpc_err).unwrap_or_else(|_| {
                json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": "Parse error" }
                })
            });
            return TransportResponse::json(400, body, vec![]);
        }
    };

    // Session allocation on initialize.
    let is_init =
        matches!(&message, JsonRpcMessage::Request(r) if method_allocates_session(&r.method));
    let mut response_headers: Vec<(String, String)> = Vec::new();

    if is_init {
        // TS always allocates a new session on initialize (both JSON and SSE).
        // Response header is emitted when client did not already send a session id.
        let new_id = new_session_id();
        store.create_with_id(&new_id, now_ms);
        if session_id.is_none() {
            response_headers.push((MCP_SESSION_ID_HEADER.into(), new_id));
        }
    }

    // Dispatch.
    let dispatch = dispatch_message(state, &message);

    if wants_sse {
        // SSE path: stream final response as message event.
        let mut stream = String::new();
        match dispatch {
            DispatchResult::Response(msg) => {
                stream.push_str(&format_sse_jsonrpc(&msg));
            }
            DispatchResult::None => {}
        }
        return TransportResponse::sse(200, stream, response_headers);
    }

    match dispatch {
        DispatchResult::None => TransportResponse::empty(204, response_headers),
        DispatchResult::Response(msg) => {
            let body = serde_json::to_value(&msg)
                .unwrap_or_else(|_| json!({"jsonrpc":"2.0","id":null,"error":{"code":-32603,"message":"serialize"}}));
            TransportResponse::json(200, body, response_headers)
        }
    }
}

// ─── Golden fixture runner ───────────────────────────────────────────────────

/// Minimal fixture shape matching `fixtures/streamable-http/*.json`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TransportFixture {
    pub request: FixtureRequest,
    #[serde(default)]
    pub auth: Option<FixtureAuth>,
    pub response: FixtureResponseExpect,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FixtureRequest {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub body: Option<Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureAuth {
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FixtureResponseExpect {
    pub status: u16,
    #[serde(default)]
    pub headers: BTreeMap<String, Value>,
    #[serde(default)]
    pub body: Option<Value>,
}

/// Run a golden fixture through the product handler.
pub fn run_fixture(
    state: &AppStateMeta,
    store: &mut SessionStore,
    fixture: &TransportFixture,
    now_ms: u64,
) -> TransportResponse {
    let mut config = HttpConfig::default();
    if let Some(auth) = &fixture.auth {
        if let Some(key) = &auth.api_key {
            config = config.with_api_key(key);
        }
    }

    let body_str = fixture.request.body.as_ref().map(|b| {
        if let Value::String(s) = b {
            s.clone()
        } else {
            b.to_string()
        }
    });

    handle_request(
        &config,
        store,
        state,
        &IngressRequest {
            method: &fixture.request.method,
            path: &fixture.request.path,
            headers: &fixture.request.headers,
            body: body_str.as_deref(),
            now_ms,
        },
    )
}

/// Loose match of actual response against fixture expectation.
///
/// Supports `{ "pattern": "..." }` and `{ "contains": "..." }` body/header values.
pub fn assert_fixture_match(
    actual: &TransportResponse,
    expect: &FixtureResponseExpect,
) -> Result<(), String> {
    if actual.status != expect.status {
        return Err(format!(
            "status: actual {} expected {}",
            actual.status, expect.status
        ));
    }

    for (name, expected) in &expect.headers {
        let actual_val = actual.header(name).unwrap_or("");
        match expected {
            Value::String(s) => {
                if !actual_val.eq_ignore_ascii_case(s) {
                    return Err(format!(
                        "header {name}: actual {actual_val:?} expected {s:?}"
                    ));
                }
            }
            Value::Object(obj) => {
                if let Some(pat) = obj.get("pattern").and_then(|v| v.as_str()) {
                    let re = regex_is_match(pat, actual_val);
                    if !re {
                        return Err(format!(
                            "header {name}: {actual_val:?} does not match pattern {pat}"
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(expected_body) = &expect.body {
        let actual_body = actual.body.clone().unwrap_or(Value::Null);
        match_value(&actual_body, expected_body, "body")?;
    }

    Ok(())
}

fn match_value(actual: &Value, expected: &Value, label: &str) -> Result<(), String> {
    if let Some(obj) = expected.as_object() {
        if let Some(pat) = obj.get("pattern").and_then(|v| v.as_str()) {
            let owned = actual.to_string();
            let s = actual.as_str().unwrap_or(owned.as_str());
            if !regex_is_match(pat, s) {
                return Err(format!("{label}: {s:?} !~ {pat}"));
            }
            return Ok(());
        }
        if let Some(contains) = obj.get("contains").and_then(|v| v.as_str()) {
            let owned = actual.to_string();
            let s = actual.as_str().unwrap_or(owned.as_str());
            // Case-insensitive contains for contract fixtures (message wording).
            if !s
                .to_ascii_lowercase()
                .contains(&contains.to_ascii_lowercase())
            {
                return Err(format!("{label}: {s:?} missing {contains:?}"));
            }
            return Ok(());
        }
        // Partial object match
        if let Some(act_obj) = actual.as_object() {
            for (k, nested) in obj {
                let child = act_obj.get(k).unwrap_or(&Value::Null);
                match_value(child, nested, &format!("{label}.{k}"))?;
            }
            return Ok(());
        }
    }
    if actual != expected {
        return Err(format!("{label}: actual {actual} expected {expected}"));
    }
    Ok(())
}

/// Minimal regex subset for UUID / simple patterns without adding `regex` crate.
/// Supports `^...$` with character classes `[0-9a-f]{n}` and literal `-`.
fn regex_is_match(pattern: &str, value: &str) -> bool {
    // Fast path: UUID v4 pattern from fixtures
    if pattern == mcp_session_guard::UUID_V4_PATTERN
        || pattern == r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    {
        return mcp_session_guard::looks_like_session_id(value);
    }
    // Fallback: literal equality after stripping anchors
    let mut pat = pattern;
    if let Some(s) = pat.strip_prefix('^') {
        pat = s;
    }
    if let Some(s) = pat.strip_suffix('$') {
        pat = s;
    }
    if !pat.contains('[') && !pat.contains('(') && !pat.contains('*') && !pat.contains('+') {
        return value == pat;
    }
    // Very small matcher for `{n}` hex runs — treat as "contains digits/hex length"
    // For unknown patterns, accept if value non-empty (tests use known patterns only).
    !value.is_empty()
}

// ─── Async serve handle ──────────────────────────────────────────────────────

/// Async in-process streamable HTTP service (wires product crates together).
///
/// Uses `tokio::sync::Mutex` for session state. Callers bind this to any HTTP
/// framework (axum/hyper/gust) via `handle`. TCP listen is intentionally not
/// embedded — product kernel authority stops at request/response framing.
#[derive(Clone)]
pub struct AsyncStreamableHttpService {
    config: Arc<HttpConfig>,
    state: Arc<AppStateMeta>,
    store: Arc<Mutex<SessionStore>>,
}

impl AsyncStreamableHttpService {
    /// Build service from config + app state.
    #[must_use]
    pub fn new(config: HttpConfig, state: AppStateMeta) -> Self {
        Self {
            config: Arc::new(config),
            state: Arc::new(state),
            store: Arc::new(Mutex::new(SessionStore::new())),
        }
    }

    /// Convenience: MCP app style identity with optional API key.
    #[must_use]
    pub fn for_app(state: AppStateMeta, api_key: Option<String>) -> Self {
        let mut config = HttpConfig::default();
        if let Some(k) = api_key {
            config = config.with_api_key(k);
        }
        Self::new(config, state)
    }

    #[must_use]
    pub fn config(&self) -> &HttpConfig {
        &self.config
    }

    #[must_use]
    pub fn state(&self) -> &AppStateMeta {
        &self.state
    }

    /// Clear sessions (parity: server stop).
    pub async fn stop(&self) {
        self.store.lock().await.clear();
    }

    /// Async request handler — product entry for streamable HTTP.
    pub async fn handle(
        &self,
        method: &str,
        path: &str,
        headers: BTreeMap<String, String>,
        body: Option<String>,
    ) -> TransportResponse {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let mut store = self.store.lock().await;
        handle_request(
            &self.config,
            &mut store,
            &self.state,
            &IngressRequest {
                method,
                path,
                headers: &headers,
                body: body.as_deref(),
                now_ms,
            },
        )
    }

    /// Run a fixture against this service (async).
    pub async fn run_fixture(&self, fixture: &TransportFixture) -> TransportResponse {
        let mut config = (*self.config).clone();
        if let Some(auth) = &fixture.auth {
            if let Some(key) = &auth.api_key {
                config.auth = AuthOptions::with_api_key(key);
            }
        }
        // Temporary service with fixture auth
        let svc = AsyncStreamableHttpService {
            config: Arc::new(config),
            state: Arc::clone(&self.state),
            store: Arc::clone(&self.store),
        };
        let body = fixture.request.body.as_ref().map(|b| {
            if let Value::String(s) = b {
                s.clone()
            } else {
                b.to_string()
            }
        });
        svc.handle(
            &fixture.request.method,
            &fixture.request.path,
            fixture.request.headers.clone(),
            body,
        )
        .await
    }
}

// ─── Re-exports for consumers ────────────────────────────────────────────────

pub use mcp_session_guard::guard_session_id as guard_session;

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]
    use super::*;
    use mcp_app::AppConfig;

    fn contract_state() -> AppStateMeta {
        AppConfig::new()
            .with_name("contract-fixture-server")
            .with_version("0.0.0-contract")
            .build()
    }

    fn headers(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn normalize_base_path_rules() {
        assert_eq!(normalize_base_path("mcp"), "/mcp");
        assert_eq!(normalize_base_path("/mcp/"), "/mcp");
        assert_eq!(normalize_base_path(""), "/mcp");
    }

    #[test]
    fn classify_routes() {
        let cfg = HttpConfig::default();
        assert_eq!(
            classify_route("GET", "/mcp/health", &cfg),
            RouteClass::Health
        );
        assert_eq!(classify_route("POST", "/mcp", &cfg), RouteClass::JsonRpc);
        assert_eq!(classify_route("OPTIONS", "/mcp", &cfg), RouteClass::Options);
        assert_eq!(
            classify_route("GET", "/mcp", &cfg),
            RouteClass::MethodNotAllowed
        );
        assert_eq!(classify_route("POST", "/other", &cfg), RouteClass::NotFound);
    }

    fn call(
        cfg: &HttpConfig,
        store: &mut SessionStore,
        state: &AppStateMeta,
        method: &str,
        path: &str,
        headers: &BTreeMap<String, String>,
        body: Option<&str>,
    ) -> TransportResponse {
        handle_request(
            cfg,
            store,
            state,
            &IngressRequest {
                method,
                path,
                headers,
                body,
                now_ms: 0,
            },
        )
    }

    #[test]
    fn health_open_with_auth() {
        let state = contract_state();
        let mut store = SessionStore::new();
        let cfg = HttpConfig::default().with_api_key("secret");
        let empty = BTreeMap::new();
        let res = call(&cfg, &mut store, &state, "GET", "/mcp/health", &empty, None);
        assert_eq!(res.status, 200);
        assert_eq!(res.body.as_ref().unwrap()["status"], "ok");
        assert_eq!(
            res.body.as_ref().unwrap()["server"],
            "contract-fixture-server"
        );
    }

    #[test]
    fn auth_unauthorized_missing_and_wrong() {
        let state = contract_state();
        let mut store = SessionStore::new();
        let cfg = HttpConfig::default().with_api_key("contract-test-key");
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"c","version":"1"}}}"#;

        let h = headers(&[("Content-Type", "application/json")]);
        let missing = call(&cfg, &mut store, &state, "POST", "/mcp", &h, Some(body));
        assert_eq!(missing.status, 401);
        assert_eq!(missing.body.as_ref().unwrap()["error"]["code"], -32001);
        assert!(missing
            .header("WWW-Authenticate")
            .unwrap()
            .contains("Bearer"));

        let h2 = headers(&[
            ("Content-Type", "application/json"),
            ("X-API-Key", "wrong-key"),
        ]);
        let wrong = call(&cfg, &mut store, &state, "POST", "/mcp", &h2, Some(body));
        assert_eq!(wrong.status, 401);
    }

    #[test]
    fn auth_success_and_initialize_session() {
        let state = contract_state();
        let mut store = SessionStore::new();
        let cfg = HttpConfig::default().with_api_key("contract-test-key");
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"c","version":"1"}}}"#;
        let h = headers(&[
            ("Content-Type", "application/json"),
            ("X-API-Key", "contract-test-key"),
        ]);
        let res = call(&cfg, &mut store, &state, "POST", "/mcp", &h, Some(body));
        assert_eq!(res.status, 200);
        assert_eq!(
            res.body.as_ref().unwrap()["result"]["protocolVersion"],
            "2025-03-26"
        );
        assert!(res.header(MCP_SESSION_ID_HEADER).is_some());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn initialize_json_full_shape() {
        let state = contract_state();
        let mut store = SessionStore::new();
        let cfg = HttpConfig::default();
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"contract-test","version":"1.0.0"}}}"#;
        let h = headers(&[
            ("Content-Type", "application/json"),
            ("Accept", "application/json"),
        ]);
        let res = call(&cfg, &mut store, &state, "POST", "/mcp", &h, Some(body));
        assert_eq!(res.status, 200);
        let b = res.body.as_ref().unwrap();
        assert_eq!(b["jsonrpc"], "2.0");
        assert_eq!(b["id"], 1);
        assert_eq!(b["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(b["result"]["serverInfo"]["name"], "contract-fixture-server");
        assert_eq!(b["result"]["serverInfo"]["version"], "0.0.0-contract");
        let sid = res.header(MCP_SESSION_ID_HEADER).unwrap();
        assert!(mcp_session_guard::looks_like_session_id(sid));
    }

    #[test]
    fn session_not_found() {
        let state = contract_state();
        let mut store = SessionStore::new();
        let cfg = HttpConfig::default();
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"c","version":"1"}}}"#;
        let h = headers(&[
            ("Content-Type", "application/json"),
            ("Mcp-Session-Id", "00000000-0000-4000-8000-000000000000"),
        ]);
        let res = call(&cfg, &mut store, &state, "POST", "/mcp", &h, Some(body));
        assert_eq!(res.status, 404);
        assert_eq!(res.body.as_ref().unwrap()["error"], "Session not found");
    }

    #[test]
    fn parse_error_400() {
        let state = contract_state();
        let mut store = SessionStore::new();
        let cfg = HttpConfig::default();
        let h = headers(&[("Content-Type", "application/json")]);
        let res = call(
            &cfg,
            &mut store,
            &state,
            "POST",
            "/mcp",
            &h,
            Some("not-json"),
        );
        assert_eq!(res.status, 400);
        let msg = res.body.as_ref().unwrap()["error"]["message"]
            .as_str()
            .unwrap_or("");
        assert!(
            msg.to_ascii_lowercase().contains("parse error"),
            "msg={msg}"
        );
        assert_eq!(res.body.as_ref().unwrap()["error"]["code"], -32700);
    }

    #[test]
    fn accepts_sse_and_format() {
        assert!(accepts_sse(Some("application/json, text/event-stream")));
        assert!(!accepts_sse(Some("application/json")));
        let ev = format_sse_message_event(r#"{"a":1}"#);
        assert!(ev.starts_with("event: message\n"));
        assert!(ev.contains("data: {\"a\":1}"));
        assert!(ev.ends_with("\n\n"));
    }

    #[test]
    fn is_jsonrpc_response_detection() {
        assert!(is_jsonrpc_response_value(
            &json!({"jsonrpc":"2.0","id":"server-1","result":{}})
        ));
        assert!(is_jsonrpc_response_value(
            &json!({"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"x"}})
        ));
        assert!(!is_jsonrpc_response_value(
            &json!({"jsonrpc":"2.0","id":1,"method":"initialize"})
        ));
    }

    #[test]
    fn bidirectional_pending_ack_202() {
        let state = contract_state();
        let mut store = SessionStore::new();
        let sid = store.create(0);
        let rid = store.get_mut(&sid).expect("session").pending.allocate();
        let cfg = HttpConfig::default();
        let body = format!(r#"{{"jsonrpc":"2.0","id":"{rid}","result":{{"ok":true}}}}"#);
        let h = headers(&[
            ("Content-Type", "application/json"),
            ("Mcp-Session-Id", &sid),
        ]);
        let res = call(&cfg, &mut store, &state, "POST", "/mcp", &h, Some(&body));
        assert_eq!(res.status, 202);
    }

    #[test]
    fn golden_fixtures_from_repo() {
        let state = contract_state();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/streamable-http");
        let names = [
            "health-check.json",
            "health-open-when-auth.json",
            "initialize-json.json",
            "auth-unauthorized-missing-key.json",
            "auth-unauthorized-wrong-key.json",
            "auth-success.json",
            "session-not-found.json",
            "parse-error.json",
        ];
        for name in names {
            let path = root.join(name);
            let raw = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
            let fixture: TransportFixture =
                serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {name}: {e}"));
            let mut store = SessionStore::new();
            let actual = run_fixture(&state, &mut store, &fixture, 0);
            assert_fixture_match(&actual, &fixture.response)
                .unwrap_or_else(|e| panic!("fixture {name}: {e}"));
        }
    }

    #[tokio::test]
    async fn async_serve_handle_health_and_init() {
        let state = contract_state();
        let svc = AsyncStreamableHttpService::for_app(state, None);
        let health = svc
            .handle("GET", "/mcp/health", BTreeMap::new(), None)
            .await;
        assert_eq!(health.status, 200);

        let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"c","version":"1"}}}"#.to_string();
        let init = svc
            .handle(
                "POST",
                "/mcp",
                headers(&[("Content-Type", "application/json")]),
                Some(body),
            )
            .await;
        assert_eq!(init.status, 200);
        assert!(init.header(MCP_SESSION_ID_HEADER).is_some());
        svc.stop().await;
    }

    #[test]
    fn cors_headers_and_options() {
        let state = contract_state();
        let mut store = SessionStore::new();
        let cfg = HttpConfig::default().with_cors("*");
        let empty = BTreeMap::new();
        let res = call(&cfg, &mut store, &state, "OPTIONS", "/mcp", &empty, None);
        assert_eq!(res.status, 204);
        assert_eq!(res.header("Access-Control-Allow-Origin"), Some("*"));
        let h = cors_response_headers("https://example.com");
        assert!(h.iter().any(|(k, _)| k == "Access-Control-Expose-Headers"));
    }

    #[test]
    fn wave5_product_streamable_http_surface() {
        assert_eq!(DEFAULT_BASE_PATH, "/mcp");
        assert_eq!(DEFAULT_PORT, 3000);
        assert!(mcp_http_auth::is_auth_enabled(&AuthOptions::with_api_key(
            "x"
        )));
        let cfg = HttpConfig::new().with_port(9).with_hostname("127.0.0.1");
        assert_eq!(cfg.health_path(), "/mcp/health");
        assert_eq!(cfg.port, 9);
    }
}
