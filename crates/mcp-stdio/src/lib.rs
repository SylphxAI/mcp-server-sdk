//! Stdio NDJSON framing product kernels.
//!
//! Parity with `src/transports/stdio.ts` and `src/app/stdio.ts` for pure framing:
//! line buffering, server→client request id minting, response routing detection,
//! and encode/decode of newline-delimited JSON-RPC messages.
//!
//! Async read loops and process stdin/stdout binding stay application-layer.
//! WAVE4: rust_impl product surface. TS is read-only oracle.

use mcp_jsonrpc::{stringify, JsonRpcMessage, JsonRpcRequest, JsonRpcSuccess};
use serde_json::Value;

/// Prefix used for server-originated request ids (`server-1`, `server-2`, …).
pub const SERVER_REQUEST_ID_PREFIX: &str = "server-";

/// Default timeout for server→client requests (milliseconds).
pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 30_000;

/// Line buffer for NDJSON stdin framing.
#[derive(Debug, Clone, Default)]
pub struct LineBuffer {
    buf: String,
}

impl LineBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a decoded UTF-8 chunk and yield complete non-empty lines.
    pub fn push_str(&mut self, chunk: &str) -> Vec<String> {
        self.buf.push_str(chunk);
        let mut lines = Vec::new();
        while let Some(idx) = self.buf.find('\n') {
            let line = self.buf[..idx].trim().to_string();
            self.buf = self.buf[idx + 1..].to_string();
            if !line.is_empty() {
                lines.push(line);
            }
        }
        lines
    }

    /// Append raw bytes (lossy UTF-8) and yield complete lines.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Vec<String> {
        self.push_str(&String::from_utf8_lossy(bytes))
    }

    /// Remaining incomplete line (no trailing newline yet).
    #[must_use]
    pub fn remainder(&self) -> &str {
        &self.buf
    }

    /// Clear buffer state.
    pub fn clear(&mut self) {
        self.buf.clear();
    }
}

/// Encode a JSON-RPC message as a single NDJSON line (with trailing newline).
#[must_use]
pub fn encode_line(message: &JsonRpcMessage) -> String {
    format!("{}\n", stringify(message))
}

/// Encode a JSON-RPC request as NDJSON.
#[must_use]
pub fn encode_request_line(req: JsonRpcRequest) -> String {
    encode_line(&JsonRpcMessage::Request(req))
}

/// Encode a JSON-RPC success response as NDJSON.
#[must_use]
pub fn encode_success_line(ok: JsonRpcSuccess) -> String {
    encode_line(&JsonRpcMessage::Success(ok))
}

/// Encode an arbitrary JSON value as NDJSON line.
#[must_use]
pub fn encode_value_line(value: &Value) -> String {
    format!("{}\n", value)
}

/// Encode a pre-serialized JSON string as NDJSON line.
#[must_use]
pub fn encode_raw_line(json: &str) -> String {
    format!("{}\n", json.trim_end_matches('\n'))
}

/// Mint the next server→client request id (`server-{n}`).
#[must_use]
pub fn next_server_request_id(counter: u64) -> String {
    format!("{SERVER_REQUEST_ID_PREFIX}{counter}")
}

/// True when `id` looks like a server-originated request id.
#[must_use]
pub fn is_server_request_id(id: &str) -> bool {
    id.starts_with(SERVER_REQUEST_ID_PREFIX)
        && id.len() > SERVER_REQUEST_ID_PREFIX.len()
        && id[SERVER_REQUEST_ID_PREFIX.len()..]
            .chars()
            .all(|c| c.is_ascii_digit())
}

/// Classify a raw NDJSON line for the stdio loop.
#[derive(Debug, Clone, PartialEq)]
pub enum StdioLineClass {
    /// Response to a server→client request (sampling/elicitation).
    ServerResponse {
        id: String,
        result: Option<Value>,
        error_message: Option<String>,
    },
    /// Client→server message body (request or notification) as raw JSON string.
    ClientMessage(String),
    /// Unparseable / empty — ignore or error at transport layer.
    Invalid,
}

/// Inspect a trimmed NDJSON line and classify it.
#[must_use]
pub fn classify_line(line: &str) -> StdioLineClass {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return StdioLineClass::Invalid;
    }
    let parsed: Value = match serde_json::from_str(trimmed) {
        Ok(v) => v,
        Err(_) => return StdioLineClass::Invalid,
    };
    if let Some(id) = parsed.get("id").and_then(|v| v.as_str()) {
        if is_server_request_id(id) && (parsed.get("result").is_some() || parsed.get("error").is_some())
        {
            let error_message = parsed
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .map(str::to_string);
            let result = if parsed.get("error").is_some() {
                None
            } else {
                parsed.get("result").cloned()
            };
            return StdioLineClass::ServerResponse {
                id: id.to_string(),
                result,
                error_message: error_message.or_else(|| {
                    if parsed.get("error").is_some() {
                        Some("Request failed".into())
                    } else {
                        None
                    }
                }),
            };
        }
    }
    StdioLineClass::ClientMessage(trimmed.to_string())
}

/// Pending server→client request registry (pure bookkeeping).
#[derive(Debug, Clone, Default)]
pub struct PendingRequests {
    next_id: u64,
    /// request id → still waiting
    waiting: std::collections::BTreeSet<String>,
}

impl PendingRequests {
    #[must_use]
    pub fn new() -> Self {
        Self {
            next_id: 1,
            waiting: std::collections::BTreeSet::new(),
        }
    }

    /// Allocate a new request id and register it as pending.
    pub fn allocate(&mut self) -> String {
        let id = next_server_request_id(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.waiting.insert(id.clone());
        id
    }

    /// Take a pending request (resolve or reject path). Returns false if unknown.
    pub fn take(&mut self, id: &str) -> bool {
        self.waiting.remove(id)
    }

    #[must_use]
    pub fn is_pending(&self, id: &str) -> bool {
        self.waiting.contains(id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.waiting.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }

    pub fn clear(&mut self) {
        self.waiting.clear();
    }
}

/// Build a server→client JSON-RPC request object.
#[must_use]
pub fn server_request(id: impl Into<String>, method: impl Into<String>, params: Option<Value>) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("jsonrpc".into(), Value::String("2.0".into()));
    map.insert("id".into(), Value::String(id.into()));
    map.insert("method".into(), Value::String(method.into()));
    if let Some(p) = params {
        map.insert("params".into(), p);
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcp_jsonrpc::{request, success, RequestId};
    use serde_json::json;

    #[test]
    fn line_buffer_splits_complete_lines() {
        let mut buf = LineBuffer::new();
        assert!(buf.push_str("{\"a\":1}\n{\"b\":2}\n").len() == 2);
        assert_eq!(buf.remainder(), "");
        let partial = buf.push_str("{\"c\":");
        assert!(partial.is_empty());
        let done = buf.push_str("3}\n");
        assert_eq!(done, vec![r#"{"c":3}"#]);
    }

    #[test]
    fn line_buffer_skips_empty_lines() {
        let mut buf = LineBuffer::new();
        let lines = buf.push_str("\n  \n{\"x\":1}\n");
        assert_eq!(lines, vec![r#"{"x":1}"#]);
    }

    #[test]
    fn encode_and_classify_client_message() {
        let req = request(RequestId::Number(1), "initialize", Some(json!({})));
        let line = encode_request_line(req);
        assert!(line.ends_with('\n'));
        match classify_line(line.trim()) {
            StdioLineClass::ClientMessage(s) => {
                assert!(s.contains("initialize"));
            }
            other => panic!("expected client message, got {other:?}"),
        }
    }

    #[test]
    fn classify_server_response() {
        let ok = json!({"jsonrpc":"2.0","id":"server-1","result":{"ok":true}});
        match classify_line(&ok.to_string()) {
            StdioLineClass::ServerResponse {
                id,
                result,
                error_message,
            } => {
                assert_eq!(id, "server-1");
                assert_eq!(result, Some(json!({"ok": true})));
                assert!(error_message.is_none());
            }
            other => panic!("expected server response {other:?}"),
        }
        let err = json!({"jsonrpc":"2.0","id":"server-2","error":{"code":-1,"message":"nope"}});
        match classify_line(&err.to_string()) {
            StdioLineClass::ServerResponse {
                id,
                result,
                error_message,
            } => {
                assert_eq!(id, "server-2");
                assert!(result.is_none());
                assert_eq!(error_message.as_deref(), Some("nope"));
            }
            other => panic!("expected error response {other:?}"),
        }
    }

    #[test]
    fn server_request_id_helpers() {
        assert_eq!(next_server_request_id(1), "server-1");
        assert!(is_server_request_id("server-42"));
        assert!(!is_server_request_id("client-1"));
        assert!(!is_server_request_id("server-"));
        assert!(!is_server_request_id("server-x"));
    }

    #[test]
    fn pending_requests_allocate_and_take() {
        let mut p = PendingRequests::new();
        let id1 = p.allocate();
        let id2 = p.allocate();
        assert_eq!(id1, "server-1");
        assert_eq!(id2, "server-2");
        assert_eq!(p.len(), 2);
        assert!(p.take(&id1));
        assert!(!p.take(&id1));
        assert!(p.is_pending(&id2));
        p.clear();
        assert!(p.is_empty());
    }

    #[test]
    fn server_request_shape() {
        let r = server_request("server-1", "sampling/createMessage", Some(json!({"maxTokens":1})));
        assert_eq!(r["jsonrpc"], "2.0");
        assert_eq!(r["id"], "server-1");
        assert_eq!(r["method"], "sampling/createMessage");
        assert_eq!(r["params"]["maxTokens"], 1);
    }

    #[test]
    fn encode_raw_and_value() {
        assert_eq!(encode_raw_line(r#"{"a":1}"#), "{\"a\":1}\n");
        assert_eq!(encode_value_line(&json!({"b":2})), "{\"b\":2}\n");
        let s = success(RequestId::Number(1), json!({}));
        assert!(encode_success_line(s).contains("result"));
        assert_eq!(DEFAULT_REQUEST_TIMEOUT_MS, 30_000);
    }

    #[test]
    fn wave4_product_stdio_invalid_lines() {
        assert!(matches!(classify_line(""), StdioLineClass::Invalid));
        assert!(matches!(classify_line("not-json"), StdioLineClass::Invalid));
        // request with server- id is still client message if no result/error
        let req = json!({"jsonrpc":"2.0","id":"server-9","method":"ping"});
        assert!(matches!(
            classify_line(&req.to_string()),
            StdioLineClass::ClientMessage(_)
        ));
    }
}
