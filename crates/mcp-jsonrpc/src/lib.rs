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
    /// Inclusive lower bound of the reserved server-error range.
    pub const SERVER_ERROR_MIN: i64 = -32099;
    /// Inclusive upper bound of the reserved server-error range.
    pub const SERVER_ERROR_MAX: i64 = -32000;
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
    (error_code::SERVER_ERROR_MIN..=error_code::SERVER_ERROR_MAX).contains(&code)
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

/// Extract params from request/notification messages; `None` for responses or absent params.
#[must_use]
pub fn message_params(msg: &JsonRpcMessage) -> Option<&Value> {
    match msg {
        JsonRpcMessage::Request(r) => r.params.as_ref(),
        JsonRpcMessage::Notification(n) => n.params.as_ref(),
        JsonRpcMessage::Success(_) | JsonRpcMessage::Error(_) => None,
    }
}

/// Standard error code → short name (parity with `ErrorCode` keys).
#[must_use]
pub fn standard_error_name(code: i64) -> Option<&'static str> {
    match code {
        error_code::PARSE_ERROR => Some("ParseError"),
        error_code::INVALID_REQUEST => Some("InvalidRequest"),
        error_code::METHOD_NOT_FOUND => Some("MethodNotFound"),
        error_code::INVALID_PARAMS => Some("InvalidParams"),
        error_code::INTERNAL_ERROR => Some("InternalError"),
        _ => None,
    }
}

/// ErrorCode name → numeric code.
#[must_use]
pub fn error_code_from_name(name: &str) -> Option<i64> {
    match name {
        "ParseError" | "parseError" => Some(error_code::PARSE_ERROR),
        "InvalidRequest" | "invalidRequest" => Some(error_code::INVALID_REQUEST),
        "MethodNotFound" | "methodNotFound" => Some(error_code::METHOD_NOT_FOUND),
        "InvalidParams" | "invalidParams" => Some(error_code::INVALID_PARAMS),
        "InternalError" | "internalError" => Some(error_code::INTERNAL_ERROR),
        _ => None,
    }
}

/// True when `code` is neither a standard JSON-RPC error nor the reserved server-error range.
///
/// Application-defined codes live outside both ranges (JSON-RPC 2.0 §5.1).
#[must_use]
pub fn is_application_error_code(code: i64) -> bool {
    !is_standard_error_code(code) && !is_server_error_code(code)
}

/// Convenience: parse error with optional structured `data`.
#[must_use]
pub fn parse_error_with_data(
    id: Option<RequestId>,
    message: impl Into<String>,
    data: Option<Value>,
) -> JsonRpcError {
    error_response(id, error_code::PARSE_ERROR, message, data)
}

/// Convenience: invalid request with optional structured `data`.
#[must_use]
pub fn invalid_request_with_data(
    id: Option<RequestId>,
    message: impl Into<String>,
    data: Option<Value>,
) -> JsonRpcError {
    error_response(id, error_code::INVALID_REQUEST, message, data)
}

/// Convenience: method not found with optional structured `data`.
#[must_use]
pub fn method_not_found_with_data(
    id: RequestId,
    method: impl Into<String>,
    data: Option<Value>,
) -> JsonRpcError {
    error_response(
        Some(id),
        error_code::METHOD_NOT_FOUND,
        format!("Method not found: {}", method.into()),
        data,
    )
}

/// Extract `result` from a success response; `None` otherwise.
#[must_use]
pub fn message_result(msg: &JsonRpcMessage) -> Option<&Value> {
    match msg {
        JsonRpcMessage::Success(s) => Some(&s.result),
        _ => None,
    }
}

/// Extract error body from an error response; `None` otherwise.
#[must_use]
pub fn message_error(msg: &JsonRpcMessage) -> Option<&JsonRpcErrorBody> {
    match msg {
        JsonRpcMessage::Error(e) => Some(&e.error),
        _ => None,
    }
}

/// Extract optional structured `data` from an error response.
#[must_use]
pub fn message_error_data(msg: &JsonRpcMessage) -> Option<&Value> {
    message_error(msg).and_then(|e| e.data.as_ref())
}

/// Convert a [`RequestId`] to a JSON value (number or string).
#[must_use]
pub fn request_id_to_value(id: &RequestId) -> Value {
    match id {
        RequestId::Number(n) => Value::Number((*n).into()),
        RequestId::String(s) => Value::String(s.clone()),
    }
}

/// Parse a JSON value as a [`RequestId`] (number or string only).
#[must_use]
pub fn request_id_from_value(v: &Value) -> Option<RequestId> {
    if let Some(n) = v.as_i64() {
        return Some(RequestId::Number(n));
    }
    if let Some(s) = v.as_str() {
        return Some(RequestId::String(s.to_string()));
    }
    None
}

// ============================================================================
// WAVE15 pure residual classifiers + extractors
// ============================================================================

/// Classify an error code: `standard`, `server`, or `application` (JSON-RPC 2.0 §5.1).
#[must_use]
pub fn error_code_kind(code: i64) -> &'static str {
    if is_standard_error_code(code) {
        "standard"
    } else if is_server_error_code(code) {
        "server"
    } else {
        "application"
    }
}

/// Classify a parsed message: `request`, `notification`, `success`, or `error`.
#[must_use]
pub fn message_kind(msg: &JsonRpcMessage) -> &'static str {
    match msg {
        JsonRpcMessage::Request(_) => "request",
        JsonRpcMessage::Notification(_) => "notification",
        JsonRpcMessage::Success(_) => "success",
        JsonRpcMessage::Error(_) => "error",
    }
}

/// Extract numeric error code from an error response; `None` otherwise.
#[must_use]
pub fn message_error_code(msg: &JsonRpcMessage) -> Option<i64> {
    message_error(msg).map(|e| e.code)
}

/// Extract error message string from an error response; `None` otherwise.
#[must_use]
pub fn message_error_message(msg: &JsonRpcMessage) -> Option<&str> {
    message_error(msg).map(|e| e.message.as_str())
}

/// True when a request/notification carries a `params` field.
#[must_use]
pub fn has_params(msg: &JsonRpcMessage) -> bool {
    message_params(msg).is_some()
}

/// Request id discriminant: `number`, `string`, or `none` (notification / null-id error).
#[must_use]
pub fn request_id_kind(id: Option<&RequestId>) -> &'static str {
    match id {
        Some(RequestId::Number(_)) => "number",
        Some(RequestId::String(_)) => "string",
        None => "none",
    }
}

// ============================================================================
// WAVE16 pure residual constructors + extractors
// ============================================================================

/// Construct a JSON-RPC error body (parity with wire `error` object).
#[must_use]
pub fn error_body(
    code: i64,
    message: impl Into<String>,
    data: Option<Value>,
) -> JsonRpcErrorBody {
    JsonRpcErrorBody {
        code,
        message: message.into(),
        data,
    }
}

/// True when `version` is the JSON-RPC 2.0 version literal.
#[must_use]
pub fn is_valid_jsonrpc_version(version: &str) -> bool {
    version == JSONRPC_VERSION
}

/// True when a JSON value is a valid request id (string or number only).
#[must_use]
pub fn is_valid_request_id_value(v: &Value) -> bool {
    request_id_from_value(v).is_some()
}

/// True when the message is a success response carrying a `result`.
#[must_use]
pub fn message_has_result(msg: &JsonRpcMessage) -> bool {
    message_result(msg).is_some()
}

/// True when the message is an error response carrying an `error` body.
#[must_use]
pub fn message_has_error(msg: &JsonRpcMessage) -> bool {
    message_error(msg).is_some()
}

/// Convenience: server-error-range response (`code` must be in `[-32099, -32000]`).
///
/// When `code` is outside the reserved server-error range, falls back to
/// [`error_code::INTERNAL_ERROR`] so callers never emit a misclassified wire code
/// from this pure residual helper.
#[must_use]
pub fn server_error_response(
    id: Option<RequestId>,
    code: i64,
    message: impl Into<String>,
    data: Option<Value>,
) -> JsonRpcError {
    let code = if is_server_error_code(code) {
        code
    } else {
        error_code::INTERNAL_ERROR
    };
    error_response(id, code, message, data)
}

/// Parse a JSON-RPC message from a pre-decoded [`Value`] (object only).
///
/// Parity with the object branch of TS `parseMessage` after JSON.parse.
#[must_use]
pub fn parse_message_value(data: Value) -> ParseResult {
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

/// True when `input` is a JSON array (JSON-RPC batch shape). Pure residual detector only —
/// batch dispatch remains product/transport authority (TS).
#[must_use]
pub fn is_batch_payload(input: &str) -> bool {
    let trimmed = input.trim_start();
    if !trimmed.starts_with('[') {
        return false;
    }
    matches!(serde_json::from_str::<Value>(input), Ok(Value::Array(_)))
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

// --- WAVE17 pure residual ---

/// Pretty-print a message (indent 2) for diagnostics; never panics.
#[must_use]
pub fn stringify_pretty(msg: &JsonRpcMessage) -> String {
    serde_json::to_string_pretty(msg).unwrap_or_else(|_| "{}".into())
}

/// Whether a parse result is Ok.
#[must_use]
pub fn parse_result_is_ok(result: &ParseResult) -> bool {
    matches!(result, ParseResult::Ok(_))
}

/// Extract error string from a failed parse.
#[must_use]
pub fn parse_result_error(result: &ParseResult) -> Option<&str> {
    match result {
        ParseResult::Err(e) => Some(e.as_str()),
        ParseResult::Ok(_) => None,
    }
}

/// Build a minimal successful response with a null result.
#[must_use]
pub fn empty_success_response(id: RequestId) -> JsonRpcMessage {
    JsonRpcMessage::Success(success(id, Value::Null))
}

/// Whether a response id matches a request id (wire equality).
#[must_use]
pub fn response_matches_request_id(response: &JsonRpcMessage, request_id: &RequestId) -> bool {
    match message_id(response) {
        Some(id) => id == request_id,
        None => false,
    }
}

/// Normalize method name (trim); empty → None.
#[must_use]
pub fn normalize_method_name(method: &str) -> Option<&str> {
    let t = method.trim();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

// --- WAVE18 pure residual ---

/// Extract string id from RequestId when it is a string; None for numbers.
#[must_use]
pub fn request_id_as_str(id: &RequestId) -> Option<&str> {
    match id {
        RequestId::String(s) => Some(s.as_str()),
        RequestId::Number(_) => None,
    }
}

/// Extract number id from RequestId when it is a number; None for strings.
#[must_use]
pub fn request_id_as_i64(id: &RequestId) -> Option<i64> {
    match id {
        RequestId::Number(n) => Some(*n),
        RequestId::String(_) => None,
    }
}

/// Whether a message is a request with method `ping`.
#[must_use]
pub fn is_ping_request(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Request(r) if r.method == "ping")
}

/// True when message is success with null result.
#[must_use]
pub fn is_null_result_success(msg: &JsonRpcMessage) -> bool {
    match msg {
        JsonRpcMessage::Success(s) => s.result.is_null(),
        _ => false,
    }
}

/// True when message is an error response (convenience dual-oracle).
#[must_use]
pub fn is_error_response(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Error(_))
}

/// True when a RequestId is numeric.
#[must_use]
pub fn request_id_is_number(id: &RequestId) -> bool {
    matches!(id, RequestId::Number(_))
}

/// True when a RequestId is a string.
#[must_use]
pub fn request_id_is_string(id: &RequestId) -> bool {
    matches!(id, RequestId::String(_))
}


// --- WAVE19 pure residual ---

/// Parse a JSON-RPC **batch** payload (JSON array of messages).
/// Returns Ok(messages) only when every element is a valid JSON-RPC 2.0 message object.
/// Empty arrays are rejected (JSON-RPC 2.0: batch must be non-empty).
pub fn parse_batch_messages(input: &str) -> Result<Vec<JsonRpcMessage>, String> {
    let trimmed = input.trim();
    if !trimmed.starts_with('[') {
        return Err("batch payload must be a JSON array".into());
    }
    let value: serde_json::Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("JSON parse error: {e}"))?;
    let arr = value
        .as_array()
        .ok_or_else(|| "batch payload must be a JSON array".to_string())?;
    if arr.is_empty() {
        return Err("batch must be a non-empty array".into());
    }
    let mut out = Vec::with_capacity(arr.len());
    for (i, el) in arr.iter().enumerate() {
        match parse_message_value(el.clone()) {
            ParseResult::Ok(msg) => out.push(msg),
            ParseResult::Err(e) => return Err(format!("batch[{i}]: {e}")),
        }
    }
    Ok(out)
}

/// Normalized method name from a request/notification message (trimmed); None otherwise.
#[must_use]
pub fn message_method_normalized(msg: &JsonRpcMessage) -> Option<&str> {
    message_method(msg).and_then(normalize_method_name)
}

/// True when message is a request that carries a `params` field (including null/empty object).
#[must_use]
pub fn request_has_params(msg: &JsonRpcMessage) -> bool {
    matches!(msg, JsonRpcMessage::Request(r) if r.params.is_some())
}

/// Display form of RequestId for logging (numbers as decimal, strings as-is).
#[must_use]
pub fn request_id_display(id: &RequestId) -> String {
    match id {
        RequestId::Number(n) => n.to_string(),
        RequestId::String(s) => s.clone(),
    }
}

/// True when two request ids are equal (wire equality).
#[must_use]
pub fn request_ids_equal(a: &RequestId, b: &RequestId) -> bool {
    a == b
}


// --- WAVE20 pure residual ---

/// Number of messages in a successfully parsed batch (convenience dual-oracle).
#[must_use]
pub fn batch_message_count(msgs: &[JsonRpcMessage]) -> usize {
    msgs.len()
}

/// First request id in a batch of messages (skips notifications/responses).
#[must_use]
pub fn first_request_id_in_batch(msgs: &[JsonRpcMessage]) -> Option<RequestId> {
    for m in msgs {
        if let JsonRpcMessage::Request(r) = m {
            return Some(r.id.clone());
        }
    }
    None
}

/// True when any message in the batch is a notification.
#[must_use]
pub fn batch_has_notification(msgs: &[JsonRpcMessage]) -> bool {
    msgs.iter().any(is_notification)
}

/// True when any message in the batch is a request (expects response).
#[must_use]
pub fn batch_has_request(msgs: &[JsonRpcMessage]) -> bool {
    msgs.iter().any(is_request)
}

/// Extract error code from an error response message.
#[must_use]
pub fn error_code_of_message(msg: &JsonRpcMessage) -> Option<i64> {
    match msg {
        JsonRpcMessage::Error(e) => Some(e.error.code),
        _ => None,
    }
}

/// Extract error message string from an error response message.
#[must_use]
pub fn error_message_of_message(msg: &JsonRpcMessage) -> Option<&str> {
    match msg {
        JsonRpcMessage::Error(e) => Some(e.error.message.as_str()),
        _ => None,
    }
}

// --- WAVE21 pure residual (handler dual-oracle jsonrpc helpers) ---

/// True when message is a success response whose id equals `id`.
#[must_use]
pub fn is_success_for_id(msg: &JsonRpcMessage, id: &RequestId) -> bool {
    match msg {
        JsonRpcMessage::Success(s) => &s.id == id,
        _ => false,
    }
}

/// True when message is an error response whose id equals `id` (null id never matches).
#[must_use]
pub fn is_error_for_id(msg: &JsonRpcMessage, id: &RequestId) -> bool {
    match msg {
        JsonRpcMessage::Error(e) => e.id.as_ref() == Some(id),
        _ => false,
    }
}

/// Error code from an error response message, if any (alias of `error_code_of_message`).
#[must_use]
pub fn response_error_code(msg: &JsonRpcMessage) -> Option<i64> {
    error_code_of_message(msg)
}

/// Error message text from an error response, if any (alias of `error_message_of_message`).
#[must_use]
pub fn response_error_message(msg: &JsonRpcMessage) -> Option<&str> {
    error_message_of_message(msg)
}

/// True when request/notification params is missing or is an empty object.
#[must_use]
pub fn params_absent_or_empty(msg: &JsonRpcMessage) -> bool {
    match msg {
        JsonRpcMessage::Request(r) => match &r.params {
            None => true,
            Some(Value::Object(m)) => m.is_empty(),
            Some(Value::Null) => true,
            _ => false,
        },
        JsonRpcMessage::Notification(n) => match &n.params {
            None => true,
            Some(Value::Object(m)) => m.is_empty(),
            Some(Value::Null) => true,
            _ => false,
        },
        _ => false,
    }
}

/// Serialize a batch of messages as a JSON array (order preserved).
#[must_use]
pub fn stringify_batch(messages: &[JsonRpcMessage]) -> String {
    let values: Vec<Value> = messages
        .iter()
        .map(|m| serde_json::to_value(m).unwrap_or(Value::Null))
        .collect();
    serde_json::to_string(&values).unwrap_or_else(|_| "[]".into())
}

/// Number of messages in a successfully parsed batch payload; None if input is not a batch array.
#[must_use]
pub fn batch_payload_message_count(input: &str) -> Option<usize> {
    parse_batch_messages(input).ok().map(|v| v.len())
}

/// True when parse result is a request with the given method (exact match).
#[must_use]
pub fn is_request_method(msg: &JsonRpcMessage, method: &str) -> bool {
    matches!(msg, JsonRpcMessage::Request(r) if r.method == method)
}

/// True when parse result is a notification with the given method (exact match).
#[must_use]
pub fn is_notification_method(msg: &JsonRpcMessage, method: &str) -> bool {
    matches!(msg, JsonRpcMessage::Notification(n) if n.method == method)
}

/// Build method-not-found error response message for `id`.
#[must_use]
pub fn method_not_found_response(id: RequestId, method: &str) -> JsonRpcMessage {
    JsonRpcMessage::Error(method_not_found(id, method))
}


// --- WAVE22 pure residual ---

/// Standard invalid-params error response dual-oracle (message enum form).
#[must_use]
pub fn invalid_params_response(id: RequestId, details: impl Into<String>) -> JsonRpcMessage {
    JsonRpcMessage::Error(invalid_params(id, details))
}

/// Standard internal-error response dual-oracle (message enum form).
#[must_use]
pub fn internal_error_response(id: RequestId, details: impl Into<String>) -> JsonRpcMessage {
    JsonRpcMessage::Error(internal_error(Some(id), details))
}

/// True when every message in a batch is a response (success or error).
#[must_use]
pub fn batch_all_responses(msgs: &[JsonRpcMessage]) -> bool {
    !msgs.is_empty() && msgs.iter().all(is_response)
}

/// First error message in a batch, if any.
#[must_use]
pub fn first_error_in_batch(msgs: &[JsonRpcMessage]) -> Option<&JsonRpcMessage> {
    msgs.iter().find(|m| is_error_response(m))
}

/// Count of request messages in a batch.
#[must_use]
pub fn batch_request_count(msgs: &[JsonRpcMessage]) -> usize {
    msgs.iter().filter(|m| is_request(m)).count()
}

/// Count of notification messages in a batch.
#[must_use]
pub fn batch_notification_count(msgs: &[JsonRpcMessage]) -> usize {
    msgs.iter().filter(|m| is_notification(m)).count()
}

/// True when a response carries a numeric error code in the JSON-RPC reserved server range.
#[must_use]
pub fn response_is_server_error(msg: &JsonRpcMessage) -> bool {
    response_error_code(msg).is_some_and(is_server_error_code)
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
            assert_eq!(min, error_code::SERVER_ERROR_MIN);
            assert_eq!(max, error_code::SERVER_ERROR_MAX);
            assert!(is_server_error_code(min));
            assert!(is_server_error_code(max));
            assert!(is_server_error_code(-32050));
            assert!(!is_server_error_code(min - 1));
            assert!(!is_server_error_code(max + 1));
        }
        if let Some(cases) = doc.get("messageResultExtract").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let input = case["input"].as_str().unwrap_or("");
                match parse_message(input) {
                    ParseResult::Ok(msg) => {
                        if case.get("hasResult").and_then(|v| v.as_bool()).unwrap_or(false) {
                            assert!(message_result(&msg).is_some(), "{name}");
                        } else {
                            assert!(message_result(&msg).is_none(), "{name}");
                        }
                        if case.get("hasError").and_then(|v| v.as_bool()).unwrap_or(false) {
                            assert!(message_error(&msg).is_some(), "{name}");
                        } else {
                            assert!(message_error(&msg).is_none(), "{name}");
                        }
                    }
                    ParseResult::Err(e) => panic!("{name}: {e}"),
                }
            }
        }
        if let Some(cases) = doc.get("requestIdRoundtrip").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let v = case.get("value").cloned().unwrap_or(Value::Null);
                let id = match request_id_from_value(&v) {
                    Some(id) => id,
                    None => panic!("expected id for {name}"),
                };
                assert_eq!(request_id_to_value(&id), v, "{name}");
            }
        }
        if let Some(cases) = doc.get("errorCodeKinds").and_then(|v| v.as_array()) {
            for case in cases {
                let code = case["code"].as_i64().unwrap_or(0);
                let kind = case["kind"].as_str().unwrap_or("");
                assert_eq!(error_code_kind(code), kind, "code {code}");
            }
        }
        if let Some(cases) = doc.get("messageKinds").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let input = case["input"].as_str().unwrap_or("");
                let expected = case["kind"].as_str().unwrap_or("");
                match parse_message(input) {
                    ParseResult::Ok(msg) => {
                        assert_eq!(message_kind(&msg), expected, "{name}");
                        if let Some(has) = case.get("hasParams").and_then(|v| v.as_bool()) {
                            assert_eq!(has_params(&msg), has, "{name} hasParams");
                        }
                        if let Some(code) = case.get("errorCode").and_then(|v| v.as_i64()) {
                            assert_eq!(message_error_code(&msg), Some(code), "{name} code");
                        }
                        if let Some(msg_text) = case.get("errorMessage").and_then(|v| v.as_str()) {
                            assert_eq!(
                                message_error_message(&msg),
                                Some(msg_text),
                                "{name} message"
                            );
                        }
                    }
                    ParseResult::Err(e) => panic!("{name}: {e}"),
                }
            }
        }
        if let Some(cases) = doc.get("requestIdKinds").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let kind = case["kind"].as_str().unwrap_or("");
                if case.get("value").is_some() && !case["value"].is_null() {
                    let v = case.get("value").cloned().unwrap_or(Value::Null);
                    let id = match request_id_from_value(&v) {
                        Some(id) => id,
                        None => panic!("expected id for {name}"),
                    };
                    assert_eq!(request_id_kind(Some(&id)), kind, "{name}");
                } else {
                    assert_eq!(request_id_kind(None), kind, "{name}");
                }
            }
        }
        if let Some(names) = doc.get("standardErrorNames") {
            assert!(names.is_object(), "standardErrorNames object");
            for (key, code_key) in [
                ("ParseError", "parseError"),
                ("InvalidRequest", "invalidRequest"),
                ("MethodNotFound", "methodNotFound"),
                ("InvalidParams", "invalidParams"),
                ("InternalError", "internalError"),
            ] {
                let code = doc["standardErrorCodes"][code_key].as_i64().unwrap_or(0);
                assert_eq!(standard_error_name(code), Some(key));
                assert_eq!(error_code_from_name(key), Some(code));
                assert_eq!(names[key].as_i64(), Some(code), "name map {key}");
            }
        }
        if let Some(cases) = doc.get("applicationErrorCodes").and_then(|v| v.as_array()) {
            for case in cases {
                let code = case["code"].as_i64().unwrap_or(0);
                let is_app = case["isApplication"].as_bool().unwrap_or(false);
                assert_eq!(is_application_error_code(code), is_app, "code {code}");
            }
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
        // WAVE16: version, request-id validity, batch detector, flags, server-error helper
        if let Some(v) = doc.get("jsonrpcVersion").and_then(|v| v.as_str()) {
            assert!(is_valid_jsonrpc_version(v));
            assert_eq!(v, JSONRPC_VERSION);
        }
        if let Some(cases) = doc.get("validRequestIdValues").and_then(|v| v.as_array()) {
            for case in cases {
                let val = case.get("value").cloned().unwrap_or(Value::Null);
                let valid = case["valid"].as_bool().unwrap_or(false);
                assert_eq!(
                    is_valid_request_id_value(&val),
                    valid,
                    "request id value {val}"
                );
            }
        }
        if let Some(cases) = doc.get("batchPayloads").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let input = case["input"].as_str().unwrap_or("");
                let expected = case["isBatch"].as_bool().unwrap_or(false);
                assert_eq!(is_batch_payload(input), expected, "batch {name}");
            }
        }
        if let Some(cases) = doc.get("serverErrorResponses").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let code = case["code"].as_i64().unwrap_or(0);
                let expected = case["expectedCode"].as_i64().unwrap_or(0);
                let e = server_error_response(Some(RequestId::Number(1)), code, "x", None);
                assert_eq!(e.error.code, expected, "server err {name}");
            }
        }
        if let Some(cases) = doc.get("messageHasFlags").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let input = case["input"].as_str().unwrap_or("");
                match parse_message(input) {
                    ParseResult::Ok(msg) => {
                        assert_eq!(
                            message_has_result(&msg),
                            case["hasResult"].as_bool().unwrap_or(false),
                            "hasResult {name}"
                        );
                        assert_eq!(
                            message_has_error(&msg),
                            case["hasError"].as_bool().unwrap_or(false),
                            "hasError {name}"
                        );
                        // parse_message_value round-trip for object inputs
                        if let Ok(val) = serde_json::from_str::<Value>(input) {
                            match parse_message_value(val) {
                                ParseResult::Ok(m2) => {
                                    assert_eq!(message_kind(&m2), message_kind(&msg), "{name}");
                                }
                                ParseResult::Err(e) => panic!("{name} value parse: {e}"),
                            }
                        }
                    }
                    ParseResult::Err(e) => panic!("{name}: {e}"),
                }
            }
        }
        // error_body constructor smoke via golden internalError code
        if let Some(codes) = doc.get("standardErrorCodes") {
            let code = codes["internalError"].as_i64().unwrap_or(0);
            let body = error_body(code, "boom", None);
            assert_eq!(body.code, code);
            assert_eq!(body.message, "boom");
        }
    }

    #[test]
    fn wave13_error_names_application_range_and_params() {
        assert_eq!(standard_error_name(error_code::PARSE_ERROR), Some("ParseError"));
        assert_eq!(standard_error_name(error_code::METHOD_NOT_FOUND), Some("MethodNotFound"));
        assert_eq!(standard_error_name(-32000), None);
        assert_eq!(error_code_from_name("InvalidParams"), Some(error_code::INVALID_PARAMS));
        assert_eq!(error_code_from_name("invalidParams"), Some(error_code::INVALID_PARAMS));
        assert_eq!(error_code_from_name("nope"), None);

        assert!(is_application_error_code(-1));
        assert!(is_application_error_code(42));
        assert!(!is_application_error_code(error_code::INTERNAL_ERROR));
        assert!(!is_application_error_code(-32050));

        let e = parse_error_with_data(None, "bad", Some(json!({"offset": 3})));
        assert_eq!(e.error.code, error_code::PARSE_ERROR);
        assert_eq!(e.error.data, Some(json!({"offset": 3})));

        let e = invalid_request_with_data(Some(RequestId::Number(1)), "no method", None);
        assert_eq!(e.error.code, error_code::INVALID_REQUEST);

        let e = method_not_found_with_data(
            RequestId::String("a".into()),
            "tools/unknown",
            Some(json!({"hint": "tools/list"})),
        );
        assert_eq!(e.error.code, error_code::METHOD_NOT_FOUND);
        assert!(e.error.message.contains("tools/unknown"));
        assert_eq!(e.error.data, Some(json!({"hint": "tools/list"})));

        let req = request(RequestId::Number(1), "tools/call", Some(json!({"name": "ping"})));
        let msg = JsonRpcMessage::Request(req);
        assert_eq!(message_params(&msg), Some(&json!({"name": "ping"})));
        let n = notification("notifications/initialized", None);
        let msg = JsonRpcMessage::Notification(n);
        assert!(message_params(&msg).is_none());
        let ok = success(RequestId::Number(2), json!(true));
        let msg = JsonRpcMessage::Success(ok);
        assert!(message_params(&msg).is_none());
    }

    #[test]
    fn wave14_result_error_extractors_and_request_id() {
        assert_eq!(error_code::SERVER_ERROR_MIN, -32099);
        assert_eq!(error_code::SERVER_ERROR_MAX, -32000);
        assert!(is_server_error_code(error_code::SERVER_ERROR_MIN));
        assert!(is_server_error_code(error_code::SERVER_ERROR_MAX));

        let ok = success(RequestId::Number(2), json!({"tools": []}));
        let msg = JsonRpcMessage::Success(ok);
        assert_eq!(message_result(&msg), Some(&json!({"tools": []})));
        assert!(message_error(&msg).is_none());
        assert!(message_error_data(&msg).is_none());

        let err = invalid_params_with_data(
            RequestId::String("a".into()),
            "bad",
            Some(json!({"field": "name"})),
        );
        let msg = JsonRpcMessage::Error(err);
        assert!(message_result(&msg).is_none());
        let body = match message_error(&msg) {
            Some(b) => b,
            None => panic!("expected error body"),
        };
        assert_eq!(body.code, error_code::INVALID_PARAMS);
        assert_eq!(message_error_data(&msg), Some(&json!({"field": "name"})));

        let req = request(RequestId::Number(1), "ping", None);
        let msg = JsonRpcMessage::Request(req);
        assert!(message_result(&msg).is_none());
        assert!(message_error(&msg).is_none());

        assert_eq!(
            request_id_to_value(&RequestId::Number(7)),
            json!(7)
        );
        assert_eq!(
            request_id_to_value(&RequestId::String("x".into())),
            json!("x")
        );
        assert_eq!(
            request_id_from_value(&json!(9)),
            Some(RequestId::Number(9))
        );
        assert_eq!(
            request_id_from_value(&json!("id-1")),
            Some(RequestId::String("id-1".into()))
        );
        assert!(request_id_from_value(&json!(true)).is_none());
        assert!(request_id_from_value(&json!(null)).is_none());
    }

    #[test]
    fn wave15_kinds_error_extractors_and_params() {
        assert_eq!(error_code_kind(error_code::PARSE_ERROR), "standard");
        assert_eq!(error_code_kind(error_code::INTERNAL_ERROR), "standard");
        assert_eq!(error_code_kind(-32050), "server");
        assert_eq!(error_code_kind(error_code::SERVER_ERROR_MIN), "server");
        assert_eq!(error_code_kind(-1), "application");
        assert_eq!(error_code_kind(42), "application");

        let req = request(
            RequestId::Number(1),
            "tools/call",
            Some(json!({"name": "ping"})),
        );
        let msg = JsonRpcMessage::Request(req);
        assert_eq!(message_kind(&msg), "request");
        assert!(has_params(&msg));
        assert_eq!(request_id_kind(message_id(&msg)), "number");

        let n = notification("notifications/initialized", None);
        let msg = JsonRpcMessage::Notification(n);
        assert_eq!(message_kind(&msg), "notification");
        assert!(!has_params(&msg));
        assert_eq!(request_id_kind(message_id(&msg)), "none");

        let ok = success(RequestId::String("a".into()), json!(true));
        let msg = JsonRpcMessage::Success(ok);
        assert_eq!(message_kind(&msg), "success");
        assert_eq!(request_id_kind(message_id(&msg)), "string");
        assert!(message_error_code(&msg).is_none());
        assert!(message_error_message(&msg).is_none());

        let err = invalid_params(RequestId::Number(3), "bad field");
        let msg = JsonRpcMessage::Error(err);
        assert_eq!(message_kind(&msg), "error");
        assert_eq!(
            message_error_code(&msg),
            Some(error_code::INVALID_PARAMS)
        );
        assert_eq!(message_error_message(&msg), Some("bad field"));
    }

    #[test]
    fn wave16_error_body_parse_value_batch_and_server_error() {
        assert!(is_valid_jsonrpc_version(JSONRPC_VERSION));
        assert!(!is_valid_jsonrpc_version("1.0"));
        assert!(!is_valid_jsonrpc_version(""));

        let body = error_body(error_code::INVALID_PARAMS, "bad", Some(json!({"f": 1})));
        assert_eq!(body.code, error_code::INVALID_PARAMS);
        assert_eq!(body.message, "bad");
        assert_eq!(body.data, Some(json!({"f": 1})));

        assert!(is_valid_request_id_value(&json!(1)));
        assert!(is_valid_request_id_value(&json!("a")));
        assert!(!is_valid_request_id_value(&json!(true)));
        assert!(!is_valid_request_id_value(&json!(null)));

        let ok = success(RequestId::Number(1), json!({"ok": true}));
        let msg = JsonRpcMessage::Success(ok);
        assert!(message_has_result(&msg));
        assert!(!message_has_error(&msg));

        let err = invalid_params(RequestId::Number(2), "x");
        let msg = JsonRpcMessage::Error(err);
        assert!(!message_has_result(&msg));
        assert!(message_has_error(&msg));

        let e = server_error_response(
            Some(RequestId::Number(9)),
            -32050,
            "busy",
            Some(json!({"retry": true})),
        );
        assert_eq!(e.error.code, -32050);
        assert!(is_server_error_code(e.error.code));
        assert_eq!(e.error.data, Some(json!({"retry": true})));

        // Out-of-range code falls back to internal error
        let e = server_error_response(None, -1, "nope", None);
        assert_eq!(e.error.code, error_code::INTERNAL_ERROR);

        let val = json!({"jsonrpc":"2.0","id":1,"method":"ping"});
        match parse_message_value(val) {
            ParseResult::Ok(msg) => {
                assert!(is_request(&msg));
                assert_eq!(message_method(&msg), Some("ping"));
            }
            ParseResult::Err(e) => panic!("{e}"),
        }
        match parse_message_value(json!([])) {
            ParseResult::Err(e) => assert!(e.to_lowercase().contains("object")),
            ParseResult::Ok(_) => panic!("expected err"),
        }
        match parse_message_value(json!({"jsonrpc":"1.0","id":1,"method":"x"})) {
            ParseResult::Err(e) => assert!(e.contains("version")),
            ParseResult::Ok(_) => panic!("expected err"),
        }

        assert!(is_batch_payload(r#"[{"jsonrpc":"2.0","id":1,"method":"ping"}]"#));
        assert!(is_batch_payload("  [1,2,3]"));
        assert!(!is_batch_payload(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#));
        assert!(!is_batch_payload("not-json"));
        assert!(!is_batch_payload(""));
    }

    #[test]
    fn wave17_stringify_pretty_parse_helpers_and_empty_success() {
        let req = JsonRpcMessage::Request(request(RequestId::Number(7), "ping", None));
        let pretty = stringify_pretty(&req);
        assert!(pretty.contains("ping"));
        assert!(pretty.contains('\n'));

        let ok = parse_message(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#);
        assert!(parse_result_is_ok(&ok));
        assert!(parse_result_error(&ok).is_none());
        let bad = parse_message("not-json");
        assert!(!parse_result_is_ok(&bad));
        assert!(parse_result_error(&bad).is_some());

        let id = RequestId::Number(9);
        let resp = empty_success_response(id.clone());
        assert!(is_success(&resp));
        assert!(response_matches_request_id(&resp, &id));
        assert!(!response_matches_request_id(&resp, &RequestId::Number(1)));
        assert_eq!(normalize_method_name("  tools/list  "), Some("tools/list"));
        assert!(normalize_method_name("   ").is_none());
        assert!(normalize_method_name("").is_none());
    }

    #[test]
    fn wave18_request_id_ping_and_error_extractors() {
        let n = RequestId::Number(42);
        let s = RequestId::String("abc".into());
        assert_eq!(request_id_as_i64(&n), Some(42));
        assert!(request_id_as_str(&n).is_none());
        assert_eq!(request_id_as_str(&s), Some("abc"));
        assert!(request_id_as_i64(&s).is_none());
        assert!(request_id_is_number(&n));
        assert!(request_id_is_string(&s));
        assert!(!request_id_is_number(&s));

        let ping = JsonRpcMessage::Request(request(RequestId::Number(1), "ping", None));
        assert!(is_ping_request(&ping));
        let tools = JsonRpcMessage::Request(request(RequestId::Number(1), "tools/list", None));
        assert!(!is_ping_request(&tools));

        let err = invalid_params(RequestId::Number(2), "bad");
        let msg = JsonRpcMessage::Error(err);
        assert!(is_error_response(&msg));
        assert!(!is_error_response(&ping));
        assert_eq!(message_error_message(&msg), Some("bad"));

        let empty = empty_success_response(RequestId::Number(9));
        assert!(is_null_result_success(&empty));
        let with = JsonRpcMessage::Success(success(RequestId::Number(1), json!({"ok": true})));
        assert!(!is_null_result_success(&with));
    }

    #[test]
    fn wave19_batch_parse_and_message_helpers() {
        let batch = r#"[
          {"jsonrpc":"2.0","id":1,"method":"ping"},
          {"jsonrpc":"2.0","method":"notifications/initialized"},
          {"jsonrpc":"2.0","id":2,"result":null}
        ]"#;
        let msgs = match parse_batch_messages(batch) {
            Ok(m) => m,
            Err(e) => panic!("batch: {e}"),
        };
        assert_eq!(msgs.len(), 3);
        assert!(is_request(&msgs[0]));
        assert!(is_notification(&msgs[1]));
        assert!(is_null_result_success(&msgs[2]));
        assert_eq!(message_method_normalized(&msgs[0]), Some("ping"));

        assert!(parse_batch_messages("[]").is_err());
        assert!(parse_batch_messages(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).is_err());
        assert!(parse_batch_messages(r#"[{"jsonrpc":"1.0","id":1,"method":"x"}]"#).is_err());

        let req = JsonRpcMessage::Request(request(
            RequestId::Number(7),
            "  tools/list  ",
            Some(json!({"cursor": "a"})),
        ));
        // method stored as provided; normalize extracts trim
        assert_eq!(message_method_normalized(&req), Some("tools/list"));
        assert!(request_has_params(&req));
        let bare = JsonRpcMessage::Request(request(RequestId::Number(1), "ping", None));
        assert!(!request_has_params(&bare));

        let err = JsonRpcMessage::Error(invalid_params(RequestId::Number(3), "bad"));
        assert_eq!(message_error_code(&err), Some(-32602));
        assert!(message_error_code(&bare).is_none());

        assert_eq!(request_id_display(&RequestId::Number(42)), "42");
        assert_eq!(request_id_display(&RequestId::String("abc".into())), "abc");
        assert!(request_ids_equal(&RequestId::Number(1), &RequestId::Number(1)));
        assert!(!request_ids_equal(
            &RequestId::Number(1),
            &RequestId::String("1".into())
        ));
    }


    #[test]
    fn wave20_batch_and_error_extractors() {
        let raw = r#"[{"jsonrpc":"2.0","id":1,"method":"ping"},{"jsonrpc":"2.0","method":"notify"}]"#;
        let batch = match parse_batch_messages(raw) {
            Ok(b) => b,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(batch_message_count(&batch), 2);
        assert!(batch_has_request(&batch));
        assert!(batch_has_notification(&batch));
        assert_eq!(
            first_request_id_in_batch(&batch).map(|id| request_id_display(&id)),
            Some("1".into())
        );

        let err_raw = r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"bad"}}"#;
        match parse_message(err_raw) {
            ParseResult::Ok(msg) => {
                assert_eq!(error_code_of_message(&msg), Some(-32600));
                assert_eq!(error_message_of_message(&msg), Some("bad"));
            }
            ParseResult::Err(e) => panic!("{e}"),
        }
        match parse_message(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#) {
            ParseResult::Ok(msg) => {
                assert!(error_code_of_message(&msg).is_none());
                assert!(error_message_of_message(&msg).is_none());
            }
            ParseResult::Err(e) => panic!("{e}"),
        }
    }

    #[test]
    fn wave21_message_matchers_and_batch() {
        let id = RequestId::Number(9);
        let ok = JsonRpcMessage::Success(success(id.clone(), json!({"ok": true})));
        assert!(is_success_for_id(&ok, &id));
        assert!(!is_success_for_id(&ok, &RequestId::Number(1)));
        assert!(!is_error_for_id(&ok, &id));

        let err = JsonRpcMessage::Error(invalid_params(id.clone(), "bad"));
        assert!(is_error_for_id(&err, &id));
        assert_eq!(response_error_code(&err), Some(-32602));
        assert_eq!(response_error_message(&err), Some("bad"));
        assert!(!is_success_for_id(&err, &id));

        let bare = JsonRpcMessage::Request(request(RequestId::Number(1), "ping", None));
        assert!(params_absent_or_empty(&bare));
        let with_empty = JsonRpcMessage::Request(request(
            RequestId::Number(2),
            "tools/list",
            Some(json!({})),
        ));
        assert!(params_absent_or_empty(&with_empty));
        let with_params = JsonRpcMessage::Request(request(
            RequestId::Number(3),
            "tools/call",
            Some(json!({"name": "x"})),
        ));
        assert!(!params_absent_or_empty(&with_params));

        let batch_raw = r#"[
            {"jsonrpc":"2.0","id":1,"method":"ping"},
            {"jsonrpc":"2.0","method":"notifications/initialized"}
        ]"#;
        assert_eq!(batch_payload_message_count(batch_raw), Some(2));
        assert!(batch_payload_message_count(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).is_none());

        let msgs = match parse_batch_messages(batch_raw) {
            Ok(m) => m,
            Err(e) => panic!("{e}"),
        };
        let s = stringify_batch(&msgs);
        assert!(s.starts_with('['));
        assert!(s.contains("ping"));

        assert!(is_request_method(&msgs[0], "ping"));
        assert!(!is_request_method(&msgs[0], "tools/list"));
        assert!(is_notification_method(&msgs[1], "notifications/initialized"));

        let mnf = method_not_found_response(RequestId::Number(5), "nope");
        assert!(is_error(&mnf));
        assert_eq!(response_error_code(&mnf), Some(error_code::METHOD_NOT_FOUND));
        assert_eq!(
            response_error_message(&mnf),
            Some("Method not found: nope")
        );
    }

    #[test]
    fn wave22_batch_and_error_helpers() {
        let id = RequestId::Number(1);
        let inv = invalid_params_response(id.clone(), "bad");
        assert_eq!(response_error_code(&inv), Some(error_code::INVALID_PARAMS));
        let ie = internal_error_response(RequestId::Number(2), "boom");
        assert_eq!(response_error_code(&ie), Some(error_code::INTERNAL_ERROR));

        let req = JsonRpcMessage::Request(request(RequestId::Number(3), "ping", None));
        let ok = JsonRpcMessage::Success(success(RequestId::Number(3), serde_json::json!({})));
        assert!(!batch_all_responses(&[req.clone(), ok.clone()]));
        assert!(batch_all_responses(&[ok.clone(), inv.clone()]));
        assert!(first_error_in_batch(&[ok.clone(), inv.clone()]).is_some());
        assert!(first_error_in_batch(&[ok.clone()]).is_none());

        let note = JsonRpcMessage::Notification(notification(
            "notifications/message",
            Some(serde_json::json!({})),
        ));
        assert_eq!(batch_request_count(&[req, note.clone(), ok.clone()]), 1);
        assert_eq!(batch_notification_count(&[note]), 1);

        let srv = JsonRpcMessage::Error(error_response(
            Some(RequestId::Number(9)),
            error_code::SERVER_ERROR_MIN,
            "x",
            None,
        ));
        assert!(response_is_server_error(&srv));
        assert!(!response_is_server_error(&ok));
    }
}
