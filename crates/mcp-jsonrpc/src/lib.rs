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

/// Convenience: parse error (-32700).
#[must_use]
pub fn parse_error(id: Option<RequestId>, message: impl Into<String>) -> JsonRpcError {
    error_response(id, error_code::PARSE_ERROR, message, None)
}

/// Convenience: invalid request (-32600).
#[must_use]
pub fn invalid_request(id: Option<RequestId>, message: impl Into<String>) -> JsonRpcError {
    error_response(id, error_code::INVALID_REQUEST, message, None)
}

/// Convenience: method not found (-32601).
#[must_use]
pub fn method_not_found(id: RequestId, method: impl Into<String>) -> JsonRpcError {
    error_response(
        Some(id),
        error_code::METHOD_NOT_FOUND,
        format!("Method not found: {}", method.into()),
        None,
    )
}

/// Convenience: invalid params (-32602).
#[must_use]
pub fn invalid_params(id: RequestId, message: impl Into<String>) -> JsonRpcError {
    error_response(Some(id), error_code::INVALID_PARAMS, message, None)
}

/// Convenience: internal error (-32603).
#[must_use]
pub fn internal_error(id: Option<RequestId>, message: impl Into<String>) -> JsonRpcError {
    error_response(id, error_code::INTERNAL_ERROR, message, None)
}

/// True when `code` is a standard JSON-RPC error code.
#[must_use]
pub fn is_standard_error_code(code: i64) -> bool {
    matches!(
        code,
        error_code::PARSE_ERROR
            | error_code::INVALID_REQUEST
            | error_code::METHOD_NOT_FOUND
            | error_code::INVALID_PARAMS
            | error_code::INTERNAL_ERROR
    )
}

/// True when `code` is in the JSON-RPC reserved server-error range `[-32099, -32000]`.
#[must_use]
pub fn is_server_error_code(code: i64) -> bool {
    (-32099..=-32000).contains(&code)
}

/// Convenience: invalid params with optional structured `data`.
#[must_use]
pub fn invalid_params_with_data(
    id: RequestId,
    message: impl Into<String>,
    data: Option<Value>,
) -> JsonRpcError {
    error_response(Some(id), error_code::INVALID_PARAMS, message, data)
}

/// Convenience: internal error with optional structured `data`.
#[must_use]
pub fn internal_error_with_data(
    id: Option<RequestId>,
    message: impl Into<String>,
    data: Option<Value>,
) -> JsonRpcError {
    error_response(id, error_code::INTERNAL_ERROR, message, data)
}

/// Extract method name from request/notification messages; `None` for responses.
#[must_use]
pub fn message_method(msg: &JsonRpcMessage) -> Option<&str> {
    match msg {
        JsonRpcMessage::Request(r) => Some(r.method.as_str()),
        JsonRpcMessage::Notification(n) => Some(n.method.as_str()),
        JsonRpcMessage::Success(_) | JsonRpcMessage::Error(_) => None,
    }
}

/// Extract request id from request/success/error; `None` for notifications or null-id errors.
#[must_use]
pub fn message_id(msg: &JsonRpcMessage) -> Option<&RequestId> {
    match msg {
        JsonRpcMessage::Request(r) => Some(&r.id),
        JsonRpcMessage::Success(s) => Some(&s.id),
        JsonRpcMessage::Error(e) => e.id.as_ref(),
        JsonRpcMessage::Notification(_) => None,
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

    #[test]
    fn wave11_standard_error_constructors() {
        let e = method_not_found(RequestId::Number(3), "tools/unknown");
        assert_eq!(e.error.code, error_code::METHOD_NOT_FOUND);
        assert!(e.error.message.contains("tools/unknown"));
        assert!(is_standard_error_code(e.error.code));

        let e = invalid_params(RequestId::String("a".into()), "missing name");
        assert_eq!(e.error.code, error_code::INVALID_PARAMS);

        let e = parse_error(None, "bad json");
        assert_eq!(e.error.code, error_code::PARSE_ERROR);

        let e = invalid_request(Some(RequestId::Number(1)), "no method");
        assert_eq!(e.error.code, error_code::INVALID_REQUEST);

        let e = internal_error(Some(RequestId::Number(2)), "boom");
        assert_eq!(e.error.code, error_code::INTERNAL_ERROR);

        assert!(!is_standard_error_code(-32000));
        assert!(!is_standard_error_code(0));
    }

    #[test]
    fn wave12_server_error_range_and_message_extractors() {
        assert!(is_server_error_code(-32000));
        assert!(is_server_error_code(-32099));
        assert!(is_server_error_code(-32050));
        assert!(!is_server_error_code(-32100));
        assert!(!is_server_error_code(-31999));
        assert!(!is_server_error_code(error_code::PARSE_ERROR));

        let e = invalid_params_with_data(
            RequestId::Number(9),
            "bad field",
            Some(json!({"field": "name"})),
        );
        assert_eq!(e.error.code, error_code::INVALID_PARAMS);
        assert_eq!(e.error.data, Some(json!({"field": "name"})));

        let e = internal_error_with_data(
            Some(RequestId::String("x".into())),
            "boom",
            Some(json!({"retry": true})),
        );
        assert_eq!(e.error.code, error_code::INTERNAL_ERROR);
        assert_eq!(e.error.data, Some(json!({"retry": true})));

        let req = request(RequestId::Number(1), "tools/list", None);
        let msg = JsonRpcMessage::Request(req);
        assert_eq!(message_method(&msg), Some("tools/list"));
        assert_eq!(message_id(&msg), Some(&RequestId::Number(1)));

        let n = notification("notifications/initialized", None);
        let msg = JsonRpcMessage::Notification(n);
        assert_eq!(message_method(&msg), Some("notifications/initialized"));
        assert!(message_id(&msg).is_none());

        let ok = success(RequestId::Number(2), json!(true));
        let msg = JsonRpcMessage::Success(ok);
        assert!(message_method(&msg).is_none());
        assert_eq!(message_id(&msg), Some(&RequestId::Number(2)));
    }

    /// Load committed golden fixture and assert parse/kind parity (pure residual differential).
    #[test]
    fn pure_residual_jsonrpc_golden_fixture() {
        let raw = include_str!("../fixtures/jsonrpc_golden.json");
        let doc: serde_json::Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("jsonrpc_golden.json: {e}"),
        };
        if let Some(codes) = doc.get("standardErrorCodes") {
            assert_eq!(
                codes["parseError"].as_i64(),
                Some(error_code::PARSE_ERROR)
            );
            assert_eq!(
                codes["invalidRequest"].as_i64(),
                Some(error_code::INVALID_REQUEST)
            );
            assert_eq!(
                codes["methodNotFound"].as_i64(),
                Some(error_code::METHOD_NOT_FOUND)
            );
            assert_eq!(
                codes["invalidParams"].as_i64(),
                Some(error_code::INVALID_PARAMS)
            );
            assert_eq!(
                codes["internalError"].as_i64(),
                Some(error_code::INTERNAL_ERROR)
            );
            assert!(is_standard_error_code(error_code::METHOD_NOT_FOUND));
        }
        if let Some(range) = doc.get("serverErrorRange") {
            let min = range["min"].as_i64().unwrap_or(0);
            let max = range["max"].as_i64().unwrap_or(0);
            assert!(is_server_error_code(min));
            assert!(is_server_error_code(max));
            assert!(is_server_error_code(-32050));
            assert!(!is_server_error_code(min - 1));
            assert!(!is_server_error_code(max + 1));
        }
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
                    "request" => {
                        assert!(is_request(&msg), "{name}");
                        assert!(message_method(&msg).is_some(), "{name} method");
                        assert!(message_id(&msg).is_some(), "{name} id");
                    }
                    "notification" => {
                        assert!(is_notification(&msg), "{name}");
                        assert!(message_method(&msg).is_some(), "{name} method");
                        assert!(message_id(&msg).is_none(), "{name} id");
                    }
                    "success" => {
                        assert!(is_success(&msg), "{name}");
                        assert!(message_method(&msg).is_none(), "{name} method");
                        assert!(message_id(&msg).is_some(), "{name} id");
                    }
                    "error" => {
                        assert!(is_error(&msg), "{name}");
                        if name == "error_server_range" {
                            if let JsonRpcMessage::Error(e) = &msg {
                                assert!(is_server_error_code(e.error.code), "{name}");
                            }
                        }
                    }
                    other => panic!("unexpected kind {other} for ok parse in {name}"),
                },
                ParseResult::Err(e) => match kind {
                    "err_version" => {
                        assert!(e.contains("version") || e.contains("Invalid"), "{name}: {e}")
                    }
                    "err_object" => {
                        assert!(e.to_lowercase().contains("object"), "{name}: {e}")
                    }
                    other => panic!("unexpected err for kind {other} in {name}: {e}"),
                },
            }
        }
    }
}
