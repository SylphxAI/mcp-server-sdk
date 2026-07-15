//! Pure MCP protocol helpers (parity with `src/protocol/mcp.ts` version surface).
//!
//! BW1 pure residual for `api/mcp-protocol` — types + version negotiation only.
//! Full handler dispatch stays TS until a later train.

use serde::{Deserialize, Serialize};

/// Latest protocol version (parity with TS `LATEST_PROTOCOL_VERSION`).
pub const LATEST_PROTOCOL_VERSION: &str = "2025-03-26";

/// Supported protocol versions, newest first.
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-03-26", "2024-11-05"];

/// True when `version` is in the supported set.
#[must_use]
pub fn is_supported_protocol_version(version: &str) -> bool {
    SUPPORTED_PROTOCOL_VERSIONS.contains(&version)
}

/// Negotiate: pick client version if supported, else latest server version.
#[must_use]
pub fn negotiate_protocol_version(client_version: Option<&str>) -> &'static str {
    match client_version {
        Some(v) if is_supported_protocol_version(v) => {
            // Return the static entry matching client (same string content).
            SUPPORTED_PROTOCOL_VERSIONS
                .iter()
                .copied()
                .find(|s| *s == v)
                .unwrap_or(LATEST_PROTOCOL_VERSION)
        }
        _ => LATEST_PROTOCOL_VERSION,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Content {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
    Audio {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
    Resource {
        resource: EmbeddedResource,
        #[serde(skip_serializing_if = "Option::is_none")]
        annotations: Option<ContentAnnotations>,
    },
}

/// Embedded resource payload (parity with MCP ResourceContents text form).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedResource {
    pub uri: String,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

impl Content {
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            annotations: None,
        }
    }

    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text { .. })
    }

    #[must_use]
    pub fn image(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self::Image {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
        }
    }

    #[must_use]
    pub fn audio(data: impl Into<String>, mime_type: impl Into<String>) -> Self {
        Self::Audio {
            data: data.into(),
            mime_type: mime_type.into(),
            annotations: None,
        }
    }

    #[must_use]
    pub fn with_annotations(mut self, annotations: ContentAnnotations) -> Self {
        match &mut self {
            Self::Text { annotations: a, .. }
            | Self::Image { annotations: a, .. }
            | Self::Audio { annotations: a, .. }
            | Self::Resource { annotations: a, .. } => *a = Some(annotations),
        }
        self
    }
}

/// Tool annotations hints (pure data).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_hint: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive_hint: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotent_hint: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_hint: Option<bool>,
}

/// Minimal tool descriptor for list/call envelopes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ToolAnnotations>,
}

/// Build a tools/list result envelope body.
#[must_use]
pub fn tools_list_result(tools: &[Tool], next_cursor: Option<String>) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "tools".into(),
        serde_json::to_value(tools).unwrap_or_else(|_| serde_json::json!([])),
    );
    if let Some(c) = next_cursor {
        map.insert("nextCursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// resources/list result envelope (cursor-aware).
#[must_use]
pub fn resources_list_result(
    resources: &[serde_json::Value],
    next_cursor: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "resources".into(),
        serde_json::Value::Array(resources.to_vec()),
    );
    if let Some(c) = next_cursor {
        map.insert("nextCursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// prompts/list result envelope (cursor-aware).
#[must_use]
pub fn prompts_list_result(
    prompts: &[serde_json::Value],
    next_cursor: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "prompts".into(),
        serde_json::Value::Array(prompts.to_vec()),
    );
    if let Some(c) = next_cursor {
        map.insert("nextCursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// Empty tools/call success (no content items).
#[must_use]
pub fn empty_tool_result() -> serde_json::Value {
    tool_content_result(&[], false)
}

/// Whether a tools/call result body marks `isError: true`.
#[must_use]
pub fn is_tool_error_result(body: &serde_json::Value) -> bool {
    body.get("isError").and_then(|v| v.as_bool()).unwrap_or(false)
}

/// tools/call error result envelope (parity with `toolError`).
#[must_use]
pub fn tool_error(message: impl Into<String>) -> serde_json::Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": message.into() }],
        "isError": true,
    })
}

/// tools/call success with single text content.
#[must_use]
pub fn tool_text_result(message: impl Into<String>, is_error: bool) -> serde_json::Value {
    serde_json::json!({
        "content": [{ "type": "text", "text": message.into() }],
        "isError": is_error,
    })
}

/// JSON pretty text content (parity with builders `json`).
#[must_use]
pub fn json_text_content(value: &serde_json::Value) -> Content {
    Content::text(serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into()))
}

/// Notification constructors (parity with `src/notifications/helpers.ts`) — pure data only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Notification {
    #[serde(rename = "progress")]
    Progress {
        #[serde(rename = "progressToken")]
        progress_token: serde_json::Value,
        progress: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        total: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    #[serde(rename = "log")]
    Log {
        level: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        logger: Option<String>,
        data: serde_json::Value,
    },
    #[serde(rename = "resources/list_changed")]
    ResourcesListChanged {},
    #[serde(rename = "tools/list_changed")]
    ToolsListChanged {},
    #[serde(rename = "prompts/list_changed")]
    PromptsListChanged {},
    #[serde(rename = "resource/updated")]
    ResourceUpdated { uri: String },
    #[serde(rename = "cancelled")]
    Cancelled {
        #[serde(rename = "requestId")]
        request_id: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

#[must_use]
pub fn progress_notification(
    progress_token: serde_json::Value,
    current: f64,
    total: Option<f64>,
    message: Option<String>,
) -> Notification {
    Notification::Progress {
        progress_token,
        progress: current,
        total,
        message,
    }
}

#[must_use]
pub fn tools_list_changed() -> Notification {
    Notification::ToolsListChanged {}
}

#[must_use]
pub fn resources_list_changed() -> Notification {
    Notification::ResourcesListChanged {}
}

#[must_use]
pub fn prompts_list_changed() -> Notification {
    Notification::PromptsListChanged {}
}

#[must_use]
pub fn resource_updated(uri: impl Into<String>) -> Notification {
    Notification::ResourceUpdated { uri: uri.into() }
}

#[must_use]
pub fn cancelled(request_id: serde_json::Value, reason: Option<String>) -> Notification {
    Notification::Cancelled {
        request_id,
        reason,
    }
}

/// Log notification (parity with src/notifications/helpers.ts `log`).
#[must_use]
pub fn log_notification(
    level: impl Into<String>,
    data: serde_json::Value,
    logger: Option<String>,
) -> Notification {
    Notification::Log {
        level: level.into(),
        logger,
        data,
    }
}

// ============================================================================
// Log levels (parity with src/protocol/mcp.ts `LogLevel`)
// ============================================================================

/// MCP log levels, most-verbose first (spec order is not severity-sorted).
pub const LOG_LEVELS: &[&str] = &[
    "debug",
    "info",
    "notice",
    "warning",
    "error",
    "critical",
    "alert",
    "emergency",
];

/// True when `level` is a recognized MCP log level string.
#[must_use]
pub fn is_valid_log_level(level: &str) -> bool {
    LOG_LEVELS.contains(&level)
}

// ============================================================================
// Notification → JSON-RPC (parity with src/notifications/emitter.ts `toJsonRpc`)
// ============================================================================

/// Pure JSON-RPC wire form of a high-level [`Notification`].
///
/// Parity with TS `toJsonRpc` in `src/notifications/emitter.ts` — method name +
/// optional params object. No I/O; transports remain TS product authority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcNotificationWire {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Convert a pure notification shape to MCP JSON-RPC method + params.
#[must_use]
pub fn notification_to_jsonrpc(n: &Notification) -> JsonRpcNotificationWire {
    match n {
        Notification::Progress {
            progress_token,
            progress,
            total,
            message,
        } => {
            let mut map = serde_json::Map::new();
            map.insert("progressToken".into(), progress_token.clone());
            map.insert(
                "progress".into(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(*progress)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );
            if let Some(t) = total {
                if let Some(num) = serde_json::Number::from_f64(*t) {
                    map.insert("total".into(), serde_json::Value::Number(num));
                }
            }
            if let Some(m) = message {
                map.insert("message".into(), serde_json::Value::String(m.clone()));
            }
            JsonRpcNotificationWire {
                method: methods::PROGRESS_NOTIFICATION.into(),
                params: Some(serde_json::Value::Object(map)),
            }
        }
        Notification::Log {
            level,
            logger,
            data,
        } => {
            let mut map = serde_json::Map::new();
            map.insert("level".into(), serde_json::Value::String(level.clone()));
            if let Some(l) = logger {
                map.insert("logger".into(), serde_json::Value::String(l.clone()));
            }
            map.insert("data".into(), data.clone());
            JsonRpcNotificationWire {
                method: methods::LOG_MESSAGE.into(),
                params: Some(serde_json::Value::Object(map)),
            }
        }
        Notification::ResourcesListChanged {} => JsonRpcNotificationWire {
            method: methods::RESOURCES_LIST_CHANGED.into(),
            params: None,
        },
        Notification::ToolsListChanged {} => JsonRpcNotificationWire {
            method: methods::TOOLS_LIST_CHANGED.into(),
            params: None,
        },
        Notification::PromptsListChanged {} => JsonRpcNotificationWire {
            method: methods::PROMPTS_LIST_CHANGED.into(),
            params: None,
        },
        Notification::ResourceUpdated { uri } => JsonRpcNotificationWire {
            method: methods::RESOURCES_UPDATED.into(),
            params: Some(serde_json::json!({ "uri": uri })),
        },
        Notification::Cancelled {
            request_id,
            reason,
        } => {
            let mut map = serde_json::Map::new();
            map.insert("requestId".into(), request_id.clone());
            if let Some(r) = reason {
                map.insert("reason".into(), serde_json::Value::String(r.clone()));
            }
            JsonRpcNotificationWire {
                method: methods::CANCELLED_NOTIFICATION.into(),
                params: Some(serde_json::Value::Object(map)),
            }
        }
    }
}

// ============================================================================
// Tool result normalize (parity with src/builders/tool.ts `normalizeResult`)
// ============================================================================

/// Normalize a tools/call handler result into a `ToolsCallResult` body.
///
/// Accepts:
/// - full result object `{ "content": [...], "isError"?: bool, ... }`
/// - array of content items
/// - single content object
#[must_use]
pub fn normalize_tool_result(result: &serde_json::Value) -> serde_json::Value {
    if let Some(obj) = result.as_object() {
        if obj.get("content").and_then(|c| c.as_array()).is_some() {
            return result.clone();
        }
    }
    if result.is_array() {
        return serde_json::json!({ "content": result });
    }
    // Single content item (or unknown shape → wrap as sole content element).
    serde_json::json!({ "content": [result] })
}

// ============================================================================
// Initialize + remaining list envelopes (pure protocol builders)
// ============================================================================

/// `initialize` result body (parity with `InitializeResult` shape).
#[must_use]
pub fn initialize_result(
    protocol_version: impl Into<String>,
    server_name: impl Into<String>,
    server_version: impl Into<String>,
    capabilities: serde_json::Value,
    instructions: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "protocolVersion".into(),
        serde_json::Value::String(protocol_version.into()),
    );
    map.insert("capabilities".into(), capabilities);
    map.insert(
        "serverInfo".into(),
        serde_json::json!({
            "name": server_name.into(),
            "version": server_version.into(),
        }),
    );
    if let Some(i) = instructions {
        map.insert("instructions".into(), serde_json::Value::String(i));
    }
    serde_json::Value::Object(map)
}

/// Default empty server capabilities object.
#[must_use]
pub fn empty_server_capabilities() -> serde_json::Value {
    serde_json::json!({})
}

/// `resources/templates/list` result envelope.
#[must_use]
pub fn resource_templates_list_result(
    templates: &[serde_json::Value],
    next_cursor: Option<String>,
) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "resourceTemplates".into(),
        serde_json::Value::Array(templates.to_vec()),
    );
    if let Some(c) = next_cursor {
        map.insert("nextCursor".into(), serde_json::Value::String(c));
    }
    serde_json::Value::Object(map)
}

/// `roots/list` result envelope.
#[must_use]
pub fn roots_list_result(roots: &[serde_json::Value]) -> serde_json::Value {
    serde_json::json!({ "roots": roots })
}

/// `completion/complete` result envelope.
#[must_use]
pub fn completion_complete_result(
    values: &[String],
    total: Option<u64>,
    has_more: Option<bool>,
) -> serde_json::Value {
    let mut completion = serde_json::Map::new();
    completion.insert(
        "values".into(),
        serde_json::Value::Array(
            values
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        ),
    );
    if let Some(t) = total {
        completion.insert("total".into(), serde_json::Value::Number(t.into()));
    }
    if let Some(h) = has_more {
        completion.insert("hasMore".into(), serde_json::Value::Bool(h));
    }
    serde_json::json!({ "completion": completion })
}

/// Protocol tool descriptor from name + fields (parity with `toProtocolTool` data shape).
#[must_use]
pub fn protocol_tool(
    name: impl Into<String>,
    description: Option<String>,
    input_schema: serde_json::Value,
    annotations: Option<ToolAnnotations>,
) -> Tool {
    Tool {
        name: name.into(),
        title: None,
        description,
        input_schema,
        annotations,
    }
}

/// Embedded resource content (parity with builders `embedded`).
#[must_use]
pub fn embedded_resource_content(
    uri: impl Into<String>,
    mime_type: Option<String>,
    text: Option<String>,
) -> Content {
    Content::Resource {
        resource: EmbeddedResource {
            uri: uri.into(),
            mime_type,
            text,
        },
        annotations: None,
    }
}

/// resources/read result envelope from one or more embedded resources.
#[must_use]
pub fn resources_read_result(items: &[EmbeddedResource]) -> serde_json::Value {
    serde_json::json!({
        "contents": items,
    })
}

/// Text resource contents (parity with builders `resourceText`).
#[must_use]
pub fn resource_text(
    uri: impl Into<String>,
    text: impl Into<String>,
    mime_type: Option<String>,
) -> EmbeddedResource {
    EmbeddedResource {
        uri: uri.into(),
        mime_type: mime_type.or_else(|| Some("text/plain".into())),
        text: Some(text.into()),
    }
}

/// Blob resource contents — blob stored as text field for pure envelope (no binary I/O).
#[must_use]
pub fn resource_blob(
    uri: impl Into<String>,
    blob_b64: impl Into<String>,
    mime_type: impl Into<String>,
) -> serde_json::Value {
    serde_json::json!({
        "contents": [{
            "uri": uri.into(),
            "mimeType": mime_type.into(),
            "blob": blob_b64.into(),
        }]
    })
}

/// tools/call multi-content result envelope.
#[must_use]
pub fn tool_content_result(content: &[Content], is_error: bool) -> serde_json::Value {
    serde_json::json!({
        "content": content,
        "isError": is_error,
    })
}

/// Create image content (base64) — parity: builders/tool.ts#image.
#[must_use]
pub fn image_content(data: impl Into<String>, mime_type: impl Into<String>) -> Content {
    Content::image(data, mime_type)
}

/// Create audio content (base64) — parity: builders/tool.ts#audio.
#[must_use]
pub fn audio_content(data: impl Into<String>, mime_type: impl Into<String>) -> Content {
    Content::audio(data, mime_type)
}

/// Prompt message with arbitrary content (parity: builders/prompt.ts#message).
#[must_use]
pub fn prompt_message(role: impl Into<String>, content: Content) -> PromptMessage {
    PromptMessage {
        role: role.into(),
        content,
    }
}

/// Pretty-printed JSON as text content (parity: builders/tool.ts#json).
#[must_use]
pub fn json_text_pretty(value: &serde_json::Value) -> Content {
    Content::text(serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".into()))
}


// ============================================================================
// MCP method name constants (parity with src/protocol/mcp.ts Method)
// ============================================================================

pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const INITIALIZED: &str = "notifications/initialized";
    pub const PING: &str = "ping";
    pub const RESOURCES_LIST: &str = "resources/list";
    pub const RESOURCES_TEMPLATES_LIST: &str = "resources/templates/list";
    pub const RESOURCES_READ: &str = "resources/read";
    pub const RESOURCES_SUBSCRIBE: &str = "resources/subscribe";
    pub const RESOURCES_UNSUBSCRIBE: &str = "resources/unsubscribe";
    pub const RESOURCES_UPDATED: &str = "notifications/resources/updated";
    pub const RESOURCES_LIST_CHANGED: &str = "notifications/resources/list_changed";
    pub const PROMPTS_LIST: &str = "prompts/list";
    pub const PROMPTS_GET: &str = "prompts/get";
    pub const PROMPTS_LIST_CHANGED: &str = "notifications/prompts/list_changed";
    pub const TOOLS_LIST: &str = "tools/list";
    pub const TOOLS_CALL: &str = "tools/call";
    pub const TOOLS_LIST_CHANGED: &str = "notifications/tools/list_changed";
    pub const LOGGING_SET_LEVEL: &str = "logging/setLevel";
    pub const LOG_MESSAGE: &str = "notifications/message";
    pub const COMPLETION_COMPLETE: &str = "completion/complete";
    pub const SAMPLING_CREATE_MESSAGE: &str = "sampling/createMessage";
    pub const ELICITATION_CREATE: &str = "elicitation/create";
    pub const PROGRESS_NOTIFICATION: &str = "notifications/progress";
    pub const CANCELLED_NOTIFICATION: &str = "notifications/cancelled";
    pub const ROOTS_LIST: &str = "roots/list";
    pub const ROOTS_LIST_CHANGED: &str = "notifications/roots/list_changed";
}

/// Check if a URI matches a template pattern (`{param}` → single path segment).
#[must_use]
pub fn matches_template(template: &str, uri: &str) -> bool {
    extract_params(template, uri).is_some()
}

/// Extract `{param}` path segments from a URI against a template.
/// Returns None when the URI does not match.
#[must_use]
pub fn extract_params(
    template: &str,
    uri: &str,
) -> Option<std::collections::BTreeMap<String, String>> {
    // Split template into literal/param alternating segments.
    let mut segments: Vec<(bool, String)> = Vec::new(); // (is_param, text)
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        let lit = &rest[..start];
        if !lit.is_empty() {
            segments.push((false, lit.to_string()));
        }
        let after = &rest[start + 1..];
        let end = after.find('}')?;
        let name = &after[..end];
        if name.is_empty() || name.contains('/') {
            return None;
        }
        segments.push((true, name.to_string()));
        rest = &after[end + 1..];
    }
    if !rest.is_empty() {
        segments.push((false, rest.to_string()));
    }

    let mut params = std::collections::BTreeMap::new();
    let mut cursor = 0usize;
    for (i, (is_param, text)) in segments.iter().enumerate() {
        if !is_param {
            let lit = text.as_str();
            if !uri[cursor..].starts_with(lit) {
                return None;
            }
            cursor += lit.len();
        } else {
            // param consumes until next literal or end; must be single segment (no /)
            let next_lit = segments.get(i + 1).and_then(|(ip, t)| if !ip { Some(t.as_str()) } else { None });
            let end = if let Some(lit) = next_lit {
                let relative = uri[cursor..].find(lit)?;
                cursor + relative
            } else {
                uri.len()
            };
            if end < cursor {
                return None;
            }
            let value = &uri[cursor..end];
            if value.is_empty() || value.contains('/') {
                return None;
            }
            params.insert(text.clone(), value.to_string());
            cursor = end;
        }
    }
    if cursor != uri.len() {
        return None;
    }
    Some(params)
}

/// Interpolate `{{key}}` placeholders (prompt template helper).
/// Missing keys remain as `{{key}}`.
#[must_use]
pub fn interpolate(template: &str, args: &std::collections::BTreeMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut i = 0;
    let chars: Vec<char> = template.chars().collect();
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i + 1] == '{' {
            // find closing }}
            let mut j = i + 2;
            while j + 1 < chars.len() {
                if chars[j] == '}' && chars[j + 1] == '}' {
                    break;
                }
                j += 1;
            }
            if j + 1 < chars.len() && chars[j] == '}' && chars[j + 1] == '}' {
                let key: String = chars[i + 2..j].iter().collect();
                if !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    if let Some(val) = args.get(&key) {
                        out.push_str(val);
                    } else {
                        out.push_str("{{");
                        out.push_str(&key);
                        out.push_str("}}");
                    }
                    i = j + 2;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Prompt message helpers (pure builders — no IO).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptMessage {
    pub role: String,
    pub content: Content,
}

#[must_use]
pub fn user_message(text: impl Into<String>) -> PromptMessage {
    PromptMessage {
        role: "user".into(),
        content: Content::text(text),
    }
}

#[must_use]
pub fn assistant_message(text: impl Into<String>) -> PromptMessage {
    PromptMessage {
        role: "assistant".into(),
        content: Content::text(text),
    }
}

#[must_use]
pub fn prompt_messages(msgs: &[PromptMessage]) -> serde_json::Value {
    serde_json::json!({ "messages": msgs })
}

#[must_use]
pub fn prompt_result(description: impl Into<String>, msgs: &[PromptMessage]) -> serde_json::Value {
    serde_json::json!({
        "description": description.into(),
        "messages": msgs,
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    fn array_len(v: &serde_json::Value, key: &str) -> usize {
        v.get(key)
            .and_then(|x| x.as_array())
            .map(|a| a.len())
            .unwrap_or(usize::MAX)
    }

    #[test]
    fn supported_versions() {
        assert!(is_supported_protocol_version("2025-03-26"));
        assert!(is_supported_protocol_version("2024-11-05"));
        assert!(!is_supported_protocol_version("1.0.0"));
    }

    #[test]
    fn negotiate_prefers_client_when_known() {
        assert_eq!(
            negotiate_protocol_version(Some("2024-11-05")),
            "2024-11-05"
        );
        assert_eq!(
            negotiate_protocol_version(Some("nope")),
            LATEST_PROTOCOL_VERSION
        );
        assert_eq!(negotiate_protocol_version(None), LATEST_PROTOCOL_VERSION);
    }

    #[test]
    fn text_content_roundtrip() {
        let c = Content::text("hello");
        assert!(c.is_text());
        let v = match serde_json::to_value(&c) {
            Ok(v) => v,
            Err(e) => panic!("serialize: {e}"),
        };
        assert_eq!(v["type"], "text");
        assert_eq!(v["text"], "hello");
    }

    #[test]
    fn tools_list_with_cursor() {
        let tools = [Tool {
            name: "ping".into(),
            title: None,
            description: Some("p".into()),
            input_schema: serde_json::json!({"type": "object"}),
            annotations: None,
        }];
        let body = tools_list_result(&tools, Some("abc".into()));
        assert_eq!(body["tools"][0]["name"], "ping");
        assert_eq!(body["nextCursor"], "abc");
    }

    #[test]
    fn content_constructors_and_tool_error() {
        let t = Content::text("hi");
        assert!(t.is_text());
        let img = Content::image("AAAA", "image/png");
        assert!(matches!(img, Content::Image { .. }));
        let err = tool_error("boom");
        assert_eq!(err["isError"], true);
        assert_eq!(err["content"][0]["text"], "boom");
        let jt = json_text_content(&serde_json::json!({"a": 1}));
        assert!(jt.is_text());
    }

    #[test]
    fn notification_helpers() {
        let n = tools_list_changed();
        assert!(matches!(n, Notification::ToolsListChanged {}));
        let p = progress_notification(serde_json::json!(1), 0.5, Some(1.0), None);
        assert!(matches!(p, Notification::Progress { .. }));
        let c = cancelled(serde_json::json!("id-1"), Some("stop".into()));
        assert!(matches!(c, Notification::Cancelled { .. }));
    }

    #[test]
    fn template_match_and_extract() {
        // {param} matches a single path segment (TS [^/]+)
        assert!(matches_template("file:///{path}", "file:///tmp"));
        assert!(!matches_template("file:///{path}", "file:///tmp/a"));
        let p = match extract_params("file:///{bucket}/{key}", "file:///b1/obj") {
            Some(p) => p,
            None => panic!("expected params"),
        };
        assert_eq!(p.get("bucket").map(String::as_str), Some("b1"));
        assert_eq!(p.get("key").map(String::as_str), Some("obj"));
        assert!(extract_params("file:///{bucket}/{key}", "file:///b1/obj/extra").is_none());
        assert!(matches_template("static", "static"));
        assert!(!matches_template("static", "static2"));
        let p2 = match extract_params("/users/{id}/files/{name}", "/users/42/files/a.txt") {
            Some(p) => p,
            None => panic!("expected params"),
        };
        assert_eq!(p2.get("id").map(String::as_str), Some("42"));
        assert_eq!(p2.get("name").map(String::as_str), Some("a.txt"));
    }

    #[test]
    fn interpolate_placeholders() {
        let mut args = std::collections::BTreeMap::new();
        args.insert("name".into(), "Ada".into());
        assert_eq!(interpolate("Hello {{name}}!", &args), "Hello Ada!");
        assert_eq!(interpolate("Hi {{missing}}", &args), "Hi {{missing}}");
        assert_eq!(interpolate("nope", &args), "nope");
    }

    #[test]
    fn prompt_message_helpers() {
        let u = user_message("hi");
        assert_eq!(u.role, "user");
        let a = assistant_message("yo");
        assert_eq!(a.role, "assistant");
        let body = prompt_result("desc", &[u, a]);
        assert_eq!(body["description"], "desc");
        assert_eq!(array_len(&body, "messages"), 2);
        assert_eq!(methods::TOOLS_CALL, "tools/call");
        assert_eq!(methods::INITIALIZE, "initialize");
    }

    #[test]
    fn wave5_log_and_embedded() {
        let n = log_notification("info", serde_json::json!({"ok": true}), Some("app".into()));
        assert!(matches!(n, Notification::Log { .. }));
        let c = embedded_resource_content("file:///x", Some("text/plain".into()), Some("hi".into()));
        assert!(matches!(c, Content::Resource { .. }));
        if let Content::Resource { resource, .. } = c {
            assert_eq!(resource.uri, "file:///x");
            assert_eq!(resource.text.as_deref(), Some("hi"));
        }
    }
    #[test]
    fn wave6_resource_and_multi_content() {
        let rt = resource_text("file:///a", "hello", None);
        assert_eq!(rt.uri, "file:///a");
        assert_eq!(rt.text.as_deref(), Some("hello"));
        assert_eq!(rt.mime_type.as_deref(), Some("text/plain"));
        let rr = resources_read_result(&[rt]);
        assert_eq!(array_len(&rr, "contents"), 1);
        let blob = resource_blob("file:///b", "AAAA", "image/png");
        assert_eq!(blob["contents"][0]["blob"], "AAAA");
        let contents = [
            Content::text("t"),
            Content::image("AA", "image/png"),
            Content::audio("BB", "audio/wav"),
        ];
        let body = tool_content_result(&contents, false);
        assert_eq!(body["isError"], false);
        assert_eq!(array_len(&body, "content"), 3);
    }

    #[test]
    fn wave7_media_prompt_json() {
        // FLEET-ENTERPRISE-WAVE7
        let img = image_content("AAAA", "image/png");
        assert!(matches!(img, Content::Image { .. }));
        let aud = audio_content("BBBB", "audio/wav");
        assert!(matches!(aud, Content::Audio { .. }));
        let msg = prompt_message("user", Content::text("hi"));
        assert_eq!(msg.role, "user");
        let pretty = json_text_pretty(&serde_json::json!({"a": 1}));
        match pretty {
            Content::Text { text, .. } => assert!(text.contains("\"a\"")),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn wave8_list_envelopes_and_error_flag() {
        // FLEET-ENTERPRISE-WAVE8
        let rr = resources_list_result(
            &[serde_json::json!({"uri": "file:///a", "name": "a"})],
            Some("c1".into()),
        );
        assert_eq!(array_len(&rr, "resources"), 1);
        assert_eq!(rr["nextCursor"], "c1");
        let pr = prompts_list_result(&[], None);
        assert_eq!(array_len(&pr, "prompts"), 0);
        assert!(pr.get("nextCursor").is_none());
        let empty = empty_tool_result();
        assert_eq!(empty["isError"], false);
        assert_eq!(array_len(&empty, "content"), 0);
        assert!(!is_tool_error_result(&empty));
        assert!(is_tool_error_result(&tool_error("boom")));
    }

    #[test]
    fn bulk_template_multi_segment_and_reject_extra() {
        assert!(!matches_template(
            "/users/{id}/posts/{pid}",
            "/users/1/posts/2/extra"
        ));
        let p = match extract_params("/users/{id}/posts/{pid}", "/users/1/posts/2") {
            Some(p) => p,
            None => panic!("expected params"),
        };
        assert_eq!(p.get("id").map(String::as_str), Some("1"));
        assert_eq!(p.get("pid").map(String::as_str), Some("2"));
        assert!(extract_params("/users/{id}", "/users/a/b").is_none());
        assert!(extract_params("/users/{id}", "/users/").is_none());
    }

    #[test]
    fn bulk_interpolate_missing_and_valid() {
        let mut args = std::collections::BTreeMap::new();
        args.insert("name".into(), "Ada".into());
        assert_eq!(interpolate("hi {{name}}", &args), "hi Ada");
        assert_eq!(interpolate("hi {{missing}}", &args), "hi {{missing}}");
    }

    #[test]
    fn bulk_notifications_cancelled_and_progress() {
        let n = cancelled(serde_json::json!(1), Some("stop".into()));
        assert!(matches!(n, Notification::Cancelled { .. }));
        let p = progress_notification(
            serde_json::json!("t1"),
            1.0,
            Some(10.0),
            Some("working".into()),
        );
        assert!(matches!(p, Notification::Progress { .. }));
        assert!(!is_tool_error_result(&empty_tool_result()));
        assert!(is_tool_error_result(&tool_text_result("x", true)));
        assert!(!is_tool_error_result(&tool_text_result("x", false)));
    }

    #[test]
    fn bulk_list_results_without_cursor_omit_next() {
        let tools = tools_list_result(&[], None);
        let next = tools.get("nextCursor");
        assert!(next.is_none() || next.is_some_and(|v| v.is_null()));
        let tools2 = tools_list_result(&[], Some("c".into()));
        assert_eq!(tools2.get("nextCursor").and_then(|v| v.as_str()), Some("c"));
    }

    /// SHA-bound pure residual golden: protocol version + tool error envelope.
    #[test]
    fn pure_residual_protocol_golden_fixture() {
        let raw = include_str!("../fixtures/protocol_golden.json");
        let doc: serde_json::Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => panic!("protocol_golden.json: {e}"),
        };
        assert_eq!(doc["schema"], "mcp-protocol-golden/v1");
        assert_eq!(doc["latestProtocolVersion"], LATEST_PROTOCOL_VERSION);
        let supported = match doc["supportedProtocolVersions"].as_array() {
            Some(a) => a,
            None => panic!("supportedProtocolVersions"),
        };
        for v in supported {
            let s = match v.as_str() {
                Some(s) => s,
                None => panic!("version not string"),
            };
            assert!(is_supported_protocol_version(s));
        }
        for case in match doc["negotiate"].as_array() {
            Some(a) => a,
            None => panic!("negotiate"),
        } {
            let client = case.get("client").and_then(|v| v.as_str());
            let expected = match case["expected"].as_str() {
                Some(s) => s,
                None => panic!("expected"),
            };
            assert_eq!(negotiate_protocol_version(client), expected);
        }
        let err = tool_error("fixture-boom");
        assert!(is_tool_error_result(&err));
        assert_eq!(err["content"][0]["text"], "fixture-boom");
        assert!(!is_tool_error_result(&empty_tool_result()));

        // WAVE10: notification → JSON-RPC method mapping golden
        if let Some(cases) = doc.get("notificationToJsonRpc").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let ntype = match case["notificationType"].as_str() {
                    Some(s) => s,
                    None => panic!("notificationType for {name}"),
                };
                let n = match ntype {
                    "tools/list_changed" => tools_list_changed(),
                    "resources/list_changed" => resources_list_changed(),
                    "prompts/list_changed" => prompts_list_changed(),
                    "resource/updated" => {
                        let uri = case["uri"].as_str().unwrap_or("file:///x");
                        resource_updated(uri)
                    }
                    "progress" => progress_notification(
                        case.get("progressToken")
                            .cloned()
                            .unwrap_or(serde_json::json!(1)),
                        case["progress"].as_f64().unwrap_or(0.0),
                        case.get("total").and_then(|v| v.as_f64()),
                        case.get("message")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    ),
                    "log" => log_notification(
                        case["level"].as_str().unwrap_or("info"),
                        case.get("data").cloned().unwrap_or(serde_json::json!({})),
                        case.get("logger")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    ),
                    "cancelled" => cancelled(
                        case.get("requestId")
                            .cloned()
                            .unwrap_or(serde_json::json!(1)),
                        case.get("reason")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    ),
                    other => panic!("unknown notificationType {other} in {name}"),
                };
                let wire = notification_to_jsonrpc(&n);
                let expected_method = match case["expectedMethod"].as_str() {
                    Some(s) => s,
                    None => panic!("expectedMethod for {name}"),
                };
                assert_eq!(wire.method, expected_method, "case {name}");
                let expect_params = case["expectParams"].as_bool().unwrap_or(false);
                assert_eq!(wire.params.is_some(), expect_params, "case {name} params");
            }
        }

        // WAVE10: normalize_tool_result golden
        if let Some(cases) = doc.get("normalizeToolResult").and_then(|v| v.as_array()) {
            for case in cases {
                let name = case["name"].as_str().unwrap_or("?");
                let input = match case.get("input") {
                    Some(v) => v,
                    None => panic!("input for {name}"),
                };
                let out = normalize_tool_result(input);
                let expected_len = case["expectedContentLen"].as_u64().unwrap_or(0) as usize;
                assert_eq!(array_len(&out, "content"), expected_len, "case {name}");
            }
        }

        // WAVE10: log levels
        if let Some(levels) = doc.get("logLevels").and_then(|v| v.as_array()) {
            for v in levels {
                let s = match v.as_str() {
                    Some(s) => s,
                    None => panic!("log level not string"),
                };
                assert!(is_valid_log_level(s), "level {s}");
            }
        }
        assert!(!is_valid_log_level("trace"));
    }

    #[test]
    fn wave10_notification_to_jsonrpc_methods() {
        let wire = notification_to_jsonrpc(&tools_list_changed());
        assert_eq!(wire.method, methods::TOOLS_LIST_CHANGED);
        assert!(wire.params.is_none());

        let wire = notification_to_jsonrpc(&resource_updated("file:///a"));
        assert_eq!(wire.method, methods::RESOURCES_UPDATED);
        assert_eq!(
            wire.params.as_ref().and_then(|p| p.get("uri")).and_then(|u| u.as_str()),
            Some("file:///a")
        );

        let wire = notification_to_jsonrpc(&progress_notification(
            serde_json::json!("tok"),
            3.0,
            Some(10.0),
            Some("working".into()),
        ));
        assert_eq!(wire.method, methods::PROGRESS_NOTIFICATION);
        let p = match wire.params.as_ref() {
            Some(p) => p,
            None => panic!("expected params"),
        };
        assert_eq!(p["progressToken"], "tok");
        assert_eq!(p["progress"], 3.0);
        assert_eq!(p["total"], 10.0);
        assert_eq!(p["message"], "working");

        let wire = notification_to_jsonrpc(&log_notification(
            "warning",
            serde_json::json!({"x": 1}),
            Some("svc".into()),
        ));
        assert_eq!(wire.method, methods::LOG_MESSAGE);
        assert_eq!(
            wire.params.as_ref().and_then(|p| p.get("level")).and_then(|l| l.as_str()),
            Some("warning")
        );

        let wire = notification_to_jsonrpc(&cancelled(serde_json::json!(42), Some("bye".into())));
        assert_eq!(wire.method, methods::CANCELLED_NOTIFICATION);
        assert_eq!(
            wire.params.as_ref().and_then(|p| p.get("requestId")),
            Some(&serde_json::json!(42))
        );
    }

    #[test]
    fn wave10_normalize_tool_result_shapes() {
        let full = serde_json::json!({
            "content": [{"type": "text", "text": "a"}],
            "isError": false
        });
        let n = normalize_tool_result(&full);
        assert_eq!(array_len(&n, "content"), 1);
        assert_eq!(n["isError"], false);

        let arr = serde_json::json!([
            {"type": "text", "text": "a"},
            {"type": "text", "text": "b"}
        ]);
        let n = normalize_tool_result(&arr);
        assert_eq!(array_len(&n, "content"), 2);

        let single = serde_json::json!({"type": "text", "text": "solo"});
        let n = normalize_tool_result(&single);
        assert_eq!(array_len(&n, "content"), 1);
        assert_eq!(n["content"][0]["text"], "solo");
    }

    #[test]
    fn wave10_initialize_and_list_envelopes() {
        let init = initialize_result(
            LATEST_PROTOCOL_VERSION,
            "test-server",
            "0.1.0",
            empty_server_capabilities(),
            Some("hi".into()),
        );
        assert_eq!(init["protocolVersion"], LATEST_PROTOCOL_VERSION);
        assert_eq!(init["serverInfo"]["name"], "test-server");
        assert_eq!(init["instructions"], "hi");

        let tpl = resource_templates_list_result(
            &[serde_json::json!({"uriTemplate": "file:///{p}", "name": "f"})],
            Some("c".into()),
        );
        assert_eq!(array_len(&tpl, "resourceTemplates"), 1);
        assert_eq!(tpl["nextCursor"], "c");

        let roots = roots_list_result(&[serde_json::json!({"uri": "file:///"})]);
        assert_eq!(array_len(&roots, "roots"), 1);

        let comp = completion_complete_result(&["a".into(), "b".into()], Some(2), Some(false));
        assert_eq!(comp["completion"]["values"][0], "a");
        assert_eq!(comp["completion"]["total"], 2);
        assert_eq!(comp["completion"]["hasMore"], false);

        assert!(is_valid_log_level("error"));
        assert!(!is_valid_log_level("verbose"));

        let tool = protocol_tool(
            "ping",
            Some("p".into()),
            serde_json::json!({"type": "object"}),
            None,
        );
        assert_eq!(tool.name, "ping");
        let body = tools_list_result(&[tool], None);
        assert_eq!(body["tools"][0]["name"], "ping");
    }
}
