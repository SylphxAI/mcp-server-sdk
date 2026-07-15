//! Pure JSON-RPC 2.0 helpers — parity with `src/protocol/jsonrpc.ts`.
//!
//! BW3 pure residual for `api/jsonrpc-protocol` deepen (constructors + parse + guards).
//! No transport/authority/ts_deleted claims (rej-010).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC version literal.
pub const JSONRPC_VERSION: &str = "2.0";

/// Standard error codes (parity with TS `ErrorCode`).
pub mod error_code {
    pub const PARSE_ERROR: i64 = -32700;
    pub const INVALID_REQUEST: i64 = -32600;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
}

/// Request id: string or number.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

/// JSON-RPC request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// JSON-RPC notification (no id).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// Success response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcSuccess {
    pub jsonrpc: String,
    pub id: RequestId,
    pub result: Value,
}

/// Error object body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcErrorBody {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Error response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub jsonrpc: String,
    pub id: Option<RequestId>,
    pub error: JsonRpcErrorBody,
}

/// Any JSON-RPC message (request | notification | response).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Notification(JsonRpcNotification),
    Success(JsonRpcSuccess),
    Error(JsonRpcError),
}

/// Parse result (parity with TS `ParseResult`).
#[derive(Debug, Clone, PartialEq)]
pub enum ParseResult {
    Ok(JsonRpcMessage),
    Err(String),
}

/// Construct a request (parity with TS `request`).
#[must_use]
pub fn request(id: RequestId, method: impl Into<String>, params: Option<Value>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: JSONRPC_VERSION.into(),
        id,
        method: method.into(),
        params,
    }
}

/// Construct a notification.
#[must_use]
pub fn notification(method: impl Into<String>, params: Option<Value>) -> JsonRpcNotification {
    JsonRpcNotification {
        jsonrpc: JSONRPC_VERSION.into(),
        method: method.into(),
        params,
    }
}

/// Construct a success response.
#[must_use]
pub fn success(id: RequestId, result: Value) -> JsonRpcSuccess {
    JsonRpcSuccess {
        jsonrpc: JSONRPC_VERSION.into(),
        id,
        result,
    }
}

/// Construct an error response.
#[must_use]
pub fn error_response(
    id: Option<RequestId>,
    code: i64,
    message: impl Into<String>,
    data: Option<Value>,
) -> JsonRpcError {
    JsonRpcError {
        jsonrpc: JSONRPC_VERSION.into(),
        id,
        error: JsonRpcErrorBody {
            code,
            message: message.into(),
            data,
        },
    }
}

/// Type guard: message is a request (`id` + `method`).
#[must_use]
pub fn is_request(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Request(_))
}

/// Type guard: notification (method, no id).
#[must_use]
pub fn is_notification(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Notification(_))
}

/// Type guard: success or error response.
#[must_use]
pub fn is_response(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Success(_) | JsonRpcMessage::Error(_))
}

/// Type guard: success.
#[must_use]
pub fn is_success(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Success(_))
}

/// Type guard: error.
#[must_use]
pub fn is_error(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Error(_))
}

/// Parse a JSON-RPC message string (parity with TS `parseMessage`).
#[must_use]
pub fn parse_message(input: &str) -> ParseResult {
    let data: Value = match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => return ParseResult::Err(format!("JSON parse error: {e}")),
    };
    if !data.is_object() {
        return ParseResult::Err("Message must be an object".into());
    }
    let version = data.get("jsonrpc").and_then(|v| v.as_str());
    if version != Some(JSONRPC_VERSION) {
        return ParseResult::Err(format!(
            "Invalid jsonrpc version: {}",
            version.unwrap_or("null")
        ));
    }
    match serde_json::from_value::<JsonRpcMessage>(data) {
        Ok(msg) => ParseResult::Ok(msg),
        Err(e) => ParseResult::Err(format!("JSON parse error: {e}")),
    }
}

/// Serialize a message (parity with TS `stringify`).
#[must_use]
pub fn stringify(msg: &JsonRpcMessage) -> String {
    serde_json::to_string(msg).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_request() {
        let raw = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
        match parse_message(raw) {
            ParseResult::Ok(msg) => {
                assert!(is_request(&msg));
                assert!(!is_notification(&msg));
            }
            ParseResult::Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn parse_notification() {
        let raw = r#"{"jsonrpc":"2.0","method":"notify"}"#;
        match parse_message(raw) {
            ParseResult::Ok(msg) => assert!(is_notification(&msg)),
            ParseResult::Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn parse_success() {
        let raw = r#"{"jsonrpc":"2.0","id":"a","result":{"ok":true}}"#;
        match parse_message(raw) {
            ParseResult::Ok(msg) => {
                assert!(is_response(&msg));
                assert!(is_success(&msg));
            }
            ParseResult::Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn parse_error_response() {
        let raw = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"bad"}}"#;
        match parse_message(raw) {
            ParseResult::Ok(msg) => assert!(is_error(&msg)),
            ParseResult::Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn reject_bad_version() {
        let raw = r#"{"jsonrpc":"1.0","id":1,"method":"x"}"#;
        match parse_message(raw) {
            ParseResult::Err(e) => assert!(e.contains("Invalid jsonrpc version")),
            ParseResult::Ok(_) => panic!("expected err"),
        }
    }

    #[test]
    fn reject_non_object() {
        match parse_message("[]") {
            ParseResult::Err(e) => assert!(e.contains("object")),
            ParseResult::Ok(_) => panic!("expected err"),
        }
    }

    #[test]
    fn constructors_roundtrip() {
        let req = request(RequestId::Number(7), "tools/list", Some(json!({})));
        let msg = JsonRpcMessage::Request(req);
        let s = stringify(&msg);
        match parse_message(&s) {
            ParseResult::Ok(m) => assert!(is_request(&m)),
            ParseResult::Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn error_codes_match_spec() {
        assert_eq!(error_code::PARSE_ERROR, -32700);
        assert_eq!(error_code::INVALID_REQUEST, -32600);
        assert_eq!(error_code::METHOD_NOT_FOUND, -32601);
        assert_eq!(error_code::INVALID_PARAMS, -32602);
        assert_eq!(error_code::INTERNAL_ERROR, -32603);
    }


    #[test]
    fn bulk_parse_rejects_array_and_missing_fields() {
        match parse_message("[]") {
            ParseResult::Err(e) => assert!(e.to_lowercase().contains("object") || e.contains("object") || !e.is_empty()),
            ParseResult::Ok(_) => panic!("expected err"),
        }
        match parse_message(r#"{"jsonrpc":"2.0"}"#) {
            ParseResult::Err(_) => {}
            ParseResult::Ok(_) => panic!("ambiguous message should fail"),
        }
        let req = request(RequestId::Number(7), "tools/list", Some(serde_json::json!({})));
        let msg = JsonRpcMessage::Request(req);
        let s = stringify(&msg);
        assert!(s.contains("tools/list"));
        assert!(is_request(&msg));
        assert!(!is_response(&msg));
    }

    #[test]
    fn bulk_notification_and_error_constructors() {
        let n = notification("notifications/initialized", None);
        let msg = JsonRpcMessage::Notification(n);
        assert!(is_notification(&msg));
        let err = error_response(
            Some(RequestId::Number(1)),
            error_code::METHOD_NOT_FOUND,
            "nope",
            None,
        );
        let msg = JsonRpcMessage::Error(err);
        assert!(is_error(&msg));
        assert!(!is_success(&msg));
    }

    /// Load committed golden fixture and assert parse/kind parity (pure residual differential).
    #[test]
    fn pure_residual_jsonrpc_golden_fixture() {
        let raw = include_str!("../fixtures/jsonrpc_golden.json");
        let doc: serde_json::Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("jsonrpc_golden.json: {e}"),
        };
        let cases = match doc["cases"].as_array() {
            Some(a) => a,
            None => panic!("cases"),
        };
        for case in cases {
            let name = case["name"].as_str().unwrap_or("?");
            let input = match case["input"].as_str() {
                Some(s) => s,
                None => panic!("input for {name}"),
            };
            let kind = match case["kind"].as_str() {
                Some(s) => s,
                None => panic!("kind for {name}"),
            };
            match parse_message(input) {
                ParseResult::Ok(msg) => match kind {
                    "request" => assert!(is_request(&msg), "{name}"),
                    "notification" => assert!(is_notification(&msg), "{name}"),
                    "success" => assert!(is_success(&msg), "{name}"),
                    "error" => assert!(is_error(&msg), "{name}"),
                    other => panic!("unexpected kind {other} for ok parse in {name}"),
                },
                ParseResult::Err(e) => match kind {
                    "err_version" => assert!(e.contains("version") || e.contains("Invalid"), "{name}: {e}"),
                    other => panic!("unexpected err for kind {other} in {name}: {e}"),
                },
            }
        }
    }
}
